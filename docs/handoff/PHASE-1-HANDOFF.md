# Phase 1 → Phase 2 Hand-off

**Audience:** the next agent starting Phase 2.
**Status:** Phase 1 closed. 7 of 7 exit criteria green
(`cargo test -p educore-storage-{postgres,mysql,sqlite}`,
`cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all -- --check`, the `educore-core::lint` binary,
and 15 `docs/coverage.toml` rows flipped from `Pending` to
`Tested` with `tests` paths).

## What's wired and working

- 3 new adapter crates delivered: `educore-storage-postgres`,
  `educore-storage-mysql`, `educore-storage-sqlite`. Each
  ships with all 4 sub-ports (`Outbox`, `AuditLog`, `EventLog`,
  `Idempotency`) as **real impls** — no `NotSupported` stubs.
  This is a deliberate departure from the Phase 0 SurrealDB
  pattern (where only `Outbox` was real).
- Workspace test count: **124 passing** (was 120 at Phase 0
  close-out; +4 from the MySQL `connection::tests` URL helper
  unit tests). Each adapter also has 1 outbox e2e test for
  a total of 3 additional e2e tests in the storage tier.
- 15 `docs/coverage.toml` rows flipped `Pending` → `Tested`:
  4 DDL rows (`outbox_ddl`, `idempotency_ddl`,
  `schema_registry_ddl`, `system_user_ddl`) × 3 adapters + 3
  storage-impl rows. The `audit_log_ddl_*` and `event_log_ddl_*`
  rows are **not** Phase 1 — those are owned by `educore-audit`
  and `educore-events` (Phase 2).
- Driver choice: **`sqlx 0.8` for all three SQL adapters**
  (PostgreSQL, MySQL, SQLite). The previous plan to use
  `mysql_async` for MySQL was rejected during this session —
  `mysql_async` and the transitive `flate2` direct dep have
  been removed from `crates/adapters/storage-mysql/Cargo.toml`.
  The workspace `Cargo.toml` still pins them for historical
  reasons; a cleanup PR can drop them.
- Phase 1 design choice: **flag-based per-call transactions**.
  `PostgresTransaction` / `MysqlTransaction` /
  `SqliteTransaction` are flag-based wrappers that hold the
  pool + 4 sub-port handles; each sub-port call opens its own
  short `pool.begin()`. The engine's at-least-once dedup
  (`event_id` PK on outbox; `ON CONFLICT DO NOTHING` /
  `ON DUPLICATE KEY UPDATE` on idempotency) is the safety
  net for the resulting non-atomic command dispatch. A future
  PR can thread a real `sqlx::Transaction` through the
  sub-port methods for true atomicity (see "Open questions"
  below).

## Per-adapter implementation notes

### `educore-storage-postgres` (`crates/adapters/storage-postgres/`)

- `PostgresConnection::connect(url, school)` opens a
  `sqlx::PgPool` and registers an `after_connect` hook that
  issues `SET search_path = engine, public` on every new
  connection. The DDL script also issues the same `SET` at
  the script level, so re-running `migrate()` on an
  already-migrated DB is safe.
- Queries use **unqualified** table names (`outbox`,
  `audit_log`, etc.); the `search_path` resolves them to the
  `engine` schema.
- `mark_published` uses PostgreSQL's `ANY($1)` with a
  `Vec<Uuid>` bind.
- `Idempotency::record` uses
  `ON CONFLICT (school_id, command_type, idempotency_key) DO NOTHING`.
- `EventLog::read` / `count` build the `WHERE` clause
  dynamically with `build_select` (a `String` SQL + a
  `Vec<FilterParam>` enum). Values are bound positionally;
  no `format!` interpolation of user input.
- The outbox e2e (`tests/outbox_e2e.rs`) is gated on
  `EDUCORE_PG_URL`; when unset, it logs via `tracing::info!`
  and returns early. CI is green without a running PG;
  contributors with a local PG can set the env var to
  exercise the full e2e.

### `educore-storage-mysql` (`crates/adapters/storage-mysql/`)

- `MysqlConnection::connect(url, school)` parses the URL and
  calls `MySqlConnectOptions::from_url(...)`. The URL must
  include `?multi_statements=true` (or
  `&multi_statements=true` if a query string already
  exists). The `ensure_multi_statements` helper in
  `connection.rs` idempotently appends it; 4 unit tests cover
  the no-query, with-query, already-present, and case-
  insensitive cases.
- `after_connect` issues
  `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci` on every
  new connection.
- Backtick-quote every identifier (`` `outbox` ``,
  `` `event_id` ``, …).
