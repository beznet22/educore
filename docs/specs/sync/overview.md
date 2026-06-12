# Sync Specification

## Overview

The sync subsystem is a **cross-cutting concern** that keeps a remote
Educore server and a local Educore runtime in step. It is not a
domain: it does not own any business state. It owns the
**bookkeeping** of changes — what was sent, what was received, where
each side stopped reading, and what the user must resolve by hand.

A local runtime (an offline-first desktop or mobile client, a
branch-server, or a build-time seed pipeline) executes commands and
applies remote changes. The sync subsystem is the part of the
runtime that:

1. Records every locally produced domain event into a durable
   outbox.
2. Pushes pending outbox entries to the server and applies the
   server's acks.
3. Pulls a per-aggregate **change feed** from the server, advances a
   local cursor, and applies remote events.
4. Hydrates a **full snapshot** on first contact and on schema
   upgrades, so the client can run read queries while disconnected.
5. Surfaces conflicts to the user, persists them until resolved, and
   replays the user's resolution back to the server.
6. Supports **per-school** sync contexts so a user with access to
   multiple schools can switch contexts without losing local state.

Sync runs as a **build feature** of the engine. The
`educore::sync` module is gated behind a Cargo feature so the core
engine library can be compiled without any sync dependency for
embedded / server-only use cases.

## Boundaries

The sync subsystem does **not** own:

- Domain logic (academic, finance, attendance, …). Sync reads and
  writes aggregates through the public command and event APIs of each
  domain crate. It never reaches into a domain's private state.
- The transport protocol. The wire format is the responsibility of
  `educore-sync-server` and the worker's HTTP client. Sync defines
  port traits; transport adapters implement them.
- The server's authorization model. Sync sends a tenant-scoped token
  and lets the server decide. It does not enforce RBAC on the
  remote call.
- Conflict resolution policy. The engine exposes conflict surfaces;
  the consumer (the desktop app, the mobile app, the build pipeline)
  decides how to present them and what rule to apply.

## Dependencies

- `educore-core` — error types, identifiers, result.
- `educore-platform` — `SchoolId`, `UserId`, `TenantContext`,
  `CorrelationId`.
- `educore-events` — the `DomainEvent` trait, `EventEnvelope`,
  `EventBus` port.
- `educore-rbac` — capability checks (`Sync.Request` and friends).
- One or more **domain crates** (academic, finance, attendance, …)
  for the aggregates that flow through sync.
- `educore-storage` — the local storage port (sync runs against
  any storage adapter that supports the change-feed surface).
- `educore-sync-server` (port) and the wire implementation
  (e.g. `educore-sync-server-http`) for the transport.

## Domain Invariants

The sync subsystem maintains the following invariants. Violating
any of these is a bug; every test in the parity suite covers at
least one.

1. **Outbox durability.** A command that has been acknowledged by
   the local store (`CommandAccepted`) is durably in the outbox
   before the user's "submit" returns.
2. **At-least-once push.** A pushed outbox entry is removed only
   after the server has acknowledged it with a stable id. A network
   failure mid-push leaves the entry in the outbox for retry.
3. **Idempotent apply.** Applying the same remote event twice has
   the same effect as applying it once. The event envelope carries
   a stable `event_id`; the local store dedupes by that id.
4. **Cursor monotonicity.** For a given `(school_id, aggregate_type,
   aggregate_id)` tuple, the local cursor only advances forward. A
   server-side reset produces a snapshot, not a cursor move.
5. **Snapshot exclusivity.** While a snapshot is being hydrated,
   tail-applied events queue. They are merged in cursor order after
   the snapshot commits. A client never sees a half-hydrated
   snapshot.
6. **Conflict immutability.** A `ConflictRecord` is append-only. The
   user's resolution is a new `ConflictResolved` event; the record
   itself is never edited in place.
7. **Tenant isolation.** Sync cursors, outbox entries, conflict
   records, and subscriptions are strictly scoped by `SchoolId`. A
   `SwitchSchoolCommand` atomically swaps the active context; in-
   flight sync work for the previous school is suspended, not
   aborted.
8. **Pause semantics.** A paused subscription does not advance its
   cursor. The server-side stream stays open; the client simply
   stops reading. Resume picks up at the last cursor value.
9. **Idempotency on commands.** Every sync command carries an
   `IdempotencyKey`. Resubmitting the same key within the dedupe
   window returns the prior result, not a duplicate execution.
10. **Audit completeness.** Every state change in the sync subsystem
    — push, pull, snapshot, conflict, resolution, pause, resume,
    school switch — is written to `sync_audit` in the same
    transaction as the state change.

## Aggregates

Sync is a cross-cutting concern, so its aggregates are
**bookkeeping aggregates**, not business aggregates. They live
alongside the domain aggregates and obey the same tenant and
audit rules. Each aggregate is documented in detail below.

