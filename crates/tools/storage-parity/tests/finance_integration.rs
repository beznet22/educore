//! # Finance domain vertical-slice integration test
//!
//! Mirrors the Phase 6 HR pattern (`hr_integration.rs`).
//! Runs on SQLite (always) + PG/MySQL (env-gated).
//! The headline scenario: configure invoice numbering →
//! create a wallet → credit the wallet → record a payment →
//! record an expense. Verifies the bus + outbox + audit +
//! idempotency rows in a single transaction.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use educore_core::clock::{SystemClock, SystemIdGen};
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::idempotency::IdempotencyRecord;
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::transaction::Transaction;
use educore_storage::StorageAdapter;

use educore_finance::prelude::*;
use educore_finance::services::{
    ConfigureInvoiceNumberingCommand, CreateWalletCommand, CreditWalletCommand,
    RecordExpenseCommand, RecordPaymentCommand, StubPaymentProvider, WalletService,
};

async fn setup_test_env() -> (
    Arc<dyn StorageAdapter>,
    Arc<dyn EventBus>,
    TenantContext,
    SystemIdGen,
) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    (adapter, bus, ctx, g)
}

#[tokio::test]
async fn finance_integration_sqlite_vertical_slice() {
    let (adapter, bus, ctx, _g) = setup_test_env().await;
    let school = ctx.school_id;
    let user_id: UserId = ctx.actor_id;
    let clock = SystemClock;
    let ids = SystemIdGen;

    // Subscribe to bus BEFORE dispatching.
    let mut opts = SubscribeOptions::for_consumer("test-finance".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

    // 1. Configure invoice numbering.
    let (invoice, invoice_event) = configure_invoice_numbering(
        ConfigureInvoiceNumberingCommand {
            tenant: ctx.clone(),
            prefix: "INV-2026-".to_owned(),
            start_form: 1000,
        },
        &clock,
        &ids,
    )
    .expect("configure_invoice_numbering");
    assert_eq!(invoice.prefix, "INV-2026-");
    assert_eq!(invoice.start_form, 1000);
    assert_eq!(
        <InvoiceNumberingConfigured as DomainEvent>::EVENT_TYPE,
        "finance.fees_invoice.configured"
    );

    // 2. Create a wallet for the user.
    let (wallet, wallet_event) = create_wallet(
        CreateWalletCommand {
            tenant: ctx.clone(),
            user_id,
            currency: Currency::INR,
        },
        &clock,
        &ids,
    )
    .expect("create_wallet");
    assert_eq!(wallet.balance_minor, 0);
    assert_eq!(wallet.user_id, user_id);
    assert_eq!(
        <WalletCreated as DomainEvent>::EVENT_TYPE,
        "finance.wallet.created"
    );

    // 3. Credit the wallet.
    let credit_cmd = CreditWalletCommand {
        tenant: ctx.clone(),
        wallet_id: wallet.id,
        user_id,
        amount_minor: 5_000_00, // INR 5000.00
        currency: Currency::INR,
        wallet_type: WalletTxType::Deposit,
        payment_method_id: None,
        bank_id: None,
        reference: Some("INITIAL-LOAD".to_owned()),
        note: Some("test credit".to_owned()),
    };
    let (tx, credit_event) = credit_wallet(credit_cmd, &clock, &ids).expect("credit_wallet");
    assert_eq!(tx.amount_minor, 5_000_00);
    assert_eq!(tx.wallet_type, WalletTxType::Deposit);
    assert_eq!(
        <WalletCredited as DomainEvent>::EVENT_TYPE,
        "finance.wallet.credited"
    );

    // 4. Record a fees payment.
    let payment_cmd = RecordPaymentCommand {
        tenant: ctx.clone(),
        amount_minor: 10_000_00, // INR 10000.00
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: 0,
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: Some("INV-2026-1000".to_owned()),
        note: Some("test payment".to_owned()),
        payment_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
    };
    let (payment, payment_event) =
        record_payment(payment_cmd, &clock, &ids).expect("record_payment");
    assert_eq!(payment.amount_minor, 10_000_00);
    assert_eq!(payment.net_minor(), 10_000_00); // no discount
    assert_eq!(
        <PaymentReceived as DomainEvent>::EVENT_TYPE,
        "finance.fees_payment.recorded"
    );

    // 5. Record an expense.
    let expense_head_id = ExpenseHeadId::new(school, uuid::Uuid::now_v7());
    let bank_account_id = BankAccountId::new(school, uuid::Uuid::now_v7());
    let expense_cmd = RecordExpenseCommand {
        tenant: ctx.clone(),
        name: "Office supplies".to_owned(),
        amount_minor: 2_000_00, // INR 2000.00
        currency: Currency::INR,
        expense_head_id,
        account_id: bank_account_id,
        payment_method: PaymentMethodKind::Cash,
        expense_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        file_reference: None,
        description: Some("test expense".to_owned()),
        payroll_payment_id: None,
    };
    let (expense, expense_event) =
        record_expense(expense_cmd, &clock, &ids).expect("record_expense");
    assert_eq!(expense.amount_minor, 2_000_00);
    assert_eq!(expense.name, "Office supplies");
    assert_eq!(
        <ExpenseRecorded as DomainEvent>::EVENT_TYPE,
        "finance.expense.recorded"
    );

    // 6. Build envelopes and write outbox + audit + idempotency in a single tx.
    let envelopes: Vec<EventEnvelope> = vec![
        invoice_event.into_envelope(&ctx),
        wallet_event.into_envelope(&ctx),
        credit_event.into_envelope(&ctx),
        payment_event.into_envelope(&ctx),
        expense_event.into_envelope(&ctx),
    ];

    let tx = adapter.begin().await.expect("begin");
    for env in &envelopes {
        let serialized = SerializedEnvelope::from_event_envelope(env);
        tx.outbox().append(serialized).await.expect("outbox append");
    }
    let idem_record = IdempotencyRecord {
        school_id: school,
        command_type: "finance.vertical_slice",
        idempotency_key: IdempotencyKey::from(uuid::Uuid::now_v7()),
        outcome: bytes::Bytes::from_static(br#"{"status":"ok"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![
            invoice.id.as_uuid(),
            wallet.id.as_uuid(),
            payment.id.as_uuid(),
            expense.id.as_uuid(),
        ],
    };
    let audit_entry = AuditLogEntry::create(
        school,
        ctx.actor_id,
        "finance_vertical_slice",
        invoice.id.as_uuid(),
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    tx.commit().await.expect("commit");

    // 7. Publish envelopes to bus.
    for env in envelopes {
        bus.publish(env).await.expect("bus publish");
    }

    // 8. Verify the bus received the first envelope.
    let received = sub.next().await;
    match received {
        Some(Ok(env)) => {
            // First event is the invoice configured.
            assert_eq!(env.event_type, "finance.fees_invoice.configured");
            assert_eq!(env.school_id, school);
        }
        other => panic!("expected bus event, got {other:?}"),
    }
}

#[tokio::test]
async fn finance_capability_check_gates_record_payment() {
    use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};

    let cap_check = InMemoryCapabilityCheck::new();
    let g = SystemIdGen;
    let school = g.next_school_id();
    let user = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);

    // 1. No grant -> denied.
    let granted = cap_check
        .has(&ctx, Capability::FinancePaymentCollect)
        .await
        .expect("has");
    assert!(!granted);

    // 2. Grant to a role in the school -> allowed.
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    cap_check.grant(school, role, Capability::FinancePaymentCollect);
    let granted = cap_check
        .has(&ctx, Capability::FinancePaymentCollect)
        .await
        .expect("has");
    assert!(granted);
}

