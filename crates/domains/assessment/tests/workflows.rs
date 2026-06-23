//! Integration tests for the **assessment domain workflows**.
//!
//! Implements: `docs/specs/assessment/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the assessment service functions and asserts that
//! the expected typed event is emitted (or, on the error
//! path, that the expected [`DomainError`] is returned and
//! no event is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! assessment service factories (`create_exam`,
//! `update_exam`, `delete_exam`, `schedule_exam`,
//! `initialize_marks_register`, `enter_marks`,
//! `submit_marks`, `publish_result`,
//! `generate_report_card`, ...) are sync, take a `Clock` and
//! an `IdGenerator`, and return the mutated aggregate plus
//! the typed event. The test wires a [`TestClock`] and a
//! [`SystemIdGen`], and an in-memory
//! [`AssessmentUniquenessChecker`] for the workflows that
//! need one.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no
//! subscriber fan-out, no outbox commit, no audit row).
//! These tests pin the contract of the **service layer**
//! that the dispatcher will eventually wrap. When the
//! handlers land, the same test bodies will gain a
//! `+ outbox + bus subscriber` assertion without changes
//! to the assertions on the returned event.
//!
//! Coverage per `docs/specs/assessment/workflows.md`:
//!
//! - **§ Exam Authoring Workflow** -> `create_exam`
//!   happy/error + `update_exam` + `delete_exam`.
//! - **§ Exam Scheduling Workflow** -> `schedule_exam` +
//!   `update_exam_schedule` + `cancel_exam_schedule`.
//! - **§ Marks Entry Workflow** -> `initialize_marks_register`
//!   + `enter_marks` + `submit_marks` +
//!   `cancel_marks_register`.
//! - **§ Result Publication Workflow** -> `publish_result` +
//!   `republish_result` + `update_result_remarks`.
//! - **§ Report Card Generation Workflow** ->
//!   `generate_report_card`.
//! - **`ResultService`** (grading engine) -> `compute_grade`,
//!   `compute_subject_marks`, `compute_total`,
//!   `determine_pass_fail`, `rank_section` (tied-rank
//!   invariant), `validate_no_overlap`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;
use std::sync::Mutex;

use chrono::{NaiveDate, NaiveTime};

