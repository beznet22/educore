# Cluster B — Workflow Infrastructure

**Root cause:** Cross-domain workflows are documented in
`docs/specs/*/workflows.md` but not implemented. Zero subscribers are
registered with the event bus. The transactional outbox is drained and
discarded at commit time instead of being relayed to the bus. No
`tests/workflows.rs` file exists for any domain.

**Estimated findings:** ~80 (Critical-heavy)

**Source ID prefixes:** `WF-*`, `CC-*` (workflow subset), `CROSSCUT-*` (event-bus subset), `ADAPT-EB-*`

**Blocks deploy:** Yes — workflows are the engine's primary user-facing
feature (admission, promotion, payroll disbursement, attendance roll-up).
A workflow that "succeeds" but doesn't propagate is silently broken.

**Estimated fix scope:** Medium. Touches 1 cross-cutting crate
(`educore-events`), 1 adapter (`educore-event-bus`), 1 tool (`educore-testkit`),
and 10 domain crates (for subscriber wiring).

## Why these findings cluster

The spec-mandated workflow pattern is:

```
1. Domain A command runs.
2. Domain A command emits DomainEvent E to outbox (transactional).
3. Outbox relay reads pending E, publishes to bus.
4. Bus delivers E to all subscribers (Domain B's subscriber, Domain C's).
5. Each subscriber's handler runs in its own command (which writes its
   own outbox row, and so on).
```

The audit found that step 3 is broken (outbox drain is a no-op per
`wave4-testkit.md` TOOL-TK-001). Step 4 has no subscribers (per
`wave7-workflows.md` WF-002). Step 5 has no handlers because step 4 has
no subscribers (chicken-and-egg).

Additionally, no integration test exercises any of this (per
`wave7-workflows.md` WF-001).

## Representative findings

| Source | ID | Sev | Stage | One-line |
|---|---|---|---|---|
| `wave7-workflows.md` | WF-001 | C | 5 | Zero `tests/workflows.rs` files |
| `wave7-workflows.md` | WF-002 | C | 4 | Zero cross-domain subscribers wired |
| `wave7-workflows.md` | WF-016 | C | 4 | `form_uploaded_public_indexing_subscriber` is a phantom (never registered) |
| `wave4-testkit.md` | TOOL-TK-001 | C | 3 | In-memory outbox drain is a no-op |
| `wave3-event-bus.md` | ADAPT-EB-001 | C | 3 | In-process bus ack/nack are no-ops |
| `wave3-event-bus.md` | ADAPT-EB-005 | C | 3 | No outbox drain in the in-process adapter |
| `wave3-event-bus.md` | ADAPT-EB-007 | C | 3 | Broadcast channel silent failures |
| `wave2-events.md` | CC-EVT-001 | C | 4 | `EventEnvelope` missing `recorded_at` / `metadata` |
| `wave2-events.md` | CC-EVT-004 | C | 4 | `DomainEvent` trait shape drifts across domains |
| `wave2-sync.md` | CC-SYNC-001 | C | 3 | Missing umbrella `sync` feature; no outbox drain |
| `wave2-sync.md` | CC-SYNC-005 | C | 3 | Three-way API drift between ADR / port doc / spec |

## What fixing this requires

**Step 1: outbox relay (unblocks everything)**

- Implement an outbox relay task. Reads `Outbox::pending(...)`, publishes
  via `EventBus::publish(...)`, marks `published_at` on success.
- For the in-process adapter (per `wave3-event-bus.md`), the relay can
  run as a `tokio::task::spawn`.
- For external adapters (NATS, Redis Streams), the relay talks to the
  external bus.
- Idempotent: re-delivery of the same `event_id` is a no-op on the
  consumer side.

**Step 2: subscriber registration**

- Define a `subscribe(handle, filter, handler)` API on `EventBus`.
- Each domain crate registers its subscribers at startup. Per the spec
  workflows, this includes ~30 subscriber functions across 10 domains.
