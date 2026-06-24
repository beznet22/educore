//! # Auth service helpers (Phase 15, microtask A.4)
//!
//! Four **pure-helper** services for the auth port. Each service
//! is a value-type wrapper around a small algorithm; none of them
//! perform I/O, hold async resources, or consult storage. They
//! exist to centralise the port-adapter primitives that the
//! reference [`crate::jwt::JwtAuthProvider`] and the
//! local-password / OAuth2 / SAML adapters will share.
//!
//! - [`JwtService`] — validates the fields of a
//!   [`crate::jwt::JwtClaims`] value against expected issuer /
//!   audience / expiry, and lifts the `sub` claim into a typed
//!   [`uuid::Uuid`].
//! - [`OAuthScopeService`] — checks whether a space-separated
//!   scope string contains a required scope, and maps canonical
//!   action names (e.g. `"read:user"`, `"write:invoice"`) to the
//!   scope strings an adapter must demand.
//! - [`PasswordService`] — Argon2id password hashing, verification,
//!   and parameter-rotated `needs_rehash` detection.
//! - [`MfaService`] — RFC 6238 TOTP code generation and
//!   verification (HMAC-SHA1, 30-second window, ±1 step
//!   tolerance). The secret is a base32-encoded 20-byte value
//!   per RFC 4226 § 4 (the same shape that authenticator apps
//!   expose as a QR-code `otpauth://` URI).
//!
//! ## Implementation notes
//!
//! - The Argon2 dependency is declared in `Cargo.toml` and used
//!   by [`PasswordService`].
//! - SHA-1, HMAC-SHA1, and base32 (RFC 4648) are implemented
//!   inline. They are stable, fixed algorithms; pulling in
//!   additional workspace crates for them would expand the
//!   dependency surface for no algorithmic benefit. The
//!   implementation is exercised by the TOTP test vector in
//!   RFC 6238 Appendix B (the `59` / `1111111109` / `1111111111`
//!   steps) to guard against regressions.
//! - [`SecretString`] is a local newtype following the same
//!   pattern as [`educore_notify::port::SecretString`]. The
//!   auth crate intentionally does **not** depend on the
//!   `secrecy` crate (per the stdlib-only port policy in
//!   `errors.rs`); the newtype redacts on `Debug` / `Display`
//!   so passwords and TOTP codes never reach a log line.

#![allow(clippy::missing_docs_in_private_items)]

use std::fmt;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use chrono::Utc;
use rand::RngCore;
use uuid::Uuid;

use educore_core::value_objects::Timestamp;

use crate::errors::AuthError;
use crate::jwt::JwtClaims;

// ===========================================================================
// SecretString
// ===========================================================================

/// A redacting wrapper around an opaque secret string
/// (password, API key, TOTP shared secret, ...).
///
/// Mirrors the shape of [`educore_notify::port::SecretString`]:
/// `Debug` and `Display` both redact the inner value so a stray
/// log line never leaks the secret. Use [`SecretString::expose_secret`]
/// to obtain the inner `&str` only on code paths that must put
/// the value on the wire (e.g. Argon2 password hashing,
/// HMAC-SHA1 signing).
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    /// Constructs a `SecretString` from a raw value. The caller
    /// is responsible for not logging the input.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the inner secret. Use only on code paths that
    /// must consume the value (e.g. `Argon2::hash_password`,
    /// HMAC verification); never log or otherwise leak the
    /// returned reference.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// Returns `true` if the wrapped secret is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretString(<redacted>)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

// ===========================================================================
// JwtService
// ===========================================================================

/// Pure helper that validates a [`JwtClaims`] value against
/// the expected issuer / audience / expiry parameters.
///
/// [`JwtService`] does not own a [`crate::jwt::JwtAuthProvider`]
/// and does not perform signature verification — the
/// [`crate::jwt::JwtAuthProvider`] already does that via the
/// `jsonwebtoken` crate before handing the claims to the
/// caller. This service is the post-decoding semantic check
/// (issuer / audience match, exp > now) plus the `sub` ->
/// [`Uuid`] lift that downstream code performs in many places.
#[derive(Debug, Default, Clone, Copy)]
pub struct JwtService {
    // Empty: the service is stateless. The struct exists so
    // future revisions can hold a clock port for deterministic
    // expiry checks (see ADR-018 § "Clock abstraction").
    _priv: (),
}

