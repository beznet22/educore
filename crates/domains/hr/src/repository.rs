//! # HR repository ports
//!
//! Phase 6 ships the 42 `#[async_trait]` repository port
//! traits (16 base + 26 child/projection aggregates).
//! Storage adapters (PG/MySQL/SQLite) implement these; the
//! test fixtures in this crate use in-memory implementations
//! matching the Phase 5 `attendance` pattern.

#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AssignClassTeacherId, AssignClassTeacherScopeId, BulkImportJobId, DepartmentHeadId,
    DepartmentId, DesignationGradeId, DesignationId, HourlyRateId, HourlyRateOverrideId,
    LeaveDefineAdjustmentId, LeaveDefineId, LeaveRequestApprovalId, LeaveRequestAttachmentId,
    LeaveRequestId, LeaveTypeId, PayrollEarnDeducId, PayrollGenerateAuditId, PayrollGenerateId,
    PayrollPaymentLinkId, SalaryTemplateId, StaffAddressId, StaffAttendanceId,
    StaffAttendanceImportBatchId, StaffAttendanceImportId, StaffAttendancePunchId,
    StaffBankDetailId, StaffCustomFieldId, StaffDocumentId, StaffDrivingLicenseId, StaffId,
    StaffImportBulkTemporaryId, StaffImportResolutionId, StaffLeaveBalanceId, StaffLeaveHistoryId,
    StaffPayrollHistoryId, StaffProfilePhotoId, StaffRegistrationFieldId,
    StaffRegistrationFieldOptionId, StaffRoleAssignmentId, StaffSocialLinkId, StaffTimelineId,
};

#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffId,
    ) -> Result<Option<crate::aggregate::Staff>>;
    async fn get_by_email(
        &self,
        school: SchoolId,
        email: &str,
    ) -> Result<Option<crate::aggregate::Staff>>;
    async fn get_by_mobile(
        &self,
        school: SchoolId,
        mobile: &str,
    ) -> Result<Option<crate::aggregate::Staff>>;
    async fn get_by_staff_no(
        &self,
        school: SchoolId,
        staff_no: u32,
    ) -> Result<Option<crate::aggregate::Staff>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<crate::aggregate::Staff>>;
    async fn list_for_department(
        &self,
        school: SchoolId,
        dept: DepartmentId,
    ) -> Result<Vec<crate::aggregate::Staff>>;
    async fn list_for_designation(
        &self,
        school: SchoolId,
        desig: DesignationId,
    ) -> Result<Vec<crate::aggregate::Staff>>;
    async fn insert(&self, ctx: &TenantContext, s: &crate::aggregate::Staff) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, s: &crate::aggregate::Staff) -> Result<()>;
}

#[async_trait]
pub trait DepartmentRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DepartmentId,
    ) -> Result<Option<crate::aggregate::Department>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<crate::aggregate::Department>>;
    async fn find_by_name(
        &self,
        school: SchoolId,
        name: &str,
    ) -> Result<Option<crate::aggregate::Department>>;
    async fn insert(&self, ctx: &TenantContext, d: &crate::aggregate::Department) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, d: &crate::aggregate::Department) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: DepartmentId) -> Result<()>;
}

#[async_trait]
pub trait DesignationRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DesignationId,
    ) -> Result<Option<crate::aggregate::Designation>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<crate::aggregate::Designation>>;
    async fn find_by_title(
        &self,
        school: SchoolId,
        title: &str,
    ) -> Result<Option<crate::aggregate::Designation>>;
    async fn insert(&self, ctx: &TenantContext, d: &crate::aggregate::Designation) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, d: &crate::aggregate::Designation) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: DesignationId) -> Result<()>;
}

#[async_trait]
pub trait LeaveTypeRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveTypeId,
    ) -> Result<Option<crate::aggregate::LeaveType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<crate::aggregate::LeaveType>>;
    async fn find_by_name(
        &self,
        school: SchoolId,
        name: &str,
    ) -> Result<Option<crate::aggregate::LeaveType>>;
    async fn insert(&self, ctx: &TenantContext, lt: &crate::aggregate::LeaveType) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, lt: &crate::aggregate::LeaveType) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: LeaveTypeId) -> Result<()>;
}

