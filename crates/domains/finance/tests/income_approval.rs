//! Integration tests for the **IncomeApproval aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 80 per-aggregate drop
//! [`RealIncomeApproval`](educore_finance::aggregate::RealIncomeApproval) —
//! the approval workflow row attached to an [`Income`]. Validates
//! IA I-1 (state machine pending → approved/rejected; invalid
//! transitions return `DomainError::conflict`), IA I-2 (every
//! transition stamps `decided_by` + `decided_at` on the aggregate;
//! the reject path also captures an optional `reason`), and the
//! `create_income_approval` / `approve_income_approval` /
//! `reject_income_approval` service functions (with EVENT_TYPE /
//! AGGREGATE_TYPE / SCHEMA_VERSION pinned).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `IncomeApproval` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! IncomeApproval { _id: () } }` placeholder. Wave 80 adds the
//! `RealIncomeApproval` aggregate (state-machine on
//! `ApprovalStatus::Pending`), the 3 headline events (Created /
//! Approved / Rejected), the 3 service functions, and this test
//! suite. Structurally identical to the Wave 79
//! `tests/expense_approval.rs` suite with the parent reference
//! renamed from `expense_id` to `income_id`. The orphaned
//! `IncomeApprovalRecorded` event from the earlier Phase 7 stub is
//! preserved untouched for backwards compatibility.

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
    ApproveIncomeApprovalCommand, CreateIncomeApprovalCommand, RejectIncomeApprovalCommand,
};
use educore_finance::events::{
    IncomeApprovalApproved, IncomeApprovalCreated, IncomeApprovalRejected,
};
use educore_finance::prelude::RealIncomeApproval;
use educore_finance::services::{
    approve_income_approval, create_income_approval, reject_income_approval,
};
use educore_finance::value_objects::{IncomeApprovalId, IncomeId};

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

fn income_approval_id(g: &SystemIdGen, school: SchoolId) -> IncomeApprovalId {
    IncomeApprovalId::new(school, g.next_uuid())
}

fn income_id(g: &SystemIdGen, school: SchoolId) -> IncomeId {
    IncomeId::new(school, g.next_uuid())
}

