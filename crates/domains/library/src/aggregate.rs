//! # Library aggregate roots
//!
//! The headline 6 aggregates per the spec at
//! `docs/specs/library/aggregates.md` (extended with `BookReturn`
//! and `Fine` as first-class roots to satisfy the prompt's
//! "6 headline aggregates" requirement):
//!
//! - `Book` — book master record (with `AvailableCopies` derived
//!   from `Quantity` minus the sum of open-issue quantities).
//! - `BookCategory` — category catalog.
//! - `LibraryMember` — registered borrower (student or staff).
//! - `BookIssue` — an issue of a book to a member, with status
//!   state machine `Issued → Renewed → Returned` (or `Overdue` as
//!   a derived state, or `Lost` as a terminal state).
//! - `BookReturn` — a historical log of a return action (an
//!   append-only record; the `BookIssue` keeps the canonical
//!   `IssueStatus = Returned`).
//! - `Fine` — a calculated or waived fine, attached to a
//!   `BookIssue`.
//!
//! Plus three aggregate stubs added in Cluster C microtask
//! (library/aggregate):
//!
//! - `BookAcquisition` — a single procurement event for a book.
//! - `BookCatalogEntry` — a versioned view of a book's
//!   cataloguing metadata.
//! - `LibraryMemberNote` — a free-text administrative note
//!   about a library member.
//!
//! These mirror the child entities in
//! [`crate::entities`] but are first-class aggregate roots
//! (typed id + 10-field audit footer) so they can be served
//! and queried as standalone records.
//!
//! Every aggregate follows the standard audit-footer pattern
//! (per `AGENTS.md`):
//!
//! - 1 typed id (e.g. `BookId`) + 1 derived `school_id` anchor
//! - domain fields
//! - audit-metadata fields: `version`, `etag`, `created_at`,
//!   `updated_at`, `created_by`, `updated_by`, `active_status`,
//!   `last_event_id`, `correlation_id`
//!
//! `school_id` is **derived from `id.school_id()`**, never taken
//! from the caller.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::{
    AcademicYearId, Author, BookAcquisitionId, BookCatalogEntryId, BookCategoryId, BookId,
    BookIssueFineId, BookIssueId, BookIssueRenewalId, BookNumber, BookPrice, BookReturnId,
    BookStatus, BookTitle, CategoryName, Details, DueDate, Edition, FineAmount, FineId, FinePerDay,
    FineReason, GivenDate, Isbn, IssueNote, IssueQuantity, IssueStatus, LibraryMemberId,
    LibraryMemberNoteId, MemberStatus, MemberUdId, RackNumber, ReturnDate, RoleId, StaffId,
    StockAdjustmentReason, StockCopies, StudentId, SubjectId,
};

use crate::value_objects::MemberId;

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// BookCategory
// =============================================================================

