//! SQLite-backed bulk insert for `StudentAttendance`.
//!
//! This module is the adapter-level path for the
//! `StorageAdapter::bulk_insert_student_attendances` and
//! `Transaction::bulk_insert_student_attendances` port methods.
//! It implements transaction-grouped multi-row `INSERT`s against
//! the `attendance_student_attendances` table created by
//! `ensure_schema` (called from `SqliteStorageAdapter::migrate`).
//!
//! ## DDL
//!
//! The DDL lives in a sibling `bulk_attendance.sql` file (embedded
//! at compile time) so it can be reviewed alongside the Rust
//! code. The column types follow the spec's per-dialect mapping:
//! `BLOB` for UUIDs (16 bytes big-endian), `TEXT` for string
//! columns, `INTEGER` for counters. The unique index on
//! `(school_id, student_id, attendance_date)` is the uniqueness
//! invariant; violations surface as `DomainError::Conflict`.
//!
//! ## Row cap and batching
//!
//! SQLite caps a single prepared statement at 999 placeholders
//! (the `SQLITE_MAX_VARIABLE_NUMBER` default). 24 columns × 40
//! rows = 960 placeholders, comfortably under the cap. Inputs
//! larger than 40 rows are batched; every batch runs inside a
//! single sqlx `BEGIN IMMEDIATE` / `COMMIT` so a partial
//! failure rolls back all of the batches, not just the failed
//! one. This is the all-or-nothing atomicity the spec requires
//! for the bulk-marking service.

use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::StudentAttendanceRow;

/// The SQLite DDL for the `attendance_student_attendances`
/// table. `include_str!`'d at compile time. The `IF NOT
/// EXISTS` clause makes the migration idempotent; a re-run is
/// a no-op.
pub(crate) const SCHEMA_SQL: &str = include_str!("bulk_attendance.sql");

/// The per-batch row cap. SQLite caps a single prepared
/// statement at 999 placeholders; 24 columns × 40 rows = 960
/// placeholders, leaving 39 spare.
pub(crate) const MAX_ROWS_PER_BATCH: usize = 40;

/// The SQLite-backed `bulk_insert` handle. Wraps a `SqlitePool`
/// and the school the bulk path is scoped to. The
/// [`SqliteStorageAdapter`](crate::storage::SqliteStorageAdapter)
/// and [`SqliteTransaction`](crate::transaction::SqliteTransaction)
/// each construct a `SqliteBulkAttendance` and delegate to
/// the same internal [`bulk_insert_into`] helper.
#[derive(Clone)]
pub struct SqliteBulkAttendance {
    pool: SqlitePool,
    school: SchoolId,
}

impl std::fmt::Debug for SqliteBulkAttendance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteBulkAttendance")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteBulkAttendance {
    /// Constructs a new bulk-insert handle bound to `pool` and
    /// scoped to `school`.
    #[must_use]
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        Self { pool, school }
    }

    /// Returns the school the bulk path is scoped to.
    #[must_use]
    pub fn school(&self) -> SchoolId {
        self.school
    }

    /// Applies the table DDL to bring the
    /// `attendance_student_attendances` table into existence.
    /// Idempotent: a re-run is a no-op. Called from
    /// [`SqliteStorageAdapter::migrate`](crate::storage::SqliteStorageAdapter::migrate)
    /// after the six engine cross-cutting tables are created.
    #[instrument(skip(self))]
    pub async fn ensure_schema(&self) -> Result<()> {
        sqlx::raw_sql(SCHEMA_SQL)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "sqlite bulk_attendance ensure_schema: {e}"
                )))
            })?;
        Ok(())
    }

    /// Bulk-inserts `rows` into the `attendance_student_attendances`
    /// table. Delegates to [`bulk_insert_into`]. The `school_id`
    /// argument is the caller-supplied tenant anchor (either the
    /// `TenantContext::school_id` for the adapter-level call or
    /// the transaction's scoped school for the transaction-level
    /// call); every row's `school_id` is validated against it
    /// before the SQL statement is built.
    #[instrument(skip(self, rows), fields(n = rows.len(), school = %self.school))]
    pub async fn bulk_insert(
        &self,
        school_id: SchoolId,
        rows: &[StudentAttendanceRow],
    ) -> Result<()> {
        bulk_insert_into(&self.pool, school_id, rows).await
    }
}

