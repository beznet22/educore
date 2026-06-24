# HR Domain — Aggregates

## Staff

**Root type:** `Staff`
**Identity:** `StaffId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** HR

### Purpose

A person employed by the school, including their identity, profile,
employment data, leave balances, salary structure, and links to the
departments and designations that organize their work.

### Owned Children

- `LeaveRequest` (zero or more, owned through `Staff`).
- `StaffAttendance` (zero or more).
- `StaffAttendanceImport` (zero or more).
- `PayrollGenerate` (zero or more, per month).
- `AssignClassTeacher` (zero or more).
- `LeaveDeductionInfo` (zero or more, per payroll).

### Invariants

1. A `Staff` belongs to exactly one `Department` and one
   `Designation` at a time.
2. A `Staff` has exactly one `UserId` binding.
3. A `Staff` is unique by `staff_no` within a school.
4. A `Staff` is unique by `email` within a school (when provided).
5. A `Staff` is unique by `mobile` within a school (when provided).
6. A `Staff`'s `Status` transitions are: `Active → Suspended →
   {Reinstated, Resigned, Terminated, Retired}`. `Resigned`,
   `Terminated`, and `Retired` are terminal.
7. A `Staff` cannot be hard-deleted while active
   `AssignClassTeacher`, `LeaveRequest`, or `PayrollGenerate`
   references it.
8. The `casual_leave`, `medical_leave`, and `maternity_leave` fields
   are non-negative integer day counts.

### Commands

- `RegisterStaff`
- `UpdateStaff`
- `ChangeStaffDepartment`
- `ChangeStaffDesignation`
- `ChangeStaffRole`
- `SuspendStaff`
- `ReinstateStaff`
- `ResignStaff`
- `TerminateStaff`
- `RetireStaff`
- `DeleteStaff`

### Events

- `StaffRegistered`
- `StaffUpdated`
- `StaffDepartmentChanged`
- `StaffDesignationChanged`
- `StaffRoleChanged`
- `StaffSuspended`
- `StaffReinstated`
- `StaffResigned`
- `StaffTerminated`
- `StaffRetired`
- `StaffDeleted`

### Consistency Boundary

All staff mutations are serialized through the `Staff` aggregate
root. A staff is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction.

---

## Department

**Root type:** `Department`
**Identity:** `DepartmentId(SchoolId, Uuid)`

### Purpose

A logical grouping of staff (e.g. "Mathematics", "Administration",
"Cleaning"). Departments carry no per-year scope.

### Invariants

1. A `Department` is uniquely named within a school.
2. A `Department` cannot be deleted while any `Staff` references it.
3. A `Department` with `is_system_defined` set is a system-defined
   department and cannot be deleted.

### Commands

- `CreateDepartment`
- `UpdateDepartment`
- `DeleteDepartment`

### Events

- `DepartmentCreated`
- `DepartmentUpdated`
- `DepartmentDeleted`

---

## Designation

**Root type:** `Designation`
**Identity:** `DesignationId(SchoolId, Uuid)`

### Purpose

A job title (e.g. "Principal", "Math Teacher", "Accountant",
"Driver"). Designations carry no per-year scope.

### Invariants

1. A `Designation` is uniquely named within a school.
2. A `Designation` cannot be deleted while any `Staff` references it.
3. A `Designation` with `is_system_defined` set is a system-defined
   designation and cannot be deleted.

### Commands

- `CreateDesignation`
- `UpdateDesignation`
- `DeleteDesignation`

### Events

- `DesignationCreated`
- `DesignationUpdated`
- `DesignationDeleted`

---

## LeaveType

**Root type:** `LeaveType`
**Identity:** `LeaveTypeId(SchoolId, Uuid)`

### Purpose

A category of leave (e.g. "Casual Leave", "Sick Leave",
"Maternity Leave"). Each leave type has a per-year total.

### Invariants

1. A `LeaveType` is uniquely named within a school.
2. A `LeaveType` cannot be deleted while any `LeaveDefine` or
   `LeaveRequest` references it.
3. `total_days >= 0`.

### Commands

- `CreateLeaveType`
- `UpdateLeaveType`
- `DeleteLeaveType`

### Events

- `LeaveTypeCreated`
- `LeaveTypeUpdated`
- `LeaveTypeDeleted`

---

## LeaveDefine

**Root type:** `LeaveDefine`
**Identity:** `LeaveDefineId(SchoolId, Uuid)`

### Purpose

The leave entitlement per role or per user for a given leave type
in a given academic year. Carries `days` and `total_days`.

### Invariants

1. A `LeaveDefine` is unique by `(school_id, academic_id, role_id,
   type_id)` or by `(school_id, academic_id, user_id, type_id)`.
2. `days >= 0` and `total_days >= 0`.
3. `days <= total_days` (a user cannot take more than the
   entitlement for the year).

### Commands

- `DefineLeavePolicy`
- `UpdateLeavePolicy`
- `DeleteLeavePolicy`

### Events

- `LeavePolicyDefined`
- `LeavePolicyUpdated`
- `LeavePolicyDeleted`

---

## LeaveRequest

**Root type:** `LeaveRequest`
**Identity:** `LeaveRequestId(SchoolId, Uuid)`

### Purpose

A staff request for leave. Carries `apply_date`, `leave_from`,
`leave_to`, `reason`, `note`, optional `file`, and an
`approve_status` of `pending`, `approved`, or `rejected`.

### Invariants

1. A `LeaveRequest` is unique by `(school_id, staff_id, leave_from,
   leave_to, type_id)` per academic year.
2. `leave_from <= leave_to`.
3. `approve_status` is `pending` on creation; it transitions to
   `approved` or `rejected` and never returns to `pending`.
4. Approval requires the staff's `LeaveDefine` for the same type
   to have remaining days for the period.
5. The number of days in the request must not exceed the
   `LeaveDefine.total_days`.

### Commands

- `RequestLeave`
- `ApproveLeave`
- `RejectLeave`
- `CancelLeave`

### Events

- `LeaveRequested`
- `LeaveApproved`
- `LeaveRejected`
- `LeaveCancelled`

---

## StaffAttendance

**Root type:** `StaffAttendance`
**Identity:** `StaffAttendanceId(SchoolId, Uuid)`

### Purpose

A daily attendance row for a staff member. Carries
`attendance_type` (`P`, `L`, `A`, `H`, `F`), `notes`, and
`attendance_date`.

### Invariants

1. A `StaffAttendance` is unique by `(school_id, staff_id,
   attendance_date, academic_id)`.
2. `attendance_type` is one of `P` (Present), `L` (Late), `A`
   (Absent), `H` (Holiday), `F` (Half Day).
3. `attendance_date` is required.

### Commands

- `MarkStaffAttendance`
- `UpdateStaffAttendance`
- `DeleteStaffAttendance`

### Events

- `StaffAttendanceMarked`
- `StaffAttendanceUpdated`
- `StaffAttendanceDeleted`

---

## StaffAttendanceImport

**Root type:** `StaffAttendanceImport`
**Identity:** `StaffAttendanceImportId(SchoolId, Uuid)`

### Purpose

A staging row from a bulk attendance import. Carries
`attendance_date`, `in_time`, `out_time`, `attendance_type`, and
`notes`. Imports are promoted into `StaffAttendance` on success.

### Invariants

1. A `StaffAttendanceImport` is unique by `(school_id, staff_id,
   attendance_date, academic_id)`.
2. `in_time` and `out_time` are stored as `String` to accommodate
   arbitrary source formats; promotion validates them.
3. The import is marked as `active` while pending promotion.

### Commands

- `ImportStaffAttendance`
- `PromoteStaffAttendance`
- `RejectStaffAttendance`

### Events

- `StaffAttendanceImported`
- `StaffAttendancePromoted`
- `StaffAttendanceImportRejected`

---

## AssignClassTeacher

**Root type:** `AssignClassTeacher`
**Identity:** `AssignClassTeacherId(SchoolId, Uuid)`

### Purpose

A higher-level "class teacher" assignment that may span classes and
sections in an academic year. The actual class-section teacher
binding is owned by the academic domain; this aggregate tracks the
school-level class teacher roster.

### Invariants

1. An `AssignClassTeacher` is unique by `(school_id, class_id,
   section_id, academic_id)`.
2. `active_status` is `1` while the assignment is open.

### Commands

- `AssignClassTeacher`
- `UpdateAssignClassTeacher`
- `DeleteAssignClassTeacher`

### Events

- `ClassTeacherAssigned`
- `AssignClassTeacherUpdated`
- `AssignClassTeacherDeleted`

---

## HourlyRate

**Root type:** `HourlyRate`
**Identity:** `HourlyRateId(SchoolId, Uuid)`

### Purpose

A per-grade hourly rate. Carries `grade` and `rate`.

### Invariants

1. An `HourlyRate` is unique by `(school_id, grade, academic_id)`.
2. `rate > 0`.

### Commands

- `SetHourlyRate`
- `UpdateHourlyRate`
- `DeleteHourlyRate`

### Events

- `HourlyRateSet`
- `HourlyRateUpdated`
- `HourlyRateDeleted`

---

## SalaryTemplate

**Root type:** `SalaryTemplate`
**Identity:** `SalaryTemplateId(SchoolId, Uuid)`

### Purpose

A reusable salary grade and structure. Carries `salary_grades`,
`salary_basic`, `overtime_rate`, `house_rent`, `provident_fund`,
`gross_salary`, `total_deduction`, `net_salary`.

### Invariants

1. A `SalaryTemplate` is unique by `(school_id, salary_grades,
   academic_id)`.
2. `gross_salary == salary_basic + house_rent + provident_fund` (or
   the consumer-defined composition).
3. `net_salary == gross_salary - total_deduction`.
4. The template is `active` while in use.

### Commands

- `CreateSalaryTemplate`
- `UpdateSalaryTemplate`
- `DeleteSalaryTemplate`

### Events

- `SalaryTemplateCreated`
- `SalaryTemplateUpdated`
- `SalaryTemplateDeleted`

---

## PayrollGenerate

**Root type:** `PayrollGenerate`
**Identity:** `PayrollGenerateId(SchoolId, Uuid)`

### Purpose

The monthly payroll run for a single staff member. Carries
`basic_salary`, `total_earning`, `total_deduction`, `gross_salary`,
`tax`, `net_salary`, `payroll_month`, `payroll_year`,
`payroll_status` (`not_generated`, `generated`, `paid`),
`payment_mode`, `payment_date`, `bank_id`, `note`, `paid_amount`,
and `is_partial`.

### Invariants

1. `gross_salary == basic_salary + total_earning`.
2. `net_salary == gross_salary - total_deduction - tax`.
3. `payroll_status` transitions:
   `not_generated → generated → paid`. `paid` is terminal.
4. `paid_amount <= net_salary`.
5. A payroll is unique by `(school_id, staff_id, payroll_month,
   payroll_year)`.
6. The payroll has at most one `LeaveDeductionInfo` line per run.

### Commands

- `GeneratePayroll`
- `UpdatePayrollAmounts`
- `ApprovePayroll`
- `MarkPayrollPaid` (HR-side acknowledgement of finance payment)

### Events

- `PayrollGenerated`
- `PayrollAmountsUpdated`
- `PayrollApproved`
- `PayrollPaid`

---

## PayrollEarnDeduc

**Root type:** `PayrollEarnDeduc`
**Identity:** `PayrollEarnDeducId(SchoolId, Uuid)`

### Purpose

A single earnings or deductions line on a `PayrollGenerate`. Carries
`type_name`, `amount`, and `earn_dedc_type` (`e` or `d`).

### Invariants

1. `amount >= 0`.
2. `earn_dedc_type` is `e` (earning) or `d` (deduction).
3. The sum of `e` rows for a payroll equals `total_earning`; the
   sum of `d` rows equals `total_deduction`.

### Commands

- `AddPayrollEarning`
- `AddPayrollDeduction`
- `UpdatePayrollEarnDeduc`
- `DeletePayrollEarnDeduc`

### Events

- `PayrollEarningAdded`
- `PayrollDeductionAdded`
- `PayrollEarnDeducUpdated`
- `PayrollEarnDeducDeleted`

---

## LeaveDeductionInfo

**Root type:** `LeaveDeductionInfo`
**Identity:** `LeaveDeductionInfoId(SchoolId, Uuid)`

### Purpose

A per-payroll leave-deduction record. Carries `staff_id`,
`payroll_id`, `extra_leave`, `salary_deduct`, `pay_month`,
`pay_year`, and `active_status`.

### Invariants

1. A `LeaveDeductionInfo` is unique by `(school_id, staff_id,
   payroll_id)`.
2. `extra_leave >= 0` and `salary_deduct >= 0`.
3. The deduction is `active` while applied.

### Commands

- `AddLeaveDeductionInfo`
- `UpdateLeaveDeductionInfo`
- `DeleteLeaveDeductionInfo`

### Events

- `LeaveDeductionInfoAdded`
- `LeaveDeductionInfoUpdated`
- `LeaveDeductionInfoDeleted`

---

## StaffRegistrationField

**Root type:** `StaffRegistrationField`
**Identity:** `StaffRegistrationFieldId(SchoolId, Uuid)`

### Purpose

A custom field on the staff registration form. Carries
`field_name`, `label_name`, `active_status`, `is_required`,
`staff_edit`, `required_type`, and `position`.

### Invariants

1. A `StaffRegistrationField` is unique by `(school_id, field_name,
   academic_id)`.
2. `position` is a non-negative integer.

### Commands

- `CreateStaffRegistrationField`
- `UpdateStaffRegistrationField`
- `DeleteStaffRegistrationField`

### Events

- `StaffRegistrationFieldCreated`
- `StaffRegistrationFieldUpdated`
- `StaffRegistrationFieldDeleted`

---

## StaffImportBulkTemporary

**Root type:** `StaffImportBulkTemporary`
**Identity:** `StaffImportBulkTemporaryId(SchoolId, Uuid)`

### Purpose

A staging row for a bulk staff import. Carries all staff fields as
strings plus the resolved `role`, `department`, `designation`,
`gender_id`, and the `user_id` of the importer.

### Invariants

1. A row is unique by `(school_id, email)` and `(school_id,
   staff_no)` (when provided).
2. The row is `active` while pending promotion; promotion creates
   a `Staff` and a `User` (the user is created by the platform
   port).

### Commands

- `ImportStaffBulk`
- `PromoteStaffImport`
- `RejectStaffImport`

### Events

- `StaffBulkImported`
- `StaffImportPromoted`
- `StaffImportRejected`

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## AssignClassTeacherScope

**Root type:** `AssignClassTeacherScope`
**Identity:** `AssignClassTeacherScopeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `AssignClassTeacherScope` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `AssignClassTeacherScopeId` within a school.