use educore_academic::value_objects::{
    AcademicYearId, ClassId, SectionId, SubjectId,
};
use educore_assessment::commands::{PublishResultCommand, ScheduleSubjectEntry};
use educore_assessment::prelude::*;
use educore_core::clock::{DeterministicIdGen, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::ActiveStatus;
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Test fixtures
// =============================================================================

/// In-memory `AssessmentUniquenessChecker` used by the
/// `create_exam` workflow.
#[derive(Debug, Default)]
struct InMemoryExamUniqueness {
    keys: Mutex<HashSet<ExamUniqueKey>>,
}

type ExamUniqueKey = (
    SchoolId,
    AcademicYearId,
    ExamTypeId,
    ClassId,
    SectionId,
    SubjectId,
);

impl InMemoryExamUniqueness {
    fn new() -> Self {
        Self::default()
    }
    fn seed(&self, key: ExamUniqueKey) {
        self.keys.lock().unwrap().insert(key);
    }
}

impl AssessmentUniquenessChecker for InMemoryExamUniqueness {
    fn exam_unique_key_exists(
        &self,
        school: SchoolId,
        academic_year: AcademicYearId,
        exam_type: ExamTypeId,
        class: ClassId,
        section: SectionId,
        subject: SubjectId,
    ) -> bool {
        self.keys
            .lock()
            .unwrap()
            .contains(&(school, academic_year, exam_type, class, section, subject))
    }
}

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
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

fn exam_id(g: &SystemIdGen, school: SchoolId) -> ExamId {
    ExamId::new(school, g.next_uuid())
}

fn exam_type_id(g: &SystemIdGen, school: SchoolId) -> ExamTypeId {
    ExamTypeId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn marks_register_id(g: &SystemIdGen, school: SchoolId) -> MarksRegisterId {
    MarksRegisterId::new(school, g.next_uuid())
}

fn result_store_id(g: &SystemIdGen, school: SchoolId) -> ResultStoreId {
    ResultStoreId::new(school, g.next_uuid())
}

fn schedule_id(g: &SystemIdGen, school: SchoolId) -> ExamScheduleId {
    ExamScheduleId::new(school, g.next_uuid())
}

fn naive_date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn naive_time(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

/// Build a `CreateExamCommand` anchored to `school` with
/// sensible defaults. Tests override individual fields after
/// construction to exercise error paths.
fn make_create_exam(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> CreateExamCommand {
    CreateExamCommand {
        tenant,
        exam_id: exam_id(g, school),
        exam_type_id: exam_type_id(g, school),
        class_id: class_id(g, school),
        section_id: section_id(g, school),
        subject_id: subject_id(g, school),
        academic_year_id: year_id(g, school),
        name: "Mid-Term Mathematics".to_owned(),
        code: "MTH-MT-2026".to_owned(),
        exam_mark: 100.0,
        pass_mark: 35.0,
        exam_date: naive_date(2026, 9, 15),
    }
}

/// Pulls the `TenantContext` we stashed in the
/// `correlation_id` field of a freshly-minted `Exam` --
/// production code would persist this in a column; in the
/// test fixture the correlation id round-trips through
/// `command -> aggregate -> event`. We rebuild the actor
/// from a fresh id because the `Exam` aggregate does not
/// store the actor (the dispatcher, Phase 16, sources the
/// actor from the inbound command at the boundary).
fn exam_tenant(exam: &Exam) -> TenantContext {
    let g = SystemIdGen;
    TenantContext::for_user(
        exam.school_id,
        g.next_user_id(),
        exam.correlation_id,
        UserType::SchoolAdmin,
    )
}

// =============================================================================
// 1. Exam Authoring Workflow
//    (`workflows.md` § "Exam Authoring Workflow")
// =============================================================================

/// Authoring step 1: [`CreateExamCommand`] must emit
/// [`ExamCreated`] carrying the same tuple the engine
/// asserted uniqueness against.
#[test]
fn create_exam_happy_path_emits_exam_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let cmd = make_create_exam(tenant, &g, school);
    let (exam, event): (Exam, ExamCreated) =
        create_exam(cmd, &clock, &ids, &uniqueness).unwrap();

    assert_eq!(<ExamCreated as DomainEvent>::EVENT_TYPE, "assessment.exam.created");
    assert_eq!(event.exam_id, exam.id);
    assert_eq!(event.aggregate_id(), exam.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.exam_mark, 100.0);
    assert_eq!(event.pass_mark, 35.0);
    assert_eq!(event.name, "Mid-Term Mathematics");
    assert_eq!(event.code, "MTH-MT-2026");
    assert_eq!(exam.school_id, school);
    assert!(!exam.is_published());
    assert_eq!(exam.correlation_id, event.correlation_id);
}

/// Authoring step 2: [`UpdateExamCommand`] mutates the
/// existing exam and emits [`ExamUpdated`] with the names
/// of the fields that actually changed.
#[test]
fn update_exam_happy_path_emits_exam_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let cmd = make_create_exam(tenant, &g, school);
    let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).unwrap();
    let initial_version = exam.version.get();

    let upd = UpdateExamCommand {
        tenant: exam_tenant(&exam),
        exam_id: exam.id,
        name: None,
        code: None,
        exam_mark: Some(120.0),
        pass_mark: Some(40.0),
        exam_date: None,
        is_published: Some(true),
    };
    let event: ExamUpdated =
        update_exam(&exam_tenant(&exam), &mut exam, upd, &clock, &ids).unwrap();
    assert_eq!(<ExamUpdated as DomainEvent>::EVENT_TYPE, "assessment.exam.updated");
    assert_eq!(event.aggregate_id(), exam.id.as_uuid());
    assert_eq!(exam.version.get(), initial_version + 1);
    assert_eq!(exam.exam_mark.as_f32(), 120.0);
    assert_eq!(exam.pass_mark.as_f32(), 40.0);
    assert!(exam.is_published());
    assert!(event.changes.contains(&"exam_mark".to_owned()));
    assert!(event.changes.contains(&"pass_mark".to_owned()));
    assert!(event.changes.contains(&"is_published".to_owned()));
}

/// Authoring step 3: deleting an exam emits [`ExamDeleted`]
/// and retires the aggregate (no more mutators allowed).
#[test]
fn delete_exam_happy_path_emits_exam_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let cmd = make_create_exam(tenant, &g, school);
    let (mut exam, _ev) = create_exam(cmd, &clock, &ids, &uniqueness).unwrap();

    let del = DeleteExamCommand {
        tenant: exam_tenant(&exam),
        exam_id: exam.id,
    };
    let event: ExamDeleted =
        delete_exam(&exam_tenant(&exam), &mut exam, del, &clock, &ids).unwrap();
    assert_eq!(<ExamDeleted as DomainEvent>::EVENT_TYPE, "assessment.exam.deleted");
    assert_eq!(event.aggregate_id(), exam.id.as_uuid());
    assert!(!exam.is_active());
}

