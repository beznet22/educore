//! Integration tests for the **DirectFeesSetting aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 69 per-aggregate drop
//! [`RealDirectFeesSetting`](educore_finance::aggregate::RealDirectFeesSetting) —
//! the per-school direct-fees programme configuration (enabled flag +
//! reminder window + installment cap + due-day-of-month). Validates
//! DFS I-1 (`reminder_before >= 0`, `no_installment >= 0`), DFS I-2
//! (`due_date_from_sem in 1..=MAX_DUE_DAY (28)`), `update_config()`
//! (version + timestamp bump), `retire()` (active → retired
//! transition), and the `create_direct_fees_setting` service function
//! (aggregate + event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `DirectFeesSetting` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! DirectFeesSetting { _id: () } }` placeholder. Wave 69 adds the
//! `RealDirectFeesSetting` aggregate, the 3 headline events, the
//! service function, and this test suite.

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

use educore_finance::commands::CreateDirectFeesSettingCommand;
use educore_finance::events::DirectFeesSettingCreated;
use educore_finance::prelude::RealDirectFeesSetting;
use educore_finance::services::create_direct_fees_setting;
use educore_finance::value_objects::DirectFeesSettingId;

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

fn direct_fees_setting_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesSettingId {
    DirectFeesSettingId::new(school, g.next_uuid())
}

fn make_direct_fees_setting(
    g: &SystemIdGen,
    school: SchoolId,
    enabled: bool,
    reminder_before: i64,
    no_installment: i64,
    due_date_from_sem: u8,
    description: Option<&str>,
) -> RealDirectFeesSetting {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealDirectFeesSetting::fresh(
        direct_fees_setting_id(g, school),
        enabled,
        reminder_before,
        no_installment,
        due_date_from_sem,
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealDirectFeesSetting: fresh() — DFS I-1 + DFS I-2 invariants
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_inputs_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, Some("Standard"));
    assert!(setting.enabled);
    assert_eq!(setting.reminder_before, 7);
    assert_eq!(setting.no_installment, 3);
    assert_eq!(setting.due_date_from_sem, 15);
    assert_eq!(setting.description.as_deref(), Some("Standard"));
    assert!(setting.is_active(), "fresh aggregate must be Active");
    assert_eq!(setting.school_id, school);
}

#[test]
fn fresh_with_disabled_and_zero_values_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    // Boundary: enabled=false, reminder_before=0, no_installment=0, due_date=1.
    let setting = make_direct_fees_setting(&g, school, false, 0, 0, 1, None);
    assert!(!setting.enabled);
    assert_eq!(setting.reminder_before, 0);
    assert_eq!(setting.no_installment, 0);
    assert_eq!(setting.due_date_from_sem, 1);
    assert!(setting.is_active());
    assert!(setting.description.is_none());
}

#[test]
fn fresh_accepts_due_date_at_max_day() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let setting = make_direct_fees_setting(&g, school, true, 7, 3, RealDirectFeesSetting::MAX_DUE_DAY, None);
    assert_eq!(
        setting.due_date_from_sem,
        RealDirectFeesSetting::MAX_DUE_DAY
    );
}

#[test]
fn fresh_with_negative_reminder_before_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDirectFeesSetting::fresh(
        direct_fees_setting_id(&g, school),
        true,
        -1,
        3,
        15,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative reminder_before must fail with Validation (DFS I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_negative_no_installment_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDirectFeesSetting::fresh(
        direct_fees_setting_id(&g, school),
        true,
        7,
        -1,
        15,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative no_installment must fail with Validation (DFS I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_due_date_zero_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDirectFeesSetting::fresh(
        direct_fees_setting_id(&g, school),
        true,
        7,
        3,
        0,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "due_date_from_sem = 0 must fail with Validation (DFS I-2), got {result:?}"
    );
}

#[test]
fn fresh_with_due_date_over_max_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDirectFeesSetting::fresh(
        direct_fees_setting_id(&g, school),
        true,
        7,
        3,
        RealDirectFeesSetting::MAX_DUE_DAY + 1,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "due_date_from_sem > MAX_DUE_DAY must fail with Validation (DFS I-2), got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealDirectFeesSetting: update_config()
// ---------------------------------------------------------------------------

#[test]
fn update_config_with_negative_reminder_before_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, None);
    let initial_version = setting.version;
    let initial_updated_at = setting.updated_at;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = setting.update_config(true, -1, 3, 15, None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative new reminder_before must fail with Validation (DFS I-1), got {result:?}"
    );
    assert_eq!(setting.reminder_before, 7);
    assert_eq!(setting.version, initial_version);
    assert_eq!(setting.updated_at, initial_updated_at);
}

#[test]
fn update_config_with_negative_no_installment_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, None);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = setting.update_config(true, 7, -1, 15, None, now, actor);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative new no_installment must fail with Validation (DFS I-1), got {result:?}"
    );
    assert_eq!(setting.no_installment, 3);
}

