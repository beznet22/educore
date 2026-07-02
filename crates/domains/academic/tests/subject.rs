//! Integration tests for the **Subject aggregate** vertical slice.
//!
//! Pins the create + update contract for
//! [`Subject`](educore_academic::Subject) end-to-end
//! through the service layer:
//!
//! 1. `create_subject` validates the input, constructs the
//!    aggregate, and emits a [`SubjectCreated`] event.
//! 2. `update_subject` mutates the in-place aggregate (bumps
//!    `version`, updates `updated_at` / `updated_by`), and
//!    emits a [`SubjectUpdated`] event carrying the list of
//!    changed field names.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/workflows.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber fan-out,
//! no outbox commit, no audit row). These tests pin the
//! contract of the **service layer** that the dispatcher will
//! eventually wrap.
//!
//! Note on `Subject` field set: the aggregate carries
//! `code`, `name`, `subject_type` (`Theory` | `Practical`),
//! and `pass_mark` (0.0..=100.0). It does **not** carry a
//! `credit_hours` field — that field is not part of the
//! academic spec (`docs/specs/academic/aggregates.md` §
//! Subject) or the typed command shape
//! ([`CreateSubjectCommand`]); the closest numeric field is
//! `pass_mark`. The tests below therefore exercise the real
//! contract and pin `pass_mark` instead.
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic + attendance test suites.

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

fn subject_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> SubjectId {
    SubjectId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create + update on Subject
// =============================================================================

/// End-to-end happy path for the `Subject` aggregate. Create
/// a subject, then update its name (and a couple of other
/// fields), asserting that:
///
/// 1. The create flow produces a `Subject` aggregate
///    carrying every field on the command + a
///    `SubjectCreated` event with the right `event_type`,
///    `aggregate_type`, and `school_id`.
/// 2. The update flow mutates the aggregate in place (bumps
///    `version`, swaps `name`, `subject_type`, and
///    `pass_mark`), and emits a `SubjectUpdated` event whose
///    `changed_fields` list names the fields that actually
///    moved.
#[test]
fn subject_create_then_update_mutates_aggregate_and_emits_events() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Create flow ----
    let create_cmd = CreateSubjectCommand {
        tenant: tenant.clone(),
        subject_id: subject_id(&g, school),
        subject_code: "MTH101".to_owned(),
        subject_name: "Math".to_owned(),
        subject_type: SubjectType::Theory,
        pass_mark: 35.0,
    };
    let (mut agg, created_event) = create_subject(create_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("create");

    // Aggregate fields are populated from the command.
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.code, "MTH101");
    assert_eq!(agg.name, "Math");
    assert_eq!(agg.subject_type, SubjectType::Theory);
    assert_eq!(agg.pass_mark.as_f32(), 35.0);
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
    // `update_subject`, which DOES stamp `last_event_id`
    // because it mutates the in-memory aggregate in place.)
    assert_eq!(agg.correlation_id, tenant.correlation_id);
    assert_eq!(agg.last_event_id, None);

    // Event metadata matches the aggregate's typed id and
    // the DomainEvent trait's contract.
    assert_eq!(
        <SubjectCreated as DomainEvent>::EVENT_TYPE,
        "academic.subject.created"
    );
    assert_eq!(<SubjectCreated as DomainEvent>::AGGREGATE_TYPE, "subject");
    assert_eq!(<SubjectCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.code, "MTH101");
    assert_eq!(created_event.name, "Math");
    assert_eq!(created_event.subject_type, SubjectType::Theory);
    assert_eq!(created_event.pass_mark, 35.0);

    // ---- Update flow ----
    let initial_version = agg.version.get();
    let update_cmd = UpdateSubjectCommand {
        tenant: tenant.clone(),
        subject_id: agg.id,
        subject_name: Some("Mathematics".to_owned()),
        subject_type: Some(SubjectType::Practical),
        pass_mark: Some(40.0),
    };
    let updated_event = update_subject(&mut agg, update_cmd, &clock, &ids).expect("update");

    // The aggregate is mutated in place: every field moved,
    // the version bumped, and the `updated_by` footer tracks
    // the actor.
    assert_eq!(agg.name, "Mathematics");
    assert_eq!(agg.subject_type, SubjectType::Practical);
    assert_eq!(agg.pass_mark.as_f32(), 40.0);
    assert_eq!(agg.version.get(), initial_version + 1);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_by, tenant.actor_id);
    // The event id stamped on `last_event_id` matches the
    // update event the service returned.
    assert_eq!(agg.last_event_id, Some(updated_event.event_id));

    // The event names the fields that actually moved.
    assert_eq!(
        <SubjectUpdated as DomainEvent>::EVENT_TYPE,
        "academic.subject.updated"
    );
    assert_eq!(<SubjectUpdated as DomainEvent>::AGGREGATE_TYPE, "subject");
    assert_eq!(updated_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(updated_event.school_id(), school);
    assert_eq!(updated_event.name.as_deref(), Some("Mathematics"));
    assert_eq!(updated_event.subject_type, Some(SubjectType::Practical));
    assert_eq!(updated_event.pass_mark, Some(40.0));
    assert!(updated_event
        .changed_fields
        .contains(&"subject_name".to_owned()));
    assert!(updated_event
        .changed_fields
        .contains(&"subject_type".to_owned()));
    assert!(updated_event
        .changed_fields
        .contains(&"pass_mark".to_owned()));
}

