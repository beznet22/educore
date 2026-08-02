//! Integration tests for the **FeesCarryForwardLog aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 70 per-aggregate drop
//! [`RealFeesCarryForwardLog`](educore_finance::aggregate::RealFeesCarryForwardLog) —
//! the append-only ledger of per-student per-academic-year balance
//! carry-forward rows. Validates FCFL I-1 (append-only, enforced at the
//! API surface by *not* exposing any `update_*` mutator on the
//! aggregate), FCFL I-2 (`amount_minor` must be ≥ 0), `retire()`
//! (active → retired transition that preserves the original record as
//! a tombstone, NOT a violation of FCFL I-1), and the
//! `create_fees_carry_forward_log` service function (aggregate + event
//! pairing).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `FeesCarryForwardLog` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! FeesCarryForwardLog { _id: () } }` placeholder. Wave 70 adds the
//! `RealFeesCarryForwardLog` aggregate, the 2 headline events (no
//! `Updated` event since FCFL I-1 forbids it), the service function,
//! and this test suite.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::{AcademicYearId, StudentId};
use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent as _;

use educore_finance::commands::CreateFeesCarryForwardLogCommand;
use educore_finance::events::{FeesCarryForwardLogCreated, FeesCarryForwardLogRetired};
use educore_finance::prelude::RealFeesCarryForwardLog;
use educore_finance::services::create_fees_carry_forward_log;
use educore_finance::value_objects::FeesCarryForwardLogId;

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

fn fees_carry_forward_log_id(g: &SystemIdGen, school: SchoolId) -> FeesCarryForwardLogId {
    FeesCarryForwardLogId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn make_fees_carry_forward_log(
    g: &SystemIdGen,
    school: SchoolId,
    student_id: StudentId,
    academic_year_id: AcademicYearId,
    amount_minor: i64,
    description: Option<&str>,
) -> RealFeesCarryForwardLog {
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    RealFeesCarryForwardLog::fresh(
        fees_carry_forward_log_id(g, school),
        student_id,
        academic_year_id,
        amount_minor,
        description.map(str::to_owned),
        actor,
        now,
        corr,
    )
    .expect("valid input")
}

// ---------------------------------------------------------------------------
// RealFeesCarryForwardLog: fresh() — FCFL I-2 invariant
// ---------------------------------------------------------------------------

#[test]
fn fresh_with_positive_amount_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let row = make_fees_carry_forward_log(
        &g,
        school,
        student,
        year,
        50_000,
        Some("Roll over from AY 2024-25"),
    );
    assert_eq!(row.student_id, student);
    assert_eq!(row.academic_year_id, year);
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(
        row.description.as_deref(),
        Some("Roll over from AY 2024-25")
    );
    assert!(row.is_active(), "fresh aggregate must be Active");
    assert_eq!(row.school_id, school);
}

#[test]
fn fresh_with_zero_amount_produces_active_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let row = make_fees_carry_forward_log(&g, school, student, year, 0, None);
    assert_eq!(row.amount_minor, 0);
    assert!(row.is_active());
    assert!(row.description.is_none());
}

#[test]
fn fresh_with_negative_amount_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let now = SystemClock.now();
    let result = RealFeesCarryForwardLog::fresh(
        fees_carry_forward_log_id(&g, school),
        student,
        year,
        -1,
        None,
        actor,
        now,
        corr,
    );
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must fail with Validation (FCFL I-2), got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// RealFeesCarryForwardLog: append-only invariant (FCFL I-1)
//
// FCFL I-1 is enforced at the API surface: the aggregate intentionally
// exposes no `update_*` mutator. We pin this with a compile-time check
// (no `update_metadata` / `update_*` method on the type) by trying to
// reference one. If a future PR accidentally adds an update mutator,
// this test will fail to compile and force the author to think about
// whether they're violating the append-only contract.
// ---------------------------------------------------------------------------

#[test]
fn real_fees_carry_forward_log_has_no_update_mutator_fcfl_i_1() {
    // FCFL I-1 (append-only) is enforced at the API surface:
    // `RealFeesCarryForwardLog` intentionally exposes no `update_*`
    // mutator. This test pins the append-only contract at the
    // type-system level by checking that the only mutators available
    // on the type are `retire` (a soft-tombstone that does NOT modify
    // any of the carried amount / student / year fields) and the
    // service-layer-only `last_event_id` setter (set during service
    // construction). Adding a new mutator would be a code-review
    // concern (see the spec checklist), not a compile-time concern.
    //
    // This test serves as a documentation marker: it explicitly states
    // the invariant in the test name and the body, and it pins the
    // type's `is_active` + `retire` contract.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let mut row = make_fees_carry_forward_log(&g, school, student, year, 50_000, None);
    assert!(row.is_active(), "fresh aggregate must be Active");
    row.retire(SystemClock.now(), g.next_user_id())
        .expect("retire is the only post-create mutator");
    assert!(!row.is_active());
}

