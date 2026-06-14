# Educore Phase 8 — Facilities domain

## Mission
Deliver `educore-facilities` — dormitory, room, transport (route, vehicle), inventory (item, category, store, issue, receive, sell), supplier. **Implementation**, not design. Closes the finance→facilities bridge (finance `ProductPurchase` / `InventoryPayment` placeholders reference `item_id: Uuid`; Phase 8 ships the canonical `ItemId` and reconciles the 2 finance placeholders).

## Deliverables
`crates/domains/facilities/` with the headline 11 aggregates (`Dormitory`, `Room`, `Route`, `Vehicle`, `Item`, `ItemCategory`, `ItemStore`, `ItemIssue`, `ItemReceive`, `ItemSell`, `Supplier`) + 9-file layout. Wire events through `DomainEvent` + `EventBus`. Capability-gate via `Capability::Facilities*` (placeholder variants already in the enum). Vertical-slice test mirroring `finance_integration.rs`. Reconcile the 2 finance placeholder stubs (`ProductPurchase` + `InventoryPayment`) with the canonical `ItemId` (Q7 from the finance hand-off). Flip `coverage.toml` rows. Write `PHASE-8-HANDOFF.md` + `phase-9-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-7-HANDOFF.md` (10 OQs carry over; 4 directly affect Phase 8: Q2 the 33 placeholder backlog, Q5/Q7 the cross-crate id reconciliations, Q9 the proptest pattern)
- `docs/build-plan.md` § "Phase 8" + § "Phase 7 outcome."
- `docs/specs/facilities/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md`
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level security"
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/finance/src/` (the 9-file template — clone this; the inventory conservation invariant mirrors the double-entry invariant), `crates/domains/hr/src/`, `crates/domains/attendance/src/`, `crates/domains/assessment/src/`, `crates/domains/academic/src/`
- `crates/tools/storage-parity/tests/{finance,hr,attendance,assessment,academic}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
14 closed crates (6 cross-cutting + 5 domain + storage-parity + 1 settings) are the foundation. `educore-finance` → `ItemId` placeholder + the 2 placeholder stubs (`ProductPurchase`, `InventoryPayment`) to reconcile. `educore-hr` → `StaffId` canonical (the Phase 5/6 placeholder-replacement pattern). `educore-academic` → `StudentId`, `ClassId`, `AcademicYearId`. 3 SQL adapters are the persistence layer. `crates/domains/facilities/{Cargo.toml,lib.rs}` is the scaffold. The inventory conservation invariant (`on_hand = sum(received) - sum(issued) - sum(sold)`) is the headline correctness check (mirrors Phase 7's `sum(debits) == sum(credits)`).

## Working With Subagents
Workstreams: A=`Dormitory` + `Room` (canonical) + the per-room occupancy state machine; B=`Route` + `Vehicle` + transport-stop aggregates; C=`Item` + `ItemCategory` + `ItemStore` + the inventory conservation invariant; D=`ItemIssue` + `ItemReceive` + `ItemSell` + the per-school movement service; E=`Supplier` + supplier-payment tracking; F=reconcile the 2 finance placeholders (`ProductPurchase` + `InventoryPayment`); G=integration test + coverage flips + proptest for the conservation invariant.

## Per-Deliverable Gotchas
- Sixth domain crate — stick to 9-file layout.
- Inventory conservation: every `ItemIssue` / `ItemReceive` / `ItemSell` must conserve `on_hand = sum(received) - sum(issued) - sum(sold)` per `(school_id, item_id)`. Property test required (mirrors Phase 7's double-entry proptest).
- Reconcile the 2 finance placeholders: replace `item_id: Uuid` in `crates/domains/finance/src/aggregate.rs` with `pub use educore_facilities::value_objects::ItemId;` (the same Phase 5 → Phase 6 `StaffId` replacement pattern).
- The `ClassRoomId` placeholder in `educore-assessment` (Phase 4/5 OQ #6) was deferred to Phase 8; land the canonical `RoomId` in the facilities workstream, then add `pub use educore_facilities::value_objects::RoomId;` in the assessment crate's `value_objects.rs`.
- Concurrent `ItemIssue` / `ItemReceive` / `ItemSell` writes: the service runs in a transaction with `SELECT ... FOR UPDATE` on the `ItemStore` row (PG) or a SQLite write lock. Mitigate race conditions with a per-row lock.
- `transport` is policy-heavy: each school's route pricing + pickup-time policy is consumer-configured. Engine provides computation primitives.
- Event bus is the single source of truth; flag-based transaction model preserved.
- Mock the `payment` port in the integration test; real payment adapters land in Phase 15 (Port adapters).

## Exit Criteria
11 headline aggregates + 9-file layout; `services::receive_item` + `issue_item` + `sell_item` return `(Movement, MovementEvent)`; inventory conservation invariant property test green (100 cases); 2 finance placeholders reconciled with canonical `ItemId`; `ClassRoomId` placeholder in assessment replaced with `RoomId` re-export; every command calls capability + audit + bus + idempotency; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have rows for the school; `cargo test/clippy/fmt/lint --workspace` green; ≥ 11 `coverage.toml` rows flipped; `PHASE-8-HANDOFF.md` + `phase-9-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 8 outcome.".

## When You Are Stuck
`PHASE-7-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. PG RLS setup: `tools/scripts/pg-rls-test-setup.sql`. The 9-file template is `crates/domains/finance/src/`. Mock the payment port; real adapters are Phase 15. The proptest pattern is `crates/domains/finance/src/services.rs::DoubleEntryService::book_payment` (100 cases; `proptest = "1"` is already in the workspace `Cargo.toml`). The placeholder-replacement pattern is `crates/domains/attendance/src/value_objects.rs::StaffId` (the Phase 6 HR fix-up).
