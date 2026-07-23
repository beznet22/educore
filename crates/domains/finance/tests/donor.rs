//! Integration tests for the **Donor aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 71 per-aggregate drop
//! [`RealDonor`](educore_finance::aggregate::RealDonor) — the school's
//! donor directory entry (alumni, parents, foundations that donate
//! funds). Validates DO I-1 (`show_public` is a boolean, pinned by the
//! Rust `bool` type), DO I-2 (email non-empty / 1..=200 chars / contains
//! `@`), `update_metadata()` (version + timestamp bump), `retire()`
//! (active → retired transition), and the `create_donor` service
//! function (aggregate + event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `Donor` previously had no real implementation beyond
//! a `finance_aggregate_stub! { struct Donor { _id: () } }`
//! placeholder. Wave 71 adds the `RealDonor` aggregate, the 3
//! headline events, the service function (replacing the Phase 7
//! Workstream D skeleton), and this test suite.

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

use educore_finance::commands::CreateDonorCommand;
use educore_finance::events::DonorCreated;
use educore_finance::prelude::RealDonor;
use educore_finance::services::create_donor;
use educore_finance::value_objects::DonorId;

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

fn donor_id(g: &SystemIdGen, school: SchoolId) -> DonorId {
    DonorId::new(school, g.next_uuid())
}

fn make_donor(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    email: &str,
    show_public: bool,
    phone: Option<&str>,
    description: Option<&str>,
) -> RealDonor {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealDonor::fresh(
        donor_id(g, school),
        name.to_owned(),
        email.to_owned(),
        show_public,
        phone.map(str::to_owned),
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealDonor: fresh() — DO I-2 invariant (email validation)
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_inputs_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let donor = make_donor(
        &g,
        school,
        "Ada Lovelace",
        "ada@example.org",
        true,
        Some("+44-20-7946-0958"),
        Some("Mathematical patron"),
    );
    assert_eq!(donor.name, "Ada Lovelace");
    assert_eq!(donor.email, "ada@example.org");
    assert!(donor.show_public, "DO I-1: show_public is a bool, set true");
    assert_eq!(donor.phone.as_deref(), Some("+44-20-7946-0958"));
    assert_eq!(donor.description.as_deref(), Some("Mathematical patron"));
    assert!(donor.is_active(), "fresh aggregate must be Active");
    assert_eq!(donor.school_id, school);
}

#[test]
fn fresh_with_show_public_false_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    // DO I-1: `show_public` is a boolean (always satisfied by Rust's
    // bool type). Pin both branches explicitly.
    let donor = make_donor(&g, school, "Anonymous Patron", "anon@example.org", false, None, None);
    assert!(!donor.show_public, "DO I-1: show_public is a bool, set false");
    assert!(donor.is_active());
    assert!(donor.phone.is_none());
    assert!(donor.description.is_none());
}

#[test]
fn fresh_trims_whitespace_in_name_and_email() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let donor = make_donor(&g, school, "  Grace Hopper  ", "  grace@navy.mil  ", true, None, None);
    assert_eq!(donor.name, "Grace Hopper");
    assert_eq!(donor.email, "grace@navy.mil");
}

#[test]
fn fresh_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDonor::fresh(
        donor_id(&g, school),
        String::new(),
        "ada@example.org".to_owned(),
        true,
        None,
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
fn fresh_with_empty_email_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDonor::fresh(
        donor_id(&g, school),
        "Ada Lovelace".to_owned(),
        String::new(),
        true,
        None,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty email must fail with Validation (DO I-2), got {result:?}"
    );
}

#[test]
fn fresh_with_email_missing_at_sign_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDonor::fresh(
        donor_id(&g, school),
        "Ada Lovelace".to_owned(),
        "not-an-email".to_owned(),
        true,
        None,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "email without '@' must fail with Validation (DO I-2), got {result:?}"
    );
}

#[test]
fn fresh_with_email_over_200_chars_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let local = "a".repeat(64);
    let overlong = format!("{local}@{local}.{local}.{local}.{local}.example.org");
    assert!(overlong.len() > 200, "fixture: email must be > 200 chars");
    let result = RealDonor::fresh(
        donor_id(&g, school),
        "Ada Lovelace".to_owned(),
        overlong,
        true,
        None,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "email over 200 chars must fail with Validation (DO I-2), got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealDonor: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_empty_email_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(&g, school, "Ada Lovelace", "ada@example.org", true, None, None);
    let initial_version = donor.version;
    let initial_updated_at = donor.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = donor.update_metadata(
        "Ada Lovelace".to_owned(),
        String::new(),
        false,
        None,
        None,
        now,
        actor,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty new email must fail with Validation (DO I-2), got {result:?}"
    );
    assert_eq!(donor.email, "ada@example.org");
    assert_eq!(donor.version, initial_version);
    assert_eq!(donor.updated_at, initial_updated_at);
}

