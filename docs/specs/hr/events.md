# HR Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Staff Lifecycle

### StaffRegistered

```rust
pub struct StaffRegistered {
    pub staff_id: StaffId,
    pub staff_no: Option<StaffNo>,
    pub full_name: FullName,
    pub email: Option<EmailAddress>,
    pub mobile: Option<PhoneNumber>,
    pub designation_id: DesignationId,
    pub department_id: DepartmentId,
    pub role_id: RoleId,
    pub user_id: UserId,
    pub date_of_joining: NaiveDate,
}
```

**Subscribers:**
- `rbac` — assign the role to the platform user.
- `finance` — bind the staff's salary template or hourly rate.
- `communication` — send a welcome message to the staff.

### StaffUpdated

```rust
pub struct StaffUpdated {
    pub staff_id: StaffId,
    pub changed_fields: Vec<&'static str>,
}
```

### StaffDepartmentChanged

```rust
pub struct StaffDepartmentChanged {
    pub staff_id: StaffId,
    pub from_department_id: Option<DepartmentId>,
    pub to_department_id: DepartmentId,
    pub effective_from: NaiveDate,
}
```

- `StaffDesignationChanged { staff_id, from, to, effective_from }`
- `StaffRoleChanged { staff_id, from_role_id, to_role_id, effective_from }`

### StaffSuspended

```rust
pub struct StaffSuspended {
    pub staff_id: StaffId,
    pub reason: SuspensionReason,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
}
```

- `StaffReinstated { staff_id, effective_from }`
- `StaffResigned { staff_id, reason, effective_from }`
- `StaffTerminated { staff_id, reason, effective_from }`
- `StaffRetired { staff_id, effective_from }`
- `StaffDeleted { staff_id, reason }`

## Departments

- `DepartmentCreated { department_id, name }`
- `DepartmentUpdated { department_id, changes }`
- `DepartmentDeleted { department_id }`

## Designations

- `DesignationCreated { designation_id, title }`
- `DesignationUpdated { designation_id, changes }`
- `DesignationDeleted { designation_id }`

## Class Teacher

### ClassTeacherAssigned

```rust
pub struct ClassTeacherAssigned {
    pub assign_class_teacher_id: AssignClassTeacherId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub staff_id: StaffId,
    pub academic_id: AcademicYearId,
}
```

- `AssignClassTeacherUpdated { assign_class_teacher_id, changes }`
- `AssignClassTeacherDeleted { assign_class_teacher_id }`
- `SubjectTeacherAssigned { class_id, section_id, subject_id, staff_id, academic_id }`

## Leave

### LeaveTypeCreated

```rust
pub struct LeaveTypeCreated {
    pub leave_type_id: LeaveTypeId,
    pub type_name: LeaveTypeName,
    pub total_days: LeaveTotalDays,
}
```

- `LeaveTypeUpdated { leave_type_id, changes }`
- `LeaveTypeDeleted { leave_type_id }`

### LeavePolicyDefined

```rust
pub struct LeavePolicyDefined {
    pub leave_define_id: LeaveDefineId,
    pub role_id: Option<RoleId>,
    pub user_id: Option<UserId>,
    pub type_id: LeaveTypeId,
    pub days: LeaveDays,
    pub total_days: LeaveTotalDays,
    pub academic_id: AcademicYearId,
}
```

- `LeavePolicyUpdated { leave_define_id, changes }`
- `LeavePolicyDeleted { leave_define_id }`

### LeaveRequested

```rust
pub struct LeaveRequested {
    pub leave_request_id: LeaveRequestId,
    pub staff_id: StaffId,
    pub type_id: LeaveTypeId,
    pub apply_date: NaiveDate,
    pub leave_from: LeaveFrom,
    pub leave_to: LeaveTo,
    pub reason: LeaveReason,
}
```

### LeaveApproved

