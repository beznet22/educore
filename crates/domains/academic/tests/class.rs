//! Integration tests for the **Class aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`Class`](educore_academic::Class) end-to-end
//! through the service layer:
//!
//! 1. `create_class` validates the input (the typed
//!    [`class_name`](CreateClassCommand::class_name) enforces
//!    non-empty + length bounds, and
//!    [`pass_mark`](CreateClassCommand::pass_mark) enforces
//!    0.0..=100.0), constructs the aggregate, and emits a
//!    [`ClassCreated`] event.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/workflows.rs` and
//! `crates/domains/academic/tests/subject.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber fan-out,
//! no outbox commit, no audit row). These tests pin the
//! contract of the **service layer** that the dispatcher will
//! eventually wrap.
//!
//! Note on `Class` field set: the aggregate carries
//! `name`, `pass_mark` (0.0..=100.0), and
//! `optional_subject_gpa_threshold` (0.0..=5.0; default 0.0).
//! It does **not** carry an `academic_year_id` field — that
//! field is not part of the academic spec
//! (`docs/specs/academic/aggregates.md` § Class) or the
//! typed command shape ([`CreateClassCommand`]); the
//! per-academic-year pairing is the `ClassSection` aggregate
//! (a later phase). The tests below therefore exercise the
//! real contract and pin `name` + `pass_mark` instead.
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic + subject test suites.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
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

fn class_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create a Class
// =============================================================================

/// End-to-end happy path for the `Class` aggregate. Create
/// a class called "Grade 1" with a pass mark of 35.0,
/// asserting that:
///
/// 1. The create flow produces a `Class` aggregate carrying
///    every field on the command + a `ClassCreated` event
///    with the right `event_type`, `aggregate_type`, and
///    `school_id`.
/// 2. The audit metadata footer is initialised correctly:
///    `version = 1`, `active_status = Active`,
///    `created_by = updated_by = actor_id`, and
///    `correlation_id` is propagated from the tenant
///    context.
/// 3. The event's typed id, name, and pass mark match the
///    command's input.
#[test]
fn class_create_builds_aggregate_and_emits_class_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateClassCommand {
        tenant: tenant.clone(),
        class_id: class_id(&g, school),
        class_name: "Grade 1".to_owned(),
        pass_mark: 35.0,
    };
    let (agg, created_event) = create_class(create_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.name, "Grade 1");
    assert_eq!(agg.pass_mark.as_f32(), 35.0);
    // The default optional-subject GPA threshold is 0.0
    // (any GPA is eligible); the school can raise it later
    // via `SetOptionalSubjectGpaThreshold`.
    assert_eq!(agg.optional_subject_gpa_threshold.as_f32(), 0.0);
    // Audit metadata footer is initialised.
    assert_eq!(agg.version.get(), 1);
    assert!(agg.active_status.is_active());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    // The correlation id is propagated from the tenant
    // context into the aggregate. `last_event_id` stays
    // `None` after the create flow — the service returns
    // the event in the tuple; the storage adapter is what
    // stamps the aggregate after persisting. (Compare to
    // `update_class`, which DOES stamp `last_event_id`
    // because it mutates the in-memory aggregate in place.)
    assert_eq!(agg.correlation_id, tenant.correlation_id);
    assert_eq!(agg.last_event_id, None);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <ClassCreated as DomainEvent>::EVENT_TYPE,
        "academic.class.created"
    );
    assert_eq!(<ClassCreated as DomainEvent>::AGGREGATE_TYPE, "class");
    assert_eq!(<ClassCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.class_name, "Grade 1");
    assert_eq!(created_event.pass_mark, 35.0);
}

// =============================================================================
// 2. Validation failure: empty class_name returns DomainError::Validation
// =============================================================================

/// Validation-failure path on the create flow: when the
/// `class_name` is empty, `create_class` returns
/// `DomainError::Validation` (via `validate_class_name`)
/// and emits no event (the function returns `Err` before
/// the aggregate or the event are constructed).
#[test]
fn class_create_with_empty_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Empty class name — must fail with Validation.
    let create_cmd = CreateClassCommand {
        tenant: tenant.clone(),
        class_id: class_id(&g, school),
        class_name: String::new(),
        pass_mark: 35.0,
    };
    let err =
        create_class(create_cmd, &clock, &ids, &NoOpUniquenessChecker).expect_err("empty class name must fail validation");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );

    // Sanity check: a subsequent call with the same
    // structure but a valid name still succeeds, proving
    // the failure was tied to the empty name (and not to
    // a corrupt clock, ids, or tenant context).
    let ok_cmd = CreateClassCommand {
        tenant,
        class_id: class_id(&g, school),
        class_name: "Grade 1".to_owned(),
        pass_mark: 35.0,
    };
    let (_agg, _event) =
        create_class(ok_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("non-empty class name must succeed");
}

// =============================================================================
// No-op UniquenessChecker for tests
// =============================================================================

/// A no-op `UniquenessChecker` that returns `false` for all
/// uniqueness queries. Tests that don't exercise uniqueness
/// (e.g. happy-path create/update/delete tests) use this stub
/// to satisfy the trait bound without dragging in a
/// backing-store fixture.
///
/// Tests that need to assert uniqueness enforcement (e.g.
/// "creating a class with a duplicate name must fail") use a
/// different stub that records which names/codes have been
/// minted in this test.
struct NoOpUniquenessChecker;

impl educore_academic::commands::UniquenessChecker for NoOpUniquenessChecker {
    fn student_admission_no_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _admission_no: &str,
    ) -> bool {
        false
    }
    fn student_email_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _email: &str,
    ) -> bool {
        false
    }
    fn roll_no_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _class_id: educore_academic::ClassId,
        _section_id: educore_academic::SectionId,
        _academic_year_id: educore_academic::AcademicYearId,
        _roll_no: &str,
    ) -> bool {
        false
    }
    fn class_name_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _name: &str,
    ) -> bool {
        false
    }
    fn section_name_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _name: &str,
    ) -> bool {
        false
    }
    fn subject_code_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _code: &str,
    ) -> bool {
        false
    }
    fn academic_year_overlaps(
        &self,
        _school: educore_core::ids::SchoolId,
        _range: educore_academic::AcademicYearRange,
        _exclude_id: Option<educore_academic::AcademicYearId>,
    ) -> bool {
        false
    }
    fn optional_subject_assigned_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _student_id: educore_academic::StudentId,
        _academic_year_id: educore_academic::AcademicYearId,
    ) -> bool {
        false
    }
    fn primary_guardian_link_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _student_id: educore_academic::StudentId,
    ) -> bool {
        false
    }
}
