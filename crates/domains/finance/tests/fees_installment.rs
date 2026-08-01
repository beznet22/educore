//! Behavioural tests for `RealFeesInstallment` (Wave 126 full drop).
//!
//! Pins FIv I-1 (`percentage ∈ [0, 100]`) + FIv I-2
//! (`amount_minor >= 0`) end-to-end via the aggregate surface,
//! the service functions, and the emitted events.

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
use educore_core::value_objects::{Timestamp, Version};
use educore_events::domain_event::DomainEvent;
use educore_finance::events::{FeesInstallmentCreated, FeesInstallmentRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::{Currency, FeesInstallmentId, FeesMasterId};

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

fn fiv_id(g: &SystemIdGen, school: SchoolId) -> FeesInstallmentId {
    FeesInstallmentId::new(school, g.next_uuid())
}

fn master_id(g: &SystemIdGen, school: SchoolId) -> FeesMasterId {
    FeesMasterId::new(school, g.next_uuid())
}

fn due_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 7, 1).expect("valid date")
}

// ---- typed-id smoke ----

#[test]
fn fees_installment_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fiv_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FIv I-1 + FIv I-2 happy path ----

#[test]
fn fresh_full_payload_valid_fiv_i_1_fiv_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let master = master_id(&g, school);
    let row = RealFeesInstallment::fresh(
        id,
        master,
        "Q1 installment".to_string(),
        due_date(),
        5_000,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with percentage = 50, amount_minor = 5_000");
    assert_eq!(row.percentage, 50);
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.fees_master_id, master);
    assert_eq!(row.name, "Q1 installment");
    assert_eq!(row.due_date, due_date());
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

// ---- FIv I-2: amount_minor >= 0 ----

#[test]
fn fresh_negative_amount_validation_error_fiv_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let result = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        -1,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor must be >= 0") && msg.contains("FIv I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_boundary_valid_fiv_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        0,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero amount_minor is valid boundary");
    assert_eq!(row.amount_minor, 0);
}

// ---- FIv I-1: percentage ∈ [0, 100] ----

#[test]
fn fresh_negative_percentage_validation_error_fiv_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let result = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
        -1,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("percentage must be in [0, 100]")
                    && msg.contains("FIv I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_percentage_over_100_validation_error_fiv_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let result = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
        101,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("percentage must be in [0, 100]")
                    && msg.contains("FIv I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_percentage_boundary_valid_fiv_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero percentage is valid boundary");
    assert_eq!(row.percentage, 0);
}

#[test]
fn fresh_100_percentage_boundary_valid_fiv_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
        100,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("100 percentage is valid upper boundary");
    assert_eq!(row.percentage, 100);
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
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
    let id = fiv_id(&g, school);
    let mut row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
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
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.percentage, 50);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fiv_id(&g, school);
    let mut row = RealFeesInstallment::fresh(
        id,
        master_id(&g, school),
        "Q1".to_string(),
        due_date(),
        5_000,
        Currency::INR,
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
fn create_fees_installment_service_emits_created_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fiv_id(&g, school);
    let master = master_id(&g, school);
    let cmd = CreateFeesInstallmentCommand {
        tenant,
        fees_installment_id: id,
        fees_master_id: master,
        name: "Q1".to_string(),
        due_date: due_date(),
        amount_minor: 5_000,
        currency: Currency::INR,
        percentage: 50,
    };
    let (_agg, evt): (RealFeesInstallment, FeesInstallmentCreated) =
        create_fees_installment(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.percentage, 50);
    assert_eq!(evt.amount_minor, 5_000);
    assert_eq!(evt.fees_master_id, master);
    assert_eq!(
        <FeesInstallmentCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment.created"
    );
    assert_eq!(
        <FeesInstallmentCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment"
    );
    assert_eq!(
        <FeesInstallmentCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn create_fees_installment_service_rejects_negative_amount_fiv_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fiv_id(&g, school);
    let cmd = CreateFeesInstallmentCommand {
        tenant,
        fees_installment_id: id,
        fees_master_id: master_id(&g, school),
        name: "Q1".to_string(),
        due_date: due_date(),
        amount_minor: -500,
        currency: Currency::INR,
        percentage: 50,
    };
    let result = create_fees_installment(cmd, &clock, &g);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn create_fees_installment_service_rejects_invalid_percentage_fiv_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fiv_id(&g, school);
    let cmd = CreateFeesInstallmentCommand {
        tenant,
        fees_installment_id: id,
        fees_master_id: master_id(&g, school),
        name: "Q1".to_string(),
        due_date: due_date(),
        amount_minor: 5_000,
        currency: Currency::INR,
        percentage: 150,
    };
    let result = create_fees_installment(cmd, &clock, &g);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn retire_fees_installment_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireFeesInstallmentCommand {
        tenant,
        fees_installment_id: fiv_id(&g, school),
        fees_master_id: master_id(&g, school),
    };
    let evt: FeesInstallmentRetired =
        retire_fees_installment(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <FeesInstallmentRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_installment.retired"
    );
    assert_eq!(
        <FeesInstallmentRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_installment"
    );
}

#[test]
fn read_fees_installment_service_returns_ok() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = ReadFeesInstallmentCommand {
        tenant,
        fees_installment_id: fiv_id(&g, school),
    };
    read_fees_installment(cmd, &clock, &g).expect("read should succeed");
}

