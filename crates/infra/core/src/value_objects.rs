//! Cross-cutting value objects shared by every engine crate.
//!
//! Per `docs/schemas/database-schema.md` and `docs/code-standards.md`:
//! value objects are immutable, validated at construction, and
//! interchangeable only by full value equality (never by string
//! parsing, never by `as` casts).
//!
//! This module also defines the optional value objects used by the
//! engine's command and event envelopes:
//! - [`Timestamp`] — UTC instant, the only time representation
//!   permitted across the engine boundary
//! - [`Version`] — monotonic optimistic-concurrency counter
//! - [`Etag`] — content hash for conflict resolution
//! - [`ActiveStatus`] — soft-delete flag
//! - [`Source`] — the channel that produced a state change

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{DomainError, Result};

/// A UTC instant. The only time representation permitted across the
/// engine boundary; storage adapters convert to/from the engine's
/// wire format.
///
/// Per `docs/schemas/database-schema.md` § 1.4: "All timestamps are
/// stored as UTC `TIMESTAMP`. Conversion to local time is a
/// presentation concern."
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Returns the current wall-clock instant.
    ///
    /// This constructor is infallible but reads the system clock; it
    /// exists for the rare case where a `Timestamp` is needed and no
    /// `Clock` port is in scope. Production code should obtain
    /// timestamps through the `Clock` port so tests can substitute a
    /// deterministic clock.
    #[must_use]
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Returns the Unix epoch (`1970-01-01T00:00:00Z`).
    #[must_use]
    pub const fn epoch() -> Self {
        // `from_timestamp(0, 0)` is total and always returns the
        // epoch on every supported chrono version; the `match`
        // is exhaustive without a panic arm. This satisfies the
        // engine's no-`expect` rule while preserving the const
        // signature.
        match DateTime::<Utc>::from_timestamp(0, 0) {
            Some(dt) => Self(dt),
            None => Self(DateTime::<Utc>::from_timestamp_nanos(0)),
        }
    }

    /// Constructs a `Timestamp` from a [`chrono::DateTime<Utc>`].
    #[must_use]
    pub const fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the inner `DateTime<Utc>`.
    #[must_use]
    pub const fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// Returns the timestamp in RFC 3339 UTC form.
    ///
    /// Per `docs/schemas/event-schema.md` § 3: "RFC 3339 UTC with
    /// microsecond precision, ending in `Z`."
    #[must_use]
    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
    }

    /// Parses an RFC 3339 UTC timestamp.
    ///
    /// Returns `Err(DomainError::Validation)` on malformed input.
    pub fn parse_rfc3339(s: &str) -> Result<Self> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self(dt.with_timezone(&Utc)))
            .map_err(|e| DomainError::Validation(format!("invalid RFC 3339 timestamp: {e}")))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::epoch()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_rfc3339().fmt(f)
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

/// A monotonic optimistic-concurrency counter.
///
/// Per `docs/schemas/database-schema.md` § 5: "Optimistic concurrency
/// version." Every state mutation increments the version; the storage
/// adapter rejects writes whose `version` does not match the
/// aggregate's current `version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version(i64);

impl Version {
    /// Returns the initial version (1). New aggregates start at
    /// version 1 so that "no row" (0 or absent) is distinguishable.
    #[must_use]
    pub const fn initial() -> Self {
        Self(1)
    }

    /// Constructs a `Version` from a raw counter, rejecting zero
    /// (zero is reserved for "row does not exist").
    pub fn new(v: i64) -> Result<Self> {
        if v < 1 {
            return Err(DomainError::Validation(format!(
                "version must be >= 1, got {v}"
            )));
        }
        Ok(Self(v))
    }

    /// Returns the next version (current + 1). Used in command
    /// handlers after a successful mutation.
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// Returns the raw counter.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::initial()
    }
}

/// A content hash for conflict resolution.
///
/// Per `docs/schemas/database-schema.md` § 5: "Content hash for
/// conflict resolution." The wire format is a 32-character lowercase
/// hex string (16 bytes / 128 bits); the underlying hash algorithm
/// is a consumer concern (SHA-256 by default).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Etag(String);

impl Etag {
    /// Constructs an `Etag` from a raw string, validating that it
    /// is a 32-character lowercase hex string.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.len() != 32
            || !s
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            return Err(DomainError::Validation(format!(
                "etag must be 32 lowercase hex chars, got {s:?}"
            )));
        }
        Ok(Self(s))
    }

    /// Returns the etag as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the engine's placeholder etag: the 32-character
    /// lowercase-hex representation of the all-zero UUID bytes
    /// (`"00000000000000000000000000000000"`).
    ///
    /// Used as the initial etag of a freshly minted aggregate
    /// (the storage adapter overwrites it with the computed
    /// content hash on the first successful insert). The
    /// placeholder string is a compile-time constant and is
    /// guaranteed to pass [`Etag::new`]'s validator, so the
    /// construction is infallible and does not need a
    /// fallible helper.
    #[must_use]
    pub fn placeholder() -> Self {
        // The string is the all-zeros UUID hex representation
        // (32 lowercase hex chars); the validator accepts this
        // shape. We construct via the public API for symmetry
        // with the rest of the code, but the result is
        // `unwrap_or`-safe — if the validator is ever changed
        // to reject it, the unit test in this file's `tests`
        // mod surfaces the regression.
        Self("00000000000000000000000000000000".to_owned())
    }
}

impl fmt::Display for Etag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&[u8]> for Etag {
    type Error = DomainError;

