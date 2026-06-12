# SurrealDB — DDL Conventions

Target: **SurrealDB 3.x** (3.0+ for `DEFINE FIELD` schema-level
types, `DEFINE EVENT` in-DB triggers, `LIVE SELECT` change-data
capture, record-link traversal, vector search `<|N,COSINE|>`,
`DEFINE ACCESS ... TYPE RECORD` auth).

The reference adapter implementing these conventions is
`educore-storage-surrealdb`. The DDL strings in this file are
emitted by `SurrealStorageAdapter::create_<table>_ddl()`.

> **Status — deferred adapter.** SurrealDB and MongoDB are
> **deferred to a future release** and are **not shipped** from
> the engine. This document records the dialect conventions the
> adapter author will follow when the adapter is built. The
> current three shipped adapters are PostgreSQL, MySQL, and
> SQLite. See `docs/ports/storage.md#future-storage-backends-deferred`
> for the rationale and the path for consumers who need this
> adapter in-tree.

SurrealDB is a multi-model database that combines a document
model, a graph model, and a relational model behind a single
query language (SurrealQL). The engine treats it as a **document
database with typed schemas** — the graph and vector features are
**not** used by the engine's port contract, but the adapter author
should know they exist (see § Parity surface).

## Identifier quoting

Use **backticks** for every identifier:

```sql
DEFINE TABLE outbox SCHEMAFUL
  PERMISSIONS
    FOR SELECT WHERE active = true
    FOR CREATE, UPDATE, DELETE NONE;

DEFINE FIELD event_id ON TABLE outbox TYPE string
  ASSERT $value != NONE AND string::length($value) = 36;
```

SurrealDB identifiers are case-sensitive. The engine always uses
lowercase snake_case identifiers, matching the canonical table
parity contract.

SurrealDB accepts both backticks and double quotes for
identifier quoting. The engine uses backticks to match the
MySQL adapter and to remain visually distinct from SurrealQL
string literals (which are single- or double-quoted).

## Default storage: in-process embedded

SurrealDB supports three execution modes:

| Mode | Engine | Network | Use case |
| --- | --- | --- | --- |
| **`memory`** | in-memory | none | tests, ephemeral state |
| **`rocksdb`** | embedded RocksDB | none | single-process desktop / mobile / CLI |
| **`tikv`** | TiKV cluster | yes | distributed deployment |

The engine's reference adapter targets **`memory` (tests) and
`rocksdb` (production single-process)**. The `tikv` mode is the
path to a true distributed SurrealDB; the engine treats it as
**out of scope for v1**. A consumer who needs distributed SurrealDB
should switch to SurrealDS (the managed SurrealDB product) or run
the engine against PostgreSQL.

The `rocksdb` mode is a single-process, single-writer, ACID-
compliant embedded database. It is the natural counterpart to
SQLite's `WITHOUT ROWID, STRICT` mode and is appropriate for
Tauri desktop apps, mobile apps, and CLI tools.

## Default types: native UUID, datetime, decimal, record links, arrays, objects

SurrealDB has a **rich native type system** that maps almost 1:1
onto the engine's canonical types:

| Engine type | SurrealDB type | Notes |
| --- | --- | --- |
| `CHAR(36)` (UUIDv7) | `string` (with `ASSERT string::length($value) = 36`) or `uuid` | SurrealDB 3.x has a `uuid` type; engine prefers the `string` form for parity with the other adapters and casts in queries |
| `BINARY(16)` | `bytes` | native |
| `BIGINT` | `int` | 64-bit signed |
| `INT` | `int` | 32-bit signed (engine emits `int`; SurrealDB coerces) |
| `TINYINT` | `int` (with `ASSERT $value >= 0 AND $value <= 255`) | engine uses for booleans and small enums |
| `BOOLEAN` | `bool` | **native** |
| `VARCHAR(N)` | `string` (with `ASSERT string::length($value) <= N`) | SurrealDB strings are not length-limited at the type level; the engine emits an `ASSERT` clause |
| `TEXT` | `string` | long-form text, no length assertion |
| `TIMESTAMP` | `datetime` | **native** nanosecond-precision UTC; engine stores UTC always |
| `DATETIME` | `datetime` | same |
| `DATE` | `string` (with `ASSERT $value = string::slice($value, 0, 10) AND string::matches($value, "^\\d{4}-\\d{2}-\\d{2}$")`) | SurrealDB has no `date`-only type; the engine asserts the ISO 8601 date format |
| `TIME` | `string` (with `ASSERT string::matches($value, "^\\d{2}:\\d{2}:\\d{2}$")`) | same |
| `JSON` | `object` / `array` / mixed | **native** — SurrealDB has object and array literals; the engine emits `object` for JSON-shaped payloads and `array` for arrays |
| `DECIMAL(P,S)` | `decimal` | **native** arbitrary-precision decimal; engine uses for money |
| `ENUM` | `string` (with `ASSERT $value IN { ... }`) | engine prefers `string + ASSERT IN` (portable with MySQL/Postgres `CHECK`) |
| Record link | `record<TABLE>` | **native** referential type; the engine uses this for FK columns |
| Optional | `option<T>` | **native** nullability marker; the engine uses `option<type>` for nullable columns |

