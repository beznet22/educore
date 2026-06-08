# SMScore Build Plan

## Phases

SMScore is implemented in eight sequential phases. Each phase produces a
reviewable, testable, and demonstrably useful increment.

## Phase 1 — Foundation

Deliverables:

- Cargo workspace skeleton.
- `smscore-core` crate providing shared types (errors, identifiers, value
  objects, result type, clock, id generator).
- `smscore-platform` crate providing `School`, `User`, `SchoolId`, `UserId`,
  `TenantContext`.
- `smscore-rbac` crate providing `Capability`, `Role`, `Permission`, the
  capability check port, and the default role catalog.
- `smscore-settings` crate providing general settings, language phrases,
  base setups.
- `smscore-events` crate providing `DomainEvent`, `EventEnvelope`,
  `EventBus` trait, in-process bus implementation.

Exit criteria:

- A consumer can construct an engine, register no domains, and resolve
  `Engine::capabilities().has("platform.tenant.create")`.
- All tests in foundation crates pass.

## Phase 2 — Domain Layer (Core)

Deliverables:

- `smscore-academic` with `Student`, `Guardian`, `Class`, `Section`,
  `Subject`, `AcademicYear`, `Enrollment`, `Promotion`.
- `smscore-hr` with `Staff`, `Department`, `Designation`, `LeaveType`,
  `LeaveRequest`, `Payroll`.
- `smscore-library` with `Book`, `BookCategory`, `LibraryMember`,
  `BookIssue`.
- `smscore-events` domain with `CalendarEvent`, `Holiday`, `Incident`.
- `smscore-documents` with `FormDownload`, `PostalDispatch`, `PostalReceive`.
- `smscore-cms` with `Page`, `News`, `Notice`, `Testimonial`.

Exit criteria:

- Every aggregate documented under `docs/specs/<domain>/aggregates.md` has
  Rust code, repository port, and command catalog implementation.
- Domain tests cover happy path and at least two error paths per command.

## Phase 3 — Attendance, Assessment, Communication, Facilities

Deliverables:

- `smscore-attendance` with `StudentAttendance`, `StaffAttendance`,
  `SubjectAttendance`, `ExamAttendance`.
- `smscore-assessment` with `Exam`, `ExamSchedule`, `MarksRegister`,
  `ResultStore`, `ReportCard`, `OnlineExam`, `SeatPlan`, `AdmitCard`.
- `smscore-communication` with `Notice`, `Complaint`, `ChatMessage`,
  `EmailLog`, `SmsLog`, `NotificationSetting`.
- `smscore-facilities` with `Dormitory`, `Room`, `Route`, `Vehicle`,
  `Item`, `ItemCategory`, `ItemStore`, `ItemIssue`, `ItemReceive`,
  `ItemSell`, `Supplier`.

Exit criteria:

- Attendance can be marked for a class-section in a single command and
  produces a domain event consumed by the communication port to notify
  absent students' guardians.
- Marks can be entered, results computed (GPA, grade, merit position),
  and report cards published, all with the appropriate audit trail.

## Phase 4 — Finance

Deliverables:

- `smscore-finance` with `FeesGroup`, `FeesType`, `FeesMaster`, `FeesAssign`,
  `FeesDiscount`, `FeesInvoice`, `FeesInstallment`, `FeesPayment`,
  `BankAccount`, `BankStatement`, `Expense`, `Income`, `Wallet`, `Payroll`.

Exit criteria:

- A consumer can configure a fees master, assign it to a class, generate
  invoices for a term, accept payments via the payment port, and produce a
  collection report.
- Carry-forward rules are enforceable as a domain service.

## Phase 5 — Ports & Adapters

Deliverables:

- `smscore-storage` port trait and a reference PostgreSQL adapter.
- `smscore-storage-sqlite` adapter for embedded deployments.
- `smscore-auth` port and reference local-password and OAuth2 adapters.
- `smscore-notify` port and reference email, SMS, push adapters.
- `smscore-payment` port and reference cash and bank-slip adapters.
- `smscore-files` port and reference S3-compatible and local adapters.
- `smscore-event-bus` implementations (in-process, NATS, Redis).

Exit criteria:

- All port contracts documented under `docs/ports/*.md` have reference
  implementations.
- Adapter trait-object compilation succeeds (`Box<dyn NotificationProvider>`).

## Phase 6 — Event Bus & Integration

Deliverables:

- Event subscription API for consumers.
- Outbox pattern for transactional event publication.
- Domain-event replay for offline reconciliation.
- Integration port and reference LMS and video-conferencing adapters.

Exit criteria:

- A consumer can subscribe to `StudentAdmitted` and persist a downstream
  projection idempotently.

## Phase 7 — AI Layer & SDK

Deliverables:

- Capability catalog exposed as typed tools.
- Permission-gated command surface for tool-using LLMs.
- High-level SDK crate `smscore-sdk` that wraps common workflows.
- Sample CLI demonstrating daily operations.

Exit criteria:

- An AI agent can be given a sandboxed capability set and complete an
  admission workflow without violating invariants.

## Phase 8 — Production Readiness

Deliverables:

- Integration test suite covering multi-tenant scenarios.
- Load test for attendance marking (10k students) and bulk fee invoice
  generation.
- Cross-compile verification (Linux x86_64, aarch64, Windows, macOS).
- Security review of every public command surface.
- Documentation audit against the 10-point validation checklist.

Exit criteria:

- All ten validation questions answer "Yes".
- CI green on Linux, macOS, Windows.

## Build Order (One-Page)

```text
1. Foundation       ─ workspace, core, platform, rbac, settings, events
2. Core Domains     ─ academic, hr, library, events, documents, cms
3. Ops Domains      ─ attendance, assessment, communication, facilities
4. Finance          ─ fees, payments, banking, expenses, payroll
5. Ports & Adapters ─ storage, auth, notify, payment, files, event bus
6. Eventing         ─ outbox, replay, integration
7. AI & SDK         ─ tool surface, capability gating, sdk, cli sample
8. Production       ─ tests, perf, cross-compile, security, docs audit
```
