//! Behavioural tests for `RealFmFeesType` (Wave 129 full drop).
//!
//! Pins FFT I-1 (`type_kind ∈ {Fee, Discount, Fine}`) + FFT I-2
//! (`amount_minor >= 0`) + FFT I-3 companion (name non-empty after
//! trim; uniqueness dispatcher-enforced) end-to-end via the
//! aggregate surface, the service functions, and the emitted events.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::str::FromStr;

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::{Timestamp, Version};
use educore_events::domain_event::DomainEvent;
use educore_finance::events::{FmFeesTypeCreated, FmFeesTypeRetired};
use educore_finance::prelude::*;
use educore_finance::value_objects::{FmFeesTypeId, FmFeesTypeKind};

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

fn fft_id(g: &SystemIdGen, school: SchoolId) -> FmFeesTypeId {
    FmFeesTypeId::new(school, g.next_uuid())
}

// ---- typed-id smoke ----

#[test]
fn fm_fees_type_typed_id_round_trips_school() {
    let (_tenant, g) = admin_context();
    let school = g.next_school_id();
    let id = fft_id(&g, school);
    assert_eq!(id.school_id(), school);
}

// ---- FFT I-1: FmFeesTypeKind enum ----

#[test]
fn fm_fees_type_kind_as_str_round_trip() {
    assert_eq!(FmFeesTypeKind::Fee.as_str(), "fee");
    assert_eq!(FmFeesTypeKind::Discount.as_str(), "discount");
    assert_eq!(FmFeesTypeKind::Fine.as_str(), "fine");
    assert_eq!(FmFeesTypeKind::parse("fee"), Some(FmFeesTypeKind::Fee));
    assert_eq!(
        FmFeesTypeKind::parse("discount"),
        Some(FmFeesTypeKind::Discount)
    );
    assert_eq!(FmFeesTypeKind::parse("fine"), Some(FmFeesTypeKind::Fine));
    assert_eq!(FmFeesTypeKind::parse("unknown"), None);
}

#[test]
fn fm_fees_type_kind_display_matches_as_str() {
    assert_eq!(FmFeesTypeKind::Fee.to_string(), "fee");
    assert_eq!(FmFeesTypeKind::Discount.to_string(), "discount");
    assert_eq!(FmFeesTypeKind::Fine.to_string(), "fine");
}

#[test]
fn fm_fees_type_kind_from_str_known_values() {
    assert_eq!(
        FmFeesTypeKind::from_str("fee").expect("parse fee"),
        FmFeesTypeKind::Fee
    );
    assert_eq!(
        FmFeesTypeKind::from_str("discount").expect("parse discount"),
        FmFeesTypeKind::Discount
    );
    assert_eq!(
        FmFeesTypeKind::from_str("fine").expect("parse fine"),
        FmFeesTypeKind::Fine
    );
}

#[test]
fn fm_fees_type_kind_from_str_unknown_returns_err() {
    let result = FmFeesTypeKind::from_str("unknown");
    assert!(result.is_err());
}

// ---- FFT I-1 + FFT I-2 + FFT I-3 companion happy paths ----

#[test]
fn fresh_full_payload_valid_fft_i_1_fft_i_2_fft_i_3_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "Tuition Fee".to_string(),
        FmFeesTypeKind::Fee,
        10_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.type_kind, FmFeesTypeKind::Fee);
    assert_eq!(row.amount_minor, 10_000);
    assert_eq!(row.name, "Tuition Fee");
    assert_eq!(row.school_id, school);
    assert!(row.is_active());
}

#[test]
fn fresh_discount_type_kind_valid_fft_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "Sibling Discount".to_string(),
        FmFeesTypeKind::Discount,
        2_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("Discount type is valid (FFT I-1)");
    assert_eq!(row.type_kind, FmFeesTypeKind::Discount);
}

#[test]
fn fresh_fine_type_kind_valid_fft_i_1() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "Late Payment Fine".to_string(),
        FmFeesTypeKind::Fine,
        500,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("Fine type is valid (FFT I-1)");
    assert_eq!(row.type_kind, FmFeesTypeKind::Fine);
}

// ---- FFT I-2: amount_minor >= 0 ----

