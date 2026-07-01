## Wave 3 Adapter Audit Report — `educore-event-bus`

**Scope:** `crates/adapters/event-bus/` (`lib.rs`, `errors.rs`,
`in_process.rs`, `nats.rs`, `redis.rs`, `tests/in_process_e2e.rs`,
`Cargo.toml`, `README.md`); the bus-port contract
(`docs/ports/event-bus.md`); the bus-port trait
(`crates/cross-cutting/events/src/event_bus.rs`); the bus-port
error type (`crates/cross-cutting/events/src/errors.rs`); the
Phase 2 hand-off (`docs/handoff/PHASE-2-HANDOFF.md`);
`docs/build-plan.md` Phase 2 task list; the testkit
re-export (`crates/tools/testkit/src/event_bus.rs`);
`docs/coverage.toml` rows for `event_bus_port` and
`event_bus_inprocess`. Findings only — no fixes proposed.

**Total findings:** 22

---

### FINDING 1

- **id:** ADAPT-EB-001
- **area:** adapters-event-bus
- **severity:** Critical
- **location:** `crates/adapters/event-bus/src/nats.rs:90-114` and
  `crates/adapters/event-bus/src/redis.rs:128-152`
- **description:** `NatsEventBus` and `RedisEventBus` are feature-
  gated stubs whose `EventBus` impl only covers `publish`,
  `publish_batch`, and `subscribe`. Both types are exposed at the
  crate root (`crates/adapters/event-bus/src/lib.rs:60-66`) and
  advertise "Phase 2 scaffold for a NATS JetStream-backed event
  bus" in their rustdoc, yet every method returns
  `EventError::not_supported("NatsEventBus::publish")` (and
  analogous Redis strings). The Phase 2 hand-off
  (`docs/handoff/PHASE-2-HANDOFF.md` § "educore-event-bus")
  acknowledges the stubs, but the README, the lib rustdoc, and
  the feature flags (`default = ["in-process"]`,
  `nats = ["dep:async-nats"]`, `redis = ["dep:redis"]`) present
  them as usable adapters. A consumer that wires
  `Arc<dyn EventBus> = Arc::new(NatsEventBus::new().connect(...))`
  gets a bus that accepts `connect` and silently fails every
  publish / subscribe — production deploys that picked the
  distributed adapter on the assumption that it works will lose
  every event without a runtime error.
- **expected:** `docs/ports/event-bus.md:171-176` says
  "Distributed adapters are consumer-supplied. The bus trait is
  intentionally minimal so any messaging system can be
  implemented." The stubs are not "consumer-supplied"; they
  ship in the engine's adapter crate and appear to be a
  production-ready adapter. The Phase 2 hand-off should be
  linked from the README and the feature flag should be
  documented as "Phase 2 stub — wire-protocol work in a later
  phase".
- **evidence:**
  ```rust
  async fn publish(
      &self,
      _envelope: EventEnvelope,
  ) -> educore_core::error::Result<PublishReceipt> {
      debug!("NatsEventBus::publish (Phase 2 stub, returning NotSupported)");
      Err(EventError::not_supported("NatsEventBus::publish").into())
  }
  ```
  (analogous body in `redis.rs:131-137`).

---

### FINDING 2

- **id:** ADAPT-EB-002
- **area:** adapters-event-bus
- **severity:** Critical
- **location:** `crates/adapters/event-bus/src/in_process.rs:457-475`
- **description:** `InProcessSubscription::ack` and `nack` are
  no-ops that always return `Ok(AckOutcome::Accepted)`, with the
  rationale "in-process delivery is direct; ack is a no-op".
  The bus-port contract at `docs/ports/event-bus.md:71-77`
  promises "At-Least-Once Delivery. The bus provides at-least-
  once delivery" and `docs/ports/event-bus.md:84-86` promises
  "Dead Letter Queue. Events that fail repeatedly (configurable
  N retries) are routed to a dead letter queue." A consumer
  that calls `nack(event_id, true)` to requeue a failed event
  receives `AckOutcome::Accepted`; the envelope stays on the
  bus broadcast channel, is delivered again to the same
  subscription, and any other subscriber that filtered
  on `EventFilter` will never see the rejected instance. There
  is no retry counter, no DLQ, no per-event delivery state, and
  no way for a consumer to detect that its `nack` was
  discarded.
