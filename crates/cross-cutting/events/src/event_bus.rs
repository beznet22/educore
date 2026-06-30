//! # `EventBus` port
//!
//! The bus port, the subscription port, and the supporting value
//! objects (`Topic`, `SubscribeOptions`, `EventFilter`,
//! `StartPosition`, `ConsumerId`, `PublishReceipt`, `BatchReceipt`).
//! Shapes are locked to [`docs/ports/event-bus.md`](../docs/ports/event-bus.md).
//!
//! Concrete adapters (`InProcessEventBus`, `NatsEventBus`,
//! `RedisEventBus`) live in the `educore-event-bus` crate; this
//! module is the port only.

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::domain_event::DomainEvent;
use crate::envelope::EventEnvelope;

/// The bus port. Object-safe.
///
/// # Implementations
///
/// - `InProcessEventBus` — `educore-event-bus` crate, default
///   feature. MPMC, bounded channels per subscription.
/// - `NatsEventBus` — `educore-event-bus` crate, `nats` feature.
///   NATS JetStream-backed.
/// - `RedisEventBus` — `educore-event-bus` crate, `redis`
///   feature. Redis Streams-backed.
#[async_trait]
pub trait EventBus: Send + Sync + fmt::Debug {
    /// Publish a single envelope. Returns once the bus has
    /// accepted the envelope (not necessarily delivered).
    async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;

    /// Publish a batch atomically. Adapters that don't support
    /// atomic batching should fall back to per-envelope
    /// `publish`; consumers cannot assume either semantics
    /// unless they pin the adapter.
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;

    /// Subscribe to a topic. The returned subscription is a
    /// long-lived async iterator.
    async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;

    /// Returns the dead-letter queue (DLQ) attached to this bus,
    /// if one is configured. Adapters that surface a DLQ
    /// override this method; the default returns `None`, in
    /// which case the `EventSubscription::nack` path treats
    /// `requeue = false` as a drop (the envelope is logged but
    /// not persisted anywhere recoverable).
    ///
    /// Per [`docs/ports/event-bus.md`](../docs/ports/event-bus.md):
    /// "`nack(requeue=false)` routes the envelope to the
    /// dead-letter queue." The DLQ port ([`DeadLetterQueue`])
    /// is the landing pad; the bus port exposes it through
    /// this accessor so subscription code can route nacks
    /// without holding a separate handle to the DLQ adapter.
    ///
    /// # Object safety
    ///
    /// The return type is `Option<Arc<dyn DeadLetterQueue>>`.
    /// The `Arc` indirection keeps the trait object-safe (no
    /// generic lifetimes leak through) and matches the
    /// `Arc<dyn AuditSink>` shape used elsewhere on the bus
    /// port.
    ///
    /// # Default
    ///
    /// Returns `None`. Existing implementations that do not
    /// yet expose a DLQ continue to compile unchanged; they
    /// should override this once they wire a DLQ adapter.
    async fn dlq(&self) -> Option<Arc<dyn DeadLetterQueue>> {
        None
    }

    /// Subscribe to a topic filtered to a specific typed event `E`.
    ///
    /// `subscribe_typed` is the typed counterpart to
    /// [`EventBus::subscribe`]. The returned
    /// [`TypedEventSubscription<E>`] yields only envelopes whose
    /// `event_type` matches `E::EVENT_TYPE`, with the payload
    /// deserialized to `E`. This is the API
    /// `library-docs.md` documents as
    /// `engine.events().subscribe::<T>()`.
    ///
    /// The default implementation composes the existing
    /// [`EventBus::subscribe`] path: it calls `subscribe` with an
    /// [`EventFilter::EventType(E::EVENT_TYPE)`] server-side
    /// filter and additionally re-applies the filter
    /// client-side (the [`TypedEventSubscription`] wrapper drops
    /// any envelope whose `event_type` does not match before
    /// deserialization). Adapters that can evaluate the
    /// `EventType` filter server-side should prefer it; the
    /// client-side re-application is a defensive belt-and-braces
    /// guard per the bus-port contract.
    ///
    /// # Object safety
    ///
    /// The method is generic over `E`, so it is NOT dispatchable
    /// through `Arc<dyn EventBus>`. The `where Self: Sized` bound
    /// enforces this; consumers that hold a `dyn EventBus` should
    /// use [`EventBus::subscribe`] with
    /// `EventFilter::EventType(E::EVENT_TYPE)` directly.
    ///
    /// # When to use
    ///
    /// Prefer `subscribe_typed` over the untyped
    /// [`EventBus::subscribe`] when you would otherwise hand-write
    /// `SubscribeOptions { filter: Some(EventFilter::EventType(
    /// E::EVENT_TYPE)), .. }` at every call site. The typed
    /// helper centralises the boilerplate and prevents drift
    /// between the type's `EVENT_TYPE` constant and the filter
    /// string.
    ///
    /// # Errors
    ///
    /// Returns `Err(_)` if the underlying [`EventBus::subscribe`]
    /// call fails. Deserialization failures of matching envelopes
    /// surface as `Some(Err(DomainError::Validation(_)))` from
    /// [`TypedEventSubscription::next`], not from this method.
    async fn subscribe_typed<E>(
        &self,
        options: TypedSubscribeOptions,
    ) -> Result<TypedEventSubscription<E>>
    where
        E: DomainEvent + Serialize + DeserializeOwned + 'static,
        Self: Sized,
    {
        let event_type: &'static str = E::EVENT_TYPE;
        let opts = SubscribeOptions {
            consumer: options.consumer,
            topic: options.topic,
            filter: Some(EventFilter::EventType(event_type)),
            start: options.start,
            batch_size: options.batch_size,
            visibility_timeout: options.visibility_timeout,
        };
        let inner = self.subscribe(opts).await?;
        Ok(TypedEventSubscription {
            inner,
            _marker: PhantomData,
        })
    }
}

/// Acknowledgement semantics for `EventSubscription::ack` /
/// `nack`. Adapters report the wire-level ack result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AckOutcome {
    /// The acknowledgement was accepted by the bus.
    Accepted,
    /// The event id was unknown (already acked, or never delivered).
    Unknown,
    /// The ack failed for an infrastructure reason.
    Failed,
}

