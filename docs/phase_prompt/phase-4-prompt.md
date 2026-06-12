# Educore Phase 4 — Assessment domain

## Mission

You are continuing the Educore engine build-out.
**Phases 0, 1, 2, and 3 are closed.** Phase 3 delivered
the `educore-academic` domain crate (the first domain
crate). Your job is **Phase 4**: deliver the
`educore-assessment` domain crate — exams, marks, results,
online exams, seat plans, admit cards, report cards.

This is **implementation**, not design. The spec already
exists in `docs/specs/assessment/`. The 5 cross-cutting
crates (`educore-events`, `educore-event-bus`,
`educore-platform`, `educore-rbac`, `educore-audit`) and
the 1 domain crate (`educore-academic`) are the
foundation. The Phase 1 SQL storage adapters (PG, MySQL,
SQLite) are your persistence layer. The Phase 2
cross-cutting integration test
(`crates/tools/storage-parity/tests/cross_cutting_integration.rs`)
and the Phase 3 academic vertical-slice test
(`crates/tools/storage-parity/tests/academic_integration.rs`)
are your templates for the assessment vertical-slice
test.

You are NOT:
- Designing new ports or new aggregates beyond what the
  spec defines
- Modifying the Phase 1 storage adapters' flag-based
  transaction model (Phase 2 hand-off open question #5;
  the assessment vertical-slice test should validate the
  current design, not modify it)
- Modifying the 6 cross-cutting / academic crates'
  public surface (the foundation is locked)
- Building the other 8 domain crates (assessment is the
  Phase 4 deliverable; attendance, hr, finance, etc.
  are later phases)
- Adding new external crates without updating ADR-015
- Re-introducing `mysql_async` or `flate2` (rejected in
  Phase 1)

You ARE:
- Implementing `educore-assessment` per the spec in
  `docs/specs/assessment/` and the build-plan's Phase 4
  task list
- Wiring `ExamCreated` / `MarksRecorded` / `ResultPublished`
  (and any other assessment events) through
  `educore-events::DomainEvent` and
  `educore-events::EventBus`
- Gating the assessment command handlers with
  `educore-rbac::Capability::AssessmentExam*` checks
  (asserted at the test layer; the dispatcher-level
  boundary is the convention)
- Writing a vertical-slice integration test that mirrors
  `crates/tools/storage-parity/tests/academic_integration.rs`
  but exercises the assessment domain
- Flipping `docs/coverage.toml` rows `Pending` → `Tested`
  in the same commits as the impls
- Writing the Phase 4 hand-off (`PHASE-4-HANDOFF.md`) and
  the Phase 5 prompt (`docs/phase_prompt/phase-5-prompt.md`)
  at phase close (per the convention in
  [`README.md`](README.md))

## Deliverables

1. **`educore-assessment`** (`crates/domains/assessment/`)
   — the assessment domain crate. Implements:
   - The `Exam` aggregate (an exam definition: name,
     academic_year_id, subject_id, class_id, exam_date,
     total_marks, pass_mark, exam_type, is_published).
   - The `ExamSchedule` aggregate (when + where a
     specific exam is held; the schedule row).
   - The `MarksRegister` aggregate (one exam's marks for
     one section; the per-student marks).
   - The `ResultStore` aggregate (a published result
     row: a student's marks + computed grade for a
     specific exam).
   - The `ReportCard` aggregate (a per-student,
     per-term / per-year report card that aggregates
     marks across exams; consumed by the parent / student
     apps).
   - The `OnlineExam` aggregate (an online exam session
     with timed start / end + a list of question ids; the
     result is captured separately via the
     `MarksRegister`).
   - The `SeatPlan` aggregate (a per-exam seating
     arrangement; a student is assigned a room and seat
     number).
   - The `AdmitCard` aggregate (a per-student,
     per-exam admit card; a printable artifact that
     includes the photo, roll number, and exam schedule).
   - The matching command / event / service / repository
     / query / entities / value_objects / errors
     modules per the 9-file layout in `AGENTS.md`.

