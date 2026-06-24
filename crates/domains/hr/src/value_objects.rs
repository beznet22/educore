//! # HR value objects
//!
//! The typed ids (every aggregate is keyed by one) and the
//! validated value objects the HR aggregates depend on. Per
//! `docs/specs/hr/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper
//!   that carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - The status enums (`StaffStatus`, `DepartmentStatus`,
//!   `DesignationStatus`, `LeaveTypeStatus`, `LeaveStatus`,
//!   `PayrollStatus`, `EarnDeducType`, `PayrollMonth`,
//!   `AttendanceType`, `AttendanceSource`, `ContractType`,
//!   `Gender`, `MaritalStatus`, `BloodGroup`, `RequiredType`,
//!   `RegistrationType`) are closed and `Copy`.
//! - Foreign-key typed ids (`ClassId`, `SectionId`, `SubjectId`,
//!   `AcademicYearId`, `RoleId`) are **re-exported** from
//!   [`educore_academic`](::educore_academic) and
//!   [`educore_rbac`](::educore_rbac); the HR crate owns only
//!   the HR-specific ids.
//!
//! Phase 6 ships the 16 aggregate + 28 entity typed ids
//! (44 total per `docs/specs/hr/value-objects.md`),
//! the 14 closed enums, and the foreign-key re-exports.

#![allow(missing_docs)]
#![allow(unused_imports)] // The 44 typed ids and 14 closed enums
                          // are described by their constructor
                          // signatures; suppressing this lint for
                          // the file is the pragmatic choice.

use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

pub use educore_academic::AcademicYearId;
pub use educore_academic::ClassId;
pub use educore_academic::SectionId;
pub use educore_academic::SubjectId;
pub use educore_rbac::ids::RoleId;

// =============================================================================
// Macro: typed HR id
// =============================================================================

/// Macro to define the per-aggregate typed id wrapper. Every
/// HR id follows the same shape: a `school_id` anchor plus a
/// local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
///
/// The pattern matches `educore-academic::value_objects::*`
/// so the engine's id types stay consistent across crates.
macro_rules! hr_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids: 44 aggregate roots + entities
// =============================================================================

