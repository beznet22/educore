# Verification Directory Builder — Generic Meta-Prompt

> A self-contained, copy-pasteable prompt for an AI to build a
> `docs/verification/` directory for **any Rust project** that
> uses a phase-per-build-plan convention. Run once per project;
> produces permanent verification prompts that future closing
> agents use to verify implementation, docs, and source-of-truth
> consistency for each phase.

---

## Section 1 — Mission

Build a `docs/verification/` directory of permanent verify prompts for each build-plan phase of a Rust project, plus a reconcile report. Four stages: pre-check, README + master TEMPLATE, per-phase verify prompts, reconcile. The directory is **docs-only** — it never touches `<IMPL_ROOT>/`, `Cargo.toml`, or any source file. Future closing agents run the per-phase prompts to verify implementation, docs, and source-of-truth consistency for each phase.

---

## Section 2 — Inputs

The receiving AI will ask for these overrides before dispatching sub-agents.

| Variable | Default | Description |
| --- | --- | --- |
| `PROJECT_NAME` | `<project name>` | Human-readable project name |
| `BUILD_PLAN_PATH` | `docs/build-plan.md` | Build-plan with `## Phase N — <Title>` headings |
| `N_PHASES` | `<auto-detect>` | Count `## Phase N` headings at runtime |
| `SPEC_DIR` | `docs/specs/` | Per-component specs; `null` if none |
| `HANDOFF_DIR` | `docs/handoff/` | Per-phase handoffs; `null` if not used |
| `COVERAGE_PATH` | `<none>` | Coverage matrix; `null` if none |
| `IMPL_ROOT` | `crates/` | Implementation root (e.g. `crates/`, `src/`) |
| `VERIFICATION_DIR` | `docs/verification/` | Output directory |
| `CO_AUTHOR_TRAILER` | `Co-Authored-By: <Name> <email>` | Commit trailer for every commit |

`null` means "the project does not have this artifact; skip all references to it." `<auto-detect>` means "count `## Phase N` headings in `<BUILD_PLAN_PATH>` at runtime."

---

## Section 3 — Output structure

Exactly this layout (file count = `1 + 1 + 1 + N_PHASES + 1`):

```text
<VERIFICATION_DIR>/
├── README.md                                 — user guide
├── TEMPLATE.md                               — master two-section template
├── PRE-CHECK-PHASES-[N1]-[N2].md            — pre-check for unimplemented phases
├── 0-PHASE-VERIFY-PROMPT.md                  — per-phase (N = 0..N_PHASES-1)
├── 1-PHASE-VERIFY-PROMPT.md
├── ...
├── [N_PHASES-1]-PHASE-VERIFY-PROMPT.md
└── RECONCILE-REPORT.md                       — consistency check
```

`[N1]`..`[N2]` = smallest and largest unimplemented phase numbers; file is omitted if every phase is implemented.

---

## Section 4 — Step 1 — Pre-check the build-plan

Dispatch one sub-agent (**P0-prep**) to gather ground truth before any verify prompt is written.

P0-prep is **read-only** with respect to source code; it only inspects the build-plan, handoffs, coverage matrix, and `Cargo.toml` files to characterize each phase. The output is a single consolidated pre-check file covering all unimplemented phases.

P0-prep:

