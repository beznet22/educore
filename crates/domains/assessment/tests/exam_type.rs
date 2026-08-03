//! Wave 214 — ExamType aggregate behavioral tests.
//!
//! Verifies the 5 spec invariants from `docs/specs/assessment/aggregates.md`:
//! - I-1: title unique (non-empty after trim)
//! - I-2: percentage in [0, 100]
//! - I-3: is_average marker (no validation, just stored)
//! - I-4: average_mark non-negative
//! - I-5: parent_id same-school guard (tenant boundary)

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::prelude::*;
use educore_core::clock::{SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};

fn school() -> SchoolId {
    SchoolId::from_uuid(uuid::Uuid::now_v7())
}

fn other_school() -> SchoolId {
    SchoolId::from_uuid(uuid::Uuid::now_v7())
}

fn make_exam_type(
    s: SchoolId,
    title: &str,
    percentage: f64,
    is_average: bool,
    average_mark: f64,
    parent_id: Option<ExamTypeId>,
) -> ExamType {
    let id = ExamTypeId::new(s, uuid::Uuid::now_v7());
    ExamType::fresh(
        id,
        title.to_owned(),
        percentage,
        is_average,
        average_mark,
        parent_id,
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    )
    .expect("fresh ExamType must succeed")
}

#[test]
fn exam_type_i1_title_unique_non_empty() {
    let et = make_exam_type(school(), "Mid-Term", 30.0, false, 0.0, None);
    assert_eq!(et.title, "Mid-Term");
    assert!(et.is_active());
}

#[test]
fn exam_type_i1_rejects_empty_title() {
    let id = ExamTypeId::new(school(), uuid::Uuid::now_v7());
    let result = ExamType::fresh(
        id,
        "   ".to_owned(),
        30.0,
        false,
        0.0,
        None,
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::Validation(msg)) => assert!(msg.contains("I-1")),
        other => panic!("expected Validation error for empty title, got {other:?}"),
    }
}

#[test]
fn exam_type_i2_percentage_in_range() {
    let et0 = make_exam_type(school(), "Zero", 0.0, false, 0.0, None);
    assert_eq!(et0.percentage, 0.0);
    let et100 = make_exam_type(school(), "Hundred", 100.0, false, 0.0, None);
    assert_eq!(et100.percentage, 100.0);
}

#[test]
fn exam_type_i2_rejects_negative_percentage() {
    let id = ExamTypeId::new(school(), uuid::Uuid::now_v7());
    let result = ExamType::fresh(
        id,
        "X".to_owned(),
        -1.0,
        false,
        0.0,
        None,
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::Validation(msg)) => assert!(msg.contains("I-2")),
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn exam_type_i2_rejects_percentage_over_100() {
    let id = ExamTypeId::new(school(), uuid::Uuid::now_v7());
    let result = ExamType::fresh(
        id,
        "X".to_owned(),
        101.0,
        false,
        0.0,
        None,
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::Validation(msg)) => assert!(msg.contains("I-2")),
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn exam_type_i3_is_average_marker() {
    let et_avg = make_exam_type(school(), "Monthly Avg", 30.0, true, 80.0, None);
    assert!(et_avg.is_average);
    let et_single = make_exam_type(school(), "Monthly Single", 30.0, false, 0.0, None);
    assert!(!et_single.is_average);
}

#[test]
fn exam_type_i4_average_mark_non_negative() {
    let et0 = make_exam_type(school(), "AvgZero", 30.0, true, 0.0, None);
    assert_eq!(et0.average_mark, 0.0);
    let et = make_exam_type(school(), "AvgPos", 30.0, true, 80.0, None);
    assert_eq!(et.average_mark, 80.0);
}

#[test]
fn exam_type_i4_rejects_negative_average_mark() {
    let id = ExamTypeId::new(school(), uuid::Uuid::now_v7());
    let result = ExamType::fresh(
        id,
        "X".to_owned(),
        30.0,
        true,
        -1.0,
        None,
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::Validation(msg)) => assert!(msg.contains("I-4")),
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn exam_type_i5_parent_same_school() {
    let s = school();
    let parent_id = ExamTypeId::new(s, uuid::Uuid::now_v7());
    let et = make_exam_type(s, "Child", 30.0, false, 0.0, Some(parent_id));
    assert_eq!(et.parent_id, Some(parent_id));
}

#[test]
fn exam_type_i5_rejects_cross_school_parent() {
    let s = school();
    let foreign_parent = ExamTypeId::new(other_school(), uuid::Uuid::now_v7());
    let id = ExamTypeId::new(s, uuid::Uuid::now_v7());
    let result = ExamType::fresh(
        id,
        "X".to_owned(),
        30.0,
        false,
        0.0,
        Some(foreign_parent),
        UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::TenantViolation(_)) => {}
        other => panic!("expected TenantViolation, got {other:?}"),
    }
}

#[test]
fn exam_type_update_metadata_revalidates() {
    let mut et = make_exam_type(school(), "Original", 30.0, false, 0.0, None);
    let actor = UserId::from_uuid(uuid::Uuid::now_v7());
    let at = Timestamp::now();
    et.update_metadata("Updated".to_owned(), 50.0, true, 75.0, at, actor)
        .expect("update must succeed");
    assert_eq!(et.title, "Updated");
    assert_eq!(et.percentage, 50.0);
    assert!(et.is_average);
    assert_eq!(et.average_mark, 75.0);

    let result = et.update_metadata("   ".to_owned(), 50.0, true, 75.0, at, actor);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn exam_type_retire_soft_deletes() {
    let mut et = make_exam_type(school(), "X", 30.0, false, 0.0, None);
    let actor = UserId::from_uuid(uuid::Uuid::now_v7());
    et.retire(Timestamp::now(), actor);
    assert!(!et.is_active());
    assert_eq!(et.active_status, ActiveStatus::Retired);
}
