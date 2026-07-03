//! Integration tests for the **IdCard aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for the `IdCard`
//! aggregate end-to-end through the service layer, exercising
//! all 2 spec invariants:
//!
//! - I-1: Boolean display flags (admission_no, name, class, photo, etc.)
//! - I-2: Layout dimensions and spacing parameters

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::commands::{DeleteIdCardCommand, RealCreateIdCardCommand, UpdateIdCardCommand};
use educore_academic::events::{IdCardDeleted, IdCardUpdated, RealIdCardCreated};
use educore_academic::prelude::*;
use educore_academic::services::{create_id_card_aggregate, delete_id_card, update_id_card};
use educore_academic::RealIdCard;
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

fn id_card_id(g: &SystemIdGen, school: SchoolId) -> IdCardId {
    IdCardId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateIdCardCommand {
    RealCreateIdCardCommand {
        tenant,
        id_card_id: id_card_id(g, school),
        name: "Student ID Card (2025)".to_string(),
        show_admission_no: true,
        show_name: true,
        show_class: true,
        show_photo: true,
        show_roll_no: true,
        show_contact: false,
        width_mm: 85,
        height_mm: 54,
        margin_mm: 3,
        spacing_mm: 2,
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn id_card_create_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_id_card_aggregate(cmd, &clock, &ids)
        .expect("create should succeed");

    // I-1: display flags
    assert!(agg.show_admission_no);
    assert!(agg.show_name);
    assert!(agg.show_class);
    assert!(agg.show_photo);
    assert!(agg.show_roll_no);
    assert!(!agg.show_contact);
    // I-2: layout
    assert_eq!(agg.width_mm, 85);
    assert_eq!(agg.height_mm, 54);
    assert_eq!(agg.margin_mm, 3);
    assert_eq!(agg.spacing_mm, 2);

    assert_eq!(RealIdCardCreated::EVENT_TYPE, "academic.id_card.created");
    assert_eq!(RealIdCardCreated::AGGREGATE_TYPE, "id_card");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-2: zero width rejected
// =============================================================================

#[test]
fn id_card_zero_width_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.width_mm = 0;

    let err = create_id_card_aggregate(cmd, &clock, &ids)
        .expect_err("zero width must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 3. I-2: zero height rejected
// =============================================================================

#[test]
fn id_card_zero_height_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.height_mm = 0;

    let err = create_id_card_aggregate(cmd, &clock, &ids)
        .expect_err("zero height must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 4. Empty name rejected
// =============================================================================

#[test]
fn id_card_empty_name_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.name = "   ".to_string();

    let err = create_id_card_aggregate(cmd, &clock, &ids)
        .expect_err("empty name must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 5. I-1: all flags false (minimal card) accepted
// =============================================================================

#[test]
fn id_card_all_flags_false_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.show_admission_no = false;
    cmd.show_name = false;
    cmd.show_class = false;
    cmd.show_photo = false;
    cmd.show_roll_no = false;
    cmd.show_contact = false;

    let (agg, _event) = create_id_card_aggregate(cmd, &clock, &ids)
        .expect("all flags false is valid (just a blank template)");
    assert!(!agg.show_admission_no);
    assert!(!agg.show_name);
}

// =============================================================================
// 6. Update
// =============================================================================

#[test]
fn id_card_update_changes_flags() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_id_card_aggregate(cmd, &clock, &ids)
        .expect("create");

    let upd = UpdateIdCardCommand {
        tenant,
        id_card_id: agg.id,
        name: None,
        show_admission_no: Some(false),
        show_name: None,
        show_class: None,
        show_photo: None,
        show_roll_no: None,
        show_contact: Some(true),
        width_mm: None,
        height_mm: None,
        margin_mm: None,
        spacing_mm: None,
    };
    let event = update_id_card(upd, &mut agg, &clock, &ids).expect("update");
    assert!(!agg.show_admission_no);
    assert!(agg.show_contact);
    let _: IdCardUpdated = event;
}

// =============================================================================
// 7. Delete
// =============================================================================

#[test]
fn id_card_delete_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_id_card_aggregate(cmd, &clock, &ids)
        .expect("create");

    let del = DeleteIdCardCommand {
        tenant,
        id_card_id: agg.id,
    };
    let event = delete_id_card(del, &mut agg, &clock, &ids).expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: IdCardDeleted = event;
}