- The single existing `form_uploaded_public_indexing_subscriber`
  (`crates/domains/cms/`) must actually be registered.

**Step 3: bus port completion**

- Fix `BatchReceipt::is_fully_accepted` bug per `wave2-events.md`.
- Implement proper ack/nack semantics (not no-ops).
- Add retry policy + dead-letter queue.
- Honor the at-least-once delivery contract per `docs/ports/event-bus.md`.

**Step 4: envelope schema**

- Add `recorded_at` and `metadata` fields to `EventEnvelope`.
- Standardize the `DomainEvent` trait shape across all 10 domains.

**Step 5: tests/workflows.rs**

- For each domain, write `tests/workflows.rs` exercising the spec'd
  workflows (admission, promotion, payroll, attendance roll-up, etc.).
- Tests must use the testkit (with the relay actually working) end-to-end.

## Suggested fix sequence

1. **Fix `EventEnvelope` schema** in `crates/cross-cutting/events/src/envelope.rs`.
   Add `recorded_at`, `metadata`. Update serialization tests.
2. **Implement proper ack/nack** on `EventBus`. The `BatchReceipt` bug
   in `wave2-events.md` is the entry point.
3. **Implement outbox relay** as a standalone task in
   `crates/cross-cutting/events/src/relay.rs`. Wire it into the
   testkit's in-memory adapter so tests can exercise it.
4. **Standardize `DomainEvent` trait** across all 10 domains. Per the
   audit, the trait shape drifts (`EVENT_TYPE` vs `TYPE`, missing
   `SCHEMA_VERSION`, etc.). Pick one canonical shape and migrate.
5. **Add subscriber registration** to each domain's `lib.rs`. For the
   academic domain: ~6 subscribers (per spec). For finance: ~4. Etc.
6. **Add `tests/workflows.rs`** per domain. Each test should:
   - Run a command that emits an event
   - Wait for the relay to publish
   - Assert the subscriber's downstream command was invoked
7. **Add `crates/tools/storage-parity/tests/workflows.rs`** to exercise
   workflows across all 4 backends.

## Verification criteria

- `cargo test -p educore-events` passes with ack/nack semantics.
- `cargo test -p educore-testkit` shows the outbox is drained **and** the
  bus subscribers received the events.
- `cargo test --workspace` shows no `tests/workflows.rs` is missing.
- The phantom `form_uploaded_public_indexing_subscriber` is actually
  invoked in `crates/domains/documents` integration tests.
- `docs/coverage.toml` rows for `workflows.md` go from 0 → ~11.

## Risk if left unfixed

- Every spec workflow (`docs/specs/*/workflows.md`) is a fiction. Admission,
  promotion, payroll, attendance roll-up all silently no-op the
  cross-domain side-effects.
- The outbox table fills up forever (no relay).
- The audit-found "phantom subscribers" remain (code that looks like a
  subscriber but is wired to nothing).
- Consumers cannot reason about at-least-once delivery.

## Cross-cluster dependencies

- **Unblocks:** Cluster C (workflows.md per spec becomes exercisable),
  Cluster F (event-bus adapter completeness can be tested).
- **Depends on:** Cluster A (transactions must atomically write outbox +
  aggregate), Cluster D (port traits must be stable).

## Files involved

- `crates/cross-cutting/events/src/envelope.rs` (the envelope)
- `crates/cross-cutting/events/src/event_bus.rs` (the port trait)
- `crates/cross-cutting/events/src/relay.rs` (new — the relay)
- `crates/adapters/event-bus/src/in_process.rs` (the in-proc adapter)
- `crates/tools/testkit/src/storage.rs` (the testkit outbox drain)
- `crates/domains/*/src/lib.rs` (subscriber registration per domain)
- `crates/domains/*/tests/workflows.rs` (per-domain integration tests)
- `docs/specs/*/workflows.md` (11 files; spec source of truth)
- `docs/ports/event-bus.md` (port contract)
