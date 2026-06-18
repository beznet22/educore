//! # educore-events-domain errors
//!
//! The `EventsDomainError` enum and `Result` type alias.
//! Mirrors the CMS pattern.

use thiserror::Error;

use educore_core::error::DomainError;
use educore_events::errors::EventError;

/// The error type for the events-domain crate.
#[derive(Debug, Error)]
pub enum EventsDomainError {
    /// Validation failed (empty title, invalid date range, etc.).
    #[error("validation error: {0}")]
    Validation(String),

    /// Conflict with current state (e.g. update on soft-deleted aggregate).
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

/// Result alias for the events-domain crate.
pub type Result<T> = std::result::Result<T, EventsDomainError>;

impl From<DomainError> for EventsDomainError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::Validation(s) => Self::Validation(s),
            DomainError::Conflict(s) => Self::Conflict(s),
            DomainError::Forbidden(s) => Self::Forbidden(s),
            DomainError::NotFound(s) => Self::NotFound(s),
            other => Self::Infrastructure(other.to_string()),
        }
    }
}

impl From<EventError> for EventsDomainError {
    fn from(e: EventError) -> Self {
        Self::Infrastructure(e.to_string())
    }
}
