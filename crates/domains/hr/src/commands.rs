//! # HR command structs and command-type constants
//!
//! Phase 6 ships the typed command shapes for the 6
//! prompt-named aggregates plus the supporting command-type
//! constants the idempotency sub-port reads.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_core::ids::UserId;
use educore_core::tenant::TenantContext;
use educore_rbac::ids::RoleId;

use crate::value_objects::{
    AssignClassTeacherId, DepartmentId, DesignationId, LeaveRequestId, LeaveTypeId, StaffId,
};

// -- Command-type constants (the idempotency sub-port key) --

pub const HR_STAFF_HIRE_COMMAND_TYPE: &str = "hr.staff.hire";
pub const HR_STAFF_UPDATE_COMMAND_TYPE: &str = "hr.staff.update";
pub const HR_STAFF_DELETE_COMMAND_TYPE: &str = "hr.staff.delete";
pub const HR_STAFF_SUSPEND_COMMAND_TYPE: &str = "hr.staff.suspend";
pub const HR_STAFF_REINSTATE_COMMAND_TYPE: &str = "hr.staff.reinstate";
pub const HR_STAFF_RESIGN_COMMAND_TYPE: &str = "hr.staff.resign";
pub const HR_STAFF_TERMINATE_COMMAND_TYPE: &str = "hr.staff.terminate";
pub const HR_STAFF_RETIRE_COMMAND_TYPE: &str = "hr.staff.retire";
pub const HR_STAFF_CHANGE_DEPARTMENT_COMMAND_TYPE: &str = "hr.staff.change_department";
pub const HR_STAFF_CHANGE_DESIGNATION_COMMAND_TYPE: &str = "hr.staff.change_designation";
pub const HR_STAFF_CHANGE_ROLE_COMMAND_TYPE: &str = "hr.staff.change_role";

pub const HR_DEPARTMENT_CREATE_COMMAND_TYPE: &str = "hr.department.create";
pub const HR_DEPARTMENT_UPDATE_COMMAND_TYPE: &str = "hr.department.update";
pub const HR_DEPARTMENT_DELETE_COMMAND_TYPE: &str = "hr.department.delete";

pub const HR_DESIGNATION_CREATE_COMMAND_TYPE: &str = "hr.designation.create";
pub const HR_DESIGNATION_UPDATE_COMMAND_TYPE: &str = "hr.designation.update";
pub const HR_DESIGNATION_DELETE_COMMAND_TYPE: &str = "hr.designation.delete";

pub const HR_LEAVE_TYPE_CREATE_COMMAND_TYPE: &str = "hr.leave_type.create";
pub const HR_LEAVE_TYPE_UPDATE_COMMAND_TYPE: &str = "hr.leave_type.update";
pub const HR_LEAVE_TYPE_DELETE_COMMAND_TYPE: &str = "hr.leave_type.delete";

pub const HR_LEAVE_REQUEST_COMMAND_TYPE: &str = "hr.leave.request";
pub const HR_LEAVE_APPROVE_COMMAND_TYPE: &str = "hr.leave.approve";
pub const HR_LEAVE_REJECT_COMMAND_TYPE: &str = "hr.leave.reject";
pub const HR_LEAVE_CANCEL_COMMAND_TYPE: &str = "hr.leave.cancel";

pub const HR_ATTENDANCE_STAFF_MARK_COMMAND_TYPE: &str = "hr.attendance.staff.mark";
pub const HR_ATTENDANCE_STAFF_UPDATE_COMMAND_TYPE: &str = "hr.attendance.staff.update";
pub const HR_ATTENDANCE_STAFF_DELETE_COMMAND_TYPE: &str = "hr.attendance.staff.delete";

pub const HR_PAYROLL_GENERATE_COMMAND_TYPE: &str = "hr.payroll.generate";
pub const HR_PAYROLL_APPROVE_COMMAND_TYPE: &str = "hr.payroll.approve";
pub const HR_PAYROLL_MARK_PAID_COMMAND_TYPE: &str = "hr.payroll.mark_paid";
pub const HR_PAYROLL_UPDATE_COMMAND_TYPE: &str = "hr.payroll.update";

