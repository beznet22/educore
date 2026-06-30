# Sync — Services

Sync ships two service surfaces: the **in-process reference
implementation** (`SyncCoordinator`) used by the engine
binary and the parity test suite, and the **worker HTTP
client** (`WorkerHttpSyncAdapter`) used by the consumer's
worker process. Both share the same bookkeeping code path
(outbox, cursor, conflict, audit, idempotency); only the
transport differs.

## SyncCoordinator

The reference implementation runs in the same process as the
domain runtime. It is the simplest possible sync: no
transport, no server — just the outbox, cursor, and conflict
machinery wired to the local storage adapter.

**Responsibilities:**

- Owns the per-`(school, aggregate_type)` subscription state.
- Runs the **push loop**: claim pending outbox entries, mark
  them `InFlight`, apply them through the local command bus,
  record the result, emit the appropriate event.
- Runs the **pull loop**: subscribe to local change events,
  advance cursors, dispatch through the local command bus as
  `ApplyRemoteChangeCommand`.
- Manages snapshot hydration on first contact and on schema
  upgrades.
- Surfaces conflicts via `ConflictReported`.
- Exposes pause / resume / school-switch as command handlers.

The `SyncCoordinator` is **not** the only implementation. It
is the reference; the storage-port-driven machinery is what
every deployment uses. A custom service (e.g. a federation
hub) may substitute its own coordinator as long as it
implements the same port surface — see
[`ports.md`](./ports.md).

## WorkerHttpSyncAdapter

The worker binary (`educore-worker`) runs in a different
process from the server, on the same machine or on a separate
node. It uses the **HTTP transport** to talk to
`educore-sync-server`.

**Responsibilities:**

- Owns the local outbox, cursor, conflict, and subscription
  state on the worker side.
- Pushes local commands to the server's REST endpoint with
  retries and exponential backoff.
- Opens a long-poll or WebSocket connection for the change
  feed, applies incoming events through the worker's local
  command bus, advances cursors.
- Hydrates snapshots on first contact via the snapshot
  endpoint.
- Translates HTTP errors (`409 Conflict`, `410 Gone`, `412
  Precondition Failed`) into `ConflictReported` events.

The adapter is **purely a transport binding**. All the
bookkeeping logic — outbox ordering, cursor monotonicity,
conflict detection, idempotency, audit — lives in
`educore-core` and is shared with the in-process reference
implementation.

## Saga / Compensating Actions

Multi-step workflows (e.g. "apply remote change, advance
cursor, write audit row, notify bus") live behind the
[`Saga<S>`](../../cross-cutting/sync/src/saga.rs) state
machine. Each step declares a forward action and a
compensating action; if any step fails the saga walks back
through the completed steps in reverse order and invokes
each compensation, restoring the system to a consistent
state. Compensation is idempotent: a second invocation is a
no-op. See `crates/cross-cutting/sync/src/saga.rs` for the
typed `SagaStep<I, O>`, `SagaResult` (`Completed`,
`Compensated`, `Failed`), and `SagaError` surface.

## In-Process Sync Adapter (Phase 0)

The Phase 0 minimum viable ships
[`InProcessSyncAdapter`](../../../crates/cross-cutting/sync-inprocess/src/lib.rs)
— the reference implementation of `SyncAdapter` that
publishes the four typed session events through an
in-process `EventBus`. It holds `Arc<dyn EventBus>` and an
`Arc<Mutex<SyncHealth>>`; the five trait methods (`start`,
`pause`, `resume`, `stop`, `health`) forward to
`send_command` which updates the health snapshot and
publishes the typed event. This adapter is the Phase 0
parity test target and the production adapter for
single-process desktop / CLI deployments.
