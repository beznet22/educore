# Educore Phase 12 — CMS

## Mission
Deliver `educore-cms` — `Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, `UploadContent`, `AboutPage`, `ContactPage`, `CoursePage`, `HomePageSetting`, `FrontendPage`. **Spec-faithful**: all 20 root aggregates per `docs/specs/cms/aggregates.md` ship as first-class ports (mirrors Phase 10 OQ #1 decision). **Implementation**, not design.

## Deliverables
`crates/domains/cms/` with 9-file layout per AGENTS.md, 20 root aggregates + 10+ child entities, ~60 typed command shapes, ~60 typed events, 19 repository port traits, 19 query stubs, 6 service factory functions, ~95 net-new `Capability::Cms*` variants (4 Phase 2 `CmsPage*` placeholders retained), 20 net-new `AuditTarget` variants, bus subscriber for `documents.form_download.uploaded` (per Phase 11 handoff OQ #6 — **not in CMS spec**), `educore-academic` dep for `ClassId`/`SectionId`, vertical-slice test mirroring `documents_integration.rs` (6 named scenarios + 2 env-gated). Flip `coverage.toml` rows. Write `PHASE-12-HANDOFF.md` + `phase-13-prompt.md`.

## Required Reading
- `docs/handoff/PHASE-11-HANDOFF.md` (8 OQs carry over; Q1 `AcademicYearId` import path, Q2 `FileStorage` port, Q6 `FormUploaded` bus subscription are the most material for Phase 12)
- `docs/build-plan.md` § "Phase 12" + § "Phase 11 outcome." (note the § Risks clause: RLS must NOT block public reads — use a special `school_id = 0` for public content)
- `docs/specs/cms/` (all 11 files: `overview`, `aggregates`, `entities`, `value-objects`, `events`, `commands`, `services`, `repositories`, `permissions`, `tables`, `workflows`)
- `docs/ports/{event-bus,storage}.md`, `docs/schemas/{tenancy,audit}-schema.md`
- `docs/decisions/ADR-013-CrateLayout.md`, `ADR-015-ExternalCrates.md`
- `crates/cross-cutting/{events,rbac,audit}/src/` (event envelope, capability check, audit writer)
- `crates/domains/documents/src/` (the most recent 9-file template, just shipped in Phase 11; `crates/domains/communication/src/` is the spec-faithful precedent for 20+ aggregates)
- `crates/tools/storage-parity/tests/documents_integration.rs` (the mature 6-scenario pattern)
- `AGENTS.md`, `docs_guidlines/system.md`, `docs_guidlines/execution_guidlines.md`
- `docs/phase_prompt/README.md` (the canonical prompt template + the closing-agent verification checklist)

## Starting Point
19 closed crates (9 cross-cutting + 9 domain + storage-parity) are the foundation. `educore-documents` is the most recent 9-file template. The `educore-cms/Cargo.toml` is scaffold-only (5 deps: `core`, `platform`, `rbac`, `events`, `settings`; `educore-academic` must be added). `crates/educore/src/lib.rs:24` already re-exports `cms`. The 4 Phase 2 `CmsPage{Create,Read,Update,Delete}` placeholders are the start point for `Capability`. The 33 finance placeholder aggregates remain as the Workstreams D-M backlog.

## Working With Subagents
Workstreams: A=`Page` (the headline; 5 events + 6 commands + 10-method repo); B=`News` family (`News` + `NewsCategory` + `NewsComment` + `NewsPage`); C=`NoticeBoard` + `Testimonial` + `HomeSlider` + `SpeechSlider` (public-site aggregates); D=`Content` family (`Content` + `ContentType` + `ContentShareList` + `TeacherUploadContent` + `UploadContent`); E=`AboutPage` + `ContactPage` + `CoursePage` + `HomePageSetting` + `FrontendPage` (per-page templates); F=`documents.form_download.uploaded` bus subscriber + reconcile cross-crate placeholders + integration test + coverage flips + handoff docs.

## Per-Deliverable Gotchas
- Tenth domain crate — stick to 9-file layout per AGENTS.md.
- **`NoticeBoard` is the spec name**, not `Notice`. The build plan uses `Notice`; a follow-up PR should align the build plan. (Per the README closing-agent verification checklist: aggregate names in the prompt MUST match `docs/specs/<domain>/aggregates.md`.)
- The 4 Phase 2 placeholders are `CmsPage{Create,Read,Update,Delete}` (named for `Page` only). Net-new variants follow the `<Domain>.<Aggregate>.<Action>` wire form.
- **RLS must NOT block public reads** — use a special `school_id = 00000000-...` for public content (per build-plan § Phase 12 § Risks).
- `educore-academic` dep required for `ClassId`/`SectionId` references in `Content` / `ContentShareList` aggregates.
- Spec-faithful scope: ≥ 20 `coverage.toml` rows flipped (one per root aggregate), ~95 net-new `Capability` variants, 20 net-new `AuditTarget` variants.
- The `FormUploaded` bus subscriber is events-only (no `educore-documents` dep — same pattern as Phase 10 OQ #5's `AbsentNotificationService`).
- Do NOT add a `educore-finance` dep (Phase 8 OQ #6 carries forward).
- Do NOT add a `educore-notify` dep (Phase 10 OQ #4 + Phase 11 OQ #4 carry forward — port lands in Phase 15).
- Do NOT add a `educore-documents` dep (Phase 11 OQ #6 — bus subscriber only).

## Exit Criteria
20 root aggregates + 9-file layout; 6 service factory functions + 6 service structs; ≥ 20 `coverage.toml` rows flipped; `documents.form_download.uploaded` bus subscriber wired; integration test green on SQLite (always) + PG + MySQL (env-gated); `cargo test/clippy/fmt/lint --workspace` green; `PHASE-12-HANDOFF.md` + `phase-13-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase 12 outcome."`.

## When You Are Stuck
`PHASE-11-HANDOFF.md` is the foundation. `cargo run -p educore-core --bin lint --features lint` is the no-gaps gate. The 9-file template is `crates/domains/documents/src/`. The proptest pattern is `crates/domains/documents/src/services.rs` (the 100-case proptest near the bottom of the file). The closing-agent verification checklist is in `docs/phase_prompt/README.md`.

## Subagent Orchestration
To prevent duplicate work, every phase must enforce: (1) **File-level ownership** — every file in the owned crate is assigned to exactly one subagent; no two subagents open the same file. (2) **Section-level pre-allocation** — for files that must be touched by multiple workstreams (e.g. `aggregate.rs` for 20 root aggregates), the prep subagent pre-creates the file with named section markers (`// === <Aggregate> section begin (owner: <WorkstreamLetter>) ===` / `// === <Aggregate> section end ===`); each workstream subagent's Edit anchors fall strictly inside its assigned range. (3) **Sequential phase gates** — `P0 prep` (single subagent, scaffolds shared files + cross-crate extensions) → `R1 reconcile-prep` (read-only verifier) → `wave 1/2/3` parallel workstreams → `R2 reconcile-impl` → `4-tests` → `5-docs` → `R3 final-validation` (9-command gate). A phase does not start until the prior phase's gate passes. (4) **Atomic commits per microtask** — every subagent produces exactly one commit with a `Phase N: <scope> (<workstream>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer; the orchestrator inspects `git log --stat` to detect any out-of-scope file. (5) **Reconciler subagents are read-only** — they verify section boundaries + duplicate detection but never write code.
