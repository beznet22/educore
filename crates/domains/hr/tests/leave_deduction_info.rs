//! Integration tests for the **LeaveDeductionInfo aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`LeaveDeductionInfo`](educore_hr::aggregate::LeaveDeductionInfo) end-to-end,
//! plus the Wave 178 mutators that enforce spec invariants
//! I-1 (composite-key uniqueness), I-2 (non-negative fields),
//! and I-3 (active while applied).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::LeaveDeductionInfo;
use educore_hr::services::LeaveDeductionInfoUniquenessChecker;
use educore_hr::value_objects::{LeaveDeductionInfoId, PayrollGenerateId, StaffId};

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

fn leave_deduction_info_id(g: &SystemIdGen, school: SchoolId) -> LeaveDeductionInfoId {
    LeaveDeductionInfoId::new(school, g.next_uuid())
}

/// Helper: build a fresh LeaveDeductionInfo for tests.
fn fresh_leave_deduction_info(tenant: &TenantContext, g: &SystemIdGen) -> LeaveDeductionInfo {
    let school = tenant.school_id;
    let id = leave_deduction_info_id(g, school);
    let staff_id = StaffId::new(school, g.next_uuid());
    let payroll_id = PayrollGenerateId::new(school, g.next_uuid());
    LeaveDeductionInfo::fresh(
        id,
        staff_id,
        payroll_id,
        2,    // extra_leave
        50.0, // salary_deduct
        7,
        2026,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `LeaveDeductionInfoUniquenessChecker` mock.
struct FakeLeaveDeductionInfoUniqueness {
    exists: bool,
}
impl LeaveDeductionInfoUniquenessChecker for FakeLeaveDeductionInfoUniqueness {
    fn leave_deduction_info_exists(
        &self,
        _school: SchoolId,
        _staff_id: StaffId,
        _payroll_id: PayrollGenerateId,
    ) -> bool {
        self.exists
    }
}

#[test]
fn leave_deduction_info_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_deduction_info_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn leave_deduction_info_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = leave_deduction_info_id(&g, school);
    let id_b = leave_deduction_info_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 178 — Spec invariant LeaveDeductionInfo#2 (non-negative)
// =============================================================================

#[test]
fn leave_deduction_info_ensure_non_negative_accepts_zero() {
    let (tenant, g) = admin_context();
    let mut ldi = fresh_leave_deduction_info(&tenant, &g);
    ldi.extra_leave = 0;
    ldi.salary_deduct = 0.0;
    assert!(ldi.ensure_non_negative().is_ok());
}

#[test]
fn leave_deduction_info_ensure_non_negative_accepts_positive() {
    let (tenant, g) = admin_context();
    let ldi = fresh_leave_deduction_info(&tenant, &g);
    assert_eq!(ldi.extra_leave, 2);
    assert_eq!(ldi.salary_deduct, 50.0);
    assert!(ldi.ensure_non_negative().is_ok());
}

#[test]
fn leave_deduction_info_ensure_non_negative_rejects_negative_salary() {
    let (tenant, g) = admin_context();
    let mut ldi = fresh_leave_deduction_info(&tenant, &g);
    ldi.salary_deduct = -1.0;
    let err = ldi.ensure_non_negative().expect_err("negative salary must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// Wave 178 — Spec invariant LeaveDeductionInfo#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn leave_deduction_info_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let ldi = fresh_leave_deduction_info(&tenant, &g);
    let checker = FakeLeaveDeductionInfoUniqueness { exists: false };
    assert!(ldi.ensure_unique(&checker).is_ok());
}

#[test]
fn leave_deduction_info_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let ldi = fresh_leave_deduction_info(&tenant, &g);
    let checker = FakeLeaveDeductionInfoUniqueness { exists: true };
    let err = ldi
        .ensure_unique(&checker)
        .expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 178 — Spec invariant LeaveDeductionInfo#3 (active while applied)
// =============================================================================

#[test]
fn leave_deduction_info_ensure_active_accepts_active() {
    let (tenant, g) = admin_context();
    let ldi = fresh_leave_deduction_info(&tenant, &g);
    assert_eq!(ldi.active_status, 1);
    assert!(ldi.ensure_active().is_ok());
}

#[test]
fn leave_deduction_info_ensure_active_rejects_inactive() {
    let (tenant, g) = admin_context();
    let mut ldi = fresh_leave_deduction_info(&tenant, &g);
    ldi.active_status = 0;
    let err = ldi.ensure_active().expect_err("inactive must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
