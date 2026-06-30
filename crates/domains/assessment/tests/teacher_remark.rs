//! Integration tests for the **TeacherRemark aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`TeacherRemark`](educore_assessment::aggregate::TeacherRemark)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `add_teacher_remark` and `update_teacher_remark` in
//! `services.rs` are **stubs**
//! (`DomainError::not_supported("TODO: ...")`). The full
//! implementation lands in a follow-up phase. These tests
//! pin the **current** behaviour so the dispatcher / facade
//! work can rely on the error surface while the real
//! validation + aggregate construction is being built out:
//!
//! 1. Happy path — any well-formed input is rejected with
//!    `DomainError::NotSupported`. No aggregate is built, no
//!    event is emitted.
//! 2. Update path — the update stub also rejects the
//!    command with the same `NotSupported` error.
//!
//! Once the real handlers land, the happy-path test will be
//! updated to assert `TeacherRemarkCreated` + `version == 1`
//! per the spec invariant; the update test will then assert
//! the spec-mandated `TeacherRemarkUpdated` event.
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
// 1. Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `add_teacher_remark` for
/// a well-formed payload. The handler is currently a stub
/// that returns
/// `DomainError::not_supported("TODO: add_teacher_remark")`
/// before any aggregate construction or event minting
/// happens. Once the real implementation lands (carrying
/// `Remark`, `TeacherId`, `StudentId`, `ExamTypeId`,
/// `AcademicId` per
/// `docs/specs/assessment/aggregates.md` § TeacherRemark),
/// this test will be updated to assert that:
///
/// - The returned event is `TeacherRemarkCreated` with
///   `version == 1`,
/// - The aggregate is school-scoped and active,
/// - The remark payload round-trips through the event.
#[tokio::test]
async fn teacher_remark_create_currently_returns_not_supported() {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    let _ids = SystemIdGen;

    let cmd = AddTeacherRemarkCommand {
        school_id: school,
        teacher_remark_id: teacher_remark_id(&g, school),
    };

    let err = add_teacher_remark(cmd)
        .await
        .expect_err("add_teacher_remark is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
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
    let _ids = SystemIdGen;

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
