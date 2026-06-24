//! Integration tests for the **Department aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Department`](educore_hr::aggregate::Department)
//! end-to-end through the service layer:
//!
//! 1. `create_department` validates the input (name must be
//!    1..=200 chars), constructs the aggregate (school id
//!    derived from the typed id), and emits a
//!    [`DepartmentCreated`] event with the right `event_type`,
//!    `aggregate_type`, `school_id`, and aggregate id.
//! 2. The validation failure path (empty name) returns
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
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

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
// Happy path: create a department
// =============================================================================

/// End-to-end happy path for the Department aggregate.
/// Create a department called "Mathematics" with a short
/// description, asserting that:
///
/// 1. The create flow produces a `Department` aggregate
///    carrying the name + description from the call args
///    (school id derived from the typed id), with the audit
///    footer initialised (`version == 1`, active).
/// 2. The emitted `DepartmentCreated` event carries the right
///    `event_type`, `aggregate_type`, `schema_version`,
///    `aggregate_id`, `school_id`, and `name`.
#[test]
fn create_department_returns_aggregate_and_event() {
    let tenant = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (dept, event) = create_department(
        tenant.clone(),
        "Mathematics".to_owned(),
        Some("Math dept".to_owned()),
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect("create department");

    // Aggregate fields are populated from the command.
    assert_eq!(dept.school_id, school);
    assert_eq!(dept.name, "Mathematics");
    assert_eq!(dept.description.as_deref(), Some("Math dept"));
    assert_eq!(dept.created_by, tenant.actor_id);
    assert_eq!(dept.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised at version 1 and
    // active.
    assert_eq!(dept.version.get(), 1);
    assert!(dept.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <DepartmentCreated as DomainEvent>::EVENT_TYPE,
        "hr.department.created"
    );
    assert_eq!(
        <DepartmentCreated as DomainEvent>::AGGREGATE_TYPE,
        "department"
    );
    assert_eq!(<DepartmentCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), dept.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.name, "Mathematics");
    assert_eq!(event.department_id, dept.id);
}

// =============================================================================
// Validation failure: empty name
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `name` is empty, `create_department` returns
/// `DomainError::Validation`. The service returns the error
/// directly (no aggregate is produced, no event is minted) so
/// there is nothing to assert on the aggregate side.
#[test]
fn create_department_rejects_empty_name() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_department(
        tenant,
        String::new(),
        Some("no name".to_owned()),
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect_err("empty name must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
