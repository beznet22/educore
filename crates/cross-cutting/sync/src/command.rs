//! The sync command catalog.
//!
//! The minimum viable defines the four session-control commands
//! (`Start`, `Pause`, `Resume`, `Stop`). The full catalog from
//! `docs/specs/sync/overview.md` § "Commands" —
//! `RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`,
//! `ResolveConflictCommand`, `SwitchSchoolCommand`, and
//! `ApplyRemoteChangeCommand` — is stubbed by these four in the
//! minimum viable; the remaining variants land in later phases
//! alongside the outbox, conflict, and subscription aggregates.

use educore_core::ids::SchoolId;
use serde::{Deserialize, Serialize};

/// Commands that can be sent to a [`SyncAdapter`].
///
/// Each variant targets a single school. The enum is `Copy` so
/// the four variants can be passed through channels and stored
/// in queues without allocation. The full catalog in
/// `docs/specs/sync/overview.md` carries additional fields
/// (aggregate types, actor, idempotency key, correlation id);
/// those land alongside the respective command handlers in a
/// later phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncCommand {
    /// Begin a sync session for the given school. Resolves to a
    /// [`SyncStarted`](crate::SyncStarted) event on the bus.
    Start(SchoolId),
    /// Pause the sync session for the given school. Resolves to
    /// a [`SyncPaused`](crate::SyncPaused) event on the bus.
    Pause(SchoolId),
    /// Resume a previously paused sync session for the given
    /// school. Resolves to a [`SyncResumed`](crate::SyncResumed)
    /// event on the bus.
    Resume(SchoolId),
    /// Stop the sync session for the given school. Resolves to a
    /// [`SyncStopped`](crate::SyncStopped) event on the bus.
    Stop(SchoolId),
}
