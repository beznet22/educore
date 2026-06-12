# Event Bus Port

## Purpose

The event bus port decouples the engine from any specific messaging
infrastructure. The engine publishes domain events to the bus and
subscribes to events from other domains and external systems.

## Trait: `EventBus`

```rust
#[async_trait]
pub trait EventBus: Send + Sync + std::fmt::Debug {
    async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;
    async fn subscribe(&self, options: SubscribeOptions) -> Result<EventSubscription>;
}
```

The trait is object-safe.

## EventEnvelope

```rust
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub schema_version: u32,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub published_at: Option<Timestamp>,
    pub payload: serde_json::Value,   // for cross-domain transport
}
```

The `payload` is `serde_json::Value` **only at the bus boundary**.
Inside the engine, payloads are typed Rust structs. Adapters serialize
to JSON for cross-process transport.

## SubscribeOptions

```rust
pub struct SubscribeOptions {
    pub consumer: ConsumerId,
    pub topic: Topic,
    pub filter: Option<EventFilter>,
    pub start: StartPosition,
    pub batch_size: u32,
    pub visibility_timeout: Duration,
}

pub enum Topic {
    Domain(&'static str),            // e.g. Domain::Academic
    Aggregate(&'static str, &'static str),
    EventType(&'static str),
    Tenant(SchoolId),
    All,
}

pub enum EventFilter {
    EventType(&'static str),
    AggregateType(&'static str),
    SchoolId(SchoolId),
    Capability(Capability),
    Expression(EventFilterExpr),     // composable
}

pub enum StartPosition {
    Latest,                          // subscribe to events published after subscribe
    Earliest,                        // replay all events (used for new projections)
    FromEventId(EventId),
    FromTimestamp(Timestamp),
}
```

## EventSubscription

```rust
#[async_trait]
pub trait EventSubscription: Send + Sync {
    async fn next(&mut self) -> Option<Result<EventEnvelope>>;
    async fn ack(&mut self, event_id: EventId) -> Result<()>;
    async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>;
    async fn close(self: Box<Self>) -> Result<()>;
}
```

The subscription is a long-lived async iterator. Consumers process
events at their own pace. The bus tracks consumer offsets.

## Outbox Pattern

The engine writes events to an outbox table within the same database
transaction as the domain state change. The outbox relay (a separate
process) reads pending events from the outbox and publishes them to
the bus. On success, the relay marks the events as published.

This guarantees:

- The state change and the event are atomically committed.
- Events are eventually published even if the bus is down.
- Consumers never see events for state changes that were rolled back.

The outbox is implemented by the storage adapter (see `storage.md`).

## At-Least-Once Delivery

The bus provides at-least-once delivery. Consumers MUST be
idempotent. The `EventId` is the idempotency key.

## Exactly-Once (Application Level)

Consumers can achieve exactly-once processing by:

1. Recording the consumed `EventId` in a "processed events" table
   within the same transaction as the side effect.
2. Skipping events already in the table.

The engine does not enforce this; the consumer implements it.

## Dead Letter Queue

Events that fail repeatedly (configurable N retries) are routed to a
dead letter queue. The consumer can inspect the DLQ, fix the issue,
and re-publish.

## Schema Versioning

Events carry `schema_version`. Consumers that handle a newer version
may ignore older ones (if their effect is already applied) or migrate.
Producers never send a payload that does not match the declared
schema version.

Schema migrations are managed by the engine. When a domain event's
shape changes, the engine publishes a new schema version and
consumers adapt.

## Replay

`StartPosition::Earliest` enables replay. New projections can
reconstruct state by replaying all events. The bus may enforce a
retention policy (e.g. 90 days of history).

## Topic Naming

Topic names follow `<domain>.<aggregate>` (e.g. `academic.student`,
`finance.invoice`). Tenant topics are `tenant.<school_id>`.

## In-Process Bus (Default)

The default implementation is an in-process bus. All subscribers run
in the same process as the engine. This is sufficient for single-node
deployments and tests.

