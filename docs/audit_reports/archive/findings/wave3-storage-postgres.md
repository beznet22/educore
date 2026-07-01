# Audit findings: educore-storage-postgres (Phase 1)

**Scope:** `crates/adapters/storage-postgres/`, port contract at
`docs/ports/storage.md`, canonical DDL at
`migrations/engine/0000_engine_core.postgres.sql`, phase handoff at
`docs/handoff/PHASE-1-HANDOFF.md`, dialect spec at
`docs/schemas/sql-dialects/postgresql.md`, port trait at
`crates/infra/storage/src/port.rs`, `transaction.rs`.

**Total findings:** 47

---

### FINDING 1

- **id:** ADAPTER-PG-001
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/storage.rs:130
- **description:** The adapter exposes a `migrate()` method on
  `StorageAdapter`, but every consumer-facing doc
  (`AGENTS.md:544, 561`, `README.md:173`,
  `docs/schemas/sql-dialects/README.md:193-198`,
  `docs/schemas/sql-dialects/postgresql.md:9`,
  `docs/build-plan.md:119, 175-179, 186`,
  `docs/architecture.md:322`,
  `migrations/engine/README.md:11`,
  `CONTRIBUTING.md:502`) refers to the runtime entry point as
  `storage.create_schema().await`. The consumer-facing API name does
  not exist on the trait.
- **expected:** `docs/build-plan.md:175-179` —
  `("create_schema", "apply_command", "query", "begin_tx", ...)`
  and `storage.create_schema().await` runs the DDL.
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:130` —
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists in the entire crate
  (`grep -rn "fn create_schema" crates/adapters/storage-postgres/`
  returns no results).

---

### FINDING 2

- **id:** ADAPTER-PG-002
- **area:** adapters
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql (entire 240-line file)
- **description:** The canonical PG DDL the adapter
  `include_str!`'s contains no row-level security policies and no
  `ENABLE ROW LEVEL SECURITY` / `FORCE ROW LEVEL SECURITY` clauses
  on any of the 6 cross-cutting tables. Per
  `docs/schemas/sql-dialects/postgresql.md:122-159` PG is required
  to use `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY` and the
  adapter must issue `SET LOCAL app.current_school_id = ?` on every
  transaction.
- **expected:** `docs/schemas/sql-dialects/postgresql.md:122-159`:
  ```sql
  ALTER TABLE "<aggregate>" ENABLE ROW LEVEL SECURITY;
  ALTER TABLE "<aggregate>" FORCE ROW LEVEL SECURITY;
  CREATE POLICY "school_isolation_<aggregate>" ON "<aggregate>"
    USING ("school_id" = current_setting('app.current_school_id')::UUID)
    WITH CHECK ("school_id" = current_setting('app.current_school_id')::UUID);
  ```
- **evidence:** `migrations/engine/0000_engine_core.postgres.sql:1-240` —
  contains only `CREATE SCHEMA`, `CREATE TABLE IF NOT EXISTS`, and
  `CREATE INDEX IF NOT EXISTS` statements. No `ALTER TABLE ... ENABLE
  ROW LEVEL SECURITY`, no `CREATE POLICY`, no RLS clause anywhere in
  the 240 lines.

---

### FINDING 3

- **id:** ADAPTER-PG-003
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/connection.rs:69-90
- **description:** The connection's `after_connect` hook issues
  `SET search_path = engine, public` but does NOT issue
  `SET app.current_school_id = '<uuid>'`. Per
  `docs/schemas/sql-dialects/postgresql.md:142-145`, the engine's
  adapter must issue `SET LOCAL app.current_school_id = ?` on every
  new transaction so RLS policies can resolve the tenant. Without
  this, even if RLS were enabled, every query would see zero rows.
- **expected:** `docs/schemas/sql-dialects/postgresql.md:142` —
  `SET LOCAL app.current_school_id = '<uuid>';`
- **evidence:** `crates/adapters/storage-postgres/src/connection.rs:69-87` —
  ```rust
  .after_connect(|conn, _meta| {
      Box::pin(async move {
          sqlx::query("SET search_path = engine, public")
              .execute(conn)
              .await?;
          Ok(())
      })
  })
  ```
  No `SET LOCAL app.current_school_id` is issued.

---

### FINDING 4

- **id:** ADAPTER-PG-004
- **area:** adapters
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql:73, 121-122, 153, 183, 208
- **description:** The canonical PG DDL declares `JSONB NOT NULL` /
  `JSONB NULL` on the JSONB columns of the 6 cross-cutting tables
  with NO `CHECK (jsonb_typeof(...) = 'object')` constraints. The
  dialect spec at `postgresql.md:58, 249, 286-288, 314, 339, 359`
  mandates the JSONB CHECK constraint on every JSONB column to
  guarantee the payload is a JSON object (not a JSON array, scalar,
  or null where forbidden).
- **expected:** `docs/schemas/sql-dialects/postgresql.md:58` —
  `JSONB NOT NULL CHECK (jsonb_typeof("payload") = 'object')`
  and line 249 — `"payload" JSONB NOT NULL CHECK (jsonb_typeof("payload") = 'object')`.
- **evidence:** `migrations/engine/0000_engine_core.postgres.sql:73` —
  `payload         JSONB        NOT NULL,` (no CHECK constraint).
  Same omission on `before_snapshot`, `after_snapshot`, `metadata`
  (lines 120-122), `outcome` (line 153), `payload` on event_log
  (line 183), `schema_json` (line 208).

---

### FINDING 5

- **id:** ADAPTER-PG-005
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/storage.rs:130-169
- **description:** The `migrate()` implementation does not walk any
  macro-emitted AST or render any domain table DDL. Per
  `docs/build-plan.md:177-179` and `docs/schemas/sql-dialects/README.md:182-187`,
  the adapter must "walk the macro-emitted AST to render the ~310
  domain tables at create_schema() time". The PG adapter only emits
  the 6 cross-cutting tables plus the `attendance_student_attendances`
  table — zero of the ~310 domain tables are emitted.
- **expected:** `docs/build-plan.md:179` —
  `Walks the macro-emitted AST to render the ~310 domain tables
  at create_schema() time`.
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:141-144` —
  ```rust
  sqlx::raw_sql(SCHEMA_SQL)
      .execute(self.conn.db())
      .await
      .map_err(DomainError::infrastructure)?;
  ```
  Followed by `PostgresBulkAttendance::new(...).ensure_schema().await?;`
  (line 149-151). No AST walk, no domain table emission, no reference
  to any macro-emitted `EntityDescriptor`.

