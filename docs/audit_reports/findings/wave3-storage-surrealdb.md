# Wave 3 — SurrealDB Storage Adapter Audit Report

**Scope:** `crates/adapters/storage-surrealdb/`, port contract
`docs/ports/storage.md`, canonical DDL
`migrations/engine/0000_engine_core.surreal.surql`, phase handoff
`docs/handoff/PHASE-0-HANDOFF.md`, dialect spec
`docs/schemas/sql-dialects/surrealdb.md`, port trait at
`crates/infra/storage/src/port.rs` + `transaction.rs`,
ADR `docs/decisions/ADR-017-SurrealDBFirst.md`.

**Audit date:** 2026-06-23.

**Total findings:** 38

---

### FINDING 1

- **id:** ADAPTER-SR-001
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)
- **description:** The adapter exposes a `migrate()` method on `StorageAdapter`, but every consumer-facing doc (`AGENTS.md:544, 561`, `README.md:173`, `docs/schemas/sql-dialects/README.md:193-198`, `docs/schemas/sql-dialects/surrealdb.md:9-10, 505-511, 752`, `docs/build-plan.md:119, 175-186`, `docs/architecture.md:322`, `migrations/engine/README.md:11`, `CONTRIBUTING.md:502`) refers to the runtime entry point as `storage.create_schema().await`. The trait method is named `migrate()` (per `docs/ports/storage.md:21, 174`, which is the *consumer migration runner*, not the engine's schema emission). The SurrealDB adapter's `migrate()` is in fact performing the engine's `create_schema()` work (executing the cross-cutting DDL); the consumer-facing API name does not exist on the trait.
- **expected:** `docs/build-plan.md:175-179` lists the trait surface as `("create_schema", "apply_command", "query", "begin_tx", ...)`; `docs/architecture.md:322` says the schema is emitted "via `storage.create_schema().await`". A `create_schema()` method on `StorageAdapter` is the contract.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:88`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport> {
  ```
  And `crates/infra/storage/src/port.rs:44`:
  ```rust
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  No `create_schema` method exists in the entire crate (`grep -n "fn create_schema" crates/adapters/storage-surrealdb/` returns no results).

---

### FINDING 2

- **id:** ADAPTER-SR-002
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)
- **description:** The `migrate()` implementation only loads `migrations/engine/0000_engine_core.surreal.surql` (6 engine cross-cutting tables) and executes the DDL once. It does not walk any macro-emitted AST to emit the ~310 domain tables the engine claims to ship, and it does not honour `docs/schemas/sql-dialects/surrealdb.md`'s `SurrealStorageAdapter::create_<table>_ddl()` per-aggregate contract at all. The dialect spec is explicit: "For the ~310 domain tables, the adapter walks the macro-emitted AST and renders each table's DDL string at runtime." None of the ~310 domain tables are emitted.
- **expected:** "The engine emits DDL **at schema-creation time** (once per process lifetime, via `storage.create_schema().await`) from a typed macro AST" (`docs/architecture.md:321-324`); "the adapter walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL" (`docs/build-plan.md:178-181`).
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:26-27` `const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.surreal.surql");` and `crates/adapters/storage-surrealdb/src/storage.rs:97-101` only executes `SCHEMA_SQL` via `self.conn.db().query(SCHEMA_SQL).await`. The crate has no `create_schema()`, no AST walk, no `EntityDescriptor` traversal, and no domain-table emission code anywhere under `src/`.

---

### FINDING 3

- **id:** ADAPTER-SR-003
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/transaction.rs:86-97` (`commit` / `rollback`)
- **description:** The `commit` and `rollback` impls are no-ops that only flip the `done` / `rolled_back` atomic flag. The SurrealDB SDK does not expose explicit transaction control (per the file's own module-level doc at lines 1-8), so the entire transactional unit-of-work contract is non-functional. The file's own comment explicitly states: "A future PR will use the SurrealDB 3.x transaction API for true atomicity." In production this means a caller that invokes `tx.outbox().append(...)` inside a transaction and then `tx.commit()` will see the outbox row durable immediately and the "commit" step is purely cosmetic; a caller that wants to roll back has no way to undo writes.
- **expected:** "Commits the transaction. All outbox appends, aggregate mutations, audit log writes, idempotency records, and event log rows become durable." (`crates/infra/storage/src/transaction.rs:35-37`); and the equivalent for `rollback`.
- **evidence:** `crates/adapters/storage-surrealdb/src/transaction.rs:87-97` `async fn commit(self: Box<Self>) -> Result<()> { self.done.store(true, std::sync::atomic::Ordering::SeqCst); Ok(()) }` and `async fn rollback(self: Box<Self>) -> Result<()> { self.rolled_back.store(true, ...); self.done.store(true, ...); Ok(()) }` — neither call interacts with the database.

---

### FINDING 4

- **id:** ADAPTER-SR-004
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:129-140` (`watch_changes`)
- **description:** `watch_changes` returns an empty `ChangeStream` (`futures::stream::empty()`) instead of `DomainError::NotSupported`. The default-impl contract per `docs/ports/storage.md:112-116` and `crates/infra/storage/src/port.rs:115-120` is: sync primitives return `NotSupported`; "the sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup — not silently at runtime — so consumers see the configuration problem immediately." The SurrealDB implementation reports success and returns an empty stream, masking the configuration problem and letting the sync engine start up against an adapter that is doing nothing. ADR-017 lists SurrealDB `LIVE SELECT` as the supported implementation path: "SurrealDB supports all four natively."
- **expected:** "Default impls on the trait return `DomainError::NotSupported('sync primitives require the sync feature and a sync-capable adapter'). The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup" (`docs/ports/storage.md:112-116`). And per `ADR-017-SurrealDBFirst.md` § "Parity surface", SurrealDB `watch_changes` is `✓ (SurrealDB live queries)`.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:129-140`:
  ```rust
  async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
      // Phase 0 stub. A future PR will use SurrealDB's
      // `LIVE SELECT` to drive a real change feed.
      if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
          return Err(DomainError::conflict(
              "StorageAdapter::watch_changes called on a closed adapter",
          ));
      }
      let s = futures::stream::empty::<std::result::Result<ChangeEvent, DomainError>>();
      let pinned = Box::pin(s);
      Ok(ChangeStream { inner: pinned })
  }
  ```

---

### FINDING 5

- **id:** ADAPTER-SR-005
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:151-171` (`cursor_for` / `advance_cursor`)
- **description:** Both `cursor_for` and `advance_cursor` silently override the trait default of `DomainError::NotSupported`. `cursor_for` returns `Ok(VersionCursor(0))` (hard-coded) and `advance_cursor` returns `Ok(())` (no-op). The default-impl contract is the sync engine's safety net: non-sync adapters must fail loudly at startup. The SurrealDB implementation reports success, masking configuration problems and letting the sync engine start up against an adapter that is actually doing nothing. ADR-017 lists `cursor_for` / `advance_cursor` as `✓` for SurrealDB.
- **expected:** Same as ADAPTER-SR-004: "Default impls on the trait return `DomainError::NotSupported`. … The sync engine, when it tries to subscribe on a non-sync adapter, fails loudly at startup" (`docs/ports/storage.md:112-116`). And per `ADR-017-SurrealDBFirst.md` § "Parity surface", SurrealDB supports both.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:151-161` `async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> { ... Ok(VersionCursor(0)) }` and `crates/adapters/storage-surrealdb/src/storage.rs:163-171` `async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> { ... Ok(()) }` — both return success instead of `DomainError::not_supported(...)`.

---

### FINDING 6

- **id:** ADAPTER-SR-006
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:75-86, 88-108, 110-122, 124-127, 129-140, 151-161, 163-171`
- **description:** Every `StorageAdapter` method (`begin`, `migrate`, `ping`, `close` (implicitly), `watch_changes`, `cursor_for`, `advance_cursor`) returns `DomainError::Conflict` when the adapter is closed. The port contract mandates `DomainError::Infrastructure`. Returning `Conflict` is structurally wrong (closing the connection is not a state conflict) and breaks error-handling callers that match on the `Infrastructure` variant to surface a degraded-storage alert.
- **expected:** "`close(self: Box<Self>) -> Result<()>; … After `close`, any further call returns `Err(Infrastructure)`." (`crates/infra/storage/src/port.rs:52-53` and `docs/ports/storage.md:23`).
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:79-82, 90-93, 112-115, 133-136, 153-156, 165-168` all call `DomainError::conflict("...")` instead of `DomainError::infrastructure(...)` after the `self.closed.load(SeqCst)` check.

---

### FINDING 7

- **id:** ADAPTER-SR-007
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:54-77` (`IdempotencyRow::to_record`)
- **description:** `IdempotencyRow::to_record` calls `Box::leak(self.command_type.clone().into_boxed_str())` on every read. The port struct's `command_type: &'static str` field forces this leak. In a long-running process serving many idempotency lookups the heap grows without bound — a slow but unbounded memory leak in production code. The byte pattern is `Box::leak(string) -> &'static str`; every call leaks a fresh `Box<str>` that the allocator never reclaims.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." Adapter code must not leak memory per-call.
- **evidence:** `crates/adapters/storage-surrealdb/src/idempotency.rs:69` `command_type: Box::leak(self.command_type.clone().into_boxed_str()),` (the `IdempotencyRecord` struct's `command_type: &'static str` field at `crates/infra/storage/src/idempotency.rs:31` forces this allocation).

---

### FINDING 8

- **id:** ADAPTER-SR-008
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:180-226` (`Outbox::append`)
- **description:** `Outbox::append` uses an `INSERT INTO outbox { ... }` statement and surfaces the underlying SurrealDB error (which includes a unique-constraint violation on `event_id`) as `DomainError::Infrastructure`. The port contract requires `DomainError::Conflict` on a duplicate `(school_id, event_id)`. The adapter silently downgrades a contract-mandated domain error to an infrastructure error.
- **expected:** "`Conflict` if an envelope with the same `event_id` was already appended in the same school." (`crates/infra/storage/src/outbox.rs:99-101`).
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:184-219` `self.db.query("INSERT INTO outbox { ... }").await.map_err(|e| StringError(format!("outbox append: {e}")))?;` — no match on the SurrealDB error variant to map a uniqueness violation to `DomainError::conflict(...)`.

---

### FINDING 9

- **id:** ADAPTER-SR-009
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:126-152` (`Idempotency::record`)
- **description:** `Idempotency::record` does a plain `INSERT INTO idempotency { ... }` and returns `DomainError::Infrastructure` on a unique-constraint violation. The port contract requires `DomainError::Conflict` when a record with the same composite key exists with a different outcome, and `Ok(())` only when the new row is identical. The current behaviour never returns `Conflict`; a duplicate insert is indistinguishable from a DB failure.
- **expected:** "Stores `record`. Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome. Returns `Ok(())` if the record is a no-op write (same key, same outcome hash) — the engine uses this for at-least-once delivery of retries." (`crates/infra/storage/src/idempotency.rs:94-100`).
- **evidence:** `crates/adapters/storage-surrealdb/src/idempotency.rs:127-149`:
  ```rust
  async fn record(&self, record: IdempotencyRecord) -> Result<()> {
      let row = IdempotencyRow::from_record(&record);
      let _ = self.db.query("INSERT INTO idempotency { ... }")...await
          .map_err(|e| StringError(format!("idempotency record: {e}")))?;
      Ok(())
  }
  ```
  No `match` on the SurrealDB error; no `exists()` check before insert.

---

### FINDING 10

- **id:** ADAPTER-SR-010
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:39-69` (`OutboxRow::from_envelope`)
- **description:** `OutboxRow::from_envelope` sets `recorded_at: Datetime::from(env.occurred_at.as_datetime())` — `recorded_at` is bound to the *envelope's* `occurred_at` instead of the wall-clock time of the persistence. The DDL declares `recorded_at` as the persistence time (a separate column from `occurred_at`), and the engine invariant is that `recorded_at >= occurred_at` (it captures ingestion latency between the producer and the outbox writer). Binding both to the same value obliterates that invariant.
- **expected:** "`recorded_at`: Wall-clock time of the persistence (≥ `occurred_at`)" (`crates/infra/storage/src/event_log.rs:73`) and the DDL column pair `occurred_at ... recorded_at ...` (`migrations/engine/0000_engine_core.surreal.surql:89-90`).
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:64-65`:
  ```rust
  occurred_at: Datetime::from(env.occurred_at.as_datetime()),
  recorded_at: Datetime::from(env.occurred_at.as_datetime()),  // BUG
  ```

---

### FINDING 11

- **id:** ADAPTER-SR-011
- **area:** adapters
- **severity:** Critical
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:1-180` (entire `StorageAdapter` impl)
- **description:** The `StorageAdapter` trait in `docs/ports/storage.md:17-89` enumerates ~22 per-aggregate repository methods (`students()`, `guardians()`, `classes()`, …, one per aggregate across 15 domains, ~80+ total). The actual port trait at `crates/infra/storage/src/port.rs:34-150` exposes only 5 methods (`begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`) plus 4 sync primitives — no per-aggregate repository handles. The SurrealDB adapter implements the actual trait (no repository methods), meaning **none** of the documented per-aggregate repository handles are implemented. The dialect spec promises `SurrealStorageAdapter::create_<table>_ddl()` per aggregate; no such method exists in the crate.
- **expected:** "`fn students(&self) -> Arc<dyn StudentRepository>;` and ~21 sibling methods, 'one handle per aggregate, across all 15 domains (~80+ total)'" (`docs/ports/storage.md:50`). Each adapter must translate the macro-emitted `QueryNode` AST into a SurrealDB execution plan.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:75-172` implements only `begin`, `migrate`, `ping`, `close`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. `grep -n 'students\|guardians\|classes\|sections' crates/adapters/storage-surrealdb/src/` returns no repository handle of any kind.

---

### FINDING 12

- **id:** ADAPTER-SR-012
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:88-108` (`migrate`)
- **description:** `migrate()` hard-codes `MigrationReport { statements_executed: 0, already_at_version: false, ... }`. The `statements_executed` field exists to report the actual count of statements applied (telemetry, migration-time SLOs, idempotency verification) and the adapter discards the SurrealDB query result without inspecting it. `already_at_version` is always `false`, even when re-running `migrate()` on an already-migrated database — so the report cannot be used by callers to distinguish a first run from a no-op run.
- **expected:** Per `crates/infra/storage/src/change_stream.rs:243-255`: "`statements_executed`: The number of statements executed (DDL or DML)." and "`already_at_version`: Whether the migration was a no-op (already at `version`)."
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:97-107`:
  ```rust
  self.conn.db().query(SCHEMA_SQL).await.map_err(DomainError::infrastructure)?;
  Ok(MigrationReport {
      version: SCHEMA_VERSION,
      statements_executed: 0,        // never updated
      duration: start.elapsed(),
      already_at_version: false,     // never updated
  })
  ```

---

### FINDING 13

- **id:** ADAPTER-SR-013
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:40-58` (`OutboxRow::from_envelope`) and `crates/adapters/storage-surrealdb/src/outbox.rs:74-101` (`OutboxRow::to_envelope`)
- **description:** Both `OutboxRow::from_envelope` and `OutboxRow::to_envelope` use `i32::try_from(env.schema_version).unwrap_or(0)` and `u32::try_from(self.event_version).unwrap_or(0)` to silently clamp `schema_version` on overflow or negative-value round-trip. The engine's invariant is that `schema_version` is a small positive integer, but the silent fallback to `0` discards data without surfacing the error — a caller that has produced a malformed envelope will not see `Err(Validation)`, and downstream consumers will silently treat the event as schema v0, which may have an unrelated payload shape.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `as` casts that truncate or lose data. Use `TryFrom` / `TryInto` with proper error handling." and "All public APIs return `Result` for fallible operations."
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:57` `event_version: i32::try_from(env.schema_version).unwrap_or(0),` and `:91` `schema_version: u32::try_from(self.event_version).unwrap_or(0),`.

---

### FINDING 14

- **id:** ADAPTER-SR-014
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:60-62, 84-88` (`EventRow` round-trip)
- **description:** `EventRow::to_entry` returns `ActiveStatus::Retired` for every value of `self.active_status` that is not the literal string `"active"`. The `from_entry` writes `entry.active_status.to_string()` which for `ActiveStatus::Active` is `"Active"` (capital A, per the Display impl) — but the read side's match arm `"active" => ActiveStatus::Active` is lower-case. A round-trip therefore corrupts `active_status` from `Active` to `Retired` on every read.
- **expected:** Per `crates/infra/storage/src/event_log.rs:81-82`: `pub active_status: ActiveStatus,`. A write-then-read must round-trip the value exactly.
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:54` `active_status: entry.active_status.to_string(),` (writer serialises as `Display`, capitalised). `:70-73`:
  ```rust
  let active_status = match self.active_status.as_str() {
      "active" => ActiveStatus::Active,
      _ => ActiveStatus::Retired,
  };
  ```
  (reader compares against lower-case). Compare `audit.rs:96-99` which has the same lower-case-vs-upper-case mismatch in the audit row's `active_status`.

---

### FINDING 15

- **id:** ADAPTER-SR-015
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/audit.rs:85-114` (`AuditRow::to_entry`)
- **description:** `AuditRow::to_entry` always returns `ActiveStatus::Retired` for any `active_status` string that is not the literal lower-case `"active"`. The writer side at `AuditRow::from_entry` (line 71) sets `active_status: entry.active_status.to_string()` — `ActiveStatus`'s `Display` impl returns `"Active"` (capital A, per `crates/infra/core/src/value_objects.rs`'s standard pattern). Every read therefore returns `Retired` for an `Active` audit row, breaking the soft-delete semantics that the audit-log is designed to enforce.
- **expected:** A write-then-read round-trip on `active_status` must preserve the value exactly.
- **evidence:** `crates/adapters/storage-surrealdb/src/audit.rs:71` `active_status: entry.active_status.to_string(),` (writer) and `crates/adapters/storage-surrealdb/src/audit.rs:96-99` (reader with lower-case match arm).

---

### FINDING 16

- **id:** ADAPTER-SR-016
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:182-220` (`Outbox::append`)
- **description:** `Outbox::append` issues an `INSERT INTO outbox { ... enqueued_at: time::now(), published_at: NONE, attempts: 0, last_error: NONE }` — `enqueued_at` is set by SurrealDB's server-side `time::now()` while every other timestamp in the adapter is set by the application's `chrono::Utc::now()`. The outbox DDL allows either: the spec does not mandate a side. But mixing the two clock sources in the same database means a single event row's `occurred_at` (application) and `enqueued_at` (server) can disagree on the order, breaking the engine's `recorded_at >= occurred_at` invariant and any post-mortem analysis of outbox-drain latency.
- **expected:** Per `crates/infra/storage/src/outbox.rs:104-108`: "The order is FIFO by append time within a school." The clock source for `enqueued_at` must be the same as the application clock used for `occurred_at` and `recorded_at` to make the FIFO ordering meaningful.
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:200` `enqueued_at: time::now(),` is set server-side by the SurrealDB engine, while `:64-65` `occurred_at` / `recorded_at` are set from the application-supplied `env.occurred_at`. The `enqueued_at` should be the adapter's `Utc::now()` converted to `Datetime` and bound as a parameter.

---

### FINDING 17

- **id:** ADAPTER-SR-017
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:257-276` (`Outbox::mark_published`)
- **description:** `Outbox::mark_published` uses server-side `time::now()` for the `published_at` column (`SET published_at = time::now()`) instead of the application clock. Same clock-mixing issue as ADAPTER-SR-016. Additionally, the helper is unaware of a `last_error` / `attempts` increment that the relay would normally perform when an envelope fails to publish — the implementation only updates `published_at`, never `attempts` or `last_error`. The port trait's `mark_published` is the only feedback path the relay has, and the column semantics in the DDL (`attempts`, `last_error`) are designed to be incremented on this path.
- **expected:** The `outbox` DDL columns `attempts INT ASSERT $value != NONE AND $value >= 0 VALUE 0` and `last_error option<string>` (`migrations/engine/0000_engine_core.surreal.surql:94-95`) imply the adapter increments `attempts` on every `mark_published` call (whether success or failure). The current impl never touches either column.
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:265-268`:
  ```rust
  "UPDATE outbox SET published_at = time::now() \
   WHERE event_id IN $ids",
  ```
  No `attempts = attempts + 1`, no `last_error` handling.

---

### FINDING 18

- **id:** ADAPTER-SR-018
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:218-273` (`EventLog::count`)
- **description:** `EventLog::count` issues `SELECT count() AS n FROM event_log WHERE school_id = $school AND {type_filter}{since_clause}{until_clause} GROUP ALL` and returns the `n` of the first row. The `aggregate_id` filter that the `read` method applies (`event_log.rs:189-192` `format!(" AND aggregate_id = SurrealUuid::from('{}')", a)`) is missing from `count`. Two semantically equivalent API methods have different filter coverage: a caller that sets `filter.aggregate_id = Some(uuid)` and then calls `count()` will get a count that is **larger** than the count of rows `read()` would return, and downstream consumers (cursor sizing, analytics) will be silently wrong.
- **expected:** `count()` must apply exactly the same filters as `read()` (minus `limit`). Per the port doc: "Returns the count of events for `school_id` matching `filter` (ignoring `limit`)." (`crates/infra/storage/src/event_log.rs:156-158`).
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:251-255`:
  ```rust
  "SELECT count() AS n FROM event_log \
   WHERE school_id = $school AND {type_filter}{since_clause}{until_clause} \
   GROUP ALL"
  ```
  Compared with `:193-201` (`read`):
  ```rust
  "SELECT event_id, ... FROM event_log \
   WHERE school_id = $school AND {type_filter}{since_clause}{until_clause}{agg_clause} \
   ORDER BY recorded_at ASC \
   LIMIT $limit"
  ```
  The `agg_clause` is missing from the `count` query.

---

### FINDING 19

- **id:** ADAPTER-SR-019
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:156-216` (`EventLog::read`); 218-273 (`EventLog::count`)
- **description:** Both `read` and `count` build a SurrealQL query by string-formatting the `event_types` filter via `format!("'{t}'")`. A caller that supplies an `event_type` containing a single-quote character (e.g. `"academic.student's.admitted"`) breaks out of the string literal and injects SurrealQL. The same applies to the `since_clause` / `until_clause` / `agg_clause` constructions, which `format!` user-supplied UUIDs and timestamps directly into the query string. The port struct's `event_type: String` is user-controllable, so this is an injection vector that reaches the storage layer.
- **expected:** Per `AGENTS.md` § "Type Safety": "No `serde_json::Value` in domain code. Use typed wrappers." and the port trait's `EventLogFilter::event_types: Vec<String>` is documented as `String` (not `&'static str`) "so the type can be deserialised from JSON / MessagePack" (`crates/infra/storage/src/event_log.rs:90-94`); this means user input can flow into it. The query must use parameterised binds for all user-controllable values.
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:163-167`:
  ```rust
  let types = filter.event_types.iter().map(|t| format!("'{t}'")).collect::<Vec<_>>().join(", ");
  format!("event_type IN [{types}]")
  ```
  and `:189-192`:
  ```rust
  .map(|a| format!(" AND aggregate_id = SurrealUuid::from('{}')", a))
  ```
  and `:172-178, 181-188` (the `since_clause` / `until_clause` format RFC-3339 strings inline).

---

### FINDING 20

- **id:** ADAPTER-SR-020
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:100-124` (`Idempotency::lookup`)
- **description:** `Idempotency::lookup` ignores `filter.aggregate_id` semantics (the lookup doesn't filter on it because the trait has no `aggregate_id` slot, OK) but, more critically, does not honour the "exists with a different outcome" semantic at read time. The trait contract says `lookup` returns the prior outcome; the dispatcher then re-checks `affected_aggregate_ids` to detect "same idempotency key, but different target" misuse. The adapter's `IdempotencyRow` struct **does** carry `affected_aggregate_ids: Option<Vec<SurrealUuid>>` (line 32) and `to_record` returns it (lines 62-66) — but the lookup query (lines 105-110) only SELECTs the 7 columns needed and the deserialised row's `affected_aggregate_ids` is then re-serialised. The round-trip preserves the column, but the `outcome` is not re-hashed for "same key, different outcome" — meaning the engine cannot tell a "same key, same outcome" replay from a "same key, different outcome" misuse without re-comparing in application code (which the spec says the storage adapter should surface as `Conflict`).
- **expected:** "`record`: Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome." (`crates/infra/storage/src/idempotency.rs:94-100`). The lookup must return enough information to make that decision; the current round-trip returns `Option<IdempotencyRecord>` but `record` doesn't pre-check.
- **evidence:** `crates/adapters/storage-surrealdb/src/idempotency.rs:126-151` `record` does a plain `INSERT INTO idempotency { ... }` with no pre-check on whether the composite key already exists. The SurrealDB unique index `idx_idempotency_pk ... UNIQUE` on `(school_id, command_type, idempotency_key)` will cause a server error, but the error is mapped to `Infrastructure` (Finding 9), not `Conflict`.

---

### FINDING 21

- **id:** ADAPTER-SR-021
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:80-83` (`SurrealIdempotency` struct)
- **description:** The `SurrealIdempotency` struct carries only a `db: Db` field, no `school: SchoolId`. The `outbox`, `audit`, and `event` sub-port structs in the same crate (`outbox.rs:160-162`, `audit.rs:137-139`, `event_log.rs:95-97`) all carry a `school: SchoolId`. The omission is asymmetric and means the idempotency handle cannot enforce a per-school write boundary at the application layer — a `record()` call from a `SurrealIdempotency` handle bound to adapter A can write an idempotency row for school B. Compare the `Outbox` handle (line 161) which carries `school: SchoolId` and uses it in the `pending()` filter (line 229) to scope the read.
- **expected:** The `Idempotency` sub-port is `tenant-scoped`: every record is keyed by `(school_id, command_type, idempotency_key)` and every read must be `school_id`-filtered. The handle should carry `school: SchoolId` to make this explicit and to allow an audit/assertion in `record()` that the record's `school_id` matches the handle's `school`.
- **evidence:** `crates/adapters/storage-surrealdb/src/idempotency.rs:80-83`:
  ```rust
  pub struct SurrealIdempotency {
      pub(crate) db: Db,
  }
  ```
  Compare `crates/adapters/storage-surrealdb/src/outbox.rs:159-162`:
  ```rust
  pub struct SurrealOutbox {
      pub(crate) db: Db,
      pub(crate) school: SchoolId,
  }
  ```

---

### FINDING 22

- **id:** ADAPTER-SR-022
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)
- **description:** `SurrealConnection::in_memory` is the only connection constructor on `SurrealConnection`. The `Cargo.toml` declares `surrealdb = { workspace = true }` and the connection module comment says "Phase 0 supports the in-memory backend (`Mem`); the RocksDB / TiKV / HTTP backends land in a later phase." The dialect spec (`docs/schemas/sql-dialects/surrealdb.md:51-72, 825-845`) is explicit that the adapter must target `rocksdb` (production single-process) as well as `memory` (tests) and that the embed pattern is `Surreal::new::<RocksDb>("./data/educore.db")`. There is no `RocksDb` constructor, no `encryption_key` plumbing, and no way for a consumer to use SurrealDB as a durable single-process database. ADR-017's rationale ("Single-binary deployment. SurrealDB embedded means one process to ship") is not realised by the shipped code.
- **expected:** `docs/schemas/sql-dialects/surrealdb.md:827-838`:
  ```rust
  let db = Surreal::new::<RocksDb>("./data/educore.db").await?;
  ```
  And: "The engine's adapter exposes an `encryption_key` parameter on the connection builder." (`docs/schemas/sql-dialects/surrealdb.md:856-866`).