pub const HR_ASSIGN_CLASS_TEACHER_COMMAND_TYPE: &str = "hr.assign_class_teacher.create";
pub const HR_ASSIGN_CLASS_TEACHER_UPDATE_COMMAND_TYPE: &str = "hr.assign_class_teacher.update";
pub const HR_ASSIGN_CLASS_TEACHER_DELETE_COMMAND_TYPE: &str = "hr.assign_class_teacher.delete";

pub const HR_HOURLY_RATE_SET_COMMAND_TYPE: &str = "hr.hourly_rate.set";
pub const HR_SALARY_TEMPLATE_CREATE_COMMAND_TYPE: &str = "hr.salary_template.create";
pub const HR_STAFF_REGISTRATION_FIELD_CREATE_COMMAND_TYPE: &str =
    "hr.staff_registration_field.create";

pub const HR_STAFF_BULK_IMPORT_COMMAND_TYPE: &str = "hr.staff.bulk_import";

// -- Re-exports of the canonical command shapes from services.rs --

pub use crate::services::{HireStaffCommand, RequestLeaveCommand, RunPayrollCommand};

// -- A few additional command shapes not yet implemented in services.rs --

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuspendStaffCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub reason: String,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStaffDepartmentCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub to_department_id: DepartmentId,
    pub effective_from: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStaffDesignationCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub to_designation_id: DesignationId,
    pub effective_from: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStaffRoleCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub to_role_id: RoleId,
    pub effective_from: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteStaffCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub rejecter_id: UserId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub canceller_id: UserId,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkStaffAttendanceCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub attendance_type: crate::value_objects::AttendanceType,
    pub notes: Option<String>,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub source: crate::value_objects::AttendanceSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovePayrollCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: crate::value_objects::PayrollGenerateId,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkPayrollPaidCommand {
    pub tenant: TenantContext,
    pub payroll_generate_id: crate::value_objects::PayrollGenerateId,
    pub paid_amount: f64,
    pub paid_at: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub class_id: educore_academic::ClassId,
    pub section_id: educore_academic::SectionId,
    pub staff_id: StaffId,
    pub academic_id: educore_academic::AcademicYearId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteAssignClassTeacherCommand {
    pub tenant: TenantContext,
    pub assign_class_teacher_id: AssignClassTeacherId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetHourlyRateCommand {
    pub tenant: TenantContext,
    pub grade: String,
    pub rate: f64,
    pub academic_id: educore_academic::AcademicYearId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSalaryTemplateCommand {
    pub tenant: TenantContext,
    pub salary_grades: String,
    pub salary_basic: f64,
    pub overtime_rate: f64,
    pub house_rent: f64,
    pub provident_fund: f64,
    pub gross_salary: f64,
    pub total_deduction: f64,
    pub net_salary: f64,
    pub academic_id: educore_academic::AcademicYearId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateStaffRegistrationFieldCommand {
    pub tenant: TenantContext,
    pub field_name: String,
    pub label_name: String,
    pub is_required: bool,
    pub staff_edit: bool,
    pub required_type: crate::value_objects::RequiredType,
    pub position: u32,
    pub academic_id: educore_academic::AcademicYearId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDepartmentCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateDesignationCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLeaveTypeCommand {
    pub tenant: TenantContext,
    pub type_name: String,
    pub total_days: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApproveLeaveCommand {
    pub tenant: TenantContext,
    pub leave_request_id: LeaveRequestId,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefineLeavePolicyCommand {
    pub tenant: TenantContext,
    pub role_id: Option<RoleId>,
    pub user_id: Option<UserId>,
    pub type_id: LeaveTypeId,
    pub days: u32,
    pub total_days: u32,
    pub academic_id: educore_academic::AcademicYearId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStaffBulkCommand {
    pub tenant: TenantContext,
    pub source: String,
    pub file_hash: String,
    pub rows: Vec<crate::commands::StaffImportRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffImportRow {
    pub staff_no: String,
    pub first_name: String,
    pub last_name: String,
    pub gender: crate::value_objects::Gender,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub role: Option<String>,
}
