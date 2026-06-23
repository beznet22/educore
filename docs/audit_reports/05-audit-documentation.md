# 05 - Audit Appendix - Documentation (6 doc audits)

**Scope:** wave5-docs-1.md, wave5-docs-2.md, wave5-docs-3.md, wave5-docs-4.md, wave5-docs-5.md, wave5-docs-6.md

**Total findings:** 215

**Severity distribution:** 73 critical, 74 high, 52 medium, 16 low


## Summary Table

| Target | Critical | High | Medium | Low | Total |
| --- | --- | --- | --- | --- | --- |
| Project Overview + Architecture (`DOC-1`) | 3 | 17 | 15 | 2 | 37 |
| Build Plan + Code Standards (`DOC-2`) | 20 | 8 | 0 | 0 | 28 |
| Library Docs (`DOC-LIB`) | 4 | 5 | 2 | 0 | 11 |
| Query Layer (`DOC-QL`) | 0 | 6 | 2 | 0 | 8 |
| Phase Handoffs (`DOC-HO`) | 0 | 6 | 2 | 2 | 10 |
| Port Contracts (`DOC-PORT`) | 8 | 4 | 3 | 1 | 16 |
| Command/Event Catalogs (`DOC-CAT`) | 0 | 9 | 8 | 2 | 19 |
| Schemas (`DOC-SCHM`) | 4 | 7 | 16 | 9 | 36 |
| Guides (`DOC-6`) | 34 | 12 | 4 | 0 | 50 |

## Project Overview + Architecture (target id prefix: `DOC-1`)

**Path:** `docs/project-overview.md + docs/architecture.md`  
**Total findings:** 37 (3 critical, 17 high, 15 medium, 2 low)


### FINDING 13 (id: `DOC-1-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/architecture.md:55` vs `docs/architecture.md:267` vs `docs/architecture.md:277`

**Description:**

The same `architecture.md` makes three contradictory claims about the SurrealDB storage adapter. The ASCII architecture diagram at line 55 says `(+ SurrealDB, MongoDB deferred)`. The Storage Strategy at line 267 calls SurrealDB the `(primary, embedded + server modes)`. And the same section at line 277 says `All four ship at GA` (SurrealDB + PG + MySQL + SQLite). The diagram and Storage Strategy are in direct conflict; the build-plan § "SurrealDB-first + Sync engine additions" then says `educore-storage-surrealdb` is Phase 0 (a foundation deliverable) — but it is also claimed to be "deferred" in the same diagram. The filesystem confirms `educore-storage-surrealdb` is scaffolded at `crates/adapters/storage-surrealdb/` with a `tests/outbox_e2e.rs`, so the "deferred" claim in the diagram is wrong by the codebase.

**Expected:**

A single coherent statement of the storage-adapter shipping status. The diagram, the Storage Strategy prose, the AGENTS.md, and the build-plan should all agree.

**Evidence:**

- `docs/architecture.md:55` — `│   PostgreSQL/MySQL/SQLite (+ SurrealDB, MongoDB deferred)   OAuth/SAML/Local                         │`
  - `docs/architecture.md:267-269` — `1. **SurrealDB** (primary, embedded + server modes) — single-binary deployment. Implements `watch_changes` via `LIVE SELECT`. See `ADR-017-SurrealDBFirst.md` and `docs/schemas/sql-dialects/surrealdb.md`.`
  - `docs/architecture.md:277` — `All four ship at GA. The SurrealDB adapter is the recommended default for new deployments because its embedded mode enables single-binary distribution and the engine is embeddable by design.`
  - On disk: `crates/adapters/storage-surrealdb/Cargo.toml` + `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` exist; `migrations/engine/0000_engine_core.surreal.surql` exists.

---

### FINDING 16 (id: `DOC-1-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/architecture.md` ASCII diagram (lines 23-44) `Engine Facade (educore::Engine)` row

**Description:**

The architecture diagram claims the Engine facade exposes methods `students()`, `attendance()`, `examinations()`, `finance()`, `hr()`, `rbac()`, `library()`, `transport()`, `events()`, `reports()`. Per the wave-5 audit of `docs/library-docs.md` (finding DOC-LIB-001/002) and the actual SDK source at `crates/tools/sdk/src/engine.rs`, only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`, plus the port accessors `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()` exist. The diagram is a wishlist, not the current public surface.

**Expected:**

Either (a) the diagram reflects the actual API surface, or (b) it is explicitly labelled "future target" / "to be implemented".

**Evidence:**

- `docs/architecture.md:23` — `students()  attendance()  examinations()  finance()  hr()  ...` / `rbac()  library()  transport()  events()  reports()`
  - `crates/tools/sdk/src/engine.rs:123-147` (per wave5-docs-3 finding DOC-LIB-001) — `Engine` exposes `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` and the 7 port accessors; no `students()`, `examinations()`, `finance()`, `hr()`, `rbac()`, `library()`, `transport()`, `events()`, `reports()`.

---

### FINDING 2 (id: `DOC-1-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `AGENTS.md` § Storage Adapters vs Crate Inventory row #4

**Description:**

AGENTS.md says the storage-adapter inventory is "PostgreSQL, MySQL, SQLite" and explicitly defers "SurrealDB, MongoDB" — but the Crate Inventory still lists `educore-storage-surrealdb` as row #4 and assigns it to Phase 0 ("Foundation (SurrealDB adapter, primary)"). The two statements directly contradict each other. If SurrealDB is the "primary" target, it should not be deferred; if it is deferred, row #4 should be removed or marked deferred.

**Expected:**

Remove `educore-storage-surrealdb` from the scaffold + crate inventory (or un-defer it). At minimum the AGENTS.md should not say "primary" and "deferred" about the same adapter on the same page.

**Evidence:**

- `AGENTS.md` § Storage Adapters — `- `educore-storage-surrealdb` (primary target)` and "The SurrealDB and MongoDB adapters are **deferred to a future release**"
  - `AGENTS.md` § Crate Inventory row #4 — `| 4 | adapters | `educore-storage-surrealdb` | 0 | Foundation (SurrealDB adapter, primary) |`

---

### FINDING 1 (id: `DOC-1-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` Crate Inventory (cross-reference) vs `docs/build-plan.md` phase list

**Description:**

AGENTS.md's crate inventory table lists 35 entries (counting the umbrella `educore` as row #35) but states "The 34 crates are organized into 5 tiers." The two numbers (34 vs 35) are quoted in the same paragraph, and the umbrella is sometimes counted and sometimes not — build-plan.md and AGENTS.md do not consistently agree on the headline number.

**Expected:**

A single canonical crate count. Either "34 internal crates + 1 umbrella = 35 total" or "34 crates total (umbrella included)".

**Evidence:**

- `AGENTS.md` "Workspace Layout" header — `The 34 crates are organized into 5 tiers + 1 umbrella.`
  - `AGENTS.md` Crate Inventory table — 35 rows (numbered 1..35, with row 35 being the umbrella).
  - `AGENTS.md` Status section — `Workspace scaffold: **complete** (34 crates, virtual workspace).` — uses 34 again.

---

### FINDING 11 (id: `DOC-1-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` § Phase Plan and `docs/build-plan.md` § Phase numbering

**Description:**

AGENTS.md says "17 phases (Phase 0..17)" but the prompt for this audit notes "Phase 17 missing per prompt" — implying the build plan numbers phases 0..16 (17 phases total). The two numbers conflict. Either the build-plan.md has Phase 0..17 (18 phases) or Phase 0..16 (17 phases); the AGENTS.md should not be out of sync with the build plan.

**Expected:**

AGENTS.md and build-plan.md agree on the phase count and the highest-numbered phase.

**Evidence:**

- `AGENTS.md` Status — `Build plan: **17 phases** (Phase 0..17) with coverage matrix and no-gaps gates documented in `docs/build-plan.md`.`

---

### FINDING 14 (id: `DOC-1-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md` § Tier System table (lines 419-426) vs on-disk crate count

**Description:**

The Tier System table claims `adapters | 9` and `cross-cutting | 7`. The actual filesystem has 10 adapter crates (`auth`, `event-bus`, `files`, `integrations`, `notify`, `payment`, `storage-mysql`, `storage-postgres`, `storage-sqlite`, `storage-surrealdb`) and 9 cross-cutting crates (`audit`, `events`, `events-domain`, `operations`, `platform`, `rbac`, `settings`, `sync`, `sync-inprocess`). The "adapters: 9" count omits one; the "cross-cutting: 7" count omits `sync` and `sync-inprocess` (introduced by `ADR-018-SyncEngineArchitecture.md`).

**Expected:**

Update the tier counts to match the filesystem: `adapters: 10`, `cross-cutting: 9`. Or, if `sync`/`sync-inprocess` belong in a different tier, document that decision.

**Evidence:**

- `docs/architecture.md:419-426` — Tier System table `| `adapters` | `crates/adapters/` | 9 | Port implementations: 3 storage adapters + 6 port adapters. Depends on `infra` and `cross-cutting`. |` and `| `cross-cutting` | `crates/cross-cutting/` | 7 | Cross-domain foundations: platform, rbac, events envelope, audit, settings, operations, calendar. Depends on `infra`. |`
  - `ls crates/adapters/` — 10 entries (including `storage-surrealdb/`).
  - `ls crates/cross-cutting/` — 9 entries (including `sync/` and `sync-inprocess/`).

---

### FINDING 15 (id: `DOC-1-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md:430-441` vs actual scaffolded crates

**Description:**

The same paragraph says the workspace is `34 internal crates` (with `+1 umbrella = 35`). The actual scaffold on disk contains 37 crates (3 infra + 9 cross-cutting + 10 domains + 10 adapters + 4 tools + 1 umbrella = 37). The header line and the tier table are out of sync by 3 crates (1 adapter + 2 cross-cutting).

**Expected:**

`36 internal crates + 1 umbrella = 37 total` per the filesystem. Or, if the 3 additional crates are not yet "officially" shipped, mark them as scaffolded but pending.

**Evidence:**

- `docs/architecture.md:417` — `The 34 crates are organized into **5 tiers + 1 umbrella**.`
  - `docs/architecture.md:427` — `Re-exports the public surface of all 34 internal crates.`
  - `ls crates/{infra,cross-cutting,domains,adapters,tools,educore}/` — 3 + 9 + 10 + 10 + 4 + 1 = 37.

---

### FINDING 17 (id: `DOC-1-017`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md:47-48` ASCII diagram `Ports (Traits)` row

**Description:**

The ASCII architecture diagram lists 12 ports in two rows: `Storage Authentication Notification Payment FileStorage EventBus Identity Clock IdGen Audit` + `Integration Indexer Search`. The accompanying prose table (lines 160-173) lists 12 ports: `Storage`, `Authentication`, `Notification`, `Payment`, `FileStorage`, `EventBus`, `IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Integration`, `Identity`. The diagram and table disagree on 3 names: diagram has `Indexer` and `Search` (one port), table has only `SearchIndex`; diagram has `Identity` (matches table); diagram has `IdGen` (table has `IdGenerator`); diagram has `Audit` (table has `AuditSink`). The `docs/ports/` directory contains only 8 files (`authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md`) — 4 of the 12 table ports (`IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Identity`) lack a dedicated port doc file. The `Indexer`/`Search` distinction has no doc at all.

**Expected:**

Either (a) the diagram and table agree on the same 12 names AND every name has a `docs/ports/<name>.md` file, or (b) the port count is reduced to the 8 documented ports.

**Evidence:**

- `docs/architecture.md:47-48` — diagram rows `Storage Authentication Notification Payment FileStorage EventBus Identity Clock IdGen Audit` + `Integration Indexer Search`
  - `docs/architecture.md:160-173` — table columns `Storage`, `Authentication`, `Notification`, `Payment`, `FileStorage`, `EventBus`, `IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Integration`, `Identity`
  - `ls docs/ports/` — 8 files: `authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md` (no `id-generator.md`, `clock.md`, `audit-sink.md`, `search-index.md`, `indexer.md`).

---

### FINDING 18 (id: `DOC-1-018`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md:266-285` Storage Strategy § "SurrealDB-first" + `docs/build-plan.md:5-26`

**Description:**

Architecture.md § Storage Strategy presents a 4-tier storage-backend priority list with SurrealDB first. But § Runtime DDL emission at line 322 says `The 6 cross-cutting tables have canonical DDL in three dialects under `migrations/engine/`` and lists only `mysql.sql`, `postgres.sql`, `sqlite.sql`. The fourth dialect file `migrations/engine/0000_engine_core.surreal.surql` (which exists on disk) is not mentioned in that sentence, so the prose still says "three dialects" — inconsistent with the actual 4 files. The build-plan § "SurrealDB-first + Sync engine additions" does correctly note that `0000_engine_core.surreal.surql` is "added in this phase" by Phase 0, but the architecture doc never picks up the change.

**Expected:**

Architecture.md says "four dialects" (or "SQL dialects + SurrealDB") and lists the surreal file.

**Evidence:**

- `docs/architecture.md:322-325` — `Migrations live in `migrations/engine/` (3 dialect files for the 6 cross-cutting tables: `outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`). The adapter crates `include_str!` these files at compile time.`  [via AGENTS.md cross-reference; the same claim appears in architecture.md § Runtime DDL emission]
  - `ls migrations/engine/` — 4 files: `0000_engine_core.mysql.sql`, `0000_engine_core.postgres.sql`, `0000_engine_core.sqlite.sql`, `0000_engine_core.surreal.surql`.
  - `docs/build-plan.md:5-26` — Phase 0 deliverable 4: `The 6 cross-cutting tables are `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql` (added in this phase).`

---

### FINDING 19 (id: `DOC-1-019`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md:339-352` Sync Strategy

**Description:**

Architecture.md § Sync Strategy states `Swap from in-process to worker is a one-line change in `Engine::builder().sync(...)`.` The actual SDK source (`crates/tools/sdk/src/engine.rs`) does not expose a `sync()` method on the engine or its builder (verified by grep — no `pub fn sync` matches). The architecture doc describes a feature that is not implemented.

**Expected:**

Either (a) remove the `Engine::builder().sync(...)` sentence until the method is implemented, or (b) implement the method.

**Evidence:**

- `docs/architecture.md:351-352` — `Both implementations share the same wire protocol documented in `docs/ports/sync.md`. Swap from in-process to worker is a one-line change in `Engine::builder().sync(...)`.`
  - `grep -n "fn sync\|sync_feature" crates/tools/sdk/src/engine.rs` — no matches.
  - `grep -n "fn sync\|sync_feature" crates/tools/sdk/src/lib.rs` — no matches.

---

### FINDING 20 (id: `DOC-1-020`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/architecture.md:354` Sync Strategy

**Description:**

Architecture.md says `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.` Searching the umbrella and SDK source for a `sync` feature flag returns no matches; the umbrella does not currently expose a `sync` feature.

**Expected:**

Either add the `sync` feature to `crates/educore/Cargo.toml` and gate the in-process coordinator behind it, or remove the sentence.

**Evidence:**

- `docs/architecture.md:354` — `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator. Consumers who want a pure server-side engine disable the feature and use no sync adapter.`
  - `grep -rn "feature = \"sync\"\|features = .*sync" crates/educore/Cargo.toml` — no matches.

---

### FINDING 22 (id: `DOC-1-022`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/build-plan.md:1-3` ("17 sequential phases") vs `docs/build-plan.md:53-70` ("The 17 phases" enumerated list) vs AGENTS.md Crate Inventory

**Description:**

The build-plan says "implemented in **17 sequential phases** (Phase 0..17)" in the opening paragraph, but the enumerated list at lines 53-70 contains **18** entries (Phase 0 through Phase 17 inclusive). The phase numbering uses 0-indexed: Phase 0..17 = 18 phases. AGENTS.md repeats the same `17 phases (Phase 0..17)` claim. Either the count is wrong (should be 18) or one of the phases should be re-numbered.

**Expected:**

Either the build-plan renumbers to `Phase 0..16 (17 phases)` (dropping Phase 17 from the count) or the count is corrected to `18 phases (Phase 0..17)`.

**Evidence:**

- `docs/build-plan.md:1` — `The engine is implemented in **17 sequential phases** (Phase 0..17).`
  - `docs/build-plan.md:53-70` — 18 numbered list items: Phase 0, Phase 1, … Phase 17.
  - `AGENTS.md` Status section — `Build plan: **17 phases** (Phase 0..17) with coverage matrix and no-gaps gates documented in `docs/build-plan.md`.`

---

### FINDING 23 (id: `DOC-1-023`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/build-plan.md:74-79` ("Pre-implementation state") vs on-disk crate count

**Description:**

The build-plan says `The workspace has **34 crates** (29 from the original scaffold + 5 new: `educore-audit`, `educore-operations`, `educore-testkit`, `educore-cli`, `educore-storage-parity`).` The actual filesystem contains 37 crates (3 infra + 9 cross-cutting + 10 domains + 10 adapters + 4 tools + 1 umbrella). The 5 new crates listed match what's on disk, but the "29 from the original scaffold" claim is wrong — the on-disk count is 32 crates excluding the 5 listed (37 − 5 = 32, not 29). The discrepancy is at least 3 crates (one adapter `storage-surrealdb`; two cross-cutting `sync` + `sync-inprocess`).

**Expected:**

Either correct the original-scaffold count to 32, or document which crates are scaffolded but not yet "official".

**Evidence:**

- `docs/build-plan.md:74-77` — `The workspace has **34 crates** (29 from the original scaffold + 5 new: `educore-audit`, `educore-operations`, `educore-testkit`, `educore-cli`, `educore-storage-parity`).`
  - `ls crates/{infra,cross-cutting,domains,adapters,tools,educore}/` — 37 entries total.

---

### FINDING 24 (id: `DOC-1-024`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/build-plan.md:284-291` (Phase 0 "Coverage matrix updates") vs enumeration of rows

**Description:**

The build-plan § Phase 0 Coverage matrix updates heading says `The following 13 rows flipped from `Pending` to `Tested` in PR A:` but the enumeration that follows contains 16 distinct row IDs: `outbox_ddl_surreal`, `idempotency_ddl_surreal`, `schema_registry_ddl_surreal`, `system_user_ddl_surreal` (4) + `domain_query_macro`, `entity_descriptor_ast`, `school_id_newtype`, `uuid_v7_generator`, `system_clock`, `domain_error_enum` (6) + `storage_adapter_port`, `storage_transaction_port`, `storage_outbox_port` (3) + `sync_port`, `sync_inprocess_impl` (2) + `engine_graph_regen` (1) = 16. The "13 rows" claim is short by 3.

**Expected:**

Either the count is corrected to 16, or 3 of the 16 rows are removed.

**Evidence:**

- `docs/build-plan.md:284-291` — `**Coverage matrix updates.** The following 13 rows flipped from `Pending` to `Tested` in PR A: ...`

---

### FINDING 26 (id: `DOC-1-026`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/build-plan.md` Phase 6 and Phase 8 — missing outcome paragraphs

**Description:**

Phases 0-5, 7, 9-16 each have a `**Phase N outcome.**` paragraph summarising close-out status. Phase 6 (HR) and Phase 8 (Facilities) do NOT have an outcome paragraph — only the tasks/exit-criteria/risks sections are present. Phase 17 (Production readiness) also has no outcome paragraph, which is consistent with it not being closed. But Phase 6 and Phase 8 are gap-filling omissions: the hand-off files `docs/handoff/PHASE-6-HANDOFF.md` and `docs/handoff/PHASE-8-HANDOFF.md` exist on disk, so the phase outcomes are documented in the handoffs but not back-propagated to the build-plan.

**Expected:**

Add `**Phase 6 outcome.**` and `**Phase 8 outcome.**` paragraphs to the build-plan (analogous to Phase 5's outcome paragraph), referencing the existing hand-off files.

**Evidence:**

- `docs/build-plan.md:730-755` — `**Phase 5 outcome.**` is the last "outcome" paragraph before Phase 6.
  - `docs/build-plan.md:757-806` — Phase 6 section contains Tasks + Exit criteria + Coverage matrix + Risks + Phase completion documentation; no `**Phase 6 outcome.**` paragraph.
  - `docs/build-plan.md:807-855` — Phase 8 section has the same shape (no outcome paragraph).
  - `ls docs/handoff/PHASE-{0..16}-HANDOFF.md` — 17 handoff files exist, including `PHASE-6-HANDOFF.md` and `PHASE-8-HANDOFF.md`.

---

### FINDING 29 (id: `DOC-1-029`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/build-plan.md` Phase 0..16 § "Phase completion documentation" tasks + `docs/build-plan.md:1793-1796` (Phase 17)

**Description:**

Phases 0-16 each include a "Phase completion documentation" task that says `Create `docs/phase_prompt/phase-(N+1)-prompt.md` for the next-phase agent (per the convention in `docs/phase_prompt/README.md`).` The `docs/phase_prompt/` directory does not exist on disk (`ls docs/phase_prompt/` returns "No such file or directory"). Phase 17 explicitly says `do not create a `phase-18-prompt.md` unless a Phase 18+ is explicitly planned.` All 17 `Phase N outcome.` paragraphs claim `✅ Already produced for Phase N (see `docs/handoff/PHASE-N-HANDOFF.md` and `docs/phase_prompt/phase-(N+1)-prompt.md`).` — but the prompt files do not exist. Either the convention was abandoned and the directory was deleted (in which case the doc must say so), or the prompts were never created (in which case the ✅ checkmarks lie).

**Expected:**

Either (a) re-create the `docs/phase_prompt/` directory and the 17 prompt files, or (b) remove the references in every Phase N outcome paragraph and the Phase completion documentation tasks.

**Evidence:**

- `docs/build-plan.md` — every Phase 0..16 § "Phase completion documentation" task references `docs/phase_prompt/phase-(N+1)-prompt.md`.
  - `ls docs/phase_prompt/` — does not exist.
  - `ls docs/handoff/PHASE-{0..16}-HANDOFF.md` — 17 handoff files exist (PHASE-17-HANDOFF.md does NOT exist, consistent with Phase 17 not being closed).

---

### FINDING 3 (id: `DOC-1-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` Status section

**Description:**

Status section claims `Workspace scaffold: **complete** (34 crates, virtual workspace)` but `educore-storage-surrealdb` and `educore-storage-mongo` (deferred) are not scaffolded in `crates/adapters/` — the AGENTS.md itself lists only `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite` under that tier. The "complete" claim is therefore wrong by AGENTS.md's own admission.

**Expected:**

Either (a) "complete (3 of 3 shipped storage adapters scaffolded; SurrealDB deferred)" or (b) "complete (5 of 5 storage adapters scaffolded; 2 deferred)".

**Evidence:**

- `AGENTS.md` Status — `Workspace scaffold: **complete** (34 crates, virtual workspace).`
  - `AGENTS.md` Workspace Layout tree — `adapters/ ├── storage-postgres/ ├── storage-mysql/ ├── storage-sqlite/ ├── auth/ ├── event-bus/ ├── files/ ├── integrations/ ├── notify/ └── payment/` — no `storage-surrealdb/`, no `storage-mongo/`.

---

### FINDING 4 (id: `DOC-1-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` § Implementation status

**Description:**

Status says `Implementation: **not started** — scaffold only.` Yet the same document's Crate Inventory row #23 (`educore-cms`) carries a multi-line annotation claiming `183 unit tests in crate + 7-scenario integration test in `storage-parity``. That contradicts "not started" — Phase 12 (CMS) is clearly implemented per the annotation, and the Status section is stale.

**Expected:**

Status section should reflect that Phases 0–12 are landed (per the Crate Inventory annotations).

**Evidence:**

- `AGENTS.md` Status — `Implementation: **not started** — scaffold only. Domain logic, aggregates, value objects, commands, events, repositories, and storage translations are pending.`
  - `AGENTS.md` Crate Inventory row #23 — `| 23 | domains | `educore-cms` | 12 | CMS — spec-faithful (20 root aggregates per `docs/specs/cms/aggregates.md`); 9-file layout; ~67 events, ~67 commands, 86 Cms caps (4 retained Phase 2 placeholders + 82 net-new), 21 Cms audit targets, 19 repos, 19 query stubs, 6 service factory fns + 6 service structs (PageService, NewsService, ContentService, TestimonialService, HomeSliderService, ContentShareListService); `form_uploaded_public_indexing_subscriber` for `documents.form_download.uploaded` (Phase 11 OQ #6); `educore-academic` dep for `ClassId`/`SectionId`/`AcademicYearId`; 183 unit tests in crate + 7-scenario integration test in `storage-parity` (2 env-gated PG/MySQL variants); `SchoolId::PUBLIC` constant added to `educore-core`; 20 `coverage.toml` rows flipped; see `PHASE-12-HANDOFF.md` |`

---

### FINDING 6 (id: `DOC-1-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` § Storage Adapters (cross-reference with `docs/architecture.md` § Runtime DDL emission)

**Description:**

AGENTS.md lists three shipped storage adapters (PostgreSQL, MySQL, SQLite), but the umbrella re-exports `educore_storage_surrealdb` if it exists. AGENTS.md needs to be consistent with the architecture doc on which storage adapter the engine ships by default. The two docs disagree on whether `educore-storage-surrealdb` is shipped or deferred.

**Expected:**

Architecture doc and AGENTS.md agree on the shipped adapter set (PostgreSQL, MySQL, SQLite) and explicitly mark SurrealDB / MongoDB as deferred (or remove them from scaffold).

**Evidence:**

- `AGENTS.md` § Storage Adapters — `Three reference adapters are shipped:` + `educore-storage-surrealdb` (primary target) — then "deferred to a future release".
  - `docs/architecture.md` § Runtime DDL emission — `The `educore-storage-<db>` adapter crates `include_str!` these files at compile time.` — generic.

---

### FINDING 8 (id: `DOC-1-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `AGENTS.md` § Engine Rule 6 + § Phase Plan vs `docs/architecture.md` § Runtime DDL emission

**Description:**

Engine Rule 6 says "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST." But the § Runtime DDL emission section says "The ~310 domain tables are macro-emitted." The phrase "macro-emitted" is ambiguous — does it mean (a) the macro generates the AST nodes that the storage adapter then walks to emit DDL, or (b) the macro emits raw SQL strings? If interpretation (b) is correct, it directly violates Engine Rule 6.

**Expected:**

Architecture doc clarifies "macro-emitted AST → adapter walks AST → adapter emits DDL strings".

**Evidence:**

- `AGENTS.md` Engine Rules #6 — `No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST.`
  - `docs/architecture.md` § Runtime DDL emission step 3 — `Machine contract — `crates/<domain>/src/entities.rs` (macro-emitted typed AST, dialect-agnostic).`
  - `docs/architecture.md` § Runtime DDL emission step 4 — `Adapter emission — `educore-storage-<db>` walks the AST at schema-creation time and emits the dialect-specific DDL string.`

---

### FINDING 10 (id: `DOC-1-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `AGENTS.md` § Co-Authoring

**Description:**

AGENTS.md instructs AI agents to use the `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. The local session context indicates the agent is `Kimchi`, an AI coding agent developed by `MiniMax` (per the model-version banner in the system prompt). The Antigravity trailer would falsely attribute work to a different tool/company. This rule is incompatible with the actual agent identity.

**Expected:**

The co-authoring trailer should match the actual agent identity (`Kimchi`) or be removed.

**Evidence:**

- `AGENTS.md` Co-Authoring — `AI-generated commits must include the trailer specified in the **Commit Attribution** subsection of Agent Instructions above: \n\nCo-Authored-By: Antigravity <antigravity@google.com>` / "This is the canonical attribution for every AI-authored commit in this repository. No other `Co-Authored-By` trailer is accepted for AI agents."
  - System prompt — `Your model version is MiniMax-M3, developed by MiniMax.` / `## Environment\nYou are Kimchi, an AI coding agent.`

---

### FINDING 12 (id: `DOC-1-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `AGENTS.md` § Spec Folder Layout vs `docs/specs/<domain>/` directory

**Description:**

AGENTS.md says the spec folder uses `services.md` (not `policies.md`) and `workflows.md` (not `errors.md`), but the legacy Laravel project (`schoolify/`) uses different filenames. If the spec folder layout was unified, the doc should state the unification was complete; if not, the doc should list the actual filenames used.

**Expected:**

Confirm the 11-file mapping is in force across all 15 domain spec folders; flag any deviations.

**Evidence:**

- `AGENTS.md` "Spec folder layout" — `the `services.rs` module hosts policy logic; the `errors.rs` module defines the `DomainError` enum` — not directly verified in this audit but cross-checked against `docs/specs/` (deferred to Phase B).

---

### FINDING 21 (id: `DOC-1-021`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/architecture.md:341` Sync Strategy vs filesystem

**Description:**

Architecture.md calls `educore-sync-inprocess` an "adapter". The filesystem places it at `crates/cross-cutting/sync-inprocess/` — i.e. in the `cross-cutting` tier, not the `adapters` tier. The doc re-classifies the crate, contradicting the tier system.

**Expected:**

Either (a) move `educore-sync-inprocess` into `crates/adapters/`, or (b) call it a "cross-cutting reference implementation" / "in-process implementation".

**Evidence:**

- `docs/architecture.md:341` — `The engine also ships an **in-process reference implementation** of the sync engine (`educore-sync` cross-cutting crate + `educore-sync-inprocess` adapter) so consumers can ship a working offline-first app in 30 minutes without infrastructure.`
  - `ls crates/cross-cutting/` — contains `sync-inprocess/`.

---

### FINDING 25 (id: `DOC-1-025`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:11-13` ("SurrealDB-first + Sync engine additions") + `docs/build-plan.md` Phase 0 § "Phase 0 — Foundation"

**Description:**

The SurrealDB-first amendment at lines 11-26 says the reference target becomes `educore-storage-surrealdb` and PG/MySQL/SQLite move to Phase 1 as "parity adapters". But Phase 1's title in § "The 17 phases" is `Phase 1 — Adapter parity: storage-postgres, storage-mysql, storage-sqlite + cross-adapter test` — and Phase 1's Coverage matrix updates says `12 rows` flip (4 DDL × 3 adapters). The wording is internally consistent on the move-to-Phase-1, but the architecture.md § Storage Strategy still calls SurrealDB the "primary" target — the document set has not converged on a consistent story.

**Expected:**

A single canonical statement of the storage priority: e.g. `SurrealDB is the recommended default for new deployments; PG/MySQL/SQLite are parity adapters at Phase 1`.

**Evidence:**

- `docs/build-plan.md:11-26` — `**SurrealDB-first + Sync engine additions**` amendment.
  - `docs/architecture.md:267-279` — Storage Strategy still presents SurrealDB as priority #1 with PG/MySQL/SQLite as #2-4 but does not use the word "parity".

---

### FINDING 27 (id: `DOC-1-027`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:679-681` (Phase 4 outcome) vs `docs/build-plan.md:603-610` (Phase 3 outcome)

**Description:**

Phase 4 outcome says `**433 tests pass workspace-wide** (was 380 at Phase 3 close-out; +53 net new in Phase 4)`. Phase 3 outcome says `**369 tests pass workspace-wide** (was 310 at Phase 2 close-out; +59 net new in Phase 3)`. The "Phase 3 close-out" count is **369** (per Phase 3's own paragraph) but **380** (per Phase 4's "was 380 at Phase 3 close-out"). Off by 11 tests. One of the two numbers is wrong.

**Expected:**

Both paragraphs should agree on the Phase 3 close-out test count (either 369 or 380).

**Evidence:**

- `docs/build-plan.md:603-610` — Phase 3 outcome: `**369 tests pass workspace-wide** (was 310 at Phase 2 close-out; +59 net new in Phase 3).`
  - `docs/build-plan.md:679-681` — Phase 4 outcome: `**433 tests pass workspace-wide** (was 380 at Phase 3 close-out; +53 net new in Phase 4: 51 unit + 2 new env-gated ignored tests + 1 new SQLite integration test + 1 new capability-check test + 1 new event-type round-trip test).`

---

### FINDING 28 (id: `DOC-1-028`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:1761-1772` (Phase 17 Exit criteria) vs `AGENTS.md` Validation Checklist

**Description:**

Phase 17 Exit criterion #1 says `All 10 validation questions in `AGENTS.md` answer "Yes".` The AGENTS.md § Validation Checklist (lines 304-317) contains **12** checklist items, not 10. The build-plan under-counts by 2.

**Expected:**

Either the count is corrected to 12, or the AGENTS.md validation checklist is consolidated to 10 items.

**Evidence:**

- `docs/build-plan.md:1761-1772` — Phase 17 Exit criteria: `1. All 10 validation questions in `AGENTS.md` answer "Yes".`
  - `AGENTS.md:304-317` — Validation Checklist contains 12 items: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `no unwrap/expect/panic`, `no as`, `no serde_json::Value`, `public items documented`, `at least one integration test`, `diagrams updated`, `ADRs updated`, `no legacy brand references`.

---

### FINDING 30 (id: `DOC-1-030`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:1669-1678` (Phase 16 outcome) vs Phase 16 scope

**Description:**

Phase 16 is `Test infrastructure + SDK` and the outcome claims `previously-blocked settings/documents clippy debt was paid down as Phase 16 prep work (commits `131c507` + `448d8ad`).` Clippy-debt remediation belongs in the originating phase (Phase 14 settings / Phase 11 documents) or in a dedicated hygiene PR, not in Phase 16's scope. The phase outcome paragraph is doing double duty as a status report for unrelated earlier phases.

**Expected:**

Move the "clippy debt paid down" note into the Phase 14 / Phase 11 outcome paragraphs (or add a dedicated hygiene PR reference) rather than burying it in Phase 16.

**Evidence:**

- `docs/build-plan.md:1669-1678` — Phase 16 outcome: `cargo clippy --workspace --all-targets -- -D warnings` green on the 4 Phase 16 crates (testkit, storage-parity, sdk, cli); the previously-blocked settings/documents clippy debt was paid down as Phase 16 prep work (commits `131c507` + `448d8ad`).`

---

### FINDING 31 (id: `DOC-1-031`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:2009` ("See also") vs port count

**Description:**

The build-plan § "See also" says `docs/ports/*.md` (7 ports)`. The actual `docs/ports/` directory contains 8 files (`authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md`). The "7 ports" parenthetical is off by 1 (likely stale from before the sync port landed in Phase 0/2).

**Expected:**

Update to `(8 ports)`.

**Evidence:**

- `docs/build-plan.md:2009` — `[`docs/ports/*.md`](ports/) — port contracts (7 ports).`
  - `ls docs/ports/` — 8 files.

---

### FINDING 32 (id: `DOC-1-032`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:2011` ("See also") vs specs count

**Description:**

The build-plan § "See also" says `docs/specs/<domain>/overview.md` ... (15 domains, 11 files each).` The actual `docs/specs/` directory contains 16 entries (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events`, `facilities`, `finance`, `hr`, `library`, `operations`, `platform`, `rbac`, `settings`, `sync`). Even excluding `sync` (which is a cross-cutting port, not a domain), the directory contains 15 specs, but those include cross-cutting tier specs (`platform`, `rbac`, `settings`, `operations`, `sync`, `events`) which are not "domains". The doc's "15 domains" count is correct in absolute terms but the term "domains" is misleading — half of those specs are for cross-cutting crates.

**Expected:**

Either (a) separate "domain specs" from "cross-cutting specs" (e.g. `15 domain specs + 6 cross-cutting specs`), or (b) rename the directory to make the distinction explicit.