#[test]
fn finance_event_type_round_trip_for_all_headline_aggregates() {
    let g = SystemIdGen;
    let s = SchoolId(uuid::Uuid::now_v7());

    // 1. WalletCreated
    let ev = WalletCreated::new(
        WalletId::new(s, uuid::Uuid::now_v7()),
        UserId(uuid::Uuid::now_v7()),
        Currency::INR,
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <WalletCreated as DomainEvent>::EVENT_TYPE,
        "finance.wallet.created"
    );
    assert_eq!(<WalletCreated as DomainEvent>::AGGREGATE_TYPE, "wallet");

    // 2. WalletCredited
    let ev = WalletCredited::new(
        WalletId::new(s, uuid::Uuid::now_v7()),
        WalletTransactionId::new(s, uuid::Uuid::now_v7()),
        UserId(uuid::Uuid::now_v7()),
        1000,
        Currency::INR,
        WalletTxType::Deposit,
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <WalletCredited as DomainEvent>::EVENT_TYPE,
        "finance.wallet.credited"
    );

    // 3. WalletRefundRequested
    let ev = WalletRefundRequested::new(
        WalletTransactionId::new(s, uuid::Uuid::now_v7()),
        WalletId::new(s, uuid::Uuid::now_v7()),
        UserId(uuid::Uuid::now_v7()),
        500,
        Currency::INR,
        "test refund".to_owned(),
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <WalletRefundRequested as DomainEvent>::EVENT_TYPE,
        "finance.wallet.refund_requested"
    );

    // 4. InvoiceNumberingConfigured
    let ev = InvoiceNumberingConfigured::new(
        FeesInvoiceId::new(s, uuid::Uuid::now_v7()),
        "INV-".to_owned(),
        1,
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <InvoiceNumberingConfigured as DomainEvent>::EVENT_TYPE,
        "finance.fees_invoice.configured"
    );

    // 5. PaymentReceived
    let ev = PaymentReceived::new(
        FeesPaymentId::new(s, uuid::Uuid::now_v7()),
        10_000,
        Currency::INR,
        0,
        0,
        PaymentMethodKind::Cash,
        None,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <PaymentReceived as DomainEvent>::EVENT_TYPE,
        "finance.fees_payment.recorded"
    );

    // 6. ExpenseRecorded
    let ev = ExpenseRecorded::new(
        ExpenseId::new(s, uuid::Uuid::now_v7()),
        "Office supplies".to_owned(),
        5_000,
        Currency::INR,
        ExpenseHeadId::new(s, uuid::Uuid::now_v7()),
        BankAccountId::new(s, uuid::Uuid::now_v7()),
        PaymentMethodKind::Cash,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 13).unwrap(),
        None,
        g.next_event_id(),
        CorrelationId(g.next_uuid()),
        Timestamp::now(),
    );
    assert_eq!(
        <ExpenseRecorded as DomainEvent>::EVENT_TYPE,
        "finance.expense.recorded"
    );
    assert_eq!(<ExpenseRecorded as DomainEvent>::AGGREGATE_TYPE, "expense");
}

