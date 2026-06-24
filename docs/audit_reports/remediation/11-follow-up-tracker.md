# Follow-Up Tracker

**Purpose:** Persistent record of TODOs, stubs, placeholders, and
follow-up items left behind by the audit-remediation PRs. Read this
**before starting any cluster** to avoid re-doing work or missing
context.

**Generated:** 2026-06-23
**Last updated by:** lead agent during Cluster A stage 3 launch

---

## A. Stubbed / placeholder work in current code

These are intentional stubs that close the audit's "missing"
findings but leave the implementation as a placeholder. The real
work is tracked below.

| ID | Item | Source commit | Stub today | Real work needed |
|---|---|---|---|---|
| A-1 | **Macro emits `ColumnType::Custom("UNKNOWN")` for every field** | `e036f73` | Every column gets `Custom("UNKNOWN")` | Type inference: parse the field's Rust type and map to a `ColumnType` variant. E.g. `Uuid` → `ColumnType::Uuid`, `String` → `ColumnType::String`, `i64` → `ColumnType::I64`, `bool` → `ColumnType::Bool`, `chrono::DateTime<Utc>` → `ColumnType::Timestamp`. Requires extending the macro to inspect `Field::ty`. |
| A-2 | **`EntityDescriptor.indexes` is always empty** | `e036f73` | `Vec::new()` | Derive indexes from `#[query(filterable)]` and `#[query(sortable)]` fields. Each filterable field gets a single-column index. Composite indexes for multi-field sorts are a follow-up. |
| A-3 | **`EntityDescriptor.foreign_keys` is always empty** | `e036f73` | `Vec::new()` | Derive from `#[query(relation = "...", builder = "...")]` fields. The referenced table can be inferred from the builder type's `ENTITY_DESCRIPTOR`. |
| A-4 | **`EntityDescriptor.rls` is always empty** | `e036f73` | `Vec::new()` | If the struct has a `school_id: SchoolId` field (or any field whose type contains `SchoolId`), emit a `school_isolation` RLS policy with `USING school_id = current_setting('app.school_id')::uuid`. Adapters that don't support RLS (SQLite, SurrealDB) skip this; others (Postgres, optionally MySQL 8) emit it. |
| A-5 | **StorageAdapter port default impl returns `NotSupported`** | in-flight | All 4 adapters are still being updated | Each adapter overrides `create_schema()`. In-flight as of this writing (Cluster A stage 3 subagents). |
| A-6 | **MySQL adapter: RLS skipped with TODO** | Cluster A stage 3 (in-flight) | `// TODO: MySQL RLS` comment in `schema.rs` | MySQL 8 supports policies but with caveats (no `current_setting()`, needs `SESSION_VAR()` or a session variable trick). Future PR. |
| A-7 | **SQLite adapter: RLS not supported** | Cluster A stage 3 (in-flight) | Documented in code | SQLite has no RLS natively. Acceptable limitation; document in adapter README. |

### Adapter column-type mapping (Cluster A stage 3 subagents implement this)

This is the canonical mapping per the audit's spec; each adapter uses it. Until type inference lands, every column gets `Custom("UNKNOWN")` which the adapter falls back to a sensible default for.

| `ColumnType` | Postgres | MySQL | SQLite | SurrealDB |
|---|---|---|---|---|
| `Uuid` | `UUID` | `CHAR(36)` | `TEXT` | `TYPE string` |
| `String` | `VARCHAR(255)` | `VARCHAR(255)` | `TEXT` | `TYPE string` |
| `Text` | `TEXT` | `TEXT` | `TEXT` | `TYPE string` |
| `I64` | `BIGINT` | `BIGINT` | `INTEGER` | `TYPE int` |
| `U64` | `BIGINT` (unsigned) | `BIGINT UNSIGNED` | `INTEGER` | `TYPE int` |
| `I32` | `INTEGER` | `INT` | `INTEGER` | `TYPE int` |
| `U32` | `INTEGER` | `INT UNSIGNED` | `INTEGER` | `TYPE int` |
| `F64` | `DOUBLE PRECISION` | `DOUBLE` | `REAL` | `TYPE float` |
| `Bool` | `BOOLEAN` | `BOOLEAN` | `INTEGER` (0/1) | `TYPE bool` |
| `Timestamp` | `TIMESTAMPTZ` | `DATETIME(6)` | `TEXT` (RFC 3339) | `TYPE datetime` |
| `Json` | `JSONB` | `JSON` | `TEXT` | `TYPE object` |
| `Bytes` | `BYTEA` | `BLOB` | `BLOB` | `TYPE bytes` |
| `Custom(s)` | emit `s` verbatim | emit `s` verbatim | emit `s` verbatim | emit `TYPE s` |
| `Custom("UNKNOWN")` fallback | `TEXT` | `TEXT` | `TEXT` | `TYPE string` |

