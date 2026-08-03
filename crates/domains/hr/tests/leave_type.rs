//! Integration tests for the **LeaveType aggregate** vertical slice.
//!
//! Pins the create + delete contract for
//! [`LeaveType`](educore_hr::aggregate::LeaveType) end-to-end
//! through the service layer:
//!
//! 1. `create_leave_type` validates the input (type_name via
//!    `validate_leave_type_name`), constructs the aggregate
//!    (school id derived from the typed id), and emits a
//!    [`LeaveTypeCreated`] event with the right `event_type`,
//!    `aggregate_type`, `school_id`, and aggregate id.
//! 2. The duplicate-name path returns `DomainError::Conflict`
//!    via the `ReferenceDataUniquenessChecker` port.
//! 3. The validation failure path (empty name) returns
//!    `DomainError::Validation`.
//! 4. `delete_leave_type` enforces spec invariant LeaveType#2
//!    (cannot delete while LeaveDefine or LeaveRequest
//!    references it) via the `LeaveTypeReferenceChecker` port.
//!
//! Wave 175 coverage:
//! - Spec invariant LeaveType#1: name unique per school.
//! - Spec invariant LeaveType#2: cannot delete while LeaveDefine
//!   or LeaveRequest references.
//! - Spec invariant LeaveType#3: `total_days >= 0` (structural
//!   via `u32`; pinned via the `ensure_total_days_valid`
//!   mutator that documents the invariant for callers).
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
    LeaveTypeReferenceChecker, ReferenceDataUniquenessChecker,
};
use educore_hr::value_objects::LeaveTypeId;

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
/// leave type name as already-existing. Used to pin the
/// duplicate-name rejection path (spec LeaveType#1).
struct DuplicateLeaveTypeUniqueness;
impl ReferenceDataUniquenessChecker for DuplicateLeaveTypeUniqueness {
    fn department_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        false
    }
    fn designation_title_exists(&self, _school: SchoolId, _title: &str) -> bool {
        false
    }
    fn leave_type_name_exists(&self, _school: SchoolId, _name: &str) -> bool {
        true
    }
}

/// Configurable `LeaveTypeReferenceChecker` mock. Each test
/// sets `leave_define` / `leave_request` to the desired
/// boolean state.
struct FakeLeaveTypeRefs {
    leave_define: bool,
    leave_request: bool,
}
impl LeaveTypeReferenceChecker for FakeLeaveTypeRefs {
    fn has_leave_define(
        &self,
        _school: SchoolId,
        _leave_type_id: LeaveTypeId,
    ) -> bool {
        self.leave_define
    }
    fn has_leave_request(
        &self,
        _school: SchoolId,
        _leave_type_id: LeaveTypeId,
    ) -> bool {
        self.leave_request
    }
}

// =============================================================================
// Happy path: create a leave type
// =============================================================================

/// End-to-end happy path for the LeaveType aggregate.
/// Create a leave type called "Casual Leave" with 12 total
/// days, asserting that:
///
/// 1. The create flow produces a `LeaveType` aggregate
///    carrying the type_name + total_days from the call args
///    (school id derived from the typed id), with the audit
///    footer initialised (`version == 1`, active).
/// 2. The emitted `LeaveTypeCreated` event carries the right
///    `event_type`, `aggregate_type`, `schema_version`,
///    `aggregate_id`, `school_id`, `type_name`, and
///    `total_days`.
#[test]
fn create_leave_type_returns_aggregate_and_event() {
    let tenant = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let (lt, event) = create_leave_type(
        tenant.clone(),
        "Casual Leave".to_owned(),
        12,
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect("create leave type");

    // Aggregate fields are populated from the command.
    assert_eq!(lt.school_id, school);
    assert_eq!(lt.type_name, "Casual Leave");
    assert_eq!(lt.total_days, 12);
    assert_eq!(lt.created_by, tenant.actor_id);
    assert_eq!(lt.updated_by, tenant.actor_id);
    // Audit metadata footer is initialised at version 1 and
    // active.
    assert_eq!(lt.version.get(), 1);
    assert!(lt.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <LeaveTypeCreated as DomainEvent>::EVENT_TYPE,
        "hr.leave_type.created"
    );
    assert_eq!(
        <LeaveTypeCreated as DomainEvent>::AGGREGATE_TYPE,
        "leave_type"
    );
    assert_eq!(<LeaveTypeCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), lt.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.type_name, "Casual Leave");
    assert_eq!(event.total_days, 12);
    assert_eq!(event.leave_type_id, lt.id);
}