- `?` placeholders throughout. `IN (?)` for `Vec<Uuid>` /
  `Vec<String>` is expanded by sqlx 0.8's MySQL driver
  automatically — but `Vec<T>` does not implement
  `Encode<MySql>` / `Type<MySql>` in sqlx 0.8 (only
  `Vec<u8>`), so the `mark_published` and `event_log` read
  paths use `sqlx::QueryBuilder<MySql>` with `.push_bind` /
  `.separated(", ")`.
- `mark_published` uses `IN (?)` (not PG's `= ANY($1)`).
- `Idempotency::record` uses
  `` ON DUPLICATE KEY UPDATE `command_id` = `command_id` ``
  (a no-op self-assignment that preserves the existing row
  on conflict and suppresses the "Rows matched: 1 Changed: 0"
  warning).
- All `sqlx::query[_as/_scalar]` calls need the
  `sqlx::MySql` turbofish (sqlx 0.8 with all 3 driver
  features enabled picks `Postgres` as the default; the
  MySQL adapter must disambiguate explicitly).
- All `Utc::now()` / `NOW()` SQL calls use `UTC_TIMESTAMP(6)`
  (UTC, microsecond precision) — `NOW()` returns local time
  on some MySQL configurations.
- The e2e is gated on `EDUCORE_MYSQL_URL` (same skip
  pattern as PG).

### `educore-storage-sqlite` (`crates/adapters/storage-sqlite/`)

- `SqliteConnection::in_memory(school)` and
  `SqliteConnection::connect(url, school)` open a
  `sqlx::SqlitePool`. In-memory mode uses
  `SqliteConnectOptions::new().in_memory(true)` for tests.
- `after_connect` issues `PRAGMA journal_mode = WAL`,
  `PRAGMA synchronous = NORMAL`, `PRAGMA foreign_keys = ON`
  on every new connection.
- UUIDs are bound as `uuid::fmt::Hyphenated` (the
  canonical 36-char hyphenated text form). sqlx 0.8's
  SQLite driver does not re-export `sqlx::types::Uuid` for
  storage as `TEXT` — `uuid::Uuid` maps to `BLOB(16)`,
  which would violate the `CHECK(length(x) = 36)` invariant
  on the DDL.
- The e2e test uses the in-memory constructor and **always
  runs** in CI (no env-var skip).
- Note: the existing e2e currently exercises the **in-memory**
  path only. The single-writer deployment model (per
  `docs/schemas/sql-dialects/sqlite.md#transactions`) is
  documented but not tested at scale.

## What's stubbed (Phase 2+ work)

- All three adapters' **sync primitives** (`watch_changes`,
  `apply_snapshot`, `cursor_for`, `advance_cursor`) return
  `NotSupported` (the storage port's default impls). PG's
  `LISTEN/NOTIFY` for `watch_changes`, MySQL binlog tail,
  SQLite polling — all deferred to a sync-capable
  follow-up. **Not blocking Phase 2** because Phase 2's
  primary deliverable is the audit / events crates, not
  sync change feeds.
- The `educore-storage-surrealdb` adapter's `AuditLog`,
  `EventLog`, and `Idempotency` sub-ports are **still**
  `NotSupported` stubs. Only its `Outbox` is real. A
  future PR should add the same 4-port parity the SQL
  adapters now have. **Not blocking Phase 2** but worth
  noting for parity.
- The cross-adapter parity test suite
  (`educore-storage-parity`) is **Phase 16 work**, not
  Phase 1. Phase 1 has per-adapter e2e only.

## Open questions

1. **Flag-based transactions.** The `PostgresTransaction` /
   `MysqlTransaction` / `SqliteTransaction` structs do not
   hold a real `sqlx::Transaction`; each sub-port call
   opens its own short `pool.begin()`. A future PR could
   thread a real transaction through the sub-port
   methods for true command-dispatch atomicity. The engine's
   at-least-once dedup is the safety net for the current
   design; a real `sqlx::Transaction` would close the window
   where a duplicate dispatch lands between two sub-port
   calls. See per-adapter `src/transaction.rs` module-level
   docs.
2. **`AuditLogEntry` / `EventLogEntry` struct fields are
   a subset of the DDL columns.** The DDL has columns the
   struct doesn't carry (`ip`, `user_agent`, `session_id`,
   `command_id`, `cross_tenant` on `audit_log`; no
   `active_status` on `event_log`). The adapters fill in
   safe defaults on write and drop the columns on read.
   A future PR should reconcile (likely expand the port
   structs to carry the missing fields).