---

## B. Lint module gaps (Cluster D follow-ups)

| ID | Item | Status | Notes |
|---|---|---|---|
| B-1 | **Code→spec direction has false positives for re-exports** | Implemented but heuristic | `lib.rs` re-exports like `pub use foo::Bar;` are not currently exempted. Will produce false-positive "undocumented public item" violations. Test for re-export exemption is in place but the implementation may need more refinement against real crates. |
| B-2 | **`as` cast detection is regex-based** | Implemented but heuristic | The current check flags `as u32`, `as i64`, etc. but may also flag `as &dyn Trait` if the trait happens to be named after a primitive. False positives are tolerable per the original design but should be tracked. |
| B-3 | **Pre-existing clippy warnings** | Unfixed | `crates/infra/core/src/lint.rs:83` (eprintln!), `:44` (duplicated attribute), `:394` (manual char comparison). These block `cargo clippy --workspace --all-targets -- -D warnings` as a CI gate. |
| B-4 | **Coverage matrix sync's TOML parser is line-based** | Implemented but not full TOML | Avoids the `toml` crate dep. Handles the audit's TOML conventions but may break on multi-line arrays, comments, or table headers. Re-validate when `docs/coverage.toml` is regenerated. |
| B-5 | **Anti-pattern check doesn't catch `as` chains on numeric constants** | Gap | `0u32 as i64` is caught; `Some(0u32) as i64` may not be if the regex misses the keyword. Re-validate on real codebase. |
| B-6 | **Single-line `mod tests { ... }` exemption fix may have edge cases** | Fixed in `b98ae86` | The change from `idx > lo && idx < hi` to `idx >= lo && idx <= hi` is correct for single-line blocks but needs regression testing across the workspace to confirm no production unwraps slipped through. |

---

## C. Quick Wins that have remaining work

| ID | Item | Source commit | Remaining |
|---|---|---|---|
| C-1 | **QW-12 Idempotency::record is additive** | `5382a6e` (port) + 4 adapter commits | The new `record_outcome()` method returns `IdempotencyOutcome::Conflict` correctly. The original `record()` method is unchanged and still returns `Ok(())` on duplicate. **Callers must migrate** to `record_outcome()` to actually detect duplicates. Track domain-by-domain migration as part of Cluster C. |
| C-2 | **QW-13 MySQL fix is defense-in-depth** | `d2f52c9` | The MySQL adapter's `pending()` was ALREADY filtering by `self.school` (set at connect time). The PR added `pending_for_school()` and a school-id predicate on `mark_published()` for cross-tenant-attempt defense. The testkit (TOOL-TK-001) is NOT fixed — outbox drain still drops envelopes in the in-memory adapter. |
| C-3 | **QW-13 testkit outbox drain not fixed** | TOOL-TK-001 | The testkit's `InMemoryTransaction::commit` drains the outbox and discards it. Need a follow-up PR to either (a) wire the outbox to the in-process bus, or (b) keep the drain behavior but add a test-only flag for explicit draining. |
| C-4 | **QW-7/QW-8 (auth) was originally two parallel agents** | `db72274` | The successful PR was done by ONE coordinated agent after two parallel agents diverged. Lesson: when changes touch a shared module (lib.rs), use ONE agent, not parallel. |

---

## D. Cross-cutting concerns

