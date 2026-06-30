# Sync — Ports

The sync subsystem defines two ports: the storage-port
additions sync requires, and the dedicated `SyncAdapter`
port trait. The cross-cutting sync port contracts live
alongside the other port contracts in
[`docs/ports/sync.md`](../../ports/sync.md); this file
documents the per-spec view.

## Storage Port Additions

The engine does **not** define a `SyncRepository`. The
outbox, cursor, conflict, and subscription tables are owned
by `educore-core` and queried through the existing storage
adapter. The port methods sync adds are:

### `watch_changes`

```rust
async fn watch_changes(
    &self,
    school_id: SchoolId,
    aggregate_type: AggregateType,
    from: VersionCursor,
) -> Result<ChangeStream>;
```

A long-lived, server-push or client-pull stream of remote
events for the given `(school, aggregate_type)` starting at
the given cursor. Returns a `ChangeStream` that yields
`EventEnvelope<Box<dyn DomainEvent>>` values. The stream is
cancellable and resumable.

### `apply_snapshot`

```rust
async fn apply_snapshot(
    &self,
    snapshot: SchoolSnapshot,
) -> Result<SnapshotReport>;
```

Replaces the local rows for a `(school, aggregate_type)`
with the contents of a full snapshot, in a single
transaction. The `SnapshotReport` returns the row count, the
cursor of the snapshot, and the number of tail events
queued for merge.

### `cursor_for`

```rust
async fn cursor_for(
    &self,
    school_id: SchoolId,
    aggregate_type: AggregateType,
    aggregate_id: Uuid,
) -> Result<Option<VersionCursor>>;
```

Returns the current cursor for a specific aggregate, or
`None` if no cursor has been recorded.

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

Moves the cursor forward, in the same transaction as the
remote event's application. A no-op if `to <= current`.
Returns `Err(SyncError::CursorRegression)` on a regressing
call.

## SyncAdapter Port Trait

```rust
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    async fn start(&self, school: SchoolId) -> Result<()>;
    async fn pause(&self, school: SchoolId) -> Result<()>;
    async fn resume(&self, school: SchoolId) -> Result<()>;
    async fn stop(&self, school: SchoolId) -> Result<()>;
    async fn health(&self) -> Result<SyncHealth>;
}
```

Per [`ADR-018`](../../decisions/ADR-018-SyncEngineArchitecture.md),
the trait is **transport-agnostic**. The in-process reference
(`educore-sync-inprocess`) and any future HTTP / WebSocket /
IPC adapter all implement the same five methods. The trait
is object-safe; consumers typically hold
`Arc<dyn SyncAdapter>`.

### Method contracts

- `start` is idempotent — calling on an already-running
  school is a no-op. Returns `Err(DomainError::Conflict)` if
  the adapter is in a terminal state that disallows starting.
- `pause` is idempotent — pausing an already-paused school
  is a no-op. The session is retained: a subsequent `resume`
  continues from the last cursor.
- `resume` is idempotent — resuming an already-running school
  is a no-op. Returns `Err(DomainError::NotFound)` if the
  school has no recorded session to resume.
- `stop` is idempotent — stopping an already-stopped school
  is a no-op. The session is removed: a subsequent `start`
  is required to begin syncing again.
- `health` is a snapshot liveness probe. Consumers invoke
  this before opening a subscription or after a transport
  error to decide whether to retry, back off, or surface the
  failure.

## Reference Implementations

| Implementation       | Tier       | Use case                                            |
| -------------------- | ---------- | --------------------------------------------------- |
| `InProcessSyncAdapter` | cross-cutting | Single-process desktop / CLI; the Phase 0 parity target. |
| `WorkerHttpSyncAdapter` (planned) | adapters | Production: client talks to `sync-engine/` worker over HTTPS + WebSocket. |
| `NullSyncAdapter` (planned) | adapters | Tests; single-process demos that do not exercise the network path. |

`educore-sync-inprocess` lives in `crates/cross-cutting/`
because it is a reference implementation for tests, not a
port implementation (per the ADR-018 amendment of 2026-06-25
and the tier rules in [`ADR-013`](../../decisions/ADR-013-CrateLayout.md)).
The HTTP and null adapters land in
`crates/adapters/educore-sync-http` and
`crates/adapters/educore-sync-null` in later phases.
