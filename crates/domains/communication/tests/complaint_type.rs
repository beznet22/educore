//! End-to-end tests for the `create_complaint_type`
//! service factory function.
//!
//! These tests live in their own integration file because the
//! older `tests/aggregates.rs` Cluster-C smoke file has
//! pre-existing compile errors that are out of scope for this
//! vertical slice. See `crates/domains/communication/tests/`
//! for the broken file; this file is the wave-4 vertical-slice
//! test surface for the `ComplaintType` aggregate (the 2nd
//! spec aggregate after `Notice`).
//!
//! Two scenarios:
//!
//! 1. `create_complaint_type_round_trip` — happy path:
//!    a complaint type is constructed through
//!    `create_complaint_type`, the returned event has the
//!    expected wire type, the aggregate has `version == 1`,
//!    and the event wire type is `communication.complaint_type.created`.
//! 2. `create_complaint_type_with_empty_name` —
//!    documents the current contract: the handler does not
//!    validate the name field. An empty name is accepted and
//!    produces an aggregate + event with `name == ""`. The
//!    spec invariant ("A `ComplaintType` is uniquely named
//!    within a school") is a uniqueness constraint enforced
//!    at the repository layer, not a field-level validation
//!    in the service factory.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` for
//! fixture style (`TestClock` + `SystemIdGen`).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use educore_communication::prelude::*;
use educore_communication::services::create_complaint_type;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::Identifier;
use educore_events::domain_event::DomainEvent;

fn fresh_tenant() -> TenantContext {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    TenantContext::for_user(
        school,
        actor,
        corr,
        educore_core::tenant::UserType::SchoolAdmin,
    )
}

/// Happy-path: a complaint type is created, the wire event
/// type is `communication.complaint_type.created`, the
/// aggregate has `version == 1`, and the event carries the
/// name from the command.
#[test]
fn create_complaint_type_round_trip() {
    let tenant = fresh_tenant();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = CreateComplaintTypeCommand {
        tenant: tenant.clone(),
        name: "Academics".to_owned(),
        description: Some("Academic-related complaints".to_owned()),
    };

    let (ct, created_event) =
        create_complaint_type(cmd, &clock, &ids).expect("create_complaint_type ok");

    // Aggregate shape: school-scoped, active, version=1.
    assert_eq!(ct.school_id, tenant.school_id);
    assert_eq!(ct.name, "Academics");
    assert_eq!(
        ct.description.as_deref(),
        Some("Academic-related complaints")
    );
    assert_eq!(ct.version.get(), 1);
    assert!(ct.active_status.is_active());
    assert_eq!(ct.created_by, tenant.actor_id);
    assert_eq!(ct.updated_by, tenant.actor_id);

    // Event metadata matches the DomainEvent trait's contract.
    assert_eq!(
        <ComplaintTypeCreated as DomainEvent>::EVENT_TYPE,
        "communication.complaint_type.created"
    );
    assert_eq!(
        <ComplaintTypeCreated as DomainEvent>::AGGREGATE_TYPE,
        "complaint_type"
    );
    assert_eq!(<ComplaintTypeCreated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(created_event.aggregate_id(), ct.id.as_uuid());
    assert_eq!(created_event.school_id(), tenant.school_id);
    assert_eq!(created_event.name, "Academics");
}

/// Current contract: the handler does not validate the name
/// field. An empty name is accepted; the aggregate and event
/// are produced with `name == ""`. The spec invariant
/// (uniqueness within a school) is enforced at the
/// repository layer, not as field-level validation in the
/// service factory.
#[test]
fn create_complaint_type_with_empty_name() {
    let tenant = fresh_tenant();
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = CreateComplaintTypeCommand {
        tenant: tenant.clone(),
        name: String::new(),
        description: None,
    };

    let (ct, created_event) =
        create_complaint_type(cmd, &clock, &ids).expect("create_complaint_type accepts empty name");

    // The handler does not validate; it produces the
    // aggregate and event with the empty name as-is.
    assert_eq!(ct.name, "");
    assert_eq!(created_event.name, "");
    assert_eq!(ct.version.get(), 1);
    assert_eq!(ct.school_id, tenant.school_id);
}