- **expected:** `docs/ports/event-bus.md:71-77` (at-least-once
  delivery) and `:84-86` (DLQ).
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

### FINDING 3

- **id:** ADAPT-EB-003
- **area:** adapters-event-bus
- **severity:** Critical
- **location:** `crates/adapters/event-bus/src/in_process.rs:280-299`
- **description:** `InProcessEventBus::publish` first pushes the
  envelope to a bounded `VecDeque` replay log (capped at
  `replay_log_capacity`, default 4096), then sends it to the
  global broadcast channel. When the broadcast channel has
  zero receivers, the `send` error is **silently swallowed**
  (`Err(_)` branch returns `Ok(PublishReceipt::new(...))` with
  no indication the envelope was not delivered to anyone). The
  envelope remains in the replay log, so a future
  `StartPosition::Earliest` subscriber sees it. But when the
  log is also at capacity and an envelope is pushed in, the
  oldest envelope is `pop_front()`'d before the new one is
  appended — a burst of more than 4096 publishes between two
  subscribers joining will silently evict the head of the log
  with no warning. There is no metric, no event, no log line
  on eviction. Per `docs/ports/event-bus.md:71-77` the bus
  promises at-least-once delivery; silent eviction of an
  envelope is at-most-once-from-the-log and contradicts the
  contract.
- **expected:** `docs/ports/event-bus.md:71-77` (at-least-once
  delivery); `docs/ports/event-bus.md:106-110` (replay retention).
- **evidence:**
  ```rust
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
  ```
  And at `crates/adapters/event-bus/src/in_process.rs:262-273`:
  ```rust
  if self.inner.config.replay_log_capacity > 0 {
      match self.inner.log.lock() {
          Ok(mut log) => {
              if log.len() == self.inner.config.replay_log_capacity {
                  log.pop_front();
              }
              log.push_back(env.clone());
          }
          ...
      }
  }
  ```

---

### FINDING 4

- **id:** ADAPT-EB-004
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/errors.rs:45-49`
- **description:** The `subscribe_failed` helper builds an
  `EventError::PublishFailed` with the message
  `"subscribe failed: ..."`. A subscribe-side error (e.g. the
  replay log mutex is poisoned — the only call site in
  `in_process.rs:316`) is reported to the consumer as a
  publish failure. Consumers that match
  `DomainError::Infrastructure("publish failed: ...")` to
  distinguish retryable transport failures from subscribe-side
  configuration errors will misroute subscribe failures to
  their publish retry path. The bus-port contract has no
  `SubscribeFailed` variant, so the adapter is forced to
  misclassify; this is a port gap surfaced by the adapter
  impl.
- **expected:** `docs/ports/event-bus.md:179-187` enumerates the
  `EventBusError` enum (no `SubscribeFailed` variant). The
  `EventError` enum in
  `crates/cross-cutting/events/src/errors.rs:23-44` adds
  `NotSupported` and `Infrastructure` variants on top of the
  port doc, but no `SubscribeFailed` variant exists.
- **evidence:**
  ```rust
  #[inline]
  pub fn subscribe_failed(msg: impl Into<String>) -> EventError {
      EventError::PublishFailed(format!("subscribe failed: {}", msg.into()))
  }
  ```

---

### FINDING 5

- **id:** ADAPT-EB-005
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:262-275`
- **description:** The replay log mutex is held inside the
  `publish` critical section and contains both the `pop_front`
  (when at capacity) and the `push_back` of the cloned
  envelope. A single poisoned mutex causes every subsequent
  publish to return `EventError::PublishFailed("replay log
  mutex poisoned")`; the envelope is not published, not in
  the log, and the broadcast channel is never reached. There
  is no recovery path; the bus is wedged until the process
  restarts. The `broadcast::Sender::send` (which is the actual
  delivery path) is independent of the replay log; the log is
  advisory. Locking the publish path on a poisoned advisory
  structure is unsafe.
- **expected:** `docs/ports/event-bus.md:71-77` (at-least-once
  delivery; consumers must not lose events on transient
  failures).
- **evidence:**
  ```rust
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
  ```

---

### FINDING 6

