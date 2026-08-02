//! Integration tests for the **PayrollGenerate aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`PayrollGenerate`](educore_hr::aggregate::PayrollGenerate) end-to-end
//! and exercises the Wave 172 mutators that enforce spec invariants
//! I-1 (gross == basic + total_earning), I-3 (status FSM), and I-4
//! (paid_amount <= net_salary).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::PayrollGenerate;
use educore_hr::value_objects::{
    PayrollGenerateId, PayrollPaymentStatus, PayrollStatus, StaffId, validate_paid_amount,
};

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

fn payroll_generate_id(g: &SystemIdGen, school: SchoolId) -> PayrollGenerateId {
    PayrollGenerateId::new(school, g.next_uuid())
}

fn fresh_payroll(g: &SystemIdGen, tenant: &TenantContext) -> PayrollGenerate {
    let id = payroll_generate_id(g, tenant.school_id);
    let staff_id = StaffId::new(tenant.school_id, g.next_uuid());
    PayrollGenerate::fresh(
        id,
        staff_id,
        1000.0,
        7,
        2026,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

#[test]
fn payroll_generate_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = payroll_generate_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn payroll_generate_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = payroll_generate_id(&g, school);
    let id_b = payroll_generate_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Spec invariant PayrollGenerate#3 — status FSM (NotGenerated -> Generated -> Paid)
// =============================================================================

#[test]
fn payroll_fsm_not_generated_can_transition_to_generated() {
    assert!(PayrollStatus::NotGenerated.can_transition_to(PayrollStatus::Generated));
}

#[test]
fn payroll_fsm_generated_can_transition_to_paid() {
    assert!(PayrollStatus::Generated.can_transition_to(PayrollStatus::Paid));
}

#[test]
fn payroll_fsm_paid_is_terminal() {
    assert!(!PayrollStatus::Paid.can_transition_to(PayrollStatus::NotGenerated));
    assert!(!PayrollStatus::Paid.can_transition_to(PayrollStatus::Generated));
    assert!(!PayrollStatus::Paid.can_transition_to(PayrollStatus::Paid));
}

#[test]
fn payroll_fsm_cannot_skip_generated_to_paid() {
    assert!(!PayrollStatus::NotGenerated.can_transition_to(PayrollStatus::Paid));
}

#[test]
fn payroll_fsm_cannot_go_backwards() {
    assert!(!PayrollStatus::Generated.can_transition_to(PayrollStatus::NotGenerated));
}

#[test]
fn payroll_mark_generated_advances_fsm() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    assert_eq!(p.payroll_status, PayrollStatus::NotGenerated);
    p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap();
    assert_eq!(p.payroll_status, PayrollStatus::Generated);
}

#[test]
fn payroll_mark_paid_full_advances_fsm_to_paid() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap();
    // Set net_salary so mark_paid can succeed with paid_amount == net_salary.
    p.net_salary = 1000.0;
    p.mark_paid(Timestamp::now(), tenant.actor_id, 1000.0).unwrap();
    assert_eq!(p.payroll_status, PayrollStatus::Paid);
    assert_eq!(p.paid_amount, 1000.0);
    assert!(!p.is_partial);
    assert_eq!(p.payment_status, PayrollPaymentStatus::FullyPaid);
}

#[test]
fn payroll_mark_paid_partial_does_not_advance_fsm() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap();
    p.net_salary = 1000.0;
    p.mark_paid(Timestamp::now(), tenant.actor_id, 400.0).unwrap();
    // Partial payment: FSM stays at Generated, is_partial flips to true.
    assert_eq!(p.payroll_status, PayrollStatus::Generated);
    assert!(p.is_partial);
    assert_eq!(p.payment_status, PayrollPaymentStatus::Partial);
}

