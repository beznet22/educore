//! Integration tests for the **StaffAttendance aggregate**
//! vertical slice.
//!
//! Pins the create + update contract for
//! [`StaffAttendance`](educore_attendance::aggregate::StaffAttendance)
//! end-to-end through the service layer:
//!
//! 1. `mark_staff_attendance` validates the input, asserts
//!    the per-(school, staff, date) uniqueness invariant
//!    via the [`AttendanceUniquenessChecker`] port,
//!    constructs the aggregate, and emits a
//!    [`StaffAttendanceMarked`] event.
//! 2. `update_staff_attendance` mutates the in-place
//!    aggregate (bumps `version`, updates `updated_at` /
//!    `updated_by`) and emits a [`StaffAttendanceUpdated`]
//!    event carrying the list of changed field names.
//!
//! Mirrors the lean template used by
//! `tests/subject_attendance.rs` (Wave 4): two happy-path
//! tests for the create + update lifecycle, plus one
//! validation-failure test for the per-day uniqueness
//! conflict path. Handlers are not wired end-to-end (no
//! outbox / bus subscriber / audit row); these tests pin
//! the service-layer contract that the dispatcher will
//! eventually wrap.
//!
//! Spec: `docs/specs/attendance/aggregates.md` §
//! StaffAttendance.

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

/// In-memory uniqueness checker for the staff-attendance
/// create-flow tests. The `staff_day_exists` check is the
/// only one the staff create flow reaches; the other three
/// methods are `false`-returning stubs that mirror the
/// `AttendanceUniquenessChecker` port's full surface.
#[derive(Debug, Default)]
struct TestUniqueness {
    staff_day: Mutex<HashSet<(SchoolId, StaffId, NaiveDate)>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self::default()
    }

    /// Pre-seed a (school, staff, date) key to drive the
    /// uniqueness-conflict path on the next
    /// `mark_staff_attendance` call.
    fn seed_staff_day(&self, school: SchoolId, staff: StaffId, date: NaiveDate) {
        self.staff_day
            .lock()
            .expect("poisoned")
            .insert((school, staff, date));
    }
}

