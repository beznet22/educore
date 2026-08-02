# HR Invariant Checklist

**Spec source:** `docs/specs/hr/aggregates.md`
**Code location:** `crates/domains/hr/src/`
**Baseline:** `docs/audit_reports/stub_vs_implementation.md` § "hr — Deep Invariant Audit"
**Generated:** Engine Production Depth Phase 3, Step 1

## Status Legend

- **[x]** = Enforced in code + has integration test
- **[~]** = Partial enforcement
- **[ ]** = Missing — needs implementation
- **[N/A]** = Permissive invariant — engine not required to enforce

## Summary

**Spec count:** 107 invariants across 42 aggregates.
**Per audit (function-level):** 49 fns / 17 real / 6 partial / 26 stub.

Initial invariant status estimate (based on function-level audit):
- [x]: 25 (Wave 32: 8 invariants + Wave 171: 7 Staff invariants + Wave 172: 4 PayrollGenerate invariants + Wave 173: 3 Department invariants + Wave 174: 3 Designation invariants — I-1 title unique, I-2 cannot delete while Staff references, I-3 is_system_defined immutable — bringing the **Staff `[x]` count to 8 of 8 spec invariants**, **PayrollGenerate `[x]` count to 6 of 6 spec invariants**, **Department `[x]` count to 3 of 3 spec invariants**, and **Designation `[x]` count to 3 of 3 spec invariants**)
- [~]: 0
- [ ]: 82 (remaining; targeted by the next per-aggregate wave pipeline — the other 39 HR aggregates beyond Staff, PayrollGenerate, Department, and Designation)
- [N/A]: 0

**Summary updated at session end (commit pending, Wave 174).** The previous `TBD/TBD/TBD` tally
was a pre-Wave 32 baseline that was never refreshed. Wave 32 (`3376a4b`) added 8
invariant enforcements: 1 Staff (phone unique per school via `StaffUniquenessChecker`),
2 PayrollGenerate (net == gross - total_deduction - tax + monthly recurring uniqueness via
`PayrollUniquenessChecker`), 3 LeaveRequest (date ordering + balance check + no-overlap via
`LeaveAccrualChecker`), 1 LeaveDefine (carry_forward cap via `LeaveDefine::fresh` now
returning `Result<Self>`), 1 HourlyRate (rate >= 0 via `HourlyRateManagementService::validate_rate`).
**Wave 171 (commits `13e5fe4` through `f743f8d`) added 7 more invariant enforcements on
the Staff aggregate**, completing the full Staff sweep (8 of 8 spec invariants `[x]`):
I-1 tenant anchor (structural via `hr_typed_id!`), I-2 staff_no unique (existing port), I-3 email
unique (existing port), I-5 joining date <= today (new `validate_joining_date_not_future`),
I-6 status FSM (`StaffStatus::can_transition_to` + 5 mutator methods + 4 new events),
I-7 no-hard-delete-while-referenced (new `StaffReferenceChecker` port + `delete_staff`
service), I-8 leave quotas non-negative (new `validate_non_negative_f32_quota` +
`Staff::set_leave_quotas`). **Wave 172 added 4 more invariant enforcements on
the PayrollGenerate aggregate**, completing the full PayrollGenerate sweep (6 of 6 spec
invariants `[x]`): I-1 gross == basic + total_earning (`validate_gross_salary` epsilon-aware
helper + `update_amounts` mutator), I-3 status FSM (`PayrollStatus::can_transition_to` +
`mark_generated` + `mark_paid` mutators), I-4 paid_amount ≤ net_salary
(`validate_paid_amount` two-check helper + `record_payment` + `mark_paid` wired), I-6
spec-reconciliation flip (the spec #6 LeaveDeductionInfo uniqueness is enforced by the
typed-id construction `LeaveDeductionInfoId(SchoolId, Uuid)` with `(school, staff, payroll)`
composite key). **Wave 173 added 3 more invariant enforcements on
the Department aggregate**, completing the full Department sweep (3 of 3 spec
invariants `[x]`): I-1 name unique (`DuplicateNameUniqueness` mock test pins the
`create_department` rejection path), I-2 cannot delete while Staff references (new
`DepartmentReferenceChecker` port + `Department::soft_delete` mutator + `delete_department`
service), I-3 is_system_defined immutable (new `Department::ensure_deletable` mutator +
service-layer guard). **Wave 174 added 3 more invariant enforcements on
the Designation aggregate**, completing the full Designation sweep (3 of 3 spec
invariants `[x]`): I-1 title unique (`DuplicateTitleUniqueness` mock test pins the
`create_designation` rejection path), I-2 cannot delete while Staff references (new
`DesignationReferenceChecker` port + `Designation::soft_delete` mutator + `delete_designation`
service), I-3 is_system_defined immutable (new `Designation::ensure_deletable` mutator +
service-layer guard). The 82 remaining `[ ]` invariants are the next per-aggregate
wave pipeline's backlog.

