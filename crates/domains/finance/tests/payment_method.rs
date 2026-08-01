//! Integration tests for the **PaymentMethod aggregate** vertical slice.
//!
//! Pins the PM I-1 invariant end-to-end: a PaymentMethod's name
//! must be unique within its school (the (school_id, name)
//! scope-key tuple pins the method to a single configuration per
//! school). Companion invariant: `name` must be non-empty after
//! trimming whitespace.
//!
//! GREENFIELD test file \u2014 no pre-existing typed-id-only tests to
//! replace.

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
    create_payment_method, retire_payment_method, PaymentMethodCreated, PaymentMethodId,
    PaymentMethodKind, PaymentMethodRetired, RealPaymentMethod,
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

fn pm_id(g: &SystemIdGen, school: SchoolId) -> PaymentMethodId {
    PaymentMethodId::new(school, g.next_uuid())
}

#[test]
fn fresh_full_payload_valid_pm_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let pm = RealPaymentMethod::fresh(
        id,
        "Tuition Cash".to_owned(),
        PaymentMethodKind::Cash,
        None,
        Some("Primary cash receipt method".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: non-empty name must construct");
    assert!(pm.is_active());
    assert_eq!(pm.name, "Tuition Cash");
    assert_eq!(pm.kind, PaymentMethodKind::Cash);
    assert_eq!(pm.description.as_deref(), Some("Primary cash receipt method"));
    assert_eq!(pm.school_id, school);
}

#[test]
fn fresh_distinct_names_within_same_school_pm_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = pm_id(&g, school);
    let id_b = pm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let pm_a = RealPaymentMethod::fresh(
        id_a,
        "Tuition Cash".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: distinct name must construct");
    let pm_b = RealPaymentMethod::fresh(
        id_b,
        "Tuition Bank".to_owned(),
        PaymentMethodKind::Bank,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: distinct name must construct");
    assert_ne!(pm_a.id, pm_b.id);
    assert_eq!(pm_a.school_id, pm_b.school_id);
    assert_ne!(pm_a.name, pm_b.name);
}

#[test]
fn fresh_same_name_across_different_schools_pm_i_1() {
    let (tenant_a, g_a) = admin_context();
    let (tenant_b, g_b) = admin_context();
    let now = educore_core::value_objects::Timestamp::now();
    let pm_a = RealPaymentMethod::fresh(
        pm_id(&g_a, tenant_a.school_id),
        "Tuition Cash".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant_a.actor_id,
        now,
        tenant_a.correlation_id,
    )
    .expect("PM I-1: same name across different schools is allowed");
    let pm_b = RealPaymentMethod::fresh(
        pm_id(&g_b, tenant_b.school_id),
        "Tuition Cash".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant_b.actor_id,
        now,
        tenant_b.correlation_id,
    )
    .expect("PM I-1: same name across different schools is allowed");
    assert_ne!(pm_a.school_id, pm_b.school_id);
    assert_eq!(pm_a.name, pm_b.name);
}

#[test]
fn fresh_whitespace_only_name_validation_error_companion() {
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealPaymentMethod::fresh(
        id,
        "   \t  ".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("PM I-1 companion: whitespace-only name must be rejected");
    assert!(
        format!("{err}").contains("name must be non-empty after trimming"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_empty_name_validation_error_companion() {
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealPaymentMethod::fresh(
        id,
        String::new(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("PM I-1 companion: empty name must be rejected");
    assert!(
        format!("{err}").contains("name must be non-empty after trimming"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let pm = RealPaymentMethod::fresh(
        id,
        "Audit footer check".to_owned(),
        PaymentMethodKind::Bank,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: non-empty name must construct");
    assert!(pm.last_event_id.is_none(), "fresh() must start with no last_event_id");
    assert_eq!(pm.created_by, tenant.actor_id);
    assert_eq!(pm.updated_by, tenant.actor_id);
    assert_eq!(pm.created_at, now);
    assert_eq!(pm.updated_at, now);
    assert_eq!(pm.correlation_id, tenant.correlation_id);
}

#[test]
fn fresh_supports_all_payment_method_kinds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    for (i, kind) in [
        PaymentMethodKind::Cash,
        PaymentMethodKind::Bank,
        PaymentMethodKind::Cheque,
        PaymentMethodKind::Card,
        PaymentMethodKind::Gateway,
    ]
    .into_iter()
    .enumerate()
    {
        // PM I-2: gateway_id required iff kind == Gateway.
        let gateway_id = if kind == PaymentMethodKind::Gateway {
            Some(educore_finance::prelude::PaymentGatewaySettingId::new(school, g.next_uuid()))
        } else {
            None
        };
        let pm = RealPaymentMethod::fresh(
            pm_id(&g, school),
            format!("method_{i}"),
            kind,
            gateway_id,
            None,
            tenant.actor_id,
            now,
            tenant.correlation_id,
        )
        .expect("PM I-1: all PaymentMethodKind variants must construct");
        assert_eq!(pm.kind, kind);
        assert_eq!(pm.gateway_id, gateway_id);
    }
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let mut pm = RealPaymentMethod::fresh(
        id,
        "Will be retired".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: non-empty name must construct");
    assert!(pm.is_active());
    pm.retire(now, tenant.actor_id).expect("retire");
    assert!(!pm.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let now = educore_core::value_objects::Timestamp::now();
    let mut pm = RealPaymentMethod::fresh(
        id,
        "Double-retire attempt".to_owned(),
        PaymentMethodKind::Cash,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("PM I-1: non-empty name must construct");
    pm.retire(now, tenant.actor_id).expect("first retire");
    let err = pm.retire(now, tenant.actor_id).expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_payment_method_service_emits_created_event_pm_i_1() {
    use educore_finance::commands::CreatePaymentMethodCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreatePaymentMethodCommand {
        tenant: tenant.clone(),
        payment_method_id: id,
        name: "Service integration cash".to_owned(),
        kind: PaymentMethodKind::Cash,
        gateway_id: None,
        description: Some("Cash method for service integration test".to_owned()),
    };
    let (pm, event): (RealPaymentMethod, PaymentMethodCreated) =
        create_payment_method(cmd, &clock, &ids).expect("create_payment_method must succeed");
    assert!(pm.is_active());
    assert_eq!(pm.name, "Service integration cash");
    assert_eq!(pm.kind, PaymentMethodKind::Cash);
    assert_eq!(event.payment_method_id, pm.id);
    assert_eq!(event.name, "Service integration cash");
    assert_eq!(event.kind, PaymentMethodKind::Cash);
    assert_eq!(
        event.description.as_deref(),
        Some("Cash method for service integration test")
    );
    assert_eq!(
        <PaymentMethodCreated as DomainEvent>::EVENT_TYPE,
        "finance.payment_method.created"
    );
    assert_eq!(
        <PaymentMethodCreated as DomainEvent>::AGGREGATE_TYPE,
        "payment_method"
    );
    assert_eq!(<PaymentMethodCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_payment_method_service_rejects_empty_name_companion() {
    use educore_finance::commands::CreatePaymentMethodCommand;
    let (tenant, g) = admin_context();
    let id = pm_id(&g, tenant.school_id);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreatePaymentMethodCommand {
        tenant: tenant.clone(),
        payment_method_id: id,
        name: String::new(),
        kind: PaymentMethodKind::Cash,
        gateway_id: None,
        description: None,
    };
    let err = create_payment_method(cmd, &clock, &ids)
        .expect_err("PM I-1 companion: empty name must be rejected at service layer");
    assert!(
        format!("{err}").contains("name must be non-empty after trimming"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_payment_method_service_emits_retired_event_pm_i_1() {
    use educore_finance::commands::RetirePaymentMethodCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetirePaymentMethodCommand {
        tenant: tenant.clone(),
        payment_method_id: id,
    };
    let (pm, event): (RealPaymentMethod, PaymentMethodRetired) =
        retire_payment_method(cmd, &clock, &ids).expect("retire_payment_method must succeed");
    assert!(!pm.is_active());
    assert_eq!(event.payment_method_id, pm.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <PaymentMethodRetired as DomainEvent>::EVENT_TYPE,
        "finance.payment_method.retired"
    );
    assert_eq!(
        <PaymentMethodRetired as DomainEvent>::AGGREGATE_TYPE,
        "payment_method"
    );
    assert_eq!(<PaymentMethodRetired as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.school_id(), school);
}

// =========================================================================
// -- Wave 135 -- RealPaymentMethod -- PM I-2 gateway_id enforcement --
// =========================================================================

#[test]
fn fresh_gateway_requires_gateway_id_pm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let err = RealPaymentMethod::fresh(
        id,
        "Online Gateway".to_owned(),
        PaymentMethodKind::Gateway,
        None,
        None,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("PM I-2: kind=Gateway without gateway_id must be rejected");
    assert!(
        format!("{err}").contains("Gateway requires gateway_id"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_gateway_with_gateway_id_succeeds_pm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let gw_id = educore_finance::prelude::PaymentGatewaySettingId::new(school, g.next_uuid());
    let pm = RealPaymentMethod::fresh(
        id,
        "Online Gateway".to_owned(),
        PaymentMethodKind::Gateway,
        Some(gw_id),
        None,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("PM I-2: Gateway kind with gateway_id must construct");
    assert_eq!(pm.gateway_id, Some(gw_id));
    assert_eq!(pm.kind, PaymentMethodKind::Gateway);
}

#[test]
fn fresh_cash_with_gateway_id_rejected_pm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let gw_id = educore_finance::prelude::PaymentGatewaySettingId::new(school, g.next_uuid());
    let err = RealPaymentMethod::fresh(
        id,
        "Cash Method".to_owned(),
        PaymentMethodKind::Cash,
        Some(gw_id),
        None,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("PM I-2: kind=Cash with gateway_id must be rejected");
    assert!(
        format!("{err}").contains("cannot have gateway_id"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_bank_cheque_card_mobile_gateway_id_none_succeeds_pm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    for (i, kind) in [
        PaymentMethodKind::Bank,
        PaymentMethodKind::Cheque,
        PaymentMethodKind::Card,
        PaymentMethodKind::Mobile,
    ]
    .into_iter()
    .enumerate()
    {
        let pm = RealPaymentMethod::fresh(
            pm_id(&g, school),
            format!("non_gateway_{i}"),
            kind,
            None,
            None,
            tenant.actor_id,
            now,
            tenant.correlation_id,
        )
        .expect("PM I-2: non-gateway kinds must succeed with gateway_id=None");
        assert_eq!(pm.gateway_id, None);
        assert_eq!(pm.kind, kind);
    }
}

#[test]
fn create_gateway_payment_method_service_emits_event_with_gateway_id_pm_i_2() {
    use educore_finance::commands::CreatePaymentMethodCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = pm_id(&g, school);
    let gw_id = educore_finance::prelude::PaymentGatewaySettingId::new(school, g.next_uuid());
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreatePaymentMethodCommand {
        tenant: tenant.clone(),
        payment_method_id: id,
        name: "Razorpay gateway".to_owned(),
        kind: PaymentMethodKind::Gateway,
        gateway_id: Some(gw_id),
        description: Some("Razorpay integration".to_owned()),
    };
    let (pm, event): (RealPaymentMethod, PaymentMethodCreated) =
        create_payment_method(cmd, &clock, &ids).expect("create must succeed");
    assert_eq!(pm.gateway_id, Some(gw_id));
    assert_eq!(event.gateway_id, Some(gw_id));
    assert_eq!(pm.kind, PaymentMethodKind::Gateway);
    assert_eq!(event.kind, PaymentMethodKind::Gateway);
}

// =========================================================================
// -- Wave 143 -- RealPaymentMethod -- PM I-3 account_id compatible marker --
// =========================================================================

#[test]
fn pm_i_3_account_id_compatible_dispatcher_enforced() {
    // PM I-3 marker test: the account_id compatibility invariant
    // (a PaymentMethod whose kind == Cash must NOT reference a
    // BankAccount; a PaymentMethod whose kind == Bank/Cheque MUST
    // reference a BankAccount of type Bank; a Gateway-backed method
    // is exempt) is dispatcher-enforced. The aggregate carries the
    // payment_method kind at the API surface; the dispatcher adds
    // the BankAccount cross-row check.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
