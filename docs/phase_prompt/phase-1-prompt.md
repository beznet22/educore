# Educore Phase 1 — Storage Adapter Parity

## Mission

You are continuing the Educore engine build-out. Phase 0 closed
on PR 0 + PR A (see [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md)
and [`docs/build-plan.md` § "Phase 0 outcome"](../build-plan.md)).
Your job is **Phase 1**: deliver `educore-storage-postgres`,
`educore-storage-mysql`, and `educore-storage-sqlite` so the
Phase 0 outbox scenario runs on all four adapters byte-equivalent
modulo the dialect differences in
`docs/schemas/sql-dialects/comparison.md`.

This is an implementation mission. The specs already exist; the
templates already exist; the cross-adapter parity shape is
already in `docs/coverage.toml`. You are not designing — you
are executing.

You are NOT:
- Designing new ports or new aggregates
- Modifying the existing `educore-storage-surrealdb` adapter
- Renaming crates or moving files
- Adding new external crates without updating ADR-015

You ARE:
- Implementing the three parity adapters per
  [`docs/ports/storage.md`](../ports/storage.md) and the per-dialect
  DDL in
  `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql`
- Wiring `sqlx` (PG, SQLite) and `mysql_async` (MySQL) per
  [`docs/decisions/ADR-015-ExternalCrates.md`](../decisions/ADR-015-ExternalCrates.md)
- Flipping `docs/coverage.toml` rows `Pending` → `Tested` in
  the same commit as the impl

## Required Reading (priority order)

1. [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md) — the hand-off from Phase 0
2. [`docs/build-plan.md`](../build-plan.md) § "Phase 1" — the canonical Phase 1 spec
3. [`docs/ports/storage.md`](../ports/storage.md) — the port contract
4. [`docs/schemas/sql-dialects/postgresql.md`](../schemas/sql-dialects/postgresql.md),
   [`mysql.md`](../schemas/sql-dialects/mysql.md),
   [`sqlite.md`](../schemas/sql-dialects/sqlite.md) — per-dialect conventions
5. [`docs/schemas/sql-dialects/comparison.md`](../schemas/sql-dialects/comparison.md) — what differs across the three
6. [`AGENTS.md`](../AGENTS.md) — workspace rules, naming, lint policy
7. `docs_guidlines/system.md` + `execution_guidlines.md` — engineering standards

## Working With Subagents

**Use the task tool to spawn subagents in parallel** for
independent workstreams. This phase has multiple discrete
deliverables that can be executed concurrently
(e.g., one per crate, one per adapter, one per
subsystem); launching them in parallel maximizes speed
and context-window utilization.

**Spawn focused, self-contained subagents.** Each
subagent gets a prompt with:
- The exact files to create or modify (paths, not summaries)
- The exact files to read for context
- The exact verification commands to run
  (`cargo build -p <pkg>`, `cargo test -p <pkg>`,
  `cargo clippy ... -D warnings`, `cargo fmt -- --check`)
- The exact acceptance criteria scoped to that workstream

**Coordinate via the filesystem.** Subagents share the
workspace git checkout. They read each other's outputs
as they complete. Do not coordinate via memory or
message-passing; the filesystem IS the contract. If
two subagents need to modify the same file, sequence
them — one runs to completion, the other reads the
result.

**Verify independently.** Do not trust a subagent's
"done" claim without running the build/test/clippy/fmt
checks on the result. The closing agent (you) is
responsible for the final workspace-wide gates and for
the integration work (coverage rows, hand-off, next-
phase prompt).

> **Phase 1 workstreams.** The three SQL adapters
> (`educore-storage-postgres`, `educore-storage-mysql`,
> `educore-storage-sqlite`) are fully independent — each
> is a self-contained subagent scope that touches its
> own crate directory. Spawn three subagents in a
> single batch; the PG template serves as the canonical
> reference for the MySQL and SQLite subagents. The
> final integration work (coverage rows, ADR-015
> audit log entry, hand-off, next-phase prompt) is the
> closing agent's job.

## Starting Point

Template: `crates/adapters/storage-surrealdb/src/` — copy its
structure, swap the driver.

Reference DDL: `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql`
are pre-written; `include_str!` them at compile time.

E2E template: `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`
— mirror it per adapter with the same invariants (school_id,
UUIDv7, append → pending round-trip, mark_published).

## Per-Dialect Gotchas

- **PostgreSQL** (`educore-storage-postgres`):
  - Native RLS via `CREATE POLICY` + `ALTER TABLE ... ENABLE ROW LEVEL SECURITY`.
  - `CHECK` constraints are reliable; no floor.
  - `sqlx` is the driver; `sqlx 0.8.x` pinned per ADR-015.

- **MySQL** (`educore-storage-mysql`):
  - **Floor: MySQL 8.0.16+** for `CHECK` constraint enforcement.
    Gate the test on this version; document in `mysql.md`.
  - **No native RLS.** Emulate via session variable:
    `SET @app_tenant_id = ?` and `WHERE school_id = @app_tenant_id`
    on every query. Per `mysql.md`.
  - `utf8mb4_unicode_ci`, `ENGINE=InnoDB`, backtick identifier
    quoting.
  - `mysql_async 0.34.x` is the driver; `flate2 1.1` with
    `rust_backend` is required for cross-compile (already in
    the workspace `Cargo.toml`).

- **SQLite** (`educore-storage-sqlite`):
  - **No RLS, no schema namespaces.** Tenant filtering is
    application-layer only.
  - **Single-writer**: concurrent writes serialize. This is a
    deployment constraint, not a correctness concern.
  - `TEXT` with `CHECK(length() = 36)` for UUIDs; `INTEGER` for
    booleans; ISO 8601 `TEXT` for timestamps.
  - JSON via the `json1` extension at the application layer.
  - `sqlx 0.8.x` with the `sqlite` feature.

## Exit Criteria

1. `cargo test -p educore-storage-postgres` green (outbox e2e).
2. `cargo test -p educore-storage-mysql` green.
3. `cargo test -p educore-storage-sqlite` green.
4. `cargo test --workspace` green (no regressions in Phase 0).
5. `cargo clippy --workspace --all-targets -- -D warnings` green.
6. `cargo fmt --all -- --check` green.
7. `docs/coverage.toml` rows for the three new adapters flipped
   to `Tested` with `tests` paths.

The cross-adapter test (all four adapters in one scenario) is
the Phase 16 deliverable, not Phase 1. Phase 1 just needs each
adapter's individual e2e to be green.

## When You Are Stuck

- Re-read the Phase 0 hand-off; it names the template, the port
  contract, and the starting-point adapter.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --features lint --bin lint`.
- The Phase 0 commit history (`git log --oneline --grep="Phase 0"`)
  is a working reference for the SurrealDB adapter.
- For design questions, do not invent — open an issue or ask
  the user. Phase 1 is execution, not design.