- **id:** ADAPT-EB-006
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:128-134`
- **description:** `InProcessConfig::with_capacity` clamps the
  replay log capacity to `0` and the channel capacity to
  `clamp_capacity(c)` (which clamps `0` to `1`). Consumers
  that call `InProcessEventBus::with_capacity(0)` to mean "no
  channel, no log" instead get a `channel_capacity = 1` bus
  that can buffer exactly one envelope per subscriber. The
  config rustdoc says "Returns a config with both capacities
  clamped to `0` and the given capacity (`1..=u32::MAX`)" — the
  prose is contradictory (it says "clamped to `0` and the given
  capacity"). A consumer reading the docstring expects a
  `0..=u32::MAX` range; the actual range is `1..=u32::MAX` for
  the channel. The replay log is hard-coded to `0`, which
  silently disables replay even if the consumer passed a
  non-zero capacity — this is a footgun that turns
  `StartPosition::Earliest` into `Latest` with no warning.
- **expected:** `docs/ports/event-bus.md:106-110` (replay
  contract — replay is mandatory for projection rebuilds).
- **evidence:**
  ```rust
  pub fn with_capacity(capacity: usize) -> Self {
      Self {
          channel_capacity: clamp_capacity(capacity),
          replay_log_capacity: 0,
      }
  }
  ```
  And `clamp_capacity`:
  ```rust
  fn clamp_capacity(c: usize) -> usize {
      // The broadcast channel rejects 0; the replay log accepts 0.
      if c == 0 {
          1
      } else {
          c
      }
  }
  ```

---

### FINDING 7

- **id:** ADAPT-EB-007
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:431-445`
- **description:** `InProcessSubscription::next` swallows
  `broadcast::error::RecvError::Lagged(skipped)` with a
  `continue` after a `debug!` log. The `skipped` count is
  recorded but the subscription has no mechanism to
  re-deliver or re-replay the missed envelopes from the bus.
  `nack(requeue = true)` is a no-op (FINDING 2). A subscriber
  that falls behind by more than `channel_capacity` envelopes
  permanently loses the gap. There is no DLQ, no replay-from-
  event-id hook, and the consumer cannot request a re-subscribe
  with `StartPosition::FromEventId(last_seen_id)` because the
  bus does not surface `last_seen_id` back to the consumer.
  Per `docs/ports/event-bus.md:71-77` this is at-most-once
  delivery under back-pressure.
- **expected:** `docs/ports/event-bus.md:71-77` (at-least-once
  delivery); `docs/ports/event-bus.md:84-86` (DLQ).
- **evidence:**
  ```rust
  Err(broadcast::error::RecvError::Lagged(skipped)) => {
      debug!(
          consumer = %self.consumer,
          skipped, "subscription lagged; skipping past missed envelopes"
      );
      continue;
  }
  ```

---

### FINDING 8

- **id:** ADAPT-EB-008
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:329-339`
  and `:457-492`
- **description:** `InProcessSubscription::next` filters by
  `Topic` and `EventFilter` in the subscription loop, but the
  filters are re-applied on every envelope after the broadcast
  `recv`. A subscription on `Topic::All` with no filter pays
  the full broadcast-receive cost per envelope; a subscription
  on `Topic::EventType("academic.student.admitted")` still
  receives every envelope on the global channel and drops the
  non-matching ones in the loop. For a bus with 1024
  subscribers on disjoint topics this is O(N) work per publish
  — the same work the port-doc attributes to "per-topic routing
  ... applied in the subscription's next loop" (`in_process.rs:
  9-18`). There is no per-topic fan-out; the design scales
  linearly with subscriber count, not topic cardinality.
- **expected:** `docs/ports/event-bus.md:117-121` (topic naming
  conventions) and `docs/ports/event-bus.md:71-77` (bus
  performance under fan-out).
- **evidence:**
  ```rust
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
      ...
  }
  ```

---

### FINDING 9

- **id:** ADAPT-EB-009
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/in_process.rs:486-492`
- **description:** `InProcessSubscription::close` consumes
  `self: Box<Self>`, dereferences the box to set
  `me.closed = true`, then lets `me` drop at end of scope. The
  `broadcast::Receiver` is dropped at end of scope, releasing
  the broadcast slot. But because the subscription was
  unboxed into a stack value, the `drop` order is: (1) set
  `closed = true` on the stack copy, (2) call
  `me.bus.strong_count()` for the diagnostic, (3) drop the
  `Weak<InProcessInner>`, (4) drop `closed: bool` (no-op),
  (5) drop the rest. The boxed-deref pattern is unusual; the
  e2e test `subscription_close_releases_resources` passes
  because `broadcast::Sender::receiver_count` is checked
  synchronously after the `close().await` returns — but the
  release happens during the drop, which the test does not
  observe deterministically. If `close` ever needs to await
  an ack of close (e.g., a distributed adapter flushing
  pending offsets), the unbox makes that impossible because
  `me` is a stack value, not `Pin`.
