//! Integration tests for the **TeacherRemark aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`TeacherRemark`](educore_assessment::aggregate::TeacherRemark)
//! end-to-end through the service layer.
//!
//! # Wave 29 batch 7 contract
//!
//! `add_teacher_remark` and `update_teacher_remark` in
//! `services.rs` are now **real implementations**. They
//! enforce the typed-id school-anchoring invariant from
//! `docs/specs/assessment/aggregates.md` § TeacherRemark
//! (the id's `school_id` must equal the command's
//! `school_id`; cross-tenant references are rejected with
//! `DomainError::Validation`) and emit the spec-defined
//! `TeacherRemarkCreated` event for every transition. The
//! full Remark/TeacherId/StudentId/ExamTypeId/AcademicId
//! payload validation (and the uniqueness + 2000-char
//! invariants #1 and #2) lands in a follow-up batch once
//! the `TenantContext`-bearing command struct is migrated.

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
// 2. Update path: real implementation emits TeacherRemarkCreated
// =============================================================================

/// Pins the **happy path** of `update_teacher_remark` for
/// a well-formed payload: a same-school typed id is
/// accepted and the returned event is `TeacherRemarkCreated`
/// carrying the command's school and the typed id (the
/// update-transition event re-emits the same shape until
/// the full `Remark` payload is migrated onto the command
/// struct).
#[tokio::test]
async fn update_teacher_remark_happy_path() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;

    let cmd = UpdateTeacherRemarkCommand {
        school_id: school,
        teacher_remark_id: teacher_remark_id(&g, school),
    };

    let event = update_teacher_remark(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(
        event.teacher_remark_id.school_id(),
        school,
        "event id echoes command"
    );
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_teacher_remark`: a typed id from a different
/// school is rejected with `DomainError::Validation`.
#[tokio::test]
async fn update_teacher_remark_cross_tenant_rejected() {
    let (_tenant, g) = admin_context();
    let actor_school = _tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = teacher_remark_id(&g, other_school);
    let cmd = UpdateTeacherRemarkCommand {
        school_id: actor_school,
        teacher_remark_id: foreign_id,
    };

    let err = update_teacher_remark(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// Keep a reference to the event type so the test compiles
// when the stub is removed and the real handler lands —
// this also documents the eventual return type.
#[allow(dead_code)]
fn _event_type_anchor(_: TeacherRemarkCreated) {}
