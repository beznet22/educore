# SMSengine Build Plan

The engine is implemented in **17 sequential phases** (Phase 0..17). Each
phase has explicit exit criteria and updates the Coverage Matrix
(§ [The Coverage Matrix](#the-coverage-matrix)). The runtime DDL
emission flow referenced throughout is documented in
[`docs/schemas/sql-dialects/README.md` § "Runtime DDL emission — end-to-end flow"](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow).

## The 17 phases

1. Phase 0 — Foundation: `core`, `query-derive`, `storage` port, `storage-postgres` + outbox e2e
2. Phase 1 — Adapter parity: `storage-mysql`, `storage-sqlite` + cross-adapter test
3. Phase 2 — Cross-cutting foundations: `platform`, `rbac`, `events`, `event-bus`, `audit`
4. Phase 3 — Academic (first vertical slice)
5. Phase 4 — Assessment
6. Phase 5 — Attendance
7. Phase 6 — HR
8. Phase 7 — Finance (largest spec)
9. Phase 8 — Facilities
10. Phase 9 — Library
11. Phase 10 — Communication
12. Phase 11 — Documents
13. Phase 12 — CMS
14. Phase 13 — Events domain (calendar)
15. Phase 14 — Settings + Operations
16. Phase 15 — Port adapters: `auth`, `notify`, `payment`, `files`, `integrations`
17. Phase 16 — Test infrastructure + SDK: `testkit`, `storage-parity`, `sdk`, `cli`
18. Phase 17 — Production readiness: integration tests, load tests, cross-compile, security review, docs audit

## Pre-implementation state

All 269 markdown files are spec'd. The workspace has **34 crates** (29
from the original scaffold + 5 new: `smsengine-audit`,
`smsengine-operations`, `smsengine-testkit`, `smsengine-cli`,
`smsengine-storage-parity`). Domain spec cleanup is complete: all
legacy `sm_` / `fm_` / `infix_` / `front_` / `check_` / `un_` table
references have been removed from `docs/specs/` and replaced with
engine `<domain>_<aggregate>` names.

The runtime DDL emission flow is documented in
[`docs/schemas/sql-dialects/README.md`](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow)
§ "Runtime DDL emission — end-to-end flow". The five views are:
**Design contract** (`docs/specs/<domain>/tables.md`) → **Type
contract** (`crates/<domain>/src/aggregate.rs`) → **Machine contract**
(`crates/<domain>/src/entities.rs`, macro-emitted AST) → **Adapter
emission** (`smsengine-storage-<db>`) → **Consumer startup**
(`storage.create_schema().await`).

Migrations live in `migrations/engine/` (3 dialect files for the 6
cross-cutting tables: `outbox`, `audit_log`, `idempotency`,
`event_log`, `schema_registry`, `system_user`). The adapter crates
`include_str!` these files at compile time. The ~310 domain tables
are emitted from the macro AST at runtime, not from `.sql` files.

---

## Phase 0 — Foundation: `core` + macro + storage port + PG adapter + outbox e2e

**Deliverables.** `smsengine-core`, `smsengine-query-derive`,
`smsengine-storage` (port trait only), `smsengine-storage-postgres`
(full impl). The first end-to-end test passes: create schema, insert
one outbox row, read it back, verify invariants.

**Tasks.**

1. `smsengine-core`: `errors.rs` (`DomainError` via `thiserror`),
   `ids.rs` (`SchoolId`, `UserId`, `EventId`, `CorrelationId`,
   `Source` — `UuidV7`), `value_objects.rs` (`Timestamp`, `Version`,
   `Etag`, `ActiveStatus`), `clock.rs` (`Clock` trait + `SystemClock`
   + `TestClock`), `id_gen.rs` (v7 UUID generator with
   deterministic test backend), `tenant.rs` (`TenantContext`), and
   `query.rs` (the `EntityDescriptor` AST types consumed by the
   macro).
2. `smsengine-query-derive`: the `#[derive(DomainQuery)]` proc macro.
   Reads the struct's fields, field types, `#[domain_query(...)]`
   attributes, and emits an `EntityDescriptor { table, columns,
   indexes, foreign_keys, rls }`. Emits a `__spec_coverage__` test
   module on every `#[derive(DomainQuery)]` (see § [The No-Gaps Gates](#the-no-gaps-gates)).
3. `smsengine-storage`: the `StorageAdapter` port trait
   (`create_schema`, `apply_command`, `query`, `begin_tx`,
   `commit_tx`, `rollback_tx`) plus the sub-ports `Outbox`,
   `AuditLog`, `Idempotency`, `EventLog` (see `docs/ports/storage.md`).
4. `smsengine-storage-postgres`: full impl. `include_str!`s
   `migrations/engine/0000_engine_core.postgres.sql` for the 6
   cross-cutting tables. Walks the macro-emitted AST to render the
   ~310 domain tables at `create_schema()` time. RLS policies via
   `CREATE POLICY`. `sqlx` + `rustls`.
5. Integration test: spin up a PG container (testcontainers), call
   `storage.create_schema().await`, insert one outbox row via the
   `Outbox` sub-port, read it back, assert the engine invariants
   (`school_id NOT NULL`, UUID column `CHAR(36)`, etc.) and that the
   emitted DDL byte-matches
   `migrations/engine/0000_engine_core.postgres.sql` for the 6
   cross-cutting tables.

**Exit criteria.**

1. `cargo build --workspace` green.
2. `cargo test -p smsengine-storage-postgres` green; the outbox
   e2e test passes.
3. The outbox DDL emitted by the adapter byte-matches
   `migrations/engine/0000_engine_core.postgres.sql`.
4. `cargo clippy --workspace --all-targets -- -D warnings` green.
5. `cargo fmt --all -- --check` green.

**Coverage matrix updates.** Rows that flip from Pending to
Implemented: `outbox table DDL (PG)`, `idempotency table DDL`,
`schema_registry table DDL`, `system_user table DDL`, `DomainQuery
macro`, `StorageAdapter port`.

**Risks.**

- *Macro complexity.* The proc macro is the most concentrated source
  of complexity in the engine. Mitigation: build it in two steps
  (struct → descriptor; descriptor → DDL), with a unit test per step.
- *Testcontainers in CI.* PG container startup adds 5–10 s per CI
  run. Mitigation: a `SqliteStorage` fast-path test in Phase 1;
  full PG e2e only on nightly.
- *UUID v7.* Rust's `uuid` crate added v7 in 1.10. Mitigation: pin
  `uuid >= 1.10` in workspace `Cargo.toml`; document the MSRV impact
  (still 1.75).

---

## Phase 1 — Adapter parity (MySQL + SQLite)

**Deliverables.** `smsengine-storage-mysql`, `smsengine-storage-sqlite`.
The same outbox scenario from Phase 0 runs in all three adapters.

**Tasks.**

1. `smsengine-storage-mysql`: full impl. `include_str!`s
   `migrations/engine/0000_engine_core.mysql.sql`. `MySQL 8.0+`
   `utf8mb4_unicode_ci`, `ENGINE=InnoDB`, `JSON`, `CHAR(36)`,
   backtick identifier quoting. RLS not native — emulate via session
   variable `SET @app_tenant_id = ?` + `WHERE school_id = @app_tenant_id`
   on every query (per `docs/schemas/sql-dialects/mysql.md`).
2. `smsengine-storage-sqlite`: full impl. `include_str!`s
   `migrations/engine/0000_engine_core.sqlite.sql`. `TEXT` with
   `CHECK(length() = 36)` for UUIDs, `INTEGER` for booleans, ISO
   8601 `TEXT` for timestamps, no RLS, no schema namespaces. JSON
   via the `json1` extension at the application layer.
3. Cross-adapter test: a single integration test that runs the
   Phase 0 outbox scenario against all three adapters and asserts
   the DDL emitted for the 6 cross-cutting tables is byte-identical
   modulo dialect syntax (whitespace, identifier quoting, type
   substitutions documented in `comparison.md`).

**Exit criteria.**

1. `cargo test -p smsengine-storage-mysql` green.
2. `cargo test -p smsengine-storage-sqlite` green.
3. The cross-adapter test passes on all three adapters.
4. `cargo test --workspace` green.

**Coverage matrix updates.** `outbox table DDL (MySQL)`, `outbox
table DDL (SQLite)`, plus the MySQL/SQLite variants of all 6
cross-cutting tables. (One row per table per dialect in the matrix;
this phase flips 12 rows.)

**Risks.**

- *MySQL `CHECK` constraints.* Enforced only from 8.0.16. Mitigation:
  document the floor in `mysql.md`; gate the test on `>= 8.0.16`.
- *SQLite single-writer.* Concurrent writes serialize. Mitigation:
  document this as a deployment constraint; not a correctness
  concern for the adapter itself.

---

## Phase 2 — Cross-cutting foundations: `platform` + `rbac` + `events` + `audit`

**Deliverables.** `smsengine-platform`, `smsengine-rbac`,
`smsengine-events`, `smsengine-event-bus`, `smsengine-audit`. The 6
cross-cutting tables (`outbox`, `audit_log`, `event_log`,
`idempotency`, `schema_registry`, `system_user`) are all exercised
end-to-end.

**Tasks.**

1. `smsengine-platform`: `School`, `User`, `SchoolId`, `UserId`,
   `TenantContext`. Spec is in `docs/specs/platform/`.
2. `smsengine-rbac`: `Capability`, `Role`, `Permission`, the
   capability check port, the default role catalog, `is_replicated`
   flag for distributed deployments. Spec is in `docs/specs/rbac/`.
3. `smsengine-events`: the **envelope** crate. `DomainEvent` trait,
   `EventEnvelope` (event_id, correlation_id, causation_id, occurred_at,
   payload), `EventBus` trait. **Not** the calendar domain (that's
   `smsengine-events-domain` in Phase 13).
4. `smsengine-event-bus`: in-process, NATS, Redis impls behind the
   `EventBus` port (per `docs/ports/event-bus.md`).
5. `smsengine-audit`: the audit log writer
   (`AuditLogEntry { actor, action, target, before, after, occurred_at,
   correlation_id }`), retention policies (configurable
   `retention_days`; engine emits a `retention_sweep_due` event when
   the policy threshold is reached), and the audit write path
   (called from every command handler in the engine).
6. Integration test: create a school, create a user, create a role,
   emit a `SchoolCreated` event from a platform command. Verify:
   - `outbox` has the event.
   - `event_log` has the delivered entry (in-process bus).
   - `audit_log` has the command audit entry.
   - `idempotency` has the command's idempotency key.
   - `schema_registry` has a row recording the schema version.

**Exit criteria.**

1. All 6 cross-cutting tables exercised in the integration test.
2. Outbox + audit_log + event_log all populated by a single
   command.
3. RLS is enforced on PG (the test uses a second `school_id` and
   asserts cross-tenant reads return zero rows).
4. `cargo test --workspace` green.

**Coverage matrix updates.** `audit_log table DDL` (all 3
dialects), `event_log table DDL` (all 3 dialects), `EventBus port`.
Plus all platform / rbac / events / events-domain / audit
aggregates, commands, and events listed in their respective spec
catalogs.

**Risks.**

- *RLS bypass via superuser.* PG superusers bypass RLS by default.
  Mitigation: the test uses a non-superuser role; document this in
  the deployment guide.
- *Audit log volume.* Every command writes one audit row. At 10k
  students × 5 daily commands × 200 schools = 10M rows/day.
  Mitigation: partition by `school_id` + month; document the
  partitioning strategy in `docs/schemas/audit-schema.md`.

---

## Phase 3 — Academic domain (first vertical slice)

**Deliverables.** `smsengine-academic`. The largest domain, exercises
the most code paths (student lifecycle, enrollment, promotion, class/
section management, subject assignment, academic year rollover).

**Tasks.**

1. `crates/academic/src/{aggregate.rs, entities.rs, value_objects.rs,
   commands.rs, events.rs, services.rs, policies.rs, repository.rs,
   query.rs, errors.rs}` plus `tests/`. One `#[derive(DomainQuery)]`
   per aggregate documented in `docs/specs/academic/aggregates.md`.
2. Aggregates: `Student`, `Guardian`, `Class`, `Section`, `Subject`,
   `AcademicYear`, `Enrollment`, `Promotion`, plus any
   relation/value-object rows in `docs/specs/academic/tables.md` not
   covered by the eight primary aggregates.
3. Commands per `docs/specs/academic/commands.md` and
   `docs/commands/academic.md`. Each emits the events listed in
   `docs/specs/academic/events.md` and `docs/events/academic.md`.
4. Repository port in `repository.rs`; the per-backend
   `smsengine-storage-<db>` crates provide the impl.
5. Integration test: end-to-end vertical slice — admit a student →
   assign to class/section → record attendance (via a stub command;
   full attendance impl is Phase 5) → mark an exam (stub; full
   assessment impl is Phase 4) → verify `outbox` has the
   `StudentAdmitted`, `EnrollmentCreated`, `AttendanceRecorded`,
   `ExamMarked` events in order; verify `audit_log` has one row per
   command; verify RLS blocks a cross-tenant read.

**Exit criteria.**

1. Every aggregate in `docs/specs/academic/aggregates.md` has a
   corresponding Rust struct with `#[derive(DomainQuery)]`.
2. Every command in `docs/commands/academic.md` has a handler.
3. Every event in `docs/events/academic.md` has a Rust enum variant.
4. The vertical-slice integration test passes against PG, MySQL, and
   SQLite.
5. `cargo test -p smsengine-academic` green.
6. `cargo clippy -p smsengine-academic --all-targets -- -D warnings`
   green.

**Coverage matrix updates.** All `academic_*` aggregate, command,
and event rows. (One row per aggregate in
`docs/specs/academic/aggregates.md`, one per command in
`docs/commands/academic.md`, one per event in
`docs/events/academic.md`.)

**Risks.**

- *Academic is the largest domain.* A naïve port from the legacy
  Schoolify schema can take 6+ weeks. Mitigation: split into
  sub-slices (Student/Guardian/Enrollment first; Class/Section/
  Subject next; AcademicYear/Promotion last) and ship each as a
  separately mergeable PR.
- *Promotion logic.* End-of-year promotion is the most complex
  service in the domain (carry-forward rules, detention logic,
  board-exam exemptions). Mitigation: prototype the policy module
  in `policies.rs` against hand-rolled fixtures before connecting
  to the repository.

---

## Phase 4 — Assessment

**Deliverables.** `smsengine-assessment`. Exams, marks, results,
online exams, seat plans, admit cards, report cards.

**Tasks.**

1. Aggregates per `docs/specs/assessment/aggregates.md`:
   `Exam`, `ExamSchedule`, `MarksRegister`, `ResultStore`,
   `ReportCard`, `OnlineExam`, `SeatPlan`, `AdmitCard`.
2. Commands per `docs/commands/assessment.md`; events per
   `docs/events/assessment.md`.
3. Services: result computation (GPA, grade, merit position),
   report-card PDF generation (delegated to `smsengine-files` port).
4. Integration test: schedule an exam, enter marks, compute result,
   publish report card. Verify outbox + audit + RLS.

**Exit criteria.**

1. Every aggregate in `docs/specs/assessment/aggregates.md` has a
   Rust struct + tests.
2. The result-computation service has a unit test per grading rule
   in `docs/specs/assessment/services.md`.
3. `cargo test -p smsengine-assessment` green.

**Coverage matrix updates.** All `assessment_*` rows.

**Risks.** *Result computation is policy-heavy.* Mitigation: keep
all grading rules in `policies.rs` as pure functions with table-
driven fixtures.

---

## Phase 5 — Attendance

**Deliverables.** `smsengine-attendance`. Student, staff, subject,
exam attendance.

**Tasks.**

1. Aggregates per `docs/specs/attendance/aggregates.md`:
   `StudentAttendance`, `StaffAttendance`, `SubjectAttendance`,
   `ExamAttendance`.
2. Bulk-marking command (CSV import + per-class UI). The
   `smsengine-storage` bulk-insert path is exercised here for the
   first time at scale.
3. Integration test: bulk-mark attendance for a class-section of 200
   students in a single command. Verify outbox emits one
   `AttendanceRecorded` per student and one `ClassAttendanceClosed`
   aggregate event.

**Exit criteria.** As Phases 3–4, plus a bulk-insert benchmark
(200 rows in <100 ms on PG).

**Coverage matrix updates.** All `attendance_*` rows.

**Risks.** *Bulk insert performance.* Mitigation: use a single
multi-row `INSERT` (PG) or transaction-grouped inserts (SQLite);
add a benchmark in `tests/benches/`.

---

## Phase 6 — HR

**Deliverables.** `smsengine-hr`. Staff, department, designation,
leave, payroll.

**Tasks.**

1. Aggregates per `docs/specs/hr/aggregates.md`:
   `Staff`, `Department`, `Designation`, `LeaveType`, `LeaveRequest`,
   `Payroll`.
2. Leave accrual service; payroll computation service (depends on
   `smsengine-finance` for the chart-of-accounts write — mock that
   dep in tests).
3. Integration test: hire a staff member, request leave, approve it,
   run payroll. Verify outbox + audit + RLS.

**Exit criteria.** As Phases 3–4. The payroll test uses a mocked
finance port; real wiring is Phase 15.

**Coverage matrix updates.** All `hr_*` rows.

**Risks.** *Payroll is regulatory.* Mitigation: explicitly
document in `services.md` that the engine provides the computation
primitives; legal/tax-rule configuration is the consumer's
responsibility.

---

## Phase 7 — Finance

**Deliverables.** `smsengine-finance`. The largest spec
(~5,567 lines). Fees (group, type, master, assign, discount,
invoice, installment, payment), bank (account, statement), expense,
income, wallet, payroll accounting.

**Tasks.**

1. Aggregates per `docs/specs/finance/aggregates.md`:
   `FeesGroup`, `FeesType`, `FeesMaster`, `FeesAssign`,
   `FeesDiscount`, `FeesInvoice`, `FeesInstallment`, `FeesPayment`,
   `BankAccount`, `BankStatement`, `Expense`, `Income`, `Wallet`,
   `Payroll` (the accounting-side payroll record, distinct from the
   HR-side `Payroll`).
2. Services: carry-forward rules, late-fee computation,
   collection-report aggregation, double-entry booking
   (debit/credit invariant).
3. Integration test: configure a fees master, assign to a class,
   generate invoices for a term, accept a payment via the mocked
   payment port, produce a collection report. Verify double-entry
   invariant (`sum(debits) == sum(credits)` per school_id).

**Exit criteria.**

1. Every aggregate in `docs/specs/finance/aggregates.md` has a
   Rust struct + tests.
2. The double-entry invariant is enforced by a property test
   (proptest) — not just example-based.
3. The carry-forward service has a unit test per rule in
   `docs/specs/finance/services.md`.
4. `cargo test -p smsengine-finance` green.

**Coverage matrix updates.** All `finance_*` rows.

**Risks.** *Money is real.* Mitigation: the engine never holds a
raw float in a money column. All amounts are `MinorUnits` (i64
cents/paisa). The `as` ban (per `AGENTS.md`) is enforced
`#[forbid]`-style on the finance crate via a custom clippy lint.

---

## Phase 8 — Facilities

**Deliverables.** `smsengine-facilities`. Dormitory, room, transport
(route, vehicle), inventory (item, category, store, issue, receive,
sell), supplier.

**Tasks.**

1. Aggregates per `docs/specs/facilities/aggregates.md`:
   `Dormitory`, `Room`, `Route`, `Vehicle`, `Item`, `ItemCategory`,
   `ItemStore`, `ItemIssue`, `ItemReceive`, `ItemSell`, `Supplier`.
2. Inventory movement service (issue/receive/sell must conserve
   `on_hand = sum(received) - sum(issued) - sum(sold)`).
3. Integration test: receive 100 items, issue 30, sell 5; verify
   `on_hand == 65` after.

**Exit criteria.** As Phases 3–4, plus the conservation invariant
test.

**Coverage matrix updates.** All `facilities_*` rows.

**Risks.** *Inventory conservation under concurrent writes.*
Mitigation: the service runs in a transaction with
`SELECT ... FOR UPDATE` on the `ItemStore` row (PG) or a SQLite
write lock.

---

## Phase 9 — Library

**Deliverables.** `smsengine-library`. Book, book category, library
member, book issue, book return, fine.

**Tasks.**

1. Aggregates per `docs/specs/library/aggregates.md`: `Book`,
   `BookCategory`, `LibraryMember`, `BookIssue`, `BookReturn`,
   `Fine`.
2. Integration test: catalog a book, issue it to a student, return
   it 5 days late, assess the fine.

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `library_*` rows.

---

## Phase 10 — Communication

**Deliverables.** `smsengine-communication`. Notice, complaint,
chat message, email log, SMS log, notification setting.

**Tasks.**

1. Aggregates per `docs/specs/communication/aggregates.md`:
   `Notice`, `Complaint`, `ChatMessage`, `EmailLog`, `SmsLog`,
   `NotificationSetting`.
2. Notification dispatch service — consumes domain events and
   delivers via the `NotificationProvider` port (real impl is
   Phase 15).
3. Integration test: a `StudentAbsent` event triggers an SMS log
   entry (the actual SMS send is mocked at the port boundary).

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `communication_*` rows.

---

## Phase 11 — Documents

**Deliverables.** `smsengine-documents`. Form download, postal
dispatch, postal receive.

**Tasks.**

1. Aggregates per `docs/specs/documents/aggregates.md`:
   `FormDownload`, `PostalDispatch`, `PostalReceive`.
2. File attachments go through the `FileStorage` port (real impl is
   Phase 15).
3. Integration test: upload a form, count a download, dispatch a
   postal item, mark it received.

**Exit criteria.** As Phases 3–4.

**Coverage matrix updates.** All `documents_*` rows.

---

## Phase 12 — CMS

**Deliverables.** `smsengine-cms`. Page, news, notice (distinct
from `smsengine-communication`'s `Notice`), testimonial.

**Tasks.**

1. Aggregates per `docs/specs/cms/aggregates.md`: `Page`, `News`,
   `Notice`, `Testimonial`.
2. Slug generation, publish/draft workflow.
3. Integration test: create a draft page, publish it, fetch via the
   public query (RLS must NOT block public reads — use a special
   `school_id` for public content).

**Exit criteria.** As Phases 3–4, plus the public-read test.

**Coverage matrix updates.** All `cms_*` rows.

**Risks.** *CMS reads cross tenant.* The public-page fetch must
work across schools. Mitigation: a `school_id` of zero
(`00000000-...`) is reserved for public content; RLS policies
explicitly allow it (per `docs/schemas/tenancy-schema.md`).

---

## Phase 13 — Events domain (calendar)

**Deliverables.** `smsengine-events-domain`. **Distinct** from
`smsengine-events` (the envelope crate from Phase 2). This is the
calendar domain: `CalendarEvent`, `Holiday`, `Incident`, `Weekend`.

**Tasks.**

1. Aggregates per `docs/specs/events/aggregates.md`:
   `CalendarEvent`, `Holiday`, `Incident`, `Weekend`.
2. Recurrence rule service (RFC 5545 RRULE subset).
3. Integration test: create a weekly recurring event, generate
   instances for a date range, exclude a holiday.

**Exit criteria.** As Phases 3–4, plus the RRULE test.

**Coverage matrix updates.** All `events_domain_*` rows.

**Risks.** *The two `events` crates are easy to confuse.*
Mitigation: `crates/events/` is the envelope; `crates/events-domain/`
is the calendar. Document this explicitly in both `lib.rs` headers
and in `AGENTS.md`.

---

## Phase 14 — Settings + Operations

**Deliverables.** `smsengine-settings`, `smsengine-operations`.

**Tasks.**

1. `smsengine-settings`: per-school configuration, language phrases,
   base setups. Aggregates per `docs/specs/settings/aggregates.md`.
2. `smsengine-operations` (new in v1): school-day operations —
   `AcademicSession`, `BellSchedule`, `Substitution`,
   `TimetableChange`, `DailyDiary`. Aggregates per
   `docs/specs/operations/aggregates.md`.
3. Integration tests per domain, as in Phases 3–4.

**Exit criteria.** As Phases 3–4, for both crates.

**Coverage matrix updates.** All `settings_*` and `operations_*`
rows.

---

## Phase 15 — Port adapters

**Deliverables.** `smsengine-auth`, `smsengine-notify`,
`smsengine-payment`, `smsengine-files`, `smsengine-integrations`.
Port trait **plus** one reference impl per port.

**Tasks.**

1. `smsengine-auth`: the `AuthProvider` port
   (per `docs/ports/authentication.md`) + a `JwtAuthProvider`
   reference impl.
2. `smsengine-notify`: the `NotificationProvider` port + email and
   SMS reference impls.
3. `smsengine-payment`: the `PaymentProvider` port + a Stripe
   reference impl.
4. `smsengine-files`: the `FileStorage` port + S3 and local
   reference impls.
5. `smsengine-integrations`: the `IntegrationGateway` port + LMS
   and video-conferencing reference impls.
6. For each port, an integration test that wires a real reference
   impl against a docker-compose stack (mailhog, localstack S3,
   stripe-mock, etc.).

**Exit criteria.**

1. All 5 port traits have a Rust trait definition and a reference
   impl.
2. `Box<dyn NotificationProvider>` (and the other four ports)
   compiles — verifying object safety.
3. Each reference impl has a green integration test.
4. `cargo test --workspace` green.

**Coverage matrix updates.** `AuthProvider port`, `NotificationProvider
port`, `PaymentProvider port`, `FileStorage port`, `IntegrationGateway
port`. Plus all reference-impl test rows.

**Risks.**

- *Stripe API drift.* Mitigation: pin the stripe-mock version; the
  reference impl is a thin client over the typed API, not a
  reflection of the wire format.
- *S3 SDK weight.* The `aws-sdk-s3` crate is large. Mitigation:
  feature-gate it; consumers who only need the local impl don't
  pay the binary-size cost.

---

## Phase 16 — Test infrastructure + SDK

**Deliverables.** `smsengine-testkit`, `smsengine-storage-parity`,
`smsengine-sdk`, `smsengine-cli`.

**Tasks.**

1. `smsengine-testkit`: in-memory impls of all 6 ports
   (`StorageAdapter`, `AuthProvider`, `NotificationProvider`,
   `PaymentProvider`, `FileStorage`, `EventBus`). Consumer tests use
   these to run domain commands without docker.
2. `smsengine-storage-parity`: a cross-adapter parity test suite
   that runs the same scenario against PG, MySQL, SQLite, and the
   in-memory testkit impl, asserting identical observable behavior
   (modulo documented dialect differences).
3. `smsengine-sdk`: a high-level consumer facade — `Engine::builder()`
   wires the umbrella crate's re-exports into a single
   configuration surface. The SDK is the public face of the engine
   for the consumer (`docs/library-docs.md`).
4. `smsengine-cli`: a sample binary demonstrating daily operations
   (admit a student, mark attendance, record a payment) for
   developer ergonomics and dogfooding.
5. A consumer-facing integration test in
   `crates/smsengine/tests/consumer_e2e.rs` that uses the SDK +
   testkit to run a full admission workflow without docker.

**Exit criteria.**

1. `smsengine-testkit` ports compile and pass their own unit tests.
2. The parity suite runs in <60 s on a developer laptop and is
   green on all four backends.
3. The CLI binary builds and the three sample commands work
   end-to-end against an in-memory backend.
4. `cargo test --workspace` green.

**Coverage matrix updates.** All port impls (`AuthProvider
impl: jwt`, `NotificationProvider impl: email`, etc.) and the
testkit/parity/sdk/cli test rows.

**Risks.** *Parity suite flakiness across backends.* Mitigation:
the suite asserts against a documented behavior matrix, not
against byte-identical SQL output. Differences in error messages
between PG and MySQL are tolerated.

---

## Phase 17 — Production readiness

**Deliverables.** Integration test suite, load test, cross-compile,
security review, documentation audit.

**Tasks.**

1. Multi-tenant integration test suite — 50+ scenarios from
   `docs/guides/saas-backend.md`, run nightly against all three
   backends.
2. Load test: 10k students, bulk fee invoice generation (Phase 7
   finance). Target: p95 < 500 ms for a bulk-invoice-of-10k-rows
   command on PG; documented in `docs/research/load-test-results.md`.
3. Cross-compile verification on Linux x86_64, Linux aarch64,
   macOS x86_64, macOS aarch64, Windows x86_64. CI matrix runs all
   five.
4. Security review of every public command surface. For each
   command in `docs/commands/<domain>.md`, verify:
   - The handler reads the `TenantContext` and asserts the
     `school_id` matches the command's `school_id`.
   - The RBAC capability is checked.
   - Idempotency is enforced for mutating commands.
5. Documentation audit against the 10-point validation checklist
   in `AGENTS.md`. Every question must answer "Yes".

**Exit criteria.**

1. All 10 validation questions in `AGENTS.md` answer "Yes".
2. `cargo build --workspace --target x86_64-unknown-linux-gnu`,
   `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`,
   `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` all green.
3. CI green on all five targets.
4. Load-test report committed under `docs/research/`.
5. Security-review report committed under `docs/decisions/`.

**Coverage matrix updates.** All remaining rows flip to
Implemented. The matrix reaches 100%.

**Risks.** *Cross-compile surprises (Windows path handling,
musl allocator).* Mitigation: smoke-test the SDK on each target in
Phase 16, before Phase 17 hardens the matrix.

---

## The Coverage Matrix

The full matrix has 226+ rows: one per implementable doc, one per
table for the 6 cross-cutting tables × 3 dialects, one per port
trait × impl. It lives in **machine-readable form** at
[`docs/coverage.toml`](coverage.toml) so CI can diff it. The
build-plan.md keeps a representative sample of the schema below;
the authoritative source is the TOML file.

The matrix has the following columns:

| Column   | Type            | Meaning |
| -------- | --------------- | ------- |
| `id`     | string          | Stable identifier, e.g. `outbox_ddl_pg` |
| `item`   | string          | Human-readable name |
| `spec`   | path (string)   | Spec doc that defines the item |
| `crate`  | string          | `smsengine-<name>` package that owns the impl |
| `phase`  | integer 0..17   | Build-plan phase that delivers the impl |
| `status` | enum            | `Pending` \| `Implemented` \| `Tested` \| `Deprecated` |
| `tests`  | path (string)?   | Integration-test path that exercises the impl (set when status >= `Tested`) |
| `notes`  | string?         | Free-form note |

The TOML schema is grouped by item kind:

```toml
[[row]]   id = "outbox_ddl_pg"        item = "outbox table DDL (PG)"        spec = "migrations/engine/0000_engine_core.postgres.sql" crate = "smsengine-storage-postgres" phase = 0  status = "Pending"
[[row]]   id = "outbox_ddl_mysql"     item = "outbox table DDL (MySQL)"     spec = "migrations/engine/0000_engine_core.mysql.sql"   crate = "smsengine-storage-mysql"    phase = 1  status = "Pending"
[[row]]   id = "outbox_ddl_sqlite"    item = "outbox table DDL (SQLite)"    spec = "migrations/engine/0000_engine_core.sqlite.sql"  crate = "smsengine-storage-sqlite"   phase = 1  status = "Pending"
[[row]]   id = "audit_log_ddl_pg"     item = "audit_log table DDL (PG)"     spec = "migrations/engine/0000_engine_core.postgres.sql" crate = "smsengine-audit"            phase = 2  status = "Pending"
# ... 222 more rows ...
[[row]]   id = "academic_students_aggregate"   item = "academic_students aggregate"   spec = "docs/specs/academic/aggregates.md"   crate = "smsengine-academic"   phase = 3  status = "Pending"
[[row]]   id = "student_admitted_event"        item = "StudentAdmitted event"         spec = "docs/events/academic.md"             crate = "smsengine-academic"   phase = 3  status = "Pending"
[[row]]   id = "admit_student_command"         item = "AdmitStudent command"          spec = "docs/commands/academic.md"           crate = "smsengine-academic"   phase = 3  status = "Pending"
# ... etc
```

See [`docs/coverage.toml`](coverage.toml) for the initial scaffold
(~80 representative rows). The full 226+ row matrix is generated
by the lint sub-module (§ [The No-Gaps Gates](#the-no-gaps-gates))
from the spec catalogs and the code inventory.

### How the matrix is kept in sync

1. **Adding an item to a spec** → add a row to `docs/coverage.toml`
   with `status = "Pending"`. The lint sub-module (Phase 0 work)
   will not fail until the spec is implemented, but a new `Pending`
   row is itself a flagged entry that the next PR must address.
2. **Implementing the item in code** → update the row's `status`
   to `Implemented` in the same commit as the implementation.
3. **Adding the integration test** → update the row's `status` to
   `Tested` and set `tests` to the test path. `Tested` is the
   terminal state; it is what the per-PR gate validates.
4. **Deprecating an item** → set `status = "Deprecated"` and add a
   note pointing to the replacement. Deprecated rows are exempt
   from the per-PR gate.

The CI step (§ [The No-Gaps Gates](#the-no-gaps-gates) item 3)
verifies that:

- Every row marked `Tested` has code that exists and a test path
  that exists.
- Every code-defined aggregate/command/event has a row.
- No row references a spec file, command catalog, or event catalog
  that doesn't exist.

---

## The No-Gaps Gates

Three mechanisms enforce the coverage invariant — that every
spec'd item is implemented and every implemented item is spec'd.

### 1. Hand-written integration tests in `crates/<domain>/tests/`

For every domain crate, the integration test directory
`crates/<domain>/tests/` contains hand-written tests that exercise
the spec'd behavior. Each test:

- Validates a real-world scenario (round-trip serialization, error
  propagation, trait object dispatch, multi-tenant isolation, etc.)
- Covers the happy path **and** at least one error path
  (per `AGENTS.md` § Agent Instructions → Testing).
- References the spec doc it implements by comment header:
  `// Implements: docs/specs/academic/aggregates.md#student-admit`

This is the **per-domain gate** — it runs in `cargo test -p
<domain>` and catches drift between the spec doc and the actual
behavior of the command/event/aggregate. Tests are authored by
hand, not generated by the macro, so they can exercise scenarios
the macro AST does not capture (e.g. side-effects, async
interactions, port adapter wiring).

Conventions for the test files:

| File                       | What it tests                                  |
| -------------------------- | ---------------------------------------------- |
| `crates/<d>/tests/aggregate_fields.rs` | Field-level invariants from `aggregates.md`     |
| `crates/<d>/tests/commands.rs`         | Command handlers from `commands.md`             |
| `crates/<d>/tests/events.rs`           | Event envelopes from `events.md`                |
| `crates/<d>/tests/services.rs`         | Domain services from `services.md`              |
| `crates/<d>/tests/repository.rs`       | Repository port methods from `repositories.md`  |
| `crates/<d>/tests/value_objects.rs`    | Value-object validation from `value-objects.md` |
| `crates/<d>/tests/workflows.rs`        | Multi-aggregate workflows from `workflows.md`   |

### 2. Cross-reference lint (`smsengine-core::lint`)

A sub-module of `smsengine-core` (not a separate crate), enabled
via the `lint` Cargo feature flag in `smsengine-core`. The lint
sub-module is a CLI binary that:

1. Walks the repo and verifies the **spec → code** direction:
   - Every `docs/specs/<domain>/tables.md` row has a corresponding
     `#[derive(DomainQuery)]` struct in
     `crates/<domain>/src/aggregate.rs` (matched by table name).
   - Every `docs/commands/<domain>.md` entry has a corresponding
     handler in `crates/<domain>/src/commands.rs` (matched by
     command name).
   - Every `docs/events/<domain>.md` entry has a corresponding
     event in `crates/<domain>/src/events.rs` (matched by event
     name).
   - Every `migrations/engine/*.sql` table has a corresponding
     `create_<table>_ddl()` function in each adapter crate (or is
     covered by the `include_str!`'d core file).
2. Walks the repo and verifies the **code → spec** direction: every
   public struct, command, and event has a spec row. The lint fails
   on undocumented public items.
3. **Anti-patterns**:
   - No `unimplemented!()`, `todo!()`, or `// TODO: implement` in
     production code (test code is exempt via `#[cfg(test)]`
     detection).
   - No `as` on numerics in domain crates (per `AGENTS.md`'s `as`
     ban).
   - No `serde_json::Value` in domain code.
   - No `HashMap<String, T>` for domain data.
4. **Parity**: every `DomainQuery` macro call has a corresponding
   spec row, and every spec row has a corresponding macro call.
5. **Coverage matrix sync**: the lint reads `docs/coverage.toml`
   and verifies:
   - Every `Tested` row has a `tests` path that exists.
   - Every code-defined aggregate/command/event has a row.
   - No row references a spec file, command catalog, or event
     catalog that doesn't exist.

This is the **per-crate gate** — it runs in CI (and locally via
`cargo run -p smsengine-core --bin lint --features lint`) and
catches missing handlers, anti-patterns, reverse-direction drift,
and matrix lies.

Putting the lint inside `smsengine-core` (rather than as a separate
crate) keeps the workspace at 34 crates and makes the lint
implementation a Phase 0 deliverable alongside the other core
primitives.

### 3. Coverage-matrix CI check (machine-readable TOML)

Because the matrix lives at `docs/coverage.toml`, the CI step is:

1. `git diff --exit-code docs/coverage.toml` on PRs that touch
   `docs/specs/` or `crates/<d>/` — the matrix MUST be updated in
   the same commit as the spec change or the implementation
   change. A PR that adds an aggregate without a matrix row fails.
2. The lint sub-module (§ 2 above) re-validates the committed
   matrix on every CI run.
3. The matrix is the **single source of truth** for "is item X
   implemented?" — no need to grep code or read 14 progress
   tracker tables.

This is the **per-PR gate** — it runs on every pull request.

### Combined effect

| Layer  | Gate                                   | When it runs    | What it catches |
| ------ | -------------------------------------- | --------------- | --------------- |
| Domain | hand-written tests in `crates/<d>/tests/` | `cargo test`    | Drift between spec and actual behavior |
| Crate  | `smsengine-core::lint` (feature-gated) | `cargo build` (CI) | Missing handlers, anti-patterns, reverse drift, matrix lies |
| Repo   | TOML matrix diff in CI                 | every PR        | Spec or impl change without matrix update |

Together, the three gates make it impossible to merge a PR that
silently drops a spec'd feature, leaves a `todo!()` in production
code, or claims implementation without updating the matrix.

---

## Build Order (One-Page)

```text
0. Foundation       — core, query-derive, storage port, storage-postgres + outbox e2e
1. Adapter parity   — storage-mysql, storage-sqlite + cross-adapter test
2. Cross-cutting    — platform, rbac, events envelope, event-bus, audit
3. Academic         — first domain vertical slice (largest)
4. Assessment       — exams, marks, results, report cards
5. Attendance       — student/staff/subject/exam attendance
6. HR               — staff, leave, payroll
7. Finance          — fees, banking, expenses, wallet, double-entry invariant
8. Facilities       — dormitory, transport, inventory
9. Library          — books, issues, returns, fines
10. Communication   — notices, complaints, email/SMS logs
11. Documents       — forms, postal dispatch/receive
12. CMS             — pages, news, notices, testimonials
13. Events domain   — calendar, holidays, incidents, weekends (events-domain crate)
14. Settings + Ops  — settings, operations (new in v1)
15. Port adapters   — auth, notify, payment, files, integrations + reference impls
16. Test + SDK      — testkit, storage-parity, sdk, cli
17. Production      — multi-tenant tests, load test, cross-compile, security, docs audit
```

---

## See also

- [`docs/progress-tracker.md`](progress-tracker.md) — per-crate
  implementation status (one row per crate, updated weekly).
- [`docs/schemas/sql-dialects/README.md`](schemas/sql-dialects/README.md#runtime-ddl-emission--end-to-end-flow)
  § "Runtime DDL emission — end-to-end flow" — the five-step
  schema-emission flow that the storage adapters implement.
- [`docs/architecture.md`](architecture.md) — the system map
  (crate dependency graph, request lifecycle, event flow).
- [`migrations/engine/README.md`](../migrations/engine/README.md) —
  the canonical DDL files for the 6 cross-cutting tables in all
  three dialects.
- [`AGENTS.md`](../AGENTS.md) — workspace layout, dependency rules,
  validation checklist, naming conventions.
- [`docs/code-standards.md`](code-standards.md) — the engineering
  rules every implementation must follow.
- [`docs/query_layer.md`](query_layer.md) — the macro-driven query
  specification consumed by `smsengine-query-derive`.
- [`docs/specs/<domain>/overview.md`](specs/) — per-domain
  specifications (15 domains, 11 files each).
- [`docs/ports/*.md`](ports/) — port contracts (7 ports).
- [`docs/commands/<domain>.md`](commands/) — command catalogs (15
  domains).
- [`docs/events/<domain>.md`](events/) — event catalogs (15
  domains).
- [`docs/guides/saas-backend.md`](guides/saas-backend.md) —
  production SaaS guide (multi-tenant scenarios used by the
  Phase 17 test suite).