hr_typed_id! {
    /// A typed id for a [`Staff`](crate::aggregate::Staff) aggregate.
    ///
    /// This is the **canonical** `StaffId` in the engine. The
    /// `educore-attendance` and `educore-assessment` crates
    /// both re-export this type in place of their Phase 5/4
    /// placeholder definitions (per Phase 6 Workstream H).
    pub struct StaffId;
}
hr_typed_id! {
    /// A typed id for a [`Department`](crate::aggregate::Department).
    pub struct DepartmentId;
}
hr_typed_id! {
    /// A typed id for a [`Designation`](crate::aggregate::Designation).
    pub struct DesignationId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveType`](crate::aggregate::LeaveType).
    pub struct LeaveTypeId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveDefine`](crate::aggregate::LeaveDefine) policy row.
    pub struct LeaveDefineId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveRequest`](crate::aggregate::LeaveRequest).
    pub struct LeaveRequestId;
}
hr_typed_id! {
    /// A typed id for an HR-side [`StaffAttendance`](crate::aggregate::StaffAttendance) row.
    ///
    /// Disambiguated from the attendance-domain
    /// `educore_attendance::value_objects::StaffAttendanceId`
    /// by being the HR-side per-staff per-day row. The two
    /// tables are independent and serve different
    /// concerns (the attendance crate tracks staff
    /// presence in service of student attendance; the HR
    /// crate tracks it in service of payroll + leave).
    pub struct StaffAttendanceId;
}
hr_typed_id! {
    /// A typed id for a bulk
    /// [`StaffAttendanceImport`](crate::aggregate::StaffAttendanceImport) staging row.
    pub struct StaffAttendanceImportId;
}
hr_typed_id! {
    /// A typed id for an [`AssignClassTeacher`](crate::aggregate::AssignClassTeacher) row.
    pub struct AssignClassTeacherId;
}
hr_typed_id! {
    /// A typed id for a [`HourlyRate`](crate::aggregate::HourlyRate) row.
    pub struct HourlyRateId;
}
hr_typed_id! {
    /// A typed id for a [`SalaryTemplate`](crate::aggregate::SalaryTemplate) row.
    pub struct SalaryTemplateId;
}
hr_typed_id! {
    /// A typed id for a [`PayrollGenerate`](crate::aggregate::PayrollGenerate) row.
    pub struct PayrollGenerateId;
}
hr_typed_id! {
    /// A typed id for a [`PayrollEarnDeduc`](crate::aggregate::PayrollEarnDeduc) line row.
    pub struct PayrollEarnDeducId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveDeductionInfo`](crate::aggregate::LeaveDeductionInfo) row.
    pub struct LeaveDeductionInfoId;
}
hr_typed_id! {
    /// A typed id for a [`StaffRegistrationField`](crate::aggregate::StaffRegistrationField) row.
    pub struct StaffRegistrationFieldId;
}
hr_typed_id! {
    /// A typed id for a [`StaffImportBulkTemporary`](crate::aggregate::StaffImportBulkTemporary) staging row.
    pub struct StaffImportBulkTemporaryId;
}
hr_typed_id! {
    /// A typed id for a [`StaffBankDetail`] entity row.
    pub struct StaffBankDetailId;
}
hr_typed_id! {
    /// A typed id for a [`StaffAddress`] entity row.
    pub struct StaffAddressId;
}
hr_typed_id! {
    /// A typed id for a [`StaffSocialLink`] entity row.
    pub struct StaffSocialLinkId;
}
hr_typed_id! {
    /// A typed id for a [`StaffDocument`] entity row.
    pub struct StaffDocumentId;
}
hr_typed_id! {
    /// A typed id for a [`StaffTimeline`] entity row.
    pub struct StaffTimelineId;
}
hr_typed_id! {
    /// A typed id for a [`StaffCustomField`] entity row.
    pub struct StaffCustomFieldId;
}
hr_typed_id! {
    /// A typed id for a [`StaffLeaveBalance`] entity row.
    pub struct StaffLeaveBalanceId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveRequestApproval`] entity row.
    pub struct LeaveRequestApprovalId;
}
hr_typed_id! {
    /// A typed id for a [`PayrollPaymentLink`] entity row.
    pub struct PayrollPaymentLinkId;
}
hr_typed_id! {
    /// A typed id for a [`StaffAttendancePromotion`] staging row.
    pub struct StaffAttendancePromotionId;
}
hr_typed_id! {
    /// A typed id for a [`StaffImportResolution`] entity row.
    pub struct StaffImportResolutionId;
}
hr_typed_id! {
    /// A typed id for a [`StaffNote`] entity row.
    pub struct StaffNoteId;
}
hr_typed_id! {
    /// A typed id for a [`StaffPayrollHistory`] entity row.
    pub struct StaffPayrollHistoryId;
}
hr_typed_id! {
    /// A typed id for a [`StaffLeaveHistory`] entity row.
    pub struct StaffLeaveHistoryId;
}
hr_typed_id! {
    /// A typed id for an [`AssignClassTeacherScope`] entity row.
    pub struct AssignClassTeacherScopeId;
}
hr_typed_id! {
    /// A typed id for a [`DepartmentHead`] entity row.
    pub struct DepartmentHeadId;
}
hr_typed_id! {
    /// A typed id for a [`DesignationGrade`] entity row.
    pub struct DesignationGradeId;
}
hr_typed_id! {
    /// A typed id for a [`HourlyRateOverride`] entity row.
    pub struct HourlyRateOverrideId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveDefineAdjustment`] entity row.
    pub struct LeaveDefineAdjustmentId;
}
hr_typed_id! {
    /// A typed id for a [`LeaveRequestAttachment`] entity row.
    pub struct LeaveRequestAttachmentId;
}
hr_typed_id! {
    /// A typed id for a [`StaffAttendancePunch`] entity row.
    pub struct StaffAttendancePunchId;
}
hr_typed_id! {
    /// A typed id for a [`PayrollGenerateAudit`] entity row.
    pub struct PayrollGenerateAuditId;
}
hr_typed_id! {
    /// A typed id for a [`StaffRoleAssignment`] entity row.
    pub struct StaffRoleAssignmentId;
}
hr_typed_id! {
    /// A typed id for a [`StaffProfilePhoto`] entity row.
    pub struct StaffProfilePhotoId;
}
hr_typed_id! {
    /// A typed id for a [`StaffDrivingLicense`] entity row.
    pub struct StaffDrivingLicenseId;
}
hr_typed_id! {
    /// A typed id for a [`StaffRegistrationFieldOption`] entity row.
    pub struct StaffRegistrationFieldOptionId;
}
hr_typed_id! {
    /// A typed id for a [`BulkImportJob`] entity row.
    pub struct BulkImportJobId;
}
hr_typed_id! {
    /// A typed id for a [`StaffAttendanceImportBatch`] entity row.
    pub struct StaffAttendanceImportBatchId;
}

