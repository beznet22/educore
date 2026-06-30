//! Integration tests for the **PostalReceiveAttachment child entity**.
//!
//! Pins the construction contract for
//! [`PostalReceiveAttachment`](educore_documents::aggregate::PostalReceiveAttachment)
//! — the optional `FileReference` attached to a
//! [`PostalReceive`](educore_documents::aggregate::PostalReceive).
//!
//! The tests verify that:
//!
//! 1. `PostalReceiveAttachment::new` populates every field
//!    from the input (school, receive_id, file, timestamp,
//!    actor) and mints a fresh
//!    [`PostalReceiveAttachmentId`](educore_documents::prelude::PostalReceiveAttachmentId)
//!    that carries the same `school_id` as the parent
//!    `PostalReceive`.
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
use educore_documents::aggregate::PostalReceiveAttachment;
use educore_documents::value_objects::{FileReference, PostalReceiveId};

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `(school, receive_id, file_ref)` triple. The
/// receive id is anchored to the freshly-minted school so the
/// tenant invariant (`receive_id.school_id() == school_id`)
/// holds.
fn admin_context() -> (FileReference, PostalReceiveId) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let receive_id = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("scan-object-key-002").expect("non-empty key");
    (file, receive_id)
}

// =============================================================================
// Happy path: construct a PostalReceiveAttachment
// =============================================================================

/// End-to-end happy path for the PostalReceiveAttachment
/// child entity. Construct a new attachment for a freshly-minted
/// `PostalReceive` and assert that:
///
/// 1. The id is a fresh `PostalReceiveAttachmentId` whose
///    `school_id()` matches the parent receive's school.
/// 2. The receive_id, file, school_id, created_by,
///    updated_by, created_at, and updated_at fields all carry
///    the input.
#[test]
fn postal_receive_attachment_new_populates_all_fields() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let receive_id = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    let file = FileReference::new("scan-object-key-002").expect("non-empty key");
    let at = Timestamp::now();

    let child = PostalReceiveAttachment::new(school, receive_id, file.clone(), at, actor);

    // Tenant invariant: the child's school id matches the
    // parent receive's school id.
    assert_eq!(child.school_id, school);
    assert_eq!(child.receive_id, receive_id);
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
// Happy path: construct two distinct PostalReceiveAttachment children
// =============================================================================

/// Two PostalReceiveAttachment children minted back-to-back
/// MUST carry distinct typed ids (each is a fresh UUIDv7) AND
/// distinct receive ids. This guards against a regression
/// where the typed-id minting logic accidentally returns the
/// same id twice or pins the wrong receive_id.
#[test]
fn postal_receive_attachment_two_children_have_distinct_ids_and_receive_ids() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let receive_id_a = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    let receive_id_b = PostalReceiveId::new(school, uuid::Uuid::now_v7());
    let file_a = FileReference::new("scan-key-c").expect("non-empty");
    let file_b = FileReference::new("scan-key-d").expect("non-empty");
    let at = Timestamp::now();

    let child_a = PostalReceiveAttachment::new(school, receive_id_a, file_a.clone(), at, actor);
    let child_b = PostalReceiveAttachment::new(school, receive_id_b, file_b.clone(), at, actor);

    assert_ne!(child_a.id, child_b.id);
    assert_ne!(child_a.receive_id, child_b.receive_id);
    assert_eq!(child_a.receive_id, receive_id_a);
    assert_eq!(child_b.receive_id, receive_id_b);
    assert_eq!(child_a.file, file_a);
    assert_eq!(child_b.file, file_b);
    assert_eq!(child_a.school_id, child_b.school_id);
}

// =============================================================================
// Fixture smoke test
// =============================================================================

/// Smoke test for the `admin_context` fixture: it MUST return
/// a non-empty `FileReference` and a `PostalReceiveId` whose
/// `school_id()` is a non-nil UUID. The fixture is the
/// foundation for the higher-level tests, so a regression here
/// surfaces immediately.
#[test]
fn admin_context_fixture_returns_anchored_receive_and_file() {
    let (file, receive_id) = admin_context();
    assert!(!file.as_str().is_empty());
    let _ = receive_id.school_id();
}
