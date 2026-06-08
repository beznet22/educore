# Audit Schema

This document is **normative**. It defines the engine's audit log:
what is recorded, how it is recorded, how it is queried, and how it
is retained. The audit log is the canonical record of every state
change in SMSengine and the foundation of compliance reporting.

## 1. Purpose

The audit log answers five questions for every event in the life of
the school:

1. **Who** — which actor performed the action?
2. **What** — which aggregate, which command, which event?
3. **When** — at what time (clock + record time)?
4. **From where** — IP, user agent, device, session?
5. **Why** — free-text rationale (where supported)?

The audit log is the source of truth for:

- Compliance reporting (GDPR data-subject requests, FERPA-style
  parental access, regulator audits).
- Forensic reconstruction of state (what was the invoice balance on
  date X?).
- Disciplinary review (who deleted the student's marks?).
- AI-agent safety review (which actions did the agent take last
  week?).

## 2. The Audit Record

Every state-changing command produces exactly one audit record. Every
domain event is mirrored to the audit log. The audit record is:

```text
AuditRecord {
    audit_id:        AuditId,           // UUIDv7, time-ordered
    school_id:       SchoolId,          // tenant anchor
    actor_id:        UserId,            // user or SYSTEM
    actor_type:      ActorType,         // user, system, agent, api_key
    action:          AuditAction,       // command type or event type
    resource_type:   ResourceType,      // "student", "fees_invoice"
    resource_id:     Uuid,              // the aggregate id
    event_id:        Option<EventId>,   // the originating event
    command_id:      Option<CommandId>, // the originating command
    correlation_id:  CorrelationId,     // join key
    occurred_at:     Timestamp,         // event time
    recorded_at:     Timestamp,         // persistence time
    ip:              Option<IpAddr>,    // network origin
    user_agent:      Option<String>,
    session_id:      Option<SessionId>,
    before:          Option<Value>,     // pre-mutation snapshot
    after:           Option<Value>,     // post-mutation snapshot
    metadata:        AuditMetadata,     // open-ended
    cross_tenant:    bool,              // true if cross-tenant op
    source:          AuditSource,       // web, mobile, api, agent, system
}
```

### 2.1 Snapshot Strategy

The `before` and `after` fields are JSON snapshots of the mutated
aggregate's state, or of the specific fields the command changed.
The engine adopts three snapshot policies, configurable per
domain:

- **None** — no snapshot, just the action and resource.
- **Diff** — only the changed fields, with old and new values.
- **Full** — the entire aggregate state, before and after.

The default is `Diff` for most aggregates and `Full` for
high-sensitivity aggregates (finance, payroll, security).

### 2.2 PII Handling

PII is captured in the audit log as part of `before` / `after`. The
audit log is subject to the same retention and erasure rules as the
underlying data. Erasure requests anonymize the PII in the audit
record but preserve the structural metadata (who, what, when, from
where).

## 3. Immutability and WORM

The audit log is **append-only**. The engine provides no
`update_audit` or `delete_audit` operation. Storage adapters enforce
this through:

- Database privileges — the audit writer has `INSERT`-only on the
  audit table.
- Schema — no `UPDATE` or `DELETE` triggers are defined; the
  application code never issues them.
- Application-level — a `DenyAll` middleware on the audit
  repository.
- Operational — the audit table is replicated to a write-once
  read-many (WORM) store (S3 Object Lock, Azure Blob Immutable
  Storage, GCS Bucket Lock) for compliance deployments.

The WORM replication is a port. The default is database-only. A
consumer MAY enable WORM in regulated environments.

## 4. What is Recorded

The engine records an audit entry for:

1. **Every state-changing command** that successfully completes
   (after persistence, before response).
2. **Every state-changing command** that fails — the audit record
   includes the error kind and message (no PII).
3. **Every authentication event** — login, logout, token issuance,
   password reset, 2FA challenge.
4. **Every authorization event** — capability check denial.
5. **Every capability / role change** — assignment, revocation,
   catalog mutation.
6. **Every cross-tenant operation**.
7. **Every backup, restore, and migration** event.
8. **Every settings change** that affects security
   (2FA policy, password policy, session timeout).
9. **Every school lifecycle event** — onboarding, suspension,
   deletion, transfer.

The engine does **not** record:

- Read-only queries (they generate access logs at the consumer's
  HTTP layer, not in the engine audit log).
- Heartbeats, health checks, or internal engine events that are not
  user-facing.
- Telemetry or metrics.

## 5. Query API

The audit log is queryable through a dedicated port:

```rust
pub trait AuditQuery: Send + Sync {
    async fn list(
        &self,
        tenant: &TenantContext,
        filter: AuditFilter,
        page: Page,
    ) -> Result<Vec<AuditRecord>, AuditError>;

    async fn get(
        &self,
        tenant: &TenantContext,
        audit_id: AuditId,
    ) -> Result<AuditRecord, AuditError>;

    async fn resource_history(
        &self,
        tenant: &TenantContext,
        resource_type: ResourceType,
        resource_id: Uuid,
        page: Page,
    ) -> Result<Vec<AuditRecord>, AuditError>;

    async fn actor_history(
        &self,
        tenant: &TenantContext,
        actor_id: UserId,
        page: Page,
    ) -> Result<Vec<AuditRecord>, AuditError>;
}
```

Filters:

- `AuditFilter::ByAction { action, since, until }` — every record
  matching an action.
- `AuditFilter::ByResource { resource_type, resource_id }` — every
  record for a specific resource.
- `AuditFilter::ByActor { actor_id, since, until }` — every record
  by a specific actor.
- `AuditFilter::ByCorrelation { correlation_id }` — every record in a
  request / workflow.
- `AuditFilter::ByTimeRange { since, until }` — every record in a
  time window.
- `AuditFilter::ByEventType` — every record of a specific event
  type.
- `AuditFilter::Custom { predicate }` — domain-specific filter
  (capability-gated).

All queries are tenant-scoped. A consumer without
`Platform.CrossTenant` cannot query across schools.

## 6. Per-Aggregate Audit

The audit query's `resource_history` method returns the full
mutation history of a single aggregate, ordered by `occurred_at`.
This is the primary tool for "what happened to this student?" and
"who last touched this invoice?".

A per-aggregate view is a read-only projection over the audit log;
the engine does not maintain a separate audit table per aggregate.

## 7. Per-Action Audit

The audit query's `ByAction` filter returns every record of a
specific action, ordered by `occurred_at`. This is the primary tool
for "every student admission in the last 30 days" or "every
payroll payment in Q1".

## 8. Compliance Reporting

### 8.1 GDPR Data-Subject Access Requests

A data-subject access request is answered with:

1. **All PII held** about the subject, across every domain.
2. **All processing activities** the subject was involved in
   (every event with the subject as actor or resource).
3. **All third parties** the data was shared with (every
   cross-tenant or integration event involving the subject).

The engine provides a `Report.SubjectAccess.Generate` command that
produces a self-contained JSON / PDF bundle. The bundle is
auditable itself — generating the report produces an audit event.

### 8.2 Right to Erasure

A data-subject erasure request:

1. **Soft-deletes** the subject's profile.
2. **Anonymizes** PII in the audit log (replaces the subject's
   fields with `REDACTED`, retains the structural metadata).
3. **Retains** financial records for the regulator-required period
   (default 7 years), anonymized to remove PII.
4. **Records** the erasure event with the actor, the reason, and
   the request id.

The engine provides a `Report.SubjectErasure.Execute` command.

### 8.3 FERPA-Style Parental Access

A parent requesting their child's records receives:

1. **Academic** — class, section, marks, attendance, homework,
   report cards.
2. **Behavioral** — notices received, complaints filed,
   disciplinary notes.
3. **Financial** — fees assigned, payments made, balances.

The engine provides a `Report.ParentAccess.Generate` command. The
parent must hold the `Parent.Read` capability for the child in
question.

### 8.4 Regulator Audit

A regulator audit is answered with:

1. **All state changes** in a time range (the full audit log for
   the school).
2. **All authorization decisions** (capability checks, including
   denials).
3. **All data exports** to third parties.
4. **All backups and restores** in the time range.

The engine provides a `Report.RegulatorAudit.Generate` command.
The consumer's compliance team reviews the report and signs off
internally before disclosure.

## 9. Retention Policy

The default retention periods are:

| Record type                   | Retention                |
| ----------------------------- | ------------------------ |
| Authentication events         | 18 months                |
| Authorization denials         | 36 months                |
| Capability / role changes     | 7 years                  |
| Finance mutations             | 7 years                  |
| Payroll mutations             | 7 years                  |
| Academic mutations            | 7 years                  |
| Library / facilities mutations| 3 years                  |
| Settings changes              | 3 years                  |
| Backup events                 | 3 years                  |
| AI agent actions              | 36 months                |

The retention periods are configurable per deployment. The engine
provides a `AuditRetention` policy struct; the consumer's
operations team configures it.

After expiry, the engine MAY archive records to cold storage
(S3 Glacier, Azure Archive) and remove them from the active audit
log. The archive is signed and tamper-evident.

## 10. Access Control

Audit log access is gated by the `AuditLog.Read` capability. A
school admin with this capability can read the school's audit log.
The `SuperAdmin` role can read the platform-wide audit log.

The engine does **not** allow editing or deletion of audit records
through any user-facing path. Database-level access is the only way
to mutate the table, and that is restricted to the audit writer.

## 11. Privacy Filtering

A consumer MAY configure the audit log to redact sensitive fields
from the `before` and `after` snapshots. The redactor is a port
that the consumer implements; the default implementation
redacts known sensitive fields (`password`, `secret`,
`api_key`, `token`, `otp`).

The redactor is applied **before** the audit record is written.
The original PII is never written to the audit log if the redactor
is configured to drop it.

## 12. Cross-Tenant Audit

Cross-tenant operations carry `cross_tenant = true` and the source
and target `school_id` (the source is in the `actor_id`'s
`TenantContext`, the target is in the resource's `school_id`).
Cross-tenant audit records are visible only to `SuperAdmin`.

## 13. Storage Layout

The audit log is stored in a separate table from the domain
aggregates. The schema (per storage adapter):

```text
audit_log(
    audit_id        UUID PRIMARY KEY,
    school_id       UUID NOT NULL,
    actor_id        UUID NOT NULL,
    actor_type      VARCHAR,
    action          VARCHAR NOT NULL,
    resource_type   VARCHAR NOT NULL,
    resource_id     UUID NOT NULL,
    event_id        UUID NULL,
    command_id      UUID NULL,
    correlation_id  UUID NOT NULL,
    occurred_at     TIMESTAMP NOT NULL,
    recorded_at     TIMESTAMP NOT NULL,
    ip              INET NULL,
    user_agent      VARCHAR NULL,
    session_id      UUID NULL,
    before          JSONB NULL,
    after           JSONB NULL,
    metadata        JSONB NULL,
    cross_tenant    BOOLEAN NOT NULL DEFAULT false,
    source          VARCHAR NOT NULL
);
CREATE INDEX idx_audit_log_school_time ON audit_log (school_id, occurred_at);
CREATE INDEX idx_audit_log_actor ON audit_log (actor_id, occurred_at);
CREATE INDEX idx_audit_log_resource ON audit_log (resource_type, resource_id, occurred_at);
CREATE INDEX idx_audit_log_correlation ON audit_log (correlation_id);
```

The indexes support the common query patterns: per-tenant time
range, per-actor history, per-resource history, per-correlation
chain.

## 14. Audit-Driven Subscriptions

Subscribers to the audit log are a port. A consumer MAY provide:

- A SIEM (Security Information and Event Management) adapter that
  ships every record to a centralized log store.
- A real-time alerting adapter that fires on suspicious patterns
  (e.g. 10 failed logins in 60 seconds).
- A retention policy engine that archives or purges records on
  schedule.

The audit port is a one-way firehose; the engine does not support
acknowledgement or replay semantics on it. Replay is supported
through the event bus.

## 15. Engine-Internal vs. User-Facing Events

The audit log is for **user-facing** events only. The engine's
internal events (cache invalidations, internal heartbeats) are not
recorded in the audit log. They are emitted to the event bus and
logged via `tracing` at `INFO` or `DEBUG` level.