---

### FINDING 6

- **id:** ADAPTER-PG-006
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/storage.rs:118-127
- **description:** The adapter implements `begin()` which returns a
  `Box<dyn Transaction>`. The `Transaction` trait exposes
  `commit`/`rollback`, but they are NO-OPs in this adapter (see
  `transaction.rs:122-129, 131-137`) — the engine's at-least-once
  outbox dedup is the safety net. Per
  `docs/ports/storage.md:120-136` and
  `crates/infra/storage/src/transaction.rs:32-91` the contract is
  for an actual transactional commit. The PHASE-1-HANDOFF.md:38-46
  acknowledges this is a flag-based stub.
- **expected:** `docs/ports/storage.md:124-127` —
  `async fn commit(self: Box<Self>) -> Result<()>` and "On commit
  the writes are persisted and the outbox events are released to
  the event bus."
- **evidence:** `crates/adapters/storage-postgres/src/transaction.rs:122-129` —
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

### FINDING 7

- **id:** ADAPTER-PG-007
- **area:** adapters
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql (entire 240-line file)
- **description:** The PG DDL uses schema-qualified unquoted
  identifiers (`engine.outbox`, `engine.audit_log`, etc.). The
  dialect spec at `postgresql.md:11-23, 96-114` mandates
  **double-quoted lowercase identifiers** (`"outbox"`,
  `"event_id"`, etc.) and reserves schema-prefixing as a consumer
  choice, not the canonical engine form.
- **expected:** `docs/schemas/sql-dialects/postgresql.md:11-23` —
  "Use **double quotes** for every identifier"
  and `"CREATE TABLE \"outbox\" ("`.
- **evidence:** `migrations/engine/0000_engine_core.postgres.sql:61` —
  `CREATE TABLE IF NOT EXISTS engine.outbox (` (unquoted,
  schema-qualified) vs spec line 237 — `CREATE TABLE IF NOT EXISTS
  "outbox" (` (double-quoted, no schema).

---

### FINDING 8

- **id:** ADAPTER-PG-008
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/tests/outbox_e2e.rs:1-81
- **description:** The entire `tests/` directory contains a single
  e2e test file with one test function (`outbox_append_and_pending_round_trip`).
  Per `docs/ports/storage.md:469-477` the port requires unit tests of
  every repository method, integration tests against a real
  database, a parity test, a tenancy test, a failure-injection
  test, and a load test (10k attendance marks in <5s). None of the
  AuditLog, Idempotency, EventLog, or BulkAttendance sub-ports have
  any test, and the single e2e test is gated behind an env var that
  is unset in CI (`EDUCORE_PG_URL`).
- **expected:** `docs/ports/storage.md:470-477` —
  - Unit tests of every repository method
  - Integration tests against a real database (testcontainers)
  - A parity test verifying identical behavior across adapters
  - A tenancy test verifying cross-tenant reads are blocked
  - A failure-injection test (e.g. deadlock retry, connection drop)
  - A load test (10k attendance marks in <5s)
- **evidence:** `crates/adapters/storage-postgres/tests/` —
  `total 12` (one file, 3078 bytes). `tests/outbox_e2e.rs:1-81`
  contains one `#[tokio::test]` function. `PHASE-1-HANDOFF.md:19-22`
  acknowledges `124 passing` for the entire workspace but
  `+4 from the MySQL connection::tests URL helper unit tests` —
  i.e. the Phase 1 e2e count is 3 total across all SQL adapters
  (1 per adapter).

---

### FINDING 9

