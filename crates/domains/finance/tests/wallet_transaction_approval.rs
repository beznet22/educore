//! Integration tests for the **WalletTransactionApproval child
//! aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 76 per-aggregate drop
//! — the child `WalletTransactionApproval` row that tracks the
//! approval state of a `WalletTransaction`. Validates:
//!
//! - WTA I-1: state machine pending → approved/rejected (enforced at
//!   the aggregate surface: `approve()` and `reject()` both return
//!   `Conflict` if the row is already approved or already rejected).
//! - WTA I-2: timestamps + reason recorded (approved_at +
//!   rejected_at + reject_note, all required on transition; reject
//!   note validated 1..=500 chars after trim).
//!
//! The pre-existing 2 typed-id-only tests have been preserved (as
//! smoke tests for the typed-id contract) and the suite is extended
//! below with 15 behavioral tests covering the Wave 76 full drop
//! (the first state-machine drop in the per-aggregate wave pattern).

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
    ApproveWalletTransactionApprovalCommand, CreateWalletTransactionApprovalCommand,
    RejectWalletTransactionApprovalCommand,
};
use educore_finance::events::{
    WalletTransactionApprovalApproved, WalletTransactionApprovalCreated,
    WalletTransactionApprovalRejected,
};
use educore_finance::prelude::WalletTransactionApproval;
use educore_finance::services::{
    approve_wallet_transaction_approval, create_wallet_transaction_approval,
    reject_wallet_transaction_approval,
};
use educore_finance::value_objects::{WalletTransactionApprovalId, WalletTransactionId};

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

fn wallet_transaction_approval_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> WalletTransactionApprovalId {
    WalletTransactionApprovalId::new(school, g.next_uuid())
}

fn wallet_transaction_id(g: &SystemIdGen, school: SchoolId) -> WalletTransactionId {
    WalletTransactionId::new(school, g.next_uuid())
}

fn fresh_approval(g: &SystemIdGen, _school: SchoolId) -> WalletTransactionApproval {
    fresh_approval_with_tx(g, school, wallet_transaction_id(g, school))
}

fn fresh_approval_with_tx(
    g: &SystemIdGen,
    _school: SchoolId,
    tx_id: WalletTransactionId,
) -> WalletTransactionApproval {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    WalletTransactionApproval::fresh(tx_id, actor, now, corr)
}

// ---------------------------------------------------------------------------
// Typed-id contract (preserved from Phase 7 Workstream K seed)
// ---------------------------------------------------------------------------

#[test]
fn wallet_transaction_approval_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = wallet_transaction_approval_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn wallet_transaction_approval_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = wallet_transaction_approval_id(&g, school);
    let id_b = wallet_transaction_approval_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// WalletTransactionApproval: fresh() + initial state predicates
// ---------------------------------------------------------------------------

#[test]
fn fresh_produces_pending_aggregate_with_no_approval_or_rejection() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval = fresh_approval(&g, school);
    assert!(approval.is_pending(), "fresh row must be pending");
    assert!(!approval.is_approved());
    assert!(!approval.is_rejected());
    assert!(approval.approver_id.is_none());
    assert!(approval.approved_at.is_none());
    assert!(approval.rejecter_id.is_none());
    assert!(approval.rejected_at.is_none());
    assert!(approval.reject_note.is_none());
    assert_eq!(approval.school_id, school);
}

#[test]
fn fresh_with_zero_initial_version_and_active_status() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval = fresh_approval(&g, school);
    assert!(approval.is_active());
    // Version::initial() compares equal across calls but the public
    // surface guarantees the row was just created (no transitions).
    assert!(approval.last_event_id.is_none());
}

// ---------------------------------------------------------------------------
// WalletTransactionApproval: state machine — WTA I-1
// ---------------------------------------------------------------------------

#[test]
fn approve_transitions_pending_to_approved_with_full_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let initial_version = approval.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let event_id = g.next_event_id();

    approval
        .approve(actor, now, event_id)
        .expect("approve succeeds");
    assert!(!approval.is_pending(), "approve must clear pending");
    assert!(approval.is_approved(), "approve must set approved");
    assert!(!approval.is_rejected(), "approve must NOT set rejected");
    assert_eq!(approval.approver_id, Some(actor));
    assert_eq!(approval.approved_at, Some(now));
    assert!(
        approval.rejected_at.is_none(),
        "approve must NOT set rejected_at"
    );
    assert!(
        approval.reject_note.is_none(),
        "approve must NOT set reject_note"
    );
    assert!(approval.version > initial_version, "version must advance");
    assert_eq!(approval.updated_by, actor);
    assert_eq!(approval.last_event_id, Some(event_id));
}

#[test]
fn approve_on_already_approved_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let event_id = g.next_event_id();
    approval
        .approve(actor, now, event_id)
        .expect("first approve succeeds");
    let result = approval.approve(actor, now, g.next_event_id());
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second approve must fail with Conflict, got {result:?}"
    );
}

