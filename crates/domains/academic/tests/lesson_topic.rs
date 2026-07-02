//! Integration tests for the **LessonTopic aggregate** vertical slice.
//!
//! Pins the create / mark-completed / delete contracts for
//! the `LessonTopic` aggregate end-to-end through the service
//! layer, exercising all 2 spec invariants:
//!
//! - I-1: A topic belongs to one lesson
//! - I-2: A topic has CompletedStatus + CompletedDate if completed

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
    DeleteLessonTopicCommand, MarkTopicCompletedCommand, RealCreateLessonTopicCommand,
};
use educore_academic::events::{LessonTopicCompleted, LessonTopicDeleted, RealLessonTopicCreated};
use educore_academic::services::{create_lesson_topic, delete_lesson_topic, mark_topic_completed};
use educore_academic::RealLessonTopic;
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

fn lesson_topic_id(g: &SystemIdGen, school: SchoolId) -> LessonTopicId {
    LessonTopicId::new(school, g.next_uuid())
}

fn lesson_id(g: &SystemIdGen, school: SchoolId) -> LessonId {
    LessonId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateLessonTopicCommand {
    RealCreateLessonTopicCommand {
        tenant,
        lesson_topic_id: lesson_topic_id(g, school),
        lesson_id: lesson_id(g, school),
        title: "Light reactions".to_string(),
        description: "The light-dependent reactions of photosynthesis".to_string(),
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn lesson_topic_create_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_lesson_topic(cmd, &clock, &ids).expect("create should succeed");

    // I-1: belongs to one lesson
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.status, CompletedStatus::Pending);
    assert_eq!(agg.completed_date, None);

    assert_eq!(RealLessonTopicCreated::EVENT_TYPE, "academic.lesson_topic.created");
    assert_eq!(RealLessonTopicCreated::AGGREGATE_TYPE, "lesson_topic");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: cross-school lesson_id rejected
// =============================================================================

#[test]
fn lesson_topic_with_cross_school_lesson_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    let other_school = g.next_school_id();
    cmd.lesson_id = LessonId::new(other_school, g.next_uuid());

    let err = create_lesson_topic(cmd, &clock, &ids)
        .expect_err("cross-school lesson_id must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 3. I-2: mark completed sets status + completed_date
// =============================================================================

#[test]
fn lesson_topic_mark_completed_sets_status_and_date() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_topic(cmd, &clock, &ids).expect("create");

    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
    let mark_cmd = MarkTopicCompletedCommand {
        tenant,
        lesson_topic_id: agg.id,
        completed_date: date,
    };
    let event = mark_topic_completed(mark_cmd, &mut agg, &clock, &ids)
        .expect("mark completed");
    assert_eq!(agg.status, CompletedStatus::Completed);
    assert_eq!(agg.completed_date, Some(date));
    let _: LessonTopicCompleted = event;
}

// =============================================================================
// 4. I-2: cannot mark already-Completed topic
// =============================================================================

#[test]
fn lesson_topic_mark_completed_from_completed_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_topic(cmd, &clock, &ids).expect("create");

    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
    let first = MarkTopicCompletedCommand {
        tenant: tenant.clone(),
        lesson_topic_id: agg.id,
        completed_date: date,
    };
    mark_topic_completed(first, &mut agg, &clock, &ids).expect("first mark");

    let second = MarkTopicCompletedCommand {
        tenant,
        lesson_topic_id: agg.id,
        completed_date: date,
    };
    let err = mark_topic_completed(second, &mut agg, &clock, &ids)
        .expect_err("Completed -> Completed must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 5. Delete
// =============================================================================

#[test]
fn lesson_topic_delete_retires_aggregate() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_lesson_topic(cmd, &clock, &ids).expect("create");

    let del_cmd = DeleteLessonTopicCommand {
        tenant,
        lesson_topic_id: agg.id,
    };
    let event = delete_lesson_topic(del_cmd, &mut agg, &clock, &ids).expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: LessonTopicDeleted = event;
}
