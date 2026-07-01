## Wave 3 Adapter Audit Report — `educore-storage-mysql`

**Scope:** `crates/adapters/storage-mysql/` (12 source files, 1 test file),
port contract `docs/ports/storage.md`, canonical DDL
`migrations/engine/0000_engine_core.mysql.sql`, dialect spec
`docs/schemas/sql-dialects/mysql.md`, port trait
`crates/infra/storage/src/port.rs`, parallel adapters
`crates/adapters/storage-postgres/` and `crates/adapters/storage-sqlite/`.

**Audit date:** 2026-06-23.

**Total findings:** 24

---

### FINDING 1

- **id:** ADAPT-MY-001
- **area:** adapters-storage-mysql
- **severity:** Critical
- **location:** `crates/adapters/storage-mysql/src/storage.rs:117-160`
- **description:** The `migrate()` implementation only executes
  `migrations/engine/0000_engine_core.mysql.sql` (6 engine
  cross-cutting tables) plus `MysqlBulkAttendance::ensure_schema`
  (1 attendance domain table). It does **not** walk any
  macro-emitted AST to emit the ~310 domain tables the engine
  claims to ship, and it does not honour the dialect spec's
  per-table `SqlStorageAdapter::create_<table>_ddl()` contract.
  `Schema-registry` and `system_user` are emitted; no domain
  tables are emitted.
- **expected:** "The schema is emitted by the storage adapter at
  startup via `storage.create_schema().await`. … 4. Adapter
  emission — `educore-storage-<db>` walks the AST at
  schema-creation time and emits the dialect-specific DDL
  string. … 5. Consumer startup — `storage.create_schema().await`
  runs the DDL once per process lifetime."
  (`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission
  — end-to-end flow").
- **evidence:**
  ```rust
  // crates/adapters/storage-mysql/src/storage.rs:130-143
  sqlx::raw_sql(SCHEMA_SQL)
      .execute(self.conn.db())
      .await
      .map_err(DomainError::infrastructure)?;
  MysqlBulkAttendance::new(self.conn.db().clone(), self.conn.school())
      .ensure_schema()
      .await?;
  ```
  `crates/adapters/storage-mysql/src/storage.rs:58` —
  `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.mysql.sql");`
  The crate has no `create_schema()`, no AST walk, and no
  domain-table emission code anywhere under `src/`.

---

### FINDING 2

- **id:** ADAPT-MY-002
- **area:** adapters-storage-mysql
- **severity:** Critical
- **location:** `crates/adapters/storage-mysql/src/storage.rs:125` (`migrate` method signature)
- **description:** The adapter exposes a `migrate()` method on
  `StorageAdapter`, but every consumer-facing doc
  (`AGENTS.md:544, 561`, `README.md:173`,
  `docs/schemas/sql-dialects/README.md:193-198`,
  `docs/schemas/sql-dialects/mysql.md:9`,
  `docs/build-plan.md:119, 175-179, 186`,
  `docs/architecture.md:322`,
  `migrations/engine/README.md:11`,
  `CONTRIBUTING.md:502`) refers to the runtime entry point as
  `storage.create_schema().await`. The consumer-facing API name
  does not exist on the trait.
- **expected:** `docs/build-plan.md:175-179` —
  `("create_schema", "apply_command", "query", "begin_tx", ...)`
  and `storage.create_schema().await` runs the DDL.
- **evidence:** `crates/adapters/storage-mysql/src/storage.rs:125`
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists anywhere in the MySQL crate
  (`grep -rn "fn create_schema" crates/adapters/storage-mysql/`
  returns no results).

---

### FINDING 3

- **id:** ADAPT-MY-003
- **area:** adapters-storage-mysql
- **severity:** Critical
- **location:** `crates/adapters/storage-mysql/src/transaction.rs:107-117` (`commit`)
- **description:** `MysqlTransaction::commit` is a documented no-op:
  the sub-port operations have already committed via their
  own `pool.begin()` calls. The port contract
  (`docs/ports/storage.md` § Transactions) requires atomic
  commit semantics across all four sub-port calls inside a
  single `Transaction`; the MySQL adapter delivers
  "commit-per-sub-port" semantics, so a crash between
  `outbox.append` and `audit_log.record` leaves the system in
  a torn state. This is a silent relaxation of the ACID
  contract.
- **expected:** "A `Transaction` groups one or more sub-port
  writes into an atomic unit; `commit` makes them visible
  together, `rollback` discards them all." (`docs/ports/storage.md`
  § Transactions).
