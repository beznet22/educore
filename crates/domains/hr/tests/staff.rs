//! Integration tests for the **Staff aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Staff`](educore_hr::aggregate::Staff) end-to-end through the
//! service layer:
//!
//! 1. `hire_staff` validates the input (person names, email,
//!    phone, DOB, **joining date not in the future**, and
//!    uniqueness checks via `StaffUniquenessChecker`), constructs
//!    the aggregate with `school_id` derived from the typed id,
//!    and emits a [`StaffRegistered`] event.
//! 2. Each validation failure path returns `DomainError::Validation`
//!    or `DomainError::Conflict` as appropriate.
//!
//! The tenant anchor (Staff I-1 / spec #1) is **structurally
//! enforced** by the `hr_typed_id!` macro: the only way to build
//! a `StaffId` is via `StaffId::new(school, uuid)`, and
//! `Staff::fresh` derives `school_id` from `id.school_id()`. The
//! tests pin that derivation explicitly.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_hr::prelude::*;
use educore_hr::services::{
    delete_staff, hire_staff, DeleteStaffCommand, StaffReferenceChecker, StaffUniquenessChecker,
};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin_context() -> (TenantContext, SchoolId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        school,
    )
}

/// `StaffUniquenessChecker` mock that flags the supplied
/// `email` / `mobile` / `staff_no` / `employee_id` as already
/// present. All other lookups return `false` (no conflict).
struct ConflictingUniqueness {
    email: Option<String>,
    mobile: Option<String>,
    staff_no: Option<u32>,
    employee_id: Option<String>,
}

impl ConflictingUniqueness {
    fn none() -> Self {
        Self {
            email: None,
            mobile: None,
            staff_no: None,
            employee_id: None,
        }
    }

    fn with_email(email: &str) -> Self {
        Self {
            email: Some(email.to_owned()),
            ..Self::none()
        }
    }

    fn with_staff_no(staff_no: u32) -> Self {
        Self {
            staff_no: Some(staff_no),
            ..Self::none()
        }
    }
}

impl StaffUniquenessChecker for ConflictingUniqueness {
    fn email_exists(&self, _school: SchoolId, email: &str) -> bool {
        self.email.as_deref() == Some(email)
    }
    fn mobile_exists(&self, _school: SchoolId, mobile: &str) -> bool {
        self.mobile.as_deref() == Some(mobile)
    }
    fn staff_no_exists(&self, _school: SchoolId, staff_no: u32) -> bool {
        self.staff_no == Some(staff_no)
    }
    fn employee_id_exists(&self, _school: SchoolId, employee_id: &str) -> bool {
        self.employee_id.as_deref() == Some(employee_id)
    }
}

/// Default command builder for happy-path tests. Individual
/// tests override the fields they care about.
fn default_command(tenant: TenantContext) -> HireStaffCommand {
    let school_id = tenant.school_id;
    HireStaffCommand {
        tenant,
        user_id: SystemIdGen.next_user_id(),
        role_id: educore_rbac::ids::RoleId::new(
            school_id,
            SystemIdGen.next_uuid(),
        ),
        staff_no: 1,
        employee_id: "EMP-001".to_owned(),
        first_name: "Alice".to_owned(),
        last_name: "Anderson".to_owned(),
        gender: educore_hr::value_objects::Gender::Female,
        date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 1).expect("valid date"),
        date_of_joining: NaiveDate::from_ymd_opt(2024, 1, 15).expect("valid date"),
        email: Some("alice@example.com".to_owned()),
        mobile: Some("+1234567890".to_owned()),
        department_id: None,
        designation_id: None,
    }
}

// =============================================================================
// I-1: Tenant anchor from SchoolId (structurally enforced)
// =============================================================================

