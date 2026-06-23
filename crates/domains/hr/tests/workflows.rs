//! Integration tests for the **HR domain workflows**.
//!
//! Implements (lean subset of) `docs/specs/hr/workflows.md`:
//!
//! - Staff Onboarding — `register_staff_happy_path`
//! - Leave Request Lifecycle — `request_leave_happy_path`
//! - Payroll Generation — `generate_payroll_happy_path`
//!
//! Pattern matches `crates/domains/cms/tests/workflows.rs` (lean).
//! The handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **aggregate layer** contract
//! that the service factory fns (`hire_staff`, `request_leave`,
//! `run_payroll`) return.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_hr::prelude::*;
use educore_rbac::ids::RoleId;

/// Stub uniqueness checker (the storage adapter is the
/// canonical implementation; tests use this no-op variant).
struct StubUniqueness;
impl StaffUniquenessChecker for StubUniqueness {
    fn email_exists(&self, _: educore_core::ids::SchoolId, _: &str) -> bool {
        false
    }
    fn staff_no_exists(&self, _: educore_core::ids::SchoolId, _: u32) -> bool {
        false
    }
    fn employee_id_exists(&self, _: educore_core::ids::SchoolId, _: &str) -> bool {
        false
    }
}

fn admin() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let tenant = TenantContext::for_user(
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    (tenant, g)
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Staff Onboarding (workflows.md § "Staff Onboarding") step 2:
/// `RegisterStaff` returns a fresh `Staff` aggregate in the
/// `Active` state and emits `StaffRegistered`.
#[test]
fn register_staff_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let cmd = HireStaffCommand {
        tenant: tenant.clone(),
        user_id: g.next_user_id(),
        role_id: RoleId::new(tenant.school_id, g.next_uuid()),
        staff_no: 1,
        employee_id: "E001".to_owned(),
        first_name: "Alice".to_owned(),
        last_name: "Patel".to_owned(),
        gender: Gender::Female,
        date_of_birth: date(1990, 1, 1),
        date_of_joining: date(2020, 1, 1),
        email: Some("alice@school.test".to_owned()),
        mobile: None,
        department_id: None,
        designation_id: None,
    };
    let (staff, event) = hire_staff(cmd, &clock, &ids, &StubUniqueness).unwrap();
    assert_eq!(staff.school_id, tenant.school_id);
    assert_eq!(staff.first_name, "Alice");
    assert_eq!(staff.status, StaffStatus::Active);
    assert_eq!(event.staff_id, staff.id);
    assert_eq!(event.employee_id, "E001");
    assert_eq!(<StaffRegistered as DomainEvent>::EVENT_TYPE, "hr.staff.registered");
}

/// Leave Request Lifecycle (workflows.md § "Leave Request Lifecycle")
/// step 3: `RequestLeave` returns a `LeaveRequest` in `Pending`
/// state and emits `LeaveRequested`.
#[test]
fn request_leave_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let cmd = RequestLeaveCommand {
        tenant,
        staff_id: StaffId::new(school, g.next_uuid()),
        type_id: LeaveTypeId::new(school, g.next_uuid()),
        apply_date: date(2026, 6, 1),
        leave_from: date(2026, 6, 5),
        leave_to: date(2026, 6, 7),
        reason: Some("Family event".to_owned()),
        note: None,
        file_reference: None,
    };
    let (lr, event) = request_leave(cmd, &clock, &ids).unwrap();
    assert_eq!(lr.school_id, school);
    assert_eq!(lr.approve_status, LeaveStatus::Pending);
    assert_eq!(event.staff_id, lr.staff_id);
    assert_eq!(event.leave_from, date(2026, 6, 5));
    assert_eq!(<LeaveRequested as DomainEvent>::EVENT_TYPE, "hr.leave.requested");
}

/// Payroll Generation (workflows.md § "Payroll Generation")
/// step 2: `RunPayroll` reads the salary template (basic_salary
/// here), applies the `InMemoryPayrollPolicy` (10% flat tax),
/// returns a `PayrollGenerate` in `Generated` state, and emits
/// `PayrollGenerated`.
#[test]
fn generate_payroll_happy_path() {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (tenant, g) = admin();
    let school = tenant.school_id;
    let policy = InMemoryPayrollPolicy::default();
    let cmd = RunPayrollCommand {
        tenant,
        staff_id: StaffId::new(school, g.next_uuid()),
        basic_salary: 50_000.0,
        payroll_month: 6,
        payroll_year: 2026,
        payment_mode: Some("bank_transfer".to_owned()),
        bank_id: Some(g.next_uuid()),
        note: None,
    };
    let (payroll, event) = run_payroll(cmd, &clock, &ids, &policy).unwrap();
    assert_eq!(payroll.school_id, school);
    assert_eq!(payroll.payroll_status, PayrollStatus::Generated);
    assert_eq!(payroll.basic_salary, 50_000.0);
    assert_eq!(payroll.tax, 5_000.0); // 10% default
    assert_eq!(payroll.net_salary, 45_000.0);
    assert_eq!(event.payroll_month, 6);
    assert_eq!(<PayrollGenerated as DomainEvent>::EVENT_TYPE, "hr.payroll.generated");
}
