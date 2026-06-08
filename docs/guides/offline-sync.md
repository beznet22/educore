# Offline Synchronization Guide

## Goal

SMScore works correctly in disconnected environments. A teacher on a
school visit in a remote village, a parent on a phone in a low-signal
area, and a clerk in a temporary office should all be able to operate
the system. State changes are queued locally and reconciled with the
central store on reconnect.

## Offline Mode Triggers

Offline mode is entered when:

- The network connection to the central store is unavailable.
- The consumer's deployment is configured for "field mode" (no
  central store at all).
- The user explicitly toggles "offline mode" (e.g. to save bandwidth).

The engine detects the mode from the storage adapter. The adapter
returns a status of `Online`, `Offline`, or `Degraded` (partial
connectivity).

## Local Storage

The consumer provides a local storage adapter, typically SQLite.
The engine uses the same storage port trait; only the adapter
implementation differs.

```rust
let storage: Arc<dyn StorageAdapter> = match online_status {
    Online => Arc::new(PostgresStorage::connect(central_url).await?),
    Offline => Arc::new(SqliteStorage::open(local_path)?),
};
```

The local database contains the same schema as the central store.
Migrations are run on first use.

## Local Event Bus

The local event bus (in-process) operates normally. Events emitted by
local commands are delivered to local subscribers immediately.

## Outbox Queue

Events destined for the central store are written to a local outbox
table. The outbox relay process (running on the local device) attempts
to publish them to the central event bus when connectivity is
restored.

The outbox also tracks command envelopes (not just events). Each
command issued offline is logged with its `TenantContext`,
`IdempotencyKey`, and `CorrelationId`. On reconnect, the consumer
replays them against the central store.

## Conflict Resolution

Conflicts can occur when:

- Two devices edit the same aggregate while offline.
- A central rule (e.g. a student's roll number) was changed while a
  device was offline.

The engine uses a **version-based** conflict resolution strategy:

- Every aggregate root carries a `version: u64` field.
- Every command carries the expected `version` in its `TenantContext`
  (or in a header).
- The engine refuses to apply a command whose expected version does
  not match the current version (`Conflict::StaleVersion`).

For non-conflicting edits (e.g. two devices editing different fields
of the same student), the engine applies both edits in the order they
arrive. The consumer may implement field-level merging if needed.

## Last-Writer-Wins (LWW)

For fields where LWW is acceptable (e.g. notes, descriptions), the
engine accepts the most recent `updated_at`. The consumer is
responsible for choosing the appropriate strategy per field.

## Command Replay

When connectivity is restored, the consumer replays queued commands:

```rust
for cmd in offline_queue.iter() {
    let result = engine.dispatch(cmd).await;
    match result {
        Ok(outcome) => offline_queue.ack(cmd.id).await?,
        Err(Conflict::StaleVersion) => {
            // Reload the aggregate, reapply the command, retry.
            let fresh = engine.students().get(cmd.target_id).await?;
            cmd.refresh_target(fresh);
            engine.dispatch(cmd).await?;
        }
        Err(e) => {
            // Surface the error to the user; the queue retains the
            // command for manual resolution.
        }
    }
}
```

## Idempotency

All offline commands carry an `IdempotencyKey` derived from the local
command id. When the central store receives the command, it checks
the key against the deduplication log. Duplicate commands return the
original outcome without re-applying.

## Event Replay

On reconnect, the local event bus is reconciled with the central bus:

1. The local outbox's unpublished events are sent to the central bus.
2. The central bus's events that occurred during the offline window
   are pulled and applied to local projections.
3. Conflicts are resolved per the strategy above.

## Local Authentication

Sessions issued offline are valid locally. On reconnect, the local
session is replaced by a central session (the user must re-authenticate
if the local session has expired).

## Local Currency

A school operating offline records payments in cash or cheque. These
records are queued and reconciled on reconnect. The central ledger
treats them as legitimate payments with `Source::Offline` and
`ReconciledAt::<timestamp>`.

## Time Skew

Local devices may have skewed clocks. The engine uses the central
clock (returned by the `Clock` port) for all timestamps. Local
events are timestamped with the local clock at the time of issue but
the central store may re-stamp them with the central clock on
replay. The original local timestamp is preserved in the event
metadata for audit.

## Background Sync

The consumer runs a background task that:

- Detects connectivity changes.
- Replays queued commands.
- Replays queued events.
- Resolves conflicts.
- Reports sync status to the user.

## Worked Example

A mobile field worker app:

```rust
let storage: Arc<dyn StorageAdapter> = Arc::new(
    SqliteStorage::open("/data/school.sqlite")?
);
let audit = Arc::new(LocalAuditSink::new(storage.clone()));
let engine = Engine::builder()
    .storage(storage.clone())
    .audit(audit)
    .event_bus(InProcessBus::new())
    .build().await?;

// Local command works offline.
let student = engine.students().admit(cmd).await?;

// On reconnect, replay queued commands.
let queue = OfflineQueue::load(storage.clone())?;
queue.replay(&engine).await?;
```

## Testing

- Unit tests of offline storage adapter.
- Integration tests of command replay.
- A test of conflict detection and resolution.
- A test of idempotency in replay.
- A test of event reconciliation.
- A test of clock skew handling.
- A test of local authentication.