### Commands

- `CreateAssignClassTeacherScope`
- `UpdateAssignClassTeacherScope`
- `DeleteAssignClassTeacherScope`

### Events

- `AssignClassTeacherScopeCreated`

---

## BulkImportJob

**Root type:** `BulkImportJob`
**Identity:** `BulkImportJobId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `BulkImportJob` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BulkImportJobId` within a school.

### Commands

- `CreateBulkImportJob`
- `UpdateBulkImportJob`
- `DeleteBulkImportJob`

### Events

- `BulkImportJobCreated`

---

## DepartmentHead

**Root type:** `DepartmentHead`
**Identity:** `DepartmentHeadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `DepartmentHead` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DepartmentHeadId` within a school.

### Commands

- `CreateDepartmentHead`
- `UpdateDepartmentHead`
- `DeleteDepartmentHead`

### Events

- `DepartmentHeadCreated`

---

## DesignationGrade

**Root type:** `DesignationGrade`
**Identity:** `DesignationGradeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `DesignationGrade` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DesignationGradeId` within a school.

### Commands

- `CreateDesignationGrade`
- `UpdateDesignationGrade`
- `DeleteDesignationGrade`

### Events

- `DesignationGradeCreated`

---

## HourlyRateOverride

**Root type:** `HourlyRateOverride`
**Identity:** `HourlyRateOverrideId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `HourlyRateOverride` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `HourlyRateOverrideId` within a school.

### Commands

- `CreateHourlyRateOverride`
- `UpdateHourlyRateOverride`
- `DeleteHourlyRateOverride`

### Events

- `HourlyRateOverrideCreated`

---

## LeaveDefineAdjustment

**Root type:** `LeaveDefineAdjustment`
**Identity:** `LeaveDefineAdjustmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveDefineAdjustment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveDefineAdjustmentId` within a school.

### Commands

- `CreateLeaveDefineAdjustment`
- `UpdateLeaveDefineAdjustment`
- `DeleteLeaveDefineAdjustment`

### Events

- `LeaveDefineAdjustmentCreated`

---

## LeaveRequestApproval

**Root type:** `LeaveRequestApproval`
**Identity:** `LeaveRequestApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveRequestApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveRequestApprovalId` within a school.

### Commands

- `CreateLeaveRequestApproval`
- `UpdateLeaveRequestApproval`
- `DeleteLeaveRequestApproval`

### Events

- `LeaveRequestApprovalCreated`

---

## LeaveRequestAttachment

**Root type:** `LeaveRequestAttachment`
**Identity:** `LeaveRequestAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveRequestAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveRequestAttachmentId` within a school.

### Commands

- `CreateLeaveRequestAttachment`
- `UpdateLeaveRequestAttachment`
- `DeleteLeaveRequestAttachment`

### Events

- `LeaveRequestAttachmentCreated`

---

## PayrollGenerateAudit

**Root type:** `PayrollGenerateAudit`
**Identity:** `PayrollGenerateAuditId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `PayrollGenerateAudit` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollGenerateAuditId` within a school.

### Commands

- `CreatePayrollGenerateAudit`
- `UpdatePayrollGenerateAudit`
- `DeletePayrollGenerateAudit`

### Events

- `PayrollGenerateAuditCreated`

---

## PayrollPaymentLink

**Root type:** `PayrollPaymentLink`
**Identity:** `PayrollPaymentLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `PayrollPaymentLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollPaymentLinkId` within a school.

