//! Integration tests for the **PostalDispatch update flow**.
//!
//! Pins the in-place mutation contract for
//! [`PostalDispatch::update`](educore_documents::aggregate::PostalDispatch::update)
//! end-to-end at the aggregate level:
//!
//! 1. The update mutates the aggregate in place: the supplied
//!    fields (`academic_id`, `to_title`, `from_title`,
//!    `address`, `date`, `note`, `file`) are written; the
//!    optimistic-concurrency `version` is bumped; the
//!    `updated_at` / `updated_by` audit fields move to the
//!    actor and timestamp on the update; `last_event_id` is
//!    stamped with the event id from the command.
//! 2. The `reference_no` is **immutable once set**: attempting
//!    to change or clear it via the `reference_no` field on
//!    `UpdatePostalDispatch` returns
//!    [`DocumentsError::ReferenceNoImmutable`]. An idempotent
//!    re-send of the existing reference (some-no-op) does NOT
//!    mutate the aggregate but also does NOT error.
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
use educore_core::ids::{CorrelationId, EventId, UserId};
use educore_core::value_objects::Timestamp;
use educore_documents::aggregate::{NewPostalDispatch, PostalDispatch, UpdatePostalDispatch};
use educore_documents::errors::DocumentsError;
use educore_documents::value_objects::{
    DispatchDate, FromAddress, FromTitle, PostalAddress, PostalDispatchId, PostalReferenceNo,
    PostalTitle, ToAddress, ToTitle,
};

// =============================================================================
// Fixtures
// =============================================================================

/// Returns a `(school, actor, correlation, timestamp,
/// academic_id)` tuple pinned to a freshly-minted school.
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
/// date. Used to seed the aggregate for the update flow.
fn seed_dispatch(
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

// =============================================================================
// Happy path: rename + change address + add a note
// =============================================================================

/// End-to-end happy path for the update flow: rename the
/// recipient, change the address, and add a note. After
/// `PostalDispatch::update` returns:
///
/// 1. `to_title`, `address`, and `note` carry the new values.
/// 2. `reference_no` is unchanged (immutable once set; we
///    left `reference_no = None` on the update command, which
///    is a no-op per the aggregate's check).
/// 3. `version` is bumped from 1 to 2.
/// 4. `updated_at` / `updated_by` / `last_event_id` carry the
///    actor, timestamp, and event id from the update command.
#[test]
fn postal_dispatch_update_renames_and_changes_address_and_adds_note() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = seed_dispatch(school, actor, correlation, at, academic_id);
    let mut dispatch = PostalDispatch::new(cmd).expect("seed");
    let initial_version = dispatch.version.get();

    let new_actor = SystemIdGen.next_user_id();
    let new_at = Timestamp::now();
    let new_event_id = EventId(uuid::Uuid::now_v7());
    let update = UpdatePostalDispatch {
        academic_id: None,
        to_title: Some(ToTitle::new(PostalTitle::new("Mr Smith Jr.").expect("non-empty"))),
        from_title: None,
        // reference_no is NOT being changed; None on the
        // update command means "no change" — the aggregate's
        // immutable-check skips when the outer option is None.
        reference_no: None,
        address: Some(ToAddress::new(PostalAddress::new("99 New St").expect("non-empty"))),
        date: None,
        note: Some(Some(
            educore_documents::value_objects::PostalNote::new("Updated note").expect("non-empty"),
        )),
        file: None,
        actor: new_actor,
        at: new_at,
        event_id: new_event_id,
    };
    dispatch.update(update).expect("update ok");

    // Fields in the update command moved.
    assert_eq!(dispatch.to_title.as_str(), "Mr Smith Jr.");
    assert_eq!(dispatch.address.as_str(), "99 New St");
    assert!(dispatch.note.is_some());

    // Fields NOT in the update command remain unchanged.
    assert_eq!(dispatch.from_title.as_str(), "Acme School");
    assert_eq!(
        dispatch.reference_no.as_ref().map(PostalReferenceNo::as_str),
        Some("REF-2026-0001")
    );
    assert_eq!(dispatch.academic_id, academic_id);

    // Audit footer is updated.
    assert_eq!(dispatch.version.get(), initial_version + 1);
    assert_eq!(dispatch.updated_by, new_actor);
    assert_eq!(dispatch.updated_at, new_at);
    assert_eq!(dispatch.last_event_id, Some(new_event_id));

    // created_* fields are NOT touched by an update.
    assert_eq!(dispatch.created_by, actor);
    assert_eq!(dispatch.created_at, at);
}

