//! Integration tests for the **PayrollPaymentApproval aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 81 per-aggregate drop
//! [`PayrollPaymentApproval`](educore_finance::entities::PayrollPaymentApproval) —
//! the approval workflow row attached to a `PayrollPayment`.
//! Validates PPA I-1 (state machine pending → approved/rejected;
//! invalid transitions return `DomainError::conflict`), PPA I-2
//! (every transition stamps `approver_id` + `approved_at` on the
//! aggregate; the reject path also captures `rejecter_id` +
//! `rejected_at` + optional `rejection_reason`), and the
//! `create_payroll_payment_approval` /
//! `approve_payroll_payment_approval` /
//! `reject_payroll_payment_approval` service functions (with
//! EVENT_TYPE / AGGREGATE_TYPE / SCHEMA_VERSION pinned).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `PayrollPaymentApproval` previously had only a
//! partial implementation (struct + `fresh()` + audit footer + the
//! state-machine methods were missing). Wave 81 adds the 6
//! state-machine methods (is_pending/is_approved/is_rejected/is_active
//! + approve + reject) to the existing entities.rs struct, the 3
//! headline events, the 3 service functions, and this test suite.
//!
//! Structurally parallel to the Wave 79 `tests/expense_approval.rs`
//! and Wave 80 `tests/income_approval.rs` suites, but with two key
//! differences:
//!   1. The PayrollPaymentApproval struct lives in entities.rs (not
//!      aggregate.rs), parallel to Wave 76 WalletTransactionApproval.
//!   2. The struct does NOT have its own id field — payroll_payment_id
//!      serves as the de-facto aggregate identifier, and the events
//!      use payroll_payment_id.as_uuid() in their aggregate_id()
//!      impl.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::{
    ApprovePayrollPaymentApprovalCommand, CreatePayrollPaymentApprovalCommand,
    RejectPayrollPaymentApprovalCommand,
};
use educore_finance::entities::PayrollPaymentApproval;
use educore_finance::events::{
    PayrollPaymentApprovalApproved, PayrollPaymentApprovalCreated, PayrollPaymentApprovalRejected,
};
use educore_finance::services::{
    approve_payroll_payment_approval, create_payroll_payment_approval,
    reject_payroll_payment_approval,
};
use educore_finance::value_objects::{PayrollPaymentApprovalId, PayrollPaymentId};

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

fn payroll_payment_id(g: &SystemIdGen, school: SchoolId) -> PayrollPaymentId {
    PayrollPaymentId::new(school, g.next_uuid())
}

fn payroll_payment_approval_id(g: &SystemIdGen, school: SchoolId) -> PayrollPaymentApprovalId {
    PayrollPaymentApprovalId::new(school, g.next_uuid())
}

