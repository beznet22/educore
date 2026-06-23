# Cluster G — Doc / Version Drift

**Root cause:** The engine's documentation has accumulated drift from
implementation reality. `AGENTS.md` claims 34 crates; the actual count
is 37. The build plan claims certain phases are closed; the audits
show their deliverables are partially or wholly missing. SurrealDB's
shipping status contradicts across 3 docs.

**Estimated findings:** ~215 (Low-Medium heavy; 12-19 Critical)

**Source ID prefixes:** `DOC-*`

**Blocks deploy:** No (docs don't block deploy by themselves)

**Estimated fix scope:** Medium. ~15 doc files to update. No code
changes. Mechanical sweep.

## Why these findings cluster

The engine has 4 categories of docs:

1. **Authoritative meta-docs** — `AGENTS.md`, `docs/project-overview.md`,
   `docs/build-plan.md`, `docs/architecture.md`.
2. **Standards docs** — `docs/code-standards.md`, `docs/library-docs.md`,
   `docs/query_layer.md`.
3. **Specs** — `docs/specs/<domain>/`.
4. **Guides** — `docs/guides/`.

Across these, the audit found 215 findings:

- **Contradictions** between docs (e.g., "SurrealDB is primary target"
  vs "SurrealDB deferred" vs "SurrealDB GA").
- **Stale crate counts** (34 vs 37 vs 38 — the actual count varies by doc).
- **Phantom APIs** documented in `docs/library-docs.md` that don't exist
  on `Engine`.
- **Phase status lies** (build-plan.md claims Phase X closed; audit
  shows Phase X deliverables missing).
- **Misleading coverage matrix** (`docs/coverage.toml` says "Tested"
  but tests don't exist).

## Representative findings (sample)

| Source | ID | Sev | Doc | One-line |
|---|---|---|---|---|
| `wave5-docs-1.md` | DOC-1-001 | C | project-overview + build-plan | SurrealDB shipping status tri-contradicts |
| `wave5-docs-1.md` | DOC-1-002 | C | library-docs | `Engine::builder()` facade methods not implemented |
| `wave5-docs-1.md` | DOC-1-003 | C | AGENTS.md | 34 vs 35 vs 37 crate count drift |
| `wave5-docs-1.md` | DOC-1-005 | C | architecture.md | Tier count wrong |
| `wave5-docs-1.md` | DOC-1-007 | H | build-plan.md | Phase 6 + Phase 8 missing outcome paragraphs |
| `wave5-docs-1.md` | DOC-1-008 | H | library-docs.md | `Engine::builder().sync(...)` claimed but not implemented |
| `wave5-docs-1.md` | DOC-1-009 | H | AGENTS.md | Cross-cutting list omits sync + sync-inprocess |
| `wave5-docs-2.md` | DOC-2-001 | C | library-docs.md | `Engine::admit_student(...)` claimed; doesn't exist |
| `wave5-docs-2.md` | DOC-2-005 | C | query_layer.md | Documented `EntityDescriptor` has 3 missing variants |
| `wave5-docs-2.md` | DOC-2-009 | C | ADR-018 | Sync feature flag not implemented |
| `wave5-docs-3.md` | DOC-3-001 | H | progress-tracker.md | Test-count off-by-11 between Phase 3 + Phase 4 outcomes |
| `wave5-docs-3.md` | DOC-3-005 | H | ADR-013 | CrateLayout docs disagree with filesystem |
| `wave5-docs-4.md` | DOC-4-002 | C | specs/sync/overview.md | 10 of 11 spec files missing |
| `wave5-docs-6.md` | DOC-6-001 | C | guides/saas-backend.md | `engine.<domain>()` phantom accessors (11 guides) |
| `wave5-docs-6.md` | DOC-6-005 | C | guides/saas-backend.md | Missing storage adapters (SurrealDB/MongoDB) |

(Full list of 215 findings is in `docs/audit_reports/05-audit-documentation.md`.)

## What fixing this requires

**Per-doc sweep**

For each affected doc:

1. **`AGENTS.md`**: reconcile the crate count (34 vs 37 vs 38) — pick
   one number, regenerate the inventory table.
2. **`docs/project-overview.md`**: remove the "SurrealDB is primary"
   claim if it's no longer accurate; either pick SurrealDB or pick
   Postgres.
3. **`docs/build-plan.md`**: rewrite Phase 0-17 outcome paragraphs to
   match audit reality; document the unfixed clusters.
4. **`docs/architecture.md`**: reconcile tier count, fix the DDL
   emission flow description, update the system map.
5. **`docs/library-docs.md`**: either implement the documented
   `Engine` methods OR remove them from the docs. Pick one.
6. **`docs/query_layer.md`**: align the documented `EntityDescriptor`
   with the actual AST.
7. **`docs/specs/sync/overview.md`**: either create the missing 10 spec
   files OR document why sync is out of scope.
8. **`docs/guides/*.md`**: same treatment as library-docs — implement
   the documented APIs or remove from the guide.
9. **`docs/coverage.toml`**: either add the missing rows OR update the
   existing rows to reflect reality.

**ADR updates**

The audit found:

- ADR-013 (CrateLayout): doesn't match filesystem.
- ADR-015 (ExternalCrates): pins need review.
- ADR-016 (EngineGraph): graph existence claimed; verify.
- ADR-017 (SurrealDBFirst): contradicts ADR-018 and project-overview.
- ADR-018 (SyncEngine): feature flag missing.

Each ADR should be reviewed and either updated or marked superseded.

## Suggested fix sequence

This cluster can be done last, in parallel with cluster E (engine rules)
which is also mechanical.

1. **Top-of-funnel sweep** — fix `AGENTS.md` crate count and tier
   inventory (1-2 days).
2. **Build plan** — rewrite phase outcomes to match audit reality (1
   week; touches the most-cited doc).
3. **Architecture + overview** — fix SurrealDB contradiction, DDL
   emission flow, system map (3-5 days).
4. **library-docs** — pick "implement or remove" for each phantom API
   (1-2 weeks; ~10 phantom methods).
5. **query_layer** — align with actual AST (2-3 days).
6. **guides** — implement or remove for each phantom API (1-2 weeks).
7. **coverage.toml** — add missing rows OR update existing rows (2-3
   days; this is mechanical).
8. **ADRs** — review and update (3-5 days).

## Verification criteria

- All `**id:** DOC-*` findings resolved by doc edit (or by code change
  that makes the doc accurate).
- `docs/coverage.toml` matches `docs/audit_reports/00-master-finding-table.md`
  in aggregate counts.
- `AGENTS.md` crate inventory matches `find crates -name Cargo.toml |
  wc -l`.

## Risk if left unfixed

- New contributors are misled by stale docs.
- Consumers write code against phantom APIs.
- The audit findings about "documented but missing" keep recurring.
- CI gates that reference coverage.toml are unenforced.

## Cross-cluster dependencies

- **Unblocks:** None directly.
- **Depends on:** Clusters A, B, C, D, E, F — many doc findings will
  become moot once the underlying code is fixed. Recommended: do
  cluster G **after** the foundation is in place.

## Files involved

- `AGENTS.md`
- `docs/project-overview.md`
- `docs/build-plan.md`
- `docs/architecture.md`
- `docs/code-standards.md`
- `docs/library-docs.md`
- `docs/query_layer.md`
- `docs/progress-tracker.md`
- `docs/decisions/ADR-013-CrateLayout.md`
- `docs/decisions/ADR-015-ExternalCrates.md`
- `docs/decisions/ADR-016-EngineGraph.md`
- `docs/decisions/ADR-017-SurrealDBFirst.md`
- `docs/decisions/ADR-018-SyncEngine.md`
- `docs/specs/sync/{aggregates,entities,commands,events,services,permissions,repositories,workflows,tables}.md` (9 new files)
- `docs/guides/*.md` (17 files)
- `docs/coverage.toml`