#[test]
fn payroll_mark_paid_rejects_when_already_paid() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap();
    p.net_salary = 1000.0;
    p.mark_paid(Timestamp::now(), tenant.actor_id, 1000.0).unwrap();
    // Second mark_paid: FSM is already Paid, must reject.
    let err = p.mark_paid(Timestamp::now(), tenant.actor_id, 1000.0).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("cannot mark payroll"), "unexpected error: {msg}");
}

#[test]
fn payroll_mark_generated_rejects_from_generated() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap();
    // Second mark_generated: FSM is already Generated, must reject.
    let err = p.mark_generated(Timestamp::now(), tenant.actor_id).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("cannot mark payroll"), "unexpected error: {msg}");
}

// =============================================================================
// Spec invariant PayrollGenerate#4 — paid_amount <= net_salary
// =============================================================================

#[test]
fn payroll_validate_paid_amount_accepts_zero() {
    validate_paid_amount(0.0, 1000.0).unwrap();
}

#[test]
fn payroll_validate_paid_amount_accepts_exact_net_salary() {
    validate_paid_amount(1000.0, 1000.0).unwrap();
}

#[test]
fn payroll_validate_paid_amount_rejects_negative() {
    let err = validate_paid_amount(-1.0, 1000.0).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("paid_amount must be >= 0.0"), "unexpected error: {msg}");
}

#[test]
fn payroll_validate_paid_amount_rejects_exceeds_net_salary() {
    let err = validate_paid_amount(1001.0, 1000.0).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("exceeds net_salary"), "unexpected error: {msg}");
}

#[test]
fn payroll_record_payment_rejects_exceeds_net_salary() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.net_salary = 500.0;
    let err = p.record_payment(501.0).unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("exceeds net_salary"), "unexpected error: {msg}");
}

#[test]
fn payroll_record_payment_sets_partial_flag() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.net_salary = 1000.0;
    p.record_payment(750.0).unwrap();
    assert!(p.is_partial);
    assert_eq!(p.paid_amount, 750.0);
    assert_eq!(p.payment_status, PayrollPaymentStatus::Partial);
}

#[test]
fn payroll_record_payment_full_does_not_set_partial() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.net_salary = 1000.0;
    p.record_payment(1000.0).unwrap();
    assert!(!p.is_partial);
    assert_eq!(p.paid_amount, 1000.0);
    assert_eq!(p.payment_status, PayrollPaymentStatus::FullyPaid);
}

// =============================================================================
// Spec invariant PayrollGenerate#1 — gross == basic + total_earning
// =============================================================================

#[test]
fn payroll_update_amounts_derives_gross_salary_correctly() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    p.basic_salary = 1000.0;
    p.update_amounts(500.0, 200.0, 100.0, Timestamp::now(), tenant.actor_id).unwrap();
    // gross_salary = basic_salary + total_earning = 1000 + 500 = 1500
    assert_eq!(p.gross_salary, 1500.0);
    // net_salary = gross - total_deduction - tax = 1500 - 200 - 100 = 1200
    assert_eq!(p.net_salary, 1200.0);
}

#[test]
fn payroll_update_amounts_rejects_negative_total_earning() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    let err = p
        .update_amounts(-1.0, 0.0, 0.0, Timestamp::now(), tenant.actor_id)
        .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("total_earning must be >= 0.0"), "unexpected error: {msg}");
}

#[test]
fn payroll_update_amounts_rejects_negative_total_deduction() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    let err = p
        .update_amounts(0.0, -1.0, 0.0, Timestamp::now(), tenant.actor_id)
        .unwrap_err();
    let msg = format!("{err:?}");
    assert!(
        msg.contains("total_deduction must be >= 0.0"),
        "unexpected error: {msg}"
    );
}

#[test]
fn payroll_update_amounts_rejects_negative_tax() {
    let (tenant, g) = admin_context();
    let mut p = fresh_payroll(&g, &tenant);
    let err = p
        .update_amounts(0.0, 0.0, -1.0, Timestamp::now(), tenant.actor_id)
        .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("tax must be >= 0.0"), "unexpected error: {msg}");
}