#[async_trait]
pub trait LeaveDefineRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveDefineId,
    ) -> Result<Option<crate::aggregate::LeaveDefine>>;
    async fn list_for_school(
        &self,
        school: SchoolId,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Vec<crate::aggregate::LeaveDefine>>;
    async fn insert(&self, ctx: &TenantContext, d: &crate::aggregate::LeaveDefine) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, d: &crate::aggregate::LeaveDefine) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: LeaveDefineId) -> Result<()>;
}

#[async_trait]
pub trait LeaveRequestRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveRequestId,
    ) -> Result<Option<crate::aggregate::LeaveRequest>>;
    async fn list_for_staff(&self, staff: StaffId) -> Result<Vec<crate::aggregate::LeaveRequest>>;
    async fn list_pending(&self, school: SchoolId) -> Result<Vec<crate::aggregate::LeaveRequest>>;
    async fn list_for_period(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<crate::aggregate::LeaveRequest>>;
    async fn insert(&self, ctx: &TenantContext, r: &crate::aggregate::LeaveRequest) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, r: &crate::aggregate::LeaveRequest) -> Result<()>;
}

#[async_trait]
pub trait StaffAttendanceRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendanceId,
    ) -> Result<Option<crate::aggregate::StaffAttendance>>;
    async fn list_for_staff(
        &self,
        staff: StaffId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<crate::aggregate::StaffAttendance>>;
    async fn list_for_school(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<crate::aggregate::StaffAttendance>>;
    async fn find_for_date(
        &self,
        staff: StaffId,
        date: NaiveDate,
    ) -> Result<Option<crate::aggregate::StaffAttendance>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffAttendance,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffAttendance,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: StaffAttendanceId) -> Result<()>;
}

#[async_trait]
pub trait StaffAttendanceImportRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendanceImportId,
    ) -> Result<Option<crate::aggregate::StaffAttendanceImport>>;
    async fn list_pending(
        &self,
        school: SchoolId,
    ) -> Result<Vec<crate::aggregate::StaffAttendanceImport>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffAttendanceImport,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffAttendanceImport,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: StaffAttendanceImportId) -> Result<()>;
}

#[async_trait]
pub trait AssignClassTeacherRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: AssignClassTeacherId,
    ) -> Result<Option<crate::aggregate::AssignClassTeacher>>;
    async fn list_for_school(
        &self,
        school: SchoolId,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Vec<crate::aggregate::AssignClassTeacher>>;
    async fn list_for_staff(
        &self,
        staff: StaffId,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Vec<crate::aggregate::AssignClassTeacher>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::AssignClassTeacher,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::AssignClassTeacher,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: AssignClassTeacherId) -> Result<()>;
}

#[async_trait]
pub trait HourlyRateRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: HourlyRateId,
    ) -> Result<Option<crate::aggregate::HourlyRate>>;
    async fn list(
        &self,
        school: SchoolId,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Vec<crate::aggregate::HourlyRate>>;
    async fn find_by_grade(
        &self,
        school: SchoolId,
        grade: &str,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Option<crate::aggregate::HourlyRate>>;
    async fn insert(&self, ctx: &TenantContext, r: &crate::aggregate::HourlyRate) -> Result<()>;
    async fn update(&self, ctx: &TenantContext, r: &crate::aggregate::HourlyRate) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: HourlyRateId) -> Result<()>;
}

