//! Integration tests for the **SalaryTemplate aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 82 per-aggregate drop
//! [`RealSalaryTemplate`](educore_finance::aggregate::RealSalaryTemplate) —
//! the per-school salary template reference data that pins the final
//! computed `gross_salary_minor` + `net_salary_minor` for query/
//! report without recomputation. Validates ST I-1
//! (`gross_salary_minor >= 0` pinned at construction), ST I-2 lower
//! bound (`net_salary_minor >= 0` pinned at construction), name
//! non-empty after trim, the `update_metadata` mutator with
//! re-validation, `retire()` (active → retired transition that
//! preserves the original audit footer), and the
//! `create_salary_template` service function (aggregate + event
//! pairing with EVENT_TYPE / AGGREGATE_TYPE / SCHEMA_VERSION
//! pinned).
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `SalaryTemplate` previously had no real
//! implementation beyond a `finance_aggregate_stub! { struct
//! SalaryTemplate { _id: () } }` placeholder + the partial
//! service-side `SalaryTemplateService::create_template` +
//! `apply_template` helpers at services.rs:2984/3026 (composition).
//! Wave 82 adds the `RealSalaryTemplate` aggregate (full lifecycle:
//! fresh + update_metadata + retire), the 3 headline events, the
//! service function, and this test suite. Composition (gross ==
//! sum of earnings, net == gross - total_deduction) is service-side
//! — this aggregate pins the FINAL values for query/report.
//!
//! Promotion of the checklist from `[~]` partial (service-side) to
//! `[x]` complete (aggregate-side pinned values + events + test
//! coverage).

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
use educore_hr::value_objects::SalaryTemplateId;

use educore_finance::commands::CreateSalaryTemplateCommand;
use educore_finance::events::SalaryTemplateCreated;
use educore_finance::prelude::RealSalaryTemplate;
use educore_finance::services::create_salary_template;
use educore_finance::value_objects::Currency;

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

fn salary_template_id(g: &SystemIdGen, school: SchoolId) -> SalaryTemplateId {
    SalaryTemplateId::new(school, g.next_uuid())
}

