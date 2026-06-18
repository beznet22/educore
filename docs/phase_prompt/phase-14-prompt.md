# Phase 14 Prompt — Settings + Operations

**Mission:** Deliver `educore-settings` + `educore-operations`. **Implementation**, not design. **Spec-faithful** interpretation per `docs/specs/settings/` and `docs/specs/operations/`.

**Deliverables:** `crates/cross-cutting/settings/` + `crates/cross-cutting/operations/` with 9-file layout per `AGENTS.md`, root aggregates per the two specs, ~typed events, ~typed commands, repository port traits, query stubs, 2-3 service factory functions + service structs, net-new `Capability` variants, net-new `AuditTarget` variants, `educore-rbac` round-trip test + `educore-audit` round-trip test, RRULE/whatever subset implementation if needed, vertical-slice test mirroring `events_integration.rs` (5 named scenarios + 2 env-gated). Flip `coverage.toml` rows. Write `PHASE-14-HANDOFF.md` + `phase-15-prompt.md`.

**Required Reading:**
- `docs/handoff/PHASE-13-HANDOFF.md` (carry-over OQs)
- `docs/build-plan.md` § "Phase 14" + § "Phase 13 outcome."
- `docs/specs/settings/` (all 11 files)
- `docs/specs/operations/` (all 11 files)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events-domain/src/` (the most recent 9-file template)
- `crates/tools/storage-parity/tests/events_integration.rs` (the mature 7-scenario pattern)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (the canonical prompt template)

**Starting Point:** 21 closed crates (10 cross-cutting + 10 domain + storage-parity) are the foundation. `educore-events-domain` is the most recent 9-file template. The `educore-settings` crate is already a dep of `educore-events-domain`.

**Working With Subagents:** Workstreams: A=`educore-settings` aggregates; B=`educore-operations` aggregates; C=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

**Per-Deliverable Gotchas:**
- Two new crates in cross-cutting tier. Stick to the 9-file layout per `AGENTS.md`.
- Do NOT add `educore-finance` dep (Phase 8 OQ #6 carry-over).
- Do NOT add `educore-notify` dep (Phase 10 OQ #4 — port lands in Phase 15).
- Do NOT add `educore-attendance` dep (Phase 10 OQ #5).
- Do NOT add `educore-documents` dep (Phase 11 OQ #6).
- `educore-academic` dep MAY be needed for `educore-settings` (per Phase 13 OQ #7 follow-up).

**Exit Criteria:** 2 crates shipped; 2-3 service factory functions + service structs per crate; ≥ 4 `coverage.toml` rows flipped; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; `PHASE-14-HANDOFF.md` + `phase-15-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase 14 outcome."`.

**When You Are Stuck:** `PHASE-13-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/cross-cutting/events-domain/src/`. The proptest pattern is `crates/cross-cutting/events-domain/src/services.rs`. The closing-agent verification checklist is in `docs/phase_prompt/README.md`.