#[test]
fn update_metadata_with_email_missing_at_sign_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(&g, school, "Ada Lovelace", "ada@example.org", true, None, None);
    let initial_version = donor.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = donor.update_metadata(
        "Ada Lovelace".to_owned(),
        "no-at-sign".to_owned(),
        true,
        None,
        None,
        now,
        actor,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "new email without '@' must fail with Validation (DO I-2), got {result:?}"
    );
    assert_eq!(donor.email, "ada@example.org");
    assert_eq!(donor.version, initial_version);
}

#[test]
fn update_metadata_with_valid_inputs_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(&g, school, "Ada Lovelace", "ada@example.org", true, None, None);
    let initial_version = donor.version;
    let initial_updated_at = donor.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced = Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );
    let actor = g.next_user_id();

    donor.update_metadata(
        "Ada King, Countess of Lovelace".to_owned(),
        "ada.lovelace@example.org".to_owned(),
        false,
        Some("+44-20-7946-9999".to_owned()),
        Some("Updated profile".to_owned()),
        advanced,
        actor,
    )
    .expect("valid update");

    assert_eq!(donor.name, "Ada King, Countess of Lovelace");
    assert_eq!(donor.email, "ada.lovelace@example.org");
    assert!(!donor.show_public, "DO I-1: show_public must accept false");
    assert_eq!(donor.phone.as_deref(), Some("+44-20-7946-9999"));
    assert_eq!(donor.description.as_deref(), Some("Updated profile"));
    assert!(
        donor.version > initial_version,
        "version must advance on update (was {initial_version:?}, now {:?})",
        donor.version
    );
    assert!(
        donor.updated_at > initial_updated_at,
        "updated_at must advance on update (was {initial_updated_at:?}, now {:?})",
        donor.updated_at
    );
    assert_eq!(donor.updated_by, actor);
}

#[test]
fn update_metadata_with_empty_phone_clears_phone() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(
        &g,
        school,
        "Ada Lovelace",
        "ada@example.org",
        true,
        Some("+44-20-7946-0958"),
        None,
    );
    let actor = g.next_user_id();
    let now = SystemClock.now();
    donor.update_metadata(
        "Ada Lovelace".to_owned(),
        "ada@example.org".to_owned(),
        true,
        None,
        None,
        now,
        actor,
    )
    .expect("valid update");
    assert!(donor.phone.is_none());
}

// ---------------------------------------------------------------------------
// RealDonor: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(&g, school, "Ada Lovelace", "ada@example.org", true, None, None);
    let initial_version = donor.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(donor.is_active());
    donor.retire(now, actor).expect("first retire succeeds");
    assert!(!donor.is_active(), "retire must flip is_active to false");
    assert!(donor.version > initial_version);
    assert_eq!(donor.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut donor = make_donor(&g, school, "Ada Lovelace", "ada@example.org", true, None, None);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    donor.retire(now, actor).expect("first retire succeeds");
    let result = donor.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_donor service function
// ---------------------------------------------------------------------------

#[test]
fn create_donor_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateDonorCommand {
        tenant: tenant.clone(),
        name: "Ada Lovelace".to_owned(),
        email: "ada@example.org".to_owned(),
        show_public: true,
        phone: Some("+44-20-7946-0958".to_owned()),
        description: Some("Mathematical patron".to_owned()),
    };
    let clock = SystemClock;
    let (donor, event) = create_donor(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(donor.name, "Ada Lovelace");
    assert_eq!(donor.email, "ada@example.org");
    assert!(donor.show_public, "DO I-1: show_public is a bool");
    assert_eq!(donor.phone.as_deref(), Some("+44-20-7946-0958"));
    assert_eq!(donor.description.as_deref(), Some("Mathematical patron"));
    assert!(donor.is_active(), "service-created aggregate must be Active");
    assert_eq!(donor.school_id, tenant.school_id);
    assert_eq!(donor.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.donor_id, donor.id);
    assert_eq!(event.name, "Ada Lovelace");
    assert_eq!(event.email, "ada@example.org");
    assert!(event.show_public);
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(DonorCreated::EVENT_TYPE, "finance.donor.created");
    assert_eq!(DonorCreated::AGGREGATE_TYPE, "donor");
    assert_eq!(DonorCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_donor_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateDonorCommand {
        tenant: tenant.clone(),
        name: "Ada Lovelace".to_owned(),
        email: "not-an-email".to_owned(), // DO I-2 violation
        show_public: true,
        phone: None,
        description: None,
    };
    let clock = SystemClock;
    let result = create_donor(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "email without '@' must propagate Validation (DO I-2), got {result:?}"
    );
}
