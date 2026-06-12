//! Platform-domain value objects.
//!
//! These are the typed wrappers the platform aggregates depend
//! on: [`EmailAddress`], [`PhoneNumber`], [`HashedPassword`],
//! [`SchoolStatus`], [`UserStatus`], [`PackageId`], and
//! [`RoleId`]. All are validated at construction, comparable by
//! value, and serialisable via `serde`.
//!
//! Per `docs/specs/platform/value-objects.md`:
//! - `EmailAddress` — RFC 5322, length cap 200, lowercased on
//!   construction for case-insensitive equality.
//! - `PhoneNumber` — E.164 (`+` prefix, 4..=15 digits).
//! - `HashedPassword` — wraps a `secrecy::SecretString` containing
//!   the password hash (the plaintext is never stored; the
//!   [`HashedPassword::from_plaintext`] helper exists for the
//!   bootstrap test path and delegates to a port-driven hasher in
//!   production).

use std::fmt;

use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

/// A validated email address.
///
/// The address is lowercased on construction. Comparisons are
/// case-insensitive at the byte level after the lowercasing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EmailAddress(String);

impl EmailAddress {
    /// The maximum length of an email address (per the
    /// `value-objects.md` spec).
    pub const MAX_LEN: usize = 200;

    /// Constructs a new `EmailAddress` from a string. The
    /// address is trimmed, lowercased, and validated against a
    /// conservative RFC-5322-ish shape: exactly one `@`, a
    /// non-empty local part, a non-empty domain part, and a
    /// total length of at most [`Self::MAX_LEN`].
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let normalised = raw.into().trim().to_lowercase();
        Self::validate(&normalised)?;
        Ok(Self(normalised))
    }

    /// Returns the email address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates the candidate string without consuming it.
    fn validate(s: &str) -> Result<()> {
        if s.is_empty() {
            return Err(DomainError::Validation(
                "email must not be empty".to_owned(),
            ));
        }
        if s.len() > Self::MAX_LEN {
            return Err(DomainError::Validation(format!(
                "email length {} exceeds {}",
                s.len(),
                Self::MAX_LEN
            )));
        }
        let Some((local, domain)) = s.split_once('@') else {
            return Err(DomainError::Validation(format!("email missing '@': {s:?}")));
        };
        if local.is_empty() {
            return Err(DomainError::Validation(format!(
                "email local part is empty: {s:?}"
            )));
        }
        if domain.is_empty() || !domain.contains('.') {
            return Err(DomainError::Validation(format!(
                "email domain part is malformed: {s:?}"
            )));
        }
        if local.chars().any(|c| c.is_whitespace() || c == '@') {
            return Err(DomainError::Validation(format!(
                "email local part contains whitespace: {s:?}"
            )));
        }
        if domain.chars().any(|c| c.is_whitespace() || c == '@') {
            return Err(DomainError::Validation(format!(
                "email domain part contains whitespace: {s:?}"
            )));
        }
        Ok(())
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<EmailAddress> for String {
    fn from(e: EmailAddress) -> Self {
        e.0
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated phone number in E.164 format: a leading `+`
/// followed by 4..=15 digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// The minimum digit count (per E.164).
    pub const MIN_DIGITS: usize = 4;
    /// The maximum digit count (per E.164).
    pub const MAX_DIGITS: usize = 15;

    /// Constructs a new `PhoneNumber` from a raw string. The
    /// string is normalised: a leading `+` is required, all
    /// whitespace and common separators (`-`, ` `, `(`, `)`) are
    /// removed, and the digit count is checked against
    /// `MIN_DIGITS..=MAX_DIGITS`.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let candidate = raw.into();
        let normalised = Self::normalise(&candidate);
        Self::validate(&normalised)?;
        Ok(Self(normalised))
    }

    /// Returns the phone number as a string slice (E.164 form).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn normalise(raw: &str) -> String {
        let mut out = String::with_capacity(raw.len());
        // Preserve a single leading `+` if the raw input had
        // one; drop every other occurrence (so e.g. `"+1+415"`
        // becomes `"+1415"` and the validation step rejects
        // it on the digit count).
        let mut seen_plus = false;
        for c in raw.chars() {
            match c {
                '+' if !seen_plus => {
                    out.push('+');
                    seen_plus = true;
                }
                '+' => {}
                '-' | ' ' | '(' | ')' | '\t' => {}
                other => out.push(other),
            }
        }
        out
    }

    fn validate(s: &str) -> Result<()> {
        if !s.starts_with('+') {
            return Err(DomainError::Validation(format!(
                "phone number must start with '+': {s:?}"
            )));
        }
        let digits = &s[1..];
        if digits.len() < Self::MIN_DIGITS || digits.len() > Self::MAX_DIGITS {
            return Err(DomainError::Validation(format!(
                "phone number digit count {} outside {}..={}",
                digits.len(),
                Self::MIN_DIGITS,
                Self::MAX_DIGITS
            )));
        }
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::Validation(format!(
                "phone number contains non-digit characters: {s:?}"
            )));
        }
        Ok(())
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<String> for PhoneNumber {
    type Error = DomainError;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl From<PhoneNumber> for String {
    fn from(p: PhoneNumber) -> Self {
        p.0
    }
}

impl AsRef<str> for PhoneNumber {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A hashed password. The plaintext is never stored; the
/// `SecretString` holds the engine's hash string (e.g. a
/// PHC-formatted Argon2id string produced by the engine's
/// password-hashing port). The `Debug` impl redacts the secret,
/// and the type intentionally exposes the hash only via
/// [`HashedPassword::expose_hash`] for code paths that need to
/// round-trip the value to a storage column.
///
/// Serde behaviour: the `Serialize` / `Deserialize` impls
/// round-trip the hash as an ordinary JSON string. The
/// `secrecy::SecretString` type is intentionally non-`Serialize`
/// (the engine never wants a hash to appear in a JSON payload
/// by accident), so the `HashedPassword` impls do not delegate
/// to `SecretString` and instead serialise the inner string
/// via [`ExposeSecret::expose_secret`].
#[derive(Clone)]
pub struct HashedPassword(secrecy::SecretString);

impl HashedPassword {
    /// Wraps a pre-computed hash string. The caller is
    /// responsible for producing the hash via the engine's
    /// password port (e.g. Argon2id in
    /// `educore-auth`). The platform crate does not perform
    /// hashing itself: the password-hasher is a port, and the
    /// adapter that implements it lives in the `adapters` tier
    /// (which `cross-cutting` crates may not depend on).
    pub fn from_hash(hash: impl Into<String>) -> Self {
        Self(secrecy::SecretString::from(hash.into()))
    }

    /// Exposes the hash string. Use only on code paths that
    /// need to serialise the hash to a wire format (e.g. the
    /// storage adapter's column write); the caller is
    /// responsible for not logging or otherwise leaking the
    /// returned value.
    #[must_use]
    pub fn expose_hash(&self) -> &str {
        self.0.expose_secret()
    }

    /// Returns `true` if the wrapped hash string is empty.
    /// Used by the storage adapter to differentiate "no password
    /// set" (an SSO-only user) from "password set to a value".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.expose_secret().is_empty()
    }
}

impl fmt::Debug for HashedPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HashedPassword(<redacted>)")
    }
}

