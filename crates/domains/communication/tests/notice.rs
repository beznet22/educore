//! End-to-end tests for the `create_notice` / `update_notice`
//! service factory functions.
//!
//! These tests live in their own integration file because the
//! older `tests/aggregates.rs` Cluster-C smoke file has
//! pre-existing compile errors that are out of scope for this
//! vertical slice. See `crates/domains/communication/tests/`
//! for the broken file; this file is the wave-1 vertical-slice
//! test surface for the `Notice` aggregate.
//!
//! Two scenarios:
//!
//! 1. `create_then_update_notice_round_trip` — happy path:
//!    a notice is constructed through `create_notice`, the
//!    returned event has the expected wire type, and the
//!    returned aggregate can be mutated through
//!    `update_notice`.
//! 2. `create_notice_rejects_publish_on_before_notice_date` —
//!    cross-field validation: scheduling a publish date that
//!    precedes the notice date itself is rejected with
//!    `DomainError::Validation`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use educore_communication::prelude::*;
use educore_communication::services::{create_notice, update_notice};
use educore_communication::value_objects::{
    AudienceDescriptor, NoticeBody, NoticeTitle,
};
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
use educore_core::error::DomainError;
use educore_core::ids::Identifier;

fn fresh_tenant() -> TenantContext {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    TenantContext::for_user(school, actor, corr, educore_core::tenant::UserType::SchoolAdmin)
}

fn notice_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2026, 9, 1).expect("valid notice date")
}

/// Happy-path: a notice is created, the wire event type is
/// `communication.notice.created`, and the resulting aggregate
/// can be mutated through `update_notice` (which records the
/// change set and emits a `communication.notice.updated` event).
#[test]
fn create_then_update_notice_round_trip() {
    let tenant = fresh_tenant();

    let create_cmd = CreateNoticeCommand {
        tenant: tenant.clone(),
        title: NoticeTitle::new("Holiday notice").expect("title valid"),
        body: NoticeBody::new("School closed on Monday for the holiday.")
            .expect("body valid"),
        notice_date: notice_date(),
        publish_on: Some(notice_date()),
        audience: AudienceDescriptor::All,
        attachment: None,
    };

    let (mut notice, created_event) =
        create_notice(create_cmd, &SystemClock, &SystemIdGen).expect("create_notice ok");

    // Aggregate shape: draft, active, school-scoped.
    assert_eq!(notice.school_id, tenant.school_id);
    assert_eq!(notice.status, NoticeStatus::Draft);
    assert!(notice.last_event_id.is_some());

    // Event wire type matches the spec.
    assert_eq!(
        <NoticeCreated as DomainEvent>::EVENT_TYPE,
        "communication.notice.created"
    );
    let _ = created_event;

    // Now mutate through update_notice.
    let update_cmd = UpdateNoticeCommand {
        tenant: tenant.clone(),
        notice_id: notice.id,
        title: Some(NoticeTitle::new("Holiday (revised)").expect("title valid")),
        body: None,
        publish_on: None,
        audience: Some(AudienceDescriptor::All),
    };

    let updated_event =
        update_notice(update_cmd, &SystemClock, &SystemIdGen, &mut notice).expect("update_notice ok");

    // The mutation must have been recorded on the aggregate.
    assert_eq!(notice.title.as_str(), "Holiday (revised)");
    assert_eq!(
        <NoticeUpdated as DomainEvent>::EVENT_TYPE,
        "communication.notice.updated"
    );
    let _ = updated_event;
}

/// Validation-failure: a `publish_on` that is strictly before
/// `notice_date` is rejected. The cross-field invariant lives
/// in `create_notice`; the value-object constructors do not
/// know about the date relationship.
#[test]
fn create_notice_rejects_publish_on_before_notice_date() {
    let tenant = fresh_tenant();

    let early = chrono::NaiveDate::from_ymd_opt(2026, 8, 30).expect("early date valid");
    let cmd = CreateNoticeCommand {
        tenant,
        title: NoticeTitle::new("Early").expect("title valid"),
        body: NoticeBody::new("Body").expect("body valid"),
        notice_date: notice_date(),
        publish_on: Some(early),
        audience: AudienceDescriptor::All,
        attachment: None,
    };

    let err = create_notice(cmd, &SystemClock, &SystemIdGen)
        .expect_err("create_notice should reject publish_on < notice_date");
    assert!(
        matches!(err, DomainError::Validation(_)),
        "expected DomainError::Validation, got {err:?}"
    );
}