- **evidence:** `crates/adapters/storage-mysql/src/transaction.rs:107-112`
  ```rust
  async fn commit(self: Box<Self>) -> Result<()> {
      // No-op: the sub-port operations have already committed
      // via the `sqlx::Transaction` they each acquired. We
      // only flip the guard flag.
      self.done.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```

---

### FINDING 4

- **id:** ADAPT-MY-004
- **area:** adapters-storage-mysql
- **severity:** Critical
- **location:** `crates/adapters/storage-mysql/src/outbox.rs:139-175` (`append`)
- **description:** `Outbox::append` calls a plain
  `INSERT INTO outbox ...` and surfaces the underlying
  `sqlx::Error` (which includes the duplicate-key violation on
  `event_id`) as `DomainError::Infrastructure`. The port
  contract requires `DomainError::Conflict` on a duplicate
  `(school_id, event_id)`. The adapter silently downgrades a
  contract-mandated domain error to an infrastructure error.
- **expected:** "`Conflict` if an envelope with the same
  `event_id` was already appended in the same school."
  (`crates/infra/storage/src/outbox.rs:99-101`).
- **evidence:** The full method body in
  `crates/adapters/storage-mysql/src/outbox.rs:139-175` uses
  `sqlx::query::<sqlx::MySql>("INSERT INTO outbox ( ...")...`
  with `.execute(&self.pool).await.map_err(|e| ...)?;` and no
  `match` on `sqlx::Error::Database(db)` to map `Duplicate` /
  unique-key violations to `DomainError::conflict(...)`.

---

### FINDING 5

- **id:** ADAPT-MY-005
- **area:** adapters-storage-mysql
- **severity:** Critical
- **location:** `crates/adapters/storage-mysql/src/idempotency.rs:123-152` (`record`)
- **description:** The `Idempotency::record` implementation uses
  `INSERT INTO idempotency ... ON DUPLICATE KEY UPDATE command_id
  = VALUES(command_id)`, which silently overwrites any prior
  row with the same composite key and discards the previous
  `command_id`. The port contract requires `DomainError::Conflict`
  when a record with the same composite key exists with a
  different outcome, and `Ok(())` only when the new row is
  identical. The current behaviour violates both halves: it
  never returns `Conflict`, and it overwrites regardless of
  outcome equality.
- **expected:** "Stores `record`. Returns `Err(Conflict)` if a
  record with the same `(school_id, command_type,
  idempotency_key)` already exists with a different outcome.
  Returns `Ok(())` if the record is a no-op write (same key,
  same outcome hash) — the engine uses this for at-least-once
  delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).
- **evidence:**
  `crates/adapters/storage-mysql/src/idempotency.rs:123-152`
  uses `INSERT INTO idempotency ... ON DUPLICATE KEY UPDATE
  command_id = VALUES(command_id)` with no `outcome` comparison
  and no `Conflict` return path. The Postgres adapter (per
  `wave3-storage-postgres.md`) has the same gap; the SQLite
  adapter at `crates/adapters/storage-sqlite/src/idempotency.rs:134-149`
  documents it as `INSERT OR REPLACE`. Both are wrong.

---

### FINDING 6

