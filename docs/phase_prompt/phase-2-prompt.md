# Educore Phase 2 — Cross-cutting foundations

## Mission

You are continuing the Educore engine build-out. Phase 0 (PR 0
+ PR A, see [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md))
and **Phase 1 (storage adapter parity, see
[`docs/handoff/PHASE-1-HANDOFF.md`](../handoff/PHASE-1-HANDOFF.md)) are
closed.** Your job is **Phase 2**: deliver the cross-cutting
foundations that every domain crate will depend on.

This is **implementation**, not design. The specs already exist
in `docs/specs/platform/`, `docs/specs/rbac/`, `docs/specs/audit/`,
`docs/specs/events/`. The DDL for the 6 engine cross-cutting
tables (outbox, audit_log, event_log, idempotency, schema_registry,
system_user) is pre-written in
`migrations/engine/0000_engine_core.{postgres,mysql,sqlite}.sql`.
The SQL storage adapters from Phase 1 are your persistence
layer.

You are NOT:
- Designing new ports or new aggregates
- Modifying the Phase 1 storage adapters' flag-based
  transaction model
- Renaming crates or moving files
- Adding new external crates without updating ADR-015
- Re-introducing `mysql_async` or `flate2` (the user
  rejected them in Phase 1)

You ARE:
- Implementing the 5 cross-cutting crates per the Phase 2
  spec in [`docs/build-plan.md`](../build-plan.md) § "Phase 2"
- Wiring `educore-events::EventEnvelope<T: DomainEvent>` to
  the Phase 1 storage adapters' `AuditLog::append` and
  `EventLog::append` (the storage-port structs are
  intermediate types until `educore-events` lands; see the
  prior hand-off's open question #5)
- Exercising all 6 cross-cutting tables in a single
  end-to-end integration test
- Flipping `docs/coverage.toml` rows `Pending` → `Tested`
  in the same commits as the impls
- Writing the Phase 2 hand-off (`PHASE-2-HANDOFF.md`) and
  the Phase 3 prompt (`docs/phase_prompt/phase-3-prompt.md`)
  at phase close (per the convention in
    [`README.md`](README.md))

## Deliverables

1. **`educore-platform`** — `School`, `User`, `SchoolId`,
   `UserId`, `TenantContext`. Spec is in
   `docs/specs/platform/`.
2. **`educore-rbac`** — `Capability`, `Role`, `Permission`,
   the capability check port, the default role catalog,
   `is_replicated` flag for distributed deployments. Spec
   is in `docs/specs/rbac/`.
3. **`educore-events`** — the **envelope** crate.
   `DomainEvent` trait, `EventEnvelope` (event_id,
   correlation_id, causation_id, occurred_at, payload),
   `EventBus` trait. **Not** the calendar domain (that's
   `educore-events-domain` in Phase 13). Per
   `AGENTS.md` "Note on educore-events vs
   educore-events-domain" — the two are easy to confuse.
4. **`educore-event-bus`** — in-process, NATS, Redis impls
   behind the `EventBus` port (per
   `docs/ports/event-bus.md`).
5. **`educore-audit`** — the audit log writer
   (`AuditLogEntry { actor, action, target, before, after,
   occurred_at, correlation_id }`), retention policies
   (configurable `retention_days`; engine emits a
   `retention_sweep_due` event when the policy threshold
   is reached), and the audit write path (called from
   every command handler in the engine).

## Required Reading (priority order)

1. [`docs/handoff/PHASE-1-HANDOFF.md`](../handoff/PHASE-1-HANDOFF.md)
   — the prior hand-off. The SQL storage adapters it
   shipped are the persistence layer for your integration
   test. Read its "Open questions" section first; question
   #5 (the `educore-events` envelope crate is the missing
   link) is the primary entry point for your work.
