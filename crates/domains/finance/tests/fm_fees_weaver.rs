//! Behavioural tests for `RealFmFeesWeaver` (Wave 95).
//!
//! Pins FFW I-1 (`percentage ∈ [0, 100]`) end-to-end via the
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
use educore_finance::events::{FmFeesWeaverCreated, FmFeesWeaverRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::FmFeesWeaverId;

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

fn fm_fees_weaver_id(g: &SystemIdGen, school: SchoolId) -> FmFeesWeaverId {
    FmFeesWeaverId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_weaver_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fm_fees_weaver_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFW I-1: percentage ∈ [0, 100] ----

#[test]
fn fresh_full_payload_percentage_valid_ffic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with percentage = 50");
    assert_eq!(row.percentage, 50);
    assert_eq!(row.name, "Weaver-A");
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_percentage_validation_error_ffic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let result = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        -1,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("percentage") && msg.contains("FFW I-1"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_percentage_above_100_validation_error_ffic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let result = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        101,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(msg.contains("percentage") && msg.contains("FFW I-1"));
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_percentage_is_valid_ffic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with percentage = 0 (boundary, valid)");
    assert_eq!(row.percentage, 0);
}

#[test]
fn fresh_percentage_100_is_valid_ffic_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        100,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with percentage = 100 (boundary, valid)");
    assert_eq!(row.percentage, 100);
}

// ---- name guard ----

#[test]
fn fresh_empty_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let result = RealFmFeesWeaver::fresh(
        id,
        "   ".to_string(),
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn fresh_name_is_trimmed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let row = RealFmFeesWeaver::fresh(
        id,
        "  Weaver-A  ".to_string(),
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed and trim name");
    assert_eq!(row.name, "Weaver-A");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        50,
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
    let id = fm_fees_weaver_id(&g, school);
    let mut row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.name, "Weaver-A");
    assert_eq!(row.percentage, 50);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_weaver_id(&g, school);
    let mut row = RealFmFeesWeaver::fresh(
        id,
        "Weaver-A".to_string(),
        50,
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
fn create_fm_fees_weaver_service_emits_created_event_ffic_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_weaver_id(&g, school);
    let cmd = CreateFmFeesWeaverCommand {
        tenant,
        fm_fees_weaver_id: id,
        name: "Weaver-A".to_string(),
        percentage: 50,
    };
    let evt: FmFeesWeaverCreated =
        create_fm_fees_weaver(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.name, "Weaver-A");
    assert_eq!(evt.percentage, 50);
    assert_eq!(evt.fm_fees_weaver_id, id);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <FmFeesWeaverCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_weaver.created"
    );
    assert_eq!(
        <FmFeesWeaverCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_weaver"
    );
    assert_eq!(<FmFeesWeaverCreated as DomainEvent>::SCHEMA_VERSION, 1);
}

#[test]
fn retire_fm_fees_weaver_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fm_fees_weaver_id(&g, school);
    let cmd = RetireFmFeesWeaverCommand {
        tenant,
        fm_fees_weaver_id: id,
    };
    let evt: FmFeesWeaverRetired =
        retire_fm_fees_weaver(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.fm_fees_weaver_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FmFeesWeaverRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_weaver.retired"
    );
}

// =========================================================================
// -- Wave 141 -- RealFmFeesWeaver -- FFW I-2 sum-on-invoice marker --
// =========================================================================

#[test]
fn ffw_i_2_sum_on_invoice_le_child_subtotals_dispatcher_enforced() {
    // FFW I-2 marker test: the sum-on-invoice-<= -sum-of-child-subtotals
    // invariant (the sum of an FmFeesInvoice's line amounts must be
    // <= the sum of its child FmFeesWeaver subtotals for the same
    // invoice) is dispatcher-enforced. This is a cross-aggregate
    // invariant that requires visibility into both the FmFeesInvoice
    // parent aggregate AND its child FmFeesWeaver rows.
    //
    // The dispatcher must, on FmFeesInvoice write, query for all
    // RealFmFeesWeaver rows where fm_fees_invoice_id ==
    // invoice.id, sum their subtotals, and reject the write if
    // invoice.amount_minor > sum(weaver_subtotals) with
    // DomainError::Conflict("FmFeesInvoice amount_minor must be <=
    // sum of child FmFeesWeaver subtotals (FFW I-2)").
    //
    // The aggregate itself cannot enforce this invariant without
    // a cross-aggregate lookup, so we document the dispatcher's
    // responsibility here.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _row = RealFmFeesWeaver::fresh(
        fm_fees_weaver_id(&g, school),
        "Test weaver".to_owned(),
        100,
        tenant.actor_id,
        educore_core::value_objects::Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("weaver constructs");
    let _ = _row;
}
