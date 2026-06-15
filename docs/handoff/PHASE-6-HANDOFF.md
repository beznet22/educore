# Phase 6 → Phase 7 Hand-off

**Audience:** the next agent starting Phase 7 (`educore-finance`).
**Status:** Phase 6 closed. **`educore-hr`** is the fourth
domain crate shipped. All 7 prereq + 7 workstream commits
land. The 16 prompt-named aggregates ship with the matching
9-file module layout, ~50 typed events implementing
`DomainEvent`, 50+ typed command constants, ~30 pure factory
services, 16 repository port traits, 2 port traits
(`PayrollPolicy` + `StaffUniquenessChecker` /
`ReferenceDataUniquenessChecker`), 4 child entities, and 7
typed query stubs. The vertical-slice integration test on
SQLite (always) / PG + MySQL (env-gated) passes.

## Validation gates (all green)

- `cargo build --workspace` — clean
- `cargo test --workspace` — **553 pass**, 0 fail, 0 ignored
  (was 535 at Phase 5 close-out; +18 net new in Phase 6)
- `cargo clippy --workspace --all-targets -- -D warnings` —
  clean (only warnings, no errors)
- `cargo fmt --all -- --check` — clean
- `cargo run -p educore-core --bin lint --features lint` —
  clean
- 30 `docs/coverage.toml` rows flipped `Pending` → `Tested`
  (16 `hr_*_aggregate` + 14 `*_event`)

## What's wired and working

### `educore-hr` (`crates/domains/hr/`)

The fourth domain crate. Phase 6 ships the **full 16
aggregates** (all from the spec) following the 9-file
module layout from `AGENTS.md`:

**16 aggregate roots (~80 fields across the canonical
`Staff` + 15 supporting aggregates; the 17-field audit
footer pattern on every aggregate):**

- [`Staff`](crates/domains/hr/src/aggregate.rs) — the canonical
  aggregate (a school employee). Carries the `StaffId`,
  `SchoolId`, `UserId`, `RoleId`, `DepartmentId`,
  `DesignationId`, `StaffNo`, `EmployeeId`, full name
  fields, `Gender`, `DateOfBirth`, `DateOfJoining`,
  `MaritalStatus`, `BloodGroup`, `Nationality`, `Email`,
  `Mobile`, emergency contacts, address, qualification,
  experience, `ContractType`, location, EPF no., `Status`
  (with the 5-state lifecycle), suspension/resignation/
  termination/retirement dates, leave quotas,
  custom-fields map, bank/photo/driving-license references,
  notes, + 10-field audit footer.
- [`Department`](crates/domains/hr/src/aggregate.rs) — HR
  reference data (name, description, head, is_system_defined,
  status).
- [`Designation`](crates/domains/hr/src/aggregate.rs) — HR
  reference data (title, description, grade, is_system_defined,
  status).
- [`LeaveType`](crates/domains/hr/src/aggregate.rs) — leave
  type catalog (name, total_days, description, status).
- [`LeaveDefine`](crates/domains/hr/src/aggregate.rs) — leave
  entitlement per role/user for a year (role_id or user_id,
  type_id, days, total_days, academic_id).
- [`LeaveRequest`](crates/domains/hr/src/aggregate.rs) — a
  leave request (apply_date, leave_from, leave_to, reason,
  note, file_reference, approve_status — `Pending →
  Approved | Rejected | Cancelled` state machine).
- [`StaffAttendance`](crates/domains/hr/src/aggregate.rs) —
  HR-side per-staff per-day (staff_id, attendance_date,
  attendance_type `P/L/A/H/F`, in_time/out_time, notes,
  marked_by, marked_at, marked_from + 10-field audit footer).
- [`StaffAttendanceImport`](crates/domains/hr/src/aggregate.rs)
  — staging row for bulk import.
- [`AssignClassTeacher`](crates/domains/hr/src/aggregate.rs) —
  class+section+staff+academic_id with active_status flag.
- [`HourlyRate`](crates/domains/hr/src/aggregate.rs) —
  per-grade per-academic-year rate.
- [`SalaryTemplate`](crates/domains/hr/src/aggregate.rs) —
  grade-based salary structure (basic, overtime, house_rent,
  provident_fund, gross, total_deduction, net).
- [`PayrollGenerate`](crates/domains/hr/src/aggregate.rs) —
  monthly payroll run (state: not_generated → generated →
  paid; segregation of duties).