/// A book category (e.g. "Fiction", "Reference", "Textbook").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCategory {
    /// The typed id.
    pub id: BookCategoryId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The category name (unique within a school).
    pub category_name: CategoryName,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookCategory {
    /// Constructs a new `BookCategory` in the initial state.
    pub fn fresh(
        id: BookCategoryId,
        category_name: CategoryName,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            category_name,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the category name. Bumps the version.
    pub fn rename(
        &mut self,
        new_name: CategoryName,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.category_name = new_name;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the category as deleted. Bumps the version.
    pub fn delete(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// Book
// =============================================================================

/// A book master record. Carries bibliographic metadata,
/// cataloguing metadata, acquisition metadata, and stock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    /// The typed id.
    pub id: BookId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this book is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The title (1..=200 chars).
    pub book_title: BookTitle,
    /// The optional cataloguing number (unique within a school).
    pub book_number: Option<BookNumber>,
    /// The optional ISBN-10 or ISBN-13.
    pub isbn_no: Option<Isbn>,
    /// The optional author.
    pub author_name: Option<Author>,
    /// The optional publisher.
    pub publisher_name: Option<String>,
    /// The optional edition label.
    pub edition: Option<Edition>,
    /// The optional rack location.
    pub rack_number: Option<RackNumber>,
    /// The total number of physical copies held by the library.
    pub quantity: StockCopies,
    /// The optional acquisition price per copy.
    pub book_price: Option<BookPrice>,
    /// The optional post date (the date the book was added).
    pub post_date: Option<NaiveDate>,
    /// Optional details.
    pub details: Option<Details>,
    /// The category this book belongs to.
    pub book_category_id: BookCategoryId,
    /// The optional subject (from the academic domain) for
    /// cross-listing.
    pub book_subject_id: Option<SubjectId>,
    /// The current cataloging state.
    pub status: BookStatus,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Book {
    /// Constructs a new `Book` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BookId,
        academic_year_id: AcademicYearId,
        book_title: BookTitle,
        book_number: Option<BookNumber>,
        isbn_no: Option<Isbn>,
        author_name: Option<Author>,
        publisher_name: Option<String>,
        edition: Option<Edition>,
        rack_number: Option<RackNumber>,
        quantity: StockCopies,
        book_price: Option<BookPrice>,
        post_date: Option<NaiveDate>,
        details: Option<Details>,
        book_category_id: BookCategoryId,
        book_subject_id: Option<SubjectId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            book_title,
            book_number,
            isbn_no,
            author_name,
            publisher_name: publisher_name.map(|p| p.as_str().to_owned()),
            edition,
            rack_number,
            quantity,
            book_price,
            post_date,
            details,
            book_category_id,
            book_subject_id,
            status: BookStatus::Catalogued,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the book metadata. Bumps the version.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        book_title: Option<BookTitle>,
        author_name: Option<Option<Author>>,
        publisher_name: Option<Option<String>>,
        rack_number: Option<Option<RackNumber>>,
        book_price: Option<Option<BookPrice>>,
        details: Option<Option<Details>>,
        book_category_id: Option<BookCategoryId>,
        book_subject_id: Option<Option<SubjectId>>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(t) = book_title {
            self.book_title = t;
            changes.push("book_title");
        }
        if let Some(a) = author_name {
            self.author_name = a;
            changes.push("author_name");
        }
        if let Some(p) = publisher_name {
            self.publisher_name = p;
            changes.push("publisher_name");
        }
        if let Some(r) = rack_number {
            self.rack_number = r;
            changes.push("rack_number");
        }
        if let Some(p) = book_price {
            self.book_price = p;
            changes.push("book_price");
        }
        if let Some(d) = details {
            self.details = d;
            changes.push("details");
        }
        if let Some(c) = book_category_id {
            self.book_category_id = c;
            changes.push("book_category_id");
        }
        if let Some(s) = book_subject_id {
            self.book_subject_id = s;
            changes.push("book_subject_id");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Adjusts the stock count. Bumps the version.
    pub fn adjust_quantity(
        &mut self,
        new_quantity: StockCopies,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.quantity = new_quantity;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the book as deleted (retired). Bumps the version.
    /// Mirrors [`BookCategory::delete`].
    pub fn delete(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Computes the available copies given a sum of open-issue
    /// quantities. Returns 0 if the open-issue sum exceeds
    /// `quantity` (which is a data-integrity violation).
    #[must_use]
    pub fn available_copies(&self, open_issue_quantity_sum: u32) -> u32 {
        self.quantity
            .value()
            .saturating_sub(open_issue_quantity_sum)
    }
}

// =============================================================================
// LibraryMember
// =============================================================================

/// A registered library borrower (a student or a staff member).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMember {
    /// The typed id.
    pub id: LibraryMemberId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year the member is registered in.
    pub academic_year_id: AcademicYearId,
    /// The underlying platform id (Student or Staff).
    pub member: MemberId,
    /// The role id (a `RoleId` from the RBAC catalog).
    pub member_type: RoleId,
    /// The user's external id (e.g. admission number, staff id).
    pub member_ud_id: MemberUdId,
    /// The current member status.
    pub status: MemberStatus,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LibraryMember {
    /// Constructs a new `LibraryMember` in the initial state.
    pub fn fresh(
        id: LibraryMemberId,
        academic_year_id: AcademicYearId,
        member: MemberId,
        member_type: RoleId,
        member_ud_id: MemberUdId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            member,
            member_type,
            member_ud_id,
            status: MemberStatus::Active,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the member. Bumps the version. Returns the
    /// list of fields that actually changed (drives the
    /// `LibraryMemberUpdated::changes` event payload).
    pub fn update(
        &mut self,
        member_ud_id: Option<MemberUdId>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(new_ud_id) = member_ud_id {
            self.member_ud_id = new_ud_id;
            changes.push("member_ud_id");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Deactivates the member. Bumps the version.
    pub fn deactivate(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = MemberStatus::Inactive;
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Reactivates the member. Bumps the version.
    pub fn reactivate(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = MemberStatus::Active;
        self.active_status = ActiveStatus::Active;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the member as deleted (retired). Bumps the
    /// version. Mirrors [`BookCategory::delete`].
    pub fn delete(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = MemberStatus::Inactive;
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// BookIssue
// =============================================================================

/// An issue of a book to a library member. The aggregate owns
/// the issue lifecycle (issue, renew, return, mark lost). The
/// `BookReturn` and `Fine` aggregates are append-only history
/// rows; the `BookIssue` is the canonical state machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssue {
    /// The typed id.
    pub id: BookIssueId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year this issue is scoped to.
    pub academic_year_id: AcademicYearId,
    /// The book being issued.
    pub book_id: BookId,
    /// The library member holding the book.
    pub library_member_id: LibraryMemberId,
    /// The number of copies in this issue.
    pub quantity: IssueQuantity,
    /// The given date.
    pub given_date: GivenDate,
    /// The current due date.
    pub due_date: DueDate,
    /// The current issue status.
    pub issue_status: IssueStatus,
    /// The optional note (issue, return, or renewal note).
    pub note: Option<IssueNote>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookIssue {
    /// Constructs a new `BookIssue` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BookIssueId,
        academic_year_id: AcademicYearId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        quantity: IssueQuantity,
        given_date: GivenDate,
        due_date: DueDate,
        note: Option<IssueNote>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            academic_year_id,
            book_id,
            library_member_id,
            quantity,
            given_date,
            due_date,
            issue_status: IssueStatus::Issued,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Renews the issue. Sets status to `Renewed` and updates
    /// the due date.
    pub fn renew(
        &mut self,
        new_due_date: DueDate,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.due_date = new_due_date;
        self.issue_status = IssueStatus::Renewed;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the issue as returned. Terminal state.
    pub fn mark_returned(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.issue_status = IssueStatus::Returned;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the issue as lost. Terminal state.
    pub fn mark_lost(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.issue_status = IssueStatus::Lost;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Returns true if the issue is in an open status (Issued,
    /// Renewed, or Overdue — i.e. not yet Returned or Lost).
    #[must_use]
    pub fn is_open(&self) -> bool {
        matches!(
            self.issue_status,
            IssueStatus::Issued | IssueStatus::Renewed | IssueStatus::Overdue
        )
    }

    /// Returns true if the issue is overdue as of `as_of` (open
    /// and past the due date).
    #[must_use]
    pub fn is_overdue_as_of(&self, as_of: NaiveDate) -> bool {
        self.is_open() && self.due_date.value() < as_of
    }
}

// =============================================================================
// BookReturn
// =============================================================================

/// A historical record of a book return. Append-only; the
/// `BookIssue` is the canonical state. The `BookReturn` aggregate
/// records who/when/how the book was returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookReturn {
    /// The typed id.
    pub id: BookReturnId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The book issue being returned.
    pub book_issue_id: BookIssueId,
    /// The book being returned.
    pub book_id: BookId,
    /// The library member returning the book.
    pub library_member_id: LibraryMemberId,
    /// The quantity returned.
    pub quantity: IssueQuantity,
    /// The date of the return.
    pub return_date: ReturnDate,
    /// The optional note.
    pub note: Option<IssueNote>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookReturn {
    /// Constructs a new `BookReturn` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BookReturnId,
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        quantity: IssueQuantity,
        return_date: ReturnDate,
        note: Option<IssueNote>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_issue_id,
            book_id,
            library_member_id,
            quantity,
            return_date,
            note,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// Fine
// =============================================================================

/// A fine attached to a `BookIssue`. The fine may be open (not
/// yet waived or paid) or waived. A `BookIssue` may have at most
/// one open fine at a time; previous fines remain as history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fine {
    /// The typed id.
    pub id: FineId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The book issue the fine is attached to.
    pub book_issue_id: BookIssueId,
    /// The book the fine is for.
    pub book_id: BookId,
    /// The library member responsible for the fine.
    pub library_member_id: LibraryMemberId,
    /// The number of days the book was overdue (zero for
    /// replacement-cost or manual fines).
    pub days_overdue: u32,
    /// The per-day rate that was applied.
    pub per_day_rate: FinePerDay,
    /// The amount of the fine (non-negative `Decimal`).
    pub amount: FineAmount,
    /// The reason the fine was calculated.
    pub reason: FineReason,
    /// `true` if the fine has been waived.
    pub waived: bool,
    /// The optional user id who waived the fine.
    pub waived_by: Option<UserId>,
    /// The optional reason for the waiver.
    pub waived_reason: Option<String>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Fine {
    /// Constructs a new `Fine` in the initial (open) state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: FineId,
        book_issue_id: BookIssueId,
        book_id: BookId,
        library_member_id: LibraryMemberId,
        days_overdue: u32,
        per_day_rate: FinePerDay,
        amount: FineAmount,
        reason: FineReason,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_issue_id,
            book_id,
            library_member_id,
            days_overdue,
            per_day_rate,
            amount,
            reason,
            waived: false,
            waived_by: None,
            waived_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Waives the fine. Records the actor and the reason.
    pub fn waive(&mut self, by: UserId, reason: String, at: Timestamp, event_id: EventId) {
        self.waived = true;
        self.waived_by = Some(by);
        self.waived_reason = Some(reason);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// BookAcquisition (aggregate stub)
// =============================================================================

/// A single procurement event for a book (aggregate root
/// stub). Carries `Vendor`, `InvoiceNumber`, `UnitCost`,
/// `Quantity`, and `AcquiredAt`. The sum of acquisitions for a
/// book is the total cost basis.
///
/// See [`crate::entities::BookAcquisition`] for the
/// append-only child-entity projection of this aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookAcquisition {
    /// The typed id.
    pub id: BookAcquisitionId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The book being acquired.
    pub book_id: BookId,
    /// The vendor name (free-text).
    pub vendor: String,
    /// The optional invoice number.
    pub invoice_number: Option<String>,
    /// The unit cost per copy.
    pub unit_cost: BookPrice,
    /// The quantity acquired.
    pub quantity: u32,
    /// The optional acquisition date.
    pub acquired_at: Option<NaiveDate>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookAcquisition {
    /// Constructs a new `BookAcquisition` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BookAcquisitionId,
        book_id: BookId,
        vendor: String,
        invoice_number: Option<String>,
        unit_cost: BookPrice,
        quantity: u32,
        acquired_at: Option<NaiveDate>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_id,
            vendor,
            invoice_number,
            unit_cost,
            quantity,
            acquired_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// BookCatalogEntry (aggregate stub)
// =============================================================================

/// A versioned view of a book's cataloguing metadata
/// (aggregate root stub). A new entry is appended whenever
/// `AddBook` or `UpdateBook` is issued. The current state is
/// the latest entry; history is the full log.
///
/// See [`crate::entities::BookCatalogEntry`] for the
/// append-only child-entity projection of this aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCatalogEntry {
    /// The typed id.
    pub id: BookCatalogEntryId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The book this entry belongs to.
    pub book_id: BookId,
    /// The entry sequence number (1-indexed, monotonically
    /// increasing per book).
    pub sequence: u32,
    /// The optional ISBN at the time of the entry.
    pub isbn_no: Option<Isbn>,
    /// The optional book number at the time of the entry.
    pub book_number: Option<BookNumber>,
    /// The book title at the time of the entry.
    pub book_title: BookTitle,
    /// The optional author at the time of the entry.
    pub author_name: Option<Author>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookCatalogEntry {
    /// Constructs a new `BookCatalogEntry` in the initial state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: BookCatalogEntryId,
        book_id: BookId,
        sequence: u32,
        isbn_no: Option<Isbn>,
        book_number: Option<BookNumber>,
        book_title: BookTitle,
        author_name: Option<Author>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_id,
            sequence,
            isbn_no,
            book_number,
            book_title,
            author_name,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// LibraryMemberNote (aggregate stub)
// =============================================================================

/// A free-text administrative note about a library member
/// (aggregate root stub). Captures overdue patterns, lost-book
/// flags, account holds, and other staff observations.
///
/// See [`crate::entities::LibraryMemberNote`] for the
/// append-only child-entity projection of this aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberNote {
    /// The typed id.
    pub id: LibraryMemberNoteId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The library member this note is about.
    pub library_member_id: LibraryMemberId,
    /// The note's local sequence (1-indexed, monotonically
    /// increasing per member).
    pub sequence: u32,
    /// The author of the note.
    pub author: UserId,
    /// The note body (free-text).
    pub body: String,
    /// Whether the note is visible to the member.
    pub visible_to_member: bool,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LibraryMemberNote {
    /// Constructs a new `LibraryMemberNote` in the initial state.
    pub fn fresh(
        id: LibraryMemberNoteId,
        library_member_id: LibraryMemberId,
        sequence: u32,
        author: UserId,
        body: String,
        visible_to_member: bool,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            library_member_id,
            sequence,
            author,
            body,
            visible_to_member,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// Suppress unused-import warnings for items that are part of the
// public type surface but aren't used by every aggregate in this
// file (e.g. `StudentId`, `StaffId` are used through `MemberId`).
#[allow(dead_code)]
fn _unused_imports(
    _: StudentId,
    _: StaffId,
    _: FinePerDay,
    _: FineAmount,
    _: FineReason,
    _: BookStatus,
    _: StockAdjustmentReason,
    _: MemberStatus,
    _: IssueStatus,
    _: Decimal,
) {
}
