//! Integration tests for the **ClassRoutine aggregate** vertical slice.
//!
//! Pins the create contract for the `ClassRoutine` aggregate
//! end-to-end through the service layer:
//!
//! 1. `create_class_routine` validates the typed id belongs to the
//!    command's `school_id` (returning `DomainError::Validation`
//!    on a mismatch), constructs the aggregate, and emits a
//!    `ClassRoutineScheduled` event.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs` and `subject.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber fan-out,
//! no outbox commit, no audit row). These tests pin the
//! contract of the **service layer** that the dispatcher will
//! eventually wrap.
//!
//! Note on `ClassRoutine` field set: the aggregate is a Phase 3
//! stub that carries only `id` (typed `ClassRoutineId`) and
//! `school_id`. The weekly-schedule fields (day, period,
//! section, subject, teacher, room) live in the full
//! `ClassRoutine` aggregate documented in
//! `docs/specs/academic/aggregates.md` § ClassRoutine and land
//! in a later phase. The service-layer function
//! `create_class_routine` exists now and is exercised here;
//! its contract is to build the stub aggregate and emit
//! `ClassRoutineScheduled` with matching typed ids.
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic test suites.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;

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

fn class_routine_id(g: &SystemIdGen, school: SchoolId) -> ClassRoutineId {
    ClassRoutineId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create a ClassRoutine
// =============================================================================

/// End-to-end happy path for the `ClassRoutine` aggregate.
/// Schedule a routine for the freshly-minted school,
/// asserting that:
///
/// 1. The create flow produces a `ClassRoutine` aggregate
///    carrying the typed `id` and `school_id` from the
///    command, plus a `ClassRoutineScheduled` event with the
///    right `event_type`, `aggregate_type`, and `school_id`.
/// 2. The event's typed `aggregate_id` matches the
///    aggregate's `id`, the school's `SchoolId` is propagated
///    onto both the aggregate and the event, and a distinct
///    `event_id` is minted per call.
#[test]
fn class_routine_create_builds_aggregate_and_emits_scheduled_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let id = class_routine_id(&g, school);
    let create_cmd = CreateClassRoutineCommand {
        id,
        school_id: school,
    };
    let (agg, scheduled_event) =
        create_class_routine(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.id, id);
    assert_eq!(agg.id.as_uuid(), id.as_uuid());
    assert_eq!(agg.id.school_id(), school);
    assert_eq!(agg.school_id, school);

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <ClassRoutineScheduled as DomainEvent>::EVENT_TYPE,
        "academic.class_routine.scheduled"
    );
    assert_eq!(
        <ClassRoutineScheduled as DomainEvent>::AGGREGATE_TYPE,
        "class_routine"
    );
    assert_eq!(<ClassRoutineScheduled as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(scheduled_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(scheduled_event.school_id(), school);
    assert_eq!(scheduled_event.aggregate_id, id);
}

// =============================================================================
// 2. Multi-tenant happy path: scheduling a second routine
// =============================================================================

/// Second happy path: confirm the create flow is repeatable
/// for the same school without leaking state between
/// invocations. Two routines minted against the same school
/// produce two independent aggregates + two independent
/// `ClassRoutineScheduled` events with distinct typed ids
/// and distinct `event_id`s.
///
/// Also pins the cross-school guard: a `ClassRoutineId`
/// minted for school A combined with a `school_id` of
/// school B on the command must fail with
/// `DomainError::Validation` (the id's `school_id()` must
/// match the command's `school_id`). The success path on the
/// same school proves the validation is keyed on the
/// mismatch, not on the typed id itself.
#[test]
fn class_routine_create_is_idempotent_per_id_and_rejects_cross_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Two routines for the same school.
    let id_a = class_routine_id(&g, school);
    let id_b = class_routine_id(&g, school);
    assert_ne!(id_a, id_b, "fresh uuids must differ");

    let (agg_a, event_a) = create_class_routine(
        CreateClassRoutineCommand {
            id: id_a,
            school_id: school,
        },
        &clock,
        &ids,
    )
    .expect("create a");
    let (agg_b, event_b) = create_class_routine(
        CreateClassRoutineCommand {
            id: id_b,
            school_id: school,
        },
        &clock,
        &ids,
    )
    .expect("create b");

    assert_eq!(agg_a.id, id_a);
    assert_eq!(agg_b.id, id_b);
    assert_eq!(agg_a.school_id, school);
    assert_eq!(agg_b.school_id, school);
    assert_eq!(event_a.aggregate_id, id_a);
    assert_eq!(event_b.aggregate_id, id_b);
    assert_ne!(event_a.event_id, event_b.event_id);

    // Cross-school guard: a routine id minted for a
    // different school must be rejected.
    let other_school = g.next_school_id();
    assert_ne!(other_school, school);
    let foreign_id = class_routine_id(&g, other_school);
    let err = create_class_routine(
        CreateClassRoutineCommand {
            id: foreign_id,
            school_id: school,
        },
        &clock,
        &ids,
    )
    .expect_err("cross-school id must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
