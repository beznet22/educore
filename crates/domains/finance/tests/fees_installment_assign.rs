//! Integration tests for the **FeesInstallmentAssign aggregate** vertical slice.
//!
//! Pins the FIA I-1 invariant end-to-end: a FeesInstallmentAssign
//! is uniquely scoped to a (fees_assign_id, fees_installment_id)
//! tuple within a school. Uniqueness is dispatcher-enforced via
//! the scope-key tuple the aggregate carries as required fields.
//!
//! Replaces the prior 2 typed-id-only tests with a 10-test
//! behavioral suite.

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
        Some("Term 1 installment".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple + valid due_date must construct");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(agg.due_date, chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap());
    assert_eq!(agg.note.as_deref(), Some("Term 1 installment"));
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
fn fresh_accepts_past_due_date_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        Some("Historical reconciliation".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("companion: past due_date is allowed for historical reconciliation");
    assert_eq!(agg.due_date, chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
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
        note: Some("Service integration test".to_owned()),
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignCreated) =
        create_fees_installment_assign(cmd, &clock, &ids)
            .expect("create_fees_installment_assign must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(event.fees_installment_assign_id, agg.id);
    assert_eq!(event.fees_assign_id, a_id);
    assert_eq!(event.fees_installment_id, i_id);
    assert_eq!(event.note.as_deref(), Some("Service integration test"));
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
