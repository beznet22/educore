# Educore Phase 2 — Cross-cutting foundations

## Mission
Deliver the cross-cutting foundations every domain crate depends on: `educore-platform`, `educore-rbac`, `educore-events` (envelope), `educore-event-bus`, `educore-audit`. **Implementation**, not design.

## Deliverables
- 5 cross-cutting crates per `docs/specs/{platform,rbac,events,audit}/`; DDL in `migrations/engine/0000_engine_core.*.sql` (pre-written)
- `educore-events::EventEnvelope<T: DomainEvent>` wired to storage-port `AuditLog::append` and `EventLog::append`
- Single end-to-end integration test exercising all 6 cross-cutting tables
- `docs/coverage.toml` flips + `PHASE-2-HANDOFF.md` + `phase-3-prompt.md`

## Required Reading
- `docs/handoff/PHASE-1-HANDOFF.md`, `docs/handoff/PHASE-0-HANDOFF.md`
- `docs/build-plan.md` § "Phase 2"
- `docs/ports/event-bus.md`, `docs/ports/storage.md`
- `docs/specs/platform/overview.md`, `docs/specs/rbac/overview.md`, `docs/specs/audit/overview.md`, `docs/specs/events/overview.md`
- `docs/schemas/audit-schema.md` § 13 (partitioning strategy for 10M rows/day)
- `docs/schemas/tenancy-schema.md` (school_id RLS / `TenantContext` contract)
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level security"
- `docs/schemas/event-schema.md`, `docs/schemas/database-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-014-Idempotency.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/`, `crates/infra/storage/src/port.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
3 SQL adapters from Phase 1 are the persistence layer. `educore-sync`'s ad-hoc `SyncEvent` refactor lands here (Phase 0 OQ #2). 6 cross-cutting DDL files are pre-written; you write Rust that writes to existing tables.

## Working With Subagents
Dispatch order: 1) `educore-events` (envelope; 1 subagent); 2) `educore-platform` + `educore-rbac` in parallel (2 subagents); 3) `educore-event-bus` (depends on envelope; 1 subagent); 4) `educore-audit` (depends on envelope; 1 subagent); 5) closing agent: integration test + coverage + hand-off + phase-3 prompt.

## Per-Deliverable Gotchas
- `educore-events` vs `educore-events-domain` (calendar, Phase 13) — two distinct crates.
- `educore-audit` retention volume (10M rows/day) — partitioning by `(school_id, month)`; document in `schemas/audit-schema.md`.
- PG RLS superuser bypass — cross-tenant test must use a non-superuser role.
- `educore-event-bus` impls: in-process is default; NATS / Redis behind Cargo features.
- Phase 1 SQL adapters' transaction model is flag-based; rely on the same model; at-least-once dedup is the safety net.

## Exit Criteria
6 cross-cutting tables exercised; outbox + audit + event log populated by one command; PG RLS enforced (cross-tenant read returns 0 rows); `cargo test/clippy/fmt --workspace` green; lint clean; Phase 0 sync envelope refactor done; `coverage.toml` rows flipped; `PHASE-2-HANDOFF.md` + `phase-3-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 2 outcome.".

## When You Are Stuck
`PHASE-1-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. RLS pattern: `schemas/sql-dialects/postgresql.md` § "Row-level security".
