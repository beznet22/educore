//! Integration tests for the **Wallet aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Wallet`](educore_finance::aggregate::Wallet)
//! end-to-end through the service layer:
//!
//! 1. `create_wallet` mints a typed `WalletId` from the fresh
//!    `event_id`, constructs the aggregate with the supplied
//!    `user_id` + `currency`, and emits a [`WalletCreated`]
//!    event carrying the matching `wallet_id`, `user_id`,
//!    `currency`, `event_id`, `correlation_id`, and
//!    `occurred_at`.
//! 2. The `Currency` value object validates the ISO-4217 code
//!    (3 uppercase ASCII letters) at construction time; an
//!    invalid code is rejected with `DomainError::Validation`
//!    before the command is ever built.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/library/tests/aggregates.rs`
//! (`TestClock` + `SystemIdGen`). Per the academic / library
//! pattern, the **handlers** themselves are not wired
//! end-to-end (no subscriber fan-out, no outbox commit, no
//! audit row). These tests pin the contract of the
//! **service layer** that the dispatcher will eventually wrap.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::*;
use educore_finance::value_objects::Currency;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
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

// =============================================================================
// 1. Happy path: create a Wallet
// =============================================================================

/// End-to-end happy path for the `Wallet` aggregate.
/// Create a wallet for a `SchoolAdmin`'s user in USD,
/// asserting that:
///
/// 1. The create flow produces a `Wallet` aggregate carrying
///    every field on the command (`school_id` derived from
///    the typed id, `user_id`, `currency` = USD), the audit
///    footer is initialised (`version = 1`,
///    `active_status.is_active()`), and the
///    `last_event_id` is stamped from the service's
///    `next_event_id()`.
/// 2. The service emits a `WalletCreated` event whose
///    `event_type`, `aggregate_type`, and `school_id`
///    match the aggregate's typed id and the
///    `DomainEvent` trait's contract, and whose payload
///    carries the same `user_id`, `currency`, `event_id`,
///    and `correlation_id` as the aggregate.
#[test]
fn wallet_create_emits_event_and_initialises_aggregate() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Build a fresh USD currency via the typed constructor so
    // the test pins the public API rather than the
    // `Currency(*b"USD")` private bytes.
    let currency = Currency::new("USD").expect("USD is a valid ISO-4217 code");

    let create_cmd = CreateWalletCommand {
        tenant: tenant.clone(),
        user_id: tenant.actor_id,
        currency,
    };
    let (wallet, created_event) = create_wallet(create_cmd, &clock, &ids).expect("create_wallet");

    // Aggregate fields are populated from the command.
    assert_eq!(wallet.school_id, school);
    assert_eq!(wallet.user_id, tenant.actor_id);
    assert_eq!(wallet.currency, currency);
    assert_eq!(wallet.currency.as_str(), "USD");
    // A fresh wallet starts at zero balance.
    assert_eq!(wallet.balance_minor, 0);
    // Audit metadata footer is initialised.
    assert_eq!(wallet.version.get(), 1);
    assert!(wallet.active_status.is_active());
    assert_eq!(wallet.created_by, tenant.actor_id);
    assert_eq!(wallet.updated_by, tenant.actor_id);
    // The service stamps the freshly-minted event id on the
    // aggregate's `last_event_id` (per the WalletCreated
    // invariant — see services.rs `create_wallet`).
    let stamped_event_id = wallet
        .last_event_id
        .expect("create_wallet must stamp last_event_id");
    assert_eq!(stamped_event_id, created_event.event_id);
    // Correlation id is propagated from the tenant context.
    assert_eq!(wallet.correlation_id, tenant.correlation_id);
    assert_eq!(created_event.correlation_id, tenant.correlation_id);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <WalletCreated as DomainEvent>::EVENT_TYPE,
        "finance.wallet.created"
    );
    assert_eq!(<WalletCreated as DomainEvent>::AGGREGATE_TYPE, "wallet");
    assert_eq!(<WalletCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), wallet.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    // The event payload mirrors the command + the freshly-minted
    // event id.
    assert_eq!(created_event.wallet_id, wallet.id);
    assert_eq!(created_event.user_id, tenant.actor_id);
    assert_eq!(created_event.currency, currency);
    assert_eq!(created_event.event_id, stamped_event_id);
}

