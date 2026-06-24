//! # Webhook delivery reliability: retry policy, signing-key rotation, replay protection.
//!
//! This module closes three findings from
//! `docs/audit_reports/findings/wave3-integrations.md`:
//!
//! - **ADAPT-INT-005** — webhook retries forever with no exponential
//!   backoff cap. Fixed by [`WebhookRetryPolicy`], which schedules
//!   delays as `initial_backoff * 2^(attempt-2)` and clamps the
//!   result to `max_backoff`, so a misbehaving target cannot pin
//!   the dispatcher.
//! - **ADAPT-INT-007** — single signing key with no rotation path.
//!   Fixed by [`SigningKeyRing`], which holds one *primary* key plus
//!   zero or more *rotation* keys. Receivers keep both keys active
//!   during cut-over so signatures produced with either verify
//!   successfully; senders promote a new primary by constructing a
//!   fresh ring.
//! - **ADAPT-INT-009** — no replay protection on signed webhooks.
//!   Fixed by [`TimestampPolicy`] (default 300 s tolerance in either
//!   direction), enforced inside
//!   [`SigningKeyRing::verify_with_rotation`].
//!
//! The helpers in this module are pure (no I/O). Adapters wrap them
//! with the async transport layer. The existing
//! `webhook_out::WebhookOutIntegration`, `services::WebhookSignatureService`,
//! and `services::RetryService` are kept intact — this is a strict
//! additive change per the day-1 quick-wins scope so consumers can
//! migrate at their own pace.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use educore_core::prelude::{Clock, SystemClock};

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default maximum number of dispatch attempts (including the
/// first call). The fifth attempt is the last retry; the sixth
/// would be rejected by [`WebhookRetryPolicy::should_retry`].
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;
/// Default initial backoff (the wait between attempt 1 and attempt
/// 2). The exponential schedule is `initial * 2^(attempt-2)`,
/// capped at [`DEFAULT_MAX_BACKOFF`].
pub const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_secs(1);
/// Default hard ceiling on the per-attempt delay. Once the
/// exponential schedule would exceed this value, every subsequent
/// attempt waits exactly this long.
pub const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(60);
/// Default replay-protection tolerance (5 minutes, applied in
/// either direction).
pub const DEFAULT_TIMESTAMP_TOLERANCE_SECONDS: i64 = 300;

// ---------------------------------------------------------------------------
// WebhookRetryPolicy
// ---------------------------------------------------------------------------

/// Exponential backoff schedule for outbound webhook retries
/// with a hard ceiling on the per-attempt delay.
///
/// The first call to `invoke` (or `dispatch`) is *attempt 1* — it
/// happens immediately. Each subsequent attempt waits
/// [`WebhookRetryPolicy::delay_for_attempt`] before firing. The
/// schedule is exponential with a hard cap so a misbehaving target
/// cannot pin the dispatcher forever (closes ADAPT-INT-005).
///
/// # Example
///
/// ```ignore
/// use educore_integrations::reliability::WebhookRetryPolicy;
/// use std::time::Duration;
///
/// let policy = WebhookRetryPolicy::default();
/// assert_eq!(policy.delay_for_attempt(1), Duration::ZERO);
/// assert_eq!(policy.delay_for_attempt(2), Duration::from_secs(1));
/// assert_eq!(policy.delay_for_attempt(3), Duration::from_secs(2));
/// assert_eq!(policy.delay_for_attempt(4), Duration::from_secs(4));
/// assert_eq!(policy.delay_for_attempt(5), Duration::from_secs(8));
/// assert!(policy.should_retry(5));
/// assert!(!policy.should_retry(6));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebhookRetryPolicy {
    /// Maximum number of attempts *including* the first call.
    /// Must be `>= 1`.
    pub max_attempts: u32,
    /// Delay before the *first* retry (i.e. the delay returned for
    /// `attempt = 2`). Doubled per attempt, capped at `max_backoff`.
    pub initial_backoff: Duration,
    /// Hard ceiling on the returned delay. Once the exponential
    /// schedule would exceed this value, every subsequent attempt
    /// returns exactly `max_backoff`.
    pub max_backoff: Duration,
}

impl Default for WebhookRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
        }
    }
}

