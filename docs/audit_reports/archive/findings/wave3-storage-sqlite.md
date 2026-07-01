# Wave 3 — SQLite Storage Adapter Audit Report

**Scope:** `crates/adapters/storage-sqlite/`, port contract `docs/ports/storage.md`, canonical DDL `migrations/engine/0000_engine_core.sqlite.sql`, Phase 1 handoff `docs/handoff/PHASE-1-HANDOFF.md`, dialect spec `docs/schemas/sql-dialects/sqlite.md`.

**Audit date:** 2026-06-23.

---

### FINDING 1

- **id:** ADAPTER-SQ-001
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:97-133`
- **description:** The `migrate()` implementation only loads `migrations/engine/0000_engine_core.sqlite.sql` (6 engine cross-cutting tables) plus the `bulk_attendance.sql` (1 attendance domain table). It does not walk any macro-emitted AST to emit the ~310 domain tables the engine claims to ship, and it does not honour `docs/schemas/sql-dialects/sqlite.md`'s `SqliteStorageAdapter::create_<table>_ddl()` contract at all. `Schema-registry` and `system_user` are emitted; no domain tables are emitted.
- **expected:** "The schema is emitted by the storage adapter at startup via `storage.create_schema().await`. … 4. Adapter emission — `educore-storage-<db>` walks the AST at schema-creation time and emits the dialect-specific DDL string. … 5. Consumer startup — `storage.create_schema().await` runs the DDL once per process lifetime." (`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission — end-to-end flow").
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:35` `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.sqlite.sql");` and `crates/adapters/storage-sqlite/src/storage.rs:108-124` only executes `SCHEMA_SQL` and `SqliteBulkAttendance::ensure_schema`. The crate has no `create_schema()`, no AST walk, and no domain-table emission code anywhere under `src/`.

---

### FINDING 2

- **id:** ADAPTER-SQ-002
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:139-175` (`append`)
- **description:** `Outbox::append` calls a plain `INSERT INTO outbox ...` and surfaces the underlying `sqlx::Error` (which includes the unique-constraint violation on `event_id`) as `DomainError::Infrastructure`. The port contract requires `DomainError::Conflict` on a duplicate `(school_id, event_id)`. The adapter silently downgrades a contract-mandated domain error to an infrastructure error.
- **expected:** "`Conflict` if an envelope with the same `event_id` was already appended in the same school." (`crates/infra/storage/src/outbox.rs:99-101`).
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:148-172` `sqlx::query::<sqlx::Sqlite>("INSERT INTO outbox ( ...")... .execute(&self.pool).await.map_err(|e| StringError(format!("outbox append: {e}")))?;` — no `match` on `sqlx::Error::Database(db)` to map `UniqueViolation` to `DomainError::conflict(...)`. Compare with `crates/adapters/storage-sqlite/src/bulk_attendance.rs:205-212` which DOES map `UniqueViolation` to `DomainError::conflict(...)`.

---

### FINDING 3

- **id:** ADAPTER-SQ-003
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/src/idempotency.rs:123-152` (`record`)
- **description:** `Idempotency::record` uses `INSERT OR REPLACE INTO idempotency ...`, which silently overwrites any prior row with the same composite key and discards the previous `command_id` (the implementation regenerates a fresh `Uuid::now_v7()` every call at line 124). The port contract requires `DomainError::Conflict` when a record with the same composite key exists with a different outcome, and `Ok(())` only when the new row is identical. The current behaviour violates both halves: it never returns `Conflict`, and it overwrites regardless of outcome equality.
- **expected:** "Stores `record`. Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome. Returns `Ok(())` if the record is a no-op write (same key, same outcome hash) — the engine uses this for at-least-once delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).
- **evidence:** `crates/adapters/storage-sqlite/src/idempotency.rs:134-149` `sqlx::query::<sqlx::Sqlite>("INSERT OR REPLACE INTO idempotency ( school_id, command_type, idempotency_key, command_id, outcome, recorded_at, expires_at ) VALUES (?, ?, ?, ?, ?, ?, ?)")` with `.bind(command_id.hyphenated())` where `command_id = Uuid::now_v7()` (line 124).

---

### FINDING 4

- **id:** ADAPTER-SQ-004
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:166-167` (`append`)
- **description:** `recorded_at` is bound to `envelope.occurred_at.as_datetime()` instead of `Utc::now()`. The DDL declares `recorded_at` as the persistence time (a separate column from `occurred_at`), and the engine invariant is that `recorded_at >= occurred_at` (it captures ingestion latency). Binding both to the same value obliterates that invariant.
- **expected:** "`recorded_at` Wall-clock time of the persistence (≥ `occurred_at`)" (`crates/infra/storage/src/event_log.rs:73`) and the outbox DDL column pair `occurred_at ... recorded_at ...` (`migrations/engine/0000_engine_core.sqlite.sql:74-75`).
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:166-169` `.bind(envelope.occurred_at.as_datetime()) // occurred_at\n        .bind(envelope.occurred_at.as_datetime()) // recorded_at ← BUG\n        .bind(payload_json)\n        .bind(now) // enqueued_at`. Compare `audit_log.rs:151-152` which correctly binds `entry.occurred_at.as_datetime()` and `recorded_at = Utc::now()` separately.

---

### FINDING 5

- **id:** ADAPTER-SQ-005
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:187-195` (`advance_cursor`) and `crates/adapters/storage-sqlite/src/storage.rs:175-185` (`cursor_for`)
- **description:** Both `cursor_for` and `advance_cursor` silently override the trait default of `DomainError::NotSupported`. `cursor_for` returns `Ok(VersionCursor::ZERO)` and `advance_cursor` returns `Ok(())`. The default-impl contract is the sync engine's safety net: non-sync adapters must fail loudly at startup. The SQLite implementation reports success, masking configuration problems and letting the sync engine start up against an adapter that is actually doing nothing.
- **expected:** "Default impls on the trait return `DomainError::NotSupported('sync primitives require the sync feature and a sync-capable adapter')`. The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup — not silently at runtime — so consumers see the configuration problem immediately." (`docs/ports/storage.md:112-116`).
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:175-185` `async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> { ... Ok(VersionCursor::ZERO) }` and `crates/adapters/storage-sqlite/src/storage.rs:187-195` `async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> { ... Ok(()) }` — both return success instead of `DomainError::not_supported(...)`.

---

### FINDING 6

- **id:** ADAPTER-SQ-006
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/idempotency.rs:65-75` (`to_record`)
- **description:** `IdempotencyRow::to_record` calls `Box::leak(self.command_type.clone().into_boxed_str())` on every read. The port struct's `command_type: &'static str` field forces this leak. In a long-running process serving many idempotency lookups the heap grows without bound — a slow but unbounded memory leak in production code.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." Adapter code must not leak memory per-call.
- **evidence:** `crates/adapters/storage-sqlite/src/idempotency.rs:68` `command_type: Box::leak(self.command_type.clone().into_boxed_str()),` (acknowledged as a known limitation in the doc-comment at lines 53-64 but not resolved in code).

---

### FINDING 7

