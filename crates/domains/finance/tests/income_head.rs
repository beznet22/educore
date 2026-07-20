//! Integration tests for the **IncomeHead aggregate** vertical slice.
//!
//! Covers two layers:
//!
//! 1. The typed-id contract for
//!    [`IncomeHead`](educore_finance::aggregate::IncomeHead) end-to-end.
//! 2. The behavioral contract for the Wave 65 per-aggregate drop
//!    [`RealIncomeHead`](educore_finance::aggregate::RealIncomeHead) —
//!    the income category catalogue entry. Validates F52 I-1
//!    (non-empty name after trim), `update_metadata()` (version +
//!    timestamp bump), `retire()` (active → retired transition), and
//!    the `create_income_head` service function (aggregate + event
//!    pairing).

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

use educore_finance::commands::CreateIncomeHeadCommand;
use educore_finance::events::IncomeHeadCreated;
use educore_finance::prelude::RealIncomeHead;
use educore_finance::services::create_income_head;
use educore_finance::value_objects::IncomeHeadId;

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

fn income_head_id(g: &SystemIdGen, school: SchoolId) -> IncomeHeadId {
    IncomeHeadId::new(school, g.next_uuid())
}

fn make_income_head(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    description: Option<&str>,
) -> RealIncomeHead {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealIncomeHead::fresh(
        income_head_id(g, school),
        name.to_owned(),
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// Typed-id contract
// ---------------------------------------------------------------------------

#[test]
fn income_head_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_head_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn income_head_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = income_head_id(&g, school);
    let id_b = income_head_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// RealIncomeHead: fresh() — F52 I-1 invariant
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_name_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let head = make_income_head(&g, school, "Donations", Some("Gifts from parents"));
    assert_eq!(head.name, "Donations");
    assert_eq!(head.description.as_deref(), Some("Gifts from parents"));
    assert!(head.is_active(), "fresh aggregate must be Active");
    assert_eq!(head.school_id, school);
}

#[test]
fn fresh_trims_whitespace_in_name() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let head = make_income_head(&g, school, "  Rentals  ", None);
    assert_eq!(head.name, "Rentals");
    assert!(head.description.is_none());
}

#[test]
fn fresh_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealIncomeHead::fresh(
        income_head_id(&g, school),
        String::new(),
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
    let result = RealIncomeHead::fresh(
        income_head_id(&g, school),
        "   \t\n  ".to_owned(),
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

// ---------------------------------------------------------------------------
// RealIncomeHead: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut head = make_income_head(&g, school, "Donations", None);
    let initial_version = head.version;
    let initial_updated_at = head.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = head.update_metadata(String::new(), None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty new name must fail with Validation, got {result:?}"
    );
    // Original state preserved on failed update.
    assert_eq!(head.name, "Donations");
    assert_eq!(head.version, initial_version);
    assert_eq!(head.updated_at, initial_updated_at);
}

#[test]
fn update_metadata_with_valid_name_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut head = make_income_head(&g, school, "Donations", Some("Initial"));
    let initial_version = head.version;
    let initial_updated_at = head.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced = Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );
    let actor = g.next_user_id();

    head.update_metadata(
        "Donations Q4".to_owned(),
        Some("Renamed for Q4".to_owned()),
        advanced,
        actor,
    )
    .expect("valid update");

    assert_eq!(head.name, "Donations Q4");
    assert_eq!(head.description.as_deref(), Some("Renamed for Q4"));
    assert!(
        head.version > initial_version,
        "version must advance on update (was {initial_version:?}, now {:?})",
        head.version
    );
    assert!(
        head.updated_at > initial_updated_at,
        "updated_at must advance on update (was {initial_updated_at:?}, now {:?})",
        head.updated_at
    );
    assert_eq!(head.updated_by, actor);
}

#[test]
fn update_metadata_with_empty_description_clears_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut head = make_income_head(&g, school, "Donations", Some("Initial"));
    let actor = g.next_user_id();
    let now = SystemClock.now();
    head.update_metadata("Donations".to_owned(), None, now, actor)
        .expect("valid update");
    assert!(head.description.is_none());
}

// ---------------------------------------------------------------------------
// RealIncomeHead: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut head = make_income_head(&g, school, "Donations", None);
    let initial_version = head.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(head.is_active());
    head.retire(now, actor).expect("first retire succeeds");
    assert!(!head.is_active(), "retire must flip is_active to false");
    assert!(head.version > initial_version);
    assert_eq!(head.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut head = make_income_head(&g, school, "Donations", None);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    head.retire(now, actor).expect("first retire succeeds");
    let result = head.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_income_head service function
// ---------------------------------------------------------------------------

#[test]
fn create_income_head_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateIncomeHeadCommand {
        tenant: tenant.clone(),
        name: "Sales".to_owned(),
        description: Some("Product sales income".to_owned()),
    };
    let clock = SystemClock;
    let (head, event) = create_income_head(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(head.name, "Sales");
    assert_eq!(head.description.as_deref(), Some("Product sales income"));
    assert!(head.is_active(), "service-created aggregate must be Active");
    assert_eq!(head.school_id, tenant.school_id);
    assert_eq!(head.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.income_head_id, head.id);
    assert_eq!(event.name, "Sales");
    assert_eq!(
        event.description.as_deref(),
        Some("Product sales income")
    );
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(IncomeHeadCreated::EVENT_TYPE, "finance.income_head.created");
    assert_eq!(IncomeHeadCreated::AGGREGATE_TYPE, "income_head");
    assert_eq!(IncomeHeadCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_income_head_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateIncomeHeadCommand {
        tenant: tenant.clone(),
        name: "   ".to_owned(),
        description: None,
    };
    let clock = SystemClock;
    let result = create_income_head(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only name must propagate Validation, got {result:?}"
    );
}
