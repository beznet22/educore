//! Integration tests for the Wave 30 stub replacements in
//! `crates/domains/library/src/services.rs`.
//!
//! Pins the contract for:
//!
//! - [`delete_book_category`](educore_library::services::delete_book_category)
//! - [`update_book`](educore_library::services::update_book)
//! - [`delete_book`](educore_library::services::delete_book)
//! - [`adjust_book_quantity`](educore_library::services::adjust_book_quantity)
//!
//! Each test pins one happy path + one validation failure for
//! the corresponding factory fn. The handlers / outbox /
//! audit fan-out are not yet wired end-to-end; these tests
//! pin the **service layer** contract that the dispatcher
//! will eventually wrap. Mirrors
//! `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::AcademicYearId;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_library::events::BookUpdated;
use educore_library::prelude::*;
use educore_library::services::{
    adjust_book_quantity, delete_book, delete_book_category, update_book,
};

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

/// Seeds a fresh `BookCategory` for the given tenant.
fn seed_category(tenant: &TenantContext, g: &SystemIdGen) -> BookCategory {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let cmd = CreateBookCategoryCommand {
        tenant: tenant.clone(),
        category_name: CategoryName::new("Fiction").expect("non-empty"),
    };
    let (category, _event) = create_book_category(cmd, &clock, &ids).expect("create");
    category
}

/// Seeds a fresh `Book` for the given tenant + category.
#[allow(clippy::too_many_arguments)]
fn seed_book(
    tenant: &TenantContext,
    g: &SystemIdGen,
    category_id: BookCategoryId,
    title: &str,
    quantity: u32,
) -> Book {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let school = tenant.school_id;
    let cmd = AddBookCommand {
        tenant: tenant.clone(),
        academic_year_id: AcademicYearId::new(school, g.next_uuid()),
        book_title: BookTitle::new(title).expect("non-empty title"),
        book_number: None,
        isbn_no: None,
        author_name: None,
        publisher_name: None,
        edition: None,
        rack_number: None,
        quantity: StockCopies(quantity),
        book_price: None,
        post_date: None,
        details: None,
        book_category_id: category_id,
        book_subject_id: None,
    };
    let (book, _event) = add_book(cmd, &clock, &ids).expect("add book");
    book
}

// =============================================================================
// delete_book_category
// =============================================================================

/// `delete_book_category` retires the aggregate (sets
/// `active_status = Retired`), bumps the version, and emits
/// a `BookCategoryDeleted` event with the right wire-form
/// `EVENT_TYPE` / `AGGREGATE_TYPE`.
#[test]
fn delete_book_category_retires_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut category = seed_category(&tenant, &g);
    let initial_version = category.version.get();

    let cmd = DeleteBookCategoryCommand {
        tenant: tenant.clone(),
        book_category_id: category.id,
    };
    let event = delete_book_category(cmd, &clock, &ids, &mut category).expect("delete");

    assert!(!category.active_status.is_active());
    assert_eq!(category.version.get(), initial_version + 1);
    assert_eq!(<BookCategoryDeleted as DomainEvent>::EVENT_TYPE, "library.book_category.deleted");
    assert_eq!(<BookCategoryDeleted as DomainEvent>::AGGREGATE_TYPE, "book_category");
    assert_eq!(event.aggregate_id(), category.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
}

/// A second `delete_book_category` against an already-retired
/// category is rejected with `DomainError::Conflict` (the
/// idempotency guard) — no version bump, no second event.
#[test]
fn delete_book_category_twice_returns_conflict() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut category = seed_category(&tenant, &g);

    let cmd = DeleteBookCategoryCommand {
        tenant: tenant.clone(),
        book_category_id: category.id,
    };
    delete_book_category(cmd.clone(), &clock, &ids, &mut category).expect("first delete");
    let err = delete_book_category(cmd, &clock, &ids, &mut category)
        .expect_err("second delete must conflict");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// update_book
// =============================================================================

