//! Integration tests for the **TeacherEvaluation aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`TeacherEvaluation`](educore_assessment::aggregate::TeacherEvaluation)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `mark_teacher_evaluation` and `approve_teacher_evaluation`
//! in `services.rs` are **stubs**
//! (`DomainError::not_supported("TODO: ...")`). The full
//! implementation lands in a follow-up phase. These tests
//! pin the **current** behaviour so the dispatcher / facade
//! work can rely on the error surface while the real
//! validation + aggregate construction is being built out:
//!
//! 1. Happy path — any well-formed input is rejected with
//!    `DomainError::NotSupported`. No aggregate is built, no
//!    event is emitted.
//! 2. Update path — the approve stub also rejects the
//!    command with the same `NotSupported` error.
//!
//! Once the real handlers land, the happy-path test will be
//! updated to assert `TeacherEvaluationCreated` +
//! `version == 1` per the spec invariant; the update test
//! will then assert the spec-mandated
//! `TeacherEvaluationApproved` (or equivalent) event.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (stub-pattern, lean).

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
use educore_core::ids::SchoolId;
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
// 1. Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `mark_teacher_evaluation`
/// for a well-formed payload. The handler is currently a
/// stub that returns
/// `DomainError::not_supported("TODO: mark_teacher_evaluation")`
/// before any aggregate construction or event minting
/// happens. Once the real implementation lands (carrying
/// `Rating`, `Comment`, `Status`, `RoleId`, `ParentId`,
/// `AcademicId` per
/// `docs/specs/assessment/aggregates.md` § TeacherEvaluation),
/// this test will be updated to assert that:
///
/// - The returned event is `TeacherEvaluationCreated` with
///   `version == 1`,
/// - The aggregate is school-scoped and active,
/// - The rating/comment payload round-trips through the
///   event.
#[tokio::test]
async fn teacher_evaluation_create_currently_returns_not_supported() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    let _ids = SystemIdGen;

    let cmd = MarkTeacherEvaluationCommand {
        school_id: school,
        teacher_evaluation_id: teacher_evaluation_id(&g, school),
    };

    let err = mark_teacher_evaluation(cmd)
        .await
        .expect_err("mark_teacher_evaluation is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
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
