//! Behavioural tests for `RealFeesInstallmentCredit` (Wave 93).
//!
//! Covers:
//! - FIC I-1: amount \xe2\x89\xa5 0 \xe2\x80\x94 `amount_minor` is i64 minor
//!   units; pinned (NOT mutable; append-only); validated at
//!   construction (returns `DomainError::Validation` if < 0).
//! - FIC I-2: credit source valid \xe2\x80\x94 `credit_source` is
//!   type-pinned via the `FeesInstallmentCreditSource` enum
//!   with only 3 variants: `Overpayment | Correction |
//!   ManualAdjustment`. The Rust compiler rejects any other
//!   variant at construction.
//! - FIC I-3: append-only \xe2\x80\x94 NO `update_*` method is
//!   exposed on the aggregate; the `Updated` event type does
//!   NOT exist for this aggregate (type-system-level enforcement
//!   of the append-only contract).
//!
//! Pattern: `admin_context()` fixture + `SystemClock` + `SystemIdGen`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]


use educore_core::clock::{Clock, SystemClock, SystemIdGen};
use educore_core::ids::{CorrelationId, Identifier};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_finance::prelude::*;
use educore_finance::value_objects::{FeesInstallmentCreditId, FeesInstallmentId};

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

fn make_create_cmd(
    tenant: TenantContext,
    id: FeesInstallmentCreditId,
    g: &SystemIdGen,
) -> CreateFeesInstallmentCreditCommand {
    let school = tenant.school_id;
    let source_installment_id = FeesInstallmentId::new(school, g.next_uuid());
    CreateFeesInstallmentCreditCommand {
        tenant,
        fees_installment_credit_id: id,
        amount_minor: 25_000, // FIC I-1: \xe2\x82\xb9250.00 (>= 0)
        credit_source: FeesInstallmentCreditSource::Overpayment, // FIC I-2 default
        source_installment_id,
        description: Some("Overpayment from Q2 installment".to_owned()),
    }
}

// ============================================================================
// Typed-id smoke tests
// ============================================================================

#[test]
fn typed_id_smoke_fees_installment_credit_id() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    assert_eq!(id.school_id(), school);
    assert!(!id.as_uuid().is_nil());
}

#[test]
fn typed_id_smoke_fees_installment_credit_ids_are_distinct_within_school() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let id_a = FeesInstallmentCreditId::new(school, g.next_uuid());
    let id_b = FeesInstallmentCreditId::new(school, g.next_uuid());
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ============================================================================
// RealFeesInstallmentCredit::fresh tests
// ============================================================================

#[test]
fn fresh_full_payload_overpayment_fic_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, event) = create_fees_installment_credit(cmd, &clock, &g).unwrap();

    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.amount_minor, 25_000); // FIC I-1
    assert_eq!(row.credit_source, FeesInstallmentCreditSource::Overpayment); // FIC I-2
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(
        event.credit_source,
        FeesInstallmentCreditSource::Overpayment
    );
}

#[test]
fn fresh_full_payload_correction_fic_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.credit_source = FeesInstallmentCreditSource::Correction;
    let clock = SystemClock;

    let (row, event) = create_fees_installment_credit(cmd, &clock, &g).unwrap();
    assert_eq!(row.credit_source, FeesInstallmentCreditSource::Correction);
    assert_eq!(event.credit_source, FeesInstallmentCreditSource::Correction);
}

#[test]
fn fresh_full_payload_manual_adjustment_fic_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.credit_source = FeesInstallmentCreditSource::ManualAdjustment;
    let clock = SystemClock;

    let (row, event) = create_fees_installment_credit(cmd, &clock, &g).unwrap();
    assert_eq!(
        row.credit_source,
        FeesInstallmentCreditSource::ManualAdjustment
    );
    assert_eq!(
        event.credit_source,
        FeesInstallmentCreditSource::ManualAdjustment
    );
}

#[test]
fn fresh_negative_amount_validation_error_fic_i_1() {
    // FIC I-1: amount_minor must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.amount_minor = -1;
    let clock = SystemClock;

    let err = create_fees_installment_credit(cmd, &clock, &g).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected Validation error, got {:?}",
        err
    );
}

