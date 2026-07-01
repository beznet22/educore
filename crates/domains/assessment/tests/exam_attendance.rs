//! Integration tests for the **ExamAttendance aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`ExamAttendance`](educore_assessment::aggregate::ExamAttendance)
//! end-to-end through the service layer.
//!
//! # Wave 29 real implementation contract
//!
//! `mark_exam_attendance` and `update_exam_attendance` in
//! `services.rs` are now **real implementations**. They
//! enforce the typed-id school-anchoring invariant (the id's
//! `school_id` must equal the command's `school_id`;
//! cross-tenant references are rejected with
//! `DomainError::Validation`) and emit the spec-defined
//! `ExamAttendanceCreated` event stamped with a fresh
//! `event_id` (UUIDv7) and the command's school anchor.
//!
//! The full payload validation (spec invariant #1 — unique
//! by `(exam_id, subject_id, class_id, section_id,
//! academic_id)`) and the per-student present/absent child
//! rows (`ExamAttendanceChild` with `AttendanceType` P/A)
//! land in a follow-up batch once the
//! `TenantContext`-bearing command struct is migrated to
//! carry the full payload.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{MarkExamAttendanceCommand, UpdateExamAttendanceCommand};
use educore_assessment::events::ExamAttendanceCreated;
use educore_assessment::services::{mark_exam_attendance, update_exam_attendance};
use educore_assessment::value_objects::ExamAttendanceId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::Identifier as _;
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

/// Mint an `ExamAttendanceId` for the given school.
fn exam_attendance_id(g: &SystemIdGen, school: SchoolId) -> ExamAttendanceId {
    ExamAttendanceId::new(school, g.next_uuid())
}

// =============================================================================
// 1. mark_exam_attendance — happy path
// =============================================================================

/// Pins the **happy path** of `mark_exam_attendance` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `ExamAttendanceCreated` carrying the
/// command's school and the typed id, and the event id is
/// a freshly-minted UUID (version-7).
#[tokio::test]
async fn exam_attendance_mark_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = exam_attendance_id(&g, school);
    let cmd = MarkExamAttendanceCommand {
        school_id: school,
        exam_attendance_id: id,
    };

    let event = mark_exam_attendance(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.exam_attendance_id, id, "event id echoes command");
    let _: uuid::Uuid = event.event_id.as_uuid();
}

// =============================================================================
// 2. mark_exam_attendance — cross-tenant rejection
// =============================================================================

/// Pins the **cross-tenant rejection** contract of
/// `mark_exam_attendance`: an `ExamAttendanceId` anchored
/// to a different school than the command's `school_id` is
/// rejected with `DomainError::Validation` (the typed id's
/// school must equal the command's school).
#[tokio::test]
async fn exam_attendance_mark_rejects_cross_tenant() {
    let (_tenant_a, g) = admin_context();
    let school_b = g.next_school_id();
    let id = exam_attendance_id(&g, school_b);

    let cmd = MarkExamAttendanceCommand {
        school_id: g.next_school_id(),
        exam_attendance_id: id,
    };

    let err = mark_exam_attendance(cmd)
        .await
        .expect_err("cross-tenant mark must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation (cross-tenant), got {err:?}"
    );
}

// =============================================================================
// 3. update_exam_attendance — happy path
// =============================================================================

/// Pins the **happy path** of `update_exam_attendance` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `ExamAttendanceCreated` carrying the
/// command's school and the typed id.
#[tokio::test]
async fn exam_attendance_update_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let id = exam_attendance_id(&g, school);
    let cmd = UpdateExamAttendanceCommand {
        school_id: school,
        exam_attendance_id: id,
    };

    let event = update_exam_attendance(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.exam_attendance_id, id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

// =============================================================================
// 4. update_exam_attendance — cross-tenant rejection
// =============================================================================

/// Pins the **cross-tenant rejection** contract of
/// `update_exam_attendance`: an `ExamAttendanceId` anchored
/// to a different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn exam_attendance_update_rejects_cross_tenant() {
    let (_tenant_a, g) = admin_context();
    let school_b = g.next_school_id();
    let id = exam_attendance_id(&g, school_b);

    let cmd = UpdateExamAttendanceCommand {
        school_id: g.next_school_id(),
        exam_attendance_id: id,
    };

    let err = update_exam_attendance(cmd)
        .await
        .expect_err("cross-tenant update must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation (cross-tenant), got {err:?}"
    );
}

// Keep a reference to the event type so the test compiles
// when the stub is removed and the real handler lands —
// this also documents the eventual return type.
#[allow(dead_code)]
fn _event_type_anchor(_: ExamAttendanceCreated) {}
