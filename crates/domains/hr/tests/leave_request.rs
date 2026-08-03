//! Integration tests for the **LeaveRequest aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`LeaveRequest`](educore_hr::aggregate::LeaveRequest) end-to-end,
//! plus the Wave 177 mutators that enforce spec invariants
//! I-3 (status FSM), I-5 (rejection reason required), and
//! the composite-key uniqueness from spec I-1.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::LeaveRequest;
use educore_hr::services::LeaveRequestUniquenessChecker;
use educore_hr::value_objects::{LeaveRequestId, LeaveStatus, LeaveTypeId, StaffId};

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

fn leave_request_id(g: &SystemIdGen, school: SchoolId) -> LeaveRequestId {
    LeaveRequestId::new(school, g.next_uuid())
}

/// Helper: build a Pending LeaveRequest for reject-path tests.
fn fresh_pending_leave_request(tenant: &TenantContext, g: &SystemIdGen) -> LeaveRequest {
    let school = tenant.school_id;
    let id = leave_request_id(g, school);
    let staff_id = StaffId::new(school, g.next_uuid());
    let type_id = LeaveTypeId::new(school, g.next_uuid());
    LeaveRequest::fresh(
        id,
        staff_id,
        type_id,
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 8, 5).unwrap(),
        NaiveDate::from_ymd_opt(2026, 8, 7).unwrap(),
        Some("family event".to_owned()),
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `LeaveRequestUniquenessChecker` mock.
struct FakeLeaveRequestUniqueness {
    exists: bool,
}
impl LeaveRequestUniquenessChecker for FakeLeaveRequestUniqueness {
    fn leave_request_exists(
        &self,
        _school: SchoolId,
        _staff_id: StaffId,
        _leave_from: NaiveDate,
        _leave_to: NaiveDate,
        _type_id: LeaveTypeId,
    ) -> bool {
        self.exists
    }
}

#[test]
fn leave_request_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_request_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn leave_request_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = leave_request_id(&g, school);
    let id_b = leave_request_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 177 — Spec invariant LeaveRequest#3 (status FSM reject)
// =============================================================================

/// Spec LeaveRequest#3 happy path: a Pending leave request
/// can be rejected with a non-empty reason. The mutator
/// flips `approve_status = Rejected` and stores the
/// `rejection_reason`.
#[test]
fn leave_request_reject_happy_path() {
    let (tenant, g) = admin_context();
    let mut lr = fresh_pending_leave_request(&tenant, &g);
    let rejecter = g.next_user_id();
    lr.reject(rejecter, "insufficient notice".to_owned(), Timestamp::now())
        .expect("reject must succeed");
    assert_eq!(lr.approve_status, LeaveStatus::Rejected);
    assert_eq!(lr.rejecter_id, Some(rejecter));
    assert_eq!(lr.rejection_reason.as_deref(), Some("insufficient notice"));
}

/// Spec LeaveRequest I-5 (reason required for rejections):
/// rejecting with an empty reason returns
/// `DomainError::Validation` and does not mutate the aggregate.
#[test]
fn leave_request_reject_rejects_empty_reason() {
    let (tenant, g) = admin_context();
    let mut lr = fresh_pending_leave_request(&tenant, &g);
    let original_status = lr.approve_status;
    let rejecter = g.next_user_id();
    let err = lr
        .reject(rejecter, String::new(), Timestamp::now())
        .expect_err("empty rejection reason must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(lr.approve_status, original_status);
}

/// Spec LeaveRequest I-5 (whitespace-only reason treated as
/// empty): the mutator trims the reason before checking.
#[test]
fn leave_request_reject_rejects_whitespace_only_reason() {
    let (tenant, g) = admin_context();
    let mut lr = fresh_pending_leave_request(&tenant, &g);
    let rejecter = g.next_user_id();
    let err = lr
        .reject(rejecter, "   \t\n".to_owned(), Timestamp::now())
        .expect_err("whitespace-only rejection reason must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Spec LeaveRequest#3 FSM: a Rejected leave request cannot
/// be rejected again (Rejected is terminal).
#[test]
fn leave_request_reject_rejects_already_rejected() {
    let (tenant, g) = admin_context();
    let mut lr = fresh_pending_leave_request(&tenant, &g);
    let rejecter = g.next_user_id();
    lr.reject(rejecter, "first".to_owned(), Timestamp::now())
        .unwrap();
    let err = lr
        .reject(rejecter, "second".to_owned(), Timestamp::now())
        .expect_err("already-rejected reject must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// Wave 177 — Spec invariant LeaveRequest#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn leave_request_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let lr = fresh_pending_leave_request(&tenant, &g);
    let checker = FakeLeaveRequestUniqueness { exists: false };
    assert!(lr.ensure_unique(&checker).is_ok());
}

#[test]
fn leave_request_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let lr = fresh_pending_leave_request(&tenant, &g);
    let checker = FakeLeaveRequestUniqueness { exists: true };
    let err = lr
        .ensure_unique(&checker)
        .expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 177 — Spec invariant LeaveRequest#5 (days <= LeaveDefine.total_days)
// =============================================================================

/// Spec LeaveRequest#5 happy path: a 3-day leave request
/// fits within a 10-day `LeaveDefine.total_days` entitlement.
#[test]
fn leave_request_ensure_within_leave_define_accepts_within_entitlement() {
    let (tenant, g) = admin_context();
    let lr = fresh_pending_leave_request(&tenant, &g);
    // 3-day request (2026-08-05 to 2026-08-07 inclusive)
    assert_eq!(lr.duration_days(), 3);
    assert!(lr.ensure_within_leave_define(10).is_ok());
}

/// Spec LeaveRequest#5 rejection: a 5-day leave request
/// against a 3-day `LeaveDefine.total_days` entitlement
/// returns `DomainError::Validation`.
#[test]
fn leave_request_ensure_within_leave_define_rejects_exceeding_entitlement() {
    let (tenant, g) = admin_context();
    let lr = fresh_pending_leave_request(&tenant, &g);
    let err = lr
        .ensure_within_leave_define(2)
        .expect_err("3-day request against 2-day entitlement must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

/// Edge case: 0-day entitlement rejects any positive request.
#[test]
fn leave_request_ensure_within_leave_define_rejects_zero_entitlement() {
    let (tenant, g) = admin_context();
    let lr = fresh_pending_leave_request(&tenant, &g);
    let err = lr
        .ensure_within_leave_define(0)
        .expect_err("3-day request against 0-day entitlement must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