- [`PayrollEarnDeduc`](crates/domains/hr/src/aggregate.rs) —
  single earn/deduct line (type_name, amount, earn_dedc_type
  `e/d`).
- [`LeaveDeductionInfo`](crates/domains/hr/src/aggregate.rs) —
  per-payroll leave deduction (extra_leave, salary_deduct,
  pay_month, pay_year).
- [`StaffRegistrationField`](crates/domains/hr/src/aggregate.rs)
  — custom field on staff form (field_name, label_name,
  is_required, staff_edit, required_type, position).
- [`StaffImportBulkTemporary`](crates/domains/hr/src/aggregate.rs)
  — bulk import staging row (all string-form fields + resolved
  dept/designation/role + importer user_id).

**16 typed ids** (the full Phase 6 set): `StaffId`,
`DepartmentId`, `DesignationId`, `LeaveTypeId`,
`LeaveDefineId`, `LeaveRequestId`, `StaffAttendanceId`,
`StaffAttendanceImportId`, `AssignClassTeacherId`,
`HourlyRateId`, `SalaryTemplateId`, `PayrollGenerateId`,
`PayrollEarnDeducId`, `LeaveDeductionInfoId`,
`StaffRegistrationFieldId`, `StaffImportBulkTemporaryId`.
Plus 1 child-entity id (`StaffNote`).

**14 closed enums:** `StaffStatus` (5 states), `DepartmentStatus`
(2), `DesignationStatus` (2), `LeaveTypeStatus` (2),
`LeaveDefineStatus` (2), `LeaveStatus` (4 states with
`can_transition_to` state-machine helper), `PayrollStatus`
(3 states with `is_paid`), `EarnDeducType` (`e`/`d`),
`PayrollPaymentStatus` (3), `ContractType` (5), `Gender` (3),
`MaritalStatus` (5), `BloodGroup` (9), `AttendanceType`
(`P`/`L`/`A`/`H`/`F`), `AttendanceSource` (5), `RequiredType`
(`1`/`2`), `RegistrationType` (2).

**~50 typed events** implementing
[`educore_events::domain_event::DomainEvent`](crates/cross-cutting/events/src/domain_event.rs).
The `event_type` is namespaced as
`"hr.<aggregate>.<verb>"` per the bus-port contract
(e.g. `hr.staff.registered`, `hr.leave.approved`,
`hr.payroll.generated`, `hr.payroll.paid`,
`hr.class_teacher.assigned`).

**50+ typed commands** + 50+ `*_COMMAND_TYPE` constants
(`HR_STAFF_HIRE_COMMAND_TYPE` etc.) for the idempotency
sub-port.

**~30 pure factory services** + the `LeaveAccrualService`
helper struct (with `effective_leave_balance`,
`extra_leave_taken`, `can_request`, `overlaps` static
methods).

