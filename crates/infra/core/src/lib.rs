//! # educore-core
//!
//! Stub. Public surface only; implementation will be filled in later.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Clock port: produces `Timestamp` values for the engine. The reference
/// implementation is `SystemClock`; tests use a frozen `TestClock`.
pub mod clock;

/// Error type, error categories, and `Result` alias used by every crate.
pub mod error;

/// Typed identifiers (`SchoolId`, `UserId`, `StudentId`, ...) wrapping UUIDv7
/// values, plus ID generation ports.
pub mod ids;

/// The query AST, value wrappers, and pattern types emitted by the
/// `#[derive(DomainQuery)]` macro and consumed by storage adapters.
pub mod query;

/// Tenant context, request-scoped metadata, and the `SchoolId` filter that
/// every command and query carries.
pub mod tenant;

/// Common value objects (email, phone, currency, date ranges) shared across
/// the workspace.
pub mod value_objects;
