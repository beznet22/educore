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
- [x]: 7 (Wave 32: Staff I-4, PayrollGenerate I-2/I-5, LeaveRequest I-1/I-2/I-4, LeaveDefine I-3, HourlyRate I-1)
- [~]: 0
- [ ]: 100 (remaining; targeted by the next per-aggregate wave pipeline)
- [N/A]: 0

**Summary updated at session end (commit `d2a9e45`).** The previous `TBD/TBD/TBD` tally
was a pre-Wave 32 baseline that was never refreshed. Wave 32 (`3376a4b`) added 7
invariant enforcements: 1 Staff (phone unique per school via `StaffUniquenessChecker`),
2 PayrollGenerate (net == gross - total_deduction - tax + monthly recurring uniqueness via
`PayrollUniquenessChecker`), 3 LeaveRequest (date ordering + balance check + no-overlap via
`LeaveAccrualChecker`), 1 LeaveDefine (carry_forward cap via `LeaveDefine::fresh` now
returning `Result<Self>`), 1 HourlyRate (rate >= 0 via `HourlyRateManagementService::validate_rate`).
The 100 remaining `[ ]` invariants are the next per-aggregate wave pipeline's backlog.

## Per-aggregate Status (compact)

### Staff (8 invariants — highest count)
- [ ] I-1: Tenant anchor from SchoolId
- [ ] I-2: Staff ID unique per school
- [ ] I-3: Email unique per school
- [x] I-4: Phone unique per school (Wave 32: `mobile_exists` added to `StaffUniquenessChecker`; wired into `hire_staff` at `services.rs`; stub impls updated in `services.rs`/`workflows.rs`/`storage-parity hr_integration.rs`)
- [ ] I-5: Joining date ≤ current date
- [ ] I-6: Status state machine (Active → {Suspended, Resigned, Terminated})
- [ ] I-7: Cannot resign while has open payroll
- [ ] I-8: Soft-delete preserves history

### PayrollGenerate (6 invariants)
- [ ] I-1: gross == basic + total_earning
- [x] I-2: net == gross - total_deduction - tax (Wave 32: `run_payroll` no longer folds tax into total_deduction; total_deduction is now only `PayrollEarnDeduc::Deduction` rows; tax subtracted separately. Pre-fix double-subtracted tax whenever tax > 0. Regression test `run_payroll_does_not_double_subtract_tax` added.)
- [ ] I-3: status state machine (not_generated → generated → paid)
- [ ] I-4: paid_amount ≤ net_salary
- [x] I-5: monthly recurring flag (Wave 32: `PayrollUniquenessChecker` port trait added; `run_payroll` now rejects duplicate (school, staff, payroll_month, payroll_year) tuples. Enforces the spec's uniqueness invariant that no two payrolls are generated for the same staff in the same period.)
- [ ] I-6: bonus + overtime handling

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
- [ ] I-1: name unique per school
- [ ] I-2: tenant anchor
- [ ] I-3: cannot delete while staff assigned

### Designation (3 invariants)
- [ ] I-1: name unique per school
- [ ] I-2: tenant anchor
- [ ] I-3: cannot delete while staff assigned

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

## Implementation Order (suggested batches)

- **Batch 1:** Staff (8) + Department (3) + Designation (3) — 14 invariants (most foundational)
- **Batch 2:** PayrollGenerate (6) + PayrollEarnDeduc (3) + SalaryTemplate (4) + HourlyRate (2) + HourlyRateOverride (2) — 17 invariants
- **Batch 3:** LeaveDefine (3) + LeaveType (3) + LeaveRequest (5) + LeaveRequestApproval (2) + LeaveDeductionInfo (3) + LeaveDefineAdjustment (2) — 18 invariants
- **Batch 4:** StaffAttendance (3) + StaffAttendanceImport (3) + StaffAttendanceImportBatch (2) + StaffAttendancePunch (2) — 10 invariants
- **Batch 5:** AssignClassTeacher (2) + BulkImportJob (2) + all 2-invariant aggregates (~30 aggregates) — ~62 invariants

**Note:** HR scope (107 invariants) is similar to academic's 72. Pattern from Phase 1+2: extending existing aggregates works; building placeholder-stub aggregates from scratch consistently aborts sub-agents.
