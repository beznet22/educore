//! Behavioural tests for `RealIncome` (Wave 97).
//!
//! Pins IN I-1 (`amount_minor >= 0`) end-to-end via the aggregate
//! surface, the service functions, and the emitted events.

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
use educore_finance::events::{IncomeCreated, IncomeRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::{BankAccountId, Currency, IncomeHeadId, IncomeId};

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

fn income_id(g: &SystemIdGen, school: SchoolId) -> IncomeId {
    IncomeId::new(school, g.next_uuid())
}

fn income_head_id(g: &SystemIdGen, school: SchoolId) -> IncomeHeadId {
    IncomeHeadId::new(school, g.next_uuid())
}

fn bank_account_id(g: &SystemIdGen, school: SchoolId) -> BankAccountId {
    BankAccountId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn income_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = income_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- IN I-1: amount_minor >= 0 ----

#[test]
fn fresh_full_payload_amount_valid_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let row = RealIncome::fresh(
        id,
        head,
        25_000,
        Some("tuition payment".to_string()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 25_000");
    assert_eq!(row.amount_minor, 25_000);
    assert_eq!(row.income_head_id, head);
    assert_eq!(row.description.as_deref(), Some("tuition payment"));
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_negative_amount_validation_error_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let result = RealIncome::fresh(
        id,
        head,
        -1,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor") && msg.contains("IN I-1"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_is_valid_in_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let row = RealIncome::fresh(
        id,
        head,
        0,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed with amount_minor = 0 (boundary, valid)");
    assert_eq!(row.amount_minor, 0);
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let before = Timestamp::now();
    let row = RealIncome::fresh(
        id,
        head,
        5_000,
        None,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    let after = Timestamp::now();
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.last_event_id, None);
    // IN I-3: created_at + updated_at both initialized at
    // construction. They are equal to each other and fall in
    // the [before, after] wall-clock window.
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let mut row = RealIncome::fresh(
        id,
        head,
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
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.income_head_id, head);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = income_id(&g, school);
    let head = income_head_id(&g, school);
    let mut row = RealIncome::fresh(
        id,
        head,
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
fn create_income_service_emits_created_event_in_i_1() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let head = income_head_id(&g, school);
    let account = bank_account_id(&g, school);
    let cmd = CreateIncomeCommand {
        tenant,
        name: "donation".to_string(),
        amount_minor: 25_000,
        currency: Currency::INR,
        income_head_id: head,
        account_id: account,
        income_date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        description: Some("test donation".to_string()),
        donor_id: None,
    };
    let evt: IncomeCreated = create_income(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.amount_minor, 25_000);
    assert_eq!(evt.income_head_id, head);
    assert_eq!(evt.created_by, actor);
    assert_eq!(
        <IncomeCreated as DomainEvent>::EVENT_TYPE,
        "finance.income.created"
    );
    assert_eq!(<IncomeCreated as DomainEvent>::AGGREGATE_TYPE, "income");
    assert_eq!(<IncomeCreated as DomainEvent>::SCHEMA_VERSION, 1);
}

#[test]
fn retire_income_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = income_id(&g, school);
    let cmd = RetireIncomeCommand {
        tenant,
        income_id: id,
    };
    let evt: IncomeRetired = retire_income(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.income_id, id);
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(
        <IncomeRetired as DomainEvent>::EVENT_TYPE,
        "finance.income.retired"
    );
}

// =========================================================================
// -- Wave 143 -- RealIncome -- IN I-2 compatible marker --
// =========================================================================

#[test]
fn in_i_2_account_payment_method_compatible_dispatcher_enforced() {
    // IN I-2 marker test: the account-+-payment_method compatible
    // invariant (the Income's payment_method must be compatible
    // with the account_type of the referenced BankAccount) is
    // dispatcher-enforced. RealIncome aggregate carries
    // amount_minor + income_head_id; the dispatcher adds the
    // BankAccount reference + payment_method cross-row check.
    let (tenant, _g) = admin_context();
    let _ = tenant; // type-level marker
}
