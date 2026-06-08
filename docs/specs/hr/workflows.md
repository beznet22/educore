# HR Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Staff Onboarding

```text
1. HR pre-configures the catalog:
   a. CreateDepartment (Mathematics, Administration, etc.)
   b. CreateDesignation (Math Teacher, Accountant, Driver, etc.)
   c. CreateLeaveType (Casual, Sick, Maternity).
   d. CreateLeavePolicy (per role or per user).
   e. CreateSalaryTemplate (per grade).
   f. SetHourlyRate (per grade).
   g. CreateStaffRegistrationField (per school customization).
2. HR triggers RegisterStaff with the staff's full profile, the
   resolved department_id, designation_id, role_id, and the
   platform user_id.
3. The platform creates the platform user (via the auth port) and
   binds the role (via the RBAC port).
4. Finance receives StaffRegistered and binds the salary template
   or hourly rate.
5. Communication receives StaffRegistered and sends a welcome
   message to the staff.
6. The staff receives an onboarding email with login credentials
   and a link to the staff portal.
```

**Pre-conditions:**
- The department, designation, and leave types exist.
- The salary template or hourly rate is configured for the staff's
  grade.

**Failure paths:**
- Duplicate staff_no / email / mobile → `ValidationError::UniqueViolation`.
- Missing department / designation / role → `NotFoundError`.
- Platform user creation failure → `InfrastructureError`.

## Staff Lifecycle

```text
1. Staff is Active.
2. HR may SuspendStaff (e.g. disciplinary, medical).
   └─ Attendance continues to be marked but payroll reflects
     suspension policy.
3. HR may ReinstateStaff; the staff returns to Active.
4. Staff may Resign, or HR may Terminate or Retire the staff.
5. When the staff leaves, HR triggers DeleteStaff (soft-delete)
   which preserves the history.
6. The platform user is deactivated; the RBAC role is revoked.
7. The class teacher and subject teacher assignments are released.
```

## Class Teacher Assignment

```text
1. SchoolAdmin (or HR) triggers AssignClassTeacher with a class,
   section, staff, and academic year.
2. The system creates an AssignClassTeacher row.
3. The academic domain may subscribe to mirror the class-section
   teacher binding.
4. SchoolAdmin may update the assignment to a different staff
   during the academic year.
5. At year-end, the assignment is closed; the next year requires a
   fresh assignment.
```

## Subject Teacher Assignment

```text
1. HR triggers AssignSubjectTeacher with a class, optional
   section, subject, staff, and academic year.
2. The system emits SubjectTeacherAssigned; the academic domain
   subscribes and creates the corresponding class-subject row.
3. The assignment is valid for the academic year and may be
   replaced mid-year.
```

## Leave Request Lifecycle

```text
1. Staff triggers RequestLeave with the type, dates, reason, and
   optional attachment.
2. The system validates that:
   a. leave_from <= leave_to.
   b. The staff has a LeaveDefine for the type with remaining
      days covering the period.
3. The system creates a LeaveRequest in `pending` state.
4. Approver (HR or department head) reviews the request and
   triggers ApproveLeave or RejectLeave.
5. On approval, the leave balance is decremented. The system emits
   LeaveApproved; finance receives it for payroll planning.
6. Staff may CancelLeave before the leave starts or within a
   configurable grace window.
7. On cancellation, the leave balance is restored.
```

**Edge cases:**
- A request that spans two `LeaveDefine` periods (e.g. cross-year)
  is rejected unless the consumer's policy allows it.
- A request whose dates overlap an existing approved request is
  rejected.
- A request whose dates fall on a holiday is shortened to working
  days by the policy `LeaveWorkingDays`.

## Staff Attendance Workflow

```text
1. Daily job (or manual input) triggers MarkStaffAttendance per
   staff member with type P, L, A, H, or F.
2. The system stores the row with the staff and the date.
3. Late attendance (L) and half-day (F) feed the payroll's
   earning or deduction lines.
4. Absent (A) is preserved for leave-policy cross-checks.
5. Bulk imports follow the same flow:
   a. ImportStaffAttendance creates StaffAttendanceImport rows.
   b. PromoteStaffAttendance converts them to StaffAttendance.
   c. RejectStaffAttendance marks the rows as rejected.
```

## Hourly Rate Management

```text
1. HR configures a per-grade hourly rate via SetHourlyRate.
2. The rate is read by the payroll service when computing
   earnings for hourly-based staff.
3. The rate may be updated mid-year; the change applies to
   subsequent payrolls only.
4. Per-staff overrides are applied via HourlyRateOverride (read by
   the payroll service).
```

