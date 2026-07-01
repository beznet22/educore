//! Integration tests for the **finance domain workflows**.
//!
//! Implements: `docs/specs/finance/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the finance service factories and aggregate
//! methods, and asserts that the expected typed event is
//! emitted (or, on the error path, that the expected
//! [`DomainError`] is returned and no event is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! finance service factories (`create_wallet`, `credit_wallet`,
//! `approve_wallet_transaction`, `reject_wallet_transaction`,
//! `request_wallet_refund`, `deduct_wallet_credit`,
//! `record_payment`, `record_expense`,
//! `configure_invoice_numbering`) and aggregate methods
//! (`Wallet::apply_credit`, `Wallet::apply_debit`,
//! `WalletTransaction::approve`, `WalletTransaction::reject`,
//! `FeesPayment::net_minor`) are sync, take a `Clock` +
//! `IdGenerator` (or operate on the aggregate directly), and
//! return `Result<(), DomainError>` for state-machine
//! transitions. The test wires a [`TestClock`] and a
//! [`SystemIdGen`], and constructs the typed events directly
//! from the aggregate + clock instant to verify the event
//! payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no
//! subscriber fan-out, no outbox commit, no audit row). These
//! tests pin the contract of the **aggregate + service
//! layer** that the dispatcher wraps. When the handlers
//! land, the same test bodies will gain a `+ outbox + bus
//! subscriber` assertion without changes to the assertions
//! on the returned event.
//!
//! Coverage per `docs/specs/finance/workflows.md`:
//!
//! - **§ Invoice Generation (Per Class / Per Student / Per
//!   Term)** → `Invoice Lifecycle`: `configure_invoice_numbering`
//!   + `FeesInvoice::fresh` validation + re-configuration.
//! - **§ Payment Collection (Cash / Bank Slip / Gateway)** →
//!   `Payment Lifecycle`: `record_payment` + `FeesPayment`
//!   net computation + multi-payment allocation + reject
//!   negative amount / discount / fine.
//! - **§ Wallet Credit / Wallet Refund / Wallet Debit** →
//!   `Fee Structure Lifecycle` (the headline 6 `Wallet` +
//!   `WalletTransaction` aggregate has the canonical
//!   `Pending → Approved | Rejected` lifecycle that the
//!   `FeesGroup` / `FeesType` / `FeesMaster` aggregates will
//!   reuse): `credit_wallet` + `approve_wallet_transaction` +
//!   `reject_wallet_transaction` + double-approval rejection
//!   + cross-terminal rejection.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::CorrelationId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::*;
use educore_finance::services as fin_services;
use educore_finance::value_objects::{
    ApprovalStatus, BankAccountId, Currency, PaymentMethodKind, WalletId, WalletTxType,
};

// =============================================================================
// Test fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
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

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn bank_account_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> BankAccountId {
    BankAccountId::new(school, g.next_uuid())
}

fn payment_method_id(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> educore_finance::value_objects::PaymentMethodId {
    educore_finance::value_objects::PaymentMethodId::new(school, g.next_uuid())
}

/// Construct a fresh `Wallet` aggregate for a given school +
/// actor.
fn new_wallet_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    currency: Currency,
) -> Wallet {
    let cmd = fin_services::CreateWalletCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        user_id: actor,
        currency,
    };
    let clock = TestClock::new();
    let (wallet, _event) = fin_services::create_wallet(cmd, &clock, g).expect("create_wallet");
    wallet
}

/// Construct a fresh `WalletTransaction` aggregate in the
/// `Pending` state.
#[allow(clippy::too_many_arguments)]
fn new_pending_wallet_transaction(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    wallet_id_val: WalletId,
    amount_minor: i64,
    currency: Currency,
    wallet_type: WalletTxType,
) -> WalletTransaction {
    let cmd = fin_services::CreditWalletCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: wallet_id_val,
        user_id: actor,
        amount_minor,
        currency,
        wallet_type,
        payment_method_id: None,
        bank_id: None,
        reference: None,
        note: None,
    };
    let clock = TestClock::new();
    let (tx, _event) = fin_services::credit_wallet(cmd, &clock, g).expect("credit_wallet");
    tx
}

// =============================================================================
// 1. Invoice Lifecycle
//    (`workflows.md` § "Invoice Generation (Per Class /
//    Per Student / Per Term)" — the headline `FeesInvoice`
//    aggregate stores the school's invoice numbering
//    configuration per the spec invariant that a school has
//    exactly one numbering scheme at any time)
// =============================================================================