The `record<TABLE>` type is the engine's FK column. When the
adapter emits `DEFINE FIELD class_id ON TABLE academic_students
TYPE record<academic_classes>`, SurrealDB enforces referential
integrity at write time — equivalent to the other adapters'
`FOREIGN KEY ... REFERENCES` clause, with no separate `ALTER
TABLE` statement required.

## DDL: `DEFINE TABLE`, `DEFINE FIELD`, `DEFINE INDEX`, `DEFINE EVENT`

SurrealDB's DDL is **per-statement** rather than the relational
`CREATE TABLE ... ( ... )` block. There is no single `CREATE
TABLE` that declares columns, indexes, triggers, and constraints
all at once. The engine's adapter emits the following statements
per aggregate:

```sql
-- 1. The table itself (one statement)
DEFINE TABLE outbox SCHEMAFUL
  PERMISSIONS FULL;

-- 2. One DEFINE FIELD per column (one statement each)
DEFINE FIELD event_id      ON TABLE outbox TYPE string
  ASSERT $value != NONE AND string::length($value) = 36;
DEFINE FIELD event_type    ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD event_version ON TABLE outbox TYPE int
  ASSERT $value != NONE AND $value > 0;
-- ...

-- 3. One DEFINE INDEX per index (one statement each)
DEFINE INDEX idx_outbox_school_enqueued ON TABLE outbox
  COLUMNS school_id, enqueued_at;
DEFINE INDEX idx_outbox_published ON TABLE outbox
  COLUMNS published_at, enqueued_at;

-- 4. Optional DEFINE EVENT for in-DB triggers (none for the engine's
--    cross-cutting tables; see § DEFINE EVENT below for the rationale)
```

This is fundamentally different from the relational `CREATE TABLE
... ( ... )` block. The adapter author must:
- Iterate the macro-emitted AST in dependency order: `DEFINE TABLE`
  first (parents before children — SurrealDB does not enforce FK
  ordering at `DEFINE TABLE` time, but a `DEFINE FIELD ... TYPE
  record<child>` on a parent emits an implicit `DEFINE TABLE child`
  reference).
- Emit one `DEFINE FIELD` per column, in column order.
- Emit one `DEFINE INDEX` per index, after all `DEFINE FIELD`s
  for the table (indexes reference columns by name, which must
  exist).
- Coalesce `DEFINE TABLE` + `DEFINE FIELD` + `DEFINE INDEX` per
  table so the engine can roll back a partial table creation on
  failure (SurrealDB transactions are **single-statement
  ACID** — the adapter wraps a per-table DDL block in a
  `BEGIN / COMMIT / CANCEL` block to make it atomic).

### `SCHEMAFUL` vs `SCHEMALESS`

The engine uses **`SCHEMAFUL`** for every table. `SCHEMAFUL`
enforces that every row has the declared fields with the declared
types. `SCHEMALESS` (the default) allows arbitrary fields per row
— the engine rejects this for the cross-cutting tables and the
domain aggregates.

```sql
DEFINE TABLE outbox SCHEMAFUL
  PERMISSIONS FULL;
```

`SCHEMALESS` is allowed only for the engine's `custom_fields`
column, which is itself typed `object` and validated by the
application layer.

## Type mapping — engine invariants to SurrealDB

The engine's seven invariant columns map to SurrealDB as follows:

| Engine invariant | SurrealDB form |
| --- | --- |
| `id CHAR(36) PRIMARY KEY` | `DEFINE FIELD id ON TABLE <t> TYPE string ASSERT string::length($value) = 36` (and the table's `id` field is implicit — SurrealDB auto-creates `id` on every row as `record<TABLE>`) |
| `school_id CHAR(36) NOT NULL` | `DEFINE FIELD school_id ON TABLE <t> TYPE string ASSERT string::length($value) = 36` |
| `version BIGINT NOT NULL DEFAULT 1` | `DEFINE FIELD version ON TABLE <t> TYPE int ASSERT $value >= 1 DEFAULT 1` |
| `etag CHAR(32) NOT NULL` | `DEFINE FIELD etag ON TABLE <t> TYPE string ASSERT string::length($value) = 32` |
| `last_event_id CHAR(36) NULL` | `DEFINE FIELD last_event_id ON TABLE <t> TYPE option<string>` |
| `correlation_id CHAR(36) NULL` | `DEFINE FIELD correlation_id ON TABLE <t> TYPE option<string>` |
| `source VARCHAR(16) NULL` | `DEFINE FIELD source ON TABLE <t> TYPE option<string>` |
| `active_status TINYINT NOT NULL DEFAULT 1` | `DEFINE FIELD active_status ON TABLE <t> TYPE bool DEFAULT true` |
| `created_at TIMESTAMP NOT NULL` | `DEFINE FIELD created_at ON TABLE <t> TYPE datetime ASSERT $value != NONE VALUE $value OR time::now()` |
| `updated_at TIMESTAMP NOT NULL` | `DEFINE FIELD updated_at ON TABLE <t> TYPE datetime ASSERT $value != NONE VALUE time::now()` |
| `created_by CHAR(36) NOT NULL` | `DEFINE FIELD created_by ON TABLE <t> TYPE string ASSERT string::length($value) = 36` |
| `updated_by CHAR(36) NOT NULL` | `DEFINE FIELD updated_by ON TABLE <t> TYPE string ASSERT string::length($value) = 36` |
| `id_v7_legacy BIGINT UNSIGNED NULL` | `DEFINE FIELD id_v7_legacy ON TABLE <t> TYPE option<int>` |

The `(school_id, active_status)` composite index:

```sql
DEFINE INDEX idx_<table>_school_active ON TABLE <table>
  COLUMNS school_id, active_status;
```

The engine emits this `DEFINE INDEX` on every aggregate.

## Identifier lengths

SurrealDB has no per-identifier length limit. The engine's
longest identifier
(`idx_academic_student_records_school_active_version`, 50 chars)
is well within any reasonable limit.

## Foreign keys via `record<TABLE>`

SurrealDB has **no separate `FOREIGN KEY` clause**. Foreign keys
are encoded in the **column type**: `record<academic_classes>` on
the `class_id` column. SurrealDB enforces referential integrity
on write:

```sql
DEFINE FIELD class_id ON TABLE academic_students
  TYPE option<record<academic_classes>>;
```

The engine's referential action is `ON DELETE RESTRICT` (the
default in SurrealDB — a `DELETE` on `academic_classes` will fail
if `academic_students` rows reference it). For derived / owned-
child rows that may use `ON DELETE CASCADE`, the adapter author
emits a `DEFINE EVENT` (see § DEFINE EVENT below). Advisory
references (`ON DELETE SET NULL`) are encoded as
`option<record<TABLE>>` plus a `DEFINE EVENT` that sets the field
to `NONE` on parent delete.

There is no need for a separate `ALTER TABLE ... ADD CONSTRAINT`
step — the column type **is** the constraint.

## Row-level security

SurrealDB supports **per-table `PERMISSIONS`** clauses that
filter reads, writes, and updates by predicate. The engine uses
this for tenant isolation as a defense-in-depth:

```sql
DEFINE TABLE academic_students SCHEMAFUL
  PERMISSIONS
    FOR SELECT WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR CREATE WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR UPDATE WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR DELETE WHERE school_id = $auth.school_id OR $auth.bypass = true;
```

The `$auth` namespace is the **session-scoped authentication
context**, populated by the consumer's `DEFINE ACCESS` setup
(see § DEFINE ACCESS). The engine requires the consumer's session
to set `$auth.school_id` on connect; the adapter issues this
automatically.

For cross-tenant operations, the consumer's session sets
`$auth.bypass = true`; the engine's `Platform.CrossTenant`
capability is required. This is the SurrealDB equivalent of
PostgreSQL's `BYPASSRLS` attribute.

The engine **also** enforces `school_id` in the application layer
via the `WHERE school_id = ?` filter injected by the adapter's
`execute_query()` method. The `PERMISSIONS` clause is the
database-level second line of defense; the application-layer
filter is the first.

## `CHECK` constraints via `ASSERT` clauses

SurrealDB has no separate `CHECK` constraint. The equivalent is
the `ASSERT` clause on `DEFINE FIELD`:

```sql
DEFINE FIELD role_type ON TABLE rbac_roles TYPE string
  ASSERT $value IN { "system", "custom" };
```

The engine emits `ASSERT` clauses on enum-like columns. The
assertion is checked on every `INSERT` and `UPDATE`.

For cross-column checks (the engine has none at the schema
level, but a consumer may add them), use a `DEFINE EVENT`:

```sql
DEFINE EVENT check_end_after_start ON TABLE academic_exams
  WHEN $event = "CREATE" OR $event = "UPDATE"
  THEN {
    IF $before.start_date > $after.end_date {
      THROW "end_date must be on or after start_date";
    };
  };
```

## Transactions — single-statement ACID

SurrealDB's transactions are **single-statement ACID**. A single
`CREATE`, `UPDATE`, `DELETE`, `INSERT`, or `UPSERT` statement is
atomic; multi-statement transactions require an explicit
`BEGIN / COMMIT / CANCEL` block:

```sql
BEGIN TRANSACTION;

-- ... multiple statements ...
CREATE academic_students:abc SET ...;
CREATE outbox:evt1 SET ...;
RELATE academic_students:abc -> has_event -> outbox:evt1;

COMMIT TRANSACTION;
-- or CANCEL TRANSACTION;
```

The engine's `Transaction` abstraction issues
`BEGIN TRANSACTION` at the start of the aggregate-mutate
operation and `COMMIT` / `CANCEL` at the end. The engine's
outbox pattern (write to the aggregate + write to `outbox` in the
same transaction) translates directly.