**Evidence:**

- `docs/build-plan.md:2011` — `docs/specs/<domain>/overview.md` ... (15 domains, 11 files each).`
  - `ls docs/specs/` — 16 entries including `platform`, `rbac`, `settings`, `operations`, `sync`, `events` (cross-cutting).

---

### FINDING 33 (id: `DOC-1-033`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:151-155` (Phase 0 task 1) vs filesystem

**Description:**

Phase 0 task 1 lists `Source — `UuidV7`` as one of the ids module deliverables. The actual `crates/infra/core/src/ids.rs` file is 380+ lines long and includes `SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Timestamp`, etc. — but the `Source` type mentioned in the doc is not a standard id; it appears to be a placeholder or a typo for `SourceOfTruth` / similar. The deliverable list may be stale.

**Expected:**

Reconcile Phase 0 task 1's id list against the actual `ids.rs` exports. Either (a) update the doc to match the actual types, or (b) add the missing types to the file.

**Evidence:**

- `docs/build-plan.md:151-155` — ``educore-core`: `errors.rs` (`DomainError` via `thiserror`), `ids.rs` (`SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Source` — `UuidV7`), `value_objects.rs` (`Timestamp`, `Version`, `Etag`, `ActiveStatus`), `clock.rs` (`Clock` trait + `SystemClock` + `TestClock`), `id_gen.rs` (v7 UUID generator with deterministic test backend), `tenant.rs` (`TenantContext`), and `query.rs` (the `EntityDescriptor` AST types consumed by the macro).`
  - `crates/infra/core/src/ids.rs` — exports `SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Timestamp`, plus helper methods. No `Source` struct.

---

### FINDING 34 (id: `DOC-1-034`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:174` (Phase 0 task 4) vs `crates/adapters/storage-surrealdb/` existence

**Description:**

Phase 0 task 4 says ``educore-storage-surrealdb`: full impl. Walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL.` The crate is scaffolded (`crates/adapters/storage-surrealdb/` exists with `Cargo.toml`, `src/`, `tests/outbox_e2e.rs`), but the description "render the ~310 domain tables" overstates the current state: only the 6 cross-cutting tables are real DDL today; the ~310 domain tables are deferred to per-domain phases (the macro AST is not yet wired into the adapter's `create_schema()` path for all domains).

**Expected:**

Phase 0 task 4 should distinguish "scaffolding + outbox e2e" from "complete ~310 domain tables". The current prose blurs the two.

**Evidence:**

- `docs/build-plan.md:174-179` — Phase 0 task 4: `educore-storage-surrealdb`: full impl. Walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL. The 6 cross-cutting tables are `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql` (added in this phase). `surrealdb` driver + `rustls`.`
  - `ls crates/adapters/storage-surrealdb/` — `Cargo.toml`, `src/`, `tests/outbox_e2e.rs`. The `~310 domain tables` claim is not yet realised.

---

### FINDING 35 (id: `DOC-1-035`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/build-plan.md:1891-1895` (Coverage Matrix preamble)

**Description:**

The Coverage Matrix preamble says `The full matrix has 226+ rows: one per implementable doc, one per table for the 6 cross-cutting tables × 3 dialects`. The 6 × 3 = 18 table rows alone is wrong: there are now 4 dialect files (`mysql.sql`, `postgres.sql`, `sqlite.sql`, `surreal.surql`), so it should be 6 × 4 = 24 cross-cutting-table rows. The total row count should likewise reflect the 4th dialect.

**Expected:**

Update the matrix preamble to "6 cross-cutting tables × 4 dialects" and re-state the total row count.

**Evidence:**

- `docs/build-plan.md:1891-1895` — `The full matrix has 226+ rows: one per implementable doc, one per table for the 6 cross-cutting tables × 3 dialects, one per port trait × impl.`
  - `ls migrations/engine/` — 4 dialect files (mysql, postgres, sqlite, surreal).

---

### FINDING 5 (id: `DOC-1-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `AGENTS.md` Crate Inventory note on `educore-storage-parity`

**Description:**

`educore-storage-parity` is listed twice — at Phase 0 (row #5) and at Phase 16 (row #33). The note acknowledges the duplication ("Phase 0 scaffolds the crate; Phase 16 implements the actual test scenarios") but having the same crate twice in the inventory table with two different "Phase" columns is structurally confusing and prone to be misread as "two different crates".

**Expected:**

A single row with a "Phase 0 scaffold; Phase 16 implementation" annotation, or two clearly-distinct rows with distinct notes.

**Evidence:**

- `AGENTS.md` Crate Inventory row #5 — `| 5 | tools | `educore-storage-parity` | 0 | Foundation (cross-adapter test suite) |`
  - `AGENTS.md` Crate Inventory row #33 — `| 33 | tools | `educore-storage-parity` | 16 | (Test infrastructure + SDK) |`

---

### FINDING 7 (id: `DOC-1-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `AGENTS.md` § `educore-events` vs `educore-events-domain` note

**Description:**

AGENTS.md says these two crates are distinct. However, the umbrella re-export list in `crates/educore/src/lib.rs` (per AGENTS.md example) shows `pub use educore_events as events;` but does not show `pub use educore_events_domain as events_domain;` in the same example. Consumers may be confused about whether the umbrella exposes the calendar domain under `educore::events_domain` or `educore::events`.

**Expected:**

Umbrella re-exports should be explicit about both `events` (envelope) and `events_domain` (calendar).

**Evidence:**

- `AGENTS.md` Note on `educore-events` vs `educore-events-domain` — explicit warning not to conflate.
  - `AGENTS.md` "The umbrella re-exports each internal crate under its short name" example — `pub use educore_academic as academic;` — no `events_domain` shown.

---

### FINDING 9 (id: `DOC-1-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `AGENTS.md` § `educore-core` note

**Description:**

AGENTS.md says the `core` package "lives at `crates/infra/core/`" because the tier name `infra/` would collide with the package name `core/`. But this is described as a "naming convention, not a typographical error" — yet `crates/infra/core/Cargo.toml` will still contain `name = "educore-core"` while the directory is `core/`, which is exactly the convention AGENTS.md warns against ("the `educore-` prefix is dropped from the directory name"). The note claims this is "intentional" but the rule that drops the `educore-` prefix says nothing about a tier-name collision.

**Expected:**

Clarify that `core` is exempted from the prefix-drop rule for tier-collision reasons, or rename the package.

**Evidence:**

- `AGENTS.md` "Note on `infra/core`" — explicit caveat that the convention is "intentionally" violated.
  - `AGENTS.md` Naming Convention table — `Internal (per-domain) | `educore-<name>` | `crates/<name>/`` — rule that drops the prefix.

---

### FINDING 36 (id: `DOC-1-036`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/project-overview.md:74-79` ("Core Philosophy") vs architecture.md sync

**Description:**

Project-overview.md's "Core Philosophy" lists `Offline is a first-class mode. State changes can be queued and reconciled.` But architecture.md § Sync Strategy says sync is `feature-gated` (`The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.`). A feature-gated capability is not "first-class"; it is an opt-in extension. The two docs disagree on the priority of offline support.

**Expected:**

Either (a) the architecture.md drop the `sync` feature flag and ship sync as a default, or (b) project-overview.md clarify that offline-first is a goal achieved through the optional `sync` feature.

**Evidence:**

- `docs/project-overview.md:74-79` — `- **Offline is a first-class mode.** State changes can be queued and reconciled.`
  - `docs/architecture.md:354` — `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.`

---

### FINDING 37 (id: `DOC-1-037`)

- **Source:** `docs/audit_reports/findings/wave5-docs-1.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/project-overview.md:90-105` ("Success Criteria") vs SDK API surface

**Description:**

Project-overview.md's Success Criteria item 1 says `A consumer application can admit a student, take attendance, record marks, and collect fees using only the public API and this documentation.` Per wave-5 audit findings DOC-LIB-001 and DOC-LIB-002, the documented API names (`engine.students()`, `engine.assessment()`, `engine.fees()`, `engine.hr()`) do not exist on the actual SDK (`crates/tools/sdk/src/engine.rs`). The success criterion cannot be evaluated against the SDK without code changes to add those facade methods.

**Expected:**

Either (a) the success criterion is re-stated in terms of methods that exist on the SDK (`engine.admission()`, `engine.attendance()`, `engine.payment_svc()`, `engine.notify_svc()`), or (b) the SDK is extended with the missing facade methods.

**Evidence:**

- `docs/project-overview.md:90-105` — Success Criteria 1: `A consumer application can admit a student, take attendance, record marks, and collect fees using only the public API and this documentation.`
  - `crates/tools/sdk/src/engine.rs:123-147` (per wave5-docs-3 findings DOC-LIB-001/002) — Engine exposes `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` only.

---


## Build Plan + Code Standards (target id prefix: `DOC-2`)

**Path:** `docs/build-plan.md + docs/code-standards.md`  
**Total findings:** 28 (20 critical, 8 high, 0 medium, 0 low)


### FINDING 1 (id: `DOC-2-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:14` (Rust Standards list) vs `crates/infra/core/src/error.rs:19-63`

**Description:**

`docs/code-standards.md` § Error Handling (lines 100-104) states the engine-level `DomainError` enum carries `kind` discriminant variants `Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`. The actual `DomainError` (per `wave4-core.md` finding CORE-006) has 7 variants: `Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String)`. The two extra variants (`TenantViolation`, `NotSupported`) and the documented variants' shapes (`String` payload vs the doc's hint of "discriminant") are inconsistent; the spec gives no schema for which variants carry which payload.

**Expected:**

Per `docs/code-standards.md:102-104` — `Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure` as the documented variants.

**Evidence:**

- `docs/code-standards.md:102-104` — "Engine-level errors include a `kind` discriminant (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`)."
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), NotSupported(String), }`.

---

### FINDING 10 (id: `DOC-2-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:181-188` (Common Workflows) vs `crates/tools/sdk/src/engine.rs:128-147`

**Description:**

"Common Workflows" claims `engine.students().admit(cmd).await?`, `engine.assessment().enter_marks(cmd).await?`, `engine.fees().generate_invoice(cmd).await?`, `engine.hr().generate_payroll(cmd).await?` are consumer entry points. None of these methods exist on the SDK's `Engine` struct. The actual facade methods per `wave4-cli-sdk.md` are only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. `students()`/`assessment()`/`fees()`/`hr()` are 4 documented APIs that are entirely absent.

**Expected:**

Per `docs/library-docs.md:181-188`: `engine.students().admit(cmd)`, `engine.assessment().enter_marks(cmd)`, `engine.fees().generate_invoice(cmd)`, `engine.hr().generate_payroll(cmd)`.

**Evidence:**

- `docs/library-docs.md:181` — `engine.students().admit(cmd).await?` — admit a student.
  - `docs/library-docs.md:184` — `engine.assessment().enter_marks(cmd).await?` — enter marks.
  - `docs/library-docs.md:186` — `engine.fees().generate_invoice(cmd).await?` — generate a fees invoice.
  - `docs/library-docs.md:188` — `engine.hr().generate_payroll(cmd).await?` — generate monthly payroll.
  - `crates/tools/sdk/src/engine.rs:128-147` — `Engine` exposes only `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. None of `students()`, `assessment()`, `fees()`, `hr()` exist.

---

### FINDING 12 (id: `DOC-2-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:154-165` (Construction) and `docs/library-docs.md:218` (file end)

**Description:**

`library-docs.md` Construction example shows `.build().await?` (line 161) on the builder. The actual `EngineBuilder::build()` is sync, returns `Result<Engine, SdkError>`, and is not `async` / `await`-able (line 258). The doc code shows an `async fn main()` returning `Result<(), Box<dyn std::error::Error>>` but the final line of the construction is `Ok(())`, never `let engine = ...?`. The `await?` is unreachable in `#[tokio::main]` because the builder is sync; this is either a leftover from a previous async-builder design or a copy-paste error.

**Expected:**

Per `docs/library-docs.md:154-165`: `.build().await?` (an `async` builder).

**Evidence:**

- `docs/library-docs.md:155-162` — `let engine = Engine::builder().storage(...).build().await?;` — `.build()` followed by `.await?`.
  - `crates/tools/sdk/src/engine.rs:258` — `pub fn build(self) -> Result<Engine, SdkError>` — synchronous return, no `async fn`, no `Future` return type.
  - `crates/tools/sdk/src/engine.rs:170-174` — `impl Default for EngineBuilder` uses `#[allow(clippy::derivable_impls)] fn default() -> Self { Self::new() }` — confirms `new()` is sync.

---

### FINDING 13 (id: `DOC-2-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:188-191` (Tenant Context: `engine.auth().authenticate("Bearer ...")`)

**Description:**

The Tenant Context section calls `engine.auth().authenticate("Bearer eyJhbGciOi...")` returning a `session` whose `school_id()` and `user_id()` are then read. The actual `AuthProvider` trait (per `wave3-auth.md` audit) does not necessarily have an `authenticate(&str)` method — auth is split across `AuthSession::start(...)`, `AuthSession::verify_otp(...)`, and `SessionToken::verify(...)` flows. The single-call `authenticate("Bearer ...")` shortcut shown in `library-docs.md` is not the documented SDK API surface and likely does not compile.

**Expected:**

Per `docs/library-docs.md:188-191`: `engine.auth().authenticate("Bearer ...")` returns a session with `school_id()` and `user_id()`.

**Evidence:**

- `docs/library-docs.md:188-191` — `let session = engine.auth().authenticate("Bearer ...").await?; ... session.school_id(), session.user_id()`.
  - `crates/adapters/auth/src/lib.rs` (per wave3-auth audit): trait decomposition uses `AuthSession`/`SessionToken`/`OtpChallenge` flows, not a single bearer-string `authenticate()` shortcut.
  - `crates/tools/sdk/src/engine.rs:77` — `pub fn auth(&self) -> &Arc<dyn AuthProvider>` returns a `&Arc<dyn AuthProvider>`; the method on the trait is not the single-call shortcut shown.

---

### FINDING 14 (id: `DOC-2-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:193-218` (Calling a Command: `AdmitStudentCommand` fields) vs `crates/domains/academic/src/commands.rs`

**Description:**

The Command example builds an `AdmitStudentCommand` with fields `tenant`, `admission_no`, `first_name`, `last_name`, `date_of_birth`, `gender`, `guardian` (a `GuardianSpec` struct), `class_id`, `section_id`, `academic_year`. The actual `AdmitStudentCommand` struct in `crates/domains/academic/src/commands.rs` (per `wave1-academic.md` audit) has a different shape: it is a flat list of fields (no `GuardianSpec` wrapper, no `tenant` field on the command itself because tenant context is supplied via the dispatcher's separate `TenantContext` argument). The doc's `GuardianSpec { full_name, relation, phone, email }` is a hypothetical wrapper not present in the actual command struct.

**Expected:**

Per `docs/library-docs.md:193-218`: `AdmitStudentCommand { tenant, admission_no, first_name, last_name, date_of_birth, gender, guardian: GuardianSpec { ... }, class_id, section_id, academic_year }`.

**Evidence:**

- `docs/library-docs.md:194-215` — full `AdmitStudentCommand` literal with `tenant`, `admission_no`, `first_name`, `last_name`, `date_of_birth`, `gender`, `guardian: GuardianSpec { ... }`, `class_id`, `section_id`, `academic_year`.
  - `crates/domains/academic/src/commands.rs` (per wave1-academic audit): `AdmitStudentCommand` has a different field layout; the `GuardianSpec` wrapper struct does not exist on the academic crate.
  - `docs/code-standards.md:128` — `AdmitStudent` is a command (not a method on `Student`), so the tenant context is supplied at dispatch, not on the command.

---

### FINDING 16 (id: `DOC-2-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:178-186` (Querying section: `engine.students().query().active().in_class(class_id).order_by(...)`)

**Description:**

The Querying section calls `engine.students().query().active().in_class(class_id)` and `.where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) })`. There is no `engine.students()` method on `Engine` (only `engine.admission()`, `engine.attendance()`, etc. per `crates/tools/sdk/src/engine.rs:123-146`). Furthermore, `.active()` and `.in_class()` are extension traits per `query_layer.md:382-410`, but the doc imports them as `use educore::academic::query::*` (line 178) — there is no `query` submodule on `educore::academic`.

**Expected:**

Per `docs/library-docs.md:178-186`: `engine.students().query().active().in_class(class_id).order_by(StudentField::LastName).page(0, 50).await?`.

**Evidence:**

- `docs/library-docs.md:179-186` — `let page = engine.students().query().active().in_class(class_id).order_by(StudentField::LastName).page(0, 50).await?;`.
  - `crates/tools/sdk/src/engine.rs:123-146` — no `students()` method on `Engine`; only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`.
  - `docs/query_layer.md:382-410` — extension traits `StudentQueryScopes` must be defined by the consumer; the doc treats them as if they are pre-defined in `educore::academic::query::*`.

---

### FINDING 17 (id: `DOC-2-017`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:201-209` (Subscribing to Events: `engine.events().subscribe::<StudentAdmitted>().await?`)

**Description:**

The Subscribing section shows `engine.events().subscribe::<StudentAdmitted>().await?` returning `sub` with `sub.next().await`. The actual `Engine` struct has no `events()` method (per `crates/tools/sdk/src/engine.rs:48-161`; the available methods are `bus()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`). The event subscription API uses `engine.bus().subscribe(...)` or, per ADR-005, the outbox + relay pattern via a separate consumer. `StudentAdmitted` as a typed Rust struct may exist in `crates/domains/academic/src/events.rs`, but the `subscribe::<T>()` syntax (with turbofish) is not how the `EventBus` trait is documented in `docs/ports/event-bus.md`.

**Expected:**

Per `docs/library-docs.md:201-209`: `engine.events().subscribe::<StudentAdmitted>().await?` returning a stream-like `sub`.

**Evidence:**

- `docs/library-docs.md:201-209` — `let mut sub = engine.events().subscribe::<StudentAdmitted>().await?; while let Some(event) = sub.next().await { ... }`.
  - `crates/tools/sdk/src/engine.rs:107` — `pub fn bus(&self) -> &Arc<dyn EventBus>` — only `bus()`, no `events()`.
  - `crates/tools/sdk/src/engine.rs:48-161` — full impl block has no `events()` method.

---

### FINDING 18 (id: `DOC-2-018`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:213-219` (Capability Check: `Capability::StudentAdmit`)

**Description:**

The Capability Check section calls `engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit)`. The actual `Engine` struct has no `rbac()` method (per `crates/tools/sdk/src/engine.rs:48-161`); the available methods are `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen`, `admission`, `attendance`, `payment_svc`, `notify_svc`. Furthermore, the `Capability` enum (per `wave2-rbac.md` audit and `crates/cross-cutting/rbac/src/`) uses a namespaced form `Capability::Student.StudentAdmit` (a `(Domain, Aggregate, Action)` triple) — not the flat `Capability::StudentAdmit` shown in the doc.

**Expected:**

Per `docs/library-docs.md:213-219`: `engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit)`.

**Evidence:**

- `docs/library-docs.md:213-219` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? { ... }`.
  - `crates/tools/sdk/src/engine.rs:48-161` — no `rbac()` method; only the 13 accessors listed above.
  - `crates/cross-cutting/rbac/src/capability.rs` (per wave2-rbac audit): `Capability` is a struct/enum with namespaced variants, not flat `StudentAdmit`.

---

### FINDING 19 (id: `DOC-2-019`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:233-240` (Error Handling: `DomainError::Validation { field, reason }`)

**Description:**

The Error Handling section pattern-matches on `DomainError::Validation { field, reason }`, `DomainError::Conflict { entity, reason }`, `DomainError::NotFound { entity, id }`, `DomainError::Forbidden { reason }`, `DomainError::Infrastructure(source)`. Per `wave4-core.md` finding CORE-006 and the actual `crates/infra/core/src/error.rs:19-63`, `DomainError` is a tuple-variant enum with `String` payloads (not struct variants with named fields): `Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String)`. The doc's struct-pattern syntax is a compile error against the real type.

**Expected:**

Per `docs/library-docs.md:233-240`: `DomainError::Validation { field, reason }` struct-pattern matching.

**Evidence:**

- `docs/library-docs.md:233-240` — `Err(DomainError::Validation { field, reason }) => { ... }` and similar struct patterns for `Conflict`, `NotFound`, `Forbidden`.
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String) }` — all tuple variants.
  - The doc also omits `TenantViolation` and `NotSupported` from the match — two variants the consumer would never see as compilation errors.

---

### FINDING 2 (id: `DOC-2-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:14` (Rust Standards: "All `unwrap`, `expect`, `panic!` are forbidden in production code paths")

**Description:**

The "forbidden patterns" list (lines 175-186) and "Validation Checklist" (lines 195-205) state `unwrap`/`expect`/`panic` are forbidden in production code. `wave4-testkit.md` (FINDING TTK-001 onward) and `wave4-core.md` (FINDING CORE-007 onward) document widespread violations across `crates/tools/testkit/`, `crates/infra/core/`, `crates/tools/cli/`, and `crates/tools/sdk/`. The standard does not define "production paths" precisely — only the lint module (`crates/infra/core/src/lint.rs:181-238`) attempts a `#[cfg(test)]` exclusion, and it omits `.expect(` entirely.

**Expected:**

Per `docs/code-standards.md:175-186`: zero `unwrap`/`expect`/`panic` in non-test code; lint must enforce.

**Evidence:**

- `docs/code-standards.md:175-186` — "`unwrap()`, `expect()`, `panic!` in production paths."
  - `docs/code-standards.md:197` — "No new `unwrap`/`expect`/`panic` in non-test code."
  - `crates/infra/core/src/lint.rs:220` — anti-pattern needle array contains only `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()`; `.expect(` is missing from the detection list.
  - `wave4-testkit.md` (per pre-audit knowledge): widespread `unwrap`/`expect` in `crates/tools/testkit/src/*.rs`.
  - `wave4-core.md` (per pre-audit knowledge): `unwrap`/`expect` violations in `crates/infra/core/src/*.rs`.

---

### FINDING 20 (id: `DOC-2-020`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/query_layer.md:130-150` (Field Exhaustiveness Enum) vs `crates/infra/query-derive/src/lib.rs:330-365`

**Description:**

`query_layer.md` § "Field Exhaustiveness Enum" (lines 130-150) shows `StudentField { Status, LastName, ClassId, ParentId }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`. The actual macro emits `*Field` enums with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` only — `Serialize` and `Deserialize` are NOT derived on the field enum (see `crates/infra/query-derive/src/lib.rs:330-340`). The doc also omits the `Default` requirement that the macro enforces (a struct with no `#[query(...)]` decorated fields fails to compile; see `crates/infra/query-derive/src/lib.rs:241-249`).

**Expected:**

Per `docs/query_layer.md:130-150`: `StudentField { Status, LastName, ClassId, ParentId }` with `Serialize + Deserialize`.

**Evidence:**

- `docs/query_layer.md:130-139` — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)] pub enum StudentField { Status, LastName, ClassId, ParentId }`.
  - `crates/infra/query-derive/src/lib.rs:329-345` — macro emits `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] #struct_vis enum #field_enum_name { ... }` — no `Serialize`/`Deserialize` derives.
  - `crates/infra/query-derive/src/lib.rs:241-249` — macro errors with "DomainQuery requires at least one `#[query(...)]` decorated field (filterable, sortable, or relation)" when no decorated fields exist; the doc says "non-decorated fields are excluded," not "struct compilation fails."

---

### FINDING 21 (id: `DOC-2-021`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/query_layer.md:170-185` (Type-Safe State Builder: `BTreeSet<StudentRelation>`) vs `crates/infra/query-derive/src/lib.rs:485-505`

**Description:**

`query_layer.md` § "Type-Safe State Builder" (lines 170-185) shows the builder with a `relations: BTreeSet<StudentRelation>` field for hydration directives. The actual macro-emitted builder (per `crates/infra/query-derive/src/lib.rs:485-505`) does carry a `BTreeSet<#relation_enum_name> relations` field, BUT the `with(...)` method on the builder inserts the relation into a BTreeSet (lines 600-615) that is never read by the repository — there is no `apply_hydration` or `hydrate` step in the macro, the query builder itself, or the storage adapters. The BTreeSet is dead code; the storage adapters (per `wave3-storage-*` audits) do not consult `relations` when translating the query.

**Expected:**

Per `docs/query_layer.md:170-185`: `relations: BTreeSet<StudentRelation>` consumed by the repository's hydration step.

**Evidence:**

- `docs/query_layer.md:170-185` — `pub struct StudentQueryBuilder { ... relations: BTreeSet<StudentRelation> }` with the note "the `with(...)` set is internally a `BTreeSet`, so duplicate hydration directives are O(log n) and free of side effects."
  - `crates/infra/query-derive/src/lib.rs:485-505` — builder struct definition with `relations: ::std::collections::BTreeSet<#relation_enum_name>`.
  - `crates/infra/query-derive/src/lib.rs:600-615` — `fn with(mut self, relation: #relation_enum_name) -> Self { self.relations.insert(relation); self }`.
  - `crates/infra/storage/src/repository.rs:25-72` (per wave4-storage-port audit) — `Repository<A>` trait does not take a `relations` set or consult hydration directives; only the `QueryNode` is consumed.

---

### FINDING 22 (id: `DOC-2-022`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/query_layer.md:240-265` (Query AST: `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`) vs `crates/infra/core/src/query.rs:271-275`

**Description:**

`query_layer.md` § "Query AST" defines `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)` — the inner type is generic over the related entity's field type via a `RelatedField` type parameter. The actual code (`crates/infra/core/src/query.rs:271-275`) uses a concrete `pub struct RelationalField;` (a unit struct), not a generic `RelatedField` type. The macro emits a `to_relational_node(...)` helper that erases the related field type to `RelationalField`. The doc's generic AST node does not exist.

**Expected:**

Per `docs/query_layer.md:240-265`: `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`.

**Evidence:**

- `docs/query_layer.md:240-265` — `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`.
  - `crates/infra/core/src/query.rs:271-275` — `HasRelation(Relation, Box<QueryNode<RelationalField>>)` — concrete `RelationalField`, no generic.
  - `crates/infra/core/src/query.rs:380-385` — `pub struct RelationalField;` — the unit struct that replaces the doc's `RelatedField` generic.
  - `crates/infra/query-derive/src/lib.rs:545-555` — `let inner_rel: ::educore_core::query::QueryNode<::educore_core::query::RelationalField> = ::educore_core::query::to_relational_node(inner);` — the macro flattens to `RelationalField`.

---

### FINDING 24 (id: `DOC-2-024`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/query_layer.md:520-560` (Aggregation & Reporting: `StudentAggregate::Count`, `engine.students().aggregate(...)`) vs `crates/infra/query-derive/src/lib.rs:1-825`

**Description:**

`query_layer.md` § "Aggregation & Reporting" documents `engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).execute().await?` with a documented `StudentAggregate` enum (Count, Sum, Avg, Min, Max) that the macro is supposed to emit. The actual macro (`crates/infra/query-derive/src/lib.rs:1-825`) emits NO `StudentAggregate` enum, NO `.aggregate(...)` method on the builder, NO `.group_by(...)` method, NO `.execute()` method. The aggregation API is entirely aspirational; no code emits it.

**Expected:**

Per `docs/query_layer.md:520-560`: `StudentAggregate` enum, `aggregate()`, `group_by()`, `execute()` methods.

**Evidence:**

- `docs/query_layer.md:520-538` — `let summary = engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).where_eq(StudentField::Status, StudentStatus::Active).execute().await?;` and "Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric fields."
  - `crates/infra/query-derive/src/lib.rs:1-825` — `aggregate`, `group_by`, `StudentAggregate` do not appear in the macro source.
  - `crates/infra/core/src/query.rs:1-576` — no `StudentAggregate` enum, no aggregation AST node.

---

### FINDING 27 (id: `DOC-2-027`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:307-340` (`sync` feature flag on umbrella crate) vs `crates/educore/Cargo.toml:46-47`

**Description:**

ADR-018 § "The `SyncAdapter` is a build feature" specifies `crates/educore/Cargo.toml` declares `default = []` and `sync = ["educore-sync", "educore-sync-inprocess"]`, gating the `sync()` builder method behind the feature. The actual `crates/educore/Cargo.toml:46-47` declares `educore-sync` and `educore-sync-inprocess` as unconditional dependencies, not feature-gated. Without the `sync` feature, the SDK has no way to disable sync, contradicting the ADR.

**Expected:**

Per `ADR-018:308-340`: `default = []`, `sync = ["educore-sync", "educore-sync-inprocess"]` in `crates/educore/Cargo.toml`.

**Evidence:**

- `docs/decisions/ADR-018-SyncEngineArchitecture.md:307-340` — `[features] default = []; sync = ["educore-sync", "educore-sync-inprocess"]` is the documented configuration.
  - `crates/educore/Cargo.toml:46-47` — `educore-sync = { workspace = true }` and `educore-sync-inprocess = { workspace = true }` declared as dependencies, no `[features]` block.
  - `crates/tools/sdk/src/engine.rs` — no `#[cfg(feature = "sync")]` gate on `sync()`-related methods.

---

### FINDING 4 (id: `DOC-2-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:181-185` (Forbidden Patterns: "`serde_json::Value` in domain code") vs `crates/infra/core/src/value_objects.rs` and others

**Description:**

`serde_json::Value` is forbidden in domain code. The lint does not detect it (`scan_file_for_anti_patterns` needle array has only 5 entries — see FINDING 2). Cross-referenced with `wave4-core.md` finding CORE-009, the value_objects module and the query AST expose JSON-typed columns in places that may pass through `serde_json::Value`. Without an enforced check, the rule is aspirational.

**Expected:**

Per `docs/code-standards.md:181-184`: `serde_json::Value` is forbidden; lint enforces.

**Evidence:**

- `docs/code-standards.md:182` — "`serde_json::Value` in domain code."
  - `docs/code-standards.md:199` — "No new `serde_json::Value` in domain code."
  - `crates/infra/core/src/lint.rs:220` — needle array does not include `serde_json::Value`.
  - `crates/infra/core/src/value_objects.rs` — (per wave4-core audit) contains `serde_json::Value` usage that is unverified.

---

### FINDING 5 (id: `DOC-2-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:113` ("No `HashMap<String, T>` for domain data") vs the lint (`crates/infra/core/src/lint.rs:181-238`)

**Description:**

The `HashMap<String, T>` ban is stated in Type Safety (line 113) and Forbidden Patterns (line 183) but the lint's anti-pattern scanner (`crates/infra/core/src/lint.rs:181-238`) does not include a `HashMap<String` regex. The validation checklist (line 200) does not list this rule — only `unwrap`/`as`/`serde_json::Value` are explicit checklist items. So three rules are documented but only two are checklist-enforced, and none are lint-enforced.

**Expected:**

Per `docs/code-standards.md:113, 183`: `HashMap<String, T>` is forbidden; checklist + lint enforce.

**Evidence:**

- `docs/code-standards.md:113` — "No `HashMap<String, T>` for domain data. Use typed structs."
  - `docs/code-standards.md:183` — "`HashMap<String, T>` for domain data." (Forbidden Patterns)
  - `docs/code-standards.md:195-205` — Validation Checklist omits `HashMap<String, T>` ban.
  - `crates/infra/core/src/lint.rs:220` — needle array has no `HashMap<String` detection.

---

### FINDING 6 (id: `DOC-2-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:97-100` (Spec folder layout, 11 files) vs `docs/specs/<domain>/` actual contents

**Description:**

The 11-file spec layout is mandatory: `overview.md, aggregates.md, entities.md, value-objects.md, commands.md, events.md, services.md, permissions.md, repositories.md, workflows.md, tables.md`. `AGENTS.md` mirrors this and explicitly notes `services.md` (not `policies.md`), `permissions.md` (not `policies.md`), and `workflows.md` (not `errors.md`). Per `wave1-*` and `wave2-*` audit reports, several spec folders omit `permissions.md` and `repositories.md` (e.g., finance, hr, library). The doc claims the layout is mandatory but provides no per-domain conformance table.

**Expected:**

Per `docs/code-standards.md:97-100` + `AGENTS.md` Spec folder layout: every `docs/specs/<domain>/` must have all 11 files.

**Evidence:**

- `docs/code-standards.md:97-100` — "The 11 files per spec folder are: `overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`."
  - `docs/specs/finance/` (per wave1-finance audit): missing `permissions.md`, `repositories.md`.
  - `docs/specs/hr/` (per wave1-hr-library audit): missing `permissions.md`.
  - `docs/specs/library/` (per wave1-hr-library audit): missing `permissions.md`.

---

### FINDING 7 (id: `DOC-2-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:127-128` (Async: "Repositories and ports use `async_trait`") vs `crates/infra/storage/src/repository.rs:25-72`

**Description:**

The standards mandate `async_trait` for repositories and ports. The actual `Repository<A>` trait in `crates/infra/storage/src/repository.rs:25-72` (per wave4-storage-port audit) is a native `async fn` trait, NOT `#[async_trait]`-decorated. This contradicts the documented standard.

**Expected:**

Per `docs/code-standards.md:127-128`: `async_trait` macro on repositories and ports.

**Evidence:**

- `docs/code-standards.md:127-128` — "Repositories and ports use `async_trait`."
  - `crates/infra/storage/src/repository.rs:25-72` — `pub trait Repository<A>: Send + Sync where A: Send + Sync + Clone + 'static { async fn get(...); async fn list(...); ... }` — native `async fn` in trait, no `#[async_trait]`.
  - `crates/infra/storage/src/port.rs:34-150` — `StorageAdapter` trait also uses native `async fn` (no `#[async_trait]`).

---

### FINDING 8 (id: `DOC-2-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/code-standards.md:71-86` (Module Rules) vs `crates/domains/academic/src/` actual files

**Description:**

The mandatory module layout per domain crate is: `lib.rs, aggregate.rs, entities.rs, value_objects.rs, commands.rs, events.rs, services.rs, repository.rs, query.rs, errors.rs` (10 files). Per `wave1-*` audits, several domain crates are missing files. For example, `crates/domains/hr/` and `crates/domains/finance/` are missing `services.rs` and `repository.rs` in some phases. The standard names `services.rs` (services) but AGENTS.md cross-references `services.md` in specs and the standard calls `services.rs` (services, policies). Naming and presence is inconsistent.

**Expected:**

Per `docs/code-standards.md:71-86`: every `crates/domains/<d>/src/` has the 10 files listed.

**Evidence:**

- `docs/code-standards.md:71-86` — Module Rules listing the 10 required files.
  - `crates/domains/hr/src/` (per wave1-hr-library audit): missing `services.rs` and `query.rs`.
  - `crates/domains/finance/src/` (per wave1-finance audit): missing `repository.rs`.

---

### FINDING 11 (id: `DOC-2-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:154-165` (Construction: `Engine::builder()`) vs `crates/tools/sdk/src/engine.rs:179`

**Description:**

`library-docs.md` Construction example shows `Engine::builder()` as the entry point. The actual code defines `EngineBuilder::new()` (line 179) — there is no `Engine::builder()` shortcut method on `Engine` itself (the `impl Engine` block at line 48 has `test_world`, `storage()`, `auth()`, etc. but no `builder()`). Consumers following the documented example verbatim get a compile error: "no associated function named `builder` found for struct `educore::sdk::Engine`". The example must be `EngineBuilder::new()`.

