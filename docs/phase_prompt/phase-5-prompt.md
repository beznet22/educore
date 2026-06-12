# Educore Phase 5 — Attendance domain

## Mission

You are continuing the Educore engine build-out. Phases 0
(foundation), 1 (storage adapter parity), 2 (cross-cutting
foundations), 3 (academic domain), and 4 (assessment domain)
are closed. Your job is **Phase 5**: deliver the
`educore-attendance` domain crate — student, staff, subject,
exam attendance.

This is **implementation**, not design. The spec already
exists in `docs/specs/attendance/`. The 6 cross-cutting
crates (`educore-events`, `educore-event-bus`,
`educore-platform`, `educore-rbac`, `educore-audit`), the
2 domain crates (`educore-academic`, `educore-assessment`),
and the 3 SQL storage adapters (PG, MySQL, SQLite) are your
foundation. The Phase 4 vertical-slice test
(`crates/tools/storage-parity/tests/assessment_integration.rs`)
and the Phase 3 vertical-slice test
(`crates/tools/storage-parity/tests/academic_integration.rs`)
are your templates for the new attendance vertical-slice test.

You are NOT:
- Designing new ports or new aggregates beyond what the
  spec defines
- Modifying the Phase 1 storage adapters' flag-based
  transaction model (Phase 2 hand-off open question #5; the
  Phase 3 + Phase 4 vertical-slice tests validate the current
  design; Phase 5 should validate it for the attendance
  domain the same way)
- Modifying the 8 closed foundation crates' public surface
- Modifying the 2 closed domain crates' public surface
- Building the other 8 domain crates (Phase 5 is attendance;
  CMS, finance, library, etc. are later phases)
- Adding new external crates without updating ADR-015
- Re-introducing `mysql_async` or `flate2` (rejected in Phase 1)

You ARE:
- Implementing `educore-attendance` per the spec in
  `docs/specs/attendance/` and the build-plan's Phase 5
  task list
- Wiring `StudentAttended` / `StaffAttended` /
  `SubjectAttended` / `ExamAttended` (and any other
  attendance events) through `educore-events::DomainEvent`
  and `educore-events::EventBus`
- Gating the attendance command handlers with
  `educore-rbac::Capability::Attendance.*` checks (asserted
  at the test layer; the dispatcher-level boundary is the
  convention)
- Writing a vertical-slice integration test that mirrors
  `crates/tools/storage-parity/tests/assessment_integration.rs`
  but exercises the attendance domain
- Flipping `docs/coverage.toml` rows `Pending` → `Tested` in
  the same commits as the impls
- Writing the Phase 5 hand-off (`PHASE-5-HANDOFF.md`) and
  the Phase 6 prompt (`docs/phase_prompt/phase-6-prompt.md`)
  at phase close (per the convention in `README.md`)

## Deliverables

1. **`educore-attendance`** (`crates/domains/attendance/`)
   — the attendance domain crate. Implements:
   - The `StudentAttendance` aggregate (per-student
     per-day attendance record: present, absent, late,
     half-day, holiday, with reason + check-in / check-out
     timestamps).
   - The `StaffAttendance` aggregate (per-staff
     per-day attendance record; the staff equivalent
     of `StudentAttendance`).
   - The `SubjectAttendance` aggregate (per-student
     per-subject attendance — a student's attendance in
     a specific subject on a specific day, used by
     schools that track attendance at the period /
     subject level rather than the day level).
   - The `ExamAttendance` aggregate (per-exam
     per-student attendance — was the student present
     for the exam; the consumer's grading pipeline
     uses this to flag absent students; the assessment
     domain's `ResultStore` reads from this aggregate
     during `publish_result`).
   - The matching command / event / service / repository
     / query / entities / value_objects / errors
     modules per the 9-file layout in `AGENTS.md`.
   - A bulk-marking command (CSV import + per-class UI)
     per the build-plan § Phase 5 task 2. The
     `educore-storage` bulk-insert path is exercised here
     for the first time at scale.
   - The `AttendanceUniquenessChecker` port trait (the
     per-day uniqueness check the bulk-marking service
     calls to enforce one-record-per-student-per-day).

## Required Reading (priority order)

1. [`docs/handoff/PHASE-4-HANDOFF.md`](../handoff/PHASE-4-HANDOFF.md)
   — the prior hand-off. The 8 closed foundation + domain
   crates are the foundation. Read its "Open questions"
   section first; the 6 Phase 2 OQs + the 4 Phase 4
   hand-off additions are carry-overs that may affect
   Phase 5.
