//! # Phase 15 integrations port vertical-slice integration test (parity)
//!
//! 5 sync scenarios (always-on) + 2 env-gated async scenarios.
//! Mirrors
//! `crates/adapters/integrations/tests/integrations_integration.rs`
//! so the parity suite runs the same shape across all five
//! port adapters. The async scenarios exercise the
//! [`LmsIntegration`](educore_integrations::lms::LmsIntegration)
//! and
//! [`WebhookOutIntegration`](educore_integrations::webhook_out::WebhookOutIntegration)
//! reference impl builders.

#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::sync::Arc;

use educore_integrations::prelude::*;
use educore_integrations::services::{
    PollingService, RateLimitService, RetryService, WebhookSignatureService,
};
use educore_testkit::integrations::InMemoryIntegrationGateway;

// Scenario 1: Webhook signature HMAC-SHA256 (associated function)
#[test]
fn port_integrations_webhook_signature() {
    let secret = "whsec_test_secret";
    let payload = b"{\"event\":\"invoice.paid\"}";
    let sig = WebhookSignatureService::compute_signature(secret, payload).expect("HMAC succeeds");
    assert!(sig.starts_with("sha256="));
    assert!(
        WebhookSignatureService::verify_signature(secret, payload, &sig).expect("verify succeeds")
    );
    assert!(
        !WebhookSignatureService::verify_signature(secret, b"{\"tampered\":true}", &sig)
            .expect("verify succeeds")
    );
    let _svc = WebhookSignatureService;
}

// Scenario 2: Retry exponential backoff
#[test]
fn port_integrations_retry_exponential() {
    let policy = RetryPolicy::Exponential {
        max_retries: 5,
        base: chrono::Duration::seconds(1),
        max: chrono::Duration::seconds(300),
    };
    let backoff1 = RetryService::next_backoff(&policy, 1).expect("attempt 1");
    let backoff2 = RetryService::next_backoff(&policy, 2).expect("attempt 2");
    let backoff3 = RetryService::next_backoff(&policy, 3).expect("attempt 3");
    assert_eq!(backoff1, std::time::Duration::from_secs(2));
    assert_eq!(backoff2, std::time::Duration::from_secs(4));
    assert_eq!(backoff3, std::time::Duration::from_secs(8));
}

// Scenario 3: Retry classification (4xx permanent, 5xx transient)
#[test]
fn port_integrations_retry_classification() {
    assert!(RetryService::is_permanent_failure(400));
    assert!(RetryService::is_permanent_failure(404));
    assert!(!RetryService::is_permanent_failure(408));
    assert!(!RetryService::is_permanent_failure(429));
    assert!(!RetryService::is_permanent_failure(500));
    assert!(!RetryService::is_permanent_failure(503));
    assert!(!RetryService::is_permanent_failure(200));
    assert!(!RetryService::is_permanent_failure(302));
}

// Scenario 4: Polling schedule
#[test]
fn port_integrations_polling_schedule() {
    let now = educore_core::value_objects::Timestamp::now();
    let one_hour_ago = educore_core::value_objects::Timestamp::from_datetime(
        chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - 3600, 0)
            .expect("valid timestamp"),
    );
    assert!(PollingService::should_poll(
        &Schedule::Hourly,
        one_hour_ago,
        now
    ));
    let one_sec_ago = educore_core::value_objects::Timestamp::from_datetime(
        chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - 1, 0)
            .expect("valid timestamp"),
    );
    assert!(!PollingService::should_poll(
        &Schedule::Hourly,
        one_sec_ago,
        now
    ));
}

// Scenario 5: Rate limit token bucket + trait-surface exercise on the
// in-memory testkit provider.
#[test]
fn port_integrations_rate_limit_bucket() {
    let mut svc = RateLimitService::new();
    let id = IntegrationId::new("twilio");
    svc.reset(&id);
    assert!(svc.try_acquire(&id, 3));
    assert!(svc.try_acquire(&id, 3));
    assert!(svc.try_acquire(&id, 3));
    assert!(!svc.try_acquire(&id, 3));

    // Trait-surface exercise.
    let _gateway: Arc<dyn IntegrationGateway> = Arc::new(InMemoryIntegrationGateway::new());
}

// Env-gated async scenarios

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var; run with: cargo test -- --ignored"]
async fn port_integrations_async_lms_roster_sync_mock() {
    let _integration = LmsIntegrationBuilder::new()
        .provider("google_classroom".to_owned())
        .api_key("test-api-key".to_owned())
        .build();
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var; run with: cargo test -- --ignored"]
async fn port_integrations_async_webhook_out_dispatch_mock() {
    let _integration = WebhookOutIntegrationBuilder::new()
        .target(WebhookTarget {
            url: "https://school.example.com/hooks/educore".to_owned(),
            secret: "test-secret".to_owned(),
            event_filter: Some("InvoicePaid".to_owned()),
        })
        .build();
}
