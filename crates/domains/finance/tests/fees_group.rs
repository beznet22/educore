//! Behavioural tests for `RealFeesGroup` (Wave 90).
//!
//! Covers:
//! - FG I-1: unique name within school (pinned + non-empty
//!   trimmed at construction via trim-then-empty-check guard;
//!   NOT mutable via update_metadata; dispatcher enforces
//!   `(school_id, name)` uniqueness at storage layer)
//! - FG I-2: non-empty name (the trim-then-empty-check guard
//!   inside `RealFeesGroup::fresh` returns
//!   `DomainError::Validation` if the trimmed name is empty)
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::FeesGroupId;

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

fn make_create_cmd(tenant: TenantContext, id: FeesGroupId) -> CreateFeesGroupCommand {
    CreateFeesGroupCommand {
        tenant,
        fees_group_id: id,
        name: "Tuition Group".to_owned(), // FG I-1 + FG I-2 pinned
        description: Some("All tuition-related fees".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests
// ============================================================================

#[test]
fn typed_id_smoke_fees_group_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = FeesGroupId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_fees_group_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = FeesGroupId::new(school, g.next_uuid());
    let id_b = FeesGroupId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealFeesGroup::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, event) = create_fees_group(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.name, "Tuition Group"); // FG I-1 + FG I-2 pinned
    assert_eq!(row.description.as_deref(), Some("All tuition-related fees"));
    assert_eq!(event.fees_group_id, id);
    assert_eq!(event.name, "Tuition Group");
    assert_eq!(
        event.description.as_deref(),
        Some("All tuition-related fees")
    );
}

#[test]
fn fresh_empty_name_validation_error_fg_i_1() {
    // FG I-1 + FG I-2 guard: empty (whitespace-only) name
    // returns Validation.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.name = "   ".to_owned(); // trims to empty
    let clock = SystemClock;

    let err = create_fees_group(cmd, &clock, &g).unwrap_err();
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
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, _) = create_fees_group(cmd, &clock, &g).unwrap();
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
// RealFeesGroup::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_changes_description_preserves_name() {
    // FG I-1: name is NOT mutable via update_metadata; only
    // description changes.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_group(cmd, &clock, &g).unwrap();

    let pinned_name = row.name.clone();

    let update_cmd = UpdateFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
        description: Some("Renamed: tuition + transport".to_owned()),
    };
    let event = update_fees_group(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable field DID change
    assert_eq!(
        row.description.as_deref(),
        Some("Renamed: tuition + transport")
    );
    assert_eq!(
        event.description.as_deref(),
        Some("Renamed: tuition + transport")
    );

    // FG I-1: name preserved (not mutable)
    assert_eq!(row.name, pinned_name);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
    assert!(row.last_event_id.is_some());
}

#[test]
fn update_metadata_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_group(cmd, &clock, &g).unwrap();

    // Retire first
    let retire_cmd = DeleteFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
    };
    let _retire_event = retire_fees_group(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Now try to update_metadata on retired row
    let update_cmd = UpdateFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
        description: Some("Should not apply".to_owned()),
    };
    let err = update_fees_group(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// RealFeesGroup::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_name() {
    // Tombstone preserves `name` (FG I-1 uniqueness anchor) for
    // legal-record retention + uniqueness queries.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_group(cmd, &clock, &g).unwrap();

    let pinned_name = row.name.clone();

    let retire_cmd = DeleteFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
    };
    let event = retire_fees_group(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped to Retired
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // FG I-1: name preserved (tombstone)
    assert_eq!(row.name, pinned_name);

    // Event carries only fees_group_id
    assert_eq!(event.fees_group_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_fees_group(cmd, &clock, &g).unwrap();

    let retire_cmd = DeleteFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
    };
    let _ = retire_fees_group(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = DeleteFeesGroupCommand {
        tenant: tenant.clone(),
        fees_group_id: id,
    };
    let err = retire_fees_group(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// FG I-1 architectural test: UpdateFeesGroupCommand has NO name field
// ============================================================================

#[test]
fn update_fees_group_command_has_no_name_field_fg_i_1() {
    // FG I-1 architectural invariant: the update command shape
    // itself enforces that `name` is NOT mutable. We verify this
    // at compile time by constructing the command and confirming
    // it has no `name` field (Rust's type system rejects any
    // attempt to set one).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = UpdateFeesGroupCommand {
        tenant,
        fees_group_id: id,
        description: Some("test".to_owned()),
    };
    // The struct has only 3 fields: tenant + fees_group_id + description.
    // Any attempt to add `name` to this struct would fail to compile.
    assert!(cmd.description.is_some());
    assert_eq!(cmd.fees_group_id.school_id(), school);
    // This test passes at compile time = architectural invariant holds.
}

// ============================================================================
// Service integration tests
// ============================================================================

#[test]
fn create_fees_group_service_event_type_is_finance_fees_group_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesGroupId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (_, event) = create_fees_group(cmd, &clock, &g).unwrap();
    assert_eq!(
        <FeesGroupCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_group.created"
    );
    assert_eq!(
        <FeesGroupCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_group"
    );
    assert_eq!(
        <FeesGroupCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.fees_group_id, id);
    assert_eq!(event.name, "Tuition Group"); // FG I-1 + FG I-2 carried downstream
}
