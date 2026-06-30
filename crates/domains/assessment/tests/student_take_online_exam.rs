//! Integration tests for the **StudentTakeOnlineExam aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`StudentTakeOnlineExam`](educore_assessment::aggregate::StudentTakeOnlineExam)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `start_online_exam` and `submit_online_exam_answer` in
//! `services.rs` are **stubs** that unconditionally return
//! `DomainError::not_supported("TODO: ...")` before any
//! aggregate construction or event minting happens. The full
//! implementation lands in a follow-up phase. These tests
//! pin the **current** behaviour so the dispatcher / facade
//! work can rely on the error surface while the real
//! validation + lifecycle is being built out:
//!
//! 1. Happy path — any well-formed input is rejected with
//!    `DomainError::NotSupported`. No aggregate is built, no
//!    event is emitted.
//! 2. Validation-failure path — the stub does not validate
//!    its input, so any payload (including a malformed one
//!    that would fail the future spec invariants on
//!    `Status`, `StudentDone`, `TotalMarks`) is rejected
//!    with the same `NotSupported` error before any
//!    field-level check runs.
//!
//! Once the real handler lands, the happy-path test will be
//! updated to assert `StudentTakeOnlineExamCreated` with
//! `version == 1` per the spec invariant
//! (`total_marks >= 0`); the validation-failure test will
//! then assert `DomainError::Validation` directly.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (lean — stub contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{StartOnlineExamCommand, SubmitOnlineExamAnswerCommand};
use educore_assessment::services::{start_online_exam, submit_online_exam_answer};
use educore_assessment::value_objects::OnlineExamId;
use educore_core::clock::{IdGenerator, SystemIdGen};
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

/// Mint an `OnlineExamId` for the given school — the
/// command-side handle the student-take flow keys off.
fn online_exam_id(g: &SystemIdGen, school: SchoolId) -> OnlineExamId {
    OnlineExamId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `start_online_exam` —
/// the natural "create" handler on the
/// `StudentTakeOnlineExam` aggregate. The handler is
/// currently a stub that returns
/// `DomainError::NotSupported("TODO: start_online_exam")`
/// before any aggregate construction or event minting
/// happens. Once the real implementation lands (carrying
/// `Status`, `StudentDone`, `TotalMarks` per
/// `docs/specs/assessment/aggregates.md` §
/// StudentTakeOnlineExam), this test will be updated to
/// assert that:
///
/// - The returned event is `StudentTakeOnlineExamCreated`
///   with `version == 1`,
/// - The aggregate is school-scoped and active,
/// - `total_marks >= 0` is enforced.
#[tokio::test]
async fn student_take_online_exam_start_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = StartOnlineExamCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let err = start_online_exam(cmd)
        .await
        .expect_err("start_online_exam is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// =============================================================================
// Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `submit_online_exam_answer`
/// — the natural "update" handler on the
/// `StudentTakeOnlineExam` aggregate (the student submits
/// answers, the take flips from `Open` to `Done`). The
/// handler is currently a stub that returns
/// `DomainError::NotSupported("TODO:
/// submit_online_exam_answer")` before any state transition
/// or event minting happens. Once the real implementation
/// lands, this test will be updated to assert that:
///
/// - The returned event is `StudentTakeOnlineExamCreated`
///   (re-emitted with the submit transition),
/// - The aggregate's `StudentDone` flag flips to `true`
///   and `version` increments.
#[tokio::test]
async fn student_take_online_exam_submit_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = SubmitOnlineExamAnswerCommand {
        school_id: school,
        online_exam_id: online_exam_id(&g, school),
    };

    let err = submit_online_exam_answer(cmd)
        .await
        .expect_err("submit_online_exam_answer is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}
