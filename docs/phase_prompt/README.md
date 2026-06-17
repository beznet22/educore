# Phase Prompts — `docs/phase_prompt/`

> Per-phase AI-agent briefs for the **Educore** school-domain
> engine. Each `phase-N-prompt.md` is a forward-looking brief
> for the agent that will execute build-plan phase N, written
> at the close of phase N-1.

---

## At a glance

This directory is one of two that together form the
"hand-off + next-phase brief" operating contract for
every implementation:

| Directory | Purpose | Written at | Read by |
| --- | --- | --- | --- |
| **`docs/phase_prompt/`** (this dir) | Per-phase forward-looking briefs (one per build-plan phase) | Close of phase N-1 | Agent about to start phase N |
| [`docs/handoff/`](../handoff/) | Per-phase retrospective hand-offs (`PHASE-N-HANDOFF.md`) | Close of phase N | Agent about to start phase N+1 (or anyone reviewing history) |

The closing agent writes **both** the hand-off (in
`docs/handoff/`) and the next-phase prompt (in this
directory) at the close of every phase, per the convention
documented below.

---

## Contents

- **[`README.md`](README.md)** — this file; the convention
  for creating and consuming per-phase prompts.
- **[`phase-0-prompt.md`](phase-0-prompt.md)** — Phase 0
  mission (foundation; **retrospective**, written at the
  close of Phase 1 for the 1:1 phase-to-prompt mapping).
- **[`phase-1-prompt.md`](phase-1-prompt.md)** — Phase 1
  mission (storage adapter parity; already executed; the
  **canonical reference** for the per-phase prompt template).
- **[`phase-2-prompt.md`](phase-2-prompt.md)** — Phase 2
  mission (cross-cutting foundations; the next-phase
  prompt for whoever picks up after Phase 1).
- _(Future phases land their `phase-N-prompt.md` at the
  close of phase N-1.)_

---

## The per-phase prompt convention

**The rule.** At the close of every build-plan phase N
(N ≥ 0), the closing agent creates
`docs/phase_prompt/phase-(N+1)-prompt.md` to brief the
next-phase agent. The build plan's per-phase "Phase
completion documentation" task (one bullet per phase,
appended to the phase's `Tasks.` list) is the
authoritative spec for this convention.

### When to create a phase prompt

- At the close of Phase 0 → create `phase-1-prompt.md`
  (already done; see file).
- At the close of Phase 1 → create `phase-2-prompt.md`
  (already done; see file).
- At the close of Phase N → create
  `phase-(N+1)-prompt.md` (going forward).
- At the close of the **last** phase (Phase 17) → no
  next-phase prompt is needed; instead, create
  `phase-17-prompt.md` only if a Phase 18+ is planned.

### When NOT to create a phase prompt

- The phase is still open (no hand-off is happening).
- The phase is in maintenance mode (no new entry point
  to brief).
- The phase is the final phase and no successor is
  planned.

### Template — what a good phase prompt contains

A per-phase prompt **MUST** contain these eight
sections in this order. The **Required Reading** section
is mandatory and must list every file the receiving
agent must read for full docs context (spec,
hand-off, build-plan, ports, schemas, ADRs, template
crates, integration tests, `AGENTS.md`, `docs_guidlines/`).

1. **Mission** — one paragraph: what the phase delivers,
   which prior phases it depends on, whether this is
   implementation or design.
2. **Required Reading** (priority order) — **MANDATORY**.
   List every file the receiving agent must read for
   full docs context: the prior hand-off, the
   build-plan phase section, the relevant port
   contracts, every file in the relevant spec dir
   (`docs/specs/<domain>/`), the relevant schemas
   (`docs/schemas/*.md`), the relevant ADRs
   (`docs/decisions/ADR-*.md`), the template crates
   (`crates/cross-cutting/...`, `crates/domains/...`),
   the integration-test template
   (`crates/tools/storage-parity/tests/..._*_integration.rs`),
   `AGENTS.md`, the engineering guidelines
   (`docs_guidlines/system.md` +
   `docs_guidlines/execution_guidlines.md`). This section
   is the most important: the receiving agent cannot
   act correctly without reading these files.
3. **Deliverables** — the crate(s) + their scope, the
   vertical-slice test mirror, the docs deliverables at
   close.