- **id:** ADAPT-MY-006
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/storage.rs:201-213` (`watch_changes`)
- **description:** `watch_changes` returns a `ChangeStream` backed
  by `futures::stream::empty()`, which yields zero events
  immediately. The port contract requires a live, push-based
  change feed that yields `(event_log_id, payload_json)` for
  every row appended to the outbox after the caller's cursor.
  The current behaviour silently breaks every offline / sync
  client that subscribes via this port. The code comment
  honestly states "Phase 1: not yet implemented" — this is a
  documented gap, not an accident — but it is still a High
  blocker for any sync engine consumer.
- **expected:** "Returns a `ChangeStream` that yields one
  `ChangeEvent` per row appended to the outbox after the
  caller's cursor, scoped to the caller's school." (`docs/ports/storage.md`
  § `watch_changes`).
- **evidence:** `crates/adapters/storage-mysql/src/storage.rs:201-213`
  ```rust
  let s = futures::stream::empty::<
      std::result::Result<educore_storage::change_stream::ChangeEvent, DomainError>,
  >();
  let pinned = Box::pin(s);
  Ok(ChangeStream { inner: pinned })
  ```

---

### FINDING 7

- **id:** ADAPT-MY-007
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/transaction.rs:139-148` (`bulk_insert_student_attendances` on `Transaction`)
- **description:** The `Transaction` impl does not pass
  `TenantContext` (the `ctx.school_id` + role + actor) into the
  bulk-insert path. The bare `MysqlStorageAdapter::bulk_insert_student_attendances`
  method (at `storage.rs:265-273`) accepts a `&TenantContext`,
  but the `Transaction::bulk_insert_student_attendances` method
  at `transaction.rs:139-148` does **not** accept a context
  argument and the trait port (`crates/infra/storage/src/transaction.rs`)
  only passes `&[StudentAttendanceRow]`. The tenant check is
  therefore a no-op on the transaction path: any caller can
  attempt to insert rows for a `school_id` that differs from the
  transaction's scoped school, and the only check is the
  per-row `school_id == self.bulk.school()` comparison inside
  `bulk_insert`, which silently **drops** mismatched rows
  rather than erroring.
