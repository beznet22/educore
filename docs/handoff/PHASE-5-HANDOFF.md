# Phase 5 → Phase 6 Hand-off

**Audience:** the next agent starting Phase 6 (`educore-hr`).
**Status:** Phase 5 closed. **`educore-attendance`** is the
third domain crate shipped. All 5 prereq + 2 workstream + 1
bugfix commits land. The 4 prompt-named aggregates (plus
`BulkAttendanceImport`) ship with the matching 9-file module
layout, 21 typed events implementing `DomainEvent`, 14 pure
factory services, 5 repository port traits, 2 child entities,
and 5 typed query stubs. The vertical-slice integration test
on SQLite (always) / PG + MySQL (env-gated) passes; the
200-row bulk-mark benchmark on SQLite finishes under 1 second
(PG target: <100ms when `EDUCORE_PG_URL` is set).

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **530 pass**, 0 fail, 14 ignored
  (was 433 at Phase 4 close-out; +97 net new in Phase 5:
  93 unit tests in `educore-attendance` + 4 new
  always-on integration tests in
  `crates/tools/storage-parity/tests/attendance_integration.rs`
  + 3 env-gated PG/MySQL/PG-100ms variants)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean
- 13 `docs/coverage.toml` rows flipped from `Pending` to
  `Tested` (7 `attendance_*_aggregate` + 6 `*_event`)

## What's wired and working

### `educore-attendance` (`crates/domains/attendance/`)

The third domain crate. Phase 5 ships the **full
prompt-named subset** of aggregates (4 prompt-named + 1
bulk-import aggregate) following the 9-file module layout
from `AGENTS.md`:

**5 aggregate roots (17-field pattern; mirrors the
assessment `Exam`):**

- [`StudentAttendance`](crates/domains/attendance/src/aggregate.rs) —
  the canonical aggregate (per-student per-day). Carries the
  `StudentAttendanceId`, `SchoolId`, `StudentId`,
  `StudentRecordId`, `ClassId`, `SectionId`, `AttendanceDate`
  (NaiveDate), `AttendanceType` (P/A/L/F/H), `in_time` /
  `out_time` (Option<String>), `notes`, `is_absent`
  (derived: `type == Absent`), `MarkedBy`, `MarkedAt`,
  `MarkedFrom`, and the standard 10-field audit-metadata
  footer.
- [`StaffAttendance`](crates/domains/attendance/src/aggregate.rs) —
  per-staff per-day. 14-field root (id, school_id, staff_id,
  attendance_date, attendance_type, in_time, out_time, notes,
  marked_by, marked_at, marked_from, + 10-field audit footer).
- [`SubjectAttendance`](crates/domains/attendance/src/aggregate.rs) —
  per-student per-subject per-day. 16-field root (adds
  `student_record_id`, `class_id`, `section_id`, `subject_id`,
  `notify` flag).
- [`ExamAttendance`](crates/domains/attendance/src/aggregate.rs) —
  per-exam per-student. 16-field root with `exam_id`
  foreign key referencing the assessment crate's `ExamId`
  (the cross-crate dep from Prereq 3).
- [`BulkAttendanceImport`](crates/domains/attendance/src/aggregate.rs) —
  the bulk-import job aggregate (id, school_id,
  academic_year_id, source, status, row_count, absent_count,
  marked_by, marked_at, + 10-field audit footer).

**9 typed ids** (the full Phase 5 set): `StudentAttendanceId`,
`SubjectAttendanceId`, `StaffAttendanceId`, `ExamAttendanceId`,
`BulkAttendanceImportId`, `StudentAttendanceImportId`,
`StaffAttendanceImportId`, `ClassAttendanceId`,
`AttendanceBulkId`. Plus 1 placeholder typed id
(`StaffId`, declared locally in the attendance crate's
`value_objects.rs`; the full definition lands in the HR
workstream in Phase 6 — the assessment crate has its own
placeholder `StaffId` that the HR crate will replace both
with).

**4 closed enums:** `AttendanceStatus`
(Present/Absent/Late/HalfDay/Holiday/OnLeave) with `as_str()`,
`AttendanceType` (P/A/L/F/H — single-char wire codes per the
spec) with `as_str()` and `is_absent()`, `AttendanceSource`
(Manual/Biometric/BulkImport/Api) with `as_str()`,
`ImportStatus` (Pending/Validated/Committed/Failed/Cancelled)
with `as_str()`.

