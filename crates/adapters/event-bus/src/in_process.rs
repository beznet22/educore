//! # In-process event bus
//!
//! The default [`EventBus`] implementation. MPMC; backed by a
//! [`tokio::sync::broadcast`] channel plus a bounded replay log
//! for [`StartPosition::Earliest`](educore_events::StartPosition::Earliest)
//! and the `FromEventId` / `FromTimestamp` cursor modes.
//!
//! # Topology
//!
//! One global [`broadcast::Sender`] per bus. Every publish fans out
//! to every active subscription. Per-topic routing and event
//! filtering are applied in the subscription's
//! [`next`](InProcessSubscription::next) loop, after a possible
//! replay-log drain. This is intentionally simple: the in-process
//! bus is for single-node deployments and tests; consumers in the
//! same process are cheap to filter, and the broadcast channel
//! is the most ergonomic MPMC primitive `tokio` offers.
//!
//! # Replay
//!
//! The bus keeps a bounded [`VecDeque`] of recent envelopes (the
//! `replay_log_capacity`). On `subscribe` with `Earliest`, the
//! subscription is hydrated with a snapshot of the log. With
//! `Latest`, the replay buffer is empty and the subscription
//! sees only envelopes published after `subscribe` returns. With
//! `FromEventId(id)` / `FromTimestamp(ts)`, the log is filtered
//! to entries strictly after the cursor.
//!
//! # Acknowledgement
//!
//! In-process delivery is direct, so [`ack`](InProcessSubscription::ack)
//! and [`nack`](InProcessSubscription::nack) are no-ops returning
//! [`AckOutcome::Accepted`]. Real acknowledgement semantics land
//! on the distributed adapters (NATS JetStream, Redis Streams).

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use tokio::sync::broadcast;
use tracing::{debug, trace};

use educore_core::ids::{EventId, Identifier};
use educore_core::value_objects::Timestamp;
use educore_events::envelope::EventEnvelope;
use educore_events::errors::EventError;
use educore_events::event_bus::{
    AckOutcome, BatchFailure, BatchReceipt, ConsumerId, EventBus, EventFilter, EventSubscription,
    PublishReceipt, StartPosition, SubscribeOptions, Topic,
};

use crate::errors::subscribe_failed;

/// Default broadcast channel capacity. Sized to absorb the
/// bursty phase of a single command's emitted events across
/// all in-process subscribers without forcing `Lagged` skips.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

/// Default replay-log capacity. Sized so a fresh projection can
/// rebuild from in-memory history for short windows; the
/// outbox / event log is the source of truth for longer replays.
pub const DEFAULT_REPLAY_LOG_CAPACITY: usize = 4096;

/// Tunable knobs for [`InProcessEventBus`]. Construct with
/// [`InProcessEventBus::with_config`] or use the `new` /
/// `with_capacity` presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InProcessConfig {
    /// Per-subscription broadcast channel capacity. Clamped to
    /// `1..=u32::MAX as usize`.
    pub channel_capacity: usize,
    /// Replay-log capacity. Clamped to `0..=u32::MAX as usize`.
    /// `0` disables replay (every subscription behaves as
    /// `StartPosition::Latest`).
    pub replay_log_capacity: usize,
}

impl Default for InProcessConfig {
    fn default() -> Self {
        Self {
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            replay_log_capacity: DEFAULT_REPLAY_LOG_CAPACITY,
        }
    }
}

impl InProcessConfig {
    /// Returns a config with both capacities clamped to `0` and the
    /// given capacity (`1..=u32::MAX`).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channel_capacity: clamp_capacity(capacity),
            replay_log_capacity: 0,
        }
    }

    /// Returns a config with the given channel and replay capacities.
    #[must_use]
    pub fn new(channel_capacity: usize, replay_log_capacity: usize) -> Self {
        Self {
            channel_capacity: clamp_capacity(channel_capacity),
            replay_log_capacity: clamp_capacity(replay_log_capacity),
        }
    }
}

fn clamp_capacity(c: usize) -> usize {
    // The broadcast channel rejects 0; the replay log accepts 0.
    if c == 0 {
        1
    } else {
        c
    }
}

/// The in-process event bus. Cheap to clone (the inner state is
/// `Arc`-shared) and `Send + Sync`; pass it to consumers via
/// `Arc<dyn EventBus>`.
#[derive(Clone)]
pub struct InProcessEventBus {
    inner: Arc<InProcessInner>,
}

