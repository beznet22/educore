//! Integration tests for the **StudentCategory aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for the
//! `StudentCategory` aggregate end-to-end through the service
//! layer, exercising the 1 spec invariant:
//!
//! - I-1: Category uniquely named within school

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;

use educore_academic::commands::{DeleteStudentCategoryCommand, RealCreateStudentCategoryCommand, UpdateStudentCategoryCommand};
use educore_academic::events::{StudentCategoryDeleted, StudentCategoryUpdated, RealStudentCategoryCreated};
use educore_academic::prelude::*;
use educore_academic::services::{create_student_category_aggregate, delete_student_category, update_student_category};
use educore_academic::{RealStudentCategory, StudentCategoryId};
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

fn student_category_id(g: &SystemIdGen, school: SchoolId) -> StudentCategoryId {
    StudentCategoryId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateStudentCategoryCommand {
    RealCreateStudentCategoryCommand {
        tenant,
        student_category_id: student_category_id(g, school),
        name: "Scholarship".to_string(),
        description: "Merit-based scholarship".to_string(),
        discount_percent: Some(50.0),
    }
}

#[derive(Default)]
struct InMemoryUniqueness {
    category_names: HashSet<(SchoolId, String)>,
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
    fn student_category_name_exists(&self, school: SchoolId, name: &str) -> bool {
        self.category_names.contains(&(school, name.to_string()))
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn student_category_create_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.name, "Scholarship");
    assert_eq!(agg.discount_percent, Some(50.0));

    assert_eq!(RealStudentCategoryCreated::EVENT_TYPE, "academic.student_category.created");
    assert_eq!(RealStudentCategoryCreated::AGGREGATE_TYPE, "student_category");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: duplicate name rejected
// =============================================================================

#[test]
fn student_category_duplicate_name_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut uniqueness = InMemoryUniqueness::default();
    uniqueness.category_names.insert((school, "Scholarship".to_string()));

    let cmd = make_cmd(tenant, &g, school);
    let err = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate name must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 3. Empty name rejected
// =============================================================================

#[test]
fn student_category_empty_name_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.name = "   ".to_string();

    let err = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect_err("empty name must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 4. Invalid discount rejected
// =============================================================================

#[test]
fn student_category_invalid_discount_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.discount_percent = Some(150.0); // > 100

    let err = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect_err("invalid discount must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 5. Update
// =============================================================================

#[test]
fn student_category_update_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let upd = UpdateStudentCategoryCommand {
        tenant,
        student_category_id: agg.id,
        name: Some("Sibling Discount".to_string()),
        description: None,
        discount_percent: None,
    };
    let event = update_student_category(upd, &mut agg, &clock, &ids)
        .expect("update");
    assert_eq!(agg.name, "Sibling Discount");
    let _: StudentCategoryUpdated = event;
}

// =============================================================================
// 6. Delete
// =============================================================================

#[test]
fn student_category_delete_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_student_category_aggregate(cmd, &clock, &ids, &uniqueness)
        .expect("create");

    let del = DeleteStudentCategoryCommand {
        tenant,
        student_category_id: agg.id,
    };
    let event = delete_student_category(del, &mut agg, &clock, &ids)
        .expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: StudentCategoryDeleted = event;
}
