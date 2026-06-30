//! Integration tests for the **LibraryMemberNote aggregate**
//! vertical slice.
//!
//! Pins the create + audit-footer contract for
//! [`LibraryMemberNote`](educore_library::prelude::LibraryMemberNote):
//!
//! 1. `LibraryMemberNote::new` constructs the child-entity
//!    projection of the aggregate root with `school_id`
//!    derived from the member id, carries the sequence /
 //!    author / body / visible_to_member fields from the
//!    call, and stamps `created_at` from the caller.
//! 2. The emitted
//!    [`LibraryMemberNoteAdded`](educore_library::events::LibraryMemberNoteAdded)
//!    event stub carries the right `event_type` /
//!    `aggregate_type` / `school_id`, with `aggregate_id`
//!    matching the typed id.
//!
//! The cluster-C service handler
//! (`add_library_member_note`) is not yet implemented
//! (TODO stub); these tests pin the **entity-level**
//! contract that the handler will populate once it is
//! wired. The aggregate root in `aggregate.rs` (also named
//! `LibraryMemberNote`) is the first-class versioned root
//! and shares its field shape with this entity projection.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_library::events::LibraryMemberNoteAdded;
use educore_library::prelude::*;
use educore_library::value_objects::LibraryMemberNoteId;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
fn admin_context() -> (TenantContext, UserType, educore_core::ids::SchoolId, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        UserType::SchoolAdmin,
        school,
        g,
    )
}

/// Mint a `LibraryMemberId` for the given school.
fn member_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> LibraryMemberId {
    LibraryMemberId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: create a LibraryMemberNote (visible to member)
// =============================================================================

/// End-to-end happy path for the LibraryMemberNote entity.
/// Add a fresh note and assert that:
///
/// 1. The entity carries the sequence / author / body /
///    visible_to_member fields from the constructor call,
///    with `school_id` derived from the member id.
/// 2. The emitted `LibraryMemberNoteAdded` event carries
///    the right `event_type` / `aggregate_type` / `school_id`
///    and `aggregate_id` matching the typed id.
#[test]
fn library_member_note_new_populates_entity_and_event() {
    let (_tenant, _actor_type, school, g) = admin_context();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let member = member_id(&g, school);
    let now = clock.now();
    let author = g.next_user_id();
    let correlation_id = g.next_correlation_id();

    // ---- Construct the entity ----
    let note = LibraryMemberNote::new(
        member,
        1,
        author,
        "Reminder: please return overdue books by Friday.".to_owned(),
        true,
        now,
    );

    // Entity fields are populated from the constructor.
    assert_eq!(note.school_id, school);
    assert_eq!(note.library_member_id, member);
    assert_eq!(note.sequence, 1);
    assert_eq!(note.author, author);
    assert_eq!(note.body, "Reminder: please return overdue books by Friday.");
    assert!(note.visible_to_member);
    assert_eq!(note.created_at, now);

    // ---- Emit the event ----
    let typed_id = LibraryMemberNoteId::new(school, g.next_uuid());
    let event_id = ids.next_event_id();
    let event = LibraryMemberNoteAdded::new(typed_id, event_id, correlation_id, now);
    assert_eq!(
        <LibraryMemberNoteAdded as DomainEvent>::EVENT_TYPE,
        "library.member_note.added"
    );
    assert_eq!(
        <LibraryMemberNoteAdded as DomainEvent>::AGGREGATE_TYPE,
        "library_member_note"
    );
    assert_eq!(<LibraryMemberNoteAdded as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), typed_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.id, typed_id);
}

// =============================================================================
// Happy path: sequence monotonicity and visibility flag
// =============================================================================

/// End-to-end happy path for the LibraryMemberNote
/// sequence monotonicity + visibility flag. Construct two
/// notes for the same member with monotonically-increasing
/// `sequence` numbers and different `visible_to_member`
/// flags, asserting that:
///
/// 1. Each note carries its own independent
///    sequence / body / visible_to_member fields.
/// 2. The sequence numbers are strictly monotonically
///    increasing — the invariant the versioned-log
///    projection relies on.
/// 3. The visibility flag is captured per-note: a
///    staff-only note has `visible_to_member = false`,
///    a member-visible note has `visible_to_member = true`.
#[test]
fn library_member_note_sequence_monotonicity_and_visibility_flag() {
    let (_tenant, _actor_type, school, g) = admin_context();
    let clock = TestClock::new();

    let member = member_id(&g, school);
    let now = clock.now();
    let author = g.next_user_id();

    let first = LibraryMemberNote::new(
        member,
        1,
        author,
        "Account flagged: 3 overdue books in last month.".to_owned(),
        false,
        now,
    );

    let second = LibraryMemberNote::new(
        member,
        2,
        author,
        "Account restored after the overdue books were returned.".to_owned(),
        true,
        now,
    );

    // Strictly monotonically increasing sequence.
    assert!(second.sequence > first.sequence);
    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);

    // Visibility flag captured per-note.
    assert!(!first.visible_to_member);
    assert!(second.visible_to_member);

    // Same-member / same-school notes both belong to the
    // same school as the member id.
    assert_eq!(first.school_id, school);
    assert_eq!(second.school_id, school);
    assert_eq!(first.library_member_id, member);
    assert_eq!(second.library_member_id, member);
}
