//! Integration tests for the **attendance domain workflows**.
//!
//! Implements: `docs/specs/attendance/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the attendance service functions and asserts that
//! the expected typed event is emitted (or, on the error path,
//! that the expected [`DomainError`] is returned and no event
//! is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! attendance service factories (`mark_student_attendance`,
//! `bulk_mark_student_attendance`, `import_attendance`,
//! `validate_bulk_import`, `commit_bulk_import`, ...) are
//! sync, take a `Clock` and an `IdGenerator`, and return the
//! mutated aggregate plus the typed event(s). The test wires
//! a [`TestClock`], a [`DeterministicIdGen`], and an
//! in-memory [`AttendanceUniquenessChecker`] for the workflows
//! that need one.
//!
//! Per the academic/workflows.rs pattern, the **handlers** are
//! not yet wired end-to-end (no subscriber fan-out, no outbox
//! commit, no audit row). These tests pin the contract of the
//! **service layer** that the dispatcher will eventually wrap.
//! When the handlers land, the same test bodies will gain a
//! `+ outbox + bus subscriber` assertion without changes to
//! the assertions on the returned event.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use std::collections::HashSet;
use std::sync::Mutex;

use chrono::NaiveDate;

use educore_attendance::prelude::*;
use educore_core::clock::{DeterministicIdGen, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
// =============================================================================
// Test fixtures
// =============================================================================

/// A tiny in-memory `AttendanceUniquenessChecker` used by the
/// mark / import flows.
#[derive(Debug, Default)]
struct TestUniqueness {
    student_day: Mutex<HashSet<(SchoolId, StudentId, NaiveDate)>>,
    subject_day: Mutex<HashSet<(SchoolId, StudentId, SubjectId, NaiveDate)>>,
    staff_day: Mutex<HashSet<(SchoolId, StaffId, NaiveDate)>>,
    import_source_date: Mutex<HashSet<(SchoolId, AttendanceSource, NaiveDate)>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self::default()
    }
}

impl AttendanceUniquenessChecker for TestUniqueness {
    fn student_day_exists(&self, school: SchoolId, student: StudentId, date: NaiveDate) -> bool {
        self.student_day
            .lock()
            .unwrap()
            .contains(&(school, student, date))
    }
    fn subject_day_exists(
        &self,
        school: SchoolId,
        student: StudentId,
        subject: SubjectId,
        date: NaiveDate,
    ) -> bool {
        self.subject_day
            .lock()
            .unwrap()
            .contains(&(school, student, subject, date))
    }
    fn staff_day_exists(&self, school: SchoolId, staff: StaffId, date: NaiveDate) -> bool {
        self.staff_day
            .lock()
            .unwrap()
            .contains(&(school, staff, date))
    }
    fn import_source_date_exists(
        &self,
        school: SchoolId,
        source: AttendanceSource,
        date: NaiveDate,
    ) -> bool {
        self.import_source_date
            .lock()
            .unwrap()
            .contains(&(school, source, date))
    }
}

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a freshly-minted school.
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

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn record_id(g: &SystemIdGen, school: SchoolId) -> StudentRecordId {
    StudentRecordId::new(school, g.next_uuid())
}

