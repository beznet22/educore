//! Integration tests for the **Exam aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Exam`](educore_assessment::aggregate::Exam) end-to-end
//! through the service layer:
//!
//! 1. `create_exam` validates the input (the typed
//!    [`ExamName`](educore_assessment::value_objects::ExamName),
//!    [`ExamCode`](educore_assessment::value_objects::ExamCode),
//!    [`ExamMark`](educore_assessment::value_objects::ExamMark),
//!    and [`PassMark`](educore_assessment::value_objects::PassMark)
//!    newtypes enforce length / charset / range invariants
//!    at command construction), asserts the
//!    `(school, academic_year, exam_type, class, section,
//!    subject)` uniqueness invariant via the
//!    [`AssessmentUniquenessChecker`] port, enforces
//!    `pass_mark <= exam_mark`, constructs the aggregate,
//!    and emits an [`ExamCreated`](educore_assessment::events::ExamCreated)
//!    event.
//!
//! The tests use the same fixture pattern as
//! `tests/aggregates.rs` in the library crate (`TestClock` +
//! `SystemIdGen` + in-memory
//! `AssessmentUniquenessChecker`). The **handlers** themselves
//! are not wired end-to-end (no subscriber fan-out, no outbox
//! commit, no audit row). These tests pin the contract of the
//! **service layer** that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = UserId(g.next_uuid());
    let corr = CorrelationId(g.next_uuid());
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

/// No-op [`AssessmentUniquenessChecker`]: every exam
/// unique-key is reported as fresh. The happy-path test
/// does not exercise the uniqueness-conflict branch — that
/// lives in `tests/workflows.rs` already.
struct NoopUniqueness;

impl AssessmentUniquenessChecker for NoopUniqueness {
    fn exam_unique_key_exists(
        &self,
        _school: SchoolId,
        _academic_year: AcademicYearId,
        _exam_type: ExamTypeId,
        _class: ClassId,
        _section: SectionId,
        _subject: SubjectId,
    ) -> bool {
        false
    }
}

// =============================================================================
// Happy path: create_exam end-to-end
// =============================================================================

/// End-to-end happy path for the `Exam` aggregate. Creates
/// an exam "Mid-term" with code "MID101", full mark 100,
/// pass mark 40, asserting that:
///
/// 1. The create flow produces an `Exam` aggregate carrying
///    every field on the command and a fresh `version = 1`.
/// 2. The aggregate is anchored to the tenant's school and
///    starts in the `Active` / unpublished state.
/// 3. The emitted event is `ExamCreated` with the right
///    `event_type`, `aggregate_type`, `school_id`, and
///    `schema_version`.
#[test]
fn exam_create_produces_aggregate_and_exam_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = NoopUniqueness;

    let cmd = CreateExamCommand {
        tenant: tenant.clone(),
        exam_id: ExamId::new(school, g.next_uuid()),
        exam_type_id: ExamTypeId::new(school, g.next_uuid()),
        class_id: ClassId::new(school, g.next_uuid()),
        section_id: SectionId::new(school, g.next_uuid()),
        subject_id: SubjectId::new(school, g.next_uuid()),
        academic_year_id: AcademicYearId::new(school, g.next_uuid()),
        name: "Mid-term".to_owned(),
        code: "MID101".to_owned(),
        exam_mark: 100.0,
        pass_mark: 40.0,
        exam_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 15).expect("valid date"),
    };

    let (exam, event) = create_exam(cmd, &clock, &ids, &uniqueness).expect("create_exam");

    // Aggregate fields are populated from the command.
    assert_eq!(exam.school_id, school);
    assert_eq!(exam.name.as_str(), "Mid-term");
    assert_eq!(exam.code.as_str(), "MID101");
    assert_eq!(exam.exam_mark.as_f32(), 100.0);
    assert_eq!(exam.pass_mark.as_f32(), 40.0);
    // Audit metadata footer is initialised.
    assert_eq!(exam.version.get(), 1);
    assert!(exam.is_active());
    assert!(!exam.is_published());
    assert_eq!(exam.created_by, tenant.actor_id);
    assert_eq!(exam.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <ExamCreated as DomainEvent>::EVENT_TYPE,
        "assessment.exam.created"
    );
    assert_eq!(
        <ExamCreated as DomainEvent>::AGGREGATE_TYPE,
        "exam"
    );
    assert_eq!(<ExamCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), exam.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.exam_id, exam.id);
    assert_eq!(event.name, "Mid-term");
    assert_eq!(event.code, "MID101");
    assert_eq!(event.exam_mark, 100.0);
    assert_eq!(event.pass_mark, 40.0);
}

// =============================================================================
// Validation failure: pass_mark > exam_mark
// =============================================================================

/// Validation-failure path on the create flow: when
/// `pass_mark > exam_mark`, `create_exam` returns
/// [`DomainError::Validation`] before any aggregate is
/// constructed or event minted. The service enforces this
/// invariant after the per-field validators run, so the
/// typed `ExamMark` / `PassMark` newtypes already passed and
/// the rejection comes from the cross-field check.
#[test]
fn exam_create_rejects_pass_mark_greater_than_exam_mark() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let uniqueness = NoopUniqueness;

    let cmd = CreateExamCommand {
        tenant,
        exam_id: ExamId::new(school, g.next_uuid()),
        exam_type_id: ExamTypeId::new(school, g.next_uuid()),
        class_id: ClassId::new(school, g.next_uuid()),
        section_id: SectionId::new(school, g.next_uuid()),
        subject_id: SubjectId::new(school, g.next_uuid()),
        academic_year_id: AcademicYearId::new(school, g.next_uuid()),
        name: "Mid-term".to_owned(),
        code: "MID101".to_owned(),
        exam_mark: 100.0,
        pass_mark: 101.0, // > exam_mark — must fail
        exam_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 15).expect("valid date"),
    };

    let err = create_exam(cmd, &clock, &ids, &uniqueness)
        .expect_err("pass_mark > exam_mark must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
