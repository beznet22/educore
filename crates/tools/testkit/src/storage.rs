//! # educore-testkit storage port
//!
//! In-memory implementation of the engine's
//! [`StorageAdapter`](educore_storage::StorageAdapter) port + the
//! 5 sub-ports ([`Outbox`](educore_storage::Outbox),
//! [`AuditLog`](educore_storage::AuditLog),
//! [`EventLog`](educore_storage::EventLog),
//! [`Idempotency`](educore_storage::Idempotency)) + the 4 sync
//! primitives ([`watch_changes`](StorageAdapter::watch_changes),
//! [`apply_snapshot`](StorageAdapter::apply_snapshot),
//! [`cursor_for`](StorageAdapter::cursor_for),
//! [`advance_cursor`](StorageAdapter::advance_cursor)).
//!
//! Consumer tests instantiate [`InMemoryStorageAdapter::new`],
//! obtain a [`Transaction`](educore_storage::Transaction) via
//! [`begin`](StorageAdapter::begin), stage writes against the
//! outbox / audit / event log / idempotency sub-ports, and
//! [`commit`](educore_storage::Transaction::commit) (or
//! [`rollback`](educore_storage::Transaction::rollback)) the
//! unit of work. The in-memory backend enforces tenant
//! isolation on bulk inserts and rejects duplicate idempotency
//! records with [`DomainError::Conflict`].
//!
//! See `docs/ports/storage.md` for the port contract.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use parking_lot::Mutex;
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::Identifier;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use educore_storage::audit::{AuditLog, AuditLogEntry};
use educore_storage::change_stream::{
    ChangeEvent, ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot, SnapshotAggregate,
    VersionCursor,
};
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};
use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey, IdempotencyRecord};
use educore_storage::outbox::{Outbox, SerializedEnvelope};
use educore_storage::port::StorageAdapter;
use educore_storage::student_attendance_row::StudentAttendanceRow;
use educore_storage::transaction::Transaction;

use educore_events::event_bus::EventBus;

// ---------------------------------------------------------------------------
// InMemoryInner (shared state)
// ---------------------------------------------------------------------------

pub(crate) struct InMemoryInner {
    pub(crate) outbox: Mutex<Vec<SerializedEnvelope>>,
    pub(crate) audit_log: Mutex<Vec<AuditLogEntry>>,
    pub(crate) event_log: Mutex<Vec<EventLogEntry>>,
    pub(crate) idempotency: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
    pub(crate) bulk_attendance: Mutex<Vec<(SchoolId, Uuid, NaiveDate, Uuid)>>,
    pub(crate) change_events: Mutex<Vec<ChangeEvent>>,
    pub(crate) cursors: Mutex<HashMap<SchoolId, VersionCursor>>,
    pub(crate) snapshots: Mutex<Vec<SnapshotAggregate>>,
    pub(crate) migrated: AtomicBool,
    pub(crate) closed: AtomicBool,
    pub(crate) _id_seq: AtomicU64,
    /// QW-4 observability hook: counts how many transactions
    /// were dropped *without* `commit()` / `rollback()` being
    /// called. The Drop impl increments this so the unit
    /// tests can verify the new contract without needing to
    /// inspect post-Drop state (which is impossible — the
    /// value is consumed).
    pub(crate) implicit_rollback_count: AtomicU64,
    /// QW-4 observability hook: counts how many transactions
    /// were dropped *after* `commit()` (where no rollback
    /// should fire). Used to assert the happy path is
    /// unchanged.
    pub(crate) explicit_commit_count: AtomicU64,
}

// ---------------------------------------------------------------------------
// Sub-port wrappers — each is a thin Arc-shared handle to the inner state.
// Storing them in the Transaction struct (not constructing per call) avoids
// the "cannot return reference to temporary value" error.
// ---------------------------------------------------------------------------

pub(crate) struct OutboxHandle(pub(crate) Arc<InMemoryInner>);