## Required Reading (priority order)

1. [`docs/handoff/PHASE-3-HANDOFF.md`](../handoff/PHASE-3-HANDOFF.md)
   — the prior hand-off. The 6 closed crates
   (5 cross-cutting + 1 domain) are the foundation. Read
   its "Open questions" section first; the 6 open
   questions are carry-overs that may affect Phase 4.
2. [`docs/build-plan.md`](../build-plan.md) § "Phase 4"
   — the canonical Phase 4 spec (the 5 tasks + 3 exit
   criteria + coverage matrix updates + risks).
3. [`docs/specs/assessment/`](../specs/assessment/) —
   the design contract. Skim all 11 files (the
   prompt-named subset is the 8 aggregates above; the
   spec has 8 aggregates in scope; the build-plan §
   "Phase 4" lists 5 tasks). The exit criteria list is
   "exams, marks, results" — wire the minimum to satisfy
   the exit criteria, scope-creep into report cards,
   online exams, seat plans, and admit cards as time
   permits.
4. [`docs/ports/event-bus.md`](../ports/event-bus.md) —
   the `EventBus` port contract that your events flow
   through.
5. [`docs/ports/storage.md`](../ports/storage.md) —
   the storage port contract; the 4 sub-ports you'll
   exercise (outbox, audit_log, event_log, idempotency).
6. [`crates/cross-cutting/events/src/lib.rs`](../../crates/cross-cutting/events/src/lib.rs)
   and `crates/cross-cutting/events/src/domain_event.rs`
   — the template for typed events. `DomainEvent` trait
   implementation is the primary entry point.
7. [`crates/cross-cutting/platform/src/services.rs`](../../crates/cross-cutting/platform/src/services.rs)
   and [`crates/cross-cutting/rbac/src/services.rs`](../../crates/cross-cutting/rbac/src/services.rs)
   — the templates for `services` factory functions and
   capability checks.
8. [`crates/cross-cutting/audit/src/writer.rs`](../../crates/cross-cutting/audit/src/writer.rs)
   — the `AuditWriter` service (the audit-sink entry
   point).
9. [`crates/domains/academic/src/`](../../crates/domains/academic/src/) —
   the first domain crate. The 9-file module layout,
   the `#[allow(clippy::too_many_arguments)]` pattern on
   event constructors, the `fresh_etag` helper, the
   `DomainEvent` impl pattern, and the service function
   shape are all Phase 3 templates you should mirror
   exactly. **Do not deviate from the 9-file layout.**
10. [`crates/tools/storage-parity/tests/academic_integration.rs`](../../crates/tools/storage-parity/tests/academic_integration.rs)
    — the Phase 3 vertical-slice test pattern. Your
    assessment test is a clone of this, with
    `SchoolCreated` → `ExamCreated` (or whatever the
    spec calls the initial event).
11. [`crates/tools/storage-parity/tests/cross_cutting_integration.rs`](../../crates/tools/storage-parity/tests/cross_cutting_integration.rs)
    — the original Phase 2 vertical-slice test pattern.
12. [`AGENTS.md`](../../AGENTS.md) — workspace rules,
    naming, lint policy, the 9-file module layout per
    domain.
13. [`docs_guidlines/system.md`](../../docs_guidlines/system.md)
    + [`docs_guidlines/execution_guidlines.md`](../../docs_guidlines/execution_guidlines.md)
    — engineering standards.

## Starting Point

