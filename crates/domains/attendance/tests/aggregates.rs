//! Integration tests for the **StudentAttendance aggregate**
//! vertical slice.
//!
//! Pins the create + update contract for
//! [`StudentAttendance`](educore_attendance::aggregate::StudentAttendance)
//! end-to-end through the service layer:
//!
//! 1. `mark_student_attendance` validates the input, asserts
//!    the per-day uniqueness invariant via the
//!    [`AttendanceUniquenessChecker`] port, constructs the
//!    aggregate, and emits a [`StudentAttendanceMarked`]
//!    event.
//! 2. `update_student_attendance` mutates the in-place
//!    aggregate (bumps `version`, updates `updated_at` /
//!    `updated_by`) and emits a [`StudentAttendanceUpdated`]
//!    event carrying the list of changed field names.
//!
//! The tests use the same fixture pattern as
//! `tests/workflows.rs` (in-memory `AttendanceUniquenessChecker`
//! + `TestClock` + `DeterministicIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber fan-out,
//! no outbox commit, no audit row). These tests pin the
//! contract of the **service layer** that the dispatcher will
//! eventually wrap.

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

use educore_attendance::prelude::*;
use educore_core::clock::{DeterministicIdGen, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Fixtures
// =============================================================================

/// In-memory uniqueness checker for the create-flow tests.
#[derive(Debug, Default)]
struct TestUniqueness {
    student_day: Mutex<HashSet<(SchoolId, StudentId, NaiveDate)>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self::default()
    }

    /// Pre-seed a (school, student, date) key to drive the
    /// uniqueness-conflict path on the next
    /// `mark_student_attendance` call.
    fn seed_student_day(&self, school: SchoolId, student: StudentId, date: NaiveDate) {
        self.student_day
            .lock()
            .expect("poisoned")
            .insert((school, student, date));
    }
}

