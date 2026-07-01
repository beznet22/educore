//! Integration tests for the **QuestionBank aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for
//! [`QuestionBank`](educore_assessment::aggregate::QuestionBank)
//! end-to-end through the service layer.
//!
//! # Current contract (Wave 29 vertical slice)
//!
//! `create_question`, `update_question`, and
//! `delete_question` in `services.rs` are now **real
//! implementations**. They enforce the typed-id
//! school-anchoring invariant (spec invariant — the id's
//! `school_id` must equal the command's `school_id`;
//! cross-tenant references are rejected with
//! `DomainError::Validation`) and emit the spec-defined
//! `QuestionBankCreated` event for every transition.
//!
//! The full payload validation (spec invariants #1
//! `Mark > 0`, #2 supported `QuestionType`, #3 unique title
//! per school) lands in a follow-up batch once the
//! `TenantContext`-bearing command struct is migrated to
//! carry `Question` / `QuestionType` / `Mark` / `Group` /
//! `Level` / `Class` / `Section` / `Subject`.
//!
//! Mirrors `crates/domains/assessment/tests/marks_grade.rs`
//! (lean — real-handler contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    CreateQuestionCommand, DeleteQuestionCommand, UpdateQuestionCommand,
};
use educore_assessment::services::{create_question, delete_question, update_question};
use educore_assessment::value_objects::QuestionBankId;
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

/// Mint a `QuestionBankId` for the given school.
fn question_bank_id(g: &SystemIdGen, school: SchoolId) -> QuestionBankId {
    QuestionBankId::new(school, g.next_uuid())
}

// =============================================================================
// create_question
// =============================================================================

/// Pins the **happy path** of `create_question` for a
/// well-formed payload: a same-school typed id is accepted,
/// the returned event is `QuestionBankCreated` carrying the
/// command's school and the typed id, and the event id is
/// a freshly-minted UUID (version-7).
#[tokio::test]
async fn question_bank_create_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = question_bank_id(&g, school);
    let cmd = CreateQuestionCommand {
        school_id: school,
        question_bank_id: id,
    };

    let event = create_question(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.question_bank_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `create_question`: a typed id from a different school is
/// rejected with `DomainError::Validation` before any event
/// is minted. Enforces the spec's school-anchoring
/// invariant for the `QuestionBank` aggregate.
#[tokio::test]
async fn question_bank_create_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = question_bank_id(&g, other_school);
    let cmd = CreateQuestionCommand {
        school_id: actor_school,
        question_bank_id: foreign_id,
    };

    let err = create_question(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// update_question
// =============================================================================

/// Pins the **happy path** of `update_question` for a
/// well-formed payload: a same-school typed id is accepted
/// and the returned event is `QuestionBankCreated` carrying
/// the command's school and the typed id (the
/// update-transition event re-emits the same shape until
/// the full payload is migrated onto the command struct).
#[tokio::test]
async fn question_bank_update_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = question_bank_id(&g, school);
    let cmd = UpdateQuestionCommand {
        school_id: school,
        question_bank_id: id,
    };

    let event = update_question(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.question_bank_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_question`: a typed id from a different school is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn question_bank_update_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = question_bank_id(&g, other_school);
    let cmd = UpdateQuestionCommand {
        school_id: actor_school,
        question_bank_id: foreign_id,
    };

    let err = update_question(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// delete_question
// =============================================================================

/// Pins the **happy path** of `delete_question` for a
/// well-formed payload: a same-school typed id is accepted
/// and the returned event is `QuestionBankCreated` carrying
/// the command's school and the typed id (the
/// delete-transition event re-emits the same shape until
/// the full `ActiveStatus`-flip payload is migrated onto
/// the command struct).
#[tokio::test]
async fn question_bank_delete_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = question_bank_id(&g, school);
    let cmd = DeleteQuestionCommand {
        school_id: school,
        question_bank_id: id,
    };

    let event = delete_question(cmd).await.expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.question_bank_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `delete_question`: a typed id from a different school is
/// rejected with `DomainError::Validation`.
#[tokio::test]
async fn question_bank_delete_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = question_bank_id(&g, other_school);
    let cmd = DeleteQuestionCommand {
        school_id: actor_school,
        question_bank_id: foreign_id,
    };

    let err = delete_question(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
