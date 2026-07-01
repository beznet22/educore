//! Integration tests for the **QuestionGroup aggregate** vertical slice.
//!
//! Pins the create + update contracts for
//! [`QuestionGroup`](educore_assessment::aggregate::QuestionGroup)
//! end-to-end through the service layer.
//!
//! # Wave 29 batch 6 contract
//!
//! `create_question_group` and `update_question_group` in
//! `services.rs` are now **real implementations**. They
//! enforce the typed-id school-anchoring invariant (the
//! id's `school_id` must equal the command's `school_id`;
//! cross-tenant references are rejected with
//! `DomainError::Validation`) and emit the spec-defined
//! `QuestionGroupCreated` event for every transition. The
//! full `Name` / `Active` payload validation lands in a
//! follow-up batch once the `TenantContext`-bearing
//! command struct is migrated.
//!
//! Mirrors `crates/domains/assessment/tests/question_bank.rs`
//! (lean — real-handler contract pin).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    CreateQuestionGroupCommand, UpdateQuestionGroupCommand,
};
use educore_assessment::services::{create_question_group, update_question_group};
use educore_assessment::value_objects::QuestionGroupId;
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

/// Mint a `QuestionGroupId` for the given school.
fn question_group_id(g: &SystemIdGen, school: SchoolId) -> QuestionGroupId {
    QuestionGroupId::new(school, g.next_uuid())
}

// =============================================================================
// create_question_group
// =============================================================================

/// Pins the **happy path** of `create_question_group` for
/// a well-formed payload: a same-school typed id is
/// accepted and the returned event is `QuestionGroupCreated`
/// carrying the command's school and the typed id.
#[tokio::test]
async fn question_group_create_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = question_group_id(&g, school);
    let cmd = CreateQuestionGroupCommand {
        school_id: school,
        question_group_id: id,
    };

    let event = create_question_group(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.question_group_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `create_question_group`: a typed id from a different
/// school is rejected with `DomainError::Validation`.
#[tokio::test]
async fn question_group_create_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = question_group_id(&g, other_school);
    let cmd = CreateQuestionGroupCommand {
        school_id: actor_school,
        question_group_id: foreign_id,
    };

    let err = create_question_group(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// update_question_group
// =============================================================================

/// Pins the **happy path** of `update_question_group` for
/// a well-formed payload: a same-school typed id is
/// accepted and the returned event is `QuestionGroupCreated`
/// carrying the command's school and the typed id (the
/// update-transition event re-emits the same shape until
/// the full `Name` / `Active`-flip payload is migrated
/// onto the command struct).
#[tokio::test]
async fn question_group_update_happy_path() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _clock = TestClock::new();
    let _ids = SystemIdGen;

    let id = question_group_id(&g, school);
    let cmd = UpdateQuestionGroupCommand {
        school_id: school,
        question_group_id: id,
    };

    let event = update_question_group(cmd)
        .await
        .expect("real handler accepts well-formed input");
    assert_eq!(event.school_id, school, "event school echoes command");
    assert_eq!(event.question_group_id, id, "event id echoes command");
    // Version-7 event id must be a valid UUID.
    let _: uuid::Uuid = event.event_id.as_uuid();
}

/// Pins the **cross-tenant rejection** contract of
/// `update_question_group`: a typed id from a different
/// school is rejected with `DomainError::Validation`.
#[tokio::test]
async fn question_group_update_cross_tenant_rejected() {
    let (tenant, g) = admin_context();
    let actor_school = tenant.school_id;
    let other_school = g.next_school_id();

    let foreign_id = question_group_id(&g, other_school);
    let cmd = UpdateQuestionGroupCommand {
        school_id: actor_school,
        question_group_id: foreign_id,
    };

    let err = update_question_group(cmd)
        .await
        .expect_err("cross-tenant id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