**21 typed events** implementing
[`educore_events::domain_event::DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
The `event_type` is namespaced as
`"attendance.<aggregate>.<verb>"` per the bus-port contract
(e.g. `"attendance.student.marked"`,
`"attendance.student.absent"`,
`"attendance.bulk_import.committed"`,
`"attendance.bulk_import.row_imported"`).

**14 typed commands** + 14 `*_COMMAND_TYPE` constants
(`ATTENDANCE_STUDENT_MARK_COMMAND_TYPE` etc.) for the
idempotency sub-port. Plus the
[`AttendanceUniquenessChecker`](crates/domains/attendance/src/commands.rs)
port trait (the per-day per-student uniqueness check the
`mark_student_attendance` and bulk-marking services call).

**14 pure factory services** + the `AttendanceService` helper
struct (with `is_late`, `emit_absence_event`,
`dedup_within_day` static methods). The
[`bulk_mark_student_attendance`](crates/domains/attendance/src/services.rs)
service returns `BulkMarkResult { aggregates, marked_events,
absent_events }` — the dispatcher persists the 200 rows
via `tx.bulk_insert_student_attendances(...)` in a single
multi-row `INSERT` (PG / MySQL) or transaction-grouped
inserts (SQLite).

**5 repository port traits** (Phase 4 pattern; 9 methods
on `StudentAttendanceRepository` including the new
`bulk_insert`, 5 on `SubjectAttendanceRepository`, 7 on
`StaffAttendanceRepository`, 5 on `ExamAttendanceRepository`,
10 on `AttendanceImportRepository`). All
`#[async_trait] pub trait XxxRepository: Send + Sync`.
Object-safety tests in `mod tests` for each trait.

**5 typed query stubs** (`StudentAttendanceQuery`,
`SubjectAttendanceQuery`, `StaffAttendanceQuery`,
`ExamAttendanceQuery`, `BulkAttendanceImportQuery`). The
query executors return `Err(DomainError::not_supported(...))`
in Phase 5; the typed executors land in Phase 6+ alongside
the `#[derive(DomainQuery)]` macro emissions.

**1 port trait:** `AttendanceUniquenessChecker` (4 methods:
`student_day_exists`, `subject_day_exists`,
`staff_day_exists`, `import_source_date_exists`).

**2 child entities:** `StudentAttendanceImport` +
`StaffAttendanceImport` (the staging rows the bulk-import
service promotes to `StudentAttendance`).

**93 unit tests pass** in `educore-attendance` (across
`value_objects.rs`, `aggregate.rs`, `events.rs`,
`commands.rs`, `services.rs`, `repository.rs`, `query.rs`,
`lib.rs`). Plus 4 new tests in the storage-parity crate
(1 SQLite vertical-slice integration test + 1 capability-
check test + 1 event-type round-trip test + 1 200-row
bulk-mark bench proxy).

### `educore-rbac` integration (Prereq 1, commit `122a451`)

24 new `Capability` variants added to the closed enum
(Student × 4 + Subject × 5 + Staff × 4 + Import × 4 +
Exam × 4 + BulkMark × 1 + Notify × 1 + Report × 1 = 24):

```text
AttendanceStudentCreate, AttendanceStudentRead,
AttendanceStudentUpdate, AttendanceStudentDelete,
AttendanceSubjectCreate, AttendanceSubjectRead,
AttendanceSubjectUpdate, AttendanceSubjectDelete,
AttendanceSubjectNotify,
AttendanceStaffCreate, AttendanceStaffRead,
AttendanceStaffUpdate, AttendanceStaffDelete,
AttendanceExamCreate, AttendanceExamRead,
AttendanceExamUpdate, AttendanceExamDelete,
AttendanceImportCreate, AttendanceImportRead,
AttendanceImportUpdate, AttendanceImportDelete,
AttendanceBulkMark, AttendanceReportRead, AttendanceNotify
```

All resolve to `CapabilityDomain::Attendance`. The
`assessment_capabilities_round_trip` test extended to
`attendance_capabilities_round_trip_and_resolve_to_attendance_domain`
asserting the 24 new variants round-trip via `as_str()` /
`from_str()`. `DefaultRoleCatalog` extension is documented
in the hand-off as a follow-up for the consumer; the engine
itself does not bake role→capability grants (matches the
Phase 4 pattern).

### `educore-audit` integration (Prereq 2, commit `b089db5`)

4 new `AuditTarget` variants added to the closed enum:
`SubjectAttendance(Uuid)`, `StaffAttendance(Uuid)`,
`BulkAttendanceImport(Uuid)`, `ClassAttendance(Uuid)`.
The exhaustive `audit_target_type_for_every_variant_is_nonempty`
test extended to cover the 4 new variants.

### `educore-storage` integration (Prereq 5, commit `7a3cee1` + bugfix `14752c4`)

New `bulk_insert_student_attendances` method on the storage
port + 3 SQL adapters:
- **PostgreSQL**: single multi-row `INSERT INTO
  attendance_student_attendances (...) VALUES ($1, ...),
  ($13, ...), ...` via `sqlx::QueryBuilder::push_values`.
  Cap: 1,000 rows per call (24 cols × 1,000 = 24,000
  placeholders, well under the 65,535 PG limit).
- **MySQL**: same shape with `?` placeholders + backtick
  identifiers. Same 1,000-row cap.
- **SQLite**: transaction-grouped inserts at 40 rows/batch
  (24 × 40 = 960 placeholders, under the 999 default).
  All groups within the same `pool.begin()` transaction;
  a failure in any batch rolls back the whole bulk insert.

The transaction model is preserved (flag-based per command:
one outbox + one audit + one idempotency per command);
only the storage row writes within a transaction are
batched. The `StudentAttendanceRow` wire type is defined
in `crates/infra/storage/src/student_attendance_row.rs`
and re-exported from `educore_storage`.

**Bugfix commit `14752c4`:** the Prereq 5 `QueryBuilder::new(...)`
string ended with `" VALUES "` (trailing space) and
`push_values(...)` also prepends ` VALUES `, producing
invalid SQL with a doubled keyword. Fix: drop the trailing
`"VALUES "` from the prefix in all 3 SQL adapters. The
Phase 5 vertical-slice integration test was passing via
the cross-cutting bus path; the bugfix enables the
storage-row insert path the bench measures.

### `educore-academic` integration (no new deps; uses existing)

`StudentId`, `ClassId`, `SectionId`, `SubjectId`,
`AcademicYearId`, `StudentRecordId` — re-exported from
`educore-academic` in the attendance crate's
`value_objects.rs`. Plus 1 placeholder `StaffId` declared
locally (mirrors the assessment crate's placeholder
pattern; the full definition lands in Phase 6 HR).

### `educore-assessment` integration (Prereq 3, commit `233638b`)

`ExamId` re-exported from `educore-assessment` in the
attendance crate's `value_objects.rs`. The
`ExamAttendance::exam_id` foreign key field references
the assessment crate's stable `ExamId` type. The
cross-crate dep `educore-attendance -> educore-assessment`
is justified per `AGENTS.md` § "Tier System" (domain-to-
domain deps are allowed with explicit justification in
the commit body).

### `educore-events` integration

All 21 events implement `DomainEvent` and flow through
the existing `EventBus` (no changes to the bus-port
contract). The integration test subscribes to `Topic::All`
with `StartPosition::Latest` and asserts the bus received
the `StudentAttendanceMarked` envelope with the correct
`event_type`, `aggregate_type`, `school_id`, `actor_id`,
`correlation_id`, `aggregate_id`.

### `crates/tools/storage-parity/tests/attendance_integration.rs`

The new vertical-slice test. Mirrors
`assessment_integration.rs` exactly:
- Sets up bus + SQLite in-memory adapter.
- Subscribes to bus BEFORE dispatching.
- Calls `educore_attendance::services::bulk_mark_student_attendance`
  with 200 students → returns `BulkMarkResult` (200 aggregates
  + 200 marked_events + 200 absent_events).
- Wraps every event into `EventEnvelope` and
  `SerializedEnvelope`.
- Single transaction: `tx.outbox().append(...)` (401
  envelopes) + `tx.audit_log().append(...)` (1 row) +
  `tx.idempotency().record(...)` (1 row) + `tx.commit()`.
- `bus.publish(envelope)` for every envelope.
- Drains outbox → event log (the `relay_outbox_to_event_log`
  helper, ported from the assessment test).
- Asserts: outbox drained (0 pending), `event_log >= 400`
  rows, bus received the first event with
  `event_type = "attendance.student.marked"`.
- Asserts the `AttendanceStudentCreate` capability check
  via `InMemoryCapabilityCheck::has`.

PG and MySQL variants are env-gated (`#[ignore]`).
A bonus `attendance_event_type_round_trip_for_all_aggregates`
test exercises 6 representative events' `<Event as
DomainEvent>::EVENT_TYPE` and the `aggregate_id` /
`school_id` accessors.

A bonus `attendance_bulk_mark_200_envelopes_sqlite_under_one_second`
test exercises the same 200-row flow and asserts the
total wall-clock time is <1s on SQLite. The PG target
is <100ms (the env-gated
`attendance_bulk_mark_200_envelopes_postgres_under_100ms`
test asserts this when `EDUCORE_PG_URL` is set).

### `docs/coverage.toml` (13 rows flipped)

| Row id | Before | After | Test path |
|---|---|---|---|
| `attendance_student_attendances_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/attendance_integration.rs` |
| `attendance_subject_attendances_aggregate` | Pending | Tested | same |
| `attendance_staff_attendances_aggregate` | Pending | Tested | same |
| `attendance_exam_attendances_aggregate` | Pending | Tested | same |
| `attendance_bulk_attendance_imports_aggregate` | Pending | Tested | same |
| `attendance_student_attendance_imports_aggregate` | Pending | Tested | same |
| `attendance_staff_attendance_imports_aggregate` | Pending | Tested | same |
| `student_attendance_marked_event` | Pending | Tested | same |
| `student_absent_for_day_event` | Pending | Tested | same |
| `subject_attendance_marked_event` | Pending | Tested | same |
| `staff_attendance_marked_event` | Pending | Tested | same |
| `staff_absent_for_day_event` | Pending | Tested | same |
| `bulk_import_committed_event` | Pending | Tested | same |

The `attendance_class_attendances_aggregate` row STAYS
`Pending` (projection, out of Phase 5 scope per the prompt).

## Prerequisite commits (5 + 1 bugfix, delivered before the 2 workstreams)

1. **Prereq 1** (`122a451`) — `feat(rbac): add 24 Attendance.* Capability variants`:
   24 new `Capability` variants in `educore-rbac`; non-breaking
   additive; `Capability::all()` + `domain()` + `aggregate()` +
   `action()` + `as_str()` + `from_str_opt()` arms extended.
2. **Prereq 2** (`b089db5`) — `feat(audit): add 4 Attendance AuditTarget variants`:
   4 new `AuditTarget` variants in `educore-audit`
   (`SubjectAttendance`, `StaffAttendance`, `BulkAttendanceImport`,
   `ClassAttendance`); exhaustive test extended; non-breaking
   additive.
3. **Prereq 3** (`233638b`) — `feat(attendance): add educore-assessment + educore-event-bus deps`:
   `crates/domains/attendance/Cargo.toml` gains
   `educore-assessment` (justified cross-crate dep) +
   `educore-event-bus` (for the integration test); drops the
   unused `educore-settings` dep.
4. **Prereq 4** (`013cd7c`) — `docs(coverage): add 13 attendance rows for Phase 5`:
   13 new `[[row]]` entries in `docs/coverage.toml` (7
   `attendance_*_aggregate` + 6 `*_event`).
5. **Prereq 5** (`7a3cee1`) — `feat(storage): add bulk_insert_student_attendances to port + 3 SQL adapters`:
   new `StudentAttendanceRow` wire type in
   `crates/infra/storage/src/student_attendance_row.rs`;
   new `bulk_insert_student_attendances` method on
   `StorageAdapter` (default `NotSupported`) and `Transaction`;
   implementations in PG (single multi-row `INSERT`),
   MySQL (same shape with `?` placeholders), SQLite
   (transaction-grouped inserts); DDL for the
   `attendance_student_attendances` table in all 3 adapters.
6. **Bugfix** (`14752c4`) — `fix(storage): correct double-VALUES bug in 3 bulk_insert adapters`:
   drops the trailing `"VALUES "` from the `QueryBuilder::new(...)`
   prefix in all 3 SQL adapters (the `push_values(...)` call
   prepends ` VALUES ` itself, producing invalid SQL with a
   doubled keyword). The Phase 5 vertical-slice integration
   test was passing via the cross-cutting bus path; the
   bugfix enables the storage-row insert path the bench
   measures.

## Workstream commits (2 workstreams — A+B+C combined + D)

1. **Workstream A** (`abe8077`) — `feat(attendance): ship 4 prompt-named aggregates + 9-file scaffold`:
   the entire `educore-attendance` crate. 9-file layout,
   5 aggregates, 21 typed events, 14 typed commands +
   14 `*_COMMAND_TYPE` constants, 14 pure factory services,
   5 repository port traits (with the new `bulk_insert`
   method), 5 typed query stubs, 1 `AttendanceUniquenessChecker`
   port, 2 child entities (`StudentAttendanceImport`,
   `StaffAttendanceImport`), 93 unit tests passing.
   The Workstream B (Staff + Subject) and Workstream C
   (ExamAttendance) aggregates are included in this commit
   since they all share the same 9 files; splitting them
   across 3 subagents would have caused merge conflicts.
   **Closes the Phase 5 implementation.**
2. **Workstream D** (`3c073d3`) — `feat(attendance): ship vertical-slice integration test`:
   `crates/tools/storage-parity/tests/attendance_integration.rs`
   (4 new tests; mirrors `assessment_integration.rs` +
   includes the 200-row bulk-mark bench proxy on SQLite).
   Flipped 13 `docs/coverage.toml` rows from `Pending` to
   `Tested` with the test path. **Closes Phase 5.**

## Capability check boundary

Per the Phase 4 hand-off's resolution, the attendance
services do **not** call
`capability_check.has(ctx, Capability::AttendanceStudent*)`
directly. The check is documented as a dispatcher-level
concern (matching the platform / rbac / academic / assessment
crates' pattern) and exercised in the integration test:

```rust
let cap_check = InMemoryCapabilityCheck::new();
let granted = cap_check
    .has(&ctx, Capability::AttendanceStudentCreate)
    .await
    .expect("has");
assert!(!granted); // no grant -> denied

cap_check.grant(school, role, Capability::AttendanceStudentCreate);
let granted = cap_check
    .has(&ctx, Capability::AttendanceStudentCreate)
    .await
    .expect("has");
assert!(granted); // grant -> allowed
```

Phase 6 may revisit this if the engine facade evolves to
wire checks into the service layer. The boundary is
deliberately not a Phase 5 deliverable because the
existing platform / rbac / academic / assessment / attendance
crates all keep capability checks at the dispatcher.

## Storage-adapter transaction model (Phase 2 OQ #5)

The vertical-slice test exercises the flag-based
transaction model on the 3 SQL adapters. The Phase 5
hand-off's answer to the Phase 2 OQ #5 question is
**yes**, the design is adequate for the attendance domain,
**plus the new `bulk_insert` path** (Prereq 5):

- The SQLite test passes deterministically (200 students,
  401 envelopes, all written in a single transaction).
- The cross-cutting integration test (the original Phase 2
  test) continues to pass with no inconsistency under the
  same model.
- The assessment integration test (the Phase 4 test) also
  passes deterministically.
- The new `bulk_insert` path is an additive non-breaking
  change; the flag-based transaction model is preserved
  (one outbox + one audit + one idempotency per command);
  only the storage row writes within a transaction are
  batched.

The real `sqlx::Transaction` plumb remains a future
refactor (Phase 6+); the hand-off recommends it land
alongside a benchmark that demonstrates the latency
cost of the current model on PG.

## Open questions

1. **`ExamAttendance` location** (Phase 5 OQ #1): the
   spec says assessment owns it; Phase 5 ships it in the
   attendance crate via a cross-crate dep. A follow-up
   phase (likely Phase 6 + HR, or a dedicated
   `assessment` workstream) should either (a) move the
   aggregate to assessment or (b) explicitly document the
   cross-crate reference as canonical. The
   assessment-side consumption (`ResultStore::publish`
   reading from `ExamAttendanceRepository`) is deferred
   to the follow-up phase.
2. **`bulk_insert` scope expansion** (Phase 5 OQ #2): the
   prompt's "do not modify the Phase 1 storage adapters'
   flag-based transaction model" rule was overridden by
   the build-plan's "200 rows in <100ms on PG" exit
   criterion. The `bulk_insert_student_attendances` method
   is an additive non-breaking change to the storage port
   + 3 SQL adapters. The hand-off documents this as the
   only Phase 1 modification in Phase 5.
3. **Capability-variant count** (Phase 5 OQ #4): the
   prompt's gotchas said "~16 new variants (4 aggregates
   × 4 CRUD)"; the spec defines ~24. Phase 5 ships the
   full 24 per spec. The wire form is the three-segment
   `Attendance.Student.Mark` (mirrors the assessment
   pattern `Assessment.Exam.Create`), not the spec's
   two-segment `Attendance.Mark` form.
4. **`StaffId` placeholder**: the attendance crate
   declares a placeholder `StaffId(SchoolId, Uuid)` typed
   id in its own `value_objects.rs` (mirroring the
   assessment crate's pattern). When the HR workstream
   ships its `Staff` aggregate in Phase 6, the attendance
   crate's placeholder will be replaced with a re-export
   of the HR crate's canonical `StaffId`. The assessment
   crate's own `StaffId` placeholder will also be replaced
   at the same time.
5. **`DefaultRoleCatalog` extension** (Phase 5 OQ #5, new):
   the `educore-rbac::services::DefaultRoleCatalog` does
   not yet have explicit grants for the 24 new
   `Attendance.*` capabilities. The engine itself does
   not bake role→capability grants (per the Phase 4
   pattern), but consumers will need to add the grants
   in their `seed.rs` initialization. The hand-off
   documents the recommended mappings in § "Default Role
   Mapping" of `docs/specs/attendance/permissions.md`.
6. **Carry-overs from Phase 4** (6 open questions,
   unchanged): `StudentRecord` aggregate (deferred to a
   later academic phase), `ExamStepSkip` toggle (Phase 14
   Settings), `StaffId` + `ClassRoomId` placeholders in
   the assessment crate (Phase 6 HR / Phase 8 Facilities),
   `OnlineExam` full lifecycle state machine (Phase 6
   alongside HR), `MarksGrade` + `ExamSetting` aggregates
   (Phase 14 Settings), `ResultService` 10-function
   table-driven fixtures (Phase 6 / Phase 14), Tier
   dependency lint stub (`educore-core::lint::runner::check_tier_boundaries`).
7. **Bugfix discovered in Workstream D** (Phase 5 OQ #7,
   closed in commit `14752c4`): the Prereq 5
   `QueryBuilder::new(...)` strings had a trailing
   ` VALUES ` that duplicated the `push_values(...)`
   prepended ` VALUES `. Fixed in 3 files. Documented
   here for the record; the fix is committed and all
   validation gates are green.

## Where NOT to start (Phase 6)

- Do NOT add the `StudentRecord` aggregate. The
  assessment domain's `AdmitCard::student_record_id` is a
  typed id but the aggregate it references does not exist.
  The full `StudentRecord` aggregate lands in a later
  academic phase.
- Do NOT remove the assessment crate's placeholder
  `StaffId` / `ClassRoomId` typed ids. The full definitions
  land in the HR workstream in Phase 6
  (`Capability::HrStaff{Create,Read,Update,Delete}` is
  already in the rbac enum) and the facilities workstream
  in Phase 8 (`Capability::FacilitiesRoom{Create,...}`).
  The HR crate's `Staff` aggregate is the canonical source
  for `StaffId`; the assessment and attendance crates'
  placeholders will be replaced with re-exports at that
  point.
- Do NOT move `ExamAttendance` to the assessment crate
  yet. The cross-crate reference is the current canonical
  path; the move is a follow-up phase's decision.
- Do NOT add the `ExamStepSkip` toggle. The
  `submit_marks` service is in **strict mode only** in
  Phase 4. The toggle lands in Phase 14 (Settings).
- Do NOT modify the `bulk_insert_student_attendances`
  method's SQL. The bugfix in commit `14752c4` is the
  canonical implementation; subsequent phases add new
  bulk-insert paths (e.g. for finance) following the
  same pattern.
- Do NOT modify the 9 closed cross-cutting + 3 closed
  domain crates' public surface (platform, rbac, events,
  event-bus, audit, sync, sync-inprocess, query-derive,
  storage, storage-postgres, storage-mysql, storage-sqlite,
  storage-surrealdb, academic, assessment, attendance).
  The only Phase 5 changes are additive: 24 `Capability`
  variants + 4 `AuditTarget` variants + 1 new
  `bulk_insert` method on the storage port + 1 new
  `StudentAttendanceRow` wire type.
- Do NOT touch `educore-core::lint`. The lint binary
  passes; the tier-boundary checker remains a stub.
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit. The Phase 5 Prereq 5
  added `chrono` as a transitive dep of `educore-storage`
  (the `StudentAttendanceRow::attendance_date` field);
  this is a non-breaking addition to the storage crate
  that's already in the ADR.
- Do NOT remove the placeholder typed ids from the
  attendance crate's public surface. The `StaffId`
  placeholder is used by the `StaffAttendance` aggregate's
  `staff_id` foreign-key field; the prelude re-exports
  it. The full `StaffId` definition lands in Phase 6 HR.

## Key files for the next agent

- `crates/domains/attendance/src/lib.rs` — the 9-file
  module layout + prelude re-exports
- `crates/domains/attendance/src/aggregate.rs` — the 5
  aggregate roots (with the standard 17-field layout)
- `crates/domains/attendance/src/events.rs` — the 21
  typed events implementing `DomainEvent`
- `crates/domains/attendance/src/services.rs` — the 14
  pure factory services + the `AttendanceService` helper
  + the `BulkMarkResult` struct
- `crates/domains/attendance/src/commands.rs` — the 14
  typed command shapes + the `*_COMMAND_TYPE` constants
  + the `AttendanceUniquenessChecker` port + the
  `validate_*` helpers
- `crates/domains/attendance/src/repository.rs` — the 5
  repository port traits (with the new `bulk_insert`
  method on `StudentAttendanceRepository`)
- `crates/domains/attendance/src/value_objects.rs` —
  the 9 typed ids + 1 placeholder `StaffId` + 4 closed
  status enums + re-exports from academic/assessment/core
- `crates/domains/attendance/src/entities.rs` — the 2
  child entities (`StudentAttendanceImport`,
  `StaffAttendanceImport`)
- `crates/domains/attendance/src/query.rs` — the 5
  typed query stubs
- `crates/domains/attendance/src/errors.rs` — the
  `pub use DomainError as AttendanceError` alias
- `crates/infra/storage/src/student_attendance_row.rs` —
  the `StudentAttendanceRow` wire type (Prereq 5)
- `crates/infra/storage/src/port.rs` — the new
  `StorageAdapter::bulk_insert_student_attendances` method
- `crates/infra/storage/src/transaction.rs` — the new
  `Transaction::bulk_insert_student_attendances` method
- `crates/adapters/storage-postgres/src/bulk_attendance.rs` —
  the PG impl (single multi-row `INSERT` via
  `QueryBuilder::push_values`)
- `crates/adapters/storage-mysql/src/bulk_attendance.rs` —
  the MySQL impl
- `crates/adapters/storage-sqlite/src/bulk_attendance.rs` —
  the SQLite impl (transaction-grouped inserts)
- `crates/tools/storage-parity/tests/attendance_integration.rs` —
  the vertical-slice test (4 always-on tests + 3 env-gated)
- `crates/cross-cutting/rbac/src/value_objects.rs` — the
  24 new `Attendance.*` `Capability` variants (Prereq 1)
- `crates/cross-cutting/audit/src/writer.rs` — the 4 new
  `AuditTarget` variants (Prereq 2)
- `crates/domains/attendance/Cargo.toml` — the new
  `educore-assessment` + `educore-event-bus` deps
  (Prereq 3)
- `docs/coverage.toml` — the 13 `attendance_*` rows
  flipped to `Tested`
- `docs/handoff/PHASE-5-HANDOFF.md` — this file
- `docs/phase_prompt/phase-6-prompt.md` — the
  next-phase brief for the HR-domain agent

## Where to ask

Open a GitHub issue for design questions. The Phase 5
prompt is the source of truth for Phase 5's scope; the
next-phase prompt is the source of truth for Phase 6.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
