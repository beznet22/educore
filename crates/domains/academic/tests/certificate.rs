//! Integration tests for the **Certificate aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Certificate`](educore_academic::aggregate::Certificate)
//! end-to-end through the service layer:
//!
//! 1. `create_certificate` validates that the typed id's
//!    school matches the command's `school_id`, constructs
//!    the aggregate, and emits a [`CertificateCreated`]
//!    event.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs` and
//! `crates/domains/academic/tests/subject.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests
//! pin the contract of the **service layer** that the
//! dispatcher will eventually wrap.
//!
//! Note on `Certificate` field set: the aggregate is a
//! placeholder stub carrying only `id` (typed
//! `CertificateId`) and `school_id`. The full attribute
//! surface (name, template body, signature lines, etc.)
//! lives in `docs/specs/academic/aggregates.md` §
//! Certificate but has not been wired into the typed
//! command shape yet. The tests below therefore exercise
//! the real contract available today: `id` + `school_id`
//! round-trip through the aggregate and the emitted
//! `CertificateCreated` event.
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic + subject test suites.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::aggregate::Certificate;
use educore_academic::commands::CreateCertificateCommand;
use educore_academic::events::CertificateCreated;
use educore_academic::services::create_certificate;
use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use uuid::Uuid;
use educore_core::error::DomainError;
use educore_academic::value_objects::CertificateId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

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

fn certificate_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> CertificateId {
    CertificateId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create a Certificate template
// =============================================================================

/// End-to-end happy path for the `Certificate` aggregate.
/// Mint a fresh school + actor, build a
/// `CreateCertificateCommand`, and assert that:
///
/// 1. `create_certificate` returns a `Certificate`
///    aggregate carrying the typed `id` and the command's
///    `school_id`.
/// 2. The emitted `CertificateCreated` event has the right
///    `event_type`, `aggregate_type`, and `schema_version`
///    from the `DomainEvent` trait, plus a matching
///    `aggregate_id` and `school_id`.
/// 3. The event's `event_id` is fresh (non-zero) and
///    `occurred_at` is sourced from the test clock.
#[test]
fn certificate_create_builds_aggregate_and_emits_certificate_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateCertificateCommand {
        id: certificate_id(&g, school),
        school_id: school,
    };
    let (agg, created_event) = create_certificate(create_cmd, &clock, &ids).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.id.school_id(), school);
    assert_eq!(agg.school_id, school);

    // Event metadata matches the DomainEvent trait contract.
    assert_eq!(
        <CertificateCreated as DomainEvent>::EVENT_TYPE,
        "academic.certificate.created"
    );
    assert_eq!(
        <CertificateCreated as DomainEvent>::AGGREGATE_TYPE,
        "certificate"
    );
    assert_eq!(<CertificateCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id, agg.id);
    assert_eq!(created_event.school_id, school);
    // The event's id and timestamp are stamped from the
    // generator and clock respectively.
    assert_ne!(created_event.event_id.0, Uuid::nil());
    assert_eq!(created_event.occurred_at, clock.now());
}

// =============================================================================
// 2. Validation failure: school_id mismatch returns DomainError::Validation
// =============================================================================

/// Validation-failure path on the create flow: when the
/// typed id's `school_id()` does not match the command's
/// `school_id`, `create_certificate` returns
/// `DomainError::Validation` and emits no event (the
/// function returns `Err` before the aggregate or the
/// event are constructed).
#[test]
fn certificate_create_with_school_id_mismatch_returns_validation_error() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    // Build the typed id in `school`, then lie about the
    // command's school — the validation guard must catch
    // the mismatch.
    let other_school = g.next_school_id();
    let mismatched_cmd = CreateCertificateCommand {
        id: CertificateId::new(school, g.next_uuid()),
        school_id: other_school,
    };
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let err = create_certificate(mismatched_cmd, &clock, &ids)
        .expect_err("cross-school id must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // Sanity check: a subsequent call with matching
    // id.school_id() and command school_id succeeds,
    // proving the failure was tied to the cross-school id
    // (and not to a corrupt clock, ids, or fixture).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let ok_cmd = CreateCertificateCommand {
        id: CertificateId::new(school, g.next_uuid()),
        school_id: school,
    };
    let (_agg, _event) =
        create_certificate(ok_cmd, &clock, &ids).expect("matching school id must succeed");
}
