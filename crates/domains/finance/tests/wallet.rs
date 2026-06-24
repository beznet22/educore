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
    let (wallet, created_event) =
        create_wallet(create_cmd, &clock, &ids).expect("create_wallet");

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
    assert_eq!(
        <WalletCreated as DomainEvent>::AGGREGATE_TYPE,
        "wallet"
    );
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
    let err_short = Currency::new("US")
        .expect_err("2-char currency code must fail validation");
    assert!(
        matches!(err_short, DomainError::Validation(_)),
        "expected Validation for short code, got {err_short:?}"
    );

    // Lowercase ASCII letters -> Validation.
    let err_lower = Currency::new("usd")
        .expect_err("lowercase currency code must fail validation");
    assert!(
        matches!(err_lower, DomainError::Validation(_)),
        "expected Validation for lowercase code, got {err_lower:?}"
    );

    // Non-alphabetic ASCII -> Validation.
    let err_digit = Currency::new("U5D")
        .expect_err("digit-bearing currency code must fail validation");
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
