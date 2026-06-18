# `docs/verification/`

Permanent verification prompts for the **Educore** school-domain
engine. The closing agent for each build-plan phase runs the
corresponding `N-PHASE-VERIFY-PROMPT.md` after the handoff +
phase prompt are committed, to verify the implementation + docs
are in sync with the spec and the source-of-truth priority.
Verify prompts share the same 5-layer subagent-orchestration
guarantees as the forward-looking phase prompts (see
[`docs/phase_prompt/README.md` § "Subagent Orchestration"](../phase_prompt/README.md#subagent-orchestration)).

---

## Directory layout

- **`README.md`** — this file (user guide).
- **`TEMPLATE.md`** — the master two-section template every
  per-phase prompt is rendered from.
- **`PRE-CHECK-PHASES-13-17.md`** — read-only snapshot of the
  pre-implementation state for the 5 unimplemented phases
  (13, 14, 15, 16, 17). Consumed by the per-phase subagents
  (V14-V18) when they write the per-phase verify prompts.
  After the verify prompts land, this file is historical.
- **`N-PHASE-VERIFY-PROMPT.md`** (N = 0..17) — per-phase
  instances, one per build-plan phase. Each is rendered from
  `TEMPLATE.md` with the per-phase preamble filled in.
- **`RECONCILE-REPORT.md`** — read-only consistency check
  across all per-phase prompts (written by a reconciler
  subagent after the per-phase prompts are committed).

---

## When to invoke

After every phase close (Phase 0 through Phase 17). The closing
agent for phase N:

1. Commits the handoff + phase prompt + coverage flips + build
   plan outcome section as usual.
2. Commits the per-phase verify prompt (`N-PHASE-VERIFY-PROMPT.md`,
   rendered from `TEMPLATE.md` with the per-phase preamble
   filled in). For Phases 14-17, this commit is informed by
   `PRE-CHECK-PHASES-13-17.md`.
3. Then runs `N-PHASE-VERIFY-PROMPT.md` as the instruction
   for a fresh verification agent.
4. The fresh agent verifies + auto-fixes any disparities
   (dispatching subagents per the 5-layer guarantees).
5. The fresh agent writes `docs/handoff/PHASE-N-VERIFY-REPORT.md`
   and commits.

The verify step is the **last** close-time action for the
phase; it may flip the phase from "closed" to "verified".

---

## Two-section prompt semantics

Each per-phase prompt has TWO sections. The future verification
agent reads the per-phase prompt, determines which section(s)
apply, and runs them:

- **Section A — Pre-Implementation Check.** Applies to Phases
  13-17 (unimplemented) and any phase not yet closed. Verifies
  the spec exists, the build-plan § "Phase N" is complete, the
  coverage rows are `Pending`, and the scaffold crate is in
  place. Output: `Pass` / `Fail` per checklist item.
- **Section B — Post-Implementation Check.** Applies to Phases
  0-11 (implemented) and Phase 12 (in progress — run when
  closed). Compares 5 dimensions: Prompt ↔ Spec, Prompt ↔
  Build-Plan, Prompt ↔ Handoff, Handoff ↔ Implementation,
  Coverage Matrix ↔ Implementation. Output: `Pass` / `Fail`
  per dimension.

For a future Phase 18+ verify prompt, both sections apply if
the phase is unimplemented (write Section A first, then
Section B for the post-implementation work the phase will do).

---

## How to run a prompt

1. Open the per-phase prompt (e.g. `11-PHASE-VERIFY-PROMPT.md`).
2. Pass it as the instruction to a fresh AI agent (opencode
   CLI, Claude, or equivalent) with the engine repo as
   working directory. The agent reads the prompt, the prior
   handoff, the build-plan § "Phase N", the spec (if any),
   and the implementation, then runs the two sections.
3. The agent dispatches subagents per the 5-layer guarantees
   (file-level ownership, section-level pre-allocation,
   sequential phase gates, atomic commits per microtask,
   read-only reconcilers). See `TEMPLATE.md` § "Subagent
   Orchestration" for the full rules.
4. The agent writes `docs/handoff/PHASE-N-VERIFY-REPORT.md`
   and commits the fixes as one atomic commit (per the
   "Atomic commits per microtask" guarantee; the verify
   report is one microtask, the fixes are a second).

---

## Expected output

`docs/handoff/PHASE-N-VERIFY-REPORT.md` with five sections:

- **Section A — Pre-Implementation Check results.**
  `Pass` / `Fail` per item, with a one-line citation.
- **Section B — Post-Implementation Check results.**
  `Pass` / `Fail` per dimension, with a one-line citation.
- **Section C — Disparities Summary.** Bullet list of every
  item that `Failed` in Section A or Section B, with the
  specific file + line + the source-of-truth priority chain
  that resolves it.
- **Section D — Fix Plan.** Ordered list of files to update
  (or "no fixes needed" if both sections pass). Each fix
  item names the file, the change, and the subagent scope
  per the 5-layer guarantees.
- **Section E — GO/NO-GO verdict.** `GO` if all checks pass
  or all disparities are fixed in the same commit; `NO-GO`
  if any fix is deferred or any check is open.

The verify report is **read-only** in spirit (it does not
modify the engine); the fixes it triggers are the only
mutation, and they go in a separate atomic commit.

---

## 5-layer guarantees

The verify prompts use the same 5-layer subagent-orchestration
guarantees as the forward-looking phase prompts. The full text
is reproduced from
[`docs/phase_prompt/README.md` § "Subagent Orchestration"](../phase_prompt/README.md#subagent-orchestration):

1. **File-level ownership.** Every file in the owned crate is
   assigned to exactly one subagent. No two subagents open
   the same file. The orchestrator maintains a file-ownership
   map in the phase plan and embeds the list of forbidden
   files in every parallel-subagent prompt.
2. **Section-level pre-allocation.** For files that must be
   touched by multiple workstreams (e.g. `aggregate.rs` for
   3+ root aggregates), the prep subagent pre-creates the
   file with named section markers
   (`// === <Aggregate> section begin (owner: <WorkstreamLetter>) ===`
   / `// === <Aggregate> section end ===`). Each workstream
   subagent's `Edit` anchors fall strictly inside its assigned
   range. A subagent that crosses a marker aborts and reports
   to the orchestrator.
3. **Sequential phase gates.** The phase advances through
   fixed stages: `P0 prep` (single subagent, scaffolds shared
   files + cross-crate extensions) → `R1 reconcile-prep`
   (read-only verifier) → `wave 1/2/3` parallel workstreams →
   `R2 reconcile-impl` → `4-tests` → `5-docs` → `R3
   final-validation` (9-command gate). A phase does not start
   until the prior phase's gate passes.
4. **Atomic commits per microtask.** Every subagent produces
   exactly one commit with a `Phase N: <scope> (<workstream>)`
   message + `Co-Authored-By: Antigravity <antigravity@google.com>`
   trailer. The orchestrator inspects `git log --stat` after
   every phase to detect any out-of-scope file. A "do not run
   cargo test" rule applies to the parallel phase — the
   orchestrator runs the gate, not the subagents.
5. **Reconciler subagents are read-only.** `R1`, `R2`, `R3`
   are dedicated reconciler subagents. They verify section
   boundaries + duplicate detection + stub-replacement but
   never write code. A reconciler that finds a violation halts
   the phase.

For verify prompts, the same guarantees apply with one
adaptation: the R1/R2/R3 reconcilers are "verify-only" (they
read the source-of-truth priority chain and report
disparities, they do not modify code). The auto-fix subagents
that resolve disparities are subject to the file-level
ownership + section-level pre-allocation rules.

---

## Adding a new per-phase prompt

To add a verify prompt for a future phase (Phase 18+):

1. Copy `TEMPLATE.md` to `N-PHASE-VERIFY-PROMPT.md` (where
   `N` is the new phase number).
2. Replace every `N` with the new phase number (Mission,
   Source-of-Truth Priority, Section A preamble, Section B
   preamble, Auto-Fix Rules, Output Format, Done Criteria,
   Per-Phase Preamble).
3. Fill in the **Per-Phase Preamble** at the bottom of the
   file: the spec path (or "no spec"), handoff path, build-plan
   line range, implementation crate, coverage row IDs, and
   any known carry-forward rules (read the prior handoff's
   "Where NOT to start" + "Open questions" sections).
4. Run `wc -l N-PHASE-VERIFY-PROMPT.md` to confirm the file
   is the same length scale as the other per-phase prompts
   (typically 80-150 lines; the TEMPLATE is ~120 lines and
   the per-phase preamble adds 5-30 lines).
5. Commit as `docs(verification): add phase N verify prompt`.

To add a verify prompt for an existing phase (e.g. retro-fitting
Phase 0-12), follow the same recipe but consume the prior
handoff + the build-plan § "Phase N" + the implementation at
its current state. Section A may be omitted (N/A) for phases
that closed before this directory existed; Section B is the
primary scope.

---

## See also

- [`docs/phase_prompt/README.md`](../phase_prompt/README.md) —
  the forward-looking phase prompts. The verify prompts mirror
  the 5-layer subagent-orchestration guarantees from the
  "Subagent Orchestration" section.
- [`docs/handoff/PHASE-N-HANDOFF.md`](../handoff/PHASE-N-HANDOFF.md) —
  the per-phase retrospective hand-off. The verify report
  (`PHASE-N-VERIFY-REPORT.md`) is written to the same
  directory.
- [`docs/build-plan.md`](../../build-plan.md) — the 17-phase
  implementation roadmap. Each phase's § "Phase N" is the
  source-of-truth priority-2 input to the verify step.
- [`docs/coverage.toml`](../../coverage.toml) — the
  machine-readable coverage matrix. The verify step is the
  authority on whether the `Pending` / `Tested` flips are
  consistent with the implementation.
- [`AGENTS.md`](../../AGENTS.md) — the engine rules (no
  `unwrap`, validation checklist, tier boundaries, file-scoped
  commands). The verify step enforces the validation checklist.
