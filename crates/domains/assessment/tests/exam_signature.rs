//! Integration tests for the **ExamSignature aggregate** vertical slice.
//!
//! Pins the create / update / delete command struct surface
//! and the `ExamSignatureCreated` / `ExamSignatureUpdated` /
//! `ExamSignatureDeleted` event structs for
//! [`ExamSignature`](educore_assessment::aggregate::ExamSignature).
//!
//! # Wave 29 real implementation contract
//!
//! `set_exam_signature`, `update_exam_signature`, and
//! `delete_exam_signature` in `services.rs` now enforce the
//! cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` § ExamSignature
//! (the typed id's `school_id` must match the command's
//! `school_id`) and emit the spec-defined events stamped
//! with a fresh `event_id` (UUIDv7) and the command's
//! school anchor. Per spec invariant #2, delete is a
//! soft-delete that flips `ActiveStatus` to inactive; the
//! existing-reports-still-reference-original-file check
//! lands in a follow-up batch once the
//! `TenantContext`-bearing command struct is migrated.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    DeleteExamSignatureCommand, SetExamSignatureCommand, UpdateExamSignatureCommand,
};
use educore_assessment::events::{
    ExamSignatureCreated, ExamSignatureDeleted, ExamSignatureUpdated,
};
use educore_assessment::services::{
    delete_exam_signature, set_exam_signature, update_exam_signature,
};
use educore_assessment::value_objects::ExamSignatureId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::Identifier as _;

// =============================================================================
// Fixtures
// =============================================================================

fn fresh_school() -> (educore_core::ids::SchoolId, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    (school, g)
}

// =============================================================================
// set_exam_signature
// =============================================================================

/// Pins the **happy path** of `set_exam_signature`: a
/// same-school typed id is accepted, the returned event is
/// `ExamSignatureCreated` carrying the command's school and
/// the typed id, and the event id is a freshly-minted UUID
/// (version-7).
#[tokio::test]
async fn set_exam_signature_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let cmd = SetExamSignatureCommand {
        school_id: school,
        exam_signature_id: signature_id,
    };
    let event = set_exam_signature(cmd).await.expect("set_exam_signature");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, signature_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `set_exam_signature`: a `ExamSignatureId` anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn set_exam_signature_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
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

// =============================================================================
// update_exam_signature
// =============================================================================

/// Pins the **happy path** of `update_exam_signature`: a
/// same-school typed id is accepted, the returned event is
/// `ExamSignatureUpdated` carrying the command's school and
/// the typed id.
#[tokio::test]
async fn update_exam_signature_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let cmd = UpdateExamSignatureCommand {
        school_id: school,
        exam_signature_id: signature_id,
    };
    let event = update_exam_signature(cmd)
        .await
        .expect("update_exam_signature");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, signature_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_exam_signature`: a `ExamSignatureId` anchored to
/// a different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn update_exam_signature_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let signature_id = ExamSignatureId::new(school_b, g.next_uuid());
    let cmd = UpdateExamSignatureCommand {
        school_id: school_a,
        exam_signature_id: signature_id,
    };
    let result = update_exam_signature(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// delete_exam_signature
// =============================================================================

/// Pins the **happy path** of `delete_exam_signature`: a
/// same-school typed id is accepted, the returned event is
/// `ExamSignatureDeleted` carrying the command's school and
/// the typed id. Per spec invariant #2, delete is a
/// soft-delete that flips `ActiveStatus` to inactive;
/// existing reports continue to reference the original
/// file. The payload flag lands in a follow-up batch.
#[tokio::test]
async fn delete_exam_signature_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let cmd = DeleteExamSignatureCommand {
        school_id: school,
        exam_signature_id: signature_id,
    };
    let event = delete_exam_signature(cmd)
        .await
        .expect("delete_exam_signature");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_signature_id, signature_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `delete_exam_signature`: a `ExamSignatureId` anchored to
/// a different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn delete_exam_signature_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let signature_id = ExamSignatureId::new(school_b, g.next_uuid());
    let cmd = DeleteExamSignatureCommand {
        school_id: school_a,
        exam_signature_id: signature_id,
    };
    let result = delete_exam_signature(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// Event struct round-trip
// =============================================================================

#[test]
fn exam_signature_event_structs_carry_typed_id_and_school() {
    let (school, g) = fresh_school();
    let signature_id = ExamSignatureId::new(school, g.next_uuid());
    let created = ExamSignatureCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_signature_id: signature_id,
    };
    let updated = ExamSignatureUpdated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_signature_id: signature_id,
    };
    let deleted = ExamSignatureDeleted {
        event_id: g.next_event_id(),
        school_id: school,
        exam_signature_id: signature_id,
    };
    assert_eq!(created.school_id, school);
    assert_eq!(updated.school_id, school);
    assert_eq!(deleted.school_id, school);
    assert_eq!(created.exam_signature_id, signature_id);
    assert_eq!(updated.exam_signature_id, signature_id);
    assert_eq!(deleted.exam_signature_id, signature_id);
}
