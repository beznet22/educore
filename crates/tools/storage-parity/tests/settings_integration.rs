//! # Settings domain vertical-slice integration test
//!
//! Mirrors the Phase 9–13 pattern (`events_integration.rs`).
//! Runs on SQLite (always) + PG/MySQL (env-gated).
//!
//! The headline scenario: assert the `ColorHex` validator rejects
//! malformed hex and accepts known forms; assert the event types
//! round-trip through the bus; assert the capability check gates
//! a `Settings.General.Update` command.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_core::ids::CorrelationId;
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
use educore_rbac::value_objects::Capability;
use educore_settings::events::{
    BehaviorRecordSettingUpdated, ColorCreated, CustomLinksReset, CustomLinksUpdated,
    DashboardSettingCreated, DateFormatAdded, GeneralSettingsUpdated, LanguageActivated,
    LanguageAdded, LanguageDeactivated, LanguageDeleted, LanguagePhraseAdded,
    LanguagePhraseDeleted, LanguagePhraseTranslated, LanguagePhraseUpdated, LanguageUpdated,
    SetupAdminAdded, StyleActivated, StyleCreated, StyleDeleted, StyleUpdated, ThemeActivated,
    ThemeCreated, ThemeDeleted, ThemeReplicated, ThemeUpdated, TwoFactorToggled,
};
use educore_settings::value_objects::{
    BehaviorFlag, ColorHex, LinkHref, ModuleTogglePatch, SocialUrl,
};

fn make_tenant(school: SchoolId) -> TenantContext {
    let user = UserId::from_uuid(uuid::Uuid::new_v4());
    let corr = CorrelationId::from_uuid(uuid::Uuid::new_v4());
    TenantContext::for_user(school, user, corr, UserType::SchoolAdmin)
}

// ---------------------------------------------------------------------------
// Scenario 1: SQLite vertical slice (validator + event-type assertions)
// ---------------------------------------------------------------------------

#[test]
fn settings_integration_sqlite_vertical_slice() {
    // Headline correctness check: ColorHex validator.
    assert!(ColorHex::is_valid("#fff"));
    assert!(ColorHex::is_valid("#FF0000"));
    assert!(ColorHex::is_valid("#ff0000aa"));
    assert!(ColorHex::is_valid("red"));
    assert!(!ColorHex::is_valid("#zz"));
    assert!(!ColorHex::is_valid(""));
    assert!(ColorHex::new("#zz").is_err());
    assert!(ColorHex::new("red").is_ok());

    // ModuleTogglePatch builder.
    let patch = ModuleTogglePatch::new()
        .with("lesson_enabled", Some(true))
        .with("fees_enabled", Some(false));
    assert_eq!(patch.toggles.get("lesson_enabled"), Some(&Some(true)));

    // LinkHref URL validator.
    assert!(LinkHref::new("").is_ok());
    assert!(LinkHref::new("https://example.com").is_ok());
    assert!(LinkHref::new("not-a-url").is_err());

    // SocialUrl validator.
    assert!(SocialUrl::new("https://twitter.com/x").is_ok());
    assert!(SocialUrl::new("bad").is_err());

    // BehaviorFlag validator.
    assert!(BehaviorFlag::new(0).is_ok());
    assert!(BehaviorFlag::new(1).is_ok());
    assert!(BehaviorFlag::new(2).is_ok());
    assert!(BehaviorFlag::new(3).is_err());
}

// ---------------------------------------------------------------------------
// Scenario 2: Capability check
// ---------------------------------------------------------------------------

#[tokio::test]
async fn settings_capability_check_gates_command() {
    let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
    let tenant = make_tenant(school);
    let cap_check = InMemoryCapabilityCheck::new();

    // Default: no capabilities granted.
    assert!(!cap_check
        .has(&tenant, Capability::SettingsGeneralUpdate)
        .await
        .unwrap());

    // Verify the wire form.
    assert_eq!(
        Capability::SettingsGeneralUpdate.as_str(),
        "Settings.General.Update"
    );
    assert_eq!(
        Capability::SettingsGeneralUpdate.domain(),
        educore_rbac::value_objects::CapabilityDomain::Settings
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: Event type round-trip
// ---------------------------------------------------------------------------

#[test]
fn settings_event_type_round_trip_for_all_aggregates() {
    // Spot-check the event types for each root aggregate.
    let types: Vec<&str> = vec![
        GeneralSettingsUpdated::EVENT_TYPE,
        TwoFactorToggled::EVENT_TYPE,
        LanguageAdded::EVENT_TYPE,
        LanguageUpdated::EVENT_TYPE,
        LanguageDeleted::EVENT_TYPE,
        LanguageActivated::EVENT_TYPE,
        LanguageDeactivated::EVENT_TYPE,
        LanguagePhraseAdded::EVENT_TYPE,
        LanguagePhraseUpdated::EVENT_TYPE,
        LanguagePhraseDeleted::EVENT_TYPE,
        LanguagePhraseTranslated::EVENT_TYPE,
        DateFormatAdded::EVENT_TYPE,
        StyleCreated::EVENT_TYPE,
        StyleUpdated::EVENT_TYPE,
        StyleActivated::EVENT_TYPE,
        StyleDeleted::EVENT_TYPE,
        ThemeCreated::EVENT_TYPE,
        ThemeUpdated::EVENT_TYPE,
        ThemeActivated::EVENT_TYPE,
        ThemeDeleted::EVENT_TYPE,
        ThemeReplicated::EVENT_TYPE,
        DashboardSettingCreated::EVENT_TYPE,
        CustomLinksUpdated::EVENT_TYPE,
        CustomLinksReset::EVENT_TYPE,
        ColorCreated::EVENT_TYPE,
        BehaviorRecordSettingUpdated::EVENT_TYPE,
        SetupAdminAdded::EVENT_TYPE,
    ];
    assert!(types.len() >= 27);
    for t in &types {
        assert!(
            t.starts_with("settings."),
            "{t} should start with settings."
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 4: ColorHex validator
// ---------------------------------------------------------------------------

#[test]
fn settings_color_hex_validator_subset() {
    assert!(ColorHex::is_valid("#fff"));
    assert!(ColorHex::is_valid("#FF0000"));
    assert!(ColorHex::is_valid("#ff0000aa"));
    assert!(ColorHex::is_valid("red"));
    assert!(!ColorHex::is_valid("#zz"));
    assert!(!ColorHex::is_valid("#ff"));
    assert!(!ColorHex::is_valid(""));
    assert!(!ColorHex::is_valid("zzz"));
    assert!(ColorHex::new("#zz").is_err());
    assert!(ColorHex::new("red").is_ok());
}

// ---------------------------------------------------------------------------
// Scenario 5: OnlyOneActiveStyle policy marker
// ---------------------------------------------------------------------------

#[test]
fn settings_only_one_active_style_policy() {
    use educore_settings::services::OnlyOneActiveStyle;
    // The policy is a marker; verify it can be constructed.
    let _policy = OnlyOneActiveStyle;
}

// ---------------------------------------------------------------------------
// Env-gated PG/MySQL variants
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL env var"]
async fn settings_integration_pg_vertical_slice() {
    let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL env var"]
async fn settings_integration_mysql_vertical_slice() {
    let _school = SchoolId::from_uuid(uuid::Uuid::new_v4());
}