### Commands

- `CreatePayrollPaymentLink`
- `UpdatePayrollPaymentLink`
- `DeletePayrollPaymentLink`

### Events

- `PayrollPaymentLinkCreated`

---

## StaffAddress

**Root type:** `StaffAddress`
**Identity:** `StaffAddressId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAddress` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAddressId` within a school.

### Commands

- `CreateStaffAddress`
- `UpdateStaffAddress`
- `DeleteStaffAddress`

### Events

- `StaffAddressCreated`

---

## StaffAttendanceImportBatch

**Root type:** `StaffAttendanceImportBatch`
**Identity:** `StaffAttendanceImportBatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAttendanceImportBatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAttendanceImportBatchId` within a school.

### Commands

- `CreateStaffAttendanceImportBatch`
- `UpdateStaffAttendanceImportBatch`
- `DeleteStaffAttendanceImportBatch`

### Events

- `StaffAttendanceImportBatchCreated`

---

## StaffAttendancePunch

**Root type:** `StaffAttendancePunch`
**Identity:** `StaffAttendancePunchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAttendancePunch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAttendancePunchId` within a school.

### Commands

- `CreateStaffAttendancePunch`
- `UpdateStaffAttendancePunch`
- `DeleteStaffAttendancePunch`

### Events

- `StaffAttendancePunchCreated`

---

## StaffBankDetail

**Root type:** `StaffBankDetail`
**Identity:** `StaffBankDetailId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffBankDetail` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffBankDetailId` within a school.