- **id:** ADAPTER-SQ-007
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/idempotency.rs:65-75` (`to_record`)
- **description:** `IdempotencyRow::to_record` returns hard-coded `outcome_version: 0` and `affected_aggregate_ids: Vec::new()` because the DDL has no columns for them. The audit and idempotency contract relies on these two fields to detect "same key, different target" misuse and to version the outcome payload. The adapter silently discards both on read.
- **expected:** "`outcome_version`: The schema version of the `outcome` payload." and "`affected_aggregate_ids`: The aggregate ids touched by the original command. Used by the dispatcher to detect 'same idempotency key, but different target' misuse." (`crates/infra/storage/src/idempotency.rs:37-44`).
- **evidence:** `crates/adapters/storage-sqlite/src/idempotency.rs:42-50` `struct IdempotencyRow { ... outcome: String, recorded_at: DateTime<Utc>, expires_at: DateTime<Utc>, }` — no columns for outcome_version or affected_aggregate_ids; and `crates/adapters/storage-sqlite/src/idempotency.rs:71-73` hard-codes both to `0` and `Vec::new()`.

---

### FINDING 8

- **id:** ADAPTER-SQ-008
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:87-205` (every method on a closed adapter)
- **description:** Every `StorageAdapter` method (`begin`, `migrate`, `ping`, `watch_changes`, `cursor_for`, `advance_cursor`, `bulk_insert_student_attendances`) returns `DomainError::Conflict` when the adapter is closed. The port contract mandates `DomainError::Infrastructure`. Returning `Conflict` is structurally wrong (closing the pool is not a state conflict) and breaks error-handling callers that match on `ErrorKind::Infrastructure` to surface a degraded-storage alert.
- **expected:** "`close(self: Box<Self>) -> Result<()>; ... After `close`, any further call returns `Err(Infrastructure)`." (`crates/infra/storage/src/port.rs:53` and `docs/ports/storage.md:23`).
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:88-91, 99-102, 137-140, 159-163, 176-179, 188-191, 202-205` all call `DomainError::conflict("...")` instead of `DomainError::infrastructure(...)` after the `self.closed.load(Ordering::SeqCst)` check.

---

### FINDING 9

- **id:** ADAPTER-SQ-009
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/connection.rs:62-64, 109-111`
- **description:** The connection layer unconditionally requests `SqliteJournalMode::Wal` for both in-memory and file-backed connections. SQLite's WAL journal mode requires a file-backed database; for `sqlite::memory:` the journal mode silently downgrades to `MEMORY` at the SQLite layer (this is documented SQLite behaviour). The application code then issues `PRAGMA journal_mode = WAL` again in `after_connect` — which is a no-op for in-memory DBs but emits a SQLite "no-op" warning that the adapter silently swallows. The `in_memory` path is therefore misadvertised as WAL-backed.
- **expected:** "The pool is constrained to a single connection so every consumer in the same process sees the same in-memory database. This is the default connection for tests and single-process embedded deployments." (`crates/adapters/storage-sqlite/src/connection.rs:43-49`); and the spec requires WAL/NORMAL/foreign_keys PRAGMAs.
- **evidence:** `crates/adapters/storage-sqlite/src/connection.rs:62-64` `.journal_mode(SqliteJournalMode::Wal).synchronous(SqliteSynchronous::Normal).foreign_keys(true)` is applied to `SqliteConnectOptions::from_str("sqlite::memory:")` (line 55) the same way as the file-backed path. `crates/adapters/storage-sqlite/src/connection.rs:68-80` re-emits the same PRAGMAs in `after_connect`. The SQLite docs are explicit that WAL is silently downgraded for `:memory:`.

---

### FINDING 10

- **id:** ADAPTER-SQ-010
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:117, 127-132`
- **description:** `migrate()` hard-codes `MigrationReport { statements_executed: 0, already_at_version: false, ... }`. The `statements_executed` field exists to report the actual count of statements applied (useful for telemetry, migration-time SLOs, and idempotency verification) and the adapter explicitly discards the `sqlx` result (`let _ = result;` at line 117). `already_at_version` is always `false`, even when re-running `migrate()` on an already-migrated database — so the report cannot be used by callers to distinguish a first run from a no-op run.
- **expected:** Per `crates/infra/storage/src/change_stream.rs:243-255`: "`statements_executed`: The number of statements executed (DDL or DML)." and "`already_at_version`: Whether the migration was a no-op (already at `version`)." The handoff doc claims the migration is "idempotent thanks to the `IF NOT EXISTS` clauses" but the report does not surface that idempotency.
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:108-132` shows `let _ = result;` discarding the `Execute` result, and `Ok(MigrationReport { version: SCHEMA_VERSION, statements_executed: 0, duration, already_at_version: false })`.

---

### FINDING 11