- **id:** ADAPTER-PG-009
- **area:** adapters
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql:117
- **description:** The DDL column `ip` on `audit_log` is declared
  `VARCHAR(45)`. The dialect spec at `postgresql.md:283` mandates
  `INET` (PostgreSQL's native IPv4/IPv6 type with validation).
- **expected:** `docs/schemas/sql-dialects/postgresql.md:283` —
  `"ip" INET,`
- **evidence:** `migrations/engine/0000_engine_core.postgres.sql:117` —
  `ip              VARCHAR(45)     NULL,`

---

### FINDING 10

- **id:** ADAPTER-PG-010
- **area:** adapters
- **severity:** Critical
- **location:** migrations/engine/0000_engine_core.postgres.sql:229
- **description:** The `system_user.active_status` column is declared
  `SMALLINT NOT NULL DEFAULT 1`. The dialect spec at
  `postgresql.md:373` mandates `BOOLEAN NOT NULL DEFAULT TRUE`.
- **expected:** `docs/schemas/sql-dialects/postgresql.md:373` —
  `"active_status" BOOLEAN     NOT NULL DEFAULT TRUE,`
- **evidence:** `migrations/engine/0000_engine_core.postgres.sql:229` —
  `active_status SMALLINT     NOT NULL DEFAULT 1,`

---

### FINDING 11

- **id:** ADAPTER-PG-011
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:238-244
- **description:** `lookup_command_type` calls `Box::leak(boxed)` on
  every read, leaking the `command_type` string into static memory.
  Per `AGENTS.md` § "Engine Rules" and `docs/code-standards.md`
  code standards, `Box::leak` in production paths is forbidden.
  Per-read memory growth is unbounded.
- **expected:** `crates/infra/storage/src/idempotency.rs:31` —
  `pub command_type: &'static str,` (the field is `&'static str`,
  but the value comes from a `VARCHAR` column read).
  The port struct should use `String` (or `Cow<'static, str>`).
  `PHASE-1-HANDOFF.md:176-180` acknowledges this as "Open question
  #3" but the leak is shipped.
- **evidence:** `crates/adapters/storage-postgres/src/idempotency.rs:238-244` —
  ```rust
  fn lookup_command_type(s: &str) -> &'static str {
      // Allocate a `Box<str>` and leak it. The leak is bounded
      // by the cardinality of the engine's command catalogue
      // (a few hundred at most) and the lifetime of the process;
      // a periodic sweep can be added if it becomes a concern.
      let boxed: Box<str> = Box::from(s);
      Box::leak(boxed)
  }
  ```

---

### FINDING 12

- **id:** ADAPTER-PG-012
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:181
- **description:** `expires_at` is computed as `recorded_at +
  Duration::hours(24)` with `.unwrap_or(recorded_at)` fallback.
  On a chrono overflow (e.g. far-future `recorded_at`), the
  fallback silently makes the record's `expires_at == recorded_at`,
  causing the row to be eligible for immediate purge on the next
  sweep — a silent data-loss path.
- **expected:** `crates/infra/storage/src/idempotency.rs:107-113` —
  `purge_older_than` returns `u64` rows affected; the adapter
  should never silently shorten a retention window to zero.
- **evidence:** `crates/adapters/storage-postgres/src/idempotency.rs:179-181` —
  ```rust
  let expires_at = recorded_at
      .checked_add_signed(Duration::hours(DEFAULT_RETENTION_HOURS))
      .unwrap_or(recorded_at);
  ```

---

### FINDING 13

- **id:** ADAPTER-PG-013
- **area:** adapters
- **severity:** Critical
- **location:** crates/adapters/storage-postgres/src/outbox.rs:175-191
- **description:** `Outbox::pending_count` accepts an arbitrary
  `school_id: SchoolId` argument and filters by it, ignoring the
  handle's scoped `self.school`. A caller can request the pending
  count for any tenant — bypassing the adapter's own scoping.
  The same pattern is broken in `event_log::read`/`count` and
  `audit_log::read_for_target` (they accept an explicit
  `school_id` arg but the handle's `school` field is
  `#[allow(dead_code)]` and unused).
- **expected:** `docs/schemas/tenancy-schema.md` and `docs/ports/storage.md:140-150` —
  "The storage adapter is responsible for enforcing tenant
  isolation. The engine always passes a SchoolId filter; the
  adapter MUST add a school_id = $1 predicate to every read query."
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:175-191` —
  ```rust
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // ... override with a direct COUNT(*)
      let row = sqlx::query(
          "SELECT COUNT(*) AS n FROM outbox WHERE school_id = $1 AND published_at IS NULL",
      )
      .bind(school_id.as_uuid())
      .fetch_one(&self.pool)
  ```
  And `crates/adapters/storage-postgres/src/audit_log.rs:117-118` —
  ```rust
  #[allow(dead_code)]
  school: SchoolId,
  ```

---

### FINDING 14

- **id:** ADAPTER-PG-014
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/storage.rs:82-113, docs/ports/storage.md:418-429
- **description:** The port contract at `docs/ports/storage.md:418-429`
  specifies a `PostgresStorage::builder().url(...).max_connections(20)
  .min_connections(2).acquire_timeout(...).statement_cache_capacity(128)
  .build()` pattern. The adapter only exposes
  `PostgresStorageAdapter::connect(url, school)` with no way to
  configure pool size, acquire timeout, statement cache, or
  statement-cache capacity.
- **expected:** `docs/ports/storage.md:418-429` — full builder
  pattern with `.max_connections(20)`, `.min_connections(2)`,
  `.acquire_timeout(...)`, `.statement_cache_capacity(128)`.
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:82-113` —
  exposes only `PostgresStorageAdapter::new(conn)` and
  `PostgresStorageAdapter::connect(url, school)`. No builder, no
  pool-config methods.
  `grep -rn "max_connections\|min_connections\|acquire_timeout\|statement_cache_capacity"
  crates/adapters/storage-postgres/src/` returns no results.

---

### FINDING 15

- **id:** ADAPTER-PG-015
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/storage.rs:163-168
- **description:** `MigrationReport.already_at_version` is
  hard-coded to `false`. Per the port contract at
  `crates/infra/storage/src/change_stream.rs:243-255`, the field
  indicates "Whether the migration was a no-op (already at
  version)". The adapter cannot distinguish a no-op re-run from a
  fresh migration.
- **expected:** `crates/infra/storage/src/change_stream.rs:253-254` —
  `/// Whether the migration was a no-op (already at version).`
  `pub already_at_version: bool,`
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:163-168` —
  ```rust
  Ok(MigrationReport {
      version: SCHEMA_VERSION,
      statements_executed,
      duration,
      already_at_version: false,
  })
  ```

---

### FINDING 16

- **id:** ADAPTER-PG-016
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/outbox.rs:61, 123, 190; event_log.rs:67, 197, 260; idempotency.rs:100, 219-220
- **description:** Eight locations use `.unwrap_or(0)` on numeric
  conversions (`u32::try_from(...).unwrap_or(0)`,
  `u64::try_from(...).unwrap_or(0)`). A negative `event_version`
  on a row (DQL corruption, manual DB tampering, or a legacy row)
  silently becomes `0`. A negative `rows_affected()` silently
  becomes `i64::MAX` then `0`. The clippy deny for `cast_possible_wrap`
  / `cast_sign_loss` is being dodged by silently substituting zero.
- **expected:** `docs/code-standards.md` and `AGENTS.md` §
  "Type Safety" — "No `as` casts that truncate or lose data.
  Use `TryFrom`/`TryInto` with proper error handling."
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:61` —
  ```rust
  schema_version: u32::try_from(self.event_version).unwrap_or(0),
  ```
  And `crates/adapters/storage-postgres/src/idempotency.rs:219-220` —
  ```rust
  let n: i64 = row.rows_affected().try_into().unwrap_or(i64::MAX);
  Ok(u64::try_from(n).unwrap_or(0))
  ```

---

### FINDING 17

- **id:** ADAPTER-PG-017
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/audit_log.rs:155-184
- **description:** `audit_log.append` always writes
  `source = 'api'` (hard-coded literal in the SQL string). The
  AuditLogEntry struct has no `source` field. Background jobs,
  the outbox relay, migrations, and other legitimate producers
  cannot distinguish their audit rows.
