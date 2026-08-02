//! Behavioural tests for `RealFmFeesInvoiceChild` (Wave 101 + Wave 120 extensions).
//!
//! Pins FFIChild I-1 (`amount_minor >= 0`) + FFIChild I-2
//! (`sub_total_minor == amount_minor + weaver_minor + fine_minor`)
//! end-to-end via the aggregate surface, the service functions,
//! and the emitted events.

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
use educore_finance::events::{FmFeesInvoiceChildCreated, FmFeesInvoiceChildRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::{FmFeesInvoiceChildId, FmFeesInvoiceId};

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

fn fm_fees_invoice_child_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceChildId {
    FmFeesInvoiceChildId::new(school, g.next_uuid())
}

fn fm_fees_invoice_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceId {
    FmFeesInvoiceId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_invoice_child_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fm_fees_invoice_child_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFIChild I-1: amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_ffi_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Library fee".to_string(),
        12_000, // amount_minor (FFIChild I-1)
        12_000, // sub_total_minor = amount + weaver + fine (FFIChild I-2)
        0,      // weaver_minor
        0,      // fine_minor
        0,      // placeholder paid_amount_minor (FFIChild I-3)
        0,      // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFIChild I-1 + I-2: fresh should succeed");
    assert_eq!(row.amount_minor, 12_000);
    assert_eq!(row.sub_total_minor, 12_000);
    assert_eq!(row.weaver_minor, 0);
    assert_eq!(row.fine_minor, 0);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_ffi_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Negative".to_string(),
        -1, // amount_minor
        -1, // sub_total_minor
        0,
        0,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_zero_amount_boundary_valid_ffi_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Free sample".to_string(),
        0, // amount_minor
        0, // sub_total_minor
        0,
        0,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero amount + zero sub_total is valid boundary");
    assert_eq!(row.amount_minor, 0);
    assert_eq!(row.sub_total_minor, 0);
}

// ---- FFIChild I-2: sub_total_minor == amount + weaver + fine ----

#[test]
fn fresh_sub_total_equals_sum_with_weaver_and_fine_valid_ffi_child_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Tuition + library + late fine".to_string(),
        10_000, // amount_minor
        12_500, // sub_total_minor = 10_000 + 1_500 + 1_000
        1_500,  // weaver_minor
        1_000,  // fine_minor
        0,      // placeholder paid_amount_minor (FFIChild I-3)
        0,      // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFIChild I-2: sub_total == amount + weaver + fine is valid");
    assert_eq!(row.amount_minor, 10_000);
    assert_eq!(row.sub_total_minor, 12_500);
    assert_eq!(row.weaver_minor, 1_500);
    assert_eq!(row.fine_minor, 1_000);
}

