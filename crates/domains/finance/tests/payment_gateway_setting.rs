//! Integration tests for the **PaymentGatewaySetting aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`PaymentGatewaySetting`](educore_finance::aggregate::PaymentGatewaySetting) end-to-end.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::value_objects::PaymentGatewaySettingId;

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

#[test]
fn payment_gateway_setting_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = payment_gateway_setting_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn payment_gateway_setting_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = payment_gateway_setting_id(&g, school);
    let id_b = payment_gateway_setting_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}