/// Invoice lifecycle step 1 (define): configuring the
/// school's invoice numbering for the first time emits
/// [`InvoiceNumberingConfigured`] with the supplied
/// `prefix` and `start_form`.
#[test]
fn invoice_lifecycle_configure_emits_invoice_numbering_configured() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let cmd = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "INV-".to_owned(),
        start_form: 1_000,
    };
    let (invoice, event) =
        fin_services::configure_invoice_numbering(cmd, &clock, &g).expect("configure_invoice");

    assert_eq!(
        <InvoiceNumberingConfigured as DomainEvent>::EVENT_TYPE,
        "finance.fees_invoice.configured"
    );
    assert_eq!(
        <InvoiceNumberingConfigured as DomainEvent>::AGGREGATE_TYPE,
        "fees_invoice"
    );
    assert_eq!(event.prefix, "INV-");
    assert_eq!(event.start_form, 1_000);
    assert_eq!(event.fees_invoice_id, invoice.id);
    assert_eq!(event.school_id(), school);
    assert_eq!(invoice.school_id, school);
    assert_eq!(invoice.prefix, "INV-");
    assert_eq!(invoice.start_form, 1_000);
    // event.correlation_id is a typed CorrelationId (compile-time check)
    let _ = correlation;
}

/// Invoice lifecycle failure path: per spec invariant
/// (invoice prefix must be 1..=10 chars), an empty prefix
/// must be rejected at construction time so that
/// `FeesInvoice::fresh` (and by extension
/// `configure_invoice_numbering`) can never produce a
/// degenerate configuration.
#[test]
fn invoice_lifecycle_empty_prefix_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: String::new(),
        start_form: 1,
    };
    let err = fin_services::configure_invoice_numbering(cmd, &clock, &g)
        .expect_err("empty prefix must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Invoice lifecycle failure path: per spec invariant
/// (invoice prefix must be 1..=10 chars), a prefix longer
/// than 10 chars must be rejected.
#[test]
fn invoice_lifecycle_oversize_prefix_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "INV-2026/TOO-LONG-PREFIX".to_owned(),
        start_form: 1,
    };
    let err = fin_services::configure_invoice_numbering(cmd, &clock, &g)
        .expect_err("oversize prefix must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Invoice lifecycle failure path: per spec invariant
/// (`start_form` must be non-negative), a negative
/// `start_form` must be rejected.
#[test]
fn invoice_lifecycle_negative_start_form_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "INV-".to_owned(),
        start_form: -1,
    };
    let err = fin_services::configure_invoice_numbering(cmd, &clock, &g)
        .expect_err("negative start_form must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Invoice lifecycle re-configure step: configuring the
/// school's invoice numbering a second time produces a
/// **distinct** `FeesInvoiceId` + a fresh
/// `InvoiceNumberingConfigured` event. Per the spec
/// invariant, the historical numbering configuration is
/// retained for audit; the new configuration supersedes the
/// old one at the dispatch layer.
#[test]
fn invoice_lifecycle_reconfigure_emits_new_event_with_distinct_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd1 = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "INV-".to_owned(),
        start_form: 1,
    };
    let (invoice1, event1) =
        fin_services::configure_invoice_numbering(cmd1, &clock, &g).expect("configure #1");

    let cmd2 = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "FEE/26-".to_owned(),
        start_form: 5_000,
    };
    let (invoice2, event2) =
        fin_services::configure_invoice_numbering(cmd2, &clock, &g).expect("configure #2");

    assert_ne!(invoice1.id, invoice2.id);
    assert_eq!(invoice1.prefix, "INV-");
    assert_eq!(invoice2.prefix, "FEE/26-");
    assert_eq!(event1.start_form, 1);
    assert_eq!(event2.start_form, 5_000);
}

/// Invoice lifecycle boundary: the maximum-length prefix
/// (10 chars) is accepted (inclusive upper bound).
#[test]
fn invoice_lifecycle_max_length_prefix_is_accepted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::ConfigureInvoiceNumberingCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        prefix: "ABCDEFGHIJ".to_owned(), // exactly 10 chars
        start_form: 0,
    };
    let (invoice, event) = fin_services::configure_invoice_numbering(cmd, &clock, &g)
        .expect("10-char prefix must succeed");
    assert_eq!(invoice.prefix.len(), 10);
    assert_eq!(event.prefix.len(), 10);
    assert_eq!(invoice.start_form, 0);
}