1. **Scan `<BUILD_PLAN_PATH>`.** For each `## Phase N` (N = 0..N_PHASES-1), capture: phase number/title, build-plan line range, status (`implemented` / `in progress` / `unimplemented`), spec path or `"no spec"`, handoff path or `"DOES NOT EXIST YET"`, implementation path (left blank — per-phase sub-agents in Step 3 discover it at runtime by listing files in `<IMPL_ROOT>/`), coverage rows for phase N (if `COVERAGE_PATH` is set).
2. **Write the pre-check file** `<VERIFICATION_DIR>/PRE-CHECK-PHASES-[N1]-[N2].md` for all unimplemented phases, with one `## Phase N — <Title>` section per phase: build-plan line range, spec path or "no spec", coverage rows, scaffold components (list dirs under `<IMPL_ROOT>/` whose `Cargo.toml` mentions the phase's components + current `lib.rs` line count + scaffold-only vs real impl), pre-implementation gaps (undercount of coverage rows, missing `Cargo.toml` deps, forward-references to later phases, headline-vs-spec drift), carry-forward rules from prior handoffs.
3. **Commit** as `<prefix>(verification): pre-check Phases [N1]-[N2] spec/build-plan/coverage` with the `CO_AUTHOR_TRAILER`.

`<prefix>` is the project's conventional commit scope (e.g. `docs`, `chore`); use the same prefix for every commit in this build.

---

## Section 5 — Step 2 — README + TEMPLATE

Dispatch one sub-agent (**V0**) to author the two foundation files. V0 runs no verification.

V0:

1. **`<VERIFICATION_DIR>/README.md`** — user guide covering: mission statement naming `<PROJECT_NAME>`; directory layout (Section 3); "When to invoke" closing-agent workflow (commit handoff + phase prompt + coverage flips + build-plan outcome → commit the per-phase verify prompt → run it as the instruction for a fresh verification agent → the fresh agent writes `<HANDOFF_DIR>/PHASE-N-VERIFY-REPORT.md` and commits); two-section prompt semantics (Section 11); how to run a prompt; expected 5-section output; 5-layer guarantees (Section 10); recipe for adding a new per-phase prompt (copy `TEMPLATE.md`, replace `N`, fill the per-phase preamble, commit); see-also links.
2. **`<VERIFICATION_DIR>/TEMPLATE.md`** — the canonical two-section template. Embed the exact code block in Section 8 verbatim, with `<PROJECT_NAME>`, `<BUILD_PLAN_PATH>`, `<HANDOFF_DIR>`, `<SPEC_DIR>`, `<COVERAGE_PATH>`, `<IMPL_ROOT>` substituted. Per-phase sub-agents instantiate it with `N` substituted.
3. **Commit** both files atomically: `<prefix>(verification): add README + master TEMPLATE` with the `CO_AUTHOR_TRAILER`.

---

## Section 6 — Step 3 — Per-phase verify prompts

For each phase N in `0..N_PHASES-1`, dispatch one sub-agent (**VN**) in parallel.

File-ownership rule guarantees no collisions. The receiving AI must run these sub-agents concurrently to maximize wall-clock throughput; the per-phase sub-agents cannot collide because each owns a unique file.

Each VN:

1. Reads `<VERIFICATION_DIR>/TEMPLATE.md`.
2. Reads `<BUILD_PLAN_PATH>` § "Phase N".
3. Reads `<HANDOFF_DIR>/PHASE-N-HANDOFF.md` if `HANDOFF_DIR` is set and the file exists; otherwise consults `PRE-CHECK-PHASES-[N1]-[N2].md`.
4. Reads every file in `<SPEC_DIR>/<component>/` if `SPEC_DIR` is set and a per-component spec exists for Phase N.
5. Reads `<COVERAGE_PATH>` rows for phase N (if set).
6. **Discovers the implementation at runtime** by listing files in `<IMPL_ROOT>/` — does NOT assume a fixed layout; records the on-disk path verbatim.
7. Writes `<VERIFICATION_DIR>/[N]-PHASE-VERIFY-PROMPT.md` containing the full TEMPLATE body (every literal `N` replaced) plus a "Per-Phase Preamble" section at the bottom (template in Section 9).
8. **Constraints:** owns only its per-phase file; does not run `cargo build` or `cargo test` (orchestrator runs the gate); does not modify build-plan, handoff, spec, coverage matrix, or any source file. One atomic commit: `<prefix>(verification): add Phase N verify prompt` with the `CO_AUTHOR_TRAILER`.

If a phase has no handoff and no scaffold (truly unimplemented and not in the pre-check range), the preamble explicitly notes Section A applies and Section B is `N/A`.

---

## Section 7 — Step 4 — Reconcile

Dispatch one read-only sub-agent (**V-reconcile**) to verify the directory is internally consistent.

V-reconcile does not edit any per-phase file or the master TEMPLATE; it only writes its own report and verifies the directory as a whole.

V-reconcile:

1. **Read every file in `<VERIFICATION_DIR>/`.**
2. **Verify each per-phase prompt:** has all 9 elements of the master TEMPLATE (Mission, Source-of-Truth Priority, Section A, Section B, Auto-Fix Rules, Subagent Orchestration, Output Format, Done Criteria, Per-Phase Preamble); the Per-Phase Preamble is accurate (spec path, handoff path, build-plan line range, implementation path re-discovered at reconcile time, coverage rows all match the on-disk state); the TEMPLATE body is byte-identical to the master modulo the `N` substitution.
3. **Verify the file-ownership map (Section 10):** no file is owned by more than one sub-agent.
4. **Write `<VERIFICATION_DIR>/RECONCILE-REPORT.md`** with a per-phase summary table, a "Findings" section, and a final verdict (`GO`, `CONDITIONAL GO`, or `NO-GO`).
5. **Read-only:** does not edit any other file. One commit: `<prefix>(verification): reconcile N per-phase verify prompts` with the `CO_AUTHOR_TRAILER`.

---

## Section 8 — Master TEMPLATE (copy-pasteable)

V0 writes the code block below verbatim into `<VERIFICATION_DIR>/TEMPLATE.md` (with `<PROJECT_NAME>`, `<BUILD_PLAN_PATH>`, `<HANDOFF_DIR>`, `<SPEC_DIR>`, `<COVERAGE_PATH>`, `<IMPL_ROOT>` substituted; nothing else). Per-phase sub-agents instantiate it with `N` substituted.

The TEMPLATE is the body of every per-phase prompt; the per-phase preamble (Section 9) is the only per-phase-specific content. The code block below is the exact content V0 writes to disk.

````markdown
# Phase N Verification Prompt (Template)

> Copy this file, replace every `N` with the phase number, then fill in the **Per-Phase Preamble** at the bottom.

## Mission

Verify that Phase N's forward-looking prompt (`<HANDOFF_DIR>/phase-N-prompt.md`), retrospective handoff (`<HANDOFF_DIR>/PHASE-N-HANDOFF.md`), build-plan section (`<BUILD_PLAN_PATH>` § "Phase N"), and on-disk implementation (`<IMPL_ROOT>/<component>/src/`) are all consistent with the component spec (where one exists) and the source-of-truth priority. Auto-fix any disparities by dispatching sub-agents per the 5-layer guarantees.

## Source-of-Truth Priority

When documents disagree, resolve in this order (highest first):

1. `<SPEC_DIR>/<component>/*.md` — canonical for aggregates, commands, events, permission/role, audit records. **N/A if no spec.**
2. `<BUILD_PLAN_PATH>` § "Phase N" — canonical for deliverables, tasks, exit criteria, risks.
3. `<HANDOFF_DIR>/PHASE-N-HANDOFF.md` — closing agent's claim of what shipped (validated against priority 4).
4. Implementation in `<IMPL_ROOT>/<component>/src/` — on-disk truth (source, tests, `Cargo.toml` deps, umbrella re-exports).
5. `<HANDOFF_DIR>/phase-N-prompt.md` — **LOWEST priority**; correct to match 1-4, not vice-versa.

## Section A: Pre-Implementation Check

> Run BEFORE implementation. Skip entirely for closed phases.

For each item: `Pass` (one-line citation) or `Fail` (one-line citation + the fix the auto-fix subagent will apply).

1. **Spec exists.** `<SPEC_DIR>/<component>/` (if applicable) has the full spec file set; for non-component phases the port contract / reference doc exists at the build-plan-named location.
2. **Build-plan § "Phase N" is complete.** Contains all required sub-sections (`Deliverables.`, `Tasks.`, `Exit criteria.`, `Risks.`, `Phase completion documentation.`).
3. **Coverage rows are `Pending`.** Every aggregate/feature the phase ships has a `Pending` row in `<COVERAGE_PATH>`. Pre-check file enumerates the current `Pending` count; the preamble lists the expected count.
4. **Scaffold component is in place.** `Cargo.toml` + `src/lib.rs` exist at `<IMPL_ROOT>/<component>/` and follow the standard scaffold pattern.

## Section B: Post-Implementation Check

> Run AFTER implementation. Applies to closed or in-progress phases.

For each dimension: `Pass` (one-line citation) or `Fail` (one-line citation + priority chain + the fix the auto-fix subagent will apply).

1. **Prompt ↔ Spec.** Every aggregate/feature name in the prompt matches `<SPEC_DIR>/<component>/` exactly.
2. **Prompt ↔ Build-Plan.** "Deliverables" + "Tasks" + "Exit Criteria" in the prompt match build-plan § "Phase N". Headline-N vs spec-faithful interpretation is consistent.
3. **Prompt ↔ Handoff.** "Where NOT to start" rules in the prompt match carry-forward rules in the handoff. "Do NOT" list in the prompt's "Per-Deliverable Gotchas" matches the handoff's "Where NOT to start" word-for-word (modulo phase-specific additions).
4. **Handoff ↔ Implementation.** Headline correctness check claimed in the handoff (e.g. proptest in `<IMPL_ROOT>/<component>/src/services.rs`) exists and is green. Aggregates, command shapes, events, permission/role + audit variants in the handoff's "What's wired and working" are present on disk. Handoff "Open questions" are resolved in the implementation or explicitly carried forward.
5. **Coverage Matrix ↔ Implementation.** Every `Tested` (or closed) row in `<COVERAGE_PATH>` for Phase N has a real implementation (not a stub returning `Err(not_supported)`). Every `Pending` row has either a real impl that should flip to `Tested` (flip in auto-fix) or an explicit "deferred to Phase N+M" rationale in the handoff.

## Auto-Fix Rules

The verify agent dispatches one sub-agent per failing dimension with file-level ownership and section-level pre-allocation per the 5-layer guarantees:

| Failing dimension | Subagent scope | Files owned |
| --- | --- | --- |
| 1. Prompt ↔ Spec | `fix-prompt-spec` | `<HANDOFF_DIR>/phase-N-prompt.md` |
| 2. Prompt ↔ Build-Plan | `fix-prompt-buildplan` | `<HANDOFF_DIR>/phase-N-prompt.md`, `<BUILD_PLAN_PATH>` § "Phase N" |
| 3. Prompt ↔ Handoff | `fix-prompt-handoff` | `<HANDOFF_DIR>/phase-N-prompt.md`, `<HANDOFF_DIR>/PHASE-N-HANDOFF.md` |
| 4. Handoff ↔ Implementation | `fix-handoff-impl` | `<IMPL_ROOT>/<component>/src/**` |
| 5. Coverage Matrix ↔ Implementation | `fix-coverage` | `<COVERAGE_PATH>`, `<IMPL_ROOT>/<component>/src/**` (stub-flips only) |

Multiple dimensions can run in parallel if they own disjoint files; if two want the same file, the prep sub-agent pre-allocates section markers. Each sub-agent produces exactly one atomic commit: `Phase N verify: <dimension> (<workstream>)` with the `CO_AUTHOR_TRAILER`.

## Subagent Orchestration (5-Layer Guarantees)

1. **File-level ownership.** Every file in the owned component is assigned to exactly one sub-agent.
2. **Section-level pre-allocation.** For shared files, the prep sub-agent pre-creates named section markers (`// === <Section> section begin (owner: <Letter>) ===` / `=== end ===` for code; `<!-- === <Section> begin (owner: <Letter>) === -->` / `=== end ===` for markdown). A sub-agent that crosses a marker aborts and reports.
3. **Sequential phase gates.** `P0 prep` → `R1 reconcile-prep` (read-only) → wave 1/2/3 parallel fix sub-agents → `R2 reconcile-impl` → `4-tests` (`cargo test --workspace`) → `5-docs` (`PHASE-N-VERIFY-REPORT.md`) → `R3 final-validation`. A stage does not start until the prior gate passes.
4. **Atomic commits per microtask.** Every sub-agent produces exactly one commit: `Phase N verify: <scope> (<workstream>)` + `CO_AUTHOR_TRAILER`. Orchestrator inspects `git log --stat` after every stage. Sub-agents in the parallel fix wave do NOT run `cargo test` — the orchestrator runs the gate.
5. **Reconciler sub-agents are read-only.** `R1`, `R2`, `R3` verify section boundaries + duplicate detection + stub-replacement but never write code. A violation halts the verify step.

## Output Format & Done Criteria

Write `<HANDOFF_DIR>/PHASE-N-VERIFY-REPORT.md` with five sections (A–E):

- **A — Pre-Implementation Check results.** `Pass`/`Fail` per item with one-line citation.
- **B — Post-Implementation Check results.** `Pass`/`Fail` per dimension with one-line citation.
- **C — Disparities Summary.** Bullet list of every `Failed` item with file + line + priority chain.
- **D — Fix Plan.** Ordered list of files to update (or "no fixes needed"); each fix names the file, change, and sub-agent scope.
- **E — GO/NO-GO verdict.** `GO` if all checks pass or all disparities are fixed in the same atomic commit; `NO-GO` otherwise.

**Done when:** all A items pass (unimplemented) OR all B dimensions pass (implemented); all listed disparities fixed or explicitly deferred with rationale + ADR reference; one atomic commit with fixes; `cargo test -p <component>` green (implemented); `cargo build --workspace` green; the project's tier-or-layer lint gate green; progress-tracker row for Phase N updated.

The verify report is one atomic commit; the fixes are a second atomic commit per the "Atomic commits per microtask" guarantee.

## Per-Phase Preamble

**Phase N — <Title>**

- **Spec:** `<path to <SPEC_DIR>/<component>/>` (or "no spec"; reference the port contract / reference doc instead).
- **Handoff:** `<HANDOFF_DIR>/PHASE-N-HANDOFF.md`.
- **Build-plan section:** `<BUILD_PLAN_PATH>` lines `<start>-<end>`.
- **Implementation component(s):** `<IMPL_ROOT>/<component>/` (one entry per component the phase ships; discovered at runtime by listing `<IMPL_ROOT>/`).
- **Coverage row IDs:** list from `<COVERAGE_PATH>` for this phase, with the spec-faithful target count if Section A applies.
- **Carry-forward rules:** paste the "Where NOT to start" + "Open questions" from the prior handoff, plus phase-specific rules from build-plan § "Phase N" "Risks." sub-section.
````

---

## Section 9 — Per-phase preamble template

Each VN appends a preamble to the bottom of the TEMPLATE body. Shape:

```markdown
# Per-Phase Preamble — Phase N

**Phase title:** <from build-plan>
**Status:** implemented / in progress / unimplemented

**Build-plan section:** `<BUILD_PLAN_PATH>` lines X–Y
**Spec:** `<path or "no spec">`           # list spec files if present
**Handoff:** `<path or "DOES NOT EXIST YET">`
**Implementation:** `<discovered at runtime under <IMPL_ROOT>/>`
**Coverage rows:** <list from `<COVERAGE_PATH>` or "no coverage matrix">
**Carry-forward rules:** <from prior handoff + build-plan § "Phase N" "Risks.">

# additional per-phase specifics (gaps, mismatches, scaffold state, drift) …
```

Typically 5–30 lines; TEMPLATE body is unchanged.

The orchestrator compares the discovered paths against the pre-check file and flags disagreement as a V-reconcile finding. Pre-fill the build-plan section, spec, handoff, implementation, and coverage rows from P0-prep's pre-check file (Step 1); only the "additional per-phase specifics" section requires per-phase research.

---

## Section 10 — Subagent orchestration (5-layer guarantees)

The 5 layers apply to **this directory's** build (not just the verify prompts):

1. **File-level ownership.** Every file in `<VERIFICATION_DIR>/` is assigned to exactly one sub-agent:

   | File | Owner |
   | --- | --- |
   | `PRE-CHECK-PHASES-[N1]-[N2].md` | **P0-prep** |
   | `README.md`, `TEMPLATE.md` | **V0** |
   | `[N]-PHASE-VERIFY-PROMPT.md` (N = 0..N_PHASES-1) | **VN** (one per phase) |
   | `RECONCILE-REPORT.md` | **V-reconcile** |

2. **Section-level pre-allocation.** Not applicable (each file is owned by exactly one sub-agent and never edited in parallel). If two per-phase prompts ever need to share a file, pre-allocate with markdown section markers.
3. **Sequential phase gates.** `P0-prep` → `V0` → `V0..VN_PHASES-1` (in parallel) → `V-reconcile`. A stage does not start until the prior stage's commit is on the branch.
4. **Atomic commits per microtask.** Every sub-agent produces exactly one commit with a `<prefix>(verification): ...` message and the `CO_AUTHOR_TRAILER`. Orchestrator inspects `git log --stat` after every stage.
5. **Reconciler sub-agents are read-only.** V-reconcile verifies consistency across the directory and never edits any file other than its own report.

---

## Section 11 — Two-section prompt semantics

Each per-phase prompt has TWO sections (Section A: pre-impl, Section B: post-impl). The future verification agent reads the preamble to determine which section(s) apply and runs them.

- **Section A: Pre-Implementation Check** applies to any phase that is unimplemented (no handoff, no scaffold implementation). It verifies the spec exists, the build-plan § "Phase N" is complete, the coverage rows are `Pending` (or the project's open-status name), and the scaffold component is in place.
- **Section B: Post-Implementation Check** applies to any phase that is implemented (closed or in progress). It compares 5 dimensions: Prompt ↔ Spec, Prompt ↔ Build-Plan, Prompt ↔ Handoff, Handoff ↔ Implementation, Coverage Matrix ↔ Implementation.

For in-progress phases, the future agent runs whichever section matches the current state. For an unscaffolded unimplemented phase, Section A only (Section B is `N/A` until the phase closes).

Full semantics live in the master TEMPLATE (Section 8); this section is a pointer, not a duplicate.

---

## Section 12 — Done criteria

The verification directory build is `Done` when ALL hold:

- `1 + 1 + 1 + N_PHASES + 1` files in `<VERIFICATION_DIR>/` (pre-check omitted if every phase is implemented).
- All per-phase prompts are byte-identical to the master TEMPLATE modulo the `N` substitution and the per-phase preamble.
- All per-phase preambles are accurate (spec path, handoff path, build-plan line range, implementation path, coverage rows all match the on-disk state).
- All commits carry the `CO_AUTHOR_TRAILER`; `cargo build --workspace` is still green (docs-only); `cargo fmt --all -- --check` is clean.
- `RECONCILE-REPORT.md` shows `GO` or `CONDITIONAL GO`.

A `NO-GO` verdict from V-reconcile is a blocker — fix the offending per-phase prompt and re-run V-reconcile until the verdict is `GO` or `CONDITIONAL GO`.

---

## Section 13 — How to run this prompt

1. Paste this prompt into a new AI session with the target Rust project as the working directory.
2. The AI asks for any variable overrides (defaults in Section 2) and dispatches sub-agents per Section 10: P0-prep → V0 → V0..VN_PHASES-1 (parallel) → V-reconcile.
3. The AI commits each sub-agent's work atomically with the `CO_AUTHOR_TRAILER`.
4. The AI runs the done-criteria checks in Section 12 and reports the result.
5. On `GO` / `CONDITIONAL GO` verdict, the directory is permanent; future closing agents consume the per-phase prompts at the close of each build-plan phase.

Output: a permanent `<VERIFICATION_DIR>/` directory for future closing agents + `RECONCILE-REPORT.md` proving internal consistency.