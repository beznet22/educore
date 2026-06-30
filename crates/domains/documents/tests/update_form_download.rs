//! Integration tests for the **FormDownload update flow**.
//!
//! Pins the in-place mutation contract for
//! [`FormDownload::update`](educore_documents::aggregate::FormDownload::update)
//! end-to-end at the aggregate level:
//!
//! 1. The update mutates the aggregate in place: the supplied
//!    fields (`title`, `short_description`, `publish_date`,
//!    `link`, `file`, `show_public`) are written; the
//!    optimistic-concurrency `version` is bumped; the
//!    `updated_at` / `updated_by` audit fields move to the
//!    actor and timestamp on the update; `last_event_id` is
//!    stamped with the event id from the command.
//! 2. Fields that are NOT in the update command remain
//!    unchanged (the `Option<Option<T>>` 3-state pattern for
//!    `short_description`, `link`, and `file`; the 2-state
//!    pattern for `title`, `publish_date`, `show_public`).
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
use educore_documents::aggregate::{
    FormDownload, NewFormDownload, UpdateFormDownload,
};
use educore_documents::value_objects::{
    FileReference, FormDescription, FormDownloadId, FormTitle, PublishDate, ShowPublic, Url,
};

// =============================================================================
// Fixtures
// =============================================================================

/// Returns a `(school, actor, correlation, timestamp)` tuple
/// pinned to a freshly-minted school.
fn admin_context() -> (
    educore_core::ids::SchoolId,
    UserId,
    CorrelationId,
    Timestamp,
) {
    let g = SystemIdGen;
    (
        g.next_school_id(),
        g.next_user_id(),
        g.next_correlation_id(),
        Timestamp::now(),
    )
}

/// A `NewFormDownload` with a URL link. Minimal happy-path
/// input for the seed aggregate.
fn seed_form_with_link(
    school: educore_core::ids::SchoolId,
    actor: UserId,
    correlation: CorrelationId,
    at: Timestamp,
) -> NewFormDownload {
    let id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    NewFormDownload {
        id,
        title: FormTitle::new("Original Title").expect("non-empty title"),
        short_description: None,
        publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        link: Some(Url::new("https://example.com/original.pdf").expect("valid url")),
        file: None,
        show_public: ShowPublic::new(false),
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

// =============================================================================
// Happy path: rename + add description + flip visibility
// =============================================================================

/// End-to-end happy path for the update flow: rename the
/// form, add a description, and flip the public-visibility
/// flag. After `FormDownload::update` returns:
///
/// 1. `title` carries the new value.
/// 2. `short_description` is `Some(Some(...))` — a "set"
///    in the 3-state `Option<Option<T>>` pattern.
/// 3. `show_public` carries the new flag.
/// 4. `version` is bumped from 1 to 2.
/// 5. `updated_at` / `updated_by` / `last_event_id` carry the
///    actor, timestamp, and event id from the update command.
/// 6. Fields NOT in the update command (`link`,
///    `publish_date`) remain unchanged.
#[test]
fn form_download_update_renames_and_adds_description_and_bumps_version() {
    let (school, actor, correlation, at) = admin_context();
    let cmd = seed_form_with_link(school, actor, correlation, at);
    let mut form = FormDownload::new(cmd).expect("seed");
    let initial_version = form.version.get();

    let new_actor = SystemIdGen.next_user_id();
    let new_at = Timestamp::now();
    let new_event_id = EventId(uuid::Uuid::now_v7());
    let update = UpdateFormDownload {
        title: Some(FormTitle::new("Renamed Title").expect("non-empty")),
        short_description: Some(Some(
            FormDescription::new("Added description").expect("non-empty"),
        )),
        publish_date: None,
        link: None,
        file: None,
        show_public: Some(ShowPublic::new(true)),
        actor: new_actor,
        at: new_at,
        event_id: new_event_id,
    };
    form.update(update).expect("update ok");

    // Fields in the update command moved.
    assert_eq!(form.title.as_str(), "Renamed Title");
    assert!(form.short_description.is_some());
    assert!(form.is_public());

    // Fields NOT in the update command remain unchanged.
    assert!(form.link.is_some());
    let initial_publish_date = PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    assert_eq!(form.publish_date, initial_publish_date);

    // Audit footer is updated.
    assert_eq!(form.version.get(), initial_version + 1);
    assert_eq!(form.updated_by, new_actor);
    assert_eq!(form.updated_at, new_at);
    assert_eq!(form.last_event_id, Some(new_event_id));

    // created_* fields are NOT touched by an update.
    assert_eq!(form.created_by, actor);
    assert_eq!(form.created_at, at);
}

// =============================================================================
// Happy path: swap link for file (3-state semantics on `file`)
// =============================================================================

/// End-to-end happy path for the update flow: clear the
/// existing `link` (None in the inner option) and set a
/// `file`. This pins the `Option<Option<T>>` 3-state
/// semantics for both `link` and `file`: outer `Some(None)`
/// means "clear the field"; outer `Some(Some(_))` means
/// "set the field". The link-or-file invariant (the form
/// must still have at least one of `link` or `file`) MUST
/// hold after the update — clearing the link and setting the
/// file satisfies it.
#[test]
fn form_download_update_swaps_link_for_file_with_3state_semantics() {
    let (school, actor, correlation, at) = admin_context();
    let cmd = seed_form_with_link(school, actor, correlation, at);
    let mut form = FormDownload::new(cmd).expect("seed");
    let initial_version = form.version.get();

    let new_event_id = EventId(uuid::Uuid::now_v7());
    let update = UpdateFormDownload {
        title: None,
        short_description: None,
        publish_date: None,
        // Clear the existing link (3-state: Some(None) = clear).
        link: Some(None),
        // Set a new file (3-state: Some(Some(_)) = set).
        file: Some(Some(FileReference::new("new-object-key").expect("non-empty"))),
        show_public: None,
        actor: SystemIdGen.next_user_id(),
        at: Timestamp::now(),
        event_id: new_event_id,
    };
    form.update(update).expect("update ok");

    // link is cleared, file is set.
    assert!(form.link.is_none());
    assert!(form.file.is_some());
    // The form is still deliverable (the invariant is
    // satisfied by the file alone).
    assert!(form.is_deliverable());
    // Version bumps.
    assert_eq!(form.version.get(), initial_version + 1);
    assert_eq!(form.last_event_id, Some(new_event_id));
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-trivial `(school, actor, correlation, timestamp)`
/// tuple. The fixture is the foundation for the higher-level
/// tests, so a regression here surfaces immediately.
#[test]
fn admin_context_fixture_returns_distinct_ids() {
    let (s1, a1, c1, _t1) = admin_context();
    let (s2, a2, c2, _t2) = admin_context();
    assert_ne!(s1, s2);
    assert_ne!(a1, a2);
    assert_ne!(c1, c2);
}