2. [`docs/build-plan.md`](../build-plan.md) § "Phase 5" —
   the canonical Phase 5 spec (the 4 tasks + 4 exit
   criteria + coverage matrix updates + risks).
3. [`docs/specs/attendance/`](../specs/attendance/) —
   the design contract. Skim all 11 files (the
   prompt-named subset is the 4 aggregates above; the
   spec has ~12 aggregates; Phase 5 ships the
   prompt-named subset only, the same scope discipline
   as Phase 3 + Phase 4).
4. [`docs/ports/event-bus.md`](../ports/event-bus.md) —
   the `EventBus` port contract that your events flow
   through.
5. [`docs/ports/storage.md`](../ports/storage.md) —
   the storage port contract; the 4 sub-ports you'll
   exercise (outbox, audit_log, event_log, idempotency)
   + the bulk-insert path.
6. [`crates/cross-cutting/events/src/lib.rs`](../../crates/cross-cutting/events/src/lib.rs)
   and `crates/cross-cutting/events/src/domain_event.rs` —
   the template for typed events. `DomainEvent` trait
   implementation is the primary entry point.
7. [`crates/cross-cutting/platform/src/services.rs`](../../crates/cross-cutting/platform/src/services.rs)
   and [`crates/cross-cutting/rbac/src/services.rs`](../../crates/cross-cutting/rbac/src/services.rs) —
   the templates for `services` factory functions and
   capability checks.
8. [`crates/cross-cutting/audit/src/writer.rs`](../../crates/cross-cutting/audit/src/writer.rs) —
   the `AuditWriter` service (the audit-sink entry point).
9. [`crates/domains/assessment/src/`](../../crates/domains/assessment/src/) —
   the Phase 4 template (the assessment crate is the
   most recent full-prompt-scope domain crate; the 9-file
   module layout, the `fresh_etag` helper, the
   `#[allow(clippy::too_many_arguments)]` on event
   constructors, the `EventEnvelope` round-trip through
   `into_envelope`, the `UniquenessChecker` port for
   per-academic-year / per-class uniqueness, and the
   `full_workflow_test` pattern all apply). **Do not
   deviate from the 9-file layout.**
10. [`crates/tools/storage-parity/tests/assessment_integration.rs`](../../crates/tools/storage-parity/tests/assessment_integration.rs) —
    the Phase 4 vertical-slice test. Your attendance
    test is a clone of this, with `create_exam` →
    `mark_student_attendance` (or whatever the spec calls
    the initial attendance-marking event).
11. [`crates/tools/storage-parity/tests/cross_cutting_integration.rs`](../../crates/tools/storage-parity/tests/cross_cutting_integration.rs) —
    the original Phase 2 vertical-slice test pattern.
12. [`crates/tools/storage-parity/tests/academic_integration.rs`](../../crates/tools/storage-parity/tests/academic_integration.rs) —
    the Phase 3 vertical-slice test pattern.
13. [`AGENTS.md`](../../AGENTS.md) — workspace rules,
    naming, lint policy, the 9-file module layout per
    domain.
14. [`docs/ports/storage.md`](../ports/storage.md) — the
    bulk-insert path is exercised for the first time at
    scale in Phase 5 task 2. The hand-off recommends
    using a single multi-row `INSERT` for PG, or
    transaction-grouped inserts for SQLite, per the
    build-plan § Phase 5 risks. **Add a benchmark in
    `tests/benches/` (200 rows in <100 ms on PG) per
    the build-plan's exit criteria.**
15. [`docs_guidlines/system.md`](../../docs_guidlines/system.md)
    + [`docs_guidlines/execution_guidlines.md`](../../docs_guidlines/execution_guidlines.md)
    — engineering standards.

## Starting Point

