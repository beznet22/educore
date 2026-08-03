//! Integration tests for the **AssignClassTeacher aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`AssignClassTeacher`](educore_hr::aggregate::AssignClassTeacher)
//! end-to-end, plus the Wave 180 mutators that enforce spec
//! invariants I-1 (composite-key uniqueness) and I-2
//! (`active_status == 1` while open).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::value_objects::{AcademicYearId, ClassId, SectionId};
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::AssignClassTeacher;
use educore_hr::services::AssignClassTeacherUniquenessChecker;
use educore_hr::value_objects::{AssignClassTeacherId, StaffId};

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

fn assign_class_teacher_id(g: &SystemIdGen, school: SchoolId) -> AssignClassTeacherId {
    AssignClassTeacherId::new(school, g.next_uuid())
}

/// Helper: build a fresh AssignClassTeacher for tests.
fn fresh_assign_class_teacher(tenant: &TenantContext, g: &SystemIdGen) -> AssignClassTeacher {
    let school = tenant.school_id;
    let id = assign_class_teacher_id(g, school);
    let class_id = ClassId::new(school, g.next_uuid());
    let section_id = SectionId::new(school, g.next_uuid());
    let staff_id = StaffId::new(school, g.next_uuid());
    let academic_id = AcademicYearId::new(school, g.next_uuid());
    AssignClassTeacher::fresh(
        id,
        class_id,
        section_id,
        staff_id,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `AssignClassTeacherUniquenessChecker` mock.
struct FakeAssignClassTeacherUniqueness {
    exists: bool,
}
impl AssignClassTeacherUniquenessChecker for FakeAssignClassTeacherUniqueness {
    fn assign_class_teacher_exists(
        &self,
        _school: SchoolId,
        _class_id: ClassId,
        _section_id: SectionId,
        _academic_id: AcademicYearId,
    ) -> bool {
        self.exists
    }
}

#[test]
fn assign_class_teacher_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = assign_class_teacher_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn assign_class_teacher_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = assign_class_teacher_id(&g, school);
    let id_b = assign_class_teacher_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 180 — Spec invariant AssignClassTeacher#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn assign_class_teacher_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let act = fresh_assign_class_teacher(&tenant, &g);
    let checker = FakeAssignClassTeacherUniqueness { exists: false };
    assert!(act.ensure_unique(&checker).is_ok());
}

#[test]
fn assign_class_teacher_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let act = fresh_assign_class_teacher(&tenant, &g);
    let checker = FakeAssignClassTeacherUniqueness { exists: true };
    let err = act.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 180 — Spec invariant AssignClassTeacher#2 (active_status == 1)
// =============================================================================

#[test]
fn assign_class_teacher_ensure_active_open_accepts_active() {
    let (tenant, g) = admin_context();
    let act = fresh_assign_class_teacher(&tenant, &g);
    assert_eq!(act.active_status, 1);
    assert!(act.ensure_active_open().is_ok());
}

#[test]
fn assign_class_teacher_ensure_active_open_rejects_inactive() {
    let (tenant, g) = admin_context();
    let mut act = fresh_assign_class_teacher(&tenant, &g);
    act.active_status = 0;
    let err = act.ensure_active_open().expect_err("inactive must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
