//! Integration tests for the **ItemStore aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`ItemStore`](educore_facilities::aggregate::ItemStore)
//! end-to-end through the service layer:
//!
//! 1. `create_item_store` validates the input (the typed
//!    [`StoreName`](educore_facilities::value_objects::StoreName)
//!    enforces non-empty + length bounds at command
//!    construction), constructs the aggregate, and emits an
//!    [`ItemStoreCreated`] event.
//! 2. `update_item_store` mutates the in-place aggregate
//!    (bumps `version`, swaps `store_name`, updates
//!    `updated_at` / `updated_by`) and emits an
//!    [`ItemStoreUpdated`] event whose `changes` list names
//!    the field that actually moved.
//!
//! The tests use the same fixture pattern as
//! `tests/vehicle.rs` (`TestClock` + `SystemIdGen`). The
//! handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **service layer** contract
//! that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` and
//! `crates/domains/attendance/tests/aggregates.rs` (lean).

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
use educore_facilities::prelude::*;
use educore_facilities::services::{create_item_store, update_item_store};

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
// Happy path: create + update on ItemStore
// =============================================================================

/// End-to-end happy path for the ItemStore aggregate. Create
/// a store "Main Warehouse" (no number), then update the name
/// to "North Warehouse" and add a store number, asserting
/// that:
///
/// 1. The create flow produces an `ItemStore` aggregate
///    carrying every field on the command (school id derived
///    from the typed id) and emits an `ItemStoreCreated`
///    event with the right `event_type`, `aggregate_type`,
///    and `school_id`.
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `store_name`, updates `updated_at` /
///    `updated_by`) and emits an `ItemStoreUpdated` event
///    whose `changes` list names the fields that actually
///    moved.
#[test]
fn item_store_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateItemStoreCommand {
        tenant: tenant.clone(),
        store_name: StoreName::new("Main Warehouse").expect("non-empty store name"),
        store_number: None,
        description: None,
    };
    let (mut store, created_event) =
        create_item_store(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(store.school_id, school);
    assert_eq!(store.store_name.as_str(), "Main Warehouse");
    assert!(store.store_number.is_none());
    assert_eq!(store.created_by, tenant.actor_id);
    assert_eq!(store.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised.
    assert_eq!(store.version.get(), 1);
    assert!(store.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <ItemStoreCreated as DomainEvent>::EVENT_TYPE,
        "facilities.item_store.created"
    );
    assert_eq!(
        <ItemStoreCreated as DomainEvent>::AGGREGATE_TYPE,
        "item_store"
    );
    assert_eq!(<ItemStoreCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), store.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.store_name, "Main Warehouse");

    // ---- Update flow ----
    let initial_version = store.version.get();
    let update_cmd = UpdateItemStoreCommand {
        tenant: tenant.clone(),
        item_store_id: store.id,
        store_name: Some(StoreName::new("North Warehouse").expect("non-empty store name")),
        store_number: Some(StoreNumber::new("WH-01").expect("non-empty store number")),
        description: Some(Description::new("Cold storage").expect("non-empty description")),
    };
    let updated_event =
        update_item_store(&mut store, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(store.store_name.as_str(), "North Warehouse");
    assert_eq!(store.store_number.as_ref().map(StoreNumber::as_str), Some("WH-01"));
    assert_eq!(
        store.description.as_ref().map(Description::as_str),
        Some("Cold storage")
    );
    assert_eq!(store.version.get(), initial_version + 1);
    assert_eq!(store.updated_by, tenant.actor_id);
    assert!(store.active_status.is_active());

    // The event names the fields that actually moved.
    assert_eq!(
        <ItemStoreUpdated as DomainEvent>::EVENT_TYPE,
        "facilities.item_store.updated"
    );
    assert_eq!(
        <ItemStoreUpdated as DomainEvent>::AGGREGATE_TYPE,
        "item_store"
    );
    assert_eq!(<ItemStoreUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), store.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(
        updated_event.changes,
        vec![
            "store_name".to_owned(),
            "store_number".to_owned(),
            "description".to_owned(),
        ]
    );
}

// =============================================================================
// Validation failure: empty store_name is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `store_name` is empty, the typed
/// [`StoreName::new`] constructor returns
/// `DomainError::Validation` before the service factory is
/// ever invoked (and therefore no event is minted).
#[test]
fn item_store_create_with_empty_name_returns_validation_error() {
    // The typed `StoreName::new("")` is the validation gate;
    // it rejects empty input with `DomainError::Validation`.
    let err = StoreName::new("").expect_err("empty store name must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
