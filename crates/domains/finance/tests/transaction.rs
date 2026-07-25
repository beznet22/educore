//! Integration tests for the **Transaction aggregate** vertical slice.
//!
//! Pins the TR I-1 invariant end-to-end: a Transaction is a
//! double-entry journal line whose `total_debits_minor` must equal
//! `total_credits_minor` (the corner-stone double-entry balancing
//! invariant). Both totals must be `>= 0`. Companion invariant:
//! `description` must be non-empty after trimming whitespace.
//!
//! Replaces the prior 2 typed-id-only tests with a 13-test
//! behavioral suite that exercises construction, validation,
//! audit-footer, retire, and service integration paths.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_transaction, retire_transaction, Currency, RealTransaction, TransactionCreated,
    TransactionRetired, TransactionId, FINANCE_TRANSACTION_CREATE_COMMAND_TYPE,
    FINANCE_TRANSACTION_RETIRE_COMMAND_TYPE,
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

fn transaction_id(g: &SystemIdGen, school: SchoolId) -> TransactionId {
    TransactionId::new(school, g.next_uuid())
}

fn balanced_payload() -> (i64, i64) {
    // 5000 + 7500 debits == 12_500 credits (balanced journal line).
    (12_500, 12_500)
}

#[test]
fn transaction_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = transaction_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn transaction_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = transaction_id(&g, school);
    let id_b = transaction_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

#[test]
fn fresh_full_payload_balanced_debits_equal_credits_tr_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = transaction_id(&g, school);
    let (debits, credits) = balanced_payload();
    let tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Tuition payment — Q3 FY26".to_owned(),
        Some("INV-2026-0042".to_owned()),
        debits,
        credits,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("TR I-1: balanced journal entry must construct");
    assert!(tx.is_active());
    assert!(tx.is_balanced());
    assert_eq!(tx.total_debits_minor, 12_500);
    assert_eq!(tx.total_credits_minor, 12_500);
    assert_eq!(tx.description, "Tuition payment — Q3 FY26");
    assert_eq!(tx.reference.as_deref(), Some("INV-2026-0042"));
    assert_eq!(tx.school_id, school);
}

#[test]
fn fresh_negative_debits_validation_error_tr_i_1() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let err = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Negative debit line".to_owned(),
        None,
        -1,
        0,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("TR I-1: negative total_debits_minor must be rejected");
    assert!(
        format!("{err}").contains("total_debits_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_credits_validation_error_tr_i_1() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let err = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Negative credit line".to_owned(),
        None,
        0,
        -1,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("TR I-1: negative total_credits_minor must be rejected");
    assert!(
        format!("{err}").contains("total_credits_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_unbalanced_credits_greater_validation_error_tr_i_1() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let err = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Credits exceed debits".to_owned(),
        None,
        5_000,
        7_500,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("TR I-1: credits > debits is unbalanced and must be rejected");
    assert!(
        format!("{err}").contains("total_debits_minor must equal total_credits_minor"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_unbalanced_debits_greater_validation_error_tr_i_1() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let err = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Debits exceed credits".to_owned(),
        None,
        7_500,
        5_000,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("TR I-1: debits > credits is unbalanced and must be rejected");
    assert!(
        format!("{err}").contains("total_debits_minor must equal total_credits_minor"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_zero_debits_and_credits_boundary_valid_tr_i_1() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Zero-amount balanced entry".to_owned(),
        None,
        0,
        0,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("TR I-1: zero+zero is balanced and must construct (boundary)");
    assert!(tx.is_balanced());
    assert_eq!(tx.total_debits_minor, 0);
    assert_eq!(tx.total_credits_minor, 0);
}

#[test]
fn fresh_empty_description_validation_error_companion() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let err = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "   \t  ".to_owned(),
        None,
        0,
        0,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("companion: whitespace-only description must be rejected");
    assert!(
        format!("{err}").contains("description must be non-empty after trimming"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Audit footer check".to_owned(),
        None,
        1_000,
        1_000,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("balanced journal entry");
    assert!(tx.last_event_id.is_none(), "fresh() must start with no last_event_id");
    assert_eq!(tx.created_by, tenant.actor_id);
    assert_eq!(tx.updated_by, tenant.actor_id);
    assert_eq!(tx.created_at, now);
    assert_eq!(tx.updated_at, now);
    assert_eq!(tx.correlation_id, tenant.correlation_id);
}

#[test]
fn is_balanced_helper_method_tracks_equality() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Balanced helper".to_owned(),
        None,
        2_500,
        2_500,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("balanced entry");
    assert!(tx.is_balanced());
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let mut tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Will be retired".to_owned(),
        None,
        1_000,
        1_000,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("balanced entry");
    assert!(tx.is_active());
    tx.retire(now, tenant.actor_id).expect("retire");
    assert!(!tx.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let id = transaction_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let mut tx = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        "Double-retire attempt".to_owned(),
        None,
        1_000,
        1_000,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("balanced entry");
    tx.retire(now, tenant.actor_id).expect("first retire");
    let err = tx.retire(now, tenant.actor_id).expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_transaction_service_emits_created_event_with_tr_i_1_payload() {
    use educore_finance::commands::CreateTransactionCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = transaction_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let (debits, credits) = balanced_payload();
    let cmd = CreateTransactionCommand {
        tenant: tenant.clone(),
        transaction_id: id,
        transaction_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        description: "Service integration — Q3 FY26".to_owned(),
        reference: Some("SVC-INV-001".to_owned()),
        total_debits_minor: debits,
        total_credits_minor: credits,
        currency: Currency::INR,
    };
    let (tx, event): (RealTransaction, TransactionCreated) =
        create_transaction(cmd, &clock, &ids).expect("create_transaction must succeed");
    assert!(tx.is_active());
    assert!(tx.is_balanced());
    assert_eq!(tx.total_debits_minor, 12_500);
    assert_eq!(tx.total_credits_minor, 12_500);
    assert_eq!(event.transaction_id, tx.id);
    assert_eq!(event.total_debits_minor, 12_500);
    assert_eq!(event.total_credits_minor, 12_500);
    assert_eq!(event.description, "Service integration — Q3 FY26");
    assert_eq!(event.reference.as_deref(), Some("SVC-INV-001"));
    assert_eq!(<TransactionCreated as DomainEvent>::EVENT_TYPE, "finance.transaction.created");
    assert_eq!(<TransactionCreated as DomainEvent>::AGGREGATE_TYPE, "transaction");
    assert_eq!(<TransactionCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

#[test]
fn retire_transaction_service_emits_retired_event_tr_i_1() {
    use educore_finance::commands::RetireTransactionCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = transaction_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireTransactionCommand {
        tenant: tenant.clone(),
        transaction_id: id,
    };
    let (tx, event): (RealTransaction, TransactionRetired) =
        retire_transaction(cmd, &clock, &ids).expect("retire_transaction must succeed");
    assert!(!tx.is_active());
    assert_eq!(event.transaction_id, tx.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(<TransactionRetired as DomainEvent>::EVENT_TYPE, "finance.transaction.retired");
    assert_eq!(<TransactionRetired as DomainEvent>::AGGREGATE_TYPE, "transaction");
    assert_eq!(<TransactionRetired as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}