- **expected:** Idiomatic `async_trait` `close(self: Box<Self>)`
  should keep the box and drop on `Drop` or use `Pin<Box<Self>>`
  if async-drop is required.
- **evidence:**
  ```rust
  async fn close(self: Box<Self>) -> educore_core::error::Result<()> {
      // Drop the receiver (releases its slot in the broadcast
      // channel) and the replay buffer (clears the heap).
      let mut me = *self;
      me.closed = true;
      ...
  }
  ```

---

### FINDING 10

- **id:** ADAPT-EB-010
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/nats.rs:55-83`
  and `crates/adapters/event-bus/src/redis.rs:69-104`
- **description:** `NatsEventBus::connect` and
  `RedisEventBus::connect` succeed silently and update an
  internal `client` / `config` slot, but the trait methods
  still return `NotSupported` because the wire-protocol work
  is deferred. A consumer that calls `connect`, observes
  `is_connected() == true`, and then calls `publish` will
  receive `EventError::NotSupported("NatsEventBus::publish")`
  — the `is_connected` flag is misleading; it reports
  "client is wired" not "the bus can deliver". The
  misleading boolean violates the principle of least surprise
  and the test in `crates/adapters/event-bus/tests/in_process_e2e.rs`
  `nats_bus_returns_not_supported_without_connection` asserts
  this exact path (without `connect`).
- **expected:** `is_connected` should be renamed to
  `has_wired_client` or removed; the rustdoc should clarify
  the stub state.
- **evidence:**
  ```rust
  pub async fn is_connected(&self) -> bool {
      self.client.lock().await.is_some()
  }
  ```
  (identical body in `redis.rs:103-105`).

---

### FINDING 11

- **id:** ADAPT-EB-011
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/lib.rs:10-19` and
  `crates/adapters/event-bus/Cargo.toml:14-25`
- **description:** The `in-process` Cargo feature is a marker
  feature (no deps, no cfg gates) and is the default. The
  crate's `default = ["in-process"]` means `cargo build` always
  pulls in `InProcessEventBus`. Consumers that want a
  distributed-only build (`default-features = false`) get a
  crate with `in-process` feature present but the module is
  still compiled (the `in_process` module is `pub mod
  in_process;` with no `#[cfg(feature = "in-process")]`
  gate at `crates/adapters/event-bus/src/lib.rs:32-34`). The
  marker feature is dead — `InProcessEventBus` is always
  available regardless of feature flags. The README claims
  "consumers can opt out in tests if they want to verify a
  `default-features = false` build" (`Cargo.toml:17-21`); this
  claim is false.
- **expected:** Either gate `in_process` on the feature or
  remove the marker feature.
- **evidence:**
  ```rust
  // crates/adapters/event-bus/src/lib.rs:32-34
  /// The in-process MPMC event bus.
  pub mod in_process;
  ```
  And `crates/adapters/event-bus/Cargo.toml:14-25`:
  ```toml
  default = ["in-process"]
  # Marker feature for the in-process bus. Always enabled by default;
  # listed as a feature so consumers can opt out in tests if they
  # want to verify a `default-features = false` build of the
  # adapter crate (e.g., for the distributed-only path).
  in-process = []
  ```

---

### FINDING 12

