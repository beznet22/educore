//! SQLite-backed `Transaction`.
//!
//! ## Design note (Phase 1)
//!
//! The engine's [`Transaction`] trait's accessors return
//! `&dyn SubPort` references, which forces the sub-port
//! handles to outlive the borrow of any sqlx-level
//! `sqlx::Transaction<'_, Sqlite>`. To keep the sub-port
//! signatures simple and `Send + Sync` without lifetime
//! parameters, the Phase 1 implementation does **not** hold a
//! real sqlx transaction. Instead:
//!
//! - The struct holds the `SqlitePool` (cheap to clone) and
//!   the 4 sub-port handles.
//! - Each sub-port method runs its own short implicit
//!   transaction via the pool (sqlx auto-commits per call).
//! - The `commit` / `rollback` trait methods set the
//!   `done` / `rolled_back` flags and return `Ok(())`.
//!
//! Net effect: the sub-port operations are visible to each
//! other immediately (a read after an `append` sees the new
//! row), just as the SurrealDB Phase 0 implementation does.
//! A future PR will swap in real sqlx `BEGIN IMMEDIATE` /
//! `COMMIT` / `ROLLBACK` semantics for true atomicity (per
//! `docs/schemas/sql-dialects/sqlite.md#transactions`).

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sqlx::SqlitePool;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::audit::AuditLog;
use educore_storage::event_log::EventLog;
use educore_storage::idempotency::Idempotency;
use educore_storage::outbox::Outbox;
use educore_storage::transaction::Transaction;
use educore_storage::StudentAttendanceRow;

use crate::audit_log::SqliteAuditLog;
use crate::bulk_attendance::SqliteBulkAttendance;
use crate::event_log::SqliteEventLog;
use crate::idempotency::SqliteIdempotency;
use crate::outbox::SqliteOutbox;

/// The SQLite-backed transaction. Owns its sub-port handles;
/// the `Transaction` trait's `&self`-returning methods can
/// hand out `&dyn SubPort` references for the transaction's
/// lifetime.
pub struct SqliteTransaction {
    outbox: SqliteOutbox,
    audit: SqliteAuditLog,
    event: SqliteEventLog,
    idem: SqliteIdempotency,
    /// The bulk-attendance handle. The bulk-insert path opens
    /// its own real `BEGIN` / `COMMIT` transaction on every
    /// call so a failure in one batched `INSERT` rolls back
    /// the work of the prior batches (matching the engine's
    /// all-or-nothing invariant for the bulk-marking
    /// service).
    bulk: SqliteBulkAttendance,
    /// `true` once the transaction has been committed or
    /// rolled back. Matches the SurrealDB flag-based pattern.
    done: AtomicBool,
    /// `true` if the transaction has been rolled back.
    rolled_back: AtomicBool,
    /// The pool is held here so the transaction owns the
    /// connection for its lifetime. `SqlitePool` is cheaply
    /// `Clone` (its inner state is `Arc`-shared).
    _pool: SqlitePool,
}

impl fmt::Debug for SqliteTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteTransaction")
            .field("school", &self.outbox.school())
            .finish_non_exhaustive()
    }
}

impl SqliteTransaction {
    /// Wraps a `SqlitePool` in a new transaction, constructing
    /// the 4 sub-port handles scoped to `school`.
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        let outbox = SqliteOutbox::new(pool.clone(), school);
        let audit = SqliteAuditLog::new(pool.clone(), school);
        let event = SqliteEventLog::new(pool.clone(), school);
        let idem = SqliteIdempotency::new(pool.clone(), school);
        let bulk = SqliteBulkAttendance::new(pool.clone(), school);
        Self {
            outbox,
            audit,
            event,
            idem,
            bulk,
            done: AtomicBool::new(false),
            rolled_back: AtomicBool::new(false),
            _pool: pool,
        }
    }
}

#[async_trait]
impl Transaction for SqliteTransaction {
    async fn commit(self: Box<Self>) -> Result<()> {
        self.done.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        self.rolled_back.store(true, Ordering::SeqCst);
        self.done.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn outbox(&self) -> &dyn Outbox {
        &self.outbox
    }

    fn audit_log(&self) -> &dyn AuditLog {
        &self.audit
    }

    fn idempotency(&self) -> &dyn Idempotency {
        &self.idem
    }

    fn event_log(&self) -> &dyn EventLog {
        &self.event
    }

    async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
        if self.done.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "Transaction::bulk_insert_student_attendances called on a completed transaction",
            ));
        }
        // The transaction is scoped to `self.bulk.school()`; the
        // per-row school_id check is delegated to
        // `SqliteBulkAttendance::bulk_insert` which compares
        // every row's `school_id` against the scoped school.
        // The internal helper opens a real `BEGIN` / `COMMIT`
        // transaction so a failure in one batched `INSERT`
        // rolls back the whole bulk insert.
        self.bulk.bulk_insert(self.bulk.school(), rows).await
    }
}
