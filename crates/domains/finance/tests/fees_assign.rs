//! Integration tests for the **FeesAssign aggregate** vertical slice.
//!
//! Pins FA I-5 end-to-end: a FeesAssign is uniquely scoped to a
//! (student_id, fees_master_id, academic_year_id) tuple. Uniqueness
//! is dispatcher-enforced via the scope-key tuple the aggregate
//! carries as required fields. Companion invariant (FA I-1):
//! `amount_minor >= 0`.
//!
//! Replaces the prior 2 typed-id-only tests with a 12-test
//! behavioral suite.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::{AcademicYearId, StudentId};
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_fees_assign, retire_fees_assign, Currency, FeesAssignCreated, FeesAssignId,
    FeesAssignRetired, FeesMasterId, RealFeesAssign,
};

fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn fa_id(g: &SystemIdGen, school: SchoolId) -> FeesAssignId {
    FeesAssignId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn fm_id(g: &SystemIdGen, school: SchoolId) -> FeesMasterId {
    FeesMasterId::new(school, g.next_uuid())
}

fn ay_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

#[test]
fn fees_assign_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let id = fa_id(&g, tenant.school_id);
    assert_eq!(id.school_id(), tenant.school_id);
}

#[test]
fn fees_assign_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let id_a = fa_id(&g, tenant.school_id);
    let id_b = fa_id(&g, tenant.school_id);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), tenant.school_id);
}

#[test]
fn fresh_full_payload_carries_scope_key_tuple_fa_i_5() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fa_id(&g, school);
    let s_id = student_id(&g, school);
    let m_id = fm_id(&g, school);
    let y_id = ay_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesAssign::fresh(
        id,
        s_id,
        m_id,
        y_id,
        25_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: scope-key tuple + valid amount must construct");
    assert!(agg.is_active());
    assert_eq!(agg.student_id, s_id);
    assert_eq!(agg.fees_master_id, m_id);
    assert_eq!(agg.academic_year_id, y_id);
    assert_eq!(agg.amount_minor, 25_000);
    assert_eq!(agg.currency, Currency::INR);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_amount_boundary_valid_fa_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        0,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-1: zero is a valid boundary");
    assert_eq!(agg.amount_minor, 0);
}