- **id:** ADAPT-EB-012
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/in_process.rs:412-425`
- **description:** `topic_matches` for `Topic::Aggregate(d, a)`
  matches the wire string `"<d>.<a>"` exactly, but the
  `EventEnvelope::aggregate_topic` helper at
  `crates/cross-cutting/events/src/envelope.rs:79-88` returns
  `"<domain>.<aggregate>"` only when `event_type` has a `.`
  separator; for events whose `event_type` has no `.` (e.g.,
  `"school"`), `aggregate_topic` returns just `aggregate_type`
  (e.g., `"school"`). A `Topic::Aggregate("platform",
  "school")` subscription against an envelope with
  `event_type = "school"` (no domain prefix) will miss the
  envelope even though the envelope is for the same aggregate.
  The unit test `topic_matches_handles_all_variants` exercises
  `SyncStarted` whose `event_type = "sync.session.started"`
  (has a `.`), so the gap is not caught.
- **expected:** `Topic::Aggregate(d, a)` should match
  `aggregate_topic()` defensively (both the full string and
  the bare aggregate form).
- **evidence:**
  ```rust
  fn topic_matches(topic: &Topic, env: &EventEnvelope) -> bool {
      match topic {
          Topic::Aggregate(d, a) => env.aggregate_topic() == format!("{d}.{a}"),
          ...
      }
  }
  ```
  And the fallback in
  `crates/cross-cutting/events/src/envelope.rs:79-88`:
  ```rust
  pub fn aggregate_topic(&self) -> String {
      match self.event_type.split_once('.') {
          Some((domain, _)) if !domain.is_empty() => {
              format!("{domain}.{}", self.aggregate_type)
          }
          _ => self.aggregate_type.to_owned(),
      }
  }
  ```

---

### FINDING 13

- **id:** ADAPT-EB-013
- **area:** adapters-event-bus
- **severity:** Critical
- **location:** `crates/adapters/event-bus/src/in_process.rs:457-475`
  and `docs/ports/event-bus.md:71-77`
- **description:** The bus-port contract
  (`docs/ports/event-bus.md:71-77`) promises
  "at-least-once delivery" and "Consumers MUST be idempotent.
  The EventId is the idempotency key." The in-process adapter
  never tracks which `event_id`s it has delivered to which
  subscription, and the `start_position_matches` helper at
  `in_process.rs:413-417` compares UUIDs lexicographically
  assuming UUIDv7 ordering. A producer that republishes the
  same envelope after a retry (the typical at-least-once
  scenario) will deliver the duplicate to every active
  subscription with no dedupe; a consumer that resumes from
  `StartPosition::FromEventId(cursor)` after a process crash
  relies entirely on the consumer's own processed-events table
  to dedupe — the bus does not surface the
  `last_delivered_event_id` per `(subscription, consumer_id)`
  pair, so a fresh subscription created mid-replay can be
  handed the same envelope twice if the cursor's UUID is not
  monotonic relative to the new subscription's start.
- **expected:** `docs/ports/event-bus.md:71-86` (at-least-once
  delivery + idempotency + DLQ + replay contract).
- **evidence:** `grep -n "last_delivered\|seen_event_ids\|
  dedupe\|dedup\|idempot" crates/adapters/event-bus/src/in_process.rs`
  returns zero rows; the `InProcessInner` struct
  (`in_process.rs:117-127`) has no `last_delivered` map.

---

### FINDING 14

- **id:** ADAPT-EB-014
- **area:** adapters-event-bus
- **severity:** Critical
- **location:** `crates/adapters/event-bus/src/in_process.rs:128-187`
  and `docs/ports/event-bus.md:60-67`