impl PartialEq for HashedPassword {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time equality on the secret would be ideal;
        // for an aggregate field used in a few equality checks
        // (testing the round-trip of a hash, mostly) byte
        // equality is sufficient and does not leak anything
        // beyond what `Debug` would already reveal.
        self.0.expose_secret().as_bytes() == other.0.expose_secret().as_bytes()
    }
}

impl Eq for HashedPassword {}

impl Serialize for HashedPassword {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}

impl<'de> Deserialize<'de> for HashedPassword {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(Self(secrecy::SecretString::from(raw)))
    }
}

/// The lifecycle status of a [`School`](crate::aggregate::School).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SchoolStatus {
    /// The school has signed up but is awaiting approval.
    #[default]
    Pending,
    /// The school is live and accepting traffic.
    Approved,
    /// The school is temporarily disabled by a platform operator.
    Suspended,
    /// The school is live and fully onboarded. The distinction
    /// between `Active` and `Approved` is a legacy of the
    /// Schoolify data model; the engine treats them identically
    /// for query purposes and exposes both for storage
    /// round-trips.
    Active,
}

impl SchoolStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Suspended => "suspended",
            Self::Active => "active",
        }
    }

    /// Returns `true` if the school can accept new sessions.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Approved | Self::Active)
    }
}

impl fmt::Display for SchoolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// The lifecycle status of a [`User`](crate::aggregate::User).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserStatus {
    /// The user can sign in.
    #[default]
    Active,
    /// The user is currently disabled (the legacy "inactive"
    /// label). The engine routes sign-in attempts for `Inactive`
    /// users to a 403.
    Inactive,
    /// The user is suspended pending an investigation. Engine
    /// sign-in attempts fail with a 403.
    Suspended,
}