4. **Working With Subagents** — a hard instruction to
   the agent to spawn parallel subagents via the task
   tool for the phase's independent workstreams, plus
   a per-phase "workstreams" callout that names the
   subagent scopes. See
   [§ "Working With Subagents" rationale](#working-with-subagents-rationale)
   below for why this is universal.
5. **Starting Point** — the template crate / file to
   copy from, the pre-written DDL / spec to lean on, the
   integration-test template to mirror.
6. **Per-Deliverable Gotchas** — the dialect-specific
   traps, the floor versions, the API gotchas, the
   workspace lint consequences. (Storage-adapter phases
   call this "Per-Dialect Gotchas"; non-storage phases
   call it "Per-Deliverable Gotchas" and adapt the
   structure.)
7. **Exit Criteria** — the bullet list of
   `cargo test ...`, `cargo clippy ...`, etc. that the
   phase must close green.
8. **When You Are Stuck** — the no-gaps gate command
   (`cargo run -p educore-core --bin lint --features lint`),
   the hand-off doc, and the policy on opening issues
   vs asking the user.

**Length: ≤50 lines. MANDATORY.** The prompt is a
digest, not a replacement for the spec, the hand-off,
the build-plan § "Phase N", or `AGENTS.md`. Long-form
context lives there — the prompt just points at it.
Each of the 8 sections above is typically 3–6 lines.
The build plan `§ "Phase N"` carries the canonical
50-line rule (added in Phase 5 close-out); every
subsequent phase's "Phase completion documentation"
task inherits the rule via a 1-line pointer.

#### "Working With Subagents" rationale

Every phase of the build plan has multiple independent
deliverables (one per crate, one per adapter, one per
subsystem). The closing agent writes the next-phase
prompt at the close of every phase, and the convention
is that the **receiving** agent uses the task tool to
spawn parallel subagents for those workstreams. This
is a hard rule, not a tip — the per-phase "Working
With Subagents" section in each prompt exists to make
this expectation explicit and to name the specific
workstream scopes for that phase.

The rationale: parallel subagents maximize speed
(independent deliverables finish in the wall-clock
time of the slowest) and context-window utilization
(each subagent's prompt is scoped to one deliverable,
not the whole phase). The closing agent retains
responsibility for the cross-cutting integration work
(workspace gates, coverage rows, hand-off, next-phase
prompt); the subagents do the per-crate work.

### What is NOT in a per-phase prompt

The prompt is a digest, not a replacement. The full
context lives elsewhere:

- **Phase spec** — `docs/build-plan.md` § "Phase N" (the
  canonical, complete spec).
- **Retrospective hand-off** —
  `docs/handoff/PHASE-N-HANDOFF.md` (what the closing
  phase did; written alongside the next-phase prompt).
- **Completion-documentation recipe** — the per-phase
  "Phase completion documentation" task in
  `docs/build-plan.md` (the bullet that tells the closing
  agent to write the hand-off + the next-phase prompt).
- **Universal agent rules** — `AGENTS.md` § "Agent
  Instructions" (no `unwrap`, file-scoped commands,
  validation checklist, …). The phase prompt adds
  phase-specific guidance on top; it does not duplicate
  the universal rules.

---

## Closing-Agent Verification Checklist

At the close of every build-plan phase N (N ≥ 0), the closing agent creates `docs/phase_prompt/phase-(N+1)-prompt.md` to brief the next-phase agent. Before committing the next-phase prompt, the closing agent must verify all 8 items below. A failed check is a blocker — fix the prompt, not the check.

