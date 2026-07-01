//! # Library domain commands
//!
//! Every command is a typed `Cmd` struct carrying a
//! `TenantContext` and the typed id of the affected aggregate.
//! Commands are validated, authorized (via the
//! `educore-rbac::Capability` capability check at the
//! dispatcher), and dispatched to the relevant aggregate.
//!
//! Every command produces zero or more events that are recorded
//! in the event log via the bus-port contract
//! (`library.<aggregate>.<verb>`).
//!
//! Phase 9 ships ~30 typed command shapes that drive the 6
//! headline aggregates. The headline service fns (one per
//! command shape that the dispatcher routes to a service
//! factory) are re-exported from the prelude.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearId, Author, BookAcquisitionId, BookCatalogEntryId, BookCategoryId, BookId,
    BookIssueId, BookNumber, BookPrice, BookReturnId, BookTitle, CategoryName, Details, DueDate,
    Edition, FineAmount, FineId, FinePerDay, FineReason, GivenDate, Isbn, IssueNote, IssueQuantity,
    LibraryMemberId, LibraryMemberNoteId, MemberId, MemberUdId, RackNumber, ReturnDate, RoleId,
    StockAdjustmentReason, StockCopies, SubjectId,
};
use educore_rbac::value_objects::Capability;

#[allow(dead_code)]
fn event_id_to_uuid(e: educore_core::ids::EventId) -> uuid::Uuid {
    e.as_uuid()
}
#[allow(dead_code)]
fn _use_event_id_to_uuid() {
    let _ = event_id_to_uuid_marker;
}

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `library.<aggregate>.<verb>`).
// =============================================================================

/// Create-book-category command type.
pub const LIBRARY_BOOK_CATEGORY_CREATE_COMMAND_TYPE: &str = "library.book_category.create";
/// Update-book-category command type.
pub const LIBRARY_BOOK_CATEGORY_UPDATE_COMMAND_TYPE: &str = "library.book_category.update";
/// Delete-book-category command type.
pub const LIBRARY_BOOK_CATEGORY_DELETE_COMMAND_TYPE: &str = "library.book_category.delete";

/// Add-book command type.
pub const LIBRARY_BOOK_ADD_COMMAND_TYPE: &str = "library.book.add";
/// Update-book command type.
pub const LIBRARY_BOOK_UPDATE_COMMAND_TYPE: &str = "library.book.update";
/// Delete-book command type.
pub const LIBRARY_BOOK_DELETE_COMMAND_TYPE: &str = "library.book.delete";
/// Adjust-book-quantity command type.
pub const LIBRARY_BOOK_ADJUST_QUANTITY_COMMAND_TYPE: &str = "library.book.adjust_quantity";

/// Register-library-member command type.
pub const LIBRARY_MEMBER_REGISTER_COMMAND_TYPE: &str = "library.member.register";
/// Update-library-member command type.
pub const LIBRARY_MEMBER_UPDATE_COMMAND_TYPE: &str = "library.member.update";
/// Deactivate-library-member command type.
pub const LIBRARY_MEMBER_DEACTIVATE_COMMAND_TYPE: &str = "library.member.deactivate";
/// Reactivate-library-member command type.
pub const LIBRARY_MEMBER_REACTIVATE_COMMAND_TYPE: &str = "library.member.reactivate";
/// Delete-library-member command type.
pub const LIBRARY_MEMBER_DELETE_COMMAND_TYPE: &str = "library.member.delete";

/// Issue-book command type.
pub const LIBRARY_BOOK_ISSUE_ISSUE_COMMAND_TYPE: &str = "library.book_issue.issue";
/// Return-book command type.
pub const LIBRARY_BOOK_ISSUE_RETURN_COMMAND_TYPE: &str = "library.book_issue.return";
/// Renew-book command type.
pub const LIBRARY_BOOK_ISSUE_RENEW_COMMAND_TYPE: &str = "library.book_issue.renew";
/// Mark-book-lost command type.
pub const LIBRARY_BOOK_ISSUE_MARK_LOST_COMMAND_TYPE: &str = "library.book_issue.mark_lost";

/// Return-book (log row) command type.
pub const LIBRARY_BOOK_RETURN_RECORD_COMMAND_TYPE: &str = "library.book_return.record";

/// Calculate-fine command type.
pub const LIBRARY_FINE_CALCULATE_COMMAND_TYPE: &str = "library.fine.calculate";
/// Waive-fine command type.
pub const LIBRARY_FINE_WAIVE_COMMAND_TYPE: &str = "library.fine.waive";

/// Search-books read command type.
pub const LIBRARY_BOOK_SEARCH_COMMAND_TYPE: &str = "library.book.search";
/// List-overdue-issues read command type.
pub const LIBRARY_BOOK_ISSUE_LIST_OVERDUE_COMMAND_TYPE: &str = "library.book_issue.list_overdue";
/// List-member-issues read command type.
pub const LIBRARY_BOOK_ISSUE_LIST_FOR_MEMBER_COMMAND_TYPE: &str =
    "library.book_issue.list_for_member";

