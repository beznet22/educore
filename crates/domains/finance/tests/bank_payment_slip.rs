//! Behavioural tests for `RealBankPaymentSlip` (Wave 130 full drop).
//!
//! Pins BP I-1 (payment_mode ∈ {Bank, Cheque}) + BP I-2
//! (Pending -> Approved | Rejected state machine) + BP I-4
//! (cannot reject after approval) + companion invariants
//! (amount_minor >= 0 + payer_name non-empty after trim) end-to-end
//! via the aggregate surface, the service functions, and the
//! emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::str::FromStr;

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{Timestamp, Version};
use educore_events::domain_event::DomainEvent;
use educore_finance::events::{
    BankPaymentSlipApproved, BankPaymentSlipCreated, BankPaymentSlipRejected,
    BankPaymentSlipRetired,
};
use educore_finance::prelude::*;
use educore_finance::value_objects::ApprovalStatus;
use educore_finance::value_objects::{BankAccountId, BankPaymentSlipId, PaymentMode};

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

fn bps_id(g: &SystemIdGen, school: SchoolId) -> BankPaymentSlipId {
    BankPaymentSlipId::new(school, g.next_uuid())
}

fn bank_id(g: &SystemIdGen, school: SchoolId) -> BankAccountId {
    BankAccountId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn bank_payment_slip_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = bps_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- BP I-1: PaymentMode enum ----

#[test]
fn payment_mode_as_str_round_trip() {
    assert_eq!(PaymentMode::Bank.as_str(), "bank");
    assert_eq!(PaymentMode::Cheque.as_str(), "cheque");
    assert_eq!(PaymentMode::parse("bank"), Some(PaymentMode::Bank));
    assert_eq!(PaymentMode::parse("bk"), Some(PaymentMode::Bank));
    assert_eq!(PaymentMode::parse("cheque"), Some(PaymentMode::Cheque));
    assert_eq!(PaymentMode::parse("cq"), Some(PaymentMode::Cheque));
    assert_eq!(PaymentMode::parse("unknown"), None);
}

#[test]
fn payment_mode_display_matches_as_str() {
    assert_eq!(PaymentMode::Bank.to_string(), "bank");
    assert_eq!(PaymentMode::Cheque.to_string(), "cheque");
}

#[test]
fn payment_mode_from_str_unknown_returns_err() {
    let result = PaymentMode::from_str("unknown");
    assert!(result.is_err());
}

// ---- BP I-1 + companion invariants happy path ----

#[test]
fn fresh_bank_mode_valid_bp_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let account = bank_id(&g, school);
    let row = RealBankPaymentSlip::fresh(
        id,
        10_000,
        PaymentMode::Bank,
        account,
        "John Doe".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.payment_mode, PaymentMode::Bank);
    assert_eq!(row.amount_minor, 10_000);
    assert_eq!(row.bank_account_id, account);
    assert_eq!(row.payer_name, "John Doe");
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_cheque_mode_valid_bp_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let account = bank_id(&g, school);
    let row = RealBankPaymentSlip::fresh(
        id,
        5_000,
        PaymentMode::Cheque,
        account,
        "Jane Doe".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("Cheque mode is valid (BP I-1)");
    assert_eq!(row.payment_mode, PaymentMode::Cheque);
}

// ---- companion invariant: amount_minor >= 0 ----

#[test]
fn fresh_negative_amount_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let result = RealBankPaymentSlip::fresh(
        id,
        -1,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_zero_amount_boundary_valid() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let row = RealBankPaymentSlip::fresh(
        id,
        0,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero amount_minor is valid boundary");
    assert_eq!(row.amount_minor, 0);
}

// ---- companion invariant: payer_name non-empty after trim ----

#[test]
fn fresh_empty_payer_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let result = RealBankPaymentSlip::fresh(
        id,
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "   ".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_payer_name_trimmed_correctly() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let row = RealBankPaymentSlip::fresh(
        id,
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "  John Doe  ".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("payer_name with whitespace is trimmed");
    assert_eq!(row.payer_name, "John Doe");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bps_id(&g, school);
    let row = RealBankPaymentSlip::fresh(
        id,
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.last_event_id, None);
}

// ---- BP I-2: state machine ----

#[test]
fn fresh_initial_status_is_pending_bp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.status, ApprovalStatus::Pending);
    assert_eq!(row.approved_by, None);
    assert_eq!(row.approved_at, None);
    assert_eq!(row.rejected_by, None);
    assert_eq!(row.rejected_at, None);
    assert_eq!(row.reject_note, None);
}

#[test]
fn approve_transitions_pending_to_approved_bp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    let event_id = g.next_event_id();
    let at = Timestamp::now();
    row.approve(tenant.actor_id, at, event_id)
        .expect("approve should succeed");
    assert_eq!(row.status, ApprovalStatus::Approved);
    assert_eq!(row.approved_by, Some(tenant.actor_id));
    assert_eq!(row.approved_at, Some(at));
    assert_eq!(row.last_event_id, Some(event_id));
}

