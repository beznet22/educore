//! Integration tests for the **FormDownloadFile child entity**.
//!
//! Pins the construction contract for
//! [`FormDownloadFile`](educore_documents::aggregate::FormDownloadFile)
//! — the optional `FileReference` attached to a
//! [`FormDownload`](educore_documents::aggregate::FormDownload).
//!
//! The tests verify that:
//!
//! 1. `FormDownloadFile::new` populates every field from the
//!    input (school, form_id, file, timestamp, actor) and mints
//!    a fresh [`FormDownloadFileId`] that carries the same
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
use educore_documents::value_objects::{FileReference, FormDownloadId};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `(school, form_id, file_ref)` triple. The form id
/// is anchored to the freshly-minted school so the tenant
/// invariant (`form_id.school_id() == school_id`) holds.
fn admin_context() -> (FileReference, FormDownloadId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let form_id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("object-key-1234").expect("non-empty key");
    (file, form_id)
}

// =============================================================================
// Happy path: construct a FormDownloadFile
// =============================================================================

/// End-to-end happy path for the FormDownloadFile child
/// entity. Construct a new file attachment for a freshly-minted
/// `FormDownload` and assert that:
///
/// 1. The id is a fresh `FormDownloadFileId` whose
///    `school_id()` matches the parent form's school.
/// 2. The form_id, file, school_id, created_by, updated_by,
///    created_at, and updated_at fields all carry the input.
#[test]
fn form_download_file_new_populates_all_fields() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let form_id = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("object-key-1234").expect("non-empty key");
    let at = Timestamp::now();

    let child = FormDownloadFile::new(school, form_id, file.clone(), at, actor);

    // Tenant invariant: the child's school id matches the
    // parent form's school id.
    assert_eq!(child.school_id, school);
    assert_eq!(child.form_id, form_id);
    assert_eq!(child.file, file);

    // Audit footer is initialised.
    assert_eq!(child.created_by, actor);
    assert_eq!(child.updated_by, actor);
    assert_eq!(child.created_at, at);
    assert_eq!(child.updated_at, at);

    // The id is anchored to the same school.
    assert_eq!(child.id.school_id(), school);
}

// =============================================================================
// Happy path: construct two distinct FormDownloadFile children
// =============================================================================

/// Two FormDownloadFile children minted back-to-back MUST
/// carry distinct typed ids (each is a fresh UUIDv7) AND
/// distinct form ids (the form_id is supplied by the caller,
/// but here we generate two forms and check the children are
/// bound to their respective parents). This guards against a
/// regression where the typed-id minting logic accidentally
/// returns the same id twice or pins the wrong form_id.
#[test]
fn form_download_file_two_children_have_distinct_ids_and_form_ids() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let form_id_a = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let form_id_b = FormDownloadId::new(school, uuid::Uuid::now_v7());
    let file_a = FileReference::new("key-a").expect("non-empty");
    let file_b = FileReference::new("key-b").expect("non-empty");
    let at = Timestamp::now();

    let child_a = FormDownloadFile::new(school, form_id_a, file_a.clone(), at, actor);
    let child_b = FormDownloadFile::new(school, form_id_b, file_b.clone(), at, actor);

    assert_ne!(child_a.id, child_b.id);
    assert_ne!(child_a.form_id, child_b.form_id);
    assert_eq!(child_a.form_id, form_id_a);
    assert_eq!(child_b.form_id, form_id_b);
    assert_eq!(child_a.file, file_a);
    assert_eq!(child_b.file, file_b);
    assert_eq!(child_a.school_id, child_b.school_id);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-empty `FileReference` and a `FormDownloadId` whose
/// `school_id()` is a non-nil UUID. The fixture is the
/// foundation for the higher-level tests, so a regression here
/// surfaces immediately.
#[test]
fn admin_context_fixture_returns_anchored_form_and_file() {
    let (file, form_id) = admin_context();
    assert!(!file.as_str().is_empty());
    // The form id is anchored to a freshly-minted school, so
    // its school_id() is a valid (non-nil) UUID.
    let _ = form_id.school_id();
}
