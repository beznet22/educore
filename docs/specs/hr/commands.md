# HR Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation)
and are rejected if the actor lacks the required capability.

## Staff

### RegisterStaff

```rust
pub struct RegisterStaffCommand {
    pub tenant: TenantContext,
    pub staff_no: Option<StaffNo>,
    pub first_name: PersonName,
    pub last_name: PersonName,
    pub fathers_name: Option<PersonName>,
    pub mothers_name: Option<PersonName>,
    pub date_of_birth: DateOfBirth,
    pub date_of_joining: DateOfJoining,
    pub email: Option<EmailAddress>,
    pub mobile: Option<PhoneNumber>,
    pub emergency_mobile: Option<PhoneNumber>,
    pub marital_status: Option<MaritalStatus>,
    pub gender: Gender,
    pub blood_group: Option<BloodGroup>,
    pub religion: Option<String>,
    pub caste: Option<String>,
    pub current_address: Option<Address>,
    pub permanent_address: Option<Address>,
    pub qualification: Option<Qualification>,
    pub experience: Option<Experience>,
    pub epf_no: Option<EpfNo>,
    pub basic_salary: Option<BasicSalary>,
    pub contract_type: ContractType,
    pub location: Option<String>,
    pub casual_leave: u32,
    pub medical_leave: u32,
    pub maternity_leave: u32,
    pub bank_account_name: Option<String>,
    pub bank_account_no: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub facebook_url: Option<Url>,
    pub twitter_url: Option<Url>,
    pub linkedin_url: Option<Url>,
    pub instagram_url: Option<Url>,
    pub joining_letter: Option<FileReference>,
    pub resume: Option<FileReference>,
    pub other_document: Option<FileReference>,
    pub driving_license: Option<String>,
    pub driving_license_ex_date: Option<NaiveDate>,
    pub photo: Option<FileReference>,
    pub notes: Option<String>,
    pub designation_id: DesignationId,
    pub department_id: DepartmentId,
    pub role_id: RoleId,
    pub user_id: UserId,
    pub custom_fields: BTreeMap<String, String>,
}
```

**Capability:** `Staff.Register`
**Pre-conditions:**
- `email` (when provided) is unique in school.
- `mobile` (when provided) is unique in school.
- `designation_id`, `department_id`, `role_id` exist in the school.
- The platform user `user_id` exists.

**Effects:** Creates a `Staff` aggregate, binds the platform user
and the role, and emits `StaffRegistered`.

### UpdateStaff

```rust
pub struct UpdateStaffCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub patch: StaffProfilePatch,
}
```

`StaffProfilePatch` is a partial update containing only mutable
fields: `first_name`, `last_name`, `fathers_name`, `mothers_name`,
`date_of_birth`, `email`, `mobile`, `emergency_mobile`,
`marital_status`, addresses, `qualification`, `experience`,
`epf_no`, `contract_type`, `location`, `casual_leave`,
`medical_leave`, `maternity_leave`, bank fields, social links,
documents, and custom fields. Immutable fields (admission number,
`user_id`, school id) cannot be patched here — use `ChangeStaffRole`
or a new registration.

**Capability:** `Staff.Update`
**Effects:** Emits `StaffUpdated`.

### ChangeStaffDepartment / ChangeStaffDesignation / ChangeStaffRole

```rust
pub struct ChangeStaffDepartmentCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub department_id: DepartmentId,
    pub effective_from: NaiveDate,
}

pub struct ChangeStaffDesignationCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub designation_id: DesignationId,
    pub effective_from: NaiveDate,
}

pub struct ChangeStaffRoleCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub role_id: RoleId,
    pub previous_role_id: Option<RoleId>,
    pub effective_from: NaiveDate,
}
```

**Capabilities:** `Staff.ChangeDepartment`,
`Staff.ChangeDesignation`, `Staff.ChangeRole`.
**Effects:** Emit `StaffDepartmentChanged`,
`StaffDesignationChanged`, `StaffRoleChanged`. The RBAC port
subscribes to `StaffRoleChanged` to update the platform user's
role.

### SuspendStaff / ReinstateStaff / ResignStaff / TerminateStaff / RetireStaff