| Aggregate          | Root Type          | Purpose                                            |
| ------------------ | ------------------ | -------------------------------------------------- |
| OutboxEntry        | `OutboxEntry`      | A locally produced command waiting to be pushed   |
| SyncCursor         | `SyncCursor`       | A per-aggregate "last applied server version" mark |
| ConflictRecord     | `ConflictRecord`   | A divergence between local and server state        |
| SyncSubscription   | `SyncSubscription` | A per-school, per-aggregate-type change-feed state |

### OutboxEntry

**Root type:** `OutboxEntry`
**Identity:** `OutboxEntryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cross-cutting (sync)

#### Purpose

Represents a domain command that the local runtime has accepted and
is committed to push to the server. The outbox is the durable
bridge between "command accepted" and "server has the change".

#### Owned Children

- `OutboxAttempt` — one per push attempt (successes and failures
  are kept for audit; the row is removed only after a final ack).
- `OutboxAttachment` — zero or more file references attached to the
  command payload.

#### Invariants

1. An `OutboxEntry` is created in the same transaction as the
   domain event that records the command's local acceptance.
2. An entry's `Status` transitions are: `Pending → InFlight →
   Acked`. From `InFlight`, transitions to `Pending` (network
   failure, server 5xx) and to `Conflict` (server 409) are allowed.
3. An entry in `Acked` status is removed by a separate compaction
   job after the retention window. The audit row is retained.
4. An entry carries the full command envelope — no live references
   to in-memory state.
5. The `IdempotencyKey` is the primary dedupe surface on the server.

#### Commands

- `EnqueueOutboxEntry` (internal; produced by the command bus)
- `MarkOutboxEntryInFlight`
- `MarkOutboxEntryAcked`
- `MarkOutboxEntryFailed`
- `MarkOutboxEntryConflicted`
- `CompactOutboxEntries`

#### Events

- `OutboxEntryEnqueued`
- `OutboxEntryInFlight`
- `OutboxEntryAcked`
- `OutboxEntryFailed`
- `OutboxEntryConflicted`
- `OutboxDrained` (emitted when the queue empties)

#### Consistency Boundary

All outbox mutations are serialized per `(school_id, command_id)`.
Two clients with the same `IdempotencyKey` collapse to one entry.
The local command bus writes the outbox row in the same transaction
as the domain event.

### SyncCursor

**Root type:** `SyncCursor`
**Identity:** `SyncCursorKey { school_id, aggregate_type,
aggregate_id }`
**Tenant:** `SchoolId`
**Bounded context:** Cross-cutting (sync)

#### Purpose

Marks the last version of a specific aggregate that the local
runtime has applied from the server. It is the "where did I stop
reading" mark used to resume a change feed.

#### Invariants

1. A cursor exists for an aggregate only if the local runtime has
   ever read a server-side change for it (or has hydrated a
   snapshot containing it).
2. A cursor is monotonically non-decreasing. The
   `advance_cursor` repository method is a no-op if the new
   version is less than or equal to the current one.
3. A cursor's `version` is opaque to the client; it is whatever the
   server's change-feed returns (LSN, event_log id, HLC, …).
4. A `SyncCursor` is tenant-scoped. Switching schools does not
   delete cursors; it makes a different set active.

#### Commands

- `InitializeCursor` (created during snapshot hydration)
- `AdvanceCursor`

#### Events

- `CursorInitialized`
- `CursorAdvanced`

#### Consistency Boundary

Cursor mutations are atomic with the application of the remote
event. If applying the event fails, the cursor does not move.
Cursors are not user-facing; they are the engine's internal
checkpoint.

### ConflictRecord

**Root type:** `ConflictRecord`
**Identity:** `ConflictId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Cross-cutting (sync)

#### Purpose

Captures a divergence between a local change and a server change
that the engine could not reconcile automatically. The record is
the durable surface that the UI presents to the user; the
resolution is a new event, not an edit to the record.

#### Invariants

1. A `ConflictRecord` references the local `OutboxEntry` and the
   server's `EventEnvelope` that diverge.
2. A record has a `Status` of `Open` or `Resolved`. The transition
   is `Open → Resolved`. There is no re-opening.
3. A record carries both sides of the conflict (local command
   payload and remote event payload) plus a typed `ConflictKind`
   (field-level, version-stale, deleted-on-remote, schema-mismatch,
   …).
4. Resolutions are first-class events. A `ConflictRecord` is
   append-only; the resolution lives in a new
   `ConflictResolved` event in the audit log.
5. A `ConflictRecord` is the only sync aggregate that blocks
   cursor advance on a specific aggregate. While a conflict is
   `Open`, no later remote change for the same aggregate is
   applied.

#### Commands

