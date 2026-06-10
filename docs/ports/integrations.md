# Integration Port

## Purpose

The integration port connects the engine to external systems that are
not covered by the more specific ports (auth, payments, notifications,
file storage). Examples include LMS sync, video conferencing, identity
providers, and custom webhooks.

The port is intentionally generic. Each integration defines its own
command/event shapes within the engine and a corresponding adapter
that performs the actual I/O.

## Trait: `IntegrationGateway`

```rust
#[async_trait]
pub trait IntegrationGateway: Send + Sync + std::fmt::Debug {
    async fn invoke(&self, request: IntegrationRequest) -> Result<IntegrationResponse>;
    async fn list_capabilities(&self) -> Result<Vec<IntegrationCapability>>;
    async fn health(&self) -> Result<IntegrationHealth>;
}
```

The trait is object-safe.

## IntegrationRequest

```rust
pub struct IntegrationRequest {
    pub tenant: TenantContext,
    pub integration: IntegrationId,
    pub action: IntegrationAction,
    pub input: serde_json::Value,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub timeout: Option<Duration>,
}
```

`IntegrationId` is a typed enum or string identifier for the
integration. `IntegrationAction` is the specific operation. `input` is
the JSON payload.

## IntegrationResponse

```rust
pub struct IntegrationResponse {
    pub status: IntegrationStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<IntegrationError>,
    pub duration: Duration,
    pub cost: Option<Money>,
    pub metadata: BTreeMap<String, String>,
}

pub enum IntegrationStatus {
    Success,
    Accepted,                          // asynchronous, will arrive via webhook
    RateLimited,
    Failed,
    TimedOut,
}
```

## IntegrationCapability

```rust
pub struct IntegrationCapability {
    pub integration: IntegrationId,
    pub action: IntegrationAction,
    pub description: String,
    pub input_schema: Option<SchemaRef>,
    pub output_schema: Option<SchemaRef>,
    pub required_capabilities: Vec<Capability>,
}
```

The engine can enumerate capabilities at runtime for UIs and AI agent
tool catalogs.

## Standard Integrations

The engine documents the expected behavior of common integrations
without shipping their adapters. Consumers implement them.

### LMS Sync (Google Classroom, Microsoft Teams, Moodle)

The engine models `lms.course.linked` and `lms.roster.synced` events.
The integration adapter:

- Creates an LMS course when an `AcademicYear` and `Class` are
  configured in the engine.
- Syncs the roster (StudentRoster) when `StudentAdmitted`,
  `StudentAssignedToSection`, or `StudentWithdrawn` events fire.
- Pulls assignment submissions from the LMS and emits
  `OnlineExamSubmitted` events with a `Source::Lms` tag.

### Video Conferencing (Zoom, Google Meet)

The engine models `videocall.scheduled` and `videocall.ended` events.
The integration adapter:

- Creates a meeting when a `LessonPlan` is scheduled.
- Records attendance by joining time and joining user list.
- Stores the recording in the file storage port and emits
  `VideoRecordingAvailable`.

### Identity Provider (External IdP)

See `authentication.md`. The IdP is an auth provider, not an
integration. The engine treats it as such.

### Payment Gateway (Stripe, etc.)

See `payments.md`. The gateway is a payment provider, not an
integration. However, **two-step flows** (e.g. Stripe Connect for
multi-tenant SaaS) may use the integration port for setup
operations (onboarding accounts, creating products, etc.) and the
payment port for charges.

### SMS Gateway (Twilio, etc.)

See `notifications.md`. The SMS gateway is a notification channel,
not an integration.

### Storage Backend (S3, GCS)

See `file-storage.md`. The storage backend is a file storage
adapter, not an integration.

### Custom Webhook (Out)

The engine can publish events to a configured webhook URL:

```rust
let adapter = WebhookIntegration::new(WebhookConfig {
    url: Url::parse("https://school.example.com/hooks/educore")?,
    secret: SecretString::from("shared-secret"),
    retry_policy: RetryPolicy::Exponential { max_retries: 5, base: Duration::from_secs(2) },
    filter: Some(EventFilter::EventType("InvoicePaid")),
});
```

The adapter signs the payload with HMAC-SHA256 and posts it. The
receiver verifies the signature.

### Polling Adapter (In)

The engine can pull from an external system on a schedule:

```rust
let adapter = PollingIntegration::new(PollingConfig {
    url: Url::parse("https://vendor.example.com/api/students")?,
    schedule: Schedule::Hourly,
    cursor_field: "updated_at",
    target_event: "ExternalStudentSynced",
    auth: AuthStrategy::Bearer { token: ... },
});
```

The adapter polls, pages through results using the cursor, and emits
events. Idempotency is enforced by the receiver (the engine).

## OAuth2 Client Credentials (Per Integration)

Some integrations require OAuth2 client credentials. The engine does
not own this flow; the adapter does. The adapter:

1. Stores the client_id and client_secret (per tenant).
2. Performs the OAuth2 token exchange.
3. Caches the token until expiry.
4. Refreshes the token before expiry.

## Per-Tenant Configuration

Integrations are configured per tenant. The `IntegrationConfig` value
is loaded from the platform domain at startup. The engine passes
`TenantContext` to the adapter; the adapter uses it to look up the
config.

## Retry Policy

```rust
pub enum RetryPolicy {
    None,
    Linear { max_retries: u32, interval: Duration },
    Exponential { max_retries: u32, base: Duration, max: Duration },
}
```

The adapter retries transient failures (5xx, network) per the policy.
Permanent failures (4xx) are returned immediately.

## Audit Logging

Every integration invocation is logged with tenant, integration,
action, status, duration, and cost. Input and output are logged at
`DEBUG` and may be redacted by the adapter.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("integration not configured: {0}")] NotConfigured(IntegrationId),
    #[error("integration not found: {0}")] NotFound(IntegrationId),
    #[error("invalid input: {0}")] InvalidInput(String),
    #[error("rate limited")] RateLimited,
    #[error("timeout after {0:?}")] Timeout(Duration),
    #[error("provider error: {0}")] Provider(String),
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

## Worked Example

A consumer registers a Twilio SMS integration and dispatches a
notification:

```rust
let gateway: Arc<dyn IntegrationGateway> = Arc::new(TwilioGateway::new(
    TwilioConfig::from_env()?
));

let response = gateway.invoke(IntegrationRequest {
    tenant,
    integration: IntegrationId::new("twilio"),
    action: IntegrationAction::new("send_sms"),
    input: json!({
        "to": "+12025550100",
        "body": "Your child is absent today.",
    }),
    idempotency_key,
    correlation_id,
    timeout: Some(Duration::from_secs(10)),
}).await?;
```

(Note: most SMS flows go through the `NotificationProvider` port
directly. The integration port is for advanced scenarios like two-way
SMS or SMS-as-auth.)

## Object Safety

`IntegrationGateway` is object-safe.

## Testing

- Unit tests of every action.
- Integration tests of OAuth2 flow, retry, timeout.
- A test of webhook signature verification.
- A test of polling pagination.
- A test of rate limiting.
- A test of per-tenant config isolation.

## Offline Mode

In offline mode, integration invocations are queued in the outbox.
The adapter flushes the queue on reconnect. Some integrations (real-
time notifications) are unavailable in offline mode.

## Audit

Every invocation, success or failure, is recorded with full
metadata. Sensitive fields are redacted by the adapter.