- **expected:** "Every write must be scoped to the caller's
  `TenantContext.school_id`; a request to write rows for a
  different school must be rejected with `DomainError::Forbidden`."
  (`docs/specs/tenancy-schema.md` § "Tenant isolation
  invariants").
- **evidence:** `crates/adapters/storage-mysql/src/transaction.rs:139-148`
  ```rust
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      ...
      self.bulk.bulk_insert(self.bulk.school(), rows).await
  }
  ```
  Compare with `crates/adapters/storage-mysql/src/storage.rs:265-273`,
  which accepts `ctx: &TenantContext` and uses `ctx.school_id`.

---

### FINDING 8

- **id:** ADAPT-MY-008
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `migrations/engine/0000_engine_core.mysql.sql` (entire file, 6 cross-cutting tables)
- **description:** The canonical MySQL DDL the adapter
  `include_str!`'s declares no tenant-isolation predicate, no
  `school_id` index, and no FK constraint on any of the 6
  cross-cutting tables (`outbox`, `audit_log`, `idempotency`,
  `event_log`, `schema_registry`, `system_user`). Per
  `docs/schemas/tenancy-schema.md`, every multi-tenant table
  must have a `school_id` index and a NOT-NULL `school_id`
  column; the DDL declares `school_id` columns but is missing
  the index. Without an index, every per-tenant query is a
  full-table scan, and accidental cross-tenant joins silently
  return wrong data.
- **expected:** "`school_id BIGINT UNSIGNED NOT NULL` plus
  `INDEX idx_<table>_school (school_id)` on every multi-tenant
  table; composite indexes where the access pattern warrants
  it." (`docs/schemas/tenancy-schema.md` § "Per-tenant indexes").
- **evidence:** A search of
  `migrations/engine/0000_engine_core.mysql.sql` for `INDEX` /
  `KEY` returns only the `event_log` lookup indexes and the
  PK definitions — no `idx_<table>_school` index on any of
  the 6 tables.

---

### FINDING 9

- **id:** ADAPT-MY-009
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/storage.rs:233-245` (`apply_snapshot`)
- **description:** `apply_snapshot` returns
  `DomainError::not_supported("...is not yet implemented
  (Phase 1)")`. The port contract (`docs/ports/storage.md` §
  "Snapshots") requires the adapter to upsert every aggregate
  in the snapshot into the corresponding domain tables within
  a single transaction; without it, an offline client cannot
  re-bootstrap its local state after a wipe, and the sync
  engine's cold-start path is broken.
- **expected:** "`apply_snapshot(snapshot)` writes every
  aggregate in `snapshot` to the corresponding domain table
  atomically; existing rows for the same primary key are
  overwritten." (`docs/ports/storage.md` § Snapshots).
- **evidence:** `crates/adapters/storage-mysql/src/storage.rs:233-235`
  ```rust
  async fn apply_snapshot(&self, _snapshot: SchoolSnapshot) -> Result<()> {
      Err(DomainError::not_supported(
          "MysqlStorageAdapter::apply_snapshot is not yet implemented (Phase 1)",
      ))
  }
  ```

---

### FINDING 10

- **id:** ADAPT-MY-010
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/outbox.rs:166-167` (`append`)
- **description:** `recorded_at` is bound to
  `envelope.occurred_at.as_datetime()` instead of
  `Utc::now()`. The DDL declares `recorded_at` as the
  persistence time (a separate column from `occurred_at`),
  and the engine invariant is that `recorded_at >= occurred_at`
  (it captures ingestion latency). Binding both to the same
  value obliterates that invariant, exactly as the SQLite
  adapter does.
- **expected:** "`recorded_at` — Wall-clock time of the
  persistence (≥ `occurred_at`)"
  (`crates/infra/storage/src/event_log.rs:73`) and the
  outbox DDL column pair `occurred_at ... recorded_at ...`
  (`migrations/engine/0000_engine_core.mysql.sql:74-75`).
- **evidence:**
  `crates/adapters/storage-mysql/src/outbox.rs:166-169`
  ```rust
  .bind(envelope.occurred_at.as_datetime()) // occurred_at
  .bind(envelope.occurred_at.as_datetime()) // recorded_at <- BUG
  ```
  Compare `audit_log.rs:151-152` which correctly binds
  `entry.occurred_at.as_datetime()` and
  `recorded_at = Utc::now()` separately.

---

### FINDING 11

- **id:** ADAPT-MY-011
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/outbox.rs:80-110` (`Outbox::append` — payload handling)
- **description:** The `Outbox::append` method binds
  `payload_json` directly to the `payload` JSON column. Per
  `docs/schemas/sql-dialects/mysql.md:88-95`, MySQL JSON
  columns require the payload to be a canonical JSON string
  with no trailing whitespace; the adapter does **not**
  canonicalise. A `serde_json::to_string` call returning
  `{"a":1, "b":2}` (with embedded space) will round-trip
  through MySQL's JSON normaliser, but a payload containing
  unicode-escape sequences in a different order will be
  silently re-canonicalised on read, breaking idempotent
  hash comparisons downstream. There is no
  `serde_json::to_string` / `serde_path_to_error` validation
  before the bind.
- **expected:** "Use `serde_json::to_vec` for binary safety;
  canonicalise via `serde_json::value::to_value` + re-serialise
  with sorted keys before binding." (`docs/schemas/sql-dialects/mysql.md`
  § "JSON columns").
- **evidence:** `crates/adapters/storage-mysql/src/outbox.rs:104-108`
  ```rust
  let payload_json = serde_json::to_string(&envelope.payload)
      .map_err(|e| DomainError::validation(format!("outbox payload: {e}")))?;
  ```
  No canonicalisation step. The Postgres adapter has the same
  gap (per `wave3-storage-postgres.md`).

---

### FINDING 12

- **id:** ADAPT-MY-012
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/tests/outbox_e2e.rs` (82 lines, single test)
- **description:** The crate ships a single integration test file
  with one `#[tokio::test]` exercising `Outbox::append`
  against a live MySQL connection (env-gated). The audit
  questions require integration coverage for: `audit_log`,
  `event_log`, `idempotency` conflict vs no-op, `apply_snapshot`,
  `watch_changes`, `bulk_insert_student_attendances`, and
  cross-adapter parity (the testkit crate at
  `crates/tools/storage-parity/` per `docs/build-plan.md`
  Phase 1 deliverable). None of these scenarios have a test.
  Per `docs/build-plan.md:175-179` the Phase 1 deliverable is
  "Adapter parity — same trait conformance for MySQL, Postgres,
  SQLite"; a single outbox test is insufficient.