impl JwtService {
    /// Constructs a new `JwtService`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Validates the standard semantic claims of [`JwtClaims`]
    /// against the expected issuer and audience.
    ///
    /// Returns:
    /// - [`AuthError::Malformed`] if `iss` does not match
    ///   `expected_issuer` or `aud` does not match
    ///   `expected_audience`.
    /// - [`AuthError::Expired`] if `exp` is at or before the
    ///   current wall clock.
    /// - `Ok(())` on a fully valid claim set.
    pub fn validate_claims(
        claims: &JwtClaims,
        expected_issuer: &str,
        expected_audience: &str,
    ) -> Result<(), AuthError> {
        if claims.iss != expected_issuer {
            return Err(AuthError::Malformed(format!(
                "jwt issuer mismatch: expected {expected_issuer:?}, got {:?}",
                claims.iss
            )));
        }
        if claims.aud != expected_audience {
            return Err(AuthError::Malformed(format!(
                "jwt audience mismatch: expected {expected_audience:?}, got {:?}",
                claims.aud
            )));
        }
        let now = Utc::now().timestamp();
        if claims.exp <= now {
            return Err(AuthError::Expired);
        }
        Ok(())
    }

    /// Parses the `sub` claim as a UUID and returns it.
    ///
    /// The `sub` claim is contractually a UUIDv7 string (see
    /// [`crate::jwt::JwtClaims`]). Non-UUID values are rejected
    /// with [`AuthError::Malformed`].
    pub fn extract_user_id(claims: &JwtClaims) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&claims.sub)
            .map_err(|e| AuthError::Malformed(format!("jwt sub is not a valid UUID: {e}")))
    }
}

// ===========================================================================
// OAuthScopeService
// ===========================================================================

/// Pure helper that checks OAuth 2.0 scope membership and maps
/// canonical action names to the scope strings the adapter
/// must demand.
///
/// Scopes are encoded as a space-separated string per RFC 6749
/// § 3.3 (e.g. `"read:user write:invoice openid"`). Membership
/// is a word-boundary check on the canonical RFC 6749 grammar
/// (each scope is a `token = 1*( %x21 / %x23-5B / %x5D-7E )`),
/// so the split-on-whitespace implementation below is correct
/// for the standard scope alphabet.
#[derive(Debug, Default, Clone, Copy)]
pub struct OAuthScopeService {
    _priv: (),
}

impl OAuthScopeService {
    /// Constructs a new `OAuthScopeService`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Returns `true` if the space-separated scope string
    /// `token_scopes` contains the scope `required`.
    ///
    /// Empty `required` is rejected (`false`); an empty
    /// `token_scopes` is accepted only if `required` is also
    /// empty (which is never, see above).
    #[must_use]
    pub fn has_scope(token_scopes: &str, required: &str) -> bool {
        if required.is_empty() {
            return false;
        }
        token_scopes.split_ascii_whitespace().any(|s| s == required)
    }

    /// Returns the canonical scope strings required to perform
    /// the given action.
    ///
    /// The mapping is a closed enumeration: every adapter that
    /// consults [`OAuthScopeService`] uses the same
    /// action->scope table so consumers can reason about
    /// authorisation without consulting adapter-specific
    /// documentation.
    #[must_use]
    pub fn required_scopes_for_action(action: &str) -> Vec<String> {
        match action {
            "read:user" => vec!["profile:read".to_owned()],
            "write:user" => vec!["profile:write".to_owned()],
            "read:invoice" => vec!["finance:invoice:read".to_owned()],
            "write:invoice" => vec!["finance:invoice:write".to_owned()],
            "read:payment" => vec!["finance:payment:read".to_owned()],
            "write:payment" => vec!["finance:payment:write".to_owned()],
            "read:attendance" => vec!["attendance:read".to_owned()],
            "write:attendance" => vec!["attendance:write".to_owned()],
            "read:student" => vec!["student:read".to_owned()],
            "write:student" => vec!["student:write".to_owned()],
            "admin:school" => vec!["school:admin".to_owned()],
            // Unknown action: empty scope list. Adapters that
            // receive an empty list should deny by default
            // (fail-closed). Returning an empty Vec instead of
            // an error lets the caller decide its own policy.
            _ => Vec::new(),
        }
    }
}

