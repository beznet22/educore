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
- [x]: TBD (Phase 1 baseline; will refine after deep audit)
- [~]: TBD
- [ ]: TBD
- [N/A]: 0

## Per-aggregate Status (compact)

### Staff (8 invariants — highest count)
- [ ] I-1: Tenant anchor from SchoolId
- [ ] I-2: Staff ID unique per school
- [ ] I-3: Email unique per school
- [ ] I-4: Phone unique per school
- [ ] I-5: Joining date ≤ current date
- [ ] I-6: Status state machine (Active → {Suspended, Resigned, Terminated})
- [ ] I-7: Cannot resign while has open payroll
- [ ] I-8: Soft-delete preserves history

### PayrollGenerate (6 invariants)
- [ ] I-1: gross == basic + total_earning
- [ ] I-2: net == gross - total_deduction - tax
- [ ] I-3: status state machine (not_generated → generated → paid)
- [ ] I-4: paid_amount ≤ net_salary
- [ ] I-5: monthly recurring flag
- [ ] I-6: bonus + overtime handling

### LeaveRequest (5 invariants)
- [ ] I-1: from_date ≤ to_date
- [ ] I-2: leave_days balance check
- [ ] I-3: status state machine (pending → {approved, rejected, cancelled})
- [ ] I-4: cannot overlap existing approved leaves
- [ ] I-5: reason required for rejections

### StaffAttendance (3 invariants)
- [ ] I-1: one attendance per staff per day
- [ ] I-2: in_time < out_time
- [ ] I-3: status state machine

### LeaveDefine (3 invariants)
- [ ] I-1: per-school unique leave type
- [ ] I-2: days_per_year > 0
- [ ] I-3: carry_forward cap

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
- [ ] I-1: rate ≥ 0
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

## Implementation Order (suggested batches)

- **Batch 1:** Staff (8) + Department (3) + Designation (3) — 14 invariants (most foundational)
- **Batch 2:** PayrollGenerate (6) + PayrollEarnDeduc (3) + SalaryTemplate (4) + HourlyRate (2) + HourlyRateOverride (2) — 17 invariants
- **Batch 3:** LeaveDefine (3) + LeaveType (3) + LeaveRequest (5) + LeaveRequestApproval (2) + LeaveDeductionInfo (3) + LeaveDefineAdjustment (2) — 18 invariants
- **Batch 4:** StaffAttendance (3) + StaffAttendanceImport (3) + StaffAttendanceImportBatch (2) + StaffAttendancePunch (2) — 10 invariants
- **Batch 5:** AssignClassTeacher (2) + BulkImportJob (2) + all 2-invariant aggregates (~30 aggregates) — ~62 invariants

**Note:** HR scope (107 invariants) is similar to academic's 72. Pattern from Phase 1+2: extending existing aggregates works; building placeholder-stub aggregates from scratch consistently aborts sub-agents.
