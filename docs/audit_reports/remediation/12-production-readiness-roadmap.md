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
| Total items | 157 |
| Done (`[x]`) | 50 |
| In-progress (`[~]`) | 13 |
| Open (`[ ]`) | 94 |
| Last update | 2026-06-25 08:46 UTC |
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

- [ ] **ADR-014-IDEM-CONFLICT-VARIANT** DomainError missing IdempotencyConflict + IdempotencyPending variants; ADR-014 § Decision 4,9 mandates both
      **Source:** docs/decisions/ADR-014-Idempotency.md, docs/audit_reports/findings/wave4-core.md CORE-003
      **Check:** `file:crates/infra/core/src/error.rs regex:IdempotencyConflict|IdempotencyPending` → _error.rs:IdempotencyConflict|IdempotencyPending_

- [ ] **ADR-013-LINT-TIER-CHECK** Lint sub-module does NOT enforce tier-boundary direction (domain importing adapter); ADR-013 § Boundary enforcement item 2
      **Source:** docs/decisions/ADR-013-CrateLayout.md
      **Check:** `file:crates/infra/core/src/lint.rs regex:tier.?boundary|tier_boundary` → _lint.rs:tier.?boundary|tier_boundary_

- [ ] **STD-CI-CROSS-COMPILE** No .github/workflows/; cross-compile mandate (Linux x86_64, aarch64, macOS, Windows) is unverified
      **Source:** docs/code-standards.md § Cross-Compilation
      **Check:** `file-exists:.github/workflows/ci.yml` → _ci.yml missing_

- [x] **FND-INFRA-QD-001** where_has macro discards relation/closure; relational filters never added to AST
      **Source:** docs/audit_reports/findings/wave4-query-derive.md INFRA-QD-001
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:where_has|HasRelation` → _lib.rs:where_has|HasRelation_

- [x] **FND-CORE-001** lint::check_coverage_matrix is a no-op (let _ = status_tested); coverage matrix gate never enforced
      **Source:** docs/audit_reports/findings/wave4-core.md CORE-001
      **Check:** `file:crates/infra/core/src/lint.rs regex:check_coverage_matrix|enforce.*coverage` → _lint.rs:check_coverage_matrix|enforce.*coverage_
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

- [ ] **PORT-STORAGE-REPOS** StorageAdapter trait missing ~80 aggregate repository handles (students, guardians, classes, …) per spec
      **Source:** docs/ports/storage.md § Trait: StorageAdapter
      **Check:** `cmd:grep -c 'fn students\|fn guardians\|fn classes' crates/infra/storage/src/por...` → _exit 1_

- [ ] **PORT-STORAGE-SD-SYNC** SurrealDB sync primitives unimplemented (apply_snapshot / watch_changes / cursor_for / advance_cursor)
      **Source:** docs/audit_reports/remediation/12-production-readiness-roadmap.md H-5
      **Check:** `file:crates/adapters/storage-surrealdb/src/storage.rs regex:is not yet implement...` → _storage.rs:is not yet implemented_

- [ ] **PORT-STORAGE-MACRO-RLS** DomainQuery macro emits rls: vec![] (A-4); PG aggregate tables have no CREATE POLICY
      **Source:** docs/schemas/tenancy-schema.md § 7, roadmap A-4
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:rls: ::std::vec!` → _lib.rs:rls: ::std::vec_

- [ ] **FND-PORT-STORE-001** StorageAdapter trait exposes migrate() but docs mandate create_schema(); port name and consumer name diverge
      **Source:** docs/audit_reports/findings/wave4-storage-port.md PORT-STORE-001
      **Check:** `commit:create_schema|StorageAdapter::create_schema` → _git log grep: create_schema|StorageAdapter::create_schema_

- [ ] **FND-PORT-STORE-003** Outbox::append/pending/mark_published have no school_id on trait; TenantContext not propagated through Outbox
      **Source:** docs/audit_reports/findings/wave4-storage-port.md PORT-STORE-003
      **Check:** `commit:outbox.*school_id|outbox.*tenant` → _git log grep: outbox.*school_id|outbox.*tenant_
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

