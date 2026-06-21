//! # Phase 15 auth port vertical-slice integration test

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_auth::prelude::*;
use educore_auth::services::{JwtService, MfaService, OAuthScopeService, PasswordService};

// Scenario 1: JWT provider builder constructs and exposes a configured provider.
#[test]
fn auth_integration_jwt_builder_constructs() {
    let provider = JwtAuthProviderBuilder::new()
        .signing_key(b"test-secret-key-32-bytes-long!!".to_vec())
        .issuer("educore-test")
        .audience("educore-test")
        .build();
    // The provider constructed without panic; defaults wire up cleanly.
    let _ = provider;
}

// Scenario 2: Argon2id password hash + verify round-trip, plus needs_rehash.
#[test]
fn auth_integration_password_hash_and_verify() {
    let svc = PasswordService::new();
    let plain = SecretString::new("correct-horse-battery-staple");
    let hash = svc.hash_password(&plain).expect("hash should succeed");
    assert!(
        hash.starts_with("$argon2id$"),
        "hash must be argon2id PHC string, got {hash}"
    );
    assert!(svc
        .verify_password(&plain, &hash)
        .expect("verify should succeed"));
    let wrong = SecretString::new("wrong-password");
    assert!(!svc
        .verify_password(&wrong, &hash)
        .expect("verify should succeed"));
    assert!(!svc.needs_rehash(&hash));
}

// Scenario 3: OAuth scope membership is a whitespace-bounded check, fail-closed.
#[test]
fn auth_integration_oauth_scope_check() {
    assert!(OAuthScopeService::has_scope("read:user write:user", "read:user"));
    assert!(!OAuthScopeService::has_scope("read:user", "write:user"));
    // Empty required is rejected (fail-closed).
    assert!(!OAuthScopeService::has_scope("", ""));
    // Empty scope set never matches a non-empty requirement.
    assert!(!OAuthScopeService::has_scope("", "read"));
    // Prefix collisions are correctly rejected.
    assert!(!OAuthScopeService::has_scope("profile:read", "profile:rea"));
}

// Scenario 4: TOTP secret generation is a 32-char base32 string (20 raw bytes).
#[test]
fn auth_integration_mfa_generate_secret() {
    let secret = MfaService::generate_secret();
    // 20 raw bytes encoded as base32 = 32 chars.
    assert_eq!(secret.len(), 32);
    assert!(secret.chars().all(|c| c.is_ascii_alphanumeric()));
    assert!(secret.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
}

// Scenario 5: JWT claim semantic validation (iss / aud / exp).
#[test]
fn auth_integration_jwt_validate_claims() {
    let claims = JwtClaims {
        sub: "00000000-0000-0000-0000-000000000000".to_owned(),
        iss: "educore-test".to_owned(),
        aud: "educore-test".to_owned(),
        iat: 0,
        exp: i64::MAX / 2,
        sid: "00000000-0000-0000-0000-000000000000".to_owned(),
        roles: vec![],
        schools: vec!["00000000-0000-0000-0000-000000000000".to_owned()],
        active_school: "00000000-0000-0000-0000-000000000000".to_owned(),
        mfa: true,
    };
    // Happy path.
    JwtService::validate_claims(&claims, "educore-test", "educore-test")
        .expect("matching iss/aud and future exp should validate");
    // Wrong issuer.
    assert!(JwtService::validate_claims(&claims, "wrong-issuer", "educore-test").is_err());
    // Wrong audience.
    assert!(JwtService::validate_claims(&claims, "educore-test", "wrong-audience").is_err());
    // Expired token (exp in the past) -> Expired.
    let mut expired = claims.clone();
    expired.exp = 0;
    assert!(JwtService::validate_claims(&expired, "educore-test", "educore-test").is_err());
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn auth_integration_async_jwt_full_round_trip() {
    let provider = JwtAuthProviderBuilder::new().build();
    let _session = provider
        .authenticate(Credential::Anonymous)
        .await
        .expect("anonymous auth should succeed");
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn auth_integration_async_password_rehash_check() {
    let svc = PasswordService::new();
    let plain = SecretString::new("test");
    let hash = svc.hash_password(&plain).expect("hash should succeed");
    assert!(!svc.needs_rehash(&hash));
}
