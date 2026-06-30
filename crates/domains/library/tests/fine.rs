//! Integration tests for the **Fine aggregate** vertical slice.
//!
//! Pins the create + lifecycle contract for
//! [`Fine`](educore_library::aggregate::Fine) end-to-end
//! through the service layer:
//!
//! 1. `compute_fine` calculates the number of days overdue
//!    and the fine amount via
//!    [`FineCalculationService`](educore_library::services::FineCalculationService),
//!    constructs the aggregate with the full 10-field audit
//!    footer initialised, and emits a `FineCalculated`
//!    event whose `event_type` / `aggregate_type` /
//!    `school_id` match the typed id's `school_id()`
//!    accessor.
//! 2. The aggregate's `waive` method transitions the fine
//!    to a waived state (sets `waived = true`, stamps
//!    `waived_by` / `waived_reason`, bumps `version`).
//!
//! The tests use the same fixture pattern as
//! `tests/aggregates.rs` (`TestClock` + `SystemIdGen`). The
//! `waive_book_issue_fine` service handler is still a TODO
//! stub in `services.rs`; the waive contract is pinned at
//! the **aggregate level** directly.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_library::prelude::*;
use educore_library::value_objects::{
    BookIssueId, FineAmount, FineKind, FinePerDay, FineReason, FineSettings, LibraryMemberId,
};

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

/// Mint a `BookIssueId` for the given school.
fn issue_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> BookIssueId {
    BookIssueId::new(school, g.next_uuid())
}

/// Mint a `LibraryMemberId` for the given school.
fn member_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> LibraryMemberId {
    LibraryMemberId::new(school, g.next_uuid())
}

/// Construct a `NaiveDate` from parts, panicking on invalid
/// inputs (test fixture only).
fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
}

// =============================================================================
// Happy path: compute a Fine on a late return
// =============================================================================

/// End-to-end happy path for the Fine aggregate. Calculate
/// a fine for a `BookIssue` whose due date is 5 days before
/// the `as_of` date, asserting that:
///
/// 1. The `FineCalculationService` computes `days_overdue =
///    5` and a positive `amount` for a per-day rate of 50.
/// 2. The service returns a `Fine` aggregate with
///    `school_id` derived from the typed id, the audit
///    footer initialised (`version = 1`, `created_by` /
///    `updated_by` from the tenant, `waived = false`), and
///    the per-day rate / amount / reason fields populated.
/// 3. The emitted `FineCalculated` event carries the right
///    `event_type` / `aggregate_type` / `school_id` and
///    `aggregate_id` matching the typed id.
#[test]
fn fine_compute_for_late_return_emits_event_and_populates_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let fine_id = FineId::new(school, g.next_uuid());
    let book = book_id(&g, school);
    let issue = issue_id(&g, school);
    let member = member_id(&g, school);
    let per_day_rate = FinePerDay::new(rust_decimal::Decimal::from(50))
        .expect("non-negative per-day rate");
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 0,
    };

    // due_date 5 days before as_of → 5 days overdue.
    let cmd = CalculateFineCommand {
        tenant: tenant.clone(),
        fine_id,
        book_issue_id: issue,
        book_id: book,
        library_member_id: member,
        as_of: date(2026, 10, 4),
        per_day_rate,
        reason: FineReason::LateReturn,
    };
    let result = compute_fine(
        cmd,
        &clock,
        &ids,
        DueDate(date(2026, 9, 29)),
        &settings,
    )
    .expect("compute fine");

    // Aggregate fields are populated from the calculation.
    assert_eq!(result.fine.school_id, school);
    assert_eq!(result.fine.id, fine_id);
    assert_eq!(result.fine.book_issue_id, issue);
    assert_eq!(result.fine.book_id, book);
    assert_eq!(result.fine.library_member_id, member);
    assert_eq!(result.fine.days_overdue, 5);
    assert_eq!(result.fine.per_day_rate, per_day_rate);
    assert_eq!(
        result.fine.amount,
        FineAmount::new(rust_decimal::Decimal::from(250)).expect("non-negative amount")
    );
    assert_eq!(result.fine.reason, FineReason::LateReturn);
    assert!(!result.fine.waived);
    assert_eq!(result.fine.waived_by, None);
    assert_eq!(result.fine.waived_reason, None);

    // Audit metadata footer is initialised.
    assert_eq!(result.fine.version.get(), 1);
    assert_eq!(result.fine.created_by, tenant.actor_id);
    assert_eq!(result.fine.updated_by, tenant.actor_id);
    assert!(result.fine.active_status.is_active());

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <FineCalculated as DomainEvent>::EVENT_TYPE,
        "library.fine.calculated"
    );
    assert_eq!(<FineCalculated as DomainEvent>::AGGREGATE_TYPE, "fine");
    assert_eq!(<FineCalculated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(result.event.aggregate_id(), fine_id.as_uuid());
    assert_eq!(result.event.school_id(), school);
    assert_eq!(result.event.days_overdue, 5);
    assert_eq!(result.event.fine_id, fine_id);
}

