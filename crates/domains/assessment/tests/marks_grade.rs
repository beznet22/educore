//! Integration tests for the **MarksGrade aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`MarksGrade`](educore_assessment::aggregate::MarksGrade)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `create_marks_grade` in `services.rs` is a **stub**
//! (`DomainError::not_supported("TODO: create_marks_grade")`).
//! The full implementation lands in a follow-up phase. These
//! tests pin the **current** behaviour so the dispatcher /
//! facade work can rely on the error surface while the real
//! validation + aggregate construction is being built out:
//!
//! 1. Happy path — any well-formed input is rejected with
//!    `DomainError::NotSupported`. No aggregate is built, no
//!    event is emitted.
//! 2. Validation-failure path — the stub does not validate
//!    its input, so any payload (including ones that would
//!    fail `PercentFrom > PercentUpTo` once the real
//!    validator lands) is rejected with the same
//!    `NotSupported` error before any field-level check runs.
//!
//! Once the real handler lands, the happy-path test will be
//! updated to assert `MarksGradeCreated` + `version == 1`
//! per the spec invariant
//! (`PercentFrom < PercentUpTo`); the validation-failure test
//! will then assert `DomainError::Validation` directly.
//!
//! Mirrors `crates/domains/assessment/tests/exam.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::CreateMarksGradeCommand;
use educore_assessment::services::create_marks_grade;
use educore_assessment::value_objects::MarksGradeId;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
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

/// Mint a `MarksGradeId` for the given school.
fn marks_grade_id(g: &SystemIdGen, school: SchoolId) -> MarksGradeId {
    MarksGradeId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `create_marks_grade` for
/// a well-formed payload. The handler is currently a stub
/// that returns `DomainError::NotSupported("TODO:
/// create_marks_grade")` before any aggregate construction
/// or event minting happens. Once the real implementation
/// lands (carrying `GradeName`, `Gpa`, `From`, `Up`,
/// `PercentFrom`, `PercentUpTo`, `Description` per
/// `docs/specs/assessment/aggregates.md` § MarksGrade), this
/// test will be updated to assert that:
///
/// - The returned event is `MarksGradeCreated` with
///   `version == 1`,
/// - The aggregate is school-scoped and active,
/// - `PercentFrom < PercentUpTo` is enforced.
#[tokio::test]
async fn marks_grade_create_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let cmd = CreateMarksGradeCommand {
        school_id: school,
        marks_grade_id: marks_grade_id(&g, school),
    };

    let err = create_marks_grade(cmd)
        .await
        .expect_err("create_marks_grade is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// =============================================================================
// Validation-failure path: current contract — stub is unconditional
// =============================================================================

/// Pins the **current** contract of `create_marks_grade` for
/// a payload that would, once the real validator lands,
/// violate spec invariant #2 (`PercentFrom < PercentUpTo`).
///
/// The stub does not differentiate input — every payload is
/// rejected with the same `NotSupported` error before any
/// field-level check runs. This test therefore asserts that
/// the stub is *unconditional* (i.e. the future
/// `PercentFrom > PercentUpTo` validation will not be
/// reachable until the stub is replaced). Once the real
/// handler lands, this test will be updated to assert
/// `DomainError::Validation` directly.
#[tokio::test]
async fn marks_grade_create_rejects_via_not_supported_independent_of_input() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    // A second well-formed command — the stub cannot tell
    // this apart from the happy-path command above; both
    // return the same `NotSupported` error.
    let cmd = CreateMarksGradeCommand {
        school_id: school,
        marks_grade_id: marks_grade_id(&g, school),
    };

    let err = create_marks_grade(cmd)
        .await
        .expect_err("create_marks_grade stub rejects unconditionally");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (stub is unconditional), got {err:?}"
    );
}
