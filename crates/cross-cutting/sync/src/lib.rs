//! # educore-sync
//!
//! The sync engine port trait, command catalog, and re-exports
//! of the four typed sync events.
//!
//! Per [`ADR-018`], the sync engine is the bridge between the
//! canonical server and edge clients. This crate defines the
//! contract that any `SyncAdapter` implementation must satisfy;
//! the `educore-sync-inprocess` crate ships the default
//! reference implementation for single-process deployments and
//! tests.
//!
//! Scope of this crate (per the build plan, Phase 0 minimum
//! viable + Phase 2 sync refactor):
//!
//! - The [`SyncAdapter`] port trait (object-safe; `Send + Sync`).
//! - The [`SyncCommand`] enum — the command catalog the in-process
//!   impl receives.
//! - The four typed sync events — re-exported from
//!   [`educore_events::sync`] (per Phase 2 Open Question #2: the
//!   ad-hoc `SyncEvent` enum that previously lived here was
//!   replaced by `SyncStarted`, `SyncPaused`, `SyncResumed`,
//!   `SyncStopped`, all implementing `educore_events::DomainEvent`).
//! - The [`SyncHealth`] / [`SyncStatus`] types — the liveness
//!   surface returned by [`SyncAdapter::health`].
//!
//! Consumers of sync events (projections, the audit sink, the
//! UI) subscribe via the [`educore_events::EventBus`] port with
//! `Topic::EventType("sync.session.started")` etc., not via a
//! per-adapter channel. The bus-port contract is the single
//! source of truth for event delivery.
//!
//! The full command/event catalog (six commands and seven events
//! from `docs/specs/sync/overview.md`) is stubbed by [`SyncCommand`]
//! and the four re-exported events in the minimum viable; the
//! remaining variants land in PR 9+ alongside the outbox,
//! cursor, conflict, and subscription aggregates.
//!
//! The wire protocol (`docs/ports/sync.md`) and the `sync-http` and
//! `sync-null` adapters are **deferred** to a later phase. The
//! port trait is intentionally transport-agnostic so the deferred
//! adapters can be added in-tree without breaking consumers.
//!
//! ## Saga / compensating actions
//!
//! Multi-step workflows in the sync engine often span several
//! resources — fetching a remote change, applying it locally,
//! committing to the audit log, and notifying the bus. If any
//! step fails partway through, the engine must undo the prior
//! steps to preserve the at-least-once semantics the engine
//! promises. The [`saga`] module provides the saga pattern:
//! each step declares a forward action and a compensating
//! action, and the [`saga::Saga`] state machine runs them in
//! sequence, invoking compensations in reverse order on
//! failure.
//!
//! [`ADR-018`]: ../../docs/decisions/ADR-018-SyncEngineArchitecture.md

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod command;
mod health;
mod port;

pub mod saga;

pub use command::SyncCommand;
pub use educore_events::sync::{SyncPaused, SyncResumed, SyncStarted, SyncStopped};
pub use health::{SyncHealth, SyncStatus};
pub use port::SyncAdapter;

pub use educore_core::error::{DomainError, Result};
pub use educore_core::ids::SchoolId;
pub use educore_core::value_objects::Timestamp;

/// Prelude of commonly-used types from the sync crate.
///
/// Consumers can opt in with `use educore_sync::prelude::*;` to
/// bring the port trait, the command catalog, the four typed
/// sync events, the health types, and the core engine types they
/// reference into scope in a single import.
pub mod prelude {
    pub use crate::command::SyncCommand;
    pub use crate::health::{SyncHealth, SyncStatus};
    pub use crate::port::SyncAdapter;
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::SchoolId;
    pub use educore_core::value_objects::Timestamp;
    pub use educore_events::sync::{SyncPaused, SyncResumed, SyncStarted, SyncStopped};
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    // Compile-time check that the trait is dyn-compatible. The
    // engine holds `Arc<dyn SyncAdapter>`; if a generic method is
    // ever added to the trait, this assertion will fail to compile
    // and the change will be caught at the trait boundary rather
    // than at the consumer site.
    #[allow(dead_code)]
    fn _assert_object_safe(_t: Box<dyn SyncAdapter + Sync>) {}

    // Smoke test: the prelude re-exports resolve.
    #[test]
    fn prelude_types_resolve() {
        fn _assert_send<T: Send + Sync>() {}
        _assert_send::<SyncCommand>();
        _assert_send::<SyncStarted>();
        _assert_send::<SyncPaused>();
        _assert_send::<SyncResumed>();
        _assert_send::<SyncStopped>();
        _assert_send::<SyncHealth>();
        _assert_send::<SyncStatus>();
        _assert_send::<Box<dyn SyncAdapter + Sync>>();
    }
}