#[test]
fn update_config_with_due_date_over_max_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, None);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let result = setting.update_config(
        true,
        7,
        3,
        RealDirectFeesSetting::MAX_DUE_DAY + 1,
        None,
        now,
        actor,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "due_date_from_sem > MAX_DUE_DAY must fail with Validation (DFS I-2), got {result:?}"
    );
    assert_eq!(setting.due_date_from_sem, 15);
}

#[test]
fn update_config_with_valid_inputs_bumps_version_and_advances_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, Some("Initial"));
    let initial_version = setting.version;
    let initial_updated_at = setting.updated_at;

    // Advance the clock by one second past initial_updated_at.
    let advanced = Timestamp::from_datetime(
        initial_updated_at.as_datetime() + chrono::Duration::seconds(1),
    );
    let actor = g.next_user_id();

    setting
        .update_config(false, 14, 6, 28, Some("Bumped".to_owned()), advanced, actor)
        .expect("valid update");

    assert!(!setting.enabled);
    assert_eq!(setting.reminder_before, 14);
    assert_eq!(setting.no_installment, 6);
    assert_eq!(setting.due_date_from_sem, 28);
    assert_eq!(setting.description.as_deref(), Some("Bumped"));
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

#[test]
fn update_config_with_empty_description_clears_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, Some("Initial"));
    let actor = g.next_user_id();
    let now = SystemClock.now();
    setting
        .update_config(true, 7, 3, 15, None, now, actor)
        .expect("valid update");
    assert!(setting.description.is_none());
}

// ---------------------------------------------------------------------------
// RealDirectFeesSetting: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, None);
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
    let mut setting = make_direct_fees_setting(&g, school, true, 7, 3, 15, None);
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
// create_direct_fees_setting service function
// ---------------------------------------------------------------------------

#[test]
fn create_direct_fees_setting_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let cmd = CreateDirectFeesSettingCommand {
        tenant: tenant.clone(),
        enabled: true,
        reminder_before: 7,
        no_installment: 3,
        due_date_from_sem: 15,
        description: Some("Standard".to_owned()),
    };
    let clock = SystemClock;
    let (setting, event) =
        create_direct_fees_setting(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert!(setting.enabled);
    assert_eq!(setting.reminder_before, 7);
    assert_eq!(setting.no_installment, 3);
    assert_eq!(setting.due_date_from_sem, 15);
    assert_eq!(setting.description.as_deref(), Some("Standard"));
    assert!(setting.is_active(), "service-created aggregate must be Active");
    assert_eq!(setting.school_id, tenant.school_id);
    assert_eq!(setting.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.direct_fees_setting_id, setting.id);
    assert!(event.enabled);
    assert_eq!(event.reminder_before, 7);
    assert_eq!(event.no_installment, 3);
    assert_eq!(event.due_date_from_sem, 15);
    assert_eq!(event.description.as_deref(), Some("Standard"));
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        DirectFeesSettingCreated::EVENT_TYPE,
        "finance.direct_fees_setting.created"
    );
    assert_eq!(
        DirectFeesSettingCreated::AGGREGATE_TYPE,
        "direct_fees_setting"
    );
    assert_eq!(DirectFeesSettingCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_direct_fees_setting_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let cmd = CreateDirectFeesSettingCommand {
        tenant: tenant.clone(),
        enabled: true,
        reminder_before: -1, // DFS I-1 violation
        no_installment: 3,
        due_date_from_sem: 15,
        description: None,
    };
    let clock = SystemClock;
    let result = create_direct_fees_setting(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative reminder_before must propagate Validation (DFS I-1), got {result:?}"
    );
}
