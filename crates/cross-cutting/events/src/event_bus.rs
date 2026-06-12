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
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

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

/// Acknowledgement that a batch of envelopes was accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchReceipt {
    /// Per-envelope receipts, in the order the envelopes were
    /// submitted.
    pub receipts: Vec<PublishReceipt>,
    /// The correlation id of the batch, if any. Producers may
    /// stamp this on a wrapping envelope; consumers use it to
    /// track in-flight batches.
    pub correlation_id: Option<CorrelationId>,
}

impl BatchReceipt {
    /// Returns `true` if every receipt in the batch succeeded.
    /// Adapters that don't support atomic batching always return
    /// `true` here (the per-receipt `PublishReceipt` carries the
    /// per-envelope status, but the trait doesn't model
    /// per-receipt failure for batches; producers that need
    /// that granularity call `publish` in a loop).
    #[must_use]
    pub fn is_fully_accepted(&self) -> bool {
        !self.receipts.is_empty()
    }

    /// Returns the number of envelopes in the batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Returns `true` if the batch is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
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
            event_type: "platform.school.created",
            schema_version: 1,
            school_id: school,
            aggregate_id: g.next_uuid(),
            aggregate_type: "school",
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
}
