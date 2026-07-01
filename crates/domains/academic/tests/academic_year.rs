//! Integration tests for the **AcademicYear aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`AcademicYear`](educore_academic::AcademicYear)
//! end-to-end through the service layer:
//!
//! 1. `create_academic_year` validates the label + title +
//!    date range, constructs the aggregate (defaults:
//!    `is_current = false`, `is_closed = false`, `version = 1`,
//!    `active_status = Active`), and emits an
//!    [`AcademicYearCreated`] event carrying the label,
//!    title, start date, end date, and `is_current` flag.
//! 2. `update_academic_year_dates` mutates the in-place
//!    aggregate's `range`, bumps `version`, and emits an
//!    [`AcademicYearDatesUpdated`] event carrying the new
//!    start + end dates.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs`,
//! `crates/domains/academic/tests/subject.rs`, and
//! `crates/domains/academic/tests/workflows.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests
//! pin the contract of the **service layer** that the
//! dispatcher will eventually wrap.
//!
//! Note on `AcademicYear` field set: the aggregate carries
//! `year` (short label, e.g. "2026"), `title` (display
//! title), `range` (start + end), `is_current` (exactly
//! one per school may be `true`), and `is_closed`
//! (read-only flag). The `is_current` setter is the
//! dedicated `SetCurrentAcademicYear` command (separate
//! test surface). These tests pin the create + update
//! flows; the `is_current` / `is_closed` flags are
//! observed via the constructor + the `close_academic_year`
//! service (covered in the unit tests in `services.rs`).
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic test suites.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_events::domain_event::DomainEvent;

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

fn year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create + update on AcademicYear
// =============================================================================

/// End-to-end happy path for the `AcademicYear`
/// aggregate. Create an academic year, then update its
/// date range, asserting that:
///
/// 1. The create flow produces an `AcademicYear`
///    aggregate carrying every field on the command
///    (year, title, range) + an `AcademicYearCreated`
///    event with the right `event_type`,
///    `aggregate_type`, and `school_id`.
/// 2. The audit metadata footer is initialised
///    correctly: `version = 1`, `active_status =
///    Active`, `is_current = false`, `is_closed =
///    false`, `created_by = updated_by = actor_id`,
///    and `correlation_id` is propagated from the
///    tenant context.
/// 3. The update flow mutates the aggregate in place
///    (bumps `version`, swaps `range`), and emits an
///    `AcademicYearDatesUpdated` event whose
///    `aggregate_id()` matches the typed id and whose
///    `school_id()` matches the school.
/// 4. The update event id stamped on `last_event_id`
///    matches the event the service returned.
#[test]
fn academic_year_create_then_update_dates_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    let create_cmd = CreateAcademicYearCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: start,
        ending_date: end,
        is_current: true,
        copy_with_academic_year: None,
    };
    let (mut agg, created_event) =
        create_academic_year(create_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.year, "2026");
    assert_eq!(agg.title, "Academic Year 2026-2027");
    assert_eq!(agg.range.start, start);
    assert_eq!(agg.range.end, end);
    assert!(agg.is_current);
    assert!(!agg.is_closed);

    // Audit metadata footer is initialised.
    assert_eq!(agg.version.get(), 1);
    assert!(agg.active_status.is_active());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.correlation_id, tenant.correlation_id);
    // `last_event_id` is stamped by the create flow
    // (the service sets it before returning the event).
    assert_eq!(agg.last_event_id, Some(created_event.event_id));

    // Event metadata matches the DomainEvent trait's
    // contract.
    assert_eq!(
        <AcademicYearCreated as DomainEvent>::EVENT_TYPE,
        "academic.academic_year.created"
    );
    assert_eq!(
        <AcademicYearCreated as DomainEvent>::AGGREGATE_TYPE,
        "academic_year"
    );
    assert_eq!(<AcademicYearCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.year, "2026");
    assert_eq!(created_event.title, "Academic Year 2026-2027");
    assert_eq!(created_event.start_date, start);
    assert_eq!(created_event.end_date, end);
    assert!(created_event.is_current);

    // ---- Update flow ----
    let initial_version = agg.version.get();
    let new_start = chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let new_end = chrono::NaiveDate::from_ymd_opt(2027, 4, 1).unwrap();
    let update_cmd = UpdateAcademicYearDatesCommand {
        tenant: tenant.clone(),
        academic_year_id: agg.id,
        starting_date: new_start,
        ending_date: new_end,
    };
    let updated_event =
        update_academic_year_dates(&mut agg, update_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("update");

    // The aggregate is mutated in place: the range
    // moved and the version bumped.
    assert_eq!(agg.range.start, new_start);
    assert_eq!(agg.range.end, new_end);
    assert_eq!(agg.version.get(), initial_version + 1);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_by, tenant.actor_id);
    // The event id stamped on `last_event_id` matches
    // the update event the service returned.
    assert_eq!(agg.last_event_id, Some(updated_event.event_id));

    // The event metadata matches the DomainEvent
    // trait's contract.
    assert_eq!(
        <AcademicYearDatesUpdated as DomainEvent>::EVENT_TYPE,
        "academic.academic_year.dates_updated"
    );
    assert_eq!(
        <AcademicYearDatesUpdated as DomainEvent>::AGGREGATE_TYPE,
        "academic_year"
    );
    assert_eq!(updated_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(updated_event.from, new_start);
    assert_eq!(updated_event.to, new_end);
}

