//! Behavioural tests for `RealFmFeesTransaction` (Wave 124 full drop).
//!
//! Pins FFT I-2 (`total_paid_amount_minor >= 0`) end-to-end via the
//! aggregate surface, the service functions, and the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{Timestamp, Version};
use educore_events::domain_event::DomainEvent;
use educore_finance::events::{FmFeesTransactionCreated, FmFeesTransactionRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::FmFeesTransactionId;

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

fn fft_id(g: &SystemIdGen, school: SchoolId) -> FmFeesTransactionId {
    FmFeesTransactionId::new(school, g.next_uuid())
}

fn txn_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 6, 13).expect("valid date")
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_transaction_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fft_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFT I-2: total_paid_amount_minor >= 0 ----

#[test]
fn fresh_full_payload_total_paid_amount_valid_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesTransaction::fresh(
        id,
        10_000,
        txn_date(),
        Some("Q2 fees batch".to_string()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with total_paid_amount_minor = 10_000");
    assert_eq!(row.total_paid_amount_minor, 10_000);
    assert_eq!(row.transaction_date, txn_date());
    assert_eq!(row.description.as_deref(), Some("Q2 fees batch"));
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_total_paid_amount_validation_error_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let result = RealFmFeesTransaction::fresh(
        id,
        -1,
        txn_date(),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("total_paid_amount_minor must be >= 0")
                    && msg.contains("FFT I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_total_paid_amount_boundary_valid_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesTransaction::fresh(
        id,
        0,
        txn_date(),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero total_paid_amount_minor is valid boundary");
    assert_eq!(row.total_paid_amount_minor, 0);
    assert!(row.is_active());
}

#[test]
fn fresh_large_total_paid_amount_valid_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesTransaction::fresh(
        id,
        i64::MAX / 2,
        txn_date(),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("large positive total_paid_amount_minor is valid");
    assert_eq!(row.total_paid_amount_minor, i64::MAX / 2);
}

#[test]
fn fresh_none_description_valid_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesTransaction::fresh(
        id,
        5_000,
        txn_date(),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("None description is valid");
    assert_eq!(row.description, None);
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesTransaction::fresh(
        id,
        5_000,
        txn_date(),
        None,
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

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let mut row = RealFmFeesTransaction::fresh(
        id,
        5_000,
        txn_date(),
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.total_paid_amount_minor, 5_000);
    assert_eq!(row.transaction_date, txn_date());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let mut row = RealFmFeesTransaction::fresh(
        id,
        5_000,
        txn_date(),
        None,
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
fn create_fm_fees_transaction_service_emits_created_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fft_id(&g, school);
    let cmd = CreateFmFeesTransactionCommand {
        tenant,
        fm_fees_transaction_id: id,
        total_paid_amount_minor: 10_000,
        transaction_date: txn_date(),
        description: Some("test".to_string()),
    };
    let (_agg, evt): (RealFmFeesTransaction, FmFeesTransactionCreated) =
        create_fm_fees_transaction(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.total_paid_amount_minor, 10_000);
    assert_eq!(evt.fm_fees_transaction_id, id);
    assert_eq!(
        <FmFeesTransactionCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_transaction.created"
    );
    assert_eq!(
        <FmFeesTransactionCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_transaction"
    );
    assert_eq!(
        <FmFeesTransactionCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn create_fm_fees_transaction_service_rejects_negative_total_paid_amount_fft_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fft_id(&g, school);
    let cmd = CreateFmFeesTransactionCommand {
        tenant,
        fm_fees_transaction_id: id,
        total_paid_amount_minor: -500,
        transaction_date: txn_date(),
        description: None,
    };
    let result = create_fm_fees_transaction(cmd, &clock, &g);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn retire_fm_fees_transaction_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireFmFeesTransactionCommand {
        tenant,
        fm_fees_transaction_id: fft_id(&g, school),
    };
    let evt: FmFeesTransactionRetired =
        retire_fm_fees_transaction(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesTransactionRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_transaction.retired"
    );
    assert_eq!(
        <FmFeesTransactionRetired as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_transaction"
    );
}

#[test]
fn read_fm_fees_transaction_service_returns_ok() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = ReadFmFeesTransactionCommand {
        tenant,
        fm_fees_transaction_id: fft_id(&g, school),
    };
    read_fm_fees_transaction(cmd, &clock, &g).expect("read should succeed");
}