| ID | Item | Source | Action |
|---|---|---|---|
| D-1 | **Pre-existing stash** | session start | `git stash list` shows `stash@{0}: pre-existing-unrelated-changes-stashed-by-agent` containing `docs_guidlines/PROMPT.md`, `docs_guidlines/execution_guidlines.md`, `docs_guidlines/query_layer.md`, `docs_guidlines/query_optimze.md`, `docs_guidlines/system.md` (deletions) and an untracked `PROMPT.md` at the repo root. **User must resolve** — these are not part of remediation work. |
| D-2 | **`docs_guidlines/` directory deletions** | session start | The 5 files in `docs_guidlines/` are deleted in the working tree. Not part of remediation. User must commit or restore. |
| D-3 | **`graphify-out/` auto-regenerated on every commit** | every commit | The graphify hook auto-regenerates `graphify-out/GRAPH_REPORT.md` and `graphify-out/graph.json`. Always shows up as "modified" in `git status`. Do NOT commit manually unless the user explicitly wants to. |
| D-4 | **Phase 17 missing from build plan** | `wave5-docs-1.md` | Per the audit, `docs/build-plan.md` describes phases 0-16 but the engine roadmap has 17 phases. Either Phase 17 doesn't exist (and the audit is wrong) or it's missing from the docs (and should be added). User decision needed. |
| D-5 | **Cross-domain ownership collisions** | `wave6-specs-1.md` | `SubjectAttendance` claimed by both academic and attendance specs; `ExamAttendance` claimed by both academic and assessment; `SpeechSlider` claimed by both cms and events-domain. User must write an ADR for each. |
| D-6 | **SurrealDB vs Postgres as primary backend** | `wave5-docs-1.md` | AGENTS.md says SurrealDB is the primary target. `docs/decisions/ADR-017-SurrealDBFirst.md` says SurrealDB-first. `docs/decisions/ADR-018-SyncEngine.md` implies Postgres. `docs/project-overview.md` may also conflict. User decision needed. |
| D-7 | **Public API renames** | `wave5-docs-2.md` | Several identity types and event types are documented under names that differ from the code. Renames touch consumers; user coordination required. |
| D-8 | **`educore-storage-surrealdb` partial implementation** | `wave3-storage-surrealdb.md` | `apply_snapshot`, `watch_changes`, `cursor_for`, `advance_cursor` are unimplemented stubs. SurrealDB's change-stream API is what these would call. |
| D-9 | **Auth crate pre-existing clippy debt** | `wave5-docs-1.md` | `educore-auth` has unresolved clippy warnings; blocks the workspace `cargo clippy ... -D warnings` gate. |
| D-10 | **Macro proc-macro `as` cast** | `docs/build-plan.md:73` | The PR-0 fix-up notes that the proc-macro had an `as` cast that needed fixing. Verify it's actually fixed in current `crates/infra/query-derive/src/lib.rs`. |

---

## E. Cluster B (workflow infrastructure) — COMPLETE

Per `08-dependency-graph.md`: B follows A. With Cluster A stage 3 in flight, B is unblocked.

| ID | Item | Source findings |
|---|---|---|
| E-1 | **Outbox relay** | `wave7-workflows.md` WF-005, `wave4-testkit.md` TOOL-TK-001 |
| E-2 | **Subscriber registration** | `wave7-workflows.md` WF-002 (zero subscribers wired), WF-016 (phantom `form_uploaded_public_indexing_subscriber`) |
| E-3 | **`tests/workflows.rs` per domain** | `wave7-workflows.md` WF-001, `docs/build-plan.md:1860` |
| E-4 | **Saga / compensating actions** | `wave7-workflows.md` WF-006 (no compensating actions for multi-step workflows) |
| E-5 | **Bus port completion** | `wave2-events.md` CC-EVT-001..007 (envelope schema, ack/nack, BatchReceipt bug) |

---

## F. Cluster C (spec↔code drift) — not started (600 findings; per-domain gap fill)

| ID | Item | Source findings |
|---|---|---|
| F-1 | **Per-domain aggregate gap fill** | `wave6-specs-1.md`, `wave6-specs-2.md` (full spec↔code drift inventory) |
| F-2 | **`#[derive(DomainQuery)]` adoption** | 0/310 across all domains; lint detects all of them |
| F-3 | **Naming drift sweep** | `wave6-specs-1.md` (e.g., `StudentId` vs `StudentIdentifier`, table singular vs plural) |
| F-4 | **Cross-domain ownership resolution** | See D-5 |

---

## G. Cluster E (engine-rule sweep) — COMPLETE

~400 violations across 10 domain crates + 7 cross-cutting + 3 infra + 10 adapters + 4 tools. Lint now auto-detects them.

| ID | Item | Source findings |
|---|---|---|
| G-1 | **`unwrap()` / `expect()` / `panic!()` sweep** | `wave5-docs-2.md` DOC-2-018, all `wave1-*` / `wave4-*` domain findings |
| G-2 | **`as` cast sweep** | Same |
| G-3 | **`serde_json::Value` sweep (domain code only)** | Same |
| G-4 | **`HashMap<String, T>` sweep (domain code only)** | Same |

---

## H. Cluster F (adapter port-contract gaps) — partial

Many findings closed by PRs 1-5; remaining are deeper port-trait issues.

