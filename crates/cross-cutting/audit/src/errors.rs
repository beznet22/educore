//! The audit error type and the re-exported [`DomainError`].
//!
//! The audit crate does not introduce a new error category: every
//! fallible operation either surfaces an infrastructure error from
//! the underlying [`educore_storage::AuditLog`] port, or a publish
//! error from the [`educore_events::EventBus`] port. Both round-trip
//! through the engine-wide [`educore_core::error::DomainError`].
//!
//! This module re-exports [`DomainError`] under the local name
//! [`AuditError`] for callers that want to write `educore_audit::AuditError`
//! without depending on `educore_core` directly. The two types are
//! identical (an alias, not a new enum).
//!
//! Per `docs/code-standards.md` § 11 the engine uses a single
//! `DomainError` enum; introducing a parallel `AuditError` enum
//! would force every call site to downcast. The alias keeps the
//! public path ergonomic while preserving the single-error invariant.

pub use educore_core::error::DomainError as AuditError;

/// Convenience alias: the audit crate's `Result` is the engine's
/// `Result<T, DomainError>`.
pub use educore_core::error::Result;
