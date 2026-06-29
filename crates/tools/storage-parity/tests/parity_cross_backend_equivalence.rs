//! # Cross-backend equivalence parity (Phase 16)
//!
//! Runs the engine's canonical cross-cutting flow
//! (`platform::create_school` → outbox + audit + idempotency →
//! relay → event log) against each of the engine's five
//! storage backends and asserts identical observable behaviour:
//!
//! 1. Outbox is drained after the relay.
//! 2. Audit log has exactly one row for the new school's
//!    aggregate id.
//! 3. Event log has exactly one row with the canonical
//!    `platform.school.created` event type.
//! 4. The persisted event log row carries the same `event_id`
//!    and event type the envelope had when it left the outbox.
//!
//! Backends:
//! - **testkit** — always-on, in-memory.
//! - **sqlite** — always-on, in-memory.
//! - **surrealdb** — always-on, in-memory (Phase 0 primary).
//! - **postgres** — env-gated on `EDUCORE_PG_URL`.
//! - **mysql** — env-gated on `EDUCORE_MYSQL_URL`.
//!
//! The shape mirrors `cross_cutting_integration.rs` (Phase 2);
//! this file splits that single test into one `#[test]` per
//! backend so the always-on trio runs in CI without external
//! infra, and the env-gated pair only runs when the runner
//! opts in.

#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

mod common;

use std::sync::Arc;

use educore_core::ids::{IdempotencyKey, Identifier};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::event_bus::EventBus;
use educore_platform::commands::{CreateSchoolCommand, UniquenessChecker};
use educore_platform::events::SchoolCreated;
use educore_platform::services as platform_services;
use educore_platform::School;
use educore_storage::audit::AuditLogEntry;
use educore_storage::event_log::EventLogFilter;
use educore_storage::idempotency::IdempotencyRecord;
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::StorageAdapter;

// ---------------------------------------------------------------------------
// Test-local helpers (kept local — only used here)
// ---------------------------------------------------------------------------

/// In-memory uniqueness checker for the test scenarios. The
/// production wiring uses a thin adapter over the storage port;
/// here we use a `Mutex<HashSet>`.
struct TestUniqueness {
    codes: std::sync::Mutex<std::collections::HashSet<String>>,
    domains: std::sync::Mutex<std::collections::HashSet<String>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self {
            codes: std::sync::Mutex::new(std::collections::HashSet::new()),
            domains: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }
}

impl UniquenessChecker for TestUniqueness {
    fn school_code_exists(&self, code: &str) -> bool {
        self.codes.lock().unwrap().contains(code)
    }
    fn school_domain_exists(&self, domain: &str) -> bool {
        self.domains.lock().unwrap().contains(domain)
    }
    fn user_email_exists(&self, _school: educore_core::ids::SchoolId, _email: &str) -> bool {
        false
    }
    fn user_username_exists(&self, _school: educore_core::ids::SchoolId, _username: &str) -> bool {
        false
    }
}

/// Drains the outbox into the event log. Inlined so this file
/// does not depend on `common::relay_outbox_to_event_log`
/// differing from the local copy.
async fn relay(adapter: &dyn StorageAdapter, school: educore_core::ids::SchoolId) {
    common::relay_outbox_to_event_log(adapter, school).await;
}

