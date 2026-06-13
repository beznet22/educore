//! The `Transaction` sub-port — a transactional unit of work.
//!
//! Per `docs/ports/storage.md` § 2 and
//! `docs/schemas/database-schema.md` § 2, every command runs
//! inside a transaction. The transaction owns:
//!
//! - The **outbox** — every state change is durably enqueued
//!   in the same transaction as the aggregate mutation
//!   (transactional outbox pattern).
//! - The **repositories** — per-aggregate handles that stage
//!   reads and writes within the transaction.
//! - The **audit log**, **idempotency store**, and **event log**
//!   — all write through the transaction so the audit row, the
//!   outbox envelope, and the event log row are committed
//!   atomically with the aggregate state.
//!
//! The transaction is consumed by `commit` or `rollback`
//! (`self: Box<Self>` to take ownership of the trait object).

use async_trait::async_trait;

use educore_core::error::Result;

use super::audit::AuditLog;
use super::event_log::EventLog;
use super::idempotency::Idempotency;
use super::outbox::Outbox;
use super::student_attendance_row::StudentAttendanceRow;

/// The transactional unit of work. Object-safe (one trait
/// object per transaction).
#[async_trait]
pub trait Transaction: Send + Sync + std::fmt::Debug {
    /// Commits the transaction. All outbox appends, aggregate
    /// mutations, audit log writes, idempotency records, and
    /// event log rows become durable. Consumes the transaction.
    ///
    /// # Errors
    /// - `Conflict` on a unique-key violation, deadlock, or
    ///   serialisation failure (the engine retries the command
    ///   automatically).
    /// - `Infrastructure` for any underlying storage error.
    async fn commit(self: Box<Self>) -> Result<()>;

    /// Rolls the transaction back. All staged writes are
    /// discarded. Consumes the transaction.
    async fn rollback(self: Box<Self>) -> Result<()>;

    /// Returns a handle to the outbox. The caller stages
    /// envelopes here; they become durable on `commit`.
    fn outbox(&self) -> &dyn Outbox;

    /// Returns a handle to the audit log. The caller stages
    /// audit rows here; they become durable on `commit`.
    fn audit_log(&self) -> &dyn AuditLog;

    /// Returns a handle to the idempotency store. The caller
    /// checks for duplicates and stores outcomes here; they
    /// become durable on `commit`.
    fn idempotency(&self) -> &dyn Idempotency;

    /// Returns a handle to the event log. The relay drains the
    /// outbox into the event log via this method.
    fn event_log(&self) -> &dyn EventLog;

    /// Bulk-inserts N `StudentAttendance` rows within the
    /// current transaction. Same invariants as
    /// [`StorageAdapter::bulk_insert_student_attendances`](super::port::StorageAdapter::bulk_insert_student_attendances):
    /// the row's `school_id` MUST equal the transaction's
    /// scoped school (enforced by the adapter) and a duplicate
    /// `(school_id, student_id, attendance_date)` is rejected
    /// with `DomainError::Conflict`. The default implementation
    /// returns `NotSupported`; SQL adapters override.
    ///
    /// The bulk-marking service uses this method (it is
    /// running inside a transaction so the outbox appends, the
    /// idempotency record, the audit row, and the
    /// `StudentAttendance` rows all commit atomically).
    ///
    /// # Errors
    /// - `Validation` if any row's `school_id` does not match
    ///   the transaction's scoped school.
    /// - `Conflict` on a unique-key violation of
    ///   `(school_id, student_id, attendance_date)`.
    /// - `Infrastructure` for any underlying storage error.
    async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> {
        let _ = rows;
        Err(educore_core::error::DomainError::not_supported(
            "Transaction::bulk_insert_student_attendances is not supported by this adapter",
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

    // Compile-time check that the trait is dyn-compatible
    // (object-safe). `Box<dyn Transaction>` is used in the
    // StorageAdapter port and elsewhere; if the trait gains a
    // generic method, this assertion will fail to compile.
    fn _assert_object_safe(_t: Box<dyn Transaction + Sync>) {}
}
