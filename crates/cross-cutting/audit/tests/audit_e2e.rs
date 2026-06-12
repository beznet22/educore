//! End-to-end tests for the `educore-audit` crate.
//!
//! Covers:
//!
//! 1. The [`AuditWriter`](educore_audit::AuditWriter) write path:
//!    it appends a row to the storage port and triggers an
//!    opportunistic sweep check.
//! 2. The sweep check: the writer does **not** emit a
//!    [`RetentionSweepDue`](educore_audit::RetentionSweepDue) on
//!    the first call, but **does** emit one when the sweep
//!    interval has elapsed AND the storage port reports a row
//!    older than the retention threshold.
//! 3. The [`RetentionSweeper`](educore_audit::RetentionSweeper)
//!    threshold-check state machine.
//! 4. The default [`RetentionPolicy`](educore_audit::RetentionPolicy).
//! 5. The `RetentionSweepDue` event wire shape (event type, payload).
//! 6. The re-exported [`AuditLogEntry`](educore_audit::AuditLogEntry)
//!    `create` helper.
//!
//! The tests run against in-memory mocks for the storage port and
//! the event bus; the mocks are the test's responsibility, not the
//! audit crate's.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use uuid::Uuid;

use educore_audit::prelude::*;
use educore_core::clock::{IdGenerator, SystemIdGen, TestClock};
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    BatchReceipt, EventBus, EventSubscription, PublishReceipt, SubscribeOptions,
};
use educore_storage::audit::{AuditLog, AuditLogEntry};

// =============================================================================
// In-memory mocks for the storage and bus ports.
// =============================================================================

/// In-memory `AuditLog` mock. Records every appended entry and
/// serves `read_for_target` lookups.
///
/// For the sentinel target id (used by the sweep check to look up
/// "the oldest row for the school"), the mock returns the oldest
/// entry with the matching `school_id`. This matches the Phase 2
/// simplification: the storage adapter is expected to interpret
/// the sentinel as "give me the oldest row for the school" (Phase
/// 3 will formalise this with a dedicated method).
#[derive(Debug, Default)]
struct InMemoryAuditLog {
    entries: Mutex<Vec<AuditLogEntry>>,
}

impl InMemoryAuditLog {
    fn new() -> Self {
        Self::default()
    }

    /// Pre-populates the mock with an entry. Used by the sweep
    /// tests to seed an "old" row.
    fn seed(&self, entry: AuditLogEntry) {
        self.entries.lock().unwrap().push(entry);
    }
}

#[async_trait]
impl AuditLog for InMemoryAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> educore_core::error::Result<()> {
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> educore_core::error::Result<Vec<AuditLogEntry>> {
        let guard = self.entries.lock().unwrap();
        let filtered: Vec<AuditLogEntry> = if target_id == SENTINEL_TARGET_ID {
            // Sentinel: return the oldest entry for the school.
            guard
                .iter()
                .filter(|e| e.school_id == school_id)
                .min_by_key(|e| e.occurred_at)
                .cloned()
                .into_iter()
                .collect()
        } else {
            guard
                .iter()
                .filter(|e| e.school_id == school_id && e.target_id == target_id)
                .cloned()
                .collect()
        };
        Ok(filtered.into_iter().take(limit as usize).collect())
    }
}

/// In-memory `EventBus` mock. Records every published envelope so
/// the tests can assert on event contents and ordering.
#[derive(Debug, Default)]
struct InMemoryEventBus {
    published: Mutex<Vec<EventEnvelope>>,
}

impl InMemoryEventBus {
    fn new() -> Self {
        Self::default()
    }

    fn snapshot(&self) -> Vec<EventEnvelope> {
        self.published.lock().unwrap().clone()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(
        &self,
        envelope: EventEnvelope,
    ) -> educore_core::error::Result<PublishReceipt> {
        let receipt = PublishReceipt::new(
            envelope.event_id,
            envelope.event_type.to_owned(),
            envelope.occurred_at,
        );
        self.published.lock().unwrap().push(envelope);
        Ok(receipt)
    }

    async fn publish_batch(
        &self,
        envelopes: Vec<EventEnvelope>,
    ) -> educore_core::error::Result<BatchReceipt> {
        let receipts: Vec<PublishReceipt> = envelopes
            .iter()
            .map(|e| PublishReceipt::new(e.event_id, e.event_type.to_owned(), e.occurred_at))
            .collect();
        self.published.lock().unwrap().extend(envelopes);
        Ok(BatchReceipt {
            receipts,
            correlation_id: None,
        })
    }

    async fn subscribe(
        &self,
        _options: SubscribeOptions,
    ) -> educore_core::error::Result<Box<dyn EventSubscription>> {
        Err(educore_core::error::DomainError::not_supported(
            "subscribe not implemented in InMemoryEventBus",
        ))
    }
}

// =============================================================================
// Test helpers
// =============================================================================

fn ts(secs: i64) -> Timestamp {
    Timestamp::from_datetime(Utc.timestamp_opt(secs, 0).single().unwrap_or_else(Utc::now))
}

fn make_ctx(school: SchoolId, user: UserId) -> TenantContext {
    let g = SystemIdGen;
    let corr = g.next_correlation_id();
    TenantContext::for_user(school, user, corr, UserType::Teacher)
}

// =============================================================================
// 1. audit_writer_appends_row_to_audit_log
// =============================================================================

#[tokio::test]
async fn audit_writer_appends_row_to_audit_log() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let target = g.next_uuid();
    let ctx = make_ctx(school, actor);

