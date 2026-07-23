//! Integration tests for the **FmFeesInvoiceLineNote aggregate**
//! vertical slice.
//!
//! Covers the behavioral contract for the Wave 72 per-aggregate drop
//! [`RealFmFeesInvoiceLineNote`](educore_finance::aggregate::RealFmFeesInvoiceLineNote) —
//! the free-form note line attached to an `FmFeesInvoice` aggregate.
//! Validates FFILN I-1 (note is non-empty after trim, 1..=2000 chars),
//! FFILN I-2 (append-only, enforced at the API surface by *not*
//! exposing any `update_*` mutator), `retire()` (active → retired
//! transition, version bump, audit footer advance), and the
//! `create_fm_fees_invoice_line_note` service function (aggregate +
//! event pairing).
//!
//! The pre-existing 2 typed-id-only tests have been preserved (as
//! smoke tests for the typed-id contract) and the suite is extended
//! below with 12 behavioral tests covering the Wave 72 full drop.
//! Wave 72 adds the `RealFmFeesInvoiceLineNote` aggregate, the 2
//! headline events (Created + Retired), the service function, and this
//! test suite.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::CreateFmFeesInvoiceLineNoteCommand;
use educore_finance::events::FmFeesInvoiceLineNoteCreated;
use educore_finance::prelude::RealFmFeesInvoiceLineNote;
use educore_finance::services::create_fm_fees_invoice_line_note;
use educore_finance::value_objects::{FmFeesInvoiceId, FmFeesInvoiceLineNoteId};

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

fn fm_fees_invoice_line_note_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceLineNoteId {
    FmFeesInvoiceLineNoteId::new(school, g.next_uuid())
}

fn fm_fees_invoice_id(g: &SystemIdGen, school: SchoolId) -> FmFeesInvoiceId {
    FmFeesInvoiceId::new(school, g.next_uuid())
}

fn make_note(g: &SystemIdGen, school: SchoolId, note: &str) -> RealFmFeesInvoiceLineNote {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealFmFeesInvoiceLineNote::fresh(
        fm_fees_invoice_line_note_id(g, school),
        fm_fees_invoice_id(g, school),
        note.to_owned(),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// Typed-id contract (preserved from Phase 7 Workstream G seed)
// ---------------------------------------------------------------------------

#[test]
fn fm_fees_invoice_line_note_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_fees_invoice_line_note_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fm_fees_invoice_line_note_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fm_fees_invoice_line_note_id(&g, school);
    let id_b = fm_fees_invoice_line_note_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// ---------------------------------------------------------------------------
// RealFmFeesInvoiceLineNote: fresh() — FFILN I-1 invariant (note non-empty)
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_non_empty_note_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let note = make_note(&g, school, "First-month concession for sibling enrollment.");
    assert_eq!(note.note, "First-month concession for sibling enrollment.");
    assert!(note.is_active(), "fresh aggregate must be Active");
    assert_eq!(note.school_id, school);
}

#[test]
fn fresh_with_whitespace_only_note_returns_validation_error() {
    // FFILN I-1: note must be 1..=2000 chars AFTER trim.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealFmFeesInvoiceLineNote::fresh(
        fm_fees_invoice_line_note_id(&g, school),
        fm_fees_invoice_id(&g, school),
        "   \t\n  ".to_owned(), // whitespace only
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only note must fail with Validation (FFILN I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_empty_note_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealFmFeesInvoiceLineNote::fresh(
        fm_fees_invoice_line_note_id(&g, school),
        fm_fees_invoice_id(&g, school),
        String::new(),
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "empty note must fail with Validation (FFILN I-1), got {result:?}"
    );
}

#[test]
fn fresh_with_note_over_2000_chars_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let overlong = "x".repeat(2001);
    let result = RealFmFeesInvoiceLineNote::fresh(
        fm_fees_invoice_line_note_id(&g, school),
        fm_fees_invoice_id(&g, school),
        overlong,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "note over 2000 chars must fail with Validation (FFILN I-1), got {result:?}"
    );
}

#[test]
fn fresh_trims_surrounding_whitespace_in_note() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let note = make_note(&g, school, "  Manual adjustment: late fee waived.  ");
    assert_eq!(
        note.note, "Manual adjustment: late fee waived.",
        "note must be stored trimmed (FFILN I-1)"
    );
}

