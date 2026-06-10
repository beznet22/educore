# SQLite — DDL Conventions

Target: **SQLite 3.x** (3.38+ for `STRICT` tables; 3.39+ for
`JSONB`-like functions; 3.41+ for `RIGHT JOIN`).

The reference adapter implementing these conventions is
`educore-storage-sqlite`. The DDL strings in this file are
emitted by `SqliteStorageAdapter::create_<table>_ddl()`.

SQLite is the engine's embedded / offline mode. It runs in the
same process as the consumer (Tauri app, mobile app, CLI). There
is no network, no server, no separate process.

## Identifier quoting

SQLite supports both **backticks** and **double quotes**. The
engine uses **double quotes** for portability with PostgreSQL:

```sql
CREATE TABLE "outbox" (
  "event_id" TEXT NOT NULL PRIMARY KEY,
  ...
);
```

Identifiers are case-sensitive. The engine always uses lowercase
identifiers.

## Default settings for every table

SQLite has **no engine, no charset, no collation** at the table
level. The database file's encoding is the application
responsibility. The engine always writes the database file as
UTF-8.

```sql
CREATE TABLE "outbox" (
  ...
);
```

There is no `ENGINE=InnoDB` or `DEFAULT CHARSET=utf8mb4` clause.
The engine's adapter does not emit them.

## Type mapping

SQLite uses **dynamic typing** (manifest types, not strict types).
The engine prefers `STRICT` tables (3.37+) for engine integrity.

The engine's canonical types and their SQLite forms:

| Engine type | SQLite type (STRICT) | Notes |
| --- | --- | --- |
| `CHAR(36)` (UUIDv7) | `TEXT` (with `CHECK (length("id") = 36)`) | SQLite has no fixed-length CHAR; `TEXT` with length check |
| `BINARY(16)` | `BLOB` | 16 bytes |
| `BIGINT` | `INTEGER` | SQLite's `INTEGER` is 8-byte signed |
| `INT` | `INTEGER` | 8-byte signed |
| `TINYINT` | `INTEGER` (with `CHECK` range 0..255 or -128..127) | engine uses for booleans and small enums |
| `BOOLEAN` | `INTEGER` (with `CHECK IN (0,1)`) | alias |
| `VARCHAR(N)` | `TEXT` (no length check at column level; can CHECK) | SQLite has no length limit |
| `TEXT` | `TEXT` | long-form text |
| `TIMESTAMP` | `TEXT` (ISO 8601 UTC string) | SQLite has no native timestamp; the engine uses ISO 8601 strings |
| `DATETIME` | `TEXT` | same as TIMESTAMP |
| `DATE` | `TEXT` | `YYYY-MM-DD` |
| `TIME` | `TEXT` | `HH:MM:SS` |
| `JSON` | `TEXT` (with `JSON_VALID` check via `json_extract`) | SQLite stores JSON as text; the engine's adapter parses with `json_extract` |
| `DECIMAL(P,S)` | `TEXT` (engine stores as `rust_decimal` string) | SQLite has no native DECIMAL; the engine's `Money` type is the source of truth |
| `ENUM` | not used | `TEXT` with `CHECK` constraint |

The engine's `STRICT` tables declaration:

```sql
CREATE TABLE "outbox" (
  ...
) STRICT;
```

`STRICT` enforces the type affinity. Without it, SQLite allows
silent type coercion (e.g. inserting `'hello'` into an `INTEGER`
column). The engine refuses to write to non-`STRICT` tables.

## The `WITHOUT ROWID` option

For lookup-only tables (the engine's cross-cutting tables fit this
profile), the engine uses `WITHOUT ROWID`:

```sql
CREATE TABLE "outbox" (
  "event_id" TEXT NOT NULL PRIMARY KEY,
  ...
) WITHOUT ROWID, STRICT;
```

`WITHOUT ROWID` saves 4-8 bytes per row and is faster for
point-lookups. The engine's `outbox`, `event_log`, `audit_log`,
`idempotency`, and `schema_registry` are all `WITHOUT ROWID`.