#[test]
fn reject_transitions_pending_to_rejected_bp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    let event_id = g.next_event_id();
    let at = Timestamp::now();
    let note = "Insufficient funds".to_string();
    row.reject(tenant.actor_id, note.clone(), at, event_id)
        .expect("reject should succeed");
    assert_eq!(row.status, ApprovalStatus::Rejected);
    assert_eq!(row.rejected_by, Some(tenant.actor_id));
    assert_eq!(row.rejected_at, Some(at));
    assert_eq!(row.reject_note, Some(note));
    assert_eq!(row.last_event_id, Some(event_id));
}

// ---- BP I-4: cannot reject after approval ----

#[test]
fn double_approve_returns_conflict_bp_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id())
        .expect("first approve should succeed");
    let result = row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn reject_after_approve_returns_conflict_bp_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id())
        .expect("first approve should succeed");
    let result = row.reject(
        tenant.actor_id,
        "too late".to_string(),
        Timestamp::now(),
        g.next_event_id(),
    );
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("first retire should succeed");
    let result = row.retire(Timestamp::now(), tenant.actor_id);
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- service integration ----

#[test]
fn create_bank_payment_slip_service_emits_created_event_bp_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = bps_id(&g, school);
    let account = bank_id(&g, school);
    let cmd = CreateBankPaymentSlipCommand {
        tenant,
        bank_payment_slip_id: id,
        amount_minor: 10_000,
        payment_mode: PaymentMode::Bank,
        bank_account_id: account,
        payer_name: "John Doe".to_string(),
    };
    let (_agg, evt): (RealBankPaymentSlip, BankPaymentSlipCreated) =
        create_bank_payment_slip(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.payment_mode, PaymentMode::Bank);
    assert_eq!(evt.amount_minor, 10_000);
    assert_eq!(
        <BankPaymentSlipCreated as DomainEvent>::EVENT_TYPE,
        "finance.bank_payment_slip.created"
    );
    assert_eq!(
        <BankPaymentSlipCreated as DomainEvent>::AGGREGATE_TYPE,
        "bank_payment_slip"
    );
}

#[test]
fn approve_bank_payment_slip_service_emits_approved_event_bp_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let agg = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        actor,
        Timestamp::now(),
        corr,
    )
    .expect("fresh should succeed");
    let id = agg.id;
    let cmd = ApproveBankPaymentSlipCommand {
        tenant,
        bank_payment_slip_id: id,
    };
    let (updated, evt): (RealBankPaymentSlip, BankPaymentSlipApproved) =
        approve_bank_payment_slip(agg, cmd, &clock, &g).expect("approve service should succeed");
    assert_eq!(updated.status, ApprovalStatus::Approved);
    assert_eq!(evt.approved_by, actor);
    assert_eq!(evt.status, ApprovalStatus::Approved);
    assert_eq!(
        <BankPaymentSlipApproved as DomainEvent>::EVENT_TYPE,
        "finance.bank_payment_slip.approved"
    );
}

#[test]
fn reject_bank_payment_slip_service_emits_rejected_event_bp_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let agg = RealBankPaymentSlip::fresh(
        bps_id(&g, school),
        5_000,
        PaymentMode::Bank,
        bank_id(&g, school),
        "Test".to_string(),
        actor,
        Timestamp::now(),
        corr,
    )
    .expect("fresh should succeed");
    let id = agg.id;
    let cmd = RejectBankPaymentSlipCommand {
        tenant,
        bank_payment_slip_id: id,
        reject_note: "Wrong amount".to_string(),
    };
    let (updated, evt): (RealBankPaymentSlip, BankPaymentSlipRejected) =
        reject_bank_payment_slip(agg, cmd, &clock, &g).expect("reject service should succeed");
    assert_eq!(updated.status, ApprovalStatus::Rejected);
    assert_eq!(evt.rejected_by, actor);
    assert_eq!(evt.reject_note, "Wrong amount");
    assert_eq!(
        <BankPaymentSlipRejected as DomainEvent>::EVENT_TYPE,
        "finance.bank_payment_slip.rejected"
    );
}

#[test]
fn retire_bank_payment_slip_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireBankPaymentSlipCommand {
        tenant,
        bank_payment_slip_id: bps_id(&g, school),
    };
    let evt: BankPaymentSlipRetired =
        retire_bank_payment_slip(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <BankPaymentSlipRetired as DomainEvent>::EVENT_TYPE,
        "finance.bank_payment_slip.retired"
    );
}

#[test]
fn read_bank_payment_slip_service_returns_ok() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = ReadBankPaymentSlipCommand {
        tenant,
        bank_payment_slip_id: bps_id(&g, school),
    };
    read_bank_payment_slip(cmd, &clock, &g).expect("read should succeed");
}