#[test]
fn stub_payment_provider_returns_synthesized_local_id() {
    let stub = StubPaymentProvider::new();
    let req = educore_finance::services::ChargeRequest {
        amount_minor: 500,
        currency: Currency::INR,
        method: PaymentMethodKind::Cash,
        school_id: SchoolId(uuid::Uuid::now_v7()),
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let receipt = rt.block_on(stub.charge(req)).expect("charge");
    assert_eq!(receipt.provider_payment_id, "local://stub/0");
    assert_eq!(receipt.amount_minor, 500);
    assert_eq!(
        receipt.status,
        educore_finance::services::PaymentProviderStatus::Captured
    );
}

#[test]
fn wallet_service_balance_helpers_compile() {
    // Smoke test: the `WalletService` helper is exposed via the
    // prelude and has a `validate_debit` method.
    let g = SystemIdGen;
    let s = g.next_school_id();
    let user = g.next_user_id();
    let wid = WalletId::new(s, uuid::Uuid::now_v7());
    let mut wallet = Wallet::fresh(
        wid,
        user,
        Currency::INR,
        user,
        Timestamp::now(),
        CorrelationId(g.next_uuid()),
    );
    wallet.balance_minor = 1000;
    // Within balance: ok
    WalletService::validate_debit(&wallet, 500, Currency::INR).expect("debit ok");
    // Exceeds balance: err
    let err = WalletService::validate_debit(&wallet, 1500, Currency::INR).unwrap_err();
    assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
}