3. **`IdempotencyRecord::command_type: &'static str` requires
   a `Box::leak` on the SQLite read path.** The storage
   port should change the field from `&'static str` to
   `String` (or `Cow<'static, str>`) in a follow-up; the
   adapter can then drop the leak.
4. **Workspace `Cargo.toml` still declares `mysql_async` and
   `flate2` as workspace deps** (for historical reasons —
   the comment block explains the original `flate2/zlib-rs`
   dependency chain). They are no longer referenced by any
   workspace crate. A cleanup PR can drop them entirely.
5. **`educore-events` envelope crate is the missing link.**
   The SQL adapters' `AuditLog::append` and
   `EventLog::append` take the storage-port structs (not
   the engine's domain events) because the `educore-events`
   crate doesn't exist yet. Phase 2's first task should
   land `educore-events` and have the audit/event sub-ports
   take `EventEnvelope<T: DomainEvent>` instead of the raw
   port structs.

## Phase 2 entry point

Start with **`educore-platform`** — the smallest cross-
cutting crate and the prerequisite for every Phase 2
crate that follows. The template is the per-domain module
layout in `AGENTS.md`:

1. Read `docs/specs/platform/overview.md` (the design
   contract).
2. Bootstrap `crates/cross-cutting/platform/src/` with
   `aggregate.rs`, `entities.rs`, `value_objects.rs`,
   `commands.rs`, `events.rs`, `services.rs`,
   `repository.rs`, `query.rs`, `errors.rs` (the 9-file
   module layout per `AGENTS.md`).
3. Add `#[derive(DomainQuery)]` per `docs/specs/platform/aggregates.md`.
4. Wire the 4 SQL adapters' `audit_log` / `event_log`
   tables (the DDLs are already there from Phase 1) to
   take `EventEnvelope<T: DomainEvent>` once
   `educore-events` ships.

Then `educore-rbac`, `educore-events`, `educore-event-bus`,
`educore-audit`, in that order (each is a prerequisite for
the next). The Phase 2 integration test (per
`docs/build-plan.md` § "Phase 2") exercises all 5
crates end-to-end.

## Where NOT to start

- Don't touch `educore-core::lint` — it's done.
- Don't add new `unwrap`/`expect` in domain code —
  `AGENTS.md` forbids it; the lint will flag it.
- Don't rename or move crates without an ADR. The current
  layout is canonical per
  `docs/decisions/ADR-013-CrateLayout.md`.
- Don't refactor the Phase 1 SQL adapters' flag-based
  transaction model unless you're also adding real
  `sqlx::Transaction` support — the current design is
  intentional and the engine's at-least-once dedup is the
  safety net.
- Don't re-add `mysql_async` to `educore-storage-mysql`;
  the user has explicitly chosen `sqlx` for all three
  SQL adapters.

## Key files for Phase 2

- `docs/build-plan.md` § "Phase 2" — the canonical Phase 2 spec.
- `docs/handoff/PHASE-0-HANDOFF.md` — the prior hand-off
  (Phase 0's open questions still apply; the ad-hoc sync
  envelope refactor is one of Phase 2's deliverables).
- `docs/ports/event-bus.md` — the `EventBus` port
  contract that `educore-event-bus` implements.
- `docs/ports/storage.md` — the storage port contract
  that the Phase 1 adapters implement.
- `docs/specs/platform/`, `docs/specs/rbac/`,
  `docs/specs/audit/` — the design contracts.
- `docs/schemas/audit-schema.md` § 13 — the `audit_log`
  table spec; the partitioning strategy for the
  10M-rows/day scale is documented here.
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level
  security" — the `CREATE POLICY` + `ENABLE ROW LEVEL
  SECURITY` pattern for the cross-tenant isolation test.
- `docs/coverage.toml` — the rows to flip on Phase 2
  close (one per platform / rbac / events / audit
  aggregate + command + event).
- `docs/phase_prompt/phase-2-prompt.md` — the forward-
  looking brief for the Phase 2 agent.
- `crates/adapters/storage-{postgres,mysql,sqlite}/src/transaction.rs`
  — the flag-based transaction model that Phase 2's
  audit / event sub-port calls will use.

## Questions? Where to ask

- Spec questions: see `docs/specs/<domain>/overview.md`
  first; then `docs/decisions/*.md`; then `AGENTS.md`
  "Authoritative Documents" reading order.
- Process questions: see `AGENTS.md` § "Agent Instructions"
  + "Validation Checklist".
- Phase 1 history: this file + `git log --oneline --grep="Phase 1"`
  after the closing PR lands.