impl fmt::Debug for InProcessEventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.inner;
        f.debug_struct("InProcessEventBus")
            .field("channel_capacity", &inner.config.channel_capacity)
            .field("replay_log_capacity", &inner.config.replay_log_capacity)
            .field(
                "subscription_id_seq",
                &inner.next_subscription_id.load(Ordering::Relaxed),
            )
            .finish()
    }
}

struct InProcessInner {
    /// Global broadcast channel. The bus is MPMC: every publish
    /// is a `Sender::send`; every active subscription holds a
    /// `Receiver`.
    sender: broadcast::Sender<EventEnvelope>,
    /// Bounded replay log. Protected by a `std::sync::Mutex`; the
    /// critical section is just a `push_back` + `pop_front`, so
    /// parking the OS thread is cheap.
    log: Mutex<VecDeque<EventEnvelope>>,
    /// Tunable knobs. Captured at construction time; the bus
    /// cannot be reconfigured in place.
    config: InProcessConfig,
    /// Monotonic subscription-id sequence. Surfaced via `Debug`
    /// and used to disambiguate subscriptions in `tracing` spans.
    next_subscription_id: AtomicU64,
}

impl InProcessEventBus {
    /// Constructs a bus with the default capacities.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(InProcessConfig::default())
    }

    /// Constructs a bus with the given channel capacity and a
    /// zero-length replay log (i.e. `StartPosition::Earliest`
    /// behaves like `Latest`).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_config(InProcessConfig::with_capacity(capacity))
    }

    /// Constructs a bus with the given configuration.
    #[must_use]
    pub fn with_config(config: InProcessConfig) -> Self {
        let sender = broadcast::Sender::new(config.channel_capacity);
        let log = if config.replay_log_capacity == 0 {
            VecDeque::new()
        } else {
            VecDeque::with_capacity(config.replay_log_capacity)
        };
        Self {
            inner: Arc::new(InProcessInner {
                sender,
                log: Mutex::new(log),
                config,
                next_subscription_id: AtomicU64::new(0),
            }),
        }
    }

    /// Returns the number of envelopes currently buffered in the
    /// replay log. Used by tests and operational tooling.
    #[must_use]
    pub fn replay_log_len(&self) -> usize {
        match self.inner.log.lock() {
            Ok(g) => g.len(),
            Err(_) => 0,
        }
    }

    /// Returns the number of active receivers on the underlying
    /// broadcast channel. Used by tests to assert that
    /// [`InProcessSubscription::close`] releases resources.
    #[must_use]
    pub fn receiver_count(&self) -> usize {
        self.inner.sender.receiver_count()
    }

    /// Returns the bus's configuration. The bus cannot be
    /// reconfigured in place; the config is informational after
    /// construction.
    #[must_use]
    pub fn config(&self) -> InProcessConfig {
        self.inner.config
    }
}

impl Default for InProcessEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InProcessEventBus {
    async fn publish(
        &self,
        envelope: EventEnvelope,
    ) -> educore_core::error::Result<PublishReceipt> {
        // Stamp `published_at` if the producer left it unset.
        let mut env = envelope;
        if env.published_at.is_none() {
            env.published_at = Some(Timestamp::now());
        }

        let topic = env.aggregate_topic();
        let event_id = env.event_id;

        // Push to the replay log under a brief lock.
        if self.inner.config.replay_log_capacity > 0 {
            match self.inner.log.lock() {
                Ok(mut log) => {
                    if log.len() == self.inner.config.replay_log_capacity {
                        log.pop_front();
                    }
                    log.push_back(env.clone());
                }
                Err(_) => {
                    return Err(
                        EventError::PublishFailed("replay log mutex poisoned".to_owned()).into(),
                    );
                }
            }
        }

        // Fan out to every active receiver. `send` only fails if
        // there are zero receivers; that's a normal idle state
        // for the in-process bus and not an error.
        match self.inner.sender.send(env) {
            Ok(_) => Ok(PublishReceipt::new(event_id, topic, Timestamp::now())),
            Err(_) => {
                // No receivers; the envelope is still in the
                // replay log (if any), so a future `Earliest`
                // subscription will see it.
                Ok(PublishReceipt::new(event_id, topic, Timestamp::now()))
            }
        }
    }