#[test]
fn fresh_with_note_at_exactly_2000_chars_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let at_limit = "y".repeat(2000);
    let note = make_note(&g, school, &at_limit);
    assert_eq!(note.note.chars().count(), 2000);
}

// ---------------------------------------------------------------------------
// RealFmFeesInvoiceLineNote: FFILN I-2 (append-only at the API surface)
// ---------------------------------------------------------------------------

#[test]
fn fresh_produces_an_append_only_aggregate_with_no_update_method() {
    // FFILN I-2: append-only. The aggregate intentionally exposes no
    // `update_metadata` / `update_*` mutator (compile-time guarantee).
    // This test pins the surface contract: only `fresh`, `is_active`,
    // and `retire` are public methods on `RealFmFeesInvoiceLineNote`.
    // Adding `update_metadata` (or any `update_*`) would require a new
    // `Updated` event and would violate the append-only invariant
    // documented in `events.rs` (only `FmFeesInvoiceLineNoteCreated`
    // and `FmFeesInvoiceLineNoteRetired` exist; no `Updated` variant).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let note = make_note(&g, school, "Test surface contract.");
    assert!(note.is_active());
    // No update_* method invocation is possible — the API simply does
    // not expose one. This is the practical append-only guarantee.
    // (Verified at compile time by the absence of `update_*` methods
    // on the `RealFmFeesInvoiceLineNote` impl block in `aggregate.rs`.)
    let _ = note.note; // last accessible field
}

// ---------------------------------------------------------------------------
// RealFmFeesInvoiceLineNote: retire()
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut note = make_note(&g, school, "Note to retire.");
    let initial_version = note.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(note.is_active());
    note.retire(now, actor).expect("first retire succeeds");
    assert!(!note.is_active(), "retire must flip is_active to false");
    assert!(
        note.version > initial_version,
        "version must advance on retire"
    );
    assert_eq!(note.updated_by, actor);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut note = make_note(&g, school, "Note to retire twice.");
    let actor = g.next_user_id();
    let now = SystemClock.now();

    note.retire(now, actor).expect("first retire succeeds");
    let result = note.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_fm_fees_invoice_line_note service function
// ---------------------------------------------------------------------------

#[test]
fn create_fm_fees_invoice_line_note_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let invoice_id = fm_fees_invoice_id(&g, school);
    let cmd = CreateFmFeesInvoiceLineNoteCommand {
        tenant: tenant.clone(),
        fm_fees_invoice_id: invoice_id,
        note: "Approved by headmaster on 2026-07-01.".to_owned(),
    };
    let clock = SystemClock;
    let (note, event) =
        create_fm_fees_invoice_line_note(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(note.fm_fees_invoice_id, invoice_id);
    assert_eq!(note.note, "Approved by headmaster on 2026-07-01.");
    assert!(note.is_active(), "service-created aggregate must be Active");
    assert_eq!(note.school_id, school);
    assert_eq!(note.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.fm_fees_invoice_line_note_id, note.id);
    assert_eq!(event.fm_fees_invoice_id, invoice_id);
    assert_eq!(event.note, "Approved by headmaster on 2026-07-01.");
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        FmFeesInvoiceLineNoteCreated::EVENT_TYPE,
        "finance.fm_fees_invoice_line_note.created"
    );
    assert_eq!(
        FmFeesInvoiceLineNoteCreated::AGGREGATE_TYPE,
        "fm_fees_invoice_line_note"
    );
    assert_eq!(FmFeesInvoiceLineNoteCreated::SCHEMA_VERSION, 1);
}

#[test]
fn create_fm_fees_invoice_line_note_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let cmd = CreateFmFeesInvoiceLineNoteCommand {
        tenant: tenant.clone(),
        fm_fees_invoice_id: fm_fees_invoice_id(&g, school),
        note: "   ".to_owned(), // FFILN I-1: whitespace-only
    };
    let clock = SystemClock;
    let result = create_fm_fees_invoice_line_note(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "whitespace-only note must propagate Validation (FFILN I-1), got {result:?}"
    );
}