### Commands

- `CreateStaffBankDetail`
- `UpdateStaffBankDetail`
- `DeleteStaffBankDetail`

### Events

- `StaffBankDetailCreated`

---

## StaffCustomField

**Root type:** `StaffCustomField`
**Identity:** `StaffCustomFieldId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffCustomField` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffCustomFieldId` within a school.

### Commands

- `CreateStaffCustomField`
- `UpdateStaffCustomField`
- `DeleteStaffCustomField`

### Events

- `StaffCustomFieldCreated`

---

## StaffDocument

**Root type:** `StaffDocument`
**Identity:** `StaffDocumentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffDocument` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffDocumentId` within a school.

### Commands

- `CreateStaffDocument`
- `UpdateStaffDocument`
- `DeleteStaffDocument`

### Events

- `StaffDocumentCreated`

---

## StaffDrivingLicense

**Root type:** `StaffDrivingLicense`
**Identity:** `StaffDrivingLicenseId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffDrivingLicense` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffDrivingLicenseId` within a school.

### Commands

- `CreateStaffDrivingLicense`
- `UpdateStaffDrivingLicense`
- `DeleteStaffDrivingLicense`

### Events

- `StaffDrivingLicenseCreated`

---

## StaffImportResolution

**Root type:** `StaffImportResolution`
**Identity:** `StaffImportResolutionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffImportResolution` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffImportResolutionId` within a school.

