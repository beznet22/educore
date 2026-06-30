# Sync — Errors

Sync errors fall into three categories: the typed saga error
(used by the saga / compensating-action machinery), the
storage-port errors that propagate from the adapter, and the
domain-error variants the `SyncAdapter` port surfaces.

## SagaError

The saga library returns a typed error from each step and
each compensation. Distinct from `educore_core::DomainError`
so the saga library stays independent of the engine's domain
error machinery.

```rust
pub struct SagaError {
    step: Option<&'static str>,
    message: String,
}

impl SagaError {
    pub fn new(message: impl Into<String>) -> Self;
    pub fn at_step(self, step: &'static str) -> Self;
    pub fn step(&self) -> Option<&'static str>;
    pub fn message(&self) -> &str;
}
```

Carries the name of the step that produced the error (when
known) and a human-readable message. Implements
`std::error::Error` and `Display`. The saga library's
`SagaResult::Failed` variant carries the `failed_at` step
name, the error message, and the list of compensations that
did succeed (so the caller can decide whether to retry,
surface, or quarantine the workflow).

## DomainError Variants Used by Sync

The `SyncAdapter` port trait propagates `Result<T, DomainError>`
from `educore_core`. The variants sync exercises are:

| Variant                       | When it fires                                          |
| ----------------------------- | ------------------------------------------------------ |
| `DomainError::NotFound`       | `resume` on a school with no recorded session.         |
| `DomainError::Conflict`       | `start` on an adapter in a terminal state; server 409 translated to a `ConflictRecord`. |
| `DomainError::Validation`     | Schema version mismatch on the change feed; payload failed RBAC; cursor regression detected. |
| `DomainError::Unavailable`    | Transport unreachable; subscription stream closed by the server with a non-recoverable code (`4001`, `4003`). |
| `DomainError::Timeout`        | Health probe exceeded the configured latency budget.   |

## Storage-Port Error Variants

The four storage-port methods sync adds (`watch_changes`,
`apply_snapshot`, `cursor_for`, `advance_cursor`) inherit
the storage adapter's error surface. The sync-specific
behaviour layered on top is:

- `advance_cursor` returns `Err(SyncError::CursorRegression)`
  on a regressing call (`to <= current`); the storage
  adapter surfaces this as `DomainError::Validation` and
  records a `sync_audit` row.
- `apply_snapshot` returns `Err(SyncError::SnapshotMismatch)`
  when the snapshot's `school_id` differs from the active
  tenant context.
- `watch_changes` returns `Err(SyncError::SubscriptionClosed)`
  when the underlying stream closes (the caller reconnects
  from the last acked version).

## Conflict Resolution Errors

The conflict resolution path can return:

| Error                              | Meaning                                              |
| ---------------------------------- | ---------------------------------------------------- |
| `SyncError::ConflictNotFound`      | The `ConflictRecord` referenced by `ResolveConflictCommand` does not exist or was already resolved. |
| `SyncError::ConflictAlreadyOpen`   | A new conflict was reported for an aggregate that already has an `Open` `ConflictRecord`. |
| `SyncError::UnsupportedResolution` | The server rejected the resolution strategy (`Merge` payloads must round-trip through the server's validator). |

## Audit Surface

Every error path writes a `sync_audit` row before returning,
with `event_type = "sync.error"`, the error variant, the
school_id, the aggregate_id (when applicable), and the
correlation_id of the originating command. The audit row is
the durable record for postmortems; the in-process error is
the user-facing surface.