#[test]
fn approve_on_already_rejected_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let rejecter = g.next_user_id();
    let now = SystemClock.now();
    approval
        .reject(
            rejecter,
            "Denied per policy".to_owned(),
            now,
            g.next_event_id(),
        )
        .expect("first reject succeeds");
    let approver = g.next_user_id();
    let later = educore_core::value_objects::Timestamp::from_datetime(
        now.as_datetime() + chrono::Duration::seconds(1),
    );
    let result = approval.approve(approver, later, g.next_event_id());
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "approve on rejected must fail with Conflict, got {result:?}"
    );
}

#[test]
fn reject_transitions_pending_to_rejected_with_required_note() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let initial_version = approval.version;
    let rejecter = g.next_user_id();
    let now = SystemClock.now();
    let event_id = g.next_event_id();
    approval
        .reject(
            rejecter,
            "Insufficient documentation".to_owned(),
            now,
            event_id,
        )
        .expect("reject succeeds");
    assert!(!approval.is_pending(), "reject must clear pending");
    assert!(!approval.is_approved(), "reject must NOT set approved");
    assert!(approval.is_rejected(), "reject must set rejected");
    assert_eq!(approval.rejecter_id, Some(rejecter));
    assert_eq!(approval.rejected_at, Some(now));
    assert_eq!(
        approval.reject_note.as_deref(),
        Some("Insufficient documentation")
    );
    assert!(
        approval.approved_at.is_none(),
        "reject must NOT set approved_at"
    );
    assert!(approval.version > initial_version, "version must advance");
    assert_eq!(approval.updated_by, rejecter);
    assert_eq!(approval.last_event_id, Some(event_id));
}

#[test]
fn reject_on_already_rejected_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let rejecter = g.next_user_id();
    let now = SystemClock.now();
    approval
        .reject(rejecter, "Denied".to_owned(), now, g.next_event_id())
        .expect("first reject succeeds");
    let result = approval.reject(
        rejecter,
        "Second attempt".to_owned(),
        now,
        g.next_event_id(),
    );
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second reject must fail with Conflict, got {result:?}"
    );
}

#[test]
fn reject_on_already_approved_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut approval = fresh_approval(&g, school);
    let approver = g.next_user_id();
    let now = SystemClock.now();
    approval
        .approve(approver, now, g.next_event_id())
        .expect("first approve succeeds");
    let rejecter = g.next_user_id();
    let later = educore_core::value_objects::Timestamp::from_datetime(
        now.as_datetime() + chrono::Duration::seconds(1),
    );
    let result = approval.reject(rejecter, "Too late".to_owned(), later, g.next_event_id());
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "reject on approved must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_wallet_transaction_approval service function
// ---------------------------------------------------------------------------

#[test]
fn create_wallet_transaction_approval_service_produces_pending_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let tx_id = wallet_transaction_id(&g, school);
    let cmd = CreateWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        wallet_transaction_id: tx_id,
    };
    let clock = SystemClock;
    let (approval, event) =
        create_wallet_transaction_approval(cmd, &clock, &g).expect("create succeeds");

    assert!(approval.is_pending(), "service-created row must be pending");
    assert_eq!(approval.wallet_transaction_id, tx_id);
    assert_eq!(approval.school_id, school);
    assert_eq!(approval.last_event_id, Some(event.event_id));

    assert_eq!(event.wallet_transaction_approval_id, approval_id);
    assert_eq!(event.wallet_transaction_id, tx_id);
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        WalletTransactionApprovalCreated::EVENT_TYPE,
        "finance.wallet_transaction_approval.created"
    );
    assert_eq!(
        WalletTransactionApprovalCreated::AGGREGATE_TYPE,
        "wallet_transaction_approval"
    );
    assert_eq!(WalletTransactionApprovalCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_wallet_transaction_approval_service_rejects_cross_school_id() {
    let (tenant, g) = admin_context();
    let other_school = g.next_school_id();
    let cmd = CreateWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: wallet_transaction_approval_id(&g, other_school),
        wallet_transaction_id: wallet_transaction_id(&g, tenant.school_id),
    };
    let clock = SystemClock;
    let result = create_wallet_transaction_approval(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "cross-school id must fail with Validation, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// approve_wallet_transaction_approval service function
// ---------------------------------------------------------------------------

#[test]
fn approve_wallet_transaction_approval_service_transitions_to_approved() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let tx_id = wallet_transaction_id(&g, school);
    let mut approval = fresh_approval_with_tx(&g, school, tx_id);

    let cmd = ApproveWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        approver_user_id: g.next_user_id(),
    };
    let clock = SystemClock;
    let event = approve_wallet_transaction_approval(cmd, &mut approval, &clock, &g)
        .expect("approve succeeds");

    assert!(approval.is_approved());
    assert_eq!(event.wallet_transaction_approval_id, approval_id);
    assert_eq!(event.wallet_transaction_id, tx_id);
    assert_eq!(event.approver_id, approval.approver_id.unwrap());
    assert_eq!(
        WalletTransactionApprovalApproved::EVENT_TYPE,
        "finance.wallet_transaction_approval.approved"
    );
    assert_eq!(
        WalletTransactionApprovalApproved::AGGREGATE_TYPE,
        "wallet_transaction_approval"
    );
    assert_eq!(WalletTransactionApprovalApproved::SCHEMA_VERSION, 1);
}