/// Spec invariant #1 / checklist I-1: `school_id` is derived
/// from `id.school_id()` at construction; the caller cannot
/// pick a different school for the aggregate.
#[test]
fn staff_tenant_anchor_matches_typed_id() {
    let (tenant, school) = admin_context();
    let tenant_school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (staff, _event) = hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff");

    // The aggregate's school_id MUST equal both the typed id's
    // school_id AND the tenant's school_id. Any divergence here
    // would indicate a tenant-anchor bypass.
    assert_eq!(staff.school_id, school);
    assert_eq!(staff.school_id, staff.id.school_id());
    assert_eq!(staff.school_id, tenant_school);
}

// =============================================================================
// I-2: Staff ID (staff_no) unique per school
// =============================================================================

/// Spec invariant #3 / checklist I-2: `staff_no` is unique
/// within a school. The `StaffUniquenessChecker` port enforces
/// this; the service layer rejects duplicate `staff_no` values.
#[test]
fn hire_staff_rejects_duplicate_staff_no() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::with_staff_no(1),
    )
    .expect_err("duplicate staff_no must fail");

    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

/// Spec invariant #3 (happy path): when `staff_no` is unique,
/// `hire_staff` succeeds and stores the value on the aggregate.
#[test]
fn hire_staff_accepts_unique_staff_no() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (staff, _event) = hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff");

    assert_eq!(staff.staff_no, 1);
}

// =============================================================================
// I-3: Email unique per school
// =============================================================================

/// Spec invariant #4 / checklist I-3: `email` is unique within
/// a school (when provided). The `StaffUniquenessChecker` port
/// enforces this; the service layer rejects duplicate emails.
#[test]
fn hire_staff_rejects_duplicate_email() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::with_email("alice@example.com"),
    )
    .expect_err("duplicate email must fail");

    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

/// Spec invariant #4 (happy path): when `email` is unique,
/// `hire_staff` succeeds and stores the value on the aggregate.
#[test]
fn hire_staff_accepts_unique_email() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (staff, _event) = hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff");

    assert_eq!(staff.email.as_deref(), Some("alice@example.com"));
}

// =============================================================================
// I-5: Joining date ≤ current date
// =============================================================================

