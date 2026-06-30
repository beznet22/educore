# Sync — Events

Sync events are domain events: append-only, immutable, and
written to the audit log. They implement the engine's
`DomainEvent` trait and ride the same event bus as business
events.

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

Consumers subscribe via the `educore_events::EventBus` port
with `Topic::EventType("sync.session.started")` etc.; the
bus-port contract is the single source of truth for event
delivery.

## SyncStarted

```rust
pub struct SyncStarted {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub from_version: VersionCursor,
    pub request_id: Uuid,
}
```

**Subscribers:** `educore-rbac` (audit the request and bind the
user); the local `SyncCoordinator` (switch from `Idle` to
`Streaming`).

## SyncPaused

```rust
pub struct SyncPaused {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub reason: Option<String>,
}
```

**Subscribers:** the local UI binding (show paused state); the
local `SyncCoordinator` (stop the reader but keep the cursor).

## SyncResumed

```rust
pub struct SyncResumed {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
    pub from_version: VersionCursor,
}
```

**Subscribers:** the local `SyncCoordinator` (resume the reader
from the last cursor); the local UI binding (clear paused
state).

## SyncStopped

```rust
pub struct SyncStopped {
    pub subscription_id: SyncSubscriptionId,
    pub school_id: SchoolId,
    pub aggregate_type: AggregateType,
}
```

**Subscribers:** the local `SyncCoordinator` (tear down the
reader); the local UI binding (show disconnected state).

## SnapshotHydrated

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

**Subscribers:** the local `SyncCoordinator` (transition to
`Streaming` once the tail is merged); the local UI binding
(show "ready" state).

## ConflictReported

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

**Subscribers:** the local UI binding (surface the conflict);
`educore-audit` (record the open conflict for compliance).

## ConflictResolved

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

**Subscribers:** the local `SyncCoordinator` (re-enqueue the
local outbox entry with the resolution, or accept the remote
side and discard the local); the local UI binding (clear the
conflict surface).

## SubscriptionStateChanged

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

**Subscribers:** the local UI binding (show connectivity
state); the local `SyncCoordinator` (drive the next phase,
e.g. enter `Backoff` after `Errored`).

## Outbox & Cursor Events

The outbox emits one event per status transition:
`OutboxEntryEnqueued`, `OutboxEntryInFlight`, `OutboxEntryAcked`,
`OutboxEntryFailed`, `OutboxEntryConflicted`, `OutboxDrained`.
Cursor advances emit `CursorInitialized` and `CursorAdvanced`.
See [`aggregates.md`](./aggregates.md) for the full mapping.