#[test]
fn fresh_negative_amount_validation_error_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let result = RealFmFeesType::fresh(
        id,
        "Test".to_string(),
        FmFeesTypeKind::Fee,
        -1,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("amount_minor must be >= 0") && msg.contains("FFT I-2"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_zero_amount_boundary_valid_fft_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "Test".to_string(),
        FmFeesTypeKind::Fee,
        0,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("zero amount_minor is valid boundary");
    assert_eq!(row.amount_minor, 0);
}

// ---- FFT I-3 companion: name non-empty after trim ----

#[test]
fn fresh_empty_name_validation_error_fft_i_3_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let result = RealFmFeesType::fresh(
        id,
        "   ".to_string(),
        FmFeesTypeKind::Fee,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    match result {
        Err(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("name must be non-empty after trim") && msg.contains("FFT I-3"),
                "unexpected error message: {msg}"
            );
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn fresh_name_trimmed_correctly() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "  Tuition Fee  ".to_string(),
        FmFeesTypeKind::Fee,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("name with whitespace is trimmed");
    assert_eq!(row.name, "Tuition Fee");
}

// ---- audit footer ----

#[test]
fn fresh_initializes_audit_footer() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let row = RealFmFeesType::fresh(
        id,
        "Test".to_string(),
        FmFeesTypeKind::Fee,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert_eq!(row.version, Version::initial());
    assert!(row.is_active());
    assert_eq!(row.created_by, tenant.actor_id);
    assert_eq!(row.updated_by, tenant.actor_id);
    assert_eq!(row.last_event_id, None);
}

// ---- retire ----

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let mut row = RealFmFeesType::fresh(
        id,
        "Test".to_string(),
        FmFeesTypeKind::Fee,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    assert!(row.is_active());
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("retire should succeed");
    assert!(!row.is_active());
    assert_eq!(row.amount_minor, 5_000);
    assert_eq!(row.type_kind, FmFeesTypeKind::Fee);
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fft_id(&g, school);
    let mut row = RealFmFeesType::fresh(
        id,
        "Test".to_string(),
        FmFeesTypeKind::Fee,
        5_000,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    )
    .expect("fresh should succeed");
    row.retire(Timestamp::now(), tenant.actor_id)
        .expect("first retire should succeed");
    let result = row.retire(Timestamp::now(), tenant.actor_id);
    assert!(matches!(result, Err(DomainError::Conflict(_))));
}

// ---- service integration ----

#[test]
fn create_fm_fees_type_service_emits_created_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fft_id(&g, school);
    let cmd = CreateFmFeesTypeCommand {
        tenant,
        fm_fees_type_id: id,
        name: "Tuition".to_string(),
        type_kind: FmFeesTypeKind::Fee,
        amount_minor: 10_000,
    };
    let (_agg, evt): (RealFmFeesType, FmFeesTypeCreated) =
        create_fm_fees_type(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.type_kind, FmFeesTypeKind::Fee);
    assert_eq!(evt.amount_minor, 10_000);
    assert_eq!(evt.name, "Tuition");
    assert_eq!(
        <FmFeesTypeCreated as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_type.created"
    );
    assert_eq!(
        <FmFeesTypeCreated as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_type"
    );
    assert_eq!(<FmFeesTypeCreated as DomainEvent>::SCHEMA_VERSION, 1);
}

#[test]
fn create_fm_fees_type_service_rejects_negative_amount_fft_i_2() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let id = fft_id(&g, school);
    let cmd = CreateFmFeesTypeCommand {
        tenant,
        fm_fees_type_id: id,
        name: "Test".to_string(),
        type_kind: FmFeesTypeKind::Fee,
        amount_minor: -100,
    };
    let result = create_fm_fees_type(cmd, &clock, &g);
    assert!(matches!(result, Err(DomainError::Validation(_))));
}

#[test]
fn retire_fm_fees_type_service_emits_retired_event() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = RetireFmFeesTypeCommand {
        tenant,
        fm_fees_type_id: fft_id(&g, school),
        name: "Test".to_string(),
    };
    let evt: FmFeesTypeRetired =
        retire_fm_fees_type(cmd, &clock, &g).expect("service should succeed");
    assert_eq!(evt.deleted_by, actor);
    assert_eq!(evt.name, "Test");
    assert_eq!(
        <FmFeesTypeRetired as DomainEvent>::EVENT_TYPE,
        "finance.fm_fees_type.retired"
    );
    assert_eq!(
        <FmFeesTypeRetired as DomainEvent>::AGGREGATE_TYPE,
        "fm_fees_type"
    );
}

#[test]
fn read_fm_fees_type_service_returns_ok() {
    let clock = SystemClock;
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let cmd = ReadFmFeesTypeCommand {
        tenant,
        fm_fees_type_id: fft_id(&g, school),
    };
    read_fm_fees_type(cmd, &clock, &g).expect("read should succeed");
}
