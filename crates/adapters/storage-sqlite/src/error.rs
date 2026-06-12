//! Error type for the SQLite adapter.
//!
//! `sqlx::Error` is a concrete type, but
//! `educore_core::error::DomainError::infrastructure` requires
//! `impl std::error::Error + Send + Sync + 'static`. This module
//! provides a `StringError` newtype so adapter code can
//! propagate human-readable error messages without depending on
//! `anyhow` (which is an internal-utility crate per
//! `ADR-015`).

use std::error::Error as StdError;
use std::fmt;

/// A `String`-backed `std::error::Error` impl. Used by
/// adapter code to wrap `format!`-style error messages
/// without depending on `anyhow` (which is reserved for
/// internal glue per `AGENTS.md`).
#[derive(Debug)]
pub(crate) struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl StdError for StringError {}

impl From<StringError> for educore_core::error::DomainError {
    fn from(e: StringError) -> Self {
        educore_core::error::DomainError::infrastructure(e)
    }
}