- **evidence:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` exposes only `in_memory(school: SchoolId)`. `crates/adapters/storage-surrealdb/Cargo.toml:20` declares `surrealdb = { workspace = true }` but the only `use` in `connection.rs:8-9` is `engine::local::{Db as LocalDb, Mem}`. No `RocksDb` import, no `File` import, no `WsClient` / `HttpClient` for server mode.

---

### FINDING 23

- **id:** ADAPTER-SR-023
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)
- **description:** `in_memory` opens a *new* in-memory SurrealDB on every call. Two calls to `SurrealConnection::in_memory(school_a)` and `SurrealConnection::in_memory(school_b)` produce two independent `Db` instances (each scoped to one school). The unit test at `audit.rs:393-421` (`read_for_target_isolates_by_school`) depends on this fact — and the audit.rs module doc explicitly states (lines 250-257): "the storage layer does not itself enforce it (the engine's `TenantContext` layer is the canonical gate per `docs/schemas/tenancy-schema.md`)." In other words, the in-memory backend is per-school-isolated by virtue of being per-process; the production storage adapter will share a single `Db` across all schools (per the architecture doc, single-binary / per-school single-tenant), and the in-memory test's "isolation by separate process" does not generalise to a shared `Db`.
- **expected:** Per `docs/ports/storage.md:140-150`: "The storage adapter is responsible for enforcing tenant isolation. The engine always passes a `SchoolId` filter; the adapter MUST add a `school_id = $1` predicate to every read query." Defense in depth: the in-memory test's "two `Db` instances" trick is not the production architecture.
- **evidence:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` constructs a fresh `Surreal::new::<Mem>(())` per call. The audit module's doc-comment at `crates/adapters/storage-surrealdb/src/audit.rs:250-257` admits the test's "two `Db` instances" trick is the only reason the isolation test passes.