- **id:** ADAPTER-SQ-011
- **area:** adapters
- **severity:** High
- **location:** `migrations/engine/0000_engine_core.sqlite.sql:64-235` (all six tables)
- **description:** The canonical SQLite DDL omits both the `STRICT` table option and the `WITHOUT ROWID` option that the dialect spec mandates for every cross-cutting table. `STRICT` enforces type affinity (preventing SQLite's silent string-to-integer coercion), and `WITHOUT ROWID` saves 4-8 bytes per row for lookup-only tables. The dialect spec is explicit that the engine refuses to write to non-`STRICT` tables.
- **expected:** "`STRICT` enforces the type affinity. Without it, SQLite allows silent type coercion (e.g. inserting `'hello'` into an `INTEGER` column). The engine refuses to write to non-`STRICT` tables." and "`WITHOUT ROWID` saves 4-8 bytes per row and is faster for point-lookups. The engine's `outbox`, `event_log`, `audit_log`, `idempotency`, and `schema_registry` are all `WITHOUT ROWID`." (`docs/schemas/sql-dialects/sqlite.md:78-99`).
- **evidence:** `migrations/engine/0000_engine_core.sqlite.sql:64-82` `outbox`, `:107-129` `audit_log`, `:151-160` `idempotency`, `:174-188` `event_log`, `:208-216` `schema_registry`, `:229-235` `system_user` all use `CREATE TABLE IF NOT EXISTS <name> ( ... );` — no `STRICT` and no `WITHOUT ROWID` clause on any of them.

---

### FINDING 12

- **id:** ADAPTER-SQ-012
- **area:** adapters
- **severity:** High
- **location:** `migrations/engine/0000_engine_core.sqlite.sql:76, 124-125, 156, 186`
- **description:** The `outbox.payload`, `audit_log.before_snapshot` / `audit_log.after_snapshot` / `audit_log.metadata`, `idempotency.outcome`, and `event_log.payload` columns are declared as plain `TEXT NOT NULL` with no `CHECK (json_valid(x))` constraint. The dialect spec mandates `json_valid(...)` CHECK on every JSON column. Without the CHECK, the database accepts any text and the type-coercion invariant that the engine relies on is enforced only by application code that may be bypassed by direct SQL (e.g. backfill scripts, ad-hoc queries by ops staff).
- **expected:** `"payload"         TEXT NOT NULL CHECK (json_valid("payload"))` and similar `"before_snapshot" TEXT CHECK ("before_snapshot" IS NULL OR json_valid("before_snapshot"))` (`docs/schemas/sql-dialects/sqlite.md:202, 238-240, 265, 290, 309`).
- **evidence:** `migrations/engine/0000_engine_core.sqlite.sql:76` `"payload" TEXT NOT NULL` (no CHECK), `:124-125` `"before_snapshot" TEXT NULL` / `"after_snapshot" TEXT NULL` (no CHECK), `:126` `"metadata" TEXT NULL` (no CHECK), `:156` `"outcome" TEXT NOT NULL` (no CHECK), `:186` `"payload" TEXT NOT NULL` (no CHECK).

---

### FINDING 13

- **id:** ADAPTER-SQ-013
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/bulk_attendance.sql:14-39` (`attendance_student_attendances` schema)
- **description:** The `attendance_student_attendances` domain table stores every UUID as `BLOB` (16 bytes) instead of `TEXT` with `CHECK (length(x) = 36)` as the engine spec mandates for UUIDv7 columns. It also omits `STRICT`, `WITHOUT ROWID`, the `json_valid` CHECKs on TEXT JSON columns, and every length-36 UUID CHECK that the spec requires. The result is that the bulk-attendance table uses a completely different wire form than the 6 engine cross-cutting tables in the same database, and an ops engineer writing a `SELECT * FROM attendance_student_attendances` JOIN against the engine tables cannot compare UUIDs without explicit hex conversion.
- **expected:** "`id` TEXT NOT NULL PRIMARY KEY" with `CHECK (length("id") = 36)` and `STRICT` (`docs/schemas/sql-dialects/sqlite.md:54, 80, 391-393`).
- **evidence:** `crates/adapters/storage-sqlite/src/bulk_attendance.sql:14-39` — every UUID column (`school_id`, `id`, `student_id`, `student_record_id`, `class_id`, `section_id`, `marked_by`, `created_by`, `updated_by`, `last_event_id`, `correlation_id`) is declared `BLOB`. `bulk_attendance.sql:13` is `CREATE TABLE IF NOT EXISTS attendance_student_attendances ( ... );` with no `STRICT` or `WITHOUT ROWID`. `crates/adapters/storage-sqlite/src/student_attendance_row.rs:111-180` confirms the adapter binds UUIDs as `Vec<u8>` (16 bytes big-endian) via `school_id_bytes()`, etc.

---

### FINDING 14

- **id:** ADAPTER-SQ-014
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:144-145, :76-77` and `crates/adapters/storage-sqlite/src/event_log.rs:55, 95`
- **description:** Both `Outbox::append`, `OutboxRow::to_envelope`, `EventLog::append`, and `EventLogRow::to_entry` use `i32::try_from(...).unwrap_or(0)` and `u32::try_from(...).unwrap_or(0)` to silently clamp `schema_version` on overflow. The engine's invariant is that `schema_version` is a small positive integer, but the silent fallback to `0` discards data without surfacing the error — a caller that has produced a malformed envelope will not see `Err(Validation)` and downstream consumers will silently treat the event as schema v0, which may have an unrelated payload shape.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." and "All public APIs return `Result` for fallible operations."
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:77` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`, `:145` `let event_version = i32::try_from(envelope.schema_version).unwrap_or(0);`; `crates/adapters/storage-sqlite/src/event_log.rs:55` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`, `:95` `let event_version = i32::try_from(entry.schema_version).unwrap_or(0);`.

---

### FINDING 15

- **id:** ADAPTER-SQ-015
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/connection.rs:135`
- **description:** `SqliteConnection::connect` logs the raw URL via `debug!(school = %school, url, "opened file-backed sqlite")`. SQLx SQLite URLs do not embed credentials, but the convention is established and any future migration to a URL-bearing driver (e.g. a remote-sqlite bridge or a query string with `?mode=ro&key=...`) will silently log secrets. Logging the raw URL is the standard mistake that turns into a credential leak later.
- **expected:** Per `AGENTS.md` § "Authoritative Documents" and the audit-first rule: no credentials or PII in log lines.
- **evidence:** `crates/adapters/storage-sqlite/src/connection.rs:135` `debug!(school = %school, url, "opened file-backed sqlite");` — the full `url: &str` is bound as a tracing field with no redaction. Also `crates/adapters/storage-sqlite/src/connection.rs:106-107, 131-133` interpolate the URL into the `StringError` message that becomes the public `DomainError::infrastructure` variant.

---

### FINDING 16

- **id:** ADAPTER-SQ-016
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:197-209` (`bulk_insert_student_attendances`)
- **description:** The adapter-level `bulk_insert_student_attendances` accepts a `&TenantContext` but immediately ignores it. It constructs a fresh `SqliteBulkAttendance` handle from `self.conn.db().clone(), self.conn.school()` and calls `handle.bulk_insert(ctx.school_id, rows)` — the per-row `school_id` validation in `bulk_insert_into` checks against `ctx.school_id` (the caller's anchor), not against `self.conn.school()`. If the adapter is opened with school A and a caller from tenant B invokes `bulk_insert_student_attendances(&TenantContext{school_id: B, ...}, rows)`, every row's `school_id` is validated against B but the rows are written to the connection's pool which is scoped to A. The validation passes (B == B) and rows are silently inserted into the wrong school.
- **expected:** "`StorageAdapter::bulk_insert_student_attendances` ... MUST validate that every row's `school_id` matches `ctx.school_id` and reject the call with a `DomainError::Validation` otherwise." (`crates/infra/storage/src/port.rs:60-63`).
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:197-209` constructs `SqliteBulkAttendance::new(self.conn.db().clone(), self.conn.school())` and passes `ctx.school_id` (which may differ from `self.conn.school()`) to `bulk_insert`. `crates/adapters/storage-sqlite/src/bulk_attendance.rs:145-152` validates `r.school_id != school_id` where `school_id` is the caller-supplied parameter, not the connection's scoped school. The transaction-level path (`transaction.rs:134-148`) correctly validates against the transaction's scoped school.

---

### FINDING 17

- **id:** ADAPTER-SQ-017
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/tests/outbox_e2e.rs` (entire file)
- **description:** The test suite contains exactly one end-to-end test (`outbox_append_and_pending_round_trip`) covering only the outbox sub-port. No tests exist for: `audit_log.append` / `audit_log.read_for_target`, `event_log.append` / `event_log.read` / `event_log.count`, `idempotency.lookup` / `idempotency.record` / `idempotency.purge_older_than`, `bulk_insert_student_attendances` (the Phase 5 critical path), `migrate()` idempotency, `cursor_for` / `advance_cursor` return-value verification, `ping()`, `close()` lifecycle, tenant-isolation enforcement, SQL-injection attempts, or any round-trip across the `SqliteTransaction` boundary. The single test path uses the in-memory constructor only.
- **expected:** Per `docs/ports/storage.md:468-477`: "The port requires: Unit tests of every repository method. Integration tests against a real database (testcontainers). A parity test verifying identical behavior across adapters. A tenancy test verifying cross-tenant reads are blocked. A failure-injection test (e.g. deadlock retry, connection drop). A load test (10k attendance marks in <5s)."
- **evidence:** `ls crates/adapters/storage-sqlite/tests/` returns only `outbox_e2e.rs`. The Phase 1 handoff at `docs/handoff/PHASE-1-HANDOFF.md:131-133` explicitly admits: "the existing e2e currently exercises the in-memory path only. The single-writer deployment model ... is documented but not tested at scale."

---

### FINDING 18

- **id:** ADAPTER-SQ-018
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:1-210` (entire `StorageAdapter` impl)
- **description:** The `StorageAdapter` trait in `docs/ports/storage.md:17-89` enumerates ~22 per-aggregate repository methods (`students()`, `guardians()`, `classes()`, ..., one per aggregate across 15 domains). The actual port trait at `crates/infra/storage/src/port.rs:34-150` exposes only 5 methods plus 4 sync primitives. The SQLite adapter implements the actual trait (no repository methods) — meaning **none** of the documented per-aggregate repository handles are implemented. The dialect spec promises `SqliteStorageAdapter::create_<table>_ddl()` per aggregate; no such method exists in the crate.
- **expected:** The port trait in `docs/ports/storage.md` declares `fn students(&self) -> Arc<dyn StudentRepository>;` and ~21 sibling methods, "one handle per aggregate, across all 15 domains (~80+ total)" (`docs/ports/storage.md:50`). Each adapter must translate the macro-emitted `QueryNode` AST into a SQLite execution plan.
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:84-211` implements only `begin`, `migrate`, `ping`, `close`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`, and `bulk_insert_student_attendances`. `grep -n 'students\|guardians\|classes\|sections' crates/adapters/storage-sqlite/src/` returns no repository handle of any kind.

---

### FINDING 19

- **id:** ADAPTER-SQ-019
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:1-38` (lib doc) and `crates/adapters/storage-sqlite/Cargo.toml:1-26`
- **description:** The dialect spec at `docs/schemas/sql-dialects/sqlite.md:7-8, 397-410` documents the adapter as using `rusqlite` (`The SqliteStorage adapter uses rusqlite for the connection. rusqlite 0.31+ is the recommended version.`) and a constructor pattern `SqliteStorage::open("path/to/db.sqlite")?` with `.with_key(b"...")?` for encryption. The actual crate uses `sqlx::SqlitePool` (no `rusqlite` dependency) and an entirely different API surface (`SqliteConnection::connect(url, school)`, `SqliteStorageAdapter::new(conn)`). The handoff at `docs/handoff/PHASE-1-HANDOFF.md:29-34` records this as a deliberate departure, but `docs/schemas/sql-dialects/sqlite.md` has not been updated to reflect the change.
- **expected:** Adapter implementation notes say rusqlite; `Cargo.toml` should declare `rusqlite` per the spec.
- **evidence:** `crates/adapters/storage-sqlite/Cargo.toml:13-26` declares `sqlx = { workspace = true }` with no `rusqlite` dependency. `crates/adapters/storage-sqlite/src/connection.rs:13-15` `use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};` — no rusqlite. The dialect spec's `Adapter implementation notes` section (`docs/schemas/sql-dialects/sqlite.md:395-410`) is stale.

---

### FINDING 20

- **id:** ADAPTER-SQ-020
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/audit_log.rs:13-25` (`SqliteAuditLog`)
- **description:** `AuditLogEntry` carries only a subset of the DDL columns. On write the adapter hardcodes `actor_type = "user"`, `source = "system"`, `recorded_at = Utc::now()`, `command_id = NULL`, `ip = NULL`, `user_agent = NULL`, `session_id = NULL`, `cross_tenant = 0` — none of which are parameterised by the entry struct. The port trait's `AuditLogEntry` has no slot for these fields, but the handoff (`docs/handoff/PHASE-1-HANDOFF.md:168-175`) acknowledges the gap. The practical effect: an audit row written through this adapter cannot distinguish a user-initiated mutation from a system mutation, cannot record the originating IP, and cannot be correlated to a `command_id` — the very fields the audit schema is designed to capture.
- **expected:** "`actor_type`: user, system, integration, scheduled. … `source`: rest, graphql, cli, internal. … `ip`, `user_agent`, `session_id`: caller context." (`docs/schemas/audit-schema.md` § 13 and the DDL column list at `migrations/engine/0000_engine_core.sqlite.sql:107-129`).
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:140-156` binds hard-coded literals: `bind("user")`, `bind("system")`, `.bind(recorded_at)` (computed locally as `Utc::now()`), and `NULL` literals embedded in the SQL string for `command_id`, `ip`, `user_agent`, `session_id`, with `0` literal for `cross_tenant`.

---

### FINDING 21

- **id:** ADAPTER-SQ-021
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:48-66` (`EventLogRow::to_entry`)
- **description:** `EventLogRow::to_entry` always returns `ActiveStatus::Active` because the DDL has no `active_status` column on `event_log`. The port contract is that consumers can transition an event row to `Retired` for GDPR erasure; this transition has no physical storage in the SQLite adapter. Once written, an event row is forever `Active` regardless of any consumer-driven retraction.
- **expected:** "The event log carries `active_status` so consumers can retire events (e.g. for GDPR erasure) without deleting the row (audit trails must remain)." (`crates/infra/storage/src/event_log.rs:42-45`).
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:29-44` `struct EventLogRow { event_id, event_type, event_version, school_id, aggregate_id, aggregate_type, actor_id, correlation_id, causation_id, occurred_at, recorded_at, payload, }` — no `active_status` column. `:64` `active_status: ActiveStatus::Active,` is hardcoded. Compare `migrations/engine/0000_engine_core.sqlite.sql:174-188` which also lacks an `active_status` column.

---

### FINDING 22

- **id:** ADAPTER-SQ-022
- **area:** adapters
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.sqlite.sql:174-188` (`event_log`)
- **description:** The `event_log` table DDL lacks the `active_status` column that the port trait's `EventLogEntry::active_status` field requires. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:170`) acknowledges the gap ("no `active_status` on `event_log`") and treats it as Phase 2 work. Production code that calls `EventLog::read` will get back rows whose `active_status` is silently hardcoded to `Active`, masking any future GDPR-retirement semantics.
- **expected:** `active_status INTEGER NOT NULL DEFAULT 1 CHECK (active_status IN (0, 1))` per `docs/schemas/sql-dialects/sqlite.md:367-368` ("`"active_status"     INTEGER NOT NULL DEFAULT 1 CHECK ("active_status" IN (0,1))`").
- **evidence:** `migrations/engine/0000_engine_core.sqlite.sql:174-188` — the column list runs `event_id ... payload` with no `active_status`. The dialect spec's example for `academic_students` (line 367) shows the column is mandatory on every aggregate; `event_log` is not exempt.

