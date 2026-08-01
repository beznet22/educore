//! Behavioural tests for `RealDirectFeesInstallmentChildPayment` (Wave 96 + Wave 122 extensions).
//!
//! Pins FFIChild I-1 (`paid_amount_minor >= 0`) + DFIACP I-2
//! (`paid_amount_minor` monotonically non-decreasing) end-to-end
//! via the aggregate surface, the service functions, and the
//! emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_core::value_objects::Version;
use educore_events::domain_event::DomainEvent;
use educore_finance::events::{DirectFeesInstallmentChildPaymentCreated, DirectFeesInstallmentChildPaymentRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::{
    DirectFeesInstallmentChildPaymentId, DirectFeesInstallmentId,
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

fn dfiacp_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesInstallmentChildPaymentId {
    DirectFeesInstallmentChildPaymentId::new(school, g.next_uuid())
}

fn dfi_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesInstallmentId {
    DirectFeesInstallmentId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn direct_fees_installment_child_payment_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = dfiacp_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFIChild I-1: paid_amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_ffic_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        10_000,
        None, // DFIACP I-2: first payment row
        Some("installment 1".to_string()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with paid_amount_minor = 10_000");
    assert_eq!(row.paid_amount_minor, 10_000);
    assert_eq!(row.installment_id, installment_id);
    assert_eq!(row.note.as_deref(), Some("installment 1"));
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_paid_amount_validation_error_ffic_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let result = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        -1,
        None,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("paid_amount_minor must be >= 0"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_boundary_valid_ffic_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        0,
        None,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero paid_amount_minor is valid boundary");
    assert_eq!(row.paid_amount_minor, 0);
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
        None,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.last_event_id, None);
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let mut row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
        None,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.paid_amount_minor, 5_000);
    assert_eq!(row.installment_id, installment_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let mut row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
        None,
        None,
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

// =========================================================================
// DFIACP I-2 tests (Wave 122 new tests for monotonicity)
// =========================================================================

#[test]
fn fresh_first_payment_with_none_previous_paid_is_valid_dfiacp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let agg = RealDirectFeesInstallmentChildPayment::fresh(
        dfiacp_id(&g, school),
        dfi_id(&g, school),
        1_000,
        None, // DFIACP I-2: first payment row, no previous cumulative
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("DFIACP I-2: first payment with None previous is valid");
    assert_eq!(agg.paid_amount_minor, 1_000);
}

#[test]
fn fresh_paid_equals_previous_boundary_valid_dfiacp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let agg = RealDirectFeesInstallmentChildPayment::fresh(
        dfiacp_id(&g, school),
        dfi_id(&g, school),
        5_000,
        Some(5_000), // DFIACP I-2: equality boundary (row that doesn't change total)
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("DFIACP I-2: paid == previous is valid boundary (no change)");
    assert_eq!(agg.paid_amount_minor, 5_000);
}

#[test]
fn fresh_paid_greater_than_previous_valid_dfiacp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let agg = RealDirectFeesInstallmentChildPayment::fresh(
        dfiacp_id(&g, school),
        dfi_id(&g, school),
        7_500,
        Some(5_000), // DFIACP I-2: monotonic increase
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("DFIACP I-2: paid > previous is valid (monotonic increase)");
    assert_eq!(agg.paid_amount_minor, 7_500);
}

#[test]
fn fresh_paid_less_than_previous_validation_error_dfiacp_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealDirectFeesInstallmentChildPayment::fresh(
        dfiacp_id(&g, school),
        dfi_id(&g, school),
        3_000,
        Some(5_000), // DFIACP I-2: regression -- should be rejected
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("paid_amount_minor must be monotonically non-decreasing")
                    && msg.contains("DFIACP I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

// ---- service integration ----

#[test]
fn create_direct_fees_installment_child_payment_service_emits_created_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = dfiacp_id(&g, school);
    let installment_id = dfi_id(&g, school);
    let cmd = CreateDirectFeesInstallmentChildPaymentCommand {
        tenant,
        direct_fees_installment_child_payment_id: id,
        installment_id,
        paid_amount_minor: 10_000,
        previous_paid_amount_minor: None, // DFIACP I-2
        note: Some("test".to_string()),
    };
    let evt: DirectFeesInstallmentChildPaymentCreated =
        create_direct_fees_installment_child_payment(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.paid_amount_minor, 10_000);
    assert_eq!(evt.installment_id, installment_id);
    assert_eq!(
        <DirectFeesInstallmentChildPaymentCreated as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment_child_payment.created"
    );
    assert_eq!(
        <DirectFeesInstallmentChildPaymentCreated as DomainEvent>::AGGREGATE_TYPE,
        "direct_fees_installment_child_payment"
    );
    assert_eq!(
        <DirectFeesInstallmentChildPaymentCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_direct_fees_installment_child_payment_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireDirectFeesInstallmentChildPaymentCommand {
        tenant,
        direct_fees_installment_child_payment_id: dfiacp_id(&g, school),
    };
    let evt: DirectFeesInstallmentChildPaymentRetired =
        retire_direct_fees_installment_child_payment(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <DirectFeesInstallmentChildPaymentRetired as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment_child_payment.retired"
    );
}