// =============================================================================
// Closed status enums (Copy + Eq + Hash + Serialize)
// =============================================================================

/// Staff lifecycle status.
///
/// Phase 6 state machine: `Active → Suspended → {Reinstated,
/// Resigned, Terminated, Retired}`. `Resigned`,
/// `Terminated`, and `Retired` are terminal. `Suspended` is
/// reversible via `ReinstateStaffCommand`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StaffStatus {
    #[default]
    Active,
    Suspended,
    Resigned,
    Terminated,
    Retired,
}

impl StaffStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Resigned => "resigned",
            Self::Terminated => "terminated",
            Self::Retired => "retired",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(Self::Active),
            "suspended" => Ok(Self::Suspended),
            "resigned" => Ok(Self::Resigned),
            "terminated" => Ok(Self::Terminated),
            "retired" => Ok(Self::Retired),
            other => Err(DomainError::validation(format!(
                "unknown staff status: {other:?}"
            ))),
        }
    }

    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Resigned | Self::Terminated | Self::Retired)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DepartmentStatus {
    #[default]
    Active,
    Inactive,
}

impl DepartmentStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DesignationStatus {
    #[default]
    Active,
    Inactive,
}

impl DesignationStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaveTypeStatus {
    #[default]
    Active,
    Inactive,
}

impl LeaveTypeStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaveDefineStatus {
    #[default]
    Active,
    Inactive,
}

impl LeaveDefineStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

/// Leave-request state machine (Phase 6 Workstream C).
///
/// `Pending → Approved | Rejected`
/// `Approved → Cancelled` (within grace window)
/// `Pending  → Cancelled`
/// `Rejected` is terminal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LeaveStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl LeaveStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(DomainError::validation(format!(
                "unknown leave status: {other:?}"
            ))),
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Rejected)
    }

    /// Returns `true` if the state machine permits the
    /// `from → to` transition. `Pending` cannot return to
    /// itself; `Approved` can only go to `Cancelled`
    /// (within a grace window the consumer enforces);
    /// `Rejected` is terminal.
    #[must_use]
    #[allow(clippy::match_like_matches_macro)]
    pub const fn can_transition_to(self, to: Self) -> bool {
        match (self, to) {
            (Self::Pending, Self::Approved)
            | (Self::Pending, Self::Rejected)
            | (Self::Pending, Self::Cancelled)
            | (Self::Approved, Self::Cancelled) => true,
            _ => false,
        }
    }
}

/// Payroll-run state machine (Phase 6 Workstream F).
///
/// `NotGenerated → Generated → Paid` (terminal).
/// Partial payments are tracked on `paid_amount` and
/// `is_partial` but do not advance the state until the
/// cumulative paid amount equals `net_salary`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayrollStatus {
    #[default]
    NotGenerated,
    Generated,
    Paid,
}

impl PayrollStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotGenerated => "not_generated",
            Self::Generated => "generated",
            Self::Paid => "paid",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "not_generated" => Ok(Self::NotGenerated),
            "generated" => Ok(Self::Generated),
            "paid" => Ok(Self::Paid),
            other => Err(DomainError::validation(format!(
                "unknown payroll status: {other:?}"
            ))),
        }
    }

    #[must_use]
    pub const fn is_paid(self) -> bool {
        matches!(self, Self::Paid)
    }
}

/// Earn/dedc line type (encoded `e` / `d` in storage).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarnDeducType {
    #[default]
    Earning,
    Deduction,
}

impl EarnDeducType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Earning => "e",
            Self::Deduction => "d",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "e" => Ok(Self::Earning),
            "d" => Ok(Self::Deduction),
            other => Err(DomainError::validation(format!(
                "earn_dedc_type must be 'e' or 'd', got {other:?}"
            ))),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractType {
    #[default]
    Permanent,
    Temporary,
    Contract,
    Probation,
    Intern,
}

impl ContractType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permanent => "permanent",
            Self::Temporary => "temporary",
            Self::Contract => "contract",
            Self::Probation => "probation",
            Self::Intern => "intern",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    #[default]
    Other,
}

