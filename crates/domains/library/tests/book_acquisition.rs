//! Integration tests for the **BookAcquisition aggregate** vertical slice.
//!
//! Pins the create + audit-footer contract for
//! [`BookAcquisition`](educore_library::prelude::BookAcquisition):
//!
//! 1. `BookAcquisition::new` constructs the child-entity
//!    projection of the aggregate root with `school_id`
//!    derived from the book id, carries the vendor /
//!    invoice / unit-cost / quantity / acquired-at fields
//!    from the call, and initialises the audit footer
//!    (`version = 1`, `created_by` / `updated_by` from the
//!    actor, `last_event_id = None`).
//! 2. The emitted
//!    [`BookAcquisitionRecorded`](educore_library::events::BookAcquisitionRecorded)
//!    event stub carries the right `event_type` /
//!    `aggregate_type` / `school_id`, with `aggregate_id`
//!    matching the typed id.
//!
//! The cluster-C service handler
//! (`record_book_acquisition`) is not yet implemented (TODO
//! stub); these tests pin the **entity-level** contract that
//! the handler will populate once it is wired. The aggregate
//! root in `aggregate.rs` (also named `BookAcquisition`) is
//! the first-class versioned root and shares its field
//! shape with this entity projection.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_library::events::BookAcquisitionRecorded;
use educore_library::prelude::*;
use educore_library::value_objects::BookAcquisitionId;

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

/// Construct a `NaiveDate` from parts, panicking on invalid
/// inputs (test fixture only).
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: create a BookAcquisition
// =============================================================================

/// End-to-end happy path for the BookAcquisition entity.
/// Construct a fresh `BookAcquisition` and assert that:
///
/// 1. The entity carries the vendor / invoice / unit-cost
///    / quantity / acquired-at fields from the constructor
///    call, with `school_id` derived from the book id, and
///    the audit footer initialised (`version = 1`,
///    `created_by` / `updated_by` from the actor,
///    `last_event_id = None`).
/// 2. The emitted `BookAcquisitionRecorded` event carries
///    the right `event_type` / `aggregate_type` / `school_id`
///    and `aggregate_id` matching the typed id.
#[test]
fn book_acquisition_new_populates_entity_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Construct the entity ----
    let book = book_id(&g, school);
    let unit_cost = BookPrice::new(2500).expect("non-negative price");
    let acquired_at = Some(date(2026, 1, 15));
    let now = clock.now();
    let correlation_id = tenant.correlation_id;

    let acquisition = BookAcquisition::new(
        book,
        "VendorCo".to_owned(),
        Some("INV-2026-0001".to_owned()),
        unit_cost,
        10,
        acquired_at,
        tenant.actor_id,
        now,
        correlation_id,
    );

    // Entity fields are populated from the constructor.
    assert_eq!(acquisition.school_id, school);
    assert_eq!(acquisition.book_id, book);
    assert_eq!(acquisition.vendor, "VendorCo");
    assert_eq!(acquisition.invoice_number.as_deref(), Some("INV-2026-0001"));
    assert_eq!(acquisition.unit_cost, unit_cost);
    assert_eq!(acquisition.quantity, 10);
    assert_eq!(acquisition.acquired_at, acquired_at);

    // Audit metadata footer is initialised.
    assert_eq!(acquisition.version.get(), 1);
    assert_eq!(acquisition.created_by, tenant.actor_id);
    assert_eq!(acquisition.updated_by, tenant.actor_id);
    assert_eq!(acquisition.last_event_id, None);
    assert_eq!(acquisition.correlation_id, correlation_id);

    // ---- Emit the event ----
    let typed_id = BookAcquisitionId::new(school, g.next_uuid());
    let event_id = ids.next_event_id();
    let event = BookAcquisitionRecorded::new(typed_id, event_id, correlation_id, now);
    assert_eq!(
        <BookAcquisitionRecorded as DomainEvent>::EVENT_TYPE,
        "library.book_acquisition.recorded"
    );
    assert_eq!(
        <BookAcquisitionRecorded as DomainEvent>::AGGREGATE_TYPE,
        "book_acquisition"
    );
    assert_eq!(<BookAcquisitionRecorded as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), typed_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.id, typed_id);
}

// =============================================================================
// Happy path: quantity arithmetic and audit invariants
// =============================================================================

/// End-to-end happy path for the BookAcquisition arithmetic
/// invariants. Construct two acquisitions for the same book
/// with different unit-cost / quantity, asserting that:
///
/// 1. Each acquisition carries its own independent
///    vendor / unit-cost / quantity fields (no shared state).
/// 2. The sum of `quantity` across the two acquisitions
///    equals the total `quantity` acquired — the
///    arithmetic invariant a downstream
///    `sum_acquisitions_for_book` projection can rely on.
/// 3. The audit footer's `updated_at` equals `created_at` on
///    a fresh entity (the constructor does not bump the
///    timestamp after `new`).
#[test]
fn book_acquisition_quantity_invariants_and_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();

    let book = book_id(&g, school);
    let now = clock.now();
    let correlation_id = tenant.correlation_id;

    let first = BookAcquisition::new(
        book,
        "VendorA".to_owned(),
        Some("INV-A".to_owned()),
        BookPrice::new(2500).expect("non-negative price"),
        10,
        Some(date(2026, 1, 15)),
        tenant.actor_id,
        now,
        correlation_id,
    );

    let second = BookAcquisition::new(
        book,
        "VendorB".to_owned(),
        Some("INV-B".to_owned()),
        BookPrice::new(3000).expect("non-negative price"),
        5,
        Some(date(2026, 2, 1)),
        tenant.actor_id,
        now,
        correlation_id,
    );

    // Independent vendor / cost / quantity fields.
    assert_eq!(first.vendor, "VendorA");
    assert_eq!(second.vendor, "VendorB");
    assert_eq!(first.unit_cost.value(), 2500);
    assert_eq!(second.unit_cost.value(), 3000);

    // Arithmetic invariant: sum of quantities is the total.
    let total_quantity = first.quantity + second.quantity;
    assert_eq!(total_quantity, 15);

    // Audit footer: created_at == updated_at on a fresh
    // entity.
    assert_eq!(first.created_at, first.updated_at);
    assert_eq!(second.created_at, second.updated_at);

    // Both entity schools are derived from the same book id.
    assert_eq!(first.school_id, school);
    assert_eq!(second.school_id, school);
}
