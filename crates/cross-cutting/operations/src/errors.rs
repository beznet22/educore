//! # educore-operations errors
//!
//! The [`OperationsDomainError`] enum and [`Result`] type alias.
//! Mirrors the settings crate pattern.

use thiserror::Error;

use educore_core::error::DomainError;
use educore_events::errors::EventError;

/// The error type for the operations-domain crate.
#[derive(Debug, Error)]
pub enum OperationsDomainError {
    /// Validation failed (empty `file_name`, invalid `ip_address`, etc.).
    #[error("validation error: {0}")]
    Validation(String),

    /// Conflict with current state (e.g. update on soft-deleted
    /// aggregate, restore-in-progress blocking delete).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Actor lacks the required capability.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Infrastructure failure (storage, bus, file-storage, etc.).
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}

/// Result alias for the operations-domain crate.
pub type Result<T> = std::result::Result<T, OperationsDomainError>;

impl From<DomainError> for OperationsDomainError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::Validation(s) => Self::Validation(s),
            DomainError::NotFound(s) => Self::NotFound(s),
            DomainError::Conflict(s) => Self::Conflict(s),
            DomainError::Forbidden(s) => Self::Forbidden(s),
            DomainError::TenantViolation(s) => Self::Forbidden(s),
            DomainError::NotSupported(s) => Self::Validation(s),
            DomainError::Infrastructure(src) => Self::Infrastructure(src.to_string()),
        }
    }
}

impl From<EventError> for OperationsDomainError {
    fn from(e: EventError) -> Self {
        Self::Infrastructure(e.to_string())
    }
}
