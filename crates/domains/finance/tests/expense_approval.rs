//! Integration tests for the **ExpenseApproval aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 79 per-aggregate drop
//! [`RealExpenseApproval`](educore_finance::aggregate::RealExpenseApproval) —
//! the approval workflow row attached to an [`Expense`]. Validates
//! EA I-1 (state machine pending → approved/rejected; invalid
//! transitions return `DomainError::conflict`), EA I-2 (every
//! transition stamps `decided_by` + `decided_at` on the aggregate;
//! the reject path also captures an optional `reason`), and the
//! `create_expense_approval` / `approve_expense_approval` /
//! `reject_expense_approval` service functions (with EVENT_TYPE /
//! AGGREGATE_TYPE / SCHEMA_VERSION pinned).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `ExpenseApproval` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! ExpenseApproval { _id: () } }` placeholder. Wave 79 adds the
//! `RealExpenseApproval` aggregate (state-machine on
//! `ApprovalStatus::Pending`), the 3 headline events (Created /
//! Approved / Rejected), the 3 service functions, and this test
//! suite. The orphaned `ExpenseApprovalRecorded` event from the
//! earlier Phase 7 stub is preserved untouched for backwards
//! compatibility.

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
    ApproveExpenseApprovalCommand, CreateExpenseApprovalCommand, RejectExpenseApprovalCommand,
};
use educore_finance::events::{
    ExpenseApprovalApproved, ExpenseApprovalCreated, ExpenseApprovalRejected,
};
use educore_finance::prelude::RealExpenseApproval;
use educore_finance::services::{
    approve_expense_approval, create_expense_approval, reject_expense_approval,
};
use educore_finance::value_objects::{ExpenseApprovalId, ExpenseId};

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

fn expense_approval_id(g: &SystemIdGen, school: SchoolId) -> ExpenseApprovalId {
    ExpenseApprovalId::new(school, g.next_uuid())
}

fn expense_id(g: &SystemIdGen, school: SchoolId) -> ExpenseId {
    ExpenseId::new(school, g.next_uuid())
}

