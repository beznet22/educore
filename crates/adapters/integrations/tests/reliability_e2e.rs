//! # Cluster F reliability E2E tests
//!
//! Integration tests for `crates/adapters/integrations/src/reliability.rs`,
//! closing the three findings from
//! `docs/audit_reports/findings/wave3-integrations.md`:
//!
//! - ADAPT-INT-005 — webhook retry cap (`WebhookRetryPolicy`).
//! - ADAPT-INT-007 — signing-key rotation (`SigningKeyRing`).
//! - ADAPT-INT-009 — replay protection (`TimestampPolicy` +
//!   ring-level tolerance check).
//!
//! These tests are the authoritative coverage for the public
//! surface defined by the `reliability` module. The unit tests
//! inside that file are smoke tests only.
//!
//! # Module-loading workaround
//!
//! The day-1 quick-wins scope forbids modifying `lib.rs` to
//! register the new `reliability` module. To still test the
//! module's public surface end-to-end, this test file pulls the
//! source in directly via `#[path] mod reliability;`. Once
//! `pub mod reliability;` lands in `lib.rs`, the production
//! import path (`educore_integrations::reliability::*`) will
//! resolve and the `#[path]` attribute on the local
//! `mod reliability;` can be deleted.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

// Mount the source file as a submodule of the integration test so
// the public surface (`WebhookRetryPolicy`, `SigningKeyRing`,
// `TimestampPolicy`, `ct_eq`, ...) is accessible without modifying
// `lib.rs`. The `#[path]` attribute bypasses the normal
// "look in `reliability.rs` next to this file" lookup and points
// at the canonical crate source. `allow(dead_code, ...)` keeps the
// inner `#[cfg(test)] mod tests` block quiet — those smoke tests
// already run under `cargo test --lib`.
#[allow(
    dead_code,
    unused_imports,
    clippy::module_name_repetitions,
    clippy::redundant_closure_for_method_calls
)]
#[path = "../src/reliability.rs"]
mod reliability;

use std::sync::Arc;
use std::time::Duration;

use educore_core::prelude::TestClock;

use self::reliability::{
    ct_eq, SigningError, SigningKeyRing, TimestampPolicy, WebhookRetryPolicy,
    DEFAULT_INITIAL_BACKOFF, DEFAULT_MAX_ATTEMPTS, DEFAULT_MAX_BACKOFF,
    DEFAULT_TIMESTAMP_TOLERANCE_SECONDS,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a `TestClock` pinned to `timestamp_unix_seconds`. Used
/// so each test can pair its `ring.sign(..., t)` with a
/// `verify_with_rotation(..., t)` and observe the replay check
/// independently of real wall-clock time.
fn clock_at(timestamp_unix_seconds: i64) -> Arc<TestClock> {
    let clock = TestClock::new();
    clock.set(educore_core::prelude::Timestamp::from_datetime(
        chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp_unix_seconds, 0)
            .expect("test timestamp must be representable in chrono"),
    ));
    Arc::new(clock)
}

// ---------------------------------------------------------------------------
// ADAPT-INT-005: WebhookRetryPolicy::delay_for_attempt exponential
// ---------------------------------------------------------------------------

#[test]
fn webhook_retry_policy_delay_for_attempt_exponential() {
    let policy = WebhookRetryPolicy::default();
    // attempt 0 and 1 return ZERO (first call is immediate).
    assert_eq!(policy.delay_for_attempt(0), Duration::ZERO);
    assert_eq!(policy.delay_for_attempt(1), Duration::ZERO);
    // From attempt 2 onwards: initial_backoff * 2^(attempt-2).
    assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(1));
    assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(2));
    assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(4));
    assert_eq!(policy.delay_for_attempt(5), Duration::from_secs(8));
}

