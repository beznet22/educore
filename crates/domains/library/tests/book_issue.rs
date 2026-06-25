//! Integration tests for the **BookIssue aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`BookIssue`](educore_library::aggregate::BookIssue)
//! end-to-end through the service layer:
//!
//! 1. `create_book_issue` validates the input (`due_date` is
//!    strictly after `given_date`, per the spec's invariant #4),
//!    constructs the aggregate with all 10 audit-footer fields
//!    initialised, and emits a [`BookIssued`] event whose
//!    `event_type` / `aggregate_type` / `school_id` match the
//!    typed id's `school_id()` accessor.
//!
//! The test uses the same fixture pattern as
//! `tests/aggregates.rs` (`TestClock` + `SystemIdGen`). The
//! handlers / outbox / audit fan-out are not yet wired
//! end-to-end; this test pins the **service layer** contract
//! that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/attendance/tests/aggregates.rs` (lean).

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
// Happy path: create on BookIssue
// =============================================================================

/// End-to-end happy path for the BookIssue aggregate. Issue
/// a single copy of a book to a member, asserting that:
///
/// 1. The create flow produces a `BookIssue` aggregate
///    carrying every field on the command, with the audit
///    footer initialised (`version = 1`, `created_by` /
///    `updated_by` from the tenant, `active_status = Active`,
///    `issue_status = Issued`).
/// 2. The event is a `BookIssued` carrying the right
///    `event_type` / `aggregate_type` / `school_id` and a
///    `book_issue_id` matching the aggregate's typed id.
#[test]
fn book_issue_create_emits_event_and_populates_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let cmd = IssueBookCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        book_id: book_id(&g, school),
        library_member_id: member_id(&g, school),
        quantity: IssueQuantity::new(1).expect("positive quantity"),
        given_date: GivenDate(date(2026, 9, 15)),
        due_date: DueDate(date(2026, 9, 29)),
        note: None,
    };
    let BookIssueCreated { book_issue, event } =
        create_book_issue(cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(book_issue.school_id, school);
    assert_eq!(book_issue.quantity.value(), 1);
    assert_eq!(book_issue.given_date.value(), date(2026, 9, 15));
    assert_eq!(book_issue.due_date.value(), date(2026, 9, 29));
    assert_eq!(book_issue.issue_status, IssueStatus::Issued);
    assert_eq!(book_issue.created_by, tenant.actor_id);
    assert_eq!(book_issue.updated_by, tenant.actor_id);
    assert!(book_issue.active_status.is_active());

    // Audit metadata footer is initialised.
    assert_eq!(book_issue.version.get(), 1);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <BookIssued as DomainEvent>::EVENT_TYPE,
        "library.book_issue.issued"
    );
    assert_eq!(<BookIssued as DomainEvent>::AGGREGATE_TYPE, "book_issue");
    assert_eq!(<BookIssued as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), book_issue.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.book_id, book_issue.book_id);
    assert_eq!(event.library_member_id, book_issue.library_member_id);
    assert_eq!(event.given_date, book_issue.given_date);
    assert_eq!(event.due_date, book_issue.due_date);
}

// =============================================================================
// Validation failure: due_date <= given_date
// =============================================================================

/// Validation-failure path on the create flow: when
/// `due_date` is on or before `given_date`, `create_book_issue`
/// returns `DomainError::Validation` per the spec's invariant #4
/// (`DueDate` is strictly after `GivenDate`). No aggregate is
/// constructed, no event is minted.
#[test]
fn book_issue_create_with_inverted_dates_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // given_date == due_date (same day). Must fail.
    let cmd = IssueBookCommand {
        tenant,
        academic_year_id: year_id(&g, school),
        book_id: book_id(&g, school),
        library_member_id: member_id(&g, school),
        quantity: IssueQuantity::new(1).expect("positive quantity"),
        given_date: GivenDate(date(2026, 9, 15)),
        due_date: DueDate(date(2026, 9, 15)),
        note: None,
    };
    let err = create_book_issue(cmd, &clock, &ids).expect_err("inverted dates must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // given_date > due_date (book due in the past). Must also
    // fail — past due dates are nonsensical for a new issue.
    let (tenant2, g2) = admin_context();
    let school2 = tenant2.school_id;
    let cmd2 = IssueBookCommand {
        tenant: tenant2,
        academic_year_id: year_id(&g2, school2),
        book_id: book_id(&g2, school2),
        library_member_id: member_id(&g2, school2),
        quantity: IssueQuantity::new(1).expect("positive quantity"),
        given_date: GivenDate(date(2026, 9, 29)),
        due_date: DueDate(date(2026, 9, 15)),
        note: None,
    };
    let err2 = create_book_issue(cmd2, &clock, &ids).expect_err("past due_date must fail");
    assert!(
        matches!(err2, DomainError::Validation(_)),
        "expected Validation, got {err2:?}"
    );
}
