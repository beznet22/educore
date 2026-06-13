# Educore Phase 5 — Attendance domain

## Mission
Deliver `educore-attendance` — student, staff, subject, exam attendance, bulk-marking. **Implementation**, not design.

## Deliverables
`crates/domains/attendance/` with 4 prompt-named aggregates (`StudentAttendance`, `StaffAttendance`, `SubjectAttendance`, `ExamAttendance`) + 9-file layout. Wire events through `DomainEvent` + `EventBus`. Capability-gate via `Capability::Attendance.*` (added in Prereq 1). Vertical-slice test mirroring `assessment_integration.rs`. Bulk-marking service (CSV import + per-class UI) with the `bulk_insert` path benchmarked at 200 rows in <100ms on PG. Flip `coverage.toml` rows. Write `PHASE-5-HANDOFF.md` + `phase-6-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-4-HANDOFF.md` (10 OQs carry over)
- `docs/build-plan.md` § "Phase 5"
- `docs/specs/attendance/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md` (incl. bulk-insert path)
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level security"
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/assessment/src/` (the 9-file template — clone this), `crates/domains/academic/src/`
- `crates/tools/storage-parity/tests/{assessment,academic,cross_cutting}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
9 closed crates (6 cross-cutting + 2 domain + storage-parity) are the foundation. `educore-academic` → `StudentId`, `StaffId` (placeholder), `ClassId`, `SectionId`, `SubjectId`, `StudentRecordId`, `AcademicYearId`. `educore-assessment` → `ExamId` (for `ExamAttendance`). 3 SQL adapters are the persistence layer. `crates/domains/attendance/{Cargo.toml,lib.rs}` is the scaffold.

## Working With Subagents
Workstreams: A=`StudentAttendance` (canonical) + bulk-marking service; B=`StaffAttendance` + `SubjectAttendance`; C=`ExamAttendance`; D=integration test + bulk-insert bench + coverage flips (depends on A).

## Per-Deliverable Gotchas
- Third domain crate — stick to 9-file layout (attendance mirrors assessment).
- `StudentAttendance` is unique per `(student_id, attendance_date)`; the `AttendanceUniquenessChecker` port enforces this.
- Prereq 1: add ~24 `Capability::Attendance.*` variants (Student×4 + Subject×5 + Staff×4 + Import×4 + Exam×4 + BulkMark + Notify + Report).
- Prereq 2: add `SubjectAttendance` + `StaffAttendance` + `BulkAttendanceImport` + `ClassAttendance` to `AuditTarget` enum.
- `bulk_insert_student_attendances` (Prereq 5): single multi-row `INSERT` on PG / MySQL, transaction-grouped inserts on SQLite. 200 rows in <100ms on PG.
- `ExamAttendance` ships in the attendance crate (cross-crate dep to assessment) since the assessment crate is locked; deferred to a follow-up phase.
- Event bus is the single source of truth — no per-domain `broadcast::Sender`.
- Flag-based transaction model is preserved; bulk-insert is an additive non-breaking change.

## Exit Criteria
4 aggregates + 9-file layout; `services::mark_student_attendance` returns `(StudentAttendance, StudentAttendanceMarked)`; bulk-marking benchmark green; every command calls capability + audit + bus + idempotency; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have rows for the school; `cargo test/clippy/fmt/lint --workspace` green; 13 `coverage.toml` rows flipped (7 aggregates + 6 events); `PHASE-5-HANDOFF.md` + `phase-6-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 5 outcome.".

## When You Are Stuck
`PHASE-4-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. PG RLS setup: `tools/scripts/pg-rls-test-setup.sql`. Bulk-insert pattern: single multi-row `INSERT` (PG / MySQL) or transaction-grouped inserts (SQLite); profile with `cargo bench`.
