//! Integration tests for the **SalaryTemplate aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`SalaryTemplate`](educore_finance::aggregate::SalaryTemplate) end-to-end.

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
use educore_finance::value_objects::SalaryTemplateId;

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

fn salary_template_id(g: &SystemIdGen, school: SchoolId) -> SalaryTemplateId {
    SalaryTemplateId::new(school, g.next_uuid())
}

#[test]
fn salary_template_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = salary_template_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn salary_template_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = salary_template_id(&g, school);
    let id_b = salary_template_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}