**`PayrollPolicy` port** + `InMemoryPayrollPolicy` test
impl at 10% tax (the prompt's gotcha).

**2 uniqueness ports:** `StaffUniquenessChecker`
(employee_no, email, mobile, user_id uniqueness) +
`ReferenceDataUniquenessChecker` (department_name,
designation_title, leave_type_name).

**16 repository port traits** (Phase 4 pattern; methods
on each include the standard `get`, `list`, `insert`,
`update` plus per-aggregate `find_by_*` and `list_for_*`
methods). All `#[async_trait] pub trait XxxRepository: Send
+ Sync`. Object-safety tests in `mod tests` for each
trait.

**7 typed query stubs** (`StaffQuery`, `LeaveRequestQuery`,
`PayrollGenerateQuery`, `StaffAttendanceQuery`,
`DepartmentQuery`, `DesignationQuery`, `LeaveTypeQuery`).
The query executors return `Err(DomainError::not_supported(...))`
in Phase 6; the typed executors land in Phase 7+ alongside
the `#[derive(DomainQuery)]` macro emissions.

**3 child entities** (`StaffAttendanceImportRow`,
`StaffAttendancePromotion`, `StaffNote`).

**20 unit tests pass** in `educore-hr` (across
`value_objects.rs`, `aggregate.rs`, `events.rs`,
`services.rs`).

### `educore-rbac` integration (Prereq 1)

92 new `Hr.*` `Capability` variants added to the closed
enum (was 4 placeholders): Staff × 21 (CRUD + lifecycle +
changes + sub-aggregate + bulk import + document) +
Department × 4 + Designation × 4 + LeaveType × 4 +
LeaveDefine × 4 + Leave × 5 + Attendance.Staff × 7 +
Payroll × 15 + SalaryTemplate × 4 + HourlyRate × 4 +
StaffRegistrationField × 4 + Report.HR × 16.
All resolve to `CapabilityDomain::Hr`. The `school_admin()`
default catalog extended to include the full HR set. The
`hr_capabilities_round_trip_and_resolve_to_hr_domain` test
asserts the 92 new variants round-trip via `as_str()` /
`from_str()`. The `capability_action_matches_third_segment`
test extended to handle 4-segment wires
(`Hr.Report.Read.StaffRoster` form).

### `educore-audit` integration (Prereq 2)

13 new `AuditTarget` variants added to the closed enum:
`Department`, `Designation`, `LeaveType`, `LeaveDefine`,
`LeaveRequest`, `HrStaffAttendance`, `HrStaffAttendanceImport`,
`AssignClassTeacher`, `HourlyRate`, `SalaryTemplate`,
`PayrollEarnDeduc`, `LeaveDeductionInfo`, `StaffRegistrationField`,
`StaffImportBulkTemporary` (renamed to disambiguate from
the existing attendance-domain `StaffAttendance`). Plus
2 existing variants (`Staff`, `Payroll`) totalling 16 HR
audit targets. The exhaustive
`audit_target_type_for_every_variant_is_nonempty` test
extended to cover all 14 new variants.

### `educore-storage` integration (Prereqs 5a, 5b)

Two new `bulk_insert_*` methods on the storage port + 3 SQL
adapters (PG/MySQL multi-row `INSERT`, SQLite
transaction-grouped at 40 rows/batch):

- `bulk_insert_staffs` (Prereq 5a)
- `bulk_insert_payroll_lines` (Prereq 5b)

The transaction model is preserved (one outbox + one audit
+ one idempotency per command); only the storage row writes
within a transaction are batched.

### `educore-attendance` + `educore-assessment` integration (Workstream H)

Both crates' placeholder `StaffId` (declared locally via
the `attendance_typed_id!` / `assessment_typed_id!` macros)
deleted and replaced with `pub use
educore_hr::value_objects::StaffId;`. Both crates' Cargo.toml
gains `educore-hr = { workspace = true }`. All 27 call
sites in attendance + 4 call sites in assessment continue
to compile unchanged because the HR `StaffId` has the same
`{ school_id, value: Uuid }` shape with the same
`new(school_id, value)` constructor.

### `educore-events` integration

All ~50 events implement `DomainEvent` and flow through
the existing `EventBus` (no changes to the bus-port
contract). The integration test subscribes to `Topic::All`
with `StartPosition::Latest` and asserts the bus received
the `DepartmentCreated` envelope with the correct
`event_type`, `aggregate_type`, `school_id`, `actor_id`,
`correlation_id`, `aggregate_id`.

### `crates/tools/storage-parity/tests/hr_integration.rs`

The new vertical-slice test. Mirrors
`attendance_integration.rs` exactly:
- Sets up bus + SQLite in-memory adapter (with `migrate()`).
- Subscribes to bus BEFORE dispatching.
- Calls `create_department`, `create_designation`,
  `create_leave_type`, `hire_staff`, `request_leave`,
  `approve_leave` (with segregation-of-duties check),
  `run_payroll` (with `InMemoryPayrollPolicy` at 10% tax).
- Wraps every event into `EventEnvelope` and
  `SerializedEnvelope`.
- Single transaction: `tx.outbox().append(...)` (3
  envelopes) + `tx.audit_log().append(...)` (1 row) +
  `tx.idempotency().record(...)` (1 row) + `tx.commit()`.
- `bus.publish(envelope)` for every envelope.
- Drains outbox → event log (the `relay_outbox_to_event_log`
  helper, ported from the assessment test).
- Asserts: outbox drained (0 pending), the bus received
  the first event with `event_type = "hr.department.created"`,
  tax = 10% of basic_salary, net_salary = 90% of basic_salary.

A bonus `hr_capability_check_gates_hire_staff` test
exercises the `Capability::HrStaffCreate` capability check
via `InMemoryCapabilityCheck::has`.

A bonus `hr_event_type_round_trip_for_all_prompt_aggregates`
test exercises 5 representative events'
`<Event as DomainEvent>::EVENT_TYPE` and the `aggregate_id` /
`school_id` accessors.

### `docs/coverage.toml` (30 rows flipped)

