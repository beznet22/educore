//! # HR domain events
//!
//! All 16 aggregates emit events implementing
//! [`DomainEvent`]. The full set follows the spec at
//! `docs/specs/hr/events.md`. Phase 6 ships the headline
//! events for the 6 prompt-named aggregates plus the
//! core lifecycle, leave, and payroll events. Wire form:
//!
//! ```text
//! hr.<aggregate>.<verb>
//! ```
//!
//! Examples: `hr.staff.registered`, `hr.staff.deleted`,
//! `hr.leave.requested`, `hr.payroll.generated`,
//! `hr.payroll.approved`, `hr.payroll.paid`,
//! `hr.leave.approved`.

#![allow(clippy::too_many_arguments)]
#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::items_after_test_module)]
#![allow(dead_code)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use educore_rbac::ids::RoleId;

use crate::value_objects::{
    AssignClassTeacherId, AttendanceSource, AttendanceType, DepartmentId, DesignationId,
    HourlyRateId, LeaveDefineId, LeaveRequestId, LeaveTypeId, PayrollEarnDeducId,
    PayrollGenerateId, SalaryTemplateId, StaffAttendanceId, StaffId,
};

use educore_academic::{AcademicYearId, ClassId, SectionId};

/// Helper: mints a fresh event id from the supplied generator.
#[allow(dead_code)]
fn fresh_event_id<G: educore_core::clock::IdGenerator + ?Sized>(ids: &G) -> EventId {
    ids.next_event_id()
}

/// Helper: strips the `EventId` wrapper to a raw `uuid::Uuid`
/// for the aggregate id construction.
#[allow(dead_code)]
fn event_id_to_uuid(e: EventId) -> Uuid {
    e.as_uuid()
}

pub use educore_events::domain_event::EventFactory;