---

### FINDING 24

- **id:** ADAPTER-SR-024
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/connection.rs:23-31` (`SurrealConnection` struct) + `crates/adapters/storage-surrealdb/src/storage.rs:1-180` (no `$auth` setup)
- **description:** The dialect spec mandates per-table `PERMISSIONS` clauses with a session-scoped `$auth.school_id` predicate (see `docs/schemas/sql-dialects/surrealdb.md:231-262`) and a "second line of defense" via `PERMISSIONS NONE` on the engine-internal tables. The migration DDL (`migrations/engine/0000_engine_core.surreal.surql:71-260`) does NOT emit any `PERMISSIONS` clause — every table is `DEFINE TABLE <name> SCHEMAFULL COMMENT "..."` with no permission scope. The connection does not set `$auth.school_id` on connect (`connection.rs:50-62` has only `use_ns("educore").use_db("engine")`). The result: a consumer's session can read every school's `outbox` rows because the DB enforces no per-school permission.
- **expected:** `docs/schemas/sql-dialects/surrealdb.md:238-244`:
  ```sql
  DEFINE TABLE academic_students SCHEMAFUL
    PERMISSIONS
      FOR SELECT WHERE school_id = $auth.school_id OR $auth.bypass = true
      ...
  ```
  And: "`PERMISSIONS NONE` on `outbox` is correct — the engine writes to it from the application layer, never from user sessions" (`docs/schemas/sql-dialects/surrealdb.md:471-474`).
- **evidence:** `migrations/engine/0000_engine_core.surreal.surql:71` `DEFINE TABLE outbox SCHEMAFULL\n    COMMENT "...";` (no `PERMISSIONS NONE`); `:127` (audit_log), `:164` (idempotency), `:193` (event_log), `:226` (schema_registry), `:252` (system_user) all lack `PERMISSIONS`. `crates/adapters/storage-surrealdb/src/connection.rs:50-62` has no `LET $auth = { school_id: ... }` setup.

---

### FINDING 25

- **id:** ADAPTER-SR-025
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:194-201` (`EventLog::read` query)
- **description:** The `read` query uses dynamic string formatting to compose the `WHERE` clause and binds only the `school` and `limit` parameters. The `event_types` filter, `since` / `until` timestamps, and `aggregate_id` are spliced into the SQL string via `format!` (per Finding 19). This bypasses the SurrealDB driver's parameterised bind path for four different user-controllable filter values, defeating the driver's type checking and the database's `DEFINE FIELD ... ASSERT` invariants. The driver has no way to validate the spliced values before they reach the parser.
- **expected:** The dialect spec's `DEFINE FIELD event_type ... ASSERT $value != NONE AND string::len($value) <= 191` (line 75 of the .surql) is meant to be enforced at the storage layer on every bind. Bypassing the bind path with string formatting defeats the assertion.
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:193-201` constructs the entire query with `format!` and only binds `school` (line 205) and `limit` (line 206).

---

### FINDING 26

- **id:** ADAPTER-SR-026
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:176-179` (`const _` blocks)
- **description:** The `storage.rs` file ends with three `const _: ... = ...` blocks (`Arc<()>`, `Duration`, `fn() = || { ... }`) whose stated purpose is "Suppress unused-import warning for `Arc` and `Duration` in this Phase 0 stub; they're reserved for the full impl." This is a code smell: the imports (`Arc`, `Duration`, `futures::StreamExt`) are dead, and the suppression blocks violate `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue."
- **expected:** Per `AGENTS.md` § "Type Safety", delete the unused imports and the suppression blocks. The `Arc` and `Duration` are not used in this file; remove the imports.
- **evidence:** `crates/adapters/storage-surrealdb/src/storage.rs:174-179`:
  ```rust
  // Suppress unused-import warning for `Arc` and `Duration`
  // in this Phase 0 stub; they're reserved for the full impl.
  const _: Option<Arc<()>> = None;
  const _: Option<Duration> = None;
  const _: fn() = || {
      std::mem::drop(futures::stream::empty::<()>().next());
  };
  ```

---

### FINDING 27

- **id:** ADAPTER-SR-027
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:112-144` (`parse_*` helpers, all `#[allow(dead_code)]`)
- **description:** The `outbox.rs` file declares five helper functions (`parse_event_id`, `parse_school_id_opt`, `parse_user_id`, `parse_correlation_id`, `parse_uuid`) all marked `#[allow(dead_code)]`. `grep -rn 'parse_event_id\|parse_school_id_opt\|parse_user_id\|parse_correlation_id' crates/adapters/storage-surrealdb/src/` shows the functions are only defined, never called. The struct field mapping (lines 41-69 / 74-101) uses `SurrealUuid::from(env.event_id.as_uuid())` directly, not the string-based parsers. This is dead code, violating `AGENTS.md` § "Type Safety".
- **expected:** Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue." Remove the five `parse_*` functions and the `parse_uuid` helper; the `SurrealUuid::from(uuid::Uuid)` direct path is the only path used.
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:112-144`:
  ```rust
  #[allow(dead_code)]
  fn parse_event_id(s: &str) -> std::result::Result<EventId, StringError> { ... }
  #[allow(dead_code)]
  fn parse_school_id_opt(s: Option<&str>) -> ... { ... }
  // ... three more
  ```
  The functions are never called in the file or in any other file under `crates/adapters/storage-surrealdb/`.

---

### FINDING 28

- **id:** ADAPTER-SR-028
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/stubs.rs:1-8` (entire file)
- **description:** `stubs.rs` is a placeholder file with only the doc comment and `#![allow(dead_code)]`. The doc says "All sub-port impls (AuditLog, EventLog, Idempotency) are now real implementations in their dedicated modules. This file remains as a marker for the wire-up completed by A'.1; the module is intentionally empty." An empty module exported from `lib.rs` (`crates/adapters/storage-surrealdb/src/lib.rs:22` `pub mod stubs;`) is dead code that the workspace lints should have caught. The module is referenced only by `audit.rs:131-134`'s comment ("`crate::stubs::SurrealAuditLog`") which is now misleading because no such type exists in the module.
- **expected:** Per `AGENTS.md` § "Type Safety": "Delete unused code, wire it in, or open a follow-up issue." Remove `pub mod stubs;` from `lib.rs` and delete the file.
- **evidence:** `crates/adapters/storage-surrealdb/src/stubs.rs:1-8`:
  ```rust
  //! SurrealDB-backed sub-port placeholders.
  //!
  //! All sub-port impls (AuditLog, EventLog, Idempotency) are now
  //! real implementations in their dedicated modules. This file
  //! remains as a marker for the wire-up completed by A'.1; the
  //! module is intentionally empty.
  #![allow(dead_code)]
  ```

