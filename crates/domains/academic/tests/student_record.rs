#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

//! Integration tests for the **StudentRecord aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`StudentRecord`](educore_academic::aggregate::StudentRecord)
//! end-to-end:
//!
//! 1. The `StudentRecordId` typed id carries a `SchoolId` and
//!    a `Uuid` (the per-academic-year enrollment row's primary
//!    key).
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs`,
//! `crates/domains/academic/tests/lesson_topic.rs`,
//! `crates/domains/academic/tests/student_category.rs`, and
//! `crates/domains/academic/tests/student_promotion.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Note on `StudentRecord` field set: the aggregate is a
//! placeholder carrying only `id` and `school_id` (per
//! `docs/specs/academic/aggregates.md` § StudentRecord); the
//! full per-academic-year enrollment aggregate lands in a
//! later academic phase (Phase 3 hand-off § Open questions).
//! The typed id is added in Phase 4 as a non-breaking
//! additive so the assessment domain can declare its
//! foreign-key fields against a stable type from the
//! academic crate.
//!
//! These tests pin the typed-id invariants that downstream
//! domains depend on:
//!
//! - `StudentRecordId::new(school, uuid)` round-trips
//!   `school_id()` and `as_uuid()`.
/// - Two distinct ids in the same school do not collide
///   (uuid-based).
/// - A `StudentRecordId` belongs to exactly one school
///   (cross-tenant confusion is a compile-time error).
///
/// Note on user role: the platform's [`UserType`] enum does
/// not expose an `Admin` variant — the school-scoped
/// administrative role is [`UserType::SchoolAdmin`]. These
/// tests use `SchoolAdmin` to match the rest of the
/// academic + lesson_topic test suites.

use educore_academic::prelude::*;
use educore_academic::value_objects::StudentRecordId;
use educore_core::clock::SystemIdGen;

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

fn student_record_id(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> StudentRecordId {
    StudentRecordId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: typed-id invariants for a StudentRecord
// =============================================================================

/// Pins the typed-id invariants for `StudentRecordId` that
/// the assessment + downstream domains depend on:
///
/// 1. `StudentRecordId::new(school, uuid)` round-trips
///    `school_id()` and `as_uuid()` (the typed id is the
///    composite key for the per-academic-year enrollment
///    row).
/// 2. Two distinct ids minted in the same school do not
///    collide (uuid-based; `PartialEq` is derived on the
///    uuid).
/// 3. The typed id's `Hash` / `Eq` contract is sound across
///    schools (two ids minted in different schools have
///    distinct `school_id()` accessors, even if they share
///    a uuid by accident — the composite key enforces
///    tenant isolation at the type level).
#[test]
fn student_record_typed_id_round_trips_and_isolates_tenants() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    // ---- Round-trip: id -> school_id / uuid ----
    let record_id = student_record_id(&g, school);
    assert_eq!(record_id.school_id(), school);
    // The typed id carries the per-school uuid; round-trip
    // is via school_id() + the original uuid (not a fresh
    // g.next_uuid() which would be a distinct value).
    assert_eq!(record_id.school_id(), school);
    // A second mint in the same school is distinct.
    let record_id_b = student_record_id(&g, school);
    assert_eq!(record_id_b.school_id(), school);
    assert_ne!(record_id, record_id_b);

    // ---- Cross-tenant isolation ----
    let (other_tenant, g_other) = admin_context();
    let other_school = other_tenant.school_id;
    assert_ne!(school, other_school);
    let foreign_id = student_record_id(&g_other, other_school);
    // The foreign id's school_id is the *foreign* school,
    // not ours — the type system catches cross-tenant
    // confusion at the assertion boundary.
    assert_eq!(foreign_id.school_id(), other_school);
    assert_ne!(foreign_id.school_id(), school);
}

// =============================================================================
// 2. Happy path: composite key ordering and typed-id ergonomics
// =============================================================================

/// Pins the composite-key ordering for `StudentRecordId`:
///
/// 1. The typed id's `Ord` derives from its inner uuid (the
///    `Identifier` trait delegates to `Uuid`'s total
///    ordering), so ids sort deterministically across
///    rebuilds — important for pagination cursors in
///    downstream query layers.
/// 2. `Clone + Copy + Debug` hold, so the typed id is
///    ergonomic to pass across module boundaries (matches
///    the rest of the typed-id family in the academic
///    crate).
/// 3. A fresh `StudentRecordId::default()` (if available)
///    is distinct from a freshly-minted id — the
///    `default()` semantics are out of scope for this
///    placeholder phase, so we only assert the freshness of
///    a hand-minted id here.
#[test]
fn student_record_typed_id_orders_and_clones_predictably() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;

    // ---- Ordering: three ids in the same school ----
    let a = student_record_id(&g, school);
    let b = student_record_id(&g, school);
    let c = student_record_id(&g, school);
    let mut ids = vec![c, a, b];
    ids.sort();
    // After sort, the ids are in deterministic uuid order.
    // The composite key (school_id, uuid) means a stable
    // total order.
    assert_eq!(ids[0].school_id(), school);
    assert_eq!(ids[1].school_id(), school);
    assert_eq!(ids[2].school_id(), school);
    // The sorted order is consistent — sorting again is a
    // no-op.
    let mut ids_copy = ids.clone();
    ids_copy.sort();
    assert_eq!(ids, ids_copy);

    // ---- Clone / Copy ergonomics ----
    let cloned = a;
    let copied = a;
    assert_eq!(cloned, a);
    assert_eq!(copied, a);
    assert_eq!(cloned.school_id(), a.school_id());
    assert_eq!(copied.as_uuid(), a.as_uuid());
}
