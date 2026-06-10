# HR Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the HR domain are typed and tenant-scoped. The
generic `Id<S, T>` wrapper carries the `SchoolId` of the owning
school and the local id.

| Identifier                          | Backing Type                | Notes                              |
| ----------------------------------- | --------------------------- | ---------------------------------- |
| `StaffId`                           | `Id<Staff>`                 | A staff member                     |
| `DepartmentId`                      | `Id<Department>`            | A department                       |
| `DesignationId`                     | `Id<Designation>`           | A designation                      |
| `LeaveTypeId`                       | `Id<LeaveType>`             | A leave type                       |
| `LeaveDefineId`                     | `Id<LeaveDefine>`           | A leave entitlement                |
| `LeaveRequestId`                    | `Id<LeaveRequest>`          | A leave request                    |
| `StaffAttendanceId`                 | `Id<StaffAttendance>`       | A daily attendance row             |
| `StaffAttendanceImportId`           | `Id<StaffAttendanceImport>` | A bulk attendance import row       |
| `AssignClassTeacherId`              | `Id<AssignClassTeacher>`    | A class teacher assignment         |
| `HourlyRateId`                      | `Id<HourlyRate>`            | An hourly rate                     |
| `SalaryTemplateId`                  | `Id<SalaryTemplate>`        | A salary grade template            |
| `PayrollGenerateId`                 | `Id<PayrollGenerate>`       | A monthly payroll run              |
| `PayrollEarnDeducId`                | `Id<PayrollEarnDeduc>`      | A payroll earnings/deductions line |
| `LeaveDeductionInfoId`              | `Id<LeaveDeductionInfo>`    | A leave deduction row              |
| `StaffRegistrationFieldId`          | `Id<StaffRegistrationField>`| A registration custom field        |
| `StaffImportBulkTemporaryId`        | `Id<StaffImportBulkTemporary>` | A bulk import staging row       |
| `StaffBankDetailId`                 | `Id<StaffBankDetail>`       | A staff bank detail (entity)       |
| `StaffAddressId`                    | `Id<StaffAddress>`          | A staff address (entity)           |
| `StaffSocialLinkId`                 | `Id<StaffSocialLink>`       | A staff social link (entity)       |
| `StaffDocumentId`                   | `Id<StaffDocument>`         | A staff document (entity)          |
| `StaffTimelineId`                   | `Id<StaffTimeline>`         | A staff timeline (entity)          |
| `StaffCustomFieldId`                | `Id<StaffCustomField>`      | A staff custom field (entity)      |
| `StaffLeaveBalanceId`               | `Id<StaffLeaveBalance>`     | A staff leave balance (entity)     |
| `LeaveRequestApprovalId`            | `Id<LeaveRequestApproval>`  | A leave approval (entity)          |
| `PayrollPaymentLinkId`              | `Id<PayrollPaymentLink>`    | A payroll-payment link (entity)    |
| `StaffAttendancePromotionId`        | `Id<StaffAttendancePromotion>` | A bulk attendance promotion row |
| `StaffImportResolutionId`           | `Id<StaffImportResolution>` | A bulk import resolution (entity)  |
| `StaffNoteId`                       | `Id<StaffNote>`             | A staff note (entity)              |
| `StaffPayrollHistoryId`             | `Id<StaffPayrollHistory>`   | A staff payroll history (entity)   |
| `StaffLeaveHistoryId`               | `Id<StaffLeaveHistory>`     | A staff leave history (entity)     |
| `AssignClassTeacherScopeId`         | `Id<AssignClassTeacherScope>`| An assignment scope (entity)      |
| `DepartmentHeadId`                  | `Id<DepartmentHead>`        | A department head (entity)         |
| `DesignationGradeId`                | `Id<DesignationGrade>`      | A designation grade (entity)       |
| `HourlyRateOverrideId`              | `Id<HourlyRateOverride>`    | An hourly rate override (entity)   |
| `LeaveDefineAdjustmentId`           | `Id<LeaveDefineAdjustment>` | A leave define adjustment (entity) |
| `LeaveRequestAttachmentId`          | `Id<LeaveRequestAttachment>`| A leave attachment (entity)        |
| `StaffAttendancePunchId`            | `Id<StaffAttendancePunch>`  | A staff punch (entity)             |
| `PayrollGenerateAuditId`            | `Id<PayrollGenerateAudit>`  | A payroll audit (entity)           |
| `StaffRoleAssignmentId`             | `Id<StaffRoleAssignment>`   | A staff role binding (entity)      |
| `StaffProfilePhotoId`               | `Id<StaffProfilePhoto>`     | A staff photo (entity)             |
| `StaffDrivingLicenseId`             | `Id<StaffDrivingLicense>`   | A staff driving license (entity)   |
| `StaffRegistrationFieldOptionId`    | `Id<StaffRegistrationFieldOption>` | A registration field option |
| `BulkImportJobId`                   | `Id<BulkImportJob>`         | A bulk import job (entity)         |
| `StaffAttendanceImportBatchId`      | `Id<StaffAttendanceImportBatch>` | A bulk attendance batch (entity)|

## Names & Identity

