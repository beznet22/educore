//! Integration tests for the **ExamSetting aggregate** vertical slice.
//!
//! Pins the create + update + delete command struct surface
//! and the `ExamSettingCreated` event struct for
//! [`ExamSetting`](educore_assessment::aggregate::ExamSetting).
//!
//! # Wave 29 real implementation contract
//!
//! `create_exam_setting`, `update_exam_setting`, and
//! `delete_exam_setting` in `services.rs` now enforce the
//! cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` § ExamSetting
//! (the typed id's `school_id` must match the command's
//! `school_id`) and emit the `ExamSettingCreated` event
//! stamped with a fresh `event_id` (UUIDv7) and the
//! command's school anchor.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    CreateExamSettingCommand, DeleteExamSettingCommand, UpdateExamSettingCommand,
};
use educore_assessment::events::ExamSettingCreated;
use educore_assessment::services::{create_exam_setting, delete_exam_setting, update_exam_setting};
use educore_assessment::value_objects::ExamSettingId;
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
// Happy path: create_exam_setting returns ExamSettingCreated
// =============================================================================

#[tokio::test]
async fn create_exam_setting_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let setting_id = ExamSettingId::new(school, g.next_uuid());
    let cmd = CreateExamSettingCommand {
        school_id: school,
        exam_setting_id: setting_id,
    };
    let event = create_exam_setting(cmd).await.expect("create_exam_setting");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_setting_id, setting_id);
}

#[tokio::test]
async fn create_exam_setting_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let setting_id = ExamSettingId::new(school_b, g.next_uuid());
    let cmd = CreateExamSettingCommand {
        school_id: school_a,
        exam_setting_id: setting_id,
    };
    let result = create_exam_setting(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

#[test]
fn exam_setting_created_event_carries_typed_id_and_school() {
    let (school, g) = fresh_school();
    let setting_id = ExamSettingId::new(school, g.next_uuid());
    let event = ExamSettingCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_setting_id: setting_id,
    };
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_setting_id, setting_id);
}

// =============================================================================
// update_exam_setting: Wave 29 batch 2 real implementation
// =============================================================================

#[tokio::test]
async fn update_exam_setting_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let setting_id = ExamSettingId::new(school, g.next_uuid());
    let cmd = UpdateExamSettingCommand {
        school_id: school,
        exam_setting_id: setting_id,
    };
    let event = update_exam_setting(cmd).await.expect("update_exam_setting");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_setting_id, setting_id);
}

#[tokio::test]
async fn update_exam_setting_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let setting_id = ExamSettingId::new(school_b, g.next_uuid());
    let cmd = UpdateExamSettingCommand {
        school_id: school_a,
        exam_setting_id: setting_id,
    };
    let result = update_exam_setting(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// delete_exam_setting: Wave 29 batch 2 real implementation
// =============================================================================

#[tokio::test]
async fn delete_exam_setting_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let setting_id = ExamSettingId::new(school, g.next_uuid());
    let cmd = DeleteExamSettingCommand {
        school_id: school,
        exam_setting_id: setting_id,
    };
    let event = delete_exam_setting(cmd).await.expect("delete_exam_setting");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_setting_id, setting_id);
}

#[tokio::test]
async fn delete_exam_setting_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let setting_id = ExamSettingId::new(school_b, g.next_uuid());
    let cmd = DeleteExamSettingCommand {
        school_id: school_a,
        exam_setting_id: setting_id,
    };
    let result = delete_exam_setting(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}
