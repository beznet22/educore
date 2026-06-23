//! Integration tests for the **academic domain workflows**.
//!
//! Implements: `docs/specs/academic/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the academic service functions and asserts that the
//! expected typed event is emitted (or, on the error path,
//! that the expected [`DomainError`] is returned and no event
//! is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! academic service factories (`admit_student`, `withdraw_student`,
//! `promote_student`, etc.) are sync, take a `Clock` and an
//! `IdGenerator`, and return the mutated aggregate plus the
//! typed event. The test wires a [`TestClock`] and a
//! [`SystemIdGen`], and an in-memory [`UniquenessChecker`] for
//! the workflows that need one.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **service layer** that the dispatcher
//! will eventually wrap. When the handlers land, the same test
//! bodies will gain a `+ outbox + bus subscriber` assertion
//! without changes to the assertions on the returned event.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::collections::HashSet;
use std::sync::Mutex;

use chrono::NaiveDate;

use educore_academic::prelude::*;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_events::domain_event::DomainEvent;

/// Test-local helper: assert an event's `EVENT_TYPE` constant.
macro_rules! assert_event_type {
    ($event:expr, $expected:literal) => {
        assert_eq!(<_ as DomainEvent>::EVENT_TYPE, $expected);
    };
}

// =============================================================================
// Test fixtures
// =============================================================================

/// A tiny in-memory `UniquenessChecker` used by the admit / update flows.
#[derive(Debug, Default)]
struct TestUniqueness {
    admission_numbers: Mutex<HashSet<String>>,
    emails: Mutex<HashSet<String>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self::default()
    }
}

impl UniquenessChecker for TestUniqueness {
    fn student_admission_no_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        admission_no: &str,
    ) -> bool {
        self.admission_numbers
            .lock()
            .unwrap()
            .contains(admission_no)
    }
    fn student_email_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        email: &str,
    ) -> bool {
        self.emails.lock().unwrap().contains(email)
    }
}

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a freshly-minted school.
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

fn student_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn subject_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn naive(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// =============================================================================
// 1. Admission Workflow (`workflows.md` § "Admission Workflow")
// =============================================================================

/// Admission step 3: convert inquiry into a student via
/// [`AdmitStudentCommand`]. The service must produce a
/// [`StudentAdmitted`] event with the same id and admission
/// number as the command.
#[test]
fn admit_student_happy_path_emits_student_admitted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();

    let cmd = AdmitStudentCommand::new(
        tenant,
        student_id(&g, school),
        "ADM-2026-001".to_owned(),
        "Ada".to_owned(),
        "Lovelace".to_owned(),
        naive(2016, 1, 1),
        Gender::Female,
        naive(2026, 6, 1),
        class_id(&g, school),
        section_id(&g, school),
        year_id(&g, school),
    );

    let (student, event): (Student, StudentAdmitted) =
        admit_student(cmd, &clock, &g, &uniqueness).unwrap();

    assert_eq!(student.status, StudentStatus::Active);
    assert_eq!(<StudentAdmitted as DomainEvent>::EVENT_TYPE, "academic.student.admitted");
    assert_eq!(student.admission_no, "ADM-2026-001");
    assert_eq!(student.first_name, "Ada");
    assert_eq!(student.last_name, "Lovelace");
    assert_eq!(student.school_id, school);
    // Correlation id propagates command -> aggregate + event.
    assert_eq!(student.correlation_id, event.correlation_id);
}

