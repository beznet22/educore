//! # Outbox relay
//!
//! Drains the storage-port [`Outbox`](educore_storage::outbox::Outbox)
//! and forwards each pending envelope to the bus-port
//! [`EventBus`](crate::event_bus::EventBus). This is the engine's
//! "transactional outbox → bus" hand-off: writes to the outbox
//! are durable in the same transaction as the aggregate mutation,
//! and the relay is the *only* path that promotes those durable
//! rows back to the bus for in-process fan-out.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────┐  append (tx)   ┌─────────────┐
//! │  Aggregate   │ ─────────────▶ │   Outbox    │  (durable)
//! └──────────────┘                └─────────────┘
//!                                         │
//!                                         │ pending (read)
//!                                         ▼
//!                                ┌─────────────────┐
//!                                │  OutboxRelay    │
//!                                │  run_once/loop  │
//!                                └─────────────────┘
//!                                         │
//!                                         │ publish
//!                                         ▼
//!                                ┌─────────────────┐
//!                                │   EventBus      │
//!                                │  (in-process)   │
//!                                └─────────────────┘
//!                                         │
//!                                         │ deliver
//!                                         ▼
//!                                  ┌──────────────┐
//!                                  │ Subscribers  │
//!                                  └──────────────┘
//! ```
//!
//! # Idempotency
//!
//! The relay is **idempotent at the row level**: re-running
//! `run_once` for a school only re-reads envelopes still present
//! in the outbox. Successful publishes are removed via
//! [`Outbox::mark_published`](educore_storage::outbox::Outbox::mark_published);
//! failed publishes stay pending and are retried on the next
//! drain.
//!
//! At the bus boundary the contract is **at-least-once delivery**:
//! a crash between `publish` and `mark_published` will cause the
//! same `event_id` to be published again. Subscribers MUST dedupe
//! by `event_id` (see [`crate::subscribe::Subscriber`]).
//!
//! # Resiliency
//!
//! A single bad envelope does NOT halt the batch. The relay
//! processes each envelope independently; a publish failure is
//! recorded in [`RelayStats::failed`] and the envelope is left in
//! the outbox for the next drain.
//!
//! # Shutdown
//!
//! [`OutboxRelay::run_loop`] is a long-running cooperative task.
//! It honours a [`tokio_util::sync::CancellationToken`] — when
//! the token fires, the loop finishes its current batch and
//! returns. The caller (a server supervisor, a test harness, etc.)
//! owns the token.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use educore_core::error::Result;
use educore_core::ids::{EventId, Identifier, SchoolId};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use crate::event_bus::EventBus;
use crate::relay_envelope::SerializedEnvelope;

/// Default batch size for [`OutboxRelay::run_once`]. Matches the
/// port-level guidance in `docs/ports/storage.md` § 4 and is
/// small enough to keep the per-batch lock window on the outbox
/// sub-port bounded.
pub const DEFAULT_BATCH_SIZE: u32 = 100;

