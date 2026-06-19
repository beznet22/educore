//! # Phase 15 Auth caps round-trip test
//!
//! Verifies the 13 Auth Capability variants declared in microtask A.1
//! are valid: each variant round-trips through `as_str()` and the
//! parser (whichever name it has in this crate).

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_rbac::value_objects::Capability;

const AUTH_VARIANTS: &[Capability] = &[
    Capability::AuthLogin,
    Capability::AuthLogout,
    Capability::AuthRefresh,
    Capability::AuthRevoke,
    Capability::AuthPasswordReset,
    Capability::OAuthAccessTokenRead,
    Capability::OAuthAccessTokenRevoke,
    Capability::OAuthClientRead,
    Capability::OAuthClientManage,
    Capability::PasswordResetRequest,
    Capability::PasswordResetConfirm,
    Capability::MfaEnroll,
    Capability::MfaVerify,
];

#[test]
fn auth_capabilities_round_trip() {
    assert_eq!(AUTH_VARIANTS.len(), 13);
    for cap in AUTH_VARIANTS {
        let wire = cap.as_str();
        assert!(
            wire.starts_with("Auth.")
                || wire.starts_with("OAuth.")
                || wire.starts_with("PasswordReset.")
                || wire.starts_with("Mfa."),
            "{cap:?}.as_str() = {wire} should start with Auth./OAuth./PasswordReset./Mfa."
        );
        let parsed = Capability::from_str_opt(wire)
            .or_else(|| wire.parse::<Capability>().ok())
            .unwrap_or_else(|| panic!("failed to parse {wire}"));
        assert_eq!(parsed, *cap, "round-trip mismatch for {cap:?}");
    }
}
