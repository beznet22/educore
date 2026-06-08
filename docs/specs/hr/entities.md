# HR Domain — Entities

Entities have identity and lifecycle but are not aggregate roots.
They are loaded and persisted only through their aggregate root.

## StaffBankDetail

**Identity:** `StaffBankDetailId(SchoolId, Uuid)`
**Owner:** `Staff`

The bank information of a staff member. Has `BankAccountName`,
`BankAccountNumber`, `BankName`, `BankBranch`. Used for payroll
disbursement.

## StaffAddress

**Identity:** `StaffAddressId(SchoolId, Uuid)`
**Owner:** `Staff`

The current and permanent addresses of a staff member. Has
`CurrentAddress`, `PermanentAddress`.

## StaffFamilyMember

**Identity:** `StaffFamilyMemberId(SchoolId, Uuid)`
**Owner:** `Staff`

A family member of a staff member. Has `Name`, `Relation`,
`DateOfBirth`, `Occupation`. (Schema-level storage is via the
`fathers_name` and `mothers_name` text fields; the entity is the
typed projection.)

## StaffSocialLink

**Identity:** `StaffSocialLinkId(SchoolId, Uuid)`
**Owner:** `Staff`

A social profile link of a staff member. Has `Platform` (Facebook,
Twitter, LinkedIn, Instagram), `Url`. Backed by the four URL fields
in the staff row.

## StaffDocument

**Identity:** `StaffDocumentId(SchoolId, Uuid)`
**Owner:** `Staff`

A document uploaded against a staff member. Has `Title`,
`FileReference`, `DocumentType` (JoiningLetter, Resume,
OtherDocument, DrivingLicense).

## StaffTimeline

**Identity:** `StaffTimelineId(SchoolId, Uuid)`
**Owner:** `Staff`

A timeline entry that may be visible to the staff member. Has
`Title`, `Date`, `Description`, optional `File`, and
`VisibleToStaff` flag.

## StaffCustomField

**Identity:** `StaffCustomFieldId(SchoolId, Uuid)`
**Owner:** `Staff`

A user-defined field on a staff profile. Has `Key`, `Value`,
`FormName`.

## StaffLeaveBalance

**Identity:** `StaffLeaveBalanceId(SchoolId, Uuid)`
**Owner:** `Staff`

The per-type leave balance for a staff member in a given academic
year. Has `LeaveDefineId`, `RemainingDays`, `UsedDays`. Derived
from `LeaveDefine` minus approved `LeaveRequest` totals; the engine
maintains this as a cached projection.

## LeaveRequestApproval

**Identity:** `LeaveRequestApprovalId(SchoolId, Uuid)`
**Owner:** `LeaveRequest`

The approval state of a leave request. Has `ApproverId`,
`Decision`, `DecisionAt`, `Note`.

## PayrollEarnDeducType

**Identity:** `PayrollEarnDeducTypeId(SchoolId, Uuid)`
**Owner:** `SalaryTemplate`

A named earning or deduction type (e.g. "Basic", "HRA", "PF",
"Loan"). Has `Name`, `Computation` (Fixed, Percentage, Formula),
`Value`.

## PayrollPaymentLink

**Identity:** `PayrollPaymentLinkId(SchoolId, Uuid)`
**Owner:** `PayrollGenerate` (read by finance)

A read-only link from a payroll to the corresponding
`PayrollPayment` row in the finance domain. Has `PayrollPaymentId`,
`Amount`, `PaymentDate`, `PaymentMethod`. Created by the finance
domain in response to a `PayrollPaid` event; the HR domain reads
it for reporting.

## StaffAttendancePromotion

**Identity:** `StaffAttendancePromotionId(SchoolId, Uuid)`
**Owner:** `StaffAttendanceImport`

The promotion state of a bulk attendance row. Has `PromotedAt`,
`PromotedStaffAttendanceId`, `Result` (Success, Failed), `Reason`.

## StaffImportResolution

**Identity:** `StaffImportResolutionId(SchoolId, Uuid)`
**Owner:** `StaffImportBulkTemporary`

The resolution state of a bulk staff import row. Has `ResolvedAt`,
`ResolvedStaffId` (when promoted), `RejectionReason` (when
rejected), `ValidationErrors`.

## StaffNote

**Identity:** `StaffNoteId(SchoolId, Uuid)`
**Owner:** `Staff`

A free-text note attached to a staff profile. Has `AuthorId`,
`CreatedAt`, `Body`, `Visible`.

