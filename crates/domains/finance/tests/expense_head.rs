//! Behavioural tests for `RealExpenseHead` (Wave 89).
//!
//! Covers:
//! - EH I-1: unique name within school (pinned + non-empty
//!   trimmed at construction via `validate_ledger_name` guard;
//!   NOT mutable via update_metadata; dispatcher enforces
//!   `(school_id, name)` uniqueness at storage layer)
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::ExpenseHeadId;

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

fn make_create_cmd(tenant: TenantContext, id: ExpenseHeadId) -> CreateExpenseHeadCommand {
    CreateExpenseHeadCommand {
        tenant,
        expense_head_id: id,
        name: "Office Supplies".to_owned(), // EH I-1 pinned
        description: Some("General office supplies and stationery".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests
// ============================================================================

#[test]
fn typed_id_smoke_expense_head_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = ExpenseHeadId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_expense_head_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = ExpenseHeadId::new(school, g.next_uuid());
    let id_b = ExpenseHeadId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealExpenseHead::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, event) = create_expense_head(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.name, "Office Supplies"); // EH I-1 pinned
    assert_eq!(
        row.description.as_deref(),
        Some("General office supplies and stationery")
    );
    assert_eq!(event.expense_head_id, id);
    assert_eq!(event.name, "Office Supplies");
    assert_eq!(
        event.description.as_deref(),
        Some("General office supplies and stationery")
    );
}

#[test]
fn fresh_empty_name_validation_error_eh_i_1() {
    // EH I-1 guard: empty (whitespace-only) name returns Validation.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id);
    cmd.name = "   ".to_owned(); // trims to empty
    let clock = SystemClock;

    let err = create_expense_head(cmd, &clock, &g).unwrap_err();
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
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (row, _) = create_expense_head(cmd, &clock, &g).unwrap();
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
// RealExpenseHead::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_changes_description_preserves_name() {
    // EH I-1: name is NOT mutable via update_metadata; only
    // description changes.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_expense_head(cmd, &clock, &g).unwrap();

    let pinned_name = row.name.clone();

    let update_cmd = UpdateExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
        description: Some("Renamed: office supplies + equipment".to_owned()),
    };
    let event = update_expense_head(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable field DID change
    assert_eq!(
        row.description.as_deref(),
        Some("Renamed: office supplies + equipment")
    );
    assert_eq!(
        event.description.as_deref(),
        Some("Renamed: office supplies + equipment")
    );

    // EH I-1: name preserved (not mutable)
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
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_expense_head(cmd, &clock, &g).unwrap();

    // Retire first
    let retire_cmd = DeleteExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
    };
    let _retire_event = retire_expense_head(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Now try to update_metadata on retired row
    let update_cmd = UpdateExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
        description: Some("Should not apply".to_owned()),
    };
    let err = update_expense_head(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// RealExpenseHead::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_name() {
    // Tombstone preserves `name` (EH I-1 uniqueness anchor) for
    // legal-record retention + uniqueness queries.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_expense_head(cmd, &clock, &g).unwrap();

    let pinned_name = row.name.clone();

    let retire_cmd = DeleteExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
    };
    let event = retire_expense_head(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped to Retired
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // EH I-1: name preserved (tombstone)
    assert_eq!(row.name, pinned_name);

    // Event carries only expense_head_id
    assert_eq!(event.expense_head_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (mut row, _) = create_expense_head(cmd, &clock, &g).unwrap();

    let retire_cmd = DeleteExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
    };
    let _ = retire_expense_head(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = DeleteExpenseHeadCommand {
        tenant: tenant.clone(),
        expense_head_id: id,
    };
    let err = retire_expense_head(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// EH I-1 architectural test: UpdateExpenseHeadCommand has NO name field
// ============================================================================

#[test]
fn update_expense_head_command_has_no_name_field_eh_i_1() {
    // EH I-1 architectural invariant: the update command shape
    // itself enforces that `name` is NOT mutable. We verify this
    // at compile time by constructing the command and confirming
    // it has no `name` field (Rust's type system rejects any
    // attempt to set one).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = UpdateExpenseHeadCommand {
        tenant,
        expense_head_id: id,
        description: Some("test".to_owned()),
    };
    // The struct has only 3 fields: tenant + expense_head_id + description.
    // Any attempt to add `name` to this struct would fail to compile.
    assert!(cmd.description.is_some());
    assert_eq!(cmd.expense_head_id.school_id(), school);
    // This test passes at compile time = architectural invariant holds.
}

// ============================================================================
// Service integration tests
// ============================================================================

#[test]
fn create_expense_head_service_event_type_is_finance_expense_head_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = ExpenseHeadId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id);
    let clock = SystemClock;

    let (_, event) = create_expense_head(cmd, &clock, &g).unwrap();
    assert_eq!(
        <ExpenseHeadCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.expense_head.created"
    );
    assert_eq!(
        <ExpenseHeadCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "expense_head"
    );
    assert_eq!(
        <ExpenseHeadCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.expense_head_id, id);
    assert_eq!(event.name, "Office Supplies"); // EH I-1 carried downstream
}