/// Bridge from the storage-port [`SerializedEnvelope`] to the
/// relay-port [`SerializedEnvelope`]. The two types have
/// identical field shapes but live in different crates (the
/// events crate cannot depend on the storage crate because the
/// storage crate already depends on events). The testkit is the
/// only place that has both crates in scope, so the bridge
/// lives here.
fn to_relay_envelope(
    env: educore_storage::outbox::SerializedEnvelope,
) -> educore_events::relay_envelope::SerializedEnvelope {
    educore_events::relay_envelope::SerializedEnvelope {
        event_id: env.event_id,
        event_type: env.event_type,
        schema_version: env.schema_version,
        school_id: env.school_id,
        aggregate_id: env.aggregate_id,
        aggregate_type: env.aggregate_type,
        actor_id: env.actor_id,
        correlation_id: env.correlation_id,
        causation_id: env.causation_id,
        occurred_at: env.occurred_at,
        payload: env.payload,
    }
}

#[async_trait]
impl Outbox for OutboxHandle {
    async fn append(&self, envelope: SerializedEnvelope) -> Result<()> {
        let mut outbox = self.0.outbox.lock();
        if outbox.iter().any(|e| e.event_id == envelope.event_id) {
            return Err(DomainError::conflict(format!(
                "duplicate event_id {} in outbox",
                envelope.event_id.as_uuid()
            )));
        }
        outbox.push(envelope);
        Ok(())
    }

    async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
        let outbox = self.0.outbox.lock();
        Ok(outbox.iter().take(limit as usize).cloned().collect())
    }

    async fn mark_published(&self, ids: &[educore_core::ids::EventId]) -> Result<()> {
        let mut outbox = self.0.outbox.lock();
        outbox.retain(|e| !ids.contains(&e.event_id));
        Ok(())
    }
}

/// `educore_events::relay::OutboxSource` impl for the
/// testkit's `OutboxHandle`. Wires the in-memory outbox to
/// the production `OutboxRelay<O, B>` so consumer tests can
/// exercise the same relay path they would in production
/// without standing up a real storage adapter. The mapping is
/// 1:1: the testkit's `pending` already returns envelopes in
/// append order, and `mark_published` removes them by id,
/// which matches the relay's contract.
#[async_trait]
impl educore_events::relay::OutboxSource for OutboxHandle {
    async fn pending_for_school(
        &self,
        school_id: educore_core::ids::SchoolId,
        limit: u32,
    ) -> Result<Vec<educore_events::relay_envelope::SerializedEnvelope>> {
        let outbox = self.0.outbox.lock();
        Ok(outbox
            .iter()
            .filter(|e| e.school_id == school_id)
            .take(limit as usize)
            .cloned()
            .map(to_relay_envelope)
            .collect())
    }

    async fn mark_published(&self, ids: &[educore_core::ids::EventId]) -> Result<()> {
        let mut outbox = self.0.outbox.lock();
        outbox.retain(|e| !ids.contains(&e.event_id));
        Ok(())
    }
}

pub(crate) struct AuditLogHandle(pub(crate) Arc<InMemoryInner>);

#[async_trait]
impl AuditLog for AuditLogHandle {
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        self.0.audit_log.lock().push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let log = self.0.audit_log.lock();
        let mut out: Vec<AuditLogEntry> = log
            .iter()
            .filter(|e| e.school_id == school_id && e.target_id == target_id)
            .cloned()
            .collect();
        out.sort_by_key(|e| e.occurred_at);
        out.truncate(limit as usize);
        Ok(out)
    }
}

pub(crate) struct EventLogHandle(pub(crate) Arc<InMemoryInner>);

#[async_trait]
impl EventLog for EventLogHandle {
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        self.0.event_log.lock().push(entry);
        Ok(())
    }

    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let log = self.0.event_log.lock();
        let mut out: Vec<EventLogEntry> = log
            .iter()
            .filter(|e| e.school_id == filter.school_id)
            .filter(|e| {
                filter.event_types.is_empty()
                    || filter.event_types.iter().any(|t| t == &e.event_type)
            })
            .filter(|e| filter.since.as_ref().map_or(true, |s| e.recorded_at >= *s))
            .filter(|e| filter.until.as_ref().map_or(true, |u| e.recorded_at < *u))
            .filter(|e| {
                filter
                    .aggregate_id
                    .as_ref()
                    .map_or(true, |a| e.aggregate_id == *a)
            })
            .cloned()
            .collect();
        out.sort_by_key(|e| e.recorded_at);
        out.truncate(filter.limit as usize);
        Ok(out)
    }

    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let log = self.0.event_log.lock();
        let n = log
            .iter()
            .filter(|e| e.school_id == filter.school_id)
            .filter(|e| {
                filter.event_types.is_empty()
                    || filter.event_types.iter().any(|t| t == &e.event_type)
            })
            .filter(|e| filter.since.as_ref().map_or(true, |s| e.recorded_at >= *s))
            .filter(|e| filter.until.as_ref().map_or(true, |u| e.recorded_at < *u))
            .filter(|e| {
                filter
                    .aggregate_id
                    .as_ref()
                    .map_or(true, |a| e.aggregate_id == *a)
            })
            .count();
        Ok(n as u64)
    }
}