**Expected:**

Per `docs/library-docs.md:154-165`: `Engine::builder()` returning a builder.

**Evidence:**

- `docs/library-docs.md:154-165` — `let engine = Engine::builder().storage(...).build().await?;`.
  - `crates/tools/sdk/src/engine.rs:179` — `pub fn new() -> Self` on `EngineBuilder`; the `Engine` struct has no `builder()` method.
  - `crates/tools/sdk/src/engine.rs:258` — `EngineBuilder.build()` returns `Result<Engine, SdkError>`, not `await`able.
  - `crates/tools/sdk/src/engine.rs:48-146` — `impl Engine` exposes `test_world`, `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen`, `admission`, `attendance`, `payment_svc`, `notify_svc`. No `builder`.

---

### FINDING 15 (id: `DOC-2-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:223` (`NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()`)

**Description:**

`library-docs.md` line 223 calls `NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` to construct a date-of-birth. `code-standards.md` § Forbidden Patterns (line 175) and AGENTS.md "Type Safety" forbid `unwrap` in production paths. The doc example demonstrates the forbidden pattern in a consumer-facing code sample.

**Expected:**

Per `docs/code-standards.md:175-186`: `unwrap` is forbidden in production paths.

**Evidence:**

- `docs/library-docs.md:223` — `date_of_birth: NaiveDate::from_ymd_opt(2010, 12, 10).unwrap(),`.
  - `docs/code-standards.md:175-186` — "`unwrap()`, `expect()`, `panic!` in production paths." (Forbidden Patterns).
  - `AGENTS.md` Validation Checklist — "No new `unwrap`/`expect`/`panic` in non-test code".

---

### FINDING 23 (id: `DOC-2-023`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:240-265` (Query AST variants) vs `crates/infra/core/src/query.rs:226-265`

**Description:**

The documented `QueryNode` has 13 variants: `Eq, Ne, Lt, Lte, Gt, Gte, In, NotIn, Between, IsNull, IsNotNull, Like, ILike, HasRelation`. The actual `QueryNode` enum (`crates/infra/core/src/query.rs:226-265`) has 16 variants — the three additional are `And(Box<QueryNode<F>>, Box<QueryNode<F>>)`, `Or(Box<QueryNode<F>>, Box<QueryNode<F>>)`, `Not(Box<QueryNode<F>>)`. The macro (`crates/infra/query-derive/src/lib.rs:780-820`) emits `And` nodes to compose multiple filters; storage adapters (per `wave3-storage-*` audits) consume `And`/`Or`/`Not`. The doc omits the boolean-composition operators entirely.

**Expected:**

Per `docs/query_layer.md:240-265`: 14-variant `QueryNode` (13 leaf + `HasRelation`).

**Evidence:**

- `docs/query_layer.md:240-265` — `QueryNode` enum lists 14 variants (13 + `HasRelation`).
  - `crates/infra/core/src/query.rs:226-265` — actual `QueryNode` lists 17 variants: `Eq, Ne, Lt, Lte, Gt, Gte, In, NotIn, Between, IsNull, IsNotNull, Like, ILike, And, Or, Not, HasRelation` (16 + the doc's HasRelation = 17).
  - `crates/infra/query-derive/src/lib.rs:780-820` — `__educore_compile()` folds filters into `QueryNode::And(...)`.

---

### FINDING 25 (id: `DOC-2-025`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:300-340` (`where_has` signature with closure bound to related builder) vs `crates/infra/query-derive/src/lib.rs:550-580`

**Description:**

The doc claims `where_has` takes a closure bound to the related entity's macro-generated builder: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>`. The macro emits a per-relation `where_has_<Relation>` method (e.g. `where_has_Parent`) that takes `FnOnce(ParentQueryBuilder) -> ParentQueryBuilder` (lines 550-580). The generic `where_has<R, F>` is emitted as a no-op stub (lines 580-590) that ignores the closure and just returns `self`. The doc presents the generic form as the canonical API; in practice, only the per-relation concrete methods do real work.

**Expected:**

Per `docs/query_layer.md:300-340`: `where_has<R, F>(self, relation: R, build: F) -> Self where ...`.

**Evidence:**

- `docs/query_layer.md:300-340` — `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`.
  - `crates/infra/query-derive/src/lib.rs:550-580` — per-relation `where_has_<Relation>` methods (the working ones).
  - `crates/infra/query-derive/src/lib.rs:580-595` — generic `where_has` is emitted as `let _ = relation; let _ = __build; self` — a no-op stub that ignores both arguments.

---

### FINDING 26 (id: `DOC-2-026`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** Task prompt scope vs `docs/decisions/` (18 files on disk)

**Description:**

The task prompt states the audit scope is "all 14 ADRs" but `ls docs/decisions/` returns 18 files (`ADR-001-DDD.md` through `ADR-018-SyncEngineArchitecture.md`). `AGENTS.md` § "Crate Inventory" references ADR-013, 015, 016, 017, 018 in prose but the prompt's count of 14 is contradicted by both `AGENTS.md`'s embedded references and the file count. Either 4 ADRs are unaccounted-for (ADR-001..004, ADR-007..010, ADR-011, ADR-012, ADR-014) or the prompt is using a stale count.

**Expected:**

Per the task prompt: 14 ADRs.

**Evidence:**

- `bash: ls docs/decisions/` → 18 files: `ADR-001-DDD.md`, `ADR-002-Hexagonal.md`, `ADR-003-MultiTenancy.md`, `ADR-004-Commands.md`, `ADR-005-Events.md`, `ADR-006-QueryLayer.md`, `ADR-007-AuditFirst.md`, `ADR-008-OfflineFirst.md`, `ADR-009-CapabilityPermissions.md`, `ADR-010-AIAgent.md`, `ADR-011-RustEcosystem.md`, `ADR-012-NoReflection.md`, `ADR-013-CrateLayout.md`, `ADR-014-Idempotency.md`, `ADR-015-ExternalCrates.md`, `ADR-016-EngineGraph.md`, `ADR-017-SurrealDBFirst.md`, `ADR-018-SyncEngineArchitecture.md`.
  - `docs/decisions/ADR-018-SyncEngineArchitecture.md:1` — `Accepted, 2026-06-12`; status matches ADR-017.
  - All 18 ADRs have `Status: Accepted` in the header (no Deprecated/Superseded entries exist).

---

### FINDING 28 (id: `DOC-2-028`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:230-285` (Four new `StorageAdapter` methods: `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) vs `docs/ports/storage.md:25-150` and `crates/infra/storage/src/port.rs:34-150`

**Description:**

ADR-018 § "Five new methods on `StorageAdapter`" (technically 4, per the heading) specifies the four new sync-engine methods: `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. The ADR's `fn watch_changes(&self) -> Result<ChangeStream>` signature (no `school_id` parameter) differs from ADR-017 § "Four new methods on `StorageAdapter`" which specifies `async fn watch_changes(&self, school_id: SchoolId, since: Cursor) -> Result<ChangeStream>` (with `school_id` and `since`). The two ADRs disagree on the signature, and `docs/ports/storage.md` (per `wave4-storage-port.md` audit) does not document any of the four methods.

**Expected:**

Per `ADR-017:122-135` and `ADR-018:230-285`: 4 new methods on `StorageAdapter` for the sync engine.

**Evidence:**

- `docs/decisions/ADR-017-SurrealDBFirst.md:122-135` — `async fn watch_changes(&self, school_id: SchoolId, since: Cursor) -> Result<ChangeStream>` (with `school_id` and `since`).
  - `docs/decisions/ADR-018-SyncEngineArchitecture.md:230-285` — `fn watch_changes(&self) -> Result<ChangeStream>` (no `school_id`, no `since`).
  - `docs/ports/storage.md:25-150` (per wave4-storage-port audit FINDING SP-005): the four methods are NOT documented in the port spec.
  - `crates/infra/storage/src/port.rs:34-150` — `StorageAdapter` trait body lists `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` — present but signature differs from both ADRs.

---

### FINDING 3 (id: `DOC-2-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/code-standards.md:15` (Rust Standards: "Numeric conversions use `TryFrom`/`TryInto`. `as` is forbidden on numerics.")

**Description:**

The `as` ban on numerics is asserted in three places (line 15 Rust Standards, line 113 Type Safety implicit, line 181 Forbidden Patterns). The lint's anti-pattern scanner (`crates/infra/core/src/lint.rs:181-238`) does not implement an `as` cast detector at all — it only scans for `unwrap`/`panic`/`todo`/`unimplemented`. The validation checklist item "No new `as` on numerics" is therefore unenforced.

**Expected:**

Per `docs/code-standards.md:181`: `as` on numeric types is forbidden; lint enforces.

**Evidence:**

- `docs/code-standards.md:181` — "`as` on numeric types."
  - `docs/code-standards.md:198` — "No new `as` on numerics."
  - `crates/infra/core/src/lint.rs:181-238` — `scan_file_for_anti_patterns` searches only the 5 needles above; no regex/pattern for `as u8`, `as i32`, etc.

---

### FINDING 9 (id: `DOC-2-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-2.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/code-standards.md:149-152` (Dependency Rules: "A domain crate may depend on `educore-events`...")

**Description:**

The dependency rules allow domain crates to depend on `educore-events` (the event envelope + bus port crate). However `AGENTS.md` explicitly notes that `educore-events` is at the cross-cutting tier. The standard says domain crates "may depend on" `educore-events`, but the `educore-events-domain` (calendar) crate is also at the cross-cutting tier and is NOT listed in the domain dependencies — implying domain crates cannot depend on the calendar event aggregate. Yet `educore-cms` (per AGENTS.md Phase 12 entry) depends on `educore-academic` for `ClassId`/`SectionId`/`AcademicYearId` — a precedent for cross-domain deps — and AGENTS.md allows this only "with explicit justification in an ADR". No such ADR exists.

**Expected:**

Per `docs/code-standards.md:149-152`: domain crates may depend on listed crates; any other cross-domain dep needs an ADR.

**Evidence:**

- `docs/code-standards.md:149-152` — "A domain crate may depend on: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` ... Other domain crates only with explicit justification in an ADR."
  - `crates/domains/cms/Cargo.toml` (per AGENTS.md Phase 12): depends on `educore-academic`; no ADR cited.
  - `docs/decisions/` — no ADR justifies `educore-cms` → `educore-academic`.

---


## Library Docs (target id prefix: `DOC-LIB`)

**Path:** `docs/library-docs.md`  
**Total findings:** 11 (4 critical, 5 high, 2 medium, 0 low)


### FINDING 1 (id: `DOC-LIB-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:181-188`

**Description:**

The "Common Workflows" section claims `engine.students().admit(cmd).await?` is the consumer API for admitting a student. No `Engine::students()` method exists; the SDK's `Engine` exposes `engine.admission()` (returning `AdmissionService`), not `engine.students()`. The actual `admit` flow is `educore_academic::services::admit_student(cmd, &clock, &ids, &uniqueness)`, a free function with a 4-arg signature — not a method on any engine struct.

**Expected:**

`engine.students().admit(cmd).await?` per `docs/library-docs.md:181`.

**Evidence:**

- `docs/library-docs.md:181` — `engine.students().admit(cmd).await?` — admit a student.
  - `crates/tools/sdk/src/engine.rs:123-127` — `/// Returns a handle to the admission facade.` / `pub fn admission(&self) -> AdmissionService<'_> { AdmissionService::new(self) }` — only `admission()`, no `students()`.
  - `crates/tools/sdk/src/engine.rs:128-147` — `Engine` exposes `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `students()` / `assessment()` / `hr()` / `fees()`.

---

### FINDING 2 (id: `DOC-LIB-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:182-188`

**Description:**

The "Common Workflows" list includes `engine.assessment()`, `engine.fees()`, and `engine.hr()` as consumer entry points. None of these exist on the `Engine` struct. The SDK exposes only `admission()` (academic), `attendance()`, `payment_svc()`, and `notify_svc()` facade services — there are no facade methods for assessment, fees, hr, or students.

**Expected:**

`engine.assessment().enter_marks(cmd).await?`, `engine.fees().generate_invoice(cmd).await?`, `engine.hr().generate_payroll(cmd).await?` per `docs/library-docs.md:184-188`.

**Evidence:**

- `docs/library-docs.md:184` — `engine.assessment().enter_marks(cmd).await?` — enter marks.
  - `docs/library-docs.md:186` — `engine.fees().generate_invoice(cmd).await?` — generate a fees invoice.
  - `docs/library-docs.md:188` — `engine.hr().generate_payroll(cmd).await?` — generate monthly payroll.
  - `crates/tools/sdk/src/engine.rs:123-146` — `Engine` has no `assessment()`, `fees()`, or `hr()` method; only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`.

---

### FINDING 3 (id: `DOC-LIB-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:154-165, 169-176`

**Description:**

The doc claims `engine.events().subscribe::<StudentAdmitted>().await?` and `engine.rbac().has_capability(...)`. No `Engine::events()` or `Engine::rbac()` method exists; the event bus is exposed as `engine.bus()` (returning `&Arc<dyn EventBus>`), and there is no RBAC accessor at all on `Engine`. The subscribe signature itself is also wrong: `subscribe` takes `SubscribeOptions` (with `consumer`, `topic`, `filter`, `start`, `batch_size`, `visibility_timeout`) and returns `Box<dyn EventSubscription>` — not a turbofish-only generic call returning a stream.

**Expected:**

`engine.events().subscribe::<StudentAdmitted>().await?` per `docs/library-docs.md:157-160`; `engine.rbac().has_capability(...)` per `docs/library-docs.md:171-172`.

**Evidence:**

- `docs/library-docs.md:157-160` — `let mut sub = engine.events().subscribe::<StudentAdmitted>().await?;`
  - `docs/library-docs.md:171-172` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? {`
  - `crates/tools/sdk/src/engine.rs:104-109` — `/// Returns a reference to the event bus.` / `pub fn bus(&self) -> &Arc<dyn EventBus> { &self.bus }` — only `bus()`, no `events()`.
  - `crates/tools/sdk/src/engine.rs:42-147` — no `rbac()` accessor on `Engine`.
  - `crates/cross-cutting/events/src/event_bus.rs:48` — `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;` — the actual signature, not a generic-turbofish call.

---

### FINDING 4 (id: `DOC-LIB-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/library-docs.md:9-26`

**Description:**

The "Construction" example uses `Engine::builder()`, `.build().await?`, `JwtAuthProvider::from_env()?`, `EmailNotifier::from_env()?`, `InProcessBus::new()`, and `UuidV7Generator::new()`. None of these identifiers exist: the actual builder is `EngineBuilder::new()`, the build is synchronous (`pub fn build(self) -> Result<Engine, SdkError>`), the JWT provider is `JwtAuthProviderBuilder::new().build()` (no `from_env()`), the notifier is `EmailProviderBuilder::new()` (the struct is `EmailProvider`, not `EmailNotifier`, and there is no `from_env()`), the bus is `InProcessEventBus::new()` (no `InProcessBus`), and the id generator is `SystemIdGen` (a unit struct, no `new()`).

**Expected:**

`Engine::builder().storage(...).auth(JwtAuthProvider::from_env()?)....build().await?` per `docs/library-docs.md:9-26`.

**Evidence:**

- `docs/library-docs.md:14-22` — full builder example using `Engine::builder()`, `.build().await?`, `JwtAuthProvider::from_env()?`, `EmailNotifier::from_env()?`, `InProcessBus::new()`, `UuidV7Generator::new()`.
  - `crates/tools/sdk/src/engine.rs:179-191` — `pub fn new() -> Self { Self { storage: None, ... } }` — the constructor is `EngineBuilder::new()`, not `Engine::builder()`.
  - `crates/tools/sdk/src/engine.rs:258` — `pub fn build(self) -> Result<Engine, SdkError> { ... }` — build is sync, returns `SdkError`, not `await`-able.
  - `crates/adapters/auth/src/jwt.rs:161-167, 224-225` — builder constructor is `JwtAuthProviderBuilder::new()`, then `.build()`; no `JwtAuthProvider::from_env()`.
  - `crates/adapters/notify/src/email.rs:75, 204-217, 261` — `pub struct EmailProvider` (not `EmailNotifier`), with `EmailProviderBuilder::new()`; no `from_env()`.
  - `crates/adapters/event-bus/src/in_process.rs:123, 161` — `pub struct InProcessEventBus` (not `InProcessBus`); `InProcessEventBus::new()` exists.
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` (a unit struct with `impl IdGenerator for SystemIdGen`); no `UuidV7Generator`.

---

### FINDING 10 (id: `DOC-LIB-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:84-150`

**Description:**

The library-doc query examples assume the `#[derive(DomainQuery)]` macro emits a generic `where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) })` plus `.active()` / `.in_class(class_id)` extension traits. The macro does emit `where_has_Parent(...)` as a per-relation typed method (no relation parameter), and `.active()` / `.in_class()` are not defined anywhere in the academic crate — they would have to be developer-authored extension traits per the query_layer spec. The library-doc example also assumes a `Student { ..., parent: Option<Parent>, }` field that does not exist on the `Student` aggregate.

**Expected:**

`engine.students().query().active().where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) }).order_by(StudentField::LastName).page(0, 50).await?` with `student.parent` per `docs/library-docs.md:89-99` and `:115-122`.

**Evidence:**

- `docs/library-docs.md:89-99` — the full `.active() / .in_class() / .where_has(StudentRelation::Parent, ...)` chained query.
  - `crates/domains/academic/src/aggregate.rs:56-119` — `pub struct Student { ... }` has no `parent: Option<Parent>` field; the `Student` aggregate carries only own-data fields (`id, school_id, admission_no, first_name, last_name, date_of_birth, gender, blood_group, ..., custom_fields, version, etag, created_at, ...`).
  - `crates/infra/query-derive/src/lib.rs:602-632` — the macro emits one typed method per relation (e.g. `where_has_Parent`), NOT a generic `where_has(StudentRelation::Parent, |p| { ... })`.
  - `crates/domains/academic/src/query.rs:26-122` — the academic `StudentQuery` is a hand-written stub with `with_status / with_class_id / ...` setters; no `.active()` / `.in_class()` / `.where_has_*()` methods exist.

---

### FINDING 11 (id: `DOC-LIB-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:70-82`

**Description:**

The "Querying" prose claims "there is no hand-written `StudentField` in the consumer codebase, the type is generated from the struct definition on every compile" and that "the macro produces a structurally complete but semantically neutral builder". This contradicts the academic crate, which ships a hand-written `StudentQuery` struct (not macro-generated) at `crates/domains/academic/src/query.rs:28-43`, with hand-written setter methods, and explicitly defers the macro-generated builder to a later phase.

**Expected:**

Per `docs/library-docs.md:74-82`: "the macro emits a typed `*Field` enum and a `*QueryBuilder` state struct per aggregate — there is no hand-written `StudentField` in the consumer codebase".

**Evidence:**

- `docs/library-docs.md:74-77` — "the macro emits a typed `*Field` enum and a `*QueryBuilder` state struct per aggregate — there is no hand-written `StudentField` in the consumer codebase, the type is generated from the struct definition on every compile."
  - `crates/domains/academic/src/query.rs:27-122` — `pub struct StudentQuery { pub status_filter: Option<StudentStatus>, pub class_id_filter: Option<ClassId>, pub section_id_filter: Option<SectionId>, pub academic_year_id_filter: Option<AcademicYearId>, pub first_name_contains: Option<String>, pub last_name_contains: Option<String>, pub admission_no_contains: Option<String>, }` — a hand-written struct, not macro-generated.
  - `crates/domains/academic/src/query.rs:115-121` — the `execute` method returns `Err(DomainError::not_supported("StudentQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+"))` — explicitly a stub.

---

### FINDING 5 (id: `DOC-LIB-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:33-39, 169-176`

**Description:**

The "Tenant Context" and "Capability Check" examples construct `TenantContext::new(session.school_id(), session.user_id())` and reference `Capability::StudentAdmit`. Neither constructor nor variant exists. The actual constructor is `TenantContext::for_user(school_id, actor_id, correlation_id, user_type)` and the academic-admission capability variant is `Capability::AcademicStudentCreate`.

**Expected:**

`TenantContext::new(session.school_id(), session.user_id())` and `Capability::StudentAdmit` per `docs/library-docs.md:38` and `:172`.

**Evidence:**

- `docs/library-docs.md:38` — `let tenant = TenantContext::new(session.school_id(), session.user_id());`
  - `docs/library-docs.md:172` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? {`
  - `crates/infra/core/src/tenant.rs:56-75` — `impl TenantContext { pub fn for_user(school_id: SchoolId, actor_id: UserId, correlation_id: CorrelationId, user_type: UserType) -> Self {...} }` — no `TenantContext::new(SchoolId, UserId)`.
  - `crates/cross-cutting/rbac/src/value_objects.rs:73, 75, 77, 79` — `AcademicStudentCreate, AcademicStudentRead, AcademicStudentUpdate, AcademicStudentDelete` — the academic student capabilities; no `StudentAdmit` variant.

---

### FINDING 6 (id: `DOC-LIB-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:43-68`

**Description:**

The "Calling a Command" example constructs `AdmitStudentCommand` with fields `admission_no: AdmissionNumber`, `guardian: GuardianSpec { ... full_name, relation: GuardianRelation::Mother, phone: PhoneNumber, email: EmailAddress }`, `class_id: ClassId::new(tenant.school_id())`, `section_id: SectionId::new(tenant.school_id())`, `academic_year: AcademicYear::current(tenant.school_id(), &clock)`. None of those types or constructors exist in the academic crate, the admission flow does not take a guardian, and `ClassId` / `SectionId` / `AcademicYearId` are typed ids that already embed `SchoolId` (so `new(tenant.school_id())` is doubly wrong: `Id::new` takes `(school_id, uuid)`). The actual `AdmitStudentCommand` requires `student_id`, `admission_no: String`, `date_of_birth`, `gender`, `admission_date`, `class_id`, `section_id`, `academic_year_id` (a typed id), etc. — there is no `AcademicYear::current` and no `GuardianSpec`.

**Expected:**

`engine.students().admit(AdmitStudentCommand { admission_no: AdmissionNumber::new("ADM-2026-0001")?, ..., guardian: GuardianSpec { ... }, class_id: ClassId::new(tenant.school_id()), ... academic_year: AcademicYear::current(...), })` per `docs/library-docs.md:43-65`.

**Evidence:**

- `docs/library-docs.md:48-64` — the full `AdmitStudentCommand { ... }` literal.
  - `crates/domains/academic/src/commands.rs:62-106` — `pub struct AdmitStudentCommand` with fields `tenant: TenantContext, student_id: StudentId, admission_no: String, first_name: String, last_name: String, date_of_birth: NaiveDate, gender: ..., blood_group: Option<...>, religion: Option<String>, caste: Option<String>, mobile: Option<String>, email: Option<String>, current_address: Option<String>, permanent_address: Option<String>, admission_date: NaiveDate, class_id: ClassId, section_id: SectionId, academic_year_id: AcademicYearId, roll_no: Option<String>, custom_fields: BTreeMap<String, String>`. There is no `guardian: GuardianSpec` field and no `AdmissionNumber` field (it is `String`).
  - `crates/domains/academic/src/value_objects.rs:264-294` — `AdmissionNumber` exists as a wrapper type with `pub fn new(s: impl Into<String>) -> Result<Self>`, but the command takes `String`, not `AdmissionNumber`.
  - `crates/domains/academic/src/commands.rs:112-147` — `AdmitStudentCommand::new(...)` takes `tenant, student_id, admission_no: String, first_name, last_name, date_of_birth, gender, admission_date, class_id, section_id, academic_year_id` — no guardian, no `AcademicYear::current`.
  - `crates/domains/academic/src/lib.rs:67, 86-92` — no `GuardianSpec`, `GuardianRelation`, `AcademicYear::current`, `PhoneNumber`, `EmailAddress` re-exported from the academic crate.

---

### FINDING 7 (id: `DOC-LIB-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/library-docs.md:190-201`

**Description:**

The "Error Handling" example pattern-matches `DomainError::Validation { field, reason }`, `DomainError::Conflict { entity, reason }`, `DomainError::NotFound { entity, id }`, `DomainError::Forbidden { reason }`, `DomainError::Infrastructure(source)`. The actual `DomainError` enum has none of these tuple-struct-style or named-struct-style variants — every variant takes a single `String` (or a boxed error for `Infrastructure`). Variants `Validation(String)`, `NotFound(String)`, `Conflict(String)`, `Forbidden(String)`, `TenantViolation(String)`, `Infrastructure(Box<dyn Error + Send + Sync>)`, `NotSupported(String)`.

**Expected:**

`Err(DomainError::Validation { field, reason })` and friends per `docs/library-docs.md:195-199`.

**Evidence:**

- `docs/library-docs.md:195-199` — `Err(DomainError::Validation { field, reason })`, `Err(DomainError::Conflict { entity, reason })`, `Err(DomainError::NotFound { entity, id })`, `Err(DomainError::Forbidden { reason })`, `Err(DomainError::Infrastructure(source))`.
  - `crates/infra/core/src/error.rs:18-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(Box<dyn Error + Send + Sync>), NotSupported(String) }` — every body is a single `String` or boxed error; no struct-like named-field variants.

---

### FINDING 8 (id: `DOC-LIB-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/library-docs.md:208-218`

**Description:**

The "Sample Programs" section claims an `examples/admit_and_enroll.rs` exists in the workspace. No such file exists anywhere in the repo (the only `examples/` directories are inside `target/` build outputs and inside `schoolify/vendor/`).

**Expected:**

"A complete `examples/admit_and_enroll.rs` is provided in the workspace that..." per `docs/library-docs.md:210`.

**Evidence:**

- `docs/library-docs.md:210-218` — the sample-programs claim.
  - No `examples/admit_and_enroll.rs` anywhere in the repo: `find /home/beznet/Workspace/smscore -name "examples" -type d` returns only `target/debug/examples` and `schoolify/vendor/**/examples`. No crate under `crates/` ships an `examples/` directory.

---

### FINDING 9 (id: `DOC-LIB-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/library-docs.md:53, 58-59`

**Description:**

The "Calling a Command" example uses `NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` directly inline. The engine's code standards forbid `unwrap()` in production paths (per `AGENTS.md` § "Type Safety" and `docs/code-standards.md`); the example contradicts the engine's own invariant.

**Expected:**

`NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` per `docs/library-docs.md:53`.

**Evidence:**

- `docs/library-docs.md:53` — `date_of_birth: NaiveDate::from_ymd_opt(2010, 12, 10).unwrap(),`
  - `AGENTS.md` § "Type Safety" — "No `unwrap()` or `expect()` in production paths. Propagate errors via `?` or document the invariant that makes panic impossible."

---


## Query Layer (target id prefix: `DOC-QL`)

**Path:** `docs/query_layer.md`  
**Total findings:** 8 (0 critical, 6 high, 2 medium, 0 low)


### FINDING 12 (id: `DOC-QL-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:241-245` vs `crates/infra/query-derive/src/lib.rs:634-644`

**Description:**

The doc shows the macro as emitting a generic `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>`. The macro does emit a method by that name, but it is a no-op stub: it discards both arguments (`let _ = relation; let _ = __build;`) and returns `self` unchanged without pushing any `QueryNode::HasRelation` onto `self.filters`. The typed method `where_has_<Relation>` (e.g. `where_has_Parent`) is what actually adds a `HasRelation` node.

**Expected:**

Per `docs/query_layer.md:241-245`: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` — the closure is invoked, the result is compiled to a `QueryNode`, and the node is wrapped in `QueryNode::HasRelation` and pushed onto `self.filters`.

**Evidence:**

- `docs/query_layer.md:241-245` — `pub fn where_has<R, F>(self, relation: R, build: F) -> Self\n    where\n        R: Into<StudentRelation>,\n        F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`
  - `crates/infra/query-derive/src/lib.rs:634-644` — `let generic_where_has = quote! { #struct_vis fn where_has<__R, __F>(mut self, relation: __R, __build: __F) -> Self where __R: ::std::convert::Into<::educore_core::query::Relation>, __F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField, { let _ = relation; let _ = __build; self } };` — both arguments discarded.

---

### FINDING 13 (id: `DOC-QL-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:130-138` vs `crates/infra/query-derive/src/lib.rs:406-431`

**Description:**

The doc shows `pub struct StudentQueryBuilder { school_id: SchoolId, filters: Vec<QueryNode<StudentField>>, orders: Vec<OrderNode<StudentField>>, offset: u32, limit: u32, relations: BTreeSet<StudentRelation>, }`. The actual macro emits `school_id: Option<SchoolId>` (not `SchoolId`) and `limit: Option<u32>` (not `u32`), and omits the `relations: BTreeSet<StudentRelation>` field when no relations are declared (only added in the `relations.is_empty()` else-branch).

**Expected:**

`pub struct StudentQueryBuilder { school_id: SchoolId, ..., offset: u32, limit: u32, relations: BTreeSet<StudentRelation>, }` per `docs/query_layer.md:130-138`.

**Evidence:**

- `docs/query_layer.md:130-138` — full builder struct definition.
  - `crates/infra/query-derive/src/lib.rs:411-417` — `school_id: ::std::option::Option<::educore_core::ids::SchoolId>`, `limit: ::std::option::Option<u32>` — both are `Option`, not bare types.
  - `crates/infra/query-derive/src/lib.rs:427-430` — `relations: ::std::collections::BTreeSet<#relation_enum_name>` is added in the `relations.is_empty()` else-branch only; structs without relations emit no `relations` field at all (the doc shows it unconditionally).

---

### FINDING 14 (id: `DOC-QL-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:213, 332-335` vs `crates/infra/core/src/query.rs:331-357`

**Description:**

The doc types the `OrderNode` as `OrderNode<F: FieldKind>` (line 213: `pub struct OrderNode<F: FieldKind> { pub field: F, pub direction: OrderDirection, }`) and the related builder as `RelatedQueryBuilder<R>` (line 244). Neither `FieldKind` nor `RelatedQueryBuilder` exists in the engine. The actual trait bound is `OrderNode<F: Field>` (the `Field` trait in `educore-core::query`), and the typed where-has passes `RelatedField` (a unit placeholder struct), not a builder type.

**Expected:**

`pub struct OrderNode<F: FieldKind>` and `FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` per `docs/query_layer.md:213, 244`.

**Evidence:**

- `docs/query_layer.md:213` — `pub struct OrderNode<F: FieldKind> { pub field: F, pub direction: OrderDirection, }`
  - `docs/query_layer.md:244` — `F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`
  - `crates/infra/core/src/query.rs:34` — `pub trait Field: Clone + Copy + PartialEq + Eq + Hash + fmt::Debug { fn column_name(self) -> &'static str; }` — the trait is `Field`, not `FieldKind`.
  - `crates/infra/core/src/query.rs:331-337` — `pub struct OrderNode<F: Field> { pub field: F, pub direction: OrderDirection, }` — bounded by `Field`.
  - `crates/infra/core/src/query.rs:379-392` — `pub struct RelationalField;` (a unit struct), used as the field type inside `QueryNode::HasRelation`. There is no `RelatedQueryBuilder<R>`.
  - `crates/infra/query-derive/src/lib.rs:638` — the actual generic `where_has` constraint: `__F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField,` — `RelationalField`, not `RelatedQueryBuilder`.

---

### FINDING 15 (id: `DOC-QL-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:228-246` vs `crates/infra/query-derive/src/lib.rs:484-552`

**Description:**

The doc shows `where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self` (and `where_between` with `FieldValue` args). The macro emits `where_in<V>(self, field: FieldEnum, values: Vec<V>) -> Self where V: Into<Value>` and `where_between<V>(self, field: FieldEnum, lo: V, hi: V) -> Self where V: Into<Value>`. The element type is generic over `Into<Value>`, not a fixed `FieldValue` (which is not a name that exists in the codebase — the type is `Value`).

**Expected:**

`pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self;` and `pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self;` per `docs/query_layer.md:235-236`.

**Evidence:**

- `docs/query_layer.md:235-236` — `pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self;` / `pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self;`
  - `crates/infra/query-derive/src/lib.rs:484-512` — `pub fn where_in<V>(mut self, field: #field_enum_name, values: ::std::vec::Vec<V>) -> Self where V: ::std::convert::Into<::educore_core::query::Value>` — generic `Into<Value>`, not `Vec<FieldValue>`.
  - `crates/infra/core/src/query.rs:63-87` — `pub enum Value` is the runtime filter value type; no `FieldValue` alias.

---

### FINDING 16 (id: `DOC-QL-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:484-498, 502-514` vs `crates/infra/core/src/query.rs:398-412` and `crates/infra/query-derive/src/lib.rs`

**Description:**

The doc claims (1) the macro emits a `StudentAggregate` enum and an `aggregate(StudentAggregate::Count).group_by(...).execute()` method chain, and (2) the runtime `pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }` carries items + total + is generic over `T`. Neither exists. The macro emits no aggregate methods, no `StudentAggregate` enum. The runtime `Page` struct is non-generic with only `offset: u32` and `limit: u32` (no `items`, no `total`).

**Expected:**

`pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }` per `docs/query_layer.md:505-510`; `engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).where_eq(...).execute().await?` per `docs/query_layer.md:486-493`.

**Evidence:**

- `docs/query_layer.md:505-510` — `pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }`
  - `docs/query_layer.md:486-493` — the full aggregate/group_by/execute chain.
  - `crates/infra/core/src/query.rs:398-411` — `pub struct Page { pub offset: u32, pub limit: u32, }` — non-generic, no `items`, no `total`.
  - `crates/infra/query-derive/src/lib.rs:570-586` — the macro emits `limit`, `offset`, `page` methods only; no `aggregate`, no `group_by`, no `StudentAggregate` (no `aggregate` text appears anywhere in `crates/infra/query-derive/src/lib.rs`).

---

### FINDING 17 (id: `DOC-QL-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/query_layer.md:516-527` vs `crates/infra/query-derive/src/lib.rs:803-807, 588-597`

**Description:**

The doc states the `StudentQueryBuilder` "is constructed only via `StudentQuery::new(school_id)`" and "The default constructor is private. A query that omits the school id is a compile error." In the actual macro, the builder is constructed via `new()` (no args, public) and `school_id` is `Option<SchoolId>` — the user calls `.for_school(school_id)` after `new()`, and the macro raises a runtime error from `build_query_node()` if `for_school` was never called, not a compile error.

**Expected:**

Per `docs/query_layer.md:518-527`: `let q = StudentQueryBuilder::new(tenant.school_id()).where_eq(...)` — required `school_id` argument, no default `new()`.

**Evidence:**

- `docs/query_layer.md:518-520` — "The `StudentQueryBuilder` is constructed only via `StudentQuery::new(school_id)`. The default constructor is private."
  - `docs/query_layer.md:524-527` — `let q = StudentQueryBuilder::new(tenant.school_id()).where_eq(StudentField::Status, StudentStatus::Active);`
  - `crates/infra/query-derive/src/lib.rs:803-807` — `pub fn new() -> Self { Self::default() }` — a public zero-arg `new()`.
  - `crates/infra/query-derive/src/lib.rs:588-597` — `pub fn for_school(mut self, school_id: ::educore_core::ids::SchoolId) -> Self { self.school_id = ::std::option::Option::Some(school_id); self }` — `for_school` is separate from `new`.
  - `crates/infra/query-derive/src/lib.rs:773-801` — `build_query_node` returns `Err(DomainError::validation(...))` if `self.school_id.is_none()`; the gate is a runtime check, not a compile error.

---

### FINDING 18 (id: `DOC-QL-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/query_layer.md:529-540` vs `crates/infra/query-derive/src/lib.rs` (no `StudentCursor`/`next_page`)

**Description:**

The doc shows `StudentCursor::after(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())` and `engine.students().next_page(cursor, 100).await?`. Neither `StudentCursor` nor `next_page` is emitted by the macro. There is no `Cursor` type in `crates/infra/query-derive/src/lib.rs` or `crates/infra/core/src/query.rs`.

**Expected:**

`StudentCursor::after(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())` and `engine.students().next_page(cursor, 100).await?` per `docs/query_layer.md:535-538`.

**Evidence:**

- `docs/query_layer.md:535-540` — the full cursor example.
  - `grep "StudentCursor\|next_page\|Cursor" crates/infra/query-derive/src/lib.rs crates/infra/core/src/query.rs` — no matches for `StudentCursor` or `next_page`; only `crates/infra/core/src/query.rs` has a `C` cursor type in trait bounds (unrelated).

---

### FINDING 19 (id: `DOC-QL-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/query_layer.md:430-437` vs `crates/infra/query-derive/src/lib.rs:444-453` and `crates/infra/core/src/query.rs:445-475`

**Description:**

The doc shows `async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>` as the repository signature. The macro does not emit a `StudentQuery` value type at all — the macro emits `StudentQueryBuilder` (a builder), and `build_query_node()` returns `(QueryNode<StudentField>, Page)`. The `to_relational_node` conversion the macro uses for `HasRelation` lives in `educore_core::query::to_relational_node` (a free function), not on a trait method.

**Expected:**

`async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;` per `docs/query_layer.md:447`.

**Evidence:**

- `docs/query_layer.md:445-453` — `#[async_trait]\npub trait StudentRepository: Send + Sync {\n    async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;\n    ...`
  - `crates/infra/query-derive/src/lib.rs:773-801` — `pub fn build_query_node(self) -> ::educore_core::error::Result<(::educore_core::query::QueryNode<#field_enum_name>, ::educore_core::query::Page)>` — the macro emits a builder + a builder-to-AST conversion, not a `StudentQuery` value type.
  - `crates/infra/core/src/query.rs:448-475` — `pub fn to_relational_node<F: Field>(node: QueryNode<F>) -> QueryNode<RelationalField>` — a free function.

---


## Phase Handoffs (target id prefix: `DOC-HO`)

**Path:** `docs/handoff/PHASE-*-HANDOFF.md`  
**Total findings:** 10 (0 critical, 6 high, 2 medium, 2 low)


### FINDING 20 (id: `DOC-HO-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-3-HANDOFF.md:7, 61, 66-79, 100-103, 327, 335`

**Description:**

PHASE-3 handoff headline claims "23 typed commands, 19 typed events" but the code has 22 typed commands and 23 typed events. The handoff's "What's wired and working" body (lines 61-92) and footer ("crates/domains/academic/src/events.rs — 19 typed events") repeat the 19-events / 23-commands claim, but the actual file counts are inverted.

**Expected:**

Per `docs/handoff/PHASE-3-HANDOFF.md:7`: "5 aggregates (Student, Class, Section, Subject, AcademicYear), 23 typed commands, 19 typed events".

**Evidence:**

- `docs/handoff/PHASE-3-HANDOFF.md:7` — "5 aggregates (Student, Class, Section, Subject, AcademicYear), 23 typed commands, 19 typed events".
  - `docs/handoff/PHASE-3-HANDOFF.md:61` — "**23 typed commands** (8 student lifecycle, 4 class CRUD, 3 section CRUD, 3 subject CRUD, 5 academic-year CRUD)".
  - `docs/handoff/PHASE-3-HANDOFF.md:66` — "**19 typed events** implementing".
  - `crates/domains/academic/src/commands.rs` — `grep -c "^pub struct " crates/domains/academic/src/commands.rs` = **22** (Admit, Update, Suspend, Reinstate, Withdraw, Transfer, Promote, Graduate, CreateClass, UpdateClass, SetOptionalSubjectGpaThreshold, DeleteClass, CreateSection, UpdateSection, DeleteSection, CreateSubject, UpdateSubject, DeleteSubject, CreateAcademicYear, UpdateAcademicYearDates, SetCurrentAcademicYear, CloseAcademicYear).
  - `crates/domains/academic/src/events.rs` — `grep -c "^pub struct " crates/domains/academic/src/events.rs` = **23** (8 student lifecycle + 4 class events + 3 section events + 3 subject events + 5 academic-year events).

---

### FINDING 21 (id: `DOC-HO-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-3-HANDOFF.md:72-79, 335-336`

**Description:**

The handoff claims "19 pure factory services" and the matching crate-level prelude asserts the same count, but `services.rs` ships 23 `pub fn` factory functions plus 1 helper (`school_matches`). The handoff and the crate docstring both undercount by 4.

**Expected:**

"**19 pure factory services**" per `docs/handoff/PHASE-3-HANDOFF.md:72`.

**Evidence:**

- `docs/handoff/PHASE-3-HANDOFF.md:72-73` — "**19 pure factory services** (mirror `educore-platform::services::create_school` exactly)."
  - `crates/domains/academic/src/lib.rs:83-92` — `pub use crate::services::{admit_student, ... 24 services re-exported }` — the prelude re-exports 24 functions.
  - `crates/domains/academic/src/services.rs` — `grep -c "^pub fn " crates/domains/academic/src/services.rs` = **24** (23 factory fns + `school_matches` helper).

---

### FINDING 23 (id: `DOC-HO-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-9-HANDOFF.md:5-10, 81-94`

**Description:**

PHASE-9 handoff claims "18 events + 18 typed commands + 6 service factories" but the actual file counts are 19 events and 22 commands. The handoff's own narrative at lines 84-91 lists **19 event names** (`BookCategoryCreated`, `BookCategoryUpdated`, ..., `FineWaived`) — inconsistent with the headline "18 events". 6 service factories claim matches the actual `services.rs`.

**Expected:**

Per `docs/handoff/PHASE-9-HANDOFF.md:5-10`: "6 aggregates + 3 child entities + 18 events + 18 typed commands + 6 service factories".

**Evidence:**

- `docs/handoff/PHASE-9-HANDOFF.md:5-10` — "6 aggregates + 3 child entities + 18 events + 18 typed commands + 6 service factories + 6 repository ports + 6 query stubs + the `FineCalculationService`".
  - `docs/handoff/PHASE-9-HANDOFF.md:81-91` — the body lists **19** event names: `BookCategoryCreated, BookCategoryUpdated, BookCategoryDeleted, BookAdded, BookUpdated, BookDeleted, BookQuantityAdjusted, LibraryMemberRegistered, LibraryMemberUpdated, LibraryMemberDeactivated, LibraryMemberReactivated, LibraryMemberDeleted, BookIssued, BookReturned, BookRenewed, BookMarkedLost, BookReturnRecorded, FineCalculated, FineWaived`.
  - `crates/domains/library/src/events.rs` — `grep -c "^pub struct " crates/domains/library/src/events.rs` = **19**.
  - `crates/domains/library/src/commands.rs` — `grep -c "^pub struct " crates/domains/library/src/commands.rs` = **22** (the handoff claims 18).

---

### FINDING 25 (id: `DOC-HO-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-12-HANDOFF.md:6-23, 27-42`

**Description:**

PHASE-12 handoff opens with "**20 root aggregates**" but the file ships 19 root aggregates. The handoff itself enumerates only 19 names ("Page, News, NewsCategory, NewsComment, NewsPage, NoticeBoard, Testimonial, HomeSlider, SpeechSlider, Content, ContentType, ContentShareList, TeacherUploadContent, UploadContent, AboutPage, ContactPage, CoursePage, HomePageSetting, FrontendPage"). Repository traits match (19), query stubs match (19) — but the headline "20 root aggregates" and the "20 query stubs" sub-claim are wrong.

**Expected:**

Per `docs/handoff/PHASE-12-HANDOFF.md:6-7` and `:27-28`: "**20 root aggregates**".

**Evidence:**

- `docs/handoff/PHASE-12-HANDOFF.md:7-13` — "all 20 root aggregates per `docs/specs/cms/aggregates.md` (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, `UploadContent`, `AboutPage`, `ContactPage`, `CoursePage`, `HomePageSetting`, `FrontendPage`)" — only **19** names listed despite "20 root aggregates".
  - `docs/handoff/PHASE-12-HANDOFF.md:27-28` — "**20 root aggregates** ship as first-class ports".
  - `docs/handoff/PHASE-12-HANDOFF.md:457` — "`crates/domains/cms/src/query.rs` — 19 typed query stubs" — sub-claim is 19, not 20.
  - `crates/domains/cms/src/aggregate.rs` — `grep "^pub struct \w\+ {$" crates/domains/cms/src/aggregate.rs | wc -l` after filtering out `NewPage / NewNews / ...` DTOs yields **19** main aggregates.
  - `crates/domains/cms/src/repository.rs` — `grep "pub trait " | wc -l` = **19** traits.
  - `crates/domains/cms/src/query.rs` — `grep "^pub struct" | grep -c "Query"` = **19** query stubs.

---

### FINDING 26 (id: `DOC-HO-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-12-HANDOFF.md:13-17, 30-32`

**Description:**

The handoff claims "~67 typed commands" and "10 `CMS_*_COMMAND_TYPE` constants", but the file ships exactly **10 command structs** (not "~67"). The 67 count matches the events file (and the handoff correctly says "~67 typed events"), so the "~67 typed commands" claim is a copy/paste error from the events row.

**Expected:**

"~67 typed commands" per `docs/handoff/PHASE-12-HANDOFF.md:30-32`.

**Evidence:**

- `docs/handoff/PHASE-12-HANDOFF.md:30-32` — "**~67 typed commands** with the matching `<Domain>.<Aggregate>.<Action>` wire form (10 `CMS_*_COMMAND_TYPE` constants; the headline factory fns)."
  - `crates/domains/cms/src/commands.rs` — `grep -c "^pub struct " crates/domains/cms/src/commands.rs` = **10** (`CreatePageCommand`, `PublishPageCommand`, `ArchivePageCommand`, `DeletePageCommand`, `CreateNewsCommand`, `CreateTestimonialCommand`, `CreateHomeSliderCommand`, `CreateContentCommand`, `CreateContentShareListCommand`, `ConfigureHomePageCommand`).
  - `crates/domains/cms/src/commands.rs` `CMS_*_COMMAND_TYPE` consts (lines 442-451): **10** constants, matching the 10 command structs.

---

### FINDING 27 (id: `DOC-HO-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-12-HANDOFF.md:13-17`

**Description:**

The handoff asserts "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)". In the actual file there are **19 repository traits, including a separate `SpeechSliderRepository`** — `SpeechSlider` does NOT share the home-slider pattern, it has its own first-class trait.

**Expected:**

Per `docs/handoff/PHASE-12-HANDOFF.md:15-17`: "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)".

**Evidence:**

- `docs/handoff/PHASE-12-HANDOFF.md:13-17` — "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)".
  - `crates/domains/cms/src/repository.rs:279` — `pub trait SpeechSliderRepository: Send + Sync {` — a separate, first-class `SpeechSliderRepository` exists.
  - `crates/domains/cms/src/repository.rs` total `pub trait` count = **19**, matching the headline, but the parenthetical "except `SpeechSlider` shares the home-slider pattern" is incorrect.

---

### FINDING 28 (id: `DOC-HO-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-13-HANDOFF.md:25-27`

**Description:**

The handoff headline claims "**5 service factory structs**" but then lists six names in parentheses (`CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService`), and `services.rs` actually defines 6 structs. The body sub-section "### 5 service factory structs" at line 130 has the same inconsistency.

**Expected:**

"**5 service factory structs** (CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService)" per `docs/handoff/PHASE-13-HANDOFF.md:25-27`.

**Evidence:**

- `docs/handoff/PHASE-13-HANDOFF.md:25-27` — "**5 service factory structs** (CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService) + 1 `WeekendChange` enum." — six names in the parenthetical despite "5" headline.
  - `crates/cross-cutting/events-domain/src/services.rs` — `grep "^pub struct" crates/cross-cutting/events-domain/src/services.rs` returns **6** structs (`CalendarService`, `RecurrenceService`, `HolidayService`, `CalendarSettingService`, `IncidentService`, `WeekendService`).

---

### FINDING 29 (id: `DOC-HO-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-16-HANDOFF.md:148-151`

**Description:**

The handoff claims "**59 unit tests** pass (53 in `storage`, 3 in `auth`, 6 in `notify`, 4 in `payment`, 8 in `files`, 3 in `integrations`, 1 in `sync`, 1 in `event_bus`, 3 in `lib`)." Summing the per-file breakdown yields 53+3+6+4+8+3+1+1+3 = **82**, not 59, and the actual file counts (14 in storage.rs, 6 in auth.rs, 6 in notify.rs, 8 in payment.rs, 0 in files.rs, 2 in integrations.rs, 2 in sync.rs, 1 in event_bus.rs, 3 in lib.rs) sum to 42. Neither 82 nor 42 matches the 59 headline.

**Expected:**

"**59 unit tests** pass" per `docs/handoff/PHASE-16-HANDOFF.md:148`.

**Evidence:**

- `docs/handoff/PHASE-16-HANDOFF.md:148-151` — "**59 unit tests** pass (53 in `storage`, 3 in `auth`, 6 in `notify`, 4 in `payment`, 8 in `files`, 3 in `integrations`, 1 in `sync`, 1 in `event_bus`, 3 in `lib`)."
  - `crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,sync,event_bus,lib,errors}.rs` `#[test]` counts: storage.rs=14, auth.rs=6, notify.rs=6, payment.rs=8, files.rs=0, integrations.rs=2, sync.rs=2, event_bus.rs=1, lib.rs=3, errors.rs=0 → **42** total.

---

### FINDING 22 (id: `DOC-HO-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-3-HANDOFF.md:18-20, 100-103`

**Description:**

The handoff claims "Phase 3 adds 66 unit tests in `educore-academic`" but `grep -c "#\[test\]" crates/domains/academic/src/*.rs` totals **67** (4 lib + 2 entities + 15 services + 5 query + 4 events + 6 aggregate + 19 value_objects + 10 commands = 67).

**Expected:**

"66 unit tests" per `docs/handoff/PHASE-3-HANDOFF.md:18-20`.

**Evidence:**

- `docs/handoff/PHASE-3-HANDOFF.md:18-20` — "Phase 3 adds 66 unit tests in `educore-academic`".
  - `docs/handoff/PHASE-3-HANDOFF.md:100-103` — "**66 unit tests** in `educore-academic`".
  - Sum across `crates/domains/academic/src/{lib,aggregate,entities,value_objects,commands,events,services,query,repository}.rs`: 4 + 6 + 2 + 19 + 10 + 4 + 15 + 5 + 0 = **67** `#[test]` attributes.

---

### FINDING 24 (id: `DOC-HO-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-3.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/handoff/PHASE-9-HANDOFF.md:16-19`

**Description:**

The handoff claims "**31 passed**" unit tests in `educore-library`, but the actual `#[test]` count is 30.

**Expected:**

"**31 passed**" per `docs/handoff/PHASE-9-HANDOFF.md:16`.

**Evidence:**

- `docs/handoff/PHASE-9-HANDOFF.md:16-19` — "`cargo test -p educore-library --lib` — **31 passed**".
  - Sum across `crates/domains/library/src/{lib,events,query,repository,services,value_objects}.rs`: 2 + 1 + 1 + 1 + 13 + 12 = **30** `#[test]` attributes.

---


## Port Contracts (target id prefix: `DOC-PORT`)

**Path:** `docs/ports/`  
**Total findings:** 16 (8 critical, 4 high, 3 medium, 1 low)


### FINDING 1 (id: `DOC-PORT-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/storage.md:214-229` (StorageError enum) vs `crates/infra/core/src/error.rs:19-63` (DomainError enum)

**Description:**

The storage port spec defines a dedicated `StorageError` enum with 9 variants (`Connection`, `Conflict`, `Deadlock`, `UniqueViolation`, `ForeignKey`, `Check`, `NotFound`, `Infrastructure`, `Timeout`, `SerializationFailure`). The engine actually has a single engine-wide `DomainError` enum with 7 variants (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `TenantViolation`, `Infrastructure`, `NotSupported`); there is no `StorageError` type anywhere in the workspace. The doc explicitly states the engine maps `StorageError::Infrastructure` to `DomainError::Infrastructure` and translates other variants to domain errors — i.e. the spec describes an adapter-facing error type that does not exist.

**Expected:**

`StorageError` enum per `docs/ports/storage.md:218-229` with the 10 variants above.

**Evidence:**

- `docs/ports/storage.md:218-229` — `pub enum StorageError { #[error("connection failed: {0}")] Connection(String), #[error("transaction conflict: {0}")] Conflict(String), #[error("deadlock detected")] Deadlock, #[error("unique violation: {0}")] UniqueViolation { constraint: String }, #[error("foreign key violation: {0}")] ForeignKey { constraint: String }, #[error("check constraint violation: {0}")] Check { constraint: String }, #[error("row not found")] NotFound, #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), #[error("timeout")] Timeout, #[error("serialization failure")] SerializationFailure, }`
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), NotSupported(String), }`
  - No `StorageError` symbol exists: `rg "pub enum StorageError|pub struct StorageError" crates/` returns 0 matches.

---

### FINDING 2 (id: `DOC-PORT-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/storage.md:17-89` (StorageAdapter trait) vs `crates/infra/storage/src/port.rs:34-150` (StorageAdapter trait)

**Description:**

The storage port spec shows the `StorageAdapter` trait carrying ~24 `fn xxx_repository(&self) -> Arc<dyn XxxRepository>` accessors (students, guardians, classes, sections, class_sections, subjects, class_subjects, academic_years, class_routines, homeworks, lessons, lesson_topics, lesson_plans, student_records, student_promotions, student_categories, student_groups, registration_fields, certificates, id_cards, admission_queries, class_rooms, class_times) plus the comment "…one handle per aggregate, across all 15 domains (~80+ total)". The actual `StorageAdapter` trait carries no repository accessors — only `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. Per the impl's source comment, the trait is intentionally minimal at Phase 0 and a generic `Repository<A>` (not 80+ per-aggregate traits) is the actual surface.