- **description:** Per `docs/ports/event-bus.md:60-67` "The
  engine writes events to an outbox table within the same
  database transaction as the domain state change. The outbox
  relay (a separate process) reads pending events from the
  outbox and publishes them to the bus." The in-process
  adapter does not read from any outbox table; it accepts
  `publish` calls directly from in-process producers and
  forwards them to the broadcast channel. There is no
  outbox-relay loop, no background task, no drain call, and
  no `subscribe_outbox` method. The cross-cutting integration
  test at `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  (per `docs/handoff/PHASE-2-HANDOFF.md` § "Cross-cutting
  integration test") exercises the
  `outbox → event_log → bus` path on SQLite, but the relay
  lives in the test, not in the adapter crate. A consumer
  that wires `InProcessEventBus::new()` and skips the
  cross-cutting test harness gets a bus that has no source of
  truth — events emitted by a command but not directly
  `publish`'d to the bus (e.g., committed via storage
  outbox) will never appear on the bus.
- **expected:** `docs/ports/event-bus.md:60-67` (outbox pattern).
- **evidence:** `grep -n "outbox\|Outbox\|drain\|relay"
  crates/adapters/event-bus/src/in_process.rs` returns zero
  rows; the `publish` signature
  (`in_process.rs:172-200`) takes a pre-built `EventEnvelope`,
  not an outbox-row id.

---

### FINDING 15

- **id:** ADAPT-EB-015
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:222-245`
- **description:** `InProcessSubscription` has no
  `batch_size` plumbing: the field is on `SubscribeOptions`
  (`crates/cross-cutting/events/src/event_bus.rs:159-178`) and
  defaults to `32`, but `next()` returns one envelope per
  call. A consumer that sets `batch_size = 256` to amortise
  the cost of `next()` over a larger window still pays one
  `broadcast::Receiver::recv` per envelope. The
  `visibility_timeout` field is similarly ignored. The port
  doc at `docs/ports/event-bus.md:50` documents both fields
  as contract; the in-process adapter accepts them but does
  not honour them. `for_consumer` defaults to `32 / 300s`
  (`event_bus.rs:181-186`); there is no clamp.
- **expected:** `docs/ports/event-bus.md:46-50` (SubscribeOptions
  shape including `batch_size` and `visibility_timeout`).
- **evidence:**
  ```rust
  let mut sub = bus
      .subscribe(make_opts(
          "test-consumer",
          Topic::All,
          StartPosition::Latest,
      ))
      .await
      .expect("subscribe");
  ```
  with `make_opts` building a 32-entry `batch_size` and 300s
  `visibility_timeout` that are never read by
  `InProcessEventBus` (`tests/in_process_e2e.rs:60-72`).

---

### FINDING 16

- **id:** ADAPT-EB-016
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:431-445`
- **description:** When a `broadcast::error::RecvError::Lagged(skipped)`
  is observed, the subscription silently continues. The
  `skipped` count is logged at `debug!` level and discarded.
  There is no public API to retrieve the count, no metric
  emitted, and no event published. A consumer monitoring lag
  via Prometheus / OpenTelemetry cannot observe the lag; a
  consumer that wants to re-replay the missed gap has no hook
  to request it. Per `docs/ports/event-bus.md:84-86` the bus
  promises a DLQ for events that fail repeatedly; a `Lagged`
  error is a "consumer cannot keep up" failure mode that
  should route to the DLQ for inspection, but no DLQ exists
  on the in-process bus.
- **expected:** `docs/ports/event-bus.md:84-86` (DLQ contract).
- **evidence:**
  ```rust
  Err(broadcast::error::RecvError::Lagged(skipped)) => {
      debug!(
          consumer = %self.consumer,
          skipped, "subscription lagged; skipping past missed envelopes"
      );
      continue;
  }
  ```

---

### FINDING 17

- **id:** ADAPT-EB-017
- **area:** adapters-event-bus
- **severity:** High
- **location:** `crates/adapters/event-bus/src/in_process.rs:172-200`
- **description:** `publish_batch` at
  `crates/adapters/event-bus/src/in_process.rs:202-219` is
  implemented as a sequential loop that calls `publish` per
  envelope and short-circuits on the first failure. The
  port-doc comment at `docs/ports/event-bus.md:179-187` (and
  the trait rustdoc at `event_bus.rs:67-72`) says "Adapters
  that don't support atomic batching should fall back to per-
  envelope `publish`; consumers cannot assume either
  semantics unless they pin the adapter." The in-process
  adapter takes the fallback path but **silently truncates
  the `BatchReceipt`**: a 10-envelope batch that fails on
  envelope #3 returns a `BatchReceipt` with 2 receipts and
  no indication that the remaining 7 were not attempted. The
  cross-cutting integration test at
  `crates/tools/storage-parity/tests/cross_cutting_integration.rs`
  relies on `BatchReceipt::is_fully_accepted()` (which is
  itself broken per `docs/audit_reports/findings/wave2-events.md`
  Finding 16) to gate downstream work; the in-process
  adapter's truncation compounds the bug.
- **expected:** `docs/ports/event-bus.md:33` (BatchReceipt
  shape) and `docs/ports/event-bus.md:67` ("Adapters that
  don't support atomic batching...").
- **evidence:**
  ```rust
  async fn publish_batch(
      &self,
      envelopes: Vec<EventEnvelope>,
  ) -> educore_core::error::Result<BatchReceipt> {
      let mut receipts = Vec::with_capacity(envelopes.len());
      for env in envelopes {
          let receipt = self.publish(env).await?;
          receipts.push(receipt);
      }
      Ok(BatchReceipt {
          receipts,
          correlation_id: None,
      })
  }
  ```

---

### FINDING 18

- **id:** ADAPT-EB-018
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/redis.rs:39-67`
- **description:** `RedisBusConfig` stores a
  `redis::aio::ConnectionManager` behind an `Arc<TokioMutex>`.
  `ConnectionManager` is itself internally `Arc`-based and
  clone-cheap; wrapping it in `Arc<TokioMutex<Option<...>>>`
  allocates an extra `Arc` and forces every config access
  through the Tokio mutex. The intent appears to be to allow
  hot-swap of the connection, but no method on `RedisEventBus`
  performs a swap (the field is write-once via `connect`).
  The double-Arc is dead weight on every `is_connected` /
  connect path. The `NatsEventBus` at `nats.rs:42-50` does
  the same dance with `async_nats::Client` inside an
  `Arc<TokioMutex<Option<...>>>` for the same reason.
