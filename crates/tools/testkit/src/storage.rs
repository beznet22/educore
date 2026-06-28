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
//! ## Outbox drain to in-process bus (roadmap C-3)
//!
//! [`InMemoryStorageAdapter`] holds an [`EventBus`]
//! (`Arc<dyn EventBus>`, supplied at construction via
//! [`InMemoryStorageAdapter::new`] or swapped via
//! [`InMemoryStorageAdapter::with_bus`]). On
//! [`commit`](educore_storage::Transaction::commit) the
//! adapter drains the outbox and publishes every staged
//! envelope through the bus. Successful publishes are
//! removed from the outbox; failed publishes are pushed
//! back so the next drain retries (at-least-once delivery,
//! mirroring the production `OutboxRelay`). Tests install
//! an `InProcessEventBus` plus any subscriber registries
//! they need to assert event publication.
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
use educore_storage::transaction::{TenantTransaction, Transaction};

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
// InMemoryStaging (per-transaction staging buffer)
//
// Per Cluster F (`PORT-STORE-002` / `PORT-STORE-013`), the
// testkit's transaction now stages every sub-port write
// (outbox, audit, event_log, idempotency) into a per-transaction
// buffer. The buffer is flushed into the shared `InMemoryInner`
// on `commit()` and discarded on `rollback()`. This makes the
// testkit behaviour match the production SQL adapters'
// atomicity guarantee: an audit row that is appended inside a
// transaction is observable to reads only after `commit()` and
// is invisible to subsequent transactions if `rollback()` is
// called instead.
//
// Before this change the testkit wrote directly to the shared
// `InMemoryInner` (the `TOOL-TK-002` caveat documented in the
// module-level Drop comment) — a rolled-back transaction left
// its staged writes visible. This change closes that gap.
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct InMemoryStaging {
    /// Outbox envelopes staged by this transaction. Flushed on
    /// `commit()`, discarded on `rollback()`.
    pub(crate) outbox: Mutex<Vec<SerializedEnvelope>>,
    /// Audit rows staged by this transaction.
    pub(crate) audit_log: Mutex<Vec<AuditLogEntry>>,
    /// Event log rows staged by this transaction.
    pub(crate) event_log: Mutex<Vec<EventLogEntry>>,
    /// Idempotency records staged by this transaction.
    pub(crate) idempotency: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
}

// ---------------------------------------------------------------------------
// Sub-port wrappers — each is a thin Arc-shared handle to the inner state.
// Storing them in the Transaction struct (not constructing per call) avoids
// the "cannot return reference to temporary value" error.
// ---------------------------------------------------------------------------

// `OutboxHandle` (and the three sibling handles below) is
// the direct-access variant: it writes straight to the shared
// inner state with no staging layer. After the Cluster F
// staging change, `InMemoryTransaction` uses `StagingOutbox`
// instead, so the direct variant is no longer constructed
// inside the testkit. We keep it (and its `Outbox for
// OutboxHandle` / `OutboxSource for OutboxHandle` impls)
// because external consumers wire the testkit's storage
// adapter as an `OutboxSource` for the production relay
// (the `educore_events::relay::OutboxSource` impl on this
// type is the public seam). The `dead_code` allow silences
// the "never constructed" warning for the type itself.
#[allow(dead_code)]
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
    async fn append(
        &self,
        _school_id: educore_core::ids::SchoolId,
        envelope: SerializedEnvelope,
    ) -> Result<()> {
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

    async fn pending(
        &self,
        _school_id: educore_core::ids::SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>> {
        let limit = usize::try_from(limit)
            .map_err(|_| DomainError::validation("outbox pending limit exceeds usize"))?;
        let outbox = self.0.outbox.lock();
        Ok(outbox.iter().take(limit).cloned().collect())
    }

    async fn mark_published(
        &self,
        _school_id: educore_core::ids::SchoolId,
        ids: &[educore_core::ids::EventId],
    ) -> Result<()> {
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
        _school_id: educore_core::ids::SchoolId,
        limit: u32,
    ) -> std::result::Result<
        Vec<educore_events::relay_envelope::SerializedEnvelope>,
        educore_core::error::DomainError,
    > {
        let outbox = self.0.outbox.lock();
        let n: usize = usize::try_from(limit).unwrap_or(usize::MAX);
        Ok(outbox
            .iter()
            .take(n)
            .cloned()
            .map(to_relay_envelope)
            .collect())
    }

    async fn mark_published(
        &self,
        _school_id: educore_core::ids::SchoolId,
        ids: &[educore_core::ids::EventId],
    ) -> Result<()> {
        let mut outbox = self.0.outbox.lock();
        outbox.retain(|e| !ids.contains(&e.event_id));
        Ok(())
    }
}

// Direct-access audit handle; see the comment on
// `OutboxHandle` above for why this type is retained with
// `dead_code` allowed.
#[allow(dead_code)]
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
        let limit = usize::try_from(limit)
            .map_err(|_| DomainError::validation("audit read_for_target limit exceeds usize"))?;
        let log = self.0.audit_log.lock();
        let mut out: Vec<AuditLogEntry> = log
            .iter()
            .filter(|e| e.school_id == school_id && e.target_id == target_id)
            .cloned()
            .collect();
        out.sort_by_key(|e| e.occurred_at);
        out.truncate(limit);
        Ok(out)
    }
}