/// Free-function form of the bulk insert. The
/// [`SqliteBulkAttendance::bulk_insert`] method delegates
/// here; the free function is the canonical implementation
/// (kept separate so callers with their own `SqlitePool` can
/// use it without constructing a `SqliteBulkAttendance`).
///
/// The function opens a real `pool.begin()` transaction, runs
/// every batched `INSERT` inside it, and commits on success.
/// A batched `INSERT` that fails rolls back the whole
/// transaction so the engine never observes a partial bulk
/// insert.
///
/// # Errors
/// - `Validation` if any row's `school_id` does not match
///   `school_id`.
/// - `Conflict` on a unique-key violation of
///   `(school_id, student_id, attendance_date)`.
/// - `Infrastructure` for any underlying storage error.
#[instrument(skip(pool, rows), fields(n = rows.len(), school = %school_id))]
pub async fn bulk_insert_into(
    pool: &SqlitePool,
    school_id: SchoolId,
    rows: &[StudentAttendanceRow],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    for (i, r) in rows.iter().enumerate() {
        if r.school_id != school_id {
            return Err(DomainError::validation(format!(
                "bulk_insert_student_attendances: row {i} school_id mismatch (expected {school_id}, got {})",
                r.school_id
            )));
        }
    }

    // `pool.begin()` acquires a connection from the pool and
    // starts a `BEGIN` transaction. We use the same connection
    // for every batched `INSERT` so a failure in one batch
    // rolls back the work of the prior batches. The whole
    // transaction is committed at the end of this function on
    // success.
    let mut tx = pool.begin().await.map_err(|e| {
        DomainError::infrastructure(crate::error::StringError(format!(
            "sqlite bulk_insert begin: {e}"
        )))
    })?;

    for chunk in rows.chunks(MAX_ROWS_PER_BATCH) {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO attendance_student_attendances (\
                school_id, id, student_id, student_record_id, class_id, section_id, \
                attendance_date, attendance_type, in_time, out_time, notes, is_absent, \
                marked_by, marked_at, marked_from, version, etag, created_at, updated_at, \
                created_by, updated_by, active_status, last_event_id, correlation_id\
             ) VALUES ",
        );
        qb.push_values(chunk.iter(), |mut b, r| {
            b.push_bind(r.school_id_bytes())
                .push_bind(r.id_bytes())
                .push_bind(r.student_id_bytes())
                .push_bind(r.student_record_id_bytes())
                .push_bind(r.class_id_bytes())
                .push_bind(r.section_id_bytes())
                .push_bind(r.attendance_date_string())
                .push_bind(&r.attendance_type)
                .push_bind(&r.in_time)
                .push_bind(&r.out_time)
                .push_bind(&r.notes)
                .push_bind(r.is_absent_value())
                .push_bind(r.marked_by_bytes())
                .push_bind(r.marked_at_string())
                .push_bind(&r.marked_from)
                .push_bind(r.version_value())
                .push_bind(r.etag.as_str())
                .push_bind(r.created_at_string())
                .push_bind(r.updated_at_string())
                .push_bind(r.created_by_bytes())
                .push_bind(r.updated_by_bytes())
                .push_bind(r.active_status_byte())
                .push_bind(r.last_event_id_bytes())
                .push_bind(r.correlation_id_bytes());
        });

        let result = qb.build().execute(&mut *tx).await;
        match result {
            Ok(_) => {}
            Err(sqlx::Error::Database(db))
                if db.kind() == sqlx::error::ErrorKind::UniqueViolation =>
            {
                let _ = tx.rollback().await;
                return Err(DomainError::conflict(
                    "bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row",
                ));
            }
            Err(other) => {
                let _ = tx.rollback().await;
                return Err(DomainError::infrastructure(other));
            }
        }
    }

    tx.commit().await.map_err(|e| {
        DomainError::infrastructure(crate::error::StringError(format!(
            "sqlite bulk_insert commit: {e}"
        )))
    })?;
    Ok(())
}