- **expected:** Idiomatic Rust would use
  `Arc<RwLock<Option<ConnectionManager>>>` (or
  `tokio::sync::RwLock`) if hot-swap is intended, or
  `OnceCell<ConnectionManager>` if it is not.
- **evidence:**
  ```rust
  pub struct RedisEventBus {
      config: Arc<TokioMutex<Option<RedisBusConfig>>>,
  }
  ```
  And `RedisBusConfig`:
  ```rust
  pub struct RedisBusConfig {
      pub url: String,
      pub manager: redis::aio::ConnectionManager,
  }
  ```

---

### FINDING 19

- **id:** ADAPT-EB-019
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/in_process.rs:457-475`
- **description:** The `EventSubscription::ack` and `nack`
  trait methods return `Result<AckOutcome>` in the port trait
  at `crates/cross-cutting/events/src/event_bus.rs:78-101`
  (and implemented in `in_process.rs:457-475`), but the bus
  port contract at `docs/ports/event-bus.md:78-82` declares:
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<()>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>;
  ```
  The return type in the docstring is `Result<()>`. A consumer
  that writes against the docstring signature will not
  compile against the actual trait (extra `AckOutcome` in the
  return). The `AckOutcome` enum at
  `crates/cross-cutting/events/src/event_bus.rs:51-61` adds
  three variants (`Accepted`, `Unknown`, `Failed`) that are
  not represented in the port-doc enum `EventBusError` at
  `docs/ports/event-bus.md:179-187`. This is the same
  deviation flagged in `docs/audit_reports/findings/wave2-events.md`
  Finding 3 (`CC-EVT-003`), but the adapter inherits the
  divergence and propagates it to consumers of
  `InProcessEventBus`.
- **expected:** `docs/ports/event-bus.md:78-82` (return type
  `Result<()>`); `docs/ports/event-bus.md:179-187` (error
  enum).
- **evidence:**
  ```rust
  async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>;
  async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome>;
  ```
  In `crates/cross-cutting/events/src/event_bus.rs:88-100` and
  the implementation in
  `crates/adapters/event-bus/src/in_process.rs:457-475`.

---

### FINDING 20