#[test]
fn webhook_retry_policy_delay_caps_at_max_backoff() {
    // Tight policy so we can observe the cap without waiting
    // through 6 doublings: initial = 1s, cap = 5s.
    let policy = WebhookRetryPolicy {
        max_attempts: 10,
        initial_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(5),
    };
    assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(1));
    assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(2));
    assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(4));
    // 1s * 2^3 = 8s > 5s cap -> clamp.
    assert_eq!(policy.delay_for_attempt(5), Duration::from_secs(5));
    // Any further attempt also clamps.
    assert_eq!(policy.delay_for_attempt(6), Duration::from_secs(5));
    assert_eq!(policy.delay_for_attempt(20), Duration::from_secs(5));
}

#[test]
fn webhook_retry_policy_does_not_panic_on_huge_attempt() {
    // Saturation: even attempt = u32::MAX must not overflow.
    let policy = WebhookRetryPolicy::default();
    let d = policy.delay_for_attempt(u32::MAX);
    assert_eq!(d, DEFAULT_MAX_BACKOFF);
}

// ---------------------------------------------------------------------------
// ADAPT-INT-005: WebhookRetryPolicy::should_retry caps at max_attempts
// ---------------------------------------------------------------------------

#[test]
fn webhook_retry_policy_should_retry_caps_at_max_attempts() {
    let policy = WebhookRetryPolicy::default();
    assert_eq!(policy.max_attempts, DEFAULT_MAX_ATTEMPTS);
    // After attempt 1 has failed, retry -> attempt 2 (allowed).
    assert!(policy.should_retry(1));
    // After attempt 4 has failed, retry -> attempt 5 (allowed,
    // because max_attempts = 5 and 4 < 5).
    assert!(policy.should_retry(4));
    // After attempt 5 has failed, retry -> attempt 6 -> reject.
    assert!(!policy.should_retry(5));
    // Same for any larger attempt.
    assert!(!policy.should_retry(6));
    assert!(!policy.should_retry(100));
}

#[test]
fn webhook_retry_policy_single_attempt_never_retries() {
    let policy = WebhookRetryPolicy {
        max_attempts: 1,
        initial_backoff: DEFAULT_INITIAL_BACKOFF,
        max_backoff: DEFAULT_MAX_BACKOFF,
    };
    assert!(!policy.should_retry(1));
    assert!(!policy.should_retry(2));
}

// ---------------------------------------------------------------------------
// ADAPT-INT-007: SigningKeyRing::sign produces valid HMAC
// ---------------------------------------------------------------------------

#[test]
fn signing_key_ring_sign_produces_valid_hmac() {
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh-secret".to_vec()));
    let payload = b"{\"event\":\"invoice.paid\",\"amount\":4200}";
    let sig = ring
        .sign(payload, "primary_v1", 1_700_000_000)
        .expect("signing with the primary key succeeds");

    // The envelope is `sha256=<64 hex chars>`.
    assert!(sig.starts_with("sha256="));
    assert_eq!(sig.len(), "sha256=".len() + 64);

    // Determinism: signing the same payload twice produces the
    // same bytes. (Timestamp is not mixed into the HMAC per the
    // public contract; bind it externally if needed.)
    let sig2 = ring
        .sign(payload, "primary_v1", 1_700_000_000)
        .expect("signing is deterministic");
    assert_eq!(sig, sig2);
}

#[test]
fn signing_key_ring_sign_rejects_unknown_key() {
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh".to_vec()));
    let err = ring
        .sign(b"payload", "rotated_key", 1)
        .expect_err("unknown key id must fail");
    assert_eq!(err, SigningError::UnknownKey("rotated_key".to_string()));
}

#[test]
fn signing_key_ring_sign_succeeds_with_rotation_key() {
    let mut ring = SigningKeyRing::new(("primary_v1".to_string(), b"a".to_vec()));
    ring.add_rotation_key("rotated_v2".to_string(), b"b".to_vec());
    let sig = ring
        .sign(b"payload", "rotated_v2", 1)
        .expect("rotation key signs successfully");
    assert!(sig.starts_with("sha256="));
}

// ---------------------------------------------------------------------------
// ADAPT-INT-007: SigningKeyRing::verify_with_rotation succeeds with
// primary key
// ---------------------------------------------------------------------------