#[async_trait]
pub trait SalaryTemplateRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: SalaryTemplateId,
    ) -> Result<Option<crate::aggregate::SalaryTemplate>>;
    async fn list(
        &self,
        school: SchoolId,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Vec<crate::aggregate::SalaryTemplate>>;
    async fn find_by_grade(
        &self,
        school: SchoolId,
        grade: &str,
        academic: educore_academic::AcademicYearId,
    ) -> Result<Option<crate::aggregate::SalaryTemplate>>;
    async fn insert(&self, ctx: &TenantContext, t: &crate::aggregate::SalaryTemplate)
        -> Result<()>;
    async fn update(&self, ctx: &TenantContext, t: &crate::aggregate::SalaryTemplate)
        -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: SalaryTemplateId) -> Result<()>;
}

#[async_trait]
pub trait PayrollGenerateRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollGenerateId,
    ) -> Result<Option<crate::aggregate::PayrollGenerate>>;
    async fn list_for_staff(
        &self,
        staff: StaffId,
    ) -> Result<Vec<crate::aggregate::PayrollGenerate>>;
    async fn list_for_period(
        &self,
        school: SchoolId,
        month: u8,
        year: u16,
    ) -> Result<Vec<crate::aggregate::PayrollGenerate>>;
    async fn list_pending_approval(
        &self,
        school: SchoolId,
    ) -> Result<Vec<crate::aggregate::PayrollGenerate>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::PayrollGenerate,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::PayrollGenerate,
    ) -> Result<()>;
}

#[async_trait]
pub trait PayrollEarnDeducRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollEarnDeducId,
    ) -> Result<Option<crate::aggregate::PayrollEarnDeduc>>;
    async fn list_for_payroll(
        &self,
        payroll: PayrollGenerateId,
    ) -> Result<Vec<crate::aggregate::PayrollEarnDeduc>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::PayrollEarnDeduc,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::PayrollEarnDeduc,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: PayrollEarnDeducId) -> Result<()>;
}

#[async_trait]
pub trait LeaveDeductionInfoRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::LeaveDeductionInfoId,
    ) -> Result<Option<crate::aggregate::LeaveDeductionInfo>>;
    async fn list_for_payroll(
        &self,
        payroll: PayrollGenerateId,
    ) -> Result<Vec<crate::aggregate::LeaveDeductionInfo>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::LeaveDeductionInfo,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::LeaveDeductionInfo,
    ) -> Result<()>;
    async fn delete(
        &self,
        ctx: &TenantContext,
        id: crate::value_objects::LeaveDeductionInfoId,
    ) -> Result<()>;
}

#[async_trait]
pub trait StaffRegistrationFieldRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffRegistrationFieldId,
    ) -> Result<Option<crate::aggregate::StaffRegistrationField>>;
    async fn list(&self, school: SchoolId)
        -> Result<Vec<crate::aggregate::StaffRegistrationField>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        f: &crate::aggregate::StaffRegistrationField,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        f: &crate::aggregate::StaffRegistrationField,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: StaffRegistrationFieldId) -> Result<()>;
}

#[async_trait]
pub trait StaffImportBulkTemporaryRepository: Send + Sync {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffImportBulkTemporaryId,
    ) -> Result<Option<crate::aggregate::StaffImportBulkTemporary>>;
    async fn list_for_school(
        &self,
        school: SchoolId,
    ) -> Result<Vec<crate::aggregate::StaffImportBulkTemporary>>;
    async fn insert(
        &self,
        ctx: &TenantContext,
        r: &crate::aggregate::StaffImportBulkTemporary,
    ) -> Result<()>;
    async fn update(
        &self,
        ctx: &TenantContext,
        r: &crate::aggregate::StaffImportBulkTemporary,
    ) -> Result<()>;
    async fn delete(&self, ctx: &TenantContext, id: StaffImportBulkTemporaryId) -> Result<()>;
}

// === AssignClassTeacherScope repository section begin ===

#[async_trait]
pub trait AssignClassTeacherScopeRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: AssignClassTeacherScopeId,
    ) -> Result<Option<crate::aggregate::AssignClassTeacherScope>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AssignClassTeacherScope,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        s: &crate::aggregate::AssignClassTeacherScope,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_assign_class_teacher_scope_object_safe() {
    let _: Box<dyn AssignClassTeacherScopeRepository>;
}

