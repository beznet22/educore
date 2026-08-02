//! Integration tests for the **FmFeesGroup aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 66 per-aggregate drop
//! [`RealFmFeesGroup`](educore_finance::aggregate::RealFmFeesGroup) —
//! the FM invoice scheme's fee-grouping primitive (per v3 Part 2
//! F40). Validates FFG I-1 (non-empty name after trim),
//! `update_metadata()` (version + timestamp bump), `retire()` (active
//! → retired transition), and the `create_fm_fees_group` service
//! function (aggregate + event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because the underlying `create_fm_fees_group` handler moved
//! from the Phase 7 Workstream G skeleton (which returned `Result<()>`
//! without constructing an aggregate) to the full Wave 66 drop
//! (returning `Result<(RealFmFeesGroup, FmFeesGroupCreated)>`).

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

use educore_finance::commands::CreateFmFeesGroupCommand;
use educore_finance::events::FmFeesGroupCreated;
use educore_finance::prelude::RealFmFeesGroup;
use educore_finance::services::create_fm_fees_group;
use educore_finance::value_objects::FmFeesGroupId;

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

fn fm_fees_group_id(g: &SystemIdGen, school: SchoolId) -> FmFeesGroupId {
    FmFeesGroupId::new(school, g.next_uuid())
}

fn make_fm_fees_group(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    description: Option<&str>,
) -> RealFmFeesGroup {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealFmFeesGroup::fresh(
        fm_fees_group_id(g, school),
        name.to_owned(),
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealFmFeesGroup: fresh() — FFG I-1 invariant
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_name_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let group = make_fm_fees_group(&g, school, "Tuition FM", Some("Annual tuition group"));
    assert_eq!(group.name, "Tuition FM");
    assert_eq!(group.description.as_deref(), Some("Annual tuition group"));
    assert!(group.is_active(), "fresh aggregate must be Active");
    assert_eq!(group.school_id, school);
}

#[test]
fn fresh_trims_whitespace_in_name() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let group = make_fm_fees_group(&g, school, "  Lab Fees  ", None);
    assert_eq!(group.name, "Lab Fees");
    assert!(group.description.is_none());
}

#[test]
fn fresh_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealFmFeesGroup::fresh(
        fm_fees_group_id(&g, school),
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
    let result = RealFmFeesGroup::fresh(
        fm_fees_group_id(&g, school),
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
// RealFmFeesGroup: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut group = make_fm_fees_group(&g, school, "Tuition FM", None);
    let initial_version = group.version;
    let initial_updated_at = group.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = group.update_metadata(String::new(), None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty new name must fail with Validation, got {result:?}"
    );
    // Original state preserved on failed update.
    assert_eq!(group.name, "Tuition FM");
    assert_eq!(group.version, initial_version);
    assert_eq!(group.updated_at, initial_updated_at);
}

#[test]
fn update_metadata_with_valid_name_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut group = make_fm_fees_group(&g, school, "Tuition FM", Some("Initial"));
    let initial_version = group.version;
    let initial_updated_at = group.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced =
        Timestamp::from_datetime(initial_updated_at.as_datetime() + chrono::Duration::seconds(1));
    let actor = g.next_user_id();

    group
        .update_metadata(
            "Tuition FM Q4".to_owned(),
            Some("Renamed for Q4".to_owned()),
            advanced,
            actor,
        )
        .expect("valid update");

    assert_eq!(group.name, "Tuition FM Q4");
    assert_eq!(group.description.as_deref(), Some("Renamed for Q4"));
    assert!(
        group.version > initial_version,
        "version must advance on update (was {initial_version:?}, now {:?})",
        group.version
    );
    assert!(
        group.updated_at > initial_updated_at,
        "updated_at must advance on update (was {initial_updated_at:?}, now {:?})",
        group.updated_at
    );
    assert_eq!(group.updated_by, actor);
}

#[test]
fn update_metadata_with_empty_description_clears_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut group = make_fm_fees_group(&g, school, "Tuition FM", Some("Initial"));
    let actor = g.next_user_id();
    let now = SystemClock.now();
    group
        .update_metadata("Tuition FM".to_owned(), None, now, actor)
        .expect("valid update");
    assert!(group.description.is_none());
}

// ---------------------------------------------------------------------------
// RealFmFeesGroup: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut group = make_fm_fees_group(&g, school, "Tuition FM", None);
    let initial_version = group.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(group.is_active());
    group.retire(now, actor).expect("first retire succeeds");
    assert!(!group.is_active(), "retire must flip is_active to false");
    assert!(group.version > initial_version);
    assert_eq!(group.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut group = make_fm_fees_group(&g, school, "Tuition FM", None);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    group.retire(now, actor).expect("first retire succeeds");
    let result = group.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_fm_fees_group service function
// ---------------------------------------------------------------------------

#[test]
fn create_fm_fees_group_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateFmFeesGroupCommand {
        tenant: tenant.clone(),
        name: "Lab Fees FM".to_owned(),
        description: Some("Laboratory fees group".to_owned()),
    };
    let clock = SystemClock;
    let (group, event) = create_fm_fees_group(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(group.name, "Lab Fees FM");
    assert_eq!(group.description.as_deref(), Some("Laboratory fees group"));
    assert!(
        group.is_active(),
        "service-created aggregate must be Active"
    );
    assert_eq!(group.school_id, tenant.school_id);
    assert_eq!(group.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.fm_fees_group_id, group.id);
    assert_eq!(event.name, "Lab Fees FM");
    assert_eq!(event.description.as_deref(), Some("Laboratory fees group"));
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        FmFeesGroupCreated::EVENT_TYPE,
        "finance.fm_fees_group.created"
    );
    assert_eq!(FmFeesGroupCreated::AGGREGATE_TYPE, "fm_fees_group");
    assert_eq!(FmFeesGroupCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_fm_fees_group_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateFmFeesGroupCommand {
        tenant: tenant.clone(),
        name: "   ".to_owned(),
        description: None,
    };
    let clock = SystemClock;
    let result = create_fm_fees_group(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only name must propagate Validation, got {result:?}"
    );
}