impl AttendanceUniquenessChecker for TestUniqueness {
    fn student_day_exists(&self, _school: SchoolId, _student: StudentId, _date: NaiveDate) -> bool {
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
    fn staff_day_exists(&self, school: SchoolId, staff: StaffId, date: NaiveDate) -> bool {
        self.staff_day
            .lock()
            .expect("poisoned")
            .contains(&(school, staff, date))
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

fn staff_id(g: &SystemIdGen, school: SchoolId) -> StaffId {
    StaffId::new(school, g.next_uuid())
}

fn naive(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: create + update on StaffAttendance
// =============================================================================

/// End-to-end happy path for the StaffAttendance
/// aggregate. Mark a staff member present, then update the
/// mark to "absent" with a reason note, asserting that:
///
/// 1. The create flow produces a `StaffAttendance`
///    aggregate carrying every field on the command + a
///    `StaffAttendanceMarked` event with the right
///    `event_type`, `aggregate_type`, and `school_id`.
/// 2. The update flow mutates the aggregate in place
///    (bumps `version`, swaps `attendance_type`, sets the
///    new `notes`), and emits a `StaffAttendanceUpdated`
///    event whose `changes` list names the fields that
///    actually moved.
#[test]
fn staff_attendance_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let staff = staff_id(&g, school);
    let date = naive(2026, 9, 15);

    // ---- Create flow ----
    let create_cmd = MarkStaffAttendanceCommand {
        tenant: tenant.clone(),
        staff_id: staff,
        attendance_date: date,
        attendance_type: AttendanceType::Present,
        notes: Some("on time".to_owned()),
        marked_from: AttendanceSource::Manual,
    };
    let (mut agg, marked_event) =
        mark_staff_attendance(create_cmd, &clock, &ids, &uniqueness).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.staff_id, staff);
    assert_eq!(agg.attendance_date, date);
    assert_eq!(agg.attendance_type, AttendanceType::Present);
    assert_eq!(agg.notes.as_deref(), Some("on time"));
    assert!(agg.in_time.is_none());
    assert!(agg.out_time.is_none());
    assert!(!agg.is_absent());
    // Audit metadata footer is initialised.
    assert_eq!(agg.version.get(), 1);
    assert!(agg.is_active());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's
    // contract for `StaffAttendanceMarked`.
    assert_eq!(
        <StaffAttendanceMarked as DomainEvent>::EVENT_TYPE,
        "attendance.staff.marked"
    );
    assert_eq!(
        <StaffAttendanceMarked as DomainEvent>::AGGREGATE_TYPE,
        "staff_attendance"
    );
    assert_eq!(<StaffAttendanceMarked as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(marked_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(marked_event.school_id(), school);
    assert_eq!(marked_event.staff_attendance_id, agg.id);
    assert_eq!(marked_event.staff_id, staff);
    assert_eq!(marked_event.attendance_date, date);
    assert_eq!(marked_event.attendance_type, AttendanceType::Present);
    assert_eq!(marked_event.notes.as_deref(), Some("on time"));
    assert_eq!(marked_event.marked_by, tenant.actor_id);
    assert_eq!(marked_event.marked_from, AttendanceSource::Manual);

    // ---- Update flow ----
    let initial_version = agg.version.get();
    let update_cmd = UpdateStaffAttendanceCommand {
        tenant: tenant.clone(),
        staff_attendance_id: agg.id,
        attendance_type: Some(AttendanceType::Absent),
        notes: Some("sick leave".to_owned()),
    };
    let updated_event =
        update_staff_attendance(&tenant, &mut agg, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place.
    assert_eq!(agg.attendance_type, AttendanceType::Absent);
    assert_eq!(agg.notes.as_deref(), Some("sick leave"));
    assert!(agg.is_absent());
    assert_eq!(agg.version.get(), initial_version + 1);
    assert_eq!(agg.updated_by, tenant.actor_id);

    // The event names the fields that actually moved.
    assert_eq!(
        <StaffAttendanceUpdated as DomainEvent>::EVENT_TYPE,
        "attendance.staff.updated"
    );
    assert_eq!(
        <StaffAttendanceUpdated as DomainEvent>::AGGREGATE_TYPE,
        "staff_attendance"
    );
    assert_eq!(updated_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(updated_event.staff_attendance_id, agg.id);
    assert!(updated_event
        .changes
        .contains(&"attendance_type".to_owned()));
    assert!(updated_event.changes.contains(&"notes".to_owned()));
}

// =============================================================================
// Validation failure: unique-key conflict on the create flow
// =============================================================================

/// Validation-failure path on the create flow: when the
/// per-(school, staff, date) unique key is already taken,
/// `mark_staff_attendance` returns `DomainError::Conflict`
/// and produces no aggregate / event.
///
/// The uniqueness invariant is enforced by the
/// `AttendanceUniquenessChecker` port; the service does not
/// mutate the uniqueness set itself (that's the storage
/// adapter's job after the aggregate is committed), so the
/// test pre-seeds the in-memory checker with the unique
/// key.
#[test]
fn staff_attendance_create_rejects_duplicate_day_with_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let staff = staff_id(&g, school);
    let date = naive(2026, 9, 15);

    // Pre-seed the unique key so the create call returns
    // the spec's "already marked" Conflict error.
    uniqueness.seed_staff_day(school, staff, date);

    let cmd = MarkStaffAttendanceCommand {
        tenant,
        staff_id: staff,
        attendance_date: date,
        attendance_type: AttendanceType::Present,
        notes: None,
        marked_from: AttendanceSource::Manual,
    };
    let err = mark_staff_attendance(cmd, &clock, &ids, &uniqueness)
        .expect_err("duplicate day mark must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

/// Validation-failure path on the update flow: when no
/// fields on the `UpdateStaffAttendanceCommand` actually
/// change the aggregate, `update_staff_attendance`
/// returns `DomainError::Validation` and the aggregate
/// stays at its original version.
#[test]
fn staff_attendance_update_with_no_changes_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = DeterministicIdGen::starting_at(1);
    let uniqueness = TestUniqueness::new();

    let create_cmd = MarkStaffAttendanceCommand {
        tenant: tenant.clone(),
        staff_id: staff_id(&g, school),
        attendance_date: naive(2026, 9, 15),
        attendance_type: AttendanceType::Present,
        notes: Some("baseline".to_owned()),
        marked_from: AttendanceSource::Manual,
    };
    let (mut agg, _event) =
        mark_staff_attendance(create_cmd, &clock, &ids, &uniqueness).expect("create");
    let initial_version = agg.version.get();

    // Update with `None` for every field — guaranteed not
    // to change the aggregate.
    let noop_update = UpdateStaffAttendanceCommand {
        tenant: tenant.clone(),
        staff_attendance_id: agg.id,
        attendance_type: None,
        notes: None,
    };
    let err = update_staff_attendance(&tenant, &mut agg, noop_update, &clock, &ids)
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
