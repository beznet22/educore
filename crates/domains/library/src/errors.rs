//! # Library domain errors
//!
//! The [`LibraryError`] enum is the typed error surface for the
//! library domain. The domain service functions return
//! `Result<T, DomainError>`; `LibraryError` is the library-specific
//! projection used by the dispatcher to produce wire-form errors
//! and to map to the engine's `DomainError` variants.
//!
//! Mirrors the Phase 7 finance pattern (a small enum
//! that wraps the engine's `DomainError` for library-specific
//! signals). The v1 surface is intentionally minimal — most
//! errors are domain-agnostic (`Validation`, `Conflict`,
//! `NotFound`, `Forbidden`) and are produced directly by the
//! services.

#![allow(missing_docs)]
#![allow(unused_imports)]

use thiserror::Error;

/// The library-domain error surface. Most signals are
/// `DomainError` values; the enum exists to anchor the
/// library's typed error vocabulary and to provide a stable
/// `is_*` API for the dispatcher.
#[derive(Debug, Error)]
pub enum LibraryError {
    /// A book was referenced by an issue that is still open.
    #[error("book {0} has open issues; delete is rejected")]
    BookHasOpenIssues(String),
    /// A library member has open issues; delete is rejected.
    #[error("library member {0} has open issues; delete is rejected")]
    MemberHasOpenIssues(String),
    /// A book category was referenced by a book.
    #[error("book category {0} is referenced by a book; delete is rejected")]
    CategoryReferencedByBook(String),
    /// An attempt to issue a book that is fully on loan.
    #[error("book {0} has no available copies")]
    BookOutOfStock(String),
    /// An attempt to renew an issue that is not in a renewable state.
    #[error("book issue {0} is not in a renewable state")]
    IssueNotRenewable(String),
    /// An attempt to return an issue that is already returned.
    #[error("book issue {0} is already returned")]
    IssueAlreadyReturned(String),
    /// A waiver attempt on a fine that does not exist.
    #[error("fine {0} not found")]
    FineNotFound(String),
    /// The stock adjustment would overdraw open issues.
    #[error(
        "new quantity {requested} is below the sum of open-issue quantities {on_issue} for book {book}"
    )]
    QuantityBelowOpenIssues {
        /// The book id.
        book: String,
        /// The requested new total quantity.
        requested: u32,
        /// The sum of quantities on open issues.
        on_issue: u32,
    },
}
