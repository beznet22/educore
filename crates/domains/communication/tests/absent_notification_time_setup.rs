//! Integration tests for the **AbsentNotificationTimeSetup aggregate**
//! vertical slice.
//!
//! Pins the create + update contract for
//! [`AbsentNotificationTimeSetup`](educore_communication::aggregate::AbsentNotificationTimeSetup)
//! end-to-end through the service layer:
//!
//! 1. `configure_absent_notification` constructs the aggregate in the
//!    initial `Disabled` state with the supplied time window, derives
//!    the typed id from the tenant's school id, and emits an
//!    [`AbsentNotificationScheduled`] event.
//! 2. The enable + disable round-trip mutates the aggregate in place
//!    (transitions `status`, bumps `version`, updates the audit
//!    footer) and emits [`AbsentNotificationEnabled`] /
//!    [`AbsentNotificationDisabled`] events whose wire types match
//!    the spec.
//!
//! The pre-existing `tests/aggregates.rs` Cluster-C smoke file pins
//! the in-place `enable` / `disable` methods on the bare aggregate;
//! this file is the wave-23 vertical slice that pins the
//! **service-layer** contract (create + update through the factory
//! functions), which is the seam the dispatcher will eventually wrap.
//!
//! Mirrors `crates/domains/library/tests/aggregates.rs` and
//! `crates/domains/communication/tests/notice.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_communication::prelude::*;
use educore_communication::services::{
    configure_absent_notification, disable_absent_notification, enable_absent_notification,
};
use educore_communication::value_objects::TimeOfDay;
use educore_core::clock::{SystemIdGen, TestClock};
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school. The `SystemIdGen` is returned alongside
/// so tests can mint child ids (e.g. `next_uuid`) from the same
/// generator sequence.
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

/// `08:00` window start — valid `HH:MM` 24-hour form.
fn time_from() -> TimeOfDay {
    TimeOfDay::new("08:00").expect("from-time valid")
}

/// `10:00` window end — valid `HH:MM` 24-hour form.
fn time_to() -> TimeOfDay {
    TimeOfDay::new("10:00").expect("to-time valid")
}

// =============================================================================
// Happy path: create schedules a Disabled AbsentNotificationTimeSetup
// =============================================================================

/// End-to-end happy path for the create flow. The service function
/// must:
///
/// 1. Build a fresh `AbsentNotificationTimeSetup` aggregate whose
///    `school_id` is derived from the typed id, whose `time_from`
///    / `time_to` mirror the command, and whose `status` is the
///    initial `Disabled` value.
/// 2. Stamp the audit footer (version 1, active, `last_event_id`
///    pointing at the event id minted by the service).
/// 3. Emit an `AbsentNotificationScheduled` event whose wire type
///    is `communication.absent_notification.scheduled` and whose
///    `aggregate_id` / `school_id` match the aggregate.
#[test]
fn absent_notification_time_setup_create() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    let cmd = ConfigureAbsentNotificationCommand {
        tenant: tenant.clone(),
        time_from: time_from(),
        time_to: time_to(),
    };

    let (setup, scheduled_event) =
        configure_absent_notification(cmd, &clock, &ids).expect("configure");

    // Aggregate fields mirror the command.
    assert_eq!(setup.school_id, school);
    assert_eq!(setup.time_from.as_str(), "08:00");
    assert_eq!(setup.time_to.as_str(), "10:00");
    assert_eq!(setup.id.school_id(), school);
    // Fresh aggregate is Disabled by construction.
    assert_eq!(setup.status, AbsentNotificationStatus::Disabled);
    // Audit footer is initialised.
    assert_eq!(setup.version.get(), 1);
    assert!(setup.active_status.is_active());
    assert_eq!(setup.created_by, tenant.actor_id);
    assert_eq!(setup.updated_by, tenant.actor_id);
    assert!(setup.last_event_id.is_some());

    // Event wire type matches the spec.
    assert_eq!(
        <AbsentNotificationScheduled as DomainEvent>::EVENT_TYPE,
        "communication.absent_notification.scheduled"
    );
    assert_eq!(
        <AbsentNotificationScheduled as DomainEvent>::AGGREGATE_TYPE,
        "absent_notification_time_setup"
    );
    assert_eq!(<AbsentNotificationScheduled as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(scheduled_event.aggregate_id(), setup.id.as_uuid());
    assert_eq!(scheduled_event.school_id(), school);
    assert_eq!(scheduled_event.time_from.as_str(), "08:00");
    assert_eq!(scheduled_event.time_to.as_str(), "10:00");
}

