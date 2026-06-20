//! # Integration service helpers
//!
//! Pure helper structs that the integration adapters (LMS, video
//! conferencing, webhook out, polling) reuse so the same logic is
//! not duplicated across modules. Each helper is stateless
//! ([`WebhookSignatureService`], [`PollingService`],
//! [`RetryService`]) or holds only an in-memory cache
//! ([`RateLimitService`]) and performs **no I/O**.
//!
//! The helpers are pure functions on top of types already defined
//! in [`crate::port`] — they do not extend the port contract and
//! never call into a provider. Adapters wrap them with their
//! async I/O layer.
//!
//! Per `docs/ports/integrations.md` § "Service Helpers":
//!
//! - [`WebhookSignatureService`] computes and verifies
//!   HMAC-SHA256 signatures for outbound webhook payloads.
//! - [`PollingService`] advances cursors and decides when the
//!   next poll cycle is due.
//! - [`RetryService`] computes the next backoff delay and
//!   classifies HTTP status codes as permanent or transient.
//! - [`RateLimitService`] keeps a per-integration token bucket so
//!   callers don't exceed the configured per-second quota.

#![allow(clippy::module_name_repetitions)]

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

use chrono::Duration as ChronoDuration;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use educore_core::value_objects::Timestamp;

use crate::errors::{IntegrationError, Result};
use crate::port::{IntegrationId, RetryPolicy};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// HMAC-SHA256 type alias used everywhere in this module.
type HmacSha256 = Hmac<Sha256>;

/// One-hour polling interval (used by [`Schedule::Hourly`]).
const HOURLY_SECONDS: i64 = 60 * 60;

/// One-day polling interval (used by [`Schedule::Daily`]).
const DAILY_SECONDS: i64 = 60 * 60 * 24;

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

/// A polling cadence for inbound adapters that pull data from a
/// provider (e.g. LMS roster sync, payment reconciliation).
///
/// Constructed from a configuration string via
/// [`PollingService::parse_schedule`] — the string form is what
/// callers store in the engine's settings table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Schedule {
    /// Poll every hour. The interval is fixed at 3 600 s.
    Hourly,
    /// Poll every day. The interval is fixed at 86 400 s.
    Daily,
    /// Never poll automatically. The adapter is driven exclusively
    /// by manual triggers (operator-initiated or webhook-driven).
    Manual,
}

impl Schedule {
    /// Returns the interval between two automatic polls, or `None`
    /// for [`Schedule::Manual`] (no automatic poll ever fires).
    #[must_use]
    pub const fn interval(self) -> Option<ChronoDuration> {
        match self {
            Self::Hourly => Some(ChronoDuration::seconds(HOURLY_SECONDS)),
            Self::Daily => Some(ChronoDuration::seconds(DAILY_SECONDS)),
            Self::Manual => None,
        }
    }

    /// Returns the lowercase string form used in configuration
    /// ("hourly", "daily", "manual"). Round-trips with
    /// [`PollingService::parse_schedule`].
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Manual => "manual",
        }
    }
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// RateState
// ---------------------------------------------------------------------------

/// Snapshot of a single integration's token bucket. Returned by
/// [`RateLimitService::current_state`] for diagnostics and
/// dashboards.
#[derive(Debug, Clone)]
pub struct RateState {
    /// Whole-token count currently available (floor of the
    /// fractional bucket). The adapter may still issue
    /// `tokens_remaining` more calls before the bucket is empty.
    pub tokens_remaining: u32,
    /// Configured per-second refill rate at the time of the
    /// snapshot.
    pub max_per_second: u32,
    /// Wall-clock instant the bucket was last refilled.
    pub last_refill: Instant,
}

// ---------------------------------------------------------------------------
// WebhookSignatureService
// ---------------------------------------------------------------------------

/// Computes and verifies HMAC-SHA256 signatures for outbound
/// webhook payloads.
///
/// The signing format is `"sha256=<hex>"`, matching the receiver
/// convention documented in `docs/ports/integrations.md` §
/// "Custom Webhook (Out)".
#[derive(Debug, Default, Clone, Copy)]
pub struct WebhookSignatureService;

