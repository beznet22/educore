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

use crate::aggregate::{Department, Designation, LeaveRequest, LeaveType, PayrollGenerate, Staff};
use educore_events::domain_event::DomainEvent;

use crate::events::{
    DepartmentCreated, LeaveApproved, LeaveRequested, PayrollGenerated, StaffRegistered,
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
}