2. [`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md)
   — the prior hand-off. Its "Open questions" still
   apply; the ad-hoc sync envelope refactor (Phase 0 open
   question #2) is one of Phase 2's deliverables — the
   `educore-sync` crate should depend on
   `educore_events::EventEnvelope`, not its own struct.
3. [`docs/build-plan.md`](../build-plan.md) § "Phase 2" —
   the canonical Phase 2 spec (the 6 tasks + 4 exit
   criteria + coverage matrix updates + risks).
4. [`docs/ports/event-bus.md`](../ports/event-bus.md) —
   the `EventBus` port contract that `educore-event-bus`
   implements.
5. [`docs/ports/storage.md`](../ports/storage.md) — the
   storage port contract; the Phase 1 SQL adapters
   implement the 4 sub-ports you'll exercise.
6. [`docs/specs/platform/overview.md`](../specs/platform/overview.md),
   [`docs/specs/rbac/overview.md`](../specs/rbac/overview.md),
   [`docs/specs/audit/overview.md`](../specs/audit/overview.md),
   [`docs/specs/events/overview.md`](../specs/events/overview.md)
   — the design contracts.
7. [`docs/schemas/audit-schema.md`](../schemas/audit-schema.md) § 13 —
   the `audit_log` table spec; the partitioning strategy
   for the 10M-rows/day scale is documented here.
8. [`docs/schemas/tenancy-schema.md`](../schemas/tenancy-schema.md) —
   the `school_id` RLS / `TenantContext` contract.
9. [`docs/schemas/sql-dialects/postgresql.md`](../schemas/sql-dialects/postgresql.md)
   § "Row-level security" — the `CREATE POLICY` +
   `ENABLE ROW LEVEL SECURITY` pattern for the cross-
   tenant isolation test.
10. [`AGENTS.md`](../AGENTS.md) — workspace rules, naming,
    lint policy, the 9-file module layout per domain.
11. [`docs_guidlines/system.md`](../docs_guidlines/system.md) +
    [`docs_guidlines/execution_guidlines.md`](../docs_guidlines/execution_guidlines.md)
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

> **Phase 2 workstreams.** The five cross-cutting
> crates (`educore-platform`, `educore-rbac`,
> `educore-events`, `educore-event-bus`,
> `educore-audit`) are independent in the sense that
> they don't share files — but `educore-events` is a
> prerequisite for `educore-audit` (the audit log
> writer takes `EventEnvelope<T: DomainEvent>`).
> Recommended subagent dispatch order:
> 1. `educore-events` first (the envelope crate; 1
>    subagent).
> 2. `educore-platform` + `educore-rbac` in parallel
>    (2 subagents).
> 3. `educore-event-bus` (depends on
>    `educore_events::EventBus`; 1 subagent).
> 4. `educore-audit` (depends on `educore_events`; 1
>    subagent).
> 5. Closing agent: integration test (exercises all
>    5 crates end-to-end), coverage rows, hand-off,
>    `phase-3-prompt.md`.

## Starting Point

- The 4 SQL storage adapters from Phase 1
  (`educore-storage-{postgres,mysql,sqlite}`) are your
  persistence layer. Their `AuditLog` and `EventLog`
  sub-ports take the storage-port structs
  (`AuditLogEntry`, `EventLogEntry`); once `educore-events`
  ships, they should accept `EventEnvelope<T: DomainEvent>`
  (or the engine's audit/event log entry wrapping the
  envelope).
- The `educore-sync` crate from Phase 0 uses an ad-hoc
  `SyncEvent` struct. Phase 2 should refactor sync to
  depend on `educore_events::EventEnvelope`. This is a
  Phase 0 carry-over flagged in
  `PHASE-0-HANDOFF.md` open question #2.
- The Phase 2 integration test
  ([`docs/build-plan.md`](../build-plan.md) § "Phase 2"
  task 6) creates a school, creates a user, creates a
  role, emits a `SchoolCreated` event, then asserts that
  the 6 cross-cutting tables are populated. The test
  should target all 4 SQL adapters (or skip PG/MySQL
  with the `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` env
  vars, like Phase 1 did).
- The 6 cross-cutting DDL files are pre-written. You
  don't write new DDL; you write Rust code that writes
  to the existing tables.

## Per-Deliverable Gotchas

- **`educore-events` vs `educore-events-domain`**: the
  envelope crate is `educore-events` (this phase). The
  calendar domain is `educore-events-domain` (Phase 13).
  Two distinct crates with two distinct packages. Per
  `AGENTS.md` "Note on educore-events vs
  educore-events-domain" — the two are easy to confuse.
- **`educore-audit` retention volume**: every command
  writes one audit row. At scale (10k students × 5 daily
  commands × 200 schools = 10M rows/day), this needs
  partitioning by `(school_id, month)`. Document the
  strategy in `docs/schemas/audit-schema.md`.
- **PG RLS superuser bypass**: PG superusers bypass RLS
  by default. The cross-tenant isolation test (Phase 2
  exit criterion #3) must use a non-superuser role.
- **The `educore-events` envelope crate's `payload` type**:
  pick a `Box<dyn DomainEvent>` for trait-object
  dispatch, or a generic `EventEnvelope<T: DomainEvent>`
  for typed dispatch. The storage port already takes
  concrete structs (not generic envelopes); the bridge
  between the two is `educore-audit` / `educore-event-log`.
- **`educore-event-bus` impls**: the in-process impl is
  the default (test target). NATS / Redis impls are
  behind Cargo features so a contributor without those
  services can still build.
- **The Phase 1 SQL adapters' transaction model is
  flag-based**: each sub-port call opens its own short
  `pool.begin()`. Your audit / event log writers should
  rely on the same model; the engine's at-least-once
  dedup is the safety net.

## Exit Criteria

1. All 6 cross-cutting tables exercised in the integration
   test.
2. Outbox + audit_log + event_log all populated by a
   single command.
3. RLS is enforced on PG (the test uses a second
   `school_id` and asserts cross-tenant reads return
   zero rows).
4. `cargo test --workspace` green.
5. `cargo clippy --workspace --all-targets -- -D warnings`
   green.
6. `cargo fmt --all -- --check` green.
7. `cargo run -p educore-core --bin lint --features lint`
   clean.
8. The Phase 0 ad-hoc sync envelope refactor
   (`PHASE-0-HANDOFF.md` open question #2) is resolved:
   `educore-sync` depends on
   `educore_events::EventEnvelope`, not its own struct.
9. `docs/coverage.toml` rows for the 6 cross-cutting
   tables (the `audit_log_ddl_*` and `event_log_ddl_*`
   rows owned by `educore-audit` / `educore-events`) plus
   all platform / rbac / events / audit aggregate /
   command / event rows flipped to `Tested` with `tests`
   paths.
10. **Phase completion documentation** (per
      [`README.md`](README.md)
    convention):
    - `docs/handoff/PHASE-2-HANDOFF.md` written.
    - `docs/progress-tracker.md` updated (workspace
      status rows for the 5 new crates; phase progress
      row for Phase 2; coverage matrix summary).
    - `docs/build-plan.md` § "Phase 2" gets a
      `**Phase 2 outcome.**` subsection.
    - `docs/phase_prompt/phase-3-prompt.md` written for the
      academic-domain agent.

## When You Are Stuck

- Re-read `docs/handoff/PHASE-1-HANDOFF.md`; the SQL
  storage adapters it shipped are the persistence layer
  for your integration test.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The Phase 0 / Phase 1 commit history
  (`git log --oneline --grep="Phase 0"` /
  `--grep="Phase 1"`) is a working reference for the
  cross-cutting crate layout.
- For RLS, `docs/schemas/sql-dialects/postgresql.md` §
  "Row-level security" has the canonical `CREATE POLICY`
  + `ENABLE ROW LEVEL SECURITY` pattern.
- For design questions, do not invent — open an issue
  or ask the user. Phase 2 is execution, not design.
