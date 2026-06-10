# PostgreSQL — DDL Conventions

Target: **PostgreSQL 14+** (14.0+ for `gen_random_uuid()` in core;
15.0+ for `MERGE`; 15.0+ for `NULLS NOT DISTINCT` in unique
constraints; 16.0+ for `SQL/JSON` improvements).

The reference adapter implementing these conventions is
`educore-storage-postgres`. The DDL strings in this file are
emitted by `PostgresStorageAdapter::create_<table>_ddl()`.

## Identifier quoting

Use **double quotes** for every identifier:

```sql
CREATE TABLE "outbox" (
  "event_id" UUID NOT NULL,
  ...
);
```

PG identifiers are case-sensitive when quoted. The engine always
uses lowercase identifiers.

## Default settings for every table

PG has no per-table engine or charset. The database is `UTF8`
encoding. The engine uses `en_US.utf8` collation by default (the
consumer's choice).

```sql
CREATE TABLE "outbox" (
  ...
);
```

There is no `ENGINE=` or `DEFAULT CHARSET=` clause.

## Type mapping

PG has a rich type system. The engine's canonical types and their
PG forms:

| Engine type | PostgreSQL type | Notes |
| --- | --- | --- |
| `CHAR(36)` (UUIDv7) | `UUID` | **native** UUID type; engine emits `UUID NOT NULL` |
| `BINARY(16)` | `BYTEA` | 16 bytes |
| `BIGINT` | `BIGINT` | 8-byte signed |
| `INT` | `INTEGER` | 4-byte signed |
| `TINYINT` | `SMALLINT` (with `CHECK` range) | engine uses `BOOLEAN` for booleans and `SMALLINT` for 1-byte ints |
| `BOOLEAN` | `BOOLEAN` | native |
| `VARCHAR(N)` | `VARCHAR(N)` | PG has no length limit on `TEXT` but the engine prefers `VARCHAR(N)` for indices |
| `TEXT` | `TEXT` | long-form text |
| `TIMESTAMP` | `TIMESTAMPTZ` | PG's `TIMESTAMP WITH TIME ZONE`; engine always uses `TIMESTAMPTZ` for UTC |
| `DATETIME` | `TIMESTAMP` (no TZ) | engine uses for date+time without timezone semantics (rare) |
| `DATE` | `DATE` | native |
| `TIME` | `TIME` | native |
| `JSON` | `JSONB` | **native** JSONB; engine emits `JSONB NOT NULL CHECK (jsonb_typeof("payload") = 'object')` |
| `DECIMAL(P,S)` | `NUMERIC(P,S)` | engine uses for money (P=14, S=2) and quantities |
| `ENUM` | `CREATE TYPE ... AS ENUM` | engine prefers `VARCHAR(N) NOT NULL` with `CHECK` constraint (portable) |

### Why `UUID` not `CHAR(36)`

The engine's canonical id type is `UUID` (UUIDv7) per RFC 9562.
PG has a native `UUID` type that is 16 bytes (more compact than
`CHAR(36)` at 36 bytes), indexed natively, and accepts the
hyphenated string form on input. The engine's `to_typed_uuid()`
function uses PG's `uuid` type.

The engine's "table parity" rule still holds: the **logical** name
is `CHAR(36)`, and the engine's adapter translates. The PG
storage uses `UUID`; the MySQL and SQLite storage use
`CHAR(36)` / `TEXT`.

The migration plan (`docs/schemas/data-migration/02-id-conversion.md`)
generates UUIDv7 strings from the legacy BIGINT. The string is
either stored as `CHAR(36)` (MySQL, SQLite) or cast to `UUID` on
insert (PG).

## Identifier lengths

PG's `NAMEDATALEN` is 64 by default; identifier lengths are
limited to 63 characters. The engine's longest identifier is
`idx_academic_student_records_school_active_version` (50 chars),
well within the limit.

## Indexes

```sql
CREATE INDEX "idx_<table>_<col1>_<col2>" ON "<table>" ("<col1>", "<col2>");
```

Same naming convention as MySQL. The engine's adapter creates the
mandatory `(school_id, active_status)` index on every aggregate.

PG supports additional index types: `BRIN`, `GIN`, `HASH`. The
engine does not use them; `BTREE` is the default and is
appropriate for the engine's access patterns.

## Foreign keys

```sql
ALTER TABLE "<child>"
  ADD CONSTRAINT "fk_<child>_<col>_<parent>"
  FOREIGN KEY ("<col>") REFERENCES "<parent>" ("id")
  ON DELETE RESTRICT
  ON UPDATE RESTRICT
  DEFERRABLE INITIALLY DEFERRED;
```

The engine's default referential action is `ON DELETE RESTRICT`.
The `DEFERRABLE INITIALLY DEFERRED` clause is PG-specific and
allows the FK check to be deferred to the end of the transaction.
The engine uses this so that the engine's domain logic can mutate
the parent and child in any order within a transaction.

The engine's `database-schema.md` § 4 enumerates the exceptions
(derived / owned-child rows may use `CASCADE`; advisory references
may use `SET NULL`).

## Row-level security

PG has the most expressive RLS of the three backends. The engine
**requires** RLS as a defense-in-depth:

```sql
-- Enable RLS on the table
ALTER TABLE "<aggregate>" ENABLE ROW LEVEL SECURITY;

-- Force RLS even for the table owner (recommended)
ALTER TABLE "<aggregate>" FORCE ROW LEVEL SECURITY;

-- Policy: the school_id must match the session's current school
CREATE POLICY "school_isolation_<aggregate>" ON "<aggregate>"
  USING ("school_id" = current_setting('app.current_school_id')::UUID)
  WITH CHECK ("school_id" = current_setting('app.current_school_id')::UUID);

-- Set the per-session variable on every connection
SET LOCAL app.current_school_id = '<uuid>';
```

The engine's adapter issues `SET LOCAL app.current_school_id = ?`
on every new transaction. The `SET LOCAL` scope is the current
transaction, so the setting is automatically cleared at
`COMMIT`/`ROLLBACK`.

For cross-tenant operations, the adapter issues
`SET LOCAL app.bypass_rls = 'on'` and adds a `BYPASSRLS`-style
policy. The `Platform.CrossTenant` capability is required.

The `BYPASSRLS` attribute is set on the engine's database role:

```sql
ALTER ROLE educore_writer BYPASSRLS;
```

The consumer's `educore_writer` role is the engine's write
role. The consumer's `educore_reader` role does NOT have
`BYPASSRLS`; it relies on the RLS policies.

## `CHECK` constraints

```sql
CREATE TABLE "rbac_roles" (
  ...
  "role_type" VARCHAR(16) NOT NULL CHECK ("role_type" IN ('system', 'custom')),
  ...
);
```

PG enforces `CHECK` constraints. The engine emits them on enum-like
columns.

## Schemas (optional)

PG supports **schemas** as namespaces. The engine's contract is
**table parity** (the same table name in all backends), so
schemas are a **consumer-side** choice. A consumer may:

- Use the default schema (no schema prefix).
- Wrap the engine tables in an `engine` schema
  (`engine.outbox`, `engine.audit_log`, etc.).
- Wrap each domain in its own schema (`academic.students`,
  `finance.invoices`, etc.).

The engine's adapter handles all three via `search_path`:

```sql
SET search_path = engine, academic, finance, hr, ..., public;
```

The default `search_path` is `$user, public` (the current user,
then `public`). The consumer's setup may override:

```sql
ALTER ROLE educore_writer SET search_path = engine, public;
ALTER DATABASE devdb_v2 SET search_path = engine, public;
```

The engine's adapter issues `SET search_path` on every new
connection as a belt-and-suspenders measure.

The engine's `Comparison` table (`comparison.md`) notes that PG is
the only backend that supports schemas. MySQL has database-level
namespaces (different databases, not schemas within one), and
SQLite has ATTACH DATABASE for multiple file-backed databases.

## Transactions

PG uses `START TRANSACTION` (or `BEGIN`) and `COMMIT` / `ROLLBACK`.
The default isolation level is `READ COMMITTED`. The engine's
adapter reads with `SELECT ... FOR UPDATE` for command-dispatch
row locks.

For SERIALIZABLE isolation (the engine's optional strict mode):

```sql
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
START TRANSACTION;
-- ... commands ...
COMMIT;
```

The engine's `Transaction::serializable()` method issues this on
transactions that need it. The default is `READ COMMITTED`.

## The 6 engine cross-cutting tables — PostgreSQL DDL

The adapter emits the dialect-specific DDL via
`PostgresStorageAdapter::create_<table>_ddl()`. The MySQL form in
`migrations/engine/0000_engine_core.mysql.sql` is the reference; the PG form is
documented per-table here.

### `outbox`

```sql
CREATE TABLE IF NOT EXISTS "outbox" (
  "event_id"        UUID         NOT NULL,
  "event_type"      VARCHAR(191) NOT NULL,
  "event_version"   INTEGER      NOT NULL,
  "school_id"       UUID         NOT NULL,
  "aggregate_id"    UUID         NOT NULL,
  "aggregate_type"  VARCHAR(64)  NOT NULL,
  "actor_id"        UUID         NOT NULL,
  "correlation_id"  UUID         NOT NULL,
  "causation_id"    UUID,
  "occurred_at"     TIMESTAMPTZ  NOT NULL,
  "recorded_at"     TIMESTAMPTZ  NOT NULL,
  "payload"         JSONB        NOT NULL CHECK (jsonb_typeof("payload") = 'object'),
  "enqueued_at"     TIMESTAMPTZ  NOT NULL,
  "published_at"    TIMESTAMPTZ,
  "attempts"        INTEGER      NOT NULL DEFAULT 0,
  "last_error"      TEXT,
  PRIMARY KEY ("event_id")
);

CREATE INDEX IF NOT EXISTS "idx_outbox_school_enqueued"
  ON "outbox" ("school_id", "enqueued_at");
CREATE INDEX IF NOT EXISTS "idx_outbox_published"
  ON "outbox" ("published_at", "enqueued_at");
CREATE INDEX IF NOT EXISTS "idx_outbox_aggregate"
  ON "outbox" ("aggregate_type", "aggregate_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_outbox_correlation"
  ON "outbox" ("correlation_id");
```

### `audit_log`

```sql
CREATE TABLE IF NOT EXISTS "audit_log" (
  "audit_id"        UUID         NOT NULL,
  "school_id"       UUID         NOT NULL,
  "actor_id"        UUID         NOT NULL,
  "actor_type"      VARCHAR(16)  NOT NULL,
  "action"          VARCHAR(191) NOT NULL,
  "resource_type"   VARCHAR(64)  NOT NULL,
  "resource_id"     UUID         NOT NULL,
  "event_id"        UUID,
  "command_id"      UUID,
  "correlation_id"  UUID         NOT NULL,
  "occurred_at"     TIMESTAMPTZ  NOT NULL,
  "recorded_at"     TIMESTAMPTZ  NOT NULL,
  "ip"              INET,
  "user_agent"      VARCHAR(512),
  "session_id"      UUID,
  "before_snapshot" JSONB        CHECK ("before_snapshot" IS NULL OR jsonb_typeof("before_snapshot") = 'object'),
  "after_snapshot"  JSONB        CHECK ("after_snapshot" IS NULL OR jsonb_typeof("after_snapshot") = 'object'),
  "metadata"        JSONB        CHECK ("metadata" IS NULL OR jsonb_typeof("metadata") = 'object'),
  "cross_tenant"    BOOLEAN      NOT NULL DEFAULT FALSE,
  "source"          VARCHAR(16)  NOT NULL,
  PRIMARY KEY ("audit_id")
);

CREATE INDEX IF NOT EXISTS "idx_audit_log_school_time"
  ON "audit_log" ("school_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_audit_log_actor"
  ON "audit_log" ("actor_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_audit_log_resource"
  ON "audit_log" ("resource_type", "resource_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_audit_log_correlation"
  ON "audit_log" ("correlation_id");
CREATE INDEX IF NOT EXISTS "idx_audit_log_action"
  ON "audit_log" ("action", "occurred_at");
```

### `idempotency`

```sql
CREATE TABLE IF NOT EXISTS "idempotency" (
  "school_id"        UUID         NOT NULL,
  "command_type"     VARCHAR(191) NOT NULL,
  "idempotency_key"  UUID         NOT NULL,
  "command_id"       UUID         NOT NULL,
  "outcome"          JSONB        NOT NULL CHECK (jsonb_typeof("outcome") = 'object'),
  "recorded_at"      TIMESTAMPTZ  NOT NULL,
  "expires_at"       TIMESTAMPTZ  NOT NULL,
  PRIMARY KEY ("school_id", "command_type", "idempotency_key")
);

CREATE INDEX IF NOT EXISTS "idx_idempotency_expires"
  ON "idempotency" ("expires_at");
```

### `event_log`

```sql
CREATE TABLE IF NOT EXISTS "event_log" (
  "event_id"        UUID         NOT NULL,
  "event_type"      VARCHAR(191) NOT NULL,
  "event_version"   INTEGER      NOT NULL,
  "school_id"       UUID         NOT NULL,
  "aggregate_id"    UUID         NOT NULL,
  "aggregate_type"  VARCHAR(64)  NOT NULL,
  "actor_id"        UUID         NOT NULL,
  "correlation_id"  UUID         NOT NULL,
  "causation_id"    UUID,
  "occurred_at"     TIMESTAMPTZ  NOT NULL,
  "recorded_at"     TIMESTAMPTZ  NOT NULL,
  "payload"         JSONB        NOT NULL CHECK (jsonb_typeof("payload") = 'object'),
  PRIMARY KEY ("event_id")
);

CREATE INDEX IF NOT EXISTS "idx_event_log_school_time"
  ON "event_log" ("school_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_event_log_type_time"
  ON "event_log" ("event_type", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_event_log_aggregate"
  ON "event_log" ("aggregate_type", "aggregate_id", "occurred_at");
CREATE INDEX IF NOT EXISTS "idx_event_log_correlation"
  ON "event_log" ("correlation_id");
```

### `schema_registry`

```sql
CREATE TABLE IF NOT EXISTS "schema_registry" (
  "event_type"      VARCHAR(191) NOT NULL,
  "event_version"   INTEGER      NOT NULL,
  "schema_json"     JSONB        NOT NULL CHECK (jsonb_typeof("schema_json") = 'object'),
  "deprecated_at"   TIMESTAMPTZ,
  "migration_path"  TEXT,
  "registered_at"   TIMESTAMPTZ  NOT NULL,
  PRIMARY KEY ("event_type", "event_version")
);
```

### `system_user`

```sql
CREATE TABLE IF NOT EXISTS "system_user" (
  "id"           UUID         NOT NULL,
  "display_name" VARCHAR(200) NOT NULL,
  "active_status" BOOLEAN     NOT NULL DEFAULT TRUE,
  "created_at"   TIMESTAMPTZ  NOT NULL,
  PRIMARY KEY ("id")
);

INSERT INTO "system_user" ("id", "display_name", "active_status", "created_at")
VALUES ('00000000-0000-7000-8000-000000000001'::UUID, 'SYSTEM', TRUE, NOW())
ON CONFLICT ("id") DO NOTHING;
```

## A domain aggregate example: `academic_students`

```sql
CREATE TABLE IF NOT EXISTS "academic_students" (
  "id"                UUID         NOT NULL PRIMARY KEY,
  "school_id"         UUID         NOT NULL,
  "admission_number"  VARCHAR(64),
  "roll_number"       VARCHAR(32),
  "first_name"        VARCHAR(200) NOT NULL,
  "last_name"         VARCHAR(200),
  "full_name"         VARCHAR(200),
  "date_of_birth"     DATE,
  "email"             VARCHAR(200),
  "mobile"            VARCHAR(32),
  "admission_date"    DATE,
  "photo_storage_key" VARCHAR(191),
  "gender_id"         UUID,
  "blood_group_id"    UUID,
  "religion_id"       UUID,
  "class_id"          UUID,
  "section_id"        UUID,
  "academic_id"       UUID,
  "category_id"       UUID,
  "group_id"          UUID,
  "route_id"          UUID,
  "vehicle_id"        UUID,
  "dormitory_id"      UUID,
  "room_id"           UUID,
  "guardian_id"       UUID,
  "user_id"           UUID,
  "role_id"           UUID,
  "version"           BIGINT       NOT NULL DEFAULT 1,
  "etag"              CHAR(32)     NOT NULL,
  "last_event_id"     UUID,
  "correlation_id"    UUID,
  "source"            VARCHAR(16),
  "active_status"     BOOLEAN      NOT NULL DEFAULT TRUE,
  "created_at"        TIMESTAMPTZ  NOT NULL,
  "updated_at"        TIMESTAMPTZ  NOT NULL,
  "created_by"        UUID         NOT NULL,
  "updated_by"        UUID         NOT NULL,
  "id_v7_legacy"      BIGINT,
  "custom_fields"     JSONB        CHECK ("custom_fields" IS NULL OR jsonb_typeof("custom_fields") = 'object'),
  CONSTRAINT "fk_academic_students_school" FOREIGN KEY ("school_id")
    REFERENCES "platform_schools" ("id") ON DELETE RESTRICT,
  CONSTRAINT "fk_academic_students_class" FOREIGN KEY ("class_id")
    REFERENCES "academic_classes" ("id") ON DELETE RESTRICT,
  -- ... more FKs ...
  CONSTRAINT "ck_academic_students_id_length" CHECK (length("id"::text) = 36),
  CONSTRAINT "ck_academic_students_school_id_length" CHECK (length("school_id"::text) = 36)
);

-- Row-level security
ALTER TABLE "academic_students" ENABLE ROW LEVEL SECURITY;
ALTER TABLE "academic_students" FORCE ROW LEVEL SECURITY;

CREATE POLICY "school_isolation_academic_students" ON "academic_students"
  USING ("school_id" = current_setting('app.current_school_id', true)::UUID)
  WITH CHECK ("school_id" = current_setting('app.current_school_id', true)::UUID);

-- Indexes
CREATE INDEX IF NOT EXISTS "idx_academic_students_school_active"
  ON "academic_students" ("school_id", "active_status");
CREATE INDEX IF NOT EXISTS "idx_academic_students_last_event"
  ON "academic_students" ("last_event_id");
CREATE INDEX IF NOT EXISTS "idx_academic_students_correlation"
  ON "academic_students" ("correlation_id");
CREATE INDEX IF NOT EXISTS "idx_academic_students_school_admission"
  ON "academic_students" ("school_id", "admission_number");
```

## Adapter implementation notes

- The `PostgresStorage` adapter uses `sqlx` (or `tokio-postgres`) for
  the connection pool. `sqlx` 0.8+ is the recommended version.
- The adapter issues `SET LOCAL app.current_school_id = ?` and
  `SET LOCAL app.bypass_rls = 'off'` on every new transaction
  (or every new connection, depending on the connection model).
- The adapter's DDL emission is unit-tested against a real
  PostgreSQL via testcontainers or a local PG instance.
- The adapter's tests verify RLS by issuing `SELECT` with two
  different `app.current_school_id` settings and confirming that
  the result sets do not cross-contaminate.

## Encryption at rest

PG supports Transparent Data Encryption (TDE) via the
`pgcrypto` extension and full-disk encryption via the OS / cloud
provider. The engine does not bundle `pgcrypto`; the consumer's
deployment may enable it for column-level encryption.

For PII columns, the consumer may use `pgcrypto`'s
`pgp_sym_encrypt()`:

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Encrypt on write
UPDATE academic_students
SET "mobile" = pgp_sym_encrypt("mobile", 'encryption-key')
WHERE id = '...';

-- Decrypt on read
SELECT pgp_sym_decrypt("mobile"::bytea, 'encryption-key')
FROM academic_students
WHERE id = '...';
```

The engine does not own the encryption key; the consumer's
deployment does. The engine's adapter can wrap the column read
in a `pgp_sym_decrypt()` call, or the consumer can use a view
that decrypts on the fly.

## References

- PostgreSQL 14+ Documentation: `CREATE TABLE`, `UUID` type, `JSONB`
  type, `CREATE POLICY`, `TIMESTAMPTZ`.
- The `educore-storage-postgres` crate README.
- `docs/ports/storage.md` § 4: `Configuration` — the engine's
  `PostgresStorage::builder()` pattern.
- `docs/schemas/database-schema.md` § 11: the canonical minimum
  schema.
