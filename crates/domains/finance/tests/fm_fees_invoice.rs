//! Behavioural tests for `RealFmFeesInvoice` (Wave 100 + Wave 127
//! FFI I-3 state machine extension).
//!
//! Pins FFI I-1 (`amount_minor >= 0`) + FFI I-2 (`due_date >=
//! invoice_date`) + FFI I-3 (Pending -> Approved | Rejected state
//! machine) end-to-end via the aggregate surface, the service
//! functions, and the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, SchoolId};
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::events::{
    FmFeesInvoiceApproved, FmFeesInvoiceCreated, FmFeesInvoiceRejected, FmFeesInvoiceRetired,
};
use educore_finance::value_objects::ApprovalStatus;
use educore_finance::prelude::*;
use educore_finance::value_objects::FmFeesInvoiceId;

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

fn fm_fees_invoice_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceId {
    FmFeesInvoiceId::new(school, g.next_uuid())
}

fn invoice_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date")
}

fn due_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 12, 31).expect("valid date")
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_invoice_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fm_fees_invoice_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFI I-1: amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_ffi_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "INV-2026-001".to_string(),
        "student-A".to_string(),
        15_000,
        Some(2_000),
        Some("Q1 tuition".to_string()),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 15_000");
    assert_eq!(row.amount_minor, 15_000);
    assert_eq!(row.invoice_number, "INV-2026-001");
    assert_eq!(row.payer_reference, "student-A");
    assert_eq!(row.discount_minor, Some(2_000));
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_ffi_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        -1,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("FFI I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_is_valid_ffi_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        0,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
}

// ---- companion invariants ----

#[test]
fn fresh_negative_discount_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        1_000,
        Some(-1),
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_empty_invoice_number_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoice::fresh(
        id,
        "   ".to_string(),
        "student-A".to_string(),
        1_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_empty_payer_reference_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "   ".to_string(),
        1_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_invoice_number_is_trimmed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "  INV-001  ".to_string(),
        "student-A".to_string(),
        1_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim invoice_number");
    assert_eq!(row.invoice_number, "INV-001");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        5_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
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
    let id = fm_fees_invoice_id(&g, school);
    let mut row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        5_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
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
    assert_eq!(row.invoice_number, "INV-001");
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let mut row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        5_000,
        None,
        None,
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
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

// ---- FFI I-2: due_date >= invoice_date ----

#[test]
fn fresh_due_date_equals_invoice_date_boundary_valid_ffi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        1_000,
        None,
        None,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFI I-2: due_date == invoice_date is the valid boundary");
    assert_eq!(row.invoice_date, chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
    assert_eq!(row.due_date, chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap());
}

#[test]
fn fresh_due_date_after_invoice_date_valid_ffi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let row = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        1_000,
        None,
        None,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("FFI I-2: due_date > invoice_date is valid");
    assert!(row.due_date > row.invoice_date);
}

