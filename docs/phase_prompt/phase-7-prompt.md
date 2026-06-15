# Educore Phase 7 — Finance domain

## Mission
Deliver `educore-finance` — fees, payments, refunds, expenses, wallet, payroll accounting. **Implementation**, not design. Closes the HR→finance bridge (HR `PayrollPaid` → finance `PayrollPaymentRecorded` → HR payroll status = `Paid`).

## Deliverables
`crates/domains/finance/` with the headline 6 aggregates (`FeesInvoice`, `FeesPayment`, `Refund`, `Expense`, `Wallet`, `WalletTransaction`) + 9-file layout. Wire events through `DomainEvent` + `EventBus`. Capability-gate via `Capability::Finance*` (placeholder variants already in the enum). Vertical-slice test mirroring `hr_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-7-HANDOFF.md` + `phase-8-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-6-HANDOFF.md` (4 OQs carry over)
- `docs/build-plan.md` § "Phase 7" + § "Phase 6 outcome."
- `docs/specs/finance/` (all 11 files: overview, aggregates, events, commands, services, repositories, value-objects, entities, tables, permissions, workflows)
- `docs/ports/event-bus.md`, `docs/ports/storage.md`, `docs/ports/payment.md`
- `docs/schemas/tenancy-schema.md`, `docs/schemas/audit-schema.md` § 13
- `docs/schemas/sql-dialects/postgresql.md` § "Row-level security"
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`
- `crates/cross-cutting/platform/src/services.rs`, `crates/cross-cutting/rbac/src/services.rs`
- `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/hr/src/` (the 9-file template — clone this; largest spec to date), `crates/domains/attendance/src/`, `crates/domains/assessment/src/`, `crates/domains/academic/src/`
- `crates/tools/storage-parity/tests/{hr,attendance,assessment,academic}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
13 closed crates (6 cross-cutting + 4 domain + storage-parity + 1 settings) are the foundation. `educore-hr` → `PayrollGenerate`, `StaffId`, `PayrollPolicy` port. `educore-academic` → `StudentId`, `ClassId`, `AcademicYearId`. 3 SQL adapters are the persistence layer. `crates/domains/finance/{Cargo.toml,lib.rs}` is the scaffold. The double-entry invariant (`sum(debits) == sum(credits)` per `school_id`) is the headline correctness check.

## Working With Subagents
Workstreams: A=`FeesInvoice` (canonical) + `FeesPayment` + double-entry invariant; B=`Wallet` + `WalletTransaction` + refund flows; C=`Expense` + `Income` + bank-account aggregate; D=HR→finance payroll bridge (HR `PayrollPaid` → finance `PayrollPaymentRecorded`); E=integration test + coverage flips; F=`DefaultRoleCatalog` extension + carry-forward rules + late-fee computation.

## Per-Deliverable Gotchas
- Fifth domain crate — stick to 9-file layout.
- `payroll` (finance-side accounting record) is **distinct from** HR's `PayrollGenerate`; finance owns the chart-of-accounts write. The HR→finance bridge subscribes to `hr.payroll.paid` and emits `finance.payroll_payment.recorded`.
- Double-entry invariant: every `FeesPayment` writes one `debit` and one `credit` row; sum must balance per `school_id`. Property test required.
- `payroll_accounting` is regulatory — engine provides computation primitives; legal/tax/banking configuration is the consumer's responsibility.
- Carry-forward rules (per `docs/specs/finance/services.md`) need a unit test per rule.
- Event bus is the single source of truth; flag-based transaction model preserved.
- Mock the `payment` port in the integration test; real payment adapters land in Phase 15 (Port adapters).

## Exit Criteria
6 headline aggregates + 9-file layout; `services::record_payment` returns `(FeesPayment, FeesPaymentRecorded)`; `services::run_carry_forward` + `compute_late_fee` work; every command calls capability + audit + bus + idempotency; HR→finance payroll bridge wired via the bus; double-entry invariant property test green; integration test green on SQLite (always) / PG + MySQL (env-gated); all 4 sub-ports have rows for the school; `cargo test/clippy/fmt/lint --workspace` green; ≥ 6 `coverage.toml` rows flipped; `PHASE-7-HANDOFF.md` + `phase-8-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 7 outcome.".

## When You Are Stuck
`PHASE-6-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. PG RLS setup: `tools/scripts/pg-rls-test-setup.sql`. The 9-file template is `crates/domains/hr/src/`. Mock the payment port; real adapters are Phase 15.
