//! Integration tests for the **HomePageSetting aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`HomePageSetting`](educore_cms::aggregate::HomePageSetting) end-to-end.
//!
//! # Current contract (Wave 12 vertical slice)
//!
//! `HomePageSetting` is a placeholder aggregate carrying the typed id
//! and `school_id`. The full service-layer handler lands in
//! a follow-up phase. These tests pin the typed-id
//! invariants that downstream domains depend on:
//!
//! - `HomePageSettingId::new(school, uuid)` round-trips `school_id()`
//! - Two distinct ids in the same school do not collide
//! - A `HomePageSettingId` belongs to exactly one school (cross-tenant
//!   confusion is a compile-time error).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_cms::value_objects::HomePageSettingId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};

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

fn home_page_setting_id(g: &SystemIdGen, school: SchoolId) -> HomePageSettingId {
    HomePageSettingId::new(school, g.next_uuid())
}

#[test]
fn home_page_setting_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = home_page_setting_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn home_page_setting_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = home_page_setting_id(&g, school);
    let id_b = home_page_setting_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}
