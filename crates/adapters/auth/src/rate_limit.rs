//! # Per-IP rate limiter + lockout for credential endpoints
//!
//! Implements QW-8 (ADAPT-AUTH-007). The [`RateLimiter`] tracks
//! failed attempts per source IP and engages a temporary
//! lockout after the configured threshold is exceeded. It is
//! intended to be shared (via `Arc<RateLimiter>`) across all
//! handler threads for the auth endpoints:
//!
//! - `/login`
//! - `/register`
//! - `/forgot-password`
//! - `/reset-password`
//!
//! ## Algorithm
//!
//! - **Per-minute cap:** at most
//!   [`RateLimiterConfig::per_minute_limit`] attempts per IP per
//!   rolling [`RateLimiterConfig::window`]. Beyond that, the
//!   caller sees [`RateLimitError::RateLimited`] with a
//!   `retry_after` hint.
//! - **Lockout:** after [`RateLimiterConfig::lockout_threshold`]
//!   consecutive failures inside one window, the IP is locked
//!   for [`RateLimiterConfig::lockout_duration`]. The caller sees
//!   [`RateLimitError::LockedOut`] with the `until` instant.
//! - **Reset:** a successful [`RateLimiter::record_success`]
//!   clears the record for that IP. A locked-out IP can also be
//!   unlocked by waiting for `locked_until` to elapse (a
//!   successful `check` clears the lockout once expired).
//!
//! ## Concurrency
//!
//! The state is held behind a single `Mutex<HashMap<...>>`. The
//! auth crate uses the same `Mutex<HashSet<...>>` pattern for
//! its revocation store, so the cost model is consistent. For
//! a multi-replica deployment the engine layers a shared store
//! on top (see ADR-019 for the planned Redis-backed adapter);
//! this in-memory implementation is the documented
//! single-process baseline.
//!
//! ## Time abstraction
//!
//! All wall-clock reads go through the [`Clock`] trait so tests
//! can advance time deterministically via [`MockClock`]
//! (`#[cfg(test)]` only). Production code uses [`SystemClock`].
//!
//! ## Configuration
//!
//! All thresholds are read from the environment at construction
//! time via [`RateLimiterConfig::from_env`]. The supported
//! variables are:
//!
//! | Variable | Default | Notes |
//! | --- | --- | --- |
//! | `AUTH_RATE_LIMIT_PER_MIN` | `5` | Max attempts per IP per window |
//! | `AUTH_LOCKOUT_THRESHOLD` | `10` | Failed attempts before lockout |
//! | `AUTH_RATE_LIMIT_WINDOW_SECS` | `60` | Window length in seconds |
//! | `AUTH_LOCKOUT_DURATION_MIN` | `15` | Lockout length in minutes |

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[cfg(test)]
use std::sync::Arc;

use crate::errors::AuthError;

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

/// A wall-clock abstraction used by [`RateLimiter`] so tests can
/// drive time without sleeping.
///
/// Production code uses [`SystemClock`]; tests use
/// [`MockClock`]. The trait is object-safe and `Send + Sync +
/// 'static` so a `Box<dyn Clock>` can be stored inside the
/// limiter and shared across handler threads.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current instant. Implementations must be
    /// monotonic (no backward jumps).
    fn now(&self) -> Instant;
}

/// Wall-clock implementation of [`Clock`]. Used by
/// [`RateLimiter::new`] and [`RateLimiter::with_config`] when
/// no clock is supplied.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Test-only mock clock. Holds a mutable `Instant` that
/// `advance` mutates and `now` reads. The clock is `Clone` (the
/// inner state lives in an `Arc<Mutex<_>>`) so a test can keep
/// one handle and hand a clone to the limiter; both see the
/// same advancing time.
///
/// `#[cfg(test)]` only — not compiled into release builds.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockClock {
    inner: Arc<Mutex<Instant>>,
}

#[cfg(test)]
impl MockClock {
    /// Constructs a mock clock starting at the given instant.
    #[must_use]
    pub fn new(start: Instant) -> Self {
        Self {
            inner: Arc::new(Mutex::new(start)),
        }
    }

