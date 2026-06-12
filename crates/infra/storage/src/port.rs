//! The `StorageAdapter` port — the engine's sole entry point for
//! persistence.
//!
//! Per `docs/ports/storage.md`, the `StorageAdapter` trait is the
//! only sanctioned way for the engine to read or write state.
//! Storage backends (SurrealDB, PostgreSQL, MySQL, SQLite, …) are
//! out-of-tree crates that implement this trait. Consumers wire
//! one adapter into the engine at startup; the rest of the
//! engine never touches a database connection directly.
//!
//! The trait is object-safe: the engine holds
//! `Arc<dyn StorageAdapter>` and dispatches commands against it.

use std::fmt;

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use super::change_stream::{
    ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot, VersionCursor,
};
use super::transaction::Transaction;

/// The engine's sole entry point for persistence.
///
/// Object-safe; consumers typically hold
/// `Arc<dyn StorageAdapter>`. Adapters are required to be
/// `Send + Sync` so the engine can drive them from any
/// async runtime.
#[async_trait]
pub trait StorageAdapter: Send + Sync + fmt::Debug {
    /// Begins a new transaction. Every command runs inside a
    /// transaction; the engine never reads or writes outside
    /// one.
    async fn begin(&self) -> Result<Box<dyn Transaction>>;

    /// Applies the engine's DDL to bring the schema up to the
    /// engine's current version. Idempotent: running on an
    /// already-migrated database returns a no-op report.
    async fn migrate(&self) -> Result<MigrationReport>;

    /// Liveness check. Returns `Ok(())` if the adapter is
    /// connected and responsive; `Err(Infrastructure)` otherwise.
    async fn ping(&self) -> Result<()>;

    /// Closes the adapter, releasing all underlying
    /// connections. After `close`, any further call returns
    /// `Err(Infrastructure)`.
    async fn close(self: Box<Self>) -> Result<()>;

    // --- Sync primitives (ADR-017 / ADR-018) ---
    //
    // The engine's sync engine (gated behind the umbrella's
    // `sync` feature) uses these four methods to:
    //   1. watch local outbox writes and push them to the server
    //   2. bulk-apply a remote SchoolSnapshot for first-time hydration
    //   3. track per-school version cursors for incremental replay
    //   4. advance the cursor after a successful apply
    //
    // Default implementations return `DomainError::NotSupported`.
    // Storage adapters that participate in sync override them
    // (SurrealDB live queries, PG `LISTEN/NOTIFY`, MySQL binlog
    // tail, SQLite polling, …). Adapters that don't override
    // (e.g. consumer-supplied adapters that run in a
    // pure-server topology with no client sync) get the error,
    // which is the correct answer for that topology.

    /// Watches the local outbox for changes matching `filter`
    /// and returns a live `ChangeStream`. The default
    /// implementation returns `NotSupported`; sync-capable
    /// adapters override.
    async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
        let _ = filter;
        Err(educore_core::error::DomainError::not_supported(
            "StorageAdapter::watch_changes is not supported by this adapter",
        ))
    }

    /// Bulk-applies a remote `SchoolSnapshot` to the local
    /// store. The default implementation returns
    /// `NotSupported`.
    async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
        let _ = snapshot;
        Err(educore_core::error::DomainError::not_supported(
            "StorageAdapter::apply_snapshot is not supported by this adapter",
        ))
    }

    /// Returns the current per-school cursor position. The
    /// default implementation returns `NotSupported`.
    async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> {
        let _ = school_id;
        Err(educore_core::error::DomainError::not_supported(
            "StorageAdapter::cursor_for is not supported by this adapter",
        ))
    }

    /// Advances the per-school cursor to `to`. The default
    /// implementation returns `NotSupported`.
    async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> {
        let _ = school_id;
        let _ = to;
        Err(educore_core::error::DomainError::not_supported(
            "StorageAdapter::advance_cursor is not supported by this adapter",
        ))
    }
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

    // Compile-time check that the trait is dyn-compatible.
    // `Box<dyn StorageAdapter>` is used in the engine; if the
    // trait gains a generic method, this assertion will fail to
    // compile.
    fn _assert_object_safe(_t: Box<dyn StorageAdapter + Sync>) {}
}
