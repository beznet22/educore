//! Integration tests for the **Lesson aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for the
//! `Lesson` aggregate end-to-end through the service layer,
//! exercising all 3 spec invariants:
//!
//! - I-1: Unique by title within (class_section, subject)
//! - I-2: Zero or more topics (Vec<LessonTopicId>)
//! - I-3: Creation user + creation timestamp (structural)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;

use educore_academic::prelude::*;
use educore_academic::commands::{DeleteLessonCommand, RealCreateLessonCommand, UpdateLessonCommand};
use educore_academic::events::{LessonDeleted, LessonUpdated, RealLessonCreated};
use educore_academic::services::{create_lesson, delete_lesson, update_lesson};
use educore_academic::RealLesson;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;

// =============================================================================
// Fixtures
// =============================================================================

fn teacher_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::Teacher),
        g,
    )
}

fn lesson_id(g: &SystemIdGen, school: SchoolId) -> LessonId {
    LessonId::new(school, g.next_uuid())
}

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn topic_id(g: &SystemIdGen, school: SchoolId) -> LessonTopicId {
    LessonTopicId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateLessonCommand {
    RealCreateLessonCommand {
        tenant,
        lesson_id: lesson_id(g, school),
        class_section_id: class_section_id(g, school),
        subject_id: subject_id(g, school),
        title: "Photosynthesis".to_string(),
        description: "How plants convert sunlight to energy".to_string(),
    }
}

/// Minimal in-memory UniquenessChecker.
#[derive(Default)]
struct InMemoryUniqueness {
    lesson_titles: HashSet<(SchoolId, ClassSectionId, SubjectId, String)>,
}

impl UniquenessChecker for InMemoryUniqueness {
    fn student_admission_no_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn student_email_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn roll_no_exists(&self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId, _: &str) -> bool { false }
    fn class_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn section_name_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn subject_code_exists(&self, _: SchoolId, _: &str) -> bool { false }
    fn lesson_title_exists(&self, school: SchoolId, cs: ClassSectionId, sub: SubjectId, title: &str) -> bool {
        self.lesson_titles.contains(&(school, cs, sub, title.to_string()))
    }
    fn class_section_exists(&self, _: SchoolId, _: ClassId, _: SectionId, _: AcademicYearId) -> bool { false }
    fn class_section_has_student_records(&self, _: SchoolId, _: ClassSectionId) -> bool { false }
    fn academic_year_overlaps(&self, _: SchoolId, _: AcademicYearRange, _: Option<AcademicYearId>) -> bool { false }
    fn optional_subject_assigned_exists(&self, _: SchoolId, _: StudentId, _: AcademicYearId) -> bool { false }
    fn primary_guardian_link_exists(&self, _: SchoolId, _: StudentId) -> bool { false }
    fn teacher_has_conflict(&self, _: SchoolId, _: UserId, _: DayOfWeek, _: u8) -> bool { false }
    fn room_has_conflict(&self, _: SchoolId, _: ClassRoomId, _: DayOfWeek, _: u8) -> bool { false }

    fn student_has_active_record(
        &self, _: SchoolId, _: StudentId, _: AcademicYearId,
    ) -> bool {
        false
    }

    fn student_category_name_exists(
        &self, _: SchoolId, _: &str,
    ) -> bool {
        false
    }
}

// =============================================================================
// 1. Happy path: create a Lesson
// =============================================================================

#[test]
fn lesson_create_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_lesson(cmd, &clock, &ids, &uniqueness)
        .expect("create should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.title, "Photosynthesis");
    assert_eq!(agg.topic_ids.len(), 0); // I-2: zero or more
    assert_eq!(agg.created_by, agg.updated_by); // I-3

    assert_eq!(RealLessonCreated::EVENT_TYPE, "academic.lesson.created");
    assert_eq!(RealLessonCreated::AGGREGATE_TYPE, "lesson");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: duplicate title rejected
// =============================================================================

#[test]
fn lesson_with_duplicate_title_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let cs = cmd.class_section_id;
    let sub = cmd.subject_id;

    let mut uniqueness = InMemoryUniqueness::default();
    uniqueness.lesson_titles.insert((school, cs, sub, "Photosynthesis".to_string()));

    let err = create_lesson(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate title must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 3. I-2: zero topics allowed
// =============================================================================

#[test]
fn lesson_with_zero_topics_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant, &g, school);
    let (agg, _event) = create_lesson(cmd, &clock, &ids, &uniqueness)
        .expect("create with 0 topics");
    assert_eq!(agg.topic_ids.len(), 0);
}

#[test]
fn lesson_add_topic_appends() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson(cmd, &clock, &ids, &uniqueness).expect("create");

    agg.add_topic(topic_id(&g, school), tenant.actor_id, clock.now());
    assert_eq!(agg.topic_ids.len(), 1);
}

// =============================================================================
// 4. Update + delete
// =============================================================================

#[test]
fn lesson_update_changes_description() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson(cmd, &clock, &ids, &uniqueness).expect("create");

    let upd = UpdateLessonCommand {
        tenant: tenant.clone(),
        lesson_id: agg.id,
        title: None,
        description: Some("Updated description".to_string()),
    };
    let event = update_lesson(upd, &mut agg, &clock, &ids, &uniqueness).expect("update");
    assert_eq!(agg.description, "Updated description");
    let _: LessonUpdated = event;
}

#[test]
fn lesson_update_with_duplicate_title_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson(cmd, &clock, &ids, &uniqueness).expect("create");

    let cs = agg.class_section_id;
    let sub = agg.subject_id;
    uniqueness.lesson_titles.insert((school, cs, sub, "Cellular Respiration".to_string()));

    let upd = UpdateLessonCommand {
        tenant,
        lesson_id: agg.id,
        title: Some("Cellular Respiration".to_string()),
        description: None,
    };
    let err = update_lesson(upd, &mut agg, &clock, &ids, &uniqueness).expect_err("duplicate");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

#[test]
fn lesson_delete_retires_aggregate() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = InMemoryUniqueness::default();

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson(cmd, &clock, &ids, &uniqueness).expect("create");

    let del_cmd = DeleteLessonCommand {
        tenant,
        lesson_id: agg.id,
    };
    let event = delete_lesson(del_cmd, &mut agg, &clock, &ids).expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: LessonDeleted = event;
}
