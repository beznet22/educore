//! Integration tests for the **FeesInstallmentAssignDiscount child
//! aggregate** vertical slice.
//!
//! Pins the FIAD I-1 invariant end-to-end: a discount applied to
//! an installment assignment must have `applied_amount_minor >=
//! 0` (a negative applied amount would silently inflate the
//! student's balance rather than reduce it). Companion invariants:
//! `discount_id` + `fees_installment_assign_id` are required FK
//! references; `currency` is required (companion: discounts must
//! be in the same currency as the underlying assignment).
//!
//! Replaces the prior 2 typed-id-only tests with a 13-test
//! behavioral suite that exercises construction, validation,
//! audit-footer, retire, and service integration paths.

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
    create_fees_installment_assign_discount, retire_fees_installment_assign_discount, Currency,
    FeesDiscountId, FeesInstallmentAssignDiscountCreated, FeesInstallmentAssignDiscountId,
    FeesInstallmentAssignDiscountRetired, FeesInstallmentAssignId,
    RealFeesInstallmentAssignDiscount,
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

fn fiad_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentAssignDiscountId {
    FeesInstallmentAssignDiscountId::new(school, g.next_uuid())
}

fn discount_id(g: &SystemIdGen, school: SchoolId) -> FeesDiscountId {
    FeesDiscountId::new(school, g.next_uuid())
}

fn assignment_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentAssignId {
    FeesInstallmentAssignId::new(school, g.next_uuid())
}

#[test]
fn fees_installment_assign_discount_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_installment_assign_discount_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fiad_id(&g, school);
    let id_b = fiad_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

#[test]
fn fresh_full_payload_applied_amount_valid_fiad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        5_000,
        Currency::INR,
        Some("Sibling discount".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: positive applied_amount_minor must construct");
    assert!(agg.is_active());
    assert_eq!(agg.applied_amount_minor, 5_000);
    assert_eq!(agg.discount_id, d_id);
    assert_eq!(agg.fees_installment_assign_id, a_id);
    assert_eq!(agg.currency, Currency::INR);
    assert_eq!(agg.note.as_deref(), Some("Sibling discount"));
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_applied_amount_boundary_valid_fiad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        0,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: zero is a valid boundary");
    assert_eq!(agg.applied_amount_minor, 0);
    assert!(agg.note.is_none());
}

#[test]
fn fresh_negative_applied_amount_validation_error_fiad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        -1,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIAD I-1: negative applied_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("applied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_large_negative_applied_amount_validation_error_fiad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        -1_000_000,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIAD I-1: large negative applied_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("applied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        2_500,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: positive applied_amount_minor must construct");
    assert!(agg.last_event_id.is_none(), "fresh() must start with no last_event_id");
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
    assert_eq!(agg.correlation_id, tenant.correlation_id);
}

#[test]
fn fresh_carries_optional_note_through_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        10_000,
        Currency::INR,
        Some("Merit scholarship — 50% tuition".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: positive applied_amount_minor must construct");
    assert_eq!(agg.note.as_deref(), Some("Merit scholarship — 50% tuition"));
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        1_000,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: positive applied_amount_minor must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssignDiscount::fresh(
        id,
        d_id,
        a_id,
        1_000,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIAD I-1: positive applied_amount_minor must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg.retire(now, tenant.actor_id).expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_fees_installment_assign_discount_service_emits_created_event_fiad() {
    use educore_finance::commands::CreateFeesInstallmentAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_installment_assign_discount_id: id,
        discount_id: d_id,
        fees_installment_assign_id: a_id,
        applied_amount_minor: 7_500,
        currency: Currency::INR,
        note: Some("Service integration — merit award".to_owned()),
    };
    let (agg, event): (RealFeesInstallmentAssignDiscount, FeesInstallmentAssignDiscountCreated) =
        create_fees_installment_assign_discount(cmd, &clock, &ids)
            .expect("create_fees_installment_assign_discount must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.applied_amount_minor, 7_500);
    assert_eq!(agg.discount_id, d_id);
    assert_eq!(agg.fees_installment_assign_id, a_id);
    assert_eq!(event.fees_installment_assign_discount_id, agg.id);
    assert_eq!(event.applied_amount_minor, 7_500);
    assert_eq!(event.discount_id, d_id);
    assert_eq!(event.fees_installment_assign_id, a_id);
    assert_eq!(event.note.as_deref(), Some("Service integration — merit award"));
    assert_eq!(
        <FeesInstallmentAssignDiscountCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign_discount.created"
    );
    assert_eq!(
        <FeesInstallmentAssignDiscountCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign_discount"
    );
    assert_eq!(
        <FeesInstallmentAssignDiscountCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_fees_installment_assign_discount_service_rejects_negative_fiad_i_1() {
    use educore_finance::commands::CreateFeesInstallmentAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let d_id = discount_id(&g, school);
    let a_id = assignment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_installment_assign_discount_id: id,
        discount_id: d_id,
        fees_installment_assign_id: a_id,
        applied_amount_minor: -100,
        currency: Currency::INR,
        note: None,
    };
    let err = create_fees_installment_assign_discount(cmd, &clock, &ids)
        .expect_err("FIAD I-1: negative applied_amount_minor must be rejected at service layer");
    assert!(
        format!("{err}").contains("applied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_fees_installment_assign_discount_service_emits_retired_event_fiad() {
    use educore_finance::commands::RetireFeesInstallmentAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiad_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesInstallmentAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_installment_assign_discount_id: id,
    };
    let (agg, event): (RealFeesInstallmentAssignDiscount, FeesInstallmentAssignDiscountRetired) =
        retire_fees_installment_assign_discount(cmd, &clock, &ids)
            .expect("retire_fees_installment_assign_discount must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_installment_assign_discount_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesInstallmentAssignDiscountRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign_discount.retired"
    );
    assert_eq!(
        <FeesInstallmentAssignDiscountRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign_discount"
    );
    assert_eq!(
        <FeesInstallmentAssignDiscountRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}
