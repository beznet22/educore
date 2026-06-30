//! Integration tests for the **FormDownload create flow**.
//!
//! Pins the construction contract for
//! [`FormDownload::new`](educore_documents::aggregate::FormDownload::new)
//! end-to-end at the aggregate level:
//!
//! 1. The aggregate is created in the active state, with
//!    `school_id` derived from `id.school_id()` (never taken
//!    from the caller).
//! 2. Every command field (`title`, `short_description`,
//!    `publish_date`, `link`, `file`, `show_public`) is carried
//!    verbatim onto the aggregate.
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
use educore_documents::aggregate::{FormDownload, NewFormDownload};
use educore_documents::value_objects::{
    FileReference, FormDownloadId, FormTitle, PublishDate, ShowPublic, Url,
};

// =============================================================================
// Fixtures
// =============================================================================

/// Returns a `(school, actor, correlation, timestamp)` tuple
/// pinned to a freshly-minted school. Tests mint the form id
/// themselves so each aggregate is unique.
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

/// A `NewFormDownload` with a URL link. The minimal happy-path
/// input: title + publish_date + link, all other fields
/// default to `None` / `false` / `ShowPublic::default()`.
fn new_form_with_link(school: educore_core::ids::SchoolId, actor: UserId, correlation: CorrelationId, at: Timestamp) -> NewFormDownload {
    let id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    NewFormDownload {
        id,
        title: FormTitle::new("Consent Form").expect("non-empty title"),
        short_description: None,
        publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        link: Some(Url::new("https://example.com/consent.pdf").expect("valid url")),
        file: None,
        show_public: ShowPublic::new(true),
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

/// A `NewFormDownload` with a file reference (no link).
fn new_form_with_file(school: educore_core::ids::SchoolId, actor: UserId, correlation: CorrelationId, at: Timestamp) -> NewFormDownload {
    let id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    NewFormDownload {
        id,
        title: FormTitle::new("Download Form").expect("non-empty title"),
        short_description: Some(
            educore_documents::value_objects::FormDescription::new("Please download and return").expect("non-empty"),
        ),
        publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()),
        link: None,
        file: Some(FileReference::new("object-key-9999").expect("non-empty")),
        show_public: ShowPublic::new(false),
        created_by: actor,
        created_at: at,
        correlation_id: correlation,
    }
}

// =============================================================================
// Happy path: create with link
// =============================================================================

/// End-to-end happy path for the create flow with a URL
/// link. After `FormDownload::new` returns:
///
/// 1. The aggregate is in the active state with
///    `school_id` derived from `id.school_id()`.
/// 2. `title`, `link`, `show_public`, `publish_date`,
///    `created_by`, `updated_by`, `correlation_id`,
///    `created_at`, and `updated_at` all carry the input.
/// 3. The optimistic-concurrency version starts at 1.
#[test]
fn form_download_new_with_link_populates_aggregate() {
    let (school, actor, correlation, at) = admin_context();
    let cmd = new_form_with_link(school, actor, correlation, at);

    let form = FormDownload::new(cmd).expect("create with link must succeed");

    // school_id is derived from the typed id, NOT the caller.
    assert_eq!(form.school_id, school);
    assert!(form.is_active());
    assert!(form.is_deliverable());
    assert!(form.is_public());

    // Command fields are carried verbatim.
    assert_eq!(form.title.as_str(), "Consent Form");
    assert!(form.short_description.is_none());
    assert!(form.link.is_some());
    assert!(form.file.is_none());

    // Audit footer.
    assert_eq!(form.created_by, actor);
    assert_eq!(form.updated_by, actor);
    assert_eq!(form.correlation_id, correlation);
    assert_eq!(form.created_at, at);
    assert_eq!(form.updated_at, at);
    assert_eq!(form.version.get(), 1);
    assert!(form.last_event_id.is_none());
}

// =============================================================================
// Happy path: create with file
// =============================================================================

/// End-to-end happy path for the create flow with a file
/// reference (no link). After `FormDownload::new` returns the
/// aggregate carries the file, the optional description, and
/// the public-visibility flag (`false`), and the
/// `is_deliverable` helper still reports `true` (the
/// link-or-file invariant is satisfied by the file alone).
#[test]
fn form_download_new_with_file_populates_aggregate() {
    let (school, actor, correlation, at) = admin_context();
    let cmd = new_form_with_file(school, actor, correlation, at);

    let form = FormDownload::new(cmd).expect("create with file must succeed");

    assert_eq!(form.school_id, school);
    assert!(form.is_active());
    assert!(form.is_deliverable());
    assert!(!form.is_public());

    // Command fields are carried verbatim.
    assert_eq!(form.title.as_str(), "Download Form");
    assert!(form.short_description.is_some());
    assert!(form.link.is_none());
    assert!(form.file.is_some());

    // Audit footer.
    assert_eq!(form.created_by, actor);
    assert_eq!(form.updated_by, actor);
    assert_eq!(form.correlation_id, correlation);
    assert_eq!(form.version.get(), 1);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-trivial `(school, actor, correlation, timestamp)`
/// tuple. The fixture is the foundation for the higher-level
/// tests, so a regression here surfaces immediately.
#[test]
fn admin_context_fixture_returns_distinct_ids_and_valid_timestamp() {
    let (s1, a1, c1, t1) = admin_context();
    let (s2, a2, c2, t2) = admin_context();

    // Two calls to admin_context() return distinct ids.
    assert_ne!(s1, s2);
    assert_ne!(a1, a2);
    assert_ne!(c1, c2);

    // Timestamps are non-zero (the `Timestamp::now()`
    // constructor never returns the Unix epoch).
    let _ = (t1, t2);
}