---

### FINDING 23

- **id:** ADAPTER-SQ-023
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:243-249` (trailing `const _`)
- **description:** The file ends with a `const _: fn() = || { let _b: Bytes = Bytes::new(); };` block whose purpose is to suppress an unused-import warning for `bytes::Bytes`. The `bytes` crate is imported but the only consumer of the `Bytes` type in this file is `SerializedEnvelope::payload`, which is constructed by callers, not by this module. The `Bytes` import is dead and the suppression block is a code smell.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:243-249` `#[allow(dead_code)]\nconst _: fn() = || { let _b: Bytes = Bytes::new(); };`. Also `crates/adapters/storage-sqlite/src/outbox.rs:122-135` defines an `IntoUuid` extension trait with `#[allow(dead_code)]` that is never referenced anywhere in the crate.

---

### FINDING 24

- **id:** ADAPTER-SQ-024
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/transaction.rs:105-117` (`commit` / `rollback`)
- **description:** The `commit` and `rollback` impls are no-ops that only flip the `done`/`rolled_back` atomic flag. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:36-46, 157-167`) acknowledges this as a deliberate Phase 1 simplification — but in production this means a caller that invokes `tx.commit()` does not actually commit anything: the writes are visible immediately (sqlx auto-commits per call) and the "commit" step is purely cosmetic. A caller that wants to roll back has no way to undo writes that were committed by earlier sub-port calls inside the same `Transaction`.
- **expected:** "Commits the transaction. All outbox appends, aggregate mutations, audit log writes, idempotency records, and event log rows become durable." (`crates/infra/storage/src/transaction.rs:35-37`).
- **evidence:** `crates/adapters/storage-sqlite/src/transaction.rs:107-116` `async fn commit(self: Box<Self>) -> Result<()> { self.done.store(true, Ordering::SeqCst); Ok(()) }` and `async fn rollback(self: Box<Self>) -> Result<()> { self.rolled_back.store(true, Ordering::SeqCst); self.done.store(true, Ordering::SeqCst); Ok(()) }` — neither call interacts with the database.