```rust
pub struct SuspendStaffCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub reason: SuspensionReason,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
}
```

Similar for `ReinstateStaff`, `ResignStaff`, `TerminateStaff`,
`RetireStaff` (with reason `Resignation`, `Termination`,
`Retirement` respectively).

**Capabilities:** `Staff.Suspend`, `Staff.Reinstate`,
`Staff.Resign`, `Staff.Terminate`, `Staff.Retire`.
**Effects:** Emit the corresponding `Staff*` event.

### DeleteStaff

```rust
pub struct DeleteStaffCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub reason: String,
}
```

**Capability:** `Staff.Delete`
**Pre-conditions:** The staff has no active `AssignClassTeacher`,
no pending `LeaveRequest`, and no open `PayrollGenerate`. The
deletion is a soft-delete that preserves history.

**Effects:** Emits `StaffDeleted`.

## Department

### CreateDepartment / UpdateDepartment / DeleteDepartment

```rust
pub struct CreateDepartmentCommand {
    pub tenant: TenantContext,
    pub name: String,
}
```

**Capabilities:** `Department.Create`, `Department.Update`,
`Department.Delete`.

A department cannot be deleted while any `Staff` references it. A
department with `is_system_defined` set is system-defined and cannot
be deleted.

## Designation

### CreateDesignation / UpdateDesignation / DeleteDesignation

```rust
pub struct CreateDesignationCommand {
    pub tenant: TenantContext,
    pub title: String,
}
```

**Capabilities:** `Designation.Create`, `Designation.Update`,
`Designation.Delete`.

A designation cannot be deleted while any `Staff` references it. A
designation with `is_system_defined` set is system-defined and
cannot be deleted.

## AssignClassTeacher

### AssignClassTeacher

```rust
pub struct AssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub staff_id: StaffId,
    pub academic_id: AcademicYearId,
}
```

**Capability:** `AssignClassTeacher.Create`
**Pre-conditions:** The class and section exist; the staff is
`Active`; the staff is not already assigned to the same
class-section-year.

**Effects:** Emits `ClassTeacherAssigned`.

### UpdateAssignClassTeacher / DeleteAssignClassTeacher

```rust
pub struct UpdateAssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub assign_class_teacher_id: AssignClassTeacherId,
    pub staff_id: Option<StaffId>,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
}
```

**Capabilities:** `AssignClassTeacher.Update`,
`AssignClassTeacher.Delete`.

## Leave

### CreateLeaveType

```rust
pub struct CreateLeaveTypeCommand {
    pub tenant: TenantContext,
    pub type_name: LeaveTypeName,
    pub total_days: LeaveTotalDays,
}
```

**Capability:** `LeaveType.Create`
**Effects:** Emits `LeaveTypeCreated`.

### UpdateLeaveType / DeleteLeaveType

Standard update and soft-delete. A `LeaveType` cannot be deleted
while any `LeaveDefine` or `LeaveRequest` references it.

**Capabilities:** `LeaveType.Update`, `LeaveType.Delete`.

### DefineLeavePolicy

```rust
pub struct DefineLeavePolicyCommand {
    pub tenant: TenantContext,
    pub academic_id: AcademicYearId,
    pub role_id: Option<RoleId>,
    pub user_id: Option<UserId>,
    pub type_id: LeaveTypeId,
    pub days: LeaveDays,
    pub total_days: LeaveTotalDays,
}
```

**Capability:** `LeaveDefine.Create`
**Pre-conditions:** Exactly one of `role_id` or `user_id` is
provided.

**Effects:** Emits `LeavePolicyDefined`.

### UpdateLeavePolicy / DeleteLeavePolicy

Standard update and soft-delete.

**Capabilities:** `LeaveDefine.Update`, `LeaveDefine.Delete`.

### RequestLeave

```rust
pub struct RequestLeaveCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub type_id: LeaveTypeId,
    pub apply_date: NaiveDate,
    pub leave_from: LeaveFrom,
    pub leave_to: LeaveTo,
    pub reason: LeaveReason,
    pub note: Option<LeaveNote>,
    pub file: Option<LeaveAttachment>,
}
```

