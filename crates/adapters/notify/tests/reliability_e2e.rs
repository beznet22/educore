//! # Cluster F reliability layer — integration tests
//!
//! Covers the four production-readiness features added in
//! `crates/adapters/notify/src/reliability.rs`:
//!
//! - [`RetryPolicy`](reliability::RetryPolicy) — exponential
//!   backoff (ADAPT-NOT-007)
//! - [`DeadLetterQueue`](reliability::DeadLetterQueue) — in-memory
//!   DLQ with retry-via-callback (ADAPT-NOT-005)
//! - [`ProviderFailover`](reliability::ProviderFailover) — primary
//!   → fallback (ADAPT-NOT-010)
//! - [`RateLimiter`](reliability::RateLimiter) — per-provider
//!   fixed-window rate limit (ADAPT-NOT-012)
//!
//! These are pure-helper tests: no SMTP relay, no HTTP gateway, no
//! async runtime sleeps. The [`MockClock`] advances time
//! deterministically so the backoff / window / lockout tests run
//! in microseconds instead of seconds.
//!
//! ## Why `#[path]` and not `use educore_notify::reliability::*`?
//!
//! The task scope explicitly forbids modifying `lib.rs` to add
//! `pub mod reliability;`. Including the source file via
//! `#[path]` is the standard Rust pattern for integration tests
//! that need to reach a source file the library crate hasn't yet
//! declared as a module. The file's own inner attributes
//! (`#![forbid(unsafe_code)]`) still apply to the included
//! module.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// `reliability.rs` is wired into the integration test binary via
// `#[path]` because the library's `lib.rs` does not declare it
// (the task scope forbids touching `lib.rs`).
#[path = "../src/reliability.rs"]
mod reliability;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use reliability::{
    Clock, DeadLetterQueue, ProviderFailover, RateLimitConfig, RateLimitError, RateLimiter,
    ReliabilityError, ReliabilityProvider, RetryPolicy, SystemClock,
};

// ---------------------------------------------------------------------------
// MockClock (test-only)
// ---------------------------------------------------------------------------

/// Test clock that exposes an `advance` method. Cloning shares the
/// underlying `Instant`, so a test can hand one clone to the
/// reliability layer and keep another to drive time.
#[derive(Debug, Clone)]
struct MockClock {
    inner: Arc<Mutex<Instant>>,
}

impl MockClock {
    fn new(start: Instant) -> Self {
        Self {
            inner: Arc::new(Mutex::new(start)),
        }
    }

    fn advance(&self, by: Duration) {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        // Saturation: an absurd input pins the clock at its
        // current value rather than panicking.
        *g = g.checked_add(by).unwrap_or(*g);
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        match self.inner.lock() {
            Ok(g) => *g,
            Err(poisoned) => *poisoned.into_inner(),
        }
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: RetryPolicy::delay_for_attempt returns exponential backoff
// ---------------------------------------------------------------------------

#[test]
fn reliability_retry_policy_exponential_backoff() {
    let p = RetryPolicy::default();
    // 1-indexed: attempt 1 has no preceding failure.
    assert_eq!(p.delay_for_attempt(1), Duration::ZERO);
    // attempt 2 → initial_backoff (1s)
    assert_eq!(p.delay_for_attempt(2), Duration::from_secs(1));
    // attempt 3 → 2s
    assert_eq!(p.delay_for_attempt(3), Duration::from_secs(2));
    // attempt 4 → 4s
    assert_eq!(p.delay_for_attempt(4), Duration::from_secs(4));
    // attempt 5 → 8s
    assert_eq!(p.delay_for_attempt(5), Duration::from_secs(8));
}

// ---------------------------------------------------------------------------
// Scenario 2: RetryPolicy caps at max_backoff
// ---------------------------------------------------------------------------

#[test]
fn reliability_retry_policy_caps_at_max_backoff() {
    // Small max to exercise the cap quickly.
    let p = RetryPolicy {
        max_attempts: 10,
        initial_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(5),
        multiplier: 2.0,
    };
    // attempt 6 → 16s uncapped → clamped to 5s.
    assert_eq!(p.delay_for_attempt(6), Duration::from_secs(5));
    // attempt 10 → would be 256s uncapped → clamped to 5s.
    assert_eq!(p.delay_for_attempt(10), Duration::from_secs(5));
    // Below the cap, normal exponential still holds.
    assert_eq!(p.delay_for_attempt(4), Duration::from_secs(4));
    // attempt 0 and 1 → ZERO (defensive).
    assert_eq!(p.delay_for_attempt(0), Duration::ZERO);
    assert_eq!(p.delay_for_attempt(1), Duration::ZERO);
}

// ---------------------------------------------------------------------------
// Scenario 3: DeadLetterQueue::push stores message + error
// ---------------------------------------------------------------------------

#[test]
fn reliability_dlq_push_stores_message_and_error() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let dlq = DeadLetterQueue::new(clock);

    let id = dlq.push("payload-A".to_owned(), "smtp timeout".to_owned());
    assert_eq!(id, 1, "first id must be 1");
    assert_eq!(dlq.len(), 1);
    assert!(!dlq.is_empty());

    let entry = dlq.get(id).expect("entry must be retrievable");
    assert_eq!(entry.payload, "payload-A");
    assert_eq!(entry.error, "smtp timeout");

    // Second push gets a unique monotonic id.
    let id2 = dlq.push("payload-B".to_owned(), "429 too many requests".to_owned());
    assert_ne!(id, id2);
    assert_eq!(dlq.len(), 2);
}

// ---------------------------------------------------------------------------
// Scenario 4: DeadLetterQueue::drain returns up to max
// ---------------------------------------------------------------------------

#[test]
fn reliability_dlq_drain_returns_up_to_max() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let dlq = DeadLetterQueue::new(clock);

