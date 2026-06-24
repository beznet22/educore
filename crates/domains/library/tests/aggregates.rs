//! Integration tests for the **BookCategory aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`BookCategory`](educore_library::aggregate::BookCategory)
//! end-to-end through the service layer:
//!
//! 1. `create_book_category` validates the input (the typed
//!    [`CategoryName`](educore_library::value_objects::CategoryName)
//!    enforces non-empty + length bounds at command
//!    construction), constructs the aggregate, and emits a
//!    [`BookCategoryCreated`] event.
//! 2. `update_book_category` validates the in-place aggregate
//!    (id + school match the command; new name differs from
//!    the current name), mutates the aggregate (bumps
//!    `version`, updates `updated_at` / `updated_by`), and
//!    emits a [`BookCategoryUpdated`] event whose `changes`
//!    list names the field that actually moved.
//!
//! The tests use the same fixture pattern as
//! `tests/workflows.rs` (`TestClock` + `SystemIdGen`). The
//! handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **service layer** contract
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

use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_library::prelude::*;
use educore_library::services::update_book_category;

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

// =============================================================================
// Happy path: create + update on BookCategory
// =============================================================================

/// End-to-end happy path for the BookCategory aggregate.
/// Create a category called "Fiction", then rename it to
/// "Reference", asserting that:
///
/// 1. The create flow produces a `BookCategory` aggregate
///    carrying every field on the command (school id derived
///    from the typed id) and emits a `BookCategoryCreated`
///    event with the right `event_type`, `aggregate_type`,
///    and `school_id`.
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `category_name`, updates
///    `updated_at` / `updated_by`) and emits a
///    `BookCategoryUpdated` event whose `changes` list names
///    the field that actually moved.
#[test]
fn book_category_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateBookCategoryCommand {
        tenant: tenant.clone(),
        category_name: CategoryName::new("Fiction").expect("non-empty name"),
    };
    let (mut category, created_event) =
        create_book_category(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(category.school_id, school);
    assert_eq!(category.category_name.as_str(), "Fiction");
    assert_eq!(category.created_by, tenant.actor_id);
    assert_eq!(category.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised.
    assert_eq!(category.version.get(), 1);
    assert!(category.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <BookCategoryCreated as DomainEvent>::EVENT_TYPE,
        "library.book_category.created"
    );
    assert_eq!(
        <BookCategoryCreated as DomainEvent>::AGGREGATE_TYPE,
        "book_category"
    );
    assert_eq!(<BookCategoryCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), category.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.category_name.as_str(), "Fiction");

    // ---- Update flow ----
    let initial_version = category.version.get();
    let update_cmd = UpdateBookCategoryCommand {
        tenant: tenant.clone(),
        book_category_id: category.id,
        new_name: CategoryName::new("Reference").expect("non-empty name"),
    };
    let updated_event =
        update_book_category(update_cmd, &clock, &ids, &mut category).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(category.category_name.as_str(), "Reference");
    assert_eq!(category.version.get(), initial_version + 1);
    assert_eq!(category.updated_by, tenant.actor_id);
    assert!(category.active_status.is_active());

    // The event names the field that actually moved.
    assert_eq!(
        <BookCategoryUpdated as DomainEvent>::EVENT_TYPE,
        "library.book_category.updated"
    );
    assert_eq!(
        <BookCategoryUpdated as DomainEvent>::AGGREGATE_TYPE,
        "book_category"
    );
    assert_eq!(<BookCategoryUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), category.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(updated_event.changes, vec!["category_name".to_owned()]);
}

// =============================================================================
// Validation failure: no-op rename is rejected
// =============================================================================

/// Validation-failure path on the update flow: when the
/// `new_name` matches the current `category_name`,
/// `update_book_category` returns `DomainError::Validation`
/// and the aggregate is left unchanged (no version bump,
/// no event minted).
#[test]
fn book_category_update_with_same_name_returns_validation_error() {
    let (tenant, _g) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Seed a BookCategory via the create flow.
    let create_cmd = CreateBookCategoryCommand {
        tenant: tenant.clone(),
        category_name: CategoryName::new("Fiction").expect("non-empty name"),
    };
    let (mut category, _created_event) =
        create_book_category(create_cmd, &clock, &ids).expect("create");
    let initial_version = category.version.get();
    let initial_updated_at = category.updated_at;
    let initial_updated_by = category.updated_by;

    // Attempt to rename to the SAME name — must fail with
    // Validation and leave the aggregate untouched.
    let noop_update = UpdateBookCategoryCommand {
        tenant: tenant.clone(),
        book_category_id: category.id,
        new_name: CategoryName::new("Fiction").expect("non-empty name"),
    };
    let err = update_book_category(noop_update, &clock, &ids, &mut category)
        .expect_err("noop rename must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The aggregate is unchanged: a failed update must not
    // bump the optimistic-concurrency counter, change the
    // name, or move the audit timestamps.
    assert_eq!(category.version.get(), initial_version);
    assert_eq!(category.updated_at, initial_updated_at);
    assert_eq!(category.updated_by, initial_updated_by);
    assert_eq!(category.category_name.as_str(), "Fiction");
}
