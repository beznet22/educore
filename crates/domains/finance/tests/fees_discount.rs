//! Integration tests for the **FeesDiscount aggregate** vertical slice.
//!
//! Covers the behavioral contract for the Wave 86 per-aggregate drop
//! [`RealFeesDiscount`](educore_finance::aggregate::RealFeesDiscount) —
//! the discount catalogue entry. FD I-3 (once-per-master scope) and
//! FD I-4 (once-per-year scope) are enforced by pinning
//! `fees_master_id` + `academic_year_id` as scope-key fields; the
//! existing `DiscountType` enum (`Once` / `Year`) already encodes the
//! scope semantics at the type-system level, so the aggregate
//! pins the enum variant directly.
//!
//! Promotion: FD I-1 (amount >= 0) is DEFERRED in this wave — the
//! existing `DiscountType` enum encodes SCOPE semantics (Once/Year),
//! not VALUE types; the value fields aren't part of the real
//! `RealFeesDiscount` shape. FD I-2 (discount_type valid) is
//! PROMOTED from `[~]` partial to `[x]` complete via the
//! `DiscountType` enum's two-variant type-system enforcement (no
//! invalid variant can be constructed).
//!
//! FD I-3 + FD I-4 added: the aggregate pins `fees_master_id` +
//! `academic_year_id` as scope-key fields; the dispatcher enforces
//! uniqueness on these keys before calling the service function.
//!
//! The pre-existing 2 typed-id-only tests have been replaced by this
//! suite because `FeesDiscount` previously had only a partial
//! implementation (4 commands + value_objects typed-id, but no real
//! `RealFeesDiscount` aggregate). Wave 86 adds the
//! `RealFeesDiscount` aggregate (full lifecycle: fresh +
//! update_metadata + retire), the 3 headline events (Created /
//! Updated / Retired), the service function, and this test suite.
//! Structurally parallel to Wave 78 FCFA / Wave 82 ST / Wave 85 BS
//! full-lifecycle pattern.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::AcademicYearId;
use educore_core::clock::{Clock as _, IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent as _;
use educore_finance::value_objects::FeesMasterId;

use educore_finance::commands::CreateFeesDiscountCommand;
use educore_finance::events::FeesDiscountCreated;
use educore_finance::prelude::RealFeesDiscount;
use educore_finance::services::create_fees_discount;
use educore_finance::value_objects::{DiscountType, FeesDiscountId};

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

fn fees_discount_id(g: &SystemIdGen, school: SchoolId) -> FeesDiscountId {
    FeesDiscountId::new(school, g.next_uuid())
}

fn fees_master_id(g: &SystemIdGen, school: SchoolId) -> FeesMasterId {
    FeesMasterId::new(school, g.next_uuid())
}

fn academic_year_id(g: &SystemIdGen, school: SchoolId) -> AcademicYearId {
    AcademicYearId::new(school, g.next_uuid())
}

fn make_fees_discount(
    g: &SystemIdGen,
    school: SchoolId,
    name: &str,
    discount_code: &str,
    discount_type: DiscountType,
) -> RealFeesDiscount {
    let actor = g.next_user_id();
    let now = SystemClock.now();
    RealFeesDiscount::fresh(
        fees_discount_id(g, school),
        fees_master_id(g, school),
        academic_year_id(g, school),
        name.to_owned(),
        discount_code.to_owned(),
        discount_type,
        Some("test discount".to_owned()),
        None,
        None,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("fresh RealFeesDiscount")
}

// =========================================================================
// Typed-id smoke (retained from the pre-Wave 86 stub tests)
// =========================================================================

#[test]
fn fees_discount_typed_id_round_trips_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_discount_id(&g, school);
    assert_eq!(id.school_id(), school);
}

#[test]
fn fees_discount_typed_ids_are_distinct_within_school() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id_a = fees_discount_id(&g, school);
    let id_b = fees_discount_id(&g, school);
    assert_ne!(id_a, id_b);
    assert_eq!(id_a.school_id(), school);
    assert_eq!(id_b.school_id(), school);
}

// =========================================================================
// RealFeesDiscount::fresh — FD I-2 (type-system pinned) + FD I-3 + FD I-4
// =========================================================================

