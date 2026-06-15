//! # Library domain queries
//!
//! Phase 9 ships the 6 typed query stubs (one per root aggregate).
//! Each query has a `query_type` method that returns a stable
//! dotted string, and an `execute` method that returns
//! `Err(DomainError::not_supported(...))` for now. The typed
//! executors land in a follow-up phase alongside the
//! `#[derive(DomainQuery)]` macro emissions (per the Phase 7
//! Workstream P pattern).
//!
//! Mirrors `crates/domains/finance/src/query.rs` and
//! `crates/domains/facilities/src/query.rs`.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

use crate::aggregate::{Book, BookCategory, BookIssue, BookReturn, Fine, LibraryMember};
use crate::value_objects::{
    AcademicYearId, BookCategoryId, BookId, BookIssueId, BookReturnId, FineId, LibraryMemberId,
};

// =============================================================================
// BookCategoryQuery
// =============================================================================

/// A typed query for `BookCategory` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BookCategoryQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by academic year.
    pub academic_year_id: Option<AcademicYearId>,
    /// If `true`, return only active categories.
    pub active_only: bool,
}

impl BookCategoryQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.book_category.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9; the typed executor lands in a follow-up phase.
    pub async fn execute(&self) -> Result<Vec<BookCategory>> {
        Err(DomainError::not_supported(
            "BookCategoryQuery::execute is a Phase 9 stub; real executor lands with the DomainQuery macro",
        ))
    }
}

// =============================================================================
// BookQuery
// =============================================================================

/// A typed query for `Book` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BookQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by academic year.
    pub academic_year_id: Option<AcademicYearId>,
    /// Filter by category.
    pub category_id: Option<BookCategoryId>,
    /// Free-text search query.
    pub search: Option<String>,
    /// If `true`, return only active books.
    pub active_only: bool,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl BookQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.book.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9.
    pub async fn execute(&self) -> Result<Vec<Book>> {
        Err(DomainError::not_supported(
            "BookQuery::execute is a Phase 9 stub",
        ))
    }
}

// =============================================================================
// LibraryMemberQuery
// =============================================================================

/// A typed query for `LibraryMember` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryMemberQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by academic year.
    pub academic_year_id: AcademicYearId,
    /// If `Some(true)`, return only active members.
    /// If `Some(false)`, return only inactive.
    /// If `None`, return all.
    pub active_only: Option<bool>,
}

impl LibraryMemberQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.member.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9.
    pub async fn execute(&self) -> Result<Vec<LibraryMember>> {
        Err(DomainError::not_supported(
            "LibraryMemberQuery::execute is a Phase 9 stub",
        ))
    }
}

// =============================================================================
// BookIssueQuery
// =============================================================================

/// A typed query for `BookIssue` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BookIssueQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by academic year.
    pub academic_year_id: Option<AcademicYearId>,
    /// Filter by member.
    pub member_id: Option<LibraryMemberId>,
    /// Filter by book.
    pub book_id: Option<BookId>,
    /// If `Some(as_of)`, return only issues that are open on or
    /// after `as_of` (i.e. the due date is in the future).
    pub as_of: Option<NaiveDate>,
    /// If `Some(true)`, return only overdue issues.
    pub overdue_only: Option<bool>,
}

impl BookIssueQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.book_issue.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9.
    pub async fn execute(&self) -> Result<Vec<BookIssue>> {
        Err(DomainError::not_supported(
            "BookIssueQuery::execute is a Phase 9 stub",
        ))
    }
}

// =============================================================================
// BookReturnQuery
// =============================================================================

/// A typed query for `BookReturn` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BookReturnQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by book.
    pub book_id: Option<BookId>,
    /// Filter by member.
    pub member_id: Option<LibraryMemberId>,
    /// If `Some((from, to))`, restrict to the inclusive
    /// return_date range.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
}

impl BookReturnQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.book_return.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9.
    pub async fn execute(&self) -> Result<Vec<BookReturn>> {
        Err(DomainError::not_supported(
            "BookReturnQuery::execute is a Phase 9 stub",
        ))
    }
}

// =============================================================================
// FineQuery
// =============================================================================

/// A typed query for `Fine` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct FineQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Filter by book issue.
    pub book_issue_id: Option<BookIssueId>,
    /// Filter by member.
    pub member_id: Option<LibraryMemberId>,
    /// If `Some(true)`, return only open (non-waived) fines.
    /// If `Some(false)`, return only waived fines.
    /// If `None`, return all.
    pub open_only: Option<bool>,
}

impl FineQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "library.fine.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 9.
    pub async fn execute(&self) -> Result<Vec<Fine>> {
        Err(DomainError::not_supported(
            "FineQuery::execute is a Phase 9 stub",
        ))
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

    #[tokio::test]
    async fn every_query_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let q1 = BookCategoryQuery {
            school_id: s,
            academic_year_id: None,
            active_only: true,
        };
        assert!(q1.execute().await.is_err());
        let q2 = BookQuery {
            school_id: s,
            academic_year_id: None,
            category_id: None,
            search: None,
            active_only: true,
            limit: None,
        };
        assert!(q2.execute().await.is_err());
        let q3 = LibraryMemberQuery {
            school_id: s,
            academic_year_id: AcademicYearId::new(s, g.next_uuid()),
            active_only: Some(true),
        };
        assert!(q3.execute().await.is_err());
        let q4 = BookIssueQuery {
            school_id: s,
            academic_year_id: None,
            member_id: None,
            book_id: None,
            as_of: None,
            overdue_only: Some(false),
        };
        assert!(q4.execute().await.is_err());
        let q5 = BookReturnQuery {
            school_id: s,
            book_id: None,
            member_id: None,
            date_range: None,
        };
        assert!(q5.execute().await.is_err());
        let q6 = FineQuery {
            school_id: s,
            book_issue_id: None,
            member_id: None,
            open_only: Some(true),
        };
        assert!(q6.execute().await.is_err());
    }

    #[test]
    fn query_type_strings_are_stable() {
        assert_eq!(
            BookCategoryQuery::query_type(),
            "library.book_category.query"
        );
        assert_eq!(BookQuery::query_type(), "library.book.query");
        assert_eq!(LibraryMemberQuery::query_type(), "library.member.query");
        assert_eq!(BookIssueQuery::query_type(), "library.book_issue.query");
        assert_eq!(BookReturnQuery::query_type(), "library.book_return.query");
        assert_eq!(FineQuery::query_type(), "library.fine.query");
    }
}
