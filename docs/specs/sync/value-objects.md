# Sync — Value Objects

Sync value objects are small, typed wrappers. They live in
`educore-core` and are re-exported from the sync module. All
are validated at construction and have no identity (compared
by value).

## Identifiers

| Identifier              | Backing Type                       | Notes                                |
| ----------------------- | ---------------------------------- | ------------------------------------ |
| `OutboxEntryId`         | `(SchoolId, Uuid)`                 | One outbox row                       |
| `SyncCursorKey`         | `(SchoolId, AggregateType, Uuid)`  | The "last applied version" mark      |
| `ConflictId`            | `(SchoolId, Uuid)`                 | One open or resolved conflict        |
| `SyncSubscriptionId`    | `(SchoolId, AggregateType, ClientId)` | One change-feed subscription       |

## Cursor & Version

| Type                 | Constraints                                                       |
| -------------------- | ----------------------------------------------------------------- |
| `VersionCursor`      | Opaque `String`; whatever the server's change-feed emits (LSN, event_log id, HLC). The client treats it as an ordered, comparable string. |
| `AggregateType`      | `&'static str` (e.g. `"academic.student"`). Identifies an aggregate's domain type on the change feed. |
| `ClientId`           | `Uuid` or short string; per-device identifier for cursor scoping. |

## Conflict Surfaces

```rust
pub enum ConflictKind {
    FieldMismatch,
    VersionStale,
    DeletedOnRemote,
    SchemaMismatch,
}

pub enum ConflictResolution {
    AcceptLocal,
    AcceptRemote,
    Merge(serde_json::Value),
    DiscardLocal,
}
```

`AcceptLocal` and `DiscardLocal` differ in audit semantics:
`AcceptLocal` re-pushes the local command with the resolution
applied; `DiscardLocal` drops the local command and accepts the
remote state. `Merge` carries a typed payload the user
constructed in the UI.

## Subscription State

```rust
pub enum SubscriptionState {
    Idle,
    Streaming,
    Backoff,
    Paused,
    Stalled,
}

pub enum SubscriptionStateReason {
    NetworkError,
    Server5xx,
    SchemaDrift,
    UserPause,
    ConflictBlocked,
}
```

## Wire Envelopes

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

pub struct EventFilter {
    pub aggregate_types: Vec<AggregateType>,
    pub event_types: Vec<&'static str>,
    pub from_version: Option<VersionCursor>,
}

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

## Sync Adapter Health

| Type           | Values                                  | Notes                                       |
| -------------- | --------------------------------------- | ------------------------------------------- |
| `SyncStatus`   | `Running`, `Paused`, `Stopped`          | Adapter-level (collapses per-subscription state in Phase 0). |
| `SyncHealth`   | `{ status: SyncStatus, last_event_at: Option<Timestamp> }` | Liveness snapshot returned by `SyncAdapter::health`. |

## Cross-Cutting Bindings

| Type              | Source                  | Notes                          |
| ----------------- | ----------------------- | ------------------------------ |
| `SchoolId`        | `educore-core`          | Tenant scope                   |
| `UserId`          | `educore-platform`      | Actor for capability checks    |
| `IdempotencyKey`  | `educore-core`          | Per-envelope dedupe key        |
| `CorrelationId`   | `educore-platform`      | Trace correlation              |
| `Timestamp`       | `educore-core`          | Event and command timestamps   |
