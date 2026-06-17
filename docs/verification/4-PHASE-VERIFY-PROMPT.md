# Phase 4 Verification Prompt

> Per-phase verify prompt for Phase 4 (Assessment), rendered
> from `docs/verification/TEMPLATE.md` with the per-phase
> preamble filled in.

---

## Mission

Verify that Phase 4's forward-looking prompt
(`docs/phase_prompt/phase-4-prompt.md`), retrospective handoff
(`docs/handoff/PHASE-4-HANDOFF.md`), build-plan section
(`docs/build-plan.md` § "Phase 4"), and on-disk implementation
(`crates/domains/assessment/src/`) are all consistent with the
domain spec (where one exists) and the source-of-truth
priority. Auto-fix any disparities by dispatching subagents
per the 5-layer guarantees.

---

## Source-of-Truth Priority

When the 5 documents above disagree, resolve them in this
order (highest priority first):

1. `docs/specs/assessment/*.md` — canonical for aggregates,
   commands, events, capabilities, audit targets. **N/A if
   Phase 4 has no domain spec** (adapter or tools tier).
2. `docs/build-plan.md` § "Phase 4" — canonical for what the
   phase builds (deliverables, tasks, exit criteria, risks).
3. `docs/handoff/PHASE-4-HANDOFF.md` — the closing agent's
   claim about what was actually shipped (validated against
   the on-disk implementation in priority 4).
4. The implementation in `crates/domains/assessment/src/` — the
   on-disk truth. Source files, tests, `Cargo.toml`
   dependencies, and the umbrella re-exports in
   `crates/educore/src/lib.rs` are the source of truth for
   "what was actually built".
5. `docs/phase_prompt/phase-4-prompt.md` — the input being
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

1. **Spec exists.** `docs/specs/assessment/` (if applicable)
   contains the 11 standard files (`overview.md`,
   `aggregates.md`, `commands.md`, `entities.md`, `events.md`,
   `permissions.md`, `repositories.md`, `services.md`,
   `tables.md`, `value-objects.md`, `workflows.md`) per
   `AGENTS.md` § "Module Layout (per domain)". For non-domain
   phases, the port contract or reference doc exists in
   `docs/ports/` or `docs/guides/`.
2. **Build-plan § "Phase 4" is complete.** The build-plan
   section for Phase 4 (between the `## Phase 4` and
   `## Phase 5` headings) contains all 5 sub-sections:
   `Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`,
   and `Phase completion documentation.` (per the per-phase
   prompt convention in `docs/phase_prompt/README.md`).
3. **Coverage rows are `Pending`.** Every aggregate or
   feature that the phase plans to ship has a row in
   `docs/coverage.toml` with `status = "Pending"`. The
   `PRE-CHECK-PHASES-13-17.md` snapshot enumerates the
   current `Pending` count; the per-phase preamble lists
   the expected count (spec-faithful vs headline-4).
4. **Scaffold crate is in place.** The `Cargo.toml` +
   `src/lib.rs` for the planned crate(s) exist at
   `crates/domains/assessment/` and follow the standard 27-line
   scaffold pattern (`PACKAGE_NAME` + `PACKAGE_VERSION` +
   the 9-file module prelude if it is a domain crate). For
   adapter or tools crates, the `Cargo.toml` declares the
   required `infra` + `cross-cutting` deps per
   `AGENTS.md` § "Tier System".

---

## Section B: Post-Implementation Check

> Run AFTER the phase is implemented. Applies to Phases
> 0-11 (closed) and Phase 12 (in progress — run when Phase
> 12 closes). Skip this section for Phases 13-17 (not yet
> implemented; covered by Section A).

For each dimension, output `Pass` (with a one-line citation
to the file + line range) or `Fail` (with a one-line
citation + the source-of-truth priority chain that resolves
it + the fix the auto-fix subagent will apply).

1. **Prompt ↔ Spec.** Every aggregate name in
   `docs/phase_prompt/phase-4-prompt.md` matches
   `docs/specs/assessment/aggregates.md` exactly (per the
   closing-agent verification checklist in
   `docs/phase_prompt/README.md`). For non-domain phases,
   every port trait / reference impl in the prompt matches
   the corresponding `docs/ports/<port>.md` file.
2. **Prompt ↔ Build-Plan.** The "Deliverables" + "Tasks" +
   "Exit Criteria" sections in the prompt match the
   build-plan § "Phase 4" section. The headline-4
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
   check claimed in the handoff (e.g. the vertical-slice
   test in `crates/tools/storage-parity/tests/assessment_integration.rs`)
   exists and is green. The aggregates, command shapes,
   events, and `Capability` / `AuditTarget` variants listed
   in the handoff's "What's wired and working" section are
   present in the on-disk source files. The "Open
   questions" in the handoff are either resolved in the
   implementation or explicitly carried forward (with a
   citation in the per-phase preamble of the next phase's
   verify prompt).
5. **Coverage Matrix ↔ Implementation.** Every `Tested`
   row in `docs/coverage.toml` for Phase 4 has a real
   implementation in the source tree (not a stub returning
   `Err(not_supported)`). Every `Pending` row has either
   a real implementation that should be flipped to `Tested`
   (then flip it as part of the auto-fix) or an explicit
   "deferred to Phase 4+M" rationale in the handoff.