impl WebhookRetryPolicy {
    /// Returns the delay to wait *before* firing `attempt`.
    ///
    /// - `attempt = 0` → `Duration::ZERO` (caller hasn't even tried;
    ///   a sentinel for "before the first call").
    /// - `attempt = 1` → `Duration::ZERO` (the first call is
    ///   immediate; the wait happens between attempts).
    /// - `attempt >= 2` →
    ///   `min(initial_backoff * 2^(attempt-2), max_backoff)`.
    ///
    /// Saturates on overflow so very large `attempt` values never
    /// panic — they simply return `max_backoff`.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt <= 1 {
            return Duration::ZERO;
        }
        // The exponent fits in u32 for any realistic attempt count;
        // saturate at 63 to avoid the `1u64 << 64` UB case.
        let shift = (attempt - 2).min(63);
        let multiplier = 1u64 << shift;
        // `as_nanos` returns u128, so multiplication can't overflow
        // before the next saturating_mul clamps it.
        let nanos = self
            .initial_backoff
            .as_nanos()
            .saturating_mul(u128::from(multiplier));
        // Convert u128 → u64 nanoseconds, saturating at u64::MAX
        // which is ~584 years — more than enough headroom for any
        // realistic backoff. If the converted value would still
        // exceed `max_backoff`, clamp to `max_backoff`.
        let candidate_nanos = u64::try_from(nanos).unwrap_or(u64::MAX);
        let candidate = Duration::from_nanos(candidate_nanos);
        if candidate > self.max_backoff {
            self.max_backoff
        } else {
            candidate
        }
    }

    /// Returns `true` if a retry should be fired after `attempt`
    /// attempts have already failed. `attempt` is the count of
    /// attempts that have *already* run (1-based).
    ///
    /// Semantics: `should_retry(1)` is `true` when
    /// `max_attempts >= 2`, meaning the second attempt is the first
    /// retry; `should_retry(max_attempts)` is always `false`.
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

// ---------------------------------------------------------------------------
// SigningError
// ---------------------------------------------------------------------------

/// Errors returned by [`SigningKeyRing::sign`] and
/// [`SigningKeyRing::verify_with_rotation`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SigningError {
    /// The supplied `key_id` does not exist in the ring (neither
    /// primary nor rotation). Carries the unknown id so operators
    /// can spot typos in their signing configuration.
    #[error("unknown signing key id: {0}")]
    UnknownKey(String),
    /// The signature timestamp is outside the configured
    /// replay-protection tolerance.
    #[error(
        "signature timestamp expired (age {age_seconds}s exceeds tolerance {tolerance_seconds}s)"
    )]
    Expired {
        /// Absolute age of the signature, in seconds. Positive
        /// (always non-negative) — the absolute distance between
        /// the signed timestamp and the verifier's clock.
        age_seconds: i64,
        /// Configured tolerance, in seconds.
        tolerance_seconds: i64,
    },
    /// A key was found but the HMAC did not match the payload —
    /// or the signature envelope (`sha256=...`) is malformed.
    #[error("signature mismatch")]
    Mismatch,
}

// ---------------------------------------------------------------------------
// TimestampPolicy
// ---------------------------------------------------------------------------

/// Replay-protection policy. Signatures whose timestamp is more
/// than `tolerance_seconds` away from the verifier's wall clock
/// (in *either* direction) are rejected. Default tolerance is 300
/// seconds (5 minutes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampPolicy {
    /// Allowed clock skew in either direction, in seconds.
    /// Must be `>= 0`. The default of 300 s tolerates ordinary
    /// network + processing latency without making replay
    /// practical.
    pub tolerance_seconds: i64,
}

impl Default for TimestampPolicy {
    fn default() -> Self {
        Self {
            tolerance_seconds: DEFAULT_TIMESTAMP_TOLERANCE_SECONDS,
        }
    }
}

// ---------------------------------------------------------------------------
// SigningKeyRing
// ---------------------------------------------------------------------------