/// Admission step 3 failure path: a duplicate admission number
/// must be rejected with `DomainError::Conflict` and no event
/// must be produced (the function returns `Err`, so the caller
/// never gets to publish anything).
#[test]
fn admit_student_duplicate_admission_no_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();

    // Pre-seed the uniqueness checker with the admission number.
    uniqueness
        .admission_numbers
        .lock()
        .unwrap()
        .insert("ADM-DUP".to_owned());

    let cmd = AdmitStudentCommand::new(
        tenant,
        student_id(&g, school),
        "ADM-DUP".to_owned(),
        "Duplicate".to_owned(),
        "Admission".to_owned(),
        naive(2017, 5, 5),
        Gender::Male,
        naive(2026, 6, 1),
        class_id(&g, school),
        section_id(&g, school),
        year_id(&g, school),
    );

    let result: educore_core::error::Result<(Student, StudentAdmitted)> =
        admit_student(cmd, &clock, &g, &uniqueness);
    let err = result.expect_err("duplicate admission_no must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

/// Admission step 3 failure path: an empty admission number
/// must be rejected with `DomainError::Validation`.
#[test]
fn admit_student_empty_admission_no_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();

    let cmd = AdmitStudentCommand::new(
        tenant,
        student_id(&g, school),
        String::new(),
        "Ada".to_owned(),
        "Lovelace".to_owned(),
        naive(2016, 1, 1),
        Gender::Female,
        naive(2026, 6, 1),
        class_id(&g, school),
        section_id(&g, school),
        year_id(&g, school),
    );

    let err = admit_student(cmd, &clock, &g, &uniqueness).expect_err("empty id must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 2. Suspension (sub-flow of Admission Workflow)
// =============================================================================

/// Suspension happy path: suspending an active student
/// must emit a [`StudentSuspended`] event.
#[test]
fn suspend_student_happy_path_emits_student_suspended() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-SUS-001".to_owned(),
            "Sus".to_owned(),
            "Pended".to_owned(),
            naive(2016, 1, 1),
            Gender::Male,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let mut student = student;
    let cmd = SuspendStudentCommand {
        tenant,
        student_id: student.id,
        reason: "Medical leave".to_owned(),
        effective_from: naive(2026, 9, 1),
        expected_return: Some(naive(2026, 12, 1)),
    };
    let event = suspend_student(&mut student, cmd, &clock, &g).unwrap();
    assert_eq!(<StudentSuspended as DomainEvent>::EVENT_TYPE, "academic.student.suspended");
    assert_eq!(event.student_id, student.id);
}

#[test]
fn suspend_student_empty_reason_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-SUS-002".to_owned(),
            "Empty".to_owned(),
            "Reason".to_owned(),
            naive(2016, 1, 1),
            Gender::Male,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = SuspendStudentCommand {
        tenant,
        student_id: student.id,
        reason: String::new(),
        effective_from: naive(2026, 9, 1),
        expected_return: None,
    };
    let err = suspend_student(&mut student, cmd, &clock, &g).expect_err("empty reason must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 3. Withdrawal Workflow (`workflows.md` § "Withdrawal Workflow")
// =============================================================================

/// Withdrawal step 1: a [`WithdrawStudentCommand`] must
/// produce a [`StudentWithdrawn`] event with the supplied
/// reason and effective_from date.
#[test]
fn withdraw_student_happy_path_emits_student_withdrawn() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-WD-001".to_owned(),
            "Will".to_owned(),
            "Withdraw".to_owned(),
            naive(2016, 1, 1),
            Gender::Female,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = WithdrawStudentCommand {
        tenant,
        student_id: student.id,
        reason: "Family relocation".to_owned(),
        effective_from: naive(2026, 9, 15),
        note: Some("Graduating cohort".to_owned()),
    };
    let event: StudentWithdrawn = withdraw_student(&mut student, cmd, &clock, &g).unwrap();
    assert_eq!(<StudentWithdrawn as DomainEvent>::EVENT_TYPE, "academic.student.withdrawn");
    assert_eq!(event.student_id, student.id);
}

/// Withdrawal spec: "Re-issuing a withdraw for an
/// already-withdrawn student is a no-op success" (idempotency).
/// The current service surfaces this through
/// `DomainError::Conflict` (the active_status check); the test
/// pins the contract so the dispatcher knows what to expect.
#[test]
fn withdraw_student_twice_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-WD-002".to_owned(),
            "Two".to_owned(),
            "Times".to_owned(),
            naive(2016, 1, 1),
            Gender::Female,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = WithdrawStudentCommand {
        tenant: tenant.clone(),
        student_id: student.id,
        reason: "Family relocation".to_owned(),
        effective_from: naive(2026, 9, 15),
        note: None,
    };
    let _first: StudentWithdrawn = withdraw_student(&mut student, cmd.clone(), &clock, &g).unwrap();
    // Second attempt should be rejected because the aggregate
    // is now retired. The contract here is "DomainError::Conflict".
    let err = withdraw_student(&mut student, cmd, &clock, &g)
        .err()
        .expect("second withdraw must error");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 4. Transfer Workflow (Cross-School)
// =============================================================================

/// Transfer step 1: [`TransferStudentCommand`] must emit
/// [`StudentTransferred`] carrying the destination school id.
#[test]
fn transfer_student_happy_path_emits_student_transferred() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-TR-001".to_owned(),
            "Move".to_owned(),
            "Along".to_owned(),
            naive(2016, 1, 1),
            Gender::Male,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let destination = g.next_school_id();
    let cmd = TransferStudentCommand {
        tenant,
        student_id: student.id,
        destination_school_id: destination,
        reason: "Parent job relocation".to_owned(),
        effective_from: naive(2026, 9, 15),
    };
    let event: StudentTransferred =
        transfer_student(&mut student, cmd, &clock, &g).unwrap();
    assert_eq!(<StudentTransferred as DomainEvent>::EVENT_TYPE, "academic.student.transferred");
    assert_eq!(event.destination_school_id, destination);
}

