//! # educore-library
//!
//! Books, categories, members, issues, returns, fines, renewals.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/library/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(unused_imports)]

pub mod value_objects;

mod aggregate;
pub mod commands;
mod entities;
mod errors;
pub mod events;
pub mod query;
mod repository;
pub mod services;

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-library";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Prelude: re-export the engine-wide types the library services
// reach for, plus the headline symbols. Mirrors the
// `educore-facilities::prelude` pattern.
#[allow(missing_docs)]
pub mod prelude {
    pub use chrono::NaiveDate;
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::TenantContext;
    pub use educore_core::value_objects::Timestamp;
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    // Headline 6 aggregate roots
    pub use crate::aggregate::{Book, BookCategory, BookIssue, BookReturn, Fine, LibraryMember};

    // Headline child entities
    pub use crate::entities::{
        BookAcquisition, BookCatalogEntry, BookIssueFine, BookIssueRenewal, LibraryMemberNote,
    };

    // Headline 18 events
    pub use crate::events::{
        BookAdded, BookCategoryCreated, BookCategoryDeleted, BookCategoryUpdated, BookDeleted,
        BookIssued, BookMarkedLost, BookQuantityAdjusted, BookRenewed, BookReturnRecorded,
        BookReturned, FineCalculated, FineWaived, LibraryMemberDeactivated, LibraryMemberDeleted,
        LibraryMemberReactivated, LibraryMemberRegistered, LibraryMemberUpdated,
    };

    // 6 query stubs
    pub use crate::query::{
        BookCategoryQuery, BookIssueQuery, BookQuery, BookReturnQuery, FineQuery,
        LibraryMemberQuery,
    };

    // 6 repository ports
    pub use crate::repository::{
        BookCategoryRepository, BookIssueRepository, BookRepository, BookReturnRepository,
        FineRepository, LibraryMemberRepository,
    };

    // Service factories + service structs (the headline 6 +
    // helpers + the late-fine service)
    pub use crate::services::{
        add_book, compute_fine, create_book_category, create_book_issue, register_library_member,
        return_book, ActiveMembers, AvailableBooks, BookIssueCreated, BookIssueEligibility,
        BookRenewalEligibility, BookReturnResult, BookService, FineCalculationService,
        FineComputed,
    };

    // Command shapes
    pub use crate::commands::{
        AddBookCommand, AdjustBookQuantityCommand, CalculateFineCommand, CreateBookCategoryCommand,
        DeactivateLibraryMemberCommand, DeleteBookCategoryCommand, DeleteBookCommand,
        DeleteLibraryMemberCommand, IssueBookCommand, ListMemberIssuesCommand,
        ListOverdueIssuesCommand, MarkBookLostCommand, ReactivateLibraryMemberCommand,
        RecordBookReturnCommand, RegisterLibraryMemberCommand, RenewBookCommand, ReturnBookCommand,
        SearchBooksCommand, UpdateBookCategoryCommand, UpdateBookCommand,
        UpdateLibraryMemberCommand, WaiveBookIssueFineCommand,
    };

    // Typed ids + value objects + enums
    pub use crate::value_objects::{
        AcademicYearId, Author, BookCategoryId, BookId, BookIssueFineId, BookIssueId,
        BookIssueRenewalId, BookNumber, BookPrice, BookReturnId, BookStatus, BookTitle,
        CategoryName, DaysOverdue, Details, DueDate, Edition, FineAmount, FineId, FineKind,
        FinePerDay, FineReason, FineSettings, GivenDate, Isbn, IssueNote, IssueQuantity,
        IssueStatus, LibraryMemberId, MemberId, MemberStatus, MemberUdId, RackNumber, ReturnDate,
        RoleId, StaffId, StockAdjustmentReason, StockCopies, StudentId, SubjectId,
    };

    // Errors
    pub use crate::errors::LibraryError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-library");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_exports_expected_symbols() {
        // Smoke test: every headline aggregate and event is
        // reachable through the prelude. The compiler enforces
        // the names.
        let _: Option<Book> = None;
        let _: Option<BookCategory> = None;
        let _: Option<LibraryMember> = None;
        let _: Option<BookIssue> = None;
        let _: Option<BookReturn> = None;
        let _: Option<Fine> = None;
        let _: Option<BookCatalogEntry> = None;
        let _: Option<BookAcquisition> = None;
        let _: Option<LibraryMemberNote> = None;
        let _: Option<BookIssueRenewal> = None;
        let _: Option<BookIssueFine> = None;
    }
}
