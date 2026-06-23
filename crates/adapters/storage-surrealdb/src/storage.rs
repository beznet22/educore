//! SurrealDB-backed `StorageAdapter`.
//!
//! Implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against SurrealDB. Phase 0: in-memory backend only.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::change_stream::{
    ChangeEvent, ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot, VersionCursor,
};
use educore_storage::port::StorageAdapter;
use educore_storage::transaction::Transaction;

use crate::connection::SurrealConnection;
use crate::transaction::SurrealTransaction;

/// The canonical SurrealDB DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time.
const SCHEMA_SQL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.surreal.surql");

/// The current schema version. Bumped on every migration; the
/// adapter's `migrate()` is idempotent thanks to the
/// `IF NOT EXISTS` clauses in the .surql file.
const SCHEMA_VERSION: u32 = 1;

/// The SurrealDB-backed storage adapter.
pub struct SurrealStorageAdapter {
    conn: SurrealConnection,
    closed: std::sync::atomic::AtomicBool,
}

impl std::fmt::Debug for SurrealStorageAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealStorageAdapter")
            .field("school", &self.conn.school())
            .finish_non_exhaustive()
    }
}

impl SurrealStorageAdapter {
    /// Constructs a new adapter from an open connection.
    pub fn new(conn: SurrealConnection) -> Self {
        Self {
            conn,
            closed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Convenience constructor: in-memory connection scoped to
    /// `school`.
    pub async fn in_memory(school: SchoolId) -> Result<Self> {
        let conn = SurrealConnection::in_memory(school).await?;
        Ok(Self::new(conn))
    }

    /// Returns the inner connection.
    pub fn connection(&self) -> &SurrealConnection {
        &self.conn
    }

    /// Returns the inner `Db` handle.
    pub fn db(&self) -> &crate::connection::Db {
        self.conn.db()
    }
}

#[async_trait]
impl StorageAdapter for SurrealStorageAdapter {
    async fn create_schema(&self) -> Result<()> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::infrastructure(crate::error::StringError(
                "StorageAdapter::create_schema called on a closed adapter".to_owned(),
            )));
        }
        // Cluster A stage 3 (surrealdb): delegate to the
        // schema module, which emits the 6 cross-cutting
        // tables and walks the registered aggregates'
        // `EntityDescriptor`s (audit finding ADAPTER-SD-001,
        // closed).
        crate::schema::create_schema(self).await
    }

    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::begin called on a closed adapter",
            ));
        }
        let db = self.conn.db().clone();
        let school = self.conn.school();
        Ok(Box::new(SurrealTransaction::new(db, school)))
    }

    async fn migrate(&self) -> Result<MigrationReport> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::migrate called on a closed adapter",
            ));
        }
        let start = std::time::Instant::now();
        // Execute the canonical schema. The .surql file is
        // idempotent (`IF NOT EXISTS` on every DDL statement).
        self.conn
            .db()
            .query(SCHEMA_SQL)
            .await
            .map_err(DomainError::infrastructure)?;
        Ok(MigrationReport {
            version: SCHEMA_VERSION,
            statements_executed: 0,
            duration: start.elapsed(),
            already_at_version: false,
        })
    }

    async fn ping(&self) -> Result<()> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::ping called on a closed adapter",
            ));
        }
        self.conn
            .db()
            .query("SELECT 1")
            .await
            .map_err(DomainError::infrastructure)?;
        Ok(())
    }

    async fn close(self: Box<Self>) -> Result<()> {
        self.closed.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
        // Phase 0 stub. A future PR will use SurrealDB's
        // `LIVE SELECT` to drive a real change feed.
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::watch_changes called on a closed adapter",
            ));
        }
        let s = futures::stream::empty::<std::result::Result<ChangeEvent, DomainError>>();
        let pinned = Box::pin(s);
        Ok(ChangeStream { inner: pinned })
    }

    async fn apply_snapshot(&self, _snapshot: SchoolSnapshot) -> Result<()> {
        // Phase 0 stub. A future PR will walk the snapshot's
        // `aggregates` and `INSERT` them into the appropriate
        // tables.
        Err(DomainError::not_supported(
            "SurrealStorageAdapter::apply_snapshot is not yet implemented",
        ))
    }

    async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::cursor_for called on a closed adapter",
            ));
        }
        // Phase 0 stub: returns cursor 0. A follow-up PR will
        // compute the cursor from the outbox's `published_at`
        // count.
        Ok(VersionCursor(0))
    }

    async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> {
        if self.closed.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::advance_cursor called on a closed adapter",
            ));
        }
        // Phase 0 stub.
        Ok(())
    }
}

// Suppress unused-import warning for `Arc` and `Duration`
// in this Phase 0 stub; they're reserved for the full impl.
const _: Option<Arc<()>> = None;
const _: Option<Duration> = None;
const _: fn() = || {
    std::mem::drop(futures::stream::empty::<()>().next());
};