/// Spec invariant #5 / checklist I-5: `date_of_joining` must
/// not be in the future. The `validate_joining_date_not_future`
/// helper enforces this; `hire_staff` rejects future dates.
#[test]
fn hire_staff_rejects_future_joining_date() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // A date 1 year in the future is unambiguously invalid.
    let future = chrono::Utc::now().date_naive() + chrono::Duration::days(365);
    let mut cmd = default_command(tenant);
    cmd.date_of_joining = future;

    let err = hire_staff(
        cmd,
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect_err("future joining date must fail");

    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec invariant #5 (happy path): when `date_of_joining` is
/// today or in the past, `hire_staff` succeeds.
#[test]
fn hire_staff_accepts_today_as_joining_date() {
    let (tenant, _school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = default_command(tenant);
    let today = chrono::Utc::now().date_naive();
    cmd.date_of_joining = today;

    let (staff, _event) = hire_staff(
        cmd,
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff (today)");

    assert_eq!(staff.date_of_joining, today);
}

// =============================================================================
// Smoke: full happy path emits a StaffRegistered event
// =============================================================================

/// End-to-end happy path: when all invariants pass, `hire_staff`
/// returns a `Staff` aggregate + a `StaffRegistered` event with
/// the right `event_type`, `aggregate_type`, `schema_version`,
/// `aggregate_id`, `school_id`, and key fields.
#[test]
fn hire_staff_returns_aggregate_and_event() {
    let (tenant, school) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (staff, event) = hire_staff(
        default_command(tenant.clone()),
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff");

    // Aggregate fields are populated from the command.
    assert_eq!(staff.school_id, school);
    assert_eq!(staff.first_name, "Alice");
    assert_eq!(staff.last_name, "Anderson");
    assert_eq!(staff.employee_id, "EMP-001");
    assert_eq!(staff.created_by, tenant.actor_id);
    assert_eq!(staff.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised at version 1 and active.
    assert_eq!(staff.version.get(), 1);
    assert!(staff.active_status.is_active());

    // Event metadata matches the aggregate's typed id and the
    // DomainEvent trait's contract.
    assert_eq!(
        <StaffRegistered as DomainEvent>::EVENT_TYPE,
        "hr.staff.registered"
    );
    assert_eq!(
        <StaffRegistered as DomainEvent>::AGGREGATE_TYPE,
        "staff"
    );
    assert_eq!(<StaffRegistered as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), staff.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.staff_id, staff.id);
}

// =============================================================================
// I-6: Status state machine (Active -> {Suspended, Resigned,
//       Terminated, Retired}; Suspended -> {Active, Resigned,
//       Terminated, Retired})
// =============================================================================

/// Helper: build a Staff in Active status with a known
/// `date_of_joining`. Reused by the FSM tests below.
fn active_staff(tenant: TenantContext) -> Staff {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    hire_staff(
        default_command(tenant),
        &clock,
        &ids,
        &ConflictingUniqueness::none(),
    )
    .expect("hire staff")
    .0
}

/// Spec invariant #6 (happy path): `Staff::suspend()` from
/// `Active` succeeds, sets `status = Suspended`, records the
/// reason + expected return.
#[test]
fn suspend_transitions_active_to_suspended() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff
        .suspend(
            "Misconduct investigation".to_owned(),
            Some(NaiveDate::from_ymd_opt(2024, 6, 1).expect("date")),
            ts,
        )
        .expect("suspend");

    assert_eq!(staff.status, StaffStatus::Suspended);
    assert_eq!(staff.suspension_reason.as_deref(), Some("Misconduct investigation"));
    assert_eq!(
        staff.expected_return_date,
        Some(NaiveDate::from_ymd_opt(2024, 6, 1).expect("date"))
    );
}

/// Spec invariant #6 (rejection): cannot suspend from a terminal
/// state. Returns `DomainError::Validation`.
#[test]
fn suspend_rejected_from_terminal_state() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    staff.status = StaffStatus::Resigned; // simulate terminal
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff
        .suspend("x".to_owned(), None, ts)
        .expect_err("cannot suspend from terminal");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec invariant #6 (happy path): `Staff::reinstate()` from
/// `Suspended` returns to `Active`, clears suspension reason.
#[test]
fn reinstate_transitions_suspended_to_active() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();
    staff.suspend("temp".to_owned(), None, ts).expect("suspend");

    staff.reinstate(ts).expect("reinstate");

    assert_eq!(staff.status, StaffStatus::Active);
    assert!(staff.suspension_reason.is_none());
    assert!(staff.expected_return_date.is_none());
}

/// Spec invariant #6 (rejection): cannot reinstate from Active.
#[test]
fn reinstate_rejected_from_active() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff.reinstate(ts).expect_err("cannot reinstate from Active");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec invariant #6 (happy path): `Staff::resign()` from Active.
#[test]
fn resign_transitions_active_to_resigned() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff
        .resign(NaiveDate::from_ymd_opt(2024, 5, 15).expect("date"), ts)
        .expect("resign");

    assert_eq!(staff.status, StaffStatus::Resigned);
    assert_eq!(
        staff.resignation_date,
        Some(NaiveDate::from_ymd_opt(2024, 5, 15).expect("date"))
    );
    assert!(staff.is_terminal());
}

/// Spec invariant #6 (rejection): terminal states cannot transition.
#[test]
fn resign_rejected_from_terminal() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    staff.status = StaffStatus::Terminated;
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff
        .resign(NaiveDate::from_ymd_opt(2024, 5, 15).expect("date"), ts)
        .expect_err("cannot resign from terminal");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec invariant #6 (happy path): `Staff::terminate()`.
#[test]
fn terminate_transitions_active_to_terminated() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff
        .terminate(NaiveDate::from_ymd_opt(2024, 4, 1).expect("date"), ts)
        .expect("terminate");

    assert_eq!(staff.status, StaffStatus::Terminated);
    assert!(staff.is_terminal());
}

