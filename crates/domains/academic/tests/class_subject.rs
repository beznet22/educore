//! Integration tests for the **ClassSubject aggregate** vertical slice.
//!
//! Pins the ClassSubject invariants from
//! `docs/specs/academic/aggregates.md` § ClassSubject:
//!
//! - **I-1**: class-or-class-section scope (closed enum
//!   `ClassSubjectScope`: `ClassOnly` requires
//!   `class_section_id == None`; `ClassSection` requires
//!   `class_section_id == Some(_)`).
//! - **I-3**: optional `PassMark` override must be in
//!   `0.0..=100.0` (via `PassMark::new`).
//!
//! Plus the create / reassign / unassign happy paths.
//!
//! The tests use the same fixture pattern as the rest of
//! the academic test suites (`TestClock` + `SystemIdGen`).
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
use educore_core::ids::UserId;
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

fn class_subject_id(g: &SystemIdGen, school: SchoolId) -> ClassSubjectId {
    ClassSubjectId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn teacher_id(g: &SystemIdGen) -> UserId {
    g.next_user_id()
}

fn build_cmd(
    tenant: TenantContext,
    cs_id: ClassSubjectId,
    class: ClassId,
    class_section: Option<ClassSectionId>,
    subject: SubjectId,
    teacher: UserId,
    scope: ClassSubjectScope,
    pass_mark: Option<PassMark>,
) -> AssignSubjectToClassCommand {
    AssignSubjectToClassCommand {
        tenant,
        class_subject_id: cs_id,
        class_id: class,
        class_section_id: class_section,
        subject_id: subject,
        teacher_id: teacher,
        scope,
        pass_mark,
    }
}

// =============================================================================
// 1. I-1 happy path: ClassOnly scope, no class_section_id
// =============================================================================

#[test]
fn class_subject_assign_with_class_only_no_section_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    let cmd = build_cmd(
        tenant,
        cs_id,
        class,
        None,
        subject,
        teacher,
        ClassSubjectScope::ClassOnly,
        None,
    );
    let (agg, event) = assign_subject_to_class(cmd, &clock, &ids).expect("assign");

    // Aggregate fields are populated.
    assert_eq!(agg.id, cs_id);
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.class_id, class);
    assert_eq!(agg.class_section_id, None);
    assert_eq!(agg.subject_id, subject);
    assert_eq!(agg.teacher_id, teacher);
    assert_eq!(agg.scope, ClassSubjectScope::ClassOnly);
    assert_eq!(agg.pass_mark, None);
    assert!(agg.is_active);
    assert_eq!(agg.active_status, ActiveStatus::Active);

    // Event metadata matches the DomainEvent trait contract.
    assert_eq!(
        <SubjectAssignedToClass as DomainEvent>::EVENT_TYPE,
        "academic.class_subject.assigned"
    );
    assert_eq!(
        <SubjectAssignedToClass as DomainEvent>::AGGREGATE_TYPE,
        "class_subject"
    );
    assert_eq!(<SubjectAssignedToClass as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), cs_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.class_subject_id, cs_id);
    assert_eq!(event.class_id, class);
    assert_eq!(event.class_section_id, None);
    assert_eq!(event.subject_id, subject);
    assert_eq!(event.teacher_id, teacher);
    assert_eq!(event.scope, ClassSubjectScope::ClassOnly);
}

// =============================================================================
// 2. I-1 happy path: ClassSection scope, with class_section_id
// =============================================================================

#[test]
fn class_subject_assign_with_class_section_requires_section_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let section = class_section_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    let cmd = build_cmd(
        tenant,
        cs_id,
        class,
        Some(section),
        subject,
        teacher,
        ClassSubjectScope::ClassSection,
        None,
    );
    let (agg, event) = assign_subject_to_class(cmd, &clock, &ids).expect("assign");
    assert_eq!(agg.class_section_id, Some(section));
    assert_eq!(agg.scope, ClassSubjectScope::ClassSection);
    assert_eq!(event.class_section_id, Some(section));
    assert_eq!(event.scope, ClassSubjectScope::ClassSection);
}

// =============================================================================
// 3. I-1 violation: ClassOnly + Some(section) is rejected
// =============================================================================

#[test]
fn class_subject_assign_with_class_only_and_section_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let section = class_section_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    // ClassOnly + Some(section) is a violation.
    let cmd = build_cmd(
        tenant,
        cs_id,
        class,
        Some(section),
        subject,
        teacher,
        ClassSubjectScope::ClassOnly,
        None,
    );
    let err = assign_subject_to_class(cmd, &clock, &ids)
        .expect_err("ClassOnly + Some(section) must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert!(
        err.to_string().contains("ClassOnly"),
        "error message should mention ClassOnly, got: {err}"
    );
}

// =============================================================================
// 4. I-1 violation: ClassSection + None is rejected
// =============================================================================

#[test]
fn class_subject_assign_with_class_section_and_no_section_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    // ClassSection + None is a violation.
    let cmd = build_cmd(
        tenant,
        cs_id,
        class,
        None,
        subject,
        teacher,
        ClassSubjectScope::ClassSection,
        None,
    );
    let err = assign_subject_to_class(cmd, &clock, &ids)
        .expect_err("ClassSection + None must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert!(
        err.to_string().contains("ClassSection"),
        "error message should mention ClassSection, got: {err}"
    );
}

// =============================================================================
// 5. I-3 happy path: PassMark in range succeeds
// =============================================================================