// === AssignClassTeacherScope repository section end ===

// === BulkImportJob repository section begin ===

#[async_trait]
pub trait BulkImportJobRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: BulkImportJobId,
    ) -> Result<Option<crate::aggregate::BulkImportJob>>;
    /// Insert a new row.
    async fn insert(&self, ctx: &TenantContext, j: &crate::aggregate::BulkImportJob) -> Result<()>;
    /// Update an existing row.
    async fn update(&self, ctx: &TenantContext, j: &crate::aggregate::BulkImportJob) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_bulk_import_job_object_safe() {
    let _: Box<dyn BulkImportJobRepository>;
}

// === BulkImportJob repository section end ===

// === DepartmentHead repository section begin ===

#[async_trait]
pub trait DepartmentHeadRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DepartmentHeadId,
    ) -> Result<Option<crate::aggregate::DepartmentHead>>;
    /// Insert a new row.
    async fn insert(&self, ctx: &TenantContext, d: &crate::aggregate::DepartmentHead)
        -> Result<()>;
    /// Update an existing row.
    async fn update(&self, ctx: &TenantContext, d: &crate::aggregate::DepartmentHead)
        -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_department_head_object_safe() {
    let _: Box<dyn DepartmentHeadRepository>;
}

// === DepartmentHead repository section end ===

// === DesignationGrade repository section begin ===

#[async_trait]
pub trait DesignationGradeRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: DesignationGradeId,
    ) -> Result<Option<crate::aggregate::DesignationGrade>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        g: &crate::aggregate::DesignationGrade,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        g: &crate::aggregate::DesignationGrade,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_designation_grade_object_safe() {
    let _: Box<dyn DesignationGradeRepository>;
}

// === DesignationGrade repository section end ===

// === HourlyRateOverride repository section begin ===

#[async_trait]
pub trait HourlyRateOverrideRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: HourlyRateOverrideId,
    ) -> Result<Option<crate::aggregate::HourlyRateOverride>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        o: &crate::aggregate::HourlyRateOverride,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        o: &crate::aggregate::HourlyRateOverride,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_hourly_rate_override_object_safe() {
    let _: Box<dyn HourlyRateOverrideRepository>;
}

// === HourlyRateOverride repository section end ===

// === LeaveDefineAdjustment repository section begin ===

#[async_trait]
pub trait LeaveDefineAdjustmentRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveDefineAdjustmentId,
    ) -> Result<Option<crate::aggregate::LeaveDefineAdjustment>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveDefineAdjustment,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveDefineAdjustment,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_leave_define_adjustment_object_safe() {
    let _: Box<dyn LeaveDefineAdjustmentRepository>;
}

// === LeaveDefineAdjustment repository section end ===

// === LeaveRequestApproval repository section begin ===

#[async_trait]
pub trait LeaveRequestApprovalRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveRequestApprovalId,
    ) -> Result<Option<crate::aggregate::LeaveRequestApproval>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveRequestApproval,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveRequestApproval,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_leave_request_approval_object_safe() {
    let _: Box<dyn LeaveRequestApprovalRepository>;
}

// === LeaveRequestApproval repository section end ===

// === LeaveRequestAttachment repository section begin ===

#[async_trait]
pub trait LeaveRequestAttachmentRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: LeaveRequestAttachmentId,
    ) -> Result<Option<crate::aggregate::LeaveRequestAttachment>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveRequestAttachment,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::LeaveRequestAttachment,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_leave_request_attachment_object_safe() {
    let _: Box<dyn LeaveRequestAttachmentRepository>;
}

// === LeaveRequestAttachment repository section end ===

// === PayrollGenerateAudit repository section begin ===

#[async_trait]
pub trait PayrollGenerateAuditRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollGenerateAuditId,
    ) -> Result<Option<crate::aggregate::PayrollGenerateAudit>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::PayrollGenerateAudit,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::PayrollGenerateAudit,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_payroll_generate_audit_object_safe() {
    let _: Box<dyn PayrollGenerateAuditRepository>;
}

