# Educore Phase 6 — HR domain

## Mission

You are continuing the Educore engine build-out. Phases 0
(foundation), 1 (storage adapter parity), 2 (cross-cutting
foundations), 3 (academic domain), 4 (assessment domain),
and 5 (attendance domain) are closed. Your job is **Phase
6**: deliver the `educore-hr` domain crate — staff,
department, designation, leave, payroll.

This is **implementation**, not design. The spec already
exists in `docs/specs/hr/`. The 6 closed cross-cutting
crates (`educore-events`, `educore-event-bus`,
`educore-platform`, `educore-rbac`, `educore-audit`), the
3 closed domain crates (`educore-academic`,
`educore-assessment`, `educore-attendance`), and the 3 SQL
storage adapters (PG, MySQL, SQLite) are your foundation.
The Phase 5 vertical-slice test
(`crates/tools/storage-parity/tests/attendance_integration.rs`),
the Phase 4 vertical-slice test
(`crates/tools/storage-parity/tests/assessment_integration.rs`),
and the Phase 3 vertical-slice test
(`crates/tools/storage-parity/tests/academic_integration.rs`)
are your templates for the new HR vertical-slice test.

You are NOT:
- Designing new ports or new aggregates beyond what the
  spec defines
- Modifying the Phase 1 storage adapters' flag-based
  transaction model (Phase 2 hand-off open question #5; the
  Phase 3 + Phase 4 + Phase 5 vertical-slice tests validate
  the current design; Phase 6 should validate it for the HR
  domain the same way)
- Modifying the 8 closed foundation + 3 closed domain
  crates' public surface
- Building the other 7 domain crates (Phase 6 is HR;
  facilities, library, CMS, etc. are later phases)
- Adding new external crates without updating ADR-015
- Re-introducing `mysql_async` or `flate2` (rejected in
  Phase 1)
