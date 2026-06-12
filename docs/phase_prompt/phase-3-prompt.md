# Educore Phase 3 — Academic domain

## Mission

You are continuing the Educore engine build-out. Phase 0
(PR 0 + PR A, see
[`docs/handoff/PHASE-0-HANDOFF.md`](../handoff/PHASE-0-HANDOFF.md)),
**Phase 1 (storage adapter parity, see
[`docs/handoff/PHASE-1-HANDOFF.md`](../handoff/PHASE-1-HANDOFF.md)),
and Phase 2 (cross-cutting foundations, see
[`docs/handoff/PHASE-2-HANDOFF.md`](../handoff/PHASE-2-HANDOFF.md))**
are closed. Your job is **Phase 3**: deliver the
`educore-academic` domain crate — the first of the 10 domain
bounded contexts.

This is **implementation**, not design. The spec already
exists in `docs/specs/academic/`. The 5 cross-cutting crates
(`educore-events`, `educore-event-bus`, `educore-platform`,
`educore-rbac`, `educore-audit`) are the foundation. The
Phase 1 SQL storage adapters (PG, MySQL, SQLite) are your
persistence layer. The Phase 2 cross-cutting integration
test (`crates/tools/storage-parity/tests/cross_cutting_integration.rs`)
is your template for the academic vertical-slice test.

You are NOT:
- Designing new ports or new aggregates beyond what the
  spec defines
- Modifying the Phase 1 storage adapters' flag-based
  transaction model (Phase 2 hand-off open question #5;
  the academic vertical-slice test should validate the
  current design, not modify it)
- Modifying the 5 cross-cutting crates' public surface
  (the foundation is locked)
- Building the other 9 domain crates (academic is the
  Phase 3 deliverable; assessment, attendance, cms, etc.
  are later phases)
- Adding new external crates without updating ADR-015
- Re-introducing `mysql_async` or `flate2` (rejected in
  Phase 1)

You ARE:
- Implementing `educore-academic` per the spec in
  `docs/specs/academic/` and the build-plan's Phase 3
  task list
- Wiring `StudentCreated` (and any other academic
  events) through `educore-events::DomainEvent` and
  `educore-events::EventBus`
- Gating the academic command handlers with
  `educore-rbac::Capability::AcademicStudent*` checks
- Writing a vertical-slice integration test that mirrors
  Phase 2's `cross_cutting_integration_sqlite` test
  but exercises the academic domain
- Flipping `docs/coverage.toml` rows `Pending` → `Tested`
  in the same commits as the impls
- Writing the Phase 3 hand-off (`PHASE-3-HANDOFF.md`) and
  the Phase 4 prompt (`docs/phase_prompt/phase-4-prompt.md`)
  at phase close (per the convention in
  [`README.md`](README.md))

## Deliverables

1. **`educore-academic`** (`crates/domains/academic/`) —
   the academic domain crate. Implements:
   - The `Student` aggregate (the canonical Phase 3
     deliverable per the spec).
   - The `Class` aggregate (a school-scoped group of
     students meeting on a schedule).
   - The `Section` aggregate (an instance of a class:
     a specific teacher, room, time slot, and academic
     year).
   - The `Subject` aggregate (a course subject like
     "Mathematics" or "English").
   - The `AcademicYear` aggregate (the school's
     academic year; typically one per school per year).
   - The matching command / event / service / repository
     / query / entities / value_objects / errors
     modules per the 9-file layout in `AGENTS.md`.

## Required Reading (priority order)

1. [`docs/handoff/PHASE-2-HANDOFF.md`](../handoff/PHASE-2-HANDOFF.md)
   — the prior hand-off. The 5 cross-cutting crates it
   shipped are the foundation. Read its "Open questions"
   section first; the 6 open questions are carry-overs
   that may affect Phase 3.

2. [`docs/build-plan.md`](../build-plan.md) § "Phase 3" —
   the canonical Phase 3 spec (the 4-5 tasks + 4 exit
   criteria + coverage matrix updates + risks).

3. [`docs/specs/academic/`](../specs/academic/) — the
   design contract. Skim all 11 files (the prompt-named
   subset is the 5 aggregates above; the spec has
   ~32 aggregates; Phase 3 ships the prompt-named
   subset only, the same scope discipline as Phase 2).

4. [`docs/ports/event-bus.md`](../ports/event-bus.md) —
   the `EventBus` port contract that your events flow
   through.

5. [`docs/ports/storage.md`](../ports/storage.md) —
   the storage port contract; the 4 sub-ports you'll
   exercise (outbox, audit_log, event_log, idempotency).