- [ ] **Mission** mentions "implementation, not design" and declares the spec-faithful scope (or explicitly notes the headline-N interpretation with OQ #1 carried forward).
- [ ] **Required Reading** lists the prior handoff, build plan § "Phase N", every file in `docs/specs/<domain>/` (all 11 per AGENTS.md), the relevant port contracts, schemas, ADRs, the most recent 9-file domain template, the storage-parity integration test, `AGENTS.md`, and `docs_guidlines/`. Required Reading is **mandatory** — the prompt is a digest, not a replacement for the spec.
- [ ] **Aggregate names** in the prompt match `docs/specs/<domain>/aggregates.md` exactly (e.g. `NoticeBoard` not `Notice` if the spec says `NoticeBoard`). Cross-reference with the spec before committing.
- [ ] **"Do NOT" carry-forward rules** are present in Per-Deliverable Gotchas (e.g. no `educore-finance` dep, no `educore-notify` dep — both per Phase 8/10 OQs; verify against the prior handoff's "Where NOT to start" section).
- [ ] **Workstreams** are named with letter notation (`A=`, `B=`, `C=`, etc.) and the last workstream is `reconcile` (cross-crate placeholders + integration test + coverage flips + handoff docs).
- [ ] **Exit Criteria** names the close-time deliverables: `PHASE-N-HANDOFF.md` + `phase-(N+1)-prompt.md` + `progress-tracker.md` + `build-plan.md § "Phase N outcome."`.
- [ ] **Coverage target** reflects the chosen scope (spec-faithful = full aggregate count, headline-N = N rows). Verify against `docs/coverage.toml` Pending count.
- [ ] **Total line count ≤ 50** (run `wc -l docs/phase_prompt/phase-N-prompt.md` to verify; the prompt is a digest, not a replacement for the spec).

## Subagent Orchestration

To prevent two or more subagents from being given the same work, every phase must enforce the following 5-layer guarantees. These are the rules that closed Phases 8–11 successfully (the first phase to break these rules will produce a duplicate-work collision and a non-mergeable state).

1. **File-level ownership.** Every file in the owned crate is assigned to exactly one subagent. No two subagents open the same file. The orchestrator maintains a file-ownership map in the phase plan and embeds the list of forbidden files in every parallel-subagent prompt.
2. **Section-level pre-allocation.** For files that must be touched by multiple workstreams (e.g. `aggregate.rs` for 3+ root aggregates), the prep subagent pre-creates the file with named section markers (`// === <Aggregate> section begin (owner: <WorkstreamLetter>) ===` / `// === <Aggregate> section end ===`). Each workstream subagent's `Edit` anchors fall strictly inside its assigned range. A subagent that crosses a marker aborts and reports to the orchestrator.
3. **Sequential phase gates.** The phase advances through fixed stages: `P0 prep` (single subagent, scaffolds shared files + cross-crate extensions) → `R1 reconcile-prep` (read-only verifier) → `wave 1/2/3` parallel workstreams → `R2 reconcile-impl` → `4-tests` → `5-docs` → `R3 final-validation` (9-command gate). A phase does not start until the prior phase's gate passes.
4. **Atomic commits per microtask.** Every subagent produces exactly one commit with a `Phase N: <scope> (<workstream>)` message + `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. The orchestrator inspects `git log --stat` after every phase to detect any out-of-scope file. A "do not run cargo test" rule applies to the parallel phase — the orchestrator runs the gate, not the subagents.
5. **Reconciler subagents are read-only.** `R1`, `R2`, `R3` are dedicated reconciler subagents. They verify section boundaries + duplicate detection + stub-replacement but never write code. A reconciler that finds a violation halts the phase.

---

## Versioning

The per-phase prompts in this directory are part of the
engine. Changes are reviewed in the same way as engine
source:

- Workspace `cargo fmt` / `cargo clippy` / `cargo test`
  must remain green (the markdown is not linted by
  rustc, but the prose must be consistent with the
  current code).
- Changes to a `phase-N-prompt.md` are scoped to that
  phase and should be reviewed alongside the phase's
  hand-off and outcome documentation in
  `docs/build-plan.md` § "Phase N outcome.".

---

## See also

- [`AGENTS.md`](../../AGENTS.md) — workspace rules, naming,
  lint policy, the universal "Agent Instructions" the
  phase prompt builds on.
- [`docs/build-plan.md`](../../build-plan.md) — the
  17-phase implementation roadmap; every phase carries
  a "Phase completion documentation" task that points
  here.
- [`docs/coverage.toml`](../../coverage.toml) — the
  machine-readable coverage matrix that each phase flips
  on close.
- [`docs/handoff/PHASE-N-HANDOFF.md`](../handoff/PHASE-N-HANDOFF.md)
  — the per-phase retrospective hand-off (one per closed
  phase; written alongside the next-phase prompt).
- [`docs/decisions/ADR-015-ExternalCrates.md`](../../decisions/ADR-015-ExternalCrates.md)
  — the external crate selection rationale (referenced
  by every per-phase prompt's "Required Reading"
  section).