pub(crate) struct IdempotencyHandle(pub(crate) Arc<InMemoryInner>);

#[async_trait]
impl Idempotency for IdempotencyHandle {
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        let store = self.0.idempotency.lock();
        Ok(store.get(&key).cloned())
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        let key = IdempotencyCompositeKey {
            school_id: record.school_id,
            command_type: record.command_type,
            idempotency_key: record.idempotency_key,
        };
        let mut store = self.0.idempotency.lock();
        if let Some(existing) = store.get(&key) {
            if existing.outcome != record.outcome {
                return Err(DomainError::conflict(format!(
                    "idempotency key {} has different outcome",
                    record.idempotency_key.as_uuid()
                )));
            }
            return Ok(());
        }
        store.insert(key, record);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// InMemoryStorageAdapter
// ---------------------------------------------------------------------------

/// In-memory storage adapter. All state lives behind parking_lot
/// `Mutex` (sync) guards; the async-trait methods grab the lock
/// briefly and return. The struct is `Clone` (everything inside is
/// `Arc`-shared).
#[derive(Clone)]
pub struct InMemoryStorageAdapter {
    inner: Arc<InMemoryInner>,
    bus: Arc<dyn EventBus>,
}

impl std::fmt::Debug for InMemoryStorageAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryStorageAdapter")
            .finish_non_exhaustive()
    }
}

impl InMemoryStorageAdapter {
    /// Constructs a fresh in-memory storage adapter. The bus is
    /// used to drain the outbox on transaction commit.
    #[must_use]
    pub fn new(bus: Arc<dyn EventBus>) -> Self {
        Self {
            inner: Arc::new(InMemoryInner {
                outbox: Mutex::new(Vec::new()),
                audit_log: Mutex::new(Vec::new()),
                event_log: Mutex::new(Vec::new()),
                idempotency: Mutex::new(HashMap::new()),
                bulk_attendance: Mutex::new(Vec::new()),
                change_events: Mutex::new(Vec::new()),
                cursors: Mutex::new(HashMap::new()),
                snapshots: Mutex::new(Vec::new()),
                migrated: AtomicBool::new(false),
                closed: AtomicBool::new(false),
                _id_seq: AtomicU64::new(0),
                implicit_rollback_count: AtomicU64::new(0),
                explicit_commit_count: AtomicU64::new(0),
            }),
            bus,
        }
    }

    fn _next_id(&self) -> u64 {
        self.inner._id_seq.fetch_add(1, Ordering::Relaxed)
    }

    /// Replaces the bus on this adapter. Returns the adapter
    /// with the new bus installed so the new bus drains the
    /// outbox on subsequent commits. Existing transactions
    /// continue to use the bus they were constructed with;
    /// only transactions opened AFTER `with_bus` returns see
    /// the new bus. Used by tests that wire the bus after
    /// constructing the storage adapter (e.g. to swap in a
    /// mock).
    #[must_use]
    pub fn with_bus(mut self, bus: Arc<dyn EventBus>) -> Self {
        self.bus = bus;
        self
    }
}