/// A long-lived async iterator over [`EventEnvelope`]s. Adapters
/// own the underlying channel / queue and release resources on
/// `close`.
///
/// Consumers MUST be idempotent: the bus provides at-least-once
/// delivery. The `event_id` is the dedupe key.
#[async_trait]
pub trait EventSubscription: Send + Sync {
    /// Returns the next envelope, or `None` if the subscription
    /// is closed. Errors are surfaced as `Some(Err(_))`.
    async fn next(&mut self) -> Option<Result<EventEnvelope>>;

    /// Acknowledges processing of `event_id`. Idempotent.
    async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;

    /// Negatively acknowledges `event_id`. `requeue = true` puts
    /// the event back on the bus (subject to retry limits);
    /// `requeue = false` routes it to the dead letter queue.
    async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome>;

    /// Closes the subscription, releasing adapter-level resources.
    async fn close(self: Box<Self>) -> Result<()>;
}

/// Options for [`EventBus::subscribe`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeOptions {
    /// Stable identifier for the consumer. The bus uses it for
    /// offset tracking and observability.
    pub consumer: ConsumerId,
    /// The topic to subscribe to.
    pub topic: Topic,
    /// Optional server-side filter; the bus may apply this for
    /// efficiency but consumers MUST also apply it client-side.
    pub filter: Option<EventFilter>,
    /// Where to start reading. See [`StartPosition`].
    pub start: StartPosition,
    /// Maximum number of envelopes the subscription may buffer
    /// locally. Adapters clamp this to a sane range (e.g. 1..=1024).
    pub batch_size: u32,
    /// Visibility timeout for in-flight envelopes. After this
    /// duration the bus may redeliver the envelope to another
    /// consumer.
    pub visibility_timeout: Duration,
}

impl SubscribeOptions {
    /// Constructs a default `SubscribeOptions` for a consumer
    /// reading the latest events on a topic.
    #[must_use]
    pub fn for_consumer(consumer: ConsumerId, topic: Topic) -> Self {
        Self {
            consumer,
            topic,
            filter: None,
            start: StartPosition::Latest,
            batch_size: 32,
            visibility_timeout: Duration::from_secs(300),
        }
    }
}

/// Options for [`EventBus::subscribe_typed`].
///
/// A thin wrapper over [`SubscribeOptions`] that omits the
/// `filter` field — the filter is fixed to
/// [`EventFilter::EventType(E::EVENT_TYPE)`] by the typed
/// subscription because it only delivers events of type `E`.
/// Consumers supply the consumer id, topic, and start position;
/// the rest of [`SubscribeOptions`] is preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedSubscribeOptions {
    /// Stable identifier for the consumer. The bus uses it for
    /// offset tracking and observability.
    pub consumer: ConsumerId,
    /// The topic to subscribe to. See [`SubscribeOptions::topic`].
    pub topic: Topic,
    /// Where to start reading. See [`StartPosition`].
    pub start: StartPosition,
    /// Maximum number of envelopes the subscription may buffer
    /// locally. Adapters clamp this to a sane range (e.g. 1..=1024).
    pub batch_size: u32,
    /// Visibility timeout for in-flight envelopes. After this
    /// duration the bus may redeliver the envelope to another
    /// consumer.
    pub visibility_timeout: Duration,
}

impl TypedSubscribeOptions {
    /// Constructs a default `TypedSubscribeOptions` for a
    /// consumer reading the latest events on a topic.
    #[must_use]
    pub fn for_consumer(consumer: ConsumerId, topic: Topic) -> Self {
        Self {
            consumer,
            topic,
            start: StartPosition::Latest,
            batch_size: 32,
            visibility_timeout: Duration::from_secs(300),
        }
    }
}

/// A typed subscription returned by [`EventBus::subscribe_typed`].
///
/// The wrapper holds an underlying [`EventSubscription`] and
/// filters / deserializes envelopes before yielding them to the
/// caller. Envelopes whose `event_type` does not match
/// `E::EVENT_TYPE` are silently skipped (the consumer-side
/// re-application of the filter per the bus-port contract);
/// envelopes that match but fail to deserialize to `E` surface
/// as `Some(Err(DomainError::Validation(_)))` from
/// [`TypedEventSubscription::next`].
///
/// # Skipped envelopes
///
/// The `next()` method silently drops non-matching envelopes
/// without acking them. The bus will redeliver the envelope to
/// another consumer after the visibility timeout if no ack is
/// received. This is the correct behavior for a typed
/// subscription: the wrapper's filter is the contract, and
/// non-matching envelopes are simply not for this consumer.
///
/// # Ack / nack / close
///
/// The wrapper delegates `ack`, `nack`, and `close` to the
/// underlying subscription unchanged. Consumers ack the
/// `EventId` of envelopes they have successfully processed;
/// the typed wrapper exposes those ids through the `E::event_id`
/// method after deserialization.
pub struct TypedEventSubscription<E> {
    /// The underlying untyped subscription. Exposed for adapters
    /// that need to introspect the wire-level state; consumers
    /// should prefer the typed `next`/`ack`/`nack`/`close`
    /// helpers on this wrapper.
    pub inner: Box<dyn EventSubscription>,
    /// Phantom marker for the typed payload. Uses `fn() -> E`
    /// (a contravariant fn pointer) so `TypedEventSubscription`
    /// is covariant in `E` rather than invariant — the wrapper
    /// never mutably borrows an `E`.
    _marker: PhantomData<fn() -> E>,
}

impl<E> fmt::Debug for TypedEventSubscription<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedEventSubscription")
            .field("inner", &"<dyn EventSubscription>")
            .finish()
    }
}