fn make_payroll_payment_approval(
    g: &SystemIdGen,
    school: SchoolId,
    payroll_payment: PayrollPaymentId,
) -> PayrollPaymentApproval {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    PayrollPaymentApproval::fresh(
        payroll_payment,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 81 stub tests)
// =========================================================================

#[test]
fn payroll_payment_approval_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = payroll_payment_approval_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn payroll_payment_approval_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = payroll_payment_approval_id(&g, school);
    let id_b = payroll_payment_approval_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// PayrollPaymentApproval::fresh + initial state — PPA I-1 (Pending-only)
// =========================================================================

#[test]
fn fresh_starts_in_pending_state() {
    // PPA I-1: a fresh approval is always in Pending.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let row = make_payroll_payment_approval(&g, school, payroll_payment);
    assert!(row.is_pending());
    assert!(!row.is_approved());
    assert!(!row.is_rejected());
    assert!(row.is_active());
    assert!(row.approver_id.is_none());
    assert!(row.approved_at.is_none());
    assert!(row.rejecter_id.is_none());
    assert!(row.rejected_at.is_none());
    assert!(row.rejection_reason.is_none());
}

#[test]
fn fresh_inherits_school_id_from_parent() {
    // The PayrollPaymentApproval struct derives school_id from
    // payroll_payment_id.school_id() in fresh().
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let row = make_payroll_payment_approval(&g, school, payroll_payment);
    assert_eq!(row.school_id, school);
    assert_eq!(row.payroll_payment_id, payroll_payment);
}

// =========================================================================
// PayrollPaymentApproval::approve — PPA I-1 transition + PPA I-2 stamps
// =========================================================================

#[test]
fn approve_transitions_pending_to_approved() {
    // PPA I-1: Pending → Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let approver = g.next_user_id();
    let now = SystemClock.now();
    row.approve(now, approver).expect("approve");
    assert!(row.is_approved());
    assert!(!row.is_pending());
    assert_eq!(row.approver_id, Some(approver)); // PPA I-2
    assert_eq!(row.approved_at, Some(now)); // PPA I-2
    assert_eq!(row.updated_at, now);
}

#[test]
fn approve_rejects_already_approved() {
    // PPA I-1: Approved is a terminal state.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let now = SystemClock.now();
    row.approve(now, g.next_user_id()).expect("first approve");
    let err = row
        .approve(now, g.next_user_id())
        .expect_err("second approve must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn approve_rejects_already_rejected() {
    // PPA I-1: terminal Rejected cannot be transitioned to Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let now = SystemClock.now();
    row.reject(None, now, g.next_user_id())
        .expect("first reject");
    let err = row
        .approve(now, g.next_user_id())
        .expect_err("approve after reject must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// PayrollPaymentApproval::reject — PPA I-1 transition + PPA I-2 stamps
// =========================================================================

#[test]
fn reject_transitions_pending_to_rejected_with_reason() {
    // PPA I-1: Pending → Rejected.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let rejecter = g.next_user_id();
    let now = SystemClock.now();
    row.reject(
        Some("payroll run closed for the month".to_owned()),
        now,
        rejecter,
    )
    .expect("reject");
    assert!(row.is_rejected());
    assert!(!row.is_pending());
    assert_eq!(row.rejecter_id, Some(rejecter)); // PPA I-2
    assert_eq!(row.rejected_at, Some(now)); // PPA I-2
    assert_eq!(
        row.rejection_reason.as_deref(),
        Some("payroll run closed for the month")
    );
}

#[test]
fn reject_transitions_without_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let now = SystemClock.now();
    row.reject(None, now, g.next_user_id()).expect("reject");
    assert!(row.is_rejected());
    assert!(row.rejection_reason.is_none());
}

#[test]
fn reject_trims_and_drops_empty_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let now = SystemClock.now();
    row.reject(Some("  pad me  ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row.rejection_reason.as_deref(), Some("pad me"));
    let payroll_payment2 = payroll_payment_id(&g, school);
    let mut row2 = make_payroll_payment_approval(&g, school, payroll_payment2);
    row2.reject(Some("   ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row2.rejection_reason, None);
}

#[test]
fn reject_rejects_already_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let now = SystemClock.now();
    row.reject(None, now, g.next_user_id())
        .expect("first reject");
    let err = row
        .reject(None, now, g.next_user_id())
        .expect_err("second reject must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// Service functions
// =========================================================================

#[test]
fn create_service_produces_aggregate_and_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let cmd = CreatePayrollPaymentApprovalCommand {
        tenant: tenant.clone(),
        payroll_payment_id: payroll_payment,
    };
    let clock = SystemClock;
    let (row, event) = create_payroll_payment_approval(cmd, &clock, &g)
        .expect("create_payroll_payment_approval should succeed");
    assert_eq!(row.payroll_payment_id, payroll_payment);
    assert!(row.is_pending());
    assert_eq!(event.payroll_payment_id, payroll_payment);
    assert_eq!(
        <PayrollPaymentApprovalCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.payroll_payment_approval.created"
    );
    assert_eq!(
        <PayrollPaymentApprovalCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "payroll_payment_approval"
    );
    assert_eq!(
        <PayrollPaymentApprovalCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    // Aggregate_id is payroll_payment_id.as_uuid() (the struct has
    // no separate id field).
    assert_eq!(event.aggregate_id(), payroll_payment.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn approve_service_transitions_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let cmd = ApprovePayrollPaymentApprovalCommand {
        tenant: tenant.clone(),
        payroll_payment_id: payroll_payment,
    };
    let clock = SystemClock;
    let event = approve_payroll_payment_approval(cmd, &clock, &g, &mut row)
        .expect("approve_payroll_payment_approval should succeed");
    assert!(row.is_approved());
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(event.payroll_payment_id, payroll_payment);
    assert_eq!(
        <PayrollPaymentApprovalApproved as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.payroll_payment_approval.approved"
    );
    assert_eq!(
        <PayrollPaymentApprovalApproved as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "payroll_payment_approval"
    );
    assert_eq!(
        <PayrollPaymentApprovalApproved as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn approve_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let clock = SystemClock;
    // First approve via service to drive into terminal state.
    approve_payroll_payment_approval(
        ApprovePayrollPaymentApprovalCommand {
            tenant: tenant.clone(),
            payroll_payment_id: payroll_payment,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first approve");
    let err = approve_payroll_payment_approval(
        ApprovePayrollPaymentApprovalCommand {
            tenant,
            payroll_payment_id: payroll_payment,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect_err("approve after approve must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn reject_service_transitions_and_emits_event_with_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let cmd = RejectPayrollPaymentApprovalCommand {
        tenant: tenant.clone(),
        payroll_payment_id: payroll_payment,
        reason: Some("duplicate payroll run".to_owned()),
    };
    let clock = SystemClock;
    let event = reject_payroll_payment_approval(cmd, &clock, &g, &mut row)
        .expect("reject_payroll_payment_approval should succeed");
    assert!(row.is_rejected());
    assert_eq!(
        row.rejection_reason.as_deref(),
        Some("duplicate payroll run")
    );
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(
        event.rejection_reason.as_deref(),
        Some("duplicate payroll run")
    );
    assert_eq!(
        <PayrollPaymentApprovalRejected as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.payroll_payment_approval.rejected"
    );
    assert_eq!(
        <PayrollPaymentApprovalRejected as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "payroll_payment_approval"
    );
    assert_eq!(
        <PayrollPaymentApprovalRejected as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn reject_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let payroll_payment = payroll_payment_id(&g, school);
    let mut row = make_payroll_payment_approval(&g, school, payroll_payment);
    let clock = SystemClock;
    reject_payroll_payment_approval(
        RejectPayrollPaymentApprovalCommand {
            tenant: tenant.clone(),
            payroll_payment_id: payroll_payment,
            reason: None,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first reject");
    let err = reject_payroll_payment_approval(
        RejectPayrollPaymentApprovalCommand {
            tenant,
            payroll_payment_id: payroll_payment,
            reason: None,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect_err("reject after reject must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}
