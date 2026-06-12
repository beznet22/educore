//! Platform-domain error helpers.
//!
//! Phase 2 does not introduce a new error type: the engine's
//! universal [`DomainError`](educore_core::error::DomainError)
//! already covers the variants the platform services emit
//! (`Validation`, `Conflict`, `NotFound`, ...). This module
//! re-exports the universal type as [`PlatformError`] for
//! symmetry with other domain crates that may grow a
//! domain-specific error helper later.

pub use educore_core::error::DomainError as PlatformError;
