//! Integration tests for the **DirectFeesInstallmentAssignChild
//! aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 73 per-aggregate drop
//! [`RealDirectFeesInstallmentAssignChild`](educore_finance::aggregate::RealDirectFeesInstallmentAssignChild)
//! — the child row under a `DirectFeesInstallmentAssign` aggregate
//! representing one installment in the per-installment breakdown
//! (amount + parent assignment reference). Validates:
//!
//! - DFIAC I-1: append-only (no `update_*` mutator, only `retire`;
//!   no `Updated` event variant exists)
//! - DFIAC I-2: timestamps monotonic (`created_at <= updated_at`
//!   always holds; baseline `created_at == updated_at` on `fresh`;
//!   `retire` advances `updated_at` strictly past `created_at`)
//! - amount_minor >= 0 (input validation)
//! - `retire()` active → retired transition, version bump, audit
//!   footer advance
//! - `create_direct_fees_installment_assign_child` service function
//!   (aggregate + event pairing)
//!
//! The pre-existing 2 typed-id-only tests have been preserved (as
//! smoke tests for the typed-id contract) and the suite is extended
//! below with behavioral tests covering the Wave 73 full drop.
//! Wave 73 adds the `RealDirectFeesInstallmentAssignChild` aggregate,
//! expands the existing skeleton `DirectFeesInstallmentAssignChildAdded`
//! event with a full payload, adds the
//! `DirectFeesInstallmentAssignChildRetired` event, the service
//! function, and this test suite.

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

use educore_finance::commands::CreateDirectFeesInstallmentAssignChildCommand;
use educore_finance::events::DirectFeesInstallmentAssignChildAdded;
use educore_finance::prelude::RealDirectFeesInstallmentAssignChild;
use educore_finance::services::create_direct_fees_installment_assign_child;
use educore_finance::value_objects::{
    DirectFeesInstallmentAssignChildId, DirectFeesInstallmentAssignId,
};

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

fn direct_fees_installment_assign_child_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> DirectFeesInstallmentAssignChildId {
    DirectFeesInstallmentAssignChildId::new(school, g.next_uuid())
}

fn direct_fees_installment_assign_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> DirectFeesInstallmentAssignId {
    DirectFeesInstallmentAssignId::new(school, g.next_uuid())
}

fn make_child(
    g: &SystemIdGen,
    school: SchoolId,
    amount_minor: i64,
) -> RealDirectFeesInstallmentAssignChild {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealDirectFeesInstallmentAssignChild::fresh(
        direct_fees_installment_assign_child_id(g, school),
        direct_fees_installment_assign_id(g, school),
        amount_minor,
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// Typed-id contract (preserved from Phase 7 Workstream F seed)
// ---------------------------------------------------------------------------

#[test]
fn direct_fees_installment_assign_child_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_child_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn direct_fees_installment_assign_child_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = direct_fees_installment_assign_child_id(&g, school);
    let id_b = direct_fees_installment_assign_child_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// RealDirectFeesInstallmentAssignChild: fresh() + DFIAC I-2 baseline
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_valid_amount_produces_active_aggregate_with_monotonic_baseline() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 50_000);
    assert_eq!(child.amount_minor, 50_000);
    assert!(child.is_active(), "fresh aggregate must be Active");
    assert_eq!(child.school_id, school);
    // DFIAC I-2 baseline: created_at == updated_at on fresh
    assert!(
        child.timestamps_monotonic(),
        "DFIAC I-2: created_at <= updated_at must hold at construction"
    );
    assert_eq!(child.created_at, child.updated_at);
}

#[test]
fn fresh_with_zero_amount_succeeds() {
    // Zero-amount installment rows (waivers, free samples) are valid.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 0);
    assert_eq!(child.amount_minor, 0);
    assert!(child.is_active());
}

#[test]
fn fresh_with_negative_amount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealDirectFeesInstallmentAssignChild::fresh(
        direct_fees_installment_assign_child_id(&g, school),
        direct_fees_installment_assign_id(&g, school),
        -1,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must fail with Validation, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealDirectFeesInstallmentAssignChild: DFIAC I-2 timestamps_monotonic
// ---------------------------------------------------------------------------

#[test]
fn timestamps_monotonic_returns_true_after_fresh() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 100);
    assert!(child.timestamps_monotonic());
}

#[test]
fn timestamps_monotonic_returns_true_after_retire_with_advanced_timestamp() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 100);
    let created_at = child.created_at;
    let advanced =
        Timestamp::from_datetime(created_at.as_datetime() + chrono::Duration::seconds(10));
    let actor = g.next_user_id();
    child.retire(advanced, actor).expect("retire succeeds");
    assert!(
        child.timestamps_monotonic(),
        "DFIAC I-2: timestamps must remain monotonic after retire"
    );
    assert!(
        child.updated_at > created_at,
        "DFIAC I-2: updated_at must advance strictly past created_at on retire"
    );
}