| ID | Item | Source findings |
|---|---|---|
| H-1 | **StorageAdapter::create_schema() impls** | In-flight (Cluster A stage 3) |
| H-2 | **Transaction::TenantContext propagation** | `wave4-storage-port.md` PORT-STORE-002 |
| H-3 | **Atomic audit-write in same transaction** | `wave4-storage-port.md` PORT-STORE-013 |
| H-4 | **Outbox partition enforcement at storage port level** | `wave4-testkit.md` TOOL-TK-004 |
| H-5 | **SurrealDB change-stream stubs** | `wave3-storage-surrealdb.md` ADAPTER-SD-005..008 |
| H-6 | **Port-adapter gaps (auth/event-bus/notify/payment/files/integrations)** | Various `wave3-*` findings |

---

## I. Cluster G (doc/version drift) — not started

~215 findings across AGENTS.md, project-overview, architecture, build-plan, code-standards, library-docs, query_layer, ADRs.

| ID | Item | Source findings |
|---|---|---|
| I-1 | **Crate count drift** | AGENTS.md says 34, actual is 37 |
| I-2 | **SurrealDB shipping status tri-contradiction** | project-overview + build-plan + ADR-017/018 |
| I-3 | **Phase 17 missing from build plan** | See D-4 |
| I-4 | **`library-docs.md` phantom `Engine::builder()` APIs** | `wave5-docs-2.md` |
| I-5 | **`guides/*.md` phantom `engine.<domain>()` accessors** | `wave5-docs-6.md` |
| I-6 | **ADR review (013, 015, 016, 017, 018)** | Various `wave5-docs-2.md` findings |

---

## J. In-flight work

Status as of this writing:

| Cluster | Subagent | Commit landed? | Notes |
|---|---|---|---|
| Cluster A stage 3 (postgres) | `acc77291` | uncommitted (work exists on disk) | subagent reports "aborted" but produced schema.rs + port trait change |
| Cluster A stage 3 (mysql) | `9404a380` | uncommitted | work exists on disk |
| Cluster A stage 3 (sqlite) | `49e78cb1` | uncommitted | subagent not yet polled |
| Cluster A stage 3 (surrealdb) | `54dd54df` | uncommitted | work exists on disk |

**All four Cluster A stage 3 subagents have produced real work but
none has committed yet.** Future sessions should:

1. Verify each agent's changes compile (`cargo build --workspace`)
2. Commit each adapter's `create_schema()` impl separately
3. Add to the audit's `findings/wave8-remediation-*.md` log

---

## K. Open questions for the user

These cannot be decided by the lead agent. Each requires human input.

1. **SurrealDB vs Postgres as primary backend** (D-6)
2. **Cross-domain ownership collisions** (D-5)
3. **Phase 17 missing from build plan** (D-4)
4. **Public API renames** (D-7)
5. **Pre-existing unrelated changes in stash** (D-1)

---

## L. Verification at end of remediation

When all clusters close, the audit's no-gaps gates (per `docs/build-plan.md:1825`) should turn green:

- [ ] `cargo run -p educore-core --bin lint --features lint` exits 0
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes (currently blocked by D-9 + B-3)
- [ ] `cargo fmt --all -- --check` passes
- [ ] All 4 storage adapters' `create_schema()` round-trip on fresh instances
- [ ] `docs/coverage.toml` rows for `tables.md` and `workflows.md` no longer reference missing files

Until all 6 boxes are checked, residual Critical findings remain deploy-blockers.

---

## M. Subagent abort-recovery pattern (from this session)

**Why this section exists.** During this remediation session, multiple subagents were aborted due to `max_turns` or `duration limit exceeded`. This section documents the recovery pattern that emerged so future sessions don't lose work.

### The three outcomes of an aborted subagent

| Outcome | Description | Recovery |
|---|---|---|
| **A. Committed before abort** | Agent finished its work and committed, then was killed by the harness | Nothing — work is in git |
| **B. Staged/unstaged work, no commit** | Agent produced real changes but the commit step was the last thing it tried | **Lead agent recovers manually**: see "Recovery procedure" below |
| **C. No output** | Agent produced nothing useful (e.g., stuck mid-thought) | **Lead agent retries** with higher turn budget |

### Recovery procedure (for outcome B)

When a subagent aborts and the lead agent sees uncommitted changes via `git status --short`:

1. **Inventory the uncommitted changes:**
   ```bash
   git status --short | grep -v "graphify-out"
   ```

