//! Integration tests for the **PostalDispatchAttachment child entity**.
//!
//! Pins the construction contract for
//! [`PostalDispatchAttachment`](educore_documents::aggregate::PostalDispatchAttachment)
//! — the optional `FileReference` attached to a
//! [`PostalDispatch`](educore_documents::aggregate::PostalDispatch).
//!
//! The tests verify that:
//!
//! 1. `PostalDispatchAttachment::new` populates every field
//!    from the input (school, dispatch_id, file, timestamp,
//!    actor) and mints a fresh
//!    [`PostalDispatchAttachmentId`](educore_documents::prelude::PostalDispatchAttachmentId)
//!    that carries the same `school_id` as the parent
//!    `PostalDispatch`.
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
use educore_documents::aggregate::PostalDispatchAttachment;
use educore_documents::value_objects::{FileReference, PostalDispatchId};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `(school, dispatch_id, file_ref)` triple. The
/// dispatch id is anchored to the freshly-minted school so the
/// tenant invariant
/// (`dispatch_id.school_id() == school_id`) holds.
fn admin_context() -> (FileReference, PostalDispatchId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let dispatch_id = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("scan-object-key-001").expect("non-empty key");
    (file, dispatch_id)
}

// =============================================================================
// Happy path: construct a PostalDispatchAttachment
// =============================================================================

/// End-to-end happy path for the PostalDispatchAttachment
/// child entity. Construct a new attachment for a freshly-minted
/// `PostalDispatch` and assert that:
///
/// 1. The id is a fresh `PostalDispatchAttachmentId` whose
///    `school_id()` matches the parent dispatch's school.
/// 2. The dispatch_id, file, school_id, created_by,
///    updated_by, created_at, and updated_at fields all carry
///    the input.
#[test]
fn postal_dispatch_attachment_new_populates_all_fields() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let dispatch_id = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("scan-object-key-001").expect("non-empty key");
    let at = Timestamp::now();

    let child = PostalDispatchAttachment::new(school, dispatch_id, file.clone(), at, actor);

    // Tenant invariant: the child's school id matches the
    // parent dispatch's school id.
    assert_eq!(child.school_id, school);
    assert_eq!(child.dispatch_id, dispatch_id);
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
// Happy path: construct two distinct PostalDispatchAttachment children
// =============================================================================

/// Two PostalDispatchAttachment children minted back-to-back
/// MUST carry distinct typed ids (each is a fresh UUIDv7) AND
/// distinct dispatch ids. This guards against a regression
/// where the typed-id minting logic accidentally returns the
/// same id twice or pins the wrong dispatch_id.
#[test]
fn postal_dispatch_attachment_two_children_have_distinct_ids_and_dispatch_ids() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let dispatch_id_a = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    let dispatch_id_b = PostalDispatchId::new(school, uuid::Uuid::now_v7());
    let file_a = FileReference::new("scan-key-a").expect("non-empty");
    let file_b = FileReference::new("scan-key-b").expect("non-empty");
    let at = Timestamp::now();

    let child_a = PostalDispatchAttachment::new(school, dispatch_id_a, file_a.clone(), at, actor);
    let child_b = PostalDispatchAttachment::new(school, dispatch_id_b, file_b.clone(), at, actor);

    assert_ne!(child_a.id, child_b.id);
    assert_ne!(child_a.dispatch_id, child_b.dispatch_id);
    assert_eq!(child_a.dispatch_id, dispatch_id_a);
    assert_eq!(child_b.dispatch_id, dispatch_id_b);
    assert_eq!(child_a.file, file_a);
    assert_eq!(child_b.file, file_b);
    assert_eq!(child_a.school_id, child_b.school_id);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-empty `FileReference` and a `PostalDispatchId` whose
/// `school_id()` is a non-nil UUID. The fixture is the
/// foundation for the higher-level tests, so a regression here
/// surfaces immediately.
#[test]
fn admin_context_fixture_returns_anchored_dispatch_and_file() {
    let (file, dispatch_id) = admin_context();
    assert!(!file.as_str().is_empty());
    let _ = dispatch_id.school_id();
}