- Modifying the placeholder `StaffId` typed id in the
  assessment or attendance crates (Phase 6 replaces both
  with re-exports of the HR crate's canonical `StaffId`)

You ARE:
- Implementing `educore-hr` per the spec in
  `docs/specs/hr/` and the build-plan's Phase 6 task list
- Wiring `StaffHired` / `StaffTerminated` / `LeaveRequested` /
  `LeaveApproved` / `LeaveRejected` / `PayrollRun` / `PayrollLineComputed`
  (and any other HR events) through
  `educore-events::DomainEvent` and `educore-events::EventBus`
- Replacing the placeholder `StaffId` typed id in the
  assessment + attendance crates with re-exports of the
  HR crate's canonical `StaffId`
- Gating the HR command handlers with
  `educore-rbac::Capability::HrStaff*` checks (asserted
  at the test layer; the dispatcher-level boundary is the
  convention)
- Writing a vertical-slice integration test that mirrors
  `crates/tools/storage-parity/tests/attendance_integration.rs`
  but exercises the HR domain
- Flipping `docs/coverage.toml` rows `Pending` → `Tested`
  in the same commits as the impls
- Writing the Phase 6 hand-off (`PHASE-6-HANDOFF.md`) and
  the Phase 7 prompt (`docs/phase_prompt/phase-7-prompt.md`)
  at phase close (per the convention in
  [`README.md`](README.md))

## Deliverables

1. **`educore-hr`** (`crates/domains/hr/`) — the HR
   domain crate. Implements:
   - The `Staff` aggregate (the canonical staff record:
     staff_id, school_id, staff_code, first_name,
     last_name, date_of_birth, date_of_joining,
     department_id, designation_id, employment_status,
     + the 10-field audit footer).
   - The `Department` aggregate (a school department:
     name, code, head_staff_id, active_status).
   - The `Designation` aggregate (a job title:
     name, code, level, active_status).
   - The `LeaveType` aggregate (a leave category: name,
     code, days_per_year, is_paid, requires_approval,
     active_status).
   - The `LeaveRequest` aggregate (a staff leave request:
     staff_id, leave_type_id, from_date, to_date,
     reason, status, approver_id, + the 10-field audit
     footer).
   - The `Payroll` aggregate (a payroll run: period,
     staff_id, base_salary, allowances, deductions,
     net_pay, status, + the 10-field audit footer).
   - The matching command / event / service / repository
     / query / entities / value_objects / errors
     modules per the 9-file layout in `AGENTS.md`.
   - The `payroll` service: a pure factory that
     computes the per-line payroll (base + allowances
     - deductions = net). Production wires a per-school
     `PayrollPolicy` port; tests wire an
     `InMemoryPayrollPolicy` (default tax/allowance rules).

2. **StaffId replacement** (2 small commits): replace
   the placeholder `StaffId` typed id in the assessment
   + attendance crates with re-exports of the HR crate's
   canonical `StaffId`. Mirror the `StudentRecordId` pattern
   from Phase 4 Prereq 3.

3. **Integration test: hire a staff member, request
   leave, approve it, run payroll.** Verify outbox +
   audit + RLS. The vertical-slice test follows the
   `attendance_integration.rs` template.

4. **Phase completion documentation.** When the phase
   closes, write `docs/handoff/PHASE-6-HANDOFF.md`
   (mirroring the `PHASE-5-HANDOFF.md` template: status,
   what's wired, what's stubbed, open questions, phase-7
   entry point, where NOT to start, key files, where to
   ask). Update `docs/progress-tracker.md` (workspace
   status row, phase progress row, coverage matrix
   summary). Add a `**Phase 6 outcome.**` subsection to
   this build plan (between `**Risks.**` and the trailing
   `---`). Create `docs/phase_prompt/phase-7-prompt.md`
   for the next-phase agent (per the convention in
   `docs/phase_prompt/README.md`).

## Required Reading (priority order)

1. [`docs/handoff/PHASE-5-HANDOFF.md`](../handoff/PHASE-5-HANDOFF.md)
   — the prior hand-off. The 9 closed foundation + 3
   closed domain crates are the foundation. Read its
   "Open questions" section first; the 7 Phase 5 OQs +
   the 6 Phase 4 hand-off additions are carry-overs that
   may affect Phase 6.
2. [`docs/build-plan.md`](../build-plan.md) § "Phase 6" —
   the canonical Phase 6 spec (the 4 tasks + 1 exit
   criteria + coverage matrix updates + risks).
3. [`docs/specs/hr/`](../specs/hr/) — the design contract.
   Skim all 11 files (the prompt-named subset is the 6
   aggregates above; the spec has 6 aggregates in scope;
   the build-plan § "Phase 6" lists 4 tasks).
4. [`docs/ports/event-bus.md`](../ports/event-bus.md) —
   the `EventBus` port contract that your events flow
   through.
5. [`docs/ports/storage.md`](../ports/storage.md) —
   the storage port contract; the 4 sub-ports you'll
   exercise (outbox, audit_log, event_log, idempotency).
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
9. [`crates/domains/attendance/src/`](../../crates/domains/attendance/src/) —
   the Phase 5 template (the attendance crate is the most
   recent full-prompt-scope domain crate; the 9-file
   module layout, the `fresh_etag` helper, the
   `#[allow(clippy::too_many_arguments)]` on event
   constructors, the `EventEnvelope` round-trip through
   `into_envelope`, the `AttendanceUniquenessChecker` port,
   and the `full_workflow_test` pattern all apply).
   **Do not deviate from the 9-file layout.**
10. [`crates/tools/storage-parity/tests/attendance_integration.rs`](../../crates/tools/storage-parity/tests/attendance_integration.rs) —
    the Phase 5 vertical-slice test. Your HR test is a
    clone of this, with `bulk_mark_student_attendance` →
    `hire_staff` (or whatever the spec calls the initial
    staff-hiring event).
11. [`crates/tools/storage-parity/tests/assessment_integration.rs`](../../crates/tools/storage-parity/tests/assessment_integration.rs) —
    the Phase 4 vertical-slice test pattern.
12. [`crates/tools/storage-parity/tests/academic_integration.rs`](../../crates/tools/storage-parity/tests/academic_integration.rs) —
    the Phase 3 vertical-slice test pattern.
13. [`AGENTS.md`](../../AGENTS.md) — workspace rules,
    naming, lint policy, the 9-file module layout per
    domain.
14. [`docs/ports/storage.md`](../ports/storage.md) — the
    bulk-insert path is exercised in the attendance crate;
    Phase 6 may add a new `bulk_insert_staff` method
    following the same pattern (the build-plan's
    `tests/benches/` directory hosts the
    200-rows-in-<100ms benchmarks).
15. [`docs_guidlines/system.md`](../../docs_guidlines/system.md)
    + [`docs_guidlines/execution_guidlines.md`](../../docs_guidlines/execution_guidlines.md) —
    engineering standards.

## Working With Subagents

Phase 6 has multiple independent deliverables (the 6
aggregates, the StaffId replacement, the integration test,
the hand-off). The closing agent writes the next-phase
prompt at the close of every phase, and the convention is
that the **receiving** agent uses the task tool to spawn
parallel subagents for those workstreams. This is a hard
rule, not a tip.

Per the README convention, your Phase 6 workstreams:

- **Workstream A**: `Staff` aggregate (the canonical;
  the unit of payroll + leave; the reference pattern for
  the other 5). Plus the `StaffId` typed id in
  `crates/domains/hr/src/value_objects.rs` (the canonical
  `StaffId` that the assessment + attendance placeholders
  re-export).
- **Workstream B**: `Department` + `Designation` +
  `LeaveType` aggregates (the reference-data aggregates
  the `Staff` aggregate's foreign keys point at).
- **Workstream C**: `LeaveRequest` + `Payroll` aggregates
  (the workflow aggregates; the leave-approval service
  is a state machine; the payroll service is a pure
  factory that consumes the per-school `PayrollPolicy`
  port).
- **Workstream D**: `StaffId` replacement in assessment +
  attendance crates (2 small commits that re-export the
  HR crate's canonical `StaffId`; the assessment +
  attendance placeholders go away).
- **Workstream E**: bulk-mark payroll + the
  vertical-slice integration test +
  `docs/coverage.toml` flips (depends on A + B + C; uses
  the attendance / assessment / academic integration
  tests as templates).
- **Workstream F**: leave-accrual + payroll services
  (the per-school policy ports; production wires
  `PayrollPolicy` from a Phase 14 Settings aggregate).

## Per-Deliverable Gotchas

- **`educore-hr` is the fourth domain crate.** The
  engine has 6 more to go (facilities, library,
  communication, documents, CMS, events-domain).
  Patterns established here will be repeated. Stick to
  the 9-file module layout exactly (`AGENTS.md` §
  "Module Layout (per domain)"). No `lib.rs` shenanigans,
  no extra modules, no per-aggregate subfolders.

- **`Staff` is the canonical aggregate.** The HR
  `Staff` aggregate is the source of truth for the
  `StaffId` typed id. The placeholder `StaffId` typed
  ids in the assessment + attendance crates (lines
  151–156 of `crates/domains/assessment/src/value_objects.rs`
  + the attendance crate's `StaffId` placeholder in
  `crates/domains/attendance/src/value_objects.rs`) are
  replaced with `pub use educore_hr::value_objects::StaffId;`
  re-exports in Workstream D.

- **`Capability::HrStaff.*` is already in the enum.**
  The 4 placeholder `HrStaff{Create,Read,Update,Delete}`
  capabilities were added in Phase 2 (assessment values_objects
  lines 297–299 of `crates/cross-cutting/rbac/src/value_objects.rs`).
  No Prereq 1 needed for the Staff capability; but
  Phase 6 may need to add more HR capabilities (e.g.
  `HrLeave*` for leave-management, `HrPayroll*` for
  payroll-run management). The spec's
  `docs/specs/hr/permissions.md` is the source of truth.

- **The audit writer is called from every command
  handler.** The pattern is:
  ```rust
  audit_writer.write(
      &ctx,
      AuditAction::Create,
      AuditTarget::Staff(aggregate_id),
      None,                              // before
      Some(serialized_after),             // after
  ).await?;
  ```
  The `AuditTarget` enum already has `Staff(Uuid)` and
  `Payroll(Uuid)` (added in Phase 1/2). Phase 6 may need
  to add `Department(Uuid)`, `Designation(Uuid)`,
  `LeaveType(Uuid)`, `LeaveRequest(Uuid)` as a Prereq 2.

- **The event bus is the single source of truth for
  event delivery.** Do NOT add a per-domain
  broadcast::Sender or mpsc::channel. The Phase 0
  `educore-sync` ad-hoc envelope pattern is the
  cautionary tale; the bus-port contract is the law.

- **The Phase 1 storage adapters' transaction model is
  flag-based.** Each sub-port call opens its own short
  `pool.begin()`. The leave-approval + payroll services
  rely on the at-least-once dedup via `idempotency_key`.
  The Phase 5 hand-off's answer to the Phase 2 OQ #5
  question is **yes**, the design is adequate for the
  attendance domain, and the same applies to HR (the
  bulk-insert path is exercised for the first time at
  scale in the payroll bulk-payrun service).

- **The 9-file module layout is mandatory.** `lib.rs`
  re-exports the public surface; the other 8 files
  contain the actual code. No `prelude.rs`, no
  `state.rs`, no `state_machine.rs`. Just the 9 files.

- **References to `educore-academic` / `educore-assessment`
  / `educore-attendance` types.** The HR domain references
  `SchoolId`, `UserId`, `TenantContext` from the platform
  crate; `StudentId` is not used in HR. The HR's
  `Staff` aggregate references `DepartmentId` +
  `DesignationId` (local typed ids). Cross-crate
  references are wired via the workspace `Cargo.toml` (the
  `domains` tier can depend on other `domains` tier crates
  per `AGENTS.md` § "Tier System"). Do not duplicate the
  type definitions in the HR crate.

- **The `payroll` service is policy-heavy.** The
  `PayrollPolicy` port returns the per-school
  tax/allowance/deduction rules. Production wires a
  `PayrollPolicy` from a Phase 14 Settings aggregate;
  tests wire an `InMemoryPayrollPolicy` (default
  10% tax, no allowances, no deductions). The service
  is pure (no I/O); the port is the only I/O surface.

- **The `leave_accrual` service is a state machine.**
  `LeaveRequest` transitions through
  `Pending → Approved/Rejected → Cancelled/Completed`.
  The transition invariants are enforced in the service
  (return `DomainError::Conflict` on illegal transitions).

## Exit Criteria

1. The 6 aggregates ship with the matching command +
   event + service + repository + query + entities +
   value_objects + errors modules (the full 9-file
   layout).
2. `services::hire_staff` returns
   `(Staff, StaffHired)` and the row is created through
   `StaffRepository::insert`.
3. The payroll service is implemented: `run_payroll`
   returns `(Payroll, PayrollComputed, Vec<PayrollLine>)`
   after asserting the per-school `PayrollPolicy`.
4. The leave-approval service is implemented:
   `approve_leave` returns `LeaveApproved` and mutates
   the `LeaveRequest` aggregate.
5. Every HR command handler:
   - Calls `capability_check.has(ctx,
     Capability::HrStaff*)` (asserted at the test
     layer; production wiring documented as
     dispatcher-level).
   - Calls `audit_writer.write(...)` after the mutation.
   - Publishes the event via `bus.publish(envelope)`.
   - Records the idempotency key.
6. The placeholder `StaffId` typed id in the assessment
   + attendance crates is replaced with re-exports of
   the HR crate's canonical `StaffId`.
7. Vertical-slice integration test passes on SQLite
   (always), PG and MySQL (env-gated). All 4 sub-ports
   have exactly one row for the school.
8. `cargo test --workspace` green.
9. `cargo clippy --workspace --all-targets -- -D warnings`
   green.
10. `cargo fmt --all -- --check` green.
11. `cargo run -p educore-core --bin lint --features lint`
    clean.
12. `docs/coverage.toml` rows for the 6 HR aggregates
    flipped to `Tested` with `tests` paths, in the same
    commits as the impls.
13. **Phase completion documentation** (per
    [`README.md`](README.md) convention):
    - `docs/handoff/PHASE-6-HANDOFF.md` written.
    - `docs/phase_prompt/phase-7-prompt.md` written for
      the Finance-domain agent.
    - `docs/progress-tracker.md` updated (workspace
      status row, phase progress row, coverage matrix
      summary).
    - `docs/build-plan.md` § "Phase 6" gets a
      `**Phase 6 outcome.**` subsection.

## When You Are Stuck

- Re-read `docs/handoff/PHASE-5-HANDOFF.md`; the 9
  closed crates it shipped are the foundation.
- The `educore-core::lint` binary is the no-gaps gate:
  `cargo run -p educore-core --bin lint --features lint`.
- The Phase 0 / Phase 1 / Phase 2 / Phase 3 / Phase 4 /
  Phase 5 commit history (`git log --oneline --grep="Phase"`)
  is a working reference for the cross-cutting + domain
  crate layout.
- For RLS on PG, the Phase 4 hand-off § "Prereq 5"
  documents the procedure; the setup script is at
  `tools/scripts/pg-rls-test-setup.sql`. Run it before
  the PG variant of the HR integration test.
- For bulk-insert performance, follow the Phase 5
  Prereq 5 pattern: add a new `bulk_insert_staff` (or
  `bulk_insert_payroll_lines`) method on the storage
  port + 3 SQL adapters. Use a single multi-row
  `INSERT` (PG / MySQL) or transaction-grouped inserts
  (SQLite). Profile with `cargo bench`.
- For the "Is the flag-based transaction model safe for
  the HR domain?" question, see Phase 2 hand-off § OQ
  #5. Phase 3 + Phase 4 + Phase 5 validated the model
  for academic / assessment / attendance; Phase 6
  should validate it for HR the same way. If the test
  shows inconsistency, document the finding in
  `PHASE-6-HANDOFF.md`.