fn make_salary_template(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    gross: i64,
    net: i64,
) -> RealSalaryTemplate {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    RealSalaryTemplate::fresh(
        salary_template_id(g, school),
        name.to_owned(),
        Currency::INR,
        gross,
        net,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh RealSalaryTemplate")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 82 stub tests)
// =========================================================================

#[test]
fn salary_template_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = salary_template_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn salary_template_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = salary_template_id(&g, school);
    let id_b = salary_template_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealSalaryTemplate::fresh — ST I-1 + ST I-2 lower bound
// =========================================================================

#[test]
fn fresh_pins_gross_and_net_salary() {
    // ST I-1 + ST I-2 lower bound: gross and net must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_salary_template(&g, school, "Senior Teacher", 500_000, 400_000);
    assert_eq!(row.gross_salary_minor, 500_000);
    assert_eq!(row.net_salary_minor, 400_000);
    assert_eq!(row.name, "Senior Teacher");
    assert_eq!(row.currency, Currency::INR);
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_zero_gross_and_zero_net_is_valid() {
    // ST I-1 + ST I-2 lower bound uses >= 0 (not > 0). A zero-gross
    // template (e.g. volunteer or honorary position) is valid.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_salary_template(&g, school, "Volunteer", 0, 0);
    assert_eq!(row.gross_salary_minor, 0);
    assert_eq!(row.net_salary_minor, 0);
    assert!(row.is_active());
}

#[test]
fn fresh_rejects_negative_gross() {
    // ST I-1: gross_salary_minor must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealSalaryTemplate::fresh(
        salary_template_id(&g, school),
        "Bad Template".to_owned(),
        Currency::INR,
        -1,
        0,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("negative gross must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_rejects_negative_net() {
    // ST I-2 lower bound: net_salary_minor must be >= 0.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    let err = RealSalaryTemplate::fresh(
        salary_template_id(&g, school),
        "Bad Template".to_owned(),
        Currency::INR,
        100,
        -1,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("negative net must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_trims_name_and_rejects_empty() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    // Trim leading/trailing whitespace.
    let row = RealSalaryTemplate::fresh(
        salary_template_id(&g, school),
        "  pad me  ".to_owned(),
        Currency::INR,
        100,
        80,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("trim is OK");
    assert_eq!(row.name, "pad me");
    // Reject empty after trim.
    let err = RealSalaryTemplate::fresh(
        salary_template_id(&g, school),
        "   ".to_owned(),
        Currency::INR,
        100,
        80,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("empty name must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let before = SystemClock.now();
    let row = make_salary_template(&g, school, "Junior Teacher", 200_000, 160_000);
    let after = SystemClock.now();
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
    assert_eq!(row.created_by, row.updated_by);
    assert!(row.last_event_id.is_none());
    assert!(row.is_active());
}

// =========================================================================
// RealSalaryTemplate::update_metadata
// =========================================================================

#[test]
fn update_metadata_updates_name_currency_gross_net_description() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Initial Name", 100, 80);
    let original_version = row.version;
    let later = SystemClock.now();
    row.update_metadata(
        "Revised Name".to_owned(),
        Currency::USD,
        200,
        160,
        Some("revised description".to_owned()),
        later,
        g.next_user_id(),
    )
    .expect("update");
    assert_eq!(row.name, "Revised Name");
    assert_eq!(row.currency, Currency::USD);
    assert_eq!(row.gross_salary_minor, 200);
    assert_eq!(row.net_salary_minor, 160);
    assert_eq!(row.description.as_deref(), Some("revised description"));
    assert_eq!(row.updated_at, later);
    assert!(row.version > original_version);
}

#[test]
fn update_metadata_validates_negative_gross() {
    // ST I-1 on update.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Template", 100, 80);
    let now = SystemClock.now();
    let err = row
        .update_metadata(
            "Template".to_owned(),
            Currency::INR,
            -10,
            0,
            None,
            now,
            g.next_user_id(),
        )
        .expect_err("negative gross on update must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    // The original gross is preserved on validation failure.
    assert_eq!(row.gross_salary_minor, 100);
}

#[test]
fn update_metadata_validates_negative_net() {
    // ST I-2 lower bound on update.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Template", 100, 80);
    let now = SystemClock.now();
    let err = row
        .update_metadata(
            "Template".to_owned(),
            Currency::INR,
            100,
            -10,
            None,
            now,
            g.next_user_id(),
        )
        .expect_err("negative net on update must be rejected");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
    assert_eq!(row.net_salary_minor, 80);
}

#[test]
fn update_metadata_rejects_on_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Template", 100, 80);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    let later = SystemClock.now();
    let err = row
        .update_metadata(
            "Revised".to_owned(),
            Currency::INR,
            200,
            160,
            None,
            later,
            g.next_user_id(),
        )
        .expect_err("update on retired must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// RealSalaryTemplate::retire
// =========================================================================

#[test]
fn retire_flips_active_status_and_preserves_gross_net() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Template", 100, 80);
    let before = row.version;
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    assert!(!row.is_active());
    // ST I-1 + ST I-2 preserved in the audit footer.
    assert_eq!(row.gross_salary_minor, 100);
    assert_eq!(row.net_salary_minor, 80);
    assert_eq!(row.updated_at, now);
    assert!(row.version > before);
}

#[test]
fn retire_rejects_double_retire() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_salary_template(&g, school, "Template", 100, 80);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("first retire");
    let err = row
        .retire(now, g.next_user_id())
        .expect_err("double retire must be rejected");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =========================================================================
// create_salary_template service function
// =========================================================================

#[test]
fn create_service_produces_aggregate_and_created_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = salary_template_id(&g, school);
    let cmd = CreateSalaryTemplateCommand {
        tenant: tenant.clone(),
        salary_template_id: id,
        name: "Senior Teacher".to_owned(),
        currency: Currency::INR,
        gross_salary_minor: 500_000,
        net_salary_minor: 400_000,
        description: Some("base + housing".to_owned()),
    };
    let clock = SystemClock;
    let (row, event) = create_salary_template(cmd, &clock, &g)
        .expect("create_salary_template should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.name, "Senior Teacher");
    assert_eq!(row.gross_salary_minor, 500_000);
    assert_eq!(row.net_salary_minor, 400_000);
    assert_eq!(event.salary_template_id, id);
    assert_eq!(
        <SalaryTemplateCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.salary_template.created"
    );
    assert_eq!(
        <SalaryTemplateCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "salary_template"
    );
    assert_eq!(
        <SalaryTemplateCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id(), id.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn create_service_propagates_negative_gross_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = salary_template_id(&g, school);
    let cmd = CreateSalaryTemplateCommand {
        tenant: tenant.clone(),
        salary_template_id: id,
        name: "Bad Template".to_owned(),
        currency: Currency::INR,
        gross_salary_minor: -1,
        net_salary_minor: 0,
        description: None,
    };
    let clock = SystemClock;
    let err = create_salary_template(cmd, &clock, &g)
        .expect_err("negative gross must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn create_service_propagates_negative_net_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = salary_template_id(&g, school);
    let cmd = CreateSalaryTemplateCommand {
        tenant: tenant.clone(),
        salary_template_id: id,
        name: "Bad Template".to_owned(),
        currency: Currency::INR,
        gross_salary_minor: 100,
        net_salary_minor: -1,
        description: None,
    };
    let clock = SystemClock;
    let err = create_salary_template(cmd, &clock, &g)
        .expect_err("negative net must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