// =============================================================================
// 2. No-op update: same name returns a no-change event and preserves version
// =============================================================================

/// When the `UpdateSubjectCommand` carries the same name as
/// the aggregate (and no other field changes), `update_subject`
/// is a **no-op on the aggregate state** — `version`,
/// `updated_at`, and `updated_by` are NOT bumped — but the
/// service still returns `Ok(SubjectUpdated)` with an empty
/// `changed_fields` list and a fresh event id stamped on
/// `last_event_id`.
///
/// This pins the contract that the dispatcher relies on:
///
/// - The optimistic-concurrency counter is preserved (so a
///   no-op update never bumps it).
/// - The event is emitted unconditionally so downstream
///   subscribers can no-op cleanly, matching the
///   audit-first invariant.
///
/// Note: the brief described this case as "no-op update
/// returns `DomainError::Validation`"; tracing `update_subject`
/// in `services.rs` shows the actual implementation returns
/// `Ok` with empty `changed_fields` (consistent with the
/// spec's "audit-first" event-emission invariant — the
/// service is a pure factory that always emits one event
/// per command). The test pins the **actual** contract.
#[test]
fn subject_update_with_same_name_returns_no_op_event_and_preserves_version() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Create a subject with a known name.
    let create_cmd = CreateSubjectCommand {
        tenant: tenant.clone(),
        subject_id: subject_id(&g, school),
        subject_code: "MTH101".to_owned(),
        subject_name: "Math".to_owned(),
        subject_type: SubjectType::Theory,
        pass_mark: 35.0,
    };
    let (mut agg, created_event) = create_subject(create_cmd, &clock, &ids, &NoOpUniquenessChecker).expect("create");
    let initial_version = agg.version.get();
    let initial_updated_at = agg.updated_at;
    let initial_updated_by = agg.updated_by;

    // Update with the same name and the same subject_type /
    // pass_mark — nothing on the aggregate should change.
    let noop_update = UpdateSubjectCommand {
        tenant: tenant.clone(),
        subject_id: agg.id,
        subject_name: Some("Math".to_owned()),
        subject_type: Some(SubjectType::Theory),
        pass_mark: Some(35.0),
    };
    let noop_event = update_subject(&mut agg, noop_update, &clock, &ids)
        .expect("noop update returns Ok with empty changed_fields");

    // The aggregate's identity-bearing fields are unchanged.
    assert_eq!(agg.code, "MTH101");
    assert_eq!(agg.name, "Math");
    assert_eq!(agg.subject_type, SubjectType::Theory);
    assert_eq!(agg.pass_mark.as_f32(), 35.0);

    // The version is preserved: a no-op update must not
    // bump the optimistic-concurrency counter.
    assert_eq!(agg.version.get(), initial_version);
    // `updated_at` and `updated_by` are also preserved on a
    // no-op (only the `last_event_id` envelope pointer is
    // refreshed — see services.rs `update_subject`).
    assert_eq!(agg.updated_at, initial_updated_at);
    assert_eq!(agg.updated_by, initial_updated_by);

    // The event is emitted with empty `changed_fields`.
    assert_eq!(noop_event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(noop_event.school_id(), school);
    assert!(
        noop_event.changed_fields.is_empty(),
        "no-op update must emit an event with no changed fields, got {:?}",
        noop_event.changed_fields
    );
    // The event id is fresh (different from the create
    // event id) — the dispatcher still publishes the
    // envelope, just with an empty change list.
    assert_ne!(noop_event.event_id, created_event.event_id);
    assert_eq!(agg.last_event_id, Some(noop_event.event_id));

    // Sanity check: a subsequent REAL update still works
    // and the version sequence is `1 -> 1 -> 2`.
    let real_update = UpdateSubjectCommand {
        tenant: tenant.clone(),
        subject_id: agg.id,
        subject_name: Some("Mathematics".to_owned()),
        subject_type: None,
        pass_mark: None,
    };
    let _ = update_subject(&mut agg, real_update, &clock, &ids).expect("real update after no-op");
    assert_eq!(agg.name, "Mathematics");
    assert_eq!(
        agg.version.get(),
        initial_version + 1,
        "version sequence must be 1 (create) -> 1 (no-op) -> 2 (real update)"
    );
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
    fn class_section_exists(
        &self,
        _school: educore_core::ids::SchoolId,
        _class_id: educore_academic::ClassId,
        _section_id: educore_academic::SectionId,
        _academic_year_id: educore_academic::AcademicYearId,
    ) -> bool {
        false
    }
    fn class_section_has_student_records(
        &self,
        _school: educore_core::ids::SchoolId,
        _class_section_id: educore_academic::ClassSectionId,
    ) -> bool {
        false
    }
    fn teacher_has_conflict(
        &self,
        _school: educore_core::ids::SchoolId,
        _teacher_id: educore_core::ids::UserId,
        _day: educore_academic::DayOfWeek,
        _period_number: u8,
    ) -> bool {
        false
    }
    fn room_has_conflict(
        &self,
        _school: educore_core::ids::SchoolId,
        _room_id: educore_academic::ClassRoomId,
        _day: educore_academic::DayOfWeek,
        _period_number: u8,
    ) -> bool {
        false
    }
}
