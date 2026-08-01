//! Integration tests for the **FeesInstallmentAssign aggregate** vertical slice.
//!
//! Pins the FIA I-1 + FIA I-2 invariants end-to-end:
//! - FIA I-1: unique per (assign, installment) scope-key tuple.
//! - FIA I-2: paid_amount <= amount + discount + 3 sub-validations:
//!   amount_minor >= 0, discount_minor >= 0, paid_amount_minor >= 0,
//!   plus companion invariant paid_amount_minor <= amount_minor +
//!   discount_minor.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_fees_installment_assign, retire_fees_installment_assign, FeesAssignId,
    FeesInstallmentAssignCreated, FeesInstallmentAssignId, FeesInstallmentAssignRetired,
    FeesInstallmentId, RealFeesInstallmentAssign,
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

fn fia_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentAssignId {
    FeesInstallmentAssignId::new(school, g.next_uuid())
}

fn assign_id(g: &SystemIdGen, school: SchoolId) -> FeesAssignId {
    FeesAssignId::new(school, g.next_uuid())
}

fn installment_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentId {
    FeesInstallmentId::new(school, g.next_uuid())
}

// =========================================================================
// FIA I-1 tests (carried over from Wave 107)
// =========================================================================

#[test]
fn fees_installment_assign_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let id = fia_id(&g, tenant.school_id);
    assert_eq!(id.school_id(), tenant.school_id);
}

#[test]
fn fees_installment_assign_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let id_a = fia_id(&g, tenant.school_id);
    let id_b = fia_id(&g, tenant.school_id);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), tenant.school_id);
}

#[test]
fn fresh_full_payload_carries_scope_key_tuple_fia_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        id,
        a_id,
        i_id,
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        50_000, // amount_minor (FIA I-2)
        0,      // discount_minor
        0,      // paid_amount_minor
        Some("Term 1 installment".to_owned()),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple + valid due_date must construct");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(agg.amount_minor, 50_000);
    assert_eq!(agg.discount_minor, 0);
    assert_eq!(agg.paid_amount_minor, 0);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_distinct_scope_key_tuples_within_same_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        10_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: distinct scope-key tuple must construct");
    let agg_b = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 10, 15).unwrap(),
        10_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: distinct scope-key tuple must construct");
    assert_ne!(agg_a.fees_assign_id, agg_b.fees_assign_id);
    assert_ne!(agg_a.fees_installment_id, agg_b.fees_installment_id);
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
    assert_eq!(agg.correlation_id, tenant.correlation_id);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-1: scope-key tuple must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

#[test]
fn create_fees_installment_assign_service_emits_created_event_fia() {
    use educore_finance::commands::CreateFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
        fees_assign_id: a_id,
        fees_installment_id: i_id,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
        amount_minor: 25_000,
        discount_minor: 0,
        paid_amount_minor: 0,
        note: Some("Service integration test".to_owned()),
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignCreated) =
        create_fees_installment_assign(cmd, &clock, &ids)
            .expect("create_fees_installment_assign must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.fees_assign_id, a_id);
    assert_eq!(agg.fees_installment_id, i_id);
    assert_eq!(event.fees_installment_assign_id, agg.id);
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(event.discount_minor, 0);
    assert_eq!(event.paid_amount_minor, 0);
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.created"
    );
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign"
    );
    assert_eq!(
        <FeesInstallmentAssignCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn retire_fees_installment_assign_service_emits_retired_event_fia() {
    use educore_finance::commands::RetireFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignRetired) =
        retire_fees_installment_assign(cmd, &clock, &ids)
            .expect("retire_fees_installment_assign must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_installment_assign_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.retired"
    );
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign"
    );
    assert_eq!(
        <FeesInstallmentAssignRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

// =========================================================================
// FIA I-2 tests (Wave 110 new tests for amount/discount/paid tracking)
// =========================================================================

#[test]
fn fresh_zero_amount_and_zero_paid_boundary_valid_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        0,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-2: zero/zero/zero is valid (boundary)");
    assert_eq!(agg.amount_minor, 0);
    assert_eq!(agg.discount_minor, 0);
    assert_eq!(agg.paid_amount_minor, 0);
}