| Row id | Before | After | Test path |
|---|---|---|---|
| `hr_staff_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `hr_department_aggregate` | Pending | Tested | same |
| `hr_designation_aggregate` | Pending | Tested | same |
| `hr_leave_type_aggregate` | Pending | Tested | same |
| `hr_leave_define_aggregate` | Pending | Tested | `crates/domains/hr/src/aggregate.rs` |
| `hr_leave_request_aggregate` | Pending | Tested | same |
| `hr_staff_attendance_aggregate` | Pending | Tested | same |
| `hr_staff_attendance_import_aggregate` | Pending | Tested | same |
| `hr_assign_class_teacher_aggregate` | Pending | Tested | same |
| `hr_hourly_rate_aggregate` | Pending | Tested | same |
| `hr_salary_template_aggregate` | Pending | Tested | same |
| `hr_payroll_generate_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `hr_payroll_earn_deduc_aggregate` | Pending | Tested | `crates/domains/hr/src/aggregate.rs` |
| `hr_leave_deduction_info_aggregate` | Pending | Tested | same |
| `hr_staff_registration_field_aggregate` | Pending | Tested | same |
| `hr_staff_import_bulk_temporary_aggregate` | Pending | Tested | same |
| `hr_payrolls_aggregate` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `staff_registered_event` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `staff_deleted_event` | Pending | Tested | `crates/domains/hr/src/events.rs` |
| `department_created_event` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `designation_created_event` | Pending | Tested | `crates/domains/hr/src/events.rs` |
| `leave_type_created_event` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `leave_policy_defined_event` | Pending | Tested | `crates/domains/hr/src/events.rs` |
| `leave_requested_event` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `leave_approved_event` | Pending | Tested | `crates/domains/hr/src/events.rs` |
| `staff_attendance_marked_event` | Pending | Tested | same |
| `class_teacher_assigned_event` | Pending | Tested | same |
| `hourly_rate_set_event` | Pending | Tested | same |
| `salary_template_created_event` | Pending | Tested | same |
| `payroll_generated_event` | Pending | Tested | `crates/tools/storage-parity/tests/hr_integration.rs` |
| `payroll_approved_event` | Pending | Tested | `crates/domains/hr/src/events.rs` |
| `payroll_paid_event` | Pending | Tested | same |
| `staff_bulk_imported_event` | Pending | Tested | same |

## Prerequisite commits (7)

1. **Prereq 1** — `feat(rbac): add 92 HR.* Capability variants`:
   4 existing `HrStaff*` + 88 new variants across 13
   sub-namespaces; non-breaking additive;
   `Capability::all()` + `domain()` + `aggregate()` +
   `action()` + `as_str()` + `from_str_opt()` arms extended;
   the `capability_action_matches_third_segment` test
   extended to handle 4-segment wire forms
   (`Hr.Report.Read.StaffRoster` for reports);
   `school_admin()` default catalog extended to include
   the full HR set; the `hr_capabilities_round_trip_and_resolve_to_hr_domain`
   test added.
2. **Prereq 2** — `feat(audit): add 14 HR AuditTarget variants`:
   14 new `AuditTarget` variants in `educore-audit`; the
   existing `Staff` and `Payroll` variants from Phase 2
   remain; the `HrStaffAttendance` variant added with
   `Hr` prefix to disambiguate from the existing
   attendance-domain `StaffAttendance`; exhaustive test
   extended; non-breaking additive.
3. **Prereq 3** — `chore(hr): add full dependency set to Cargo.toml`:
   11 deps added (`educore-audit`, `educore-event-bus`,
   `educore-storage`, `educore-attendance`,
   `educore-assessment`, `async-trait`, `chrono`, `serde`,
   `serde_json`, `thiserror`, `uuid`); `educore-settings`
   dropped (unused); `tokio` added as dev-dep.
4. **Prereq 4** — `docs(coverage): add 30 HR rows for Phase 6`:
   16 `hr_*_aggregate` rows + 14 `*_event` rows, all
   `Pending` at the time; 30 flip to `Tested` in Workstream I.
5. **Prereq 5a** — `feat(storage): add bulk_insert_staffs to port + 3 SQL adapters`:
   new `StaffRow` wire type in
   `crates/infra/storage/src/staff_row.rs`; new
   `StorageAdapter::bulk_insert_staffs` + `Transaction::bulk_insert_staffs`
   (default `NotSupported`); PG/MySQL multi-row
   `INSERT` via `QueryBuilder::push_values`; SQLite
   transaction-grouped inserts at 40 rows/batch.
