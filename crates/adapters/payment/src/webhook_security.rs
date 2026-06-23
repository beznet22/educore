//! # Webhook signature verification + signing-key rotation
//!
//! Async verifier trait, an HMAC-SHA256 reference implementation
//! with constant-time comparison + replay window enforcement,
//! and a multi-key ring for staged secret rotation.
//!
//! Addresses audit findings:
//! - ADAPT-PAY-005 — webhook signature verification was
//!   incomplete (no constant-time check, no replay window).
//! - ADAPT-PAY-008 — added explicit PCI scope markers on every
//!   code path that handles webhook secrets.
//!
//! All public functions and impls document whether they touch
//! secret material via the `// PCI-SCOPE: signature-verification`
//! marker so the scope is auditable in code review and in
//! `git grep`.

use std::collections::BTreeMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Typed alias for the HMAC-SHA256 MAC used by the reference
/// verifier. Not exported because the trait is the contract.
type HmacSha256 = Hmac<Sha256>;

/// The default replay window. Stripe and most payment gateways
/// default to a 5-minute (300 s) tolerance; we match that.
pub const DEFAULT_TOLERANCE_SECONDS: i64 = 300;

/// Outcome of a signature verification attempt.
///
/// `Mismatch` and `Expired` both indicate the request should be
/// rejected; `MissingHeader` and `MalformedHeader` indicate the
/// caller failed to assemble a usable `X-Educore-Signature`-style
/// header before invoking the verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureError {
    /// The recomputed signature did not match the header value.
    /// May indicate tampering, a wrong key, or a missing header
    /// passed as a syntactically-valid hex string.
    Mismatch,
    /// The timestamp on the signature is older than the
    /// configured replay window. The request is rejected to
    /// prevent replay attacks.
    Expired,
    /// The signature header was absent (empty string).
    MissingHeader,
    /// The signature header was present but did not match the
    /// expected `sha256=<64 hex chars>` wire format.
    MalformedHeader,
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Mismatch => "signature mismatch",
            Self::Expired => "signature timestamp outside replay window",
            Self::MissingHeader => "signature header missing",
            Self::MalformedHeader => "signature header malformed",
        };
        // Deliberately do not include any secret-derived bytes in
        // the diagnostic — see PCI scope marker on the verify
        // methods below.
        write!(f, "{kind}")
    }
}

impl std::error::Error for SignatureError {}

/// Async verifier contract.
///
/// The trait is object-safe (`Send + Sync`, no associated types,
/// single method) so consumers can stash a `Box<dyn
/// WebhookSignatureVerifier>` in router middleware. Adapters that
/// only need synchronous verification can still implement this
/// trait and return immediately from the `async fn`.
#[async_trait]
pub trait WebhookSignatureVerifier: Send + Sync {
    /// Verify the signature on `payload` against the
    /// provider-supplied `signature` header value and the
    /// `timestamp` (Unix epoch seconds) embedded in the header.
    ///
    /// `signature` is expected in the wire format
    /// `sha256=<64 lower-case hex chars>`. Implementations MUST
    /// reject empty input as [`SignatureError::MissingHeader`]
    /// and non-conforming input as [`SignatureError::MalformedHeader`].
    async fn verify(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: i64,
    ) -> Result<(), SignatureError>;
}

/// Clock source used by [`HmacSha256Verifier`] to compute the
/// current Unix epoch in seconds.
///
/// Wrapping the call behind a trait lets the tests inject a
/// frozen clock without exposing an extra constructor argument
/// on every verifier. The production path uses the
/// [`SystemClock`] implementation, which reads
/// [`SystemTime::now`].
pub trait Clock: Send + Sync {
    /// Return the current time as Unix epoch seconds.
    fn now_unix_seconds(&self) -> i64;
}

/// Production clock — reads [`SystemTime::now`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_seconds(&self) -> i64 {
        // PCI-SCOPE: signature-verification — non-secret;
        // included for completeness in the audit trail.
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
            // Pre-1970 systems: saturate to 0 rather than panic.
            // The replay-window check is still safe — every
            // legitimate timestamp will be `>= now`.
            Err(_) => 0,
        }
    }
}

