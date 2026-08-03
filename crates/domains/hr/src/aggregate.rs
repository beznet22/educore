//! # HR aggregate roots
//!
//! Phase 6 ships all 16 prompt-named aggregates from
//! `docs/specs/hr/aggregates.md`. Each aggregate follows
//! the standard 17-field pattern (per `AGENTS.md`):

#![allow(clippy::too_many_arguments)]
//!
//! - 1 typed id + 1 `school_id` anchor
//! - domain fields
//! - 10 audit-metadata fields (version, etag, created_at,
//!   updated_at, created_by, updated_by, active_status,
//!   last_event_id, correlation_id, ...)
//!
//! `school_id` is **derived from `id.school_id()`**, never
//! taken from the caller.

#![allow(missing_docs)]
#![allow(unused_imports)]
// The 16 aggregates each carry the
// 10 audit-metadata fields; suppressing
// the file-level missing_docs lint is
// the pragmatic Phase 5/6 pattern.
#![allow(dead_code)] // The 26 Cluster-C minimal stubs (id +
                     // school_id) are not constructed yet —
                     // they exist as type-level scaffolding
                     // for the owning Workstreams.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_core::error::{DomainError, Result};

use educore_rbac::ids::RoleId;

use crate::value_objects::{
    AssignClassTeacherId, AssignClassTeacherScopeId, AttendanceSource, AttendanceType, BloodGroup,
    BulkImportJobId, ContractType, DepartmentHeadId, DepartmentId, DepartmentStatus,
    DesignationGradeId, DesignationId, DesignationStatus, EarnDeducType, Gender, HourlyRateId,
    HourlyRateOverrideId, LeaveDeductionInfoId, LeaveDefineAdjustmentId, LeaveDefineId,
    LeaveDefineStatus, LeaveRequestApprovalId, LeaveRequestAttachmentId, LeaveRequestId,
    LeaveStatus, LeaveTypeId, LeaveTypeStatus, MaritalStatus, PayrollEarnDeducId,
    PayrollGenerateAuditId, PayrollGenerateId, PayrollPaymentLinkId, PayrollPaymentStatus,
    PayrollStatus, RequiredType, SalaryTemplateId, StaffAddressId, StaffAttendanceId,
    StaffAttendanceImportBatchId, StaffAttendanceImportId, StaffAttendancePromotionId,
    StaffAttendancePunchId, StaffBankDetailId, StaffCustomFieldId, StaffDocumentId,
    StaffDrivingLicenseId, StaffId, StaffImportBulkTemporaryId, StaffImportResolutionId,
    StaffLeaveBalanceId, StaffLeaveHistoryId, StaffNoteId, StaffPayrollHistoryId,
    StaffProfilePhotoId, StaffRegistrationFieldId, StaffRegistrationFieldOptionId,
    StaffRoleAssignmentId, StaffSocialLinkId, StaffStatus, StaffTimelineId,
};

use educore_academic::{AcademicYearId, ClassId, SectionId, SubjectId};

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

// =============================================================================
// Staff (canonical)
// =============================================================================