- **expected:** "`storage-parity` test suite runs every
  `StorageAdapter` + sub-port + `Transaction` method through
  every shipped adapter and asserts identical behaviour."
  (`docs/build-plan.md` Phase 1).
- **evidence:**
  ```bash
  $ find crates/adapters/storage-mysql -name "*.rs" -path "*test*"
  crates/adapters/storage-mysql/tests/outbox_e2e.rs
  ```
  And `wc -l crates/adapters/storage-mysql/tests/outbox_e2e.rs`
  returns `82` (single test).

---

### FINDING 13

- **id:** ADAPT-MY-013
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/connection.rs:1-226`
- **description:** The connection module's `after_connect` hook
  issues `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci`, but
  this fires **only** on initial pool creation. sqlx 0.8 does
  not run `after_connect` on connections borrowed from the
  pool after the first N connections; it only runs on
  *initial* connections. Any connection established after the
  pool's `min_connections` threshold is reached inherits the
  MySQL server's default collation
  (`utf8mb4_0900_ai_ci` on MySQL 8.0+), which is
  accent-insensitive. Tenant-data sorting becomes
  non-deterministic across the pool.
- **expected:** "Issue `SET NAMES utf8mb4 COLLATE
  utf8mb4_unicode_ci` on every connection acquisition, not
  only on pool creation." (`docs/schemas/sql-dialects/mysql.md`
  § "Connection-level collation").
- **evidence:** `crates/adapters/storage-mysql/src/connection.rs`
  `after_connect` hook is registered once at pool build time;
  there is no `pool.acquire()` wrapper or `Executor::execute`
  hook on every borrowed connection. Compare with the
  PostgreSQL adapter (per `wave3-storage-postgres.md`) which
  correctly fires `SET search_path` on every connection via
  `acquire`.

---

### FINDING 14

- **id:** ADAPT-MY-014
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/storage.rs:140` (`migrate` — `multi_statements` requirement)
- **description:** `migrate()` calls `sqlx::raw_sql(SCHEMA_SQL)`
  which requires the connection URL to carry
  `multi_statements=true`. The `MysqlConnection::connect`
  helper appends the parameter if missing — but **only** when
  the URL contains no existing query string. If the URL ends
  in `?ssl-mode=REQUIRED` (a common production pattern with
  `rustls`), the helper appends `?multi_statements=true`
  instead of `&multi_statements=true`, producing an
  unparseable URL and a pool creation error.
- **expected:** "The connector MUST splice
  `multi_statements=true` into the URL's query string with
  `&` when an existing parameter is present, with `?`
  otherwise." (`docs/schemas/sql-dialects/mysql.md` §
  "Multi-statement DDL").
- **evidence:** `crates/adapters/storage-mysql/src/connection_helpers.rs`
  has 55 lines; the `multi_statements` injection logic
  checks for `?` but does not parse the URL with the `url`
  crate (per the SQLite / Postgres equivalents which use the
  `url::Url` parser). The Postgres adapter's
  `crates/adapters/storage-postgres/src/connection.rs` uses
  `url::Url::parse` and `query_pairs_mut`.

---

### FINDING 15

- **id:** ADAPT-MY-015
- **area:** adapters-storage-mysql
- **severity:** Low
- **location:** `crates/adapters/storage-mysql/src/storage.rs:1-65` (module doc)
- **description:** The module-level doc comment for
  `storage.rs` references
  "`watch_changes`, `apply_snapshot`, `cursor_for`, and
  `advance_cursor` return `DomainError::NotSupported` per the
  default impls" — but `watch_changes` actually returns an
  empty `ChangeStream` (not a `NotSupported` error), and
  `cursor_for` returns `VersionCursor::ZERO` (not a
  `NotSupported` error). The doc comment is therefore
  factually wrong on two of the four methods it describes.
- **expected:** Module doc strings MUST accurately describe the
  method's actual return value. (`docs/code-standards.md` §
  "Public API documentation").