// =============================================================================
// Happy path: clear a note via 3-state semantics
// =============================================================================

/// End-to-end happy path for the update flow: clear an
/// existing `note`. The seed dispatch has no note; we first
/// set a note via `update`, then clear it via the 3-state
/// `Option<Option<T>>` pattern (outer `Some(None)` means
/// "clear"). The aggregate must end up with `note = None`.
#[test]
fn postal_dispatch_update_clears_note_with_3state_semantics() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = seed_dispatch(school, actor, correlation, at, academic_id);
    let mut dispatch = PostalDispatch::new(cmd).expect("seed");

    // Step 1: set a note.
    let event_id_set = EventId(uuid::Uuid::now_v7());
    dispatch
        .update(UpdatePostalDispatch {
            academic_id: None,
            to_title: None,
            from_title: None,
            reference_no: None,
            address: None,
            date: None,
            note: Some(Some(
                educore_documents::value_objects::PostalNote::new("set").expect("non-empty"),
            )),
            file: None,
            actor: SystemIdGen.next_user_id(),
            at: Timestamp::now(),
            event_id: event_id_set,
        })
        .expect("set note ok");
    assert!(dispatch.note.is_some());

    // Step 2: clear the note (3-state: Some(None) = clear).
    let initial_version = dispatch.version.get();
    let event_id_clear = EventId(uuid::Uuid::now_v7());
    dispatch
        .update(UpdatePostalDispatch {
            academic_id: None,
            to_title: None,
            from_title: None,
            reference_no: None,
            address: None,
            date: None,
            note: Some(None),
            file: None,
            actor: SystemIdGen.next_user_id(),
            at: Timestamp::now(),
            event_id: event_id_clear,
        })
        .expect("clear note ok");

    assert!(dispatch.note.is_none());
    assert_eq!(dispatch.version.get(), initial_version + 1);
    assert_eq!(dispatch.last_event_id, Some(event_id_clear));
}

// =============================================================================
// Validation failure: changing reference_no is rejected
// =============================================================================

/// Validation failure path: `reference_no` is immutable once
/// set. Attempting to change it to a different value returns
/// [`DocumentsError::ReferenceNoImmutable`]. The aggregate is
/// left untouched (no version bump).
#[test]
fn postal_dispatch_update_with_reference_no_change_returns_reference_no_immutable() {
    let (school, actor, correlation, at, academic_id) = admin_context();
    let cmd = seed_dispatch(school, actor, correlation, at, academic_id);
    let mut dispatch = PostalDispatch::new(cmd).expect("seed");
    let initial_version = dispatch.version.get();

    let update = UpdatePostalDispatch {
        academic_id: None,
        to_title: None,
        from_title: None,
        // Attempt to change the reference number.
        reference_no: Some(Some(
            PostalReferenceNo::new("REF-OTHER").expect("non-empty"),
        )),
        address: None,
        date: None,
        note: None,
        file: None,
        actor: SystemIdGen.next_user_id(),
        at: Timestamp::now(),
        event_id: EventId(uuid::Uuid::now_v7()),
    };
    let err = dispatch.update(update).expect_err("reference_no change must fail");
    assert!(
        matches!(err, DocumentsError::ReferenceNoImmutable),
        "expected ReferenceNoImmutable, got {err:?}"
    );

    // The aggregate is unchanged.
    assert_eq!(dispatch.version.get(), initial_version);
    assert_eq!(
        dispatch.reference_no.as_ref().map(PostalReferenceNo::as_str),
        Some("REF-2026-0001")
    );
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

// Silence unused-import warning for `FromAddress` (the field
// exists on the aggregate but is exercised in other tests).
#[allow(dead_code)]
fn _unused_from_address() {
    let _ = FromAddress::new(PostalAddress::new("x").expect("non-empty"));
}
