//! Integration tests for the **FmFeesGroup aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`FmFeesGroup`](educore_finance::aggregate::FmFeesGroup)
//! end-to-end through the service layer.
//!
//! The pre-existing `create_fm_fees_group` handler in
//! `crates/domains/finance/src/services.rs` is currently a
//! Phase 7 Workstream G skeleton (per the handler docstring):
//! it accepts the typed `CreateFmFeesGroupCommand { tenant,
//! fm_fees_group_id }` and returns `Ok(())` without
//! constructing an aggregate or emitting an event. The spec
//! target ([`docs/specs/finance/aggregates.md`](docs/specs/finance/aggregates.md) §
//! `FmFeesGroup`) expects a `FinanceFeesGroupCreated` event
//! plus field-level validation on `name` — both land with
//! the Workstream G fill-in. These tests pin the **current**
//! stub contract so the eventual full implementation can be
//! validated against this surface without breaking changes.
//!
//! Two scenarios:
//!
//! 1. `fm_fees_group_create_succeeds_with_valid_command` —
//!    happy path: a well-formed command (tenant +
//!    school-scoped `FmFeesGroupId`) is accepted by the
//!    handler and returns `Ok(())`. Pins the current
//!    skeleton contract.
//! 2. `fm_fees_group_create_with_mismatched_school_id` —
//!    documents the current contract: the skeleton handler
//!    does not validate that the `fm_fees_group_id`'s
//!    `school_id` matches the tenant's `school_id`. When
//!    full validation lands in Phase 7 Workstream G, this
//!    test should be updated to assert
//!    `DomainError::Validation` for the same input.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/finance/tests/wallet.rs` and
//! `crates/domains/finance/tests/workflows.rs`
//! (`TestClock` + `SystemIdGen`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_finance::commands::CreateFmFeesGroupCommand;
use educore_finance::services::create_fm_fees_group;
use educore_finance::value_objects::FmFeesGroupId;

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

// =============================================================================
// 1. Happy path: create an FmFeesGroup
// =============================================================================

/// End-to-end happy path for the `FmFeesGroup` aggregate.
/// Construct a `CreateFmFeesGroupCommand` whose
/// `fm_fees_group_id` is scoped to the same school as the
/// tenant, invoke the handler, and assert it returns
/// `Ok(())`. This pins the current Phase 7 Workstream G
/// skeleton contract: the handler accepts well-formed
/// commands and reports success without constructing an
/// aggregate or emitting an event (both land with the
/// Workstream G fill-in).
#[test]
fn fm_fees_group_create_succeeds_with_valid_command() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // School-scoped typed id: matches the tenant's school.
    let fm_fees_group_id = FmFeesGroupId::new(school, g.next_uuid());

    let cmd = CreateFmFeesGroupCommand {
        tenant: tenant.clone(),
        fm_fees_group_id,
    };

    let result = create_fm_fees_group(cmd, &clock, &ids);
    assert!(
        result.is_ok(),
        "create_fm_fees_group must accept a well-formed command, got {result:?}"
    );

    // The typed id's school anchor matches the tenant's
    // school — the compile-time tenant guard is honoured by
    // the test fixture even though the skeleton handler does
    // not yet enforce it at runtime.
    assert_eq!(fm_fees_group_id.school_id(), school);
    assert_eq!(tenant.school_id, school);
}

// =============================================================================
// 2. Current contract: mismatched school_id is accepted
// =============================================================================

/// Documents the current Phase 7 Workstream G skeleton
/// contract: the handler does not validate that the
/// `fm_fees_group_id`'s `school_id` matches the tenant's
/// `school_id`. A command whose typed id points at a
/// different school than the tenant is accepted and returns
/// `Ok(())`. When full validation lands in Phase 7
/// Workstream G, this test should be updated to assert
/// `DomainError::Validation` for the same input.
///
/// The test exists today so the eventual fill-in has a
/// pinned surface to validate against: if the Workstream G
/// fill-in adds a school-id cross-check, this test will
/// start failing and force the author to either tighten the
/// check or update the test deliberately.
#[test]
fn fm_fees_group_create_with_mismatched_school_id_succeeds_under_skeleton() {
    let (tenant, g) = admin_context();
    let _school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Mint a *different* school and a typed id anchored to
    // it. The tenant belongs to the original school, so
    // this is a cross-tenant id.
    let other_school = g.next_school_id();
    let cross_tenant_id = FmFeesGroupId::new(other_school, g.next_uuid());

    let cmd = CreateFmFeesGroupCommand {
        tenant: tenant.clone(),
        fm_fees_group_id: cross_tenant_id,
    };

    let result = create_fm_fees_group(cmd, &clock, &ids);

    // Current skeleton contract: the handler accepts the
    // cross-tenant id and returns Ok(()).
    assert!(
        result.is_ok(),
        "current skeleton accepts cross-tenant ids, got {result:?}"
    );

    // Sanity check: the ids are indeed from different
    // schools (the invariant the future validation will
    // enforce).
    assert_ne!(tenant.school_id, cross_tenant_id.school_id());
}