    let audit_log = Arc::new(InMemoryAuditLog::new());
    let bus = Arc::new(InMemoryEventBus::new());
    let clock = Arc::new(TestClock::at(ts(1_700_000_000)));
    let policy = RetentionPolicy::default();
    let writer = AuditWriter::new(audit_log.clone(), bus.clone(), clock, policy);

    let after = bytes::Bytes::from_static(b"{\"id\":\"x\"}");
    writer
        .write(
            &ctx,
            AuditAction::Create,
            AuditTarget::Student(target),
            None,
            Some(after),
        )
        .await
        .unwrap();

    let rows = audit_log.read_for_target(school, target, 1).await.unwrap();
    assert_eq!(rows.len(), 1);
    let entry = &rows[0];
    assert_eq!(entry.school_id, school);
    assert_eq!(entry.actor_id, actor);
    assert_eq!(entry.action, "create");
    assert_eq!(entry.target_type, "student");
    assert_eq!(entry.target_id, target);
    assert!(entry.before.is_none());
    assert!(entry.after.is_some());
}

// =============================================================================
// 2. audit_writer_does_not_emit_sweep_due_on_first_write
// =============================================================================

#[tokio::test]
async fn audit_writer_does_not_emit_sweep_due_on_first_write() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let target = g.next_uuid();
    let ctx = make_ctx(school, actor);

    let audit_log = Arc::new(InMemoryAuditLog::new());
    let bus = Arc::new(InMemoryEventBus::new());
    let clock = Arc::new(TestClock::at(ts(1_700_000_000)));
    let policy = RetentionPolicy::default();
    let writer = AuditWriter::new(audit_log.clone(), bus.clone(), clock, policy);

    writer
        .write(
            &ctx,
            AuditAction::Update,
            AuditTarget::Student(target),
            Some(bytes::Bytes::from_static(b"{}")),
            Some(bytes::Bytes::from_static(b"{}")),
        )
        .await
        .unwrap();

    let envelopes = bus.snapshot();
    assert!(
        envelopes.is_empty(),
        "first write must not publish a sweep_due event (sweep interval not elapsed)"
    );
}

// =============================================================================
// 3. audit_writer_emits_sweep_due_when_threshold_reached
// =============================================================================

#[tokio::test]
async fn audit_writer_emits_sweep_due_when_threshold_reached() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let target = g.next_uuid();
    let ctx = make_ctx(school, actor);

    let audit_log = Arc::new(InMemoryAuditLog::new());
    // Pre-seed an "old" row from 100 days ago (retention = 90 days
    // in the policy below). The mock's sentinel lookup returns
    // the oldest row for the school.
    let now = ts(1_700_000_000);
    let hundred_days_ago =
        Timestamp::from_datetime(now.as_datetime() - chrono::Duration::days(100));
    let old_entry = AuditLogEntry {
        school_id: school,
        actor_id: g.next_user_id(),
        action: "create".to_owned(),
        target_type: "student".to_owned(),
        target_id: g.next_uuid(),
        before: None,
        after: Some(bytes::Bytes::from_static(b"{}")),
        event_id: None,
        correlation_id: g.next_correlation_id(),
        occurred_at: hundred_days_ago,
        active_status: educore_core::value_objects::ActiveStatus::Active,
        metadata: serde_json::Value::Null,
    };
    audit_log.seed(old_entry);

    let bus = Arc::new(InMemoryEventBus::new());
    let clock = Arc::new(TestClock::at(now));
    let policy = RetentionPolicy {
        retention_days: 90,
        sweep_check_interval: std::time::Duration::from_secs(60),
    };
    let writer = AuditWriter::new(audit_log.clone(), bus.clone(), clock.clone(), policy);

    // First write — seed the sweep clock, no event published.
    writer
        .write(
            &ctx,
            AuditAction::Create,
            AuditTarget::Student(target),
            None,
            Some(bytes::Bytes::from_static(b"{}")),
        )
        .await
        .unwrap();
    assert!(
        bus.snapshot().is_empty(),
        "first write must not emit a sweep_due"
    );

    // Advance the clock past the sweep interval.
    clock.advance(chrono::Duration::seconds(120));

    // Second write — sweep check fires, threshold reached, event
    // is published.
    writer
        .write(
            &ctx,
            AuditAction::Update,
            AuditTarget::Student(target),
            Some(bytes::Bytes::from_static(b"{}")),
            Some(bytes::Bytes::from_static(b"{}")),
        )
        .await
        .unwrap();

    let envelopes = bus.snapshot();
    assert_eq!(envelopes.len(), 1, "exactly one sweep_due event expected");
    let env = &envelopes[0];
    assert_eq!(env.event_type, "audit.retention.sweep_due");
    assert_eq!(env.school_id, school);
    // The payload is the serialised event struct.
    assert!(env.payload.get("cutoff").is_some());
    assert!(env.payload.get("at").is_some());
    assert!(env.payload.get("school_id").is_some());
}

