//! # Library domain events
//!
//! Every aggregate's state change emits an event implementing
//! [`DomainEvent`](::educore_events::domain_event::DomainEvent).
//! The full set follows the spec at
//! `docs/specs/library/events.md` (extended with the
//! `FineWaived` event from the spec's `permissions.md` § "Fine
//! Waiver").
//!
//! Wire form: `library.<aggregate>.<verb>` (e.g.
//! `library.book.added`, `library.book_issue.issued`).
//!
//! Phase 9 ships the headline 18 events that cover the 6
//! root aggregates plus the per-line child events.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::value_objects::MemberId;
use crate::value_objects::{
    Author, BookAcquisitionId, BookCatalogEntryId, BookCategoryId, BookId, BookIssueFineId,
    BookIssueId, BookIssueRenewalId, BookNumber, BookReturnId, BookTitle, CategoryName, DueDate,
    Edition, FineAmount, FineId, FinePerDay, FineReason, GivenDate, Isbn, IssueNote,
    IssueQuantity, LibraryMemberId, LibraryMemberNoteId, RackNumber, ReturnDate, RoleId,
    StockAdjustmentReason, StockCopies, SubjectId,
};

// =============================================================================
// BookCategory events
// =============================================================================

/// Emitted when a new `BookCategory` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCategoryCreated {
    pub book_category_id: BookCategoryId,
    pub category_name: CategoryName,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookCategoryCreated {
    /// Constructs a new `BookCategoryCreated` event.
    pub fn new(
        book_category_id: BookCategoryId,
        category_name: CategoryName,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_category_id,
            category_name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookCategoryCreated {
    const EVENT_TYPE: &'static str = "library.book_category.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_category_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BookCategory` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCategoryUpdated {
    pub book_category_id: BookCategoryId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookCategoryUpdated {
    /// Constructs a new `BookCategoryUpdated` event.
    pub fn new(
        book_category_id: BookCategoryId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_category_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookCategoryUpdated {
    const EVENT_TYPE: &'static str = "library.book_category.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_category_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BookCategory` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCategoryDeleted {
    pub book_category_id: BookCategoryId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookCategoryDeleted {
    /// Constructs a new `BookCategoryDeleted` event.
    pub fn new(
        book_category_id: BookCategoryId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_category_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookCategoryDeleted {
    const EVENT_TYPE: &'static str = "library.book_category.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_category_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Book events
// =============================================================================

/// Emitted when a new `Book` is added to the catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookAdded {
    pub book_id: BookId,
    pub book_title: BookTitle,
    pub book_number: Option<BookNumber>,
    pub isbn_no: Option<Isbn>,
    pub author_name: Option<Author>,
    pub rack_number: Option<RackNumber>,
    pub quantity: StockCopies,
    pub book_category_id: BookCategoryId,
    pub book_subject_id: Option<SubjectId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookAdded {
    /// Constructs a new `BookAdded` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        book_id: BookId,
        book_title: BookTitle,
        book_number: Option<BookNumber>,
        isbn_no: Option<Isbn>,
        author_name: Option<Author>,
        rack_number: Option<RackNumber>,
        quantity: StockCopies,
        book_category_id: BookCategoryId,
        book_subject_id: Option<SubjectId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_id,
            book_title,
            book_number,
            isbn_no,
            author_name,
            rack_number,
            quantity,
            book_category_id,
            book_subject_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookAdded {
    const EVENT_TYPE: &'static str = "library.book.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Book` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookUpdated {
    pub book_id: BookId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookUpdated {
    /// Constructs a new `BookUpdated` event.
    pub fn new(
        book_id: BookId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookUpdated {
    const EVENT_TYPE: &'static str = "library.book.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Book` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookDeleted {
    pub book_id: BookId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookDeleted {
    /// Constructs a new `BookDeleted` event.
    pub fn new(
        book_id: BookId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookDeleted {
    const EVENT_TYPE: &'static str = "library.book.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Book`'s stock quantity is adjusted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookQuantityAdjusted {
    pub book_id: BookId,
    pub from_quantity: StockCopies,
    pub to_quantity: StockCopies,
    pub reason: StockAdjustmentReason,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookQuantityAdjusted {
    /// Constructs a new `BookQuantityAdjusted` event.
    pub fn new(
        book_id: BookId,
        from_quantity: StockCopies,
        to_quantity: StockCopies,
        reason: StockAdjustmentReason,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_id,
            from_quantity,
            to_quantity,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookQuantityAdjusted {
    const EVENT_TYPE: &'static str = "library.book.quantity_adjusted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// LibraryMember events
// =============================================================================

/// Emitted when a new `LibraryMember` is registered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberRegistered {
    pub library_member_id: LibraryMemberId,
    pub member: MemberId,
    pub member_type: RoleId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberRegistered {
    /// Constructs a new `LibraryMemberRegistered` event.
    pub fn new(
        library_member_id: LibraryMemberId,
        member: MemberId,
        member_type: RoleId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            library_member_id,
            member,
            member_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberRegistered {
    const EVENT_TYPE: &'static str = "library.member.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.library_member_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.library_member_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LibraryMember` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberUpdated {
    pub library_member_id: LibraryMemberId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberUpdated {
    /// Constructs a new `LibraryMemberUpdated` event.
    pub fn new(
        library_member_id: LibraryMemberId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            library_member_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberUpdated {
    const EVENT_TYPE: &'static str = "library.member.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.library_member_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.library_member_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LibraryMember` is deactivated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberDeactivated {
    pub library_member_id: LibraryMemberId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberDeactivated {
    /// Constructs a new `LibraryMemberDeactivated` event.
    pub fn new(
        library_member_id: LibraryMemberId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            library_member_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberDeactivated {
    const EVENT_TYPE: &'static str = "library.member.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.library_member_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.library_member_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LibraryMember` is reactivated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberReactivated {
    pub library_member_id: LibraryMemberId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberReactivated {
    /// Constructs a new `LibraryMemberReactivated` event.
    pub fn new(
        library_member_id: LibraryMemberId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            library_member_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberReactivated {
    const EVENT_TYPE: &'static str = "library.member.reactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.library_member_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.library_member_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LibraryMember` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberDeleted {
    pub library_member_id: LibraryMemberId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberDeleted {
    /// Constructs a new `LibraryMemberDeleted` event.
    pub fn new(
        library_member_id: LibraryMemberId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            library_member_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberDeleted {
    const EVENT_TYPE: &'static str = "library.member.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.library_member_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.library_member_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BookIssue events
// =============================================================================

/// Emitted when a `BookIssue` is created (a book is issued to a member).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssued {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub quantity: IssueQuantity,
    pub given_date: GivenDate,
    pub due_date: DueDate,
    pub note: Option<IssueNote>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookIssued {
    /// Constructs a new `BookIssued` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        quantity: IssueQuantity,
        given_date: GivenDate,
        due_date: DueDate,
        note: Option<IssueNote>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_issue_id,
            book_id,
            library_member_id,
            quantity,
            given_date,
            due_date,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookIssued {
    const EVENT_TYPE: &'static str = "library.book_issue.issued";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BookIssue` is returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookReturned {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub return_date: ReturnDate,
    pub note: Option<IssueNote>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookReturned {
    /// Constructs a new `BookReturned` event.
    pub fn new(
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        return_date: ReturnDate,
        note: Option<IssueNote>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_issue_id,
            book_id,
            library_member_id,
            return_date,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookReturned {
    const EVENT_TYPE: &'static str = "library.book_issue.returned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BookIssue` is renewed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookRenewed {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub from_due_date: DueDate,
    pub to_due_date: DueDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookRenewed {
    /// Constructs a new `BookRenewed` event.
    pub fn new(
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        from_due_date: DueDate,
        to_due_date: DueDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_issue_id,
            book_id,
            library_member_id,
            from_due_date,
            to_due_date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookRenewed {
    const EVENT_TYPE: &'static str = "library.book_issue.renewed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BookIssue` is marked as lost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookMarkedLost {
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub quantity: IssueQuantity,
    pub note: Option<IssueNote>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookMarkedLost {
    /// Constructs a new `BookMarkedLost` event.
    pub fn new(
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        quantity: IssueQuantity,
        note: Option<IssueNote>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_issue_id,
            book_id,
            library_member_id,
            quantity,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookMarkedLost {
    const EVENT_TYPE: &'static str = "library.book_issue.marked_lost";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_issue_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_issue_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// BookReturn events
// =============================================================================

/// Emitted when a `BookReturn` aggregate is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookReturnRecorded {
    pub book_return_id: BookReturnId,
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub return_date: ReturnDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookReturnRecorded {
    /// Constructs a new `BookReturnRecorded` event.
    pub fn new(
        book_return_id: BookReturnId,
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        return_date: ReturnDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            book_return_id,
            book_issue_id,
            book_id,
            library_member_id,
            return_date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookReturnRecorded {
    const EVENT_TYPE: &'static str = "library.book_return.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_return";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.book_return_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.book_return_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Fine events
// =============================================================================

/// Emitted when a `Fine` is calculated for a `BookIssue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FineCalculated {
    pub fine_id: FineId,
    pub book_issue_id: BookIssueId,
    pub book_id: BookId,
    pub library_member_id: LibraryMemberId,
    pub days_overdue: u32,
    pub per_day_rate: FinePerDay,
    pub amount: FineAmount,
    pub reason: FineReason,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FineCalculated {
    /// Constructs a new `FineCalculated` event.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        fine_id: FineId,
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        days_overdue: u32,
        per_day_rate: FinePerDay,
        amount: FineAmount,
        reason: FineReason,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fine_id,
            book_issue_id,
            book_id,
            library_member_id,
            days_overdue,
            per_day_rate,
            amount,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FineCalculated {
    const EVENT_TYPE: &'static str = "library.fine.calculated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fine";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Fine` is waived by a privileged actor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FineWaived {
    pub fine_id: FineId,
    pub book_issue_id: BookIssueId,
    pub library_member_id: LibraryMemberId,
    pub waived_by: UserId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FineWaived {
    /// Constructs a new `FineWaived` event.
    pub fn new(
        fine_id: FineId,
        book_issue_id: BookIssueId,
        library_member_id: LibraryMemberId,
        waived_by: UserId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            fine_id,
            book_issue_id,
            library_member_id,
            waived_by,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FineWaived {
    const EVENT_TYPE: &'static str = "library.fine.waived";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "fine";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.fine_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.fine_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Cluster C: minimal event stubs (id + school_id + aggregate_id)
//
// Each event stub mirrors a child entity typed id added in commit
// c8a29a1 (`Cluster C (library): add missing ID types to
// value_objects`). They carry only the typed id, the derived
// `school_id` anchor, the aggregate_id pointer, and the standard
// envelope metadata (`event_id`, `correlation_id`, `occurred_at`).
// The full payload (changed fields, actor, reason, ...) is left for
// the owning Workstream to fill in. These stubs exist so downstream
// code (subscribers, projection rebuilders, integration tests) can
// wire type-safe handles to the owning Workstream's event shape
// without forcing an all-at-once refactor.
//
// `school_id` is derived from `id.school_id()`, never taken from
// the caller, matching the engine's tenant-anchor invariant.
// =============================================================================

/// Emitted when a new [`BookCatalogEntry`](crate::entities::BookCatalogEntry)
/// is appended to a book's cataloguing history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCatalogEntryAppended {
    pub id: BookCatalogEntryId,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookCatalogEntryAppended {
    /// Constructs a new `BookCatalogEntryAppended` stub.
    pub fn new(
        id: BookCatalogEntryId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: id.school_id(),
            aggregate_id: id.as_uuid(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookCatalogEntryAppended {
    const EVENT_TYPE: &'static str = "library.book_catalog_entry.appended";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_catalog_entry";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`BookAcquisition`](crate::entities::BookAcquisition)
/// procurement row is recorded against a book.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookAcquisitionRecorded {
    pub id: BookAcquisitionId,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookAcquisitionRecorded {
    /// Constructs a new `BookAcquisitionRecorded` stub.
    pub fn new(
        id: BookAcquisitionId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: id.school_id(),
            aggregate_id: id.as_uuid(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookAcquisitionRecorded {
    const EVENT_TYPE: &'static str = "library.book_acquisition.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_acquisition";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`LibraryMemberNote`](crate::entities::LibraryMemberNote)
/// is attached to a library member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberNoteAdded {
    pub id: LibraryMemberNoteId,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LibraryMemberNoteAdded {
    /// Constructs a new `LibraryMemberNoteAdded` stub.
    pub fn new(
        id: LibraryMemberNoteId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: id.school_id(),
            aggregate_id: id.as_uuid(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LibraryMemberNoteAdded {
    const EVENT_TYPE: &'static str = "library.member_note.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "library_member_note";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`BookIssueRenewal`](crate::entities::BookIssueRenewal)
/// history row is appended for a `BookIssue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssueRenewalAppended {
    pub id: BookIssueRenewalId,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookIssueRenewalAppended {
    /// Constructs a new `BookIssueRenewalAppended` stub.
    pub fn new(
        id: BookIssueRenewalId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: id.school_id(),
            aggregate_id: id.as_uuid(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookIssueRenewalAppended {
    const EVENT_TYPE: &'static str = "library.book_issue_renewal.appended";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue_renewal";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`BookIssueFine`](crate::entities::BookIssueFine)
/// history row is appended for a `BookIssue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssueFineAppended {
    pub id: BookIssueFineId,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BookIssueFineAppended {
    /// Constructs a new `BookIssueFineAppended` stub.
    pub fn new(
        id: BookIssueFineId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: id.school_id(),
            aggregate_id: id.as_uuid(),
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BookIssueFineAppended {
    const EVENT_TYPE: &'static str = "library.book_issue_fine.appended";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "book_issue_fine";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
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
    use educore_core::clock::{IdGenerator, SystemIdGen};

    #[test]
    fn every_event_type_is_stable() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let eid = g.next_event_id();
        let corr = g.next_correlation_id();
        let at = Timestamp::now();

        let cat_id = BookCategoryId::new(s, g.next_uuid());
        let ev =
            BookCategoryCreated::new(cat_id, CategoryName::new("Fiction").unwrap(), eid, corr, at);
        assert_eq!(
            <BookCategoryCreated as DomainEvent>::EVENT_TYPE,
            "library.book_category.created"
        );

        let book_id = BookId::new(s, g.next_uuid());
        let ev = BookAdded::new(
            book_id,
            BookTitle::new("Test").unwrap(),
            None,
            None,
            None,
            None,
            StockCopies(10),
            cat_id,
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(<BookAdded as DomainEvent>::EVENT_TYPE, "library.book.added");

        let member_id = LibraryMemberId::new(s, g.next_uuid());
        let ev = LibraryMemberRegistered::new(
            member_id,
            MemberId::Student(crate::value_objects::StudentId::new(s, g.next_uuid())),
            RoleId::new(s, g.next_uuid()),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <LibraryMemberRegistered as DomainEvent>::EVENT_TYPE,
            "library.member.registered"
        );

        let issue_id = BookIssueId::new(s, g.next_uuid());
        let ev = BookIssued::new(
            issue_id,
            book_id,
            member_id,
            IssueQuantity(1),
            GivenDate(NaiveDate::from_ymd_opt(2026, 6, 14).unwrap()),
            DueDate(NaiveDate::from_ymd_opt(2026, 6, 28).unwrap()),
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <BookIssued as DomainEvent>::EVENT_TYPE,
            "library.book_issue.issued"
        );

        let fine_id = FineId::new(s, g.next_uuid());
        let ev = FineCalculated::new(
            fine_id,
            issue_id,
            book_id,
            member_id,
            5,
            FinePerDay(Decimal::from(50)),
            FineAmount(Decimal::from(250)),
            FineReason::LateReturn,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <FineCalculated as DomainEvent>::EVENT_TYPE,
            "library.fine.calculated"
        );

        let _ = (ev, at);
    }
}
