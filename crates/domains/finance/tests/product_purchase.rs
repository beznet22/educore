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

// ====================================================================
// -- Wave 137 -- PPr I-2 (supplier_reference) + PPr I-3 (state machine) --
// ====================================================================

use educore_core::ids::UserId;
use educore_core::value_objects::Timestamp;
use educore_finance::commands::{
    CancelProductPurchaseCommand, RecordProductPurchaseReceiptCommand,
};
use educore_finance::events::{ProductPurchaseCancelled, ProductPurchaseReceived};
use educore_finance::services::{cancel_product_purchase, record_product_purchase_receipt};
use educore_finance::value_objects::ProductPurchaseLifecycleStatus;

fn build_pp(actor: UserId, supplier_reference: Option<String>) -> RealProductPurchase {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    RealProductPurchase::fresh(
        product_purchase_id(&g, school),
        "Test product".to_owned(),
        5,
        5_000,
        supplier_reference,
        actor,
        Timestamp::now(),
        _tenant.correlation_id,
    )
    .expect("fresh should succeed")
}

// ---- PPr I-2: supplier_reference non-empty after trim when Some ----

#[test]
fn fresh_empty_supplier_reference_validation_error_ppr_i_2() {
    let (tenant, _g) = admin_context();
    let err = RealProductPurchase::fresh(
        product_purchase_id(&_g, tenant.school_id),
        "Test product".to_owned(),
        5,
        5_000,
        Some("   ".to_owned()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("PPr I-2: whitespace supplier_reference must be rejected");
    assert!(
        format!("{err}").contains("supplier_reference must be non-empty after trim"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_supplier_reference_is_trimmed_ppr_i_2() {
    let (tenant, g) = admin_context();
    let pp = RealProductPurchase::fresh(
        product_purchase_id(&g, tenant.school_id),
        "Test product".to_owned(),
        5,
        5_000,
        Some("  ACME-SUPPLIER-001  ".to_owned()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("PPr I-2: whitespace-padded supplier_reference must succeed and be trimmed");
    assert_eq!(pp.supplier_reference.as_deref(), Some("ACME-SUPPLIER-001"));
}

#[test]
fn fresh_none_supplier_reference_succeeds_ppr_i_2() {
    let (tenant, _g) = admin_context();
    let pp = build_pp(tenant.actor_id, None);
    assert_eq!(pp.supplier_reference, None);
}

// ---- ProductPurchaseLifecycleStatus enum round-trip ----

#[test]
fn ppr_lifecycle_status_as_str_round_trip_ppr_i_3() {
    assert_eq!(ProductPurchaseLifecycleStatus::Draft.as_str(), "draft");
    assert_eq!(ProductPurchaseLifecycleStatus::Received.as_str(), "received");
    assert_eq!(ProductPurchaseLifecycleStatus::Cancelled.as_str(), "cancelled");
    assert_eq!(ProductPurchaseLifecycleStatus::parse("draft"), Some(ProductPurchaseLifecycleStatus::Draft));
    assert_eq!(ProductPurchaseLifecycleStatus::parse("received"), Some(ProductPurchaseLifecycleStatus::Received));
    assert_eq!(ProductPurchaseLifecycleStatus::parse("cancelled"), Some(ProductPurchaseLifecycleStatus::Cancelled));
    assert_eq!(ProductPurchaseLifecycleStatus::parse("unknown"), None);
}

#[test]
fn ppr_lifecycle_can_transition_only_from_draft_ppr_i_3() {
    assert!(ProductPurchaseLifecycleStatus::Draft.can_transition_to(ProductPurchaseLifecycleStatus::Received));
    assert!(ProductPurchaseLifecycleStatus::Draft.can_transition_to(ProductPurchaseLifecycleStatus::Cancelled));
    assert!(!ProductPurchaseLifecycleStatus::Received.can_transition_to(ProductPurchaseLifecycleStatus::Draft));
    assert!(!ProductPurchaseLifecycleStatus::Received.can_transition_to(ProductPurchaseLifecycleStatus::Cancelled));
    assert!(!ProductPurchaseLifecycleStatus::Cancelled.can_transition_to(ProductPurchaseLifecycleStatus::Draft));
    assert!(!ProductPurchaseLifecycleStatus::Cancelled.can_transition_to(ProductPurchaseLifecycleStatus::Received));
}

// ---- fresh initializes lifecycle + audit footer ----

#[test]
fn fresh_initializes_lifecycle_draft_ppr_i_3() {
    let (tenant, _g) = admin_context();
    let pp = build_pp(tenant.actor_id, None);
    assert_eq!(pp.lifecycle_status, ProductPurchaseLifecycleStatus::Draft);
    assert_eq!(pp.received_by, None);
    assert_eq!(pp.received_at, None);
    assert_eq!(pp.cancelled_by, None);
    assert_eq!(pp.cancelled_at, None);
    assert_eq!(pp.cancel_reason, None);
}

// ---- record_receipt mutator ----

#[test]
fn record_receipt_transitions_draft_to_received_ppr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut pp = build_pp(actor, Some("ACME".to_owned()));
    let at = Timestamp::now();
    let event_id = g.next_event_id();
    pp.record_receipt(actor, at, event_id).expect("record_receipt should succeed");
    assert_eq!(pp.lifecycle_status, ProductPurchaseLifecycleStatus::Received);
    assert_eq!(pp.received_by, Some(actor));
    assert_eq!(pp.received_at, Some(at));
    assert_eq!(pp.last_event_id, Some(event_id));
}

#[test]
fn double_record_receipt_returns_conflict_ppr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut pp = build_pp(actor, None);
    pp.record_receipt(actor, Timestamp::now(), g.next_event_id()).expect("first receipt");
    let result = pp.record_receipt(actor, Timestamp::now(), g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- cancel mutator ----

#[test]
fn cancel_draft_transitions_to_cancelled_ppr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut pp = build_pp(actor, Some("ACME".to_owned()));
    let at = Timestamp::now();
    let event_id = g.next_event_id();
    pp.cancel(actor, "Vendor out of stock".to_owned(), at, event_id)
        .expect("cancel should succeed");
    assert_eq!(pp.lifecycle_status, ProductPurchaseLifecycleStatus::Cancelled);
    assert_eq!(pp.cancelled_by, Some(actor));
    assert_eq!(pp.cancelled_at, Some(at));
    assert_eq!(pp.cancel_reason.as_deref(), Some("Vendor out of stock"));
}

#[test]
fn cancel_after_receipt_returns_conflict_ppr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut pp = build_pp(actor, None);
    pp.record_receipt(actor, Timestamp::now(), g.next_event_id()).expect("receipt");
    let result = pp.cancel(actor, "too late".to_owned(), Timestamp::now(), g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn cancel_empty_reason_validation_error_ppr_i_3() {
    let (tenant, g) = admin_context();
    let actor = tenant.actor_id;
    let mut pp = build_pp(actor, None);
    let result = pp.cancel(actor, "   ".to_owned(), Timestamp::now(), g.next_event_id());
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

// ---- service integration ----

#[test]
fn record_receipt_service_emits_event_ppr_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealProductPurchase::fresh(
        product_purchase_id(&g, school),
        "Service test product".to_owned(),
        5,
        5_000,
        Some("ACME".to_owned()),
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = RecordProductPurchaseReceiptCommand {
        tenant,
        product_purchase_id: id,
    };
    let (updated, evt): (RealProductPurchase, ProductPurchaseReceived) =
        record_product_purchase_receipt(agg, cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(updated.lifecycle_status, ProductPurchaseLifecycleStatus::Received);
    assert_eq!(evt.received_by, actor);
    assert_eq!(evt.lifecycle_status, ProductPurchaseLifecycleStatus::Received);
    assert_eq!(
        <ProductPurchaseReceived as DomainEvent>::EVENT_TYPE,
        "finance.product_purchase.received"
    );
}

#[test]
fn cancel_service_emits_event_ppr_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealProductPurchase::fresh(
        product_purchase_id(&g, school),
        "Cancel service test product".to_owned(),
        5,
        5_000,
        None,
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = CancelProductPurchaseCommand {
        tenant,
        product_purchase_id: id,
        cancel_reason: "Out of stock".to_owned(),
    };
    let (updated, evt): (RealProductPurchase, ProductPurchaseCancelled) =
        cancel_product_purchase(agg, cmd, &clock, &g).expect("service should succeed");
    assert_eq!(updated.lifecycle_status, ProductPurchaseLifecycleStatus::Cancelled);
    assert_eq!(evt.cancelled_by, actor);
    assert_eq!(evt.cancel_reason, "Out of stock");
    assert_eq!(
        <ProductPurchaseCancelled as DomainEvent>::EVENT_TYPE,
        "finance.product_purchase.cancelled"
    );
}