For high-concurrency command dispatch, the engine's adapter
issues `SELECT ... FROM <table> WHERE id = ... FOR UPDATE`
semantics via the `UPDATE ... WHERE ... RETURNING ...` pattern
(or the equivalent SurrealQL `UPDATE ... SET version = version + 1
WHERE id = ... AND version = ?` optimistic-concurrency form). The
engine's `version` column is the canonical optimistic-concurrency
token, exactly as in the relational adapters.

## `DEFINE EVENT` — the trigger equivalent

`DEFINE EVENT` is SurrealDB's in-DB trigger primitive. It fires
on `CREATE`, `UPDATE`, or `DELETE` of a row and runs a SurrealQL
block. The engine does **not** use `DEFINE EVENT` for the
cross-cutting tables (outbox drain, audit, etc. happen in the
application layer via the port implementations), but the adapter
author should know it exists because:

1. **A consumer may opt in** to in-DB outbox fan-out by emitting
   a `DEFINE EVENT` on the aggregate that writes to the `outbox`
   table within the same transaction:

   ```sql
   DEFINE EVENT outbox_on_academic_students ON TABLE academic_students
     WHEN $event = "CREATE" OR $event = "UPDATE" OR $event = "DELETE"
     THEN {
       CREATE outbox SET
         event_id = $after.last_event_id,
         event_type = "academic.student." + $event,
         school_id = $after.school_id,
         aggregate_id = $after.id,
         aggregate_type = "academic_students",
         payload = $after,
         occurred_at = time::now(),
         recorded_at = time::now(),
         enqueued_at = time::now();
     };
   ```

2. **Cross-domain fan-out** (e.g. an HR event triggers a
   finance invoice) is more naturally expressed in SurrealDB via
   `DEFINE EVENT` than in the relational adapters (which do the
   fan-out in the application layer). The consumer's choice; the
   engine does not require either pattern.

3. **The cascade-delete** for owned-child rows uses `DEFINE EVENT`
   in the same transaction:

   ```sql
   DEFINE EVENT cascade_delete_enrollments ON TABLE academic_students
     WHEN $event = "DELETE"
     THEN {
       DELETE academic_enrollments WHERE student_id = $before.id;
     };
   ```

The engine's `database-schema.md` § 4 enumerates the FK actions
that are eligible for cascade / set-null. The adapter author
emits `DEFINE EVENT` for these and not for the `ON DELETE
RESTRICT` cases.

## `DEFINE ACCESS` — native auth (out of scope for the storage adapter)

SurrealDB has **native auth** via `DEFINE ACCESS ... TYPE RECORD`:

```sql
DEFINE ACCESS system ON DATABASE TYPE RECORD
  SIGNUP (
    CREATE user SET
      email = $email,
      password = crypto::argon2::generate($password)
  )
  SIGNIN (
    SELECT * FROM user WHERE email = $email
      AND crypto::argon2::compare(password, $password)
  )
  DURATION FOR SESSION 12h, FOR TOKEN 1h;
```

The engine's **storage adapter does not use this**. The
engine's auth model is the `educore-auth` port adapter, which
implements auth in the application layer and passes the resulting
`UserId` / `SchoolId` / session token to the storage adapter
via the `$auth` namespace at the start of every transaction.
This keeps the engine's port contract uniform across all
storage adapters (no backdoor into the storage layer's native
auth).

