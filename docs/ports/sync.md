# Sync Port

## Purpose

The sync port defines the contract that bridges an **offline-first
client** (which embeds the engine and writes to a local SQLite outbox)
and the **central authoritative store** (the consumer's SaaS backend).

The sync port is used when:

- A Tauri / mobile / browser client runs the engine locally and must
  replay locally-committed events to the central API.
- A field-deployment device needs to operate against a remote store
  over a lossy or intermittent connection.
- A new client instance joins a school for the first time and must
  hydrate its local database from the central snapshot.

The sync port is **not** used for in-process eventing between domains —
that is the event bus port (`event-bus.md`). The sync port is
exclusively for **remote replication** of commands and events across the
local / central boundary.

The port is a **build-time feature** on the umbrella. Consumers enable
it via:

```toml
educore = { version = "0.1", features = ["sync"] }
```

Without the feature flag, the `SyncAdapter` trait, the `CommandEnvelope`
type, and the reference implementations are not compiled into the
binary. This keeps the embedded-only footprint (no Tauri app, no CLI
binary that talks to a remote store) free of the HTTP / WebSocket
dependencies.

## Trait: `SyncAdapter`

```rust
#[async_trait]
pub trait SyncAdapter: Send + Sync + std::fmt::Debug {
    async fn dispatch(&self, envelope: CommandEnvelope) -> Result<CommandOutcome>;
    async fn subscribe(&self, filter: EventFilter) -> Result<EventStream>;
    async fn snapshot(&self, school_id: SchoolId) -> Result<SchoolSnapshot>;
    async fn health(&self) -> Result<SyncHealth>;
}
```

The trait is object-safe. Consumers typically use it as
`Arc<dyn SyncAdapter>`.

### `dispatch`

Push a local command to the authoritative store. The engine produces a
`CommandEnvelope` from the command executed against the local outbox;
the adapter transports it to the central API. The returned
`CommandOutcome` indicates whether the central store accepted the
command (`Accepted`), surfaced a conflict (`Conflict { details }`), or
deferred the decision (`Deferred { reason }`).

### `subscribe`

Open a long-lived stream of remote events filtered by `EventFilter`.
The stream is a `BoxStream<Result<EventEnvelope>>` style async
iterator. The client receives events for the given `school_id` whose
version is greater than `since_version`, restricted to the requested
`aggregate_types`. The stream terminates only on `close()` or on
transport error; the client is responsible for reconnecting.

### `snapshot`

Request a full snapshot of a school's aggregates for first-time
hydration. The returned `SchoolSnapshot` includes the snapshot version,
the serialized aggregates, and a list of events that have been emitted
since the snapshot was taken (so the client can apply them
incrementally). The snapshot is **not** a database dump; it is a typed
engine payload that the client replays through the same domain
commands the central store used.

### `health`

Connectivity check. Returns `SyncHealth { reachable, latency_ms,
server_version }`. The client calls this before opening a subscription
or after a transport error to decide whether to retry, back off, or
surface the failure to the user.

## `CommandEnvelope`

```rust
pub struct CommandEnvelope {
    pub idempotency_key: IdempotencyKey,
    pub version: u32,
    pub tenant: TenantContext,
    pub payload: CommandPayload,
}

pub struct TenantContext {
    pub school_id: SchoolId,
    pub actor_id: UserId,
    pub device_id: DeviceId,
    pub session_id: SessionId,
}

pub enum CommandPayload {
    /// A typed command, serialized to its canonical wire form.
    Typed(serde_json::Value),
    /// A pre-serialized event from the local outbox (the common case
    /// for replay after offline).
    OutboxEvent {
        event_id: EventId,
        event_type: &'static str,
        occurred_at: Timestamp,
        body: serde_json::Value,
    },
}
```

`idempotency_key` is the same key the engine produces for the local
command (UUIDv7 derived from the command hash + device id). The
central store uses it for de-duplication per
[`docs/decisions/ADR-014-Idempotency.md`](../decisions/ADR-014-Idempotency.md).
`version` is the envelope's schema version — not the aggregate's
version. `payload` carries either the typed command or the raw outbox
event the client captured locally.

## `EventFilter`

```rust
pub struct EventFilter {
    pub school_id: SchoolId,
    pub since_version: u64,
    pub aggregate_types: Vec<&'static str>,
    pub batch_size: u32,
}
```

`school_id` is mandatory; the central store MUST reject a subscription
that does not scope to a single tenant. `since_version` is the
client's last-seen per-school version cursor; the central store
replays events strictly greater than this value. `aggregate_types` is
an allowlist; an empty list means "all aggregates for this school."
`batch_size` controls how many envelopes are buffered per WebSocket
frame.

## `EventStream`

```rust
#[async_trait]
pub trait EventStream: Send + Sync {
    async fn next(&mut self) -> Option<Result<EventEnvelope>>;
    async fn close(self: Box<Self>) -> Result<()>;
}
```

`EventStream::next` returns `None` when the underlying transport
closes cleanly. On transport error it returns `Some(Err(_))` and the
stream MUST be closed; the client is expected to re-subscribe from the
last acknowledged version.

## `SchoolSnapshot`

```rust
pub struct SchoolSnapshot {
    pub version: u64,
    pub school_id: SchoolId,
    pub aggregates: Vec<SnapshotAggregate>,
    pub events_since: Vec<EventEnvelope>,
}

pub struct SnapshotAggregate {
    pub aggregate_type: &'static str,
    pub rows: serde_json::Value,
}
```

`version` is the per-school version at which the snapshot was taken.
`aggregates` is a list of typed payload blobs, one per aggregate type
the central store has data for. The client deserializes each blob
through the same domain types the central store uses, then applies
`events_since` in order to bring the local copy up to
`version + events_since.last().version`. The snapshot and the events
form a consistent migration point: applying them produces a local
state that is logically equivalent to the central state at
`version + N`.

## `CommandOutcome`

```rust
pub enum CommandOutcome {
    Accepted { version: u64 },
    Conflict { details: ConflictDetails },
    Deferred { reason: String },
}

pub struct ConflictDetails {
    pub server_version: u64,
    pub server_state: serde_json::Value,
    pub conflicting_fields: Vec<String>,
}
```

`Accepted` carries the new per-school version. `Conflict` carries the
server's current state and the field-level diff; the client surfaces
this to the user (or applies a domain rule per the conflict policy in
[`docs/specs/<domain>/services.md`](../specs/)).
`Deferred` is for commands the central store chose to enqueue
(e.g. a finance approval that requires human review); the client
removes the envelope from its outbox and trusts the eventual
notification.

## `SyncHealth`

```rust
pub struct SyncHealth {
    pub reachable: bool,
    pub latency_ms: u32,
    pub server_version: &'static str,
    pub server_schema_version: u32,
}
```

`server_version` is the consumer's central API version (semver). The
client refuses to dispatch or subscribe if its local schema version
is incompatible with `server_schema_version`; the engine surfaces
this as `DomainError::IncompatibleServer`.

## Reference Implementations

| Implementation | Constructor | Use case | Infra cost |
| --- | --- | --- | --- |
| `EducoreSyncAdapter::in_process()` | `EducoreSyncAdapter::in_process()` | Local-first / embedded; the central store is the same process as the client (single-process demos, tests of the offline → central flow with a memory transport). | zero |
| `WorkerHttpSyncAdapter::connect(url, token)` | `WorkerHttpSyncAdapter::connect(url, bearer_token)` | Production: client talks to the consumer's worker binary (`sync-engine/`) over HTTPS + WebSocket. | the worker's hosting cost |
| `NullSyncAdapter` | `NullSyncAdapter::new()` | Tests; single-process demos that do not exercise the network path. Discards all dispatches, returns an empty snapshot, and closes subscriptions immediately. | zero |

`EducoreSyncAdapter` (in-process) is the **default** for local-first
deployments: it does not open any sockets, it shares the engine's
in-process event bus, and the "central" writes go to a separate
storage instance in the same process. It is the right choice for
Tauri desktop apps that the user runs without a hosted backend.

`WorkerHttpSyncAdapter` is the production adapter shipped by the
engine. It implements the wire protocol described in the next section
against the consumer's `sync-engine/` worker. The worker is a thin
relay: it accepts the request, runs it through the engine, and
returns the result. The engine does not impose a specific worker
language; the wire protocol is the contract.

`NullSyncAdapter` is for unit tests and the `cargo run --example
single-process` demo. It is exported from the same crate as the trait
so tests do not need a network.

## Wire Protocol (HTTP + WebSocket)

The wire protocol is the contract between `WorkerHttpSyncAdapter` and
the consumer's `sync-engine/` worker. The worker is expected to be a
thin HTTP server that runs commands through the engine and returns
the result. All requests are authenticated with a device token
(`Authorization: Bearer <token>`). All payloads are JSON. The
Content-Type is `application/json` for request/response bodies; the
WebSocket subprotocol is `educore.sync.v1`.

### `POST /v1/sync`

Dispatch one or more `CommandEnvelope`s to the central store.

Request body:

```json
{
  "envelopes": [
    {
      "idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e01",
      "version": 1,
      "tenant": {
        "school_id": "7c1d8a2e-9b1f-4c3a-8e2d-1a2b3c4d5e6f",
        "actor_id": "0a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d",
        "device_id": "tauri-desktop-mac-001",
        "session_id": "f0e1d2c3-b4a5-9687-6543-210fedcba987"
      },
      "payload": {
        "kind": "outbox_event",
        "event_id": "1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
        "event_type": "academic.StudentAdmitted",
        "occurred_at": "2026-06-12T09:42:11.013Z",
        "body": { /* typed event body */ }
      }
    }
  ]
}
```

Response body (200 OK):

```json
{
  "results": [
    {
      "idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e01",
      "outcome": {
        "kind": "accepted",
        "version": 4711
      }
    }
  ]
}
```

Response body (207 Multi-Status, mixed outcomes):

```json
{
  "results": [
    {
      "idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e01",
      "outcome": { "kind": "accepted", "version": 4711 }
    },
    {
      "idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e02",
      "outcome": {
        "kind": "conflict",
        "details": {
          "server_version": 4709,
          "server_state": { /* the conflicting aggregate as the server sees it */ },
          "conflicting_fields": ["date_of_birth", "guardian_phone"]
        }
      }
    }
  ]
}
```

The server processes envelopes **in order** within a single request.
The order of results matches the order of envelopes in the request.

### `GET /v1/sync/snapshot?school_id=<uuid>`

Request a full snapshot for first-time hydration.

Request: `GET /v1/sync/snapshot?school_id=7c1d8a2e-9b1f-4c3a-8e2d-1a2b3c4d5e6f`

Response body (200 OK):

```json
{
  "version": 4700,
  "school_id": "7c1d8a2e-9b1f-4c3a-8e2d-1a2b3c4d5e6f",
  "aggregates": [
    {
      "aggregate_type": "academic.student",
      "rows": [ /* typed Student payloads */ ]
    },
    {
      "aggregate_type": "academic.class",
      "rows": [ /* typed Class payloads */ ]
    }
  ],
  "events_since": [
    {
      "event_id": "1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
      "event_type": "academic.StudentAdmitted",
      "occurred_at": "2026-06-12T09:42:11.013Z",
      "version": 4701,
      "body": { /* typed event body */ }
    }
  ]
}
```

The client replays the snapshot aggregates through the corresponding
domain constructors, then applies `events_since` in version order.
The result is a local state logically equivalent to the central
state at `version + events_since.last().version`.

### `WS /v1/sync/subscribe`

Open a WebSocket subscription. The client passes the filter as query
parameters.

Request:

```text
GET /v1/sync/subscribe?school_id=<uuid>&since_version=<u64>&aggregate_types=academic.student,academic.class&batch_size=32
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: educore.sync.v1
Authorization: Bearer <token>
```

Server-to-client frames (JSON, one envelope per line):

```json
{
  "event_id": "1a2b3c4d-5e6f-7a8b-9c0d-1e2f3a4b5c6d",
  "event_type": "academic.StudentAdmitted",
  "schema_version": 1,
  "school_id": "7c1d8a2e-9b1f-4c3a-8e2d-1a2b3c4d5e6f",
  "aggregate_id": "0a1b2c3d-4e5f-6a7b-8c9d-0e1f2a3b4c5d",
  "aggregate_type": "academic.student",
  "actor_id": "9a8b7c6d-5e4f-3a2b-1c0d-9e8f7a6b5c4d",
  "correlation_id": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e01",
  "causation_id": null,
  "occurred_at": "2026-06-12T09:42:11.013Z",
  "payload": { /* typed event body */ }
}
```

Client-to-server frames (heartbeat / cursor advance):

```json
{ "kind": "ack", "last_version": 4711 }
```

The client sends an `ack` after successfully persisting each batch.
The server uses the highest `last_version` to compute lag for
metrics. The client does **not** need to ack every event; the
subscription is at-least-once and the client de-duplicates by
`event_id` against its local processed-events table.

The server may close the subscription with a close code:

- `1000` — clean close.
- `4001` — authentication failed; the client must not reconnect.
- `4003` — schema version mismatch; the client must upgrade.
- `4008` — rate limit exceeded; the client may reconnect with backoff.
- `4009` — message too large; the client must reduce `batch_size`.

### `GET /v1/sync/health`

Lightweight liveness probe. No body.

Response body (200 OK):

```json
{
  "reachable": true,
  "latency_ms": 42,
  "server_version": "1.4.2",
  "server_schema_version": 7
}
```

Response (503 Service Unavailable) is also valid; the body shape is
the same with `reachable: false`.

### `POST /v1/sync/conflict/{id}/resolve`

Push a user-resolved conflict back to the central store. The client
sends the same `CommandEnvelope` that originally produced the
conflict, with a `resolution` block describing what the user chose.

Request body:

```json
{
  "idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e02",
  "version": 1,
  "tenant": { "...": "..." },
  "payload": {
    "kind": "typed",
    "body": { /* the user-resolved command */ }
  },
  "resolution": {
    "original_idempotency_key": "0190f8c4-9c1d-7e2a-bb1a-1c6f3b4a2e01",
    "strategy": "client_wins",
    "fields_overridden": ["date_of_birth", "guardian_phone"]
  }
}
```

Response body: same shape as `POST /v1/sync`. The server records the
resolution in the audit log so the central team can review
field-level overrides.

## Auth

All endpoints require a **device token** in the `Authorization`
header:

```text
Authorization: Bearer <device-token>
```

The device token is provisioned by the consumer's identity provider
out of band. It is **not** a user session token — it is a long-lived
token bound to the device. The central store maps the token to a
`(school_id, device_id, allowed_scopes)` tuple and rejects requests
whose `tenant.school_id` does not match the token's school.

The token is the only auth mechanism on the sync port. There is no
cookie auth, no session cookie, and no per-request signature. This is
deliberate: the sync client is a headless process (Tauri worker, CLI
binary) and cannot perform interactive logins. The token is rotated
by the consumer's identity flow; the engine does not define a
rotation policy.

The WebSocket subscription re-uses the same token. Browsers cannot
set `Authorization` on a `WebSocket` constructor; the client passes
the token in the URL only over `wss://` and the server rejects
tokens that arrive over `ws://`.

## Idempotency

The sync port inherits the engine's idempotency contract
([ADR-014](../decisions/ADR-014-Idempotency.md)). The
`idempotency_key` on every `CommandEnvelope` is the same key the
engine produced for the local command; the central store records the
key alongside the resulting state and **silently no-ops** replays
within the idempotency window (24 hours by default).

For the sync path, the central store's idempotency table is keyed on
`(idempotency_key, school_id, device_id)`. This is a strict superset
of the engine's per-school key: the device id is included so a
malicious or buggy device cannot shadow another device's key.

The client may resubmit a `CommandEnvelope` on transport error
without coordinating with the server. The server's response is
deterministic for a given key within the window. The client
**never** retries a `Conflict` outcome; the user must resolve the
conflict (or the policy engine must auto-resolve it) before the
client re-dispatches.

## Version Cursor Semantics

The per-school `version` (u64) is the central store's monotonic
ordering. It is:

- **Monotonically increasing** per school. Every accepted
  `CommandEnvelope` increments it by exactly one.
- **Gaps forbidden.** A successful `POST /v1/sync` increments the
  version by the number of envelopes accepted in the request.
- **Stable across replicas.** The central store sequences versions
  even when the underlying database is sharded; this is the consumer
  worker's responsibility (the engine does not define a replication
  protocol).