// =============================================================================
// 4. retention_policy_default_is_90_days
// =============================================================================

#[test]
fn retention_policy_default_is_90_days() {
    let p = RetentionPolicy::default();
    assert_eq!(p.retention_days, 90);
    assert_eq!(p.sweep_check_interval, std::time::Duration::from_secs(3600));
}

// =============================================================================
// 5. retention_sweeper_should_sweep_returns_false_before_interval
// =============================================================================

#[test]
fn retention_sweeper_should_sweep_returns_false_before_interval() {
    let mut s = RetentionSweeper::new();
    let policy = RetentionPolicy {
        retention_days: 90,
        sweep_check_interval: std::time::Duration::from_secs(60),
    };
    let now = ts(1_000);
    assert!(!s.should_sweep(now, &policy));
}

// =============================================================================
// 6. retention_sweeper_should_sweep_returns_true_after_interval
// =============================================================================

#[test]
fn retention_sweeper_should_sweep_returns_true_after_interval() {
    let mut s = RetentionSweeper::new();
    let policy = RetentionPolicy {
        retention_days: 90,
        sweep_check_interval: std::time::Duration::from_secs(60),
    };
    let t0 = ts(1_000);
    let t1 = ts(1_000 + 120); // 120s later
    assert!(!s.should_sweep(t0, &policy));
    assert!(s.should_sweep(t1, &policy));
}

// =============================================================================
// 7. retention_sweep_due_event_has_correct_event_type
// =============================================================================

#[test]
fn retention_sweep_due_event_has_correct_event_type() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::system(school, corr);
    let now = ts(1_700_000_000);
    let cutoff = ts(1_700_000_000 - 7 * 86_400);
    let event = RetentionSweepDue::new(school, cutoff, now);
    let env = event.into_envelope(&ctx);
    assert_eq!(env.event_type, "audit.retention.sweep_due");
    assert_eq!(env.aggregate_type, "audit_retention");
    assert_eq!(env.school_id, school);
    assert_eq!(env.correlation_id, corr);
    assert_eq!(env.aggregate_id, school.as_uuid());
    assert_eq!(env.occurred_at, now);
    // The payload is a JSON Value carrying the typed event's
    // serialised fields.
    assert_eq!(
        env.payload["school_id"],
        serde_json::json!(school.as_uuid())
    );
    assert!(env.payload["cutoff"].is_string() || env.payload["cutoff"].is_object());
    assert!(env.payload["at"].is_string() || env.payload["at"].is_object());
}

// =============================================================================
// 8. audit_log_entry_create_helper_no_before
// =============================================================================

#[test]
fn audit_log_entry_create_helper_no_before() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let target = g.next_uuid();
    let corr = g.next_correlation_id();
    let entry = AuditLogEntry::create(
        school,
        actor,
        "student",
        target,
        bytes::Bytes::from_static(b"{\"id\":\"x\"}"),
        corr,
    );
    assert_eq!(entry.action, "create");
    assert!(entry.before.is_none());
    assert!(entry.after.is_some());
    assert_eq!(entry.school_id, school);
    assert_eq!(entry.actor_id, actor);
    assert_eq!(entry.target_id, target);
    assert_eq!(entry.target_type, "student");
    assert_eq!(entry.correlation_id, corr);
}