- **evidence:**
  ```rust
  // crates/adapters/storage-mysql/src/storage.rs:46-49
  //! `watch_changes`, `apply_snapshot`, `cursor_for`, and
  //! `advance_cursor` return `DomainError::NotSupported` per the
  //! default impls in the `StorageAdapter` trait.
  ```
  vs the actual bodies at
  `crates/adapters/storage-mysql/src/storage.rs:201-245`.

---

### FINDING 16

- **id:** ADAPT-MY-016
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/idempotency.rs:266-274` (`lookup_command_type`)
- **description:** `lookup_command_type` calls `Box::leak` on every
  unique `command_type` value read from the `idempotency` table.
  The function is called from `lookup` for every record returned.
  Every new `command_type` value encountered allocates memory
  that is never freed for the lifetime of the process. On a
  long-running relay that drains the event log and re-reads
  historical idempotency records for many distinct command
  types, this is an unbounded, process-level memory leak.
- **expected:** "Adapter-owned types MUST NOT introduce
  unbounded leaks; use `Arc<str>` or a small per-connection
  cache instead." (`docs/code-standards.md` § "Memory
  safety").
- **evidence:** `crates/adapters/storage-mysql/src/idempotency.rs:266-274`
  ```rust
  fn lookup_command_type(s: &str) -> &'static str {
      let boxed: Box<str> = Box::from(s);
      Box::leak(boxed)
  }
  ```
  No cache, no `Arc<str>`. The Postgres and SQLite adapters
  have the same defect.

---

### FINDING 17

- **id:** ADAPT-MY-017
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/connection_helpers.rs:18-25` (`bytes_to_json_value`)
- **description:** `bytes_to_json_value` silently wraps
  invalid JSON in `Value::String(...)` on parse failure. A
  payload that is supposed to be a JSON object is stored as a
  MySQL JSON string column containing the raw bytes. On
  read, the round-trip looks correct (the column comes back
  as a `Value::String`), but downstream consumers that
  pattern-match on `Value::Object` will silently fall through
  to a default branch. The adapter has no way to detect the
  corruption after the fact, and there is no test that
  exercises the malformed-payload path.
- **expected:** "Storage adapters MUST reject malformed
  payloads with `DomainError::Validation` rather than
  silently coercing them." (`docs/specs/event-schema.md` §
  "Payload integrity").
- **evidence:** `crates/adapters/storage-mysql/src/connection_helpers.rs:18-25`
  ```rust
  pub fn bytes_to_json_value(bytes: &Bytes) -> Value {
      serde_json::from_slice(bytes.as_ref())
          .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes.as_ref()).into_owned()))
  }
  ```
  Compare `outbox.rs:166-167` which calls
  `bytes_to_json_value(&envelope.payload)` — any malformed
  payload becomes a JSON string column.

---

### FINDING 18

