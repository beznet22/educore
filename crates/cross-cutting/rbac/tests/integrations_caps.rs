//! # Phase 15 Integrations caps round-trip test

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_rbac::value_objects::Capability;

const INTEGRATIONS_VARIANTS: &[Capability] = &[
    Capability::IntegrationInvoke,
    Capability::IntegrationListCapabilities,
    Capability::IntegrationHealth,
    Capability::IntegrationConfigure,
    Capability::WebhookOut,
    Capability::PollingIn,
    Capability::LmsRosterSync,
    Capability::VideoSchedule,
];

#[test]
fn integrations_capabilities_round_trip() {
    assert_eq!(INTEGRATIONS_VARIANTS.len(), 8);
    for cap in INTEGRATIONS_VARIANTS {
        let wire = cap.as_str();
        assert!(
            wire.starts_with("Integration.")
                || wire.starts_with("Webhook")
                || wire.starts_with("Polling")
                || wire.starts_with("Lms")
                || wire.starts_with("Video"),
            "{cap:?}.as_str() = {wire} should start with Integration./Webhook/Polling/Lms/Video"
        );
        let parsed = Capability::from_str_opt(wire)
            .or_else(|| wire.parse::<Capability>().ok())
            .unwrap_or_else(|| panic!("failed to parse {wire}"));
        assert_eq!(parsed, *cap, "round-trip mismatch for {cap:?}");
    }
}