fn make_income_approval(
    g: &SystemIdGen,
    school: SchoolId,
    income: IncomeId,
) -> RealIncomeApproval {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    RealIncomeApproval::fresh(
        income_approval_id(g, school),
        income,
        actor,
        now,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh RealIncomeApproval")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 80 stub tests)
// =========================================================================

#[test]
fn income_approval_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_approval_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn income_approval_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = income_approval_id(&g, school);
    let id_b = income_approval_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealIncomeApproval::fresh — IA I-1 (Pending-only) + IA I-2 (initial
// timestamps)
// =========================================================================

#[test]
fn fresh_starts_in_pending_state() {
    // IA I-1: a fresh approval is always in Pending.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let row = make_income_approval(&g, school, income);
    assert!(row.is_pending());
    assert!(!row.is_approved());
    assert!(!row.is_rejected());
    assert!(row.is_active());
}

#[test]
fn fresh_records_requested_by_and_requested_at() {
    // IA I-2 partial: requested_by + requested_at are stamped on
    // creation (decided_by + decided_at remain None until the
    // approval is decided).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let row = make_income_approval(&g, school, income);
    assert_eq!(row.requested_by, row.created_by);
    assert_eq!(row.requested_at, row.created_at);
    assert!(row.decided_by.is_none());
    assert!(row.decided_at.is_none());
    assert!(row.reject_reason.is_none());
}

#[test]
fn fresh_rejects_cross_school_income() {
    // IA I-1 cross-school defense-in-depth: the income must belong
    // to the same school as the approval.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let other_school = g.next_school_id();
    assert_ne!(other_school, school);
    let cross_school_income = income_id(&g, other_school);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealIncomeApproval::fresh(
        income_approval_id(&g, school),
        cross_school_income,
        actor,
        now,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("cross-school income must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =========================================================================
// RealIncomeApproval::approve — IA I-1 transition + IA I-2 stamps
// =========================================================================

#[test]
fn approve_transitions_pending_to_approved() {
    // IA I-1: Pending → Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let decider = g.next_user_id();
    let now = SystemClock.now();
    row.approve(now, decider).expect("approve");
    assert!(row.is_approved());
    assert!(!row.is_pending());
    assert_eq!(row.decided_by, Some(decider)); // IA I-2
    assert_eq!(row.decided_at, Some(now)); // IA I-2
    assert_eq!(row.updated_at, now);
}

#[test]
fn approve_rejects_already_approved() {
    // IA I-1: Approved is a terminal state.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
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
    // IA I-1: terminal Rejected cannot be transitioned to Approved.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
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
// RealIncomeApproval::reject — IA I-1 transition + IA I-2 stamps
// =========================================================================

#[test]
fn reject_transitions_pending_to_rejected_with_reason() {
    // IA I-1: Pending → Rejected.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let decider = g.next_user_id();
    let now = SystemClock.now();
    row.reject(Some("missing receipt".to_owned()), now, decider)
        .expect("reject");
    assert!(row.is_rejected());
    assert!(!row.is_pending());
    assert_eq!(row.decided_by, Some(decider)); // IA I-2
    assert_eq!(row.decided_at, Some(now)); // IA I-2
    assert_eq!(row.reject_reason.as_deref(), Some("missing receipt"));
}

#[test]
fn reject_transitions_without_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let now = SystemClock.now();
    row.reject(None, now, g.next_user_id()).expect("reject");
    assert!(row.is_rejected());
    assert!(row.reject_reason.is_none());
}

#[test]
fn reject_trims_and_drops_empty_reason() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let now = SystemClock.now();
    row.reject(Some("  pad me  ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row.reject_reason.as_deref(), Some("pad me"));
    let mut row2 = make_income_approval(&g, school, income_id(&g, school));
    row2.reject(Some("   ".to_owned()), now, g.next_user_id())
        .expect("reject");
    assert_eq!(row2.reject_reason, None);
}

#[test]
fn reject_rejects_already_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
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
    let id = income_approval_id(&g, school);
    let income = income_id(&g, school);
    let cmd = CreateIncomeApprovalCommand {
        tenant: tenant.clone(),
        income_approval_id: id,
        income_id: income,
        requested_by: g.next_user_id(),
    };
    let clock = SystemClock;
    let (row, event) = create_income_approval(cmd, &clock, &g)
        .expect("create_income_approval should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.income_id, income);
    assert!(row.is_pending());
    assert_eq!(event.income_approval_id, id);
    assert_eq!(
        <IncomeApprovalCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.income_approval.created"
    );
    assert_eq!(
        <IncomeApprovalCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "income_approval"
    );
    assert_eq!(
        <IncomeApprovalCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn create_service_propagates_cross_school_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_approval_id(&g, school);
    let other_school = g.next_school_id();
    let cross_school_income = income_id(&g, other_school);
    let cmd = CreateIncomeApprovalCommand {
        tenant: tenant.clone(),
        income_approval_id: id,
        income_id: cross_school_income,
        requested_by: g.next_user_id(),
    };
    let clock = SystemClock;
    let err = create_income_approval(cmd, &clock, &g)
        .expect_err("cross-school income must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn approve_service_transitions_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_approval_id(&g, school);
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let cmd = ApproveIncomeApprovalCommand {
        tenant: tenant.clone(),
        income_approval_id: id,
    };
    let clock = SystemClock;
    let event = approve_income_approval(cmd, &clock, &g, &mut row)
        .expect("approve_income_approval should succeed");
    assert!(row.is_approved());
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(
        <IncomeApprovalApproved as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.income_approval.approved"
    );
    assert_eq!(
        <IncomeApprovalApproved as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "income_approval"
    );
    assert_eq!(
        <IncomeApprovalApproved as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn approve_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_approval_id(&g, school);
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let clock = SystemClock;
    // First approve via service to drive into terminal state.
    approve_income_approval(
        ApproveIncomeApprovalCommand {
            tenant: tenant.clone(),
            income_approval_id: id,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first approve");
    let err = approve_income_approval(
        ApproveIncomeApprovalCommand {
            tenant,
            income_approval_id: id,
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
    let id = income_approval_id(&g, school);
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let cmd = RejectIncomeApprovalCommand {
        tenant: tenant.clone(),
        income_approval_id: id,
        reason: Some("duplicate entry".to_owned()),
    };
    let clock = SystemClock;
    let event = reject_income_approval(cmd, &clock, &g, &mut row)
        .expect("reject_income_approval should succeed");
    assert!(row.is_rejected());
    assert_eq!(row.reject_reason.as_deref(), Some("duplicate entry"));
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(event.reject_reason.as_deref(), Some("duplicate entry"));
    assert_eq!(
        <IncomeApprovalRejected as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.income_approval.rejected"
    );
    assert_eq!(
        <IncomeApprovalRejected as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "income_approval"
    );
    assert_eq!(
        <IncomeApprovalRejected as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn reject_service_rejects_terminal_state() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_approval_id(&g, school);
    let income = income_id(&g, school);
    let mut row = make_income_approval(&g, school, income);
    let clock = SystemClock;
    reject_income_approval(
        RejectIncomeApprovalCommand {
            tenant: tenant.clone(),
            income_approval_id: id,
            reason: None,
        },
        &clock,
        &g,
        &mut row,
    )
    .expect("first reject");
    let err = reject_income_approval(
        RejectIncomeApprovalCommand {
            tenant,
            income_approval_id: id,
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
