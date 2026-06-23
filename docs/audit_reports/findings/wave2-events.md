## Wave 2 Cross-Cutting Audit Report — `educore-events` (Envelope + Bus Port)

**Scope:** `crates/cross-cutting/events/` (the `educore-events` package — `Cargo.toml`, `src/lib.rs`, `src/domain_event.rs`, `src/envelope.rs`, `src/event_bus.rs`, `src/errors.rs`, `src/outbox.rs`, `src/sync.rs`), the bus-port contract (`docs/ports/event-bus.md`), the wire-format schema (`docs/schemas/event-schema.md`), the events-domain spec (`docs/specs/events/events.md`, `docs/specs/events/commands.md`), ADR-005 (event-driven design), the Phase 2 deliverable (`docs/handoff/PHASE-2-HANDOFF.md`), `docs/build-plan.md` Phase 2 block, and a skim of the `educore-event-bus` adapter (`crates/adapters/event-bus/`). The calendar bounded-context crate `educore-events-domain` is out of scope. Findings only — no fixes proposed.

**Total findings:** 28

---

### FINDING 1

- **id:** CC-EVT-001
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/envelope.rs:47-103`
- **description:** `EventEnvelope` is missing the `recorded_at: Timestamp` field that `docs/schemas/event-schema.md` § 1 declares as a mandatory wire-format field. The schema spec explicitly defines `recorded_at` as "the clock time of persistence (>= occurred_at)" and shows it in the canonical JSON sample (§ 3), but the Rust envelope only carries `occurred_at` and `published_at`. `published_at` is set by the bus adapter on `publish` and is not equivalent to `recorded_at` (which is set at outbox persistence time per the storage-port `EventLogEntry::from_serialized_envelope` at `crates/infra/storage/src/event_log.rs:175-187`). Events emitted through `into_envelope` carry no `recorded_at` and consumers cannot compute ingestion latency from the envelope alone.
- **expected:** `docs/schemas/event-schema.md:32` (`recorded_at: Timestamp`) and `:77` (JSON sample) declare `recorded_at` as a required wire field.
- **evidence:**
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
      pub payload: serde_json::Value,
  }
  ```

---

### FINDING 2

- **id:** CC-EVT-002
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/envelope.rs:47-103`
- **description:** `EventEnvelope` carries no `metadata: EventMetadata` field, but `docs/schemas/event-schema.md` § 1 declares `metadata: EventMetadata` (an open-ended, versioned key map) as a required envelope field and § 6 enumerates the recommended keys (`source`, `user_agent`, `ip`, `request_id`, `device_id`, `session_id`, `geo`, `feature_flags`, `trace`). The audit / outbox / central-fan-out / sync consumers cannot stamp distributed-trace ids, source-channel (`web` / `mobile` / `api` / `agent` / `import` / `system`), or request id onto the wire without a field on the envelope. The `RawPayload` helper at `domain_event.rs:174-204` exists but carries only `correlation_id` and `actor_id` and is never invoked by any producer.
- **expected:** `docs/schemas/event-schema.md:39` (`metadata: EventMetadata`) and § 6 key catalogue.
- **evidence:**
  ```rust
  // No `metadata` field in the struct above. The schema spec
  // requires metadata; the field is absent.
  ```
  And at `crates/cross-cutting/events/src/domain_event.rs:178-204`:
  ```rust
  pub struct RawPayload {
      pub payload: serde_json::Value,
      pub correlation_id: CorrelationId,
      pub actor_id: UserId,
  }
  ```
  No `metadata` carrier exists; `RawPayload::new` is never called from any other crate (`grep RawPayload crates/` returns only the definition site and its `impl` block).

---

### FINDING 3

- **id:** CC-EVT-003
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/event_bus.rs:55-65, 84-117` and `docs/ports/event-bus.md:60-66`
- **description:** `EventSubscription::ack` and `EventSubscription::nack` return `Result<AckOutcome>` in code, but the bus-port contract at `docs/ports/event-bus.md:60-66` specifies `Result<()>` for both. The deviation is non-trivial: any consumer written against the port-doc signature will not compile (or will silently discard the `AckOutcome::Unknown` / `Failed` discriminant) against the actual trait. Additionally, `AckOutcome::Unknown` is unrepresentable in the spec contract; the spec requires that `ack` be idempotent and the return type be `()`. There is no migration note in either the bus-port doc or the Phase 2 hand-off acknowledging this divergence.
- **expected:** `docs/ports/event-bus.md:60-66`:
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<()>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>;
  ```
- **evidence:**
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  async fn nack(&mut self, event_id: Event_id: EventId, requeue: bool) -> Result<AckOutcome>;
  ```