6. [`crates/cross-cutting/events/src/lib.rs`](../../crates/cross-cutting/events/src/lib.rs)
   and `crates/cross-cutting/events/src/sync.rs` — the
   template for typed events. `DomainEvent` trait
   implementation is the primary entry point.

7. [`crates/cross-cutting/platform/src/services.rs`](../../crates/cross-cutting/platform/src/services.rs)
   and `crates/cross-cutting/rbac/src/services.rs` —
   the templates for `services` factory functions and
   capability checks.

8. [`crates/tools/storage-parity/tests/cross_cutting_integration.rs`](../../crates/tools/storage-parity/tests/cross_cutting_integration.rs)
   — the vertical-slice test pattern. Your academic
   test is a clone of this, with `SchoolCreated` →
   `StudentAdmitted` (or whatever the spec calls the
   initial student-creation event).

9. [`AGENTS.md`](../../AGENTS.md) — workspace rules,
   naming, lint policy, the 9-file module layout per
   domain.

10. [`docs_guidlines/system.md`](../../docs_guidlines/system.md)
    + [`docs_guidlines/execution_guidlines.md`](../../docs_guidlines/execution_guidlines.md)
    — engineering standards.

## Starting Point

- The 5 cross-cutting crates from Phase 2 are your
  foundation. Specifically:
  - `educore-events` — provides `DomainEvent` trait,
    `EventEnvelope` struct, `EventBus` port. Your events
    implement `DomainEvent` and are published via the
    bus.
  - `educore-event-bus` — provides the
    `InProcessEventBus` (default, always built) for
    tests. NATS / Redis impls exist but are stubs.
  - `educore-platform` — provides `TenantContext`,
    `SchoolId`, `UserId`, the in-memory uniqueness
    checker pattern (use it for student
    `register_number` / `email` uniqueness).
  - `educore-rbac` — provides `Capability` (55 variants
    including `AcademicStudent{Create,Read,Update,Delete}`),
    `CapabilityCheck` port, `InMemoryCapabilityCheck` test
    impl. Your command handlers MUST call
    `capability_check.has(ctx, Capability::AcademicStudentCreate)`
    before mutating the `Student` aggregate.
  - `educore-audit` — provides `AuditWriter`. Every
    student command writes an audit row via the writer.
- The 3 SQL storage adapters (PG, MySQL, SQLite) are
  the persistence layer. The Phase 2 cross-cutting
  integration test uses the SQLite in-memory adapter
  for the always-on test variant; copy that pattern.
- The `educore-platform::services::create_school`
  function is the template for the academic
  `services::admit_student` / `register_class` /
  etc. functions (pure factory that returns the mutated
  aggregate + the typed event).
- The existing `crates/domains/academic/Cargo.toml` is
  a scaffold (no deps yet) and `src/lib.rs` is just a
  `PACKAGE_NAME` const. Start from there.

## Working With Subagents

Phase 3 has multiple independent deliverables
(`Student` aggregate, `Class` aggregate, `Section`
aggregate, `Subject` aggregate, `AcademicYear`
aggregate, the integration test). The closing agent
writes the next-phase prompt at the close of every
phase, and the convention is that the **receiving**
agent uses the task tool to spawn parallel subagents
for those workstreams. This is a hard rule, not a tip.

Per the README convention, your Phase 3 workstreams:

- **Workstream A**: `Student` aggregate + commands +
  events + service + repository (the canonical
  deliverable).
- **Workstream B**: `Class` + `Section` aggregates
  (class scheduling).
- **Workstream C**: `Subject` + `AcademicYear`
  aggregates (the academic catalog).
- **Workstream D**: vertical-slice integration test
  + `docs/coverage.toml` flips (depends on A; uses
  the Phase 2 integration test as a template).

## Per-Deliverable Gotchas

- **`educore-academic` is the first domain crate.** The
  engine has 9 more to go (assessment, attendance, cms,
  communication, documents, finance, hr, library,
  facilities). Patterns you establish here will be
  repeated. Stick to the 9-file module layout exactly
  (`AGENTS.md` § "Module Layout (per domain)"). No
  `lib.rs` shenanigans, no extra modules, no
  per-aggregate subfolders.

- **The `Student` aggregate's identity is per-school.**
  Per the spec, `StudentId(SchoolId, Uuid)` — the school
  is part of the typed id. This is the
  cross-tenant-compile-time-safety pattern the engine
  uses everywhere. Do NOT use a global
  `StudentId(Uuid)` — that would let a school pass its
  student id to another school's student, which the
  type system should catch.