impl<E: DomainEvent + DeserializeOwned + 'static> TypedEventSubscription<E> {
    /// Returns the next typed event, or `None` if the
    /// underlying subscription is closed.
    ///
    /// Envelopes whose `event_type` does not match
    /// `E::EVENT_TYPE` are silently skipped (no ack is issued
    /// for them; the bus will redeliver after the visibility
    /// timeout). Envelopes that match but fail to deserialize
    /// to `E` surface as `Some(Err(DomainError::Validation(_)))`.
    pub async fn next(&mut self) -> Option<Result<E>> {
        loop {
            match self.inner.next().await {
                Some(Ok(env)) => {
                    if env.event_type != E::EVENT_TYPE {
                        // Client-side re-application of the filter.
                        continue;
                    }
                    match serde_json::from_value::<E>(env.payload) {
                        Ok(event) => return Some(Ok(event)),
                        Err(e) => {
                            return Some(Err(DomainError::Validation(format!(
                                "subscribe_typed: failed to deserialize event {} \
                                 to type {}: {}",
                                env.event_type,
                                std::any::type_name::<E>(),
                                e,
                            ))));
                        }
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }

    /// Acknowledges processing of `event_id`. Idempotent.
    /// Delegates to the underlying subscription unchanged.
    pub async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome> {
        self.inner.ack(event_id).await
    }

    /// Negatively acknowledges `event_id`. `requeue = true`
    /// puts the event back on the bus; `requeue = false`
    /// routes it to the dead-letter queue. Delegates to the
    /// underlying subscription.
    pub async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome> {
        self.inner.nack(event_id, requeue).await
    }

    /// Closes the subscription, releasing adapter-level
    /// resources. Consumes the wrapper.
    pub async fn close(self: Box<Self>) -> Result<()> {
        self.inner.close().await
    }
}

/// A topic name. Variants are advisory; the bus builds the wire
/// string from the variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Topic {
    /// A whole domain. Wire string: the domain literal.
    Domain(&'static str),
    /// A specific aggregate in a domain. Wire string:
    /// `<domain>.<aggregate>`.
    Aggregate(&'static str, &'static str),
    /// A specific event type. Wire string: the event_type literal.
    EventType(&'static str),
    /// A specific tenant's events. Wire string: `tenant.<school_id>`.
    Tenant(SchoolId),
    /// Every event the bus knows about. Wire string: `*`.
    All,
}

impl Topic {
    /// Returns the wire string for this topic.
    #[must_use]
    pub fn wire(&self) -> String {
        match self {
            Self::Domain(d) => (*d).to_owned(),
            Self::Aggregate(d, a) => format!("{d}.{a}"),
            Self::EventType(t) => (*t).to_owned(),
            Self::Tenant(s) => format!("tenant.{}", s.as_uuid()),
            Self::All => "*".to_owned(),
        }
    }
}

/// A composable server-side filter. The bus may apply any of
/// these for efficiency; consumers MUST re-apply them
/// client-side because not every adapter can evaluate every
/// variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventFilter {
    /// Filter on the `event_type` string.
    EventType(&'static str),
    /// Filter on the `aggregate_type` string.
    AggregateType(&'static str),
    /// Filter on the `school_id`.
    SchoolId(SchoolId),
    /// Filter on a capability string. The capability namespace
    /// is owned by `educore-rbac::Capability`; for Phase 2 the
    /// filter is a `String` (stringly-typed) to avoid a circular
    /// `cross-cutting → cross-cutting` dependency. The bus
    /// matches this filter against a `payload["capability"]`
    /// field when present, or against the envelope's
    /// `event_type` prefix when not.
    Capability(String),
    /// A composable boolean expression.
    Expression(Box<EventFilterExpr>),
}

impl EventFilter {
    /// Returns `true` if the given envelope matches this filter.
    /// Consumers call this client-side as a defensive
    /// re-application; adapters may also call it server-side.
    #[must_use]
    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        match self {
            Self::EventType(t) => envelope.event_type == *t,
            Self::AggregateType(t) => envelope.aggregate_type == *t,
            Self::SchoolId(s) => envelope.school_id == *s,
            Self::Capability(s) => {
                // The capability may be carried either as a
                // top-level field of the payload (when the event
                // represents a capability check) or as the
                // first two segments of the event_type.
                envelope.payload.get("capability").and_then(|v| v.as_str()) == Some(s.as_str())
                    || envelope.event_type.starts_with(s.as_str())
            }
            Self::Expression(expr) => expr.matches(envelope),
        }
    }
}

/// A composable boolean expression of [`EventFilter`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventFilterExpr {
    /// Logical AND of two sub-expressions.
    And(Box<Self>, Box<Self>),
    /// Logical OR of two sub-expressions.
    Or(Box<Self>, Box<Self>),
    /// Logical NOT of a sub-expression.
    Not(Box<Self>),
    /// A leaf filter.
    Leaf(Box<EventFilter>),
}

impl EventFilterExpr {
    /// Returns `true` if the given envelope matches this
    /// expression.
    #[must_use]
    pub fn matches(&self, envelope: &EventEnvelope) -> bool {
        match self {
            Self::And(a, b) => a.matches(envelope) && b.matches(envelope),
            Self::Or(a, b) => a.matches(envelope) || b.matches(envelope),
            Self::Not(e) => !e.matches(envelope),
            Self::Leaf(f) => f.matches(envelope),
        }
    }
}

/// Where a subscription should start reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartPosition {
    /// Subscribe to events published after the subscription is
    /// created.
    Latest,
    /// Replay all events on the bus (subject to retention).
    Earliest,
    /// Start after the given event id.
    FromEventId(EventId),
    /// Start after the given timestamp.
    FromTimestamp(Timestamp),
}

/// A stable identifier for a consumer. The bus uses it for offset
/// tracking and observability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConsumerId(pub String);

impl ConsumerId {
    /// Constructs a new `ConsumerId` from a string. The string is
    /// expected to be stable across process restarts (e.g.
    /// `"finance.fee-assigner"`).
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the id as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ConsumerId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for ConsumerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for ConsumerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Acknowledgement that a single envelope was accepted by the bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishReceipt {
    /// The event id (echoed for caller convenience).
    pub event_id: EventId,
    /// The wire topic the envelope was published to.
    pub topic: String,
    /// The clock time the bus accepted the envelope. The bus
    /// adapter sets this; producers can compare against their
    /// own `occurred_at` to gauge ingestion latency.
    pub accepted_at: Timestamp,
}

impl PublishReceipt {
    /// Convenience constructor.
    #[must_use]
    pub fn new(event_id: EventId, topic: String, accepted_at: Timestamp) -> Self {
        Self {
            event_id,
            topic,
            accepted_at,
        }
    }
}

/// A per-envelope failure recorded by [`EventBus::publish_batch`]
/// when the adapter falls back to per-envelope `publish` (per
/// the bus-port contract). The relay uses these to decide
/// which outbox rows to retry: a failure here means the
/// envelope was NOT published, so the outbox row must remain
/// pending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchFailure {
    /// The event id of the envelope that failed. `None` only
    /// when the input couldn't even be parsed (e.g. malformed
    /// payload); adapters that successfully constructed an
    /// [`EventEnvelope`] always populate this field.
    pub event_id: Option<EventId>,
    /// Human-readable failure reason. Adapters SHOULD include
    /// the underlying error chain (e.g. `"transport closed:
    /// <inner>"`) so operators can debug without re-parsing
    /// logs.
    pub error: String,
}

impl BatchFailure {
    /// Convenience constructor.
    #[must_use]
    pub fn new(event_id: Option<EventId>, error: impl Into<String>) -> Self {
        Self {
            event_id,
            error: error.into(),
        }
    }
}

/// Acknowledgement that a batch of envelopes was accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReceipt {
    /// Per-envelope receipts, in the order the envelopes were
    /// submitted. Each entry corresponds to an envelope the
    /// adapter successfully published.
    pub receipts: Vec<PublishReceipt>,
    /// Per-envelope failures, in the order the envelopes were
    /// submitted. Each entry corresponds to an envelope the
    /// adapter failed to publish; the relay (or any other
    /// batch consumer) uses this list to decide which source
    /// rows to retry vs. mark-published.
    ///
    /// Invariant: `receipts.len() + failures.len() == submitted_count`,
    /// where `submitted_count` is the input to `publish_batch`.
    /// Adapters that don't track per-envelope failure (the
    /// legacy `?`-short-circuit shape) leave this empty and
    /// `is_fully_accepted()` returns `false` defensively —
    /// partial failure cannot be ruled out.
    pub failures: Vec<BatchFailure>,
    /// The correlation id of the batch, if any. Producers may
    /// stamp this on a wrapping envelope; consumers use it to
    /// track in-flight batches.
    pub correlation_id: Option<CorrelationId>,
}

impl BatchReceipt {
    /// Constructs a `BatchReceipt` with no correlation id.
    /// Used by adapters that always publish without one.
    #[must_use]
    pub fn new(receipts: Vec<PublishReceipt>, failures: Vec<BatchFailure>) -> Self {
        Self {
            receipts,
            failures,
            correlation_id: None,
        }
    }

    /// Returns `true` iff every input envelope was published
    /// successfully AND at least one envelope was submitted.
    ///
    /// The previous implementation (`!receipts.is_empty()`)
    /// reported a partial failure as a full success, which is
    /// the CC-EVT-007 audit finding. The corrected
    /// implementation inspects [`Self::failures`]; adapters
    /// that don't track per-envelope failure return an empty
    /// `failures` list but `is_fully_accepted()` still requires
    /// at least one successful receipt, so an all-failure batch
    /// (where the adapter short-circuited before recording any
    /// receipts) reports `false` defensively rather than
    /// silently true.
    #[must_use]
    pub fn is_fully_accepted(&self) -> bool {
        self.failures.is_empty() && !self.receipts.is_empty()
    }

    /// Returns the total number of envelopes the adapter
    /// accounted for (`receipts + failures`).
    #[must_use]
    pub fn total(&self) -> usize {
        self.receipts.len() + self.failures.len()
    }

    /// Returns the number of successful publishes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Returns `true` if no envelopes were published AND no
    /// failures were recorded. Used to distinguish "no work
    /// submitted" from "everything failed".
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty() && self.failures.is_empty()
    }
}