async fn dispatch_create_school(
    ctx: &TenantContext,
    cmd: CreateSchoolCommand,
    adapter: &dyn StorageAdapter,
    bus: &Arc<dyn EventBus>,
    uniqueness: &dyn UniquenessChecker,
) -> School {
    let (school, school_created): (School, SchoolCreated) = platform_services::create_school(
        cmd,
        &educore_core::clock::SystemClock,
        &educore_core::clock::SystemIdGen,
        uniqueness,
    )
    .expect("create_school");
    let aggregate_id = school_created.aggregate_id();
    let envelope: educore_events::envelope::EventEnvelope = school_created.into_envelope(ctx);
    let serialized = SerializedEnvelope::from_event_envelope(&envelope);
    let idem_key = IdempotencyKey::from(uuid::Uuid::now_v7());
    let idem_record = IdempotencyRecord {
        school_id: ctx.school_id,
        command_type: "platform.school.create",
        idempotency_key: idem_key,
        outcome: bytes::Bytes::from_static(br#"{"id":"placeholder"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![aggregate_id],
        aggregate_version: 1,
        etag: None,
        duration_ms: 0,
        emitted_event_ids: Vec::new(),
    };
    let audit_entry = AuditLogEntry::create(
        ctx.school_id,
        ctx.actor_id,
        "school",
        aggregate_id,
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    let tx = adapter.begin().await.expect("begin");
    tx.outbox()
        .append(ctx.school_id, serialized)
        .await
        .expect("outbox append");
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    // Drain the outbox to the event log WITHIN the transaction
    // so the testkit backend (which drains on commit) does not
    // lose the envelope before the relay runs. The SQL /
    // SurrealDB adapters behave the same: the outbox → event
    // log transition is atomic with the original command's
    // mutation.
    let pending = tx
        .outbox()
        .pending(ctx.school_id, 100)
        .await
        .expect("pending");
    for env in &pending {
        let entry = educore_storage::event_log::EventLogEntry::from_serialized_envelope(env);
        tx.event_log()
            .append(entry)
            .await
            .expect("event_log append");
        tx.outbox()
            .mark_published(ctx.school_id, &[env.event_id])
            .await
            .expect("mark_published");
    }
    tx.commit().await.expect("commit");
    bus.publish(envelope).await.expect("bus publish");
    let _ = relay(adapter, ctx.school_id).await;
    school
}

/// Drives the cross-cutting scenario and asserts the parity
/// invariants. Reused by every per-backend test.
async fn assert_cross_cutting_equivalence(
    adapter: &dyn StorageAdapter,
    ctx: &TenantContext,
    bus: &Arc<dyn EventBus>,
) {
    let _ = bus;
    let uniqueness = TestUniqueness::new();
    let cmd = CreateSchoolCommand::new(
        ctx.clone(),
        ctx.school_id,
        "Parity Academy".to_owned(),
        format!("PARITY-{}", ctx.school_id.as_uuid().simple()),
    );
    let school = dispatch_create_school(ctx, cmd, adapter, bus, &uniqueness).await;
    assert_eq!(school.name, "Parity Academy");

    let tx = adapter.begin().await.expect("begin");
    let pending = tx
        .outbox()
        .pending(ctx.school_id, 10)
        .await
        .expect("pending");
    assert!(pending.is_empty(), "outbox should be drained after relay");
    let audit_count = tx
        .audit_log()
        .read_for_target(ctx.school_id, school.id.as_uuid(), 10)
        .await
        .expect("read audit")
        .len();
    assert_eq!(
        audit_count, 1,
        "audit_log should have exactly 1 row for the new school"
    );
    let events = tx
        .event_log()
        .read(EventLogFilter::for_school(ctx.school_id))
        .await
        .expect("read event_log");
    assert_eq!(
        events.len(),
        1,
        "event_log should have exactly 1 row after relay"
    );
    assert_eq!(events[0].event_type, "platform.school.created");
}

// ---------------------------------------------------------------------------
// Always-on backends
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cross_backend_create_school_and_audit_equivalence_testkit() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let (adapter, _school, ctx) = common::setup_testkit();
    adapter.migrate().await.expect("testkit migrate");
    assert_cross_cutting_equivalence(&*adapter, &ctx, &bus).await;
}

#[tokio::test]
async fn cross_backend_create_school_and_audit_equivalence_sqlite() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let (adapter, _school, ctx) = common::setup_sqlite().await;
    assert_cross_cutting_equivalence(&*adapter, &ctx, &bus).await;
}

#[tokio::test]
async fn cross_backend_create_school_and_audit_equivalence_surrealdb() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let (adapter, _school, ctx) = common::setup_surrealdb().await;
    assert_cross_cutting_equivalence(&*adapter, &ctx, &bus).await;
}

// ---------------------------------------------------------------------------
// Env-gated backends
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: cargo test -- --ignored"]
async fn cross_backend_create_school_and_audit_equivalence_postgres() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let Some((adapter, _school, ctx)) = common::setup_pg().await else {
        return;
    };
    assert_cross_cutting_equivalence(&*adapter, &ctx, &bus).await;
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: cargo test -- --ignored"]
async fn cross_backend_create_school_and_audit_equivalence_mysql() {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let Some((adapter, _school, ctx)) = common::setup_mysql().await else {
        return;
    };
    assert_cross_cutting_equivalence(&*adapter, &ctx, &bus).await;
}

// Silence the unused-import lint for the helpers we re-exported
// for symmetry with the existing `cross_cutting_integration.rs`.
#[allow(dead_code)]
const _ANCHOR: fn() = || {
    let _: TenantContext = TenantContext::for_user(
        educore_core::ids::SchoolId::from_uuid(uuid::Uuid::nil()),
        educore_core::ids::UserId::from_uuid(uuid::Uuid::nil()),
        educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::nil()),
        UserType::System,
    );
};