### Commands

- `CreateStaffImportResolution`
- `UpdateStaffImportResolution`
- `DeleteStaffImportResolution`

### Events

- `StaffImportResolutionCreated`

---

## StaffLeaveBalance

**Root type:** `StaffLeaveBalance`
**Identity:** `StaffLeaveBalanceId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffLeaveBalance` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffLeaveBalanceId` within a school.

### Commands

- `CreateStaffLeaveBalance`
- `UpdateStaffLeaveBalance`
- `DeleteStaffLeaveBalance`

### Events

- `StaffLeaveBalanceCreated`

---

## StaffLeaveHistory

**Root type:** `StaffLeaveHistory`
**Identity:** `StaffLeaveHistoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffLeaveHistory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffLeaveHistoryId` within a school.

### Commands

- `CreateStaffLeaveHistory`
- `UpdateStaffLeaveHistory`
- `DeleteStaffLeaveHistory`

### Events

- `StaffLeaveHistoryCreated`

---

## StaffPayrollHistory

**Root type:** `StaffPayrollHistory`
**Identity:** `StaffPayrollHistoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffPayrollHistory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffPayrollHistoryId` within a school.

### Commands

- `CreateStaffPayrollHistory`
- `UpdateStaffPayrollHistory`
- `DeleteStaffPayrollHistory`

### Events

- `StaffPayrollHistoryCreated`

---

## StaffProfilePhoto

**Root type:** `StaffProfilePhoto`
**Identity:** `StaffProfilePhotoId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffProfilePhoto` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffProfilePhotoId` within a school.

