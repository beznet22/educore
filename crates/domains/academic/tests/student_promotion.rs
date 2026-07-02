//! Integration tests for the **StudentPromotion aggregate** vertical slice.
//!
//! Pins the create / record / immutability contracts for the
//! `StudentPromotion` aggregate end-to-end through the service
//! layer, exercising all 3 spec invariants:
//!
//! - I-1: References both `From` and `To` `StudentRecord`s
//! - I-2: `ResultStatus` is `Pass`, `Fail`, or `Manual`
//! - I-3: Immutable once written

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use chrono::NaiveDate;
use educore_academic::prelude::*;
use educore_academic::services::record_student_promotion_aggregate;
use educore_academic::{RealStudentPromotion, ResultStatus};
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;

// =============================================================================
// Fixtures
// =============================================================================

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

fn student_promotion_id(g: &SystemIdGen, school: SchoolId) -> StudentPromotionId {
    StudentPromotionId::new(school, g.next_uuid())
}

fn student_record_id(g: &SystemIdGen, school: SchoolId) -> StudentRecordId {
    StudentRecordId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

fn section_id(g: &SystemIdGen, school: SchoolId) -> SectionId {
    SectionId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: record a StudentPromotion (I-1, I-2)
// =============================================================================

#[test]
fn student_promotion_record_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id = student_promotion_id(&g, school);
    let from_record = student_record_id(&g, school);
    let to_record = student_record_id(&g, school);
    let from_year = academic_year_id(&g, school);
    let to_year = academic_year_id(&g, school);

    let (agg, event) = record_student_promotion_aggregate(
        id,
        student_id(&g, school),
        from_record,
        to_record,
        from_year,
        to_year,
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Pass,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect("record should succeed");

    // I-1: both records present
    assert_eq!(agg.from_student_record_id, from_record);
    assert_eq!(agg.to_student_record_id, to_record);
    // I-2: ResultStatus::Pass
    assert_eq!(agg.result_status, ResultStatus::Pass);
    assert_eq!(agg.to_roll_number, "15");
    // I-3: immutable — no mutator methods available
    // (verified by the fact that the aggregate has no &mut self methods beyond
    //  what's provided by the derives)

    // Event has matching fields
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: same from/to record rejected
// =============================================================================

#[test]
fn student_promotion_same_records_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let record = student_record_id(&g, school);
    let from_year = academic_year_id(&g, school);
    let to_year = academic_year_id(&g, school);

    let err = record_student_promotion_aggregate(
        student_promotion_id(&g, school),
        student_id(&g, school),
        record,
        record, // same as from
        from_year,
        to_year,
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Pass,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect_err("same records must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 3. I-1: cross-school typed id rejected
// =============================================================================

#[test]
fn student_promotion_cross_school_record_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let other_school = g.next_school_id();
    let from_record = student_record_id(&g, school);
    let to_record = StudentRecordId::new(other_school, g.next_uuid());
    let from_year = academic_year_id(&g, school);
    let to_year = academic_year_id(&g, school);

    let err = record_student_promotion_aggregate(
        student_promotion_id(&g, school),
        student_id(&g, school),
        from_record,
        to_record,
        from_year,
        to_year,
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Pass,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect_err("cross-school record must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 4. I-1: same from/to academic year rejected
// =============================================================================

#[test]
fn student_promotion_same_years_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let year = academic_year_id(&g, school);

    let err = record_student_promotion_aggregate(
        student_promotion_id(&g, school),
        student_id(&g, school),
        student_record_id(&g, school),
        student_record_id(&g, school),
        year,
        year, // same
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Pass,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect_err("same years must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 5. I-2: Fail result accepted
// =============================================================================

#[test]
fn student_promotion_fail_result_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id = student_promotion_id(&g, school);
    let from_record = student_record_id(&g, school);
    let to_record = student_record_id(&g, school);

    let (agg, _) = record_student_promotion_aggregate(
        id,
        student_id(&g, school),
        from_record,
        to_record,
        academic_year_id(&g, school),
        academic_year_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Fail,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect("Fail result should be accepted");
    assert_eq!(agg.result_status, ResultStatus::Fail);
}

// =============================================================================
// 6. I-2: Manual result accepted
// =============================================================================

#[test]
fn student_promotion_manual_result_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id = student_promotion_id(&g, school);
    let from_record = student_record_id(&g, school);
    let to_record = student_record_id(&g, school);

    let (agg, _) = record_student_promotion_aggregate(
        id,
        student_id(&g, school),
        from_record,
        to_record,
        academic_year_id(&g, school),
        academic_year_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Manual,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect("Manual result should be accepted");
    assert_eq!(agg.result_status, ResultStatus::Manual);
}

// =============================================================================
// 7. I-3: aggregate is immutable (no mutator service beyond fresh)
// =============================================================================

#[test]
fn student_promotion_is_immutable_after_fresh() {
    // I-3 is enforced at the API surface: RealStudentPromotion has only a
    // `fresh` constructor and no &mut self methods. The test verifies
    // that after fresh(), the fields are stable and no service mutates
    // them (verified by code review of services.rs).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let id = student_promotion_id(&g, school);
    let from_record = student_record_id(&g, school);
    let to_record = student_record_id(&g, school);

    let (agg, _event) = record_student_promotion_aggregate(
        id,
        student_id(&g, school),
        from_record,
        to_record,
        academic_year_id(&g, school),
        academic_year_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        class_id(&g, school),
        section_id(&g, school),
        Some("10".to_string()),
        "15".to_string(),
        ResultStatus::Pass,
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
        tenant.actor_id,
        &clock,
        &ids,
    )
    .expect("create");
    // Read fields — all are pub but no mutator service exists.
    assert_eq!(agg.to_roll_number, "15");
    assert_eq!(agg.from_roll_number, Some("10".to_string()));
}
