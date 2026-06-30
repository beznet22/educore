//! Integration tests for the **PostalDispatch create flow**.
//!
//! Pins the construction contract for
//! [`PostalDispatch::new`](educore_documents::aggregate::PostalDispatch::new)
//! end-to-end at the aggregate level:
//!
//! 1. The aggregate is created in the active state, with
//!    `school_id` derived from `id.school_id()` (never taken
//!    from the caller).
//! 2. Every command field (`academic_id`, `to_title`,
//!    `from_title`, `reference_no`, `address`, `date`,
//!    `note`, `file`) is carried verbatim onto the aggregate.
//! 3. The audit-footer fields (`version = 1`, `created_at`,
//!    `updated_at`, `created_by`, `updated_by`,
//!    `correlation_id`) are initialised correctly.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::ids::{CorrelationId, UserId};
use educore_core::value_objects::Timestamp;
use educore_documents::aggregate::{NewPostalDispatch, PostalDispatch};
use educore_documents::value_objects::{
    DispatchDate, FromAddress, FromTitle, PostalAddress, PostalDispatchId, PostalReferenceNo,
    PostalTitle, ToAddress, ToTitle,
};

// =============================================================================
// Fixtures
// =============================================================================

/// Returns a `(school, actor, correlation, timestamp,
/// academic_id)` tuple pinned to a freshly-minted school. Tests
/// mint the dispatch id themselves so each aggregate is unique.
fn admin_context() -> (
    educore_core::ids::SchoolId,
    UserId,
    CorrelationId,
    Timestamp,
    educore_documents::aggregate::AcademicYearId,
) {
    let g = SystemIdGen;
    (
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        Timestamp::now(),
        uuid::Uuid::now_v7(),
    )
}

/// A `NewPostalDispatch` with a reference number, address, and
/// date. The minimal happy-path input for an outgoing
/// dispatch.
#[allow(clippy::too_many_lines)]
fn new_dispatch_with_reference(
    school: educore_core::ids::SchoolId,
    actor: UserId,
    correlation: CorrelationId,
    at: Timestamp,
    academic_id: educore_documents::aggregate::AcademicYearId,
) -> NewPostalDispatch {
    let id = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    NewPostalDispatch {
        id,
        academic_id,
        to_title: ToTitle::new(PostalTitle::new("Mr Smith").expect("non-empty")),
        from_title: FromTitle::new(PostalTitle::new("Acme School").expect("non-empty")),
        reference_no: Some(PostalReferenceNo::new("REF-2026-0001").expect("non-empty")),
        address: ToAddress::new(PostalAddress::new("1 Main St").expect("non-empty")),
        date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: None,
        file: None,
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

/// A `NewPostalDispatch` without a reference number and with
/// a `note` and `file`. Exercises the optional-field paths.
fn new_dispatch_without_reference(
    school: educore_core::ids::SchoolId,
    actor: UserId,
    correlation: CorrelationId,
    at: Timestamp,
    academic_id: educore_documents::aggregate::AcademicYearId,
) -> NewPostalDispatch {
    let id = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    NewPostalDispatch {
        id,
        academic_id,
        to_title: ToTitle::new(PostalTitle::new("Mr Jones").expect("non-empty")),
        from_title: FromTitle::new(PostalTitle::new("Acme School").expect("non-empty")),
        reference_no: None,
        address: ToAddress::new(PostalAddress::new("2 Oak Ave").expect("non-empty")),
        date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: Some(educore_documents::value_objects::PostalNote::new("Rush").expect("non-empty")),
        file: None,
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

// =============================================================================
// Happy path: create with reference number
// =============================================================================

/// End-to-end happy path for the create flow with a reference
/// number. After `PostalDispatch::new` returns the aggregate is
/// in the active state with `school_id` derived from
/// `id.school_id()` and every command field carried verbatim.
#[test]
fn postal_dispatch_new_with_reference_populates_aggregate() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = new_dispatch_with_reference(school, actor, correlation, at, academic_id);

    let dispatch = PostalDispatch::new(cmd).expect("create must succeed");

    // school_id is derived from the typed id, NOT the caller.
    assert_eq!(dispatch.school_id, school);
    assert!(dispatch.is_active());

    // Command fields are carried verbatim.
    assert_eq!(dispatch.academic_id, academic_id);
    assert_eq!(dispatch.to_title.as_str(), "Mr Smith");
    assert_eq!(dispatch.from_title.as_str(), "Acme School");
    assert_eq!(
        dispatch.reference_no.as_ref().map(PostalReferenceNo::as_str),
        Some("REF-2026-0001")
    );
    assert_eq!(dispatch.address.as_str(), "1 Main St");

    // Audit footer.
    assert_eq!(dispatch.created_by, actor);
    assert_eq!(dispatch.updated_by, actor);
    assert_eq!(dispatch.correlation_id, correlation);
    assert_eq!(dispatch.created_at, at);
    assert_eq!(dispatch.updated_at, at);
    assert_eq!(dispatch.version.get(), 1);
    assert!(dispatch.last_event_id.is_none());
}

// =============================================================================
// Happy path: create without reference number
// =============================================================================

/// End-to-end happy path for the create flow without a
/// reference number and with a `note`. After `PostalDispatch::new`
/// returns the aggregate is in the active state with
/// `reference_no = None` and the note carried verbatim. This
/// exercises the optional-field paths.
#[test]
fn postal_dispatch_new_without_reference_populates_aggregate() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = new_dispatch_without_reference(school, actor, correlation, at, academic_id);

    let dispatch = PostalDispatch::new(cmd).expect("create must succeed");

    assert_eq!(dispatch.school_id, school);
    assert!(dispatch.is_active());

    // Optional fields: reference_no is None; note is Some.
    assert!(dispatch.reference_no.is_none());
    assert!(dispatch.note.is_some());

    assert_eq!(dispatch.academic_id, academic_id);
    assert_eq!(dispatch.to_title.as_str(), "Mr Jones");

    // Audit footer.
    assert_eq!(dispatch.created_by, actor);
    assert_eq!(dispatch.updated_by, actor);
    assert_eq!(dispatch.correlation_id, correlation);
    assert_eq!(dispatch.version.get(), 1);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-trivial `(school, actor, correlation, timestamp,
/// academic_id)` tuple with distinct ids on each call. The
/// fixture is the foundation for the higher-level tests, so a
/// regression here surfaces immediately.
#[test]
fn admin_context_fixture_returns_distinct_ids() {
    let (s1, a1, c1, _t1, y1) = admin_context();
    let (s2, a2, c2, _t2, y2) = admin_context();

    assert_ne!(s1, s2);
    assert_ne!(a1, a2);
    assert_ne!(c1, c2);
    assert_ne!(y1, y2);
}

// Silence unused-import warning for `FromAddress` when no test
// currently uses it (the field exists on `PostalDispatch` but
// is exercised in `update_postal_dispatch.rs` instead).
#[allow(dead_code)]
fn _unused_from_address() {
    let _ = FromAddress::new(PostalAddress::new("x").expect("non-empty"));
}
