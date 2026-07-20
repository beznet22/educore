//! Integration tests for the **QuestionBankFee aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 68 per-aggregate drop
//! [`RealQuestionBankFee`](educore_finance::aggregate::RealQuestionBankFee) —
//! the per-question fee amount attached to the school's question bank,
//! per v3 Part 2 F62. Validates QBF I-1 (`amount_minor` must be ≥ 0),
//! `update_metadata()` (version + timestamp bump), `retire()` (active →
//! retired transition), and the `create_question_bank_fee` service
//! function (aggregate + event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `QuestionBankFee` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! QuestionBankFee { _id: () } }` placeholder. Wave 68 adds the
//! `RealQuestionBankFee` aggregate, the 3 headline events, the service
//! function, and this test suite.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::CreateQuestionBankFeeCommand;
use educore_finance::events::QuestionBankFeeCreated;
use educore_finance::prelude::RealQuestionBankFee;
use educore_finance::services::create_question_bank_fee;
use educore_finance::value_objects::QuestionBankFeeId;

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

fn question_bank_fee_id(g: &SystemIdGen, school: SchoolId) -> QuestionBankFeeId {
    QuestionBankFeeId::new(school, g.next_uuid())
}

fn make_question_bank_fee(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    amount_minor: i64,
    description: Option<&str>,
) -> RealQuestionBankFee {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealQuestionBankFee::fresh(
        question_bank_fee_id(g, school),
        name.to_owned(),
        amount_minor,
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealQuestionBankFee: fresh() — QBF I-1 invariant
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_positive_amount_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, Some("Per paper"));
    assert_eq!(fee.name, "Re-marking fee");
    assert_eq!(fee.amount_minor, 5000);
    assert_eq!(fee.description.as_deref(), Some("Per paper"));
    assert!(fee.is_active(), "fresh aggregate must be Active");
    assert_eq!(fee.school_id, school);
}

#[test]
fn fresh_with_zero_amount_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let fee = make_question_bank_fee(&g, school, "Free sample", 0, None);
    assert_eq!(fee.amount_minor, 0);
    assert!(fee.is_active(), "zero amount is allowed (>= 0), aggregate must be Active");
    assert!(fee.description.is_none());
}

#[test]
fn fresh_trims_whitespace_in_name() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let fee = make_question_bank_fee(&g, school, "  Lab fee  ", 1000, None);
    assert_eq!(fee.name, "Lab fee");
}

#[test]
fn fresh_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealQuestionBankFee::fresh(
        question_bank_fee_id(&g, school),
        String::new(),
        1000,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty name must fail with Validation, got {result:?}"
    );
}

#[test]
fn fresh_with_whitespace_only_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealQuestionBankFee::fresh(
        question_bank_fee_id(&g, school),
        "   \t\n  ".to_owned(),
        1000,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only name must fail with Validation, got {result:?}"
    );
}

#[test]
fn fresh_with_negative_amount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealQuestionBankFee::fresh(
        question_bank_fee_id(&g, school),
        "Bad fee".to_owned(),
        -1,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must fail with Validation (QBF I-1), got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealQuestionBankFee: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, None);
    let initial_version = fee.version;
    let initial_updated_at = fee.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = fee.update_metadata(String::new(), 6000, None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty new name must fail with Validation, got {result:?}"
    );
    // Original state preserved on failed update.
    assert_eq!(fee.name, "Re-marking fee");
    assert_eq!(fee.amount_minor, 5000);
    assert_eq!(fee.version, initial_version);
    assert_eq!(fee.updated_at, initial_updated_at);
}

#[test]
fn update_metadata_with_negative_amount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, None);
    let initial_version = fee.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = fee.update_metadata("Re-marking fee".to_owned(), -1, None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative new amount_minor must fail with Validation (QBF I-1), got {result:?}"
    );
    assert_eq!(fee.amount_minor, 5000);
    assert_eq!(fee.version, initial_version);
}

#[test]
fn update_metadata_with_valid_inputs_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, Some("Initial"));
    let initial_version = fee.version;
    let initial_updated_at = fee.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced = Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );
    let actor = g.next_user_id();

    fee.update_metadata(
        "Re-marking fee v2".to_owned(),
        7500,
        Some("Bumped".to_owned()),
        advanced,
        actor,
    )
    .expect("valid update");

    assert_eq!(fee.name, "Re-marking fee v2");
    assert_eq!(fee.amount_minor, 7500);
    assert_eq!(fee.description.as_deref(), Some("Bumped"));
    assert!(
        fee.version > initial_version,
        "version must advance on update (was {initial_version:?}, now {:?})",
        fee.version
    );
    assert!(
        fee.updated_at > initial_updated_at,
        "updated_at must advance on update (was {initial_updated_at:?}, now {:?})",
        fee.updated_at
    );
    assert_eq!(fee.updated_by, actor);
}

#[test]
fn update_metadata_with_empty_description_clears_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, Some("Initial"));
    let actor = g.next_user_id();
    let now = SystemClock.now();
    fee.update_metadata("Re-marking fee".to_owned(), 5000, None, now, actor)
        .expect("valid update");
    assert!(fee.description.is_none());
}

// ---------------------------------------------------------------------------
// RealQuestionBankFee: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, None);
    let initial_version = fee.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(fee.is_active());
    fee.retire(now, actor).expect("first retire succeeds");
    assert!(!fee.is_active(), "retire must flip is_active to false");
    assert!(fee.version > initial_version);
    assert_eq!(fee.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut fee = make_question_bank_fee(&g, school, "Re-marking fee", 5000, None);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    fee.retire(now, actor).expect("first retire succeeds");
    let result = fee.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_question_bank_fee service function
// ---------------------------------------------------------------------------

#[test]
fn create_question_bank_fee_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateQuestionBankFeeCommand {
        tenant: tenant.clone(),
        name: "Re-marking fee".to_owned(),
        amount_minor: 5000,
        description: Some("Per paper re-marking".to_owned()),
    };
    let clock = SystemClock;
    let (fee, event) =
        create_question_bank_fee(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(fee.name, "Re-marking fee");
    assert_eq!(fee.amount_minor, 5000);
    assert_eq!(fee.description.as_deref(), Some("Per paper re-marking"));
    assert!(fee.is_active(), "service-created aggregate must be Active");
    assert_eq!(fee.school_id, tenant.school_id);
    assert_eq!(fee.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.question_bank_fee_id, fee.id);
    assert_eq!(event.name, "Re-marking fee");
    assert_eq!(event.amount_minor, 5000);
    assert_eq!(
        event.description.as_deref(),
        Some("Per paper re-marking")
    );
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        QuestionBankFeeCreated::EVENT_TYPE,
        "finance.question_bank_fee.created"
    );
    assert_eq!(QuestionBankFeeCreated::AGGREGATE_TYPE, "question_bank_fee");
    assert_eq!(QuestionBankFeeCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_question_bank_fee_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateQuestionBankFeeCommand {
        tenant: tenant.clone(),
        name: "Bad fee".to_owned(),
        amount_minor: -1,
        description: None,
    };
    let clock = SystemClock;
    let result = create_question_bank_fee(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must propagate Validation (QBF I-1), got {result:?}"
    );
}
