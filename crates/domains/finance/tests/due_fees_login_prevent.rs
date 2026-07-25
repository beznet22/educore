//! Behavioural tests for `RealDueFeesLoginPrevent` (Wave 91).
//!
//! Covers:
//! - DFLP I-1: unique per (school, academic, user, role) \xe2\x80\x94
//!   pinned at construction (scope-key fields
//!   academic_year_id + user_id + user_type + school_id are
//!   NOT mutable via update_metadata; dispatcher enforces the
//!   4-key tuple uniqueness at storage layer)
//! - DFLP I-2: auto-pruned when balance = 0 \xe2\x80\x94
//!   `outstanding_balance_minor > 0` validation in
//!   `RealDueFeesLoginPrevent::fresh` + dedicated `prune()`
//!   method (distinct from manual `retire()`) emits a separate
//!   `DueFeesLoginPreventPruned` event
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use educore_academic::AcademicYearId;
use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::DueFeesLoginPreventId;

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

fn make_block_cmd(
    tenant: TenantContext,
    id: DueFeesLoginPreventId,
    g: &SystemIdGen,
) -> BlockLoginForDueFeesCommand {
    let school = tenant.school_id;
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let user_id = g.next_user_id();
    BlockLoginForDueFeesCommand {
        tenant,
        due_fees_login_prevent_id: id,
        academic_year_id, // DFLP I-1 pinned
        user_id,          // DFLP I-1 pinned
        user_type: DueFeesLoginPreventRole::Student, // DFLP I-1 pinned
        outstanding_balance_minor: 50_000, // \xe2\x82\xb9500.00 (DFLP I-2: must be > 0)
        reason: "Tuition overdue Q3 2026".to_owned(),
    }
}

// ============================================================================
// Typed-id smoke tests
// ============================================================================

#[test]
fn typed_id_smoke_due_fees_login_prevent_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_due_fees_login_prevent_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = DueFeesLoginPreventId::new(school, g.next_uuid());
    let id_b = DueFeesLoginPreventId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealDueFeesLoginPrevent::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload_student_role_dflp_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, event) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    // DFLP I-1: scope-key fields pinned
    assert_eq!(row.user_type, DueFeesLoginPreventRole::Student);
    assert!(row.is_student_role());
    assert!(!row.is_parent_role());
    assert!(!row.is_staff_role());
    assert_eq!(row.outstanding_balance_minor, 50_000);
    assert_eq!(row.reason, "Tuition overdue Q3 2026");
    assert_eq!(event.due_fees_login_prevent_id, id);
    assert_eq!(event.outstanding_balance_minor, 50_000);
    assert_eq!(event.reason, "Tuition overdue Q3 2026");
}

#[test]
fn fresh_full_payload_parent_role_dflp_i_1() {
    // DFLP I-1: Parent role accepted
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let mut cmd = make_block_cmd(tenant.clone(), id, &g);
    cmd.user_type = DueFeesLoginPreventRole::Parent;
    let clock = SystemClock;

    let (row, event) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();
    assert_eq!(row.user_type, DueFeesLoginPreventRole::Parent);
    assert!(row.is_parent_role());
    assert!(!row.is_student_role());
    assert_eq!(event.user_type, DueFeesLoginPreventRole::Parent);
}

#[test]
fn fresh_full_payload_staff_role_dflp_i_1() {
    // DFLP I-1: Staff role accepted
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let mut cmd = make_block_cmd(tenant.clone(), id, &g);
    cmd.user_type = DueFeesLoginPreventRole::Staff;
    let clock = SystemClock;

    let (row, event) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();
    assert_eq!(row.user_type, DueFeesLoginPreventRole::Staff);
    assert!(row.is_staff_role());
    assert!(!row.is_student_role());
    assert_eq!(event.user_type, DueFeesLoginPreventRole::Staff);
}