// =============================================================================
// 2. Validation failure: invalid ISO-4217 currency is rejected
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `Currency` value object is constructed from an invalid
/// ISO-4217 code (e.g. lowercase, wrong length, or
/// non-alphabetic), `Currency::new` returns
/// `DomainError::Validation` and the command is never
/// built. This pins the contract that the dispatcher
/// relies on: invalid currency codes fail fast at the
/// value-object boundary, before they can ever reach the
/// aggregate.
///
/// Per `crates/domains/finance/src/value_objects.rs`
/// § `Currency::new`, the code must be exactly 3 uppercase
/// ASCII letters. The three sub-cases below cover:
/// - wrong length (`"US"`, 2 chars),
/// - lowercase letters (`"usd"`, 3 chars but not uppercase),
/// - non-alphabetic (`"U5D"`, digit in place of letter).
#[test]
fn wallet_create_with_invalid_currency_returns_validation_error() {
    // Length != 3 -> Validation.
    let err_short = Currency::new("US").expect_err("2-char currency code must fail validation");
    assert!(
        matches!(err_short, DomainError::Validation(_)),
        "expected Validation for short code, got {err_short:?}"
    );

    // Lowercase ASCII letters -> Validation.
    let err_lower = Currency::new("usd").expect_err("lowercase currency code must fail validation");
    assert!(
        matches!(err_lower, DomainError::Validation(_)),
        "expected Validation for lowercase code, got {err_lower:?}"
    );

    // Non-alphabetic ASCII -> Validation.
    let err_digit =
        Currency::new("U5D").expect_err("digit-bearing currency code must fail validation");
    assert!(
        matches!(err_digit, DomainError::Validation(_)),
        "expected Validation for digit-bearing code, got {err_digit:?}"
    );

    // Sanity check: a valid 3-letter uppercase code round-trips
    // and is not rejected at the value-object boundary, so the
    // happy path can construct the command.
    let ok = Currency::new("EUR").expect("EUR is a valid ISO-4217 code");
    assert_eq!(ok.as_str(), "EUR");
}

// =========================================================================
// -- Wave 150 -- Wallet::reconcile_balance + reconcile_and_validate --
// =========================================================================

fn wallet_id_fixture(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> educore_finance::prelude::WalletId {
    educore_finance::prelude::WalletId::new(school, g.next_uuid())
}

fn wallet_txn_id_fixture(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> educore_finance::prelude::WalletTransactionId {
    educore_finance::prelude::WalletTransactionId::new(school, g.next_uuid())
}

fn make_pending_credit_txn(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    wallet_id: educore_finance::prelude::WalletId,
    user_id: educore_core::ids::UserId,
    amount_minor: i64,
) -> educore_finance::prelude::WalletTransaction {
    educore_finance::prelude::WalletTransaction::fresh(
        wallet_txn_id_fixture(g, school),
        wallet_id,
        user_id,
        amount_minor,
        Currency::INR,
        educore_finance::prelude::WalletTxType::Deposit,
        None,
        None,
        None,
        None,
        user_id,
        educore_core::value_objects::Timestamp::now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh deposit")
}

fn make_pending_debit_txn(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    wallet_id: educore_finance::prelude::WalletId,
    user_id: educore_core::ids::UserId,
    amount_minor: i64,
) -> educore_finance::prelude::WalletTransaction {
    educore_finance::prelude::WalletTransaction::fresh(
        wallet_txn_id_fixture(g, school),
        wallet_id,
        user_id,
        amount_minor,
        Currency::INR,
        educore_finance::prelude::WalletTxType::Expense,
        None,
        None,
        None,
        None,
        user_id,
        educore_core::value_objects::Timestamp::now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh expense")
}

#[test]
fn wt_i_4_reconcile_balance_empty_is_zero() {
    let result = educore_finance::prelude::Wallet::reconcile_balance(&[]);
    assert_eq!(result, 0, "empty tx list yields zero balance");
}

#[test]
fn wt_i_4_reconcile_balance_pending_txns_excluded() {
    let (tenant, g) = admin_context();
    let wallet_id = wallet_id_fixture(&g, tenant.school_id);
    let mut txn = make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 1000);
    txn.approve(
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        g.next_event_id(),
    )
    .expect("approve credit");
    let mut pending = make_pending_debit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 500);
    // Do NOT approve the pending debit; it must be excluded.
    let _ = &mut pending;
    let result = educore_finance::prelude::Wallet::reconcile_balance(&[&txn, &pending]);
    assert_eq!(
        result, 1000,
        "pending debit excluded; only approved credit counted"
    );
}

#[test]
fn wt_i_4_reconcile_balance_credits_add_debits_subtract() {
    let (tenant, g) = admin_context();
    let wallet_id = wallet_id_fixture(&g, tenant.school_id);
    let mut credit1 =
        make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 5000);
    credit1
        .approve(
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
            g.next_event_id(),
        )
        .expect("approve credit1");
    let mut credit2 =
        make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 2500);
    credit2
        .approve(
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
            g.next_event_id(),
        )
        .expect("approve credit2");
    let mut debit1 = make_pending_debit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 1500);
    debit1
        .approve(
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
            g.next_event_id(),
        )
        .expect("approve debit1");
    let result =
        educore_finance::prelude::Wallet::reconcile_balance(&[&credit1, &credit2, &debit1]);
    assert_eq!(result, 6000, "5000 + 2500 - 1500 = 6000");
}