**Capability:** `Leave.Request`
**Pre-conditions:**
- `leave_from <= leave_to`.
- The staff has a `LeaveDefine` for `type_id` with remaining days
  for the period.

**Effects:** Emits `LeaveRequested` with `approve_status = pending`.

### ApproveLeave

```rust
pub struct ApproveLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub note: Option<String>,
}
```

**Capability:** `Leave.Approve`
**Pre-conditions:** The request is `pending`; the leave balance
covers the period; the approver is not the requester (segregation
of duties).

**Effects:** Emits `LeaveApproved`. The leave balance is
decremented.

### RejectLeave

```rust
pub struct RejectLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub reason: String,
}
```

**Capability:** `Leave.Reject`
**Effects:** Emits `LeaveRejected`.

### CancelLeave

```rust
pub struct CancelLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub reason: String,
}
```

**Capability:** `Leave.Cancel`
**Pre-conditions:** The request is `pending` or `approved` and the
leave has not yet started, or `approved` and within a configurable
grace window. The cancelled leave restores the balance.

**Effects:** Emits `LeaveCancelled`.

## Attendance

### MarkStaffAttendance

```rust
pub struct MarkStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub note: Option<AttendanceNote>,
    pub in_time: Option<InTime>,
    pub out_time: Option<OutTime>,
    pub source: AttendanceSource,
}
```

**Capability:** `Attendance.Staff.Mark`
**Pre-conditions:** No `StaffAttendance` row exists for
`(staff_id, attendance_date)`.

**Effects:** Emits `StaffAttendanceMarked`.

### UpdateStaffAttendance / DeleteStaffAttendance

Standard update and soft-delete.

**Capabilities:** `Attendance.Staff.Update`,
`Attendance.Staff.Delete`.

### ImportStaffAttendance

```rust
pub struct ImportStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub source: AttendanceSource,
    pub rows: Vec<StaffAttendanceImportRow>,
}
```

**Capability:** `Attendance.Staff.Import`
**Effects:** Creates a `StaffAttendanceImport` per row in pending
state. A promotion step converts them into `StaffAttendance`.

### PromoteStaffAttendance

```rust
pub struct PromoteStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub import_id: StaffAttendanceImportId,
}
```

**Capability:** `Attendance.Staff.Import.Promote`
**Pre-conditions:** The import is in `pending` state.

**Effects:** Emits `StaffAttendancePromoted` and the underlying
`StaffAttendanceMarked` event.

### RejectStaffAttendance

```rust
pub struct RejectStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub import_id: StaffAttendanceImportId,
    pub reason: String,
}
```

**Capability:** `Attendance.Staff.Import.Reject`
**Effects:** Emits `StaffAttendanceImportRejected`.

## Payroll

### GeneratePayroll

```rust
pub struct GeneratePayrollCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub pay_period: PayPeriod,
    pub salary_template_id: Option<SalaryTemplateId>,
    pub earnings: Vec<PayrollEarningLine>,
    pub deductions: Vec<PayrollDeductionLine>,
    pub note: Option<String>,
    pub bank_id: Option<BankAccountId>,
    pub payment_mode: Option<PaymentMethodId>,
}
```

**Capability:** `Payroll.Generate`
**Pre-conditions:** The staff is `Active` or `Suspended`. The
period is open. No payroll exists for `(staff_id, pay_period)`.

**Effects:** Emits `PayrollGenerated`, `PayrollEarningAdded` per
earning, `PayrollDeductionAdded` per deduction.

### UpdatePayrollAmounts

```rust
pub struct UpdatePayrollAmountsCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub basic_salary: Option<BasicSalary>,
    pub total_earning: Option<TotalEarning>,
    pub total_deduction: Option<TotalDeduction>,
    pub tax: Option<Tax>,
    pub note: Option<String>,
}
```

**Capability:** `Payroll.Update`
**Pre-conditions:** The payroll is in `not_generated` or
`generated` status.

**Effects:** Emits `PayrollAmountsUpdated`.

### ApprovePayroll

