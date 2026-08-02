//! Integration tests for the **FmFeesTransactionChild aggregate**
//! vertical slice.
//!
//! Covers the behavioral contract for the Wave 77 per-aggregate drop
//! [`RealFmFeesTransactionChild`](educore_finance::aggregate::RealFmFeesTransactionChild)
//! — the child row under a `FmFeesTransaction` aggregate. Validates:
//!
//! - FFTC I-1: `amount_minor >= 0` (validated in fresh + update_metadata)
//! - FFTC I-2: parent `FmFeesTransactionId` belongs to the same school
//!   as the child id (cross-school defense-in-depth at the aggregate
//!   surface; existence check is the dispatcher's concern). Parent
//!   reference is immutable on update (the spec forbids re-parenting).
//! - `update_metadata()` (version + timestamp bump + re-validation)
//! - `retire()` (active → retired transition)
//! - `create_fm_fees_transaction_child` service function (aggregate +
//!   event pairing, cross-school defense-in-depth)
//!
//! The pre-existing 2 typed-id-only tests have been preserved (as
//! smoke tests for the typed-id contract) and the suite is extended
//! below with behavioral tests covering the Wave 77 full drop.

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
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::CreateFmFeesTransactionChildCommand;
use educore_finance::events::FmFeesTransactionChildCreated;
use educore_finance::prelude::RealFmFeesTransactionChild;
use educore_finance::services::create_fm_fees_transaction_child;
use educore_finance::value_objects::{FmFeesTransactionChildId, FmFeesTransactionId};

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

fn fm_fees_transaction_child_id(g: &SystemIdGen, school: SchoolId) -> FmFeesTransactionChildId {
    FmFeesTransactionChildId::new(school, g.next_uuid())
}

fn fm_fees_transaction_id(g: &SystemIdGen, school: SchoolId) -> FmFeesTransactionId {
    FmFeesTransactionId::new(school, g.next_uuid())
}

fn make_child(g: &SystemIdGen, school: SchoolId, amount_minor: i64) -> RealFmFeesTransactionChild {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealFmFeesTransactionChild::fresh(
        fm_fees_transaction_child_id(g, school),
        fm_fees_transaction_id(g, school),
        amount_minor,
        Some("Test line".to_owned()),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// Typed-id contract (preserved from Phase 7 Workstream G seed)
// ---------------------------------------------------------------------------

#[test]
fn fm_fees_transaction_child_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_transaction_child_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fm_fees_transaction_child_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fm_fees_transaction_child_id(&g, school);
    let id_b = fm_fees_transaction_child_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// RealFmFeesTransactionChild: fresh() — FFTC I-1 + I-2
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_amount_and_same_school_parent_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 50_000);
    assert_eq!(child.amount_minor, 50_000);
    assert!(child.is_active(), "fresh aggregate must be Active");
    assert_eq!(child.school_id, school);
    // FFTC I-2: parent reference belongs to the same school as the
    // child id (verified by construction in fresh_aggregate.rs).
    assert_eq!(child.fm_fees_transaction_id.school_id(), school);
}

#[test]
fn fresh_with_zero_amount_succeeds() {
    // Zero-amount child rows (waivers, free samples) are valid.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 0);
    assert_eq!(child.amount_minor, 0);
    assert!(child.is_active());
}

#[test]
fn fresh_with_negative_amount_returns_validation_error() {
    // FFTC I-1: amount_minor must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealFmFeesTransactionChild::fresh(
        fm_fees_transaction_child_id(&g, school),
        fm_fees_transaction_id(&g, school),
        -1,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must fail with Validation (FFTC I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_cross_school_parent_returns_validation_error() {
    // FFTC I-2 (cross-school): parent fm_fees_transaction_id must
    // belong to the same school as the child id.
    let (tenant, g) = admin_context();
    let child_school = tenant.school_id;
    let other_school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let parent_id_in_other_school = fm_fees_transaction_id(&g, other_school);
    let result = RealFmFeesTransactionChild::fresh(
        fm_fees_transaction_child_id(&g, child_school),
        parent_id_in_other_school,
        50_000,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "cross-school parent must fail with Validation (FFTC I-2), got {result:?}"
    );
}

#[test]
fn fresh_trims_surrounding_whitespace_in_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let child = RealFmFeesTransactionChild::fresh(
        fm_fees_transaction_child_id(&g, school),
        fm_fees_transaction_id(&g, school),
        1_000,
        Some("  Test line  ".to_owned()),
        actor,
        now,
        corr,
    )
    .expect("valid input");
    assert_eq!(child.description.as_deref(), Some("Test line"));
}

#[test]
fn fresh_with_empty_description_stores_none() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let child = RealFmFeesTransactionChild::fresh(
        fm_fees_transaction_child_id(&g, school),
        fm_fees_transaction_id(&g, school),
        1_000,
        Some("   ".to_owned()),
        actor,
        now,
        corr,
    )
    .expect("valid input");
    assert_eq!(child.description, None);
}

