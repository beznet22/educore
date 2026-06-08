# Database Schema — Cross-Cutting Invariants

This document is **normative**. Every storage adapter, migration, and schema
produced for SMScore MUST conform to the rules defined here. The goal is that
the engine's domain logic can be ported across PostgreSQL, SQLite, SurrealDB,
and other backends without re-deriving the invariants.

## 1. Engine-Wide Conventions

### 1.1 Naming

- All identifiers (table, column, index, constraint) are `snake_case`.
- All schema names are `snake_case` and singular or plural by aggregate
  semantics (`student`, `student_record`, `fees_assign`).
- No mixed case, no abbreviations without team agreement, no leading `sm_`
  or platform-specific prefixes inherited from prior systems.
- Index names follow the pattern `idx_<table>_<column>[_<column>...]`.
- Foreign key constraint names follow
  `fk_<table>_<column>_<referenced_table>`.

### 1.2 Charset and Collation

- Default character set: `utf8mb4`.
- Default collation: `utf8mb4_unicode_ci`.
- No table may declare a different charset or collation. Emoji and full
  Unicode are first-class.

### 1.3 Engine

- Default storage engine: `InnoDB`.
- InnoDB is required for transactional foreign key enforcement. No
  `MyISAM`, no `MEMORY`, no engine-specific tricks.

### 1.4 Time and Identifiers

- All timestamps are stored as UTC `TIMESTAMP` (or `timestamptz` in
  PostgreSQL). Conversion to local time is a presentation concern.
- Primary keys are typed identifiers. The default implementation uses
  UUIDv7 (time-ordered) for distributed generation and global uniqueness.
  Adapter implementations MAY swap to ULID, snowflake, or auto-increment
  integers behind the storage port, but the engine API always returns
  typed identifier wrappers.
- Identifiers are opaque to consumers. Strings are never parsed.

## 2. Required Columns on Every Table

The following columns are **mandatory** on every aggregate-bearing table:

| Column        | Type                  | Nullable | Purpose                                                                 |
| ------------- | --------------------- | -------- | ----------------------------------------------------------------------- |
| `id`          | typed identifier      | no       | Surrogate primary key, never reused, never recycled on soft delete.     |
| `school_id`   | `SchoolId`            | no       | Tenant anchor. The engine never queries without it.                    |
| `created_at`  | `TIMESTAMP`           | no       | UTC time of row creation. Set by storage layer, immutable thereafter.   |
| `updated_at`  | `TIMESTAMP`           | no       | UTC time of last mutation. Updated by storage layer on every change.   |
| `created_by`  | `UserId`              | no       | The actor that created the row. Required even for system-inserted rows; |
|               |                       |          | system rows use the engine's `SYSTEM_USER_ID` constant.                 |
| `updated_by`  | `UserId`              | no       | The actor that last mutated the row.                                   |
| `active_status` | `TINYINT` / `BOOLEAN` | no       | Soft-delete flag. `1` = active, `0` = retired. See § 6.                 |

If a table belongs to an academic-year-scoped record (e.g. `student_record`,
`class_section`, `fees_assign`), the following are also required:

| Column        | Type                  | Nullable | Purpose                                                  |
| ------------- | --------------------- | -------- | -------------------------------------------------------- |
| `academic_id` | `AcademicYearId`      | no       | The academic year the row belongs to.                    |
| `record_id`   | `StudentRecordId`     | sometimes| For per-student-year rows, the parent enrollment record. |

These three identifiers — `id`, `school_id`, `academic_id`, `record_id` — are
**reserved column names**. Implementations MUST NOT reuse them for any
other purpose.

## 3. Tenant Isolation

### 3.1 Every Aggregate Anchors to a School

Every aggregate root carries a `school_id`. The engine's storage
adapters MUST:

