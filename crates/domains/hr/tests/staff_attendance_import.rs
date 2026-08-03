//! Integration tests for the **StaffAttendanceImport aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`StaffAttendanceImport`](educore_hr::aggregate::StaffAttendanceImport)
//! end-to-end, plus the Wave 188 mutators that enforce spec
//! invariants I-1 (composite-key uniqueness), I-2 (time fields
//! are stored as String for arbitrary source formats), and I-3
//! (active while pending promotion).

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
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_hr::prelude::{AttendanceSource, AttendanceType, StaffAttendanceImport};
use educore_hr::services::StaffAttendanceImportUniquenessChecker;
use educore_hr::value_objects::{StaffAttendanceImportId, StaffId};

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

fn staff_attendance_import_id(g: &SystemIdGen, school: SchoolId) -> StaffAttendanceImportId {
    StaffAttendanceImportId::new(school, g.next_uuid())
}

/// Helper: build a fresh StaffAttendanceImport for tests.
fn fresh_staff_attendance_import(
    tenant: &TenantContext,
    g: &SystemIdGen,
) -> StaffAttendanceImport {
    let school = tenant.school_id;
    let id = staff_attendance_import_id(g, school);
    let staff_id = StaffId::new(school, g.next_uuid());
    StaffAttendanceImport::fresh(
        id,
        staff_id,
        AttendanceSource::Import,
        NaiveDate::from_ymd_opt(2026, 8, 20).unwrap(),
        AttendanceType::Present,
        Some("09:00".to_owned()),
        Some("17:00".to_owned()),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `StaffAttendanceImportUniquenessChecker` mock.
struct FakeStaffAttendanceImportUniqueness {
    exists: bool,
}
impl StaffAttendanceImportUniquenessChecker for FakeStaffAttendanceImportUniqueness {
    fn staff_attendance_import_exists(
        &self,
        _school: SchoolId,
        _staff_id: StaffId,
        _attendance_date: NaiveDate,
    ) -> bool {
        self.exists
    }
}

#[test]
fn staff_attendance_import_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = staff_attendance_import_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn staff_attendance_import_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = staff_attendance_import_id(&g, school);
    let id_b = staff_attendance_import_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 188 — Spec invariant StaffAttendanceImport#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn staff_attendance_import_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let sai = fresh_staff_attendance_import(&tenant, &g);
    let checker = FakeStaffAttendanceImportUniqueness { exists: false };
    assert!(sai.ensure_unique(&checker).is_ok());
}

#[test]
fn staff_attendance_import_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let sai = fresh_staff_attendance_import(&tenant, &g);
    let checker = FakeStaffAttendanceImportUniqueness { exists: true };
    let err = sai.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 188 — Spec invariant StaffAttendanceImport#2 (time fields as String)
// =============================================================================

#[test]
fn staff_attendance_import_ensure_time_fields_valid_accepts_any_string() {
    let (tenant, g) = admin_context();
    let sai = fresh_staff_attendance_import(&tenant, &g);
    assert!(sai.ensure_time_fields_valid().is_ok());
}

#[test]
fn staff_attendance_import_ensure_time_fields_valid_accepts_no_times() {
    let (tenant, g) = admin_context();
    let mut sai = fresh_staff_attendance_import(&tenant, &g);
    sai.in_time = None;
    sai.out_time = None;
    assert!(sai.ensure_time_fields_valid().is_ok());
}

// =============================================================================
// Wave 188 — Spec invariant StaffAttendanceImport#3 (active while pending)
// =============================================================================

#[test]
fn staff_attendance_import_ensure_active_accepts_active() {
    let (tenant, g) = admin_context();
    let sai = fresh_staff_attendance_import(&tenant, &g);
    assert!(sai.ensure_active().is_ok());
}

#[test]
fn staff_attendance_import_ensure_active_rejects_inactive() {
    let (tenant, g) = admin_context();
    let mut sai = fresh_staff_attendance_import(&tenant, &g);
    sai.active_status = ActiveStatus::Retired;
    let err = sai.ensure_active().expect_err("inactive must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