A consumer who wants to use SurrealDB's native auth directly
(without the engine's auth port) can do so at the application
boundary; the storage adapter does not read or write `DEFINE
ACCESS` statements. This is documented here for completeness
because it is a common point of confusion.

## Outbox: regular table, written within the same transaction

The engine's outbox pattern is **a regular table written within
the same transaction as the aggregate mutation** — identical to
the MySQL / Postgres / SQLite pattern. The engine does not use
SurrealDB's change-data-capture (`LIVE SELECT`) for the outbox
itself, because:

- The outbox is the **engine's contract** with the consumer's
  event bus. A consumer that uses `LIVE SELECT` instead of
  polling the outbox gets the data, but it bypasses the engine's
  retry / dead-letter / idempotency controls.
- `LIVE SELECT` is best-effort (the consumer must handle
  reconnection, missed events, etc.) — the outbox is durable
  (rows are committed to disk within the same transaction as
  the aggregate mutation and only deleted after the consumer
  acknowledges).

The outbox table in SurrealDB:

```surql
DEFINE TABLE outbox SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD event_id       ON TABLE outbox TYPE string
  ASSERT $value != NONE AND string::length($value) = 36;
DEFINE FIELD event_type     ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD event_version  ON TABLE outbox TYPE int
  ASSERT $value != NONE AND $value > 0;
DEFINE FIELD school_id      ON TABLE outbox TYPE string
  ASSERT $value != NONE AND string::length($value) = 36;
DEFINE FIELD aggregate_id   ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD aggregate_type ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD actor_id       ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD correlation_id ON TABLE outbox TYPE string
  ASSERT $value != NONE;
DEFINE FIELD causation_id   ON TABLE outbox TYPE option<string>;
DEFINE FIELD occurred_at    ON TABLE outbox TYPE datetime
  ASSERT $value != NONE;
DEFINE FIELD recorded_at    ON TABLE outbox TYPE datetime
  ASSERT $value != NONE;
DEFINE FIELD payload        ON TABLE outbox TYPE object
  ASSERT $value != NONE;
DEFINE FIELD enqueued_at    ON TABLE outbox TYPE datetime
  ASSERT $value != NONE;
DEFINE FIELD published_at   ON TABLE outbox TYPE option<datetime>;
DEFINE FIELD attempts       ON TABLE outbox TYPE int
  ASSERT $value >= 0 DEFAULT 0;
DEFINE FIELD last_error     ON TABLE outbox TYPE option<string>;

DEFINE INDEX idx_outbox_school_enqueued ON TABLE outbox
  COLUMNS school_id, enqueued_at;
DEFINE INDEX idx_outbox_published ON TABLE outbox
  COLUMNS published_at, enqueued_at;
DEFINE INDEX idx_outbox_aggregate ON TABLE outbox
  COLUMNS aggregate_type, aggregate_id, occurred_at;
DEFINE INDEX idx_outbox_correlation ON TABLE outbox
  COLUMNS correlation_id;
```

`PERMISSIONS NONE` on `outbox` is correct — the engine writes to
it from the application layer, never from user sessions. The
adapter issues all `outbox` writes within the same
`BEGIN TRANSACTION` block as the aggregate mutation.

## Schema versioning — `VERSION d'...'` (engine does not use)

SurrealDB has a schema-versioning primitive via the `VERSION
d'YYYY-MM-DD'` clause:

```surql
DEFINE TABLE academic_students SCHEMAFUL VERSION d'2026-01-15'
  PERMISSIONS FULL;
```

This enables time-travel queries (`SELECT ... VERSION d'...'`)
and is part of SurrealDB's data-bronze / data-silver / data-gold
layering model.

**The engine does not use this.** The engine's schema-versioning
contract is the `schema_registry` table (one of the 6 cross-
cutting tables) plus the typed Rust struct (`aggregate.rs`) plus
the macro-emitted AST (`entities.rs`). The consumer's migration
runner reads the `schema_registry` rows and applies in-order; the
storage adapter does not need SurrealDB's native `VERSION` for
this.

The adapter author should know `VERSION d'...'` exists (so they
do not mistake it for a missing engine feature) but should not
emit it.

## The 6 engine cross-cutting tables — SurrealDB DDL

The adapter emits the dialect-specific DDL via
`SurrealStorageAdapter::create_<table>_ddl()`. The MySQL form in
`migrations/engine/0000_engine_core.mysql.sql` is the reference;
the SurrealDB form is documented per-table here.

### `outbox`

See the DDL block in § Outbox above.

### `audit_log`

```surql
DEFINE TABLE audit_log SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD audit_id        ON TABLE audit_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD school_id       ON TABLE audit_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD actor_id        ON TABLE audit_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD actor_type      ON TABLE audit_log TYPE string;
DEFINE FIELD action          ON TABLE audit_log TYPE string;
DEFINE FIELD resource_type   ON TABLE audit_log TYPE string;
DEFINE FIELD resource_id     ON TABLE audit_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD event_id        ON TABLE audit_log TYPE option<string>;
DEFINE FIELD command_id      ON TABLE audit_log TYPE option<string>;
DEFINE FIELD correlation_id  ON TABLE audit_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD occurred_at     ON TABLE audit_log TYPE datetime;
DEFINE FIELD recorded_at     ON TABLE audit_log TYPE datetime;
DEFINE FIELD ip              ON TABLE audit_log TYPE option<string>;
DEFINE FIELD user_agent      ON TABLE audit_log TYPE option<string>;
DEFINE FIELD session_id      ON TABLE audit_log TYPE option<string>;
DEFINE FIELD before_snapshot ON TABLE audit_log TYPE option<object>;
DEFINE FIELD after_snapshot  ON TABLE audit_log TYPE option<object>;
DEFINE FIELD metadata        ON TABLE audit_log TYPE option<object>;
DEFINE FIELD cross_tenant    ON TABLE audit_log TYPE bool DEFAULT false;
DEFINE FIELD source          ON TABLE audit_log TYPE string;

DEFINE INDEX idx_audit_log_school_time ON TABLE audit_log
  COLUMNS school_id, occurred_at;
DEFINE INDEX idx_audit_log_actor ON TABLE audit_log
  COLUMNS actor_id, occurred_at;
DEFINE INDEX idx_audit_log_resource ON TABLE audit_log
  COLUMNS resource_type, resource_id, occurred_at;
DEFINE INDEX idx_audit_log_correlation ON TABLE audit_log
  COLUMNS correlation_id;
DEFINE INDEX idx_audit_log_action ON TABLE audit_log
  COLUMNS action, occurred_at;
```

### `idempotency`

```surql
DEFINE TABLE idempotency SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD school_id       ON TABLE idempotency TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD command_type    ON TABLE idempotency TYPE string;
DEFINE FIELD idempotency_key ON TABLE idempotency TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD command_id      ON TABLE idempotency TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD outcome         ON TABLE idempotency TYPE object;
DEFINE FIELD recorded_at     ON TABLE idempotency TYPE datetime;
DEFINE FIELD expires_at      ON TABLE idempotency TYPE datetime;

DEFINE INDEX idx_idempotency_expires ON TABLE idempotency
  COLUMNS expires_at;
```

SurrealDB's primary key is the implicit `id` field (a
`record<idempotency>`). For the engine's composite PK
`(school_id, command_type, idempotency_key)`, the adapter
emits a `DEFINE INDEX` with `UNIQUE`:

