//! Integration tests for the **Supplier aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`Supplier`](educore_facilities::aggregate::Supplier)
//! end-to-end through the service layer:
//!
//! 1. `create_supplier` validates the input (the typed
//!    [`SupplierName`](educore_facilities::value_objects::SupplierName)
//!    enforces non-empty + length bounds at command
//!    construction), constructs the aggregate (with an
//!    initial `Active` status), and emits a
//!    [`SupplierCreated`] event.
//! 2. `update_supplier` mutates the in-place aggregate
//!    (bumps `version`, swaps `company_name`, updates
//!    `updated_at` / `updated_by`) and emits a
//!    [`SupplierUpdated`] event whose `changes` list names
//!    the fields that actually moved.
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
use educore_facilities::services::{create_supplier, update_supplier};

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
// Happy path: create + update on Supplier
// =============================================================================

/// End-to-end happy path for the Supplier aggregate. Create
/// a supplier "Acme Corp" (with a contact person), then
/// update the company name to "Acme Holdings" and change the
/// contact email, asserting that:
///
/// 1. The create flow produces a `Supplier` aggregate
///    carrying every field on the command (school id derived
///    from the typed id) and emits a `SupplierCreated` event
///    with the right `event_type`, `aggregate_type`, and
///    `school_id`.
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `company_name`, updates `updated_at` /
///    `updated_by`) and emits a `SupplierUpdated` event
///    whose `changes` list names the fields that actually
///    moved.
#[test]
fn supplier_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateSupplierCommand {
        tenant: tenant.clone(),
        company_name: SupplierName::new("Acme Corp").expect("non-empty supplier name"),
        company_address: Some(Address::new("123 Main St").expect("non-empty address")),
        contact_person_name: Some(
            ContactPersonName::new("Jane Doe").expect("non-empty contact name"),
        ),
        contact_person_mobile: Some(
            PhoneNumber::new("+15551234567").expect("valid phone number"),
        ),
        contact_person_email: Some(
            EmailAddress::new("jane@acme.example").expect("valid email"),
        ),
        contact_person_address: None,
        description: Some(Description::new("Primary stationery vendor").expect("non-empty")),
    };
    let (mut supplier, created_event) =
        create_supplier(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(supplier.school_id, school);
    assert_eq!(supplier.company_name.as_str(), "Acme Corp");
    assert_eq!(supplier.status, SupplierStatus::Active);
    assert_eq!(
        supplier
            .contact_person_email
            .as_ref()
            .map(EmailAddress::as_str),
        Some("jane@acme.example")
    );
    assert_eq!(supplier.created_by, tenant.actor_id);
    assert_eq!(supplier.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised.
    assert_eq!(supplier.version.get(), 1);
    assert!(supplier.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <SupplierCreated as DomainEvent>::EVENT_TYPE,
        "facilities.supplier.created"
    );
    assert_eq!(
        <SupplierCreated as DomainEvent>::AGGREGATE_TYPE,
        "supplier"
    );
    assert_eq!(<SupplierCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), supplier.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.company_name, "Acme Corp");

    // ---- Update flow ----
    let initial_version = supplier.version.get();
    let update_cmd = UpdateSupplierCommand {
        tenant: tenant.clone(),
        supplier_id: supplier.id,
        company_name: Some(SupplierName::new("Acme Holdings").expect("non-empty supplier name")),
        company_address: None,
        contact_person_name: None,
        contact_person_mobile: None,
        contact_person_email: Some(
            EmailAddress::new("ops@acme-holdings.example").expect("valid email"),
        ),
        contact_person_address: None,
        description: None,
    };
    let updated_event =
        update_supplier(&mut supplier, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(supplier.company_name.as_str(), "Acme Holdings");
    assert_eq!(
        supplier
            .contact_person_email
            .as_ref()
            .map(EmailAddress::as_str),
        Some("ops@acme-holdings.example")
    );
    assert_eq!(supplier.version.get(), initial_version + 1);
    assert_eq!(supplier.updated_by, tenant.actor_id);
    assert!(supplier.active_status.is_active());

    // The event names the fields that actually moved.
    assert_eq!(
        <SupplierUpdated as DomainEvent>::EVENT_TYPE,
        "facilities.supplier.updated"
    );
    assert_eq!(
        <SupplierUpdated as DomainEvent>::AGGREGATE_TYPE,
        "supplier"
    );
    assert_eq!(<SupplierUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), supplier.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(
        updated_event.changes,
        vec!["company_name".to_owned(), "contact_person_email".to_owned()]
    );
}

// =============================================================================
// Validation failure: empty supplier name is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `company_name` is empty, the typed
/// [`SupplierName::new`] constructor returns
/// `DomainError::Validation` before the service factory is
/// ever invoked (and therefore no event is minted).
#[test]
fn supplier_create_with_empty_name_returns_validation_error() {
    let err = SupplierName::new("").expect_err("empty supplier name must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
