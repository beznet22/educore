
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]
//! Behavioural tests for `RealDirectFeesReminder` (Wave 88).
//!
//! Covers:
//! - DFR I-1: due_date_before_days ≥ 0 (pinned + non-negative guard
//!   in `fresh()` + `update_metadata()`; returns
//!   `DomainError::Validation` if < 0)
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use chrono::NaiveDate;
use educore_academic::StudentId;
use educore_core::clock::{SystemClock, SystemIdGen};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::{DirectFeesInstallmentId, DirectFeesReminderId};

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

fn make_create_cmd(
    tenant: TenantContext,
    id: DirectFeesReminderId,
    g: &SystemIdGen,
) -> CreateDirectFeesReminderCommand {
    let school = tenant.school_id;
    let installment_id = DirectFeesInstallmentId::new(school, g.next_uuid());
    let student_id = StudentId::new(school, g.next_uuid());
    CreateDirectFeesReminderCommand {
        tenant,
        direct_fees_reminder_id: id,
        direct_fees_installment_id: installment_id,
        student_id,
        remind_at: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        due_date_before_days: 5,
        note: Some("Tuition installment reminder".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests (parallel to Wave 87 bank_account.rs pattern)
// ============================================================================

#[test]
fn typed_id_smoke_direct_fees_reminder_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_direct_fees_reminder_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = DirectFeesReminderId::new(school, g.next_uuid());
    let id_b = DirectFeesReminderId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealDirectFeesReminder::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, event) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.due_date_before_days, 5); // DFR I-1
    assert_eq!(row.remind_at, NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    assert_eq!(row.note.as_deref(), Some("Tuition installment reminder"));
    assert_eq!(event.direct_fees_reminder_id, id);
    assert_eq!(event.due_date_before_days, 5);
}

#[test]
fn fresh_zero_days_is_valid_dfr_i_1() {
    // DFR I-1 boundary: due_date_before_days == 0 is allowed (the
    // reminder fires ON the due date, not before).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.due_date_before_days = 0;
    let clock = SystemClock;

    let (row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();
    assert_eq!(row.due_date_before_days, 0);
}

#[test]
fn fresh_negative_days_validation_error_dfr_i_1() {
    // DFR I-1 guard: due_date_before_days < 0 returns Validation.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.due_date_before_days = -1;
    let clock = SystemClock;

    let err = create_direct_fees_reminder(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_audit_footer_initialized() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();
    assert_eq!(row.version, Version::initial());
    assert_eq!(row.active_status, ActiveStatus::Active);
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.created_at, row.updated_at);
    assert_eq!(row.correlation_id, tenant.correlation_id);
    assert!(row.last_event_id.is_some());
    assert_eq!(row.etag, Etag::placeholder());
}

// ============================================================================
// RealDirectFeesReminder::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_mutates_mutable_fields_only() {
    // DFR I-1: due_date_before_days is mutable (within >= 0
    // constraint); scope-key fields (direct_fees_installment_id +
    // student_id) are NOT mutable — retire + create-new required.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    // Capture scope-key fields (must be preserved after update)
    let pinned_installment = row.direct_fees_installment_id;
    let pinned_student = row.student_id;

    let update_cmd = UpdateDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
        remind_at: Some(NaiveDate::from_ymd_opt(2026, 6, 20).unwrap()),
        due_date_before_days: Some(10), // DFR I-1: 0..N valid
        note: Some("Updated reminder note".to_owned()),
    };
    let event = update_direct_fees_reminder(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable fields DID change
    assert_eq!(row.remind_at, NaiveDate::from_ymd_opt(2026, 6, 20).unwrap());
    assert_eq!(row.due_date_before_days, 10);
    assert_eq!(row.note.as_deref(), Some("Updated reminder note"));
    assert_eq!(event.due_date_before_days, 10);

    // Scope-key fields preserved
    assert_eq!(row.direct_fees_installment_id, pinned_installment);
    assert_eq!(row.student_id, pinned_student);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
    assert!(row.last_event_id.is_some());
}

#[test]
fn update_metadata_negative_days_validation_error_dfr_i_1() {
    // DFR I-1 guard: negative days rejected on update too.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    let update_cmd = UpdateDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
        remind_at: None,
        due_date_before_days: Some(-5), // DFR I-1 violation
        note: None,
    };
    let err = update_direct_fees_reminder(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn update_metadata_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    // Retire first
    let retire_cmd = DeleteDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
    };
    let _retire_event = retire_direct_fees_reminder(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Now try to update_metadata on retired row
    let update_cmd = UpdateDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
        remind_at: Some(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
        due_date_before_days: Some(7),
        note: None,
    };
    let err = update_direct_fees_reminder(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// RealDirectFeesReminder::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_scope_keys() {
    // Tombstone preserves scope-key fields
    // (direct_fees_installment_id + student_id + remind_at +
    // due_date_before_days DFR I-1) for legal-record retention.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    let pinned_installment = row.direct_fees_installment_id;
    let pinned_student = row.student_id;
    let pinned_days = row.due_date_before_days;
    let pinned_remind_at = row.remind_at;

    let retire_cmd = DeleteDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
    };
    let event = retire_direct_fees_reminder(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped to Retired
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Scope-key + mutable fields preserved (tombstone)
    assert_eq!(row.direct_fees_installment_id, pinned_installment);
    assert_eq!(row.student_id, pinned_student);
    assert_eq!(row.due_date_before_days, pinned_days);
    assert_eq!(row.remind_at, pinned_remind_at);

    // Event carries only direct_fees_reminder_id
    assert_eq!(event.direct_fees_reminder_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();

    let retire_cmd = DeleteDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
    };
    let _ = retire_direct_fees_reminder(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = DeleteDirectFeesReminderCommand {
        tenant: tenant.clone(),
        direct_fees_reminder_id: id,
    };
    let err = retire_direct_fees_reminder(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// Service integration tests (parallel to Wave 87 bank_account.rs
// service round-trip tests)
// ============================================================================

#[test]
fn create_direct_fees_reminder_service_event_type_is_finance_direct_fees_reminder_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DirectFeesReminderId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (_, event) = create_direct_fees_reminder(cmd, &clock, &g).unwrap();
    assert_eq!(
        <DirectFeesReminderCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_reminder.created"
    );
    assert_eq!(
        <DirectFeesReminderCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "direct_fees_reminder"
    );
    assert_eq!(
        <DirectFeesReminderCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.direct_fees_reminder_id, id);
}