/// `update_book` mutates the supplied fields on the
/// aggregate (bumps `version`, updates `updated_at` /
/// `updated_by`) and emits a `BookUpdated` event whose
/// `changes` list names the fields that actually moved.
#[test]
fn update_book_mutates_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let school = tenant.school_id;
    let category_id = BookCategoryId::new(school, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Original Title", 5);
    let initial_version = book.version.get();

    let cmd = UpdateBookCommand {
        tenant: tenant.clone(),
        book_id: book.id,
        book_title: Some(BookTitle::new("Renamed Title").expect("non-empty")),
        author_name: Some(Author::new("Jane Doe").expect("non-empty")),
        publisher_name: Some("O'Reilly".to_owned()),
        rack_number: None,
        book_price: None,
        details: None,
        book_category_id: None,
        book_subject_id: None,
    };
    let event = update_book(cmd, &clock, &ids, &mut book).expect("update");

    assert_eq!(book.book_title.as_str(), "Renamed Title");
    assert_eq!(book.author_name.as_ref().map(Author::as_str), Some("Jane Doe"));
    assert_eq!(book.version.get(), initial_version + 1);
    assert_eq!(<BookUpdated as DomainEvent>::EVENT_TYPE, "library.book.updated");
    assert_eq!(<BookUpdated as DomainEvent>::AGGREGATE_TYPE, "book");
    assert_eq!(event.aggregate_id(), book.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert!(event.changes.contains(&"book_title".to_owned()));
    assert!(event.changes.contains(&"author_name".to_owned()));
    assert!(event.changes.contains(&"publisher_name".to_owned()));
}

/// An `update_book` command whose id does not match the
/// aggregate's id returns `DomainError::Validation` and
/// leaves the aggregate untouched.
#[test]
fn update_book_with_mismatched_id_returns_validation_error() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let category_id = BookCategoryId::new(tenant.school_id, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Some Title", 1);
    let initial_version = book.version.get();

    let cmd = UpdateBookCommand {
        tenant: tenant.clone(),
        book_id: BookId::new(tenant.school_id, g.next_uuid()),
        book_title: Some(BookTitle::new("Anything").expect("non-empty")),
        author_name: None,
        publisher_name: None,
        rack_number: None,
        book_price: None,
        details: None,
        book_category_id: None,
        book_subject_id: None,
    };
    let err = update_book(cmd, &clock, &ids, &mut book).expect_err("mismatched id must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(book.version.get(), initial_version);
}

// =============================================================================
// delete_book
// =============================================================================

/// `delete_book` retires the aggregate, bumps the version,
/// and emits a `BookDeleted` event.
#[test]
fn delete_book_retires_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let category_id = BookCategoryId::new(tenant.school_id, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Some Title", 1);
    let initial_version = book.version.get();

    let cmd = DeleteBookCommand {
        tenant: tenant.clone(),
        book_id: book.id,
    };
    let event = delete_book(cmd, &clock, &ids, &mut book).expect("delete");

    assert!(!book.active_status.is_active());
    assert_eq!(book.version.get(), initial_version + 1);
    assert_eq!(<BookDeleted as DomainEvent>::EVENT_TYPE, "library.book.deleted");
    assert_eq!(<BookDeleted as DomainEvent>::AGGREGATE_TYPE, "book");
    assert_eq!(event.aggregate_id(), book.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
}

/// `delete_book` with a mismatched tenant returns
/// `DomainError::TenantViolation` and leaves the aggregate
/// untouched.
#[test]
fn delete_book_with_mismatched_tenant_returns_tenant_violation() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let other_g = SystemIdGen;
    let category_id = BookCategoryId::new(tenant.school_id, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Some Title", 1);
    let initial_version = book.version.get();

    let other_school = other_g.next_school_id();
    let cmd = DeleteBookCommand {
        tenant: TenantContext::for_user(
            other_school,
            other_g.next_user_id(),
            other_g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        book_id: book.id,
    };
    let err = delete_book(cmd, &clock, &ids, &mut book).expect_err("tenant mismatch must fail");
    assert!(
        matches!(err, DomainError::TenantViolation(_)),
        "expected TenantViolation, got {err:?}"
    );
    assert_eq!(book.version.get(), initial_version);
    assert!(book.active_status.is_active());
}

// =============================================================================
// adjust_book_quantity
// =============================================================================

/// `adjust_book_quantity` updates the stock count, bumps
/// the version, and emits a `BookQuantityAdjusted` event
/// with `from_quantity`, `to_quantity`, and the supplied
/// reason.
#[test]
fn adjust_book_quantity_updates_stock_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let category_id = BookCategoryId::new(tenant.school_id, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Some Title", 3);
    let initial_version = book.version.get();

    let cmd = AdjustBookQuantityCommand {
        tenant: tenant.clone(),
        book_id: book.id,
        new_quantity: StockCopies(7),
        reason: StockAdjustmentReason::Acquisition,
    };
    let event = adjust_book_quantity(cmd, &clock, &ids, &mut book).expect("adjust");

    assert_eq!(book.quantity, StockCopies(7));
    assert_eq!(book.version.get(), initial_version + 1);
    assert_eq!(
        <BookQuantityAdjusted as DomainEvent>::EVENT_TYPE,
        "library.book.quantity_adjusted"
    );
    assert_eq!(<BookQuantityAdjusted as DomainEvent>::AGGREGATE_TYPE, "book");
    assert_eq!(event.aggregate_id(), book.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.from_quantity, StockCopies(3));
    assert_eq!(event.to_quantity, StockCopies(7));
    assert_eq!(event.reason, StockAdjustmentReason::Acquisition);
}

/// A no-op `adjust_book_quantity` (new quantity equals
/// current quantity) returns `DomainError::Validation` and
/// leaves the aggregate untouched.
#[test]
fn adjust_book_quantity_noop_returns_validation_error() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let category_id = BookCategoryId::new(tenant.school_id, g.next_uuid());
    let mut book = seed_book(&tenant, &g, category_id, "Some Title", 4);
    let initial_version = book.version.get();

    let cmd = AdjustBookQuantityCommand {
        tenant: tenant.clone(),
        book_id: book.id,
        new_quantity: StockCopies(4),
        reason: StockAdjustmentReason::Stocktake,
    };
    let err = adjust_book_quantity(cmd, &clock, &ids, &mut book)
        .expect_err("noop adjustment must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(book.quantity, StockCopies(4));
    assert_eq!(book.version.get(), initial_version);
}