    for i in 0..5 {
        dlq.push(format!("payload-{i}"), format!("err-{i}"));
    }
    assert_eq!(dlq.len(), 5);

    // drain(2) → 2 entries, 3 remain.
    let drained = dlq.drain(2);
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].payload, "payload-0");
    assert_eq!(drained[1].payload, "payload-1");
    assert_eq!(dlq.len(), 3);

    // drain(100) → all remaining.
    let rest = dlq.drain(100);
    assert_eq!(rest.len(), 3);
    assert!(dlq.is_empty());

    // drain on empty queue → empty vec (not an error).
    let empty = dlq.drain(10);
    assert!(empty.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario 5: DLQ retries message via send_fn
// ---------------------------------------------------------------------------

#[tokio::test]
async fn reliability_dlq_retries_message_via_send_fn() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let dlq = DeadLetterQueue::new(clock);
    let id = dlq.push("payload-retry".to_owned(), "first failure".to_owned());
    assert_eq!(dlq.len(), 1);

    // The send_fn is FnOnce so it can capture per-attempt state.
    let received = Arc::new(Mutex::new(None::<String>));
    let received_clone = Arc::clone(&received);
    let result = dlq
        .retry(id, move |payload: String| {
            let received = Arc::clone(&received_clone);
            async move {
                *received.lock().unwrap_or_else(PoisonError::into_inner) = Some(payload);
                Ok(())
            }
        })
        .await;

    assert!(
        result.is_ok(),
        "retry must succeed when send_fn returns Ok: {result:?}"
    );
    assert_eq!(dlq.len(), 0, "successful retry must remove the entry");
    assert_eq!(
        received
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_deref(),
        Some("payload-retry"),
    );

    // A second retry against the same id now fails (already removed).
    let second = dlq.retry(id, |_p: String| async { Ok(()) }).await;
    assert_eq!(second, Err(ReliabilityError::UnknownDlqId(id)));
}

// ---------------------------------------------------------------------------
// Scenario 6: ProviderFailover succeeds on fallback when primary fails
// ---------------------------------------------------------------------------

/// Always-failing provider. Returns a deterministic error string.
#[derive(Debug)]
struct FailingProvider {
    label: &'static str,
}

#[async_trait]
impl ReliabilityProvider for FailingProvider {
    async fn try_send(&self, _payload: &str) -> Result<String, ReliabilityError> {
        Err(ReliabilityError::Primary(format!("{} failed", self.label)))
    }
}

/// Always-succeeding provider. Returns a deterministic receipt id.
#[derive(Debug)]
struct SucceedingProvider {
    label: &'static str,
}

#[async_trait]
impl ReliabilityProvider for SucceedingProvider {
    async fn try_send(&self, payload: &str) -> Result<String, ReliabilityError> {
        Ok(format!("{}:{}", self.label, payload))
    }
}

#[tokio::test]
async fn reliability_provider_failover_succeeds_on_fallback() {
    let primary = FailingProvider { label: "primary" };
    let fallback = SucceedingProvider { label: "fallback" };
    let fo = ProviderFailover::new(primary, fallback);

    let receipt = fo.send("hello").await.expect("fallback must succeed");
    assert_eq!(receipt, "fallback:hello");
}

#[tokio::test]
async fn reliability_provider_failover_returns_primary_receipt_when_primary_ok() {
    let primary = SucceedingProvider { label: "primary" };
    let fallback = SucceedingProvider { label: "fallback" };
    let fo = ProviderFailover::new(primary, fallback);

    let receipt = fo.send("hello").await.expect("primary must succeed");
    assert_eq!(receipt, "primary:hello");
}

