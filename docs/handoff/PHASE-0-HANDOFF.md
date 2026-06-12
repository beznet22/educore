# Phase 0 → Phase 1 Hand-off

**Audience:** the next agent starting Phase 1.
**Status:** Phase 0 closed on PR 0 + PR A. All 6 Phase 0 exit
criteria are green (`cargo build`, `cargo test`, `cargo fmt`,
`cargo clippy -D warnings`, SurrealDB outbox e2e, sync e2e).

## What's wired and working

- 6 crates delivered: `educore-core`, `educore-query-derive`,
  `educore-storage`, `educore-storage-surrealdb`, `educore-sync`,
  `educore-sync-inprocess`. Plus the umbrella re-exports
  (`educore::storage_surrealdb`, `educore::sync`,
  `educore::sync_inprocess`).
- 120 tests pass workspace-wide. Of those, the Phase 0 e2e is
  `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`:
  in-memory SurrealDB → outbox round-trip → sync coordinator
  fan-out, asserts invariants (school_id, UUIDv7, byte-for-byte
  DDL match).
- SurrealDB driver: `surrealdb = "2"` with `kv-mem` + `rustls`,
  pinned to the last pre-1.75 line. See
  `docs/decisions/ADR-015-ExternalCrates.md` for the line number.

## What's stubbed (don't rely on these)

- `educore-storage-surrealdb` `AuditLog`, `EventLog`, and
  `Idempotency` sub-ports return `NotSupported`. Only `Outbox` is
  real. Phase 2 (`educore-audit` + `educore-events`) is when the
  other three get real impls on every backend.
- `educore-sync` is the in-process coordinator only. The HTTP
  worker (`educore-sync-http`) is **deferred to Phase 2** and the
  no-op (`educore-sync-null`) to **Phase 16**. The sync port
  surface is fully tested; the wire format is not.
- 4 sync events of 7 (per `docs/specs/sync/overview.md`) are
  emitted by the in-process impl: `SyncStarted`, `SyncPaused`,
  `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`. Missing:
  `SyncAcknowledge` command, `SyncConflictDetected` event.
- The `#[derive(DomainQuery)]` macro is real but **not yet used
  by any domain crate**. Its tests are the proof of life.
- `EntityDescriptor` AST type is referenced in
  `docs/query_layer.md` and `build-plan.md` § Phase 0 task 1 but
  is not yet defined as a concrete type. **Design gap — resolve
  at Phase 3 (first domain crate).** Do not refactor around it in
  Phase 1.

## Open questions

1. **EntityDescriptor AST shape** — what fields? `table`,
   `columns`, `indexes`, `foreign_keys`, `rls` per the build
   plan; concrete types TBD. Phase 3 will land them.
2. **Ad-hoc sync envelope types** — the Phase 0 sync impl uses
   its own event struct, not `educore_events::EventEnvelope`.
   Phase 2 should refactor sync to depend on the envelope crate.
3. **Phase 0 graph in `graphify-out/`** is auto-rebuilt on every
   commit via the local `graphify hook install` (one-time setup).
   `tools/scripts/check-graph-freshness.sh` is the freshness
   gate.

## Phase 1 entry point

Start with `educore-storage-postgres`. The template is
`educore-storage-surrealdb`:

1. Copy the adapter skeleton from
   `crates/adapters/storage-surrealdb/src/`.
2. `include_str!` `migrations/engine/0000_engine_core.postgres.sql`
   (already exists).
3. Implement the 4 sub-ports (Outbox, AuditLog, EventLog,
   Idempotency) — all real this time, not stubbed.
4. Add `crates/adapters/storage-postgres/tests/outbox_e2e.rs`
   mirroring the SurrealDB e2e.
5. Update `docs/coverage.toml` rows
   `outbox_ddl_pg` → `Tested`, etc.

Then MySQL and SQLite. The cross-adapter test
(`educore-storage-parity`, Phase 16) will run all four backends
through the same scenario.

## Where NOT to start

- Don't touch `educore-core` `lint` sub-module — it's done.
- Don't add new `unwrap`/`expect` in domain code — AGENTS.md
  forbids it; the lint will flag it.
- Don't rename or move crates without an ADR. The current layout
  is canonical per `docs/decisions/ADR-013-CrateLayout.md`.

## Key files for Phase 1

- `docs/build-plan.md` § Phase 1 — the canonical Phase 1 spec.
- `docs/ports/storage.md` — the port contract.
- `docs/schemas/sql-dialects/postgresql.md` — PG conventions.
- `migrations/engine/0000_engine_core.postgres.sql` — the
  reference DDL to `include_str!`.
- `crates/adapters/storage-surrealdb/src/` — the template.
- `docs/coverage.toml` — flip `Pending` → `Tested` in the same
  commit as the impl.

## Questions? Where to ask

- Spec questions: see `docs/specs/<domain>/overview.md` first;
  then `docs/decisions/*.md`; then AGENTS.md "Authoritative
  Documents" reading order.
- Process questions: see AGENTS.md "Agent Instructions" +
  "Validation Checklist".
- Phase 0 history: this file + git log (`git log --oneline
  --grep="Phase 0"` after PR 0 + PR A land).