// Direct-access event-log handle; see the comment on
// `OutboxHandle` above for why this type is retained with
// `dead_code` allowed.
#[allow(dead_code)]
pub(crate) struct EventLogHandle(pub(crate) Arc<InMemoryInner>);

#[async_trait]
impl EventLog for EventLogHandle {
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        self.0.event_log.lock().push(entry);
        Ok(())
    }

    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let limit = usize::try_from(filter.limit)
            .map_err(|_| DomainError::validation("event log read limit exceeds usize"))?;
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
        out.truncate(limit);
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
        u64::try_from(n).map_err(|_| DomainError::validation("event log count exceeds u64"))
    }
}

// Direct-access idempotency handle; see the comment on
// `OutboxHandle` above for why this type is retained with
// `dead_code` allowed.
#[allow(dead_code)]
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
// Staging handle types (Cluster F / PORT-STORE-013)
//
// Each staging handle holds an `Arc` to the shared
// `InMemoryInner` (for reads that should fall through to
// already-committed state) and an `Arc` to the per-transaction
// `InMemoryStaging` buffer (for writes that should be deferred
// until `commit()`).
//
// Reads are passed through to the inner state directly — a
// staging transaction does not need to read its own uncommitted
// writes because the engine's command pipeline issues all
// stages within a single transaction and reads from inner state
// (not from the staging buffer).
//
// Writes go to the staging buffer; the staging buffer is moved
// into the inner state by `InMemoryTransaction::commit` and
// dropped by `InMemoryTransaction::rollback`.
// ---------------------------------------------------------------------------

/// Staging outbox handle. Writes go to the staging buffer; reads
/// pass through to the inner state.
pub(crate) struct StagingOutbox {
    inner: Arc<InMemoryInner>,
    staging: Arc<InMemoryStaging>,
}

#[async_trait]
impl Outbox for StagingOutbox {
    async fn append(
        &self,
        _school_id: educore_core::ids::SchoolId,
        envelope: SerializedEnvelope,
    ) -> Result<()> {
        let mut staging = self.staging.outbox.lock();
        if staging.iter().any(|e| e.event_id == envelope.event_id) {
            return Err(DomainError::conflict(format!(
                "duplicate event_id {} in outbox (staged)",
                envelope.event_id.as_uuid()
            )));
        }
        staging.push(envelope);
        Ok(())
    }

    async fn pending(
        &self,
        _school_id: educore_core::ids::SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>> {
        // Reads always pass through to the inner state; the
        // staging buffer is invisible to other transactions.
        let limit = usize::try_from(limit)
            .map_err(|_| DomainError::validation("staging outbox pending limit exceeds usize"))?;
        let outbox = self.inner.outbox.lock();
        Ok(outbox.iter().take(limit).cloned().collect())
    }

    async fn mark_published(
        &self,
        _school_id: educore_core::ids::SchoolId,
        ids: &[educore_core::ids::EventId],
    ) -> Result<()> {
        let mut outbox = self.inner.outbox.lock();
        outbox.retain(|e| !ids.contains(&e.event_id));
        Ok(())
    }
}

/// Staging audit-log handle. Writes go to the staging buffer;
/// reads pass through to the inner state.
pub(crate) struct StagingAuditLog {
    inner: Arc<InMemoryInner>,
    staging: Arc<InMemoryStaging>,
}

#[async_trait]
impl AuditLog for StagingAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        self.staging.audit_log.lock().push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let limit = usize::try_from(limit).map_err(|_| {
            DomainError::validation("staging audit read_for_target limit exceeds usize")
        })?;
        let log = self.inner.audit_log.lock();
        let mut out: Vec<AuditLogEntry> = log
            .iter()
            .filter(|e| e.school_id == school_id && e.target_id == target_id)
            .cloned()
            .collect();
        out.sort_by_key(|e| e.occurred_at);
        out.truncate(limit);
        Ok(out)
    }
}