#[tokio::test]
async fn reliability_provider_failover_both_failed_returns_combined_error() {
    let primary = FailingProvider { label: "primary" };
    let fallback = FailingProvider { label: "fallback" };
    let fo = ProviderFailover::new(primary, fallback);

    let err = fo.send("hello").await.expect_err("both fail");
    assert!(
        matches!(err, ReliabilityError::BothFailed { .. }),
        "expected BothFailed, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7: RateLimiter allows N then blocks N+1
// ---------------------------------------------------------------------------

#[test]
fn reliability_rate_limiter_allows_n_then_blocks_n_plus_one() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let cfg = RateLimitConfig {
        capacity: 3,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::new(cfg, clock);

    // First 3 checks succeed.
    for i in 0..3 {
        limiter
            .check("sendgrid")
            .unwrap_or_else(|e| panic!("attempt {i} must pass: {e:?}"));
    }
    assert_eq!(limiter.remaining("sendgrid"), Some(0));

    // 4th is blocked.
    let err = limiter
        .check("sendgrid")
        .expect_err("4th attempt must be rate-limited");
    match err {
        RateLimitError::Exhausted {
            provider,
            retry_after,
        } => {
            assert_eq!(provider, "sendgrid");
            assert!(retry_after > Duration::ZERO);
        }
    }
}

#[test]
fn reliability_rate_limiter_is_per_provider() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let cfg = RateLimitConfig {
        capacity: 1,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::new(cfg, clock);

    // Exhaust provider A.
    limiter.check("sendgrid").expect("a 1st passes");
    limiter
        .check("sendgrid")
        .expect_err("a 2nd must be rate-limited");

    // Provider B is on a fresh bucket.
    limiter.check("twilio").expect("b 1st passes");
}

#[test]
fn reliability_rate_limiter_window_reset_refills() {
    let clock = MockClock::new(Instant::now());
    let clock_dyn: Arc<dyn Clock> = Arc::new(clock.clone());
    let cfg = RateLimitConfig {
        capacity: 2,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::new(cfg, clock_dyn);

    limiter.check("sendgrid").expect("1st passes");
    limiter.check("sendgrid").expect("2nd passes");
    limiter
        .check("sendgrid")
        .expect_err("3rd must be rate-limited");

    // Advance past the window.
    clock.advance(Duration::from_secs(61));

    limiter
        .check("sendgrid")
        .expect("after window expires the bucket refills");
}

#[test]
fn reliability_rate_limiter_record_resets_bucket() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let cfg = RateLimitConfig {
        capacity: 1,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::new(cfg, clock);

    limiter.check("sendgrid").expect("1st passes");
    limiter
        .check("sendgrid")
        .expect_err("2nd must be rate-limited");

    // record() refills the bucket.
    limiter.record("sendgrid");
    limiter
        .check("sendgrid")
        .expect("after record() the bucket refills");
}

// ---------------------------------------------------------------------------
// API-surface coverage: exercises every public method/type so the
// `dead_code` lint stays quiet when the module is compiled in the
// test crate (the module is also intended to be wired into
// `lib.rs` once Cluster F ships).
// ---------------------------------------------------------------------------

#[test]
fn reliability_system_clock_returns_monotonic_instant() {
    // Smoke-test the production default clock. We can't assert
    // a specific value, but two consecutive reads must be
    // non-decreasing.
    let c = SystemClock;
    let a = c.now();
    let b = c.now();
    assert!(b >= a, "SystemClock must be monotonic: a={a:?} b={b:?}");
}

#[test]
fn reliability_rate_limit_config_default_is_sane() {
    let cfg = RateLimitConfig::default();
    assert_eq!(cfg.capacity, 100);
    assert_eq!(cfg.window, Duration::from_secs(60));
}

#[test]
fn reliability_rate_limiter_admin_methods() {
    let clock: Arc<dyn Clock> = Arc::new(MockClock::new(Instant::now()));
    let cfg = RateLimitConfig {
        capacity: 5,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::new(cfg, clock);

    // config() returns the same config we constructed with.
    assert_eq!(limiter.config(), cfg);

    // provider_count starts at 0, grows as providers are touched.
    assert_eq!(limiter.provider_count(), 0);
    limiter.check("sendgrid").expect("1st passes");
    assert_eq!(limiter.provider_count(), 1);
    limiter.check("twilio").expect("1st passes");
    assert_eq!(limiter.provider_count(), 2);

    // remaining() reports the post-check remaining count.
    assert_eq!(limiter.remaining("sendgrid"), Some(4));
    // unknown provider → None.
    assert_eq!(limiter.remaining("never-touched"), None);

    // clear() wipes all per-provider state.
    limiter.clear();
    assert_eq!(limiter.provider_count(), 0);
    assert_eq!(limiter.remaining("sendgrid"), None);
}

#[test]
fn reliability_error_variants_construct_and_display() {
    // Exercise every error variant so future callers can rely
    // on them being usable.
    let _ = ReliabilityError::Fallback("smtp 421".to_owned());
    let _ = ReliabilityError::RetryFailed("callback panicked".to_owned());
    let both = ReliabilityError::BothFailed {
        primary: "primary err".to_owned(),
        fallback: "fallback err".to_owned(),
    };
    let msg = both.to_string();
    assert!(msg.contains("primary err"));
    assert!(msg.contains("fallback err"));
    let unknown = ReliabilityError::UnknownDlqId(42);
    assert_eq!(unknown.to_string(), "DLQ entry not found: 42");
}

#[tokio::test]
async fn reliability_provider_failover_accessors() {
    let primary = SucceedingProvider { label: "p" };
    let fallback = FailingProvider { label: "f" };
    let fo = ProviderFailover::new(primary, fallback);

    // Accessors return references to the underlying providers.
    let _ = fo.primary();
    let _ = fo.fallback();

    // send() still works end-to-end.
    let receipt = fo.send("hi").await.expect("primary succeeds");
    assert_eq!(receipt, "p:hi");
}
