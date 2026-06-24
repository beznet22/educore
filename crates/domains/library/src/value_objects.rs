//! # Library value objects
//!
//! The typed ids (one per aggregate), the validated value objects,
//! and the closed enums the library aggregates depend on. Per
//! `docs/specs/library/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Money is `Decimal` (from `rust_decimal`) per the build-plan
//!   § "Risks" + the late-fine service pattern at
//!   `crates/domains/finance/src/services.rs:1259`.
//! - Foreign-key typed ids (`StudentId`, `StaffId`, `RoleId`,
//!   `AcademicYearId`, `SubjectId`) are **re-exported** from
//!   [`educore_academic`] and [`educore_hr`]; the library crate
//!   owns only the library-specific ids.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

pub use educore_academic::AcademicYearId;
pub use educore_academic::StudentId;
pub use educore_academic::SubjectId;
pub use educore_hr::value_objects::RoleId;
pub use educore_hr::value_objects::StaffId;

// =============================================================================
// Macro: typed library id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// library id follows the same shape: a `school_id` anchor plus
/// a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
macro_rules! library_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids: 6 aggregate roots + 5 child ids
// =============================================================================

library_typed_id! {
    /// A typed id for a [`Book`](crate::aggregate::Book).
    pub struct BookId;
}
library_typed_id! {
    /// A typed id for a [`BookCategory`](crate::aggregate::BookCategory).
    pub struct BookCategoryId;
}
library_typed_id! {
    /// A typed id for a [`LibraryMember`](crate::aggregate::LibraryMember).
    pub struct LibraryMemberId;
}
library_typed_id! {
    /// A typed id for a [`BookIssue`](crate::aggregate::BookIssue).
    pub struct BookIssueId;
}
library_typed_id! {
    /// A typed id for a [`BookReturn`](crate::aggregate::BookReturn).
    pub struct BookReturnId;
}
library_typed_id! {
    /// A typed id for a [`Fine`](crate::aggregate::Fine).
    pub struct FineId;
}
library_typed_id! {
    /// A typed id for a [`BookIssueRenewal`](crate::entities::BookIssueRenewal).
    pub struct BookIssueRenewalId;
}
library_typed_id! {
    /// A typed id for a [`BookIssueFine`](crate::entities::BookIssueFine).
    pub struct BookIssueFineId;
}
library_typed_id! {
    /// A typed id for a [`BookAcquisition`](crate::entities::BookAcquisition).
    pub struct BookAcquisitionId;
}
library_typed_id! {
    /// A typed id for a [`BookCatalogEntry`](crate::entities::BookCatalogEntry).
    pub struct BookCatalogEntryId;
}
library_typed_id! {
    /// A typed id for a [`LibraryMemberNote`](crate::entities::LibraryMemberNote).
    pub struct LibraryMemberNoteId;
}

// =============================================================================
// Bibliographic value objects
// =============================================================================

/// An ISBN-10 or ISBN-13 with optional hyphens. Validated by
/// checksum on construction. `Isbn` is unique within a school.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Isbn(String);

impl Isbn {
    /// Parses an ISBN string with optional hyphens.
    pub fn parse(raw: &str) -> Result<Self> {
        let stripped: String = raw.chars().filter(|c| *c != '-' && *c != ' ').collect();
        match stripped.len() {
            10 => {
                if !stripped[..9].chars().all(|c| c.is_ascii_digit())
                    || !(stripped.chars().nth(9).is_some_and(|c| c.is_ascii_digit())
                        || stripped.chars().nth(9) == Some('X'))
                {
                    return Err(DomainError::validation("invalid ISBN-10 format"));
                }
                // ISBN-10 checksum: sum_{i=0..9} (d_i * (10 - i)) % 11 == 0
                let sum: u32 = stripped
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        let d = if c == 'X' {
                            10
                        } else {
                            c.to_digit(10).unwrap_or(0)
                        };
                        // The 10-i arithmetic is bounded: i is in 0..9,
                        // so 10 - i is in 1..11, well within i32 range.
                        d * (10_u32 - u32::try_from(i).unwrap_or(0))
                    })
                    .sum();
                if sum % 11 != 0 {
                    return Err(DomainError::validation("ISBN-10 checksum failed"));
                }
                Ok(Self(stripped))
            }
            13 => {
                if !stripped.chars().all(|c| c.is_ascii_digit()) {
                    return Err(DomainError::validation("invalid ISBN-13 format"));
                }
                // ISBN-13 checksum: alternating 1, 3 weights, sum % 10 == 0
                let sum: u32 = stripped
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        let d = c.to_digit(10).unwrap_or(0);
                        d * if i % 2 == 0 { 1 } else { 3 }
                    })
                    .sum();
                if sum % 10 != 0 {
                    return Err(DomainError::validation("ISBN-13 checksum failed"));
                }
                Ok(Self(stripped))
            }
            _ => Err(DomainError::validation(
                "ISBN must be 10 or 13 digits (with optional hyphens)",
            )),
        }
    }

    /// Returns the inner string representation (digits only, no
    /// hyphens).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A book title (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookTitle(String);