/// Authoring failure path: pass_mark > exam_mark must be
/// rejected with `DomainError::Validation` (per the
/// `pass_mark <= exam_mark` invariant in `create_exam`).
#[test]
fn create_exam_pass_mark_above_exam_mark_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let mut cmd = make_create_exam(tenant, &g, school);
    cmd.pass_mark = 110.0; // > exam_mark 100.0
    let err = create_exam(cmd, &clock, &ids, &uniqueness).expect_err("must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

/// Authoring failure path: a duplicate
/// `(school, academic_year, exam_type, class, section,
/// subject)` tuple must be rejected with
/// `DomainError::Conflict` (per the spec's "Duplicate Exam
/// -> ValidationError::UniqueViolation" path).
#[test]
fn create_exam_duplicate_unique_key_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let cmd = make_create_exam(tenant, &g, school);
    // Pre-seed the unique key.
    uniqueness.seed((
        school,
        cmd.academic_year_id,
        cmd.exam_type_id,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
    ));

    let err = create_exam(cmd, &clock, &ids, &uniqueness).expect_err("must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

/// Authoring failure path: an empty exam name must be
/// rejected with `DomainError::Validation`.
#[test]
fn create_exam_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = InMemoryExamUniqueness::new();

    let mut cmd = make_create_exam(tenant, &g, school);
    cmd.name = String::new();
    let err = create_exam(cmd, &clock, &ids, &uniqueness).expect_err("must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 2. Exam Scheduling Workflow
//    (`workflows.md` § "Exam Scheduling Workflow")
// =============================================================================

/// Scheduling step 1: [`ScheduleExamCommand`] emits
/// [`ExamScheduled`] with the subject count from the
/// command's `subjects` vector.
#[test]
fn schedule_exam_happy_path_emits_exam_scheduled() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = ScheduleExamCommand {
        tenant,
        schedule_id: schedule_id(&g, school),
        exam_id: exam_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        date: naive_date(2026, 9, 16),
        start_time: naive_time(9, 0),
        end_time: naive_time(11, 0),
        subjects: vec![
            ScheduleSubjectEntry {
                subject_id: subject_id(&g, school),
                date: naive_date(2026, 9, 16),
                start_time: naive_time(9, 0),
                end_time: naive_time(11, 0),
                room_id: None,
                full_mark: 100.0,
                pass_mark: 35.0,
            },
            ScheduleSubjectEntry {
                subject_id: subject_id(&g, school),
                date: naive_date(2026, 9, 17),
                start_time: naive_time(9, 0),
                end_time: naive_time(11, 0),
                room_id: None,
                full_mark: 100.0,
                pass_mark: 35.0,
            },
        ],
    };
    let (schedule, event): (ExamSchedule, ExamScheduled) =
        schedule_exam(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <ExamScheduled as DomainEvent>::EVENT_TYPE,
        "assessment.exam_scheduled.created"
    );
    assert_eq!(event.schedule_id, schedule.id);
    assert_eq!(event.aggregate_id(), schedule.id.as_uuid());
    assert_eq!(event.subject_count, 2);
    assert_eq!(event.date, schedule.date);
}

/// Scheduling update: [`UpdateExamScheduleCommand`]
/// mutates the existing schedule and emits
/// [`ExamScheduleUpdated`].
#[test]
fn update_exam_schedule_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = ScheduleExamCommand {
        tenant,
        schedule_id: schedule_id(&g, school),
        exam_id: exam_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        date: naive_date(2026, 9, 16),
        start_time: naive_time(9, 0),
        end_time: naive_time(11, 0),
        subjects: vec![],
    };
    let (mut schedule, _ev) = schedule_exam(cmd, &clock, &ids).unwrap();
    let initial_version = schedule.version.get();

    let upd = UpdateExamScheduleCommand {
        tenant: TenantContext::clone(&TenantContext::for_user(
            school,
            SystemIdGen.next_user_id(),
            schedule.correlation_id,
            UserType::SchoolAdmin,
        )),
        schedule_id: schedule.id,
        date: Some(naive_date(2026, 9, 17)),
        start_time: None,
        end_time: None,
    };
    let event: ExamScheduleUpdated =
        update_exam_schedule(&mut schedule, upd, &clock, &ids).unwrap();
    assert_eq!(
        <ExamScheduleUpdated as DomainEvent>::EVENT_TYPE,
        "assessment.exam_scheduled.updated"
    );
    assert_eq!(event.aggregate_id(), schedule.id.as_uuid());
    assert_eq!(schedule.version.get(), initial_version + 1);
    assert_eq!(schedule.date, naive_date(2026, 9, 17));
}

/// Scheduling cancel: [`CancelExamScheduleCommand`]
/// retires the schedule and emits
/// [`ExamScheduleCancelled`].
#[test]
fn cancel_exam_schedule_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = ScheduleExamCommand {
        tenant,
        schedule_id: schedule_id(&g, school),
        exam_id: exam_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        date: naive_date(2026, 9, 16),
        start_time: naive_time(9, 0),
        end_time: naive_time(11, 0),
        subjects: vec![],
    };
    let (mut schedule, _ev) = schedule_exam(cmd, &clock, &ids).unwrap();
    let cancel = CancelExamScheduleCommand {
        tenant: TenantContext::for_user(
            school,
            SystemIdGen.next_user_id(),
            schedule.correlation_id,
            UserType::SchoolAdmin,
        ),
        schedule_id: schedule.id,
        reason: "Holiday on exam date".to_owned(),
    };
    let event: ExamScheduleCancelled =
        cancel_exam_schedule(&mut schedule, cancel, &clock, &ids).unwrap();
    assert_eq!(
        <ExamScheduleCancelled as DomainEvent>::EVENT_TYPE,
        "assessment.exam_scheduled.cancelled"
    );
    assert_eq!(event.aggregate_id(), schedule.id.as_uuid());
    assert_eq!(event.reason, "Holiday on exam date");
    assert_eq!(schedule.active_status, ActiveStatus::Retired);
}

