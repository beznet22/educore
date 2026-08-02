//! Behavioural tests for `RealBankAccount` (Wave 87).
//!
//! Covers:
//! - BA I-1: account_number unique (pinned + non-empty trimmed at
//!   construction; NOT mutable via update_metadata; dispatcher
//!   enforces (school_id, account_number) uniqueness at storage
//!   layer)
//! - BA I-2: current_balance derived from BankStatement (STRUCTURAL
//!   enforcement via absence of `current_balance_minor` field —
//!   `opening_balance_minor` is immutable post-creation; running
//!   balance is derived from BankStatement rows)
//! - BA I-3: account_type ∈ {bank, cash} (typed `AccountType` enum
//!   parameter; compiler rejects any variant other than `Bank` or
//!   `Cash`)
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.

use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::BankAccountId;

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

fn make_open_cmd(
    tenant: TenantContext,
    id: BankAccountId,
    g: &SystemIdGen,
) -> OpenBankAccountCommand {
    OpenBankAccountCommand {
        tenant,
        bank_account_id: id,
        account_name: "HDFC Operating Account".to_owned(),
        account_number: "50100123456789".to_owned(),
        account_type: AccountType::Bank,
        bank_name: "HDFC Bank".to_owned(),
        ifsc_code: Some("HDFC0000123".to_owned()),
        branch: Some("Koramangala".to_owned()),
        opening_balance_minor: 500_000, // ₹5,000.00
        currency: Currency::INR,
        description: Some("Primary operating account".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests (parallel to Wave 86's typed_id_smoke_* tests)
// ============================================================================

#[test]
fn typed_id_smoke_bank_account_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = BankAccountId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_bank_account_id_unique() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id1 = BankAccountId::new(school, g.next_uuid());
    let id2 = BankAccountId::new(school, g.next_uuid());
    assert_ne!(id1, id2);
    assert_eq!(id1.school_id(), school);
    assert_eq!(id2.school_id(), school);
}

// ============================================================================
// RealBankAccount::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload_bank_type() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, event) = open_bank_account(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.account_name, "HDFC Operating Account");
    assert_eq!(row.account_number, "50100123456789");
    assert_eq!(row.account_type, AccountType::Bank);
    assert!(row.is_bank());
    assert!(!row.is_cash());
    assert_eq!(row.bank_name, "HDFC Bank");
    assert_eq!(row.ifsc_code.as_deref(), Some("HDFC0000123"));
    assert_eq!(row.branch.as_deref(), Some("Koramangala"));
    assert_eq!(row.opening_balance_minor, 500_000);
    assert_eq!(row.currency, Currency::INR);
    assert_eq!(
        row.description.as_deref(),
        Some("Primary operating account")
    );
    assert_eq!(event.bank_account_id, id);
    assert_eq!(event.account_number, "50100123456789");
    assert_eq!(event.account_type, AccountType::Bank);
    assert_eq!(event.opening_balance_minor, 500_000);
}

#[test]
fn fresh_supports_cash_account_type() {
    // BA I-3: Cash variant pinned at type-system level
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let mut cmd = make_open_cmd(tenant.clone(), id, &g);
    cmd.account_type = AccountType::Cash;
    cmd.account_number = "CASH-DRAWER-01".to_owned();
    let clock = SystemClock;

    let (row, event) = open_bank_account(cmd, &clock, &g).unwrap();
    assert_eq!(row.account_type, AccountType::Cash);
    assert!(row.is_cash());
    assert!(!row.is_bank());
    assert_eq!(event.account_type, AccountType::Cash);
}

#[test]
fn fresh_empty_account_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let mut cmd = make_open_cmd(tenant.clone(), id, &g);
    cmd.account_name = "   ".to_owned(); // whitespace only → trims to empty
    let clock = SystemClock;

    let err = open_bank_account(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_empty_account_number_validation_error() {
    // BA I-1: account_number is the uniqueness anchor + non-empty guard
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let mut cmd = make_open_cmd(tenant.clone(), id, &g);
    cmd.account_number = "  ".to_owned(); // whitespace only → trims to empty
    let clock = SystemClock;

    let err = open_bank_account(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_empty_bank_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let mut cmd = make_open_cmd(tenant.clone(), id, &g);
    cmd.bank_name = "  ".to_owned(); // whitespace only → trims to empty
    let clock = SystemClock;

    let err = open_bank_account(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_audit_footer_initialized() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, _) = open_bank_account(cmd, &clock, &g).unwrap();
    assert_eq!(row.version, Version::initial());
    assert_eq!(row.active_status, ActiveStatus::Active);
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.created_at, row.updated_at);
    assert_eq!(row.correlation_id, tenant.correlation_id);
    assert!(row.last_event_id.is_some());
    assert_eq!(row.etag, Etag::placeholder());
}

// ============================================================================
// RealBankAccount::update_metadata tests
// ============================================================================

#[test]
fn update_metadata_mutates_mutable_fields_only() {
    // BA I-1 + BA I-2 + BA I-3 structural: account_number +
    // opening_balance_minor + account_type + currency are NOT
    // mutable; only account_name + bank_name + ifsc_code + branch
    // + description change.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = open_bank_account(cmd, &clock, &g).unwrap();

    // Capture immutable fields (BA I-1 + BA I-2 + BA I-3 + currency)
    let pinned_number = row.account_number.clone();
    let pinned_type = row.account_type;
    let pinned_opening = row.opening_balance_minor;
    let pinned_currency = row.currency;

    // Mutable-only update
    let update_cmd = UpdateBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
        account_name: "HDFC Operating Account v2".to_owned(),
        bank_name: "HDFC Bank Ltd".to_owned(),
        ifsc_code: Some("HDFC0000999".to_owned()),
        branch: Some("Indiranagar".to_owned()),
        description: Some("Renamed operating account".to_owned()),
    };
    let event = update_bank_account(update_cmd, &clock, &g, &mut row).unwrap();

    // Mutable fields DID change
    assert_eq!(row.account_name, "HDFC Operating Account v2");
    assert_eq!(row.bank_name, "HDFC Bank Ltd");
    assert_eq!(row.ifsc_code.as_deref(), Some("HDFC0000999"));
    assert_eq!(row.branch.as_deref(), Some("Indiranagar"));
    assert_eq!(
        row.description.as_deref(),
        Some("Renamed operating account")
    );
    assert_eq!(event.account_name, "HDFC Operating Account v2");
    assert_eq!(event.bank_name, "HDFC Bank Ltd");

    // BA I-1: account_number preserved (uniqueness anchor)
    assert_eq!(row.account_number, pinned_number);
    // BA I-3: account_type preserved (type-system pinned)
    assert_eq!(row.account_type, pinned_type);
    // BA I-2: opening_balance_minor preserved (structural — running
    // balance derived from BankStatement rows, NOT from this field)
    assert_eq!(row.opening_balance_minor, pinned_opening);
    // currency preserved (changing currency = different account)
    assert_eq!(row.currency, pinned_currency);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
    assert!(row.last_event_id.is_some());
}

#[test]
fn update_metadata_on_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = open_bank_account(cmd, &clock, &g).unwrap();

    // Retire first
    let retire_cmd = DeleteBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
    };
    let _retire_event = retire_bank_account(retire_cmd, &clock, &g, &mut row).unwrap();
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // Now try to update_metadata on retired row
    let update_cmd = UpdateBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
        account_name: "HDFC Operating Account v3".to_owned(),
        bank_name: "HDFC Bank".to_owned(),
        ifsc_code: None,
        branch: None,
        description: None,
    };
    let err = update_bank_account(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

#[test]
fn update_metadata_empty_account_name_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = open_bank_account(cmd, &clock, &g).unwrap();

    let update_cmd = UpdateBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
        account_name: "   ".to_owned(), // whitespace only
        bank_name: "HDFC".to_owned(),
        ifsc_code: None,
        branch: None,
        description: None,
    };
    let err = update_bank_account(update_cmd, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

// ============================================================================
// RealBankAccount::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_immutable_fields() {
    // BA I-1 + BA I-2 + BA I-3 structural: tombstone preserves
    // account_number + opening_balance_minor + account_type +
    // currency for legal-record retention + uniqueness queries.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = open_bank_account(cmd, &clock, &g).unwrap();

    // Capture all fields before retire
    let pinned_number = row.account_number.clone();
    let pinned_type = row.account_type;
    let pinned_opening = row.opening_balance_minor;
    let pinned_currency = row.currency;
    let pinned_name = row.account_name.clone();

    let retire_cmd = DeleteBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
    };
    let event = retire_bank_account(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped to Retired
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // All BA I-1 + BA I-2 + BA I-3 fields preserved (tombstone)
    assert_eq!(row.account_number, pinned_number);
    assert_eq!(row.account_type, pinned_type);
    assert_eq!(row.opening_balance_minor, pinned_opening);
    assert_eq!(row.currency, pinned_currency);
    // account_name also preserved (not a BA I invariant but
    // audit-trail completeness)
    assert_eq!(row.account_name, pinned_name);

    // Event carries only bank_account_id (immutable fields are in
    // the audit footer of the aggregate, not the event)
    assert_eq!(event.bank_account_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = BankAccountId::new(school, g.next_uuid());
    let cmd = make_open_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = open_bank_account(cmd, &clock, &g).unwrap();

    let retire_cmd = DeleteBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
    };
    let _ = retire_bank_account(retire_cmd, &clock, &g, &mut row).unwrap();

    // Try to retire again
    let retire_cmd2 = DeleteBankAccountCommand {
        tenant: tenant.clone(),
        bank_account_id: id,
    };
    let err = retire_bank_account(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// Accessor tests (BA I-3 enum variant accessors)
// ============================================================================

#[test]
fn accessors_is_active_is_bank_is_cash() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_bank = BankAccountId::new(school, g.next_uuid());
    let mut cmd = make_open_cmd(tenant.clone(), id_bank, &g);
    cmd.account_type = AccountType::Bank;
    let clock = SystemClock;
    let (row_bank, _) = open_bank_account(cmd, &clock, &g).unwrap();

    assert!(row_bank.is_active());
    assert!(row_bank.is_bank());
    assert!(!row_bank.is_cash());

    let id_cash = BankAccountId::new(school, g.next_uuid());
    let mut cmd_cash = make_open_cmd(tenant.clone(), id_cash, &g);
    cmd_cash.account_type = AccountType::Cash;
    cmd_cash.account_number = "CASH-01".to_owned();
    let (row_cash, _) = open_bank_account(cmd_cash, &clock, &g).unwrap();
    assert!(row_cash.is_active());
    assert!(row_cash.is_cash());
    assert!(!row_cash.is_bank());
}
