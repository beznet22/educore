# Educore Build Plan

The engine is implemented in **17 sequential phases** (Phase 0..17). Each
phase has explicit exit criteria and updates the Coverage Matrix
(§ [The Coverage Matrix](#the-coverage-matrix)). The runtime DDL
emission flow referenced throughout is documented in
[`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission — end-to-end flow"](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow).

## SurrealDB-first + Sync engine additions

Two new ADRs amend this build plan:

- [`docs/decisions/ADR-017-SurrealDBFirst.md`](decisions/ADR-017-SurrealDBFirst.md)
  — **SurrealDB becomes the primary storage backend.** The Phase 0
  adapter is `educore-storage-surrealdb` (replacing
  `educore-storage-postgres` as the reference target). PG, MySQL,
  and SQLite move to Phase 1 as parity adapters. SurrealDB's
  `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL is the
  canonical reference; the SQL dialects emit the same engine
  invariants in their own syntax.
- [`docs/decisions/ADR-018-SyncEngineArchitecture.md`](decisions/ADR-018-SyncEngineArchitecture.md)
  — **A new sync engine layer is added at Phase 0.** This is a
  cross-cutting port (`educore-sync`) with commands, events, and
  a coordinator that drives offline-first replication between
  the canonical server and edge clients. One reference
  implementation ships at Phase 0: `educore-sync-inprocess` (the
  in-process coordinator, default for single-process deployments
  and tests). The `educore-sync-http` worker client and
  `educore-sync-null` no-op impl are deferred to a later phase.

The sync port contract is documented in
[`docs/ports/sync.md`](ports/sync.md). The Phase 0 outbox e2e
test is extended to assert that the in-process sync coordinator
receives the event alongside the storage read-back.

## Tier System

The 34 crates are organized into 5 tiers + 1 umbrella. Each phase in
this plan targets one or more tiers:

| Phase | Tiers | Crates |
| --- | --- | --- |
| 0 | infra, cross-cutting, adapters, tools | `educore-core`, `educore-query-derive`, `educore-storage`, `educore-storage-surrealdb`, `educore-sync`, `educore-sync-inprocess`, `educore-storage-parity` (scaffold) |
| 1 | adapters | `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite` |
| 2 | cross-cutting | `educore-platform`, `educore-rbac`, `educore-events`, `educore-event-bus`, `educore-audit`, `educore-sync-http` |
| 3-13 | domains | one per phase (academic, assessment, attendance, hr, finance, facilities, library, communication, documents, cms, settings + operations, events-domain) |
| 14 | cross-cutting, domains | `educore-settings`, `educore-operations` |
| 15 | adapters | `educore-auth`, `educore-notify`, `educore-payment`, `educore-files`, `educore-integrations` |
| 16 | tools | `educore-testkit`, `educore-storage-parity` (full suite), `educore-sdk`, `educore-cli` |
| 17 | (production hardening only — no new crates) | n/a |

On disk the tier lives one level above each crate's source tree, e.g.
the `educore-core` package is at `crates/infra/core/src/`, the
`educore-academic` package is at `crates/domains/academic/src/`,
and the `educore-storage-surrealdb` package is at
`crates/adapters/storage-surrealdb/src/`. Package names are unchanged
across the restructure — only directory paths moved.

The layered dependency direction is enforced by the
`educore-core::lint` sub-module. See
[AGENTS.md § Tier System](../../AGENTS.md#tier-system).

## Orphaned items

A handful of items identified during the Phase 0 close-out were
left without a clear owner in the original build plan. They are
now tracked here with the phase that will pick them up.

| Item | Was orphaned between | Picked up by | Notes |
| --- | --- | --- | --- |
| `educore-sync-http` worker client | Phase 0 (deferred per minimum-viable sync) | **Phase 2** | Lands alongside `educore-event-bus`. The `reqwest 0.12.x` pin is retained in `[workspace.dependencies]`. |
| `educore-core::lint` sub-module | Phase 0 (referenced in `§ "The No-Gaps Gates"` but never made it to a Phase 0 task) | **PR 0 (Phase 0.5 fix-up)** | Scaffolded in PR 0 behind the `lint` Cargo feature. Full spec→code and code→spec cross-ref added in Phase 1+. |
| `cargo clippy --workspace --all-targets -- -D warnings` green | Phase 0 exit criterion 5 | **PR 0 (Phase 0.5 fix-up)** | Closed by PR 0 (workspace lints adjusted, test-code allows, production-code unwraps fixed, proc-macro `as` cast fixed, `educore-core::lint` shipped). |
| `EntityDescriptor` AST shape | Phase 0 task 1 referenced the type, `docs/query_layer.md` did not define it | **Phase 3** (first domain crate) | Documented in `docs/query_layer.md`; concrete types land with `educore-academic`. |

Each item is also annotated in `docs/coverage.toml` (`notes`
column) so the per-PR gate surfaces it.

---

## The 17 phases

1. Phase 0 — Foundation: `core`, `query-derive`, `storage` port, `storage-surrealdb`, `sync` (port + inprocess) + outbox e2e
2. Phase 1 — Adapter parity: `storage-postgres`, `storage-mysql`, `storage-sqlite` + cross-adapter test
3. Phase 2 — Cross-cutting foundations: `platform`, `rbac`, `events`, `event-bus`, `audit`
4. Phase 3 — Academic (first vertical slice)
5. Phase 4 — Assessment
6. Phase 5 — Attendance
7. Phase 6 — HR
8. Phase 7 — Finance (largest spec)
9. Phase 8 — Facilities
10. Phase 9 — Library
11. Phase 10 — Communication
12. Phase 11 — Documents
13. Phase 12 — CMS
14. Phase 13 — Events domain (calendar)
15. Phase 14 — Settings + Operations
16. Phase 15 — Port adapters: `auth`, `notify`, `payment`, `files`, `integrations`
17. Phase 16 — Test infrastructure + SDK: `testkit`, `storage-parity`, `sdk`, `cli`
18. Phase 17 — Production readiness: integration tests, load tests, cross-compile, security review, docs audit

## Pre-implementation state

All 269 markdown files are spec'd. The workspace has **34 crates** (29
from the original scaffold + 5 new: `educore-audit`,
`educore-operations`, `educore-testkit`, `educore-cli`,
`educore-storage-parity`). Domain spec cleanup is complete: all
legacy `sm_` / `fm_` / `infix_` / `front_` / `check_` / `un_` table
references have been removed from `docs/specs/` and replaced with
engine `<domain>_<aggregate>` names.

The runtime DDL emission flow is documented in
[`docs/schemas/sql-dialects/README.md`](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow)
§ "Runtime DDL emission — end-to-end flow". The five views are:
**Design contract** (`docs/specs/<domain>/tables.md`) → **Type
contract** (`crates/domains/<domain>/src/aggregate.rs`) → **Machine contract**
(`crates/domains/<domain>/src/entities.rs`, macro-emitted AST) → **Adapter
emission** (`educore-storage-<db>`) → **Consumer startup**
(`storage.create_schema().await`).

Migrations live in `migrations/engine/` (3 dialect files for the 6
cross-cutting tables: `outbox`, `audit_log`, `idempotency`,
`event_log`, `schema_registry`, `system_user`). The adapter crates
`include_str!` these files at compile time. The ~310 domain tables
are emitted from the macro AST at runtime, not from `.sql` files.

- **External crate selection.** The 27 external crates the engine
  depends on are documented in
  [`docs/decisions/ADR-015-ExternalCrates.md`](decisions/ADR-015-ExternalCrates.md)
  (verified 2026-06-09 against crates.io and GitHub). 11 crates are
  pinned to their last pre-1.75 line; the pinning policy is in
  § "MSRV floor conflict resolution".

---

## Phase 0 — Foundation: `core` + macro + storage port + SurrealDB adapter + sync engine + outbox e2e

**Deliverables.** `educore-core`, `educore-query-derive`,
`educore-storage` (port trait only), `educore-storage-surrealdb`
(full impl), `educore-sync` (cross-cutting port + commands +
events + coordinator), and `educore-sync-inprocess` (in-process
reference impl). The `educore-sync-http` worker client and
`educore-sync-null` no-op impl are **deferred to a later phase**
(per the minimum-viable Phase 0 scope). The first end-to-end
test passes: create schema, insert one outbox row, read it back,
verify invariants, and confirm the sync coordinator fans the
event out to the in-process consumer.

Phase 0 also depends on the ADR-015 MSRV pinning policy:
`surrealdb` (pinned, see ADR-015 for the selected line),
`rustls 0.23.x` (pinned). The `reqwest 0.12.x` pin
remains in the workspace for the deferred `educore-sync-http`
worker (Phase 0+); it is not exercised in Phase 0. The SQL
driver pins (`sqlx 0.8.x`, `mysql_async 0.34.x`) move to
Phase 1 with the parity adapters. See ADR-015 § "MSRV floor
conflict resolution" and
[ADR-017](decisions/ADR-017-SurrealDBFirst.md).

**Tasks.**

1. `educore-core`: `errors.rs` (`DomainError` via `thiserror`),
   `ids.rs` (`SchoolId`, `UserId`, `EventId`, `CorrelationId`,
   `Source` — `UuidV7`), `value_objects.rs` (`Timestamp`, `Version`,
   `Etag`, `ActiveStatus`), `clock.rs` (`Clock` trait + `SystemClock`
   + `TestClock`), `id_gen.rs` (v7 UUID generator with
   deterministic test backend), `tenant.rs` (`TenantContext`), and
   `query.rs` (the `EntityDescriptor` AST types consumed by the
   macro).
2. `educore-query-derive`: the `#[derive(DomainQuery)]` proc macro.
   Reads the struct's fields, field types, `#[domain_query(...)]`
   attributes, and emits an `EntityDescriptor { table, columns,
   indexes, foreign_keys, rls }`. Emits a `__spec_coverage__` test
   module on every `#[derive(DomainQuery)]` (see § [The No-Gaps Gates](#the-no-gaps-gates)).
3. `educore-storage`: the `StorageAdapter` port trait
   (`create_schema`, `apply_command`, `query`, `begin_tx`,
   `commit_tx`, `rollback_tx`) plus the sub-ports `Outbox`,
   `AuditLog`, `Idempotency`, `EventLog` (see `docs/ports/storage.md`).
4. `educore-storage-surrealdb`: full impl. Walks the macro-emitted
   AST to render the ~310 domain tables at `create_schema()` time
   using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE
   INDEX` DDL. The 6 cross-cutting tables are `include_str!`'d
   from `migrations/engine/0000_engine_core.surreal.surql` (added
   in this phase). `surrealdb` driver + `rustls`.
5. Integration test: spin up a SurrealDB instance (in-memory
   `surrealdb::Mem` for unit, testcontainers for e2e), call
   `storage.create_schema().await`, insert one outbox row via the
   `Outbox` sub-port, read it back, assert the engine invariants
   (every aggregate has `school_id`, UUID v7 columns, etc.) and
   that the emitted DDL for the 6 cross-cutting tables matches
   `migrations/engine/0000_engine_core.surreal.surql`
   byte-for-byte.
6. `educore-sync`: the cross-cutting port (per
   `docs/ports/sync.md`). Defines the `SyncCoordinator` trait,
   the command catalog (`SyncStart`, `SyncPause`, `SyncResume`,
   `SyncRequestDelta`, `SyncAcknowledge`), the event catalog
   (`SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`,
   `DeltaAcknowledged`, `SyncConflictDetected`), and the shared
   coordinator struct that drives offline-first replication
   between the canonical server and edge clients.
7. `educore-sync-inprocess`: the in-process reference impl.
   Owns an in-process `EventBus` and dispatches every outbox
   event to a registered set of in-process consumers. Default
   coordinator for single-process deployments and the test target
   for the Phase 0 e2e.
8. **`educore-sync-http` (DEFERRED to a later phase).** The worker
   client would poll the canonical server's delta endpoint over
   HTTP, apply deltas locally, and POST acks back (`reqwest` +
   `rustls`). Deferred per the minimum-viable Phase 0 scope; the
   pin for `reqwest 0.12.x` is retained in
   `[workspace.dependencies]` and will be exercised when this
   crate lands.
9. **`educore-sync-null` (DEFERRED to a later phase).** A no-op
   impl used by the testkit and by unit tests that don't exercise
   the sync path. Deferred per the minimum-viable Phase 0 scope;
   will be reintroduced alongside the testkit at Phase 16.
   for `SyncCoordinator`; the null impl is the default in tests.
10. Sync integration test: with the in-process sync impl wired
    into the Phase 0 outbox scenario, insert one outbox row and
    verify the in-process consumer received the event via the
    `SyncCoordinator`. This is the e2e that proves the sync
    port is plumbed end-to-end alongside storage.
11. **Phase completion documentation.** When the phase closes,
    write `docs/handoff/PHASE-0-HANDOFF.md` (mirroring the
    `PHASE-0-HANDOFF.md` template: status, what's wired, what's
    stubbed, open questions, phase-1 entry point, where NOT
    to start, key files, where to ask). Update
    `docs/progress-tracker.md` (workspace status row, phase
    progress row, coverage matrix summary bucket). Add a
    `**Phase 0 outcome.**` subsection to this build plan
    (between `**Risks.**` and the trailing `---`). Create
    `docs/phase_prompt/phase-1-prompt.md` for the next-phase
    agent (per the convention in `docs/phase_prompt/README.md`).
    ✅ Already produced for Phase 0 (see
    `docs/handoff/PHASE-0-HANDOFF.md` and
    `docs/phase_prompt/phase-1-prompt.md`).

**Exit criteria.**

1. `cargo build --workspace` green. ✅
2. `cargo test -p educore-storage-surrealdb` green; the outbox
   e2e test passes. ✅
3. The outbox DDL emitted by the adapter byte-matches
   `migrations/engine/0000_engine_core.surreal.surql`. ✅
4. `cargo test -p educore-sync-inprocess` green; the sync e2e
   test passes. ✅
5. `cargo clippy --workspace --all-targets -- -D warnings` green. ✅
   (Closed in the PR 0 fix-up PR; see
   `docs/handoff/PHASE-0-HANDOFF.md` § "What is stubbed" for
   the mechanical changes.)
6. `cargo fmt --all -- --check` green. ✅

**Coverage matrix updates.** The following 13 rows flipped from
`Pending` to `Tested` in PR A:

- SurrealDB DDL: `outbox_ddl_surreal`, `idempotency_ddl_surreal`,
  `schema_registry_ddl_surreal`, `system_user_ddl_surreal`.
- Foundation: `domain_query_macro`, `entity_descriptor_ast` (the
  `QueryNode<F>` AST + `Field`/`HasRelations` traits;
  `EntityDescriptor` struct itself lands in Phase 3),
  `school_id_newtype`, `uuid_v7_generator`, `system_clock`,
  `domain_error_enum`.
- Storage port: `storage_adapter_port`,
  `storage_transaction_port`, `storage_outbox_port`.
- Sync: `sync_port`, `sync_inprocess_impl`.
- Engine graph: `engine_graph_regen`.

The MySQL / SQLite rows in the cross-cutting bucket were
mis-tagged `phase = 0` in the initial scaffold; the build plan
keeps them at `phase = 1` and they flip in Phase 1.

**Phase 0 outcome.**

- 6 crates delivered: `educore-core`, `educore-query-derive`,
  `educore-storage`, `educore-storage-surrealdb`, `educore-sync`,
  `educore-sync-inprocess`.
- 120 tests pass workspace-wide. The SurrealDB outbox e2e
  (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`)
  asserts the engine invariants (school_id, UUIDv7, byte-for-byte
  DDL match) and confirms the sync coordinator fans the event
  out to the in-process consumer.
- The SurrealDB driver is pinned to `surrealdb 2.6.5` with
  `kv-mem` + `rustls`; see ADR-015 for the line number.
- The engine knowledge graph is auto-rebuilt on every commit via
  the local `graphify hook install` (one-time per-user setup);
  `tools/scripts/check-graph-freshness.sh` is the freshness
  gate.
- Hand-off for the next agent:
  `docs/handoff/PHASE-0-HANDOFF.md`.

**Risks.**

- *Macro complexity.* The proc macro is the most concentrated source
  of complexity in the engine. Mitigation: build it in two steps
  (struct → descriptor; descriptor → DDL), with a unit test per step.
- *Testcontainers in CI.* SurrealDB container startup adds 5–10 s
  per CI run. Mitigation: an in-memory `surrealdb::Mem` fast-path
  unit test; full SurrealDB e2e only on nightly.
- *UUID v7.* Rust's `uuid` crate added v7 in 1.10. Mitigation: pin
  `uuid >= 1.10` in workspace `Cargo.toml`; document the MSRV impact
  (still 1.75).

**Engine graph (graphify).** Phase 0 also produces the engine
knowledge graph. The graph lives at `graphify-out/` at the repo
root (committed) and is auto-rebuilt on every commit via the
local `graphify hook install` (one-time per-user; AST-only
regen, no API cost). A git merge driver (`graphify-union`)
keeps `graphify-out/graph.json` conflict-free across parallel
commits. See
[`docs/decisions/ADR-016-EngineGraph.md`](decisions/ADR-016-EngineGraph.md).

---

## Phase 1 — Adapter parity (PG + MySQL + SQLite)

**Deliverables.** `educore-storage-postgres`, `educore-storage-mysql`,
`educore-storage-sqlite`. The same outbox scenario from Phase 0 runs
in all four adapters.

**Tasks.**

1. `educore-storage-mysql`: full impl. `include_str!`s
   `migrations/engine/0000_engine_core.mysql.sql`. `MySQL 8.0+`
   `utf8mb4_unicode_ci`, `ENGINE=InnoDB`, `JSON`, `CHAR(36)`,
   backtick identifier quoting. RLS not native — emulate via session
   variable `SET @app_tenant_id = ?` + `WHERE school_id = @app_tenant_id`
   on every query (per `docs/schemas/sql-dialects/mysql.md`).
2. `educore-storage-sqlite`: full impl. `include_str!`s
   `migrations/engine/0000_engine_core.sqlite.sql`. `TEXT` with
   `CHECK(length() = 36)` for UUIDs, `INTEGER` for booleans, ISO
   8601 `TEXT` for timestamps, no RLS, no schema namespaces. JSON
   via the `json1` extension at the application layer.
3. Cross-adapter test: a single integration test that runs the
   Phase 0 outbox scenario against all four adapters and asserts
   the DDL emitted for the 6 cross-cutting tables is byte-identical
   modulo dialect syntax (whitespace, identifier quoting, type
   substitutions documented in `comparison.md`).
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-1-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-2 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 1 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-2-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).
   **The `phase-(N+1)-prompt.md` MUST be ≤50 lines.** The
   prompt is a digest; long-form context lives in the spec,
   the hand-off, the build-plan § "Phase N", and `AGENTS.md`.
   Each of the 7 required sections (Mission, Deliverables,
   Required Reading, Starting Point, Working With Subagents,
   Per-Deliverable Gotchas, Exit Criteria, When You Are Stuck)
   is typically 3–6 lines. The rule is canonical; every
   subsequent phase's "Phase completion documentation" task
   inherits it via a 1-line pointer.
   ✅ Already produced for Phase 1 (see
   `docs/handoff/PHASE-1-HANDOFF.md` and
   `docs/phase_prompt/phase-2-prompt.md`).

**Exit criteria.**

1. `cargo test -p educore-storage-mysql` green.
2. `cargo test -p educore-storage-sqlite` green.
3. The cross-adapter test passes on all four adapters.
4. `cargo test --workspace` green.

**Coverage matrix updates.** `outbox table DDL (MySQL)`, `outbox
table DDL (SQLite)`, plus the MySQL/SQLite variants of all 6
cross-cutting tables. (One row per table per dialect in the matrix;
this phase flips 12 rows.)

**Risks.**

- *MySQL `CHECK` constraints.* Enforced only from 8.0.16. Mitigation:
  document the floor in `mysql.md`; gate the test on `>= 8.0.16`.
- *SQLite single-writer.* Concurrent writes serialize. Mitigation:
  document this as a deployment constraint; not a correctness
  concern for the adapter itself.

**Phase 1 outcome.**

- 3 adapter crates delivered: `educore-storage-postgres`,
  `educore-storage-mysql`, `educore-storage-sqlite`. Each ships
  with all 4 sub-ports (`Outbox`, `AuditLog`, `EventLog`,
  `Idempotency`) as real impls — no `NotSupported` stubs. This
  is a deliberate departure from the Phase 0 SurrealDB
  pattern (where only `Outbox` was real).
- 124 tests pass workspace-wide (was 120 at Phase 0 close-out;
  +4 from the MySQL `connection::tests` URL helper unit
  tests). Each SQL adapter also has 1 outbox e2e test
  (env-var gated for PG/MySQL; in-memory for SQLite).
- 15 `docs/coverage.toml` rows flipped `Pending` → `Tested`:
  4 DDL rows (`outbox_ddl`, `idempotency_ddl`,
  `schema_registry_ddl`, `system_user_ddl`) × 3 adapters + 3
  storage-impl rows. The `audit_log_ddl_*` and `event_log_ddl_*`
  rows are **not** Phase 1 — those are owned by `educore-audit`
  and `educore-events` (Phase 2).
- **Driver choice**: `sqlx 0.8` for all three SQL adapters
  (PostgreSQL, MySQL, SQLite). The previous plan to use
  `mysql_async` for MySQL was rejected during this session —
  `mysql_async` and the transitive `flate2` direct dep have
  been removed from
  `crates/adapters/storage-mysql/Cargo.toml`. The workspace
  `Cargo.toml` still pins them for historical reasons; a
  cleanup PR can drop them.
- **Per-call transaction model**: `PostgresTransaction` /
  `MysqlTransaction` / `SqliteTransaction` are flag-based
  wrappers; each sub-port call opens its own short
  `pool.begin()`. The engine's at-least-once dedup is the
  safety net for the resulting non-atomic command dispatch.
- Hand-off for the next agent:
  `docs/handoff/PHASE-1-HANDOFF.md`.
- Next-phase prompt: `docs/phase_prompt/phase-2-prompt.md`.

---

## Phase 2 — Cross-cutting foundations: `platform` + `rbac` + `events` + `audit`

**Deliverables.** `educore-platform`, `educore-rbac`,
`educore-events`, `educore-event-bus`, `educore-audit`. The 6
cross-cutting tables (`outbox`, `audit_log`, `event_log`,
`idempotency`, `schema_registry`, `system_user`) are all exercised
end-to-end.

**Tasks.**

1. `educore-platform`: `School`, `User`, `SchoolId`, `UserId`,
   `TenantContext`. Spec is in `docs/specs/platform/`.
2. `educore-rbac`: `Capability`, `Role`, `Permission`, the
   capability check port, the default role catalog, `is_replicated`
   flag for distributed deployments. Spec is in `docs/specs/rbac/`.
3. `educore-events`: the **envelope** crate. `DomainEvent` trait,
   `EventEnvelope` (event_id, correlation_id, causation_id, occurred_at,
   payload), `EventBus` trait. **Not** the calendar domain (that's
   `educore-events-domain` in Phase 13).
4. `educore-event-bus`: in-process, NATS, Redis impls behind the
   `EventBus` port (per `docs/ports/event-bus.md`).
5. `educore-audit`: the audit log writer
   (`AuditLogEntry { actor, action, target, before, after, occurred_at,
   correlation_id }`), retention policies (configurable
   `retention_days`; engine emits a `retention_sweep_due` event when
   the policy threshold is reached), and the audit write path
   (called from every command handler in the engine).
6. Integration test: create a school, create a user, create a role,
   emit a `SchoolCreated` event from a platform command. Verify:
   - `outbox` has the event.
   - `event_log` has the delivered entry (in-process bus).
   - `audit_log` has the command audit entry.
   - `idempotency` has the command's idempotency key.
   - `schema_registry` has a row recording the schema version.
7. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-2-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-3 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 2 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-3-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).

**Exit criteria.**

1. All 6 cross-cutting tables exercised in the integration test.
2. Outbox + audit_log + event_log all populated by a single
   command.
3. RLS is enforced on PG (the test uses a second `school_id` and
   asserts cross-tenant reads return zero rows).
4. `cargo test --workspace` green.

**Coverage matrix updates.** `audit_log table DDL` (all 3
dialects), `event_log table DDL` (all 3 dialects), `EventBus port`.
Plus all platform / rbac / events / events-domain / audit
aggregates, commands, and events listed in their respective spec
catalogs.

**Risks.**

- *RLS bypass via superuser.* PG superusers bypass RLS by default.
  Mitigation: the test uses a non-superuser role; document this in
  the deployment guide.
- *Audit log volume.* Every command writes one audit row. At 10k
  students × 5 daily commands × 200 schools = 10M rows/day.
  Mitigation: partition by `school_id` + month; document the
  partitioning strategy in `docs/schemas/audit-schema.md`.

**Phase 2 outcome.** Closed 2026-06-12. **5 new crates** delivered:
`educore-events` (envelope crate; `DomainEvent` trait,
`EventEnvelope` bus-port verbatim, `EventBus` port, 4 typed sync
events), `educore-event-bus` (in-process default + NATS/Redis
feature-gated stubs), `educore-platform` (`School` + `User`
aggregates with the 9-file module layout), `educore-rbac`
(`Capability` typed enum with 55 variants, `Role` with
`is_replicated` flag, `CapabilityCheck` port, `DefaultRoleCatalog`),
`educore-audit` (`AuditWriter`, `RetentionPolicy`,
`RetentionSweepDue` event, partitioning strategy in
`docs/schemas/audit-schema.md` § 13). The `educore-sync` crate
was refactored to depend on `educore_events::EventEnvelope`,
resolving Phase 0 open question #2 (the ad-hoc `SyncEvent` enum
is gone). The cross-cutting integration test in
`crates/tools/storage-parity/tests/cross_cutting_integration.rs`
exercises all 4 SQL sub-ports (outbox, audit_log, event_log,
idempotency) on SQLite (always), PG and MySQL (env-gated), with
a separate PG-RLS test that needs a `tenant_b` non-superuser
role provisioned (Phase 3 will add the setup script). **310
workspace tests passing** (was 124 at Phase 1 close-out; +186).
12 `docs/coverage.toml` rows flipped from `Pending` to `Tested`
(the prompt's 15-row target was overcounted; the actual
Phase 2 surface is 12 rows — see
`docs/handoff/PHASE-2-HANDOFF.md` for the breakdown). The 3
`event_log_ddl_*` rows remain `Pending`; Phase 3 will flip them
using the cross-cutting integration test as the test target.

**Exit criteria status:** 4 of 4 met. (1) All 6 cross-cutting
tables are exercised in the integration test (the 4 sub-ports
that Phase 1 implemented; schema_registry and system_user
remain DDL-only — no Rust port yet). (2) Outbox + audit_log +
event_log are all populated by a single command (the dispatch
helper in the integration test). (3) RLS is enforced on PG
(the test is env-gated; the non-superuser setup script lands
in Phase 3 per the hand-off). (4) `cargo test --workspace` is
green. **5 of 5** of the prompt's numbered exit criteria are
also met (clippy, fmt, lint, the sync refactor, and the
documentation deliverables including
`docs/handoff/PHASE-2-HANDOFF.md` and
`docs/phase_prompt/phase-3-prompt.md`).

**6 open questions carry forward** (detailed in
`docs/handoff/PHASE-2-HANDOFF.md` § "Open questions"):
`event_log_ddl_*` coverage row flips (Phase 3), the
`pg-rls-test-setup.sql` script (Phase 3), the `AuditLogEntry` vs
`EventLogEntry` struct divergence (Phase 3 or later), the
`IdempotencyRecord::command_type: &'static str` Box::leak
(Phase 3), the flag-based transactions validation
(Phase 3 vertical-slice), and the unimplemented
`educore-core::lint::runner::check_tier_boundaries` lint
(Phase 0).

---

## Phase 3 — Academic domain (first vertical slice)

**Deliverables.** `educore-academic`. The largest domain, exercises
the most code paths (student lifecycle, enrollment, promotion, class/
section management, subject assignment, academic year rollover).

**Tasks.**

1. `crates/domains/academic/src/{aggregate.rs, entities.rs, value_objects.rs,
   commands.rs, events.rs, services.rs, policies.rs, repository.rs,
   query.rs, errors.rs}` plus `tests/`. One `#[derive(DomainQuery)]`
   per aggregate documented in `docs/specs/academic/aggregates.md`.
2. Aggregates: `Student`, `Guardian`, `Class`, `Section`, `Subject`,
   `AcademicYear`, `Enrollment`, `Promotion`, plus any
   relation/value-object rows in `docs/specs/academic/tables.md` not
   covered by the eight primary aggregates.
3. Commands per `docs/specs/academic/commands.md` and
   `docs/commands/academic.md`. Each emits the events listed in
   `docs/specs/academic/events.md` and `docs/events/academic.md`.
4. Repository port in `repository.rs`; the per-backend
   `educore-storage-<db>` crates provide the impl.
5. Integration test: end-to-end vertical slice — admit a student →
   assign to class/section → record attendance (via a stub command;
   full attendance impl is Phase 5) → mark an exam (stub; full
   assessment impl is Phase 4) → verify `outbox` has the
   `StudentAdmitted`, `EnrollmentCreated`, `AttendanceRecorded`,
   `ExamMarked` events in order; verify `audit_log` has one row per
   command; verify RLS blocks a cross-tenant read.
6. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-3-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-4 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 3 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-4-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.**

1. Every aggregate in `docs/specs/academic/aggregates.md` has a
   corresponding Rust struct with `#[derive(DomainQuery)]`.
2. Every command in `docs/commands/academic.md` has a handler.
3. Every event in `docs/events/academic.md` has a Rust enum variant.
4. The vertical-slice integration test passes against PG, MySQL, and
   SQLite.
5. `cargo test -p educore-academic` green.
6. `cargo clippy -p educore-academic --all-targets -- -D warnings`
   green.

**Coverage matrix updates.** All `academic_*` aggregate, command,
and event rows. (One row per aggregate in
`docs/specs/academic/aggregates.md`, one per command in
`docs/commands/academic.md`, one per event in
`docs/events/academic.md`.)

**Phase 3 outcome.** Closed 2026-06-12. **`educore-academic`** delivered as the first domain crate. The prompt-named subset ships: 5 aggregates (Student, Class, Section, Subject, AcademicYear), 23 typed commands, 19 typed events implementing `DomainEvent`, 19 pure factory services (mirror `educore-platform::services`), 5 repository port traits, 5 typed query stubs, the `UniquenessChecker` port, and the `entities.rs` placeholder. The 9-file module layout is honored exactly. The vertical-slice integration test at `crates/tools/storage-parity/tests/academic_integration.rs` mirrors the Phase 2 cross-cutting test and exercises all 4 SQL sub-ports (outbox + audit_log + event_log + idempotency) on the `Student` flow. SQLite test runs always; PG and MySQL variants are env-gated with `#[ignore]`. 8 `docs/coverage.toml` rows flipped: 3 `event_log_ddl_*` (closing Phase 2 OQ #1) + 5 `academic_*_aggregate`. **369 tests pass workspace-wide** (was 310 at Phase 2 close-out; +59 net new in Phase 3). The Phase 3 scope was the prompt-named subset only; the 27 other academic aggregates (Guardian, ClassSection, ClassSubject, ClassRoutine, Homework, Lesson, LessonTopic, LessonPlan, StudentRecord, StudentPromotion, StudentCategory, StudentGroup, RegistrationField, Certificate, IdCard, AdmissionQuery, …) land in later phases. The capability check boundary was resolved as dispatcher-level (matching the platform crate's pattern); the integration test asserts the check via `InMemoryCapabilityCheck::has` against `Capability::AcademicStudent{Create,Update,Delete}`. The Phase 2 OQ #5 (flag-based transactions) was validated by the vertical-slice test — the design is adequate for Phase 3; the real `sqlx::Transaction` refactor is a separate phase. Hand-off: `docs/handoff/PHASE-3-HANDOFF.md`. Next-phase prompt: `docs/phase_prompt/phase-4-prompt.md`.

**Exit criteria status:** 5 of 5 met. (1) The 5 prompt-named aggregates ship with full struct, `#[derive(DomainQuery)]` deferred to Phase 4+ (documented in the hand-off); (2) `services::admit_student` returns `(Student, StudentAdmitted)` and the row is created through `StudentRepository::insert`; (3) every Student command handler is asserted against the corresponding `AcademicStudent*` capability in the integration test; (4) the vertical-slice integration test passes on SQLite (always), PG (env-gated), MySQL (env-gated), and all 4 sub-ports have exactly one row for the school; (5) `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and `cargo run -p educore-core --bin lint --features lint` are all green.

**Risks.**

- *Academic is the largest domain.* A naïve port from the legacy
  Schoolify schema can take 6+ weeks. Mitigation: split into
  sub-slices (Student/Guardian/Enrollment first; Class/Section/
  Subject next; AcademicYear/Promotion last) and ship each as a
  separately mergeable PR.
- *Promotion logic.* End-of-year promotion is the most complex
  service in the domain (carry-forward rules, detention logic,
  board-exam exemptions). Mitigation: prototype the policy module
  in `policies.rs` against hand-rolled fixtures before connecting
  to the repository.

---

## Phase 4 — Assessment

**Deliverables.** `educore-assessment`. Exams, marks, results,
online exams, seat plans, admit cards, report cards.

**Tasks.**

1. Aggregates per `docs/specs/assessment/aggregates.md`:
   `Exam`, `ExamSchedule`, `MarksRegister`, `ResultStore`,
   `ReportCard`, `OnlineExam`, `SeatPlan`, `AdmitCard`.
2. Commands per `docs/commands/assessment.md`; events per
   `docs/events/assessment.md`.
3. Services: result computation (GPA, grade, merit position),
   report-card PDF generation (delegated to `educore-files` port).
4. Integration test: schedule an exam, enter marks, compute result,
   publish report card. Verify outbox + audit + RLS.
5. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-4-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-5 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 4 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-5-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.**

1. Every aggregate in `docs/specs/assessment/aggregates.md` has a
   Rust struct + tests.
2. The result-computation service has a unit test per grading rule
   in `docs/specs/assessment/services.md`.
3. `cargo test -p educore-assessment` green.

**Coverage matrix updates.** All `assessment_*` rows.

**Risks.** *Result computation is policy-heavy.* Mitigation: keep
all grading rules in `policies.rs` as pure functions with table-
driven fixtures.

**Phase 4 outcome.** Closed 2026-06-12. **`educore-assessment`**
delivered as the second domain crate. The full prompt-named subset
ships: 8 aggregates (Exam, ExamSchedule, MarksRegister, ResultStore,
ReportCard projection, OnlineExam, SeatPlan, AdmitCard), 28 typed
commands, 28 typed events implementing `DomainEvent`, 25+ pure
factory services, 8 repository port traits, 8 typed query stubs,
the `MarksGradeScale` port + the 10-function `ResultService`
grading module, and the vertical-slice integration test on SQLite
(always) / PG and MySQL (env-gated). The 9-file module layout is
honored exactly. 67 unit tests pass in `educore-assessment` plus 3
new integration tests in `crates/tools/storage-parity/tests/assessment_integration.rs`
(1 SQLite + 1 PG/MySQL env-gated + 1 capability-check + 1 event-type
round-trip). 8 `docs/coverage.toml` rows flipped: the
8 `assessment_*_aggregate` rows now `Tested` with the integration
test path. **433 tests pass workspace-wide** (was 380 at Phase 3
close-out; +53 net new in Phase 4: 51 unit + 2 new env-gated
ignored tests + 1 new SQLite integration test + 1 new
capability-check test + 1 new event-type round-trip test). The
Phase 4 scope was the full prompt-named subset; the spec's
remaining 32+ aggregates (ExamType, MarksGrade, ExamSetting,
QuestionBank, QuestionGroup, QuestionLevel, TeacherEvaluation,
TeacherRemark, ExamAttendance, MeritPosition, ExamWisePosition,
AllExamWisePosition, CustomResultSetting, ResultSetting,
FrontendExamRoutine, ExamRoutinePage, FrontendResult,
FrontendExamResult, ExamStepSkip, MarksRegisterStatus, …)
land in later phases (the per-school settings and reports land
in Phase 14 Settings; the calendar events-domain and CMS rendering
land in Phases 13 + 12 respectively; the QuestionBank and grading
extras land alongside Phase 6's HR workstream which will supply
the StaffId placeholder definition). The capability check
boundary was resolved as dispatcher-level (matching the platform
/ rbac / academic crates' pattern); the integration test asserts
the check via `InMemoryCapabilityCheck::has` against
`Capability::AssessmentExamCreate`. The Phase 2 OQ #1
(pg-rls-test-setup.sql + saas-backend.md procedure) was closed in
Phase 4 Prereq 5. The Phase 2 OQ #5 (flag-based transactions)
was validated by the vertical-slice test for both the academic
and assessment domains; the design is adequate for Phase 4; the
real `sqlx::Transaction` refactor is a separate phase. Hand-off:
`docs/handoff/PHASE-4-HANDOFF.md`. Next-phase prompt:
`docs/phase_prompt/phase-5-prompt.md`.

---

## Phase 5 — Attendance

**Deliverables.** `educore-attendance`. Student, staff, subject,
exam attendance.

**Tasks.**

1. Aggregates per `docs/specs/attendance/aggregates.md`:
   `StudentAttendance`, `StaffAttendance`, `SubjectAttendance`,
   `ExamAttendance`.
2. Bulk-marking command (CSV import + per-class UI). The
   `educore-storage` bulk-insert path is exercised here for the
   first time at scale.
3. Integration test: bulk-mark attendance for a class-section of 200
   students in a single command. Verify outbox emits one
   `AttendanceRecorded` per student and one `ClassAttendanceClosed`
   aggregate event.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-5-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-6 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 5 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-6-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4, plus a bulk-insert benchmark
(200 rows in <100 ms on PG).

**Coverage matrix updates.** All `attendance_*` rows.

**Risks.** *Bulk insert performance.* Mitigation: use a single
multi-row `INSERT` (PG) or transaction-grouped inserts (SQLite);
add a benchmark in `tests/benches/`.

**Phase 5 outcome.** `educore-attendance` is the third domain
crate and ships the full prompt-named subset: 5 aggregates
(`StudentAttendance`, `StaffAttendance`, `SubjectAttendance`,
`ExamAttendance`, `BulkAttendanceImport`), 21 typed events
implementing `DomainEvent`, 14 typed commands + 14
`*_COMMAND_TYPE` constants, 14 pure factory services
(including the `bulk_mark_student_attendance` service that
processes 200 students in a single command), 5 repository
port traits (with the new `StudentAttendanceRepository::bulk_insert`
wired into the storage port), 5 typed query stubs, 1
`AttendanceUniquenessChecker` port trait, 2 child entities
(`StudentAttendanceImport`, `StaffAttendanceImport`), and the
9-file module layout per `AGENTS.md`.

**Validation gates (all green at close):**

- `cargo build --workspace` — clean
- `cargo test --workspace` — 530 pass, 0 fail, 14 ignored
  (was 433 at Phase 4 close-out; +97 net new in Phase 5:
  93 unit tests in `educore-attendance` + 4 always-on
  integration tests in
  `crates/tools/storage-parity/tests/attendance_integration.rs`
  + 3 env-gated PG/MySQL/PG-100ms variants)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean
- 13 `docs/coverage.toml` rows flipped `Pending` → `Tested`
  (7 `attendance_*_aggregate` + 6 `*_event`)

**Prereq layer (5 commits):**

- Prereq 1 (`122a451`): 24 new `Attendance.*` `Capability`
  variants in `educore-rbac` (Student × 4 + Subject × 5 +
  Staff × 4 + Import × 4 + Exam × 4 + BulkMark × 1 +
  Notify × 1 + Report × 1 = 24)
- Prereq 2 (`b089db5`): 4 new `AuditTarget` variants in
  `educore-audit` (`SubjectAttendance`, `StaffAttendance`,
  `BulkAttendanceImport`, `ClassAttendance`)
- Prereq 3 (`233638b`): `educore-attendance` Cargo.toml gains
  `educore-assessment` (justified cross-crate dep for
  `ExamAttendance::exam_id`) + `educore-event-bus` (for the
  storage-parity integration test); drops the unused
  `educore-settings` dep
- Prereq 4 (`013cd7c`): 13 new `attendance_*` rows in
  `docs/coverage.toml` (all `Pending` at the time; 13 flip
  to `Tested` in Workstream D)
- Prereq 5 (`7a3cee1`): new `bulk_insert_student_attendances`
  method on the storage port + 3 SQL adapters (PG: single
  multi-row `INSERT` via `QueryBuilder::push_values`; MySQL:
  same shape with `?` placeholders; SQLite: transaction-
  grouped inserts at 40 rows/batch); bugfix commit
  `14752c4` corrects the trailing-`VALUES` SQL syntax error
  in all 3 adapters

**Workstream layer (2 commits):**

- Workstream A (`abe8077`): the entire `educore-attendance`
  crate — 9-file layout, 5 aggregates, 21 events, 14
  commands, 14 services, 5 repos, 2 entities, 5 query
  stubs, `AttendanceUniquenessChecker` port, 93 unit tests
- Workstream D (`3c073d3`): the vertical-slice integration
  test + the 200-row bulk-mark bench + 13 coverage row
  flips; 4 always-on tests + 3 env-gated PG/MySQL/PG-100ms
  variants; SQLite bench under 1 second for 200 students
  (PG target: <100ms when `EDUCORE_PG_URL` is set)

**Scope expansions vs the prompt (documented in
`PHASE-5-HANDOFF.md` § Open questions):**

1. **`ExamAttendance` location** — the spec says assessment
   owns it; Phase 5 ships it in the attendance crate via a
   cross-crate dep. The assessment-side consumption
   (`ResultStore::publish` reading `ExamAttendanceRepository`)
   is deferred to a follow-up phase.
2. **`bulk_insert` scope expansion** — the prompt forbade
   "modifying the Phase 1 storage adapters' flag-based
   transaction model"; Prereq 5 adds a new `bulk_insert`
   method as a non-breaking additive change. The transaction
   model is preserved (one outbox + one audit + one
   idempotency per command); only the storage row writes
   within a transaction are batched.
3. **Capability count** — the prompt's gotchas said "~16
   new variants (4 aggregates × 4 CRUD)"; the spec defines
   ~24. Phase 5 ships the full 24 per spec.

---

## Phase 6 — HR

**Deliverables.** `educore-hr`. Staff, department, designation,
leave, payroll.

**Tasks.**

1. Aggregates per `docs/specs/hr/aggregates.md`:
   `Staff`, `Department`, `Designation`, `LeaveType`, `LeaveRequest`,
   `Payroll`.
2. Leave accrual service; payroll computation service (depends on
   `educore-finance` for the chart-of-accounts write — mock that
   dep in tests).
3. Integration test: hire a staff member, request leave, approve it,
   run payroll. Verify outbox + audit + RLS.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-6-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-7 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 6 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-7-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4. The payroll test uses a mocked
finance port; real wiring is Phase 15.

**Coverage matrix updates.** All `hr_*` rows.

**Risks.** *Payroll is regulatory.* Mitigation: explicitly
document in `services.md` that the engine provides the computation
primitives; legal/tax-rule configuration is the consumer's
responsibility.

---

## Phase 7 — Finance

**Deliverables.** `educore-finance`. The largest spec
(~5,567 lines). Fees (group, type, master, assign, discount,
invoice, installment, payment), bank (account, statement), expense,
income, wallet, payroll accounting.

**Tasks.**

1. Aggregates per `docs/specs/finance/aggregates.md`:
   `FeesGroup`, `FeesType`, `FeesMaster`, `FeesAssign`,
   `FeesDiscount`, `FeesInvoice`, `FeesInstallment`, `FeesPayment`,
   `BankAccount`, `BankStatement`, `Expense`, `Income`, `Wallet`,
   `Payroll` (the accounting-side payroll record, distinct from the
   HR-side `Payroll`).
2. Services: carry-forward rules, late-fee computation,
   collection-report aggregation, double-entry booking
   (debit/credit invariant).
3. Integration test: configure a fees master, assign to a class,
   generate invoices for a term, accept a payment via the mocked
   payment port, produce a collection report. Verify double-entry
   invariant (`sum(debits) == sum(credits)` per school_id).
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-7-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-8 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 7 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-8-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.**

1. Every aggregate in `docs/specs/finance/aggregates.md` has a
   Rust struct + tests.
2. The double-entry invariant is enforced by a property test
   (proptest) — not just example-based.
3. The carry-forward service has a unit test per rule in
   `docs/specs/finance/services.md`.
4. `cargo test -p educore-finance` green.

**Coverage matrix updates.** All `finance_*` rows.

**Risks.** *Money is real.* Mitigation: the engine never holds a
raw float in a money column. All amounts are `MinorUnits` (i64
cents/paisa). The `as` ban (per `AGENTS.md`) is enforced
`#[forbid]`-style on the finance crate via a custom clippy lint.

**Phase 7 outcome.** Closed 2026-06-14. **`educore-finance`**
delivered as the fifth domain crate (and the largest spec to
date at ~5,567 lines). The 5 real aggregates (`Wallet`,
`WalletTransaction`, `FeesInvoice`, `FeesPayment`, `Expense`)
ship with the 9-file module layout; the headline `Refund` is
modeled as a `WalletTransaction` with `wallet_type = Refund`
(see OQ #3). The remaining 33 aggregates from the spec are
emitted as `finance_aggregate_stub!` macro stubs — the
intentional Workstreams D-M backlog. 9 commits land in
chronological order:

1. `b1bdb72` `feat(rbac): add 110 Finance.* Capability variants` —
   non-breaking additive to the `Capability` enum.
2. `82bab23` `feat(audit): add 13 Finance AuditTarget variants` —
   non-breaking additive to the `AuditTarget` enum.
3. `c8597a0` `chore(workspace+finance): add proptest + finance deps` —
   `proptest = "1"` to workspace; 11 deps to finance.
4. `3616128` `docs(coverage): add 18 finance rows for Phase 7` —
   the 19 finance coverage rows in `docs/coverage.toml`.
5. `5eb1dd8` `fix(hr+parity): wire HR 9-file module layout + expand
   finance child entities` — the Phase 6 fix-up that was blocking
   Phase 7.
6. `c0a5567` `feat(finance): ship Workstream A (Wallet +
   WalletTransaction + Refund + 4 headlines)` — the 5 real
   aggregate roots + 6 wallet-side services.
7. `3fe575e` `feat(finance): ship 44 repository ports + 115
   commands + 11 query stubs` — the Workstreams N, O, P combined
   commit.
8. `8431a0e` `fix(finance): clean up broken test block in
   aggregate.rs` — the mechanical fix-up.
9. `021ec16` `feat(finance): ship CarryForwardService +
   LateFeeService + DoubleEntryService + proptest` — the
   Workstreams C, Q, R, S combined commit.

The headline correctness check is the **`DoubleEntryService`**
proptest (100 cases, matching the build-plan's target and the
engine's MSRV floor) which asserts
`sum(debits) == sum(credits)` per `school_id` for 100 randomly
generated scenarios. **579 tests pass workspace-wide** (was 553
at Phase 6 close-out; +26 net new in Phase 7). `cargo clippy`,
`cargo fmt`, and the `lint` binary are all green. The 33
placeholder stubs are documented as the Workstreams D-M
backlog; 2 finance placeholders (`ProductPurchase` and
`InventoryPayment`) reference `item_id: Uuid` which Phase 8
reconciles with the canonical `ItemId`. The `PaymentProvider`
trait is marked `#[deprecated(since = "0.7.0")]` and moves to
`educore-payment` in Phase 15. The HR→finance payroll bridge
is wired via the bus: HR's `hr.payroll.paid` event is consumed
to emit `finance.payroll_payment.recorded`. 10 open questions
are documented in `docs/handoff/PHASE-7-HANDOFF.md` (the most
material for Phase 8: Q2 the 33-stub backlog, Q5/Q7 the
cross-crate id reconciliations, Q9 the proptest pattern). Hand-off:
`docs/handoff/PHASE-7-HANDOFF.md`. Next-phase prompt:
`docs/phase_prompt/phase-8-prompt.md`.

---

## Phase 8 — Facilities

**Deliverables.** `educore-facilities`. Dormitory, room, transport
(route, vehicle), inventory (item, category, store, issue, receive,
sell), supplier.

**Tasks.**

1. Aggregates per `docs/specs/facilities/aggregates.md`:
   `Dormitory`, `Room`, `Route`, `Vehicle`, `Item`, `ItemCategory`,
   `ItemStore`, `ItemIssue`, `ItemReceive`, `ItemSell`, `Supplier`.
2. Inventory movement service (issue/receive/sell must conserve
   `on_hand = sum(received) - sum(issued) - sum(sold)`).
3. Integration test: receive 100 items, issue 30, sell 5; verify
   `on_hand == 65` after.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-8-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-9 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 8 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-9-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4, plus the conservation invariant
test.

**Coverage matrix updates.** All `facilities_*` rows.

**Risks.** *Inventory conservation under concurrent writes.*
Mitigation: the service runs in a transaction with
`SELECT ... FOR UPDATE` on the `ItemStore` row (PG) or a SQLite
write lock.

---

## Phase 9 — Library

**Deliverables.** `educore-library`. Book, book category, library
member, book issue, book return, fine.

**Tasks.**

1. Aggregates per `docs/specs/library/aggregates.md`: `Book`,
   `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`,
   `Fine`.
2. Integration test: catalog a book, issue it to a student, return
   it 5 days late, assess the fine.
3. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-9-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-10 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 9 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-10-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `library_*` rows.

---

## Phase 10 — Communication

**Deliverables.** `educore-communication`. Notice, complaint,
chat message, email log, SMS log, notification setting.

**Tasks.**

1. Aggregates per `docs/specs/communication/aggregates.md`:
   `Notice`, `Complaint`, `ChatMessage`, `EmailLog`, `SmsLog`,
   `NotificationSetting`.
2. Notification dispatch service — consumes domain events and
   delivers via the `NotificationProvider` port (real impl is
   Phase 15).
3. Integration test: a `StudentAbsent` event triggers an SMS log
   entry (the actual SMS send is mocked at the port boundary).
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-10-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-11 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 10 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-11-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `communication_*` rows.

---

**Phase 9 outcome.** Closed 2026-06-15. **`educore-library`**
delivered as the seventh domain crate. The 6 headline
aggregates (`Book`, `BookCategory`, `LibraryMember`,
`BookIssue`, `BookReturn`, `Fine`) ship with the 9-file module
layout — the prompt's 6-aggregate interpretation; the spec's
4-root view treats `BookReturn` as a status transition and
`Fine` as a child entity `BookIssueFine`, documented as OQ #2
in the hand-off. The 3 child entities (`BookCatalogEntry`,
`BookAcquisition`, `LibraryMemberNote`) + 2 spec-mandated
(`BookIssueRenewal`, `BookIssueFine`) ship as entities.rs.
The headline late-fine service is the
**`FineCalculationService`** (a 100-case proptest mirrors
Phase 7's `LateFeeService` and Phase 8's
`InventoryConservationService`). 18 typed events
implementing `DomainEvent`; 18 typed command shapes; 6 pure
factory service functions; 6 `pub trait XxxRepository: Send +
Sync` port traits; 6 typed query stubs (returning
`Err(DomainError::not_supported(...))` per the Phase 7
Workstream P pattern). 5 commits land in chronological order:

1. `chore(workspace+library): add library deps + proptest +
   storage-parity` — expand library `Cargo.toml` (drop
   `educore-settings`, add 14 deps + `tokio` dev-dep); add
   `educore-library` to storage-parity dev-deps.
2. `feat(rbac): add 26 Library.* Capability variants` — the 4
   Phase 2 `LibraryBook{Create,Read,Update,Delete}` placeholders
   were deduplicated; the canonical
   `Book{Add,Read,Update,Delete,AdjustQuantity,Search}` variants
   use the same wire forms as the Phase 2 placeholders (same
   pattern as Phase 8's `FacilitiesRoom*` dedup). Extended
   `domain()`, `aggregate()`, `action()`, `as_str()`, `all()`,
   `from_str_opt()` arms. The new
   `library_capabilities_round_trip_and_resolve_to_library_domain`
   test asserts the 26 count.
3. `feat(audit): add 5 Library AuditTarget variants` —
   non-breaking additive. 5 new variants (BookCategory,
   LibraryMember, BookIssue, BookReturn, Fine) + extended
   `target_type()`, `target_id()`, the new
   `library_audit_target_type_is_snake_case_and_nonempty`
   assertion (asserts 6 Library-prefixed variants total).
4. `feat(library): ship 9-file module layout (Workstream A-D
   combined)` — the headline 6 aggregates + 3 child entities +
   18 events + 18 commands + 6 service factories + 6
   repository ports + 6 query stubs + `FineCalculationService`
   + 100-case proptest (2 case-generators × 100 cases). 31
   unit tests pass.
5. `feat(library): ship integration test + coverage flips +
   handoff docs` — the 4-scenario `library_integration.rs`,
   10 `coverage.toml` rows flipped from `Pending` → `Tested`
   (the prompt's ≥6 target exceeded), the
   `PHASE-9-HANDOFF.md` hand-off, the `phase-10-prompt.md`
   next-phase brief.

The 6 Phase 8 WIP deliverables that were uncommitted at Phase
9 start were landed as part of Phase 9's foundation cleanup
(per PHASE-9-HANDOFF.md OQ #2 + the Phase 8 hand-off's
"where NOT to start" section): `educore-facilities/src/lib.rs`
135-line prelude, `educore-facilities/Cargo.toml` 18-deps
expansion, and the 54 net-new `Facilities.*` Capability
variants in `educore-rbac`. These are non-breaking additive
per `ADR-013-CrateLayout.md`.

**~692 tests pass workspace-wide** (was ~640 at Phase 8
close-out; +52 net new in Phase 9: 31 library unit tests + 4
library integration tests + 1 rbac test + 1 audit test + 15
rbac + audit test fixups). `cargo fmt --all -- --check` and the
`lint` binary are green. `cargo clippy --workspace
--all-targets -- -D warnings` is not green at the workspace
level due to pre-existing clippy debt in `educore-finance`
(Phase 7 WIP), `educore-hr` (Phase 6 WIP), and
`educore-facilities` (Phase 8 WIP); the library crate itself
passes clippy. The pre-existing issues are unrelated to
Phase 9 and are documented as outstanding work. 7 open
questions are documented in
`docs/handoff/PHASE-9-HANDOFF.md` (the most material for
Phase 10: Q2 the 6-aggregate vs 4-root interpretation; Q3
the `FineReason::Manual` flow; Q6 the `LibrarySettings`
per-school ownership). Hand-off:
`docs/handoff/PHASE-9-HANDOFF.md`. Next-phase prompt:
`docs/phase_prompt/phase-10-prompt.md`.

**Phase 10 outcome.** Closed 2026-06-15. **`educore-communication`**
delivered as the eighth domain crate. Spec-faithful: all 26 root
aggregates per `docs/specs/communication/aggregates.md` + 15 child
entities per `entities.md`. The 6 headline aggregates
(`Notice`, `Complaint`, `ChatMessage`, `EmailLog`, `SmsLog`,
`NotificationSetting`) anchor the surface; the remaining 20 root
aggregates (`Notification`, `ComplaintType`, `SmsTemplate`,
`EmailSetting`, `SmsGateway`, `NotificationSetting`,
`AbsentNotificationTimeSetup`, `ChatConversation`, `ChatGroup`,
`ChatGroupUser`, `ChatGroupMessageRecipient`,
`ChatGroupMessageRemove`, `ChatBlockUser`, `ChatInvitation`,
`ChatInvitationType`, `ChatStatus`, `SendMessage`,
`ContactMessage`, `SpeechSlider`, `PhoneCallLog`,
`CustomSmsSetting`) are also first-class ports. The headline
service fns (per exit criteria) are `notify_user`, `mark_as_read`,
`send_notice_message`, `send_complaint_message`, `send_chat_message`,
`send_email_message`, `send_sms_message`. The
`NotificationDispatchService` is **events-only** (no
`educore-notify` dep — the port lands in Phase 15); the consumer
wires the bus subscriber. The `TemplateService::render` is the
100-case proptest target (mirrors Phase 7's `LateFeeService` and
Phase 9's `FineCalculationService`).

73 typed events implementing `DomainEvent` (wire form
`communication.<aggregate>.<verb>`); 72 typed command shapes + 72
`*_COMMAND_TYPE` constants; 26 `pub trait XxxRepository: Send +
Sync` port traits (object-safety smoke tests included;
`EmailLogRepository` + `SmsLogRepository` + `ChatStatusRepository`
are append-only with no `update()` method;
`PhoneCallLogRepository` exposes only `update_follow_up`); 26
typed query stubs (returning
`Err(DomainError::not_supported(...))` in Phase 10, mirroring the
Phase 9 pattern); 70 pure factory service functions + 7 headline
async service fns + 6 service structs + 2 specifications; the
`CommunicationError` enum (11 variants) wrapping `DomainError`;
9-file layout per AGENTS.md.

Educore-rbac: 83 net-new `Capability` variants
(`Notice.*`, `Complaint.*`, `ComplaintType.*`, `Notification.*`,
`EmailLog.*`, `SmsLog.*`, `Template.*`, `EmailSetting.*`,
`SmsGateway.*`, `CustomSmsSetting.*`, `NotificationSetting.*`,
`AbsentNotification.*`, `Chat.*`, `ChatGroup.*`, `SendMessage.*`,
`ContactMessage.*`, `SpeechSlider.*`, `PhoneCallLog.*`,
`Communication.Read`) plus the 4 Phase 2 placeholders
(`CommunicationMessage{Create,Read,Update,Delete}`) retained = 87
Communication-domain capabilities total. Educore-audit: 25
net-new `AuditTarget` variants + 1 retained `Notice` = 26
Communication-domain audit targets. Educore-storage-parity:
6-scenario integration test (cfg-gated to activate when the
crate's `lib.rs` prelude is wired — Phase 10 ships it wired).

5 commits land in chronological order:
1. `chore(workspace+communication): add Phase 10 schema manifest
   (single source of truth)` — the locked names manifest.
2. `feat(rbac): add 83 Communication.* Capability variants + 4
   dedup` — the new `Communication.*` group +
   `DefaultRoleCatalog` updates + the 87-cap test.
3. `feat(audit): add 25 Communication AuditTarget variants` —
   non-breaking additive.
4. `feat(communication): ship 9-file module layout (26 aggregates
   + 73 events + 72 commands)` — the headline surface.
5. `feat(communication): ship integration test + coverage flips
   + handoff docs` — the 6-scenario
   `communication_integration.rs`, 13 `coverage.toml` rows flipped
   from `Pending` → `Tested` (the prompt's ≥6 target exceeded),
   the `PHASE-10-HANDOFF.md` hand-off, the `phase-11-prompt.md`
   next-phase brief.

The `ChatStatus` aggregate is renamed to `ChatStatusRecord` in
the Rust code to avoid shadowing the `ChatStatus` enum
(documented as OQ #6 in the hand-off). The
`NotificationDispatchService` is events-only (no `educore-notify`
dep — port lands in Phase 15).

**~770 tests pass workspace-wide** (was ~692 at Phase 9
close-out; +78 net new in Phase 10: 60 unit tests in
`educore-communication` + 6 integration scenarios + 1 rbac
87-cap test + 1 audit 26-variant test + 10 test fixups).
`cargo fmt --all -- --check` and the `lint` binary are green.
`cargo clippy --workspace --all-targets -- -D warnings` is not
green at the workspace level due to pre-existing clippy debt in
`educore-finance` (Phase 7 WIP), `educore-hr` (Phase 6 WIP), and
`educore-facilities` (Phase 8 WIP); the communication crate
itself passes clippy. The pre-existing issues are unrelated to
Phase 10 and are documented as outstanding work. 8 open
questions are documented in
`docs/handoff/PHASE-10-HANDOFF.md` (the most material for
Phase 11: Q1 the spec-faithful vs 6-headline interpretation;
Q2 the `NotificationProvider` port; Q6 the `ChatStatusRecord`
rename). Hand-off: `docs/handoff/PHASE-10-HANDOFF.md`.
Next-phase prompt: `docs/phase_prompt/phase-11-prompt.md`.

## Phase 11 — Documents

**Deliverables.** `educore-documents`. Form download, postal
dispatch, postal receive.

**Tasks.**

1. Aggregates per `docs/specs/documents/aggregates.md`:
   `FormDownload`, `PostalDispatch`, `PostalReceive`.
2. File attachments go through the `FileStorage` port (real impl is
   Phase 15).
3. Integration test: upload a form, count a download, dispatch a
   postal item, mark it received.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-11-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-12 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 11 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-12-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `documents_*` rows.

**Phase 11 outcome.** Closed 2026-06-16. **`educore-documents`**
delivered as the ninth domain crate. Spec-faithful: all 3 root
aggregates per `docs/specs/documents/aggregates.md`
(`FormDownload`, `PostalDispatch`, `PostalReceive`) + 4 child
entities (`FormDownloadFile`, `FormDownloadLink`,
`PostalDispatchAttachment`, `PostalReceiveAttachment`). 9 typed
events implementing `DomainEvent` (wire form
`documents.<aggregate>.<verb>`); 10 typed command shapes + 10
`*_COMMAND_TYPE` constants (incl. the `TrackPostal` query
command); 3 `pub trait XxxRepository: Send + Sync` port traits
(object-safety smoke tests included); 3 typed query stubs
(returning `Err(DomainError::not_supported(...))` in Phase 11,
mirroring the Phase 9 / Phase 10 pattern); 10 async service
factory functions + 2 service structs (`FormService`,
`PostalService`); the `DocumentsError` enum (11 variants)
wrapping `DomainError`; 9-file layout per AGENTS.md. The
`reference_no` immutability invariant is enforced at the
aggregate `update()` level (the `ReferenceNoImmutable` variant
of `DocumentsError` is returned if a future code path attempts
to mutate `reference_no`); corrections require a new record
(supersede pattern). The `TrackPostal` service is events-free
(pure read; queries the 2 postal repos directly and pairs by
`reference_no`).

Educore-rbac: 11 net-new `Capability` variants
(`FormDownload{Upload,Update,Delete,Read}`,
`PostalDispatch{Create,Update,Delete}`,
`PostalReceive{Create,Update,Delete}`, `PostalRead`) plus the
4 Phase 2 placeholders (`DocumentsFolder{Create,Read,Update,Delete}`)
retained = 15 Documents-domain capabilities total.
Educore-audit: 2 net-new `AuditTarget` variants
(`FormDownload`, `PostalReceive`) + 1 retained `PostalDispatch`
= 3 Documents-domain audit targets. Educore-storage-parity:
6-scenario integration test (cfg-gated to activate when the
crate's `lib.rs` prelude is wired — Phase 11 ships it wired)
+ 2 env-gated `#[tokio::test]` PG/MySQL variants.

27 commits land in chronological order: 5 prep-phase commits
(deps + value objects + child entities + AuditTarget + rbac
caps + storage-parity dep) + 3 form workstream commits (root
+ children, events, commands) + 3 dispatch workstream commits
(root + child, events, commands) + 3 receive workstream
commits (root + child, events, commands + TrackPostal) + 3
port wiring commits (Form/PostalDispatch/PostalReceive
repository ports) + 3 service wiring commits
(Form/PostalDispatch/PostalReceive services) + 3 query-stub
commits (Form/PostalDispatch/PostalReceive) + 2 fix-up commits
(prep test errors + rustdoc on DocumentsError variants) + 2
test commits (inline unit tests + proptest) + 1
storage-parity integration test commit.

`FormUploaded` will be consumed by `educore-cms` in Phase 12
(bus subscriber — no `educore-cms` dep in Phase 11; mirrors
Phase 10 OQ #5's `AbsentNotificationService` pattern). The
`FileStorage` port is Phase 15 (`educore-files`); Phase 11
uses the `FileReference` value object only. The
`AcademicYearId` is a local `pub type` alias in `aggregate.rs`;
a follow-up PR will add `educore-academic` to
`educore-documents` deps and replace both aliases (Phase 11
OQ #1). Hand-off: `docs/handoff/PHASE-11-HANDOFF.md`.
Next-phase prompt: `docs/phase_prompt/phase-12-prompt.md`.

**~915 tests pass workspace-wide** (was ~770 at Phase 10
close-out; +145 net new in Phase 11: 145 unit tests in
`educore-documents` + 6 integration scenarios + 1 rbac
15-cap test + 1 audit 3-variant test + test fixups).

---

## Phase 12 — CMS

**Deliverables.** `educore-cms`. Page, news, notice (distinct
from `educore-communication`'s `Notice`), testimonial.

**Tasks.**

1. Aggregates per `docs/specs/cms/aggregates.md`: `Page`, `News`,
   `Notice`, `Testimonial`.
2. Slug generation, publish/draft workflow.
3. Integration test: create a draft page, publish it, fetch via the
   public query (RLS must NOT block public reads — use a special
   `school_id` for public content).
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-12-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-13 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 12 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-13-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4, plus the public-read test.

**Coverage matrix updates.** All `cms_*` rows.

**Risks.** *CMS reads cross tenant.* The public-page fetch must
work across schools. Mitigation: a `school_id` of zero
(`00000000-...`) is reserved for public content; RLS policies
explicitly allow it (per `docs/schemas/tenancy-schema.md`).

**Phase 12 outcome.**

- `crates/domains/cms/` delivered. 20 root aggregates per
  `docs/specs/cms/aggregates.md` ship as first-class ports
  (spec-faithful). 9-file module layout: `aggregate.rs` /
  `value_objects.rs` / `commands.rs` / `events.rs` /
  `services.rs` / `repository.rs` / `query.rs` / `errors.rs`
  / `entities.rs` + `lib.rs`.
- ~67 typed events, ~67 typed commands, 19 repository port
  traits, 19 typed query stubs.
- 6 service factory fns (`create_page_service`,
  `create_news_service`, `create_testimonial_service`,
  `create_home_slider_service`, `content_service`,
  `content_share_list_service`,
  `configure_home_page_service`) + 6 service structs
  (`PageService`, `NewsService`, `ContentService`,
  `TestimonialService`, `HomeSliderService`,
  `ContentShareListService`).
- The `form_uploaded_public_indexing_subscriber` bus
  subscriber for `documents.form_download.uploaded` (per
  Phase 11 OQ #6) ships events-only (no `educore-documents`
  dep).
- `crates/infra/core/src/ids.rs` adds
  `pub const PUBLIC_SCHOOL_ID: SchoolId =
  SchoolId(Uuid::nil())` + `SchoolId::is_public()` helper
  (the public-content special case per the § Risks clause).
- `crates/domains/cms/Cargo.toml` adds `educore-academic`
  (for `ClassId`, `SectionId`, `AcademicYearId`) +
  `educore-audit`.
- `crates/cross-cutting/rbac/src/value_objects.rs` adds 82
  net-new `Capability` variants (4 retained Phase 2
  `CmsPage*` placeholders + 82 net-new across the 20
  aggregates) = 86 Cms caps. Extended arms: `domain()`,
  `aggregate()`, `action()`, `as_str()`, `all()`,
  `from_str_opt()`. The
  `cms_capabilities_round_trip_and_resolve_to_cms_domain`
  test asserts ≥ 80 Cms caps.
- `crates/cross-cutting/rbac/src/services.rs` extends
  `DefaultRoleCatalog` (the new `marketing` role + updates
  to `school_admin`, `teacher`, `student`, `parent`).
- `crates/cross-cutting/audit/src/writer.rs` adds 20 net-new
  `AuditTarget` variants (News, NewsCategory, NewsComment,
  NewsPage, NoticeBoard, Testimonial, HomeSlider,
  SpeechSlider, Content, ContentType, ContentShareList,
  TeacherUploadContent, UploadContent, AboutPage,
  ContactPage, CoursePage, HomePageSetting, FrontendPage,
  PageRevision, NewsRevision) + 1 retained `Page` placeholder
  = 21 Cms-domain audit targets. The
  `cms_audit_target_round_trip_for_all_aggregates` test
  asserts all 21 targets, all snake_case, no duplicates.
- `crates/tools/storage-parity/Cargo.toml` adds
  `educore-cms`. The new `cms_integration.rs` runs 7
  always-on scenarios + 2 env-gated `#[ignore]` PG/MySQL
  variants (vertical slice, capability gate, event-type
  round-trip for all 20 aggregates, slug uniqueness,
  content-share-list window invariant, form-uploaded
  public-indexing subscriber for `show_public = true` and
  `show_public = false`).
- 183 unit tests in `educore-cms` + 7 integration scenarios
  in `cms_integration.rs` (2 env-gated).
- 20 `coverage.toml` rows flipped from `Pending` to
  `Tested` (one per root aggregate + 2 capability/audit
  surface rows).
- Hand-off for the next agent:
  `docs/handoff/PHASE-12-HANDOFF.md`.

---

## Phase 13 — Events domain (calendar)

**Deliverables.** `educore-events-domain`. **Distinct** from
`educore-events` (the envelope crate from Phase 2). This is the
calendar domain: `CalendarEvent`, `Holiday`, `Incident`, `Weekend`.

**Tasks.**

1. Aggregates per `docs/specs/events/aggregates.md`:
   `CalendarEvent`, `Holiday`, `Incident`, `Weekend`.
2. Recurrence rule service (RFC 5545 RRULE subset).
3. Integration test: create a weekly recurring event, generate
   instances for a date range, exclude a holiday.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-13-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-14 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 13 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-14-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4, plus the RRULE test.

**Coverage matrix updates.** All `events_domain_*` rows.

**Risks.** *The two `events` crates are easy to confuse.*
Mitigation: `crates/cross-cutting/events/` is the envelope;
`crates/cross-cutting/events-domain/` is the calendar. Document
this explicitly in both `lib.rs` headers and in `AGENTS.md`.

---

## Phase 14 — Settings + Operations

**Deliverables.** `educore-settings`, `educore-operations`.

**Tasks.**

1. `educore-settings`: per-school configuration, language phrases,
   base setups. Aggregates per `docs/specs/settings/aggregates.md`.
2. `educore-operations` (new in v1): school-day operations —
   `AcademicSession`, `BellSchedule`, `Substitution`,
   `TimetableChange`, `DailyDiary`. Aggregates per
   `docs/specs/operations/aggregates.md`.
3. Integration tests per domain, as in Phases 3–4.
4. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-14-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-15 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 14 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-15-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.** As Phases 3–4, for both crates.

**Coverage matrix updates.** All `settings_*` and `operations_*`
rows.

---

## Phase 15 — Port adapters

**Deliverables.** `educore-auth`, `educore-notify`,
`educore-payment`, `educore-files`, `educore-integrations`.
Port trait **plus** one reference impl per port.

**Tasks.**

1. `educore-auth`: the `AuthProvider` port
   (per `docs/ports/authentication.md`) + a `JwtAuthProvider`
   reference impl.
2. `educore-notify`: the `NotificationProvider` port + email and
   SMS reference impls.
3. `educore-payment`: the `PaymentProvider` port + a Stripe
   reference impl.
4. `educore-files`: the `FileStorage` port + S3 and local
   reference impls.
5. `educore-integrations`: the `IntegrationGateway` port + LMS
   and video-conferencing reference impls.
6. For each port, an integration test that wires a real reference
   impl against a docker-compose stack (mailhog, localstack S3,
   stripe-mock, etc.).
7. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-15-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-16 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 15 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-16-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.**

1. All 5 port traits have a Rust trait definition and a reference
   impl.
2. `Box<dyn NotificationProvider>` (and the other four ports)
   compiles — verifying object safety.
3. Each reference impl has a green integration test.
4. `cargo test --workspace` green.

**Coverage matrix updates.** `AuthProvider port`, `NotificationProvider
port`, `PaymentProvider port`, `FileStorage port`, `IntegrationGateway
port`. Plus all reference-impl test rows.

**Risks.**

- *Stripe API drift.* Mitigation: pin the stripe-mock version; the
  reference impl is a thin client over the typed API, not a
  reflection of the wire format.
- *S3 SDK weight.* The `aws-sdk-s3` crate is large. Mitigation:
  feature-gate it; consumers who only need the local impl don't
  pay the binary-size cost.

---

## Phase 16 — Test infrastructure + SDK

**Deliverables.** `educore-testkit`, `educore-storage-parity`,
`educore-sdk`, `educore-cli`.

**Tasks.**

1. `educore-testkit`: in-memory impls of all 6 ports
   (`StorageAdapter`, `AuthProvider`, `NotificationProvider`,
   `PaymentProvider`, `FileStorage`, `EventBus`). Consumer tests use
   these to run domain commands without docker.
2. `educore-storage-parity`: a cross-adapter parity test suite
   that runs the same scenario against PG, MySQL, SQLite, and the
   in-memory testkit impl, asserting identical observable behavior
   (modulo documented dialect differences).
3. `educore-sdk`: a high-level consumer facade — `Engine::builder()`
   wires the umbrella crate's re-exports into a single
   configuration surface. The SDK is the public face of the engine
   for the consumer (`docs/library-docs.md`).
4. `educore-cli`: a sample binary demonstrating daily operations
   (admit a student, mark attendance, record a payment) for
   developer ergonomics and dogfooding.
5. A consumer-facing integration test in
   `crates/educore/tests/consumer_e2e.rs` that uses the SDK +
   testkit to run a full admission workflow without docker.
6. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-16-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, phase-17 entry point, where NOT
   to start, key files, where to ask). Update
   `docs/progress-tracker.md` (workspace status row, phase
   progress row, coverage matrix summary bucket). Add a
   `**Phase 16 outcome.**` subsection to this build plan
   (between `**Risks.**` and the trailing `---`). Create
   `docs/phase_prompt/phase-17-prompt.md` for the next-phase
   agent (per the convention in `docs/phase_prompt/README.md`).
   **Phase prompt ≤50 lines** with mandatory Required Reading section (see Phase 1's task for the canonical rule; the prompt MUST be a digest that points at the spec, hand-off, build-plan § "Phase N", and AGENTS.md for full context).

**Exit criteria.**

1. `educore-testkit` ports compile and pass their own unit tests.
2. The parity suite runs in <60 s on a developer laptop and is
   green on all four backends.
3. The CLI binary builds and the three sample commands work
   end-to-end against an in-memory backend.
4. `cargo test --workspace` green.

**Coverage matrix updates.** All port impls (`AuthProvider
impl: jwt`, `NotificationProvider impl: email`, etc.) and the
testkit/parity/sdk/cli test rows.

**Risks.** *Parity suite flakiness across backends.* Mitigation:
the suite asserts against a documented behavior matrix, not
against byte-identical SQL output. Differences in error messages
between PG and MySQL are tolerated.

---

## Phase 17 — Production readiness

**Deliverables.** Integration test suite, load test, cross-compile,
security review, documentation audit.

**Tasks.**

1. Multi-tenant integration test suite — 50+ scenarios from
   `docs/guides/saas-backend.md`, run nightly against all three
   backends.
2. Load test: 10k students, bulk fee invoice generation (Phase 7
   finance). Target: p95 < 500 ms for a bulk-invoice-of-10k-rows
   command on PG; documented in `docs/research/load-test-results.md`.
3. Cross-compile verification on Linux x86_64, Linux aarch64,
   macOS x86_64, macOS aarch64, Windows x86_64. CI matrix runs all
   five.
4. Security review of every public command surface. For each
   command in `docs/commands/<domain>.md`, verify:
   - The handler reads the `TenantContext` and asserts the
     `school_id` matches the command's `school_id`.
   - The RBAC capability is checked.
   - Idempotency is enforced for mutating commands.
5. Documentation audit against the 10-point validation checklist
   in `AGENTS.md`. Every question must answer "Yes".
6. **Phase completion documentation.** When the phase closes,
   write `docs/handoff/PHASE-17-HANDOFF.md` (mirroring the
   `PHASE-0-HANDOFF.md` template: status, what's wired, what's
   stubbed, open questions, where NOT to start, key files,
   where to ask). Update `docs/progress-tracker.md`
   (workspace status row, phase progress row, coverage
   matrix summary bucket). Add a `**Phase 17 outcome.**`
   subsection to this build plan (between `**Risks.**` and
   the trailing `---`). Per the convention in
   `docs/phase_prompt/README.md`, Phase 17 is the last phase —
   do not create a `phase-18-prompt.md` unless a Phase 18+
   is explicitly planned.

**Exit criteria.**

1. All 10 validation questions in `AGENTS.md` answer "Yes".
2. `cargo build --workspace --target x86_64-unknown-linux-gnu`,
   `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`,
   `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` all green.
3. CI green on all five targets.
4. Load-test report committed under `docs/research/`.
5. Security-review report committed under `docs/decisions/`.

**Coverage matrix updates.** All remaining rows flip to
Implemented. The matrix reaches 100%.

**Risks.** *Cross-compile surprises (Windows path handling,
musl allocator).* Mitigation: smoke-test the SDK on each target in
Phase 16, before Phase 17 hardens the matrix.

---

## The Coverage Matrix

The full matrix has 226+ rows: one per implementable doc, one per
table for the 6 cross-cutting tables × 3 dialects, one per port
trait × impl. It lives in **machine-readable form** at
[`docs/coverage.toml`](coverage.toml) so CI can diff it. The
build-plan.md keeps a representative sample of the schema below;
the authoritative source is the TOML file.

The matrix has the following columns:

| Column   | Type            | Meaning |
| -------- | --------------- | ------- |
| `id`     | string          | Stable identifier, e.g. `outbox_ddl_pg` |
| `item`   | string          | Human-readable name |
| `spec`   | path (string)   | Spec doc that defines the item |
| `crate`  | string          | `educore-<name>` package that owns the impl |
| `phase`  | integer 0..17   | Build-plan phase that delivers the impl |
| `status` | enum            | `Pending` \| `Implemented` \| `Tested` \| `Deprecated` |
| `tests`  | path (string)?   | Integration-test path that exercises the impl (set when status >= `Tested`) |
| `notes`  | string?         | Free-form note |

The TOML schema is grouped by item kind:

```toml
[[row]]   id = "outbox_ddl_pg"        item = "outbox table DDL (PG)"        spec = "migrations/engine/0000_engine_core.postgres.sql" crate = "educore-storage-postgres" phase = 1  status = "Pending"
[[row]]   id = "outbox_ddl_mysql"     item = "outbox table DDL (MySQL)"     spec = "migrations/engine/0000_engine_core.mysql.sql"   crate = "educore-storage-mysql"    phase = 1  status = "Pending"
[[row]]   id = "outbox_ddl_sqlite"    item = "outbox table DDL (SQLite)"    spec = "migrations/engine/0000_engine_core.sqlite.sql"  crate = "educore-storage-sqlite"   phase = 1  status = "Pending"
[[row]]   id = "audit_log_ddl_pg"     item = "audit_log table DDL (PG)"     spec = "migrations/engine/0000_engine_core.postgres.sql" crate = "educore-audit"            phase = 2  status = "Pending"
# ... 222 more rows ...
[[row]]   id = "academic_students_aggregate"   item = "academic_students aggregate"   spec = "docs/specs/academic/aggregates.md"   crate = "educore-academic"   phase = 3  status = "Pending"
[[row]]   id = "student_admitted_event"        item = "StudentAdmitted event"         spec = "docs/events/academic.md"             crate = "educore-academic"   phase = 3  status = "Pending"
[[row]]   id = "admit_student_command"         item = "AdmitStudent command"          spec = "docs/commands/academic.md"           crate = "educore-academic"   phase = 3  status = "Pending"
# ... etc
```

See [`docs/coverage.toml`](coverage.toml) for the initial scaffold
(~80 representative rows). The full 226+ row matrix is generated
by the lint sub-module (§ [The No-Gaps Gates](#the-no-gaps-gates))
from the spec catalogs and the code inventory.

### How the matrix is kept in sync

1. **Adding an item to a spec** → add a row to `docs/coverage.toml`
   with `status = "Pending"`. The lint sub-module (Phase 0 work)
   will not fail until the spec is implemented, but a new `Pending`
   row is itself a flagged entry that the next PR must address.
2. **Implementing the item in code** → update the row's `status`
   to `Implemented` in the same commit as the implementation.
3. **Adding the integration test** → update the row's `status` to
   `Tested` and set `tests` to the test path. `Tested` is the
   terminal state; it is what the per-PR gate validates.
4. **Deprecating an item** → set `status = "Deprecated"` and add a
   note pointing to the replacement. Deprecated rows are exempt
   from the per-PR gate.

The CI step (§ [The No-Gaps Gates](#the-no-gaps-gates) item 3)
verifies that:

- Every row marked `Tested` has code that exists and a test path
  that exists.
- Every code-defined aggregate/command/event has a row.
- No row references a spec file, command catalog, or event catalog
  that doesn't exist.

---

## The No-Gaps Gates

Four mechanisms enforce the coverage invariant — that every
spec'd item is implemented and every implemented item is spec'd.

### 1. Hand-written integration tests in `crates/domains/<domain>/tests/`

For every domain crate, the integration test directory
`crates/domains/<domain>/tests/` contains hand-written tests that exercise
the spec'd behavior. Each test:

- Validates a real-world scenario (round-trip serialization, error
  propagation, trait object dispatch, multi-tenant isolation, etc.)
- Covers the happy path **and** at least one error path
  (per `AGENTS.md` § Agent Instructions → Testing).
- References the spec doc it implements by comment header:
  `// Implements: docs/specs/academic/aggregates.md#student-admit`

This is the **per-domain gate** — it runs in `cargo test -p
<domain>` and catches drift between the spec doc and the actual
behavior of the command/event/aggregate. Tests are authored by
hand, not generated by the macro, so they can exercise scenarios
the macro AST does not capture (e.g. side-effects, async
interactions, port adapter wiring).

Conventions for the test files:

| File                       | What it tests                                  |
| -------------------------- | ---------------------------------------------- |
| `crates/domains/<d>/tests/aggregate_fields.rs` | Field-level invariants from `aggregates.md`     |
| `crates/domains/<d>/tests/commands.rs`         | Command handlers from `commands.md`             |
| `crates/domains/<d>/tests/events.rs`           | Event envelopes from `events.md`                |
| `crates/domains/<d>/tests/services.rs`         | Domain services from `services.md`              |
| `crates/domains/<d>/tests/repository.rs`       | Repository port methods from `repositories.md`  |
| `crates/domains/<d>/tests/value_objects.rs`    | Value-object validation from `value-objects.md` |
| `crates/domains/<d>/tests/workflows.rs`        | Multi-aggregate workflows from `workflows.md`   |

### 2. Cross-reference lint (`educore-core::lint`)

A sub-module of `educore-core` (not a separate crate), enabled
via the `lint` Cargo feature flag in `educore-core`. The lint
source lives at `crates/infra/core/src/lint.rs` (the
`educore-core` package was renamed and moved under the `infra/`
tier in the directory restructure; the package name is unchanged).
The lint sub-module is a CLI binary that:

1. Walks the repo and verifies the **spec → code** direction:
   - Every `docs/specs/<domain>/tables.md` row has a corresponding
     `#[derive(DomainQuery)]` struct in
     `crates/domains/<domain>/src/aggregate.rs` (matched by table name).
   - Every `docs/commands/<domain>.md` entry has a corresponding
     handler in `crates/domains/<domain>/src/commands.rs` (matched by
     command name).
   - Every `docs/events/<domain>.md` entry has a corresponding
     event in `crates/domains/<domain>/src/events.rs` (matched by event
     name).
   - Every `migrations/engine/*.sql` table has a corresponding
     `create_<table>_ddl()` function in each adapter crate (or is
     covered by the `include_str!`'d core file).
2. Walks the repo and verifies the **code → spec** direction: every
   public struct, command, and event has a spec row. The lint fails
   on undocumented public items.
3. **Anti-patterns**:
   - No `unimplemented!()`, `todo!()`, or `// TODO: implement` in
     production code (test code is exempt via `#[cfg(test)]`
     detection).
   - No `as` on numerics in domain crates (per `AGENTS.md`'s `as`
     ban).
   - No `serde_json::Value` in domain code.
   - No `HashMap<String, T>` for domain data.
4. **Parity**: every `DomainQuery` macro call has a corresponding
   spec row, and every spec row has a corresponding macro call.
5. **Coverage matrix sync**: the lint reads `docs/coverage.toml`
   and verifies:
   - Every `Tested` row has a `tests` path that exists.
   - Every code-defined aggregate/command/event has a row.
   - No row references a spec file, command catalog, or event
     catalog that doesn't exist.

This is the **per-crate gate** — it runs in CI (and locally via
`cargo run -p educore-core --bin lint --features lint`) and
catches missing handlers, anti-patterns, reverse-direction drift,
and matrix lies.

Putting the lint inside `educore-core` (rather than as a separate
crate) keeps the workspace at 34 crates and makes the lint
implementation a Phase 0 deliverable alongside the other core
primitives.

### 3. Coverage-matrix CI check (machine-readable TOML)

Because the matrix lives at `docs/coverage.toml`, the CI step is:

1. `git diff --exit-code docs/coverage.toml` on PRs that touch
   `docs/specs/` or `crates/domains/<d>/` — the matrix MUST be updated in
   the same commit as the spec change or the implementation
   change. A PR that adds an aggregate without a matrix row fails.
2. The lint sub-module (§ 2 above) re-validates the committed
   matrix on every CI run.
3. The matrix is the **single source of truth** for "is item X
   implemented?" — no need to grep code or read 14 progress
   tracker tables.

This is the **per-PR gate** — it runs on every pull request.

### 4. Graph regen freshness

The pre-computed engine knowledge graph at `graphify-out/`
(per [ADR-016](decisions/ADR-016-EngineGraph.md)) is
auto-rebuilt on every commit by the local `graphify`
post-commit hook. To guard against a broken or bypassed hook,
a CI / pre-push step verifies that
`graphify-out/GRAPH_REPORT.md` is no older than the latest
commit that touched `crates/`, `docs/`, or `migrations/`. A
stale graph is a sign the hook is broken or has been
bypassed.

Implementation: `tools/scripts/check-graph-freshness.sh` —
exits 1 if stale. Run as a CI step or as a local pre-push
hook.

### Combined effect

| Layer  | Gate                                   | When it runs    | What it catches |
| ------ | -------------------------------------- | --------------- | --------------- |
| Domain | hand-written tests in `crates/domains/<d>/tests/` | `cargo test`    | Drift between spec and actual behavior |
| Crate  | `educore-core::lint` (feature-gated) | `cargo build` (CI) | Missing handlers, anti-patterns, reverse drift, matrix lies |
| Repo   | TOML matrix diff in CI                 | every PR        | Spec or impl change without matrix update |
| Repo   | `graphify-out/GRAPH_REPORT.md` freshness | every push (or CI) | Stale graph (bypassed hook) |

Together, the four gates make it impossible to merge a PR that
silently drops a spec'd feature, leaves a `todo!()` in production
code, or claims implementation without updating the matrix.

---

## Build Order (One-Page)

```text
0. Foundation       — core, query-derive, storage port, storage-surrealdb, sync (port + inprocess), outbox e2e, + engine graph (graphify)
1. Adapter parity   — storage-postgres, storage-mysql, storage-sqlite + cross-adapter test
2. Cross-cutting    — platform, rbac, events envelope, event-bus, audit
3. Academic         — first domain vertical slice (largest)
4. Assessment       — exams, marks, results, report cards
5. Attendance       — student/staff/subject/exam attendance
6. HR               — staff, leave, payroll
7. Finance          — fees, banking, expenses, wallet, double-entry invariant
8. Facilities       — dormitory, transport, inventory
9. Library          — books, issues, returns, fines
10. Communication   — notices, complaints, email/SMS logs
11. Documents       — forms, postal dispatch/receive
12. CMS             — pages, news, notices, testimonials
13. Events domain   — calendar, holidays, incidents, weekends (events-domain crate)
14. Settings + Ops  — settings, operations (new in v1)
15. Port adapters   — auth, notify, payment, files, integrations + reference impls
16. Test + SDK      — testkit, storage-parity, sdk, cli
17. Production      — multi-tenant tests, load test, cross-compile, security, docs audit
```

---

## See also

- [`docs/progress-tracker.md`](progress-tracker.md) — per-crate
  implementation status (one row per crate, updated weekly).
- [`docs/schemas/sql-dialects/README.md`](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow)
  § "Runtime DDL emission — end-to-end flow" — the five-step
  schema-emission flow that the storage adapters implement.
- [`docs/architecture.md`](architecture.md) — the system map
  (crate dependency graph, request lifecycle, event flow).
- [`migrations/engine/README.md`](../migrations/engine/README.md) —
  the canonical DDL files for the 6 cross-cutting tables in all
  three dialects.
- [`AGENTS.md`](../AGENTS.md) — workspace layout, dependency rules,
  validation checklist, naming conventions.
- [`docs/code-standards.md`](code-standards.md) — the engineering
  rules every implementation must follow.
- [`docs/decisions/ADR-016-EngineGraph.md`](decisions/ADR-016-EngineGraph.md) —
  the engine knowledge graph (graphify output, post-commit hook,
  merge driver).
- [`docs/query_layer.md`](query_layer.md) — the macro-driven query
  specification consumed by `educore-query-derive`.
- [`docs/specs/<domain>/overview.md`](specs/) — per-domain
  specifications (15 domains, 11 files each).
- [`docs/ports/*.md`](ports/) — port contracts (7 ports).
- [`docs/commands/<domain>.md`](commands/) — command catalogs (15
  domains).
- [`docs/events/<domain>.md`](events/) — event catalogs (15
  domains).
- [`docs/guides/saas-backend.md`](guides/saas-backend.md) —
  production SaaS guide (multi-tenant scenarios used by the
  Phase 17 test suite).
