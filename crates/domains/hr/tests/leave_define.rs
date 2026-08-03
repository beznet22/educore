//! Integration tests for the **LeaveDefine aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`LeaveDefine`](educore_hr::aggregate::LeaveDefine) end-to-end,
//! plus the Wave 176 mutators that enforce spec invariants
//! I-1 (uniqueness by `(school, academic, role/user, type)`
//! composite key) and I-2 (`days >= 0` and `total_days >= 0`,
//! structural via `u32`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::LeaveDefine;
use educore_hr::services::LeaveDefineUniquenessChecker;
use educore_hr::value_objects::{
    AcademicYearId, LeaveDefineId, LeaveTypeId, RoleId,
};

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

fn leave_define_id(g: &SystemIdGen, school: SchoolId) -> LeaveDefineId {
    LeaveDefineId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn leave_type_id(g: &SystemIdGen, school: SchoolId) -> LeaveTypeId {
    LeaveTypeId::new(school, g.next_uuid())
}

/// Configurable `LeaveDefineUniquenessChecker` mock.
struct FakeLeaveDefineUniqueness {
    exists: bool,
}
impl LeaveDefineUniquenessChecker for FakeLeaveDefineUniqueness {
    fn leave_define_exists(
        &self,
        _school: SchoolId,
        _academic_id: AcademicYearId,
        _role_id: Option<RoleId>,
        _user_id: Option<UserId>,
        _type_id: LeaveTypeId,
    ) -> bool {
        self.exists
    }
}

#[test]
fn leave_define_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_define_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn leave_define_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = leave_define_id(&g, school);
    let id_b = leave_define_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Spec invariant LeaveDefine#3 (days <= total_days) — already enforced in
// LeaveDefine::fresh since Wave 32. Regression tests below.
// =============================================================================

#[test]
fn leave_define_fresh_rejects_days_exceeding_total_days() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_define_id(&g, school);
    let role_id = RoleId::new(tenant.school_id, g.next_uuid());
    let type_id = leave_type_id(&g, school);
    let academic_id = academic_year_id(&g, school);
    let err = LeaveDefine::fresh(
        id,
        Some(role_id),
        None,
        type_id,
        20, // days
        10, // total_days (less than days)
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect_err("days > total_days must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn leave_define_fresh_accepts_days_equal_total_days() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_define_id(&g, school);
    let role_id = RoleId::new(tenant.school_id, g.next_uuid());
    let type_id = leave_type_id(&g, school);
    let academic_id = academic_year_id(&g, school);
    let ld = LeaveDefine::fresh(
        id,
        Some(role_id),
        None,
        type_id,
        10,
        10,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("days == total_days must succeed");
    assert_eq!(ld.days, 10);
    assert_eq!(ld.total_days, 10);
}

// =============================================================================
// Wave 176 — Spec invariant LeaveDefine#2 (days >= 0, total_days >= 0)
// =============================================================================

#[test]
fn leave_define_ensure_non_negative_accepts_zero() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_define_id(&g, school);
    let role_id = RoleId::new(tenant.school_id, g.next_uuid());
    let type_id = leave_type_id(&g, school);
    let academic_id = academic_year_id(&g, school);
    let ld = LeaveDefine::fresh(
        id,
        Some(role_id),
        None,
        type_id,
        0,
        0,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero days + zero total_days must succeed");
    assert!(ld.ensure_non_negative().is_ok());
}

#[test]
fn leave_define_ensure_non_negative_accepts_positive() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = leave_define_id(&g, school);
    let role_id = RoleId::new(tenant.school_id, g.next_uuid());
    let type_id = leave_type_id(&g, school);
    let academic_id = academic_year_id(&g, school);
    let ld = LeaveDefine::fresh(
        id,
        Some(role_id),
        None,
        type_id,
        5,
        10,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("positive days + total_days must succeed");
    assert!(ld.ensure_non_negative().is_ok());
}

// =============================================================================
// Wave 176 — Spec invariant LeaveDefine#1 (uniqueness)
// =============================================================================

/// Helper: build a LeaveDefine with role_id set, for
/// uniqueness tests.
fn fresh_leave_define_with_role(
    tenant: &TenantContext,
    g: &SystemIdGen,
) -> LeaveDefine {
    let school = tenant.school_id;
    let id = leave_define_id(g, school);
    let role_id = RoleId::new(tenant.school_id, g.next_uuid());
    let type_id = leave_type_id(g, school);
    let academic_id = academic_year_id(g, school);
    LeaveDefine::fresh(
        id,
        Some(role_id),
        None,
        type_id,
        5,
        10,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("create leave define")
}

#[test]
fn leave_define_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let ld = fresh_leave_define_with_role(&tenant, &g);
    let checker = FakeLeaveDefineUniqueness { exists: false };
    assert!(ld.ensure_unique(&checker).is_ok());
}

#[test]
fn leave_define_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let ld = fresh_leave_define_with_role(&tenant, &g);
    let checker = FakeLeaveDefineUniqueness { exists: true };
    let err = ld.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

#[test]
fn leave_define_ensure_unique_rejects_neither_role_nor_user() {
    let (tenant, g) = admin_context();
    let mut ld = fresh_leave_define_with_role(&tenant, &g);
    ld.role_id = None;
    ld.user_id = None;
    let checker = FakeLeaveDefineUniqueness { exists: false };
    let err = ld
        .ensure_unique(&checker)
        .expect_err("neither role nor user must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
