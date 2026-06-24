//! Integration tests for the **Vehicle aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`Vehicle`](educore_facilities::aggregate::Vehicle)
//! end-to-end through the service layer:
//!
//! 1. `create_vehicle` validates the input (the typed
//!    [`VehicleNumber`](educore_facilities::value_objects::VehicleNumber)
//!    enforces non-empty + length bounds + alphanumeric+dash at
//!    construction), constructs the aggregate, and emits a
//!    [`VehicleCreated`] event.
//! 2. `update_vehicle` mutates the in-place aggregate (bumps
//!    `version`, swaps `vehicle_model`, updates `updated_at` /
//!    `updated_by`) and emits a [`VehicleUpdated`] event whose
//!    `changes` list names the field that actually moved.
//!
//! The tests use the same fixture pattern as
//! `tests/workflows.rs` (`TestClock` + `SystemIdGen`). The
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
use educore_facilities::services::{create_vehicle, update_vehicle};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same school.
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

/// Mint a typed `AcademicYearId` for the given school.
fn academic_year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: create + update on Vehicle
// =============================================================================

/// End-to-end happy path for the Vehicle aggregate. Create a
/// vehicle "V001" (model "Toyota"), then update the model to
/// "Honda", asserting that:
///
/// 1. The create flow produces a `Vehicle` aggregate carrying
///    every field on the command (school id derived from the
///    typed id) and emits a `VehicleCreated` event with the
///    right `event_type`, `aggregate_type`, and `school_id`.
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `vehicle_model`, updates `updated_at` /
///    `updated_by`) and emits a `VehicleUpdated` event whose
///    `changes` list names the field that actually moved.
#[test]
fn vehicle_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateVehicleCommand {
        tenant: tenant.clone(),
        academic_year_id: academic_year_id(&g, school),
        vehicle_no: VehicleNumber::new("V001").expect("non-empty vehicle number"),
        vehicle_model: VehicleModel::new("Toyota").expect("non-empty model"),
        made_year: None,
        driver_id: None,
        note: None,
    };
    let (mut vehicle, created_event) =
        create_vehicle(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(vehicle.school_id, school);
    assert_eq!(vehicle.vehicle_no.as_str(), "V001");
    assert_eq!(vehicle.vehicle_model.as_str(), "Toyota");
    assert_eq!(vehicle.created_by, tenant.actor_id);
    assert_eq!(vehicle.updated_by, tenant.actor_id);
    assert_eq!(vehicle.status, VehicleStatus::Active);
    // Audit metadata footer is initialised.
    assert_eq!(vehicle.version.get(), 1);
    assert!(vehicle.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <VehicleCreated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.created"
    );
    assert_eq!(
        <VehicleCreated as DomainEvent>::AGGREGATE_TYPE,
        "vehicle"
    );
    assert_eq!(<VehicleCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), vehicle.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.vehicle_no.as_str(), "V001");
    assert_eq!(created_event.vehicle_model, "Toyota");
    assert!(created_event.driver_id.is_none());

    // ---- Update flow ----
    let initial_version = vehicle.version.get();
    let update_cmd = UpdateVehicleCommand {
        tenant: tenant.clone(),
        vehicle_id: vehicle.id,
        vehicle_model: Some(VehicleModel::new("Honda").expect("non-empty model")),
        made_year: None,
        status: None,
        note: None,
    };
    let updated_event =
        update_vehicle(&mut vehicle, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(vehicle.vehicle_model.as_str(), "Honda");
    assert_eq!(vehicle.version.get(), initial_version + 1);
    assert_eq!(vehicle.updated_by, tenant.actor_id);
    assert!(vehicle.active_status.is_active());

    // The event names the field that actually moved.
    assert_eq!(
        <VehicleUpdated as DomainEvent>::EVENT_TYPE,
        "facilities.vehicle.updated"
    );
    assert_eq!(
        <VehicleUpdated as DomainEvent>::AGGREGATE_TYPE,
        "vehicle"
    );
    assert_eq!(<VehicleUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(updated_event.aggregate_id(), vehicle.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(updated_event.changes, vec!["vehicle_model".to_owned()]);
}

// =============================================================================
// Validation failure: empty vehicle_number is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `vehicle_no` is empty, the typed
/// [`VehicleNumber::new`](educore_facilities::value_objects::VehicleNumber::new)
/// constructor returns `DomainError::Validation` before the
/// service factory is ever invoked (and therefore no event is
/// minted).
#[test]
fn vehicle_create_with_empty_vehicle_number_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    // The typed `VehicleNumber::new("")` is the validation
    // gate; it rejects empty input with `DomainError::Validation`.
    let err = VehicleNumber::new("").expect_err("empty vehicle number must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The service factory is never called in this path: a
    // failed validation on the typed id means the command
    // cannot even be constructed, so no aggregate is built
    // and no event is emitted. Verify the school id we
    // minted is sane (i.e. the test setup worked) and the
    // `AcademicYearId` plumbing is in scope.
    assert_ne!(school, educore_core::ids::SchoolId(uuid::Uuid::nil()));
    let _ = academic_year_id(&g, school);
}
