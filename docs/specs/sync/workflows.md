# Sync — Workflows

The three paths below cover every state machine in the sync
subsystem. They are documented as ordered steps; the parity
test suite has an end-to-end test per path.

## Happy Path: Local Command → Outbox → Push → Ack

```text
1. The user submits a command (e.g. AdmitStudent) through the
   local UI. The local command bus validates it, applies it,
   emits a domain event, and writes the outbox row in the same
   transaction. The user gets a "saved" confirmation.
2. The SyncCoordinator push loop sees the new outbox entry. It
   marks the entry InFlight and sends the CommandEnvelope to
   educore-sync-server.
3. The server validates the command against its domain rules
   (which may differ from the client's offline validation),
   accepts it, and returns the new EventEnvelope with the
   server-assigned event id.
4. The push loop records the server event id on the outbox
   entry, marks it Acked, and emits OutboxEntryAcked.
5. The outbox compaction job removes the entry after the
   retention window. The sync_audit row is retained.
6. The user sees "synced" in the UI.
```

The change-feed side is concurrent: the pull loop is reading
remote events for the same school the whole time. If a remote
event arrives for the aggregate the local command just
mutated, the cursor advances past it and the local change
"wins" by virtue of the cursor moving last.

## Recovery Path: Snapshot + Tail on Reconnect

```text
1. The client comes back online after being offline for a long
   time. It has a SyncCursor per aggregate it has seen.
2. The SyncCoordinator checks the server's "since" API for each
   (school, aggregate_type). The server reports that the gap
   is too large for incremental tail (e.g. the client is more
   than N versions behind, or a schema migration happened).
3. The SyncCoordinator issues a snapshot request. The server
   returns a SchoolSnapshot with the full state at version V.
4. The client calls apply_snapshot. In one transaction, the
   local rows for the aggregate type are replaced with the
   snapshot, and the cursor moves to V. SnapshotHydrated is
   emitted.
5. The client then opens a tail subscription starting at V.
   Events that arrived at the server between snapshot
   generation and tail subscription are queued by the server
   and delivered in order.
6. As each tail event is applied, the cursor advances
   monotonically. CursorAdvanced events are emitted.
```

The snapshot step is exclusive: the local UI sees a "ready"
state only after `SnapshotHydrated` is emitted and the queued
tail events are merged in cursor order.

## Conflict Path: Surface → User Resolves

```text
1. The user mutates an aggregate locally. The outbox row is
   written with the local command payload.
2. Before the push loop fires, a remote change for the same
   aggregate arrives. The pull loop applies it; the local
   domain event is written; the cursor advances.
3. The push loop now sends the local command. The server
   detects a divergence (e.g. version mismatch, or a
   server-side policy reject) and returns 409 Conflict with a
   payload describing both sides.
4. The push loop writes a ConflictRecord row with status Open,
   marks the outbox entry as Conflict, and emits
   ConflictReported. The cursor does not regress; the local
   aggregate is left in the post-pull state, and the user's
   original command is held in the conflict record.
5. The UI surfaces the conflict. The user picks a resolution:
   AcceptLocal, AcceptRemote, or Merge.
6. The user submits ResolveConflictCommand. The handler
   records the resolution as a ConflictResolved event, updates
   the outbox entry's status, and either re-pushes the local
   command (with the resolution merged) or accepts the remote
   side and drops the local.
7. The push loop sends the resolution. The server applies it,
   returns the new event id, and the cursor advances.
```

While a conflict is `Open`, the local runtime refuses to
apply further remote changes for the same aggregate — they
queue in the change feed but are not dispatched. The queue is
released once the conflict resolves.

## Saga-Backed Multi-Step Apply

The "apply remote change" workflow is wrapped in a
[`Saga<S>`](../../cross-cutting/sync/src/saga.rs) so a
partial failure rolls back cleanly. Steps:

```text
1. resolve_event — fetch the remote event envelope.
2. apply_event — dispatch through the local command bus.
3. advance_cursor — move the per-aggregate cursor.
4. write_audit — record the sync_audit row.
5. notify_bus — publish the typed event.
```

If step 2 fails the saga compensates step 1 (drop the in-
memory reference). If step 3 fails it compensates steps 1
and 2. If step 4 fails it compensates steps 1, 2, 3 (cursor
advance is undone, the event is un-applied). If step 5
fails it compensates 1-4 but tolerates a notify failure (the
audit row is the durable record).
