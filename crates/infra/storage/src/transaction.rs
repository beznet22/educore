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
//!
//! ## Drop contract (PORT-STORE-014 / QW-4)
//!
//! **Every `Transaction` impl MUST implement `Drop`.** If the
//! transaction is dropped without `commit()` or `rollback()`
//! having been called, the `Drop` impl MUST perform a
//! best-effort rollback (logging a warning if it cannot reach
//! a runtime to do the rollback asynchronously).
//!
//! The engine's transactional contract is "rollback by
//! default" — a dropped transaction that committed silently
//! would violate this. Implementations that hold an underlying
//! `sqlx::Transaction` (or equivalent) MUST NOT rely on the
//! upstream type's default `Drop` (which may commit); they
//! MUST invoke an explicit rollback from their own `Drop`.
//!
//! See the wave-3 adapter findings
//! (`ADAPTER-PG-005` / `ADAPT-MY-005` / `ADAPTER-SQ-005` /
//! `ADAPTER-SD-005`) and the wave-4 port finding
//! (`PORT-STORE-014`) for the original audit trail.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use super::audit::AuditLog;
use super::event_log::EventLog;
use super::idempotency::Idempotency;
use super::outbox::Outbox;
use super::student_attendance_row::StudentAttendanceRow;

/// The transactional unit of work. Object-safe (one trait
/// object per transaction).
///
/// ## Drop contract
///
/// **Implementations MUST implement `Drop`** such that
/// dropping a transaction that was neither `commit()`ted nor
/// `rollback()`ed triggers an automatic rollback. The
/// contract is advisory here (Rust cannot put `Drop` on a
/// trait; it lives on the concrete impl) and is enforced by
/// convention: every shipped `Transaction` impl has a `Drop`
/// that honors it. See the module-level docs for the
/// rationale and the source findings.
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

/// Extension trait that exposes the tenant scope of a
/// [`Transaction`].
///
/// ## Why a separate trait?
///
/// Per `docs/audit_reports/findings/wave4-storage-port.md`
/// finding **PORT-STORE-002**, the base `Transaction` trait
/// carries no tenant anchor — there is no way for an adapter
/// to know which `SchoolId` scope the transaction runs in.
/// Adding the accessor to the base trait would be a breaking
/// change (every adapter would have to implement a new
/// method); instead the accessor is exposed as a separate
/// extension trait so existing code keeps compiling.
///
/// Every shipped `Transaction` impl implements both `Transaction`
/// and `TenantTransaction`; consumers that need the tenant
/// scope (e.g. a generic adapter wrapper that wants to log the
/// tenant on each sub-port call, or a per-tenant routing layer)
/// hold the concrete transaction type and call `school_id()`
/// directly, or rely on the adapter to propagate the tenant
/// scope internally.
///
/// ## Audit-handle guarantee (PORT-STORE-013)
///
/// The base trait already exposes `audit_log()` which returns a
/// `&dyn AuditLog`; the handle returned by `audit_log()` on a
/// `TenantTransaction` impl MUST commit (or roll back) atomically
/// with the rest of the transaction's writes. Adapters that
/// open per-method transactions (the SQL adapters) honour this
/// by acquiring the per-method connection from the same pool
/// and letting the database's transactional outbox + audit
/// insert share the connection. Adapters with a staging layer
/// (the testkit) honour this by staging all writes until
/// `commit()` and discarding them on `rollback()`.
pub trait TenantTransaction: Transaction {
    /// Returns the `SchoolId` this transaction is scoped to.
    ///
    /// Every staged write (outbox append, audit row, idempotency
    /// record, event log row) belongs to this school. Adapters
    /// MUST reject any staged entry whose `school_id` differs
    /// from the value returned here.
    fn school_id(&self) -> SchoolId;
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

    // Compile-time check that `TenantTransaction` is also
    // dyn-compatible. The extension trait only adds a
    // `&self -> SchoolId` accessor, which is object-safe.
    fn _assert_tenant_object_safe(_t: Box<dyn TenantTransaction + Sync>) {}
}