```rust
pub struct LeaveApproved {
    pub leave_request_id: LeaveRequestId,
    pub approver_id: UserId,
    pub approved_at: Timestamp,
    pub note: Option<String>,
}
```

**Subscribers:**
- `finance` — when payroll is generated, the approved leave
  reduces the available balance and the extra-leave deduction is
  computed.

- `LeaveRejected { leave_request_id, rejecter_id, reason, rejected_at }`
- `LeaveCancelled { leave_request_id, canceller_id, reason, cancelled_at }`

## Attendance

### StaffAttendanceMarked

```rust
pub struct StaffAttendanceMarked {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: AttendanceDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<InTime>,
    pub out_time: Option<OutTime>,
    pub source: AttendanceSource,
}
```

- `StaffAttendanceUpdated { staff_attendance_id, changes }`
- `StaffAttendanceDeleted { staff_attendance_id }`
- `StaffAttendanceImported { import_id, staff_id, attendance_date, attendance_type }`
- `StaffAttendancePromoted { import_id, staff_attendance_id }`
- `StaffAttendanceImportRejected { import_id, reason }`

## Payroll

### PayrollGenerated

```rust
pub struct PayrollGenerated {
    pub payroll_generate_id: PayrollGenerateId,
    pub staff_id: StaffId,
    pub pay_period: PayPeriod,
    pub basic_salary: BasicSalary,
    pub total_earning: TotalEarning,
    pub total_deduction: TotalDeduction,
    pub tax: Tax,
    pub gross_salary: GrossSalary,
    pub net_salary: NetSalary,
    pub bank_id: Option<BankAccountId>,
    pub payment_mode: Option<PaymentMethodId>,
    pub note: Option<String>,
}
```

**Subscribers:**
- `finance` — read the payroll and queue it for payment.

- `PayrollAmountsUpdated { payroll_generate_id, changes }`
- `PayrollApproved { payroll_generate_id, approver_id, approved_at }`
- `PayrollPaid { payroll_generate_id, paid_amount, paid_at }`

### PayrollEarningAdded

```rust
pub struct PayrollEarningAdded {
    pub payroll_earn_deduc_id: PayrollEarnDeducId,
    pub payroll_generate_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: Amount,
}
```

- `PayrollDeductionAdded { payroll_earn_deduc_id, payroll_generate_id, type_name, amount }`
- `PayrollEarnDeducUpdated { payroll_earn_deduc_id, changes }`
- `PayrollEarnDeducDeleted { payroll_earn_deduc_id }`

### LeaveDeductionInfoAdded

```rust
pub struct LeaveDeductionInfoAdded {
    pub leave_deduction_info_id: LeaveDeductionInfoId,
    pub staff_id: StaffId,
    pub payroll_id: PayrollGenerateId,
    pub extra_leave: u32,
    pub salary_deduct: SalaryDeduct,
    pub pay_month: PayrollMonth,
    pub pay_year: PayrollYear,
}
```

- `LeaveDeductionInfoUpdated { leave_deduction_info_id, changes }`
- `LeaveDeductionInfoDeleted { leave_deduction_info_id }`

## Salary & Rate

- `SalaryTemplateCreated { salary_template_id, grade, basic, gross, net }`
- `SalaryTemplateUpdated { salary_template_id, changes }`
- `SalaryTemplateDeleted { salary_template_id }`
- `HourlyRateSet { hourly_rate_id, grade, rate }`
- `HourlyRateUpdated { hourly_rate_id, changes }`
- `HourlyRateDeleted { hourly_rate_id }`

## Registration Field

- `StaffRegistrationFieldCreated { id, field_name, label_name, is_required }`
- `StaffRegistrationFieldUpdated { id, changes }`
- `StaffRegistrationFieldDeleted { id }`

## Bulk Import

- `StaffBulkImported { bulk_import_job_id, total_rows, source }`
- `StaffImportPromoted { staff_import_bulk_temporary_id, staff_id }`
- `StaffImportRejected { staff_import_bulk_temporary_id, reason }`
