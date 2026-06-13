//! PostgreSQL-backed bulk insert for `StudentAttendance`.
//!
//! This module is the adapter-level path for the
//! `StorageAdapter::bulk_insert_student_attendances` and
//! `Transaction::bulk_insert_student_attendances` port methods.
//! It implements a single multi-row `INSERT` against the
//! `attendance_student_attendances` table created by
//! `ensure_schema` (called from `PostgresStorageAdapter::migrate`).
//!
//! ## DDL
//!
//! The DDL lives in a sibling `bulk_attendance.sql` file (embedded
//! at compile time) so it can be reviewed alongside the Rust
//! code. The column types follow the spec's per-dialect mapping:
//! `BYTEA` for UUIDs, `TEXT` for string columns, `INTEGER` for
//! counters. The unique index on
//! `(school_id, student_id, attendance_date)` is the
//! uniqueness invariant; violations surface as
//! `DomainError::Conflict`.
//!
//! ## Row cap
//!
//! PostgreSQL caps a single statement at 65,535 placeholders.
//! The row is 24 columns wide, so the cap is 1,000 rows per
//! call. The bulk-marking service batches larger inputs in
//! chunks of 1,000.

use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::StudentAttendanceRow;

/// The PostgreSQL DDL for the `attendance_student_attendances`
/// table. `include_str!`'d at compile time. The `IF NOT
/// EXISTS` clause makes the migration idempotent; a re-run is
/// a no-op.
pub(crate) const SCHEMA_SQL: &str = include_str!("bulk_attendance.sql");

/// The per-call row cap. PostgreSQL caps a single prepared
/// statement at 65,535 placeholders; 24 columns × 1,000 rows
/// = 24,000 placeholders (well under the cap).
pub(crate) const MAX_ROWS_PER_CALL: usize = 1000;

/// The PostgreSQL-backed `bulk_insert` handle. Wraps a `PgPool`
/// and the school the bulk path is scoped to. The
/// [`PostgresStorageAdapter`](crate::storage::PostgresStorageAdapter)
/// and [`PostgresTransaction`](crate::transaction::PostgresTransaction)
/// each construct a `PostgresBulkAttendance` and delegate to
/// the same internal [`bulk_insert_into`] helper.
#[derive(Clone)]
pub struct PostgresBulkAttendance {
    pool: PgPool,
    school: SchoolId,
}

impl std::fmt::Debug for PostgresBulkAttendance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresBulkAttendance")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl PostgresBulkAttendance {
    /// Constructs a new bulk-insert handle bound to `pool` and
    /// scoped to `school`.
    #[must_use]
    pub fn new(pool: PgPool, school: SchoolId) -> Self {
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
    /// [`PostgresStorageAdapter::migrate`](crate::storage::PostgresStorageAdapter::migrate)
    /// after the six engine cross-cutting tables are created.
    #[instrument(skip(self))]
    pub async fn ensure_schema(&self) -> Result<()> {
        sqlx::raw_sql(SCHEMA_SQL)
            .execute(&self.pool)
            .await
            .map_err(DomainError::infrastructure)?;
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
/// [`PostgresBulkAttendance::bulk_insert`] method delegates
/// here; the free function is the canonical implementation
/// (kept separate so callers with their own `PgPool` can use
/// it without constructing a `PostgresBulkAttendance`).
///
/// # Errors
/// - `Validation` if any row's `school_id` does not match
///   `school_id` or if `rows.len() > MAX_ROWS_PER_CALL`.
/// - `Conflict` on a unique-key violation of
///   `(school_id, student_id, attendance_date)`.
/// - `Infrastructure` for any underlying storage error.
#[instrument(skip(pool, rows), fields(n = rows.len(), school = %school_id))]
pub async fn bulk_insert_into(
    pool: &PgPool,
    school_id: SchoolId,
    rows: &[StudentAttendanceRow],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    if rows.len() > MAX_ROWS_PER_CALL {
        return Err(DomainError::validation(format!(
            "bulk_insert_student_attendances: at most {MAX_ROWS_PER_CALL} rows per call, got {}",
            rows.len()
        )));
    }
    for (i, r) in rows.iter().enumerate() {
        if r.school_id != school_id {
            return Err(DomainError::validation(format!(
                "bulk_insert_student_attendances: row {i} school_id mismatch (expected {school_id}, got {})",
                r.school_id
            )));
        }
    }

    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "INSERT INTO attendance_student_attendances (\
            school_id, id, student_id, student_record_id, class_id, section_id, \
            attendance_date, attendance_type, in_time, out_time, notes, is_absent, \
            marked_by, marked_at, marked_from, version, etag, created_at, updated_at, \
            created_by, updated_by, active_status, last_event_id, correlation_id\
         ) VALUES ",
    );
    qb.push_values(rows.iter(), |mut b, r| {
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

    match qb.build().execute(pool).await {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(db))
            if db.kind() == sqlx::error::ErrorKind::UniqueViolation =>
        {
            Err(DomainError::conflict(
                "bulk_insert_student_attendances: duplicate (school_id, student_id, attendance_date) row",
            ))
        }
        Err(other) => Err(DomainError::infrastructure(other)),
    }
}