#[test]
fn fresh_empty_reason_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let mut cmd = make_block_cmd(tenant.clone(), id, &g);
    cmd.reason = "   ".to_owned(); // trims to empty
    let clock = SystemClock;

    let err = create_due_fees_login_prevent(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_zero_balance_validation_error_dflp_i_2() {
    // DFLP I-2: outstanding_balance_minor must be > 0 at creation
    // (a zero balance means no block is needed).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let mut cmd = make_block_cmd(tenant.clone(), id, &g);
    cmd.outstanding_balance_minor = 0; // zero \xe2\x86\x92 block is meaningless
    let clock = SystemClock;

    let err = create_due_fees_login_prevent(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_negative_balance_validation_error_dflp_i_2() {
    // DFLP I-2: outstanding_balance_minor must be > 0 (negative
    // balances are nonsensical for a block).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let mut cmd = make_block_cmd(tenant.clone(), id, &g);
    cmd.outstanding_balance_minor = -1;
    let clock = SystemClock;

    let err = create_due_fees_login_prevent(cmd, &clock, &g).unwrap_err();
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
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();
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
// RealDueFeesLoginPrevent::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_changes_reason_preserves_scope_keys_dflp_i_1() {
    // DFLP I-1: scope-key fields (academic_year_id + user_id +
    // user_type) are NOT mutable via update_metadata; only
    // `reason` changes.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    let pinned_year = row.academic_year_id;
    let pinned_user = row.user_id;
    let pinned_user_type = row.user_type;
    let pinned_balance = row.outstanding_balance_minor;

    let update_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "Updated: tuition + transport overdue".to_owned(),
    };
    let event = update_due_fees_login_prevent(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable field DID change
    assert_eq!(row.reason, "Updated: tuition + transport overdue");
    assert_eq!(event.reason, "Updated: tuition + transport overdue");

    // DFLP I-1: scope-key fields preserved (not mutable)
    assert_eq!(row.academic_year_id, pinned_year);
    assert_eq!(row.user_id, pinned_user);
    assert_eq!(row.user_type, pinned_user_type);
    // outstanding_balance_minor also pinned at construction
    assert_eq!(row.outstanding_balance_minor, pinned_balance);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
    assert!(row.last_event_id.is_some());
}

#[test]
fn update_metadata_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    // Retire first (manual)
    let retire_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "manual retire".to_owned(),
    };
    let _retire_event =
        retire_due_fees_login_prevent(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());

    // Now try to update_metadata on retired row
    let update_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "Should not apply".to_owned(),
    };
    let err = update_due_fees_login_prevent(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

#[test]
fn update_metadata_empty_reason_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    let update_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "   ".to_owned(), // trims to empty
    };
    let err = update_due_fees_login_prevent(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

// ============================================================================
// RealDueFeesLoginPrevent::retire tests (MANUAL retirement)
// ============================================================================

#[test]
fn manual_retire_flips_active_status_preserves_scope_keys_dflp_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    let pinned_year = row.academic_year_id;
    let pinned_user = row.user_id;
    let pinned_user_type = row.user_type;

    let retire_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "manual override".to_owned(),
    };
    let event = retire_due_fees_login_prevent(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // DFLP I-1: scope-key fields preserved (tombstone)
    assert_eq!(row.academic_year_id, pinned_year);
    assert_eq!(row.user_id, pinned_user);
    assert_eq!(row.user_type, pinned_user_type);

    // Event type is RETIRED (not Pruned \xe2\x80\x94 manual retirement)
    assert_eq!(
        <DueFeesLoginPreventRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.due_fees_login_prevent.retired"
    );
    assert_eq!(event.due_fees_login_prevent_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn manual_retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    let retire_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "first".to_owned(),
    };
    let _ = retire_due_fees_login_prevent(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "second".to_owned(),
    };
    let err = retire_due_fees_login_prevent(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// RealDueFeesLoginPrevent::prune tests (DFLP I-2 AUTO-prune)
// ============================================================================

#[test]
fn auto_prune_flips_active_status_preserves_scope_keys_dflp_i_2() {
    // DFLP I-2: dispatcher calls `prune` when the user's
    // outstanding balance reaches 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    let pinned_year = row.academic_year_id;
    let pinned_user = row.user_id;
    let pinned_user_type = row.user_type;

    let prune_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "balance = 0".to_owned(),
    };
    let event = prune_due_fees_login_prevent(prune_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // DFLP I-1: scope-key fields preserved (tombstone)
    assert_eq!(row.academic_year_id, pinned_year);
    assert_eq!(row.user_id, pinned_user);
    assert_eq!(row.user_type, pinned_user_type);

    // Event type is PRUNED (distinct from RETIRED \xe2\x80\x94 auto-prune)
    assert_eq!(
        <DueFeesLoginPreventPruned as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.due_fees_login_prevent.pruned"
    );
    assert_ne!(
        <DueFeesLoginPreventRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        <DueFeesLoginPreventPruned as educore_events::domain_event::DomainEvent>::EVENT_TYPE
    );
    assert_eq!(event.due_fees_login_prevent_id, id);
    assert_eq!(event.pruned_by, tenant.actor_id);
}

#[test]
fn auto_prune_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();

    // Retire first (manual)
    let retire_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "manual first".to_owned(),
    };
    let _ = retire_due_fees_login_prevent(retire_cmd, &clock, &g, &mut row).unwrap();

    // Now try to auto-prune (should fail because already retired)
    let prune_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        reason: "balance = 0 (auto)".to_owned(),
    };
    let err = prune_due_fees_login_prevent(prune_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

#[test]
fn manual_retire_emits_retired_event_type_not_pruned() {
    // Architectural invariant: manual retirement + auto-pruning
    // produce DISTINCT event types so the audit log can
    // distinguish them.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = DueFeesLoginPreventId::new(school, g.next_uuid());
    let id_b = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd_a = make_block_cmd(tenant.clone(), id_a, &g);
    let cmd_b = make_block_cmd(tenant.clone(), id_b, &g);
    let clock = SystemClock;

    let (mut row_a, _) = create_due_fees_login_prevent(cmd_a, &clock, &g).unwrap();
    let (mut row_b, _) = create_due_fees_login_prevent(cmd_b, &clock, &g).unwrap();

    let retire_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id_a,
        reason: "manual".to_owned(),
    };
    let _retired_event =
        retire_due_fees_login_prevent(retire_cmd, &clock, &g, &mut row_a).unwrap();

    let prune_cmd = UnblockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id_b,
        reason: "balance = 0".to_owned(),
    };
    let _pruned_event = prune_due_fees_login_prevent(prune_cmd, &clock, &g, &mut row_b).unwrap();

    // Both rows ended up retired, but via DIFFERENT paths.
    // The audit log distinguishes them by event type.
    assert!(!row_a.is_active());
    assert!(!row_b.is_active());
    assert_eq!(
        <DueFeesLoginPreventRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.due_fees_login_prevent.retired"
    );
    assert_eq!(
        <DueFeesLoginPreventPruned as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.due_fees_login_prevent.pruned"
    );
}

