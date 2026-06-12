//! SQLite-backed `StorageAdapter`.
//!
//! Implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against SQLite 3.x. The DDL is `include_str!`'d from the
//! engine's canonical migration file so the schema stays in
//! lockstep with the other SQL adapters.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sqlx::Executor;
use sqlx::SqlitePool;
use tracing::debug;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::change_stream::{
    ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot, VersionCursor,
};
use educore_storage::port::StorageAdapter;
use educore_storage::transaction::Transaction;

use crate::connection::SqliteConnection;
use crate::transaction::SqliteTransaction;

/// The canonical SQLite DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time from the engine
/// migration file (per
/// `docs/schemas/sql-dialects/sqlite.md#the-6-engine-cross-cutting-tables--sqlite-ddl`).
const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.sqlite.sql");

/// The current schema version. Bumped on every migration; the
/// adapter's `migrate()` is idempotent thanks to the
/// `IF NOT EXISTS` clauses in the .sql file.
const SCHEMA_VERSION: u32 = 1;

/// The SQLite-backed storage adapter.
pub struct SqliteStorageAdapter {
    conn: SqliteConnection,
    closed: AtomicBool,
}

impl fmt::Debug for SqliteStorageAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteStorageAdapter")
            .field("school", &self.conn.school())
            .finish_non_exhaustive()
    }
}

impl SqliteStorageAdapter {
    /// Constructs a new adapter from an open connection.
    pub fn new(conn: SqliteConnection) -> Self {
        Self {
            conn,
            closed: AtomicBool::new(false),
        }
    }

    /// Convenience constructor: in-memory connection scoped to
    /// `school`. The pool is single-connection (see
    /// [`SqliteConnection::in_memory`]).
    pub async fn in_memory(school: SchoolId) -> Result<Self> {
        let conn = SqliteConnection::in_memory(school).await?;
        Ok(Self::new(conn))
    }

    /// Returns the inner connection.
    pub fn connection(&self) -> &SqliteConnection {
        &self.conn
    }

    /// Returns the inner `SqlitePool` handle.
    pub fn pool(&self) -> &SqlitePool {
        self.conn.db()
    }
}

#[async_trait]
impl StorageAdapter for SqliteStorageAdapter {
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::begin called on a closed adapter",
            ));
        }
        let pool = self.conn.db().clone();
        let school = self.conn.school();
        Ok(Box::new(SqliteTransaction::new(pool, school)))
    }

    async fn migrate(&self) -> Result<MigrationReport> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::migrate called on a closed adapter",
            ));
        }
        let start = std::time::Instant::now();
        // `sqlx::raw_sql` handles multi-statement scripts
        // (every DDL in SCHEMA_SQL is a `CREATE TABLE IF NOT
        // EXISTS` / `CREATE INDEX IF NOT EXISTS` so the
        // migration is idempotent).
        let result = self
            .pool()
            .execute(sqlx::raw_sql(SCHEMA_SQL))
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "sqlite migrate: {e}"
                )))
            })?;
        let _ = result; // result.rows_affected() is uninteresting for raw_sql
        let duration = start.elapsed();
        debug!(?duration, version = SCHEMA_VERSION, "sqlite migrate done");
        Ok(MigrationReport {
            version: SCHEMA_VERSION,
            statements_executed: 0,
            duration,
            already_at_version: false,
        })
    }

    async fn ping(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::ping called on a closed adapter",
            ));
        }
        sqlx::query::<sqlx::Sqlite>("SELECT 1")
            .execute(self.pool())
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!("sqlite ping: {e}")))
            })?;
        Ok(())
    }

    async fn close(self: Box<Self>) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
        // Phase 1 stub. A future PR will poll the outbox on a
        // timer (per `docs/ports/storage.md` "MySQL / SQLite:
        // poll the outbox table on a timer").
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::watch_changes called on a closed adapter",
            ));
        }
        Err(DomainError::not_supported(
            "SqliteStorageAdapter::watch_changes is not yet implemented",
        ))
    }

    async fn apply_snapshot(&self, _snapshot: SchoolSnapshot) -> Result<()> {
        Err(DomainError::not_supported(
            "SqliteStorageAdapter::apply_snapshot is not yet implemented",
        ))
    }

    async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::cursor_for called on a closed adapter",
            ));
        }
        // Phase 1 stub: returns cursor 0. A follow-up PR will
        // compute the cursor from the outbox's `published_at`
        // count.
        Ok(VersionCursor::ZERO)
    }

    async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::advance_cursor called on a closed adapter",
            ));
        }
        // Phase 1 stub.
        Ok(())
    }
}