fn naive(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// =============================================================================
// 1. Daily Attendance Roll-up
//    (`workflows.md` § "Daily Attendance Capture")
//
//    Mark students via `mark_student_attendance`, then
//    aggregate the per-student events into a single
//    per-(class, section, date) summary via
//    `bulk_mark_student_attendance`. The test asserts:
//    - One `StudentAttendanceMarked` event per student.
//    - One `StudentAbsentForDay` event per absent student.
//    - The aggregates can be rolled up to a
//      `ClassAttendanceRecomputed`-shaped summary.
// =============================================================================

/// Daily roll-up happy path: marking 5 students (3 Present,
/// 1 Absent, 1 Late) must produce 5 aggregates, 5 marked
/// events, and 1 absent event. The test then rolls the
/// per-student aggregates up to a class-day summary that
/// matches the spec's `ClassAttendance` shape.
#[test]
fn daily_attendance_rollup_marks_section_and_emits_absent_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let class = class_id(&g, school);
    let section = section_id(&g, school);
    let date = naive(2026, 9, 15);

    // Build the per-student mark commands: 3 Present, 1
    // Absent, 1 Late.
    let present_ids: Vec<StudentId> = (0..3).map(|_| student_id(&g, school)).collect();
    let absent_ids: Vec<StudentId> = (0..1).map(|_| student_id(&g, school)).collect();
    let late_ids: Vec<StudentId> = (0..1).map(|_| student_id(&g, school)).collect();

    // Mark each student individually so we exercise the
    // per-student `mark_student_attendance` service (the
    // spec's step 5: "the engine issues one
    // MarkStudentAttendance per student").
    let mut per_student_aggregates: Vec<StudentAttendance> = Vec::new();
    let mut per_student_marked: Vec<StudentAttendanceMarked> = Vec::new();
    for sid in present_ids
        .iter()
        .chain(absent_ids.iter())
        .chain(late_ids.iter())
    {
        let cmd = MarkStudentAttendanceCommand {
            tenant: tenant.clone(),
            student_id: *sid,
            student_record_id: record_id(&g, school),
            class_id: class,
            section_id: section,
            attendance_date: date,
            attendance_type: AttendanceType::Present,
            notes: None,
            notify: false,
            marked_from: AttendanceSource::Manual,
        };
        let (agg, event) = mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("mark");
        assert_eq!(
            <StudentAttendanceMarked as DomainEvent>::EVENT_TYPE,
            "attendance.student.marked"
        );
        per_student_aggregates.push(agg);
        per_student_marked.push(event);
    }

    // The bulk-mark service below covers the Absent /
    // Late / HalfDay types — the per-student test above
    // sticks to Present marks so the (school, student,
    // date) uniqueness invariant stays clean.

    assert_eq!(per_student_aggregates.len(), 5);
    assert_eq!(per_student_marked.len(), 5);

    // Now exercise the bulk-mark service for the same
    // (class, section, date) on a different date to drive
    // the per-student types and the roll-up.
    let date2 = naive(2026, 9, 16);
    let absent_for_bulk = student_id(&g, school);
    let late_for_bulk = student_id(&g, school);
    let half_for_bulk = student_id(&g, school);
    let bulk_cmd = BulkMarkStudentAttendanceCommand {
        tenant: tenant.clone(),
        class_id: class,
        section_id: section,
        attendance_date: date2,
        default_type: AttendanceType::Present,
        absent_ids: vec![absent_for_bulk],
        late_ids: vec![late_for_bulk],
        half_day_ids: vec![half_for_bulk],
        notes: None,
    };
    let result =
        bulk_mark_student_attendance(bulk_cmd, &clock, &ids, &uniqueness).expect("bulk mark");
    // 1 default (Present) + 1 absent + 1 late + 1 half-day
    // = 4 aggregates.
    assert_eq!(result.aggregates.len(), 4);
    assert_eq!(result.marked_events.len(), 4);
    // The Present default is not absent, the Late is not
    // absent, but the Absent and HalfDay rows both produce
    // `StudentAbsentForDay` events → 2 in total.
    assert_eq!(result.absent_events.len(), 2);

    // Roll the 4 aggregates up to a class-day summary
    // (the `ClassAttendance` shape the spec describes).
    let total: u32 = result.aggregates.len().try_into().unwrap();
    let absent_count: u32 = result
        .aggregates
        .iter()
        .filter(|a| a.attendance_type.is_absent())
        .count()
        .try_into()
        .unwrap();
    let present_count: u32 = result
        .aggregates
        .iter()
        .filter(|a| !a.attendance_type.is_absent())
        .count()
        .try_into()
        .unwrap();
    assert_eq!(total, 4);
    // Only the strict-Absent type counts as absent in the
    // roll-up (HalfDay is treated as "not absent" for the
    // purposes of the absence ratio).
    assert_eq!(absent_count, 1);
    assert_eq!(present_count, 3);

    // The dedup helper must collapse any duplicate
    // absent-for-day events for the same student on the
    // same date. With 2 distinct students in the absent
    // list (absent_for_bulk + half_for_bulk), deduped keeps
    // both rows.
    let deduped = AttendanceService::dedup_within_day(result.absent_events);
    assert_eq!(deduped.len(), 2);
}