    /// Advances the clock by the given duration. Saturation
    /// arithmetic is used: an overflow pins the clock at its
    /// current value rather than panicking (this is test code,
    /// so we want robust behaviour over panics on absurd
    /// inputs).
    pub fn advance(&self, by: Duration) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = *guard;
        // Saturation fallback for absurd inputs in tests: pin
        // the clock at its current value rather than panicking.
        *guard = now.checked_add(by).unwrap_or(now);
    }
}

#[cfg(test)]
impl Clock for MockClock {
    fn now(&self) -> Instant {
        *self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned by [`RateLimiter::check`].
#[derive(Debug, PartialEq, Eq)]
pub enum RateLimitError {
    /// The IP exceeded the per-window attempt cap. The wrapped
    /// duration is the time until the window resets (an
    /// advisory `retry_after` for HTTP responses).
    RateLimited {
        /// Time until the current window resets.
        retry_after: Duration,
    },

    /// The IP is locked out after exceeding the failure
    /// threshold. The wrapped instant is when the lockout
    /// expires.
    LockedOut {
        /// When the lockout ends.
        until: Instant,
    },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RateLimited { retry_after } => {
                write!(f, "rate-limited; retry after {retry_after:?}")
            }
            Self::LockedOut { until } => {
                write!(f, "locked out until {until:?}")
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Conversion from [`RateLimitError`] into the crate's
/// universal [`AuthError`]. The conversion maps both variants
/// to [`AuthError::RateLimited`] so existing error-handling
/// middleware that already pattern-matches on
/// [`AuthError::RateLimited`] keeps working. The richer
/// `RateLimitError` is preserved for callers that want the
/// `retry_after` / `until` detail.
impl From<RateLimitError> for AuthError {
    fn from(_err: RateLimitError) -> Self {
        AuthError::RateLimited
    }
}

// ---------------------------------------------------------------------------
// AttemptRecord
// ---------------------------------------------------------------------------

/// Per-IP state tracked by [`RateLimiter`].
#[derive(Debug, Clone)]
pub struct AttemptRecord {
    /// Number of attempts (successes + failures) in the
    /// current window.
    pub count: u32,
    /// Start of the current rolling window.
    pub window_start: Instant,
    /// If `Some(t)`, the IP is locked out until instant `t`.
    pub locked_until: Option<Instant>,
}

impl AttemptRecord {
    fn new(now: Instant) -> Self {
        Self {
            count: 0,
            window_start: now,
            locked_until: None,
        }
    }
}

// ---------------------------------------------------------------------------
// RateLimiterConfig
// ---------------------------------------------------------------------------

/// Tunables for [`RateLimiter`]. Constructed either explicitly
/// via [`RateLimiterConfig::new`] or from environment variables
/// via [`RateLimiterConfig::from_env`].
///
/// All thresholds are read **once** at construction time (the
/// env-reading helpers cache into `OnceLock`s so repeated
/// [`RateLimiterConfig::new`] calls don't reparse the env on
/// every call). If a consumer needs hot-reload of these values
/// they should construct new [`RateLimiter`] instances rather
/// than mutating the existing config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimiterConfig {
    /// Max attempts per IP per `window` before the IP is
    /// rate-limited.
    pub per_minute_limit: u32,
    /// Failed attempts in one window that engage the lockout.
    pub lockout_threshold: u32,
    /// Rolling window length.
    pub window: Duration,
    /// Lockout length once the threshold is hit.
    pub lockout_duration: Duration,
}

impl RateLimiterConfig {
    /// Constructs a config with explicit values. No env reads.
    #[must_use]
    pub fn new(
        per_minute_limit: u32,
        lockout_threshold: u32,
        window: Duration,
        lockout_duration: Duration,
    ) -> Self {
        Self {
            per_minute_limit,
            lockout_threshold,
            window,
            lockout_duration,
        }
    }

    /// Reads config from env vars. Each variable is read once
    /// (cached in a [`OnceLock`]) so repeated
    /// [`RateLimiterConfig::new`] calls during process startup
    /// don't re-read the env.
    ///
    /// - `AUTH_RATE_LIMIT_PER_MIN` (default `5`)
    /// - `AUTH_LOCKOUT_THRESHOLD` (default `10`)
    /// - `AUTH_RATE_LIMIT_WINDOW_SECS` (default `60`)
    /// - `AUTH_LOCKOUT_DURATION_MIN` (default `15`)
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(
            per_minute_limit(),
            lockout_threshold(),
            Duration::from_secs(u64::from(window_secs())),
            Duration::from_secs(u64::from(lockout_duration_min()) * 60),
        )
    }
}

// ---------------------------------------------------------------------------
// One-shot env readers (cached via OnceLock)
// ---------------------------------------------------------------------------

fn per_minute_limit() -> u32 {
    static VALUE: OnceLock<u32> = OnceLock::new();
    *VALUE.get_or_init(|| parse_env_u32("AUTH_RATE_LIMIT_PER_MIN", 5))
}

fn lockout_threshold() -> u32 {
    static VALUE: OnceLock<u32> = OnceLock::new();
    *VALUE.get_or_init(|| parse_env_u32("AUTH_LOCKOUT_THRESHOLD", 10))
}

fn window_secs() -> u32 {
    static VALUE: OnceLock<u32> = OnceLock::new();
    *VALUE.get_or_init(|| parse_env_u32("AUTH_RATE_LIMIT_WINDOW_SECS", 60))
}

fn lockout_duration_min() -> u32 {
    static VALUE: OnceLock<u32> = OnceLock::new();
    *VALUE.get_or_init(|| parse_env_u32("AUTH_LOCKOUT_DURATION_MIN", 15))
}

fn parse_env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

/// Per-IP rate limiter and lockout for credential endpoints.
///
/// Stores attempt counts keyed by source IP behind a single
/// `Mutex<HashMap<...>>` and gates access through the [`Clock`]
/// trait so tests can drive time deterministically.
///
/// Cloning a [`RateLimiter`] is intentional: the inner state
/// lives behind `Arc<...>` (via the `Mutex`) so all clones share
/// the same counter map. Wrap a freshly-constructed limiter in
/// `Arc<RateLimiter>` and clone the `Arc` across handler
/// threads.
pub struct RateLimiter {
    state: Mutex<HashMap<IpAddr, AttemptRecord>>,
    config: RateLimiterConfig,
    clock: Box<dyn Clock>,
}

impl RateLimiter {
    /// Constructs a [`RateLimiter`] with config read from env
    /// vars and the [`SystemClock`] as the time source.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RateLimiterConfig::from_env())
    }

    /// Constructs a [`RateLimiter`] with explicit config and
    /// the [`SystemClock`] as the time source.
    #[must_use]
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self::with_clock(config, Box::new(SystemClock))
    }