#[test]
fn fresh_sub_total_mismatch_validation_error_ffi_child_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Sub_total mismatch".to_string(),
        10_000, // amount_minor
        11_000, // sub_total_minor: should be 12_500
        1_500,
        1_000,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("sub_total_minor must equal amount_minor + weaver_minor + fine_minor")
                    && msg.contains("FFIChild I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_negative_weaver_validation_error_ffi_child_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Negative weaver".to_string(),
        10_000,
        10_000, // sub_total would be 10_000 + -1000 + 0 = 9_000 (but weaver is invalid)
        -1_000,
        0,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("weaver_minor must be >= 0") && msg.contains("FFIChild I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_negative_fine_validation_error_ffi_child_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Negative fine".to_string(),
        10_000,
        10_000, // sub_total would be 10_000 + 0 + -500 = 9_500 (but fine is invalid)
        0,
        -500,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("fine_minor must be >= 0") && msg.contains("FFIChild I-2"),
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
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Audit footer".to_string(),
        5_000,
        5_000,
        0,
        0,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Will be retired".to_string(),
        5_000,
        5_000,
        0,
        0,
        0, // placeholder paid_amount_minor (FFIChild I-3)
        0, // placeholder service_charge_minor (FFIChild I-3)
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
}

// ---- service integration ----

#[test]
fn create_fm_fees_invoice_child_service_emits_created_event_ffi_child_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let child_id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let cmd = CreateFmFeesInvoiceChildCommand {
        tenant,
        fm_fees_invoice_child_id: child_id,
        invoice_id,
        description: "Service integration".to_string(),
        amount_minor: 10_000,
        sub_total_minor: 12_500,
        weaver_minor: 1_500,
        fine_minor: 1_000,
        paid_amount_minor: 0,
        service_charge_minor: 0,
    };
    let evt: FmFeesInvoiceChildCreated =
        create_fm_fees_invoice_child(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 10_000);
    assert_eq!(evt.sub_total_minor, 12_500);
    assert_eq!(evt.weaver_minor, 1_500);
    assert_eq!(evt.fine_minor, 1_000);
    assert_eq!(evt.fm_fees_invoice_child_id, child_id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <FmFeesInvoiceChildCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_child.created"
    );
    assert_eq!(
        <FmFeesInvoiceChildCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_invoice_child"
    );
    assert_eq!(
        <FmFeesInvoiceChildCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn create_fm_fees_invoice_child_service_rejects_sub_total_mismatch_ffi_child_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = CreateFmFeesInvoiceChildCommand {
        tenant,
        fm_fees_invoice_child_id: fm_fees_invoice_child_id(&g, school),
        invoice_id: fm_fees_invoice_id(&g, school),
        description: "Sub-total mismatch".to_string(),
        amount_minor: 10_000,
        sub_total_minor: 11_000, // mismatch: should be 10_000 + 1_500 + 1_000 = 12_500
        weaver_minor: 1_500,
        fine_minor: 1_000,
        paid_amount_minor: 0,
        service_charge_minor: 0,
    };
    let result = create_fm_fees_invoice_child(cmd, &clock, &g);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

// =========================================================================
// FFIChild I-3 tests (Wave 121 new tests for paid_amount +
// service_charge payment tracking)
// =========================================================================

#[test]
fn fresh_paid_amount_zero_boundary_valid_ffi_child_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Unpaid row".to_string(),
        5_000,
        5_000,
        0,
        0,
        0, // paid_amount_minor
        0, // service_charge_minor
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFIChild I-3: zero paid_amount is valid boundary");
    assert_eq!(row.paid_amount_minor, 0);
    assert_eq!(row.service_charge_minor, 0);
}

#[test]
fn fresh_paid_equals_cap_with_service_charge_boundary_valid_ffi_child_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    // sub_total = 12_000, service_charge = 3_000, cap = 15_000; paid = 15_000 is valid boundary
    let row = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Paid in full".to_string(),
        10_000,
        12_000,
        1_500,
        500,
        15_000, // paid = sub_total + service_charge
        3_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFIChild I-3: paid == sub_total + service_charge is valid boundary");
    assert_eq!(row.paid_amount_minor, 15_000);
    assert_eq!(row.service_charge_minor, 3_000);
}

#[test]
fn fresh_negative_paid_amount_validation_error_ffi_child_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Negative paid".to_string(),
        5_000,
        5_000,
        0,
        0,
        -1, // paid_amount_minor
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("paid_amount_minor must be >= 0") && msg.contains("FFIChild I-3"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_paid_exceeds_cap_validation_error_ffi_child_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    // sub_total = 10_000, service_charge = 2_000, cap = 12_000; paid = 13_000 must be rejected
    let result = RealFmFeesInvoiceChild::fresh(
        fm_fees_invoice_child_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "Overpayment".to_string(),
        10_000,
        10_000,
        0,
        0,
        13_000, // paid > cap
        2_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("paid_amount_minor must be <= sub_total_minor + service_charge_minor")
                    && msg.contains("FFIChild I-3"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn retire_fm_fees_invoice_child_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireFmFeesInvoiceChildCommand {
        tenant,
        fm_fees_invoice_child_id: fm_fees_invoice_child_id(&g, school),
    };
    let evt: FmFeesInvoiceChildRetired =
        retire_fm_fees_invoice_child(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesInvoiceChildRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_child.retired"
    );
}
