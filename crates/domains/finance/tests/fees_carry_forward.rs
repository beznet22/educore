//! Integration tests for the **FeesCarryForward aggregate** vertical slice.
//!
//! Pins the FCF I-3 invariant end-to-end: a FeesCarryForward is
//! uniquely scoped to a (school_id, student_id, academic_year_id)
//! tuple. Uniqueness is dispatcher-enforced via the scope-key
//! tuple the aggregate carries as required fields. Companion
//! invariant (FCF I-1): `balance_minor >= 0`.
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
    create_fees_carry_forward, retire_fees_carry_forward, BalanceType, Currency,
    FeesCarryForwardCreated, FeesCarryForwardId, FeesCarryForwardRetired, RealFeesCarryForward,
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

fn fcf_id(g: &SystemIdGen, school: SchoolId) -> FeesCarryForwardId {
    FeesCarryForwardId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =========================================================================
// FCF I-3 typed-id smoke
// =========================================================================

#[test]
fn fees_carry_forward_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_carry_forward_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fcf_id(&g, school);
    let id_b = fcf_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// FCF I-3 construction tests
// =========================================================================

#[test]
fn fresh_full_payload_carries_scope_key_tuple_fcf_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        5_000,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: scope-key tuple + valid balance must construct");
    assert!(agg.is_active());
    assert_eq!(agg.student_id, s_id);
    assert_eq!(agg.academic_year_id, ay_id);
    assert_eq!(agg.balance_minor, 5_000);
    assert_eq!(agg.balance_type, BalanceType::Debit);
    assert_eq!(agg.currency, Currency::INR);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_balance_boundary_valid_fcf_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        0,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-1 companion: zero balance is a valid boundary");
    assert_eq!(agg.balance_minor, 0);
}

#[test]
fn fresh_negative_balance_validation_error_fcf_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        -1,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FCF I-1 companion: negative balance must be rejected");
    assert!(
        format!("{err}").contains("balance_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_distinct_scope_key_tuples_within_same_school_fcf_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesCarryForward::fresh(
        fcf_id(&g, school),
        student_id(&g, school),
        academic_year_id(&g, school),
        5_000,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: distinct scope-key tuple must construct");
    let agg_b = RealFeesCarryForward::fresh(
        fcf_id(&g, school),
        student_id(&g, school),
        academic_year_id(&g, school),
        5_000,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: distinct scope-key tuple must construct");
    assert_ne!(agg_a.student_id, agg_b.student_id);
    assert_ne!(agg_a.academic_year_id, agg_b.academic_year_id);
    assert_eq!(agg_a.school_id, agg_b.school_id);
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        2_500,
        BalanceType::Credit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: scope-key tuple must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.balance_type, BalanceType::Credit);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        1_000,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: scope-key tuple must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesCarryForward::fresh(
        id,
        s_id,
        ay_id,
        1_000,
        BalanceType::Debit,
        Currency::INR,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FCF I-3: scope-key tuple must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(
        format!("{err}").contains("already retired"),
        "unexpected error: {err}"
    );
}

// =========================================================================
// FCF I-3 service integration tests
// =========================================================================

#[test]
fn create_fees_carry_forward_service_emits_created_event_fcf_i_3() {
    use educore_finance::commands::CreateFeesCarryForwardCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesCarryForwardCommand {
        tenant: tenant.clone(),
        fees_carry_forward_id: id,
        student_id: s_id,
        academic_year_id: ay_id,
        balance_minor: 12_500,
        balance_type: BalanceType::Debit,
        currency: Currency::INR,
    };
    let (agg, event): (RealFeesCarryForward, FeesCarryForwardCreated) =
        create_fees_carry_forward(cmd, &clock, &ids)
            .expect("create_fees_carry_forward must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.balance_minor, 12_500);
    assert_eq!(event.fees_carry_forward_id, agg.id);
    assert_eq!(event.student_id, s_id);
    assert_eq!(event.academic_year_id, ay_id);
    assert_eq!(
        <FeesCarryForwardCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_carry_forward.created"
    );
    assert_eq!(
        <FeesCarryForwardCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_carry_forward"
    );
    assert_eq!(<FeesCarryForwardCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_fees_carry_forward_service_rejects_negative_balance_fcf_i_1() {
    use educore_finance::commands::CreateFeesCarryForwardCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let s_id = student_id(&g, school);
    let ay_id = academic_year_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesCarryForwardCommand {
        tenant: tenant.clone(),
        fees_carry_forward_id: id,
        student_id: s_id,
        academic_year_id: ay_id,
        balance_minor: -100,
        balance_type: BalanceType::Debit,
        currency: Currency::INR,
    };
    let err = create_fees_carry_forward(cmd, &clock, &ids)
        .expect_err("FCF I-1: negative balance must be rejected at service layer");
    assert!(
        format!("{err}").contains("balance_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_fees_carry_forward_service_emits_retired_event_fcf() {
    use educore_finance::commands::RetireFeesCarryForwardCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fcf_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesCarryForwardCommand {
        tenant: tenant.clone(),
        fees_carry_forward_id: id,
    };
    let (agg, event): (RealFeesCarryForward, FeesCarryForwardRetired) =
        retire_fees_carry_forward(cmd, &clock, &ids)
            .expect("retire_fees_carry_forward must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_carry_forward_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesCarryForwardRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_carry_forward.retired"
    );
    assert_eq!(
        <FeesCarryForwardRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_carry_forward"
    );
    assert_eq!(<FeesCarryForwardRetired as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}
