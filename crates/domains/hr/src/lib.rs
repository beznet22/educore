//! # educore-hr
//!
//!  Staff lifecycle, leave, payroll, staff attendance, designations, departments.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/hr/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(unused_imports)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-hr";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod value_objects;

mod aggregate;
mod commands;
mod entities;
mod errors;
pub mod events;
mod query;
mod repository;
pub mod services;

/// Prelude re-exports the 16 aggregates + 14 closed enums +
/// foreign-key typed ids that the HR services and consumers
/// reach for. The full 16-aggregate + 50+-event surface is
/// exposed per the spec; this prelude is the high-traffic subset.
#[allow(missing_docs)]
pub mod prelude {
    pub use chrono::NaiveDate;
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::{TenantContext, UserType};
    pub use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    pub use crate::aggregate::{
        AssignClassTeacher, Department, Designation, HourlyRate, LeaveDeductionInfo, LeaveDefine,
        LeaveRequest, LeaveType, PayrollEarnDeduc, PayrollGenerate, SalaryTemplate, Staff,
        StaffAttendance, StaffAttendanceImport, StaffImportBulkTemporary, StaffRegistrationField,
    };
    pub use crate::commands::{
        ApproveLeaveCommand, ApprovePayrollCommand, AssignClassTeacherCommand, CancelLeaveCommand,
        ChangeStaffDepartmentCommand, ChangeStaffDesignationCommand, ChangeStaffRoleCommand,
        CreateDepartmentCommand, CreateDesignationCommand, CreateLeaveTypeCommand,
        CreateSalaryTemplateCommand, CreateStaffRegistrationFieldCommand, DefineLeavePolicyCommand,
        DeleteAssignClassTeacherCommand, DeleteStaffCommand, HireStaffCommand,
        ImportStaffBulkCommand, MarkPayrollPaidCommand, MarkStaffAttendanceCommand,
        RejectLeaveCommand, RequestLeaveCommand, RunPayrollCommand, SetHourlyRateCommand,
        StaffImportRow, SuspendStaffCommand, HR_ASSIGN_CLASS_TEACHER_COMMAND_TYPE,
        HR_ASSIGN_CLASS_TEACHER_DELETE_COMMAND_TYPE, HR_ASSIGN_CLASS_TEACHER_UPDATE_COMMAND_TYPE,
        HR_ATTENDANCE_STAFF_DELETE_COMMAND_TYPE, HR_ATTENDANCE_STAFF_MARK_COMMAND_TYPE,
        HR_ATTENDANCE_STAFF_UPDATE_COMMAND_TYPE, HR_DEPARTMENT_CREATE_COMMAND_TYPE,
        HR_DEPARTMENT_DELETE_COMMAND_TYPE, HR_DEPARTMENT_UPDATE_COMMAND_TYPE,
        HR_DESIGNATION_CREATE_COMMAND_TYPE, HR_DESIGNATION_DELETE_COMMAND_TYPE,
        HR_DESIGNATION_UPDATE_COMMAND_TYPE, HR_HOURLY_RATE_SET_COMMAND_TYPE,
        HR_LEAVE_APPROVE_COMMAND_TYPE, HR_LEAVE_CANCEL_COMMAND_TYPE, HR_LEAVE_REJECT_COMMAND_TYPE,
        HR_LEAVE_REQUEST_COMMAND_TYPE, HR_LEAVE_TYPE_CREATE_COMMAND_TYPE,
        HR_LEAVE_TYPE_DELETE_COMMAND_TYPE, HR_LEAVE_TYPE_UPDATE_COMMAND_TYPE,
        HR_PAYROLL_APPROVE_COMMAND_TYPE, HR_PAYROLL_GENERATE_COMMAND_TYPE,
        HR_PAYROLL_MARK_PAID_COMMAND_TYPE, HR_PAYROLL_UPDATE_COMMAND_TYPE,
        HR_SALARY_TEMPLATE_CREATE_COMMAND_TYPE, HR_STAFF_BULK_IMPORT_COMMAND_TYPE,
        HR_STAFF_CHANGE_DEPARTMENT_COMMAND_TYPE, HR_STAFF_CHANGE_DESIGNATION_COMMAND_TYPE,
        HR_STAFF_CHANGE_ROLE_COMMAND_TYPE, HR_STAFF_DELETE_COMMAND_TYPE,
        HR_STAFF_HIRE_COMMAND_TYPE, HR_STAFF_REGISTRATION_FIELD_CREATE_COMMAND_TYPE,
        HR_STAFF_REINSTATE_COMMAND_TYPE, HR_STAFF_RESIGN_COMMAND_TYPE,
        HR_STAFF_RETIRE_COMMAND_TYPE, HR_STAFF_SUSPEND_COMMAND_TYPE,
        HR_STAFF_TERMINATE_COMMAND_TYPE, HR_STAFF_UPDATE_COMMAND_TYPE,
    };
    pub use crate::entities::{StaffAttendanceImportRow, StaffAttendancePromotion, StaffNote};
    pub use crate::errors::HrError;
    pub use crate::events::{
        ClassTeacherAssigned, ClassTeacherAssignmentDeleted, ClassTeacherAssignmentUpdated,
        DepartmentCreated, DepartmentDeleted, DepartmentUpdated, DesignationCreated,
        DesignationDeleted, DesignationUpdated, HourlyRateDeleted, HourlyRateSet,
        HourlyRateUpdated, LeaveApproved, LeaveCancelled, LeaveDeductionInfoAdded,
        LeaveDeductionInfoDeleted, LeaveDeductionInfoUpdated, LeavePolicyDefined,
        LeavePolicyDeleted, LeavePolicyUpdated, LeaveRejected, LeaveRequested, LeaveTypeCreated,
        LeaveTypeDeleted, LeaveTypeUpdated, PayrollAmountsUpdated, PayrollApproved,
        PayrollDeductionAdded, PayrollEarnDeducDeleted, PayrollEarnDeducUpdated,
        PayrollEarningAdded, PayrollGenerated, PayrollPaid, SalaryTemplateCreated,
        SalaryTemplateDeleted, SalaryTemplateUpdated, StaffAttendanceDeleted,
        StaffAttendanceMarked, StaffAttendanceUpdated, StaffBulkImported, StaffDeleted,
        StaffImportPromoted, StaffRegistered, StaffUpdated,
    };
    pub use crate::query::{
        DepartmentQuery, DesignationQuery, LeaveRequestQuery, LeaveTypeQuery, PayrollGenerateQuery,
        StaffAttendanceQuery, StaffQuery,
    };
    pub use crate::repository::{
        AssignClassTeacherRepository, DepartmentRepository, DesignationRepository,
        HourlyRateRepository, LeaveDeductionInfoRepository, LeaveDefineRepository,
        LeaveRequestRepository, LeaveTypeRepository, PayrollEarnDeducRepository,
        PayrollGenerateRepository, SalaryTemplateRepository, StaffAttendanceImportRepository,
        StaffAttendanceRepository, StaffImportBulkTemporaryRepository,
        StaffRegistrationFieldRepository, StaffRepository,
    };
    pub use crate::services::{
        approve_leave, create_department, create_designation, create_leave_type, hire_staff,
        request_leave, run_payroll, InMemoryPayrollPolicy, LeaveAccrualService, PayrollPolicy,
        ReferenceDataUniquenessChecker, StaffUniquenessChecker,
    };
    pub use crate::value_objects::{
        validate_address, validate_date_of_birth, validate_email, validate_leave_reason,
        validate_leave_type_name, validate_pay_period, validate_person_name, validate_phone,
        validate_qualification, validate_salary_grade, AssignClassTeacherId, AttendanceSource,
        AttendanceType, BloodGroup, ContractType, DepartmentId, DepartmentStatus, DesignationId,
        DesignationStatus, EarnDeducType, Gender, LeaveDeductionInfoId, LeaveDefineId,
        LeaveDefineStatus, LeaveRequestId, LeaveStatus, LeaveTypeId, LeaveTypeStatus,
        MaritalStatus, PayrollEarnDeducId, PayrollGenerateId, PayrollPaymentStatus, PayrollStatus,
        RequiredType, SalaryTemplateId, StaffAttendanceId, StaffAttendanceImportId, StaffId,
        StaffImportBulkTemporaryId, StaffRegistrationFieldId, StaffStatus,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-hr");
        assert!(!PACKAGE_VERSION.is_empty());
    }
}
