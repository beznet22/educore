//! Integration tests for the **StudentGroup aggregate** vertical slice.
//!
//! Pins the create / update / add-student / remove-student / delete
//! contracts for the `StudentGroup` aggregate end-to-end through
//! the service layer, exercising all 2 spec invariants:
//!
//! - I-1: Group uniquely named within school
//! - I-2: A student can be in many groups

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;

use educore_academic::commands::{
    AddStudentToGroupCommand, DeleteStudentGroupCommand, RealCreateStudentGroupCommand,
    RemoveStudentFromGroupCommand, UpdateStudentGroupCommand,
};
use educore_academic::events::{
    RealStudentGroupCreated, StudentAddedToGroup, StudentGroupDeleted, StudentGroupUpdated,
    StudentRemovedFromGroup,
};
use educore_academic::prelude::*;
use educore_academic::services::{
    add_student_to_group, create_student_group_aggregate, delete_student_group,
    remove_student_from_group, update_student_group,
};
use educore_academic::{RealStudentGroup, StudentGroupId};
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;

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

fn student_group_id(g: &SystemIdGen, school: SchoolId) -> StudentGroupId {
    StudentGroupId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateStudentGroupCommand {
    RealCreateStudentGroupCommand {
        tenant,
        student_group_id: student_group_id(g, school),
        name: "Chess Club".to_string(),
        description: "After-school chess activities".to_string(),
    }
}

#[derive(Default)]
struct InMemoryUniqueness {
    group_names: HashSet<(SchoolId, String)>,
}

impl UniquenessChecker for InMemoryUniqueness {
    fn student_admission_no_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn student_email_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn roll_no_exists(&self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId, _: &str) -> bool { false }
    fn class_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn section_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn subject_code_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn lesson_title_exists(&self, _: SchoolId, _: ClassSectionId, _: SubjectId, _: &str) -> bool { false }
    fn class_section_exists(&self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId) -> bool { false }
    fn class_section_has_student_records(&self, _: SchoolId, _: ClassSectionId) -> bool { false }
    fn academic_year_overlaps(&self, _: SchoolId, _: AcademicYearRange, _: Option<AcademicYearId>) -> bool { false }
    fn optional_subject_assigned_exists(&self, _: SchoolId, _: StudentId, _: AcademicYearId) -> bool { false }
    fn primary_guardian_link_exists(&self, _: SchoolId, _: StudentId) -> bool { false }
    fn student_has_active_record(&self, _: SchoolId, _: StudentId, _: AcademicYearId) -> bool { false }
    fn teacher_has_conflict(&self, _: SchoolId, _: UserId, _: DayOfWeek, _: u8) -> bool { false }
    fn room_has_conflict(&self, _: SchoolId, _: ClassRoomId, _: DayOfWeek, _: u8) -> bool { false }
    fn student_category_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn student_group_name_exists(&self, school: SchoolId, name: &str) -> bool {
        self.group_names.contains(&(school, name.to_string()))
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn student_group_create_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.name, "Chess Club");
    assert!(agg.member_ids.is_empty());

    assert_eq!(RealStudentGroupCreated::EVENT_TYPE, "academic.student_group.created");
    assert_eq!(RealStudentGroupCreated::AGGREGATE_TYPE, "student_group");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: duplicate name rejected
// =============================================================================

#[test]
fn student_group_duplicate_name_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut uniqueness = InMemoryUniqueness::default();
    uniqueness.group_names.insert((school, "Chess Club".to_string()));

    let cmd = make_cmd(tenant, &g, school);
    let err = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate name must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 3. I-2: add student to group
// =============================================================================

#[test]
fn student_group_add_student_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let student = g.next_user_id();
    let add_cmd = AddStudentToGroupCommand {
        tenant,
        student_group_id: agg.id,
        student_id: StudentId::new(school, g.next_uuid()),
    };
    let event = add_student_to_group(add_cmd, &mut agg, &clock, &ids).expect("add");
    assert_eq!(agg.member_ids.len(), 1);
    let _: StudentAddedToGroup = event;
}

// =============================================================================
// 4. I-2: idempotent add (same student twice)
// =============================================================================

#[test]
fn student_group_add_same_student_idempotent() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let student_id = StudentId::new(school, g.next_uuid());

    let add_cmd1 = AddStudentToGroupCommand {
        tenant: tenant.clone(),
        student_group_id: agg.id,
        student_id,
    };
    add_student_to_group(add_cmd1, &mut agg, &clock, &ids).expect("first add");

    let add_cmd2 = AddStudentToGroupCommand {
        tenant,
        student_group_id: agg.id,
        student_id,
    };
    add_student_to_group(add_cmd2, &mut agg, &clock, &ids).expect("second add (idempotent)");
    assert_eq!(agg.member_ids.len(), 1);
}

// =============================================================================
// 5. I-2: remove student
// =============================================================================

#[test]
fn student_group_remove_student_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let student_id = StudentId::new(school, g.next_uuid());

    let add_cmd = AddStudentToGroupCommand {
        tenant: tenant.clone(),
        student_group_id: agg.id,
        student_id,
    };
    add_student_to_group(add_cmd, &mut agg, &clock, &ids).expect("add");
    assert_eq!(agg.member_ids.len(), 1);

    let rm_cmd = RemoveStudentFromGroupCommand {
        tenant,
        student_group_id: agg.id,
        student_id,
    };
    let event = remove_student_from_group(rm_cmd, &mut agg, &clock, &ids).expect("remove");
    assert_eq!(agg.member_ids.len(), 0);
    let _: StudentRemovedFromGroup = event;
}

// =============================================================================
// 6. Update
// =============================================================================

#[test]
fn student_group_update_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let upd = UpdateStudentGroupCommand {
        tenant,
        student_group_id: agg.id,
        name: Some("Chess & Board Games".to_string()),
        description: None,
    };
    let event = update_student_group(upd, &mut agg, &clock, &ids).expect("update");
    assert_eq!(agg.name, "Chess & Board Games");
    let _: StudentGroupUpdated = event;
}

// =============================================================================
// 7. Delete
// =============================================================================

#[test]
fn student_group_delete_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_group_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let del = DeleteStudentGroupCommand {
        tenant,
        student_group_id: agg.id,
    };
    let event = delete_student_group(del, &mut agg, &clock, &ids).expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: StudentGroupDeleted = event;
}