#[test]
fn fresh_negative_amount_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        -1,
        0,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_discount_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        -100,
        0,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative discount_minor must be rejected");
    assert!(
        format!("{err}").contains("discount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_negative_paid_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        0,
        -1,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: negative paid_amount_minor must be rejected");
    assert!(
        format!("{err}").contains("paid_amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_paid_exceeds_cap_validation_error_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    // amount=5000 + discount=500 = cap=5500; paid=6000 must be rejected.
    let err = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        500,
        6_000,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FIA I-2: paid > amount + discount must be rejected");
    assert!(
        format!("{err}")
            .contains("paid_amount_minor must be <= amount_minor + discount_minor"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_paid_equals_cap_boundary_valid_fia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    // amount=5000 + discount=500 = cap=5500; paid=5500 is the valid boundary.
    let agg = RealFeesInstallmentAssign::fresh(
        fia_id(&g, school),
        assign_id(&g, school),
        installment_id(&g, school),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        5_000,
        500,
        5_500,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FIA I-2: paid == amount + discount is the valid boundary");
    assert_eq!(agg.paid_amount_minor, 5_500);
    assert_eq!(agg.amount_minor, 5_000);
    assert_eq!(agg.discount_minor, 500);
}

#[test]
fn create_fees_installment_assign_service_propagates_payment_fields_fia_i_2() {
    use educore_finance::commands::CreateFeesInstallmentAssignCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fia_id(&g, school);
    let a_id = assign_id(&g, school);
    let i_id = installment_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesInstallmentAssignCommand {
        tenant: tenant.clone(),
        fees_installment_assign_id: id,
        fees_assign_id: a_id,
        fees_installment_id: i_id,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 11, 30).unwrap(),
        amount_minor: 100_000,
        discount_minor: 10_000,
        paid_amount_minor: 25_000,
        note: None,
    };
    let (agg, event): (RealFeesInstallmentAssign, FeesInstallmentAssignCreated) =
        create_fees_installment_assign(cmd, &clock, &ids)
            .expect("create_fees_installment_assign must succeed");
    assert_eq!(agg.amount_minor, 100_000);
    assert_eq!(agg.discount_minor, 10_000);
    assert_eq!(agg.paid_amount_minor, 25_000);
    assert_eq!(event.amount_minor, 100_000);
    assert_eq!(event.discount_minor, 10_000);
    assert_eq!(event.paid_amount_minor, 25_000);
}

// ====================================================================
// -- Wave 132 -- FIA I-3 state machine extension tests --
// ====================================================================

use educore_core::error::DomainError;
use educore_core::ids::UserId;
use educore_core::value_objects::Timestamp;
use educore_finance::commands::{
    CancelFeesInstallmentAssignCommand, CloseFeesInstallmentAssignCommand,
};
use educore_finance::events::{FeesInstallmentAssignCancelled, FeesInstallmentAssignClosed};
use educore_finance::services::{cancel_fees_installment_assign, close_fees_installment_assign};
use educore_finance::value_objects::LifecycleStatus;

fn build_fia(actor: UserId, amount: i64, paid: i64) -> RealFeesInstallmentAssign {
    let (_tenant, g) = admin_context();
    let school = _tenant.school_id;
    RealFeesInstallmentAssign::fresh(
        FeesInstallmentAssignId::new(school, g.next_uuid()),
        FeesAssignId::new(school, g.next_uuid()),
        FeesInstallmentId::new(school, g.next_uuid()),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        amount,
        0,
        paid,
        None,
        actor,
        Timestamp::now(),
        _tenant.correlation_id,
    )
    .expect("fresh should succeed")
}

// ---- LifecycleStatus Closed variant + transitions ----

#[test]
fn lifecycle_status_closed_round_trip_fia_i_3() {
    assert_eq!(LifecycleStatus::Closed.as_str(), "closed");
    assert_eq!(LifecycleStatus::parse("closed"), Some(LifecycleStatus::Closed));
    assert_eq!(LifecycleStatus::parse("cl"), Some(LifecycleStatus::Closed));
}

#[test]
fn lifecycle_status_can_transition_paid_to_closed_fia_i_3() {
    assert!(LifecycleStatus::Paid.can_transition_to(LifecycleStatus::Closed));
    assert!(!LifecycleStatus::Closed.can_transition_to(LifecycleStatus::Paid));
    assert!(!LifecycleStatus::Closed.can_transition_to(LifecycleStatus::Open));
    assert!(!LifecycleStatus::Cancelled.can_transition_to(LifecycleStatus::Closed));
}

// ---- fresh initializes lifecycle_status ----

#[test]
fn fresh_initializes_lifecycle_open_balance_correct_fia_i_3() {
    let (tenant, _g) = admin_context();
    let agg = build_fia(tenant.actor_id, 10_000, 0);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Open);
    assert_eq!(agg.balance_minor, 10_000);
    assert_eq!(agg.current_balance_minor(), 10_000);
}

#[test]
fn fresh_with_partial_paid_balance_partial_fia_i_3() {
    let (tenant, _g) = admin_context();
    let agg = build_fia(tenant.actor_id, 10_000, 4_000);
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Open);
    assert_eq!(agg.current_balance_minor(), 6_000);
}

// ---- close mutator ----

#[test]
fn close_open_transitions_to_closed_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.close(actor, Timestamp::now(), _g.next_event_id())
        .expect("close should succeed");
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Closed);
    assert_eq!(agg.current_balance_minor(), 0);
}

#[test]
fn close_paid_transitions_to_closed_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 10_000);
    agg.close(actor, Timestamp::now(), _g.next_event_id())
        .expect("close should succeed");
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Closed);
}