impl UserStatus {
    /// Returns the canonical snake_case wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Suspended => "suspended",
        }
    }

    /// Returns `true` if the user can sign in.
    #[must_use]
    pub const fn can_authenticate(self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

/// A per-school package identifier. Wraps a `Uuid`; the
/// underlying catalog of packages is global (a `PackageId`
/// minted for one school is meaningful to all schools), but the
/// school-row binding `package_id` lives on [`School`](crate::aggregate::School).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PackageId(pub Uuid);

impl PackageId {
    /// Constructs a `PackageId` from a `Uuid`.
    #[must_use]
    pub const fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// Returns the inner `Uuid`.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for PackageId {
    fn from(u: Uuid) -> Self {
        Self(u)
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A role identifier scoped to a school. The pair
/// `(school_id, role_uuid)` is globally unique; a role belongs
/// to exactly one school.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub SchoolId, pub Uuid);

impl RoleId {
    /// Constructs a `RoleId` for `school` and the given inner
    /// `Uuid`.
    #[must_use]
    pub const fn new(school: SchoolId, id: Uuid) -> Self {
        Self(school, id)
    }

    /// Returns the owning school.
    #[must_use]
    pub const fn school_id(self) -> SchoolId {
        self.0
    }

    /// Returns the inner `Uuid`.
    #[must_use]
    pub const fn role_uuid(self) -> Uuid {
        self.1
    }
}

impl fmt::Display for RoleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0, self.1)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn email_lowercases_on_construction() {
        let e = EmailAddress::new("Foo@Bar.COM").unwrap();
        assert_eq!(e.as_str(), "foo@bar.com");
    }

    #[test]
    fn email_rejects_empty() {
        assert!(EmailAddress::new("").is_err());
    }

    #[test]
    fn email_rejects_missing_at() {
        assert!(EmailAddress::new("foobar.com").is_err());
    }

    #[test]
    fn email_rejects_missing_dot() {
        assert!(EmailAddress::new("foo@bar").is_err());
    }

    #[test]
    fn email_rejects_overlong() {
        let long = format!("{}@example.com", "a".repeat(EmailAddress::MAX_LEN));
        assert!(EmailAddress::new(long).is_err());
    }

    #[test]
    fn email_round_trips_through_string() {
        let e = EmailAddress::new("Ada@Example.COM").unwrap();
        let s: String = e.clone().into();
        assert_eq!(s, "ada@example.com");
        let back = EmailAddress::try_from(s).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn phone_strips_separators_and_validates() {
        let p = PhoneNumber::new("+1 (415) 555-2671").unwrap();
        assert_eq!(p.as_str(), "+14155552671");
    }

    #[test]
    fn phone_rejects_missing_plus() {
        assert!(PhoneNumber::new("14155552671").is_err());
    }

    #[test]
    fn phone_rejects_too_few_digits() {
        assert!(PhoneNumber::new("+12").is_err());
    }

    #[test]
    fn phone_rejects_too_many_digits() {
        let digits = "1".repeat(PhoneNumber::MAX_DIGITS + 1);
        assert!(PhoneNumber::new(format!("+{digits}")).is_err());
    }

    #[test]
    fn phone_rejects_non_digit() {
        assert!(PhoneNumber::new("+1415a552671").is_err());
    }

    #[test]
    fn hashed_password_from_hash_round_trip() {
        let p = HashedPassword::from_hash("$argon2id$v=19$m=19456,t=2,p=1$abc$def");
        assert_eq!(p.expose_hash(), "$argon2id$v=19$m=19456,t=2,p=1$abc$def");
        assert_eq!(format!("{p:?}"), "HashedPassword(<redacted>)");
    }

    #[test]
    fn school_status_round_trip() {
        for s in [
            SchoolStatus::Pending,
            SchoolStatus::Approved,
            SchoolStatus::Suspended,
            SchoolStatus::Active,
        ] {
            assert!(!s.as_str().is_empty());
        }
        assert!(SchoolStatus::Approved.is_live());
        assert!(SchoolStatus::Active.is_live());
        assert!(!SchoolStatus::Pending.is_live());
        assert!(!SchoolStatus::Suspended.is_live());
    }

    #[test]
    fn user_status_can_authenticate() {
        assert!(UserStatus::Active.can_authenticate());
        assert!(!UserStatus::Inactive.can_authenticate());
        assert!(!UserStatus::Suspended.can_authenticate());
    }

    #[test]
    fn package_id_round_trip() {
        let u = Uuid::now_v7();
        let p = PackageId::from_uuid(u);
        assert_eq!(p.as_uuid(), u);
        assert_eq!(p.to_string(), u.to_string());
    }

    #[test]
    fn role_id_displays_school_and_uuid() {
        let school = SchoolId(Uuid::nil());
        let u = Uuid::now_v7();
        let r = RoleId::new(school, u);
        assert_eq!(r.school_id(), school);
        assert_eq!(r.role_uuid(), u);
        assert_eq!(r.to_string(), format!("{school}/{u}"));
    }
}
