//! # Phase 15 Payment caps round-trip test

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_rbac::value_objects::Capability;

const PAYMENT_VARIANTS: &[Capability] = &[
    Capability::PaymentCharge,
    Capability::PaymentRefund,
    Capability::PaymentStatus,
    Capability::PaymentMethodList,
    Capability::PaymentWebhook,
    Capability::PaymentSettlement,
    Capability::BankSlipGenerate,
    Capability::BankSlipApprove,
];

#[test]
fn payment_capabilities_round_trip() {
    assert_eq!(PAYMENT_VARIANTS.len(), 8);
    for cap in PAYMENT_VARIANTS {
        let wire = cap.as_str();
        assert!(
            wire.starts_with("Payment.") || wire.starts_with("BankSlip."),
            "{cap:?}.as_str() = {wire} should start with Payment. or BankSlip."
        );
        let parsed = Capability::from_str_opt(wire)
            .or_else(|| wire.parse::<Capability>().ok())
            .unwrap_or_else(|| panic!("failed to parse {wire}"));
        assert_eq!(parsed, *cap, "round-trip mismatch for {cap:?}");
    }
}
