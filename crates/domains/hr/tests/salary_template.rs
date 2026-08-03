//! Integration tests for the **SalaryTemplate aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`SalaryTemplate`](educore_hr::aggregate::SalaryTemplate)
//! end-to-end, plus the Wave 187 mutators that enforce spec
//! invariants I-1 (composite-key uniqueness), I-2 (gross_salary
//! composition), I-3 (net_salary composition), and I-4
//! (active while in use).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::value_objects::AcademicYearId;
use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_hr::prelude::SalaryTemplate;
use educore_hr::services::SalaryTemplateUniquenessChecker;
use educore_hr::value_objects::SalaryTemplateId;

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

/// Helper: build a consistent SalaryTemplate for tests.
fn fresh_salary_template(
    tenant: &TenantContext,
    g: &SystemIdGen,
) -> SalaryTemplate {
    let school = tenant.school_id;
    let id = salary_template_id(g, school);
    let academic_id = AcademicYearId::new(school, g.next_uuid());
    let salary_basic = 10_000.0;
    let house_rent = 5_000.0;
    let provident_fund = 1_000.0;
    let gross_salary = salary_basic + house_rent + provident_fund;
    let total_deduction = 2_000.0;
    let net_salary = gross_salary - total_deduction;
    SalaryTemplate::fresh(
        id,
        "Grade-A".to_owned(),
        salary_basic,
        0.0, // overtime_rate
        house_rent,
        provident_fund,
        gross_salary,
        total_deduction,
        net_salary,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `SalaryTemplateUniquenessChecker` mock.
struct FakeSalaryTemplateUniqueness {
    exists: bool,
}
impl SalaryTemplateUniquenessChecker for FakeSalaryTemplateUniqueness {
    fn salary_template_exists(
        &self,
        _school: SchoolId,
        _salary_grades: &str,
        _academic_id: AcademicYearId,
    ) -> bool {
        self.exists
    }
}

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

// =============================================================================
// Wave 187 — Spec invariant SalaryTemplate#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn salary_template_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let st = fresh_salary_template(&tenant, &g);
    let checker = FakeSalaryTemplateUniqueness { exists: false };
    assert!(st.ensure_unique(&checker).is_ok());
}

#[test]
fn salary_template_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let st = fresh_salary_template(&tenant, &g);
    let checker = FakeSalaryTemplateUniqueness { exists: true };
    let err = st.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 187 — Spec invariant SalaryTemplate#2 (gross_salary composition)
// =============================================================================

#[test]
fn salary_template_ensure_gross_salary_consistent_accepts_consistent() {
    let (tenant, g) = admin_context();
    let st = fresh_salary_template(&tenant, &g);
    assert!(st.ensure_gross_salary_consistent().is_ok());
}

#[test]
fn salary_template_ensure_gross_salary_consistent_rejects_drift() {
    let (tenant, g) = admin_context();
    let mut st = fresh_salary_template(&tenant, &g);
    st.gross_salary += 1.0; // drift beyond epsilon
    let err = st
        .ensure_gross_salary_consistent()
        .expect_err("drift must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// Wave 187 — Spec invariant SalaryTemplate#3 (net_salary composition)
// =============================================================================

#[test]
fn salary_template_ensure_net_salary_consistent_accepts_consistent() {
    let (tenant, g) = admin_context();
    let st = fresh_salary_template(&tenant, &g);
    assert!(st.ensure_net_salary_consistent().is_ok());
}

#[test]
fn salary_template_ensure_net_salary_consistent_rejects_drift() {
    let (tenant, g) = admin_context();
    let mut st = fresh_salary_template(&tenant, &g);
    st.net_salary += 1.0;
    let err = st
        .ensure_net_salary_consistent()
        .expect_err("drift must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =============================================================================
// Wave 187 — Spec invariant SalaryTemplate#4 (active while in use)
// =============================================================================

#[test]
fn salary_template_ensure_active_accepts_active() {
    let (tenant, g) = admin_context();
    let st = fresh_salary_template(&tenant, &g);
    assert!(st.ensure_active().is_ok());
}

#[test]
fn salary_template_ensure_active_rejects_inactive() {
    let (tenant, g) = admin_context();
    let mut st = fresh_salary_template(&tenant, &g);
    st.active_status = educore_core::value_objects::ActiveStatus::Retired;
    let err = st.ensure_active().expect_err("inactive must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