impl WebhookSignatureService {
    /// Returns the `"sha256=<hex>"` signature of `payload` under
    /// `secret`.
    ///
    /// HMAC-SHA256 accepts keys of any length; this method does
    /// not validate `secret` beyond what HMAC itself rejects (no
    /// rejection — every byte sequence is a valid HMAC key).
    pub fn compute_signature(secret: &str, payload: &[u8]) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| IntegrationError::Infrastructure(format!("HMAC key error: {e}").into()))?;
        mac.update(payload);
        let bytes = mac.finalize().into_bytes();
        Ok(format!("sha256={}", hex_encode(&bytes)))
    }

    /// Verifies a `"sha256=<hex>"` signature in constant time.
    ///
    /// Returns `false` if `provided` has the wrong length, the
    /// wrong prefix, or any byte differs from the recomputed
    /// signature. The byte comparison walks the full buffer in
    /// constant time so an attacker cannot probe the receiver by
    /// timing partial matches.
    pub fn verify_signature(secret: &str, payload: &[u8], provided: &str) -> Result<bool> {
        let expected = Self::compute_signature(secret, payload)?;
        Ok(constant_time_eq(expected.as_bytes(), provided.as_bytes()))
    }
}

// ---------------------------------------------------------------------------
// PollingService
// ---------------------------------------------------------------------------

/// Cursor advancement + due-date checks for inbound polling
/// adapters.
#[derive(Debug, Default, Clone, Copy)]
pub struct PollingService;

impl PollingService {
    /// Returns the next cursor to use on the following poll.
    ///
    /// Precedence: a non-`None` `response_cursor` (the provider's
    /// latest cursor) wins over the `current_cursor` (the cursor
    /// we sent on the previous poll). When the provider omits a
    /// cursor the adapter should keep using `current_cursor` —
    /// either the provider has no more pages or the cursor moves
    /// out-of-band.
    #[must_use]
    pub fn compute_next_cursor(
        current_cursor: Option<String>,
        response_cursor: Option<String>,
    ) -> Option<String> {
        response_cursor.or(current_cursor)
    }

    /// Returns `true` if the adapter should fire a poll right
    /// now.
    ///
    /// For [`Schedule::Manual`] the method is always `false` —
    /// manual schedules never fire on their own. For
    /// [`Schedule::Hourly`] and [`Schedule::Daily`] the method
    /// returns `true` once `now - last_poll >= schedule_interval`.
    /// A zero `last_poll` (e.g. never polled) is treated as
    /// "due immediately".
    #[must_use]
    pub fn should_poll(schedule: &Schedule, last_poll: Timestamp, now: Timestamp) -> bool {
        match schedule.interval() {
            None => false,
            Some(interval) => {
                let elapsed = now.as_datetime() - last_poll.as_datetime();
                elapsed >= interval
            }
        }
    }

    /// Parses the string form of a [`Schedule`].
    ///
    /// Accepts `"hourly"`, `"daily"`, and `"manual"` (case-
    /// insensitive, surrounding whitespace tolerated). Any other
    /// input falls back to [`Schedule::Manual`] — manual is the
    /// safe default because it never fires on its own.
    #[must_use]
    pub fn parse_schedule(s: &str) -> Schedule {
        match s.trim().to_ascii_lowercase().as_str() {
            "hourly" => Schedule::Hourly,
            "daily" => Schedule::Daily,
            _ => Schedule::Manual,
        }
    }
}

// ---------------------------------------------------------------------------
// RetryService
// ---------------------------------------------------------------------------

/// Backoff calculation + permanent-failure classification for the
/// outbound adapters.
#[derive(Debug, Default, Clone, Copy)]
pub struct RetryService;

impl RetryService {
    /// Computes the next backoff delay for `attempt`, where
    /// `attempt == 1` is the first retry (the original call is
    /// `attempt == 0`).
    ///
    /// Returns `None` when the caller has exhausted the policy
    /// (give up) or when the policy is [`RetryPolicy::None`].
    ///
    /// `Linear` always returns the same `interval` for every
    /// attempt in range. `Exponential` returns
    /// `min(base * 2^attempt, max)`.
    #[must_use]
    pub fn next_backoff(policy: &RetryPolicy, attempt: u32) -> Option<Duration> {
        match *policy {
            RetryPolicy::None => None,
            RetryPolicy::Linear {
                max_retries,
                interval,
            } => {
                if attempt == 0 || attempt > max_retries {
                    None
                } else {
                    chrono_to_std(interval)
                }
            }
            RetryPolicy::Exponential {
                max_retries,
                base,
                max,
            } => {
                if attempt == 0 || attempt > max_retries {
                    return None;
                }
                let base_std = chrono_to_std(base).unwrap_or(Duration::from_secs(1));
                let max_std = chrono_to_std(max).unwrap_or(Duration::from_secs(30));
                Some(exponential_backoff(base_std, max_std, attempt))
            }
        }
    }