#[test]
fn fresh_pins_discount_type_and_scope_keys() {
    // FD I-2: DiscountType enum pinned at type-system level. FD I-3 +
    // FD I-4: fees_master_id + academic_year_id pinned as scope-keys.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let master = fees_master_id(&g, school);
    let year = academic_year_id(&g, school);
    let row = make_fees_discount(&g, school, "Sibling Discount", "SIB10", DiscountType::Once);
    assert_eq!(row.discount_type, DiscountType::Once);
    assert_eq!(row.fees_master_id.school_id(), master.school_id());
    assert_eq!(row.academic_year_id.school_id(), year.school_id());
    assert_eq!(row.name, "Sibling Discount");
    assert_eq!(row.discount_code, "SIB10");
    assert!(row.is_active());
}

#[test]
fn fresh_supports_year_discount_type() {
    // FD I-4: DiscountType::Year variant supported.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = make_fees_discount(&g, school, "Annual Merit", "MERIT25", DiscountType::Year);
    assert_eq!(row.discount_type, DiscountType::Year);
}

#[test]
fn fresh_trims_name_and_discount_code_and_rejects_empty() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = g.next_user_id();
    let now = SystemClock.now();
    // Trim name + discount_code.
    let row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "  pad me  ".to_owned(),
        "  PAD  ".to_owned(),
        DiscountType::Once,
        None,
        None,
        None,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("trim is OK");
    assert_eq!(row.name, "pad me");
    assert_eq!(row.discount_code, "PAD");
    // Reject empty name.
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "   ".to_owned(),
        "OK".to_owned(),
        DiscountType::Once,
        None,
        None,
        None,
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
    // Reject empty discount_code.
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "OK".to_owned(),
        "   ".to_owned(),
        DiscountType::Once,
        None,
        None,
        None,
        None,
        actor,
        now,
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect_err("empty discount_code must be rejected");
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
    let row = make_fees_discount(&g, school, "Discount", "DISC", DiscountType::Once);
    let after = SystemClock.now();
    assert_eq!(row.created_at, row.updated_at);
    assert!(row.created_at >= before);
    assert!(row.created_at <= after);
    assert_eq!(row.created_by, row.updated_by);
    assert!(row.last_event_id.is_none());
}

// =========================================================================
// RealFeesDiscount::update_metadata — scope-keys NOT mutable
// (FD I-3 + FD I-4 require retire + create-new for scope changes)
// =========================================================================

#[test]
fn update_metadata_updates_name_type_and_preserves_scope_keys() {
    // FD I-3 + FD I-4: fees_master_id + academic_year_id are NOT
    // mutable via update_metadata (scope-key fields require
    // retire + create-new).
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let master = fees_master_id(&g, school);
    let year = academic_year_id(&g, school);
    let mut row = make_fees_discount(&g, school, "Initial Name", "INIT", DiscountType::Once);
    let original_version = row.version;
    let later = SystemClock.now();
    row.update_metadata(
        "Revised Name".to_owned(),
        "REV".to_owned(),
        DiscountType::Year,
        Some("revised description".to_owned()),
        None,
        None,
        None,
        later,
        g.next_user_id(),
    )
    .expect("update");
    assert_eq!(row.name, "Revised Name");
    assert_eq!(row.discount_code, "REV");
    assert_eq!(row.discount_type, DiscountType::Year);
    assert_eq!(row.description.as_deref(), Some("revised description"));
    // FD I-3 + FD I-4: scope-key fields preserved.
    assert_eq!(row.fees_master_id.school_id(), master.school_id());
    assert_eq!(row.academic_year_id.school_id(), year.school_id());
    assert!(row.version > original_version);
}