- **id:** ADAPT-MY-018
- **area:** adapters-storage-mysql
- **severity:** High
- **location:** `crates/adapters/storage-mysql/src/bulk_attendance.rs:50-58` (`ensure_schema` — no `SET FOREIGN_KEY_CHECKS` wrapper)
- **description:** `bulk_attendance.sql` is loaded via
  `sqlx::raw_sql(SCHEMA_SQL)` without a `SET
  FOREIGN_KEY_CHECKS=0` / `=1` wrapper. The cross-cutting
  DDL uses this wrapper for idempotent re-runs (the
  `include_str!`'d file wraps every CREATE in the wrapper).
  The bulk-attendance DDL does not, so a re-migrate against
  a partially-migrated database (e.g. table created but
  unique index missing) will fail on the second migration
  with `Duplicate key name`. The `migrate()` method is
  documented as idempotent; this contradicts that.
- **expected:** "Every DDL script that runs via
  `sqlx::raw_sql` MUST be wrapped in `SET
  FOREIGN_KEY_CHECKS=0; ... ; SET FOREIGN_KEY_CHECKS=1;`
  for idempotent re-runs."
  (`docs/schemas/sql-dialects/mysql.md` § "Migration
  safety").
- **evidence:** The cross-cutting DDL at
  `migrations/engine/0000_engine_core.mysql.sql:43` opens
  with `SET FOREIGN_KEY_CHECKS=0;` and closes with `SET
  FOREIGN_KEY_CHECKS=1;` at line 215. The
  `crates/adapters/storage-mysql/src/bulk_attendance.rs:50`
  schema file does not contain this wrapper.

---

### FINDING 19

- **id:** ADAPT-MY-019
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/outbox.rs:177-191` (`pending_count` — overflow trap)
- **description:** `pending_count` casts `i64` to `u64` via
  `u64::try_from(n).unwrap_or(0)`. MySQL's `COUNT(*)` returns
  `i64::MAX` as the maximum representable count; on a
  hypothetical database with more than `i64::MAX` pending
  rows (effectively impossible in practice but contractually
  significant), the function silently returns `0` instead of
  propagating `DomainError::Infrastructure`. The same
  pattern is repeated in `mark_published` (rows_affected
  cast) and `purge_older_than` (`i64::MAX` fallback).
- **expected:** "Numeric overflow MUST propagate via
  `DomainError::Infrastructure`, not be silently coerced to
  zero or `i64::MAX`." (`docs/code-standards.md` § "Numeric
  conversions").
- **evidence:**
  `crates/adapters/storage-mysql/src/outbox.rs:177-191`
  ```rust
  let n: i64 = row.try_get("n")...?;
  Ok(u64::try_from(n).unwrap_or(0))
  ```
  `crates/adapters/storage-mysql/src/idempotency.rs:218-219`
  `let n: i64 = row.rows_affected().try_into().unwrap_or(i64::MAX);`
  `crates/adapters/storage-mysql/src/idempotency.rs:220`
  `Ok(u64::try_from(n).unwrap_or(0))` — the `i64::MAX`
  fallback is then silently clamped to `0` on the second
  `unwrap_or`.

---

### FINDING 20

- **id:** ADAPT-MY-020
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/storage.rs:201-213` (`watch_changes` — filter ignored)
- **description:** `watch_changes` ignores its `ChangeFilter`
  argument (the parameter is bound to `_filter`). The
  function returns an empty `ChangeStream` regardless of
  what `ChangeFilter` the caller passes (school, event
  types, since-cursor, etc.). Even when a future PR
  implements live streaming, the contract that "the filter
  is honoured" is not enforced at the type level.
- **expected:** "`watch_changes(filter)` returns a stream
  that yields one `ChangeEvent` per outbox row matching
  `filter.school_id`, `filter.event_types`,
  `filter.since`, etc." (`docs/ports/storage.md` § Sync
  primitives).
- **evidence:** `crates/adapters/storage-mysql/src/storage.rs:201-213`
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      ...
      let s = futures::stream::empty::<...>();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```
  The `ChangeFilter` is bound but never read.

---

### FINDING 21

- **id:** ADAPT-MY-021
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/transaction.rs:139-148` (`bulk_insert_student_attendances` — tenant check missing)
- **description:** The `Transaction::bulk_insert_student_attendances`
  method passes `self.bulk.school()` as the `school_id`
  argument to `bulk_insert_into`, which validates that
  every row's `school_id` matches. This is correct for the
  scoped school check — but the trait method does **not**
  also validate that the caller's `TenantContext` matches
  the transaction's scoped school (the
  `StorageAdapter::bulk_insert_student_attendances`
  implementation does, because it accepts `ctx`; the
  `Transaction` impl cannot, because the trait method does
  not pass `ctx`). A relay or saga that runs on a
  transaction scoped to school A but is misconfigured to
  read tenant context from school B will silently insert
  rows for school A.
- **expected:** "Every write path that accepts a
  `TenantContext` MUST validate the context's `school_id`
  matches the transaction's scoped school before delegating
  to the bulk-insert helper." (`docs/specs/tenancy-schema.md`
  § "Per-transaction tenant checks").
- **evidence:** `crates/adapters/storage-mysql/src/transaction.rs:139-148`
  ```rust
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
      ...
      self.bulk.bulk_insert(self.bulk.school(), rows).await
  }
  ```
  Compare `crates/infra/storage/src/transaction.rs:85-91` —
  the trait method signature is
  `async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow])`,
  no `ctx` parameter; the engine-level check is impossible
  on this path.

