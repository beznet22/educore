//! # Phase 15 Notify caps round-trip test
//!
//! Verifies the 9 Notify Capability variants declared in microtask B.1
//! are valid: each variant round-trips through `as_str()` and
//! `from_str_opt` (or `FromStr::from_str` if that's the API).

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_rbac::value_objects::Capability;

const NOTIFY_VARIANTS: &[Capability] = &[
    Capability::NotifyEmailSend,
    Capability::NotifySmsSend,
    Capability::NotifyPushSend,
    Capability::NotifyInApp,
    Capability::NotifyVoice,
    Capability::NotifyWebhook,
    Capability::NotifyTemplateRead,
    Capability::NotifyTemplateWrite,
    Capability::NotifyBulkSend,
];

#[test]
fn notify_capabilities_round_trip() {
    assert_eq!(NOTIFY_VARIANTS.len(), 9);
    for cap in NOTIFY_VARIANTS {
        let wire = cap.as_str();
        assert!(
            wire.starts_with("Notify."),
            "{cap:?}.as_str() = {wire} should start with Notify."
        );
        let parsed = Capability::from_str_opt(wire)
            .or_else(|| wire.parse::<Capability>().ok())
            .unwrap_or_else(|| panic!("failed to parse {wire}"));
        assert_eq!(parsed, *cap, "round-trip mismatch for {cap:?}");
    }
}
