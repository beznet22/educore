//! Integration tests for the **RegistrationField aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for the
//! `RegistrationField` aggregate end-to-end through the service
//! layer, exercising all 3 spec invariants:
//!
//! - I-1: FieldName + LabelName + Type (Student/Staff)
//! - I-2: IsRequired, IsVisible, editability flags
//! - I-3: AdminSection for placement on form

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::commands::{
    DeleteRegistrationFieldCommand, RealCreateRegistrationFieldCommand,
    UpdateRegistrationFieldCommand,
};
use educore_academic::events::{
    RealRegistrationFieldCreated, RegistrationFieldDeleted, RegistrationFieldUpdated,
};
use educore_academic::prelude::*;
use educore_academic::services::{
    create_registration_field_aggregate, delete_registration_field, update_registration_field,
};
use educore_academic::{AdminSection, FieldName, LabelName, RealRegistrationField, RegistrationFieldType};
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

fn registration_field_id(g: &SystemIdGen, school: SchoolId) -> RegistrationFieldId {
    RegistrationFieldId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateRegistrationFieldCommand {
    RealCreateRegistrationFieldCommand {
        tenant,
        registration_field_id: registration_field_id(g, school),
        field_name: FieldName::new("birth_country").expect("valid"),
        label_name: LabelName::new("Country of Birth").expect("valid"),
        field_type: RegistrationFieldType::Student,
        is_required: true,
        is_visible: true,
        is_editable: true,
        admin_section: AdminSection::Personal,
        display_order: 1,
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn registration_field_create_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_registration_field_aggregate(cmd, &clock, &ids)
        .expect("create should succeed");

    // I-1
    assert_eq!(agg.field_name.as_str(), "birth_country");
    assert_eq!(agg.label_name.as_str(), "Country of Birth");
    assert_eq!(agg.field_type, RegistrationFieldType::Student);
    // I-2
    assert!(agg.is_required);
    assert!(agg.is_visible);
    assert!(agg.is_editable);
    // I-3
    assert_eq!(agg.admin_section, AdminSection::Personal);

    assert_eq!(RealRegistrationFieldCreated::EVENT_TYPE, "academic.registration_field.created");
    assert_eq!(RealRegistrationFieldCreated::AGGREGATE_TYPE, "registration_field");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: invalid field name rejected (constructor validation)
// =============================================================================

#[test]
fn registration_field_empty_label_name_rejected() {
    // LabelName::new rejects empty strings at the constructor level.
    let bad = LabelName::new("");
    assert!(matches!(bad, Err(DomainError::Validation(_))), "got {:?}", bad);
}

// =============================================================================
// 3. I-2: update flags
// =============================================================================

#[test]
fn registration_field_update_flags_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_registration_field_aggregate(cmd, &clock, &ids)
        .expect("create");

    let upd = UpdateRegistrationFieldCommand {
        tenant,
        registration_field_id: agg.id,
        label_name: None,
        is_required: Some(false),
        is_visible: Some(false),
        is_editable: Some(false),
        admin_section: Some(AdminSection::Other),
        display_order: None,
    };
    let event = update_registration_field(upd, &mut agg, &clock, &ids).expect("update");
    assert!(!agg.is_required);
    assert!(!agg.is_visible);
    assert!(!agg.is_editable);
    assert_eq!(agg.admin_section, AdminSection::Other);
    let _: RegistrationFieldUpdated = event;
}

// =============================================================================
// 4. I-3: admin section placement
// =============================================================================

#[test]
fn registration_field_admin_section_persisted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.admin_section = AdminSection::Documents;

    let (agg, _event) = create_registration_field_aggregate(cmd, &clock, &ids)
        .expect("create");
    assert_eq!(agg.admin_section, AdminSection::Documents);
}

// =============================================================================
// 5. Staff type
// =============================================================================

#[test]
fn registration_field_staff_type_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.field_type = RegistrationFieldType::Staff;

    let (agg, _event) = create_registration_field_aggregate(cmd, &clock, &ids)
        .expect("create");
    assert_eq!(agg.field_type, RegistrationFieldType::Staff);
}

// =============================================================================
// 6. Delete
// =============================================================================

#[test]
fn registration_field_delete_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_registration_field_aggregate(cmd, &clock, &ids)
        .expect("create");

    let del = DeleteRegistrationFieldCommand {
        tenant,
        registration_field_id: agg.id,
    };
    let event = delete_registration_field(del, &mut agg, &clock, &ids)
        .expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: RegistrationFieldDeleted = event;
}
