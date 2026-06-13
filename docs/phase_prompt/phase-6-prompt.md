# Educore Phase 6 — HR domain

## Mission
Deliver `educore-hr` — staff, department, designation, leave, payroll. **Implementation**, not design.

## Deliverables
`crates/domains/hr/` with 6 aggregates (`Staff`, `Department`, `Designation`, `LeaveType`, `LeaveRequest`, `Payroll`) + 9-file layout. Wire events through `DomainEvent` + `EventBus`. Capability-gate via `Capability::HrStaff*` (placeholder variants already in the enum). Vertical-slice test mirroring `attendance_integration.rs`. Replace placeholder `StaffId` in assessment + attendance crates with re-exports of the HR crate's canonical `StaffId`. Flip `coverage.toml` rows. Write `PHASE-6-HANDOFF.md` + `phase-7-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-5-HANDOFF.md` (7 OQs carry over)
- `docs/build-plan.md` § "Phase 6"
- `docs/specs/hr/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md` (incl. bulk-insert path)
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level security"
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/attendance/src/` (the 9-file template — clone this), `crates/domains/assessment/src/`, `crates/domains/academic/src/`
- `crates/tools/storage-parity/tests/{attendance,assessment,academic}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
10 closed crates (6 cross-cutting + 3 domain + storage-parity) are the foundation. `educore-platform` → `TenantContext`, `SchoolId`, `UserId`. `educore-academic` → `AcademicYearId`. 3 SQL adapters are the persistence layer. `crates/domains/hr/{Cargo.toml,lib.rs}` is the scaffold.

## Working With Subagents
Workstreams: A=`Staff` (canonical) + `StaffId` typed id; B=`Department` + `Designation` + `LeaveType`; C=`LeaveRequest` + `Payroll`; D=`StaffId` replacement in assessment + attendance crates; E=integration test + bulk-payrun + coverage flips; F=leave-accrual + `PayrollPolicy` ports.

## Per-Deliverable Gotchas
- Fourth domain crate — stick to 9-file layout.
- Replace placeholder `StaffId` in assessment (`value_objects.rs`) + attendance (`value_objects.rs`) with `pub use educore_hr::value_objects::StaffId;` re-exports.
- Add `Department` / `Designation` / `LeaveType` / `LeaveRequest` to `AuditTarget` enum as a Prereq 2.
- `payroll` service is policy-heavy: `PayrollPolicy` port (per-school tax/allowance/deduction rules); tests wire `InMemoryPayrollPolicy` (10% tax).
- `leave_accrual` is a state machine: `Pending → Approved/Rejected → Cancelled/Completed`; service returns `Conflict` on illegal transitions.
- Event bus is the single source of truth.
- Flag-based transaction model preserved.

## Exit Criteria
6 aggregates + 9-file layout; `services::hire_staff` returns `(Staff, StaffHired)`; `services::run_payroll` + `approve_leave` work; every command calls capability + audit + bus + idempotency; `StaffId` replacement committed; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have rows for the school; `cargo test/clippy/fmt/lint --workspace` green; 6 `coverage.toml` rows flipped; `PHASE-6-HANDOFF.md` + `phase-7-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 6 outcome.".

## When You Are Stuck
`PHASE-5-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. PG RLS setup: `tools/scripts/pg-rls-test-setup.sql`. Bulk-insert pattern: add `bulk_insert_staff` (or `bulk_insert_payroll_lines`) to storage port + 3 SQL adapters following the Phase 5 Prereq 5 pattern.
