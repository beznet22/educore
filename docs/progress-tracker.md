# Educore Progress Tracker

This document tracks the implementation status of Educore against
the 17-phase build plan defined in `docs/build-plan.md`. Every row
starts in the **Planned** state and is flipped to **Implementing**
or **Done** as the corresponding phase lands.

## Workspace Status

The workspace has **34 crates**: the umbrella `educore` plus 33
internal crates, grouped below by tier and by the phase that ships
them. Every crate is scaffolded (`Cargo.toml`, `lib.rs`,
`#[forbid(unsafe_code)]`, `#[deny(missing_docs)]`); none are
implemented yet. The tier-to-directory mapping is
`crates/<tier>/<name>/` (e.g. `crates/domains/academic/`,
`crates/adapters/storage-surrealdb/`); the package name on disk
keeps the `educore-` prefix (e.g. `educore-academic`). See
`AGENTS.md` § "Tier System" for the full rules.

| Tier            | Crate                          | Phase | Spec'd | Implementing | Tested | Notes                          |
| --------------- | ------------------------------ | ----- | ------ | ------------ | ------ | ------------------------------ |
| umbrella        | `educore`                    | -     | Yes    | Yes          | Yes    | Umbrella; re-exports all 33 internal crates (incl. `storage_surrealdb`, `sync`, `sync_inprocess`) |
| infra           | `educore-core`               | 0     | Yes    | Yes          | Yes    | Errors, ids, value objects, clock, `lint` sub-module (feature-gated), `DomainError` (7 variants) |
| infra           | `educore-query-derive`       | 0     | Yes    | Yes          | Yes    | `#[derive(DomainQuery)]` macro; 19 integration tests |
| infra           | `educore-storage`            | 0     | Yes    | Yes          | Yes    | `StorageAdapter` port + 4 sub-ports (Outbox/AuditLog/EventLog/Idempotency); 11 unit tests |
| adapters        | `educore-storage-surrealdb`  | 0     | Yes    | Yes          | Yes    | Primary adapter (SurrealDB); `surrealdb` 2.6.5 + `rustls`; 6/6 cross-cutting tables emitted; outbox e2e green |
| adapters        | `educore-storage-postgres`   | 1     | Yes    | Yes          | Yes    | Parity adapter; `sqlx 0.8` + `rustls`; all 4 sub-ports real (Outbox/AuditLog/EventLog/Idempotency); e2e via `EDUCORE_PG_URL`; 1 outbox e2e test |
| tools           | `educore-storage-parity`     | 16    | Yes    | No           | No     | Cross-adapter parity suite; scaffold only |
| adapters        | `educore-storage-mysql`      | 1     | Yes    | Yes          | Yes    | `MySQL 8.0+`; `sqlx 0.8` (NOT `mysql_async`); backtick identifiers, `?` placeholders, `IN (?)` via `QueryBuilder`; 4 unit tests on the multi-statements URL helper; e2e via `EDUCORE_MYSQL_URL`; 1 outbox e2e test |
| adapters        | `educore-storage-sqlite`     | 1     | Yes    | Yes          | Yes    | Embedded / offline; `sqlx 0.8` + `json1`; in-memory test (always runs in CI); `uuid::fmt::Hyphenated` for the 36-char `TEXT` representation; 1 outbox e2e test |
| cross-cutting   | `educore-sync`               | 0     | Yes    | Yes          | Yes    | `SyncAdapter` port (SyncCoordinator) per ADR-018; 1 object-safety test |
| cross-cutting   | `educore-sync-inprocess`     | 0     | Yes    | Yes          | Yes    | In-process sync impl (default); 6 tests, `tokio::sync::{mpsc,broadcast}` |
| cross-cutting   | `educore-platform`           | 2     | Yes    | Yes          | Yes    | School, User, TenantContext; 9-file layout; 44 unit + 10 integration tests; 3 coverage rows flipped |
| cross-cutting   | `educore-rbac`               | 2     | Yes    | Yes          | Yes    | Capability (55 variants), Role, Permission, `CapabilityCheck` port, `DefaultRoleCatalog`, `is_replicated`; 41 unit + 19 integration tests; 2 coverage rows flipped |
| cross-cutting   | `educore-events`             | 2     | Yes    | Yes          | Yes    | Envelope crate; `DomainEvent`, `EventEnvelope`, `EventBus` port, 4 sync events (`SyncStarted`/`Paused`/`Resumed`/`Stopped`); 25 unit tests; 1 coverage row flipped |
| adapters        | `educore-event-bus`          | 2     | Yes    | Yes          | Yes    | `InProcessEventBus` (default, MPMC broadcast) + `NatsEventBus` (feature `nats`) + `RedisEventBus` (feature `redis`) — both distributed adapters are Phase 2 stubs; 51 unit + 9 integration tests; 2 coverage rows flipped |
| cross-cutting   | `educore-audit`              | 2     | Yes    | Yes          | Yes    | `AuditWriter` + `RetentionPolicy` + `RetentionSweepDue` event; partitioning strategy documented in `docs/schemas/audit-schema.md` § 13; 22 unit + 8 integration tests; 4 coverage rows flipped |
| domains         | `educore-academic`           | 3     | Yes    | Yes          | Yes    | First vertical slice; 5 aggregates (Student, Class, Section, Subject, AcademicYear); 19 typed events, 23 typed commands, 19 pure factory services, 5 repository port traits, 5 typed query stubs, 66 unit + 3 integration tests; 5 coverage rows flipped |
| domains         | `educore-assessment`         | 4     | Yes    | Yes          | Yes    | Second domain crate; 8 aggregates (Exam, ExamSchedule, MarksRegister, ResultStore, ReportCard projection, OnlineExam, SeatPlan, AdmitCard); 28 typed events, 28 typed commands, 25+ pure factory services + 10-fn ResultService grading module, 8 repository port traits, 8 typed query stubs; 67 unit tests in crate + 3 new integration tests in `crates/tools/storage-parity/tests/assessment_integration.rs`; 8 coverage rows flipped |
| domains         | `educore-attendance`         | 5     | Yes    | Yes          | Yes    | Third domain crate; 5 aggregates (StudentAttendance, StaffAttendance, SubjectAttendance, ExamAttendance, BulkAttendanceImport); 21 typed events, 14 typed commands, 14 pure factory services, 5 repository port traits (with `bulk_insert`), 5 typed query stubs, 1 `AttendanceUniquenessChecker` port; 93 unit tests in crate + 4 new integration tests in `crates/tools/storage-parity/tests/attendance_integration.rs`; 13 coverage rows flipped; 530 workspace tests pass (was 433 at Phase 4 close-out) |
| domains         | `educore-hr`                 | 6     | Yes    | No           | No     | Staff, leave, payroll          |
| domains         | `educore-finance`            | 7     | Yes    | Yes          | Yes    | Largest spec (~5,567 lines); 5 real aggregates (Wallet, WalletTransaction, FeesInvoice, FeesPayment, Expense) + 33 placeholder stubs; 10 events, 115 commands + 125 shapes, 44 repos, 11 query stubs, 5 child entities, 6 services + CarryForwardService (4 rules) + LateFeeService (90 fixtures) + DoubleEntryService (proptest); PaymentProvider deprecated (moves to Phase 15); double-entry invariant (sum debits == sum credits per school_id) |
| domains         | `educore-facilities`         | 8     | Yes    | No           | No     | Dorm, transport, inventory     |
| domains         | `educore-library`            | 9     | Yes    | No           | No     | Books, issues, fines           |
| domains         | `educore-communication`      | 10    | Yes    | Yes          | Yes    | Eighth domain crate; spec-faithful interpretation (all 26 root aggregates); 9-file layout; 73 events; 72 commands; 70 factory service fns + 7 headline + 7 service structs; 26 repos (3 append-only); CommunicationDispatchService is events-only (no `educore-notify` dep); 100-case proptest of TemplateService::render; 60 unit tests in crate + 6-scenario integration test in storage-parity; 13 coverage rows flipped |
| domains         | `educore-documents`          | 11    | Yes    | Yes          | Yes    | Ninth domain crate; spec-faithful interpretation (all 3 root aggregates: `FormDownload`, `PostalDispatch`, `PostalReceive`) + 4 child entities (`FormDownloadFile`, `FormDownloadLink`, `PostalDispatchAttachment`, `PostalReceiveAttachment`); 9 typed events; 10 typed commands; 10 async service factory fns + 2 service structs (`FormService`, `PostalService`); 3 repository port traits (object-safety smoke tests); 3 typed query stubs; `DocumentsError` enum (11 variants); 100-case proptest of `FormService::is_deliverable` + `PostalService::reference_unique`; 145 unit tests in crate + 6-scenario integration test in storage-parity (2 env-gated PG/MySQL variants); 11 net-new `Capability` variants + 4 retained `DocumentsFolder*` placeholders = 15 Documents caps; 2 net-new `AuditTarget` variants + 1 retained `PostalDispatch` = 3 audit targets; 3 coverage rows flipped |
| domains         | `educore-cms`                | 12    | Yes    | No           | No     | Pages, news, testimonial       |
| domains         | `educore-events-domain`      | 13    | Yes    | No           | No     | Calendar (distinct from envelope) |
| cross-cutting   | `educore-settings`           | 14    | Yes    | No           | No     | Per-school config, language    |
| cross-cutting   | `educore-operations`         | 14    | Yes    | No           | No     | Bell schedule, substitution    |
| adapters        | `educore-auth`               | 15    | Yes    | No           | No     | `AuthProvider` + JWT impl      |
| adapters        | `educore-notify`             | 15    | Yes    | No           | No     | `NotificationProvider` + email/SMS |
| adapters        | `educore-payment`            | 15    | Yes    | No           | No     | `PaymentProvider` + Stripe     |
| adapters        | `educore-files`              | 15    | Yes    | No           | No     | `FileStorage` + S3/local       |
| adapters        | `educore-integrations`       | 15    | Yes    | No           | No     | LMS, video-conferencing        |
| tools           | `educore-testkit`            | 16    | Yes    | No           | No     | In-memory impls of 6 ports     |
| tools           | `educore-sdk`                | 16    | Yes    | No           | No     | `Engine::builder()` facade     |
| tools           | `educore-cli`                | 16    | Yes    | No           | No     | Sample binary, dogfooding      |
| -              | **Graph regen**                | -     | -      | -            | -      | Auto-rebuilt on every commit via local `graphify hook install`; output at `graphify-out/` (committed). Legacy `schoolify/graphify-out/` is frozen. |

