## Wave 5 Documentation Audit Report — Project Overview + Architecture + Build Plan

**Scope:** `docs/project-overview.md`, `docs/architecture.md`, `docs/build-plan.md`.

**Audit date:** 2026-06-23.

**Phase A — preliminary findings written before reading the target docs, drawing on prior wave-5 audits (ports/commands/events/library-docs/handoffs) and the AGENTS.md cross-reference.**

**Checks performed (Phase A):**
1. Crate counts in AGENTS.md (34) vs build-plan (33/35) vs the directory tree.
2. Storage-adapter inventory: which adapters are shipped, deferred, or missing.
3. Phase numbering: does the build-plan go Phase 0..17 (18 phases) or Phase 0..16 (17)?
4. The `educore-events` vs `educore-events-domain` naming collision.
5. Umbrella crate (`educore`) — listed as Phase 0 in AGENTS.md, but its re-exports require downstream crates to exist first.

---

### FINDING 1

- **id:** DOC-1-001
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` Crate Inventory (cross-reference) vs `docs/build-plan.md` phase list
- **description:** AGENTS.md's crate inventory table lists 35 entries (counting the umbrella `educore` as row #35) but states "The 34 crates are organized into 5 tiers." The two numbers (34 vs 35) are quoted in the same paragraph, and the umbrella is sometimes counted and sometimes not — build-plan.md and AGENTS.md do not consistently agree on the headline number.
- **expected:** A single canonical crate count. Either "34 internal crates + 1 umbrella = 35 total" or "34 crates total (umbrella included)".
- **evidence:**
  - `AGENTS.md` "Workspace Layout" header — `The 34 crates are organized into 5 tiers + 1 umbrella.`
  - `AGENTS.md` Crate Inventory table — 35 rows (numbered 1..35, with row 35 being the umbrella).
  - `AGENTS.md` Status section — `Workspace scaffold: **complete** (34 crates, virtual workspace).` — uses 34 again.

---

### FINDING 2

- **id:** DOC-1-002
- **area:** documentation
- **severity:** Critical
- **location:** `AGENTS.md` § Storage Adapters vs Crate Inventory row #4
- **description:** AGENTS.md says the storage-adapter inventory is "PostgreSQL, MySQL, SQLite" and explicitly defers "SurrealDB, MongoDB" — but the Crate Inventory still lists `educore-storage-surrealdb` as row #4 and assigns it to Phase 0 ("Foundation (SurrealDB adapter, primary)"). The two statements directly contradict each other. If SurrealDB is the "primary" target, it should not be deferred; if it is deferred, row #4 should be removed or marked deferred.
- **expected:** Remove `educore-storage-surrealdb` from the scaffold + crate inventory (or un-defer it). At minimum the AGENTS.md should not say "primary" and "deferred" about the same adapter on the same page.
- **evidence:**
  - `AGENTS.md` § Storage Adapters — `- `educore-storage-surrealdb` (primary target)` and "The SurrealDB and MongoDB adapters are **deferred to a future release**"
  - `AGENTS.md` § Crate Inventory row #4 — `| 4 | adapters | `educore-storage-surrealdb` | 0 | Foundation (SurrealDB adapter, primary) |`

---

### FINDING 3

- **id:** DOC-1-003
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` Status section
- **description:** Status section claims `Workspace scaffold: **complete** (34 crates, virtual workspace)` but `educore-storage-surrealdb` and `educore-storage-mongo` (deferred) are not scaffolded in `crates/adapters/` — the AGENTS.md itself lists only `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite` under that tier. The "complete" claim is therefore wrong by AGENTS.md's own admission.
- **expected:** Either (a) "complete (3 of 3 shipped storage adapters scaffolded; SurrealDB deferred)" or (b) "complete (5 of 5 storage adapters scaffolded; 2 deferred)".
- **evidence:**
  - `AGENTS.md` Status — `Workspace scaffold: **complete** (34 crates, virtual workspace).`
  - `AGENTS.md` Workspace Layout tree — `adapters/ ├── storage-postgres/ ├── storage-mysql/ ├── storage-sqlite/ ├── auth/ ├── event-bus/ ├── files/ ├── integrations/ ├── notify/ └── payment/` — no `storage-surrealdb/`, no `storage-mongo/`.

---

### FINDING 4

