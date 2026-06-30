//! Integration tests for the **QuestionBank aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`QuestionBank`](educore_assessment::aggregate::QuestionBank)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 4 vertical slice)
//!
//! `create_question` and `update_question` in `services.rs`
//! are **stubs** that unconditionally return
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
//!    `Question`, `QuestionType`, `Mark`, `Group`,
//!    `Level`, `Class`, `Section`, `Subject`) is rejected
//!    with the same `NotSupported` error before any
//!    field-level check runs.
//!
//! Once the real handler lands, the happy-path test will be
//! updated to assert `QuestionBankCreated` with `version ==
//! 1` per the spec invariant (`mark >= 0`); the
//! validation-failure test will then assert
//! `DomainError::Validation` directly.
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

use educore_assessment::commands::{CreateQuestionCommand, UpdateQuestionCommand};
use educore_assessment::services::{create_question, update_question};
use educore_assessment::value_objects::QuestionBankId;
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

/// Mint a `QuestionBankId` for the given school.
fn question_bank_id(g: &SystemIdGen, school: SchoolId) -> QuestionBankId {
    QuestionBankId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `create_question` for a
/// well-formed payload. The handler is currently a stub
/// that returns `DomainError::NotSupported("TODO:
/// create_question")` before any aggregate construction or
/// event minting happens. Once the real implementation
/// lands (carrying `Question`, `QuestionType`, `Mark`,
/// `Group`, `Level`, `Class`, `Section`, `Subject` per
/// `docs/specs/assessment/aggregates.md` § QuestionBank),
/// this test will be updated to assert that:
///
/// - The returned event is `QuestionBankCreated` with
///   `version == 1`,
/// - The aggregate is school-scoped and active,
/// - `mark >= 0` is enforced.
#[tokio::test]
async fn question_bank_create_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = CreateQuestionCommand {
        school_id: school,
        question_bank_id: question_bank_id(&g, school),
    };

    let err = create_question(cmd)
        .await
        .expect_err("create_question is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}

// =============================================================================
// Update path: current contract — stub returns NotSupported
// =============================================================================

/// Pins the **current** contract of `update_question` — the
/// natural "update" handler on the `QuestionBank`
/// aggregate. The handler is currently a stub that returns
/// `DomainError::NotSupported("TODO: update_question")`
/// before any state transition or event minting happens.
/// Once the real implementation lands, this test will be
/// updated to assert that:
///
/// - The returned event is `QuestionBankCreated` (re-emitted
///   with the update transition),
/// - The aggregate's `version` increments and the etag
///   rotates.
#[tokio::test]
async fn question_bank_update_currently_returns_not_supported() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    let cmd = UpdateQuestionCommand {
        school_id: school,
        question_bank_id: question_bank_id(&g, school),
    };

    let err = update_question(cmd)
        .await
        .expect_err("update_question is currently a stub");
    assert!(
        matches!(err, DomainError::NotSupported(_)),
        "expected NotSupported (current stub contract), got {err:?}"
    );
}
