//! # HR repository ports
//!
//! Phase 6 ships the 16 `#[async_trait]` repository port
//! traits. Storage adapters (PG/MySQL/SQLite) implement
//! these; the test fixtures in this crate use in-memory
//! implementations matching the Phase 5 `attendance` pattern.

#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_imports)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::value_objects::{
    AssignClassTeacherId, DepartmentId, DesignationId, HourlyRateId, LeaveDefineId, LeaveRequestId,
    LeaveTypeId, PayrollEarnDeducId, PayrollGenerateId, SalaryTemplateId, StaffAttendanceId,
    StaffAttendanceImportId, StaffId, StaffImportBulkTemporaryId, StaffRegistrationFieldId,
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
    }
}
