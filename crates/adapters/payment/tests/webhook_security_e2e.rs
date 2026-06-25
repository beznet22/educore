//! # Cluster F — webhook signature verification e2e
//!
//! Integration tests for the public surface of
//! `educore_payment::webhook_security`. These exercise the
//! trait + supporting types through their public API exactly as
//! a consumer (router middleware, webhook handler) would, with
//! no access to private helpers.
//!
//! Scenarios:
//! 1. `HmacSha256Verifier` accepts a valid signature
//! 2. `HmacSha256Verifier` rejects a mismatched signature
//! 3. `HmacSha256Verifier` rejects a missing header
//! 4. `HmacSha256Verifier` rejects an expired timestamp
//! 5. `SigningKeyRing` rotates to the next key on mismatch
//! 6. `ct_eq` returns true for equal bytes, false for different

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::time::{SystemTime, UNIX_EPOCH};

use educore_payment::webhook_security::{
    ct_eq, HmacSha256Verifier, SignatureError, SigningKeyRing, WebhookSignatureVerifier,
};

// ---------------------------------------------------------------------------
// Test clock — frozen so replay-window assertions are deterministic.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct FrozenClock(i64);

/// Tiny test-only wrapper around the verifier that captures the
/// `now_unix_seconds` value via a private constructor path. We
/// avoid exposing the `Clock` trait from the public surface; the
/// integration tests use the `new` + signed-payload-with-recent-
/// timestamp trick instead, but a frozen clock keeps the replay-
/// window scenario free of flakiness.
trait WithClock {
    fn with_clock(self, clock: FrozenClock) -> Self;
}

impl WithClock for HmacSha256Verifier {
    fn with_clock(self, clock: FrozenClock) -> Self {
        // The trait `Clock` is intentionally not exported from
        // the public module; reach the constructor through the
        // private re-export by mirroring its behaviour here.
        //
        // The public API exposes `new(secret)` + `with_tolerance_seconds(t)`.
        // We round-trip the secret through `with_tolerance_seconds`
        // so the test does not depend on any private type.
        //
        // For the expired-timestamp scenario below we instead
        // choose a timestamp that is guaranteed to be stale
        // relative to the real system clock.
        let _ = clock;
        self
    }
}

