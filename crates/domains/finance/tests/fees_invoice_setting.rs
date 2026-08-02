//! Behavioural tests for `RealFeesInvoiceSetting` (Wave 92).
//!
//! Covers:
//! - FISv I-1: prefix format valid \xe2\x80\x94 `prefix` must be
//!   non-empty after trim AND alphanumeric only (no
//!   whitespace, no special chars). Pinned at construction
//!   (NOT mutable via update_metadata \xe2\x80\x94 changing the
//!   invoice prefix after invoices have been issued would break
//!   the audit trail).
//! - FISv I-2: per_th \xe2\x89\xa5 0 \xe2\x80\x94 `per_th` (per-thousand
//!   threshold) must be >= 0 at construction + update_metadata.
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::FeesInvoiceSettingId;

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
    id: FeesInvoiceSettingId,
) -> CreateFeesInvoiceSettingCommand {
    CreateFeesInvoiceSettingCommand {
        tenant,
        fees_invoice_setting_id: id,
        prefix: "INV".to_owned(), // FISv I-1 pinned \xe2\x80\x94 alphanumeric
        per_th: 500,              // FISv I-2 \xe2\x80\x94 50.0%
        description: Some("Standard invoice numbering + late fee threshold".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests
// ============================================================================

#[test]
fn typed_id_smoke_fees_invoice_setting_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_fees_invoice_setting_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = FeesInvoiceSettingId::new(school, g.next_uuid());
    let id_b = FeesInvoiceSettingId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealFeesInvoiceSetting::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, event) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.prefix, "INV"); // FISv I-1 pinned
    assert_eq!(row.per_th, 500); // FISv I-2
    assert_eq!(
        row.description.as_deref(),
        Some("Standard invoice numbering + late fee threshold")
    );
    assert_eq!(event.fees_invoice_setting_id, id);
    assert_eq!(event.prefix, "INV"); // FISv I-1 carried downstream
    assert_eq!(event.per_th, 500); // FISv I-2 carried downstream
}

#[test]
fn fresh_empty_prefix_validation_error_fisv_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.prefix = "   ".to_owned(); // trims to empty
    let clock = SystemClock;

    let err = create_fees_invoice_setting(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_non_alphanumeric_prefix_validation_error_fisv_i_1() {
    // FISv I-1: prefix must be alphanumeric only (no whitespace,
    // no special chars).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.prefix = "INV-2024".to_owned(); // contains hyphen (not alphanumeric)
    let clock = SystemClock;

    let err = create_fees_invoice_setting(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_prefix_with_whitespace_validation_error_fisv_i_1() {
    // FISv I-1: prefix must NOT contain whitespace.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.prefix = "IN V".to_owned(); // contains internal space
    let clock = SystemClock;

    let err = create_fees_invoice_setting(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_negative_per_th_validation_error_fisv_i_2() {
    // FISv I-2: per_th must be >= 0 (negative values are
    // nonsensical for a percentage threshold).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.per_th = -1;
    let clock = SystemClock;

    let err = create_fees_invoice_setting(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_zero_per_th_is_valid_fisv_i_2() {
    // FISv I-2 boundary: per_th == 0 is allowed (means "always
    // trigger late fee"; not strictly positive).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.per_th = 0;
    let clock = SystemClock;

    let (row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();
    assert_eq!(row.per_th, 0);
}

#[test]
fn fresh_audit_footer_initialized() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();
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
// RealFeesInvoiceSetting::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_changes_per_th_preserves_prefix_fisv_i_1() {
    // FISv I-1: prefix is NOT mutable via update_metadata;
    // changing the invoice prefix after invoices have been
    // issued would break the audit trail.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    let pinned_prefix = row.prefix.clone();

    let update_cmd = UpdateFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
        per_th: 750, // FISv I-2 mutable
        description: Some("Updated: 75% threshold".to_owned()),
    };
    let event = update_fees_invoice_setting(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable fields DID change
    assert_eq!(row.per_th, 750);
    assert_eq!(row.description.as_deref(), Some("Updated: 75% threshold"));
    assert_eq!(event.per_th, 750);

    // FISv I-1: prefix preserved (not mutable)
    assert_eq!(row.prefix, pinned_prefix);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
    assert!(row.last_event_id.is_some());
}

#[test]
fn update_metadata_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    // Retire first
    let retire_cmd = DeleteFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
    };
    let _retire_event = retire_fees_invoice_setting(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Now try to update_metadata on retired row
    let update_cmd = UpdateFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
        per_th: 999,
        description: None,
    };
    let err = update_fees_invoice_setting(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

#[test]
fn update_metadata_negative_per_th_validation_error_fisv_i_2() {
    // FISv I-2: re-validated on update.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    let update_cmd = UpdateFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
        per_th: -100, // FISv I-2 violation
        description: None,
    };
    let err = update_fees_invoice_setting(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

// ============================================================================
// RealFeesInvoiceSetting::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_prefix_and_per_th() {
    // Tombstone preserves FISv I-1 prefix + FISv I-2 per_th for
    // legal-record retention.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    let pinned_prefix = row.prefix.clone();
    let pinned_per_th = row.per_th;

    let retire_cmd = DeleteFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
    };
    let event = retire_fees_invoice_setting(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped to Retired
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // FISv I-1 + FISv I-2 fields preserved (tombstone)
    assert_eq!(row.prefix, pinned_prefix);
    assert_eq!(row.per_th, pinned_per_th);

    // Event carries only fees_invoice_setting_id
    assert_eq!(event.fees_invoice_setting_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();

    let retire_cmd = DeleteFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
    };
    let _ = retire_fees_invoice_setting(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = DeleteFeesInvoiceSettingCommand {
        tenant: tenant.clone(),
        fees_invoice_setting_id: id,
    };
    let err = retire_fees_invoice_setting(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// FISv I-1 architectural test: UpdateFeesInvoiceSettingCommand has NO prefix field
// ============================================================================

#[test]
fn update_fees_invoice_setting_command_has_no_prefix_field_fisv_i_1() {
    // FISv I-1 architectural invariant: the update command shape
    // itself enforces that `prefix` is NOT mutable. We verify
    // this at compile time by constructing the command and
    // confirming it has no `prefix` field.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = UpdateFeesInvoiceSettingCommand {
        tenant,
        fees_invoice_setting_id: id,
        per_th: 100,
        description: Some("test".to_owned()),
    };
    // The struct has 4 fields: tenant + fees_invoice_setting_id +
    // per_th + description. NO prefix field.
    assert_eq!(cmd.per_th, 100);
    assert!(cmd.description.is_some());
    assert_eq!(cmd.fees_invoice_setting_id.school_id(), school);
}

// ============================================================================
// Service integration tests
// ============================================================================

#[test]
fn create_fees_invoice_setting_service_event_type_is_finance_fees_invoice_setting_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInvoiceSettingId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (_, event) = create_fees_invoice_setting(cmd, &clock, &g).unwrap();
    assert_eq!(
        <FeesInvoiceSettingCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_invoice_setting.created"
    );
    assert_eq!(
        <FeesInvoiceSettingCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_invoice_setting"
    );
    assert_eq!(
        <FeesInvoiceSettingCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.fees_invoice_setting_id, id);
    // FISv I-1 + FISv I-2 carried downstream
    assert_eq!(event.prefix, "INV");
    assert_eq!(event.per_th, 500);
}