---

## Auto-Fix Rules (per dimension)

The verify agent dispatches one subagent per failing
dimension, with file-level ownership and section-level
pre-allocation per the 5-layer guarantees. The
subagent-scope mapping is:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `docs/phase_prompt/phase-4-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `docs/phase_prompt/phase-4-prompt.md`, `docs/build-plan.md` § "Phase 4" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `docs/phase_prompt/phase-4-prompt.md`, `docs/handoff/PHASE-4-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `crates/domains/assessment/src/**`, `crates/cross-cutting/{rbac,audit}/src/**` (if prereq 1/2 is missing) |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `docs/coverage.toml`, `crates/domains/assessment/src/**` (only for stub-flips) |

Multiple dimensions can run in parallel if they own disjoint
files. If two dimensions want to edit the same file (e.g.
dimension 1 + dimension 3 both touch
`docs/phase_prompt/phase-4-prompt.md`), the prep subagent
pre-allocates the file with section markers per the
5-layer guarantees; each subagent's edits stay inside its
assigned section.

The auto-fix subagent produces exactly one atomic commit
per the "Atomic commits per microtask" guarantee. The commit
message is `Phase 4 verify: <dimension> (<workstream>)` with
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
2. **Section-level pre-allocation.** For files that must
   be touched by multiple workstreams (e.g. `aggregate.rs`
   for 3+ root aggregates, or `phase-4-prompt.md` for two
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
   (`docs/handoff/PHASE-4-VERIFY-REPORT.md`) → `R3
   final-validation` (9-command gate). A verify step does
   not advance to the next stage until the prior stage's
   gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a
   `Phase 4 verify: <scope> (<workstream>)` message +
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

Write `docs/handoff/PHASE-4-VERIFY-REPORT.md` with these
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

- [ ] `docs/handoff/PHASE-4-VERIFY-REPORT.md` exists with all
  5 sections (A, B, C, D, E) populated.
- [ ] All Section A items pass (for unimplemented phases
  13-17) or all Section B dimensions pass (for implemented
  phases 0-11 and 12).
- [ ] All listed disparities fixed (or explicitly deferred
  with a rationale + an ADR reference in Section C).
- [ ] One atomic commit with the fixes (per the
  "Atomic commits per microtask" guarantee).
- [ ] `cargo test -p educore-assessment` green (for implemented phases;
  the crate name comes from the per-phase preamble).
- [ ] `cargo build --workspace` green.
- [ ] `cargo run -p educore-core --bin lint --features lint`
  green (the no-gaps gate per `AGENTS.md`).
- [ ] `docs/progress-tracker.md` row for Phase 4 updated
  (status reflects the verified close).

---

# Per-Phase Preamble — Phase 4 (Assessment)

**Phase title:** Assessment

**Status:** Implemented (per `docs/handoff/PHASE-4-HANDOFF.md`)

**Build-plan section:** `docs/build-plan.md` lines 622–707

**Spec:** `docs/specs/assessment/` (11 files: overview, aggregates, entities, value-objects, events, commands, services, repositories, permissions, tables, workflows)

**Spec aggregate count:** 8 root aggregates per `docs/specs/assessment/aggregates.md` (Exam, ExamSchedule, MarksRegister, ResultStore, ReportCard, OnlineExam, SeatPlan, AdmitCard) — headline subset of the 45-entry spec

**Handoff:** `docs/handoff/PHASE-4-HANDOFF.md`

**Implementation crate:** `crates/domains/assessment/` (`educore-assessment`)

**Coverage rows in `docs/coverage.toml` for `phase = 4` or `crate = "educore-assessment"`:**
- `assessment_exams_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_marks_registers_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_exam_schedules_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_result_stores_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_report_cards_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_online_exams_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_seat_plans_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)
- `assessment_admit_cards_aggregate` — `status = "Tested"` (spec: `docs/specs/assessment/aggregates.md`)

**Known carry-forward rules relevant to this phase:**
- `educore-assessment` depends on `educore-academic` (Phase 3) for `ClassId`/`SectionId`/`SubjectId` references (per AGENTS.md § "Tier System" — domain crates may depend on infra + cross-cutting; cross-domain deps require an ADR).
- The handoff's "What's wired and working" should match the implementation (no known OQs to carry forward for Phase 4).

**Pre-implementation gaps found in PRE-CHECK-PHASES-13-17.md (if any for this phase):** N/A (Phase 4 is already implemented; the PRE-CHECK file covers Phases 13–17 only).

**Specific verification focus:**
- The 8 root aggregates vs the 28 commands vs the 28 events (per handoff)
- The 8 coverage Tested rows (must match the 8 aggregates)
- The `Exam` aggregate's 17-field audit footer
- The `ExamField` enum (macro-emitted)
- The `ExamRepository` trait: 7-10 methods
- The `MarksRegister` aggregate (the largest of the 8; per handoff)
- The `ResultStore` aggregate (compute pipeline for grade calculation)
- The `educore-assessment` `Cargo.toml` deps: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-events-domain`, `educore-academic` (cross-domain dep)
