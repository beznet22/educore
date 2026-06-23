//! Integration tests for the **Library domain workflows**.
//!
//! Implements (lean subset of) `docs/specs/library/workflows.md`:
//!
//! - Book Cataloging Workflow — `acquire_book_happy_path`
//! - Book Issue Workflow — `issue_book_happy_path`
//! - Book Return Workflow — `return_book_happy_path`
//!
//! Pattern matches `crates/domains/cms/tests/workflows.rs` (lean).
//! The handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **aggregate layer** contract
//! that the service factory fns (`add_book`, `create_book_issue`,
//! `return_book`) return.

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
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_library::prelude::*;

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

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Book Cataloging (workflows.md § "Book Cataloging Workflow")
/// step 2: `AddBook` returns a fresh `Book` aggregate with
/// the requested stock count and emits `BookAdded`.
#[test]
fn acquire_book_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let cat_id = BookCategoryId::new(school, g.next_uuid());
    let cmd = AddBookCommand {
        tenant,
        academic_year_id: AcademicYearId::new(school, g.next_uuid()),
        book_title: BookTitle::new("The Rust Programming Language").unwrap(),
        book_number: None,
        isbn_no: None,
        author_name: None,
        publisher_name: None,
        edition: None,
        rack_number: None,
        quantity: StockCopies(5),
        book_price: None,
        post_date: None,
        details: None,
        book_category_id: cat_id,
        book_subject_id: None,
    };
    let (book, event) = add_book(cmd, &clock, &ids).unwrap();
    assert_eq!(book.school_id, school);
    assert_eq!(book.quantity, StockCopies(5));
    assert_eq!(event.book_id, book.id);
    assert_eq!(event.quantity, StockCopies(5));
    assert_eq!(<BookAdded as DomainEvent>::EVENT_TYPE, "library.book.added");
}

/// Book Issue (workflows.md § "Book Issue Workflow") step 3:
/// `IssueBook` returns a fresh `BookIssue` aggregate in the
/// `Issued` (open) state and emits `BookIssued`.
#[test]
fn issue_book_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let cmd = IssueBookCommand {
        tenant,
        academic_year_id: AcademicYearId::new(school, g.next_uuid()),
        book_id: BookId::new(school, g.next_uuid()),
        library_member_id: LibraryMemberId::new(school, g.next_uuid()),
        quantity: IssueQuantity(1),
        given_date: GivenDate(date(2026, 6, 14)),
        due_date: DueDate(date(2026, 6, 28)),
        note: None,
    };
    let created = create_book_issue(cmd, &clock, &ids).unwrap();
    assert_eq!(created.book_issue.school_id, school);
    assert!(created.book_issue.is_open());
    assert_eq!(created.book_issue.quantity, IssueQuantity(1));
    assert_eq!(created.event.book_id, created.book_issue.book_id);
    assert_eq!(<BookIssued as DomainEvent>::EVENT_TYPE, "library.book_issue.issued");
}

/// Book Return (workflows.md § "Book Issue Workflow" step 7):
/// `ReturnBook` transitions the issue to `Returned`, creates
/// a `BookReturn` aggregate, and emits `BookReturned` (plus
/// `BookReturnRecorded`; no fine here — return is on-time).
#[test]
fn return_book_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let school = tenant.school_id;
    // Seed an open BookIssue to return.
    let mut issue = BookIssue::fresh(
        BookIssueId::new(school, g.next_uuid()),
        AcademicYearId::new(school, g.next_uuid()),
        BookId::new(school, g.next_uuid()),
        LibraryMemberId::new(school, g.next_uuid()),
        IssueQuantity(1),
        GivenDate(date(2026, 6, 14)),
        DueDate(date(2026, 6, 28)),
        None,
        g.next_user_id(),
        Timestamp::now(),
        g.next_correlation_id(),
    );
    assert!(issue.is_open());

    let cmd = ReturnBookCommand {
        tenant,
        book_issue_id: issue.id,
        return_date: ReturnDate(date(2026, 6, 28)),
        note: None,
    };
    let result = return_book(cmd, &clock, &ids, &mut issue, None).unwrap();
    assert_eq!(result.book_issue.id, issue.id);
    assert!(!issue.is_open());
    assert_eq!(
        <BookReturned as DomainEvent>::EVENT_TYPE,
        "library.book_issue.returned"
    );
}