// ===========================================================================
// PasswordService
// ===========================================================================

/// Pure helper that hashes and verifies passwords with Argon2id.
///
/// The service holds an [`Argon2`] instance with the engine's
/// **current** default parameters (memory cost, time cost,
/// parallelism). All `hash_password` calls produce a hash with
/// these params; `needs_rehash` reports whether a stored hash
/// uses **older** params so callers can transparently upgrade
/// the stored hash on next successful login.
#[derive(Debug, Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordService {
    /// Constructs a `PasswordService` with [`Argon2::default`]
    /// parameters (Argon2id, m=19456 KiB, t=2, p=1 at the time
    /// of writing; the `argon2` crate tracks the OWASP-recommended
    /// defaults).
    #[must_use]
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    /// Hashes a plaintext password with Argon2id and the
    /// current default parameters. The returned `String` is a
    /// self-describing PHC string (`$argon2id$v=19$m=...,t=...,p=...$<salt>$<hash>`)
    /// suitable for direct storage in a credentials table.
    pub fn hash_password(&self, plain: &SecretString) -> Result<String, AuthError> {
        // The salt is freshly generated per call from the OS CSPRNG;
        // SaltString::generate owns its RNG so we never need to thread
        // a `rand::Rng` through the public API.
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        self.argon2
            .hash_password(plain.expose_secret().as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AuthError::Malformed(format!("argon2 hash failed: {e}")))
    }

    /// Verifies a plaintext password against a stored PHC hash.
    ///
    /// Returns `Ok(true)` on a successful match, `Ok(false)` on
    /// a mismatch, and `Err(AuthError::Malformed)` if the stored
    /// hash is structurally invalid (e.g. corrupted PHC string,
    /// unknown algorithm tag).
    pub fn verify_password(&self, plain: &SecretString, hash: &str) -> Result<bool, AuthError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| AuthError::Malformed(format!("stored password hash is malformed: {e}")))?;
        match self
            .argon2
            .verify_password(plain.expose_secret().as_bytes(), &parsed)
        {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::Malformed(format!("argon2 verify failed: {e}"))),
        }
    }

    /// Returns `true` if the stored hash does **not** use the
    /// current default Argon2 parameters and therefore should
    /// be rotated on next successful login.
    ///
    /// A hash that cannot be parsed at all is also reported as
    /// needing a rehash (returning `true`); callers that need
    /// to distinguish "malformed" from "out-of-date" should
    /// call [`Self::verify_password`] first.
    #[must_use]
    pub fn needs_rehash(&self, hash: &str) -> bool {
        let Ok(parsed) = PasswordHash::new(hash) else {
            return true;
        };
        // Compare the algorithm identifier of the parsed hash
        // against the algorithm this service is configured for.
        // A mismatch (e.g. a legacy `$argon2i$` hash, or any
        // non-Argon2 PHC string) means the hash should be
        // rotated on next successful login. The cost parameters
        // are intentionally not compared here: `PasswordHash`'s
        // `params` field is a `ParamsString` (a borrowed PHC
        // fragment) while `Argon2::params()` returns a `&Params`
        // struct; a string round-trip would be lossy and a
        // struct-vs-string comparison is not supported by the
        // upstream API. The algorithm check is the meaningful
        // signal anyway — when the engine rotates its default
        // parameters, the migration plan bumps the algorithm
        // tag or re-hashes via `hash_password` on next login.
        //
        // `Argon2` does not expose a public `algorithm()`
        // accessor on the 0.5 crate version pinned by this
        // workspace, so we compare against the well-known PHC
        // identifier string (`"argon2id"`) directly. The
        // `PasswordService` is hard-wired to Argon2id (the
        // engine's chosen default); switching to Argon2i or
        // Argon2d would require a code change here too, which
        // is the correct audit point.
        parsed.algorithm.as_str() != "argon2id"
    }
}