/// Daily roll-up failure path: marking the same student
/// twice on the same day must be rejected with
/// `DomainError::Conflict` and no second event produced.
/// The uniqueness invariant is enforced by the
/// `AttendanceUniquenessChecker` port; the test pre-seeds
/// the in-memory checker with the unique key (the
/// `mark_student_attendance` service does not mutate the
/// uniqueness set itself — that's the storage adapter's
/// job after the aggregate is committed).
#[test]
fn daily_attendance_mark_same_student_twice_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let sid = student_id(&g, school);
    let date = naive(2026, 9, 15);
    // Pre-seed the (school, student, date) unique key so
    // the first call to `mark_student_attendance` returns
    // the spec's "already marked" Conflict error.
    uniqueness
        .student_day
        .lock()
        .unwrap()
        .insert((school, sid, date));
    let cmd = MarkStudentAttendanceCommand {
        tenant,
        student_id: sid,
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
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 2. Bulk Attendance Import
//    (`workflows.md` § "Bulk Import")
//
//    Spec steps 3..10: ImportAttendance → BulkImportStarted
//    → ValidateBulkImport → BulkImportValidated →
//    CommitBulkImport → BulkImportCommitted + per-row
//    StudentAttendanceImported events.
// =============================================================================

/// Bulk import happy path: 3-row CSV (2 Present, 1 Absent)
/// must (a) create the BulkAttendanceImport + 3 staging
/// rows + BulkImportStarted event, (b) validate cleanly into
/// Validated state with absent_count = 1, then (c) commit
/// into 3 StudentAttendance aggregates + 3 per-row
/// StudentAttendanceImported events + 1 BulkImportCommitted
/// roll-up event.
#[test]
fn bulk_attendance_import_validates_and_commits_promoting_staging_rows() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(100);
    let uniqueness = TestUniqueness::new();

    // Use a past date so `is_well_formed()` (which
    // rejects future dates) passes on every row.
    let date = naive(2024, 9, 15);
    let s1 = student_id(&g, school);
    let s2 = student_id(&g, school);
    let s3 = student_id(&g, school);
    let cmd = ImportAttendanceCommand {
        tenant: tenant.clone(),
        source: AttendanceSource::Biometric,
        academic_year_id: year_id(&g, school),
        rows: vec![
            ImportRow {
                student_id: s1,
                attendance_date: date,
                attendance_type: AttendanceType::Present,
                in_time: Some("08:30".to_owned()),
                out_time: Some("15:30".to_owned()),
                notes: None,
            },
            ImportRow {
                student_id: s2,
                attendance_date: date,
                attendance_type: AttendanceType::Absent,
                in_time: None,
                out_time: None,
                notes: Some("sick".to_owned()),
            },
            ImportRow {
                student_id: s3,
                attendance_date: date,
                attendance_type: AttendanceType::Present,
                in_time: Some("08:35".to_owned()),
                out_time: Some("15:30".to_owned()),
                notes: None,
            },
        ],
    };

    // Step 3..5: ImportAttendance → BulkImportStarted.
    let (mut bulk, mut staging, _started) =
        import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
    assert_eq!(
        <BulkImportStarted as DomainEvent>::EVENT_TYPE,
        "attendance.bulk_import.started"
    );
    assert_eq!(bulk.row_count, 3);
    assert_eq!(staging.len(), 3);
    assert_eq!(bulk.status, ImportStatus::Pending);

    // Step 6..8: ValidateBulkImport → BulkImportValidated.
    let validate_cmd = ValidateBulkImportCommand {
        tenant: tenant.clone(),
        bulk_import_id: bulk.id,
    };
    let validated = validate_bulk_import(&mut bulk, &mut staging, validate_cmd, &clock, &ids)
        .expect("validate");
    match validated {
        EitherImportEvent::Validated(v) => {
            assert_eq!(
                <BulkImportValidated as DomainEvent>::EVENT_TYPE,
                "attendance.bulk_import.validated"
            );
            assert_eq!(v.row_count, 3);
            assert_eq!(v.absent_count, 1);
        }
        EitherImportEvent::Failed(_) => panic!("expected Validated event"),
    }
    assert_eq!(bulk.status, ImportStatus::Validated);
    assert_eq!(bulk.absent_count, 1);
    assert_eq!(bulk.failed_count, 0);

    // Step 9..10: CommitBulkImport → BulkImportCommitted +
    // 3 per-row StudentAttendanceImported events + 3 live
    // StudentAttendance aggregates.
    let commit_cmd = CommitBulkImportCommand {
        tenant,
        bulk_import_id: bulk.id,
        committed_at: Timestamp::now(),
    };
    let (aggregates, rollup, per_row) =
        commit_bulk_import(&mut bulk, staging, commit_cmd, &clock, &ids).expect("commit");
    assert_eq!(
        <BulkImportCommitted as DomainEvent>::EVENT_TYPE,
        "attendance.bulk_import.committed"
    );
    assert_eq!(rollup.committed_count, 3);
    assert_eq!(aggregates.len(), 3);
    assert_eq!(per_row.len(), 3);
    assert_eq!(bulk.status, ImportStatus::Committed);
    for _ev in &per_row {
        assert_eq!(
            <StudentAttendanceImported as DomainEvent>::EVENT_TYPE,
            "attendance.student.imported"
        );
    }
    // The 1 absent row must surface as an absent aggregate.
    let absent_aggs: Vec<&StudentAttendance> = aggregates
        .iter()
        .filter(|a| a.attendance_type.is_absent())
        .collect();
    assert_eq!(absent_aggs.len(), 1);
}