// =============================================================================
// 3. Marks Entry Workflow
//    (`workflows.md` § "Marks Entry Workflow")
// =============================================================================

/// Marks entry step 1: [`InitializeMarksRegisterCommand`]
/// emits [`MarksRegisterCreated`] and returns an open
/// (is_open=true) marks register aggregate.
#[test]
fn initialize_marks_register_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = InitializeMarksRegisterCommand {
        tenant,
        marks_register_id: marks_register_id(&g, school),
        exam_id: exam_id(&g, school),
        student_id: student_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        academic_year_id: year_id(&g, school),
        subject_ids: vec![subject_id(&g, school), subject_id(&g, school)],
    };
    let (register, event): (MarksRegister, MarksRegisterCreated) =
        initialize_marks_register(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <MarksRegisterCreated as DomainEvent>::EVENT_TYPE,
        "assessment.marks_register.created"
    );
    assert_eq!(event.marks_register_id, register.id);
    assert_eq!(event.aggregate_id(), register.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert!(register.is_open);
}

/// Marks entry step 2: [`EnterMarksCommand`] emits
/// [`MarksEntered`] for a single (student, subject) row.
/// Absent students get `is_absent=true` and `marks=None`
/// (per the spec's absent-handling rule).
#[test]
fn enter_marks_happy_path_emits_marks_entered() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = EnterMarksCommand {
        tenant,
        marks_register_id: marks_register_id(&g, school),
        subject_id: subject_id(&g, school),
        student_id: student_id(&g, school),
        marks: Some(85.0),
        is_absent: false,
        comment: None,
    };
    let event: MarksEntered = enter_marks(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <MarksEntered as DomainEvent>::EVENT_TYPE,
        "assessment.marks.entered"
    );
    assert_eq!(event.marks, Some(85.0));
    assert!(!event.is_absent);
}

