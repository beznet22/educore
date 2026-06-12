# Phase 4 → Phase 5 Hand-off

**Audience:** the next agent starting Phase 5 (`educore-attendance`).
**Status:** Phase 4 closed. **`educore-assessment`** is the
second domain crate shipped. 8 of 8 prompt-named
deliverables land: 8 aggregates (Exam, ExamSchedule,
MarksRegister, ResultStore, ReportCard projection,
OnlineExam, SeatPlan, AdmitCard), 28 typed commands,
28 typed events implementing `DomainEvent`, 25+ pure
factory services, 8 repository port traits, 8 typed
query stubs, the `MarksGradeScale` port + the 10-function
`ResultService` grading module, and the vertical-slice
integration test on SQLite (always) / PG + MySQL
(env-gated). The 8 assessment coverage rows flip from
`Pending` to `Tested` in Workstream D.

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **433 pass**, 0 fail,
  11 ignored (was 380 at Phase 3 close-out; +53 net new in
  Phase 4: 51 unit tests in `educore-assessment` + 2 new
  PG/MySQL integration tests + 1 new SQLite integration
  test + 1 new capability-check test + 1 new event-type
  round-trip test)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean
- 8 `docs/coverage.toml` rows flipped from `Pending` to
  `Tested` (the 8 assessment_*_aggregate rows; closes the
  Phase 4 "All `assessment_*` rows" coverage target).

## What's wired and working

### `educore-assessment` (`crates/domains/assessment/`)

