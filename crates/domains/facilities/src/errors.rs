//! # Facilities errors
//!
//! The single error enum the facilities domain uses. Per
//! `docs/code-standards.md`, all fallible APIs return
//! `Result<T, DomainError>` — the engine's canonical error
//! type. This enum is reserved for facilities-specific helpers
//! that need to attach context before bubbling up to
//! `DomainError`.

#![allow(missing_docs)]
#![allow(unused_imports)]

use educore_core::error::DomainError;

/// Facilities-specific error helpers. The domain surfaces
/// `DomainError` directly to the dispatcher; this module is
/// a thin shim that produces a `DomainError::Validation`
/// with a facilities-prefixed message.
pub struct FacilitiesError;

impl FacilitiesError {
    /// Returns a `Validation` error with the given facilities
    /// message.
    pub fn validation(msg: impl Into<String>) -> DomainError {
        DomainError::validation(format!("facilities: {}", msg.into()))
    }

    /// Returns a `Conflict` error with the given facilities
    /// message.
    pub fn conflict(msg: impl Into<String>) -> DomainError {
        DomainError::conflict(format!("facilities: {}", msg.into()))
    }

    /// Returns a `NotFound` error with the given facilities
    /// message.
    pub fn not_found(msg: impl Into<String>) -> DomainError {
        DomainError::not_found(format!("facilities: {}", msg.into()))
    }

    /// Returns a `NotSupported` error with the given facilities
    /// message.
    pub fn not_supported(msg: impl Into<String>) -> DomainError {
        DomainError::not_supported(format!("facilities: {}", msg.into()))
    }
}