---

### FINDING 25

- **id:** ADAPTER-SQ-025
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/transaction.rs:52-73` (`SqliteTransaction` struct)
- **description:** The `SqliteTransaction` holds four `*Box` sub-port handles and a `_pool: SqlitePool` even though the `commit`/`rollback` impls do nothing with the pool. The `_pool` field name (leading underscore) is a code smell that the field is unused; the actual sub-port handles each cloned their own pool internally (`SqliteOutbox::new(pool.clone(), school)`, etc.) so the field could be deleted entirely. The unused field makes the struct bigger than necessary and obscures the lack of real transactional semantics.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `crates/adapters/storage-sqlite/src/transaction.rs:72` `_pool: SqlitePool,` and `crates/adapters/storage-sqlite/src/transaction.rs:86-101` shows the field is only assigned, never read in the rest of the file.

---

### FINDING 26

- **id:** ADAPTER-SQ-026
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:118-135` (`IntoUuid` trait)
- **description:** The file declares a `pub(crate) trait IntoUuid { fn into_uuid(self) -> uuid::Uuid; }` and a blanket impl `impl IntoUuid for Hyphenated` with `#[allow(dead_code)]`. The trait is never used anywhere in the workspace (the conversion from `Hyphenated` to `uuid::Uuid` is done inline via `*self.as_uuid()` at every call site). This is dead code that survived from a SurrealDB-pattern mirror.
- **expected:** Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue."
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:122-135` defines the trait and impl with `#[allow(dead_code)]`. `grep -rn 'IntoUuid' crates/` shows only the definition and impl, no consumers.

---

### FINDING 27

- **id:** ADAPTER-SQ-027
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/util.rs:10-13` (`bytes_to_json`)
- **description:** `bytes_to_json` calls `serde_json::from_slice(bytes).unwrap_or_else(|_| ...)` and falls back to a JSON-string wrapper when the bytes are not valid JSON. The round-trip is therefore lossy: a payload that fails to parse as JSON is stored as a quoted UTF-8 string. The downstream `json_to_bytes` (line 18-23) does the inverse — it serialises the JSON-Value back to bytes, so a non-JSON payload round-trips as `"original utf-8 bytes"` (a JSON string literal). The semantic of `Outbox::pending` therefore silently changes: a caller reading back an outbox row whose payload was a binary blob or invalid UTF-8 receives a string-wrapped version that loses the original byte boundaries.
- **expected:** Per `AGENTS.md` § "Production-ready" and "All public APIs are documented with rustdoc"; payload round-trip should be byte-exact or documented as lossy.
- **evidence:** `crates/adapters/storage-sqlite/src/util.rs:10-13` `pub(crate) fn bytes_to_json(bytes: &Bytes) -> serde_json::Value { serde_json::from_slice(bytes).unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(bytes).into_owned())) }`. The mirror on the write path at `idempotency.rs:133` `let outcome_str = String::from_utf8_lossy(&record.outcome).into_owned();` is similarly lossy on non-UTF-8 payloads.

---

### FINDING 28

- **id:** ADAPTER-SQ-028
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:150-153` (`close`)
- **description:** `close(self: Box<Self>)` flips the `closed` atomic and returns `Ok(())`. It does not call `pool.close().await` to release the underlying connections. For an in-memory `SqlitePool`, dropping the pool is sufficient; for a file-backed pool, not awaiting `pool.close()` can leave the WAL writer thread alive until the process exits. The method name promises a graceful close but the implementation does not.
- **expected:** "`close(self: Box<Self>) -> Result<()>; // Closes the adapter, releasing all underlying connections." (`crates/infra/storage/src/port.rs:52-53`).
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:150-153` `async fn close(self: Box<Self>) -> Result<()> { self.closed.store(true, Ordering::SeqCst); Ok(()) }`. No call to `self.conn.db().close().await`.

---

### FINDING 29

- **id:** ADAPTER-SQ-029
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:108-115` (`bulk_insert`) and `crates/adapters/storage-sqlite/src/bulk_attendance.rs:145-152`
- **description:** `bulk_insert` is decorated with `#[instrument(skip(self, rows), fields(n = rows.len(), school = %school_id))]` and the validation error message at lines 147-151 includes `expected {school_id}, got {}` — both UUIDs. UUIDs themselves are not PII, but the validation path also leaks the row index `i` and the `school_id` into the error, which is then surfaced as `DomainError::Validation`. The handoff (`docs/handoff/PHASE-1-HANDOFF.md:113-134`) describes the bulk path as the engine's "bulk-marking service" entry point — student attendance rows carry PII (student names, dates) even if the school_id alone is not PII, and the validation error does not redact that the discrepancy happened on a specific row index.
- **expected:** Per `AGENTS.md` § "Engine Rules": "Production-ready. Real schools, real students, real money."
- **evidence:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:147-151` `return Err(DomainError::validation(format!("bulk_insert_student_attendances: row {i} school_id mismatch (expected {school_id}, got {})", r.school_id)));` and the `#[instrument]` at line 108 includes the school_id.

---

### FINDING 30