// === PayrollGenerateAudit repository section end ===

// === PayrollPaymentLink repository section begin ===

#[async_trait]
pub trait PayrollPaymentLinkRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: PayrollPaymentLinkId,
    ) -> Result<Option<crate::aggregate::PayrollPaymentLink>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::PayrollPaymentLink,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::PayrollPaymentLink,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_payroll_payment_link_object_safe() {
    let _: Box<dyn PayrollPaymentLinkRepository>;
}

// === PayrollPaymentLink repository section end ===

// === StaffAddress repository section begin ===

#[async_trait]
pub trait StaffAddressRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAddressId,
    ) -> Result<Option<crate::aggregate::StaffAddress>>;
    /// Insert a new row.
    async fn insert(&self, ctx: &TenantContext, a: &crate::aggregate::StaffAddress) -> Result<()>;
    /// Update an existing row.
    async fn update(&self, ctx: &TenantContext, a: &crate::aggregate::StaffAddress) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_address_object_safe() {
    let _: Box<dyn StaffAddressRepository>;
}

// === StaffAddress repository section end ===

// === StaffAttendanceImportBatch repository section begin ===

#[async_trait]
pub trait StaffAttendanceImportBatchRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendanceImportBatchId,
    ) -> Result<Option<crate::aggregate::StaffAttendanceImportBatch>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        b: &crate::aggregate::StaffAttendanceImportBatch,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        b: &crate::aggregate::StaffAttendanceImportBatch,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_attendance_import_batch_object_safe() {
    let _: Box<dyn StaffAttendanceImportBatchRepository>;
}

// === StaffAttendanceImportBatch repository section end ===

// === StaffAttendancePunch repository section begin ===

#[async_trait]
pub trait StaffAttendancePunchRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffAttendancePunchId,
    ) -> Result<Option<crate::aggregate::StaffAttendancePunch>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::StaffAttendancePunch,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::StaffAttendancePunch,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_attendance_punch_object_safe() {
    let _: Box<dyn StaffAttendancePunchRepository>;
}

// === StaffAttendancePunch repository section end ===

// === StaffBankDetail repository section begin ===

#[async_trait]
pub trait StaffBankDetailRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffBankDetailId,
    ) -> Result<Option<crate::aggregate::StaffBankDetail>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        d: &crate::aggregate::StaffBankDetail,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        d: &crate::aggregate::StaffBankDetail,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_bank_detail_object_safe() {
    let _: Box<dyn StaffBankDetailRepository>;
}

// === StaffBankDetail repository section end ===

// === StaffCustomField repository section begin ===

#[async_trait]
pub trait StaffCustomFieldRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffCustomFieldId,
    ) -> Result<Option<crate::aggregate::StaffCustomField>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        f: &crate::aggregate::StaffCustomField,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        f: &crate::aggregate::StaffCustomField,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_custom_field_object_safe() {
    let _: Box<dyn StaffCustomFieldRepository>;
}

// === StaffCustomField repository section end ===

// === StaffDocument repository section begin ===

#[async_trait]
pub trait StaffDocumentRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffDocumentId,
    ) -> Result<Option<crate::aggregate::StaffDocument>>;
    /// Insert a new row.
    async fn insert(&self, ctx: &TenantContext, d: &crate::aggregate::StaffDocument) -> Result<()>;
    /// Update an existing row.
    async fn update(&self, ctx: &TenantContext, d: &crate::aggregate::StaffDocument) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_document_object_safe() {
    let _: Box<dyn StaffDocumentRepository>;
}

// === StaffDocument repository section end ===

// === StaffDrivingLicense repository section begin ===

#[async_trait]
pub trait StaffDrivingLicenseRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffDrivingLicenseId,
    ) -> Result<Option<crate::aggregate::StaffDrivingLicense>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::StaffDrivingLicense,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::StaffDrivingLicense,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_driving_license_object_safe() {
    let _: Box<dyn StaffDrivingLicenseRepository>;
}