// =========================================================================
// -- Wave 138 -- RealFeesInstallment -- FIv I-4 due_date ordering marker --
// =========================================================================

#[test]
fn fiv_i_4_due_date_ordering_dispatcher_enforced() {
    // FIv I-4 marker test: the due_date ordering invariant
    // (installments must have strictly ascending due_dates
    // within a single fees_master) is dispatcher-enforced --
    // the aggregate carries `due_date: NaiveDate` as a
    // required field, but the cross-row check requires
    // visibility into the sibling rows. The dispatcher must,
    // on create, query for existing installments with the
    // same `fees_master_id` and reject the create if the new
    // `due_date` is <= the latest existing `due_date`.

    // The aggregate itself cannot enforce this invariant
    // without a sibling-row lookup, so we document the
    // dispatcher's responsibility here. If a future wave
    // adds the per-master ordering check to the aggregate,
    // this marker test should be updated to invoke the
    // aggregate-level guard.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _first = RealFeesInstallment::fresh(
        fiv_id(&g, school),
        master_id(&g, school),
        "First installment".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        5_000,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("first installment constructs with any due_date");
    let _ = _first;
}

// =========================================================================
// -- Wave 139 -- RealFeesInstallment -- FIv I-3 percentage sum marker --
// =========================================================================

#[test]
fn fiv_i_3_percentage_sum_dispatcher_enforced() {
    // FIv I-3 marker test: the percentage sum invariant (the
    // sum of all installment percentages for a single fees_master
    // must be <= 100) is dispatcher-enforced -- the aggregate
    // carries `percentage: i64` + `fees_master_id: FeesMasterId`
    // as required fields, but the cross-row sum check requires
    // visibility into the sibling rows. The dispatcher must, on
    // create, query for existing installments with the same
    // `fees_master_id`, sum their `percentage` values, and
    // reject the create if (new.percentage + sum) > 100.

    // The aggregate itself cannot enforce this invariant
    // without a sibling-row aggregation, so we document the
    // dispatcher's responsibility here.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _first = RealFeesInstallment::fresh(
        fiv_id(&g, school),
        master_id(&g, school),
        "First installment 50pct".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        5_000,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("first installment at 50pct constructs");
    let _ = _first;
}

// =========================================================================
// -- Wave 140 -- RealFeesInstallment -- FIv I-5 non-overlapping windows --
// =========================================================================

#[test]
fn fiv_i_5_non_overlapping_windows_dispatcher_enforced() {
    // FIv I-5 marker test: the non-overlapping windows invariant
    // (no two installments for the same fees_master may have
    // overlapping due_date windows) is dispatcher-enforced.
    //
    // The exact window definition is dispatcher-implemented
    // (e.g., each installment covers [due_date - lead_days,
    // due_date + grace_days]); for now we document that the
    // dispatcher must enforce non-overlap on create. The
    // aggregate itself cannot enforce this without a
    // sibling-row aggregation.

    // The aggregate itself cannot enforce this invariant
    // without a sibling-row aggregation, so we document the
    // dispatcher's responsibility here.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let _first = RealFeesInstallment::fresh(
        fiv_id(&g, school),
        master_id(&g, school),
        "First installment".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        5_000,
        Currency::INR,
        50,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("first installment constructs with any due_date");
    let _ = _first;
}