    /// Returns `true` for HTTP status codes that should not be
    /// retried: 4xx responses other than `408 Request Timeout` and
    /// `429 Too Many Requests`. 5xx, 2xx, and 3xx all return
    /// `false`.
    ///
    /// Per `docs/ports/integrations.md` § "Retry Policy": a 4xx
    /// response signals "the request is malformed or forbidden";
    /// retrying will not change the outcome. 408 and 429 are
    /// transient — the client should back off and retry.
    #[must_use]
    pub fn is_permanent_failure(status_code: u16) -> bool {
        let is_4xx = (400..500).contains(&status_code);
        let is_transient_4xx = status_code == 408 || status_code == 429;
        is_4xx && !is_transient_4xx
    }

    /// Combines [`RetryService::next_backoff`] and
    /// [`RetryService::is_permanent_failure`].
    ///
    /// Returns `true` only when (a) the policy still has retries
    /// left for `attempt` and (b) the failure is not permanent.
    /// Either exhaustion or a permanent failure short-circuits to
    /// `false`.
    #[must_use]
    pub fn should_retry(policy: &RetryPolicy, attempt: u32, status_code: u16) -> bool {
        if Self::is_permanent_failure(status_code) {
            return false;
        }
        Self::next_backoff(policy, attempt).is_some()
    }
}

// ---------------------------------------------------------------------------
// RateLimitService
// ---------------------------------------------------------------------------

/// In-memory per-integration token bucket.
///
/// The service is `Send + Sync` so adapters can hold it behind an
/// `Arc<Mutex<_>>` (or wrap it in their own synchronisation
/// primitive). The bucket itself is single-threaded; concurrent
/// adapters must serialise access.
#[derive(Debug)]
pub struct RateLimitService {
    buckets: HashMap<IntegrationId, RateBucket>,
}

#[derive(Debug)]
struct RateBucket {
    tokens: f64,
    max_per_second: u32,
    last_refill: Instant,
}

impl Default for RateLimitService {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimitService {
    /// Returns a fresh service with no buckets.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }

