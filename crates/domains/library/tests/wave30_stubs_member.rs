//! Integration tests for the Wave 30 stub replacements in
//! `crates/domains/library/src/services.rs` (batch 2 —
//! LibraryMember CRUD).
//!
//! Pins the contract for:
//!
//! - [`update_library_member`](educore_library::services::update_library_member)
//! - [`deactivate_library_member`](educore_library::services::deactivate_library_member)
//! - [`reactivate_library_member`](educore_library::services::reactivate_library_member)
//! - [`delete_library_member`](educore_library::services::delete_library_member)
//!
//! Each test pins the happy path of the corresponding
//! factory fn. Mirrors the pattern in
//! `crates/domains/library/tests/wave30_stubs.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::AcademicYearId;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_hr::value_objects::{RoleId, StaffId};
use educore_library::prelude::*;
use educore_library::services::{
    deactivate_library_member, delete_library_member, reactivate_library_member,
    update_library_member,
};
use educore_library::value_objects::MemberId;

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let tenant = TenantContext::for_user(
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        UserType::SchoolAdmin,
    );
    (tenant, g)
}

/// Mint a `StaffId` for the given school.
fn staff_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> StaffId {
    StaffId::new(school, g.next_uuid())
}

/// Mint an `AcademicYearId` for the given school.
fn year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

/// Seeds a fresh `LibraryMember` for the given tenant.
fn seed_member(tenant: &TenantContext, g: &SystemIdGen, ud_id: &str) -> LibraryMember {
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let school = tenant.school_id;
    let cmd = RegisterLibraryMemberCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(g, school),
        member: MemberId::Staff(staff_id(g, school)),
        member_type: RoleId::new(school, g.next_uuid()),
        member_ud_id: MemberUdId::new(ud_id).expect("non-empty ud_id"),
    };
    let (member, _event) = register_library_member(cmd, &clock, &ids).expect("register");
    member
}

// =============================================================================
// update_library_member
// =============================================================================

/// `update_library_member` mutates the supplied
/// `member_ud_id` on the aggregate (bumps `version`,
/// updates `updated_at` / `updated_by`) and emits a
/// `LibraryMemberUpdated` event whose `changes` list names
/// the field that actually moved.
#[test]
fn update_library_member_mutates_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut member = seed_member(&tenant, &g, "STF-100");
    let initial_version = member.version.get();

    let cmd = UpdateLibraryMemberCommand {
        tenant: tenant.clone(),
        library_member_id: member.id,
        member_ud_id: Some(MemberUdId::new("STF-100-RENAMED").expect("non-empty")),
        note: None,
    };
    let event = update_library_member(cmd, &clock, &ids, &mut member).expect("update");

    assert_eq!(member.member_ud_id.as_str(), "STF-100-RENAMED");
    assert_eq!(member.version.get(), initial_version + 1);
    assert_eq!(member.updated_by, tenant.actor_id);
    assert_eq!(
        <LibraryMemberUpdated as DomainEvent>::EVENT_TYPE,
        "library.member.updated"
    );
    assert_eq!(
        <LibraryMemberUpdated as DomainEvent>::AGGREGATE_TYPE,
        "library_member"
    );
    assert_eq!(event.aggregate_id(), member.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
    assert!(event.changes.contains(&"member_ud_id".to_owned()));
}

// =============================================================================
// deactivate_library_member
// =============================================================================

/// `deactivate_library_member` transitions the member to
/// `Inactive` / `Retired`, bumps `version`, and emits a
/// `LibraryMemberDeactivated` event carrying the
/// deactivation reason.
#[test]
fn deactivate_library_member_retires_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut member = seed_member(&tenant, &g, "STF-200");
    let initial_version = member.version.get();

    let cmd = DeactivateLibraryMemberCommand {
        tenant: tenant.clone(),
        library_member_id: member.id,
        reason: "Graduated from school".to_owned(),
    };
    let event = deactivate_library_member(cmd, &clock, &ids, &mut member).expect("deactivate");

    assert_eq!(member.status, MemberStatus::Inactive);
    assert!(!member.active_status.is_active());
    assert_eq!(member.version.get(), initial_version + 1);
    assert_eq!(
        <LibraryMemberDeactivated as DomainEvent>::EVENT_TYPE,
        "library.member.deactivated"
    );
    assert_eq!(
        <LibraryMemberDeactivated as DomainEvent>::AGGREGATE_TYPE,
        "library_member"
    );
    assert_eq!(event.aggregate_id(), member.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.reason, "Graduated from school");
}

// =============================================================================
// reactivate_library_member
// =============================================================================

/// `reactivate_library_member` reverses a prior
/// deactivation, transitioning the member back to `Active`
/// / `Active`, bumping `version`, and emitting a
/// `LibraryMemberReactivated` event.
#[test]
fn reactivate_library_member_restores_active_status_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut member = seed_member(&tenant, &g, "STF-300");

    // First deactivate so reactivate has work to do.
    let deactivate_cmd = DeactivateLibraryMemberCommand {
        tenant: tenant.clone(),
        library_member_id: member.id,
        reason: "Temporary suspension".to_owned(),
    };
    deactivate_library_member(deactivate_cmd, &clock, &ids, &mut member).expect("deactivate");
    assert_eq!(member.status, MemberStatus::Inactive);
    let version_after_deactivate = member.version.get();

    // Now reactivate.
    let reactivate_cmd = ReactivateLibraryMemberCommand {
        tenant: tenant.clone(),
        library_member_id: member.id,
    };
    let event =
        reactivate_library_member(reactivate_cmd, &clock, &ids, &mut member).expect("reactivate");

    assert_eq!(member.status, MemberStatus::Active);
    assert!(member.active_status.is_active());
    assert_eq!(member.version.get(), version_after_deactivate + 1);
    assert_eq!(
        <LibraryMemberReactivated as DomainEvent>::EVENT_TYPE,
        "library.member.reactivated"
    );
    assert_eq!(
        <LibraryMemberReactivated as DomainEvent>::AGGREGATE_TYPE,
        "library_member"
    );
    assert_eq!(event.aggregate_id(), member.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
}

// =============================================================================
// delete_library_member
// =============================================================================

/// `delete_library_member` retires the aggregate
/// (`active_status = Retired`), bumps `version`, and emits
/// a `LibraryMemberDeleted` event.
#[test]
fn delete_library_member_retires_aggregate_and_emits_event() {
    let (tenant, g) = admin();
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let mut member = seed_member(&tenant, &g, "STF-400");
    let initial_version = member.version.get();

    let cmd = DeleteLibraryMemberCommand {
        tenant: tenant.clone(),
        library_member_id: member.id,
    };
    let event = delete_library_member(cmd, &clock, &ids, &mut member).expect("delete");

    assert!(!member.active_status.is_active());
    assert_eq!(member.version.get(), initial_version + 1);
    assert_eq!(
        <LibraryMemberDeleted as DomainEvent>::EVENT_TYPE,
        "library.member.deleted"
    );
    assert_eq!(
        <LibraryMemberDeleted as DomainEvent>::AGGREGATE_TYPE,
        "library_member"
    );
    assert_eq!(event.aggregate_id(), member.id.as_uuid());
    assert_eq!(event.school_id(), tenant.school_id);
}