fn make_expense_approval(
    g: &SystemIdGen,
    school: SchoolId,
    expense: ExpenseId,
) -> RealExpenseApproval {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    RealExpenseApproval::fresh(
        expense_approval_id(g, school),
        expense,
        actor,
        now,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh RealExpenseApproval")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 79 stub tests)
// =========================================================================

#[test]
fn expense_approval_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_approval_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn expense_approval_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = expense_approval_id(&g, school);
    let id_b = expense_approval_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealExpenseApproval::fresh — EA I-1 (Pending-only) + EA I-2 (initial
// timestamps)
// =========================================================================

#[test]
fn fresh_starts_in_pending_state() {
    // EA I-1: a fresh approval is always in Pending.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let row = make_expense_approval(&g, school, expense);
    assert!(row.is_pending());
    assert!(!row.is_approved());
    assert!(!row.is_rejected());
    assert!(row.is_active());
}

#[test]
fn fresh_records_requested_by_and_requested_at() {
    // EA I-2 partial: requested_by + requested_at are stamped on
    // creation (decided_by + decided_at remain None until the
    // approval is decided).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let row = make_expense_approval(&g, school, expense);
    assert_eq!(row.requested_by, row.created_by);
    assert_eq!(row.requested_at, row.created_at);
    assert!(row.decided_by.is_none());
    assert!(row.decided_at.is_none());
    assert!(row.reject_reason.is_none());
}

#[test]
fn fresh_rejects_cross_school_expense() {
    // EA I-1 cross-school defense-in-depth: the expense must belong
    // to the same school as the approval.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let other_school = g.next_school_id();
    assert_ne!(other_school, school);
    let cross_school_expense = expense_id(&g, other_school);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealExpenseApproval::fresh(
        expense_approval_id(&g, school),
        cross_school_expense,
        actor,
        now,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("cross-school expense must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =========================================================================
// RealExpenseApproval::approve — EA I-1 transition + EA I-2 stamps
// =========================================================================

#[test]
fn approve_transitions_pending_to_approved() {
    // EA I-1: Pending → Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let decider = g.next_user_id();
    let now = SystemClock.now();
    row.approve(now, decider).expect("approve");
    assert!(row.is_approved());
    assert!(!row.is_pending());
    assert_eq!(row.decided_by, Some(decider)); // EA I-2
    assert_eq!(row.decided_at, Some(now)); // EA I-2
    assert_eq!(row.updated_at, now);
}

#[test]
fn approve_rejects_already_approved() {
    // EA I-1: Approved is a terminal state.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
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
    // EA I-1: terminal Rejected cannot be transitioned to Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
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
// RealExpenseApproval::reject — EA I-1 transition + EA I-2 stamps
// =========================================================================

#[test]
fn reject_transitions_pending_to_rejected_with_reason() {
    // EA I-1: Pending → Rejected.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let decider = g.next_user_id();
    let now = SystemClock.now();
    row.reject(Some("insufficient documentation".to_owned()), now, decider)
        .expect("reject");
    assert!(row.is_rejected());
    assert!(!row.is_pending());
    assert_eq!(row.decided_by, Some(decider)); // EA I-2
    assert_eq!(row.decided_at, Some(now)); // EA I-2
    assert_eq!(row.reject_reason.as_deref(), Some("insufficient documentation"));
}

#[test]
fn reject_transitions_without_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let now = SystemClock.now();
    row.reject(None, now, g.next_user_id()).expect("reject");
    assert!(row.is_rejected());
    assert!(row.reject_reason.is_none());
}

#[test]
fn reject_trims_and_drops_empty_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let now = SystemClock.now();
    row.reject(Some("  pad me  ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row.reject_reason.as_deref(), Some("pad me"));
    let mut row2 = make_expense_approval(&g, school, expense_id(&g, school));
    row2.reject(Some("   ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row2.reject_reason, None);
}

#[test]
fn reject_rejects_already_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
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
    let id = expense_approval_id(&g, school);
    let expense = expense_id(&g, school);
    let cmd = CreateExpenseApprovalCommand {
        tenant: tenant.clone(),
        expense_approval_id: id,
        expense_id: expense,
        requested_by: g.next_user_id(),
    };
    let clock = SystemClock;
    let (row, event) = create_expense_approval(cmd, &clock, &g)
        .expect("create_expense_approval should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.expense_id, expense);
    assert!(row.is_pending());
    assert_eq!(event.expense_approval_id, id);
    assert_eq!(
        <ExpenseApprovalCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.expense_approval.created"
    );
    assert_eq!(
        <ExpenseApprovalCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "expense_approval"
    );
    assert_eq!(
        <ExpenseApprovalCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn create_service_propagates_cross_school_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_approval_id(&g, school);
    let other_school = g.next_school_id();
    let cross_school_expense = expense_id(&g, other_school);
    let cmd = CreateExpenseApprovalCommand {
        tenant: tenant.clone(),
        expense_approval_id: id,
        expense_id: cross_school_expense,
        requested_by: g.next_user_id(),
    };
    let clock = SystemClock;
    let err = create_expense_approval(cmd, &clock, &g)
        .expect_err("cross-school expense must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn approve_service_transitions_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_approval_id(&g, school);
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let cmd = ApproveExpenseApprovalCommand {
        tenant: tenant.clone(),
        expense_approval_id: id,
    };
    let clock = SystemClock;
    let event = approve_expense_approval(cmd, &clock, &g, &mut row)
        .expect("approve_expense_approval should succeed");
    assert!(row.is_approved());
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(
        <ExpenseApprovalApproved as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.expense_approval.approved"
    );
    assert_eq!(
        <ExpenseApprovalApproved as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "expense_approval"
    );
    assert_eq!(
        <ExpenseApprovalApproved as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn approve_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_approval_id(&g, school);
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let clock = SystemClock;
    // First approve via service to drive into terminal state.
    approve_expense_approval(
        ApproveExpenseApprovalCommand {
            tenant: tenant.clone(),
            expense_approval_id: id,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first approve");
    let err = approve_expense_approval(
        ApproveExpenseApprovalCommand {
            tenant,
            expense_approval_id: id,
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
    let id = expense_approval_id(&g, school);
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let cmd = RejectExpenseApprovalCommand {
        tenant: tenant.clone(),
        expense_approval_id: id,
        reason: Some("duplicate request".to_owned()),
    };
    let clock = SystemClock;
    let event = reject_expense_approval(cmd, &clock, &g, &mut row)
        .expect("reject_expense_approval should succeed");
    assert!(row.is_rejected());
    assert_eq!(row.reject_reason.as_deref(), Some("duplicate request"));
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(event.reject_reason.as_deref(), Some("duplicate request"));
    assert_eq!(
        <ExpenseApprovalRejected as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.expense_approval.rejected"
    );
    assert_eq!(
        <ExpenseApprovalRejected as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "expense_approval"
    );
    assert_eq!(
        <ExpenseApprovalRejected as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn reject_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = expense_approval_id(&g, school);
    let expense = expense_id(&g, school);
    let mut row = make_expense_approval(&g, school, expense);
    let clock = SystemClock;
    reject_expense_approval(
        RejectExpenseApprovalCommand {
            tenant: tenant.clone(),
            expense_approval_id: id,
            reason: None,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first reject");
    let err = reject_expense_approval(
        RejectExpenseApprovalCommand {
            tenant,
            expense_approval_id: id,
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