- **id:** ADAPT-EB-020
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/in_process.rs:117-127`
- **description:** `InProcessInner` holds a `std::sync::Mutex`
  guarding the replay log. `publish` is `async fn` and locks
  this mutex while the broadcast `send` (which is also async-
  aware but does not block the runtime in the same way) is
  attempted. The lock is held across the clone of the
  envelope (`log.push_back(env.clone())`) and the
  `pop_front` on capacity. A burst of publishes from
  multiple producers serialises on this mutex; under
  contention the lock is held across an allocation (the
  `clone`). Per `docs/build-plan.md:497` the bus is intended
  to scale to "10k students × 5 daily commands × 200 schools"
  volumes; a serialising mutex on the hot path is a
  bottleneck. There is no sharding; a `parking_lot::Mutex`
  or a per-producer sharded log would scale better.
- **expected:** `docs/ports/event-bus.md:71-77` (at-least-once
  delivery at scale); `docs/build-plan.md:497` (audit log
  volume — same scale argument).
- **evidence:**
  ```rust
  if self.inner.config.replay_log_capacity > 0 {
      match self.inner.log.lock() {
          Ok(mut log) => {
              if log.len() == self.inner.config.replay_log_capacity {
                  log.pop_front();
              }
              log.push_back(env.clone());
          }
          ...
      }
  }
  ```

---

### FINDING 21

- **id:** ADAPT-EB-021
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/tools/testkit/src/event_bus.rs:1-37` and
  `docs/coverage.toml:2183-2195`
- **description:** The testkit exposes
  `educore_testkit::event_bus::InMemoryEventBus` as a `type`
  alias for `educore_event_bus::InProcessEventBus`. The
  rustdoc on the alias claims it is a "Testkit-local alias"
  and that "The alias exists so consumers can write `use
  educore_testkit::event_bus::InMemoryEventBus;` without
  taking a direct dep on `educore-event-bus`." But the testkit
  crate's `Cargo.toml` (per `docs/coverage.toml:2183-2195`
  and the crate's own deps) depends on `educore-event-bus`
  to provide the alias; consumers that use `InMemoryEventBus`
  therefore still pull in the `educore-event-bus` crate. The
  alias adds nothing except a name; the lib rustdoc at
  `crates/adapters/event-bus/src/lib.rs:6` refers to the
  same `InProcessEventBus` as the "default" — the testkit's
  rebranding is purely cosmetic. A test consumer that searches
  the engine for "the default bus" finds two names for the
  same type and has to choose between them.
- **expected:** The alias is a stylistic convenience; the
  testkit should either (a) remove the alias and force tests
  to use `InProcessEventBus` directly, or (b) document the
  alias as a stable re-export for `educore-testkit` consumers
  and not as a separate type.
- **evidence:**
  ```rust
  pub use educore_event_bus::InProcessEventBus;
  pub type InMemoryEventBus = InProcessEventBus;
  ```
  And `crates/adapters/event-bus/src/lib.rs:6-19`:
  ```rust
  //! - [`InProcessEventBus`] — the default, always-built, MPMC
  //!   bus backed by [`tokio::sync::broadcast`].
  ```

---

### FINDING 22

- **id:** ADAPT-EB-022
- **area:** adapters-event-bus
- **severity:** Medium
- **location:** `crates/adapters/event-bus/src/nats.rs:13-31`
  and `crates/adapters/event-bus/src/redis.rs:13-32`
- **description:** The NATS and Redis stubs document a
  future subject-mapping convention (`events.<d>.<a>`,
  `events.<d>.>`, `tenant.<s>.>`, `events.>`) in their
  rustdoc but the convention does not match the port-doc at
  `docs/ports/event-bus.md:117-121` which declares topic
  naming as `<domain>.<aggregate>` for aggregates and
  `tenant.<school_id>` for tenants. The NATS stub's proposed
  `events.<d>.<a>` prefix adds a leading `events.` segment
  not in the port doc; the Redis stub's proposed
  `stream:events:<d>:<a>` introduces a `stream:` key prefix
  not in the port doc. When the wire-protocol work lands,
  the consumer will need to translate between the bus-port
  topic strings and the adapter-specific wire strings; this
  translation is not declared anywhere. The
  `Topic::wire()` helper at
  `crates/cross-cutting/events/src/event_bus.rs:206-217`
  returns the bus-port form (no `events.` prefix, no
  `stream:` prefix); the adapter convention diverges.
- **expected:** `docs/ports/event-bus.md:117-121` (topic
  naming convention).
- **evidence:**
  ```text
  - `Aggregate(d, a)` → `events.<d>.<a>`
  - `Domain(d)` → `events.<d>.>`
  - `EventType(t)` → `events.<dotted t>`
  - `Tenant(s)` → `tenant.<s>.>`
  - `All` → `events.>`
  ```
  (in `crates/adapters/event-bus/src/nats.rs:14-21` rustdoc).

---