#[test]
fn fresh_negative_amount_validation_error_fa_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        -1,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FA I-1: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_distinct_scope_key_tuples_within_same_school_fa_i_5() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        10_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: distinct scope-key tuple must construct");
    let agg_b = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        10_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: distinct scope-key tuple must construct");
    assert_ne!(agg_a.student_id, agg_b.student_id);
    assert_ne!(agg_a.fees_master_id, agg_b.fees_master_id);
    assert_ne!(agg_a.academic_year_id, agg_b.academic_year_id);
    assert_eq!(agg_a.school_id, agg_b.school_id);
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: scope-key tuple must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: scope-key tuple must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesAssign::fresh(
        fa_id(&g, school),
        student_id(&g, school),
        fm_id(&g, school),
        ay_id(&g, school),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FA I-5: scope-key tuple must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_fees_assign_service_emits_created_event_fa_i_5() {
    use educore_finance::commands::CreateFeesAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fa_id(&g, school);
    let s_id = student_id(&g, school);
    let m_id = fm_id(&g, school);
    let y_id = ay_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesAssignCommand {
        tenant: tenant.clone(),
        fees_assign_id: id,
        student_id: s_id,
        fees_master_id: m_id,
        academic_year_id: y_id,
        amount_minor: 15_000,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let (agg, event): (RealFeesAssign, FeesAssignCreated) =
        create_fees_assign(cmd, &clock, &ids).expect("create_fees_assign must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.amount_minor, 15_000);
    assert_eq!(event.fees_assign_id, agg.id);
    assert_eq!(event.student_id, s_id);
    assert_eq!(event.fees_master_id, m_id);
    assert_eq!(event.academic_year_id, y_id);
    assert_eq!(
        <FeesAssignCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign.created"
    );
    assert_eq!(
        <FeesAssignCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_assign"
    );
    assert_eq!(
        <FeesAssignCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_fees_assign_service_rejects_negative_amount_fa_i_1() {
    use educore_finance::commands::CreateFeesAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesAssignCommand {
        tenant: tenant.clone(),
        fees_assign_id: fa_id(&g, school),
        student_id: student_id(&g, school),
        fees_master_id: fm_id(&g, school),
        academic_year_id: ay_id(&g, school),
        amount_minor: -100,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let err = create_fees_assign(cmd, &clock, &ids)
        .expect_err("FA I-1: negative amount_minor must be rejected at service layer");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_fees_assign_service_emits_retired_event_fa() {
    use educore_finance::commands::RetireFeesAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesAssignCommand {
        tenant: tenant.clone(),
        fees_assign_id: fa_id(&g, school),
    };
    let (agg, event): (RealFeesAssign, FeesAssignRetired) =
        retire_fees_assign(cmd, &clock, &ids).expect("retire_fees_assign must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_assign_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesAssignRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign.retired"
    );
    assert_eq!(
        <FeesAssignRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_assign"
    );
    assert_eq!(
        <FeesAssignRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

// ====================================================================
// -- Wave 131 -- FA I-3 + FA I-4 state machine extension tests --
// ====================================================================

use educore_core::error::DomainError;
use educore_core::ids::UserId;
use educore_core::value_objects::Timestamp;
use educore_finance::commands::{CancelFeesAssignCommand, RecordFeesAssignPaymentCommand};
use educore_finance::events::{FeesAssignCancelled, FeesAssignPaymentRecorded};
use educore_finance::services::{cancel_fees_assign, record_fees_assign_payment};
use educore_finance::value_objects::LifecycleStatus;

fn build_assign(actor: UserId, amount_minor: i64) -> RealFeesAssign {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    RealFeesAssign::fresh(
        FeesAssignId::new(school, g.next_uuid()),
        StudentId::new(school, g.next_uuid()),
        FeesMasterId::new(school, g.next_uuid()),
        AcademicYearId::new(school, g.next_uuid()),
        amount_minor,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        actor,
        Timestamp::now(),
        _tenant.correlation_id,
    )
    .expect("fresh should succeed")
}

// ---- LifecycleStatus enum round-trips ----

#[test]
fn lifecycle_status_as_str_round_trip() {
    assert_eq!(LifecycleStatus::Open.as_str(), "open");
    assert_eq!(LifecycleStatus::Paid.as_str(), "paid");
    assert_eq!(LifecycleStatus::Cancelled.as_str(), "cancelled");
    assert_eq!(LifecycleStatus::parse("open"), Some(LifecycleStatus::Open));
    assert_eq!(LifecycleStatus::parse("paid"), Some(LifecycleStatus::Paid));
    assert_eq!(LifecycleStatus::parse("cancelled"), Some(LifecycleStatus::Cancelled));
    assert_eq!(LifecycleStatus::parse("unknown"), None);
}

#[test]
fn lifecycle_status_can_transition_only_from_open() {
    assert!(LifecycleStatus::Open.can_transition_to(LifecycleStatus::Paid));
    assert!(LifecycleStatus::Open.can_transition_to(LifecycleStatus::Cancelled));
    assert!(!LifecycleStatus::Paid.can_transition_to(LifecycleStatus::Open));
    assert!(!LifecycleStatus::Paid.can_transition_to(LifecycleStatus::Cancelled));
    assert!(!LifecycleStatus::Cancelled.can_transition_to(LifecycleStatus::Open));
    assert!(!LifecycleStatus::Cancelled.can_transition_to(LifecycleStatus::Paid));
}

// ---- fresh initializes lifecycle + paid_amount_minor ----

#[test]
fn fresh_initializes_paid_amount_zero_lifecycle_open_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let agg = build_assign(actor, 10_000);
    assert_eq!(agg.paid_amount_minor, 0);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Open);
    assert_eq!(agg.balance_minor(), 10_000);
}

// ---- FA I-3: payment progression ----

#[test]
fn partial_payment_bumps_paid_amount_keeps_open_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(3_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("partial payment should succeed");
    assert_eq!(agg.paid_amount_minor, 3_000);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Open);
    assert_eq!(agg.balance_minor(), 7_000);
}

#[test]
fn full_payment_transitions_open_to_paid_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(10_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("full payment should succeed");
    assert_eq!(agg.paid_amount_minor, 10_000);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Paid);
    assert_eq!(agg.balance_minor(), 0);
}

#[test]
fn cumulative_payments_reach_paid_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(4_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("first payment");
    agg.record_payment(4_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("second payment");
    agg.record_payment(2_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("third payment reaches cap");
    assert_eq!(agg.paid_amount_minor, 10_000);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Paid);
}

// ---- FA I-3: payment guards ----

#[test]
fn zero_payment_returns_validation_error_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    let result = agg.record_payment(0, actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn negative_payment_returns_validation_error_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    let result = agg.record_payment(-1, actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn overpayment_returns_conflict_fa_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    let result = agg.record_payment(10_001, actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn payment_after_paid_returns_conflict_fa_i_4() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(10_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("full payment");
    let result = agg.record_payment(1, actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- FA I-4: cancel ----

#[test]
fn cancel_open_assignment_transitions_to_cancelled_fa_i_4() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.cancel(actor, Timestamp::now(), _g.next_event_id())
        .expect("cancel should succeed");
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Cancelled);
}

#[test]
fn cancel_after_payment_returns_conflict_fa_i_4() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(3_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("partial payment");
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn cancel_after_cancelled_returns_conflict_fa_i_4() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.cancel(actor, Timestamp::now(), _g.next_event_id())
        .expect("first cancel");
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn cancel_after_paid_returns_conflict_fa_i_4() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_assign(actor, 10_000);
    agg.record_payment(10_000, actor, Timestamp::now(), _g.next_event_id())
        .expect("full payment");
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- service integration ----

#[test]
fn record_payment_service_emits_event_fa_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealFeesAssign::fresh(
        FeesAssignId::new(school, g.next_uuid()),
        StudentId::new(school, g.next_uuid()),
        FeesMasterId::new(school, g.next_uuid()),
        AcademicYearId::new(school, g.next_uuid()),
        10_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = RecordFeesAssignPaymentCommand {
        tenant,
        fees_assign_id: id,
        amount_minor: 10_000,
    };
    let (updated, evt): (RealFeesAssign, FeesAssignPaymentRecorded) =
        record_fees_assign_payment(agg, cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(updated.lifecycle_status, LifecycleStatus::Paid);
    assert_eq!(evt.amount_minor, 10_000);
    assert_eq!(evt.paid_amount_minor, 10_000);
    assert_eq!(evt.lifecycle_status, LifecycleStatus::Paid);
    assert_eq!(
        <FeesAssignPaymentRecorded as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign.payment_recorded"
    );
    assert_eq!(
        <FeesAssignPaymentRecorded as DomainEvent>::AGGREGATE_TYPE,
        "fees_assign"
    );
}

#[test]
fn cancel_service_emits_event_fa_i_4() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealFeesAssign::fresh(
        FeesAssignId::new(school, g.next_uuid()),
        StudentId::new(school, g.next_uuid()),
        FeesMasterId::new(school, g.next_uuid()),
        AcademicYearId::new(school, g.next_uuid()),
        10_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = CancelFeesAssignCommand {
        tenant,
        fees_assign_id: id,
    };
    let (updated, evt): (RealFeesAssign, FeesAssignCancelled) =
        cancel_fees_assign(agg, cmd, &clock, &g).expect("service should succeed");
    assert_eq!(updated.lifecycle_status, LifecycleStatus::Cancelled);
    assert_eq!(evt.cancelled_by, actor);
    assert_eq!(evt.lifecycle_status, LifecycleStatus::Cancelled);
    assert_eq!(
        <FeesAssignCancelled as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign.cancelled"
    );
}