The `system_user` table is a single-row lookup; it is `WITHOUT
ROWID` and uses `id TEXT NOT NULL PRIMARY KEY`.

## Indexes

```sql
CREATE INDEX "idx_<table>_<col1>_<col2>" ON "<table>" ("<col1>", "<col2>");
```

Same naming convention as MySQL. The engine's adapter creates the
mandatory `(school_id, active_status)` index on every aggregate.

## Foreign keys

SQLite requires `PRAGMA foreign_keys = ON;` per connection for FK
enforcement. The engine's adapter issues this `PRAGMA` on every
new connection.

```sql
CREATE TABLE "academic_students" (
  ...
  CONSTRAINT "fk_academic_students_school" FOREIGN KEY ("school_id")
    REFERENCES "platform_schools" ("id") ON DELETE RESTRICT
);
```

`PRAGMA foreign_keys = ON` is connection-scoped, not database-
scoped. The adapter sets it on every connection. The test suite
also sets it.

## Row-level security

SQLite has **no RLS**. The engine enforces tenant isolation in the
application layer via the `school_id` filter on every query. The
adapter's `execute_query()` method injects `WHERE school_id = ?`
automatically.

The `school_id` filter is the **only** tenant-isolation mechanism
in SQLite. The adapter's tests verify the filter on every query
method.

## `CHECK` constraints

```sql
CREATE TABLE "rbac_roles" (
  ...
  "role_type" TEXT NOT NULL CHECK ("role_type" IN ('system', 'custom')),
  ...
);
```

SQLite enforces `CHECK` constraints since 3.3.0. The engine emits
them on enum-like columns.

## Transactions

SQLite uses `BEGIN` (or `BEGIN DEFERRED` / `BEGIN IMMEDIATE` /
`BEGIN EXCLUSIVE`) and `COMMIT` / `ROLLBACK`. The default is
`DEFERRED`. The engine's adapter uses `BEGIN IMMEDIATE` for
command dispatch (acquires the write lock at the start of the
transaction; avoids `SQLITE_BUSY` at the upgrade-to-write step).

```sql
BEGIN IMMEDIATE;
-- ... commands ...
COMMIT;
-- or ROLLBACK;
```

The engine's `WAL` (Write-Ahead Logging) mode is enabled by default
in the adapter:

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
```

`WAL` allows concurrent readers + a single writer, which is
important for the engine's offline-mode use case (a Tauri app
reading from SQLite while the user types).

## The 6 engine cross-cutting tables — SQLite DDL

The adapter emits the dialect-specific DDL via
`SqliteStorageAdapter::create_<table>_ddl()`. The MySQL form in
`migrations/engine/0000_engine_core.mysql.sql` is the reference; the SQLite
form is documented per-table here.

### `outbox`

```sql
CREATE TABLE IF NOT EXISTS "outbox" (
  "event_id"        TEXT NOT NULL PRIMARY KEY,
  "event_type"      TEXT NOT NULL,
  "event_version"   INTEGER NOT NULL,
  "school_id"       TEXT NOT NULL,
  "aggregate_id"    TEXT NOT NULL,
  "aggregate_type"  TEXT NOT NULL,
  "actor_id"        TEXT NOT NULL,
  "correlation_id"  TEXT NOT NULL,
  "causation_id"    TEXT,
  "occurred_at"     TEXT NOT NULL,
  "recorded_at"     TEXT NOT NULL,
  "payload"         TEXT NOT NULL CHECK (json_valid("payload")),
  "enqueued_at"     TEXT NOT NULL,
  "published_at"    TEXT,
  "attempts"        INTEGER NOT NULL DEFAULT 0,
  "last_error"      TEXT
) WITHOUT ROWID, STRICT;

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
  "audit_id"        TEXT NOT NULL PRIMARY KEY,
  "school_id"       TEXT NOT NULL,
  "actor_id"        TEXT NOT NULL,
  "actor_type"      TEXT NOT NULL,
  "action"          TEXT NOT NULL,
  "resource_type"   TEXT NOT NULL,
  "resource_id"     TEXT NOT NULL,
  "event_id"        TEXT,
  "command_id"      TEXT,
  "correlation_id"  TEXT NOT NULL,
  "occurred_at"     TEXT NOT NULL,
  "recorded_at"     TEXT NOT NULL,
  "ip"              TEXT,
  "user_agent"      TEXT,
  "session_id"      TEXT,
  "before_snapshot" TEXT CHECK ("before_snapshot" IS NULL OR json_valid("before_snapshot")),
  "after_snapshot"  TEXT CHECK ("after_snapshot" IS NULL OR json_valid("after_snapshot")),
  "metadata"        TEXT CHECK ("metadata" IS NULL OR json_valid("metadata")),
  "cross_tenant"    INTEGER NOT NULL DEFAULT 0 CHECK ("cross_tenant" IN (0,1)),
  "source"          TEXT NOT NULL
) WITHOUT ROWID, STRICT;

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
  "school_id"        TEXT NOT NULL,
  "command_type"     TEXT NOT NULL,
  "idempotency_key"  TEXT NOT NULL,
  "command_id"       TEXT NOT NULL,
  "outcome"          TEXT NOT NULL CHECK (json_valid("outcome")),
  "recorded_at"      TEXT NOT NULL,
  "expires_at"       TEXT NOT NULL,
  PRIMARY KEY ("school_id", "command_type", "idempotency_key")
) WITHOUT ROWID, STRICT;

