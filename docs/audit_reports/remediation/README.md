# Remediation Roadmap

**Generated:** 2026-06-23
**Source audit:** `docs/audit_reports/findings/` (46 files, 1,878 findings)
**Companion reports:** `docs/audit_reports/00-master-finding-table.md` and `01-...` through `08-...`

## What this is

A planning document for fixing the 1,878 production-readiness findings surfaced by
the audit. It is **not** the audit itself (no findings, no evidence) and it is
**not** implementation (no code changes). It is the triage layer between the two.

## The 7 root-cause clusters

The audit surfaced 1,878 findings, but most are symptoms of a small number of
underlying root causes. Grouping by root cause (instead of by finding) collapses
~1,878 items into 7 workstreams.

| ID | Cluster | Estimated findings | Source ID prefixes | Severity | Blocks deploy |
|---|---|---|---|---|---|
| **A** | DDL emission gap | ~150 | ADAPTER, ADAPT, PAR, PORT | Critical-heavy | **Yes** |
| **B** | Workflow infrastructure | ~80 | WF, CC, CROSSCUT | Critical-heavy | **Yes** |
| **C** | Spec ↔ code drift | ~600 | DOMAIN, DOM, SPEC | Mixed | **Yes** (partial) |
| **D** | Foundation crate gaps | ~70 | CORE, PORT, INFRA, UMB | Mixed | **Yes** (partial) |
| **E** | Engine-rule violations | ~400 | DOMAIN, DOM, INFRA, CC | Medium-heavy | **Yes** (per code-standards) |
| **F** | Adapter port-contract gaps | ~250 | ADAPTER, ADAPT, PAR | Critical-heavy | **Yes** (per adapter) |
| **G** | Doc / version drift | ~215 | DOC | Low-Medium | No |

Counts are estimates from sampling; the per-cluster files contain the actual
finding IDs that were assigned to each cluster.

## File index

```
docs/audit_reports/remediation/
├── README.md                       ← you are here (index + methodology)
├── 00-overview.md                  ← triage summary, top hotspots, recommended entry point
├── 01-cluster-a-ddl-emission.md    ← cluster A — 310 domain tables not emitted
├── 02-cluster-b-workflow.md        ← cluster B — no subscribers, no outbox relay
├── 03-cluster-c-spec-drift.md      ← cluster C — aggregate/command/event gaps per domain
├── 04-cluster-d-foundation.md      ← cluster D — educore-core::lint stub, error/AST gaps
├── 05-cluster-e-engine-rules.md    ← cluster E — unwrap/expect/panic/as/Value violations
├── 06-cluster-f-adapter-gaps.md    ← cluster F — port contract not implemented per adapter
├── 07-cluster-g-doc-drift.md       ← cluster G — phase status lies, naming drift, contradictions
├── 08-dependency-graph.md          ← what blocks what; recommended sequencing
└── 09-quick-wins.md                ← first things to fix (no blockers)
```

## Severity legend (binary readiness)

- **Critical** = blocks deploy
- **High** = major gap / feature unusable
- **Medium** = minor broken
- **Low** = cosmetic

## Methodology

1. **Cluster assignment is heuristic.** A finding belongs to a cluster if its
   root cause is the cluster's root cause. Many findings touch multiple
   clusters (e.g., a spec-drift finding in a domain crate also violates an
   engine rule); they are assigned to the cluster that represents their
   primary failure mode.

2. **Severity is taken from the source finding**, not re-derived.

3. **Counts are sampling-based**, not exhaustive. Each cluster file lists a
   representative subset of finding IDs that explain the cluster; the master
   table (`docs/audit_reports/00-master-finding-table.md`) is the
   authoritative inventory.

4. **No fixes are proposed.** This document is for prioritization and
   sequencing only. Implementation lives in PRs against the codebase.

## Recommended entry point

Read `00-overview.md` first. It contains the triage summary, the top
hotspots (files with the most Critical findings), and the recommended first
fix sequence.

Then read `08-dependency-graph.md` to understand the sequencing constraints
between clusters.

Then drill into the per-cluster files in priority order:
1. Cluster **D** (foundation) — unlocks everything else
2. Cluster **A** (DDL emission) — unblocks all adapter work
3. Cluster **B** (workflow) — needed for any multi-aggregate test
4. Cluster **C** (spec drift) — bulk of remaining work
5. Cluster **E** (engine rules) — mechanical cleanup, can run in parallel
6. Cluster **F** (adapter gaps) — depends on A + D
7. Cluster **G** (doc drift) — last; non-blocking

## Known gaps in the roadmap

- The 7-cluster model is provisional. Findings touching multiple clusters are
  assigned once; the master table contains the uncollapsed view.
- Some findings may be technically infeasible to fix without redesigning the
  affected crate. These are flagged in their cluster file with a note.
- The roadmap does not estimate effort (story points, days). Each cluster
  file calls out *scope* (small / medium / large / XL) but not *duration*.
- No CI integration is proposed. The audit's no-gaps gates are documented in
  `docs/build-plan.md` § "The No-Gaps Gates" (line 1825); whether to add a
  new gate or amend an existing one is an open question.