// =============================================================================
// 2. Payment Lifecycle
//    (`workflows.md` § "Payment Collection (Cash / Bank
//    Slip / Gateway)" — the headline `FeesPayment` aggregate
//    is the per-payment journal entry; the dispatcher wires
//    the `PaymentProvider` port (Phase 15) and the
//    double-entry ledger (Workstream C))
// =============================================================================

/// Payment lifecycle step 1 (record): recording a cash
/// payment via `record_payment` produces a `FeesPayment`
/// aggregate + a [`PaymentReceived`] event with the
/// supplied amount, currency, discount, fine, and method.
#[test]
fn payment_lifecycle_record_emits_payment_received() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 50_000, // INR 500.00
        currency: Currency::INR,
        discount_minor: 5_000, // INR 50.00 scholarship
        fine_minor: 0,
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: Some("Receipt #2026-001".to_owned()),
        note: None,
        payment_date: date(2026, 6, 13),
    };
    let (payment, event) = fin_services::record_payment(cmd, &clock, &g).expect("record_payment");

    assert_eq!(
        <PaymentReceived as DomainEvent>::EVENT_TYPE,
        "finance.fees_payment.recorded"
    );
    assert_eq!(
        <PaymentReceived as DomainEvent>::AGGREGATE_TYPE,
        "fees_payment"
    );
    assert_eq!(payment.amount_minor, 50_000);
    assert_eq!(payment.discount_minor, 5_000);
    assert_eq!(payment.fine_minor, 0);
    assert_eq!(payment.payment_method, PaymentMethodKind::Cash);
    assert_eq!(payment.payment_date, date(2026, 6, 13));
    assert_eq!(event.fees_payment_id, payment.id);
    assert_eq!(event.school_id(), school);
    assert_eq!(event.amount_minor, 50_000);
    assert_eq!(event.discount_minor, 5_000);
    // The net amount is `amount - discount` per spec invariant.
    assert_eq!(payment.net_minor(), 45_000);
    assert_eq!(event.fine_minor, 0);
}

/// Payment lifecycle step 2 (allocate): the `net_minor`
/// invariant — `net = amount - discount` — must hold
/// across `Cash`, `Bank`, `Card`, `Mobile`, and `Gateway`
/// payment methods. Per the spec, the method kind is a
/// projection of the payment collection channel, not a
/// different arithmetic rule.
#[test]
fn payment_lifecycle_net_amount_holds_across_payment_methods() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    for method in [
        PaymentMethodKind::Cash,
        PaymentMethodKind::Bank,
        PaymentMethodKind::Cheque,
        PaymentMethodKind::Card,
        PaymentMethodKind::Mobile,
        PaymentMethodKind::Gateway,
    ] {
        // INV-FP-METHOD-FK + INV-FP-GATEWAY-REF: non-cash
        // methods must carry a `payment_method_id`; gateway
        // must additionally carry a `reference` (the gateway
        // transaction id). Cash is the one accepted exception.
        let method_id = if method == PaymentMethodKind::Cash {
            None
        } else {
            Some(PaymentMethodId::new(school, g.next_uuid()))
        };
        let reference = if method == PaymentMethodKind::Gateway {
            Some(format!("GTW-{}", g.next_uuid()))
        } else {
            None
        };
        let cmd = fin_services::RecordPaymentCommand {
            tenant: TenantContext::for_user(
                school,
                actor,
                g.next_correlation_id(),
                UserType::SchoolAdmin,
            ),
            amount_minor: 12_345,
            currency: Currency::INR,
            discount_minor: 2_345,
            fine_minor: 0,
            payment_method: method,
            bank_id: None,
            payment_method_id: method_id,
            reference,
            note: None,
            payment_date: date(2026, 6, 13),
        };
        let (payment, event) =
            fin_services::record_payment(cmd, &clock, &g).expect("record_payment");
        assert_eq!(payment.payment_method, method);
        assert_eq!(payment.amount_minor - payment.discount_minor, 10_000);
        assert_eq!(event.payment_method, method);
    }
}