- **`Capability::AcademicStudentCreate` is a placeholder
  variant** in the current `educore-rbac::Capability`
  enum. Phase 2 added placeholders for all 14 domains
  so you don't need to add new variants. The
  capability check just works out of the box.

- **The audit writer is called from every command
  handler.** The pattern is:
  ```rust
  audit_writer.write(
      &ctx,
      AuditAction::Create,
      AuditTarget::Student(aggregate_id),
      None,                              // before
      Some(serialized_after),             // after
  ).await?;
  ```
  The `AuditTarget` enum has a `Student(Uuid)` variant
  (added in Phase 2's `educore-audit::writer`).

- **The event bus is the single source of truth for
  event delivery.** Do NOT add a per-domain
  broadcast::Sender or mpsc::channel. The Phase 0
  `educore-sync` ad-hoc envelope pattern is the
  cautionary tale; the bus-port contract is the law.

- **The Phase 1 storage adapters' transaction model is
  flag-based.** Each sub-port call opens its own short
  `pool.begin()`. Your command handlers should rely on
  the same model; the engine's at-least-once dedup is
  the safety net. Do NOT add a real
  `sqlx::Transaction` parameter to the sub-port methods
  in this phase; that's a separate refactor (Phase 2
  hand-off open question #5).

- **The 9-file module layout is mandatory.** `lib.rs`
  re-exports the public surface; the other 8 files
  contain the actual code. No `prelude.rs`, no
  `state.rs`, no `state_machine.rs`. Just the 9 files.

## Exit Criteria

1. `Student`, `Class`, `Section`, `Subject`,
   `AcademicYear` aggregates implemented per the spec
   (prompt-named subset; the other 27 academic
   aggregates in the spec land in later phases).
2. The `Student` aggregate is admitted via a
   `services::admit_student` factory function that
   returns the `Student` + `StudentAdmitted` event, and
   the `Student` row is created through the
   `StudentRepository` port.
3. Every student command handler:
   - Calls `capability_check.has(ctx,
     Capability::AcademicStudentCreate)` (or
     `Update` / `Delete`) before mutating.
   - Calls `audit_writer.write(...)` after the
     mutation.
   - Publishes the event via `bus.publish(envelope)`.
   - Records the idempotency key.
4. Vertical-slice integration test passes on SQLite
   (always), PG and MySQL (env-gated). The test
   creates a `Student` via the academic service and
   asserts the 4 sub-ports (outbox, audit_log,
   event_log, idempotency) each have exactly one row
   for the school.
5. `cargo test --workspace` green.
6. `cargo clippy --workspace --all-targets -- -D warnings`
   green.
7. `cargo fmt --all -- --check` green.
8. `cargo run -p educore-core --bin lint --features lint`
   clean.
9. `docs/coverage.toml` rows for the 5 academic
   aggregates flipped to `Tested` with `tests` paths,
   in the same commits as the impls.
10. **Phase completion documentation** (per
    [`README.md`](README.md)
    convention):
    - `docs/handoff/PHASE-3-HANDOFF.md` written.
    - `docs/progress-tracker.md` updated (workspace
      status row for `educore-academic`; phase progress
      row for Phase 3; coverage matrix summary).
    - `docs/build-plan.md` § "Phase 3" gets a
      `**Phase 3 outcome.**` subsection.
    - `docs/phase_prompt/phase-4-prompt.md` written for
      the assessment-domain agent.

## When You Are Stuck

- Re-read `docs/handoff/PHASE-2-HANDOFF.md`; the 5
  cross-cutting crates it shipped are the foundation.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The Phase 0 / Phase 1 / Phase 2 commit history
  (`git log --oneline --grep="Phase N"`) is a working
  reference for the cross-cutting crate layout.
- For RLS on PG, the Phase 2 hand-off open question
  #2 documents the `tools/scripts/pg-rls-test-setup.sql`
  gap. If your academic integration test needs a
  PG-RLS variant, write the setup script first.
- For the audit log partitioning strategy, see
  `docs/schemas/audit-schema.md` § 13 (added in
  Phase 2). The academic `Student` aggregate produces
  the highest-volume event stream (10k students × 5
  daily commands × 200 schools = 10M rows/day) — the
  partitioning strategy must be in place before the
  SaaS backend launches.
- For the "Is the SQL adapter's flag-based transaction
  model safe for the academic domain?" question, see
  Phase 2 hand-off open question #5. The Phase 3
  vertical-slice test should validate the current
  design; if it shows inconsistency, document the
  finding in `PHASE-3-HANDOFF.md` (the fix is a real
  `sqlx::Transaction` plumbed through the sub-port
  methods, which is a separate refactor).
