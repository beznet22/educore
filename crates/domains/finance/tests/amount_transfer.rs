//! Integration tests for the **AmountTransfer aggregate** vertical slice.
//!
//! Pins the AT I-2 invariant end-to-end: an AmountTransfer's
//! (from_account_id, to_account_id) scope-key tuple must be
//! distinct (the aggregate cannot transfer to the same account)
//! AND amount_minor must be >= 0 (a negative transfer is a
//! reversal, not a fresh transfer).
//!
//! Replaces the prior 2 typed-id-only tests with an 11-test
//! behavioral suite.

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
    create_amount_transfer, retire_amount_transfer, AmountTransferCreated, AmountTransferId,
    AmountTransferRetired, BankAccountId, Currency, RealAmountTransfer,
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

fn at_id(g: &SystemIdGen, school: SchoolId) -> AmountTransferId {
    AmountTransferId::new(school, g.next_uuid())
}

fn bank_id(g: &SystemIdGen, school: SchoolId) -> BankAccountId {
    BankAccountId::new(school, g.next_uuid())
}

#[test]
fn amount_transfer_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let id = at_id(&g, tenant.school_id);
    assert_eq!(id.school_id(), tenant.school_id);
}

#[test]
fn amount_transfer_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let id_a = at_id(&g, tenant.school_id);
    let id_b = at_id(&g, tenant.school_id);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), tenant.school_id);
}

#[test]
fn fresh_full_payload_valid_at_i_2_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let from = bank_id(&g, school);
    let to = bank_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealAmountTransfer::fresh(
        id,
        from,
        to,
        50_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        Some("Tuition account → operations account".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("AT I-2: distinct (from, to) + positive amount must construct");
    assert!(agg.is_active());
    assert_eq!(agg.from_account_id, from);
    assert_eq!(agg.to_account_id, to);
    assert_eq!(agg.amount_minor, 50_000);
    assert_eq!(agg.currency, Currency::INR);
    assert_eq!(agg.note.as_deref(), Some("Tuition account → operations account"));
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_amount_boundary_valid_at_i_2_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let from = bank_id(&g, school);
    let to = bank_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealAmountTransfer::fresh(
        id,
        from,
        to,
        0,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("AT I-2: zero amount is a valid boundary");
    assert_eq!(agg.amount_minor, 0);
}

#[test]
fn fresh_same_account_validation_error_at_i_2_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let same_account = bank_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealAmountTransfer::fresh(
        id,
        same_account,
        same_account,
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("AT I-2: from == to must be rejected");
    assert!(
        format!("{err}").contains("must differ from to_account_id"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_amount_validation_error_at_i_2_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let from = bank_id(&g, school);
    let to = bank_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealAmountTransfer::fresh(
        id,
        from,
        to,
        -1,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("AT I-2: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealAmountTransfer::fresh(
        at_id(&g, school),
        bank_id(&g, school),
        bank_id(&g, school),
        25_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("AT I-2: distinct (from, to) must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealAmountTransfer::fresh(
        at_id(&g, school),
        bank_id(&g, school),
        bank_id(&g, school),
        10_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("AT I-2: distinct (from, to) must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealAmountTransfer::fresh(
        at_id(&g, school),
        bank_id(&g, school),
        bank_id(&g, school),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 8, 15).unwrap(),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("AT I-2: distinct (from, to) must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_amount_transfer_service_emits_created_event_at_i_2() {
    use educore_finance::commands::CreateAmountTransferCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let from = bank_id(&g, school);
    let to = bank_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateAmountTransferCommand {
        tenant: tenant.clone(),
        amount_transfer_id: id,
        from_account_id: from,
        to_account_id: to,
        amount_minor: 75_000,
        currency: Currency::INR,
        transfer_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        note: Some("Service integration test".to_owned()),
    };
    let (agg, event): (RealAmountTransfer, AmountTransferCreated) =
        create_amount_transfer(cmd, &clock, &ids)
            .expect("create_amount_transfer must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.amount_minor, 75_000);
    assert_eq!(event.amount_transfer_id, agg.id);
    assert_eq!(event.from_account_id, from);
    assert_eq!(event.to_account_id, to);
    assert_eq!(event.amount_minor, 75_000);
    assert_eq!(
        event.note.as_deref(),
        Some("Service integration test")
    );
    assert_eq!(
        <AmountTransferCreated as DomainEvent>::EVENT_TYPE,
        "finance.amount_transfer.created"
    );
    assert_eq!(
        <AmountTransferCreated as DomainEvent>::AGGREGATE_TYPE,
        "amount_transfer"
    );
    assert_eq!(
        <AmountTransferCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_amount_transfer_service_rejects_same_account_at_i_2() {
    use educore_finance::commands::CreateAmountTransferCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = at_id(&g, school);
    let same = bank_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateAmountTransferCommand {
        tenant: tenant.clone(),
        amount_transfer_id: id,
        from_account_id: same,
        to_account_id: same,
        amount_minor: 1_000,
        currency: Currency::INR,
        transfer_date: chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        note: None,
    };
    let err = create_amount_transfer(cmd, &clock, &ids)
        .expect_err("AT I-2: same from/to must be rejected at service layer");
    assert!(
        format!("{err}").contains("must differ from to_account_id"),
        "unexpected error: {err}"
    );
}