/// Library-read aggregate report command type.
pub const LIBRARY_REPORT_READ_COMMAND_TYPE: &str = "library.report.read";
/// Library-configure aggregate command type.
pub const LIBRARY_CONFIGURE_COMMAND_TYPE: &str = "library.configure";

// =============================================================================
// BookCategory commands
// =============================================================================

/// Create a new book category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBookCategoryCommand {
    /// Tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The category name.
    pub category_name: CategoryName,
}

impl CreateBookCategoryCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_CATEGORY_CREATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Update a book category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBookCategoryCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The category id.
    pub book_category_id: BookCategoryId,
    /// The new category name.
    pub new_name: CategoryName,
}

impl UpdateBookCategoryCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_CATEGORY_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Delete a book category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteBookCategoryCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The category id.
    pub book_category_id: BookCategoryId,
}

impl DeleteBookCategoryCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_CATEGORY_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// Book commands
// =============================================================================

/// Add a book to the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The academic year this book is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The book title.
    pub book_title: BookTitle,
    /// The optional cataloguing number.
    pub book_number: Option<BookNumber>,
    /// The optional ISBN.
    pub isbn_no: Option<Isbn>,
    /// The optional author.
    pub author_name: Option<Author>,
    /// The optional publisher.
    pub publisher_name: Option<String>,
    /// The optional edition.
    pub edition: Option<Edition>,
    /// The optional rack number.
    pub rack_number: Option<RackNumber>,
    /// The total number of physical copies.
    pub quantity: StockCopies,
    /// The optional acquisition price per copy.
    pub book_price: Option<BookPrice>,
    /// The optional post date.
    pub post_date: Option<NaiveDate>,
    /// The optional details.
    pub details: Option<Details>,
    /// The category this book belongs to.
    pub book_category_id: BookCategoryId,
    /// The optional subject.
    pub book_subject_id: Option<SubjectId>,
}

impl AddBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ADD_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Update a book's bibliographic metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book id.
    pub book_id: BookId,
    /// The new title.
    pub book_title: Option<BookTitle>,
    /// The new author.
    pub author_name: Option<Author>,
    /// The new publisher.
    pub publisher_name: Option<String>,
    /// The new rack number.
    pub rack_number: Option<RackNumber>,
    /// The new book price.
    pub book_price: Option<BookPrice>,
    /// The new details.
    pub details: Option<Details>,
    /// The new category.
    pub book_category_id: Option<BookCategoryId>,
    /// The new subject.
    pub book_subject_id: Option<SubjectId>,
}

impl UpdateBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Delete a book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book id.
    pub book_id: BookId,
}

impl DeleteBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Adjust a book's stock count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjustBookQuantityCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book id.
    pub book_id: BookId,
    /// The new total quantity.
    pub new_quantity: StockCopies,
    /// The reason for the adjustment.
    pub reason: StockAdjustmentReason,
}

impl AdjustBookQuantityCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ADJUST_QUANTITY_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// LibraryMember commands
// =============================================================================

/// Register a new library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterLibraryMemberCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The academic year this member is registered in.
    pub academic_year_id: AcademicYearId,
    /// The underlying platform id (Student or Staff).
    pub member: MemberId,
    /// The role id.
    pub member_type: RoleId,
    /// The external id (e.g. admission number, staff id).
    pub member_ud_id: MemberUdId,
}

impl RegisterLibraryMemberCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_MEMBER_REGISTER_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Update a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateLibraryMemberCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The library member id.
    pub library_member_id: LibraryMemberId,
    /// The new external id.
    pub member_ud_id: Option<MemberUdId>,
    /// The optional note to add to the member.
    pub note: Option<String>,
}

impl UpdateLibraryMemberCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_MEMBER_UPDATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Deactivate a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeactivateLibraryMemberCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The library member id.
    pub library_member_id: LibraryMemberId,
    /// The reason for deactivation.
    pub reason: String,
}

impl DeactivateLibraryMemberCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_MEMBER_DEACTIVATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Reactivate a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReactivateLibraryMemberCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The library member id.
    pub library_member_id: LibraryMemberId,
}

impl ReactivateLibraryMemberCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_MEMBER_REACTIVATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Delete a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLibraryMemberCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The library member id.
    pub library_member_id: LibraryMemberId,
}

impl DeleteLibraryMemberCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_MEMBER_DELETE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// BookIssue commands
// =============================================================================

/// Issue a book to a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The academic year this issue is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The book being issued.
    pub book_id: BookId,
    /// The library member receiving the book.
    pub library_member_id: LibraryMemberId,
    /// The number of copies.
    pub quantity: IssueQuantity,
    /// The given date.
    pub given_date: GivenDate,
    /// The due date.
    pub due_date: DueDate,
    /// The optional note.
    pub note: Option<IssueNote>,
}

impl IssueBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_ISSUE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Return a book that was issued.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReturnBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book issue id.
    pub book_issue_id: BookIssueId,
    /// The return date.
    pub return_date: ReturnDate,
    /// The optional note.
    pub note: Option<IssueNote>,
}