- [ ] **PORT-STORAGE-STREAMING** StudentRepository::stream returning BoxStream<Result<Student>> is not declared
      **Source:** docs/ports/storage.md § Streaming
      **Check:** `file:crates/infra/storage/src/port.rs regex:fn stream` → _port.rs:fn stream_

- [ ] **PORT-AUTH-SAML** SamlAuthProvider not shipped (enterprise IdP)
      **Source:** docs/ports/authentication.md § Configuration
      **Check:** `file-exists:crates/adapters/auth/src/saml.rs` → _saml.rs missing_

- [x] **PORT-NOTIFY-RATE-LIMIT** Per-tenant per-channel rate limiting not implemented at adapter boundary
      **Source:** docs/ports/notifications.md § Rate Limiting
      **Check:** `file:crates/adapters/notify/src/reliability.rs regex:rate.?limit` → _reliability.rs:rate.?limit_

- [ ] **PORT-NOTIFY-CRITICAL** Priority::Critical requires synchronous delivery; no bypass path in adapters
      **Source:** docs/ports/notifications.md § Priority
      **Check:** `file:crates/adapters/notify/src/email.rs regex:Critical` → _email.rs:Critical_

- [ ] **PORT-PAY-3DS** PaymentError::ThreeDSRequired declared but no 3DS auth-capture orchestration
      **Source:** docs/ports/payments.md § Error Type
      **Check:** `file:crates/adapters/payment/src/stripe.rs regex:ThreeDS` → _stripe.rs:ThreeDS_

- [ ] **PORT-FILE-LIFECYCLE** Lifecycle-rule config port (Hot→Cool→Archive→expire) not implemented
      **Source:** docs/ports/file-storage.md § Lifecycle Rules
      **Check:** `file-exists:crates/adapters/files/src/lifecycle.rs` → _lifecycle.rs missing_

- [ ] **PORT-FILE-OFFLINE-CACHE** Offline-mode local URI on FileReference not implemented
      **Source:** docs/ports/file-storage.md § Offline Mode
      **Check:** `file:crates/adapters/files/src/local.rs regex:local://` → _local.rs:local://_

- [ ] **PORT-FILE-TENANT-DENIAL-TEST** Cross-tenant denial test missing from tests/
      **Source:** docs/ports/file-storage.md § Testing
      **Check:** `file-exists:crates/adapters/files/tests/cross_tenant.rs` → _cross_tenant.rs missing_

- [ ] **PORT-INT-OAUTH2-HELPER** Per-tenant OAuth2 client-credentials token cache + refresh not extracted as a port helper
      **Source:** docs/ports/integrations.md § OAuth2 Client Credentials
      **Check:** `file:crates/adapters/integrations/src/lib.rs regex:client_credentials` → _lib.rs:client_credentials_

- [ ] **PORT-EVENTBUS-AUDIT** Spec requires every publish/consume recorded in audit; no AuditSink wiring on the bus
      **Source:** docs/ports/event-bus.md § Audit
      **Check:** `file:crates/cross-cutting/events/src/event_bus.rs regex:audit_log|AuditSink` → _event_bus.rs:audit_log|AuditSink_

- [ ] **SCHEMA-AUDIT-GDPR-ERASURE** Report.SubjectErasure.Execute command missing; PII anonymization in audit not implemented
      **Source:** docs/schemas/audit-schema.md § 8.2
      **Check:** `file:crates/domains/platform/src/commands.rs regex:SubjectErasure` → _commands.rs missing_

- [ ] **SCHEMA-AUDIT-FERPA** Report.ParentAccess.Generate command missing
      **Source:** docs/schemas/audit-schema.md § 8.3
      **Check:** `file:crates/domains/communication/src/commands.rs regex:ParentAccess` → _commands.rs:ParentAccess_