// ---------------------------------------------------------------------------
// RealFmFeesTransactionChild: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_valid_amount_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 50_000);
    let initial_version = child.version;
    let initial_updated_at = child.updated_at;
    let actor = g.next_user_id();
    let advanced = educore_core::value_objects::Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );

    child
        .update_metadata(75_000, Some("Revised line".to_owned()), advanced, actor)
        .expect("valid update");

    assert_eq!(child.amount_minor, 75_000);
    assert_eq!(child.description.as_deref(), Some("Revised line"));
    assert!(child.version > initial_version);
    assert!(child.updated_at > initial_updated_at);
    assert_eq!(child.updated_by, actor);
}

#[test]
fn update_metadata_with_negative_amount_returns_validation_error() {
    // FFTC I-1 must hold on update as well as fresh.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 50_000);
    let initial_amount = child.amount_minor;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = child.update_metadata(-1, None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount on update must fail with Validation (FFTC I-1), got {result:?}"
    );
    assert_eq!(child.amount_minor, initial_amount);
}

#[test]
fn update_metadata_on_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 50_000);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    child.retire(now, actor).expect("first retire succeeds");
    let later = educore_core::value_objects::Timestamp::from_datetime(
        now.as_datetime() + chrono::Duration::seconds(1),
    );
    let result = child.update_metadata(75_000, None, later, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "update on retired must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealFmFeesTransactionChild: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 50_000);
    let initial_version = child.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(child.is_active());
    child.retire(now, actor).expect("first retire succeeds");
    assert!(!child.is_active(), "retire must flip is_active to false");
    assert!(child.version > initial_version);
    assert_eq!(child.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 50_000);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    child.retire(now, actor).expect("first retire succeeds");
    let result = child.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_fm_fees_transaction_child service function
// ---------------------------------------------------------------------------

#[test]
fn create_fm_fees_transaction_child_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child_id = fm_fees_transaction_child_id(&g, school);
    let parent_id = fm_fees_transaction_id(&g, school);
    let cmd = CreateFmFeesTransactionChildCommand {
        tenant: tenant.clone(),
        fm_fees_transaction_child_id: child_id,
        fm_fees_transaction_id: parent_id,
        amount_minor: 50_000,
        description: Some("Test line".to_owned()),
    };
    let clock = SystemClock;
    let (child, event) =
        create_fm_fees_transaction_child(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(child.amount_minor, 50_000);
    assert_eq!(child.fm_fees_transaction_id, parent_id);
    assert!(
        child.is_active(),
        "service-created aggregate must be Active"
    );
    assert_eq!(child.school_id, school);
    assert_eq!(child.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.fm_fees_transaction_child_id, child_id);
    assert_eq!(event.fm_fees_transaction_id, parent_id);
    assert_eq!(event.amount_minor, 50_000);
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        FmFeesTransactionChildCreated::EVENT_TYPE,
        "finance.fm_fees_transaction_child.created"
    );
    assert_eq!(
        FmFeesTransactionChildCreated::AGGREGATE_TYPE,
        "fm_fees_transaction_child"
    );
    assert_eq!(FmFeesTransactionChildCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_fm_fees_transaction_child_service_rejects_cross_school_id() {
    let (tenant, g) = admin_context();
    let other_school = g.next_school_id();
    let cmd = CreateFmFeesTransactionChildCommand {
        tenant: tenant.clone(),
        fm_fees_transaction_child_id: fm_fees_transaction_child_id(&g, other_school),
        fm_fees_transaction_id: fm_fees_transaction_id(&g, tenant.school_id),
        amount_minor: 50_000,
        description: None,
    };
    let clock = SystemClock;
    let result = create_fm_fees_transaction_child(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "cross-school id must fail with Validation, got {result:?}"
    );
}

#[test]
fn create_fm_fees_transaction_child_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let cmd = CreateFmFeesTransactionChildCommand {
        tenant: tenant.clone(),
        fm_fees_transaction_child_id: fm_fees_transaction_child_id(&g, school),
        fm_fees_transaction_id: fm_fees_transaction_id(&g, school),
        amount_minor: -100, // FFTC I-1 violation
        description: None,
    };
    let clock = SystemClock;
    let result = create_fm_fees_transaction_child(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must propagate Validation (FFTC I-1), got {result:?}"
    );
}