- `OpenConflictRecord`
- `ResolveConflict` (records the user's choice)

#### Events

- `ConflictReported` (the conflict was detected)
- `ConflictResolved` (the user chose a side or merged)

#### Consistency Boundary

Conflict detection runs as part of remote-event application. The
record is written in the same transaction as the rejection of the
remote event. The local pending command remains in the outbox
under `Conflict` status until the resolution is pushed and acked.

### SyncSubscription

**Root type:** `SyncSubscription`
**Identity:** `SyncSubscriptionId { school_id, aggregate_type,
client_id }`
**Tenant:** `SchoolId`
**Bounded context:** Cross-cutting (sync)

#### Purpose

Tracks the state of a per-aggregate-type change feed for a single
client. A subscription is the unit of pause / resume, retry, and
error accounting. The local runtime typically holds one
subscription per (school, aggregate_type) pair.

#### Invariants

1. A subscription has a `State` of `Idle`, `Streaming`, `Backoff`,
   `Paused`, or `Stalled`. The transitions are documented under
   [§ Workflows](#workflows).
2. A subscription carries a backoff policy (initial, multiplier,
   cap) and a stream cursor. A subscription in `Backoff` does not
   read; it waits for the next retry window.
3. A subscription is per-client. A second device for the same user
   holds its own subscription; the server keeps cursors per
   client.
4. Pause keeps the subscription's stream alive on the server; the
   client simply stops reading. Resume picks up at the last cursor.

#### Commands

- `StartSubscription`
- `PauseSubscription`
- `ResumeSubscription`
- `ReportSubscriptionError`
- `MarkSubscriptionStreaming`
- `MarkSubscriptionStalled`

#### Events

- `SubscriptionStarted`
- `SubscriptionStateChanged`
- `SubscriptionPaused`
- `SubscriptionResumed`
- `SubscriptionErrored`
- `SubscriptionStalled`

#### Consistency Boundary

Subscription state is local to the client. The server does not see
pause / resume transitions; it only sees the cursor. This keeps
the server stateless about the client's transport health.

## Events

Sync events are domain events: they are append-only, immutable, and
written to the audit log. They implement the engine's
`DomainEvent` trait and ride the same event bus as business events.

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

### SyncStarted

```rust
pub struct SyncStarted {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub from_version: VersionCursor,
    pub request_id: Uuid,
}
```

**Subscribers:**
- `educore-rbac` — audit the request and bind the user.
- The local `SyncCoordinator` — switch the subscription from
  `Idle` to `Streaming`.

### SyncCompleted

```rust
pub struct SyncCompleted {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub to_version: VersionCursor,
    pub events_applied: u32,
    pub duration_ms: u64,
}
```

**Subscribers:**
- The local `SyncCoordinator` — switch the subscription from
  `Streaming` to `Idle`.
- The local UI binding — refresh derived queries.

### SnapshotHydrated

```rust
pub struct SnapshotHydrated {
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub version: VersionCursor,
    pub rows_hydrated: u64,
    pub duration_ms: u64,
    pub tail_events_merged: u32,
}
```

**Subscribers:**
- The local `SyncCoordinator` — emit
  `SubscriptionStateChanged` to `Streaming` once the tail is
  merged.
- The local UI binding — show "ready" state.

### ConflictReported

```rust
pub struct ConflictReported {
    pub conflict_id: ConflictId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub aggregate_id: Uuid,
    pub conflict_kind: ConflictKind,
    pub local_outbox_id: OutboxEntryId,
    pub remote_event_id: EventId,
}
```

**Subscribers:**
- The local UI binding — surface the conflict to the user.
- `educore-audit` — record the open conflict for compliance.

### ConflictResolved

```rust
pub struct ConflictResolved {
    pub conflict_id: ConflictId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub aggregate_id: Uuid,
    pub resolution: ConflictResolution,
    pub resolved_by: UserId,
    pub resolved_at: Timestamp,
}
```

**Subscribers:**
- The local `SyncCoordinator` — re-enqueue the local outbox entry
  with the resolution applied, or accept the remote side and
  discard the local.
- The local UI binding — clear the conflict surface.

### OutboxDrained

```rust
pub struct OutboxDrained {
    pub school_id: SchoolId,
    pub drained_at: Timestamp,
    pub remaining: u32,
}
```

**Subscribers:**
- The local UI binding — clear the "syncing…" indicator.
- The local `SyncCoordinator` — close the push loop.

### SubscriptionStateChanged

```rust
pub struct SubscriptionStateChanged {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub from_state: SubscriptionState,
    pub to_state: SubscriptionState,
    pub reason: Option<SubscriptionStateReason>,
}
```

**Subscribers:**
- The local UI binding — show connectivity state.
- The local `SyncCoordinator` — drive the next phase (e.g. enter
  `Backoff` after `Errored`).

### Outbox Entry Events

The outbox emits one event per status transition. They are listed
in the aggregate's `Events` block above:

- `OutboxEntryEnqueued { outbox_id, command_id, idempotency_key, payload_ref }`
- `OutboxEntryInFlight { outbox_id, attempt_no, sent_at }`
- `OutboxEntryAcked { outbox_id, server_event_id, acked_at }`
- `OutboxEntryFailed { outbox_id, attempt_no, error_class, will_retry }`
- `OutboxEntryConflicted { outbox_id, conflict_id }`

### Cursor Events

- `CursorInitialized { school_id, aggregate_type, aggregate_id, version }`
- `CursorAdvanced { school_id, aggregate_type, aggregate_id, from_version, to_version }`

## Commands

Sync commands are the entry points for both the local UI and the
internal command bus. They obey the same idempotency, RBAC, and
audit rules as domain commands.

### RequestSyncCommand

```rust
pub struct RequestSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
}
```

Begins a sync session. Resolves to a `SyncStarted` event per
aggregate type and, where no snapshot exists, a
`SnapshotHydrated` followed by a streaming pull.

### PauseSyncCommand

```rust
pub struct PauseSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub reason: Option<String>,
    pub idempotency_key: IdempotencyKey,
}
```

Stops the local reader for the listed aggregate types. The
server-side stream stays open; the cursor does not move. Emits
`SubscriptionStateChanged` to `Paused`.

### ResumeSyncCommand

```rust
pub struct ResumeSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

Resumes reading from the last cursor. Emits
`SubscriptionStateChanged` to `Streaming`.

### ResolveConflictCommand

```rust
pub struct ResolveConflictCommand {
    pub conflict_id: ConflictId,
    pub resolution: ConflictResolution,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

Records the user's choice for an open conflict. Emits
`ConflictResolved`. The local outbox entry that was in
`Conflict` status is re-pushed with the resolution applied (or
discarded if the user accepted the remote side).

### SwitchSchoolCommand

```rust
pub struct SwitchSchoolCommand {
    pub from_school_id: SchoolId,
    pub to_school_id: SchoolId,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

Atomically swaps the active tenant context. In-flight subscriptions
for the previous school are paused (not aborted); the new school's
subscriptions are resumed (or initialized via a snapshot if first
contact).

### ApplyRemoteChangeCommand

```rust
pub struct ApplyRemoteChangeCommand {
    pub school_id: SchoolId,
    pub envelope: EventEnvelope<Box<dyn DomainEvent>>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

The internal command the pull loop produces for each remote
event. The handler advances the cursor, dispatches to the target
domain's command bus, and writes a `sync_audit` row in the same
transaction. On conflict, it short-circuits to
`ConflictReported` and does not advance.

## Repositories

Sync's storage surface is intentionally small. The engine
**does not** define a "SyncRepository" — the outbox, cursor,
conflict, and subscription tables are owned by `educore-core` and
queried through the existing storage port, so any adapter that
supports the change-feed surface works for sync.

The **port methods** sync adds to the storage adapter are:

### `watch_changes`

```rust
async fn watch_changes(
    &self,
    school_id: SchoolId,
    aggregate_type: AggregateType,
    from: VersionCursor,
) -> Result<ChangeStream>;
```

A long-lived, server-push or client-pull stream of remote events
for the given `(school, aggregate_type)` starting at the given
cursor. Returns a `ChangeStream` that yields
`EventEnvelope<Box<dyn DomainEvent>>` values. The stream is
cancellable and resumable.

### `apply_snapshot`

```rust
async fn apply_snapshot(
    &self,
    snapshot: SchoolSnapshot,
) -> Result<SnapshotReport>;
```

Replaces the local rows for a `(school, aggregate_type)` with the
contents of a full snapshot, in a single transaction. The
`SnapshotReport` returns the row count, the cursor of the
snapshot, and the number of tail events queued for merge.

### `cursor_for`

```rust
async fn cursor_for(
    &self,
    school_id: SchoolId,
    aggregate_type: AggregateType,
    aggregate_id: Uuid,
) -> Result<Option<VersionCursor>>;
```

Returns the current cursor for a specific aggregate, or `None` if
no cursor has been recorded.

### `advance_cursor`

```rust
async fn advance_cursor(
    &self,
    school_id: SchoolId,
    aggregate_type: AggregateType,
    aggregate_id: Uuid,
    to: VersionCursor,
    transaction: &mut Transaction,
) -> Result<()>;
```

Moves the cursor forward, in the same transaction as the remote
event's application. A no-op if `to <= current`. Returns
`Err(SyncError::CursorRegression)` on a regressing call.

## Value Objects

Sync's value objects are small, typed wrappers. They live in
`educore-core` and are re-exported from the sync module.

### `CommandEnvelope`

```rust
pub struct CommandEnvelope {
    pub command_id: Uuid,
    pub command_type: &'static str,
    pub school_id: SchoolId,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub idempotency_key: IdempotencyKey,
    pub issued_at: Timestamp,
    pub payload: serde_json::Value,
}
```

The wire-friendly wrapper around a locally produced command. The
`payload` is the JSON serialization of the domain command; sync
does not inspect it, but the server's command bus will.

### `EventFilter`

```rust
pub struct EventFilter {
    pub aggregate_types: Vec<AggregateType>,
    pub event_types: Vec<&'static str>,
    pub from_version: Option<VersionCursor>,
}
```

The subscription-side filter passed to `watch_changes`. Allows
the client to subscribe to a subset of the change feed (e.g.
"everything in `academic` since yesterday").

### `SchoolSnapshot`

```rust
pub struct SchoolSnapshot {
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub version: VersionCursor,
    pub rows: Vec<SnapshotRow>,
    pub generated_at: Timestamp,
}

pub struct SnapshotRow {
    pub aggregate_id: Uuid,
    pub payload: serde_json::Value,
    pub last_modified: Timestamp,
}
```

The payload of an `apply_snapshot` call. The `version` is the
server-side cursor the snapshot represents; rows in the snapshot
are at-or-before that version.

### `VersionCursor`

```rust
pub struct VersionCursor(pub String);
```

Opaque cursor value. The server returns whatever the change-feed
emits (LSN, event_log id, HLC). The client treats it as an
ordered, comparable string.

### `ConflictId`

```rust
pub struct ConflictId(pub SchoolId, pub Uuid);
```

Typed identifier for a `ConflictRecord`. The `Uuid` is generated
locally when the conflict is detected; it is reported to the
server on resolution so the server can map back to the divergent
events.

### `ConflictResolution`

```rust
pub enum ConflictResolution {
    AcceptLocal,
    AcceptRemote,
    Merge(serde_json::Value),
    DiscardLocal,
}
```

The user's choice. `AcceptLocal` and `DiscardLocal` differ in
audit semantics: `AcceptLocal` re-pushes the local command with
the resolution applied, `DiscardLocal` drops the local command
and accepts the remote state.

## Tables

Sync adds four local tables to the storage schema. The DDL is
emitted by the storage adapter at startup via
`storage.create_schema().await` — the same path as domain tables.

| Table                 | Purpose                                                  | Tenant-scoped |
| --------------------- | -------------------------------------------------------- | -------------- |
| `local_outbox`        | Pending commands awaiting push                           | yes            |
| `sync_cursor`         | Per-aggregate last-applied server version                | yes            |
| `local_conflict_queue`| Open `ConflictRecord`s                                   | yes            |
| `sync_audit`          | Append-only audit of every sync state transition         | yes            |

### `local_outbox`

| Column            | Type            | Notes                                                |
| ----------------- | --------------- | ---------------------------------------------------- |
| `school_id`       | `Uuid`          | Part of PK                                           |
| `outbox_id`       | `Uuid`          | Part of PK; the `OutboxEntryId`                      |
| `command_id`      | `Uuid`          | The command the entry is a result of                 |
| `command_type`    | `VARCHAR(128)`  | The static `COMMAND_TYPE` of the producer            |
| `idempotency_key` | `VARCHAR(64)`   | Server-side dedupe key                               |
| `payload`         | `JSONB` / `TEXT`| The serialized `CommandEnvelope`                     |
| `status`          | `VARCHAR(16)`   | `Pending` \| `InFlight` \| `Acked` \| `Conflict`     |
| `attempt_count`   | `INTEGER`       | Number of push attempts                              |
| `last_error`      | `TEXT`          | Last error message (nullable)                        |
| `next_attempt_at` | `TIMESTAMP`     | Scheduled retry time                                |
| `enqueued_at`     | `TIMESTAMP`     | Local acceptance time                                |
| `acked_at`        | `TIMESTAMP`     | Time of server ack (nullable)                        |

Indexes: `(school_id, status, next_attempt_at)` for the push
loop, `(school_id, idempotency_key)` for dedupe.

### `sync_cursor`

| Column           | Type            | Notes                                              |
| ---------------- | --------------- | -------------------------------------------------- |
| `school_id`      | `Uuid`          | Part of PK                                         |
| `aggregate_type` | `VARCHAR(64)`   | Part of PK                                         |
| `aggregate_id`   | `Uuid`          | Part of PK                                         |
| `version`        | `VARCHAR(128)`  | Opaque `VersionCursor` string                      |
| `updated_at`     | `TIMESTAMP`     | Last advance time                                  |

Indexes: PK only. Lookups are always by full PK.

### `local_conflict_queue`

| Column            | Type            | Notes                                            |
| ----------------- | --------------- | ------------------------------------------------ |
| `school_id`       | `Uuid`          | Part of PK                                       |
| `conflict_id`     | `Uuid`          | Part of PK; the `ConflictId`                     |
| `aggregate_type`  | `VARCHAR(64)`   | The aggregate under conflict                     |
| `aggregate_id`    | `Uuid`          | The specific aggregate under conflict            |
| `conflict_kind`   | `VARCHAR(64)`   | `FieldMismatch` \| `VersionStale` \| `DeletedOnRemote` \| `SchemaMismatch` |
| `local_outbox_id` | `Uuid`          | The local entry that diverged                    |
| `remote_event_id` | `Uuid`          | The server event that diverged                   |
| `local_payload`   | `JSONB` / `TEXT`| Snapshot of the local command payload            |
| `remote_payload`  | `JSONB` / `TEXT`| Snapshot of the remote event payload             |
| `status`          | `VARCHAR(16)`   | `Open` \| `Resolved`                             |
| `opened_at`       | `TIMESTAMP`     | Detection time                                   |
| `resolved_at`     | `TIMESTAMP`     | Resolution time (nullable)                       |

Indexes: `(school_id, status)` for the open-conflict query.

### `sync_audit`

| Column           | Type            | Notes                                            |
| ---------------- | --------------- | ------------------------------------------------ |
| `school_id`      | `Uuid`          | Tenant scope                                     |
| `audit_id`       | `Uuid`          | PK; new uuid per row                             |
| `event_type`     | `VARCHAR(64)`   | The sync event that was written                  |
| `aggregate_type` | `VARCHAR(64)`   | Nullable; set when the event targets an aggregate type |
| `actor_id`       | `Uuid`          | The user that triggered the transition           |
| `correlation_id` | `Uuid`          | The originating correlation                      |
| `payload`        | `JSONB` / `TEXT`| The event payload                                |
| `recorded_at`    | `TIMESTAMP`     | Server-side write time                           |

Indexes: `(school_id, recorded_at DESC)` for the audit view.

## Services

Sync ships two services: the **in-process reference
implementation** that the engine binary uses to drive its own
sync, and the **client-side HTTP adapter** that a worker process
uses to talk to a remote `educore-sync-server`.

### `SyncCoordinator` (in-process reference)

The reference implementation runs in the same process as the
domain runtime. It is the simplest possible sync: no transport,
no server — just the outbox, cursor, and conflict machinery wired
to the local storage adapter.

Responsibilities:

- Owns the per-`(school, aggregate_type)` subscription state.
- Runs the **push loop**: claim pending outbox entries, mark
  them `InFlight`, apply them through the local command bus
  (this is the server-side command bus in the in-process case),
  record the result, emit the appropriate event.
- Runs the **pull loop**: subscribe to local change events
  produced by other replicas or by a co-located server, advance
  cursors, dispatch through the local command bus as
  `ApplyRemoteChangeCommand`.
- Manages snapshot hydration on first contact and on schema
  upgrades.
- Surfaces conflicts to the consumer via the `ConflictReported`
  event.
- Exposes pause / resume / school-switch as command handlers.

The `SyncCoordinator` is **not** the only implementation. It is
the reference; the storage-port-driven machinery is what every
deployment uses. A custom service (e.g. a federation hub) can
substitute its own coordinator as long as it implements the
same port surface.

### `WorkerHttpSyncAdapter` (worker client)

The worker binary (`educore-worker`) runs in a different process
from the server, on the same machine or on a separate node. It
uses the **HTTP transport** to talk to `educore-sync-server`.

Responsibilities:

- Owns the local outbox, cursor, conflict, and subscription
  state on the worker side.
- Pushes local commands to the server's REST endpoint, with
  retries and exponential backoff.
- Opens a long-poll or WebSocket connection for the change
  feed, applies incoming events through the worker's local
  command bus, advances cursors.
- Hydrates snapshots on first contact via the snapshot endpoint.
- Translates HTTP errors (409 Conflict, 410 Gone, 412
  Precondition Failed) into `ConflictReported` events.

The adapter is **purely a transport binding**. All the
bookkeeping logic — outbox ordering, cursor monotonicity,
conflict detection, idempotency, audit — lives in
`educore-core` and is shared with the in-process reference
implementation.

## Permissions

Sync capabilities are defined in `educore-rbac` and checked on
every sync command. They are additive: a user with `Sync.Request`
does not automatically have `Sync.ResolveConflict`.

| Capability               | Granted To                        | Description                                |
| ------------------------ | --------------------------------- | ------------------------------------------ |
| `Sync.Request`           | Any authenticated user with school access | Start a sync session for a school   |
| `Sync.Pause`             | Same as `Sync.Request`            | Pause a sync subscription                  |
| `Sync.Resume`            | Same as `Sync.Request`            | Resume a paused subscription               |
| `Sync.ResolveConflict`   | A user with the **edit** role on the aggregate's domain | Resolve an open conflict        |
| `Sync.SwitchSchool`      | A user with access to **both** schools in their session | Switch the active tenant context |
| `Sync.CompactOutbox`     | Server-side operator role only    | Manually trigger outbox compaction         |

The server enforces the same set of capabilities on the
incoming `CommandEnvelope`. A client cannot bypass RBAC by
sending a `ResolveConflict` for a school the user no longer
has access to — the server returns `403` and the local
`ApplyRemoteChangeCommand` is not produced.

## Workflows

The three paths below cover every state machine in the sync
subsystem. They are documented as ordered steps; the parity
test suite has an end-to-end test per path.

### Happy Path: Local Command → Outbox → Push → Ack

1. The user submits a command (e.g. `AdmitStudent`) through the
   local UI. The local command bus validates it, applies it,
   emits a domain event, and writes the outbox row in the same
   transaction. The user gets a "saved" confirmation.
2. The `SyncCoordinator` push loop sees the new outbox entry.
   It marks the entry `InFlight` and sends the
   `CommandEnvelope` to `educore-sync-server`.
3. The server validates the command against its domain rules
   (which may differ from the client's offline validation),
   accepts it, and returns the new `EventEnvelope` with the
   server-assigned event id.
4. The push loop records the server event id on the outbox
   entry, marks it `Acked`, and emits `OutboxEntryAcked`.
5. The outbox compaction job removes the entry after the
   retention window. The `sync_audit` row is retained.
6. The user sees "synced" in the UI.

The change-feed side is concurrent: the pull loop is reading
remote events for the same school the whole time. If a remote
event arrives for the aggregate the local command just
mutated, the cursor advances past it and the local change
"wins" by virtue of the cursor moving last.

### Recovery Path: Snapshot + Tail on Reconnect

1. The client comes back online after being offline for a long
   time. It has a `SyncCursor` per aggregate it has seen.
2. The `SyncCoordinator` checks the server's "since" API for
   each `(school, aggregate_type)`. The server reports that
   the gap is too large for incremental tail (e.g. the client
   is more than N versions behind, or a schema migration
   happened).
3. The `SyncCoordinator` issues a snapshot request. The server
   returns a `SchoolSnapshot` with the full state at version
   `V`.
4. The client calls `apply_snapshot`. In one transaction,
   the local rows for the aggregate type are replaced with the
   snapshot, and the cursor moves to `V`. `SnapshotHydrated`
   is emitted.
5. The client then opens a tail subscription starting at
   `V`. Events that arrived at the server between snapshot
   generation and tail subscription are queued by the server
   and delivered in order.
6. As each tail event is applied, the cursor advances
   monotonically. `CursorAdvanced` events are emitted.

The snapshot step is exclusive: the local UI sees a "ready"
state only after `SnapshotHydrated` is emitted and the
queued tail events are merged in cursor order.

### Conflict Path: Surface → User Resolves

1. The user mutates an aggregate locally. The outbox row is
   written with the local command payload.
2. Before the push loop fires, a remote change for the same
   aggregate arrives. The pull loop applies it; the local
   domain event is written; the cursor advances.
3. The push loop now sends the local command. The server
   detects a divergence (e.g. `version` mismatch, or a
   server-side policy reject) and returns `409 Conflict`
   with a payload describing both sides.
4. The push loop writes a `ConflictRecord` row with status
   `Open`, marks the outbox entry as `Conflict`, and emits
   `ConflictReported`. **The cursor does not regress**; the
   local aggregate is left in the post-pull state, and the
   user's original command is held in the conflict record.
5. The UI surfaces the conflict. The user picks a
   resolution: `AcceptLocal`, `AcceptRemote`, or `Merge`.
6. The user submits `ResolveConflictCommand`. The handler
   records the resolution as a `ConflictResolved` event,
   updates the outbox entry's status, and either re-pushes
   the local command (with the resolution merged) or accepts
   the remote side and drops the local.
7. The push loop sends the resolution. The server applies
   it, returns the new event id, and the cursor advances.

While a conflict is `Open`, the local runtime refuses to
apply further remote changes for the same aggregate — they
queue in the change feed but are not dispatched. The queue
is released once the conflict resolves.

## Decisions

This section captures the design rationale. Each decision has
a matching ADR in `docs/decisions/`.

### Server-Authoritative

The server is the source of truth for domain state. The client
is a **read replica with a write buffer**, not an equal peer.
Implications:

- Every server-side state change produces an event in the
  server's outbox, and the client sees the event through the
  change feed. The client never produces a "primary" event
  that the server accepts as-is; the client's command is
  re-validated server-side and may be rejected.
- The cursor is **server-issued**. The client does not
  invent versions. This keeps the change feed stable across
  schema upgrades and makes the cursor a safe snapshot point.
- Conflicts are first-class: the server is allowed to say
  "no" to a local command. The engine treats that as a
  normal outcome, not an error.

### In-Process Reference + Worker Binary

The engine ships **two** sync deployments:

- The **in-process reference** (`SyncCoordinator`): a single
  process runs the server, the storage adapter, and the sync
  machinery. This is what `educore-server` uses in single-
  node deployments and what the test suite uses for parity
  tests.
- The **worker binary** (`educore-worker` + `WorkerHttpSync
  Adapter`): the storage adapter lives in one process; the
  engine library runs in another. They talk over the HTTP
  transport.

Both deployments share the same bookkeeping code path
(outbox, cursor, conflict, audit, idempotency). The transport
is the only thing that differs. This means a parity test
written against the in-process reference is also a parity
test for the worker, modulo the transport boundary.

### Sync as a Build Feature

The sync module is gated behind a Cargo feature:

```toml
[features]
sync = ["dep:educore-events", "dep:educore-rbac"]
```

A consumer that does not need sync (an embedded device that
never talks to a server, a server-only deployment) compiles
the engine without it. The sync tables are not emitted by
`storage.create_schema()` unless the `sync` feature is on.
This keeps the engine usable in contexts where sync is
inappropriate (e.g. ephemeral build containers, read-only
reporting replicas).

### Cursor-Atomic Application

The cursor advance and the remote event application happen
in **one transaction**. If the event fails to apply
(validation, RBAC, business rule), the cursor does not move
and the change is retried. This is the engine's
"at-least-once with monotonic checkpoint" property: a
client that crashes mid-apply restarts from the last
cursor, never from "some events applied, some not".

### Local Outbox in the Same Transaction

The outbox row is written in the **same transaction** as
the domain event that records the command's local
acceptance. This is the transactional outbox pattern, and
it is the engine's guarantee that "if the user got a
success response, the outbox has the command". No separate
flush step is needed; the push loop reads rows that are
already committed.

### Conflicts Block, Not Roll Back

A conflict does not roll back the local state. The local
aggregate remains in the post-pull state, and the user's
original command is held in the conflict record for
resolution. This is a deliberate choice: rolling back the
local state would be invisible to the user (the local
command "disappeared") and would introduce a
"phantom-undo" surface. Surfacing the conflict and asking
the user to choose is the engine's contract.

### Per-School, Per-Aggregate-Type Subscriptions

A subscription is keyed by `(school, aggregate_type,
client_id)`, not by user or by device. This lets a single
user with multiple devices run independent subscriptions
(server-side cursors are per-client) and lets a multi-school
user switch contexts without losing state. Pause and resume
are subscription-scoped, never user-scoped.

### No Silent Drops

The engine never silently drops a sync state transition.
A push failure, a pull error, a snapshot abort, and a
conflict all produce an event in `sync_audit` and (where
applicable) a `SubscriptionStateChanged` event. The UI can
subscribe to `sync_audit` to render a connectivity timeline
without polling.

---

## Phase 0 status (per [ADR-018](../../decisions/ADR-018-SyncEngineArchitecture.md))

Per the minimum-viable Phase 0 scope, the sync subsystem
ships with a subset of the spec's full surface:

- **Crates delivered:** `educore-sync` (port) and
  `educore-sync-inprocess` (default impl).
- **Commands shipped (4 of 6):** `SyncStart`, `SyncPause`,
  `SyncResume`, `SyncRequestDelta`. The `SyncAcknowledge`
  command is deferred (the in-process impl acknowledges
  inline in the test path).
- **Events shipped (5 of 7):** `SyncStarted`, `SyncPaused`,
  `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`.
  `SyncConflictDetected` and `SyncStopped` are deferred.
- **Impls shipped:** `InProcessSyncAdapter` only. The
  `educore-sync-http` worker client lands in **Phase 2**
  (alongside `educore-event-bus`); the `educore-sync-null`
  no-op impl lands in **Phase 16** (alongside the testkit).
- **Port surface:** the `SyncAdapter` trait is fully wired
  with object-safety verified. The Phase 0 in-process
  impl dispatches outbox events to registered consumers
  via `tokio::sync::{mpsc, broadcast}`.
- **Ad-hoc envelope types:** the Phase 0 sync impl uses
  its own `SyncEvent` struct, not `educore_events::EventEnvelope`
  (which lands in Phase 2). The refactor to use the envelope
  is a Phase 2 deliverable.

The Phase 0 e2e (`crates/adapters/storage-surrealdb/tests/outbox_e2e.rs`
plus the in-process sync integration test in
`crates/cross-cutting/sync-inprocess/src/lib.rs`) proves the
sync port is plumbed end-to-end alongside storage: an outbox
append is followed by a sync-coordinator fan-out to a
registered in-process consumer.

Hand-off details for the next agent:
[`docs/handoff/PHASE-0-HANDOFF.md`](../../handoff/PHASE-0-HANDOFF.md).
