//! MySQL-backed `StorageAdapter`.
//!
//! Implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against MySQL 8+ via `sqlx` 0.8. The adapter wraps a
//! [`MysqlConnection`] (a `sqlx::MySqlPool` + `SchoolId`) and
//! exposes the four sub-port handles (`Outbox`, `AuditLog`,
//! `EventLog`, `Idempotency`) on a [`MysqlTransaction`].
//!
//! ## Schema
//!
//! The canonical DDL for the 6 engine cross-cutting tables is
//! `include_str!`'d at compile time from
//! `migrations/engine/0000_engine_core.mysql.sql`. The file is
//! MySQL 8+ specific: backtick-quoted identifiers,
//! `ENGINE=InnoDB`, `CHARSET=utf8mb4`,
//! `COLLATE=utf8mb4_unicode_ci`, `JSON` columns (not `JSONB`),
//! and `CHAR(36)` UUID columns. The DDL is wrapped in a
//! `SET FOREIGN_KEY_CHECKS=0` / `=1` pair so the script is
//! idempotent and can be re-run on an already-migrated database.
//!
//! ## Migrations
//!
//! `migrate()` executes the DDL via `sqlx::raw_sql`, which
//! requires `multi_statements=true` on the connection (enabled
//! in [`MysqlConnection::connect`]). The DDL is idempotent
//! (`CREATE TABLE IF NOT EXISTS`, `INSERT IGNORE`, etc.), so
//! the operation is safe to call repeatedly. The reported
//! `MigrationReport` records the wall-clock duration.
//!
//! ## Sync primitives
//!
//! `watch_changes`, `apply_snapshot`, `cursor_for`, and
//! `advance_cursor` return `DomainError::NotSupported` per the
//! default impls in the `StorageAdapter` trait. A future PR
//! could implement `watch_changes` by polling the
//! `enqueued_at` column of the `outbox` table on a short
//! interval, or by tailing the MySQL binlog; both are deferred
//! to Phase 2+.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use async_trait::async_trait;
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;
use educore_storage::change_stream::{
    ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot, VersionCursor,
};
use educore_storage::port::StorageAdapter;
use educore_storage::transaction::Transaction;
use educore_storage::StudentAttendanceRow;

use crate::bulk_attendance::MysqlBulkAttendance;
use crate::connection::MysqlConnection;
use crate::transaction::MysqlTransaction;

/// The canonical MySQL DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time.
const SCHEMA_SQL: &str = include_str!("../../../../migrations/engine/0000_engine_core.mysql.sql");

/// The current schema version. Bumped on every migration; the
/// adapter's `migrate()` is idempotent thanks to the
/// `IF NOT EXISTS` clauses (and the `SET FOREIGN_KEY_CHECKS=0`
/// wrapper) in the .sql file.
const SCHEMA_VERSION: u32 = 1;

/// The MySQL-backed storage adapter.
pub struct MysqlStorageAdapter {
    conn: MysqlConnection,
    closed: AtomicBool,
}

impl std::fmt::Debug for MysqlStorageAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MysqlStorageAdapter")
            .field("school", &self.conn.school())
            .field("closed", &self.closed.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl MysqlStorageAdapter {
    /// Constructs a new adapter from an open connection.
    #[must_use]
    pub fn new(conn: MysqlConnection) -> Self {
        Self {
            conn,
            closed: AtomicBool::new(false),
        }
    }

    /// Convenience constructor: opens a connection to `url` and
    /// scopes the adapter to `school`. The connection's
    /// `after_connect` hook issues `SET NAMES utf8mb4
    /// COLLATE utf8mb4_unicode_ci`.
    ///
    /// # Errors
    /// - `Infrastructure` if the pool cannot reach the database.
    #[instrument(skip(url), fields(school = %school))]
    pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
        let conn = MysqlConnection::connect(url, school).await?;
        Ok(Self::new(conn))
    }

    /// Returns the inner connection.
    pub fn connection(&self) -> &MysqlConnection {
        &self.conn
    }

    /// Returns the inner `sqlx::MySqlPool` handle.
    pub fn db(&self) -> &sqlx::MySqlPool {
        self.conn.db()
    }
}