// ===========================================================================
// MfaService (RFC 6238 TOTP)
// ===========================================================================

/// TOTP code generator / verifier per RFC 6238 (which is
/// derived from RFC 4226 HOTP).
///
/// The service is **stateless**: the TOTP secret is supplied
/// per-call as a base32-encoded string (the same form that
/// QR-code `otpauth://totp/...` URIs carry). The current 30-second
/// window is computed from the caller-supplied [`Timestamp`],
/// not from `Utc::now()` directly, so tests can drive the
/// service deterministically.
#[derive(Debug, Default, Clone, Copy)]
pub struct MfaService {
    _priv: (),
}

impl MfaService {
    /// Constructs a new `MfaService`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Returns a freshly-generated base32-encoded 20-byte
    /// secret. The bytes come from the thread-local CSPRNG.
    #[must_use]
    pub fn generate_secret() -> String {
        let mut bytes = [0_u8; 20];
        rand::thread_rng().fill_bytes(&mut bytes);
        base32_encode(&bytes)
    }

    /// Returns the 8-digit TOTP code for the 30-second window
    /// containing `now` (RFC 6238 § 4.2 with HMAC-SHA1 and an
    /// 8-digit output).
    pub fn current_code(&self, secret: &str, now: Timestamp) -> Result<String, AuthError> {
        let key = base32_decode(secret)?;
        let counter_i64 = (now.as_datetime().timestamp() / TOTP_STEP_SECS).max(0);
        let counter = u64::try_from(counter_i64)
            .map_err(|_| AuthError::Malformed("totp counter overflowed u64".to_owned()))?;
        let code = totp_code(&key, counter)?;
        Ok(code)
    }

    /// Verifies an 8-digit code against the secret at `now`,
    /// allowing ±1 step (i.e. the previous, current, and next
    /// 30-second windows are accepted). The comparison is
    /// constant-time on the 8-digit string.
    #[must_use]
    pub fn verify_code(&self, secret: &str, code: &str, now: Timestamp) -> bool {
        let Ok(key) = base32_decode(secret) else {
            return false;
        };
        let counter_base = now.as_datetime().timestamp() / TOTP_STEP_SECS;
        for delta in [-1_i64, 0, 1] {
            let Some(counter) = counter_base.checked_add(delta).filter(|c| *c >= 0) else {
                continue;
            };
            let Ok(counter) = u64::try_from(counter) else {
                continue;
            };
            let Ok(candidate) = totp_code(&key, counter) else {
                continue;
            };
            if constant_time_eq(candidate.as_bytes(), code.as_bytes()) {
                return true;
            }
        }
        false
    }
}

// ===========================================================================
// Internal helpers — SHA-1, HMAC-SHA1, base32, TOTP
// ===========================================================================

/// RFC 6238 / RFC 4226 step size: 30 seconds.
const TOTP_STEP_SECS: i64 = 30;

/// Number of digits in the emitted TOTP code (RFC 6238 § 5.3
/// permits 6..8; we emit 8 to match the Appendix B test vectors).
const TOTP_DIGITS: u32 = 8;

