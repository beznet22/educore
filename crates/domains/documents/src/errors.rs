//! Documents-domain error type.

use thiserror::Error;

/// Convenient `Result` alias scoped to the documents crate.
pub type Result<T> = core::result::Result<T, DocumentsError>;

/// Documents-domain error type. All fallible operations in the
/// documents crate return `Result<T, DocumentsError>`.
#[derive(Debug, Error)]
pub enum DocumentsError {
    /// Generic validation failure (e.g. malformed value object).
    #[error("validation: {0}")]
    Validation(String),

    /// A form download has neither a `link` nor a `file` set;
    /// the spec requires at least one (invariant 2).
    #[error("form has neither link nor file")]
    FormHasNoContent,

    /// No form download exists with the given id.
    #[error("form not found: {0}")]
    FormNotFound(uuid::Uuid),

    /// No postal dispatch exists with the given id.
    #[error("postal dispatch not found: {0}")]
    PostalDispatchNotFound(uuid::Uuid),

    /// No postal receive exists with the given id.
    #[error("postal receive not found: {0}")]
    PostalReceiveNotFound(uuid::Uuid),

    /// Reference number is already in use for the given school
    /// and academic year; reference numbers must be unique
    /// within that scope.
    #[error("reference number '{0}' already exists for school {1} in academic year {2}")]
    DuplicateReferenceNo(String, educore_core::ids::SchoolId, uuid::Uuid),

    /// The reference number of an existing form is immutable;
    /// a mutation attempted to change it.
    #[error("reference number is immutable once set")]
    ReferenceNoImmutable,

    /// The caller is not authorized to perform the operation.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// The operation conflicts with the current state of a
    /// domain resource (unique key, state machine, etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// An infrastructure adapter (storage, event bus, files)
    /// reported a failure.
    #[error("infrastructure: {0}")]
    Infrastructure(String),

    /// Catch-all variant for wrapped errors that do not map to a
    /// documents-specific case.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
