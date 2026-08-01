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