```surql
DEFINE INDEX uniq_idempotency_key ON TABLE idempotency
  COLUMNS school_id, command_type, idempotency_key UNIQUE;
```

`INSERT ... ON CONFLICT IGNORE` is not the SurrealDB syntax; the
engine's adapter uses `UPSERT` with a `WHERE` clause that matches
the existing row, or a `CREATE` followed by a `SELECT` and
conditional `UPDATE` within the same `BEGIN TRANSACTION` block.

### `event_log`

```surql
DEFINE TABLE event_log SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD event_id       ON TABLE event_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD event_type     ON TABLE event_log TYPE string;
DEFINE FIELD event_version  ON TABLE event_log TYPE int;
DEFINE FIELD school_id      ON TABLE event_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD aggregate_id   ON TABLE event_log TYPE string;
DEFINE FIELD aggregate_type ON TABLE event_log TYPE string;
DEFINE FIELD actor_id       ON TABLE event_log TYPE string;
DEFINE FIELD correlation_id ON TABLE event_log TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD causation_id   ON TABLE event_log TYPE option<string>;
DEFINE FIELD occurred_at    ON TABLE event_log TYPE datetime;
DEFINE FIELD recorded_at    ON TABLE event_log TYPE datetime;
DEFINE FIELD payload        ON TABLE event_log TYPE object;

DEFINE INDEX idx_event_log_school_time ON TABLE event_log
  COLUMNS school_id, occurred_at;
DEFINE INDEX idx_event_log_type_time ON TABLE event_log
  COLUMNS event_type, occurred_at;
DEFINE INDEX idx_event_log_aggregate ON TABLE event_log
  COLUMNS aggregate_type, aggregate_id, occurred_at;
DEFINE INDEX idx_event_log_correlation ON TABLE event_log
  COLUMNS correlation_id;
```

### `schema_registry`

```surql
DEFINE TABLE schema_registry SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD event_type     ON TABLE schema_registry TYPE string;
DEFINE FIELD event_version  ON TABLE schema_registry TYPE int;
DEFINE FIELD schema_json    ON TABLE schema_registry TYPE object;
DEFINE FIELD deprecated_at  ON TABLE schema_registry TYPE option<datetime>;
DEFINE FIELD migration_path ON TABLE schema_registry TYPE option<string>;
DEFINE FIELD registered_at  ON TABLE schema_registry TYPE datetime;

DEFINE INDEX uniq_schema_registry_pk ON TABLE schema_registry
  COLUMNS event_type, event_version UNIQUE;
```

### `system_user`

```surql
DEFINE TABLE system_user SCHEMAFUL
  PERMISSIONS NONE;

DEFINE FIELD display_name ON TABLE system_user TYPE string;
DEFINE FIELD active_status ON TABLE system_user TYPE bool DEFAULT true;
DEFINE FIELD created_at   ON TABLE system_user TYPE datetime;
```

Seed:

```surql
INSERT INTO system_user (id, display_name, active_status, created_at)
  VALUES (
    system_user:⟨00000000-0000-7000-8000-000000000001⟩,
    "SYSTEM",
    true,
    time::now()
  )
  ON DUPLICATE KEY IGNORE;
```

SurrealDB's record-id syntax is `table:⟨value⟩`; the engine's
UUIDv7 canonical id is the value in angle brackets.

## A domain aggregate example: `academic_students`