## StaffPayrollHistory

**Identity:** `StaffPayrollHistoryId(SchoolId, Uuid)`
**Owner:** `Staff`

A read-only history row of a staff's payroll. Has `PayrollGenerateId`,
`Period`, `NetSalary`, `PaidAt`, `Status`.

## StaffLeaveHistory

**Identity:** `StaffLeaveHistoryId(SchoolId, Uuid)`
**Owner:** `Staff`

A read-only history row of a staff's leave usage. Has `LeaveRequestId`,
`Period`, `TypeId`, `Days`, `Status`.

## AssignClassTeacherScope

**Identity:** `AssignClassTeacherScopeId(SchoolId, Uuid)`
**Owner:** `AssignClassTeacher`

The scope of the assignment. Has `ClassId`, `SectionId`. The
school-level assignment may cover multiple class-sections; this
entity represents one scope row.

## DepartmentHead

**Identity:** `DepartmentHeadId(SchoolId, Uuid)`
**Owner:** `Department`

The staff member who heads a department. Has `StaffId`,
`EffectiveFrom`, `EffectiveTo`.

## DesignationGrade

**Identity:** `DesignationGradeId(SchoolId, Uuid)`
**Owner:** `Designation`

The salary grade bound to a designation. Has `SalaryGrade`,
`SalaryTemplateId`, `EffectiveFrom`.

## HourlyRateOverride

**Identity:** `HourlyRateOverrideId(SchoolId, Uuid)`
**Owner:** `HourlyRate`

A per-staff override of an hourly rate. Has `StaffId`, `Rate`,
`EffectiveFrom`, `EffectiveTo`.

## LeaveDefineAdjustment

**Identity:** `LeaveDefineAdjustmentId(SchoolId, Uuid)`
**Owner:** `LeaveDefine`

An adjustment to a leave entitlement (e.g. a bonus day). Has
`Delta`, `Reason`, `AdjustedAt`, `AdjustedBy`.

## LeaveRequestAttachment

**Identity:** `LeaveRequestAttachmentId(SchoolId, Uuid)`
**Owner:** `LeaveRequest`

A medical or supporting document attached to a leave request. Has
`File`, `UploadedAt`, `Type`.

## StaffAttendancePunch

**Identity:** `StaffAttendancePunchId(SchoolId, Uuid)`
**Owner:** `StaffAttendance`

A punch (in or out) recorded for the attendance row. Has
`PunchType` (In, Out), `PunchTime`, `Source` (Manual, Biometric,
RFID, Mobile).

## PayrollGenerateAudit

**Identity:** `PayrollGenerateAuditId(SchoolId, Uuid)`
**Owner:** `PayrollGenerate`

An audit row of a state change on a payroll. Has `FromStatus`,
`ToStatus`, `ActorId`, `At`, `Note`.

## StaffRoleAssignment

**Identity:** `StaffRoleAssignmentId(SchoolId, Uuid)`
**Owner:** `Staff`

The current role binding of a staff member. Has `RoleId`,
`EffectiveFrom`, `PreviousRoleId`. The platform's RBAC port
mirrors this binding.

## StaffProfilePhoto

**Identity:** `StaffProfilePhotoId(SchoolId, Uuid)`
**Owner:** `Staff`

The stored profile photo of a staff member. Has `FileReference`,
`UploadedAt`.

## StaffDrivingLicense

**Identity:** `StaffDrivingLicenseId(SchoolId, Uuid)`
**Owner:** `Staff`

The driving license of a staff member (used for transport staff).
Has `Number`, `ExpiryDate`, `File`.

## StaffRegistrationFieldOption

**Identity:** `StaffRegistrationFieldOptionId(SchoolId, Uuid)`
**Owner:** `StaffRegistrationField`

A fixed-option list for a registration field (e.g. dropdown). Has
`Value`, `Label`, `Position`.

## BulkImportJob

**Identity:** `BulkImportJobId(SchoolId, Uuid)`
**Owner:** `School` (HR)

A bulk import job. Has `Source`, `File`, `StartedAt`, `CompletedAt`,
`Total`, `Succeeded`, `Failed`, `Status`. Each row of the import
is a `StaffImportBulkTemporary`.

## StaffAttendanceImportBatch

**Identity:** `StaffAttendanceImportBatchId(SchoolId, Uuid)`
**Owner:** `StaffAttendanceImport`

A logical batch of bulk attendance rows. Has `Source`, `File`,
`ImportedAt`, `ImportedBy`.
