//! Integration tests for the **DirectFeesInstallment aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`DirectFeesInstallment`](educore_finance::aggregate::DirectFeesInstallment) end-to-end.

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
use educore_finance::value_objects::DirectFeesInstallmentId;

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

fn direct_fees_installment_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesInstallmentId {
    DirectFeesInstallmentId::new(school, g.next_uuid())
}

#[test]
fn direct_fees_installment_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn direct_fees_installment_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = direct_fees_installment_id(&g, school);
    let id_b = direct_fees_installment_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}
