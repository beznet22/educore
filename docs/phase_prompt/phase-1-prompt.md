# Educore Phase 1 — Storage Adapter Parity

## Mission
Deliver `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite` so the Phase 0 outbox scenario runs on all four adapters byte-equivalent modulo dialect differences. **Implementation**, not design.

## Deliverables
- 3 SQL adapter crates implementing the 4 sub-ports per `docs/ports/storage.md`
- DDL in `migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql` via `include_str!`
- Per-adapter e2e tests mirroring `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`
- `docs/coverage.toml` rows flipped `Pending` → `Tested` in the same commits

## Required Reading
- `docs/handoff/PHASE-0-HANDOFF.md`
- `docs/build-plan.md` § "Phase 1"
- `docs/ports/storage.md`
- `docs/schemas/sql-dialects/{postgresql,mysql,sqlite,comparison}.md`
- `docs/schemas/database-schema.md`, `docs/schemas/event-schema.md`, `docs/schemas/command-schema.md`
- `docs/decisions/ADR-015-ExternalCrates.md`, `ADR-013-CrateLayout.md`
- `crates/adapters/storage-surrealdb/src/` (the canonical Phase 0 template)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
Template: `crates/adapters/storage-surrealdb/src/` (copy structure, swap driver). Reference DDL pre-written; `include_str!` at compile time.

## Working With Subagents
The 3 SQL adapters are fully independent — one subagent scope per crate. Spawn 3 subagents in a single batch. PG is the canonical reference for the MySQL and SQLite subagents. Closing agent: coverage rows, ADR-015 audit, hand-off, next-phase prompt.

## Per-Dialect Gotchas
- **PostgreSQL**: `sqlx 0.8.x`; native RLS via `CREATE POLICY` + `ENABLE ROW LEVEL SECURITY`.
- **MySQL**: floor MySQL 8.0.16+ for `CHECK`; no native RLS (emulate via `SET @app_tenant_id`); `utf8mb4_unicode_ci` + `InnoDB`; `sqlx 0.8.x` (NOT `mysql_async`).
- **SQLite**: no RLS (application-layer only); single-writer; `TEXT` with `CHECK(length() = 36)` for UUIDs; `sqlx 0.8.x` + `sqlite` feature + `json1`.

## Exit Criteria
Each adapter's e2e green; `cargo test/clippy/fmt --workspace` green; no Phase 0 regressions; 3 `coverage.toml` rows flipped. Cross-adapter test (all 4 in one scenario) is Phase 16, not Phase 1.

## When You Are Stuck
Phase 0 hand-off is the template. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. `git log --oneline --grep="Phase 0"` is the working reference.