// === StaffDrivingLicense repository section end ===

// === StaffImportResolution repository section begin ===

#[async_trait]
pub trait StaffImportResolutionRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffImportResolutionId,
    ) -> Result<Option<crate::aggregate::StaffImportResolution>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        r: &crate::aggregate::StaffImportResolution,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        r: &crate::aggregate::StaffImportResolution,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_import_resolution_object_safe() {
    let _: Box<dyn StaffImportResolutionRepository>;
}

// === StaffImportResolution repository section end ===

// === StaffLeaveBalance repository section begin ===

#[async_trait]
pub trait StaffLeaveBalanceRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffLeaveBalanceId,
    ) -> Result<Option<crate::aggregate::StaffLeaveBalance>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        b: &crate::aggregate::StaffLeaveBalance,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        b: &crate::aggregate::StaffLeaveBalance,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_leave_balance_object_safe() {
    let _: Box<dyn StaffLeaveBalanceRepository>;
}

// === StaffLeaveBalance repository section end ===

// === StaffLeaveHistory repository section begin ===

#[async_trait]
pub trait StaffLeaveHistoryRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffLeaveHistoryId,
    ) -> Result<Option<crate::aggregate::StaffLeaveHistory>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        h: &crate::aggregate::StaffLeaveHistory,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        h: &crate::aggregate::StaffLeaveHistory,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_leave_history_object_safe() {
    let _: Box<dyn StaffLeaveHistoryRepository>;
}

// === StaffLeaveHistory repository section end ===

// === StaffPayrollHistory repository section begin ===

#[async_trait]
pub trait StaffPayrollHistoryRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffPayrollHistoryId,
    ) -> Result<Option<crate::aggregate::StaffPayrollHistory>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        h: &crate::aggregate::StaffPayrollHistory,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        h: &crate::aggregate::StaffPayrollHistory,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_payroll_history_object_safe() {
    let _: Box<dyn StaffPayrollHistoryRepository>;
}

// === StaffPayrollHistory repository section end ===

// === StaffProfilePhoto repository section begin ===

#[async_trait]
pub trait StaffProfilePhotoRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffProfilePhotoId,
    ) -> Result<Option<crate::aggregate::StaffProfilePhoto>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::StaffProfilePhoto,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        p: &crate::aggregate::StaffProfilePhoto,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_profile_photo_object_safe() {
    let _: Box<dyn StaffProfilePhotoRepository>;
}

// === StaffProfilePhoto repository section end ===

// === StaffRegistrationFieldOption repository section begin ===

#[async_trait]
pub trait StaffRegistrationFieldOptionRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffRegistrationFieldOptionId,
    ) -> Result<Option<crate::aggregate::StaffRegistrationFieldOption>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        o: &crate::aggregate::StaffRegistrationFieldOption,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        o: &crate::aggregate::StaffRegistrationFieldOption,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_registration_field_option_object_safe() {
    let _: Box<dyn StaffRegistrationFieldOptionRepository>;
}

// === StaffRegistrationFieldOption repository section end ===

// === StaffRoleAssignment repository section begin ===

#[async_trait]
pub trait StaffRoleAssignmentRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffRoleAssignmentId,
    ) -> Result<Option<crate::aggregate::StaffRoleAssignment>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffRoleAssignment,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        a: &crate::aggregate::StaffRoleAssignment,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_role_assignment_object_safe() {
    let _: Box<dyn StaffRoleAssignmentRepository>;
}

// === StaffRoleAssignment repository section end ===

// === StaffSocialLink repository section begin ===

