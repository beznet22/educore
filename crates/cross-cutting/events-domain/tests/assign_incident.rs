//! Integration tests for the **AssignIncident aggregate** vertical slice.
//!
//! Pins the constructor + update contract for
//! [`AssignIncident`](educore_events_domain::aggregate::AssignIncident)
//! end-to-end through the aggregate layer:
//!
//! 1. `AssignIncident::new(...)` validates the input (exactly one
//!    of `student_id` / `user_id` must be set, and `point` must
//!    be in `0..=1000`), constructs the aggregate with
//!    `version = Version::initial()` and `active_status = true`,
//!    and propagates the typed id's `school_id` to the
//!    denormalised field.
//! 2. `AssignIncident::reassign(...)` validates the new `point`
//!    value (`0..=1000`), mutates the aggregate (swaps `point`,
//!    bumps `version`, updates `updated_at`).
//!
//! The tests follow the **constructor + update pattern** of this
//! domain: no factory handlers, no events emitted from the
//! constructor. The handlers / outbox / audit fan-out are not yet
//! wired end-to-end; these tests pin the **aggregate layer**
//! contract that the service factory fns and dispatcher will
//! eventually wrap.
//!
//! Implements: `docs/specs/events/aggregates.md` ## AssignIncident
//! and `docs/specs/events/workflows.md` ## "AssignIncident
//! Configuration Workflow".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{SchoolId, UserId};
use educore_core::value_objects::{Timestamp, Version};
use educore_events_domain::aggregate::AssignIncident;
use educore_events_domain::errors::EventsDomainError;
use educore_events_domain::prelude::*;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh school id + system actor id from a single
/// `SystemIdGen`. Mirrors the fixture style used in
/// `tests/workflows.rs` and `tests/holiday.rs`.
fn fixture_ids() -> (SchoolId, UserId, Timestamp) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let at = TestClock::new().now();
    (school, actor, at)
}

// =============================================================================
// 1. Happy path: AssignIncident::new + reassign
// =============================================================================

/// End-to-end happy path for the AssignIncident aggregate.
/// Build an assignment for an existing incident mapped to a
/// student with `point = 10`, then bump the point value to
/// `point = 25` via `reassign`, asserting that:
///
/// 1. `AssignIncident::new` returns an aggregate whose
///    `school_id`, `incident_id`, `student_id`, `point`,
///    `added_by`, `academic_id`, `version`, and
///    `active_status` all match the command inputs.
/// 2. `AssignIncident::reassign` mutates the aggregate
///    in-place, swapping `point`, bumping `version`
///    exactly once, and moving `updated_at`.
#[test]
fn assign_incident_create_then_reassign_mutates_aggregate_and_bumps_version() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = AssignIncidentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());
    let student_id = g.next_uuid();
    let academic_id = AcademicYearRef::new(school, g.next_uuid());

    // ---- Create flow ----
    let mut assignment = AssignIncident::new(
        id,
        incident_id,
        Some(student_id),
        None,
        10,
        actor,
        academic_id,
        ts,
    )
    .expect("AssignIncident::new must succeed for valid input");

    // Identity: typed id and denormalised school_id agree.
    assert_eq!(assignment.id, id);
    assert_eq!(assignment.id.school_id(), school);
    assert_eq!(assignment.school_id, school);

    // Payload: every field on the command is present on the aggregate.
    assert_eq!(assignment.incident_id, incident_id);
    assert_eq!(assignment.student_id, Some(student_id));
    assert_eq!(assignment.user_id, None);
    assert_eq!(assignment.point, 10);
    assert_eq!(assignment.added_by, actor);
    assert_eq!(assignment.academic_id, academic_id);

    // Audit metadata footer is initialised per the constructor
    // contract: version starts at 1, active_status is true, and
    // created_by/updated_by both equal the supplied actor.
    assert_eq!(assignment.version, Version::initial());
    assert!(assignment.active_status);
    assert_eq!(assignment.created_by, actor);
    assert_eq!(assignment.updated_by, actor);
    assert_eq!(assignment.created_at, ts);
    assert_eq!(assignment.updated_at, ts);

    // ---- Reassign flow ----
    let version_before = assignment.version;
    let updated_at = Timestamp::now();
    assignment
        .reassign(25, updated_at)
        .expect("AssignIncident::reassign must succeed for valid point");

    // The aggregate is mutated in place.
    assert_eq!(assignment.point, 25);
    assert_eq!(
        assignment.version,
        version_before.next(),
        "version must be bumped exactly once"
    );
    assert_eq!(assignment.updated_at, updated_at);
    // Identity + audit fields that reassign must NOT touch.
    assert_eq!(assignment.id, id);
    assert_eq!(assignment.school_id, school);
    assert_eq!(assignment.incident_id, incident_id);
    assert_eq!(assignment.student_id, Some(student_id));
    assert_eq!(assignment.added_by, actor);
    assert!(assignment.active_status);
}

// =============================================================================
// 2. Validation failure: both student_id and user_id set is rejected
// =============================================================================

/// Validation-failure path on the create flow: per spec
/// invariant, exactly one of `student_id` / `user_id` must be
/// set. Setting both is forbidden — `AssignIncident::new` must
/// reject it with [`EventsDomainError::Validation`].
#[test]
fn assign_incident_create_with_both_student_and_user_returns_validation_error() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = AssignIncidentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());
    let academic_id = AcademicYearRef::new(school, g.next_uuid());

    let res = AssignIncident::new(
        id,
        incident_id,
        Some(g.next_uuid()),
        Some(g.next_uuid()),
        10,
        actor,
        academic_id,
        ts,
    );
    let err = res.expect_err("both student_id and user_id set must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "expected EventsDomainError::Validation, got {err:?}"
    );
}

// =============================================================================
// 3. Validation failure: point out of range is rejected
// =============================================================================

/// Validation-failure path on the create flow: per spec
/// invariant, `point` must be in `0..=1000`. `point = 1500`
/// must be rejected with [`EventsDomainError::Validation`].
#[test]
fn assign_incident_create_with_point_out_of_range_returns_validation_error() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = AssignIncidentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());
    let academic_id = AcademicYearRef::new(school, g.next_uuid());

    let res = AssignIncident::new(
        id,
        incident_id,
        Some(g.next_uuid()),
        None,
        1500,
        actor,
        academic_id,
        ts,
    );
    let err = res.expect_err("point = 1500 must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "expected EventsDomainError::Validation, got {err:?}"
    );
}
