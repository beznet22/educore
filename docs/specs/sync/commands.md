# Sync — Commands

Sync commands are the entry points for both the local UI and
the internal command bus. They obey the same idempotency, RBAC,
and audit rules as domain commands and are rejected if the
actor lacks the required capability.

Every sync command carries an `IdempotencyKey`. Resubmitting
the same key within the dedupe window returns the prior
result, not a duplicate execution. Every sync command writes
a `sync_audit` row in the same transaction as the state
change.

## RequestSyncCommand

```rust
pub struct RequestSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
}
```

**Capability:** `Sync.Request`
**Effects:** Begins a sync session. Emits `SyncStarted` per
aggregate type and, where no snapshot exists, a
`SnapshotHydrated` followed by a streaming pull.

## PauseSyncCommand

```rust
pub struct PauseSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub reason: Option<String>,
    pub idempotency_key: IdempotencyKey,
}
```

**Capability:** `Sync.Pause`
**Effects:** Stops the local reader. The server-side stream
stays open; the cursor does not move. Emits
`SubscriptionStateChanged` to `Paused`.

## ResumeSyncCommand

```rust
pub struct ResumeSyncCommand {
    pub school_id: SchoolId,
    pub aggregate_types: Vec<AggregateType>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

**Capability:** `Sync.Resume`
**Effects:** Resumes reading from the last cursor. Emits
`SubscriptionStateChanged` to `Streaming`.

## ResolveConflictCommand

```rust
pub struct ResolveConflictCommand {
    pub conflict_id: ConflictId,
    pub resolution: ConflictResolution,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

**Capability:** `Sync.ResolveConflict`
**Effects:** Records the user's choice for an open conflict.
Emits `ConflictResolved`. The local outbox entry that was in
`Conflict` status is re-pushed with the resolution applied (or
discarded if the user accepted the remote side).

## SwitchSchoolCommand

```rust
pub struct SwitchSchoolCommand {
    pub from_school_id: SchoolId,
    pub to_school_id: SchoolId,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

**Capability:** `Sync.SwitchSchool`
**Effects:** Atomically swaps the active tenant context.
In-flight subscriptions for the previous school are paused (not
aborted); the new school's subscriptions are resumed (or
initialized via a snapshot on first contact).

## ApplyRemoteChangeCommand

```rust
pub struct ApplyRemoteChangeCommand {
    pub school_id: SchoolId,
    pub envelope: EventEnvelope<Box<dyn DomainEvent>>,
    pub actor_id: UserId,
    pub idempotency_key: IdempotencyKey,
}
```

**Capability:** `Sync.ApplyRemoteChange`
**Effects:** Internal command the pull loop produces for each
remote event. Advances the cursor and writes a `sync_audit`
row in the same transaction. On conflict, short-circuits to
`ConflictReported` and does not advance.

## Phase 0 minimum viable

Per [`ADR-018`](../../decisions/ADR-018-SyncEngineArchitecture.md),
the Phase 0 `SyncAdapter` port trait ships only the session-
control commands (`Start`, `Pause`, `Resume`, `Stop`); the full
command catalog above is wired in alongside the outbox, cursor,
conflict, and subscription aggregates in later phases. See
[`overview.md`](./overview.md) § "Phase 0 status" for the
current delivery window.