- The 8 closed crates (5 cross-cutting + 1 academic
  domain + 1 assessment domain + the storage-parity
  tools crate) are your foundation. Specifically:
  - `educore-events` — `DomainEvent` trait, `EventEnvelope`,
    `EventBus` port. Your events implement `DomainEvent`
    and are published via the bus.
  - `educore-event-bus` — `InProcessEventBus` (default,
    always built) for tests. NATS / Redis impls exist but
    are stubs.
  - `educore-platform` — `TenantContext`, `SchoolId`,
    `UserId`, the in-memory uniqueness checker pattern
    (use it for `AttendanceUniquenessChecker`).
  - `educore-rbac` — `Capability` (the `Attendance.*`
    placeholder variants were added in Phase 4 — wait,
    they weren't; only `CapabilityDomain::Attendance`
    was. Phase 5 needs to add the actual `Attendance.*`
    capability variants as a prereq, similar to Phase 4
    Prereq 1 for Assessment). `CapabilityCheck` port,
    `InMemoryCapabilityCheck` test impl. Your command
    handlers MUST call
    `capability_check.has(ctx, Capability::Attendance*)`
    (asserted at the test layer).
  - `educore-audit` — `AuditWriter`. Every attendance
    command writes an audit row via the writer.
  - `educore-academic` — `StudentId`, `StaffId`
    (placeholder), `ClassId`, `SectionId`, `SubjectId`,
    `AcademicYearId`, `StudentRecordId`. Your attendance
    aggregates reference these foreign keys.
  - `educore-assessment` — `ExamId` (your `ExamAttendance`
    aggregate references exams by id).
- The 3 SQL storage adapters (PG, MySQL, SQLite) are
  the persistence layer. The Phase 4 academic + assessment
  integration tests use the SQLite in-memory adapter for
  the always-on test variant; copy that pattern.
- The `educore-platform::services::create_school` function
  is the template for the attendance `services::mark_*`
  factory functions (pure factory that returns the
  mutated aggregate + the typed event).
- The existing `crates/domains/attendance/Cargo.toml` is
  a scaffold (no deps yet) and `src/lib.rs` is just a
  `PACKAGE_NAME` const. Start from there.

## Working With Subagents

Phase 5 has multiple independent deliverables
(`StudentAttendance` aggregate, `StaffAttendance`
aggregate, `SubjectAttendance` aggregate, `ExamAttendance`
aggregate, the bulk-marking command, the integration
test). The closing agent writes the next-phase prompt at
the close of every phase, and the convention is that the
**receiving** agent uses the task tool to spawn parallel
subagents for those workstreams. This is a hard rule, not
a tip.

Per the README convention, your Phase 5 workstreams:

- **Workstream A**: `StudentAttendance` aggregate
  (canonical; the unit of bulk-marking; the reference
  pattern for the other 3).
- **Workstream B**: `StaffAttendance` +
  `SubjectAttendance` aggregates (per-staff and
  per-subject attendance records).
