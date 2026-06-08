# Notification Port

## Purpose

The notification port delivers messages to guardians, students, staff,
and external recipients. The engine does not own SMTP, SMS, push, or
chat transports. The consumer supplies an adapter that implements the
trait.

## Trait: `NotificationProvider`

```rust
#[async_trait]
pub trait NotificationProvider: Send + Sync + std::fmt::Debug {
    async fn send(&self, request: SendNotification) -> Result<NotificationReceipt>;
    async fn send_bulk(&self, request: SendBulkNotification) -> Result<BulkReceipt>;
    async fn status(&self, receipt_id: NotificationReceiptId) -> Result<DeliveryStatus>;
}
```

The trait is object-safe.

## SendNotification

```rust
pub struct SendNotification {
    pub tenant: TenantContext,
    pub channel: Channel,
    pub template: TemplateRef,
    pub recipient: Recipient,
    pub variables: BTreeMap<String, TemplateValue>,
    pub attachments: Vec<AttachmentRef>,
    pub priority: Priority,
    pub scheduled_at: Option<Timestamp>,
    pub idempotency_key: Option<IdempotencyKey>,
    pub correlation_id: Option<CorrelationId>,
}
```

`TemplateRef` is a typed reference to a template stored in the
notification domain (`NotificationTemplateId`). The adapter resolves
the template body, applies variables, and delivers.

## Channel

```rust
pub enum Channel {
    Email { from: Option<EmailAddress>, reply_to: Option<EmailAddress> },
    Sms { from: Option<PhoneNumber>, unicode: bool },
    Push { topic: Option<String>, ttl: Option<Duration>, collapse_key: Option<String> },
    InApp,
    Chat { provider: ChatProvider }, // SMS-via-chat apps
    Voice { voice_id: Option<String>, language: LanguageTag },
    Webhook { url: Url, secret: Option<SecretString> },
}
```

A single notification can target multiple channels. The consumer
adapter may fan out internally.

## Recipient

```rust
pub enum Recipient {
    Direct(ContactInfo),
    User(UserId),
    Student(StudentId),
    Guardian(StudentId, GuardianRole),  // the primary or specific guardian of a student
    Staff(StaffId),
    Group(GroupId),
    List(Vec<Recipient>),
    Expression(RecipientExpr),          // e.g. "all students in class 5A"
}
```

`Recipient::Expression` is evaluated by the engine using the query
layer; the adapter receives the materialized list.

## NotificationReceipt

```rust
pub struct NotificationReceipt {
    pub receipt_id: NotificationReceiptId,
    pub provider_message_id: Option<String>,
    pub channel: Channel,
    pub status: DeliveryStatus,
    pub cost: Option<Money>,
    pub sent_at: Timestamp,
    pub metadata: BTreeMap<String, String>,
}
```

The receipt is durable. The engine stores it in the
`sm_email_sms_logs` table and emits a `NotificationSent` event.

## DeliveryStatus

```rust
pub enum DeliveryStatus {
    Queued,
    Sent,
    Delivered,
    Opened,
    Clicked,
    Bounced { reason: String },
    Failed { reason: String, retryable: bool },
    Rejected { reason: String },
}
```

The adapter updates the status as the provider reports it (via
webhook). The engine polls or subscribes to status changes for
reconciliation.

## Templates

Templates are owned by the communication domain. The notification port
references them by id. Templates have:

- A name (per school, scoped to `school_id`).
- A channel (email body, SMS body, push title, push body).
- A subject (for email and push).
- A list of variables (typed, validated at template creation).
- A version (for backward-compatible evolution).

The engine validates that all required variables are provided in
`SendNotification::variables`. Missing variables fail the send.

## Bulk Send

`send_bulk` accepts a request that targets many recipients. The
adapter batches them (per channel limits, e.g. 100 SMS per batch).
Returns a `BulkReceipt` with per-recipient status.

```rust
pub struct SendBulkNotification {
    pub tenant: TenantContext,
    pub template: TemplateRef,
    pub recipients: Vec<BulkRecipient>,
    pub variables_per_recipient: bool,
    pub channel: Channel,
    pub priority: Priority,
    pub scheduled_at: Option<Timestamp>,
    pub idempotency_key: Option<IdempotencyKey>,
}

pub struct BulkRecipient {
    pub recipient: Recipient,
    pub variables: BTreeMap<String, TemplateValue>,
}

pub struct BulkReceipt {
    pub bulk_id: BulkId,
    pub receipts: Vec<NotificationReceipt>,
    pub failed: Vec<(BulkRecipientIndex, NotificationError)>,
}
```

The engine emits one `BulkNotificationSent` event and per-recipient
`NotificationSent` events.

## Idempotency

`idempotency_key` is used by the adapter to deduplicate retries. The
engine generates a deterministic key from `(command_id, recipient,
template_version)` so the same logical send is not duplicated.

## Rate Limiting

The adapter enforces per-tenant, per-channel rate limits (e.g. 100
SMS/second). Limits are configurable per tenant. The adapter returns
`NotificationError::RateLimited` when a limit is hit; the engine
retries with backoff.

## Priority

```rust
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,  // e.g. emergency alerts, must reach
}
```

`Critical` notifications bypass queues and are delivered
synchronously. The adapter may charge a premium for `Critical`.

## Cost Tracking

`cost: Option<Money>` is set by the adapter (e.g. $0.0075 per SMS). The
engine logs the cost for tenant-level reporting and budget control.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("template not found: {0}")] TemplateNotFound(NotificationTemplateId),
    #[error("missing variable: {0}")] MissingVariable(String),
    #[error("invalid recipient: {0}")] InvalidRecipient(String),
    #[error("rate limited")] RateLimited,
    #[error("provider error: {0}")] Provider(String),
    #[error("quota exceeded")] QuotaExceeded,
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

The engine maps provider errors to `DomainError::Infrastructure` and
logs the source. Missing templates and variables are mapped to
`DomainError::Validation`.

## Worked Example

The engine composes a notification request:

```rust
let receipt = engine.notify().send(SendNotification {
    tenant,
    channel: Channel::Email { from: None, reply_to: None },
    template: TemplateRef::Id(tmpl_id),
    recipient: Recipient::Guardian(student_id, GuardianRole::Primary),
    variables: btreemap! {
        "student_name".into() => "Ada Lovelace".into(),
        "absence_date".into() => "2026-06-08".into(),
    },
    attachments: vec![],
    priority: Priority::Normal,
    scheduled_at: None,
    idempotency_key: Some(idempotency_key),
    correlation_id: Some(corr_id),
}).await?;
```

The adapter resolves the template, applies variables, sends via the
provider, and returns the receipt. The engine logs the receipt and
emits `NotificationSent`.

## Object Safety

`NotificationProvider` is object-safe.

## Testing

The port requires:

- Unit tests of every `Channel` variant.
- Integration tests of template resolution, variable application, and
  idempotency.
- A test of bulk send with partial failure.
- A test of rate limiting and retry.
- A test of recipient expression evaluation.
- A test of cost tracking.
- A test of status updates (delivered, opened, clicked).

## Offline Mode

In offline mode, notifications are queued in the local outbox. The
adapter flushes the queue on reconnect. The engine still emits
`NotificationSent` with status `Queued` immediately and updates to
`Sent` on flush.

## Audit

Every send, success or failure, is recorded in the audit log with
template id, recipient hash, channel, status, and cost. PII (phone
numbers, email addresses) is hashed before logging.