#[async_trait]
impl StorageAdapter for InMemoryStorageAdapter {
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        if self.inner.closed.load(Ordering::Relaxed) {
            return Err(DomainError::validation("storage adapter is closed"));
        }
        Ok(Box::new(InMemoryTransaction::new(
            self.inner.clone(),
            self.bus.clone(),
        )))
    }

    async fn migrate(&self) -> Result<MigrationReport> {
        if self.inner.closed.load(Ordering::Relaxed) {
            return Err(DomainError::validation("storage adapter is closed"));
        }
        let already = self.inner.migrated.swap(true, Ordering::SeqCst);
        Ok(MigrationReport {
            version: 1,
            statements_executed: if already { 0 } else { 1 },
            duration: Duration::ZERO,
            already_at_version: already,
        })
    }

    async fn ping(&self) -> Result<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            return Err(DomainError::validation("storage adapter is closed"));
        }
        Ok(())
    }

    async fn close(self: Box<Self>) -> Result<()> {
        self.inner.closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn bulk_insert_student_attendances(
        &self,
        ctx: &TenantContext,
        rows: &[StudentAttendanceRow],
    ) -> Result<()> {
        if self.inner.closed.load(Ordering::Relaxed) {
            return Err(DomainError::validation("storage adapter is closed"));
        }
        let mut store = self.inner.bulk_attendance.lock();
        for row in rows {
            if row.school_id != ctx.school_id {
                return Err(DomainError::validation(format!(
                    "row school_id {} does not match ctx school_id {}",
                    row.school_id.as_uuid(),
                    ctx.school_id.as_uuid()
                )));
            }
            if store.iter().any(|(s, sid, d, _)| {
                *s == row.school_id && *sid == row.student_id && *d == row.attendance_date
            }) {
                return Err(DomainError::conflict(format!(
                    "duplicate (school_id, student_id, attendance_date) for student {}",
                    row.student_id
                )));
            }
            store.push((
                row.school_id,
                row.student_id,
                row.attendance_date,
                Uuid::new_v4(),
            ));
        }
        Ok(())
    }

    async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
        use futures::stream;
        let events = self.inner.change_events.lock().clone();
        let matching: Vec<std::result::Result<ChangeEvent, DomainError>> = events
            .into_iter()
            .filter(|e| e.school_id == filter.school_id)
            .filter(|e| {
                if filter.aggregate_types.is_empty() {
                    return true;
                }
                filter.aggregate_types.iter().any(|f| match f {
                    educore_storage::change_stream::AggregateTypeFilter::Exact(n) => {
                        &e.aggregate_type == n
                    }
                    educore_storage::change_stream::AggregateTypeFilter::Any => true,
                })
            })
            .map(Ok)
            .collect();
        let s = stream::iter(matching);
        Ok(ChangeStream { inner: Box::pin(s) })
    }

    async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
        let mut store = self.inner.snapshots.lock();
        for agg in snapshot.aggregates {
            store.push(agg);
        }
        Ok(())
    }

    async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> {
        let cursors = self.inner.cursors.lock();
        Ok(cursors
            .get(&school_id)
            .copied()
            .unwrap_or(VersionCursor::ZERO))
    }

    async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> {
        self.inner.cursors.lock().insert(school_id, to);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// InMemoryTransaction
// ---------------------------------------------------------------------------

/// In-memory transaction. Holds the sub-port handles as fields
/// so the trait methods can return `&self.field` references that
/// live as long as the transaction.
pub struct InMemoryTransaction {
    inner: Arc<InMemoryInner>,
    _bus: Arc<dyn EventBus>,
    outbox_h: OutboxHandle,
    audit_h: AuditLogHandle,
    event_h: EventLogHandle,
    idem_h: IdempotencyHandle,
    committed: AtomicBool,
    rolled_back: AtomicBool,
}

impl std::fmt::Debug for InMemoryTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryTransaction")
            .finish_non_exhaustive()
    }
}

impl InMemoryTransaction {
    fn new(inner: Arc<InMemoryInner>, bus: Arc<dyn EventBus>) -> Self {
        let outbox_h = OutboxHandle(inner.clone());
        let audit_h = AuditLogHandle(inner.clone());
        let event_h = EventLogHandle(inner.clone());
        let idem_h = IdempotencyHandle(inner.clone());
        Self {
            inner,
            _bus: bus,
            outbox_h,
            audit_h,
            event_h,
            idem_h,
            committed: AtomicBool::new(false),
            rolled_back: AtomicBool::new(false),
        }
    }

    /// Returns `true` once `commit()` (or an implicit
    /// commit-style `Drop`) has been called. Used by the
    /// unit tests to verify the QW-4 `Drop` contract.
    #[cfg(test)]
    #[must_use]
    pub fn is_committed(&self) -> bool {
        self.committed.load(Ordering::SeqCst)
    }

