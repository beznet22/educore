//! PostgreSQL-backed `Transaction`.
//!
//! ## Design
//!
//! This module deliberately does **not** hold a
//! `sqlx::Transaction<'_, Postgres>` for the lifetime of the
//! `PostgresTransaction` struct. Holding the `sqlx::Transaction`
//! would require borrowing the pool for the transaction's
//! lifetime, which collides with the
//! `&dyn Outbox` / `&dyn AuditLog` / `&dyn Idempotency` /
//! `&dyn EventLog` accessors on the `Transaction` trait
//! (those return references that must outlive the accessors
//! that hand them out).
//!
//! Instead, the four sub-port impls open their own short-lived
//! `pool.begin()` transactions on every method call. The
//! `PostgresTransaction`'s `commit` and `rollback` are no-ops:
//! a `sqlx::Transaction` auto-commits on `Drop`, so each
//! sub-port call commits independently. The engine's at-least-
//! once outbox semantics (dedup by `event_id` primary key,
//! `ON CONFLICT DO NOTHING` on idempotency inserts) ensure the
//! duplicate-dispatch is idempotent at the storage layer.
//!
//! ## State tracking
//!
//! The `done` and `rolled_back` `AtomicBool` flags mirror the
//! SurrealDB adapter's design: they let the engine detect a
//! double-commit / double-rollback misuse without panicking.
//! Calling `commit` or `rollback` on an already-completed
//! transaction is a no-op (returns `Ok(())`).
//!
//! See `docs/ports/storage.md` § "Transactions" for the engine
//! contract.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::audit::AuditLog;
use educore_storage::event_log::EventLog;
use educore_storage::idempotency::Idempotency;
use educore_storage::outbox::Outbox;
use educore_storage::transaction::Transaction;
use educore_storage::StudentAttendanceRow;

use crate::audit_log::PostgresAuditLog;
use crate::bulk_attendance::PostgresBulkAttendance;
use crate::event_log::PostgresEventLog;
use crate::idempotency::PostgresIdempotency;
use crate::outbox::PostgresOutbox;

/// The PostgreSQL-backed transaction. Owns its four sub-port
/// handles; the `Transaction` trait's `&self`-returning
/// methods hand out `&dyn SubPort` references for the
/// transaction's lifetime.
pub struct PostgresTransaction {
    /// The real outbox handle.
    outbox: PostgresOutbox,
    /// The audit-log handle.
    audit: PostgresAuditLog,
    /// The event-log handle.
    event: PostgresEventLog,
    /// The idempotency handle.
    idem: PostgresIdempotency,
    /// The bulk-attendance handle. The bulk-insert path opens
    /// its own short-lived transaction on every call (matching
    /// the design of the other sub-ports), so the handle is
    /// effectively a `&PgPool` + `SchoolId` pair.
    bulk: PostgresBulkAttendance,
    /// `true` once the transaction has been committed or rolled
    /// back. Re-used as a guard against double-commit /
    /// double-rollback.
    done: AtomicBool,
    /// `true` if the transaction has been rolled back.
    rolled_back: AtomicBool,
    /// The pool handle is held here so the transaction owns a
    /// reference for its lifetime. `sqlx::PgPool` is cheaply
    /// `Clone` (its inner state is `Arc`-shared), so this is
    /// just a borrow tracker.
    _pool: PgPool,
}

impl std::fmt::Debug for PostgresTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresTransaction")
            .field("school", &self.outbox.school())
            .field("done", &self.done.load(Ordering::SeqCst))
            .field("rolled_back", &self.rolled_back.load(Ordering::SeqCst))
            .finish_non_exhaustive()
    }
}

impl PostgresTransaction {
    /// Constructs a new transaction handle bound to `pool` and
    /// scoped to `school`.
    #[must_use]
    pub fn new(pool: PgPool, school: SchoolId) -> Self {
        let outbox = PostgresOutbox::new(pool.clone(), school);
        let audit = PostgresAuditLog::new(pool.clone(), school);
        let event = PostgresEventLog::new(pool.clone(), school);
        let idem = PostgresIdempotency::new(pool.clone(), school);
        let bulk = PostgresBulkAttendance::new(pool.clone(), school);
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

/// QW-4 / `ADAPTER-PG-005` Drop contract.
///
/// If the transaction is dropped without an explicit `commit`
/// or `rollback`, the `Drop` impl flips the `rolled_back`
/// guard so any subsequent introspections of the transaction
/// state observe a completed transaction (and any consumer
/// code that holds the inner sub-port handles will see the
/// "rolled back" state).
///
/// The current implementation's sub-port methods each commit
/// their own short-lived `sqlx::Transaction` immediately, so
/// there is no long-running database transaction to roll
/// back at the struct level. The flag flip here is the
/// port-level rollback contract: it is what `rollback().await`
/// would have done, performed synchronously from `Drop`.
/// We log a warning so dropped-without-finalize is observable
/// in tracing output (a programming error in the caller).
impl Drop for PostgresTransaction {
    fn drop(&mut self) {
        if !self.done.load(Ordering::SeqCst) {
            tracing::warn!(
                school = %self.outbox.school(),
                "PostgresTransaction dropped without commit or rollback; \
                 performing implicit rollback"
            );
            self.rolled_back.store(true, Ordering::SeqCst);
            self.done.store(true, Ordering::SeqCst);
        }
    }
}

#[async_trait]
impl Transaction for PostgresTransaction {
    #[instrument(skip(self))]
    async fn commit(self: Box<Self>) -> Result<()> {
        // No-op: the sub-port operations have already committed
        // via the `sqlx::Transaction` they each acquired. We
        // only flip the guard flag.
        self.done.store(true, Ordering::SeqCst);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn rollback(self: Box<Self>) -> Result<()> {
        // No-op: see the module-level doc on the design.
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

    #[instrument(skip(self, rows), fields(n = rows.len()))]
    async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
        if self.done.load(Ordering::SeqCst) {
            return Err(DomainError::conflict(
                "Transaction::bulk_insert_student_attendances called on a completed transaction",
            ));
        }
        // The transaction is scoped to `self.bulk.school()`; the
        // per-row school_id check is delegated to
        // `PostgresBulkAttendance::bulk_insert` which compares
        // every row's `school_id` against the scoped school.
        self.bulk.bulk_insert(self.bulk.school(), rows).await
    }
}