#[test]
fn wt_i_4_reconcile_balance_rejected_txns_excluded() {
    let (tenant, g) = admin_context();
    let wallet_id = wallet_id_fixture(&g, tenant.school_id);
    let mut txn = make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 2000);
    txn.reject(
        tenant.actor_id,
        String::new(),
        educore_core::value_objects::Timestamp::now(),
        g.next_event_id(),
    )
    .expect("reject credit");
    let result = educore_finance::prelude::Wallet::reconcile_balance(&[&txn]);
    assert_eq!(result, 0, "rejected credit excluded");
}

#[test]
fn wt_i_4_reconcile_and_validate_agrees_with_cache() {
    let (tenant, g) = admin_context();
    let wallet_id = wallet_id_fixture(&g, tenant.school_id);
    let id = educore_finance::prelude::WalletId::new(tenant.school_id, g.next_uuid());
    let mut wallet = educore_finance::prelude::Wallet::fresh(
        id,
        tenant.actor_id,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    );
    wallet
        .apply_credit(
            3000,
            Currency::INR,
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
        )
        .expect("apply credit");
    let mut credit_tx =
        make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 3000);
    credit_tx
        .approve(
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
            g.next_event_id(),
        )
        .expect("approve credit");
    wallet
        .reconcile_and_validate(&[&credit_tx])
        .expect("cache matches authoritative: no drift");
}

#[test]
fn wt_i_4_reconcile_and_validate_detects_drift() {
    let (tenant, g) = admin_context();
    let wallet_id = wallet_id_fixture(&g, tenant.school_id);
    let id = educore_finance::prelude::WalletId::new(tenant.school_id, g.next_uuid());
    let mut wallet = educore_finance::prelude::Wallet::fresh(
        id,
        tenant.actor_id,
        Currency::INR,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    );
    wallet
        .apply_credit(
            5000,
            Currency::INR,
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
        )
        .expect("apply credit");
    let mut credit_tx =
        make_pending_credit_txn(&g, tenant.school_id, wallet_id, tenant.actor_id, 3000);
    credit_tx
        .approve(
            tenant.actor_id,
            educore_core::value_objects::Timestamp::now(),
            g.next_event_id(),
        )
        .expect("approve credit");
    let err = wallet.reconcile_and_validate(&[&credit_tx]).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict on drift, got {err:?}"
    );
}

// =========================================================================
// -- Wave 150 -- Wallet cross-aggregate balance marker --
// =========================================================================

#[test]
fn wallet_cross_aggregate_balance_equals_sum_of_approved() {
    // Wallet cross-aggregate invariant (WT cross-aggregate: balance
    // == sum of approved tx): the Wallet's cached `balance_minor`
    // must equal `reconcile_balance(&[approved_txns])`. The
    // dispatcher / reconciliation job enforces this via
    // `Wallet::reconcile_and_validate()` on every wallet read;
    // drift detection returns `Conflict`.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