```surql
DEFINE TABLE academic_students SCHEMAFUL
  PERMISSIONS
    FOR SELECT WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR CREATE WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR UPDATE WHERE school_id = $auth.school_id OR $auth.bypass = true
    FOR DELETE WHERE school_id = $auth.school_id OR $auth.bypass = true;

DEFINE FIELD id              ON TABLE academic_students TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD school_id       ON TABLE academic_students TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD admission_number ON TABLE academic_students TYPE option<string>;
DEFINE FIELD roll_number     ON TABLE academic_students TYPE option<string>;
DEFINE FIELD first_name      ON TABLE academic_students TYPE string;
DEFINE FIELD last_name       ON TABLE academic_students TYPE option<string>;
DEFINE FIELD full_name       ON TABLE academic_students TYPE option<string>;
DEFINE FIELD date_of_birth   ON TABLE academic_students TYPE option<string>;
DEFINE FIELD email           ON TABLE academic_students TYPE option<string>;
DEFINE FIELD mobile          ON TABLE academic_students TYPE option<string>;
DEFINE FIELD admission_date  ON TABLE academic_students TYPE option<string>;
DEFINE FIELD photo_storage_key ON TABLE academic_students TYPE option<string>;
DEFINE FIELD gender_id       ON TABLE academic_students TYPE option<record<academic_genders>>;
DEFINE FIELD blood_group_id  ON TABLE academic_students TYPE option<record<academic_blood_groups>>;
DEFINE FIELD religion_id     ON TABLE academic_students TYPE option<record<academic_religions>>;
DEFINE FIELD class_id        ON TABLE academic_students TYPE option<record<academic_classes>>;
DEFINE FIELD section_id      ON TABLE academic_students TYPE option<record<academic_sections>>;
DEFINE FIELD academic_id     ON TABLE academic_students TYPE option<record<academic_academics>>;
DEFINE FIELD category_id     ON TABLE academic_students TYPE option<record<academic_categories>>;
DEFINE FIELD group_id        ON TABLE academic_students TYPE option<record<academic_groups>>;
DEFINE FIELD route_id        ON TABLE academic_students TYPE option<record<transport_routes>>;
DEFINE FIELD vehicle_id      ON TABLE academic_students TYPE option<record<transport_vehicles>>;
DEFINE FIELD dormitory_id    ON TABLE academic_students TYPE option<record<facilities_dormitories>>;
DEFINE FIELD room_id         ON TABLE academic_students TYPE option<record<facilities_rooms>>;
DEFINE FIELD guardian_id     ON TABLE academic_students TYPE option<record<communication_guardians>>;
DEFINE FIELD user_id         ON TABLE academic_students TYPE option<record<platform_users>>;
DEFINE FIELD role_id         ON TABLE academic_students TYPE option<record<rbac_roles>>;
DEFINE FIELD version         ON TABLE academic_students TYPE int
  ASSERT $value >= 1 DEFAULT 1;
DEFINE FIELD etag            ON TABLE academic_students TYPE string
  ASSERT string::length($value) = 32;
DEFINE FIELD last_event_id   ON TABLE academic_students TYPE option<string>;
DEFINE FIELD correlation_id  ON TABLE academic_students TYPE option<string>;
DEFINE FIELD source          ON TABLE academic_students TYPE option<string>;
DEFINE FIELD active_status   ON TABLE academic_students TYPE bool DEFAULT true;
DEFINE FIELD created_at      ON TABLE academic_students TYPE datetime;
DEFINE FIELD updated_at      ON TABLE academic_students TYPE datetime;
DEFINE FIELD created_by      ON TABLE academic_students TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD updated_by      ON TABLE academic_students TYPE string
  ASSERT string::length($value) = 36;
DEFINE FIELD id_v7_legacy    ON TABLE academic_students TYPE option<int>;
DEFINE FIELD custom_fields   ON TABLE academic_students TYPE option<object>;

DEFINE INDEX idx_academic_students_school_active ON TABLE academic_students
  COLUMNS school_id, active_status;
DEFINE INDEX idx_academic_students_last_event ON TABLE academic_students
  COLUMNS last_event_id;
DEFINE INDEX idx_academic_students_correlation ON TABLE academic_students
  COLUMNS correlation_id;
DEFINE INDEX idx_academic_students_school_admission ON TABLE academic_students
  COLUMNS school_id, admission_number;
```

Note: SurrealDB has no separate `FOREIGN KEY` clause; the
`record<academic_classes>` type on `class_id` is the FK. The
adapter does not emit `ALTER TABLE` to add FKs.

## Parity surface — what works the same, what is SurrealDB-only

### Parity (works identically to the other adapters)

- Pure CRUD: `CREATE`, `SELECT`, `UPDATE`, `DELETE` map directly.
  The engine's `execute_query()` method emits the equivalent
  SurrealQL statement per query.
- Idempotency: `idempotency` table is identical in shape and
  semantics. `INSERT ... ON DUPLICATE KEY IGNORE` becomes
  `UPSERT` (or a `CREATE` + `SELECT` + conditional `UPDATE`
  within `BEGIN TRANSACTION`).
- Outbox: same pattern (write to outbox + aggregate in the same
  transaction). The adapter emits the outbox DDL block at
  `create_schema()` time.
- Audit log: identical shape, `PERMISSIONS NONE` semantics.
- Multi-tenancy: `school_id` column model + `PERMISSIONS` clause
  + adapter-injected `WHERE school_id = ?` filter. Same three
  layers of defense as Postgres.
- Engine invariants: all 7 invariant columns are emitted the
  same way.

### SurrealDB-only (not in MySQL/Postgres/SQLite)

- **`LIVE SELECT`** for change-data-capture: a consumer can
  subscribe to a table's changes via
  `LIVE SELECT * FROM academic_students WHERE school_id = $auth.school_id`
  and receive a stream of `CREATE`, `UPDATE`, `DELETE` events.
  This is SurrealDB's answer to logical replication; the engine
  does not use it for the outbox (the outbox is the engine's
  contract), but consumers can use it for sync watch_changes.
