//! # Service helpers
//!
//! Pure helper structs that sit alongside the [`FileStorage`](crate::port::FileStorage)
//! port and its adapters. None of them perform I/O; they exist so
//! the adapter crates (S3, local, GCS, …) can keep their hot paths
//! tight and so the engine's checksum, signed-URL, namespacing, and
//! visibility logic can be unit-tested without spinning up an
//! object store or a network.
//!
//! The four services are:
//!
//! - [`ChecksumService`] — computes SHA-256 content hashes and
//!   S3-style quoted ETags, and verifies checksums in constant time.
//! - [`SignedUrlService`] — mints and verifies HMAC-SHA256 URL
//!   signatures (and assembles a `base/key?expires=…&signature=…`
//!   URL on demand).
//! - [`KeyNamespaceService`] — composes and parses the
//!   `<school_id>/<domain>/<aggregate>/<id>/<filename>` key form
//!   locked at `docs/ports/file-storage.md` § "Key Namespacing"
//!   (lines 89–95).
//! - [`VisibilityService`] — classifies a [`Visibility`] value
//!   (`Private` / `Public` / `TenantPrivate`) and answers the
//!   "can this user read this file?" question for the engine's
//!   read-side authorisation middleware.
//!
//! # Deviations from the spec
//!
//! - The signing key inside [`SignedUrlService`] is held as a
//!   `String`, not a `secrecy::SecretString`. The `secrecy` crate
//!   is not in `educore-files`' dependency set (the port
//!   explicitly opts out in [`port`](crate::port) § "Deviations
//!   from `docs/ports/file-storage.md`"). The `Debug` impl on
//!   [`SignedUrlService`] redacts the secret so the helper is
//!   still safe to print.
//! - The HMAC `new_from_slice` call returns a
//!   `Result<HMAC, InvalidLength>`. The `Result` is forwarded
//!   rather than `expect`-unwrapped so the engine's "no
//!   `expect`/`unwrap` in production paths" rule is preserved
//!   (see `docs/code-standards.md` § "Type Safety").
//! - [`KeyNamespaceService::is_in_tenant`] is a lexical check on
//!   the first path segment. It does not consult a tenant table;
//!   a malicious consumer that uploads a key with a fabricated
//!   prefix can bypass it. The engine relies on the `put` path
//!   (which the adapter validates) for the authoritative
//!   tenant check.

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

use educore_core::value_objects::Timestamp;

use crate::errors::FileStorageError;
use crate::port::Visibility;

/// The HMAC-SHA256 typed alias used throughout this module.
type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// 1. ChecksumService
// ---------------------------------------------------------------------------

/// Computes SHA-256 content hashes, verifies them in constant
/// time, and emits S3-style quoted ETags.
///
/// The service holds no state; every method is a pure function
/// over the inputs. The struct exists so adapter hot paths can
/// write `ChecksumService::compute_sha256(&bytes)` and read
/// identically to the other services in this module.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChecksumService;

impl ChecksumService {
    /// Constructs a new `ChecksumService`. The struct is a
    /// zero-sized marker; the constructor is provided for API
    /// symmetry with the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns the lowercase hex SHA-256 digest of `content`.
    ///
    /// Per `docs/ports/file-storage.md` § "Content-Addressable
    /// Hashing": the engine verifies the checksum on read; the
    /// wire format is the lowercase hex representation of the
    /// digest (64 chars, 32 bytes).
    #[must_use]
    pub fn compute_sha256(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex_encode(&hasher.finalize())
    }

    /// Returns `true` iff the recomputed SHA-256 of `content`
    /// equals `expected_hex`. The comparison is constant-time
    /// over the length of the expected string so a timing-side-
    /// channel attacker cannot recover the digest byte-by-byte.
    #[must_use]
    pub fn verify(content: &[u8], expected_hex: &str) -> bool {
        let computed = Self::compute_sha256(content);
        constant_time_eq_str(&computed, expected_hex)
    }