CREATE INDEX IF NOT EXISTS "idx_idempotency_expires"
  ON "idempotency" ("expires_at");
```

### `event_log`

```sql
CREATE TABLE IF NOT EXISTS "event_log" (
  "event_id"        TEXT NOT NULL PRIMARY KEY,
  "event_type"      TEXT NOT NULL,
  "event_version"   INTEGER NOT NULL,
  "school_id"       TEXT NOT NULL,
  "aggregate_id"    TEXT NOT NULL,
  "aggregate_type"  TEXT NOT NULL,
  "actor_id"        TEXT NOT NULL,
  "correlation_id"  TEXT NOT NULL,
  "causation_id"    TEXT,
  "occurred_at"     TEXT NOT NULL,
  "recorded_at"     TEXT NOT NULL,
  "payload"         TEXT NOT NULL CHECK (json_valid("payload"))
) WITHOUT ROWID, STRICT;

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
  "event_type"      TEXT NOT NULL,
  "event_version"   INTEGER NOT NULL,
  "schema_json"     TEXT NOT NULL CHECK (json_valid("schema_json")),
  "deprecated_at"   TEXT,
  "migration_path"  TEXT,
  "registered_at"   TEXT NOT NULL,
  PRIMARY KEY ("event_type", "event_version")
) WITHOUT ROWID, STRICT;
```

### `system_user`

```sql
CREATE TABLE IF NOT EXISTS "system_user" (
  "id"           TEXT NOT NULL PRIMARY KEY,
  "display_name" TEXT NOT NULL,
  "active_status" INTEGER NOT NULL DEFAULT 1 CHECK ("active_status" IN (0,1)),
  "created_at"   TEXT NOT NULL
) WITHOUT ROWID, STRICT;

