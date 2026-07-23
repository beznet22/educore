//! Integration tests for the **ChartOfAccount aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 74 per-aggregate drop
//! [`RealChartOfAccount`](educore_finance::aggregate::RealChartOfAccount)
//! — the foundational double-entry bookkeeping aggregate (every
//! ledger entry references a `ChartOfAccount` by id). Validates:
//!
//! - COA I-1: shape validation that per-school uniqueness will key on
//!   (name 1..=100 chars, code matches `[A-Z0-9-]{1,20}`)
//! - COA I-2: retire() tombstone (per-school reference integrity is
//!   the dispatcher's concern; this drop pins the lifecycle that the
//!   reference check will gate on)
//! - `update_metadata()` (version + timestamp bump + re-validation)
//! - `retire()` (active → retired transition, conflict on double-retire)
//! - `create_chart_of_account` service function (aggregate + event
//!   pairing, tenant scope defense-in-depth)
//!
//! The pre-existing 2 typed-id-only tests have been preserved (as
//! smoke tests for the typed-id contract) and the suite is extended
//! below with behavioral tests covering the Wave 74 full drop.

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

use educore_finance::commands::CreateChartOfAccountCommand;
use educore_finance::events::ChartOfAccountCreated;
use educore_finance::prelude::RealChartOfAccount;
use educore_finance::services::create_chart_of_account;
use educore_finance::value_objects::{AccountType, ChartOfAccountId};

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

fn chart_of_account_id(g: &SystemIdGen, school: SchoolId) -> ChartOfAccountId {
    ChartOfAccountId::new(school, g.next_uuid())
}

fn make_account(
    g: &SystemIdGen,
    school: SchoolId,
    code: &str,
    name: &str,
) -> RealChartOfAccount {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealChartOfAccount::fresh(
        chart_of_account_id(g, school),
        code.to_owned(),
        name.to_owned(),
        AccountType::Bank,
        Some("Petty cash drawer".to_owned()),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// Typed-id contract (preserved from Phase 7 Workstream D seed)
// ---------------------------------------------------------------------------

#[test]
fn chart_of_account_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = chart_of_account_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn chart_of_account_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = chart_of_account_id(&g, school);
    let id_b = chart_of_account_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// RealChartOfAccount: fresh() — COA I-1 shape validation
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_code_and_name_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let account = make_account(&g, school, "1000-CASH", "Cash");
    assert_eq!(account.code, "1000-CASH");
    assert_eq!(account.name, "Cash");
    assert_eq!(account.account_type, AccountType::Bank);
    assert!(account.is_active(), "fresh aggregate must be Active");
    assert_eq!(account.school_id, school);
}

#[test]
fn fresh_trims_whitespace_in_code_and_name() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let account = make_account(&g, school, "  1000-CASH  ", "  Cash  ");
    assert_eq!(account.code, "1000-CASH");
    assert_eq!(account.name, "Cash");
}