- **Snapshot-anchored.** A snapshot's `version` is the cursor at
  which the snapshot was taken. Applying `events_since` brings the
  client to a version equal to
  `snapshot.version + events_since.last().version - snapshot.version`
  — i.e. the maximum version in `events_since`.

The client tracks the highest version it has **persisted and
acknowledged** in its local storage. On reconnect, the client passes
this value as `since_version`. The server replays every event with
`version > since_version` in increasing order. The client persists
the events **in order** before sending the next `ack`.

If the client's local version is **ahead of the server's**, the
client MUST fall back to a snapshot. The server signals this with a
`409 Conflict` on `POST /v1/sync` with `details.code = "client_ahead"`.
The client takes a fresh snapshot and replays its pending envelopes
against the new state.

## Conflict Surfacing

When the central store rejects a `CommandEnvelope` because applying
it would violate a domain rule or overwrite a more recent server
state, the response is a **4xx**:

- `409 Conflict` — the most common case. The response body is a
  `CommandOutcome::Conflict` (per-envelope in a `POST /v1/sync` batch,
  or the top-level body for a single-envelope dispatch).
- `422 Unprocessable Entity` — the command itself is malformed
  against the server's current schema (e.g. a `Student` admission
  with an unknown section id). The client surfaces this as a bug and
  does not retry.