    /// Returns the S3-style quoted entity tag for `content`:
    /// the lowercase hex SHA-256 digest wrapped in double
    /// quotes (e.g. `"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`).
    ///
    /// This is the form S3 returns in `ETag` response headers
    /// and the form [`FileReference::etag`](crate::port::FileReference::etag)
    /// stores. Multipart-upload ETags are different (they drop
    /// the trailing `-N`); adapters that handle multipart
    /// uploads should not use this method for those parts.
    #[must_use]
    pub fn compute_etag(content: &[u8]) -> String {
        let hex = Self::compute_sha256(content);
        let mut out = String::with_capacity(hex.len() + 2);
        out.push('"');
        out.push_str(&hex);
        out.push('"');
        out
    }
}

// ---------------------------------------------------------------------------
// 2. SignedUrlService
// ---------------------------------------------------------------------------

/// Computes and verifies HMAC-SHA256 signed-URL tokens against
/// a per-adapter signing key.
///
/// The wire format mirrors the `LocalFileStorage` adapter
/// (`docs/ports/file-storage.md` § "Signed URLs"): the lowercase
/// hex of `HMAC-SHA256(signing_key, "<key>:<expires_at>")`.
/// The `Debug` impl redacts the key so the helper is safe to
/// print in logs.
pub struct SignedUrlService {
    signing_key: String,
}

impl SignedUrlService {
    /// Constructs a new `SignedUrlService` from a raw signing-key
    /// string. The string is moved into the service and redacted
    /// on `Debug`.
    #[must_use]
    pub fn new(signing_key: impl Into<String>) -> Self {
        Self {
            signing_key: signing_key.into(),
        }
    }

    /// Returns the lowercase hex HMAC-SHA256 signature of
    /// `"<key>:<expires_at>"` using the configured signing key.
    ///
    /// The canonical message format `format!("{key}:{expires_at}")`
    /// matches the local adapter's `"<key>|<expires_in>"` pattern
    /// but uses `:` instead of `|` so signed URLs can be embedded
    /// inside the query string without ambiguity. The
    /// `expires_at` is the RFC 3339 representation of the
    /// [`Timestamp`] — the absolute expiry instant, not a
    /// relative duration.
    ///
    /// The `Result` is needed because `Hmac::new_from_slice`
    /// returns a `Result` (and the engine forbids `expect` in
    /// production paths). The `Err` arm is theoretically
    /// unreachable for HMAC-SHA256 (which accepts any key
    /// length); it surfaces as `FileStorageError::Infrastructure`
    /// if the impl ever changes.
    pub fn sign(&self, key: &str, expires_at: Timestamp) -> Result<String, FileStorageError> {
        let message = format!("{key}:{}", expires_at.to_rfc3339());
        let mut mac = HmacSha256::new_from_slice(self.signing_key.as_bytes()).map_err(|e| {
            FileStorageError::InvalidKey(format!(
                "hmac key rejected (unreachable for HMAC-SHA256): {e}"
            ))
        })?;
        mac.update(message.as_bytes());
        Ok(hex_encode(&mac.finalize().into_bytes()))
    }

    /// Returns `true` iff the provided hex signature matches the
    /// recomputed signature for `(key, expires_at)` AND the
    /// expiry is still in the future. The HMAC comparison is
    /// constant-time over the length of the recomputed digest.
    #[must_use]
    pub fn verify(&self, key: &str, expires_at: Timestamp, provided: &str) -> bool {
        let Ok(expected) = self.sign(key, expires_at) else {
            return false;
        };
        if !constant_time_eq_str(&expected, provided) {
            return false;
        }
        Timestamp::now() < expires_at
    }

