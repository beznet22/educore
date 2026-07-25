//! Behavioural tests for `RealDirectFeesInstallmentChildPayment` (Wave 96).
//!
//! Pins FFIChild I-1 (`paid_amount_minor >= 0`) end-to-end via the
//! aggregate surface, the service functions, and the emitted events.

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
use educore_finance::events::{
    DirectFeesInstallmentChildPaymentCreated, DirectFeesInstallmentChildPaymentRetired,
};
use educore_finance::prelude::*;
use educore_finance::value_objects::DirectFeesInstallmentChildPaymentId;

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

fn direct_fees_installment_child_payment_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> DirectFeesInstallmentChildPaymentId {
    DirectFeesInstallmentChildPaymentId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn direct_fees_installment_child_payment_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = direct_fees_installment_child_payment_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFIChild I-1: paid_amount_minor >= 0 ----

#[test]
fn fresh_full_payload_paid_amount_valid_ffic_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        10_000,
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
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let result = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        -1,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("paid_amount_minor") && msg.contains("FFIChild I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_paid_amount_is_valid_ffic_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        0,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with paid_amount_minor = 0 (boundary, valid)");
    assert_eq!(row.paid_amount_minor, 0);
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
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
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let mut row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
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
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let mut row = RealDirectFeesInstallmentChildPayment::fresh(
        id,
        installment_id,
        5_000,
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

// ---- service integration ----

#[test]
fn create_direct_fees_installment_child_payment_service_emits_created_event_ffic_child_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = direct_fees_installment_child_payment_id(&g, school);
    let installment_id = educore_finance::value_objects::DirectFeesInstallmentId::new(
        school,
        g.next_uuid(),
    );
    let cmd = CreateDirectFeesInstallmentChildPaymentCommand {
        tenant,
        direct_fees_installment_child_payment_id: id,
        installment_id,
        paid_amount_minor: 10_000,
        note: Some("test".to_string()),
    };
    let evt: DirectFeesInstallmentChildPaymentCreated =
        create_direct_fees_installment_child_payment(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.paid_amount_minor, 10_000);
    assert_eq!(evt.installment_id, installment_id);
    assert_eq!(evt.direct_fees_installment_child_payment_id, id);
    assert_eq!(evt.created_by, actor);
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
    let id = direct_fees_installment_child_payment_id(&g, school);
    let cmd = RetireDirectFeesInstallmentChildPaymentCommand {
        tenant,
        direct_fees_installment_child_payment_id: id,
    };
    let evt: DirectFeesInstallmentChildPaymentRetired =
        retire_direct_fees_installment_child_payment(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.direct_fees_installment_child_payment_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <DirectFeesInstallmentChildPaymentRetired as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment_child_payment.retired"
    );
}
