//! Integration tests for the **IncidentComment aggregate** vertical slice.
//!
//! Pins the constructor + soft-delete contract for
//! [`IncidentComment`](educore_events_domain::aggregate::IncidentComment)
//! end-to-end through the aggregate layer:
//!
//! 1. `IncidentComment::new(...)` validates the input
//!    (non-empty `comment`), constructs the aggregate with
//!    `version = Version::initial()` and `active_status = true`,
//!    and propagates the typed id's `school_id` to the
//!    denormalised field.
//! 2. `IncidentComment::delete(...)` soft-deletes the comment
//!    (admin override): sets `active_status = false`, bumps
//!    `version`, and moves `updated_at`.
//!
//! The tests follow the **constructor + update pattern** of this
//! domain: no factory handlers, no events emitted from the
//! constructor. The handlers / outbox / audit fan-out are not yet
//! wired end-to-end; these tests pin the **aggregate layer**
//! contract that the service factory fns and dispatcher will
//! eventually wrap.
//!
//! Implements: `docs/specs/events/aggregates.md` ## IncidentComment
//! and `docs/specs/events/workflows.md` ## "IncidentComment
//! Configuration Workflow".

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{SchoolId, UserId};
use educore_core::value_objects::{Timestamp, Version};
use educore_events_domain::aggregate::IncidentComment;
use educore_events_domain::errors::EventsDomainError;
use educore_events_domain::prelude::*;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh school id + system actor id from a single
/// `SystemIdGen`. Mirrors the fixture style used in
/// `tests/workflows.rs` and `tests/holiday.rs`.
fn fixture_ids() -> (SchoolId, UserId, Timestamp) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let at = TestClock::new().now();
    (school, actor, at)
}

// =============================================================================
// 1. Happy path: IncidentComment::new + delete
// =============================================================================

/// End-to-end happy path for the IncidentComment aggregate.
/// Build a comment on an existing incident authored by a
/// system user, then soft-delete the comment via `delete`,
/// asserting that:
///
/// 1. `IncidentComment::new` returns an aggregate whose
///    `school_id`, `incident_id`, `user_id`, `comment`,
///    `version`, and `active_status` all match the command
///    inputs.
/// 2. `IncidentComment::delete` mutates the aggregate
///    in-place, flipping `active_status` to `false`, bumping
///    `version` exactly once, and moving `updated_at`.
#[test]
fn incident_comment_create_then_delete_soft_deletes_aggregate_and_bumps_version() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = IncidentCommentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());

    // ---- Create flow ----
    let mut comment = IncidentComment::new(
        id,
        incident_id,
        actor,
        "Escalated to principal office.".to_owned(),
        ts,
    )
    .expect("IncidentComment::new must succeed for non-empty input");

    // Identity: typed id and denormalised school_id agree.
    assert_eq!(comment.id, id);
    assert_eq!(comment.id.school_id(), school);
    assert_eq!(comment.school_id, school);

    // Payload: every field on the command is present on the aggregate.
    assert_eq!(comment.incident_id, incident_id);
    assert_eq!(comment.user_id, actor);
    assert_eq!(comment.comment, "Escalated to principal office.");

    // Audit metadata footer is initialised per the constructor
    // contract: version starts at 1, active_status is true, and
    // created_by/updated_by both equal the supplied actor.
    assert_eq!(comment.version, Version::initial());
    assert!(comment.active_status);
    assert_eq!(comment.created_by, actor);
    assert_eq!(comment.updated_by, actor);
    assert_eq!(comment.created_at, ts);
    assert_eq!(comment.updated_at, ts);

    // ---- Delete (soft) flow ----
    let version_before = comment.version;
    let deleted_at = Timestamp::now();
    comment.delete(deleted_at);

    // The aggregate is mutated in place: active_status flipped,
    // version bumped exactly once, updated_at moved.
    assert!(!comment.active_status, "delete must flip active_status");
    assert_eq!(
        comment.version,
        version_before.next(),
        "version must be bumped exactly once"
    );
    assert_eq!(comment.updated_at, deleted_at);
    // Identity + payload fields that delete must NOT touch.
    assert_eq!(comment.id, id);
    assert_eq!(comment.school_id, school);
    assert_eq!(comment.incident_id, incident_id);
    assert_eq!(comment.user_id, actor);
    assert_eq!(comment.comment, "Escalated to principal office.");
    // created_by / created_at must remain anchored to the
    // original authoring event.
    assert_eq!(comment.created_by, actor);
    assert_eq!(comment.created_at, ts);
}

// =============================================================================
// 2. Validation failure: empty comment is rejected
// =============================================================================

/// Validation-failure path on the create flow: per spec
/// invariant, `comment` must be non-empty (after trim).
/// `IncidentComment::new` must reject an empty string with
/// [`EventsDomainError::Validation`].
#[test]
fn incident_comment_create_with_empty_comment_returns_validation_error() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = IncidentCommentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());

    let res = IncidentComment::new(id, incident_id, actor, String::new(), ts);
    let err = res.expect_err("empty comment must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "expected EventsDomainError::Validation, got {err:?}"
    );
}

// =============================================================================
// 3. Validation failure: whitespace-only comment is rejected
// =============================================================================

/// Validation-failure path on the create flow: per spec
/// invariant, `comment` must be non-empty after trimming.
/// `IncidentComment::new` must reject a whitespace-only
/// string with [`EventsDomainError::Validation`].
#[test]
fn incident_comment_create_with_whitespace_only_comment_returns_validation_error() {
    let (school, actor, ts) = fixture_ids();
    let g = SystemIdGen;
    let id = IncidentCommentId::new(school, g.next_uuid());
    let incident_id = IncidentId::new(school, g.next_uuid());

    let res = IncidentComment::new(id, incident_id, actor, "   \t  \n".to_owned(), ts);
    let err = res.expect_err("whitespace-only comment must be rejected");
    assert!(
        matches!(err, EventsDomainError::Validation(_)),
        "expected EventsDomainError::Validation, got {err:?}"
    );
}