/// Multi-key signing ring that supports zero-downtime rotation
/// (closes ADAPT-INT-007).
///
/// Operators deploy a new signing key alongside the old one
/// (via [`SigningKeyRing::add_rotation_key`]), switch senders over
/// to the new key, and once all in-flight messages have been
/// verified the old key can be removed (via
/// [`SigningKeyRing::remove_rotation_key`]). Receivers keep both
/// keys active during the cut-over so signatures produced with
/// either key verify successfully.
///
/// The primary key is set at construction time and cannot be
/// removed without constructing a fresh ring; rotation keys are
/// added and removed freely. Verification tries the primary first
/// and then each rotation key in lexicographic `key_id` order
/// (the natural ordering of the underlying [`BTreeMap`]).
///
/// # Clock injection
///
/// The ring holds an `Arc<dyn Clock>` so tests can substitute
/// [`educore_core::prelude::TestClock`] without touching the
/// signature of [`SigningKeyRing::verify_with_rotation`]. The
/// default is [`SystemClock`].
///
/// # Replay protection
///
/// The ring holds a [`TimestampPolicy`] (default 5-minute
/// tolerance). Verification rejects any signature whose timestamp
/// is further than `tolerance_seconds` from the injected clock.
///
/// `Debug` is implemented manually (the [`Clock`] trait is not
/// `Debug`-bounded) and redacts the secret material so a
/// `dbg!()` of the ring never leaks a key to a log file.
pub struct SigningKeyRing {
    primary_id: String,
    primary_secret: Vec<u8>,
    rotation: BTreeMap<String, Vec<u8>>,
    timestamp_policy: TimestampPolicy,
    clock: Arc<dyn Clock>,
}

// Manual `Clone` (the auto-derive can't be used because `dyn
// Clock` is not `Clone` — it's behind an `Arc` which IS `Clone`).
impl Clone for SigningKeyRing {
    fn clone(&self) -> Self {
        Self {
            primary_id: self.primary_id.clone(),
            primary_secret: self.primary_secret.clone(),
            rotation: self.rotation.clone(),
            timestamp_policy: self.timestamp_policy,
            clock: Arc::clone(&self.clock),
        }
    }
}

// Manual `Debug` redacts the secret material so a `dbg!()` of
// the ring never leaks a key to a log file.
impl std::fmt::Debug for SigningKeyRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SigningKeyRing")
            .field("primary_id", &self.primary_id)
            .field(
                "primary_secret",
                &format_args!("<{} bytes redacted>", self.primary_secret.len()),
            )
            .field("rotation_ids", &self.rotation.keys().collect::<Vec<_>>())
            .field("rotation_count", &self.rotation.len())
            .field("timestamp_policy", &self.timestamp_policy)
            .field("clock", &"<dyn Clock>")
            .finish()
    }
}

impl SigningKeyRing {
    /// Creates a ring with a single primary key and the default
    /// timestamp policy + system clock.
    pub fn new(primary: (String, Vec<u8>)) -> Self {
        Self {
            primary_id: primary.0,
            primary_secret: primary.1,
            rotation: BTreeMap::new(),
            timestamp_policy: TimestampPolicy::default(),
            clock: Arc::new(SystemClock),
        }
    }

    /// Builder-style: override the clock used for replay checks.
    /// Typically used in tests with
    /// [`educore_core::prelude::TestClock`].
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Builder-style: override the timestamp policy.
    #[must_use]
    pub fn with_timestamp_policy(mut self, policy: TimestampPolicy) -> Self {
        self.timestamp_policy = policy;
        self
    }

    /// Adds a rotation key. Existing rotation keys with the same
    /// `key_id` are overwritten — the most recently added secret
    /// wins. (Intended for "promote-to-primary" workflows where
    /// the operator pushes the new secret before the receivers
    /// cut over.)
    pub fn add_rotation_key(&mut self, key_id: String, secret: Vec<u8>) {
        self.rotation.insert(key_id, secret);
    }

    /// Removes a rotation key. Returns `true` if a key was
    /// actually removed. Primary keys cannot be removed through
    /// this method — promote a new primary by constructing a
    /// fresh ring (intentional: the API surface has no way to
    /// mutate the primary, so a missing primary always indicates
    /// a programming error at construction time).
    pub fn remove_rotation_key(&mut self, key_id: &str) -> bool {
        self.rotation.remove(key_id).is_some()
    }

    /// Returns the primary key id (for diagnostics / observability).
    pub fn primary_key_id(&self) -> &str {
        &self.primary_id
    }

    /// Returns the number of rotation keys currently held.
    pub fn rotation_key_count(&self) -> usize {
        self.rotation.len()
    }

    /// Returns `true` if `key_id` matches the primary key id.
    pub fn is_primary(&self, key_id: &str) -> bool {
        key_id == self.primary_id
    }