/// Spec invariant #6 (happy path): `Staff::retire()`.
#[test]
fn retire_transitions_active_to_retired() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff
        .retire(NaiveDate::from_ymd_opt(2030, 1, 1).expect("date"), ts)
        .expect("retire");

    assert_eq!(staff.status, StaffStatus::Retired);
    assert!(staff.is_terminal());
}

/// Spec invariant #6 (rejection): terminal-to-terminal rejected.
#[test]
fn terminate_rejected_from_terminal() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    staff.status = StaffStatus::Retired;
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff
        .terminate(NaiveDate::from_ymd_opt(2024, 4, 1).expect("date"), ts)
        .expect_err("cannot terminate from terminal");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec invariant #6: full happy-path chain
/// Active -> Suspended -> Active -> Resigned.
#[test]
fn full_fsm_chain_active_suspended_active_resigned() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    // Active -> Suspended
    staff.suspend("temp".to_owned(), None, ts).expect("suspend");
    assert_eq!(staff.status, StaffStatus::Suspended);

    // Suspended -> Active
    staff.reinstate(ts).expect("reinstate");
    assert_eq!(staff.status, StaffStatus::Active);
    assert!(!staff.is_terminal());

    // Active -> Resigned (terminal)
    staff
        .resign(NaiveDate::from_ymd_opt(2024, 12, 31).expect("date"), ts)
        .expect("resign");
    assert_eq!(staff.status, StaffStatus::Resigned);
    assert!(staff.is_terminal());
}

// =============================================================================
// I-7: Cannot hard-delete while active references exist
// =============================================================================

/// `StaffReferenceChecker` mock. Each field is `true` to
/// simulate an existing reference.
struct StubReferences {
    has_teacher: bool,
    has_leave: bool,
    has_payroll: bool,
}

impl StubReferences {
    fn none() -> Self {
        Self {
            has_teacher: false,
            has_leave: false,
            has_payroll: false,
        }
    }
}

impl StaffReferenceChecker for StubReferences {
    fn has_active_assign_class_teacher(
        &self,
        _school: educore_core::ids::SchoolId,
        _staff_id: StaffId,
    ) -> bool {
        self.has_teacher
    }
    fn has_active_leave_request(
        &self,
        _school: educore_core::ids::SchoolId,
        _staff_id: StaffId,
    ) -> bool {
        self.has_leave
    }
    fn has_open_payroll(
        &self,
        _school: educore_core::ids::SchoolId,
        _staff_id: StaffId,
    ) -> bool {
        self.has_payroll
    }
}

/// Spec invariant #7 (happy path): when no active references
/// exist, `delete_staff` soft-deletes the staff member
/// (`active_status = Retired`) and returns a `StaffDeleted`
/// event. The FSM `status` is unchanged (soft-delete is
/// distinct from the FSM `retire()` transition).
#[test]
fn delete_staff_soft_deletes_when_no_references() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant.clone());
    let school = staff.school_id;
    let staff_id = staff.id;

    let clock = TestClock::new();
    let ids = SystemIdGen;
    let cmd = DeleteStaffCommand {
        tenant,
        staff_id,
        reason: "End of contract".to_owned(),
    };

    let event = delete_staff(&mut staff, cmd, &clock, &ids, &StubReferences::none())
        .expect("delete staff");

    assert_eq!(staff.active_status, ActiveStatus::Retired);
    assert!(!staff.active_status.is_active());
    // FSM status is untouched.
    assert_eq!(staff.status, StaffStatus::Active);
    assert_eq!(event.staff_id, staff_id);
    assert_eq!(
        <StaffDeleted as DomainEvent>::EVENT_TYPE,
        "hr.staff.deleted"
    );
    assert_eq!(<StaffDeleted as DomainEvent>::AGGREGATE_TYPE, "staff");
    assert_eq!(event.school_id(), school);
    assert_eq!(event.reason, "End of contract");
}