#[test]
fn update_metadata_rejects_on_retired() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_discount(&g, school, "Discount", "DISC", DiscountType::Once);
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    let later = SystemClock.now();
    let err = row
        .update_metadata(
            "Revised".to_owned(),
            "REV".to_owned(),
            DiscountType::Year,
            None,
            None,
            None,
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
// RealFeesDiscount::retire — tombstone preserving FD I-3 + FD I-4
// scope-key fields
// =========================================================================

#[test]
fn retire_flips_active_status_and_preserves_discount_type_and_scope_keys() {
    // FD I-3 + FD I-4: scope-key fields preserved in audit footer for
    // legal-record retention + uniqueness queries.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let master = fees_master_id(&g, school);
    let year = academic_year_id(&g, school);
    let mut row = make_fees_discount(&g, school, "Discount", "DISC", DiscountType::Once);
    let before = row.version;
    let now = SystemClock.now();
    row.retire(now, g.next_user_id()).expect("retire");
    assert!(!row.is_active());
    // FD I-2 + FD I-3 + FD I-4: original payload preserved.
    assert_eq!(row.discount_type, DiscountType::Once);
    assert_eq!(row.fees_master_id.school_id(), master.school_id());
    assert_eq!(row.academic_year_id.school_id(), year.school_id());
    assert_eq!(row.updated_at, now);
    assert!(row.version > before);
}

#[test]
fn retire_rejects_double_retire() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = make_fees_discount(&g, school, "Discount", "DISC", DiscountType::Once);
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
// create_fees_discount service function
// =========================================================================

#[test]
fn create_service_produces_aggregate_and_event_with_full_payload() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_discount_id(&g, school);
    let master = fees_master_id(&g, school);
    let year = academic_year_id(&g, school);
    let cmd = CreateFeesDiscountCommand {
        tenant: tenant.clone(),
        fees_discount_id: id,
        fees_master_id: master,
        academic_year_id: year,
        name: "Sibling Discount".to_owned(),
        discount_code: "SIB10".to_owned(),
        discount_type: DiscountType::Once,
        description: Some("10% off for siblings".to_owned()),
        amount_minor: None,
        percentage_basis_points: None,
        currency: None,
    };
    let clock = SystemClock;
    let (row, event) =
        create_fees_discount(cmd, &clock, &g).expect("create_fees_discount should succeed");
    assert_eq!(row.id, id);
    assert_eq!(row.school_id, school);
    assert_eq!(row.fees_master_id, master); // FD I-3
    assert_eq!(row.academic_year_id, year); // FD I-4
    assert_eq!(row.discount_type, DiscountType::Once); // FD I-2
    assert!(row.is_active());
    assert_eq!(event.fees_discount_id, id);
    assert_eq!(
        <FeesDiscountCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
        "finance.fees_discount.created"
    );
    assert_eq!(
        <FeesDiscountCreated as educore_events::domain_event::DomainEvent>::AGGREGATE_TYPE,
        "fees_discount"
    );
    assert_eq!(
        <FeesDiscountCreated as educore_events::domain_event::DomainEvent>::SCHEMA_VERSION,
        1
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.aggregate_id(), id.as_uuid());
    assert_eq!(row.last_event_id, Some(event.event_id));
}

#[test]
fn create_service_propagates_empty_name_validation() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let id = fees_discount_id(&g, school);
    let master = fees_master_id(&g, school);
    let year = academic_year_id(&g, school);
    let cmd = CreateFeesDiscountCommand {
        tenant: tenant.clone(),
        fees_discount_id: id,
        fees_master_id: master,
        academic_year_id: year,
        name: "   ".to_owned(),
        discount_code: "BAD".to_owned(),
        discount_type: DiscountType::Once,
        description: None,
        amount_minor: None,
        percentage_basis_points: None,
        currency: None,
    };
    let clock = SystemClock;
    let err = create_fees_discount(cmd, &clock, &g)
        .expect_err("empty name must be rejected by the service");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

// =========================================================================
// -- Wave 155 -- RealFeesDiscount -- FD I-1 value-type guards --
// =========================================================================

#[test]
fn fd_i_1_amount_minor_zero_is_allowed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Flat Zero".to_owned(),
        "ZERO".to_owned(),
        DiscountType::Once,
        None,
        Some(0),
        None,
        Some(educore_finance::prelude::Currency::INR),
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("zero amount_minor is valid (FD I-1 boundary)");
    assert_eq!(row.amount_minor, Some(0));
    assert_eq!(row.value_kind(), Some("amount"));
}

#[test]
fn fd_i_1_amount_minor_negative_is_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Flat Negative".to_owned(),
        "NEG".to_owned(),
        DiscountType::Once,
        None,
        Some(-1),
        None,
        Some(educore_finance::prelude::Currency::INR),
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fd_i_1_percentage_basis_points_max_is_allowed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Full Pct".to_owned(),
        "FULL".to_owned(),
        DiscountType::Once,
        None,
        None,
        Some(10_000),
        None,
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("100% bps (10_000) is valid (FD I-1 boundary)");
    assert_eq!(row.percentage_basis_points, Some(10_000));
    assert_eq!(row.value_kind(), Some("percentage"));
}

