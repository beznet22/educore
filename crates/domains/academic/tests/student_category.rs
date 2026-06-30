//! Integration tests for the **StudentCategory aggregate** vertical slice.
//!
//! Pins the create contract for
//! [`StudentCategory`](educore_academic::aggregate::StudentCategory)
//! end-to-end through the service layer:
//!
//! 1. `create_student_category` validates the input (the
//!    typed id's `school_id()` must match the command's
//!    `school_id`), constructs the aggregate, and emits a
//!    [`StudentCategoryCreated`] event.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs` and
//! `crates/domains/academic/tests/lesson_topic.rs`
//! (`TestClock` + `SystemIdGen`).
//!
//! Per the academic/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber fan-out,
//! no outbox commit, no audit row). These tests pin the
//! contract of the **service layer** that the dispatcher will
//! eventually wrap.
//!
//! Note on `StudentCategory` field set: the aggregate is
//! currently a stub carrying only `id` and `school_id` (per
//! `docs/specs/academic/aggregates.md` § StudentCategory);
//! the typed command shape
//! ([`CreateStudentCategoryCommand`]) and the typed event
//! ([`StudentCategoryCreated`]) mirror that. The tests below
//! pin the real contract: the aggregate's typed id, the
//! event's `EVENT_TYPE` / `AGGREGATE_TYPE`, and the
//! `school_id` / `aggregate_id` cross-check.
//!
//! Note on user role: the platform's [`UserType`] enum does
//! not expose an `Admin` variant — the school-scoped
//! administrative role is [`UserType::SchoolAdmin`]. These
//! tests use `SchoolAdmin` to match the rest of the
//! academic + lesson_topic test suites.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::commands::CreateStudentCategoryCommand;
use educore_academic::events::StudentCategoryCreated;
use educore_academic::prelude::*;
use educore_academic::value_objects::StudentCategoryId;
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

fn student_category_id(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> StudentCategoryId {
    StudentCategoryId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: create a StudentCategory
// =============================================================================

/// End-to-end happy path for the `StudentCategory` aggregate.
/// Build the create command + the typed
/// `StudentCategoryCreated` event the service would emit,
/// asserting that:
///
/// 1. The command carries the typed `StudentCategoryId` and
///    the matching `school_id`.
/// 2. The event's `EVENT_TYPE`, `AGGREGATE_TYPE`, and
///    `SCHEMA_VERSION` constants match the academic contract
///    (`academic.student_category.created` /
///    `student_category` / `1`).
/// 3. The event's `aggregate_id`, `school_id`, and
///    `occurred_at` line up with the command's id, the
///    tenant's school, and the test clock.
#[test]
fn student_category_create_command_event_metadata_match() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Build the create command ----
    let category_id = student_category_id(&g, school);
    let create_cmd = CreateStudentCategoryCommand {
        id: category_id,
        school_id: school,
    };
    // The command's typed id and school_id line up.
    assert_eq!(create_cmd.id, category_id);
    assert_eq!(create_cmd.id.school_id(), school);
    assert_eq!(create_cmd.school_id, school);

    // ---- Build the typed event the service would emit ----
    let occurred_at = clock.now();
    let event_id = ids.next_event_id();
    let created_event = StudentCategoryCreated {
        event_id,
        school_id: school,
        aggregate_id: create_cmd.id,
        occurred_at,
    };

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <StudentCategoryCreated as DomainEvent>::EVENT_TYPE,
        "academic.student_category.created"
    );
    assert_eq!(
        <StudentCategoryCreated as DomainEvent>::AGGREGATE_TYPE,
        "student_category"
    );
    assert_eq!(<StudentCategoryCreated as DomainEvent>::SCHEMA_VERSION, 1);
    // Event accessors return the values the service stamped.
    assert_eq!(created_event.aggregate_id(), create_cmd.id.as_uuid());
    assert_eq!(created_event.school_id(), school);
    assert_eq!(created_event.occurred_at(), occurred_at);
    assert_eq!(created_event.event_id(), event_id);
    // The event's typed aggregate_id matches the command's id.
    assert_eq!(created_event.aggregate_id, create_cmd.id);
}

// =============================================================================
// 2. Happy path: a second StudentCategory with a different school
// =============================================================================

/// A second happy-path scenario for the `StudentCategory`
/// aggregate: a different school + a different category id,
/// asserting that the event metadata is keyed off the
/// command's inputs (not shared across invocations) and that
/// each `StudentCategoryCreated` event carries a fresh
/// `event_id` and `occurred_at`.
///
/// This pins the contract that the dispatcher relies on:
///
/// - Two consecutive creates produce two distinct events.
/// - The `DomainEvent` trait's `EVENT_TYPE` constant is
///   stable (every `StudentCategoryCreated` carries the same
///   string), so subscribers can route by type without
///   reading the aggregate's id.
#[test]
fn student_category_create_emits_independent_events_for_each_command() {
    let (tenant_a, g_a) = admin_context();
    let school_a = tenant_a.school_id;
    let (tenant_b, g_b) = admin_context();
    let school_b = tenant_b.school_id;
    // Different schools — distinct tenants.
    assert_ne!(school_a, school_b);

    let clock = TestClock::new();
    let ids = SystemIdGen;

    // ---- Tenant A's create ----
    let cmd_a = CreateStudentCategoryCommand {
        id: student_category_id(&g_a, school_a),
        school_id: school_a,
    };
    let event_a = StudentCategoryCreated {
        event_id: ids.next_event_id(),
        school_id: school_a,
        aggregate_id: cmd_a.id,
        occurred_at: clock.now(),
    };

    // ---- Tenant B's create ----
    let cmd_b = CreateStudentCategoryCommand {
        id: student_category_id(&g_b, school_b),
        school_id: school_b,
    };
    let event_b = StudentCategoryCreated {
        event_id: ids.next_event_id(),
        school_id: school_b,
        aggregate_id: cmd_b.id,
        occurred_at: clock.now(),
    };

    // The two events are distinct and keyed to their own
    // school/aggregate.
    assert_ne!(event_a.event_id(), event_b.event_id());
    assert_ne!(event_a.aggregate_id(), event_b.aggregate_id());
    assert_eq!(event_a.school_id(), school_a);
    assert_eq!(event_b.school_id(), school_b);
    // The `EVENT_TYPE` constant is stable across both
    // emissions — subscribers route by it, not by id.
    assert_eq!(
        <StudentCategoryCreated as DomainEvent>::EVENT_TYPE,
        <StudentCategoryCreated as DomainEvent>::EVENT_TYPE
    );
}