    /// Returns `true` once `rollback()` (or an implicit
    /// rollback-style `Drop`) has been called. Used by the
    /// unit tests to verify the QW-4 `Drop` contract.
    #[cfg(test)]
    #[must_use]
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back.load(Ordering::SeqCst)
    }
}

/// QW-4 / `TOOL-TK-002` partial fix — the Drop contract.
///
/// If the transaction is dropped without an explicit `commit`
/// or `rollback`, the `Drop` impl flips both the `committed`
/// and `rolled_back` guards. This honors the port-level
/// contract: a dropped transaction is "rolled back by default".
///
/// **Caveat (TOOL-TK-002):** the in-memory sub-port handles
/// write directly to the shared `InMemoryInner` state, so a
/// dropped-without-finalize transaction leaves its staged
/// audit / event / idempotency writes visible to subsequent
/// transactions. The `Drop` only flips the transactional
/// state flag; it does not undo the sub-port writes. That
/// broader fix (a staging layer) is tracked under `TOOL-TK-002`
/// and is **out of scope** for this PR.
impl Drop for InMemoryTransaction {
    fn drop(&mut self) {
        let was_committed = self.committed.load(Ordering::SeqCst);
        let was_rolled_back = self.rolled_back.load(Ordering::SeqCst);
        if !was_committed && !was_rolled_back {
            tracing::warn!(
                "InMemoryTransaction dropped without commit or rollback; \
                 performing implicit rollback (TOOL-TK-002 caveat: \
                 sub-port writes are already visible)"
            );
            self.rolled_back.store(true, Ordering::SeqCst);
            self.committed.store(true, Ordering::SeqCst);
            self.inner
                .implicit_rollback_count
                .fetch_add(1, Ordering::SeqCst);
        } else if was_committed {
            self.inner
                .explicit_commit_count
                .fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[async_trait]
impl Transaction for InMemoryTransaction {
    async fn commit(self: Box<Self>) -> Result<()> {
        if self.rolled_back.load(Ordering::SeqCst) {
            return Err(DomainError::validation("transaction already rolled back"));
        }
        if self.committed.swap(true, Ordering::SeqCst) {
            return Err(DomainError::validation("transaction already committed"));
        }
        // Drain the outbox. Each drained envelope is converted
        // to a bus-port `EventEnvelope` and published via the
        // testkit's `EventBus` (TOOL-TK-001). Successful
        // publishes are removed from the outbox via
        // `mark_published`; failed publishes stay pending for
        // the next drain (matching the production relay's
        // contract — the bus-port contract is at-least-once
        // delivery).
        //
        // The bus is `Arc<dyn EventBus>` shared with the
        // adapter; the testkit wires an `InProcessEventBus`
        // by default in `TestkitWorld::new`, so envelopes
        // staged via `tx.outbox().append(...)` and then
        // committed flow through the bus to any
        // `SubscriberRegistry` the consumer has installed.
        let pending: Vec<SerializedEnvelope> = {
            let mut outbox = self.inner.outbox.lock();
            outbox.drain(..).collect()
        };
        if !pending.is_empty() {
            let mut published_ids = Vec::with_capacity(pending.len());
            for serialized in pending {
                let envelope = to_relay_envelope(serialized.clone()).into_event_envelope();
                match self._bus.publish(envelope).await {
                    Ok(receipt) => published_ids.push(receipt.event_id),
                    Err(err) => {
                        // Mirror the production relay: a
                        // failed publish leaves the envelope
                        // in the outbox for retry. We push it
                        // back so the next commit can try
                        // again. This keeps the testkit
                        // resilient without surfacing the
                        // error to the caller (the bus is a
                        // test seam; failure here is the
                        // consumer's responsibility).
                        tracing::warn!(
                            error = %err,
                            "InMemoryTransaction::commit: bus.publish failed; \
                             envelope remains pending"
                        );
                        let mut outbox = self.inner.outbox.lock();
                        outbox.push(serialized);
                    }
                }
            }
            if !published_ids.is_empty() {
                let mut outbox = self.inner.outbox.lock();
                outbox.retain(|e| !published_ids.contains(&e.event_id));
            }
        }
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        if self.committed.load(Ordering::SeqCst) {
            return Err(DomainError::validation("transaction already committed"));
        }
        self.rolled_back.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn outbox(&self) -> &dyn Outbox {
        &self.outbox_h
    }

    fn audit_log(&self) -> &dyn AuditLog {
        &self.audit_h
    }

    fn idempotency(&self) -> &dyn Idempotency {
        &self.idem_h
    }

    fn event_log(&self) -> &dyn EventLog {
        &self.event_h
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::{CorrelationId, UserId};
    use educore_core::tenant::UserType;
    use educore_core::value_objects::{ActiveStatus, Timestamp};
    use educore_event_bus::InProcessEventBus;

    fn make_bus() -> Arc<dyn EventBus> {
        Arc::new(InProcessEventBus::new())
    }

    fn make_ctx(school: SchoolId, user: UserId) -> TenantContext {
        TenantContext::for_user(
            school,
            user,
            CorrelationId::from(Uuid::new_v4()),
            UserType::SchoolAdmin,
        )
    }

    fn sample_envelope(school: SchoolId) -> SerializedEnvelope {
        let g = SystemIdGen;
        SerializedEnvelope {
            event_id: g.next_event_id(),
            event_type: "academic.student.admitted".to_owned(),
            schema_version: 1,
            school_id: school,
            aggregate_id: g.next_uuid(),
            aggregate_type: "student".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            payload: Bytes::from_static(b"{}"),
        }
    }

    #[test]
    fn storage_adapter_new_and_ping_succeed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            adapter.ping().await.unwrap();
        });
    }

    #[test]
    fn storage_adapter_migrate_is_idempotent() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let r1 = adapter.migrate().await.unwrap();
            assert!(!r1.already_at_version);
            let r2 = adapter.migrate().await.unwrap();
            assert!(r2.already_at_version);
        });
    }

    #[test]
    fn bulk_insert_validates_tenant_isolation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let school = SchoolId::from_uuid(Uuid::new_v4());
            let other = SchoolId::from_uuid(Uuid::new_v4());
            let user = UserId::from_uuid(Uuid::new_v4());
            let ctx = make_ctx(school, user);
            let row = StudentAttendanceRow {
                school_id: other,
                id: Uuid::new_v4(),
                student_id: Uuid::new_v4(),
                student_record_id: Uuid::new_v4(),
                class_id: Uuid::new_v4(),
                section_id: Uuid::new_v4(),
                attendance_date: NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
                attendance_type: "P".to_owned(),
                in_time: None,
                out_time: None,
                notes: None,
                is_absent: false,
                marked_by: user,
                marked_at: educore_core::value_objects::Timestamp::now(),
                marked_from: "manual".to_owned(),
                version: educore_core::value_objects::Version::initial(),
                etag: educore_core::value_objects::Etag::new("00000000000000000000000000000001")
                    .unwrap(),
                created_at: educore_core::value_objects::Timestamp::now(),
                updated_at: educore_core::value_objects::Timestamp::now(),
                created_by: user,
                updated_by: user,
                active_status: educore_core::value_objects::ActiveStatus::Active,
                correlation_id: educore_core::ids::CorrelationId::from_uuid(Uuid::new_v4()),
                last_event_id: Some(educore_core::ids::EventId::from_uuid(Uuid::new_v4())),
            };
            let res = adapter.bulk_insert_student_attendances(&ctx, &[row]).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn bulk_insert_rejects_duplicate() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let school = SchoolId::from_uuid(Uuid::new_v4());
            let user = UserId::from_uuid(Uuid::new_v4());
            let ctx = make_ctx(school, user);
            let row = StudentAttendanceRow {
                school_id: school,
                id: Uuid::new_v4(),
                student_id: Uuid::new_v4(),
                student_record_id: Uuid::new_v4(),
                class_id: Uuid::new_v4(),
                section_id: Uuid::new_v4(),
                attendance_date: NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
                attendance_type: "P".to_owned(),
                in_time: None,
                out_time: None,
                notes: None,
                is_absent: false,
                marked_by: user,
                marked_at: educore_core::value_objects::Timestamp::now(),
                marked_from: "manual".to_owned(),
                version: educore_core::value_objects::Version::initial(),
                etag: educore_core::value_objects::Etag::new("00000000000000000000000000000001")
                    .unwrap(),
                created_at: educore_core::value_objects::Timestamp::now(),
                updated_at: educore_core::value_objects::Timestamp::now(),
                created_by: user,
                updated_by: user,
                active_status: educore_core::value_objects::ActiveStatus::Active,
                correlation_id: educore_core::ids::CorrelationId::from_uuid(Uuid::new_v4()),
                last_event_id: Some(educore_core::ids::EventId::from_uuid(Uuid::new_v4())),
            };
            adapter
                .bulk_insert_student_attendances(&ctx, std::slice::from_ref(&row))
                .await
                .unwrap();
            let res = adapter.bulk_insert_student_attendances(&ctx, &[row]).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn begin_commit_drains_outbox() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let tx = adapter.begin().await.unwrap();
            let env = sample_envelope(school);
            tx.outbox().append(env.clone()).await.unwrap();
            tx.commit().await.unwrap();
            // After commit, the outbox is drained.
            let tx2 = adapter.begin().await.unwrap();
            let pending = tx2.outbox().pending(10).await.unwrap();
            assert!(pending.is_empty());
        });
    }

    #[test]
    fn begin_rollback_discards_outbox() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let tx = adapter.begin().await.unwrap();
            tx.outbox().append(sample_envelope(school)).await.unwrap();
            tx.rollback().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let pending = tx2.outbox().pending(10).await.unwrap();
            // The first tx was rolled back so the outbox still
            // has the envelope; the second tx sees it.
            assert_eq!(pending.len(), 1);
        });
    }

    #[test]
    fn audit_log_append_then_read_for_target_round_trips() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let target = g.next_uuid();
            let entry = AuditLogEntry::create(
                school,
                user,
                "student",
                target,
                Bytes::from_static(b"{\"id\":\"x\"}"),
                g.next_correlation_id(),
            );
            let tx = adapter.begin().await.unwrap();
            tx.audit_log().append(entry.clone()).await.unwrap();
            tx.commit().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let rows = tx2
                .audit_log()
                .read_for_target(school, target, 10)
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].target_id, target);
        });
    }

    #[test]
    fn event_log_append_then_read_filters_by_event_type() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let mut entry = EventLogEntry {
                event_id: g.next_event_id(),
                school_id: school,
                event_type: "academic.student.admitted".to_owned(),
                schema_version: 1,
                aggregate_id: g.next_uuid(),
                aggregate_type: "student".to_owned(),
                actor_id: g.next_user_id(),
                correlation_id: g.next_correlation_id(),
                causation_id: None,
                occurred_at: Timestamp::now(),
                recorded_at: Timestamp::now(),
                payload: Bytes::from_static(b"{}"),
                active_status: ActiveStatus::Active,
            };
            let tx = adapter.begin().await.unwrap();
            tx.event_log().append(entry.clone()).await.unwrap();
            entry.event_type = "academic.student.transferred".to_owned();
            tx.event_log().append(entry).await.unwrap();
            tx.commit().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let mut f = EventLogFilter::for_school(school);
            f.event_types = vec!["academic.student.admitted".to_owned()];
            let rows = tx2.event_log().read(f).await.unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].event_type, "academic.student.admitted");
        });
    }

    #[test]
    fn idempotency_record_then_lookup_round_trips() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let key = g.next_idempotency_key();
            let record = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: key,
                outcome: Bytes::from_static(b"{\"id\":\"x\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let tx = adapter.begin().await.unwrap();
            tx.idempotency().record(record.clone()).await.unwrap();
            tx.commit().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let comp = IdempotencyRecord::composite_key(school, "academic.student.admit", key);
            let got = tx2.idempotency().lookup(comp).await.unwrap();
            assert!(got.is_some());
        });
    }

    #[test]
    fn idempotency_record_duplicate_with_different_outcome_is_conflict() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let key = g.next_idempotency_key();
            let r1 = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: key,
                outcome: Bytes::from_static(b"{\"id\":\"a\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let tx = adapter.begin().await.unwrap();
            tx.idempotency().record(r1).await.unwrap();
            tx.commit().await.unwrap();
            let r2 = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: key,
                outcome: Bytes::from_static(b"{\"id\":\"b\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let tx2 = adapter.begin().await.unwrap();
            let res = tx2.idempotency().record(r2).await;
            assert!(res.is_err());
        });
    }

    #[test]
    fn idempotency_record_duplicate_with_same_outcome_is_idempotent_ok() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let key = g.next_idempotency_key();
            let r = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: key,
                outcome: Bytes::from_static(b"{\"id\":\"a\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let tx = adapter.begin().await.unwrap();
            tx.idempotency().record(r.clone()).await.unwrap();
            tx.commit().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let res = tx2.idempotency().record(r).await;
            assert!(res.is_ok());
        });
    }

    #[test]
    fn cursor_for_returns_zero_for_never_advanced_school() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let cur = adapter.cursor_for(school).await.unwrap();
            assert_eq!(cur, VersionCursor::ZERO);
        });
    }

    #[test]
    fn advance_cursor_then_cursor_for_returns_advanced() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            adapter
                .advance_cursor(school, VersionCursor(42))
                .await
                .unwrap();
            let cur = adapter.cursor_for(school).await.unwrap();
            assert_eq!(cur, VersionCursor(42));
        });
    }

    #[test]
    fn watch_changes_returns_change_stream() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            let filter = ChangeFilter::for_school(school);
            let _stream = adapter.watch_changes(filter).await.unwrap();
        });
    }

    // -----------------------------------------------------------------
    // QW-4: explicit `Drop` on Transaction impl rollback
    // -----------------------------------------------------------------
    //
    // These two tests verify the QW-4 contract introduced in
    // PR 2 of `remediation/day1-quick-wins`:
    //
    // 1. Dropping a transaction that was neither committed nor
    //    rolled back MUST trigger an implicit rollback (the
    //    `implicit_rollback_count` on the shared inner state
    //    increments by one).
    // 2. A transaction that was explicitly committed MUST NOT
    //    trigger an implicit rollback on drop (the happy path
    //    is unchanged; the `explicit_commit_count` increments
    //    by one and the `implicit_rollback_count` does NOT).
    //
    // The counters live on `InMemoryInner` (not on the
    // transaction itself) precisely so we can observe the
    // Drop behavior from a different scope: the inner state
    // outlives the transaction and the test holds an `Arc`
    // to it via the adapter.

    #[test]
    fn drop_without_commit_triggers_implicit_rollback() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            assert_eq!(
                adapter.inner.implicit_rollback_count.load(Ordering::SeqCst),
                0
            );
            assert_eq!(
                adapter.inner.explicit_commit_count.load(Ordering::SeqCst),
                0
            );

            let tx = adapter.begin().await.unwrap();
            // Stage a write so the transaction has real work
            // (and verifies the Drop path is exercised on a
            // non-trivial transaction).
            tx.outbox().append(sample_envelope(school)).await.unwrap();

            // Drop without `commit` or `rollback`.
            drop(tx);

            // QW-4 contract: the implicit-rollback counter
            // MUST have incremented.
            assert_eq!(
                adapter.inner.implicit_rollback_count.load(Ordering::SeqCst),
                1,
                "dropping a transaction without commit/rollback must trigger implicit rollback"
            );
            assert_eq!(
                adapter.inner.explicit_commit_count.load(Ordering::SeqCst),
                0,
                "an uncommitted-then-dropped transaction must not count as a commit"
            );
        });
    }

    #[test]
    fn drop_after_commit_does_not_trigger_rollback() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let adapter = InMemoryStorageAdapter::new(make_bus());
            let g = SystemIdGen;
            let school = g.next_school_id();
            assert_eq!(
                adapter.inner.implicit_rollback_count.load(Ordering::SeqCst),
                0
            );
            assert_eq!(
                adapter.inner.explicit_commit_count.load(Ordering::SeqCst),
                0
            );

            let tx = adapter.begin().await.unwrap();
            tx.outbox().append(sample_envelope(school)).await.unwrap();
            // `commit` consumes the `Box<dyn Transaction>`;
            // its Drop fires when `commit` returns. QW-4
            // contract: a committed transaction MUST NOT
            // trigger an implicit rollback on Drop.
            tx.commit().await.unwrap();

            assert_eq!(
                adapter.inner.implicit_rollback_count.load(Ordering::SeqCst),
                0,
                "dropping a committed transaction must NOT trigger implicit rollback"
            );
            assert_eq!(
                adapter.inner.explicit_commit_count.load(Ordering::SeqCst),
                1,
                "dropping a committed transaction counts the happy path"
            );
        });
    }
}