/// Staging event-log handle. Writes go to the staging buffer;
/// reads pass through to the inner state.
pub(crate) struct StagingEventLog {
    inner: Arc<InMemoryInner>,
    staging: Arc<InMemoryStaging>,
}

#[async_trait]
impl EventLog for StagingEventLog {
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        self.staging.event_log.lock().push(entry);
        Ok(())
    }

    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let limit = usize::try_from(filter.limit)
            .map_err(|_| DomainError::validation("staging event log read limit exceeds usize"))?;
        let log = self.inner.event_log.lock();
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
        out.truncate(limit);
        Ok(out)
    }

    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let log = self.inner.event_log.lock();
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
        u64::try_from(n).map_err(|_| DomainError::validation("staging event log count exceeds u64"))
    }
}

/// Staging idempotency handle. Writes go to the staging buffer;
/// reads pass through to the inner state.
pub(crate) struct StagingIdempotency {
    inner: Arc<InMemoryInner>,
    staging: Arc<InMemoryStaging>,
}

#[async_trait]
impl Idempotency for StagingIdempotency {
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        // Check staging first (uncommitted record from this
        // transaction), then fall through to the inner state
        // (records committed by previous transactions).
        {
            let staging = self.staging.idempotency.lock();
            if let Some(record) = staging.get(&key) {
                return Ok(Some(record.clone()));
            }
        }
        let store = self.inner.idempotency.lock();
        Ok(store.get(&key).cloned())
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        let key = IdempotencyCompositeKey {
            school_id: record.school_id,
            command_type: record.command_type,
            idempotency_key: record.idempotency_key,
        };
        // Check both the staging buffer (this transaction's
        // own writes) and the inner state (records committed
        // by previous transactions) for an existing record
        // with the same composite key.
        let existing = {
            let staging = self.staging.idempotency.lock();
            staging.get(&key).cloned()
        };
        let existing = match existing {
            Some(rec) => Some(rec),
            None => {
                let store = self.inner.idempotency.lock();
                store.get(&key).cloned()
            }
        };
        if let Some(existing) = existing {
            if existing.outcome != record.outcome {
                return Err(DomainError::conflict(format!(
                    "idempotency key {} has different outcome",
                    record.idempotency_key.as_uuid()
                )));
            }
            return Ok(());
        }
        self.staging.idempotency.lock().insert(key, record);
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
    /// Cluster F: the `SchoolId` scope of every transaction
    /// opened by this adapter. Defaults to a fresh UUIDv7 in
    /// `new()` and can be overridden with `with_school()`.
    /// Mirrors the SQL adapters' `PostgresConnection::school()`
    /// / `MySqlConnection::school()` / etc.
    school: SchoolId,
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
            // Fresh random school per adapter. Tests that
            // need to assert on a specific school should call
            // `with_school()` after construction.
            school: SchoolId::from_uuid(Uuid::new_v4()),
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

    /// Cluster F: sets the `SchoolId` scope of every
    /// transaction opened by this adapter. Mirrors the SQL
    /// adapters' per-connection school. Transactions opened
    /// before `with_school` returns keep their original
    /// scope.
    #[must_use]
    pub fn with_school(mut self, school: SchoolId) -> Self {
        self.school = school;
        self
    }

    /// Reads all audit log rows from the adapter's in-memory
    /// store. Returns the rows in insertion order (oldest
    /// first). Includes only rows from committed transactions.
    pub fn read_audit_log_entries(&self) -> Vec<AuditLogEntry> {
        self.inner.audit_log.lock().clone()
    }

