# Entry-Point Agent Prompt

**Purpose:** This file contains the prompt to hand to the first
engineering agent (or subagent) starting remediation work. It does NOT
constrain scope — the receiving agent reads the remediation roadmap,
picks the highest-priority unblocked cluster, and executes. Parallel
subagents are authorized for independent work.

**Expected outcome:** A stream of PRs that close out clusters in the
order recommended by `08-dependency-graph.md`.

---

## Agent Prompt

**Role:** You are the remediation lead for the Educore engine audit.
Your job is to drive the 1,878 findings surfaced by the audit to zero.

**Authoritative plan:** `docs/audit_reports/remediation/`

The remediation roadmap is split across 11 files in that directory.
Start by reading them in this order:

1. `README.md` — methodology and severity legend
2. `00-overview.md` — triage, top hotspots, top 10 most-impactful
   findings
3. `08-dependency-graph.md` — sequencing rules, topological order,
   parallel windows
4. `01-...-a` through `07-...-g` — per-cluster detail (only as you
   pick up each cluster)
5. `09-quick-wins.md` — 15 items you can pick up any time without
   coordination

**Your responsibilities:**

1. **Day 1: ship quick wins first.** Before touching any cluster, run
   this Day 1 sprint. It produces 5+ small, independently-mergeable
   PRs in the first 3-5 days and warms the team's familiarity with the
   audit findings. See "Day 1 sprint plan" below.

2. **Pick the next cluster to work on.** Follow the dependency graph:
   D → A → B → F → C → E → G. Don't start C until A, B, D are stable.
   Don't start F until A is done. Quick wins can be picked up in any
   window where capacity is free.

3. **For each cluster you take on:**
   - Read the cluster file end-to-end.
   - Identify which findings are Critical vs High (skip Medium/Low
     until the Criticals are gone — they will get fixed as a side effect).
   - Open one PR per logically-grouped set of findings (don't lump
     unrelated fixes into one PR).
   - Reference the source finding ID (`DOM-AC-022`, `ADAPTER-PG-001`,
     etc.) in each PR body so the audit remains traceable.
   - Verify against the cluster's "Verification criteria" section
     before merging.

4. **Use parallel subagents freely.** The audit is large enough that
   serial work will take forever. Recommended parallelization:
   - Different clusters that the dependency graph says can run in
     parallel: spawn one subagent per cluster.
   - Different files within the same cluster: one subagent per file
     or per logical group.
   - Different quick wins: independent subagents, one per win.
   - All subagents must use `minimax-m3`. Reference this exact model
     in every `Agent` call. See the "Single-Model Mode" rule in the
     project's `AGENTS.md` for rationale.

5. **Update the audit as you go.** When a finding is closed by a PR:
   - Don't delete the finding from `docs/audit_reports/findings/`.
     The audit is a historical record.
   - Append a new file `docs/audit_reports/findings/closed/wave8-remediation-<date>.md`
     noting the PR, the finding IDs closed, and any new findings the
     PR surfaced.
   - The master table at `docs/audit_reports/00-master-finding-table.md`
     does not need to be regenerated; it remains a snapshot of the
     pre-remediation state. New waves (wave8+) track progress.

6. **Respect the audit charter.** The findings are findings only — no
   fixes, no recommendations, no "you should" language inside any
   `wave*.md` file. Remediation PRs are separate from the audit
   document.

**Day 1 sprint plan (run this before starting any cluster):**

Read `09-quick-wins.md` for the full list. The recommended first 5 PRs
are below. Each item is small enough to ship as one PR and is
independently mergeable. Spawn one subagent per item using
`minimax-m3`.

| PR | Quick win | Effort | Subagent scope |
|---|---|---|---|
| 1 | **QW-1** — make the lint binary runnable + **QW-9** — add 2 missing umbrella re-exports | ~2 hours | Single subagent, one PR, both items |
| 2 | **QW-4** — add explicit `Drop` impl to `Transaction` port | 1 day | Single subagent; touches all 4 adapters (impl updates) |
| 3 | **QW-3** — replace `Box::leak` in storage adapters with `OnceCell` / `LazyLock` interning | 1-2 days per adapter | **4 parallel subagents**, one per storage adapter (postgres, mysql, sqlite, surrealdb) |
| 4 | **QW-12** — implement `Idempotency::record` returning `Conflict` | 2-3 days per adapter | **4 parallel subagents**, one per storage adapter |
| 5 | **QW-6** — add `school_id` index to all 6 cross-cutting tables | 1 hour per adapter | **4 parallel subagents**, one per adapter |

Total Day 1 PR count: 5 PRs touching ~25 files. Estimated wall-clock
to land all 5: 3-5 days with the parallel subagent structure above.
Total estimated person-days: ~10 person-days if done serially,
~3-5 days with parallelism.

**While the Day 1 PRs are landing, in parallel:**

- You (the lead) start on **Cluster D** — the lint module. Read
  `04-cluster-d-foundation.md` end-to-end. The lint module is the
  single highest-leverage cluster and will take 2-4 weeks. This is
  too large to delegate cleanly; do it yourself or pair on it.
- Pick up **QW-7** (JWT secret loading) and **QW-8** (rate limiting)
  yourself or via subagents — both touch `educore-auth` which is a
  high-stakes crate and benefits from senior review.
- Pick up **QW-13** (outbox partition enforcement) via 4 parallel
  subagents, one per storage adapter.

**After Day 1 (Day 5+):**

- Cluster D's lint module should be 30-50% done.
- 5+ quick wins landed.
- The team is familiar with the audit findings format, the dependency
  graph, and the per-cluster files.
- Begin Cluster A (DDL emission gap) per `01-cluster-a-ddl-emission.md`.
  This is also large; consider pairing or splitting the macro work
  (cluster A stage 2) from the adapter work (cluster A stage 3).

**Out of scope (escalate to the user, don't decide yourself):**

- Resolving cross-domain ownership collisions flagged in
  `03-cluster-c-spec-drift.md` (SubjectAttendance, ExamAttendance,
  SpeechSlider). These need a human ADR.
- Picking SurrealDB vs Postgres as the primary storage backend. The
  docs contradict each other; the engine cannot ship until this is
  decided.
- Renaming any public API (engine-rule violations, identity type
  renames). Coordinate across consumers first.

**Stop conditions (escalate, don't continue):**

- If a cluster's fix touches more than 50 files in a single PR,
  split it and continue.
- If a cluster's "Done means" criteria from `08-dependency-graph.md`
  fail after the fix is in, investigate before declaring done.
- If the lint module's spec→code direction discovers >100 missing
  items, don't try to fix them in one PR — pick the most-trafficked
  10 and ship a partial.
- If a subagent fails 3 times on the same task, stop spawning and
  read the failure output yourself.

**Done means (overall, when all clusters are closed):**

- `cargo run -p educore-core --bin lint --features lint` exits 0
- `cargo test --workspace` passes
- `cargo clippy --workspace --all-targets -- -D warnings` passes
- `cargo fmt --all -- --check` passes
- All 4 storage adapters' `create_schema()` round-trip on fresh
  instances
- `docs/coverage.toml` rows for `tables.md` and `workflows.md` no
  longer reference missing files

Begin by reading the 4 priority files (README, 00-overview, 08-deps,
01-cluster-d) and announcing which cluster you're taking first.
