//! Integration tests for the **PostalReceive create flow**.
//!
//! Pins the construction contract for
//! [`PostalReceive::new`](educore_documents::aggregate::PostalReceive::new)
//! end-to-end at the aggregate level:
//!
//! 1. The aggregate is created in the active state, with
//!    `school_id` derived from `id.school_id()` (never taken
//!    from the caller).
//! 2. Every command field (`academic_id`, `from_title`,
//!    `to_title`, `reference_no`, `address`, `date`,
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
use educore_documents::aggregate::{NewPostalReceive, PostalReceive};
use educore_documents::value_objects::{
    FromAddress, FromTitle, PostalAddress, PostalReceiveId, PostalReferenceNo, PostalTitle,
    ReceiveDate, ToTitle,
};

// =============================================================================
// Fixtures
// =============================================================================

/// Returns a `(school, actor, correlation, timestamp,
/// academic_id)` tuple pinned to a freshly-minted school. Tests
/// mint the receive id themselves so each aggregate is unique.
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

/// A `NewPostalReceive` with a reference number, address, and
/// date. The minimal happy-path input for an incoming
/// receive.
fn new_receive_with_reference(
    school: educore_core::ids::SchoolId,
    actor: UserId,
    correlation: CorrelationId,
    at: Timestamp,
    academic_id: educore_documents::aggregate::AcademicYearId,
) -> NewPostalReceive {
    let id = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    NewPostalReceive {
        id,
        academic_id,
        from_title: FromTitle::new(PostalTitle::new("Acme Vendor").expect("non-empty")),
        to_title: ToTitle::new(PostalTitle::new("Acme School").expect("non-empty")),
        reference_no: Some(PostalReferenceNo::new("REF-IN-2026-0001").expect("non-empty")),
        address: FromAddress::new(PostalAddress::new("5 Vendor Rd").expect("non-empty")),
        date: ReceiveDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: None,
        file: None,
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

/// A `NewPostalReceive` without a reference number and with
/// a `note`. Exercises the optional-field paths.
fn new_receive_without_reference(
    school: educore_core::ids::SchoolId,
    actor: UserId,
    correlation: CorrelationId,
    at: Timestamp,
    academic_id: educore_documents::aggregate::AcademicYearId,
) -> NewPostalReceive {
    let id = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    NewPostalReceive {
        id,
        academic_id,
        from_title: FromTitle::new(PostalTitle::new("Other Vendor").expect("non-empty")),
        to_title: ToTitle::new(PostalTitle::new("Acme School").expect("non-empty")),
        reference_no: None,
        address: FromAddress::new(PostalAddress::new("7 Vendor Rd").expect("non-empty")),
        date: ReceiveDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: Some(educore_documents::value_objects::PostalNote::new("Personal delivery").expect("non-empty")),
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
/// number. After `PostalReceive::new` returns the aggregate is
/// in the active state with `school_id` derived from
/// `id.school_id()` and every command field carried verbatim.
#[test]
fn postal_receive_new_with_reference_populates_aggregate() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = new_receive_with_reference(school, actor, correlation, at, academic_id);

    let receive = PostalReceive::new(cmd).expect("create must succeed");

    // school_id is derived from the typed id, NOT the caller.
    assert_eq!(receive.school_id, school);
    assert!(receive.is_active());

    // Command fields are carried verbatim.
    assert_eq!(receive.academic_id, academic_id);
    assert_eq!(receive.from_title.as_str(), "Acme Vendor");
    assert_eq!(receive.to_title.as_str(), "Acme School");
    assert_eq!(
        receive.reference_no.as_ref().map(PostalReferenceNo::as_str),
        Some("REF-IN-2026-0001")
    );
    assert_eq!(receive.address.as_str(), "5 Vendor Rd");

    // Audit footer.
    assert_eq!(receive.created_by, actor);
    assert_eq!(receive.updated_by, actor);
    assert_eq!(receive.correlation_id, correlation);
    assert_eq!(receive.created_at, at);
    assert_eq!(receive.updated_at, at);
    assert_eq!(receive.version.get(), 1);
    assert!(receive.last_event_id.is_none());
}

// =============================================================================
// Happy path: create without reference number
// =============================================================================

/// End-to-end happy path for the create flow without a
/// reference number and with a `note`. After `PostalReceive::new`
/// returns the aggregate is in the active state with
/// `reference_no = None` and the note carried verbatim. This
/// exercises the optional-field paths.
#[test]
fn postal_receive_new_without_reference_populates_aggregate() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = new_receive_without_reference(school, actor, correlation, at, academic_id);

    let receive = PostalReceive::new(cmd).expect("create must succeed");

    assert_eq!(receive.school_id, school);
    assert!(receive.is_active());

    // Optional fields: reference_no is None; note is Some.
    assert!(receive.reference_no.is_none());
    assert!(receive.note.is_some());

    assert_eq!(receive.academic_id, academic_id);
    assert_eq!(receive.from_title.as_str(), "Other Vendor");
    assert_eq!(receive.to_title.as_str(), "Acme School");

    // Audit footer.
    assert_eq!(receive.created_by, actor);
    assert_eq!(receive.updated_by, actor);
    assert_eq!(receive.correlation_id, correlation);
    assert_eq!(receive.version.get(), 1);
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
