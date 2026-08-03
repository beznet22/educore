//! Integration tests for the **PayrollEarnDeduc aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`PayrollEarnDeduc`](educore_hr::aggregate::PayrollEarnDeduc)
//! end-to-end, plus the Wave 179 mutators that enforce spec
//! invariants I-1 (amount >= 0) and I-2 (earn_dedc_type enum).

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
use educore_hr::prelude::{EarnDeducType, PayrollEarnDeduc};
use educore_hr::value_objects::{PayrollEarnDeducId, PayrollGenerateId};

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

fn payroll_earn_deduc_id(g: &SystemIdGen, school: SchoolId) -> PayrollEarnDeducId {
    PayrollEarnDeducId::new(school, g.next_uuid())
}

/// Helper: build a fresh PayrollEarnDeduc for tests.
fn fresh_payroll_earn_deduc(
    tenant: &TenantContext,
    g: &SystemIdGen,
    amount: f64,
) -> PayrollEarnDeduc {
    let school = tenant.school_id;
    let id = payroll_earn_deduc_id(g, school);
    let payroll_id = PayrollGenerateId::new(school, g.next_uuid());
    PayrollEarnDeduc::fresh(
        id,
        payroll_id,
        "Bonus".to_owned(),
        amount,
        EarnDeducType::Earning,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

#[test]
fn payroll_earn_deduc_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = payroll_earn_deduc_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn payroll_earn_deduc_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = payroll_earn_deduc_id(&g, school);
    let id_b = payroll_earn_deduc_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 179 — Spec invariant PayrollEarnDeduc#1 (amount >= 0)
// =============================================================================

#[test]
fn payroll_earn_deduc_ensure_amount_non_negative_accepts_zero() {
    let (tenant, g) = admin_context();
    let ped = fresh_payroll_earn_deduc(&tenant, &g, 0.0);
    assert!(ped.ensure_amount_non_negative().is_ok());
}

#[test]
fn payroll_earn_deduc_ensure_amount_non_negative_accepts_positive() {
    let (tenant, g) = admin_context();
    let ped = fresh_payroll_earn_deduc(&tenant, &g, 500.0);
    assert!(ped.ensure_amount_non_negative().is_ok());
}

#[test]
fn payroll_earn_deduc_ensure_amount_non_negative_rejects_negative() {
    let (tenant, g) = admin_context();
    let ped = fresh_payroll_earn_deduc(&tenant, &g, -1.0);
    let err = ped
        .ensure_amount_non_negative()
        .expect_err("negative amount must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// Wave 179 — Spec invariant PayrollEarnDeduc#2 (earn_dedc_type enum)
// =============================================================================

#[test]
fn payroll_earn_deduc_ensure_earn_dedc_type_valid_accepts_both_variants() {
    let (tenant, g) = admin_context();
    let mut ped = fresh_payroll_earn_deduc(&tenant, &g, 100.0);
    ped.earn_dedc_type = EarnDeducType::Earning;
    assert!(ped.ensure_earn_dedc_type_valid().is_ok());
    ped.earn_dedc_type = EarnDeducType::Deduction;
    assert!(ped.ensure_earn_dedc_type_valid().is_ok());
}
