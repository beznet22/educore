//! Integration tests for the **Guardian aggregate** vertical slice (Batch 1).
//!
//! Pins the full register contract for
//! [`Guardian`](educore_academic::Guardian)
//! end-to-end through the service layer:
//!
//! 1. `register_guardian` validates the inputs (first/last
//!    name length, optional phone/email format), constructs
//!    the aggregate, and emits a [`GuardianRegistered`] event
//!    with the typed id + contact payload.
//! 2. The aggregate carries a `phone` (Option) and `email`
//!    (Option) per Guardian I-1 (at most one of each).
//!
//! The tests use the same fixture pattern as
//! `crates/domains/academic/tests/class.rs`
//! (`TestClock` + `SystemIdGen`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_academic::prelude::*;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_events::domain_event::DomainEvent;

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

fn guardian_id(g: &SystemIdGen, school: SchoolId) -> GuardianId {
    GuardianId::new(school, g.next_uuid())
}

// =============================================================================
// 1. Happy path: register a Guardian
// =============================================================================

#[test]
fn guardian_register_builds_aggregate_and_emits_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let gid = guardian_id(&g, school);
    let phone = PhoneNumber::new("+14155552671").unwrap();
    let email = EmailAddress::new("jane@example.com").unwrap();
    let cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: "Jane".to_owned(),
        last_name: "Doe".to_owned(),
        phone: Some(phone.clone()),
        email: Some(email.clone()),
    };
    let (agg, event) = register_guardian(cmd, &clock, &ids).expect("create");

    assert_eq!(agg.id, gid);
    assert_eq!(agg.school_id, school);
    assert_eq!(agg.first_name, "Jane");
    assert_eq!(agg.last_name, "Doe");
    assert_eq!(agg.phone.as_ref().map(|p| p.as_str().to_owned()), Some(phone.as_str().to_owned()));
    assert_eq!(agg.email.as_ref().map(|e| e.as_str().to_owned()), Some(email.as_str().to_owned()));
    assert_eq!(agg.full_name(), "Jane Doe");
    assert!(agg.active_status.is_active());

    assert_eq!(
        <GuardianRegistered as DomainEvent>::EVENT_TYPE,
        "academic.guardian.registered"
    );
    assert_eq!(<GuardianRegistered as DomainEvent>::AGGREGATE_TYPE, "guardian");
    assert_eq!(event.aggregate_id(), agg.id.as_uuid());
    assert_eq!(event.school_id(), school);
    assert_eq!(event.first_name, "Jane");
    assert_eq!(event.phone.as_ref().map(|p| p.as_str().to_owned()), Some(phone.as_str().to_owned()));
}

// =============================================================================
// 2. I-1: phone/email cap enforced at construction
// =============================================================================

#[test]
fn guardian_phone_number_rejects_invalid_format() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    assert!(PhoneNumber::new("").is_err());
    assert!(PhoneNumber::new("14155552671").is_err()); // missing +
    assert!(PhoneNumber::new("+abc").is_err()); // non-digit
    assert!(PhoneNumber::new("+14155552671").is_ok());
    let _ = (g, school);
}

#[test]
fn guardian_email_rejects_invalid_format() {
    assert!(EmailAddress::new("").is_err());
    assert!(EmailAddress::new("ada@example.com").is_ok());
    assert!(EmailAddress::new("no-at-sign").is_err());
    assert!(EmailAddress::new("@example.com").is_err());
    assert!(EmailAddress::new("user@").is_err());
}

// =============================================================================
// 3. Validation failure: empty first_name
// =============================================================================

#[test]
fn guardian_register_with_empty_first_name_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;
    let gid = guardian_id(&g, school);
    let cmd = RegisterGuardianCommand {
        guardian_id: gid,
        first_name: String::new(),
        last_name: "Doe".to_owned(),
        phone: None,
        email: None,
    };
    let err = register_guardian(cmd, &clock, &ids).expect_err("empty first name must fail");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected Validation, got {err:?}"
    );
}
