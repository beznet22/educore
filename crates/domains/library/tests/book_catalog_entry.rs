//! Integration tests for the **BookCatalogEntry aggregate**
//! vertical slice.
//!
//! Pins the create + audit-footer contract for
//! [`BookCatalogEntry`](educore_library::prelude::BookCatalogEntry):
//!
//! 1. `BookCatalogEntry::new` constructs the child-entity
//!    projection of the aggregate root with `school_id`
//!    derived from the book id, carries the sequence /
//!    isbn / book-number / title / author fields from the
//!    call, and stamps `occurred_at` / `actor_id` /
//!    `correlation_id` for the append-only log.
//! 2. The emitted
//!    [`BookCatalogEntryAppended`](educore_library::events::BookCatalogEntryAppended)
//!    event stub carries the right `event_type` /
//!    `aggregate_type` / `school_id`, with `aggregate_id`
//!    matching the typed id.
//!
//! The cluster-C service handler
//! (`append_book_catalog_entry`) is not yet implemented
//! (TODO stub); these tests pin the **entity-level**
//! contract that the handler will populate once it is
//! wired. The aggregate root in `aggregate.rs` (also named
//! `BookCatalogEntry`) is the first-class versioned root
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
use educore_library::events::BookCatalogEntryAppended;
use educore_library::prelude::*;
use educore_library::value_objects::BookCatalogEntryId;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. Returns the context plus the
/// generator so tests can mint child ids from the same
/// school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

/// Mint a `BookId` for the given school.
fn book_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> BookId {
    BookId::new(school, g.next_uuid())
}

// =============================================================================
// Happy path: append a BookCatalogEntry
// =============================================================================

/// End-to-end happy path for the BookCatalogEntry entity.
/// Append a fresh cataloging entry and assert that:
///
/// 1. The entity carries the sequence / isbn / book-number /
///    title / author fields from the constructor call, with
///    `school_id` derived from the book id.
/// 2. The emitted `BookCatalogEntryAppended` event carries
///    the right `event_type` / `aggregate_type` / `school_id`
///    and `aggregate_id` matching the typed id.
#[test]
fn book_catalog_entry_new_populates_entity_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Construct the entity ----
    let book = book_id(&g, school);
    let isbn = Isbn::parse("9780134685991").expect("valid ISBN-13");
    let book_number = BookNumber::new("BK-0001").expect("non-empty number");
    let title = BookTitle::new("The Rust Programming Language").expect("non-empty title");
    let author = Some(Author::new("Steve Klabnik").expect("non-empty author"));
    let now = clock.now();

    let entry = BookCatalogEntry::new(
        book,
        1,
        Some(isbn.clone()),
        Some(book_number.clone()),
        title.clone(),
        author.clone(),
        now,
        tenant.actor_id,
        tenant.correlation_id,
    );

    // Entity fields are populated from the constructor.
    assert_eq!(entry.school_id, school);
    assert_eq!(entry.book_id, book);
    assert_eq!(entry.sequence, 1);
    assert_eq!(entry.isbn_no.as_ref(), Some(&isbn));
    assert_eq!(entry.book_number.as_ref(), Some(&book_number));
    assert_eq!(entry.book_title, title);
    assert_eq!(entry.author_name, author);
    assert_eq!(entry.occurred_at, now);
    assert_eq!(entry.actor_id, tenant.actor_id);
    assert_eq!(entry.correlation_id, tenant.correlation_id);

    // ---- Emit the event ----
    let typed_id = BookCatalogEntryId::new(school, g.next_uuid());
    let event_id = ids.next_event_id();
    let event = BookCatalogEntryAppended::new(typed_id, event_id, tenant.correlation_id, now);
    assert_eq!(
        <BookCatalogEntryAppended as DomainEvent>::EVENT_TYPE,
        "library.book_catalog_entry.appended"
    );
    assert_eq!(
        <BookCatalogEntryAppended as DomainEvent>::AGGREGATE_TYPE,
        "book_catalog_entry"
    );
    assert_eq!(<BookCatalogEntryAppended as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), typed_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.id, typed_id);
}

// =============================================================================
// Happy path: versioned append log monotonicity
// =============================================================================

/// End-to-end happy path for the BookCatalogEntry
/// append-log monotonicity. Construct two cataloging
/// entries for the same book with monotonically-increasing
/// `sequence` numbers, asserting that:
///
/// 1. Each entry carries its own independent
///    sequence / title / author fields (no shared state).
/// 2. The sequence numbers are strictly monotonically
///    increasing — the invariant the version-history
///    projection relies on.
/// 3. Each entry derives its `school_id` from the book id
///    so cross-tenant log entries can never accidentally
///    interleave.
#[test]
fn book_catalog_entry_sequence_monotonicity_and_school_isolation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();

    let book = book_id(&g, school);
    let now = clock.now();

    let first = BookCatalogEntry::new(
        book,
        1,
        Some(Isbn::parse("9780134685991").expect("valid ISBN-13")),
        Some(BookNumber::new("BK-0001").expect("non-empty number")),
        BookTitle::new("The Rust Programming Language").expect("non-empty title"),
        Some(Author::new("Steve Klabnik").expect("non-empty author")),
        now,
        tenant.actor_id,
        tenant.correlation_id,
    );

    let second = BookCatalogEntry::new(
        book,
        2,
        Some(Isbn::parse("9780134685991").expect("valid ISBN-13")),
        Some(BookNumber::new("BK-0001").expect("non-empty number")),
        BookTitle::new("The Rust Programming Language (2nd ed.)").expect("non-empty title"),
        Some(Author::new("Steve Klabnik").expect("non-empty author")),
        now,
        tenant.actor_id,
        tenant.correlation_id,
    );

    // Strictly monotonically increasing sequence.
    assert!(second.sequence > first.sequence);
    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);

    // Title changes between entries are captured per-entry
    // (history is the full log).
    assert_ne!(first.book_title, second.book_title);

    // Same-book / same-school entries both belong to the
    // same school as the book id.
    assert_eq!(first.school_id, school);
    assert_eq!(second.school_id, school);
    assert_eq!(first.book_id, book);
    assert_eq!(second.book_id, book);
}
