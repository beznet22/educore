//! Integration tests for the **BankStatement aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 85 per-aggregate drop
//! [`RealBankStatement`](educore_finance::aggregate::RealBankStatement) —
//! the per-account transaction log row. Validates 4 invariants:
//! BS I-1 (`amount_minor >= 0` pinned at construction + on update),
//! BS I-2 (`statement_type` ∈ {Income, Expense}, enforced at
//! type-system level via the `StatementType` enum — cannot construct
//! with an invalid variant), BS I-3 (`balance_after_minor >= 0`
//! lower bound pinned at construction + on update; cross-statement
//! running balance consistency is the dispatcher's responsibility),
//! and BS I-4 (append-only; corrections happen via a new
//! opposite-direction row, NOT content mutation of the original).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `BankStatement` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! BankStatement { _id: () } }` placeholder. Wave 85 adds the
//! `RealBankStatement` aggregate (full lifecycle: fresh +
//! update_metadata + retire), the 4 headline events (Created /
//! Updated / Reversed / Retired), the 4 service functions
//! (create / update / reverse / retire), and this test suite.
//! Structurally parallel to Wave 74 COA / Wave 78 FCFA full-lifecycle
//! pattern + Wave 84 BankStatementAttachment extend-existing-struct
//! pattern (but BankStatement is GREENFIELD — no partial impl
//! existed in entities.rs).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::{
    CreateBankStatementCommand, RetireBankStatementCommand, ReverseBankStatementCommand,
    UpdateBankStatementCommand,
};
use educore_finance::events::{
    BankStatementCreated, BankStatementReversed, BankStatementRetired, BankStatementUpdated,
};
use educore_finance::prelude::RealBankStatement;
use educore_finance::services::{
    create_bank_statement, retire_bank_statement, reverse_bank_statement, update_bank_statement,
};
use educore_finance::value_objects::{BankAccountId, BankStatementId, Currency, StatementType};

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

fn bank_statement_id(g: &SystemIdGen, school: SchoolId) -> BankStatementId {
    BankStatementId::new(school, g.next_uuid())
}

fn bank_account_id(g: &SystemIdGen, school: SchoolId) -> BankAccountId {
    BankAccountId::new(school, g.next_uuid())
}

