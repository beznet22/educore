//! Integration tests for the **SeatPlan aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`SeatPlan`](educore_assessment::aggregate::SeatPlan)
//! end-to-end through the service layer:
//!
//! 1. `generate_seat_plan` validates the per-room
//!    [`SeatPlanAllocation`](educore_assessment::commands::SeatPlanAllocation)
//!    entries (time windows must be well-formed), sums
//!    `assign_students` across allocations to derive
//!    `total_students`, constructs the aggregate via
//!    [`SeatPlan::fresh`](educore_assessment::aggregate::SeatPlan::fresh),
//!    and emits a [`SeatPlanGenerated`](educore_assessment::events::SeatPlanGenerated)
//!    event.
//!
//! 2. `update_seat_plan` applies the new `total_students`
//!    (when supplied and different), records the
//!    `changes` vector, bumps the aggregate's `version`,
//!    refreshes `updated_at` / `updated_by`, and emits a
//!    [`SeatPlanUpdated`](educore_assessment::events::SeatPlanUpdated)
//!    event.
//!
//! The tests use the same fixture pattern as
//! `tests/exam.rs` in the library crate (`TestClock` +
//! `SystemIdGen`). The **handlers** themselves are not wired
//! end-to-end (no subscriber fan-out, no outbox commit, no
//! audit row). These tests pin the contract of the **service
//! layer** that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/assessment/tests/exam.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{GenerateSeatPlanCommand, SeatPlanAllocation, UpdateSeatPlanCommand};
use educore_assessment::events::{SeatPlanGenerated, SeatPlanUpdated};
use educore_assessment::services::{generate_seat_plan, update_seat_plan};
use educore_assessment::value_objects::{ClassRoomId, ExamId, SeatPlanId, SectionId};
use educore_core::clock::{IdGenerator, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::ActiveStatus;
use educore_events::domain_event::DomainEvent;

use educore_academic::value_objects::ClassId;

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

/// Two `SeatPlanAllocation` rows summing to 60 students —
/// `Room 101` for 30 students from 09:00 to 11:00, and
/// `Room 102` for 30 students from 11:30 to 13:30.
fn sample_allocations(g: &SystemIdGen, school: SchoolId) -> Vec<SeatPlanAllocation> {
    vec![
        SeatPlanAllocation {
            room_id: ClassRoomId::new(school, g.next_uuid()),
            assign_students: 30,
            start_time: chrono::NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(11, 0, 0).expect("valid time"),
        },
        SeatPlanAllocation {
            room_id: ClassRoomId::new(school, g.next_uuid()),
            assign_students: 30,
            start_time: chrono::NaiveTime::from_hms_opt(11, 30, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(13, 30, 0).expect("valid time"),
        },
    ]
}

// =============================================================================
// Happy path: generate_seat_plan end-to-end
// =============================================================================

/// End-to-end happy path for the `SeatPlan` aggregate.
/// Generates a seat plan for a section with 60 students
/// split across two rooms, asserting that:
///
/// 1. The create flow produces a `SeatPlan` aggregate
///    carrying every field on the command, with
///    `total_students == 60` and a fresh `version = 1`.
/// 2. The aggregate is anchored to the tenant's school and
///    starts in the `Active` state.
/// 3. The emitted event is `SeatPlanGenerated` with the
///    right `event_type`, `aggregate_type`, `school_id`,
///    and `schema_version`.
#[test]
fn seat_plan_generate_produces_aggregate_and_seat_plan_generated_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = GenerateSeatPlanCommand {
        tenant: tenant.clone(),
        seat_plan_id: SeatPlanId::new(school, g.next_uuid()),
        exam_id: ExamId::new(school, g.next_uuid()),
        class_id: ClassId::new(school, g.next_uuid()),
        section_id: SectionId::new(school, g.next_uuid()),
        allocations: sample_allocations(&g, school),
    };

    let (plan, event) = generate_seat_plan(cmd, &clock, &ids).expect("generate_seat_plan");

    // Aggregate fields are populated from the command.
    assert_eq!(plan.school_id, school);
    assert_eq!(plan.total_students, 60);
    // Audit metadata footer is initialised.
    assert_eq!(plan.version.get(), 1);
    assert!(plan.active_status.is_active());
    assert_eq!(plan.created_by, tenant.actor_id);
    assert_eq!(plan.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <SeatPlanGenerated as DomainEvent>::EVENT_TYPE,
        "assessment.seat_plan.generated"
    );
    assert_eq!(
        <SeatPlanGenerated as DomainEvent>::AGGREGATE_TYPE,
        "seat_plan"
    );
    assert_eq!(<SeatPlanGenerated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), plan.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.total_students, 60);
}

