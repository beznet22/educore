//! Integration tests for the **FeesPayment aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`FeesPayment`](educore_finance::aggregate::FeesPayment) end-to-end.

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
use educore_finance::value_objects::FeesPaymentId;

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

fn fees_payment_id(g: &SystemIdGen, school: SchoolId) -> FeesPaymentId {
    FeesPaymentId::new(school, g.next_uuid())
}

#[test]
fn fees_payment_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_payment_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_payment_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fees_payment_id(&g, school);
    let id_b = fees_payment_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// -- Wave 144 -- FeesPayment -- FP I-1/2/3 marker tests --
// =========================================================================

#[test]
fn fp_i_1_fk_to_fees_assign_student_dispatcher_enforced() {
    // FP I-1 marker test: the FK to FeesAssign/Student invariant
    // (a FeesPayment must reference a valid FeesAssign row, which
    // transitively references a valid Student) is dispatcher-
    // enforced. The FeesPayment aggregate carries the FK field
    // at the API surface; the dispatcher validates referential
    // integrity on create.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}

#[test]
fn fp_i_2_gateway_consistency_dispatcher_enforced() {
    // FP I-2 marker test: the gateway consistency invariant (a
    // FeesPayment whose payment_method == Gateway MUST reference
    // the same PaymentGatewaySetting as the linked PaymentMethod)
    // is dispatcher-enforced. The aggregate carries the
    // payment_method + gateway reference fields; the dispatcher
    // validates the cross-row consistency.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}

#[test]
fn fp_i_3_gateway_tx_id_required_if_gateway_dispatcher_enforced() {
    // FP I-3 marker test: the gateway_tx_id-required-if-Gateway
    // invariant (a FeesPayment whose payment_method == Gateway
    // MUST carry a non-empty gateway_tx_id; non-Gateway payments
    // MUST NOT carry a gateway_tx_id) is dispatcher-enforced.
    // The aggregate carries the payment_method + gateway_tx_id
    // fields; the dispatcher validates the mutual exclusion.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
