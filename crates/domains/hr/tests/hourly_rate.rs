//! Integration tests for the **HourlyRate aggregate** vertical slice.
//!
//! Pins the typed-id contract for
//! [`HourlyRate`](educore_hr::aggregate::HourlyRate) end-to-end,
//! plus the Wave 189 mutators that enforce spec invariants
//! I-1 (composite-key uniqueness) and I-2 (`rate > 0`).

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
use educore_hr::prelude::HourlyRate;
use educore_hr::services::HourlyRateUniquenessChecker;
use educore_hr::value_objects::HourlyRateId;

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

fn hourly_rate_id(g: &SystemIdGen, school: SchoolId) -> HourlyRateId {
    HourlyRateId::new(school, g.next_uuid())
}

/// Helper: build a positive-rate HourlyRate for tests.
fn fresh_hourly_rate(tenant: &TenantContext, g: &SystemIdGen) -> HourlyRate {
    let school = tenant.school_id;
    let id = hourly_rate_id(g, school);
    let academic_id = AcademicYearId::new(school, g.next_uuid());
    HourlyRate::fresh(
        id,
        "Grade-A".to_owned(),
        150.0,
        academic_id,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
}

/// Configurable `HourlyRateUniquenessChecker` mock.
struct FakeHourlyRateUniqueness {
    exists: bool,
}
impl HourlyRateUniquenessChecker for FakeHourlyRateUniqueness {
    fn hourly_rate_exists(
        &self,
        _school: SchoolId,
        _grade: &str,
        _academic_id: AcademicYearId,
    ) -> bool {
        self.exists
    }
}

#[test]
fn hourly_rate_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = hourly_rate_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn hourly_rate_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = hourly_rate_id(&g, school);
    let id_b = hourly_rate_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =============================================================================
// Wave 189 — Spec invariant HourlyRate#1 (composite-key uniqueness)
// =============================================================================

#[test]
fn hourly_rate_ensure_unique_accepts_when_no_duplicate() {
    let (tenant, g) = admin_context();
    let hr = fresh_hourly_rate(&tenant, &g);
    let checker = FakeHourlyRateUniqueness { exists: false };
    assert!(hr.ensure_unique(&checker).is_ok());
}

#[test]
fn hourly_rate_ensure_unique_rejects_duplicate() {
    let (tenant, g) = admin_context();
    let hr = fresh_hourly_rate(&tenant, &g);
    let checker = FakeHourlyRateUniqueness { exists: true };
    let err = hr.ensure_unique(&checker).expect_err("duplicate must fail");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

// =============================================================================
// Wave 189 — Spec invariant HourlyRate#2 (rate > 0)
// =============================================================================

#[test]
fn hourly_rate_ensure_rate_positive_accepts_positive() {
    let (tenant, g) = admin_context();
    let hr = fresh_hourly_rate(&tenant, &g);
    assert_eq!(hr.rate, 150.0);
    assert!(hr.ensure_rate_positive().is_ok());
}

#[test]
fn hourly_rate_ensure_rate_positive_rejects_zero() {
    let (tenant, g) = admin_context();
    let mut hr = fresh_hourly_rate(&tenant, &g);
    hr.rate = 0.0;
    let err = hr.ensure_rate_positive().expect_err("zero rate must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn hourly_rate_ensure_rate_positive_rejects_negative() {
    let (tenant, g) = admin_context();
    let mut hr = fresh_hourly_rate(&tenant, &g);
    hr.rate = -1.0;
    let err = hr.ensure_rate_positive().expect_err("negative rate must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