#[test]
fn class_subject_assign_with_pass_mark_in_range_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    let pm = PassMark::new(45.0).expect("valid pass mark");
    let cmd = build_cmd(
        tenant,
        cs_id,
        class,
        None,
        subject,
        teacher,
        ClassSubjectScope::ClassOnly,
        Some(pm),
    );
    let (agg, event) = assign_subject_to_class(cmd, &clock, &ids).expect("assign");
    assert_eq!(agg.pass_mark, Some(pm));
    assert_eq!(event.pass_mark, Some(pm));

    // Edge values: 0.0 and 100.0 must succeed.
    for v in [0.0_f32, 100.0_f32] {
        let cs_id2 = class_subject_id(&g, school);
        let pm2 = PassMark::new(v).expect("edge pass mark");
        let cmd2 = build_cmd(
            TenantContext::for_user(school, teacher, ids.next_correlation_id(), UserType::SchoolAdmin),
            cs_id2,
            class,
            None,
            subject,
            teacher,
            ClassSubjectScope::ClassOnly,
            Some(pm2),
        );
        let (agg2, _) = assign_subject_to_class(cmd2, &clock, &ids).expect("assign edge");
        assert_eq!(agg2.pass_mark, Some(pm2));
    }
}

// =============================================================================
// 6. I-3 violation: PassMark out of range is rejected at the service boundary
// =============================================================================
//
// Note: `PassMark::new` is the primary value-object guard.
// Since `AssignSubjectToClassCommand.pass_mark` is an
// `Option<PassMark>` (not an `Option<f32>`), the only way
// for an out-of-range value to reach the service is via
// the aggregate constructor's internal re-validation
// (which calls `PassMark::new(pm.as_f32())` again to
// assert the invariant). This test pins that re-check by
// constructing a `ClassSubject` directly with a manually
// stitched `PassMark`-bypass.
//
// Because we cannot bypass `PassMark::new` from outside
// the crate, we instead pin the rejection at the
// `PassMark::new` boundary (the canonical gate).
#[test]
fn pass_mark_constructor_rejects_out_of_range() {
    assert!(PassMark::new(-0.01).is_err());
    assert!(PassMark::new(100.01).is_err());
    assert!(PassMark::new(50.0).is_ok());
    assert!(PassMark::new(0.0).is_ok());
    assert!(PassMark::new(100.0).is_ok());
}

// =============================================================================
// 7. Reassign teacher: updates teacher_id, bumps version
// =============================================================================

#[test]
fn class_subject_reassign_teacher_updates_teacher_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher1 = teacher_id(&g);
    let teacher2 = teacher_id(&g);
    let cmd = build_cmd(
        tenant.clone(),
        cs_id,
        class,
        None,
        subject,
        teacher1,
        ClassSubjectScope::ClassOnly,
        None,
    );
    let (mut agg, _ev) = assign_subject_to_class(cmd, &clock, &ids).expect("assign");
    let initial_version = agg.version;
    assert_eq!(agg.teacher_id, teacher1);

    // Reassign to teacher2.
    let reassign_cmd = ReassignTeacherCommand {
        tenant: tenant.clone(),
        class_subject_id: cs_id,
        new_teacher_id: teacher2,
    };
    let event = reassign_teacher(reassign_cmd, &clock, &ids, &mut agg).expect("reassign");
    assert_eq!(event.previous_teacher_id, teacher1);
    assert_eq!(event.new_teacher_id, teacher2);
    assert_eq!(event.class_subject_id, cs_id);
    assert_eq!(agg.teacher_id, teacher2);
    assert!(
        agg.version > initial_version,
        "version must bump on reassign"
    );
    assert_eq!(
        <TeacherReassigned as DomainEvent>::EVENT_TYPE,
        "academic.class_subject.teacher_reassigned"
    );
}

// =============================================================================
// 8. Unassign: retires the aggregate, bumps version, emits SubjectUnassigned
// =============================================================================

#[test]
fn class_subject_unassign_retires() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let class = class_id(&g, school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    let cmd = build_cmd(
        tenant.clone(),
        cs_id,
        class,
        None,
        subject,
        teacher,
        ClassSubjectScope::ClassOnly,
        None,
    );
    let (mut agg, _ev) = assign_subject_to_class(cmd, &clock, &ids).expect("assign");
    let initial_version = agg.version;
    assert_eq!(agg.active_status, ActiveStatus::Active);
    assert!(agg.is_active);

    // Unassign (retire).
    let unassign_cmd = UnassignSubjectCommand {
        tenant: tenant.clone(),
        class_subject_id: cs_id,
    };
    let event = unassign_subject(unassign_cmd, &clock, &ids, &mut agg).expect("unassign");
    assert_eq!(event.class_subject_id, cs_id);
    assert_eq!(agg.active_status, ActiveStatus::Retired);
    assert!(!agg.is_active);
    assert!(
        agg.version > initial_version,
        "version must bump on unassign"
    );
    assert_eq!(
        <SubjectUnassigned as DomainEvent>::EVENT_TYPE,
        "academic.class_subject.unassigned"
    );
}

// =============================================================================
// 9. Tenant-anchor: cross-school class_id is rejected
// =============================================================================

#[test]
fn class_subject_assign_cross_school_class_id_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let other_school = g.next_school_id();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cs_id = class_subject_id(&g, school);
    let other_class = class_id(&g, other_school);
    let subject = subject_id(&g, school);
    let teacher = teacher_id(&g);
    let cmd = build_cmd(
        tenant,
        cs_id,
        other_class,
        None,
        subject,
        teacher,
        ClassSubjectScope::ClassOnly,
        None,
    );
    let err = assign_subject_to_class(cmd, &clock, &ids)
        .expect_err("cross-school class_id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
