//! Integration tests for the **LibraryMember aggregate** vertical slice.
//!
//! Pins the create + lifecycle contract for
//! [`LibraryMember`](educore_library::aggregate::LibraryMember)
//! end-to-end through the service layer:
//!
//! 1. `register_library_member` validates the input (typed
//!    [`MemberUdId`](educore_library::value_objects::MemberUdId)
//!    enforces non-empty + length bounds), constructs the
//!    aggregate with the full 10-field audit footer
//!    initialised, and emits a [`LibraryMemberRegistered`]
//!    event whose `event_type` / `aggregate_type` /
//!    `school_id` match the typed id's `school_id()`
//!    accessor.
//! 2. The aggregate's `deactivate` method transitions the
//!    member to `Inactive` / `Retired`, bumps `version`,
//!    and stamps `updated_at` / `updated_by` from the actor.
//!    `reactivate` reverses the transition.
//!
//! The tests use the same fixture pattern as
//! `tests/aggregates.rs` (`TestClock` + `SystemIdGen`). The
//! handlers / outbox / audit fan-out are not yet wired
//! end-to-end; these tests pin the **service layer** contract
//! that the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

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
use educore_library::value_objects::MemberId;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
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

/// Mint a `StaffId` for the given school.
fn staff_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> StaffId {
    StaffId::new(school, g.next_uuid())
}

/// Mint an `AcademicYearId` for the given school.
fn year_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: create LibraryMember via service factory
// =============================================================================

/// End-to-end happy path for the LibraryMember aggregate.
/// Register a staff member and assert that:
///
/// 1. The service returns a `LibraryMember` aggregate with
///    `school_id` derived from the typed id, `status =
///    Active`, `active_status = Active`, and the audit
///    footer initialised (`version = 1`, `created_by` /
///    `updated_by` from the tenant).
/// 2. The emitted event is a `LibraryMemberRegistered`
///    carrying the right `event_type` / `aggregate_type` /
///    `school_id`, with `library_member_id` matching the
///    aggregate's typed id.
#[test]
fn library_member_register_emits_event_and_populates_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let cmd = RegisterLibraryMemberCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        member: MemberId::Staff(staff_id(&g, school)),
        member_type: RoleId::new(school, g.next_uuid()),
        member_ud_id: MemberUdId::new("STF-001").expect("non-empty ud_id"),
    };
    let (member, event) = register_library_member(cmd, &clock, &ids).expect("register");

    // Aggregate fields are populated from the command.
    assert_eq!(member.school_id, school);
    assert_eq!(member.member_ud_id.as_str(), "STF-001");
    assert_eq!(member.status, MemberStatus::Active);
    assert!(member.active_status.is_active());
    assert_eq!(member.created_by, tenant.actor_id);
    assert_eq!(member.updated_by, tenant.actor_id);

    // Audit metadata footer is initialised.
    assert_eq!(member.version.get(), 1);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <LibraryMemberRegistered as DomainEvent>::EVENT_TYPE,
        "library.member.registered"
    );
    assert_eq!(
        <LibraryMemberRegistered as DomainEvent>::AGGREGATE_TYPE,
        "library_member"
    );
    assert_eq!(<LibraryMemberRegistered as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), member.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.library_member_id, member.id);
}

// =============================================================================
// Happy path: deactivate and reactivate a LibraryMember
// =============================================================================

/// End-to-end happy path for the LibraryMember deactivate
/// + reactivate lifecycle. The aggregate's `deactivate`
/// method transitions the member to `Inactive` / `Retired`,
/// bumps the version, and stamps `updated_at` /
/// `updated_by`. `reactivate` reverses the transition.
///
/// (The `deactivate_library_member` service handler is still
/// a TODO stub in `services.rs`, so this test pins the
/// **aggregate-level** transition directly — the contract the
/// dispatcher will wrap once the handler is wired.)
#[test]
fn library_member_deactivate_then_reactivate_mutates_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Seed a LibraryMember via the service.
    let cmd = RegisterLibraryMemberCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        member: MemberId::Staff(staff_id(&g, school)),
        member_type: RoleId::new(school, g.next_uuid()),
        member_ud_id: MemberUdId::new("STF-002").expect("non-empty ud_id"),
    };
    let (mut member, _event) = register_library_member(cmd, &clock, &ids).expect("register");
    let initial_version = member.version.get();
    let event_id = ids.next_event_id();
    let at = clock.now();

    // ---- Deactivate ----
    member.deactivate(tenant.actor_id, at, event_id);
    assert_eq!(member.status, MemberStatus::Inactive);
    assert!(!member.active_status.is_active());
    assert_eq!(member.version.get(), initial_version + 1);
    assert_eq!(member.updated_by, tenant.actor_id);
    assert_eq!(member.last_event_id, Some(event_id));

    // ---- Reactivate ----
    let reactivate_event_id = ids.next_event_id();
    let reactivate_at = clock.now();
    member.reactivate(tenant.actor_id, reactivate_at, reactivate_event_id);
    assert_eq!(member.status, MemberStatus::Active);
    assert!(member.active_status.is_active());
    assert_eq!(member.version.get(), initial_version + 2);
    assert_eq!(member.updated_by, tenant.actor_id);
    assert_eq!(member.last_event_id, Some(reactivate_event_id));
}

// =============================================================================
// Validation failure: empty member_ud_id is rejected
// =============================================================================

/// Validation-failure path on the create flow: when
/// `member_ud_id` is empty (after trimming), the
/// [`MemberUdId::new`] constructor returns a validation
/// error before the service is even called. This test pins
/// that the constructor enforces the spec's non-empty
/// invariant.
#[test]
fn library_member_register_with_empty_ud_id_returns_validation_error() {
    use educore_core::error::DomainError;
    let err = MemberUdId::new("   ").expect_err("empty ud_id must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