### Commands

- `CreateStaffProfilePhoto`
- `UpdateStaffProfilePhoto`
- `DeleteStaffProfilePhoto`

### Events

- `StaffProfilePhotoCreated`

---

## StaffRegistrationFieldOption

**Root type:** `StaffRegistrationFieldOption`
**Identity:** `StaffRegistrationFieldOptionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffRegistrationFieldOption` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffRegistrationFieldOptionId` within a school.

### Commands

- `CreateStaffRegistrationFieldOption`
- `UpdateStaffRegistrationFieldOption`
- `DeleteStaffRegistrationFieldOption`

### Events

- `StaffRegistrationFieldOptionCreated`

---

## StaffRoleAssignment

**Root type:** `StaffRoleAssignment`
**Identity:** `StaffRoleAssignmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffRoleAssignment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffRoleAssignmentId` within a school.

### Commands

- `CreateStaffRoleAssignment`
- `UpdateStaffRoleAssignment`
- `DeleteStaffRoleAssignment`

### Events

- `StaffRoleAssignmentCreated`

---

## StaffSocialLink

**Root type:** `StaffSocialLink`
**Identity:** `StaffSocialLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffSocialLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffSocialLinkId` within a school.

### Commands

- `CreateStaffSocialLink`
- `UpdateStaffSocialLink`
- `DeleteStaffSocialLink`

### Events

- `StaffSocialLinkCreated`

---

## StaffTimeline

**Root type:** `StaffTimeline`
**Identity:** `StaffTimelineId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffTimeline` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffTimelineId` within a school.

### Commands

- `CreateStaffTimeline`
- `UpdateStaffTimeline`
- `DeleteStaffTimeline`

### Events

- `StaffTimelineCreated`

---



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## AssignClassTeacherScope

**Root type:** `AssignClassTeacherScope`
**Identity:** `AssignClassTeacherScopeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `AssignClassTeacherScope` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `AssignClassTeacherScopeId` within a school.

### Commands

- `CreateAssignClassTeacherScope`
- `UpdateAssignClassTeacherScope`
- `DeleteAssignClassTeacherScope`

### Events

- `AssignClassTeacherScopeCreated`

---

## BulkImportJob

**Root type:** `BulkImportJob`
**Identity:** `BulkImportJobId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `BulkImportJob` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `BulkImportJobId` within a school.

### Commands

- `CreateBulkImportJob`
- `UpdateBulkImportJob`
- `DeleteBulkImportJob`

### Events

- `BulkImportJobCreated`

---

## DepartmentHead

**Root type:** `DepartmentHead`
**Identity:** `DepartmentHeadId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `DepartmentHead` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DepartmentHeadId` within a school.

### Commands

- `CreateDepartmentHead`
- `UpdateDepartmentHead`
- `DeleteDepartmentHead`

### Events

- `DepartmentHeadCreated`

---

## DesignationGrade