    /// Assembles a signed URL of the form
    /// `<base_url>/<key>?expires=<rfc3339>&signature=<hex>`.
    ///
    /// The `expires_in` is added to the current wall clock to
    /// produce the absolute expiry; the same instant is fed
    /// into the HMAC so the URL round-trips through `verify`.
    #[must_use]
    pub fn build_signed_url(&self, base_url: &str, key: &str, expires_in: Duration) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let expires_secs = now.saturating_add(expires_in.as_secs());
        let expires_rfc3339 = unix_secs_to_rfc3339(expires_secs);
        let expires_at =
            Timestamp::parse_rfc3339(&expires_rfc3339).unwrap_or_else(|_| Timestamp::now());
        let signature = self.sign(key, expires_at).unwrap_or_else(|_| String::new());
        format!("{base_url}/{key}?expires={expires_rfc3339}&signature={signature}")
    }
}

impl fmt::Debug for SignedUrlService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SignedUrlService")
            .field("signing_key", &"<redacted>")
            .finish()
    }
}

// ---------------------------------------------------------------------------
// 3. KeyNamespaceService
// ---------------------------------------------------------------------------

/// Composes and parses the tenant-scoped key form locked at
/// `docs/ports/file-storage.md` § "Key Namespacing":
///
/// ```text
/// <school_id>/<domain>/<aggregate>/<id>/<filename>
/// ```
///
/// The service holds no state; every method is a pure function
/// over the inputs. The struct exists so adapter hot paths can
/// write `KeyNamespaceService::namespace_key(…)` and read
/// identically to the other services in this module.
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyNamespaceService;

impl KeyNamespaceService {
    /// Constructs a new `KeyNamespaceService`. The struct is a
    /// zero-sized marker; the constructor is provided for API
    /// symmetry with the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Composes a tenant-scoped key from the five components.
    /// The output is `format!("{school_id}/{domain}/{aggregate}/{id}/{filename}")`.
    ///
    /// The five components are joined with `/`; no normalisation
    /// or escaping is applied. Adapters that reject keys
    /// containing `..` or leading `/` perform that check inside
    /// their own `put` implementation (e.g. the local adapter's
    /// `resolve`).
    #[must_use]
    pub fn namespace_key(
        school_id: &str,
        domain: &str,
        aggregate: &str,
        id: &str,
        filename: &str,
    ) -> String {
        format!("{school_id}/{domain}/{aggregate}/{id}/{filename}")
    }

    /// Parses a namespaced key back into its five components.
    /// Returns [`FileStorageError::InvalidKey`] if the input
    /// does not contain exactly four `/` separators (i.e. it is
    /// not `school/domain/aggregate/id/filename` shaped).
    ///
    /// The parser uses `splitn(5, '/')` so a `filename` that
    /// itself contains `/` is preserved verbatim (the fifth
    /// component captures the rest of the string after the
    /// fourth separator).
    pub fn parse_namespaced_key(
        namespaced: &str,
    ) -> Result<(String, String, String, String, String), FileStorageError> {
        let parts: Vec<&str> = namespaced.splitn(5, '/').collect();
        if parts.len() != 5 {
            return Err(FileStorageError::InvalidKey(format!(
                "namespaced key must have 5 '/'-separated components, got {}: {namespaced}",
                parts.len()
            )));
        }
        Ok((
            parts[0].to_owned(),
            parts[1].to_owned(),
            parts[2].to_owned(),
            parts[3].to_owned(),
            parts[4].to_owned(),
        ))
    }

    /// Returns `true` iff the first `/`-separated segment of
    /// `namespaced` equals `school_id`. This is a lexical check
    /// only — it does not consult a tenant table or a
    /// `TenantContext`.
    #[must_use]
    pub fn is_in_tenant(namespaced: &str, school_id: &str) -> bool {
        match namespaced.split_once('/') {
            Some((prefix, _)) => prefix == school_id,
            None => false,
        }
    }
}

// ---------------------------------------------------------------------------
// 4. VisibilityService
// ---------------------------------------------------------------------------

