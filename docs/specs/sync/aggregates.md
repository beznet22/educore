# Sync — Aggregates

The sync subsystem is a cross-cutting concern (per
[`overview.md`](./overview.md)). It owns the **bookkeeping**
aggregates that record what was sent, what was received, where
each side stopped reading, and what the user must resolve by
hand. These are not business aggregates; they sit alongside the
domain aggregates and obey the same tenant and audit rules.

| Aggregate          | Root Type          | Purpose                                            |
| ------------------ | ------------------ | -------------------------------------------------- |
| `OutboxEntry`      | `OutboxEntry`      | A locally produced command waiting to be pushed   |
| `SyncCursor`       | `SyncCursor`       | A per-aggregate "last applied server version" mark |
| `ConflictRecord`   | `ConflictRecord`   | A divergence between local and server state        |
| `SyncSubscription` | `SyncSubscription` | A per-school, per-aggregate-type change-feed state |

## OutboxEntry

**Root type:** `OutboxEntry`
**Identity:** `OutboxEntryId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

Represents a domain command that the local runtime has accepted
and is committed to push to the server. The outbox is the
durable bridge between "command accepted" and "server has the
change".

**Invariants:** Status transitions are `Pending → InFlight →
Acked`; `InFlight` may return to `Pending` (network failure) or
move to `Conflict` (server 409). The outbox row is written in
the same transaction as the domain event that records the
command's local acceptance (transactional outbox pattern).

**Commands:** `EnqueueOutboxEntry`, `MarkOutboxEntryInFlight`,
`MarkOutboxEntryAcked`, `MarkOutboxEntryFailed`,
`MarkOutboxEntryConflicted`, `CompactOutboxEntries`.

## SyncCursor

**Root type:** `SyncCursor`
**Identity:** `SyncCursorKey { school_id, aggregate_type, aggregate_id }`
**Tenant:** `SchoolId`

Marks the last version of a specific aggregate that the local
runtime has applied from the server. It is the "where did I
stop reading" mark used to resume a change feed.

**Invariants:** Monotonically non-decreasing per key. The
`advance_cursor` repository method is a no-op if the new version
is `<=` the current one. The cursor advances **in the same
transaction** as the remote event's application — if the event
fails to apply, the cursor does not move.

**Commands:** `InitializeCursor`, `AdvanceCursor`.

## ConflictRecord

**Root type:** `ConflictRecord`
**Identity:** `ConflictId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

Captures a divergence between a local change and a server
change that the engine could not reconcile automatically. The
record is the durable surface that the UI presents to the user;
the resolution is a new event, not an edit to the record.

**Invariants:** Append-only. Status transitions `Open →
Resolved`; there is no re-opening. The record carries both
sides of the conflict (local command payload and remote event
payload) plus a typed `ConflictKind` (field-level,
version-stale, deleted-on-remote, schema-mismatch). While a
conflict is `Open`, the local runtime refuses to apply further
remote changes for the same aggregate — they queue in the
change feed until the conflict resolves.

**Commands:** `OpenConflictRecord`, `ResolveConflict`.

## SyncSubscription

**Root type:** `SyncSubscription`
**Identity:** `SyncSubscriptionId { school_id, aggregate_type, client_id }`
**Tenant:** `SchoolId`

Tracks the state of a per-aggregate-type change feed for a
single client. A subscription is the unit of pause / resume,
retry, and error accounting.

**Invariants:** States are `Idle`, `Streaming`, `Backoff`,
`Paused`, or `Stalled`. A subscription is per-client (not
per-user); a second device for the same user holds its own
subscription. Pause keeps the server-side stream alive; the
client simply stops reading.

**Commands:** `StartSubscription`, `PauseSubscription`,
`ResumeSubscription`, `ReportSubscriptionError`,
`MarkSubscriptionStreaming`, `MarkSubscriptionStalled`.

## Consistency Boundaries

All outbox mutations are serialized per
`(school_id, command_id)`. Two clients with the same
`IdempotencyKey` collapse to one entry. Cursor mutations are
atomic with the application of the remote event. Subscription
state is local to the client — the server does not see pause /
resume transitions; it only sees the cursor. See
[`ports.md`](./ports.md) for the storage-port additions
(`watch_changes`, `apply_snapshot`, `cursor_for`,
`advance_cursor`).
