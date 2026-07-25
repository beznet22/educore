//! Behavioural tests for `RealProductPurchase` (Wave 99).
//!
//! Pins PPr I-1 (`amount_minor >= 0`) end-to-end via the aggregate
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
use educore_finance::events::{ProductPurchaseCreated, ProductPurchaseRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::ProductPurchaseId;

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

fn product_purchase_id(g: &SystemIdGen, school: SchoolId) -> ProductPurchaseId {
    ProductPurchaseId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn product_purchase_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = product_purchase_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- PPr I-1: amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_ppr_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let row = RealProductPurchase::fresh(
        id,
        "A4 paper ream".to_string(),
        5,
        7_500,
        Some("Acme Stationery PO#12345".to_string()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 7_500");
    assert_eq!(row.amount_minor, 7_500);
    assert_eq!(row.product_name, "A4 paper ream");
    assert_eq!(row.quantity, 5);
    assert_eq!(row.supplier_reference.as_deref(), Some("Acme Stationery PO#12345"));
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_ppr_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let result = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        1,
        -1,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("PPr I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_is_valid_ppr_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let row = RealProductPurchase::fresh(
        id,
        "free sample".to_string(),
        1,
        0,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
}

// ---- companion invariants ----

#[test]
fn fresh_zero_quantity_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let result = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        0,
        1_000,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_negative_quantity_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let result = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        -3,
        1_000,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_empty_product_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let result = RealProductPurchase::fresh(
        id,
        "   ".to_string(),
        1,
        1_000,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_product_name_is_trimmed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let row = RealProductPurchase::fresh(
        id,
        "  A4 paper ream  ".to_string(),
        5,
        7_500,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim product_name");
    assert_eq!(row.product_name, "A4 paper ream");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let row = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        1,
        5_000,
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
    let id = product_purchase_id(&g, school);
    let mut row = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        1,
        5_000,
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
    assert_eq!(row.product_name, "A4 paper");
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = product_purchase_id(&g, school);
    let mut row = RealProductPurchase::fresh(
        id,
        "A4 paper".to_string(),
        1,
        5_000,
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
fn create_product_purchase_service_emits_created_event_ppr_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = product_purchase_id(&g, school);
    let cmd = CreateProductPurchaseCommand {
        tenant,
        product_purchase_id: id,
        product_name: "A4 paper".to_string(),
        quantity: 5,
        amount_minor: 7_500,
        supplier_reference: Some("PO#12345".to_string()),
    };
    let evt: ProductPurchaseCreated =
        create_product_purchase(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 7_500);
    assert_eq!(evt.product_name, "A4 paper");
    assert_eq!(evt.quantity, 5);
    assert_eq!(evt.product_purchase_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <ProductPurchaseCreated as DomainEvent>::EVENT_TYPE,
        "finance.product_purchase.created"
    );
    assert_eq!(
        <ProductPurchaseCreated as DomainEvent>::AGGREGATE_TYPE,
        "product_purchase"
    );
    assert_eq!(
        <ProductPurchaseCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_product_purchase_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = product_purchase_id(&g, school);
    let cmd = RetireProductPurchaseCommand {
        tenant,
        product_purchase_id: id,
    };
    let evt: ProductPurchaseRetired =
        retire_product_purchase(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.product_purchase_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <ProductPurchaseRetired as DomainEvent>::EVENT_TYPE,
        "finance.product_purchase.retired"
    );
}
