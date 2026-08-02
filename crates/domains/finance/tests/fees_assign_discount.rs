//! Integration tests for the **FeesAssignDiscount aggregate** vertical slice.
//!
//! Pins FAD I-3 end-to-end: a FeesAssignDiscount records
//! timestamps via the standard audit footer (`created_at` +
//! `updated_at`) + the event-level `occurred_at` timestamp on
//! the emitted event. The aggregate has a public
//! `has_recorded_timestamps()` helper that returns `true` after
//! `fresh()` succeeds. Companion invariants: FAD I-1
//! (`applied_amount_minor >= 0` + `unapplied_amount_minor >= 0`)
//! + FAD I-2 (no mutator exposes an update path).
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

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_fees_assign_discount, retire_fees_assign_discount, Currency, FeesAssignDiscountCreated,
    FeesAssignDiscountId, FeesAssignDiscountRetired, FeesAssignId, FeesDiscountId,
    RealFeesAssignDiscount,
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

fn fad_id(g: &SystemIdGen, school: SchoolId) -> FeesAssignDiscountId {
    FeesAssignDiscountId::new(school, g.next_uuid())
}

fn fees_assign_id(g: &SystemIdGen, school: SchoolId) -> FeesAssignId {
    FeesAssignId::new(school, g.next_uuid())
}

fn fees_discount_id(g: &SystemIdGen, school: SchoolId) -> FeesDiscountId {
    FeesDiscountId::new(school, g.next_uuid())
}

// =========================================================================
// FAD I-3 typed-id smoke
// =========================================================================

#[test]
fn fees_assign_discount_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_assign_discount_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fad_id(&g, school);
    let id_b = fad_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// FAD I-3 construction tests
// =========================================================================

#[test]
fn fresh_full_payload_records_timestamps_fad_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        2_000,
        1_000,
        Currency::INR,
        Some("Scholarship discount".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD I-3: fresh must construct with all amounts valid");
    // FAD I-3: has_recorded_timestamps() must be true after fresh.
    assert!(agg.has_recorded_timestamps());
    // FAD I-3: created_at + updated_at are populated (not default).
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
    // FAD I-1: applied_amount_minor + unapplied_amount_minor are valid.
    assert_eq!(agg.applied_amount_minor, 2_000);
    assert_eq!(agg.unapplied_amount_minor, 1_000);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_amounts_boundary_valid_fad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        0,
        0,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD I-1: zero amounts are valid boundary");
    assert_eq!(agg.applied_amount_minor, 0);
    assert_eq!(agg.unapplied_amount_minor, 0);
    assert!(agg.has_recorded_timestamps());
}

#[test]
fn fresh_negative_applied_amount_validation_error_fad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        -1,
        0,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FAD I-1: negative applied_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("applied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_unapplied_amount_validation_error_fad_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        0,
        -1,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FAD I-1: negative unapplied_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("unapplied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_carries_distinct_scope_key_fk_references() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesAssignDiscount::fresh(
        fad_id(&g, school),
        fees_assign_id(&g, school),
        fees_discount_id(&g, school),
        1_000,
        500,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD: distinct FK refs must construct");
    let agg_b = RealFeesAssignDiscount::fresh(
        fad_id(&g, school),
        fees_assign_id(&g, school),
        fees_discount_id(&g, school),
        1_000,
        500,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD: distinct FK refs must construct");
    assert_ne!(agg_a.fees_assign_id, agg_b.fees_assign_id);
    assert_ne!(agg_a.discount_id, agg_b.discount_id);
    assert_eq!(agg_a.school_id, agg_b.school_id);
    assert!(agg_a.has_recorded_timestamps());
    assert!(agg_b.has_recorded_timestamps());
}

#[test]
fn retire_flips_active_status_to_retired_fad_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        1_000,
        500,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD I-3: fresh must construct");
    assert!(agg.is_active());
    let retire_at = educore_core::value_objects::Timestamp::now();
    agg.retire(retire_at, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
    // FAD I-3: updated_at advances on retire.
    assert_eq!(agg.updated_at, retire_at);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesAssignDiscount::fresh(
        id,
        a_id,
        d_id,
        1_000,
        500,
        Currency::INR,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FAD I-3: fresh must construct");
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
// FAD I-3 service integration tests
// =========================================================================

#[test]
fn create_fees_assign_discount_service_emits_created_event_fad_i_3() {
    use educore_finance::commands::CreateFeesAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let a_id = fees_assign_id(&g, school);
    let d_id = fees_discount_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_assign_discount_id: id,
        fees_assign_id: a_id,
        discount_id: d_id,
        applied_amount_minor: 3_000,
        unapplied_amount_minor: 1_500,
        currency: Currency::INR,
        note: Some("Service integration FAD I-3".to_owned()),
    };
    let (agg, event): (RealFeesAssignDiscount, FeesAssignDiscountCreated) =
        create_fees_assign_discount(cmd, &clock, &ids)
            .expect("create_fees_assign_discount must succeed");
    assert!(agg.is_active());
    assert!(agg.has_recorded_timestamps());
    assert_eq!(agg.applied_amount_minor, 3_000);
    assert_eq!(agg.unapplied_amount_minor, 1_500);
    assert_eq!(event.fees_assign_discount_id, agg.id);
    assert_eq!(event.applied_amount_minor, 3_000);
    assert_eq!(event.unapplied_amount_minor, 1_500);
    assert_eq!(event.fees_assign_id, a_id);
    assert_eq!(event.discount_id, d_id);
    // FAD I-3: event-level occurred_at timestamp is recorded.
    assert_eq!(
        <FeesAssignDiscountCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign_discount.created"
    );
    assert_eq!(
        <FeesAssignDiscountCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_assign_discount"
    );
    assert_eq!(
        <FeesAssignDiscountCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_fees_assign_discount_service_rejects_negative_applied_fad_i_1() {
    use educore_finance::commands::CreateFeesAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_assign_discount_id: id,
        fees_assign_id: fees_assign_id(&g, school),
        discount_id: fees_discount_id(&g, school),
        applied_amount_minor: -100,
        unapplied_amount_minor: 0,
        currency: Currency::INR,
        note: None,
    };
    let err = create_fees_assign_discount(cmd, &clock, &ids)
        .expect_err("FAD I-1: negative applied_amount_minor must be rejected at service layer");
    assert!(
        format!("{err}").contains("applied_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_fees_assign_discount_service_emits_retired_event_fad_i_3() {
    use educore_finance::commands::RetireFeesAssignDiscountCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fad_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesAssignDiscountCommand {
        tenant: tenant.clone(),
        fees_assign_discount_id: id,
    };
    let (agg, event): (RealFeesAssignDiscount, FeesAssignDiscountRetired) =
        retire_fees_assign_discount(cmd, &clock, &ids)
            .expect("retire_fees_assign_discount must succeed");
    assert!(!agg.is_active());
    assert!(agg.has_recorded_timestamps());
    assert_eq!(event.fees_assign_discount_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesAssignDiscountRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_assign_discount.retired"
    );
    assert_eq!(
        <FeesAssignDiscountRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_assign_discount"
    );
    assert_eq!(
        <FeesAssignDiscountRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}