## Salary Template

```text
1. HR defines a SalaryTemplate with the grade, basic, house rent,
   provident fund, gross, total deduction, and net.
2. The system validates that gross equals basic + house rent +
   provident fund and net equals gross - total deduction.
3. The template is bound to a staff by HR (typically via the
   staff's designation or directly on the staff profile).
4. When payroll is generated, the template pre-fills the
   PayrollGenerate; the payroll service may override per-staff.
```

## Payroll Generation

```text
1. HR (or scheduled job) triggers GeneratePayroll for one or more
   staff members in a given pay period.
2. The system reads the staff's salary template or hourly rate
   and computes the base amounts.
3. The system adds the approved leave deductions
   (AddLeaveDeductionInfo) and any hourly earnings.
4. The system emits PayrollGenerated with the full set of
   PayrollEarningAdded and PayrollDeductionAdded events.
5. The payroll register is reviewed by HR and the school admin.
```

## Payroll Approval

```text
1. SchoolAdmin reviews the payroll register.
2. SchoolAdmin triggers ApprovePayroll per row.
3. The system emits PayrollApproved.
4. The system locks the earnings and deductions; further changes
   require a reversal (delete and re-create).
5. Segregation of duties: the approver is not the generator.
```

## Payroll Disbursement (Cross-Domain)

```text
1. Finance (the source of truth for money movement) triggers
   RecordPayrollPayment in the finance domain.
2. The finance domain emits PayrollPaymentRecorded and a
   PayrollPaid event (when fully paid).
3. The HR domain subscribes to PayrollPaid and triggers
   MarkPayrollPaid on the PayrollGenerate.
4. The HR domain emits PayrollPaid (a separate event with the HR
   aggregate id).
5. Communication sends a payslip notification to the staff.
6. The HR domain updates the staff's payroll history.
```

## Leave Deduction on Payroll

```text
1. When the leave approver approves a request whose dates fall
   outside the staff's LeaveDefine (extra leave), the system
   records an extra-leave note.
2. During payroll generation, the system reads the approved
   extra-leave totals and creates an AddLeaveDeductionInfo line.
3. The deduction is folded into the payroll's total_deduction and
   net_salary.
4. The system emits LeaveDeductionInfoAdded.
```

## Bulk Staff Import

```text
1. HR uploads a CSV via ImportStaffBulk.
2. The system creates a BulkImportJob and one
   StaffImportBulkTemporary per row.
3. HR reviews the staging rows; missing departments, designations,
   or roles are resolved.
4. HR triggers PromoteStaffImport per row with the resolved
   references.
5. The system creates a Staff per row and emits
   StaffImportPromoted plus the underlying StaffRegistered.
6. Rows that fail validation are rejected via RejectStaffImport.
```

## Idempotency

- `RegisterStaff` is idempotent on `(school_id, email)` and
  `(school_id, staff_no)`. A duplicate is a no-op success.
- `MarkStaffAttendance` is idempotent on
  `(staff_id, attendance_date)`. A duplicate is a no-op success.
- `RequestLeave` is idempotent on
  `(staff_id, leave_from, leave_to, type_id)`.
- `ApproveLeave` / `RejectLeave` are idempotent on
  `leave_request_id`. Re-issuing a decision on the same status is
  a no-op.
- `GeneratePayroll` is idempotent on
  `(staff_id, pay_period)`. A duplicate returns the existing
  payroll.
- `SetHourlyRate` is idempotent on `(school_id, grade,
  academic_id)`. A duplicate updates the rate.
- `AssignClassTeacher` is idempotent on `(class_id, section_id,
  academic_id, school_id)`.
- `ImportStaffBulk` is idempotent on
  `(school_id, source, file_hash)`.

## Cross-Workflow Order

The HR domain observes the following order:

1. `StaffRegistered` (HR) → `SalaryTemplate` / `HourlyRate` is
   bound (finance), role is assigned (RBAC).
2. `LeaveRequested` → `LeaveApproved` or `LeaveRejected` is the
   terminal transition.
3. `PayrollGenerated` → `PayrollApproved` → `PayrollPaid` is the
   terminal transition.
4. `PayrollPaid` is the only event that allows the finance domain
   to mark the payroll as fully paid; partial payments are
   recorded but the HR-side status remains `generated` until the
   cumulative equals the net salary.

The HR domain never calls the finance domain directly. It only
subscribes to finance events (notably `PayrollPaymentRecorded` and
`PayrollPaid`) and reacts through its own commands.