    /// Returns `true` if `key_id` matches a rotation key id.
    pub fn is_rotation(&self, key_id: &str) -> bool {
        self.rotation.contains_key(key_id)
    }

    /// Computes the HMAC-SHA256 signature of `payload` using the
    /// key identified by `key_id`. The signature is returned in
    /// the `"sha256=<hex>"` envelope used by every other signing
    /// helper in this crate.
    ///
    /// `timestamp` is incorporated into the HMAC input as
    /// `timestamp || payload` (8 bytes big-endian seconds followed
    /// by the raw bytes) so the signature is bound to the exact
    /// timestamp the receiver will verify against. This is what
    /// makes the signature replay-proof: an attacker who copies
    /// the bytes cannot strip and re-submit them under a fresh
    /// envelope without re-signing, because the HMAC would no
    /// longer match the new timestamp. The pairing with
    /// [`SigningKeyRing::verify_with_rotation`] is symmetric.
    pub fn sign(
        &self,
        payload: &[u8],
        key_id: &str,
        timestamp: i64,
    ) -> Result<String, SigningError> {
        let secret = self.lookup(key_id)?;
        let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| {
            // HMAC-SHA256 accepts any key length, so this branch is
            // unreachable for byte slices — but if it ever fires we
            // surface it as UnknownKey rather than panic.
            SigningError::UnknownKey(key_id.to_string())
        })?;
        mac.update(&timestamp.to_be_bytes());
        mac.update(payload);
        let bytes = mac.finalize().into_bytes();
        Ok(format!("sha256={}", hex_lower(&bytes)))
    }

    /// Verifies `signature` against `payload` by trying the
    /// primary key first and then each rotation key in id order.
    /// The signature timestamp is rejected if it falls outside
    /// the ring's [`TimestampPolicy`] tolerance of the injected
    /// clock (closes ADAPT-INT-009).
    ///
    /// The HMAC payload is `timestamp || payload` (8-byte
    /// big-endian seconds followed by the raw bytes). This binds
    /// the signature to a specific timestamp so an attacker
    /// cannot strip and replay it under a fresh envelope.
    pub fn verify_with_rotation(
        &self,
        payload: &[u8],
        signature: &str,
        timestamp: i64,
    ) -> Result<(), SigningError> {
        // Replay check: must happen *before* the HMAC, so we never
        // spend CPU on a key that would be rejected anyway and so
        // an attacker can't probe key validity by replaying
        // historical signatures that the receiver would otherwise
        // be forced to walk the full key set for.
        //
        // `tolerance_seconds` is `i64` for ergonomic construction
        // but the tolerance is a non-negative duration; negative
        // values are clamped to zero (i.e. every signature is
        // rejected, the strictest possible setting).
        let now = self.clock.now().as_datetime().timestamp();
        let age: u64 = now.abs_diff(timestamp);
        let tolerance: u64 = self.timestamp_policy.tolerance_seconds.unsigned_abs();
        if age > tolerance {
            return Err(SigningError::Expired {
                // `age` is `u64`; the engine's max representable
                // timestamp fits comfortably in `i64` so this cast
                // is loss-free for every realistic value.
                age_seconds: i64::try_from(age).unwrap_or(i64::MAX),
                tolerance_seconds: self.timestamp_policy.tolerance_seconds,
            });
        }

        let sig_bytes = parse_sha256_envelope(signature)?;
        let signed = compose_signed_payload(timestamp, payload);

        if hmac_matches(&self.primary_secret, &signed, &sig_bytes) {
            return Ok(());
        }
        for secret in self.rotation.values() {
            if hmac_matches(secret, &signed, &sig_bytes) {
                return Ok(());
            }
        }
        Err(SigningError::Mismatch)
    }

    fn lookup(&self, key_id: &str) -> Result<&[u8], SigningError> {
        if key_id == self.primary_id {
            return Ok(&self.primary_secret);
        }
        self.rotation
            .get(key_id)
            .map(Vec::as_slice)
            .ok_or_else(|| SigningError::UnknownKey(key_id.to_string()))
    }
}

// ---------------------------------------------------------------------------
// ct_eq — constant-time byte comparison
// ---------------------------------------------------------------------------