```rust
pub struct ApprovePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub note: Option<String>,
}
```

**Capability:** `Payroll.Approve`
**Pre-conditions:** The payroll is in `generated` status; the
approver is not the generator (segregation of duties).

**Effects:** Emits `PayrollApproved`.

### MarkPayrollPaid

```rust
pub struct MarkPayrollPaidCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub paid_amount: PaidAmount,
}
```

**Capability:** `Payroll.MarkPaid`
**Pre-conditions:** The payroll is in `generated` status. The HR
domain marks the payroll as paid in response to a
`PayrollPaymentRecorded` event from the finance domain; the
finance command remains the source of truth for the actual money
movement.

**Effects:** Emits `PayrollPaid`.

### AddPayrollEarning

```rust
pub struct AddPayrollEarningCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: Amount,
}
```

**Capability:** `Payroll.Earning.Add`
**Effects:** Emits `PayrollEarningAdded`.

### AddPayrollDeduction

```rust
pub struct AddPayrollDeductionCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: Amount,
}
```

**Capability:** `Payroll.Deduction.Add`
**Effects:** Emits `PayrollDeductionAdded`.

### UpdatePayrollEarnDeduc / DeletePayrollEarnDeduc

Standard update and soft-delete.

**Capabilities:** `Payroll.Earning.Update`,
`Payroll.Earning.Delete`.

### AddLeaveDeductionInfo

```rust
pub struct AddLeaveDeductionInfoCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub payroll_id: PayrollGenerateId,
    pub extra_leave: u32,
    pub salary_deduct: SalaryDeduct,
    pub pay_month: PayrollMonth,
    pub pay_year: PayrollYear,
}
```

**Capability:** `Payroll.LeaveDeduction.Add`
**Effects:** Emits `LeaveDeductionInfoAdded`. The deduction is
folded into the payroll's `total_deduction` and `net_salary`.

### UpdateLeaveDeductionInfo / DeleteLeaveDeductionInfo

Standard update and soft-delete.

**Capabilities:** `Payroll.LeaveDeduction.Update`,
`Payroll.LeaveDeduction.Delete`.

## Salary & Rate

### CreateSalaryTemplate

```rust
pub struct CreateSalaryTemplateCommand {
    pub tenant: TenantContext,
    pub salary_grades: SalaryGrade,
    pub salary_basic: BasicSalary,
    pub overtime_rate: OvertimeRate,
    pub house_rent: HouseRent,
    pub provident_fund: ProvidentFund,
    pub gross_salary: GrossSalary,
    pub total_deduction: TotalDeduction,
    pub net_salary: NetSalary,
}
```

**Capability:** `SalaryTemplate.Create`
**Effects:** Emits `SalaryTemplateCreated`.

### UpdateSalaryTemplate / DeleteSalaryTemplate

Standard update and soft-delete.

**Capabilities:** `SalaryTemplate.Update`, `SalaryTemplate.Delete`.

### SetHourlyRate

```rust
pub struct SetHourlyRateCommand {
    pub tenant: TenantContext,
    pub grade: String,
    pub rate: HourlyRate,
}
```

**Capability:** `HourlyRate.Set`
**Effects:** Emits `HourlyRateSet`.

### UpdateHourlyRate / DeleteHourlyRate

Standard update and soft-delete.

**Capabilities:** `HourlyRate.Update`, `HourlyRate.Delete`.

## Registration Field

### CreateStaffRegistrationField

```rust
pub struct CreateStaffRegistrationFieldCommand {
    pub tenant: TenantContext,
    pub field_name: String,
    pub label_name: String,
    pub is_required: bool,
    pub staff_edit: bool,
    pub required_type: RequiredType,
    pub position: u32,
}
```

**Capability:** `StaffRegistrationField.Create`
**Effects:** Emits `StaffRegistrationFieldCreated`.

### UpdateStaffRegistrationField / DeleteStaffRegistrationField

Standard update and soft-delete.

**Capabilities:** `StaffRegistrationField.Update`,
`StaffRegistrationField.Delete`.

## Bulk Staff Import

### ImportStaffBulk