impl BookTitle {
    /// Constructs a new `BookTitle`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("book title must not be empty"));
        }
        if s.chars().count() > 200 {
            return Err(DomainError::validation("book title must be <= 200 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A book number (1..=200 chars, unique within a school).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookNumber(String);

impl BookNumber {
    /// Constructs a new `BookNumber`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("book number must not be empty"));
        }
        if s.chars().count() > 200 {
            return Err(DomainError::validation("book number must be <= 200 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A book author (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author(String);

impl Author {
    /// Constructs a new `Author`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("author must not be empty"));
        }
        if s.chars().count() > 200 {
            return Err(DomainError::validation("author must be <= 200 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A publisher name (1..=200 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publisher(String);

impl Publisher {
    /// Constructs a new `Publisher`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("publisher must not be empty"));
        }
        if s.chars().count() > 200 {
            return Err(DomainError::validation("publisher must be <= 200 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An edition label (1..=50 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edition(String);

impl Edition {
    /// Constructs a new `Edition`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("edition must not be empty"));
        }
        if s.chars().count() > 50 {
            return Err(DomainError::validation("edition must be <= 50 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A rack number (0..=50 chars, optional).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RackNumber(String);

impl RackNumber {
    /// Constructs a new `RackNumber`, validating length-bounded.
    /// Empty strings are allowed (representing "no rack assigned").
    pub fn new(raw: &str) -> Result<Self> {
        if raw.chars().count() > 50 {
            return Err(DomainError::validation("rack number must be <= 50 chars"));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A book category name (1..=200 chars, unique within a school).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryName(String);

impl CategoryName {
    /// Constructs a new `CategoryName`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("category name must not be empty"));
        }
        if s.chars().count() > 200 {
            return Err(DomainError::validation(
                "category name must be <= 200 chars",
            ));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A free-text description (0..=500 chars).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Details(String);

impl Details {
    /// Constructs a new `Details`, validating length-bounded.
    /// Empty strings are allowed.
    pub fn new(raw: &str) -> Result<Self> {
        if raw.chars().count() > 500 {
            return Err(DomainError::validation("details must be <= 500 chars"));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Member value objects
// =============================================================================

/// The polymorphic reference to a student or staff member.
/// In storage, the `student_staff_id` column is interpreted
/// based on the `MemberType` (RoleId).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemberId {
    /// A student member.
    Student(StudentId),
    /// A staff member.
    Staff(StaffId),
}

impl MemberId {
    /// Returns the underlying platform id (Student or Staff).
    #[must_use]
    pub fn as_uuid(&self) -> Uuid {
        match self {
            Self::Student(s) => s.as_uuid(),
            Self::Staff(s) => s.as_uuid(),
        }
    }
}

/// The member's external id (e.g. admission number, staff id) —
/// 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberUdId(String);

impl MemberUdId {
    /// Constructs a new `MemberUdId`, validating non-empty and
    /// length-bounded.
    pub fn new(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return Err(DomainError::validation("member ud_id must not be empty"));
        }
        if s.chars().count() > 191 {
            return Err(DomainError::validation("member ud_id must be <= 191 chars"));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Issue value objects
// =============================================================================

/// The quantity of copies issued (or returned / renewed) in a
/// single transaction. Always > 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IssueQuantity(pub u32);

impl IssueQuantity {
    /// Constructs a new `IssueQuantity`, validating `> 0`.
    pub fn new(value: u32) -> Result<Self> {
        if value == 0 {
            return Err(DomainError::validation("issue quantity must be > 0"));
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// The date the book was given to the member. Must be on or
/// after the academic year start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GivenDate(pub NaiveDate);

impl GivenDate {
    /// Constructs a new `GivenDate`.
    pub fn new(date: NaiveDate) -> Result<Self> {
        Ok(Self(date))
    }
    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// The date the book is due. Must be strictly after the given
/// date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DueDate(pub NaiveDate);

impl DueDate {
    /// Constructs a new `DueDate` from a given date + loan period.
    pub fn from_given(given: GivenDate, loan_days: u16) -> Result<Self> {
        let due = given.0 + chrono::Duration::days(i64::from(loan_days));
        Ok(Self(due))
    }

    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// The date the book was returned. Must be >= the given date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReturnDate(pub NaiveDate);

impl ReturnDate {
    /// Returns the inner date.
    #[must_use]
    pub const fn value(self) -> NaiveDate {
        self.0
    }
}

/// A free-text note attached to a book issue, return, renewal,
/// or fine. 0..=500 chars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueNote(String);

impl IssueNote {
    /// Constructs a new `IssueNote`, validating length-bounded.
    /// Empty strings are allowed.
    pub fn new(raw: &str) -> Result<Self> {
        if raw.chars().count() > 500 {
            return Err(DomainError::validation("issue note must be <= 500 chars"));
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Money & quantity
// =============================================================================

/// The acquisition price per copy of a book (non-negative integer
/// in minor units). Mirrors the facilities `BookPrice`/`SellPrice`
/// pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BookPrice(pub i64);

impl BookPrice {
    /// Constructs a new `BookPrice`, validating `>= 0`.
    pub fn new(value: i64) -> Result<Self> {
        if value < 0 {
            return Err(DomainError::validation("book price must be >= 0"));
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> i64 {
        self.0
    }
}

/// The amount of a fine. Non-negative `Decimal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FineAmount(pub Decimal);

impl FineAmount {
    /// Constructs a new `FineAmount`, validating `>= 0`.
    pub fn new(value: Decimal) -> Result<Self> {
        if value.is_sign_negative() {
            return Err(DomainError::validation("fine amount must be >= 0"));
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> Decimal {
        self.0
    }
}

/// The per-day fine rate configured per school.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FinePerDay(pub Decimal);

impl FinePerDay {
    /// Constructs a new `FinePerDay`, validating `>= 0`.
    pub fn new(value: Decimal) -> Result<Self> {
        if value.is_sign_negative() {
            return Err(DomainError::validation("fine per day must be >= 0"));
        }
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> Decimal {
        self.0
    }
}

/// The number of days a book is overdue. `u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DaysOverdue(pub u32);

impl DaysOverdue {
    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// The number of physical copies of a book held by the library.
/// Non-negative `u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StockCopies(pub u32);

impl StockCopies {
    /// Constructs a new `StockCopies`, validating `>= 0`.
    pub fn new(value: u32) -> Result<Self> {
        // u32 is non-negative by construction; this is a hook
        // for future validation (e.g. upper bounds).
        Ok(Self(value))
    }

    /// Returns the inner value.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

// =============================================================================
// Enums
// =============================================================================

/// The lifecycle status of a `BookIssue`. `Overdue` is a derived
/// state computed from the due date; it is set by a query or a
/// scheduled job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueStatus {
    /// The book has been issued; not yet returned.
    Issued,
    /// The book has been returned.
    Returned,
    /// The issue has been renewed at least once.
    Renewed,
    /// The book's due date is in the past and the book has not
    /// been returned.
    Overdue,
    /// The book has been marked lost.
    Lost,
}

impl IssueStatus {
    /// Returns the wire-form string for the issue status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Issued => "Issued",
            Self::Returned => "Returned",
            Self::Renewed => "Renewed",
            Self::Overdue => "Overdue",
            Self::Lost => "Lost",
        }
    }
}

/// The active/inactive state of a `LibraryMember`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemberStatus {
    /// The member is active and may receive new issues.
    Active,
    /// The member is inactive (deactivated or withdrawn).
    Inactive,
    /// The member is blocked (e.g. for unpaid fines).
    Blocked,
}

impl MemberStatus {
    /// Returns the wire-form string for the member status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Blocked => "Blocked",
        }
    }
}

/// The cataloging state of a `Book`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BookStatus {
    /// The book is available in the catalog and at least one
    /// copy is in stock.
    Available,
    /// The book is in the catalog but all copies are on issue.
    Catalogued,
    /// The book has been retired from the catalog.
    Retired,
    /// The book has been lost.
    Lost,
}

impl BookStatus {
    /// Returns the wire-form string for the book status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "Available",
            Self::Catalogued => "Catalogued",
            Self::Retired => "Retired",
            Self::Lost => "Lost",
        }
    }
}

/// The reason a book's stock count was adjusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StockAdjustmentReason {
    /// New copies were acquired.
    Acquisition,
    /// Copies were written off (damaged, lost in inventory).
    WriteOff,
    /// Copies were transferred to another location.
    Transfer,
    /// A periodic stocktake correction.
    Stocktake,
    /// A manual correction by a librarian.
    Manual,
}

impl StockAdjustmentReason {
    /// Returns the wire-form string for the adjustment reason.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acquisition => "Acquisition",
            Self::WriteOff => "WriteOff",
            Self::Transfer => "Transfer",
            Self::Stocktake => "Stocktake",
            Self::Manual => "Manual",
        }
    }
}

/// The reason a `Fine` was calculated. Used by the
/// `FineCalculated` event payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FineReason {
    /// A late-return fine (the most common).
    LateReturn,
    /// A lost-book fine (replacement cost).
    Lost,
    /// A manually-entered fine by a librarian.
    Manual,
}

impl FineReason {
    /// Returns the wire-form string for the fine reason.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LateReturn => "LateReturn",
            Self::Lost => "Lost",
            Self::Manual => "Manual",
        }
    }
}

/// The kind of fine computation. Mirrors the
/// `LateFeeKind` pattern at
/// `crates/domains/finance/src/services.rs:1259` and the spec's
/// `services.md#FineCalculationService`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FineKind {
    /// A flat amount in minor units (regardless of the
    /// overdueness or replacement cost).
    FixedAmount(i64),
    /// A per-day rate in minor units.
    PerDayRate(i64),
    /// A percentage of the book price (0..=100).
    PercentOfPrice(u8),
}

impl FineKind {
    /// Returns the wire-form string for the fine kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FixedAmount(_) => "FixedAmount",
            Self::PerDayRate(_) => "PerDayRate",
            Self::PercentOfPrice(_) => "PercentOfPrice",
        }
    }
}

/// The per-school fine settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FineSettings {
    /// The fine computation kind.
    pub kind: FineKind,
    /// The grace period in days (no fine is calculated for
    /// returns within `given_date + grace_period_days`).
    pub grace_period_days: u16,
}

impl Default for FineSettings {
    fn default() -> Self {
        Self {
            kind: FineKind::PerDayRate(50),
            grace_period_days: 0,
        }
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

    #[test]
    fn isbn_10_valid() {
        // ISBN-10: 0-306-40615-2 (checksum-valid).
        Isbn::parse("0306406152").expect("valid ISBN-10");
    }

    #[test]
    fn isbn_10_with_hyphens_valid() {
        Isbn::parse("0-306-40615-2").expect("valid ISBN-10 with hyphens");
    }

    #[test]
    fn isbn_10_invalid_checksum() {
        let err = Isbn::parse("0306406153").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn isbn_13_valid() {
        // ISBN-13: 978-0-13-468599-1 (checksum-valid).
        Isbn::parse("9780134685991").expect("valid ISBN-13");
    }

    #[test]
    fn isbn_13_invalid_checksum() {
        let err = Isbn::parse("9780134685992").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn isbn_wrong_length_rejected() {
        let err = Isbn::parse("12345").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn book_title_validates_length() {
        assert!(BookTitle::new("").is_err());
        assert!(BookTitle::new("   ").is_err());
        assert!(BookTitle::new("Pride and Prejudice").is_ok());
        assert!(BookTitle::new(&"x".repeat(201)).is_err());
    }

    #[test]
    fn issue_quantity_must_be_positive() {
        assert!(IssueQuantity::new(0).is_err());
        assert!(IssueQuantity::new(1).is_ok());
        assert!(IssueQuantity::new(100).is_ok());
    }

    #[test]
    fn fine_amount_must_be_non_negative() {
        assert!(FineAmount::new(Decimal::from(100)).is_ok());
        assert!(FineAmount::new(Decimal::from(0)).is_ok());
        assert!(FineAmount::new(Decimal::from(-1)).is_err());
    }

    #[test]
    fn fine_settings_default_is_per_day_50() {
        let s = FineSettings::default();
        assert!(matches!(s.kind, FineKind::PerDayRate(50)));
        assert_eq!(s.grace_period_days, 0);
    }

    #[test]
    fn issue_status_wire_forms() {
        assert_eq!(IssueStatus::Issued.as_str(), "Issued");
        assert_eq!(IssueStatus::Returned.as_str(), "Returned");
        assert_eq!(IssueStatus::Renewed.as_str(), "Renewed");
        assert_eq!(IssueStatus::Overdue.as_str(), "Overdue");
        assert_eq!(IssueStatus::Lost.as_str(), "Lost");
    }

    #[test]
    fn member_id_sums_student_and_staff() {
        use educore_core::clock::IdGenerator;
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let s = MemberId::Student(StudentId::new(school, Uuid::now_v7()));
        assert!(matches!(s, MemberId::Student(_)));
        let s2 = MemberId::Staff(StaffId::new(school, Uuid::now_v7()));
        assert!(matches!(s2, MemberId::Staff(_)));
    }
}