/// The audit sink port for the event bus.
///
/// Per [`docs/ports/event-bus.md`](../docs/ports/event-bus.md)
/// § Audit: "Every publish and consume is recorded in the
/// audit log. The audit record includes event id, event type,
/// actor (publisher), consumer id, and timestamp."
///
/// The bus port delegates audit-write calls to an `AuditSink`
/// implementation at two points in the envelope lifecycle:
///
/// - **`record_publish`** — invoked when [`EventBus::publish`]
///   (or [`EventBus::publish_batch`]) accepts an envelope.
///   The `publisher` is the envelope's [`actor_id`](EventEnvelope::actor_id).
///   The bus writes one row per envelope, after the underlying
///   transport has acknowledged acceptance so an audit row
///   never describes an envelope the bus later rejected.
/// - **`record_consume`** — invoked when a subscriber
///   successfully receives an envelope from
///   [`EventSubscription::next`]. The `consumer` identifies
///   which subscription yielded the envelope (sourced from
///   [`SubscribeOptions::consumer`]). The `publisher` remains
///   the envelope's original [`actor_id`](EventEnvelope::actor_id);
///   audit replay can correlate publish and consume rows by
///   `event_id`.
///
/// # Object safety
///
/// The trait is object-safe: the async methods use
/// `async_trait`, which keeps the futures boxed. Bus
/// implementations hold the sink as
/// `std::sync::Arc<dyn AuditSink>` and pass it across spawn
/// boundaries without generic-type plumbing.
///
/// # Failure handling
///
/// Implementations MUST NOT silently drop audit records. If the
/// underlying audit_log storage is unreachable, `record_publish`
/// and `record_consume` MUST return `Err(_)` so the bus can
/// decide whether to fail-fast (reject the publish / drop the
/// consume) or record-and-continue. Adapters that choose
/// record-and-continue MUST still log the failure via
/// `tracing::warn!` with the event id so operators can
/// reconcile the audit gap offline.
///
/// # Default implementation
///
/// [`NoopAuditSink`] is provided for tests and for adapter
/// configurations where audit is intentionally disabled (e.g.
/// ephemeral local-development runs where the audit_log table
/// is not provisioned). Production wiring MUST use an adapter
/// that forwards to the
/// [`AuditLog`](educore_storage::audit::AuditLog) port or an
/// equivalent audit_log sink; see
/// `docs/decisions/ADR-018-SyncEngine.md` for the cross-cutting
/// wiring convention.
#[async_trait]
pub trait AuditSink: Send + Sync + fmt::Debug {
    /// Record that the bus accepted `envelope` for publishing.
    /// The bus invokes this method AFTER the underlying
    /// transport has acknowledged acceptance (so the audit
    /// row never references an envelope the bus later
    /// rejected). The `publisher` for the audit record is the
    /// envelope's [`actor_id`](EventEnvelope::actor_id); the
    /// record's timestamp is the envelope's `occurred_at`.
    ///
    /// # Errors
    ///
    /// Implementations MUST return `Err(_)` if the audit row
    /// could not be persisted. Callers decide whether to
    /// fail-fast or record-and-continue.
    async fn record_publish(&self, envelope: &EventEnvelope) -> Result<()>;