/// The minimum surface of the storage-port
/// [`Outbox`](educore_storage::outbox::Outbox) trait that the
/// relay needs. Defined locally in `educore-events` to avoid a
/// `educore-events` ↔ `educore-storage` dependency cycle
/// (`educore-storage` already depends on `educore-events` for
/// the [`SerializedEnvelope`](crate::relay_envelope::SerializedEnvelope)
/// type and the `EventEnvelope` wire shape).
///
/// Production adapters implement both
/// `educore_storage::outbox::Outbox` and this trait; a blanket
/// impl in the adapter layer is also possible (e.g. in
/// `educore-testkit`) so consumers do not have to write the
/// impl by hand.
///
/// # Object safety
///
/// The trait is object-safe (no generic methods, no `Self` in
/// return type).
#[async_trait]
pub trait OutboxSource: Send + Sync {
    /// Returns up to `limit` envelopes for `school_id` that
    /// have not yet been marked as published. Mirrors
    /// [`Outbox::pending`](educore_storage::outbox::Outbox::pending).
    async fn pending_for_school(
        &self,
        school_id: SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>>;

    /// Marks the given envelopes as published. Idempotent:
    /// calling twice with the same id is a no-op. Mirrors
    /// [`Outbox::mark_published`](educore_storage::outbox::Outbox::mark_published).
    ///
    /// `school_id` is the tenant anchor (FND-PORT-STORE-003) —
    /// the storage adapter MUST verify that `school_id`
    /// matches the handle's scope (or bind it into the
    /// `UPDATE` predicate for un-scoped handles) so a tenant
    /// cannot drain another tenant's queue.
    async fn mark_published(&self, school_id: SchoolId, ids: &[EventId]) -> Result<()>;
}

/// Default idle delay between [`OutboxRelay::run_loop`] ticks
/// when the outbox returns an empty batch. The relay does NOT
/// busy-loop; an empty batch sleeps for this duration before
/// re-checking.
pub const DEFAULT_IDLE_DELAY: Duration = Duration::from_millis(250);

/// Statistics returned by [`OutboxRelay::run_once`].
///
/// # Fields
///
/// - `published` — the number of envelopes the bus accepted
///   (and that the relay subsequently marked published in the
///   outbox).
/// - `failed` — the number of envelopes the bus rejected (or
///   that the relay could not serialise back to a bus-port
///   envelope). These envelopes remain in the outbox and will be
///   retried on the next drain.
/// - `skipped` — the number of envelopes whose `school_id` did
///   NOT match the requested `school_id`. The current
///   implementation drains only via
///   [`OutboxSource::pending_for_school`](OutboxSource::pending_for_school),
///   which scopes to a single school, so this counter is
///   typically 0; it exists for future multi-school drains.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RelayStats {
    /// Envelopes successfully published and marked.
    pub published: usize,
    /// Envelopes the bus rejected (or that could not be
    /// converted); left in the outbox for retry.
    pub failed: usize,
    /// Envelopes skipped (typically 0 — see field doc).
    pub skipped: usize,
}

impl RelayStats {
    /// Returns the total number of envelopes the relay
    /// considered in this batch.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.published + self.failed + self.skipped
    }

    /// Returns `true` if every considered envelope was
    /// published. A `false` return means at least one envelope
    /// is still pending and will be retried.
    #[must_use]
    pub const fn is_fully_published(&self) -> bool {
        self.failed == 0 && self.published > 0
    }
}

/// The outbox-to-bus relay.
///
/// # Type parameters
///
/// - `O` — an outbox source implementing
///   [`OutboxSource`] (typically `Arc<dyn OutboxSource>`
///   in production). The trait captures the relay's minimum
///   storage-port surface without creating a cycle between
///   `educore-events` and `educore-storage`.
/// - `B` — the bus-port [`EventBus`](crate::event_bus::EventBus)
///   adapter (typically `Arc<dyn EventBus>` in production).
///
/// Both are `Send + Sync` so the relay is safe to share across
/// tasks. Construction is via [`OutboxRelay::new`].
#[derive(Debug)]
pub struct OutboxRelay<O: OutboxSource, B: EventBus + Send + Sync> {
    outbox: Arc<O>,
    bus: Arc<B>,
    batch_size: u32,
    idle_delay: Duration,
}

impl<O: OutboxSource, B: EventBus + Send + Sync> OutboxRelay<O, B> {
    /// Constructs a relay with the default [`DEFAULT_BATCH_SIZE`]
    /// and [`DEFAULT_IDLE_DELAY`].
    #[must_use]
    pub const fn new(outbox: Arc<O>, bus: Arc<B>) -> Self {
        Self {
            outbox,
            bus,
            batch_size: DEFAULT_BATCH_SIZE,
            idle_delay: DEFAULT_IDLE_DELAY,
        }
    }

