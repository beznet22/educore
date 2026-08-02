//! Integration tests for the **InvoiceSetting aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 67 per-aggregate drop
//! [`RealInvoiceSetting`](educore_finance::aggregate::RealInvoiceSetting) —
//! the school's invoice-numbering configuration (prefix + start_form),
//! per v3 Part 2 F54. Validates ISv I-1 (prefix must be 1..=10 chars
//! after trim), `update_config()` (version + timestamp bump),
//! `retire()` (active → retired transition), and the
//! `create_invoice_setting` service function (aggregate + event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `InvoiceSetting` previously had no real implementation
//! beyond a `finance_aggregate_stub! { struct InvoiceSetting { _id: () } }`
//! placeholder. Wave 67 adds the `RealInvoiceSetting` aggregate, the
//! 3 headline events, the service function, and this test suite.

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

use educore_finance::commands::CreateInvoiceSettingCommand;
use educore_finance::events::InvoiceSettingCreated;
use educore_finance::prelude::RealInvoiceSetting;
use educore_finance::services::create_invoice_setting;
use educore_finance::value_objects::InvoiceSettingId;

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

fn invoice_setting_id(g: &SystemIdGen, school: SchoolId) -> InvoiceSettingId {
    InvoiceSettingId::new(school, g.next_uuid())
}

fn make_invoice_setting(
    g: &SystemIdGen,
    school: SchoolId,
    prefix: &str,
    start_form: i64,
) -> RealInvoiceSetting {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealInvoiceSetting::fresh(
        invoice_setting_id(g, school),
        prefix.to_owned(),
        start_form,
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealInvoiceSetting: fresh() — ISv I-1 invariant
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_prefix_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let setting = make_invoice_setting(&g, school, "INV-", 1000);
    assert_eq!(setting.prefix, "INV-");
    assert_eq!(setting.start_form, 1000);
    assert!(setting.is_active(), "fresh aggregate must be Active");
    assert_eq!(setting.school_id, school);
}

#[test]
fn fresh_trims_whitespace_in_prefix() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let setting = make_invoice_setting(&g, school, "  INV-  ", 1000);
    assert_eq!(setting.prefix, "INV-");
}

#[test]
fn fresh_accepts_prefix_at_max_length() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let max = "A".repeat(RealInvoiceSetting::MAX_PREFIX_LEN);
    let setting = make_invoice_setting(&g, school, &max, 0);
    assert_eq!(
        setting.prefix.chars().count(),
        RealInvoiceSetting::MAX_PREFIX_LEN
    );
}

#[test]
fn fresh_with_empty_prefix_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealInvoiceSetting::fresh(
        invoice_setting_id(&g, school),
        String::new(),
        0,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty prefix must fail with Validation, got {result:?}"
    );
}

#[test]
fn fresh_with_whitespace_only_prefix_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealInvoiceSetting::fresh(
        invoice_setting_id(&g, school),
        "   \t\n  ".to_owned(),
        0,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only prefix must fail with Validation, got {result:?}"
    );
}

#[test]
fn fresh_with_prefix_over_max_length_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let too_long = "A".repeat(RealInvoiceSetting::MAX_PREFIX_LEN + 1);
    let result = RealInvoiceSetting::fresh(
        invoice_setting_id(&g, school),
        too_long,
        0,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "prefix over MAX_PREFIX_LEN must fail with Validation, got {result:?}"
    );
}

#[test]
fn fresh_with_negative_start_form_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealInvoiceSetting::fresh(
        invoice_setting_id(&g, school),
        "INV-".to_owned(),
        -1,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative start_form must fail with Validation, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealInvoiceSetting: update_config()
// ---------------------------------------------------------------------------

#[test]
fn update_config_with_empty_prefix_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let initial_version = setting.version;
    let initial_updated_at = setting.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = setting.update_config(String::new(), 2000, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty new prefix must fail with Validation, got {result:?}"
    );
    // Original state preserved on failed update.
    assert_eq!(setting.prefix, "INV-");
    assert_eq!(setting.start_form, 1000);
    assert_eq!(setting.version, initial_version);
    assert_eq!(setting.updated_at, initial_updated_at);
}

#[test]
fn update_config_with_prefix_over_max_length_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let initial_version = setting.version;
    let initial_updated_at = setting.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let too_long = "A".repeat(RealInvoiceSetting::MAX_PREFIX_LEN + 1);
    let result = setting.update_config(too_long, 2000, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "prefix over MAX_PREFIX_LEN must fail with Validation, got {result:?}"
    );
    assert_eq!(setting.prefix, "INV-");
    assert_eq!(setting.version, initial_version);
    assert_eq!(setting.updated_at, initial_updated_at);
}

#[test]
fn update_config_with_negative_start_form_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let initial_version = setting.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = setting.update_config("INV-".to_owned(), -1, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative new start_form must fail with Validation, got {result:?}"
    );
    assert_eq!(setting.start_form, 1000);
    assert_eq!(setting.version, initial_version);
}

#[test]
fn update_config_with_valid_prefix_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let initial_version = setting.version;
    let initial_updated_at = setting.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced =
        Timestamp::from_datetime(initial_updated_at.as_datetime() + chrono::Duration::seconds(1));
    let actor = g.next_user_id();

    setting
        .update_config("RCPT-".to_owned(), 2000, advanced, actor)
        .expect("valid update");

    assert_eq!(setting.prefix, "RCPT-");
    assert_eq!(setting.start_form, 2000);
    assert!(
        setting.version > initial_version,
        "version must advance on update (was {initial_version:?}, now {:?})",
        setting.version
    );
    assert!(
        setting.updated_at > initial_updated_at,
        "updated_at must advance on update (was {initial_updated_at:?}, now {:?})",
        setting.updated_at
    );
    assert_eq!(setting.updated_by, actor);
}

// ---------------------------------------------------------------------------
// RealInvoiceSetting: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let initial_version = setting.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(setting.is_active());
    setting.retire(now, actor).expect("first retire succeeds");
    assert!(!setting.is_active(), "retire must flip is_active to false");
    assert!(setting.version > initial_version);
    assert_eq!(setting.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_invoice_setting(&g, school, "INV-", 1000);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    setting.retire(now, actor).expect("first retire succeeds");
    let result = setting.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_invoice_setting service function
// ---------------------------------------------------------------------------

#[test]
fn create_invoice_setting_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateInvoiceSettingCommand {
        tenant: tenant.clone(),
        prefix: "INV-".to_owned(),
        start_form: 1000,
    };
    let clock = SystemClock;
    let (setting, event) = create_invoice_setting(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(setting.prefix, "INV-");
    assert_eq!(setting.start_form, 1000);
    assert!(
        setting.is_active(),
        "service-created aggregate must be Active"
    );
    assert_eq!(setting.school_id, tenant.school_id);
    assert_eq!(setting.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.invoice_setting_id, setting.id);
    assert_eq!(event.prefix, "INV-");
    assert_eq!(event.start_form, 1000);
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        InvoiceSettingCreated::EVENT_TYPE,
        "finance.invoice_setting.created"
    );
    assert_eq!(InvoiceSettingCreated::AGGREGATE_TYPE, "invoice_setting");
    assert_eq!(InvoiceSettingCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_invoice_setting_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateInvoiceSettingCommand {
        tenant: tenant.clone(),
        prefix: "   ".to_owned(),
        start_form: 1000,
    };
    let clock = SystemClock;
    let result = create_invoice_setting(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only prefix must propagate Validation, got {result:?}"
    );
}