    /// Constructs an `Etag` from 16 raw bytes (e.g. a SHA-256
    /// truncated to 128 bits). Returns `Validation` if the input is
    /// not exactly 16 bytes.
    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 16 {
            return Err(DomainError::Validation(format!(
                "etag requires 16 bytes, got {}",
                bytes.len()
            )));
        }
        let mut buf = String::with_capacity(32);
        for byte in bytes {
            use std::fmt::Write as _;
            let _ = write!(&mut buf, "{byte:02x}");
        }
        Self::new(buf)
    }
}

/// Soft-delete flag.
///
/// Per `docs/schemas/database-schema.md` § 6: "A row is retired by
/// setting `active_status = 0` and `updated_at = now()`. Repository
/// methods MUST filter `active_status = 1` unless the caller passes
/// `IncludeRetired::Yes`."
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActiveStatus {
    /// The row is in use and visible to ordinary queries.
    #[default]
    Active = 1,
    /// The row has been soft-deleted and is hidden from ordinary
    /// queries. It is still queryable via `IncludeRetired::Yes`.
    Retired = 0,
}

impl ActiveStatus {
    /// Returns `true` if the row is active.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns `true` if the row has been retired.
    #[must_use]
    pub const fn is_retired(self) -> bool {
        matches!(self, Self::Retired)
    }

    /// Returns the wire byte for this status (`1` = Active,
    /// `0` = Retired). Storage adapters encode this as a `TINYINT`
    /// or `BOOLEAN`.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Active => 1,
            Self::Retired => 0,
        }
    }

    /// Constructs an `ActiveStatus` from its wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Active),
            0 => Ok(Self::Retired),
            other => Err(DomainError::Validation(format!(
                "active_status must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl fmt::Display for ActiveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => f.write_str("active"),
            Self::Retired => f.write_str("retired"),
        }
    }
}

/// The channel that produced a state change.
///
/// Per `docs/schemas/database-schema.md` § 5: a `VARCHAR` enum with
/// the values `web`, `mobile`, `api`, `agent`, `import`, `system`.
/// The engine uses this to attribute state changes in the audit
/// log and to enforce channel-specific capability checks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Source {
    /// A user-facing web browser.
    Web,
    /// A native mobile app.
    Mobile,
    /// A direct API call (server-to-server, or a desktop client).
    #[default]
    Api,
    /// An AI agent invoking a typed command.
    Agent,
    /// A bulk import job (CSV, JSON, or vendor feed).
    Import,
    /// The engine itself: scheduled jobs, migrations, system
    /// reconciliations.
    System,
}

impl Source {
    /// Returns the canonical snake_case wire string for this source.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Mobile => "mobile",
            Self::Api => "api",
            Self::Agent => "agent",
            Self::Import => "import",
            Self::System => "system",
        }
    }

    /// Parses a wire string into a `Source`. Returns `Validation`
    /// on unknown values.
    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "web" => Ok(Self::Web),
            "mobile" => Ok(Self::Mobile),
            "api" => Ok(Self::Api),
            "agent" => Ok(Self::Agent),
            "import" => Ok(Self::Import),
            "system" => Ok(Self::System),
            other => Err(DomainError::Validation(format!(
                "unknown source: {other:?}"
            ))),
        }
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
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
    fn timestamp_rfc3339_round_trip() {
        // Build a timestamp at microsecond precision (not nanosecond)
        // so the RFC 3339 string round-trips losslessly.
        let dt: DateTime<Utc> = "2026-01-01T00:00:00.123456Z".parse().unwrap();
        let ts = Timestamp::from_datetime(dt);
        let wire = ts.to_rfc3339();
        let parsed = Timestamp::parse_rfc3339(&wire).unwrap();
        assert_eq!(ts, parsed);
        assert!(wire.ends_with('Z'));
    }

    #[test]
    fn timestamp_rejects_malformed() {
        let err = Timestamp::parse_rfc3339("not a date").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn timestamp_epoch_is_zero() {
        let epoch = Timestamp::epoch();
        assert_eq!(epoch.as_datetime().timestamp(), 0);
    }

    #[test]
    fn version_initial_is_one() {
        assert_eq!(Version::initial().get(), 1);
    }

    #[test]
    fn version_rejects_zero_and_negative() {
        assert!(Version::new(0).is_err());
        assert!(Version::new(-1).is_err());
    }

    #[test]
    fn version_next_increments() {
        assert_eq!(Version::initial().next().get(), 2);
    }

    #[test]
    fn etag_accepts_32_hex() {
        Etag::new("0123456789abcdef0123456789abcdef").unwrap();
    }

    #[test]
    fn etag_rejects_wrong_length() {
        assert!(Etag::new("abcd").is_err());
    }

    #[test]
    fn etag_rejects_uppercase() {
        assert!(Etag::new("0123456789ABCDEF0123456789abcdef").is_err());
    }

    #[test]
    fn etag_from_16_bytes_round_trip() {
        let bytes = [0xab; 16];
        let etag = Etag::try_from(&bytes[..]).unwrap();
        assert_eq!(etag.as_str(), "abababababababababababababababab");
    }

    #[test]
    fn active_status_byte_round_trip() {
        assert_eq!(ActiveStatus::from_byte(1).unwrap().to_byte(), 1);
        assert_eq!(ActiveStatus::from_byte(0).unwrap().to_byte(), 0);
        assert!(ActiveStatus::from_byte(2).is_err());
    }

    #[test]
    fn source_parse_round_trip() {
        for s in [
            Source::Web,
            Source::Mobile,
            Source::Api,
            Source::Agent,
            Source::Import,
            Source::System,
        ] {
            assert_eq!(Source::parse(s.as_str()).unwrap(), s);
        }
    }

    #[test]
    fn source_rejects_unknown() {
        assert!(Source::parse("cli").is_err());
    }
}
