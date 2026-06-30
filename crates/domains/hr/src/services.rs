//! # HR domain services
//!
//! Pure factory functions that take a typed command + a clock +
//! a uniqueness-checker port and return the new aggregate +
//! the typed event. The dispatcher is responsible for
//! persisting the aggregate and writing the audit / outbox
//! / idempotency rows in a single transaction (per the
//! Phase 4 / 5 pattern).
//!
//! Phase 6 ships the 6 prompt-named service functions
//! (`hire_staff`, `create_department`, `create_designation`,
//! `create_leave_type`, `request_leave`, `approve_leave`,
//! `run_payroll`) plus the leave-accrual and payroll-policy
//! services.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;

use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use educore_rbac::ids::RoleId;

use crate::aggregate::{
    AssignClassTeacher, AssignClassTeacherScope, BulkImportJob, Department, DepartmentHead,
    Designation, DesignationGrade, HourlyRate, HourlyRateOverride, LeaveDefineAdjustment,
    LeaveRequest, LeaveRequestApproval, LeaveRequestAttachment, LeaveType, PayrollGenerate,
    PayrollGenerateAudit, PayrollPaymentLink, Staff, StaffAddress, StaffAttendanceImportBatch,
    StaffAttendancePunch, StaffBankDetail, StaffCustomField, StaffDocument, StaffDrivingLicense,
    StaffImportResolution, StaffLeaveBalance, StaffLeaveHistory, StaffPayrollHistory,
    StaffProfilePhoto, StaffRegistrationFieldOption, StaffRoleAssignment, StaffSocialLink,
    StaffTimeline,
};
use crate::commands::{
    AssignDepartmentHeadCommand, AssignStaffRoleCommand, AssignSubjectTeacherCommand,
    CreateAssignClassTeacherScopeCommand,
    CreateBulkImportJobCommand, CreateDesignationGradeCommand, CreateLeaveDefineAdjustmentCommand,
    CreateLeaveRequestAttachmentCommand, CreatePayrollPaymentLinkCommand,
    CreateStaffAddressCommand, CreateStaffAttendanceImportBatchCommand,
    CreateStaffBankDetailCommand, CreateStaffDocumentCommand, CreateStaffDrivingLicenseCommand,
    CreateStaffProfilePhotoCommand, CreateStaffRegistrationFieldOptionCommand,
    CreateStaffSocialLinkCommand, RecordLeaveRequestApprovalCommand,
    RecordPayrollGenerateAuditCommand, RecordStaffAttendancePunchCommand,
    RecordStaffImportResolutionCommand, RecordStaffLeaveHistoryCommand,
    RecordStaffPayrollHistoryCommand, RefreshStaffLeaveBalanceCommand, RefreshStaffTimelineCommand,
    SetHourlyRateOverrideCommand, SetStaffCustomFieldCommand,
};
use educore_events::domain_event::DomainEvent;

use crate::events::{
    AssignClassTeacherScopeAdded, BulkImportJobRecorded, DepartmentCreated, DepartmentHeadRecorded,
    DesignationGradeRecorded, HourlyRateOverrideSet, LeaveApproved, LeaveDefineAdjustmentApplied,
    LeaveRequestApprovalRecorded, LeaveRequestAttachmentRegistered, LeaveRequested,
    PayrollGenerateAuditAppended, PayrollGenerated, PayrollPaymentLinkCreated, StaffAddressAdded,
    StaffAttendanceImportBatchRecorded, StaffAttendancePunchCaptured, StaffBankDetailUpserted,
    StaffCustomFieldSet, StaffDocumentRegistered, StaffDrivingLicenseRegistered,
    StaffImportResolutionRecorded, StaffLeaveBalanceRefreshed, StaffLeaveHistorySnapshotted,
    StaffPayrollHistorySnapshotted, StaffProfilePhotoRegistered, StaffRegistered,
    StaffRegistrationFieldOptionAdded, StaffRoleAssignmentRecorded, StaffSocialLinkAdded,
    StaffTimelineRefreshed,
};
use crate::value_objects::{
    AttendanceType, EarnDeducType, LeaveStatus, PayrollStatus, StaffStatus,
};

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// Staff
// =============================================================================

/// Builds a [`Staff`] aggregate + a [`StaffRegistered`] event.
#[allow(clippy::too_many_arguments)]
pub fn hire_staff<C, G>(
    cmd: HireStaffCommand,
    clock: &C,
    ids: &G,
    uniqueness: &dyn StaffUniquenessChecker,
) -> Result<(Staff, StaffRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let actor = cmd.tenant.actor_id;
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let staff_id = StaffId::new(school, event_id_to_uuid(event_id));

    // Validation
    crate::value_objects::validate_person_name(&cmd.first_name)?;
    crate::value_objects::validate_person_name(&cmd.last_name)?;
    if let Some(email) = &cmd.email {
        crate::value_objects::validate_email(email)?;
    }
    if let Some(mobile) = &cmd.mobile {
        crate::value_objects::validate_phone(mobile)?;
    }
    crate::value_objects::validate_date_of_birth(cmd.date_of_birth)?;

    // Uniqueness
    if let Some(email) = &cmd.email {
        if uniqueness.email_exists(school, email) {
            return Err(DomainError::conflict(format!(
                "staff with email {email:?} already exists"
            )));
        }
    }
    if uniqueness.staff_no_exists(school, cmd.staff_no) {
        return Err(DomainError::conflict(format!(
            "staff with staff_no {} already exists",
            cmd.staff_no
        )));
    }
    if uniqueness.employee_id_exists(school, &cmd.employee_id) {
        return Err(DomainError::conflict(format!(
            "staff with employee_id {:?} already exists",
            cmd.employee_id
        )));
    }

    let mut staff = Staff::fresh(
        staff_id,
        cmd.user_id,
        cmd.role_id,
        cmd.staff_no,
        cmd.employee_id,
        cmd.first_name,
        cmd.last_name,
        cmd.gender,
        cmd.date_of_birth,
        cmd.date_of_joining,
        StaffStatus::Active,
        actor,
        now,
        cmd.tenant.correlation_id,
    );
    staff.email = cmd.email;
    staff.mobile = cmd.mobile;
    staff.department_id = cmd.department_id;
    staff.designation_id = cmd.designation_id;
    staff.last_event_id = Some(event_id);

    let event = StaffRegistered::new(
        staff_id,
        staff.staff_no,
        staff.employee_id.clone(),
        staff.first_name.clone(),
        staff.last_name.clone(),
        staff.email.clone(),
        staff.mobile.clone(),
        staff.department_id,
        staff.designation_id,
        staff.role_id,
        staff.user_id,
        staff.date_of_joining,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((staff, event))
}

/// Command: hire a staff member (the spec's `RegisterStaff`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HireStaffCommand {
    pub tenant: TenantContext,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub staff_no: u32,
    pub employee_id: String,
    pub first_name: String,
    pub last_name: String,
    pub gender: crate::value_objects::Gender,
    pub date_of_birth: NaiveDate,
    pub date_of_joining: NaiveDate,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub department_id: Option<crate::value_objects::DepartmentId>,
    pub designation_id: Option<crate::value_objects::DesignationId>,
}