// =============================================================================
// 2. Validation failure: inverted date range returns DomainError::Validation
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `starting_date` is **after** the `ending_date`,
/// `create_academic_year` returns
/// `DomainError::Validation` (via `AcademicYearRange::new`,
/// which enforces `start < end`). No aggregate or event is
/// constructed.
///
/// Note: this is the second test (not a separate
/// "happy-path update" test of the second variant) so the
/// test suite also pins the **negative** branch the
/// create flow exposes — the update flow's negative
/// branches (e.g. inverted range on
/// `update_academic_year_dates`) reuse the same
/// `AcademicYearRange::new` validator and are covered by
/// the unit tests in `services.rs`.
#[test]
fn academic_year_create_with_inverted_range_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Inverted range: start is AFTER end. Must fail
    // with Validation (no aggregate, no event).
    let start = chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let create_cmd = CreateAcademicYearCommand {
        tenant: tenant.clone(),
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: start,
        ending_date: end,
        is_current: false,
        copy_with_academic_year: None,
    };
    let err = create_academic_year(create_cmd, &clock, &ids, &NoOpUniquenessChecker)
        .expect_err("inverted date range must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // Sanity check: a subsequent call with a valid
    // range (start < end) still succeeds, proving the
    // failure was tied to the inverted range (and not
    // to a corrupt clock, ids, or tenant context).
    let ok_start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let ok_end = chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap();
    let ok_cmd = CreateAcademicYearCommand {
        tenant,
        academic_year_id: year_id(&g, school),
        year: "2026".to_owned(),
        title: "Academic Year 2026-2027".to_owned(),
        starting_date: ok_start,
        ending_date: ok_end,
        is_current: true,
        copy_with_academic_year: None,
    };
    let (_agg, _event) =
        create_academic_year(ok_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("valid range must succeed");
}

// =============================================================================
// No-op UniquenessChecker for tests
// =============================================================================

struct NoOpUniquenessChecker;

impl educore_academic::commands::UniquenessChecker for NoOpUniquenessChecker {
    fn student_admission_no_exists(&self, _school: educore_core::ids::SchoolId, _admission_no: &str) -> bool { false }
    fn student_email_exists(&self, _school: educore_core::ids::SchoolId, _email: &str) -> bool { false }
    fn roll_no_exists(&self, _school: educore_core::ids::SchoolId, _class_id: educore_academic::ClassId, _section_id: educore_academic::SectionId, _academic_year_id: educore_academic::AcademicYearId, _roll_no: &str) -> bool { false }
    fn class_name_exists(&self, _school: educore_core::ids::SchoolId, _name: &str) -> bool { false }
    fn section_name_exists(&self, _school: educore_core::ids::SchoolId, _name: &str) -> bool { false }
    fn subject_code_exists(&self, _school: educore_core::ids::SchoolId, _code: &str) -> bool { false }
    fn academic_year_overlaps(&self, _school: educore_core::ids::SchoolId, _range: educore_academic::AcademicYearRange, _exclude_id: Option<educore_academic::AcademicYearId>) -> bool { false }
    fn optional_subject_assigned_exists(&self, _school: educore_core::ids::SchoolId, _student_id: educore_academic::StudentId, _academic_year_id: educore_academic::AcademicYearId) -> bool { false }
    fn primary_guardian_link_exists(&self, _school: educore_core::ids::SchoolId, _student_id: educore_academic::StudentId) -> bool { false }
}