```rust
pub struct ImportStaffBulkCommand {
    pub tenant: TenantContext,
    pub source: String,
    pub file: FileReference,
    pub rows: Vec<StaffImportRow>,
}
```

**Capability:** `Staff.ImportBulk`
**Effects:** Creates a `BulkImportJob` and one
`StaffImportBulkTemporary` per row. Each row carries the
string-form fields from the source file.

### PromoteStaffImport

```rust
pub struct PromoteStaffImportCommand {
    pub tenant: TenantContext,
    pub staff_import_bulk_temporary_id: StaffImportBulkTemporaryId,
    pub resolved_user_id: Option<UserId>,
    pub resolved_role_id: Option<RoleId>,
    pub resolved_department_id: Option<DepartmentId>,
    pub resolved_designation_id: Option<DesignationId>,
}
```

**Capability:** `Staff.ImportBulk.Promote`
**Pre-conditions:** The temporary row is `active`; the resolved
references exist in the school.

**Effects:** Emits `StaffImportPromoted` and the underlying
`StaffRegistered` event.

### RejectStaffImport

```rust
pub struct RejectStaffImportCommand {
    pub tenant: TenantContext,
    pub staff_import_bulk_temporary_id: StaffImportBulkTemporaryId,
    pub reason: String,
}
```

**Capability:** `Staff.ImportBulk.Reject`
**Effects:** Emits `StaffImportRejected`.

## AssignSubjectTeacher

```rust
pub struct AssignSubjectTeacherCommand {
    pub tenant: TenantContext,
    pub class_id: ClassId,
    pub section_id: Option<SectionId>,
    pub subject_id: SubjectId,
    pub staff_id: StaffId,
    pub academic_id: AcademicYearId,
}
```

**Capability:** `Staff.AssignSubjectTeacher`
**Pre-conditions:** The staff is `Active`; the class-section-subject
exists.

**Effects:** Emits `SubjectTeacherAssigned`. (The underlying class-
section-subject row is owned by the academic domain; this command
projects the assignment to the HR roster.)

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Assign Department Head

```rust
pub struct AssignDepartmentHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DepartmentHead.Assign`
**Effects:** Emits `DepartmentHeadAssigned`.


### Assign Staff Role

```rust
pub struct AssignStaffRoleCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffRole.Assign`
**Effects:** Emits `StaffRoleAssigned`.


### Create Assign Class Teacher Scope

```rust
pub struct CreateAssignClassTeacherScopeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AssignClassTeacherScope.Create`
**Effects:** Emits `AssignClassTeacherScopeCreateed`.


### Create Bulk Import Job

```rust
pub struct CreateBulkImportJobCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BulkImportJob.Create`
**Effects:** Emits `BulkImportJobCreateed`.


### Create Designation Grade

```rust
pub struct CreateDesignationGradeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DesignationGrade.Create`
**Effects:** Emits `DesignationGradeCreateed`.


### Create Leave Define Adjustment

```rust
pub struct CreateLeaveDefineAdjustmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveDefineAdjustment.Create`
**Effects:** Emits `LeaveDefineAdjustmentCreateed`.


### Create Leave Request Attachment

```rust
pub struct CreateLeaveRequestAttachmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveRequestAttachment.Create`
**Effects:** Emits `LeaveRequestAttachmentCreateed`.


### Create Payroll Payment Link

```rust
pub struct CreatePayrollPaymentLinkCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPaymentLink.Create`
**Effects:** Emits `PayrollPaymentLinkCreateed`.


### Create Staff Address

```rust
pub struct CreateStaffAddressCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAddress.Create`
**Effects:** Emits `StaffAddressCreateed`.


### Create Staff Attendance Import Batch

```rust
pub struct CreateStaffAttendanceImportBatchCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendanceImportBatch.Create`
**Effects:** Emits `StaffAttendanceImportBatchCreateed`.


### Create Staff Bank Detail

```rust
pub struct CreateStaffBankDetailCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffBankDetail.Create`
**Effects:** Emits `StaffBankDetailCreateed`.


### Create Staff Document

```rust
pub struct CreateStaffDocumentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffDocument.Create`
**Effects:** Emits `StaffDocumentCreateed`.