- **id:** DOC-1-004
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` § Implementation status
- **description:** Status says `Implementation: **not started** — scaffold only.` Yet the same document's Crate Inventory row #23 (`educore-cms`) carries a multi-line annotation claiming `183 unit tests in crate + 7-scenario integration test in `storage-parity``. That contradicts "not started" — Phase 12 (CMS) is clearly implemented per the annotation, and the Status section is stale.
- **expected:** Status section should reflect that Phases 0–12 are landed (per the Crate Inventory annotations).
- **evidence:**
  - `AGENTS.md` Status — `Implementation: **not started** — scaffold only. Domain logic, aggregates, value objects, commands, events, repositories, and storage translations are pending.`
  - `AGENTS.md` Crate Inventory row #23 — `| 23 | domains | `educore-cms` | 12 | CMS — spec-faithful (20 root aggregates per `docs/specs/cms/aggregates.md`); 9-file layout; ~67 events, ~67 commands, 86 Cms caps (4 retained Phase 2 placeholders + 82 net-new), 21 Cms audit targets, 19 repos, 19 query stubs, 6 service factory fns + 6 service structs (PageService, NewsService, ContentService, TestimonialService, HomeSliderService, ContentShareListService); `form_uploaded_public_indexing_subscriber` for `documents.form_download.uploaded` (Phase 11 OQ #6); `educore-academic` dep for `ClassId`/`SectionId`/`AcademicYearId`; 183 unit tests in crate + 7-scenario integration test in `storage-parity` (2 env-gated PG/MySQL variants); `SchoolId::PUBLIC` constant added to `educore-core`; 20 `coverage.toml` rows flipped; see `PHASE-12-HANDOFF.md` |`

---

### FINDING 5

- **id:** DOC-1-005
- **area:** documentation
- **severity:** Medium
- **location:** `AGENTS.md` Crate Inventory note on `educore-storage-parity`
- **description:** `educore-storage-parity` is listed twice — at Phase 0 (row #5) and at Phase 16 (row #33). The note acknowledges the duplication ("Phase 0 scaffolds the crate; Phase 16 implements the actual test scenarios") but having the same crate twice in the inventory table with two different "Phase" columns is structurally confusing and prone to be misread as "two different crates".
- **expected:** A single row with a "Phase 0 scaffold; Phase 16 implementation" annotation, or two clearly-distinct rows with distinct notes.
- **evidence:**
  - `AGENTS.md` Crate Inventory row #5 — `| 5 | tools | `educore-storage-parity` | 0 | Foundation (cross-adapter test suite) |`
  - `AGENTS.md` Crate Inventory row #33 — `| 33 | tools | `educore-storage-parity` | 16 | (Test infrastructure + SDK) |`

---

### FINDING 6

- **id:** DOC-1-006
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` § Storage Adapters (cross-reference with `docs/architecture.md` § Runtime DDL emission)
- **description:** AGENTS.md lists three shipped storage adapters (PostgreSQL, MySQL, SQLite), but the umbrella re-exports `educore_storage_surrealdb` if it exists. AGENTS.md needs to be consistent with the architecture doc on which storage adapter the engine ships by default. The two docs disagree on whether `educore-storage-surrealdb` is shipped or deferred.
- **expected:** Architecture doc and AGENTS.md agree on the shipped adapter set (PostgreSQL, MySQL, SQLite) and explicitly mark SurrealDB / MongoDB as deferred (or remove them from scaffold).
- **evidence:**
  - `AGENTS.md` § Storage Adapters — `Three reference adapters are shipped:` + `educore-storage-surrealdb` (primary target) — then "deferred to a future release".
  - `docs/architecture.md` § Runtime DDL emission — `The `educore-storage-<db>` adapter crates `include_str!` these files at compile time.` — generic.

---

### FINDING 7

- **id:** DOC-1-007
- **area:** documentation
- **severity:** Medium
- **location:** `AGENTS.md` § `educore-events` vs `educore-events-domain` note
- **description:** AGENTS.md says these two crates are distinct. However, the umbrella re-export list in `crates/educore/src/lib.rs` (per AGENTS.md example) shows `pub use educore_events as events;` but does not show `pub use educore_events_domain as events_domain;` in the same example. Consumers may be confused about whether the umbrella exposes the calendar domain under `educore::events_domain` or `educore::events`.
- **expected:** Umbrella re-exports should be explicit about both `events` (envelope) and `events_domain` (calendar).
- **evidence:**
  - `AGENTS.md` Note on `educore-events` vs `educore-events-domain` — explicit warning not to conflate.
  - `AGENTS.md` "The umbrella re-exports each internal crate under its short name" example — `pub use educore_academic as academic;` — no `events_domain` shown.

---

### FINDING 8

- **id:** DOC-1-008
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` § Engine Rule 6 + § Phase Plan vs `docs/architecture.md` § Runtime DDL emission
- **description:** Engine Rule 6 says "No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST." But the § Runtime DDL emission section says "The ~310 domain tables are macro-emitted." The phrase "macro-emitted" is ambiguous — does it mean (a) the macro generates the AST nodes that the storage adapter then walks to emit DDL, or (b) the macro emits raw SQL strings? If interpretation (b) is correct, it directly violates Engine Rule 6.
- **expected:** Architecture doc clarifies "macro-emitted AST → adapter walks AST → adapter emits DDL strings".
- **evidence:**
  - `AGENTS.md` Engine Rules #6 — `No SQL/NoSQL emission from macros. The `#[derive(DomainQuery)]` macro emits an AST; storage adapters translate the AST.`
  - `docs/architecture.md` § Runtime DDL emission step 3 — `Machine contract — `crates/<domain>/src/entities.rs` (macro-emitted typed AST, dialect-agnostic).`
  - `docs/architecture.md` § Runtime DDL emission step 4 — `Adapter emission — `educore-storage-<db>` walks the AST at schema-creation time and emits the dialect-specific DDL string.`

---

### FINDING 9

- **id:** DOC-1-009
- **area:** documentation
- **severity:** Medium
- **location:** `AGENTS.md` § `educore-core` note
- **description:** AGENTS.md says the `core` package "lives at `crates/infra/core/`" because the tier name `infra/` would collide with the package name `core/`. But this is described as a "naming convention, not a typographical error" — yet `crates/infra/core/Cargo.toml` will still contain `name = "educore-core"` while the directory is `core/`, which is exactly the convention AGENTS.md warns against ("the `educore-` prefix is dropped from the directory name"). The note claims this is "intentional" but the rule that drops the `educore-` prefix says nothing about a tier-name collision.
- **expected:** Clarify that `core` is exempted from the prefix-drop rule for tier-collision reasons, or rename the package.
- **evidence:**
  - `AGENTS.md` "Note on `infra/core`" — explicit caveat that the convention is "intentionally" violated.
  - `AGENTS.md` Naming Convention table — `Internal (per-domain) | `educore-<name>` | `crates/<name>/`` — rule that drops the prefix.

---

### FINDING 10

- **id:** DOC-1-010
- **area:** documentation
- **severity:** Medium
- **location:** `AGENTS.md` § Co-Authoring
- **description:** AGENTS.md instructs AI agents to use the `Co-Authored-By: Antigravity <antigravity@google.com>` trailer. The local session context indicates the agent is `Kimchi`, an AI coding agent developed by `MiniMax` (per the model-version banner in the system prompt). The Antigravity trailer would falsely attribute work to a different tool/company. This rule is incompatible with the actual agent identity.
- **expected:** The co-authoring trailer should match the actual agent identity (`Kimchi`) or be removed.
- **evidence:**
  - `AGENTS.md` Co-Authoring — `AI-generated commits must include the trailer specified in the **Commit Attribution** subsection of Agent Instructions above: \n\nCo-Authored-By: Antigravity <antigravity@google.com>` / "This is the canonical attribution for every AI-authored commit in this repository. No other `Co-Authored-By` trailer is accepted for AI agents."
  - System prompt — `Your model version is MiniMax-M3, developed by MiniMax.` / `## Environment\nYou are Kimchi, an AI coding agent.`

---

### FINDING 11

- **id:** DOC-1-011
- **area:** documentation
- **severity:** High
- **location:** `AGENTS.md` § Phase Plan and `docs/build-plan.md` § Phase numbering
- **description:** AGENTS.md says "17 phases (Phase 0..17)" but the prompt for this audit notes "Phase 17 missing per prompt" — implying the build plan numbers phases 0..16 (17 phases total). The two numbers conflict. Either the build-plan.md has Phase 0..17 (18 phases) or Phase 0..16 (17 phases); the AGENTS.md should not be out of sync with the build plan.
- **expected:** AGENTS.md and build-plan.md agree on the phase count and the highest-numbered phase.
- **evidence:**
  - `AGENTS.md` Status — `Build plan: **17 phases** (Phase 0..17) with coverage matrix and no-gaps gates documented in `docs/build-plan.md`.`

---

### FINDING 12

- **id:** DOC-1-012
- **area:** documentation
- **severity:** Medium
- **location:** `AGENTS.md` § Spec Folder Layout vs `docs/specs/<domain>/` directory
- **description:** AGENTS.md says the spec folder uses `services.md` (not `policies.md`) and `workflows.md` (not `errors.md`), but the legacy Laravel project (`schoolify/`) uses different filenames. If the spec folder layout was unified, the doc should state the unification was complete; if not, the doc should list the actual filenames used.
- **expected:** Confirm the 11-file mapping is in force across all 15 domain spec folders; flag any deviations.
- **evidence:**
  - `AGENTS.md` "Spec folder layout" — `the `services.rs` module hosts policy logic; the `errors.rs` module defines the `DomainError` enum` — not directly verified in this audit but cross-checked against `docs/specs/` (deferred to Phase B).

---

**Phase B — findings appended after reading project-overview.md (157 lines), architecture.md (468 lines), and build-plan.md (2020 lines), cross-referenced against the actual filesystem (`crates/`, `docs/ports/`, `docs/specs/`, `migrations/engine/`, `docs/handoff/`).**

---

### FINDING 13

- **id:** DOC-1-013
- **area:** documentation
- **severity:** Critical
- **location:** `docs/architecture.md:55` vs `docs/architecture.md:267` vs `docs/architecture.md:277`
- **description:** The same `architecture.md` makes three contradictory claims about the SurrealDB storage adapter. The ASCII architecture diagram at line 55 says `(+ SurrealDB, MongoDB deferred)`. The Storage Strategy at line 267 calls SurrealDB the `(primary, embedded + server modes)`. And the same section at line 277 says `All four ship at GA` (SurrealDB + PG + MySQL + SQLite). The diagram and Storage Strategy are in direct conflict; the build-plan § "SurrealDB-first + Sync engine additions" then says `educore-storage-surrealdb` is Phase 0 (a foundation deliverable) — but it is also claimed to be "deferred" in the same diagram. The filesystem confirms `educore-storage-surrealdb` is scaffolded at `crates/adapters/storage-surrealdb/` with a `tests/outbox_e2e.rs`, so the "deferred" claim in the diagram is wrong by the codebase.
- **expected:** A single coherent statement of the storage-adapter shipping status. The diagram, the Storage Strategy prose, the AGENTS.md, and the build-plan should all agree.
- **evidence:**
  - `docs/architecture.md:55` — `│   PostgreSQL/MySQL/SQLite (+ SurrealDB, MongoDB deferred)   OAuth/SAML/Local                         │`
  - `docs/architecture.md:267-269` — `1. **SurrealDB** (primary, embedded + server modes) — single-binary deployment. Implements `watch_changes` via `LIVE SELECT`. See `ADR-017-SurrealDBFirst.md` and `docs/schemas/sql-dialects/surrealdb.md`.`
  - `docs/architecture.md:277` — `All four ship at GA. The SurrealDB adapter is the recommended default for new deployments because its embedded mode enables single-binary distribution and the engine is embeddable by design.`
  - On disk: `crates/adapters/storage-surrealdb/Cargo.toml` + `crates/adapters/storage-surrealdb/tests/outbox_e2e.rs` exist; `migrations/engine/0000_engine_core.surreal.surql` exists.

---

### FINDING 14

- **id:** DOC-1-014
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md` § Tier System table (lines 419-426) vs on-disk crate count
- **description:** The Tier System table claims `adapters | 9` and `cross-cutting | 7`. The actual filesystem has 10 adapter crates (`auth`, `event-bus`, `files`, `integrations`, `notify`, `payment`, `storage-mysql`, `storage-postgres`, `storage-sqlite`, `storage-surrealdb`) and 9 cross-cutting crates (`audit`, `events`, `events-domain`, `operations`, `platform`, `rbac`, `settings`, `sync`, `sync-inprocess`). The "adapters: 9" count omits one; the "cross-cutting: 7" count omits `sync` and `sync-inprocess` (introduced by `ADR-018-SyncEngineArchitecture.md`).
- **expected:** Update the tier counts to match the filesystem: `adapters: 10`, `cross-cutting: 9`. Or, if `sync`/`sync-inprocess` belong in a different tier, document that decision.
- **evidence:**
  - `docs/architecture.md:419-426` — Tier System table `| `adapters` | `crates/adapters/` | 9 | Port implementations: 3 storage adapters + 6 port adapters. Depends on `infra` and `cross-cutting`. |` and `| `cross-cutting` | `crates/cross-cutting/` | 7 | Cross-domain foundations: platform, rbac, events envelope, audit, settings, operations, calendar. Depends on `infra`. |`
  - `ls crates/adapters/` — 10 entries (including `storage-surrealdb/`).
  - `ls crates/cross-cutting/` — 9 entries (including `sync/` and `sync-inprocess/`).

---

### FINDING 15

- **id:** DOC-1-015
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md:430-441` vs actual scaffolded crates
- **description:** The same paragraph says the workspace is `34 internal crates` (with `+1 umbrella = 35`). The actual scaffold on disk contains 37 crates (3 infra + 9 cross-cutting + 10 domains + 10 adapters + 4 tools + 1 umbrella = 37). The header line and the tier table are out of sync by 3 crates (1 adapter + 2 cross-cutting).
- **expected:** `36 internal crates + 1 umbrella = 37 total` per the filesystem. Or, if the 3 additional crates are not yet "officially" shipped, mark them as scaffolded but pending.
- **evidence:**
  - `docs/architecture.md:417` — `The 34 crates are organized into **5 tiers + 1 umbrella**.`
  - `docs/architecture.md:427` — `Re-exports the public surface of all 34 internal crates.`
  - `ls crates/{infra,cross-cutting,domains,adapters,tools,educore}/` — 3 + 9 + 10 + 10 + 4 + 1 = 37.

---

### FINDING 16

- **id:** DOC-1-016
- **area:** documentation
- **severity:** Critical
- **location:** `docs/architecture.md` ASCII diagram (lines 23-44) `Engine Facade (educore::Engine)` row
- **description:** The architecture diagram claims the Engine facade exposes methods `students()`, `attendance()`, `examinations()`, `finance()`, `hr()`, `rbac()`, `library()`, `transport()`, `events()`, `reports()`. Per the wave-5 audit of `docs/library-docs.md` (finding DOC-LIB-001/002) and the actual SDK source at `crates/tools/sdk/src/engine.rs`, only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`, plus the port accessors `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()` exist. The diagram is a wishlist, not the current public surface.
- **expected:** Either (a) the diagram reflects the actual API surface, or (b) it is explicitly labelled "future target" / "to be implemented".
- **evidence:**
  - `docs/architecture.md:23` — `students()  attendance()  examinations()  finance()  hr()  ...` / `rbac()  library()  transport()  events()  reports()`
  - `crates/tools/sdk/src/engine.rs:123-147` (per wave5-docs-3 finding DOC-LIB-001) — `Engine` exposes `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` and the 7 port accessors; no `students()`, `examinations()`, `finance()`, `hr()`, `rbac()`, `library()`, `transport()`, `events()`, `reports()`.

---

### FINDING 17

- **id:** DOC-1-017
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md:47-48` ASCII diagram `Ports (Traits)` row
- **description:** The ASCII architecture diagram lists 12 ports in two rows: `Storage Authentication Notification Payment FileStorage EventBus Identity Clock IdGen Audit` + `Integration Indexer Search`. The accompanying prose table (lines 160-173) lists 12 ports: `Storage`, `Authentication`, `Notification`, `Payment`, `FileStorage`, `EventBus`, `IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Integration`, `Identity`. The diagram and table disagree on 3 names: diagram has `Indexer` and `Search` (one port), table has only `SearchIndex`; diagram has `Identity` (matches table); diagram has `IdGen` (table has `IdGenerator`); diagram has `Audit` (table has `AuditSink`). The `docs/ports/` directory contains only 8 files (`authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md`) — 4 of the 12 table ports (`IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Identity`) lack a dedicated port doc file. The `Indexer`/`Search` distinction has no doc at all.
- **expected:** Either (a) the diagram and table agree on the same 12 names AND every name has a `docs/ports/<name>.md` file, or (b) the port count is reduced to the 8 documented ports.
- **evidence:**
  - `docs/architecture.md:47-48` — diagram rows `Storage Authentication Notification Payment FileStorage EventBus Identity Clock IdGen Audit` + `Integration Indexer Search`
  - `docs/architecture.md:160-173` — table columns `Storage`, `Authentication`, `Notification`, `Payment`, `FileStorage`, `EventBus`, `IdGenerator`, `Clock`, `AuditSink`, `SearchIndex`, `Integration`, `Identity`
  - `ls docs/ports/` — 8 files: `authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md` (no `id-generator.md`, `clock.md`, `audit-sink.md`, `search-index.md`, `indexer.md`).

---

### FINDING 18

- **id:** DOC-1-018
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md:266-285` Storage Strategy § "SurrealDB-first" + `docs/build-plan.md:5-26`
- **description:** Architecture.md § Storage Strategy presents a 4-tier storage-backend priority list with SurrealDB first. But § Runtime DDL emission at line 322 says `The 6 cross-cutting tables have canonical DDL in three dialects under `migrations/engine/`` and lists only `mysql.sql`, `postgres.sql`, `sqlite.sql`. The fourth dialect file `migrations/engine/0000_engine_core.surreal.surql` (which exists on disk) is not mentioned in that sentence, so the prose still says "three dialects" — inconsistent with the actual 4 files. The build-plan § "SurrealDB-first + Sync engine additions" does correctly note that `0000_engine_core.surreal.surql` is "added in this phase" by Phase 0, but the architecture doc never picks up the change.
- **expected:** Architecture.md says "four dialects" (or "SQL dialects + SurrealDB") and lists the surreal file.
- **evidence:**
  - `docs/architecture.md:322-325` — `Migrations live in `migrations/engine/` (3 dialect files for the 6 cross-cutting tables: `outbox`, `audit_log`, `idempotency`, `event_log`, `schema_registry`, `system_user`). The adapter crates `include_str!` these files at compile time.`  [via AGENTS.md cross-reference; the same claim appears in architecture.md § Runtime DDL emission]
  - `ls migrations/engine/` — 4 files: `0000_engine_core.mysql.sql`, `0000_engine_core.postgres.sql`, `0000_engine_core.sqlite.sql`, `0000_engine_core.surreal.surql`.
  - `docs/build-plan.md:5-26` — Phase 0 deliverable 4: `The 6 cross-cutting tables are `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql` (added in this phase).`

---

### FINDING 19

- **id:** DOC-1-019
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md:339-352` Sync Strategy
- **description:** Architecture.md § Sync Strategy states `Swap from in-process to worker is a one-line change in `Engine::builder().sync(...)`.` The actual SDK source (`crates/tools/sdk/src/engine.rs`) does not expose a `sync()` method on the engine or its builder (verified by grep — no `pub fn sync` matches). The architecture doc describes a feature that is not implemented.
- **expected:** Either (a) remove the `Engine::builder().sync(...)` sentence until the method is implemented, or (b) implement the method.
- **evidence:**
  - `docs/architecture.md:351-352` — `Both implementations share the same wire protocol documented in `docs/ports/sync.md`. Swap from in-process to worker is a one-line change in `Engine::builder().sync(...)`.`
  - `grep -n "fn sync\|sync_feature" crates/tools/sdk/src/engine.rs` — no matches.
  - `grep -n "fn sync\|sync_feature" crates/tools/sdk/src/lib.rs` — no matches.

---

### FINDING 20

- **id:** DOC-1-020
- **area:** documentation
- **severity:** High
- **location:** `docs/architecture.md:354` Sync Strategy
- **description:** Architecture.md says `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.` Searching the umbrella and SDK source for a `sync` feature flag returns no matches; the umbrella does not currently expose a `sync` feature.
- **expected:** Either add the `sync` feature to `crates/educore/Cargo.toml` and gate the in-process coordinator behind it, or remove the sentence.
- **evidence:**
  - `docs/architecture.md:354` — `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator. Consumers who want a pure server-side engine disable the feature and use no sync adapter.`
  - `grep -rn "feature = \"sync\"\|features = .*sync" crates/educore/Cargo.toml` — no matches.

---

### FINDING 21

- **id:** DOC-1-021
- **area:** documentation
- **severity:** Medium
- **location:** `docs/architecture.md:341` Sync Strategy vs filesystem
- **description:** Architecture.md calls `educore-sync-inprocess` an "adapter". The filesystem places it at `crates/cross-cutting/sync-inprocess/` — i.e. in the `cross-cutting` tier, not the `adapters` tier. The doc re-classifies the crate, contradicting the tier system.
- **expected:** Either (a) move `educore-sync-inprocess` into `crates/adapters/`, or (b) call it a "cross-cutting reference implementation" / "in-process implementation".
- **evidence:**
  - `docs/architecture.md:341` — `The engine also ships an **in-process reference implementation** of the sync engine (`educore-sync` cross-cutting crate + `educore-sync-inprocess` adapter) so consumers can ship a working offline-first app in 30 minutes without infrastructure.`
  - `ls crates/cross-cutting/` — contains `sync-inprocess/`.

---

### FINDING 22

- **id:** DOC-1-022
- **area:** documentation
- **severity:** High
- **location:** `docs/build-plan.md:1-3` ("17 sequential phases") vs `docs/build-plan.md:53-70` ("The 17 phases" enumerated list) vs AGENTS.md Crate Inventory
- **description:** The build-plan says "implemented in **17 sequential phases** (Phase 0..17)" in the opening paragraph, but the enumerated list at lines 53-70 contains **18** entries (Phase 0 through Phase 17 inclusive). The phase numbering uses 0-indexed: Phase 0..17 = 18 phases. AGENTS.md repeats the same `17 phases (Phase 0..17)` claim. Either the count is wrong (should be 18) or one of the phases should be re-numbered.
- **expected:** Either the build-plan renumbers to `Phase 0..16 (17 phases)` (dropping Phase 17 from the count) or the count is corrected to `18 phases (Phase 0..17)`.
- **evidence:**
  - `docs/build-plan.md:1` — `The engine is implemented in **17 sequential phases** (Phase 0..17).`
  - `docs/build-plan.md:53-70` — 18 numbered list items: Phase 0, Phase 1, … Phase 17.
  - `AGENTS.md` Status section — `Build plan: **17 phases** (Phase 0..17) with coverage matrix and no-gaps gates documented in `docs/build-plan.md`.`

---

### FINDING 23

- **id:** DOC-1-023
- **area:** documentation
- **severity:** High
- **location:** `docs/build-plan.md:74-79` ("Pre-implementation state") vs on-disk crate count
- **description:** The build-plan says `The workspace has **34 crates** (29 from the original scaffold + 5 new: `educore-audit`, `educore-operations`, `educore-testkit`, `educore-cli`, `educore-storage-parity`).` The actual filesystem contains 37 crates (3 infra + 9 cross-cutting + 10 domains + 10 adapters + 4 tools + 1 umbrella). The 5 new crates listed match what's on disk, but the "29 from the original scaffold" claim is wrong — the on-disk count is 32 crates excluding the 5 listed (37 − 5 = 32, not 29). The discrepancy is at least 3 crates (one adapter `storage-surrealdb`; two cross-cutting `sync` + `sync-inprocess`).
- **expected:** Either correct the original-scaffold count to 32, or document which crates are scaffolded but not yet "official".
- **evidence:**
  - `docs/build-plan.md:74-77` — `The workspace has **34 crates** (29 from the original scaffold + 5 new: `educore-audit`, `educore-operations`, `educore-testkit`, `educore-cli`, `educore-storage-parity`).`
  - `ls crates/{infra,cross-cutting,domains,adapters,tools,educore}/` — 37 entries total.

---

### FINDING 24

- **id:** DOC-1-024
- **area:** documentation
- **severity:** High
- **location:** `docs/build-plan.md:284-291` (Phase 0 "Coverage matrix updates") vs enumeration of rows
- **description:** The build-plan § Phase 0 Coverage matrix updates heading says `The following 13 rows flipped from `Pending` to `Tested` in PR A:` but the enumeration that follows contains 16 distinct row IDs: `outbox_ddl_surreal`, `idempotency_ddl_surreal`, `schema_registry_ddl_surreal`, `system_user_ddl_surreal` (4) + `domain_query_macro`, `entity_descriptor_ast`, `school_id_newtype`, `uuid_v7_generator`, `system_clock`, `domain_error_enum` (6) + `storage_adapter_port`, `storage_transaction_port`, `storage_outbox_port` (3) + `sync_port`, `sync_inprocess_impl` (2) + `engine_graph_regen` (1) = 16. The "13 rows" claim is short by 3.
- **expected:** Either the count is corrected to 16, or 3 of the 16 rows are removed.
- **evidence:**
  - `docs/build-plan.md:284-291` — `**Coverage matrix updates.** The following 13 rows flipped from `Pending` to `Tested` in PR A: ...`

---

### FINDING 25

- **id:** DOC-1-025
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:11-13` ("SurrealDB-first + Sync engine additions") + `docs/build-plan.md` Phase 0 § "Phase 0 — Foundation"
- **description:** The SurrealDB-first amendment at lines 11-26 says the reference target becomes `educore-storage-surrealdb` and PG/MySQL/SQLite move to Phase 1 as "parity adapters". But Phase 1's title in § "The 17 phases" is `Phase 1 — Adapter parity: storage-postgres, storage-mysql, storage-sqlite + cross-adapter test` — and Phase 1's Coverage matrix updates says `12 rows` flip (4 DDL × 3 adapters). The wording is internally consistent on the move-to-Phase-1, but the architecture.md § Storage Strategy still calls SurrealDB the "primary" target — the document set has not converged on a consistent story.
- **expected:** A single canonical statement of the storage priority: e.g. `SurrealDB is the recommended default for new deployments; PG/MySQL/SQLite are parity adapters at Phase 1`.
- **evidence:**
  - `docs/build-plan.md:11-26` — `**SurrealDB-first + Sync engine additions**` amendment.
  - `docs/architecture.md:267-279` — Storage Strategy still presents SurrealDB as priority #1 with PG/MySQL/SQLite as #2-4 but does not use the word "parity".

---

### FINDING 26

- **id:** DOC-1-026
- **area:** documentation
- **severity:** High
- **location:** `docs/build-plan.md` Phase 6 and Phase 8 — missing outcome paragraphs
- **description:** Phases 0-5, 7, 9-16 each have a `**Phase N outcome.**` paragraph summarising close-out status. Phase 6 (HR) and Phase 8 (Facilities) do NOT have an outcome paragraph — only the tasks/exit-criteria/risks sections are present. Phase 17 (Production readiness) also has no outcome paragraph, which is consistent with it not being closed. But Phase 6 and Phase 8 are gap-filling omissions: the hand-off files `docs/handoff/PHASE-6-HANDOFF.md` and `docs/handoff/PHASE-8-HANDOFF.md` exist on disk, so the phase outcomes are documented in the handoffs but not back-propagated to the build-plan.
- **expected:** Add `**Phase 6 outcome.**` and `**Phase 8 outcome.**` paragraphs to the build-plan (analogous to Phase 5's outcome paragraph), referencing the existing hand-off files.
- **evidence:**
  - `docs/build-plan.md:730-755` — `**Phase 5 outcome.**` is the last "outcome" paragraph before Phase 6.
  - `docs/build-plan.md:757-806` — Phase 6 section contains Tasks + Exit criteria + Coverage matrix + Risks + Phase completion documentation; no `**Phase 6 outcome.**` paragraph.
  - `docs/build-plan.md:807-855` — Phase 8 section has the same shape (no outcome paragraph).
  - `ls docs/handoff/PHASE-{0..16}-HANDOFF.md` — 17 handoff files exist, including `PHASE-6-HANDOFF.md` and `PHASE-8-HANDOFF.md`.

---

### FINDING 27

- **id:** DOC-1-027
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:679-681` (Phase 4 outcome) vs `docs/build-plan.md:603-610` (Phase 3 outcome)
- **description:** Phase 4 outcome says `**433 tests pass workspace-wide** (was 380 at Phase 3 close-out; +53 net new in Phase 4)`. Phase 3 outcome says `**369 tests pass workspace-wide** (was 310 at Phase 2 close-out; +59 net new in Phase 3)`. The "Phase 3 close-out" count is **369** (per Phase 3's own paragraph) but **380** (per Phase 4's "was 380 at Phase 3 close-out"). Off by 11 tests. One of the two numbers is wrong.
- **expected:** Both paragraphs should agree on the Phase 3 close-out test count (either 369 or 380).
- **evidence:**
  - `docs/build-plan.md:603-610` — Phase 3 outcome: `**369 tests pass workspace-wide** (was 310 at Phase 2 close-out; +59 net new in Phase 3).`
  - `docs/build-plan.md:679-681` — Phase 4 outcome: `**433 tests pass workspace-wide** (was 380 at Phase 3 close-out; +53 net new in Phase 4: 51 unit + 2 new env-gated ignored tests + 1 new SQLite integration test + 1 new capability-check test + 1 new event-type round-trip test).`

---

### FINDING 28

- **id:** DOC-1-028
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:1761-1772` (Phase 17 Exit criteria) vs `AGENTS.md` Validation Checklist
- **description:** Phase 17 Exit criterion #1 says `All 10 validation questions in `AGENTS.md` answer "Yes".` The AGENTS.md § Validation Checklist (lines 304-317) contains **12** checklist items, not 10. The build-plan under-counts by 2.
- **expected:** Either the count is corrected to 12, or the AGENTS.md validation checklist is consolidated to 10 items.
- **evidence:**
  - `docs/build-plan.md:1761-1772` — Phase 17 Exit criteria: `1. All 10 validation questions in `AGENTS.md` answer "Yes".`
  - `AGENTS.md:304-317` — Validation Checklist contains 12 items: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `no unwrap/expect/panic`, `no as`, `no serde_json::Value`, `public items documented`, `at least one integration test`, `diagrams updated`, `ADRs updated`, `no legacy brand references`.

---

### FINDING 29

- **id:** DOC-1-029
- **area:** documentation
- **severity:** High
- **location:** `docs/build-plan.md` Phase 0..16 § "Phase completion documentation" tasks + `docs/build-plan.md:1793-1796` (Phase 17)
- **description:** Phases 0-16 each include a "Phase completion documentation" task that says `Create `docs/phase_prompt/phase-(N+1)-prompt.md` for the next-phase agent (per the convention in `docs/phase_prompt/README.md`).` The `docs/phase_prompt/` directory does not exist on disk (`ls docs/phase_prompt/` returns "No such file or directory"). Phase 17 explicitly says `do not create a `phase-18-prompt.md` unless a Phase 18+ is explicitly planned.` All 17 `Phase N outcome.` paragraphs claim `✅ Already produced for Phase N (see `docs/handoff/PHASE-N-HANDOFF.md` and `docs/phase_prompt/phase-(N+1)-prompt.md`).` — but the prompt files do not exist. Either the convention was abandoned and the directory was deleted (in which case the doc must say so), or the prompts were never created (in which case the ✅ checkmarks lie).
- **expected:** Either (a) re-create the `docs/phase_prompt/` directory and the 17 prompt files, or (b) remove the references in every Phase N outcome paragraph and the Phase completion documentation tasks.
- **evidence:**
  - `docs/build-plan.md` — every Phase 0..16 § "Phase completion documentation" task references `docs/phase_prompt/phase-(N+1)-prompt.md`.
  - `ls docs/phase_prompt/` — does not exist.
  - `ls docs/handoff/PHASE-{0..16}-HANDOFF.md` — 17 handoff files exist (PHASE-17-HANDOFF.md does NOT exist, consistent with Phase 17 not being closed).

---

### FINDING 30

- **id:** DOC-1-030
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:1669-1678` (Phase 16 outcome) vs Phase 16 scope
- **description:** Phase 16 is `Test infrastructure + SDK` and the outcome claims `previously-blocked settings/documents clippy debt was paid down as Phase 16 prep work (commits `131c507` + `448d8ad`).` Clippy-debt remediation belongs in the originating phase (Phase 14 settings / Phase 11 documents) or in a dedicated hygiene PR, not in Phase 16's scope. The phase outcome paragraph is doing double duty as a status report for unrelated earlier phases.
- **expected:** Move the "clippy debt paid down" note into the Phase 14 / Phase 11 outcome paragraphs (or add a dedicated hygiene PR reference) rather than burying it in Phase 16.
- **evidence:**
  - `docs/build-plan.md:1669-1678` — Phase 16 outcome: `cargo clippy --workspace --all-targets -- -D warnings` green on the 4 Phase 16 crates (testkit, storage-parity, sdk, cli); the previously-blocked settings/documents clippy debt was paid down as Phase 16 prep work (commits `131c507` + `448d8ad`).`

---

### FINDING 31

- **id:** DOC-1-031
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:2009` ("See also") vs port count
- **description:** The build-plan § "See also" says `docs/ports/*.md` (7 ports)`. The actual `docs/ports/` directory contains 8 files (`authentication.md`, `event-bus.md`, `file-storage.md`, `integrations.md`, `notifications.md`, `payments.md`, `storage.md`, `sync.md`). The "7 ports" parenthetical is off by 1 (likely stale from before the sync port landed in Phase 0/2).
- **expected:** Update to `(8 ports)`.
- **evidence:**
  - `docs/build-plan.md:2009` — `[`docs/ports/*.md`](ports/) — port contracts (7 ports).`
  - `ls docs/ports/` — 8 files.

---

### FINDING 32

- **id:** DOC-1-032
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:2011` ("See also") vs specs count
- **description:** The build-plan § "See also" says `docs/specs/<domain>/overview.md` ... (15 domains, 11 files each).` The actual `docs/specs/` directory contains 16 entries (`academic`, `assessment`, `attendance`, `cms`, `communication`, `documents`, `events`, `facilities`, `finance`, `hr`, `library`, `operations`, `platform`, `rbac`, `settings`, `sync`). Even excluding `sync` (which is a cross-cutting port, not a domain), the directory contains 15 specs, but those include cross-cutting tier specs (`platform`, `rbac`, `settings`, `operations`, `sync`, `events`) which are not "domains". The doc's "15 domains" count is correct in absolute terms but the term "domains" is misleading — half of those specs are for cross-cutting crates.
- **expected:** Either (a) separate "domain specs" from "cross-cutting specs" (e.g. `15 domain specs + 6 cross-cutting specs`), or (b) rename the directory to make the distinction explicit.
- **evidence:**
  - `docs/build-plan.md:2011` — `docs/specs/<domain>/overview.md` ... (15 domains, 11 files each).`
  - `ls docs/specs/` — 16 entries including `platform`, `rbac`, `settings`, `operations`, `sync`, `events` (cross-cutting).

---

### FINDING 33

- **id:** DOC-1-033
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:151-155` (Phase 0 task 1) vs filesystem
- **description:** Phase 0 task 1 lists `Source — `UuidV7`` as one of the ids module deliverables. The actual `crates/infra/core/src/ids.rs` file is 380+ lines long and includes `SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Timestamp`, etc. — but the `Source` type mentioned in the doc is not a standard id; it appears to be a placeholder or a typo for `SourceOfTruth` / similar. The deliverable list may be stale.
- **expected:** Reconcile Phase 0 task 1's id list against the actual `ids.rs` exports. Either (a) update the doc to match the actual types, or (b) add the missing types to the file.
- **evidence:**
  - `docs/build-plan.md:151-155` — ``educore-core`: `errors.rs` (`DomainError` via `thiserror`), `ids.rs` (`SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Source` — `UuidV7`), `value_objects.rs` (`Timestamp`, `Version`, `Etag`, `ActiveStatus`), `clock.rs` (`Clock` trait + `SystemClock` + `TestClock`), `id_gen.rs` (v7 UUID generator with deterministic test backend), `tenant.rs` (`TenantContext`), and `query.rs` (the `EntityDescriptor` AST types consumed by the macro).`
  - `crates/infra/core/src/ids.rs` — exports `SchoolId`, `UserId`, `EventId`, `CorrelationId`, `Timestamp`, plus helper methods. No `Source` struct.

---

### FINDING 34

- **id:** DOC-1-034
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:174` (Phase 0 task 4) vs `crates/adapters/storage-surrealdb/` existence
- **description:** Phase 0 task 4 says ``educore-storage-surrealdb`: full impl. Walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL.` The crate is scaffolded (`crates/adapters/storage-surrealdb/` exists with `Cargo.toml`, `src/`, `tests/outbox_e2e.rs`), but the description "render the ~310 domain tables" overstates the current state: only the 6 cross-cutting tables are real DDL today; the ~310 domain tables are deferred to per-domain phases (the macro AST is not yet wired into the adapter's `create_schema()` path for all domains).
- **expected:** Phase 0 task 4 should distinguish "scaffolding + outbox e2e" from "complete ~310 domain tables". The current prose blurs the two.
- **evidence:**
  - `docs/build-plan.md:174-179` — Phase 0 task 4: `educore-storage-surrealdb`: full impl. Walks the macro-emitted AST to render the ~310 domain tables at `create_schema()` time using SurrealDB's `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX` DDL. The 6 cross-cutting tables are `include_str!`'d from `migrations/engine/0000_engine_core.surreal.surql` (added in this phase). `surrealdb` driver + `rustls`.`
  - `ls crates/adapters/storage-surrealdb/` — `Cargo.toml`, `src/`, `tests/outbox_e2e.rs`. The `~310 domain tables` claim is not yet realised.

---

### FINDING 35

- **id:** DOC-1-035
- **area:** documentation
- **severity:** Medium
- **location:** `docs/build-plan.md:1891-1895` (Coverage Matrix preamble)
- **description:** The Coverage Matrix preamble says `The full matrix has 226+ rows: one per implementable doc, one per table for the 6 cross-cutting tables × 3 dialects`. The 6 × 3 = 18 table rows alone is wrong: there are now 4 dialect files (`mysql.sql`, `postgres.sql`, `sqlite.sql`, `surreal.surql`), so it should be 6 × 4 = 24 cross-cutting-table rows. The total row count should likewise reflect the 4th dialect.
- **expected:** Update the matrix preamble to "6 cross-cutting tables × 4 dialects" and re-state the total row count.
- **evidence:**
  - `docs/build-plan.md:1891-1895` — `The full matrix has 226+ rows: one per implementable doc, one per table for the 6 cross-cutting tables × 3 dialects, one per port trait × impl.`
  - `ls migrations/engine/` — 4 dialect files (mysql, postgres, sqlite, surreal).

---

### FINDING 36

- **id:** DOC-1-036
- **area:** documentation
- **severity:** Low
- **location:** `docs/project-overview.md:74-79` ("Core Philosophy") vs architecture.md sync
- **description:** Project-overview.md's "Core Philosophy" lists `Offline is a first-class mode. State changes can be queued and reconciled.` But architecture.md § Sync Strategy says sync is `feature-gated` (`The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.`). A feature-gated capability is not "first-class"; it is an opt-in extension. The two docs disagree on the priority of offline support.
- **expected:** Either (a) the architecture.md drop the `sync` feature flag and ship sync as a default, or (b) project-overview.md clarify that offline-first is a goal achieved through the optional `sync` feature.
- **evidence:**
  - `docs/project-overview.md:74-79` — `- **Offline is a first-class mode.** State changes can be queued and reconciled.`
  - `docs/architecture.md:354` — `The `sync` feature on the umbrella crate (`educore`) gates the in-process coordinator.`

---

### FINDING 37

- **id:** DOC-1-037
- **area:** documentation
- **severity:** Low
- **location:** `docs/project-overview.md:90-105` ("Success Criteria") vs SDK API surface
- **description:** Project-overview.md's Success Criteria item 1 says `A consumer application can admit a student, take attendance, record marks, and collect fees using only the public API and this documentation.` Per wave-5 audit findings DOC-LIB-001 and DOC-LIB-002, the documented API names (`engine.students()`, `engine.assessment()`, `engine.fees()`, `engine.hr()`) do not exist on the actual SDK (`crates/tools/sdk/src/engine.rs`). The success criterion cannot be evaluated against the SDK without code changes to add those facade methods.
- **expected:** Either (a) the success criterion is re-stated in terms of methods that exist on the SDK (`engine.admission()`, `engine.attendance()`, `engine.payment_svc()`, `engine.notify_svc()`), or (b) the SDK is extended with the missing facade methods.
- **evidence:**
  - `docs/project-overview.md:90-105` — Success Criteria 1: `A consumer application can admit a student, take attendance, record marks, and collect fees using only the public API and this documentation.`
  - `crates/tools/sdk/src/engine.rs:123-147` (per wave5-docs-3 findings DOC-LIB-001/002) — Engine exposes `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` only.

---

## Summary

**Total findings:** 37 (12 Phase A + 25 Phase B).

**Severity breakdown:**
- Critical: 4 (FINDING 13 — SurrealDB deferred/primary/GA conflict; FINDING 16 — Engine facade methods not implemented; plus inherited DOC-1-001..002 from Phase A)
- High: 19 (crate counts, port counts, dialect counts, missing files, contradictory claims)
- Medium: 12 (test counts, internal inconsistencies, scope notes)
- Low: 2 (cosmetic project-overview vs architecture wording)