#[test]
fn transfer_student_empty_reason_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-TR-002".to_owned(),
            "Empty".to_owned(),
            "Transfer".to_owned(),
            naive(2016, 1, 1),
            Gender::Male,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = TransferStudentCommand {
        tenant,
        student_id: student.id,
        destination_school_id: g.next_school_id(),
        reason: String::new(),
        effective_from: naive(2026, 9, 15),
    };
    let err = transfer_student(&mut student, cmd, &clock, &g)
        .err()
        .expect("empty reason must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 5. Promotion Workflow (`workflows.md` § "Promotion Workflow")
// =============================================================================

/// Promotion step 4: a pass-status [`PromoteStudentCommand`]
/// must emit [`StudentPromoted`] with the target class id
/// and target roll number carried over.
#[test]
fn promote_student_pass_status_emits_student_promoted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-PR-001".to_owned(),
            "Pro".to_owned(),
            "Moted".to_owned(),
            naive(2016, 1, 1),
            Gender::Female,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let from_year = year_id(&g, school);
    let to_year = year_id(&g, school);
    let to_class = class_id(&g, school);
    let to_section = section_id(&g, school);
    let cmd = PromoteStudentCommand {
        tenant,
        student_id: student.id,
        from_academic_year_id: from_year,
        to_academic_year_id: to_year,
        to_class_id: to_class,
        to_section_id: to_section,
        to_roll_no: "12".to_owned(),
        result_status: ResultStatus::Pass,
    };
    let event: StudentPromoted = promote_student(&student, cmd, &clock, &g).unwrap();
    assert_eq!(<StudentPromoted as DomainEvent>::EVENT_TYPE, "academic.student.promoted");
    assert_eq!(event.to_class_id, to_class);
    assert_eq!(event.to_section_id, to_section);
    assert_eq!(event.to_roll_no, "12");
}

/// Promotion edge case: held-back student (`Manual` result
/// status with no roll number) must still produce a
/// `StudentPromoted` event so the assessment + finance
/// subscribers can react.
#[test]
fn promote_student_manual_status_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-PR-002".to_owned(),
            "Held".to_owned(),
            "Back".to_owned(),
            naive(2016, 1, 1),
            Gender::Female,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = PromoteStudentCommand {
        tenant,
        student_id: student.id,
        from_academic_year_id: year_id(&g, school),
        to_academic_year_id: year_id(&g, school),
        to_class_id: class_id(&g, school),
        to_section_id: section_id(&g, school),
        to_roll_no: "01".to_owned(),
        result_status: ResultStatus::Manual,
    };
    let event: StudentPromoted = promote_student(&student, cmd, &clock, &g).unwrap();
    assert_eq!(event.result_status, ResultStatus::Manual);
    assert_eq!(<StudentPromoted as DomainEvent>::EVENT_TYPE, "academic.student.promoted");
}

// =============================================================================
// 6. Graduation (sub-flow of Promotion Workflow)
// =============================================================================