#[test]
fn signing_key_ring_verify_with_rotation_succeeds_with_primary_key() {
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh-secret".to_vec()));
    let now = 1_700_000_000_i64;
    let ring = ring.with_clock(clock_at(now));

    let payload = b"{\"event\":\"invoice.paid\"}";
    let sig = ring
        .sign(payload, "primary_v1", now)
        .expect("sign succeeds");

    ring.verify_with_rotation(payload, &sig, now)
        .expect("primary key verifies the signature it produced");
}

#[test]
fn signing_key_ring_verify_rejects_tampered_payload() {
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh-secret".to_vec()));
    let now = 1_700_000_000_i64;
    let ring = ring.with_clock(clock_at(now));

    let payload = b"original";
    let tampered = b"tampered";
    let sig = ring
        .sign(payload, "primary_v1", now)
        .expect("sign succeeds");

    let err = ring
        .verify_with_rotation(tampered, &sig, now)
        .expect_err("tampered payload must not verify");
    assert_eq!(err, SigningError::Mismatch);
}

#[test]
fn signing_key_ring_verify_rejects_unknown_signature_format() {
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh-secret".to_vec()));
    let now = 1_700_000_000_i64;
    let ring = ring.with_clock(clock_at(now));
    let err = ring
        .verify_with_rotation(b"payload", "deadbeef", now)
        .expect_err("missing sha256= prefix fails");
    assert_eq!(err, SigningError::Mismatch);
}

// ---------------------------------------------------------------------------
// ADAPT-INT-007: SigningKeyRing::verify_with_rotation succeeds with
// rotation key (cut-over scenario)
// ---------------------------------------------------------------------------

#[test]
fn signing_key_ring_verify_with_rotation_succeeds_with_rotation_key() {
    // Cut-over scenario: the receiver has both the old primary and
    // a freshly-deployed rotation key active. A signature produced
    // with the rotation key must verify.
    let mut ring = SigningKeyRing::new(("primary_v1".to_string(), b"old-secret".to_vec()));
    ring.add_rotation_key("rotated_v2".to_string(), b"new-secret".to_vec());
    let now = 1_700_000_123_i64;
    let ring = ring.with_clock(clock_at(now));

    let payload = b"{\"event\":\"student.admitted\"}";

    // Sender has switched to the new key.
    let sig = ring
        .sign(payload, "rotated_v2", now)
        .expect("rotation key signs");

    // Receiver still verifies the rotation key.
    ring.verify_with_rotation(payload, &sig, now)
        .expect("rotation key verifies during cut-over");

    // And the primary key still verifies signatures produced with
    // itself (in-flight messages from before the cut-over).
    let primary_sig = ring
        .sign(payload, "primary_v1", now)
        .expect("primary still signs");
    ring.verify_with_rotation(payload, &primary_sig, now)
        .expect("primary key still verifies its own signatures");
}

#[test]
fn signing_key_ring_remove_rotation_key_disables_verification() {
    let mut ring = SigningKeyRing::new(("primary_v1".to_string(), b"a".to_vec()));
    ring.add_rotation_key("rotated_v2".to_string(), b"b".to_vec());
    let now = 1_700_000_000_i64;
    let ring = ring.with_clock(clock_at(now));

    let payload = b"x";
    let sig = ring.sign(payload, "rotated_v2", now).expect("sign");

    // Sanity: rotation key works while present.
    ring.verify_with_rotation(payload, &sig, now)
        .expect("rotation key works");

    // Now drop the rotation key.
    let mut ring = ring;
    assert!(ring.remove_rotation_key("rotated_v2"));
    assert_eq!(ring.rotation_key_count(), 0);

    // Re-verifying must fail because no key matches.
    let err = ring
        .verify_with_rotation(payload, &sig, now)
        .expect_err("removed rotation key no longer verifies");
    assert_eq!(err, SigningError::Mismatch);
}

// ---------------------------------------------------------------------------
// ADAPT-INT-009: SigningKeyRing::verify_with_rotation rejects expired
// timestamp
// ---------------------------------------------------------------------------