#[test]
fn fresh_with_empty_code_returns_validation_error() {
    // COA I-1 shape: code is required.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealChartOfAccount::fresh(
        chart_of_account_id(&g, school),
        String::new(),
        "Cash".to_owned(),
        AccountType::Bank,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty code must fail with Validation (COA I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_lowercase_code_returns_validation_error() {
    // COA I-1 shape: code must be uppercase alphanumeric + dashes.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealChartOfAccount::fresh(
        chart_of_account_id(&g, school),
        "1000-cash".to_owned(), // lowercase not allowed
        "Cash".to_owned(),
        AccountType::Bank,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "lowercase code must fail with Validation (COA I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_code_over_20_chars_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let overlong = "A".repeat(21);
    let result = RealChartOfAccount::fresh(
        chart_of_account_id(&g, school),
        overlong,
        "Cash".to_owned(),
        AccountType::Bank,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "code over 20 chars must fail with Validation (COA I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealChartOfAccount::fresh(
        chart_of_account_id(&g, school),
        "1000-CASH".to_owned(),
        String::new(),
        AccountType::Bank,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty name must fail with Validation (COA I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_name_over_100_chars_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let overlong = "n".repeat(101);
    let result = RealChartOfAccount::fresh(
        chart_of_account_id(&g, school),
        "1000-CASH".to_owned(),
        overlong,
        AccountType::Bank,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "name over 100 chars must fail with Validation (COA I-1), got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealChartOfAccount: update_metadata()
// ---------------------------------------------------------------------------

#[test]
fn update_metadata_with_valid_inputs_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut account = make_account(&g, school, "1000-CASH", "Cash");
    let initial_version = account.version;
    let initial_updated_at = account.updated_at;
    let actor = g.next_user_id();
    let advanced = educore_core::value_objects::Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );

    account
        .update_metadata(
            "1100-CASH-EQUIV".to_owned(),
            "Cash Equivalents".to_owned(),
            AccountType::Cash,
            Some("Short-term investments".to_owned()),
            advanced,
            actor,
        )
        .expect("valid update");

    assert_eq!(account.code, "1100-CASH-EQUIV");
    assert_eq!(account.name, "Cash Equivalents");
    assert_eq!(account.account_type, AccountType::Cash);
    assert_eq!(account.description.as_deref(), Some("Short-term investments"));
    assert!(account.version > initial_version);
    assert!(account.updated_at > initial_updated_at);
    assert_eq!(account.updated_by, actor);
}

#[test]
fn update_metadata_with_invalid_code_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut account = make_account(&g, school, "1000-CASH", "Cash");
    let initial_code = account.code.clone();
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = account.update_metadata(
        "1000-cash".to_owned(), // lowercase not allowed
        "Cash".to_owned(),
        AccountType::Bank,
        None,
        now,
        actor,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "lowercase code on update must fail with Validation (COA I-1), got {result:?}"
    );
    assert_eq!(account.code, initial_code, "code must be unchanged on validation failure");
}

#[test]
fn update_metadata_on_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut account = make_account(&g, school, "1000-CASH", "Cash");
    let actor = g.next_user_id();
    let now = SystemClock.now();
    account.retire(now, actor).expect("first retire succeeds");
    let later = educore_core::value_objects::Timestamp::from_datetime(
        now.as_datetime() + chrono::Duration::seconds(1),
    );
    let result = account.update_metadata(
        "1000-CASH".to_owned(),
        "Cash".to_owned(),
        AccountType::Bank,
        None,
        later,
        actor,
    );
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "update on retired must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealChartOfAccount: retire() — COA I-2 lifecycle
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut account = make_account(&g, school, "1000-CASH", "Cash");
    let initial_version = account.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(account.is_active());
    account.retire(now, actor).expect("first retire succeeds");
    assert!(!account.is_active(), "retire must flip is_active to false");
    assert!(account.version > initial_version);
    assert_eq!(account.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut account = make_account(&g, school, "1000-CASH", "Cash");
    let actor = g.next_user_id();
    let now = SystemClock.now();

    account.retire(now, actor).expect("first retire succeeds");
    let result = account.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_chart_of_account service function
// ---------------------------------------------------------------------------

#[test]
fn create_chart_of_account_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let coa_id = chart_of_account_id(&g, school);
    let cmd = CreateChartOfAccountCommand {
        tenant: tenant.clone(),
        chart_of_account_id: coa_id,
        code: "1000-CASH".to_owned(),
        name: "Cash".to_owned(),
        account_type: AccountType::Bank,
        description: Some("Petty cash drawer".to_owned()),
    };
    let clock = SystemClock;
    let (account, event) =
        create_chart_of_account(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(account.code, "1000-CASH");
    assert_eq!(account.name, "Cash");
    assert_eq!(account.account_type, AccountType::Bank);
    assert!(account.is_active(), "service-created aggregate must be Active");
    assert_eq!(account.school_id, school);
    assert_eq!(account.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.chart_of_account_id, account.id);
    assert_eq!(event.code, "1000-CASH");
    assert_eq!(event.name, "Cash");
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(ChartOfAccountCreated::EVENT_TYPE, "finance.chart_of_account.created");
    assert_eq!(ChartOfAccountCreated::AGGREGATE_TYPE, "chart_of_account");
    assert_eq!(ChartOfAccountCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_chart_of_account_service_rejects_cross_school_id() {
    // Defense-in-depth: the service re-validates that the supplied
    // chart_of_account_id belongs to the tenant's school.
    let (tenant, g) = admin_context();
    let other_school = g.next_school_id();
    let coa_id_other_school = chart_of_account_id(&g, other_school);
    let cmd = CreateChartOfAccountCommand {
        tenant: tenant.clone(),
        chart_of_account_id: coa_id_other_school,
        code: "1000-CASH".to_owned(),
        name: "Cash".to_owned(),
        account_type: AccountType::Bank,
        description: None,
    };
    let clock = SystemClock;
    let result = create_chart_of_account(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "cross-school id must fail with Validation, got {result:?}"
    );
}

#[test]
fn create_chart_of_account_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let cmd = CreateChartOfAccountCommand {
        tenant: tenant.clone(),
        chart_of_account_id: chart_of_account_id(&g, school),
        code: "1000-cash".to_owned(), // lowercase not allowed (COA I-1)
        name: "Cash".to_owned(),
        account_type: AccountType::Bank,
        description: None,
    };
    let clock = SystemClock;
    let result = create_chart_of_account(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "lowercase code must propagate Validation (COA I-1), got {result:?}"
    );
}