// =============================================================================
// Validation failure: empty name
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `type_name` is empty, `create_leave_type` returns
/// `DomainError::Validation` (via `validate_leave_type_name`).
#[test]
fn create_leave_type_rejects_empty_name() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_leave_type(
        tenant,
        String::new(),
        12,
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
// Wave 175 — Spec invariant LeaveType#1 (name unique)
// =============================================================================

/// Spec LeaveType#1: "A `LeaveType` is uniquely named within
/// a school." Pinned via the `DuplicateLeaveTypeUniqueness`
/// mock that reports every name as already-existing. The
/// service must reject with `DomainError::Conflict`.
#[test]
fn create_leave_type_rejects_duplicate_name_via_uniqueness_checker() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let err = create_leave_type(
        tenant,
        "Casual Leave".to_owned(),
        12,
        &clock,
        &ids,
        &DuplicateLeaveTypeUniqueness,
    )
    .expect_err("duplicate name must fail uniqueness check");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 175 — Spec invariant LeaveType#3 (total_days >= 0)
// =============================================================================

/// Spec LeaveType#3: "`total_days >= 0`." This is a
/// **structural** invariant: the `total_days` field is a
/// `u32`, which cannot hold a negative value. The
/// `ensure_total_days_valid` mutator documents the invariant
/// for callers and tests.
#[test]
fn leave_type_ensure_total_days_valid_accepts_zero() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (lt, _event) = create_leave_type(
        tenant,
        "Zero Days".to_owned(),
        0,
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect("create leave type with zero days");
    assert_eq!(lt.total_days, 0);
    assert!(lt.ensure_total_days_valid().is_ok());
}

#[test]
fn leave_type_ensure_total_days_valid_accepts_positive() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let (lt, _event) = create_leave_type(
        tenant,
        "Casual Leave".to_owned(),
        12,
        &clock,
        &ids,
        &NoOpRefUniqueness,
    )
    .expect("create leave type");
    assert_eq!(lt.total_days, 12);
    assert!(lt.ensure_total_days_valid().is_ok());
}

// =============================================================================
// Wave 175 — Spec invariant LeaveType#2 (delete guards)
// =============================================================================

/// Helper: build a deletable leave type for delete-path tests.
/// Skips the uniqueness check (uses `NoOpRefUniqueness`).
fn fresh_leave_type_for_delete(
    tenant: &TenantContext,
    clock: &TestClock,
    ids: &SystemIdGen,
) -> LeaveType {
    let (lt, _event) = create_leave_type(
        tenant.clone(),
        "Casual Leave".to_owned(),
        12,
        clock,
        ids,
        &NoOpRefUniqueness,
    )
    .expect("create leave type");
    lt
}

/// Spec LeaveType#2 happy path: a leave type with no
/// LeaveDefine and no LeaveRequest references can be
/// soft-deleted. The mutator flips `active_status = Retired`
/// and the service emits a `LeaveTypeDeleted` event with the
/// correct `EVENT_TYPE`.
#[test]
fn delete_leave_type_happy_path_when_no_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut lt = fresh_leave_type_for_delete(&tenant, &clock, &ids);
    assert!(lt.active_status.is_active());

    let cmd = DeleteLeaveTypeCommand {
        tenant: tenant.clone(),
        leave_type_id: lt.id,
        reason: "decommissioning".to_owned(),
    };
    let refs = FakeLeaveTypeRefs {
        leave_define: false,
        leave_request: false,
    };
    let event = delete_leave_type(&mut lt, cmd, &clock, &ids, &refs)
        .expect("clean delete must succeed");
    assert_eq!(
        <LeaveTypeDeleted as DomainEvent>::EVENT_TYPE,
        "hr.leave_type.deleted"
    );
    assert_eq!(event.leave_type_id, lt.id);
    assert!(!lt.active_status.is_active());
}

/// Spec LeaveType#2 rejection: a leave type with a LeaveDefine
/// row referencing it cannot be hard-deleted. The mutator
/// returns `DomainError::Conflict` and does not mutate the
/// aggregate.
#[test]
fn delete_leave_type_rejects_when_leave_define_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut lt = fresh_leave_type_for_delete(&tenant, &clock, &ids);
    let original_active = lt.active_status;

    let cmd = DeleteLeaveTypeCommand {
        tenant: tenant.clone(),
        leave_type_id: lt.id,
        reason: "rejected".to_owned(),
    };
    let refs = FakeLeaveTypeRefs {
        leave_define: true,
        leave_request: false,
    };
    let err = delete_leave_type(&mut lt, cmd, &clock, &ids, &refs)
        .expect_err("LeaveDefine-referenced delete must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert_eq!(lt.active_status, original_active);
}

/// Spec LeaveType#2 secondary check: even with no LeaveDefine
/// row, a leave type with a LeaveRequest referencing it
/// cannot be hard-deleted. Same Conflict error class.
#[test]
fn delete_leave_type_rejects_when_leave_request_references() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut lt = fresh_leave_type_for_delete(&tenant, &clock, &ids);
    let original_active = lt.active_status;

    let cmd = DeleteLeaveTypeCommand {
        tenant: tenant.clone(),
        leave_type_id: lt.id,
        reason: "rejected".to_owned(),
    };
    let refs = FakeLeaveTypeRefs {
        leave_define: false,
        leave_request: true,
    };
    let err = delete_leave_type(&mut lt, cmd, &clock, &ids, &refs)
        .expect_err("LeaveRequest-referenced delete must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert_eq!(lt.active_status, original_active);
}

/// `LeaveType::soft_delete` direct unit test: with a clean
/// reference checker, the mutator flips `active_status =
/// Retired` and updates the audit footer (`updated_at`,
/// `updated_by`).
#[test]
fn leave_type_soft_delete_flips_active_status_and_audit_footer() {
    let tenant = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut lt = fresh_leave_type_for_delete(&tenant, &clock, &ids);
    let now = Timestamp::now();
    let refs = FakeLeaveTypeRefs {
        leave_define: false,
        leave_request: false,
    };
    lt.soft_delete(&refs, now, tenant.actor_id)
        .expect("clean soft-delete must succeed");
    assert!(!lt.active_status.is_active());
    assert_eq!(lt.updated_by, tenant.actor_id);
}
