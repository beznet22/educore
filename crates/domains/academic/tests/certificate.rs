//! Integration tests for the **Certificate aggregate** vertical slice.
//!
//! Pins the create / update / delete contracts for the
//! `Certificate` aggregate end-to-end through the service
//! layer, exercising all 3 spec invariants:
//!
//! - I-1: layout (Portrait/Landscape) + body + footer (≤3 labels) + photo flag
//! - I-2: may have an attached file (PDF or image template)
//! - I-3: DefaultFor flag for course certificates

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::commands::{DeleteCertificateCommand, RealCreateCertificateCommand, UpdateCertificateCommand};
use educore_academic::events::{CertificateDeleted, CertificateUpdated, RealCertificateCreated};
use educore_academic::prelude::*;
use educore_academic::services::{create_certificate_aggregate, delete_certificate, update_certificate};
use educore_academic::{CertificateLayout, FileId, RealCertificate};
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

fn certificate_id(g: &SystemIdGen, school: SchoolId) -> CertificateId {
    CertificateId::new(school, g.next_uuid())
}

fn file_id(g: &SystemIdGen, school: SchoolId) -> FileId {
    FileId::new(school, g.next_uuid())
}

fn make_cmd(tenant: TenantContext, g: &SystemIdGen, school: SchoolId) -> RealCreateCertificateCommand {
    RealCreateCertificateCommand {
        tenant,
        certificate_id: certificate_id(g, school),
        name: "Transfer Certificate".to_string(),
        layout: CertificateLayout::Landscape,
        body: "This is to certify that...".to_string(),
        footer_labels: vec!["Principal".to_string(), "Date".to_string()],
        has_photo: true,
        attachment_id: Some(file_id(g, school)),
        default_for_course: false,
    }
}

// =============================================================================
// 1. Happy path
// =============================================================================

#[test]
fn certificate_create_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant, &g, school);
    let (agg, event) = create_certificate_aggregate(cmd, &clock, &ids)
        .expect("create should succeed");

    // I-1
    assert_eq!(agg.layout, CertificateLayout::Landscape);
    assert_eq!(agg.footer_labels.len(), 2);
    assert!(agg.has_photo);
    // I-2
    assert!(agg.attachment_id.is_some());
    // I-3
    assert!(!agg.default_for_course);

    assert_eq!(RealCertificateCreated::EVENT_TYPE, "academic.certificate.created");
    assert_eq!(RealCertificateCreated::AGGREGATE_TYPE, "certificate");
    assert_eq!(event.school_id(), school);
}

// =============================================================================
// 2. I-1: more than 3 footer labels rejected
// =============================================================================

#[test]
fn certificate_too_many_footer_labels_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.footer_labels = vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
        "D".to_string(), // 4th
    ];

    let err = create_certificate_aggregate(cmd, &clock, &ids)
        .expect_err("4 footer labels must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 3. I-1: empty body rejected
// =============================================================================

#[test]
fn certificate_empty_body_rejected() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.body = "   ".to_string();

    let err = create_certificate_aggregate(cmd, &clock, &ids)
        .expect_err("empty body must fail");
    assert!(matches!(err, DomainError::Validation(_)), "got {err:?}");
}

// =============================================================================
// 4. I-2: optional attachment (None) accepted
// =============================================================================

#[test]
fn certificate_without_attachment_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.attachment_id = None;

    let (agg, _event) = create_certificate_aggregate(cmd, &clock, &ids)
        .expect("no attachment is fine");
    assert!(agg.attachment_id.is_none());
}

// =============================================================================
// 5. I-3: default_for_course flag
// =============================================================================

#[test]
fn certificate_default_for_course_succeeds() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let mut cmd = make_cmd(tenant, &g, school);
    cmd.default_for_course = true;

    let (agg, _event) = create_certificate_aggregate(cmd, &clock, &ids)
        .expect("default_for_course=true");
    assert!(agg.default_for_course);
}

// =============================================================================
// 6. Update
// =============================================================================

#[test]
fn certificate_update_changes_body() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_certificate_aggregate(cmd, &clock, &ids)
        .expect("create");

    let upd = UpdateCertificateCommand {
        tenant,
        certificate_id: agg.id,
        name: Some("Course Completion Certificate".to_string()),
        layout: None,
        body: Some("Updated body text".to_string()),
        footer_labels: None,
        has_photo: None,
        attachment_id: None,
        default_for_course: Some(true),
    };
    let event = update_certificate(upd, &mut agg, &clock, &ids).expect("update");
    assert_eq!(agg.name, "Course Completion Certificate");
    assert_eq!(agg.body, "Updated body text");
    assert!(agg.default_for_course);
    let _: CertificateUpdated = event;
}

// =============================================================================
// 7. Delete
// =============================================================================

#[test]
fn certificate_delete_retires_aggregate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = make_cmd(tenant.clone(), &g, school);
    let (mut agg, _event) = create_certificate_aggregate(cmd, &clock, &ids)
        .expect("create");

    let del = DeleteCertificateCommand {
        tenant,
        certificate_id: agg.id,
    };
    let event = delete_certificate(del, &mut agg, &clock, &ids)
        .expect("delete");
    assert!(matches!(agg.active_status, ActiveStatus::Retired));
    let _: CertificateDeleted = event;
}
