//! # Cluster F reliability layer for `educore-notify`
//!
//! Production-readiness features that close the top four findings
//! from
//! [`docs/audit_reports/findings/wave3-notify.md`](../../../../docs/audit_reports/findings/wave3-notify.md):
//!
//! - **ADAPT-NOT-005** (no DLQ) → [`DeadLetterQueue`]
//! - **ADAPT-NOT-007** (no exponential backoff) → [`RetryPolicy`]
//! - **ADAPT-NOT-010** (provider failures downgrade silently)
//!   → [`ProviderFailover`]
//! - **ADAPT-NOT-012** (no per-provider rate limit) → [`RateLimiter`]
//!
//! All four types are self-contained, `Send + Sync`, and driven by
//! a [`Clock`] trait so tests can advance time deterministically
//! instead of relying on real `Instant::now()` calls.
//!
//! ## Scope
//!
//! This module is **deliberately decoupled** from the existing
//! [`crate::port`] types (`SendNotification`, `NotificationReceipt`,
//! etc.). Reliability features operate on the simplest possible
//! payload (`&str` / `String`) so they can be unit-tested without
//! constructing full port requests, and so they remain useful for
//! any future channel that ships a reference adapter (Push, InApp,
//! Chat, Voice, Webhook).
//!
//! Consumers wire the reliability layer into a reference adapter by
//! implementing the [`ReliabilityProvider`] trait on the adapter
//! (or by wrapping an existing [`crate::port::NotificationProvider`]
//! in an adapter shim).
//!
//! ## Concurrency
//!
//! All shared state lives behind a single `Mutex<...>` per type,
//! matching the pattern used by [`crate`]'s sibling adapters
//! (see `educore-auth::rate_limit`). For multi-replica deployments
//! the engine layers a shared store on top; these in-memory
//! implementations are the documented single-process baseline.

#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

/// Wall-clock abstraction used by every time-sensitive type in this
/// module so tests can drive time deterministically.
///
/// Production code uses [`SystemClock`]; tests substitute a mock
/// that exposes an `advance` method. The trait is object-safe and
/// `Send + Sync + 'static` so a single `Arc<dyn Clock>` can be
/// shared across the [`DeadLetterQueue`] and [`RateLimiter`].
pub trait Clock: Send + Sync + 'static {
    /// Returns the current instant. Implementations must be
    /// monotonic — backwards jumps are the caller's problem.
    fn now(&self) -> Instant;
}

/// Wall-clock implementation of [`Clock`]. Used by default in
/// production code.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant::now()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors returned by the reliability layer. `PartialEq` is derived
/// so tests can assert on specific variants without `match`ing.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReliabilityError {
    /// The primary provider failed and the fallback succeeded.
    /// This variant is currently never returned; kept for forward
    /// compatibility with callers that want to surface which leg
    /// of the failover actually delivered.
    #[error("primary provider failed: {0}")]
    Primary(String),

    /// The fallback provider failed.
    #[error("fallback provider failed: {0}")]
    Fallback(String),

    /// Both the primary and the fallback provider failed. The
    /// wrapped strings are the error messages from each leg.
    #[error("both providers failed: primary={primary}, fallback={fallback}")]
    BothFailed {
        /// Error from the primary provider attempt.
        primary: String,
        /// Error from the fallback provider attempt.
        fallback: String,
    },

    /// A `DeadLetterQueue::retry` call referenced an id that does
    /// not exist in the queue.
    #[error("DLQ entry not found: {0}")]
    UnknownDlqId(u64),

    /// The user-supplied `send_fn` passed to
    /// [`DeadLetterQueue::retry`] returned an error.
    #[error("retry callback failed: {0}")]
    RetryFailed(String),
}

// ---------------------------------------------------------------------------
// RetryPolicy
// ---------------------------------------------------------------------------