/// Marks entry absent-handling: `is_absent=true` carries
/// `marks=None` (per the spec's "is_absent=true; marks is
/// treated as zero" rule).
#[test]
fn enter_marks_absent_student_emits_event_with_none_marks() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = EnterMarksCommand {
        tenant,
        marks_register_id: marks_register_id(&g, school),
        subject_id: subject_id(&g, school),
        student_id: student_id(&g, school),
        marks: None,
        is_absent: true,
        comment: Some("Medical leave".to_owned()),
    };
    let event: MarksEntered = enter_marks(cmd, &clock, &ids).unwrap();
    assert_eq!(event.marks, None);
    assert!(event.is_absent);
    assert_eq!(
        <MarksEntered as DomainEvent>::EVENT_TYPE,
        "assessment.marks.entered"
    );
}

/// Marks entry step 3: [`SubmitMarksCommand`] locks the
/// register and emits [`MarksSubmitted`].
#[test]
fn submit_marks_happy_path_emits_marks_submitted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = SubmitMarksCommand {
        tenant,
        marks_register_id: marks_register_id(&g, school),
    };
    let event: MarksSubmitted = submit_marks(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <MarksSubmitted as DomainEvent>::EVENT_TYPE,
        "assessment.marks.submitted"
    );
    assert_eq!(event.aggregate_id(), event.marks_register_id.as_uuid());
    assert_eq!(event.school_id(), school);
}

/// Marks entry: [`cancel_marks_register`] emits
/// [`MarksRegisterCancelled`].
#[test]
fn cancel_marks_register_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = SubmitMarksCommand {
        tenant,
        marks_register_id: marks_register_id(&g, school),
    };
    let event: MarksRegisterCancelled =
        cancel_marks_register(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <MarksRegisterCancelled as DomainEvent>::EVENT_TYPE,
        "assessment.marks_register.cancelled"
    );
    assert_eq!(event.aggregate_id(), event.marks_register_id.as_uuid());
}

// =============================================================================
// 4. Result Publication Workflow
//    (`workflows.md` § "Result Publication Workflow")
// =============================================================================

/// Publication step 4: [`PublishResultCommand`] emits
/// [`ResultPublished`] carrying the student count the
/// engine materialised.
#[test]
fn publish_result_happy_path_emits_result_published() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = PublishResultCommand {
        tenant,
        exam_id: exam_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        academic_year_id: year_id(&g, school),
        published_at: clock.now(),
    };
    let event: ResultPublished = publish_result(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <ResultPublished as DomainEvent>::EVENT_TYPE,
        "assessment.result.published"
    );
    assert_eq!(event.school_id(), school);
}

/// Republish step: [`RepublishResultCommand`] emits
/// [`ResultRepublished`] (the previous `ResultPublished`
/// remains in the event log per the spec).
#[test]
fn republish_result_happy_path_emits_result_republished() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = RepublishResultCommand {
        tenant,
        result_store_id: result_store_id(&g, school),
        reason: "Attendance reconciliation".to_owned(),
        republished_at: clock.now(),
    };
    let event: ResultRepublished = republish_result(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <ResultRepublished as DomainEvent>::EVENT_TYPE,
        "assessment.result.republished"
    );
    assert_eq!(event.reason, "Attendance reconciliation");
}

