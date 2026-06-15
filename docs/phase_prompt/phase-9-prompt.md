# Phase 9 Prompt — Library

**Mission:** Deliver `educore-library`. Book, book category, library member, book issue, book return, fine. **Implementation**, not design.

**Deliverables:** `crates/domains/library/` with 9-file layout, 6 headline aggregates (`Book`, `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine`) + 3 child entities, ~30 typed command shapes, ~15 typed events, 6 repository ports, 6 query stubs, 6 service factory functions, capability additions (`Library.{Create,Read,Update,Delete}` already exist; extend if needed for the fine + member-issue flows), audit additions. Vertical-slice test mirroring `facilities_integration.rs`. Reconcile any library cross-crate placeholders (none currently). Flip `coverage.toml` rows. Write `PHASE-9-HANDOFF.md` + `phase-10-prompt.md`.

**Required Reading:**
- `docs/handoff/PHASE-8-HANDOFF.md` (6 OQs carry over; Q2 the 33-stub backlog and Q6 the no-finance-dep decision are the most material for Phase 9)
- `docs/build-plan.md` § "Phase 9" + § "Phase 8 outcome."
- `docs/specs/library/` (all 11 files: overview, aggregates, entities, value-objects, events, commands, services, repositories, permissions, tables, workflows)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`, `crates/cross-cutting/rbac/src/services.rs`, `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/finance/src/` (9-file template) + `crates/domains/{hr,facilities,attendance,assessment,academic}/src/`
- `crates/tools/storage-parity/tests/{finance,facilities,hr,attendance,assessment,academic}_integration.rs`
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

**Starting Point:** 15 closed crates (7 cross-cutting + 6 domain + storage-parity + settings) are the foundation. `educore-facilities` is the most recent template (9-file layout, 18 events, 13 repos, 13 query stubs, 100-case proptest). The headline 11 aggregates + 6 child entities + 18 events + 13 service factories all shipped. The 6 new `Library.*` `Capability` placeholders from Phase 2 (`LibraryBookCreate/Read/Update/Delete`) are the start point.

**Workstreams:** A = `Book` + `BookCategory`; B = `LibraryMember`; C = `BookIssue` + `BookReturn`; D = `Fine` + the late-return fine proptest; E = reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- Seventh domain crate — stick to 9-file layout.
- Late-fine computation: `fine = days_late * daily_rate` per `LateFineKind` (fixed/per-day/percentage). Property test required (mirrors Phase 7's `LateFeeService`).
- `BookIssue` decrements `Book.AvailableCopies` atomically. The `BookReturn` increments. The same row-level lock strategy as Phase 8 inventory.
- The 6 `Library.*` capability placeholders from Phase 2 may need to be extended for fine payment + member suspension flows.

**Exit Criteria:** 6 headline aggregates + 9-file layout; `create_book_issue` + `return_book` + `compute_fine` service functions; fine proptest green (100 cases); integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; ≥ 6 `coverage.toml` rows flipped; `PHASE-9-HANDOFF.md` + `phase-10-prompt.md` written.

**When You Are Stuck:** `PHASE-8-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/finance/src/`. The proptest pattern is `crates/domains/finance/src/services.rs:1259` (100 cases).
