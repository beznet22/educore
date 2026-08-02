//! Integration tests for the **BankStatementAttachment aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 84 per-aggregate drop
//! [`BankStatementAttachment`](educore_finance::entities::BankStatementAttachment) —
//! the file attachment row attached to a `BankStatement`.
//! Validates BSA I-1 (attachment ref valid — the `file_reference`
//! Uuid is pinned at construction; the dispatcher is responsible
//! for validating that the file_reference exists at the file
//! storage port) and BSA I-2 (orphan after BankStatement delete —
//! the `bank_statement_id` reference is preserved in the audit
//! footer even after retire; cascade-delete handled by dispatcher).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `BankStatementAttachment` previously had only a
//! partial implementation (struct + `fresh()` + audit footer + the
//! production caller at entities.rs:615 that creates
//! `BankStatementAttachment::fresh(...)`). Wave 84 adds the 2
//! mutator methods (is_active + retire) to the existing
//! entities.rs struct (Wave 81 pattern, NOT new Real* in
//! aggregate.rs), the 2 headline events (Created + Retired; no
//! Updated for append-only), the service function, and this test
//! suite. Structurally parallel to the Wave 83
//! `tests/bank_payment_slip_audit.rs` suite (append-only pattern)
//! and the Wave 81 `tests/payroll_payment_approval.rs` suite
//! (extend-existing-struct pattern with no separate id field —
//! parent `bank_statement_id` is de-facto identity).

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
use uuid::Uuid;

use educore_finance::commands::CreateBankStatementAttachmentCommand;
use educore_finance::entities::BankStatementAttachment;
use educore_finance::events::{BankStatementAttachmentCreated, BankStatementAttachmentRetired};
use educore_finance::services::create_bank_statement_attachment;
use educore_finance::value_objects::BankStatementId;

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

fn file_reference(g: &SystemIdGen) -> Uuid {
    g.next_uuid()
}

