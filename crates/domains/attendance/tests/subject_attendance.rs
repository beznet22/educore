//! Integration tests for the **SubjectAttendance aggregate**
//! vertical slice (Wave 4).
//!
//! Pins the create contract for
//! [`SubjectAttendance`](educore_attendance::aggregate::SubjectAttendance)
//! end-to-end through the service layer:
//!
//! `mark_subject_attendance` validates the input, asserts
//! the per-(school, student, subject, date) uniqueness
//! invariant via the [`AttendanceUniquenessChecker`] port,
//! constructs the aggregate, and emits a
//! [`SubjectAttendanceMarked`] event.
//!
//! The test follows the same fixture pattern as
//! `crates/domains/attendance/tests/aggregates.rs` (Wave 1,
//! `StudentAttendance`): an in-memory
//! `AttendanceUniquenessChecker`, a `TestClock`, and a
//! `SystemIdGen` / `DeterministicIdGen`.
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **service layer** that the dispatcher
//! will eventually wrap. When the handler lands, the same
//! test bodies will gain a `+ outbox + bus subscriber`
//! assertion without changes to the assertions on the
//! returned event.
//!
//! Mirrors `crates/domains/attendance/tests/aggregates.rs`
//! (lean: happy path + one validation failure).
//!
//! Spec: `docs/specs/attendance/aggregates.md` § SubjectAttendance.

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
use educore_core::ids::{CorrelationId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Fixtures
// =============================================================================

/// In-memory uniqueness checker that always reports "no
/// conflict". The SubjectAttendance create flow only checks
/// `subject_day_exists`, so the other three methods are
/// `false`-returning stubs that mirror the `AttendanceUniquenessChecker`
/// port's full surface.
#[derive(Debug, Default)]
struct TestUniqueness;

impl AttendanceUniquenessChecker for TestUniqueness {
    fn student_day_exists(
        &self,
        _school: SchoolId,
        _student: StudentId,
        _date: NaiveDate,
    ) -> bool {
        false
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
    fn staff_day_exists(
        &self,
        _school: SchoolId,
        _staff: StaffId,
        _date: NaiveDate,
    ) -> bool {
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

/// Silence the "unused" warning on `HashSet` / `Mutex` /
/// `StaffId` if the fixture's `subject_day_exists` ever
/// grows a seedable surface (it currently doesn't, but
/// keeps the imports stable against future extension).
#[allow(dead_code)]
fn _unused_imports_anchors() {
    let _h: HashSet<(SchoolId, StudentId, SubjectId, NaiveDate)> = HashSet::new();
    let _m: Option<Mutex<(SchoolId, StudentId, SubjectId, NaiveDate)>> = None;
    let _s: Option<StaffId> = None;
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

fn subject_id(g: &SystemIdGen, school: SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

fn naive(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: create on SubjectAttendance
// =============================================================================

/// End-to-end happy path for the SubjectAttendance
/// aggregate. Mark a student present in a specific subject
/// for a specific period, asserting that:
///
/// 1. The create flow produces a `SubjectAttendance`
///    aggregate carrying every field on the command +
///    audit metadata (version=1, `active_status`=Active,
///    `created_by`/`updated_by` set to the actor).
/// 2. The aggregate's `notify` flag is plumbed through
///    from the command (it's a real field on
///    `SubjectAttendance`, unlike on `StudentAttendance`).
/// 3. The emitted `SubjectAttendanceMarked` event matches
///    the aggregate's typed id, has the spec-mandated
///    `EVENT_TYPE` / `AGGREGATE_TYPE` / `SCHEMA_VERSION`,
///    and carries every command field.
#[test]
fn subject_attendance_create_emits_marked_event_v1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness;

    let student = student_id(&g, school);
    let record = record_id(&g, school);
    let class = class_id(&g, school);
    let section = section_id(&g, school);
    let subject = subject_id(&g, school);
    let date = naive(2026, 9, 15);

    // ---- Create flow ----
    let create_cmd = MarkSubjectAttendanceCommand {
        tenant: tenant.clone(),
        student_id: student,
        student_record_id: record,
        class_id: class,
        section_id: section,
        subject_id: subject,
        attendance_date: date,
        attendance_type: AttendanceType::Present,
        notes: Some("assembled in homeroom".to_owned()),
        notify: true,
        marked_from: AttendanceSource::Manual,
    };
    let (agg, marked_event) =
        mark_subject_attendance(create_cmd, &clock, &ids, &uniqueness).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.student_id, student);
    assert_eq!(agg.student_record_id, record);
    assert_eq!(agg.class_id, class);
    assert_eq!(agg.section_id, section);
    assert_eq!(agg.subject_id, subject);
    assert_eq!(agg.attendance_date, date);
    assert_eq!(agg.attendance_type, AttendanceType::Present);
    assert_eq!(agg.notes.as_deref(), Some("assembled in homeroom"));
    assert!(agg.notify, "notify must be plumbed through the command");
    assert!(!agg.is_absent());
    // Audit metadata footer is initialised.
    assert_eq!(agg.version.get(), 1);
    assert!(agg.is_active());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's
    // contract for `SubjectAttendanceMarked`.
    assert_eq!(
        <SubjectAttendanceMarked as DomainEvent>::EVENT_TYPE,
        "attendance.subject.marked"
    );
    assert_eq!(
        <SubjectAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
        "subject_attendance"
    );
    assert_eq!(
        <SubjectAttendanceMarked as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(marked_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(marked_event.school_id(), school);
    assert_eq!(marked_event.subject_attendance_id, agg.id);
    assert_eq!(marked_event.student_id, student);
    assert_eq!(marked_event.student_record_id, record);
    assert_eq!(marked_event.class_id, class);
    assert_eq!(marked_event.section_id, section);
    assert_eq!(marked_event.subject_id, subject);
    assert_eq!(marked_event.attendance_date, date);
    assert_eq!(marked_event.attendance_type, AttendanceType::Present);
    assert_eq!(marked_event.notes.as_deref(), Some("assembled in homeroom"));
    assert!(marked_event.notify);
    assert_eq!(marked_event.marked_by, tenant.actor_id);
    assert_eq!(marked_event.marked_from, AttendanceSource::Manual);
}

// =============================================================================
// Validation failure: oversized notes
// =============================================================================

/// Validation-failure path on the create flow: when the
/// optional `notes` field exceeds 500 characters,
/// `mark_subject_attendance` returns `DomainError::Validation`
/// before any aggregate is constructed or event minted.
///
/// The 500-char cap is enforced by
/// [`validate_notes`](educore_attendance::commands::validate_notes)
/// and is the only input-driven `DomainError::Validation`
/// path reachable through the public typed command surface
/// (the typed `SubjectId` / `AttendanceType` /
/// `AttendanceSource` / `NaiveDate` fields cannot be
/// constructed in an "invalid" state).
///
/// No aggregate / event is produced.
#[test]
fn subject_attendance_create_rejects_oversized_notes_with_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness;

    let cmd = MarkSubjectAttendanceCommand {
        tenant,
        student_id: student_id(&g, school),
        student_record_id: record_id(&g, school),
        class_id: class_id(&g, school),
        section_id: section_id(&g, school),
        subject_id: subject_id(&g, school),
        attendance_date: naive(2026, 9, 15),
        attendance_type: AttendanceType::Late,
        notes: Some("x".repeat(501)),
        notify: false,
        marked_from: AttendanceSource::Manual,
    };
    let err = mark_subject_attendance(cmd, &clock, &ids, &uniqueness)
        .expect_err("oversized notes must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