| Type                | Constraints                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `StaffNo`           | `u32` per school                                                 |
| `EmployeeId`        | 1..50 chars, unique within school                                |
| `PersonName`        | 1..100 chars, unicode letters and basic punctuation allowed     |
| `FullName`          | Computed from `PersonName` parts                                |
| `EmailAddress`      | RFC 5322 with length cap 200                                    |
| `PhoneNumber`       | E.164 format preferred; alternative national formats accepted    |
| `Address`           | 1..500 chars                                                     |
| `Occupation`        | 1..200 chars                                                     |
| `Qualification`     | 1..200 chars                                                     |
| `Experience`        | 0..200 chars (free text or years)                                |
| `MaritalStatus`     | `Single`, `Married`, `Divorced`, `Widowed`                       |
| `Gender`            | `Male`, `Female`, `Other`                                        |
| `BloodGroup`        | `A+`, `A-`, `B+`, `B-`, `AB+`, `AB-`, `O+`, `O-`                  |
| `Religion`          | 1..100 chars                                                     |
| `Caste`             | 1..100 chars                                                     |
| `ContractType`      | `Permanent`, `Temporary`, `Contract`, `Probation`, `Intern`       |
| `Location`          | 1..50 chars                                                      |
| `EpfNo`             | 0..20 chars                                                      |

## Salary & Money

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `Salary`             | `Amount` (typed)                                                 |
| `BasicSalary`        | `Amount`                                                          |
| `HouseRent`          | `Amount`                                                          |
| `ProvidentFund`      | `Amount`                                                          |
| `GrossSalary`        | `Amount`                                                          |
| `NetSalary`          | `Amount`                                                          |
| `TotalEarning`       | `Amount`                                                          |
| `TotalDeduction`     | `Amount`                                                          |
| `Tax`                | `Amount`                                                          |
| `PaidAmount`         | `Amount`                                                          |
| `SalaryGrade`        | 1..200 chars                                                      |
| `OvertimeRate`       | `Amount`                                                          |
| `HourlyRate`         | `Amount` (typed)                                                 |
| `SalaryDeduct`       | `Amount`                                                          |
| `ExtraLeave`         | `u32`                                                             |
| `PayPeriod`          | `(Month, Year)` with `Month in 1..=12`                           |
| `PayrollMonth`       | `u8` in 1..=12                                                    |
| `PayrollYear`        | `u16` in 1900..=9999                                              |
| `EarnDeducType`      | `Earning`, `Deduction` (encoded `e` / `d` in storage)             |

## Leave

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `LeaveTypeName`      | 1..200 chars                                                      |
| `LeaveDays`          | `u32` in 0..=365                                                  |
| `LeaveTotalDays`     | `u32` in 0..=365                                                  |
| `LeaveFrom`          | `NaiveDate`                                                       |
| `LeaveTo`            | `NaiveDate`                                                       |
| `LeaveReason`        | 0..2000 chars                                                     |
| `LeaveNote`          | 0..2000 chars                                                     |
| `LeaveAttachment`    | `FileReference`                                                   |
| `LeaveStatus`        | `Pending`, `Approved`, `Rejected`, `Cancelled`                    |
| `LeaveDecision`      | `Approve`, `Reject`                                               |

## Attendance

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `AttendanceType`     | `Present`, `Late`, `Absent`, `Holiday`, `HalfDay` (encoded `P`, `L`, `A`, `H`, `F`) |
| `AttendanceDate`     | `NaiveDate`                                                       |
| `AttendanceNote`     | 0..500 chars                                                      |
| `InTime`             | `String` (raw source format)                                     |
| `OutTime`            | `String` (raw source format)                                     |
| `AttendanceSource`   | `Manual`, `Biometric`, `Rfid`, `Mobile`, `Import`                 |

## Dates

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `DateOfBirth`         | `NaiveDate`, must result in age between 18 and 80           |
| `DateOfJoining`       | `NaiveDate`                                                  |
| `DrivingLicenseExpiry`| `NaiveDate`                                                  |

## Status Enums

| Type                | Values                                                                              |
| ------------------- | ----------------------------------------------------------------------------------- |
| `StaffStatus`       | `Active`, `Suspended`, `Resigned`, `Terminated`, `Retired`                          |
| `DepartmentStatus`  | `Active`, `Inactive`                                                                |
| `DesignationStatus` | `Active`, `Inactive`                                                                |
| `PayrollStatus`     | `NotGenerated`, `Generated`, `Paid`                                                |
| `LeaveTypeStatus`   | `Active`, `Inactive`                                                                |
| `LeaveDefineStatus` | `Active`, `Inactive`                                                                |
| `RequiredType`      | `On`, `Off` (encoded `1`, `2` in storage)                                          |
| `RegistrationType`  | `Student`, `Staff`                                                                  |

## School Identity Bindings

| Type                  | Notes                                                       |
| --------------------- | ----------------------------------------------------------- |
| `SchoolId`            | From `educore-platform`                                     |
| `TenantContext`       | `(SchoolId, UserId, ...)` from `educore-platform`           |
| `UserId`              | From `educore-platform`                                     |
| `RoleId`              | From `educore-rbac`                                         |
| `ClassId`             | From `educore-academic`                                     |
| `SectionId`           | From `educore-academic`                                     |
| `SubjectId`           | From `educore-academic`                                     |
| `AcademicYearId`      | From `educore-academic`                                     |
| `BankAccountId`       | From `educore-finance`                                      |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let email = EmailAddress::parse("ada@example.com")?;
let period = PayPeriod::new(6, 2026)?;
let rate = HourlyRate::new(Amount::new(Money::USD, 2500)?)?;
```

Parsing returns `Result<T, ValueError>`. There are no setters that
bypass validation.