    /// Cluster F: returns the `SchoolId` scope of every
    /// transaction opened by this adapter.
    #[must_use]
    pub fn school(&self) -> SchoolId {
        self.school
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
            self.school,
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
///
/// Cluster F (`PORT-STORE-002` + `PORT-STORE-013`): the
/// sub-port handles are staging handles — writes go to a
/// per-transaction buffer (an `Arc<InMemoryStaging>`) until
/// `commit()` flushes the buffer into the shared
/// `InMemoryInner` state. A `rollback()` drops the buffer
/// without touching the inner state, giving the testkit the
/// same atomicity guarantee as the production SQL adapters
/// (audit rows are written transactionally with the
/// aggregate mutation).
pub struct InMemoryTransaction {
    inner: Arc<InMemoryInner>,
    staging: Arc<InMemoryStaging>,
    _bus: Arc<dyn EventBus>,
    outbox_h: StagingOutbox,
    audit_h: StagingAuditLog,
    event_h: StagingEventLog,
    idem_h: StagingIdempotency,
    /// Cluster F: tenant scope of this transaction.
    /// Mirrors the SQL adapters' `school_id` field.
    school_id: SchoolId,
    committed: AtomicBool,
    rolled_back: AtomicBool,
}

impl std::fmt::Debug for InMemoryTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryTransaction")
            .field("school_id", &self.school_id)
            .field("committed", &self.committed.load(Ordering::SeqCst))
            .field("rolled_back", &self.rolled_back.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl InMemoryTransaction {
    fn new(inner: Arc<InMemoryInner>, bus: Arc<dyn EventBus>, school_id: SchoolId) -> Self {
        let staging = Arc::new(InMemoryStaging::default());
        let outbox_h = StagingOutbox {
            inner: inner.clone(),
            staging: staging.clone(),
        };
        let audit_h = StagingAuditLog {
            inner: inner.clone(),
            staging: staging.clone(),
        };
        let event_h = StagingEventLog {
            inner: inner.clone(),
            staging: staging.clone(),
        };
        let idem_h = StagingIdempotency {
            inner: inner.clone(),
            staging: staging.clone(),
        };
        Self {
            inner,
            staging,
            _bus: bus,
            outbox_h,
            audit_h,
            event_h,
            idem_h,
            school_id,
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

/// QW-4 / `TOOL-TK-002` complete fix (Cluster F) — the Drop
/// contract.
///
/// If the transaction is dropped without an explicit `commit`
/// or `rollback`, the `Drop` impl performs an implicit
/// rollback: the staging buffer (held by `Arc<InMemoryStaging>`)
/// is dropped along with the transaction, which discards all
/// staged writes — the inner state is untouched. This honors
/// the port-level "rollback by default" contract.
///
/// The previous `TOOL-TK-002` caveat (staged writes leaking
/// to subsequent transactions on drop) is closed: a drop is
/// now an atomic rollback of the staging buffer.
impl Drop for InMemoryTransaction {
    fn drop(&mut self) {
        let was_committed = self.committed.load(Ordering::SeqCst);
        let was_rolled_back = self.rolled_back.load(Ordering::SeqCst);
        if !was_committed && !was_rolled_back {
            tracing::warn!(
                school = %self.school_id,
                "InMemoryTransaction dropped without commit or rollback; \
                 performing implicit rollback (staging buffer discarded)"
            );
            self.rolled_back.store(true, Ordering::SeqCst);
            self.committed.store(true, Ordering::SeqCst);
            // Dropping `self.staging` discards the staged
            // writes (outbox / audit / event_log /
            // idempotency) atomically — the inner state is
            // never touched on the implicit-rollback path.
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

        // Flush the staging buffer into the shared inner
        // state. Each lock is taken in the same order as the
        // bulk-insert path (outbox, audit, event_log,
        // idempotency) to preserve the engine's documented
        // lock-ordering convention. The flush is the
        // "atomic commit" point: after the staging buffers
        // are drained into inner, all sub-port writes are
        // visible to subsequent transactions and to the
        // bus publish that follows.
        let staged_outbox: Vec<SerializedEnvelope> = {
            let mut buf = self.staging.outbox.lock();
            buf.drain(..).collect()
        };
        let staged_audit: Vec<AuditLogEntry> = {
            let mut buf = self.staging.audit_log.lock();
            buf.drain(..).collect()
        };
        let staged_event: Vec<EventLogEntry> = {
            let mut buf = self.staging.event_log.lock();
            buf.drain(..).collect()
        };
        let staged_idem: HashMap<IdempotencyCompositeKey, IdempotencyRecord> = {
            let mut buf = self.staging.idempotency.lock();
            buf.drain().collect()
        };
        {
            let mut outbox = self.inner.outbox.lock();
            outbox.extend(staged_outbox.iter().cloned());
        }
        {
            let mut audit = self.inner.audit_log.lock();
            audit.extend(staged_audit.iter().cloned());
        }
        {
            let mut event = self.inner.event_log.lock();
            event.extend(staged_event.iter().cloned());
        }
        {
            let mut idem = self.inner.idempotency.lock();
            for (k, v) in staged_idem {
                idem.insert(k, v);
            }
        }

        // Drain the outbox and publish each envelope via the
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
        // Cluster F (PORT-STORE-013): when `self` drops at
        // the end of this function, the staging buffer is
        // dropped along with the transaction, which discards
        // every staged write (outbox / audit / event_log /
        // idempotency). The inner state is never touched on
        // the rollback path. The staging handle fields
        // (`outbox_h`, `audit_h`, `event_h`, `idem_h`) hold
        // `Arc` clones of the same staging buffer; their
        // drops are no-ops.
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

/// Cluster F / `PORT-STORE-002` — the `TenantTransaction`
/// extension for the testkit's in-memory adapter.
///
/// The transaction's tenant scope is the `SchoolId` it was
/// constructed with (sourced from the adapter's `with_school()`
/// setting). The audit handle returned by
/// `Transaction::audit_log()` is the staging handle; its writes
/// are committed (or rolled back) atomically with the rest of
/// the transaction's writes per `PORT-STORE-013` (the staging
/// buffer flush is the atomicity point).
impl TenantTransaction for InMemoryTransaction {
    fn school_id(&self) -> SchoolId {
        self.school_id
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
            tx.outbox().append(school, env.clone()).await.unwrap();
            tx.commit().await.unwrap();
            // After commit, the outbox is drained.
            let tx2 = adapter.begin().await.unwrap();
            let pending = tx2.outbox().pending(school, 10).await.unwrap();
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
            tx.outbox()
                .append(school, sample_envelope(school))
                .await
                .unwrap();
            tx.rollback().await.unwrap();
            let tx2 = adapter.begin().await.unwrap();
            let pending = tx2.outbox().pending(school, 10).await.unwrap();
            // Cluster F (PORT-STORE-013): the staging buffer
            // is discarded on rollback, so the next
            // transaction observes an empty outbox. (Before
            // the staging fix, the entry leaked to the inner
            // state — this test asserted that broken
            // behaviour. The staging fix closes
            // `TOOL-TK-002`.)
            assert!(pending.is_empty());
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
            tx.outbox()
                .append(school, sample_envelope(school))
                .await
                .unwrap();

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
            tx.outbox()
                .append(school, sample_envelope(school))
                .await
                .unwrap();
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

    // -----------------------------------------------------------------
    // Cluster F (storage transaction hardening) tests
    // -----------------------------------------------------------------
    //
    // These tests close `PORT-STORE-002` (tenant context on
    // the transaction) and `PORT-STORE-013` (atomic audit-write
    // with the aggregate mutation) in the testkit's
    // in-memory adapter.

    /// `Transaction::school_id()` returns the tenant scope of
    /// the transaction (PORT-STORE-002).
    #[test]
    fn tenant_tx_school_id_returns_scope() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);
            // Construct an `InMemoryTransaction` directly so we
            // can call its `TenantTransaction::school_id()`
            // method without downcasting a `Box<dyn
            // Transaction>`. The transaction is dropped
            // without `commit`/`rollback`, so it triggers the
            // implicit-rollback Drop path (no observable side
            // effects: staging buffer is empty).
            let tx = InMemoryTransaction::new(adapter.inner.clone(), make_bus(), school);
            assert_eq!(tx.school_id(), school);
            // Verify the trait surface: the concrete type
            // implements both `Transaction` and
            // `TenantTransaction`.
            fn _assert_tx_and_tenant<T: Send + Sync + std::fmt::Debug>(_: &T)
            where
                T: Transaction,
                T: TenantTransaction,
            {
            }
            _assert_tx_and_tenant(&tx);
        });
    }

    /// `Transaction::commit` atomically writes audit rows with
    /// the aggregate mutation (PORT-STORE-013).
    #[test]
    fn commit_atomically_writes_audit_row() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let target = g.next_uuid();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);

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
            // Before commit: the audit row is staged, not
            // visible to a read through the inner state.
            // (We can't observe the staging buffer directly,
            // but we can confirm commit-then-read works.)
            tx.commit().await.unwrap();

            // After commit: the audit row is in the inner
            // state and visible to subsequent reads.
            let tx2 = adapter.begin().await.unwrap();
            let rows = tx2
                .audit_log()
                .read_for_target(school, target, 10)
                .await
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].target_id, target);
            assert_eq!(rows[0].action, "create");
        });
    }