impl AttendanceUniquenessChecker for TestUniqueness {
    fn student_day_exists(&self, school: SchoolId, student: StudentId, date: NaiveDate) -> bool {
        self.student_day
            .lock()
            .expect("poisoned")
            .contains(&(school, student, date))
    }
    fn subject_day_exists(
        &self,
        _school: SchoolId,
        _student: StudentId,
        _subject: SubjectId,
        _date: NaiveDate,
    ) -> bool {
        false
    }
    fn staff_day_exists(&self, _school: SchoolId, _staff: StaffId, _date: NaiveDate) -> bool {
        false
    }
    fn import_source_date_exists(
        &self,
        _school: SchoolId,
        _source: AttendanceSource,
        _date: NaiveDate,
    ) -> bool {
        false
    }
}

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

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn record_id(g: &SystemIdGen, school: SchoolId) -> StudentRecordId {
    StudentRecordId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn naive(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: create + update on StudentAttendance
// =============================================================================

/// End-to-end happy path for the StudentAttendance
/// aggregate. Mark a student present, then update the mark
/// to "late" with a note, asserting that:
///
/// 1. The create flow produces a `StudentAttendance`
///    aggregate carrying every field on the command + a
///    `StudentAttendanceMarked` event with the right
///    `event_type`, `aggregate_type`, and `school_id`.
/// 2. The update flow mutates the aggregate in place
///    (bumps `version`, swaps `attendance_type`, sets the
///    new `notes`), and emits a `StudentAttendanceUpdated`
///    event whose `changes` list names the fields that
///    actually moved.
#[test]
fn student_attendance_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let student = student_id(&g, school);
    let record = record_id(&g, school);
    let class = class_id(&g, school);
    let section = section_id(&g, school);
    let date = naive(2026, 9, 15);

    // ---- Create flow ----
    let create_cmd = MarkStudentAttendanceCommand {
        tenant: tenant.clone(),
        student_id: student,
        student_record_id: record,
        class_id: class,
        section_id: section,
        attendance_date: date,
        attendance_type: AttendanceType::Present,
        notes: None,
        notify: false,
        marked_from: AttendanceSource::Manual,
    };
    let (mut agg, marked_event) =
        mark_student_attendance(create_cmd, &clock, &ids, &uniqueness).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.student_id, student);
    assert_eq!(agg.student_record_id, record);
    assert_eq!(agg.class_id, class);
    assert_eq!(agg.section_id, section);
    assert_eq!(agg.attendance_date, date);
    assert_eq!(agg.attendance_type, AttendanceType::Present);
    assert!(agg.notes.is_none());
    assert!(!agg.is_absent());
    // Audit metadata footer is initialised.
    assert_eq!(agg.version.get(), 1);
    assert!(agg.is_active());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <StudentAttendanceMarked as DomainEvent>::EVENT_TYPE,
        "attendance.student.marked"
    );
    assert_eq!(
        <StudentAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
        "student_attendance"
    );
    assert_eq!(<StudentAttendanceMarked as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(marked_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(marked_event.school_id(), school);
    assert_eq!(marked_event.student_id, student);
    assert_eq!(marked_event.attendance_date, date);
    assert_eq!(marked_event.attendance_type, AttendanceType::Present);

    // ---- Update flow ----
    let initial_version = agg.version.get();
    let update_cmd = UpdateStudentAttendanceCommand {
        tenant: tenant.clone(),
        student_attendance_id: agg.id,
        attendance_type: Some(AttendanceType::Late),
        notes: Some("traffic delay".to_owned()),
        notify: Some(true),
    };
    let updated_event =
        update_student_attendance(&tenant, &mut agg, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(agg.attendance_type, AttendanceType::Late);
    assert_eq!(agg.notes.as_deref(), Some("traffic delay"));
    assert_eq!(agg.version.get(), initial_version + 1);
    assert_eq!(agg.updated_by, tenant.actor_id);

    // The event names the fields that actually moved.
    assert_eq!(
        <StudentAttendanceUpdated as DomainEvent>::EVENT_TYPE,
        "attendance.student.updated"
    );
    assert_eq!(
        <StudentAttendanceUpdated as DomainEvent>::AGGREGATE_TYPE,
        "student_attendance"
    );
    assert_eq!(updated_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert!(updated_event
        .changes
        .contains(&"attendance_type".to_owned()));
    assert!(updated_event.changes.contains(&"notes".to_owned()));
    // `notify` is not a field on the StudentAttendance
    // aggregate (only on SubjectAttendance); the service
    // silently ignores it. The contract is: only fields
    // that genuinely changed are listed in `changes`.
    assert!(!updated_event.changes.contains(&"notify".to_owned()));
}

// =============================================================================
// Validation failure: unique-key conflict on the create flow
// =============================================================================

/// Validation-failure path on the create flow: when the
/// per-(school, student, date) unique key is already taken,
/// `mark_student_attendance` returns `DomainError::Conflict`
/// and produces no aggregate / event.
///
/// The uniqueness invariant is enforced by the
/// `AttendanceUniquenessChecker` port; the service does not
/// mutate the uniqueness set itself (that's the storage
/// adapter's job after the aggregate is committed), so the
/// test pre-seeds the in-memory checker with the unique
/// key.
#[test]
fn student_attendance_create_rejects_duplicate_day_with_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let student = student_id(&g, school);
    let date = naive(2026, 9, 15);

    // Pre-seed the unique key so the create call returns
    // the spec's "already marked" Conflict error.
    uniqueness.seed_student_day(school, student, date);

    let cmd = MarkStudentAttendanceCommand {
        tenant,
        student_id: student,
        student_record_id: record_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        attendance_date: date,
        attendance_type: AttendanceType::Present,
        notes: None,
        notify: false,
        marked_from: AttendanceSource::Manual,
    };
    let err = mark_student_attendance(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate day mark must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

/// Validation-failure path on the create flow: when the
/// optional `notes` field exceeds 500 characters,
/// `mark_student_attendance` returns `DomainError::Validation`
/// before any aggregate is constructed or event minted.
///
/// The 500-char cap is enforced by
/// [`validate_notes`](educore_attendance::commands::validate_notes).
#[test]
fn student_attendance_create_rejects_oversized_notes_with_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let cmd = MarkStudentAttendanceCommand {
        tenant,
        student_id: student_id(&g, school),
        student_record_id: record_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        attendance_date: naive(2026, 9, 15),
        attendance_type: AttendanceType::Absent,
        notes: Some("x".repeat(501)),
        notify: false,
        marked_from: AttendanceSource::Manual,
    };
    let err = mark_student_attendance(cmd, &clock, &ids, &uniqueness)
        .expect_err("oversized notes must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Validation-failure path on the update flow: when no
/// fields on the `UpdateStudentAttendanceCommand` actually
/// change the aggregate, `update_student_attendance`
/// returns `DomainError::Validation` and the aggregate
/// stays at its original version.
#[test]
fn student_attendance_update_with_no_changes_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let create_cmd = MarkStudentAttendanceCommand {
        tenant: tenant.clone(),
        student_id: student_id(&g, school),
        student_record_id: record_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        attendance_date: naive(2026, 9, 15),
        attendance_type: AttendanceType::Present,
        notes: Some("baseline".to_owned()),
        notify: false,
        marked_from: AttendanceSource::Manual,
    };
    let (mut agg, _event) =
        mark_student_attendance(create_cmd, &clock, &ids, &uniqueness).expect("create");
    let initial_version = agg.version.get();

    // Update with `None` for every field — guaranteed not
    // to change the aggregate.
    let noop_update = UpdateStudentAttendanceCommand {
        tenant: tenant.clone(),
        student_attendance_id: agg.id,
        attendance_type: None,
        notes: None,
        notify: None,
    };
    let err = update_student_attendance(&tenant, &mut agg, noop_update, &clock, &ids)
        .expect_err("noop update must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // The aggregate's version must be unchanged: a failed
    // update must not bump the optimistic-concurrency
    // counter.
    assert_eq!(agg.version.get(), initial_version);
}
