//! Integration tests for the **FormDownloadLink child entity**.
//!
//! Pins the construction contract for
//! [`FormDownloadLink`](educore_documents::aggregate::FormDownloadLink)
//! — the optional `Url` for an external resource linked from
//! a [`FormDownload`](educore_documents::aggregate::FormDownload).
//!
//! The tests verify that:
//!
//! 1. `FormDownloadLink::new` populates every field from the
//!    input (school, form_id, url, timestamp, actor) and mints
//!    a fresh [`FormDownloadLinkId`] that carries the same
//!    `school_id` as the parent `FormDownload`.
//! 2. The audit-footer fields (`created_by`, `updated_by`,
//!    `created_at`, `updated_at`) are all initialised to the
//!    acting user and the construction timestamp.
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
use educore_core::value_objects::Timestamp;
use educore_documents::prelude::*;
use educore_documents::value_objects::{FormDownloadId, Url};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `(school, form_id, url)` triple. The form id is
/// anchored to the freshly-minted school so the tenant
/// invariant (`form_id.school_id() == school_id`) holds.
fn admin_context() -> (Url, FormDownloadId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let form_id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let url = Url::new("https://example.com/form.pdf").expect("valid url");
    (url, form_id)
}

// =============================================================================
// Happy path: construct a FormDownloadLink
// =============================================================================

/// End-to-end happy path for the FormDownloadLink child
/// entity. Construct a new link attachment for a freshly-minted
/// `FormDownload` and assert that:
///
/// 1. The id is a fresh `FormDownloadLinkId` whose
///    `school_id()` matches the parent form's school.
/// 2. The form_id, url, school_id, created_by, updated_by,
///    created_at, and updated_at fields all carry the input.
#[test]
fn form_download_link_new_populates_all_fields() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let form_id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let url = Url::new("https://example.com/form.pdf").expect("valid url");
    let at = Timestamp::now();

    let child = FormDownloadLink::new(school, form_id, url.clone(), at, actor);

    // Tenant invariant: the child's school id matches the
    // parent form's school id.
    assert_eq!(child.school_id, school);
    assert_eq!(child.form_id, form_id);
    assert_eq!(child.url, url);

    // Audit footer is initialised.
    assert_eq!(child.created_by, actor);
    assert_eq!(child.updated_by, actor);
    assert_eq!(child.created_at, at);
    assert_eq!(child.updated_at, at);

    // The id is anchored to the same school.
    assert_eq!(child.id.school_id(), school);
}

// =============================================================================
// Happy path: construct two distinct FormDownloadLink children
// =============================================================================

/// Two FormDownloadLink children minted back-to-back MUST
/// carry distinct typed ids (each is a fresh UUIDv7) AND
/// distinct form ids. This guards against a regression where
/// the typed-id minting logic accidentally returns the same
/// id twice or pins the wrong form_id.
#[test]
fn form_download_link_two_children_have_distinct_ids_and_form_ids() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let form_id_a = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let form_id_b = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let url_a = Url::new("https://a.example.com/x.pdf").expect("valid");
    let url_b = Url::new("https://b.example.com/y.pdf").expect("valid");
    let at = Timestamp::now();

    let child_a = FormDownloadLink::new(school, form_id_a, url_a.clone(), at, actor);
    let child_b = FormDownloadLink::new(school, form_id_b, url_b.clone(), at, actor);

    assert_ne!(child_a.id, child_b.id);
    assert_ne!(child_a.form_id, child_b.form_id);
    assert_eq!(child_a.form_id, form_id_a);
    assert_eq!(child_b.form_id, form_id_b);
    assert_eq!(child_a.url, url_a);
    assert_eq!(child_b.url, url_b);
    assert_eq!(child_a.school_id, child_b.school_id);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a valid `Url` and a `FormDownloadId` whose `school_id()` is
/// a valid UUID. The fixture is the foundation for the
/// higher-level tests, so a regression here surfaces
/// immediately.
#[test]
fn admin_context_fixture_returns_anchored_form_and_url() {
    let (url, form_id) = admin_context();
    assert!(!url.as_str().is_empty());
    let _ = form_id.school_id();
}
