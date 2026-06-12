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

Following the [`phase-1-prompt.md`](phase-1-prompt.md)
template, a per-phase prompt should contain these seven
sections in this order:

1. **Mission** — one paragraph: what the phase delivers,
   which prior phases it depends on, whether this is
   implementation or design.
2. **Required Reading** (priority order) — the prior
   hand-off, the build-plan phase section, the relevant
   port contracts, the relevant specs, `AGENTS.md`, the
   guidelines.
3. **Working With Subagents** — a hard instruction to
   the agent to spawn parallel subagents via the task
   tool for the phase's independent workstreams, plus
   a per-phase "workstreams" callout that names the
   subagent scopes (e.g., "Phase 1 workstreams: PG,
   MySQL, SQLite are three independent subagent
   scopes; spawn them in a single batch."). See
   [§ "Working With Subagents" rationale](#working-with-subagents-rationale)
   below for why this is universal.
4. **Starting Point** — the template crate / file to
   copy from, the pre-written DDL / spec to lean on, the
   integration-test template to mirror.
5. **Per-Deliverable Gotchas** — the dialect-specific
   traps, the floor versions, the API gotchas, the
   workspace lint consequences. (Storage-adapter phases
   call this "Per-Dialect Gotchas"; non-storage phases
   call it "Per-Deliverable Gotchas" and adapt the
   structure.)
6. **Exit Criteria** — the bullet list of
   `cargo test ...`, `cargo clippy ...`, etc. that the
   phase must close green.
7. **When You Are Stuck** — the no-gaps gate command
   (`cargo run -p educore-core --bin lint --features lint`),
   the hand-off doc, and the policy on opening issues
   vs asking the user.

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