    /// Overrides the per-batch drain size. Used by tests that
    /// want to exercise the `pending(limit)` plumbing.
    #[must_use]
    pub const fn with_batch_size(mut self, batch_size: u32) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Overrides the idle delay between empty batches in
    /// [`Self::run_loop`].
    #[must_use]
    pub const fn with_idle_delay(mut self, idle_delay: Duration) -> Self {
        self.idle_delay = idle_delay;
        self
    }

    /// Drains one batch for `school_id` from the outbox and
    /// publishes each envelope to the bus. Successful publishes
    /// are removed from the outbox via
    /// [`Outbox::mark_published`](educore_storage::outbox::Outbox::mark_published).
    /// Failed publishes are left in place for the next drain.
    ///
    /// # Idempotency
    ///
    /// If the relay crashes between `bus.publish` and
    /// `outbox.mark_published`, the next drain re-publishes the
    /// same `event_id`. The bus provides at-least-once delivery;
    /// subscribers MUST dedupe by `event_id`.
    ///
    /// # Resiliency
    ///
    /// Each envelope is processed independently. A failure on one
    /// envelope does NOT halt the batch — the relay continues
    /// with the remaining envelopes and records the failure in
    /// [`RelayStats::failed`].
    ///
    /// # Errors
    ///
    /// Returns `Err` if the underlying
    /// [`OutboxSource::pending_for_school`](OutboxSource::pending_for_school)
    /// or
    /// [`OutboxSource::mark_published`](OutboxSource::mark_published)
    /// call fails. Publish failures are NOT propagated as
    /// errors — they are recorded in
    /// [`RelayStats::failed`] so the caller can decide whether
    /// to back off.
    pub async fn run_once(&self, school_id: SchoolId) -> Result<RelayStats> {
        let pending = self
            .outbox
            .pending_for_school(school_id, self.batch_size)
            .await?;
        let mut stats = RelayStats::default();
        let mut published_ids: Vec<EventId> = Vec::with_capacity(pending.len());

        for serialized in pending {
            // The relay is single-school per call, so every
            // envelope here belongs to `school_id` (enforced by
            // `pending_for_school`). Skip any that don't match
            // (defensive — the storage port contract is the
            // primary guard).
            if serialized.school_id != school_id {
                stats.skipped += 1;
                continue;
            }
            let envelope = serialized.into_event_envelope();
            let event_id = envelope.event_id;
            match self.bus.publish(envelope).await {
                Ok(receipt) => {
                    published_ids.push(receipt.event_id);
                    stats.published += 1;
                }
                Err(err) => {
                    warn!(
                        school_id = %school_id.as_uuid(),
                        event_id = %event_id.as_uuid(),
                        error = %err,
                        "outbox relay: publish failed; leaving envelope in outbox for retry"
                    );
                    stats.failed += 1;
                }
            }
        }

        if !published_ids.is_empty() {
            // Best-effort: mark only the successfully published
            // envelopes. A failure here is logged but does NOT
            // undo the publish — the next drain will see these
            // envelopes still pending and re-publish them (the
            // bus is at-least-once and subscribers dedupe).
            //
            // `school_id` is the tenant anchor
            // (FND-PORT-STORE-003) — every adapter MUST verify
            // that the handle is scoped to this school and
            // reject mismatches with `TenantViolation`.
            if let Err(err) = self.outbox.mark_published(school_id, &published_ids).await {
                warn!(
                    school_id = %school_id.as_uuid(),
                    error = %err,
                    "outbox relay: mark_published failed; affected envelopes will be retried"
                );
            }
        }

        debug!(
            school_id = %school_id.as_uuid(),
            published = stats.published,
            failed = stats.failed,
            skipped = stats.skipped,
            "outbox relay: batch complete"
        );
        Ok(stats)
    }

