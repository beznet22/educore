//! Integration tests for the **TeacherEvaluation aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`TeacherEvaluation`](educore_assessment::aggregate::TeacherEvaluation)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 29 batch 6)
//!
//! `mark_teacher_evaluation` is now a **real
//! implementation** (Wave 29 batch 6) that enforces the
//! typed-id school-anchoring invariant and emits the
//! spec-defined `TeacherEvaluationCreated` event stamped
//! with a fresh `event_id` (UUIDv7) and the command's
//! school anchor. The full `Rating` / `Comment` / `Status`
//! / `RoleId` / `ParentId` / `AcademicId` payload
//! validation lands in a follow-up batch once the
//! `TenantContext`-bearing command struct is migrated to
//! carry the full payload.
//!
//! `approve_teacher_evaluation` remains a stub
//! (`DomainError::not_supported`) pending its own batch.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (real-handler contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    ApproveTeacherEvaluationCommand, MarkTeacherEvaluationCommand,
};
use educore_assessment::events::TeacherEvaluationCreated;
use educore_assessment::services::{approve_teacher_evaluation, mark_teacher_evaluation};
use educore_assessment::value_objects::TeacherEvaluationId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::{Identifier as _, SchoolId};
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

/// Mint a `TeacherEvaluationId` for the given school.
fn teacher_evaluation_id(g: &SystemIdGen, school: SchoolId) -> TeacherEvaluationId {
    TeacherEvaluationId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: real implementation
// =============================================================================

/// Pins the **happy path** of `mark_teacher_evaluation`
/// for a well-formed payload: a same-school typed id is
/// accepted and the returned event is
/// `TeacherEvaluationCreated` carrying the command's school
/// and the typed id. The full `Rating` / `Comment` /
/// `Status` payload validation lands in a follow-up batch
/// once the `TenantContext`-bearing command struct is
/// migrated to carry the full payload.
#[tokio::test]
async fn teacher_evaluation_create_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _ids = SystemIdGen;

    let id = teacher_evaluation_id(&g, school);
    let cmd = MarkTeacherEvaluationCommand {
        school_id: school,
        teacher_evaluation_id: id,
    };

    let event = mark_teacher_evaluation(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.teacher_evaluation_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `mark_teacher_evaluation`: a typed id from a different
/// school is rejected with `DomainError::Validation`.
#[tokio::test]
async fn teacher_evaluation_create_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = teacher_evaluation_id(&g, other_school);
    let cmd = MarkTeacherEvaluationCommand {
        school_id: actor_school,
        teacher_evaluation_id: foreign_id,
    };

    let err = mark_teacher_evaluation(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// 2. Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `approve_teacher_evaluation`
/// (the spec's "update" branch for a teacher evaluation —
/// records the principal's approval) for a well-formed
/// payload. The handler is currently a stub that returns
/// `DomainError::not_supported("TODO: approve_teacher_evaluation")`
/// before any aggregate mutation or event minting happens.
/// Once the real implementation lands, this test will be
/// updated to assert that the returned event is
/// `TeacherEvaluationApproved` (or the spec-mandated name).
#[tokio::test]
async fn teacher_evaluation_update_currently_returns_not_supported() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    let _ids = SystemIdGen;

    let cmd = ApproveTeacherEvaluationCommand {
        school_id: school,
        teacher_evaluation_id: teacher_evaluation_id(&g, school),
    };

    let err = approve_teacher_evaluation(cmd)
        .await
        .expect_err("approve_teacher_evaluation is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// Keep a reference to the event type so the test compiles
// when the stub is removed and the real handler lands —
// this also documents the eventual return type.
#[allow(dead_code)]
fn _event_type_anchor(_: TeacherEvaluationCreated) {}