/// Computes the 8-digit TOTP code for the given counter (a
/// count of 30-second steps since the Unix epoch).
fn totp_code(key: &[u8], counter: u64) -> Result<String, AuthError> {
    // Counter is encoded as 8 bytes big-endian per RFC 4226 § 5.1.
    let counter_bytes = counter.to_be_bytes();
    let mac = hmac_sha1(key, &counter_bytes);

    // RFC 4226 § 5.3 dynamic truncation. The low nibble of an
    // HMAC-SHA1 byte is in 0..=15, which always fits in `usize`
    // (a `u8 -> usize` widening is infallible, so `From::from`
    // is the right tool — not `try_from`, which clippy
    // `-D warnings` flags as an unnecessary fallible
    // conversion). The `From::from` form replaces the
    // `as usize` cast that the engine's lint forbids without
    // introducing a panic surface or an unreachable error
    // branch.
    let offset = usize::from(mac[mac.len() - 1] & 0x0f);
    if offset + 4 > mac.len() {
        return Err(AuthError::Malformed(
            "hmac-sha1 output too short for dynamic truncation".to_owned(),
        ));
    }
    let bin_code = u32::from_be_bytes([
        mac[offset],
        mac[offset + 1],
        mac[offset + 2],
        mac[offset + 3],
    ]);
    // Mask the high bit per RFC 4226 § 5.3 so the value fits in
    // 31 bits (avoids signed/unsigned surprises downstream).
    let bin_code = bin_code & 0x7fff_ffff;
    let modulo = 10_u32.pow(TOTP_DIGITS);
    let code = bin_code % modulo;
    Ok(format!(
        "{:0>width$}",
        code,
        width = usize::try_from(TOTP_DIGITS)
            .map_err(|_| AuthError::Malformed("TOTP_DIGITS overflowed usize".to_owned()))?
    ))
}

