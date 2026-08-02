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
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_hr::prelude::*;
use educore_hr::services::{
    DesignationReferenceChecker, ReferenceDataUniquenessChecker,
};
use educore_hr::value_objects::DesignationId;

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

/// `ReferenceDataUniquenessChecker` mock that reports every
/// designation title as already-existing. Used to pin the
/// duplicate-title rejection path (spec Designation#1).
struct DuplicateTitleUniqueness;
impl ReferenceDataUniquenessChecker for DuplicateTitleUniqueness {
    fn department_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
    fn designation_title_exists(&self, _school: SchoolId, _title: &str) -> bool {
        true
    }
    fn leave_type_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
}

/// Configurable `DesignationReferenceChecker` mock.
struct FakeDesignationRefs {
    staff: bool,
}
impl DesignationReferenceChecker for FakeDesignationRefs {
    fn has_assigned_staff(
        &self,
        _school: SchoolId,
        _designation_id: DesignationId,
    ) -> bool {
        self.staff
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

// =============================================================================
// Wave 174 — Spec invariant Designation#1 (title unique)
// =============================================================================

/// Spec Designation#1: "A `Designation` is uniquely named within
/// a school." Pinned via the `DuplicateTitleUniqueness` mock that
/// reports every title as already-existing. The service must
/// reject with `DomainError::Conflict`.
#[test]
fn create_designation_rejects_duplicate_title_via_uniqueness_checker() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_designation(
        tenant,
        "Principal".to_owned(),
        None,
        &clock,
        &ids,
        &DuplicateTitleUniqueness,
    )
    .expect_err("duplicate title must fail uniqueness check");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 174 — Spec invariants Designation#2 + Designation#3
// (delete guards)
// =============================================================================

/// Helper: build a deletable (non-system-defined) designation
/// for delete-path tests. Skips the uniqueness check.
fn fresh_designation_for_delete(
    tenant: &TenantContext,
    clock: &TestClock,
    ids: &SystemIdGen,
) -> Designation {
    let (desig, _event) = create_designation(
        tenant.clone(),
        "Principal".to_owned(),
        None,
        clock,
        ids,
        &NoOpRefUniqueness,
    )
    .expect("create designation");
    desig
}

/// Spec Designation#2 happy path: a designation with no
/// assigned Staff can be soft-deleted. The mutator flips
/// `active_status = Retired` and the service emits a
/// `DesignationDeleted` event with the correct `EVENT_TYPE`.
#[test]
fn delete_designation_happy_path_when_no_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut desig = fresh_designation_for_delete(&tenant, &clock, &ids);
    assert!(desig.active_status.is_active());

    let cmd = DeleteDesignationCommand {
        tenant: tenant.clone(),
        designation_id: desig.id,
        reason: "decommissioning".to_owned(),
    };
    let refs = FakeDesignationRefs { staff: false };
    let event = delete_designation(&mut desig, cmd, &clock, &ids, &refs)
        .expect("clean delete must succeed");
    assert_eq!(
        <DesignationDeleted as DomainEvent>::EVENT_TYPE,
        "hr.designation.deleted"
    );
    assert_eq!(event.designation_id, desig.id);
    assert!(!desig.active_status.is_active());
}

/// Spec Designation#2 rejection: designation with an assigned
/// Staff row cannot be hard-deleted. The mutator returns
/// `DomainError::Conflict` and does not mutate the aggregate.
#[test]
fn delete_designation_rejects_when_staff_assigned() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut desig = fresh_designation_for_delete(&tenant, &clock, &ids);
    let original_active = desig.active_status;

    let cmd = DeleteDesignationCommand {
        tenant: tenant.clone(),
        designation_id: desig.id,
        reason: "rejected".to_owned(),
    };
    let refs = FakeDesignationRefs { staff: true };
    let err = delete_designation(&mut desig, cmd, &clock, &ids, &refs)
        .expect_err("Staff-assigned delete must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert_eq!(desig.active_status, original_active);
}

/// Spec Designation#3: system-defined designations are
/// immutable. The service rejects the delete with
/// `DomainError::Validation` BEFORE the cross-aggregate
/// reference check runs (cheaper guard first).
#[test]
fn delete_designation_rejects_system_defined() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut desig = fresh_designation_for_delete(&tenant, &clock, &ids);
    desig.is_system_defined = true;
    let original_active = desig.active_status;

    let cmd = DeleteDesignationCommand {
        tenant: tenant.clone(),
        designation_id: desig.id,
        reason: "cannot delete system designation".to_owned(),
    };
    let refs = FakeDesignationRefs { staff: false };
    let err = delete_designation(&mut desig, cmd, &clock, &ids, &refs)
        .expect_err("system-defined delete must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(desig.active_status, original_active);
}

/// `Designation::ensure_deletable` direct unit test.
#[test]
fn designation_ensure_deletable_rejects_system_defined() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut desig = fresh_designation_for_delete(&tenant, &clock, &ids);
    assert!(desig.ensure_deletable().is_ok());
    desig.is_system_defined = true;
    let err = desig.ensure_deletable().unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// `Designation::soft_delete` direct unit test: with a clean
/// reference checker, the mutator flips `active_status =
/// Retired` and updates the audit footer (`updated_at`,
/// `updated_by`).
#[test]
fn designation_soft_delete_flips_active_status_and_audit_footer() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut desig = fresh_designation_for_delete(&tenant, &clock, &ids);
    let now = Timestamp::now();
    let refs = FakeDesignationRefs { staff: false };
    desig.soft_delete(&refs, now, tenant.actor_id)
        .expect("clean soft-delete must succeed");
    assert!(!desig.active_status.is_active());
    assert_eq!(desig.updated_by, tenant.actor_id);
}