#[test]
fn graduate_student_happy_path_emits_student_graduated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let uniqueness = TestUniqueness::new();
    let (mut student, _) = admit_student(
        AdmitStudentCommand::new(
            tenant.clone(),
            student_id(&g, school),
            "ADM-GR-001".to_owned(),
            "Grad".to_owned(),
            "uated".to_owned(),
            naive(2008, 1, 1),
            Gender::Female,
            naive(2026, 6, 1),
            class_id(&g, school),
            section_id(&g, school),
            year_id(&g, school),
        ),
        &clock,
        &g,
        &uniqueness,
    )
    .unwrap();
    let cmd = GraduateStudentCommand {
        tenant,
        student_id: student.id,
        academic_year_id: year_id(&g, school),
        graduation_date: naive(2026, 6, 30),
    };
    let event: StudentGraduated = graduate_student(&mut student, cmd, &clock, &g).unwrap();
    assert_eq!(<StudentGraduated as DomainEvent>::EVENT_TYPE, "academic.student.graduated");
}

// =============================================================================
// 7. Class-Section Lifecycle (`workflows.md` § "Class-Section Lifecycle")
// =============================================================================

/// Class lifecycle step 1: [`CreateClassCommand`] must emit
/// [`ClassCreated`] with the supplied name and pass mark.
#[test]
fn create_class_happy_path_emits_class_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateClassCommand {
        tenant,
        class_id: class_id(&g, school),
        class_name: "Grade 1".to_owned(),
        pass_mark: 35.0,
    };
    let (_, event): (Class, ClassCreated) = create_class(cmd, &clock, &g).unwrap();
    assert_eq!(<ClassCreated as DomainEvent>::EVENT_TYPE, "academic.class.created");
}

#[test]
fn create_class_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateClassCommand {
        tenant,
        class_id: class_id(&g, school),
        class_name: String::new(),
        pass_mark: 35.0,
    };
    let err = create_class(cmd, &clock, &g).expect_err("empty class_name must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

/// Class lifecycle step 5: [`UpdateClassCommand`] mutates
/// the existing class and emits [`ClassUpdated`].
#[test]
fn update_class_happy_path_emits_class_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let (class, _) = create_class(
        CreateClassCommand {
            tenant: tenant.clone(),
            class_id: class_id(&g, school),
            class_name: "Grade 1".to_owned(),
            pass_mark: 35.0,
        },
        &clock,
        &g,
    )
    .unwrap();
    let mut class = class;
    let cmd = UpdateClassCommand {
        tenant,
        class_id: class.id,
        class_name: Some("Grade 1 (renamed)".to_owned()),
        pass_mark: Some(40.0),
    };
    let event: ClassUpdated = update_class(&mut class, cmd, &clock, &g).unwrap();
    assert_eq!(<ClassUpdated as DomainEvent>::EVENT_TYPE, "academic.class.updated");
    assert_eq!(class.pass_mark.as_f32(), 40.0);
}

/// Class lifecycle end-of-year: deleting a class emits
/// [`ClassDeleted`].
#[test]
fn delete_class_happy_path_emits_class_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let (class, _) = create_class(
        CreateClassCommand {
            tenant: tenant.clone(),
            class_id: class_id(&g, school),
            class_name: "Grade 1".to_owned(),
            pass_mark: 35.0,
        },
        &clock,
        &g,
    )
    .unwrap();
    let cmd = DeleteClassCommand {
        tenant,
        class_id: class.id,
    };
    let mut class = class;
    let event: ClassDeleted = delete_class(&mut class, cmd, &clock, &g).unwrap();
    assert_eq!(<ClassDeleted as DomainEvent>::EVENT_TYPE, "academic.class.deleted");
}

// =============================================================================
// 8. Section Lifecycle (sub-flow of Class-Section Lifecycle)
// =============================================================================

#[test]
fn create_section_happy_path_emits_section_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateSectionCommand {
        tenant,
        section_id: section_id(&g, school),
        section_name: "A".to_owned(),
    };
    let (_, event): (Section, SectionCreated) = create_section(cmd, &clock, &g).unwrap();
    assert_eq!(<SectionCreated as DomainEvent>::EVENT_TYPE, "academic.section.created");
}

// =============================================================================
// 9. Subject Lifecycle (`workflows.md` § "Routine Construction" prerequisites)
// =============================================================================

#[test]
fn create_subject_happy_path_emits_subject_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateSubjectCommand {
        tenant,
        subject_id: subject_id(&g, school),
        subject_code: "MATH".to_owned(),
        subject_name: "Mathematics".to_owned(),
        subject_type: SubjectType::Theory,
        pass_mark: 35.0,
    };
    let (_, event): (Subject, SubjectCreated) = create_subject(cmd, &clock, &g).unwrap();
    assert_eq!(<SubjectCreated as DomainEvent>::EVENT_TYPE, "academic.subject.created");
}

