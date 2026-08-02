//! Behavioural tests for `RealDirectFeesInstallmentAssign` (Wave 103).
//!
//! Pins DFIA I-2 (`amount_minor >= 0`) + DFIA I-3 (`balance_minor >= 0`)
//! end-to-end via the aggregate surface, the service functions, and
//! the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::StudentId;
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::events::{
    DirectFeesInstallmentAssignCreated, DirectFeesInstallmentAssignRetired,
};
use educore_finance::prelude::*;
use educore_finance::value_objects::DirectFeesInstallmentAssignId;

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

fn direct_fees_installment_assign_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> DirectFeesInstallmentAssignId {
    DirectFeesInstallmentAssignId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn installment_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesInstallmentId {
    DirectFeesInstallmentId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn direct_fees_installment_assign_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = direct_fees_installment_assign_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- DFIA I-2 + DFIA I-3: amount >= 0 + balance >= 0 ----

#[test]
fn fresh_full_payload_amount_and_balance_valid_dfia() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        15_000, // DFIA I-2
        15_000, // DFIA I-3 (unpaid)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount = balance = 15_000");
    assert_eq!(row.amount_minor, 15_000);
    assert_eq!(row.balance_minor, 15_000);
    assert_eq!(row.student_id, student);
    assert_eq!(row.installment_id, installment);
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_dfia_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let result = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        -1, // DFIA I-2 violation
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("DFIA I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_negative_balance_validation_error_dfia_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let result = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        15_000,
        -1, // DFIA I-3 violation
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("balance_minor") && msg.contains("DFIA I-3"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_and_balance_is_valid_dfia() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        0, // DFIA I-2 boundary
        0, // DFIA I-3 boundary
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount = balance = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
    assert_eq!(row.balance_minor, 0);
}

#[test]
fn fresh_partial_payment_balance_lt_amount_is_valid_dfia() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        15_000,
        10_000, // partial payment
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with balance < amount (partial payment)");
    assert_eq!(row.amount_minor, 15_000);
    assert_eq!(row.balance_minor, 10_000);
}

#[test]
fn fresh_balance_gt_amount_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let result = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        10_000,
        15_000, // balance > amount is nonsensical
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("balance_minor must be <= amount_minor"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        5_000,
        5_000,
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
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let mut row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        5_000,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.balance_minor, 5_000);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let mut row = RealDirectFeesInstallmentAssign::fresh(
        id,
        student,
        installment,
        5_000,
        5_000,
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
fn create_direct_fees_installment_assign_service_emits_created_event_dfia() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = direct_fees_installment_assign_id(&g, school);
    let student = student_id(&g, school);
    let installment = installment_id(&g, school);
    let cmd = CreateDirectFeesInstallmentAssignCommand {
        tenant,
        direct_fees_installment_assign_id: id,
        student_id: student,
        installment_id: installment,
        amount_minor: 15_000,
        balance_minor: 15_000,
    };
    let evt: DirectFeesInstallmentAssignCreated =
        create_direct_fees_installment_assign(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 15_000);
    assert_eq!(evt.balance_minor, 15_000);
    assert_eq!(evt.student_id, student);
    assert_eq!(evt.direct_fees_installment_assign_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <DirectFeesInstallmentAssignCreated as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment_assign.created"
    );
    assert_eq!(
        <DirectFeesInstallmentAssignCreated as DomainEvent>::AGGREGATE_TYPE,
        "direct_fees_installment_assign"
    );
    assert_eq!(
        <DirectFeesInstallmentAssignCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_direct_fees_installment_assign_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = direct_fees_installment_assign_id(&g, school);
    let cmd = RetireDirectFeesInstallmentAssignCommand {
        tenant,
        direct_fees_installment_assign_id: id,
    };
    let evt: DirectFeesInstallmentAssignRetired =
        retire_direct_fees_installment_assign(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.direct_fees_installment_assign_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <DirectFeesInstallmentAssignRetired as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment_assign.retired"
    );
}