    async fn publish_batch(
        &self,
        envelopes: Vec<EventEnvelope>,
    ) -> educore_core::error::Result<BatchReceipt> {
        let mut receipts = Vec::with_capacity(envelopes.len());
        let mut failures = Vec::new();
        for env in envelopes {
            // Per the bus-port contract: adapters that don't
            // support atomic batching fall back to per-envelope
            // `publish`. We record BOTH successes and failures
            // so the caller (e.g. the outbox relay) can decide
            // which source rows to mark-published vs. retry.
            // The previous shape short-circuited on the first
            // error and dropped the trailing envelopes silently
            // — CC-EVT-007.
            match self.publish(env.clone()).await {
                Ok(receipt) => receipts.push(receipt),
                Err(e) => {
                    tracing::warn!(
                        event_id = %env.event_id.as_uuid(),
                        event_type = env.event_type,
                        error = %e,
                        "in-process bus: publish_batch entry failed"
                    );
                    failures.push(BatchFailure::new(Some(env.event_id), e.to_string()));
                }
            }
        }
        Ok(BatchReceipt::new(receipts, failures))
    }

    async fn subscribe(
        &self,
        options: SubscribeOptions,
    ) -> educore_core::error::Result<Box<dyn EventSubscription>> {
        let subscription_id = self
            .inner
            .next_subscription_id
            .fetch_add(1, Ordering::Relaxed);

        // Snapshot the replay log under one lock to avoid races
        // between a concurrent publish and this snapshot. The
        // lock is held briefly (just a `clone` of the relevant
        // slice).
        let replay = {
            let log_guard = self
                .inner
                .log
                .lock()
                .map_err(|_| subscribe_failed("replay log mutex poisoned"))?;
            build_replay_buffer(&log_guard, &options)
        };

        let receiver = self.inner.sender.subscribe();
        let consumer = options.consumer.clone();
        let topic = options.topic.clone();
        let filter = options.filter.clone();

        debug!(
            subscription_id,
            consumer = %consumer,
            topic_wire = %topic.wire(),
            replay_len = replay.len(),
            "subscribed to in-process event bus"
        );

        let sub: Box<dyn EventSubscription> = Box::new(InProcessSubscription {
            receiver,
            replay: Mutex::new(replay),
            consumer,
            topic,
            filter,
            bus: Arc::downgrade(&self.inner),
            closed: false,
        });
        Ok(sub)
    }
}

/// Builds the initial replay buffer for a subscription based on
/// its `StartPosition`. The buffer is filtered against the
/// subscription's `Topic` and `EventFilter` so the consumer
/// never has to re-filter replayed entries.
fn build_replay_buffer(
    log: &VecDeque<EventEnvelope>,
    options: &SubscribeOptions,
) -> VecDeque<EventEnvelope> {
    log.iter()
        .filter(|env| start_position_matches(&options.start, env))
        .filter(|env| topic_matches(&options.topic, env))
        .filter(|env| filter_matches(options.filter.as_ref(), env))
        .cloned()
        .collect()
}

fn start_position_matches(start: &StartPosition, env: &EventEnvelope) -> bool {
    match start {
        StartPosition::Latest => false,
        StartPosition::Earliest => true,
        StartPosition::FromEventId(id) => {
            // Replay strictly after the cursor. `id` is the last
            // event the consumer has processed; the consumer will
            // see `id` itself in the live stream if the bus has
            // it, but we only deliver unseen events here.
            // UUIDv7 is time-ordered: lexicographic comparison
            // gives chronological ordering.
            env.event_id.as_uuid() > id.as_uuid()
        }
        StartPosition::FromTimestamp(ts) => env.occurred_at.as_datetime() > ts.as_datetime(),
    }
}

fn topic_matches(topic: &Topic, env: &EventEnvelope) -> bool {
    match topic {
        Topic::Aggregate(d, a) => env.aggregate_topic() == format!("{d}.{a}"),
        Topic::Domain(d) => {
            let wire = env.aggregate_topic();
            wire == *d || wire.starts_with(&format!("{d}."))
        }
        Topic::EventType(t) => env.event_type == *t,
        Topic::Tenant(s) => env.is_for_school(*s),
        Topic::All => true,
    }
}