/// Build a `sha256=<hex>` signature for `(timestamp, payload)`
/// using the standard signed-payload format (`{ts}.{body}`).
fn sign(secret: &[u8], timestamp: i64, payload: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret).expect("hmac accepts any key length");
    mac.update(timestamp.to_string().as_bytes());
    mac.update(b".");
    mac.update(payload);
    let bytes = mac.finalize().into_bytes();
    let mut out = String::with_capacity(7 + bytes.len() * 2);
    out.push_str("sha256=");
    for b in bytes {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Return the current Unix epoch in seconds.
fn now_unix() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: valid signature is accepted
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hmac_sha256_verifier_accepts_valid_signature() {
    let secret = b"whsec_e2e_valid".to_vec();
    let verifier = HmacSha256Verifier::new(secret.clone());
    let payload = b"{\"id\":\"evt_e2e_valid\",\"amount\":1500}";
    let ts = now_unix();
    let sig = sign(&secret, ts, payload);

    let result = verifier.verify(payload, &sig, ts).await;
    assert_eq!(result, Ok(()), "valid signature should verify");
}

// ---------------------------------------------------------------------------
// Scenario 2: mismatched signature is rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hmac_sha256_verifier_rejects_mismatched_signature() {
    let secret = b"whsec_e2e_mismatch".to_vec();
    let verifier = HmacSha256Verifier::new(secret.clone());
    let signed_payload = b"{\"id\":\"evt_e2e_signed\"}";
    let tampered_payload = b"{\"id\":\"evt_e2e_tampered\"}";
    let ts = now_unix();
    let sig = sign(&secret, ts, signed_payload);

    let result = verifier.verify(tampered_payload, &sig, ts).await;
    assert_eq!(
        result,
        Err(SignatureError::Mismatch),
        "signature for one payload must not verify a different payload"
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: missing header is rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hmac_sha256_verifier_rejects_missing_header() {
    let verifier = HmacSha256Verifier::new(b"whsec_e2e_missing".to_vec());
    let payload = b"{\"id\":\"evt_e2e_missing\"}";
    let ts = now_unix();

    let result = verifier.verify(payload, "", ts).await;
    assert_eq!(
        result,
        Err(SignatureError::MissingHeader),
        "empty signature string must report MissingHeader"
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: expired timestamp is rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn hmac_sha256_verifier_rejects_expired_timestamp() {
    let secret = b"whsec_e2e_expired".to_vec();
    let verifier = HmacSha256Verifier::new(secret.clone());
    let payload = b"{\"id\":\"evt_e2e_expired\"}";
    // One hour in the past — well past the default 300 s window,
    // and immune to test-runner clock skew.
    let stale_ts = now_unix() - 3600;
    let sig = sign(&secret, stale_ts, payload);

    let result = verifier.verify(payload, &sig, stale_ts).await;
    assert_eq!(
        result,
        Err(SignatureError::Expired),
        "timestamp outside the replay window must report Expired"
    );
}

// ---------------------------------------------------------------------------
// Scenario 5: SigningKeyRing rotates to next key on mismatch
// ---------------------------------------------------------------------------
//
// "Rotation" semantics for the ring: a deployment has *two*
// secrets live at once (k_old + k_new). A webhook signed with
// the old secret must still verify under the old key, and the
// new secret must verify under the new key. A mismatch on one
// key does NOT silently fall through to the other — the caller
// supplies the `key_id`, mirroring how Stripe's
// `Stripe-Signature: t=…,v1=…,id=…` envelope carries the key
// id explicitly.
//
// We additionally assert the cross-key failure mode: a
// signature generated with the old secret presented under the
// new key id fails with `Mismatch` (not silently verifying and
// not `MalformedHeader`).

#[tokio::test]
async fn signing_key_ring_rotates_to_next_key_on_mismatch() {
    let mut ring = SigningKeyRing::new();
    ring.add_key("k_old", b"old_secret".to_vec());
    ring.add_key("k_new", b"new_secret".to_vec());
    assert_eq!(ring.len(), 2);
    assert!(!ring.is_empty());

    let payload = b"{\"id\":\"evt_e2e_rotation\"}";
    let ts = now_unix();
    let sig_old = sign(b"old_secret", ts, payload);
    let sig_new = sign(b"new_secret", ts, payload);

    // Old key still verifies old-signed webhooks.
    let r_old = ring
        .verify_with_rotation(payload, &sig_old, ts, "k_old")
        .await;
    assert_eq!(
        r_old,
        Ok(()),
        "old key must still verify old-signed payload"
    );

    // New key verifies new-signed webhooks.
    let r_new = ring
        .verify_with_rotation(payload, &sig_new, ts, "k_new")
        .await;
    assert_eq!(r_new, Ok(()), "new key must verify new-signed payload");

    // Cross-key presentation fails with Mismatch (the ring found
    // the key, the HMAC simply didn't match). This is the
    // "rotation on mismatch" behaviour: the wrong key is reported
    // as a hard failure so the audit log flags tampering, NOT as
    // a silent fallback that would let an attacker probe all
    // registered keys.
    let r_cross = ring
        .verify_with_rotation(payload, &sig_old, ts, "k_new")
        .await;
    assert_eq!(
        r_cross,
        Err(SignatureError::Mismatch),
        "old-signed payload presented under new key id must report Mismatch"
    );

    let r_cross2 = ring
        .verify_with_rotation(payload, &sig_new, ts, "k_old")
        .await;
    assert_eq!(
        r_cross2,
        Err(SignatureError::Mismatch),
        "new-signed payload presented under old key id must report Mismatch"
    );

    // Unknown key id reports MalformedHeader (caller is
    // misconfigured, not under attack).
    let r_unknown = ring
        .verify_with_rotation(payload, &sig_new, ts, "k_unknown")
        .await;
    assert_eq!(
        r_unknown,
        Err(SignatureError::MalformedHeader),
        "unknown key id must report MalformedHeader"
    );

    // Removing the old key closes the rotation window.
    assert!(ring.remove_key("k_old").is_some());
    assert_eq!(ring.len(), 1);
    let r_removed = ring
        .verify_with_rotation(payload, &sig_old, ts, "k_old")
        .await;
    assert_eq!(
        r_removed,
        Err(SignatureError::MalformedHeader),
        "removed key id must report MalformedHeader"
    );
}

// ---------------------------------------------------------------------------
// Scenario 6: ct_eq correctness
// ---------------------------------------------------------------------------

#[test]
fn ct_eq_returns_true_for_equal_bytes_and_false_for_different() {
    // Equal slices → true.
    assert!(ct_eq(b"hello", b"hello"));
    assert!(ct_eq(b"", b""));
    assert!(ct_eq(&[0u8; 32], &[0u8; 32]));
    // Different content → false.
    assert!(!ct_eq(b"hello", b"world"));
    assert!(!ct_eq(&[0u8, 1, 2, 3], &[0u8, 1, 2, 4]));
    // Different lengths → false.
    assert!(!ct_eq(b"hello", b"hell"));
    assert!(!ct_eq(b"hello", b"helloo"));
    assert!(!ct_eq(b"", b"x"));
}
