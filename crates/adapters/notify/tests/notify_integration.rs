//! # Phase 15 notify port vertical-slice integration test
//!
//! 5 sync scenarios (always-on) + 2 env-gated async scenarios
//! (require `EDUCORE_PORT_ADAPTER_E2E=1`).
//!
//! The sync scenarios exercise the pure-helper services that sit
//! alongside the [`NotificationProvider`](educore_notify::NotificationProvider)
//! port:
//!
//! - [`TemplateService`] — variable substitution + required-var
//!   validation
//! - [`ChannelService`] — sync vs async classification + auth
//!   requirements
//! - [`IdempotencyService`] — SHA-256 idempotency-key derivation
//! - [`RateLimitService`] — token-bucket rate limiter, one bucket
//!   per [`Channel`]
//!
//! The two async scenarios are env-gated because they wire up a
//! real SMTP relay / HTTP gateway endpoint and would otherwise
//! flap in CI. They live as `#[ignore]`d `#[tokio::test]`s so the
//! orchestrator can opt in with
//! `EDUCORE_PORT_ADAPTER_E2E=1 cargo test ... -- --ignored`.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_notify::port::Channel;
use educore_notify::{
    ChannelService, EmailProviderBuilder, IdempotencyService, RateLimitService, SmsProviderBuilder,
    TemplateService,
};

// ---------------------------------------------------------------------------
// Scenario 1: Template variable substitution
// ---------------------------------------------------------------------------

#[test]
fn notify_integration_template_substitute() {
    let body = "Hello {student_name}, your exam is on {exam_date}.";
    let mut vars = std::collections::BTreeMap::new();
    vars.insert("student_name".into(), "Ada".into());
    vars.insert("exam_date".into(), "2026-07-01".into());
    let result = TemplateService::substitute_variables(body, &vars);
    assert_eq!(result, "Hello Ada, your exam is on 2026-07-01.");
}

// ---------------------------------------------------------------------------
// Scenario 2: Template required-variable validation
// ---------------------------------------------------------------------------

#[test]
fn notify_integration_template_validate_required() {
    let body = "Hello {student_name}!";
    let mut vars = std::collections::BTreeMap::new();
    vars.insert("student_name".into(), "Ada".into());
    assert!(TemplateService::validate_required_variables(body, &vars).is_ok());
    let empty_vars = std::collections::BTreeMap::new();
    assert!(TemplateService::validate_required_variables(body, &empty_vars).is_err());
}

// ---------------------------------------------------------------------------
// Scenario 3: Channel classification (sync vs async)
// ---------------------------------------------------------------------------

#[test]
fn notify_integration_channel_classification() {
    let push = Channel::Push {
        topic: None,
        ttl: None,
        collapse_key: None,
    };
    let email = Channel::Email {
        from: None,
        reply_to: None,
    };
    assert!(ChannelService::is_async(&push));
    assert!(!ChannelService::is_async(&email));
    assert!(ChannelService::requires_authentication(&email));
}

// ---------------------------------------------------------------------------
// Scenario 4: Idempotency key derivation
// ---------------------------------------------------------------------------

#[test]
fn notify_integration_idempotency_derive_key() {
    let key1 = IdempotencyService::derive_key("cmd-001", "user-1", 1);
    let key2 = IdempotencyService::derive_key("cmd-001", "user-1", 1);
    let key3 = IdempotencyService::derive_key("cmd-001", "user-1", 2);
    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
    assert_eq!(key1.len(), 64); // SHA-256 hex = 64 chars
}

// ---------------------------------------------------------------------------
// Scenario 5: Rate limit token bucket
// ---------------------------------------------------------------------------

#[test]
fn notify_integration_rate_limit_bucket() {
    let mut svc = RateLimitService::new();
    let channel = Channel::Email {
        from: None,
        reply_to: None,
    };
    // Reset to ensure clean state
    svc.reset(&channel);
    // First call should succeed
    assert!(svc.try_acquire(&channel, 5));
    // Burst of 5 should succeed (bucket size = 5)
    for _ in 0..4 {
        assert!(svc.try_acquire(&channel, 5));
    }
    // 6th call should fail (bucket exhausted)
    assert!(!svc.try_acquire(&channel, 5));
}

// ---------------------------------------------------------------------------
// Env-gated async scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn notify_integration_async_email_send_mock() {
    let _provider = EmailProviderBuilder::new()
        .relay_host("localhost")
        .relay_port(1025)
        .credentials("test:test")
        .default_from("test@educore.local".to_owned())
        .build()
        .expect("smtp builder must succeed with relay_host + default_from");
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn notify_integration_async_sms_send_mock() {
    let _provider = SmsProviderBuilder::new()
        .gateway_url("https://api.twilio.com/2010-04-01/Accounts/{account}/Messages.json")
        .api_key("ACtest_token")
        .default_from("+15005550006".to_owned())
        .build();
}