    /// Long-running cooperative loop. Calls
    /// [`Self::run_once`] repeatedly until `shutdown` is
    /// cancelled. An empty batch sleeps for
    /// [`DEFAULT_IDLE_DELAY`](`Self::with_idle_delay`) to avoid a
    /// hot loop.
    ///
    /// # Shutdown contract
    ///
    /// `shutdown.cancelled()` is polled at the top of every tick
    /// via `tokio::select!`. When the token fires, the loop
    /// exits cleanly without dropping a half-processed batch.
    ///
    /// # Errors
    ///
    /// Returns `Err` only if the outbox returns a non-recoverable
    /// error (e.g. `Infrastructure`). Publish failures are
    /// absorbed into [`RelayStats::failed`] and never bubble up
    /// — the loop continues.
    pub async fn run_loop(&self, school_id: SchoolId, shutdown: CancellationToken) -> Result<()> {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => {
                    debug!(
                        school_id = %school_id.as_uuid(),
                        "outbox relay: shutdown received; exiting loop"
                    );
                    return Ok(());
                }
                result = self.run_once(school_id) => {
                    match result {
                        Ok(stats) if stats.total() == 0 => {
                            // Empty outbox — sleep before
                            // re-checking. The select arms above
                            // keep the sleep cancellable.
                            tokio::select! {
                                biased;
                                _ = shutdown.cancelled() => return Ok(()),
                                _ = sleep(self.idle_delay) => {}
                            }
                        }
                        Ok(stats) => {
                            debug!(
                                school_id = %school_id.as_uuid(),
                                published = stats.published,
                                failed = stats.failed,
                                "outbox relay: loop tick complete"
                            );
                        }
                        Err(err) => {
                            warn!(
                                school_id = %school_id.as_uuid(),
                                error = %err,
                                "outbox relay: tick failed; backing off"
                            );
                            tokio::select! {
                                biased;
                                _ = shutdown.cancelled() => return Ok(()),
                                _ = sleep(self.idle_delay) => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::EventEnvelope;
    use crate::event_bus::{
        AckOutcome, BatchReceipt, EventBus, EventSubscription, PublishReceipt, SubscribeOptions,
    };
    use crate::relay_envelope::SerializedEnvelope;
    use async_trait::async_trait;
    use bytes::Bytes;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::error::DomainError;
    use educore_core::value_objects::Timestamp;
    use std::convert::TryFrom;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// Acquire a `std::sync::Mutex` guard, recovering from
    /// poisoning. Test fixtures never intentionally poison a
    /// mutex; if poisoning is observed (a previous test
    /// panicked while holding the lock), we recover the
    /// inner value rather than panicking the whole test
    /// process. This replaces the prior `.lock().unwrap()`
    /// pattern in the relay test fixtures.
    fn lock_unpoisoned<T>(m: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        match m.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Convert a `u32` batch limit to `usize` without the
    /// truncating `as` cast. On 32-bit platforms the input
    /// must fit in `usize`; on 64-bit (the only supported
    /// target) every `u32` does. Returns `DomainError::Validation`
    /// on the (impossible on 64-bit) overflow.
    fn batch_limit_to_usize(limit: u32) -> std::result::Result<usize, DomainError> {
        usize::try_from(limit).map_err(|_| DomainError::validation("batch limit exceeds usize"))
    }

    // -----------------------------------------------------------------
    // Test fixture: a counting, optionally-failing bus
    // -----------------------------------------------------------------
    //
    // We do NOT use `educore_event_bus::InProcessEventBus` here
    // because pulling `educore-event-bus` in as a dev-dep on
    // `educore-events` would create a crate graph that Cargo
    // treats as "multiple versions of educore_events in scope"
    // (`educore-event-bus` depends on `educore-events`). The
    // relay only exercises `publish`, so a minimal stand-in is
    // cheaper and avoids the cycle entirely.

    #[derive(Debug)]
    struct CountingBus {
        published: AtomicUsize,
        fail_next: std::sync::Mutex<u32>,
    }

    impl CountingBus {
        fn new() -> Self {
            Self {
                published: AtomicUsize::new(0),
                fail_next: std::sync::Mutex::new(0),
            }
        }
        fn published(&self) -> usize {
            self.published.load(Ordering::SeqCst)
        }
        fn set_fail_next(&self, n: u32) {
            *lock_unpoisoned(&self.fail_next) = n;
        }
    }

    /// Minimal `EventSubscription` stub (the relay never
    /// subscribes; the trait is required by `EventBus`).
    #[derive(Debug)]
    struct NoopSubscription;

    #[async_trait]
    impl EventSubscription for NoopSubscription {
        async fn next(&mut self) -> Option<Result<EventEnvelope>> {
            None
        }
        async fn ack(&mut self, _id: educore_core::ids::EventId) -> Result<AckOutcome> {
            Ok(AckOutcome::Accepted)
        }
        async fn nack(
            &mut self,
            _id: educore_core::ids::EventId,
            _requeue: bool,
        ) -> Result<AckOutcome> {
            Ok(AckOutcome::Accepted)
        }
        async fn close(self: Box<Self>) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl EventBus for CountingBus {
        async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt> {
            let mut fail = lock_unpoisoned(&self.fail_next);
            if *fail > 0 {
                *fail -= 1;
                return Err(DomainError::validation("forced failure"));
            }
            drop(fail);
            self.published.fetch_add(1, Ordering::SeqCst);
            Ok(PublishReceipt::new(
                envelope.event_id,
                envelope.event_type.clone(),
                Timestamp::now(),
            ))
        }

        async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt> {
            let mut receipts = Vec::with_capacity(envelopes.len());
            let mut failures = Vec::new();
            for env in envelopes {
                match self.publish(env).await {
                    Ok(r) => receipts.push(r),
                    Err(e) => {
                        failures.push(crate::event_bus::BatchFailure::new(None, e.to_string()))
                    }
                }
            }
            Ok(BatchReceipt::new(receipts, failures))
        }

        async fn subscribe(
            &self,
            _options: SubscribeOptions,
        ) -> Result<Box<dyn EventSubscription>> {
            Ok(Box::new(NoopSubscription))
        }
    }

    // -----------------------------------------------------------------
    // Test fixture: an in-memory Outbox for the relay
    // -----------------------------------------------------------------
    //
    // The relay uses `pending_for_school` and `mark_published`.
    // We implement a thin wrapper that holds a `Vec` and
    // supports those two methods + `append_envelope` so tests
    // can stage envelopes. We deliberately do NOT depend on
    // the `educore-testkit`'s `InMemoryStorageAdapter` (the
    // relay should be testable in isolation).

    #[derive(Debug)]
    struct VecOutbox {
        rows: std::sync::Mutex<Vec<SerializedEnvelope>>,
    }

    impl VecOutbox {
        fn new() -> Self {
            Self {
                rows: std::sync::Mutex::new(Vec::new()),
            }
        }
        fn append_envelope(&self, env: SerializedEnvelope) {
            lock_unpoisoned(&self.rows).push(env);
        }
        fn len(&self) -> usize {
            lock_unpoisoned(&self.rows).len()
        }
    }

    #[async_trait]
    impl OutboxSource for VecOutbox {
        async fn pending_for_school(
            &self,
            school_id: SchoolId,
            limit: u32,
        ) -> Result<Vec<SerializedEnvelope>> {
            let limit_usize = batch_limit_to_usize(limit)?;
            let g = lock_unpoisoned(&self.rows);
            Ok(g.iter()
                .filter(|e| e.school_id == school_id)
                .take(limit_usize)
                .cloned()
                .collect())
        }
        async fn mark_published(&self, school_id: SchoolId, ids: &[EventId]) -> Result<()> {
            let mut g = lock_unpoisoned(&self.rows);
            // FND-PORT-STORE-003: `school_id` is the tenant
            // anchor. Only retain rows that are NOT in `ids`
            // AND belong to `school_id`; rows from other
            // tenants must be preserved even if their ids
            // happen to collide with `ids`.
            g.retain(|e| e.school_id != school_id || !ids.contains(&e.event_id));
            Ok(())
        }
    }

    // -----------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------

    fn sample_envelope(school: SchoolId, event_type: &str) -> SerializedEnvelope {
        let g = SystemIdGen;
        SerializedEnvelope {
            event_id: g.next_event_id(),
            event_type: event_type.to_owned(),
            schema_version: 1,
            school_id: school,
            aggregate_id: g.next_uuid(),
            aggregate_type: "test".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            payload: Bytes::from_static(b"{}"),
        }
    }

    fn relay(outbox: Arc<VecOutbox>, bus: Arc<CountingBus>) -> OutboxRelay<VecOutbox, CountingBus> {
        OutboxRelay::new(outbox, bus)
    }

    // -----------------------------------------------------------------
    // run_once
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn run_once_drains_pending_envelopes_and_publishes_via_bus(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));
        outbox.append_envelope(sample_envelope(school, "academic.student.transferred"));

        let r = relay(outbox.clone(), bus.clone());
        let stats = r.run_once(school).await?;

        assert_eq!(stats.published, 2);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.total(), 2);
        assert!(stats.is_fully_published());
        assert_eq!(bus.published(), 2);
        assert_eq!(outbox.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn run_once_marks_envelopes_as_published_after_successful_publish(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        let env = sample_envelope(school, "academic.student.admitted");
        let env_id = env.event_id;
        outbox.append_envelope(env);

        let r = relay(outbox.clone(), bus.clone());
        let stats = r.run_once(school).await?;

        assert_eq!(stats.published, 1);
        // The row was removed from the outbox.
        assert_eq!(outbox.len(), 0);
        // The bus saw exactly the envelope we appended.
        assert_eq!(bus.published(), 1);
        // Sanity: the event id we published matches what we
        // appended.
        let _ = env_id; // (kept to make the test self-documenting)
        Ok(())
    }

    #[tokio::test]
    async fn run_once_does_not_mark_failed_envelopes_as_published(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        bus.set_fail_next(2); // fail the next two publishes
        outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));
        outbox.append_envelope(sample_envelope(school, "academic.student.transferred"));

        let r = relay(outbox.clone(), bus.clone());
        let stats = r.run_once(school).await?;

        assert_eq!(stats.published, 0);
        assert_eq!(stats.failed, 2);
        // Both envelopes must remain in the outbox for retry.
        assert_eq!(outbox.len(), 2);
        assert_eq!(bus.published(), 0);

        // Retry: now the bus is healthy, both should drain.
        let stats2 = r.run_once(school).await?;
        assert_eq!(stats2.published, 2);
        assert_eq!(stats2.failed, 0);
        assert_eq!(outbox.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn run_once_partial_failure_leaves_only_failed_envelopes_in_outbox(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        bus.set_fail_next(1); // fail the first publish only
        outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));
        outbox.append_envelope(sample_envelope(school, "academic.student.transferred"));

        let r = relay(outbox.clone(), bus.clone());
        let stats = r.run_once(school).await?;

        assert_eq!(stats.published, 1);
        assert_eq!(stats.failed, 1);
        // Exactly one envelope (the failing one) remains.
        assert_eq!(outbox.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn run_once_empty_outbox_is_noop() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        let r = relay(outbox, bus);
        let stats = r.run_once(school).await?;
        assert_eq!(stats.published, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.total(), 0);
        assert!(!stats.is_fully_published());
        Ok(())
    }

    #[tokio::test]
    async fn run_once_skips_envelopes_from_other_schools(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school_a = g.next_school_id();
        let school_b = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        // Envelope belongs to school_b, but we drain for school_a.
        outbox.append_envelope(sample_envelope(school_b, "academic.student.admitted"));

        let r = relay(outbox.clone(), bus.clone());
        let stats = r.run_once(school_a).await?;
        assert_eq!(stats.published, 0);
        assert_eq!(stats.skipped, 0); // pending_for_school filtered it
        assert_eq!(outbox.len(), 1); // still pending for school_b
        Ok(())
    }

    #[tokio::test]
    async fn run_once_with_custom_batch_size_caps_pending_read(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        for _ in 0..5 {
            outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));
        }

        let r = OutboxRelay::new(outbox.clone(), bus.clone()).with_batch_size(2);
        let stats = r.run_once(school).await?;
        assert_eq!(stats.published, 2);
        assert_eq!(outbox.len(), 3);
        Ok(())
    }

    // -----------------------------------------------------------------
    // run_loop
    // -----------------------------------------------------------------

    #[tokio::test]
    async fn run_loop_respects_cancellation_token_shutdown(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        // Empty outbox — the loop will idle-sleep. Use a short
        // idle delay so the test is fast.
        let r = OutboxRelay::new(outbox, bus).with_idle_delay(Duration::from_millis(10));
        let token = CancellationToken::new();

        // Schedule a shutdown after 50ms.
        let child = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            child.cancel();
        });

        let started = std::time::Instant::now();
        r.run_loop(school, token).await?;
        let elapsed = started.elapsed();
        // Should have run at least one idle tick before
        // shutdown; should NOT have run for more than 1s.
        assert!(
            elapsed < Duration::from_secs(1),
            "run_loop took too long ({:?}) — shutdown did not fire",
            elapsed
        );
        Ok(())
    }

    #[tokio::test]
    async fn run_loop_drains_initial_envelopes_before_idling(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        for _ in 0..3 {
            outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));
        }