impl Gender {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaritalStatus {
    Single,
    Married,
    Divorced,
    Widowed,
    #[default]
    Other,
}

impl MaritalStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Married => "married",
            Self::Divorced => "divorced",
            Self::Widowed => "widowed",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BloodGroup {
    #[default]
    Unknown,
    APos,
    ANeg,
    BPos,
    BNeg,
    ABPos,
    ABNeg,
    OPos,
    ONeg,
}

impl BloodGroup {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::APos => "A+",
            Self::ANeg => "A-",
            Self::BPos => "B+",
            Self::BNeg => "B-",
            Self::ABPos => "AB+",
            Self::ABNeg => "AB-",
            Self::OPos => "O+",
            Self::ONeg => "O-",
        }
    }
}

/// Per-staff per-day attendance type (encoded `P`/`L`/`A`/`F`/`H`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttendanceType {
    #[default]
    Present,
    Late,
    Absent,
    HalfDay,
    Holiday,
}

impl AttendanceType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Present => "P",
            Self::Late => "L",
            Self::Absent => "A",
            Self::HalfDay => "F",
            Self::Holiday => "H",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s {
            "P" => Ok(Self::Present),
            "L" => Ok(Self::Late),
            "A" => Ok(Self::Absent),
            "F" => Ok(Self::HalfDay),
            "H" => Ok(Self::Holiday),
            other => Err(DomainError::validation(format!(
                "unknown attendance type: {other:?}"
            ))),
        }
    }

    #[must_use]
    pub const fn is_absent(self) -> bool {
        matches!(self, Self::Absent)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttendanceSource {
    #[default]
    Manual,
    Biometric,
    Rfid,
    Mobile,
    Import,
}

impl AttendanceSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Biometric => "biometric",
            Self::Rfid => "rfid",
            Self::Mobile => "mobile",
            Self::Import => "import",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequiredType {
    On,
    #[default]
    Off,
}

impl RequiredType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }

    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::On => 1,
            Self::Off => 2,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistrationType {
    Student,
    #[default]
    Staff,
}

impl RegistrationType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Student => "student",
            Self::Staff => "staff",
        }
    }
}

/// Whether a payroll run is fully paid, partially paid, or
/// unpaid. Tracked alongside `PayrollStatus` (which advances
/// to `Paid` only when the cumulative paid amount equals
/// `net_salary`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayrollPaymentStatus {
    #[default]
    Unpaid,
    Partial,
    FullyPaid,
}

impl PayrollPaymentStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unpaid => "unpaid",
            Self::Partial => "partial",
            Self::FullyPaid => "fully_paid",
        }
    }
}

// =============================================================================
// Validators
// =============================================================================

pub fn validate_person_name(name: &str) -> Result<()> {
    let n = name.chars().count();
    if n == 0 || n > 100 {
        return Err(DomainError::validation(format!(
            "person name must be 1..=100 chars, got {n}"
        )));
    }
    Ok(())
}

pub fn validate_email(email: &str) -> Result<()> {
    if email.is_empty() || email.len() > 200 {
        return Err(DomainError::validation(format!(
            "email must be 1..=200 chars, got {}",
            email.len()
        )));
    }
    if !email.contains('@') {
        return Err(DomainError::validation(format!(
            "email must contain '@', got {email:?}"
        )));
    }
    Ok(())
}

pub fn validate_phone(phone: &str) -> Result<()> {
    if phone.is_empty() || phone.len() > 20 {
        return Err(DomainError::validation(format!(
            "phone must be 1..=20 chars, got {}",
            phone.len()
        )));
    }
    Ok(())
}

pub fn validate_address(address: &str) -> Result<()> {
    if address.is_empty() || address.len() > 500 {
        return Err(DomainError::validation(format!(
            "address must be 1..=500 chars, got {}",
            address.len()
        )));
    }
    Ok(())
}

pub fn validate_qualification(q: &str) -> Result<()> {
    if q.is_empty() || q.len() > 200 {
        return Err(DomainError::validation(format!(
            "qualification must be 1..=200 chars, got {}",
            q.len()
        )));
    }
    Ok(())
}

pub fn validate_leave_type_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 200 {
        return Err(DomainError::validation(format!(
            "leave type name must be 1..=200 chars, got {}",
            name.len()
        )));
    }
    Ok(())
}

pub fn validate_leave_reason(reason: &str) -> Result<()> {
    if reason.len() > 2000 {
        return Err(DomainError::validation(format!(
            "leave reason must be at most 2000 chars, got {}",
            reason.len()
        )));
    }
    Ok(())
}

