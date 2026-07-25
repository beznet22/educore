//! Behavioural tests for `RealInventoryPayment` (Wave 98).
//!
//! Pins IP I-1 (`amount_minor >= 0`) end-to-end via the aggregate
//! surface, the service functions, and the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::events::{InventoryPaymentCreated, InventoryPaymentRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::InventoryPaymentId;

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

fn inventory_payment_id(g: &SystemIdGen, school: SchoolId) -> InventoryPaymentId {
    InventoryPaymentId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn inventory_payment_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = inventory_payment_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- IP I-1: amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let row = RealInventoryPayment::fresh(
        id,
        "ACME Stationery".to_string(),
        12_500,
        Currency::INR,
        Some("paper + pens bulk order".to_string()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 12_500");
    assert_eq!(row.amount_minor, 12_500);
    assert_eq!(row.supplier_name, "ACME Stationery");
    assert_eq!(row.currency, Currency::INR);
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let result = RealInventoryPayment::fresh(
        id,
        "ACME".to_string(),
        -1,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("IP I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_is_valid_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let row = RealInventoryPayment::fresh(
        id,
        "ACME".to_string(),
        0,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
}

// ---- supplier_name guard ----

#[test]
fn fresh_empty_supplier_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let result = RealInventoryPayment::fresh(
        id,
        "   ".to_string(),
        1000,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_supplier_name_is_trimmed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let row = RealInventoryPayment::fresh(
        id,
        "  ACME Stationery  ".to_string(),
        1000,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim supplier_name");
    assert_eq!(row.supplier_name, "ACME Stationery");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let row = RealInventoryPayment::fresh(
        id,
        "ACME".to_string(),
        5_000,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.last_event_id, None);
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let mut row = RealInventoryPayment::fresh(
        id,
        "ACME".to_string(),
        5_000,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.supplier_name, "ACME");
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = inventory_payment_id(&g, school);
    let mut row = RealInventoryPayment::fresh(
        id,
        "ACME".to_string(),
        5_000,
        Currency::INR,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("first retire should succeed");
    let result = row.retire(Timestamp::now(), tenant.actor_id);
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- service integration ----

#[test]
fn create_inventory_payment_service_emits_created_event_in_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = inventory_payment_id(&g, school);
    let cmd = CreateInventoryPaymentCommand {
        tenant,
        inventory_payment_id: id,
        supplier_name: "ACME".to_string(),
        amount_minor: 12_500,
        currency: Currency::INR,
        note: Some("bulk order".to_string()),
    };
    let evt: InventoryPaymentCreated =
        create_inventory_payment(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 12_500);
    assert_eq!(evt.supplier_name, "ACME");
    assert_eq!(evt.inventory_payment_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <InventoryPaymentCreated as DomainEvent>::EVENT_TYPE,
        "finance.inventory_payment.created"
    );
    assert_eq!(
        <InventoryPaymentCreated as DomainEvent>::AGGREGATE_TYPE,
        "inventory_payment"
    );
    assert_eq!(
        <InventoryPaymentCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_inventory_payment_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = inventory_payment_id(&g, school);
    let cmd = RetireInventoryPaymentCommand {
        tenant,
        inventory_payment_id: id,
    };
    let evt: InventoryPaymentRetired =
        retire_inventory_payment(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.inventory_payment_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <InventoryPaymentRetired as DomainEvent>::EVENT_TYPE,
        "finance.inventory_payment.retired"
    );
}
