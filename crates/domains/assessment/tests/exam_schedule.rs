//! Integration tests for the **ExamSchedule aggregate** vertical slice.
//!
//! Pins the typed-event struct surface for [`ExamSchedule`].
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `ExamSchedule` has `ExamScheduled`, `ExamScheduleUpdated`,
//! and `ExamScheduleCancelled` event structs defined. The
//! events are stubs and the service is not yet implemented.
//! These tests pin the struct field set.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::events::{
    ExamScheduleCancelled, ExamScheduleUpdated,
};
use educore_assessment::value_objects::ExamScheduleId;
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

fn schedule_id(g: &SystemIdGen, school: SchoolId) -> ExamScheduleId {
    ExamScheduleId::new(school, g.next_uuid())
}

// =============================================================================
// Typed-event struct surface pins
// =============================================================================

#[test]
fn exam_schedule_updated_event_carries_changes_and_correlation() {
    let (tenant, g) = admin_context();
    let schedule_id = schedule_id(&g, tenant.school_id);
    let event = ExamScheduleUpdated {
        schedule_id,
        changes: vec!["date".to_owned(), "start_time".to_owned()],
        event_id: g.next_event_id(),
        correlation_id: tenant.correlation_id,
        occurred_at: educore_core::value_objects::Timestamp::now(),
    };
    assert_eq!(event.schedule_id.school_id(), tenant.school_id);
    assert_eq!(event.changes.len(), 2);
}

#[test]
fn exam_schedule_cancelled_event_carries_reason() {
    let (tenant, g) = admin_context();
    let schedule_id = schedule_id(&g, tenant.school_id);
    let event = ExamScheduleCancelled {
        schedule_id,
        reason: "school closed".to_owned(),
        event_id: g.next_event_id(),
        correlation_id: tenant.correlation_id,
        occurred_at: educore_core::value_objects::Timestamp::now(),
    };
    assert_eq!(event.schedule_id.school_id(), tenant.school_id);
    assert_eq!(event.reason, "school closed");
}

#[test]
fn exam_schedule_typed_id_isolates_tenants() {
    let (tenant_a, g_a) = admin_context();
    let (tenant_b, g_b) = admin_context();
    let id_a = schedule_id(&g_a, tenant_a.school_id);
    let id_b = schedule_id(&g_b, tenant_b.school_id);
    assert_ne!(id_a.school_id(), id_b.school_id());
    assert_ne!(id_a, id_b);
}
