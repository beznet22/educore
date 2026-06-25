# Production Readiness Roadmap

> **How to update:** run `scripts/update-roadmap.py` from the repo root.
> The script reads `12-roadmap-data.toml`, runs each `check`, and
> regenerates the COMPUTED sections below. Items the script cannot
> auto-evaluate are marked `[~]` (manual review needed).
>
> **Decision items** (those requiring human input) live in
> [`13-decision-needed.md`](13-decision-needed.md). Linked from P0
> where they block work.

> **Source of truth:** `12-roadmap-data.toml` is the authoritative
> data file. This `.md` is generated. To add an item: edit the TOML,
> re-run the script.

---

<!-- COMPUTED:status -->
| Metric | Value |
|---|---|
| Total items | 56 |
| Done (`[x]`) | 33 |
| In-progress (`[~]`) | 13 |
| Open (`[ ]`) | 10 |
| Last update | 2026-06-25 07:30 UTC |
| Last commit covered | `2eb7d88` |
<!-- END COMPUTED -->

---

## Production gates — all 6 must be `[x]` to declare production-ready

<!-- COMPUTED:gates -->
- [x] **Gate-1 Lint:** `cargo run -p educore-core --bin lint --features lint` exits 0
      _check: `cmd:cargo run -p educore-core --bin lint --features lint` → exit 0_
- [ ] **Gate-2 Tests:** `cargo test --workspace` passes (zero failures)
      _check: `manual:cargo test --workspace` → manual: cargo test --workspace_
- [ ] **Gate-3 Clippy:** `cargo clippy --workspace --all-targets -- -D warnings` exits 0
      _check: `cmd:cargo clippy --workspace --all-targets -- -D warnings` → exit 101_
- [ ] **Gate-4 Fmt:** `cargo fmt --all -- --check` exits 0
      _check: `cmd:cargo fmt --all -- --check` → exit 1_
- [ ] **Gate-5 Adapters:** All 4 storage adapters' `create_schema()` round-trip on a fresh DB
      _check: `manual:cargo test -p educore-storage-parity --features all-dbs` → manual: cargo test -p educore-storage-parity --features all-dbs_
- [ ] **Gate-6 Decisions:** All items in `13-decision-needed.md` resolved
      _check: `manual:see 13-decision-needed.md` → manual: see 13-decision-needed.md_
<!-- END COMPUTED -->

---

## P0 — Production blockers (must close before deploy)

### P0-ENGINE — Engine must enforce rules at runtime, not just lint

<!-- COMPUTED:items.P0.ENGINE -->
- [x] **A-1** Macro emits `ColumnType::Custom("UNKNOWN")` for every field
      **Source:** e036f73, crates/infra/storage/src/entities.rs
      **Check:** `file:crates/adapters/storage-mysql/src/schema.rs regex:ColumnType::(Uuid|String|...` → _schema.rs:ColumnType::(Uuid|String|I64|Bool)_

- [ ] **A-2** `EntityDescriptor.indexes` is always empty
      **Source:** e036f73, docs/specs/port/storage.md
      **Check:** `commit:derive.*indexes|EntityDescriptor.*indexes` → _git log grep: derive.*indexes|EntityDescriptor.*indexes_

- [ ] **A-3** `EntityDescriptor.foreign_keys` is always empty
      **Source:** e036f73
      **Check:** `commit:derive.*foreign_key|EntityDescriptor.*foreign_keys` → _git log grep: derive.*foreign_key|EntityDescriptor.*foreign_keys_

- [ ] **A-4** `EntityDescriptor.rls` is always empty
      **Source:** e036f73, docs/schemas/tenancy-schema.md
      **Check:** `commit:rls|school_isolation` → _git log grep: rls|school_isolation_

- [x] **A-6** MySQL adapter: RLS skipped with TODO
      **Source:** crates/adapters/storage-mysql/src/schema.rs
      **Check:** `file:crates/adapters/storage-mysql/src/schema.rs regex:policy|rls` → _schema.rs:policy|rls_

- [~] **A-7** SQLite adapter: RLS not supported (documented limitation)
      **Source:** documented in adapter README
      **Check:** `manual:accepted limitation per docs/decisions/ADR-017` → _manual: accepted limitation per docs/decisions/ADR-017_

- [x] **B-3** Pre-existing clippy warnings in `crates/infra/core/src/lint.rs:83/44/394`
      **Source:** docs/audit_reports/findings/wave1-lint.md
      **Check:** `cmd:cargo clippy -p educore-core --lib -- -D warnings` → _exit 0_