// =============================================================================
// Happy path: enable then disable cycles status through the service
// =============================================================================

/// End-to-end happy path for the update lifecycle. Starting from the
/// `Disabled` aggregate produced by `configure_absent_notification`,
/// the service functions must:
///
/// 1. `enable_absent_notification` flips `status` to `Enabled`,
///    bumps `version` to 2, and emits an
///    `AbsentNotificationEnabled` event.
/// 2. `disable_absent_notification` flips `status` back to
///    `Disabled`, bumps `version` to 3, and emits an
///    `AbsentNotificationDisabled` event.
///
/// Throughout, the audit footer (`updated_at` / `updated_by`) is
/// maintained by the service; the wire event types match the
/// `communication.absent_notification.{enabled,disabled}` constants
/// declared on `DomainEvent`.
#[test]
fn absent_notification_time_setup_update() {
    let (tenant, _g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let ids = SystemIdGen;

    // Seed via the create flow.
    let cmd = ConfigureAbsentNotificationCommand {
        tenant: tenant.clone(),
        time_from: time_from(),
        time_to: time_to(),
    };
    let (mut setup, _scheduled_event) =
        configure_absent_notification(cmd, &clock, &ids).expect("configure");
    assert_eq!(setup.version.get(), 1);
    assert_eq!(setup.status, AbsentNotificationStatus::Disabled);

    // ---- Enable ----
    let enable_cmd = EnableAbsentNotificationCommand {
        tenant: tenant.clone(),
        absent_notification_time_setup_id: setup.id,
    };
    let enabled_event =
        enable_absent_notification(enable_cmd, &clock, &ids, &mut setup).expect("enable");
    assert_eq!(setup.status, AbsentNotificationStatus::Enabled);
    assert_eq!(setup.version.get(), 2);
    assert_eq!(setup.updated_by, tenant.actor_id);

    assert_eq!(
        <AbsentNotificationEnabled as DomainEvent>::EVENT_TYPE,
        "communication.absent_notification.enabled"
    );
    assert_eq!(
        <AbsentNotificationEnabled as DomainEvent>::AGGREGATE_TYPE,
        "absent_notification_time_setup"
    );
    assert_eq!(<AbsentNotificationEnabled as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(enabled_event.aggregate_id(), setup.id.as_uuid());
    assert_eq!(enabled_event.school_id(), school);

    // ---- Disable ----
    let disable_cmd = DisableAbsentNotificationCommand {
        tenant: tenant.clone(),
        absent_notification_time_setup_id: setup.id,
    };
    let disabled_event =
        disable_absent_notification(disable_cmd, &clock, &ids, &mut setup).expect("disable");
    assert_eq!(setup.status, AbsentNotificationStatus::Disabled);
    assert_eq!(setup.version.get(), 3);
    assert_eq!(setup.updated_by, tenant.actor_id);
    // The schedule is still active (not soft-deleted) — disable is a
    // status transition, not a delete.
    assert!(setup.active_status.is_active());

    assert_eq!(
        <AbsentNotificationDisabled as DomainEvent>::EVENT_TYPE,
        "communication.absent_notification.disabled"
    );
    assert_eq!(
        <AbsentNotificationDisabled as DomainEvent>::AGGREGATE_TYPE,
        "absent_notification_time_setup"
    );
    assert_eq!(<AbsentNotificationDisabled as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(disabled_event.aggregate_id(), setup.id.as_uuid());
    assert_eq!(disabled_event.school_id(), school);
}