    /// Constructs a [`RateLimiter`] with explicit config and a
    /// custom [`Clock`] (used by tests with [`MockClock`]).
    #[must_use]
    pub fn with_clock(config: RateLimiterConfig, clock: Box<dyn Clock>) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            config,
            clock,
        }
    }

    /// Returns the active config. Useful for logging and for
    /// callers that want to surface the current thresholds to
    /// an admin endpoint.
    #[must_use]
    pub fn config(&self) -> RateLimiterConfig {
        self.config
    }

    /// Checks whether the given IP is allowed to make another
    /// request right now. Returns `Ok(())` if the request is
    /// allowed, otherwise the appropriate [`RateLimitError`].
    ///
    /// This call **does not** increment any counters — it only
    /// inspects state. Callers should call
    /// [`RateLimiter::record_failure`] on a failed attempt and
    /// [`RateLimiter::record_success`] on a successful one to
    /// keep the limiter's view of the world in sync with
    /// reality.
    pub fn check(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = self.clock.now();

        let Some(record) = state.get_mut(&ip) else {
            return Ok(());
        };

        // Lockout takes precedence over the rate-limit window:
        // if the IP is currently locked out, reject even before
        // counting attempts.
        if let Some(until) = record.locked_until {
            if until > now {
                return Err(RateLimitError::LockedOut { until });
            }
            // Lockout expired: clear it and reset the window so
            // the IP gets a fresh start.
            record.locked_until = None;
            record.count = 0;
            record.window_start = now;
        }

        // Window expired: reset the count and start a fresh
        // window.
        if now.saturating_duration_since(record.window_start) >= self.config.window {
            record.count = 0;
            record.window_start = now;
        }

        if record.count >= self.config.per_minute_limit {
            let elapsed = now.saturating_duration_since(record.window_start);
            let retry_after = self.config.window.saturating_sub(elapsed);
            return Err(RateLimitError::RateLimited { retry_after });
        }

        Ok(())
    }

    /// Records a failed attempt for the given IP. Increments
    /// the in-window count; engages the lockout if the count
    /// crosses [`RateLimiterConfig::lockout_threshold`].
    pub fn record_failure(&self, ip: IpAddr) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = self.clock.now();
        let record = state.entry(ip).or_insert_with(|| AttemptRecord::new(now));

        // If the lockout expired since the last check, clear
        // it so we count failures fresh.
        if let Some(until) = record.locked_until {
            if until <= now {
                record.locked_until = None;
                record.count = 0;
                record.window_start = now;
            }
        }

        // Window expired: reset count.
        if now.saturating_duration_since(record.window_start) >= self.config.window {
            record.count = 0;
            record.window_start = now;
        }

        record.count = record.count.saturating_add(1);

        if record.count >= self.config.lockout_threshold
            && record.locked_until.is_none()
        {
            record.locked_until = Some(
                now.checked_add(self.config.lockout_duration)
                    .unwrap_or(now),
            );
        }
    }

    /// Records a successful attempt for the given IP. Clears
    /// the record (the IP gets a fresh window and any in-effect
    /// lockout is removed).
    pub fn record_success(&self, ip: IpAddr) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.remove(&ip);
    }

    /// Removes all per-IP state. Useful for tests and admin
    /// endpoints that need to reset the limiter.
    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_docs_in_private_items
)]
mod tests {
    use super::*;