- `423 Locked` — the aggregate is locked by an in-flight command on
  the server. The client retries with backoff.

The 4xx response includes the full `ConflictDetails` so the client
can present the server's view to the user without a follow-up
roundtrip. The client **does not** interpret the conflict; it
surfaces it to the user (or the policy engine) and waits for a
resolution. The resolution arrives as a fresh `POST /v1/sync` with
the `resolution` block described above, or as a manual edit in the
central web UI that the user then re-replays.

The server records every conflict in the audit log with the
`idempotency_key`, the `server_version`, the `client_payload`, and
the `conflicting_fields`. The audit log is the canonical record for
postmortems.

## Configuration

The `WorkerHttpSyncAdapter` is constructed by the consumer with their
own configuration:

```rust
let sync: Arc<dyn SyncAdapter> = Arc::new(
    WorkerHttpSyncAdapter::builder()
        .url(env::var("SYNC_ENGINE_URL")?)
        .device_token(env::var("DEVICE_TOKEN")?)
        .bearer_token(env::var("DEVICE_TOKEN")?)
        .request_timeout(Duration::from_secs(15))
        .subscription_reconnect_backoff(Backoff::exponential(
            Duration::from_millis(500),
            Duration::from_secs(30),
        ))
        .batch_size(32)
        .build()
        .await?,
);
```