/// HMAC-SHA256 webhook signature verifier.
///
/// This is the reference implementation of
/// [`WebhookSignatureVerifier`]. It:
/// 1. Validates the `sha256=<hex>` wire format.
/// 2. Enforces the replay window (`now - timestamp <= tolerance`).
/// 3. Recomputes `HMAC_SHA256(secret, "{timestamp}.{payload}")`
///    (Stripe-compatible signed payload format).
/// 4. Compares the recomputed MAC against the supplied signature
///    using [`ct_eq`] (constant time over the expected length).
///
/// The `secret` is stored as raw bytes (not a `String`) and the
/// `Debug` impl redacts it so logs and panic messages cannot leak
/// the material. See PCI scope marker on `Debug`.
pub struct HmacSha256Verifier {
    // PCI-SCOPE: signature-verification — webhook secret held as
    // raw bytes. Never logged; `Debug` impl below redacts.
    secret: Vec<u8>,
    tolerance_seconds: i64,
    clock: Box<dyn Clock>,
}

impl HmacSha256Verifier {
    /// Construct a verifier with the default 300 s replay window
    /// and the production [`SystemClock`].
    #[must_use]
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self {
            secret: secret.into(),
            tolerance_seconds: DEFAULT_TOLERANCE_SECONDS,
            clock: Box::new(SystemClock),
        }
    }

    /// Override the replay window (seconds). A request whose
    /// `timestamp` is older than `now - tolerance_seconds` (or
    /// more than `tolerance_seconds` in the future) is rejected
    /// with [`SignatureError::Expired`].
    #[must_use]
    pub fn with_tolerance_seconds(mut self, tolerance_seconds: i64) -> Self {
        self.tolerance_seconds = tolerance_seconds;
        self
    }

    /// Override the clock source. Intended for deterministic
    /// tests; production code should construct via [`Self::new`].
    #[must_use]
    pub fn with_clock(mut self, clock: impl Clock + 'static) -> Self {
        self.clock = Box::new(clock);
        self
    }

    /// Recompute the expected signature for `(timestamp, payload)`.
    ///
    /// Returns the raw 32-byte MAC; the caller is responsible for
    /// hex-encoding it before transmission.
    fn compute(&self, payload: &[u8], timestamp: i64) -> Result<[u8; 32], SignatureError> {
        // PCI-SCOPE: signature-verification — handles webhook
        // secrets (HMAC key) and the unverified payload bytes.
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|_| SignatureError::MalformedHeader)?;
        // Stripe-compatible signed payload: "{ts}.{body}".
        // The literal `.` separator is part of the contract and
        // cannot be changed without bumping the verifier version.
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        let bytes = mac.finalize().into_bytes();
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }

    /// Encode 32 bytes as 64 lower-case hex characters. Exposed
    /// (private) so tests can build expected signatures without
    /// pulling in the `hex` crate.
    fn hex_encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0f) as usize] as char);
        }
        out
    }
}

impl fmt::Debug for HmacSha256Verifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PCI-SCOPE: signature-verification — redact secret +
        // tolerance to keep the `Debug` output safe for logs.
        f.debug_struct("HmacSha256Verifier")
            .field("secret", &format_args!("<redacted {} bytes>", self.secret.len()))
            .field("tolerance_seconds", &self.tolerance_seconds)
            .finish()
    }
}

#[async_trait]
impl WebhookSignatureVerifier for HmacSha256Verifier {
    async fn verify(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: i64,
    ) -> Result<(), SignatureError> {
        // PCI-SCOPE: signature-verification — this is the hot path
        // that touches the webhook secret + the unverified payload.
        // Implementation MUST stay constant-time over the expected
        // signature length and MUST reject expired timestamps
        // before doing any HMAC work (to avoid leaking timing info
        // about secret material to an attacker probing the replay
        // window).

        // 1. Header presence.
        if signature.is_empty() {
            return Err(SignatureError::MissingHeader);
        }
        // 2. Wire format: `sha256=` + 64 hex chars.
        let hex = match signature.strip_prefix("sha256=") {
            Some(hex) => hex,
            None => return Err(SignatureError::MalformedHeader),
        };
        if hex.len() != 64 {
            return Err(SignatureError::MalformedHeader);
        }
        // Cheap pre-check that every char is hex. We still do the
        // constant-time MAC comparison below; this just produces
        // the right error variant for bad wire-format input.
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(SignatureError::MalformedHeader);
        }