- **expected:** `migrations/engine/0000_engine_core.postgres.sql:124` —
  `source VARCHAR(16) NOT NULL,` (the DDL column is mandatory,
  so it must carry meaningful producer information).
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:163-165` —
  ```rust
  ) VALUES (
      $1, $2, $3, $4, $5, $6, $7, $8, NULL, $9, $10, \
      $11, NULL, NULL, NULL, $12, $13, $14, FALSE, 'api'\
  )
  ```
  No binding for source; always literal `'api'`.

---

### FINDING 18

- **id:** ADAPTER-PG-018
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/audit_log.rs:154, 177
- **description:** `audit_log.append` sets `recorded_at =
  occurred_at` (a literal copy of the command's wall-clock time).
  The DDL declares a separate `recorded_at TIMESTAMPTZ` column
  (line 116) intended to track ingestion latency — the difference
  between when the command occurred and when the row was persisted.
  The adapter discards the latency signal.
- **expected:** `docs/schemas/audit-schema.md` § 13 (referenced
  in DDL line 94) — `recorded_at` is the time the row was written
  by the audit sink.
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:154, 177` —
  ```rust
  let recorded_at: DateTime<Utc> = entry.occurred_at.as_datetime();
  ...
  .bind(recorded_at)
  ```

---

### FINDING 19

- **id:** ADAPTER-PG-019
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/audit_log.rs:78
- **description:** `AuditLogRow::metadata` is declared
  `Json<Value>` (NOT NULL). The DDL declares
  `metadata JSONB NULL` (line 122). The adapter's read shape and
  the DDL disagree on nullability, but the row's metadata is
  always `Value::Null` (the port default), so this is silently
  mapped to JSON null rather than SQL NULL.
- **expected:** `migrations/engine/0000_engine_core.postgres.sql:122` —
  `metadata        JSONB            NULL,`
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:78` —
  ```rust
  metadata: Json<Value>,
  ```
  vs `migrations/engine/0000_engine_core.postgres.sql:122` —
  `metadata        JSONB            NULL,`

---

### FINDING 20

- **id:** ADAPTER-PG-020
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:51-61, audit_log.rs:53-83, event_log.rs:35-49
- **description:** `IdempotencyRow`, `AuditLogRow`, and
  `EventLogRow` declare DDL columns that the read query never
  actually consumes (`#[allow(dead_code)]` on `audit_id`,
  `actor_type`, `command_id`, `ip`, `user_agent`, `session_id`,
  `recorded_at`, `cross_tenant`, `source`, `command_id`,
  `expires_at`). These fields are queried from the database (the
  SELECT includes them) but discarded on read. This wastes I/O
  and signals that the adapter has drifted from the port struct's
  shape — `PHASE-1-HANDOFF.md:168-175` acknowledges this in
  Open question #2.
- **expected:** `crates/infra/storage/src/audit.rs:62-101` —
  `AuditLogEntry` should be the superset of DDL columns the
  port cares about.
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:53-83` —
  ```rust
  struct AuditLogRow {
      #[allow(dead_code)]
      audit_id: Uuid,
      ...
      #[allow(dead_code)]
      ip: Option<String>,
      #[allow(dead_code)]
      user_agent: Option<String>,
      #[allow(dead_code)]
      session_id: Option<Uuid>,
      ...
      #[allow(dead_code)]
      cross_tenant: bool,
      #[allow(dead_code)]
      source: String,
  }
  ```
  Eight `#[allow(dead_code)]` annotations on a single 30-line
  struct.

---

### FINDING 21

- **id:** ADAPTER-PG-021
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/bulk_attendance.sql:14-39
- **description:** The bulk-attendance DDL stores UUIDs as `BYTEA`,
  dates as `TEXT`, and counters as `INTEGER`. The dialect spec at
  `postgresql.md:39-68` mandates `UUID` for UUIDv7 ids, `DATE`
  for calendar dates, and `INTEGER` is acceptable for counters,
  but the spec also says "engine emits `UUID NOT NULL`" and
  prefers native types over wire-decoupled `BYTEA`/`TEXT`. The
  port comment at `student_attendance_row.rs:108-114` explicitly
  admits this is "decoupled from the canonical engine form."
- **expected:** `docs/schemas/sql-dialects/postgresql.md:46, 56` —
  `"id" UUID NOT NULL`, `"date_of_birth" DATE`.
- **evidence:** `crates/adapters/storage-postgres/src/bulk_attendance.sql:14-39` —
  ```sql
  CREATE TABLE IF NOT EXISTS attendance_student_attendances (
      school_id            BYTEA      NOT NULL,
      id                   BYTEA      NOT NULL,
      student_id           BYTEA      NOT NULL,
      ...
      attendance_date      TEXT       NOT NULL,
      ...
      is_absent            INTEGER    NOT NULL DEFAULT 0,
      ...
      active_status        INTEGER    NOT NULL DEFAULT 1,
      ...
  )
  ```

---

### FINDING 22

- **id:** ADAPTER-PG-022
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/error.rs:1-50
- **description:** The PG adapter does not define a
  `StorageError` enum. The port contract at
  `docs/ports/storage.md:216-235` defines a 10-variant
  `StorageError` enum (`Connection`, `Conflict`, `Deadlock`,
  `UniqueViolation`, `ForeignKey`, `Check`, `NotFound`,
  `Infrastructure`, `Timeout`, `SerializationFailure`) and
  states "The engine maps `StorageError::Infrastructure` to
  `DomainError::Infrastructure`". The PG adapter only provides a
  `StringError` wrapper plus a free function `map_infrastructure`,
  with no typed translation for conflict / deadlock / unique
  violation / foreign key / check / not found / timeout /
  serialization failure.
- **expected:** `docs/ports/storage.md:217-230` —
  ```rust
  pub enum StorageError {
      #[error("connection failed: {0}")] Connection(String),
      #[error("transaction conflict: {0}")] Conflict(String),
      #[error("deadlock detected")] Deadlock,
      #[error("unique violation: {0}")] UniqueViolation { constraint: String },
      ...
  }
  ```
- **evidence:** `crates/adapters/storage-postgres/src/error.rs:1-50` —
  defines only `pub struct StringError(pub String);` and
  `pub fn map_infrastructure<E>(e: E) -> ...`. No `StorageError`
  enum. `grep -rn "StorageError" crates/adapters/storage-postgres/`
  returns no results.

---

### FINDING 23

- **id:** ADAPTER-PG-023
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/audit_log.rs:147-151
- **description:** `actor_type` is computed as a hard-coded
  `if entry.actor_id == SYSTEM_USER_ID { "system" } else { "user" }`.
  There is no provision for background-job actors, scheduled-job
  actors, sync-relay actors, or migration actors — all are
  misclassified as `"user"`. The DDL column is
  `VARCHAR(16) NOT NULL` with no CHECK constraint to catch
  drift.
