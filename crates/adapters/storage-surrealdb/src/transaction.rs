//! SurrealDB-backed `Transaction`.
//!
//! Phase 0 implements a simple commit/rollback wrapper. The
//! SurrealDB SDK does not currently expose explicit transaction
//! control (transactions are implicit on the `query` API), so
//! the implementation tracks a "done" flag and an
//! "in-transaction" flag. A future PR will use the SurrealDB 3.x
//! transaction API for true atomicity.

use std::sync::atomic::AtomicBool;

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_storage::audit::AuditLog;
use educore_storage::event_log::EventLog;
use educore_storage::idempotency::Idempotency;
use educore_storage::outbox::Outbox;
use educore_storage::transaction::Transaction;

use crate::connection::Db;
use crate::outbox::SurrealOutbox;
use crate::audit::SurrealAuditLog;
use crate::event_log::SurrealEventLog;
use crate::idempotency::SurrealIdempotency;

/// The SurrealDB-backed transaction. Owns its sub-port
/// handles; the `Transaction` trait's `&self`-returning
/// methods can hand out `&dyn SubPort` references for the
/// transaction's lifetime.
pub struct SurrealTransaction {
    /// The real outbox handle.
    outbox: SurrealOutbox,
    /// The audit-log stub.
    audit: SurrealAuditLog,
    /// The event-log stub.
    event: SurrealEventLog,
    /// The idempotency stub.
    idem: SurrealIdempotency,
    /// `true` once the transaction has been committed or rolled
    /// back.
    done: AtomicBool,
    /// `true` if the transaction has been rolled back.
    rolled_back: AtomicBool,
    /// The DB handle is held here so the transaction owns the
    /// connection for its lifetime. `Surreal<Db>` is cheaply
    /// `Clone` (its inner state is `Arc`-shared).
    _db: Db,
}

impl std::fmt::Debug for SurrealTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealTransaction")
            .field("school", &self.outbox.school)
            .finish_non_exhaustive()
    }
}

impl SurrealTransaction {
    /// Wraps a SurrealDB instance in a new transaction.
    pub fn new(db: Db, school: SchoolId) -> Self {
        let outbox = SurrealOutbox::new(db.clone(), school);
        let audit = SurrealAuditLog {
            db: db.clone(),
            school,
        };
        let event = SurrealEventLog {
            db: db.clone(),
            school,
        };
        let idem = SurrealIdempotency { db: db.clone() };
        Self {
            outbox,
            audit,
            event,
            idem,
            done: AtomicBool::new(false),
            rolled_back: AtomicBool::new(false),
            _db: db,
        }
    }
}

#[async_trait]
impl Transaction for SurrealTransaction {
    async fn commit(self: Box<Self>) -> Result<()> {
        self.done.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<()> {
        self.rolled_back
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.done.store(true, std::sync::atomic::Ordering::SeqCst);
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
}
