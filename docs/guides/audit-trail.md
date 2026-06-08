# Audit Trail Guide

## Goal

Every state-changing command writes an immutable audit record. Auditors
can reconstruct any historical state and explain who did what, when,
and why.

## AuditSink Port

The engine writes audit records through the `AuditSink` port. The
default implementation persists to a database table. Compliance
deployments may write to WORM storage, SIEM, or both.

```rust
#[async_trait]
pub trait AuditSink: Send + Sync {
    async fn write(&self, record: AuditRecord) -> Result<()>;
    async fn query(&self, q: AuditQuery) -> Result<Vec<AuditRecord>>;
}
```

## AuditRecord

```rust
pub struct AuditRecord {
    pub audit_id: AuditId,
    pub tenant: TenantContext,
    pub actor_id: UserId,
    pub actor_capabilities: Vec<Capability>,
    pub action: String,                  // e.g. "Student.Admit"
    pub resource_type: &'static str,     // e.g. "Student"
    pub resource_id: Uuid,
    pub outcome: AuditOutcome,
    pub timestamp: Timestamp,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub before: Option<serde_json::Value>,  // snapshot before the change
    pub after: Option<serde_json::Value>,   // snapshot after the change
    pub diff: Option<JsonDiff>,             // structured diff
    pub metadata: BTreeMap<String, String>,  // request id, IP, user agent, etc.
    pub signature: Option<DigitalSignature>, // optional cryptographic signature
}

pub enum AuditOutcome {
    Success,
    Failure { reason: String, code: &'static str },
    Denied { reason: String },
}
```

## What Gets Audited

The engine writes an audit record for:

- Every command (success, failure, or denied).
- Every state-changing domain event.
- Authentication attempts (success, failure, MFA challenge).
- Permission changes.
- Configuration changes.
- File uploads and downloads.
- Payment transactions.
- Notification deliveries.
- Backup and restore operations.
- Login/logout events.

The engine does NOT audit:

- Read-only queries (they are logged at `DEBUG` for performance
  debugging, not for audit).
- Internal computations.
- Caching operations.

## Storage

Audit records are append-only. The database table is configured with
`INSERT` permissions only for the audit user. `UPDATE` and `DELETE`
are denied. The audit sink is responsible for enforcing this.

Compliance deployments may use:

- PostgreSQL with `INSERT`-only roles.
- A dedicated audit database (separate from the operational database).
- WORM storage (S3 Object Lock, Azure Immutable Blob).
- A SIEM (Splunk, Elastic, Sumo Logic).

## Retention

The default retention is **indefinite**. Compliance requirements
dictate shorter or longer retention (e.g. FERPA requires 5 years for
educational records). The consumer configures retention per audit
class.

## Querying

```rust
let records = engine.audit().query(AuditQuery {
    tenant: Some(tenant.school_id),
    actor: Some(actor_id),
    action: Some("Student.Admit"),
    resource_type: Some("Student"),
    resource_id: Some(student_id.into()),
    from: Some(timestamp_yesterday),
    to: Some(timestamp_now),
    limit: 50,
    ..Default::default()
}).await?;
```

## Compliance Reports

The engine produces standard compliance reports:

- `audit.trail.actor` — all actions by an actor.
- `audit.trail.resource` — all actions on a resource.
- `audit.trail.action` — all instances of an action.
- `audit.trail.failed` — all failed/denied actions.
- `audit.trail.suspicious` — actions outside normal patterns.

The audit sink is queried through the engine's reports port.

## Cryptographic Signing (Optional)

For high-integrity deployments, the audit sink signs each record
with a server-side key. The signature is included in the record. A
tampering attempt is detected by signature verification.

## PII Handling

Audit records may contain PII (e.g. the `after` snapshot of a
student profile update). The consumer is responsible for:

- Restricting access to the audit sink to authorized roles.
- Encrypting the audit database at rest.
- Redacting PII in long-term archives per applicable law.

## Worked Example

A consumer adds a Redis-backed audit sink alongside the database
sink for high-volume debugging:

```rust
let audit: Arc<dyn AuditSink> = Arc::new(
    TeeAuditSink::new(
        PostgresAuditSink::new(pool.clone()),
        RedisAuditSink::new(redis_client.clone()),
    )
);
```

The engine's `EngineBuilder::audit(audit)` wires it in.

## Object Safety

`AuditSink` is object-safe.

## Testing

- Unit tests of every command producing an audit record.
- A test of the audit query API.
- A test of failure outcomes producing failure audit records.
- A test of denied commands producing denied audit records.
- A test of the `INSERT`-only role enforcement (database-level).
