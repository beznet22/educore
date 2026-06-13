# Educore Phase 4 — Assessment domain

## Mission
Deliver `educore-assessment` — exams, marks, results, online exams, seat plans, admit cards, report cards. **Implementation**, not design.

## Deliverables
`crates/domains/assessment/` with 8 aggregates (`Exam`, `ExamSchedule`, `MarksRegister`, `ResultStore`, `ReportCard`, `OnlineExam`, `SeatPlan`, `AdmitCard`) + 9-file layout. Wire events through `DomainEvent` + `EventBus`. Capability-gate via `Capability::Assessment*`. Vertical-slice test mirroring `academic_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-4-HANDOFF.md` + `phase-5-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-3-HANDOFF.md` (6 OQs carry over)
- `docs/build-plan.md` § "Phase 4"
- `docs/specs/assessment/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md`
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/academic/src/` (the 9-file template — clone this)
- `crates/tools/storage-parity/tests/{academic,cross_cutting}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
6 closed crates (5 cross-cutting + `educore-academic`) are the foundation. `educore-academic` → `StudentId`, `AcademicYearId`, `ClassId`, `SectionId`, `SubjectId`. `educore-platform::services::create_school` is the template. 3 SQL adapters (PG/MySQL/SQLite) are the persistence layer. `crates/domains/assessment/{Cargo.toml,lib.rs}` is the scaffold.

## Working With Subagents
Workstreams: A=`Exam` (canonical) + commands/events/service/repo; B=`ExamSchedule` + `SeatPlan` + `AdmitCard`; C=`MarksRegister` + `ResultStore` + `ReportCard`; D=`OnlineExam` + integration test + coverage flips (depends on A).

## Per-Deliverable Gotchas
- Second domain crate — stick to the 9-file layout (assessment mirrors academic exactly).
- `Exam.code` unique within `(school_id, academic_year_id)`; the `AssessmentUniquenessChecker` port enforces this.
- `Capability::Assessment*` placeholders are in the enum (Phase 2); capability check works out of the box.
- `AuditTarget::Exam(Uuid)` and `AuditTarget::MarksRegister(Uuid)` for the two primary aggregates; the rest use `AuditTarget::Other("name".into(), id)`.
- Event bus is the single source of truth — no per-domain `broadcast::Sender`.
- Phase 1 storage adapters' flag-based transaction model is locked.
- `marks_register` row must not reference a deleted exam (invariant #3).
- PG RLS setup script (`tools/scripts/pg-rls-test-setup.sql`) is a Phase 4 deliverable; document in `docs/guides/saas-backend.md`.

## Exit Criteria
8 aggregates + 9-file layout; `services::create_exam` returns `(Exam, ExamCreated)`; every command calls capability + audit + bus + idempotency; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have 1 row for the school; `cargo test/clippy/fmt/lint --workspace` green; 8 `coverage.toml` rows flipped; `PHASE-4-HANDOFF.md` + `phase-5-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 4 outcome.".

## When You Are Stuck
`PHASE-3-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. `git log --oneline --grep="Phase"` is the working reference. PG RLS setup: `tools/scripts/pg-rls-test-setup.sql`.
