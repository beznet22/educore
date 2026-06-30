//! Integration tests for the **ContentShareList aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`ContentShareList`](educore_cms::aggregate::ContentShareList) end-to-end.
//!
//! # Current contract (Wave 12 vertical slice)
//!
//! `ContentShareList` is a placeholder aggregate carrying the typed id
//! and `school_id`. The full service-layer handler lands in
//! a follow-up phase. These tests pin the typed-id
//! invariants that downstream domains depend on:
//!
//! - `ContentShareListId::new(school, uuid)` round-trips `school_id()`
//! - Two distinct ids in the same school do not collide
//! - A `ContentShareListId` belongs to exactly one school (cross-tenant
//!   confusion is a compile-time error).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_cms::value_objects::ContentShareListId;
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

fn content_share_list_id(g: &SystemIdGen, school: SchoolId) -> ContentShareListId {
    ContentShareListId::new(school, g.next_uuid())
}

#[test]
fn content_share_list_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = content_share_list_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn content_share_list_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = content_share_list_id(&g, school);
    let id_b = content_share_list_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}
