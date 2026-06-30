//! Integration tests for the **AdmitCard aggregate** vertical slice.
//!
//! Pins the create + regenerate contract for
//! [`AdmitCard`](educore_assessment::aggregate::AdmitCard)
//! end-to-end through the service layer:
//!
//! 1. `generate_admit_card` constructs the aggregate
//!    (id, student_record_id, exam_type_id, academic_year_id,
//    generated_at, version = 1, active) and emits an
//!    [`AdmitCardGenerated`](educore_assessment::events::AdmitCardGenerated)
//!    event.
//! 2. `regenerate_admit_card` produces an
//!    [`AdmitCardRegenerated`](educore_assessment::events::AdmitCardRegenerated)
//!    event carrying the new id, the previous id, and the
//!    reason text.
//!
//! The tests use the same fixture pattern as
//! `crates/domains/assessment/tests/exam.rs`
//! (`TestClock` + `SystemIdGen` + `SchoolAdmin` tenant).
//!
//! Per the assessment/workflows.rs pattern, the **handlers**
//! themselves are not wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests
//! pin the contract of the **service layer** that the
//! dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/assessment/tests/exam.rs` (lean).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_assessment::commands::{
    GenerateAdmitCardCommand, RegenerateAdmitCardCommand,
};
use educore_assessment::events::{AdmitCardGenerated, AdmitCardRegenerated};
use educore_assessment::prelude::*;
use educore_assessment::services::{generate_admit_card, regenerate_admit_card};
use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};

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

// =============================================================================
// 1. Happy path: generate_admit_card end-to-end
// =============================================================================

/// End-to-end happy path for the `AdmitCard` aggregate.
/// Mints a fresh school + actor, builds a
/// `GenerateAdmitCardCommand`, and asserts that:
///
/// 1. `generate_admit_card` returns an `AdmitCard`
///    aggregate carrying every field on the command plus
///    a fresh audit footer (`version = 1`, etag =
///    `0000...`, `active_status = Active`).
/// 2. The aggregate is anchored to the tenant's school and
///    the typed `id` matches the command's
///    `admit_card_id`.
/// 3. The emitted `AdmitCardGenerated` event has the right
///    `event_type` (`assessment.admit_card.generated`),
///    `aggregate_type` (`admit_card`), `schema_version`
///    (1), plus a matching `aggregate_id` and `school_id`.
#[test]
fn admit_card_generate_builds_aggregate_and_emits_admit_card_generated_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let admit_card_id = AdmitCardId::new(school, g.next_uuid());
    let student_record_id = StudentRecordId::new(school, g.next_uuid());
    let exam_type_id = ExamTypeId::new(school, g.next_uuid());
    let academic_year_id = AcademicYearId::new(school, g.next_uuid());

    let cmd = GenerateAdmitCardCommand {
        tenant: tenant.clone(),
        admit_card_id,
        student_record_id,
        exam_type_id,
        academic_year_id,
    };

    let now = clock.now();
    let (card, event) =
        generate_admit_card(cmd, &clock, &ids).expect("generate_admit_card");

    // Aggregate fields are populated from the command.
    assert_eq!(card.id, admit_card_id);
    assert_eq!(card.school_id, school);
    assert_eq!(card.student_record_id, student_record_id);
    assert_eq!(card.exam_type_id, exam_type_id);
    assert_eq!(card.academic_year_id, academic_year_id);
    // Audit metadata footer is initialised.
    assert_eq!(card.version.get(), 1);
    assert_eq!(card.etag.as_str(), AdmitCard::FRESH_ETAG);
    assert!(card.active_status.is_active());
    assert_eq!(card.created_by, tenant.actor_id);
    assert_eq!(card.updated_by, tenant.actor_id);
    assert_eq!(card.generated_at, now);
    assert_eq!(card.correlation_id, tenant.correlation_id);

    // Event metadata matches the DomainEvent trait contract.
    assert_eq!(
        <AdmitCardGenerated as DomainEvent>::EVENT_TYPE,
        "assessment.admit_card.generated"
    );
    assert_eq!(
        <AdmitCardGenerated as DomainEvent>::AGGREGATE_TYPE,
        "admit_card"
    );
    assert_eq!(<AdmitCardGenerated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), admit_card_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.admit_card_id, admit_card_id);
    assert_eq!(event.student_record_id, student_record_id);
    assert_eq!(event.exam_type_id, exam_type_id);
    assert_eq!(event.academic_year_id, academic_year_id);
    assert_eq!(event.occurred_at, now);
}

// =============================================================================
// 2. Happy path: regenerate_admit_card end-to-end
// =============================================================================

/// End-to-end happy path for `regenerate_admit_card`.
/// Builds a `RegenerateAdmitCardCommand` (new id,
/// previous id, reason) and asserts that the returned
/// `AdmitCardRegenerated` event:
///
/// 1. Carries `event_type =
///    "assessment.admit_card.regenerated"`, `aggregate_type
///    = "admit_card"`, `schema_version = 1`.
/// 2. Carries the new `admit_card_id`, the `previous_id`,
///    and the reason verbatim.
/// 3. Has a fresh `aggregate_id` matching the new
///    `admit_card_id`, and is anchored to the tenant's
///    school.
#[test]
fn admit_card_regenerate_emits_admit_card_regenerated_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let new_id = AdmitCardId::new(school, g.next_uuid());
    let previous_id = AdmitCardId::new(school, g.next_uuid());
    let reason = "Photo updated".to_owned();

    let cmd = RegenerateAdmitCardCommand {
        tenant: tenant.clone(),
        admit_card_id: new_id,
        previous_id,
        reason: reason.clone(),
    };

    let now = clock.now();
    let event =
        regenerate_admit_card(cmd, &clock, &ids).expect("regenerate_admit_card");

    // Event metadata matches the DomainEvent trait contract.
    assert_eq!(
        <AdmitCardRegenerated as DomainEvent>::EVENT_TYPE,
        "assessment.admit_card.regenerated"
    );
    assert_eq!(
        <AdmitCardRegenerated as DomainEvent>::AGGREGATE_TYPE,
        "admit_card"
    );
    assert_eq!(<AdmitCardRegenerated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(event.aggregate_id(), new_id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.admit_card_id, new_id);
    assert_eq!(event.previous_id, previous_id);
    assert_eq!(event.reason, reason);
    assert_eq!(event.occurred_at, now);
    assert_eq!(event.correlation_id, tenant.correlation_id);
}
