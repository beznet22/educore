//! Integration tests for the **BookReturn aggregate** vertical slice.
//!
//! Pins the create + lifecycle contract for
//! [`BookReturn`](educore_library::aggregate::BookReturn)
//! end-to-end through the service layer:
//!
//! 1. `return_book` validates that the source `BookIssue`
//!    is open (an already-returned or lost issue raises a
//!    conflict), transitions the issue to `Returned`,
//!    constructs a fresh `BookReturn` aggregate with the
//!    full 10-field audit footer initialised, and emits
//!    `BookReturned` (issue-level) + `BookReturnRecorded`
//!    (return-row-level) events.
//! 2. The emitted `BookReturned` event carries the right
//!    `event_type` / `aggregate_type` / `school_id`, with
//!    `aggregate_id` pointing at the **issue** (the
//!    canonical state lives on `BookIssue`; the `BookReturn`
//!    is an append-only history row).
//!
//! The tests use the same fixture pattern as
//! `tests/book_issue.rs` (`TestClock` + `SystemIdGen`).
//!
//! Mirrors `crates/domains/library/tests/book_issue.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_academic::AcademicYearId;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_library::prelude::*;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

/// Mint a `BookId` for the given school.
fn book_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> BookId {
    BookId::new(school, g.next_uuid())
}

/// Mint a `LibraryMemberId` for the given school.
fn member_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> LibraryMemberId {
    LibraryMemberId::new(school, g.next_uuid())
}

/// Mint an `AcademicYearId` for the given school.
fn year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

/// Construct a `NaiveDate` from parts, panicking on invalid
/// inputs (test fixture only).
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: create a BookReturn on an open issue
// =============================================================================

/// End-to-end happy path for the BookReturn aggregate.
/// Seed an open `BookIssue` and run `return_book` against
/// it on the due date, asserting that:
///
/// 1. The `BookReturn` aggregate carries the right
///    `school_id` (derived from the issue id), `book_id`,
///    `library_member_id`, `quantity`, `return_date`,
///    `note`, and the audit footer initialised (`version =
///    1`, `created_by` / `updated_by` from the actor).
/// 2. The source `BookIssue` transitions to `Returned`
///    (its `is_open` predicate returns false).
/// 3. The emitted `BookReturned` event is a
///    `library.book_issue.returned` carrying the right
///    `event_type` / `aggregate_type` / `school_id`, with
///    `aggregate_id` matching the **issue** id (the
///    canonical state lives on `BookIssue`).
#[test]
fn book_return_on_open_issue_emits_event_and_populates_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Seed an open BookIssue.
    let mut issue = BookIssue::fresh(
        BookIssueId::new(school, g.next_uuid()),
        year_id(&g, school),
        book_id(&g, school),
        member_id(&g, school),
        IssueQuantity::new(1).expect("positive quantity"),
        GivenDate(date(2026, 9, 15)),
        DueDate(date(2026, 9, 29)),
        None,
        g.next_user_id(),
        Timestamp::now(),
        g.next_correlation_id(),
    );
    assert!(issue.is_open());

    // Run the return flow on the due date (on-time return).
    let cmd = ReturnBookCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        return_date: ReturnDate(date(2026, 9, 29)),
        note: None,
    };
    let result = return_book(cmd, &clock, &ids, &mut issue, None).expect("return");

    // The BookReturn aggregate is populated.
    assert_eq!(result.book_return.school_id, school);
    assert_eq!(result.book_return.book_issue_id, issue.id);
    assert_eq!(result.book_return.book_id, result.book_issue.book_id);
    assert_eq!(
        result.book_return.library_member_id,
        result.book_issue.library_member_id
    );
    assert_eq!(result.book_return.quantity.value(), 1);
    assert_eq!(result.book_return.return_date.value(), date(2026, 9, 29));
    assert_eq!(result.book_return.created_by, tenant.actor_id);
    assert_eq!(result.book_return.updated_by, tenant.actor_id);
    assert_eq!(result.book_return.version.get(), 1);
    assert!(result.book_return.active_status.is_active());

    // The source issue is now Returned (not open).
    assert!(!issue.is_open());
    assert_eq!(issue.issue_status, IssueStatus::Returned);

    // The BookReturned event is library.book_issue.returned.
    assert_eq!(
        <BookReturned as DomainEvent>::EVENT_TYPE,
        "library.book_issue.returned"
    );
    assert_eq!(<BookReturned as DomainEvent>::AGGREGATE_TYPE, "book_issue");
    assert_eq!(<BookReturned as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(result.returned_event.aggregate_id(), issue.id.as_uuid());
    assert_eq!(result.returned_event.school_id(), school);
    assert_eq!(result.returned_event.book_issue_id, issue.id);

    // The BookReturnRecorded event is
    // library.book_return.recorded.
    assert_eq!(
        <BookReturnRecorded as DomainEvent>::EVENT_TYPE,
        "library.book_return.recorded"
    );
    assert_eq!(
        <BookReturnRecorded as DomainEvent>::AGGREGATE_TYPE,
        "book_return"
    );
    assert_eq!(<BookReturnRecorded as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(
        result.return_recorded_event.aggregate_id(),
        result.book_return.id.as_uuid()
    );
    assert_eq!(result.return_recorded_event.school_id(), school);
}

// =============================================================================
// Validation failure: returning an already-returned issue is rejected
// =============================================================================

/// Validation-failure path on the return flow: when the
/// source `BookIssue` is already in a non-open status
/// (e.g. `Returned` or `Lost`), `return_book` returns
/// `DomainError::Conflict` and the issue is left unchanged.
#[test]
fn book_return_on_already_returned_issue_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Seed an open BookIssue and run the return flow once.
    let mut issue = BookIssue::fresh(
        BookIssueId::new(school, g.next_uuid()),
        year_id(&g, school),
        book_id(&g, school),
        member_id(&g, school),
        IssueQuantity::new(1).expect("positive quantity"),
        GivenDate(date(2026, 9, 15)),
        DueDate(date(2026, 9, 29)),
        None,
        g.next_user_id(),
        Timestamp::now(),
        g.next_correlation_id(),
    );
    let first_cmd = ReturnBookCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        return_date: ReturnDate(date(2026, 9, 29)),
        note: None,
    };
    let _first = return_book(first_cmd, &clock, &ids, &mut issue, None).expect("first return");
    assert!(!issue.is_open());

    // A second return on the now-closed issue must fail with
    // a Conflict error.
    let second_cmd = ReturnBookCommand {
        tenant,
        book_issue_id: issue.id,
        return_date: ReturnDate(date(2026, 9, 30)),
        note: None,
    };
    let err = return_book(second_cmd, &clock, &ids, &mut issue, None)
        .expect_err("second return must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );

    // The issue is still Returned (the failed second return
    // must not move the state).
    assert_eq!(issue.issue_status, IssueStatus::Returned);
}