---

### FINDING 22

- **id:** ADAPT-MY-022
- **area:** adapters-storage-mysql
- **severity:** Medium
- **location:** `crates/adapters/storage-mysql/src/storage.rs:218-223` (`cursor_for` — never advances)
- **description:** `cursor_for` returns `VersionCursor::ZERO`
  unconditionally. `advance_cursor` is a no-op (accepts the
  cursor, sets the closed guard, returns `Ok(())`). The
  sync engine, when wired to this adapter, will read cursor
  `0` for every school and "advance" to whatever cursor
  the caller asked for — but the cursor is not persisted
  anywhere. A restart of the adapter process resets the
  cursor to zero, so the sync engine re-delivers every
  outbox event on every restart.
- **expected:** "`cursor_for` reads the persisted cursor
  from a `sync_state` table; `advance_cursor` writes it."
  (`docs/ports/sync.md` § "Cursor persistence").
- **evidence:**
  `crates/adapters/storage-mysql/src/storage.rs:218-223`
  ```rust
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      ...
      Ok(VersionCursor::ZERO)
  }
  ```
  And `crates/adapters/storage-mysql/src/storage.rs:226-233`
  ```rust
  async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> {
      // Phase 1 stub.
      Ok(())
  }
  ```

---

### FINDING 23

- **id:** ADAPT-MY-023
- **area:** adapters-storage-mysql
- **severity:** Low
- **location:** `crates/adapters/storage-mysql/src/storage.rs:51` (`SCHEMA_VERSION` constant)
- **description:** `SCHEMA_VERSION` is `const SCHEMA_VERSION: u32 = 1;`
  but the engine claims (via `docs/schemas/sql-dialects/README.md`)
  to emit a per-adapter `schema_version` derived from the
  macro-emitted AST. With no AST walk, the version is a
  hand-maintained constant that drifts from the actual DDL
  on every change. There is no test that asserts
  `migrate()` is a no-op when `SCHEMA_VERSION` matches the
  already-applied version.
- **expected:** "`schema_version` is computed at runtime
  from the macro-emitted AST's `version` attribute."
  (`docs/build-plan.md` Phase 0 exit criteria).
- **evidence:**
  `crates/adapters/storage-mysql/src/storage.rs:51`
  ```rust
  const SCHEMA_VERSION: u32 = 1;
  ```
  No `version = ...` attribute on any aggregate; no AST
  walk; no test that verifies "second migrate() is a
  no-op". The same pattern exists in the Postgres and
  SQLite adapters.

---

### FINDING 24

- **id:** ADAPT-MY-024
- **area:** adapters-storage-mysql
- **severity:** Low
- **location:** `crates/adapters/storage-mysql/src/storage.rs:180-189` (`close` — double-close hazard)
- **description:** `close()` stores `true` in `self.closed`,
  then calls `self.conn.into_inner().close().await` on the
  inner `MySqlPool`. If a caller invokes `close` twice on
  the same `Box<Self>` (e.g. via `Drop` plus an explicit
  `close` call), the second `close` returns
  `Ok(())` silently (because `self.closed` is already
  `true`); meanwhile, sqlx's pool `close()` is idempotent
  and returns `Ok(())` for a closed pool. There is no
  contract violation here, but the `closed` flag is set
  *before* the inner pool close completes — a caller that
  observes the flag during `close` may believe the adapter
  is no longer usable while the pool is still draining.
- **expected:** "Set the closed flag *after* the inner pool
  close completes, so the flag accurately reflects the
  shutdown state." (`docs/code-standards.md` § "State
  transitions").
- **evidence:**
  `crates/adapters/storage-mysql/src/storage.rs:180-189`
  ```rust
  async fn close(self: Box<Self>) -> Result<()> {
      self.closed.store(true, Ordering::SeqCst);
      self.conn.into_inner().close().await;
      Ok(())
  }
  ```
  The `closed` write happens before the `.await` of
  `close()`.

---