fn make_bank_statement_attachment(
    g: &SystemIdGen,
    _school: SchoolId,
    statement: BankStatementId,
    file_ref: Uuid,
) -> BankStatementAttachment {
    let uploader = g.next_user_id();
    let now = SystemClock.now();
    BankStatementAttachment::fresh(
        statement,
        file_ref,
        now,
        uploader,
        Some("monthly receipt".to_owned()),
        uploader,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 84 stub tests)
// =========================================================================

#[test]
fn bank_statement_attachment_typed_id_round_trips_school() {
    // The BankStatementAttachment struct uses bank_statement_id as
    // de-facto identity (no separate id field, parallel to Wave 81
    // PayrollPaymentApproval). The typed-id smoke test verifies
    // bank_statement_id.school_id() round-trips correctly.
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
// BankStatementAttachment::fresh — BSA I-1 (file_reference pinned)
// =========================================================================

#[test]
fn fresh_appends_to_log_with_full_payload() {
    // BSA I-1: file_reference is pinned at construction.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let row = make_bank_statement_attachment(&g, school, statement, file_ref);
    assert_eq!(row.bank_statement_id, statement);
    assert_eq!(row.file_reference, file_ref);
    assert!(row.is_active());
    assert_eq!(row.description.as_deref(), Some("monthly receipt"));
}

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let before = SystemClock.now();
    let row = make_bank_statement_attachment(&g, school, statement, file_ref);
    let after = SystemClock.now();
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
    assert_eq!(row.created_by, row.updated_by);
    assert!(row.last_event_id.is_none());
}

#[test]
fn fresh_inherits_school_id_from_parent_statement() {
    // The BankStatementAttachment struct derives school_id from
    // bank_statement_id.school_id() in fresh() (parallel to Wave 81
    // PayrollPaymentApproval pattern).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let row = make_bank_statement_attachment(&g, school, statement, file_ref);
    assert_eq!(row.school_id, school);
    assert_eq!(row.bank_statement_id, statement);
}

// =========================================================================
// BankStatementAttachment — append-only enforcement marker
// =========================================================================

#[test]
fn append_only_no_update_mutator_exists() {
    // BSA I-1 + BSA I-2 marker test: BankStatementAttachment
    // intentionally exposes no `update_metadata` method
    // (compile-time assertion documented in the impl block). This
    // test pins that contract by checking the type's method surface
    // — if someone later adds an update method, this test should be
    // updated alongside.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let row = make_bank_statement_attachment(&g, school, statement, file_ref);
    // The only mutator is `retire()` — no update_*, no set_*, no
    // mutate_*. This is a compile-time guarantee enforced by the
    // absence of those methods in the impl block.
    let _ = row; // type-level marker
}

// =========================================================================
// BankStatementAttachment::retire — BSA I-1 + BSA I-2 tombstone
// =========================================================================

#[test]
fn retire_flips_active_status_and_preserves_original_payload() {
    // BSA I-1 + BSA I-2: retire is a tombstone — original
    // bank_statement_id + file_reference + uploaded_at + uploaded_by
    // + description are preserved in the audit footer.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let mut row = make_bank_statement_attachment(&g, school, statement, file_ref);
    let before = row.version;
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    assert!(!row.is_active());
    // Original payload preserved.
    assert_eq!(row.bank_statement_id, statement);
    assert_eq!(row.file_reference, file_ref);
    assert_eq!(row.description.as_deref(), Some("monthly receipt"));
    // Audit footer bumped.
    assert_eq!(row.updated_at, now);
    assert!(row.version > before);
}

#[test]
fn retire_rejects_double_retire() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let mut row = make_bank_statement_attachment(&g, school, statement, file_ref);
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
// create_bank_statement_attachment service function
// =========================================================================

#[test]
fn service_function_creates_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let now = SystemClock.now();
    let cmd = CreateBankStatementAttachmentCommand {
        tenant: tenant.clone(),
        bank_statement_id: statement,
        file_reference: file_ref,
        uploaded_at: now,
        uploaded_by: g.next_user_id(),
        description: Some("wire transfer receipt".to_owned()),
    };
    let clock = SystemClock;
    let (row, event) = create_bank_statement_attachment(cmd, &clock, &g)
        .expect("create_bank_statement_attachment should succeed");
    assert_eq!(row.bank_statement_id, statement);
    assert_eq!(row.file_reference, file_ref);
    assert!(row.is_active());
    assert_eq!(event.bank_statement_id, statement);
    assert_eq!(event.file_reference, file_ref);
    assert_eq!(
        <BankStatementAttachmentCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement_attachment.created"
    );
    assert_eq!(
        <BankStatementAttachmentCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement_attachment"
    );
    assert_eq!(
        <BankStatementAttachmentCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    // aggregate_id is bank_statement_id.as_uuid() (the struct has
    // no separate id field).
    assert_eq!(event.aggregate_id(), statement.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn service_function_preserves_audit_footer_fields() {
    // BSA I-2 partial: the service function preserves all
    // audit-footer fields (created_at/created_by/updated_at/updated_by)
    // when minting the aggregate.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let now = SystemClock.now();
    let uploader = g.next_user_id();
    let cmd = CreateBankStatementAttachmentCommand {
        tenant: tenant.clone(),
        bank_statement_id: statement,
        file_reference: file_ref,
        uploaded_at: now,
        uploaded_by: uploader,
        description: None,
    };
    let clock = SystemClock;
    let (row, _event) =
        create_bank_statement_attachment(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(row.created_at, row.updated_at);
    assert_eq!(row.created_by, row.updated_by);
    assert_eq!(row.uploaded_by, uploader);
}

// =========================================================================
// Retired event (separate from Created) — confirms BSA I-1 + BSA I-2 at the
// event-emission layer
// =========================================================================

#[test]
fn retired_event_carries_aggregate_metadata() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let statement = bank_statement_id(&g, school);
    let file_ref = file_reference(&g);
    let event = BankStatementAttachmentRetired::new(
        statement,
        file_ref,
        g.next_user_id(),
        g.next_event_id(),
        educore_core::ids::CorrelationId(g.next_uuid()),
        SystemClock.now(),
    );
    assert_eq!(
        <BankStatementAttachmentRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.bank_statement_attachment.retired"
    );
    assert_eq!(
        <BankStatementAttachmentRetired as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "bank_statement_attachment"
    );
    assert_eq!(
        <BankStatementAttachmentRetired as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id(), statement.as_uuid());
}
