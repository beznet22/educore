# educore-event-bus

Event bus adapter implementations for the Educore engine.

The bus **port** lives in the [`educore-events`](../../cross-cutting/events/src/event_bus.rs) crate
(`EventBus`, `EventSubscription`, `Topic`, `SubscribeOptions`, `EventFilter`,
`StartPosition`, `ConsumerId`, `PublishReceipt`, `BatchReceipt`,
`EventError`). This crate supplies three concrete adapters:

- **`InProcessEventBus`** — the default, always-built, MPMC bus backed by
  [`tokio::sync::broadcast`]. MPMC; bounded channel per subscription; bounded
  replay log for `StartPosition::Earliest` and the `FromEventId` /
  `FromTimestamp` cursor modes. In-process delivery is direct, so `ack` /
  `nack` are no-ops returning `AckOutcome::Accepted`.

- **`NatsEventBus`** — gated behind the `nats` Cargo feature. Phase 2
  stub: the type, constructor, `connect`, and trait surface are wired;
  all trait methods return `EventError::NotSupported`. Wire-protocol work
  (NATS JetStream) lands in a future phase.

- **`RedisEventBus`** — gated behind the `redis` Cargo feature. Phase 2
  stub: same shape as `NatsEventBus`. Wire-protocol work (Redis Streams +
  consumer groups) lands in a future phase.

## Feature flags

```text
default = ["in-process"]
in-process = []            # always built (default)
nats = ["dep:async-nats"]
redis = ["dep:redis"]
```

The `in-process` bus is the default and is the only adapter always
compiled. `nats` and `redis` are opt-in; their dependency crates
(`async-nats ^0.33`, `redis ^0.25`) are pulled in only when the
corresponding feature is enabled.

## Usage

```rust,no_run
use std::sync::Arc;
use educore_event_bus::InProcessEventBus;
use educore_events::event_bus::EventBus;

let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
```

Distributed adapters (when the feature is enabled):

```rust,no_run
# #[cfg(feature = "nats")] {
use educore_event_bus::NatsEventBus;
let _bus = NatsEventBus::new(); // Phase 2 stub
# }
```

## Phase 2 scope

This crate lands the type surface and Cargo feature gates for all three
adapters. The in-process bus is a complete implementation; the NATS and
Redis adapters are scaffolds that return `NotSupported` from every
trait method. See:

- `docs/ports/event-bus.md` for the bus-port contract.
- `docs/decisions/ADR-015-ExternalCrates.md` for the crate pin policy.
- `docs/build-plan.md` § "Phase 2" for the workstream-A / workstream-B
  split.

## Tests

- Unit tests in each module cover construction, configuration
  clamping, topic and start-position predicates, and (for the stubs)
  `NotSupported` responses.
- Integration tests in `tests/in_process_e2e.rs` cover the round-trip,
  fan-out, filter, cross-tenant isolation, close, `Latest` vs.
  `Earliest`, and the feature-gated stub behaviour.
