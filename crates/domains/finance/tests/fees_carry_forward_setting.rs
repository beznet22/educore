//! Integration tests for the **FeesCarryForwardSetting aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 78 per-aggregate drop
//! [`RealFeesCarryForwardSetting`](educore_finance::aggregate::RealFeesCarryForwardSetting) —
//! the per-school configuration for the fees-carry-forward feature.
//! Validates FCFA I-1 (per-school config; the typed id carries the
//! school_id, so the aggregate is inherently school-scoped; uniqueness
//! across schools is meaningless because the aggregate is keyed by
//! `(school_id, uuid)`, and one-per-school is a dispatcher concern),
//! FCFA I-2 (`threshold_minor` must be ≥ 0), `update_metadata()`
//! (threshold / enabled / description), `retire()` (active → retired
//! transition), and the `create_fees_carry_forward_setting` service
//! function (aggregate + event pairing with cross-school
//! defense-in-depth).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `FeesCarryForwardSetting` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! FeesCarryForwardSetting { _id: () } }` placeholder. Wave 78 adds
//! the `RealFeesCarryForwardSetting` aggregate, the 3 headline events
//! (Created / Updated / Retired — the setting is reference data so
//! updates are expected, not append-only), the service function, and
//! this test suite.

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

use educore_finance::commands::CreateFeesCarryForwardSettingCommand;
use educore_finance::events::FeesCarryForwardSettingCreated;
use educore_finance::prelude::RealFeesCarryForwardSetting;
use educore_finance::services::create_fees_carry_forward_setting;
use educore_finance::value_objects::FeesCarryForwardSettingId;

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

fn fees_carry_forward_setting_id(g: &SystemIdGen, school: SchoolId) -> FeesCarryForwardSettingId {
    FeesCarryForwardSettingId::new(school, g.next_uuid())
}

fn make_fees_carry_forward_setting(
    g: &SystemIdGen,
    school: SchoolId,
    threshold_minor: i64,
    enabled: bool,
    description: Option<&str>,
) -> RealFeesCarryForwardSetting {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealFeesCarryForwardSetting::fresh(
        fees_carry_forward_setting_id(g, school),
        threshold_minor,
        enabled,
        description.map(|s| s.to_owned()),
        actor,
        now,
        corr,
    )
    .expect("fresh RealFeesCarryForwardSetting")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 78 stub tests)
// =========================================================================

#[test]
fn fees_carry_forward_setting_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_carry_forward_setting_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_carry_forward_setting_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fees_carry_forward_setting_id(&g, school);
    let id_b = fees_carry_forward_setting_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealFeesCarryForwardSetting::fresh — FCFA I-1 + FCFA I-2
// =========================================================================

#[test]
fn fresh_zero_threshold_is_valid() {
    // FCFA I-2 lower bound is 0, not > 0; a threshold of 0 means
    // "carry forward everything above zero".
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_fees_carry_forward_setting(&g, school, 0, true, None);
    assert_eq!(row.threshold_minor, 0);
    assert!(row.enabled);
    assert!(row.is_active());
    assert_eq!(row.school_id, school);
}

#[test]
fn fresh_positive_threshold_is_valid() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_fees_carry_forward_setting(&g, school, 5_000, true, Some("above 50.00"));
    assert_eq!(row.threshold_minor, 5_000);
    assert_eq!(row.description.as_deref(), Some("above 50.00"));
}

#[test]
fn fresh_disabled_flag_is_preserved() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_fees_carry_forward_setting(&g, school, 1_000, false, None);
    assert!(!row.enabled);
}

#[test]
fn fresh_negative_threshold_returns_validation() {
    // FCFA I-2: threshold must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_carry_forward_setting_id(&g, school);
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let err = RealFeesCarryForwardSetting::fresh(id, -1, true, None, actor, now, corr)
        .expect_err("negative threshold must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_trims_description_and_drops_empty() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_fees_carry_forward_setting(&g, school, 100, true, Some("  pad me  "));
    assert_eq!(row.description.as_deref(), Some("pad me"));
    let row2 = make_fees_carry_forward_setting(&g, school, 100, true, Some("   "));
    assert_eq!(row2.description, None);
}

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let before = SystemClock.now();
    let row = make_fees_carry_forward_setting(&g, school, 100, true, None);
    let after = SystemClock.now();
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
    assert_eq!(row.created_by, row.updated_by);
    assert!(row.last_event_id.is_none());
    assert!(row.is_active());
}