- [x] **D-9** `educore-auth` has unresolved clippy warnings
      **Source:** docs/audit_reports/findings/wave5-docs-1.md
      **Check:** `cmd:cargo clippy -p educore-auth --all-targets -- -D warnings` → _exit 0_

- [x] **D-10** Verify proc-macro `as` cast in `crates/infra/query-derive/src/lib.rs` is fixed
      **Source:** docs/build-plan.md:73
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:as (i32|u32|u64|i64)!` → _lib.rs:as (i32|u32|u64|i64)_
<!-- END COMPUTED -->

### P0-STORAGE — All 4 adapters must work end-to-end

<!-- COMPUTED:items.P0.STORAGE -->
- [x] **A-5** All 4 adapters override `create_schema()`
      **Source:** Cluster A stage 3
      **Check:** `commit:create_schema` → _git log grep: create_schema_

- [ ] **H-5** SurrealDB change-stream stubs unimplemented (apply_snapshot, watch_changes, cursor_for, advance_cursor)
      **Source:** wave3-storage-surrealdb.md ADAPTER-SD-005..008
      **Check:** `file:crates/adapters/storage-surrealdb/src/storage.rs regex:is not yet implement...` → _storage.rs:is not yet implemented|todo!|unimplement_

- [x] **C-3** Testkit outbox drain not wired to in-process bus
      **Source:** wave4-testkit.md TOOL-TK-001
      **Check:** `file:crates/tools/testkit/src/storage.rs regex:outbox.*bus|drain.*publish` → _storage.rs:outbox.*bus|drain.*publish_
<!-- END COMPUTED -->

### P0-DOCS — Decisions must be resolved (see `13-decision-needed.md`)

<!-- COMPUTED:items.P0.DOCS -->
- [x] **D-4** Phase 17 missing from build plan (or doesn't exist)
      **Source:** docs/build-plan.md
      **Check:** `file:docs/build-plan.md regex:Phase 17|phase 17` → _build-plan.md:Phase 17|phase 17_

- [ ] **D-5** Cross-domain ownership collisions — 3 ADRs needed (SubjectAttendance, ExamAttendance, SpeechSlider)
      **Source:** wave6-specs-1.md
      **Check:** `commit:cross-domain ownership|ADR.*ownership` → _git log grep: cross-domain ownership|ADR.*ownership_

- [ ] **D-6** SurrealDB vs Postgres as primary backend
      **Source:** wave5-docs-1.md; AGENTS.md vs ADR-017 vs ADR-018
      **Check:** `commit:ADR.*primary.*backend|SurrealDB.*primary` → _git log grep: ADR.*primary.*backend|SurrealDB.*primary_

- [~] **D-7** Public API renames (identity types + event types canonical)
      **Source:** wave5-docs-2.md
      **Check:** `manual:see 13-decision-needed.md D-7` → _manual: see 13-decision-needed.md D-7_
<!-- END COMPUTED -->

---

## P1 — High priority (feature incomplete)

### P1-TESTS — Domain test coverage

<!-- COMPUTED:items.P1.TESTS -->
- [x] **TS-1** academic: 2/14+ aggregates tested (Class + Subject)
      **Source:** Wave 2 (subject) + Wave 4 (class)
      **Check:** `file-exists:crates/domains/academic/tests/subject.rs` → _subject.rs exists_

- [x] **TS-2** assessment: 2/20+ aggregates tested (Exam + MarksGrade)
      **Source:** Wave 3 + Wave 4
      **Check:** `file-exists:crates/domains/assessment/tests/exam.rs` → _exam.rs exists_

- [x] **TS-3** attendance: 2/10 aggregates tested (StudentAttendance + SubjectAttendance)
      **Source:** Wave 1 + Wave 4
      **Check:** `file-exists:crates/domains/attendance/tests/aggregates.rs` → _aggregates.rs exists_

- [x] **TS-4** cms: 2/20+ aggregates tested (Page + News)
      **Source:** Wave 2 + Wave 4
      **Check:** `file-exists:crates/domains/cms/tests/page.rs` → _page.rs exists_

- [x] **TS-5** communication: 2/27 aggregates tested (Notice + ComplaintType)
      **Source:** Wave 1 + Wave 4
      **Check:** `file-exists:crates/domains/communication/tests/notice.rs` → _notice.rs exists_

- [x] **TS-6** documents: 2/20 aggregates tested (FormDownload + PostalReceive)
      **Source:** Wave 1 + Wave 4
      **Check:** `file-exists:crates/domains/documents/tests/form_handlers.rs` → _form_handlers.rs exists_

- [x] **TS-7** events-domain: 3/7 aggregates tested (Holiday, Weekend, CalendarSetting)
      **Source:** Wave 5
      **Check:** `file-exists:crates/cross-cutting/events-domain/tests/holiday.rs` → _holiday.rs exists_

- [x] **TS-8** facilities: 2/16 aggregates tested (Vehicle + Route)
      **Source:** Wave 2 + Wave 4
      **Check:** `file-exists:crates/domains/facilities/tests/vehicle.rs` → _vehicle.rs exists_

- [x] **TS-9** finance: 2/51 aggregates tested (Wallet + FmFeesGroup)
      **Source:** Wave 3 + Wave 4
      **Check:** `file-exists:crates/domains/finance/tests/wallet.rs` → _wallet.rs exists_

- [x] **TS-10** hr: 2/40+ aggregates tested (Department + Designation)
      **Source:** Wave 3 + Wave 4
      **Check:** `file-exists:crates/domains/hr/tests/department.rs` → _department.rs exists_

- [x] **TS-11** library: 2/9 aggregates tested (BookCategory + BookIssue)
      **Source:** Wave 1 + Wave 4
      **Check:** `file-exists:crates/domains/library/tests/aggregates.rs` → _aggregates.rs exists_

- [~] **TS-12** 48 integration tests pass across 11 domains
      **Source:** Wave 1-5
      **Check:** `manual:run each domain's test suite` → _manual: run each domain's test suite_

