//! # Academic-domain error helpers
//!
//! Phase 3 does not introduce a new error type: the engine's
//! universal [`DomainError`] already covers the variants the
//! academic services emit (`Validation`, `Conflict`,
//! `NotFound`, `Forbidden`, ...). This module re-exports
//! the universal type as [`AcademicError`] for symmetry with
//! `educore_platform::errors::PlatformError`. Domain crates
//! that grow a domain-specific error helper later can extend
//! this module without breaking the public surface.

pub use educore_core::error::DomainError as AcademicError;