```rust
let bus: Arc<dyn EventBus> = Arc::new(InProcessBus::new());
```

The in-process bus is MPMC (multi-producer, multi-consumer) and uses
a bounded channel per subscription.

## Distributed Bus

Distributed adapters are consumer-supplied. The bus trait is
intentionally minimal so any messaging system can be implemented.

Common adapters:

- `NatsBus` (NATS JetStream)
- `RedisStreamsBus` (Redis Streams)
- `KafkaBus` (Apache Kafka)
- `RabbitMqBus` (RabbitMQ)

The engine does not ship these. Consumers implement them.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("topic not found: {0}")] TopicNotFound(Topic),
    #[error("subscription closed")] SubscriptionClosed,
    #[error("publish failed: {0}")] PublishFailed(String),
    #[error("deserialize failed: {0}")] DeserializeFailed(String),
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

## Worked Example

A consumer subscribes to `StudentAdmitted`:

```rust
let mut sub = engine.events().subscribe(SubscribeOptions {
    consumer: ConsumerId::new("welcome-emailer"),
    topic: Topic::EventType("StudentAdmitted"),
    filter: None,
    start: StartPosition::Latest,
    batch_size: 32,
    visibility_timeout: Duration::from_minutes(5),
}).await?;

while let Some(event) = sub.next().await {
    let event = event?;
    handle_student_admitted(&event).await?;
    sub.ack(event.event_id).await?;
}
```

A cross-domain subscriber (Finance subscribes to StudentAdmitted to
auto-assign fees):

```rust
engine.events().subscribe(SubscribeOptions {
    consumer: ConsumerId::new("finance.fee-assigner"),
    topic: Topic::EventType("StudentAdmitted"),
    filter: None,
    start: StartPosition::Latest,
    batch_size: 16,
    visibility_timeout: Duration::from_minutes(2),
}).await?;
```

## Object Safety

`EventBus` and `EventSubscription` are object-safe.

## Testing

- Unit tests of publish, subscribe, ack, nack.
- Integration tests of outbox + relay.
- A test of at-least-once delivery.
- A test of dead letter routing.
- A test of replay.
- A test of consumer offset tracking.
- A test of schema version handling.
- A test of cross-tenant isolation in subscriptions.

## Offline Mode

In offline mode, the outbox accumulates events. The bus (in-process
during offline) operates normally. On reconnect, the outbox relay
publishes accumulated events to the distributed bus.

## Audit

Every publish and consume is recorded in the audit log. The audit
record includes event id, event type, actor (publisher), consumer id,
and timestamp.

## Sync and the Event Bus

The event bus port has two consumers in the Educore architecture,
each backed by the same logical `EventBus` trait shape but a
different physical transport:

1. **The in-process event bus** — used by the domain core to publish
   events from commands. It is consumed by event handlers,
   projections, the audit sink, and the outbox. This is the default
   `InProcessBus` described above; it runs in the same process as
   the engine.

2. **The central fan-out** — used by the central engine to publish
   events to subscribed clients. Its physical transport is
   typically NATS JetStream, Redis pub/sub, or an in-process
   channel (single-node central), but the trait surface is
   identical to the in-process bus. Distributed adapters
   (NATS, Redis, Kafka, RabbitMQ) implement this transport.

The **local client's** `EventBus` is the in-process one. It does
**not** receive central events directly. Central events reach the
local client via the **sync engine** — specifically via
`SyncAdapter::subscribe`, which subscribes to the central `EventBus`
and pulls events down to the local process. The local client then
re-publishes the received events onto its own in-process bus for
local handlers, projections, and the outbox relay.

In other words, the sync engine is a bridge between two
`EventBus` instances: the central distributed bus (the
publishing/transport side) and the local in-process bus (the
consuming/domain side). The `EventBus` trait itself is unaware of
this bridge; the awareness lives in the sync adapter.

See `docs/ports/sync.md` and
[`docs/decisions/ADR-018-SyncEngineArchitecture.md`](docs/decisions/ADR-018-SyncEngineArchitecture.md)
for the full picture.
