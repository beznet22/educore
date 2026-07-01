//! Integration tests for the **TeacherRemark aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`TeacherRemark`](educore_assessment::aggregate::TeacherRemark)
//! end-to-end through the service layer.
//!
//! # Wave 29 real implementation contract
//!
//! `add_teacher_remark` in `services.rs` now enforces the
//! cross-tenant invariant from
//! `docs/specs/assessment/aggregates.md` § TeacherRemark
//! (the typed id's `school_id` must match the command's
//! `school_id`) and emits the `TeacherRemarkCreated` event
//! stamped with a fresh `event_id` (UUIDv7) and the
//! command's school anchor.
//!
//! `update_teacher_remark` remains a stub pending the full
//! Remark/TeacherId/StudentId/ExamTypeId/AcademicId payload
//! migration to TenantContext-bearing commands.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{AddTeacherRemarkCommand, UpdateTeacherRemarkCommand};
use educore_assessment::events::TeacherRemarkCreated;
use educore_assessment::services::{add_teacher_remark, update_teacher_remark};
use educore_assessment::value_objects::TeacherRemarkId;
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

/// Mint a `TeacherRemarkId` for the given school.
fn teacher_remark_id(g: &SystemIdGen, school: SchoolId) -> TeacherRemarkId {
    TeacherRemarkId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: add_teacher_remark returns TeacherRemarkCreated
// =============================================================================

/// `add_teacher_remark` now emits a `TeacherRemarkCreated`
/// event with a fresh UUIDv7 `event_id` and the command's
/// school anchor when the typed id is school-anchored to
/// the same school. The full Remark/TeacherId/StudentId/
/// ExamTypeId/AcademicId payload lands in a follow-up batch
/// once the TenantContext-bearing command struct is migrated.
#[tokio::test]
async fn add_teacher_remark_emits_event_for_anchored_id() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;

    let cmd = AddTeacherRemarkCommand {
        school_id: school,
        teacher_remark_id: teacher_remark_id(&g, school),
    };

    let event = add_teacher_remark(cmd)
        .await
        .expect("add_teacher_remark is real");
    // Spec invariant: event carries the command's school + typed id.
    assert_eq!(event.school_id, school);
    assert_eq!(event.teacher_remark_id.school_id(), school);
}

#[tokio::test]
async fn add_teacher_remark_rejects_cross_tenant_id() {
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();
    // Typed id anchored to school_b; command claims school_a.
    let cmd = AddTeacherRemarkCommand {
        school_id: school_a,
        teacher_remark_id: TeacherRemarkId::new(school_b, g.next_uuid()),
    };

    let result = add_teacher_remark(cmd).await;
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "expected Validation error for cross-tenant id, got {:?}",
        result.map(|_| "ok")
    );
}

// =============================================================================
// 2. Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `update_teacher_remark`
/// for a well-formed payload. The handler is currently a
/// stub that returns
/// `DomainError::not_supported("TODO: update_teacher_remark")`
/// before any aggregate mutation or event minting happens.
/// Once the real implementation lands, this test will be
/// updated to assert that the returned event is
/// `TeacherRemarkUpdated` (or the spec-mandated name).
#[tokio::test]
async fn teacher_remark_update_currently_returns_not_supported() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;

    let cmd = UpdateTeacherRemarkCommand {
        school_id: school,
        teacher_remark_id: teacher_remark_id(&g, school),
    };

    let err = update_teacher_remark(cmd)
        .await
        .expect_err("update_teacher_remark is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// Keep a reference to the event type so the test compiles
// when the stub is removed and the real handler lands —
// this also documents the eventual return type.
#[allow(dead_code)]
fn _event_type_anchor(_: TeacherRemarkCreated) {}