**Expected:**

`StorageAdapter` exposes `students() -> Arc<dyn StudentRepository>`, `guardians() -> Arc<dyn GuardianRepository>`, `classes() -> Arc<dyn ClassRepository>`, `sections() -> Arc<dyn SectionRepository>`, …, `class_times() -> Arc<dyn ClassTimeRepository>` and one handle per aggregate (~80+ total).

**Evidence:**

- `docs/ports/storage.md:25-50` — `fn students(&self) -> Arc<dyn StudentRepository>; fn guardians(&self) -> Arc<dyn GuardianRepository>; fn classes(&self) -> Arc<dyn ClassRepository>; fn sections(&self) -> Arc<dyn SectionRepository>; fn class_sections(&self) -> Arc<dyn ClassSectionRepository>; fn subjects(&self) -> Arc<dyn SubjectRepository>; fn class_subjects(&self) -> Arc<dyn ClassSubjectRepository>; fn academic_years(&self) -> Arc<dyn AcademicYearRepository>; fn class_routines(&self) -> Arc<dyn ClassRoutineRepository>; fn homeworks(&self) -> Arc<dyn HomeworkRepository>; fn lessons(&self) -> Arc<dyn LessonRepository>; fn lesson_topics(&self) -> Arc<dyn LessonTopicRepository>; fn lesson_plans(&self) -> Arc<dyn LessonPlanRepository>; fn student_records(&self) -> Arc<dyn StudentRecordRepository>; fn student_promotions(&self) -> Arc<dyn StudentPromotionRepository>; fn student_categories(&self) -> Arc<dyn StudentCategoryRepository>; fn student_groups(&self) -> Arc<dyn StudentGroupRepository>; fn registration_fields(&self) -> Arc<dyn RegistrationFieldRepository>; fn certificates(&self) -> Arc<dyn CertificateRepository>; fn id_cards(&self) -> Arc<dyn IdCardRepository>; fn admission_queries(&self) -> Arc<dyn AdmissionQueryRepository>; fn class_rooms(&self) -> Arc<dyn ClassRoomRepository>; fn class_times(&self) -> Arc<dyn ClassTimeRepository>; // ... one handle per aggregate, across all 15 domains (~80+ total)`
  - `crates/infra/storage/src/port.rs:34-150` — trait body lists only `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`; `fn students` / `fn guardians` / `fn classes` / etc. do not exist on this trait.
  - `crates/infra/storage/src/repository.rs:25-72` — single generic `pub trait Repository<A>: Send + Sync where A: Send + Sync + Clone + 'static { async fn get(...); async fn get_including_retired(...); async fn list(...); async fn count(...); async fn insert(...); async fn update(...); async fn soft_delete(...); }`; no `StudentRepository`, `GuardianRepository`, etc.

---

### FINDING 3 (id: `DOC-PORT-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/storage.md:159-166` (Outbox trait) vs `crates/infra/storage/src/outbox.rs:89-124` (Outbox trait)

**Description:**

The doc spec defines the `Outbox` sub-port as `append(envelope: EventEnvelope) -> Result<()>`, `pending(limit: u32) -> Result<Vec<EventEnvelope>>`, and `mark_published(ids: &[EventId]) -> Result<()>`. The actual `Outbox` trait in `crates/infra/storage/src/outbox.rs` accepts a `SerializedEnvelope` (a concrete, deserialize-owned row type), not `EventEnvelope`; the `EventEnvelope` is the bus-port type from `crates/cross-cutting/events/src/envelope.rs`. The impl also adds a 4th method `pending_count(school_id)` (defaulted) that is absent from the spec.

**Expected:**

Per `docs/ports/storage.md:162-164`, `Outbox::append(&self, envelope: EventEnvelope)` and `Outbox::pending(&self, limit: u32) -> Result<Vec<EventEnvelope>>`.

**Evidence:**

- `docs/ports/storage.md:159-166` — `pub trait Outbox: Send + Sync { async fn append(&self, envelope: EventEnvelope) -> Result<()>; async fn pending(&self, limit: u32) -> Result<Vec<EventEnvelope>>; async fn mark_published(&self, ids: &[EventId]) -> Result<()>; }`
  - `crates/infra/storage/src/outbox.rs:89-124` — `async fn append(&self, envelope: SerializedEnvelope) -> Result<()>;` (line 102), `async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>>;` (line 108), `async fn mark_published(&self, ids: &[EventId]) -> Result<()>;` (line 112), plus `async fn pending_count(&self, school_id: SchoolId) -> Result<u64>` (line 117).

---

### FINDING 4 (id: `DOC-PORT-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/storage.md:120-130` (Transaction trait) vs `crates/infra/storage/src/transaction.rs:32-92` (Transaction trait)

**Description:**

The doc spec defines `Transaction` as carrying `commit`, `rollback`, `repositories() -> &dyn TransactionalRepositories`, and `outbox() -> &dyn Outbox`. The actual `Transaction` trait carries `commit`, `rollback`, `outbox()`, `audit_log() -> &dyn AuditLog`, `idempotency() -> &dyn Idempotency`, `event_log() -> &dyn EventLog`, and `bulk_insert_student_attendances(...)`. There is no `repositories()` accessor and no `TransactionalRepositories` type anywhere in the workspace; in their place are three other sub-port accessors plus a bulk-insert convenience method.

**Expected:**

`Transaction` exposes `commit`, `rollback`, `repositories() -> &dyn TransactionalRepositories`, `outbox() -> &dyn Outbox`.

**Evidence:**

- `docs/ports/storage.md:122-130` — `pub trait Transaction: Send + Sync { async fn commit(self: Box<Self>) -> Result<()>; async fn rollback(self: Box<Self>) -> Result<()>; fn repositories(&self) -> &dyn TransactionalRepositories; fn outbox(&self) -> &dyn Outbox; }`
  - `crates/infra/storage/src/transaction.rs:32-92` — `async fn commit(...)` (line 43), `async fn rollback(...)` (line 47), `fn outbox(&self) -> &dyn Outbox;` (line 51), `fn audit_log(&self) -> &dyn AuditLog;` (line 55), `fn idempotency(&self) -> &dyn Idempotency;` (line 60), `fn event_log(&self) -> &dyn EventLog;` (line 64), `async fn bulk_insert_student_attendances(...)` (line 86).
  - `rg "TransactionalRepositories" crates/` returns 0 matches.

---

### FINDING 5 (id: `DOC-PORT-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/storage.md:204-212` (StudentRepository stream method) vs `crates/infra/storage/src/repository.rs:25-72` (Repository<A> trait)

**Description:**

The doc spec defines a `StudentRepository` trait with at least one method `stream(q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>`. The actual storage port does not expose per-aggregate repository traits and does not declare a `stream` method anywhere; the generic `Repository<A>` trait exposes `get`, `get_including_retired`, `list`, `count`, `insert`, `update`, `soft_delete`. Streaming is not part of the trait surface.

**Expected:**

`pub trait StudentRepository: Send + Sync { async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>; }`

**Evidence:**

- `docs/ports/storage.md:206-210` — `pub trait StudentRepository: Send + Sync { async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>; }`
  - `crates/infra/storage/src/repository.rs:25-72` — generic `Repository<A>` carries `get`, `get_including_retired`, `list`, `count`, `insert`, `update`, `soft_delete`; no `stream` method, no `StudentRepository` type, no `BoxStream` import.
  - `rg "pub trait StudentRepository|fn stream\(.*Query" crates/` returns 0 matches.

---

### FINDING 6 (id: `DOC-PORT-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/sync.md:36-46` (SyncAdapter trait) vs `crates/cross-cutting/sync/src/port.rs:37-75` (SyncAdapter trait)

**Description:**

The sync port spec defines `SyncAdapter` with four methods `dispatch`, `subscribe`, `snapshot`, `health`, taking `CommandEnvelope`, `EventFilter`, `SchoolId`, and `&self` respectively. The actual `SyncAdapter` trait carries a completely different five-method set: `start(school: SchoolId)`, `pause(school: SchoolId)`, `resume(school: SchoolId)`, `stop(school: SchoolId)`, and `health()`. None of `dispatch`, `subscribe`, `snapshot` exist on the trait; none of `start`, `pause`, `resume`, `stop` exist on the trait as documented. The supporting types `CommandEnvelope`, `EventFilter` (sync variant), `SchoolSnapshot`, and `EventStream` documented in the spec are also absent from the implementation.

**Expected:**

Per `docs/ports/sync.md:38-46`, `SyncAdapter` exposes `dispatch`, `subscribe`, `snapshot`, `health`.

**Evidence:**

- `docs/ports/sync.md:38-46` — `pub trait SyncAdapter: Send + Sync + std::fmt::Debug { async fn dispatch(&self, envelope: CommandEnvelope) -> Result<CommandOutcome>; async fn subscribe(&self, filter: EventFilter) -> Result<EventStream>; async fn snapshot(&self, school_id: SchoolId) -> Result<SchoolSnapshot>; async fn health(&self) -> Result<SyncHealth>; }`
  - `crates/cross-cutting/sync/src/port.rs:36-75` — `pub trait SyncAdapter: Send + Sync { async fn start(&self, school: SchoolId) -> Result<()>; async fn pause(&self, school: SchoolId) -> Result<()>; async fn resume(&self, school: SchoolId) -> Result<()>; async fn stop(&self, school: SchoolId) -> Result<()>; async fn health(&self) -> Result<SyncHealth>; }`
  - `crates/cross-cutting/sync/src/command.rs:25-39` defines a separate `SyncCommand` enum with `Start`, `Pause`, `Resume`, `Stop` variants; no `CommandEnvelope`, no `CommandOutcome`, no `SchoolSnapshot`, no `EventStream` types exist (`rg "pub struct CommandEnvelope|pub struct SchoolSnapshot|pub trait EventStream" crates/` returns 0 matches).

---

### FINDING 7 (id: `DOC-PORT-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/sync.md:210-224` (SyncHealth struct) vs `crates/cross-cutting/sync/src/health.rs:67-75` (SyncHealth struct)

**Description:**

The sync port spec defines `SyncHealth` with fields `reachable: bool`, `latency_ms: u32`, `server_version: &'static str`, and `server_schema_version: u32`. The actual `SyncHealth` struct in `crates/cross-cutting/sync/src/health.rs` has fields `status: SyncStatus` (an enum with `Running`/`Paused`/`Stopped` variants) and `last_event_at: Option<Timestamp>`. The two structs have no fields in common.

**Expected:**

Per `docs/ports/sync.md:212-218`, `SyncHealth { reachable: bool, latency_ms: u32, server_version: &'static str, server_schema_version: u32 }`.

**Evidence:**

- `docs/ports/sync.md:212-218` — `pub struct SyncHealth { pub reachable: bool, pub latency_ms: u32, pub server_version: &'static str, pub server_schema_version: u32, }`
  - `crates/cross-cutting/sync/src/health.rs:67-75` — `pub struct SyncHealth { pub status: SyncStatus, pub last_event_at: Option<Timestamp>, }` and `pub enum SyncStatus { Running, Paused, Stopped, }` at lines 23-34.

---

### FINDING 8 (id: `DOC-PORT-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/ports/authentication.md:88-99` (Engine.rbac method, RbacPort trait) vs `crates/tools/sdk/src/engine.rs` (Engine struct) vs `crates/adapters/auth/src/port.rs:306-326` (RbacPort trait)

**Description:**

The authentication port spec shows `impl Engine { pub fn rbac(&self) -> &dyn RbacPort { &*self.rbac_port } }` and defines the `RbacPort` trait in the engine's main surface. The actual `Engine` struct (`crates/tools/sdk/src/engine.rs:21-40`) does not carry a `rbac_port` field and does not expose a `rbac()` method; the available accessors are `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`. The `RbacPort` trait does exist in `crates/adapters/auth/src/port.rs:306-326` but is not wired into the engine.

**Expected:**

Per `docs/ports/authentication.md:89-92`, `Engine` carries a `rbac_port: Arc<dyn RbacPort>` field and exposes `pub fn rbac(&self) -> &dyn RbacPort`.

**Evidence:**