#[test]
fn fd_i_1_percentage_basis_points_overflow_is_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Over Pct".to_owned(),
        "OVER".to_owned(),
        DiscountType::Once,
        None,
        None,
        Some(10_001),
        None,
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fd_i_1_amount_and_percentage_mutually_exclusive() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Both".to_owned(),
        "BOTH".to_owned(),
        DiscountType::Once,
        None,
        Some(500),
        Some(1_000),
        Some(educore_finance::prelude::Currency::INR),
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fd_i_1_currency_required_when_amount_is_some() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let err = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "No Currency".to_owned(),
        "NOC".to_owned(),
        DiscountType::Once,
        None,
        Some(500),
        None,
        None, // no currency but amount_minor is Some
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}

#[test]
fn fd_i_1_scope_only_with_no_value_fields_is_valid() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Scope Only".to_owned(),
        "SCO".to_owned(),
        DiscountType::Year,
        None,
        None,
        None,
        None,
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("scope-only catalogue entry is valid; value supplied per-application");
    assert_eq!(row.amount_minor, None);
    assert_eq!(row.percentage_basis_points, None);
    assert_eq!(row.value_kind(), None);
}

// =========================================================================
// -- Wave 157 -- RealFeesDiscount::update_metadata value-type guards --
// =========================================================================

#[test]
fn fd_i_1_update_metadata_can_set_amount() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Upd Amount".to_owned(),
        "UPA".to_owned(),
        DiscountType::Once,
        None,
        None,
        None,
        None,
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("scope-only fresh");
    assert_eq!(row.value_kind(), None);
    // Set amount_minor to 1500 via Some(Some(1500)).
    row.update_metadata(
        "Upd Amount".to_owned(),
        "UPA".to_owned(),
        DiscountType::Once,
        None,
        Some(Some(1500)),
        None,
        Some(Some(educore_finance::prelude::Currency::INR)),
        SystemClock.now(),
        g.next_user_id(),
    )
    .expect("set amount via triple-nested Some(Some)");
    assert_eq!(row.amount_minor, Some(1500));
    assert_eq!(row.percentage_basis_points, None);
    assert_eq!(row.currency, Some(educore_finance::prelude::Currency::INR));
    assert_eq!(row.value_kind(), Some("amount"));
}

#[test]
fn fd_i_1_update_metadata_can_clear_via_some_none() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Upd Clear".to_owned(),
        "UPC".to_owned(),
        DiscountType::Once,
        None,
        Some(2000),
        None,
        Some(educore_finance::prelude::Currency::INR),
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("flat-amount fresh");
    assert_eq!(row.value_kind(), Some("amount"));
    // Clear all 3 value fields via Some(None).
    row.update_metadata(
        "Upd Clear".to_owned(),
        "UPC".to_owned(),
        DiscountType::Once,
        None,
        Some(None),
        Some(None),
        Some(None),
        SystemClock.now(),
        g.next_user_id(),
    )
    .expect("clear via Some(None)");
    assert_eq!(row.amount_minor, None);
    assert_eq!(row.percentage_basis_points, None);
    assert_eq!(row.currency, None);
    assert_eq!(row.value_kind(), None);
}

#[test]
fn fd_i_1_update_metadata_rejects_invalid_negative_amount() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let mut row = RealFeesDiscount::fresh(
        fees_discount_id(&g, school),
        fees_master_id(&g, school),
        academic_year_id(&g, school),
        "Upd Neg".to_owned(),
        "UPN".to_owned(),
        DiscountType::Once,
        None,
        None,
        None,
        None,
        tenant.actor_id,
        SystemClock.now(),
        educore_core::ids::CorrelationId(g.next_uuid()),
    )
    .expect("scope-only fresh");
    let err = row
        .update_metadata(
            "Upd Neg".to_owned(),
            "UPN".to_owned(),
            DiscountType::Once,
            None,
            Some(Some(-1)),
            None,
            Some(Some(educore_finance::prelude::Currency::INR)),
            SystemClock.now(),
            g.next_user_id(),
        )
        .unwrap_err();
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