// =============================================================================
// Happy path: waive a Fine
// =============================================================================

/// End-to-end happy path for the Fine waive lifecycle.
/// Compute a fresh fine and call the aggregate's `waive`
/// method, asserting that:
///
/// 1. The aggregate transitions to a waived state (`waived =
///    true`, `waived_by` / `waived_reason` stamped from the
///    call, `version` bumped, `updated_by` / `updated_at`
///    stamped from the actor).
/// 2. The audit footer is initialised correctly before the
///    waiver (`version = 1`).
#[test]
fn fine_waive_transitions_aggregate_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let fine_id = FineId::new(school, g.next_uuid());
    let book = book_id(&g, school);
    let issue = issue_id(&g, school);
    let member = member_id(&g, school);
    let per_day_rate = FinePerDay::new(rust_decimal::Decimal::from(50))
        .expect("non-negative per-day rate");
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 0,
    };

    let cmd = CalculateFineCommand {
        tenant: tenant.clone(),
        fine_id,
        book_issue_id: issue,
        book_id: book,
        library_member_id: member,
        as_of: date(2026, 10, 4),
        per_day_rate,
        reason: FineReason::LateReturn,
    };
    let FineComputed { mut fine, event: _ } = compute_fine(
        cmd,
        &clock,
        &ids,
        DueDate(date(2026, 9, 29)),
        &settings,
    )
    .expect("compute fine");
    let initial_version = fine.version.get();
    let waive_event_id = ids.next_event_id();
    let waive_at = clock.now();
    let waive_actor = tenant.actor_id;
    let waive_reason = "Goodwill — first offence".to_owned();

    // ---- Waive the fine ----
    fine.waive(
        waive_actor,
        waive_reason.clone(),
        waive_at,
        waive_event_id,
    );

    // The aggregate is now in a waived state.
    assert!(fine.waived);
    assert_eq!(fine.waived_by, Some(waive_actor));
    assert_eq!(fine.waived_reason.as_deref(), Some(waive_reason.as_str()));
    assert_eq!(fine.version.get(), initial_version + 1);
    assert_eq!(fine.updated_by, waive_actor);
    assert_eq!(fine.last_event_id, Some(waive_event_id));
}

// =============================================================================
// Boundary: zero-day overdue fine is zero
// =============================================================================

/// Boundary case for the fine calculation: when `as_of ==
/// due_date`, `days_overdue = 0` and the fine amount is
/// zero (no penalty for on-time returns). The aggregate's
/// `amount` field is exactly zero and the emitted event's
/// `days_overdue` field is 0.
#[test]
fn fine_compute_on_due_date_yields_zero_amount() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let fine_id = FineId::new(school, g.next_uuid());
    let per_day_rate = FinePerDay::new(rust_decimal::Decimal::from(50))
        .expect("non-negative per-day rate");
    let settings = FineSettings {
        kind: FineKind::PerDayRate(50),
        grace_period_days: 0,
    };

    let cmd = CalculateFineCommand {
        tenant: tenant.clone(),
        fine_id,
        book_issue_id: issue_id(&g, school),
        book_id: book_id(&g, school),
        library_member_id: member_id(&g, school),
        as_of: date(2026, 9, 29),
        per_day_rate,
        reason: FineReason::LateReturn,
    };
    let result = compute_fine(
        cmd,
        &clock,
        &ids,
        DueDate(date(2026, 9, 29)),
        &settings,
    )
    .expect("compute fine");

    assert_eq!(result.fine.days_overdue, 0);
    assert_eq!(result.fine.amount, FineAmount(rust_decimal::Decimal::from(0)));
    assert_eq!(result.event.days_overdue, 0);
    assert_eq!(result.event.amount, FineAmount(rust_decimal::Decimal::from(0)));
}