- **id:** ADAPTER-SQ-030
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:198-224` (`mark_published`)
- **description:** `mark_published` updates `published_at` using SQLite's `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` (SQL-side clock), while every other timestamp in the adapter is written by the application via `chrono::Utc::now()` (application-side clock). A single adapter now has two clocks: SQLite's `strftime` returns UTC wall-clock, but on a host whose system clock has drifted, the two will disagree and `pending` queries ordered by `enqueued_at` may interleave with `published_at` writes from a different clock. This makes post-mortem reasoning about event publication latency unreliable.
- **expected:** Per `crates/adapters/storage-sqlite/src/outbox.rs:147` `let now = Utc::now();` — the write path uses `chrono::Utc::now()`. Consistency requires the same clock source for all timestamps.
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:207-211` `let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new("UPDATE outbox SET published_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE event_id IN (", );`. Compare `outbox.rs:169` `.bind(now)` where `now = Utc::now()` is the application clock used for `enqueued_at`.

---

### FINDING 31

- **id:** ADAPTER-SQ-031
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:97-122` (`EventLog::append`) and `migrations/engine/0000_engine_core.sqlite.sql:174-188`
- **description:** `EventLog::append` inserts a row with no `ON CONFLICT` clause; the underlying primary-key violation on duplicate `event_id` is converted to `DomainError::Infrastructure` via `StringError`. The port trait does not specify the duplicate behaviour, but the outbox trait does (`Conflict` — see ADAPTER-SQ-002). Without explicit `Conflict` mapping, callers cannot distinguish "already recorded" from "DB failure" without inspecting the error string.
- **expected:** Consistent error semantics across sub-ports: `Conflict` for duplicate primary-key violations on event-bearing tables (outbox, event_log, idempotency).
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:97-119` `sqlx::query::<sqlx::Sqlite>("INSERT INTO event_log ( ...")... .execute(&self.pool).await.map_err(|e| StringError(format!("event_log append: {e}")))?;` — no `ON CONFLICT` clause and no `match` on `UniqueViolation`.

---

### FINDING 32

- **id:** ADAPTER-SQ-032
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/idempotency.rs:154-169` (`purge_older_than`)
- **description:** `purge_older_than` executes a `DELETE FROM idempotency WHERE school_id = ?1 AND recorded_at < ?2` and returns `result.rows_affected()`. The DELETE is not wrapped in a transaction and does not `LIMIT` the batch size. A school with millions of expired records and a one-time retention sweep will issue a single huge DELETE that holds a write lock for the entire duration. SQLite serialises writes; blocking the only writer for the duration of a multi-million-row DELETE stalls every concurrent command.
- **expected:** A batched purge (e.g. `DELETE ... WHERE ... LIMIT 1000` looped) so the writer thread is freed between batches and other commands can interleave.
- **evidence:** `crates/adapters/storage-sqlite/src/idempotency.rs:155-168` `let result = sqlx::query::<sqlx::Sqlite>("DELETE FROM idempotency WHERE school_id = ?1 AND recorded_at < ?2").bind(school_id.as_uuid().hyphenated()).bind(cutoff.as_datetime()).execute(&self.pool).await.map_err(...)?; let n = result.rows_affected();` — no `LIMIT`, no batching.

---

### FINDING 33

- **id:** ADAPTER-SQ-033
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:151-194` (`build_read_query`)
- **description:** The dynamic `WHERE` builder emits `LIMIT` via `qb.push_bind(i64::from(filter.limit))`. There is no upper-bound enforcement: a caller passing `filter.limit = u32::MAX` (the trait default is 1000 but `EventLogFilter::limit` is a public field) will issue a single query that materialises the entire school's event log. SQLite's `LIMIT` accepts up to `i64::MAX` placeholders, so the query is technically legal but operationally catastrophic.
- **expected:** Per `crates/infra/storage/src/event_log.rs:154`: "Returns events matching `filter` ordered by `recorded_at` ascending. The cap is `filter.limit`; the adapter may enforce a lower cap for safety." — the adapter should clamp the limit.
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:189-192` `if !count_only { qb.push(" ORDER BY recorded_at ASC LIMIT ").push_bind(i64::from(filter.limit)); }`. No `min(filter.limit, MAX_LIMIT)` clamp.

---

### FINDING 34

- **id:** ADAPTER-SQ-034
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-sqlite/src/audit_log.rs:164-190` (`read_for_target`)
- **description:** `read_for_target` returns audit rows in `occurred_at ASC` order with no secondary sort by `audit_id`. The audit log's primary key is `audit_id` (a UUIDv7, which is time-ordered), but `audit_id` and `occurred_at` are produced by different sources — `audit_id` by `Uuid::now_v7()` (line 121) and `occurred_at` by the caller's `entry.occurred_at` (line 151). Two audit rows appended in the same scheduler tick will have `occurred_at` equal to the millisecond and `audit_id` differing by UUIDv7 sub-millisecond. SQLite's default sort is unstable for ties, so pagination by `LIMIT` will return arbitrary slices across ties — auditors paginating through a target's history will see inconsistent results between pages.
- **expected:** A deterministic tiebreaker on `audit_id` for pagination.
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:178-182` `FROM audit_log WHERE school_id = ?1 AND resource_id = ?2 ORDER BY occurred_at ASC LIMIT ?3` — no secondary sort key.

---

### FINDING 35

- **id:** ADAPTER-SQ-035
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:205-217`
- **description:** The unique-violation handler in `bulk_insert_into` rolls back the transaction and returns `DomainError::conflict("bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row")`. The rollback uses `let _ = tx.rollback().await;` (line 208 and 214), discarding any error from the rollback itself. If the rollback fails (e.g. connection broken during rollback), the adapter silently swallows the failure and returns the original error, leaving the connection in an indeterminate state. The next operation may reuse a connection that the pool believes was rolled back.
- **expected:** The rollback error should be logged via `tracing::error!` even if the original error is preferred for return.
- **evidence:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:208, 214` `let _ = tx.rollback().await;` (twice).

---

### FINDING 36

- **id:** ADAPTER-SQ-036
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:118-148` (`bulk_insert_into`)
- **description:** `bulk_insert_into` validates `r.school_id != school_id` per row before opening the transaction. For large batches (the doc says batches of up to 40 rows), this is fine; but for a 10k-row input the function still opens one transaction holding the writer lock for the entire batch duration. The doc-string at line 28-29 claims "a partial failure rolls back all of the batches, not just the failed one" but the implementation holds the writer for the full 10k-row duration, which on SQLite is a single-writer lock that blocks every other command.
- **expected:** Per `docs/ports/storage.md:477`: "A load test (10k attendance marks in <5s)." — the implementation holds the writer lock across the entire 10k-row insert; no test exists to verify the lock-hold time.
- **evidence:** `crates/adapters/storage-sqlite/src/bulk_attendance.rs:160-164` `let mut tx = pool.begin().await.map_err(...)?;` and `crates/adapters/storage-sqlite/src/bulk_attendance.rs:166-218` loops all batches inside this single transaction.

---

### FINDING 37

