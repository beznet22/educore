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
//! `publish_online_exam` remains a stub
//! (`DomainError::not_supported`) pending its own batch.

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
// Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `publish_online_exam` —
/// the natural "update" / state-transition handler on the
/// `OnlineExam` aggregate. The handler is currently a stub
/// that returns `DomainError::NotSupported("TODO:
/// publish_online_exam")` before any state transition or
/// event minting happens. Once the real implementation
/// lands, this test will be updated to assert that:
///
/// - The returned event is `OnlineExamCreated` (re-emitted
///   with the publish transition; see events.rs contract),
/// - The aggregate's `IsTaken` flag flips to `true` and
///   `version` increments.
#[tokio::test]
async fn online_exam_publish_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = educore_assessment::commands::PublishOnlineExamCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let err = publish_online_exam(cmd)
        .await
        .expect_err("publish_online_exam is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}
