//! # educore-settings errors
//!
//! The `SettingsDomainError` enum and `Result` type alias. Mirrors
//! the events-domain pattern.

use thiserror::Error;

use educore_core::error::DomainError;
use educore_events::errors::EventError;

/// The error type for the settings crate.
#[derive(Debug, Error)]
pub enum SettingsDomainError {
    /// Validation failed (empty title, invalid format, etc.).
    #[error("validation error: {0}")]
    Validation(String),

    /// Conflict with current state (e.g. trying to delete a default
    /// theme, update a soft-deleted aggregate).
    #[error("conflict: {0}")]
    Conflict(String),

    /// Actor lacks the required capability.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Infrastructure failure (storage, bus, etc.).
    #[error("infrastructure error: {0}")]
    Infrastructure(String),
}

/// Result alias for the settings crate.
pub type Result<T> = std::result::Result<T, SettingsDomainError>;

impl From<DomainError> for SettingsDomainError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::Validation(s) => Self::Validation(s),
            DomainError::NotFound(s) => Self::NotFound(s),
            DomainError::Conflict(s) => Self::Conflict(s),
            DomainError::IdempotencyConflict {
                key,
                existing_outcome_ref,
            } => Self::Conflict(format!(
                "idempotency conflict for key {key}: existing outcome {existing_outcome_ref}"
            )),
            DomainError::IdempotencyPending { key, started_at } => Self::Conflict(format!(
                "idempotency pending for key {key} (started {started_at})"
            )),
            DomainError::Forbidden(s) => Self::Forbidden(s),
            DomainError::TenantViolation(s) => Self::Forbidden(s),
            DomainError::NotSupported(s) => Self::Validation(s),
            DomainError::Infrastructure(src) => Self::Infrastructure(src.to_string()),
        }
    }
}

impl From<EventError> for SettingsDomainError {
    fn from(e: EventError) -> Self {
        Self::Infrastructure(e.to_string())
    }
}
