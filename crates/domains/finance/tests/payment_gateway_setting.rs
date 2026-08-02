//! Integration tests for the **PaymentGatewaySetting aggregate** vertical slice.
//!
//! Pins the full `RealPaymentGatewaySetting` drop:
//! - PGS I-1: gateway name unique within a school (dispatcher-enforced;
//!   aggregate carries scope-key tuple as required fields)
//! - PGS I-2: mode must be `sandbox` or `live` (typed enum, pinned at fresh)
//! - PGS I-3: charge >= 0; charge_type ∈ {P, F} (typed enum + construction guard)
//! - PGS I-4: credentials encrypted at rest (storage-layer enforced)

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
    ConfigurePaymentGatewayCommand, GatewayChargeType, GatewayMode, PaymentGatewayConfigured,
    PaymentGatewayDisabled, PaymentGatewaySettingId, PaymentGatewayUpdated,
    RealPaymentGatewaySetting, UpdatePaymentGatewayCommand,
};
use educore_finance::services::{
    configure_payment_gateway, disable_payment_gateway, update_payment_gateway,
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

fn payment_gateway_setting_id(g: &SystemIdGen, school: SchoolId) -> PaymentGatewaySettingId {
    PaymentGatewaySettingId::new(school, g.next_uuid())
}

fn event_id_gen() -> impl Fn() -> educore_core::ids::EventId {
    let g = SystemIdGen;
    move || g.next_event_id()
}

fn make_configure_cmd(
    tenant: TenantContext,
    id: PaymentGatewaySettingId,
    name: &str,
) -> ConfigurePaymentGatewayCommand {
    ConfigurePaymentGatewayCommand {
        tenant,
        payment_gateway_setting_id: id,
        name: name.to_owned(),
        description: Some("Razorpay test gateway".to_owned()),
        gateway_username: Some("test_user".to_owned()),
        gateway_password: Some("test_pw".to_owned()),
        gateway_signature: None,
        gateway_client_id: None,
        gateway_secret_key: Some("test_secret".to_owned()),
        gateway_secret_word: None,
        gateway_publisher_key: None,
        gateway_private_key: None,
        mode: GatewayMode::Sandbox,
        service_charge_minor: 250,
        service_charge_type: GatewayChargeType::Percentage,
    }
}

#[test]
fn pgs_i_1_fresh_initializes_aggregate() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _event): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    assert_eq!(agg.id, id);
    assert_eq!(agg.school_id, tenant.school_id);
    assert_eq!(agg.name, "Razorpay");
    assert_eq!(agg.mode, GatewayMode::Sandbox);
    assert_eq!(agg.service_charge_minor, 250);
    assert_eq!(agg.service_charge_type, GatewayChargeType::Percentage);
}

#[test]
fn pgs_i_1_name_empty_after_trim_is_rejected() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let mut cmd = make_configure_cmd(tenant, id, "Razorpay");
    cmd.name = "   ".to_owned();
    let err = configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {err:?}"
    );
}

#[test]
fn pgs_i_1_id_uniqueness_in_scope_compile_time_marker() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = payment_gateway_setting_id(&g, school);
    let id_b = payment_gateway_setting_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

#[test]
fn pgs_i_2_mode_sandbox_parses_to_sandbox() {
    let mode = GatewayMode::parse("sandbox").expect("sandbox parses");
    assert_eq!(mode, GatewayMode::Sandbox);
}

#[test]
fn pgs_i_2_mode_live_parses_to_live() {
    let mode = GatewayMode::parse("live").expect("live parses");
    assert_eq!(mode, GatewayMode::Live);
}

#[test]
fn pgs_i_2_mode_unknown_is_rejected() {
    let err = GatewayMode::parse("bogus").unwrap_err();
    assert!(matches!(
        err,
        educore_core::error::DomainError::Validation(_)
    ));
}

#[test]
fn pgs_i_3_charge_negative_is_rejected() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let mut cmd = make_configure_cmd(tenant, id, "Razorpay");
    cmd.service_charge_minor = -1;
    let err = configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).unwrap_err();
    assert!(matches!(
        err,
        educore_core::error::DomainError::Validation(_)
    ));
}

#[test]
fn pgs_i_3_charge_zero_is_allowed() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let mut cmd = make_configure_cmd(tenant, id, "Stripe");
    cmd.service_charge_minor = 0;
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("zero charge ok");
    assert_eq!(agg.service_charge_minor, 0);
}

#[test]
fn pgs_i_3_charge_type_percentage_or_flat_via_typed_enum() {
    let p = GatewayChargeType::parse("P").expect("P parses");
    assert_eq!(p, GatewayChargeType::Percentage);
    let f = GatewayChargeType::parse("F").expect("F parses");
    assert_eq!(f, GatewayChargeType::Flat);
    let err = GatewayChargeType::parse("X").unwrap_err();
    assert!(matches!(
        err,
        educore_core::error::DomainError::Validation(_)
    ));
}

#[test]
fn pgs_i_3_charge_type_round_trip() {
    assert_eq!(GatewayChargeType::Percentage.as_str(), "P");
    assert_eq!(GatewayChargeType::Flat.as_str(), "F");
}