/// Exponential-backoff retry configuration.
///
/// `delay_for_attempt(n)` returns the wait **before** attempt `n`
/// (1-indexed). Attempt 1 has no wait (returns [`Duration::ZERO`]);
/// attempt 2 waits `initial_backoff`; attempt 3 waits
/// `initial_backoff * multiplier`; and so on, capped at
/// `max_backoff`.
///
/// # Example
///
/// ```
/// use std::time::Duration;
/// use educore_notify::reliability::RetryPolicy;
///
/// let policy = RetryPolicy::default();
/// assert_eq!(policy.delay_for_attempt(1), Duration::ZERO);
/// assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(1));
/// assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(2));
/// assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(4));
/// // attempt 8 would be 64s without the cap; default caps at 30s
/// assert_eq!(policy.delay_for_attempt(8), Duration::from_secs(30));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first try).
    /// `0` means "do not retry at all"; the policy still returns
    /// a well-defined delay for any attempt up to and including
    /// `max_attempts`.
    pub max_attempts: u32,
    /// Backoff before the **second** attempt (i.e. after the
    /// first failure).
    pub initial_backoff: Duration,
    /// Hard ceiling for any single delay. Prevents runaway
    /// exponential growth (`2.0_f64.powi(63)` is astronomical).
    pub max_backoff: Duration,
    /// Per-attempt growth factor. `2.0` doubles the delay each
    /// attempt; `1.5` gives a gentler curve.
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    /// 5 attempts, 1s → 30s, 2x multiplier, capped at 30s.
    /// Matches the audit report's recommendation (ADAPT-NOT-007).
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Returns the delay to wait **before** the given attempt.
    ///
    /// - `attempt == 0` → [`Duration::ZERO`] (defensive: should
    ///   not happen in normal use, but callers occasionally pass
    ///   0 from off-by-one loops).
    /// - `attempt == 1` → [`Duration::ZERO`] (first attempt has
    ///   no preceding failure).
    /// - `attempt == 2` → `initial_backoff`.
    /// - `attempt == n` for `n >= 2` →
    ///   `min(initial_backoff * multiplier^(n-2), max_backoff)`.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return Duration::ZERO;
        }
        let max_secs = self.max_backoff.as_secs_f64();
        let mut secs = self.initial_backoff.as_secs_f64();
        // For attempt 2: 0 multiplications → initial_backoff.
        // For attempt 3: 1 multiplication → initial_backoff * multiplier.
        // For attempt n: (n-2) multiplications → initial_backoff * multiplier^(n-2).
        // Loop multiplies in place; no integer→float casts that
        // could trip the `cast_possible_*` lints, and no
        // `powi` which would need an `as i32` on the exponent.
        for _ in 3..=attempt {
            secs *= self.multiplier;
            if secs >= max_secs {
                return self.max_backoff;
            }
        }
        // Final clamp — if the last iteration just barely
        // underflowed past `max_secs` due to f64 rounding.
        Duration::from_secs_f64(secs.min(max_secs))
    }
}

// ---------------------------------------------------------------------------
// DeadLetterQueue
// ---------------------------------------------------------------------------

/// A single message that exhausted its retry budget.
///
/// Returned by [`DeadLetterQueue::drain`] and consumed by
/// [`DeadLetterQueue::retry`]. Cloned for inspection; the queue
/// itself is the source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetterEntry {
    /// Process-unique monotonic id. Assigned by the queue at
    /// [`DeadLetterQueue::push`] time.
    pub id: u64,
    /// The payload that failed to send. Stringified to keep the
    /// reliability layer decoupled from the port types.
    pub payload: String,
    /// The error returned by the last send attempt. Stringified
    /// for the same reason.
    pub error: String,
    /// When the entry was pushed into the queue (per the
    /// configured [`Clock`]).
    pub failed_at: Instant,
}

/// In-memory dead-letter queue.
///
/// Holds messages that exhausted their retry budget. A separate
/// worker (or the next operator tick) drains the queue and calls
/// [`Self::retry`] on each entry, which delegates to the
/// caller's `send_fn` and removes the entry on success.
///
/// # Concurrency
///
/// All state lives behind a single `Mutex<VecDeque<...>>`. IDs are
/// assigned by a process-global `AtomicU64` so two queues in the
/// same process never collide (useful for tests that construct
/// multiple queues).
pub struct DeadLetterQueue {
    inner: Mutex<VecDeque<DeadLetterEntry>>,
    clock: Arc<dyn Clock>,
    next_id: AtomicU64,
}