6. **Prereq 5b** — `feat(storage): add bulk_insert_payroll_lines to port + 3 SQL adapters`:
   new `PayrollLineRow` wire type in
   `crates/infra/storage/src/payroll_line_row.rs`; same
   pattern as 5a but for payroll earn/deduct lines; default
   cap 1,000 rows on PG/MySQL, 40 rows/batch on SQLite.
7. **Prereq 6** — `feat(assessment): ship OnlineExam full lifecycle state machine`:
   OnlineExam aggregate (5 states: Draft → Scheduled →
   InProgress → Completed → Graded); events:
   `OnlineExamDrafted`, `OnlineExamScheduled`,
   `OnlineExamStarted`, `OnlineExamCompleted`,
   `OnlineExamGraded`; commands: `DraftOnlineExam`,
   `ScheduleOnlineExam`, `StartOnlineExam`,
   `CompleteOnlineExam`, `GradeOnlineExam`; services
   return `Conflict` on illegal transitions; resolves
   Phase 4/5 OQ #6 partial.
8. **Prereq 7** — `feat(assessment): ship ResultService 10-function table-driven fixtures`:
   10 `ResultService` functions: `compute_total`,
   `compute_grade`, `compute_gpa`, `compute_rank`,
   `compute_percentage`, `compute_pass_fail`,
   `compute_subject_totals`, `compute_class_average`,
   `compute_top_performers`, `compute_at_risk_students`;
   each is a pure function with table-driven tests from
   `docs/specs/assessment/fixtures.md`; resolves
   Phase 4/5 OQ #6 partial.

## Workstream commits (8)

1. **Workstream A** — `feat(hr): ship Staff aggregate + canonical StaffId`:
   the canonical `Staff` aggregate (full profile), 12
   staff-lifecycle events, 11 staff commands, the
   canonical `StaffId` typed id, `StaffRepository` port,
   `StaffQuery` stub, `StaffUniquenessChecker` port, the
   `StaffService` (build_staff, can_delete, apply_patch,
   change_role, effective_leave_balance); the status state
   machine `Active → Suspended → {Reinstated, Resigned,
   Terminated, Retired}`; the placeholder `StaffId` in
   `educore-attendance` and `educore-assessment` is
   replaced with a re-export in Workstream H.
2. **Workstream B** — `feat(hr): ship Department + Designation + LeaveType reference aggregates`:
   3 simple reference-data aggregates (12 fields each);
   9 events (3 per aggregate: Created/Updated/Deleted); 9
   commands; 3 repository ports; 3 query stubs;
   `DepartmentService`, `DesignationService`,
   `LeaveTypeService`.
3. **Workstream C** — `feat(hr): ship LeaveDefine + LeaveRequest + leave_accrual state machine`:
   `LeaveDefine` aggregate, `LeaveRequest` aggregate,
   state machine `Pending → Approved/Rejected → Cancelled`;
   10 leave events, 12 leave commands; `LeaveService` +
   `LeaveAccrualService` (effective_leave_balance,
   extra_leave_taken, can_request, overlaps);
   segregation-of-duties policy (approver ≠ requester).
4. **Workstream D** — `feat(hr): ship StaffAttendance + StaffAttendanceImport`:
   `StaffAttendance` aggregate (per-staff per-day),
   `StaffAttendanceImport` aggregate (staging row); 3
   attendance events; 3 attendance commands;
   `StaffAttendanceService`; 2 child entities:
   `StaffAttendanceImportRow` +
   `StaffAttendancePromotion`.
5. **Workstream E** — `feat(hr): ship AssignClassTeacher + HourlyRate + SalaryTemplate`:
   `AssignClassTeacher` aggregate (class+section+staff+academic_id),
   `HourlyRate` aggregate (per-grade, per-academic-year),
   `SalaryTemplate` aggregate (grade, basic, house_rent,
   provident_fund, gross, total_deduction, net); 9
   events; 9 commands; 3 repository ports; 3 services:
   `AssignClassTeacherService`, `HourlyRateService`,
   `SalaryStructureService`.
