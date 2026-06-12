//! Error type for the MySQL adapter.
//!
//! `sqlx::Error` is a concrete type that implements
//! `std::error::Error + Send + Sync + 'static`, so we can wrap it
//! directly with `DomainError::infrastructure`. This module is
//! kept crate-internal because adapters do not need a public error
//! type; every fallible API returns
//! `educore_core::error::Result<T>` (i.e.
//! `Result<T, DomainError>`).
//!
//! This file mirrors the PostgreSQL adapter's `error.rs` for
//! consistency across the adapter family. The `StringError` wrapper
//! is provided for cases where adapter code wants to format a
//! human-readable error message (e.g. when a `sqlx::Error`'s
//! display string is not informative enough on its own).

use std::error::Error as StdError;
use std::fmt;

/// A `String`-backed `std::error::Error` impl. Used by adapter code
/// to wrap `format!`-style error messages without depending on
/// `anyhow` (which is reserved for internal glue per
/// `AGENTS.md`).
#[derive(Debug)]
pub struct StringError(pub String);

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

/// Wraps a `sqlx::Error` (or any other adapter-internal error)
/// into a `DomainError::Infrastructure` variant. Used by the
/// adapter's `?` propagation.
#[inline]
pub fn map_infrastructure<E>(e: E) -> educore_core::error::DomainError
where
    E: StdError + Send + Sync + 'static,
{
    educore_core::error::DomainError::infrastructure(e)
}