#[test]
fn fresh_due_date_before_invoice_date_validation_error_ffi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_id(&g, school);
    let result = RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "student-A".to_string(),
        1_000,
        None,
        None,
        chrono::NaiveDate::from_ymd_opt(2026, 9, 30).unwrap(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("due_date must be >= invoice_date") && msg.contains("FFI I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

// ---- service integration ----

#[test]
fn create_fm_fees_invoice_service_emits_created_event_ffi_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_id(&g, school);
    let cmd = CreateFmFeesInvoiceCommand {
        tenant,
        fm_fees_invoice_id: id,
        invoice_number: "INV-2026-001".to_string(),
        payer_reference: "student-A".to_string(),
        amount_minor: 15_000,
        discount_minor: Some(2_000),
        note: Some("Q1 tuition".to_string()),
        invoice_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let evt: FmFeesInvoiceCreated =
        create_fm_fees_invoice(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 15_000);
    assert_eq!(evt.invoice_number, "INV-2026-001");
    assert_eq!(evt.payer_reference, "student-A");
    assert_eq!(evt.fm_fees_invoice_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <FmFeesInvoiceCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice.created"
    );
    assert_eq!(
        <FmFeesInvoiceCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_invoice"
    );
    assert_eq!(
        <FmFeesInvoiceCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_fm_fees_invoice_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_invoice_id(&g, school);
    let cmd = RetireFmFeesInvoiceCommand {
        tenant,
        fm_fees_invoice_id: id,
    };
    let evt: FmFeesInvoiceRetired =
        retire_fm_fees_invoice(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.fm_fees_invoice_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesInvoiceRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice.retired"
    );
}

// =============================================================================
// FFI I-3: state machine tests (Wave 127)
// =============================================================================

fn make_invoice_for_state_machine(
    g: &SystemIdGen,
    actor: UserId,
    corr: CorrelationId,
    school: SchoolId,
) -> RealFmFeesInvoice {
    let id = fm_fees_invoice_id(g, school);
    RealFmFeesInvoice::fresh(
        id,
        "INV-001".to_string(),
        "STU-001".to_string(),
        10_000,
        None,
        None,
        invoice_date(),
        due_date(),
        actor,
        Timestamp::now(),
        corr,
    )
    .expect("fresh should succeed")
}

#[test]
fn fresh_initial_status_is_pending_ffi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_invoice_for_state_machine(&g, tenant.actor_id, tenant.correlation_id, school);
    assert_eq!(row.status, ApprovalStatus::Pending);
    assert_eq!(row.approved_by, None);
    assert_eq!(row.approved_at, None);
    assert_eq!(row.rejected_by, None);
    assert_eq!(row.rejected_at, None);
    assert_eq!(row.reject_note, None);
}

#[test]
fn approve_transitions_pending_to_approved_ffi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_invoice_for_state_machine(&g, tenant.actor_id, tenant.correlation_id, school);
    assert_eq!(row.status, ApprovalStatus::Pending);
    let event_id = g.next_event_id();
    let at = Timestamp::now();
    row.approve(tenant.actor_id, at, event_id).expect("approve should succeed");
    assert_eq!(row.status, ApprovalStatus::Approved);
    assert_eq!(row.approved_by, Some(tenant.actor_id));
    assert_eq!(row.approved_at, Some(at));
    assert_eq!(row.last_event_id, Some(event_id));
}

#[test]
fn reject_transitions_pending_to_rejected_ffi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_invoice_for_state_machine(&g, tenant.actor_id, tenant.correlation_id, school);
    let event_id = g.next_event_id();
    let at = Timestamp::now();
    let note = "Missing receipt".to_string();
    row.reject(tenant.actor_id, note.clone(), at, event_id)
        .expect("reject should succeed");
    assert_eq!(row.status, ApprovalStatus::Rejected);
    assert_eq!(row.rejected_by, Some(tenant.actor_id));
    assert_eq!(row.rejected_at, Some(at));
    assert_eq!(row.reject_note, Some(note));
    assert_eq!(row.last_event_id, Some(event_id));
}

#[test]
fn double_approve_returns_conflict_ffi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_invoice_for_state_machine(&g, tenant.actor_id, tenant.correlation_id, school);
    row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id())
        .expect("first approve should succeed");
    let result = row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id());
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn reject_after_approve_returns_conflict_ffi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_invoice_for_state_machine(&g, tenant.actor_id, tenant.correlation_id, school);
    row.approve(tenant.actor_id, Timestamp::now(), g.next_event_id())
        .expect("first approve should succeed");
    let result = row.reject(
        tenant.actor_id,
        "too late".to_string(),
        Timestamp::now(),
        g.next_event_id(),
    );
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

#[test]
fn approve_fm_fees_invoice_service_emits_approved_event_ffi_i_3() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let agg = make_invoice_for_state_machine(&g, actor, corr, school);
    let id = agg.id;
    let cmd = ApproveFmFeesInvoiceCommand {
        tenant,
        fm_fees_invoice_id: id,
    };
    let (updated, evt): (RealFmFeesInvoice, FmFeesInvoiceApproved) =
        approve_fm_fees_invoice(agg, cmd, &clock, &g).expect("approve service should succeed");
    assert_eq!(updated.status, ApprovalStatus::Approved);
    assert_eq!(evt.approved_by, actor);
    assert_eq!(evt.status, ApprovalStatus::Approved);
    assert_eq!(
        <FmFeesInvoiceApproved as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice.approved"
    );
    assert_eq!(
        <FmFeesInvoiceApproved as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_invoice"
    );
}

#[test]
fn reject_fm_fees_invoice_service_emits_rejected_event_ffi_i_3() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let agg = make_invoice_for_state_machine(&g, actor, corr, school);
    let id = agg.id;
    let cmd = RejectFmFeesInvoiceCommand {
        tenant,
        fm_fees_invoice_id: id,
        reject_note: "Wrong amount".to_string(),
    };
    let (updated, evt): (RealFmFeesInvoice, FmFeesInvoiceRejected) =
        reject_fm_fees_invoice(agg, cmd, &clock, &g).expect("reject service should succeed");
    assert_eq!(updated.status, ApprovalStatus::Rejected);
    assert_eq!(evt.rejected_by, actor);
    assert_eq!(evt.reject_note, "Wrong amount");
    assert_eq!(
        <FmFeesInvoiceRejected as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_invoice.rejected"
    );
    assert_eq!(
        <FmFeesInvoiceRejected as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_invoice"
    );
}
