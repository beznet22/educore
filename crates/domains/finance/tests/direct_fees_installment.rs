//! Integration tests for the **DirectFeesInstallment aggregate** vertical slice.
//!
//! Pins the DFI I-2 + DFI I-3 + DFI I-4 invariants end-to-end:
//! - DFI I-2: amount_minor >= 0 (pinned at construction).
//! - DFI I-3: percentage_minor in [0, 100000] (pinned at construction).
//! - DFI I-4: non-overlapping windows. window_end >= window_start when
//!   both are present (companion). Cross-row non-overlap is
//!   dispatcher-enforced.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::StudentId;
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_direct_fees_installment, retire_direct_fees_installment, Currency,
    DirectFeesInstallmentCreated, DirectFeesInstallmentId, DirectFeesInstallmentRetired,
    RealDirectFeesInstallment,
};

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

fn dfi_id(g: &SystemIdGen, school: SchoolId) -> DirectFeesInstallmentId {
    DirectFeesInstallmentId::new(school, g.next_uuid())
}

fn student_id(g: &SystemIdGen, school: SchoolId) -> StudentId {
    StudentId::new(school, g.next_uuid())
}

// =========================================================================
// DFI I-2 tests (amount_minor >= 0)
// =========================================================================

#[test]
fn fresh_full_payload_amount_valid_dfi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Q1 2026 Tuition Installment".to_owned(),
        25_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        50_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-2: positive amount_minor must construct");
    assert!(agg.is_active());
    assert_eq!(agg.student_id, s_id);
    assert_eq!(agg.name, "Q1 2026 Tuition Installment");
    assert_eq!(agg.amount_minor, 25_000);
    assert_eq!(agg.percentage_minor, 50_000);
    assert!(agg.window_start.is_none());
    assert!(agg.window_end.is_none());
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_amount_boundary_valid_dfi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Free scholarship".to_owned(),
        0,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        0,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-2: zero is a valid boundary");
    assert_eq!(agg.amount_minor, 0);
    assert_eq!(agg.percentage_minor, 0);
}