/// Teacher remarks: [`UpdateResultRemarksCommand`] emits
/// [`ResultRemarksUpdated`].
#[test]
fn update_result_remarks_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = UpdateResultRemarksCommand {
        tenant,
        result_store_id: result_store_id(&g, school),
        teacher_remarks: "Excellent progress in Q3.".to_owned(),
    };
    let event: ResultRemarksUpdated = update_result_remarks(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <ResultRemarksUpdated as DomainEvent>::EVENT_TYPE,
        "assessment.result_store.remarks_updated"
    );
    assert_eq!(event.teacher_remarks, "Excellent progress in Q3.");
}

// =============================================================================
// 5. Report Card Generation Workflow
//    (`workflows.md` § "Report Card Generation Workflow")
// =============================================================================

/// Report card step 2: [`GenerateReportCardCommand`] emits
/// [`ReportCardGenerated`] for the (student, result) pair.
/// `include_remarks` toggles whether teacher remarks are
/// baked into the payload.
#[test]
fn generate_report_card_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = GenerateReportCardCommand {
        tenant,
        result_store_id: result_store_id(&g, school),
        student_id: student_id(&g, school),
        include_remarks: true,
    };
    let event: ReportCardGenerated = generate_report_card(cmd, &clock, &ids).unwrap();
    assert_eq!(
        <ReportCardGenerated as DomainEvent>::EVENT_TYPE,
        "assessment.report_card.generated"
    );
    assert_eq!(event.student_id.school_id(), school);
    assert!(event.include_remarks);
}

/// Report card with `include_remarks=false` produces a
/// streamlined payload (no remarks section).
#[test]
fn generate_report_card_without_remarks_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);

    let cmd = GenerateReportCardCommand {
        tenant,
        result_store_id: result_store_id(&g, school),
        student_id: student_id(&g, school),
        include_remarks: false,
    };
    let event: ReportCardGenerated = generate_report_card(cmd, &clock, &ids).unwrap();
    assert!(!event.include_remarks);
}

// =============================================================================
// 6. ResultService -- grading engine
//    (`workflows.md` § "Result Publication Workflow" step 5)
// =============================================================================

/// Grading rule: a percent of 85.0 maps to grade "A" with
/// GPA 4.0 (per the standard A-F scale in `compute_grade`).
#[test]
fn result_service_compute_grade_returns_a_at_85_percent() {
    let (grade, gpa) = ResultService::compute_grade(85.0);
    assert_eq!(grade.as_str(), "A");
    assert_eq!(gpa.as_f32(), 4.0);
}

/// Grading rule: a percent below the failing threshold
/// (33%) maps to grade "F" with GPA 0.0.
#[test]
fn result_service_compute_grade_returns_f_below_33_percent() {
    let (grade, gpa) = ResultService::compute_grade(20.0);
    assert_eq!(grade.as_str(), "F");
    assert_eq!(gpa.as_f32(), 0.0);
}

/// Per-subject grade: 78 / 100 = 78% -> "B+" with GPA 3.5.
#[test]
fn result_service_compute_subject_marks_uses_full_mark() {
    let (grade, gpa) = ResultService::compute_subject_marks(78.0, 100.0);
    assert_eq!(grade.as_str(), "B+");
    assert_eq!(gpa.as_f32(), 3.5);
}

/// Per-subject grade: a zero full-mark denominator is
/// guarded against (returns "F" with GPA 0.0).
#[test]
fn result_service_compute_subject_marks_zero_full_mark_is_f() {
    let (grade, gpa) = ResultService::compute_subject_marks(0.0, 0.0);
    assert_eq!(grade.as_str(), "F");
    assert_eq!(gpa.as_f32(), 0.0);
}