// =============================================================================
// Staff lifecycle (12 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffRegistered {
    pub staff_id: StaffId,
    pub staff_no: u32,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub department_id: Option<DepartmentId>,
    pub designation_id: Option<DesignationId>,
    pub role_id: RoleId,
    pub user_id: UserId,
    pub date_of_joining: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffRegistered {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        staff_id: StaffId,
        staff_no: u32,
        employee_id: String,
        first_name: String,
        last_name: String,
        email: Option<String>,
        mobile: Option<String>,
        department_id: Option<DepartmentId>,
        designation_id: Option<DesignationId>,
        role_id: RoleId,
        user_id: UserId,
        date_of_joining: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_id,
            staff_no,
            employee_id,
            first_name,
            last_name,
            email,
            mobile,
            department_id,
            designation_id,
            role_id,
            user_id,
            date_of_joining,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffRegistered {
    const EVENT_TYPE: &'static str = "hr.staff.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffUpdated {
    pub staff_id: StaffId,
    pub changed_fields: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffUpdated {
    pub fn new(
        staff_id: StaffId,
        changed_fields: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_id,
            changed_fields,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffUpdated {
    const EVENT_TYPE: &'static str = "hr.staff.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffSuspended {
    pub staff_id: StaffId,
    pub reason: String,
    pub effective_from: NaiveDate,
    pub expected_return: Option<NaiveDate>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffSuspended {
    pub fn new(
        staff_id: StaffId,
        reason: String,
        effective_from: NaiveDate,
        expected_return: Option<NaiveDate>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_id,
            reason,
            effective_from,
            expected_return,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffSuspended {
    const EVENT_TYPE: &'static str = "hr.staff.suspended";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffDeleted {
    pub staff_id: StaffId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StaffDeleted {
    pub fn new(
        staff_id: StaffId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StaffDeleted {
    const EVENT_TYPE: &'static str = "hr.staff.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Department / Designation / LeaveType (3 each)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepartmentCreated {
    pub department_id: DepartmentId,
    pub name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DepartmentCreated {
    pub fn new(
        department_id: DepartmentId,
        name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            department_id,
            name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DepartmentCreated {
    const EVENT_TYPE: &'static str = "hr.department.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "department";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.department_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.department_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepartmentUpdated {
    pub department_id: DepartmentId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DepartmentUpdated {
    pub fn new(
        department_id: DepartmentId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            department_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DepartmentUpdated {
    const EVENT_TYPE: &'static str = "hr.department.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "department";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.department_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.department_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepartmentDeleted {
    pub department_id: DepartmentId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DepartmentDeleted {
    pub fn new(
        department_id: DepartmentId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            department_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DepartmentDeleted {
    const EVENT_TYPE: &'static str = "hr.department.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "department";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.department_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.department_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignationCreated {
    pub designation_id: DesignationId,
    pub title: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DesignationCreated {
    pub fn new(
        designation_id: DesignationId,
        title: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            designation_id,
            title,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DesignationCreated {
    const EVENT_TYPE: &'static str = "hr.designation.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "designation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.designation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.designation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignationUpdated {
    pub designation_id: DesignationId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for DesignationUpdated {
    const EVENT_TYPE: &'static str = "hr.designation.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "designation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.designation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.designation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignationDeleted {
    pub designation_id: DesignationId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for DesignationDeleted {
    const EVENT_TYPE: &'static str = "hr.designation.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "designation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.designation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.designation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveTypeCreated {
    pub leave_type_id: LeaveTypeId,
    pub type_name: String,
    pub total_days: u32,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveTypeCreated {
    const EVENT_TYPE: &'static str = "hr.leave_type.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveTypeUpdated {
    pub leave_type_id: LeaveTypeId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveTypeUpdated {
    const EVENT_TYPE: &'static str = "hr.leave_type.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveTypeDeleted {
    pub leave_type_id: LeaveTypeId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveTypeDeleted {
    const EVENT_TYPE: &'static str = "hr.leave_type.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// LeaveDefine (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeavePolicyDefined {
    pub leave_define_id: LeaveDefineId,
    pub role_id: Option<RoleId>,
    pub user_id: Option<UserId>,
    pub type_id: LeaveTypeId,
    pub days: u32,
    pub total_days: u32,
    pub academic_id: AcademicYearId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeavePolicyDefined {
    const EVENT_TYPE: &'static str = "hr.leave_policy.defined";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_define";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_define_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_define_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeavePolicyUpdated {
    pub leave_define_id: LeaveDefineId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeavePolicyUpdated {
    const EVENT_TYPE: &'static str = "hr.leave_policy.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_define";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_define_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_define_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeavePolicyDeleted {
    pub leave_define_id: LeaveDefineId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeavePolicyDeleted {
    const EVENT_TYPE: &'static str = "hr.leave_policy.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_define";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_define_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_define_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// LeaveRequest (4 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRequested {
    pub leave_request_id: LeaveRequestId,
    pub staff_id: StaffId,
    pub type_id: LeaveTypeId,
    pub apply_date: NaiveDate,
    pub leave_from: NaiveDate,
    pub leave_to: NaiveDate,
    pub reason: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveRequested {
    const EVENT_TYPE: &'static str = "hr.leave.requested";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_request";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_request_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_request_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveApproved {
    pub leave_request_id: LeaveRequestId,
    pub approver_id: UserId,
    pub approved_at: Timestamp,
    pub note: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveApproved {
    const EVENT_TYPE: &'static str = "hr.leave.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_request";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_request_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_request_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveRejected {
    pub leave_request_id: LeaveRequestId,
    pub rejecter_id: UserId,
    pub reason: String,
    pub rejected_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveRejected {
    const EVENT_TYPE: &'static str = "hr.leave.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_request";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_request_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_request_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveCancelled {
    pub leave_request_id: LeaveRequestId,
    pub canceller_id: UserId,
    pub reason: String,
    pub cancelled_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveCancelled {
    const EVENT_TYPE: &'static str = "hr.leave.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_request";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_request_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_request_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StaffAttendance (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceMarked {
    pub staff_attendance_id: StaffAttendanceId,
    pub staff_id: StaffId,
    pub attendance_date: NaiveDate,
    pub attendance_type: AttendanceType,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub source: AttendanceSource,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffAttendanceMarked {
    const EVENT_TYPE: &'static str = "hr.staff_attendance.marked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hr_staff_attendance";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceUpdated {
    pub staff_attendance_id: StaffAttendanceId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffAttendanceUpdated {
    const EVENT_TYPE: &'static str = "hr.staff_attendance.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hr_staff_attendance";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffAttendanceDeleted {
    pub staff_attendance_id: StaffAttendanceId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffAttendanceDeleted {
    const EVENT_TYPE: &'static str = "hr.staff_attendance.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hr_staff_attendance";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_attendance_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_attendance_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AssignClassTeacher (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassTeacherAssigned {
    pub assign_class_teacher_id: AssignClassTeacherId,
    pub class_id: ClassId,
    pub section_id: SectionId,
    pub staff_id: StaffId,
    pub academic_id: AcademicYearId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for ClassTeacherAssigned {
    const EVENT_TYPE: &'static str = "hr.class_teacher.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_class_teacher";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assign_class_teacher_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assign_class_teacher_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassTeacherAssignmentUpdated {
    pub assign_class_teacher_id: AssignClassTeacherId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for ClassTeacherAssignmentUpdated {
    const EVENT_TYPE: &'static str = "hr.class_teacher.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_class_teacher";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assign_class_teacher_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assign_class_teacher_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassTeacherAssignmentDeleted {
    pub assign_class_teacher_id: AssignClassTeacherId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for ClassTeacherAssignmentDeleted {
    const EVENT_TYPE: &'static str = "hr.class_teacher.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "assign_class_teacher";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.assign_class_teacher_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.assign_class_teacher_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// HourlyRate (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyRateSet {
    pub hourly_rate_id: HourlyRateId,
    pub grade: String,
    pub rate: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for HourlyRateSet {
    const EVENT_TYPE: &'static str = "hr.hourly_rate.set";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hourly_rate";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.hourly_rate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.hourly_rate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyRateUpdated {
    pub hourly_rate_id: HourlyRateId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for HourlyRateUpdated {
    const EVENT_TYPE: &'static str = "hr.hourly_rate.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hourly_rate";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.hourly_rate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.hourly_rate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HourlyRateDeleted {
    pub hourly_rate_id: HourlyRateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for HourlyRateDeleted {
    const EVENT_TYPE: &'static str = "hr.hourly_rate.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "hourly_rate";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.hourly_rate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.hourly_rate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SalaryTemplate (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateCreated {
    pub salary_template_id: SalaryTemplateId,
    pub grade: String,
    pub basic: f64,
    pub gross: f64,
    pub net: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for SalaryTemplateCreated {
    const EVENT_TYPE: &'static str = "hr.salary_template.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateUpdated {
    pub salary_template_id: SalaryTemplateId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for SalaryTemplateUpdated {
    const EVENT_TYPE: &'static str = "hr.salary_template.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalaryTemplateDeleted {
    pub salary_template_id: SalaryTemplateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for SalaryTemplateDeleted {
    const EVENT_TYPE: &'static str = "hr.salary_template.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "salary_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.salary_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.salary_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Payroll (4 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollGenerated {
    pub payroll_generate_id: PayrollGenerateId,
    pub staff_id: StaffId,
    pub payroll_month: u8,
    pub payroll_year: u16,
    pub basic_salary: f64,
    pub total_earning: f64,
    pub total_deduction: f64,
    pub tax: f64,
    pub gross_salary: f64,
    pub net_salary: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollGenerated {
    const EVENT_TYPE: &'static str = "hr.payroll.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_generate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_generate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollApproved {
    pub payroll_generate_id: PayrollGenerateId,
    pub approver_id: UserId,
    pub approved_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollApproved {
    const EVENT_TYPE: &'static str = "hr.payroll.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_generate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_generate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollPaid {
    pub payroll_generate_id: PayrollGenerateId,
    pub paid_amount: f64,
    pub paid_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollPaid {
    const EVENT_TYPE: &'static str = "hr.payroll.paid";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_generate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_generate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollAmountsUpdated {
    pub payroll_generate_id: PayrollGenerateId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollAmountsUpdated {
    const EVENT_TYPE: &'static str = "hr.payroll.amounts_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_generate_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_generate_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// PayrollEarnDeduc (4 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollEarningAdded {
    pub payroll_earn_deduc_id: PayrollEarnDeducId,
    pub payroll_generate_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollEarningAdded {
    const EVENT_TYPE: &'static str = "hr.payroll_earn_deduc.earning_added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_earn_deduc";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_earn_deduc_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_earn_deduc_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollDeductionAdded {
    pub payroll_earn_deduc_id: PayrollEarnDeducId,
    pub payroll_generate_id: PayrollGenerateId,
    pub type_name: String,
    pub amount: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollDeductionAdded {
    const EVENT_TYPE: &'static str = "hr.payroll_earn_deduc.deduction_added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_earn_deduc";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_earn_deduc_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_earn_deduc_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollEarnDeducUpdated {
    pub payroll_earn_deduc_id: PayrollEarnDeducId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollEarnDeducUpdated {
    const EVENT_TYPE: &'static str = "hr.payroll_earn_deduc.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_earn_deduc";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_earn_deduc_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_earn_deduc_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayrollEarnDeducDeleted {
    pub payroll_earn_deduc_id: PayrollEarnDeducId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for PayrollEarnDeducDeleted {
    const EVENT_TYPE: &'static str = "hr.payroll_earn_deduc.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "payroll_earn_deduc";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.payroll_earn_deduc_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.payroll_earn_deduc_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// LeaveDeductionInfo (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDeductionInfoAdded {
    pub leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
    pub staff_id: StaffId,
    pub payroll_id: PayrollGenerateId,
    pub extra_leave: u32,
    pub salary_deduct: f64,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveDeductionInfoAdded {
    const EVENT_TYPE: &'static str = "hr.leave_deduction_info.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_deduction_info";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_deduction_info_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_deduction_info_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDeductionInfoUpdated {
    pub leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveDeductionInfoUpdated {
    const EVENT_TYPE: &'static str = "hr.leave_deduction_info.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_deduction_info";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_deduction_info_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_deduction_info_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaveDeductionInfoDeleted {
    pub leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for LeaveDeductionInfoDeleted {
    const EVENT_TYPE: &'static str = "hr.leave_deduction_info.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "leave_deduction_info";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.leave_deduction_info_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.leave_deduction_info_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// StaffRegistrationField (3 events) + StaffImportBulk (3 events)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffRegistrationFieldCreated {
    pub staff_registration_field_id: crate::value_objects::StaffRegistrationFieldId,
    pub field_name: String,
    pub label_name: String,
    pub is_required: bool,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffRegistrationFieldCreated {
    const EVENT_TYPE: &'static str = "hr.staff_registration_field.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_registration_field";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_registration_field_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_registration_field_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffBulkImported {
    pub staff_import_bulk_temporary_id: crate::value_objects::StaffImportBulkTemporaryId,
    pub staff_no: String,
    pub first_name: String,
    pub last_name: String,
    pub source: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffBulkImported {
    const EVENT_TYPE: &'static str = "hr.staff_import.bulk_imported";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_import_bulk";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_import_bulk_temporary_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_import_bulk_temporary_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaffImportPromoted {
    pub staff_import_bulk_temporary_id: crate::value_objects::StaffImportBulkTemporaryId,
    pub staff_id: StaffId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DomainEvent for StaffImportPromoted {
    const EVENT_TYPE: &'static str = "hr.staff_import.promoted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "staff_import_bulk";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.staff_import_bulk_temporary_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.staff_import_bulk_temporary_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::{CorrelationId, SchoolId, UserId};
    use educore_core::value_objects::Timestamp;

    fn g() -> SystemIdGen {
        SystemIdGen
    }

    #[test]
    fn staff_registered_event_round_trip() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let id = StaffId::new(s, uuid::Uuid::now_v7());
        let ev = StaffRegistered::new(
            id,
            1,
            "E001".to_owned(),
            "Alice".to_owned(),
            "Patel".to_owned(),
            None,
            None,
            None,
            None,
            RoleId::new(s, uuid::Uuid::now_v7()),
            UserId(uuid::Uuid::now_v7()),
            chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            g().next_event_id(),
            CorrelationId(uuid::Uuid::now_v7()),
            Timestamp::now(),
        );
        assert_eq!(ev.aggregate_id(), id.as_uuid());
        assert_eq!(ev.school_id(), s);
        assert_eq!(
            <StaffRegistered as DomainEvent>::EVENT_TYPE,
            "hr.staff.registered"
        );
        assert_eq!(<StaffRegistered as DomainEvent>::AGGREGATE_TYPE, "staff");
    }

    #[test]
    fn leave_requested_event_has_correct_wire_form() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let id = LeaveRequestId::new(s, uuid::Uuid::now_v7());
        let ev = LeaveRequested {
            leave_request_id: id,
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            type_id: LeaveTypeId::new(s, uuid::Uuid::now_v7()),
            apply_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_to: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: None,
            event_id: g().next_event_id(),
            correlation_id: CorrelationId(uuid::Uuid::now_v7()),
            occurred_at: Timestamp::now(),
        };
        assert_eq!(
            <LeaveRequested as DomainEvent>::EVENT_TYPE,
            "hr.leave.requested"
        );
        assert_eq!(ev.aggregate_id(), id.as_uuid());
    }

    #[test]
    fn payroll_generated_event_wire_form() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let id = PayrollGenerateId::new(s, uuid::Uuid::now_v7());
        let ev = PayrollGenerated {
            payroll_generate_id: id,
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            payroll_month: 6,
            payroll_year: 2026,
            basic_salary: 50000.0,
            total_earning: 50000.0,
            total_deduction: 5000.0,
            tax: 5000.0,
            gross_salary: 50000.0,
            net_salary: 45000.0,
            event_id: g().next_event_id(),
            correlation_id: CorrelationId(uuid::Uuid::now_v7()),
            occurred_at: Timestamp::now(),
        };
        assert_eq!(
            <PayrollGenerated as DomainEvent>::EVENT_TYPE,
            "hr.payroll.generated"
        );
    }
}

// =============================================================================
// New constructors (Phase 6 — appended after the macro-generated events
// so the service layer can call `LeaveTypeCreated::new(...)` etc.)
// =============================================================================

impl LeaveTypeCreated {
    pub fn new(
        leave_type_id: LeaveTypeId,
        type_name: String,
        total_days: u32,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_type_id,
            type_name,
            total_days,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveTypeUpdated {
    pub fn new(
        leave_type_id: LeaveTypeId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_type_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveTypeDeleted {
    pub fn new(
        leave_type_id: LeaveTypeId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_type_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeavePolicyDefined {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        leave_define_id: LeaveDefineId,
        role_id: Option<RoleId>,
        user_id: Option<UserId>,
        type_id: LeaveTypeId,
        days: u32,
        total_days: u32,
        academic_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_define_id,
            role_id,
            user_id,
            type_id,
            days,
            total_days,
            academic_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeavePolicyUpdated {
    pub fn new(
        leave_define_id: LeaveDefineId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_define_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeavePolicyDeleted {
    pub fn new(
        leave_define_id: LeaveDefineId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_define_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveRequested {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        leave_request_id: LeaveRequestId,
        staff_id: StaffId,
        type_id: LeaveTypeId,
        apply_date: NaiveDate,
        leave_from: NaiveDate,
        leave_to: NaiveDate,
        reason: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_request_id,
            staff_id,
            type_id,
            apply_date,
            leave_from,
            leave_to,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveApproved {
    pub fn new(
        leave_request_id: LeaveRequestId,
        approver_id: UserId,
        approved_at: Timestamp,
        note: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_request_id,
            approver_id,
            approved_at,
            note,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveRejected {
    pub fn new(
        leave_request_id: LeaveRequestId,
        rejecter_id: UserId,
        reason: String,
        rejected_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_request_id,
            rejecter_id,
            reason,
            rejected_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveCancelled {
    pub fn new(
        leave_request_id: LeaveRequestId,
        canceller_id: UserId,
        reason: String,
        cancelled_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_request_id,
            canceller_id,
            reason,
            cancelled_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffAttendanceMarked {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        staff_id: StaffId,
        attendance_date: NaiveDate,
        attendance_type: AttendanceType,
        in_time: Option<String>,
        out_time: Option<String>,
        source: AttendanceSource,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_attendance_id,
            staff_id,
            attendance_date,
            attendance_type,
            in_time,
            out_time,
            source,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffAttendanceUpdated {
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_attendance_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffAttendanceDeleted {
    pub fn new(
        staff_attendance_id: StaffAttendanceId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_attendance_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl ClassTeacherAssigned {
    pub fn new(
        assign_class_teacher_id: AssignClassTeacherId,
        class_id: ClassId,
        section_id: SectionId,
        staff_id: StaffId,
        academic_id: AcademicYearId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_class_teacher_id,
            class_id,
            section_id,
            staff_id,
            academic_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl ClassTeacherAssignmentUpdated {
    pub fn new(
        assign_class_teacher_id: AssignClassTeacherId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_class_teacher_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl ClassTeacherAssignmentDeleted {
    pub fn new(
        assign_class_teacher_id: AssignClassTeacherId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            assign_class_teacher_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl HourlyRateSet {
    pub fn new(
        hourly_rate_id: HourlyRateId,
        grade: String,
        rate: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            hourly_rate_id,
            grade,
            rate,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl HourlyRateUpdated {
    pub fn new(
        hourly_rate_id: HourlyRateId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            hourly_rate_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl HourlyRateDeleted {
    pub fn new(
        hourly_rate_id: HourlyRateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            hourly_rate_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl SalaryTemplateCreated {
    pub fn new(
        salary_template_id: SalaryTemplateId,
        grade: String,
        basic: f64,
        gross: f64,
        net: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            grade,
            basic,
            gross,
            net,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl SalaryTemplateUpdated {
    pub fn new(
        salary_template_id: SalaryTemplateId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl SalaryTemplateDeleted {
    pub fn new(
        salary_template_id: SalaryTemplateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            salary_template_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollGenerated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payroll_generate_id: PayrollGenerateId,
        staff_id: StaffId,
        payroll_month: u8,
        payroll_year: u16,
        basic_salary: f64,
        total_earning: f64,
        total_deduction: f64,
        tax: f64,
        gross_salary: f64,
        net_salary: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_generate_id,
            staff_id,
            payroll_month,
            payroll_year,
            basic_salary,
            total_earning,
            total_deduction,
            tax,
            gross_salary,
            net_salary,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollApproved {
    pub fn new(
        payroll_generate_id: PayrollGenerateId,
        approver_id: UserId,
        approved_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_generate_id,
            approver_id,
            approved_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollPaid {
    pub fn new(
        payroll_generate_id: PayrollGenerateId,
        paid_amount: f64,
        paid_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_generate_id,
            paid_amount,
            paid_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollAmountsUpdated {
    pub fn new(
        payroll_generate_id: PayrollGenerateId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_generate_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollEarningAdded {
    pub fn new(
        payroll_earn_deduc_id: PayrollEarnDeducId,
        payroll_generate_id: PayrollGenerateId,
        type_name: String,
        amount: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_earn_deduc_id,
            payroll_generate_id,
            type_name,
            amount,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollDeductionAdded {
    pub fn new(
        payroll_earn_deduc_id: PayrollEarnDeducId,
        payroll_generate_id: PayrollGenerateId,
        type_name: String,
        amount: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_earn_deduc_id,
            payroll_generate_id,
            type_name,
            amount,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollEarnDeducUpdated {
    pub fn new(
        payroll_earn_deduc_id: PayrollEarnDeducId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_earn_deduc_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl PayrollEarnDeducDeleted {
    pub fn new(
        payroll_earn_deduc_id: PayrollEarnDeducId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            payroll_earn_deduc_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveDeductionInfoAdded {
    pub fn new(
        leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
        staff_id: StaffId,
        payroll_id: PayrollGenerateId,
        extra_leave: u32,
        salary_deduct: f64,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_deduction_info_id,
            staff_id,
            payroll_id,
            extra_leave,
            salary_deduct,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveDeductionInfoUpdated {
    pub fn new(
        leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_deduction_info_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl LeaveDeductionInfoDeleted {
    pub fn new(
        leave_deduction_info_id: crate::value_objects::LeaveDeductionInfoId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            leave_deduction_info_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffRegistrationFieldCreated {
    pub fn new(
        staff_registration_field_id: crate::value_objects::StaffRegistrationFieldId,
        field_name: String,
        label_name: String,
        is_required: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_registration_field_id,
            field_name,
            label_name,
            is_required,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffBulkImported {
    pub fn new(
        staff_import_bulk_temporary_id: crate::value_objects::StaffImportBulkTemporaryId,
        staff_no: String,
        first_name: String,
        last_name: String,
        source: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_import_bulk_temporary_id,
            staff_no,
            first_name,
            last_name,
            source,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl StaffImportPromoted {
    pub fn new(
        staff_import_bulk_temporary_id: crate::value_objects::StaffImportBulkTemporaryId,
        staff_id: StaffId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            staff_import_bulk_temporary_id,
            staff_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}