**Root type:** `DesignationGrade`
**Identity:** `DesignationGradeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `DesignationGrade` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `DesignationGradeId` within a school.

### Commands

- `CreateDesignationGrade`
- `UpdateDesignationGrade`
- `DeleteDesignationGrade`

### Events

- `DesignationGradeCreated`

---

## HourlyRateOverride

**Root type:** `HourlyRateOverride`
**Identity:** `HourlyRateOverrideId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `HourlyRateOverride` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `HourlyRateOverrideId` within a school.

### Commands

- `CreateHourlyRateOverride`
- `UpdateHourlyRateOverride`
- `DeleteHourlyRateOverride`

### Events

- `HourlyRateOverrideCreated`

---

## LeaveDefineAdjustment

**Root type:** `LeaveDefineAdjustment`
**Identity:** `LeaveDefineAdjustmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveDefineAdjustment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveDefineAdjustmentId` within a school.

### Commands

- `CreateLeaveDefineAdjustment`
- `UpdateLeaveDefineAdjustment`
- `DeleteLeaveDefineAdjustment`

### Events

- `LeaveDefineAdjustmentCreated`

---

## LeaveRequestApproval

**Root type:** `LeaveRequestApproval`
**Identity:** `LeaveRequestApprovalId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveRequestApproval` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveRequestApprovalId` within a school.

### Commands

- `CreateLeaveRequestApproval`
- `UpdateLeaveRequestApproval`
- `DeleteLeaveRequestApproval`

### Events

- `LeaveRequestApprovalCreated`

---

## LeaveRequestAttachment

**Root type:** `LeaveRequestAttachment`
**Identity:** `LeaveRequestAttachmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `LeaveRequestAttachment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `LeaveRequestAttachmentId` within a school.

### Commands

- `CreateLeaveRequestAttachment`
- `UpdateLeaveRequestAttachment`
- `DeleteLeaveRequestAttachment`

### Events

- `LeaveRequestAttachmentCreated`

---

## PayrollGenerateAudit

**Root type:** `PayrollGenerateAudit`
**Identity:** `PayrollGenerateAuditId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `PayrollGenerateAudit` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollGenerateAuditId` within a school.

### Commands

- `CreatePayrollGenerateAudit`
- `UpdatePayrollGenerateAudit`
- `DeletePayrollGenerateAudit`

### Events

- `PayrollGenerateAuditCreated`

---

## PayrollPaymentLink

**Root type:** `PayrollPaymentLink`
**Identity:** `PayrollPaymentLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `PayrollPaymentLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `PayrollPaymentLinkId` within a school.

### Commands

- `CreatePayrollPaymentLink`
- `UpdatePayrollPaymentLink`
- `DeletePayrollPaymentLink`

### Events

- `PayrollPaymentLinkCreated`

---

## StaffAddress

**Root type:** `StaffAddress`
**Identity:** `StaffAddressId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAddress` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAddressId` within a school.

### Commands

- `CreateStaffAddress`
- `UpdateStaffAddress`
- `DeleteStaffAddress`

### Events

- `StaffAddressCreated`

---

## StaffAttendanceImportBatch

**Root type:** `StaffAttendanceImportBatch`
**Identity:** `StaffAttendanceImportBatchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAttendanceImportBatch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAttendanceImportBatchId` within a school.

### Commands

- `CreateStaffAttendanceImportBatch`
- `UpdateStaffAttendanceImportBatch`
- `DeleteStaffAttendanceImportBatch`

### Events

- `StaffAttendanceImportBatchCreated`

---

## StaffAttendancePunch

**Root type:** `StaffAttendancePunch`
**Identity:** `StaffAttendancePunchId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffAttendancePunch` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffAttendancePunchId` within a school.

### Commands

- `CreateStaffAttendancePunch`
- `UpdateStaffAttendancePunch`
- `DeleteStaffAttendancePunch`

### Events

- `StaffAttendancePunchCreated`

---

## StaffBankDetail

**Root type:** `StaffBankDetail`
**Identity:** `StaffBankDetailId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffBankDetail` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffBankDetailId` within a school.

### Commands

- `CreateStaffBankDetail`
- `UpdateStaffBankDetail`
- `DeleteStaffBankDetail`

### Events

- `StaffBankDetailCreated`

---

## StaffCustomField

**Root type:** `StaffCustomField`
**Identity:** `StaffCustomFieldId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffCustomField` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffCustomFieldId` within a school.

### Commands

- `CreateStaffCustomField`
- `UpdateStaffCustomField`
- `DeleteStaffCustomField`

### Events

- `StaffCustomFieldCreated`

---

## StaffDocument

**Root type:** `StaffDocument`
**Identity:** `StaffDocumentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffDocument` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffDocumentId` within a school.