/// Classifies a [`Visibility`] value and answers the
/// "can this user read this file?" question for the engine's
/// read-side authorisation middleware.
///
/// The service holds no state; every method is a pure function
/// over the inputs.
#[derive(Debug, Default, Clone, Copy)]
pub struct VisibilityService;

impl VisibilityService {
    /// Constructs a new `VisibilityService`. The struct is a
    /// zero-sized marker; the constructor is provided for API
    /// symmetry with the other services in this module.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns `true` iff the visibility is [`Visibility::Private`].
    #[must_use]
    pub const fn is_private(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Private)
    }

    /// Returns `true` iff the visibility is [`Visibility::Public`].
    #[must_use]
    pub const fn is_public(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::Public)
    }

    /// Returns `true` iff the visibility is
    /// [`Visibility::TenantPrivate`] — i.e. the file is
    /// readable by any authenticated user inside the same
    /// tenant.
    #[must_use]
    pub const fn is_tenant_scoped(visibility: &Visibility) -> bool {
        matches!(visibility, Visibility::TenantPrivate)
    }

    /// Returns `true` iff a user with `user_school_id` is
    /// permitted to access a file with the given visibility.
    ///
    /// The current policy is:
    ///
    /// - [`Visibility::Public`] — always accessible.
    /// - [`Visibility::TenantPrivate`] — always accessible
    ///   (the engine relies on the adapter's key-prefix check
    ///   for the authoritative tenant boundary; the
    ///   `user_school_id` parameter is plumbed through for
    ///   future tightening without changing the call site).
    /// - [`Visibility::Private`] — not accessible (the caller
    ///   must hold a valid signed URL).
    #[must_use]
    #[allow(clippy::unused_self)]
    pub const fn can_access(visibility: &Visibility, _user_school_id: &str) -> bool {
        match visibility {
            Visibility::Public => true,
            Visibility::TenantPrivate => true,
            Visibility::Private => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Lowercase hex encoder. Stays in this module rather than
/// pulling in a `hex` dependency that the rest of the crate does
/// not need.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Constant-time string equality. Iterates over the full length
/// of the shorter string regardless of where the first mismatch
/// is found, so the comparison does not leak the position of the
/// first differing byte.
///
/// The loop runs over the length of `a`; if `b` is shorter, the
/// pre-loop length check returns `false`. The result still
/// depends only on equality of the inputs (no timing leak of
/// `b`'s length beyond the immediate `a.len() != b.len()` check,
/// which is acceptable for HMAC hex digests of fixed length).
fn constant_time_eq_str(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.as_bytes().iter().zip(b.as_bytes().iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Formats a Unix timestamp (seconds since `1970-01-01T00:00:00Z`)
/// as RFC 3339 with second precision and a `Z` suffix.
///
/// The conversion uses Howard Hinnant's `civil_from_days` algorithm
/// (public domain) rather than pulling in a `chrono` or `time`
/// dependency — the crate's `Cargo.toml` is intentionally minimal,
/// and the helper only needs whole-second precision for signed-URL
/// expiry (sub-second precision would be lost to the HMAC's whole-
/// second format anyway).
fn unix_secs_to_rfc3339(secs: u64) -> String {
    let secs = i64::try_from(secs.min(i64::MAX as u64)).unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let time_secs_i64 = secs.rem_euclid(86_400);
    let time_secs = u32::try_from(time_secs_i64).unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    let hour = time_secs / 3600;
    let minute = (time_secs / 60) % 60;
    let second = time_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converts days since the Unix epoch (`1970-01-01`) to a
/// proleptic Gregorian `(year, month, day)` triple.
///
/// Howard Hinnant's `civil_from_days`, public domain. The
/// algorithm is exact for the entire `i64` range — the engine
/// does not need to handle BC dates (signed URLs only deal in
/// future times), so the function is only used with positive
/// inputs.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe_i64 = z - era * 146_097;
    let doe = u64::try_from(doe_i64).unwrap_or(0);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let yoe_i64 = i64::try_from(yoe).unwrap_or(0);
    let y = yoe_i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d_i64 = i64::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(0);
    let d = u32::try_from(d_i64).unwrap_or(0);
    let mp_i64 = i64::try_from(mp).unwrap_or(0);
    let m_raw = mp_i64;
    let m = if m_raw < 10 { m_raw + 3 } else { m_raw - 9 };
    let m_u32 = u32::try_from(m).unwrap_or(0);
    let y = if m <= 2 { y + 1 } else { y };
    (y, m_u32, d)
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

    // ---- 1. ChecksumService -----------------------------------------

    #[test]
    fn checksum_sha256_matches_fips_180_4_test_vector() {
        // FIPS 180-4 Appendix B.1: SHA-256("abc") =
        // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert_eq!(
            ChecksumService::compute_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        // Empty input → the well-known SHA-256 of nothing.
        assert_eq!(
            ChecksumService::compute_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn checksum_verify_accepts_matching_and_rejects_other() {
        let payload = b"the quick brown fox jumps over the lazy dog";
        let hex = ChecksumService::compute_sha256(payload);
        assert!(ChecksumService::verify(payload, &hex));

        // Different payload fails.
        assert!(!ChecksumService::verify(b"the quick brown fox", &hex));

        // Same length, different content (constant-time guard).
        let bad_hex = "0".repeat(hex.len());
        assert!(!ChecksumService::verify(payload, &bad_hex));

        // Length mismatch fails.
        assert!(!ChecksumService::verify(payload, "abc"));
    }

    #[test]
    fn checksum_etag_wraps_digest_in_quotes() {
        let etag = ChecksumService::compute_etag(b"abc");
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        // 64 hex chars + 2 quotes.
        assert_eq!(etag.len(), 66);
        assert_eq!(
            etag,
            "\"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad\""
        );
    }

    // ---- 2. SignedUrlService ----------------------------------------

    #[test]
    fn signed_url_round_trip_through_verify() {
        let svc = SignedUrlService::new("super-secret-key");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        let future_secs = now.saturating_add(60);
        let expires_at =
            Timestamp::parse_rfc3339(&unix_secs_to_rfc3339(future_secs)).expect("parse future");

        let sig = svc
            .sign("students/photos/ada.jpg", expires_at)
            .expect("sign");
        assert_eq!(sig.len(), 64); // 32-byte SHA-256 → 64 hex chars
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));

        // Same key + same expiry verifies.
        assert!(svc.verify("students/photos/ada.jpg", expires_at, &sig));

        // Different key fails.
        assert!(!svc.verify("students/photos/other.jpg", expires_at, &sig));

        // Different expiry fails.
        let other_secs = future_secs.saturating_add(1);
        let other_expiry =
            Timestamp::parse_rfc3339(&unix_secs_to_rfc3339(other_secs)).expect("parse other");
        assert!(!svc.verify("students/photos/ada.jpg", other_expiry, &sig));

        // Past expiry fails even with a correct signature.
        let past_secs = now.saturating_sub(60);
        let past = Timestamp::parse_rfc3339(&unix_secs_to_rfc3339(past_secs)).expect("parse past");
        let past_sig = svc
            .sign("students/photos/ada.jpg", past)
            .expect("sign past");
        assert!(!svc.verify("students/photos/ada.jpg", past, &past_sig));

        // Debug redacts the secret.
        let dbg = format!("{svc:?}");
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("super-secret-key"));
    }

    #[test]
    fn signed_url_build_signed_url_round_trips() {
        let svc = SignedUrlService::new("another-secret");
        let url = svc.build_signed_url(
            "https://files.example.com",
            "students/photos/ada.jpg",
            Duration::from_secs(60),
        );

        // The URL has the canonical shape.
        assert!(url.starts_with("https://files.example.com/students/photos/ada.jpg?expires="));
        assert!(url.contains("&signature="));

        // The signature parses out and verifies with the service.
        let signature_start =
            url.rfind("&signature=").expect("signature marker") + "&signature=".len();
        let signature = &url[signature_start..];
        let expires_str = url
            .split("expires=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .expect("expires query param");
        let expires_at = Timestamp::parse_rfc3339(expires_str).expect("parse expires");
        assert!(svc.verify("students/photos/ada.jpg", expires_at, signature));
    }

    // ---- 3. KeyNamespaceService --------------------------------------

    #[test]
    fn key_namespace_round_trips() {
        let composed = KeyNamespaceService::namespace_key(
            "school-abc",
            "academic",
            "student",
            "std-001",
            "photos/ada.jpg",
        );
        assert_eq!(
            composed,
            "school-abc/academic/student/std-001/photos/ada.jpg"
        );

        let parsed = KeyNamespaceService::parse_namespaced_key(&composed).expect("parse");
        assert_eq!(parsed.0, "school-abc");
        assert_eq!(parsed.1, "academic");
        assert_eq!(parsed.2, "student");
        assert_eq!(parsed.3, "std-001");
        assert_eq!(parsed.4, "photos/ada.jpg");
    }

    #[test]
    fn key_namespace_parse_rejects_malformed() {
        // No separators: 1 component.
        let err = KeyNamespaceService::parse_namespaced_key("not-a-key");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));

        // 3 components.
        let err = KeyNamespaceService::parse_namespaced_key("a/b/c");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));

        // 4 components.
        let err = KeyNamespaceService::parse_namespaced_key("a/b/c/d");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));

        // Empty input.
        let err = KeyNamespaceService::parse_namespaced_key("");
        assert!(matches!(err, Err(FileStorageError::InvalidKey(_))));
    }

    #[test]
    fn key_namespace_is_in_tenant_checks_first_segment() {
        let key = "school-abc/academic/student/std-001/photo.jpg";
        assert!(KeyNamespaceService::is_in_tenant(key, "school-abc"));
        assert!(!KeyNamespaceService::is_in_tenant(key, "school-xyz"));
        assert!(!KeyNamespaceService::is_in_tenant(
            "no-separator",
            "school-abc"
        ));
    }

    // ---- 4. VisibilityService ---------------------------------------

    #[test]
    fn visibility_classifies_correctly() {
        assert!(VisibilityService::is_private(&Visibility::Private));
        assert!(!VisibilityService::is_private(&Visibility::Public));
        assert!(!VisibilityService::is_private(&Visibility::TenantPrivate));

        assert!(VisibilityService::is_public(&Visibility::Public));
        assert!(!VisibilityService::is_public(&Visibility::Private));
        assert!(!VisibilityService::is_public(&Visibility::TenantPrivate));

        assert!(VisibilityService::is_tenant_scoped(
            &Visibility::TenantPrivate
        ));
        assert!(!VisibilityService::is_tenant_scoped(&Visibility::Private));
        assert!(!VisibilityService::is_tenant_scoped(&Visibility::Public));
    }

    #[test]
    fn visibility_can_access_policy() {
        // Public — always accessible.
        assert!(VisibilityService::can_access(
            &Visibility::Public,
            "school-a"
        ));
        assert!(VisibilityService::can_access(
            &Visibility::Public,
            "school-b"
        ));

        // TenantPrivate — always accessible (current policy; the
        // engine relies on the adapter's key-prefix check for the
        // authoritative boundary).
        assert!(VisibilityService::can_access(
            &Visibility::TenantPrivate,
            "school-a"
        ));
        assert!(VisibilityService::can_access(
            &Visibility::TenantPrivate,
            "school-b"
        ));

        // Private — never accessible via can_access (caller must
        // hold a signed URL).
        assert!(!VisibilityService::can_access(
            &Visibility::Private,
            "school-a"
        ));
    }
}