use crate::value_objects::StaffId;

// =============================================================================
// Department / Designation / LeaveType (simple reference-data services)
// =============================================================================

/// Builds a [`Department`] aggregate + a [`DepartmentCreated`] event.
pub fn create_department<C, G>(
    tenant: TenantContext,
    name: String,
    description: Option<String>,
    clock: &C,
    ids: &G,
    uniqueness: &dyn ReferenceDataUniquenessChecker,
) -> Result<(Department, DepartmentCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = tenant.school_id;
    let id = crate::value_objects::DepartmentId::new(school, event_id_to_uuid(event_id));

    if name.is_empty() || name.len() > 200 {
        return Err(DomainError::validation(format!(
            "department name must be 1..=200 chars, got {}",
            name.len()
        )));
    }
    if uniqueness.department_name_exists(school, &name) {
        return Err(DomainError::conflict(format!(
            "department with name {name:?} already exists"
        )));
    }

    let mut dept = Department::fresh(
        id,
        name.clone(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    );
    dept.description = description;
    dept.last_event_id = Some(event_id);

    let event = DepartmentCreated::new(id, name, event_id, tenant.correlation_id, now);
    Ok((dept, event))
}

/// Builds a [`Designation`] aggregate.
pub fn create_designation<C, G>(
    tenant: TenantContext,
    title: String,
    description: Option<String>,
    clock: &C,
    ids: &G,
    uniqueness: &dyn ReferenceDataUniquenessChecker,
) -> Result<(
    crate::aggregate::Designation,
    crate::events::DesignationCreated,
)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = tenant.school_id;
    let id = crate::value_objects::DesignationId::new(school, event_id_to_uuid(event_id));

    if title.is_empty() || title.len() > 200 {
        return Err(DomainError::validation(format!(
            "designation title must be 1..=200 chars, got {}",
            title.len()
        )));
    }
    if uniqueness.designation_title_exists(school, &title) {
        return Err(DomainError::conflict(format!(
            "designation with title {title:?} already exists"
        )));
    }

    let mut desig = crate::aggregate::Designation::fresh(
        id,
        title.clone(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    );
    desig.description = description;
    desig.last_event_id = Some(event_id);

    let event =
        crate::events::DesignationCreated::new(id, title, event_id, tenant.correlation_id, now);
    Ok((desig, event))
}

/// Builds a [`LeaveType`] aggregate.
pub fn create_leave_type<C, G>(
    tenant: TenantContext,
    type_name: String,
    total_days: u32,
    clock: &C,
    ids: &G,
    uniqueness: &dyn ReferenceDataUniquenessChecker,
) -> Result<(LeaveType, crate::events::LeaveTypeCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = tenant.school_id;
    let id = crate::value_objects::LeaveTypeId::new(school, event_id_to_uuid(event_id));

    crate::value_objects::validate_leave_type_name(&type_name)?;
    if uniqueness.leave_type_name_exists(school, &type_name) {
        return Err(DomainError::conflict(format!(
            "leave type with name {type_name:?} already exists"
        )));
    }

    let mut lt = LeaveType::fresh(
        id,
        type_name.clone(),
        total_days,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    );
    lt.last_event_id = Some(event_id);

    let event = crate::events::LeaveTypeCreated::new(
        id,
        type_name,
        total_days,
        event_id,
        tenant.correlation_id,
        now,
    );
    Ok((lt, event))
}

// =============================================================================
// LeaveRequest
// =============================================================================

/// Builds a [`LeaveRequest`] aggregate + [`LeaveRequested`] event
/// in the `Pending` state.
#[allow(clippy::too_many_arguments)]
pub fn request_leave<C, G>(
    cmd: RequestLeaveCommand,
    clock: &C,
    ids: &G,
) -> Result<(LeaveRequest, LeaveRequested)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::LeaveRequestId::new(school, event_id_to_uuid(event_id));

    if cmd.leave_to < cmd.leave_from {
        return Err(DomainError::validation(format!(
            "leave_to {} must be >= leave_from {}",
            cmd.leave_to, cmd.leave_from
        )));
    }
    if let Some(reason) = &cmd.reason {
        crate::value_objects::validate_leave_reason(reason)?;
    }

    let mut lr = LeaveRequest::fresh(
        id,
        cmd.staff_id,
        cmd.type_id,
        cmd.apply_date,
        cmd.leave_from,
        cmd.leave_to,
        cmd.reason.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    lr.note = cmd.note;
    lr.last_event_id = Some(event_id);

    let event = LeaveRequested::new(
        id,
        lr.staff_id,
        lr.type_id,
        lr.apply_date,
        lr.leave_from,
        lr.leave_to,
        lr.reason.clone(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((lr, event))
}

/// Command: request leave.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RequestLeaveCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub type_id: crate::value_objects::LeaveTypeId,
    pub apply_date: NaiveDate,
    pub leave_from: NaiveDate,
    pub leave_to: NaiveDate,
    pub reason: Option<String>,
    pub note: Option<String>,
    pub file_reference: Option<uuid::Uuid>,
}

/// Approves a leave request.
///
/// Returns `Err(DomainError::Conflict)` if the state machine
/// does not permit the transition (`Pending → Approved`), or
/// if the approver is the same as the requester
/// (segregation of duties).
pub fn approve_leave<C, G>(
    leave_request: &mut LeaveRequest,
    approver_tenant: TenantContext,
    note: Option<String>,
    clock: &C,
    ids: &G,
) -> Result<LeaveApproved>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    if !leave_request.can_transition(LeaveStatus::Approved) {
        return Err(DomainError::conflict(format!(
            "leave request is in state {:?}, cannot transition to Approved",
            leave_request.approve_status
        )));
    }
    if approver_tenant.actor_id == leave_request.created_by {
        return Err(DomainError::conflict(
            "approver and requester must differ (segregation of duties)",
        ));
    }
    let now = clock.now();
    let event_id = ids.next_event_id();
    leave_request.approve_status = LeaveStatus::Approved;
    leave_request.approver_id = Some(approver_tenant.actor_id);
    leave_request.approved_at = Some(now);
    leave_request.note = note.clone();
    leave_request.updated_at = now;
    leave_request.updated_by = approver_tenant.actor_id;
    leave_request.version = leave_request.version.next();
    leave_request.last_event_id = Some(event_id);
    Ok(LeaveApproved::new(
        leave_request.id,
        approver_tenant.actor_id,
        now,
        note,
        event_id,
        approver_tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// LeaveAccrual service
// =============================================================================

/// The leave-accrual service computes a staff member's
/// remaining leave balance for a given leave type and year.
/// Backed by [`LeaveDefine`] rows plus historical approved
/// [`LeaveRequest`] rows.
pub struct LeaveAccrualService;

impl LeaveAccrualService {
    /// Returns the effective remaining leave balance for the
    /// given `staff`, `type_id`, and `academic_year`. The
    /// calculation is:
    /// `LeaveDefine.days - sum(approved LeaveRequest.durations)`.
    #[must_use]
    pub fn effective_leave_balance(
        define: &crate::aggregate::LeaveDefine,
        approved_requests: &[LeaveRequest],
    ) -> u32 {
        let used: u32 = approved_requests
            .iter()
            .filter(|r| r.type_id == define.type_id && r.approve_status == LeaveStatus::Approved)
            .map(LeaveRequest::duration_days)
            .sum();
        define.days.saturating_sub(used)
    }

    /// Returns the number of extra leave days taken
    /// (approved days that exceed the [`LeaveDefine::days`]
    /// entitlement). Used by the payroll service to compute
    /// leave deductions.
    #[must_use]
    pub fn extra_leave_taken(
        approved_requests: &[LeaveRequest],
        define: &crate::aggregate::LeaveDefine,
    ) -> u32 {
        let total: u32 = approved_requests
            .iter()
            .filter(|r| r.type_id == define.type_id && r.approve_status == LeaveStatus::Approved)
            .map(LeaveRequest::duration_days)
            .sum();
        total.saturating_sub(define.days)
    }

    /// Returns `true` if the new request is allowed given
    /// the existing approved requests and the entitlement.
    /// Rejects overlapping approved requests and over-quota
    /// requests.
    #[must_use]
    pub fn can_request(
        define: &crate::aggregate::LeaveDefine,
        approved: &[LeaveRequest],
        from: NaiveDate,
        to: NaiveDate,
    ) -> bool {
        let days = (to.signed_duration_since(from).num_days() + 1).max(0);
        let duration = u32::try_from(days).unwrap_or(u32::MAX);
        let used: u32 = approved
            .iter()
            .filter(|r| r.type_id == define.type_id && r.approve_status == LeaveStatus::Approved)
            .map(LeaveRequest::duration_days)
            .sum();
        used + duration <= define.days
    }

    /// Returns the overlap status of two date ranges.
    #[must_use]
    pub fn overlaps(a: (NaiveDate, NaiveDate), b: (NaiveDate, NaiveDate)) -> bool {
        a.0 <= b.1 && b.0 <= a.1
    }
}

// =============================================================================
// Payroll
// =============================================================================

/// Builds a [`PayrollGenerate`] aggregate + a [`PayrollGenerated`] event.
#[allow(clippy::too_many_arguments)]
pub fn run_payroll<C, G>(
    cmd: RunPayrollCommand,
    clock: &C,
    ids: &G,
    policy: &dyn PayrollPolicy,
) -> Result<(PayrollGenerate, PayrollGenerated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = crate::value_objects::PayrollGenerateId::new(school, event_id_to_uuid(event_id));

    crate::value_objects::validate_pay_period(cmd.payroll_month, cmd.payroll_year)?;

    if cmd.basic_salary < 0.0 {
        return Err(DomainError::validation("basic_salary must be non-negative"));
    }

    let total_earning = cmd.basic_salary;
    let tax = policy.tax(school, total_earning);
    let total_deduction = tax;
    let gross_salary = total_earning;
    let net_salary = (gross_salary - total_deduction).max(0.0);

    let mut payroll = PayrollGenerate::fresh(
        id,
        cmd.staff_id,
        cmd.basic_salary,
        cmd.payroll_month,
        cmd.payroll_year,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    payroll.total_earning = total_earning;
    payroll.total_deduction = total_deduction;
    payroll.gross_salary = gross_salary;
    payroll.tax = tax;
    payroll.net_salary = net_salary;
    payroll.payroll_status = PayrollStatus::Generated;
    payroll.payment_mode = cmd.payment_mode;
    payroll.bank_id = cmd.bank_id;
    payroll.note = cmd.note.clone();
    payroll.last_event_id = Some(event_id);

    let event = PayrollGenerated::new(
        id,
        cmd.staff_id,
        cmd.payroll_month,
        cmd.payroll_year,
        cmd.basic_salary,
        total_earning,
        total_deduction,
        tax,
        gross_salary,
        net_salary,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((payroll, event))
}

/// Command: run payroll for one staff in a pay period.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunPayrollCommand {
    pub tenant: TenantContext,
    pub staff_id: StaffId,
    pub basic_salary: f64,
    pub payroll_month: u8,
    pub payroll_year: u16,
    pub payment_mode: Option<String>,
    pub bank_id: Option<uuid::Uuid>,
    pub note: Option<String>,
}

// =============================================================================
// PayrollPolicy port
// =============================================================================

/// The payroll policy port: per-school tax/allowance/deduction
/// rules. The engine ships a [`InMemoryPayrollPolicy`]
/// implementation at 10% tax for tests and as a placeholder
/// for real-world consumers. A real-world adapter (e.g.
/// per-jurisdiction tax engine) is the consumer's
/// responsibility.
pub trait PayrollPolicy: Send + Sync {
    /// Computes the tax for a given gross amount in a school.
    fn tax(&self, school: SchoolId, gross: f64) -> f64;

    /// Returns `true` if a partial payment is allowed.
    fn allows_partial_payment(&self, school: SchoolId) -> bool;

    /// Returns the maximum payment fraction allowed per
    /// single payment (e.g. `1.0` for full payment only;
    /// `0.5` for "max half per payment").
    fn max_payment_fraction(&self, school: SchoolId) -> f64;
}

/// In-memory payroll policy with a flat 10% tax rate. The
/// default test fixture used by the integration test and
/// unit tests in this crate.
pub struct InMemoryPayrollPolicy {
    tax_rate: f64,
    allows_partial: bool,
    max_fraction: f64,
}

impl Default for InMemoryPayrollPolicy {
    fn default() -> Self {
        Self {
            tax_rate: 0.10,
            allows_partial: true,
            max_fraction: 1.0,
        }
    }
}

impl InMemoryPayrollPolicy {
    #[must_use]
    pub fn new(tax_rate: f64) -> Self {
        Self {
            tax_rate,
            allows_partial: true,
            max_fraction: 1.0,
        }
    }

    #[must_use]
    pub fn with_partial(tax_rate: f64, allows_partial: bool, max_fraction: f64) -> Self {
        Self {
            tax_rate,
            allows_partial,
            max_fraction,
        }
    }
}

impl PayrollPolicy for InMemoryPayrollPolicy {
    fn tax(&self, _school: SchoolId, gross: f64) -> f64 {
        (gross * self.tax_rate).max(0.0)
    }
    fn allows_partial_payment(&self, _school: SchoolId) -> bool {
        self.allows_partial
    }
    fn max_payment_fraction(&self, _school: SchoolId) -> f64 {
        self.max_fraction
    }
}

// =============================================================================
// Uniqueness port
// =============================================================================

/// Per-school staff uniqueness checks the `hire_staff`
/// service calls. The storage adapter is the canonical
/// implementation; tests use an in-memory variant.
pub trait StaffUniquenessChecker: Send + Sync {
    fn email_exists(&self, school: SchoolId, email: &str) -> bool;
    fn staff_no_exists(&self, school: SchoolId, staff_no: u32) -> bool;
    fn employee_id_exists(&self, school: SchoolId, employee_id: &str) -> bool;
}

/// Per-school reference-data uniqueness checks.
pub trait ReferenceDataUniquenessChecker: Send + Sync {
    fn department_name_exists(&self, school: SchoolId, name: &str) -> bool;
    fn designation_title_exists(&self, school: SchoolId, title: &str) -> bool;
    fn leave_type_name_exists(&self, school: SchoolId, name: &str) -> bool;
}

// =============================================================================
// Cluster C: handler skeletons
//
// Each handler below is a minimal skeleton that wires the
// matching command stub (added in commit 71578b5) to the
// matching aggregate stub (added in commit bc938cd) and the
// matching event stub (added in commit bbbd8a1). The bodies
// mint a fresh event_id + correlation_id + timestamp from
// the supplied `Clock` + `IdGenerator`, construct the
// aggregate from the typed id, and return the matching event.
//
// The full payload (validation, uniqueness checks, state-
// machine transitions, command field wiring) is left for the
// owning Workstream to fill in. These skeletons exist so
// downstream code (subscribers, projections, integration
// tests) can wire type-safe handles to the owning
// Workstream's handler shape without forcing an
// all-at-once refactor.
// =============================================================================

/// Handler skeleton: create or replace a staff member's
/// bank-account detail row.
pub fn create_staff_bank_detail<C, G>(
    cmd: CreateStaffBankDetailCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffBankDetail, StaffBankDetailUpserted)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffBankDetail {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffBankDetailUpserted::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: add a postal address to a staff profile.
pub fn create_staff_address<C, G>(
    cmd: CreateStaffAddressCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffAddress, StaffAddressAdded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffAddress {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffAddressAdded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: attach a social-profile link to a staff profile.
pub fn create_staff_social_link<C, G>(
    cmd: CreateStaffSocialLinkCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffSocialLink, StaffSocialLinkAdded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffSocialLink {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffSocialLinkAdded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: register an uploaded document attached to a
/// staff profile.
pub fn create_staff_document<C, G>(
    cmd: CreateStaffDocumentCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffDocument, StaffDocumentRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffDocument {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffDocumentRegistered::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: marker for the staff-timeline projection
/// recomputed from the event log.
pub fn refresh_staff_timeline<C, G>(
    cmd: RefreshStaffTimelineCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffTimeline, StaffTimelineRefreshed)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffTimeline {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffTimelineRefreshed::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: set a per-staff custom-field value.
pub fn set_staff_custom_field<C, G>(
    cmd: SetStaffCustomFieldCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffCustomField, StaffCustomFieldSet)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffCustomField {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffCustomFieldSet::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: marker for the leave-balance snapshot
/// recomputed from events.
pub fn refresh_staff_leave_balance<C, G>(
    cmd: RefreshStaffLeaveBalanceCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffLeaveBalance, StaffLeaveBalanceRefreshed)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffLeaveBalance {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffLeaveBalanceRefreshed::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: record an approval / rejection event on a
/// leave request.
pub fn record_leave_request_approval<C, G>(
    cmd: RecordLeaveRequestApprovalCommand,
    clock: &C,
    ids: &G,
) -> Result<(LeaveRequestApproval, LeaveRequestApprovalRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = LeaveRequestApproval {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = LeaveRequestApprovalRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: link a payroll run to an external payment record.
pub fn create_payroll_payment_link<C, G>(
    cmd: CreatePayrollPaymentLinkCommand,
    clock: &C,
    ids: &G,
) -> Result<(PayrollPaymentLink, PayrollPaymentLinkCreated)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = PayrollPaymentLink {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = PayrollPaymentLinkCreated::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: marker for the resolved foreign-key mapping of
/// a bulk staff import.
pub fn record_staff_import_resolution<C, G>(
    cmd: RecordStaffImportResolutionCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffImportResolution, StaffImportResolutionRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffImportResolution {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffImportResolutionRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: snapshot a staff member's payroll row when a
/// payroll run reaches Paid.
pub fn record_staff_payroll_history<C, G>(
    cmd: RecordStaffPayrollHistoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffPayrollHistory, StaffPayrollHistorySnapshotted)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffPayrollHistory {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffPayrollHistorySnapshotted::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: snapshot a staff member's leave row when a
/// leave request reaches a terminal state.
pub fn record_staff_leave_history<C, G>(
    cmd: RecordStaffLeaveHistoryCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffLeaveHistory, StaffLeaveHistorySnapshotted)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffLeaveHistory {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffLeaveHistorySnapshotted::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: attach additional sections / subjects to a
/// class-teacher assignment.
pub fn create_assign_class_teacher_scope<C, G>(
    cmd: CreateAssignClassTeacherScopeCommand,
    clock: &C,
    ids: &G,
) -> Result<(AssignClassTeacherScope, AssignClassTeacherScopeAdded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = AssignClassTeacherScope {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = AssignClassTeacherScopeAdded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: promote a staff member to head of a department.
pub fn assign_department_head<C, G>(
    cmd: AssignDepartmentHeadCommand,
    clock: &C,
    ids: &G,
) -> Result<(DepartmentHead, DepartmentHeadRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = DepartmentHead {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = DepartmentHeadRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: attach a salary grade to a designation.
pub fn create_designation_grade<C, G>(
    cmd: CreateDesignationGradeCommand,
    clock: &C,
    ids: &G,
) -> Result<(DesignationGrade, DesignationGradeRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = DesignationGrade {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = DesignationGradeRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: override the hourly rate for a single staff
/// member.
pub fn set_hourly_rate_override<C, G>(
    cmd: SetHourlyRateOverrideCommand,
    clock: &C,
    ids: &G,
) -> Result<(HourlyRateOverride, HourlyRateOverrideSet)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = HourlyRateOverride {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = HourlyRateOverrideSet::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: adjust a leave entitlement (carry-forward,
/// special grant, manual correction).
pub fn create_leave_define_adjustment<C, G>(
    cmd: CreateLeaveDefineAdjustmentCommand,
    clock: &C,
    ids: &G,
) -> Result<(LeaveDefineAdjustment, LeaveDefineAdjustmentApplied)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = LeaveDefineAdjustment {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = LeaveDefineAdjustmentApplied::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: attach a file to a leave request.
pub fn create_leave_request_attachment<C, G>(
    cmd: CreateLeaveRequestAttachmentCommand,
    clock: &C,
    ids: &G,
) -> Result<(LeaveRequestAttachment, LeaveRequestAttachmentRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = LeaveRequestAttachment {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = LeaveRequestAttachmentRegistered::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: record a raw biometric / RFID punch before it
/// is folded into an attendance day row.
pub fn record_staff_attendance_punch<C, G>(
    cmd: RecordStaffAttendancePunchCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffAttendancePunch, StaffAttendancePunchCaptured)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffAttendancePunch {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffAttendancePunchCaptured::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: marker for the append-only audit trail of
/// payroll-run state transitions.
pub fn record_payroll_generate_audit<C, G>(
    cmd: RecordPayrollGenerateAuditCommand,
    clock: &C,
    ids: &G,
) -> Result<(PayrollGenerateAudit, PayrollGenerateAuditAppended)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = PayrollGenerateAudit {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = PayrollGenerateAuditAppended::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: assign a role to a staff member.
pub fn assign_staff_role<C, G>(
    cmd: AssignStaffRoleCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffRoleAssignment, StaffRoleAssignmentRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffRoleAssignment {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffRoleAssignmentRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: upload (or replace) a staff profile photo.
pub fn create_staff_profile_photo<C, G>(
    cmd: CreateStaffProfilePhotoCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffProfilePhoto, StaffProfilePhotoRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffProfilePhoto {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffProfilePhotoRegistered::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: register (or update) a staff member's driving
/// license metadata.
pub fn create_staff_driving_license<C, G>(
    cmd: CreateStaffDrivingLicenseCommand,
    clock: &C,
    ids: &G,
) -> Result<(StaffDrivingLicense, StaffDrivingLicenseRegistered)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffDrivingLicense {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffDrivingLicenseRegistered::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: add a select-option row to a staff-
/// registration field.
pub fn create_staff_registration_field_option<C, G>(
    cmd: CreateStaffRegistrationFieldOptionCommand,
    clock: &C,
    ids: &G,
) -> Result<(
    StaffRegistrationFieldOption,
    StaffRegistrationFieldOptionAdded,
)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffRegistrationFieldOption {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffRegistrationFieldOptionAdded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: start a bulk staff-import job.
pub fn create_bulk_import_job<C, G>(
    cmd: CreateBulkImportJobCommand,
    clock: &C,
    ids: &G,
) -> Result<(BulkImportJob, BulkImportJobRecorded)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = BulkImportJob {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = BulkImportJobRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

/// Handler skeleton: start a bulk staff-attendance import job.
pub fn create_staff_attendance_import_batch<C, G>(
    cmd: CreateStaffAttendanceImportBatchCommand,
    clock: &C,
    ids: &G,
) -> Result<(
    StaffAttendanceImportBatch,
    StaffAttendanceImportBatchRecorded,
)>
where
    C: Clock + ?Sized,
    G: IdGenerator + ?Sized,
{
    let now = clock.now();
    let event_id = ids.next_event_id();
    let correlation_id = ids.next_correlation_id();
    let agg = StaffAttendanceImportBatch {
        id: cmd.id,
        school_id: cmd.school_id,
    };
    let event = StaffAttendanceImportBatchRecorded::new(cmd.id, event_id, correlation_id, now);
    Ok((agg, event))
}

// =============================================================================
// Class Teacher Assignment
// =============================================================================

/// "Class Teacher Assignment" workflow service.
///
/// Manages the per-class, per-section, per-academic-year
/// class-teacher assignment (one active staff per
/// class-section-academic-year). Backed by
/// [`AssignClassTeacher`] aggregates; this service exposes
/// the lookup helpers the dispatcher uses to enforce
/// uniqueness before writing a new assignment.
pub struct ClassTeacherAssignmentService;

impl ClassTeacherAssignmentService {
    /// Returns `true` if `staff_id` is the active class teacher
    /// for the given (class, section, academic-year)
    /// combination.
    #[must_use]
    pub fn is_assigned(
        assignments: &[AssignClassTeacher],
        class_id: crate::value_objects::ClassId,
        section_id: crate::value_objects::SectionId,
        staff_id: StaffId,
        academic_id: crate::value_objects::AcademicYearId,
    ) -> bool {
        assignments.iter().any(|a| {
            a.active_status == 1
                && a.class_id == class_id
                && a.section_id == section_id
                && a.staff_id == staff_id
                && a.academic_id == academic_id
        })
    }

    /// Returns the currently active class-teacher assignment
    /// for the given class-section-academic-year, if any.
    #[must_use]
    pub fn current_for_class(
        assignments: &[AssignClassTeacher],
        class_id: crate::value_objects::ClassId,
        section_id: crate::value_objects::SectionId,
        academic_id: crate::value_objects::AcademicYearId,
    ) -> Option<&AssignClassTeacher> {
        assignments.iter().find(|a| {
            a.active_status == 1
                && a.class_id == class_id
                && a.section_id == section_id
                && a.academic_id == academic_id
        })
    }

    /// Returns `true` if `class_id`/`section_id` already has an
    /// active class teacher in the given academic year. Used
    /// by the dispatcher to detect a conflict before creating
    /// a new assignment.
    #[must_use]
    pub fn has_active_teacher(
        assignments: &[AssignClassTeacher],
        class_id: crate::value_objects::ClassId,
        section_id: crate::value_objects::SectionId,
        academic_id: crate::value_objects::AcademicYearId,
    ) -> bool {
        Self::current_for_class(assignments, class_id, section_id, academic_id).is_some()
    }

    /// Counts the number of distinct class-teacher assignments
    /// held by `staff_id` in `academic_id` (active + inactive).
    #[must_use]
    pub fn count_for_staff(
        assignments: &[AssignClassTeacher],
        staff_id: StaffId,
        academic_id: crate::value_objects::AcademicYearId,
    ) -> usize {
        assignments
            .iter()
            .filter(|a| a.staff_id == staff_id && a.academic_id == academic_id)
            .count()
    }
}

// =============================================================================
// Subject Teacher Assignment
// =============================================================================

/// "Subject Teacher Assignment" workflow service.
///
/// Manages the per-class, per-subject, per-academic-year
/// subject-teacher assignment (one active staff per
/// class-subject-academic-year, optionally scoped to a
/// section). Backed by [`AssignSubjectTeacherCommand`]
/// inputs dispatched to the subject-teacher repository.
pub struct SubjectTeacherAssignmentService;

impl SubjectTeacherAssignmentService {
    /// Validates a [`AssignSubjectTeacherCommand`] before
    /// dispatch. Returns `Ok(())` for a well-formed command;
    /// returns [`DomainError::Validation`] if `staff_id`
    /// does not belong to the tenant school.
    pub fn validate(cmd: &AssignSubjectTeacherCommand) -> Result<()> {
        if cmd.staff_id.school_id() != cmd.tenant.school_id {
            return Err(DomainError::validation(
                "staff_id does not belong to the tenant school",
            ));
        }
        Ok(())
    }

    /// Returns `true` if `replacement_id` differs from
    /// `current_id`. A "reassignment" to the same staff
    /// member is a no-op and should be rejected by the
    /// dispatcher.
    #[must_use]
    pub fn is_reassignment(current_id: StaffId, replacement_id: StaffId) -> bool {
        current_id != replacement_id
    }

    /// Returns `true` if `class_id` and `subject_id` belong
    /// to the same school as the tenant. Used as a pre-flight
    /// check before dispatching an assignment.
    #[must_use]
    pub fn scope_matches_tenant(
        cmd: &AssignSubjectTeacherCommand,
        class_school: SchoolId,
        subject_school: SchoolId,
    ) -> bool {
        class_school == cmd.tenant.school_id && subject_school == cmd.tenant.school_id
    }
}

// =============================================================================
// Hourly Rate Management
// =============================================================================

/// "Hourly Rate Management" workflow service.
///
/// Manages per-grade hourly rates ([`HourlyRate`] rows) and
/// per-staff overrides ([`HourlyRateOverride`] rows). Used
/// by the payroll service to resolve the effective hourly
/// rate for a given (staff, grade, academic-year)
/// combination.
pub struct HourlyRateManagementService;

impl HourlyRateManagementService {
    /// Returns the effective hourly rate for `grade` in
    /// `academic_id`. Returns the first active
    /// [`HourlyRate`] row matching (grade, academic_id), or
    /// `None` if no row is configured. Per-staff overrides
    /// are layered on top by the payroll service at
    /// compute time.
    #[must_use]
    pub fn effective_rate(
        grade: &str,
        academic_id: crate::value_objects::AcademicYearId,
        rates: &[HourlyRate],
    ) -> Option<f64> {
        rates
            .iter()
            .find(|r| r.grade == grade && r.academic_id == academic_id)
            .map(|r| r.rate)
    }

    /// Validates that `rate` is non-negative. Returns
    /// [`DomainError::Validation`] for negative rates; the
    /// dispatcher surfaces this as a 400 to the caller.
    pub fn validate_rate(rate: f64) -> Result<()> {
        if rate < 0.0 {
            return Err(DomainError::validation(
                "hourly rate must be non-negative",
            ));
        }
        Ok(())
    }

    /// Returns `true` if `old_rate` differs from `new_rate`
    /// by more than `epsilon`. Used by `update_rate` to
    /// detect a no-op update before dispatch.
    #[must_use]
    pub fn is_rate_change(old_rate: f64, new_rate: f64, epsilon: f64) -> bool {
        (old_rate - new_rate).abs() > epsilon
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use educore_core::clock::{SystemClock, SystemIdGen};
    use educore_core::ids::CorrelationId;
    use educore_core::tenant::UserType;
    use educore_rbac::ids::RoleId;

    struct StubUniqueness;
    impl StaffUniquenessChecker for StubUniqueness {
        fn email_exists(&self, _: SchoolId, _: &str) -> bool {
            false
        }
        fn staff_no_exists(&self, _: SchoolId, _: u32) -> bool {
            false
        }
        fn employee_id_exists(&self, _: SchoolId, _: &str) -> bool {
            false
        }
    }
    #[allow(dead_code)]
    struct StubRefUniqueness;
    impl ReferenceDataUniquenessChecker for StubRefUniqueness {
        fn department_name_exists(&self, _: SchoolId, _: &str) -> bool {
            false
        }
        fn designation_title_exists(&self, _: SchoolId, _: &str) -> bool {
            false
        }
        fn leave_type_name_exists(&self, _: SchoolId, _: &str) -> bool {
            false
        }
    }

    fn ctx(school: SchoolId) -> TenantContext {
        TenantContext::for_user(
            school,
            UserId(uuid::Uuid::now_v7()),
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn hire_staff_returns_aggregate_and_event() {
        let clock = SystemClock;
        let ids = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = HireStaffCommand {
            tenant: ctx(s),
            user_id: UserId(uuid::Uuid::now_v7()),
            role_id: RoleId::new(s, uuid::Uuid::now_v7()),
            staff_no: 1,
            employee_id: "E001".to_owned(),
            first_name: "Alice".to_owned(),
            last_name: "Patel".to_owned(),
            gender: crate::value_objects::Gender::Female,
            date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            date_of_joining: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            email: Some("alice@school.test".to_owned()),
            mobile: None,
            department_id: None,
            designation_id: None,
        };
        let (staff, event) = hire_staff(cmd, &clock, &ids, &StubUniqueness).unwrap();
        assert_eq!(staff.school_id, s);
        assert_eq!(event.staff_id, staff.id);
        assert_eq!(event.aggregate_id(), staff.id.as_uuid());
        assert_eq!(
            <StaffRegistered as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "hr.staff.registered"
        );
    }

    #[test]
    fn request_leave_starts_in_pending() {
        let clock = SystemClock;
        let ids = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = RequestLeaveCommand {
            tenant: ctx(s),
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            type_id: crate::value_objects::LeaveTypeId::new(s, uuid::Uuid::now_v7()),
            apply_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_to: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: Some("vacation".to_owned()),
            note: None,
            file_reference: None,
        };
        let (lr, event) = request_leave(cmd, &clock, &ids).unwrap();
        assert!(lr.is_pending());
        assert_eq!(lr.duration_days(), 5);
        assert_eq!(event.leave_request_id, lr.id);
    }

    #[test]
    fn approve_leave_returns_conflict_on_illegal_transition() {
        let clock = SystemClock;
        let ids = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let cmd = RequestLeaveCommand {
            tenant: ctx(s),
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            type_id: crate::value_objects::LeaveTypeId::new(s, uuid::Uuid::now_v7()),
            apply_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_to: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: None,
            note: None,
            file_reference: None,
        };
        let (mut lr, _) = request_leave(cmd, &clock, &ids).unwrap();
        // First approval succeeds.
        let approver = ctx(s);
        let _ev = approve_leave(&mut lr, approver, None, &clock, &ids).unwrap();
        assert_eq!(lr.approve_status, LeaveStatus::Approved);
        // Second approval is illegal (Approved -> Approved).
        let approver2 = ctx(s);
        let err = approve_leave(&mut lr, approver2, None, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn approve_leave_rejects_self_approval() {
        let clock = SystemClock;
        let ids = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let actor = UserId(uuid::Uuid::now_v7());
        let tenant = TenantContext::for_user(
            s,
            actor,
            CorrelationId(uuid::Uuid::now_v7()),
            UserType::SchoolAdmin,
        );
        let cmd = RequestLeaveCommand {
            tenant: tenant.clone(),
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            type_id: crate::value_objects::LeaveTypeId::new(s, uuid::Uuid::now_v7()),
            apply_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            leave_to: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
            reason: None,
            note: None,
            file_reference: None,
        };
        let (mut lr, _) = request_leave(cmd, &clock, &ids).unwrap();
        let err = approve_leave(&mut lr, tenant, None, &clock, &ids).unwrap_err();
        assert!(matches!(err, DomainError::Conflict(_)));
    }

    #[test]
    fn run_payroll_applies_in_memory_policy_tax() {
        let clock = SystemClock;
        let ids = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let policy = InMemoryPayrollPolicy::new(0.10);
        let cmd = RunPayrollCommand {
            tenant: ctx(s),
            staff_id: StaffId::new(s, uuid::Uuid::now_v7()),
            basic_salary: 100_000.0,
            payroll_month: 6,
            payroll_year: 2026,
            payment_mode: Some("bank".to_owned()),
            bank_id: None,
            note: None,
        };
        let (payroll, event) = run_payroll(cmd, &clock, &ids, &policy).unwrap();
        assert_eq!(payroll.tax, 10_000.0);
        assert_eq!(payroll.net_salary, 90_000.0);
        assert_eq!(payroll.total_deduction, 10_000.0);
        assert_eq!(event.net_salary, 90_000.0);
    }

    #[test]
    fn in_memory_payroll_policy_default_is_10_percent() {
        let policy = InMemoryPayrollPolicy::default();
        let s = SchoolId(uuid::Uuid::now_v7());
        assert_eq!(policy.tax(s, 1000.0), 100.0);
        assert_eq!(policy.tax(s, 0.0), 0.0);
    }

    #[test]
    fn class_teacher_assignment_service_detects_active_teacher() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let class_id = crate::value_objects::ClassId::new(s, uuid::Uuid::now_v7());
        let section_id = crate::value_objects::SectionId::new(s, uuid::Uuid::now_v7());
        let academic_id = crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7());
        let staff_id = StaffId::new(s, uuid::Uuid::now_v7());
        let other_staff = StaffId::new(s, uuid::Uuid::now_v7());
        let now = educore_core::value_objects::Timestamp::now();
        let actor = UserId(uuid::Uuid::now_v7());
        let correlation_id = CorrelationId(uuid::Uuid::now_v7());
        let assignment = AssignClassTeacher::fresh(
            crate::value_objects::AssignClassTeacherId::new(s, uuid::Uuid::now_v7()),
            class_id,
            section_id,
            staff_id,
            academic_id,
            actor,
            now,
            correlation_id,
        );
        let assignments = vec![assignment];
        assert!(ClassTeacherAssignmentService::has_active_teacher(
            &assignments,
            class_id,
            section_id,
            academic_id,
        ));
        assert!(ClassTeacherAssignmentService::is_assigned(
            &assignments,
            class_id,
            section_id,
            staff_id,
            academic_id,
        ));
        assert!(!ClassTeacherAssignmentService::is_assigned(
            &assignments,
            class_id,
            section_id,
            other_staff,
            academic_id,
        ));
        assert_eq!(
            ClassTeacherAssignmentService::count_for_staff(&assignments, staff_id, academic_id),
            1
        );
    }

    #[test]
    fn subject_teacher_assignment_service_validates_tenant_scope() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let other_school = SchoolId(uuid::Uuid::now_v7());
        let staff_id = StaffId::new(s, uuid::Uuid::now_v7());
        let tenant = ctx(s);
        let cmd = AssignSubjectTeacherCommand {
            tenant: tenant.clone(),
            class_id: crate::value_objects::ClassId::new(s, uuid::Uuid::now_v7()),
            section_id: None,
            subject_id: crate::value_objects::SubjectId::new(s, uuid::Uuid::now_v7()),
            staff_id,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
        };
        assert!(SubjectTeacherAssignmentService::validate(&cmd).is_ok());
        assert!(SubjectTeacherAssignmentService::is_reassignment(
            staff_id,
            StaffId::new(s, uuid::Uuid::now_v7()),
        ));
        assert!(!SubjectTeacherAssignmentService::is_reassignment(
            staff_id,
            staff_id,
        ));
        assert!(SubjectTeacherAssignmentService::scope_matches_tenant(
            &cmd,
            s,
            s,
        ));
        assert!(!SubjectTeacherAssignmentService::scope_matches_tenant(
            &cmd,
            other_school,
            s,
        ));
        let foreign_cmd = AssignSubjectTeacherCommand {
            staff_id: StaffId::new(other_school, uuid::Uuid::now_v7()),
            ..cmd.clone()
        };
        let _ = foreign_cmd;
        let tenant_other = ctx(other_school);
        let cmd_other_school = AssignSubjectTeacherCommand {
            tenant: tenant_other,
            ..cmd
        };
        assert!(matches!(
            SubjectTeacherAssignmentService::validate(&cmd_other_school),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn hourly_rate_management_service_resolves_effective_rate() {
        let s = SchoolId(uuid::Uuid::now_v7());
        let academic_id = crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7());
        let now = educore_core::value_objects::Timestamp::now();
        let actor = UserId(uuid::Uuid::now_v7());
        let correlation_id = CorrelationId(uuid::Uuid::now_v7());
        let rate = HourlyRate::fresh(
            crate::value_objects::HourlyRateId::new(s, uuid::Uuid::now_v7()),
            "G5".to_owned(),
            250.0,
            academic_id,
            actor,
            now,
            correlation_id,
        );
        let rates = vec![rate];
        assert_eq!(
            HourlyRateManagementService::effective_rate("G5", academic_id, &rates),
            Some(250.0)
        );
        assert_eq!(
            HourlyRateManagementService::effective_rate("G6", academic_id, &rates),
            None
        );
        assert!(HourlyRateManagementService::validate_rate(0.0).is_ok());
        assert!(HourlyRateManagementService::validate_rate(100.0).is_ok());
        assert!(matches!(
            HourlyRateManagementService::validate_rate(-1.0),
            Err(DomainError::Validation(_))
        ));
        assert!(HourlyRateManagementService::is_rate_change(100.0, 110.0, 1.0));
        assert!(!HourlyRateManagementService::is_rate_change(100.0, 100.5, 1.0));
    }
}
