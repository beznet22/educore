//! Behavioural tests for `RealFmFeesInvoiceSetting` (Wave 94).
//!
//! Pins FFIS I-1 (per_th >= 0), FFIS I-2 (due_date_offset_days >= 0),
//! FFIS I-3 (prefix alphanumeric-only + non-empty trimmed + NOT
//! mutable) end-to-end via the aggregate surface, the service
//! functions, and the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::events::{
    FmFeesInvoiceSettingCreated, FmFeesInvoiceSettingRetired, FmFeesInvoiceSettingUpdated,
};
use educore_finance::prelude::*;
use educore_finance::value_objects::FmFeesInvoiceSettingId;

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

fn fm_fees_invoice_setting_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceSettingId {
    FmFeesInvoiceSettingId::new(school, g.next_uuid())
}

fn sample_due_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 12, 31).expect("valid date")
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_invoice_setting_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fm_fees_invoice_setting_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFIS I-1: per_th >= 0 ----

#[test]
fn fresh_full_payload_fic_per_th_valid_fic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with per_th = 2500");
    assert_eq!(row.per_th, 2500);
    assert_eq!(row.prefix, "INV");
    assert_eq!(row.due_date_offset_days, 30);
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_per_th_validation_error_fic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let result = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        -1,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("per_th") && msg.contains("FFIS I-1"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_per_th_is_valid_fic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        0,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with per_th = 0 (boundary, valid)");
    assert_eq!(row.per_th, 0);
}

// ---- FFIS I-2: due_date_offset_days >= 0 ----

#[test]
fn fresh_negative_due_date_offset_validation_error_fic_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let result = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        -1,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("due_date_offset_days") && msg.contains("FFIS I-2"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_due_date_offset_is_valid_fic_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with due_date_offset_days = 0 (boundary, valid)");
    assert_eq!(row.due_date_offset_days, 0);
}

// ---- FFIS I-3: prefix alphanumeric-only + non-empty trimmed + NOT mutable ----

#[test]
fn fresh_empty_prefix_validation_error_fic_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let result = RealFmFeesInvoiceSetting::fresh(
        id,
        "   ".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("prefix") && msg.contains("FFIS I-3"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_non_alphanumeric_prefix_validation_error_fic_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let result = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV-2024".to_string(), // hyphen is not alphanumeric
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("alphanumeric") && msg.contains("FFIS I-3"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn prefix_is_trimmed_on_fresh_fic_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let row = RealFmFeesInvoiceSetting::fresh(
        id,
        "  INV  ".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim prefix");
    assert_eq!(row.prefix, "INV");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
}

// ---- update + retire ----

#[test]
fn update_metadata_mutates_per_th_and_due_date_offset_fic_i_1_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let mut row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    let new_due = NaiveDate::from_ymd_opt(2027, 1, 31).expect("valid date");
    row.update_metadata(
        5000,
        new_due,
        45,
        Timestamp::now(),
        tenant.actor_id,
    )
    .expect("update_metadata should succeed");
    assert_eq!(row.per_th, 5000);
    assert_eq!(row.due_date, new_due);
    assert_eq!(row.due_date_offset_days, 45);
    assert_eq!(row.prefix, "INV"); // prefix unchanged \u2014 FFIS I-3 immutability
    assert!(row.version > Version::initial());
}

#[test]
fn update_metadata_re_validates_fic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let mut row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    let result = row.update_metadata(
        -1,
        sample_due_date(),
        30,
        Timestamp::now(),
        tenant.actor_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let mut row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.prefix, "INV");
    assert_eq!(row.per_th, 2500);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_setting_id(&g, school);
    let mut row = RealFmFeesInvoiceSetting::fresh(
        id,
        "INV".to_string(),
        2500,
        sample_due_date(),
        30,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("first retire should succeed");
    let result = row.retire(Timestamp::now(), tenant.actor_id);
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- service integration ----

#[test]
fn create_fm_fees_invoice_setting_service_emits_created_event_fic_i_1_i_2_i_3() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_setting_id(&g, school);
    let cmd = CreateFmFeesInvoiceSettingCommand {
        tenant,
        fm_fees_invoice_setting_id: id,
        prefix: "INV".to_string(),
        per_th: 2500,
        due_date: sample_due_date(),
        due_date_offset_days: 30,
    };
    let evt: FmFeesInvoiceSettingCreated =
        create_fm_fees_invoice_setting(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.prefix, "INV");
    assert_eq!(evt.per_th, 2500);
    assert_eq!(evt.due_date_offset_days, 30);
    assert_eq!(evt.fm_fees_invoice_setting_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <FmFeesInvoiceSettingCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_setting.created"
    );
    assert_eq!(
        <FmFeesInvoiceSettingCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_invoice_setting"
    );
    assert_eq!(
        <FmFeesInvoiceSettingCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn update_fm_fees_invoice_setting_service_emits_updated_event_fic_i_1_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_setting_id(&g, school);
    let cmd = UpdateFmFeesInvoiceSettingCommand {
        tenant,
        fm_fees_invoice_setting_id: id,
        per_th: 5000,
        due_date: sample_due_date(),
        due_date_offset_days: 45,
    };
    let evt: FmFeesInvoiceSettingUpdated =
        update_fm_fees_invoice_setting(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.per_th, 5000);
    assert_eq!(evt.due_date_offset_days, 45);
    assert_eq!(
        <FmFeesInvoiceSettingUpdated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_setting.updated"
    );
}

#[test]
fn retire_fm_fees_invoice_setting_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_setting_id(&g, school);
    let cmd = RetireFmFeesInvoiceSettingCommand {
        tenant,
        fm_fees_invoice_setting_id: id,
    };
    let evt: FmFeesInvoiceSettingRetired =
        retire_fm_fees_invoice_setting(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.fm_fees_invoice_setting_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesInvoiceSettingRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_setting.retired"
    );
}