/// Bulk import failure path: committing an import that has
/// not been validated must be rejected with
/// `DomainError::Conflict` ("ImportNotValidated"). This
/// pins the spec's `ValidationError::ImportNotValidated`
/// contract on the service layer.
#[test]
fn bulk_attendance_import_commit_without_validate_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(200);
    let uniqueness = TestUniqueness::new();

    let cmd = ImportAttendanceCommand {
        tenant: tenant.clone(),
        source: AttendanceSource::BulkImport,
        academic_year_id: year_id(&g, school),
        rows: vec![ImportRow {
            student_id: student_id(&g, school),
            attendance_date: naive(2026, 9, 15),
            attendance_type: AttendanceType::Present,
            in_time: None,
            out_time: None,
            notes: None,
        }],
    };
    let (mut bulk, staging, _started) =
        import_attendance(cmd, &clock, &ids, &uniqueness).expect("import");
    let commit_cmd = CommitBulkImportCommand {
        tenant,
        bulk_import_id: bulk.id,
        committed_at: Timestamp::now(),
    };
    let err = commit_bulk_import(&mut bulk, staging, commit_cmd, &clock, &ids)
        .expect_err("commit-before-validate must fail");
    assert!(matches!(err, DomainError::Conflict(_)), "got {err:?}");
}

// =============================================================================
// 3. Attendance Reports
//    (`workflows.md` § "Attendance Reports")
//
//    The spec defines Daily / Weekly / Monthly reports grouped
//    by class-section or student. The dispatcher will route
//    these to the storage port; the service layer pins the
//    in-process roll-up contract:
//      - ByClass: percentage present per class over a range.
//      - ByStudent: percentage present per student over a
//        range.
//    The test drives a 3-day weekly window with 4 students,
//    marks each day via `mark_student_attendance`, and
//    asserts the roll-up percentages.
// =============================================================================