### Commands

- `CreateStaffDocument`
- `UpdateStaffDocument`
- `DeleteStaffDocument`

### Events

- `StaffDocumentCreated`

---

## StaffDrivingLicense

**Root type:** `StaffDrivingLicense`
**Identity:** `StaffDrivingLicenseId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffDrivingLicense` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffDrivingLicenseId` within a school.

### Commands

- `CreateStaffDrivingLicense`
- `UpdateStaffDrivingLicense`
- `DeleteStaffDrivingLicense`

### Events

- `StaffDrivingLicenseCreated`

---

## StaffImportResolution

**Root type:** `StaffImportResolution`
**Identity:** `StaffImportResolutionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffImportResolution` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffImportResolutionId` within a school.

### Commands

- `CreateStaffImportResolution`
- `UpdateStaffImportResolution`
- `DeleteStaffImportResolution`

### Events

- `StaffImportResolutionCreated`

---

## StaffLeaveBalance

**Root type:** `StaffLeaveBalance`
**Identity:** `StaffLeaveBalanceId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffLeaveBalance` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffLeaveBalanceId` within a school.

### Commands

- `CreateStaffLeaveBalance`
- `UpdateStaffLeaveBalance`
- `DeleteStaffLeaveBalance`

### Events

- `StaffLeaveBalanceCreated`

---

## StaffLeaveHistory

**Root type:** `StaffLeaveHistory`
**Identity:** `StaffLeaveHistoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffLeaveHistory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffLeaveHistoryId` within a school.

### Commands

- `CreateStaffLeaveHistory`
- `UpdateStaffLeaveHistory`
- `DeleteStaffLeaveHistory`

### Events

- `StaffLeaveHistoryCreated`

---

## StaffPayrollHistory

**Root type:** `StaffPayrollHistory`
**Identity:** `StaffPayrollHistoryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffPayrollHistory` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffPayrollHistoryId` within a school.

### Commands

- `CreateStaffPayrollHistory`
- `UpdateStaffPayrollHistory`
- `DeleteStaffPayrollHistory`

### Events

- `StaffPayrollHistoryCreated`

---

## StaffProfilePhoto

**Root type:** `StaffProfilePhoto`
**Identity:** `StaffProfilePhotoId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffProfilePhoto` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffProfilePhotoId` within a school.

### Commands

- `CreateStaffProfilePhoto`
- `UpdateStaffProfilePhoto`
- `DeleteStaffProfilePhoto`

### Events

- `StaffProfilePhotoCreated`

---

## StaffRegistrationFieldOption

**Root type:** `StaffRegistrationFieldOption`
**Identity:** `StaffRegistrationFieldOptionId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffRegistrationFieldOption` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffRegistrationFieldOptionId` within a school.

### Commands

- `CreateStaffRegistrationFieldOption`
- `UpdateStaffRegistrationFieldOption`
- `DeleteStaffRegistrationFieldOption`

### Events

- `StaffRegistrationFieldOptionCreated`

---

## StaffRoleAssignment

**Root type:** `StaffRoleAssignment`
**Identity:** `StaffRoleAssignmentId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffRoleAssignment` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffRoleAssignmentId` within a school.

### Commands

- `CreateStaffRoleAssignment`
- `UpdateStaffRoleAssignment`
- `DeleteStaffRoleAssignment`

### Events

- `StaffRoleAssignmentCreated`

---

## StaffSocialLink

**Root type:** `StaffSocialLink`
**Identity:** `StaffSocialLinkId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffSocialLink` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffSocialLinkId` within a school.

### Commands

- `CreateStaffSocialLink`
- `UpdateStaffSocialLink`
- `DeleteStaffSocialLink`

### Events

- `StaffSocialLinkCreated`

---

## StaffTimeline

**Root type:** `StaffTimeline`
**Identity:** `StaffTimelineId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Hr

### Purpose

The `StaffTimeline` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `StaffTimelineId` within a school.

### Commands

- `CreateStaffTimeline`
- `UpdateStaffTimeline`
- `DeleteStaffTimeline`

### Events

- `StaffTimelineCreated`

---
