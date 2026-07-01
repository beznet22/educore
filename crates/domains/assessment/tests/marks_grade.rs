//! Integration tests for the **MarksGrade aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for
//! [`MarksGrade`](educore_assessment::aggregate::MarksGrade)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 29 vertical slice)
//!
//! `create_marks_grade`, `update_marks_grade`, and
//! `delete_marks_grade` in `services.rs` are now **real
//! implementations**. They enforce the typed-id
//! school-anchoring invariant (spec invariant — the id's
//! `school_id` must equal the command's `school_id`;
//! cross-tenant references are rejected with
//! `DomainError::Validation`) and emit the spec-defined
//! events (`MarksGradeCreated`, `MarksGradeUpdated`,
//! `MarksGradeDeleted`).
//!
//! The full payload validation (spec invariants #1
//! `From < Up`, #2 `PercentFrom < PercentUpTo`, #3
//! non-overlapping/contiguous scale, #4 exactly one
//! `Gpa = 0.0` per school) lands in a follow-up batch once
//! the `TenantContext`-bearing command struct is migrated
//! to carry `GradeName` / `Gpa` / `From` / `Up` /
//! `PercentFrom` / `PercentUpTo` / `Description`.
//!
//! Mirrors `crates/domains/assessment/tests/exam.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    CreateMarksGradeCommand, DeleteMarksGradeCommand, UpdateMarksGradeCommand,
};
use educore_assessment::services::{create_marks_grade, delete_marks_grade, update_marks_grade};
use educore_assessment::value_objects::MarksGradeId;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
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

/// Mint a `MarksGradeId` for the given school.
fn marks_grade_id(g: &SystemIdGen, school: SchoolId) -> MarksGradeId {
    MarksGradeId::new(school, g.next_uuid())
}

// =============================================================================
// create_marks_grade
// =============================================================================

/// Pins the **happy path** of `create_marks_grade` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `MarksGradeCreated` carrying the
/// command's school and the typed id, and the event id is
/// a freshly-minted UUID (version-7).
#[tokio::test]
async fn marks_grade_create_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = marks_grade_id(&g, school);
    let cmd = CreateMarksGradeCommand {
        school_id: school,
        marks_grade_id: id,
    };

    let event = create_marks_grade(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.marks_grade_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `create_marks_grade`: a `MarksGradeId` anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation` (the typed id's
/// school must equal the command's school).
#[tokio::test]
async fn marks_grade_create_rejects_cross_tenant() {
    let (_tenant_a, g) = admin_context();
    let school_b = g.next_school_id();
    let id = marks_grade_id(&g, school_b);

    let cmd = CreateMarksGradeCommand {
        school_id: g.next_school_id(),
        marks_grade_id: id,
    };

    let err = create_marks_grade(cmd)
        .await
        .expect_err("cross-tenant create must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation (cross-tenant), got {err:?}"
    );
}

// =============================================================================
// update_marks_grade
// =============================================================================

/// Pins the **happy path** of `update_marks_grade` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `MarksGradeUpdated` carrying the
/// command's school and the typed id.
#[tokio::test]
async fn marks_grade_update_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();

    let id = marks_grade_id(&g, school);
    let cmd = UpdateMarksGradeCommand {
        school_id: school,
        marks_grade_id: id,
    };

    let event = update_marks_grade(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.marks_grade_id, id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_marks_grade`: a `MarksGradeId` anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn marks_grade_update_rejects_cross_tenant() {
    let (_tenant_a, g) = admin_context();
    let school_b = g.next_school_id();
    let id = marks_grade_id(&g, school_b);

    let cmd = UpdateMarksGradeCommand {
        school_id: g.next_school_id(),
        marks_grade_id: id,
    };

    let err = update_marks_grade(cmd)
        .await
        .expect_err("cross-tenant update must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation (cross-tenant), got {err:?}"
    );
}

// =============================================================================
// delete_marks_grade
// =============================================================================

/// Pins the **happy path** of `delete_marks_grade` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `MarksGradeDeleted` carrying the
/// command's school and the typed id.
#[tokio::test]
async fn marks_grade_delete_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();

    let id = marks_grade_id(&g, school);
    let cmd = DeleteMarksGradeCommand {
        school_id: school,
        marks_grade_id: id,
    };

    let event = delete_marks_grade(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school);
    assert_eq!(event.marks_grade_id, id);
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `delete_marks_grade`: a `MarksGradeId` anchored to a
/// different school than the command's `school_id` is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn marks_grade_delete_rejects_cross_tenant() {
    let (_tenant_a, g) = admin_context();
    let school_b = g.next_school_id();
    let id = marks_grade_id(&g, school_b);

    let cmd = DeleteMarksGradeCommand {
        school_id: g.next_school_id(),
        marks_grade_id: id,
    };

    let err = delete_marks_grade(cmd)
        .await
        .expect_err("cross-tenant delete must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation (cross-tenant), got {err:?}"
    );
}
