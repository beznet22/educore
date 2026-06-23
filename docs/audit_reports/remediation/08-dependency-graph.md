# Dependency Graph — Cluster Sequencing

**Purpose:** Visualize what blocks what. Identifies the topologically
sorted order in which clusters can be fixed.

## Cluster nodes

```
A: DDL emission gap          ~150 findings, Critical-heavy
B: Workflow infrastructure    ~80 findings, Critical-heavy
C: Spec ↔ code drift         ~600 findings, Mixed
D: Foundation crate gaps      ~70 findings, Mixed
E: Engine-rule violations    ~400 findings, Medium-heavy
F: Adapter port-contract gaps ~250 findings, Critical-heavy
G: Doc / version drift       ~215 findings, Low-Medium
```

## Dependency graph

```
   ┌─────────┐
   │  D      │  (foundation — no deps)
   └────┬────┘
        │ unlocks AST, port stability, lint module
        ├──────────────────┐
        ▼                  ▼
   ┌─────────┐       ┌─────────┐
   │  A      │       │  B      │
   │  DDL    │       │ workflow│
   └────┬────┘       └────┬────┘
        │                 │
        │ unlocks adapter │
        │ DDL emission    │ unlocks per-domain
        │                 │ integration tests
        ▼                 ▼
   ┌─────────┐       ┌─────────┐
   │  F      │◄──────│  C      │
   │ adapter │  feeds│ spec    │
   │ gaps    │  gaps │ drift   │
   └────┬────┘       └────┬────┘
        │                 │
        │ both feed       │
        ▼                 ▼
   ┌─────────────────────────┐
   │  E                       │
   │  engine rules            │
   └─────────────────────────┘

   ┌─────────┐
   │  G      │  (doc drift — depends on A-F being fixed first
   └─────────┘   so the docs can be written against reality)
```

## Edges (what blocks what)

| Edge | Meaning |
|---|---|
| D → A | Cluster D's `EntityDescriptor` AST completion is required for cluster A's macro emission. |
| D → B | Cluster D's port-trait stability is required for cluster B's bus port completion. |
| A → F | Cluster A's DDL emission is required for cluster F's adapter completeness. |
| B → C | Cluster B's workflow infrastructure is required for cluster C's `tests/workflows.rs` per domain. |
| C → F | Cluster C's per-domain aggregates/commands/events are required inputs to cluster F's per-adapter repository implementations. |
| A,B,C,D,F → E | Cluster E (engine rules) is enabled by lint detection from D; mechanically applied across A, B, C, F's file changes. |
| A,B,C,D,E,F → G | Cluster G (docs) should be done last; many doc findings become moot after A-F land. |

## Topological order (fix in this order)

| Step | Cluster | Why this slot |
|---|---|---|
| 1 | **D** (foundation) | No deps; unlocks everything else. The lint module is the single highest-leverage fix. |
| 2 | **A** (DDL emission) | Depends on D's AST; unblocks F (adapter completeness). |
| 3 | **B** (workflow) | Depends on D's port stability; can be parallel with A. |
| 4 | **F** (adapters) | Depends on A's DDL emission; depends on D's port stability. |
| 5 | **C** (spec drift) | Depends on A (macro works), B (workflow infra), D (lint checks). Largest cluster; takes longest. |
| 6 | **E** (engine rules) | Should run in parallel with C; can be done as a final sweep before G. Mechanical. |
| 7 | **G** (doc drift) | Should be done last; depends on A-F being fixed so docs can be accurate. |

## Parallelism

Some clusters can run in parallel:

| Window | Clusters that can run in parallel |
|---|---|
| Window 1 | **D** alone (foundation has no deps; should start first) |
| Window 2 | **A** + **B** (both depend on D; independent of each other) |
| Window 3 | **F** + (start of **C**) (F depends on A; C can start filling in `aggregate.rs` while F is being implemented) |
| Window 4 | **C** + **E** (engine rules sweep can run continuously) |
| Window 5 | **G** (after everything else) |

## Critical-path analysis

The shortest path from "today" to "no Critical findings":

```
D → A → F → (some C sub-tasks) → "no Critical"
```

This is the critical path. Estimated duration:

| Step | Estimate |
|---|---|
| D | 2-4 weeks |
| A | 2-3 weeks |
| F | 4-6 weeks |
| C (Critical subset) | 8-12 weeks |
| **Total critical path** | **~6 months** |

(Cluster E and G run in parallel and don't add to the critical path.)

## Risks to the sequencing

| Risk | Mitigation |
|---|---|
| D's lint module is more complex than estimated | Cut scope: ship the spec→code direction first (the most valuable check), defer anti-pattern checks to a follow-up. |
| A's macro emission hits a spec ambiguity | Resolve per-domain before applying the macro. Track spec edits in `docs/coverage.toml`. |
| B's outbox relay requires upstream changes to the storage adapters | Combine B with F's adapter work in a single effort. |
| C's per-domain gap fill uncovers more spec gaps | Update spec docs in the same PR as the code fix. Avoid spec/code drift. |
| E's engine-rule sweep conflicts with in-flight refactors | Coordinate via a single PR; freeze Cluster E until critical path clusters are stable. |

## Open sequencing questions

1. **Should D's lint module be a separate, larger effort, or scoped to
   spec↔code direction first?** Recommended: scoped. ~2 weeks for the
   smallest useful version.
2. **Should F's adapter work happen before or after C's per-domain
   work?** Recommended: in parallel. Adapters don't need every domain's
   full surface to start emitting DDL; pick one or two domains
   (academic + finance) for F's reference impl.
3. **Should E's engine-rule sweep be a single mass PR or per-crate?**
   Recommended: per-crate, batched by tier (infra → cross-cutting →
   domains → adapters → tools).

## Verification at each step

After each cluster is "done":

| Cluster | Verification |
|---|---|
| D | `cargo run -p educore-core --bin lint --features lint` exits 0 on the same codebase (or at least: fewer findings than before). |
| A | `storage.create_schema().await` round-trips on a fresh PG instance. |
| B | `cargo test -p educore-testkit` shows outbox drains AND bus subscribers receive. |
| C | `cargo test -p <domain>` passes for each closed-out domain. |
| E | `cargo clippy --workspace --all-targets -- -D warnings` exits 0. |
| F | `cargo test --workspace` passes; parity suite green for all 4 backends. |
| G | All `DOC-*` findings closed; no new doc drift introduced. |