- **expected:** `docs/schemas/sql-dialects/postgresql.md:160-168` —
  `"role_type" VARCHAR(16) NOT NULL CHECK ("role_type" IN
  ('system', 'custom'))` — the engine mandates CHECK constraints
  on enum-like columns.
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:147-151` —
  ```rust
  let actor_type: &'static str = if entry.actor_id == SYSTEM_USER_ID {
      "system"
  } else {
      "user"
  };
  ```

---

### FINDING 24

- **id:** ADAPTER-PG-024
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/event_log.rs:114-164, 213-240
- **description:** `build_select` constructs dynamic SQL by string
  concatenation (`sql.push_str(" AND event_type = ANY($"); ...`).
  While the only interpolated values are operator symbols, column
  names, and `$N` placeholders (no user input is interpolated),
  the comment at lines 121-124 and 137-141 explicitly justifies
  bypassing `format!` for clippy cleanliness rather than for
  safety. The pattern is fragile: any future change that adds a
  user-input interpolation would be invisible to code review.
- **expected:** `docs/ports/storage.md:296-313` and
  `docs/schemas/sql-dialects/README.md:103-200` — typed AST
  translation via `sqlx::QueryBuilder`.
- **evidence:** `crates/adapters/storage-postgres/src/event_log.rs:135-144` —
  ```rust
  sql.push_str(" AND event_type = ANY($");
  // append the next index (params.len() before this push + 1)
  let idx = params.len();
  // We need to write the index without `format!` to keep
  // the build clippy-clean. Push the digit chars one by
  // one.
  let idx_str = idx.to_string();
  sql.push_str(&idx_str);
  sql.push(')');
  ```

---

### FINDING 25

- **id:** ADAPTER-PG-025
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:124-193
- **description:** The bulk-attendance `bulk_insert_into` returns
  `DomainError::conflict` immediately on a unique-key violation
  (line 185-189) without any retry / backoff. Per
  `crates/infra/storage/src/transaction.rs:38-42` the engine
  expects "the engine retries the command automatically" on
  conflicts. No retry policy exists.
- **expected:** `crates/infra/storage/src/transaction.rs:39-41` —
  "Conflict on a unique-key violation, deadlock, or serialisation
  failure (the engine retries the command automatically)."
- **evidence:** `crates/adapters/storage-postgres/src/bulk_attendance.rs:182-192` —
  ```rust
  match qb.build().execute(pool).await {
      Ok(_) => Ok(()),
      Err(sqlx::Error::Database(db))
          if db.kind() == sqlx::error::ErrorKind::UniqueViolation =>
      {
          Err(DomainError::conflict(
              "bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row",
          ))
      }
      Err(other) => Err(DomainError::infrastructure(other)),
  }
  ```

---

### FINDING 26

- **id:** ADAPTER-PG-026
- **area:** adapters
- **severity:** High
- **location:** crates/adapters/storage-postgres/src/transaction.rs:122-137
- **description:** `PostgresTransaction::commit` and
  `PostgresTransaction::rollback` are no-ops. The
  `outbox().append(...)`, `audit_log().append(...)`,
  `event_log().append(...)`, and `idempotency().record(...)` calls
  each open their own short-lived `pool.begin()` inside the
  sub-port method and auto-commit on drop. Between any two of
  these calls, a duplicate dispatch can land in another
  transaction. PHASE-1-HANDOFF.md:38-46 acknowledges this as
  "Open question #1" — the engine's at-least-once dedup is the
  only safety net.
- **expected:** `docs/ports/storage.md:131-137` —
  "Reads see writes from the same transaction. On commit the
  writes are persisted and the outbox events are released to the
  event bus. On rollback the writes are discarded."
- **evidence:** `crates/adapters/storage-postgres/src/transaction.rs:122-129` —
  ```rust
  async fn commit(self: Box<Self>) -> Result<()> {
      // No-op: the sub-port operations have already committed
      // via the `sqlx::Transaction` they each acquired.
      self.done.store(true, Ordering::SeqCst);
      Ok(())
  }
  ```

---

### FINDING 27

- **id:** ADAPTER-PG-027
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/storage.rs:186-194
- **description:** `PostgresStorageAdapter::close` calls
  `self.conn.into_inner().close().await` (consuming the
  `PostgresConnection` to get the inner `PgPool`). The
  `sqlx::Pool::close` future returns `()`, but the `await` is
  performed without inspecting any error. The outer
  `Result<()>` is always `Ok(())` — close cannot fail per this
  signature, but the API surface suggests it can.
- **expected:** `crates/infra/storage/src/port.rs:50-53` —
  "Closes the adapter, releasing all underlying connections.
  After `close`, any further call returns `Err(Infrastructure)`."
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:186-194` —
  ```rust
  async fn close(self: Box<Self>) -> Result<()> {
      self.closed.store(true, Ordering::SeqCst);
      self.conn.into_inner().close().await;
      Ok(())
  }
  ```

---

### FINDING 28

- **id:** ADAPTER-PG-028
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/storage.rs:119, 131, 173, 206, 227, 238, 254
- **description:** Every public method on `PostgresStorageAdapter`
  checks `if self.closed.load(...) { return Err(DomainError::conflict(...)) }`
  and returns `DomainError::conflict` (a domain-level conflict
  variant). The port contract at `crates/infra/storage/src/port.rs:52-53`
  states "After `close`, any further call returns
  `Err(Infrastructure)`". Returning `Conflict` on a closed adapter
  is a wrong error variant.
- **expected:** `crates/infra/storage/src/port.rs:53` —
  "After `close`, any further call returns `Err(Infrastructure)`."
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:119-123` —
  ```rust
  if self.closed.load(Ordering::SeqCst) {
      return Err(DomainError::conflict(
          "StorageAdapter::begin called on a closed adapter",
      ));
  }
  ```

---

### FINDING 29

- **id:** ADAPTER-PG-029
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/outbox.rs:175-191
- **description:** `Outbox::pending_count` is a method that exists
  on the trait with a default impl. The adapter overrides it with
  a direct `COUNT(*)` (good), but ignores the `self.school` field
  and accepts any `school_id: SchoolId` argument. Combined with
  finding ADAPTER-PG-013 this means the trait API allows any
  tenant to query any other tenant's pending outbox count.
- **expected:** Port should validate `school_id` against the
  handle's scope; doc-vs-code drift.
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:175-191` —
  ```rust
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // ... the default impl in the trait materialises every
      // pending row just to count them, which is O(n) memory
      // for a 1-line aggregate. Override with a direct
      // `COUNT(*)` for back-pressure sizing.
      let row = sqlx::query(
          "SELECT COUNT(*) AS n FROM outbox WHERE school_id = $1 AND published_at IS NULL",
      )
      .bind(school_id.as_uuid())
      ...
  ```