2. **For each modified/added file:**
   - If it's a new file (??) — likely safe, just verify it compiles
   - If it's modified (M or M ) — may have build errors from incomplete work
   - If it's staged (M with leading space) — agent got partway through commit

3. **Verify the workspace still builds:**
   ```bash
   cargo build --workspace 2>&1 | tail -3
   ```
   If errors, identify which files are broken.

4. **For each build error:**
   - Read the error message
   - Look at the offending file
   - Common errors from this session:
     - **Type mismatch on `event_type`**: the events crate migration from `&'static str` to `String` left consumers expecting `&str`. Fix: `envelope.event_type.as_str()` or `event_type.to_owned()`
     - **Missing `pub mod schema`**: a new module file was added but `lib.rs` not updated. Fix: add `pub mod schema;`
     - **Wrong function signature**: agent created a function with extra parameter. Fix: align with the port trait
     - **Missing dep**: agent used `futures::executor::block_on` but `futures` isn't a dep. Fix: use `tokio` runtime helper
   - Fix the error
   - Re-run `cargo build`

5. **Commit on behalf of the agent:**
   ```bash
   git add <fixed files>
   GIT_EDITOR=true git commit -m "<original commit message, slightly modified>"
   ```

6. **Verify the commit landed:**
   ```bash
   git log -1 --stat
   ```

7. **Document the recovery** in this tracker's section M (this section)

### Tuning parameters to reduce aborts

Future subagent launches should use these turn budgets:

| Task type | max_turns | token_budget | max_duration |
|---|---|---|---|
| Single-file refactor | 60-80 | 60k | 600s |
| Cross-crate coordination (e.g., QW-7+QW-8) | 100-120 | 120k | 900s |
| Multi-file adapter implementation | **150-200** | **150k** | **1500s** |
| Macro / proc-macro work | 100-150 | 100k | 1200s |
| Lint extension (multi-direction) | 80-100 | 100k | 900s |
| Per-domain workflows.rs file | 150 | 150k | 1200s |

**Lesson learned:** subagents reading many files in Phase 1 (10+ spec files) then writing in Phase 2 need 80-100 turns minimum. Default to **max_turns=150** unless the task is trivial.

### Aborted-agent recovery log (this session)

| Agent ID | Task | Outcome | Recovery |
|---|---|---|---|
| `57553e56` | QW-7 JWT secret loading | C — no commit | Discarded + retried as part of `db72274` (QW-7+QW-8 coordinated) |
| `01245b16` | QW-8 rate limiting | C — no commit | Discarded + retried as part of `db72274` |
| `6d6d6f4a` | Cluster D parity + matrix | B — partial, code broken | Discarded + retried as `3528cfb` |
| `acc77291` | Cluster A stage 3 postgres | A — already in `d18d8ee` | Lead's commit accidentally absorbed the agent's staged work |
| `9404a380` | Cluster A stage 3 mysql | B — staged, signature wrong | Lead fixed: `db288ff` → wait, that's wrong. `d7ae99c` |
| `49e78cb1` | Cluster A stage 3 sqlite | A — committed `02a2e63` | (subagent committed before being marked aborted) |
| `54dd54df` | Cluster A stage 3 surrealdb | B — staged, return-type wrong | Lead fixed: `d7ae99c` |
| `c3713d9b` | Cluster B subscribers | B — partial | Discarded + retried (combined with QW-8 in `db72274` later) |
| `1139ced8` | Cluster B workflows tests | B — partial (1 domain only) | Discarded + retried as `81ef68b` (academic) + per-domain agents |
| `7b00123b` | Cluster B outbox relay | A — relay_envelope.rs in `cbb7d3a` | Lead did bridging work + `6f6cb87` |
| `6f27037a` | Cluster B relay wiring retry | B — partial, Cargo.toml dep missing | Lead committed `1db5ad8` (Cargo.toml cleanup) |
| `f99532be` | Cluster B hr-library (first attempt) | C — no output | Discarded + retried as `5204a82` |
| `5cee76aa`, `bafdf739`, `3d1d49ad`, `90f3409a`, `8afdb7d5`, `9b206603`, `1bc7bad0`, `2614797`, `380033a`, `df0bc02`, `df0b74c`, `3c45e28`, `5204a82` | Cluster B per-domain workflows | A — all committed | Nothing to do |

### Future improvement (deferred)

A `scripts/recover-aborted-agents.sh` automation that:
1. Detects uncommitted changes after a subagent abort
2. Runs `cargo build` to identify errors
3. Applies common fixes (the 4 listed in step 4 above)
4. Commits with the original commit message
5. Notifies the lead agent

