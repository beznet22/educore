//! Integration tests for the **Homework aggregate** vertical slice.
//!
//! Pins the create contract for the `Homework` aggregate
//! end-to-end through the service layer:
//!
//! 1. `create_homework` validates the typed id belongs to the
//!    command's `school_id` (returning `DomainError::Validation`
//!    on a mismatch), constructs the aggregate, and emits a
//!    `HomeworkAssigned` event.
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
//! Note on `Homework` field set: the aggregate is a Phase 3
//! stub that carries only `id` (typed `HomeworkId`) and
//! `school_id`. The assignment fields (class-section, subject,
//! deadline, description, attachments) live in the full
//! `Homework` aggregate documented in
//! `docs/specs/academic/aggregates.md` § Homework and land in
//! a later phase. The service-layer function `create_homework`
//! exists now and is exercised here; its contract is to build
//! the stub aggregate and emit `HomeworkAssigned` with
//! matching typed ids.
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

fn homework_id(g: &SystemIdGen, school: SchoolId) -> HomeworkId {
    HomeworkId::new(school, g.next_uuid())
}

#[test]
fn homework_create_builds_aggregate_and_emits_assigned_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id = homework_id(&g, school);
    let create_cmd = CreateHomeworkCommand {
        id,
        school_id: school,
    };
    let (agg, assigned_event) = create_homework(create_cmd, &clock, &ids).expect("create");

    assert_eq!(agg.id, id);
    assert_eq!(agg.id.as_uuid(), id.as_uuid());
    assert_eq!(agg.id.school_id(), school);
    assert_eq!(agg.school_id, school);

    assert_eq!(
        <HomeworkAssigned as DomainEvent>::EVENT_TYPE,
        "academic.homework.assigned"
    );
    assert_eq!(<HomeworkAssigned as DomainEvent>::AGGREGATE_TYPE, "homework");
    assert_eq!(<HomeworkAssigned as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(assigned_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(assigned_event.school_id(), school);
    assert_eq!(assigned_event.aggregate_id, id);
}

#[test]
fn homework_create_is_idempotent_per_id_and_rejects_cross_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id_a = homework_id(&g, school);
    let id_b = homework_id(&g, school);
    assert_ne!(id_a, id_b);

    let (agg_a, event_a) = create_homework(
        CreateHomeworkCommand {
            id: id_a,
            school_id: school,
        },
        &clock,
        &ids,
    )
    .expect("create a");
    let (agg_b, event_b) = create_homework(
        CreateHomeworkCommand {
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

    let other_school = g.next_school_id();
    assert_ne!(other_school, school);
    let foreign_id = homework_id(&g, other_school);
    let err = create_homework(
        CreateHomeworkCommand {
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
