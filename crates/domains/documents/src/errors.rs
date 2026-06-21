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

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn form_has_no_content_display_is_stable() {
        // The display form is the engine's wire form for the
        // error message. It is part of the API surface.
        let msg = DocumentsError::FormHasNoContent.to_string();
        assert_eq!(msg, "form has neither link nor file");
    }

    #[test]
    fn form_not_found_carries_uuid() {
        let id = uuid::Uuid::now_v7();
        let err = DocumentsError::FormNotFound(id);
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn postal_dispatch_not_found_carries_uuid() {
        let id = uuid::Uuid::now_v7();
        let err = DocumentsError::PostalDispatchNotFound(id);
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn postal_receive_not_found_carries_uuid() {
        let id = uuid::Uuid::now_v7();
        let err = DocumentsError::PostalReceiveNotFound(id);
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn duplicate_reference_no_carries_payload() {
        let s = educore_core::ids::SchoolId(uuid::Uuid::now_v7());
        let year = uuid::Uuid::now_v7();
        let err = DocumentsError::DuplicateReferenceNo("REF-1".to_owned(), s, year);
        let msg = err.to_string();
        assert!(msg.contains("REF-1"));
    }

    #[test]
    fn reference_no_immutable_display_is_stable() {
        let msg = DocumentsError::ReferenceNoImmutable.to_string();
        assert_eq!(msg, "reference number is immutable once set");
    }

    #[test]
    fn forbidden_carries_message() {
        let err = DocumentsError::Forbidden("nope".to_owned());
        assert_eq!(err.to_string(), "forbidden: nope");
    }

    #[test]
    fn conflict_carries_message() {
        let err = DocumentsError::Conflict("stale".to_owned());
        assert_eq!(err.to_string(), "conflict: stale");
    }

    #[test]
    fn validation_carries_message() {
        let err = DocumentsError::Validation("bad input".to_owned());
        assert_eq!(err.to_string(), "validation: bad input");
    }

    #[test]
    fn infrastructure_carries_message() {
        let err = DocumentsError::Infrastructure("bus down".to_owned());
        assert_eq!(err.to_string(), "infrastructure: bus down");
    }

    #[test]
    fn other_wraps_arbitrary_error() {
        let inner: Box<dyn std::error::Error + Send + Sync> = std::io::Error::other("disk").into();
        let err: DocumentsError = inner.into();
        assert!(matches!(err, DocumentsError::Other(_)));
    }

    #[test]
    fn result_alias_is_standard_result() {
        // The `Result` alias resolves to `core::result::Result<T,
        // DocumentsError>`. This is a compile-time check.
        let r: Result<u32> = Ok(7);
        assert!(r.is_ok());
    }
}
