//! Integration tests for the **ExamSetup aggregate** vertical slice.
//!
//! Pins the typed-event struct surface for
//! [`ExamSetup`](educore_assessment::aggregate::ExamSetup).
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `ExamSetup` has `ExamSetupCreated` and `ExamSetupUpdated`
//! event structs defined. The events are stubs (no
//! `DomainEvent` trait impl yet) and the service is not yet
//! implemented. These tests pin the typed-event struct
//! field set so the dispatcher / facade work can rely on it.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (stub pattern).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::events::ExamSetupCreated;
use educore_assessment::value_objects::ExamSetupId;
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

fn exam_setup_id(g: &SystemIdGen, school: SchoolId) -> ExamSetupId {
    ExamSetupId::new(school, g.next_uuid())
}

// =============================================================================
// Typed-event struct surface pins
// =============================================================================

#[test]
fn exam_setup_created_event_carries_typed_id_and_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let setup_id = exam_setup_id(&g, school);
    let event = ExamSetupCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_setup_id: setup_id,
    };
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_setup_id, setup_id);
    assert_eq!(event.exam_setup_id.school_id(), school);
}

#[test]
fn exam_setup_created_event_round_trips_across_tenants() {
    let (tenant_a, g_a) = admin_context();
    let (tenant_b, g_b) = admin_context();
    let id_a = exam_setup_id(&g_a, tenant_a.school_id);
    let id_b = exam_setup_id(&g_b, tenant_b.school_id);
    assert_ne!(id_a.school_id(), id_b.school_id());
    let event_a = ExamSetupCreated {
        event_id: g_a.next_event_id(),
        school_id: tenant_a.school_id,
        exam_setup_id: id_a,
    };
    let event_b = ExamSetupCreated {
        event_id: g_b.next_event_id(),
        school_id: tenant_b.school_id,
        exam_setup_id: id_b,
    };
    assert_ne!(event_a.school_id, event_b.school_id);
    assert_ne!(event_a.exam_setup_id, event_b.exam_setup_id);
}