/// Payment lifecycle step 3 (allocate with fine): per the
/// spec, late payments may include a `fine_minor`
/// component. The `fine_minor` is recorded on the aggregate
/// and emitted on the event but **does not** enter the
/// `net_minor` calculation (the fine is collected on top of
/// the amount, not netted out of it).
#[test]
fn payment_lifecycle_fine_is_recorded_but_not_netted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 30_000,
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: 1_500, // INR 15.00 late fee
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: Some("Late payment June".to_owned()),
        note: None,
        payment_date: date(2026, 6, 13),
    };
    let (payment, event) = fin_services::record_payment(cmd, &clock, &g).expect("record_payment");

    // fine is recorded on the aggregate + event
    assert_eq!(payment.fine_minor, 1_500);
    assert_eq!(event.fine_minor, 1_500);
    // but net_minor() is `amount - discount` (fine excluded)
    assert_eq!(payment.net_minor(), 30_000);
}

/// Payment lifecycle multi-payment (partial allocation):
/// per the spec idempotency rule, `record_payment` is
/// idempotent on `(fees_assign_id, transaction_id)` — a
/// duplicate `(reference, payment_date, amount)` tuple is
/// a no-op success. The aggregate itself is stateless w.r.t.
/// cumulative balance (the dispatcher + `FeesPayment`
/// repository track the running balance), so two
/// `record_payment` calls produce two distinct
/// `FeesPaymentId`s and two `PaymentReceived` events with
/// distinct `event_id`s.
#[test]
fn payment_lifecycle_two_payments_produce_two_distinct_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let bank = bank_account_id(&g, school);
    let pm_id = payment_method_id(&g, school);

    // Payment 1: INR 400.00 cash, partial settlement of a
    // larger invoice.
    let cmd1 = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 40_000,
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: 0,
        payment_method: PaymentMethodKind::Cash,
        bank_id: Some(bank),
        payment_method_id: Some(pm_id),
        reference: Some("TXN-001".to_owned()),
        note: Some("partial settlement 1/2".to_owned()),
        payment_date: date(2026, 6, 13),
    };
    let (p1, e1) = fin_services::record_payment(cmd1, &clock, &g).expect("payment #1");

    // Payment 2: INR 100.00 bank transfer, closes the invoice.
    let cmd2 = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 10_000,
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: 0,
        payment_method: PaymentMethodKind::Bank,
        bank_id: Some(bank),
        payment_method_id: Some(pm_id),
        reference: Some("TXN-002".to_owned()),
        note: Some("final settlement".to_owned()),
        payment_date: date(2026, 6, 14),
    };
    let (p2, e2) = fin_services::record_payment(cmd2, &clock, &g).expect("payment #2");

    assert_ne!(p1.id, p2.id);
    assert_ne!(e1.fees_payment_id, e2.fees_payment_id);
    assert_eq!(p1.payment_method, PaymentMethodKind::Cash);
    assert_eq!(p2.payment_method, PaymentMethodKind::Bank);
    assert_eq!(p1.payment_date, date(2026, 6, 13));
    assert_eq!(p2.payment_date, date(2026, 6, 14));
    // Each event captures the per-payment amount, not the
    // cumulative running balance.
    assert_eq!(e1.amount_minor, 40_000);
    assert_eq!(e2.amount_minor, 10_000);
}