// =============================================================================
// Update path: update_seat_plan mutates and emits SeatPlanUpdated
// =============================================================================

/// End-to-end happy path for the `SeatPlan` update flow.
/// Generates a seat plan with 60 students, then updates it
/// to 75 students (3-room allocation), asserting that:
///
/// 1. The aggregate's `total_students` flips to 75 and
///    `version` increments to 2.
/// 2. `updated_by` is the new actor and `updated_at`
///    advances.
/// 3. The emitted event is `SeatPlanUpdated` with the
///    correct `event_type`, `aggregate_type`,
///    `schema_version`, and `changes == ["total_students"]`.
#[test]
fn seat_plan_update_mutates_aggregate_and_emits_seat_plan_updated_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = GenerateSeatPlanCommand {
        tenant: tenant.clone(),
        seat_plan_id: SeatPlanId::new(school, g.next_uuid()),
        exam_id: ExamId::new(school, g.next_uuid()),
        class_id: ClassId::new(school, g.next_uuid()),
        section_id: SectionId::new(school, g.next_uuid()),
        allocations: sample_allocations(&g, school),
    };

    let (mut plan, _generated) = generate_seat_plan(cmd, &clock, &ids).expect("generate_seat_plan");
    assert_eq!(plan.total_students, 60);
    assert_eq!(plan.version.get(), 1);

    // Build a new 3-room allocation totalling 75 students.
    let new_allocations = vec![
        SeatPlanAllocation {
            room_id: ClassRoomId::new(school, g.next_uuid()),
            assign_students: 25,
            start_time: chrono::NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(11, 0, 0).expect("valid time"),
        },
        SeatPlanAllocation {
            room_id: ClassRoomId::new(school, g.next_uuid()),
            assign_students: 25,
            start_time: chrono::NaiveTime::from_hms_opt(11, 30, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(13, 30, 0).expect("valid time"),
        },
        SeatPlanAllocation {
            room_id: ClassRoomId::new(school, g.next_uuid()),
            assign_students: 25,
            start_time: chrono::NaiveTime::from_hms_opt(14, 0, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(16, 0, 0).expect("valid time"),
        },
    ];

    let update_cmd = UpdateSeatPlanCommand {
        tenant: tenant.clone(),
        seat_plan_id: plan.id,
        allocations: Some(new_allocations),
    };

    let event = update_seat_plan(&mut plan, update_cmd, &clock, &ids)
        .expect("update_seat_plan");

    // Aggregate mutated: total_students flipped, version bumped,
    // updated_by refreshed.
    assert_eq!(plan.total_students, 75);
    assert_eq!(plan.version.get(), 2);
    assert_eq!(plan.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <SeatPlanUpdated as DomainEvent>::EVENT_TYPE,
        "assessment.seat_plan.updated"
    );
    assert_eq!(
        <SeatPlanUpdated as DomainEvent>::AGGREGATE_TYPE,
        "seat_plan"
    );
    assert_eq!(<SeatPlanUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), plan.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.changes, vec!["total_students".to_owned()]);
}

// =============================================================================
// Validation failure: update with no changes
// =============================================================================

/// Validation-failure path on the update flow: when
/// `update_seat_plan` is invoked with `allocations: None`
/// (i.e. no changes supplied), the service returns
/// [`DomainError::Validation`] before any state change or
/// event minting happens. The aggregate is left untouched
/// (still at `version == 1`).
#[test]
fn seat_plan_update_rejects_when_no_changes_supplied() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = GenerateSeatPlanCommand {
        tenant: tenant.clone(),
        seat_plan_id: SeatPlanId::new(school, g.next_uuid()),
        exam_id: ExamId::new(school, g.next_uuid()),
        class_id: ClassId::new(school, g.next_uuid()),
        section_id: SectionId::new(school, g.next_uuid()),
        allocations: sample_allocations(&g, school),
    };

    let (mut plan, _generated) = generate_seat_plan(cmd, &clock, &ids).expect("generate_seat_plan");

    let update_cmd = UpdateSeatPlanCommand {
        tenant,
        seat_plan_id: plan.id,
        allocations: None, // no changes supplied — must fail
    };

    let err = update_seat_plan(&mut plan, update_cmd, &clock, &ids)
        .expect_err("update with no changes must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    // Aggregate remains untouched.
    assert_eq!(plan.total_students, 60);
    assert_eq!(plan.version.get(), 1);
}
