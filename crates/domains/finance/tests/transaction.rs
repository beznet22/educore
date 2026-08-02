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
use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_transaction, retire_transaction, Currency, RealTransaction, TransactionCreated,
    TransactionId, TransactionRetired, FINANCE_TRANSACTION_CREATE_COMMAND_TYPE,
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
    assert!(
        tx.last_event_id.is_none(),
        "fresh() must start with no last_event_id"
    );
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
    let err = tx
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(
        format!("{err}").contains("already retired"),
        "unexpected error: {err}"
    );
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
    assert_eq!(
        <TransactionCreated as DomainEvent>::EVENT_TYPE,
        "finance.transaction.created"
    );
    assert_eq!(
        <TransactionCreated as DomainEvent>::AGGREGATE_TYPE,
        "transaction"
    );
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
    assert_eq!(
        <TransactionRetired as DomainEvent>::EVENT_TYPE,
        "finance.transaction.retired"
    );
    assert_eq!(
        <TransactionRetired as DomainEvent>::AGGREGATE_TYPE,
        "transaction"
    );
    assert_eq!(<TransactionRetired as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

// =========================================================================
// -- Wave 136 -- RealTransaction -- TR I-2 append-only enforcement --
// =========================================================================

#[test]
fn append_only_no_update_mutator_exists_tr_i_2() {
    // TR I-2 marker test: RealTransaction intentionally exposes
    // no `update_*` method (compile-time assertion documented in
    // the impl block). The only mutators are `fresh()`, `retire()`,
    // and (in Wave 136) the state-machine mutators `post()` +
    // `reverse()`. State transitions DO update lifecycle_status
    // but they DO NOT mutate payload fields (description,
    // reference, total_debits_minor, total_credits_minor, currency,
    // transaction_date are all preserved in the audit footer for
    // legal-record retention).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = transaction_id(&g, school);
    let row = RealTransaction::fresh(
        id,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        "TR I-2 marker test transaction".to_owned(),
        None,
        5_000,
        5_000,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    // The only payload mutators are state-machine transitions
    // (post, reverse) + retire (tombstone). No update_* methods.
    let _ = row; // type-level marker
}

// =========================================================================
// -- Wave 136 -- RealTransaction -- TR I-3 state machine extension tests --
// =========================================================================

use educore_core::error::DomainError;
use educore_finance::commands::PostTransactionCommand;
use educore_finance::events::TransactionPosted;
use educore_finance::services::post_transaction;
use educore_finance::value_objects::TransactionLifecycleStatus;

fn build_tx(actor: UserId) -> RealTransaction {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    RealTransaction::fresh(
        transaction_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        "Test transaction".to_owned(),
        None,
        5_000,
        5_000,
        Currency::INR,
        actor,
        educore_core::value_objects::Timestamp::now(),
        _tenant.correlation_id,
    )
    .expect("fresh should succeed")
}

// ---- TransactionLifecycleStatus enum round-trip ----

#[test]
fn transaction_lifecycle_status_as_str_round_trip_tr_i_3() {
    assert_eq!(TransactionLifecycleStatus::Draft.as_str(), "draft");
    assert_eq!(TransactionLifecycleStatus::Posted.as_str(), "posted");
    assert_eq!(
        TransactionLifecycleStatus::parse("draft"),
        Some(TransactionLifecycleStatus::Draft)
    );
    assert_eq!(
        TransactionLifecycleStatus::parse("posted"),
        Some(TransactionLifecycleStatus::Posted)
    );
    assert_eq!(TransactionLifecycleStatus::parse("unknown"), None);
}

#[test]
fn transaction_lifecycle_status_can_transition_only_draft_to_posted_tr_i_3() {
    assert!(TransactionLifecycleStatus::Draft.can_transition_to(TransactionLifecycleStatus::Posted));
    assert!(
        !TransactionLifecycleStatus::Posted.can_transition_to(TransactionLifecycleStatus::Draft)
    );
    assert!(
        !TransactionLifecycleStatus::Posted.can_transition_to(TransactionLifecycleStatus::Posted)
    );
}

// ---- fresh initializes lifecycle ----

#[test]
fn fresh_initializes_lifecycle_draft_tr_i_3() {
    let (tenant, _g) = admin_context();
    let tx = build_tx(tenant.actor_id);
    assert_eq!(tx.lifecycle_status, TransactionLifecycleStatus::Draft);
    assert_eq!(tx.posted_by, None);
    assert_eq!(tx.posted_at, None);
}

// ---- post mutator ----

#[test]
fn post_transitions_draft_to_posted_tr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut tx = build_tx(actor);
    let at = educore_core::value_objects::Timestamp::now();
    let event_id = g.next_event_id();
    tx.post(actor, at, event_id).expect("post should succeed");
    assert_eq!(tx.lifecycle_status, TransactionLifecycleStatus::Posted);
    assert_eq!(tx.posted_by, Some(actor));
    assert_eq!(tx.posted_at, Some(at));
    assert_eq!(tx.last_event_id, Some(event_id));
}

#[test]
fn double_post_returns_conflict_tr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut tx = build_tx(actor);
    let at = educore_core::value_objects::Timestamp::now();
    tx.post(actor, at, g.next_event_id()).expect("first post");
    let result = tx.post(actor, at, g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- payload preservation after post (TR I-2 + I-3 interplay) ----

#[test]
fn post_preserves_all_payload_fields_tr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut tx = RealTransaction::fresh(
        transaction_id(&g, tenant.school_id),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        "Preservation test".to_owned(),
        Some("REF-001".to_owned()),
        12_500,
        12_500,
        Currency::INR,
        actor,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let at = educore_core::value_objects::Timestamp::now();
    let event_id = g.next_event_id();
    tx.post(actor, at, event_id).expect("post");
    // TR I-2: payload fields preserved after state-machine
    // transition (no update_* mutator means the post transition
    // cannot mutate any payload).
    assert_eq!(
        tx.transaction_date,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
    );
    assert_eq!(tx.description, "Preservation test");
    assert_eq!(tx.reference.as_deref(), Some("REF-001"));
    assert_eq!(tx.total_debits_minor, 12_500);
    assert_eq!(tx.total_credits_minor, 12_500);
    assert_eq!(tx.currency, Currency::INR);
    assert!(tx.is_balanced());
}

// ---- retire after post ----

#[test]
fn retire_after_post_succeeds_tr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut tx = build_tx(actor);
    tx.post(
        actor,
        educore_core::value_objects::Timestamp::now(),
        g.next_event_id(),
    )
    .expect("post");
    tx.retire(educore_core::value_objects::Timestamp::now(), actor)
        .expect("retire after post should succeed");
    assert!(!tx.is_active());
    assert_eq!(tx.lifecycle_status, TransactionLifecycleStatus::Posted);
}

// ---- service integration ----

#[test]
fn post_service_emits_event_tr_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealTransaction::fresh(
        transaction_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        "Service integration test".to_owned(),
        None,
        5_000,
        5_000,
        Currency::INR,
        actor,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = PostTransactionCommand {
        tenant,
        transaction_id: id,
    };
    let (updated, evt): (RealTransaction, TransactionPosted) =
        post_transaction(agg, cmd, &clock, &g).expect("service should succeed");
    assert_eq!(updated.lifecycle_status, TransactionLifecycleStatus::Posted);
    assert_eq!(evt.lifecycle_status, TransactionLifecycleStatus::Posted);
    assert_eq!(evt.posted_by, actor);
    assert_eq!(
        <TransactionPosted as DomainEvent>::EVENT_TYPE,
        "finance.transaction.posted"
    );
    assert_eq!(
        <TransactionPosted as DomainEvent>::AGGREGATE_TYPE,
        "transaction"
    );
}
