# Educore Phase 0 — Foundation

> **Status: ✅ Done** (retrospective). Closed on PR 0 + PR A.
> See [`PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md) for the canonical hand-off.

## Mission
Foundation: typed ids, the storage port, the SurrealDB primary adapter, the sync engine port + in-process reference impl, and the outbox e2e. **Implementation**, not design.

## Deliverables
- `educore-core` (errors, ids, value objects, `Clock`, `IdGenerator`, `lint` sub-module behind the `lint` feature)
- `educore-query-derive` (`#[derive(DomainQuery)]` proc macro)
- `educore-storage` (`StorageAdapter` port + 4 sub-ports)
- `educore-storage-surrealdb` (SurrealDB primary; outbox real, others `NotSupported`)
- `educore-sync` + `educore-sync-inprocess` (per ADR-018)

## Required Reading
- `docs/handoff/PHASE-0-HANDOFF.md` (canonical)
- `docs/build-plan.md` § "Phase 0"
- `docs/ports/storage.md`, `docs/ports/sync.md`
- `docs/specs/sync/`, `docs/specs/platform/` (foundation references)
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-014-Idempotency.md`, `ADR-015-ExternalCrates.md`, `ADR-016-EngineGraph.md`, `ADR-017-SurrealDBFirst.md`, `ADR-018-SyncEngineArchitecture.md`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
Empty scaffold. Bootstrap 6 crates with `cargo new --lib --vcs none crates/<name>`; per-crate layout in `AGENTS.md`. Infra tier at `crates/infra/<name>/`; sync at `crates/cross-cutting/<name>/`.

## Working With Subagents
6 crates = 6 parallel subagent scopes. Subagent prompt must include: exact files to read/write, exact verification commands, scoped acceptance criteria. Coordinate via the filesystem; do not trust "done" without running build/test/clippy/fmt.

## Per-Deliverable Gotchas
- `educore-core` lint is `#[cfg(feature = "lint")]`; binary at `src/bin/lint.rs`; scanner must skip its own source.
- `educore-storage-surrealdb`: only `Outbox` real; others `NotSupported` (Phase 0 baseline; Phase 1 SQL adapters add the real impls).
- `surrealdb` driver pinned to the last pre-1.75 line per ADR-015.
- `EntityDescriptor` concrete struct lands with the first domain crate (Phase 3+); Phase 0 ships the macro plumbing.
- `mysql_async` is **not** in Phase 0 (rejected; MySQL parity is Phase 1 with `sqlx`).

## Exit Criteria
`cargo build/test/clippy/fmt --workspace` green; outbox e2e on SurrealDB passes; outbox DDL byte-matches the `.surql` file; sync e2e passes; 13 `coverage.toml` rows flipped; `PHASE-0-HANDOFF.md` + `build-plan.md` § "Phase 0 outcome." + `phase-1-prompt.md` written.

## When You Are Stuck
`cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. Phase 0 commit history is the working reference. For design questions, open an issue — Phase 0 is execution.
