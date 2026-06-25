//! Integration tests for the **Designation aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Designation`](educore_hr::aggregate::Designation)
//! end-to-end through the service layer:
//!
//! 1. `create_designation` validates the input (title must be
//!    1..=200 chars), constructs the aggregate (school id
//!    derived from the typed id), and emits a
//!    [`DesignationCreated`] event with the right `event_type`,
//!    `aggregate_type`, `school_id`, and aggregate id.
//! 2. The validation failure path (empty title) returns
//!    `DomainError::Validation` and does not emit an event.
//!
//! The tests use the same fixture pattern as
//! `tests/workflows.rs` (`TestClock` + `SystemIdGen`) plus a
//! no-op `ReferenceDataUniquenessChecker` mock (the storage
//! adapter is the canonical implementation). The handlers /
//! outbox / audit fan-out are not yet wired end-to-end; these
//! tests pin the **service layer** contract that the
//! dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/hr/tests/department.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_hr::prelude::*;
use educore_hr::services::ReferenceDataUniquenessChecker;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin_context() -> TenantContext {
    let g = SystemIdGen;
    TenantContext::for_user(
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    )
}

/// No-op `ReferenceDataUniquenessChecker` mock: every name /
/// title is reported as unique. Mirrors the in-test pattern
/// used by `services.rs`'s own `StubRefUniqueness` stub.
struct NoOpRefUniqueness;
impl ReferenceDataUniquenessChecker for NoOpRefUniqueness {
    fn department_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
    fn designation_title_exists(&self, _school: SchoolId, _title: &str) -> bool {
        false
    }
    fn leave_type_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
}

// =============================================================================
// Happy path: create a designation
// =============================================================================

/// End-to-end happy path for the Designation aggregate.
/// Create a designation titled "Principal" with a short
/// description, asserting that:
///
/// 1. The create flow produces a `Designation` aggregate
///    carrying the title + description from the call args
///    (school id derived from the typed id), with the audit
///    footer initialised (`version == 1`, active).
/// 2. The emitted `DesignationCreated` event carries the right
///    `event_type`, `aggregate_type`, `schema_version`,
///    `aggregate_id`, `school_id`, and `title`.
#[test]
fn create_designation_returns_aggregate_and_event() {
    let tenant = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (desig, event) = create_designation(
        tenant.clone(),
        "Principal".to_owned(),
        Some("Head of school".to_owned()),
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect("create designation");

    // Aggregate fields are populated from the command.
    assert_eq!(desig.school_id, school);
    assert_eq!(desig.title, "Principal");
    assert_eq!(desig.description.as_deref(), Some("Head of school"));
    assert_eq!(desig.created_by, tenant.actor_id);
    assert_eq!(desig.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised at version 1 and
    // active.
    assert_eq!(desig.version.get(), 1);
    assert!(desig.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <DesignationCreated as DomainEvent>::EVENT_TYPE,
        "hr.designation.created"
    );
    assert_eq!(
        <DesignationCreated as DomainEvent>::AGGREGATE_TYPE,
        "designation"
    );
    assert_eq!(<DesignationCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), desig.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.title, "Principal");
    assert_eq!(event.designation_id, desig.id);
}

// =============================================================================
// Validation failure: empty title
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `title` is empty, `create_designation` returns
/// `DomainError::Validation`. The service returns the error
/// directly (no aggregate is produced, no event is minted) so
/// there is nothing to assert on the aggregate side.
#[test]
fn create_designation_rejects_empty_title() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_designation(
        tenant,
        String::new(),
        Some("no title".to_owned()),
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect_err("empty title must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