/// The full profile of a school employee.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Staff {
    pub id: StaffId,
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub department_id: Option<DepartmentId>,
    pub designation_id: Option<DesignationId>,
    pub staff_no: u32,
    pub employee_id: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub gender: Gender,
    pub date_of_birth: NaiveDate,
    pub date_of_joining: NaiveDate,
    pub marital_status: MaritalStatus,
    pub blood_group: BloodGroup,
    pub nationality: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub address: Option<String>,
    pub qualification: Option<String>,
    pub experience: Option<String>,
    pub contract_type: ContractType,
    pub location: Option<String>,
    pub epf_no: Option<String>,
    pub status: StaffStatus,
    pub suspension_reason: Option<String>,
    pub expected_return_date: Option<NaiveDate>,
    pub resignation_date: Option<NaiveDate>,
    pub termination_date: Option<NaiveDate>,
    pub retirement_date: Option<NaiveDate>,
    pub casual_leave_quota: f32,
    pub medical_leave_quota: f32,
    pub maternity_leave_quota: f32,
    pub custom_fields: std::collections::BTreeMap<String, String>,
    pub bank_account_id: Option<Uuid>,
    pub profile_photo_id: Option<Uuid>,
    pub driving_license_no: Option<String>,
    pub driving_license_expiry: Option<NaiveDate>,
    pub notes: Option<String>,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Staff {
    pub const FRESH_ETAG: &'static str = "00000000000000000000000000000000";

    /// Constructs a fresh [`Staff`] aggregate.
    ///
    /// **Spec invariants enforced (Wave 171):**
    ///
    /// - **I-1 (tenant anchor):** `school_id` is **derived from
    ///   `id.school_id()`** — never taken from the caller. The
    ///   `hr_typed_id!` macro's typed wrapper enforces this at the
    ///   type-system level: the caller can only construct a
    ///   `StaffId` carrying the correct school, and `school_id` is
    ///   set to `id.school_id()` at field init. See `aggregate.rs`
    ///   line ~92 (the `school_id: id.school_id()` assignment).
    /// - **I-2 (UserId binding):** the `user_id: UserId` field is
    ///   non-Option (one binding per staff); structurally enforced
    ///   by the field type. Duplicate `user_id` bindings are a
    ///   follow-up for a future `user_id_exists` method on the
    ///   `StaffUniquenessChecker` port (currently the trait
    ///   enforces `email`, `mobile`, `staff_no`, `employee_id` —
    ///   see `services.rs::StaffUniquenessChecker`).
    /// - **I-8 (leave quotas non-negative):** `casual_leave_quota`,
    ///   `medical_leave_quota`, `maternity_leave_quota` are
    ///   initialised to `0.0` here; the
    ///   `validate_non_negative_f32_quota` helper enforces the
    ///   invariant on any future mutator that touches them.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: StaffId,
        user_id: UserId,
        role_id: RoleId,
        staff_no: u32,
        employee_id: String,
        first_name: String,
        last_name: String,
        gender: Gender,
        date_of_birth: NaiveDate,
        date_of_joining: NaiveDate,
        status: StaffStatus,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            user_id,
            role_id,
            department_id: None,
            designation_id: None,
            staff_no,
            employee_id,
            first_name,
            middle_name: None,
            last_name,
            gender,
            date_of_birth,
            date_of_joining,
            marital_status: MaritalStatus::Single,
            blood_group: BloodGroup::Unknown,
            nationality: String::new(),
            email: None,
            mobile: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            address: None,
            qualification: None,
            experience: None,
            contract_type: ContractType::Permanent,
            location: None,
            epf_no: None,
            status,
            suspension_reason: None,
            expected_return_date: None,
            resignation_date: None,
            termination_date: None,
            retirement_date: None,
            casual_leave_quota: 0.0,
            medical_leave_quota: 0.0,
            maternity_leave_quota: 0.0,
            custom_fields: std::collections::BTreeMap::new(),
            bank_account_id: None,
            profile_photo_id: None,
            driving_license_no: None,
            driving_license_expiry: None,
            notes: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    pub fn full_name(&self) -> String {
        match &self.middle_name {
            Some(m) if !m.is_empty() => format!("{} {} {}", self.first_name, m, self.last_name),
            _ => format!("{} {}", self.first_name, self.last_name),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_status.is_active() && self.status == StaffStatus::Active
    }

    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Suspends an Active staff member (spec invariant #6).
    /// Sets `status = Suspended`, records the suspension reason
    /// and expected return date (if known), and bumps `updated_at`.
    ///
    /// The `effective_from` date is captured by the
    /// [`StaffSuspended`](crate::events::StaffSuspended) event
    /// in the service layer; the aggregate only stores the
    /// reason + expected return.
    ///
    /// Returns [`DomainError::validation`] if the current
    /// `status` does not permit `Active → Suspended`.
    pub fn suspend(
        &mut self,
        reason: String,
        expected_return: Option<NaiveDate>,
        updated_at: Timestamp,
    ) -> Result<()> {
        if !self.status.can_transition_to(StaffStatus::Suspended) {
            return Err(DomainError::validation(format!(
                "cannot suspend staff in status {:?} (only Active)",
                self.status
            )));
        }
        self.status = StaffStatus::Suspended;
        self.suspension_reason = Some(reason);
        self.expected_return_date = expected_return;
        self.updated_at = updated_at;
        Ok(())
    }

    /// Reinstates a Suspended staff member back to Active
    /// (spec invariant #6). Clears the suspension reason +
    /// expected return date.
    ///
    /// Returns [`DomainError::validation`] if the current
    /// `status` does not permit `Suspended → Active`.
    pub fn reinstate(&mut self, updated_at: Timestamp) -> Result<()> {
        if !self.status.can_transition_to(StaffStatus::Active) {
            return Err(DomainError::validation(format!(
                "cannot reinstate staff in status {:?} (only Suspended)",
                self.status
            )));
        }
        self.status = StaffStatus::Active;
        self.suspension_reason = None;
        self.expected_return_date = None;
        self.updated_at = updated_at;
        Ok(())
    }

    /// Marks an Active or Suspended staff member as Resigned
    /// (spec invariant #6). Terminal: the staff cannot
    /// transition out of `Resigned`.
    ///
    /// Returns [`DomainError::validation`] if the current
    /// `status` does not permit the transition.
    pub fn resign(&mut self, resignation_date: NaiveDate, updated_at: Timestamp) -> Result<()> {
        if !self.status.can_transition_to(StaffStatus::Resigned) {
            return Err(DomainError::validation(format!(
                "cannot resign staff in status {:?} (only Active or Suspended)",
                self.status
            )));
        }
        self.status = StaffStatus::Resigned;
        self.resignation_date = Some(resignation_date);
        self.updated_at = updated_at;
        Ok(())
    }

    /// Marks an Active or Suspended staff member as Terminated
    /// (spec invariant #6). Terminal.
    pub fn terminate(&mut self, termination_date: NaiveDate, updated_at: Timestamp) -> Result<()> {
        if !self.status.can_transition_to(StaffStatus::Terminated) {
            return Err(DomainError::validation(format!(
                "cannot terminate staff in status {:?} (only Active or Suspended)",
                self.status
            )));
        }
        self.status = StaffStatus::Terminated;
        self.termination_date = Some(termination_date);
        self.updated_at = updated_at;
        Ok(())
    }

    /// Marks an Active or Suspended staff member as Retired
    /// (spec invariant #6). Terminal.
    pub fn retire(&mut self, retirement_date: NaiveDate, updated_at: Timestamp) -> Result<()> {
        if !self.status.can_transition_to(StaffStatus::Retired) {
            return Err(DomainError::validation(format!(
                "cannot retire staff in status {:?} (only Active or Suspended)",
                self.status
            )));
        }
        self.status = StaffStatus::Retired;
        self.retirement_date = Some(retirement_date);
        self.updated_at = updated_at;
        Ok(())
    }

    /// Soft-deletes the staff member by flipping
    /// `active_status` to `Inactive`. The row is preserved
    /// (spec invariant #7: history is retained).
    ///
    /// This is distinct from the FSM `retire()` transition
    /// (which moves `status` to `Retired`). Soft-delete
    /// keeps the FSM status untouched.
    pub fn soft_delete(&mut self, updated_at: Timestamp) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = updated_at;
    }

    /// Sets the three leave-quota fields
    /// (`casual_leave_quota`, `medical_leave_quota`,
    /// `maternity_leave_quota`) atomically with
    /// non-negative validation (spec invariant #8).
    ///
    /// Each quota is validated via
    /// [`crate::value_objects::validate_non_negative_f32_quota`];
    /// a single negative value short-circuits with
    /// [`DomainError::validation`] and no state is changed.
    #[allow(clippy::too_many_arguments)]
    pub fn set_leave_quotas(
        &mut self,
        casual: f32,
        medical: f32,
        maternity: f32,
        updated_at: Timestamp,
    ) -> Result<()> {
        crate::value_objects::validate_non_negative_f32_quota("casual_leave_quota", casual)?;
        crate::value_objects::validate_non_negative_f32_quota("medical_leave_quota", medical)?;
        crate::value_objects::validate_non_negative_f32_quota("maternity_leave_quota", maternity)?;
        self.casual_leave_quota = casual;
        self.medical_leave_quota = medical;
        self.maternity_leave_quota = maternity;
        self.updated_at = updated_at;
        Ok(())
    }
}

// =============================================================================
// Department (reference data)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Department {
    pub id: DepartmentId,
    pub school_id: SchoolId,
    pub name: String,
    pub description: Option<String>,
    pub head_staff_id: Option<StaffId>,
    pub is_system_defined: bool,
    pub status: DepartmentStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Department {
    pub fn fresh(
        id: DepartmentId,
        name: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            description: None,
            head_staff_id: None,
            is_system_defined: false,
            status: DepartmentStatus::Active,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant Department#3: "A `Department` with
    /// `is_system_defined` set is a system-defined department and
    /// cannot be deleted." Returns [`DomainError::validation`] when
    /// the department is system-defined.
    pub fn ensure_deletable(&self) -> Result<()> {
        if self.is_system_defined {
            return Err(DomainError::validation(format!(
                "department {} is system-defined and cannot be deleted",
                self.id
            )));
        }
        Ok(())
    }

    /// Spec invariant Department#2: "A `Department` cannot be deleted
    /// while any `Staff` references it."
    ///
    /// The mutator delegates the cross-aggregate reference check to
    /// a [`DepartmentReferenceChecker`] port (defined in
    /// `crates/domains/hr/src/services.rs`). On a clean check, the
    /// mutator flips `active_status` to `Retired` (soft-delete; the
    /// `DepartmentDeleted` event captures the lifecycle transition
    /// for downstream consumers). The FSM `status` field is left
    /// unchanged so the audit history is preserved.
    pub fn soft_delete(
        &mut self,
        refs: &dyn crate::services::DepartmentReferenceChecker,
        at: Timestamp,
        by: UserId,
    ) -> Result<()> {
        self.ensure_deletable()?;
        if refs.has_assigned_staff(self.school_id, self.id) {
            return Err(DomainError::conflict(format!(
                "cannot delete department {}: staff members are assigned",
                self.id
            )));
        }
        if refs.has_department_head(self.school_id, self.id) {
            return Err(DomainError::conflict(format!(
                "cannot delete department {}: a DepartmentHead row references it",
                self.id
            )));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }
}

// =============================================================================
// Designation (reference data)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Designation {
    pub id: DesignationId,
    pub school_id: SchoolId,
    pub title: String,
    pub description: Option<String>,
    pub grade: Option<String>,
    pub is_system_defined: bool,
    pub status: DesignationStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Designation {
    pub fn fresh(
        id: DesignationId,
        title: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            title,
            description: None,
            grade: None,
            is_system_defined: false,
            status: DesignationStatus::Active,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant Designation#3: "A `Designation` with
    /// `is_system_defined` set is a system-defined designation
    /// and cannot be deleted." Returns [`DomainError::validation`]
    /// when the designation is system-defined.
    pub fn ensure_deletable(&self) -> Result<()> {
        if self.is_system_defined {
            return Err(DomainError::validation(format!(
                "designation {} is system-defined and cannot be deleted",
                self.id
            )));
        }
        Ok(())
    }

    /// Spec invariant Designation#2: "A `Designation` cannot be
    /// deleted while any `Staff` references it."
    ///
    /// The mutator delegates the cross-aggregate reference check to
    /// a [`DesignationReferenceChecker`] port (defined in
    /// `crates/domains/hr/src/services.rs`). On a clean check, the
    /// mutator flips `active_status` to `Retired` (soft-delete; the
    /// `DesignationDeleted` event captures the lifecycle transition
    /// for downstream consumers). The FSM `status` field is left
    /// unchanged so the audit history is preserved.
    pub fn soft_delete(
        &mut self,
        refs: &dyn crate::services::DesignationReferenceChecker,
        at: Timestamp,
        by: UserId,
    ) -> Result<()> {
        self.ensure_deletable()?;
        if refs.has_assigned_staff(self.school_id, self.id) {
            return Err(DomainError::conflict(format!(
                "cannot delete designation {}: staff members reference it",
                self.id
            )));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }
}

// =============================================================================
// LeaveType (reference data)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveType {
    pub id: LeaveTypeId,
    pub school_id: SchoolId,
    pub type_name: String,
    pub total_days: u32,
    pub description: Option<String>,
    pub status: LeaveTypeStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LeaveType {
    pub fn fresh(
        id: LeaveTypeId,
        type_name: String,
        total_days: u32,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            type_name,
            total_days,
            description: None,
            status: LeaveTypeStatus::Active,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant LeaveType#3: "`total_days >= 0`." This is
    /// structurally enforced by the `u32` field type (Rust's
    /// unsigned 32-bit integer cannot hold a negative value),
    /// so the mutator is a no-op that documents the invariant
    /// for callers and tests.
    pub fn ensure_total_days_valid(&self) -> Result<()> {
        // Structural invariant: u32 is always >= 0.
        Ok(())
    }

    /// Spec invariant LeaveType#2: "A `LeaveType` cannot be deleted
    /// while any `LeaveDefine` or `LeaveRequest` references it."
    ///
    /// The mutator delegates the cross-aggregate reference check to
    /// a [`LeaveTypeReferenceChecker`] port (defined in
    /// `crates/domains/hr/src/services.rs`). On a clean check, the
    /// mutator flips `active_status` to `Retired` (soft-delete; the
    /// `LeaveTypeDeleted` event captures the lifecycle transition
    /// for downstream consumers).
    pub fn soft_delete(
        &mut self,
        refs: &dyn crate::services::LeaveTypeReferenceChecker,
        at: Timestamp,
        by: UserId,
    ) -> Result<()> {
        if refs.has_leave_define(self.school_id, self.id) {
            return Err(DomainError::conflict(format!(
                "cannot delete leave type {}: LeaveDefine rows reference it",
                self.id
            )));
        }
        if refs.has_leave_request(self.school_id, self.id) {
            return Err(DomainError::conflict(format!(
                "cannot delete leave type {}: LeaveRequest rows reference it",
                self.id
            )));
        }
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }
}

// =============================================================================
// LeaveDefine (leave entitlement per role/user for a year)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDefine {
    pub id: LeaveDefineId,
    pub school_id: SchoolId,
    pub role_id: Option<RoleId>,
    pub user_id: Option<UserId>,
    pub type_id: LeaveTypeId,
    pub days: u32,
    pub total_days: u32,
    pub academic_id: AcademicYearId,
    pub status: LeaveDefineStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LeaveDefine {
    /// Constructs a fresh [`LeaveDefine`] aggregate.
    ///
    /// **Invariant (LeaveDefine#3):** `days <= total_days`.
    /// The constructor asserts this; a violation yields
    /// [`DomainError::validation`] before any state is built.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: LeaveDefineId,
        role_id: Option<RoleId>,
        user_id: Option<UserId>,
        type_id: LeaveTypeId,
        days: u32,
        total_days: u32,
        academic_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Result<Self> {
        if days > total_days {
            return Err(DomainError::validation(format!(
                "leave define days ({days}) must be <= total_days ({total_days})"
            )));
        }
        Ok(Self {
            school_id: id.school_id(),
            id,
            role_id,
            user_id,
            type_id,
            days,
            total_days,
            academic_id,
            status: LeaveDefineStatus::Active,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        })
    }

    /// Spec invariant LeaveDefine#2: "`days >= 0` and
    /// `total_days >= 0`." Both fields are `u32`, which is
    /// structurally non-negative. This mutator documents the
    /// invariant for callers and tests.
    pub fn ensure_non_negative(&self) -> Result<()> {
        // Structural invariant: u32 is always >= 0.
        Ok(())
    }

    /// Spec invariant LeaveDefine#1: "A `LeaveDefine` is unique
    /// by `(school_id, academic_id, role_id, type_id)` or by
    /// `(school_id, academic_id, user_id, type_id)`."
    ///
    /// The mutator delegates the cross-aggregate uniqueness
    /// check to a [`LeaveDefineUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`). The
    /// composite key is `(school, academic, role-or-user, type)`;
    /// the dispatcher decides which key to check based on
    /// whether `role_id` or `user_id` is set.
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::LeaveDefineUniquenessChecker,
    ) -> Result<()> {
        let role_id = self.role_id;
        let user_id = self.user_id;
        if role_id.is_none() && user_id.is_none() {
            return Err(DomainError::validation(
                "leave define must have either role_id or user_id set",
            ));
        }
        if uniqueness.leave_define_exists(
            self.school_id,
            self.academic_id,
            role_id,
            user_id,
            self.type_id,
        ) {
            return Err(DomainError::conflict("leave define already exists for (school, academic, role/user, type) composite key".to_string()));
        }
        Ok(())
    }
}

// =============================================================================
// LeaveRequest (state machine: Pending -> Approved/Rejected/Cancelled)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRequest {
    pub id: LeaveRequestId,
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub type_id: LeaveTypeId,
    pub apply_date: NaiveDate,
    pub leave_from: NaiveDate,
    pub leave_to: NaiveDate,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub file_reference: Option<Uuid>,
    pub approve_status: LeaveStatus,
    pub approver_id: Option<UserId>,
    pub approved_at: Option<Timestamp>,
    pub rejecter_id: Option<UserId>,
    pub rejected_at: Option<Timestamp>,
    pub rejection_reason: Option<String>,
    pub canceller_id: Option<UserId>,
    pub cancelled_at: Option<Timestamp>,
    pub cancellation_reason: Option<String>,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LeaveRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: LeaveRequestId,
        staff_id: StaffId,
        type_id: LeaveTypeId,
        apply_date: NaiveDate,
        leave_from: NaiveDate,
        leave_to: NaiveDate,
        reason: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            type_id,
            apply_date,
            leave_from,
            leave_to,
            reason,
            note: None,
            file_reference: None,
            approve_status: LeaveStatus::Pending,
            approver_id: None,
            approved_at: None,
            rejecter_id: None,
            rejected_at: None,
            rejection_reason: None,
            canceller_id: None,
            cancelled_at: None,
            cancellation_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    pub fn duration_days(&self) -> u32 {
        let days = (self
            .leave_to
            .signed_duration_since(self.leave_from)
            .num_days()
            + 1)
        .max(0);
        u32::try_from(days).unwrap_or(u32::MAX)
    }

    pub fn is_pending(&self) -> bool {
        self.approve_status == LeaveStatus::Pending
    }

    pub fn can_transition(&self, to: LeaveStatus) -> bool {
        self.approve_status.can_transition_to(to)
    }

    /// Spec invariant LeaveRequest#3: "`approve_status` is `pending`
    /// on creation; it transitions to `approved` or `rejected` and
    /// never returns to `pending`."
    ///
    /// This is the I-3 reject mutator. The transition guard
    /// (`Pending → {Approved, Rejected}`) is enforced by
    /// [`LeaveStatus::can_transition_to`] (Wave 32). This mutator
    /// also enforces spec I-5 (reason required for rejections):
    /// the rejection reason must be non-empty when transitioning
    /// to `Rejected`.
    pub fn reject(
        &mut self,
        rejecter_id: UserId,
        rejection_reason: String,
        at: Timestamp,
    ) -> Result<()> {
        if !self.approve_status.can_transition_to(LeaveStatus::Rejected) {
            return Err(DomainError::validation(format!(
                "cannot reject leave request in status {:?}",
                self.approve_status
            )));
        }
        let trimmed = rejection_reason.trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation(
                "rejection reason is required (spec LeaveRequest I-5)",
            ));
        }
        self.approve_status = LeaveStatus::Rejected;
        self.rejecter_id = Some(rejecter_id);
        self.rejected_at = Some(at);
        self.rejection_reason = Some(rejection_reason);
        self.updated_at = at;
        self.updated_by = rejecter_id;
        self.version = self.version.next();
        Ok(())
    }

    /// Spec invariant LeaveRequest#1: "A `LeaveRequest` is unique
    /// by `(school_id, staff_id, leave_from, leave_to, type_id)`
    /// per academic year."
    ///
    /// The mutator delegates the cross-aggregate uniqueness check
    /// to a [`LeaveRequestUniquenessChecker`] port (defined in
    /// `crates/domains/hr/src/services.rs`). On a duplicate the
    /// mutator returns `DomainError::Conflict`.
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::LeaveRequestUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.leave_request_exists(
            self.school_id,
            self.staff_id,
            self.leave_from,
            self.leave_to,
            self.type_id,
        ) {
            return Err(DomainError::conflict("leave request already exists for (school, staff, leave_from, leave_to, type) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant LeaveRequest#5: "The number of days in the
    /// request must not exceed the `LeaveDefine.total_days`."
    ///
    /// The mutator is a pure validator: it compares
    /// [`Self::duration_days`] against the supplied
    /// `leave_define_total_days` and returns
    /// [`DomainError::validation`] if the request would exceed
    /// the entitlement.
    pub fn ensure_within_leave_define(
        &self,
        leave_define_total_days: u32,
    ) -> Result<()> {
        let requested = self.duration_days();
        if requested > leave_define_total_days {
            return Err(DomainError::validation(format!(
                "leave request duration {requested} days exceeds LeaveDefine.total_days {leave_define_total_days}"
            )));
        }
        Ok(())
    }
}

// =============================================================================
// StaffAttendance (HR-side per-staff per-day)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendance {
    pub id: StaffAttendanceId,
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub notes: Option<String>,
    pub marked_by: UserId,
    pub marked_at: Timestamp,
    pub marked_from: AttendanceSource,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffAttendance {
    /// Spec invariant StaffAttendance#1: "A `StaffAttendance` is
    /// unique by `(school_id, staff_id, attendance_date,
    /// academic_id)`." Note: the struct doesn't have an
    /// `academic_id` field directly — the composite key uses
    /// the staff's current academic year. The mutator delegates
    /// the cross-aggregate uniqueness check to a
    /// [`StaffAttendanceUniquenessChecker`] port (defined in
    /// `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::StaffAttendanceUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.staff_attendance_exists(
            self.school_id,
            self.staff_id,
            self.attendance_date,
        ) {
            return Err(DomainError::conflict("staff attendance already exists for (school, staff, date) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant StaffAttendance#3: "`attendance_date` is
    /// required." This is **structural**: the field is
    /// `NaiveDate` (non-Optional). The mutator documents the
    /// invariant for callers and tests.
    pub fn ensure_date_required(&self) -> Result<()> {
        // Structural invariant: NaiveDate is always present.
        Ok(())
    }

    /// Spec invariant StaffAttendance#2: "`attendance_type` is
    /// one of `P` (Present), `L` (Late), `A` (Absent), `H`
    /// (Holiday), `F` (Half Day)." The [`AttendanceType`]
    /// enum has exactly these 5 variants and [`AttendanceType::parse`]
    /// rejects unknown values at the storage boundary. The
    /// mutator is a no-op that documents the invariant for
    /// callers.
    pub fn ensure_attendance_type_valid(&self) -> Result<()> {
        // Structural invariant: enum is exhaustive.
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: StaffAttendanceId,
        staff_id: StaffId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        notes: Option<String>,
        marked_by: UserId,
        marked_at: Timestamp,
        marked_from: AttendanceSource,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            notes,
            marked_by,
            marked_at,
            marked_from,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: marked_at,
            updated_at: marked_at,
            created_by: marked_by,
            updated_by: marked_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// StaffAttendanceImport (staging row for bulk import)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceImport {
    pub id: StaffAttendanceImportId,
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub source: AttendanceSource,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub notes: Option<String>,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffAttendanceImport {
    /// Spec invariant StaffAttendanceImport#1: "A
    /// `StaffAttendanceImport` is unique by `(school_id,
    /// staff_id, attendance_date, academic_id)`." NOTE: the
    /// struct doesn't have an `academic_id` field directly;
    /// the operational uniqueness key is the (school, staff,
    /// attendance_date) triple. The mutator delegates the
    /// cross-aggregate uniqueness check to a
    /// [`StaffAttendanceImportUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::StaffAttendanceImportUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.staff_attendance_import_exists(
            self.school_id,
            self.staff_id,
            self.attendance_date,
        ) {
            return Err(DomainError::conflict("staff attendance import already exists for (school, staff, date) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant StaffAttendanceImport#2: "`in_time` and
    /// `out_time` are stored as `String` to accommodate
    /// arbitrary source formats; promotion validates them."
    /// This is structural: both fields are `Option<String>`.
    /// The mutator documents the invariant for callers.
    pub fn ensure_time_fields_valid(&self) -> Result<()> {
        // Structural invariant: Option<String> accepts any format.
        Ok(())
    }

    /// Spec invariant StaffAttendanceImport#3: "The import is
    /// marked as `active` while pending promotion." Returns
    /// `DomainError::Validation` when `active_status != Active`.
    pub fn ensure_active(&self) -> Result<()> {
        if self.active_status != ActiveStatus::Active {
            return Err(DomainError::validation(format!(
                "staff attendance import {} is not active (active_status = {:?})",
                self.id, self.active_status
            )));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: StaffAttendanceImportId,
        staff_id: StaffId,
        source: AttendanceSource,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        notes: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            source,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            notes,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// AssignClassTeacher
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignClassTeacher {
    pub id: AssignClassTeacherId,
    pub school_id: SchoolId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub staff_id: StaffId,
    pub academic_id: AcademicYearId,
    pub active_status: i32,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_flag: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl AssignClassTeacher {
    /// Spec invariant AssignClassTeacher#1: "An `AssignClassTeacher`
    /// is unique by `(school_id, class_id, section_id, academic_id)`."
    ///
    /// The mutator delegates the cross-aggregate uniqueness check
    /// to an [`AssignClassTeacherUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::AssignClassTeacherUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.assign_class_teacher_exists(
            self.school_id,
            self.class_id,
            self.section_id,
            self.academic_id,
        ) {
            return Err(DomainError::conflict("assign class teacher already exists for (school, class, section, academic) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant AssignClassTeacher#2: "`active_status` is `1`
    /// while the assignment is open." The mutator returns
    /// `DomainError::Validation` when `active_status != 1`.
    pub fn ensure_active_open(&self) -> Result<()> {
        if self.active_status != 1 {
            return Err(DomainError::validation(format!(
                "assign class teacher {} is not open (active_status = {})",
                self.id, self.active_status
            )));
        }
        Ok(())
    }

    pub fn fresh(
        id: AssignClassTeacherId,
        class_id: ClassId,
        section_id: SectionId,
        staff_id: StaffId,
        academic_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            class_id,
            section_id,
            staff_id,
            academic_id,
            active_status: 1,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_flag: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// HourlyRate (per-grade rate)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyRate {
    pub id: HourlyRateId,
    pub school_id: SchoolId,
    pub grade: String,
    pub rate: f64,
    pub academic_id: AcademicYearId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl HourlyRate {
    /// Spec invariant HourlyRate#1: "An `HourlyRate` is
    /// unique by `(school_id, grade, academic_id)`."
    ///
    /// The mutator delegates the cross-aggregate uniqueness
    /// check to an [`HourlyRateUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::HourlyRateUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.hourly_rate_exists(
            self.school_id,
            &self.grade,
            self.academic_id,
        ) {
            return Err(DomainError::conflict("hourly rate already exists for (school, grade, academic) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant HourlyRate#2: "`rate > 0`." The mutator
    /// returns `DomainError::Validation` when `rate <= 0`.
    pub fn ensure_rate_positive(&self) -> Result<()> {
        if self.rate <= 0.0 {
            return Err(DomainError::validation(format!(
                "hourly rate must be > 0.0, got {}",
                self.rate
            )));
        }
        Ok(())
    }

    pub fn fresh(
        id: HourlyRateId,
        grade: String,
        rate: f64,
        academic_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            grade,
            rate,
            academic_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// SalaryTemplate
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplate {
    pub id: SalaryTemplateId,
    pub school_id: SchoolId,
    pub salary_grades: String,
    pub salary_basic: f64,
    pub overtime_rate: f64,
    pub house_rent: f64,
    pub provident_fund: f64,
    pub gross_salary: f64,
    pub total_deduction: f64,
    pub net_salary: f64,
    pub academic_id: AcademicYearId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SalaryTemplate {
    /// Spec invariant SalaryTemplate#1: "A `SalaryTemplate` is
    /// unique by `(school_id, salary_grades, academic_id)`."
    ///
    /// The mutator delegates the cross-aggregate uniqueness
    /// check to a [`SalaryTemplateUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::SalaryTemplateUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.salary_template_exists(
            self.school_id,
            &self.salary_grades,
            self.academic_id,
        ) {
            return Err(DomainError::conflict("salary template already exists for (school, salary_grades, academic) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant SalaryTemplate#2: "`gross_salary ==
    /// salary_basic + house_rent + provident_fund` (or the
    /// consumer-defined composition)." The mutator validates
    /// the invariant within a small epsilon (abs drift allowed
    /// up to 1e-6 to absorb f64 rounding).
    pub fn ensure_gross_salary_consistent(&self) -> Result<()> {
        const EPSILON: f64 = 1e-6;
        let expected = self.salary_basic + self.house_rent + self.provident_fund;
        if (self.gross_salary - expected).abs() > EPSILON {
            return Err(DomainError::validation(format!(
                "gross_salary {} != salary_basic {} + house_rent {} + provident_fund {} = {}",
                self.gross_salary, self.salary_basic, self.house_rent, self.provident_fund, expected
            )));
        }
        Ok(())
    }

    /// Spec invariant SalaryTemplate#3: "`net_salary ==
    /// gross_salary - total_deduction`." Validated within a
    /// small epsilon.
    pub fn ensure_net_salary_consistent(&self) -> Result<()> {
        const EPSILON: f64 = 1e-6;
        let expected = self.gross_salary - self.total_deduction;
        if (self.net_salary - expected).abs() > EPSILON {
            return Err(DomainError::validation(format!(
                "net_salary {} != gross_salary {} - total_deduction {} = {}",
                self.net_salary, self.gross_salary, self.total_deduction, expected
            )));
        }
        Ok(())
    }

    /// Spec invariant SalaryTemplate#4: "The template is
    /// `active` while in use." The mutator returns
    /// `DomainError::Validation` when `active_status != Active`.
    pub fn ensure_active(&self) -> Result<()> {
        if self.active_status != ActiveStatus::Active {
            return Err(DomainError::validation(format!(
                "salary template {} is not active (active_status = {:?})",
                self.id, self.active_status
            )));
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: SalaryTemplateId,
        salary_grades: String,
        salary_basic: f64,
        overtime_rate: f64,
        house_rent: f64,
        provident_fund: f64,
        gross_salary: f64,
        total_deduction: f64,
        net_salary: f64,
        academic_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            salary_grades,
            salary_basic,
            overtime_rate,
            house_rent,
            provident_fund,
            gross_salary,
            total_deduction,
            net_salary,
            academic_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// PayrollGenerate (state: not_generated -> generated -> paid)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollGenerate {
    pub id: PayrollGenerateId,
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub basic_salary: f64,
    pub total_earning: f64,
    pub total_deduction: f64,
    pub gross_salary: f64,
    pub tax: f64,
    pub net_salary: f64,
    pub payroll_month: u8,
    pub payroll_year: u16,
    pub payroll_status: PayrollStatus,
    pub payment_status: PayrollPaymentStatus,
    pub payment_mode: Option<String>,
    pub payment_date: Option<NaiveDate>,
    pub bank_id: Option<Uuid>,
    pub note: Option<String>,
    pub paid_amount: f64,
    pub is_partial: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl PayrollGenerate {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: PayrollGenerateId,
        staff_id: StaffId,
        basic_salary: f64,
        payroll_month: u8,
        payroll_year: u16,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            basic_salary,
            total_earning: 0.0,
            total_deduction: 0.0,
            gross_salary: 0.0,
            tax: 0.0,
            net_salary: 0.0,
            payroll_month,
            payroll_year,
            payroll_status: PayrollStatus::NotGenerated,
            payment_status: PayrollPaymentStatus::Unpaid,
            payment_mode: None,
            payment_date: None,
            bank_id: None,
            note: None,
            paid_amount: 0.0,
            is_partial: false,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant PayrollGenerate#3:
    /// transitions `NotGenerated → Generated → Paid`.
    ///
    /// This is the I-3 mutator. The FSM is strictly forward-only;
    /// see [`PayrollStatus::can_transition_to`] for the allowed
    /// transitions. `Paid` is terminal — once a payroll is paid
    /// it cannot be re-opened.
    pub fn mark_generated(&mut self, at: Timestamp, by: UserId) -> Result<()> {
        if !self.payroll_status.can_transition_to(PayrollStatus::Generated) {
            return Err(DomainError::validation(format!(
                "cannot mark payroll {} generated from status {:?}",
                self.id, self.payroll_status
            )));
        }
        self.payroll_status = PayrollStatus::Generated;
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }

    /// Spec invariant PayrollGenerate#3:
    /// transitions `NotGenerated → Generated → Paid`.
    ///
    /// `Paid` is terminal. Partial payments advance `paid_amount`
    /// via [`Self::record_payment`] but do **not** advance the FSM
    /// until `paid_amount == net_salary`.
    pub fn mark_paid(
        &mut self,
        at: Timestamp,
        by: UserId,
        paid_amount: f64,
    ) -> Result<()> {
        crate::value_objects::validate_paid_amount(paid_amount, self.net_salary)?;
        self.paid_amount = paid_amount;
        self.is_partial = paid_amount < self.net_salary;
        if paid_amount > 0.0 {
            self.payment_status = if self.is_partial {
                PayrollPaymentStatus::Partial
            } else {
                PayrollPaymentStatus::FullyPaid
            };
        }
        // Only advance the FSM to Paid when the full net_salary is paid.
        if !self.is_partial && !self.payroll_status.can_transition_to(PayrollStatus::Paid) {
            return Err(DomainError::validation(format!(
                "cannot mark payroll {} paid from status {:?}",
                self.id, self.payroll_status
            )));
        }
        if !self.is_partial {
            self.payroll_status = PayrollStatus::Paid;
        }
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }

    /// Spec invariant PayrollGenerate#4:
    /// "`paid_amount <= net_salary`".
    ///
    /// Used when a partial payment is recorded before the payroll
    /// is fully paid. Delegates to
    /// [`crate::value_objects::validate_paid_amount`].
    pub fn record_payment(&mut self, paid_amount: f64) -> Result<()> {
        crate::value_objects::validate_paid_amount(paid_amount, self.net_salary)?;
        self.paid_amount = paid_amount;
        self.is_partial = paid_amount < self.net_salary;
        if paid_amount > 0.0 {
            self.payment_status = if self.is_partial {
                PayrollPaymentStatus::Partial
            } else {
                PayrollPaymentStatus::FullyPaid
            };
        }
        Ok(())
    }

    /// Spec invariant PayrollGenerate#1:
    /// "`gross_salary == basic_salary + total_earning`".
    ///
    /// Updates `total_earning` from the sum of `PayrollEarnDeduc`
    /// earning lines + re-derives `gross_salary` and validates the
    /// invariant. The dispatcher passes the summed value from the
    /// storage layer; tests can call it directly.
    pub fn update_amounts(
        &mut self,
        total_earning: f64,
        total_deduction: f64,
        tax: f64,
        at: Timestamp,
        by: UserId,
    ) -> Result<()> {
        if total_earning < 0.0 {
            return Err(DomainError::validation(format!(
                "total_earning must be >= 0.0, got {total_earning}"
            )));
        }
        if total_deduction < 0.0 {
            return Err(DomainError::validation(format!(
                "total_deduction must be >= 0.0, got {total_deduction}"
            )));
        }
        if tax < 0.0 {
            return Err(DomainError::validation(format!(
                "tax must be >= 0.0, got {tax}"
            )));
        }
        let gross_salary = self.basic_salary + total_earning;
        crate::value_objects::validate_gross_salary(self.basic_salary, total_earning, gross_salary)?;
        // Spec invariant PayrollGenerate#2:
        // net_salary == gross_salary - total_deduction - tax
        let net_salary = (gross_salary - total_deduction - tax).max(0.0);
        self.total_earning = total_earning;
        self.total_deduction = total_deduction;
        self.tax = tax;
        self.gross_salary = gross_salary;
        self.net_salary = net_salary;
        self.updated_at = at;
        self.updated_by = by;
        Ok(())
    }
}

// =============================================================================
// PayrollEarnDeduc
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollEarnDeduc {
    pub id: PayrollEarnDeducId,
    pub school_id: SchoolId,
    pub payroll_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: f64,
    pub earn_dedc_type: EarnDeducType,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl PayrollEarnDeduc {
    pub fn fresh(
        id: PayrollEarnDeducId,
        payroll_id: PayrollGenerateId,
        type_name: String,
        amount: f64,
        earn_dedc_type: EarnDeducType,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            payroll_id,
            type_name,
            amount,
            earn_dedc_type,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant PayrollEarnDeduc#1: "`amount >= 0`."
    /// `amount` is `f64`, so it needs a runtime validator.
    pub fn ensure_amount_non_negative(&self) -> Result<()> {
        if self.amount < 0.0 {
            return Err(DomainError::validation(format!(
                "payroll earn/deduc amount must be >= 0.0, got {}",
                self.amount
            )));
        }
        Ok(())
    }

    /// Spec invariant PayrollEarnDeduc#2: "`earn_dedc_type` is
    /// `e` (earning) or `d` (deduction)." The [`EarnDeducType`]
    /// enum has exactly these 2 variants and [`EarnDeducType::parse`]
    /// rejects unknown values at the storage boundary. The
    /// mutator is a no-op that documents the invariant for
    /// callers.
    pub fn ensure_earn_dedc_type_valid(&self) -> Result<()> {
        // Structural invariant: enum is exhaustive.
        Ok(())
    }
}

// =============================================================================
// LeaveDeductionInfo
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDeductionInfo {
    pub id: LeaveDeductionInfoId,
    pub school_id: SchoolId,
    pub staff_id: StaffId,
    pub payroll_id: PayrollGenerateId,
    pub extra_leave: u32,
    pub salary_deduct: f64,
    pub pay_month: u8,
    pub pay_year: u16,
    pub active_status: i32,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_flag: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl LeaveDeductionInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: LeaveDeductionInfoId,
        staff_id: StaffId,
        payroll_id: PayrollGenerateId,
        extra_leave: u32,
        salary_deduct: f64,
        pay_month: u8,
        pay_year: u16,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_id,
            payroll_id,
            extra_leave,
            salary_deduct,
            pay_month,
            pay_year,
            active_status: 1,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_flag: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Spec invariant LeaveDeductionInfo#2: "`extra_leave >= 0`
    /// and `salary_deduct >= 0`." `extra_leave` is `u32`
    /// (structurally non-negative); `salary_deduct` is `f64`
    /// and needs a runtime validator.
    pub fn ensure_non_negative(&self) -> Result<()> {
        if self.salary_deduct < 0.0 {
            return Err(DomainError::validation(format!(
                "leave deduction salary_deduct must be >= 0.0, got {}",
                self.salary_deduct
            )));
        }
        // extra_leave is u32, structurally >= 0.
        Ok(())
    }

    /// Spec invariant LeaveDeductionInfo#1: "A `LeaveDeductionInfo`
    /// is unique by `(school_id, staff_id, payroll_id)`."
    ///
    /// The mutator delegates the cross-aggregate uniqueness check
    /// to a [`LeaveDeductionInfoUniquenessChecker`] port
    /// (defined in `crates/domains/hr/src/services.rs`).
    pub fn ensure_unique(
        &self,
        uniqueness: &dyn crate::services::LeaveDeductionInfoUniquenessChecker,
    ) -> Result<()> {
        if uniqueness.leave_deduction_info_exists(
            self.school_id,
            self.staff_id,
            self.payroll_id,
        ) {
            return Err(DomainError::conflict("leave deduction info already exists for (school, staff, payroll) composite key".to_string()));
        }
        Ok(())
    }

    /// Spec invariant LeaveDeductionInfo#3: "The deduction is
    /// `active` while applied." The aggregate stores this as
    /// `active_status: i32` (1 = active, 0 = inactive per the
    /// legacy Laravel convention). The mutator returns
    /// `Ok(())` when the deduction is active and
    /// [`DomainError::validation`] when it is inactive.
    pub fn ensure_active(&self) -> Result<()> {
        if self.active_status == 0 {
            return Err(DomainError::validation(format!(
                "leave deduction info {} is inactive (active_status = 0)",
                self.id
            )));
        }
        Ok(())
    }
}

// =============================================================================
// StaffRegistrationField (custom field on staff form)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffRegistrationField {
    pub id: StaffRegistrationFieldId,
    pub school_id: SchoolId,
    pub field_name: String,
    pub label_name: String,
    pub active_status: i32,
    pub is_required: bool,
    pub staff_edit: bool,
    pub required_type: RequiredType,
    pub position: u32,
    pub academic_id: AcademicYearId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_flag: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffRegistrationField {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: StaffRegistrationFieldId,
        field_name: String,
        label_name: String,
        is_required: bool,
        staff_edit: bool,
        required_type: RequiredType,
        position: u32,
        academic_id: AcademicYearId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            field_name,
            label_name,
            active_status: 1,
            is_required,
            staff_edit,
            required_type,
            position,
            academic_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_flag: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// StaffImportBulkTemporary (staging row for bulk import)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffImportBulkTemporary {
    pub id: StaffImportBulkTemporaryId,
    pub school_id: SchoolId,
    pub staff_no: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub gender: Gender,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub date_of_joining: Option<NaiveDate>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub role: Option<String>,
    pub resolved_department_id: Option<DepartmentId>,
    pub resolved_designation_id: Option<DesignationId>,
    pub resolved_role_id: Option<RoleId>,
    pub resolved_user_id: Option<UserId>,
    pub importer_user_id: UserId,
    pub source: String,
    pub file_hash: String,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl StaffImportBulkTemporary {
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: StaffImportBulkTemporaryId,
        staff_no: String,
        first_name: String,
        last_name: String,
        gender: Gender,
        source: String,
        file_hash: String,
        importer_user_id: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            staff_no,
            first_name,
            middle_name: None,
            last_name,
            gender,
            email: None,
            mobile: None,
            date_of_birth: None,
            date_of_joining: None,
            department: None,
            designation: None,
            role: None,
            resolved_department_id: None,
            resolved_designation_id: None,
            resolved_role_id: None,
            resolved_user_id: None,
            importer_user_id,
            source,
            file_hash,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by: importer_user_id,
            updated_by: importer_user_id,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// Cluster C: minimal aggregate stubs (id + school_id)
//
// These 26 typed-id structs were introduced in commit 98b47fd
// (`Cluster C (hr): add missing ID types to value_objects`)
// but the corresponding aggregate roots were not landed at the
// same time. Each stub carries only the typed id and the
// derived `school_id` anchor; the full audit-metadata footer
// and domain fields are left for the owning Workstream to fill
// in. `school_id` is derived from `id.school_id()`, never
// taken from the caller.
//
// Two IDs added in 98b47fd (`StaffAttendancePromotionId`,
// `StaffNoteId`) reference entity names already defined in
// `entities.rs`; they are intentionally **not** redefined
// here to avoid a same-crate name collision. The owning
// Workstream can decide whether to migrate the entity into
// `aggregate.rs` (with the typed id) or keep it in
// `entities.rs` (with the legacy fields).
// =============================================================================

/// Bank account information for a staff member.
///
/// Owned child of [`Staff`]. Created during onboarding and
/// updated when the staff member changes banks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffBankDetail {
    pub id: StaffBankDetailId,
    pub school_id: SchoolId,
}

/// Postal address for a staff member.
///
/// Owned child of [`Staff`]. A staff member may have several
/// addresses (permanent, current, emergency).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAddress {
    pub id: StaffAddressId,
    pub school_id: SchoolId,
}

/// Social-profile link (LinkedIn, GitHub, etc.) for a staff member.
///
/// Owned child of [`Staff`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffSocialLink {
    pub id: StaffSocialLinkId,
    pub school_id: SchoolId,
}

/// Uploaded document attached to a staff profile (CV, ID copy, etc.).
///
/// Owned child of [`Staff`]. The blob payload lives in the
/// files adapter; this row carries the metadata + pointer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffDocument {
    pub id: StaffDocumentId,
    pub school_id: SchoolId,
}

/// Per-staff timeline event (promotion, transfer, leave, etc.).
///
/// Projection aggregate; the engine recomputes it from the
/// event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffTimeline {
    pub id: StaffTimelineId,
    pub school_id: SchoolId,
}

/// Per-staff custom-field value row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffCustomField {
    pub id: StaffCustomFieldId,
    pub school_id: SchoolId,
}

/// Per-staff, per-leave-type balance snapshot.
///
/// Recomputed from `LeaveRequestApproved` events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffLeaveBalance {
    pub id: StaffLeaveBalanceId,
    pub school_id: SchoolId,
}

/// Approval/rejection event row attached to a [`LeaveRequest`].
///
/// One leave request may be approved, rejected, then reopened,
/// so this is a history row rather than a single field on the
/// request itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRequestApproval {
    pub id: LeaveRequestApprovalId,
    pub school_id: SchoolId,
}

/// Link between a payroll run and an external payment-record
/// (bank advice, payment gateway transaction, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaymentLink {
    pub id: PayrollPaymentLinkId,
    pub school_id: SchoolId,
}

/// Resolved foreign-key mapping produced by a bulk staff import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffImportResolution {
    pub id: StaffImportResolutionId,
    pub school_id: SchoolId,
}

/// Per-staff, per-period payroll history row.
///
/// Snapshotted when a payroll row advances to `Paid`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffPayrollHistory {
    pub id: StaffPayrollHistoryId,
    pub school_id: SchoolId,
}

/// Per-staff, per-period leave history row.
///
/// Snapshotted when a leave row reaches a terminal state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffLeaveHistory {
    pub id: StaffLeaveHistoryId,
    pub school_id: SchoolId,
}

/// Scope row that attaches additional sections / subjects to an
/// [`AssignClassTeacher`] assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignClassTeacherScope {
    pub id: AssignClassTeacherScopeId,
    pub school_id: SchoolId,
}

/// Head-of-department row (a department may have multiple
/// historical heads; the current head is the latest active row).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepartmentHead {
    pub id: DepartmentHeadId,
    pub school_id: SchoolId,
}

/// Salary grade attached to a [`Designation`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignationGrade {
    pub id: DesignationGradeId,
    pub school_id: SchoolId,
}

/// Per-staff override of an [`HourlyRate`] row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyRateOverride {
    pub id: HourlyRateOverrideId,
    pub school_id: SchoolId,
}

/// Adjustment to a [`LeaveDefine`] entitlement (carry-forward,
/// special grant, manual correction).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDefineAdjustment {
    pub id: LeaveDefineAdjustmentId,
    pub school_id: SchoolId,
}

/// File attachment (medical certificate, travel ticket) attached
/// to a [`LeaveRequest`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRequestAttachment {
    pub id: LeaveRequestAttachmentId,
    pub school_id: SchoolId,
}

/// Raw biometric / RFID punch row, before it is folded into a
/// [`StaffAttendance`] day row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendancePunch {
    pub id: StaffAttendancePunchId,
    pub school_id: SchoolId,
}

/// Append-only audit trail of every state transition on a
/// [`PayrollGenerate`] row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollGenerateAudit {
    pub id: PayrollGenerateAuditId,
    pub school_id: SchoolId,
}

/// Role assignment row (a staff member may hold several roles
/// over time; the current role is the latest active row).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffRoleAssignment {
    pub id: StaffRoleAssignmentId,
    pub school_id: SchoolId,
}

/// Profile-photo metadata row. The blob lives in the files
/// adapter; this row carries the pointer + crop metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffProfilePhoto {
    pub id: StaffProfilePhotoId,
    pub school_id: SchoolId,
}

/// Driving-license metadata row (number, expiry, file pointer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffDrivingLicense {
    pub id: StaffDrivingLicenseId,
    pub school_id: SchoolId,
}

/// Select-option row for a [`StaffRegistrationField`] of type
/// `select` / `multi_select`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffRegistrationFieldOption {
    pub id: StaffRegistrationFieldOptionId,
    pub school_id: SchoolId,
}

/// Top-level metadata row for a bulk staff import job (file
/// hash, status, totals). The individual rows live in
/// [`StaffImportBulkTemporary`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BulkImportJob {
    pub id: BulkImportJobId,
    pub school_id: SchoolId,
}

/// Top-level metadata row for a bulk staff-attendance import
/// job. The individual rows live in [`StaffAttendanceImport`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceImportBatch {
    pub id: StaffAttendanceImportBatchId,
    pub school_id: SchoolId,
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
    use educore_core::ids::{SchoolId, UserId};

    fn school() -> SchoolId {
        SchoolId(Uuid::now_v7())
    }
    fn user() -> UserId {
        UserId(Uuid::now_v7())
    }
    fn corr() -> CorrelationId {
        CorrelationId(Uuid::now_v7())
    }

    #[test]
    fn staff_aggregate_fresh_constructor_initializes_audit_footer() {
        let s = school();
        let id = StaffId::new(s, Uuid::now_v7());
        let staff = Staff::fresh(
            id,
            user(),
            RoleId::new(s, Uuid::now_v7()),
            1,
            "E001".to_owned(),
            "Alice".to_owned(),
            "Patel".to_owned(),
            Gender::Female,
            NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            StaffStatus::Active,
            user(),
            Timestamp::now(),
            corr(),
        );
        assert_eq!(staff.school_id, s);
        assert_eq!(staff.id, id);
        assert_eq!(staff.status, StaffStatus::Active);
        assert!(staff.is_active());
        assert!(!staff.is_terminal());
        assert_eq!(staff.full_name(), "Alice Patel");
    }

    #[test]
    fn department_aggregate_derives_school_id_from_typed_id() {
        let s = school();
        let id = DepartmentId::new(s, Uuid::now_v7());
        let d = Department::fresh(
            id,
            "Mathematics".to_owned(),
            user(),
            Timestamp::now(),
            corr(),
        );
        assert_eq!(d.school_id, s);
        assert!(!d.is_system_defined);
    }

    #[test]
    fn leave_request_state_machine_is_enforced() {
        let s = school();
        let id = LeaveRequestId::new(s, Uuid::now_v7());
        let lr = LeaveRequest::fresh(
            id,
            StaffId::new(s, Uuid::now_v7()),
            LeaveTypeId::new(s, Uuid::now_v7()),
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            Some("Family".to_owned()),
            user(),
            Timestamp::now(),
            corr(),
        );
        assert!(lr.is_pending());
        assert_eq!(lr.duration_days(), 5);
        assert!(lr.can_transition(LeaveStatus::Approved));
        assert!(lr.can_transition(LeaveStatus::Rejected));
        assert!(lr.can_transition(LeaveStatus::Cancelled));
    }

    #[test]
    fn payroll_aggregate_starts_as_not_generated() {
        let s = school();
        let id = PayrollGenerateId::new(s, Uuid::now_v7());
        let p = PayrollGenerate::fresh(
            id,
            StaffId::new(s, Uuid::now_v7()),
            50_000.0,
            6,
            2026,
            user(),
            Timestamp::now(),
            corr(),
        );
        assert_eq!(p.payroll_status, PayrollStatus::NotGenerated);
        assert!(!p.payroll_status.is_paid());
        assert_eq!(p.payment_status, PayrollPaymentStatus::Unpaid);
    }
}
