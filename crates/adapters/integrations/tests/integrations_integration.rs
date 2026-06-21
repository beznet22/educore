//! # Phase 15 integrations port vertical-slice integration test
//!
//! 5 sync scenarios (always-on) + 2 env-gated async scenarios.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_integrations::prelude::*;
use educore_integrations::services::{PollingService, RateLimitService, RetryService, WebhookSignatureService};

// ---------------------------------------------------------------------------
// Scenario 1: Webhook signature HMAC-SHA256
// ---------------------------------------------------------------------------
//
// `WebhookSignatureService` is a unit struct; its
// `compute_signature` and `verify_signature` are associated
// functions (no `&self` parameter).

#[test]
fn integrations_integration_webhook_signature() {
    let secret = "whsec_test_secret";
    let payload = b"{\"event\":\"invoice.paid\"}";
    let sig = WebhookSignatureService::compute_signature(secret, payload).expect("HMAC succeeds");
    assert!(sig.starts_with("sha256="));
    assert!(WebhookSignatureService::verify_signature(secret, payload, &sig)
        .expect("verify succeeds"));
    assert!(!WebhookSignatureService::verify_signature(secret, b"{\"tampered\":true}", &sig)
        .expect("verify succeeds"));
    // Touch the unit struct to silence "unused" if the import is
    // ever trimmed; the struct is referenced in the doc comment.
    let _svc = WebhookSignatureService;
}

// ---------------------------------------------------------------------------
// Scenario 2: Retry exponential backoff
// ---------------------------------------------------------------------------
//
// `RetryPolicy::Exponential.base` and `.max` are `chrono::Duration`
// (not `std::time::Duration`). `next_backoff` returns
// `Option<std::time::Duration>` (not a `Result`) and returns
// `None` for `attempt == 0` (the original call). The exponential
// formula is `base * 2^attempt` for `attempt >= 1`. To get the
// `2s, 4s, 8s` sequence use `base = 1s` and `attempt = 1, 2, 3`.

#[test]
fn integrations_integration_retry_exponential() {
    let policy = RetryPolicy::Exponential {
        max_retries: 5,
        base: chrono::Duration::seconds(1),
        max: chrono::Duration::seconds(300),
    };
    let backoff1 = RetryService::next_backoff(&policy, 1).expect("attempt 1");
    let backoff2 = RetryService::next_backoff(&policy, 2).expect("attempt 2");
    let backoff3 = RetryService::next_backoff(&policy, 3).expect("attempt 3");
    // Exponential: base * 2^1 = 2s, base * 2^2 = 4s, base * 2^3 = 8s
    assert_eq!(backoff1, std::time::Duration::from_secs(2));
    assert_eq!(backoff2, std::time::Duration::from_secs(4));
    assert_eq!(backoff3, std::time::Duration::from_secs(8));
}

// ---------------------------------------------------------------------------
// Scenario 3: Retry classification (4xx = permanent, 5xx = transient)
// ---------------------------------------------------------------------------
//
// `is_permanent_failure` is an associated function (no `&self`).

#[test]
fn integrations_integration_retry_classification() {
    // 4xx (except 408, 429) = permanent failure
    assert!(RetryService::is_permanent_failure(400));
    assert!(RetryService::is_permanent_failure(404));
    assert!(!RetryService::is_permanent_failure(408)); // timeout = transient
    assert!(!RetryService::is_permanent_failure(429)); // rate-limited = transient
    // 5xx = transient
    assert!(!RetryService::is_permanent_failure(500));
    assert!(!RetryService::is_permanent_failure(503));
    // 2xx, 3xx = not permanent (success, would not be retried anyway)
    assert!(!RetryService::is_permanent_failure(200));
    assert!(!RetryService::is_permanent_failure(302));
}

// ---------------------------------------------------------------------------
// Scenario 4: Polling schedule
// ---------------------------------------------------------------------------
//
// `should_poll` is an associated function (no `&self`).

#[test]
fn integrations_integration_polling_schedule() {
    let now = educore_core::value_objects::Timestamp::now();
    let one_hour_ago = educore_core::value_objects::Timestamp::from_datetime(
        chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - 3600, 0)
            .expect("valid timestamp"),
    );
    // If last poll was 1 hour ago and schedule is hourly, should poll
    assert!(PollingService::should_poll(&Schedule::Hourly, one_hour_ago, now));
    // If last poll was 1 second ago, should NOT poll (not enough time passed)
    let one_sec_ago = educore_core::value_objects::Timestamp::from_datetime(
        chrono::DateTime::from_timestamp(chrono::Utc::now().timestamp() - 1, 0)
            .expect("valid timestamp"),
    );
    assert!(!PollingService::should_poll(&Schedule::Hourly, one_sec_ago, now));
}

// ---------------------------------------------------------------------------
// Scenario 5: Rate limit token bucket
// ---------------------------------------------------------------------------
//
// `RateLimitService` is a single-threaded stateful cache; the
// `try_acquire` and `reset` methods take `&mut self`.

#[test]
fn integrations_integration_rate_limit_bucket() {
    let mut svc = RateLimitService::new();
    let id = IntegrationId::new("twilio");
    svc.reset(&id);
    assert!(svc.try_acquire(&id, 3));
    assert!(svc.try_acquire(&id, 3));
    assert!(svc.try_acquire(&id, 3));
    assert!(!svc.try_acquire(&id, 3)); // bucket exhausted
}

// ---------------------------------------------------------------------------
// Env-gated async scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn integrations_integration_async_lms_roster_sync_mock() {
    let _integration = LmsIntegrationBuilder::new()
        .provider("google_classroom".to_owned())
        .api_key("test-api-key".to_owned())
        .build();
}

#[tokio::test]
#[ignore = "requires EDUCORE_PORT_ADAPTER_E2E env var"]
async fn integrations_integration_async_webhook_out_dispatch_mock() {
    let _integration = WebhookOutIntegrationBuilder::new()
        .target(WebhookTarget {
            url: "https://school.example.com/hooks/educore".to_owned(),
            secret: "test-secret".to_owned(),
            event_filter: Some("InvoicePaid".to_owned()),
        })
        .build();
}