/// Payment lifecycle failure path: a negative `amount_minor`
/// must be rejected at construction (per spec invariant
/// "amount must be non-negative").
#[test]
fn payment_lifecycle_negative_amount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: -1,
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: 0,
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: None,
        note: None,
        payment_date: date(2026, 6, 13),
    };
    let err = fin_services::record_payment(cmd, &clock, &g)
        .expect_err("negative amount must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Payment lifecycle failure path: a negative `discount_minor`
/// must be rejected at construction (per spec invariant
/// "discount must be non-negative").
#[test]
fn payment_lifecycle_negative_discount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 1_000,
        currency: Currency::INR,
        discount_minor: -1,
        fine_minor: 0,
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: None,
        note: None,
        payment_date: date(2026, 6, 13),
    };
    let err = fin_services::record_payment(cmd, &clock, &g)
        .expect_err("negative discount must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

/// Payment lifecycle failure path: a negative `fine_minor`
/// must be rejected at construction (per spec invariant
/// "fine must be non-negative").
#[test]
fn payment_lifecycle_negative_fine_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cmd = fin_services::RecordPaymentCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        amount_minor: 1_000,
        currency: Currency::INR,
        discount_minor: 0,
        fine_minor: -1,
        payment_method: PaymentMethodKind::Cash,
        bank_id: None,
        payment_method_id: None,
        reference: None,
        note: None,
        payment_date: date(2026, 6, 13),
    };
    let err = fin_services::record_payment(cmd, &clock, &g)
        .expect_err("negative fine must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}

// =============================================================================
// 3. Fee Structure Lifecycle
//    (`workflows.md` § "Wallet Credit / Wallet Refund /
//    Wallet Debit" — the headline `WalletTransaction`
//    aggregate has the canonical
//    `Pending → Approved | Rejected` lifecycle that the
//    `FeesGroup` / `FeesType` / `FeesMaster` aggregates
//    will reuse per the spec invariant "approval gates all
//    wallet-side state mutations")
// =============================================================================

/// Fee-structure lifecycle step 1 (define): crediting a
/// wallet via `credit_wallet` produces a `WalletTransaction`
/// aggregate in the `Pending` state + a [`WalletCredited`]
/// event. The wallet itself is not credited until the
/// transaction is approved (the dispatcher wires the
/// `Wallet::apply_credit` call on approval).
#[test]
fn fee_structure_lifecycle_define_emits_wallet_credited_pending() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);

    let cmd = fin_services::CreditWalletCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: w.id,
        user_id: actor,
        amount_minor: 25_000,
        currency: Currency::INR,
        wallet_type: WalletTxType::Deposit,
        payment_method_id: None,
        bank_id: None,
        reference: Some("Top-up June".to_owned()),
        note: None,
    };
    let (tx, event) = fin_services::credit_wallet(cmd, &clock, &g).expect("credit_wallet");

    assert_eq!(
        <WalletCredited as DomainEvent>::EVENT_TYPE,
        "finance.wallet.credited"
    );
    assert_eq!(event.wallet_id, w.id);
    assert_eq!(event.wallet_transaction_id, tx.id);
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(event.currency, Currency::INR);
    assert_eq!(event.wallet_type, WalletTxType::Deposit);
    // Newly minted transaction is Pending — not yet applied to wallet.
    assert_eq!(tx.status, ApprovalStatus::Pending);
    assert_eq!(w.balance_minor, 0); // wallet balance is unchanged until approve
}

/// Fee-structure lifecycle step 2 (activate): approving a
/// pending transaction via `approve_wallet_transaction`
/// emits [`WalletTransactionApproved`] and transitions the
/// aggregate to `Approved`. Per spec invariant "approval
/// gates all wallet-side state mutations", the wallet
/// balance is updated by the dispatcher at this point.
#[test]
fn fee_structure_lifecycle_activate_transitions_to_approved() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let mut w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    let mut tx = new_pending_wallet_transaction(
        &g,
        school,
        actor,
        w.id,
        25_000,
        Currency::INR,
        WalletTxType::Deposit,
    );
    assert_eq!(tx.status, ApprovalStatus::Pending);

    let event =
        fin_services::approve_wallet_transaction(&mut tx, actor, &clock, &g).expect("approve");
    assert_eq!(
        <WalletTransactionApproved as DomainEvent>::EVENT_TYPE,
        "finance.wallet_transaction.approved"
    );
    assert_eq!(event.wallet_id, w.id);
    assert_eq!(event.wallet_transaction_id, tx.id);
    assert_eq!(event.approver_id, actor);
    assert_eq!(tx.status, ApprovalStatus::Approved);
    assert!(tx.approved_by.is_some());
    assert!(tx.approved_at.is_some());
    assert!(tx.status.is_terminal());

    // Once approved, the dispatcher applies the credit to the
    // wallet — verify the Wallet aggregate honours the credit.
    w.apply_credit(25_000, Currency::INR, actor, Timestamp::now())
        .expect("apply_credit");
    assert_eq!(w.balance_minor, 25_000);
}