---

### FINDING 29

- **id:** ADAPTER-SR-029
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:41-46` (`OutboxRow::from_envelope`)
- **description:** `OutboxRow::from_envelope` calls `serde_json::from_slice(&env.payload).unwrap_or_else(|_| serde_json::Value::String(String::from_utf8_lossy(&env.payload).into_owned()))` to convert the payload bytes to a `serde_json::Value` for the `payload` column. The round-trip is therefore lossy: a payload that fails to parse as JSON is stored as a JSON-stringified UTF-8 lossy version of the original bytes. The downstream `payload_to_bytes` (lines 149-155) re-serialises the JSON-Value to a `String` and wraps it in `Bytes`, so a non-JSON payload round-trips as `"<original utf-8 bytes>"` (a JSON string literal). The semantic of `Outbox::pending` therefore silently changes: a caller reading back an outbox row whose payload was a binary blob or invalid UTF-8 receives a string-wrapped version that loses the original byte boundaries.
- **expected:** The DDL declares `payload` as `TYPE object` (`migrations/engine/0000_engine_core.surreal.surql:91`) and the port contract allows any serialised payload. The round-trip should be byte-exact or the lossy behaviour should be documented and gated.
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:42-45`:
  ```rust
  let payload_value: serde_json::Value =
      serde_json::from_slice(&env.payload).unwrap_or_else(|_| {
          serde_json::Value::String(String::from_utf8_lossy(&env.payload).into_owned())
      });
  ```