INSERT OR IGNORE INTO "system_user" ("id", "display_name", "active_status", "created_at")
VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
```

## A domain aggregate example: `academic_students`

```sql
CREATE TABLE IF NOT EXISTS "academic_students" (
  "id"                TEXT NOT NULL PRIMARY KEY,
  "school_id"         TEXT NOT NULL,
  "admission_number"  TEXT,
  "roll_number"       TEXT,
  "first_name"        TEXT NOT NULL,
  "last_name"         TEXT,
  "full_name"         TEXT,
  "date_of_birth"     TEXT,
  "email"             TEXT,
  "mobile"            TEXT,
  "admission_date"    TEXT,
  "photo_storage_key" TEXT,
  "gender_id"         TEXT,
  "blood_group_id"    TEXT,
  "religion_id"       TEXT,
  "class_id"          TEXT,
  "section_id"        TEXT,
  "academic_id"       TEXT,
  "category_id"       TEXT,
  "group_id"          TEXT,
  "route_id"          TEXT,
  "vehicle_id"        TEXT,
  "dormitory_id"      TEXT,
  "room_id"           TEXT,
  "guardian_id"       TEXT,
  "user_id"           TEXT,
  "role_id"           TEXT,
  "version"           INTEGER NOT NULL DEFAULT 1,
  "etag"              TEXT NOT NULL,
  "last_event_id"     TEXT,
  "correlation_id"    TEXT,
  "source"            TEXT,
  "active_status"     INTEGER NOT NULL DEFAULT 1 CHECK ("active_status" IN (0,1)),
  "created_at"        TEXT NOT NULL,
  "updated_at"        TEXT NOT NULL,
  "created_by"        TEXT NOT NULL,
  "updated_by"        TEXT NOT NULL,
  "id_v7_legacy"      INTEGER,
  "custom_fields"     TEXT CHECK ("custom_fields" IS NULL OR json_valid("custom_fields")),
  FOREIGN KEY ("school_id") REFERENCES "platform_schools" ("id") ON DELETE RESTRICT,
  FOREIGN KEY ("class_id") REFERENCES "academic_classes" ("id") ON DELETE RESTRICT,
  -- ... more FKs ...
  CHECK (length("id") = 36),
  CHECK (length("school_id") = 36)
) STRICT;

CREATE INDEX IF NOT EXISTS "idx_academic_students_school_active"
  ON "academic_students" ("school_id", "active_status");
CREATE INDEX IF NOT EXISTS "idx_academic_students_last_event"
  ON "academic_students" ("last_event_id");
CREATE INDEX IF NOT EXISTS "idx_academic_students_correlation"
  ON "academic_students" ("correlation_id");
CREATE INDEX IF NOT EXISTS "idx_academic_students_school_admission"
  ON "academic_students" ("school_id", "admission_number");
```

The `length("id") = 36` and `length("school_id") = 36` checks
are the engine's UUIDv7 length invariant. The adapter issues them
on every aggregate.

## Adapter implementation notes

- The `SqliteStorage` adapter uses `rusqlite` for the connection.
  `rusqlite` 0.31+ is the recommended version.
- The adapter emits DDL lazily; the consumer's migration runner
  applies it.
- The adapter issues `PRAGMA foreign_keys = ON`,
  `PRAGMA journal_mode = WAL`, and `PRAGMA synchronous = NORMAL`
  on every new connection.
- The adapter enforces `school_id` in the application layer via
  the `WHERE` clause injection. The consumer does not need to
  write `WHERE school_id = ?`; the adapter does it.
- The adapter's tests run against an in-memory SQLite via
  `rusqlite::Connection::open_in_memory()`. The DDL is verified
  before any test queries run.

## Encryption at rest

SQLite supports database-level encryption via the
`SQLite Encryption Extension` (SEE) or `SQLCipher`. The engine
does not bundle either; the consumer's deployment may use them
for sensitive data.

The engine's adapter exposes a `key` parameter on the connection
builder:

```rust
SqliteStorage::open("path/to/db.sqlite")?
    .with_key(b"encryption-key-32-bytes-xxx")?  // optional
```

If `with_key` is set, the adapter issues `PRAGMA key = '...'` on
every connection. The consumer is responsible for the key
management (typically via the OS keychain or a hardware security
module).

## References

- SQLite Documentation: `CREATE TABLE`, `STRICT` tables, `WITHOUT
  ROWID`, `json_valid`/`json_extract`, `PRAGMA`.
- The `educore-storage-sqlite` crate README.
- `docs/ports/storage.md` § 4: `Configuration` — the engine's
  `SqliteStorage::open()` pattern.
- `docs/schemas/database-schema.md` § 11: the canonical minimum
  schema.
