//! Integration tests for the **StaffAttendance aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`StaffAttendance`](educore_hr::aggregate::StaffAttendance)
//! end-to-end, plus the Wave 179 mutators that enforce spec
//! invariants I-1 (composite-key uniqueness), I-2 (attendance
//! type enum), and I-3 (attendance_date required).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::{AttendanceSource, AttendanceType, StaffAttendance};
use educore_hr::services::StaffAttendanceUniquenessChecker;
use educore_hr::value_objects::{StaffAttendanceId, StaffId};

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

fn staff_attendance_id(g: &SystemIdGen, school: SchoolId) -> StaffAttendanceId {
    StaffAttendanceId::new(school, g.next_uuid())
}

/// Helper: build a fresh StaffAttendance for tests.
fn fresh_staff_attendance(tenant: &TenantContext, g: &SystemIdGen) -> StaffAttendance {
    let school = tenant.school_id;
    let id = staff_attendance_id(g, school);
    let staff_id = StaffId::new(school, g.next_uuid());
    StaffAttendance::fresh(
        id,
        staff_id,
        NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        AttendanceType::Present,
        None,
        None,
        None,
        tenant.actor_id,
        Timestamp::now(),
        AttendanceSource::Manual,
        tenant.correlation_id,
    )
}

/// Configurable `StaffAttendanceUniquenessChecker` mock.
struct FakeStaffAttendanceUniqueness {
    exists: bool,
}
impl StaffAttendanceUniquenessChecker for FakeStaffAttendanceUniqueness {
    fn staff_attendance_exists(
        &self,
        _school: SchoolId,
        _staff_id: StaffId,
        _attendance_date: NaiveDate,
    ) -> bool {
        self.exists
    }
}

#[test]
fn staff_attendance_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = staff_attendance_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn staff_attendance_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = staff_attendance_id(&g, school);
    let id_b = staff_attendance_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 179 — Spec invariant StaffAttendance#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn staff_attendance_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let sa = fresh_staff_attendance(&tenant, &g);
    let checker = FakeStaffAttendanceUniqueness { exists: false };
    assert!(sa.ensure_unique(&checker).is_ok());
}

#[test]
fn staff_attendance_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let sa = fresh_staff_attendance(&tenant, &g);
    let checker = FakeStaffAttendanceUniqueness { exists: true };
    let err = sa.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 179 — Spec invariant StaffAttendance#3 (date required)
// =============================================================================

#[test]
fn staff_attendance_ensure_date_required_accepts_any_date() {
    let (tenant, g) = admin_context();
    let sa = fresh_staff_attendance(&tenant, &g);
    assert!(sa.ensure_date_required().is_ok());
}

// =============================================================================
// Wave 179 — Spec invariant StaffAttendance#2 (attendance_type enum)
// =============================================================================

#[test]
fn staff_attendance_ensure_attendance_type_valid_accepts_all_variants() {
    let (tenant, g) = admin_context();
    let mut sa = fresh_staff_attendance(&tenant, &g);
    for variant in [
        AttendanceType::Present,
        AttendanceType::Late,
        AttendanceType::Absent,
        AttendanceType::HalfDay,
        AttendanceType::Holiday,
    ] {
        sa.attendance_type = variant;
        assert!(sa.ensure_attendance_type_valid().is_ok());
    }
}