- Add a database-level row-security policy on every aggregate table:
  `USING (school_id = current_setting('app.school_id')::int)` (or the
  adapter's equivalent mechanism for the target database).
- Inject a `school_id` predicate into every query unless the caller is
  an explicit, capability-gated cross-tenant operation.
- Reject writes whose `school_id` does not match the caller's
  `TenantContext::school_id`.

### 3.2 Cross-Tenant Operations

Cross-tenant operations are **never** the default. They are explicit,
capability-gated commands (e.g. `TransferStudent`, `MergeSchool`). The
caller must hold the `Platform.CrossTenant` capability AND the operation
must be auditable with `cross_tenant = true` on the audit record.

## 4. Foreign Key Rules

- Every cross-aggregate reference is a foreign key.
- The default referential action is `ON DELETE RESTRICT`. Deletion of an
  aggregate is refused while children reference it.
- Soft delete (`active_status = 0`) does NOT remove foreign key
  references. Soft-deleted parents still block hard deletion.
- `ON DELETE CASCADE` is permitted only for **derived** or **owned-child**
  rows whose entire existence is bound to the parent (e.g. `student_record`
  children, audit rows). The cascade is reviewed per ADR.
- `ON DELETE SET NULL` is permitted only for non-essential, advisory
  references (e.g. an optional `note` linked to a deleted class).
- Self-referential foreign keys are permitted (e.g. `class` → `class` for
  prerequisite chains) but must always use `RESTRICT`.
- Foreign key columns are indexed. Composite foreign keys are indexed as a
  leading-prefix composite index.

## 5. Audit Columns

The § 2 columns are sufficient for most audit needs. For domains that
require append-only audit history beyond `created_at` / `updated_at`, the
following columns MAY be added:

| Column            | Type                | Purpose                                                       |
| ----------------- | ------------------- | ------------------------------------------------------------- |
| `version`         | `BIGINT`            | Optimistic concurrency version. See § 9.                     |
| `etag`            | `CHAR(32)` / hash   | Content hash for conflict resolution. See § 9.                |
| `last_event_id`   | `EventId`           | The last domain event that mutated the row.                   |
| `source`          | `VARCHAR` (enum)    | `web`, `mobile`, `api`, `agent`, `import`, `system`.          |
| `correlation_id`  | `CorrelationId`     | The correlation id of the originating chain.                  |

These columns are not duplicated to the audit log: the audit log carries
its own immutable record of who did what, when.

## 6. Soft Delete

Deletion in SMScore is **soft by default**:

- A row is retired by setting `active_status = 0` and `updated_at = now()`.
- Hard deletion is reserved for GDPR-style erasure requests, system
  migrations, and rollback of aborted imports.
- Aggregates MUST keep historical rows queryable. Repository methods that
  return rows MUST filter by `active_status = 1` unless the caller passes
  `IncludeRetired::Yes`.
- Indexes on `active_status` are optional. Most queries use
  `WHERE school_id = ? AND active_status = 1`, and the leading `school_id`
  is selective enough.

## 7. Multi-Tenancy and Row-Level Security

For multi-tenant SaaS deployments:

- A single database holds all tenants. The `school_id` column discriminates.
- Storage adapters MUST configure row-level security:
  - PostgreSQL: `CREATE POLICY school_isolation ON <table> USING
    (school_id = current_setting('app.school_id')::uuid);`
  - SQLite: enforced in adapter code via session-bound `school_id`
    parameter, with no escape hatch.
  - SurrealDB: `DEFINE TABLE ... WITH PERMISSIONS WHERE school_id =
    $auth.school_id;`
- Service-role connections (jobs, system agents) MUST run as a separate
  role with `BYPASSRLS` and MUST still set `app.school_id` explicitly.

For single-tenant deployments (on-premise school), the same `school_id`
column is present but contains a single value per database. The engine
treats both cases identically.

## 8. Naming Conventions Summary

| Element               | Convention                                      | Example                          |
| --------------------- | ----------------------------------------------- | -------------------------------- |
| Table                 | `snake_case`, plural                            | `student_records`                |
| Column                | `snake_case`                                    | `admission_number`               |
| Primary key           | `id`                                            | `id`                             |
| Foreign key column    | `<aggregate_singular>_id`                       | `student_id`                     |
| Index                 | `idx_<table>_<col1>_<col2>`                    | `idx_student_records_school_class` |
| Unique index          | `uq_<table>_<col1>_<col2>`                     | `uq_student_school_admission`    |
| Foreign key           | `fk_<table>_<col>_<ref_table>`                 | `fk_student_records_student`     |
| Check constraint      | `ck_<table>_<rule>`                            | `ck_invoice_amount_non_negative` |
| Sequence / autoinc    | `seq_<table>` (where applicable)               | `seq_audit_log`                  |

## 9. Offline Sync and Conflict Resolution

SMScore supports offline-first operation. Each aggregate table MUST
expose the following two columns:

| Column     | Type            | Purpose                                                                                  |
| ---------- | --------------- | ---------------------------------------------------------------------------------------- |
| `version`  | `BIGINT`        | Monotonically increasing version, incremented on every write. Used for OCC.              |
| `etag`     | `BINARY(16)` / hash | Content-addressed hash of the row's mutable fields. Used for client-side conflict check. |

Conflict resolution rules:

- The engine resolves conflicts using last-writer-wins on `version`,
  with the conflict recorded in the audit log.
- If `etag` is provided by the client, the engine rejects writes whose
  client-side `etag` does not match the server's `etag` (HTTP
  `412 Precondition Failed`).