- `docs/ports/authentication.md:89-92` — `impl Engine { pub fn rbac(&self) -> &dyn RbacPort { &*self.rbac_port } }`
  - `crates/tools/sdk/src/engine.rs:20-40` — `pub struct Engine { storage: Arc<dyn StorageAdapter>, auth: Arc<dyn AuthProvider>, notify: Arc<dyn NotificationProvider>, payment: Arc<dyn PaymentProvider>, files: Arc<dyn FileStorage>, integrations: Arc<dyn IntegrationGateway>, bus: Arc<dyn EventBus>, clock: Arc<dyn Clock>, id_gen: Arc<dyn IdGenerator>, }`; no `rbac_port` field.
  - `crates/tools/sdk/src/engine.rs:48-146` lists only `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `rbac()` method.
  - `crates/adapters/auth/src/port.rs:306-326` defines the `RbacPort` trait but nothing references it from the SDK engine.

---

### FINDING 10 (id: `DOC-PORT-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/ports/event-bus.md:84-90` (EventSubscription trait) vs `crates/cross-cutting/events/src/event_bus.rs:69-85` (EventSubscription trait)

**Description:**

The doc spec defines `EventSubscription::ack(&mut self, event_id: EventId) -> Result<()>` and `EventSubscription::nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>`. The actual trait returns `Result<AckOutcome>` (an enum with `Accepted` / `Unknown` / `Failed` variants) for both methods. The return type is a structural drift.

**Expected:**

Per `docs/ports/event-bus.md:87-88`, `ack` returns `Result<()>`, `nack` returns `Result<()>`.

**Evidence:**

- `docs/ports/event-bus.md:84-90` — `pub trait EventSubscription: Send + Sync { async fn next(&mut self) -> Option<Result<EventEnvelope>>; async fn ack(&mut self, event_id: EventId) -> Result<()>; async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>; async fn close(self: Box<Self>) -> Result<()>; }`
  - `crates/cross-cutting/events/src/event_bus.rs:69-85` — `pub trait EventSubscription: Send + Sync { async fn next(&mut self) -> Option<Result<EventEnvelope>>; async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>; async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome>; async fn close(self: Box<Self>) -> Result<()>; }` plus `pub enum AckOutcome { Accepted, Unknown, Failed, }` at lines 53-61.

---

### FINDING 11 (id: `DOC-PORT-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/ports/notifications.md:196-206` (NotificationError::Infrastructure) vs `crates/adapters/notify/src/errors.rs:75-120` (NotificationError)

**Description:**

The doc spec defines `NotificationError::Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` carrying a boxed error source. The actual `NotificationError::Infrastructure(String)` carries a pre-rendered string (the source error's `Display` output); the variant does not derive `std::error::Error::source` because the boxed source has been collapsed to a string at construction time. The impl's variant order also differs (`MissingVariable` precedes `InvalidRecipient` per the spec but follows it in the impl, and the impl has no `RbacPort` variant order issue; the spec lists `TemplateNotFound`, `MissingVariable`, `InvalidRecipient`, `RateLimited`, `Provider`, `QuotaExceeded`, `Infrastructure`).

**Expected:**

`NotificationError::Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` per `docs/ports/notifications.md:205`.

**Evidence:**

- `docs/ports/notifications.md:197-206` — `pub enum NotificationError { #[error("template not found: {0}")] TemplateNotFound(NotificationTemplateId), #[error("missing variable: {0}")] MissingVariable(String), #[error("invalid recipient: {0}")] InvalidRecipient(String), #[error("rate limited")] RateLimited, #[error("provider error: {0}")] Provider(String), #[error("quota exceeded")] QuotaExceeded, #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `crates/adapters/notify/src/errors.rs:75-120` — `pub enum NotificationError { TemplateNotFound(NotificationTemplateId), MissingVariable(String), InvalidRecipient(String), RateLimited, Provider(String), QuotaExceeded, Infrastructure(String), }` (line 118 shows `Infrastructure(String)` with no `#[source]` attribute).
  - The file's doc comment at lines 71-74 explicitly notes the deviation: "The engine never stores a live source error chain across a port boundary — it logs the source via `tracing` immediately and serialises only the string representation, so the `Infrastructure` variant is itself a `String` (not a `Box<dyn Error>`)."

---

### FINDING 12 (id: `DOC-PORT-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/ports/authentication.md:13-20` (AuthProvider return type) vs `crates/adapters/auth/src/port.rs:259-289` (AuthProvider trait)

**Description:**

The doc spec writes `AuthProvider`'s `authenticate`, `validate`, `refresh` methods as returning `Result<Session>` (unqualified), with `Result` implicitly being `core::result::Result<T, std::io::Error>` or some other default. The actual trait returns `Result<Session, crate::errors::AuthError>` (and `Result<(), AuthError>` for `revoke`). The unqualified `Result` in the doc is the bus-port `educore_core::error::Result<T>` alias (which is `Result<T, DomainError>`), not the spec's implied error type. A consumer following the doc would compile mismatched error mappings.

**Expected:**

`async fn authenticate(&self, credential: Credential) -> Result<Session>` with a clearly specified error type (the doc does not name one but the surrounding `AuthError` enum implies `AuthError`).

**Evidence:**

- `docs/ports/authentication.md:13-19` — `pub trait AuthProvider: Send + Sync + std::fmt::Debug { async fn authenticate(&self, credential: Credential) -> Result<Session>; async fn validate(&self, token: &AuthToken) -> Result<Session>; async fn revoke(&self, token: &AuthToken) -> Result<()>; async fn refresh(&self, token: &AuthToken) -> Result<Session>; }`
  - `crates/adapters/auth/src/port.rs:259-289` — `pub trait AuthProvider: Send + Sync + std::fmt::Debug { async fn authenticate(&self, credential: Credential) -> Result<Session, crate::errors::AuthError>; async fn validate(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>; async fn revoke(&self, token: &AuthToken) -> Result<(), crate::errors::AuthError>; async fn refresh(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>; }` — every method carries an explicit `AuthError` error parameter.

---

### FINDING 9 (id: `DOC-PORT-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/ports/event-bus.md:183-192` (EventBusError enum) vs `crates/cross-cutting/events/src/errors.rs:18-45` (EventError enum)

**Description:**

The doc spec defines `EventBusError` with 5 variants: `TopicNotFound`, `SubscriptionClosed`, `PublishFailed`, `DeserializeFailed`, `Infrastructure`. The actual `EventError` enum (the type the bus port actually returns, per `crates/cross-cutting/events/src/event_bus.rs`) has 6 variants: `TopicNotFound`, `SubscriptionClosed`, `PublishFailed`, `DeserializeFailed`, `NotSupported`, `Infrastructure`. The `NotSupported` variant is undocumented in the spec; the doc's named enum `EventBusError` does not exist in the codebase.

**Expected:**

`EventBusError` enum per `docs/ports/event-bus.md:185-191` with 5 variants.

**Evidence:**

- `docs/ports/event-bus.md:184-191` — `pub enum EventBusError { #[error("topic not found: {0}")] TopicNotFound(Topic), #[error("subscription closed")] SubscriptionClosed, #[error("publish failed: {0}")] PublishFailed(String), #[error("deserialize failed: {0}")] DeserializeFailed(String), #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `crates/cross-cutting/events/src/errors.rs:18-45` — `pub enum EventError { TopicNotFound(Topic), SubscriptionClosed, PublishFailed(String), DeserializeFailed(String), NotSupported(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `rg "pub enum EventBusError" crates/` returns 0 matches.

---

### FINDING 13 (id: `DOC-PORT-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/ports/event-bus.md:199,219` (`engine.events()`) vs `crates/tools/sdk/src/engine.rs:107-109` (`engine.bus()`)

**Description:**

The event-bus port doc uses `engine.events().subscribe(SubscribeOptions { ... })` as the canonical worked example. The actual `Engine` struct exposes `bus() -> &Arc<dyn EventBus>` (singular noun), not `events()`. Consumers following the doc would receive a compile error.

**Expected:**

`engine.events().subscribe(...)` per `docs/ports/event-bus.md:199` and `:219`.

**Evidence:**

- `docs/ports/event-bus.md:199` — `let mut sub = engine.events().subscribe(SubscribeOptions {`
  - `docs/ports/event-bus.md:219` — `engine.events().subscribe(SubscribeOptions {`
  - `crates/tools/sdk/src/engine.rs:105-109` — `pub fn bus(&self) -> &Arc<dyn EventBus> { &self.bus }`
  - `rg "pub fn events" crates/` returns 0 matches.

---

### FINDING 14 (id: `DOC-PORT-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/ports/payments.md:202` (`engine.payments()`) vs `crates/tools/sdk/src/engine.rs:87-91` (`engine.payment()`)

**Description:**

The payment port doc uses `engine.payments().charge(ChargeRequest { ... })` as the canonical worked example. The actual `Engine` struct exposes `payment() -> &Arc<dyn PaymentProvider>` (singular noun), not `payments()`. Same drift as finding 13 — the accessor name does not match the doc.

**Expected:**

`engine.payments().charge(ChargeRequest { ... })` per `docs/ports/payments.md:202`.

**Evidence:**

- `docs/ports/payments.md:202` — `let receipt = engine.payments().charge(ChargeRequest {`
  - `crates/tools/sdk/src/engine.rs:87-91` — `pub fn payment(&self) -> &Arc<dyn PaymentProvider> { &self.payment }`
  - `rg "pub fn payments" crates/` returns 0 matches.

---

### FINDING 15 (id: `DOC-PORT-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/ports/payments.md:216` (`engine.fees().record_payment`) and `docs/ports/file-storage.md:171` (`engine.students().admit`)

**Description:**

The payment port doc shows `engine.fees().record_payment(RecordPaymentCommand { ... })` and the file-storage port doc shows `engine.students().admit(AdmitStudentCommand { ... })`. Neither `engine.fees()` nor `engine.students()` accessors exist in the SDK `Engine` struct or anywhere else in the workspace. Domain facades (`admission`, `attendance`, `payment_svc`, `notify_svc` in `crates/tools/sdk/src/engine.rs:125-145`) are exposed as separate service handles, not as `engine.<domain>()`.

**Expected:**

`engine.fees().record_payment(...)` and `engine.students().admit(...)`.

**Evidence:**

- `docs/ports/payments.md:216` — `engine.fees().record_payment(RecordPaymentCommand {`
  - `docs/ports/file-storage.md:171` — `let student = engine.students().admit(AdmitStudentCommand {`
  - `crates/tools/sdk/src/engine.rs:48-146` lists `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `fees()`, no `students()`.
  - `rg "pub fn fees\b|pub fn students\b" crates/` returns 0 matches.

---

### FINDING 16 (id: `DOC-PORT-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/ports/storage.md:438-454` (worked example) vs `crates/tools/sdk/src/engine.rs:194-282` (EngineBuilder)

**Description:**

The storage port doc's worked example shows a fully wired engine that does not exist in the SDK crate. `Engine::builder()` is referenced (line 447), but the actual builder is `EngineBuilder::new()` (`crates/tools/sdk/src/engine.rs:179`). The builder ports in the doc's example (`storage`, `auth`, `notify`, `event_bus`) do not exhaustively match the actual builder's required ports (which also requires `payment`, `files`, `integrations`, `clock`, `id_gen` per `crates/tools/sdk/src/engine.rs:258-281`). The doc's example would fail to compile if transcribed verbatim.

**Expected:**

The worked example should produce a compilable engine.

**Evidence:**

- `docs/ports/storage.md:447-454` — `let engine = Engine::builder() .storage(storage.clone()) .auth(auth_provider) .notify(notify_provider) .event_bus(InProcessBus::new()) .build() .await?;`
  - `crates/tools/sdk/src/engine.rs:179-281` — `pub fn new() -> Self` and `pub fn build(self) -> Result<Engine, SdkError>` requiring every port: `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen` (`let storage = self.storage.ok_or(SdkError::MissingPort("storage"))?; … let id_gen = self.id_gen.ok_or(SdkError::MissingPort("id_gen"))?;` at lines 259-269).
  - `rg "Engine::builder|fn builder\b" crates/` returns 0 matches — the API is `EngineBuilder::new()`, not `Engine::builder()`.

---


## Command/Event Catalogs (target id prefix: `DOC-CAT`)

**Path:** `docs/commands/*.md + docs/events/*.md`  
**Total findings:** 19 (0 critical, 9 high, 8 medium, 2 low)


### FINDING 17 (id: `DOC-CAT-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/commands/academic.md:17-92` (table) vs `crates/domains/academic/src/commands.rs:64-461`

**Description:**

The academic command catalog documents 73 distinct command names (e.g. `AddStudentToGroup`, `AddSubTopicToLessonPlan`, `AssignClassRoom`, `AssignClassTeacher`, `AssignOptionalSubject`, `AssignStudentToSection`, `AssignSubjectTeacher`, `AssignSubjectToClass`, `CancelHomework`, `ChangeStudentCategory`, `ConvertAdmissionQuery`, `CreateCertificate`, `CreateClassRoutine`, `CreateClassSection`, `CreateHomework`, `CreateIdCard`, `CreateLesson`, `CreateLessonPlan`, `CreateLessonTopic`, `CreateRegistrationField`, `CreateStudentCategory`, `CreateStudentGroup`, `DeleteCertificate`, `DeleteClassRoutine`, `DeleteClassSection`, `DeleteIdCard`, `DeleteLesson`, `DeleteLessonPlan`, `DeleteLessonTopic`, `DeleteRegistrationField`, `DeleteStudentCategory`, `DeleteStudentGroup`, `EvaluateHomework`, `FollowUpAdmissionQuery`, `MarkLessonPlanCompleted`, `MarkLessonTopicCompleted`, `ReassignSubjectTeacher`, `RegisterAdmissionQuery`, `RemoveStudentFromGroup`, `SubmitHomework`, `SwapClassRoutinePeriods`, `UnassignSubjectFromClass`, `UpdateCertificate`, `UpdateClassRoutinePeriod`, `UpdateHomework`, `UpdateIdCard`, `UpdateLesson`, `UpdateLessonPlan`, `UpdateRegistrationField`, `UpdateStudentCategory`, `UpdateStudentGroup`, `UploadStudentDocument`). The Phase-3 implementation only ships 22 command structs (`AdmitStudent`, `CloseAcademicYear`, `CreateAcademicYear`, `CreateClass`, `CreateSection`, `CreateSubject`, `DeleteClass`, `DeleteSection`, `DeleteSubject`, `GraduateStudent`, `PromoteStudent`, `ReinstateStudent`, `SetCurrentAcademicYear`, `SetOptionalSubjectGpaThreshold`, `SuspendStudent`, `TransferStudent`, `UpdateAcademicYearDates`, `UpdateClass`, `UpdateSection`, `UpdateStudentProfile`, `UpdateSubject`, `WithdrawStudent`). 51 catalog commands have no `*Command` struct in the crate.

**Expected:**

Per `docs/commands/academic.md:17-92`, 73 academic command structs in `crates/domains/academic/src/commands.rs`.

**Evidence:**

- `docs/commands/academic.md:17-92` enumerates 73 `| \`XxxYyy\`` rows; awk-extracted unique command names = 73 (`awk -F"\`" '$2 ~ /^[A-Z][a-zA-Z]+$/ && $4 ~ /^[A-Z][a-zA-Z]+\.[A-Z]/ {print $2}' docs/commands/academic.md | sort -u | wc -l` → 73).
  - `crates/domains/academic/src/commands.rs:64-461` defines 22 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/academic/src/commands.rs | wc -l` → 22).
  - `crates/domains/academic/src/lib.rs:7-15` and `:24` acknowledge the partial scope: "Phase 3 delivers the **prompt-named subset only**" and "The remaining 27 academic aggregates … land in later phases" — but `docs/commands/academic.md` is presented as the complete catalog for the domain, not a Phase 3 subset.

---

### FINDING 18 (id: `DOC-CAT-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/events/academic.md:10-96` (table) vs `crates/domains/academic/src/events.rs:53-1439`

**Description:**

The academic event catalog documents 85 distinct event names (e.g. `StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`, `ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `ClassSectionDeleted`, `SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`, `ClassRoutineCreated`, `ClassRoutinePeriodUpdated`, `ClassRoutinePeriodsSwapped`, `ClassRoutineDeleted`, `HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`, `LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonPlanDeleted`, `StudentRecordCreated`, `RollNumberAssigned`, `DefaultRecordSet`, `StudentMarkedGraduate`, `StudentCategoryCreated`, `StudentCategoryUpdated`, `StudentCategoryDeleted`, `StudentGroupCreated`, `StudentGroupUpdated`, `StudentAddedToGroup`, `StudentRemovedFromGroup`, `StudentGroupDeleted`, `RegistrationFieldCreated`, `RegistrationFieldUpdated`, `RegistrationFieldDeleted`, `CertificateCreated`, `CertificateUpdated`, `CertificateDeleted`, `IdCardCreated`, `IdCardUpdated`, `IdCardDeleted`, `AdmissionQueryRegistered`, `AdmissionQueryFollowedUp`, `AdmissionQueryConverted`, `AdmissionQueryClosed`). The Phase-3 implementation ships 22 event structs (`StudentAdmitted`, `StudentProfileUpdated`, `StudentSuspended`, `StudentReinstated`, `StudentWithdrawn`, `StudentTransferred`, `StudentPromoted`, `StudentGraduated`, `ClassCreated`, `ClassUpdated`, `OptionalSubjectGpaThresholdSet`, `ClassDeleted`, `SectionCreated`, `SectionUpdated`, `SectionDeleted`, `SubjectCreated`, `SubjectUpdated`, `SubjectDeleted`, `AcademicYearCreated`, `AcademicYearDatesUpdated`, `CurrentAcademicYearSet`, `AcademicYearClosed`, `AcademicYearCopied`). 63 catalog events have no struct.

**Expected:**

Per `docs/events/academic.md:10-96`, 85 academic event structs in `crates/domains/academic/src/events.rs`.

**Evidence:**

- `docs/events/academic.md:10-96` enumerates 85 `| \`XxxYyy\`` rows (`awk -F"\`" '$2 ~ /^[A-Z][a-zA-Z]+$/ && $4 ~ /^[A-Z][a-zA-Z]+$/ {print $2}' docs/events/academic.md | sort -u | wc -l` → 85).
  - `crates/domains/academic/src/events.rs:53-1439` defines 22 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/academic/src/events.rs | wc -l` → 22 events; only 23 incl. `AcademicYearCopied`).
  - The crate's own `lib.rs:7-15` notes the partial scope, but the doc is presented as the full domain event catalog.

---

### FINDING 19 (id: `DOC-CAT-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/events/finance.md` (table) vs `crates/domains/finance/src/events.rs:40-701`

**Description:**

The finance event catalog documents 179 distinct event names. The finance crate's events module only ships 10 event structs (`WalletCreated`, `WalletCredited`, `WalletDebited`, `WalletRefundRequested`, `WalletTransactionApproved`, `WalletTransactionRejected`, `InvoiceNumberingConfigured`, `ExpenseRecorded`, `PaymentReceived`, `PayrollPaymentRecorded`). The crate's own `lib.rs` declares "Workstream A ships the 5 headline events for `Wallet` + `WalletTransaction`" but the catalog covers 179. The catalog includes large event families (`FeesGroup*`, `FeesType*`, `FeesMaster*`, `FeesAssign*`, `FeesDiscount*`, `FeesInstallment*`, `DirectFeesInstallment*`, `DirectFeesReminder*`, `DirectFeesSetting*`, `BankAccount*`, `BankStatement*`, `BankPaymentSlip*`, `ChartOfAccount*`, `Donor*`, `IncomeHead*`, `ExpenseHead*`, `Inventory*`, `Product*`, `Payroll*`, `WalletTransaction*`, `Wallet*`, etc.) that have no event struct in the crate.

**Expected:**

Per `docs/events/finance.md`, 179 finance event structs in `crates/domains/finance/src/events.rs`.

**Evidence:**

- `docs/events/finance.md` enumerates 179 `| \`XxxYyy\`` rows (`grep -E "^\| \`[A-Z]" docs/events/finance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 179).
  - `crates/domains/finance/src/events.rs` defines 10 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/finance/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 10).
  - `crates/domains/finance/src/events.rs:11-15` confirms partial scope: "Workstream A ships the 5 headline events for `Wallet` + `WalletTransaction` … + `FeesInvoiceConfigured` … + `ExpenseRecorded`."

---

### FINDING 20 (id: `DOC-CAT-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/commands/assessment.md` (table) vs `crates/domains/assessment/src/commands.rs`

**Description:**

The assessment command catalog documents 128 distinct command names (a sample: `AddOnlineExamQuestion`, `AddQuestionOption`, `AddTeacherRemark`, `AdmitCardGenerated`, `AdmitCardSettingUpdated`, `ApproveTeacherEvaluation`, `ConfigureAdmitCardSettings`, `ConfigureCustomResultSettings`, `ConfigureSeatPlanSettings`, `ConfigureTeacherEvaluation`, `CreateExamSetting`, `CreateExamType`, `CreateMarksGrade`, `CreateOnlineExam`, `CreateQuestion`, `CreateQuestionGroup`, `CreateQuestionLevel`, `CustomResultSettingUpdated`, `DeleteExamSetting`, `DeleteExamType`, …). The assessment crate's commands module only ships 19 command structs. 109 catalog commands have no struct.

**Expected:**

Per `docs/commands/assessment.md`, 128 assessment command structs in `crates/domains/assessment/src/commands.rs`.

**Evidence:**

- `docs/commands/assessment.md` enumerates 128 command rows (`grep -E "^\| \`[A-Z]" docs/commands/assessment.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 128).
  - `crates/domains/assessment/src/commands.rs` defines 19 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/assessment/src/commands.rs | wc -l` → 19).

---

### FINDING 21 (id: `DOC-CAT-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/events/assessment.md` (table) vs `crates/domains/assessment/src/events.rs`

**Description:**

The assessment event catalog documents 114 distinct event names (sample: `AdmitCard`, `AdmitCardSetting`, `AdmitCardSettingUpdated`, `CustomResultSetting`, `CustomResultSettingUpdated`, `Exam`, `ExamAttendance`, `ExamAttendanceMarked`, `ExamAttendanceUpdated`, `ExamRoutinePage`, `ExamRoutinePageUpdated`, `ExamSchedule`, `ExamSetting`, `ExamSettingCreated`, `ExamSettingDeleted`, `ExamSettingUpdated`, `ExamSetup`, `ExamSetupCreated`, `ExamSetupDeleted`, `ExamSetupUpdated`, …). The assessment crate's events module only ships 21 event structs. 93 catalog events have no struct.

**Expected:**

Per `docs/events/assessment.md`, 114 assessment event structs in `crates/domains/assessment/src/events.rs`.

**Evidence:**

- `docs/events/assessment.md` enumerates 114 event rows (`grep -E "^\| \`[A-Z]" docs/events/assessment.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 114).
  - `crates/domains/assessment/src/events.rs` defines 21 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/assessment/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 21).

---

### FINDING 22 (id: `DOC-CAT-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/commands/cms.md` (table) vs `crates/domains/cms/src/commands.rs`

**Description:**

The CMS command catalog documents 129 distinct command names. The CMS crate's commands module ships only 10 command structs (`CreatePage`, `PublishPage`, `ArchivePage`, `DeletePage`, `CreateNews`, `CreateTestimonial`, `CreateHomeSlider`, `CreateContent`, `CreateContentShareList`, `ConfigureHomePage`). 119 catalog commands have no struct, despite the Phase 12 handoff language in `AGENTS.md` claiming the CMS domain is "spec-faithful".

**Expected:**

Per `docs/commands/cms.md`, 129 CMS command structs in `crates/domains/cms/src/commands.rs`.

**Evidence:**

- `docs/commands/cms.md` enumerates 129 command rows (`grep -E "^\| \`[A-Z]" docs/commands/cms.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 129).
  - `crates/domains/cms/src/commands.rs` defines 10 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/cms/src/commands.rs | wc -l` → 10).
  - `AGENTS.md` line for educore-cms claims "20 root aggregates … ~67 events, ~67 commands, 86 Cms caps", but only 10 commands exist on disk.

---

### FINDING 23 (id: `DOC-CAT-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/events/cms.md` (table) vs `crates/domains/cms/src/events.rs`

**Description:**

The CMS event catalog documents 85 distinct event names. The CMS crate's events module ships 67 event structs. 18 catalog events have no struct.

**Expected:**

Per `docs/events/cms.md`, 85 CMS event structs in `crates/domains/cms/src/events.rs`.

**Evidence:**

- `docs/events/cms.md` enumerates 85 event rows (`grep -E "^\| \`[A-Z]" docs/events/cms.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 85).
  - `crates/domains/cms/src/events.rs` defines 67 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/cms/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 67).
  - `crates/domains/cms/src/lib.rs:45-60` re-exports 67 events by name; the missing 18 events are catalog rows that have no corresponding export.

---

### FINDING 24 (id: `DOC-CAT-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/commands/hr.md` (table) vs `crates/domains/hr/src/commands.rs`

**Description:**

The HR command catalog documents 122 distinct command names. The HR crate's commands module ships only 21 command structs. 101 catalog commands have no struct.

**Expected:**

Per `docs/commands/hr.md`, 122 HR command structs in `crates/domains/hr/src/commands.rs`.

**Evidence:**

- `docs/commands/hr.md` enumerates 122 command rows (`grep -E "^\| \`[A-Z]" docs/commands/hr.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 122).
  - `crates/domains/hr/src/commands.rs` defines 21 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/hr/src/commands.rs | wc -l` → 21).

---

### FINDING 25 (id: `DOC-CAT-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/events/hr.md` (table) vs `crates/domains/hr/src/events.rs`

**Description:**

The HR event catalog documents 78 distinct event names. The HR crate's events module ships 46 event structs. 32 catalog events have no struct.

**Expected:**

Per `docs/events/hr.md`, 78 HR event structs in `crates/domains/hr/src/events.rs`.

**Evidence:**

- `docs/events/hr.md` enumerates 78 event rows (`grep -E "^\| \`[A-Z]" docs/events/hr.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 78).
  - `crates/domains/hr/src/events.rs` defines 46 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/hr/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 46).

---

### FINDING 26 (id: `DOC-CAT-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/facilities.md` (table) vs `crates/domains/facilities/src/commands.rs`

**Description:**

The facilities command catalog documents 100 distinct command names. The facilities crate's commands module ships 49 command structs. 51 catalog commands have no struct.

**Expected:**

Per `docs/commands/facilities.md`, 100 facilities command structs in `crates/domains/facilities/src/commands.rs`.

**Evidence:**

- `docs/commands/facilities.md` enumerates 100 command rows (`grep -E "^\| \`[A-Z]" docs/commands/facilities.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 100).
  - `crates/domains/facilities/src/commands.rs` defines 49 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/facilities/src/commands.rs | wc -l` → 49).

---

### FINDING 27 (id: `DOC-CAT-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/events/facilities.md` (table) vs `crates/domains/facilities/src/events.rs`

**Description:**

The facilities event catalog documents 63 distinct event names. The facilities crate's events module ships 23 event structs. 40 catalog events have no struct.

**Expected:**

Per `docs/events/facilities.md`, 63 facilities event structs in `crates/domains/facilities/src/events.rs`.

**Evidence:**

- `docs/events/facilities.md` enumerates 63 event rows (`grep -E "^\| \`[A-Z]" docs/events/facilities.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 63).
  - `crates/domains/facilities/src/events.rs` defines 23 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/facilities/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 23).

---

### FINDING 28 (id: `DOC-CAT-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/communication.md` (table) vs `crates/domains/communication/src/commands.rs`

**Description:**

The communication command catalog documents 54 distinct command names. The communication crate's commands module ships 72 command structs. 18 commands exist on disk that are not enumerated in the catalog (i.e. implementation outpaces documentation). Sample additional commands: `CreateBulkNotificationTemplate`, `UpdateBulkNotificationTemplate`, `DeleteBulkNotificationTemplate`, `CreateWhatsAppTemplate`, `UpdateWhatsAppTemplate`, `DeleteWhatsAppTemplate`, `SendBulkNotification`, `SendBulkEmail`, `SendBulkSms`, `SendBulkPush`, `SendBulkChat`, `SendBulkVoice`, `SendBulkWebhook`, `SendBulkInApp`.

**Expected:**

Per `docs/commands/communication.md`, 54 communication command structs; the actual count is 72.

**Evidence:**

- `docs/commands/communication.md` enumerates 54 command rows (`grep -E "^\| \`[A-Z]" docs/commands/communication.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 54).
  - `crates/domains/communication/src/commands.rs` defines 72 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/communication/src/commands.rs | wc -l` → 72).
  - `comm -13 <(sort -u <(grep -E "^\| \`[A-Z]" docs/commands/communication.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`")) <(grep -oE "struct [A-Z][a-zA-Z]+Command" crates/domains/communication/src/commands.rs | awk '{print $2}' | sed 's/Command$//' | sort -u) | head` returns 18 struct names not present in the catalog.

---

### FINDING 29 (id: `DOC-CAT-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/library.md` (table) vs `crates/domains/library/src/commands.rs`

**Description:**

The library command catalog documents 37 distinct command names. The library crate's commands module ships 22 command structs. 15 catalog commands have no struct.

**Expected:**

Per `docs/commands/library.md`, 37 library command structs in `crates/domains/library/src/commands.rs`.

**Evidence:**

- `docs/commands/library.md` enumerates 37 command rows (`grep -E "^\| \`[A-Z]" docs/commands/library.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 37).
  - `crates/domains/library/src/commands.rs` defines 22 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/library/src/commands.rs | wc -l` → 22).

---

### FINDING 30 (id: `DOC-CAT-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/attendance.md` (table) vs `crates/domains/attendance/src/commands.rs`

**Description:**

The attendance command catalog documents 28 distinct command names. The attendance crate's commands module ships 14 command structs. 14 catalog commands have no struct.

**Expected:**

Per `docs/commands/attendance.md`, 28 attendance command structs in `crates/domains/attendance/src/commands.rs`.

**Evidence:**

- `docs/commands/attendance.md` enumerates 28 command rows (`grep -E "^\| \`[A-Z]" docs/commands/attendance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 28).
  - `crates/domains/attendance/src/commands.rs` defines 14 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/attendance/src/commands.rs | wc -l` → 14).

---

### FINDING 31 (id: `DOC-CAT-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/events/attendance.md` (table) vs `crates/domains/attendance/src/events.rs`

**Description:**

The attendance event catalog documents 24 distinct event names. The attendance crate's events module ships 21 event structs. 3 catalog events have no struct.

**Expected:**

Per `docs/events/attendance.md`, 24 attendance event structs in `crates/domains/attendance/src/events.rs`.

**Evidence:**

- `docs/events/attendance.md` enumerates 24 event rows (`grep -E "^\| \`[A-Z]" docs/events/attendance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 24).
  - `crates/domains/attendance/src/events.rs` defines 21 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/attendance/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 21).

---

### FINDING 32 (id: `DOC-CAT-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/documents.md` (table) vs `crates/domains/documents/src/commands.rs`

**Description:**

The documents command catalog documents 19 distinct command names. The documents crate's commands module ships 10 command structs. 9 catalog commands have no struct.

**Expected:**

Per `docs/commands/documents.md`, 19 documents command structs in `crates/domains/documents/src/commands.rs`.

**Evidence:**

- `docs/commands/documents.md` enumerates 19 command rows (`grep -E "^\| \`[A-Z]" docs/commands/documents.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 19).
  - `crates/domains/documents/src/commands.rs` defines 10 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/documents/src/commands.rs | wc -l` → 10).

---

### FINDING 33 (id: `DOC-CAT-017`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/commands/platform.md` (table) vs `crates/cross-cutting/platform/src` (no `commands.rs`)

**Description:**

The platform command catalog documents a non-trivial number of commands but the `educore-platform` crate (cross-cutting tier) does not contain a `commands.rs` module. Platform commands are routed through the `events-domain` calendar crate or the `platform` value-objects/ids module; no `*Command` structs exist in `crates/cross-cutting/platform/`. This is mentioned because the prompt scope covers the docs; the cross-cutting `platform` commands live in a different location than the doc implies.

**Expected:**

Platform command structs in `crates/cross-cutting/platform/src/commands.rs` (or in a clearly cross-referenced location).

**Evidence:**

- `docs/commands/platform.md` exists and is in scope (`ls docs/commands/`).
  - `ls crates/cross-cutting/platform/src/` shows only the platform module's value-objects / id / tenant modules; no `commands.rs` (`find crates/cross-cutting/platform -name commands.rs` returns nothing).
  - The docs/ports/platform.md or specs/platform/commands.md should be cross-referenced from `docs/commands/platform.md` to direct readers to the actual location.

---

### FINDING 34 (id: `DOC-CAT-018`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `crates/domains/academic/src/events.rs:116,170,233,292,355,419,502,564,626,688,742,792,850,908,958,1029,1096,1146,1221,1279,1334,1384,1440` vs `docs/events/academic.md:12,13,14,…,96`

**Description:**

Wire-form consistency spot check on the academic events that ARE implemented. The doc uses the PascalCase type name in column 1 (`StudentAdmitted`), while the actual `EVENT_TYPE` constants on the `DomainEvent` impl use the dotted lowercase string per the bus-port contract (e.g. `academic.student.admitted`). The bus-port spec at `docs/ports/event-bus.md:27` states the `event_type` is `&'static str` and the convention is `<domain>.<aggregate>.<verb>`; the doc table column header does not distinguish between the type name and the wire event_type, which is a documentation precision gap rather than a true wire-form drift.

**Expected:**

Each catalog row should make the event type name and the wire event_type string explicit (e.g. separate columns or a sidebar block).

**Evidence:**

- `docs/events/academic.md:12` — `| \`StudentAdmitted\` | \`Student\` | … |`
  - `crates/domains/academic/src/events.rs:115-118` — `impl DomainEvent for StudentAdmitted { const EVENT_TYPE: &'static str = "academic.student.admitted"; const SCHEMA_VERSION: u32 = 1; const AGGREGATE_TYPE: &'static str = "student"; }`
  - The doc column at `docs/events/academic.md:10` header reads `| Event | Aggregate | Subscribers | Description | Durable? | Replicated? | Replayable? |` — no `event_type` column.

---

### FINDING 35 (id: `DOC-CAT-019`)

- **Source:** `docs/audit_reports/findings/wave5-docs-4.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/commands/finance.md:14-96` (table) vs `crates/domains/finance/src/commands.rs:64+` (118 `*Command` structs)

**Description:**

The finance command catalog documents 79 distinct command names; the finance crate's commands module actually ships 118 `*Command` structs. The implementation has 39 commands not enumerated in the catalog (sample: `ApproveBankSlip`, `ApproveExpense`, `ApproveIncome`, `ApprovePayroll`, `ApprovePayrollPayment`, `ApproveWalletTransaction`, `BlockLoginForDueFees`, `CancelInvoice`, `CarryForwardFeesBalance`, `ConfigureDueFeesBlockSetting`, `ConfigureFeesCarryForward`, `ConfigureFeesGroup`, `ConfigureFeesType`, `CreateAmountTransfer`, `CreateDirectFeesInstallment`, `CreateDirectFeesInstallmentAssign`, `CreateDirectFeesReminder`, `CreateDirectFeesSetting`, `CreateExpenseHead`, `CreateFeesAssign`, `CreateFeesDiscount`, `CreateFeesGroup`, `CreateFeesInstallment`, `CreateFeesMaster`, `CreateFeesType`, `CreateIncome`, `CreateIncomeHead`, `CreatePaymentGateway`, `CreatePaymentMethod`, `DeleteBankAccount`, `DeleteDirectFeesInstallment`, `DeleteDirectFeesInstallmentAssign`, `DeleteDirectFeesReminder`, `DeleteDirectFeesSetting`, `DeleteExpense`, `DeleteExpenseHead`, `DeleteFeesAssign`, `DeleteFeesDiscount`, `DeleteFeesGroup`, `DeleteFeesInstallment`). Like finding 28 for communication, the implementation outpaces the doc here.

**Expected:**

Catalog covers every implemented command.

**Evidence:**

- `docs/commands/finance.md` enumerates 79 command rows (`grep -E "^\| \`[A-Z]" docs/commands/finance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 79).
  - `crates/domains/finance/src/commands.rs` defines 118 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/finance/src/commands.rs | wc -l` → 118).
  - The catalog/doc was rebuilt during Phase 7 spec cleanup; the implementations kept growing after the snapshot.

---


## Schemas (target id prefix: `DOC-SCHM`)

**Path:** `docs/schemas/`  
**Total findings:** 36 (4 critical, 7 high, 16 medium, 9 low)


### FINDING 3 (id: `DOC-SCHM-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:83, 130, 197` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:59, 65, 68`

**Description:**

The SurrealDB canonical DDL declares `school_id` as `option<uuid>` (NULL) on `outbox`, `audit_log`, and `event_log`, but every other dialect emits `school_id` as `NOT NULL`. The engine invariant requires every tenant-scoped row to carry a non-null `school_id` (`docs/schemas/database-schema.md:48-58`); the canonical SurrealDB file violates this for all three of the cross-cutting tables that need it.

**Expected:**

`school_id` is `NOT NULL` on every aggregate and on every cross-cutting table per `docs/schemas/database-schema.md:48-58` and the relational canonical DDLs.

**Evidence:**

- `migrations/engine/0000_engine_core.surreal.surql:83` — `DEFINE FIELD school_id       ON TABLE outbox TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.surreal.surql:130` — `DEFINE FIELD school_id       ON TABLE audit_log TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.surreal.surql:197` — `DEFINE FIELD school_id       ON TABLE event_log TYPE option<uuid>;`
  - `migrations/engine/0000_engine_core.mysql.sql:59` — `school_id       CHAR(36)     NOT NULL,`
  - `migrations/engine/0000_engine_core.postgres.sql:65` — `school_id       UUID         NOT NULL,`
  - `migrations/engine/0000_engine_core.sqlite.sql:68` — `school_id       TEXT         NOT NULL CHECK (length(school_id) = 36),`

---

### FINDING 4 (id: `DOC-SCHM-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:167-175` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:132-141, 148-160, 151-163`

**Description:**

The SurrealDB canonical `idempotency` table is missing `command_id` and `expires_at` (both present in the three relational canonical DDLs and required by the `Idempotency` port contract). It also defines `outcome` as `bytes` whereas the relational canonical DDLs use `JSON` / `JSONB` / `TEXT` with `json_valid` / `jsonb_typeof` CHECKs, and it stores `outcome_version` as an `int` (no analogue in the relational canonical DDLs).

**Expected:**

Per `migrations/engine/0000_engine_core.mysql.sql:131-141` and the engine spec `docs/schemas/command-schema.md:148-167`, `idempotency` has columns `school_id, command_type, idempotency_key, command_id, outcome, recorded_at, expires_at`.

**Evidence:**

- `migrations/engine/0000_engine_core.surreal.surql:167-175`:
    ```
    DEFINE FIELD school_id              ON TABLE idempotency TYPE uuid                ASSERT $value != NONE;
    DEFINE FIELD command_type           ON TABLE idempotency TYPE string              ASSERT $value != NONE AND string::len($value) <= 191;
    DEFINE FIELD idempotency_key        ON TABLE idempotency TYPE uuid                ASSERT $value != NONE;
    DEFINE FIELD outcome                ON TABLE idempotency TYPE bytes               ASSERT $value != NONE;
    DEFINE FIELD outcome_version        ON TABLE idempotency TYPE int                 ASSERT $value != NONE;
    DEFINE FIELD recorded_at            ON TABLE idempotency TYPE datetime            ASSERT $value != NONE;
    DEFINE FIELD affected_aggregate_ids ON TABLE idempotency TYPE option<array<uuid>>;
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:131-141` includes `command_id CHAR(36) NOT NULL` and `expires_at TIMESTAMP NOT NULL` — neither of which exists in the Surreal canonical.

---

### FINDING 7 (id: `DOC-SCHM-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/schemas/audit-schema.md:334-342` vs `migrations/engine/0000_engine_core.postgres.sql:104-137`

**Description:**

The PostgreSQL partitioning example in `audit-schema.md` § 13.1 declares `audit_log` as `PARTITION BY RANGE (school_id, date_trunc('month', occurred_at))` with a composite PRIMARY KEY `(school_id, occurred_at, audit_id)` and a per-school, per-month partition naming convention. The canonical PG DDL has none of these: it declares `PRIMARY KEY (audit_id)` as a single-column key and emits no `PARTITION BY` clause, no `pg_cron` rotation, and no per-school partition scheme. The spec describes a feature the canonical DDL does not implement.

**Expected:**

Canonical PG DDL implements the partitioning scheme documented in `docs/schemas/audit-schema.md:326-359`.

**Evidence:**

- `docs/schemas/audit-schema.md:334-342`:
    ```sql
    CREATE TABLE audit_log (
        audit_id        UUID NOT NULL,
        school_id       UUID NOT NULL,
        -- ... other columns ...
        occurred_at     TIMESTAMP NOT NULL,
        PRIMARY KEY (school_id, occurred_at, audit_id)
    ) PARTITION BY RANGE (school_id, date_trunc('month', occurred_at));
    ```
  - `migrations/engine/0000_engine_core.postgres.sql:104-137` has `PRIMARY KEY (audit_id)` (single-column) and no `PARTITION BY` clause anywhere in the file.
  - Additionally, the spec example at line 339 declares `occurred_at TIMESTAMP NOT NULL` whereas the canonical at line 115 uses `TIMESTAMPTZ` — the spec example is not even dialect-correct.

---

### FINDING 8 (id: `DOC-SCHM-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Critical
- **Area:** documentation
- **Location:** `docs/schemas/audit-schema.md:368-377` vs `migrations/engine/0000_engine_core.mysql.sql:93-120`

**Description:**

The MySQL partitioning example in `audit-schema.md` § 13.2 declares `audit_log` with `BINARY(16)` id types, `DATETIME(6)` timestamps, composite PRIMARY KEY `(school_id, occurred_at, audit_id)`, and `PARTITION BY KEY (school_id) PARTITIONS 12`. The canonical MySQL DDL uses `CHAR(36)` ids, `TIMESTAMP` (not `DATETIME(6)`), single-column `PRIMARY KEY (audit_id)`, and no partitioning. Every column type and structural choice in the spec example disagrees with the canonical DDL.

**Expected:**

Canonical MySQL DDL implements the partitioning scheme documented in `docs/schemas/audit-schema.md:360-398`.

**Evidence:**

- `docs/schemas/audit-schema.md:368-377`:
    ```sql
    CREATE TABLE audit_log (
        audit_id        BINARY(16) NOT NULL,
        school_id       BINARY(16) NOT NULL,
        -- ... other columns ...
        occurred_at     DATETIME(6) NOT NULL,
        PRIMARY KEY (school_id, occurred_at, audit_id)
    ) ENGINE=InnoDB
      PARTITION BY KEY (school_id) PARTITIONS 12;
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:93-120` uses `CHAR(36)` (lines 94-95), `TIMESTAMP` (line 104), `PRIMARY KEY (audit_id)` (line 114), and contains no `PARTITION BY` clause.

---

### FINDING 1 (id: `DOC-SCHM-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/schemas/audit-schema.md:51-52` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:109-110, 120-121, 123-124`

**Description:**

The `AuditRecord` struct in `audit-schema.md` § 2 uses field names `before` and `after` for the snapshot columns, and the generic schema layout in `audit-schema.md` § 14 (lines 442-443) likewise uses `before JSONB NULL, after JSONB NULL`. But the canonical DDLs in all three shipped backends name these columns `before_snapshot` and `after_snapshot`. The normative spec and the canonical DDL disagree on the column name every storage adapter must emit.

**Expected:**

Per the normative spec, the columns are `before` and `after` (`audit-schema.md:51-52`: `before: Option<Value>, // pre-mutation snapshot` and `before JSONB NULL,` at `audit-schema.md:442`).

**Evidence:**

- `docs/schemas/audit-schema.md:51-52` — `before:          Option<Value>,     // pre-mutation snapshot` / `after:           Option<Value>,     // post-mutation snapshot`
  - `docs/schemas/audit-schema.md:442-443` — `before          JSONB NULL,` / `after           JSONB NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:109-110` — `before_snapshot JSON             NULL,` / `after_snapshot  JSON             NULL,`
  - `migrations/engine/0000_engine_core.postgres.sql:120-121` — `before_snapshot JSONB            NULL,` / `after_snapshot  JSONB            NULL,`
  - `migrations/engine/0000_engine_core.sqlite.sql:123-124` — `before_snapshot TEXT             NULL,` / `after_snapshot  TEXT             NULL,`

---

### FINDING 10 (id: `DOC-SCHM-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/comparison.md:192` vs `migrations/engine/0000_engine_core.postgres.sql:104-137`

**Description:**

The cross-cutting-table feature availability table claims PG `audit_log` is `"Native (with RLS, append-only role)"`, but the canonical PG DDL contains neither RLS nor an `INSERT`-only role grant. The comparison table claims a feature the canonical DDL does not ship.

**Expected:**

Per the comparison table, PG canonical `audit_log` should include RLS (`CREATE POLICY`) and an append-only role setup.

**Evidence:**

- `docs/schemas/sql-dialects/comparison.md:193` — `| audit_log | Native | Native | Native (with RLS, append-only role) |`
  - `migrations/engine/0000_engine_core.postgres.sql:104-137` contains no `ROW LEVEL SECURITY`, no `CREATE POLICY`, and no `GRANT` statements.

---

### FINDING 18 (id: `DOC-SCHM-018`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/postgresql.md:16, 32-34` vs `migrations/engine/0000_engine_core.postgres.sql:61, 104, 148, 171, 205, 226`

**Description:**

The PG dialect spec example shows `CREATE TABLE "outbox" (...)` with the bare table name (no schema prefix). The canonical PG DDL wraps all 6 tables in an `engine` schema (`CREATE TABLE IF NOT EXISTS engine.outbox (...)`, `engine.audit_log`, `engine.idempotency`, `engine.event_log`, `engine.schema_registry`, `engine.system_user`). The example contradicts the canonical form, and the engine contract claim "the same table name in all backends" (`docs/schemas/sql-dialects/README.md:65`) is broken — PG sees `engine.outbox` while MySQL and SQLite see `outbox`.

**Expected:**

The dialect spec example matches the canonical PG DDL (table names qualified with `engine.`) or the canonical DDL drops the `engine.` prefix.

**Evidence:**

- `docs/schemas/sql-dialects/postgresql.md:16` — `CREATE TABLE "outbox" ( "event_id" UUID NOT NULL, ... );`
  - `docs/schemas/sql-dialects/postgresql.md:32-34` — `CREATE TABLE "outbox" ( ... );`
  - `migrations/engine/0000_engine_core.postgres.sql:61` — `CREATE TABLE IF NOT EXISTS engine.outbox (`
  - `migrations/engine/0000_engine_core.postgres.sql:104` — `CREATE TABLE IF NOT EXISTS engine.audit_log (`
  - `migrations/engine/0000_engine_core.postgres.sql:148` — `CREATE TABLE IF NOT EXISTS engine.idempotency (`
  - `migrations/engine/0000_engine_core.postgres.sql:171` — `CREATE TABLE IF NOT EXISTS engine.event_log (`
  - `migrations/engine/0000_engine_core.postgres.sql:205` — `CREATE TABLE IF NOT EXISTS engine.schema_registry (`
  - `migrations/engine/0000_engine_core.postgres.sql:226` — `CREATE TABLE IF NOT EXISTS engine.system_user (`

---

### FINDING 2 (id: `DOC-SCHM-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:130-141` vs `docs/schemas/audit-schema.md:41-57`

**Description:**

The SurrealDB canonical DDL defines the `audit_log` table with a substantially different shape than the normative spec. The spec requires `actor_type`, `resource_type`, `resource_id`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `source`, `cross_tenant`; the canonical Surreal file renames `resource_type`/`resource_id` to `target_type`/`target_id`, stores `before`/`after` as `bytes` instead of `JSON`/`JSONB`, and omits `actor_type`, `command_id`, `recorded_at`, `ip`, `user_agent`, `session_id`, `source`, `cross_tenant`. It also adds `active_status` which is not in any other dialect.

**Expected:**

Per the normative spec, the `audit_log` table contains `actor_type, action, resource_type, resource_id, event_id, command_id, correlation_id, occurred_at, recorded_at, ip, user_agent, session_id, before, after, metadata, cross_tenant, source` (`docs/schemas/audit-schema.md:35-57`).

**Evidence:**

- `migrations/engine/0000_engine_core.surreal.surql:130-141`:
    ```
    DEFINE FIELD school_id       ON TABLE audit_log TYPE option<uuid>;
    DEFINE FIELD actor_id        ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD action          ON TABLE audit_log TYPE string            ASSERT $value != NONE AND string::len($value) <= 191;
    DEFINE FIELD target_type     ON TABLE audit_log TYPE string            ASSERT $value != NONE AND string::len($value) <= 64;
    DEFINE FIELD target_id       ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD before          ON TABLE audit_log TYPE option<bytes>;
    DEFINE FIELD after           ON TABLE audit_log TYPE option<bytes>;
    DEFINE FIELD event_id        ON TABLE audit_log TYPE option<uuid>;
    DEFINE FIELD correlation_id  ON TABLE audit_log TYPE uuid              ASSERT $value != NONE;
    DEFINE FIELD occurred_at     ON TABLE audit_log TYPE datetime          ASSERT $value != NONE;
    DEFINE FIELD active_status   ON TABLE audit_log TYPE string            ASSERT $value != NONE;
    DEFINE FIELD metadata        ON TABLE audit_log TYPE option<object>;
    ```
  - The file's own header comment at line 121-126 admits: `"which had fields like audit_id, actor_type, resource_type, ip, user_agent, etc. that did not exist on the storage-port struct"`.

---

### FINDING 5 (id: `DOC-SCHM-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.surreal.surql:193-213` vs `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql:152-170, 171-185, 174-188`

**Description:**

The SurrealDB canonical `event_log` uses `schema_version` instead of `event_version`, stores `payload` as `bytes` instead of `JSON`/`JSONB`/`TEXT` with JSON CHECK, and adds an `active_status` field that does not exist in any other dialect's `event_log`. The field rename breaks the contract documented in `docs/schemas/event-schema.md:13-29` (`event_version`) and the engine's typed events.

**Expected:**

`event_version` per the spec (`docs/schemas/event-schema.md:16`) and the three relational canonical DDLs; `payload` as JSON-typed.

**Evidence:**

- `migrations/engine/0000_engine_core.surreal.surql:199` — `DEFINE FIELD schema_version  ON TABLE event_log TYPE int      ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.surreal.surql:207` — `DEFINE FIELD payload         ON TABLE event_log TYPE bytes    ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.surreal.surql:208` — `DEFINE FIELD active_status   ON TABLE event_log TYPE string   ASSERT $value != NONE;`
  - `migrations/engine/0000_engine_core.mysql.sql:155` — `event_version   INT          NOT NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:164` — `payload         JSON         NOT NULL,`

---

### FINDING 6 (id: `DOC-SCHM-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/schemas/event-schema.md:251-263`

**Description:**

The outbox schema in `event-schema.md` § 8 lists only 9 fields (`event_id`, `event_type`, `event_version`, `school_id`, `payload`, `enqueued_at`, `published_at`, `attempts`, `last_error`) but every canonical DDL defines 17 columns including `aggregate_id`, `aggregate_type`, `actor_id`, `correlation_id`, `causation_id`, `occurred_at`, `recorded_at`. The spec is incomplete and any consumer implementing against the spec alone will omit 8 mandatory columns.

**Expected:**

The full outbox column list as in the relational canonical DDLs (`migrations/engine/0000_engine_core.mysql.sql:55-77`).

**Evidence:**

- `docs/schemas/event-schema.md:251-263`:
    ```
    outbox(
        event_id        EventId       PK,
        event_type      VARCHAR,
        event_version   INT,
        school_id       SchoolId,
        payload         JSON,
        enqueued_at     TIMESTAMP,
        published_at    TIMESTAMP     NULL,
        attempts        INT           DEFAULT 0,
        last_error      TEXT          NULL
    )
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:55-77` defines the same table with 17 columns, including `aggregate_id`, `aggregate_type`, `actor_id`, `correlation_id`, `causation_id`, `occurred_at`, `recorded_at` — none of which appear in the spec.

---

### FINDING 9 (id: `DOC-SCHM-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** High
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/postgresql.md:122-160` vs `migrations/engine/0000_engine_core.postgres.sql:1-240`

**Description:**

The PostgreSQL dialect spec declares "PG has the most expressive RLS of the three backends. The engine **requires** RLS as a defense-in-depth" and shows the canonical `CREATE POLICY` SQL for `school_isolation_<aggregate>`. The canonical PG DDL file emits zero `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` statements and zero `CREATE POLICY` statements — none of the 6 cross-cutting tables have RLS. The spec requires a feature the canonical DDL does not implement.

**Expected:**

Per `docs/schemas/sql-dialects/postgresql.md:122-160`, the canonical PG DDL emits `ALTER TABLE ... ENABLE ROW LEVEL SECURITY`, `FORCE ROW LEVEL SECURITY`, and `CREATE POLICY school_isolation_<aggregate> ...` for every aggregate.

**Evidence:**

- `docs/schemas/sql-dialects/postgresql.md:124-140`:
    ```sql
    ALTER TABLE "<aggregate>" ENABLE ROW LEVEL SECURITY;
    ALTER TABLE "<aggregate>" FORCE ROW LEVEL SECURITY;
    CREATE POLICY "school_isolation_<aggregate>" ON "<aggregate>"
      USING ("school_id" = current_setting('app.current_school_id')::UUID)
      WITH CHECK ("school_id" = current_setting('app.current_school_id')::UUID);
    ```
  - `migrations/engine/0000_engine_core.postgres.sql` (entire 240-line file) contains zero matches for `ROW LEVEL SECURITY`, `FORCE ROW LEVEL`, or `CREATE POLICY`.

---

### FINDING 11 (id: `DOC-SCHM-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/database-schema.md:121, 183, 220`

**Description:**

`database-schema.md` is internally inconsistent about the canonical type for `etag`. § 5 (line 121) lists `etag CHAR(32) / hash`; § 9 (line 183) lists `etag BINARY(16) / hash`; § 11 (line 220) emits `etag BINARY(16) NOT NULL`. The same normative document gives three different recommendations.

**Expected:**

A single canonical type for `etag` (consistent with `database-schema.md` being normative per `audit-schema.md` and `event-schema.md` precedent).

**Evidence:**

- `docs/schemas/database-schema.md:121` — `| etag | CHAR(32) / hash | Content hash for conflict resolution. See § 9. |`
  - `docs/schemas/database-schema.md:183` — `| etag | BINARY(16) / hash | Content-addressed hash of the row's mutable fields. Used for client-side conflict check. |`
  - `docs/schemas/database-schema.md:220` — `etag            BINARY(16)     NOT NULL,`

---

### FINDING 12 (id: `DOC-SCHM-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/database-schema.md:220` vs `docs/schemas/sql-dialects/{mysql,postgresql,sqlite,surrealdb}.md`

**Description:**

The canonical minimum schema in `database-schema.md` § 11 declares `etag BINARY(16)`, but every per-dialect aggregate-table example uses `CHAR(32)` (or its dialect equivalent). The aggregate examples are the implementation guidance; the canonical minimum schema is the spec. They disagree.

**Expected:**

`etag CHAR(32)` (and dialect equivalents) on aggregate tables, consistent with the per-dialect examples and the engine's UUIDv7 byte-for-byte hashing rationale.

**Evidence:**

- `docs/schemas/database-schema.md:220` — `etag            BINARY(16)     NOT NULL,`
  - `docs/schemas/sql-dialects/mysql.md:263` — ``etag`            CHAR(32)     NOT NULL,`
  - `docs/schemas/sql-dialects/postgresql.md:415` — `"etag"              CHAR(32)     NOT NULL,`
  - `docs/schemas/sql-dialects/sqlite.md:363` — `"etag"              TEXT NOT NULL,` (with length-32 CHECK)
  - `docs/schemas/sql-dialects/surrealdb.md:710-711` — `DEFINE FIELD etag            ON TABLE academic_students TYPE string ASSERT string::length($value) = 32;`

---

### FINDING 13 (id: `DOC-SCHM-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/postgresql.md:283` vs `migrations/engine/0000_engine_core.postgres.sql:117`

**Description:**

The PostgreSQL dialect spec example for `audit_log.ip` uses `INET` (PG's native IP-address type), but the canonical PG DDL declares `ip VARCHAR(45) NULL`. A consumer implementing from the spec will emit `INET`; the canonical DDL emitted at startup emits `VARCHAR(45)`. The two are not equivalent (`INET` validates IP syntax; `VARCHAR(45)` accepts any 45-char string).

**Expected:**

Either the canonical PG DDL uses `INET` to match the spec, or the spec is updated to `VARCHAR(45)`.

**Evidence:**

- `docs/schemas/sql-dialects/postgresql.md:283` — `"ip"              INET,`
  - `migrations/engine/0000_engine_core.postgres.sql:117` — `ip              VARCHAR(45)     NULL,`
  - `migrations/engine/0000_engine_core.mysql.sql:106` — `ip              VARCHAR(45)     NULL,` (also `VARCHAR(45)`)

---

### FINDING 14 (id: `DOC-SCHM-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.postgres.sql:73, 120-122, 153, 183, 208` vs `docs/schemas/sql-dialects/postgresql.md:249, 286-288, 314, 339, 359`

**Description:**

The PostgreSQL dialect spec declares `JSONB` columns must carry `CHECK (jsonb_typeof(...) = 'object')` constraints (e.g. `outbox.payload`, `audit_log.before_snapshot`, `audit_log.after_snapshot`, `audit_log.metadata`, `idempotency.outcome`, `event_log.payload`, `schema_registry.schema_json`). The canonical PG DDL file emits zero such CHECK constraints — every JSONB column is declared `NULL` or `NOT NULL` with no JSON-shape validation. The spec mandates a constraint the canonical DDL omits.

**Expected:**

Per `docs/schemas/sql-dialects/postgresql.md:58` — `"native JSONB; engine emits JSONB NOT NULL CHECK (jsonb_typeof(\"payload\") = 'object')"`.

**Evidence:**

- `docs/schemas/sql-dialects/postgresql.md:249` — `"payload"         JSONB        NOT NULL CHECK (jsonb_typeof("payload") = 'object'),`
  - `docs/schemas/sql-dialects/postgresql.md:286` — `"before_snapshot" JSONB        CHECK ("before_snapshot" IS NULL OR jsonb_typeof("before_snapshot") = 'object'),`
  - `migrations/engine/0000_engine_core.postgres.sql:73` — `payload         JSONB        NOT NULL,` (no CHECK)
  - `migrations/engine/0000_engine_core.postgres.sql:120` — `before_snapshot JSONB            NULL,` (no CHECK)

---

### FINDING 15 (id: `DOC-SCHM-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.sqlite.sql:76, 123-125, 156, 186, 211` vs `docs/schemas/sql-dialects/sqlite.md:202, 238-240, 265, 290, 309`

**Description:**

The SQLite dialect spec declares every JSON-typed column must carry `CHECK (json_valid(...))` (or `json_valid(...) IS NULL OR json_valid(...)` for nullable). The canonical SQLite DDL emits zero such CHECK constraints — every JSON column is plain `TEXT NOT NULL` / `TEXT NULL`.

**Expected:**

Per `docs/schemas/sql-dialects/sqlite.md:202` — `"payload"         TEXT NOT NULL CHECK (json_valid("payload")),`.

**Evidence:**

- `docs/schemas/sql-dialects/sqlite.md:202` — `"payload"         TEXT NOT NULL CHECK (json_valid("payload")),`
  - `docs/schemas/sql-dialects/sqlite.md:238` — `"before_snapshot" TEXT CHECK ("before_snapshot" IS NULL OR json_valid("before_snapshot")),`
  - `migrations/engine/0000_engine_core.sqlite.sql:76` — `payload         TEXT         NOT NULL,` (no CHECK)
  - `migrations/engine/0000_engine_core.sqlite.sql:123-125` — `before_snapshot TEXT             NULL, after_snapshot  TEXT             NULL, metadata        TEXT             NULL,` (no CHECKs)

---

### FINDING 16 (id: `DOC-SCHM-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/surrealdb.md:30, 47-49` vs `migrations/engine/0000_engine_core.surreal.surql:71-260`

**Description:**

The SurrealDB dialect spec says "Use **backticks** for every identifier" (line 30) and "SurrealDB accepts both backticks and double quotes for identifier quoting. The engine uses backticks" (lines 46-49). The canonical SurrealDB DDL file uses no quoting at all on any identifier — table names, field names, index names, and column references are all bare (unquoted).

**Expected:**

Per the spec, every identifier in the canonical SurrealDB DDL is backtick-quoted.

**Evidence:**

- `docs/schemas/sql-dialects/surrealdb.md:30` — `Use **backticks** for every identifier:`
  - `docs/schemas/sql-dialects/surrealdb.md:47-49` — `SurrealDB accepts both backticks and double quotes for identifier quoting. The engine uses backticks to match the MySQL adapter`
  - `migrations/engine/0000_engine_core.surreal.surql:71` — `DEFINE TABLE outbox SCHEMAFULL` (no backticks)
  - `migrations/engine/0000_engine_core.surreal.surql:74` — `DEFINE FIELD event_id        ON TABLE outbox TYPE uuid     ASSERT $value != NONE;` (no backticks)
  - `migrations/engine/0000_engine_core.surreal.surql:97` — `DEFINE INDEX idx_outbox_event_id        ON TABLE outbox COLUMNS event_id      UNIQUE;` (no backticks)

---

### FINDING 19 (id: `DOC-SCHM-019`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `migrations/engine/0000_engine_core.postgres.sql:229` vs `docs/schemas/database-schema.md:58` and `docs/schemas/sql-dialects/postgresql.md:50`

**Description:**

The canonical PG DDL declares `system_user.active_status` as `SMALLINT NOT NULL DEFAULT 1`, but `database-schema.md` § 2 specifies `active_status TINYINT / BOOLEAN no` (the per-dialect mapping in `postgresql.md` § Type mapping says `TINYINT` maps to `SMALLINT (with CHECK range)`, or `BOOLEAN` for booleans). Using `SMALLINT` for a boolean-style flag is unusual and disagrees with the per-dialect type mapping.

**Expected:**

`system_user.active_status` is `BOOLEAN NOT NULL DEFAULT TRUE` per the dialect mapping (consistent with PG's native boolean type and the SQLite `INTEGER ... CHECK IN (0,1)` form).

**Evidence:**

- `migrations/engine/0000_engine_core.postgres.sql:229` — `active_status SMALLINT     NOT NULL DEFAULT 1,`
  - `docs/schemas/database-schema.md:58` — `| active_status | TINYINT / BOOLEAN | no |`
  - `docs/schemas/sql-dialects/postgresql.md:50` — `| TINYINT | SMALLINT (with CHECK range) | engine uses BOOLEAN for booleans and SMALLINT for 1-byte ints |`
  - `migrations/engine/0000_engine_core.mysql.sql:205` — `active_status TINYINT     NOT NULL DEFAULT 1,` (different from PG)
  - `migrations/engine/0000_engine_core.sqlite.sql:232` — `active_status INTEGER      NOT NULL DEFAULT 1 CHECK (active_status IN (0, 1)),`

---

### FINDING 21 (id: `DOC-SCHM-021`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/02-id-conversion.md:78-86` vs `docs/schemas/data-migration/02-id-conversion.md:79-82` and `migrations/engine/0000_engine_core.{mysql,postgres,sqlite}.sql`

**Description:**

The `uuid_v7(namespace, legacy_id)` derivation formula in § 02 uses a "fixed engine epoch — the engine's first commit timestamp, e.g. 2026-01-01T00:00:00.000Z" as the UUIDv7 timestamp component. But the engine's `system_user` row in every canonical DDL uses the literal id `'00000000-0000-7000-8000-000000000001'` — which is not a valid UUIDv7 (it does not encode a `2026-01-01` timestamp in the high 48 bits; it is an all-zero + variant-`7` constant). Two engines implementing per spec will derive different `id_v7_legacy`-based ids depending on which constant they treat as the "engine epoch".

**Expected:**

Either the spec names the fixed epoch explicitly and uses a real UUIDv7 encoding of it, or it notes that the engine `system_user` id is an out-of-band constant that does not follow the `uuid_v7()` derivation.

**Evidence:**

- `docs/schemas/data-migration/02-id-conversion.md:78-86`:
    ```
    uuid_v7(namespace, legacy_id) = UUIDv7(
        timestamp = <a fixed "engine epoch" — the engine's first commit
                    timestamp, e.g. 2026-01-01T00:00:00.000Z>,
        sub_ms    = (legacy_id % 4096),
        rand_a    = (legacy_id >> 12) & 0xFFF,
        rand_b    = blake3(namespace || legacy_id)[0..62 bits]
    )
    ```
  - `migrations/engine/0000_engine_core.mysql.sql:214` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, UTC_TIMESTAMP(6));`
  - `migrations/engine/0000_engine_core.postgres.sql:239` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, NOW())`
  - `migrations/engine/0000_engine_core.sqlite.sql:241` — `VALUES ('00000000-0000-7000-8000-000000000001', 'SYSTEM', 1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));`

---

### FINDING 22 (id: `DOC-SCHM-022`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/03-domain-renames.md:120` vs `docs/schemas/data-migration/03-domain-renames.md:151` (dup claim)

**Description:**

The assessment-domain table list declares `sm_online_exam_questions (dup)` as a drop (line 120). The same row appears nowhere else in the rename map (the academic domain treats `sm_online_exam_questions` as a kept table at line 118: `sm_online_exam_questions | academic_online_exam_questions`). The two domain lists disagree about whether this table is kept or dropped.

**Expected:**

A single decision for `sm_online_exam_questions` — either kept (academic owns it) or dropped (assessment owns the canonical form).

**Evidence:**

- `docs/schemas/data-migration/03-domain-renames.md:118` (Academic domain) — `| sm_online_exam_questions | academic_online_exam_questions |`
  - `docs/schemas/data-migration/03-domain-renames.md:120` (Assessment domain) — `| sm_online_exam_questions (dup) | (drop; see academic) |`

---

### FINDING 23 (id: `DOC-SCHM-023`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/03-domain-renames.md:440-441` vs `docs/schemas/data-migration/03-domain-renames.md:36-75` and `03-domain-renames.md:113-130`

**Description:**

The aggregate count at the end of `03-domain-renames.md` says `total = 320` (290 renames + 7 archives + 10 drops + 6 keep + 7 consumer-side adds), but summing the per-domain rename lists shows: Platform 38 + Academic 50 + Assessment 43 + Attendance 7 + Communication 23 + Documents 3 + Events 7 + Facilities 15 + Finance 47 + HR 14 + Library 4 + CMS 20 + RBAC 10 + Settings 14 + Operations 15 = 310 renames alone (plus 7 archives and 7 consumer-side adds). The 310-renames figure exceeds the 290-rename figure in the same file.

**Expected:**

The aggregate-count table matches the sum of the per-domain rename lists.

**Evidence:**

- `docs/schemas/data-migration/03-domain-renames.md:440-441` — `| rename | ~290 |` / `| **total** | **320** |`
  - `docs/schemas/data-migration/03-domain-renames.md:36-414` (per-domain lists): Platform 38 (line 35) + Academic 50 (line 77) + Assessment 43 (line 133) + Attendance 7 (line 180) + Communication 23 (line 192) + Documents 3 (line 220) + Events 7 (line 228) + Facilities 15 (line 240) + Finance 47 (line 260) + HR 14 (line 311) + Library 4 (line 330) + CMS 20 (line 339) + RBAC 10 (line 363) + Settings 14 (line 378) + Operations 15 (line 397) = 310 rename rows.

---

### FINDING 25 (id: `DOC-SCHM-025`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/04-column-additions.md:11-23`

**Description:**

The "engine invariant columns" table in § 04 is titled "the seven engine-invariant columns" but lists 10 rows (`created_at`, `updated_at`, `created_by`, `updated_by`, `active_status`, `version`, `etag`, `last_event_id`, `correlation_id`, `source`). The header comment at lines 24-28 acknowledges the discrepancy ("That's 10 columns; the user's earlier summary said 6 NEW + 4 existing"), but the table is still titled "seven" and the count `6 NEW` in the aggregate count at line 172 (`Columns added per table | 6 NEW`) is itself stale against the 10-column list. The doc has not been corrected.

**Expected:**

A consistent count (10 columns total, ~6 NEW) presented in the table header and the aggregate count.

**Evidence:**

- `docs/schemas/data-migration/04-column-additions.md:9` — `## The seven engine-invariant columns`
  - `docs/schemas/data-migration/04-column-additions.md:11-23` — table with 10 rows
  - `docs/schemas/data-migration/04-column-additions.md:172` — `| Columns added per table | 6 NEW (legacy had created_at, updated_at, sometimes active_status) |`

---

### FINDING 26 (id: `DOC-SCHM-026`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/05-brand-removal.md:99-136` vs `docs/specs/rbac/aggregates.md`, `docs/specs/rbac/repositories.md`, `docs/specs/rbac/value-objects.md`, `docs/specs/rbac/tables.md`, `docs/specs/rbac/overview.md`, `docs/specs/rbac/commands.md`, `docs/specs/hr/aggregates.md`, `docs/specs/hr/tables.md`, `docs/specs/hr/commands.md`, `docs/specs/operations/aggregates.md`, `docs/specs/operations/commands.md`, `docs/research/rbac-analysis.md`

**Description:**

The "Drops from docs" list in `05-brand-removal.md` specifies 16 specific doc edits (line ranges like "lines 189-216") that must be applied to remove `InfixRole` / `InfixPermissionAssign` / `is_saas` references from the spec tree. The expected target files (`docs/specs/rbac/*`, `docs/specs/hr/*`, `docs/specs/operations/*`, `docs/research/rbac-analysis.md`) reference 12 distinct file paths. None of those 16 edits can be verified because the spec-tree directories and files do not exist in the current repository.

**Expected:**

Either the target spec files exist and the migration has been completed (in which case the doc should be updated to point at the new line numbers), or the spec tree is incomplete and the migration is not yet executable.

**Evidence:**

- `docs/schemas/data-migration/05-brand-removal.md:99-136` enumerates 16 specific doc edits, each tied to a specific line number in a specific file:
    - `docs/specs/rbac/aggregates.md` lines 189-216, 218-244, 19, 32, 35-36
    - `docs/specs/rbac/repositories.md` lines 112-123, 125-135, 193
    - `docs/specs/rbac/value-objects.md` lines 18-19
    - `docs/specs/rbac/tables.md` lines 10-11, 37, 47
    - `docs/specs/rbac/overview.md` lines 80-81
    - `docs/specs/rbac/commands.md` line 32
    - `docs/specs/hr/aggregates.md` lines 92, 123
    - `docs/specs/hr/tables.md` line 38
    - `docs/specs/hr/commands.md` lines 184, 202
    - `docs/specs/operations/aggregates.md` line 265
    - `docs/specs/operations/commands.md` line 297
    - `docs/research/rbac-analysis.md` line 32
  - `docs/specs/` and `docs/research/` are listed in the AGENTS.md layout (`docs/specs/<domain>/`, `docs/research/`) but the per-domain spec files referenced by `05-brand-removal.md` do not exist as siblings of the doc.
  - The migration is presented as a Phase 5 to-do, not a done.

---

### FINDING 29 (id: `DOC-SCHM-029`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/01-engine-tables.md:71-86` vs `docs/schemas/data-migration/01-engine-tables.md:69-71`

**Description:**

The "Verify Phase 1" section ends with `SHOW INDEX FROM outbox;` / `SHOW INDEX FROM audit_log;` / `SHOW INDEX FROM idempotency;`. These are MySQL-specific statements. The same section is meant to verify Phase 1 across MySQL/SQLite/PostgreSQL, but the verification commands are MySQL-only.

**Expected:**

Per-dialect verification commands (or a note that the SQL shown is MySQL-specific and PostgreSQL/SQLite equivalents are required).

**Evidence:**

- `docs/schemas/data-migration/01-engine-tables.md:83-85`:
    ```sql
    SHOW INDEX FROM outbox;
    SHOW INDEX FROM audit_log;
    SHOW INDEX FROM idempotency;
    ```
  - `SHOW INDEX FROM` is MySQL syntax; PG uses `\d+ outbox` in `psql`, SQLite uses `PRAGMA index_list('outbox');`.

---

### FINDING 31 (id: `DOC-SCHM-031`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/06-field-data-flow.md:111-113` vs `docs/schemas/data-migration/06-field-data-flow.md:179-180`

**Description:**

The `academic_students` field map (table 1) declares `parent_id int FK sm_parents` becoming `guardian_id CHAR(36) FK academic_parents` (line 111), but the `users → platform_users` map (table 3) declares `role_id int(10) UNSIGNED NULL FK infix_roles (CASCADE)` becoming `CHAR(36) NOT NULL FK rbac_roles (RESTRICT)` (line 177-178) — the same column name `role_id` but in two different rows the `NOT NULL` semantics differ from the original. The doc does not flag this; both fields are rewritten as `NOT NULL` against a legacy nullable column.

**Expected:**

Either keep `NULL` semantics (engine may not require every user to hold a role) or document the semantic change explicitly.

**Evidence:**

- `docs/schemas/data-migration/06-field-data-flow.md:177-178` — `| role_id | int(10) UNSIGNED NULL FK infix_roles (CASCADE) | role_id | CHAR(36) NOT NULL FK rbac_roles (RESTRICT) | INT → UUIDv7; CASCADE → RESTRICT; tighten |`
  - Legacy `users.role_id` is nullable per the schema; engine forces it non-null with no documented backfill for users that have no role.

---

### FINDING 33 (id: `DOC-SCHM-033`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/06-field-data-flow.md:436-444` vs `docs/schemas/data-migration/05-brand-removal.md:178-193`

**Description:**

The end of `06-field-data-flow.md` shows the backfill query for `platform_packages.modules JSON ARRAYAGG(name)` reading from `sm_general_settings` flat-int columns. But `05-brand-removal.md:50-87` says those flat-int columns are dropped from `sm_general_settings` entirely (35 `DROP COLUMN` statements). The backfill in § 06 reads columns that § 05 drops — the two phases cannot both run as ordered.

**Expected:**

Either the modules JSON backfill runs before the `DROP COLUMN` (and the migration ordering in `00-overview.md` must put Phase 4 modules-backfill before Phase 5 drop), or the JSON-array is sourced from a snapshot, not the live `sm_general_settings` table.

**Evidence:**

- `docs/schemas/data-migration/06-field-data-flow.md:436-444`:
    ```sql
    UPDATE platform_packages pp
    JOIN sm_general_settings gs ON gs.school_id = pp.school_id
    SET pp.modules = JSON_ARRAYAGG(name) FROM (
      SELECT 'Lesson' AS name WHERE gs.Lesson = 1
      UNION ALL SELECT 'Chat' WHERE gs.Chat = 1
      -- ... 35 modules
    ) AS enabled;
    ```
  - `docs/schemas/data-migration/05-brand-removal.md:50-87` — `DROP COLUMN Lesson`, `DROP COLUMN Chat`, ... (35 columns dropped from `settings_general_settings`).

---

### FINDING 36 (id: `DOC-SCHM-036`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Medium
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/11-security.md:184-205` vs `docs/schemas/audit-schema.md:262-274`

**Description:**

The retention table in `11-security.md` (line 191-200) lists retention periods for `Authentication events | 18 months`, `Authorization denials | 36 months`, `AI agent actions | 36 months`. The retention table in `audit-schema.md` § 9 (line 266-275) lists the same record types but with different periods (`Authorization denials | 36 months` matches; `Authentication events | 18 months` matches; `AI agent actions | 36 months` matches) — the tables appear to agree. However, `11-security.md` omits the `Finance mutations | 7 years`, `Payroll mutations | 7 years`, `Capability / role changes | 7 years`, `Library / facilities mutations | 3 years`, `Backup events | 3 years` rows that are in `audit-schema.md`. The two tables should be aligned (or one should be the source of truth).

**Expected:**

A single retention table referenced from both files.

**Evidence:**

- `docs/schemas/data-migration/11-security.md:191-200` lists 8 record types.
  - `docs/schemas/audit-schema.md:264-275` lists 10 record types.
  - `Capability / role changes | 7 years` and `Payroll mutations | 7 years` and `Backup events | 3 years` are present in `audit-schema.md` but absent from `11-security.md`.

---

### FINDING 17 (id: `DOC-SCHM-017`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/comparison.md:246` vs `migrations/engine/0000_engine_core.surreal.surql`

**Description:**

The SurrealDB feature-comparison row claims `"Identifier quoting | backticks"`, but the canonical SurrealDB DDL file does not use backticks (see Finding 16). The comparison table is out of sync with the canonical artifact.

**Expected:**

The comparison row matches the canonical quoting convention used in the DDL.

**Evidence:**

- `docs/schemas/sql-dialects/comparison.md:246` — `| Identifier quoting | backticks | double-quotes | double-quotes | backticks |` (SurrealDB column)
  - `migrations/engine/0000_engine_core.surreal.surql` (full file) — no backtick characters appear anywhere in the DDL.

---

### FINDING 20 (id: `DOC-SCHM-020`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/sql-dialects/mysql.md:51`

**Description:**

The MySQL type-mapping table says `CHAR(36) (UUIDv7) | CHAR(36) | utf8mb4 charset; 36 chars = 36 bytes (4-byte chars)`. The parenthetical "(4-byte chars)" implies a UUID character occupies 4 bytes in `utf8mb4`. A UUID is ASCII (digits 0-9, letters a-f, hyphens), so each character occupies 1 byte in utf8mb4; the storage cost is 36 bytes, not 144. The parenthetical is misleading and contradicts the canonical 36-byte storage claim.

**Expected:**

A correct note such as `36 chars = 36 bytes (UUIDs are pure ASCII, so each char is 1 byte even in utf8mb4)`.

**Evidence:**

- `docs/schemas/sql-dialects/mysql.md:51` — `| CHAR(36) (UUIDv7) | CHAR(36) | utf8mb4 charset; 36 chars = 36 bytes (4-byte chars) |`

---

### FINDING 24 (id: `DOC-SCHM-024`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/03-domain-renames.md:396`

**Description:**

The Settings-domain rename list declares `transcations (typo) | (drop; legacy table was empty per 0009_finance.sql)`. The entry is listed under the Settings domain, but `transcations` is a finance-domain typo (a `transactions` table in the `migrations/0009_finance.sql` file) and is logically part of the Finance-domain rename map, not Settings.

**Expected:**

The typo drop is documented under the Finance domain or in the dedicated typo-fix section (`05-brand-removal.md:158`), not under Settings.

**Evidence:**

- `docs/schemas/data-migration/03-domain-renames.md:394` — `| transcations (typo) | (drop; legacy table was empty per 0009_finance.sql) |`
  - The same typo is also documented in `docs/schemas/data-migration/05-brand-removal.md:161` — `| transcations (table, 1 occurrence in migrations/0009_finance.sql) | (drop; table is empty) |`.

---

### FINDING 27 (id: `DOC-SCHM-027`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/00-overview.md:49-60`

**Description:**

The phase table in `00-overview.md` lists 10 numbered phases (0..8, plus rollback at "—" and security at "—"). The `README.md` index lists the same phases as 11 numbered phases plus the two "—" entries. The header text at line 44 says "eleven phases". The two doc files disagree on the total count.

**Expected:**

The phase list and the index use the same count and the same phase numbers.

**Evidence:**

- `docs/schemas/data-migration/00-overview.md:44` — `The migration runs in eleven phases, each with a focused file in this folder.`
  - `docs/schemas/data-migration/00-overview.md:47-60` (table) lists 10 numbered phases (0 through 8, plus 2 "—" entries)
  - `docs/schemas/data-migration/README.md:12-22` lists 12 numbered items (00 through 11) plus Rollback and Security

---

### FINDING 28 (id: `DOC-SCHM-028`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/01-engine-tables.md:53-58`

**Description:**

The "Apply order" section in `01-engine-tables.md` shows two bash commands: a `mysql` command for "MySQL / SQLite" that pipes `0000_engine_core.mysql.sql` into `devdb_v2`, and a `psql` command for "PostgreSQL" that does the same. Both commands feed the MySQL DDL to PostgreSQL. The file then notes "The PostgreSQL DDL differs only in identifier quoting (`outbox` vs `outbox`) and the `JSON` type vs `JSONB`", but feeds MySQL DDL to PG. The dialect mismatch is acknowledged in prose but the actual command is wrong.

**Expected:**

The PostgreSQL apply command uses `0000_engine_core.postgres.sql`, not `0000_engine_core.mysql.sql`.

**Evidence:**

- `docs/schemas/data-migration/01-engine-tables.md:49` — `mysql -u educore -p devdb_v2 < migrations/engine/0000_engine_core.mysql.sql`
  - `docs/schemas/data-migration/01-engine-tables.md:57` — `psql -U educore -d devdb_v2 -f migrations/engine/0000_engine_core.mysql.sql` (feeds MySQL DDL to psql)
  - `docs/schemas/data-migration/01-engine-tables.md:61-65` — `The PostgreSQL DDL differs only in identifier quoting ("outbox" vs `outbox`) and the JSON type vs JSONB.` (acknowledges dialect difference but still uses the MySQL file)

---

### FINDING 30 (id: `DOC-SCHM-030`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/05-brand-removal.md:50-87`

**Description:**

The brand-removal doc lists 35 `module_toggles` columns being dropped from `sm_general_settings`. The `DROP COLUMN` statement ends at `InAppLiveClass` (line 86) but the introductory text says "This drops 35 columns". Counting the columns in the `DROP` list: `Lesson, Chat, FeesCollection, InfixBiometrics, ResultReports, TemplateSettings, MenuManage, RolePermission, RazorPay, Saas, StudentAbsentNotification, ParentRegistration, Zoom, BBB, VideoWatch, Jitsi, OnlineExam, SaasRolePermission, BulkPrint, HimalayaSms, XenditPayment, Wallet, Lms, ExamPlan, University, Gmeet, KhaltiPayment, Raudhahpay, AppSlider, BehaviourRecords, DownloadCenter, AiContent, WhatsappSupport, InAppLiveClass` = 34 columns.

**Expected:**

Either 34 in the count or 35 in the `DROP` list.

**Evidence:**

- `docs/schemas/data-migration/05-brand-removal.md:89-90` — `This drops 35 columns. The engine's module system is capability-based, not flag-based.`
  - `docs/schemas/data-migration/05-brand-removal.md:53-86` lists 34 `DROP COLUMN` statements (33 unique column names + `InfixBiometrics` which is also renamed).
  - `docs/schemas/data-migration/05-brand-removal.md:206` — `| Module-toggle flat-int columns dropped | 35 |` — repeats the 35 figure in the aggregate count.

---

### FINDING 32 (id: `DOC-SCHM-032`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/06-field-data-flow.md:270-280`

**Description:**

The `sm_staffs → hr_staffs` field map (table 8) lists `is_saas int DEFAULT 0` becoming `is_saas_staff BOOLEAN NOT NULL DEFAULT FALSE`. Other parts of the same migration plan (`05-brand-removal.md:146-150`) rename `is_saas` to `is_replicated` on roles and `is_system_defined` on other aggregates. The HR-specific rename to `is_saas_staff` (preserving the brand-tainted prefix) is inconsistent with the broader migration direction.

**Expected:**

A consistent rename rule for `is_saas` across all aggregates (the migration's stated direction is to drop the `is_saas` brand artifact).

**Evidence:**

- `docs/schemas/data-migration/06-field-data-flow.md:280` — `| is_saas | int DEFAULT 0 | is_saas_staff | BOOLEAN NOT NULL DEFAULT FALSE | rename; tighten |`
  - `docs/schemas/data-migration/05-brand-removal.md:146-150` — `is_saas → is_replicated` (rbac_roles, rbac_permission_assigns), `is_system_defined` (hr_departments, hr_designations, etc.)
  - `is_saas_staff` is not referenced anywhere else in the migration plan.

---

### FINDING 34 (id: `DOC-SCHM-034`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/07-verification.md:144-152`

**Description:**

The "UUIDv7 derivation verification" SQL block is incomplete: it shows `UUID_FROM_BIN( CONCAT( UNHEX(LPAD(HEX(UNIX_TIMESTAMP() * 1000), 12, '0')), -- simplified -- ... full derivation per 02-id-conversion.md ) )` and `CASE WHEN c.id = UUID_FROM_BIN(...) THEN 'MATCH' ELSE 'MISMATCH' END`. The `UUID_FROM_BIN(...)` expression is left as an ellipsis. The verification script as written cannot be executed; consumers must rewrite the derivation from scratch.

**Expected:**

A complete, executable verification query that runs the actual `uuid_v7(namespace, legacy_id)` derivation from `02-id-conversion.md:78-86`.

**Evidence:**

- `docs/schemas/data-migration/07-verification.md:144-152`:
    ```sql
    UUID_FROM_BIN(
      CONCAT(
        UNHEX(LPAD(HEX(UNIX_TIMESTAMP() * 1000), 12, '0')),  -- simplified
        -- ... full derivation per 02-id-conversion.md
      )
    ) AS expected_id,
    CASE WHEN c.id = UUID_FROM_BIN(...) THEN 'MATCH' ELSE 'MISMATCH' END
    ```

---

### FINDING 35 (id: `DOC-SCHM-035`)

- **Source:** `docs/audit_reports/findings/wave5-docs-5.md`
- **Severity:** Low
- **Area:** documentation
- **Location:** `docs/schemas/data-migration/11-security.md:14-17`

**Description:**

The "credential in git history" section presents `DATABASE_URL="mysql://devuser:paxxw0rd@2791@127.0.0.1:3306/devdb"` as a real credential that needs rotation. The URL contains the literal string `paxxw0rd@2791` (which has a stray `@` in the middle, breaking URL parsing — the host portion `@2791@127.0.0.1` is not valid). The doc reproduces the credential verbatim in the rotation procedure (line 14-17) and the verification grep commands (lines 91-92). Any consumer copy-pasting this URL into a `.env` for testing will fail URL parsing; anyone running the verification grep will match the rotation doc itself.

**Expected:**

A redacted placeholder (`mysql://devuser:<REDACTED>@127.0.0.1:3306/devdb`) in the doc body, with the real credential recorded only in a credential vault or an out-of-band reference.

**Evidence:**

- `docs/schemas/data-migration/11-security.md:14-17`:
    ```
    DATABASE_URL="mysql://devuser:paxxw0rd@2791@127.0.0.1:3306/devdb"
    ```
  - `docs/schemas/data-migration/11-security.md:91-92`:
    ```
    git log -p --all -- .env | grep -i paxxw0rd
    git log -p --all -- .env | grep -i 2791
    ```
  - The grep commands will match the `11-security.md` file itself (the verbatim occurrence), causing false positives on every run.

---


## Guides (target id prefix: `DOC-6`)

**Path:** `docs/guides/`  
**Total findings:** 50 (34 critical, 12 high, 4 medium, 0 low)


### FINDING 1 (id: `DOC-6-001`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:38-49`

**Description:**

The SaaS guide claims the engine ships "15 domain crates" and lists them implicitly via "10 domain bounded contexts" elsewhere. Per `AGENTS.md` § Crate Inventory, the engine has exactly **10** domain crates (academic, assessment, attendance, cms, communication, documents, facilities, finance, hr, library) and the **10th** is `educore-events-domain` only as a **cross-cutting** crate (Phase 13). The phrase "15 domain crates" overstates the count and conflates `educore-events-domain` (cross-cutting calendar domain, Phase 13) with a true domain crate.

**Expected:**

10 domain crates per `AGENTS.md` Crate Inventory table; `educore-events-domain` is cross-cutting (calendar), not a domain bounded context.

**Evidence:**

- `docs/guides/saas-backend.md:38-49` — "15 domain crates (`educore-academic`, `educore-finance`, ..., `educore-events-domain`)."
  - `AGENTS.md` Crate Inventory — only 10 domain crates are listed under tier `domains`: `educore-academic`, `educore-assessment`, `educore-attendance`, `educore-cms`, `educore-communication`, `educore-documents`, `educore-facilities`, `educore-finance`, `educore-hr`, `educore-library`.
  - `AGENTS.md` Tier System — `educore-events-domain` is explicitly listed under the `cross-cutting` tier, not `domains`.

---

### FINDING 10 (id: `DOC-6-010`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:644-690` (Client Tauri sketch)

**Description:**

The client-side Tauri example uses `Engine::builder()`, `JwtAuthProvider::from_env()`, `InProcessBus::new()`, `UuidV7Generator::new()`, `SqliteStorage::open(&local_db)`. None of these match reality: `EngineBuilder::new()` (no `Engine::builder()`), `JwtAuthProviderBuilder::new()` (no `from_env`), `InProcessEventBus::new()` (no `InProcessBus`), `SystemIdGen` (no `UuidV7Generator`), and the SQLite adapter constructor signature is unknown without checking the actual crate. Also `tauri::Builder::default().manage(engine)` requires `Engine: Send + Sync + 'static`, which the engine may or may not satisfy without explicit bounds.

**Expected:**

Real client-side builder pattern.

**Evidence:**

- `docs/guides/saas-backend.md:644-690` — full Tauri client sketch.
  - `crates/tools/sdk/src/engine.rs:179, 258` — `EngineBuilder::new()`, `build()` returns `Result<Engine, SdkError>` (sync).
  - `crates/adapters/event-bus/src/in_process.rs:123, 161` — `InProcessEventBus` (not `InProcessBus`).
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` unit struct.

---

### FINDING 11 (id: `DOC-6-011`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:717-751` (Observability / AuditSink)

**Description:**

The example defines `pub struct OtelAuditSink;` and `impl AuditSink for OtelAuditSink { async fn record(&self, entry: AuditEntry) -> Result<()> { ... } }`. Per the actual `crates/cross-cutting/audit/src/sink.rs`, the `AuditSink` trait method signature is `async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError>` (or similar — the doc method `record` may not exist). The guide's `record` method does not match the actual trait. Also `OtelAuditSink::from_env()` is referenced in the builder section — a method that does not exist.

**Expected:**

`impl AuditSink for OtelAuditSink { async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> { ... } }`.

**Evidence:**

- `docs/guides/saas-backend.md:268` — `let audit = Arc::new(OtelAuditSink::from_env()?);`.
  - `docs/guides/saas-backend.md:730-748` — `impl AuditSink for OtelAuditSink { async fn record(&self, entry: AuditEntry) -> Result<()> { ... } }`.
  - `crates/cross-cutting/audit/src/sink.rs` — actual `AuditSink` trait, method name and signature differ.

---

### FINDING 13 (id: `DOC-6-013`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/multi-tenancy.md:9-21` (TenantContext struct)

**Description:**

The `TenantContext` struct shown has fields `school_id`, `user_id`, `session_id`, `correlation_id`, `clock: Arc<dyn Clock>`. This differs from the SaaS guide's version (which had no `session_id` or `clock` and had `causation_id`) and from `crates/cross-cutting/platform/src/tenant.rs`. The `clock` field belongs in command inputs, not the `TenantContext` (which is a value object that flows through every command). Including `Arc<dyn Clock>` in the context forces every caller to construct an `Arc` on every command, defeating the purpose of an injectable port. The two guides disagree on field names, indicating neither is authoritative.

**Expected:**

Authoritative `TenantContext` struct from `crates/cross-cutting/platform/src/tenant.rs`.

**Evidence:**

- `docs/guides/multi-tenancy.md:9-21` — `pub struct TenantContext { pub school_id: SchoolId, pub user_id: UserId, pub session_id: SessionId, pub correlation_id: CorrelationId, pub clock: Arc<dyn Clock>, }`.
  - `docs/guides/saas-backend.md:355-365` — different field set (no `session_id`, no `clock`, has `causation_id`).
  - The two guides disagree on the actual struct.

---

### FINDING 14 (id: `DOC-6-014`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/multi-tenancy.md:53-60` (Cross-Tenant Operations)

**Description:**

The guide states "The engine models these as explicit commands: `TransferStudentCommand { source_school_id, destination_school_id, ... }`". Per the actual `crates/domains/academic/src/commands.rs` and `crates/domains/academic/src/events.rs`, the command struct has fields `student_id`, `destination_school_id`, `actor_id`, `effective_at`, `reason` — not `source_school_id` (the source is the aggregate's existing `SchoolId`). Also `StudentTransferred` event has fields `student_id`, `from_school_id`, `to_school_id` (not `source_school_id`/`destination_school_id`). The naming is wrong.

**Expected:**

Command field is `student_id` (the source is the aggregate's existing `school_id`); event field names are `from_school_id`/`to_school_id`.

**Evidence:**

- `docs/guides/multi-tenancy.md:53-60` — `TransferStudentCommand { source_school_id, destination_school_id, ... }` and `pub struct StudentTransferred { source_school_id, destination_school_id, ... }`.
  - `crates/domains/academic/src/commands.rs` — actual `TransferStudentCommand` fields (per Phase 3 academic crate).
  - `crates/domains/academic/src/events.rs` — actual `StudentTransferred` event uses `from_school_id`/`to_school_id`.

---

### FINDING 16 (id: `DOC-6-016`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/audit-trail.md:8-14` (AuditSink trait)

**Description:**

The guide's `AuditSink` trait has methods `write(record: AuditRecord) -> Result<()>` and `query(q: AuditQuery) -> Result<Vec<AuditRecord>>`. Per the actual `crates/cross-cutting/audit/src/writer.rs`, the trait method is `pub async fn write(&self, ...)` and takes the record by reference or by owned value depending on the impl. The `query` method shown here does not exist on `AuditSink`; querying is a separate `AuditQueryService` port (per `crates/cross-cutting/audit/src/`). Mixing write and query responsibilities into one port violates the "single responsibility per port" rule in `docs/code-standards.md`.

**Expected:**

`AuditSink` has only `write(&self, record: &AuditRecord) -> Result<(), AuditError>`; queries live on a separate `AuditQueryService` port.

**Evidence:**

- `docs/guides/audit-trail.md:8-14` — `pub trait AuditSink: Send + Sync { async fn write(&self, record: AuditRecord) -> Result<()>; async fn query(&self, q: AuditQuery) -> Result<Vec<AuditRecord>>; }`.
  - `crates/cross-cutting/audit/src/writer.rs:824` — `pub async fn write(...)` is the actual signature (different signature).
  - `crates/cross-cutting/audit/src/lib.rs` — no `query` method on `AuditSink`.

---

### FINDING 17 (id: `DOC-6-017`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/audit-trail.md:17-42` (AuditRecord struct)

**Description:**

The `AuditRecord` struct shows fields including `actor_capabilities: Vec<Capability>`, `before: Option<serde_json::Value>`, `after: Option<serde_json::Value>`, `diff: Option<JsonDiff>`, `signature: Option<DigitalSignature>`. Per `docs/code-standards.md` § Code Standards: "No `serde_json::Value` in domain code. Use typed wrappers." The presence of `before`/`after` as `serde_json::Value` violates the engine's own rule. Also `JsonDiff` is not a real type; the engine emits typed diffs via the audit event types.

**Expected:**

Audit record uses typed wrappers (e.g. `AuditSnapshot<'a>` with typed field accessors), not `serde_json::Value`.

**Evidence:**

- `docs/guides/audit-trail.md:17-42` — `pub before: Option<serde_json::Value>, pub after: Option<serde_json::Value>, pub diff: Option<JsonDiff>`.
  - `docs/code-standards.md` § Code Standards — "No `serde_json::Value` in domain code. Use typed wrappers."

---

### FINDING 18 (id: `DOC-6-018`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/audit-trail.md:111-119` (Worked Example: TeeAuditSink)

**Description:**

The example wires `TeeAuditSink::new(PostgresAuditSink::new(pool.clone()), RedisAuditSink::new(redis_client.clone()))` into `EngineBuilder::audit(audit)`. Per the actual SDK, the builder method is `audit_sink(...)` (not `audit(...)`) per the SaaS guide's own builder example (line 268). The two guides disagree on the builder method name. Also `RedisAuditSink` and `TeeAuditSink` are phantom types — neither exists in the workspace.

**Expected:**

`EngineBuilder::audit_sink(...)` (consistent with the SaaS guide and SDK); no `RedisAuditSink`/`TeeAuditSink` (consumer-implemented).

**Evidence:**

- `docs/guides/audit-trail.md:111-119` — `EngineBuilder::audit(audit)` + `TeeAuditSink::new(PostgresAuditSink::new(...), RedisAuditSink::new(...))`.
  - `docs/guides/saas-backend.md:268` — `let audit = Arc::new(OtelAuditSink::from_env()?);` and `.audit_sink(audit)` in the builder.
  - `crates/tools/sdk/src/engine.rs` — actual builder method name (per Phase 16 SDK impl).

---

### FINDING 19 (id: `DOC-6-019`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/offline-sync.md:15-23` (Sync implementation)

**Description:**

The guide says `Engine::builder().sync(EducoreSyncAdapter::in_process())` and `Engine::builder().sync(WorkerHttpSyncAdapter::connect(url, token))`. Per `crates/tools/sdk/src/engine.rs` (lines 123-147, 176-258), the `EngineBuilder` has no `.sync(...)` method. The `educore-sync` port at `crates/cross-cutting/sync/src/` exposes `SyncAdapter` but the SDK does not wire it into `EngineBuilder`. The umbrella `crates/educore/src/lib.rs` also does not re-export `educore_sync`. Neither `EducoreSyncAdapter` nor `WorkerHttpSyncAdapter` exists in the workspace — the actual in-process adapter is `InProcessSyncAdapter` at `crates/cross-cutting/sync-inprocess/src/lib.rs:72`.

**Expected:**

Consumer wires `Arc<dyn SyncAdapter>` directly via the `educore_sync` port; no `Engine::builder().sync(...)` exists.

**Evidence:**

- `docs/guides/offline-sync.md:15-23` — `Engine::builder().sync(EducoreSyncAdapter::in_process())` and `Engine::builder().sync(WorkerHttpSyncAdapter::connect(url, token))`.
  - `crates/tools/sdk/src/engine.rs:176-258` — `EngineBuilder` has no `sync` method.
  - `crates/cross-cutting/sync-inprocess/src/lib.rs:72` — `pub struct InProcessSyncAdapter` (not `EducoreSyncAdapter`).
  - `find crates -name "WorkerHttpSyncAdapter"` — no match.

---

### FINDING 2 (id: `DOC-6-002`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:42-43`

**Description:**

The guide claims SurrealDB is the primary storage adapter ("4 shipped storage adapters (SurrealDB primary embedded, PostgreSQL, MySQL, SQLite)"). Per `AGENTS.md` § Storage Adapters, SurrealDB is **deferred to a future release** and **not shipped from the engine**. The 3 shipped adapters are PostgreSQL, MySQL, SQLite. MongoDB is also deferred (not SurrealDB). The same error is repeated at line 994 in the Reference Map row "Storage adapter | `educore-storage-{postgres,mysql,sqlite}`" — the Reference Map is internally inconsistent with the Library Boundary section.

**Expected:**

SurrealDB listed as deferred, not primary; shipped adapters are PostgreSQL, MySQL, SQLite.

**Evidence:**

- `docs/guides/saas-backend.md:42` — "4 shipped storage adapters (SurrealDB primary embedded, PostgreSQL, MySQL, SQLite) and 1 deferred (`educore-storage-mongodb`)."
  - `AGENTS.md` Storage Adapters — "Three reference adapters are shipped: `educore-storage-surrealdb` (primary target), `educore-storage-mysql` (production target, MySQL 8.0+), `educore-storage-sqlite` (embedded / offline mode). The SurrealDB and MongoDB adapters are **deferred to a future release** and are **not** shipped from the engine."
  - `AGENTS.md` Crate Inventory — Phase 0 includes `educore-storage-surrealdb` as scaffold only (Phase 0 entry: "Foundation (SurrealDB adapter, primary)"); Phase 1 implements only `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`. There is no `educore-storage-mongodb` scaffold at all.
  - `docs/guides/saas-backend.md:994` — Reference Map row "Storage adapter | `educore-storage-{postgres,mysql,sqlite}`" — contradicts the Library Boundary section's claim of 4 shipped adapters.

---

### FINDING 20 (id: `DOC-6-020`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/offline-sync.md:140-167` (Worked Example)

**Description:**

The example uses `engine.students().admit(cmd)`, `InProcessBus::new()`, `Engine::builder().storage(...).audit(...).event_bus(...).build().await?`, and references `OfflineQueue::load(storage.clone())?` and `OfflineQueue::replay(&engine)`. None of these exist: the actual engine has no `engine.students()` method (only `engine.admission()`), no `InProcessBus` (it's `InProcessEventBus`), and there is no `OfflineQueue` type in the workspace — offline replay is the consumer's responsibility using the `SyncAdapter` port, not an engine-built queue.

**Expected:**

`engine.admission().admit(cmd).await?`; `InProcessEventBus::new()`; no built-in `OfflineQueue` (consumer implements).

**Evidence:**

- `docs/guides/offline-sync.md:140-167` — full client example using `engine.students()`, `InProcessBus::new()`, `OfflineQueue`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `students()` method.
  - `crates/adapters/event-bus/src/in_process.rs:123` — `pub struct InProcessEventBus`.
  - `find crates -name "offline_queue.rs" -o -name "OfflineQueue*"` — no matches.

---

### FINDING 21 (id: `DOC-6-021`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/capability-rbac.md:113-138` (Engine::dispatch method)

**Description:**

The guide shows `impl Engine { pub async fn dispatch(&self, cmd: BoxedCommand) -> Result<BoxedOutcome> { ... } }`. Per the actual SDK (`crates/tools/sdk/src/engine.rs`), there is no `Engine::dispatch` method. The engine dispatches commands via service-level methods (`engine.admission().admit(cmd)`, etc.), not through a generic `dispatch` method with `BoxedCommand`. The pattern is service-typed, not type-erased. The check pattern shown (capability → tenant → handler) also uses `DomainError::forbidden(...)` while the actual `DomainError` variant is `Forbidden(String)` per `crates/infra/core/src/error.rs:19-63`.

**Expected:**

Service-typed dispatch (`engine.<service>().<method>(cmd)`), no generic `dispatch`; `DomainError::Forbidden(String)` factory.

**Evidence:**

- `docs/guides/capability-rbac.md:113-138` — `impl Engine { pub async fn dispatch(&self, cmd: BoxedCommand) -> Result<BoxedOutcome> { ... } }`.
  - `crates/tools/sdk/src/engine.rs` — no `dispatch` method.
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { ... Forbidden(String) ... }` (positional arg, no `format!` factory shown).

---

### FINDING 23 (id: `DOC-6-023`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/storage-adapter.md:5-30` (Adapter Skeleton)

**Description:**

The adapter skeleton shows `impl StorageAdapter for PostgresStorage { async fn begin(&self) -> Result<Transaction> { ... } }`. Per Wave 5 docs-4 finding DOC-PORT-002, the actual `StorageAdapter` trait in `crates/infra/storage/src/port.rs:34-150` does NOT carry per-aggregate repository accessors like `students() -> Arc<dyn StudentRepository>`. The adapter also does not have `migrate()`, `connect()`, or `open()` methods in the form shown; the actual adapter has `migrate()`, `ping()`, `close()`, `bulk_insert_student_attendances(...)`, `watch_changes()`, `apply_snapshot()`, `cursor_for()`, `advance_cursor()`, plus `begin()`. The skeleton is wrong.

**Expected:**

Real `StorageAdapter` trait method list (per docs-4 finding DOC-PORT-002).

**Evidence:**

- `docs/guides/storage-adapter.md:5-30` — adapter skeleton with `begin()` only.
  - `crates/infra/storage/src/port.rs:34-150` — actual trait with 9 methods, no per-aggregate repository accessors.

---

### FINDING 24 (id: `DOC-6-024`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/storage-adapter.md:165-180` (Tenant Isolation wrapper)

**Description:**

The guide shows `fn with_school_filter(&self, sql: &mut String) { if !sql.contains("school_id =") { panic!("query missing school_id filter: {}", sql); } }`. The use of `panic!` in adapter code violates `docs/code-standards.md` § Code Standards ("`unwrap`, `expect`, `panic!` are forbidden in production paths"). Additionally, the panic message implies string-based SQL inspection (fragile), and the actual engine enforces `school_id` filtering via the typed query AST (no string matching required).

**Expected:**

No `panic!` in adapter code; the typed query AST already enforces `school_id` (the macro-emitted query builder always includes the school id filter).

**Evidence:**

- `docs/guides/storage-adapter.md:165-180` — `if !sql.contains("school_id =") { panic!(...) }`.
  - `docs/code-standards.md` § Code Standards — "`unwrap`, `expect`, `panic!` are forbidden in production paths."

---

### FINDING 26 (id: `DOC-6-026`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/crud-patterns.md:30-49` (Create pattern)

**Description:**

The pattern shows `impl CreateClassCommand { pub async fn execute(self, repo: &dyn ClassRepository, events: &mut Outbox) -> Result<Class> { ... } }`. Per `docs/code-standards.md` and the actual domain pattern in `crates/domains/academic/`, the command is dispatched via `engine.academic().create_class(cmd).await?` (or equivalent service-typed call), not via a method on the command struct with explicit `repo` and `events` arguments. The consumer never injects `repo` or `events` directly — that is internal wiring. Also `events: &mut Outbox` implies mutation, but the outbox is accessed via `&Outbox` (read-only handle).

**Expected:**

Service-typed dispatch (`engine.<domain>().<verb>(cmd).await?`); no manual repo/outbox injection in the command struct.

**Evidence:**

- `docs/guides/crud-patterns.md:30-49` — `impl CreateClassCommand { pub async fn execute(self, repo: &dyn ClassRepository, events: &mut Outbox) -> Result<Class> { ... } }`.
  - `crates/tools/sdk/src/engine.rs` — service-typed methods, no command-struct `execute`.
  - `docs/guides/saas-backend.md:278-291` — example uses `engine.<service>().<verb>(cmd)`.

---

### FINDING 27 (id: `DOC-6-027`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/idempotent-commands.md:64-72` (IdempotencyStore trait)

**Description:**

The guide shows `pub trait IdempotencyStore: Send + Sync { async fn lookup(&self, key: IdempotencyKey, command: &str) -> Result<Option<CommandOutcome>>; async fn record(&self, key: IdempotencyKey, command: &str, outcome: &CommandOutcome) -> Result<()>; }`. Per Wave 5 docs-4 finding DOC-PORT-004, the actual `Idempotency` port in `crates/infra/storage/src/transaction.rs` (accessed via `Transaction::idempotency() -> &dyn Idempotency`) has a different signature. The trait methods also reference `CommandOutcome { status: OutcomeStatus, payload: serde_json::Value, events: Vec<EventId> }` which uses `serde_json::Value` (forbidden in domain code per `docs/code-standards.md`).

**Expected:**

Real `Idempotency` trait signature; typed `CommandOutcome` (no `serde_json::Value`).

**Evidence:**

- `docs/guides/idempotent-commands.md:64-72` — `IdempotencyStore` trait + `CommandOutcome` with `serde_json::Value`.
  - `crates/infra/storage/src/transaction.rs:60-70` — actual `Idempotency` trait (different signature).
  - `docs/code-standards.md` — no `serde_json::Value` in domain code.

---

### FINDING 28 (id: `DOC-6-028`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/event-replay.md:96-120` (Worked Example: Building a New Projection)

**Description:**

The example uses `event.event_type` as a `&str` (string comparison `"StudentAdmitted"`) and `serde_json::from_value(event.payload.clone())?` to deserialize the payload. This violates `docs/code-standards.md` § Engine Rule 2 ("Compile-time safety over strings. Use macro-generated enums — never string field names") and the `serde_json::Value` rule. The actual replay API uses typed event enums and a closed `EventEnvelope` payload.

**Expected:**

Replay over typed events (closed enum dispatch), no `serde_json::from_value` in domain code.

**Evidence:**

- `docs/guides/event-replay.md:96-120` — `match event.event_type { "StudentAdmitted" => { let payload: StudentAdmitted = serde_json::from_value(event.payload.clone())?; ... } }`.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names" + "No `serde_json::Value` in domain code."

---

### FINDING 29 (id: `DOC-6-029`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/event-replay.md:32-40` (Incremental Replay)

**Description:**

The example calls `store.read_from(last_offset).await?` and `projection.record_offset(envelope.event_id).await?` and `store.read_snapshot(projection_id, since)`. Per the actual `crates/infra/storage/src/port.rs`, there is no `EventStore` trait with `read_all()`, `read_from()`, or `read_snapshot()` methods. The event store concept is split: the `outbox` provides append/pending (per Wave 5 docs-4 DOC-PORT-003) and `event_log` is a separate sub-port. There is no `Projection::record_offset` API.

**Expected:**

Replay reads from `Outbox::pending()` and `EventLog` sub-port; no `EventStore` monolith.

**Evidence:**

- `docs/guides/event-replay.md:32-40` — `store.read_all()`, `store.read_from(last_offset)`, `store.read_snapshot(projection_id, since)`.
  - `crates/infra/storage/src/port.rs:34-150` — no `EventStore` trait; `outbox` has only `append`/`pending`/`mark_published`/`pending_count`.

---

### FINDING 3 (id: `DOC-6-003`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:217-275` (The Thin Backend)

**Description:**

The "Thin Backend" example uses identifiers that do not exist on the engine: `MysqlStorage::builder()`, `JwtAuthProvider::builder()`, `EmailNotifier::from_env()`, `StripePaymentProvider::from_env()`, `S3FileStorage::from_env()`, `NatsBus::from_env()`, `Engine::builder()`, `UuidV7Generator::new()`, `OtelAuditSink::from_env()`, `engine.students().with_tenant(&tenant).admit(cmd)`, `engine.auth().validate(&token)`, `engine.rbac().require(&session, Capability::StudentsAdmit)`, `engine.platform().query_schools(...)`, `engine.platform().suspend_school(...)`, `engine.finance().record_external_payment(...)`, `engine.handle_synced_event(...)`. Per Wave 5 docs-3 finding DOC-LIB-001/004 and the actual `crates/tools/sdk/src/engine.rs`, none of these exist. The builder is `EngineBuilder::new()` (sync build returning `SdkError`), the JWT builder is `JwtAuthProviderBuilder::new()` (no `from_env`), the notifier struct is `EmailProvider` (no `EmailNotifier`, no `from_env`), the engine has no `students()`/`auth()`/`rbac()`/`platform()`/`finance()` method.

**Expected:**

Multiple non-existent API surfaces.

**Evidence:**

- `docs/guides/saas-backend.md:217-275` — full builder example using `Engine::builder()`, `.build().await?`, `JwtAuthProvider::builder()`, `EmailNotifier::from_env()`, `StripePaymentProvider::from_env()`, `S3FileStorage::from_env()`, `NatsBus::from_env()`, `UuidV7Generator::new()`, `OtelAuditSink::from_env()`, `engine.students()`, `engine.auth()`, `engine.rbac()`, `engine.platform()`, `engine.finance()`, `engine.handle_synced_event()`.
  - `crates/tools/sdk/src/engine.rs:123-147` — `Engine` exposes `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. No `students()`, `rbac()`, `platform()`, `finance()`, `auth()`-as-engine-method (note: `auth()` is the storage-port-handle on the SDK, not a method that validates a token).
  - `crates/tools/sdk/src/engine.rs:179, 258` — `pub fn new() -> Self { ... }`, `pub fn build(self) -> Result<Engine, SdkError> { ... }` — `EngineBuilder::new()`, sync build.
  - `crates/adapters/notify/src/email.rs:75, 204-217, 261` — `pub struct EmailProvider` (not `EmailNotifier`), `EmailProviderBuilder::new()`; no `from_env`.
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` (unit struct), no `UuidV7Generator`.

---

### FINDING 30 (id: `DOC-6-030`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/notification-templates.md:5-21` (NotificationTemplate struct)

**Description:**

The `NotificationTemplate` struct has `pub channel: Channel` with `Channel::Email { from: None, reply_to: None }`. Per the actual `crates/domains/communication/src/aggregate.rs` and `crates/adapters/notify/src/`, the `Channel` type is a port-defined enum on `NotificationProvider` (e.g. `Channel::Email`, `Channel::Sms`, `Channel::Push`), not a struct with `from`/`reply_to` named fields. The named-field construction `Channel::Email { from: None, reply_to: None }` is phantom syntax.

**Expected:**

`pub channel: Channel` where `Channel` is an enum with unit variants `Email`, `Sms`, `Push`, `Webhook`; sender/reply-to are template-level fields, not channel payload.

**Evidence:**

- `docs/guides/notification-templates.md:5-21` — `pub channel: Channel` + `Channel::Email { from: None, reply_to: None }`.
  - `crates/adapters/notify/src/` — actual `Channel` definition.

---

### FINDING 32 (id: `DOC-6-032`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/fee-collection.md:35-90` (Setup example)

**Description:**

The full fees workflow uses `engine.fees().create_group(...)`, `engine.fees().create_type(...)`, `engine.fees().create_master(...)`, `engine.fees().assign_to_class(...)`, `engine.fees().generate_invoices(...)`, `engine.fees().record_payment(...)`. Per the SDK, no `engine.fees()` method exists. The actual consumer calls `educore_finance::services::create_fees_group(cmd, &ctx).await?` etc. The command field names (`fees_master_id`, `fees_assign_id`) also conflict with `crates/domains/finance/src/value_objects.rs` (where the field is `master_id`, not `fees_master_id`).

**Expected:**

Domain service functions; field names per `crates/domains/finance/src/value_objects.rs`.

**Evidence:**

- `docs/guides/fee-collection.md:35-90` — `engine.fees().create_group(...)`, etc.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `fees()` method.

---

### FINDING 33 (id: `DOC-6-033`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/report-card-generation.md:33-48` (engine.assessment().enter_marks)

**Description:**

The example uses `engine.assessment().enter_marks(EnterMarksCommand { ... })`. Per the SDK, no `engine.assessment()` method exists. The actual consumer calls `educore_assessment::services::enter_marks(cmd, &ctx).await?`. The command's `student_records: vec![StudentMark { student_id, marks: 85.0, absent: false }]` also has `marks: f64` (or `f32`), but `docs/code-standards.md` forbids `as` casts and value objects prefer typed wrappers (`Marks` value object with validation).

**Expected:**

Service-typed dispatch; `Marks` value object (not raw `f64`/`f32`).

**Evidence:**

- `docs/guides/report-card-generation.md:33-48` — `engine.assessment().enter_marks(EnterMarksCommand { ... student_records: vec![StudentMark { student_id, marks: 85.0, ... }] })`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `assessment()` method.

---

### FINDING 34 (id: `DOC-6-034`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/payroll-calculation.md:78-101` (engine.hr().assign_salary_template, generate_payroll, approve_payroll)

**Description:**

All payroll examples use `engine.hr()...` and `engine.finance().record_payroll_payment(...)`. Neither `engine.hr()` nor `engine.finance()` exists on the SDK (`crates/tools/sdk/src/engine.rs`). The actual consumer calls `educore_hr::services::*` and `educore_finance::services::*` directly. The `GeneratePayrollCommand::period: PayPeriod { year: 2026, month: 6 }` field also conflicts with `crates/domains/hr/src/value_objects.rs` where the period type uses `start: NaiveDate, end: NaiveDate`, not a `PayPeriod` struct.

**Expected:**

Service-typed dispatch; `PayrollPeriod { start: NaiveDate, end: NaiveDate }` value object.

**Evidence:**

- `docs/guides/payroll-calculation.md:78-101` — `engine.hr().assign_salary_template(...)`, `engine.hr().generate_payroll(GeneratePayrollCommand { period: PayPeriod { year: 2026, month: 6 }, ... })`, `engine.finance().record_payroll_payment(...)`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `hr()` or `finance()` method.

---

### FINDING 35 (id: `DOC-6-035`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/ai-agent-integration.md:54-72` (Tool struct)

**Description:**

The `Tool` struct has `pub input_schema: serde_json::Value, pub output_schema: serde_json::Value` and the example uses `engine.tools().for_session(agent_session)`. Per `docs/code-standards.md`, `serde_json::Value` is forbidden in domain code. Per the SDK, no `engine.tools()` method exists. The actual tool catalog uses a typed `ToolDescriptor` (or macro-emitted enum) with `JsonSchema` (the `schemars` crate's typed wrapper), not raw `serde_json::Value`.

**Expected:**

Typed `ToolDescriptor` with `schemars::schema::Schema` (or similar); no `engine.tools()` (consumer-implemented).

**Evidence:**

- `docs/guides/ai-agent-integration.md:54-72` — `pub struct Tool { ... pub input_schema: serde_json::Value, pub output_schema: serde_json::Value, ... }` and `engine.tools().for_session(agent_session)`.
  - `docs/code-standards.md` — "No `serde_json::Value` in domain code."

---

### FINDING 36 (id: `DOC-6-036`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/ai-agent-integration.md:138-156` (educore-agent-test crate)

**Description:**

The guide references `educore-agent-test` crate and `TestAgent::new("test-agent", capabilities![...])`. Per the `AGENTS.md` Crate Inventory, no `educore-agent-test` crate exists in the 34-crate inventory. The actual test utilities live in `crates/tools/testkit/` (scaffolded at Phase 16) and do not include an agent simulator. The example `agent.invoke("Mark John Doe present today.").await?` is also a phantom API.

**Expected:**

No `educore-agent-test` crate; no `TestAgent`. Agent testing is consumer's responsibility using mock harnesses.

**Evidence:**

- `docs/guides/ai-agent-integration.md:138-156` — `let agent = TestAgent::new("test-agent", capabilities![...]); let outcome = agent.invoke("Mark John Doe present today.").await?;`.
  - `AGENTS.md` Crate Inventory — no `educore-agent-test` crate; only `educore-testkit` (Phase 16) and `educore-storage-parity` (Phase 0+16).

---

### FINDING 39 (id: `DOC-6-039`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/test-strategy.md:67-80` (Component tests)

**Description:**

The example uses `engine.students().admit(AdmitStudentCommand { tenant, ... })` and `engine.student_records().default_for_student(student.id)`. Per the SDK, no `engine.students()` or `engine.student_records()` method exists. The actual admission API is `engine.admission().admit(cmd).await?`. Also `test_tenant()` is referenced but the actual testkit API uses a builder pattern (`TenantContext::for_test(school_id)`).

**Expected:**

`engine.admission().admit(cmd).await?`; `testkit::tenant(school_id)` builder.

**Evidence:**

- `docs/guides/test-strategy.md:67-80` — `engine.students().admit(...)`, `engine.student_records().default_for_student(...)`, `let tenant = test_tenant();`.
  - `crates/tools/sdk/src/engine.rs:123-127` — `pub fn admission(&self) -> AdmissionService<'_>`, no `students()` or `student_records()`.

---

### FINDING 4 (id: `DOC-6-004`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:278-303` (HTTP layer examples)

**Description:**

The HTTP dispatcher example uses `engine.students().with_tenant(&tenant).admit(cmd)` and the route table uses capability strings `"students.admit"`, `"students.read"`, `"attendance.mark"`. Per Wave 5 docs-3 finding DOC-LIB-001/002 and the actual SDK surface, no `engine.students()` method exists and the actual consumer entry point for admission is `engine.admission().admit(cmd).await?`. The capability enum is the macro-generated `Capability` enum (e.g. `Capability::StudentAdmit`, `Capability::StudentsRead`), not bare strings.

**Expected:**

`engine.admission().admit(cmd).await?`; `Capability::StudentAdmit`, `Capability::StudentsRead`, `Capability::AttendanceMark`.

**Evidence:**

- `docs/guides/saas-backend.md:278-291` — `let student = engine.students().with_tenant(&tenant).admit(cmd).await.map_err(...)?;` plus route strings `"students.admit"`, `"students.read"`, `"attendance.mark"`.
  - `crates/tools/sdk/src/engine.rs:123-127` — only `pub fn admission(&self) -> AdmissionService<'_> { AdmissionService::new(self) }`; no `students()`.
  - `crates/cross-cutting/rbac/src/capability.rs` — `Capability` enum is macro-generated; variants are typed (`StudentAdmit`, `StudentsRead`, `AttendanceMark`).

---

### FINDING 40 (id: `DOC-6-040`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/test-strategy.md:174-194` (Storage Adapter Parity Tests)

**Description:**

The guide claims parity tests run "against the PostgreSQL, SQLite, SurrealDB, and MongoDB adapters". Per `AGENTS.md` § Storage Adapters, **SurrealDB and MongoDB adapters are deferred and not shipped from the engine**. Running parity tests against SurrealDB/MongoDB is impossible because those adapters don't exist in the workspace.

**Expected:**

Parity tests run only against PostgreSQL, MySQL, and SQLite (the 3 shipped adapters).

**Evidence:**

- `docs/guides/test-strategy.md:174-194` — "PostgreSQL, SQLite, SurrealDB, and MongoDB adapters".
  - `AGENTS.md` § Storage Adapters — "The SurrealDB and MongoDB adapters are **deferred to a future release** and are **not** shipped from the engine."

---

### FINDING 41 (id: `DOC-6-041`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/test-strategy.md:243-260` (Test Utilities / educore-test crate)

**Description:**

The guide references `educore-test` crate with `test_engine()`, `test_tenant(school_id)`, `test_clock()`, `assert_events_published!`, `assert_audit_record!`. Per `AGENTS.md` Crate Inventory, the actual test crate is **`educore-testkit`** (Phase 16, scaffold only). The name `educore-test` does not match any crate in the inventory.

**Expected:**

`educore-testkit` (per Crate Inventory); correct crate name and API once Phase 16 lands.

**Evidence:**

- `docs/guides/test-strategy.md:243-260` — `The engine ships a educore-test crate`.
  - `AGENTS.md` Crate Inventory — entry 32: `educore-testkit` (Phase 16, "Test infrastructure + SDK"), not `educore-test`.

---

### FINDING 5 (id: `DOC-6-005`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:375-462` (Identity Provider section)

**Description:**

The identity section references `LocalPasswordAuthProvider`, `Oauth2AuthProvider`, `SamlAuthProvider` and calls `engine.auth().validate(&token)` and `engine.rbac().require(&session, Capability::StudentsAdmit)`. Per Wave 5 docs-3 finding DOC-LIB-003 and the actual code, the engine has no `Engine::auth()` method that takes a token (the `auth()` accessor on `Engine` returns the auth provider handle for storage port routing, not a `validate(token)` call) and no `Engine::rbac()` method at all. The port is at `crates/adapters/auth/src/jwt.rs` and the RBAC engine is at `crates/cross-cutting/rbac/src/checker.rs`, both consumed by the consumer directly, not through `engine.auth()`/`engine.rbac()`.

**Expected:**

`let session = auth_provider.validate(&token).await?;` where `auth_provider` is the consumer's `Arc<dyn AuthProvider>`; `rbac_checker.require(&session, Capability::StudentsAdmit).await?;` on the consumer's `Arc<dyn RbacChecker>`.

**Evidence:**

- `docs/guides/saas-backend.md:380-440` — identity options A/B/C and "Capability check at the handler" example `engine.rbac().require(&session, Capability::StudentsAdmit).await?;`.
  - `crates/tools/sdk/src/engine.rs:104-147` — `Engine::auth()` returns `&AuthHandle` (a sub-handle for storage routing), not a token validator; no `rbac()` method exists on `Engine`.

---

### FINDING 6 (id: `DOC-6-006`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:474-518` (Control Plane)

**Description:**

The control-plane example calls `engine.platform().query_schools(...)` and `engine.platform().suspend_school(SuspendSchoolCommand { ... })`. Per the actual `crates/cross-cutting/platform/` crate and `Engine` API, `engine.platform()` does not exist on `Engine`. The platform crate exposes `CreateSchoolCommand`, `SuspendSchoolCommand`, etc. as free functions or via the platform service, not as a `Engine::platform()` accessor. The Engine struct only has `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` accessors.

**Expected:**

Platform admin operates through `engine.platform_admin()` if a wrapper is added, or directly via `educore_platform::commands::suspend_school(cmd, &ctx).await?`.

**Evidence:**

- `docs/guides/saas-backend.md:474-518` — control-plane example using `engine.platform().query_schools(...)` and `engine.platform().suspend_school(...)`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `platform()` method.
  - `crates/cross-cutting/platform/src/commands.rs` — typed commands exist but the consumer calls them directly, not via an `Engine` accessor.

---

### FINDING 7 (id: `DOC-6-007`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:523-619` (Sync Engine)

**Description:**

The sync section references `educore.handle_synced_event(event)` (line 593), an `educore-sync` crate gated by a `sync` feature (line 997), and an in-process "sync coordinator". Per `AGENTS.md` § Crate Inventory, no `educore-sync` crate exists in the 34-crate inventory. There is no `SyncAdapter` port in `docs/ports/`. The actual sync pattern is the consumer's `sync-engine/` worker calling `POST /v1/sync` on the backend (per the same guide's deployment topology), but the backend handler is documented as `for each event: educore.handle_synced_event(event)` — a method that does not exist.

**Expected:**

No `educore-sync` crate; sync is consumer-side; backend handler calls `educore_academic::services::admit_student(cmd, ...)` per-event.

**Evidence:**

- `docs/guides/saas-backend.md:593-600` — backend handler sketch `for each event: educore.handle_synced_event(event)`.
  - `docs/guides/saas-backend.md:997` — Reference Map row "Sync port | `educore-sync` (gated by `sync` feature) | `docs/ports/sync.md`".
  - `AGENTS.md` Crate Inventory — no `educore-sync` crate listed among 34 crates.
  - `find docs/ports/sync.md` — no such file exists in `docs/ports/`.

---

### FINDING 8 (id: `DOC-6-008`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:67-89` (PG RLS section)

**Description:**

The RLS test procedure claims a `pg_rls_blocks_cross_tenant_audit_reads` test exists at `crates/tools/storage-parity/tests/cross_cutting_integration.rs` and references `tools/scripts/pg-rls-test-setup.sql`. Per the actual workspace tree, `crates/tools/storage-parity/tests/` does not contain a `cross_cutting_integration.rs` file (storage-parity is scaffold only per Phase 0 inventory), and no `tools/scripts/` directory exists. The script is a phantom.

**Expected:**

Test path and setup script that actually exist in the workspace.

**Evidence:**

- `docs/guides/saas-backend.md:78-89` — `pg_rls_blocks_cross_tenant_audit_reads` test at `crates/tools/storage-parity/tests/cross_cutting_integration.rs`; `psql -U postgres -d educore -f tools/scripts/pg-rls-test-setup.sql`.
  - `find crates/tools/storage-parity -type f` — only `Cargo.toml`, `src/lib.rs`, README scaffold; no `tests/` directory.
  - `find tools -type f` — no `tools/` directory exists at repo root.

---

### FINDING 9 (id: `DOC-6-009`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Critical
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:355-365` (TenantContext struct)

**Description:**

The guide shows `pub struct TenantContext { school_id, user_id, correlation_id, causation_id }` as the engine's value type. Per `crates/cross-cutting/platform/src/tenant.rs` (and `crates/infra/core/src/ids.rs`), the actual tenant context struct in the engine uses different field names — `school_id`, `actor_id`, `correlation_id`, plus possibly `tenant_id` (consumer-set). The field is `actor_id` or `user_id` depending on the port, but the guide's depiction is a hand-written pseudo-struct that does not match any engine type. The guide also states "consumer-extensible fields can be added by the consumer in their own wrapper, not in the engine" — but the actual `TenantContext` may already be extensible via a typed extension pattern.

**Expected:**

Real `TenantContext` definition from `crates/cross-cutting/platform/src/tenant.rs`.

**Evidence:**

- `docs/guides/saas-backend.md:355-365` — `pub struct TenantContext { pub school_id: SchoolId, pub user_id: UserId, pub correlation_id: CorrelationId, pub causation_id: Option<CorrelationId>, }`.
  - `crates/cross-cutting/platform/src/tenant.rs` — actual struct uses different fields; the field name `actor` vs `user_id` differs.

---

### FINDING 12 (id: `DOC-6-012`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:64-66` (Two Layers of Tenancy table)

**Description:**

The guide's tenancy table says "Engine tenancy" is "Managed by School admin" and "Platform tenancy" is "Managed by System / platform admin". Per `docs/guides/multi-tenancy.md` and the `educore-platform` crate, the engine's `SchoolId` is a **structural foreign key** on every aggregate — the engine itself does not have a "school admin" concept; it only enforces tenant isolation. The guide conflates identity (who acts) with tenancy (which school owns the row). Additionally, the claim "Cross-school commands are forbidden by the engine itself" is partially true (the `SchoolId` match check is enforced) but the doc shows a `Conflict` mapping while the actual error variant is `TenantViolation` per `DomainError` in `crates/infra/core/src/error.rs`.

**Expected:**

`DomainError::TenantViolation` (not `Conflict`) for cross-school command attempts.

**Evidence:**

- `docs/guides/saas-backend.md:64-66` — "Cross-school commands are forbidden by the engine itself — the aggregate's `SchoolId` must equal the `TenantContext::school_id` or the command returns `DomainError::Forbidden`."
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { ... TenantViolation(String) ... }`. Cross-school mismatch returns `TenantViolation`, not `Forbidden`.

---

### FINDING 15 (id: `DOC-6-015`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/multi-tenancy.md:62-78` (Tenant Onboarding/Deletion)

**Description:**

The guide says "A new school is created via `CreateSchoolCommand`. The platform domain emits `SchoolCreated`." and "The engine does not provide a soft-delete or 'archive' tenant command in v1". This contradicts `docs/guides/saas-backend.md:474-518` which says the platform crate ships `CreateSchoolCommand`, `SuspendSchoolCommand`, `UnsuspendSchoolCommand`, `ArchiveSchoolCommand`. The two guides disagree: one says archive/suspend don't exist in v1, the other lists `ArchiveSchoolCommand` as a shipped command. Also `SchoolCreated` event vs reality — the actual event name per the events catalog is `SchoolProvisioned` (or similar).

**Expected:**

Consistent description of platform commands; verified event name.

**Evidence:**

- `docs/guides/multi-tenancy.md:62-78` — "A new school is created via `CreateSchoolCommand`. The platform domain emits `SchoolCreated`" + "no soft-delete or 'archive' tenant command in v1".
  - `docs/guides/saas-backend.md:474-518` — lists `SuspendSchoolCommand`, `UnsuspendSchoolCommand`, `ArchiveSchoolCommand` as shipped.
  - `docs/events/platform.md` (per audit scope) — actual event names from the events catalog.

---

### FINDING 22 (id: `DOC-6-022`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/capability-rbac.md:139-180` (Role struct)

**Description:**

The `Role` struct shows `pub struct Role { pub role_id, name, school_id, capabilities, is_system, created_at, updated_at }`. The capability set is `BTreeSet<Capability>` (correct) but the struct is shown as a value type with direct field access. Per `docs/code-standards.md` and the `educore-rbac` crate, aggregates are not exposed for direct construction; they are constructed via `TryFrom<CreateRoleCommand>` with invariants checked. The struct's `pub` fields violate encapsulation.

**Expected:**

`pub struct Role { role_id: RoleId, name: RoleName, capabilities: CapabilitySet, ... }` with private fields and constructor-based creation.

**Evidence:**

- `docs/guides/capability-rbac.md:139-180` — `pub struct Role { pub role_id, pub name, pub school_id, pub capabilities: BTreeSet<Capability>, pub is_system, pub created_at, pub updated_at }`.
  - `docs/code-standards.md` § Code Standards — "Aggregates are not exposed for direct construction; they are constructed via `TryFrom`."

---

### FINDING 25 (id: `DOC-6-025`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/storage-adapter.md:235-247` (Worked Example)

**Description:**

The example shows `let storage: Arc<dyn StorageAdapter> = Arc::new(PostgresStorage::connect(&env::var("DATABASE_URL")?).await?);` and `storage.migrate().await?;` and `let engine = Engine::builder().storage(storage).build().await?;`. The actual adapter constructor may be `PostgresStorage::connect(url)` returning `Result<Self>`, but the `migrate()` call here is the consumer's responsibility (per `docs/guides/saas-backend.md` § "Migrations: The engine does not own migrations; the consumer does"). The two guides disagree on whether `migrate()` is called by the consumer or by the engine.

**Expected:**

Consumer runs migrations; the storage adapter does not own `migrate()` (per `saas-backend.md`).

**Evidence:**

- `docs/guides/storage-adapter.md:235-247` — `storage.migrate().await?;` immediately after `PostgresStorage::connect`.
  - `docs/guides/saas-backend.md:48-49` — "A migration runner (migrations are owned by the consumer; see `docs/ports/storage.md#migrations`)."

---

### FINDING 31 (id: `DOC-6-031`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/notification-templates.md:64-95` (engine.communication().create_template)

**Description:**

The example uses `engine.communication().create_template(CreateTemplateCommand { ... })`. Per the actual SDK (`crates/tools/sdk/src/engine.rs`), `Engine` has no `communication()` accessor. The communication domain crate is `crates/domains/communication/` and the consumer calls its service functions directly, or via a service accessor the consumer adds on top of the SDK.

**Expected:**

Consumer calls `educore_communication::services::create_template(cmd, &ctx).await?` directly, not via `engine.communication()`.

**Evidence:**

- `docs/guides/notification-templates.md:64-95` — `engine.communication().create_template(CreateTemplateCommand { ... })`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `communication()` method.

---

### FINDING 37 (id: `DOC-6-037`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/school-onboarding.md:99-141` (Worked Example: onboard_school)

**Description:**

The example uses `engine.platform().create_school(...)`, `engine.auth().issue_session(...)`, `engine.settings().update_general_settings(...)`, `engine.academic().create_class(...)`, `engine.fees().create_group(...)`, `engine.finance().open_bank_account(...)`, `engine.hr().register_staff(...)`. Per the SDK, none of these `engine.<x>()` methods exist. The actual consumer wires the platform crate (`educore_platform::commands::create_school(cmd, &ctx)`), settings crate (`educore_settings::*`), and per-domain commands directly.

**Expected:**

Service-typed dispatch via direct crate function calls; no `engine.platform()`/`engine.settings()`/`engine.academic()`/`engine.fees()`/`engine.finance()`/`engine.hr()`.

**Evidence:**

- `docs/guides/school-onboarding.md:99-141` — full onboarding function using 7 non-existent engine accessors.
  - `crates/tools/sdk/src/engine.rs:123-147` — `Engine` exposes only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` (and port handles), no domain CRUD accessors.

---

### FINDING 38 (id: `DOC-6-038`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/ci-cd.md:120-130` (Deployment Dockerfile)

**Description:**

The Dockerfile uses `FROM rust:1.75 AS builder` (the engine's MSRV is 1.75 per `docs/code-standards.md`). However the Dockerfile's `COPY . .` copies the entire monorepo into the build context and runs `cargo build --release`. The actual binary name is not `educore` — it would be the consumer's binary (e.g. `backend`, `sync-engine`). The image's `ENTRYPOINT ["educore"]` references a non-existent binary. The published engine has no `educore` binary; the closest is `educore-cli` (Phase 16 tool, scaffold only).

**Expected:**

Docker build for the consumer's specific binary; no `educore` binary in the engine.

**Evidence:**

- `docs/guides/ci-cd.md:120-130` — `ENTRYPOINT ["educore"]`.
  - `AGENTS.md` Crate Inventory — `educore-cli` is a binary crate (Phase 16, scaffold only); no `educore` binary at the umbrella level.

---

### FINDING 42 (id: `DOC-6-042`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/license-faq.md:106` (third-party crates count)

**Description:**

The guide says "The engine's 27 third-party dependencies (per ADR-015-ExternalCrates.md)". Per the actual workspace `Cargo.toml` (root) and `ADR-015-ExternalCrates.md`, the engine has more or fewer third-party deps depending on feature flags. The "27" number is unverified and likely stale.

**Expected:**

Count from the actual workspace dependency list.

**Evidence:**

- `docs/guides/license-faq.md:106` — "27 third-party dependencies".
  - `docs/decisions/ADR-015-ExternalCrates.md` — should be the source of truth for the count.

---

### FINDING 44 (id: `DOC-6-044`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:902-919` (Deployment Topology)

**Description:**

The topology diagram includes an "Event bus (NATS JetStream)" at the bottom. Per the actual adapter at `crates/adapters/event-bus/src/`, the in-process adapter is `InProcessEventBus`; NATS/Redis/Kafka adapters are not in the workspace (only the in-process adapter is shipped per `AGENTS.md` inventory). The `NatsBus::from_env()` reference in the builder example (line 264) is also a phantom API.

**Expected:**

Only `InProcessEventBus` is shipped; NATS/Redis/Kafka are consumer responsibilities.

**Evidence:**

- `docs/guides/saas-backend.md:264, 902-919` — `NatsBus::from_env()?` and "Event bus (NATS JetStream)" topology.
  - `AGENTS.md` Crate Inventory — only `educore-event-bus` (port, Phase 2) is shipped; no NATS adapter.

---

### FINDING 45 (id: `DOC-6-045`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:744-751` (Metrics query)

**Description:**

The metrics query is raw SQL: `SELECT school_id, date_trunc('day', occurred_at), count(*) FROM outbox WHERE event_type = 'StudentAdmitted' GROUP BY 1, 2;`. Per `docs/code-standards.md` § Engine Rules ("Compile-time safety over strings. Use macro-generated enums") and the typed query layer, the consumer should query through the engine's query port, not raw SQL on the `outbox` table. Raw SQL bypasses the storage port.

**Expected:**

Use the typed query API (`educore_events::query::outbox().where_eq(EventField::Type, EventType::StudentAdmitted).list().await?`).

**Evidence:**

- `docs/guides/saas-backend.md:744-751` — raw SQL on `outbox` table.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names."

---

### FINDING 48 (id: `DOC-6-048`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** `docs/guides/README.md:18` (README contents)

**Description:**

The README's "Available Guides" table is missing `event-replay.md`, `crud-patterns.md`, `idempotent-commands.md`, `test-strategy.md`, and `license-faq.md` (some are listed but the table has only 16 entries for 17 guide files). The README lists 16 guide files; `ls docs/guides/*.md` shows 18 files (17 guides + README.md). The table is internally inconsistent with the directory.

**Expected:**

README lists all 17 guide files in the table.

**Evidence:**

- `docs/guides/README.md:18` — table has 16 rows.
  - `ls docs/guides/*.md` — 18 files: README.md + 17 guides.

---

### FINDING 49 (id: `DOC-6-049`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** High
- **Area:** documentation-guides
- **Location:** Cross-cutting — multiple guides

**Description:**

Multiple guides (`fee-collection.md`, `report-card-generation.md`, `payroll-calculation.md`, `crud-patterns.md`, `school-onboarding.md`, `test-strategy.md`, `ai-agent-integration.md`, `saas-backend.md`, `notification-templates.md`, `offline-sync.md`, `audit-trail.md`) all use `engine.<domain>()` method-style accessors (`engine.fees()`, `engine.assessment()`, `engine.hr()`, `engine.finance()`, `engine.students()`, `engine.communication()`, `engine.platform()`, `engine.settings()`, `engine.academic()`, `engine.tools()`, `engine.student_records()`). Per the SDK at `crates/tools/sdk/src/engine.rs:123-147`, **none of these methods exist on `Engine`**. The actual `Engine` exposes only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` (and port handles like `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`). This is a systemic error across 11 of 17 guides.

**Expected:**

Consumer uses direct crate service functions (`educore_<domain>::services::*`) or builds its own service wrappers on top of the SDK. No `engine.<domain>()` shortcut.

**Evidence:**

- `docs/guides/fee-collection.md` — `engine.fees()` (5 occurrences).
  - `docs/guides/report-card-generation.md` — `engine.assessment()` (7 occurrences).
  - `docs/guides/payroll-calculation.md` — `engine.hr()`, `engine.finance()` (6 occurrences).
  - `docs/guides/crud-patterns.md` — `engine.classes()`, `engine.student_records()`, `engine.students()` (referenced via the `Class.Create` capability and examples).
  - `docs/guides/school-onboarding.md` — `engine.platform()`, `engine.settings()`, `engine.academic()`, `engine.fees()`, `engine.finance()`, `engine.hr()` (10 occurrences).
  - `docs/guides/test-strategy.md` — `engine.students()`, `engine.student_records()` (4 occurrences).
  - `docs/guides/ai-agent-integration.md` — `engine.tools()`.
  - `docs/guides/saas-backend.md` — `engine.students()`, `engine.platform()`, `engine.rbac()`, `engine.finance()`, `engine.handle_synced_event()` (10+ occurrences).
  - `docs/guides/notification-templates.md` — `engine.communication()` (2 occurrences).
  - `docs/guides/offline-sync.md` — `engine.students()` (2 occurrences).
  - `docs/guides/audit-trail.md` — `engine.audit()` (1 occurrence).
  - `crates/tools/sdk/src/engine.rs:123-147` — actual `Engine` method list.

---

### FINDING 43 (id: `DOC-6-043`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Medium
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:786-800` (Billing Integration)

**Description:**

The billing example uses `engine.finance().record_external_payment(RecordExternalPaymentCommand { tenant: platform_tenant_to_school(inv.account_id), ... })`. The helper `platform_tenant_to_school(inv.account_id)` is referenced without definition; the consumer must define it. The example also uses `RecordExternalPaymentCommand` which may or may not exist in `crates/domains/finance/src/commands.rs`. The actual command shape and the helper function are undocumented (consumer-implementation gap).

**Expected:**

Documented command shape (per `docs/commands/finance.md`) and a sketched `platform_tenant_to_school` helper signature.

**Evidence:**

- `docs/guides/saas-backend.md:786-800` — `tenant: platform_tenant_to_school(inv.account_id)` helper used without definition; `RecordExternalPaymentCommand { stripe_invoice_id: inv.id, amount: inv.amount_paid.into(), currency: inv.currency.into(), ... }` with `.into()` conversions on Stripe types.

---

### FINDING 46 (id: `DOC-6-046`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Medium
- **Area:** documentation-guides
- **Location:** `docs/guides/storage-adapter.md:170-178` (RLS SQL)

**Description:**

The RLS policy uses `current_setting('app.current_school_id', true)::uuid`. The `true` second argument to `current_setting` returns `NULL` if the GUC is unset, and the cast to `uuid` will then error at row-evaluation time. The actual adapter per `docs/ports/storage.md` and `docs/schemas/tenancy-schema.md` should set the GUC on every connection before any query runs (so the second argument is unnecessary), and the cast should be guarded.

**Expected:**

The adapter sets `SET LOCAL app.current_school_id = '<uuid>'` on every connection acquired from the pool, then the policy can use `current_setting('app.current_school_id')::uuid` without the `true` fallback.

**Evidence:**

- `docs/guides/storage-adapter.md:170-178` — `USING (school_id = current_setting('app.current_school_id', true)::uuid);`.
  - `docs/ports/storage.md` (per Wave 5 docs-4) — adapter responsibility is to set the GUC per connection.
  - `docs/schemas/tenancy-schema.md` — canonical RLS pattern.

---

### FINDING 47 (id: `DOC-6-047`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Medium
- **Area:** documentation-guides
- **Location:** `docs/guides/saas-backend.md:330-345` (Route groups / capability strings)

**Description:**

The route table uses string-based capabilities: `capability("students.admit")`, `capability("students.read")`, `capability("attendance.mark")`. Per `docs/guides/capability-rbac.md` and the `educore-rbac` crate, capabilities are typed enums (`Capability::StudentAdmit`, `Capability::StudentsRead`, `Capability::AttendanceMark`). String-based capability checks lose compile-time safety and violate `docs/code-standards.md` § Engine Rule 2.

**Expected:**

`capability(Capability::StudentAdmit)` (typed enum value, not string).

**Evidence:**

- `docs/guides/saas-backend.md:330-345` — `.route("/v1/students", post(admit).layer(capability("students.admit")))`.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names."

---

### FINDING 50 (id: `DOC-6-050`)

- **Source:** `docs/audit_reports/findings/wave5-docs-6.md`
- **Severity:** Medium
- **Area:** documentation-guides
- **Location:** Cross-cutting — multiple guides

**Description:**

Multiple guides use `unwrap()`, `expect()`, and `.parse().unwrap_or(20)` patterns in example code: `saas-backend.md:243` (`.parse().unwrap_or(20)`), `saas-backend.md:258` (`.unwrap()`), `crud-patterns.md:124-126` (`NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()`), `fee-collection.md:67,90,97,116,141` (`.unwrap()`), `report-card-generation.md` (multiple), `payroll-calculation.md` (multiple), `school-onboarding.md`, `notification-templates.md`. Per `docs/code-standards.md` § Code Standards, "`unwrap`, `expect`, `panic!` are forbidden in production paths". The guides either need to be marked as sketch/pseudo-code OR use proper error propagation (`?`).

**Expected:**

Use `?` propagation with `map_err` or `TryFrom` in all guide examples; or explicitly mark the code block as "sketch — production code uses `?`".

**Evidence:**

- `docs/guides/saas-backend.md:243, 258, 681` — `.parse().unwrap_or(20)`, `.unwrap()`.
  - `docs/guides/crud-patterns.md:124-126` — `NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()`.
  - `docs/guides/fee-collection.md` (5+ `.unwrap()`).
  - `docs/code-standards.md` § Code Standards — "`unwrap`, `expect`, `panic!` are forbidden in production paths".

---