#[test]
fn fresh_negative_amount_validation_error_dfi_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Negative test".to_owned(),
        -1,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        50_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("DFI I-2: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_empty_name_validation_error_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "   \t  ".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        50_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("companion: whitespace-only name must be rejected");
    assert!(
        format!("{err}").contains("name must be non-empty after trimming"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Audit footer check".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        25_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-2: positive amount_minor must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
    assert_eq!(agg.created_at, now);
    assert_eq!(agg.updated_at, now);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Will be retired".to_owned(),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        50_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-2: positive amount_minor must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Double-retire".to_owned(),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        50_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-2: positive amount_minor must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(
        format!("{err}").contains("already retired"),
        "unexpected error: {err}"
    );
}

#[test]
fn create_direct_fees_installment_service_emits_created_event_dfi_i_2() {
    use educore_finance::commands::CreateDirectFeesInstallmentCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateDirectFeesInstallmentCommand {
        tenant: tenant.clone(),
        direct_fees_installment_id: id,
        student_id: s_id,
        name: "Service integration".to_owned(),
        amount_minor: 12_000,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        percentage_minor: 50_000,
        window_start: None,
        window_end: None,
    };
    let (agg, event): (RealDirectFeesInstallment, DirectFeesInstallmentCreated) =
        create_direct_fees_installment(cmd, &clock, &ids)
            .expect("create_direct_fees_installment must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.amount_minor, 12_000);
    assert_eq!(agg.percentage_minor, 50_000);
    assert_eq!(event.direct_fees_installment_id, agg.id);
    assert_eq!(event.amount_minor, 12_000);
    assert_eq!(event.percentage_minor, 50_000);
    assert_eq!(
        <DirectFeesInstallmentCreated as DomainEvent>::EVENT_TYPE,
        "finance.direct_fees_installment.created"
    );
    assert_eq!(
        <DirectFeesInstallmentCreated as DomainEvent>::AGGREGATE_TYPE,
        "direct_fees_installment"
    );
    assert_eq!(
        <DirectFeesInstallmentCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_direct_fees_installment_service_rejects_negative_amount_dfi_i_2() {
    use educore_finance::commands::CreateDirectFeesInstallmentCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateDirectFeesInstallmentCommand {
        tenant: tenant.clone(),
        direct_fees_installment_id: id,
        student_id: s_id,
        name: "Negative service test".to_owned(),
        amount_minor: -1,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        percentage_minor: 50_000,
        window_start: None,
        window_end: None,
    };
    let err = create_direct_fees_installment(cmd, &clock, &ids)
        .expect_err("DFI I-2: negative amount_minor must be rejected at service layer");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

// =========================================================================
// DFI I-3 tests (percentage_minor in [0, 100000])
// =========================================================================

#[test]
fn fresh_percentage_minor_zero_boundary_valid_dfi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Zero percent installment".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        0,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-3: zero percentage is valid boundary");
    assert_eq!(agg.percentage_minor, 0);
}

#[test]
fn fresh_percentage_minor_one_hundred_percent_boundary_valid_dfi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Full payment installment".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        100_000,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-3: 100_000 (100%) is valid boundary");
    assert_eq!(agg.percentage_minor, 100_000);
}

#[test]
fn fresh_percentage_minor_negative_validation_error_dfi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Negative percentage".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        -1,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("DFI I-3: negative percentage must be rejected");
    assert!(
        format!("{err}").contains("percentage_minor must be in [0, 100000]"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_percentage_minor_above_100_percent_validation_error_dfi_i_3() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Above 100 percent".to_owned(),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        100_001,
        None,
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("DFI I-3: percentage > 100_000 must be rejected");
    assert!(
        format!("{err}").contains("percentage_minor must be in [0, 100000]"),
        "unexpected error: {err}"
    );
}

#[test]
fn create_direct_fees_installment_service_propagates_percentage_minor_dfi_i_3() {
    use educore_finance::commands::CreateDirectFeesInstallmentCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateDirectFeesInstallmentCommand {
        tenant: tenant.clone(),
        direct_fees_installment_id: id,
        student_id: s_id,
        name: "Service integration DFI I-3".to_owned(),
        amount_minor: 20_000,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        percentage_minor: 75_000,
        window_start: None,
        window_end: None,
    };
    let (agg, event): (RealDirectFeesInstallment, DirectFeesInstallmentCreated) =
        create_direct_fees_installment(cmd, &clock, &ids)
            .expect("create_direct_fees_installment must succeed");
    assert_eq!(agg.percentage_minor, 75_000);
    assert_eq!(event.percentage_minor, 75_000);
}

// =========================================================================
// DFI I-4 tests (non-overlapping windows: window_end >= window_start)
// =========================================================================

#[test]
fn fresh_window_start_equals_window_end_boundary_valid_dfi_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let same_day = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Single-day window".to_owned(),
        5_000,
        Currency::INR,
        same_day,
        50_000,
        Some(same_day),
        Some(same_day),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-4: window_start == window_end is valid boundary");
    assert_eq!(agg.window_start, Some(same_day));
    assert_eq!(agg.window_end, Some(same_day));
}

#[test]
fn fresh_window_end_after_window_start_valid_dfi_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Q3 window".to_owned(),
        5_000,
        Currency::INR,
        end,
        50_000,
        Some(start),
        Some(end),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-4: window_end > window_start is valid");
    assert_eq!(agg.window_start, Some(start));
    assert_eq!(agg.window_end, Some(end));
}

#[test]
fn fresh_window_end_before_window_start_validation_error_dfi_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let start = chrono::NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let err = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Inverted window".to_owned(),
        5_000,
        Currency::INR,
        end,
        50_000,
        Some(start),
        Some(end),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("DFI I-4: window_end < window_start must be rejected");
    assert!(
        format!("{err}").contains("window_end must be >= window_start"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_only_window_start_present_no_validation_dfi_i_4() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let agg = RealDirectFeesInstallment::fresh(
        id,
        s_id,
        "Open-ended window start".to_owned(),
        5_000,
        Currency::INR,
        start,
        50_000,
        Some(start),
        None,
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("DFI I-4: window_start only (no window_end) is allowed");
    assert_eq!(agg.window_start, Some(start));
    assert!(agg.window_end.is_none());
}

#[test]
fn create_direct_fees_installment_service_propagates_windows_dfi_i_4() {
    use educore_finance::commands::CreateDirectFeesInstallmentCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = dfi_id(&g, school);
    let s_id = student_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let start = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 8, 31).unwrap();
    let cmd = CreateDirectFeesInstallmentCommand {
        tenant: tenant.clone(),
        direct_fees_installment_id: id,
        student_id: s_id,
        name: "Service integration DFI I-4".to_owned(),
        amount_minor: 20_000,
        currency: Currency::INR,
        due_date: end,
        percentage_minor: 75_000,
        window_start: Some(start),
        window_end: Some(end),
    };
    let (agg, event): (RealDirectFeesInstallment, DirectFeesInstallmentCreated) =
        create_direct_fees_installment(cmd, &clock, &ids)
            .expect("create_direct_fees_installment must succeed");
    assert_eq!(agg.window_start, Some(start));
    assert_eq!(agg.window_end, Some(end));
    assert_eq!(event.window_start, Some(start));
    assert_eq!(event.window_end, Some(end));
}