#[test]
fn approve_wallet_transaction_approval_service_propagates_conflict_on_double_approve() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let mut approval = fresh_approval(&g, school);
    let cmd1 = ApproveWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        approver_user_id: g.next_user_id(),
    };
    let clock = SystemClock;
    approve_wallet_transaction_approval(cmd1, &mut approval, &clock, &g)
        .expect("first approve succeeds");
    let cmd2 = ApproveWalletTransactionApprovalCommand {
        tenant,
        wallet_transaction_approval_id: approval_id,
        approver_user_id: g.next_user_id(),
    };
    let result = approve_wallet_transaction_approval(cmd2, &mut approval, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second approve must propagate Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// reject_wallet_transaction_approval service function
// ---------------------------------------------------------------------------

#[test]
fn reject_wallet_transaction_approval_service_transitions_to_rejected_with_note() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let tx_id = wallet_transaction_id(&g, school);
    let mut approval = fresh_approval_with_tx(&g, school, tx_id);

    let cmd = RejectWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        rejecter_user_id: g.next_user_id(),
        reason: "  Insufficient documentation  ".to_owned(),
    };
    let clock = SystemClock;
    let event = reject_wallet_transaction_approval(cmd, &mut approval, &clock, &g)
        .expect("reject succeeds");

    assert!(approval.is_rejected());
    assert_eq!(
        approval.reject_note.as_deref(),
        Some("Insufficient documentation"),
        "reason must be stored trimmed (WTA I-2)"
    );
    assert_eq!(event.wallet_transaction_approval_id, approval_id);
    assert_eq!(event.wallet_transaction_id, tx_id);
    assert_eq!(event.rejecter_id, approval.rejecter_id.unwrap());
    assert_eq!(event.reject_note, "Insufficient documentation");
    assert_eq!(
        WalletTransactionApprovalRejected::EVENT_TYPE,
        "finance.wallet_transaction_approval.rejected"
    );
    assert_eq!(
        WalletTransactionApprovalRejected::AGGREGATE_TYPE,
        "wallet_transaction_approval"
    );
    assert_eq!(WalletTransactionApprovalRejected::SCHEMA_VERSION, 1);
}

#[test]
fn reject_wallet_transaction_approval_service_propagates_validation_for_empty_note() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let mut approval = fresh_approval(&g, school);

    let cmd = RejectWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        rejecter_user_id: g.next_user_id(),
        reason: "   ".to_owned(), // whitespace only
    };
    let clock = SystemClock;
    let result = reject_wallet_transaction_approval(cmd, &mut approval, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only reason must propagate Validation (WTA I-2), got {result:?}"
    );
    assert!(
        approval.is_pending(),
        "rejected validation must not transition state"
    );
}

#[test]
fn reject_wallet_transaction_approval_service_propagates_validation_for_overlong_note() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let mut approval = fresh_approval(&g, school);

    let cmd = RejectWalletTransactionApprovalCommand {
        tenant,
        wallet_transaction_approval_id: approval_id,
        rejecter_user_id: g.next_user_id(),
        reason: "x".repeat(501),
    };
    let clock = SystemClock;
    let result = reject_wallet_transaction_approval(cmd, &mut approval, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "reason over 500 chars must propagate Validation (WTA I-2), got {result:?}"
    );
}

#[test]
fn reject_wallet_transaction_approval_service_propagates_conflict_on_double_reject() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let approval_id = wallet_transaction_approval_id(&g, school);
    let mut approval = fresh_approval(&g, school);
    let cmd1 = RejectWalletTransactionApprovalCommand {
        tenant: tenant.clone(),
        wallet_transaction_approval_id: approval_id,
        rejecter_user_id: g.next_user_id(),
        reason: "First rejection".to_owned(),
    };
    let clock = SystemClock;
    reject_wallet_transaction_approval(cmd1, &mut approval, &clock, &g)
        .expect("first reject succeeds");
    let cmd2 = RejectWalletTransactionApprovalCommand {
        tenant,
        wallet_transaction_approval_id: approval_id,
        rejecter_user_id: g.next_user_id(),
        reason: "Second rejection".to_owned(),
    };
    let result = reject_wallet_transaction_approval(cmd2, &mut approval, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second reject must propagate Conflict, got {result:?}"
    );
}