fn make_bank_statement(
    g: &SystemIdGen,
    school: SchoolId,
    bank: BankAccountId,
    statement_type: StatementType,
    amount: i64,
    balance_after: i64,
) -> RealBankStatement {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    RealBankStatement::fresh(
        bank_statement_id(g, school),
        bank,
        statement_type,
        amount,
        balance_after,
        Currency::INR,
        now,
        Some("test entry".to_owned()),
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh RealBankStatement")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 85 stub tests)
// =========================================================================

#[test]
fn bank_statement_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bank_statement_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn bank_statement_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = bank_statement_id(&g, school);
    let id_b = bank_statement_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealBankStatement::fresh — BS I-1 + BS I-2 + BS I-3
// =========================================================================

#[test]
fn fresh_pins_income_statement_with_balance() {
    // BS I-1 + BS I-2 + BS I-3: fresh row pins amount >= 0 +
    // statement_type + balance_after >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let row = make_bank_statement(&g, school, bank, StatementType::Income, 50_000, 150_000);
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(row.balance_after_minor, 150_000);
    assert_eq!(row.statement_type, StatementType::Income);
    assert_eq!(row.bank_account_id, bank);
    assert!(row.is_active());
}

#[test]
fn fresh_zero_amount_and_zero_balance_is_valid() {
    // BS I-1 + BS I-3 lower bounds use >= 0 (not > 0); zero is
    // valid (e.g. opening balance row with no transaction yet).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let row = make_bank_statement(&g, school, bank, StatementType::Income, 0, 0);
    assert_eq!(row.amount_minor, 0);
    assert_eq!(row.balance_after_minor, 0);
    assert!(row.is_active());
}

#[test]
fn fresh_rejects_negative_amount() {
    // BS I-1: amount_minor must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealBankStatement::fresh(
        bank_statement_id(&g, school),
        bank,
        StatementType::Income,
        -1,
        0,
        Currency::INR,
        now,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("negative amount must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_rejects_negative_balance() {
    // BS I-3: balance_after_minor must be >= 0 (lower bound).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealBankStatement::fresh(
        bank_statement_id(&g, school),
        bank,
        StatementType::Income,
        100,
        -1,
        Currency::INR,
        now,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("negative balance must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_supports_expense_statement_type() {
    // BS I-2: StatementType enum has Income + Expense variants only;
    // both are accepted by fresh().
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let row = make_bank_statement(&g, school, bank, StatementType::Expense, 25_000, 75_000);
    assert_eq!(row.statement_type, StatementType::Expense);
    assert_eq!(row.amount_minor, 25_000);
    assert_eq!(row.balance_after_minor, 75_000);
}

#[test]
fn fresh_trims_description_and_drops_empty() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let row = make_bank_statement(&g, school, bank, StatementType::Income, 100, 1100);
    assert_eq!(row.description.as_deref(), Some("test entry"));
    // Test trimming + empty filtering.
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let row2 = RealBankStatement::fresh(
        bank_statement_id(&g, school),
        bank,
        StatementType::Income,
        100,
        1200,
        Currency::INR,
        now,
        Some("  pad me  ".to_owned()),
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("trim is OK");
    assert_eq!(row2.description.as_deref(), Some("pad me"));
    let row3 = RealBankStatement::fresh(
        bank_statement_id(&g, school),
        bank,
        StatementType::Income,
        100,
        1300,
        Currency::INR,
        now,
        Some("   ".to_owned()),
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("empty-after-trim is OK");
    assert_eq!(row3.description, None);
}

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let before = SystemClock.now();
    let row = make_bank_statement(&g, school, bank, StatementType::Income, 100, 1100);
    let after = SystemClock.now();
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
    assert_eq!(row.created_by, row.updated_by);
    assert!(row.last_event_id.is_none());
}

// =========================================================================
// RealBankStatement::update_metadata — BS I-1 + BS I-3 re-validation
// =========================================================================

#[test]
fn update_metadata_updates_description_and_preserves_amount() {
    // BS I-4: update_metadata only allows description changes;
    // amount_minor + balance_after_minor + statement_type are
    // immutable here.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Income, 50_000, 150_000);
    let original_amount = row.amount_minor;
    let original_balance = row.balance_after_minor;
    let original_type = row.statement_type;
    let original_version = row.version;
    let later = SystemClock.now();
    row.update_metadata(Some("revised entry".to_owned()), later, g.next_user_id())
        .expect("update");
    assert_eq!(row.description.as_deref(), Some("revised entry"));
    // BS I-4: amount + balance + type preserved.
    assert_eq!(row.amount_minor, original_amount);
    assert_eq!(row.balance_after_minor, original_balance);
    assert_eq!(row.statement_type, original_type);
    assert!(row.version > original_version);
}

#[test]
fn update_metadata_rejects_on_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Income, 100, 1100);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    let later = SystemClock.now();
    let err = row
        .update_metadata(Some("revised".to_owned()), later, g.next_user_id())
        .expect_err("update on retired must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// RealBankStatement::retire — BS I-4 tombstone
// =========================================================================

#[test]
fn retire_flips_active_status_and_preserves_amount_balance_type() {
    // BS I-4: retire is a tombstone — original amount + balance +
    // statement_type are preserved in the audit footer.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Expense, 25_000, 75_000);
    let before = row.version;
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    assert!(!row.is_active());
    // BS I-4: amount + balance + type preserved.
    assert_eq!(row.amount_minor, 25_000);
    assert_eq!(row.balance_after_minor, 75_000);
    assert_eq!(row.statement_type, StatementType::Expense);
    assert_eq!(row.updated_at, now);
    assert!(row.version > before);
}

#[test]
fn retire_rejects_double_retire() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Income, 100, 1100);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("first retire");
    let err = row
        .retire(now, g.next_user_id())
        .expect_err("double retire must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// Service functions — 4 functions
// =========================================================================

#[test]
fn create_service_produces_aggregate_and_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bank_statement_id(&g, school);
    let bank = bank_account_id(&g, school);
    let now = SystemClock.now();
    let cmd = CreateBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: id,
        bank_account_id: bank,
        statement_type: StatementType::Income,
        amount_minor: 50_000,
        balance_after_minor: 150_000,
        currency: Currency::INR,
        occurred_at: now,
        description: Some("monthly fees".to_owned()),
    };
    let clock = SystemClock;
    let (row, event) = create_bank_statement(cmd, &clock, &g)
        .expect("create_bank_statement should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.bank_account_id, bank);
    assert_eq!(row.statement_type, StatementType::Income);
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(row.balance_after_minor, 150_000);
    assert!(row.is_active());
    assert_eq!(event.bank_statement_id, id);
    assert_eq!(
        <BankStatementCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement.created"
    );
    assert_eq!(
        <BankStatementCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement"
    );
    assert_eq!(
        <BankStatementCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id(), id.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn create_service_propagates_negative_amount_validation() {
    // BS I-1 propagation.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bank_statement_id(&g, school);
    let bank = bank_account_id(&g, school);
    let now = SystemClock.now();
    let cmd = CreateBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: id,
        bank_account_id: bank,
        statement_type: StatementType::Income,
        amount_minor: -1,
        balance_after_minor: 100,
        currency: Currency::INR,
        occurred_at: now,
        description: None,
    };
    let clock = SystemClock;
    let err = create_bank_statement(cmd, &clock, &g)
        .expect_err("negative amount must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn create_service_propagates_negative_balance_validation() {
    // BS I-3 propagation.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = bank_statement_id(&g, school);
    let bank = bank_account_id(&g, school);
    let now = SystemClock.now();
    let cmd = CreateBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: id,
        bank_account_id: bank,
        statement_type: StatementType::Income,
        amount_minor: 100,
        balance_after_minor: -1,
        currency: Currency::INR,
        occurred_at: now,
        description: None,
    };
    let clock = SystemClock;
    let err = create_bank_statement(cmd, &clock, &g)
        .expect_err("negative balance must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn update_service_updates_description_and_emits_event() {
    // BS I-4: update only allows description changes; amount +
    // balance + type are preserved (immutable here).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Income, 50_000, 150_000);
    let cmd = UpdateBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: row.id,
        description: Some("revised description".to_owned()),
    };
    let clock = SystemClock;
    let event = update_bank_statement(cmd, &clock, &g, &mut row)
        .expect("update_bank_statement should succeed");
    assert_eq!(row.description.as_deref(), Some("revised description"));
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(
        <BankStatementUpdated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement.updated"
    );
    assert_eq!(
        <BankStatementUpdated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement"
    );
    assert_eq!(
        <BankStatementUpdated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    // BS I-4: amount + balance + type unchanged after update.
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(row.balance_after_minor, 150_000);
    assert_eq!(row.statement_type, StatementType::Income);
}

#[test]
fn reverse_service_emits_reversed_event_without_mutating_original() {
    // BS I-4: reverse is the correction mechanism — the dispatcher
    // creates a new opposite-direction row; this service function
    // emits the BankStatementReversed event marking the original as
    // corrected. The original row is NOT mutated.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let original_id = bank_statement_id(&g, school);
    let reverse_id = bank_statement_id(&g, school);
    let cmd = ReverseBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: original_id,
        reverse_row_id: reverse_id,
    };
    let clock = SystemClock;
    let event = reverse_bank_statement(cmd, &clock, &g)
        .expect("reverse_bank_statement should succeed");
    assert_eq!(event.bank_statement_id, original_id);
    assert_eq!(event.reverse_row_id, reverse_id);
    assert_eq!(
        <BankStatementReversed as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement.reversed"
    );
    assert_eq!(
        <BankStatementReversed as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement"
    );
    assert_eq!(
        <BankStatementReversed as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}

#[test]
fn retire_service_flips_active_status_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let bank = bank_account_id(&g, school);
    let mut row = make_bank_statement(&g, school, bank, StatementType::Expense, 25_000, 75_000);
    let cmd = RetireBankStatementCommand {
        tenant: tenant.clone(),
        bank_statement_id: row.id,
    };
    let clock = SystemClock;
    let event = retire_bank_statement(cmd, &clock, &g, &mut row)
        .expect("retire_bank_statement should succeed");
    assert!(!row.is_active());
    // BS I-4: original payload preserved.
    assert_eq!(row.amount_minor, 25_000);
    assert_eq!(row.balance_after_minor, 75_000);
    assert_eq!(row.statement_type, StatementType::Expense);
    assert_eq!(row.last_event_id, Some(event.event_id));
    assert_eq!(
        <BankStatementRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement.retired"
    );
    assert_eq!(
        <BankStatementRetired as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement"
    );
    assert_eq!(
        <BankStatementRetired as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
}