fn filter_matches(filter: Option<&EventFilter>, env: &EventEnvelope) -> bool {
    filter.map_or(true, |f| f.matches(env))
}

/// A long-lived subscription to the in-process bus.
pub struct InProcessSubscription {
    /// The live broadcast channel receiver.
    receiver: broadcast::Receiver<EventEnvelope>,
    /// Pre-buffered replay log; drained before the live receiver
    /// in `next`. Wrapped in a `Mutex` so `close` can take it
    /// without aliasing.
    replay: Mutex<VecDeque<EventEnvelope>>,
    /// Stable consumer id (echoed for tracing).
    consumer: ConsumerId,
    /// The routing target. Envelopes not matching this topic are
    /// silently dropped in `next`.
    topic: Topic,
    /// Optional server-side filter (re-applied client-side per
    /// the bus-port contract).
    filter: Option<EventFilter>,
    /// Weak reference to the bus for diagnostics / `close` race
    /// detection. The bus does not need to outlive the
    /// subscription — when the bus is dropped, the broadcast
    /// channel will start returning `RecvError::Closed`.
    bus: Weak<InProcessInner>,
    /// `true` after `close` has been called.
    closed: bool,
}

impl fmt::Debug for InProcessSubscription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InProcessSubscription")
            .field("consumer", &self.consumer)
            .field("topic", &self.topic.wire())
            .field("closed", &self.closed)
            .field(
                "replay_remaining",
                &self.replay.lock().map(|g| g.len()).unwrap_or(0),
            )
            .finish()
    }
}