- [ ] **SCHEMA-AUDIT-REGULATOR** Report.RegulatorAudit.Generate command missing
      **Source:** docs/schemas/audit-schema.md § 8.4
      **Check:** `file:crates/domains/platform/src/commands.rs regex:RegulatorAudit` → _commands.rs missing_

- [ ] **SCHEMA-EVENTLOG-RETENTION** event_log has no retention sweeper (compare with audit RetentionSweepDue)
      **Source:** docs/schemas/event-schema.md § 9
      **Check:** `file:crates/cross-cutting/audit/src/retention.rs regex:event_log` → _retention.rs:event_log_

- [ ] **SCHEMA-CMD-ASYNC** Async command handle (CommandHandle / engine.commands.status) not implemented
      **Source:** docs/schemas/command-schema.md § 13
      **Check:** `file:crates/educore/src/lib.rs regex:CommandHandle` → _lib.rs:CommandHandle_

- [ ] **SCHEMA-CMD-BULK-EVENTS** BulkCommandStarted / BulkCommandItemProcessed / BulkCommandCompleted events not declared
      **Source:** docs/schemas/command-schema.md § 12
      **Check:** `file:crates/cross-cutting/events/src/domain_event.rs regex:BulkCommand` → _domain_event.rs:BulkCommand_

- [ ] **SCHEMA-TENANT-CONFIG-SVC** ConfigurationService port not implemented; per-tenant settings live in ad-hoc repos
      **Source:** docs/schemas/tenancy-schema.md § 9
      **Check:** `file-exists:crates/cross-cutting/platform/src/configuration.rs` → _configuration.rs missing_

- [x] **SCHEMA-IDEM-MIGRATE-OUTCOME** Idempotency::record() callers must migrate to record_outcome(); sweep not complete (C-1)
      **Source:** roadmap C-1
      **Check:** `cmd:grep -r 'Idempotency::record(' crates/ --include='*.rs' | grep -v record_out...` → _exit 0_

- [ ] **SCHEMA-IDEM-TTL-SWEEP** No engine-side idempotency TTL sweep; only audit has RetentionSweepDue
      **Source:** docs/schemas/command-schema.md § 6
      **Check:** `file:crates/cross-cutting/audit/src/retention.rs regex:Idempotency` → _retention.rs:Idempotency_

- [x] **X-CUT-C1-IDEM** Idempotency.record() callers must migrate to record_outcome(); sweep incomplete
      **Source:** roadmap C-1
      **Check:** `cmd:grep -r 'Idempotency::record(' crates/ --include='*.rs' | grep -v record_out...` → _exit 0_

- [x] **X-CUT-OUTBOX-RELAY-PARTITION** Outbox relay polls without per-school partitioning; back-pressure model unspecified
      **Source:** crates/cross-cutting/events/src/relay.rs
      **Check:** `file:crates/cross-cutting/events/src/relay.rs regex:school_id` → _relay.rs:school_id_

- [ ] **X-CUT-NOTIFY-BULK-EVENT** Per-recipient NotificationSent event emission not wired for bulk send
      **Source:** docs/ports/notifications.md § Bulk Send
      **Check:** `file:crates/adapters/notify/src/services.rs regex:NotificationSent` → _services.rs:NotificationSent_

- [ ] **X-CUT-WF-COMPENSATE** Saga / compensating action library not implemented (E-4)
      **Source:** roadmap E-4
      **Check:** `file:crates/cross-cutting/sync/src/lib.rs regex:compensat` → _lib.rs:compensat_

- [ ] **ADR-014-OUTCOME-FIELDS** Idempotency record missing aggregate_version, etag, duration, emitted_event_ids fields (ADR-014 § Decision 6)
      **Source:** docs/decisions/ADR-014-Idempotency.md § Decision 6
      **Check:** `file:crates/infra/storage/src/idempotency.rs regex:aggregate_version|etag` → _idempotency.rs:aggregate_version|etag_