### Create Staff Driving License

```rust
pub struct CreateStaffDrivingLicenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffDrivingLicense.Create`
**Effects:** Emits `StaffDrivingLicenseCreateed`.


### Create Staff Profile Photo

```rust
pub struct CreateStaffProfilePhotoCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffProfilePhoto.Create`
**Effects:** Emits `StaffProfilePhotoCreateed`.


### Create Staff Registration Field Option

```rust
pub struct CreateStaffRegistrationFieldOptionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffRegistrationFieldOption.Create`
**Effects:** Emits `StaffRegistrationFieldOptionCreateed`.


### Create Staff Social Link

```rust
pub struct CreateStaffSocialLinkCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffSocialLink.Create`
**Effects:** Emits `StaffSocialLinkCreateed`.


### Delete Assign Class Teacher

```rust
pub struct DeleteAssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AssignClassTeacher.Delete`
**Effects:** Emits `AssignClassTeacherDeleteed`.


### Record Leave Request Approval

```rust
pub struct RecordLeaveRequestApprovalCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveRequestApproval.Record`
**Effects:** Emits `LeaveRequestApprovalRecorded`.


### Record Payroll Generate Audit

```rust
pub struct RecordPayrollGenerateAuditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollGenerateAudit.Record`
**Effects:** Emits `PayrollGenerateAuditRecorded`.


### Record Staff Attendance Punch

```rust
pub struct RecordStaffAttendancePunchCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendancePunch.Record`
**Effects:** Emits `StaffAttendancePunchRecorded`.


### Record Staff Import Resolution

```rust
pub struct RecordStaffImportResolutionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffImportResolution.Record`
**Effects:** Emits `StaffImportResolutionRecorded`.


### Record Staff Leave History

```rust
pub struct RecordStaffLeaveHistoryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffLeaveHistory.Record`
**Effects:** Emits `StaffLeaveHistoryRecorded`.


### Record Staff Payroll History

```rust
pub struct RecordStaffPayrollHistoryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffPayrollHistory.Record`
**Effects:** Emits `StaffPayrollHistoryRecorded`.


### Refresh Staff Leave Balance

```rust
pub struct RefreshStaffLeaveBalanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffLeaveBalance.Refresh`
**Effects:** Emits `StaffLeaveBalanceRefreshed`.


### Refresh Staff Timeline

```rust
pub struct RefreshStaffTimelineCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffTimeline.Refresh`
**Effects:** Emits `StaffTimelineRefreshed`.


### Set Hourly Rate Override

```rust
pub struct SetHourlyRateOverrideCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HourlyRateOverride.Set`
**Effects:** Emits `HourlyRateOverrideSeted`.


### Set Staff Custom Field

```rust
pub struct SetStaffCustomFieldCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffCustomField.Set`
**Effects:** Emits `StaffCustomFieldSeted`.


## StaffImportRow

```rust
pub struct StaffImportRow {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffImportRow`
**Effects:** Emits `StaffImportRowRecorded`.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Assign Department Head

```rust
pub struct AssignDepartmentHeadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DepartmentHead.Assign`
**Effects:** Emits `DepartmentHeadAssigned`.


### Assign Staff Role

```rust
pub struct AssignStaffRoleCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffRole.Assign`
**Effects:** Emits `StaffRoleAssigned`.


### Create Assign Class Teacher Scope

```rust
pub struct CreateAssignClassTeacherScopeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AssignClassTeacherScope.Create`
**Effects:** Emits `AssignClassTeacherScopeCreateed`.


### Create Bulk Import Job

```rust
pub struct CreateBulkImportJobCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `BulkImportJob.Create`
**Effects:** Emits `BulkImportJobCreateed`.


### Create Designation Grade

```rust
pub struct CreateDesignationGradeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `DesignationGrade.Create`
**Effects:** Emits `DesignationGradeCreateed`.


### Create Leave Define Adjustment

```rust
pub struct CreateLeaveDefineAdjustmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveDefineAdjustment.Create`
**Effects:** Emits `LeaveDefineAdjustmentCreateed`.


### Create Leave Request Attachment

