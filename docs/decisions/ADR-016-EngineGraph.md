# ADR-016: Engine Graph (graphify)

## Status

Accepted, 2026-06-09.

## Context

AI agents exploring the engine burn tokens when navigating 4 MB of
docs + 616 KB of crates + 452 KB of migrations via `Read` /
`Grep` / `Glob` calls. A pre-computed knowledge graph gives
O(1) lookups for spec-to-code traceability, aggregate-to-event
chains, and crate dependency relationships.

The existing `graphify` tool (already used for the legacy
`schoolify/` Laravel codebase) supports incremental regen,
git-merge-driver union-merge, and post-commit hooks.

## Decision

Index the engine source tree (`crates/`, `docs/`, `migrations/`,
and the top-level `*.md` files) with `graphify`, output at
`graphify-out/` at the repo root. Auto-rebuild on every commit
via a tracked post-commit hook at `.githooks/post-commit`
(per ADR-016 update, 2026-06-30). Users opt in once per clone
with `git config core.hooksPath .githooks`; the hook is
non-fatal (warn-but-do-not-fail) so a missing `graphify`
binary never blocks a commit. The legacy per-user
`graphify hook install` path remains supported for users who
already have it set up.

The legacy `schoolify/graphify-out/` is **frozen** and retained
as a research artefact only; AGENTS.md no longer directs agents
to it.

## Rationale

- **Token efficiency:** a 1-time graph download replaces N
  `Read` / `Grep` calls during exploration. A graph node for
  "academic_students aggregate" is ~200 bytes; reading the full
  spec + aggregate + command + event is ~50 KB. 250× compression.
- **Existing infra:** the `graphify` tool is already in use; no
  new dependency.
- **Conflict-free merges:** `graphify hook install` sets up a git
  merge driver that union-merges `graph.json` so two developers
  committing in parallel never get conflict markers.
- **Standard convention:** `graphify-out/` at the project root is
  the standard graphify layout. The schoolify one lives at
  `schoolify/graphify-out/` only because it was run from inside
  the `schoolify/` subdirectory.

## Excludes

The `.graphifyignore` at the repo root excludes:
- `schoolify/` (legacy Laravel, frozen; separate graph)
- `docs_guidlines/` (the authoring guidelines that were used to
  write the current docs; not part of the engine source)
- `target/` (cargo build output)
- `.git/` (git internals)
- `graphify-out/cache/` (graphify volatile cache)
- `graphify-out/cost.json` (regen cost metrics, local-only)

The `.gitignore` at the repo root also excludes
`graphify-out/cost.json` for the same reason.

The committed artefacts are: `graphify-out/graph.html`,
`graphify-out/graph.json`, `graphify-out/manifest.json`,
`graphify-out/GRAPH_REPORT.md`. These are committed for static
browsing; new contributors get the latest graph on clone.

## Consequences

- (+) ~5-10 MB git repo growth (the committed parts of
  `graphify-out/`).
- (+) Sub-second exploration queries via `graphify query`.
- (+) Auto-rebuild on commit; no manual step.
- (+) Git merge driver prevents graph.json conflict markers.
- (+) **Fresh-clone parity**: the post-commit hook at
  `.githooks/post-commit` is committed in the repo
  (Wave 33, ADR-016-GRAPHIFY-HOOK). New contributors no
  longer miss the automation — they only need a one-line
  `git config core.hooksPath .githooks` after cloning.
- (-) One-time `git config core.hooksPath .githooks` per
  clone (replaces the prior per-user `graphify hook install`
  step; lower friction).
- (-) New 5th validation criterion (graph regen freshness) in
  `docs/build-plan.md` § No-Gaps Gates.

## Alternatives

- **Weekly cron regen:** rejected — stale intra-week.
- **Phase-completion gate:** rejected — stale intra-phase.
- **CI regen on every PR:** rejected — duplicates the local hook;
  graph would be re-extracted twice per commit (once local, once CI).
- **No graph for the engine:** rejected — token burn is real.
- **Use a different graph tool** (e.g. `cog`, `codegraph`,
  `aider`'s repo map): rejected — `graphify` is already in use, no
  reason to add a new dep.

## See also

- [`.githooks/post-commit`](../../.githooks/post-commit) — the
  committed post-commit hook (Wave 33, ADR-016-GRAPHIFY-HOOK)
- [`AGENTS.md` § "Engine Graph (graphify)"](../../AGENTS.md#engine-graph-graphify)
- `graphify-out/GRAPH_REPORT.md` — the engine graph report
- `schoolify/graphify-out/` — the legacy Laravel graph (frozen)
- `.graphifyignore` at the repo root — the engine source exclude list
- [`docs/build-plan.md` § "No-Gaps Gates"](../build-plan.md#the-no-gaps-gates) —
  the 5th validation criterion (graph regen freshness)
- [`CONTRIBUTING.md` § "The engine graph"](../../CONTRIBUTING.md#the-engine-graph)