impl DeadLetterQueue {
    /// Constructs a new DLQ driven by the given clock.
    #[must_use]
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Mutex::new(VecDeque::new()),
            clock,
            next_id: AtomicU64::new(1),
        }
    }

    /// Pushes a failed message into the queue and returns its
    /// assigned id. The caller passes the payload and error as
    /// owned strings so the queue owns the data for the lifetime
    /// of the entry.
    pub fn push(&self, payload: String, error: String) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let failed_at = self.clock.now();
        let entry = DeadLetterEntry {
            id,
            payload,
            error,
            failed_at,
        };
        match self.inner.lock() {
            Ok(mut g) => g.push_back(entry),
            Err(poisoned) => poisoned.into_inner().push_back(entry),
        }
        id
    }

    /// Removes and returns up to `max` entries from the front of
    /// the queue. If the queue has fewer than `max` entries, all
    /// are returned.
    pub fn drain(&self, max: usize) -> Vec<DeadLetterEntry> {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let take = max.min(g.len());
        g.drain(..take).collect()
    }

    /// Returns the current queue depth. Useful for metrics + tests.
    #[must_use]
    pub fn len(&self) -> usize {
        match self.inner.lock() {
            Ok(g) => g.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Returns `true` if the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Looks up an entry by id without removing it. Returns
    /// `None` if no entry with that id is currently in the queue.
    #[must_use]
    pub fn get(&self, id: u64) -> Option<DeadLetterEntry> {
        let g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        g.iter().find(|e| e.id == id).cloned()
    }

    /// Retries the message with id `id` by handing its payload to
    /// `send_fn`. On success, the entry is removed from the queue
    /// and `Ok(())` is returned. On failure (either the entry
    /// does not exist or `send_fn` returned an error), the queue
    /// is left unchanged and the error is returned.
    ///
    /// `send_fn` is `FnOnce` so the caller's callback can capture
    /// per-attempt state (e.g. an attempt counter) without
    /// `Arc<Mutex<...>>` plumbing.
    pub async fn retry<F, Fut>(&self, id: u64, send_fn: F) -> Result<(), ReliabilityError>
    where
        F: FnOnce(String) -> Fut,
        Fut: Future<Output = Result<(), ReliabilityError>>,
    {
        // Snapshot the payload under the lock, release before
        // awaiting the user callback so we don't hold the mutex
        // across an arbitrary await point.
        let payload = {
            let g = match self.inner.lock() {
                Ok(g) => g,
                Err(poisoned) => poisoned.into_inner(),
            };
            g.iter().find(|e| e.id == id).map(|e| e.payload.clone())
        };
        let payload = payload.ok_or(ReliabilityError::UnknownDlqId(id))?;

        send_fn(payload).await?;

        // Success: remove the entry. If it was already drained
        // by a concurrent retry call we leave it removed (no-op).
        match self.inner.lock() {
            Ok(mut g) => g.retain(|e| e.id != id),
            Err(poisoned) => poisoned.into_inner().retain(|e| e.id != id),
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ProviderFailover
// ---------------------------------------------------------------------------

/// Minimal async trait that both legs of a [`ProviderFailover`]
/// must implement.
///
/// Decoupled from [`crate::port::NotificationProvider`] so the
/// reliability layer can be unit-tested with cheap fake
/// implementations and so it remains usable for non-notification
/// adapters (e.g. webhook delivery, file uploads) that share the
/// same retry / failover shape.
///
/// `try_send` returns `Ok(String)` on success (the `String` is an
/// opaque receipt identifier the caller can log) or `Err` on
/// failure.
#[async_trait]
pub trait ReliabilityProvider: Send + Sync {
    /// Attempts to deliver `payload`. Returns `Ok(receipt_id)` on
    /// success or `Err` on failure.
    async fn try_send(&self, payload: &str) -> Result<String, ReliabilityError>;
}

/// Primary + fallback provider pair.
///
/// `send` first tries the primary; on failure it transparently
/// tries the fallback. Both providers must implement
/// [`ReliabilityProvider`]. The struct is generic over the two
/// concrete provider types so they can carry per-leg state
/// (endpoints, credentials, circuit breakers) without `dyn`
/// boxing.
pub struct ProviderFailover<P, F> {
    primary: P,
    fallback: F,
}

impl<P, F> ProviderFailover<P, F>
where
    P: ReliabilityProvider,
    F: ReliabilityProvider,
{
    /// Constructs a failover pair. Order is `(primary, fallback)`;
    /// `primary` is tried first.
    #[must_use]
    pub fn new(primary: P, fallback: F) -> Self {
        Self { primary, fallback }
    }

    /// Sends `payload` via the primary provider, falling back to
    /// the fallback provider if the primary fails.
    ///
    /// - Primary `Ok` → returns the primary's receipt id.
    /// - Primary `Err` + fallback `Ok` → returns the fallback's
    ///   receipt id.
    /// - Both `Err` → returns
    ///   [`ReliabilityError::BothFailed`] carrying both error
    ///   messages.
    pub async fn send(&self, payload: &str) -> Result<String, ReliabilityError> {
        match self.primary.try_send(payload).await {
            Ok(receipt) => Ok(receipt),
            Err(primary_err) => match self.fallback.try_send(payload).await {
                Ok(receipt) => Ok(receipt),
                Err(fallback_err) => Err(ReliabilityError::BothFailed {
                    primary: primary_err.to_string(),
                    fallback: fallback_err.to_string(),
                }),
            },
        }
    }

    /// Borrows the primary provider (useful for health checks
    /// and metrics that want to inspect per-leg state).
    #[must_use]
    pub fn primary(&self) -> &P {
        &self.primary
    }

    /// Borrows the fallback provider.
    #[must_use]
    pub fn fallback(&self) -> &F {
        &self.fallback
    }
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

/// Per-provider rate-limit configuration.
///
/// The default is a fixed-window bucket of 100 requests per
/// 60 seconds. The values are conservative defaults; production
/// callers should construct an explicit [`RateLimitConfig`] from
/// provider-specific quotas (e.g. Twilio's 1 msg/sec voice cap,
/// SendGrid's 100k/month soft limit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitConfig {
    /// Max requests permitted per [`Self::window`].
    pub capacity: u32,
    /// Window length. When the window expires, the bucket
    /// refills to `capacity`.
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            window: Duration::from_secs(60),
        }
    }
}

/// Errors returned by [`RateLimiter::check`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RateLimitError {
    /// The bucket for `provider` has no remaining tokens. The
    /// wrapped `retry_after` is the time until the current
    /// window expires.
    #[error("rate limit exhausted for provider {provider}; retry after {retry_after:?}")]
    Exhausted {
        /// The provider whose bucket is exhausted.
        provider: String,
        /// Time until the current window resets.
        retry_after: Duration,
    },
}

/// Per-provider bucket state.
#[derive(Debug, Clone, Copy)]
struct ProviderBucket {
    /// Remaining tokens in the current window.
    remaining: u32,
    /// When the current window started (per the configured
    /// [`Clock`]).
    window_started_at: Instant,
}

/// Per-provider, in-memory rate limiter.
///
/// Implements a fixed-window counter: each provider gets a
/// bucket of [`RateLimitConfig::capacity`] tokens per
/// [`RateLimitConfig::window`]. [`Self::check`] decrements the
/// counter; [`Self::record`] resets it back to capacity (used by
/// the worker that periodically refills buckets).
///
/// State is keyed by `String` provider id (cheap, ergonomic;
/// callers can use the [`crate::port::Channel`] variant name or
/// any other stable identifier).
///
/// # Concurrency
///
/// All state lives behind a single `Mutex<HashMap<...>>`. The
/// pattern matches the existing `educore-auth::rate_limit`
/// design so the cost model is consistent across adapters. For
/// multi-replica deployments the engine layers a shared store on
/// top; this in-memory implementation is the documented
/// single-process baseline.
pub struct RateLimiter {
    inner: Mutex<HashMap<String, ProviderBucket>>,
    config: RateLimitConfig,
    clock: Arc<dyn Clock>,
}

impl RateLimiter {
    /// Constructs a [`RateLimiter`] with the given config and
    /// clock.
    #[must_use]
    pub fn new(config: RateLimitConfig, clock: Arc<dyn Clock>) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            config,
            clock,
        }
    }

    /// Returns the active config. Useful for logging and admin
    /// endpoints.
    #[must_use]
    pub fn config(&self) -> RateLimitConfig {
        self.config
    }

    /// Checks whether `provider` has tokens remaining in the
    /// current window. Returns `Ok(())` if a token was consumed,
    /// or [`RateLimitError::Exhausted`] if the bucket is empty.
    ///
    /// If the current window has expired (per the configured
    /// [`Clock`]) the bucket is refilled to
    /// [`RateLimitConfig::capacity`] before the decrement.
    pub fn check(&self, provider: &str) -> Result<(), RateLimitError> {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = self.clock.now();
        let entry = g.entry(provider.to_owned()).or_insert(ProviderBucket {
            remaining: self.config.capacity,
            window_started_at: now,
        });
        // Window expired → refill.
        if now.saturating_duration_since(entry.window_started_at) >= self.config.window {
            entry.remaining = self.config.capacity;
            entry.window_started_at = now;
        }
        if entry.remaining == 0 {
            let elapsed = now.saturating_duration_since(entry.window_started_at);
            let retry_after = self.config.window.saturating_sub(elapsed);
            return Err(RateLimitError::Exhausted {
                provider: provider.to_owned(),
                retry_after,
            });
        }
        entry.remaining -= 1;
        Ok(())
    }

    /// Resets the bucket for `provider` back to
    /// [`RateLimitConfig::capacity`] and starts a fresh window
    /// at the configured [`Clock`]'s current instant.
    ///
    /// Intended for periodic refill workers; production code
    /// should not call this from the hot path.
    pub fn record(&self, provider: &str) {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = self.clock.now();
        g.insert(
            provider.to_owned(),
            ProviderBucket {
                remaining: self.config.capacity,
                window_started_at: now,
            },
        );
    }

    /// Removes all per-provider state. Useful for tests and
    /// admin endpoints.
    pub fn clear(&self) {
        match self.inner.lock() {
            Ok(mut g) => g.clear(),
            Err(poisoned) => poisoned.into_inner().clear(),
        }
    }

    /// Returns the number of tracked providers. Useful for
    /// metrics and tests.
    #[must_use]
    pub fn provider_count(&self) -> usize {
        match self.inner.lock() {
            Ok(g) => g.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Returns the remaining tokens for `provider` (or `None`
    /// if the provider has never been checked). Useful for
    /// metrics and tests.
    #[must_use]
    pub fn remaining(&self, provider: &str) -> Option<u32> {
        let g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        g.get(provider).map(|b| b.remaining)
    }
}
