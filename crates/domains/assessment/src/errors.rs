//! # Assessment-domain error helpers
//!
//! Phase 4 does not introduce a new error type: the engine's
//! universal [`DomainError`] already covers the variants the
//! assessment services emit (`Validation`, `Conflict`,
//! `NotFound`, `Forbidden`, ...). This module re-exports the
//! universal type as [`AssessmentError`] for symmetry with
//! `educore_academic::errors::AcademicError`. Domain crates
//! that grow a domain-specific error helper later can extend
//! this module without breaking the public surface.

pub use educore_core::error::DomainError as AssessmentError;
