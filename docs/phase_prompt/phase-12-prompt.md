# Educore Phase 12 — CMS

## Mission
Deliver `educore-cms` — page, news, notice (distinct from `educore-communication`'s `Notice`), testimonial. **Implementation**, not design. 1 root aggregate (`Page`) anchors the public-site surface; subscribes to `FormUploaded` for public-site form publication.

## Deliverables
`crates/domains/cms/` with 9-file layout, 4 root aggregates (`Page`, `News`, `Notice` (CMS-flavoured, distinct from `educore-communication::Notice`), `Testimonial`) + child entities, ~12 typed command shapes, ~12 typed events, 4 repository ports, 4 query stubs, 4 service factory functions, capability additions (the new `Cms.*` group + the 4 Phase 2 placeholders for dedup), audit additions. Bus subscriber for `documents.FormUploaded` (reads `show_public`, indexes on public site). Vertical-slice test mirroring `documents_integration.rs`. Flip `coverage.toml` rows. Write `PHASE-12-HANDOFF.md` + `phase-13-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-11-HANDOFF.md` (8 OQs carry over; Q1 the `AcademicYearId` import path, Q2 the `FileStorage` port, Q6 the `FormUploaded` bus subscription are the most material for Phase 12)
- `docs/build-plan.md` § "Phase 12" + § "Phase 11 outcome."
- `docs/specs/cms/` (all 11 files)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/events/src/{lib,domain_event,envelope}.rs`, `crates/cross-cutting/rbac/src/services.rs`, `crates/cross-cutting/audit/src/writer.rs`
- `crates/domains/documents/src/` (the most recent 9-file template)
- `crates/domains/communication/src/` (the 26-aggregate spec-faithful template)
- `crates/tools/storage-parity/tests/documents_integration.rs` (the vertical-slice template)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`

## Starting Point
18 closed crates (8 cross-cutting + 8 domain + storage-parity + settings) are the foundation. `educore-documents` is the most recent 9-file template. The 4 Phase 2 `Cms{Create,Read,Update,Delete}` placeholders are the start point. The 33 finance placeholder aggregates remain as the Workstreams D-M backlog.

## Working With Subagents
Workstreams: A=`Page`; B=`News`; C=`Notice` (CMS-flavoured); D=`Testimonial` + the `FormUploaded` bus subscriber; E=reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

## Per-Deliverable Gotchas
- Ninth domain crate — stick to 9-file layout.
- `Page` is the public-site aggregate; RLS must NOT block public reads (use a special `school_id` for public content per the build plan).
- `Notice` in CMS is distinct from `educore-communication::Notice` — the CMS `Notice` is a public-site notice, the communication one is a school-wide internal notice. No `Capability` name collision.
- `News` and `Testimonial` are public-site aggregates; the `published_at` + `unpublished_at` lifecycle is the state machine.
- The 4 Phase 2 `Cms{Create,Read,Update,Delete}` placeholders will likely dedup against the new `Page*` / `News*` / `Notice*` / `Testimonial*` variants. Use the same wire-form dedup pattern as Phase 11.
- The `FormUploaded` bus subscriber is events-only (no `educore-documents` dep — same pattern as Phase 10 OQ #5's `AbsentNotificationService`).
- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 carries forward).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 + Phase 11 OQ #4 carry forward — port lands in Phase 15).

## Exit Criteria
4 root aggregates + 9-file layout; 4 service factory functions; the `CmsEvent` event family; capability-gate every read command via `Capability::CmsRead`; the `FormUploaded` bus subscriber wired; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; ≥ 4 `coverage.toml` rows flipped; `PHASE-12-HANDOFF.md` + `phase-13-prompt.md` + `progress-tracker.md` + `build-plan.md` § "Phase 12 outcome.".

## When You Are Stuck
`PHASE-11-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/documents/src/`. The proptest pattern is `crates/domains/documents/src/services.rs` (the 100-case proptest near the bottom of the file).