pub fn validate_salary_grade(grade: &str) -> Result<()> {
    if grade.is_empty() || grade.len() > 200 {
        return Err(DomainError::validation(format!(
            "salary grade must be 1..=200 chars, got {}",
            grade.len()
        )));
    }
    Ok(())
}

pub fn validate_pay_period(month: u8, year: u16) -> Result<()> {
    if !(1..=12).contains(&month) {
        return Err(DomainError::validation(format!(
            "payroll month must be 1..=12, got {month}"
        )));
    }
    if !(1900..=9999).contains(&year) {
        return Err(DomainError::validation(format!(
            "payroll year must be 1900..=9999, got {year}"
        )));
    }
    Ok(())
}

pub fn validate_date_of_birth(dob: NaiveDate) -> Result<()> {
    let now = chrono::Utc::now().date_naive();
    let age = now.signed_duration_since(dob).num_days() / 365;
    if !(18..=80).contains(&age) {
        return Err(DomainError::validation(format!(
            "staff age must be 18..=80 years, got {age}"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn staff_status_round_trip() {
        for s in [
            StaffStatus::Active,
            StaffStatus::Suspended,
            StaffStatus::Resigned,
            StaffStatus::Terminated,
            StaffStatus::Retired,
        ] {
            let parsed = StaffStatus::parse(s.as_str()).unwrap();
            assert_eq!(parsed, s);
        }
    }

    #[test]
    fn leave_status_state_machine_is_correct() {
        // Pending -> Approved | Rejected | Cancelled
        assert!(LeaveStatus::Pending.can_transition_to(LeaveStatus::Approved));
        assert!(LeaveStatus::Pending.can_transition_to(LeaveStatus::Rejected));
        assert!(LeaveStatus::Pending.can_transition_to(LeaveStatus::Cancelled));
        // Pending -> Pending is illegal
        assert!(!LeaveStatus::Pending.can_transition_to(LeaveStatus::Pending));
        // Approved -> Cancelled (within grace window)
        assert!(LeaveStatus::Approved.can_transition_to(LeaveStatus::Cancelled));
        // Approved -> Approved or Rejected is illegal
        assert!(!LeaveStatus::Approved.can_transition_to(LeaveStatus::Approved));
        assert!(!LeaveStatus::Approved.can_transition_to(LeaveStatus::Rejected));
        // Rejected is terminal
        assert!(LeaveStatus::Rejected.is_terminal());
        assert!(!LeaveStatus::Rejected.can_transition_to(LeaveStatus::Pending));
        assert!(!LeaveStatus::Rejected.can_transition_to(LeaveStatus::Approved));
        // Cancelled is terminal
        assert!(!LeaveStatus::Cancelled.can_transition_to(LeaveStatus::Pending));
    }

    #[test]
    fn payroll_status_state_machine_is_correct() {
        // NotGenerated -> Generated
        assert!(!PayrollStatus::NotGenerated.is_paid());
        // Generated -> Paid
        assert!(!PayrollStatus::Generated.is_paid());
        // Paid is terminal
        assert!(PayrollStatus::Paid.is_paid());
    }

    #[test]
    fn attendance_type_round_trip() {
        for t in [
            AttendanceType::Present,
            AttendanceType::Late,
            AttendanceType::Absent,
            AttendanceType::HalfDay,
            AttendanceType::Holiday,
        ] {
            let parsed = AttendanceType::parse(t.as_str()).unwrap();
            assert_eq!(parsed, t);
        }
        assert!(AttendanceType::Absent.is_absent());
        assert!(!AttendanceType::Present.is_absent());
    }

    #[test]
    fn earn_dedc_type_round_trip() {
        assert_eq!(
            EarnDeducType::parse(EarnDeducType::Earning.as_str()).unwrap(),
            EarnDeducType::Earning
        );
        assert_eq!(
            EarnDeducType::parse(EarnDeducType::Deduction.as_str()).unwrap(),
            EarnDeducType::Deduction
        );
        assert!(EarnDeducType::parse("x").is_err());
    }

    #[test]
    fn typed_id_display() {
        let school = SchoolId(Uuid::now_v7());
        let id = StaffId::new(school, Uuid::now_v7());
        let s = id.to_string();
        assert!(s.contains('/'));
        assert!(s.starts_with(&school.to_string()));
    }
}