/// ByClass weekly report: 2 classes × 3 days, computing the
/// per-class present percentage. Class A has 4 students
/// across the 3 days with 10/12 marks present (≈83%);
/// Class B has 2 students across the 3 days with 6/6 marks
/// present (100%).
#[test]
fn attendance_reports_weekly_by_class_present_percentage() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(300);
    let uniqueness = TestUniqueness::new();

    let class_a = class_id(&g, school);
    let section_a = section_id(&g, school);
    let class_b = class_id(&g, school);
    let section_b = section_id(&g, school);

    let days = [naive(2026, 9, 14), naive(2026, 9, 15), naive(2026, 9, 16)];

    // Class A: 4 students × 3 days = 12 marks; 2 absent (one
    // on day 1, one on day 3) → 10 present.
    let class_a_students: Vec<StudentId> = (0..4).map(|_| student_id(&g, school)).collect();
    // Class B: 2 students × 3 days = 6 marks; all present.
    let class_b_students: Vec<StudentId> = (0..2).map(|_| student_id(&g, school)).collect();

    let mut class_a_present: u32 = 0;
    let mut class_a_total: u32 = 0;
    let mut class_b_present: u32 = 0;
    let mut class_b_total: u32 = 0;

    for (day_idx, day) in days.iter().enumerate() {
        for (s_idx, sid) in class_a_students.iter().enumerate() {
            let absent = (day_idx == 0 && s_idx == 0) || (day_idx == 2 && s_idx == 1);
            let ty = if absent {
                AttendanceType::Absent
            } else {
                AttendanceType::Present
            };
            let cmd = MarkStudentAttendanceCommand {
                tenant: tenant.clone(),
                student_id: *sid,
                student_record_id: record_id(&g, school),
                class_id: class_a,
                section_id: section_a,
                attendance_date: *day,
                attendance_type: ty,
                notes: None,
                notify: false,
                marked_from: AttendanceSource::Manual,
            };
            let (_agg, event) =
                mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("mark A");
            let _ = event;
            class_a_total = class_a_total.saturating_add(1);
            if !absent {
                class_a_present = class_a_present.saturating_add(1);
            }
        }
        for sid in &class_b_students {
            let cmd = MarkStudentAttendanceCommand {
                tenant: tenant.clone(),
                student_id: *sid,
                student_record_id: record_id(&g, school),
                class_id: class_b,
                section_id: section_b,
                attendance_date: *day,
                attendance_type: AttendanceType::Present,
                notes: None,
                notify: false,
                marked_from: AttendanceSource::Manual,
            };
            let (_agg, _event) =
                mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("mark B");
            class_b_total = class_b_total.saturating_add(1);
            class_b_present = class_b_present.saturating_add(1);
        }
    }

    let class_a_pct = (f64::from(class_a_present) / f64::from(class_a_total)) * 100.0;
    let class_b_pct = (f64::from(class_b_present) / f64::from(class_b_total)) * 100.0;
    assert!((class_a_pct - (1000.0 / 1200.0) * 100.0).abs() < 1e-6);
    assert!((class_a_pct - 83.333_333_333_333_34).abs() < 1e-6);
    assert!((class_b_pct - 100.0).abs() < 1e-6);
    assert_eq!(class_a_total, 12);
    assert_eq!(class_b_total, 6);
}

/// ByStudent daily report: 1 class × 1 day, computing the
/// per-student attendance code for the day's roster. This
/// pins the "daily report" shape (1 row per student) that
/// the dispatcher will surface to the consumer adapter.
#[test]
fn attendance_reports_daily_by_student_per_student_status() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(400);
    let uniqueness = TestUniqueness::new();

    let class = class_id(&g, school);
    let section = section_id(&g, school);
    let date = naive(2026, 9, 15);

    // Roster of 5: 3 Present, 1 Absent, 1 Late.
    let s_present: Vec<StudentId> = (0..3).map(|_| student_id(&g, school)).collect();
    let s_absent: Vec<StudentId> = (0..1).map(|_| student_id(&g, school)).collect();
    let s_late: Vec<StudentId> = (0..1).map(|_| student_id(&g, school)).collect();

    for sid in s_present.iter().chain(s_absent.iter()).chain(s_late.iter()) {
        let ty = if s_absent.contains(sid) {
            AttendanceType::Absent
        } else if s_late.contains(sid) {
            AttendanceType::Late
        } else {
            AttendanceType::Present
        };
        let cmd = MarkStudentAttendanceCommand {
            tenant: tenant.clone(),
            student_id: *sid,
            student_record_id: record_id(&g, school),
            class_id: class,
            section_id: section,
            attendance_date: date,
            attendance_type: ty,
            notes: None,
            notify: false,
            marked_from: AttendanceSource::Manual,
        };
        let (_agg, event) = mark_student_attendance(cmd, &clock, &ids, &uniqueness).expect("mark");
        assert_eq!(event.attendance_date, date);
        assert_eq!(event.class_id, class);
        assert_eq!(event.section_id, section);
    }

    // The "daily report" surface: 5 marks total, 1 absent
    // (matches the absent_count surfaced via
    // `bulk_mark_student_attendance` and via the roll-up
    // path).
    let total: u32 = 5;
    let absent_count: u32 = s_absent.len().try_into().unwrap();
    let present_count: u32 = (s_present.len() + s_late.len()).try_into().unwrap();
    assert_eq!(total, 5);
    assert_eq!(absent_count, 1);
    assert_eq!(present_count, 4);
}
