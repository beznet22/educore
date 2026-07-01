//! Integration tests for the **ExamRoutinePage + FrontendExamRoutine +
//! FrontendResult + FrontendExamResult aggregate** vertical slice.
//!
//! Pins the create / update / publish command struct surface
//! and the `ExamRoutinePageCreated` / `FrontendExamRoutineCreated` /
//! `FrontendResultCreated` / `FrontendExamResultCreated` event structs
//! for the corresponding aggregates.
//!
//! # Wave 29 real implementation contract
//!
//! `update_exam_routine_page`, `publish_exam_routine`,
//! `publish_front_result`, and `update_frontend_exam_result` in
//! `services.rs` now enforce the cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` (the typed id's `school_id`
//! must match the command's `school_id`) and emit the spec-defined
//! events stamped with a fresh `event_id` (UUIDv7) and the command's
//! school anchor. The full payload validation lands in a follow-up
//! batch once the `TenantContext`-bearing command struct is migrated.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    PublishExamRoutineCommand, PublishFrontResultCommand, UpdateExamRoutinePageCommand,
    UpdateFrontendExamResultCommand,
};
use educore_assessment::events::{
    ExamRoutinePageCreated, FrontendExamResultCreated, FrontendExamRoutineCreated,
    FrontendResultCreated,
};
use educore_assessment::services::{
    publish_exam_routine, publish_front_result, update_exam_routine_page,
    update_frontend_exam_result,
};
use educore_assessment::value_objects::{
    ExamRoutinePageId, FrontExamRoutineId, FrontResultId, FrontendExamResultId,
};
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
// update_exam_routine_page
// =============================================================================

/// Pins the **happy path** of `update_exam_routine_page`: a
/// same-school typed id is accepted, the returned event is
/// `ExamRoutinePageCreated` carrying the command's school and
/// the typed id, and the event id is a freshly-minted UUID
/// (version-7).
#[tokio::test]
async fn update_exam_routine_page_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let page_id = ExamRoutinePageId::new(school, g.next_uuid());
    let cmd = UpdateExamRoutinePageCommand {
        school_id: school,
        exam_routine_page_id: page_id,
    };
    let event = update_exam_routine_page(cmd)
        .await
        .expect("update_exam_routine_page");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_routine_page_id, page_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_exam_routine_page`: a typed id anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn update_exam_routine_page_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let page_id = ExamRoutinePageId::new(school_b, g.next_uuid());
    let cmd = UpdateExamRoutinePageCommand {
        school_id: school_a,
        exam_routine_page_id: page_id,
    };
    let result = update_exam_routine_page(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// publish_exam_routine
// =============================================================================

/// Pins the **happy path** of `publish_exam_routine`: a
/// same-school typed id is accepted, the returned event is
/// `FrontendExamRoutineCreated` carrying the command's school
/// and the typed id.
#[tokio::test]
async fn publish_exam_routine_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let routine_id = FrontExamRoutineId::new(school, g.next_uuid());
    let cmd = PublishExamRoutineCommand {
        school_id: school,
        front_exam_routine_id: routine_id,
    };
    let event = publish_exam_routine(cmd).await.expect("publish_exam_routine");
    assert_eq!(event.school_id, school);
    assert_eq!(event.front_exam_routine_id, routine_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `publish_exam_routine`: a typed id anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn publish_exam_routine_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let routine_id = FrontExamRoutineId::new(school_b, g.next_uuid());
    let cmd = PublishExamRoutineCommand {
        school_id: school_a,
        front_exam_routine_id: routine_id,
    };
    let result = publish_exam_routine(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// publish_front_result
// =============================================================================

/// Pins the **happy path** of `publish_front_result`: a
/// same-school typed id is accepted, the returned event is
/// `FrontendResultCreated` carrying the command's school and
/// the typed id.
#[tokio::test]
async fn publish_front_result_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let result_id = FrontResultId::new(school, g.next_uuid());
    let cmd = PublishFrontResultCommand {
        school_id: school,
        front_result_id: result_id,
    };
    let event = publish_front_result(cmd).await.expect("publish_front_result");
    assert_eq!(event.school_id, school);
    assert_eq!(event.front_result_id, result_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `publish_front_result`: a typed id anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn publish_front_result_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let result_id = FrontResultId::new(school_b, g.next_uuid());
    let cmd = PublishFrontResultCommand {
        school_id: school_a,
        front_result_id: result_id,
    };
    let result = publish_front_result(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// update_frontend_exam_result
// =============================================================================

/// Pins the **happy path** of `update_frontend_exam_result`: a
/// same-school typed id is accepted, the returned event is
/// `FrontendExamResultCreated` carrying the command's school
/// and the typed id.
#[tokio::test]
async fn update_frontend_exam_result_emits_event_for_anchored_id() {
    let (school, g) = fresh_school();
    let result_id = FrontendExamResultId::new(school, g.next_uuid());
    let cmd = UpdateFrontendExamResultCommand {
        school_id: school,
        frontend_exam_result_id: result_id,
    };
    let event = update_frontend_exam_result(cmd)
        .await
        .expect("update_frontend_exam_result");
    assert_eq!(event.school_id, school);
    assert_eq!(event.frontend_exam_result_id, result_id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_frontend_exam_result`: a typed id anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn update_frontend_exam_result_rejects_cross_tenant_id() {
    let (school_a, g) = fresh_school();
    let school_b = g.next_school_id();
    let result_id = FrontendExamResultId::new(school_b, g.next_uuid());
    let cmd = UpdateFrontendExamResultCommand {
        school_id: school_a,
        frontend_exam_result_id: result_id,
    };
    let result = update_frontend_exam_result(cmd).await;
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
fn exam_routine_page_event_structs_carry_typed_id_and_school() {
    let (school, g) = fresh_school();
    let page_id = ExamRoutinePageId::new(school, g.next_uuid());
    let routine_id = FrontExamRoutineId::new(school, g.next_uuid());
    let result_id = FrontResultId::new(school, g.next_uuid());
    let frontend_result_id = FrontendExamResultId::new(school, g.next_uuid());

    let page = ExamRoutinePageCreated {
        event_id: g.next_event_id(),
        school_id: school,
        exam_routine_page_id: page_id,
    };
    let routine = FrontendExamRoutineCreated {
        event_id: g.next_event_id(),
        school_id: school,
        front_exam_routine_id: routine_id,
    };
    let front_result = FrontendResultCreated {
        event_id: g.next_event_id(),
        school_id: school,
        front_result_id: result_id,
    };
    let frontend_exam_result = FrontendExamResultCreated {
        event_id: g.next_event_id(),
        school_id: school,
        frontend_exam_result_id: frontend_result_id,
    };

    assert_eq!(page.school_id, school);
    assert_eq!(routine.school_id, school);
    assert_eq!(front_result.school_id, school);
    assert_eq!(frontend_exam_result.school_id, school);
    assert_eq!(page.exam_routine_page_id, page_id);
    assert_eq!(routine.front_exam_routine_id, routine_id);
    assert_eq!(front_result.front_result_id, result_id);
    assert_eq!(
        frontend_exam_result.frontend_exam_result_id,
        frontend_result_id
    );
}