// ============================================================================
// DFLP I-1 architectural test: BlockLoginForDueFeesCommand has the scope-key fields
// ============================================================================

#[test]
fn block_login_for_due_fees_command_has_scope_key_fields_dflp_i_1() {
    // DFLP I-1 architectural invariant: the create command shape
    // itself enforces that the (school, academic, user, role)
    // 4-key tuple is provided at construction. We verify this
    // at compile time by constructing the command and confirming
    // it has the 4 scope-key fields.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());
    let user_id = g.next_user_id();
    let cmd = BlockLoginForDueFeesCommand {
        tenant: tenant.clone(),
        due_fees_login_prevent_id: id,
        academic_year_id,
        user_id,
        user_type: DueFeesLoginPreventRole::Student,
        outstanding_balance_minor: 10_000,
        reason: "test".to_owned(),
    };
    // The struct has 7 fields: tenant + due_fees_login_prevent_id +
    // academic_year_id (DFLP I-1) + user_id (DFLP I-1) + user_type
    // (DFLP I-1) + outstanding_balance_minor + reason.
    assert_eq!(cmd.academic_year_id, academic_year_id);
    assert_eq!(cmd.user_id, user_id);
    assert_eq!(cmd.user_type, DueFeesLoginPreventRole::Student);
    assert_eq!(cmd.outstanding_balance_minor, 10_000);
    assert_eq!(cmd.due_fees_login_prevent_id.school_id(), school);
}

// ============================================================================
// Service integration tests
// ============================================================================

#[test]
fn create_due_fees_login_prevent_service_event_type_is_finance_due_fees_login_prevent_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = DueFeesLoginPreventId::new(school, g.next_uuid());
    let cmd = make_block_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (_, event) = create_due_fees_login_prevent(cmd, &clock, &g).unwrap();
    assert_eq!(
        <DueFeesLoginPreventCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.due_fees_login_prevent.created"
    );
    assert_eq!(
        <DueFeesLoginPreventCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "due_fees_login_prevent"
    );
    assert_eq!(
        <DueFeesLoginPreventCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.due_fees_login_prevent_id, id);
    // DFLP I-1: scope-key fields carried downstream
    assert_eq!(event.user_type, DueFeesLoginPreventRole::Student);
    assert_eq!(event.outstanding_balance_minor, 50_000);
}
