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
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_hr::prelude::*;
use educore_hr::services::{DepartmentReferenceChecker, ReferenceDataUniquenessChecker};
use educore_hr::value_objects::DepartmentId;

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

/// `ReferenceDataUniquenessChecker` mock that reports the
/// supplied name as already-existing. Used to pin the
/// duplicate-name rejection path (spec Department#1).
struct DuplicateNameUniqueness;
impl ReferenceDataUniquenessChecker for DuplicateNameUniqueness {
    fn department_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        true
    }
    fn designation_title_exists(&self, _school: SchoolId, _title: &str) -> bool {
        false
    }
    fn leave_type_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
}

/// Configurable `DepartmentReferenceChecker` mock. Each test
/// sets `staff` / `head` to the desired boolean state.
struct FakeDeptRefs {
    staff: bool,
    head: bool,
}
impl DepartmentReferenceChecker for FakeDeptRefs {
    fn has_assigned_staff(
        &self,
        _school: SchoolId,
        _department_id: DepartmentId,
    ) -> bool {
        self.staff
    }
    fn has_department_head(
        &self,
        _school: SchoolId,
        _department_id: DepartmentId,
    ) -> bool {
        self.head
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

// =============================================================================
// Wave 173 — Spec invariant Department#1 (name unique)
// =============================================================================

/// Spec Department#1: "A `Department` is uniquely named within
/// a school." Pinned via the `DuplicateNameUniqueness` mock that
/// reports every name as already-existing. The service must
/// reject with `DomainError::Conflict` (not Validation — this is
/// a uniqueness conflict, not a malformed input).
#[test]
fn create_department_rejects_duplicate_name_via_uniqueness_checker() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_department(
        tenant,
        "Mathematics".to_owned(),
        None,
        &clock,
        &ids,
        &DuplicateNameUniqueness,
    )
    .expect_err("duplicate name must fail uniqueness check");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 173 — Spec invariant Department#2 + Department#3
// (delete guards)
// =============================================================================

/// Helper: build a deletable (non-system-defined) department
/// for delete-path tests. Mirrors the service-layer pattern
/// but skips the uniqueness check (we use `NoOpRefUniqueness`).
fn fresh_department_for_delete(
    tenant: &TenantContext,
    clock: &TestClock,
    ids: &SystemIdGen,
) -> Department {
    let (dept, _event) = create_department(
        tenant.clone(),
        "Mathematics".to_owned(),
        None,
        clock,
        ids,
        &NoOpRefUniqueness,
    )
    .expect("create department");
    dept
}

/// Spec Department#2 happy path: a department with no
/// assigned Staff and no DepartmentHead row can be
/// soft-deleted. The mutator flips `active_status = Retired`
/// and the service emits a `DepartmentDeleted` event with the
/// correct `EVENT_TYPE`.
#[test]
fn delete_department_happy_path_when_no_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    assert!(dept.active_status.is_active());

    let cmd = DeleteDepartmentCommand {
        tenant: tenant.clone(),
        department_id: dept.id,
        reason: "decommissioning".to_owned(),
    };
    let refs = FakeDeptRefs { staff: false, head: false };
    let event = delete_department(&mut dept, cmd, &clock, &ids, &refs)
        .expect("clean delete must succeed");
    assert_eq!(
        <DepartmentDeleted as DomainEvent>::EVENT_TYPE,
        "hr.department.deleted"
    );
    assert_eq!(event.department_id, dept.id);
    assert!(!dept.active_status.is_active());
}

/// Spec Department#2 rejection: department with an assigned
/// Staff row cannot be hard-deleted. The mutator returns
/// `DomainError::Conflict` and does not mutate the aggregate.
#[test]
fn delete_department_rejects_when_staff_assigned() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    let original_active = dept.active_status;

    let cmd = DeleteDepartmentCommand {
        tenant: tenant.clone(),
        department_id: dept.id,
        reason: "rejected".to_owned(),
    };
    let refs = FakeDeptRefs { staff: true, head: false };
    let err = delete_department(&mut dept, cmd, &clock, &ids, &refs)
        .expect_err("Staff-assigned delete must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    // Aggregate must be unchanged after the rejection.
    assert_eq!(dept.active_status, original_active);
}

/// Spec Department#2 secondary check: even with no Staff
/// assigned, a department with a DepartmentHead row cannot be
/// hard-deleted. Same Conflict error class.
#[test]
fn delete_department_rejects_when_department_head_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    let original_active = dept.active_status;

    let cmd = DeleteDepartmentCommand {
        tenant: tenant.clone(),
        department_id: dept.id,
        reason: "rejected".to_owned(),
    };
    let refs = FakeDeptRefs { staff: false, head: true };
    let err = delete_department(&mut dept, cmd, &clock, &ids, &refs)
        .expect_err("DepartmentHead-referenced delete must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert_eq!(dept.active_status, original_active);
}

/// Spec Department#3: system-defined departments are
/// immutable. The service rejects the delete with
/// `DomainError::Validation` BEFORE the cross-aggregate
/// reference check runs (cheaper guard first).
#[test]
fn delete_department_rejects_system_defined() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    dept.is_system_defined = true;
    let original_active = dept.active_status;

    let cmd = DeleteDepartmentCommand {
        tenant: tenant.clone(),
        department_id: dept.id,
        reason: "cannot delete system dept".to_owned(),
    };
    // Even with a clean reference checker, the system-defined
    // guard must reject.
    let refs = FakeDeptRefs { staff: false, head: false };
    let err = delete_department(&mut dept, cmd, &clock, &ids, &refs)
        .expect_err("system-defined delete must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(dept.active_status, original_active);
}

/// `Department::ensure_deletable` direct unit test: a
/// non-system-defined department passes the guard; a
/// system-defined one fails with Validation. Bypasses the
/// service layer to pin the mutator in isolation.
#[test]
fn department_ensure_deletable_rejects_system_defined() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    assert!(dept.ensure_deletable().is_ok());
    dept.is_system_defined = true;
    let err = dept.ensure_deletable().unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// `Department::soft_delete` direct unit test: with a clean
/// reference checker, the mutator flips `active_status =
/// Retired` and updates the audit footer (`updated_at`,
/// `updated_by`).
#[test]
fn department_soft_delete_flips_active_status_and_audit_footer() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut dept = fresh_department_for_delete(&tenant, &clock, &ids);
    let now = Timestamp::now();
    let refs = FakeDeptRefs { staff: false, head: false };
    dept.soft_delete(&refs, now, tenant.actor_id)
        .expect("clean soft-delete must succeed");
    assert!(!dept.active_status.is_active());
    assert_eq!(dept.updated_by, tenant.actor_id);
}