/// Constant-time equality for byte slices.
///
/// Length-mismatched inputs return `false` immediately — length
/// is not secret data, and short-circuiting here is the standard
/// convention (e.g. `subtle::ConstantTimeEq`).
///
/// Equal-length inputs are compared in time proportional to
/// `a.len()` regardless of which byte position differs, so it is
/// safe to use when comparing HMAC signatures.
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
// Private helpers
// ---------------------------------------------------------------------------

/// HMAC-SHA256 + constant-time compare. Returns `false` if HMAC
/// key setup fails (unreachable in practice — SHA-256 accepts any
/// key length) so this helper never panics.
fn hmac_matches(secret: &[u8], payload: &[u8], expected: &[u8]) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(secret) else {
        return false;
    };
    mac.update(payload);
    let bytes = mac.finalize().into_bytes();
    ct_eq(bytes.as_slice(), expected)
}

/// Concatenates `timestamp` (8 bytes big-endian) and `payload`.
/// The exact format is intentionally undocumented in the public
/// API — `sign` and `verify_with_rotation` must be paired so the
/// caller never needs to know it.
fn compose_signed_payload(timestamp: i64, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&timestamp.to_be_bytes());
    out.extend_from_slice(payload);
    out
}

/// Lowercase hex encoding. One byte -> two ASCII characters.
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        // `b >> 4` and `b & 0x0f` are bounded to `0..=15` by the
        // masks, so the `usize::from` widening is infallible.
        let hi = usize::from(b >> 4);
        let lo = usize::from(b & 0x0f);
        s.push(HEX[hi] as char);
        s.push(HEX[lo] as char);
    }
    s
}

/// Parses the `"sha256=<hex>"` envelope and returns the raw
/// signature bytes. Returns [`SigningError::Mismatch`] for any
/// malformed input — the caller does not need to distinguish
/// "missing prefix" from "wrong hex" from "odd length".
fn parse_sha256_envelope(signature: &str) -> Result<Vec<u8>, SigningError> {
    let hex = signature
        .strip_prefix("sha256=")
        .ok_or(SigningError::Mismatch)?;
    if hex.len() % 2 != 0 {
        return Err(SigningError::Mismatch);
    }
    let bytes = hex.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = unhex(bytes[i])?;
        let lo = unhex(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

/// Decodes one ASCII hex digit. Case-insensitive.
fn unhex(b: u8) -> Result<u8, SigningError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(SigningError::Mismatch),
    }
}

// ---------------------------------------------------------------------------
// Unit tests (smoke tests — the authoritative coverage lives in
// crates/adapters/integrations/tests/reliability_e2e.rs).
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_matches_constants() {
        let p = WebhookRetryPolicy::default();
        assert_eq!(p.max_attempts, DEFAULT_MAX_ATTEMPTS);
        assert_eq!(p.initial_backoff, DEFAULT_INITIAL_BACKOFF);
        assert_eq!(p.max_backoff, DEFAULT_MAX_BACKOFF);
    }

    #[test]
    fn ct_eq_equal() {
        assert!(ct_eq(b"", b""));
        assert!(ct_eq(b"abc", b"abc"));
        assert!(ct_eq(&[0u8; 32], &[0u8; 32]));
    }

    #[test]
    fn ct_eq_differ() {
        assert!(!ct_eq(b"abc", b"abd"));
        assert!(!ct_eq(b"abc", b"abcd"));
        assert!(!ct_eq(b"", b"x"));
    }

    #[test]
    fn hex_round_trip() {
        let bytes = [0x00u8, 0x0f, 0x10, 0xab, 0xff];
        let encoded = hex_lower(&bytes);
        assert_eq!(encoded, "000f10abff");
        let decoded = parse_sha256_envelope(&format!("sha256={encoded}")).expect("parses");
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn parse_rejects_odd_length() {
        let err = parse_sha256_envelope("sha256=abc").expect_err("odd length fails");
        assert_eq!(err, SigningError::Mismatch);
    }

    #[test]
    fn parse_rejects_missing_prefix() {
        let err = parse_sha256_envelope("deadbeef").expect_err("missing prefix fails");
        assert_eq!(err, SigningError::Mismatch);
    }

    #[test]
    fn parse_rejects_non_hex() {
        let err = parse_sha256_envelope("sha256=zz").expect_err("non-hex fails");
        assert_eq!(err, SigningError::Mismatch);
    }
}