---

### FINDING 4

- **id:** CC-EVT-004
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/domain_event.rs:54-122` and `docs/specs/events/events.md:10-16`
- **description:** The `DomainEvent` trait shipped in code does not match the shape declared in `docs/specs/events/events.md:10-16`. The spec declares:
  ```rust
  pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
      const TYPE: &'static str;
      fn aggregate_id(&self) -> Uuid;
      fn school_id(&self) -> SchoolId;
      fn occurred_at(&self) -> Timestamp;
  }
  ```
  but the code declares `DomainEvent: Send + Sync + 'static` (no `Serialize + DeserializeOwned` bound), renames the const from `TYPE` to `EVENT_TYPE`, and adds two extra consts (`SCHEMA_VERSION`, `AGGREGATE_TYPE`) plus an `event_id()` getter. All 10+ domain impls in `crates/domains/*/src/events.rs` (e.g. `crates/domains/finance/src/events.rs:71-73`, `crates/domains/hr/src/events.rs:121-123`) implement the code-shape, not the spec-shape, so the spec and the code have drifted in opposite directions: code has more required surface, spec has fewer required bounds.
- **expected:** `docs/specs/events/events.md:10-16` spec text quoted above.
- **evidence:**
  ```rust
  pub trait DomainEvent: Send + Sync + 'static {
      const EVENT_TYPE: &'static str;
      const SCHEMA_VERSION: u32;
      const AGGREGATE_TYPE: &'static str;
      fn event_id(&self) -> EventId;
      fn aggregate_id(&self) -> Uuid;
      fn school_id(&self) -> SchoolId;
      fn occurred_at(&self) -> Timestamp;
      fn to_value(&self) -> serde_json::Value where Self: Serialize { ... }
      fn into_envelope(self, ctx: &TenantContext) -> EventEnvelope where Self: Sized + Serialize { ... }
  }
  ```

---

### FINDING 5

- **id:** CC-EVT-005
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/` (whole crate) and `docs/schemas/event-schema.md:128-145`
- **description:** `docs/schemas/event-schema.md` § 7 ("Schema Registry") mandates a port with two public methods: `engine.events.list()` returning `(event_type, current_version, deprecated_versions)`, and `engine.events.schema(event_type, version)` returning the JSON schema. The `educore-events` crate ships no such port, no `SchemaRegistry` trait, no in-memory default implementation, and no re-export of an `engine.events` facade. Phase 2 hand-off § "Open questions" does not mention this gap. The 6th cross-cutting table (`schema_registry`) referenced in `docs/build-plan.md` Phase 2 task 6 has a DDL but no port surface for it in `educore-events`.
- **expected:** `docs/schemas/event-schema.md:128-145` (Schema Registry section quoted above).
- **evidence:**
  ```text
  $ grep -rn 'SchemaRegistry\|schema_registry\|events.list\|events.schema' crates/cross-cutting/events/
  (no matches)
  ```

---

### FINDING 6

- **id:** CC-EVT-006
- **area:** cross-cutting-events
- **severity:** Critical
- **location:** `crates/cross-cutting/events/src/event_bus.rs:52-66` and `docs/ports/event-bus.md:22-26`
- **description:** `EventBus::subscribe` is declared as `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>` in code, but the bus-port contract at `docs/ports/event-bus.md:22-26` declares it as `Result<EventSubscription>` (no box, no `dyn`). The deviation from the doc is consistent with object-safety, but the spec doc is the contract that downstream consumers (Phase 3+ domain subscribers) will code against. The worked-example subscription code in `docs/ports/event-bus.md:177-197` writes `let mut sub = engine.events().subscribe(...)` without any `Box<dyn _>` deref, so it cannot compile against the actual trait signature.
- **expected:** `docs/ports/event-bus.md:24` `async fn subscribe(&self, options: SubscribeOptions) -> Result<EventSubscription>;`
- **evidence:**
  ```rust
  async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  ```
  vs.
  ```rust
  // From docs/ports/event-bus.md worked example:
  let mut sub = engine.events().subscribe(SubscribeOptions { ... }).await?;
  ```

---

### FINDING 7

- **id:** CC-EVT-007
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/cross-cutting/events/src/event_bus.rs:280-296`
- **description:** `BatchReceipt::is_fully_accepted()` documents itself as "Returns `true` if every receipt in the batch succeeded" but its implementation is `!self.receipts.is_empty()`. The body of the function contradicts the doc comment: a batch with one successful and one failed receipt will report `is_fully_accepted() == true`. A producer relying on this method to gate downstream work (e.g. an outbox relay that only deletes the source rows when the batch is fully accepted) will silently delete rows for partially-failed batches. The in-process bus adapter at `crates/adapters/event-bus/src/in_process.rs:202-219` short-circuits on the first failure inside `publish_batch`, so the bug is latent today, but the doc/API contract is wrong.
- **expected:** Doc comment matches behaviour: `is_fully_accepted` should iterate `receipts` and assert a per-receipt success field, OR the doc should be rewritten to "returns true iff any receipt was produced" and the field set extended with a per-receipt status.
- **evidence:**
  ```rust
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
  ```