- [~] **TS-13** Storage-parity tests for the 25 covered aggregates
      **Source:** crates/tools/storage-parity/tests/
      **Check:** `manual:scenarios needed for each covered aggregate` → _manual: scenarios needed for each covered aggregate_
<!-- END COMPUTED -->

### P1-WORKFLOWS — Multi-step workflows

<!-- COMPUTED:items.P1.WORKFLOWS -->
- [x] **E-1** Outbox relay wired
      **Source:** Cluster B
      **Check:** `file-exists:crates/cross-cutting/events/src/relay_envelope.rs` → _relay_envelope.rs exists_

- [x] **E-2** Subscriber registration
      **Source:** Cluster B
      **Check:** `commit:subscriber` → _git log grep: subscriber_

- [x] **E-3** `tests/workflows.rs` per domain (11/11 domains have it)
      **Source:** Cluster B
      **Check:** `file-exists:crates/domains/academic/tests/workflows.rs` → _workflows.rs exists_

- [ ] **E-4** Saga / compensating actions for multi-step workflows
      **Source:** wave7-workflows.md WF-006
      **Check:** `file:crates/cross-cutting/sync/src/lib.rs regex:compensat` → _lib.rs:compensat_

- [x] **E-5** Bus port completion
      **Source:** Cluster B
      **Check:** `file-exists:crates/cross-cutting/events/src/event_bus.rs` → _event_bus.rs exists_
<!-- END COMPUTED -->

### P1-API — Public API consistency

<!-- COMPUTED:items.P1.API -->
- [ ] **F-3** Naming drift sweep (StudentId vs StudentIdentifier, table singular vs plural)
      **Source:** wave6-specs-1.md
      **Check:** `commit:naming.*drift|rename.*Identifier` → _git log grep: naming.*drift|rename.*Identifier_
<!-- END COMPUTED -->

---

## P2 — Medium priority (technical debt)

<!-- COMPUTED:items.P2 -->
- [x] **B-1** Code→spec direction false positives for re-exports
      **Source:** Cluster D follow-ups
      **Check:** `file:crates/infra/core/src/lint.rs regex:re_export|reexport` → _lint.rs:re_export|reexport_

- [x] **B-2** `as` cast detection is regex-based (false positives possible)
      **Source:** Cluster D follow-ups
      **Check:** `file:crates/infra/core/src/lint.rs regex:as_.*cast` → _lint.rs:as_.*cast_

- [x] **B-4** Coverage matrix sync's TOML parser is line-based
      **Source:** Cluster D follow-ups
      **Check:** `file:crates/infra/core/src/lint.rs regex:toml.*parser|line.*based` → _lint.rs:toml.*parser|line.*based_

- [~] **B-5** Anti-pattern check doesn't catch `as` chains on numeric constants
      **Source:** Cluster D follow-ups
      **Check:** `manual:re-validate on real codebase` → _manual: re-validate on real codebase_

- [x] **C-1** Idempotency.record() callers must migrate to record_outcome()
      **Source:** 5382a6e (port) + 4 adapter commits
      **Check:** `commit:record_outcome` → _git log grep: record_outcome_

- [ ] **C-2** QW-13 MySQL defense-in-depth
      **Source:** QW-13, d2f52c9
      **Check:** `commit:QW-13|d2f52c9` → _git log grep: QW-13|d2f52c9_