- **id:** ADAPTER-SQ-037
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/outbox.rs:46-65` (`OutboxRow`)
- **description:** `OutboxRow` carries `recorded_at`, `enqueued_at`, `published_at`, `attempts`, `last_error` but the `to_envelope` method does not surface any of them — only `occurred_at` is mapped into the `SerializedEnvelope`. The handoff at `docs/handoff/PHASE-1-HANDOFF.md:47` notes `#[allow(dead_code)]` to silence the warning, but the practical consequence is that callers reading pending envelopes cannot see how many times the relay has retried (`attempts`) or what the last error was (`last_error`) — fields that exist in the table and are written on every state transition but are dead-on-arrival on read.
- **expected:** Either expose retry state through the port or omit the columns from the SELECT.
- **evidence:** `crates/adapters/storage-sqlite/src/outbox.rs:47` `#[allow(dead_code)] // `recorded_at`, `enqueued_at`, `published_at`, `attempts`, `last_error` are read for future parity tests.` and `crates/adapters/storage-sqlite/src/outbox.rs:71-87` `to_envelope` only constructs `event_id, event_type, schema_version, school_id, aggregate_id, aggregate_type, actor_id, correlation_id, causation_id, occurred_at, payload`.

---

### FINDING 38

- **id:** ADAPTER-SQ-038
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/idempotency.rs:128-129`
- **description:** `expires_at = recorded_at + Duration::days(30)` is hardcoded in the adapter. The port trait documents that retention is consumer-configurable (the engine ships no default), but the SQLite adapter unilaterally picks 30 days. A consumer that needs 7-day or 365-day retention has no adapter-level override and must patch the crate.
- **expected:** Per `crates/infra/storage/src/idempotency.rs:104-105`: "Purges idempotency records older than the configured retention window."
- **evidence:** `crates/adapters/storage-sqlite/src/idempotency.rs:129` `let expires_at = record.recorded_at.as_datetime() + Duration::days(30);` — no configuration hook.

---

### FINDING 39

- **id:** ADAPTER-SQ-039
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/audit_log.rs:120-162` (`append`)
- **description:** `recorded_at` is set to `Utc::now()` and `occurred_at` is set from `entry.occurred_at` (passed by the caller). The DDL allows `recorded_at` to precede `occurred_at` if a caller sets `entry.occurred_at` in the future or if the host clock has drifted. The schema has no `CHECK (recorded_at >= occurred_at)` to catch this invariant violation at the database layer.
- **expected:** `CHECK (recorded_at >= occurred_at)` on `audit_log` and `event_log`.
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:131, 151-152` shows `recorded_at = Utc::now()` and `occurred_at = entry.occurred_at.as_datetime()`. `migrations/engine/0000_engine_core.sqlite.sql:107-129` has no `CHECK` on the `occurred_at`/`recorded_at` pair.

---

### FINDING 40

- **id:** ADAPTER-SQ-040
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:124-148` (`read` and `count`)
- **description:** `read` and `count` use `WHERE recorded_at >= ?` and `WHERE recorded_at < ?` for the `since` and `until` filters (lines 181-188). The port trait's `EventLogFilter::since`/`until` fields are typed as `Timestamp` and bind `as_datetime()` (a `chrono::DateTime<Utc>`). SQLite TEXT columns sort lexicographically; ISO 8601 with a fixed `Z` suffix sorts correctly only when every value uses the same suffix. The `event_log` DDL has no `CHECK` that timestamps end with `Z` or use a fixed-length format, so a write that uses `+00:00` instead of `Z` will sort incorrectly against a write that uses `Z`.
- **expected:** A `CHECK` constraint enforcing ISO 8601 UTC suffix on every timestamp column.
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:181-188` `if let Some(since) = filter.since { qb.push(" AND recorded_at >= ").push_bind(since.as_datetime()); } if let Some(until) = filter.until { qb.push(" AND recorded_at < ").push_bind(until.as_datetime()); }` — no format constraint at the schema layer (`migrations/engine/0000_engine_core.sqlite.sql:184-185`).

---

### FINDING 41

- **id:** ADAPTER-SQ-041
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/connection.rs:68-80, 114-126`
- **description:** The `after_connect` hook issues `PRAGMA journal_mode = WAL` first, then `PRAGMA synchronous = NORMAL`, then `PRAGMA foreign_keys = ON`. The order matters because `journal_mode` for a file-backed DB is sticky (changing it requires an exclusive lock and may rewrite the file). The hook does not verify that the `journal_mode` PRAGMA actually succeeded; a SQLite error or warning would be silently ignored.
- **expected:** Verify the PRAGMA result for `journal_mode` (e.g. `PRAGMA journal_mode` after the SET to confirm the value is `wal`).
- **evidence:** `crates/adapters/storage-sqlite/src/connection.rs:68-80` and `:114-126` `sqlx::query("PRAGMA journal_mode = WAL").execute(&mut *conn).await?;` etc. — no verification of the round-tripped value.

---

### FINDING 42

- **id:** ADAPTER-SQ-042
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:38-40`
- **description:** `SCHEMA_VERSION: u32 = 1` is a hard-coded constant with no link to a migration-tracking table. There is no `schema_migrations` or `schema_registry.version` row that records the applied version, so `already_at_version` in the `MigrationReport` is structurally incapable of ever being `true`. The handoff claims idempotency via `IF NOT EXISTS`, but the report cannot reflect that idempotency.
- **expected:** A migration-tracking table (e.g. `schema_migrations(version INT PRIMARY KEY, applied_at TEXT)`) so the adapter can distinguish a no-op run from a fresh migration.
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:40` `const SCHEMA_VERSION: u32 = 1;` and `:127-132` `Ok(MigrationReport { ..., already_at_version: false })` (always false).

---

### FINDING 43

