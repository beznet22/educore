# Educore Phase 0 — Foundation

> **Status: ✅ Done.** Phase 0 closed on PR 0 + PR A
> (the first of which flipped the clippy gap; the
> second flipped the docs). This document is a
> **retrospective** written at the close of Phase 1 to
> give a 1:1 phase-to-prompt mapping per the
> convention in [`README.md`](README.md).
> Future agents reading this in isolation should know
> the work it describes is already done; the canonical
> hand-off is [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md).

## Mission (retrospective)

You are starting the Educore engine build-out. Phase 0
lays the foundation: typed identifiers, the storage
port, the SurrealDB primary adapter, the sync engine
port + in-process reference impl, and the outbox e2e
that proves the storage + sync ports are wired
end-to-end.

This is **implementation**, not design. The specs
already exist in `docs/specs/` and `docs/ports/`. The
canonical hand-off narrative is
[`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md);
this prompt is a digest.

## Deliverables (✅ shipped at Phase 0 close-out)

- **`educore-core`** — errors (`DomainError` via
  `thiserror`), ids (`SchoolId`, `UserId`, `EventId`,
  `CorrelationId`, `IdempotencyKey`, `Source`,
  `UuidV7`), value objects (`Timestamp`, `Version`,
  `Etag`, `ActiveStatus`), clock (`Clock` port +
  `SystemClock` / `TestClock`), id_gen (`IdGenerator`
  port + `SystemIdGen` / `DeterministicIdGen`),
  `tenant.rs`, `query.rs` (the `EntityDescriptor` AST),
  `lint` sub-module (gated behind the `lint` feature).
- **`educore-query-derive`** — the
  `#[derive(DomainQuery)]` proc macro. Emits the
  field-exhaustiveness enum, the typed state builder,
  and the relation enum. 19 integration tests.
- **`educore-storage`** — the `StorageAdapter` port
  + 4 sub-ports (`Outbox`, `AuditLog`, `EventLog`,
  `Idempotency`). 11 unit tests.
- **`educore-storage-surrealdb`** — SurrealDB-backed
  adapter (`surrealdb 2.6.5` with `kv-mem` + `rustls`).
  6 of 6 cross-cutting tables emitted at `migrate()`
  time. Outbox e2e green. **The `Outbox` sub-port is
  real; `AuditLog`, `EventLog`, and `Idempotency` are
  `NotSupported` stubs** (the Phase 0 baseline; Phase 1
  parity adapters implement all 4 as real impls).
- **`educore-sync`** — the cross-cutting port
  (`SyncAdapter` / `SyncCoordinator`) per ADR-018.
  Command catalog (`SyncStart`, `SyncPause`,
  `SyncResume`, `SyncRequestDelta`, `SyncAcknowledge`).
  Event catalog (`SyncStarted`, `SyncPaused`,
  `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`,
  `SyncConflictDetected`). 1 object-safety test.
- **`educore-sync-inprocess`** — the in-process
  reference impl. Default for single-process
  deployments and the test target for the Phase 0
  e2e. 6 tests using `tokio::sync::{mpsc, broadcast}`.

## Required Reading (priority order)

1. [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md)
   — the canonical hand-off. Read first; it names
   the template, the port contract, and the
   starting-point adapter.
2. [`docs/build-plan.md`](../build-plan.md) § "Phase 0"
   — the canonical Phase 0 spec (the 10 tasks + 6
   exit criteria + coverage matrix updates + risks).
3. [`docs/ports/storage.md`](../ports/storage.md),
   [`docs/ports/sync.md`](../ports/sync.md) — the
   port contracts.
4. [`docs/decisions/ADR-013-CrateLayout.md`](../decisions/ADR-013-CrateLayout.md),
   [`ADR-014-Idempotency.md`](../decisions/ADR-014-Idempotency.md),
   [`ADR-015-ExternalCrates.md`](../decisions/ADR-015-ExternalCrates.md),
   [`ADR-016-EngineGraph.md`](../decisions/ADR-016-EngineGraph.md),
   [`ADR-017-SurrealDBFirst.md`](../decisions/ADR-017-SurrealDBFirst.md),
   [`ADR-018-SyncEngineArchitecture.md`](../decisions/ADR-018-SyncEngineArchitecture.md).
5. `AGENTS.md` — workspace rules, naming, lint policy,
   the 9-file module layout per domain.
6. `docs_guidlines/system.md` + `execution_guidlines.md`
   — engineering standards.

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

> **Retrospective note (Phase 0).** Phase 0 was
> implemented before the parallel-subagent pattern was
> formalised. The 6 crates shipped in sequential PRs
> (PR 0, PR A, …). Future agents who re-do Phase 0 work
> should treat the 6 crates as 6 parallel subagent
> workstreams and collapse the elapsed time.

## Starting Point

Empty scaffold. Bootstrap the 6 crates with
`cargo new --lib --vcs none crates/<name>` and follow
the per-crate module layout in `AGENTS.md`. The
canonical directory placement is
`crates/infra/<name>/` for the infra tier and
`crates/cross-cutting/<name>/` for the sync engine.

## Per-Deliverable Gotchas (retrospective — all hit at Phase 0)

- **`educore-core` lint sub-module**: gated behind
  the `lint` Cargo feature. Binary at
  `src/bin/lint.rs`. The lint source file itself
  contains `unwrap` / `expect` / `panic!` /
  `unimplemented!` as string literals (the patterns
  being scanned for); the scanner must skip its own
  source. Phase 0 ships the lint binary as
  cargo-shippable but features-gated.
- **`educore-storage-surrealdb` stub pattern**: only
  the `Outbox` sub-port is real. `AuditLog`,
  `EventLog`, and `Idempotency` return
  `NotSupported`. This is the Phase 0 baseline; the
  Phase 1 SQL adapters implement all 4 as real
  impls. The SurrealDB stub pattern is still in
  place at Phase 1 close (a future PR should add
  the same 4-port parity to SurrealDB).
- **`educore-sync` event catalog**: 5 of 7 events
  emitted by the in-process impl. `SyncAcknowledge`
  command and `SyncConflictDetected` event are
  deferred. The Phase 0 ad-hoc sync envelope refactor
  (use `educore_events::EventEnvelope` instead of
  the local `SyncEvent` struct) is flagged as a
  Phase 2 deliverable.
- **`surrealdb` driver**: pinned to the last
  pre-1.75 line per ADR-015. The crate is pre-1.0
  and the latest dev line raises MSRV above the
  engine's 1.75 floor.
- **`EntityDescriptor` AST type**: the macro emits
  `QueryNode<F>` AST + `Field` / `HasRelations`
  traits. The concrete `EntityDescriptor` struct
  itself lands with the first domain crate (Phase 3+).
  Phase 0 ships the macro plumbing; Phase 3 wires the
  concrete struct.
- **`mysql_async` line for MySQL**: NOT in Phase 0
  scope. MySQL parity is Phase 1 (which delivered
  with `sqlx`, not `mysql_async`).

## Exit Criteria (✅ all met at Phase 0 close-out)

1. `cargo build --workspace` green
2. `cargo test -p educore-storage-surrealdb` green;
   the outbox e2e test passes
3. The outbox DDL emitted by the adapter byte-matches
   `migrations/engine/0000_engine_core.surreal.surql`
4. `cargo test -p educore-sync-inprocess` green; the
   sync e2e test passes
5. `cargo clippy --workspace --all-targets -- -D warnings`
   green (PR 0 closed the clippy gap; see the
   hand-off)
6. `cargo fmt --all -- --check` green

120 tests pass at Phase 0 close-out (was 124 at
Phase 1 close-out; the +4 are the MySQL
`connection::tests` URL helper unit tests).

## When You Are Stuck (retrospective pointers)

- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The SurrealDB adapter is the canonical "engine
  internal" template; no prior art existed at Phase 0
  start.
- For the macro, build it in two steps: struct →
  descriptor (the AST walk) and descriptor → DDL (the
  SurrealDB emission). Unit test each step
  independently.
- For design questions, do not invent — open an
  issue or ask the user. Phase 0 is execution.

## Outcomes to verify at Phase 0 close-out

- [x] 6 crates delivered
- [x] 120 tests pass
- [x] `cargo build --workspace` green
- [x] `cargo test -p educore-storage-surrealdb` green
- [x] Outbox DDL byte-matches the `.surql` file
- [x] `cargo test -p educore-sync-inprocess` green
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
      green
- [x] `cargo fmt --all -- --check` green
- [x] 13 `docs/coverage.toml` rows flipped
      (4 SurrealDB DDL rows + 6 foundation rows + 3
      sync rows)
- [x] Engine knowledge graph
      (`graphify-out/GRAPH_REPORT.md`) auto-rebuilt
- [x] `docs/handoff/PHASE-0-HANDOFF.md` written
- [x] `docs/build-plan.md` § "Phase 0 outcome." added
- [x] `docs/phase_prompt/phase-1-prompt.md` written (the
      next-phase prompt, per the convention in
      [`README.md`](README.md))