---

### FINDING 8

- **id:** CC-EVT-008
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:472-487`
- **description:** The in-process bus `ack` and `nack` are no-ops that return `AckOutcome::Accepted` without doing anything. Per `docs/ports/event-bus.md:104-108` ("Dead Letter Queue. Events that fail repeatedly (configurable N retries) are routed to a dead letter queue") and ADR-005 § "Decision" item 4 ("outbox + relay pattern is mandatory for at-least-once delivery"), the bus-port promises at-least-once delivery with retry + DLQ. The in-process adapter has no retry counter, no DLQ sink, no `requeue` semantics on `nack`, and no `visibility_timeout` enforcement — the `visibility_timeout` field on `SubscribeOptions` is consumed by no code path. A consumer that calls `nack(id, true)` expecting the envelope to re-arrive will silently lose it.
- **expected:** Bus-port contract § "Dead Letter Queue" and § "At-Least-Once Delivery"; `docs/ports/event-bus.md:104-108`.
- **evidence:**
  ```rust
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
  ```

---

### FINDING 9

- **id:** CC-EVT-009
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/cross-cutting/events/src/domain_event.rs:107-115` and `crates/cross-cutting/events/src/outbox.rs:32-46`
- **description:** Both `DomainEvent::to_value` (the default payload serializer) and `outbox::payload_bytes` / `outbox::envelope_bytes` silently swallow serialization failures via `.unwrap_or(serde_json::Value::Null)` / `.unwrap_or_default()`. A producer that constructs an event with an unserializable field (e.g. a non-`Serialize` value inside a `serde_json::Value` it didn't construct itself, a `SecretString` without `Serialize`, a f32 NaN, or any future `Serialize` impl bug) will publish a `payload: null` envelope onto the bus. Consumers that do not null-check will then deserialize the broken event and crash, or worse, store a meaningless `null` payload in the event log. The doc-comment on `to_value` notes the relaxation but does not flag it as a contract violation. `AGENTS.md` forbids `unwrap`/`expect`/`panic` in production paths; `.unwrap_or_default()` here is a silent form of the same anti-pattern.
- **expected:** Either (a) `to_value` returns `Result<serde_json::Value, EventError>` and `payload_bytes` propagates the error, or (b) the outbox/appender layer rejects the event with a `DomainError::Validation` before the bad payload reaches the bus.
- **evidence:**
  ```rust
  fn to_value(&self) -> serde_json::Value
  where
      Self: Serialize,
  {
      serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
  }
  ```
  And at `crates/cross-cutting/events/src/outbox.rs:32-46`:
  ```rust
  pub fn payload_bytes(envelope: &EventEnvelope) -> bytes::Bytes {
      bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default())
  }
  ```

---

### FINDING 10

- **id:** CC-EVT-010
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/cross-cutting/events/src/event_bus.rs:159-183, 220-244` and `crates/adapters/event-bus/src/in_process.rs:301-318`
- **description:** The two filter-routing paths in the bus port — `Topic::Aggregate(d, a)` matched in the in-process adapter via `topic_matches`, and `EventFilter::AggregateType(t)` matched via `filter_matches` — disagree on what "aggregate" means. `Topic::Aggregate("platform", "school")` matches an envelope whose `aggregate_topic()` (i.e. domain prefix + `aggregate_type`) equals `platform.school`. `EventFilter::AggregateType("school")` matches the envelope's raw `aggregate_type` field, which is just `"school"`. A subscriber that subscribes to `Topic::Aggregate("platform", "school")` AND attaches a defensive `EventFilter::AggregateType("school")` to the same `SubscribeOptions` will receive an envelope that passes the topic check but fail the filter check on every event whose `aggregate_type` contains the domain prefix (e.g. `platform_school` in some encodings). The two naming schemes are mixed in the same file and have no shared helper.
- **expected:** Either both `Topic` and `EventFilter` operate on the same field, or the docs explicitly state the asymmetry.
- **evidence:**
  ```rust
  // In topic_matches (crates/adapters/event-bus/src/in_process.rs:301-318):
  Topic::Aggregate(d, a) => env.aggregate_topic() == format!("{d}.{a}"),

  // In filter_matches (crates/cross-cutting/events/src/event_bus.rs:220-244):
  Self::AggregateType(t) => envelope.aggregate_type == *t,
  ```
  Note: `aggregate_topic()` (envelope.rs:111-122) returns `{domain}.{aggregate_type}`, e.g. `platform.school`; `aggregate_type` is just `school`.

---

### FINDING 11

- **id:** CC-EVT-011
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/cross-cutting/events/src/event_bus.rs:30-66`
- **description:** The `EventBus` port trait has no retry / DLQ / requeue configuration surface. Per `docs/ports/event-bus.md:104-108` ("Events that fail repeatedly (configurable N retries) are routed to a dead letter queue") and ADR-005 ("the outbox + relay pattern is mandatory for at-least-once delivery"), the bus is contracted to support configurable retries and a DLQ. The trait carries `nack(requeue: bool)` but no `max_retries` / `dead_letter_topic` / `visibility_timeout` knob; consumers have no way to opt into retry-with-backoff behaviour. The in-process bus hard-codes `AckOutcome::Accepted` for all acks/nacks (see CC-EVT-008), so the entire retry + DLQ stack is unimplemented for the default adapter.
- **expected:** Bus-port contract § "Dead Letter Queue" (quoted); `docs/ports/event-bus.md:104-108`.
- **evidence:**
  ```rust
  #[async_trait]
  pub trait EventBus: Send + Sync + fmt::Debug {
      async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;
      async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```
  No `retry`, `dlq`, or `requeue` API exists on the trait.

---

### FINDING 12

- **id:** CC-EVT-012
- **area:** cross-cutting-events
- **severity:** High
- **location:** `crates/cross-cutting/events/src/domain_event.rs:133-138` and `crates/cross-cutting/events/src/lib.rs:59` and `crates/cross-cutting/platform/src/lib.rs:86` and `crates/domains/hr/src/events.rs:55`
- **description:** The `EventFactory` trait is declared as a "recommended pattern" template in `domain_event.rs:133-138` and re-exported from the prelude of `educore-events`, `educore-platform`, and `educore-hr`, but no `impl EventFactory for SomeEvent` exists anywhere in the workspace (`grep "impl EventFactory" crates/` returns zero results). The Phase 2 hand-off § "What's wired" documents `EventFactory` as shipped. The 4 sync events in `sync.rs` define `now()` / `for_session()` constructors that do not satisfy the `fn mint(occurred_at, event_id) -> Self` signature. Consumers reading the prelude believe they have a typed builder; they don't.
- **expected:** Either remove the trait (and its re-exports) until a domain implements it, or implement it for the sync events and the platform events to match the documented intent.
- **evidence:**
  ```rust
  pub trait EventFactory: DomainEvent + Sized {
      /// Mint a new event with a fresh `event_id` and the given
      /// `occurred_at`.
      #[must_use]
      fn mint(occurred_at: Timestamp, event_id: EventId) -> Self;
  }
  ```
  No implementation exists in the workspace.

---

### FINDING 13

- **id:** CC-EVT-013
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/domain_event.rs:54-58` and `docs/specs/events/events.md:10-16`
- **description:** The `DomainEvent` trait bound in code is `Send + Sync + 'static`, but the events-domain spec (`docs/specs/events/events.md:10-16`) and `docs/schemas/event-schema.md` § 12 ("Event Immutability") imply events must be `Serialize + DeserializeOwned` so the engine can persist them in the outbox and replay them from the event log. The default `into_envelope` helper at `domain_event.rs:107-115` only requires `Self: Sized + Serialize`, and the outbox helper `SerializedEnvelope::from_event_envelope` at `crates/infra/storage/src/outbox.rs:139-160` calls `serde_json::to_vec(&envelope.payload)` — meaning the engine implicitly assumes every event's `payload: serde_json::Value` is serializable (fine, it's `Value`) but the *typed event struct itself* has no `Deserialize` bound, so an event stored in the event log as JSON cannot be reconstructed into the typed struct by the engine.
- **expected:** Either add `DeserializeOwned` to the trait bound (matching the spec), or document the asymmetry: the typed event is publish-only; the event-log row is the durable record; consumers re-materialise typed events themselves.
- **evidence:**
  ```rust
  pub trait DomainEvent: Send + Sync + 'static { ... }
  ```
  Spec: `pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync { ... }` (`docs/specs/events/events.md:10-16`).

---

### FINDING 14

- **id:** CC-EVT-014
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:159-170`
- **description:** `EventFilter::Capability` accepts `String` rather than the typed `educore_rbac::Capability` enum. The doc comment at `event_bus.rs:155-165` acknowledges this is "to avoid a circular cross-cutting → cross-cutting dependency", but the trade-off means the bus-port filter is stringly-typed and a typo at the call site (e.g. `EventFilter::Capability("platfrom.user.read".into())`) will silently match no events. The `matches` implementation also uses `envelope.event_type.starts_with(s.as_str())`, which is a substring match: a filter `"fin"` would match `"finance.invoice.generated"`, `"finance.payment.collected"`, `"finance.fees_invoice.configured"`, AND any future event type whose type begins with `"fin"`.
- **expected:** Either typed `Capability` with a `rbac` dep in `Cargo.toml`, or a domain-prefix exact-match instead of `starts_with`.
- **evidence:**
  ```rust
  pub enum EventFilter {
      ...
      /// The capability namespace is owned by `educore-rbac::Capability`;
      /// for Phase 2 the filter is a `String` (stringly-typed) to avoid
      /// a circular `cross-cutting → cross-cutting` dependency.
      Capability(String),
      ...
  }
  ...
  Self::Capability(s) => {
      envelope.payload.get("capability").and_then(|v| v.as_str()) == Some(s.as_str())
          || envelope.event_type.starts_with(s.as_str())
  }
  ```

---

### FINDING 15

- **id:** CC-EVT-015
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/sync.rs:42-58, 80-115, 140-175, 200-235`
- **description:** The four typed sync events have `AGGREGATE_TYPE = "sync_session"` but `EVENT_TYPE = "sync.session.*"`. The naming convention in `docs/schemas/event-schema.md:51` (`<domain>.<aggregate>.<verb>`) requires the middle component to match the aggregate name. With `aggregate_type = "sync_session"`, the natural event_type would be `sync.sync_session.started`. The code emits `sync.session.started` (using `session` as the aggregate component) and stores `sync_session` as the aggregate name, so the `aggregate_topic()` helper at `envelope.rs:111-122` produces `sync.sync_session` (domain prefix `sync` + `aggregate_type` `sync_session`), which does not match the `<domain>.<aggregate>` topic convention used by `Topic::Aggregate`. Every consumer subscribing to `Topic::Aggregate("sync", "session")` will receive zero events.
- **expected:** Either rename `AGGREGATE_TYPE` to `"session"` (matching the dot-separated event_type), or rename `EVENT_TYPE` to `"sync.sync_session.started"` (matching the aggregate_type).
- **evidence:**
  ```rust
  impl DomainEvent for SyncStarted {
      const EVENT_TYPE: &'static str = "sync.session.started";
      const SCHEMA_VERSION: u32 = 1;
      const AGGREGATE_TYPE: &'static str = "sync_session";
      ...
  }
  ```

---

### FINDING 16

- **id:** CC-EVT-016
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/envelope.rs:50` and `crates/infra/storage/src/outbox.rs:54-79, 139-160` and `docs/schemas/event-schema.md:30`
- **description:** The wire-format field name `schema_version` in `EventEnvelope` and `SerializedEnvelope` conflicts with `docs/schemas/event-schema.md:30`, which defines the canonical name as `event_version`. The bus-port doc at `docs/ports/event-bus.md:34-49` uses `schema_version`, so the two spec docs disagree with each other and the code matches the bus-port spec. Phase 3+ consumers written against the event-schema spec will look for `event_version` in the JSON and find nothing.
- **expected:** Pick one name and update both spec docs and the code in lockstep. `event_version` (event-schema) or `schema_version` (bus-port).
- **evidence:**
  ```rust
  // From crates/cross-cutting/events/src/envelope.rs:50:
  pub schema_version: u32,
  // From docs/schemas/event-schema.md:30:
  event_version:     u32,               // schema version of the payload
  ```

---

### FINDING 17

- **id:** CC-EVT-017
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/domain_event.rs:170-204`
- **description:** `RawPayload` (a JSON wrapper carrying `payload`, `correlation_id`, `actor_id`) is declared at `domain_event.rs:170-204` and re-exported nowhere (no `pub use` in `lib.rs:51-67`, not in the prelude). It is documented as "for the audit / outbox writers that need to stamp the `correlation_id` into the JSON body when no typed event is available" but no audit writer or outbox writer calls `RawPayload::new`. The type is dead code.
- **expected:** Either delete the type or wire it into the audit writer / integration test that needs the audit-sink stamp.
- **evidence:**
  ```rust
  pub struct RawPayload {
      pub payload: serde_json::Value,
      pub correlation_id: CorrelationId,
      pub actor_id: UserId,
  }
  ```
  `grep "RawPayload::new\|RawPayload {" crates/` returns only the definition site.

---

### FINDING 18

- **id:** CC-EVT-018
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:108-130, 187-198`
- **description:** `Topic::EventType(&'static str)`, `Topic::Aggregate(&'static str, &'static str)`, `Topic::Domain(&'static str)`, and `EventFilter::EventType(&'static str)` / `EventFilter::AggregateType(&'static str)` all require `&'static str` arguments. A consumer that discovers an event type at runtime (e.g. by reading `engine.events.list()` per `docs/schemas/event-schema.md:139`) cannot construct a `SubscribeOptions` from the dynamic `String` without `Box::leak` or a similar lifetime hack. The `Serialize` / `Deserialize` derives that would let the type carry a runtime string are also absent.
- **expected:** Either `String` (with `Serialize`/`Deserialize`) or `Cow<'static, str>` so dynamic discovery works without `Box::leak`.
- **evidence:**
  ```rust
  pub enum Topic {
      Domain(&'static str),
      Aggregate(&'static str, &'static str),
      EventType(&'static str),
      Tenant(SchoolId),
      All,
  }
  ...
  pub enum EventFilter {
      EventType(&'static str),
      AggregateType(&'static str),
      SchoolId(SchoolId),
      Capability(String),
      Expression(Box<EventFilterExpr>),
  }
  ```

---

### FINDING 19

- **id:** CC-EVT-019
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:99-118`
- **description:** `SubscribeOptions::batch_size` is documented to be clamped to a "sane range (e.g. 1..=1024)" by adapters, but neither the trait nor the in-process adapter enforces any range. A caller passing `batch_size = 0` or `batch_size = u32::MAX` will silently get the un-clamped value at the broadcast-channel layer (the in-process bus `clamp_capacity` only clamps the channel capacity, not `batch_size`). The `visibility_timeout` field is also unused at the trait level — no adapter reads it.
- **expected:** Trait-level validation in `SubscribeOptions::new` or `for_consumer`, plus adapter enforcement per the bus-port doc.
- **evidence:**
  ```rust
  /// Maximum number of envelopes the subscription may buffer
  /// locally. Adapters clamp this to a sane range (e.g. 1..=1024).
  pub batch_size: u32,
  /// Visibility timeout for in-flight envelopes. After this
  /// duration the bus may redeliver the envelope to another
  /// consumer.
  pub visibility_timeout: Duration,
  ```
  No clamp in `for_consumer` (defaults to 32 / 300s) or in `in_process.rs`.

---

### FINDING 20

- **id:** CC-EVT-020
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:55-65, 78-101`
- **description:** `EventBus` has no `unsubscribe` method. The audit checklist asks about a clean unsubscribe; the only path is `Box<dyn EventSubscription>::close(self)`, which consumes the subscription. A consumer holding the subscription by reference (e.g. inside an actor loop or an axum `WebSocket` task) cannot cancel without taking the subscription by value. There is no method like `EventBus::unsubscribe(&self, consumer: &ConsumerId)` that closes the subscription from the bus side, nor any way to enumerate active subscriptions.
- **expected:** Either an explicit `EventBus::unsubscribe(&self, ConsumerId)` method, or a doc note explaining that subscription lifetime is consumer-owned.
- **evidence:**
  ```rust
  #[async_trait]
  pub trait EventBus: Send + Sync + fmt::Debug {
      async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt>;
      async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt>;
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```
  No `unsubscribe`.

---

### FINDING 21

- **id:** CC-EVT-021
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:78-101` and `crates/adapters/event-bus/src/in_process.rs:370-410`
- **description:** `EventSubscription::next` and `EventSubscription::ack` / `nack` have no documentation of cancellation safety. Per the Rust async ecosystem norm (the tokio docs and `cc-EVT-013` open question in the Phase 2 hand-off), a `Future` is either `CancelSafe` (can be dropped at any await point without side effects) or not (dropping loses state). The in-process adapter's `next()` polls a `broadcast::Receiver::recv()` — if dropped mid-await, the receiver stays subscribed and the next `next()` call continues correctly, so the adapter is cancel-safe by accident. `ack`/`nack` are no-ops, so they're trivially cancel-safe. But the trait docs at `event_bus.rs:78-101` say nothing, so a future NATS/Redis adapter can introduce a non-cancel-safe state machine without violating any documented contract.
- **expected:** Doc comments on each trait method stating the cancellation-safety guarantee (or explicitly marking it `#[must_use]`).
- **evidence:**
  ```rust
  /// Returns the next envelope, or `None` if the subscription
  /// is closed. Errors are surfaced as `Some(Err(_))`.
  async fn next(&mut self) -> Option<Result<EventEnvelope>>;

  /// Acknowledges processing of `event_id`. Idempotent.
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  ```
  No cancellation-safety statement on either method.

---

### FINDING 22

- **id:** CC-EVT-022
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:262-298`
- **description:** `BatchReceipt` lacks any per-receipt status field. The doc comment at `:271-275` acknowledges this: "the trait doesn't model per-receipt failure for batches; producers that need that granularity call `publish` in a loop". But the in-process adapter at `crates/adapters/event-bus/src/in_process.rs:202-219` short-circuits `publish_batch` on the first failure inside the loop and returns `Ok(BatchReceipt { receipts: [...up to the failure], correlation_id: None })` — so a partial batch is indistinguishable from a successful batch except by counting receipts. There is no `BatchFailure` variant or per-receipt `Ok`/`Err` enum to carry the failure.
- **expected:** Either add `BatchItemStatus` to `PublishReceipt` (mirroring the doc's own suggestion), or change `BatchReceipt::receipts` to `Vec<Result<PublishReceipt, EventError>>`.
- **evidence:**
  ```rust
  pub struct BatchReceipt {
      /// Per-envelope receipts, in the order the envelopes were
      /// submitted.
      pub receipts: Vec<PublishReceipt>,
      /// The correlation id of the batch, if any. ...
      pub correlation_id: Option<CorrelationId>,
  }
  ```

---

### FINDING 23

- **id:** CC-EVT-023
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:255-260` and `docs/ports/event-bus.md:182-187`
- **description:** `ConsumerId` is the only stable identifier for a subscription; it is used for offset tracking and observability per the doc. However, the bus-port worked example at `docs/ports/event-bus.md:194` uses `ConsumerId::new("welcome-emailer")` as the consumer id for a single in-process consumer, and the doc-comment on `ConsumerId::new` (`:255-258`) says the string "is expected to be stable across process restarts". The trait provides no method to look up a subscription by `ConsumerId`, no method to enumerate active `ConsumerId`s, no method to read a consumer's offset / lag — and `EventBus::subscribe` does not enforce that the `ConsumerId` is unique across concurrent subscriptions. Two concurrent `subscribe` calls with the same `ConsumerId` will silently create two parallel subscriptions.
- **expected:** Bus-port contract section "Subscription Model" (`docs/ports/event-bus.md:184-195`); offset tracking is the consumer's responsibility only because the port is silent on it.
- **evidence:**
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct ConsumerId(pub String);
  ```

---

### FINDING 24

- **id:** CC-EVT-024
- **area:** cross-cutting-events
- **severity:** Medium
- **location:** `crates/cross-cutting/events/src/event_bus.rs:34-46` and `crates/cross-cutting/events/src/event_bus.rs:138-157`
- **description:** `EventFilter::Expression(Box<EventFilterExpr>)` is the only way to compose filters; there is no flat `Filter::All(Vec<EventFilter>)` or `Filter::Any(Vec<EventFilter>)` shape. The `EventFilterExpr` enum has 4 variants (`And`, `Or`, `Not`, `Leaf`), all binary except `Leaf`. Constructing an N-way OR requires a right-leaning tree: `Or(Leaf(A), Or(Leaf(B), Leaf(C)))`, which is awkward and forces `Box` allocations for every internal node. The audit checklist asks about subscription filter expressiveness; the current shape is minimal.
- **expected:** `EventFilter::All(Vec<EventFilter>)` / `Any(Vec<EventFilter>)` variants for N-ary composition, OR an explicit note that the tree shape is intentional.
- **evidence:**
  ```rust
  pub enum EventFilterExpr {
      And(Box<Self>, Box<Self>),
      Or(Box<Self>, Box<Self>),
      Not(Box<Self>),
      Leaf(Box<EventFilter>),
  }
  ```

---

### FINDING 25

- **id:** CC-EVT-025
- **area:** cross-cutting-events
- **severity:** Low
- **location:** `crates/cross-cutting/events/src/domain_event.rs:144-167`
- **description:** `EmittedEvent<T>` is a small wrapper that pairs `T: DomainEvent` with the `TenantContext` it was emitted under. It is documented as the recommended construction pattern (`EmittedEvent::new(event, ctx).into_envelope()`), is re-exported in the prelude, and is unit-tested at `domain_event.rs:267-285`. No domain crate uses it: `crates/domains/*/src/services.rs` constructs `into_envelope` directly from a typed event + `TenantContext` (e.g. `crates/domains/finance/src/services.rs`). The wrapper is dead in production.
- **expected:** Either remove `EmittedEvent` from the prelude, or document the Phase 3+ convention that domain services return `EmittedEvent<T>` instead of `(T, EventEnvelope)`.
- **evidence:**
  ```rust
  pub struct EmittedEvent<T: DomainEvent + Serialize> {
      pub event: T,
      pub ctx: TenantContext,
  }
  ```
  `grep "EmittedEvent::new" crates/` returns only the test in `domain_event.rs`.

---

### FINDING 26

- **id:** CC-EVT-026
- **area:** cross-cutting-events
- **severity:** Low
- **location:** `crates/cross-cutting/events/src/event_bus.rs:75-87`
- **description:** `EventSubscription::next()` returns `Option<Result<EventEnvelope>>` — a `None` means "subscription closed". A consumer cannot distinguish a slow broker (no message yet) from a permanently closed subscription without timing out. The bus-port spec at `docs/ports/event-bus.md:108` says the same shape, so the implementation matches the spec, but the shape conflates "idle" with "closed" which is a known footgun. The audit checklist calls this out as an edge case.
- **expected:** Either an explicit `RecvOutcome { Idle, Envelope, Closed }` enum, or a separate `EventSubscription::is_closed` method.
- **evidence:**
  ```rust
  /// Returns the next envelope, or `None` if the subscription
  /// is closed. Errors are surfaced as `Some(Err(_))`.
  async fn next(&mut self) -> Option<Result<EventEnvelope>>;
  ```

---

### FINDING 27

- **id:** CC-EVT-027
- **area:** cross-cutting-events
- **severity:** Low
- **location:** `crates/cross-cutting/events/src/envelope.rs:43-46` and `crates/cross-cutting/events/src/outbox.rs:6-17`
- **description:** The `EventEnvelope` carries `&'static str` for `event_type` and `aggregate_type`, which means it does NOT implement `DeserializeOwned`. The envelope.rs:138-142 test comment acknowledges this ("`EventEnvelope` has `&'static str` fields (per the bus-port contract), so it does NOT implement `DeserializeOwned`"). The round-trip bridge to `SerializedEnvelope` (which uses `String` for the same fields) lives in `crates/infra/storage/src/outbox.rs:139-160` and `crates/infra/storage/src/event_log.rs:175-187` — i.e. the envelope cannot round-trip through the bus-port; only the storage-port mirror can. Any consumer code (audit writer, integration test, central-fan-out) that wants to re-materialise the typed envelope from a stored row must depend on the storage port rather than the events port.
- **expected:** Either move the bridge to `educore-events` (reverse the tier dep that PHASE-2-HANDOFF § "Open questions" #6 flags), or document the storage-port dependency at the bus-port consumer's API surface.
- **evidence:**
  ```rust
  // crates/cross-cutting/events/src/envelope.rs:43-46:
  // **Stability:** the field set, names, and order are part of the
  // engine's public API. Renames or removals are breaking changes
  // and require an ADR.
  pub struct EventEnvelope {
      ...
      pub event_type: &'static str,
      ...
      pub aggregate_type: &'static str,
      ...
  }
  ```
  And the bridge lives at `crates/infra/storage/src/outbox.rs:139-160`, in `educore-storage` (infra), not in `educore-events` (cross-cutting).

---

### FINDING 28

- **id:** CC-EVT-028
- **area:** cross-cutting-events
- **severity:** Low
- **location:** `crates/cross-cutting/events/src/event_bus.rs:124-135`
- **description:** `StartPosition::FromTimestamp` and `StartPosition::FromEventId` rely on UUIDv7 time ordering for cursor semantics. The doc on `FromEventId` says "UUIDv7 is time-ordered: lexicographic comparison gives chronological ordering" (`crates/adapters/event-bus/src/in_process.rs:329-333`). This is correct for UUIDv7 minted by the same generator, but the bus-port spec at `docs/ports/event-bus.md:55-60` does not require that all `event_id`s on the bus are UUIDv7 — any consumer that hand-mints a UUIDv4 (or a UUIDv7 with a non-monotonic clock skew, or a foreign system's UUID) will be ordered incorrectly. The cursor code at `in_process.rs:325-345` performs a raw lex compare without verifying that the cursor itself is UUIDv7.
- **expected:** Bus-port spec section "Schema Versioning" + "Replay" (`docs/ports/event-bus.md:71-83`) and event-schema § 1.1 require UUIDv7 but don't assert it on the wire.
- **evidence:**
  ```rust
  StartPosition::FromEventId(id) => {
      env.event_id.as_uuid() > id.as_uuid()
  }
  ```
  No version-bit check; a UUIDv4 cursor would sort lexicographically by its random bits, not by time.

---