        // 3. Replay window. Reject BEFORE recomputing the MAC so
        // the timing of `Expired` does not depend on secret bytes.
        let now = self.clock.now_unix_seconds();
        let age = now.saturating_sub(timestamp);
        let skew = timestamp.saturating_sub(now);
        if age > self.tolerance_seconds || skew > self.tolerance_seconds {
            return Err(SignatureError::Expired);
        }

        // 4. Recompute and compare in constant time.
        let expected = self.compute(payload, timestamp)?;
        let provided = match hex.as_bytes().chunks_exact(2).enumerate().try_fold(
            [0u8; 32],
            |mut acc, (i, pair)| {
                let hi = hex_nibble(pair[0])?;
                let lo = hex_nibble(pair[1])?;
                acc[i] = (hi << 4) | lo;
                Some(acc)
            },
        ) {
            Some(bytes) => bytes,
            None => return Err(SignatureError::MalformedHeader),
        };

        if ct_eq(&expected, &provided) {
            Ok(())
        } else {
            Err(SignatureError::Mismatch)
        }
    }
}

/// Decode a single ASCII hex nibble to its 0..=15 value, returning
/// `None` for non-hex bytes.
fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// In-memory ring of [`HmacSha256Verifier`] instances keyed by an
/// adapter-defined key id (e.g. the Stripe webhook's `id` claim,
/// or a deployment-specific rotation label).
///
/// Operators add the **next** key to the ring before rotating
/// the gateway's signing key, so existing in-flight webhooks
/// signed with the previous key still verify while new webhooks
/// are signed with the new key. After the replay window passes
/// the old key can be removed with [`SigningKeyRing::remove_key`].
///
/// Lookup is by `key_id` via [`SigningKeyRing::verify_with_rotation`].
/// The ring does NOT auto-fallback to other keys on mismatch —
/// the caller supplies the `key_id` they expect, which is what
/// the gateway's webhook payload carries (e.g. Stripe's
/// `t=…,v1=…,id=…` envelope includes the key id).
#[derive(Default)]
pub struct SigningKeyRing {
    // BTreeMap keeps iteration order deterministic for tests.
    keys: BTreeMap<String, HmacSha256Verifier>,
    default_tolerance_seconds: i64,
}

impl SigningKeyRing {
    /// Construct an empty ring with the default replay window.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keys: BTreeMap::new(),
            default_tolerance_seconds: DEFAULT_TOLERANCE_SECONDS,
        }
    }

    /// Override the default replay window for keys added later.
    /// Existing keys keep whatever window they were constructed
    /// with.
    #[must_use]
    pub fn with_default_tolerance_seconds(mut self, tolerance_seconds: i64) -> Self {
        self.default_tolerance_seconds = tolerance_seconds;
        self
    }

    /// Register a verifier under `key_id`. Subsequent calls to
    /// [`verify_with_rotation`](Self::verify_with_rotation) with
    /// the same `key_id` will use this verifier.
    pub fn add_key(&mut self, key_id: impl Into<String>, secret: impl Into<Vec<u8>>) {
        let verifier = HmacSha256Verifier::new(secret)
            .with_tolerance_seconds(self.default_tolerance_seconds);
        self.keys.insert(key_id.into(), verifier);
    }

    /// Register a pre-built verifier under `key_id`. Useful when
    /// a key needs a custom clock (e.g. for tests).
    pub fn add_verifier(
        &mut self,
        key_id: impl Into<String>,
        verifier: HmacSha256Verifier,
    ) {
        self.keys.insert(key_id.into(), verifier);
    }

    /// Remove a key from the ring. Returns the removed verifier
    /// (so the caller can recover the secret material under a
    /// secure wipe path) or `None` if the key id was unknown.
    pub fn remove_key(&mut self, key_id: &str) -> Option<HmacSha256Verifier> {
        self.keys.remove(key_id)
    }

    /// Number of keys currently in the ring.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// `true` iff the ring has no keys registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Verify `signature` against the verifier registered under
    /// `key_id`. Returns [`SignatureError::MalformedHeader`] if
    /// the `key_id` is unknown — distinguishing "unknown key"
    /// from "known key + bad signature" so the audit trail can
    /// tell a misconfigured consumer apart from a tampered
    /// request.
    pub async fn verify_with_rotation(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: i64,
        key_id: &str,
    ) -> Result<(), SignatureError> {
        // PCI-SCOPE: signature-verification — dispatches to the
        // verifier bound to `key_id`. The map lookup is O(log n)
        // and does not touch secret bytes.
        let verifier = self
            .keys
            .get(key_id)
            .ok_or(SignatureError::MalformedHeader)?;
        verifier.verify(payload, signature, timestamp).await
    }
}