/// Spec invariant #7 (rejection): cannot delete a staff with
/// open payroll references.
#[test]
fn delete_staff_rejected_by_open_payroll() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant.clone());
    let cmd = DeleteStaffCommand {
        tenant,
        staff_id: staff.id,
        reason: "x".to_owned(),
    };

    let refs = StubReferences {
        has_teacher: false,
        has_leave: false,
        has_payroll: true,
    };
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = delete_staff(&mut staff, cmd, &clock, &ids, &refs)
        .expect_err("open payroll must block delete");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    // Staff remains active after rejection.
    assert!(staff.active_status.is_active());
}

/// Spec invariant #7 (rejection): cannot delete a staff with
/// active LeaveRequest references.
#[test]
fn delete_staff_rejected_by_active_leave_request() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant.clone());
    let cmd = DeleteStaffCommand {
        tenant,
        staff_id: staff.id,
        reason: "x".to_owned(),
    };

    let refs = StubReferences {
        has_teacher: false,
        has_leave: true,
        has_payroll: false,
    };
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = delete_staff(&mut staff, cmd, &clock, &ids, &refs)
        .expect_err("active leave must block delete");
    assert!(matches!(err, DomainError::Conflict(_)));
    assert!(staff.active_status.is_active());
}

/// Spec invariant #7 (rejection): cannot delete a staff with
/// active AssignClassTeacher references.
#[test]
fn delete_staff_rejected_by_active_class_teacher() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant.clone());
    let cmd = DeleteStaffCommand {
        tenant,
        staff_id: staff.id,
        reason: "x".to_owned(),
    };

    let refs = StubReferences {
        has_teacher: true,
        has_leave: false,
        has_payroll: false,
    };
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = delete_staff(&mut staff, cmd, &clock, &ids, &refs)
        .expect_err("active class teacher must block delete");
    assert!(matches!(err, DomainError::Conflict(_)));
    assert!(staff.active_status.is_active());
}

// =============================================================================
// I-8: Leave quotas non-negative (spec invariant #8)
// =============================================================================

/// Spec invariant #8 (happy path): `Staff::set_leave_quotas`
/// accepts non-negative values for casual, medical, and
/// maternity.
#[test]
fn set_leave_quotas_accepts_non_negative() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff.set_leave_quotas(12.0, 8.0, 90.0, ts).expect("set quotas");

    assert_eq!(staff.casual_leave_quota, 12.0);
    assert_eq!(staff.medical_leave_quota, 8.0);
    assert_eq!(staff.maternity_leave_quota, 90.0);
}

/// Spec invariant #8 (rejection): a single negative quota
/// rejects the entire `set_leave_quotas` call and the state
/// remains unchanged.
#[test]
fn set_leave_quotas_rejects_negative_casual() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();
    let original_casual = staff.casual_leave_quota;

    let err = staff
        .set_leave_quotas(-1.0, 8.0, 90.0, ts)
        .expect_err("negative casual must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    // State unchanged after rejection.
    assert_eq!(staff.casual_leave_quota, original_casual);
}

/// Spec invariant #8 (rejection): medical quota negative.
#[test]
fn set_leave_quotas_rejects_negative_medical() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff
        .set_leave_quotas(12.0, -0.5, 90.0, ts)
        .expect_err("negative medical must fail");
    assert!(matches!(err, DomainError::Validation(_)));
}

/// Spec invariant #8 (rejection): maternity quota negative.
#[test]
fn set_leave_quotas_rejects_negative_maternity() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    let err = staff
        .set_leave_quotas(12.0, 8.0, -90.0, ts)
        .expect_err("negative maternity must fail");
    assert!(matches!(err, DomainError::Validation(_)));
}

/// Spec invariant #8 (boundary): zero is non-negative, accepted.
#[test]
fn set_leave_quotas_accepts_zero() {
    let (tenant, _school) = admin_context();
    let mut staff = active_staff(tenant);
    let ts = educore_core::value_objects::Timestamp::now();

    staff.set_leave_quotas(0.0, 0.0, 0.0, ts).expect("zero ok");

    assert_eq!(staff.casual_leave_quota, 0.0);
    assert_eq!(staff.medical_leave_quota, 0.0);
    assert_eq!(staff.maternity_leave_quota, 0.0);
}
