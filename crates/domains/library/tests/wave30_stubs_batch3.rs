//! Integration tests for the Wave 30 stub replacements in
//! `crates/domains/library/src/services.rs` (batch 3 —
//! book_issue lifecycle).
//!
//! Pins the contract for:
//!
//! - [`renew_book`](educore_library::services::renew_book)
//! - [`mark_book_lost`](educore_library::services::mark_book_lost)
//! - [`record_book_return`](educore_library::services::record_book_return)
//! - [`waive_book_issue_fine`](educore_library::services::waive_book_issue_fine)
//!
//! Each test pins one happy path for the corresponding
//! factory fn. Mirrors the pattern in `tests/wave30_stubs.rs`
//! and `tests/wave30_stubs_member.rs`.

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
use educore_library::services::{
    mark_book_lost, record_book_return, renew_book, waive_book_issue_fine,
};
use educore_library::value_objects::{
    BookReturnId, DueDate, FineAmount, FineId, FinePerDay, FineReason, GivenDate, IssueNote,
    IssueQuantity, ReturnDate,
};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let tenant = TenantContext::for_user(
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    (tenant, g)
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

/// Construct a `NaiveDate` from parts (test fixture only).
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

/// Seeds a fresh open `BookIssue` for the given tenant with
/// a 14-day loan window.
fn seed_open_issue(tenant: &TenantContext, g: &SystemIdGen) -> BookIssue {
    let school = tenant.school_id;
    BookIssue::fresh(
        BookIssueId::new(school, g.next_uuid()),
        year_id(g, school),
        book_id(g, school),
        member_id(g, school),
        IssueQuantity::new(1).expect("positive quantity"),
        GivenDate(date(2026, 9, 15)),
        DueDate(date(2026, 9, 29)),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

// =============================================================================
// renew_book
// =============================================================================

/// `renew_book` transitions the issue to `Renewed`, updates
/// the due date, bumps the version, and emits a `BookRenewed`
/// event whose `from_due_date` matches the original and
/// `to_due_date` matches the new due date.
#[test]
fn renew_book_extends_due_date_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut issue = seed_open_issue(&tenant, &g);
    let original_due = issue.due_date;
    let initial_version = issue.version.get();
    let new_due = DueDate(date(2026, 10, 13));

    let cmd = RenewBookCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        new_due_date: new_due,
    };
    let (returned_issue, event) = renew_book(cmd, &clock, &ids, &mut issue).expect("renew");

    assert_eq!(returned_issue.due_date, new_due);
    assert_eq!(issue.due_date, new_due);
    assert_eq!(issue.issue_status, IssueStatus::Renewed);
    assert_eq!(issue.version.get(), initial_version + 1);
    assert_eq!(
        <BookRenewed as DomainEvent>::EVENT_TYPE,
        "library.book_issue.renewed"
    );
    assert_eq!(<BookRenewed as DomainEvent>::AGGREGATE_TYPE, "book_issue");
    assert_eq!(event.aggregate_id(), issue.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.from_due_date, original_due);
    assert_eq!(event.to_due_date, new_due);
}

/// `renew_book` with a `new_due_date` on or before the
/// current due date is rejected by the eligibility check
/// and leaves the aggregate untouched.
#[test]
fn renew_book_with_earlier_due_date_returns_validation_error() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut issue = seed_open_issue(&tenant, &g);
    let initial_due = issue.due_date;
    let initial_version = issue.version.get();

    let cmd = RenewBookCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        new_due_date: DueDate(date(2026, 9, 20)),
    };
    let err = renew_book(cmd, &clock, &ids, &mut issue)
        .expect_err("earlier due date must fail");
    assert!(
        matches!(err, DomainError::Validation(_) | DomainError::Conflict(_)),
        "expected Validation/Conflict, got {err:?}"
    );
    assert_eq!(issue.due_date, initial_due);
    assert_eq!(issue.issue_status, IssueStatus::Issued);
    assert_eq!(issue.version.get(), initial_version);
}

// =============================================================================
// mark_book_lost
// =============================================================================

/// `mark_book_lost` transitions an open issue to `Lost`,
/// bumps the version, and emits a `BookMarkedLost` event
/// with the right `EVENT_TYPE` / `AGGREGATE_TYPE` /
/// `school_id`.
#[test]
fn mark_book_lost_transitions_issue_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut issue = seed_open_issue(&tenant, &g);
    let initial_version = issue.version.get();

    let cmd = MarkBookLostCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        note: Some(IssueNote::new("book never returned").expect("note")),
    };
    let (returned_issue, event) = mark_book_lost(cmd, &clock, &ids, &mut issue).expect("lost");

    assert_eq!(returned_issue.issue_status, IssueStatus::Lost);
    assert_eq!(issue.issue_status, IssueStatus::Lost);
    assert!(!issue.is_open());
    assert_eq!(issue.version.get(), initial_version + 1);
    assert_eq!(
        <BookMarkedLost as DomainEvent>::EVENT_TYPE,
        "library.book_issue.marked_lost"
    );
    assert_eq!(<BookMarkedLost as DomainEvent>::AGGREGATE_TYPE, "book_issue");
    assert_eq!(event.aggregate_id(), issue.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.quantity.value(), 1);
    assert_eq!(event.note.as_ref().map(IssueNote::as_str), Some("book never returned"));
}