- **Workstream C**: `ExamAttendance` aggregate (the
  per-exam per-student attendance that the assessment
  domain's `ResultStore::publish` reads from).
- **Workstream D**: bulk-marking command + the
  vertical-slice integration test + `docs/coverage.toml`
  flips (depends on A; uses the assessment / academic
  integration test as a template).

## Per-Deliverable Gotchas

- **`educore-attendance` is the third domain crate.** The
  engine has 7 more to go (CMS, finance, library, etc.).
  Patterns established here will be repeated. Stick to
  the 9-file module layout exactly (`AGENTS.md` §
  "Module Layout (per domain)"). No `lib.rs` shenanigans,
  no extra modules, no per-aggregate subfolders.

- **The `StudentAttendance` aggregate's identity is
  per-day.** Per the spec, `StudentAttendance` is uniquely
  identified within a school by
  `(student_id, attendance_date)`. The
  `AttendanceUniquenessChecker` port enforces this at the
  service layer (the service is called by
  `mark_student_attendance` and by the bulk-marking
  service).

- **`Capability::Attendance.*` is a placeholder variant**
  in the current `educore-rbac::Capability` enum (similar
  to how `Capability::Assessment.*` was a placeholder
  before Phase 4 Prereq 1). Phase 5 needs to add the
  actual capability variants as a prereq. Add ~16 new
  variants (4 aggregates × 4 CRUD).

- **The audit writer is called from every command
  handler.** The pattern is:
  ```rust
  audit_writer.write(
      &ctx,
      AuditAction::Create,
      AuditTarget::StudentAttendance(aggregate_id),
      None,                              // before
      Some(serialized_after),             // after
  ).await?;
  ```
  The `AuditTarget` enum currently has `StudentAttendance`
  and `StaffAttendance` (added in Phase 4) but needs
  `SubjectAttendance` and `ExamAttendance` (Phase 5
  Prereq 2).

- **The event bus is the single source of truth for
  event delivery.** Do NOT add a per-domain
  broadcast::Sender or mpsc::channel. The Phase 0
  `educore-sync` ad-hoc envelope pattern is the
  cautionary tale; the bus-port contract is the law.

- **The Phase 1 storage adapters' transaction model is
  flag-based.** Each sub-port call opens its own short
  `pool.begin()`. The bulk-marking service relies on the
  at-least-once dedup via `idempotency_key` for the
  bulk-insert path. The Phase 5 hand-off recommends
  using a single multi-row `INSERT` for PG and
  transaction-grouped inserts for SQLite. The build-plan's
  exit criteria add a **200 rows in <100 ms on PG**
  benchmark in `tests/benches/`.

- **The 9-file module layout is mandatory.** `lib.rs`
  re-exports the public surface; the other 8 files
  contain the actual code. No `prelude.rs`, no
  `state.rs`, no `state_machine.rs`. Just the 9 files.

- **References to `educore-academic` / `educore-assessment`
  types.** The attendance domain references `StudentId`
  and `StaffId` (placeholder) from academic, and `ExamId`
  from assessment. These cross-crate references are wired
  via the workspace `Cargo.toml` (the `domains` tier can
  depend on other `domains` tier crates per `AGENTS.md`
  § "Tier System"). Do not duplicate the type
  definitions in the attendance crate.

- **The `exam_attendance` aggregate is consumed by the
  assessment domain's `ResultStore::publish`.** The
  `publish_result` service reads from
  `ExamAttendanceRepository` to flag absent students.
  Phase 5 ships the `ExamAttendance` aggregate; the
  assessment-side consumption lands in a follow-up phase
  (the assessment crate is locked for the current Phase
  4; the follow-up is likely Phase 6 alongside the HR
  workstream).

## Exit Criteria

1. The 4 aggregates ship with the matching command +
   event + service + repository + query + entities +
   value_objects + errors modules (the full 9-file
   layout).
2. `services::mark_student_attendance` returns
   `(StudentAttendance, StudentAttended)` and the row is
   created through `StudentAttendanceRepository::insert`.
3. The bulk-marking service (CSV import + per-class UI)
   is implemented and benchmarked: 200 rows in <100 ms
   on PG (per the build-plan's exit criteria). The
   benchmark lives in `crates/tools/storage-parity/benches/`
   (or `crates/domains/attendance/benches/`).
4. Every attendance command handler:
   - Calls `capability_check.has(ctx,
     Capability::Attendance*)` (asserted at the test
     layer; production wiring documented as
     dispatcher-level).
   - Calls `audit_writer.write(...)` after the mutation.
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
10. `docs/coverage.toml` rows for the 4 attendance
    aggregates flipped to `Tested` with `tests` paths,
    in the same commits as the impls.
11. **Phase completion documentation** (per
    [`README.md`](README.md)
    convention):
    - `docs/handoff/PHASE-5-HANDOFF.md` written.
    - `docs/phase_prompt/phase-6-prompt.md` written for
      the HR-domain agent.
    - `docs/progress-tracker.md` updated (workspace
      status row, phase progress row, coverage matrix
      summary).
    - `docs/build-plan.md` § "Phase 5" gets a
      `**Phase 5 outcome.**` subsection.

## When You Are Stuck

- Re-read `docs/handoff/PHASE-4-HANDOFF.md`; the 8
  closed crates it shipped are the foundation.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The Phase 0 / Phase 1 / Phase 2 / Phase 3 / Phase 4
  commit history (`git log --oneline --grep="Phase"`) is
  a working reference for the cross-cutting + domain
  crate layout.
- For RLS on PG, the Phase 4 hand-off § "Prereq 5"
  documents the procedure; the setup script is at
  `tools/scripts/pg-rls-test-setup.sql`. Run it before
  the PG variant of the attendance integration test.
- For the bulk-insert performance, the build-plan §
  Phase 5 risks and exit criteria add the
  200-rows-in-<100-ms-on-PG benchmark. Use a single
  multi-row `INSERT` (PG) or transaction-grouped inserts
  (SQLite). Profile with `cargo bench`.
- For the "Is the flag-based transaction model safe for
  the attendance domain?" question, see Phase 2 hand-off
  § OQ #5. Phase 3 + Phase 4 validated the model for
  academic and assessment; Phase 5 should validate it
  for attendance the same way. If the test shows
  inconsistency, document the finding in
  `PHASE-5-HANDOFF.md`.
