//! Behavioural tests for `RealFmFeesInvoiceChild` (Wave 101).
//!
//! Pins FFIChild I-1 (`amount_minor >= 0`) end-to-end via the
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

fn fm_fees_invoice_child_id(
    g: &SystemIdGen,
    school: SchoolId,
) -> FmFeesInvoiceChildId {
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
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Tuition fee Q1".to_string(),
        12_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 12_000");
    assert_eq!(row.amount_minor, 12_000);
    assert_eq!(row.description, "Tuition fee Q1");
    assert_eq!(row.invoice_id, invoice_id);
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_ffi_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Tuition fee".to_string(),
        -1,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("FFIChild I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_is_valid_ffi_child_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Waived line item".to_string(),
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
}

// ---- companion invariants ----

#[test]
fn fresh_empty_description_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "   ".to_string(),
        1_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_description_is_trimmed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "  Tuition fee Q1  ".to_string(),
        1_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim description");
    assert_eq!(row.description, "Tuition fee Q1");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Tuition fee".to_string(),
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
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let mut row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Tuition fee".to_string(),
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
    assert_eq!(row.invoice_id, invoice_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let mut row = RealFmFeesInvoiceChild::fresh(
        id,
        invoice_id,
        "Tuition fee".to_string(),
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
fn create_fm_fees_invoice_child_service_emits_created_event_ffi_child_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_child_id(&g, school);
    let invoice_id = fm_fees_invoice_id(&g, school);
    let cmd = CreateFmFeesInvoiceChildCommand {
        tenant,
        fm_fees_invoice_child_id: id,
        invoice_id,
        description: "Tuition fee Q1".to_string(),
        amount_minor: 12_000,
    };
    let evt: FmFeesInvoiceChildCreated =
        create_fm_fees_invoice_child(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.amount_minor, 12_000);
    assert_eq!(evt.invoice_id, invoice_id);
    assert_eq!(evt.description, "Tuition fee Q1");
    assert_eq!(evt.fm_fees_invoice_child_id, id);
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
fn retire_fm_fees_invoice_child_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_child_id(&g, school);
    let cmd = RetireFmFeesInvoiceChildCommand {
        tenant,
        fm_fees_invoice_child_id: id,
    };
    let evt: FmFeesInvoiceChildRetired =
        retire_fm_fees_invoice_child(cmd, &clock, &g)
            .expect("service should succeed");
    assert_eq!(evt.fm_fees_invoice_child_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesInvoiceChildRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice_child.retired"
    );
}