    /// Record that a consumer received `envelope`. The
    /// `consumer` is the [`ConsumerId`] of the subscription
    /// that yielded the envelope. The `publisher` is the
    /// envelope's original [`actor_id`](EventEnvelope::actor_id);
    /// audit replay correlates publish and consume rows by
    /// `event_id`.
    ///
    /// # Errors
    ///
    /// Implementations MUST return `Err(_)` if the audit row
    /// could not be persisted. The consume path typically
    /// chooses record-and-continue (the envelope has already
    /// been delivered) and logs the failure via `tracing`.
    async fn record_consume(&self, envelope: &EventEnvelope, consumer: &ConsumerId) -> Result<()>;
}

/// A no-op [`AuditSink`] for tests and for adapter
/// configurations where audit is intentionally disabled.
///
/// `NoopAuditSink::record_publish` and `record_consume` both
/// return `Ok(())`. This is the only situation where the bus
/// may legitimately drop audit records; the choice is
/// explicit at the call site (the adapter constructs a
/// `NoopAuditSink` rather than omitting the `Arc<dyn
/// AuditSink>` field).
///
/// # When to use
///
/// - Unit and integration tests where audit wiring is out of
///   scope.
/// - Local-development binaries that intentionally skip the
///   audit_log table.
/// - Benchmarks where the audit path is the variable under
///   test and a no-op baseline is needed.
///
/// Production wiring MUST NOT use `NoopAuditSink`; use an
/// adapter that forwards to the audit_log port.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopAuditSink;

#[async_trait]
impl AuditSink for NoopAuditSink {
    async fn record_publish(&self, _envelope: &EventEnvelope) -> Result<()> {
        Ok(())
    }

    async fn record_consume(
        &self,
        _envelope: &EventEnvelope,
        _consumer: &ConsumerId,
    ) -> Result<()> {
        Ok(())
    }
}

/// The reason an envelope was routed to the dead-letter queue
/// (DLQ).
///
/// The DLQ is the terminal sink for envelopes that the bus or
/// its consumers cannot process. The variant tells operators
/// *why* a given envelope landed there so the right replay
/// policy can be chosen (re-publish, drop, or escalate).
///
/// # Why an enum
///
/// A free-form `String` reason is rejected: the operator
/// dashboard filters by reason, replay tooling keys off it,
/// and metrics aggregate per-variant counts. A `String` would
/// silently couple dashboards and replay scripts.
///
/// # When each variant is set
///
/// - [`NackRejected`](Self::NackRejected) — the consumer
///   called [`EventSubscription::nack`] with `requeue =
///   false`. This is the most common path; the consumer
///   decided the envelope was unprocessable (e.g. business
///   rule violation it does not intend to retry).
/// - [`MaxRetriesExceeded`](Self::MaxRetriesExceeded) — the
///   consumer retried up to the configured ceiling (typically
///   tracked by the adapter, not by the engine) and the
///   envelope is now poison.
/// - [`TimeoutExpired`](Self::TimeoutExpired) — the envelope
///   exceeded the consumer's processing budget without an
///   explicit nack; the adapter timed the consumer out and
///   routed the envelope to the DLQ on its behalf.
/// - [`SchemaMismatch`](Self::SchemaMismatch) — the envelope's
///   `schema_version` could not be deserialised by the
///   consumer, or the payload failed the consumer's
///   type-check before any business logic ran. Schema
///   mismatches are almost always terminal; replaying them
///   after a code change requires re-publishing from the
///   outbox, not re-driving the DLQ entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeadLetterReason {
    /// The consumer called `nack(requeue = false)` and asked
    /// for the envelope to be dead-lettered.
    NackRejected,
    /// The envelope exceeded the consumer's retry ceiling.
    MaxRetriesExceeded,
    /// The consumer's processing budget expired without an
    /// explicit ack or nack; the adapter routed the envelope
    /// to the DLQ on the consumer's behalf.
    TimeoutExpired,
    /// The envelope's `schema_version` did not match what the
    /// consumer could deserialise, or the payload failed a
    /// type-check before any business logic ran.
    SchemaMismatch,
}

/// A single entry in the dead-letter queue.
///
/// An entry is the (envelope, reason, attempt history)
/// triple that an operator inspects when triaging a DLQ
/// back-log. Adapters persist this shape verbatim; the bus
/// port treats it as the wire format for `list()`.
///
/// # Stability
///
/// The field set and order are part of the engine's public
/// API. Renames or removals are breaking changes and require
/// an ADR. New fields are additive and may be added in a
/// minor release.
///
/// # Why a struct (not a tuple)
///
/// Consumers in operator dashboards and replay scripts
/// pattern-match on field names. Tuple positions silently
/// shift when fields are inserted; named fields do not.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeadLetterEntry {
    /// The envelope that was dead-lettered. The full
    /// `EventEnvelope` is retained (not just `event_id`) so
    /// operators can inspect the payload and `schema_version`
    /// without cross-referencing the event log.
    pub envelope: EventEnvelope,
    /// Why the envelope was routed here. See
    /// [`DeadLetterReason`].
    pub reason: DeadLetterReason,
    /// How many delivery attempts the bus made before
    /// giving up. For [`NackRejected`](DeadLetterReason::NackRejected)
    /// entries this is typically 1; for
    /// [`MaxRetriesExceeded`](DeadLetterReason::MaxRetriesExceeded)
    /// it equals the configured ceiling.
    pub attempt_count: u32,
    /// The first time the bus observed this envelope. For
    /// retry-driven entries this is the original
    /// `occurred_at`; for `NackRejected` entries this is the
    /// time of the consumer's nack call.
    pub first_seen: Timestamp,
    /// The most recent delivery attempt's clock time.
    /// Operators compare this against `first_seen` to gauge
    /// how long an envelope sat in the DLQ between retries.
    pub last_attempt_at: Timestamp,
}

