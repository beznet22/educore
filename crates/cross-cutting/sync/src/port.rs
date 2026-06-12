//! The [`SyncAdapter`] port trait.
//!
//! This is the engine's only sanctioned entry point for
//! per-school sync session control. Adapters are required to be
//! `Send + Sync` so the engine can drive them from any async
//! runtime, and the trait is object-safe so consumers can hold
//! `Arc<dyn SyncAdapter>`.
//!
//! The five methods are intentionally minimal: start, pause,
//! resume, stop, and a liveness probe. The full surface
//! (`RequestSyncCommand`, `ResolveConflictCommand`,
//! `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`, and the
//! outbox / cursor / conflict / subscription aggregates) lands
//! in later phases. The minimum viable is sufficient for the
//! Phase 0 e2e test and for single-process deployments where
//! the in-process adapter is wired.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use crate::health::SyncHealth;

/// The sync engine port trait.
///
/// Object-safe; consumers typically hold `Arc<dyn SyncAdapter>`.
/// Implementations are required to be `Send + Sync` so the engine
/// can drive them from any async runtime.
///
/// Per `docs/decisions/ADR-018-SyncEngineArchitecture.md`, the
/// trait is transport-agnostic: the in-process reference
/// implementation (`educore-sync-inprocess`) and any future
/// HTTP / WebSocket / IPC adapter all implement the same five
/// methods.
#[async_trait]
pub trait SyncAdapter: Send + Sync {
    /// Begins a sync session for the given school.
    ///
    /// Idempotent: calling `start` on a school that is already
    /// running is a no-op for the in-process adapter and MUST be
    /// a no-op for any adapter that wires this trait. Returns
    /// `Err(DomainError::Conflict)` if the adapter is in a
    /// terminal state that disallows starting.
    async fn start(&self, school: SchoolId) -> Result<()>;

    /// Pauses the sync session for the given school.
    ///
    /// The session is retained: a subsequent [`resume`](Self::resume)
    /// continues from the last cursor. Idempotent: pausing an
    /// already-paused school is a no-op.
    async fn pause(&self, school: SchoolId) -> Result<()>;

    /// Resumes a previously paused sync session for the given
    /// school.
    ///
    /// Idempotent: resuming an already-running school is a
    /// no-op. Returns `Err(DomainError::NotFound)` if the school
    /// has no recorded session to resume.
    async fn resume(&self, school: SchoolId) -> Result<()>;

    /// Stops the sync session for the given school.
    ///
    /// The session is removed: a subsequent [`start`](Self::start)
    /// is required to begin syncing again. Idempotent: stopping
    /// an already-stopped school is a no-op.
    async fn stop(&self, school: SchoolId) -> Result<()>;

    /// Returns the current health of the sync adapter.
    ///
    /// Liveness probe: callers invoke this before opening a
    /// subscription or after a transport error to decide whether
    /// to retry, back off, or surface the failure to the user.
    async fn health(&self) -> Result<SyncHealth>;
}