    /// Constructs a limiter whose clock is a fresh [`MockClock`]
    /// shared between the limiter and the returned handle. The
    /// test advances the handle to drive the limiter's view of
    /// time.
    fn limiter_with(
        per_minute: u32,
        threshold: u32,
        window: Duration,
        lockout: Duration,
    ) -> (RateLimiter, MockClock) {
        let clock = MockClock::new(Instant::now());
        let limiter = RateLimiter::with_clock(
            RateLimiterConfig::new(per_minute, threshold, window, lockout),
            Box::new(clock.clone()),
        );
        (limiter, clock)
    }

    fn ip() -> IpAddr {
        "192.0.2.1".parse().expect("ip parses")
    }

    fn ip2() -> IpAddr {
        "198.51.100.7".parse().expect("ip parses")
    }

    #[test]
    fn test_rate_limiter_allows_up_to_n_then_blocks() {
        // Limit = 3 per minute.
        let (limiter, _clock) = limiter_with(
            3,
            100,
            Duration::from_secs(60),
            Duration::from_secs(900),
        );
        let me = ip();

        // First 3 attempts pass.
        for i in 0..3 {
            limiter
                .check(me)
                .unwrap_or_else(|e| panic!("attempt {i} should pass: {e:?}"));
            limiter.record_failure(me);
        }
        // 4th is blocked.
        let err = limiter
            .check(me)
            .expect_err("4th attempt must be rate-limited");
        assert!(
            matches!(err, RateLimitError::RateLimited { .. }),
            "expected RateLimited, got {err:?}"
        );
    }

