//! Integration tests for the **ExamSignature aggregate** vertical slice.
//!
//! Pins the create command struct surface and the
//! `ExamSignatureCreated` event struct for
//! [`ExamSignature`](educore_assessment::aggregate::ExamSignature).
//!
//! # Wave 29 real implementation contract
//!
//! `set_exam_signature` in `services.rs` now enforces the
//! cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` § ExamSignature
//! (the typed id's `school_id` must match the command's
//! `school_id`) and emits the `ExamSignatureCreated` event
//! stamped with a fresh `event_id` (UUIDv7) and the
//! command's school anchor.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::SetExamSignatureCommand;
use educore_assessment::events::ExamSignatureCreated;
use educore_assessment::services::set_exam_signature;
use educore_assessment::value_objects::ExamSignatureId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;

// =============================================================================
// Fixtures
// =============================================================================

fn fresh_school() -> (educore_core::ids::SchoolId, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    (school, g)
}

// =============================================================================
// Happy path: set_exam_signature returns ExamSignatureCreated
// =============================================================================

#[tokio::test]
async fn set_exam_signature_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let cmd = SetExamSignatureCommand {
        school_id: school,
        exam_signature_id: signature_id,
    };
    let event = set_exam_signature(cmd).await.expect("set_exam_signature");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, signature_id);
}

#[tokio::test]
async fn set_exam_signature_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let signature_id = ExamSignatureId::new(school_b, g.next_uuid());
    let cmd = SetExamSignatureCommand {
        school_id: school_a,
        exam_signature_id: signature_id,
    };
    let result = set_exam_signature(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

#[test]
fn exam_signature_created_event_carries_typed_id_and_school() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let event = ExamSignatureCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_signature_id: signature_id,
    };
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, signature_id);
}