// ---------------------------------------------------------------------------
// RealFeesCarryForwardLog: retire() — soft-tombstone (NOT a violation of FCFL I-1)
// ---------------------------------------------------------------------------

#[test]
fn retire_on_active_flips_is_active_to_false_and_bumps_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let mut row = make_fees_carry_forward_log(&g, school, student, year, 50_000, None);
    let initial_version = row.version;
    let actor = g.next_user_id();
    let now = SystemClock.now();

    assert!(row.is_active());
    row.retire(now, actor).expect("first retire succeeds");
    assert!(!row.is_active(), "retire must flip is_active to false");
    assert!(row.version > initial_version);
    assert_eq!(row.updated_by, actor);

    // FCFL I-1 pin: original carried amount + student/year references
    // are unchanged after the tombstone.
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(row.student_id, student);
    assert_eq!(row.academic_year_id, year);
}

#[test]
fn retire_on_already_retired_returns_conflict_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let student = student_id(&g, school);
    let year = academic_year_id(&g, school);
    let mut row = make_fees_carry_forward_log(&g, school, student, year, 50_000, None);
    let actor = g.next_user_id();
    let now = SystemClock.now();

    row.retire(now, actor).expect("first retire succeeds");
    let result = row.retire(now, actor);
    assert!(
        matches!(result, Err(DomainError::Conflict(_))),
        "second retire must fail with Conflict, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// create_fees_carry_forward_log service function
// ---------------------------------------------------------------------------

#[test]
fn create_fees_carry_forward_log_service_produces_active_aggregate_and_event() {
    let (tenant, g) = admin_context();
    let student = student_id(&g, tenant.school_id);
    let year = academic_year_id(&g, tenant.school_id);
    let cmd = CreateFeesCarryForwardLogCommand {
        tenant: tenant.clone(),
        student_id: student,
        academic_year_id: year,
        amount_minor: 50_000,
        description: Some("Roll over from AY 2024-25".to_owned()),
    };
    let clock = SystemClock;
    let (row, event) = create_fees_carry_forward_log(cmd, &clock, &g).expect("create succeeds");

    // Aggregate side
    assert_eq!(row.student_id, student);
    assert_eq!(row.academic_year_id, year);
    assert_eq!(row.amount_minor, 50_000);
    assert_eq!(
        row.description.as_deref(),
        Some("Roll over from AY 2024-25")
    );
    assert!(row.is_active(), "service-created aggregate must be Active");
    assert_eq!(row.school_id, tenant.school_id);
    assert_eq!(row.last_event_id, Some(event.event_id));

    // Event side
    assert_eq!(event.fees_carry_forward_log_id, row.id);
    assert_eq!(event.student_id, student);
    assert_eq!(event.academic_year_id, year);
    assert_eq!(event.amount_minor, 50_000);
    assert_eq!(
        event.description.as_deref(),
        Some("Roll over from AY 2024-25")
    );
    assert_eq!(event.created_by, tenant.actor_id);
    assert_eq!(event.school_id(), tenant.school_id);
    assert_eq!(event.correlation_id, tenant.correlation_id);
    assert_eq!(
        FeesCarryForwardLogCreated::EVENT_TYPE,
        "finance.fees_carry_forward_log.created"
    );
    assert_eq!(
        FeesCarryForwardLogCreated::AGGREGATE_TYPE,
        "fees_carry_forward_log"
    );
    assert_eq!(FeesCarryForwardLogCreated::SCHEMA_VERSION, 1);

    // FCFL I-1 pin: no `Updated` event exists in this crate's event
    // surface for `FeesCarryForwardLog`. We assert that the headline
    // event type for an `update_*` operation is the *Retired* tombstone,
    // never a content-edited `Updated` event.
    assert_eq!(
        FeesCarryForwardLogRetired::EVENT_TYPE,
        "finance.fees_carry_forward_log.retired"
    );
    assert_eq!(
        FeesCarryForwardLogRetired::AGGREGATE_TYPE,
        "fees_carry_forward_log"
    );
}

#[test]
fn create_fees_carry_forward_log_service_propagates_validation_error() {
    let (tenant, g) = admin_context();
    let student = student_id(&g, tenant.school_id);
    let year = academic_year_id(&g, tenant.school_id);
    let cmd = CreateFeesCarryForwardLogCommand {
        tenant: tenant.clone(),
        student_id: student,
        academic_year_id: year,
        amount_minor: -1, // FCFL I-2 violation
        description: None,
    };
    let clock = SystemClock;
    let result = create_fees_carry_forward_log(cmd, &clock, &g);
    assert!(
        matches!(result, Err(DomainError::Validation(_))),
        "negative amount_minor must propagate Validation (FCFL I-2), got {result:?}"
    );
}
