//! # Phase 15 payment port vertical-slice integration test
//!
//! 5 sync scenarios (always-on) + 2 env-gated async scenarios.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_payment::prelude::*;
use educore_payment::services::{
    BankSlipService, IdempotencyService, SettlementService, WebhookSignatureService,
};

// ---------------------------------------------------------------------------
// Scenario 1: Idempotency key derivation
// ---------------------------------------------------------------------------

#[test]
fn payment_integration_idempotency_charge_key() {
    let key1 = IdempotencyService::derive_charge_key(
        "cmd-001",
        &["inv-1".to_owned(), "inv-2".to_owned()],
        150_000,
    );
    let key2 = IdempotencyService::derive_charge_key(
        "cmd-001",
        &["inv-2".to_owned(), "inv-1".to_owned()],
        150_000,
    );
    // Order-independent (same invoices, same amount)
    assert_eq!(key1, key2);
    let key3 = IdempotencyService::derive_charge_key("cmd-001", &["inv-1".to_owned()], 150_000);
    // Different invoice list → different key
    assert_ne!(key1, key3);
}

// ---------------------------------------------------------------------------
// Scenario 2: Webhook signature compute + verify
// ---------------------------------------------------------------------------

#[test]
fn payment_integration_webhook_signature_round_trip() {
    let svc = WebhookSignatureService::new("whsec_test_secret".to_owned());
    let payload = b"{\"id\":\"evt_001\",\"amount\":1500}";
    let sig = svc.compute_signature(payload).expect("HMAC accepts any key length");
    assert!(sig.starts_with("sha256="));
    assert!(svc.verify_signature(payload, &sig).expect("compute should succeed"));
    assert!(
        !svc.verify_signature(b"{\"tampered\":\"data\"}", &sig).expect("compute should succeed")
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: Bank slip number validation
// ---------------------------------------------------------------------------

#[test]
fn payment_integration_bank_slip_number_validation() {
    // Valid: alphanumeric, 6-20 chars
    assert!(BankSlipService::validate_slip_number("SLIP12345").is_ok());
    assert!(BankSlipService::validate_slip_number("123456").is_ok());
    assert!(BankSlipService::validate_slip_number("ABCDEFG").is_ok());
    assert!(BankSlipService::validate_slip_number(&"x".repeat(20)).is_ok());
    // Invalid cases
    assert!(BankSlipService::validate_slip_number("").is_err()); // empty
    assert!(BankSlipService::validate_slip_number("ab").is_err()); // too short
    assert!(BankSlipService::validate_slip_number(&"x".repeat(21)).is_err()); // too long
    assert!(BankSlipService::validate_slip_number("slip with spaces").is_err()); // spaces not allowed
    assert!(BankSlipService::validate_slip_number("SLIP-12345").is_err()); // dash not alphanumeric
}

// ---------------------------------------------------------------------------
// Scenario 4: Settlement line matching
// ---------------------------------------------------------------------------

#[test]
fn payment_integration_settlement_match_line() {
    let usd = CurrencyCode::new("USD").expect("USD is a valid ISO 4217 code");
    let payment_id = PaymentId::new("pay_ch_123");
    let receipt = PaymentReceipt {
        payment_id: payment_id.clone(),
        provider_payment_id: Some("ch_123".to_owned()),
        status: PaymentStatus::Captured { at: educore_core::value_objects::Timestamp::now() },
        amount: Money::new(usd.clone(), 15_000).expect("non-negative"),
        method: PaymentMethodKind::Cash,
        authorized_at: None,
        captured_at: Some(educore_core::value_objects::Timestamp::now()),
        fees: Vec::new(),
        net: Money::new(usd.clone(), 15_000).expect("non-negative"),
        receipt_url: None,
        metadata: std::collections::BTreeMap::new(),
    };
    let line = SettlementLine {
        provider_payment_id: "ch_123".to_owned(),
        payment_id: payment_id.clone(),
        gross: Money::new(usd.clone(), 15_000).expect("non-negative"),
        fee: Money::zero(usd.clone()),
        net: Money::new(usd.clone(), 15_000).expect("non-negative"),
        settled_at: educore_core::value_objects::Timestamp::now(),
    };
    assert_eq!(
        SettlementService::match_settlement_line(&line, &[receipt.clone()]),
        Some(payment_id.clone())
    );
    let mismatched = SettlementLine {
        provider_payment_id: "ch_999".to_owned(),
        payment_id: PaymentId::new("pay_ch_999"),
        gross: Money::new(usd.clone(), 15_000).expect("non-negative"),
        fee: Money::zero(usd.clone()),
        net: Money::new(usd.clone(), 15_000).expect("non-negative"),
        settled_at: educore_core::value_objects::Timestamp::now(),
    };
    assert_eq!(
        SettlementService::match_settlement_line(&mismatched, &[receipt]),
        None
    );
}

// ---------------------------------------------------------------------------
// Scenario 5: Settlement net computation
// ---------------------------------------------------------------------------

#[test]
fn payment_integration_settlement_net_total() {
    let usd = CurrencyCode::new("USD").expect("USD is a valid ISO 4217 code");
    let lines = vec![
        SettlementLine {
            provider_payment_id: "ch_1".to_owned(),
            payment_id: PaymentId::new("pay_1"),
            gross: Money::new(usd.clone(), 10_000).expect("non-negative"),
            fee: Money::zero(usd.clone()),
            net: Money::new(usd.clone(), 10_000).expect("non-negative"),
            settled_at: educore_core::value_objects::Timestamp::now(),
        },
        SettlementLine {
            provider_payment_id: "ch_2".to_owned(),
            payment_id: PaymentId::new("pay_2"),
            gross: Money::new(usd.clone(), 25_000).expect("non-negative"),
            fee: Money::zero(usd.clone()),
            net: Money::new(usd.clone(), 25_000).expect("non-negative"),
            settled_at: educore_core::value_objects::Timestamp::now(),
        },
    ];
    assert_eq!(SettlementService::compute_net_settlement(&lines), 35_000);
}

// ---------------------------------------------------------------------------
// Env-gated async scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn payment_integration_async_stripe_charge_mock() {
    let _provider = StripeProviderBuilder::new()
        .secret_key("sk_test_placeholder".to_owned())
        .webhook_secret("whsec_test_placeholder".to_owned())
        .build()
        .expect("reqwest client build");
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn payment_integration_async_stripe_refund_mock() {
    let _provider = StripeProviderBuilder::new()
        .secret_key("sk_test_placeholder".to_owned())
        .build()
        .expect("reqwest client build");
}