/// Total marks + grade + GPA across all children.
#[test]
fn result_service_compute_total_aggregates_children() {
    // Three subjects: marks 80, 70, 60 over fulls 100 each
    // = total 210 / 300 = 70% -> "B+" with GPA 3.5.
    let children = [80.0_f32, 70.0, 60.0];
    let full_marks = [100.0_f32, 100.0, 100.0];
    let (total, grade, gpa) = ResultService::compute_total(&children, &full_marks);
    assert_eq!(total, 210.0);
    assert_eq!(grade.as_str(), "B+");
    assert_eq!(gpa.as_f32(), 3.5);
}

/// Pass/Fail rule: all subjects above their pass marks
/// -> `Pass`. One below -> `Fail`.
#[test]
fn result_service_determine_pass_fail_uses_per_subject_pass_marks() {
    let marks = [80.0_f32, 70.0, 60.0];
    let pass_marks = [35.0_f32, 35.0, 35.0];
    assert_eq!(
        ResultService::determine_pass_fail(&marks, &pass_marks),
        ResultStatus::Pass,
    );

    let marks_with_fail = [80.0_f32, 30.0, 60.0]; // 30 < 35
    assert_eq!(
        ResultService::determine_pass_fail(&marks_with_fail, &pass_marks),
        ResultStatus::Fail,
    );
}

/// Pass/Fail rule: mismatched array lengths -> `Fail`
/// (defensive: cannot evaluate per-subject pass without
/// aligned inputs).
#[test]
fn result_service_determine_pass_fail_mismatched_lengths_returns_fail() {
    let marks = [80.0_f32, 70.0];
    let pass_marks = [35.0_f32, 35.0, 35.0];
    assert_eq!(
        ResultService::determine_pass_fail(&marks, &pass_marks),
        ResultStatus::Fail,
    );
}

/// Merit-position invariant: tied students share a rank
/// and the next rank skips the tied count (per the spec's
/// "ties -> tied rank; positions skip the next integer").
#[test]
fn result_service_rank_section_assigns_tied_ranks_and_skips_integers() {
    // Totals: [90, 80, 80, 70] -> ranks [1, 2, 2, 4]
    // (positions skip the integer 3 because two students
    // tied at rank 2).
    let totals = [90.0_f32, 80.0, 80.0, 70.0];
    let ranks = ResultService::rank_section(&totals);
    assert_eq!(ranks, vec![1, 2, 2, 4]);
}

/// Merit-position invariant: three-way tie yields a single
/// rank followed by a triple-skip.
#[test]
fn result_service_rank_section_three_way_tie() {
    // Totals: [70, 70, 70, 50] -> ranks [1, 1, 1, 4]
    let totals = [70.0_f32, 70.0, 70.0, 50.0];
    let ranks = ResultService::rank_section(&totals);
    assert_eq!(ranks, vec![1, 1, 1, 4]);
}

/// Merit-position invariant: single student -> rank 1.
#[test]
fn result_service_rank_section_single_student() {
    let totals = [85.0_f32];
    let ranks = ResultService::rank_section(&totals);
    assert_eq!(ranks, vec![1]);
}

/// Cross-section ranker delegates to `rank_section`
/// (same algorithm; per the spec, cross-section ranks
/// use the same skip-on-tie invariant).
#[test]
fn result_service_rank_all_sections_delegates_to_rank_section() {
    let totals = [90.0_f32, 80.0, 80.0, 70.0];
    let section_ranks = ResultService::rank_section(&totals);
    let all_ranks = ResultService::rank_all_sections(&totals);
    assert_eq!(section_ranks, all_ranks);
}

// =============================================================================
// 7. School matches (cross-cutting helper)
// =============================================================================

/// The dispatcher (Phase 16) uses `school_matches` to
/// assert command -> aggregate school alignment. A
/// matching context returns `true`.
#[test]
fn school_matches_returns_true_for_same_school() {
    let (tenant, _g) = admin_context();
    let s = tenant.school_id;
    assert!(school_matches(&tenant, s));
}

/// A non-matching context returns `false`.
#[test]
fn school_matches_returns_false_for_different_school() {
    let (tenant, g) = admin_context();
    let other = g.next_school_id();
    assert!(!school_matches(&tenant, other));
}