    #[test]
    fn test_rate_limiter_resets_after_window_expires() {
        let (limiter, clock) = limiter_with(
            2,
            100,
            Duration::from_secs(60),
            Duration::from_secs(900),
        );
        let me = ip();

        // Hit the limit.
        limiter.check(me).expect("1st passes");
        limiter.record_failure(me);
        limiter.check(me).expect("2nd passes");
        limiter.record_failure(me);
        limiter
            .check(me)
            .expect_err("3rd must be rate-limited");

        // Advance past the window.
        clock.advance(Duration::from_secs(61));

        // After the window expires the counter resets.
        limiter
            .check(me)
            .expect("after window expires the IP is unblocked");
    }

    #[test]
    fn test_rate_limiter_engages_lockout_after_threshold() {
        let (limiter, _clock) = limiter_with(
            100, // per-minute limit high enough to not fire
            3,   // lockout threshold
            Duration::from_secs(60),
            Duration::from_secs(900),
        );
        let me = ip();

        // 3 failed attempts.
        for _ in 0..3 {
            limiter.record_failure(me);
        }

        // Now check returns LockedOut.
        let err = limiter.check(me).expect_err("must be locked out");
        assert!(
            matches!(err, RateLimitError::LockedOut { .. }),
            "expected LockedOut, got {err:?}"
        );
    }

    #[test]
    fn test_rate_limiter_success_clears_record() {
        let (limiter, _clock) = limiter_with(
            2,
            100,
            Duration::from_secs(60),
            Duration::from_secs(900),
        );
        let me = ip();

        limiter.check(me).expect("1st passes");
        limiter.record_failure(me);
        limiter.check(me).expect("2nd passes");
        limiter.record_failure(me);

        // Now blocked.
        limiter
            .check(me)
            .expect_err("3rd must be rate-limited");

        // A successful attempt clears the record.
        limiter.record_success(me);

        // Fresh attempts are allowed again.
        limiter
            .check(me)
            .expect("after success the IP is unblocked");
    }

    #[test]
    fn test_rate_limiter_is_per_ip() {
        // One IP hitting the limit must not affect another IP.
        let (limiter, _clock) = limiter_with(
            2,
            100,
            Duration::from_secs(60),
            Duration::from_secs(900),
        );
        let a = ip();
        let b = ip2();

        limiter.check(a).expect("a 1st passes");
        limiter.record_failure(a);
        limiter.check(a).expect("a 2nd passes");
        limiter.record_failure(a);
        limiter.check(a).expect_err("a 3rd must be rate-limited");

        // b is unaffected.
        limiter
            .check(b)
            .expect("b is on a fresh counter, must pass");
    }

    #[test]
    fn test_rate_limiter_lockout_expires() {
        let (limiter, clock) = limiter_with(
            100,
            2,
            Duration::from_secs(60),
            Duration::from_secs(60), // 1-minute lockout
        );
        let me = ip();

        limiter.record_failure(me);
        limiter.record_failure(me);

        // Locked.
        let err = limiter.check(me).expect_err("must be locked");
        assert!(matches!(err, RateLimitError::LockedOut { .. }));

        // Advance past the lockout.
        clock.advance(Duration::from_secs(61));

        // Now allowed again (and the lockout has been cleared).
        limiter
            .check(me)
            .expect("after lockout expires, IP is unblocked");
    }

    #[test]
    fn test_rate_limiter_from_env_defaults() {
        let cfg = RateLimiterConfig::from_env();
        // We don't assert specific env-var values (the test
        // process may or may not have them set); just assert
        // the defaults are reasonable if the env is empty.
        assert!(cfg.per_minute_limit > 0);
        assert!(cfg.lockout_threshold > 0);
        assert!(cfg.window > Duration::ZERO);
        assert!(cfg.lockout_duration > Duration::ZERO);
    }

    #[test]
    fn test_rate_limit_error_converts_to_auth_error() {
        let r: AuthError = RateLimitError::RateLimited {
            retry_after: Duration::from_secs(30),
        }
        .into();
        assert_eq!(r, AuthError::RateLimited);

        let locked_instant = Instant::now();
        let r: AuthError = RateLimitError::LockedOut {
            until: locked_instant,
        }
        .into();
        assert_eq!(r, AuthError::RateLimited);
    }
}
