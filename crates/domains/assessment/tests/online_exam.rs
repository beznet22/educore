//! Integration tests for the **OnlineExam aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`OnlineExam`](educore_assessment::aggregate::OnlineExam)
//! end-to-end through the service layer.
//!
//! # Wave 29 real implementation contract
//!
//! `create_online_exam` in `services.rs` now enforces the
//! cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` § OnlineExam
//! (the typed id's `school_id` must match the command's
//! `school_id`) and emits the `OnlineExamCreated` event
//! stamped with a fresh `event_id` (UUIDv7) and the
//! command's school anchor. The full
//! `Status`/`IsTaken`/`IsClosed`/`IsWaiting`/`IsRunning`/
//! `AutoMark`/`StartTime`/`EndTime`/`EndDateTime` payload
//! lands in a follow-up batch once the TenantContext-bearing
//! command struct is migrated.
//!
//! `publish_online_exam` is now a **real implementation**
//! (Wave 29 batch 6) that enforces the typed-id
//! school-anchoring invariant and emits the
//! `OnlineExamCreated` event stamped with a fresh
//! `event_id` (UUIDv7) and the command's school anchor.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::CreateOnlineExamCommand;
use educore_assessment::services::{create_online_exam, publish_online_exam};
use educore_assessment::value_objects::OnlineExamId;
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::Identifier as _;
use educore_core::tenant::{TenantContext, UserType};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
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

/// Mint an `OnlineExamId` for the given school.
fn online_exam_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> OnlineExamId {
    OnlineExamId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: create_online_exam returns OnlineExamCreated
// =============================================================================

#[tokio::test]
async fn create_online_exam_emits_event_for_anchored_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = CreateOnlineExamCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let event = create_online_exam(cmd).await.expect("create_online_exam");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.online_exam_id.school_id(), school);
}

#[tokio::test]
async fn create_online_exam_rejects_cross_tenant_id() {
    let (tenant, g) = admin_context();
    let school_a = tenant.school_id;
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let cmd = CreateOnlineExamCommand {
        school_id: school_a,
        online_exam_id: online_exam_id(&g, school_b),
    };
    let result = create_online_exam(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// Update path: real implementation
// =============================================================================

/// Pins the **happy path** of `publish_online_exam` (the
/// natural "update" / state-transition handler on the
/// `OnlineExam` aggregate). A same-school typed id is
/// accepted and the returned event is `OnlineExamCreated`
/// carrying the command's school and the typed id.
#[tokio::test]
async fn online_exam_publish_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = online_exam_id(&g, school);
    let cmd = educore_assessment::commands::PublishOnlineExamCommand {
        school_id: school,
        online_exam_id: id,
    };

    let event = publish_online_exam(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.online_exam_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `publish_online_exam`: a typed id from a different
/// school is rejected with `DomainError::Validation`.
#[tokio::test]
async fn online_exam_publish_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = online_exam_id(&g, other_school);
    let cmd = educore_assessment::commands::PublishOnlineExamCommand {
        school_id: actor_school,
        online_exam_id: foreign_id,
    };

    let err = publish_online_exam(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