    /// Attempts to acquire one token for `integration`.
    ///
    /// The bucket is refilled lazily: each call adds
    /// `(elapsed_seconds * max_per_second)` tokens up to
    /// `max_per_second` before the check. If the bucket holds at
    /// least one whole token, the call returns `true` and
    /// decrements the bucket by one. Otherwise it returns `false`
    /// without modifying state beyond the refill.
    ///
    /// `max_per_second == 0` always returns `false` (the caller
    /// has explicitly disabled the integration).
    pub fn try_acquire(&mut self, integration: &IntegrationId, max_per_second: u32) -> bool {
        if max_per_second == 0 {
            return false;
        }

        let now = Instant::now();
        let bucket = self.buckets.entry(integration.clone()).or_insert_with(|| RateBucket {
            tokens: f64::from(max_per_second),
            max_per_second,
            last_refill: now,
        });

        if bucket.max_per_second != max_per_second {
            bucket.max_per_second = max_per_second;
            if bucket.tokens > f64::from(max_per_second) {
                bucket.tokens = f64::from(max_per_second);
            }
        }

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens =
            (bucket.tokens + elapsed * f64::from(max_per_second)).min(f64::from(max_per_second));
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Removes the bucket for `integration`. The next
    /// [`RateLimitService::try_acquire`] call rebuilds the bucket
    /// at full capacity.
    pub fn reset(&mut self, integration: &IntegrationId) {
        self.buckets.remove(integration);
    }

    /// Returns the current [`RateState`] snapshot for
    /// `integration`, or `None` if no bucket has been created yet.
    ///
    /// The snapshot performs a lazy refill using the current
    /// `Instant::now()` so the reported `tokens_remaining` is
    /// accurate as of the call.
    #[must_use]
    pub fn current_state(&mut self, integration: &IntegrationId) -> Option<RateState> {
        let now = Instant::now();
        let bucket = self.buckets.get_mut(integration)?;

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens =
            (bucket.tokens + elapsed * f64::from(bucket.max_per_second)).min(f64::from(bucket.max_per_second));
        bucket.last_refill = now;

        Some(RateState {
            tokens_remaining: {
                let v = bucket.tokens.floor();
                if v >= f64::from(u32::MAX) {
                    u32::MAX
                } else if v <= 0.0 {
                    0
                } else {
                    // v is finite and in (0, u32::MAX); cast is exact.
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    {
                        v as u32
                    }
                }
            },
            max_per_second: bucket.max_per_second,
            last_refill: bucket.last_refill,
        })
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Lowercase hex encoding (one byte -> two hex chars).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Constant-time byte slice equality. Walks the full buffer
/// regardless of where the first mismatch occurs so an attacker
/// cannot infer the prefix of the expected value from timing.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Converts a [`chrono::Duration`] to a [`std::time::Duration`],
/// clamping a negative or overflowing value to `None`.
fn chrono_to_std(d: ChronoDuration) -> Option<Duration> {
    if d < ChronoDuration::zero() {
        return None;
    }
    d.to_std().ok()
}

/// Computes `min(base * 2^attempt, max)`. The shift is clamped to
/// 63 to avoid `u64` overflow in the multiplier; once the
/// intermediate value exceeds `max`, the function returns `max`
/// without further arithmetic.
fn exponential_backoff(base: Duration, max: Duration, attempt: u32) -> Duration {
    if attempt >= 64 {
        return max;
    }
    let factor = 1u64 << attempt;
    let base_nanos = base.as_nanos().min(u128::from(u64::MAX));
    let max_nanos = max.as_nanos().min(u128::from(u64::MAX));
    let scaled = (base_nanos.saturating_mul(u128::from(factor))).min(max_nanos);
    Duration::from_nanos(u64::try_from(scaled).unwrap_or(0))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    use chrono::TimeZone;
    use chrono::Utc;

    use educore_core::value_objects::Timestamp;

    fn dt(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Timestamp {
        Timestamp::from_datetime(
            Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
                .single()
                .expect("valid test timestamp"),
        )
    }

    #[test]
    fn test_webhook_signature_service_compute_and_verify() {
        let secret = "shared-secret";
        let payload = br#"{"event":"InvoicePaid","amount_minor":12500}"#;

        let sig = WebhookSignatureService::compute_signature(secret, payload).expect("HMAC succeeds");
        assert!(
            sig.starts_with("sha256="),
            "signature must carry the sha256= prefix, got {sig}"
        );
        assert_eq!(sig.len(), "sha256=".len() + 64);

        assert!(
            WebhookSignatureService::verify_signature(secret, payload, &sig).expect("HMAC succeeds"),
            "freshly-computed signature must verify against itself"
        );

        assert!(
            !WebhookSignatureService::verify_signature(secret, payload, "sha256=deadbeef")
                .expect("HMAC succeeds"),
            "wrong signature must not verify"
        );

        assert!(
            !WebhookSignatureService::verify_signature(secret, payload, "sha256=")
                .expect("HMAC succeeds"),
            "truncated signature must not verify"
        );

        assert!(
            !WebhookSignatureService::verify_signature("other-secret", payload, &sig)
                .expect("HMAC succeeds"),
            "signature under a different secret must not verify"
        );
    }

    #[test]
    fn test_polling_service_should_poll() {
        let last = dt(2026, 1, 1, 0, 0, 0);
        let one_hour_later = dt(2026, 1, 1, 1, 0, 0);
        let half_hour_later = dt(2026, 1, 1, 0, 30, 0);
        let one_day_later = dt(2026, 1, 2, 0, 0, 0);

        assert!(
            !PollingService::should_poll(&Schedule::Hourly, last, half_hour_later),
            "30 minutes into an hourly schedule is not yet due"
        );
        assert!(
            PollingService::should_poll(&Schedule::Hourly, last, one_hour_later),
            "exactly one hour later must trigger the hourly poll"
        );
        assert!(
            PollingService::should_poll(&Schedule::Daily, last, one_day_later),
            "exactly one day later must trigger the daily poll"
        );
        assert!(
            !PollingService::should_poll(&Schedule::Daily, last, one_hour_later),
            "one hour into a daily schedule is not yet due"
        );
        assert!(
            !PollingService::should_poll(&Schedule::Manual, last, one_day_later),
            "manual schedule must never auto-fire"
        );

        assert_eq!(
            PollingService::compute_next_cursor(Some("c1".into()), Some("c2".into())),
            Some("c2".into()),
            "response cursor wins over current"
        );
        assert_eq!(
            PollingService::compute_next_cursor(Some("c1".into()), None),
            Some("c1".into()),
            "missing response cursor keeps the current cursor"
        );
        assert_eq!(
            PollingService::compute_next_cursor(None, None),
            None,
            "no cursor anywhere yields no cursor"
        );

        assert_eq!(PollingService::parse_schedule("hourly"), Schedule::Hourly);
        assert_eq!(PollingService::parse_schedule("daily"), Schedule::Daily);
        assert_eq!(PollingService::parse_schedule("manual"), Schedule::Manual);
        assert_eq!(PollingService::parse_schedule("HOURLY"), Schedule::Hourly);
        assert_eq!(PollingService::parse_schedule("  daily "), Schedule::Daily);
        assert_eq!(
            PollingService::parse_schedule("every-minute"),
            Schedule::Manual,
            "unknown schedule must default to Manual (safe no-auto-fire)"
        );
    }

    #[test]
    fn test_retry_service_exponential_backoff() {
        let policy = RetryPolicy::Exponential {
            max_retries: 3,
            base: ChronoDuration::seconds(1),
            max: ChronoDuration::seconds(30),
        };

        assert_eq!(
            RetryService::next_backoff(&policy, 1),
            Some(StdDuration::from_secs(2)),
            "attempt 1 -> base * 2 = 2 s"
        );
        assert_eq!(
            RetryService::next_backoff(&policy, 2),
            Some(StdDuration::from_secs(4)),
            "attempt 2 -> base * 4 = 4 s"
        );
        assert_eq!(
            RetryService::next_backoff(&policy, 3),
            Some(StdDuration::from_secs(8)),
            "attempt 3 -> base * 8 = 8 s"
        );
        assert_eq!(
            RetryService::next_backoff(&policy, 4),
            None,
            "attempt beyond max_retries yields None (give up)"
        );
        assert_eq!(
            RetryService::next_backoff(&policy, 0),
            None,
            "attempt 0 means the original call (no backoff computed)"
        );

        let capped = RetryPolicy::Exponential {
            max_retries: 10,
            base: ChronoDuration::seconds(1),
            max: ChronoDuration::seconds(5),
        };
        assert_eq!(
            RetryService::next_backoff(&capped, 5),
            Some(StdDuration::from_secs(5)),
            "exponential must cap at the configured max"
        );

        let linear = RetryPolicy::Linear {
            max_retries: 2,
            interval: ChronoDuration::seconds(7),
        };
        assert_eq!(
            RetryService::next_backoff(&linear, 1),
            Some(StdDuration::from_secs(7))
        );
        assert_eq!(
            RetryService::next_backoff(&linear, 2),
            Some(StdDuration::from_secs(7))
        );
        assert_eq!(RetryService::next_backoff(&linear, 3), None);

        assert_eq!(RetryService::next_backoff(&RetryPolicy::None, 1), None);

        assert!(
            RetryService::is_permanent_failure(400),
            "400 Bad Request is permanent"
        );
        assert!(
            RetryService::is_permanent_failure(404),
            "404 Not Found is permanent"
        );
        assert!(
            !RetryService::is_permanent_failure(408),
            "408 Request Timeout is transient"
        );
        assert!(
            !RetryService::is_permanent_failure(429),
            "429 Too Many Requests is transient"
        );
        assert!(
            !RetryService::is_permanent_failure(500),
            "5xx is transient"
        );
        assert!(
            !RetryService::is_permanent_failure(200),
            "2xx is not a failure at all"
        );

        assert!(
            !RetryService::should_retry(&policy, 1, 404),
            "permanent 4xx -> no retry"
        );
        assert!(
            RetryService::should_retry(&policy, 1, 500),
            "5xx within retry budget -> retry"
        );
        assert!(
            RetryService::should_retry(&policy, 1, 429),
            "429 within retry budget -> retry (transient 4xx)"
        );
        assert!(
            !RetryService::should_retry(&policy, 4, 500),
            "5xx beyond retry budget -> give up"
        );
    }

    #[test]
    fn test_rate_limit_service_token_bucket() {
        let mut svc = RateLimitService::new();
        let id = IntegrationId::from("twilio");

        assert!(
            svc.try_acquire(&id, 2),
            "first call into an empty bucket must succeed"
        );
        assert!(
            svc.try_acquire(&id, 2),
            "second call (still under capacity) must succeed"
        );
        assert!(
            !svc.try_acquire(&id, 2),
            "third call into a 2-per-second bucket must fail immediately"
        );

        let state = svc.current_state(&id).expect("bucket exists");
        assert_eq!(state.max_per_second, 2);
        assert!(
            state.tokens_remaining <= 2,
            "tokens must be capped at max_per_second, got {}",
            state.tokens_remaining
        );

        thread::sleep(StdDuration::from_millis(600));
        assert!(
            svc.try_acquire(&id, 2),
            "after sleeping ~600 ms the bucket should have refilled enough for one call"
        );

        assert!(
            !svc.try_acquire(&id, 0),
            "max_per_second = 0 disables the integration"
        );

        let other = IntegrationId::from("stripe");
        assert!(
            svc.try_acquire(&other, 1),
            "each integration has its own bucket"
        );

        svc.reset(&id);
        assert!(
            svc.try_acquire(&id, 2),
            "reset must restore the bucket to full capacity"
        );

        let missing = IntegrationId::from("never-seen");
        assert!(
            svc.current_state(&missing).is_none(),
            "current_state must return None for an integration that has never called try_acquire"
        );
    }
}