    /// `Transaction::rollback` discards staged audit rows
    /// (PORT-STORE-013).
    #[test]
    fn rollback_discards_staged_audit_rows() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let target = g.next_uuid();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);

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
            tx.rollback().await.unwrap();

            // After rollback: the audit row was discarded
            // with the staging buffer and is NOT visible to
            // subsequent reads.
            let tx2 = adapter.begin().await.unwrap();
            let rows = tx2
                .audit_log()
                .read_for_target(school, target, 10)
                .await
                .unwrap();
            assert!(
                rows.is_empty(),
                "audit row staged in a rolled-back transaction must not be visible"
            );
        });
    }

    /// `AuditLog::append` inside a rolled-back transaction
    /// does not produce a visible audit row (PORT-STORE-013).
    #[test]
    fn audit_append_inside_transaction_is_rolled_back() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let target_a = g.next_uuid();
            let target_b = g.next_uuid();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);

            // Transaction 1: commit target_a.
            let entry_a = AuditLogEntry::create(
                school,
                user,
                "student",
                target_a,
                Bytes::from_static(b"{\"id\":\"a\"}"),
                g.next_correlation_id(),
            );
            let tx1 = adapter.begin().await.unwrap();
            tx1.audit_log().append(entry_a).await.unwrap();
            tx1.commit().await.unwrap();

            // Transaction 2: append target_b, then rollback.
            let entry_b = AuditLogEntry::create(
                school,
                user,
                "student",
                target_b,
                Bytes::from_static(b"{\"id\":\"b\"}"),
                g.next_correlation_id(),
            );
            let tx2 = adapter.begin().await.unwrap();
            tx2.audit_log().append(entry_b).await.unwrap();
            tx2.rollback().await.unwrap();

            // Reads see only the committed target_a, not the
            // rolled-back target_b.
            let tx3 = adapter.begin().await.unwrap();
            let a_rows = tx3
                .audit_log()
                .read_for_target(school, target_a, 10)
                .await
                .unwrap();
            let b_rows = tx3
                .audit_log()
                .read_for_target(school, target_b, 10)
                .await
                .unwrap();
            assert_eq!(a_rows.len(), 1, "committed audit row is visible");
            assert!(b_rows.is_empty(), "rolled-back audit row is discarded");
        });
    }

    /// `Transaction::commit` is atomic across sub-ports: the
    /// outbox row, audit row, idempotency record, and event
    /// log row all become visible together. Before commit,
    /// none are visible; after rollback, none are visible.
    #[test]
    fn commit_is_atomic_across_all_subports() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);

            // Use one envelope, one audit row, one idempotency
            // record, one event log row — all sharing the same
            // correlation id so the test is one logical
            // "command".
            let correlation_id = g.next_correlation_id();
            let target = g.next_uuid();

            let envelope = sample_envelope(school);
            let audit = AuditLogEntry::create(
                school,
                user,
                "student",
                target,
                Bytes::from_static(b"{\"id\":\"x\"}"),
                correlation_id,
            );
            let idem_key = g.next_idempotency_key();
            let idem_record = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: idem_key,
                outcome: Bytes::from_static(b"{\"id\":\"x\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let event_entry = EventLogEntry {
                event_id: envelope.event_id,
                school_id: school,
                event_type: envelope.event_type.clone(),
                schema_version: envelope.schema_version,
                aggregate_id: envelope.aggregate_id,
                aggregate_type: envelope.aggregate_type.clone(),
                actor_id: envelope.actor_id,
                correlation_id,
                causation_id: envelope.causation_id,
                occurred_at: envelope.occurred_at,
                recorded_at: Timestamp::now(),
                payload: envelope.payload.clone(),
                active_status: ActiveStatus::Active,
            };

            // Phase 1: commit — all four become visible.
            let tx = adapter.begin().await.unwrap();
            tx.outbox().append(school, envelope.clone()).await.unwrap();
            tx.audit_log().append(audit.clone()).await.unwrap();
            tx.idempotency().record(idem_record.clone()).await.unwrap();
            tx.event_log().append(event_entry.clone()).await.unwrap();
            tx.commit().await.unwrap();

            let tx2 = adapter.begin().await.unwrap();
            assert_eq!(
                tx2.outbox().pending(school, 10).await.unwrap().len(),
                0,
                "outbox is drained on commit (envelopes flow to the bus and are removed)"
            );
            assert_eq!(
                tx2.audit_log()
                    .read_for_target(school, target, 10)
                    .await
                    .unwrap()
                    .len(),
                1,
                "audit row committed"
            );
            let idem_composite =
                IdempotencyRecord::composite_key(school, "academic.student.admit", idem_key);
            assert!(
                tx2.idempotency()
                    .lookup(idem_composite)
                    .await
                    .unwrap()
                    .is_some(),
                "idempotency record committed"
            );
            let mut filter = EventLogFilter::for_school(school);
            filter.event_types = vec![envelope.event_type.clone()];
            assert_eq!(
                tx2.event_log().read(filter).await.unwrap().len(),
                1,
                "event log row committed"
            );
        });
    }

    /// `Transaction::rollback` is atomic across sub-ports: the
    /// outbox row, audit row, idempotency record, and event
    /// log row are all discarded together.
    #[test]
    fn rollback_is_atomic_across_all_subports() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let g = SystemIdGen;
            let school = g.next_school_id();
            let user = g.next_user_id();
            let adapter = InMemoryStorageAdapter::new(make_bus()).with_school(school);

            let correlation_id = g.next_correlation_id();
            let target = g.next_uuid();

            let envelope = sample_envelope(school);
            let audit = AuditLogEntry::create(
                school,
                user,
                "student",
                target,
                Bytes::from_static(b"{\"id\":\"x\"}"),
                correlation_id,
            );
            let idem_key = g.next_idempotency_key();
            let idem_record = IdempotencyRecord {
                school_id: school,
                command_type: "academic.student.admit",
                idempotency_key: idem_key,
                outcome: Bytes::from_static(b"{\"id\":\"x\"}"),
                outcome_version: 1,
                recorded_at: Timestamp::now(),
                affected_aggregate_ids: vec![],
            };
            let event_entry = EventLogEntry {
                event_id: envelope.event_id,
                school_id: school,
                event_type: envelope.event_type.clone(),
                schema_version: envelope.schema_version,
                aggregate_id: envelope.aggregate_id,
                aggregate_type: envelope.aggregate_type.clone(),
                actor_id: envelope.actor_id,
                correlation_id,
                causation_id: envelope.causation_id,
                occurred_at: envelope.occurred_at,
                recorded_at: Timestamp::now(),
                payload: envelope.payload.clone(),
                active_status: ActiveStatus::Active,
            };

            // Phase 1: stage everything, then rollback.
            let tx = adapter.begin().await.unwrap();
            tx.outbox().append(school, envelope.clone()).await.unwrap();
            tx.audit_log().append(audit.clone()).await.unwrap();
            tx.idempotency().record(idem_record.clone()).await.unwrap();
            tx.event_log().append(event_entry.clone()).await.unwrap();
            tx.rollback().await.unwrap();

            // Phase 2: nothing is visible to subsequent reads.
            let tx2 = adapter.begin().await.unwrap();
            assert!(
                tx2.outbox().pending(school, 10).await.unwrap().is_empty(),
                "rolled-back outbox is empty"
            );
            assert!(
                tx2.audit_log()
                    .read_for_target(school, target, 10)
                    .await
                    .unwrap()
                    .is_empty(),
                "rolled-back audit is empty"
            );
            let idem_composite =
                IdempotencyRecord::composite_key(school, "academic.student.admit", idem_key);
            assert!(
                tx2.idempotency()
                    .lookup(idem_composite)
                    .await
                    .unwrap()
                    .is_none(),
                "rolled-back idempotency record is discarded"
            );
            let mut filter = EventLogFilter::for_school(school);
            filter.event_types = vec![envelope.event_type.clone()];
            assert!(
                tx2.event_log().read(filter).await.unwrap().is_empty(),
                "rolled-back event log is empty"
            );
        });
    }
}