- [x] **H-1** StorageAdapter::create_schema() impls (all 4 adapters)
      **Source:** Cluster A stage 3
      **Check:** `file-exists:crates/adapters/storage-surrealdb/src/schema.rs` → _schema.rs exists_

- [ ] **H-2** Transaction::TenantContext propagation
      **Source:** wave4-storage-port.md PORT-STORE-002
      **Check:** `file:crates/infra/storage/src/transaction.rs regex:TenantContext` → _transaction.rs:TenantContext_

- [x] **H-3** Atomic audit-write in same transaction
      **Source:** wave4-storage-port.md PORT-STORE-013
      **Check:** `commit:PORT-STORE-013` → _git log grep: PORT-STORE-013_

- [x] **H-4** Outbox partition enforcement at storage port level
      **Source:** wave4-testkit.md TOOL-TK-004
      **Check:** `commit:TOOL-TK-004` → _git log grep: TOOL-TK-004_

- [~] **H-6** Port-adapter gaps (auth/event-bus/notify/payment/files/integrations)
      **Source:** Various wave3-* findings
      **Check:** `manual:see wave3 findings` → _manual: see wave3 findings_

- [x] **F-2** `#[derive(DomainQuery)]` adoption
      **Source:** 0/310 across all domains
      **Check:** `cmd:grep -r DomainQuery crates/domains/ crates/cross-cutting/ | wc -l` → _exit 0_
<!-- END COMPUTED -->

---

## P3 — Low priority (cosmetic / housekeeping)

<!-- COMPUTED:items.P3 -->
- [~] **B-6** Single-line `mod tests { ... }` exemption regression test sweep
      **Source:** Cluster D
      **Check:** `manual:regression sweep complete` → _manual: regression sweep complete_

- [~] **D-1** Pre-existing unrelated stash (user decision)
      **Source:** session start
      **Check:** `manual:user must resolve` → _manual: user must resolve_

- [~] **D-2** `docs_guidlines/` directory deletions (user decision)
      **Source:** session start
      **Check:** `manual:user must commit or restore` → _manual: user must commit or restore_

- [x] **I-1** Crate count drift (AGENTS.md: 34 → 37 actual)
      **Source:** wave5-docs-1.md
      **Check:** `file:AGENTS.md regex:36 internal crates|37 packages` → _AGENTS.md:36 internal crates|37 packages_

- [~] **I-2** SurrealDB shipping status tri-contradiction
      **Source:** wave5-docs-1.md
      **Check:** `manual:resolve SurrealDB primary status across docs` → _manual: resolve SurrealDB primary status across docs_

- [x] **I-3** Phase 17 missing from build plan
      **Source:** See D-4 (P0)
      **Check:** `duplicate:D-4` → _build-plan.md:Phase 17|phase 17_

- [~] **I-4** `library-docs.md` phantom `Engine::builder()` APIs
      **Source:** wave5-docs-2.md
      **Check:** `manual:audit library-docs.md APIs vs code` → _manual: audit library-docs.md APIs vs code_

- [~] **I-5** `guides/*.md` phantom `engine.<domain>()` accessors
      **Source:** wave5-docs-6.md
      **Check:** `manual:audit guides/*.md accessors vs code` → _manual: audit guides/*.md accessors vs code_

- [~] **I-6** ADR review (013, 015, 016, 017, 018)
      **Source:** wave5-docs-2.md
      **Check:** `manual:ADR review checklist complete` → _manual: ADR review checklist complete_
<!-- END COMPUTED -->

---

## Notes

- **Cluster E, F, G** are closed (per remediation commits `5a6e37c`, `e700c67`, `faaecca`). Lint anti-pattern violations = 0.
- **48 integration tests** across 11 domains added in Wave 1-5.
- **`remediation-checkpoint-vertical-slice-wave{1..5}` tags** mark each wave's end.
- **`docs/coverage.toml`** updated for each new test file (rows added/extended).
- **Branch collisions** (Wave 4: library → communication, assessment → finance) were recovered manually. Wave 5 introduced a recovery pattern for agents that commit before being aborted.

## See also

- [`00-overview.md`](00-overview.md) — cluster overview
- [`08-dependency-graph.md`](08-dependency-graph.md) — cluster dependencies
- [`09-quick-wins.md`](09-quick-wins.md) — closed quick wins (QW-1 .. QW-15)
- [`11-follow-up-tracker.md`](11-follow-up-tracker.md) — historical record (replaced by this file)
- [`13-decision-needed.md`](13-decision-needed.md) — items requiring user input
- [`12-roadmap-data.toml`](12-roadmap-data.toml) — source-of-truth data file
