//! Integration tests for the **LessonPlan aggregate** vertical slice.
//!
//! Pins the create / update / complete / sub-topic / delete
//! contracts for the `LessonPlan` aggregate end-to-end through
//! the service layer, exercising all 4 spec invariants:
//!
//! - I-1: Anchored to Lesson + topic + class-section + subject + date
//! - I-2: Sub-topics (Vec<SubTopic>, zero allowed)
//! - I-3: CompletedStatus enum (Pending/InProgress/Completed/Skipped)
//! - I-4: Single teacher owner (teacher_id immutable after creation)
//!
//! Test fixture pattern matches prior waves.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_academic::prelude::*;
use educore_academic::commands::{
    AddSubTopicCommand, DeleteLessonPlanCommand, MarkLessonPlanCompletedCommand,
    RealCreateLessonPlanCommand, UpdateLessonPlanCommand,
};
use educore_academic::events::{
    LessonPlanCompleted, LessonPlanDeleted, LessonPlanUpdated, RealLessonPlanCreated,
    SubTopicAdded,
};
use educore_academic::services::{
    add_sub_topic, create_lesson_plan, delete_lesson_plan, mark_lesson_plan_completed,
    update_lesson_plan,
};
use educore_academic::{CompletedStatus, RealLessonPlan};
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

fn lesson_plan_id(g: &SystemIdGen, school: SchoolId) -> LessonPlanId {
    LessonPlanId::new(school, g.next_uuid())
}

fn lesson_id(g: &SystemIdGen, school: SchoolId) -> LessonId {
    LessonId::new(school, g.next_uuid())
}

fn topic_id(g: &SystemIdGen, school: SchoolId) -> LessonTopicId {
    LessonTopicId::new(school, g.next_uuid())
}

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateLessonPlanCommand {
    RealCreateLessonPlanCommand {
        tenant,
        lesson_plan_id: lesson_plan_id(g, school),
        lesson_id: lesson_id(g, school),
        topic_id: topic_id(g, school),
        class_section_id: class_section_id(g, school),
        subject_id: subject_id(g, school),
        teacher_id: g.next_user_id(),
        scheduled_date: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        teaching_method: "Lecture + discussion".to_string(),
        objectives: "Students understand photosynthesis".to_string(),
        materials: vec!["Textbook chapter 5".to_string(), "Slides".to_string()],
    }
}

// =============================================================================
// 1. Happy path: create a LessonPlan
// =============================================================================

#[test]
fn lesson_plan_create_with_full_anchors_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    assert_eq!(agg.school_id, school);
    assert_eq!(agg.status, CompletedStatus::Pending);
    assert_eq!(agg.sub_topics.len(), 0);
    assert_eq!(agg.materials.len(), 2);

    assert_eq!(RealLessonPlanCreated::EVENT_TYPE, "academic.lesson_plan.created");
    assert_eq!(RealLessonPlanCreated::AGGREGATE_TYPE, "lesson_plan");
    assert_eq!(RealLessonPlanCreated::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: cross-school typed id rejected
// =============================================================================

#[test]
fn lesson_plan_create_with_cross_school_typed_id_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    let other_school = g.next_school_id();
    cmd.lesson_id = LessonId::new(other_school, g.next_uuid());

    let err = create_lesson_plan(cmd, &clock, &ids)
        .expect_err("cross-school typed id must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 3. I-2: sub-topics (zero allowed)
// =============================================================================

#[test]
fn lesson_plan_with_no_sub_topics_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");
    assert_eq!(agg.sub_topics.len(), 0);
}

#[test]
fn lesson_plan_add_sub_topic_appends() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    let sub_cmd = AddSubTopicCommand {
        tenant,
        lesson_plan_id: agg.id,
        title: "Light reactions".to_string(),
        description: "The light-dependent reactions of photosynthesis".to_string(),
    };
    let event = add_sub_topic(sub_cmd, &mut agg, &clock, &ids).expect("add_sub_topic");
    assert_eq!(agg.sub_topics.len(), 1);
    let _: SubTopicAdded = event;
}

// =============================================================================
// 4. I-3: CompletedStatus transitions
// =============================================================================

#[test]
fn lesson_plan_mark_completed_transitions_status() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");
    assert_eq!(agg.status, CompletedStatus::Pending);

    let comp_cmd = MarkLessonPlanCompletedCommand {
        tenant,
        lesson_plan_id: agg.id,
        final_status: CompletedStatus::Completed,
    };
    let event = mark_lesson_plan_completed(comp_cmd, &mut agg, &clock, &ids).expect("mark");
    assert_eq!(agg.status, CompletedStatus::Completed);
    let _: LessonPlanCompleted = event;
}

#[test]
fn lesson_plan_mark_completed_from_completed_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    // First transition: Pending -> Completed.
    let comp_cmd = MarkLessonPlanCompletedCommand {
        tenant: tenant.clone(),
        lesson_plan_id: agg.id,
        final_status: CompletedStatus::Completed,
    };
    mark_lesson_plan_completed(comp_cmd, &mut agg, &clock, &ids).expect("first mark");

    // Second transition: Completed -> Completed (no transition allowed).
    let comp_cmd2 = MarkLessonPlanCompletedCommand {
        tenant,
        lesson_plan_id: agg.id,
        final_status: CompletedStatus::Completed,
    };
    let err = mark_lesson_plan_completed(comp_cmd2, &mut agg, &clock, &ids)
        .expect_err("Completed -> Completed must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// 5. I-4: teacher_id immutable
// =============================================================================

#[test]
fn lesson_plan_update_teacher_id_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    let upd_cmd = UpdateLessonPlanCommand {
        tenant,
        lesson_plan_id: agg.id,
        teacher_id: g.next_user_id(), // different teacher
        scheduled_date: None,
        teaching_method: None,
        objectives: None,
        materials: None,
    };
    let err = update_lesson_plan(upd_cmd, &mut agg, &clock, &ids)
        .expect_err("changing teacher_id must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn lesson_plan_update_with_same_teacher_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    let upd_cmd = UpdateLessonPlanCommand {
        tenant,
        lesson_plan_id: agg.id,
        teacher_id: agg.teacher_id, // same teacher
        scheduled_date: Some(NaiveDate::from_ymd_opt(2025, 3, 8).unwrap()),
        teaching_method: None,
        objectives: None,
        materials: None,
    };
    let event = update_lesson_plan(upd_cmd, &mut agg, &clock, &ids).expect("update should succeed");
    assert_eq!(agg.scheduled_date, NaiveDate::from_ymd_opt(2025, 3, 8).unwrap());
    let _: LessonPlanUpdated = event;
}

// =============================================================================
// 6. Delete
// =============================================================================

#[test]
fn lesson_plan_delete_retires_aggregate() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_plan(cmd, &clock, &ids).expect("create should succeed");

    let del_cmd = DeleteLessonPlanCommand {
        tenant,
        lesson_plan_id: agg.id,
    };
    let event = delete_lesson_plan(del_cmd, &mut agg, &clock, &ids).expect("delete should succeed");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: LessonPlanDeleted = event;
}
