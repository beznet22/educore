//! Integration tests for the **ExamSignature aggregate** vertical slice.
//!
//! Pins the typed-event struct surface for [`ExamSignature`].
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `ExamSignature` has `ExamSignatureCreated`,
//! `ExamSignatureUpdated`, `ExamSignatureDeleted` event
//! structs defined. The events are stubs and the service is
//! not yet implemented. These tests pin the struct field
//! set.
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
    ExamSignatureCreated, ExamSignatureDeleted, ExamSignatureUpdated,
};
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

fn exam_signature_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> educore_assessment::value_objects::ExamSignatureId {
    educore_assessment::value_objects::ExamSignatureId::new(school, g.next_uuid())
}

// =============================================================================
// Typed-event struct surface pins
// =============================================================================

#[test]
fn exam_signature_events_carry_typed_id_and_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let sig_id = exam_signature_id(&g, school);
    let event = ExamSignatureCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_signature_id: sig_id,
    };
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, sig_id);
    assert_eq!(event.exam_signature_id.school_id(), school);
}

#[test]
fn exam_signature_updated_and_deleted_events_carry_typed_id() {
    let (tenant, g) = admin_context();
    let sig_id = exam_signature_id(&g, tenant.school_id);
    let _: ExamSignatureUpdated = ExamSignatureUpdated {
        event_id: g.next_event_id(),
        school_id: tenant.school_id,
        exam_signature_id: sig_id,
    };
    let _: ExamSignatureDeleted = ExamSignatureDeleted {
        event_id: g.next_event_id(),
        school_id: tenant.school_id,
        exam_signature_id: sig_id,
    };
}