---

### FINDING 30

- **id:** ADAPTER-PG-030
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/audit_log.rs:144
- **description:** `audit_log.append` instruments with
  `fields(actor = %entry.actor_id, target_type = %entry.target_type)`.
  The `actor_id` (a `UserId`) and `target_type` (a free-form
  aggregate name) are exposed in tracing span fields. Per
  `AGENTS.md` and `docs/code-standards.md` PII (and tenant-scoped
  identifiers) should be filtered from tracing output.
- **expected:** `docs/code-standards.md` § "PII Logging" (if it
  exists) — tracing spans should redact UserId, CorrelationId,
  and aggregate identifiers.
- **evidence:** `crates/adapters/storage-postgres/src/audit_log.rs:144` —
  ```rust
  #[instrument(skip(self, entry), fields(actor = %entry.actor_id, target_type = %entry.target_type))]
  ```
  And `crates/adapters/storage-postgres/src/bulk_attendance.rs:101` —
  ```rust
  #[instrument(skip(self, rows), fields(n = rows.len(), school = %self.school))]
  ```
  `school = %self.school` exposes the tenant identifier in spans.

---

### FINDING 31

- **id:** ADAPTER-PG-031
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:174-206
- **description:** `Idempotency::record` uses
  `INSERT ... ON CONFLICT (school_id, command_type, idempotency_key)
  DO NOTHING`. The default port impl comment at
  `crates/infra/storage/src/idempotency.rs:94-100` says:
  "Returns `Err(Conflict)` if a record with the same
  `(school_id, command_type, idempotency_key)` already exists with
  a different outcome. Returns `Ok(())` if the record is a no-op
  write (same key, same outcome hash) — the engine uses this for
  at-least-once delivery of retries." The PG adapter conflates
  the "different outcome" case with the "same outcome" case —
  both are silently swallowed as `Ok(())`.
- **expected:** `crates/infra/storage/src/idempotency.rs:94-100` —
  detect "same key, different outcome" and return
  `DomainError::Conflict`.
- **evidence:** `crates/adapters/storage-postgres/src/idempotency.rs:188-206` —
  ```rust
  sqlx::query(
      "INSERT INTO idempotency (\
          school_id, command_type, idempotency_key, \
          command_id, outcome, recorded_at, expires_at\
      ) VALUES ($1, $2, $3, $4, $5, $6, $7) \
       ON CONFLICT (school_id, command_type, idempotency_key) DO NOTHING",
  )
  ```

---

### FINDING 32

- **id:** ADAPTER-PG-032
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/outbox.rs:160-172
- **description:** `Outbox::mark_published` uses `ANY($1)` against
  a `Vec<Uuid>` for the IN-list. PG allows up to
  ~32,000 parameters in a single statement; with no per-call cap,
  a single bulk publish call could exceed the limit and produce
  a runtime error.