// =========================================================================
// RealFeesCarryForwardSetting::update_metadata
// =========================================================================

#[test]
fn update_metadata_updates_threshold_enabled_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_carry_forward_setting(&g, school, 100, true, Some("initial"));
    let original_version = row.version;
    let later = SystemClock.now();
    row.update_metadata(
        500,
        false,
        Some("revised".to_owned()),
        later,
        g.next_user_id(),
    )
    .expect("update");
    assert_eq!(row.threshold_minor, 500);
    assert!(!row.enabled);
    assert_eq!(row.description.as_deref(), Some("revised"));
    assert_eq!(row.updated_at, later);
    assert!(row.version > original_version);
}

#[test]
fn update_metadata_validates_negative_threshold() {
    // FCFA I-2 on update.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_carry_forward_setting(&g, school, 100, true, None);
    let now = SystemClock.now();
    let err = row
        .update_metadata(-10, true, None, now, g.next_user_id())
        .expect_err("negative threshold on update must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    // The original threshold is preserved on validation failure.
    assert_eq!(row.threshold_minor, 100);
}

#[test]
fn update_metadata_rejects_on_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_carry_forward_setting(&g, school, 100, true, None);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    let later = SystemClock.now();
    let err = row
        .update_metadata(200, false, None, later, g.next_user_id())
        .expect_err("update on retired must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// RealFeesCarryForwardSetting::retire
// =========================================================================

#[test]
fn retire_flips_active_status_and_preserves_threshold() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_carry_forward_setting(&g, school, 100, true, Some("keep me"));
    let before = row.version;
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    assert!(!row.is_active());
    assert_eq!(row.threshold_minor, 100);
    assert_eq!(row.description.as_deref(), Some("keep me"));
    assert_eq!(row.updated_at, now);
    assert!(row.version > before);
}

#[test]
fn retire_rejects_double_retire() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_carry_forward_setting(&g, school, 100, true, None);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("first retire");
    let err = row
        .retire(now, g.next_user_id())
        .expect_err("double retire must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// create_fees_carry_forward_setting service function
// =========================================================================

#[test]
fn service_function_creates_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_carry_forward_setting_id(&g, school);
    let cmd = CreateFeesCarryForwardSettingCommand {
        tenant: tenant.clone(),
        fees_carry_forward_setting_id: id,
        threshold_minor: 250,
        enabled: true,
        description: Some("above 2.50".to_owned()),
    };
    let clock = SystemClock;
    let (row, event) = create_fees_carry_forward_setting(cmd, &clock, &g)
        .expect("service function should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.threshold_minor, 250);
    assert!(row.enabled);
    assert_eq!(row.description.as_deref(), Some("above 2.50"));
    assert_eq!(event.fees_carry_forward_setting_id, id);
    assert_eq!(event.threshold_minor, 250);
    assert_eq!(event.enabled, true);
    assert_eq!(
        <FeesCarryForwardSettingCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_carry_forward_setting.created"
    );
    assert_eq!(
        <FeesCarryForwardSettingCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_carry_forward_setting"
    );
    assert_eq!(
        <FeesCarryForwardSettingCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id(), id.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn service_function_propagates_negative_threshold_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_carry_forward_setting_id(&g, school);
    let cmd = CreateFeesCarryForwardSettingCommand {
        tenant: tenant.clone(),
        fees_carry_forward_setting_id: id,
        threshold_minor: -1,
        enabled: true,
        description: None,
    };
    let clock = SystemClock;
    let err = create_fees_carry_forward_setting(cmd, &clock, &g)
        .expect_err("negative threshold must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn service_function_rejects_cross_school_id() {
    // FCFA I-1 cross-school defense-in-depth: the supplied id must
    // belong to the tenant's school.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    // Mint an id belonging to a DIFFERENT school.
    let other_school = g.next_school_id();
    assert_ne!(other_school, school);
    let wrong_id = fees_carry_forward_setting_id(&g, other_school);
    let cmd = CreateFeesCarryForwardSettingCommand {
        tenant: tenant.clone(),
        fees_carry_forward_setting_id: wrong_id,
        threshold_minor: 100,
        enabled: true,
        description: None,
    };
    let clock = SystemClock;
    let err = create_fees_carry_forward_setting(cmd, &clock, &g)
        .expect_err("cross-school id must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