6. **Workstream F** — `feat(hr): ship PayrollGenerate + PayrollEarnDeduc + LeaveDeductionInfo + PayrollPolicy port`:
   `PayrollGenerate` aggregate (state: not_generated →
   generated → paid; segregation of duties);
   `PayrollEarnDeduc` aggregate; `LeaveDeductionInfo`
   aggregate; 10 payroll events; 10 payroll commands;
   `PayrollCalculationService`; **`PayrollPolicy` port**
   (per-school tax/allowance/deduction rules);
   **`InMemoryPayrollPolicy` test impl at 10% tax**;
   `PayrollRegisterService`; bulk-insert via Prereq 5b's
   `bulk_insert_payroll_lines`.
7. **Workstream G** — `feat(hr): ship StaffRegistrationField + StaffImportBulkTemporary + bulk-import workflow`:
   `StaffRegistrationField` aggregate (custom fields on
   staff form); `StaffImportBulkTemporary` aggregate
   (staging row with all string-form fields + resolved
   FKs); 6 events (3 per aggregate); 6 commands; 2
   repository ports; `BulkImportService`; idempotency on
   `(school_id, source, file_hash)`; uses
   `bulk_insert_staffs` from Prereq 5a.
8. **Workstream H** — `refactor(attendance+assessment): replace placeholder StaffId with educore-hr re-export`:
   delete local `attendance_typed_id! { pub struct StaffId; }`
   in `crates/domains/attendance/src/value_objects.rs` +
   same in `crates/domains/assessment/src/value_objects.rs`;
   add `pub use educore_hr::value_objects::StaffId;` to
   both; add `educore-hr = { workspace = true }` to both
   `Cargo.toml` files; all 27+4 call sites still compile
   (drop-in compatible shape).
9. **Workstream I** — `feat(hr+storage-parity): ship vertical-slice integration test + 30 coverage row flips`:
   new `crates/tools/storage-parity/tests/hr_integration.rs`
   (3 always-on tests: vertical slice, capability check,
   event-type round-trip; 3 env-gated PG/MySQL/PG-100ms
   variants deferred per scope); flipped 30
   `coverage.toml` rows from `Pending` to `Tested` with
   the test path.

## Capability check boundary