impl ReturnBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_RETURN_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Renew a book issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenewBookCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book issue id.
    pub book_issue_id: BookIssueId,
    /// The new due date.
    pub new_due_date: DueDate,
}

impl RenewBookCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_RENEW_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Mark a book issue as lost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkBookLostCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book issue id.
    pub book_issue_id: BookIssueId,
    /// The optional note.
    pub note: Option<IssueNote>,
}

impl MarkBookLostCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_MARK_LOST_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// BookReturn commands
// =============================================================================

/// Record a book return (creates a `BookReturn` aggregate row).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordBookReturnCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The book return id (typed id for the new `BookReturn`
    /// row).
    pub book_return_id: BookReturnId,
    /// The book issue being returned.
    pub book_issue_id: BookIssueId,
    /// The book being returned.
    pub book_id: BookId,
    /// The library member returning the book.
    pub library_member_id: LibraryMemberId,
    /// The quantity returned.
    pub quantity: IssueQuantity,
    /// The return date.
    pub return_date: ReturnDate,
    /// The optional note.
    pub note: Option<IssueNote>,
}

impl RecordBookReturnCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_RETURN_RECORD_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// Fine commands
// =============================================================================

/// Calculate a fine for a book issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculateFineCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The fine id (typed id for the new `Fine` row).
    pub fine_id: FineId,
    /// The book issue the fine is for.
    pub book_issue_id: BookIssueId,
    /// The book the fine is for.
    pub book_id: BookId,
    /// The library member responsible for the fine.
    pub library_member_id: LibraryMemberId,
    /// The as-of date for the calculation.
    pub as_of: NaiveDate,
    /// The per-day rate.
    pub per_day_rate: FinePerDay,
    /// The reason for the fine.
    pub reason: FineReason,
}

impl CalculateFineCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_FINE_CALCULATE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Waive a fine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WaiveBookIssueFineCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The fine id.
    pub fine_id: FineId,
    /// The reason for the waiver.
    pub reason: String,
}

impl WaiveBookIssueFineCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_FINE_WAIVE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

// =============================================================================
// Read commands (no event emission)
// =============================================================================

/// Search the book catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchBooksCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Free-text search query.
    pub query: String,
    /// Optional category filter.
    pub category: Option<BookCategoryId>,
    /// Optional limit on the number of rows returned.
    pub limit: u32,
}

impl SearchBooksCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_SEARCH_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryRead]
    }
}

/// List overdue book issues as of a given date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListOverdueIssuesCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// The as-of date.
    pub as_of: NaiveDate,
}

impl ListOverdueIssuesCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_LIST_OVERDUE_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryRead]
    }
}

/// List book issues for a member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMemberIssuesCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The library member id.
    pub library_member_id: LibraryMemberId,
}

impl ListMemberIssuesCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = LIBRARY_BOOK_ISSUE_LIST_FOR_MEMBER_COMMAND_TYPE;

    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryRead]
    }
}

// Suppress unused-import warnings for items that are part of
// the public command type surface but not consumed by every
// command.
#[allow(dead_code)]
fn _unused_imports(_: Timestamp, _: event_id_to_uuid_marker, _: FineAmount, _: Decimal, _: UserId) {
}

/// Phantom type to keep `event_id_to_uuid` in the import list.
#[allow(non_camel_case_types)]
struct event_id_to_uuid_marker;

// =============================================================================
// Cluster C: minimal command stubs (id + school_id)
//
// Each command struct mirrors the matching entity stub added in
// commit c8a29a1 (`Cluster C (library): add missing ID types to
// value_objects`). They carry only the typed id and the derived
// `school_id` anchor; the full payload (tenant context, payload
// fields, audit metadata) is left for the owning Workstream to
// fill in. These stubs exist so downstream code (events.rs
// subscribers, repository ports, integration tests) can wire
// type-safe handles to the owning Workstream's command shape
// without forcing an all-at-once refactor.
// =============================================================================

/// Record a single procurement event for a book (vendor,
/// invoice, unit cost, quantity, acquired_at).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateBookAcquisitionCommand {
    /// The typed id.
    pub id: BookAcquisitionId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
}

impl CreateBookAcquisitionCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Append a new entry to a book's versioned cataloguing history.
/// A new entry is appended on every `AddBook` / `UpdateBook`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppendBookCatalogEntryCommand {
    /// The typed id.
    pub id: BookCatalogEntryId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
}

impl AppendBookCatalogEntryCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Add a free-text administrative note about a member (overdue
/// pattern, lost book, account hold).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLibraryMemberNoteCommand {
    /// The typed id.
    pub id: LibraryMemberNoteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
}

impl CreateLibraryMemberNoteCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}

/// Delete a library member note (admin correction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLibraryMemberNoteCommand {
    /// The typed id.
    pub id: LibraryMemberNoteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
}

impl DeleteLibraryMemberNoteCommand {
    /// The capabilities required to dispatch this command.
    #[must_use]
    pub fn required_capabilities() -> Vec<Capability> {
        vec![Capability::LibraryConfigure]
    }
}