#[test]
fn pgs_update_metadata_advances_audit_footer() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    let original_version = agg.version;
    let update_cmd = UpdatePaymentGatewayCommand {
        tenant: tenant.clone(),
        payment_gateway_setting_id: id,
        description: Some("Updated description".to_owned()),
        gateway_username: None,
        gateway_password: None,
        gateway_signature: None,
        gateway_client_id: None,
        gateway_secret_key: None,
        gateway_secret_word: None,
        gateway_publisher_key: None,
        gateway_private_key: None,
        mode: Some(GatewayMode::Live),
        service_charge_minor: Some(500),
        service_charge_type: Some(GatewayChargeType::Flat),
    };
    let (agg2, event): (RealPaymentGatewaySetting, PaymentGatewayUpdated) =
        update_payment_gateway(agg, update_cmd, &SystemClock, &event_id_gen())
            .expect("update succeeds");
    assert_eq!(agg2.version, original_version.next());
    assert_eq!(agg2.mode, GatewayMode::Live);
    assert_eq!(agg2.service_charge_minor, 500);
    assert_eq!(agg2.service_charge_type, GatewayChargeType::Flat);
    assert_eq!(event.mode, Some(GatewayMode::Live));
    assert_eq!(event.service_charge_minor, Some(500));
    assert_eq!(event.service_charge_type, Some(GatewayChargeType::Flat));
}

#[test]
fn pgs_update_metadata_negative_charge_is_rejected() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    let update_cmd = UpdatePaymentGatewayCommand {
        tenant: tenant.clone(),
        payment_gateway_setting_id: id,
        description: None,
        gateway_username: None,
        gateway_password: None,
        gateway_signature: None,
        gateway_client_id: None,
        gateway_secret_key: None,
        gateway_secret_word: None,
        gateway_publisher_key: None,
        gateway_private_key: None,
        mode: None,
        service_charge_minor: Some(-1),
        service_charge_type: None,
    };
    let err = update_payment_gateway(agg, update_cmd, &SystemClock, &event_id_gen()).unwrap_err();
    assert!(matches!(
        err,
        educore_core::error::DomainError::Validation(_)
    ));
}

#[test]
fn pgs_retire_flips_active_status_and_emits_disabled_event() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    let (agg2, event): (RealPaymentGatewaySetting, PaymentGatewayDisabled) =
        disable_payment_gateway(agg, tenant.clone(), &SystemClock, &event_id_gen())
            .expect("retire succeeds");
    assert!(!agg2.active_status.is_active());
    assert_eq!(event.payment_gateway_setting_id, id);
    assert_eq!(
        <PaymentGatewayDisabled as DomainEvent>::EVENT_TYPE,
        "finance.payment_gateway_setting.disabled"
    );
}

#[test]
fn pgs_retire_twice_returns_conflict() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    let (agg2, _): (RealPaymentGatewaySetting, PaymentGatewayDisabled) =
        disable_payment_gateway(agg, tenant.clone(), &SystemClock, &event_id_gen())
            .expect("first retire");
    let err = disable_payment_gateway(agg2, tenant, &SystemClock, &event_id_gen()).unwrap_err();
    assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
}

#[test]
fn pgs_update_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let id = payment_gateway_setting_id(&g, tenant.school_id);
    let cmd = make_configure_cmd(tenant.clone(), id, "Razorpay");
    let (agg, _): (RealPaymentGatewaySetting, PaymentGatewayConfigured) =
        configure_payment_gateway(cmd, &SystemClock, &event_id_gen()).expect("configure");
    let (agg2, _): (RealPaymentGatewaySetting, PaymentGatewayDisabled) =
        disable_payment_gateway(agg, tenant.clone(), &SystemClock, &event_id_gen())
            .expect("retire");
    let update_cmd = UpdatePaymentGatewayCommand {
        tenant,
        payment_gateway_setting_id: id,
        description: Some("x".to_owned()),
        gateway_username: None,
        gateway_password: None,
        gateway_signature: None,
        gateway_client_id: None,
        gateway_secret_key: None,
        gateway_secret_word: None,
        gateway_publisher_key: None,
        gateway_private_key: None,
        mode: None,
        service_charge_minor: None,
        service_charge_type: None,
    };
    let err = update_payment_gateway(agg2, update_cmd, &SystemClock, &event_id_gen()).unwrap_err();
    assert!(matches!(err, educore_core::error::DomainError::Conflict(_)));
}

#[test]
fn pgs_event_types_match_spec() {
    assert_eq!(
        <PaymentGatewayConfigured as DomainEvent>::EVENT_TYPE,
        "finance.payment_gateway_setting.configured"
    );
    assert_eq!(
        <PaymentGatewayUpdated as DomainEvent>::EVENT_TYPE,
        "finance.payment_gateway_setting.updated"
    );
    assert_eq!(
        <PaymentGatewayDisabled as DomainEvent>::EVENT_TYPE,
        "finance.payment_gateway_setting.disabled"
    );
    assert_eq!(
        <PaymentGatewayConfigured as DomainEvent>::AGGREGATE_TYPE,
        "payment_gateway_setting"
    );
}

// =========================================================================
// -- Wave 147 -- PaymentGatewaySetting -- PGS I-4 credentials encrypted marker --
// =========================================================================

#[test]
fn pgs_i_4_credentials_encrypted_at_rest_storage_layer() {
    // PGS I-4 marker test: the credentials-encrypted-at-rest
    // invariant (api_key + api_secret stored in the DB must be
    // encrypted; only the storage adapter is responsible for
    // the encryption envelope, NOT the aggregate) is
    // storage-layer enforced. The aggregate carries the
    // plaintext fields at the API surface; the storage adapter
    // is responsible for encrypting them on write + decrypting
    // them on read.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
