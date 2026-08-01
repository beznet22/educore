//! Integration tests for the **FeesMaster aggregate** vertical slice.
//!
//! Pins the FM I-2 invariant end-to-end: a FeesMaster is uniquely
//! scoped to a (school_id, name, fees_group_id) tuple. Uniqueness
//! is dispatcher-enforced via the scope-key tuple the aggregate
//! carries as required fields. Companion invariant (FM I-1):
//! `amount_minor >= 0`.
//!
//! Replaces the prior 2 typed-id-only tests with a 12-test
//! behavioral suite.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;
use educore_finance::prelude::{
    create_fees_master, retire_fees_master, Currency, FeesGroupId, FeesMasterCreated,
    FeesMasterId, FeesMasterRetired, RealFeesMaster,
};
use educore_finance::value_objects::ClassId;

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

fn fm_id(g: &SystemIdGen, school: SchoolId) -> FeesMasterId {
    FeesMasterId::new(school, g.next_uuid())
}

fn group_id(g: &SystemIdGen, school: SchoolId) -> FeesGroupId {
    FeesGroupId::new(school, g.next_uuid())
}

fn class_id(g: &SystemIdGen, school: SchoolId) -> ClassId {
    ClassId::new(school, g.next_uuid())
}

// =========================================================================
// FM I-2 typed-id smoke
// =========================================================================

#[test]
fn fees_master_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_master_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fm_id(&g, school);
    let id_b = fm_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// FM I-2 construction tests
// =========================================================================

#[test]
fn fresh_full_payload_carries_scope_key_tuple_fm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let grp = group_id(&g, school);
    let cls = class_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesMaster::fresh(
        id,
        "Q1 2026 Tuition".to_owned(),
        grp,
        cls,
        50_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: scope-key tuple + valid amount must construct");
    assert!(agg.is_active());
    assert_eq!(agg.name, "Q1 2026 Tuition");
    assert_eq!(agg.fees_group_id, grp);
    assert_eq!(agg.class_id, cls);
    assert_eq!(agg.amount_minor, 50_000);
    assert_eq!(agg.currency, Currency::INR);
    assert_eq!(agg.school_id, school);
}

#[test]
fn fresh_zero_amount_boundary_valid_fm_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesMaster::fresh(
        id,
        "Free scholarship".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        0,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-1 companion: zero amount is a valid boundary");
    assert_eq!(agg.amount_minor, 0);
}

#[test]
fn fresh_negative_amount_validation_error_fm_i_1_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesMaster::fresh(
        id,
        "Negative test".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        -1,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect_err("FM I-1 companion: negative amount_minor must be rejected");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn fresh_empty_name_validation_error_companion() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let err = RealFeesMaster::fresh(
        id,
        "   \t  ".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
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
fn fresh_distinct_scope_key_tuples_within_same_school_fm_i_2() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let now = educore_core::value_objects::Timestamp::now();
    let agg_a = RealFeesMaster::fresh(
        fm_id(&g, school),
        "Master A".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: distinct scope-key tuple must construct");
    let agg_b = RealFeesMaster::fresh(
        fm_id(&g, school),
        "Master B".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        5_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: distinct scope-key tuple must construct");
    assert_ne!(agg_a.name, agg_b.name);
    assert_ne!(agg_a.fees_group_id, agg_b.fees_group_id);
    assert_eq!(agg_a.school_id, agg_b.school_id);
}

#[test]
fn fresh_initializes_audit_footer_with_no_last_event_id() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let agg = RealFeesMaster::fresh(
        id,
        "Audit footer check".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        2_500,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: scope-key tuple must construct");
    assert!(agg.last_event_id.is_none());
    assert_eq!(agg.created_by, tenant.actor_id);
    assert_eq!(agg.updated_by, tenant.actor_id);
}

#[test]
fn retire_flips_active_status_to_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesMaster::fresh(
        id,
        "Will be retired".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: scope-key tuple must construct");
    assert!(agg.is_active());
    agg.retire(now, tenant.actor_id).expect("retire");
    assert!(!agg.is_active());
}

#[test]
fn retire_already_retired_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let now = educore_core::value_objects::Timestamp::now();
    let mut agg = RealFeesMaster::fresh(
        id,
        "Double-retire".to_owned(),
        group_id(&g, school),
        class_id(&g, school),
        1_000,
        Currency::INR,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        tenant.actor_id,
        now,
        tenant.correlation_id,
    )
    .expect("FM I-2: scope-key tuple must construct");
    agg.retire(now, tenant.actor_id).expect("first retire");
    let err = agg
        .retire(now, tenant.actor_id)
        .expect_err("double-retire must conflict");
    assert!(format!("{err}").contains("already retired"), "unexpected error: {err}");
}

// =========================================================================
// FM I-2 service integration tests
// =========================================================================

#[test]
fn create_fees_master_service_emits_created_event_fm_i_2() {
    use educore_finance::commands::CreateFeesMasterCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let grp = group_id(&g, school);
    let cls = class_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesMasterCommand {
        tenant: tenant.clone(),
        fees_master_id: id,
        fees_group_id: grp,
        class_id: cls,
        amount_minor: 25_000,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        name: "Service integration".to_owned(),
    };
    let (agg, event): (RealFeesMaster, FeesMasterCreated) =
        create_fees_master(cmd, &clock, &ids).expect("create_fees_master must succeed");
    assert!(agg.is_active());
    assert_eq!(agg.amount_minor, 25_000);
    assert_eq!(agg.name, "Service integration");
    assert_eq!(event.fees_master_id, agg.id);
    assert_eq!(event.amount_minor, 25_000);
    assert_eq!(
        <FeesMasterCreated as DomainEvent>::EVENT_TYPE,
        "finance.fees_master.created"
    );
    assert_eq!(
        <FeesMasterCreated as DomainEvent>::AGGREGATE_TYPE,
        "fees_master"
    );
    assert_eq!(
        <FeesMasterCreated as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}

#[test]
fn create_fees_master_service_rejects_negative_amount_fm_i_1() {
    use educore_finance::commands::CreateFeesMasterCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = CreateFeesMasterCommand {
        tenant: tenant.clone(),
        fees_master_id: id,
        fees_group_id: group_id(&g, school),
        class_id: class_id(&g, school),
        amount_minor: -100,
        currency: Currency::INR,
        due_date: chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        name: "Negative service test".to_owned(),
    };
    let err = create_fees_master(cmd, &clock, &ids)
        .expect_err("FM I-1: negative amount_minor must be rejected at service layer");
    assert!(
        format!("{err}").contains("amount_minor must be >= 0"),
        "unexpected error: {err}"
    );
}

#[test]
fn retire_fees_master_service_emits_retired_event_fm() {
    use educore_finance::commands::RetireFeesMasterCommand;
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fm_id(&g, school);
    let clock = SystemClock;
    let ids = SystemIdGen;
    let cmd = RetireFeesMasterCommand {
        tenant: tenant.clone(),
        fees_master_id: id,
    };
    let (agg, event): (RealFeesMaster, FeesMasterRetired) =
        retire_fees_master(cmd, &clock, &ids).expect("retire_fees_master must succeed");
    assert!(!agg.is_active());
    assert_eq!(event.fees_master_id, agg.id);
    assert_eq!(event.retired_by, tenant.actor_id);
    assert_eq!(
        <FeesMasterRetired as DomainEvent>::EVENT_TYPE,
        "finance.fees_master.retired"
    );
    assert_eq!(
        <FeesMasterRetired as DomainEvent>::AGGREGATE_TYPE,
        "fees_master"
    );
    assert_eq!(
        <FeesMasterRetired as DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
}
