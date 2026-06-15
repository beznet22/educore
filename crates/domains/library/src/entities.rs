//! # Library domain entities
//!
//! Entities have identity and lifecycle but are not aggregate
//! roots. They are loaded and persisted only through their
//! aggregate root.
//!
//! The 3 headline child entities (per the spec's
//! `entities.md` + the prompt's "+3 child entities" requirement):
//!
//! - [`BookCatalogEntry`] — versioned view of a book's
//!   cataloguing metadata (appended on every `AddBook` /
//!   `UpdateBook`).
//! - [`BookAcquisition`] — single procurement event for a book
//!   (vendor, invoice, unit cost, quantity, acquired_at).
//! - [`LibraryMemberNote`] — free-text administrative note about
//!   a member.
//!
//! The two spec-mandated child entities
//! ([`BookIssueRenewal`], [`BookIssueFine`]) are also shipped.
//! They are loaded and persisted through `BookIssue` and `Fine`
//! respectively (the renewal/fine history is the canonical
//! per-issue audit trail).

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};

use crate::value_objects::MemberId;
use crate::value_objects::{
    BookId, BookIssueFineId, BookIssueId, BookIssueRenewalId, BookPrice, DueDate, FineAmount,
    FineId, FinePerDay, FineReason, IssueQuantity, LibraryMemberId, MemberUdId, RoleId,
};

// Re-export the NaiveDate alias path for entity convenience.
type Date = chrono::NaiveDate;

// =============================================================================
// BookCatalogEntry
// =============================================================================

/// A versioned view of a book's cataloguing metadata. A new entry
/// is appended whenever `AddBook` or `UpdateBook` is issued. The
/// current state is the latest entry; history is the full log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookCatalogEntry {
    /// The owning school (derived from the book id).
    pub school_id: SchoolId,
    /// The book this entry belongs to.
    pub book_id: BookId,
    /// The entry sequence number (1-indexed, monotonically
    /// increasing per book).
    pub sequence: u32,
    /// The optional ISBN at the time of the entry.
    pub isbn_no: Option<crate::value_objects::Isbn>,
    /// The optional book number at the time of the entry.
    pub book_number: Option<crate::value_objects::BookNumber>,
    /// The book title at the time of the entry.
    pub book_title: crate::value_objects::BookTitle,
    /// The optional author at the time of the entry.
    pub author_name: Option<crate::value_objects::Author>,
    /// The occurred-at timestamp.
    pub occurred_at: Timestamp,
    /// The actor id who created the entry.
    pub actor_id: UserId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl BookCatalogEntry {
    /// Constructs a new `BookCatalogEntry`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        book_id: BookId,
        sequence: u32,
        isbn_no: Option<crate::value_objects::Isbn>,
        book_number: Option<crate::value_objects::BookNumber>,
        book_title: crate::value_objects::BookTitle,
        author_name: Option<crate::value_objects::Author>,
        occurred_at: Timestamp,
        actor_id: UserId,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: book_id.school_id(),
            book_id,
            sequence,
            isbn_no,
            book_number,
            book_title,
            author_name,
            occurred_at,
            actor_id,
            correlation_id,
        }
    }
}

// =============================================================================
// BookAcquisition
// =============================================================================