#[test]
fn signing_key_ring_verify_rejects_expired_timestamp() {
    // Fix the ring's clock at t = 1_700_000_000. A signature with
    // a timestamp 10 minutes in the past must be rejected (default
    // tolerance is 5 minutes).
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh".to_vec()))
        .with_clock(clock_at(1_700_000_000));

    let payload = b"payload";
    let signed_at = 1_700_000_000_i64 - 600; // 10 min in the past
    let sig = ring
        .sign(payload, "primary_v1", signed_at)
        .expect("signing ignores clock");

    let err = ring
        .verify_with_rotation(payload, &sig, signed_at)
        .expect_err("stale timestamp is rejected");

    match err {
        SigningError::Expired {
            age_seconds,
            tolerance_seconds,
        } => {
            assert_eq!(age_seconds, 600);
            assert_eq!(tolerance_seconds, DEFAULT_TIMESTAMP_TOLERANCE_SECONDS);
        }
        other => panic!("expected Expired, got {other:?}"),
    }
}

#[test]
fn signing_key_ring_verify_accepts_future_within_tolerance() {
    // Symmetric tolerance: a signature up to 5 minutes in the
    // future must also verify (covers client clock skew).
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh".to_vec()))
        .with_clock(clock_at(1_700_000_000));

    let payload = b"payload";
    let signed_at = 1_700_000_000_i64 + 60; // 1 min in the future
    let sig = ring
        .sign(payload, "primary_v1", signed_at)
        .expect("signing ignores clock");

    ring.verify_with_rotation(payload, &sig, signed_at)
        .expect("future timestamp within tolerance verifies");
}

// ---------------------------------------------------------------------------
// ADAPT-INT-009: TimestampPolicy rejects timestamp older than tolerance
// ---------------------------------------------------------------------------

#[test]
fn timestamp_policy_default_is_five_minutes() {
    let p = TimestampPolicy::default();
    assert_eq!(p.tolerance_seconds, DEFAULT_TIMESTAMP_TOLERANCE_SECONDS);
    assert_eq!(p.tolerance_seconds, 300);
}

#[test]
fn timestamp_policy_custom_tolerance_is_honoured() {
    // 30-second tolerance with a 60-second-old timestamp.
    let ring = SigningKeyRing::new(("primary_v1".to_string(), b"shh".to_vec()))
        .with_clock(clock_at(1_700_000_000))
        .with_timestamp_policy(TimestampPolicy {
            tolerance_seconds: 30,
        });

    let payload = b"payload";
    let signed_at = 1_700_000_000_i64 - 60; // 60 s in the past
    let sig = ring
        .sign(payload, "primary_v1", signed_at)
        .expect("signing ignores clock");

    let err = ring
        .verify_with_rotation(payload, &sig, signed_at)
        .expect_err("60s-old timestamp must fail a 30s tolerance");
    match err {
        SigningError::Expired {
            age_seconds,
            tolerance_seconds,
        } => {
            assert_eq!(age_seconds, 60);
            assert_eq!(tolerance_seconds, 30);
        }
        other => panic!("expected Expired, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// ct_eq — constant-time byte comparison helper
// ---------------------------------------------------------------------------

#[test]
fn ct_eq_returns_true_for_equal_slices() {
    assert!(ct_eq(b"", b""));
    assert!(ct_eq(b"hello", b"hello"));
    assert!(ct_eq(&[0xde, 0xad, 0xbe, 0xef], &[0xde, 0xad, 0xbe, 0xef]));
}

#[test]
fn ct_eq_returns_false_for_unequal_slices() {
    assert!(!ct_eq(b"hello", b"world"));
    assert!(!ct_eq(b"hello", b"hell"));
    assert!(!ct_eq(b"", b"x"));
}

#[test]
fn ct_eq_length_mismatch_is_fast_path() {
    // Length mismatch short-circuits without scanning; we can't
    // measure timing in a unit test, but we can verify the
    // behaviour is correct on every length-mismatch combination.
    for len_a in 0..8 {
        for len_b in 0..8 {
            if len_a == len_b {
                continue;
            }
            let a = vec![0u8; len_a];
            let b = vec![0u8; len_b];
            assert!(!ct_eq(&a, &b), "len_a={len_a} len_b={len_b} must differ");
        }
    }
}