- **`DEFINE EVENT`** for in-DB triggers: the consumer can opt in
  to in-DB outbox fan-out, cross-domain event fan-out, and
  cascade-delete automation. See § DEFINE EVENT.
- **Graph traversal with `->`**: `SELECT ->has_event->outbox FROM
  academic_students:$id` traverses the graph of records. The
  engine does not use this; the engine's query layer is
  relational-style joins. But a consumer building a feature like
  "show me the student and all their attendance events" can use
  it for a single round-trip.
- **Vector search with `<|N,COSINE|>`**: SurrealDB has native
  vector similarity search. The engine does not ship a vector
  type, but a consumer adding a "find similar students" feature
  can use it on a `DEFINE FIELD embedding ON TABLE
  academic_students TYPE array<float, 384>` column.
- **`DEFINE FIELD` schema-level types and constraints**: the
  `ASSERT` clause is more expressive than `CHECK` (it can
  reference `$value`, `$before`, `$after`, `$auth`, etc.). The
  engine's adapter uses `ASSERT` for enum-like and length
  constraints; a consumer can extend with cross-column
  assertions.
- **`record<TABLE>` type**: SurrealDB's first-class referential
  type. Eliminates the need for separate `FOREIGN KEY` clauses
  and `ALTER TABLE` statements.
- **Embedded mode (`memory`, `rocksdb`)**: runs in the same
  process as the consumer. No network, no separate server. The
  engine's adapter targets `rocksdb` for production single-
  process and `memory` for tests.

## Embedded-mode constraints

The SurrealDB adapter targets the **embedded mode** (`memory` and
`rocksdb`). Embedded mode is **single-process**:

- There is **no network**; the consumer's process opens the
  database file (or in-memory instance) directly.
- There is **no replication**; for high-availability, the
  consumer must use the **distributed mode** (`tikv`).
- There is **no multi-tenant server**; one process, one database.
- The consumer's Rust binary is the only writer.

For **distributed deployment**, the consumer should switch to
**SurrealDS** (the managed SurrealDB product) or run a TiKV
cluster with SurrealDB on top. **This is out of scope for v1 of
the engine's SurrealDB adapter**; the adapter ships embedded
mode only. A future release may add a network-mode adapter
(`educore-storage-surrealdb-server`) that speaks the SurrealDB
HTTP / WebSocket protocol to a remote server.

The adapter is **not** a substitute for the relational adapters
in a multi-instance deployment. For a SaaS backend with multiple
servers, the consumer should use PostgreSQL.

## Adapter implementation notes

- The `SurrealStorage` adapter uses the official
  `surrealdb::Surreal` Rust crate. `surrealdb` 3.x is the
  required version.
- The adapter connects to an embedded `RocksDb` or `Mem`
  backend:

  ```rust
  use surrealdb::Surreal;
  use surrealdb::engine::local::{Db, Mem, RocksDb};

  // Production (single-process desktop / mobile)
  let db = Surreal::new::<RocksDb>("./data/educore.db").await?;

  // Tests
  let db = Surreal::new::<Mem>(()).await?;
  ```

- The adapter issues `BEGIN TRANSACTION` / `COMMIT TRANSACTION` /
  `CANCEL TRANSACTION` for the engine's `Transaction`
  abstraction. Single-statement operations are atomic by default;
  multi-statement outbox writes require an explicit transaction
  block.
- The adapter sets `$auth.school_id` and `$auth.bypass` on every
  new session via `db.query("LET $auth = { school_id: $sid, bypass: $b }").bind(...)`.
- The adapter's DDL emission is unit-tested against an in-memory
  SurrealDB instance. The DDL is verified before any test queries
  run.

## Encryption at rest

SurrealDB supports database-level encryption via the
`encryption-key` connection parameter. The engine does not bundle
this; the consumer's deployment may use it for sensitive data:

```rust
let db = Surreal::new::<RocksDb>(("./data/educore.db", "encryption-key"))
    .await?;
```

The engine's adapter exposes an `encryption_key` parameter on the
connection builder. The consumer is responsible for the key
management (typically via the OS keychain or a hardware security
module).

## References

- SurrealDB 3.x Documentation: `DEFINE TABLE`, `DEFINE FIELD`,
  `DEFINE INDEX`, `DEFINE EVENT`, `LIVE SELECT`, record-link
  traversal, `<|N,COSINE|>` vector search, `BEGIN TRANSACTION`.
- The `educore-storage-surrealdb` crate README (when shipped).
- `docs/ports/storage.md` § 4: `Configuration` — the engine's
  `SurrealStorage::builder()` pattern.
- `docs/schemas/database-schema.md` § 11: the canonical minimum
  schema.
- `docs/ports/storage.md#future-storage-backends-deferred` — the
  rationale for deferring the SurrealDB and MongoDB adapters.
