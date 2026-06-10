# ADR-007: Audit-First Design

## Status

Accepted.

## Context

A school is a system of trust. Parents trust the school with
their children. Regulators trust the school with student
data. Courts trust the school's records in disputes. Staff
trust the school with their salaries. The audit log is the
mechanism by which all of these trusts are honored.

The naive approach — "we have a `created_at` and `updated_at`
column, that's enough" — is not. It loses the actor, the
correlation, the IP, the before/after state, and the
rationale. It is also fragile: a `created_at` column can be
overwritten by a careless migration.

The school's audit needs are not edge cases. They are
routine:

- "Why was this student withdrawn last Tuesday?" — needs
  actor, command, reason, before/after.
- "Who collected the fees payment of 10,000 rupees on March
  3?" — needs actor, payment record, IP, session.
- "Show me every change to the payroll in the last quarter."
  — needs actor, command, version, before/after.
- "A parent is disputing the marks. Show me the audit trail."
  — needs every marks change with timestamps and actors.

A bug that loses the audit log is a bug that loses the
school's institutional memory. It is the kind of bug that
ends careers.

## Decision

Educore is **audit-first**: every state-changing command
writes a durable audit record, and the audit log is the
canonical record of every change in the engine.

Concretely:

1. **Every state-changing command produces exactly one
   audit record.** The record is written in the same
   database transaction as the aggregate mutation. Either
   both succeed or both fail.
2. **Every domain event is mirrored to the audit log.**
   The audit record carries the `event_id`, the
   `correlation_id`, the actor, the source IP and user
   agent, and the before/after snapshot.
3. **The audit log is append-only.** The engine provides
   no `update_audit` or `delete_audit` operation.
4. **Storage adapters enforce immutability** at the
   database level: the audit writer has `INSERT`-only
   privilege; no `UPDATE` or `DELETE` triggers exist;
   no application code issues them.
5. **Compliance deployments replicate the audit log to a
   WORM store** (S3 Object Lock, Azure Blob Immutable
   Storage, GCS Bucket Lock). Replication is a port.
6. **Audit retention is configurable per deployment.**
   Default retention periods are defined in
   `audit-schema.md` § 9. The consumer's compliance team
   may override.
7. **PII in audit snapshots is redacted** by a configurable
   redactor. The redactor is a port.
8. **The audit log is queryable** through a typed
   `AuditQuery` port. Cross-tenant queries are capability-
   gated.
9. **Compliance commands** — `Report.SubjectAccess`,
   `Report.SubjectErasure`, `Report.ParentAccess`,
   `Report.RegulatorAudit` — produce their own audit
   events. Generating a report is itself audited.

The audit log is not a debugging tool. It is a **legal
record**. The schema and retention are designed to survive
inquiries from regulators, courts, and parents.

## Consequences

### Positive

- **Every change is reconstructable.** An auditor can
  reproduce any historical state by replaying the event
  log.
- **Disciplinary review is evidence-based.** A
  principal can review who admitted, who withdrew, who
  edited the marks.
- **AI agents are accountable.** Every command the agent
  invokes is audited with `source = "agent"`, the agent's
  user id, the prompt context (where captured), and the
  outcome.
- **Compliance is built in.** GDPR data-subject requests,
  FERPA-style parental access, regulator audits are
  first-class commands.
- **Tamper resistance.** The WORM replication makes the
  audit log tamper-evident in regulated deployments.
- **Cross-domain traceability.** A `correlation_id`
  links a single user request to every event it caused,
  across every domain.

### Negative

- **Storage cost.** The audit log grows linearly with
  every state change. A school with 5,000 students, 200
  staff, and 50,000 events per day produces ~18 million
  events per year. Mitigated by retention policies,
  archival to cold storage, and snapshot compaction.
- **Performance cost.** Writing the audit record adds
  one insert per command. The insert is in the same
  transaction as the aggregate, so the overhead is
  bounded. The audit table is indexed for the common
  query patterns.
- **PII handling complexity.** The audit log captures
  PII, which is subject to erasure requests. The
  redactor + erasure flow is non-trivial; the engine
  provides it.
- **Read performance is secondary.** The audit table is
  optimized for append and the common query patterns,
  not for arbitrary analytics. Heavy analytics on the
  audit log is done by replicating to a data warehouse
  via the integration port.

### Mitigations

- The `audit_log` table is partitioned by month
  (PostgreSQL declarative partitioning) in the default
  adapter. Old partitions are detached and archived
  to cold storage.
- Snapshot policies are configurable per aggregate. The
  default is `Diff` (only changed fields); `Full` is
  reserved for high-sensitivity aggregates.
- The redactor is a port; the default implementation
  redacts known sensitive fields. Consumers may extend.
- The audit port supports replication to a SIEM via the
  `AuditSink` port.

## Alternatives Considered

### 1. Database triggers

The database writes the audit row in a trigger. Rejected
because it ties the audit to a specific database, hides
the audit logic in DDL, and is impossible to test
without a database.

### 2. Application-level audit, no enforcement

The application writes audit records; trust the
developer. Rejected because the audit is the school's
institutional memory; "trust the developer" is not
defense in depth.

### 3. Logging framework (e.g. `tracing`) as the audit log

Use the structured logger. Rejected because logs are
not durable, not queryable, not retention-managed, and
not subject to the same compliance controls.

### 4. Event sourcing as the audit

The event log is the audit. Rejected because events
lack the actor, the IP, the user agent, and the
before/after snapshot. The engine's event log is the
**input** to the audit log; the audit log is the
**canonical record**.

### 5. No audit, "we have backups"

Reconstruct from backups. Rejected because backups
are slow, lossy, and do not answer "who did this?"
or "why?" They answer "what was the state?" — and
even then only approximately.