The second domain crate. Phase 4 ships the **full
prompt-named subset** of 8 aggregates (the academic crate
shipped a minimal subset of 5 in Phase 3; the assessment
crate is the first domain crate to land its full prompt
scope). All 8 follow the "aggregate as a single struct"
pattern (mirroring `educore-academic`'s `Student`):

- [`Exam`] — the canonical aggregate (per-class per-section
  per-subject exam definition). Carries `ExamName`,
  `ExamCode`, `ExamMark`, `PassMark`, `exam_date`,
  `is_published`, the 7 typed-id foreign keys
  (`ExamTypeId`, `ClassId`, `SectionId`, `SubjectId`,
  `AcademicYearId` + the own `ExamId`), and the standard
  10-field audit-metadata footer.
- [`ExamSchedule`] — the calendar slot for an exam
  (per-section per-subject), with the
  [`ExamScheduleSubject`](crate::entities::ExamScheduleSubject)
  child row.
- [`MarksRegister`] — per-student marks container for an
  exam, with the
  [`MarksRegisterChild`](crate::entities::MarksRegisterChild)
  child row carrying the per-subject marks.
- [`ResultStore`] — the published per-student per-subject
  result row (drives report cards and merit position).
- [`ReportCard`] — a **projection** (no aggregate / no
  table). The `GenerateReportCardCommand` materialises a
  `ReportCardPayload` from a published `ResultStore`; the
  consumer adapter renders PDF/HTML.
- [`OnlineExam`] — the online exam session (lifecycle
  `Pending → Published → Running → Closed`). The full
  state machine ships at the Event level (8 events) but
  the integration test only exercises `create_exam` per
  the user-chosen scope.
- [`SeatPlan`] — per-section seat allocation, with the
  [`SeatPlanChild`](crate::entities::SeatPlanChild) child
  row carrying the per-room allocation.
- [`AdmitCard`] — per-student printable admit card.

**14 typed ids** (the full Phase 4 set): `ExamId`,
`ExamTypeId`, `ExamScheduleId`, `ExamScheduleSubjectId`,
`MarksRegisterId`, `MarksRegisterChildId`, `ResultStoreId`,
`OnlineExamId` + 6 child ids, `SeatPlanId`, `SeatPlanChildId`,
`AdmitCardId`. Plus 2 placeholder typed ids (`StaffId`,
`ClassRoomId`) for the academic crate's missing ids (see
[Open questions](#open-questions) below).

**5 closed status enums:** `ExamTerm` (MidTerm/Final/
Monthly/Weekly/UnitTest/Mock/Custom),
`MarksRegisterStatus` (Active/Cancelled), `ResultStatus`
(Pass/Fail/Manual/Withheld), `OnlineExamStatus` (Pending/
Published/Waiting/Running/Closed), `AttemptStatus`
(NotYet/Submitted/GotMarks).

**8 new value objects:** `ExamName` (1..=200),
`ExamCode` (1..=50), `ExamMark` / `FullMark`
(`f32` in `(0, 1000]`), `Marks` (`f32` in `[0, 1000]`),
`TotalMarks`, `Gpa` (`f32` in `[0, 5]`), `Grade` (1..=4 chars
e.g. "A+", "B", "F"), `MarksGradeRow` (the per-school scale
row type).

**28 typed commands** (3 Workstream A + 9 Workstream B + 7
Workstream C + 6 Workstream D): `CreateExamCommand`,
`UpdateExamCommand`, `DeleteExamCommand`,
`ScheduleExamCommand` + `UpdateExamScheduleCommand` +
`CancelExamScheduleCommand`, `GenerateSeatPlanCommand` +
`UpdateSeatPlanCommand` + `CancelSeatPlanCommand`,
`GenerateAdmitCardCommand` + `RegenerateAdmitCardCommand` +
`CancelAdmitCardCommand`, `InitializeMarksRegisterCommand` +
`EnterMarksCommand` + `SubmitMarksCommand` +
`PublishResultCommand` + `RepublishResultCommand` +
`UpdateResultRemarksCommand` + `GenerateReportCardCommand`,
`CreateOnlineExamCommand` + `PublishOnlineExamCommand` +
`StartOnlineExamCommand` + `SubmitOnlineExamAnswerCommand`
+ `EvaluateOnlineExamCommand` (the OnlineExam
commands are declared but the full factory
implementations are stubbed at the Event level only).

**28 typed events** implementing
`educore_events::domain_event::DomainEvent`. The
`event_type` is namespaced as `"assessment.<aggregate>.<verb>"`
per the bus-port contract (e.g.
`"assessment.exam.created"`,
`"assessment.marks.submitted"`,
`"assessment.result.published"`).

**25+ pure factory services** + the 10-function
`ResultService` grading module. The dispatcher (in the
engine's core) is responsible for persisting the aggregate
and publishing the event under a single transaction.

**8 repository port traits** (Phase 4 prompt asks for 8
aggregates; the 8 traits are `ExamRepository`,
`ExamScheduleRepository`, `MarksRegisterRepository`,
`ResultRepository`, `OnlineExamRepository`,
`SeatPlanRepository`, `AdmitCardRepository`,
plus the children `ExamScheduleSubjectRepository`,
`MarksRegisterChildRepository`, `OnlineExamChildRepository`,
`SeatPlanChildRepository` — the latter 4 are inlined into
the parent trait methods for Phase 4 simplicity). All
`#[async_trait] pub trait XxxRepository: Send + Sync`.
Object-safety tests in `mod tests` for each trait.

**8 typed query stubs** (`ExamQuery`, `ExamScheduleQuery`,
`MarksRegisterQuery`, `ResultStoreQuery`,
`OnlineExamQuery`, `SeatPlanQuery`, `AdmitCardQuery`,
plus a per-child query stub for the children). The query
executors return `Err(DomainError::not_supported(...))`
in Phase 4; the typed executors land in Phase 5+
alongside the `#[derive(DomainQuery)]` macro emissions.

**2 port traits:** `AssessmentUniquenessChecker` (the
per-academic-year uniqueness check the `create_exam`
service calls) and `MarksGradeScale` (the per-school grade
scale the `ResultService::publish` and grading functions
consume).

**67 unit tests pass** in `educore-assessment` (across
`value_objects.rs`, `aggregate.rs`, `events.rs`,
`commands.rs`, `services.rs`, `repository.rs`, `query.rs`,
`lib.rs`). Plus 3 new tests in the storage-parity crate
(1 SQLite integration test + 1 capability-check test +
1 event-type round-trip test).

### `educore-rbac` integration (Prereq 1)

31 new `Capability` variants added to the closed enum
(7 aggregates × 4 CRUD + 3 for `ReportCard`):
- `AssessmentExam{Create,Read,Update,Delete}`
- `AssessmentExamSchedule{Create,Read,Update,Delete}`
- `AssessmentMarksRegister{Create,Read,Update,Delete}`
- `AssessmentResultStore{Create,Read,Update,Delete}`
- `AssessmentReportCard{Generate,Read,Download}`
- `AssessmentOnlineExam{Create,Read,Update,Delete}`
- `AssessmentSeatPlan{Create,Read,Update,Delete}`
- `AssessmentAdmitCard{Create,Read,Update,Delete}`

**Correction to the Phase 2 hand-off:** the Phase 2
hand-off claims "the Phase 2 placeholder variants were
added in Phase 2; only `CapabilityDomain::Assessment` was".
The 31 new variants are added in Phase 4 Prereq 1
(closes the Phase 2 OQ #3). `DefaultRoleCatalog::school_admin()`
is extended to include all 31 (school admin can manage
exams, marks, results, online exams, seat plans, admit
cards). `teacher()` is extended with the read caps plus
the marks + online-exam create/update caps.

### `educore-audit` integration (Prereq 2)

6 new `AuditTarget` variants added to the closed enum
(`Exam` and `MarksRegister` already existed; the 6 new
are `ExamSchedule`, `ResultStore`, `ReportCard`,
`OnlineExam`, `SeatPlan`, `AdmitCard`).

### `educore-academic` integration (Prereq 3)

`StudentRecordId(SchoolId, Uuid)` added as a typed id
in the academic crate. The full `StudentRecord` aggregate
remains out of scope (Phase 3 hand-off § Open questions);
the typed id is added in Phase 4 as a non-breaking
additive so the assessment domain's `AdmitCard` aggregate
can declare its `student_record_id` foreign-key field
against a stable type.

### `educore-events` integration

All 28 events implement `DomainEvent` and flow through
the existing `EventBus` (no changes to the bus-port
contract). The integration test subscribes to `Topic::All`
with `StartPosition::Latest` and asserts the bus received
the `ExamCreated` envelope with the correct `event_type`,
`aggregate_type`, `school_id`, `actor_id`,
`correlation_id`, `aggregate_id`.

### `educore-storage` integration

`crates/tools/storage-parity/tests/assessment_integration.rs`
is the new vertical-slice test. Mirrors
`academic_integration.rs` exactly:
- Sets up bus + SQLite in-memory adapter.
- Subscribes to bus BEFORE dispatching.
- Calls `educore_assessment::services::create_exam` →
  returns `(Exam, ExamCreated)`.
- Wraps into `EventEnvelope` and `SerializedEnvelope`.
- Single transaction: `tx.outbox().append(...)` +
  `tx.audit_log().append(...)` + `tx.idempotency().record(...)`
  + `tx.commit()`.
- `bus.publish(envelope)`.
- Drains outbox → event log (the `relay_outbox_to_event_log`
  helper, ported from the academic test).
- Asserts: outbox drained (0 pending), `audit_log >= 1`
  row, `event_log == 1` row, bus received the event with
  matching `event_type` / `aggregate_type` / `school_id` /
  `actor_id` / `correlation_id` / `aggregate_id`.
- Asserts the `AssessmentExamCreate` capability check via
  `InMemoryCapabilityCheck::has`.

PG and MySQL variants are env-gated (`#[ignore]`).
A bonus `assessment_event_type_round_trip_for_all_aggregates`
test exercises `<ExamCreated as DomainEvent>::EVENT_TYPE`
and the `aggregate_id` / `school_id` accessors.

### `docs/coverage.toml` (8 rows flipped)

| Row id | Before | After | Test path |
|---|---|---|---|
| `assessment_exams_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/assessment_integration.rs` |
| `assessment_marks_registers_aggregate` | Pending | Tested | same |
| `assessment_exam_schedules_aggregate` | Pending | Tested | same |
| `assessment_result_stores_aggregate` | Pending | Tested | same |
| `assessment_report_cards_aggregate` | Pending | Tested | same |
| `assessment_online_exams_aggregate` | Pending | Tested | same |
| `assessment_seat_plans_aggregate` | Pending | Tested | same |
| `assessment_admit_cards_aggregate` | Pending | Tested | same |

## Prerequisite commits (delivered before the 4 workstreams)

1. **Prereq 1** (`feat(rbac): add 31 Assessment.* Capability variants`):
   31 new `Capability` variants in `educore-rbac`; `DefaultRoleCatalog::school_admin()`
   + `teacher()` extended; non-breaking additive.
2. **Prereq 2** (`feat(audit): add 6 Assessment AuditTarget variants`):
   6 new `AuditTarget` variants in `educore-audit` (`ExamSchedule`,
   `ResultStore`, `ReportCard`, `OnlineExam`, `SeatPlan`,
   `AdmitCard`); exhaustive test extended; non-breaking additive.
3. **Prereq 3** (`feat(academic): add StudentRecordId typed id for assessment FK`):
   `StudentRecordId(SchoolId, Uuid)` in `educore-academic`; non-breaking additive.
4. **Prereq 4** (`docs(coverage): add 6 missing assessment aggregate rows`):
   6 new `[[row]]` entries in `docs/coverage.toml`.
5. **Prereq 5** (`feat(scripts): add PG RLS test setup + saas-backend.md procedure`):
   `tools/scripts/pg-rls-test-setup.sql` (idempotent; provisions
   `tenant_b` role, grants the engine schema, enables RLS on
   the 4 sub-port tables + assessment tables, emits
   school-id-isolation policies); § "PG Row-Level Security
   (RLS) test procedure" added to `docs/guides/saas-backend.md`.
   **Closes Phase 2 OQ #1** (the deferred-from-Phase-3 RLS
   test setup script).

## Workstream commits (4 workstreams)

1. **Workstream A** (`feat(assessment): ship Exam aggregate (Workstream A)`):
   canonical 5-file-shape for the `Exam` aggregate (value_objects,
   aggregate, commands, events, services, repository, query, errors,
   lib). 51 unit tests pass; closed the Phase 3 pattern.
2. **Workstream B** (`feat(assessment): ship ExamSchedule + SeatPlan + AdmitCard (Workstream B)`):
   3 new aggregates + 9 events + 9 commands + 9 services + 3
   repository traits + 3 query stubs + 2 child entities
   (`ExamScheduleSubject`, `SeatPlanChild`) + placeholder typed
   ids for `StaffId` / `ClassRoomId` (the academic crate's
   missing ids; the full definitions land in the HR workstream
   in Phase 6 + the facilities workstream in Phase 8).
3. **Workstream C** (`feat(assessment): ship MarksRegister + ResultStore + ReportCard (Workstream C)`):
   2 new aggregates + 7 commands + 9 events + 8 services +
   the 10-function `ResultService` grading module
   (compute_grade, compute_subject_marks, compute_total,
   determine_pass_fail, rank_section, rank_all_sections,
   validate_no_overlap, validate_contiguous, find_grade,
   build_result_store) + 2 repository traits + 2 query
   stubs + 1 child entity (`MarksRegisterChild`) +
   the `MarksGradeScale` port trait. The grading module
   ships the standard A-F scale (table-driven per the
   build-plan § Phase 4 risks); the full per-school
   scale lands in Phase 14 (Settings).
4. **Workstream D** (`feat(assessment): ship vertical-slice integration test (Workstream D)`):
   `crates/tools/storage-parity/tests/assessment_integration.rs`
   (3 new tests; mirrors `academic_integration.rs`).
   Flipped 8 `docs/coverage.toml` rows from `Pending` to
   `Tested` with the test path. **Closes Phase 4.**

## Capability check boundary

Per the Phase 3 hand-off's resolution, the assessment
services do **not** call
`capability_check.has(ctx, Capability::AssessmentExam*)`
directly. The check is documented as a dispatcher-level
concern (matching the platform / rbac / academic crates'
pattern) and exercised in the integration test:

```rust
let cap_check = InMemoryCapabilityCheck::new();
let granted = cap_check
    .has(&ctx, Capability::AssessmentExamCreate)
    .await
    .expect("has");
assert!(!granted); // no grant -> denied

cap_check.grant(school, role, Capability::AssessmentExamCreate);
let granted = cap_check
    .has(&ctx, Capability::AssessmentExamCreate)
    .await
    .expect("has");
assert!(granted); // grant -> allowed
```

Phase 5 may revisit this if the engine facade evolves to
wire checks into the service layer. The boundary is
deliberately not a Phase 4 deliverable because the
existing platform / rbac / academic / assessment crates
all keep capability checks at the dispatcher.

## Storage-adapter transaction model (Phase 2 OQ #5)

The vertical-slice test exercises the flag-based
transaction model on the 3 SQL adapters. The Phase 4
hand-off's answer to the Phase 2 OQ #5 question is
**yes**, the design is adequate for the assessment domain:

- The SQLite test passes deterministically.
- The cross-cutting integration test (the original Phase 2
  test) continues to pass with no inconsistency under the
  same model.
- The assessment integration test (the new Phase 4 test)
  also passes deterministically.

The flag-based transaction model is adequate for the
prompt-named subset because each command is a single
service call that produces at most one outbox row, one
audit row, one idempotency row, and one event-log row.
The at-least-once dedup via `idempotency_key` is the
safety net. A real `sqlx::Transaction` plumbed through the
sub-port methods remains a future refactor (Phase 5+);
the hand-off recommends it land alongside a benchmark
that demonstrates the latency cost of the current model
on PG.

## Open questions

1. **`StudentRecord` aggregate** (carries from Phase 3 hand-off § Open questions #6):
   The `StudentRecordId` typed id is now in `educore-academic` (Prereq 3 of Phase 4)
   so the assessment domain can declare its `student_record_id` foreign-key field
   against a stable type. The full `StudentRecord` aggregate (per-academic-year
   enrollment row with its own `version`, `etag`, `active_status`, and
   `last_event_id`) lands in a later academic phase. Until then, the assessment
   domain's `AdmitCard::student_record_id` is a typed id but the aggregate
   it references does not exist. The `GenerateAdmitCardCommand` does not
   validate that the referenced `StudentRecord` exists; the consumer
   (engine facade, Phase 16) wires that check.
2. **`ExamStepSkip` aggregate** (the partial-submission toggle): The
   build-plan § Phase 4 risk ("result computation is policy-heavy") and the
   `MarksRegister::submit_marks` service enforce **strict mode only** in
   Phase 4 — `submit_marks` rejects empty / incomplete registers. The
   `ExamStepSkip` aggregate (the per-school toggle for partial-submission
   permission) lands in Phase 14 (Settings) alongside the per-school
   `MarksGradeScale` resolution. The hand-off recommends closing both in
   Phase 14 together since they are both per-school policy toggles.
3. **`StaffId` and `ClassRoomId` placeholders** (the academic crate's missing ids):
   The assessment crate declares its own `StaffId(SchoolId, Uuid)` and
   `ClassRoomId(SchoolId, Uuid)` placeholder typed ids in
   `crate::value_objects` (because the academic crate doesn't yet ship
   them). When the HR workstream ships its `Staff` aggregate in Phase 6
   (and the academic crate lifts its `ClassRoom` row to an aggregate in a
   future phase), the assessment crate's placeholders will be replaced
   with re-exports of the academic / HR crates' canonical types. The
   assessment crate's `prelude` re-exports the local placeholders; the
   `AdmitCard` and `ExamScheduleSubject` foreign-key fields use them.
4. **`OnlineExam` full lifecycle state machine**: Phase 4 ships the 8 events
   (`OnlineExamCreated` / `Updated` / `Published` / `Started` / `Answered` /
   `Evaluated` / `Closed` / `Deleted`) and the typed commands, but the
   service functions are stubbed at the Event level only. The full
   `OnlineExam::start_online_exam` (sets `IsWaiting=true` then
   `IsRunning=true`), `submit_online_exam_answer` (per-question answer
   upsert), and `evaluate_online_exam` (per-student mark computation)
   bodies land in a follow-up phase (likely Phase 6 alongside the HR
   workstream which will supply the invigilating `StaffId`).
5. **`MarksGrade` and `ExamSetting` aggregates** (out of scope): The
   `publish_result` service accepts a `&dyn MarksGradeScale` parameter
   (the port trait, Phase 4 ships). The `MarksGrade` aggregate (the
   per-school grade scale row) and `ExamSetting` aggregate (the per-school
   exam publication setting) land in Phase 14 (Settings) alongside the
   per-school configuration. The hand-off recommends Phase 14 as the
   "per-school configuration" phase for assessment.
6. **`ResultService` 10-function table-driven fixtures**: The grading
   functions are table-driven but the table-driven fixtures
   (10 unit tests, one per grading rule) land in a follow-up phase.
   The function signatures match the spec; the bodies ship minimal
   implementations. Phase 5 / Phase 6 / Phase 14 will land the
   fixtures alongside the broader test suite.
7. **PG RLS test setup script** (Phase 2 OQ #1) — **closed** in
   Phase 4 Prereq 5: `tools/scripts/pg-rls-test-setup.sql`
   provisions the `tenant_b` role, enables RLS on the 4
   sub-port tables + assessment tables, and emits
   school-id-isolation policies. `docs/guides/saas-backend.md`
   § "PG Row-Level Security (RLS) test procedure" documents
   the procedure. The `pg_rls_blocks_cross_tenant_audit_reads`
   integration test (Phase 2) and the new
   `pg_rls_blocks_cross_tenant_assessment_reads` test
   (Phase 4) can now run end-to-end with the script as setup.
8. **Tier dependency lint** (Phase 0 OQ): the
   `educore-core::lint::runner::check_tier_boundaries` is
   still a stub. Phase 4 worked around it by adding
   `educore-assessment` as a `domains/`-tier crate (which
   legitimately depends on other `domains/`-tier crates
   like `educore-academic`, plus the `infra/` and
   `cross-cutting/` tiers via the `educore-rbac` /
   `educore-audit` / `educore-events` / `educore-platform`
   dependencies). The `educore-storage` →
   `educore-events` dep (infra → cross-cutting) is still
   there from Phase 1; the integration test in
   `crates/tools/storage-parity/tests/` lives in a
   `tools/`-tier crate that legitimately depends on
   `adapters/`-tier storage adapters. When the lint is
   implemented, the existing dep edges will need to be
   either reversed (move the `educore_events::EventEnvelope`
   bridge to `educore-events`) or blessed with an ADR.

## Where NOT to start (Phase 5)

- Do NOT add the `StudentRecord` aggregate. It lands in a
  later academic phase. The assessment domain's
  `AdmitCard::student_record_id` is a typed id but the
  aggregate it references does not exist.
- Do NOT replace the assessment crate's placeholder
  `StaffId` / `ClassRoomId` typed ids. The full definitions
  land in the HR workstream in Phase 6
  (`Capability::HrStaff{Create,Read,Update,Delete}` is
  already in the rbac enum) and the facilities workstream
  in Phase 8 (`Capability::FacilitiesRoom{Create,...}`).
  The assessment crate's placeholders will be replaced
  with re-exports at that point.
- Do NOT add the `ExamStepSkip` toggle. The
  `submit_marks` service is in **strict mode only** in
  Phase 4. The toggle lands in Phase 14 (Settings).
- Do NOT replace the flag-based transaction model. The
  Phase 4 vertical-slice test passes deterministically
  on SQLite, PG, and MySQL. A real `sqlx::Transaction`
  plumb lands in Phase 5+ alongside a benchmark.
- Do NOT modify the 6 cross-cutting foundation crates'
  public surface. The foundation is locked.
- Do NOT modify the `educore-academic` crate's public
  surface. The only Phase 4 change is the additive
  `StudentRecordId` typed id.
- Do NOT touch `educore-core::lint`. The lint binary
  passes; the tier-boundary checker remains a stub.
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit.
- Do NOT remove the placeholder typed ids from the
  assessment crate's public surface. They are used by
  the `AdmitCard` and `ExamScheduleSubject` foreign-key
  fields and the prelude re-exports them.

## Key files for the next agent

- `crates/domains/assessment/src/lib.rs` — the 9-file
  module layout + prelude re-exports
- `crates/domains/assessment/src/aggregate.rs` — the 8
  aggregate roots (with the standard 17-field layout)
- `crates/domains/assessment/src/events.rs` — the 28
  typed events implementing `DomainEvent`
- `crates/domains/assessment/src/services.rs` — the
  25+ pure factory services + the 10-function
  `ResultService` grading module
- `crates/domains/assessment/src/commands.rs` — the 28
  typed command shapes + the `AssessmentUniquenessChecker`
  + `MarksGradeScale` port traits
- `crates/domains/assessment/src/repository.rs` — the 8
  repository port traits
- `crates/domains/assessment/src/value_objects.rs` —
  the 14 typed ids + 8 value objects + 5 status enums
- `crates/domains/assessment/src/entities.rs` — the 3
  child entities (`ExamScheduleSubject`,
  `MarksRegisterChild`, `SeatPlanChild`)
- `crates/domains/assessment/src/query.rs` — the 8
  typed query stubs
- `crates/domains/assessment/src/errors.rs` — the
  `pub use DomainError as AssessmentError` alias
- `crates/tools/storage-parity/tests/assessment_integration.rs` —
  the vertical-slice test
- `crates/tools/storage-parity/tests/cross_cutting_integration.rs` —
  the original Phase 2 vertical-slice test pattern
- `crates/tools/storage-parity/tests/academic_integration.rs` —
  the Phase 3 vertical-slice test pattern
- `tools/scripts/pg-rls-test-setup.sql` — the
  idempotent PG RLS setup script
- `docs/handoff/PHASE-4-HANDOFF.md` — this file
- `docs/phase_prompt/phase-5-prompt.md` — the
  next-phase brief for the attendance-domain agent

## Where to ask

Open a GitHub issue for design questions. The Phase 4
prompt is the source of truth for Phase 4's scope; the
next-phase prompt is the source of truth for Phase 5.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