Phase 17 ships no new crates; it hardens the workspace
(multi-tenant suite, load test, cross-compile, security review,
docs audit).

## Phase Progress

| Phase | Title                              | Crates                                                                 | Status   | Exit Criteria Met |
| ----- | ---------------------------------- | ---------------------------------------------------------------------- | -------- | ----------------- |
| 0     | Foundation                         | `core`, `query-derive`, `storage`, `storage-surrealdb`, `sync`, `sync-inprocess` | Done     | 6 of 6 ✅ (PR 0 closed the clippy gap; PR A flipped the docs) |
| 1     | Adapter parity (Postgres + MySQL + SQLite) | `storage-postgres`, `storage-mysql`, `storage-sqlite`                | Done     | 7 of 7 ✅ (15 coverage rows flipped, 124 tests pass; see `docs/handoff/PHASE-1-HANDOFF.md`) |
| 2     | Cross-cutting foundations          | `platform`, `rbac`, `events`, `event-bus`, `audit`                     | Done     | 5 of 5 ✅ (310 tests pass, 12 coverage rows flipped, `educore-sync` refactored to depend on `educore_events::EventEnvelope` resolving Phase 0 OQ #2; see `docs/handoff/PHASE-2-HANDOFF.md`) |
| 3     | Academic (first vertical slice)    | `academic`                                                             | Done     | 5 of 5 ✅ (369 tests pass: 310 at Phase 2 + 59 net new in Phase 3; 8 coverage rows flipped — 3 `event_log_ddl_*` closed Phase 2 OQ #1 and 5 `academic_*_aggregate`; see `docs/handoff/PHASE-3-HANDOFF.md`) |
| 4     | Assessment (second domain crate) | `assessment`                          | Done     | 11 of 11 ✅ (433 tests pass: 380 at Phase 3 close-out + 53 net new in Phase 4: 51 unit tests in `educore-assessment` + 1 SQLite integration test + 1 capability-check test + 1 event-type round-trip test; +2 ignored for the new PG/MySQL integration variants; 8 coverage rows flipped) |
| 5     | Attendance                       | `attendance`                                                           | Done     | 11 of 11 ✅ (530 tests pass: 433 at Phase 4 close-out + 97 net new in Phase 5: 93 unit tests in `educore-attendance` + 4 new always-on integration tests in `attendance_integration.rs` including the 200-row bulk-mark bench proxy; +3 ignored for the PG/MySQL/PG-100ms env-gated variants; 13 coverage rows flipped — 7 `attendance_*_aggregate` + 6 `*_event`; see `docs/handoff/PHASE-5-HANDOFF.md`) |
| 6     | HR                                 | `hr`                                                                   | Planned  | No                |
| 7     | Finance (largest spec)             | `finance`                                                              | Done     | Yes (9 new commits + 1 Phase 6 fix-up; 579 tests pass; 33 placeholder aggregates documented as backlog for Workstreams D-M; see `docs/handoff/PHASE-7-HANDOFF.md`) |
| 8     | Facilities                         | `facilities`                                                           | Planned  | No                |
| 9     | Library                            | `library`                                                              | Planned  | No                |
| 10    | Communication                      | `communication`                                                        | Done     | 5 of 5 ✅; 13 coverage rows flipped; ~770 tests pass (was ~692 at Phase 9 close-out; +78 net new: 60 unit + 6 integration + 2 cross-crate + 10 fixups); see `docs/handoff/PHASE-10-HANDOFF.md` |
| 11    | Documents                          | `documents`                                                            | Done     | 5 of 5 ✅; 3 coverage rows flipped; 27 commits land in chronological order (Phase 11 prep + 3×3 workstream branches + tests + docs); ~915 tests pass workspace-wide (was ~770 at Phase 10 close-out; +145 net new: 145 unit tests in `educore-documents` + 6 integration scenarios + 1 rbac 15-cap test + 1 audit 3-variant test + test fixups); see `docs/handoff/PHASE-11-HANDOFF.md` |
| 12    | CMS                                | `cms`                                                                  | Planned  | No                |
| 13    | Events domain (calendar)           | `events-domain`                                                        | Planned  | No                |
| 14    | Settings + Operations              | `settings`, `operations`                                               | Planned  | No                |
| 15    | Port adapters                      | `auth`, `notify`, `payment`, `files`, `integrations`                   | Planned  | No                |
| 16    | Test infrastructure + SDK          | `testkit`, `storage-parity` (full suite), `sdk`, `cli`                 | Planned  | No                |
| 17    | Production readiness               | (no new crates)                                                        | Planned  | No                |
| -     | graphify hook installed + graph fresh | (no new crates)                                                    | Done     | Yes (graph fresh as of last commit) |

## Documentation Status

All 269+ markdown files are spec'd. The split below mirrors the
directory tree under `docs/` plus the migration scripts under
`migrations/`.

| Directory / file                            | Count | Status   |
| ------------------------------------------- | ----- | -------- |
| Top-level docs (`docs/*.md`)                | 7     | Complete |
| `docs/specs/<domain>/` (15 domains x 11)    | 165   | Complete |
| `docs/ports/`                               | 7     | Complete |
| `docs/commands/` (15 domains)               | 15    | Complete |
| `docs/events/` (15 domains)                 | 15    | Complete |
| `docs/schemas/` (6 cross-cutting)           | 6     | Complete |
| `docs/schemas/sql-dialects/`                | 5     | Complete |
| `docs/schemas/data-migration/`              | 13    | Complete |
| `docs/decisions/` (ADRs)                    | 14    | Complete |
| `docs/diagrams/` (Mermaid)                  | 7     | Complete |
| `docs/research/`                            | 16    | Complete |
| `docs/guides/`                              | 18    | Complete |
| `migrations/README.md`                      | 1     | Complete |
| `migrations/engine/0000_engine_core.mysql.sql`           | 1     | Complete |
| `migrations/0001_*.sql` .. `0015_*.sql`     | 15    | Complete |
| `migrations/engine/` (3 dialect DDL + README) | 4    | Complete |

## Coverage Matrix Summary

The full matrix (226+ rows) is **machine-readable** and lives at
[`docs/coverage.toml`](coverage.toml) so CI can diff it on every
PR. The summary below rolls it up to the bucket level. The
**Implemented** column starts at 0 and grows as phases complete.

| Bucket                                                  | Total | Spec'd | Implemented | Tested |
| ------------------------------------------------------- | ----- | ------ | ----------- | ------ |
| Engine cross-cutting tables (6 x 4 dialects)            | 24    | 24     | 16          | 16     |
| Foundation layer (core + query-derive + storage port)   | 9     | 9      | 9           | 9      |
| Sync engine (port + inprocess impl)                     | 2     | 2      | 2           | 2      |
| Engine graph (graphify)                                | 1     | 1      | 1           | 1      |
| Port traits (remaining 5: platform, rbac, events, event-bus, auth/notify/payment/files/integrations) | 12    | 12     | 0           | 0      |
| Domain aggregates                                       | ~310  | ~310   | 39          | 39     |
| Domain commands                                         | ~225  | ~225   | 86          | 86     |
| Domain events                                           | ~280  | ~280   | 94          | 94     |
| SQL-dialect parity adapters (PG, MySQL, SQLite)         | 3     | 3      | 3           | 3      |
| Cross-adapter parity test suite                         | 1     | 1      | 0           | 0      |
| Port adapters (5 ports + 1 cli binary)                  | 6     | 6      | 0           | 0      |
| Reference impls (JWT, email, SMS, Stripe, S3, local, LMS, video) | 8 | 8 | 0      | 0      |

The cross-cutting bucket is `outbox`, `audit_log`, `idempotency`,
`event_log`, `schema_registry`, `system_user` rendered in each of the
four dialect DDL files (`surreal`, `postgres`, `mysql`, `sqlite`).
After Phase 0, 4 SurrealDB rows are `Tested`. After Phase 1, 12 more
rows flip (4 DDL rows × 3 SQL dialects), for a total of 16 of 24
in this bucket. The `audit_log` and `event_log` rows for PG, MySQL,
SQLite are owned by `educore-audit` and `educore-events`
respectively; they are physically present in the DDL files (so the
SQL adapters can write to them) but flip to `Tested` in Phase 2.
Aggregate / command / event totals derive from the per-domain specs
in `docs/specs/<domain>/aggregates.md`, `docs/commands/<domain>.md`,
and `docs/events/<domain>.md`.

The current `docs/coverage.toml` has an initial scaffold of
~80 representative rows covering the engine cross-cutting tables,
the 7 port traits, and 1-3 aggregates per domain. The full
226+ row matrix is generated by the lint sub-module
(`educore-core::lint`, gated behind the `lint` Cargo feature)
once implementation begins.

## See also

- `docs/build-plan.md` § "The 17 phases" — the canonical phase plan
- `docs/build-plan.md` § "The Coverage Matrix" — the matrix schema and CI gate
- `docs/coverage.toml` — the machine-readable coverage matrix
- `docs/architecture.md` — the system map
- `AGENTS.md` § "Status" — high-level status