impl fmt::Debug for SigningKeyRing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PCI-SCOPE: signature-verification — list key ids only,
        // never any verifier internals that could leak secret
        // material via log aggregation.
        f.debug_struct("SigningKeyRing")
            .field("key_ids", &self.keys.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Constant-time byte-slice equality.
///
/// Returns `false` immediately when the slices have different
/// lengths (the length leak is unavoidable, but it does not
/// expose any byte of either secret). When the lengths match,
/// the comparison iterates over the full length regardless of
/// where the first mismatch occurs, so the timing profile does
/// not reveal the position of the first differing byte.
///
/// This is the hand-rolled equivalent of `subtle::ConstantTimeEq`
/// — the engine does not depend on the `subtle` crate, so the
/// loop is inlined here. **Never compare secret bytes with `==`.**
#[must_use]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    /// Frozen clock that always reports the configured epoch.
    #[derive(Debug, Clone, Copy)]
    struct FrozenClock(i64);

    impl Clock for FrozenClock {
        fn now_unix_seconds(&self) -> i64 {
            self.0
        }
    }

    fn make_signature(secret: &[u8], timestamp: i64, payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).expect("hmac accepts any key length");
        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);
        format!("sha256={}", HmacSha256Verifier::hex_encode(&mac.finalize().into_bytes()))
    }

    #[test]
    fn ct_eq_returns_true_for_equal_bytes() {
        assert!(ct_eq(b"hello", b"hello"));
        assert!(ct_eq(b"", b""));
    }

    #[test]
    fn ct_eq_returns_false_for_different_bytes() {
        assert!(!ct_eq(b"hello", b"world"));
        assert!(!ct_eq(b"hello", b"hell"));
        // Length mismatch must return false (length leak is OK).
        assert!(!ct_eq(b"hello", b"helloo"));
    }

    #[tokio::test]
    async fn hmac_sha256_verifier_accepts_valid_signature() {
        let secret = b"whsec_test_secret".to_vec();
        let verifier = HmacSha256Verifier::new(secret.clone()).with_clock(FrozenClock(1_700_000_000));
        let payload = b"{\"id\":\"evt_001\"}";
        let ts = 1_700_000_000;
        let sig = make_signature(&secret, ts, payload);
        assert_eq!(verifier.verify(payload, &sig, ts).await, Ok(()));
    }

    #[tokio::test]
    async fn hmac_sha256_verifier_rejects_mismatched_signature() {
        let secret = b"whsec_test_secret".to_vec();
        let verifier =
            HmacSha256Verifier::new(secret.clone()).with_clock(FrozenClock(1_700_000_000));
        let payload = b"{\"id\":\"evt_001\"}";
        let other_payload = b"{\"id\":\"evt_002\"}";
        let ts = 1_700_000_000;
        let sig = make_signature(&secret, ts, other_payload);
        assert_eq!(
            verifier.verify(payload, &sig, ts).await,
            Err(SignatureError::Mismatch)
        );
    }

    #[tokio::test]
    async fn hmac_sha256_verifier_rejects_missing_header() {
        let verifier =
            HmacSha256Verifier::new(b"whsec_test_secret".to_vec()).with_clock(FrozenClock(1_700_000_000));
        assert_eq!(
            verifier.verify(b"{}", "", 1_700_000_000).await,
            Err(SignatureError::MissingHeader)
        );
    }

    #[tokio::test]
    async fn hmac_sha256_verifier_rejects_malformed_header() {
        let verifier =
            HmacSha256Verifier::new(b"whsec_test_secret".to_vec()).with_clock(FrozenClock(1_700_000_000));
        // Wrong prefix.
        assert_eq!(
            verifier
                .verify(b"{}", "md5=abcdef", 1_700_000_000)
                .await,
            Err(SignatureError::MalformedHeader)
        );
        // Right prefix, wrong length.
        assert_eq!(
            verifier
                .verify(b"{}", "sha256=deadbeef", 1_700_000_000)
                .await,
            Err(SignatureError::MalformedHeader)
        );
        // Right prefix, non-hex chars.
        assert_eq!(
            verifier
                .verify(
                    b"{}",
                    "sha256=zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                    1_700_000_000
                )
                .await,
            Err(SignatureError::MalformedHeader)
        );
    }

    #[tokio::test]
    async fn hmac_sha256_verifier_rejects_expired_timestamp() {
        let secret = b"whsec_test_secret".to_vec();
        let now = 1_700_000_000_i64;
        let verifier = HmacSha256Verifier::new(secret.clone()).with_clock(FrozenClock(now));
        let payload = b"{\"id\":\"evt_old\"}";
        // 1 hour old, well past the 300 s window.
        let stale_ts = now - 3600;
        let sig = make_signature(&secret, stale_ts, payload);
        assert_eq!(
            verifier.verify(payload, &sig, stale_ts).await,
            Err(SignatureError::Expired)
        );
    }

    #[tokio::test]
    async fn signing_key_ring_dispatches_to_named_key() {
        // Use a recent timestamp (within replay tolerance) instead
        // of a hardcoded 2023 value, which the verifier correctly
        // rejects as expired.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_secs() as i64;
        let mut ring = SigningKeyRing::new();
        ring.add_key("k_old", b"old_secret".to_vec());
        ring.add_key("k_new", b"new_secret".to_vec());

        let payload = b"{\"id\":\"evt_001\"}";
        let ts = now;
        let sig_old = make_signature(b"old_secret", ts, payload);
        let sig_new = make_signature(b"new_secret", ts, payload);

        // Each signature verifies against its own key.
        assert_eq!(
            ring.verify_with_rotation(payload, &sig_old, ts, "k_old").await,
            Ok(())
        );
        assert_eq!(
            ring.verify_with_rotation(payload, &sig_new, ts, "k_new").await,
            Ok(())
        );

        // Cross-key verification must fail with Mismatch (not
        // MalformedHeader) — the ring found the key, just the
        // HMAC didn't match.
        assert_eq!(
            ring.verify_with_rotation(payload, &sig_old, ts, "k_new").await,
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            ring.verify_with_rotation(payload, &sig_new, ts, "k_old").await,
            Err(SignatureError::Mismatch)
        );

        // Unknown key id returns MalformedHeader (deliberate,
        // so the audit trail distinguishes "misconfigured
        // consumer" from "tampered request").
        assert_eq!(
            ring.verify_with_rotation(payload, &sig_new, ts, "k_unknown").await,
            Err(SignatureError::MalformedHeader)
        );
    }

    #[test]
    fn signing_key_ring_add_remove() {
        let mut ring = SigningKeyRing::new();
        assert!(ring.is_empty());
        ring.add_key("k1", b"secret".to_vec());
        assert_eq!(ring.len(), 1);
        assert!(!ring.is_empty());
        assert!(ring.remove_key("k1").is_some());
        assert!(ring.is_empty());
        assert!(ring.remove_key("k1").is_none());
    }
}