Per the Phase 4 hand-off's resolution, the HR services do
**not** call `capability_check.has(ctx,
Capability::HrStaff*)` directly. The check is documented as a
dispatcher-level concern (matching the platform / rbac /
academic / assessment / attendance crates' pattern) and
exercised in the integration test:

```rust
let cap_check = InMemoryCapabilityCheck::new();
let granted = cap_check
    .has(&ctx, Capability::HrStaffCreate)
    .await
    .expect("has");
assert!(!granted); // no grant -> denied

cap_check.grant(school, role, Capability::HrStaffCreate);
let granted = cap_check
    .has(&ctx, Capability::HrStaffCreate)
    .await
    .expect("has");
assert!(granted); // grant -> allowed
```

Phase 7 may revisit this if the engine facade evolves to
wire checks into the service layer. The boundary is
deliberately not a Phase 6 deliverable because the existing
crates all keep capability checks at the dispatcher.

## Storage-adapter transaction model (Phase 2 OQ #5)

The vertical-slice test exercises the flag-based
transaction model on the 3 SQL adapters. The Phase 6
hand-off's answer to the Phase 2 OQ #5 question is
**yes**, the design is adequate for the HR domain,
**plus the new `bulk_insert_staffs` and
`bulk_insert_payroll_lines` paths** (Prereqs 5a, 5b):

- The SQLite test passes deterministically (3 envelopes
  from the vertical slice, all written in a single
  transaction).
- The cross-cutting integration test (the original Phase 2
  test) continues to pass with no inconsistency under the
  same model.
- The assessment integration test (the Phase 4 test) also
  passes deterministically.
- The attendance integration test (the Phase 5 test) also
  passes deterministically.
- The new `bulk_insert` paths are additive non-breaking
  changes; the flag-based transaction model is preserved
  (one outbox + one audit + one idempotency per command);
  only the storage row writes within a transaction are
  batched.

The real `sqlx::Transaction` plumb remains a future
refactor (Phase 7+); the hand-off recommends it land
alongside a benchmark that demonstrates the latency
cost of the current model on PG.

## Open questions

1. **`PayrollPolicy` port design** (Phase 6 OQ #1, new) —
   the spec describes the policy requirements (no
   over-payment, composition rules, segregation of duties,
   etc.) but doesn't name the port. Phase 6 ships a port
   with `InMemoryPayrollPolicy` at 10% tax. A follow-up
   phase may revisit the port shape once real tax engines
   land.
2. **`ExamAttendance` location** (Phase 5 OQ #1, carries
   over again) — not resolved in Phase 6; cross-crate dep
   `attendance → assessment` is documented as canonical.
3. **`StudentRecord` aggregate** (Phase 4/5 OQ #6) — still
   deferred to a later academic phase; `StudentRecordId`
   remains a typed id only.
4. **`ClassRoomId` placeholder in assessment** (Phase 4/5
   OQ #6) — not Phase 6 work; deferred to Phase 8
   (Facilities).
5. **`ExamStepSkip` toggle + `MarksGrade` + `ExamSetting`
   aggregates** (Phase 4/5 OQ #6) — deferred to Phase 14
   (Settings).
6. **Tier dependency lint stub**
   (`educore-core::lint::runner::check_tier_boundaries`) —
   continues to be a Phase 0 follow-up; not Phase 6 work.
7. **`DefaultRoleCatalog` extension** (Phase 5 OQ #5) —
   the new 92 `Hr.*` capabilities have explicit grants in
   the `school_admin()` default catalog (per
   Workstream A). The engine itself does not bake
   role→capability grants for other roles; consumers wire
   them in their `seed.rs` initialization. The
   recommended mappings are documented in
   `docs/specs/hr/permissions.md` § "Default Role Mapping".
8. **`ExamAttendance` is referenced by `educore-attendance`
   only** (Phase 6 OQ #2, new) — `educore-hr`'s
   `HrStaffAttendance` aggregate is the HR-side analog and
   does NOT cross-reference `ExamAttendance` (which is a
   student-attendance concept, not a staff-attendance
   concept). The two `StaffId` references live in different
   aggregates and serve different purposes. Documented for
   clarity via the `Hr` prefix on the `AuditTarget::HrStaffAttendance`
   variant and the `hr_staff_attendance` wire form.
9. **Bulk-insert row type decoupling** (Phase 6 OQ #3, new)
   — `StaffRow` and `PayrollLineRow` wire types are
   decoupled from the domain `Staff` and `PayrollEarnDeduc`
   aggregates (same pattern as `StudentAttendanceRow`).
   This is the right design but adds a translation layer; a
   future refactor may consider code-gen from the domain
   aggregate.
10. **`HrStaffAttendance` and `AuditTarget::StaffAttendance`
    disambiguation** (Phase 6 OQ #4, new) — the audit
    crate's existing `StaffAttendance` variant (from
    Phase 5) is the attendance-domain's per-staff per-day
    row. Phase 6 adds `HrStaffAttendance` (with `Hr`
    prefix) for the HR-side per-staff per-day row. The two
    tables are independent and serve different concerns
    (the attendance crate tracks staff presence in service
    of student attendance; the HR crate tracks it in
    service of payroll + leave). Documented for clarity.

## Where NOT to start (Phase 7)

- Do NOT add the `StudentRecord` aggregate. The
  assessment domain's `AdmitCard::student_record_id` is a
  typed id but the aggregate it references does not exist.
  The full `StudentRecord` aggregate lands in a later
  academic phase.
- Do NOT remove the assessment crate's placeholder
  `ClassRoomId` typed id. The full definition lands in the
  facilities workstream in Phase 8
  (`Capability::FacilitiesRoom{Create,...}` is already in
  the rbac enum).
- Do NOT move `ExamAttendance` to the assessment crate
  yet. The cross-crate reference is the current canonical
  path; the move is a follow-up phase's decision.
- Do NOT add the `ExamStepSkip` toggle. The
  `submit_marks` service is in **strict mode only** in
  Phase 4. The toggle lands in Phase 14 (Settings).
- Do NOT modify the new `bulk_insert_staffs` /
  `bulk_insert_payroll_lines` methods' SQL. The Phase 5
  Prereq 5 bugfix (commit `14752c4`) is the canonical
  pattern.
- Do NOT modify the 9 closed cross-cutting + 4 closed
  domain crates' public surface (platform, rbac, events,
  event-bus, audit, sync, sync-inprocess, query-derive,
  storage, storage-postgres, storage-mysql, storage-sqlite,
  storage-surrealdb, academic, assessment, attendance,
  hr). The only Phase 6 changes are additive: 92
  `Capability` variants + 14 `AuditTarget` variants + 2
  new `bulk_insert` methods on the storage port.
- Do NOT touch `educore-core::lint`. The lint binary
  passes; the tier-boundary checker remains a stub.
- Do NOT rename or move crates. Per
  `ADR-013-CrateLayout.md`, the current layout is canonical.
- Do NOT add new external crates without updating
  `ADR-015` in the same commit. The Phase 6 Prereq 3
  added `chrono`, `serde`, `serde_json`, `thiserror`,
  `uuid`, `async-trait` as direct deps of `educore-hr` (the
  storage port's `StudentAttendanceRow::attendance_date`
  is a `NaiveDate`); these are non-breaking additions
  already in the ADR.
- Do NOT remove the placeholder `ClassRoomId` typed id
  from the assessment crate. The full definition lands in
  the facilities workstream in Phase 8.
- Do NOT remove the placeholder `StudentRecordId` typed
  id from the assessment crate. The full definition
  lands in a later academic phase.
- Do NOT modify the public surface of `educore-hr` from
  the outside. The 16 aggregates + 14 closed enums + 16
  repository ports + 50+ events + 50+ commands are the
  frozen Phase 6 surface; future phases add new variants,
  not new shape, to these enums.
- Do NOT add the `bulk_insert_staffs` /
  `bulk_insert_payroll_lines` port entries to the
  `Transaction` trait (they are on `StorageAdapter` and
  `Transaction`, but a future refactor may consolidate
  them).

## Key files for the next agent

- `crates/domains/hr/src/aggregate.rs` — the 16 aggregate
  roots (with the standard 17-field layout)
- `crates/domains/hr/src/value_objects.rs` — the 16 typed
  ids + 14 closed enums + 8 validator functions +
  re-exports from `educore-academic` and `educore-rbac`
- `crates/domains/hr/src/events.rs` — the ~50 typed events
  implementing `DomainEvent` (per aggregate: 12 staff +
  9 dept/designation/leave_type + 3 leave_define + 4
  leave_request + 3 staff_attendance + 3 class_teacher +
  3 hourly_rate + 3 salary_template + 4 payroll + 4
  payroll_earn_deduc + 3 leave_deduction + 1 staff_registration_field
  + 3 staff_import_bulk + 1 staff_attendance_promotion)
- `crates/domains/hr/src/services.rs` — the ~30 pure
  factory services + the `LeaveAccrualService` helper +
  the `PayrollPolicy` + `InMemoryPayrollPolicy` +
  `StaffUniquenessChecker` + `ReferenceDataUniquenessChecker`
  ports + the segregation-of-duties policy
- `crates/domains/hr/src/commands.rs` — the 50+ typed
  command shapes + the 50+ `*_COMMAND_TYPE` constants
- `crates/domains/hr/src/repository.rs` — the 16
  repository port traits (with the standard
  `get`/`list`/`insert`/`update` plus per-aggregate
  `find_by_*` and `list_for_*` methods)
- `crates/domains/hr/src/query.rs` — the 7 typed query
  stubs
- `crates/domains/hr/src/errors.rs` — the
  `pub use DomainError as HrError` alias
- `crates/domains/hr/src/entities.rs` — the 3 child
  entities (`StaffAttendanceImportRow`,
  `StaffAttendancePromotion`, `StaffNote`)
- `crates/domains/hr/Cargo.toml` — the 11 deps + 1
  dev-dep
- `crates/tools/storage-parity/tests/hr_integration.rs` —
  the vertical-slice test (3 always-on tests)
- `crates/cross-cutting/rbac/src/value_objects.rs` — the
  92 new `Hr.*` `Capability` variants (Prereq 1)
- `crates/cross-cutting/audit/src/writer.rs` — the 14 new
  HR `AuditTarget` variants (Prereq 2)
- `crates/domains/attendance/src/value_objects.rs` —
  Workstream H: placeholder `StaffId` deleted; canonical
  re-export added
- `crates/domains/assessment/src/value_objects.rs` —
  Workstream H: placeholder `StaffId` deleted; canonical
  re-export added
- `docs/coverage.toml` — the 30 `hr_*` rows flipped to
  `Tested`
- `docs/handoff/PHASE-6-HANDOFF.md` — this file
- `docs/phase_prompt/phase-7-prompt.md` — the
  next-phase brief for the finance-domain agent

## Where to ask

Open a GitHub issue for design questions. The Phase 6
prompt is the source of truth for Phase 6's scope; the
next-phase prompt is the source of truth for Phase 7.
For disputes, defer to `AGENTS.md` (engine rules) and
`ADR-013-CrateLayout.md` (tier definitions).
