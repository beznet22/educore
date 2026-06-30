//! Integration tests for the **MarkStore aggregate** vertical slice.
//!
//! Pins the typed-event struct surface for [`MarkStore`].
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `MarkStore` has `MarkStoreCreated`, `MarkStoreDeleted`
//! event structs defined. The events are stubs and the
//! service is not yet implemented. These tests pin the
//! struct field set.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::events::{MarkStoreCreated, MarkStoreDeleted};
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};

// =============================================================================
// Fixtures
// =============================================================================

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

fn mark_store_id(g: &SystemIdGen, school: SchoolId) -> educore_assessment::value_objects::MarkStoreId {
    educore_assessment::value_objects::MarkStoreId::new(school, g.next_uuid())
}

// =============================================================================
// Typed-event struct surface pins
// =============================================================================

#[test]
fn mark_store_created_event_carries_typed_id_and_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let ms_id = mark_store_id(&g, school);
    let event = MarkStoreCreated {
        event_id: g.next_event_id(),
        school_id: school,
        mark_store_id: ms_id,
    };
    assert_eq!(event.school_id, school);
    assert_eq!(event.mark_store_id, ms_id);
    assert_eq!(event.mark_store_id.school_id(), school);
}

#[test]
fn mark_store_deleted_event_carries_typed_id() {
    let (tenant, g) = admin_context();
    let ms_id = mark_store_id(&g, tenant.school_id);
    let event = MarkStoreDeleted {
        event_id: g.next_event_id(),
        school_id: tenant.school_id,
        mark_store_id: ms_id,
    };
    assert_eq!(event.mark_store_id, ms_id);
}