#[async_trait]
impl StorageAdapter for MysqlStorageAdapter {
    #[instrument(skip(self))]
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::begin called on a closed adapter",
            ));
        }
        let pool = self.conn.db().clone();
        let school = self.conn.school();
        Ok(Box::new(MysqlTransaction::new(pool, school)))
    }

    #[instrument(skip(self))]
    async fn migrate(&self) -> Result<MigrationReport> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::migrate called on a closed adapter",
            ));
        }
        let start = Instant::now();
        // `sqlx::raw_sql` requires the connection to have
        // `multi_statements=true` (enabled by
        // `MysqlConnection::connect`). The DDL is split on `;`
        // boundaries by sqlx. The DDL is idempotent
        // (`CREATE TABLE IF NOT EXISTS`, `INSERT IGNORE`, etc.)
        // and wrapped in `SET FOREIGN_KEY_CHECKS=0` / `=1` so a
        // re-run is a no-op.
        sqlx::raw_sql(SCHEMA_SQL)
            .execute(self.conn.db())
            .await
            .map_err(DomainError::infrastructure)?;
        // The bulk-attendance table is the storage-port
        // target for the Phase 5 bulk-marking service; the
        // DDL is embedded in the `bulk_attendance` module so
        // it lives next to the implementation that owns it.
        MysqlBulkAttendance::new(self.conn.db().clone(), self.conn.school())
            .ensure_schema()
            .await?;
        let duration = start.elapsed();
        // Count statements by counting top-level `;` separators
        // plus one. The number is a coarse lower bound (it
        // includes the six `CREATE TABLE`s, the indexes, the
        // `SET FOREIGN_KEY_CHECKS=0/1` pair, and the seed
        // `INSERT IGNORE`).
        let statements_executed = u32::try_from(
            SCHEMA_SQL
                .split(';')
                .filter(|s| !s.trim().is_empty())
                .count(),
        )
        .unwrap_or(0);
        Ok(MigrationReport {
            version: SCHEMA_VERSION,
            statements_executed,
            duration,
            already_at_version: false,
        })
    }

    #[instrument(skip(self))]
    async fn create_schema(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::create_schema called on a closed adapter",
            ));
        }
        crate::schema::create_schema(self).await
    }

    #[instrument(skip(self))]
    async fn ping(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::ping called on a closed adapter",
            ));
        }
        sqlx::query::<sqlx::MySql>("SELECT 1")
            .execute(self.conn.db())
            .await
            .map_err(DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn close(self: Box<Self>) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        // sqlx::MySqlPool's `close` is async and graceful; it
        // returns once all in-flight connections have been
        // returned. We call it on the inner pool, then the
        // outer `Box<Self>` is dropped by the caller.
        self.conn.into_inner().close().await;
        Ok(())
    }

    #[instrument(skip(self, _filter))]
    async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
        // Phase 1: not yet implemented. A future PR could
        // implement this by polling the `outbox.enqueued_at`
        // column on a short interval, or by tailing the MySQL
        // binlog for `outbox` writes. We keep the default
        // `NotSupported` behaviour by simply constructing the
        // `ChangeStream` shell and returning it; the default
        // impl in the trait already does this. For clarity we
        // spell it out so future readers can find the entry
        // point.
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::watch_changes called on a closed adapter",
            ));
        }
        let s = futures::stream::empty::<
            std::result::Result<educore_storage::change_stream::ChangeEvent, DomainError>,
        >();
        let pinned = Box::pin(s);
        Ok(ChangeStream { inner: pinned })
    }

    #[instrument(skip(self, _snapshot))]
    async fn apply_snapshot(&self, _snapshot: SchoolSnapshot) -> Result<()> {
        Err(DomainError::not_supported(
            "MysqlStorageAdapter::apply_snapshot is not yet implemented (Phase 1)",
        ))
    }

    #[instrument(skip(self, _school_id))]
    async fn cursor_for(&self, _school_id: SchoolId) -> Result<VersionCursor> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::cursor_for called on a closed adapter",
            ));
        }
        // Phase 1 stub: returns cursor 0. A follow-up PR will
        // compute the cursor from a `sync_state` table.
        Ok(VersionCursor::ZERO)
    }

    #[instrument(skip(self, _school_id, _to))]
    async fn advance_cursor(&self, _school_id: SchoolId, _to: VersionCursor) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::advance_cursor called on a closed adapter",
            ));
        }
        // Phase 1 stub.
        Ok(())
    }

    #[instrument(skip(self, ctx, rows), fields(n = rows.len(), school = %ctx.school_id))]
    async fn bulk_insert_student_attendances(
        &self,
        ctx: &TenantContext,
        rows: &[StudentAttendanceRow],
    ) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::bulk_insert_student_attendances called on a closed adapter",
            ));
        }
        let handle = MysqlBulkAttendance::new(self.conn.db().clone(), self.conn.school());
        handle.bulk_insert(ctx.school_id, rows).await
    }
}
