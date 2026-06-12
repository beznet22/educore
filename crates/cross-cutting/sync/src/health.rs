//! Sync health and status types.
//!
//! [`SyncHealth`] is the value returned by
//! [`crate::SyncAdapter::health`]. It is a snapshot of the
//! adapter's operational state at the moment of the call: the
//! current [`SyncStatus`] and the timestamp of the last emitted
//! event, if any.

use educore_core::value_objects::Timestamp;
use serde::{Deserialize, Serialize};

/// The operational status of the sync adapter.
///
/// Mirrors the four-state subscription machine from
/// `docs/specs/sync/overview.md` § "Subscription" (Idle,
///
/// Paused, Streaming, Errored) at the **adapter** level rather
/// than the per-subscription level. The minimum viable collapses
/// the per-subscription state into the adapter-level
/// `Running` / `Paused` / `Stopped` triple; the full per-school,
/// per-aggregate-type state machine lands alongside the
/// subscription aggregate in a later phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncStatus {
    /// The adapter is actively running. At least one sync
    /// session has been started and has not been stopped.
    Running,
    /// The adapter is paused. All active sessions are retained
    /// and can be resumed without losing cursor position.
    Paused,
    /// The adapter is stopped. No active sessions; `start` is
    /// required to begin syncing.
    Stopped,
}

impl Default for SyncStatus {
    /// Returns [`SyncStatus::Stopped`], the initial state of a
    /// freshly constructed adapter before any command has been
    /// processed.
    fn default() -> Self {
        Self::Stopped
    }
}

impl SyncStatus {
    /// Returns the canonical snake_case wire string for this
    /// status. Storage adapters and event subscribers can use
    /// this to match against a stable, language-agnostic
    /// representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }
}

/// The health state of the sync adapter.
///
/// Returned by [`crate::SyncAdapter::health`]. The struct is a
/// pure data snapshot: it carries no behavior and no locks. The
/// adapter is responsible for producing a consistent view
/// internally; consumers can compare snapshots across calls to
/// detect transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncHealth {
    /// The current operational status.
    pub status: SyncStatus,
    /// The timestamp of the last event the adapter emitted, if
    /// any. `None` for a freshly constructed adapter that has
    /// not yet processed any command.
    pub last_event_at: Option<Timestamp>,
}

impl Default for SyncHealth {
    /// Returns the initial health of a freshly constructed
    /// adapter: [`SyncStatus::Stopped`] and no recorded event.
    fn default() -> Self {
        Self {
            status: SyncStatus::Stopped,
            last_event_at: None,
        }
    }
}