#[test]
fn double_close_returns_conflict_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.close(actor, Timestamp::now(), _g.next_event_id())
        .expect("first close");
    let result = agg.close(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- cancel mutator ----

#[test]
fn cancel_open_no_payments_transitions_to_cancelled_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.cancel(actor, Timestamp::now(), _g.next_event_id())
        .expect("cancel should succeed");
    assert_eq!(agg.lifecycle_status, LifecycleStatus::Cancelled);
}

#[test]
fn cancel_after_payment_returns_conflict_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 3_000);
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn cancel_after_close_returns_conflict_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.close(actor, Timestamp::now(), _g.next_event_id())
        .expect("close");
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn double_cancel_returns_conflict_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.cancel(actor, Timestamp::now(), _g.next_event_id())
        .expect("first cancel");
    let result = agg.cancel(actor, Timestamp::now(), _g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- retire after terminal state ----

#[test]
fn retire_after_close_succeeds_fia_i_3() {
    let (tenant, _g) = admin_context();
    let actor = tenant.actor_id;
    let mut agg = build_fia(actor, 10_000, 0);
    agg.close(actor, Timestamp::now(), _g.next_event_id())
        .expect("close");
    agg.retire(Timestamp::now(), actor)
        .expect("retire after close should succeed (FIA I-3 active_status decoupled)");
    assert!(!agg.is_active());
}

// ---- service integration ----

#[test]
fn close_service_emits_event_fia_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealFeesInstallmentAssign::fresh(
        FeesInstallmentAssignId::new(school, g.next_uuid()),
        FeesAssignId::new(school, g.next_uuid()),
        FeesInstallmentId::new(school, g.next_uuid()),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        10_000,
        0,
        0,
        None,
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = CloseFeesInstallmentAssignCommand {
        tenant,
        fees_installment_assign_id: id,
    };
    let (updated, evt): (RealFeesInstallmentAssign, FeesInstallmentAssignClosed) =
        close_fees_installment_assign(agg, cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(updated.lifecycle_status, LifecycleStatus::Closed);
    assert_eq!(evt.lifecycle_status, LifecycleStatus::Closed);
    assert_eq!(evt.closed_by, actor);
    assert_eq!(
        <FeesInstallmentAssignClosed as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.closed"
    );
    assert_eq!(
        <FeesInstallmentAssignClosed as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_assign"
    );
}

#[test]
fn cancel_service_emits_event_fia_i_3() {
    let clock = educore_core::clock::SystemClock;
    let g = SystemIdGen;
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let agg = RealFeesInstallmentAssign::fresh(
        FeesInstallmentAssignId::new(school, g.next_uuid()),
        FeesAssignId::new(school, g.next_uuid()),
        FeesInstallmentId::new(school, g.next_uuid()),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        10_000,
        0,
        0,
        None,
        actor,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh");
    let id = agg.id;
    let cmd = CancelFeesInstallmentAssignCommand {
        tenant,
        fees_installment_assign_id: id,
    };
    let (updated, evt): (RealFeesInstallmentAssign, FeesInstallmentAssignCancelled) =
        cancel_fees_installment_assign(agg, cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(updated.lifecycle_status, LifecycleStatus::Cancelled);
    assert_eq!(evt.cancelled_by, actor);
    assert_eq!(evt.lifecycle_status, LifecycleStatus::Cancelled);
    assert_eq!(
        <FeesInstallmentAssignCancelled as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_assign.cancelled"
    );
}