This is out of scope for this remediation session but should be a follow-up PR to `crates/tools/cli/` or a new `crates/tools/recovery/` crate.

---

## N. Cluster F status (Cluster F COMPLETE)

After Cluster F work (6 commits: `1952864`, `b8dc5e7`, `d414782`, `dd429ed`, `eba1ae3`, `e700c67`):

### Completed (6 of 6 sub-clusters) — CLUSTER F COMPLETE

| Crate | Findings closed | Tests added |
|---|---|---|
| `educore-notify` | ADAPT-NOT-005, 007, 010, 012 (DLQ, retry, failover, rate limit) | 17 |
| `educore-files` | ADAPT-FILE-003 (storage quota + tenant guard) | 6 |
| `educore-payment` | ADAPT-PAY-005, 008 (webhook signature + signing key) | 6 |
| `educore-integrations` | ADAPT-INT-005, 007, 009 (retry cap, signing-key rotation, replay protection) | 27 |
| `educore-storage` (port trait) | PORT-STORE-002, PORT-STORE-013 (TenantContext + atomic audit) | ~30 (postgres/mysql/sqlite/surrealdb/testkit) |

Total: 12 findings closed, ~86 new tests passing.

### Final state: Cluster E closed (post-F-cluster microtasks)

After 23+ Cluster E commits (4 first-batch + 19 microtasks), all
engine-rule anti-pattern violations are resolved. The lint
detection of `unwrap()`/`as`/`serde_json::Value`/`HashMap<String, _>`
in domain code is clean.

Total Cluster E impact: ~150+ sites fixed, lint anti-pattern
violations dropped from ~28 active files (multi-violation) to 0.

Note: The microtask pattern (one file per agent, strict scope)
was the key learning — multi-file agents aborted at the duration
limit, microtasks completed in <10 minutes wall-clock each.

After Cluster F work (6 commits: `1952864`, `b8dc5e7`, `d414782`, `dd429ed`, `eba1ae3`, `e700c67`):

### Completed (6 of 6 sub-clusters) — CLUSTER F COMPLETE

| Crate | Findings closed | Tests added |
|---|---|---|
| `educore-notify` | ADAPT-NOT-005, 007, 010, 012 (DLQ, retry, failover, rate limit) | 17 |
| `educore-files` | ADAPT-FILE-003 (storage quota + tenant guard) | 6 |
| `educore-payment` | ADAPT-PAY-005, 008 (webhook signature + signing key) | 6 |
| `educore-integrations` | ADAPT-INT-005, 007, 009 (retry cap, signing-key rotation, replay protection) | 27 |
| `educore-storage` (port trait) | PORT-STORE-002, PORT-STORE-013 (TenantContext + atomic audit) | ~30 (postgres/mysql/sqlite/surrealdb/testkit) |

Total: 12 findings closed, ~86 new tests passing.

### Isolated-scope pattern (refined)

After the first Cluster F attempt aborted due to overlapping changes across 4 parallel agents, the retry used this refined pattern in agent prompts:

> **STRICT SCOPE:** Only ADD a new file `<path>`. Do NOT modify any other file. Tests go in `<test_path>`.

Result: 3 of 4 isolated-scope agents completed cleanly; 1 needed a small test-fix commit by the lead. The final storage-transaction PR (PORT-STORE-002/013) used the **single-agent sequential** pattern with 250 turns / 250k tokens.

**Recommendation for future cross-cutting work:** when the work would touch shared infrastructure (port traits, umbrella crates, testkit), use ONE agent with a high turn budget (200+) doing sequential work. Reserve parallel agents for isolated file additions.

---

## O. Checkpoint tags

| Tag | Commit | Status |
|---|---|---|
| `remediation-checkpoint-1` | `4d403d1` (Migration fix) | Cluster A, B, D done; ~1,416 lib tests pass |
| `remediation-checkpoint-2` | `eba1ae3` (Cluster F payment test fix) | Cluster A, B, D, F (partial) done; ~1,425 lib tests pass |
| `remediation-checkpoint-3` | `e700c67` (Cluster F storage transaction) | Cluster A, B, D, F (complete) done; ~1,500 lib tests pass |
| `remediation-checkpoint-4` | `5a6e37c` (Cluster E final mop-up) | Cluster A, B, D, E, F done; 0 anti-pattern violations; ~1,440 lib tests pass |