- The 6 closed crates (5 cross-cutting + 1 domain
  `educore-academic`) are your foundation. Specifically:
  - `educore-events` — provides `DomainEvent` trait,
    `EventEnvelope` struct, `EventBus` port. Your events
    implement `DomainEvent` and are published via the bus.
  - `educore-event-bus` — provides the `InProcessEventBus`
    (default, always built) for tests. NATS / Redis
    impls exist but are stubs.
  - `educore-platform` — provides `TenantContext`,
    `SchoolId`, `UserId`, the in-memory uniqueness
    checker pattern (use it for `Exam.code` /
    `Exam.name` per-academic-year uniqueness).
  - `educore-rbac` — provides `Capability` (the
    `Assessment.*` placeholder variants were added in
    Phase 2: `Capability::AssessmentExam{Create,Read,Update,Delete}`,
    `Capability::AssessmentMarksRegister{Create,...}`),
    `CapabilityCheck` port, `InMemoryCapabilityCheck`
    test impl. Your command handlers MUST call
    `capability_check.has(ctx, Capability::AssessmentExam*)`
    (asserted at the test layer).
  - `educore-audit` — provides `AuditWriter`. Every
    assessment command writes an audit row via the
    writer.
  - `educore-academic` — provides `StudentId` and
    `AcademicYearId` (the assessment domain references
    both: marks and results are per-student and
    per-academic-year). Do NOT modify `educore-academic`'s
    public surface; just `use` the types you need.
- The 3 SQL storage adapters (PG, MySQL, SQLite) are
  the persistence layer. The Phase 3 academic
  integration test uses the SQLite in-memory adapter
  for the always-on test variant; copy that pattern.
- The `educore-platform::services::create_school`
  function is the template for the assessment
  `services::create_exam` / `record_marks` / etc.
  functions (pure factory that returns the mutated
  aggregate + the typed event).
- The existing `crates/domains/assessment/Cargo.toml`
  is a scaffold (no deps yet) and `src/lib.rs` is just
  a `PACKAGE_NAME` const. Start from there.

## Working With Subagents

Phase 4 has multiple independent deliverables
(`Exam` aggregate, `MarksRegister` aggregate,
`ResultStore` aggregate, the `services` factory
functions, the integration test). The closing agent
writes the next-phase prompt at the close of every
phase, and the convention is that the **receiving**
agent uses the task tool to spawn parallel subagents
for those workstreams. This is a hard rule, not a tip.

Per the README convention, your Phase 4 workstreams:

- **Workstream A**: `Exam` aggregate + commands +
  events + service + repository (the canonical
  deliverable; the exam is the unit that everything
  else attaches to).
- **Workstream B**: `ExamSchedule` + `SeatPlan` +
  `AdmitCard` aggregates (per-exam operational data;
  the seat plan and admit card are printed artefacts).
- **Workstream C**: `MarksRegister` + `ResultStore` +
  `ReportCard` aggregates (the marks flow:
  enter marks → compute result → publish → assemble
  report card).
- **Workstream D**: `OnlineExam` aggregate + the
  vertical-slice integration test + `docs/coverage.toml`
  flips (depends on A; uses the academic integration
  test as a template).

## Per-Deliverable Gotchas

- **`educore-assessment` is the second domain crate.**
  The patterns established in `educore-academic` (the
  9-file module layout, the `fresh_etag` helper, the
  `#[allow(clippy::too_many_arguments)]` on event
  constructors, the `EventEnvelope` round-trip
  through `into_envelope`, the `UniquenessChecker`
  port for per-academic-year / per-class uniqueness)
  propagate here. Stick to the layout exactly.

- **The `Exam` aggregate's identity is
  per-academic-year.** Per the spec, `Exam.code` is
  unique within `(school_id, academic_year_id)`. The
  `ExamId(SchoolId, Uuid)` is the global id; the
  `code` is the unique key for human reference.

- **`Capability::AssessmentExamCreate` is a placeholder
  variant** in the current `educore-rbac::Capability`
  enum (added in Phase 2). The capability check just
  works out of the box.

