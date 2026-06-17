# Phase 9 Verification Prompt

> Master template for the per-phase verify prompts in
> `docs/verification/`. To create a new per-phase prompt
> (e.g. `18-PHASE-VERIFY-PROMPT.md`), copy this file and
> replace every `9` with the new phase number, then fill in
> the **Per-Phase Preamble** at the bottom.

---

## Mission

Verify that Phase 9's forward-looking prompt
(`docs/phase_prompt/phase-9-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-9-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 9"), and on-disk implementation
(`crates/<tier>/<name>/src/`) are all consistent with the
domain spec (where one exists) and the source-of-truth
priority. Auto-fix any disparities by dispatching subagents
per the 5-layer guarantees.

---

## Source-of-Truth Priority

When the 5 documents above disagree, resolve them in this
order (highest priority first):

1. `docs/specs/<domain>/*.md` — canonical for aggregates,
   commands, events, capabilities, audit targets. **N/A if
   Phase 9 has no domain spec** (adapter or tools tier).
2. `docs/build-plan.md` § "Phase 9" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-9-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in `crates/<tier>/<name>/src/` — the
   on-disk truth. Source files, tests, `Cargo.toml`
   dependencies, and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-9-prompt.md` — the input being
   verified. **LOWEST priority**: a prompt that diverges
   from priorities 1-4 must be corrected to match, not the
   other way around.

---

## Section A: Pre-Implementation Check

> Run BEFORE the phase is implemented. Applies to Phases
> 13-17 (unimplemented at the time this directory was
> created) and any future phase that is not yet closed.
> Skip this section entirely for Phases 0-11 (already
> closed and verified).

For each item, output `Pass` (with a one-line citation) or
`Fail` (with a one-line citation + the fix the auto-fix
subagent will apply).

1. **Spec exists.** `docs/specs/<domain>/` (if applicable)
   contains the 11 standard files (`overview.md`,
   `aggregates.md`, `commands.md`, `entities.md`, `events.md`,
   `permissions.md`, `repositories.md`, `services.md`,
   `tables.md`, `value-objects.md`, `workflows.md`) per
   `AGENTS.md` § "Module Layout (per domain)". For non-domain
   phases, the port contract or reference doc exists in
   `docs/ports/` or `docs/guides/`.
2. **Build-plan § "Phase 9" is complete.** The build-plan
   section for Phase 9 (between the `## Phase 9` and
   `## Phase 10` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-9).
4. **Scaffold crate is in place.** The `Cargo.toml` +
   `src/lib.rs` for the planned crate(s) exist at
   `crates/<tier>/<name>/` and follow the standard 27-line
   scaffold pattern (`PACKAGE_NAME` + `PACKAGE_VERSION` +
   the 9-file module prelude if it is a domain crate). For
   adapter or tools crates, the `Cargo.toml` declares the
   required `infra` + `cross-cutting` deps per
   `AGENTS.md` § "Tier System".

---

## Section B: Post-Implementation Check

> Run AFTER the phase is implemented. Applies to Phases
> 0-11 (closed) and Phase 12 (in progress — run when
> Phase 12 closes). Skip this section for Phases 13-17
> (not yet implemented; covered by Section A).

For each dimension, output `Pass` (with a one-line citation
to the file + line range) or `Fail` (with a one-line
citation + the source-of-truth priority chain that resolves
it + the fix the auto-fix subagent will apply).

1. **Prompt ↔ Spec.** Every aggregate name in
   `docs/phase_prompt/phase-9-prompt.md` matches
   `docs/specs/<domain>/aggregates.md` exactly (per the
   closing-agent verification checklist in
   `docs/phase_prompt/README.md`). For non-domain phases,
   every port trait / reference impl in the prompt matches
   the corresponding `docs/ports/<port>.md` file.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 9" section. The headline-9
   interpretation (or spec-faithful declaration) is
   consistent between the prompt and the build-plan.
3. **Prompt ↔ Handoff.** The "Where NOT to start" rules in
   the prompt match the carry-forward rules in the handoff
   (no `educore-finance` dep, no `educore-notify` dep, no
   `educore-attendance` dep, no `educore-documents` dep, etc.,
   as applicable). The "Do NOT" list in the prompt's
   "Per-Deliverable Gotchas" matches the handoff's "Where
   NOT to start" section word-for-word (modulo phase-specific
   additions).
4. **Handoff ↔ Implementation.** The headline correctness
   check claimed in the handoff (e.g. the 100-case proptest
   in `crates/domains/<name>/src/services.rs`) exists and
   is green. The aggregates, command shapes, events, and
   `Capability` / `AuditTarget` variants listed in the
   handoff's "What's wired and working" section are present
   in the on-disk source files. The "Open questions" in the
   handoff are either resolved in the implementation
   or explicitly carried forward (with a citation in the
   per-phase preamble of the next phase's verify prompt).
5. **Coverage Matrix ↔ Implementation.** Every `Tested`
   row in `docs/coverage.toml` for Phase 9 has a real
   implementation in the source tree (not a stub returning
   `Err(not_supported)`). Every `Pending` row has either
   a real implementation that should be flipped to `Tested`
   (then flip it as part of the auto-fix) or an explicit
   "deferred to Phase 9+M" rationale in the handoff.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-9-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-9-prompt.md`, `docs/build-plan.md` § "Phase 9" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-9-prompt.md`, `docs/handoff/PHASE-9-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/<tier>/<name>/src/**`, `crates/cross-cutting/{rbac,audit}/src/**` (if prereq 2A/2B is missing) |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/<tier>/<name>/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-9-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 9 verify: <dimension> (<workstream>)` with
the standard
`Co-Authored-By: Antigravity <antigravity@google.com>` trailer.

---

## Subagent Orchestration (5-Layer Guarantees)

To prevent two or more subagents from being given the same
work, every verify prompt must enforce the following 5-layer
guarantees. These are the same rules that closed Phases
8-11 successfully (the first phase to break these rules
will produce a duplicate-work collision and a non-mergeable
state):

1. **File-level ownership.** Every file in the owned crate
   is assigned to exactly one subagent. No two subagents
   open the same file. The orchestrator maintains a
   file-ownership map in the phase plan and embeds the list
   of forbidden files in every parallel-subagent prompt.
2. **Section-level pre-allocation.** For files that must be
   touched by multiple workstreams (e.g. `aggregate.rs` for
   3+ root aggregates, or `phase-9-prompt.md` for two
   failing dimensions), the prep subagent pre-creates the
   file with named section markers
   (`// === <Aggregate> section begin (owner: <WorkstreamLetter>) ===`
   / `// === <Aggregate> section end ===` for code; or
   `<!-- === <Section> section begin (owner: <WorkstreamLetter>) === -->`
   / `<!-- === <Section> section end === -->` for markdown).
   Each workstream subagent's `Edit` anchors fall strictly
   inside its assigned range. A subagent that crosses a
   marker aborts and reports to the orchestrator.
3. **Sequential phase gates.** The verify step advances
   through fixed stages: `P0 prep` (single subagent,
   scaffolds shared files + cross-crate extensions) →
   `R1 reconcile-prep` (read-only verifier) → `wave 1/2/3`
   parallel fix-subagents → `R2 reconcile-impl` →
   `4-tests` (`cargo test --workspace`) → `5-docs`
   (`docs/handoff/PHASE-9-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 9 verify: <scope> (<workstream>)` message +
   `Co-Authored-By: Antigravity <antigravity@google.com>`
   trailer. The orchestrator inspects `git log --stat` after
   every stage to detect any out-of-scope file. A "do not
   run cargo test" rule applies to the parallel fix wave —
   the orchestrator runs the gate at stage 4, not the
   subagents.
5. **Reconciler subagents are read-only.** `R1`, `R2`, `R3`
   are dedicated reconciler subagents. They verify section
   boundaries + duplicate detection + stub-replacement but
   never write code. A reconciler that finds a violation
   halts the verify step.

---

## Output Format

Write `docs/handoff/PHASE-9-VERIFY-REPORT.md` with these
five sections:

- **Section A — Pre-Implementation Check results.**
  `Pass` / `Fail` per item, with a one-line citation to the
  file + line range.
- **Section B — Post-Implementation Check results.**
  `Pass` / `Fail` per dimension, with a one-line citation.
- **Section C — Disparities Summary.** Bullet list of every
  item that `Failed` in Section A or Section B, with the
  specific file + line + the source-of-truth priority chain
  that resolves it (e.g. "Spec line 42 says `NoticeBoard`,
  build-plan line 1435 says `Notice` — Spec wins (priority 1),
  fix the build-plan").
- **Section D — Fix Plan.** Ordered list of files to update
  (or "no fixes needed" if both sections pass). Each fix
  item names the file, the change, and the subagent scope
  per the 5-layer guarantees.
- **Section E — GO/NO-GO verdict.** `GO` if all checks pass
  or all disparities are fixed in the same atomic commit;
  `NO-GO` if any fix is deferred or any check is open.

The verify report itself is one atomic commit; the fixes
it triggers are a second atomic commit per the
"Atomic commits per microtask" guarantee.

---

## Done Criteria

The verify step is `Done` when ALL of the following hold:

- [ ] `docs/handoff/PHASE-9-VERIFY-REPORT.md` exists with all
  5 sections (A, B, C, D, E) populated.
- [ ] All Section A items pass (for unimplemented phases
  13-17) or all Section B dimensions pass (for implemented
  phases 0-11 and 12).
- [ ] All listed disparities fixed (or explicitly deferred
  with a rationale + an ADR reference in Section C).
- [ ] One atomic commit with the fixes (per the
  "Atomic commits per microtask" guarantee).
- [ ] `cargo test -p <crate>` green (for implemented phases;
  the crate name comes from the per-phase preamble).
- [ ] `cargo build --workspace` green.
- [ ] `cargo run -p educore-core --bin lint --features lint`
  green (the no-gaps gate per `AGENTS.md`).
- [ ] `docs/progress-tracker.md` row for Phase 9 updated
  (status reflects the verified close).

---

## Per-Phase Preamble

> Copy this section to the bottom of the per-phase prompt
> and fill in the bracketed fields. For Phases 14-17 (the
> unimplemented phases at the time this directory was
> created), use `PRE-CHECK-PHASES-13-17.md` as the source
> for the spec path, scaffold line counts, coverage row
> undercount, and carry-forward rules.

**Phase 9 — <Title>**

- **Spec:** `<path to docs/specs/<domain>/>` (or "no spec"
  for adapter / tools tier phases; reference the port
  contract or guide doc instead).
- **Handoff:** `docs/handoff/PHASE-9-HANDOFF.md`.
- **Build-plan section:** `docs/build-plan.md` lines
  <start>-<end> (between the `## Phase 9` and `## Phase 10`
  headings).
- **Implementation crate(s):** `crates/<tier>/<name>/`
  (one entry per crate the phase ships).
- **Coverage row IDs:** <list the `Pending` / `Tested` row
  IDs from `docs/coverage.toml` for this phase, with the
  spec-faithful target count if Section A applies>.
- **Carry-forward rules:** <paste the "Where NOT to start"
  + "Open questions" from the prior handoff, plus the
  phase-specific rules from the build-plan § "Phase 9"
  "Risks." sub-section>.

---

# Per-Phase Preamble — Phase 9 (Library)

**Phase title:** Library

**Status:** Implemented (per `docs/handoff/PHASE-9-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 1026–1055

**Spec:** `docs/specs/library/` (11 files)

**Spec aggregate count:** 6 root aggregates per `docs/specs/library/aggregates.md` (Book, BookCategory, LibraryMember, BookIssue, BookReturn, Fine)

**Handoff:** `docs/handoff/PHASE-9-HANDOFF.md`

**Implementation crate:** `crates/domains/library/` (`educore-library`)

**Coverage rows in `docs/coverage.toml` for `phase = 9` or `crate = "educore-library"`:**
- `library_books_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#book`)
- `library_book_categories_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#bookcategory`)
- `library_members_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#librarymember`)
- `library_book_issues_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#bookissue`)
- `library_book_returns_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#bookreturn`)
- `library_fines_aggregate` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#fine`)
- `library_book_issued_event` — `status = "Tested"` (spec: `docs/specs/library/events.md#bookissued`)
- `library_book_returned_event` — `status = "Tested"` (spec: `docs/specs/library/events.md#bookreturned`)
- `library_fine_calculated_event` — `status = "Tested"` (spec: `docs/specs/library/events.md#finecalculated`)
- `library_capability_variants` — `status = "Tested"` (spec: `docs/specs/library/permissions.md`)
- `library_audit_target_variants` — `status = "Tested"` (spec: `docs/specs/library/aggregates.md#audittargets`)
- `library_fine_calculation_service` — `status = "Tested"` (spec: `docs/specs/library/services.md#finecalculationservice`)

(12 rows total for `phase = 9` OR `crate = "educore-library"`, all `Tested`. The handoff counts 10 rows flipped for `crate = "educore-library"`; the 2 additional rows are the `educore-rbac` and `educore-audit` cross-cutting rows that carry `phase = 9`.)

**Known carry-forward rules relevant to this phase:**
- **Phase 8 OQ #6: "no `educore-finance` dep"** — Phase 9 is a consumer of this rule (per Subagent 3's report, Phase 9's handoff's "Where NOT to start" includes this rule).
- Per Subagent 3's report: "Phase 9 handoff: 'Do NOT add `educore-finance` as a dep (Phase 8 OQ #6 carries forward — no cross-domain deps; the bus handles the `FineCalculated` → `Receivable` cross-domain coordination).'"
- 26 `Library.*` `Capability` variants in `educore-rbac` (4 Phase 2 `LibraryBook{Create,Read,Update,Delete}` placeholders deduplicated; the canonical `Book{Add,Read,Update,Delete,AdjustQuantity,Search}` variants use the same wire forms as the Phase 2 placeholders per the handoff § "educore-rbac integration").
- 5 net-new `Library` `AuditTarget` variants in `educore-audit` (1 Phase 2 `Book(Uuid)` placeholder retained + 5 net-new: `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`, `Fine` per the handoff § "educore-audit integration").

**Pre-implementation gaps found in PRE-CHECK-PHASES-13-17.md (if any for this phase):** N/A. Phase 9 is implemented (closed 2026-06-15 per `docs/build-plan.md` line 1091); the pre-check snapshot is scoped to Phases 13–17.

**Known secondary-doc gaps (from the Subagent 2 survey of Phase 11 close-out):**
- `docs/progress-tracker.md` row 78 says Phase 9 Library is "Planned" with "No" exit criteria — **stale** (the handoff says it's done, the build-plan § "Phase 9 outcome." at line 1091 confirms the close, and 10 coverage rows are flipped to `Tested`).

**Specific verification focus:**
- The 6 root aggregates vs the 18 events (per handoff) vs the 18 commands
- The `Book` aggregate (the largest; 13-digit ISBN checksum validation per the spec)
- The `BookIssue` aggregate (the 4-state machine: Available → Issued → Renewed* → Returned/Overdue)
- The `BookReturn` aggregate
- The `Fine` aggregate (the per-day late-fee calculation pipeline)
- The `FineCalculationService` (the 100-case proptest per handoff)
- The 31 unit tests
- The 4-scenario integration test
- The 10 coverage rows flipped
- The `educore-library` `Cargo.toml` deps: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-events-domain`
- The 5 commits per handoff