---

### FINDING 30

- **id:** ADAPTER-SR-030
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/idempotency.rs:126-152` (`Idempotency::record`)
- **description:** `Idempotency::record` discards the SurrealDB query result: `let _ = self.db.query("INSERT INTO idempotency { ... }")...await?;`. The query response is checked only for the error case; the success case is treated as a unit. The SurrealDB driver can return per-statement errors via the response's `take(N)` API; the `let _` discards that path. A statement that fails server-side (e.g. a `DEFINE FIELD` `ASSERT` violation, an `affected_aggregate_ids` type mismatch) is silently treated as success.
- **expected:** Pull the typed result at position 0 to surface server-side errors, matching the pattern used in `outbox.rs:222-225` (`Outbox::append`) which does `let _: Vec<OutboxRow> = response.take(0).map_err(...)?;`. The same pattern is missing here.
- **evidence:** `crates/adapters/storage-surrealdb/src/idempotency.rs:128-149`:
  ```rust
  let _ = self
      .db
      .query("INSERT INTO idempotency { ... }")...await
      .map_err(|e| StringError(format!("idempotency record: {e}")))?;
  Ok(())
  ```

---

### FINDING 31

- **id:** ADAPTER-SR-031
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:117-154` (`EventLog::append`)
- **description:** `EventLog::append` discards the SurrealDB query result: `let _ = self.db.query("INSERT INTO event_log { ... }")...await?;`. A duplicate-`event_id` insert is silently treated as success; the unique index `idx_event_log_event_id ... UNIQUE` on `event_log.event_id` (`migrations/engine/0000_engine_core.surreal.surql:210`) will produce a server-side error, but it is mapped to `Infrastructure` (Finding 9's pattern, generalised here) and the success path discards any per-statement server warning.
- **expected:** `let _: Vec<EventRow> = response.take(0).map_err(...)?;` pattern from `outbox.rs:222-225`. And the duplicate must be mapped to `Conflict` per Finding 9.
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:119-152`:
  ```rust
  let _ = self
      .db
      .query("INSERT INTO event_log { ... }")...await
      .map_err(|e| StringError(format!("event_log append: {e}")))?;
  Ok(())
  ```

---

### FINDING 32

- **id:** ADAPTER-SR-032
- **area:** adapters
- **severity:** High
- **location:** `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs:1-58` (entire file)
- **description:** The test suite contains exactly one end-to-end test (`outbox_append_and_pending_round_trip`) covering only the outbox sub-port. No tests exist for: `audit_log.append` / `audit_log.read_for_target`, `event_log.append` / `event_log.read` / `event_log.count`, `idempotency.lookup` / `idempotency.record` / `idempotency.purge_older_than`, `migrate()` idempotency, `cursor_for` / `advance_cursor` return-value verification, `ping()`, `close()` lifecycle, tenant-isolation enforcement across a single `Db` instance, SQL-injection attempts on `event_log.read`, double-commit / double-rollback, or any round-trip across the `SurrealTransaction` boundary. The single test path uses the in-memory constructor only.
- **expected:** Per `docs/ports/storage.md:468-477`: "The port requires: Unit tests of every repository method. Integration tests against a real database (testcontainers). A parity test verifying identical behavior across adapters. A tenancy test verifying cross-tenant reads are blocked. A failure-injection test (e.g. deadlock retry, connection drop). A load test (10k attendance marks in <5s)."
- **evidence:** `ls crates/adapters/storage-surrealdb/tests/` returns only `outbox_e2e.rs`. The file is 58 lines and exercises one round-trip. The handoff at `docs/handoff/PHASE-0-HANDOFF.md:14-19` records "120 tests pass workspace-wide" but does not name a SurrealDB parity / tenancy / failure-injection / load test.

---

### FINDING 33

- **id:** ADAPTER-SR-033
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-surrealdb/src/connection.rs:50-62` (`SurrealConnection::in_memory`)
- **description:** The `tracing` crate is a declared dependency (`Cargo.toml:21`) but the connection code (`connection.rs:50-62`) emits no `tracing::info!` / `tracing::debug!` on `connect`, no `tracing::warn!` on slow `use_ns` / `use_db`, and no `tracing::error!` on failure. The same is true for every other file in the crate (`grep -rn 'tracing::' crates/adapters/storage-surrealdb/src/` returns zero matches). The dialect spec (`docs/schemas/sql-dialects/surrealdb.md:847-849`) is explicit: "The adapter's DDL emission is unit-tested against an in-memory SurrealDB instance. The DDL is verified before any test queries run." The other adapters (`storage-postgres`, `storage-sqlite`) use `#[instrument(skip(self))]` on every port method and `tracing::debug!` on the connection open. The SurrealDB adapter's silence is asymmetric.
- **expected:** Per `AGENTS.md` § "Engine Rules" + the workspace convention: every port method is `#[instrument]`-decorated and emits `tracing` events on the connection lifecycle. The PG adapter at `crates/adapters/storage-postgres/src/storage.rs:41, 117, 129, 171, 185, 196, 218, 225, 237, 248` is the reference.
- **evidence:** `grep -rn 'tracing::' crates/adapters/storage-surrealdb/src/` returns zero matches. `crates/adapters/storage-surrealdb/Cargo.toml:21` `tracing = { workspace = true }` is declared but unused. `crates/adapters/storage-surrealdb/src/error.rs:8-9` documents the use of `StringError` to wrap "format!-style error messages without depending on anyhow" — no tracing integration.

---

### FINDING 34

- **id:** ADAPTER-SR-034
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-surrealdb/src/outbox.rs:41-46, 70-100` (`OutboxRow` payload handling) + `crates/adapters/storage-surrealdb/src/audit.rs:62-80, 100-114` (`AuditRow` payload handling) + `crates/adapters/storage-surrealdb/src/event_log.rs:50-70` (`EventRow` payload handling) + `crates/adapters/storage-surrealdb/src/idempotency.rs:35-76` (`IdempotencyRow` payload handling)
- **description:** Every sub-port that handles a payload (`outbox.payload`, `audit.before`, `audit.after`, `event_log.payload`, `idempotency.outcome`) writes and reads the payload as a `serde_json::Value` or as a `SurrealBytes` (raw bytes). The DDL declares `outbox.payload` as `TYPE object` (surreal.surql:91) — SurrealDB's object type, which is structurally a JSON object. The audit row's `before` / `after` are `TYPE option<bytes>` (surreal.surql:135-136) — raw bytes. The two semantics are different: `outbox.payload` is JSON-shaped (the DDL says `object`), `audit.before` is raw bytes. But the outbox `to_envelope` reads back a `serde_json::Value` and converts it to a `String` via `payload_to_bytes` (outbox.rs:149-155) which calls `other.to_string()` on non-string JSON values — meaning a JSON number, boolean, array, or null in the original payload round-trips as a stringified version, not as a binary blob. The semantic mismatch means a payload that was an object `{"x": 1}` round-trips as a string `{"x": 1}` (the JSON-serialised text) — not as a byte-exact JSON. The downstream `from_envelope` does the inverse: it tries to `serde_json::from_slice(&env.payload)` to recover the value, which succeeds for the stringified version (the string parses as JSON), so the round-trip is *visible* to be lossy only when the original was a primitive (string-of-string-of-string) or non-UTF-8.
- **expected:** The port contract says `payload: bytes::Bytes` is "the JSON (or MessagePack) representation" of the typed event (`crates/infra/storage/src/outbox.rs:75-84`). The round-trip must preserve the exact bytes. The SurrealDB `bytes` type is the natural fit; the `object` type is lossy.
- **evidence:** `crates/adapters/storage-surrealdb/src/outbox.rs:42-45` (writer) and `:149-155` (reader). Compare `crates/adapters/storage-surrealdb/src/audit.rs:62-67` and `:93-94` which correctly use `SurrealBytes::from(b.to_vec())` and `Bytes::from(b.to_vec())` for raw-byte round-trip.

---

### FINDING 35

- **id:** ADAPTER-SR-035
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-surrealdb/src/event_log.rs:218-273` (`EventLog::count`)
- **description:** `EventLog::count` always returns `0` (via the `rows.first().map(|r| ...).unwrap_or(0)` fallback) when the result set is empty, but it also clamps any non-zero `n` to `0` via `u64::try_from(r.n).unwrap_or(0)`. A negative `n` (impossible per the SurrealDB `count()` aggregate, but the code is defensive) is clamped to `0` — which is the same value as "no rows matched", so a real negative-n would be silently reported as zero count. More importantly, the deserialised `CountRow { n: i64 }` does not validate that `n >= 0` before the `u64::try_from` cast.
- **expected:** Per `crates/infra/storage/src/event_log.rs:156-158`: "Returns the count of events for `school_id` matching `filter`." The count must be exact.
- **evidence:** `crates/adapters/storage-surrealdb/src/event_log.rs:263-272`:
  ```rust
  #[derive(serde::Deserialize)]
  struct CountRow {
      n: i64,
  }
  let rows: Vec<CountRow> = response.take(0).map_err(|e| StringError(format!("event_log count take: {e}")))?;
  Ok(rows
      .first()
      .map(|r| u64::try_from(r.n).unwrap_or(0))
      .unwrap_or(0))
  ```

---

### FINDING 36

- **id:** ADAPTER-SR-036
- **area:** adapters
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.surreal.surql:71-260` (six tables, no `PERMISSIONS`)
- **description:** None of the six engine cross-cutting tables (`outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`) carry a `PERMISSIONS` clause. The dialect spec is explicit: "`PERMISSIONS NONE` on `outbox` is correct — the engine writes to it from the application layer, never from user sessions" (`docs/schemas/sql-dialects/surrealdb.md:471-474`). The DDL file is also missing the `PERMISSIONS NONE` for the other engine-internal tables. The migration comments on lines 110-118 acknowledge the engine's PERMISSIONS scope is the canonical RLS path: "Append-only is enforced at the engine layer (the SurrealDB adapter does not expose update/delete for this table to domain code) and at the SurrealDB layer via a REVOKE-style permission scope on the `audit_log` table for non-system roles." But the DDL does not emit that permission scope.
- **expected:** `DEFINE TABLE outbox SCHEMAFULL PERMISSIONS NONE;` and similarly for `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`.
- **evidence:** `migrations/engine/0000_engine_core.surreal.surql:71-72, 127-128, 164-165, 193-194, 226-227, 252-253` — every `DEFINE TABLE` is followed only by a `COMMENT` and a newline, no `PERMISSIONS` clause on any of the six.

---

### FINDING 37

- **id:** ADAPTER-SR-037
- **area:** adapters
- **severity:** Medium
- **location:** `migrations/engine/0000_engine_core.surreal.surql:91` (`payload` column on `outbox`) + `migrations/engine/0000_engine_core.surreal.surql:207` (`payload` column on `event_log`)
- **description:** The `outbox.payload` column is `TYPE object` (line 91) but the writer side at `outbox.rs:42-45` constructs a `serde_json::Value::String(...)` when the payload is non-JSON; SurrealDB's `object` type rejects `string` values. A binary payload (e.g. a CBOR-encoded event body, an attachment, a binary image) would fail the `ASSERT $value != NONE` and the `TYPE object` constraint — but the error is mapped to `Infrastructure` (Finding 8). A consumer using MessagePack or CBOR would see their outbox writes fail at runtime with no clear port-level error. The same risk applies to `event_log.payload`, which is `TYPE bytes` (line 207) — the round-trip is via `SurrealBytes::from(entry.payload.to_vec())` (event_log.rs:53) and `Bytes::from(self.payload.to_vec())` (event_log.rs:69), which is correct, but `surreal_bytes` and `bytes` are the same type only if `to_vec()` is consistent — the `surreal::sql::Bytes` type's serde repr may differ from `bytes::Bytes` (see the discrepancy in `outbox.rs:34` `payload: serde_json::Value` vs `event_log.rs:35` `payload: SurrealBytes`).
- **expected:** Pick one wire type for `payload` (either `object` or `bytes`) and make the Rust adapter use the corresponding converter consistently. Document the choice in `docs/schemas/sql-dialects/surrealdb.md`.
- **evidence:** `migrations/engine/0000_engine_core.surreal.surql:91` `DEFINE FIELD payload ON TABLE outbox TYPE object` and `:207` `DEFINE FIELD payload ON TABLE event_log TYPE bytes`. `crates/adapters/storage-surrealdb/src/outbox.rs:34` `pub payload: serde_json::Value` vs `crates/adapters/storage-surrealdb/src/event_log.rs:35` `pub payload: SurrealBytes`.

---

### FINDING 38

- **id:** ADAPTER-SR-038
- **area:** adapters
- **severity:** Medium
- **location:** `crates/adapters/storage-surrealdb/src/storage.rs:1-180` + `crates/adapters/storage-surrealdb/src/transaction.rs:1-114` (no `bulk_insert_student_attendances`)
- **description:** The `StorageAdapter` trait at `crates/infra/storage/src/port.rs:83-92` defines `bulk_insert_student_attendances(ctx, rows)` as a required port method (with a default `NotSupported` implementation). The same method exists on `Transaction` at `crates/infra/storage/src/transaction.rs:86-91`. The SurrealDB adapter does not override either. The Phase 5 bulk-marking service's critical path (per `docs/ports/storage.md:469-477` and the Phase 5 exit criterion: "200 rows in under 100 ms on PostgreSQL") has no SurrealDB implementation. The trait default returns `NotSupported`, which is the correct answer for the Phase 0 stub per the trait's own doc, but the `SurrealTransaction` struct has no `bulk` field and no plumbing for the bulk-insert path.
- **expected:** The PG / SQLite adapters implement `bulk_insert_student_attendances` (per the parity audit at `docs/audit_reports/findings/wave3-storage-sqlite.md:194-197`). SurrealDB does not — its consumers will see `DomainError::NotSupported("StorageAdapter::bulk_insert_student_attendances is not supported by this adapter")` on the bulk-marking service.
- **evidence:** `grep -n 'bulk_insert_student_attendances' crates/adapters/storage-surrealdb/src/` returns no matches. `crates/adapters/storage-surrealdb/src/transaction.rs:32-50` has fields `outbox, audit, event, idem, done, rolled_back, _db` but no `bulk` field. `crates/adapters/storage-surrealdb/src/storage.rs:75-172` has no override of `bulk_insert_student_attendances`.

---

### END FINDINGS

Total findings: 38 (Critical: 11, High: 17, Medium: 10, Low: 0).