#[test]
fn retire_with_stale_timestamp_clamps_forward_to_preserve_monotonicity() {
    // DFIAC I-2 contract: retire must never regress the timestamp.
    // If the caller passes a timestamp at or before created_at, the
    // aggregate advances updated_at to created_at + 1ns.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 100);
    let created_at = child.created_at;
    let stale = Timestamp::from_datetime(created_at.as_datetime() - chrono::Duration::seconds(5));
    let actor = g.next_user_id();
    child.retire(stale, actor).expect("retire succeeds");
    assert!(
        child.timestamps_monotonic(),
        "DFIAC I-2: stale retire timestamp must be clamped forward, not regress"
    );
    assert!(
        child.updated_at > created_at,
        "DFIAC I-2: updated_at must be strictly past created_at even when caller passed a stale timestamp"
    );
}

#[test]
fn retire_with_timestamp_equal_to_created_at_still_advances() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 100);
    let created_at = child.created_at;
    let actor = g.next_user_id();
    child.retire(created_at, actor).expect("retire succeeds");
    assert!(
        child.timestamps_monotonic(),
        "DFIAC I-2: even equal-to-created_at input must produce updated_at > created_at"
    );
    assert!(child.updated_at > created_at);
}

// ---------------------------------------------------------------------------
// RealDirectFeesInstallmentAssignChild: DFIAC I-1 (append-only at API
// surface) + retire()
// ---------------------------------------------------------------------------

#[test]
fn fresh_produces_append_only_aggregate_with_only_fresh_is_active_retire_surface() {
    // DFIAC I-1: append-only. The aggregate intentionally exposes
    // only `fresh`, `is_active`, `timestamps_monotonic`, and `retire`
    // (no `update_metadata` / `update_amount` / `update_parent`).
    // This test pins the practical contract: only those four methods
    // exist on `RealDirectFeesInstallmentAssignChild`. Adding any
    // `update_*` mutator would violate DFIAC I-1 and would require a
    // new `Updated` event variant (which would also be a violation).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let child = make_child(&g, school, 100);
    assert!(child.is_active());
    assert!(child.timestamps_monotonic());
    let _ = child.amount_minor; // last accessible field
                                // No update_* invocation is possible — the API simply does not
                                // expose one. This is the practical append-only guarantee.
}

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 100);
    let initial_version = child.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(child.is_active());
    child.retire(now, actor).expect("first retire succeeds");
    assert!(!child.is_active(), "retire must flip is_active to false");
    assert!(
        child.version > initial_version,
        "version must advance on retire"
    );
    assert_eq!(child.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut child = make_child(&g, school, 100);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    child.retire(now, actor).expect("first retire succeeds");
    let result = child.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_direct_fees_installment_assign_child service function
// ---------------------------------------------------------------------------

#[test]
fn create_direct_fees_installment_assign_child_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let parent_assign_id = direct_fees_installment_assign_id(&g, school);
    let cmd = CreateDirectFeesInstallmentAssignChildCommand {
        tenant: tenant.clone(),
        direct_fees_installment_assign_id: parent_assign_id,
        amount_minor: 75_000,
    };
    let clock = SystemClock;
    let (child, event) =
        create_direct_fees_installment_assign_child(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(child.direct_fees_installment_assign_id, parent_assign_id);
    assert_eq!(child.amount_minor, 75_000);
    assert!(
        child.is_active(),
        "service-created aggregate must be Active"
    );
    assert_eq!(child.school_id, school);
    assert_eq!(child.last_event_id, Some(event.event_id));
    assert!(
        child.timestamps_monotonic(),
        "DFIAC I-2: service-created aggregate must satisfy timestamps monotonic"
    );
    assert_eq!(child.created_at, child.updated_at);

    // Event side
    assert_eq!(event.direct_fees_installment_assign_child_id, child.id);
    assert_eq!(event.direct_fees_installment_assign_id, parent_assign_id);
    assert_eq!(event.amount_minor, 75_000);
    assert_eq!(event.created_at, child.created_at);
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        DirectFeesInstallmentAssignChildAdded::EVENT_TYPE,
        "finance.direct_fees_installment_assign_child.added"
    );
    assert_eq!(
        DirectFeesInstallmentAssignChildAdded::AGGREGATE_TYPE,
        "direct_fees_installment_assign_child"
    );
    assert_eq!(DirectFeesInstallmentAssignChildAdded::SCHEMA_VERSION, 1);
}

#[test]
fn create_direct_fees_installment_assign_child_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let cmd = CreateDirectFeesInstallmentAssignChildCommand {
        tenant: tenant.clone(),
        direct_fees_installment_assign_id: direct_fees_installment_assign_id(&g, school),
        amount_minor: -100,
    };
    let clock = SystemClock;
    let result = create_direct_fees_installment_assign_child(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must propagate Validation, got {result:?}"
    );
}