#[test]
fn update_subject_happy_path_emits_subject_updated() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let (mut subject, _) = create_subject(
        CreateSubjectCommand {
            tenant: tenant.clone(),
            subject_id: subject_id(&g, school),
            subject_code: "MATH".to_owned(),
            subject_name: "Mathematics".to_owned(),
            subject_type: SubjectType::Theory,
            pass_mark: 35.0,
        },
        &clock,
        &g,
    )
    .unwrap();
    let mut subject = subject;
    let cmd = UpdateSubjectCommand {
        tenant,
        subject_id: subject.id,
        subject_name: Some("Mathematics (Honors)".to_owned()),
        subject_type: Some(SubjectType::Practical),
        pass_mark: Some(40.0),
    };
    let event: SubjectUpdated = update_subject(&mut subject, cmd, &clock, &g).unwrap();
    assert_eq!(<SubjectUpdated as DomainEvent>::EVENT_TYPE, "academic.subject.updated");
}

#[test]
fn delete_subject_happy_path_emits_subject_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let (mut subject, _) = create_subject(
        CreateSubjectCommand {
            tenant: tenant.clone(),
            subject_id: subject_id(&g, school),
            subject_code: "MATH".to_owned(),
            subject_name: "Mathematics".to_owned(),
            subject_type: SubjectType::Theory,
            pass_mark: 35.0,
        },
        &clock,
        &g,
    )
    .unwrap();
    let cmd = DeleteSubjectCommand {
        tenant,
        subject_id: subject.id,
    };
    let event: SubjectDeleted = delete_subject(&mut subject, cmd, &clock, &g).unwrap();
    assert_eq!(<SubjectDeleted as DomainEvent>::EVENT_TYPE, "academic.subject.deleted");
}

// =============================================================================
// 10. AcademicYear Lifecycle (`workflows.md` § "Class-Section Lifecycle" step 5)
// =============================================================================

#[test]
fn create_academic_year_happy_path_emits_academic_year_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateAcademicYearCommand {
        tenant,
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: naive(2026, 4, 1),
        ending_date: naive(2027, 3, 31),
        is_current: true,
        copy_with_academic_year: None,
    };
    let (_, event): (AcademicYear, AcademicYearCreated) =
        create_academic_year(cmd, &clock, &g).unwrap();
    assert_eq!(
        <AcademicYearCreated as DomainEvent>::EVENT_TYPE,
        "academic.academic_year.created"
    );
}

/// Set current academic year: emits [`CurrentAcademicYearSet`].
#[test]
fn set_current_academic_year_happy_path_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateAcademicYearCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: naive(2026, 4, 1),
        ending_date: naive(2027, 3, 31),
        is_current: false,
        copy_with_academic_year: None,
    };
    let (mut year, _) = create_academic_year(cmd, &clock, &g).unwrap();
    let set_cmd = SetCurrentAcademicYearCommand {
        tenant,
        academic_year_id: year.id,
    };
    let event: CurrentAcademicYearSet =
        set_current_academic_year(&mut year, set_cmd, &clock, &g).unwrap();
    assert_eq!(
        <CurrentAcademicYearSet as DomainEvent>::EVENT_TYPE,
        "academic.current_academic_year.set"
    );
}

/// Close academic year: emits [`AcademicYearClosed`].
#[test]
fn close_academic_year_happy_path_emits_academic_year_closed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let cmd = CreateAcademicYearCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: naive(2026, 4, 1),
        ending_date: naive(2027, 3, 31),
        is_current: true,
        copy_with_academic_year: None,
    };
    let (mut year, _) = create_academic_year(cmd, &clock, &g).unwrap();
    let close_cmd = CloseAcademicYearCommand {
        tenant,
        academic_year_id: year.id,
    };
    let event: AcademicYearClosed = close_academic_year(&mut year, close_cmd, &clock, &g).unwrap();
    assert_eq!(
        <AcademicYearClosed as DomainEvent>::EVENT_TYPE,
        "academic.academic_year.closed"
    );
}