/// A single procurement event for a book. Carries `Vendor`,
/// `InvoiceNumber`, `UnitCost`, `Quantity`, and `AcquiredAt`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookAcquisition {
    /// The owning school (derived from the book id).
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
    pub acquired_at: Option<Date>,
    /// Audit footer.
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl BookAcquisition {
    /// Constructs a new `BookAcquisition`.
    pub fn new(
        book_id: BookId,
        vendor: String,
        invoice_number: Option<String>,
        unit_cost: BookPrice,
        quantity: u32,
        acquired_at: Option<Date>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: book_id.school_id(),
            book_id,
            vendor,
            invoice_number,
            unit_cost,
            quantity,
            acquired_at,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// LibraryMemberNote
// =============================================================================

/// A free-text administrative note about a member (overdue
/// pattern, lost book, account hold).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryMemberNote {
    /// The owning school (derived from the member id).
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
    /// The created-at timestamp.
    pub created_at: Timestamp,
    /// Whether the note is visible to the member.
    pub visible_to_member: bool,
}

impl LibraryMemberNote {
    /// Constructs a new `LibraryMemberNote`.
    pub fn new(
        library_member_id: LibraryMemberId,
        sequence: u32,
        author: UserId,
        body: String,
        visible_to_member: bool,
        created_at: Timestamp,
    ) -> Self {
        Self {
            school_id: library_member_id.school_id(),
            library_member_id,
            sequence,
            author,
            body,
            created_at,
            visible_to_member,
        }
    }
}

// =============================================================================
// BookIssueRenewal
// =============================================================================

/// A historical record of a renewal. Has `RenewedAt`,
/// `FromDueDate`, `ToDueDate`, `RenewedBy: UserId`. The current
/// due date is the `ToDueDate` of the most recent renewal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssueRenewal {
    /// The typed id.
    pub id: BookIssueRenewalId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The book issue being renewed.
    pub book_issue_id: BookIssueId,
    /// The renewal sequence (1-indexed, monotonically increasing
    /// per issue).
    pub sequence: u32,
    /// The previous due date.
    pub from_due_date: DueDate,
    /// The new due date.
    pub to_due_date: DueDate,
    /// The actor who performed the renewal.
    pub renewed_by: UserId,
    /// The renewal timestamp.
    pub renewed_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl BookIssueRenewal {
    /// Constructs a new `BookIssueRenewal`.
    pub fn new(
        id: BookIssueRenewalId,
        book_issue_id: BookIssueId,
        sequence: u32,
        from_due_date: DueDate,
        to_due_date: DueDate,
        renewed_by: UserId,
        renewed_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_issue_id,
            sequence,
            from_due_date,
            to_due_date,
            renewed_by,
            renewed_at,
            correlation_id,
        }
    }
}

// =============================================================================
// BookIssueFine
// =============================================================================

/// A historical record of a fine attached to a `BookIssue`.
/// Has `CalculatedAt`, `DaysOverdue`, `PerDayRate`, `Amount`,
/// `Waived: bool`, `WaivedBy: Option<UserId>`, `WaivedReason`.
///
/// A `BookIssue` may have at most one open fine at a time;
/// previous fines remain as history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BookIssueFine {
    /// The typed id.
    pub id: BookIssueFineId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The book issue the fine is attached to.
    pub book_issue_id: BookIssueId,
    /// The associated fine aggregate (the canonical record).
    pub fine_id: FineId,
    /// The days overdue at the time of calculation.
    pub days_overdue: u32,
    /// The per-day rate at the time of calculation.
    pub per_day_rate: FinePerDay,
    /// The amount of the fine.
    pub amount: FineAmount,
    /// The reason the fine was calculated.
    pub reason: FineReason,
    /// `true` if the fine was waived.
    pub waived: bool,
    /// The optional user id who waived the fine.
    pub waived_by: Option<UserId>,
    /// The optional reason for the waiver.
    pub waived_reason: Option<String>,
    /// The calculated-at timestamp.
    pub calculated_at: Timestamp,
}

impl BookIssueFine {
    /// Constructs a new `BookIssueFine` from the canonical
    /// [`Fine`](crate::aggregate::Fine) record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: BookIssueFineId,
        book_issue_id: BookIssueId,
        fine_id: FineId,
        days_overdue: u32,
        per_day_rate: FinePerDay,
        amount: FineAmount,
        reason: FineReason,
        waived: bool,
        waived_by: Option<UserId>,
        waived_reason: Option<String>,
        calculated_at: Timestamp,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            book_issue_id,
            fine_id,
            days_overdue,
            per_day_rate,
            amount,
            reason,
            waived,
            waived_by,
            waived_reason,
            calculated_at,
        }
    }
}

// Suppress unused-import warnings for items referenced via the
// spec's optional metadata path.
#[allow(dead_code)]
fn _unused_imports(
    _: MemberUdId,
    _: RoleId,
    _: MemberId,
    _: IssueQuantity,
    _: BookPrice,
    _: Decimal,
) {
}
