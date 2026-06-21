//! The testkit error type.
//!
//! Every in-memory port impl returns
//! [`Result<T, TestkitError>`](crate::Result). The variants cover
//! every failure mode the in-memory backends can produce.
//! Adapters map their own internal errors (e.g. "row not found",
//! "tenant mismatch") into the appropriate variant.
//!
//! This is the testkit's only public error type; consumers that
//! want a structured error taxonomy can downcast
//! `Box<dyn std::error::Error>` from the engine's own
//! `DomainError` instead.

use thiserror::Error;

/// The testkit error type.
#[derive(Debug, Error)]
pub enum TestkitError {
    /// The requested entity (storage row, file, payment receipt,
    /// session, integration invocation) was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A write conflicted with an existing unique key
    /// (`(school_id, table, id)` for storage; `idempotency_key`
    /// for payment; `FileKey` for files; etc.).
    #[error("conflict: {0}")]
    Conflict(String),

    /// A request violated a tenant-isolation rule (e.g. writing
    /// to a `school_id` that does not match the `TenantContext`).
    #[error("tenant mismatch: {0}")]
    TenantMismatch(String),

    /// A required input was missing or malformed (e.g. an empty
    /// bearer token, a payment without an idempotency key, a
    /// file key with `..` traversal).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A backend invariant was violated (e.g. an outbox
    /// `mark_published` call for an unknown `event_id`).
    #[error("invariant violation: {0}")]
    InvariantViolation(String),
}

impl From<TestkitError> for educore_core::error::DomainError {
    fn from(err: TestkitError) -> Self {
        use educore_core::error::DomainError;
        match err {
            TestkitError::NotFound(m) => DomainError::not_found(m),
            TestkitError::Conflict(m) => DomainError::conflict(m),
            TestkitError::TenantMismatch(m) => DomainError::validation(m),
            TestkitError::InvalidInput(m) => DomainError::validation(m),
            TestkitError::InvariantViolation(m) => DomainError::infrastructure(InvariantError(m)),
        }
    }
}

/// Tiny `Error` wrapper used to ferry a `String` reason into
/// `DomainError::infrastructure`, whose signature requires
/// `impl std::error::Error + Send + Sync + 'static`. Private to
/// this module — never exposed beyond the `From` impl above.
#[derive(Debug)]
struct InvariantError(String);

impl std::fmt::Display for InvariantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InvariantError {}