- **expected:** `docs/ports/storage.md:188-189` —
  "Timeouts are configurable per adapter."
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:160-172` —
  ```rust
  async fn mark_published(&self, ids: &[EventId]) -> Result<()> {
      if ids.is_empty() {
          return Ok(());
      }
      let id_uuids: Vec<Uuid> = ids.iter().map(|i| i.as_uuid()).collect();
      sqlx::query("UPDATE outbox SET published_at = NOW() WHERE event_id = ANY($1)")
          .bind(&id_uuids)
          .execute(&self.pool)
          ...
  ```

---

### FINDING 33

- **id:** ADAPTER-PG-033
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:42-44, 132-137
- **description:** `MAX_ROWS_PER_CALL = 1000` is enforced as a
  hard cap (line 132-137 returns `Validation`). The comment at
  lines 42-44 cites "PostgreSQL caps a single prepared statement
  at 65,535 placeholders; 24 columns × 1,000 rows = 24,000
  placeholders (well under the cap)." With 24 columns × 2730 rows
  the cap would be exceeded — 1000 is a conservative choice but
  no chunking / batch path exists in the adapter. The caller
  must split the input themselves.
- **expected:** `docs/ports/storage.md:477` —
  "A load test (10k attendance marks in <5s)." A 10k batch would
  require 10 adapter calls.
- **evidence:** `crates/adapters/storage-postgres/src/bulk_attendance.rs:128-137` —
  ```rust
  if rows.is_empty() {
      return Ok(());
  }
  if rows.len() > MAX_ROWS_PER_CALL {
      return Err(DomainError::validation(format!(
          "bulk_insert_student_attendances: at most {MAX_ROWS_PER_CALL} rows per call, got {}",
          rows.len()
      )));
  }
  ```

---

### FINDING 34

- **id:** ADAPTER-PG-034
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/transaction.rs:60-86, 156-167
- **description:** `PostgresTransaction::bulk_insert_student_attendances`
  uses `self.bulk.school()` as the tenant anchor (line 166),
  while `PostgresStorageAdapter::bulk_insert_student_attendances`
  uses the caller-supplied `ctx.school_id` (storage.rs:260).
  These two paths can disagree on which school is authoritative.
  Additionally, neither path opens a real `sqlx::Transaction`
  for atomic commit with the surrounding outbox / audit appends.
- **expected:** `docs/ports/storage.md:131-137` — the same
  transaction must own both the bulk insert and the outbox
  append.
- **evidence:** `crates/adapters/storage-postgres/src/transaction.rs:166` —
  ```rust
  self.bulk.bulk_insert(self.bulk.school(), rows).await
  ```
  And `crates/adapters/storage-postgres/src/storage.rs:259-261` —
  ```rust
  let handle = PostgresBulkAttendance::new(self.conn.db().clone(), self.conn.school());
  handle.bulk_insert(ctx.school_id, rows).await
  ```

---

### FINDING 35

- **id:** ADAPTER-PG-035
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/outbox.rs:140-158
- **description:** `Outbox::pending` uses `LIMIT $2` (line 149)
  bound to `i64::from(limit)` (line 152). The port method takes
  `limit: u32`. Negative limits silently become very large
  positive numbers due to `i64::from(limit)` on a `u32` — but
  since the port passes `u32`, this is safe at the API boundary.
  However, the adapter does not enforce an upper cap on the
  limit value, so a caller could request billions of rows.
- **expected:** `crates/infra/storage/src/event_log.rs:101-103` —
  "The cap is `filter.limit`; the adapter may enforce a lower cap
  for safety."
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:140-158` —
  ```rust
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
      let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(
          "SELECT ... FROM outbox WHERE school_id = $1 AND published_at IS NULL \
           ORDER BY enqueued_at ASC LIMIT $2",
      )
      .bind(self.school.as_uuid())
      .bind(i64::from(limit))
      ...
  ```

---

### FINDING 36

- **id:** ADAPTER-PG-036
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/storage.rs:197-216
- **description:** `watch_changes` is implemented (rather than
  falling back to the trait's default `NotSupported`) by
  returning an empty `futures::stream::empty` boxed into a
  `ChangeStream`. This silently swallows subscribers — a sync
  client receives no events and no error. The trait default
  returns `NotSupported` so the sync engine "fails loudly at
  startup". The override masks the error.
- **expected:** `docs/ports/storage.md:112-118` —
  "the sync engine, when it tries to subscribe on a non-sync
  adapter, fails loudly at startup — not silently at runtime".
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:196-216` —
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      // ... keep the default `NotSupported` behaviour by simply
      // constructing the `ChangeStream` shell and returning it
      // ...
      let s = futures::stream::empty::<
          std::result::Result<educore_storage::change_stream::ChangeEvent, DomainError>,
      >();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```

---

### FINDING 37

- **id:** ADAPTER-PG-037
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/storage.rs:225-235
- **description:** `cursor_for` returns `VersionCursor::ZERO`
  unconditionally (line 234). `advance_cursor` returns `Ok(())`
  unconditionally (line 245). The sync engine, when it relies on
  these to track per-school replay position, will replay every
  event from scratch on every restart.
- **expected:** `docs/ports/storage.md:108-110` —
  "The cursor is a per-school monotonically increasing `version`
  (or `event_id`). It's stored in a small engine-internal table;
  the adapter implements the read/write."
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:225-235` —
  ```rust
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      if self.closed.load(Ordering::SeqCst) {
          return Err(DomainError::conflict(
              "StorageAdapter::cursor_for called on a closed adapter",
          ));
      }
      Ok(VersionCursor::ZERO)
  }
  ```

---

### FINDING 38

- **id:** ADAPTER-PG-038
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/connection.rs:30-95
- **description:** `PostgresConnection` does not configure pool
  size, acquire timeout, idle timeout, max lifetime, or statement
  cache. The defaults (`max_connections = 10`, no acquire timeout
  in sqlx 0.8 defaults) are taken. A consumer that opens many
  concurrent commands will exhaust the pool with no graceful
  backpressure.
- **expected:** `docs/ports/storage.md:418-429` — full builder
  pattern with `.max_connections(20)`, `.acquire_timeout(...)`.
- **evidence:** `crates/adapters/storage-postgres/src/connection.rs:69-90` —
  ```rust
  let pool = PgPoolOptions::new()
      .after_connect(|conn, _meta| { ... })
      .connect(url)
      .await
      ...
  ```
  No `.max_connections(...)`, `.min_connections(...)`,
  `.acquire_timeout(...)`, or statement cache configuration.

---

### FINDING 39

- **id:** ADAPTER-PG-039
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/storage.rs:99-102
- **description:** The `connect()` constructor doesn't validate the
  URL beyond sqlx's own parser. A consumer that passes
  `postgres://localhost/nonexistent` will fail at the pool
  acquire step with no actionable error mapping. The error path
  is `DomainError::infrastructure(sqlx::Error)` which loses
  the "is the DB reachable? is the URL valid? are credentials
  correct?" diagnostic.
- **expected:** `crates/infra/storage/src/port.rs:46-48` —
  "Liveness check. Returns `Ok(())` if the adapter is connected
  and responsive; `Err(Infrastructure)` otherwise."
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:99-102` —
  ```rust
  pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
      let conn = PostgresConnection::connect(url, school).await?;
      Ok(Self::new(conn))
  }
  ```

---

### FINDING 40

- **id:** ADAPTER-PG-040
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:208-221
- **description:** `purge_older_than` issues
  `DELETE FROM idempotency WHERE school_id = $1 AND recorded_at < $2`
  but does not issue it inside a transaction. A long-running
  delete can take row-level locks for the duration. Combined with
  the per-call transaction model (each sub-port call opens its own
  short transaction), a concurrent insert may serialize against
  the purge.
- **expected:** `docs/ports/storage.md:209-226` — bulk operations
  on cross-cutting tables should run inside an explicit
  transaction with appropriate batch sizing.
- **evidence:** `crates/adapters/storage-postgres/src/idempotency.rs:208-221` —
  ```rust
  async fn purge_older_than(&self, school_id: SchoolId, cutoff: Timestamp) -> Result<u64> {
      let row = sqlx::query("DELETE FROM idempotency WHERE school_id = $1 AND recorded_at < $2")
          .bind(school_id.as_uuid())
          .bind(cutoff.as_datetime())
          .execute(&self.pool)
          .await
          .map_err(educore_core::error::DomainError::infrastructure)?;
      ...
  ```

---

### FINDING 41

- **id:** ADAPTER-PG-041
- **area:** adapters
- **severity:** Medium
- **location:** crates/adapters/storage-postgres/src/connection_helpers.rs:20-23
- **description:** `bytes_to_json_value` silently wraps
  non-JSON-serialisable bytes in `Value::String(...)` (the
  lossy-UTF8 fallback at line 22). A round-trip
  `bytes → JSONB → bytes` is no longer lossless for binary
  payloads; the JSONB CHECK constraint in the dialect spec
  (`jsonb_typeof(...) = 'object'`) would reject this at insert
  time, but the adapter never checks the constraint at insert.
- **expected:** `docs/ports/storage.md:131-137` —
  lossless round-trip is the contract.
- **evidence:** `crates/adapters/storage-postgres/src/connection_helpers.rs:18-23` —
  ```rust
  pub fn bytes_to_json_value(bytes: &Bytes) -> Value {
      serde_json::from_slice(bytes.as_ref())
          .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(bytes.as_ref()).into_owned()))
  }
  ```

---

### FINDING 42

- **id:** ADAPTER-PG-042
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/storage.rs:130-169
- **description:** `migrate()` executes the canonical DDL but
  reports a `statements_executed` count derived from
  `SCHEMA_SQL.split(';').filter(|s| !s.trim().is_empty()).count()`.
  This is a naive splitter that over-counts (a semicolon inside a
  string literal or a PL/pgSQL block would inflate the count) and
  under-counts (statements terminated by `;` followed by
  comments). The reported number is meaningless.
- **expected:** `crates/infra/storage/src/change_stream.rs:249-250` —
  "The number of statements executed (DDL or DML)."
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:156-162` —
  ```rust
  let statements_executed = u32::try_from(
      SCHEMA_SQL
          .split(';')
          .filter(|s| !s.trim().is_empty())
          .count(),
  )
  .unwrap_or(0);
  ```