/// `mark_book_lost` against a non-open (e.g. already
/// returned) issue is rejected with `DomainError::Conflict`
/// and leaves the aggregate untouched.
#[test]
fn mark_book_lost_on_returned_issue_returns_conflict() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut issue = seed_open_issue(&tenant, &g);
    issue.mark_returned(tenant.actor_id, Timestamp::now(), ids.next_event_id());
    let initial_version = issue.version.get();

    let cmd = MarkBookLostCommand {
        tenant: tenant.clone(),
        book_issue_id: issue.id,
        note: None,
    };
    let err = mark_book_lost(cmd, &clock, &ids, &mut issue).expect_err("closed issue must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert_eq!(issue.issue_status, IssueStatus::Returned);
    assert_eq!(issue.version.get(), initial_version);
}

// =============================================================================
// record_book_return
// =============================================================================

/// `record_book_return` constructs a fresh `BookReturn`
/// aggregate from the command, initialises the audit footer,
/// and emits a `BookReturnRecorded` event with the right
/// `EVENT_TYPE` / `AGGREGATE_TYPE` / `school_id`.
#[test]
fn record_book_return_emits_event_and_populates_aggregate() {
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let issue_id = BookIssueId::new(school, g.next_uuid());
    let book = book_id(&g, school);
    let member = member_id(&g, school);

    let cmd = RecordBookReturnCommand {
        tenant: tenant.clone(),
        book_return_id: BookReturnId::new(school, g.next_uuid()),
        book_issue_id: issue_id,
        book_id: book,
        library_member_id: member,
        quantity: IssueQuantity::new(2).expect("positive"),
        return_date: ReturnDate(date(2026, 9, 28)),
        note: None,
    };
    let (book_return, event) = record_book_return(cmd, &clock, &ids).expect("record");

    assert_eq!(book_return.school_id, school);
    assert_eq!(book_return.book_issue_id, issue_id);
    assert_eq!(book_return.book_id, book);
    assert_eq!(book_return.library_member_id, member);
    assert_eq!(book_return.quantity.value(), 2);
    assert_eq!(book_return.return_date.value(), date(2026, 9, 28));
    assert_eq!(book_return.created_by, tenant.actor_id);
    assert_eq!(book_return.version.get(), 1);
    assert!(book_return.active_status.is_active());
    assert_eq!(
        <BookReturnRecorded as DomainEvent>::EVENT_TYPE,
        "library.book_return.recorded"
    );
    assert_eq!(<BookReturnRecorded as DomainEvent>::AGGREGATE_TYPE, "book_return");
    assert_eq!(event.aggregate_id(), book_return.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.book_issue_id, issue_id);
}

// =============================================================================
// waive_book_issue_fine
// =============================================================================

/// `waive_book_issue_fine` transitions an open fine to
/// `waived = true`, stamps `waived_by` / `waived_reason`,
/// bumps the version, and emits a `FineWaived` event with
/// the right `EVENT_TYPE` / `AGGREGATE_TYPE` / `school_id`.
#[test]
fn waive_book_issue_fine_marks_waived_and_emits_event() {
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let issue_id = BookIssueId::new(school, g.next_uuid());
    let book = book_id(&g, school);
    let member = member_id(&g, school);
    let fine_id = FineId::new(school, g.next_uuid());

    let mut fine = Fine::fresh(
        fine_id,
        issue_id,
        book,
        member,
        5,
        FinePerDay::new(rust_decimal::Decimal::from(50)).expect("non-negative rate"),
        FineAmount(rust_decimal::Decimal::from(250)),
        FineReason::LateReturn,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    let initial_version = fine.version.get();
    assert!(!fine.waived);

    let cmd = WaiveBookIssueFineCommand {
        tenant: tenant.clone(),
        fine_id,
        reason: "Compassionate waiver".to_owned(),
    };
    let (returned_fine, event) =
        waive_book_issue_fine(cmd, &clock, &ids, &mut fine).expect("waive");

    assert!(returned_fine.waived);
    assert_eq!(fine.waived, true);
    assert_eq!(fine.waived_by, Some(tenant.actor_id));
    assert_eq!(
        fine.waived_reason.as_deref(),
        Some("Compassionate waiver")
    );
    assert_eq!(fine.version.get(), initial_version + 1);
    assert_eq!(<FineWaived as DomainEvent>::EVENT_TYPE, "library.fine.waived");
    assert_eq!(<FineWaived as DomainEvent>::AGGREGATE_TYPE, "fine");
    assert_eq!(event.aggregate_id(), fine.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.reason, "Compassionate waiver");
    assert_eq!(event.waived_by, tenant.actor_id);
}

/// `waive_book_issue_fine` against an already-waived fine is
/// rejected with `DomainError::Conflict` and leaves the
/// aggregate untouched.
#[test]
fn waive_book_issue_fine_twice_returns_conflict() {
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let issue_id = BookIssueId::new(school, g.next_uuid());
    let book = book_id(&g, school);
    let member = member_id(&g, school);
    let fine_id = FineId::new(school, g.next_uuid());

    let mut fine = Fine::fresh(
        fine_id,
        issue_id,
        book,
        member,
        3,
        FinePerDay::new(rust_decimal::Decimal::from(25)).expect("non-negative rate"),
        FineAmount(rust_decimal::Decimal::from(75)),
        FineReason::LateReturn,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    let initial_version = fine.version.get();

    let cmd = WaiveBookIssueFineCommand {
        tenant: tenant.clone(),
        fine_id,
        reason: "first waiver".to_owned(),
    };
    waive_book_issue_fine(cmd.clone(), &clock, &ids, &mut fine).expect("first waive");

    let err = waive_book_issue_fine(cmd, &clock, &ids, &mut fine)
        .expect_err("second waive must conflict");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    // The aggregate version is still bumped only once.
    assert_eq!(fine.version.get(), initial_version + 1);
}
