//! Integration tests for the **Weekend aggregate** vertical slice.
//!
//! Pins the constructor + update contract for
//! [`Weekend`](educore_events_domain::aggregate::Weekend)
//! end-to-end through the aggregate layer:
//!
//! 1. `Weekend::new(...)` validates the input (non-empty
//!    `name`, `order` in `0..=7`), constructs the aggregate
//!    with `version = Version::initial()` and
//!    `active_status = true`, and stores the supplied
//!    `is_weekend` flag and optional `academic_id`.
//! 2. `Weekend::update(...)` validates the in-place fields
//!    (non-empty `name`, `order` in `0..=7`), mutates the
//!    aggregate (swaps `name` / `order` / `is_weekend`,
//!    bumps `version`, updates `updated_at` / `updated_by`).
//!
//! The tests follow the **constructor + update pattern** of
//! this domain: no factory handlers, no events emitted from
//! the constructor. The handlers / outbox / audit fan-out
//! are not yet wired end-to-end; these tests pin the
//! **aggregate layer** contract that the service factory
//! fns and dispatcher will eventually wrap.
//!
//! Implements: `docs/specs/events/aggregates.md` ## Weekend
//! and `docs/specs/events/workflows.md` ## "Weekend
//! Configuration Workflow".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events_domain::aggregate::Weekend;
use educore_events_domain::errors::EventsDomainError;
use educore_events_domain::prelude::*;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh school id + system actor id from a single
/// `SystemIdGen`. Mirrors the fixture style used in
/// `tests/workflows.rs`.
fn fixture_ids() -> (SchoolId, UserId, Timestamp) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let at = TestClock::new().now();
    (school, actor, at)
}

// =============================================================================
// 1. Happy path: create + update on Weekend
// =============================================================================

/// End-to-end happy path for the Weekend aggregate.
/// Create a weekend named "Friday" with `order = 5`,
/// `is_weekend = true`, then rename it to "FRI" via
/// `update`, asserting that:
///
/// 1. `Weekend::new` returns an aggregate whose
///    `name`, `order`, `is_weekend`, `version`,
///    `active_status`, and `created_by` all match the
///    command inputs.
#[test]
fn weekend_create_then_update_mutates_aggregate_and_bumps_version() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = WeekendId::new(school, g.next_uuid());

    // ---- Create flow ----
    let mut weekend = Weekend::new(id, "Friday".to_owned(), 5, true, None, actor, ts)
        .expect("Weekend::new must succeed for valid input");

    // Aggregate fields are populated from the constructor.
    assert_eq!(weekend.id, id);
    assert_eq!(weekend.school_id, school);
    assert_eq!(weekend.name, "Friday");
    assert_eq!(weekend.order, 5);
    assert!(weekend.is_weekend);
    assert_eq!(weekend.academic_id, None);
    assert_eq!(weekend.created_by, actor);
    assert_eq!(weekend.updated_by, actor);
    assert_eq!(
        weekend.version,
        educore_core::value_objects::Version::initial()
    );
    assert!(weekend.active_status);
    assert_eq!(weekend.created_at, ts);
    assert_eq!(weekend.updated_at, ts);

    // ---- Update flow ----
    let version_before = weekend.version;
    let updated_at = Timestamp::now();
    weekend
        .update(
            Some("FRI".to_owned()),
            Some(5),
            Some(true),
            actor,
            updated_at,
        )
        .expect("Weekend::update must succeed for valid input");

    // The aggregate is mutated in place.
    assert_eq!(weekend.name, "FRI");
    assert_eq!(weekend.order, 5);
    assert!(weekend.is_weekend);
    assert_eq!(weekend.updated_by, actor);
    assert_eq!(weekend.updated_at, updated_at);
    assert_eq!(
        weekend.version,
        version_before.next(),
        "version must be bumped exactly once"
    );
}

// =============================================================================
// 2. Validation failure: order out of range
// =============================================================================

/// Validation-failure path on the create flow: per spec
/// invariant 2, the `order` field must be in `0..=7`.
/// `Weekend::new` must reject `order = 8` with
/// `EventsDomainError::Validation`.
#[test]
fn weekend_create_with_order_out_of_range_returns_validation_error() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = WeekendId::new(school, g.next_uuid());

    let res = Weekend::new(id, "Bad".to_owned(), 8, true, None, actor, ts);
    let err = res.expect_err("order = 8 must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "got {err:?}"
    );
}