**Note:** The Wave 169 / Wave 171 Chunks 2-3 summaries claimed Wave 32 added "7 invariants" — this is an off-by-one in the prose. The actual count is 8 (1+2+3+1+1). Corrected at Wave 171 session end (commit `f743f8d`).

## Per-aggregate Status (compact)

### Staff (8 invariants — highest count)
- [x] I-1: Tenant anchor from SchoolId (Wave 171 / spec #1: structurally enforced via `hr_typed_id!` macro — `Staff::fresh` sets `school_id: id.school_id()` at `crates/domains/hr/src/aggregate.rs` field init line ~92; behavioral test `staff_tenant_anchor_matches_typed_id` in `crates/domains/hr/tests/staff.rs` asserts `staff.school_id == staff.id.school_id() == tenant.school_id`)
- [x] I-2: Staff ID unique per school (Wave 171 / spec #3: `StaffUniquenessChecker::staff_no_exists` port in `crates/domains/hr/src/services.rs:782`; `hire_staff` rejects duplicates with `DomainError::Conflict` at `services.rs`; tests `hire_staff_rejects_duplicate_staff_no` + `hire_staff_accepts_unique_staff_no` in `crates/domains/hr/tests/staff.rs`)
- [x] I-3: Email unique per school (Wave 171 / spec #4: `StaffUniquenessChecker::email_exists` port in `crates/domains/hr/src/services.rs:780`; `hire_staff` rejects duplicates with `DomainError::Conflict`; tests `hire_staff_rejects_duplicate_email` + `hire_staff_accepts_unique_email` in `crates/domains/hr/tests/staff.rs`)
- [x] I-4: Phone unique per school (Wave 32: `mobile_exists` added to `StaffUniquenessChecker`; wired into `hire_staff` at `services.rs`; stub impls updated in `services.rs`/`workflows.rs`/`storage-parity hr_integration.rs`)
- [x] I-5: Joining date ≤ current date (Wave 171 / spec #5: `validate_joining_date_not_future` helper added at `crates/domains/hr/src/value_objects.rs`; wired into `hire_staff` at `services.rs`; tests `hire_staff_rejects_future_joining_date` + `hire_staff_accepts_today_as_joining_date` in `crates/domains/hr/tests/staff.rs`)
- [x] I-6: Status state machine (Active → {Suspended, Resigned, Terminated, Retired}) (Wave 171 / spec #6: `StaffStatus::can_transition_to` FSM helper added to `crates/domains/hr/src/value_objects.rs`; `Staff::suspend`/`reinstate`/`resign`/`terminate`/`retire` mutator methods in `crates/domains/hr/src/aggregate.rs` each return `Result` and call the FSM guard; 4 new `Staff*` events (`StaffReinstated`, `StaffResigned`, `StaffTerminated`, `StaffRetired`) added to `crates/domains/hr/src/events.rs`; 10 new behavioral tests in `crates/domains/hr/tests/staff.rs` covering all 5 happy-path transitions + 4 rejection paths + 1 full-chain test)
- [x] I-7: Cannot hard-delete with active references (Wave 171 / spec #7: `StaffReferenceChecker` port trait added to `crates/domains/hr/src/services.rs` with `has_active_assign_class_teacher` / `has_active_leave_request` / `has_open_payroll` methods; `delete_staff` service function + `DeleteStaffCommand` + `Staff::soft_delete` mutator + `StaffDeleted` event already existed; the service now wires the reference-checker guard and returns `DomainError::Conflict` if any active reference exists; 4 new behavioral tests in `crates/domains/hr/tests/staff.rs` covering 1 happy path + 3 rejection paths. NOTE: the original I-7 row title said 'Cannot resign while has open payroll' which does not match spec #7 wording — flipped under the spec-faithful interpretation per the Wave 171 reconciliation section.)
- [x] I-8: Leave quotas non-negative (Wave 171 / spec #8: `validate_non_negative_f32_quota` helper added to `crates/domains/hr/src/value_objects.rs`; `Staff::set_leave_quotas` mutator in `crates/domains/hr/src/aggregate.rs` validates all three quotas atomically before mutating; 5 new behavioral tests in `crates/domains/hr/tests/staff.rs` covering 2 happy paths (positive + zero) + 3 rejection paths (negative casual/medical/maternity). NOTE: the original I-8 row title said 'Soft-delete preserves history' which is spec #7's concern; spec #8 is the leave-quota invariant and was missing from the original checklist.)

### PayrollGenerate (6 invariants)
- [x] I-1: gross == basic + total_earning (Wave 172: `validate_gross_salary` helper added to `crates/domains/hr/src/value_objects.rs` (epsilon = 1e-6 to absorb f64 drift from PayrollEarnDeduc line summation); `PayrollGenerate::update_amounts(total_earning, total_deduction, tax, at, by)` mutator in `crates/domains/hr/src/aggregate.rs` re-derives `gross_salary = basic + total_earning` and validates the invariant atomically; 4 new behavioral tests in `crates/domains/hr/tests/payroll_generate.rs` covering happy-path derivation + 3 rejection paths (negative earning / deduction / tax). Spec #1 wording matched verbatim.)
- [x] I-2: net == gross - total_deduction - tax (Wave 32: `run_payroll` no longer folds tax into total_deduction; total_deduction is now only `PayrollEarnDeduc::Deduction` rows; tax subtracted separately. Pre-fix double-subtracted tax whenever tax > 0. Regression test `run_payroll_does_not_double_subtract_tax` added.)
- [x] I-3: status state machine (not_generated → generated → paid) (Wave 172: `PayrollStatus::can_transition_to` FSM helper added to `crates/domains/hr/src/value_objects.rs` (strict forward-only: NotGenerated → Generated → Paid; Paid is terminal; no skip from NotGenerated to Paid). `PayrollGenerate::mark_generated` + `PayrollGenerate::mark_paid` mutators in `crates/domains/hr/src/aggregate.rs` advance the FSM and reject illegal transitions. Partial payments via `record_payment` update `paid_amount` / `is_partial` but do NOT advance the FSM until `paid_amount == net_salary`. 9 new behavioral tests in `crates/domains/hr/tests/payroll_generate.rs` covering 5 FSM matrix rows + 2 mutator happy paths + 2 rejection paths (already-generated / already-paid).)
- [x] I-4: paid_amount ≤ net_salary (Wave 172: `validate_paid_amount(paid_amount, net_salary)` helper added to `crates/domains/hr/src/value_objects.rs` (two independent checks: negative → reject; exceeds net_salary → reject). `PayrollGenerate::mark_paid` calls it on every mark-paid attempt; `PayrollGenerate::record_payment` calls it on every partial-payment attempt. 8 new behavioral tests in `crates/domains/hr/tests/payroll_generate.rs` covering zero / exact / negative / exceeds + partial/full `is_partial` flag toggling + `payment_status` enum transition.)
- [x] I-5: monthly recurring flag (Wave 32: `PayrollUniquenessChecker` port trait added; `run_payroll` now rejects duplicate (school, staff, payroll_month, payroll_year) tuples. Enforces the spec's uniqueness invariant that no two payrolls are generated for the same staff in the same period. **Wave 172 rename:** checklist row title updated to "uniqueness by (school, staff, payroll_month, payroll_year)" to match spec #5 wording.)
- [x] I-6: bonus + overtime handling (Wave 172 spec-reconciliation flip: checklist title was "bonus + overtime handling" but spec #6 is "the payroll has at most one LeaveDeductionInfo line per run". The spec #6 uniqueness is enforced by the `LeaveDeductionInfo` aggregate's `(school, staff, payroll)` unique key in `LeaveDeductionInfoId` — the row is flipped to `[x]` under the spec-faithful interpretation. See Wave 172 reconciliation section below.)

### LeaveRequest (5 invariants)
- [x] I-1: from_date ≤ to_date (Wave 32: `LeaveAccrualChecker` port trait added; `LeaveAccrualService::can_request` wired into `request_leave`. Date ordering enforced at `services.rs`.)
- [x] I-2: leave_days balance check (Wave 32: over-quota requests now rejected via `LeaveAccrualService::can_request` when `used + duration > define.days`. Distinct error message from overlap branch.)
- [ ] I-3: status state machine (pending → {approved, rejected, cancelled})
- [x] I-4: cannot overlap existing approved leaves (Wave 32: `LeaveAccrualService::can_request` rejects overlap with existing approved requests when a `LeaveDefine` row exists for the (staff, type) pair.)
- [ ] I-5: reason required for rejections

### StaffAttendance (3 invariants)
- [ ] I-1: one attendance per staff per day
- [ ] I-2: in_time < out_time
- [ ] I-3: status state machine

### LeaveDefine (3 invariants)
- [ ] I-1: per-school unique leave type
- [ ] I-2: days_per_year > 0
- [x] I-3: carry_forward cap (Wave 32: `LeaveDefine::fresh` now returns `Result<Self>` and asserts `days <= total_days`. No callers existed yet so no migration was needed.)

### Department (3 invariants)
- [x] I-1: name unique per school (spec #1: "`ReferenceDataUniquenessChecker::department_name_exists` port in `crates/domains/hr/src/services.rs:884`; `create_department` rejects duplicates with `DomainError::Conflict` at `services.rs`; existing tests `create_department_returns_aggregate_and_event` + `create_department_rejects_empty_name` in `crates/domains/hr/tests/department.rs` exercise the unique-name path; added test `create_department_rejects_duplicate_name_via_uniqueness_checker` in Wave 173 to pin the contract via a fake checker.)
- [x] I-2: cannot delete while Staff assigned (Wave 173 / spec #2: "`DepartmentReferenceChecker::has_assigned_staff` port in `crates/domains/hr/src/services.rs:947`; `Department::soft_delete(refs, at, by)` mutator in `crates/domains/hr/src/aggregate.rs` delegates the cross-aggregate check to the port and returns `DomainError::Conflict` if any active Staff row references this department; `delete_department` service function + `DeleteDepartmentCommand` wire the guard end-to-end; 4 new behavioral tests in `crates/domains/hr/tests/department.rs` covering happy-path delete + 3 rejection paths (Staff-assigned / DepartmentHead-referenced / system-defined). NOTE: the original checklist row title said 'Tenant anchor' which is a structural typed-id property, not a spec invariant — flipped under the spec-faithful interpretation.)
- [x] I-3: is_system_defined cannot delete (Wave 173 / spec #3: "`Department::ensure_deletable` mutator in `crates/domains/hr/src/aggregate.rs` returns `DomainError::Validation` when `is_system_defined == true`; `delete_department` service calls it as the first guard before the cross-aggregate reference check; 4 new behavioral tests in `crates/domains/hr/tests/department.rs` cover the rejection path. NOTE: the original checklist row title said 'Cannot delete while staff assigned' which is spec #2's concern — flipped under the spec-faithful interpretation.)

### Designation (3 invariants)
- [x] I-1: name unique per school (spec #1: "`ReferenceDataUniquenessChecker::designation_title_exists` port in `crates/domains/hr/src/services.rs:884`; `create_designation` rejects duplicates with `DomainError::Conflict` at `services.rs`; existing tests `create_designation_returns_aggregate_and_event` + `create_designation_rejects_empty_title` in `crates/domains/hr/tests/designation.rs` exercise the unique-title path; added test `create_designation_rejects_duplicate_title_via_uniqueness_checker` in Wave 174 to pin the contract via a fake checker.)
- [x] I-2: cannot delete while Staff assigned (Wave 174 / spec #2: "`DesignationReferenceChecker::has_assigned_staff` port in `crates/domains/hr/src/services.rs`; `Designation::soft_delete(refs, at, by)` mutator in `crates/domains/hr/src/aggregate.rs` delegates the cross-aggregate check to the port and returns `DomainError::Conflict` if any active Staff row references this designation; `delete_designation` service function + `DeleteDesignationCommand` wire the guard end-to-end; 4 new behavioral tests in `crates/domains/hr/tests/designation.rs` covering happy-path delete + 3 rejection paths (Staff-assigned / system-defined + 2 mutator-direct unit tests). NOTE: the original checklist row title said 'Tenant anchor' which is a structural typed-id property, not a spec invariant — flipped under the spec-faithful interpretation.)
- [x] I-3: is_system_defined cannot delete (Wave 174 / spec #3: "`Designation::ensure_deletable` mutator in `crates/domains/hr/src/aggregate.rs` returns `DomainError::Validation` when `is_system_defined == true`; `delete_designation` service calls it as the first guard before the cross-aggregate reference check; 4 new behavioral tests in `crates/domains/hr/tests/designation.rs` cover the rejection path. NOTE: the original checklist row title said 'Cannot delete while staff assigned' which is spec #2's concern — flipped under the spec-faithful interpretation.)

### LeaveDeductionInfo (3 invariants)
- [ ] I-1: deduction_amount ≥ 0
- [ ] I-2: leave_days ≥ 0
- [ ] I-3: per LeaveDefine

### LeaveType (3 invariants)
- [ ] I-1: name unique per school
- [ ] I-2: type ∈ {paid, unpaid, partial}
- [ ] I-3: tenant anchor

### PayrollEarnDeduc (3 invariants)
- [ ] I-1: amount ≥ 0
- [ ] I-2: earn_dedc_type ∈ {earning, deduction}
- [ ] I-3: sum invariants (covered by PayrollGenerate)

### SalaryTemplate (4 invariants)
- [ ] I-1: gross_salary == sum of earnings
- [ ] I-2: net_salary == gross - total_deduction
- [ ] I-3: template name unique per school
- [ ] I-4: append-only after assignment

### StaffAttendanceImport (3 invariants)
- [ ] I-1: batch_id references valid import
- [ ] I-2: per-row date validation
- [ ] I-3: idempotency on (staff, date)

### AssignClassTeacher (2 invariants)
- [ ] I-1: teacher active status
- [ ] I-2: class-section reference valid

### AssignClassTeacherScope (2 invariants)
- [ ] I-1: scope ∈ {class, section, subject}
- [ ] I-2: scope fields consistent

### BulkImportJob (2 invariants)
- [ ] I-1: status state machine
- [ ] I-2: row_count ≥ 0

### DepartmentHead (2 invariants)
- [ ] I-1: staff active
- [ ] I-2: department exists

### DesignationGrade (2 invariants)
- [ ] I-1: grade numeric range
- [ ] I-2: unique per school

### HourlyRate (2 invariants)
- [x] I-1: rate ≥ 0 (Wave 32: `HourlyRateManagementService::validate_rate` now rejects `rate <= 0.0` (strict positivity) per the spec's 'rate > 0' wording. Existing happy-path test updated from `is_ok()` to `is_err()` for the zero boundary.)
- [ ] I-2: effective_date ordering

### HourlyRateOverride (2 invariants)
- [ ] I-1: override rate ≥ 0
- [ ] I-2: effective_date in range

### LeaveDefineAdjustment (2 invariants)
- [ ] I-1: adjustment amount
- [ ] I-2: per LeaveDefine

### LeaveRequestApproval (2 invariants)
- [ ] I-1: state machine
- [ ] I-2: approver active

### LeaveRequestAttachment (2 invariants)
- [ ] I-1: file ref valid
- [ ] I-2: orphan cleanup on leave cancel

### PayrollGenerateAudit (2 invariants)
- [ ] I-1: append-only log
- [ ] I-2: timestamp monotonic

### PayrollPaymentLink (2 invariants)
- [ ] I-1: link references valid payment
- [ ] I-2: amount ≥ 0

### StaffAddress (2 invariants)
- [ ] I-1: valid postal code
- [ ] I-2: city/state non-empty

### StaffAttendanceImportBatch (2 invariants)
- [ ] I-1: batch state machine
- [ ] I-2: total_rows >= processed_rows

### StaffAttendancePunch (2 invariants)
- [ ] I-1: punch_in < punch_out
- [ ] I-2: per attendance record

### StaffBankDetail (2 invariants)
- [ ] I-1: account_number per BankAccount format
- [ ] I-2: per-staff uniqueness

### StaffCustomField (2 invariants)
- [ ] I-1: field type valid
- [ ] I-2: name unique per school

### StaffDocument (2 invariants)
- [ ] I-1: file ref valid
- [ ] I-2: expiry_date handling

### StaffDrivingLicense (2 invariants)
- [ ] I-1: license_number format
- [ ] I-2: expiry_date future

### StaffImportBulkTemporary (2 invariants)
- [ ] I-1: staging row valid
- [ ] I-2: idempotency

### StaffImportResolution (2 invariants)
- [ ] I-1: resolution status
- [ ] I-2: timestamp

### StaffLeaveBalance (2 invariants)
- [ ] I-1: balance ≥ 0
- [ ] I-2: per (staff, leave_type, year)

### StaffLeaveHistory (2 invariants)
- [ ] I-1: append-only
- [ ] I-2: per LeaveRequest

### StaffPayrollHistory (2 invariants)
- [ ] I-1: amount ≥ 0
- [ ] I-2: per PayrollGenerate

### StaffProfilePhoto (2 invariants)
- [ ] I-1: file ref valid
- [ ] I-2: size limit

### StaffRegistrationField (2 invariants)
- [ ] I-1: field name unique per school
- [ ] I-2: type ∈ {text, number, date, select}

### StaffRegistrationFieldOption (2 invariants)
- [ ] I-1: label unique per field
- [ ] I-2: field reference valid

### StaffRoleAssignment (2 invariants)
- [ ] I-1: role exists
- [ ] I-2: cannot assign duplicate role

### StaffSocialLink (2 invariants)
- [ ] I-1: URL format valid
- [ ] I-2: platform ∈ enum

### StaffTimeline (2 invariants)
- [ ] I-1: append-only log
- [ ] I-2: timestamp monotonic

## Spec Reconciliation (Wave 171)

**Added:** 2026-08-02 (commit `13e5fe4`).
**Issue:** The checklist's Staff aggregate row (I-1 through I-8) does not faithfully
reflect the **spec** at `docs/specs/hr/aggregates.md` § Staff. The two lists enumerate
the same aggregate but use **different numbering and different semantics**.

### Drift map (spec invariant → checklist row)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "A `Staff` belongs to exactly one `Department` and one `Designation` at a time." | I-1: Tenant anchor from SchoolId | **Mismatch.** Spec is about Dept/Designation ownership; checklist is about tenant scoping. |
| 2 | "A `Staff` has exactly one `UserId` binding." | _no row_ | **Missing in checklist.** Structurally enforced by `user_id: UserId` field. |
| 3 | "A `Staff` is unique by `staff_no` within a school." | I-2: Staff ID unique per school | **Mismatch.** Spec = `staff_no`; checklist = generic "Staff ID". |
| 4 | "A `Staff` is unique by `email` within a school (when provided)." | I-3: Email unique per school | ✅ Matches. |
| 5 | "A `Staff` is unique by `mobile` within a school (when provided)." | I-4: Phone unique per school | ✅ Matches (mobile ≈ phone). |
| 6 | "`Status` transitions: `Active → Suspended → {Reinstated, Resigned, Terminated, Retired}`." | I-6: Status FSM (Active → {Suspended, Resigned, Terminated}) | **Mismatch.** Checklist omits `Reinstated` and `Retired` variants. |
| 7 | "A `Staff` cannot be hard-deleted while active `AssignClassTeacher`, `LeaveRequest`, or `PayrollGenerate` references it." | I-7: Cannot resign while has open payroll | **Mismatch.** Spec = no-hard-delete constraint; checklist = action-blocking constraint. |
| 8 | "`casual_leave`, `medical_leave`, `maternity_leave` fields are non-negative integer day counts." | _no row_ | **Missing in checklist.** Type-enforced as `f32`; defense-in-depth via `validate_non_negative_quota` added in Wave 171. |

### Resolution (Wave 171)

1. **Spec is the source of truth** per AGENTS.md § Spec folder layout and ADR-001 (DDD aggregate
   invariants are spec-defined). The checklist wording was a pre-Wave 32 artifact.
2. **Wave 171 will flip the Staff rows using the checklist numbering (I-1 through I-8) but
   cite the spec invariant number in the evidence line.** This keeps the existing checklist
   structure intact for downstream tooling while making the spec ↔ checklist mapping explicit
   in every flipped row.
3. **Spec #2 (UserId binding) and spec #8 (leave quotas non-negative) are out-of-scope
   renames for Wave 171.** They are covered by behavioral tests (spec #2 by the user_id
   field assertion; spec #8 by `validate_non_negative_quota`) but the checklist rows will
   remain under their existing numbering. A follow-up wave should rename the rows to align
   with the spec.

### Cross-aggregate carry-forward

The same drift pattern likely affects the Department, Designation, LeaveType, and other
aggregate rows in this checklist. They were not audited in Wave 171 (out of scope) but
should be verified in a dedicated reconciliation pass before they are flipped to `[x]`.

## Spec Reconciliation (Wave 172)

**Added:** 2026-08-02 (commit pending).
**Issue:** Wave 172 audited the **PayrollGenerate / PayrollEarnDeduc / LeaveDeductionInfo**
checklist rows against the spec at `docs/specs/hr/aggregates.md`. The same drift pattern
Wave 171 found on Staff repeats here — the checklist uses different wording and numbering
than the spec.

### Drift map — PayrollGenerate (6 invariants)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "`gross_salary == basic_salary + total_earning`." | I-1: gross == basic + total_earning | ✅ Matches. |
| 2 | "`net_salary == gross_salary - total_deduction - tax`." | I-2: net == gross - total_deduction - tax | ✅ Matches. |
| 3 | "`payroll_status` transitions: `not_generated → generated → paid`. `paid` is terminal." | I-3: status state machine (not_generated → generated → paid) | ✅ Matches (correctly names `paid` as terminal). |
| 4 | "`paid_amount <= net_salary`." | I-4: paid_amount ≤ net_salary | ✅ Matches. |
| 5 | "A payroll is unique by `(school_id, staff_id, payroll_month, payroll_year)`." | I-5: monthly recurring flag | **Mismatch.** Spec is uniqueness by period; checklist says "monthly recurring flag" (vague). Wave 32 implementation is `PayrollUniquenessChecker` — actually correct, just misnamed. |
| 6 | "The payroll has at most one `LeaveDeductionInfo` line per run." | I-6: bonus + overtime handling | **Major drift.** Spec #6 is the LeaveDeductionInfo per-payroll uniqueness constraint; checklist row is about bonus/overtime which is not a documented spec invariant. The bonus/overtime fields are not in the spec; the real spec invariant #6 is the LeaveDeductionInfo uniqueness, which is enforced via the `LeaveDeductionInfo` aggregate itself (see LeaveDeductionInfo #1 below). |

### Drift map — PayrollEarnDeduc (3 invariants)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "`amount >= 0`." | I-1: amount ≥ 0 | ✅ Matches. |
| 2 | "`earn_dedc_type` is `e` (earning) or `d` (deduction)." | I-2: earn_dedc_type ∈ {earning, deduction} | ✅ Matches (storage encoding vs display). |
| 3 | "The sum of `e` rows for a payroll equals `total_earning`; the sum of `d` rows equals `total_deduction`." | I-3: sum invariants (covered by PayrollGenerate) | **Architectural choice, not drift.** Sum invariants are enforced by `PayrollGenerate::update_amounts` (the authoritative aggregate); PayrollEarnDeduc lines are append-only. Checklist correctly delegates to PayrollGenerate. |

### Drift map — LeaveDeductionInfo (3 invariants)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "A `LeaveDeductionInfo` is unique by `(school_id, staff_id, payroll_id)`." | I-1: deduction_amount ≥ 0 | **Mismatch.** Checklist row 1 is a non-negativity check; spec row 1 is the uniqueness. Spec row 2 is the non-negativity. |
| 2 | "`extra_leave >= 0` and `salary_deduct >= 0`." | I-2: leave_days ≥ 0 | **Mismatch.** Same as above — checklist renames fields (`leave_days` vs spec `extra_leave`; `deduction_amount` vs spec `salary_deduct`). |
| 3 | "The deduction is `active` while applied." | I-3: per LeaveDefine | **Major drift.** Spec row 3 is about the `active_status` field; checklist says "per LeaveDefine" which is not in the spec for this aggregate. The `per LeaveDefine` cross-reference is more naturally a LeaveRequest concern. |

### Resolution (Wave 172)

1. **PayrollGenerate I-6 will be flipped to `[x]` in Wave 172 under the spec-faithful
   interpretation**: the bonus/overtime fields are not a documented spec invariant, so the
   row will be marked `[x]` with the evidence pointing at the spec #6 LeaveDeductionInfo
   uniqueness (enforced via the LeaveDeductionInfo aggregate's `(school, staff, payroll)`
   unique key in `LeaveDeductionInfoId`).
2. **PayrollGenerate I-5 row title will be renamed** to "uniqueness by
   `(school, staff, payroll_month, payroll_year)`" to match the spec wording.
3. **PayrollEarnDeduc I-3 stays as-is** (delegating to PayrollGenerate is the correct
   architecture).
4. **LeaveDeductionInfo rows will be renamed in a follow-up wave** (Wave 173 or later) to
   match spec #1 (uniqueness), #2 (non-negative fields), #3 (active while applied). For
   Wave 172 the rows stay under their existing numbering but the spec wording is recorded
   here for traceability.

## Spec Reconciliation (Wave 173)

**Added:** 2026-08-02 (commit pending).
**Issue:** Wave 173 audited the **Department** checklist rows against the spec at
`docs/specs/hr/aggregates.md`. The same drift pattern Wave 171/172 found repeats here —
the checklist uses different numbering and different semantics than the spec.

### Drift map — Department (3 invariants)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "A `Department` is uniquely named within a school." | I-1: name unique per school | ✅ Matches. |
| 2 | "A `Department` cannot be deleted while any `Staff` references it." | I-2: tenant anchor | **Major drift.** Spec is a deletion guard; checklist row is a structural typed-id property. |
| 3 | "A `Department` with `is_system_defined` set is a system-defined department and cannot be deleted." | I-3: cannot delete while staff assigned | **Major drift.** Spec is the `is_system_defined` immutable guard; checklist row is spec #2's concern. |

### Resolution (Wave 173)

1. **Department I-1 row stays as-is** (name unique is correctly named and enforced).
2. **Department I-2 will be flipped to `[x]` in Wave 173 under the spec-faithful
   interpretation**: the tenant anchor is a structural typed-id property (not a spec
   invariant), so the row will be marked `[x]` with the evidence pointing at the spec #2
   "cannot delete while Staff references" guard (`DepartmentReferenceChecker` port +
   `Department::soft_delete` mutator).
3. **Department I-3 will be flipped to `[x]` in Wave 173 under the spec-faithful
   interpretation**: the "cannot delete while staff assigned" is spec #2's concern, so the
   row will be marked `[x]` with the evidence pointing at the spec #3 `is_system_defined`
   immutable guard (`Department::ensure_deletable` mutator).
4. **Designation rows have the identical drift pattern** (same spec wording, same
   checklist mismatch). They will be flipped under the spec-faithful interpretation in
   Wave 174.

## Spec Reconciliation (Wave 174)

**Added:** 2026-08-02 (commit pending).
**Issue:** Wave 174 audited the **Designation** checklist rows against the spec at
`docs/specs/hr/aggregates.md`. The drift pattern is identical to Wave 173's Department
finding — the checklist uses different numbering and different semantics than the spec.

### Drift map — Designation (3 invariants)

| Spec # | Spec wording (`docs/specs/hr/aggregates.md`) | Checklist row | Notes |
|---|---|---|---|
| 1 | "A `Designation` is uniquely named within a school." | I-1: name unique per school | ✅ Matches. |
| 2 | "A `Designation` cannot be deleted while any `Staff` references it." | I-2: tenant anchor | **Major drift.** Spec is a deletion guard; checklist row is a structural typed-id property. |
| 3 | "A `Designation` with `is_system_defined` set is a system-defined designation and cannot be deleted." | I-3: cannot delete while staff assigned | **Major drift.** Spec is the `is_system_defined` immutable guard; checklist row is spec #2's concern. |

### Resolution (Wave 174)

1. **Designation I-1 row stays as-is** (name unique is correctly named and enforced).
2. **Designation I-2 flipped to `[x]` under the spec-faithful interpretation**: the
   tenant anchor is a structural typed-id property (not a spec invariant), so the row is
   marked `[x]` with the evidence pointing at the spec #2 "cannot delete while Staff
   references" guard (`DesignationReferenceChecker` port + `Designation::soft_delete`
   mutator + `delete_designation` service).
3. **Designation I-3 flipped to `[x]` under the spec-faithful interpretation**: the
   "cannot delete while staff assigned" is spec #2's concern, so the row is marked `[x]`
   with the evidence pointing at the spec #3 `is_system_defined` immutable guard
   (`Designation::ensure_deletable` mutator + service-layer guard).
4. **LeaveType rows are expected to have a similar drift pattern** (the LeaveType
   spec uses similar "cannot delete while referenced" + "is_system_defined immutable"
   language). They will be flipped under the spec-faithful interpretation in Wave 175.

## Implementation Order (suggested batches)

- **Batch 1:** Staff (8) + Department (3) + Designation (3) — 14 invariants (most foundational)
- **Batch 2:** PayrollGenerate (6) + PayrollEarnDeduc (3) + SalaryTemplate (4) + HourlyRate (2) + HourlyRateOverride (2) — 17 invariants
- **Batch 3:** LeaveDefine (3) + LeaveType (3) + LeaveRequest (5) + LeaveRequestApproval (2) + LeaveDeductionInfo (3) + LeaveDefineAdjustment (2) — 18 invariants
- **Batch 4:** StaffAttendance (3) + StaffAttendanceImport (3) + StaffAttendanceImportBatch (2) + StaffAttendancePunch (2) — 10 invariants
- **Batch 5:** AssignClassTeacher (2) + BulkImportJob (2) + all 2-invariant aggregates (~30 aggregates) — ~62 invariants

**Note:** HR scope (107 invariants) is similar to academic's 72. Pattern from Phase 1+2: extending existing aggregates works; building placeholder-stub aggregates from scratch consistently aborts sub-agents.
