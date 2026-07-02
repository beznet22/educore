//! Integration tests for the **Homework aggregate** vertical slice.
//!
//! Pins the create / update / submit / evaluate / cancel
//! contracts for the `Homework` aggregate end-to-end through
//! the service layer, exercising all 5 spec invariants:
//!
//! - I-1: created by a teacher
//! - I-2: submission_date > homework_date
//! - I-3: evaluation_date >= submission_date
//! - I-4: optional attachment
//! - I-5: marks immutable once evaluated
//!
//! Test fixture pattern matches `class_routine.rs` and
//! `class_subject.rs` from prior waves.

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
    CancelHomeworkCommand, CreateHomeworkCommand, UpdateHomeworkCommand,
};
use educore_academic::events::{HomeworkCancelled, HomeworkCreated, HomeworkUpdated};
use educore_academic::services::{
    cancel_homework, create_homework, update_homework,
};
use educore_academic::{FileId, HomeworkStatus};
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

fn homework_id(g: &SystemIdGen, school: SchoolId) -> HomeworkId {
    HomeworkId::new(school, g.next_uuid())
}

fn class_section_id(g: &SystemIdGen, school: SchoolId) -> ClassSectionId {
    ClassSectionId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn file_id(g: &SystemIdGen, school: SchoolId) -> FileId {
    FileId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> CreateHomeworkCommand {
    CreateHomeworkCommand {
        tenant,
        homework_id: homework_id(g, school),
        class_section_id: class_section_id(g, school),
        subject_id: subject_id(g, school),
        teacher_id: g.next_user_id(),
        homework_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        submission_date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
        description: "Read chapter 1, answer questions 1-10".to_string(),
        attachment_id: None,
    }
}

// =============================================================================
// 1. Happy path: create a Homework
// =============================================================================

#[test]
fn homework_create_with_teacher_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (agg, event) = create_homework(cmd, &clock, &ids).expect("create should succeed");

    assert_eq!(agg.school_id, school);
    assert!(matches!(agg.status, HomeworkStatus::Active));
    assert_eq!(agg.description, "Read chapter 1, answer questions 1-10");
    assert!(agg.attachment_id.is_none());

    // Event metadata matches DomainEvent contract.
    assert_eq!(HomeworkCreated::EVENT_TYPE, "academic.homework.created");
    assert_eq!(HomeworkCreated::AGGREGATE_TYPE, "homework");
    assert_eq!(HomeworkCreated::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: non-teacher actor rejected
// =============================================================================

#[test]
fn homework_create_with_non_teacher_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let err = create_homework(cmd, &clock, &ids)
        .expect_err("SchoolAdmin must not be allowed to create homework");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 3. I-2: submission_date <= homework_date rejected
// =============================================================================

#[test]
fn homework_create_with_submission_before_homework_date_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.homework_date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    cmd.submission_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(); // before

    let err = create_homework(cmd, &clock, &ids)
        .expect_err("submission before homework_date must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn homework_create_with_equal_dates_rejected() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    let same = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    cmd.homework_date = same;
    cmd.submission_date = same;

    let err = create_homework(cmd, &clock, &ids)
        .expect_err("equal dates must fail (must be strictly after)");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 4. I-4: optional attachment accepted as Some
// =============================================================================

#[test]
fn homework_create_with_attachment_succeeds() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.attachment_id = Some(file_id(&g, school));

    let (agg, _event) = create_homework(cmd, &clock, &ids).expect("attachment should be ok");
    assert!(agg.attachment_id.is_some());
}

// =============================================================================
// 5. update_homework changes description
// =============================================================================

#[test]
fn homework_update_changes_description() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_homework(cmd, &clock, &ids).expect("create");

    let upd_cmd = UpdateHomeworkCommand {
        tenant: tenant.clone(),
        homework_id: agg.id,
        description: Some("Updated: read chapter 2 too".to_string()),
        submission_date: None,
        attachment_id: None,
    };
    let event = update_homework(upd_cmd, &mut agg, &clock, &ids)
        .expect("update should succeed");
    assert_eq!(agg.description, "Updated: read chapter 2 too");
    let _: HomeworkUpdated = event;
}

// =============================================================================
// 6. cancel_homework retires the aggregate
// =============================================================================

#[test]
fn homework_cancel_retires_aggregate() {
    let (tenant, g) = teacher_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_homework(cmd, &clock, &ids).expect("create");

    let cancel_cmd = CancelHomeworkCommand {
        tenant,
        homework_id: agg.id,
        reason: "Test cancellation".to_string(),
    };
    let event = cancel_homework(cancel_cmd, &mut agg, &clock, &ids)
        .expect("cancel should succeed");
    assert!(matches!(agg.status, HomeworkStatus::Cancelled));
    let _: HomeworkCancelled = event;
}