#[async_trait]
pub trait StaffSocialLinkRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffSocialLinkId,
    ) -> Result<Option<crate::aggregate::StaffSocialLink>>;
    /// Insert a new row.
    async fn insert(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::StaffSocialLink,
    ) -> Result<()>;
    /// Update an existing row.
    async fn update(
        &self,
        ctx: &TenantContext,
        l: &crate::aggregate::StaffSocialLink,
    ) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_social_link_object_safe() {
    let _: Box<dyn StaffSocialLinkRepository>;
}

// === StaffSocialLink repository section end ===

// === StaffTimeline repository section begin ===

#[async_trait]
pub trait StaffTimelineRepository: Send + Sync {
    /// Fetch by typed id. Returns `Ok(None)` if not found or soft-deleted.
    async fn get(
        &self,
        ctx: &TenantContext,
        id: StaffTimelineId,
    ) -> Result<Option<crate::aggregate::StaffTimeline>>;
    /// Insert a new row.
    async fn insert(&self, ctx: &TenantContext, t: &crate::aggregate::StaffTimeline) -> Result<()>;
    /// Update an existing row.
    async fn update(&self, ctx: &TenantContext, t: &crate::aggregate::StaffTimeline) -> Result<()>;
}

/// Object-safety smoke test.
fn _assert_staff_timeline_object_safe() {
    let _: Box<dyn StaffTimelineRepository>;
}

// === StaffTimeline repository section end ===

#[cfg(test)]
mod tests {
    use super::*;
    fn _assert_object_safe() {
        let _: Box<dyn StaffRepository>;
        let _: Box<dyn DepartmentRepository>;
        let _: Box<dyn DesignationRepository>;
        let _: Box<dyn LeaveTypeRepository>;
        let _: Box<dyn LeaveDefineRepository>;
        let _: Box<dyn LeaveRequestRepository>;
        let _: Box<dyn StaffAttendanceRepository>;
        let _: Box<dyn StaffAttendanceImportRepository>;
        let _: Box<dyn AssignClassTeacherRepository>;
        let _: Box<dyn HourlyRateRepository>;
        let _: Box<dyn SalaryTemplateRepository>;
        let _: Box<dyn PayrollGenerateRepository>;
        let _: Box<dyn PayrollEarnDeducRepository>;
        let _: Box<dyn LeaveDeductionInfoRepository>;
        let _: Box<dyn StaffRegistrationFieldRepository>;
        let _: Box<dyn StaffImportBulkTemporaryRepository>;
        let _: Box<dyn AssignClassTeacherScopeRepository>;
        let _: Box<dyn BulkImportJobRepository>;
        let _: Box<dyn DepartmentHeadRepository>;
        let _: Box<dyn DesignationGradeRepository>;
        let _: Box<dyn HourlyRateOverrideRepository>;
        let _: Box<dyn LeaveDefineAdjustmentRepository>;
        let _: Box<dyn LeaveRequestApprovalRepository>;
        let _: Box<dyn LeaveRequestAttachmentRepository>;
        let _: Box<dyn PayrollGenerateAuditRepository>;
        let _: Box<dyn PayrollPaymentLinkRepository>;
        let _: Box<dyn StaffAddressRepository>;
        let _: Box<dyn StaffAttendanceImportBatchRepository>;
        let _: Box<dyn StaffAttendancePunchRepository>;
        let _: Box<dyn StaffBankDetailRepository>;
        let _: Box<dyn StaffCustomFieldRepository>;
        let _: Box<dyn StaffDocumentRepository>;
        let _: Box<dyn StaffDrivingLicenseRepository>;
        let _: Box<dyn StaffImportResolutionRepository>;
        let _: Box<dyn StaffLeaveBalanceRepository>;
        let _: Box<dyn StaffLeaveHistoryRepository>;
        let _: Box<dyn StaffPayrollHistoryRepository>;
        let _: Box<dyn StaffProfilePhotoRepository>;
        let _: Box<dyn StaffRegistrationFieldOptionRepository>;
        let _: Box<dyn StaffRoleAssignmentRepository>;
        let _: Box<dyn StaffSocialLinkRepository>;
        let _: Box<dyn StaffTimelineRepository>;
    }
}