The in-process and null adapters take no configuration.

## Object Safety

`SyncAdapter` is object-safe. `EventStream` is object-safe. Consumers
may store `Arc<dyn SyncAdapter>` and use it across threads.

## Worked Example

A Tauri client embeds the engine and syncs to a hosted backend:

```rust
let storage: Arc<dyn StorageAdapter> = Arc::new(
    SqliteStorage::open(local_db_path).await?,
);
let sync: Arc<dyn SyncAdapter> = Arc::new(
    WorkerHttpSyncAdapter::connect(env::var("SYNC_ENGINE_URL")?, device_token)
        .await?,
);

let engine = Engine::builder()
    .storage(storage.clone())
    .sync(sync.clone())
    .event_bus(InProcessBus::new())
    .build()
    .await?;

// On startup: hydrate from a snapshot if this is a fresh install.
if is_fresh_install() {
    let snapshot = sync.snapshot(school_id).await?;
    apply_snapshot(&storage, &snapshot).await?;
}

// Open a subscription for ongoing replication.
let mut stream = sync.subscribe(EventFilter {
    school_id,
    since_version: storage.last_version(school_id).await?,
    aggregate_types: vec![],
    batch_size: 32,
}).await?;

while let Some(event) = stream.next().await {
    let event = event?;
    storage.apply_event(&event).await?;
    storage.record_ack(event.school_id, event.version).await?;
}
```

