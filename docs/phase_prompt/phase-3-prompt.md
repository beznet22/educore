# Educore Phase 3 — Academic domain

## Mission
Deliver `educore-academic` — the first of 10 domain bounded contexts. **Implementation**, not design.

## Deliverables
`crates/domains/academic/` with the 5 prompt-named aggregates (`Student`, `Class`, `Section`, `Subject`, `AcademicYear`) + 9-file layout. Wire `StudentCreated` (and other academic events) through `DomainEvent` + `EventBus`. Capability-gate via `Capability::AcademicStudent*`. Vertical-slice integration test mirroring `cross_cutting_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-3-HANDOFF.md` + `phase-4-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-2-HANDOFF.md` (6 OQs carry over)
- `docs/build-plan.md` § "Phase 3"
- `docs/specs/academic/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md`
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/tools/storage-parity/tests/cross_cutting_integration.rs` (vertical-slice test template)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
5 cross-cutting crates (Phase 2) are the foundation. `educore-events` → `DomainEvent` + `EventEnvelope` + `EventBus`. `educore-platform` → `TenantContext` + uniqueness-checker pattern. `educore-rbac` → `Capability` (placeholders for all 14 domains pre-added). `educore-audit` → `AuditWriter`. 3 SQL adapters (PG/MySQL/SQLite) are the persistence layer; SQLite in-memory is the always-on test variant. `crates/domains/academic/{Cargo.toml,lib.rs}` is the scaffold.

## Working With Subagents
Workstreams: A=`Student` (canonical) + commands/events/service/repo; B=`Class` + `Section`; C=`Subject` + `AcademicYear`; D=integration test + coverage flips (depends on A).

## Per-Deliverable Gotchas
- First domain crate — patterns propagate to the other 9. Stick to 9-file layout (no `prelude.rs`, no `state.rs`).
- `StudentId(SchoolId, Uuid)` — school is part of the typed id (cross-tenant compile-time safety).
- `Capability::AcademicStudent*` placeholders already in the enum; no Prereq 1 needed.
- `AuditTarget::Student(Uuid)` is the variant for student audits.
- Event bus is the single source of truth — no per-domain `broadcast::Sender`.
- Phase 1 storage adapters' flag-based transaction model is locked for this phase.

## Exit Criteria
5 aggregates + 9-file layout; `services::admit_student` returns `(Student, StudentAdmitted)`; every command calls capability + audit + bus + idempotency; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have 1 row for the school; `cargo test/clippy/fmt/lint --workspace` green; 5 `coverage.toml` rows flipped; `PHASE-3-HANDOFF.md` + `phase-4-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 3 outcome.".

## When You Are Stuck
`PHASE-2-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. `git log --oneline --grep="Phase"` is the working reference. PG RLS setup script gap: `tools/scripts/pg-rls-test-setup.sql`.
