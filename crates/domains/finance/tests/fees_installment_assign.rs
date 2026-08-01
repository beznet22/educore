//! Integration tests for the **FeesInstallmentAssign aggregate** vertical slice.
//!
//! Pins the FIA I-1 + FIA I-2 invariants end-to-end:
//! - FIA I-1: unique per (assign, installment) scope-key tuple.
//! - FIA I-2: paid_amount <= amount + discount + 3 sub-validations:
//!   amount_minor >= 0, discount_minor >= 0, paid_amount_minor >= 0,
//!   plus companion invariant paid_amount_minor <= amount_minor +
//!   discount_minor.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_fees_installment_assign, retire_fees_installment_assign, FeesAssignId,
    FeesInstallmentAssignCreated, FeesInstallmentAssignId, FeesInstallmentAssignRetired,
    FeesInstallmentId, RealFeesInstallmentAssign,
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

fn fia_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentAssignId {
    FeesInstallmentAssignId::new(school, g.next_uuid())
}

fn assign_id(g: &SystemIdGen, school: SchoolId) -> FeesAssignId {
    FeesAssignId::new(school, g.next_uuid())
}

fn installment_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentId {
    FeesInstallmentId::new(school, g.next_uuid())
}

// =========================================================================
// FIA I-1 tests (carried over from Wave 107)
// =========================================================================

#[test]
fn fees_installment_assign_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let id = fia_id(&g, tenant.school_id);
    assert_eq!(id.school_id(), tenant.school_id);
}

#[test]
fn fees_installment_assign_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let id_a = fia_id(&g, tenant.school_id);
    let id_b = fia_id(&g, tenant.school_id);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), tenant.school_id);
}

#[test]
fn fresh_full_payload_carries_scope_key_tuple_fia_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        id,
        a_id,
        i_id,
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        50_000, // amount_minor (FIA I-2)
        0,      // discount_minor
        0,      // paid_amount_minor
        Some("Term 1 installment".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple + valid due_date must construct");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(agg.amount_minor, 50_000);
    assert_eq!(agg.discount_minor, 0);
    assert_eq!(agg.paid_amount_minor, 0);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_distinct_scope_key_tuples_within_same_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        10_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: distinct scope-key tuple must construct");
    let agg_b = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
        10_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: distinct scope-key tuple must construct");
    assert_ne!(agg_a.fees_assign_id, agg_b.fees_assign_id);
    assert_ne!(agg_a.fees_installment_id, agg_b.fees_installment_id);
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
    assert_eq!(agg.correlation_id, tenant.correlation_id);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_fees_installment_assign_service_emits_created_event_fia() {
    use educore_finance::commands::CreateFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
        fees_assign_id: a_id,
        fees_installment_id: i_id,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
        amount_minor: 25_000,
        discount_minor: 0,
        paid_amount_minor: 0,
        note: Some("Service integration test".to_owned()),
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignCreated) =
        create_fees_installment_assign(cmd, &clock, &ids)
            .expect("create_fees_installment_assign must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(event.fees_installment_assign_id, agg.id);
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(event.discount_minor, 0);
    assert_eq!(event.paid_amount_minor, 0);
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.created"
    );
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign"
    );
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn retire_fees_installment_assign_service_emits_retired_event_fia() {
    use educore_finance::commands::RetireFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignRetired) =
        retire_fees_installment_assign(cmd, &clock, &ids)
            .expect("retire_fees_installment_assign must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_installment_assign_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.retired"
    );
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign"
    );
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

// =========================================================================
// FIA I-2 tests (Wave 110 new tests for amount/discount/paid tracking)
// =========================================================================

#[test]
fn fresh_zero_amount_and_zero_paid_boundary_valid_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        0,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-2: zero/zero/zero is valid (boundary)");
    assert_eq!(agg.amount_minor, 0);
    assert_eq!(agg.discount_minor, 0);
    assert_eq!(agg.paid_amount_minor, 0);
}

#[test]
fn fresh_negative_amount_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        -1,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_discount_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        -100,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative discount_minor must be rejected");
    assert!(
        format!("{err}").contains("discount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_paid_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        -1,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative paid_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("paid_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_paid_exceeds_cap_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    // amount=5000 + discount=500 = cap=5500; paid=6000 must be rejected.
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        500,
        6_000,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: paid > amount + discount must be rejected");
    assert!(
        format!("{err}")
            .contains("paid_amount_minor must be <= amount_minor + discount_minor"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_paid_equals_cap_boundary_valid_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    // amount=5000 + discount=500 = cap=5500; paid=5500 is the valid boundary.
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        500,
        5_500,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-2: paid == amount + discount is the valid boundary");
    assert_eq!(agg.paid_amount_minor, 5_500);
    assert_eq!(agg.amount_minor, 5_000);
    assert_eq!(agg.discount_minor, 500);
}

#[test]
fn create_fees_installment_assign_service_propagates_payment_fields_fia_i_2() {
    use educore_finance::commands::CreateFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
        fees_assign_id: a_id,
        fees_installment_id: i_id,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
        amount_minor: 100_000,
        discount_minor: 10_000,
        paid_amount_minor: 25_000,
        note: None,
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignCreated) =
        create_fees_installment_assign(cmd, &clock, &ids)
            .expect("create_fees_installment_assign must succeed");
    assert_eq!(agg.amount_minor, 100_000);
    assert_eq!(agg.discount_minor, 10_000);
    assert_eq!(agg.paid_amount_minor, 25_000);
    assert_eq!(event.amount_minor, 100_000);
    assert_eq!(event.discount_minor, 10_000);
    assert_eq!(event.paid_amount_minor, 25_000);
}
