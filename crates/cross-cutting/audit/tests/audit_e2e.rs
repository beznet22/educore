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
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};
use educore_storage::idempotency::{
    Idempotency, IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};
use educore_storage::outbox::{Outbox, SerializedEnvelope};
use educore_storage::transaction::{TenantTransaction, Transaction};

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
            failures: vec![],
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
// Stub sub-port impls for `TestTransaction`
//
// The audit writer only uses `audit_log()` on the `Transaction` it
// receives. The other three sub-ports (`outbox`, `idempotency`,
// `event_log`) are never touched by the writer's code path, but the
// trait still requires an implementation. These stubs return
// `DomainError::NotSupported` for any non-trivial operation so any
// accidental use fails loudly.
// =============================================================================

/// No-op outbox stub. The audit writer never appends to the
/// outbox, so this is a structural placeholder that exists only
/// to satisfy the `Transaction` trait contract.
#[derive(Debug, Default)]
struct StubOutbox;

#[async_trait]
impl Outbox for StubOutbox {
    async fn append(
        &self,
        _school_id: SchoolId,
        _envelope: SerializedEnvelope,
    ) -> educore_core::error::Result<()> {
        Err(educore_core::error::DomainError::not_supported(
            "Outbox stub: AuditWriter does not append to the outbox",
        ))
    }

    async fn pending(
        &self,
        _school_id: SchoolId,
        _limit: u32,
    ) -> educore_core::error::Result<Vec<SerializedEnvelope>> {
        Ok(Vec::new())
    }

    async fn mark_published(
        &self,
        _school_id: SchoolId,
        _ids: &[educore_core::ids::EventId],
    ) -> educore_core::error::Result<()> {
        Ok(())
    }
}

/// No-op idempotency stub. Mirrors `StubOutbox`: never used by
/// the audit writer, present only to satisfy the `Transaction`
/// trait contract.
#[derive(Debug, Default)]
struct StubIdempotency;

#[async_trait]
impl Idempotency for StubIdempotency {
    async fn lookup(
        &self,
        _key: IdempotencyCompositeKey,
    ) -> educore_core::error::Result<Option<IdempotencyRecord>> {
        Ok(None)
    }

    async fn record(&self, _record: IdempotencyRecord) -> educore_core::error::Result<()> {
        Err(educore_core::error::DomainError::not_supported(
            "Idempotency stub: AuditWriter does not record idempotency keys",
        ))
    }
}

/// No-op event-log stub. Mirrors `StubOutbox`: never used by
/// the audit writer, present only to satisfy the `Transaction`
/// trait contract.
#[derive(Debug, Default)]
struct StubEventLog;

#[async_trait]
impl EventLog for StubEventLog {
    async fn append(&self, _entry: EventLogEntry) -> educore_core::error::Result<()> {
        Err(educore_core::error::DomainError::not_supported(
            "EventLog stub: AuditWriter does not append to the event log",
        ))
    }

    async fn read(
        &self,
        _filter: EventLogFilter,
    ) -> educore_core::error::Result<Vec<EventLogEntry>> {
        Ok(Vec::new())
    }

    async fn count(&self, _filter: EventLogFilter) -> educore_core::error::Result<u64> {
        Ok(0)
    }
}

// =============================================================================
// TestTransaction — a hand-rolled `Transaction` impl for the audit e2e tests.
//
// The audit writer only consumes the `audit_log()` sub-port of the
// `Transaction` it is given (per SCHEMA-AUDIT-ATOMIC, the audit
// row is staged on the caller's transaction). The other three
// sub-ports are stubbed via the `Stub*` types above.
//
// The wrapped `InMemoryAuditLog` is the same mock the tests have
// always used for `AuditWriter::new(audit_log, ...)`. We continue
// to expose it directly so the existing assertions
// (`audit_log.read_for_target(...)` after a `write`) keep
// working without changes.
// =============================================================================

/// A `Transaction` that wraps an `InMemoryAuditLog` (the audit
/// sub-port) and three no-op stubs (outbox, idempotency, event
/// log). The audit writer only ever calls `audit_log().append()`,
/// so the stubs are unreachable in normal test flow; calling them
/// returns `DomainError::NotSupported` for visibility.
struct TestTransaction {
    audit_log: Arc<InMemoryAuditLog>,
    school_id: SchoolId,
    outbox: StubOutbox,
    idempotency: StubIdempotency,
    event_log: StubEventLog,
}

impl std::fmt::Debug for TestTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestTransaction")
            .field("school_id", &self.school_id)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Transaction for TestTransaction {
    async fn commit(self: Box<Self>) -> educore_core::error::Result<()> {
        // The audit writer stages rows directly into the
        // shared `InMemoryAuditLog` (no staging buffer). The
        // commit is therefore a no-op: rows are already
        // visible after `audit_log().append(...)` returns.
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> educore_core::error::Result<()> {
        // No-op: see `commit`.
        Ok(())
    }

    fn outbox(&self) -> &dyn Outbox {
        &self.outbox
    }

    fn audit_log(&self) -> &dyn AuditLog {
        &*self.audit_log
    }

    fn idempotency(&self) -> &dyn Idempotency {
        &self.idempotency
    }

    fn event_log(&self) -> &dyn EventLog {
        &self.event_log
    }
}

impl TenantTransaction for TestTransaction {
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
}

/// Helper: constructs a `Box<dyn Transaction>` wrapping `audit_log`
/// with `school_id` as the tenant anchor. Tests use this to
/// satisfy `AuditWriter::write(&dyn Transaction, ...)` without
/// pulling in `educore-testkit` (which would create a circular
/// dev-dep: `educore-audit -> educore-testkit -> educore-audit`).
fn test_txn(audit_log: Arc<InMemoryAuditLog>, school_id: SchoolId) -> Box<dyn Transaction> {
    Box::new(TestTransaction {
        audit_log,
        school_id,
        outbox: StubOutbox,
        idempotency: StubIdempotency,
        event_log: StubEventLog,
    })
}

// Silence the unused-import warning for `IdempotencyOutcome`
// (only the trait method signatures are referenced, not the
// enum directly).
#[allow(dead_code)]
fn _force_idempotency_outcome_import(o: IdempotencyOutcome) -> IdempotencyOutcome {
    o
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
    let writer = AuditWriter::new(school, audit_log.clone(), bus.clone(), clock, policy)
        .expect("valid school_id for test writer");

    let after = bytes::Bytes::from_static(b"{\"id\":\"x\"}");
    let txn = test_txn(audit_log.clone(), school);
    writer
        .write(
            &*txn,
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
    let writer = AuditWriter::new(school, audit_log.clone(), bus.clone(), clock, policy)
        .expect("valid school_id for test writer");

    let txn = test_txn(audit_log.clone(), school);
    writer
        .write(
            &*txn,
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
    let writer = AuditWriter::new(
        school,
        audit_log.clone(),
        bus.clone(),
        clock.clone(),
        policy,
    )
    .expect("valid school_id for test writer");

    // First write — seed the sweep clock, no event published.
    let txn = test_txn(audit_log.clone(), school);
    writer
        .write(
            &*txn,
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
    let txn2 = test_txn(audit_log.clone(), school);
    writer
        .write(
            &*txn2,
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