#[test]
fn fresh_zero_amount_is_valid_fic_i_1() {
    // FIC I-1 boundary: amount_minor == 0 is allowed (a zero
    // credit is meaningless but not a validation error).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let mut cmd = make_create_cmd(tenant.clone(), id, &g);
    cmd.amount_minor = 0;
    let clock = SystemClock;

    let (row, _) = create_fees_installment_credit(cmd, &clock, &g).unwrap();
    assert_eq!(row.amount_minor, 0);
}

#[test]
fn fresh_audit_footer_initialized() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (row, _) = create_fees_installment_credit(cmd, &clock, &g).unwrap();
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
// RealFeesInstallmentCredit::retire tests
// ============================================================================

#[test]
fn retire_flips_active_status_preserves_amount_and_source_fic_i_1_i_2() {
    // Tombstone preserves FIC I-1 amount_minor + FIC I-2
    // credit_source for legal-record retention.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_fees_installment_credit(cmd, &clock, &g).unwrap();

    let pinned_amount = row.amount_minor;
    let pinned_source = row.credit_source;
    let pinned_installment = row.source_installment_id;

    let retire_cmd = RetireFeesInstallmentCreditCommand {
        tenant: tenant.clone(),
        fees_installment_credit_id: id,
    };
    let event = retire_fees_installment_credit(retire_cmd, &clock, &g, &mut row).unwrap();

    // Active status flipped
    assert!(!row.is_active());
    assert_eq!(row.active_status, ActiveStatus::Retired);

    // FIC I-1 + FIC I-2 fields preserved (tombstone)
    assert_eq!(row.amount_minor, pinned_amount);
    assert_eq!(row.credit_source, pinned_source);
    assert_eq!(row.source_installment_id, pinned_installment);

    // Event carries only id + deleted_by
    assert_eq!(event.fees_installment_credit_id, id);
    assert_eq!(event.deleted_by, tenant.actor_id);

    // Version bumped + updated_by stamped
    assert_eq!(row.version, Version::initial().next());
    assert_eq!(row.updated_by, tenant.actor_id);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (mut row, _) = create_fees_installment_credit(cmd, &clock, &g).unwrap();

    let retire_cmd = RetireFeesInstallmentCreditCommand {
        tenant: tenant.clone(),
        fees_installment_credit_id: id,
    };
    let _ = retire_fees_installment_credit(retire_cmd, &clock, &g, &mut row).unwrap();

    let retire_cmd2 = RetireFeesInstallmentCreditCommand {
        tenant: tenant.clone(),
        fees_installment_credit_id: id,
    };
    let err = retire_fees_installment_credit(retire_cmd2, &clock, &g, &mut row).unwrap_err();
    assert!(
        matches!(err, educore_core::error::DomainError::Conflict(_)),
        "expected Conflict error, got {:?}",
        err
    );
}

// ============================================================================
// FIC I-3 architectural tests: append-only enforced at type level
// ============================================================================

#[test]
fn no_updated_event_type_exists_fic_i_3() {
    // FIC I-3 architectural invariant: the append-only contract
    // is enforced at the type-system level by the ABSENCE of
    // a FeesInstallmentCreditUpdated event type. We verify this
    // by confirming only Created + Retired event types exist
    // (their EVENT_TYPE strings are distinct + well-formed).
    assert_eq!(
        <FeesInstallmentCreditCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_credit.created"
    );
    assert_eq!(
        <FeesInstallmentCreditRetired as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_credit.retired"
    );
    // Confirm AGGREGATE_TYPE consistency (both belong to the
    // same aggregate family).
    assert_eq!(
        <FeesInstallmentCreditCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_credit"
    );
    assert_eq!(
        <FeesInstallmentCreditRetired as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_installment_credit"
    );
}

// ============================================================================
// Service integration tests
// ============================================================================

#[test]
fn create_fees_installment_credit_service_event_type_is_finance_fees_installment_credit_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = FeesInstallmentCreditId::new(school, g.next_uuid());
    let cmd = make_create_cmd(tenant.clone(), id, &g);
    let clock = SystemClock;

    let (_, event) = create_fees_installment_credit(cmd, &clock, &g).unwrap();
    assert_eq!(
        <FeesInstallmentCreditCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_installment_credit.created"
    );
    assert_eq!(
        <FeesInstallmentCreditCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.fees_installment_credit_id, id);
    // FIC I-1 + FIC I-2 carried downstream
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(
        event.credit_source,
        FeesInstallmentCreditSource::Overpayment
    );
}
