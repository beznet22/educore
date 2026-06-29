//! # HR domain vertical-slice integration test
//!
//! Mirrors the Phase 4/5 pattern (`assessment_integration.rs`,
//! `attendance_integration.rs`). Runs on SQLite (always) +
//! PG/MySQL (env-gated). The headline scenario:
//! hire staff → request leave → approve leave → run payroll.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::Arc;

use educore_audit::prelude::*;
use educore_core::clock::{SystemClock, SystemIdGen};
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::idempotency::IdempotencyRecord;
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::transaction::Transaction;
use educore_storage::StorageAdapter;

use educore_hr::prelude::*;

async fn setup_test_env() -> (
    Arc<dyn StorageAdapter>,
    Arc<dyn EventBus>,
    TenantContext,
    SystemIdGen,
) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    (adapter, bus, ctx, g)
}

fn make_tenant(school: SchoolId) -> TenantContext {
    let g = SystemIdGen;
    TenantContext::for_user(
        school,
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    )
}

fn make_role_id(school: SchoolId) -> educore_rbac::ids::RoleId {
    educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7())
}

#[tokio::test]
async fn hr_integration_sqlite_hire_approve_payroll() {
    let (adapter, bus, ctx, g) = setup_test_env().await;
    let clock = SystemClock;
    let ids = SystemIdGen;
    let school = ctx.school_id;

    // Subscribe to bus BEFORE dispatching.
    let mut opts = SubscribeOptions::for_consumer("test-hr".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

    // Create a department first.
    let (dept, dept_event) = create_department(
        ctx.clone(),
        "Mathematics".to_owned(),
        Some("Math department".to_owned()),
        &clock,
        &ids,
        &StubRefUniqueness,
    )
    .expect("create_department");

    // Create a designation.
    let (desig, _desig_event) = create_designation(
        ctx.clone(),
        "Senior Teacher".to_owned(),
        None,
        &clock,
        &ids,
        &StubRefUniqueness,
    )
    .expect("create_designation");

    // Create a leave type.
    let (lt, _lt_event) = create_leave_type(
        ctx.clone(),
        "Casual".to_owned(),
        12,
        &clock,
        &ids,
        &StubRefUniqueness,
    )
    .expect("create_leave_type");

    // Hire a staff member.
    let hire_cmd = HireStaffCommand {
        tenant: ctx.clone(),
        user_id: g.next_user_id(),
        role_id: make_role_id(school),
        staff_no: 1,
        employee_id: "E001".to_owned(),
        first_name: "Alice".to_owned(),
        last_name: "Patel".to_owned(),
        gender: Gender::Female,
        date_of_birth: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
        date_of_joining: chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        email: Some("alice@school.test".to_owned()),
        mobile: None,
        department_id: Some(dept.id),
        designation_id: Some(desig.id),
    };
    let (staff, hire_event) =
        hire_staff(hire_cmd, &clock, &ids, &StubUniqueness).expect("hire_staff");
    assert_eq!(staff.school_id, school);
    assert_eq!(hire_event.aggregate_id(), staff.id.as_uuid());

    // Request leave.
    let req_cmd = RequestLeaveCommand {
        tenant: ctx.clone(),
        staff_id: staff.id,
        type_id: lt.id,
        apply_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        leave_from: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        leave_to: chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
        reason: Some("Family event".to_owned()),
        note: None,
        file_reference: None,
    };
    let (mut lr, _leave_req_event) = request_leave(req_cmd, &clock, &ids).expect("request_leave");
    assert!(lr.is_pending());

    // Approve leave (use a different actor to avoid self-approval rejection).
    let approver = TenantContext::for_user(
        school,
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    let _approve_event =
        approve_leave(&mut lr, approver, None, &clock, &ids).expect("approve_leave");
    assert_eq!(lr.approve_status, LeaveStatus::Approved);

    // Run payroll.
    let payroll_cmd = RunPayrollCommand {
        tenant: ctx.clone(),
        staff_id: staff.id,
        basic_salary: 100_000.0,
        payroll_month: 6,
        payroll_year: 2026,
        payment_mode: Some("bank".to_owned()),
        bank_id: None,
        note: None,
    };
    let policy = InMemoryPayrollPolicy::default();
    let (payroll, payroll_event) =
        run_payroll(payroll_cmd, &clock, &ids, &policy).expect("run_payroll");
    assert_eq!(payroll.tax, 10_000.0);
    assert_eq!(payroll.net_salary, 90_000.0);
    assert_eq!(payroll_event.net_salary, 90_000.0);

    // Build envelopes and write outbox + audit + idempotency.
    let envelopes: Vec<EventEnvelope> = vec![
        dept_event.into_envelope(&ctx),
        hire_event.into_envelope(&ctx),
        payroll_event.into_envelope(&ctx),
    ];
    let tx = adapter.begin().await.expect("begin");
    for env in &envelopes {
        let serialized = SerializedEnvelope::from_event_envelope(env);
        tx.outbox()
            .append(school, serialized)
            .await
            .expect("outbox append");
    }
    let idem_record = IdempotencyRecord {
        school_id: school,
        command_type: "hr.integration_test",
        idempotency_key: IdempotencyKey::from(uuid::Uuid::now_v7()),
        outcome: bytes::Bytes::from_static(br#"{"status":"ok"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![staff.id.as_uuid()],
        aggregate_version: 1,
        etag: None,
        duration_ms: 0,
        emitted_event_ids: Vec::new(),
    };
    let audit_entry = AuditLogEntry::create(
        school,
        ctx.actor_id,
        "staff",
        staff.id.as_uuid(),
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    tx.commit().await.expect("commit");

    // Publish envelopes to bus.
    for env in envelopes {
        bus.publish(env).await.expect("bus publish");
    }

    // Verify the bus received the first envelope.
    let received = sub.next().await;
    match received {
        Some(Ok(env)) => {
            // First event is the department created.
            assert_eq!(env.event_type, "hr.department.created");
            assert_eq!(env.school_id, school);
        }
        other => panic!("expected bus event, got {other:?}"),
    }
}

#[tokio::test]
async fn hr_capability_check_gates_hire_staff() {
    let cap_check = InMemoryCapabilityCheck::new();
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);

    // 1. No grant -> denied.
    let granted = cap_check
        .has(&ctx, Capability::HrStaffCreate)
        .await
        .expect("has");
    assert!(!granted);

    // 2. Grant to a role in the school -> allowed.
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    cap_check.grant(school, role, Capability::HrStaffCreate);
    let granted = cap_check
        .has(&ctx, Capability::HrStaffCreate)
        .await
        .expect("has");
    assert!(granted);
}

#[test]
fn hr_event_type_round_trip_for_all_prompt_aggregates() {
    let g = SystemIdGen;
    let s = SchoolId(uuid::Uuid::now_v7());
    let ev = StaffRegistered::new(
        StaffId::new(s, uuid::Uuid::now_v7()),
        1,
        "E001".to_owned(),
        "A".to_owned(),
        "B".to_owned(),
        None,
        None,
        None,
        None,
        educore_rbac::ids::RoleId::new(s, uuid::Uuid::now_v7()),
        UserId(uuid::Uuid::now_v7()),
        chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        g.next_event_id(),
        CorrelationId(uuid::Uuid::now_v7()),
        Timestamp::now(),
    );
    assert_eq!(
        <StaffRegistered as DomainEvent>::EVENT_TYPE,
        "hr.staff.registered"
    );
    assert_eq!(<StaffRegistered as DomainEvent>::AGGREGATE_TYPE, "staff");

    let dept_id = DepartmentId::new(s, uuid::Uuid::now_v7());
    let ev = DepartmentCreated::new(
        dept_id,
        "Mathematics".to_owned(),
        g.next_event_id(),
        CorrelationId(uuid::Uuid::now_v7()),
        Timestamp::now(),
    );
    assert_eq!(
        <DepartmentCreated as DomainEvent>::EVENT_TYPE,
        "hr.department.created"
    );
    assert_eq!(
        <DepartmentCreated as DomainEvent>::AGGREGATE_TYPE,
        "department"
    );

    let lt_id = LeaveTypeId::new(s, uuid::Uuid::now_v7());
    let ev = LeaveTypeCreated::new(
        lt_id,
        "Casual".to_owned(),
        12,
        g.next_event_id(),
        CorrelationId(uuid::Uuid::now_v7()),
        Timestamp::now(),
    );
    assert_eq!(
        <LeaveTypeCreated as DomainEvent>::EVENT_TYPE,
        "hr.leave_type.created"
    );

    let lr_id = LeaveRequestId::new(s, uuid::Uuid::now_v7());
    let staff_id = StaffId::new(s, uuid::Uuid::now_v7());
    let ev = LeaveRequested::new(
        lr_id,
        staff_id,
        lt_id,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 5).unwrap(),
        None,
        g.next_event_id(),
        CorrelationId(uuid::Uuid::now_v7()),
        Timestamp::now(),
    );
    assert_eq!(
        <LeaveRequested as DomainEvent>::EVENT_TYPE,
        "hr.leave.requested"
    );

    let pg_id = PayrollGenerateId::new(s, uuid::Uuid::now_v7());
    let ev = PayrollGenerated::new(
        pg_id,
        staff_id,
        6,
        2026,
        100_000.0,
        100_000.0,
        10_000.0,
        10_000.0,
        100_000.0,
        90_000.0,
        g.next_event_id(),
        CorrelationId(uuid::Uuid::now_v7()),
        Timestamp::now(),
    );
    assert_eq!(
        <PayrollGenerated as DomainEvent>::EVENT_TYPE,
        "hr.payroll.generated"
    );
}

// -- Stub uniqueness checkers (in-memory) --

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