#[async_trait]
impl EventSubscription for InProcessSubscription {
    async fn next(&mut self) -> Option<educore_core::error::Result<EventEnvelope>> {
        if self.closed {
            return None;
        }

        loop {
            // 1) Drain the replay buffer first (only on the first
            //    pass — once it is empty, we live in the broadcast
            //    channel for the rest of the subscription's life).
            {
                let mut replay = match self.replay.lock() {
                    Ok(g) => g,
                    Err(_) => {
                        return Some(Err(EventError::SubscriptionClosed.into()));
                    }
                };
                if let Some(env) = replay.pop_front() {
                    trace!(event_id = %env.event_id.as_uuid(), "replaying envelope");
                    return Some(Ok(env));
                }
            }

            // 2) Live: await the broadcast channel.
            match self.receiver.recv().await {
                Ok(env) => {
                    if !topic_matches(&self.topic, &env) {
                        continue;
                    }
                    if !filter_matches(self.filter.as_ref(), &env) {
                        continue;
                    }
                    return Some(Ok(env));
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    debug!(
                        consumer = %self.consumer,
                        skipped, "subscription lagged; skipping past missed envelopes"
                    );
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    }

    async fn ack(&mut self, _event_id: EventId) -> educore_core::error::Result<AckOutcome> {
        // In-process delivery is direct; ack is a no-op.
        Ok(AckOutcome::Accepted)
    }

    async fn nack(
        &mut self,
        _event_id: EventId,
        _requeue: bool,
    ) -> educore_core::error::Result<AckOutcome> {
        // In-process delivery is direct; nack is a no-op.
        Ok(AckOutcome::Accepted)
    }

    async fn close(self: Box<Self>) -> educore_core::error::Result<()> {
        // Drop the receiver (releases its slot in the broadcast
        // channel) and the replay buffer (clears the heap).
        let mut me = *self;
        me.closed = true;
        // The `Weak<InProcessInner>` is informational: if the
        // bus has been dropped, the broadcast channel will
        // start returning `RecvError::Closed` to anyone still
        // holding a `Receiver`. We log whether the bus is still
        // alive at close time so consumers can detect abnormal
        // lifetimes in production.
        if me.bus.strong_count() == 0 {
            tracing::debug!(
                consumer = %me.consumer,
                "subscription closed after bus was dropped"
            );
        } else {
            tracing::trace!(
                consumer = %me.consumer,
                "subscription closed; bus still alive"
            );
        }
        // The `broadcast::Receiver` is dropped when `me` is dropped
        // at end of scope; the replay `Mutex<VecDeque>` is also
        // dropped. Both releases are deterministic at the
        // end-of-scope boundary.
        Ok(())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_events::domain_event::DomainEvent;
    use educore_events::sync::SyncStarted;

    fn sample_envelope(event_type: &'static str, aggregate_type: &'static str) -> EventEnvelope {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let env = SyncStarted::now(school);
        let mut envelope = env.into_envelope(&educore_core::tenant::TenantContext::system(
            school,
            g.next_correlation_id(),
        ));
        envelope.event_type = event_type;
        envelope.aggregate_type = aggregate_type;
        envelope
    }

    #[test]
    fn config_clamps_zero_capacity_to_one() {
        let cfg = InProcessConfig::with_capacity(0);
        assert_eq!(cfg.channel_capacity, 1);
        assert_eq!(cfg.replay_log_capacity, 0);
    }

    #[test]
    fn default_config_uses_constants() {
        let cfg = InProcessConfig::default();
        assert_eq!(cfg.channel_capacity, DEFAULT_CHANNEL_CAPACITY);
        assert_eq!(cfg.replay_log_capacity, DEFAULT_REPLAY_LOG_CAPACITY);
    }

    #[test]
    fn new_bus_reports_zero_receivers() {
        let bus = InProcessEventBus::new();
        assert_eq!(bus.receiver_count(), 0);
        assert_eq!(bus.replay_log_len(), 0);
    }

    #[tokio::test]
    async fn publish_stamps_published_at_and_records_log() {
        let bus = InProcessEventBus::new();
        let env = sample_envelope("sync.session.started", "sync_session");
        let r = bus.publish(env.clone()).await.expect("publish");
        assert_eq!(r.event_id, env.event_id);
        assert!(bus.replay_log_len() >= 1);
    }

    #[test]
    fn start_position_matches_handles_all_variants() {
        let env = sample_envelope("sync.session.started", "sync_session");
        assert!(!start_position_matches(&StartPosition::Latest, &env));
        assert!(start_position_matches(&StartPosition::Earliest, &env));
        // FromEventId before the env's id -> not yet seen -> replay
        let cursor = EventId::from_uuid(uuid::Uuid::from_u128(
            env.event_id.as_uuid().as_u128().saturating_sub(1),
        ));
        assert!(start_position_matches(
            &StartPosition::FromEventId(cursor),
            &env
        ));
        // FromEventId after the env's id -> already seen -> skip
        let cursor = EventId::from_uuid(uuid::Uuid::from_u128(
            env.event_id.as_uuid().as_u128().saturating_add(1),
        ));
        assert!(!start_position_matches(
            &StartPosition::FromEventId(cursor),
            &env
        ));
        // FromTimestamp before the env's occurred_at -> not yet seen -> replay
        // We use a 1-second offset via the chrono Datetime API
        // (Timestamp wraps chrono; the API is accessible without
        // pulling chrono in directly here, but for relative
        // arithmetic we round-trip through a Timestamp::from_datetime
        // constructed from the env's own datetime shifted by ±1s).
        let base = env.occurred_at.as_datetime();
        let before = Timestamp::from_datetime(base - chrono::Duration::seconds(1));
        assert!(start_position_matches(
            &StartPosition::FromTimestamp(before),
            &env
        ));
        // FromTimestamp after the env's occurred_at -> already seen -> skip
        let after = Timestamp::from_datetime(base + chrono::Duration::seconds(1));
        assert!(!start_position_matches(
            &StartPosition::FromTimestamp(after),
            &env
        ));
    }

    #[test]
    fn topic_matches_handles_all_variants() {
        let env = sample_envelope("sync.session.started", "sync_session");
        // SyncStarted has event_type "sync.session.started" and
        // aggregate_type "sync_session"; the aggregate_topic is
        // "sync.sync_session" (the domain prefix is "sync").
        assert!(topic_matches(
            &Topic::Aggregate("sync", "sync_session"),
            &env
        ));
        assert!(!topic_matches(
            &Topic::Aggregate("academic", "student"),
            &env
        ));
        assert!(topic_matches(&Topic::Domain("sync"), &env));
        assert!(!topic_matches(&Topic::Domain("academic"), &env));
        assert!(topic_matches(
            &Topic::EventType("sync.session.started"),
            &env
        ));
        assert!(!topic_matches(
            &Topic::EventType("academic.student.admitted"),
            &env
        ));
        assert!(topic_matches(&Topic::Tenant(env.school_id), &env));
        assert!(topic_matches(&Topic::All, &env));
    }
}