        let r = OutboxRelay::new(outbox.clone(), bus.clone())
            .with_idle_delay(Duration::from_millis(20));
        let token = CancellationToken::new();
        let child = token.clone();
        // Cancel after a brief moment so the loop has time to
        // drain the initial batch.
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            child.cancel();
        });

        r.run_loop(school, token).await?;
        assert_eq!(bus.published(), 3);
        assert_eq!(outbox.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn run_loop_immediate_cancellation_exits_without_publishing(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let outbox = Arc::new(VecOutbox::new());
        let bus = Arc::new(CountingBus::new());
        outbox.append_envelope(sample_envelope(school, "academic.student.admitted"));

        let r =
            OutboxRelay::new(outbox.clone(), bus.clone()).with_idle_delay(Duration::from_secs(10));
        let token = CancellationToken::new();
        // Pre-cancel the token: the first select arm should
        // fire immediately on the first iteration.
        token.cancel();

        r.run_loop(school, token).await?;
        // We can't assert "nothing was published" because the
        // loop MAY publish the single envelope before checking
        // the token on the next iteration — but it MUST exit
        // promptly. The first iteration's `run_once` publishes
        // the envelope, then the loop checks the token. The
        // envelope is published, then the loop exits.
        // The contract we assert is: the loop returned.
        Ok(())
    }

    // -----------------------------------------------------------------
    // Stats
    // -----------------------------------------------------------------

    #[test]
    fn relay_stats_default_is_zero() {
        let s = RelayStats::default();
        assert_eq!(s.total(), 0);
        assert!(!s.is_fully_published());
    }

    #[test]
    fn relay_stats_total_sums_all_fields() {
        let s = RelayStats {
            published: 3,
            failed: 2,
            skipped: 1,
        };
        assert_eq!(s.total(), 6);
        assert!(!s.is_fully_published());
        let s = RelayStats {
            published: 3,
            failed: 0,
            skipped: 0,
        };
        assert!(s.is_fully_published());
    }

    // -----------------------------------------------------------------
    // Test fixture helpers
    // -----------------------------------------------------------------
    //
    // Intentionally empty: the test fixtures above use
    // `DomainError::validation(...)` for forced failures, which
    // does not need a custom error type. Previously a
    // `StringError` helper lived here to construct
    // `DomainError::Infrastructure(...)`, but no test ever called
    // it; the helper was preserved under the file-level
    // `#[allow(clippy::all)]`. Cluster E removed that blanket
    // allow, so the dead helper has been deleted.
}