- **The audit writer is called from every command
  handler.** The pattern is:
  ```rust
  audit_writer.write(
      &ctx,
      AuditAction::Create,
      AuditTarget::Exam(aggregate_id),
      None,                              // before
      Some(serialized_after),             // after
  ).await?;
  ```
  The `AuditTarget` enum has an `Exam(Uuid)` variant
  (added in Phase 2's `educore-audit::writer`). For
  `MarksRegister` and `ResultStore`, use
  `AuditTarget::MarksRegister(aggregate_id)` and a
  custom `AuditTarget::Other("result_store".into(),
  aggregate_id)`.

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

- **References to `educore-academic` types.** The
  assessment domain references `StudentId` and
  `AcademicYearId` from the academic crate. These
  cross-crate references are wired via the workspace
  `Cargo.toml` (the `domains` tier can depend on other
  `domains` crates per `AGENTS.md` § "Tier System").
  Do not duplicate the type definitions in the
  assessment crate.

## Exit Criteria

1. The 8 aggregates ship with the matching command +
   event + service + repository + query + entities +
   value_objects + errors modules (the full 9-file
   layout).
2. `services::create_exam` returns `(Exam,
   ExamCreated)` and the `Exam` row is created through
   `ExamRepository::insert`.
3. `services::record_marks` returns
   `(MarksRegister, MarksRecorded)` and the row is
   created through `MarksRegisterRepository::insert`.
4. Every assessment command handler:
   - Calls `capability_check.has(ctx,
     Capability::AssessmentExam*)` (asserted at the
     test layer; production wiring documented as
     dispatcher-level).
   - Calls `audit_writer.write(...)` after the
     mutation.
   - Publishes the event via `bus.publish(envelope)`.
   - Records the idempotency key.
5. Vertical-slice integration test passes on SQLite
   (always), PG and MySQL (env-gated). All 4 sub-ports
   have exactly one row for the school.
6. `cargo test --workspace` green.
7. `cargo clippy --workspace --all-targets -- -D warnings`
   green.
8. `cargo fmt --all -- --check` green.
9. `cargo run -p educore-core --bin lint --features lint`
   clean.
10. `docs/coverage.toml` rows for the 8 assessment
    aggregates flipped to `Tested` with `tests` paths,
    in the same commits as the impls.
11. **Phase completion documentation** (per
    [`README.md`](README.md)
    convention):
    - `docs/handoff/PHASE-4-HANDOFF.md` written.
    - `docs/phase_prompt/phase-5-prompt.md` written for
      the attendance-domain agent.
    - `docs/progress-tracker.md` updated (workspace
      status row for `educore-assessment`; phase
      progress row for Phase 4; coverage matrix
      summary).
    - `docs/build-plan.md` § "Phase 4" gets a
      `**Phase 4 outcome.**` subsection.

## When You Are Stuck

- Re-read `docs/handoff/PHASE-3-HANDOFF.md`; the 6
  closed crates it shipped are the foundation.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The Phase 0 / Phase 1 / Phase 2 / Phase 3 commit
  history (`git log --oneline --grep="Phase"`) is a
  working reference for the cross-cutting + domain
  crate layout.
- For RLS on PG, the Phase 2 hand-off open question
  #2 documents the `tools/scripts/pg-rls-test-setup.sql`
  gap. **Phase 4 should add the script and document
  the procedure in `docs/guides/saas-backend.md`**
  (this was deferred from Phase 3 and is a Phase 4
  deliverable).
- For the audit log partitioning strategy, see
  `docs/schemas/audit-schema.md` § 13. The assessment
  `Exam` aggregate produces a high-volume event
  stream (10k students × 5 exams/year × 200 schools
  = 10M marks events/year) — the partitioning strategy
  must be in place before the SaaS backend launches.
- For the "Is the SQL adapter's flag-based transaction
  model safe for the assessment domain?" question, see
  Phase 2 hand-off open question #5. The Phase 3
  vertical-slice test validated the current design
  for `Student`; the Phase 4 test should validate
  the current design for `MarksRegister` and
  `ResultStore`. If the test shows inconsistency,
  document the finding in `PHASE-4-HANDOFF.md` (the
  fix is a real `sqlx::Transaction` plumbed through the
  sub-port methods, which is a separate refactor).