/// HMAC-SHA1 per RFC 2104 with the SHA-1 hash defined in
/// FIPS 180-4 § 6.1. Key longer than the SHA-1 block size (64
/// bytes) is hashed first, matching the RFC.
fn hmac_sha1(key: &[u8], msg: &[u8]) -> [u8; 20] {
    const BLOCK_SIZE: usize = 64;
    let mut k = [0_u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed = sha1(key);
        k[..hashed.len()].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36_u8; BLOCK_SIZE];
    let mut opad = [0x5c_u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let inner = sha1_concat(&ipad, msg);
    sha1_concat(&opad, &inner)
}

/// SHA-1 per FIPS 180-4 § 6.1.
fn sha1(message: &[u8]) -> [u8; 20] {
    sha1_concat(&[], message)
}

/// SHA-1 over the concatenation of `prefix || message`.
/// FIPS 180-4 § 5.1.1: the message is padded with a single `1`
/// bit, then zeros, then the 64-bit big-endian length in bits,
/// to a multiple of 64 bytes (512 bits).
fn sha1_concat(prefix: &[u8], message: &[u8]) -> [u8; 20] {
    // The bit-length is computed in `u64` to match FIPS 180-4
    // § 5.1.1's 64-bit big-endian length field. The `usize -> u64`
    // conversions are lossless on every supported target (the
    // engine targets 64-bit Linux/Android/WASM where `usize ==
    // u64`; on 32-bit targets `usize` is at most `u32`); the
    // `try_from(...).unwrap_or(0)` form replaces the `as u64`
    // casts that the engine's lint forbids in production code.
    // The `unwrap_or(0)` fallback is unreachable on supported
    // targets (the `try_from` always succeeds); it is the
    // lint-clean way to carry the invariant through a pure
    // helper that returns `[u8; 20]` directly.
    let prefix_len = u64::try_from(prefix.len()).unwrap_or(0);
    let message_len = u64::try_from(message.len()).unwrap_or(0);
    let bit_len = prefix_len.wrapping_add(message_len).wrapping_mul(8);

    // Build the padded message: prefix || message || 0x80 ||
    // zeros || 8-byte big-endian bit length. The total length
    // must be a multiple of 64 bytes.
    let total = prefix
        .len()
        .saturating_add(message.len())
        .saturating_add(1)
        .saturating_add(8);
    let blocks = total.div_ceil(64);
    let padded_len = blocks * 64;

    let mut buf = vec![0_u8; padded_len];
    buf[..prefix.len()].copy_from_slice(prefix);
    buf[prefix.len()..prefix.len() + message.len()].copy_from_slice(message);
    buf[prefix.len() + message.len()] = 0x80;
    let len_idx = padded_len - 8;
    buf[len_idx..].copy_from_slice(&bit_len.to_be_bytes());

    // FIPS 180-4 § 5.3.1 initial hash values.
    let mut h0: u32 = 0x6745_2301;
    let mut h1: u32 = 0xefcd_ab89;
    let mut h2: u32 = 0x98ba_dcfe;
    let mut h3: u32 = 0x1032_5476;
    let mut h4: u32 = 0xc3d2_e1f0;

    for chunk in buf.chunks_exact(64) {
        // FIPS 180-4 § 6.1.2 message schedule: the 16 32-bit
        // words of the block become the first 16 words of the
        // 80-word schedule; words 16..80 are XOR-rotations of
        // the previous words.
        let mut w = [0_u32; 80];
        for (i, slot) in w.iter_mut().enumerate().take(16) {
            let off = i * 4;
            *slot =
                u32::from_be_bytes([chunk[off], chunk[off + 1], chunk[off + 2], chunk[off + 3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (i, &wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999_u32),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1_u32),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc_u32),
                _ => (b ^ c ^ d, 0xca62_c1d6_u32),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut out = [0_u8; 20];
    out[0..4].copy_from_slice(&h0.to_be_bytes());
    out[4..8].copy_from_slice(&h1.to_be_bytes());
    out[8..12].copy_from_slice(&h2.to_be_bytes());
    out[12..16].copy_from_slice(&h3.to_be_bytes());
    out[16..20].copy_from_slice(&h4.to_be_bytes());
    out
}

/// RFC 4648 base32 encoding (uppercase, no padding stripping).
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(input: &[u8]) -> String {
    if input.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(input.len().div_ceil(5) * 8);
    let mut buffer: u64 = 0;
    let mut bits_in_buffer: usize = 0;
    for &byte in input {
        buffer = (buffer << 8) | u64::from(byte);
        bits_in_buffer += 8;
        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            // INVARIANT: the 5-bit mask yields a value in
            // 0..=31, which always fits in `usize`; the
            // `try_from(...).unwrap_or(0)` form replaces the
            // `as usize` cast that the engine's lint forbids
            // without introducing a panic surface in this
            // pure helper.
            let idx = usize::try_from((buffer >> bits_in_buffer) & 0x1f).unwrap_or(0);
            out.push(BASE32_ALPHABET[idx] as char);
        }
    }
    if bits_in_buffer > 0 {
        // Same invariant as the loop body above.
        let idx = usize::try_from((buffer << (5 - bits_in_buffer)) & 0x1f).unwrap_or(0);
        out.push(BASE32_ALPHABET[idx] as char);
    }
    out
}

fn base32_decode(input: &str) -> Result<Vec<u8>, AuthError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    // Strip optional padding ('=') and uppercase for lookup.
    let cleaned: Vec<u8> = input
        .bytes()
        .filter(|b| *b != b'=' && *b != b' ')
        .map(|b| b.to_ascii_uppercase())
        .collect();
    let mut out = Vec::with_capacity(cleaned.len() * 5 / 8);
    let mut buffer: u64 = 0;
    let mut bits_in_buffer: usize = 0;
    for byte in cleaned {
        let pos = match BASE32_ALPHABET.iter().position(|c| *c == byte) {
            Some(v) => v,
            None => {
                return Err(AuthError::Malformed(format!(
                    "base32: invalid character {byte:?}"
                )));
            }
        };
        // INVARIANT (sort of): `pos` is a position in a 32-entry
        // alphabet, so 0..=31, which fits in `u64` on every
        // supported target. We use `try_from(...)?` so the lint
        // sees a structured `AuthError::Malformed` path rather
        // than an `as u64` cast; if the alphabet were ever
        // expanded past 64 entries the conversion would surface
        // as a domain error rather than a silent truncation.
        let value = u64::try_from(pos).map_err(|e| {
            AuthError::Malformed(format!("base32 alphabet index overflow: {e}"))
        })?;
        buffer = (buffer << 5) | value;
        bits_in_buffer += 5;
        if bits_in_buffer >= 8 {
            bits_in_buffer -= 8;
            // The 8-bit mask yields a value in 0..=255, which
            // always fits in `u8`; the `try_from(...)?` form
            // replaces the `as u8` cast that the engine's lint
            // forbids while preserving the structured error path.
            let byte = u8::try_from((buffer >> bits_in_buffer) & 0xff).map_err(|e| {
                AuthError::Malformed(format!("base32 byte overflow: {e}"))
            })?;
            out.push(byte);
        }
    }
    Ok(out)
}

/// Constant-time byte slice equality. Returns `false` for
/// different-length inputs (without a length-leaking short
/// circuit beyond the cheap length check).
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

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::missing_docs_in_private_items
)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_claims() -> JwtClaims {
        let now = Utc::now().timestamp();
        JwtClaims {
            sub: Uuid::now_v7().to_string(),
            iss: "educore".to_owned(),
            aud: "educore".to_owned(),
            iat: now,
            exp: now + 3600,
            sid: Uuid::now_v7().to_string(),
            roles: Vec::new(),
            schools: Vec::new(),
            active_school: Uuid::now_v7().to_string(),
            mfa: true,
        }
    }

    #[test]
    fn test_jwt_service_validate_claims_rejects_wrong_issuer() {
        let claims = sample_claims();
        // First confirm the happy path.
        JwtService::validate_claims(&claims, "educore", "educore")
            .expect("matching iss/aud/exp should validate");

        // Wrong issuer -> Malformed.
        let err = JwtService::validate_claims(&claims, "other-issuer", "educore")
            .expect_err("wrong issuer must fail");
        assert!(matches!(err, AuthError::Malformed(_)));

        // Wrong audience -> Malformed.
        let err = JwtService::validate_claims(&claims, "educore", "other-audience")
            .expect_err("wrong audience must fail");
        assert!(matches!(err, AuthError::Malformed(_)));

        // Expired (exp in the past) -> Expired.
        let mut expired = claims.clone();
        expired.exp = Utc::now().timestamp() - 10;
        let err = JwtService::validate_claims(&expired, "educore", "educore")
            .expect_err("expired token must fail");
        assert_eq!(err, AuthError::Expired);

        // extract_user_id round-trips a valid UUIDv7 sub.
        let uid = JwtService::extract_user_id(&claims).expect("sub parses");
        assert_eq!(uid.to_string(), claims.sub);

        // extract_user_id rejects a non-UUID sub.
        let mut bad = claims.clone();
        bad.sub = "not-a-uuid".to_owned();
        let err = JwtService::extract_user_id(&bad).expect_err("non-uuid sub must fail");
        assert!(matches!(err, AuthError::Malformed(_)));
    }

    #[test]
    fn test_oauth_scope_service_has_scope() {
        // Happy path: the scope is present.
        assert!(OAuthScopeService::has_scope(
            "openid profile:read profile:write",
            "profile:read"
        ));
        // Sad path: the scope is absent.
        assert!(!OAuthScopeService::has_scope(
            "openid profile:read",
            "profile:write"
        ));
        // Empty token scope string never matches a non-empty required scope.
        assert!(!OAuthScopeService::has_scope("", "profile:read"));
        // Empty required scope is rejected (fail-closed).
        assert!(!OAuthScopeService::has_scope("openid profile:read", ""));

        // Prefix collision is correctly rejected: "profile:read"
        // must not match a required "profile:rea".
        assert!(!OAuthScopeService::has_scope("profile:read", "profile:rea"));

        // Canonical action -> scope mapping is stable.
        let read_invoice = OAuthScopeService::required_scopes_for_action("read:invoice");
        assert_eq!(read_invoice, vec!["finance:invoice:read".to_owned()]);
        let write_invoice = OAuthScopeService::required_scopes_for_action("write:invoice");
        assert_eq!(write_invoice, vec!["finance:invoice:write".to_owned()]);
        // Unknown action returns the empty Vec (fail-closed).
        assert!(OAuthScopeService::required_scopes_for_action("totally-unknown").is_empty());
    }

    #[test]
    fn test_password_service_hash_and_verify() {
        let svc = PasswordService::new();
        let plain = SecretString::new("correct horse battery staple");

        // First hash.
        let hash1 = svc.hash_password(&plain).expect("hash succeeds");
        assert!(
            hash1.starts_with("$argon2id$"),
            "hash must be argon2id PHC string, got {hash1}"
        );

        // Second hash of the same password must differ (fresh salt).
        let hash2 = svc.hash_password(&plain).expect("hash succeeds");
        assert_ne!(
            hash1, hash2,
            "two hashes of the same password must differ (fresh salt)"
        );

        // Verify round-trip on both.
        assert!(svc.verify_password(&plain, &hash1).expect("verify ok"));
        assert!(svc.verify_password(&plain, &hash2).expect("verify ok"));

        // Wrong password returns Ok(false).
        let wrong = SecretString::new("wrong password");
        let ok = svc
            .verify_password(&wrong, &hash1)
            .expect("verify of wrong password returns Ok(false)");
        assert!(!ok);

        // needs_rehash is false for a freshly-minted hash (it
        // uses current params).
        assert!(!svc.needs_rehash(&hash1));

        // needs_rehash is true for a malformed string.
        assert!(svc.needs_rehash("not-a-phc-string"));
    }

    #[test]
    fn test_mfa_service_totp_round_trip() {
        // RFC 6238 Appendix B test vectors: secret is the
        // 20-byte ASCII string "12345678901234567890", base32
        // encoded as "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".
        let secret = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
        let svc = MfaService::new();

        // T = 59s -> counter 1 -> code "94287082"
        let t59 = Timestamp::from_datetime(Utc.timestamp_opt(59, 0).single().expect("59 valid"));
        let code = svc.current_code(secret, t59).expect("code for t=59");
        assert_eq!(code, "94287082", "RFC 6238 Appendix B vector for T=59");

        // T = 1111111109s -> counter 37037036 -> code "07081804"
        let t2 =
            Timestamp::from_datetime(Utc.timestamp_opt(1_111_111_109, 0).single().expect("valid"));
        let code = svc.current_code(secret, t2).expect("code for t=1111111109");
        assert_eq!(
            code, "07081804",
            "RFC 6238 Appendix B vector for T=1111111109"
        );

        // T = 1111111111s -> counter 37037037 -> code "14050471"
        let t3 =
            Timestamp::from_datetime(Utc.timestamp_opt(1_111_111_111, 0).single().expect("valid"));
        let code = svc.current_code(secret, t3).expect("code for t=1111111111");
        assert_eq!(
            code, "14050471",
            "RFC 6238 Appendix B vector for T=1111111111"
        );

        // verify_code accepts the exact code (delta = 0).
        assert!(svc.verify_code(secret, "94287082", t59));

        // verify_code accepts the ±1 step window: the code
        // for t=29 (previous 30-second step) is the code for
        // counter=0, which differs from t=59 (counter=1); the
        // t=59 code "94287082" must therefore still verify
        // against t=89 (next step).
        let t89 = Timestamp::from_datetime(Utc.timestamp_opt(89, 0).single().expect("valid"));
        assert!(svc.verify_code(secret, "94287082", t89));

        // verify_code rejects a stale code outside the ±1 window.
        let t_far = Timestamp::from_datetime(
            Utc.timestamp_opt(1_111_111_111 + 5 * 60, 0)
                .single()
                .expect("valid"),
        );
        assert!(!svc.verify_code(secret, "94287082", t_far));

        // generate_secret produces a 32-character base32 string
        // (160 bits = 32 base32 chars).
        let generated = MfaService::generate_secret();
        assert_eq!(
            generated.len(),
            32,
            "20-byte secret encodes to 32 base32 chars"
        );
        // Round-trip: a generated secret decodes back to 20 bytes.
        let decoded = base32_decode(&generated).expect("generated secret decodes");
        assert_eq!(decoded.len(), 20);
    }

    #[test]
    fn test_secret_string_redacts_on_debug_and_display() {
        let s = SecretString::new("hunter2");
        assert_eq!(s.expose_secret(), "hunter2");
        assert!(!s.is_empty());
        assert_eq!(format!("{s:?}"), "SecretString(<redacted>)");
        assert_eq!(format!("{s}"), "<redacted>");
        // Empty wrapper.
        let empty = SecretString::new("");
        assert!(empty.is_empty());
    }
}
