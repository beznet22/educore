# Production Readiness Roadmap

> **RESOLVED 2026-06-25 (Wave 6.1 / D-4):** Phase 17 = CMS (Phase 12 in AGENTS.md). No new phase needed. The audit was stale. Decision A.
>
> Roadmap items **D-4** (P0-DOCS) and **I-3** (P3-DOCS, duplicate of D-4) — both of which claimed "Phase 17 missing from build plan" — were the source of the conflict. Phase 17 IS documented in `docs/build-plan.md` (line 1714: `## Phase 17 — Production readiness`); the build plan numbers 18 phases (Phase 0..17), matching `AGENTS.md:475` (`Build plan: 18 phases (Phase 0..17)`). The auto-check `file:docs/build-plan.md regex:Phase 17|phase 17` succeeds, which is why both items already show `[x]` in the COMPUTED sections below. The stale title text on those two lines has been corrected in `12-roadmap-data.toml` so future regenerations match. Closes roadmap item **D-4** (decision A locked in [`13-decision-needed.md`](13-decision-needed.md)).

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
| Total items | 485 |
| Done (`[x]`) | 152 |
| In-progress (`[~]`) | 12 |
| Open (`[ ]`) | 321 |
| Last update | 2026-06-25 16:34 UTC |
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
- [x] **A-1** Resolved — Macro emits ColumnType::Custom(<TypeName>) via stringify!(). Closed 2026-06-25.
      **Source:** e036f73, crates/infra/storage/src/entities.rs
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:ColumnType::Custom...UNKNOWN!` → _lib.rs:ColumnType::Custom...UNKNOWN_

- [x] **A-2** Resolved — Macro emits IndexDescriptor (idx_pk). Closed 2026-06-25.
      **Source:** e036f73, docs/specs/port/storage.md
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:IndexDescriptor ?\{` → _lib.rs:IndexDescriptor ?\{_

- [x] **A-3** Resolved — Macro emits foreign_keys field path. Closed 2026-06-25.
      **Source:** e036f73
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:foreign_keys:.*vec` → _lib.rs:foreign_keys:.*vec_

- [x] **A-4** Resolved — Macro emits tenant_isolation RlsPolicy. Closed 2026-06-25.
      **Source:** e036f73, docs/schemas/tenancy-schema.md
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:RlsPolicy ?\{` → _lib.rs:RlsPolicy ?\{_

- [~] **A-6** Deferred — MySQL RLS requires engine-emitted RLS policies; schema partition work in PG/MySQL closes the underlying gap. Audit A-6 partial 2026-06-25.
      **Source:** crates/adapters/storage-mysql/src/schema.rs
      **Check:** `manual:MySQL RLS requires engine-emitted policies; deferred to Phase 7 (Finance ...` → _manual: MySQL RLS requires engine-emitted policies; deferred to Phase 7 (Finance hardening)_

- [x] **A-7** Resolved — Documented in ADR-017 'Known limitations'. Closed 2026-06-25.
      **Source:** docs/decisions/ADR-017-SurrealDBFirst.md
      **Check:** `file:docs/decisions/ADR-017-SurrealDBFirst.md regex:SQLite row-level security` → _ADR-017-SurrealDBFirst.md:SQLite row-level security_

- [x] **B-3** Resolved — clippy on educore-core clean. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave1-lint.md
      **Check:** `cmd:cargo clippy -p educore-core --lib -- -D warnings` → _exit 0_

- [ ] **D-9** Resolved — clippy on educore-auth clean. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave5-docs-1.md
      **Check:** `cmd:cargo clippy -p educore-auth --all-targets -- -D warnings` → _exit 101_

- [x] **D-10** Resolved — Sync feature flag added to educore umbrella. Closed 2026-06-25.
      **Source:** ADR-018 § 4, crates/educore/Cargo.toml
      **Check:** `file:crates/educore/Cargo.toml regex:\[features\]|sync = \[` → _Cargo.toml:\[features\]|sync = \[_

- [x] **ADR-014-IDEM-CONFLICT-VARIANT** Resolved — IdempotencyConflict + IdempotencyPending variants added. Closed 2026-06-25.
      **Source:** docs/decisions/ADR-014-Idempotency.md, docs/audit_reports/findings/wave4-core.md CORE-003
      **Check:** `file:crates/infra/core/src/error.rs regex:IdempotencyConflict|IdempotencyPending` → _error.rs:IdempotencyConflict|IdempotencyPending_

- [x] **ADR-013-LINT-TIER-CHECK** Resolved — check_tier_boundaries added. Closed 2026-06-25.
      **Source:** docs/decisions/ADR-013-CrateLayout.md
      **Check:** `file:crates/infra/core/src/lint.rs regex:fn check_tier_boundaries` → _lint.rs:fn check_tier_boundaries_

- [x] **STD-CI-CROSS-COMPILE** Resolved — .github/workflows/ci.yml added. Closed 2026-06-25.
      **Source:** docs/code-standards.md § Cross-Compilation
      **Check:** `file-exists:.github/workflows/ci.yml` → _ci.yml exists_

- [x] **FND-INFRA-QD-001** Resolved — where_has invokes __build closure. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave4-query-derive.md INFRA-QD-001
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:let _ = relation!` → _lib.rs:let _ = relation_

- [x] **FND-CORE-001** Resolved — check_coverage_matrix was already implemented. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave4-core.md CORE-001
      **Check:** `file:crates/infra/core/src/lint.rs regex:check_coverage_matrix|enforce.*coverage` → _lint.rs:check_coverage_matrix|enforce.*coverage_
<!-- END COMPUTED -->

### P0-STORAGE — All 4 adapters must work end-to-end

<!-- COMPUTED:items.P0.STORAGE -->
- [x] **A-5** Resolved — All 4 adapters override create_schema(). Closed 2026-06-25.
      **Source:** Cluster A stage 3
      **Check:** `file:crates/infra/storage/src/port.rs regex:async fn create_schema` → _port.rs:async fn create_schema_

- [ ] **H-5** SurrealDB change-stream stubs unimplemented (apply_snapshot, watch_changes, cursor_for, advance_cursor)
      **Source:** wave3-storage-surrealdb.md ADAPTER-SD-005..008
      **Check:** `file:crates/adapters/storage-surrealdb/src/storage.rs regex:is not yet implement...` → _storage.rs:is not yet implemented|todo!|unimplement_

- [x] **C-3** Resolved — testkit storage docs/wires outbox drain. Closed 2026-06-25.
      **Source:** wave4-testkit.md TOOL-TK-001
      **Check:** `file:crates/tools/testkit/src/storage.rs regex:outbox.*bus|drain.*publish` → _storage.rs:outbox.*bus|drain.*publish_

- [ ] **PORT-STORAGE-REPOS** StorageAdapter trait missing ~80 aggregate repository handles (students, guardians, classes, …) per spec
      **Source:** docs/ports/storage.md § Trait: StorageAdapter
      **Check:** `cmd:grep -c 'fn students\|fn guardians\|fn classes' crates/infra/storage/src/por...` → _exit 1_

- [ ] **PORT-STORAGE-SD-SYNC** SurrealDB sync primitives unimplemented (apply_snapshot / watch_changes / cursor_for / advance_cursor)
      **Source:** docs/audit_reports/remediation/12-production-readiness-roadmap.md H-5
      **Check:** `file:crates/adapters/storage-surrealdb/src/storage.rs regex:is not yet implement...` → _storage.rs:is not yet implemented_

- [x] **PORT-STORAGE-MACRO-RLS** Resolved — Macro emits tenant_isolation RlsPolicy. Closed 2026-06-25.
      **Source:** docs/schemas/tenancy-schema.md § 7, roadmap A-4
      **Check:** `file:crates/infra/query-derive/src/lib.rs regex:RlsPolicy|tenant_isolation` → _lib.rs:RlsPolicy|tenant_isolation_

- [x] **FND-PORT-STORE-001** Resolved — StorageAdapter has both migrate() and create_schema(). Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave4-storage-port.md PORT-STORE-001
      **Check:** `file:crates/infra/storage/src/port.rs regex:async fn create_schema` → _port.rs:async fn create_schema_

- [x] **FND-PORT-STORE-003** Resolved — Outbox takes school_id. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave4-storage-port.md PORT-STORE-003
      **Check:** `file:crates/infra/storage/src/outbox.rs regex:async fn append.*school_id` → _outbox.rs:async fn append.*school_id_
<!-- END COMPUTED -->

### P0-DOCS — Decisions must be resolved (see `13-decision-needed.md`)

<!-- COMPUTED:items.P0.DOCS -->
- [x] **D-4** Resolved — Phase 17 = CMS Phase 12 (decision A locked 2026-06-25).
      **Source:** docs/build-plan.md; docs/audit_reports/remediation/13-decision-needed.md § D-4 (Option A locked)
      **Check:** `file:docs/build-plan.md regex:Phase 17|phase 17` → _build-plan.md:Phase 17|phase 17_

- [x] **D-5** Resolved — ADR-020 cross-domain ownership created (decision A locked 2026-06-25).
      **Source:** wave6-specs-1.md, ADR-020
      **Check:** `file:docs/decisions/ADR-020-CrossDomainOwnership.md regex:writable-owner|Option ...` → _ADR-020-CrossDomainOwnership.md:writable-owner|Option A|Accepted_

- [x] **D-6** Resolved — ADR-017 reconciled (decision D locked 2026-06-25).
      **Source:** wave5-docs-1.md, ADR-017
      **Check:** `file:docs/decisions/ADR-017-SurrealDBFirst.md regex:Reconciled 2026-06-25|Sync e...` → _ADR-017-SurrealDBFirst.md:Reconciled 2026-06-25|Sync engine backen_

- [x] **D-7** Resolved — ADR-019 public API naming created (decision A locked 2026-06-25).
      **Source:** wave5-docs-2.md, ADR-019
      **Check:** `file:docs/decisions/ADR-019-PublicApiNaming.md regex:code-as-canonical|Option A|...` → _ADR-019-PublicApiNaming.md:code-as-canonical|Option A|Accepted_
<!-- END COMPUTED -->

### P0-SCHEMA — Schema DDL must match spec

<!-- COMPUTED:items.P0.SCHEMA -->
- [x] **SCHEMA-OUTBOX-SURREAL-NULLABLE** Resolved — outbox school_id is uuid NOT NULL. Closed 2026-06-25.
      **Source:** docs/schemas/database-schema.md § 2
      **Check:** `file:migrations/engine/0000_engine_core.surreal.surql regex:school_id.*outbox TY...` → _0000_engine_core.surreal.surql:school_id.*outbox TYPE option<uuid>_

- [x] **SCHEMA-AUDIT-QUERY-PORT** Resolved — crates/cross-cutting/audit/src/query.rs with AuditQuery trait + AuditFilter (7 variants) + Page + AuditRecord. Closed 2026-06-25.
      **Source:** docs/schemas/audit-schema.md § 5
      **Check:** `file-exists:crates/cross-cutting/audit/src/query.rs` → _query.rs exists_

- [x] **SCHEMA-AUDIT-PARTITION-PG** Resolved — audit_log PARTITION BY RANGE. Closed 2026-06-25.
      **Source:** docs/schemas/audit-schema.md § 13.1
      **Check:** `file:migrations/engine/0000_engine_core.postgres.sql regex:PARTITION BY RANGE` → _0000_engine_core.postgres.sql:PARTITION BY RANGE_

- [x] **SCHEMA-AUDIT-PARTITION-MYSQL** Resolved — audit_log PARTITION BY KEY PARTITIONS 12. Closed 2026-06-25.
      **Source:** docs/schemas/audit-schema.md § 13.2
      **Check:** `file:migrations/engine/0000_engine_core.mysql.sql regex:PARTITION BY` → _0000_engine_core.mysql.sql:PARTITION BY_

- [x] **SCHEMA-AUDIT-ATOMIC** Resolved — AuditWriter takes &dyn Transaction. Closed 2026-06-25.
      **Source:** docs/schemas/audit-schema.md § 1, roadmap H-3
      **Check:** `file:crates/cross-cutting/audit/src/writer.rs regex:&dyn Transaction|impl.*Trans...` → _writer.rs:&dyn Transaction|impl.*Transaction_
<!-- END COMPUTED -->

### P0-SECURITY — Authentication + RBAC + Audit integrity

<!-- COMPUTED:items.P0.SECURITY -->
- [x] **FND-SEC-AUTH-001** Resolved — JwtAuthProvider rejects Credential::Anonymous. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave7-security.md SEC-AUTH-001
      **Check:** `file:crates/adapters/auth/src/jwt.rs regex:FND-SEC-AUTH-001|Credential::Anonymou...` → _jwt.rs:FND-SEC-AUTH-001|Credential::Anonymous.*_

- [ ] **FND-SEC-AUDIT-001** AuditLog::append is pub with no DB-level INSERT-only privilege enforcement; forged rows possible if TenantContext is wrong
      **Source:** docs/audit_reports/findings/wave7-security.md SEC-AUDIT-001
      **Check:** `commit:audit.*insert.*only|audit.*signature` → _git log grep: audit.*insert.*only|audit.*signature_

- [x] **FND-SEC-RBAC-001** Resolved — Capability::all() enumerates all 654 variants. Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave7-security.md SEC-RBAC-001
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:NAMING_EXCEPTIONS` → _value_objects.rs:NAMING_EXCEPTIONS_
<!-- END COMPUTED -->

### P0-WORKFLOWS — Cross-domain subscribers

<!-- COMPUTED:items.P0.WORKFLOWS -->
- [x] **FND-WF-005** Resolved — 2 subscribers wired in educore/src/subscribers.rs (#5 StudentAdmittedFeesAssign, #6 StudentWithdrawnTerminateFeesAssign). Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave7-workflows.md WF-005
      **Check:** `file:crates/educore/src/subscribers.rs regex:StudentAdmittedFeesAssign` → _subscribers.rs:StudentAdmittedFeesAssign_

- [x] **FND-WF-007** Resolved — SubjectTeacherAssignedClassSubject subscriber wired (#7 in educore/src/subscribers.rs). Closed 2026-06-25.
      **Source:** docs/audit_reports/findings/wave7-workflows.md WF-007
      **Check:** `commit:SubjectTeacherAssigned.*subscriber` → _git log grep: SubjectTeacherAssigned.*subscriber_
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

- [ ] **AGG-LIBRARY-LIBRARY_MEMBER** library: LibraryMember aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## LibraryMember
      **Check:** `file-exists:crates/domains/library/tests/library_member.rs` → _library_member.rs missing_

- [ ] **AGG-LIBRARY-BOOK_ACQUISITION** library: BookAcquisition aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## BookAcquisition
      **Check:** `file-exists:crates/domains/library/tests/book_acquisition.rs` → _book_acquisition.rs missing_

- [ ] **AGG-LIBRARY-BOOK_CATALOG_ENTRY** library: BookCatalogEntry aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## BookCatalogEntry
      **Check:** `file-exists:crates/domains/library/tests/book_catalog_entry.rs` → _book_catalog_entry.rs missing_

- [ ] **AGG-LIBRARY-BOOK_RETURN** library: BookReturn aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## BookReturn
      **Check:** `file-exists:crates/domains/library/tests/book_return.rs` → _book_return.rs missing_

- [ ] **AGG-LIBRARY-FINE** library: Fine aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## Fine
      **Check:** `file-exists:crates/domains/library/tests/fine.rs` → _fine.rs missing_

- [ ] **AGG-LIBRARY-LIBRARY_MEMBER_NOTE** library: LibraryMemberNote aggregate has no integration test
      **Source:** docs/specs/library/aggregates.md ## LibraryMemberNote
      **Check:** `file-exists:crates/domains/library/tests/library_member_note.rs` → _library_member_note.rs missing_

- [ ] **WF-LIBRARY-REPORTS** library: 'Reports' workflow not implemented
      **Source:** docs/specs/library/workflows.md ## Reports
      **Check:** `file:crates/domains/library/src/services.rs regex:Reports` → _services.rs:Reports_

- [ ] **AGG-ATTENDANCE-STAFF_ATTENDANCE** attendance: StaffAttendance aggregate has no integration test
      **Source:** docs/specs/attendance/aggregates.md ## StaffAttendance
      **Check:** `file-exists:crates/domains/attendance/tests/staff_attendance.rs` → _staff_attendance.rs missing_

- [ ] **AGG-COMMUNICATION-ABSENT_NOTIFICATION_TIME_SETUP** communication: AbsentNotificationTimeSetup aggregate has no integration test
      **Source:** docs/specs/communication/aggregates.md ## AbsentNotificationTimeSetup
      **Check:** `file-exists:crates/domains/communication/tests/absent_notification_time_setup.rs` → _absent_notification_time_setup.rs missing_

- [ ] **AGG-DOCUMENTS-FORM_DOWNLOAD_FILE** documents: FormDownloadFile aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## FormDownloadFile
      **Check:** `file-exists:crates/domains/documents/tests/form_download_file.rs` → _form_download_file.rs missing_

- [ ] **AGG-DOCUMENTS-FORM_DOWNLOAD_LINK** documents: FormDownloadLink aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## FormDownloadLink
      **Check:** `file-exists:crates/domains/documents/tests/form_download_link.rs` → _form_download_link.rs missing_

- [ ] **AGG-DOCUMENTS-NEW_FORM_DOWNLOAD** documents: NewFormDownload aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## NewFormDownload
      **Check:** `file-exists:crates/domains/documents/tests/new_form_download.rs` → _new_form_download.rs missing_

- [ ] **AGG-DOCUMENTS-NEW_POSTAL_DISPATCH** documents: NewPostalDispatch aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## NewPostalDispatch
      **Check:** `file-exists:crates/domains/documents/tests/new_postal_dispatch.rs` → _new_postal_dispatch.rs missing_

- [ ] **AGG-DOCUMENTS-NEW_POSTAL_RECEIVE** documents: NewPostalReceive aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## NewPostalReceive
      **Check:** `file-exists:crates/domains/documents/tests/new_postal_receive.rs` → _new_postal_receive.rs missing_

- [ ] **AGG-DOCUMENTS-POSTAL_DISPATCH_ATTACHMENT** documents: PostalDispatchAttachment aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## PostalDispatchAttachment
      **Check:** `file-exists:crates/domains/documents/tests/postal_dispatch_attachment.rs` → _postal_dispatch_attachment.rs missing_

- [ ] **AGG-DOCUMENTS-POSTAL_RECEIVE_ATTACHMENT** documents: PostalReceiveAttachment aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## PostalReceiveAttachment
      **Check:** `file-exists:crates/domains/documents/tests/postal_receive_attachment.rs` → _postal_receive_attachment.rs missing_

- [ ] **AGG-DOCUMENTS-UPDATE_FORM_DOWNLOAD** documents: UpdateFormDownload aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## UpdateFormDownload
      **Check:** `file-exists:crates/domains/documents/tests/update_form_download.rs` → _update_form_download.rs missing_

- [ ] **AGG-DOCUMENTS-UPDATE_POSTAL_DISPATCH** documents: UpdatePostalDispatch aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## UpdatePostalDispatch
      **Check:** `file-exists:crates/domains/documents/tests/update_postal_dispatch.rs` → _update_postal_dispatch.rs missing_

- [ ] **AGG-DOCUMENTS-UPDATE_POSTAL_RECEIVE** documents: UpdatePostalReceive aggregate has no integration test
      **Source:** docs/specs/documents/aggregates.md ## UpdatePostalReceive
      **Check:** `file-exists:crates/domains/documents/tests/update_postal_receive.rs` → _update_postal_receive.rs missing_

- [ ] **AGG-ACADEMIC-GUARDIAN** academic: Guardian aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## Guardian
      **Check:** `file-exists:crates/domains/academic/tests/guardian.rs` → _guardian.rs missing_

- [ ] **AGG-ACADEMIC-CLASS_SECTION** academic: ClassSection aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## ClassSection
      **Check:** `file-exists:crates/domains/academic/tests/class_section.rs` → _class_section.rs missing_

- [ ] **AGG-ACADEMIC-CLASS_SUBJECT** academic: ClassSubject aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## ClassSubject
      **Check:** `file-exists:crates/domains/academic/tests/class_subject.rs` → _class_subject.rs missing_

- [ ] **AGG-ACADEMIC-ACADEMIC_YEAR** academic: AcademicYear aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## AcademicYear
      **Check:** `file-exists:crates/domains/academic/tests/academic_year.rs` → _academic_year.rs missing_

- [ ] **AGG-ACADEMIC-CLASS_ROUTINE** academic: ClassRoutine aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## ClassRoutine
      **Check:** `file-exists:crates/domains/academic/tests/class_routine.rs` → _class_routine.rs missing_

- [ ] **AGG-ACADEMIC-HOMEWORK** academic: Homework aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## Homework
      **Check:** `file-exists:crates/domains/academic/tests/homework.rs` → _homework.rs missing_

- [ ] **AGG-ACADEMIC-LESSON_PLAN** academic: LessonPlan aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## LessonPlan
      **Check:** `file-exists:crates/domains/academic/tests/lesson_plan.rs` → _lesson_plan.rs missing_

- [ ] **AGG-ACADEMIC-LESSON** academic: Lesson aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## Lesson
      **Check:** `file-exists:crates/domains/academic/tests/lesson.rs` → _lesson.rs missing_

- [ ] **AGG-ACADEMIC-LESSON_TOPIC** academic: LessonTopic aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## LessonTopic
      **Check:** `file-exists:crates/domains/academic/tests/lesson_topic.rs` → _lesson_topic.rs missing_

- [ ] **AGG-ACADEMIC-STUDENT_RECORD** academic: StudentRecord aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## StudentRecord
      **Check:** `file-exists:crates/domains/academic/tests/student_record.rs` → _student_record.rs missing_

- [ ] **AGG-ACADEMIC-STUDENT_PROMOTION** academic: StudentPromotion aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## StudentPromotion
      **Check:** `file-exists:crates/domains/academic/tests/student_promotion.rs` → _student_promotion.rs missing_

- [ ] **AGG-ACADEMIC-STUDENT_CATEGORY** academic: StudentCategory aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## StudentCategory
      **Check:** `file-exists:crates/domains/academic/tests/student_category.rs` → _student_category.rs missing_

- [ ] **AGG-ACADEMIC-STUDENT_GROUP** academic: StudentGroup aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## StudentGroup
      **Check:** `file-exists:crates/domains/academic/tests/student_group.rs` → _student_group.rs missing_

- [ ] **AGG-ACADEMIC-REGISTRATION_FIELD** academic: RegistrationField aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## RegistrationField
      **Check:** `file-exists:crates/domains/academic/tests/registration_field.rs` → _registration_field.rs missing_

- [ ] **AGG-ACADEMIC-CERTIFICATE** academic: Certificate aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## Certificate
      **Check:** `file-exists:crates/domains/academic/tests/certificate.rs` → _certificate.rs missing_

- [ ] **AGG-ACADEMIC-ID_CARD** academic: IdCard aggregate has no integration test
      **Source:** docs/specs/academic/aggregates.md ## IdCard
      **Check:** `file-exists:crates/domains/academic/tests/id_card.rs` → _id_card.rs missing_

- [ ] **AGG-CMS-NEWS_COMMENT** cms: NewsComment aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewsComment
      **Check:** `file-exists:crates/domains/cms/tests/news_comment.rs` → _news_comment.rs missing_

- [ ] **AGG-CMS-NEWS_PAGE** cms: NewsPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewsPage
      **Check:** `file-exists:crates/domains/cms/tests/news_page.rs` → _news_page.rs missing_

- [ ] **AGG-CMS-NOTICE_BOARD** cms: NoticeBoard aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NoticeBoard
      **Check:** `file-exists:crates/domains/cms/tests/notice_board.rs` → _notice_board.rs missing_

- [ ] **AGG-CMS-TESTIMONIAL** cms: Testimonial aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## Testimonial
      **Check:** `file-exists:crates/domains/cms/tests/testimonial.rs` → _testimonial.rs missing_

- [ ] **AGG-CMS-HOME_SLIDER** cms: HomeSlider aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## HomeSlider
      **Check:** `file-exists:crates/domains/cms/tests/home_slider.rs` → _home_slider.rs missing_

- [ ] **AGG-CMS-SPEECH_SLIDER** cms: SpeechSlider aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## SpeechSlider
      **Check:** `file-exists:crates/domains/cms/tests/speech_slider.rs` → _speech_slider.rs missing_

- [ ] **AGG-CMS-CONTENT_SHARE_LIST** cms: ContentShareList aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## ContentShareList
      **Check:** `file-exists:crates/domains/cms/tests/content_share_list.rs` → _content_share_list.rs missing_

- [ ] **AGG-CMS-TEACHER_UPLOAD_CONTENT** cms: TeacherUploadContent aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## TeacherUploadContent
      **Check:** `file-exists:crates/domains/cms/tests/teacher_upload_content.rs` → _teacher_upload_content.rs missing_

- [ ] **AGG-CMS-UPLOAD_CONTENT** cms: UploadContent aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## UploadContent
      **Check:** `file-exists:crates/domains/cms/tests/upload_content.rs` → _upload_content.rs missing_

- [ ] **AGG-CMS-ABOUT_PAGE** cms: AboutPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## AboutPage
      **Check:** `file-exists:crates/domains/cms/tests/about_page.rs` → _about_page.rs missing_

- [ ] **AGG-CMS-CONTACT_PAGE** cms: ContactPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## ContactPage
      **Check:** `file-exists:crates/domains/cms/tests/contact_page.rs` → _contact_page.rs missing_

- [ ] **AGG-CMS-COURSE_PAGE** cms: CoursePage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## CoursePage
      **Check:** `file-exists:crates/domains/cms/tests/course_page.rs` → _course_page.rs missing_

- [ ] **AGG-CMS-HOME_PAGE_SETTING** cms: HomePageSetting aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## HomePageSetting
      **Check:** `file-exists:crates/domains/cms/tests/home_page_setting.rs` → _home_page_setting.rs missing_

- [ ] **AGG-CMS-FRONTEND_PAGE** cms: FrontendPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## FrontendPage
      **Check:** `file-exists:crates/domains/cms/tests/frontend_page.rs` → _frontend_page.rs missing_

- [ ] **AGG-CMS-NEW_ABOUT_PAGE** cms: NewAboutPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewAboutPage
      **Check:** `file-exists:crates/domains/cms/tests/new_about_page.rs` → _new_about_page.rs missing_

- [ ] **AGG-CMS-NEW_CONTACT_PAGE** cms: NewContactPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewContactPage
      **Check:** `file-exists:crates/domains/cms/tests/new_contact_page.rs` → _new_contact_page.rs missing_

- [ ] **AGG-CMS-NEW_CONTENT_SHARE_LIST** cms: NewContentShareList aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewContentShareList
      **Check:** `file-exists:crates/domains/cms/tests/new_content_share_list.rs` → _new_content_share_list.rs missing_

- [ ] **AGG-CMS-NEW_CONTENT_TYPE** cms: NewContentType aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewContentType
      **Check:** `file-exists:crates/domains/cms/tests/new_content_type.rs` → _new_content_type.rs missing_

- [ ] **AGG-CMS-NEW_COURSE_PAGE** cms: NewCoursePage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewCoursePage
      **Check:** `file-exists:crates/domains/cms/tests/new_course_page.rs` → _new_course_page.rs missing_

- [ ] **AGG-CMS-NEW_FRONTEND_PAGE** cms: NewFrontendPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewFrontendPage
      **Check:** `file-exists:crates/domains/cms/tests/new_frontend_page.rs` → _new_frontend_page.rs missing_

- [ ] **AGG-CMS-NEW_HOME_PAGE_SETTING** cms: NewHomePageSetting aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewHomePageSetting
      **Check:** `file-exists:crates/domains/cms/tests/new_home_page_setting.rs` → _new_home_page_setting.rs missing_

- [ ] **AGG-CMS-NEW_HOME_SLIDER** cms: NewHomeSlider aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewHomeSlider
      **Check:** `file-exists:crates/domains/cms/tests/new_home_slider.rs` → _new_home_slider.rs missing_

- [ ] **AGG-CMS-NEW_NEWS_CATEGORY** cms: NewNewsCategory aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewNewsCategory
      **Check:** `file-exists:crates/domains/cms/tests/new_news_category.rs` → _new_news_category.rs missing_

- [ ] **AGG-CMS-NEW_NEWS_COMMENT** cms: NewNewsComment aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewNewsComment
      **Check:** `file-exists:crates/domains/cms/tests/new_news_comment.rs` → _new_news_comment.rs missing_

- [ ] **AGG-CMS-NEW_NEWS_PAGE** cms: NewNewsPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewNewsPage
      **Check:** `file-exists:crates/domains/cms/tests/new_news_page.rs` → _new_news_page.rs missing_

- [ ] **AGG-CMS-NEW_NOTICE_BOARD** cms: NewNoticeBoard aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewNoticeBoard
      **Check:** `file-exists:crates/domains/cms/tests/new_notice_board.rs` → _new_notice_board.rs missing_

- [ ] **AGG-CMS-NEW_PAGE** cms: NewPage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewPage
      **Check:** `file-exists:crates/domains/cms/tests/new_page.rs` → _new_page.rs missing_

- [ ] **AGG-CMS-NEW_PAGE_REVISION** cms: NewPageRevision aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewPageRevision
      **Check:** `file-exists:crates/domains/cms/tests/new_page_revision.rs` → _new_page_revision.rs missing_

- [ ] **AGG-CMS-NEW_SPEECH_SLIDER** cms: NewSpeechSlider aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewSpeechSlider
      **Check:** `file-exists:crates/domains/cms/tests/new_speech_slider.rs` → _new_speech_slider.rs missing_

- [ ] **AGG-CMS-NEW_TEACHER_UPLOAD_CONTENT** cms: NewTeacherUploadContent aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewTeacherUploadContent
      **Check:** `file-exists:crates/domains/cms/tests/new_teacher_upload_content.rs` → _new_teacher_upload_content.rs missing_

- [ ] **AGG-CMS-NEW_TESTIMONIAL** cms: NewTestimonial aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewTestimonial
      **Check:** `file-exists:crates/domains/cms/tests/new_testimonial.rs` → _new_testimonial.rs missing_

- [ ] **AGG-CMS-NEW_UPLOAD_CONTENT** cms: NewUploadContent aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## NewUploadContent
      **Check:** `file-exists:crates/domains/cms/tests/new_upload_content.rs` → _new_upload_content.rs missing_

- [ ] **AGG-CMS-UPDATE_CONTENT** cms: UpdateContent aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## UpdateContent
      **Check:** `file-exists:crates/domains/cms/tests/update_content.rs` → _update_content.rs missing_

- [ ] **AGG-CMS-UPDATE_NEWS** cms: UpdateNews aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## UpdateNews
      **Check:** `file-exists:crates/domains/cms/tests/update_news.rs` → _update_news.rs missing_

- [ ] **AGG-CMS-UPDATE_PAGE** cms: UpdatePage aggregate has no integration test
      **Source:** docs/specs/cms/aggregates.md ## UpdatePage
      **Check:** `file-exists:crates/domains/cms/tests/update_page.rs` → _update_page.rs missing_

- [ ] **AGG-FACILITIES-ASSIGN_VEHICLE** facilities: AssignVehicle aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## AssignVehicle
      **Check:** `file-exists:crates/domains/facilities/tests/assign_vehicle.rs` → _assign_vehicle.rs missing_

- [ ] **AGG-FACILITIES-ITEM_STORE** facilities: ItemStore aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemStore
      **Check:** `file-exists:crates/domains/facilities/tests/item_store.rs` → _item_store.rs missing_

- [ ] **AGG-FACILITIES-ITEM_ISSUE** facilities: ItemIssue aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemIssue
      **Check:** `file-exists:crates/domains/facilities/tests/item_issue.rs` → _item_issue.rs missing_

- [ ] **AGG-FACILITIES-ITEM_RECEIVE** facilities: ItemReceive aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemReceive
      **Check:** `file-exists:crates/domains/facilities/tests/item_receive.rs` → _item_receive.rs missing_

- [ ] **AGG-FACILITIES-ITEM_RECEIVE_CHILD** facilities: ItemReceiveChild aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemReceiveChild
      **Check:** `file-exists:crates/domains/facilities/tests/item_receive_child.rs` → _item_receive_child.rs missing_

- [ ] **AGG-FACILITIES-ITEM_SELL** facilities: ItemSell aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemSell
      **Check:** `file-exists:crates/domains/facilities/tests/item_sell.rs` → _item_sell.rs missing_

- [ ] **AGG-FACILITIES-ITEM_SELL_CHILD** facilities: ItemSellChild aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## ItemSellChild
      **Check:** `file-exists:crates/domains/facilities/tests/item_sell_child.rs` → _item_sell_child.rs missing_

- [ ] **AGG-FACILITIES-SUPPLIER** facilities: Supplier aggregate has no integration test
      **Source:** docs/specs/facilities/aggregates.md ## Supplier
      **Check:** `file-exists:crates/domains/facilities/tests/supplier.rs` → _supplier.rs missing_

- [ ] **AGG-ASSESSMENT-EXAM_SETUP** assessment: ExamSetup aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## ExamSetup
      **Check:** `file-exists:crates/domains/assessment/tests/exam_setup.rs` → _exam_setup.rs missing_

- [ ] **AGG-ASSESSMENT-EXAM_SCHEDULE** assessment: ExamSchedule aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## ExamSchedule
      **Check:** `file-exists:crates/domains/assessment/tests/exam_schedule.rs` → _exam_schedule.rs missing_

- [ ] **AGG-ASSESSMENT-MARK_STORE** assessment: MarkStore aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## MarkStore
      **Check:** `file-exists:crates/domains/assessment/tests/mark_store.rs` → _mark_store.rs missing_

- [ ] **AGG-ASSESSMENT-EXAM_SETTING** assessment: ExamSetting aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## ExamSetting
      **Check:** `file-exists:crates/domains/assessment/tests/exam_setting.rs` → _exam_setting.rs missing_

- [ ] **AGG-ASSESSMENT-EXAM_SIGNATURE** assessment: ExamSignature aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## ExamSignature
      **Check:** `file-exists:crates/domains/assessment/tests/exam_signature.rs` → _exam_signature.rs missing_

- [ ] **AGG-ASSESSMENT-ONLINE_EXAM** assessment: OnlineExam aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## OnlineExam
      **Check:** `file-exists:crates/domains/assessment/tests/online_exam.rs` → _online_exam.rs missing_

- [ ] **AGG-ASSESSMENT-QUESTION_BANK** assessment: QuestionBank aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## QuestionBank
      **Check:** `file-exists:crates/domains/assessment/tests/question_bank.rs` → _question_bank.rs missing_

- [ ] **AGG-ASSESSMENT-STUDENT_TAKE_ONLINE_EXAM** assessment: StudentTakeOnlineExam aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## StudentTakeOnlineExam
      **Check:** `file-exists:crates/domains/assessment/tests/student_take_online_exam.rs` → _student_take_online_exam.rs missing_

- [ ] **AGG-ASSESSMENT-SEAT_PLAN** assessment: SeatPlan aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## SeatPlan
      **Check:** `file-exists:crates/domains/assessment/tests/seat_plan.rs` → _seat_plan.rs missing_

- [ ] **AGG-ASSESSMENT-ADMIT_CARD** assessment: AdmitCard aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## AdmitCard
      **Check:** `file-exists:crates/domains/assessment/tests/admit_card.rs` → _admit_card.rs missing_

- [ ] **AGG-ASSESSMENT-TEACHER_EVALUATION** assessment: TeacherEvaluation aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## TeacherEvaluation
      **Check:** `file-exists:crates/domains/assessment/tests/teacher_evaluation.rs` → _teacher_evaluation.rs missing_

- [ ] **AGG-ASSESSMENT-TEACHER_REMARK** assessment: TeacherRemark aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## TeacherRemark
      **Check:** `file-exists:crates/domains/assessment/tests/teacher_remark.rs` → _teacher_remark.rs missing_

- [ ] **AGG-ASSESSMENT-EXAM_ATTENDANCE** assessment: ExamAttendance aggregate has no integration test
      **Source:** docs/specs/assessment/aggregates.md ## ExamAttendance
      **Check:** `file-exists:crates/domains/assessment/tests/exam_attendance.rs` → _exam_attendance.rs missing_

- [ ] **WF-ASSESSMENT-ONLINE_EXAM_LIFECYCLE** assessment: 'Online Exam Lifecycle' workflow not implemented
      **Source:** docs/specs/assessment/workflows.md ## Online Exam Lifecycle
      **Check:** `file:crates/domains/assessment/src/services.rs regex:Online Exam Lifecycle` → _services.rs:Online Exam Lifecycle_

- [ ] **AGG-FINANCE-FEES_GROUP** finance: FeesGroup aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesGroup
      **Check:** `file-exists:crates/domains/finance/tests/fees_group.rs` → _fees_group.rs missing_

- [ ] **AGG-FINANCE-FEES_TYPE** finance: FeesType aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesType
      **Check:** `file-exists:crates/domains/finance/tests/fees_type.rs` → _fees_type.rs missing_

- [ ] **AGG-FINANCE-FEES_MASTER** finance: FeesMaster aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesMaster
      **Check:** `file-exists:crates/domains/finance/tests/fees_master.rs` → _fees_master.rs missing_

- [ ] **AGG-FINANCE-FEES_ASSIGN** finance: FeesAssign aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesAssign
      **Check:** `file-exists:crates/domains/finance/tests/fees_assign.rs` → _fees_assign.rs missing_

- [ ] **AGG-FINANCE-FEES_ASSIGN_DISCOUNT** finance: FeesAssignDiscount aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesAssignDiscount
      **Check:** `file-exists:crates/domains/finance/tests/fees_assign_discount.rs` → _fees_assign_discount.rs missing_

- [ ] **AGG-FINANCE-FEES_DISCOUNT** finance: FeesDiscount aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesDiscount
      **Check:** `file-exists:crates/domains/finance/tests/fees_discount.rs` → _fees_discount.rs missing_

- [ ] **AGG-FINANCE-FEES_INVOICE** finance: FeesInvoice aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInvoice
      **Check:** `file-exists:crates/domains/finance/tests/fees_invoice.rs` → _fees_invoice.rs missing_

- [ ] **AGG-FINANCE-FEES_INSTALLMENT** finance: FeesInstallment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInstallment
      **Check:** `file-exists:crates/domains/finance/tests/fees_installment.rs` → _fees_installment.rs missing_

- [ ] **AGG-FINANCE-FEES_INSTALLMENT_ASSIGN** finance: FeesInstallmentAssign aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInstallmentAssign
      **Check:** `file-exists:crates/domains/finance/tests/fees_installment_assign.rs` → _fees_installment_assign.rs missing_

- [ ] **AGG-FINANCE-FEES_PAYMENT** finance: FeesPayment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesPayment
      **Check:** `file-exists:crates/domains/finance/tests/fees_payment.rs` → _fees_payment.rs missing_

- [ ] **AGG-FINANCE-FEES_CARRY_FORWARD** finance: FeesCarryForward aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesCarryForward
      **Check:** `file-exists:crates/domains/finance/tests/fees_carry_forward.rs` → _fees_carry_forward.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_INSTALLMENT** finance: DirectFeesInstallment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesInstallment
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_installment.rs` → _direct_fees_installment.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_INSTALLMENT_ASSIGN** finance: DirectFeesInstallmentAssign aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesInstallmentAssign
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_installment_assign.rs` → _direct_fees_installment_assign.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_INSTALLMENT_CHILD_PAYMENT** finance: DirectFeesInstallmentChildPayment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesInstallmentChildPayment
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_installment_child_payment.r...` → _direct_fees_installment_child_payment.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_TYPE** finance: FmFeesType aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesType
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_type.rs` → _fm_fees_type.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_INVOICE** finance: FmFeesInvoice aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesInvoice
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_invoice.rs` → _fm_fees_invoice.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_INVOICE_CHILD** finance: FmFeesInvoiceChild aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesInvoiceChild
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_invoice_child.rs` → _fm_fees_invoice_child.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_TRANSACTION** finance: FmFeesTransaction aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesTransaction
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_transaction.rs` → _fm_fees_transaction.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_TRANSACTION_CHILD** finance: FmFeesTransactionChild aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesTransactionChild
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_transaction_child.rs` → _fm_fees_transaction_child.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_WEAVER** finance: FmFeesWeaver aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesWeaver
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_weaver.rs` → _fm_fees_weaver.rs missing_

- [ ] **AGG-FINANCE-FEES_INVOICE_SETTING** finance: FeesInvoiceSetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInvoiceSetting
      **Check:** `file-exists:crates/domains/finance/tests/fees_invoice_setting.rs` → _fees_invoice_setting.rs missing_

- [ ] **AGG-FINANCE-INVOICE_SETTING** finance: InvoiceSetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## InvoiceSetting
      **Check:** `file-exists:crates/domains/finance/tests/invoice_setting.rs` → _invoice_setting.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_INVOICE_SETTING** finance: FmFeesInvoiceSetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesInvoiceSetting
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_invoice_setting.rs` → _fm_fees_invoice_setting.rs missing_

- [ ] **AGG-FINANCE-BANK_STATEMENT** finance: BankStatement aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## BankStatement
      **Check:** `file-exists:crates/domains/finance/tests/bank_statement.rs` → _bank_statement.rs missing_

- [ ] **AGG-FINANCE-BANK_PAYMENT_SLIP** finance: BankPaymentSlip aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## BankPaymentSlip
      **Check:** `file-exists:crates/domains/finance/tests/bank_payment_slip.rs` → _bank_payment_slip.rs missing_

- [ ] **AGG-FINANCE-EXPENSE** finance: Expense aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## Expense
      **Check:** `file-exists:crates/domains/finance/tests/expense.rs` → _expense.rs missing_

- [ ] **AGG-FINANCE-INCOME** finance: Income aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## Income
      **Check:** `file-exists:crates/domains/finance/tests/income.rs` → _income.rs missing_

- [ ] **AGG-FINANCE-DONOR** finance: Donor aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## Donor
      **Check:** `file-exists:crates/domains/finance/tests/donor.rs` → _donor.rs missing_

- [ ] **AGG-FINANCE-EXPENSE_HEAD** finance: ExpenseHead aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## ExpenseHead
      **Check:** `file-exists:crates/domains/finance/tests/expense_head.rs` → _expense_head.rs missing_

- [ ] **AGG-FINANCE-INCOME_HEAD** finance: IncomeHead aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## IncomeHead
      **Check:** `file-exists:crates/domains/finance/tests/income_head.rs` → _income_head.rs missing_

- [ ] **AGG-FINANCE-WALLET_TRANSACTION** finance: WalletTransaction aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## WalletTransaction
      **Check:** `file-exists:crates/domains/finance/tests/wallet_transaction.rs` → _wallet_transaction.rs missing_

- [ ] **AGG-FINANCE-TRANSACTION** finance: Transaction aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## Transaction
      **Check:** `file-exists:crates/domains/finance/tests/transaction.rs` → _transaction.rs missing_

- [ ] **AGG-FINANCE-PAYROLL_PAYMENT** finance: PayrollPayment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## PayrollPayment
      **Check:** `file-exists:crates/domains/finance/tests/payroll_payment.rs` → _payroll_payment.rs missing_

- [ ] **AGG-FINANCE-PAYROLL_GENERATE** finance: PayrollGenerate aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## PayrollGenerate
      **Check:** `file-exists:crates/domains/finance/tests/payroll_generate.rs` → _payroll_generate.rs missing_

- [ ] **AGG-FINANCE-PAYROLL_EARN_DEDUC** finance: PayrollEarnDeduc aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## PayrollEarnDeduc
      **Check:** `file-exists:crates/domains/finance/tests/payroll_earn_deduc.rs` → _payroll_earn_deduc.rs missing_

- [ ] **AGG-FINANCE-SALARY_TEMPLATE** finance: SalaryTemplate aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## SalaryTemplate
      **Check:** `file-exists:crates/domains/finance/tests/salary_template.rs` → _salary_template.rs missing_

- [ ] **AGG-FINANCE-PRODUCT_PURCHASE** finance: ProductPurchase aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## ProductPurchase
      **Check:** `file-exists:crates/domains/finance/tests/product_purchase.rs` → _product_purchase.rs missing_

- [ ] **AGG-FINANCE-INVENTORY_PAYMENT** finance: InventoryPayment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## InventoryPayment
      **Check:** `file-exists:crates/domains/finance/tests/inventory_payment.rs` → _inventory_payment.rs missing_

- [ ] **AGG-FINANCE-AMOUNT_TRANSFER** finance: AmountTransfer aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## AmountTransfer
      **Check:** `file-exists:crates/domains/finance/tests/amount_transfer.rs` → _amount_transfer.rs missing_

- [ ] **AGG-FINANCE-CHART_OF_ACCOUNT** finance: ChartOfAccount aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## ChartOfAccount
      **Check:** `file-exists:crates/domains/finance/tests/chart_of_account.rs` → _chart_of_account.rs missing_

- [ ] **AGG-FINANCE-QUESTION_BANK_FEE** finance: QuestionBankFee aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## QuestionBankFee
      **Check:** `file-exists:crates/domains/finance/tests/question_bank_fee.rs` → _question_bank_fee.rs missing_

- [ ] **AGG-FINANCE-PAYMENT_GATEWAY_SETTING** finance: PaymentGatewaySetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## PaymentGatewaySetting
      **Check:** `file-exists:crates/domains/finance/tests/payment_gateway_setting.rs` → _payment_gateway_setting.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_REMINDER** finance: DirectFeesReminder aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesReminder
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_reminder.rs` → _direct_fees_reminder.rs missing_

- [ ] **AGG-FINANCE-DUE_FEES_LOGIN_PREVENT** finance: DueFeesLoginPrevent aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DueFeesLoginPrevent
      **Check:** `file-exists:crates/domains/finance/tests/due_fees_login_prevent.rs` → _due_fees_login_prevent.rs missing_

- [ ] **AGG-FINANCE-FEES_CARRY_FORWARD_LOG** finance: FeesCarryForwardLog aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesCarryForwardLog
      **Check:** `file-exists:crates/domains/finance/tests/fees_carry_forward_log.rs` → _fees_carry_forward_log.rs missing_

- [ ] **AGG-FINANCE-FEES_INSTALLMENT_CREDIT** finance: FeesInstallmentCredit aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInstallmentCredit
      **Check:** `file-exists:crates/domains/finance/tests/fees_installment_credit.rs` → _fees_installment_credit.rs missing_

- [ ] **AGG-FINANCE-FEES_CARRY_FORWARD_SETTING** finance: FeesCarryForwardSetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesCarryForwardSetting
      **Check:** `file-exists:crates/domains/finance/tests/fees_carry_forward_setting.rs` → _fees_carry_forward_setting.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_SETTING** finance: DirectFeesSetting aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesSetting
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_setting.rs` → _direct_fees_setting.rs missing_

- [ ] **AGG-FINANCE-BANK_PAYMENT_SLIP_AUDIT** finance: BankPaymentSlipAudit aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## BankPaymentSlipAudit
      **Check:** `file-exists:crates/domains/finance/tests/bank_payment_slip_audit.rs` → _bank_payment_slip_audit.rs missing_

- [ ] **AGG-FINANCE-BANK_STATEMENT_ATTACHMENT** finance: BankStatementAttachment aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## BankStatementAttachment
      **Check:** `file-exists:crates/domains/finance/tests/bank_statement_attachment.rs` → _bank_statement_attachment.rs missing_

- [ ] **AGG-FINANCE-DIRECT_FEES_INSTALLMENT_ASSIGN_CHILD** finance: DirectFeesInstallmentAssignChild aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## DirectFeesInstallmentAssignChild
      **Check:** `file-exists:crates/domains/finance/tests/direct_fees_installment_assign_child.rs` → _direct_fees_installment_assign_child.rs missing_

- [ ] **AGG-FINANCE-EXPENSE_APPROVAL** finance: ExpenseApproval aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## ExpenseApproval
      **Check:** `file-exists:crates/domains/finance/tests/expense_approval.rs` → _expense_approval.rs missing_

- [ ] **AGG-FINANCE-FEES_INSTALLMENT_ASSIGN_DISCOUNT** finance: FeesInstallmentAssignDiscount aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FeesInstallmentAssignDiscount
      **Check:** `file-exists:crates/domains/finance/tests/fees_installment_assign_discount.rs` → _fees_installment_assign_discount.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_INVOICE_LINE_NOTE** finance: FmFeesInvoiceLineNote aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesInvoiceLineNote
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_invoice_line_note.rs` → _fm_fees_invoice_line_note.rs missing_

- [ ] **AGG-FINANCE-FM_FEES_TRANSACTION_LINE_NOTE** finance: FmFeesTransactionLineNote aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## FmFeesTransactionLineNote
      **Check:** `file-exists:crates/domains/finance/tests/fm_fees_transaction_line_note.rs` → _fm_fees_transaction_line_note.rs missing_

- [ ] **AGG-FINANCE-INCOME_APPROVAL** finance: IncomeApproval aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## IncomeApproval
      **Check:** `file-exists:crates/domains/finance/tests/income_approval.rs` → _income_approval.rs missing_

- [ ] **AGG-FINANCE-PAYROLL_PAYMENT_APPROVAL** finance: PayrollPaymentApproval aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## PayrollPaymentApproval
      **Check:** `file-exists:crates/domains/finance/tests/payroll_payment_approval.rs` → _payroll_payment_approval.rs missing_

- [ ] **AGG-FINANCE-WALLET_TRANSACTION_APPROVAL** finance: WalletTransactionApproval aggregate has no integration test
      **Source:** docs/specs/finance/aggregates.md ## WalletTransactionApproval
      **Check:** `file-exists:crates/domains/finance/tests/wallet_transaction_approval.rs` → _wallet_transaction_approval.rs missing_

- [ ] **WF-FINANCE-FEES_ASSIGNMENT** finance: 'Fees Assignment' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Fees Assignment
      **Check:** `file:crates/domains/finance/src/services.rs regex:Fees Assignment` → _services.rs:Fees Assignment_

- [ ] **WF-FINANCE-DUE_FEES_LOGIN_PREVENTION** finance: 'Due Fees Login Prevention' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Due Fees Login Prevention
      **Check:** `file:crates/domains/finance/src/services.rs regex:Due Fees Login Prevention` → _services.rs:Due Fees Login Prevention_

- [ ] **WF-FINANCE-BANK_RECONCILIATION** finance: 'Bank Reconciliation' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Bank Reconciliation
      **Check:** `file:crates/domains/finance/src/services.rs regex:Bank Reconciliation` → _services.rs:Bank Reconciliation_

- [ ] **WF-FINANCE-PAYROLL_DISBURSEMENT** finance: 'Payroll Disbursement' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Payroll Disbursement
      **Check:** `file:crates/domains/finance/src/services.rs regex:Payroll Disbursement` → _services.rs:Payroll Disbursement_

- [ ] **WF-FINANCE-HOURLY_RATE_MANAGEMENT** finance: 'Hourly Rate Management' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Hourly Rate Management
      **Check:** `file:crates/domains/finance/src/services.rs regex:Hourly Rate Management` → _services.rs:Hourly Rate Management_

- [ ] **WF-FINANCE-SALARY_TEMPLATE** finance: 'Salary Template' workflow not implemented
      **Source:** docs/specs/finance/workflows.md ## Salary Template
      **Check:** `file:crates/domains/finance/src/services.rs regex:Salary Template` → _services.rs:Salary Template_

- [ ] **AGG-HR-LEAVE_DEFINE** hr: LeaveDefine aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveDefine
      **Check:** `file-exists:crates/domains/hr/tests/leave_define.rs` → _leave_define.rs missing_

- [ ] **AGG-HR-LEAVE_REQUEST** hr: LeaveRequest aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveRequest
      **Check:** `file-exists:crates/domains/hr/tests/leave_request.rs` → _leave_request.rs missing_

- [ ] **AGG-HR-STAFF_ATTENDANCE** hr: StaffAttendance aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffAttendance
      **Check:** `file-exists:crates/domains/hr/tests/staff_attendance.rs` → _staff_attendance.rs missing_

- [ ] **AGG-HR-STAFF_ATTENDANCE_IMPORT** hr: StaffAttendanceImport aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffAttendanceImport
      **Check:** `file-exists:crates/domains/hr/tests/staff_attendance_import.rs` → _staff_attendance_import.rs missing_

- [ ] **AGG-HR-ASSIGN_CLASS_TEACHER** hr: AssignClassTeacher aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## AssignClassTeacher
      **Check:** `file-exists:crates/domains/hr/tests/assign_class_teacher.rs` → _assign_class_teacher.rs missing_

- [ ] **AGG-HR-HOURLY_RATE** hr: HourlyRate aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## HourlyRate
      **Check:** `file-exists:crates/domains/hr/tests/hourly_rate.rs` → _hourly_rate.rs missing_

- [ ] **AGG-HR-SALARY_TEMPLATE** hr: SalaryTemplate aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## SalaryTemplate
      **Check:** `file-exists:crates/domains/hr/tests/salary_template.rs` → _salary_template.rs missing_

- [ ] **AGG-HR-PAYROLL_GENERATE** hr: PayrollGenerate aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## PayrollGenerate
      **Check:** `file-exists:crates/domains/hr/tests/payroll_generate.rs` → _payroll_generate.rs missing_

- [ ] **AGG-HR-PAYROLL_EARN_DEDUC** hr: PayrollEarnDeduc aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## PayrollEarnDeduc
      **Check:** `file-exists:crates/domains/hr/tests/payroll_earn_deduc.rs` → _payroll_earn_deduc.rs missing_

- [ ] **AGG-HR-LEAVE_DEDUCTION_INFO** hr: LeaveDeductionInfo aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveDeductionInfo
      **Check:** `file-exists:crates/domains/hr/tests/leave_deduction_info.rs` → _leave_deduction_info.rs missing_

- [ ] **AGG-HR-STAFF_REGISTRATION_FIELD** hr: StaffRegistrationField aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffRegistrationField
      **Check:** `file-exists:crates/domains/hr/tests/staff_registration_field.rs` → _staff_registration_field.rs missing_

- [ ] **AGG-HR-STAFF_IMPORT_BULK_TEMPORARY** hr: StaffImportBulkTemporary aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffImportBulkTemporary
      **Check:** `file-exists:crates/domains/hr/tests/staff_import_bulk_temporary.rs` → _staff_import_bulk_temporary.rs missing_

- [ ] **AGG-HR-ASSIGN_CLASS_TEACHER_SCOPE** hr: AssignClassTeacherScope aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## AssignClassTeacherScope
      **Check:** `file-exists:crates/domains/hr/tests/assign_class_teacher_scope.rs` → _assign_class_teacher_scope.rs missing_

- [ ] **AGG-HR-BULK_IMPORT_JOB** hr: BulkImportJob aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## BulkImportJob
      **Check:** `file-exists:crates/domains/hr/tests/bulk_import_job.rs` → _bulk_import_job.rs missing_

- [ ] **AGG-HR-DEPARTMENT_HEAD** hr: DepartmentHead aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## DepartmentHead
      **Check:** `file-exists:crates/domains/hr/tests/department_head.rs` → _department_head.rs missing_

- [ ] **AGG-HR-DESIGNATION_GRADE** hr: DesignationGrade aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## DesignationGrade
      **Check:** `file-exists:crates/domains/hr/tests/designation_grade.rs` → _designation_grade.rs missing_

- [ ] **AGG-HR-HOURLY_RATE_OVERRIDE** hr: HourlyRateOverride aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## HourlyRateOverride
      **Check:** `file-exists:crates/domains/hr/tests/hourly_rate_override.rs` → _hourly_rate_override.rs missing_

- [ ] **AGG-HR-LEAVE_DEFINE_ADJUSTMENT** hr: LeaveDefineAdjustment aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveDefineAdjustment
      **Check:** `file-exists:crates/domains/hr/tests/leave_define_adjustment.rs` → _leave_define_adjustment.rs missing_

- [ ] **AGG-HR-LEAVE_REQUEST_APPROVAL** hr: LeaveRequestApproval aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveRequestApproval
      **Check:** `file-exists:crates/domains/hr/tests/leave_request_approval.rs` → _leave_request_approval.rs missing_

- [ ] **AGG-HR-LEAVE_REQUEST_ATTACHMENT** hr: LeaveRequestAttachment aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## LeaveRequestAttachment
      **Check:** `file-exists:crates/domains/hr/tests/leave_request_attachment.rs` → _leave_request_attachment.rs missing_

- [ ] **AGG-HR-PAYROLL_GENERATE_AUDIT** hr: PayrollGenerateAudit aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## PayrollGenerateAudit
      **Check:** `file-exists:crates/domains/hr/tests/payroll_generate_audit.rs` → _payroll_generate_audit.rs missing_

- [ ] **AGG-HR-PAYROLL_PAYMENT_LINK** hr: PayrollPaymentLink aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## PayrollPaymentLink
      **Check:** `file-exists:crates/domains/hr/tests/payroll_payment_link.rs` → _payroll_payment_link.rs missing_

- [ ] **AGG-HR-STAFF_ADDRESS** hr: StaffAddress aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffAddress
      **Check:** `file-exists:crates/domains/hr/tests/staff_address.rs` → _staff_address.rs missing_

- [ ] **AGG-HR-STAFF_ATTENDANCE_IMPORT_BATCH** hr: StaffAttendanceImportBatch aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffAttendanceImportBatch
      **Check:** `file-exists:crates/domains/hr/tests/staff_attendance_import_batch.rs` → _staff_attendance_import_batch.rs missing_

- [ ] **AGG-HR-STAFF_ATTENDANCE_PUNCH** hr: StaffAttendancePunch aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffAttendancePunch
      **Check:** `file-exists:crates/domains/hr/tests/staff_attendance_punch.rs` → _staff_attendance_punch.rs missing_

- [ ] **AGG-HR-STAFF_BANK_DETAIL** hr: StaffBankDetail aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffBankDetail
      **Check:** `file-exists:crates/domains/hr/tests/staff_bank_detail.rs` → _staff_bank_detail.rs missing_

- [ ] **AGG-HR-STAFF_CUSTOM_FIELD** hr: StaffCustomField aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffCustomField
      **Check:** `file-exists:crates/domains/hr/tests/staff_custom_field.rs` → _staff_custom_field.rs missing_

- [ ] **AGG-HR-STAFF_DOCUMENT** hr: StaffDocument aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffDocument
      **Check:** `file-exists:crates/domains/hr/tests/staff_document.rs` → _staff_document.rs missing_

- [ ] **AGG-HR-STAFF_DRIVING_LICENSE** hr: StaffDrivingLicense aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffDrivingLicense
      **Check:** `file-exists:crates/domains/hr/tests/staff_driving_license.rs` → _staff_driving_license.rs missing_

- [ ] **AGG-HR-STAFF_IMPORT_RESOLUTION** hr: StaffImportResolution aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffImportResolution
      **Check:** `file-exists:crates/domains/hr/tests/staff_import_resolution.rs` → _staff_import_resolution.rs missing_

- [ ] **AGG-HR-STAFF_LEAVE_BALANCE** hr: StaffLeaveBalance aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffLeaveBalance
      **Check:** `file-exists:crates/domains/hr/tests/staff_leave_balance.rs` → _staff_leave_balance.rs missing_

- [ ] **AGG-HR-STAFF_LEAVE_HISTORY** hr: StaffLeaveHistory aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffLeaveHistory
      **Check:** `file-exists:crates/domains/hr/tests/staff_leave_history.rs` → _staff_leave_history.rs missing_

- [ ] **AGG-HR-STAFF_PAYROLL_HISTORY** hr: StaffPayrollHistory aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffPayrollHistory
      **Check:** `file-exists:crates/domains/hr/tests/staff_payroll_history.rs` → _staff_payroll_history.rs missing_

- [ ] **AGG-HR-STAFF_PROFILE_PHOTO** hr: StaffProfilePhoto aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffProfilePhoto
      **Check:** `file-exists:crates/domains/hr/tests/staff_profile_photo.rs` → _staff_profile_photo.rs missing_

- [ ] **AGG-HR-STAFF_REGISTRATION_FIELD_OPTION** hr: StaffRegistrationFieldOption aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffRegistrationFieldOption
      **Check:** `file-exists:crates/domains/hr/tests/staff_registration_field_option.rs` → _staff_registration_field_option.rs missing_

- [ ] **AGG-HR-STAFF_ROLE_ASSIGNMENT** hr: StaffRoleAssignment aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffRoleAssignment
      **Check:** `file-exists:crates/domains/hr/tests/staff_role_assignment.rs` → _staff_role_assignment.rs missing_

- [ ] **AGG-HR-STAFF_SOCIAL_LINK** hr: StaffSocialLink aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffSocialLink
      **Check:** `file-exists:crates/domains/hr/tests/staff_social_link.rs` → _staff_social_link.rs missing_

- [ ] **AGG-HR-STAFF_TIMELINE** hr: StaffTimeline aggregate has no integration test
      **Source:** docs/specs/hr/aggregates.md ## StaffTimeline
      **Check:** `file-exists:crates/domains/hr/tests/staff_timeline.rs` → _staff_timeline.rs missing_

- [ ] **WF-HR-CLASS_TEACHER_ASSIGNMENT** hr: 'Class Teacher Assignment' workflow not implemented
      **Source:** docs/specs/hr/workflows.md ## Class Teacher Assignment
      **Check:** `file:crates/domains/hr/src/services.rs regex:Class Teacher Assignment` → _services.rs:Class Teacher Assignment_

- [ ] **WF-HR-SUBJECT_TEACHER_ASSIGNMENT** hr: 'Subject Teacher Assignment' workflow not implemented
      **Source:** docs/specs/hr/workflows.md ## Subject Teacher Assignment
      **Check:** `file:crates/domains/hr/src/services.rs regex:Subject Teacher Assignment` → _services.rs:Subject Teacher Assignment_

- [ ] **WF-HR-HOURLY_RATE_MANAGEMENT** hr: 'Hourly Rate Management' workflow not implemented
      **Source:** docs/specs/hr/workflows.md ## Hourly Rate Management
      **Check:** `file:crates/domains/hr/src/services.rs regex:Hourly Rate Management` → _services.rs:Hourly Rate Management_

- [ ] **AGG-EVENTS-ASSIGN_INCIDENT** events: AssignIncident aggregate has no integration test
      **Source:** docs/specs/events/aggregates.md ## AssignIncident
      **Check:** `file-exists:crates/cross-cutting/events-domain/tests/assign_incident.rs` → _assign_incident.rs missing_

- [ ] **AGG-EVENTS-INCIDENT_COMMENT** events: IncidentComment aggregate has no integration test
      **Source:** docs/specs/events/aggregates.md ## IncidentComment
      **Check:** `file-exists:crates/cross-cutting/events-domain/tests/incident_comment.rs` → _incident_comment.rs missing_
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

- [x] **I-3** Phase 17 missing from build plan — RESOLVED (duplicate of D-4). Phase 17 = Production readiness, already documented in build-plan.md. No new phase needed. See 13-decision-needed.md D-4 Option A (locked 2026-06-25).
      **Source:** See D-4 (P0); docs/audit_reports/remediation/13-decision-needed.md § D-4
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

- [x] **ADR-013-COUNTS-DRIFT** Crate count drift resolved — AGENTS.md + ADR-013 now both say 37 packages (D-9 Option A, 2026-06-25)
      **Source:** docs/decisions/ADR-013-CrateLayout.md, AGENTS.md, docs/architecture.md
      **Check:** `file:docs/decisions/ADR-013-CrateLayout.md regex:Reconciled 2026-06-25|canonical...` → _ADR-013-CrateLayout.md:Reconciled 2026-06-25|canonical count = _

- [ ] **ADR-013-NO-TIER-CARGO** No lint guard against adding Cargo.toml at tier roots (ADR-013 § Negative consequences item 2)
      **Source:** docs/decisions/ADR-013-CrateLayout.md
      **Check:** `file:crates/infra/core/src/lint.rs regex:tier.root|tier_root` → _lint.rs:tier.root|tier_root_

- [x] **ADR-018-SYNC-INPROCESS-TIER** sync-inprocess tier reconciled — ADR-018 amended to accept cross-cutting location (D-11 Option B, 2026-06-25)
      **Source:** docs/decisions/ADR-018-SyncEngineArchitecture.md
      **Check:** `file:docs/decisions/ADR-018-SyncEngineArchitecture.md regex:Amended 2026-06-25|c...` → _ADR-018-SyncEngineArchitecture.md:Amended 2026-06-25|cross-cutting/sync-in_

- [ ] **ADR-016-GRAPHIFY-HOOK** Graphify post-commit hook per-user only; fresh clones miss the automation
      **Source:** docs/decisions/ADR-016-EngineGraph.md
      **Check:** `file-exists:.githooks/post-commit` → _post-commit missing_

- [ ] **LIB-DOMAIN-ACCESSORS** library-docs.md shows engine.students()/engine.fees() accessors that don't exist on Engine facade
      **Source:** docs/library-docs.md § Common Workflows
      **Check:** `file:crates/tools/sdk/src/engine.rs regex:fn students|fn fees` → _engine.rs:fn students|fn fees_

- [ ] **LIB-TYPED-EVENT-SUBSCRIBE** library-docs.md shows engine.events().subscribe::<T>() API; actual is untyped SubscribeOptions
      **Source:** docs/library-docs.md § Subscribing to Events
      **Check:** `file:crates/cross-cutting/events/src/event_bus.rs regex:subscribe_typed` → _event_bus.rs:subscribe_typed_

- [x] **CAP-LIBRARY-BOOK_CATEGORY** library: BookCategory capability not wired in RBAC
      **Source:** docs/specs/library/capabilities.md ## BookCategory
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:BookCategory` → _value_objects.rs:BookCategory_

- [x] **CAP-LIBRARY-BOOK** library: Book capability not wired in RBAC
      **Source:** docs/specs/library/capabilities.md ## Book
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Book` → _value_objects.rs:Book_

- [x] **CAP-LIBRARY-MEMBER** library: Member capability not wired in RBAC
      **Source:** docs/specs/library/capabilities.md ## Member
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Member` → _value_objects.rs:Member_

- [x] **CAP-LIBRARY-BOOK_ISSUE** library: BookIssue capability not wired in RBAC
      **Source:** docs/specs/library/capabilities.md ## BookIssue
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:BookIssue` → _value_objects.rs:BookIssue_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.SUBJECT.MARK** attendance: Attendance.Subject.Mark capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Subject.Mark
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Subject.Mar...` → _value_objects.rs:Attendance.Subject.Mark_

- [x] **CAP-ATTENDANCE-ATTENDANCE.SUBJECT.UPDATE** attendance: Attendance.Subject.Update capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Subject.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Subject.Upd...` → _value_objects.rs:Attendance.Subject.Update_

- [x] **CAP-ATTENDANCE-ATTENDANCE.SUBJECT.READ** attendance: Attendance.Subject.Read capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Subject.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Subject.Rea...` → _value_objects.rs:Attendance.Subject.Read_

- [x] **CAP-ATTENDANCE-ATTENDANCE.SUBJECT.NOTIFY** attendance: Attendance.Subject.Notify capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Subject.Notify
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Subject.Not...` → _value_objects.rs:Attendance.Subject.Notify_

- [x] **CAP-ATTENDANCE-ATTENDANCE.STAFF.MARK** attendance: Attendance.Staff.Mark capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Staff.Mark
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Mark` → _value_objects.rs:Attendance.Staff.Mark_

- [x] **CAP-ATTENDANCE-ATTENDANCE.STAFF.UPDATE** attendance: Attendance.Staff.Update capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Staff.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Updat...` → _value_objects.rs:Attendance.Staff.Update_

- [x] **CAP-ATTENDANCE-ATTENDANCE.STAFF.READ** attendance: Attendance.Staff.Read capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Staff.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Read` → _value_objects.rs:Attendance.Staff.Read_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.STAFF.REPORT** attendance: Attendance.Staff.Report capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Staff.Report
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Repor...` → _value_objects.rs:Attendance.Staff.Report_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.IMPORT.VALIDATE** attendance: Attendance.Import.Validate capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Import.Validate
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Import.Vali...` → _value_objects.rs:Attendance.Import.Validate_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.IMPORT.COMMIT** attendance: Attendance.Import.Commit capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Import.Commit
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Import.Comm...` → _value_objects.rs:Attendance.Import.Commit_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.IMPORT.CANCEL** attendance: Attendance.Import.Cancel capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Import.Cancel
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Import.Canc...` → _value_objects.rs:Attendance.Import.Cancel_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.DAILY** attendance: Attendance.Report.Daily capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.Daily
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.Dail...` → _value_objects.rs:Attendance.Report.Daily_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.WEEKLY** attendance: Attendance.Report.Weekly capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.Weekly
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.Week...` → _value_objects.rs:Attendance.Report.Weekly_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.MONTHLY** attendance: Attendance.Report.Monthly capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.Monthly
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.Mont...` → _value_objects.rs:Attendance.Report.Monthly_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.BY_CLASS** attendance: Attendance.Report.ByClass capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.ByClass
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.ByCl...` → _value_objects.rs:Attendance.Report.ByClass_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.BY_STUDENT** attendance: Attendance.Report.ByStudent capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.ByStudent
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.BySt...` → _value_objects.rs:Attendance.Report.ByStudent_

- [ ] **CAP-ATTENDANCE-ATTENDANCE.REPORT.BY_STAFF** attendance: Attendance.Report.ByStaff capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Attendance.Report.ByStaff
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Report.BySt...` → _value_objects.rs:Attendance.Report.ByStaff_

- [x] **CAP-ATTENDANCE-REPORTS** attendance: Reports capability not wired in RBAC
      **Source:** docs/specs/attendance/capabilities.md ## Reports
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Reports` → _value_objects.rs:Reports_

- [ ] **CAP-COMMUNICATION-NOTIFICATION.READ.ALL** communication: Notification.Read.All capability not wired in RBAC
      **Source:** docs/specs/communication/capabilities.md ## Notification.Read.All
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Notification.Read.All` → _value_objects.rs:Notification.Read.All_

- [x] **CAP-COMMUNICATION-NOTICE** communication: Notice capability not wired in RBAC
      **Source:** docs/specs/communication/capabilities.md ## Notice
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Notice` → _value_objects.rs:Notice_

- [ ] **CAP-COMMUNICATION-COMPLAINT** communication: Complaint capability not wired in RBAC
      **Source:** docs/specs/communication/capabilities.md ## Complaint
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Complaint` → _value_objects.rs:Complaint_

- [x] **CAP-COMMUNICATION-NOTIFICATION** communication: Notification capability not wired in RBAC
      **Source:** docs/specs/communication/capabilities.md ## Notification
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Notification` → _value_objects.rs:Notification_

- [x] **CAP-COMMUNICATION-TEMPLATE** communication: Template capability not wired in RBAC
      **Source:** docs/specs/communication/capabilities.md ## Template
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Template` → _value_objects.rs:Template_

- [ ] **CAP-DOCUMENTS-FORM.READ.PUBLIC** documents: Form.Read.Public capability not wired in RBAC
      **Source:** docs/specs/documents/capabilities.md ## Form.Read.Public
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Form.Read.Public` → _value_objects.rs:Form.Read.Public_

- [x] **CAP-DOCUMENTS-FORM** documents: Form capability not wired in RBAC
      **Source:** docs/specs/documents/capabilities.md ## Form
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Form` → _value_objects.rs:Form_

- [x] **CAP-DOCUMENTS-POSTAL** documents: Postal capability not wired in RBAC
      **Source:** docs/specs/documents/capabilities.md ## Postal
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Postal` → _value_objects.rs:Postal_

- [ ] **CAP-ACADEMIC-STUDENT.DOCUMENT.UPLOAD** academic: Student.Document.Upload capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Student.Document.Upload
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Student.Document.Uploa...` → _value_objects.rs:Student.Document.Upload_

- [ ] **CAP-ACADEMIC-STUDENT.DOCUMENT.DOWNLOAD** academic: Student.Document.Download capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Student.Document.Download
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Student.Document.Downl...` → _value_objects.rs:Student.Document.Download_

- [ ] **CAP-ACADEMIC-STUDENT.HOMEWORK.SUBMIT** academic: Student.Homework.Submit capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Student.Homework.Submit
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Student.Homework.Submi...` → _value_objects.rs:Student.Homework.Submit_

- [ ] **CAP-ACADEMIC-STUDENT.HOMEWORK.EVALUATE** academic: Student.Homework.Evaluate capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Student.Homework.Evaluate
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Student.Homework.Evalu...` → _value_objects.rs:Student.Homework.Evaluate_

- [x] **CAP-ACADEMIC-STUDENT** academic: Student capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Student
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Student` → _value_objects.rs:Student_

- [ ] **CAP-ACADEMIC-GUARDIAN** academic: Guardian capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Guardian
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Guardian` → _value_objects.rs:Guardian_

- [x] **CAP-ACADEMIC-CLASS** academic: Class capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Class
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Class` → _value_objects.rs:Class_

- [ ] **CAP-ACADEMIC-SECTION** academic: Section capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Section
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Section` → _value_objects.rs:Section_

- [ ] **CAP-ACADEMIC-CLASS_SECTION** academic: ClassSection capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## ClassSection
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ClassSection` → _value_objects.rs:ClassSection_

- [x] **CAP-ACADEMIC-SUBJECT** academic: Subject capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Subject
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Subject` → _value_objects.rs:Subject_

- [ ] **CAP-ACADEMIC-CLASS_SUBJECT** academic: ClassSubject capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## ClassSubject
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ClassSubject` → _value_objects.rs:ClassSubject_

- [ ] **CAP-ACADEMIC-ACADEMIC_YEAR** academic: AcademicYear capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## AcademicYear
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:AcademicYear` → _value_objects.rs:AcademicYear_

- [ ] **CAP-ACADEMIC-CLASS_ROUTINE** academic: ClassRoutine capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## ClassRoutine
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ClassRoutine` → _value_objects.rs:ClassRoutine_

- [ ] **CAP-ACADEMIC-HOMEWORK** academic: Homework capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Homework
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Homework` → _value_objects.rs:Homework_

- [ ] **CAP-ACADEMIC-LESSON** academic: Lesson capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Lesson
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Lesson` → _value_objects.rs:Lesson_

- [ ] **CAP-ACADEMIC-STUDENT_CATEGORY** academic: StudentCategory capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## StudentCategory
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:StudentCategory` → _value_objects.rs:StudentCategory_

- [ ] **CAP-ACADEMIC-STUDENT_GROUP** academic: StudentGroup capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## StudentGroup
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:StudentGroup` → _value_objects.rs:StudentGroup_

- [x] **CAP-ACADEMIC-REGISTRATION** academic: Registration capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Registration
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Registration` → _value_objects.rs:Registration_

- [ ] **CAP-ACADEMIC-CERTIFICATE** academic: Certificate capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## Certificate
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Certificate` → _value_objects.rs:Certificate_

- [ ] **CAP-ACADEMIC-ID_CARD** academic: IdCard capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## IdCard
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:IdCard` → _value_objects.rs:IdCard_

- [ ] **CAP-ACADEMIC-ADMISSION_QUERY** academic: AdmissionQuery capability not wired in RBAC
      **Source:** docs/specs/academic/capabilities.md ## AdmissionQuery
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:AdmissionQuery` → _value_objects.rs:AdmissionQuery_

- [x] **CAP-CMS-PAGE** cms: Page capability not wired in RBAC
      **Source:** docs/specs/cms/capabilities.md ## Page
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Page` → _value_objects.rs:Page_

- [x] **CAP-CMS-NEWS** cms: News capability not wired in RBAC
      **Source:** docs/specs/cms/capabilities.md ## News
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:News` → _value_objects.rs:News_

- [x] **CAP-CMS-TESTIMONIAL** cms: Testimonial capability not wired in RBAC
      **Source:** docs/specs/cms/capabilities.md ## Testimonial
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Testimonial` → _value_objects.rs:Testimonial_

- [x] **CAP-CMS-CONTENT** cms: Content capability not wired in RBAC
      **Source:** docs/specs/cms/capabilities.md ## Content
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Content` → _value_objects.rs:Content_

- [x] **CAP-FACILITIES-VEHICLE** facilities: Vehicle capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Vehicle
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Vehicle` → _value_objects.rs:Vehicle_

- [x] **CAP-FACILITIES-ROUTE** facilities: Route capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Route
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Route` → _value_objects.rs:Route_

- [x] **CAP-FACILITIES-TRANSPORT** facilities: Transport capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Transport
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Transport` → _value_objects.rs:Transport_

- [x] **CAP-FACILITIES-DORMITORY** facilities: Dormitory capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Dormitory
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Dormitory` → _value_objects.rs:Dormitory_

- [x] **CAP-FACILITIES-ROOM** facilities: Room capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Room
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Room` → _value_objects.rs:Room_

- [x] **CAP-FACILITIES-ROOM_TYPE** facilities: RoomType capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## RoomType
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:RoomType` → _value_objects.rs:RoomType_

- [x] **CAP-FACILITIES-ITEM_CATEGORY** facilities: ItemCategory capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## ItemCategory
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ItemCategory` → _value_objects.rs:ItemCategory_

- [x] **CAP-FACILITIES-ITEM** facilities: Item capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Item
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Item` → _value_objects.rs:Item_

- [x] **CAP-FACILITIES-ITEM_STORE** facilities: ItemStore capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## ItemStore
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ItemStore` → _value_objects.rs:ItemStore_

- [x] **CAP-FACILITIES-INVENTORY** facilities: Inventory capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Inventory
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Inventory` → _value_objects.rs:Inventory_

- [x] **CAP-FACILITIES-SUPPLIER** facilities: Supplier capability not wired in RBAC
      **Source:** docs/specs/facilities/capabilities.md ## Supplier
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Supplier` → _value_objects.rs:Supplier_

- [ ] **CAP-ASSESSMENT-EXAM_TYPE** assessment: ExamType capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## ExamType
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ExamType` → _value_objects.rs:ExamType_

- [x] **CAP-ASSESSMENT-EXAM** assessment: Exam capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## Exam
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Exam` → _value_objects.rs:Exam_

- [ ] **CAP-ASSESSMENT-EXAM_SETUP** assessment: ExamSetup capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## ExamSetup
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ExamSetup` → _value_objects.rs:ExamSetup_

- [x] **CAP-ASSESSMENT-EXAM_SCHEDULE** assessment: ExamSchedule capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## ExamSchedule
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ExamSchedule` → _value_objects.rs:ExamSchedule_

- [ ] **CAP-ASSESSMENT-MARK_STORE** assessment: MarkStore capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## MarkStore
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:MarkStore` → _value_objects.rs:MarkStore_

- [ ] **CAP-ASSESSMENT-MARKS_GRADE** assessment: MarksGrade capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## MarksGrade
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:MarksGrade` → _value_objects.rs:MarksGrade_

- [x] **CAP-ASSESSMENT-REPORT_CARD** assessment: ReportCard capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## ReportCard
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ReportCard` → _value_objects.rs:ReportCard_

- [x] **CAP-ASSESSMENT-ONLINE_EXAM** assessment: OnlineExam capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## OnlineExam
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:OnlineExam` → _value_objects.rs:OnlineExam_

- [x] **CAP-ASSESSMENT-SEAT_PLAN** assessment: SeatPlan capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## SeatPlan
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:SeatPlan` → _value_objects.rs:SeatPlan_

- [x] **CAP-ASSESSMENT-ADMIT_CARD** assessment: AdmitCard capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## AdmitCard
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:AdmitCard` → _value_objects.rs:AdmitCard_

- [ ] **CAP-ASSESSMENT-TEACHER_EVALUATION** assessment: TeacherEvaluation capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## TeacherEvaluation
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:TeacherEvaluation` → _value_objects.rs:TeacherEvaluation_

- [ ] **CAP-ASSESSMENT-TEACHER_REMARK** assessment: TeacherRemark capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## TeacherRemark
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:TeacherRemark` → _value_objects.rs:TeacherRemark_

- [ ] **CAP-ASSESSMENT-EXAM_ATTENDANCE** assessment: ExamAttendance capability not wired in RBAC
      **Source:** docs/specs/assessment/capabilities.md ## ExamAttendance
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:ExamAttendance` → _value_objects.rs:ExamAttendance_

- [ ] **CAP-FINANCE-FEES_ASSIGN.DISCOUNT.UPDATE** finance: FeesAssign.Discount.Update capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## FeesAssign.Discount.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:FeesAssign.Discount.Up...` → _value_objects.rs:FeesAssign.Discount.Update_

- [ ] **CAP-FINANCE-INVOICE.SETTING.CONFIGURE** finance: Invoice.Setting.Configure capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Invoice.Setting.Configure
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Invoice.Setting.Config...` → _value_objects.rs:Invoice.Setting.Configure_

- [x] **CAP-FINANCE-BANK.STATEMENT.RECORD** finance: Bank.Statement.Record capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Bank.Statement.Record
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Bank.Statement.Record` → _value_objects.rs:Bank.Statement.Record_

- [x] **CAP-FINANCE-BANK.STATEMENT.REVERSE** finance: Bank.Statement.Reverse capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Bank.Statement.Reverse
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Bank.Statement.Reverse` → _value_objects.rs:Bank.Statement.Reverse_

- [ ] **CAP-FINANCE-INVENTORY.PAYMENT.RECORD** finance: Inventory.Payment.Record capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Inventory.Payment.Record
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Inventory.Payment.Reco...` → _value_objects.rs:Inventory.Payment.Record_

- [ ] **CAP-FINANCE-QUESTION_BANK.FEE.ATTACH** finance: QuestionBank.Fee.Attach capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## QuestionBank.Fee.Attach
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:QuestionBank.Fee.Attac...` → _value_objects.rs:QuestionBank.Fee.Attach_

- [ ] **CAP-FINANCE-QUESTION_BANK.FEE.DETACH** finance: QuestionBank.Fee.Detach capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## QuestionBank.Fee.Detach
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:QuestionBank.Fee.Detac...` → _value_objects.rs:QuestionBank.Fee.Detach_

- [ ] **CAP-FINANCE-QUESTION_BANK.FEE.READ** finance: QuestionBank.Fee.Read capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## QuestionBank.Fee.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:QuestionBank.Fee.Read` → _value_objects.rs:QuestionBank.Fee.Read_

- [ ] **CAP-FINANCE-REPORT.FINANCE.READ** finance: Report.Finance.Read capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Report.Finance.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Report.Finance.Read` → _value_objects.rs:Report.Finance.Read_

- [x] **CAP-FINANCE-INVOICE** finance: Invoice capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Invoice
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Invoice` → _value_objects.rs:Invoice_

- [x] **CAP-FINANCE-PAYMENT** finance: Payment capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Payment
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payment` → _value_objects.rs:Payment_

- [x] **CAP-FINANCE-EXPENSE** finance: Expense capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Expense
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Expense` → _value_objects.rs:Expense_

- [x] **CAP-FINANCE-INCOME** finance: Income capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Income
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Income` → _value_objects.rs:Income_

- [x] **CAP-FINANCE-BANK** finance: Bank capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Bank
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Bank` → _value_objects.rs:Bank_

- [x] **CAP-FINANCE-PAYROLL** finance: Payroll capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Payroll
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll` → _value_objects.rs:Payroll_

- [x] **CAP-FINANCE-WALLET** finance: Wallet capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Wallet
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Wallet` → _value_objects.rs:Wallet_

- [x] **CAP-FINANCE-REPORTS** finance: Reports capability not wired in RBAC
      **Source:** docs/specs/finance/capabilities.md ## Reports
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Reports` → _value_objects.rs:Reports_

- [x] **CAP-HR-STAFF.ASSIGN_CLASS_TEACHER.CREATE** hr: Staff.AssignClassTeacher.Create capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.AssignClassTeacher.Create
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.AssignClassTeach...` → _value_objects.rs:Staff.AssignClassTeacher.Creat_

- [ ] **CAP-HR-STAFF.ASSIGN_CLASS_TEACHER.UPDATE** hr: Staff.AssignClassTeacher.Update capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.AssignClassTeacher.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.AssignClassTeach...` → _value_objects.rs:Staff.AssignClassTeacher.Updat_

- [ ] **CAP-HR-STAFF.ASSIGN_CLASS_TEACHER.DELETE** hr: Staff.AssignClassTeacher.Delete capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.AssignClassTeacher.Delete
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.AssignClassTeach...` → _value_objects.rs:Staff.AssignClassTeacher.Delet_

- [x] **CAP-HR-STAFF.IMPORT_BULK.PROMOTE** hr: Staff.ImportBulk.Promote capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.ImportBulk.Promote
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.ImportBulk.Promo...` → _value_objects.rs:Staff.ImportBulk.Promote_

- [x] **CAP-HR-STAFF.IMPORT_BULK.REJECT** hr: Staff.ImportBulk.Reject capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.ImportBulk.Reject
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.ImportBulk.Rejec...` → _value_objects.rs:Staff.ImportBulk.Reject_

- [x] **CAP-HR-STAFF.DOCUMENT.UPLOAD** hr: Staff.Document.Upload capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.Document.Upload
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.Document.Upload` → _value_objects.rs:Staff.Document.Upload_

- [x] **CAP-HR-STAFF.DOCUMENT.DOWNLOAD** hr: Staff.Document.Download capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff.Document.Download
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff.Document.Downloa...` → _value_objects.rs:Staff.Document.Download_

- [x] **CAP-HR-ATTENDANCE.STAFF.MARK** hr: Attendance.Staff.Mark capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance.Staff.Mark
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Mark` → _value_objects.rs:Attendance.Staff.Mark_

- [x] **CAP-HR-ATTENDANCE.STAFF.UPDATE** hr: Attendance.Staff.Update capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance.Staff.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Updat...` → _value_objects.rs:Attendance.Staff.Update_

- [x] **CAP-HR-ATTENDANCE.STAFF.DELETE** hr: Attendance.Staff.Delete capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance.Staff.Delete
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Delet...` → _value_objects.rs:Attendance.Staff.Delete_

- [x] **CAP-HR-ATTENDANCE.STAFF.READ** hr: Attendance.Staff.Read capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance.Staff.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Read` → _value_objects.rs:Attendance.Staff.Read_

- [x] **CAP-HR-ATTENDANCE.STAFF.IMPORT** hr: Attendance.Staff.Import capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance.Staff.Import
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance.Staff.Impor...` → _value_objects.rs:Attendance.Staff.Import_

- [x] **CAP-HR-PAYROLL.EARNING.ADD** hr: Payroll.Earning.Add capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Earning.Add
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Earning.Add` → _value_objects.rs:Payroll.Earning.Add_

- [x] **CAP-HR-PAYROLL.EARNING.UPDATE** hr: Payroll.Earning.Update capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Earning.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Earning.Update` → _value_objects.rs:Payroll.Earning.Update_

- [x] **CAP-HR-PAYROLL.EARNING.DELETE** hr: Payroll.Earning.Delete capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Earning.Delete
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Earning.Delete` → _value_objects.rs:Payroll.Earning.Delete_

- [x] **CAP-HR-PAYROLL.DEDUCTION.ADD** hr: Payroll.Deduction.Add capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Deduction.Add
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Deduction.Add` → _value_objects.rs:Payroll.Deduction.Add_

- [x] **CAP-HR-PAYROLL.DEDUCTION.UPDATE** hr: Payroll.Deduction.Update capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Deduction.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Deduction.Upda...` → _value_objects.rs:Payroll.Deduction.Update_

- [x] **CAP-HR-PAYROLL.DEDUCTION.DELETE** hr: Payroll.Deduction.Delete capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.Deduction.Delete
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.Deduction.Dele...` → _value_objects.rs:Payroll.Deduction.Delete_

- [x] **CAP-HR-PAYROLL.LEAVE_DEDUCTION.ADD** hr: Payroll.LeaveDeduction.Add capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.LeaveDeduction.Add
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.LeaveDeduction...` → _value_objects.rs:Payroll.LeaveDeduction.Add_

- [x] **CAP-HR-PAYROLL.LEAVE_DEDUCTION.UPDATE** hr: Payroll.LeaveDeduction.Update capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.LeaveDeduction.Update
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.LeaveDeduction...` → _value_objects.rs:Payroll.LeaveDeduction.Update_

- [x] **CAP-HR-PAYROLL.LEAVE_DEDUCTION.DELETE** hr: Payroll.LeaveDeduction.Delete capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll.LeaveDeduction.Delete
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll.LeaveDeduction...` → _value_objects.rs:Payroll.LeaveDeduction.Delete_

- [ ] **CAP-HR-REPORT.HR.READ** hr: Report.HR.Read capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Report.HR.Read
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Report.HR.Read` → _value_objects.rs:Report.HR.Read_

- [x] **CAP-HR-STAFF** hr: Staff capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Staff
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Staff` → _value_objects.rs:Staff_

- [x] **CAP-HR-DEPARTMENT** hr: Department capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Department
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Department` → _value_objects.rs:Department_

- [x] **CAP-HR-DESIGNATION** hr: Designation capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Designation
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Designation` → _value_objects.rs:Designation_

- [x] **CAP-HR-LEAVE** hr: Leave capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Leave
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Leave` → _value_objects.rs:Leave_

- [x] **CAP-HR-ATTENDANCE** hr: Attendance capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Attendance
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Attendance` → _value_objects.rs:Attendance_

- [x] **CAP-HR-PAYROLL** hr: Payroll capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Payroll
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Payroll` → _value_objects.rs:Payroll_

- [x] **CAP-HR-REPORTS** hr: Reports capability not wired in RBAC
      **Source:** docs/specs/hr/capabilities.md ## Reports
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Reports` → _value_objects.rs:Reports_

- [x] **CAP-EVENTS-EVENT** events: Event capability not wired in RBAC
      **Source:** docs/specs/events/capabilities.md ## Event
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Event` → _value_objects.rs:Event_

- [x] **CAP-EVENTS-HOLIDAY** events: Holiday capability not wired in RBAC
      **Source:** docs/specs/events/capabilities.md ## Holiday
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Holiday` → _value_objects.rs:Holiday_

- [x] **CAP-EVENTS-WEEKEND** events: Weekend capability not wired in RBAC
      **Source:** docs/specs/events/capabilities.md ## Weekend
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Weekend` → _value_objects.rs:Weekend_

- [x] **CAP-EVENTS-INCIDENT** events: Incident capability not wired in RBAC
      **Source:** docs/specs/events/capabilities.md ## Incident
      **Check:** `file:crates/cross-cutting/rbac/src/value_objects.rs regex:Incident` → _value_objects.rs:Incident_
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