impl DeadLetterEntry {
    /// Convenience constructor. Adapters that build entries
    /// in a loop use this to avoid the verbose struct
    /// literal; tests and one-off operator scripts use it
    /// the same way.
    #[must_use]
    pub fn new(
        envelope: EventEnvelope,
        reason: DeadLetterReason,
        attempt_count: u32,
        first_seen: Timestamp,
        last_attempt_at: Timestamp,
    ) -> Self {
        Self {
            envelope,
            reason,
            attempt_count,
            first_seen,
            last_attempt_at,
        }
    }

    /// Returns the [`EventId`] of the dead-lettered envelope.
    /// Convenience for log lines and dashboards that only
    /// need the id; avoids a full envelope clone.
    #[must_use]
    pub fn event_id(&self) -> EventId {
        self.envelope.event_id
    }
}

/// The dead-letter queue (DLQ) port. Object-safe.
///
/// Per [`docs/ports/event-bus.md`](../docs/ports/event-bus.md):
/// "`nack(requeue=false)` routes the envelope to the
/// dead-letter queue. Operators inspect the queue via
/// `list()` and replay via a future `replay_dlq()` helper."
///
/// The DLQ is the terminal sink for envelopes the bus (or
/// its consumers) cannot process. It is a *separate* port
/// from [`EventBus`]: the bus moves envelopes through
/// delivery, the DLQ stores the ones delivery gave up on.
/// Adapters in `educore-event-bus` wire the DLQ into the
/// bus's `nack(requeue = false)` path; operators wire the
/// DLQ into their dashboards and replay tools.
///
/// # Implementations
///
/// - `InMemoryDeadLetterQueue` — `educore-event-bus` crate,
///   default feature. Useful for tests and ephemeral
///   single-process deployments.
/// - `DatabaseDeadLetterQueue` — `educore-event-bus` crate,
///   for production wiring. Persists entries to a
///   `dead_letter` table on the same engine database (or a
///   sidecar DLQ database, depending on operational
///   policy).
///
/// # Object safety
///
/// The trait is object-safe: all async methods use
/// `async_trait`, which keeps the futures boxed. Bus
/// implementations hold the queue as
/// `std::sync::Arc<dyn DeadLetterQueue>` and pass it across
/// spawn boundaries without generic-type plumbing. The
/// [`EventBus::dlq`] accessor returns the same shape.
///
/// # Failure handling
///
/// `send` MUST return `Err(_)` if the entry could not be
/// persisted. The bus adapter that called `send` decides
/// whether to fail-fast (reject the nack, leaving the
/// envelope in-flight) or record-and-continue (log via
/// `tracing::error!` with the event id and let the envelope
/// be redelivered after the visibility timeout). Both
/// policies are defensible; the port does not pick one.
///
/// `list` MUST return `Err(_)` if the entries could not be
/// read. The operator dashboard typically surfaces the
/// error directly.
///
/// # Default implementation
///
/// [`NoopDeadLetterQueue`] is provided for tests and for
/// adapter configurations where the DLQ is intentionally
/// disabled (e.g. ephemeral local-development runs where the
/// `dead_letter` table is not provisioned). Production wiring
/// MUST use an adapter that persists entries to durable
/// storage.
#[async_trait]
pub trait DeadLetterQueue: Send + Sync + fmt::Debug {
    /// Persist `envelope` to the DLQ with the given
    /// `reason`. The DLQ adapter stamps the `attempt_count`,
    /// `first_seen`, and `last_attempt_at` fields of the
    /// resulting [`DeadLetterEntry`]; callers supply only
    /// the envelope, reason, and attempt count.
    ///
    /// # Errors
    ///
    /// Returns `Err(_)` if the entry could not be persisted.
    /// See the trait-level docs for the fail-fast vs.
    /// record-and-continue policy decision.
    async fn send(
        &self,
        envelope: &EventEnvelope,
        reason: DeadLetterReason,
        attempt_count: u32,
    ) -> Result<()>;

    /// Return up to `limit` DLQ entries, in insertion order
    /// (oldest first). Operators call this from dashboards
    /// and replay scripts.
    ///
    /// The `limit` parameter caps the result set so a
    /// back-log with millions of entries does not blow the
    /// operator's memory. Adapters SHOULD clamp `limit` to a
    /// sane upper bound (e.g. 1..=1024) and reject 0.
    ///
    /// # Errors
    ///
    /// Returns `Err(_)` if the entries could not be read.
    async fn list(&self, limit: u32) -> Result<Vec<DeadLetterEntry>>;

    /// Drop a DLQ entry by `event_id`. Used by replay
    /// tooling after the entry has been re-published and
    /// acknowledged, so the DLQ does not grow without
    /// bound.
    ///
    /// The default implementation returns `Ok(())` so
    /// in-memory DLQ adapters that do not need explicit
    /// pruning (e.g. tests that discard the whole queue at
    /// process exit) do not have to implement it.
    ///
    /// # Errors
    ///
    /// Returns `Err(_)` if the entry could not be removed.
    /// Adapters that cannot find the entry also return
    /// `Err(_)`; "not found" is a normal operator scenario
    /// but the port treats it as an error so callers do not
    /// silently assume success.
    async fn drop_entry(&self, event_id: EventId) -> Result<()> {
        let _ = event_id;
        Ok(())
    }
}

/// A no-op [`DeadLetterQueue`] for tests and for adapter
/// configurations where the DLQ is intentionally disabled.
///
/// `NoopDeadLetterQueue::send` returns `Ok(())` without
/// persisting the entry; `list` returns `Ok(vec![])`. This
/// is the only situation where the bus may legitimately drop
/// dead-letter records; the choice is explicit at the call
/// site (the adapter constructs a `NoopDeadLetterQueue`
/// rather than omitting the `Arc<dyn DeadLetterQueue>`
/// handle).
///
/// # When to use
///
/// - Unit and integration tests where DLQ wiring is out of
///   scope.
/// - Local-development binaries that intentionally skip the
///   `dead_letter` table.
/// - Benchmarks where the DLQ path is the variable under
///   test and a no-op baseline is needed.
///
/// Production wiring MUST NOT use `NoopDeadLetterQueue`;
/// envelopes that should be dead-lettered will be silently
/// dropped.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopDeadLetterQueue;