---

### FINDING 43

- **id:** ADAPTER-PG-043
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/storage.rs:117
- **description:** `#[instrument(skip(self))]` on every public
  method but no structured event emission; tracing spans are
  scoped only to the method body. Cross-tenant operations
  (e.g. `cursor_for(school_id)`) log nothing about the tenant.
- **expected:** `docs/code-standards.md` (Telemetry section if
  any) — structured events for tenant-scoped operations.
- **evidence:** `crates/adapters/storage-postgres/src/storage.rs:225-235` —
  ```rust
  #[instrument(skip(self, _school_id))]
  async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
      ...
  ```
  The `_school_id` is explicitly skipped from tracing.

---

### FINDING 44

- **id:** ADAPTER-PG-044
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/idempotency.rs:142-171
- **description:** `Idempotency::lookup` returns the stored
  `command_type` after leaking it via `Box::leak` (finding
  ADAPTER-PG-011). On every read, a new `Box<str>` allocation is
  made and leaked. Under steady-state load (a single consumer
  with N tenants, each issuing K commands/hour, each retried
  once), the leak rate is `2NK` bytes/hour.
- **expected:** `crates/infra/storage/src/idempotency.rs:31` —
  the port uses `&'static str`; the column type is `VARCHAR`.
  The port should change to `String` to avoid the leak.
- **evidence:** `crates/adapters/storage-postgres/src/idempotency.rs:159-167` —
  ```rust
  let (payload, version, agg_ids) = unwrap_envelope(&r.outcome.0);
  Ok(Some(IdempotencyRecord {
      ...
      command_type: lookup_command_type(&r.command_type),
      ...
  }))
  ```

---

### FINDING 45

- **id:** ADAPTER-PG-045
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/connection.rs:40-46
- **description:** `PostgresConnection::fmt::Debug` omits the
  connection URL and pool stats, but also omits the
  `closed`-ness / readiness state of the underlying pool. A
  debugging session cannot tell from `Debug` whether the pool is
  exhausted or healthy.
- **expected:** `crates/infra/storage/src/port.rs:28-34` —
  Object-safe, `Send + Sync`, `Debug`-able adapters.
- **evidence:** `crates/adapters/storage-postgres/src/connection.rs:40-46` —
  ```rust
  impl fmt::Debug for PostgresConnection {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          f.debug_struct("PostgresConnection")
              .field("school", &self.school)
              .finish_non_exhaustive()
      }
  }
  ```
  Only `school` is exposed; `closed`, pool size, in-flight
  connection count are all absent.

---

### FINDING 46

- **id:** ADAPTER-PG-046
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/outbox.rs:139-158
- **description:** `Outbox::pending` selects only 12 columns
  (event_id, event_type, event_version, school_id, aggregate_id,
  aggregate_type, actor_id, correlation_id, causation_id,
  occurred_at, payload) and does not read `recorded_at`,
  `enqueued_at`, `published_at`, `attempts`, or `last_error`. A
  consumer that wants to size the relay or back off on
  repeatedly-failing rows cannot read those columns without a
  separate query.
- **expected:** `docs/schemas/event-schema.md` § 8 (referenced
  in DDL line 53).
- **evidence:** `crates/adapters/storage-postgres/src/outbox.rs:139-158` —
  ```rust
  let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(
      "SELECT \
          event_id, event_type, event_version, school_id, \
          aggregate_id, aggregate_type, actor_id, \
          correlation_id, causation_id, occurred_at, payload \
       FROM outbox \
       ...
  ```

---

### FINDING 47

- **id:** ADAPTER-PG-047
- **area:** adapters
- **severity:** Low
- **location:** crates/adapters/storage-postgres/src/bulk_attendance.rs:43-44
- **description:** The comment claims
  `24 columns × 1,000 rows = 24,000 placeholders (well under the cap)`,
  but the actual INSERT column list (line 148-152) is 24 columns
  × N rows where N is `rows.len()`. With `MAX_ROWS_PER_CALL = 1000`,
  the placeholder count is `24 × 1000 = 24000`. PG's
  `MAX_BINNED_TYPES` / parameter limit (per sqlx 0.8 docs) is
  `u16::MAX = 65535`. The 24k figure is correct, but the comment
  does not mention that `?` placeholders are used twice (one for
  VALUES, one for column list) — i.e. the bound parameter count
  is `2 × 24 × N = 48 × N`. With N=1000, that's 48000 placeholders.
  The comment understates by 2x.
- **expected:** `docs/schemas/sql-dialects/postgresql.md` and
  sqlx 0.8 documentation on parameter limits.
- **evidence:** `crates/adapters/storage-postgres/src/bulk_attendance.rs:42-44` —
  ```rust
  /// The per-call row cap. PostgreSQL caps a single prepared
  /// statement at 65,535 placeholders; 24 columns × 1,000 rows
  /// = 24,000 placeholders (well under the cap).
  pub(crate) const MAX_ROWS_PER_CALL: usize = 1000;
  ```

---

### END FINDINGS

**Total findings: 47**