A command runs locally and is replayed centrally:

```rust
// Local: commit to SQLite + outbox.
let mut tx = storage.begin().await?;
let outcome = engine.handle(AdmitStudent { ... }).await?;
tx.commit().await?;

// Replay: dispatch the outbox event to the central store.
let envelope = CommandEnvelope::from_outbox(&outcome.event);
match sync.dispatch(envelope).await? {
    CommandOutcome::Accepted { version } => storage.record_ack(school_id, version).await?,
    CommandOutcome::Conflict { details } => surface_to_user(details).await?,
    CommandOutcome::Deferred { reason } => log::info!("deferred: {reason}"),
}
```

## Testing

The port requires:

- Unit tests of every trait method.
- An integration test against a `WorkerHttpSyncAdapter` running
  against a local `sync-engine/` worker (testcontainers).
- A round-trip test: client dispatches → server applies → server
  publishes → client subscribes → client receives.
- A conflict test: two clients dispatch conflicting commands; the
  second receives a 4xx with `ConflictDetails`.
- A reconnect test: subscription closes mid-flight; client
  reconnects from the last acked version.
- A version-cursor test: client with a stale `since_version` receives
  the correct replay window.
- A snapshot test: fresh client takes a snapshot, replays, and
  converges to the server's state.
- A failure-injection test: dispatch with the device token revoked
  → 401; dispatch with a malformed envelope → 422.

