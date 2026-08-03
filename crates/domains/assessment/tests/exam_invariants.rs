//! Wave 215 — Exam aggregate invariant tests.
//!
//! Verifies the 3 spec invariants from `docs/specs/assessment/aggregates.md`:
//! - I-1: unique by (exam_type_id, class_id, section_id, subject_id,
//!       academic_year_id) — enforced by ExamUniquenessChecker
//!       (port trait; tested in services layer).
//! - I-2: pass_mark <= exam_mark, both non-negative.
//! - I-3: cannot delete while MarksRegister rows reference — enforced
//!       by ExamReferenceChecker (port trait; tested in services
//!       layer).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::prelude::*;
use educore_core::error::DomainError;
use educore_core::ids::CorrelationId;
use educore_core::value_objects::Timestamp;

fn make_exam_args(s: educore_core::ids::SchoolId) -> ExamArgs {
    ExamArgs {
        school: s,
        exam_mark: 100.0,
        pass_mark: 35.0,
    }
}

struct ExamArgs {
    school: educore_core::ids::SchoolId,
    exam_mark: f32,
    pass_mark: f32,
}

#[test]
fn exam_i2_pass_mark_leq_exam_mark_happy() {
    let s = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let args = make_exam_args(s);
    let result = Exam::try_fresh(
        ExamId::new(s, uuid::Uuid::now_v7()),
        ExamTypeId::new(s, uuid::Uuid::now_v7()),
        ClassId::new(s, uuid::Uuid::now_v7()),
        SectionId::new(s, uuid::Uuid::now_v7()),
        SubjectId::new(s, uuid::Uuid::now_v7()),
        AcademicYearId::new(s, uuid::Uuid::now_v7()),
        ExamName::new("Mid-Term").unwrap(),
        ExamCode::new("MT-2024").unwrap(),
        ExamMark::new(args.exam_mark).unwrap(),
        PassMark::new(args.pass_mark).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    assert!(result.is_ok(), "happy path must succeed: {result:?}");
}

#[test]
fn exam_i2_rejects_pass_mark_gt_exam_mark() {
    let s = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let result = Exam::try_fresh(
        ExamId::new(s, uuid::Uuid::now_v7()),
        ExamTypeId::new(s, uuid::Uuid::now_v7()),
        ClassId::new(s, uuid::Uuid::now_v7()),
        SectionId::new(s, uuid::Uuid::now_v7()),
        SubjectId::new(s, uuid::Uuid::now_v7()),
        AcademicYearId::new(s, uuid::Uuid::now_v7()),
        ExamName::new("X").unwrap(),
        ExamCode::new("X").unwrap(),
        ExamMark::new(50.0).unwrap(),
        PassMark::new(60.0).unwrap(), // > exam_mark
        chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::Validation(msg)) => assert!(msg.contains("I-2")),
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn exam_i2_pass_mark_equal_exam_mark_ok() {
    let s = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let result = Exam::try_fresh(
        ExamId::new(s, uuid::Uuid::now_v7()),
        ExamTypeId::new(s, uuid::Uuid::now_v7()),
        ClassId::new(s, uuid::Uuid::now_v7()),
        SectionId::new(s, uuid::Uuid::now_v7()),
        SubjectId::new(s, uuid::Uuid::now_v7()),
        AcademicYearId::new(s, uuid::Uuid::now_v7()),
        ExamName::new("X").unwrap(),
        ExamCode::new("X").unwrap(),
        ExamMark::new(50.0).unwrap(),
        PassMark::new(50.0).unwrap(), // == exam_mark (edge case)
        chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    assert!(result.is_ok(), "pass_mark == exam_mark must be valid");
}

#[test]
fn exam_i2_tenant_boundary_rejects_cross_school() {
    let s1 = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let s2 = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let result = Exam::try_fresh(
        ExamId::new(s1, uuid::Uuid::now_v7()),
        ExamTypeId::new(s2, uuid::Uuid::now_v7()), // cross-school!
        ClassId::new(s1, uuid::Uuid::now_v7()),
        SectionId::new(s1, uuid::Uuid::now_v7()),
        SubjectId::new(s1, uuid::Uuid::now_v7()),
        AcademicYearId::new(s1, uuid::Uuid::now_v7()),
        ExamName::new("X").unwrap(),
        ExamCode::new("X").unwrap(),
        ExamMark::new(100.0).unwrap(),
        PassMark::new(35.0).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
        educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7()),
        Timestamp::now(),
        CorrelationId::from_uuid(uuid::Uuid::now_v7()),
    );
    match result {
        Err(DomainError::TenantViolation(_)) => {}
        other => panic!("expected TenantViolation, got {other:?}"),
    }
}

#[test]
fn exam_fresh_infallible_panics_on_i2_violation() {
    // `Exam::fresh` (infallible) panics if pass_mark > exam_mark.
    // This is the legacy contract; new code should use `try_fresh`.
    let s = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
    let result = std::panic::catch_unwind(|| {
        Exam::fresh(
            ExamId::new(s, uuid::Uuid::now_v7()),
            ExamTypeId::new(s, uuid::Uuid::now_v7()),
            ClassId::new(s, uuid::Uuid::now_v7()),
            SectionId::new(s, uuid::Uuid::now_v7()),
            SubjectId::new(s, uuid::Uuid::now_v7()),
            AcademicYearId::new(s, uuid::Uuid::now_v7()),
            ExamName::new("X").unwrap(),
            ExamCode::new("X").unwrap(),
            ExamMark::new(50.0).unwrap(),
            PassMark::new(60.0).unwrap(), // > exam_mark
            chrono::NaiveDate::from_ymd_opt(2024, 9, 15).unwrap(),
            educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7()),
            Timestamp::now(),
            CorrelationId::from_uuid(uuid::Uuid::now_v7()),
        )
    });
    assert!(result.is_err(), "fresh must panic on I-2 violation");
}
