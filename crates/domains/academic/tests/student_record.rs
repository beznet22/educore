//! Integration tests for the **StudentRecord aggregate** vertical slice.
//!
//! Pins the enroll / set-roll / set-default / mark-graduate
//! contracts for the `StudentRecord` aggregate end-to-end
//! through the service layer, exercising all 6 spec invariants:
//!
//! - I-1: At most one non-graduate, non-withdrawn record per student per year
//! - I-2: RollNumber unique within (class, section, academic_year)
//! - I-3: IsDefault flag
//! - I-4: IsPromote=false until StudentPromoted closes
//! - I-5: IsGraduate=true when graduated
//! - I-6: AdmissionNumber carried from admission

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;

use educore_academic::prelude::*;
use educore_academic::commands::{
    EnrollStudentCommand, MarkGraduateCommand, SetDefaultRecordCommand, SetRollNumberCommand,
};
use educore_academic::events::{
    DefaultRecordSet, RollNumberAssigned, StudentMarkedGraduate, StudentRecordEnrolled,
};
use educore_academic::services::{
    enroll_student, mark_graduate, set_default_record, set_roll_number,
};
use educore_academic::StudentRecord;
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

fn student_record_id(g: &SystemIdGen, school: SchoolId) -> StudentRecordId {
    StudentRecordId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> EnrollStudentCommand {
    EnrollStudentCommand {
        tenant,
        student_record_id: student_record_id(g, school),
        student_id: student_id(g, school),
        class_id: class_id(g, school),
        section_id: section_id(g, school),
        academic_year_id: academic_year_id(g, school),
        admission_number: Some("ADM-2025-0001".to_string()),
    }
}

/// In-memory uniqueness checker.
#[derive(Default)]
struct InMemoryUniqueness {
    active_records: HashSet<(SchoolId, StudentId, AcademicYearId)>,
    roll_numbers: HashSet<(SchoolId, ClassId, SectionId, AcademicYearId, String)>,
}

impl UniquenessChecker for InMemoryUniqueness {
    fn student_admission_no_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn student_email_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn roll_no_exists(
        &self, school: SchoolId, class: ClassId, section: SectionId,
        year: AcademicYearId, roll: &str,
    ) -> bool {
        self.roll_numbers.contains(&(school, class, section, year, roll.to_string()))
    }
    fn class_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn section_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn subject_code_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn lesson_title_exists(&self, _: SchoolId, _: ClassSectionId, _: SubjectId, _: &str) -> bool { false }
    fn class_section_exists(&self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId) -> bool { false }
    fn class_section_has_student_records(&self, _: SchoolId, _: ClassSectionId) -> bool { false }
    fn academic_year_overlaps(&self, _: SchoolId, _: AcademicYearRange, _: Option<AcademicYearId>) -> bool { false }
    fn optional_subject_assigned_exists(&self, _: SchoolId, _: StudentId, _: AcademicYearId) -> bool { false }
    fn primary_guardian_link_exists(&self, _: SchoolId, _: StudentId) -> bool { false }
    fn student_has_active_record(
        &self, school: SchoolId, student: StudentId, year: AcademicYearId,
    ) -> bool {
        self.active_records.contains(&(school, student, year))
    }
    fn teacher_has_conflict(&self, _: SchoolId, _: UserId, _: DayOfWeek, _: u8) -> bool { false }
    fn room_has_conflict(&self, _: SchoolId, _: ClassRoomId, _: DayOfWeek, _: u8) -> bool { false }

    fn student_category_name_exists(
        &self, _: SchoolId, _: &str,
    ) -> bool {
        false
    }

    fn student_group_name_exists(
        &self, _: SchoolId, _: &str,
    ) -> bool {
        false
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn student_record_enroll_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = enroll_student(cmd, &clock, &ids, &uniqueness)
        .expect("enroll should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.is_default, true); // I-3: default on initial enrollment
    assert_eq!(agg.is_promote, false); // I-4: false initially
    assert_eq!(agg.is_graduate, false); // I-5
    assert_eq!(agg.admission_number, Some("ADM-2025-0001".to_string())); // I-6

    assert_eq!(StudentRecordEnrolled::EVENT_TYPE, "academic.student_record.enrolled");
    assert_eq!(StudentRecordEnrolled::AGGREGATE_TYPE, "student_record");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: duplicate active record rejected
// =============================================================================

#[test]
fn student_record_duplicate_active_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let year = cmd.academic_year_id;
    let student = cmd.student_id;
    let mut uniqueness = InMemoryUniqueness::default();
    uniqueness.active_records.insert((school, student, year));

    let err = enroll_student(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate active record must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 3. I-2: duplicate roll number rejected
// =============================================================================

#[test]
fn student_record_duplicate_roll_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let class = cmd.class_id;
    let section = cmd.section_id;
    let year = cmd.academic_year_id;
    let uniqueness = InMemoryUniqueness::default();

    let (mut agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");

    // Pre-record roll 5 as taken.
    let mut uniqueness = uniqueness;
    uniqueness.roll_numbers.insert((school, class, section, year, "5".to_string()));

    let roll_cmd = SetRollNumberCommand {
        tenant,
        student_record_id: agg.id,
        roll_number: "5".to_string(),
    };
    let err = set_roll_number(roll_cmd, &mut agg, &clock, &ids, &uniqueness)
        .expect_err("duplicate roll must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 4. I-3: set default
// =============================================================================

#[test]
fn student_record_set_default_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");

    let default_cmd = SetDefaultRecordCommand {
        tenant,
        student_record_id: agg.id,
    };
    let event = set_default_record(default_cmd, &mut agg, &clock, &ids).expect("set default");
    assert!(agg.is_default);
    let _: DefaultRecordSet = event;
}

// =============================================================================
// 5. I-4: promote mark + close
// =============================================================================

#[test]
fn student_record_mark_promote_and_close() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (mut agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");
    assert!(!agg.is_promote);

    agg.mark_promote(agg.created_by, clock.now());
    assert!(agg.is_promote);

    agg.close_promotion(agg.created_by, clock.now());
    assert!(!agg.is_promote);
}

// =============================================================================
// 6. I-5: mark graduate
// =============================================================================

#[test]
fn student_record_mark_graduate_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");

    let grad_cmd = MarkGraduateCommand {
        tenant,
        student_record_id: agg.id,
    };
    let event = mark_graduate(grad_cmd, &mut agg, &clock, &ids).expect("mark graduate");
    assert!(agg.is_graduate);
    let _: StudentMarkedGraduate = event;
}

// =============================================================================
// 7. I-6: admission number carried + can be reassigned
// =============================================================================

#[test]
fn student_record_admission_number_carried() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");
    assert_eq!(agg.admission_number, Some("ADM-2025-0001".to_string()));

    // Reassign admission number (e.g. on promotion).
    let (mut agg, _) = (agg, ());
    agg.set_admission_number("ADM-2026-0001".to_string(), agg.created_by, clock.now());
    assert_eq!(agg.admission_number, Some("ADM-2026-0001".to_string()));
}

// =============================================================================
// 8. set_roll_number happy path
// =============================================================================

#[test]
fn student_record_set_roll_number_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = enroll_student(cmd, &clock, &ids, &uniqueness).expect("enroll");

    let roll_cmd = SetRollNumberCommand {
        tenant,
        student_record_id: agg.id,
        roll_number: "12".to_string(),
    };
    let event = set_roll_number(roll_cmd, &mut agg, &clock, &ids, &uniqueness)
        .expect("set roll");
    assert_eq!(agg.roll_number, Some("12".to_string()));
    let _: RollNumberAssigned = event;
}
