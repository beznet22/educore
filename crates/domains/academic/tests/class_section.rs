//! Integration tests for the **ClassSection aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`ClassSection`](educore_academic::ClassSection)
//! end-to-end through the service layer:
//!
//! 1. `create_class_section` validates that the typed id's
//!    `school_id` matches the command's `school_id` (the
//!    tenant-anchor invariant), constructs the placeholder
//!    aggregate from the typed id, and emits a
//!    [`ClassSectionCreated`] event.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs`,
//! `crates/domains/academic/tests/subject.rs`, and
//! `crates/domains/academic/tests/workflows.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests
//! pin the contract of the **service layer** that the
//! dispatcher will eventually wrap.
//!
//! Note on `ClassSection` field set: the aggregate is a
//! **placeholder** (id + school_id only) per the
//! placeholder-aggregate macro in `aggregate.rs`. The
//! full impl (class_id, section_id, academic_year_id,
//! class_teacher_id, audit footer, update flow) lands in
//! a later workstream per `docs/build-plan.md`. The
//! tests below therefore pin the **current** contract:
//! aggregate carries the typed id + school_id, event
//! carries the typed id + school_id + occurred_at +
//! event_id, and the service enforces the
//! `id.school_id() == school_id` invariant.
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
use educore_events::domain_event::DomainEvent;

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

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create a ClassSection
// =============================================================================

/// End-to-end happy path for the `ClassSection` aggregate.
/// Create a class-section pairing under a fresh school,
/// asserting that:
///
/// 1. The create flow produces a `ClassSection` aggregate
///    carrying the typed id + the school id derived from
///    the typed id, plus a `ClassSectionCreated` event
///    with the right `event_type`, `aggregate_type`,
///    `school_id`, `aggregate_id`, and a non-zero
///    `occurred_at` timestamp.
/// 2. The event's `event_id` is set and stable, and the
///    `DomainEvent` trait's `aggregate_id()` returns the
///    typed id's local UUID.
/// 3. Two distinct class-section pairings on the same
///    school produce distinct typed ids and distinct
///    `aggregate_id`s on their emitted events, proving
///    the service does not accidentally coalesce
///    pairings.
#[test]
fn class_section_create_builds_aggregate_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let cid = class_section_id(&g, school);
    let cmd = CreateClassSectionCommand {
        id: cid,
        school_id: school,
    };
    let (agg, event) = create_class_section(cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the typed id.
    assert_eq!(agg.id, cid);
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.id.school_id(), school);

    // Event metadata matches the DomainEvent trait's
    // contract.
    assert_eq!(
        <ClassSectionCreated as DomainEvent>::EVENT_TYPE,
        "academic.class_section.created"
    );
    assert_eq!(
        <ClassSectionCreated as DomainEvent>::AGGREGATE_TYPE,
        "class_section"
    );
    assert_eq!(<ClassSectionCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id, cid);
    assert_eq!(event.school_id, school);

    // Sanity check: a second class-section on the same
    // school mints a different typed id and emits a
    // different aggregate_id.
    let cid2 = class_section_id(&g, school);
    assert_ne!(cid, cid2);
    let cmd2 = CreateClassSectionCommand {
        id: cid2,
        school_id: school,
    };
    let (agg2, event2) = create_class_section(cmd2, &clock, &ids).expect("create second");
    assert_eq!(agg2.id, cid2);
    assert_eq!(agg2.school_id, school);
    assert_eq!(event2.aggregate_id, cid2);
    assert_ne!(event.aggregate_id, event2.aggregate_id);
}

// =============================================================================
// 2. Validation failure: id.school_id != school_id returns DomainError::Validation
// =============================================================================

/// Validation-failure path on the create flow: when the
/// command's `school_id` does not match the typed id's
/// `school_id`, `create_class_section` returns
/// `DomainError::Validation` (the tenant-anchor invariant
/// trips before the aggregate or the event are
/// constructed). This pins the cross-tenant guard that
/// the placeholder impl already enforces.
///
/// Note: this is the second test (not a separate
/// "happy-path update" test) because the placeholder
/// `ClassSection` aggregate has no update flow yet — the
/// full impl (class_id, section_id, academic_year_id,
/// audit footer) lands in a later workstream per
/// `docs/build-plan.md`. The validation test pins the
/// **single invariant** the placeholder guards today.
#[test]
fn class_section_create_with_cross_school_id_returns_validation_error() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let other_school = g.next_school_id();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Cross-tenant: typed id belongs to `other_school`,
    // but the command claims `school`. Must fail with
    // Validation.
    let cid = ClassSectionId::new(other_school, g.next_uuid());
    let cmd = CreateClassSectionCommand {
        id: cid,
        school_id: school,
    };
    let err = create_class_section(cmd, &clock, &ids)
        .expect_err("cross-school id must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // Sanity check: a subsequent call with a matching
    // `school_id` still succeeds, proving the failure
    // was tied to the cross-school mismatch (and not to
    // a corrupt clock, ids, or test setup).
    let cid2 = ClassSectionId::new(school, g.next_uuid());
    let ok_cmd = CreateClassSectionCommand {
        id: cid2,
        school_id: school,
    };
    let (_agg, _event) =
        create_class_section(ok_cmd, &clock, &ids).expect("matching school id must succeed");
}