- **id:** ADAPTER-SQ-043
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/error.rs:29-33`
- **description:** The `From<StringError> for educore_core::error::DomainError` impl wraps the `StringError` as `DomainError::infrastructure`. This is the conversion the entire crate relies on for `?` propagation. The conversion drops the original `sqlx::Error` type entirely; callers that want to match on `sqlx::Error::Database(...)` variants (to map unique-violations to `Conflict`, for example) cannot. This is the structural reason behind ADAPTER-SQ-002 and ADAPTER-SQ-031: every error path returns `Infrastructure` because the typed error chain is broken at the conversion boundary.
- **expected:** Error wrappers that preserve the underlying `sqlx::Error` (or at least its `kind()`) so adapters can pattern-match on `UniqueViolation`, `ForeignKeyViolation`, etc., and produce `Conflict` / `Validation` / `Infrastructure` accordingly.
- **evidence:** `crates/adapters/storage-sqlite/src/error.rs:29-33` `impl From<StringError> for educore_core::error::DomainError { fn from(e: StringError) -> Self { educore_core::error::DomainError::infrastructure(e) } }` — drops the structured error information.

---

### FINDING 44

- **id:** ADAPTER-SQ-044
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:55, 95` and `crates/adapters/storage-sqlite/src/outbox.rs:77, 145`
- **description:** The `event_version` column is bound as `i32` on both write and read paths with a `try_from(...,).unwrap_or(0)` clamp. The DDL declares the column as `INTEGER` (8-byte signed) which maps to `i64` in sqlx. Binding `i32` succeeds only because sqlx widens small integers; binding a value > `i32::MAX` would fail with an out-of-range error and the adapter would convert that to `Infrastructure` rather than `Validation`.
- **expected:** Bind `i64` (matching the DDL's 8-byte `INTEGER`) and surface overflow as `Validation`.
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:95` `let event_version = i32::try_from(entry.schema_version).unwrap_or(0);` and `crates/adapters/storage-sqlite/src/event_log.rs:107` `.bind(event_version)`. `migrations/engine/0000_engine_core.sqlite.sql:67, 177` declares `event_version INTEGER NOT NULL`.

---

### FINDING 45

- **id:** ADAPTER-SQ-045
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/audit_log.rs:43-67` (`AuditLogRow`)
- **description:** `AuditLogRow` declares `actor_type: String` and `source: String` (lines 50, 66) as the only string columns on the read path. The port trait's `AuditLogEntry` does not carry these fields (per the doc-vs-code drift at ADAPTER-SQ-020), so they are read but discarded. The `to_entry` function explicitly hardcodes `active_status: ActiveStatus::Active` (line 85) despite the DDL having no `active_status` column.
- **expected:** Either the port trait carries these fields and the adapter round-trips them, or the SELECT avoids them. The current state is "read but drop", which signals an incomplete implementation.
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:69-93` `to_entry` constructs an `AuditLogEntry` from a row but does not populate `actor_type`, `source`, `recorded_at`, `audit_id`, `ip`, `user_agent`, `session_id`, or `command_id` — none of which the port struct carries.

---

### FINDING 46

- **id:** ADAPTER-SQ-046
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:155-167` (`watch_changes`)
- **description:** `watch_changes` is documented in the port contract as the entry point for the sync engine ("MySQL / SQLite: poll the outbox table on a timer"). The SQLite implementation returns `DomainError::not_supported(...)` (line 164-166), but the Phase 1 handoff claims it is a "Phase 1 stub. A future PR will poll the outbox on a timer (per `docs/ports/storage.md` 'MySQL / SQLite: poll the outbox table on a timer')." The port trait already has a default implementation that returns `NotSupported`, so the explicit override is redundant. The override is also misleading because it overrides a perfectly good default with an identical-typed error string.
- **expected:** Either delete the override and rely on the trait default, or implement the polling loop. The current code duplicates the default's behaviour without adding value.
- **evidence:** `crates/adapters/storage-sqlite/src/storage.rs:155-167` and the default at `crates/infra/storage/src/port.rs:115-120` `async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> { let _ = filter; Err(educore_core::error::DomainError::not_supported("StorageAdapter::watch_changes is not supported by this adapter")) }`.

---

### FINDING 47

- **id:** ADAPTER-SQ-047
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/storage.rs:1-38` (`lib.rs` doc-comment)
- **description:** The crate's `lib.rs` documents the adapter against `ADR-017` (`Multi-writer scenarios are out-of-scope: SQLite is the engine's embedded / offline mode (per ADR-017).`). ADR-017 in `docs/decisions/ADR-017-SurrealDBFirst.md` is about the SurrealDB-first strategy, not about SQLite's single-writer limitation. The cross-reference is wrong: there is no `ADR-017-SurrealDBFirst` that documents SQLite's single-writer deployment model. The doc-link at line 15 is broken.
- **expected:** A correct ADR reference (or no ADR reference at all).
- **evidence:** `crates/adapters/storage-sqlite/src/lib.rs:15` `//! [`ADR-017`]: ../../docs/decisions/ADR-017-SurrealDBFirst.md` — the link path resolves to `docs/decisions/ADR-017-SurrealDBFirst.md`, which is unrelated to SQLite's single-writer constraints.

---

### FINDING 48

- **id:** ADAPTER-SQ-048
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/audit_log.rs:85` (`AuditLogRow::to_entry`)
- **description:** `to_entry` always returns `active_status: ActiveStatus::Active` despite the audit log being explicitly an append-only, write-once store where rows should never be soft-deleted. The hardcoded `Active` is correct in spirit but the lack of a schema column means there is no physical way to record the row's "retired" state if a future auditor flag demands it. The port contract at `crates/infra/storage/src/audit.rs:94-96` documents that "Audit rows are never hard-deleted; this is `Retired` when an auditor marks a row as superseded."
- **expected:** An `active_status` column on `audit_log` with a `CHECK (active_status IN (0, 1))` constraint, matching the spec.
- **evidence:** `crates/adapters/storage-sqlite/src/audit_log.rs:85` `active_status: ActiveStatus::Active,` (hardcoded). `migrations/engine/0000_engine_core.sqlite.sql:107-129` — the `audit_log` table does not have an `active_status` column.

---

### FINDING 49

- **id:** ADAPTER-SQ-049
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/event_log.rs:181-188`
- **description:** The `since` and `until` filters are applied to `recorded_at` (the persistence time), not `occurred_at` (the event-time). The handoff claims this matches the spec, but the spec at `docs/schemas/event-schema.md:6` says consumers query by `(school_id, [event_type], since, until)` where `since/until` are event-time. The SQLite adapter's filtering on `recorded_at` returns events in the order they were persisted, which can be a few seconds to a few minutes (or hours, if a relay backlog exists) after they occurred. Analytics consumers expecting "events from the last hour" will silently miss late-arriving events.
- **expected:** Either filter on `occurred_at` (matching the spec) or document the drift explicitly.
- **evidence:** `crates/adapters/storage-sqlite/src/event_log.rs:181-188` `if let Some(since) = filter.since { qb.push(" AND recorded_at >= ").push_bind(since.as_datetime()); } if let Some(until) = filter.until { qb.push(" AND recorded_at < ").push_bind(until.as_datetime()); }`. The index that backs this query (`idx_event_log_school_time ON event_log(school_id, occurred_at)` at `migrations/engine/0000_engine_core.sqlite.sql:190-191`) is on `occurred_at`, so the `recorded_at` filter will not use the index — every event-log query is a sequential scan over the school's events.

---

### FINDING 50

- **id:** ADAPTER-SQ-050
- **area:** adapters
- **severity:** Low
- **location:** `crates/adapters/storage-sqlite/src/transaction.rs:75-81` (`Debug` for `SqliteTransaction`)
- **description:** The `Debug` impl for `SqliteTransaction` prints the school field from `self.outbox.school()` but the struct has four sub-port handles, each with its own `school` field. The `Debug` impl is consistent today only because every constructor path passes the same `school` to every sub-port (`SqliteTransaction::new` lines 86-91). A future change that introduces a sub-port with a different school scope (e.g. cross-tenant audit reads) will silently produce a misleading `Debug` output.
- **expected:** Either iterate every sub-port's school, or document the invariant.
- **evidence:** `crates/adapters/storage-sqlite/src/transaction.rs:75-81` `impl fmt::Debug for SqliteTransaction { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("SqliteTransaction").field("school", &self.outbox.school()).finish_non_exhaustive() } }`.

---

### END FINDINGS

Total Findings: 50