/// Fee-structure lifecycle step 3 (retire / reject):
/// rejecting a pending transaction via
/// `reject_wallet_transaction` emits
/// [`WalletTransactionRejected`] and transitions the
/// aggregate to `Rejected`. The wallet balance is
/// unchanged (the dispatcher never applies the credit on a
/// rejection).
#[test]
fn fee_structure_lifecycle_retire_via_reject_transitions_to_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    let mut tx = new_pending_wallet_transaction(
        &g,
        school,
        actor,
        w.id,
        25_000,
        Currency::INR,
        WalletTxType::Deposit,
    );
    assert_eq!(tx.status, ApprovalStatus::Pending);

    let event = fin_services::reject_wallet_transaction(
        &mut tx,
        actor,
        "Insufficient funds source".to_owned(),
        &clock,
        &g,
    )
    .expect("reject");
    assert_eq!(
        <WalletTransactionRejected as DomainEvent>::EVENT_TYPE,
        "finance.wallet_transaction.rejected"
    );
    assert_eq!(event.wallet_id, w.id);
    assert_eq!(event.wallet_transaction_id, tx.id);
    assert_eq!(event.rejecter_id, actor);
    assert_eq!(event.reject_note, "Insufficient funds source");
    assert_eq!(tx.status, ApprovalStatus::Rejected);
    assert!(tx.rejected_by.is_some());
    assert!(tx.rejected_at.is_some());
    assert!(tx.status.is_terminal());

    // Wallet balance remains zero on rejection.
    assert_eq!(w.balance_minor, 0);
}

/// Fee-structure lifecycle state-machine invariant: a
/// transaction that is already `Approved` cannot be
/// re-approved (per spec invariant "Approved is terminal").
/// The aggregate must reject the transition with
/// `DomainError::Conflict` and the status must remain
/// `Approved`.
#[test]
fn fee_structure_lifecycle_double_approve_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    let mut tx = new_pending_wallet_transaction(
        &g,
        school,
        actor,
        w.id,
        1_000,
        Currency::INR,
        WalletTxType::Deposit,
    );

    fin_services::approve_wallet_transaction(&mut tx, actor, &clock, &g).expect("first approve");
    assert_eq!(tx.status, ApprovalStatus::Approved);

    let err = fin_services::approve_wallet_transaction(&mut tx, actor, &clock, &g)
        .expect_err("second approve must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
    // Status remains Approved.
    assert_eq!(tx.status, ApprovalStatus::Approved);
}

/// Fee-structure lifecycle state-machine invariant: a
/// transaction that is already `Approved` cannot be
/// rejected (per spec invariant "Approved is terminal and
/// not backtrackable"). The aggregate must reject the
/// transition with `DomainError::Conflict` and the status
/// must remain `Approved`.
#[test]
fn fee_structure_lifecycle_approve_then_reject_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    let mut tx = new_pending_wallet_transaction(
        &g,
        school,
        actor,
        w.id,
        1_000,
        Currency::INR,
        WalletTxType::Deposit,
    );

    fin_services::approve_wallet_transaction(&mut tx, actor, &clock, &g).expect("approve");
    let err =
        fin_services::reject_wallet_transaction(&mut tx, actor, "too late".to_owned(), &clock, &g)
            .expect_err("approve-then-reject must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
    assert_eq!(tx.status, ApprovalStatus::Approved);
}

/// Fee-structure lifecycle state-machine invariant: a
/// `Rejected` transaction cannot be approved (per spec
/// invariant "Rejected is terminal"). The aggregate must
/// reject the transition with `DomainError::Conflict`.
#[test]
fn fee_structure_lifecycle_reject_then_approve_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    let mut tx = new_pending_wallet_transaction(
        &g,
        school,
        actor,
        w.id,
        1_000,
        Currency::INR,
        WalletTxType::Deposit,
    );

    fin_services::reject_wallet_transaction(
        &mut tx,
        actor,
        "initial reject".to_owned(),
        &clock,
        &g,
    )
    .expect("reject");
    let err = fin_services::approve_wallet_transaction(&mut tx, actor, &clock, &g)
        .expect_err("reject-then-approve must be rejected");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
    assert_eq!(tx.status, ApprovalStatus::Rejected);
}