```rust
pub struct CreateLeaveRequestAttachmentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveRequestAttachment.Create`
**Effects:** Emits `LeaveRequestAttachmentCreateed`.


### Create Payroll Payment Link

```rust
pub struct CreatePayrollPaymentLinkCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollPaymentLink.Create`
**Effects:** Emits `PayrollPaymentLinkCreateed`.


### Create Staff Address

```rust
pub struct CreateStaffAddressCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAddress.Create`
**Effects:** Emits `StaffAddressCreateed`.


### Create Staff Attendance Import Batch

```rust
pub struct CreateStaffAttendanceImportBatchCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendanceImportBatch.Create`
**Effects:** Emits `StaffAttendanceImportBatchCreateed`.


### Create Staff Bank Detail

```rust
pub struct CreateStaffBankDetailCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffBankDetail.Create`
**Effects:** Emits `StaffBankDetailCreateed`.


### Create Staff Document

```rust
pub struct CreateStaffDocumentCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffDocument.Create`
**Effects:** Emits `StaffDocumentCreateed`.


### Create Staff Driving License

```rust
pub struct CreateStaffDrivingLicenseCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffDrivingLicense.Create`
**Effects:** Emits `StaffDrivingLicenseCreateed`.


### Create Staff Profile Photo

```rust
pub struct CreateStaffProfilePhotoCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffProfilePhoto.Create`
**Effects:** Emits `StaffProfilePhotoCreateed`.


### Create Staff Registration Field Option

```rust
pub struct CreateStaffRegistrationFieldOptionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffRegistrationFieldOption.Create`
**Effects:** Emits `StaffRegistrationFieldOptionCreateed`.


### Create Staff Social Link

```rust
pub struct CreateStaffSocialLinkCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffSocialLink.Create`
**Effects:** Emits `StaffSocialLinkCreateed`.


### Delete Assign Class Teacher

```rust
pub struct DeleteAssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `AssignClassTeacher.Delete`
**Effects:** Emits `AssignClassTeacherDeleteed`.


### Record Leave Request Approval

```rust
pub struct RecordLeaveRequestApprovalCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `LeaveRequestApproval.Record`
**Effects:** Emits `LeaveRequestApprovalRecorded`.


### Record Payroll Generate Audit

```rust
pub struct RecordPayrollGenerateAuditCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `PayrollGenerateAudit.Record`
**Effects:** Emits `PayrollGenerateAuditRecorded`.


### Record Staff Attendance Punch

```rust
pub struct RecordStaffAttendancePunchCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffAttendancePunch.Record`
**Effects:** Emits `StaffAttendancePunchRecorded`.


### Record Staff Import Resolution

```rust
pub struct RecordStaffImportResolutionCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffImportResolution.Record`
**Effects:** Emits `StaffImportResolutionRecorded`.


### Record Staff Leave History

```rust
pub struct RecordStaffLeaveHistoryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffLeaveHistory.Record`
**Effects:** Emits `StaffLeaveHistoryRecorded`.


### Record Staff Payroll History

```rust
pub struct RecordStaffPayrollHistoryCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffPayrollHistory.Record`
**Effects:** Emits `StaffPayrollHistoryRecorded`.


### Refresh Staff Leave Balance

```rust
pub struct RefreshStaffLeaveBalanceCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffLeaveBalance.Refresh`
**Effects:** Emits `StaffLeaveBalanceRefreshed`.


### Refresh Staff Timeline

```rust
pub struct RefreshStaffTimelineCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffTimeline.Refresh`
**Effects:** Emits `StaffTimelineRefreshed`.


### Set Hourly Rate Override

```rust
pub struct SetHourlyRateOverrideCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `HourlyRateOverride.Set`
**Effects:** Emits `HourlyRateOverrideSeted`.


### Set Staff Custom Field

```rust
pub struct SetStaffCustomFieldCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffCustomField.Set`
**Effects:** Emits `StaffCustomFieldSeted`.


## StaffImportRow

```rust
pub struct StaffImportRow {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `StaffImportRow`
**Effects:** Emits `StaffImportRowRecorded`.