- [ ] **ADR-015-CARGO-DENY** No deny.toml / cargo deny; license audit unverified (ADR-015 § Dependency hygiene policy rule 5)
      **Source:** docs/decisions/ADR-015-ExternalCrates.md
      **Check:** `file-exists:deny.toml` → _deny.toml missing_

- [ ] **FND-SPEC-3-001** HR spec uses legacy Sm_ brand prefix (SmAssignClassTeacher) violating AGENTS.md 'Brand is Educore'
      **Source:** docs/audit_reports/findings/wave6-specs-3.md SPEC-3-001
      **Check:** `commit:Sm_.*rename|hr.*brand.*cleanup` → _git log grep: Sm_.*rename|hr.*brand.*cleanup_

- [ ] **FND-SPEC-4-001** docs/specs/sync/ contains only overview.md (1 of 11 files)
      **Source:** docs/audit_reports/findings/wave6-specs-4.md SPEC-4-001
      **Check:** `commit:sync.*spec|specs/sync/tables.md` → _git log grep: sync.*spec|specs/sync/tables.md_

- [ ] **FND-CORE-005** ids module rustdoc links to crate::id_gen::IdGenerator but no id_gen module exists
      **Source:** docs/audit_reports/findings/wave4-core.md CORE-005
      **Check:** `commit:id_gen.*module|core.*id_gen` → _git log grep: id_gen.*module|core.*id_gen_

- [ ] **FND-UMB-005** Umbrella deps (34) vs re-exports (32) vs AGENTS.md (36 internal crates); inventory stale
      **Source:** docs/audit_reports/findings/wave4-umbrella.md UMB-005
      **Check:** `commit:AGENTS.md.*crate.*inventory|34 internal crates` → _git log grep: AGENTS.md.*crate.*inventory|34 internal crates_
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

- [x] **ADR-013-COUNTS-DRIFT** ADR-013 / AGENTS.md / architecture.md disagree on crate count (33 vs 36 vs 37); canonicalise
      **Source:** docs/decisions/ADR-013-CrateLayout.md, AGENTS.md, docs/architecture.md
      **Check:** `cmd:grep -nE '3[3-7].*crates|3[3-7].*internal' AGENTS.md docs/architecture.md do...` → _exit 0_

- [ ] **ADR-013-NO-TIER-CARGO** No lint guard against adding Cargo.toml at tier roots (ADR-013 § Negative consequences item 2)
      **Source:** docs/decisions/ADR-013-CrateLayout.md
      **Check:** `file:crates/infra/core/src/lint.rs regex:tier.root|tier_root` → _lint.rs:tier.root|tier_root_

- [ ] **ADR-018-SYNC-INPROCESS-TIER** sync-inprocess lives at crates/cross-cutting/ not crates/adapters/ as ADR-018 § 3 says
      **Source:** docs/decisions/ADR-018-SyncEngine.md § 3
      **Check:** `file-exists:crates/adapters/sync-inprocess/Cargo.toml` → _Cargo.toml missing_

- [ ] **ADR-016-GRAPHIFY-HOOK** Graphify post-commit hook per-user only; fresh clones miss the automation
      **Source:** docs/decisions/ADR-016-EngineGraph.md
      **Check:** `file-exists:.githooks/post-commit` → _post-commit missing_

- [ ] **LIB-DOMAIN-ACCESSORS** library-docs.md shows engine.students()/engine.fees() accessors that don't exist on Engine facade
      **Source:** docs/library-docs.md § Common Workflows
      **Check:** `file:crates/tools/sdk/src/engine.rs regex:fn students|fn fees` → _engine.rs:fn students|fn fees_

- [ ] **LIB-TYPED-EVENT-SUBSCRIBE** library-docs.md shows engine.events().subscribe::<T>() API; actual is untyped SubscribeOptions
      **Source:** docs/library-docs.md § Subscribing to Events
      **Check:** `file:crates/cross-cutting/events/src/event_bus.rs regex:subscribe_typed` → _event_bus.rs:subscribe_typed_
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