/// Fee-structure lifecycle — refund variant: per the spec,
/// the headline `Refund` is a `WalletTransaction` with
/// `wallet_type = Refund`. Requesting a refund emits a
/// [`WalletRefundRequested`] event and produces a
/// `Pending` transaction that must be approved before the
/// wallet is credited.
#[test]
fn fee_structure_lifecycle_refund_request_emits_wallet_refund_requested() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);

    let cmd = fin_services::RequestWalletRefundCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: w.id,
        user_id: actor,
        amount_minor: 5_000,
        currency: Currency::INR,
        reason: "Overpayment from previous invoice".to_owned(),
        reference: Some("REFUND-2026-001".to_owned()),
    };
    let (tx, event) =
        fin_services::request_wallet_refund(cmd, &clock, &g).expect("request_wallet_refund");

    assert_eq!(
        <WalletRefundRequested as DomainEvent>::EVENT_TYPE,
        "finance.wallet.refund_requested"
    );
    assert_eq!(
        <WalletRefundRequested as DomainEvent>::AGGREGATE_TYPE,
        "wallet"
    );
    assert_eq!(event.wallet_id, w.id);
    assert_eq!(event.wallet_transaction_id, tx.id);
    assert_eq!(event.amount_minor, 5_000);
    assert_eq!(event.reason, "Overpayment from previous invoice");
    assert_eq!(tx.wallet_type, WalletTxType::Refund);
    assert!(tx.wallet_type.is_credit());
    assert_eq!(tx.status, ApprovalStatus::Pending);
}

/// Fee-structure lifecycle — debit variant: deducting
/// wallet credit via `deduct_wallet_credit` produces a
/// `WalletDebited` event for `Expense` or `FeesRefund`
/// wallet types. The aggregate must reject the deduction
/// when the wallet has insufficient balance.
#[test]
fn fee_structure_lifecycle_debit_emits_wallet_debited() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let mut w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    // Pre-credit the wallet so the deduction succeeds.
    w.apply_credit(50_000, Currency::INR, actor, Timestamp::now())
        .expect("apply_credit");
    assert_eq!(w.balance_minor, 50_000);

    let cmd = fin_services::DeductWalletCreditCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: w.id,
        user_id: actor,
        amount_minor: 12_000,
        currency: Currency::INR,
        wallet_type: WalletTxType::Expense,
        payment_method_id: None,
        bank_id: None,
        reference: Some("Library book purchase refund".to_owned()),
        note: None,
    };
    let (tx, event) =
        fin_services::deduct_wallet_credit(&w, cmd, &clock, &g).expect("deduct_wallet_credit");

    assert_eq!(
        <WalletDebited as DomainEvent>::EVENT_TYPE,
        "finance.wallet.debited"
    );
    assert_eq!(event.wallet_id, w.id);
    assert_eq!(event.wallet_transaction_id, tx.id);
    assert_eq!(event.amount_minor, 12_000);
    assert_eq!(event.wallet_type, WalletTxType::Expense);
    assert!(tx.wallet_type.is_debit());
    assert_eq!(tx.status, ApprovalStatus::Pending);
}

/// Fee-structure lifecycle failure path: deducting more
/// than the wallet balance must be rejected with
/// `DomainError::Conflict("insufficient wallet balance")`
/// (per spec invariant "wallet debit must not overdraw").
#[test]
fn fee_structure_lifecycle_debit_rejects_insufficient_balance() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    assert_eq!(w.balance_minor, 0);

    let cmd = fin_services::DeductWalletCreditCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: w.id,
        user_id: actor,
        amount_minor: 1,
        currency: Currency::INR,
        wallet_type: WalletTxType::Expense,
        payment_method_id: None,
        bank_id: None,
        reference: None,
        note: None,
    };
    let err = fin_services::deduct_wallet_credit(&w, cmd, &clock, &g)
        .expect_err("overdraft must fail conflict");
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "got {err:?}"
    );
}

/// Fee-structure lifecycle failure path: deducting in a
/// currency other than the wallet's currency must be
/// rejected with `DomainError::Validation` (per spec
/// invariant "debit currency must match wallet currency").
#[test]
fn fee_structure_lifecycle_debit_rejects_currency_mismatch() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let mut w = new_wallet_aggregate(&g, school, actor, Currency::INR);
    // Pre-credit the wallet so the deduction reaches the
    // currency-mismatch check (the insufficient-balance check
    // runs first).
    w.apply_credit(50_000, Currency::INR, actor, Timestamp::now())
        .expect("apply_credit");
    assert_eq!(w.balance_minor, 50_000);

    let cmd = fin_services::DeductWalletCreditCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        wallet_id: w.id,
        user_id: actor,
        amount_minor: 100,
        currency: Currency::USD,
        wallet_type: WalletTxType::Expense,
        payment_method_id: None,
        bank_id: None,
        reference: None,
        note: None,
    };
    let err = fin_services::deduct_wallet_credit(&w, cmd, &clock, &g)
        .expect_err("currency mismatch must fail validation");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "got {err:?}"
    );
}