- Domain events are the authoritative history; the row state is a
  projection. The engine can always rebuild `version` and `etag` by
  re-applying events.

## 10. Charset, Collation, and Locale

- All string columns are `utf8mb4_unicode_ci` by default.
- Case-insensitive search uses the column's collation. Where locale-
  specific behavior is required, an explicit `COLLATE` is set per
  column.
- Binary data (files, hashes) is `VARBINARY` / `BYTEA`, never `VARCHAR`.

## 11. Required and Optional Columns — Quick Reference

The following table is the **canonical minimum schema** for any new
aggregate table:

```text
CREATE TABLE <table_name> (
    id              <typed-id>     PRIMARY KEY,
    school_id       <SchoolId>     NOT NULL,
    -- aggregate-specific columns --
    active_status   TINYINT        NOT NULL DEFAULT 1,
    created_at      TIMESTAMP      NOT NULL,
    updated_at      TIMESTAMP      NOT NULL,
    created_by      <UserId>       NOT NULL,
    updated_by      <UserId>       NOT NULL,
    version         BIGINT         NOT NULL DEFAULT 1,
    etag            BINARY(16)     NOT NULL,
    last_event_id   <EventId>      NULL,
    CONSTRAINT fk_<table>_school FOREIGN KEY (school_id)
        REFERENCES school (id) ON DELETE RESTRICT
);

CREATE INDEX idx_<table>_school_active
    ON <table_name> (school_id, active_status);
```

Academic-year-scoped rows additionally require `academic_id` and
optionally `record_id`:

```text
CREATE TABLE <table_name> (
    -- canonical minimum --
    academic_id     <AcademicYearId> NOT NULL,
    record_id       <StudentRecordId> NULL,
    CONSTRAINT fk_<table>_academic FOREIGN KEY (academic_id)
        REFERENCES academic_year (id) ON DELETE RESTRICT,
    CONSTRAINT fk_<table>_record FOREIGN KEY (record_id)
        REFERENCES student_record (id) ON DELETE RESTRICT
);

CREATE INDEX idx_<table>_school_academic
    ON <table_name> (school_id, academic_id);
```

## 12. Reserved Column Names

The following column names are **reserved** and MUST NOT be used for any
other purpose:

- `id` — primary key.
- `school_id` — tenant anchor.
- `academic_id` — academic year scope.
- `record_id` — student record scope.
- `session_id` — used for backend session storage; never a domain field.
- `version` — optimistic concurrency version.
- `etag` — content hash.
- `active_status` — soft-delete flag.
- `created_at`, `updated_at` — timestamps.
- `created_by`, `updated_by` — actor columns.
- `last_event_id` — last mutating event.

## 13. Compliance and Storage

- Personally identifiable information (PII) MUST be encrypted at rest in
  production deployments. The storage adapter enforces this via
  transparent data encryption or column-level encryption.
- Audit records are stored separately and replicated to a write-once
  medium in regulated deployments (see `audit-schema.md`).
- Soft-deleted rows are purged after the retention period defined by
  the consumer's data-retention policy. The engine never auto-purges.