## Offline Mode

The sync port is the **only** port whose default reference
implementation works without a network. The in-process adapter
(`EducoreSyncAdapter::in_process()`) routes commands to a
second storage instance in the same process — the "central" store
is a logical concept, not a physical one. Offline-first clients
configure the in-process adapter and never enable the network code
path; production clients configure `WorkerHttpSyncAdapter` and
rely on the worker for remote replication.

The local outbox (owned by the storage adapter, per `storage.md`)
is the source of truth for un-replicated commands. The sync port
is invoked only when connectivity is available; the engine does
not gate commands on sync health.

## Audit

The sync port records the following audit events:

- `SyncDispatched` — `envelope.idempotency_key`, `school_id`,
  `device_id`, `payload.kind`, outcome (`accepted` / `conflict` /
  `deferred`), `version` on success.
- `SyncConflictSurfaced` — `envelope.idempotency_key`,
  `conflicting_fields`, `server_version`, `server_state_hash`.
- `SyncConflictResolved` — `original_idempotency_key`,
  `strategy`, `fields_overridden`, `actor_id`.
- `SyncSnapshotTaken` — `school_id`, `version`, `aggregate_count`.
- `SyncSubscribed` — `school_id`, `since_version`, `consumer`.
- `SyncHealthFailed` — `latency_ms`, `error_code` (when
  `health()` returns `reachable: false`).
- `SyncReconnected` — `school_id`, `gap_versions` (the number
  of versions between the last acked version and the server's
  current version at reconnect time).

The central store writes these audit events to the same `audit_log`
table the engine uses for command audit. The client writes a
subset (`SyncDispatched`, `SyncHealthFailed`, `SyncReconnected`)
to its local audit log for field debugging.