#[async_trait]
impl DeadLetterQueue for NoopDeadLetterQueue {
    async fn send(
        &self,
        _envelope: &EventEnvelope,
        _reason: DeadLetterReason,
        _attempt_count: u32,
    ) -> Result<()> {
        Ok(())
    }

    async fn list(&self, _limit: u32) -> Result<Vec<DeadLetterEntry>> {
        Ok(Vec::new())
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

    fn sample_envelope(school: SchoolId) -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: "platform.school.created".to_owned(),
            schema_version: 1,
            school_id: school,
            aggregate_id: g.next_uuid(),
            aggregate_type: "school".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload: serde_json::json!({}),
        }
    }

    #[test]
    fn topic_wire_strings() {
        assert_eq!(Topic::Domain("platform").wire(), "platform");
        assert_eq!(
            Topic::Aggregate("platform", "school").wire(),
            "platform.school"
        );
        assert_eq!(Topic::EventType("SchoolCreated").wire(), "SchoolCreated");
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        assert_eq!(Topic::Tenant(s).wire(), format!("tenant.{}", s.as_uuid()));
        assert_eq!(Topic::All.wire(), "*");
    }

    #[test]
    fn filter_event_type_matches() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let env = sample_envelope(s);
        assert!(EventFilter::EventType("platform.school.created").matches(&env));
        assert!(!EventFilter::EventType("academic.student.admitted").matches(&env));
    }

    #[test]
    fn filter_school_id_matches() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let env = sample_envelope(s);
        assert!(EventFilter::SchoolId(s).matches(&env));
        let other = SchoolId::from_uuid(uuid::Uuid::from_u128(1));
        assert!(!EventFilter::SchoolId(other).matches(&env));
    }

    #[test]
    fn filter_aggregate_type_matches() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let env = sample_envelope(s);
        assert!(EventFilter::AggregateType("school").matches(&env));
        assert!(!EventFilter::AggregateType("user").matches(&env));
    }

    #[test]
    fn filter_capability_via_event_type_prefix() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let env = sample_envelope(s);
        assert!(EventFilter::Capability("platform.school".to_owned()).matches(&env));
        assert!(!EventFilter::Capability("rbac".to_owned()).matches(&env));
    }

    #[test]
    fn filter_capability_via_payload_field() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let mut env = sample_envelope(s);
        env.payload = serde_json::json!({"capability": "Platform.User.Read"});
        assert!(EventFilter::Capability("Platform.User.Read".to_owned()).matches(&env));
    }

    #[test]
    fn filter_expression_and_or_not() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let env = sample_envelope(s);
        let expr = EventFilterExpr::And(
            Box::new(EventFilterExpr::Leaf(Box::new(EventFilter::AggregateType(
                "school",
            )))),
            Box::new(EventFilterExpr::Leaf(Box::new(EventFilter::SchoolId(s)))),
        );
        assert!(expr.matches(&env));
        let expr = EventFilterExpr::Or(
            Box::new(EventFilterExpr::Leaf(Box::new(EventFilter::AggregateType(
                "user",
            )))),
            Box::new(EventFilterExpr::Leaf(Box::new(EventFilter::SchoolId(s)))),
        );
        assert!(expr.matches(&env));
        let expr = EventFilterExpr::Not(Box::new(EventFilterExpr::Leaf(Box::new(
            EventFilter::AggregateType("user"),
        ))));
        assert!(expr.matches(&env));
    }

    #[test]
    fn subscribe_options_default_construction() {
        let opts = SubscribeOptions::for_consumer(ConsumerId::new("test-consumer"), Topic::All);
        assert_eq!(opts.batch_size, 32);
        assert_eq!(opts.visibility_timeout, Duration::from_secs(300));
        assert_eq!(opts.start, StartPosition::Latest);
    }

    #[test]
    fn consumer_id_round_trip() {
        let id = ConsumerId::new("finance.fee-assigner");
        assert_eq!(id.as_str(), "finance.fee-assigner");
        let from_str: ConsumerId = "x".into();
        assert_eq!(from_str.0, "x");
    }

    /// Local `DomainEvent` for typed-subscription tests. Mirrors
    /// the shape of the `TestEvent` in `domain_event::tests` but
    /// is duplicated here so the event-bus tests do not depend on
    /// a private symbol from the `domain_event` module.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TypedBusTestEvent {
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: uuid::Uuid,
        occurred_at: Timestamp,
        name: String,
    }

    impl DomainEvent for TypedBusTestEvent {
        const EVENT_TYPE: &'static str = "test.event.created";
        const SCHEMA_VERSION: u32 = 1;
        const AGGREGATE_TYPE: &'static str = "test_event";
        fn event_id(&self) -> EventId {
            self.event_id
        }
        fn aggregate_id(&self) -> uuid::Uuid {
            self.aggregate_id
        }
        fn school_id(&self) -> SchoolId {
            self.school_id
        }
        fn occurred_at(&self) -> Timestamp {
            self.occurred_at
        }
    }

    /// A mock `EventSubscription` that yields a fixed queue of
    /// envelopes (in order) and reports Accepted for ack/nack.
    /// Used by the `subscribe_typed` tests to exercise the
    /// client-side filter and deserialization paths without
    /// standing up a full bus adapter.
    struct MockSubscription {
        envelopes: std::collections::VecDeque<EventEnvelope>,
    }

    #[async_trait::async_trait]
    impl EventSubscription for MockSubscription {
        async fn next(&mut self) -> Option<Result<EventEnvelope>> {
            self.envelopes.pop_front().map(Ok)
        }

        async fn ack(&mut self, _event_id: EventId) -> Result<AckOutcome> {
            Ok(AckOutcome::Accepted)
        }

        async fn nack(&mut self, _event_id: EventId, _requeue: bool) -> Result<AckOutcome> {
            Ok(AckOutcome::Accepted)
        }

        async fn close(self: Box<Self>) -> Result<()> {
            Ok(())
        }
    }

    fn typed_matching_envelope(school: SchoolId, name: &str) -> EventEnvelope {
        let mut env = sample_envelope(school);
        env.event_type = TypedBusTestEvent::EVENT_TYPE.to_owned();
        // Round-trip the payload through `serde_json::to_value` to
        // match the wire format the typed subscription will
        // deserialise from in production.
        let event = TypedBusTestEvent {
            event_id: env.event_id,
            school_id: school,
            aggregate_id: env.aggregate_id,
            occurred_at: env.occurred_at,
            name: name.to_owned(),
        };
        env.payload = serde_json::to_value(&event).expect("serialize test event");
        env
    }

    fn typed_non_matching_envelope(school: SchoolId) -> EventEnvelope {
        let mut env = sample_envelope(school);
        env.event_type = "other.event.created".to_owned();
        env
    }

    #[tokio::test]
    async fn subscribe_typed_filters_by_event_type() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let mut envelopes = std::collections::VecDeque::new();
        // Order: non-matching, matching, non-matching, matching,
        // non-matching — the wrapper must skip the non-matching
        // envelopes and yield only the two matching ones.
        envelopes.push_back(typed_non_matching_envelope(s));
        envelopes.push_back(typed_matching_envelope(s, "first"));
        envelopes.push_back(typed_non_matching_envelope(s));
        envelopes.push_back(typed_matching_envelope(s, "second"));
        envelopes.push_back(typed_non_matching_envelope(s));

        let mock = MockSubscription { envelopes };
        let inner: Box<dyn EventSubscription> = Box::new(mock);
        let mut typed: TypedEventSubscription<TypedBusTestEvent> = TypedEventSubscription {
            inner,
            _marker: PhantomData,
        };

        let first = typed
            .next()
            .await
            .expect("subscription yielded Some")
            .expect("first event deserialised");
        assert_eq!(first.name, "first");

        let second = typed
            .next()
            .await
            .expect("subscription yielded Some")
            .expect("second event deserialised");
        assert_eq!(second.name, "second");

        // The remaining envelope is non-matching, so the wrapper
        // must consume it and return None (the underlying
        // subscription is empty).
        assert!(typed.next().await.is_none());
    }

    #[tokio::test]
    async fn subscribe_typed_returns_none_when_underlying_closes() {
        let mock = MockSubscription {
            envelopes: std::collections::VecDeque::new(),
        };
        let inner: Box<dyn EventSubscription> = Box::new(mock);
        let mut typed: TypedEventSubscription<TypedBusTestEvent> = TypedEventSubscription {
            inner,
            _marker: PhantomData,
        };
        assert!(typed.next().await.is_none());
    }

    #[tokio::test]
    async fn subscribe_typed_surfaces_deserialization_errors() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        // Matching event_type but the payload is missing the
        // `name` field — deserialization must fail.
        let mut env = sample_envelope(s);
        env.event_type = TypedBusTestEvent::EVENT_TYPE.to_owned();
        env.payload = serde_json::json!({
            "event_id": env.event_id,
            "school_id": s,
            "aggregate_id": env.aggregate_id,
            "occurred_at": env.occurred_at,
        });
        let mock = MockSubscription {
            envelopes: std::collections::VecDeque::from(vec![env]),
        };
        let inner: Box<dyn EventSubscription> = Box::new(mock);
        let mut typed: TypedEventSubscription<TypedBusTestEvent> = TypedEventSubscription {
            inner,
            _marker: PhantomData,
        };
        let err = typed
            .next()
            .await
            .expect("subscription yielded Some")
            .expect_err("deserialization must fail");
        let DomainError::Validation(msg) = err else {
            panic!("expected DomainError::Validation, got {err:?}");
        };
        assert!(msg.contains("subscribe_typed"), "unexpected message: {msg}");
    }

    #[tokio::test]
    async fn typed_subscribe_options_default_construction() {
        let opts = TypedSubscribeOptions::for_consumer(
            ConsumerId::new("test-consumer"),
            Topic::All,
        );
        assert_eq!(opts.batch_size, 32);
        assert_eq!(opts.visibility_timeout, Duration::from_secs(300));
        assert_eq!(opts.start, StartPosition::Latest);
    }

    #[tokio::test]
    async fn typed_subscription_delegates_ack_nack_close() {
        let s = SchoolId::from_uuid(uuid::Uuid::nil());
        let event_id = SystemIdGen.next_event_id();
        let mut env = sample_envelope(s);
        env.event_type = TypedBusTestEvent::EVENT_TYPE.to_owned();
        env.payload = serde_json::json!({
            "event_id": event_id,
            "school_id": s,
            "aggregate_id": env.aggregate_id,
            "occurred_at": env.occurred_at,
            "name": "ack-me".to_owned(),
        });
        let mock = MockSubscription {
            envelopes: std::collections::VecDeque::from(vec![env]),
        };
        let inner: Box<dyn EventSubscription> = Box::new(mock);
        let mut typed: TypedEventSubscription<TypedBusTestEvent> = TypedEventSubscription {
            inner,
            _marker: PhantomData,
        };
        let _event = typed.next().await.expect("some").expect("ok");
        // ack / nack / close all delegate to the underlying
        // subscription (the MockSubscription returns Accepted).
        assert_eq!(typed.ack(event_id).await.expect("ack"), AckOutcome::Accepted);
        assert_eq!(
            typed.nack(event_id, false).await.expect("nack"),
            AckOutcome::Accepted
        );
        Box::new(typed).close().await.expect("close");
    }

    #[test]
    fn typed_subscription_debug_includes_inner() {
        let mock = MockSubscription {
            envelopes: std::collections::VecDeque::new(),
        };
        let inner: Box<dyn EventSubscription> = Box::new(mock);
        let typed: TypedEventSubscription<TypedBusTestEvent> = TypedEventSubscription {
            inner,
            _marker: PhantomData,
        };
        let dbg = format!("{typed:?}");
        assert!(dbg.contains("TypedEventSubscription"), "debug: {dbg}");
        assert!(dbg.contains("inner"), "debug: {dbg}");
    }
}
