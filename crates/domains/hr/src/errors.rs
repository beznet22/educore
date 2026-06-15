//! # HR-domain error helpers
//!
//! Phase 6 does not introduce a domain-specific error type: the
//! engine's universal [`DomainError`] already covers the
//! variants the HR services emit. This module re-exports
//! the universal type as [`HrError`] for symmetry with
//! `educore_academic::errors::AcademicError` and
//! `educore_assessment::errors::AssessmentError`.

pub use educore_core::error::DomainError as HrError;
