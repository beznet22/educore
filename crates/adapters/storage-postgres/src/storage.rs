//! PostgreSQL-backed `StorageAdapter`.
//!
//! Implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against PostgreSQL 14+ via `sqlx` 0.8. The adapter wraps a
//! [`PostgresConnection`] (a `sqlx::PgPool` + `SchoolId`) and
//! exposes the four sub-port handles (`Outbox`, `AuditLog`,
//! `EventLog`, `Idempotency`) on a [`PostgresTransaction`].
//!
//! ## Schema
//!
//! The canonical DDL for the 6 engine cross-cutting tables is
//! `include_str!`'d at compile time from
//! `migrations/engine/0000_engine_core.postgres.sql`. The file
//! wraps the tables in the `engine` schema; the
//! [`PostgresConnection`](crate::connection::PostgresConnection)
//! sets `search_path = engine, public` on every new connection,
//! so queries can reference `outbox`, `audit_log`, etc.
//! unqualified.
//!
//! In addition to the canonical DDL, `migrate()` also applies the
//! per-tenant `school_id` indexes from
//! [`crate::ddl::SCHOOL_ID_INDEXES_SQL`]. Those indexes are
//! applied as a separate step so the canonical file stays
//! authoritative for the 6 cross-cutting tables themselves, and
//! the adapter-local index additions stay scoped to this crate.
//!
//! ## Migrations
//!
//! `migrate()` executes the DDL via `sqlx::raw_sql`, which
//! handles multi-statement scripts. The DDL is idempotent
//! (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`),
//! so the operation is safe to call repeatedly. The reported
//! `MigrationReport` records the wall-clock duration.
//!
//! ## Sync primitives
//!
//! `watch_changes`, `apply_snapshot`, `cursor_for`, and
//! `advance_cursor` return `DomainError::NotSupported` per the
//! default impls in the `StorageAdapter` trait. A future PR will
//! implement these via `LISTEN/NOTIFY` and a small
//! `sync_state` table.

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

use crate::bulk_attendance::PostgresBulkAttendance;
use crate::connection::PostgresConnection;
use crate::ddl::SCHOOL_ID_INDEXES_SQL;
use crate::transaction::PostgresTransaction;

/// The canonical PostgreSQL DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time.
const SCHEMA_SQL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.postgres.sql");

/// The current schema version. Bumped on every migration; the
/// adapter's `migrate()` is idempotent thanks to the
/// `IF NOT EXISTS` clauses in the .sql file and in
/// [`SCHOOL_ID_INDEXES_SQL`].
///
/// Version history:
/// * `1` — initial schema; 6 cross-cutting tables + indexes
///   from the canonical DDL only.
/// * `2` — QW-6: per-tenant `school_id` indexes for the 4
///   multi-tenant cross-cutting tables (`outbox`, `audit_log`,
///   `idempotency`, `event_log`).
const SCHEMA_VERSION: u32 = 2;

/// The PostgreSQL-backed storage adapter.
pub struct PostgresStorageAdapter {
    conn: PostgresConnection,
    closed: AtomicBool,
}

impl std::fmt::Debug for PostgresStorageAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresStorageAdapter")
            .field("school", &self.conn.school())
            .field("closed", &self.closed.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl PostgresStorageAdapter {
    /// Constructs a new adapter from an open connection.
    #[must_use]
    pub fn new(conn: PostgresConnection) -> Self {
        Self {
            conn,
            closed: AtomicBool::new(false),
        }
    }

    /// Convenience constructor: opens a connection to `url` and
    /// scopes the adapter to `school`. The connection's
    /// `after_connect` hook sets the engine search path.
    ///
    /// # Errors
    /// - `Infrastructure` if the pool cannot reach the database.
    #[instrument(skip(url), fields(school = %school))]
    pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
        let conn = PostgresConnection::connect(url, school).await?;
        Ok(Self::new(conn))
    }

    /// Returns the inner connection.
    pub fn connection(&self) -> &PostgresConnection {
        &self.conn
    }

    /// Returns the inner `sqlx::PgPool` handle.
    pub fn db(&self) -> &sqlx::PgPool {
        self.conn.db()
    }
}

#[async_trait]
impl StorageAdapter for PostgresStorageAdapter {
    #[instrument(skip(self))]
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::begin called on a closed adapter",
            ));
        }
        let pool = self.conn.db().clone();
        let school = self.conn.school();
        Ok(Box::new(PostgresTransaction::new(pool, school)))
    }

    #[instrument(skip(self))]
    async fn migrate(&self) -> Result<MigrationReport> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::migrate called on a closed adapter",
            ));
        }
        let start = Instant::now();
        // `sqlx::raw_sql` handles multi-statement scripts; the
        // DDL is split on `;` boundaries by sqlx. The DDL is
        // idempotent (`CREATE TABLE IF NOT EXISTS` etc.) so a
        // re-run is a no-op.
        sqlx::raw_sql(SCHEMA_SQL)
            .execute(self.conn.db())
            .await
            .map_err(DomainError::infrastructure)?;
        // QW-6: per-tenant school_id indexes for the 4
        // multi-tenant cross-cutting tables. Applied after
        // SCHEMA_SQL so the tables are guaranteed to exist.
        // Idempotent (`CREATE INDEX IF NOT EXISTS`).
        sqlx::raw_sql(SCHOOL_ID_INDEXES_SQL)
            .execute(self.conn.db())
            .await
            .map_err(DomainError::infrastructure)?;
        // The bulk-attendance table is the storage-port
        // target for the Phase 5 bulk-marking service; the
        // DDL is embedded in the `bulk_attendance` module so
        // it lives next to the implementation that owns it.
        PostgresBulkAttendance::new(self.conn.db().clone(), self.conn.school())
            .ensure_schema()
            .await?;
        let duration = start.elapsed();
        // Count statements by counting top-level `;` separators
        // plus one across both the canonical schema and the
        // adapter-local school_id indexes. The number is a
        // coarse lower bound (it includes the `CREATE SCHEMA`
        // and the seed `INSERT`).
        let statements_executed = u32::try_from(
            SCHEMA_SQL
                .split(';')
                .filter(|s| !s.trim().is_empty())
                .count()
                + SCHOOL_ID_INDEXES_SQL
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
    async fn ping(&self) -> Result<()> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "StorageAdapter::ping called on a closed adapter",
            ));
        }
        sqlx::query("SELECT 1")
            .execute(self.conn.db())
            .await
            .map_err(DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn close(self: Box<Self>) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        // sqlx::PgPool's `close` is async and graceful; it
        // returns once all in-flight connections have been
        // returned. We call it on the inner pool, then the
        // outer `Box<Self>` is dropped by the caller.
        self.conn.into_inner().close().await;
        Ok(())
    }

    #[instrument(skip(self, _filter))]
    async fn watch_changes(&self, _filter: ChangeFilter) -> Result<ChangeStream> {
        // Phase 1: not yet implemented. A future PR will use
        // PostgreSQL's `LISTEN outbox_channel` + `pg_notify` to
        // drive a real change feed. We keep the default
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
            "PostgresStorageAdapter::apply_snapshot is not yet implemented (Phase 1)",
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
        let handle = PostgresBulkAttendance::new(self.conn.db().clone(), self.conn.school());
        handle.bulk_insert(ctx.school_id, rows).await
    }
}
