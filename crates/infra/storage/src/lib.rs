//! `educore-storage` — the engine's sole entry point for
//! persistence.
//!
//! This crate defines the storage port: the traits that storage
//! adapters must implement, the sub-ports for the outbox / audit
//! log / idempotency store / event log, and the types carried
//! across the port boundary. The engine never writes directly
//! to a database; all persistence flows through this port.
//!
//! See `docs/ports/storage.md` for the full contract.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Change-stream / snapshot / cursor wire types.
pub mod change_stream;

/// The `AuditLog` sub-port.
pub mod audit;

/// The `EventLog` sub-port.
pub mod event_log;

/// The `Idempotency` sub-port.
pub mod idempotency;

/// The `Outbox` sub-port (transactional outbox pattern).
pub mod outbox;

/// The generic per-aggregate `Repository<A>` trait.
pub mod repository;

/// The `Transaction` sub-port.
pub mod transaction;

/// The bulk-insert row type for `StudentAttendance`. Wire format
/// for the `StorageAdapter::bulk_insert_student_attendances` and
/// `Transaction::bulk_insert_student_attendances` port methods
/// (Phase 5 Workstream A).
pub mod student_attendance_row;

/// The `StorageAdapter` port itself.
pub mod port;

pub use audit::{AuditLog, AuditLogEntry};
pub use change_stream::{
    AggregateTypeFilter, ChangeEvent, ChangeFilter, ChangeStream, MigrationReport, SchoolSnapshot,
    SerializedChangeEvent, SnapshotAggregate, VersionCursor,
};
pub use event_log::{EventLog, EventLogEntry, EventLogFilter};
pub use idempotency::{
    Idempotency, IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};
pub use outbox::{Outbox, SerializedEnvelope};
pub use port::StorageAdapter;
pub use repository::Repository;
pub use student_attendance_row::StudentAttendanceRow;
pub use transaction::Transaction;

// Re-export the `educore_core::error::Result` alias for convenience.
pub use educore_core::error::Result;

#[cfg(test)]
mod integration {
    //! Cross-module smoke tests: ensure the public re-exports
    //! line up and the sub-ports compose.
    use super::*;

    #[test]
    fn re_exports_are_consistent() {
        // Reference each trait to silence "unused import" warnings
        // and to catch circular-definition bugs at compile time.
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<Box<dyn StorageAdapter + '_>>();
        _assert_send_sync::<Box<dyn Transaction + '_>>();
        _assert_send_sync::<Box<dyn Outbox + '_>>();
        _assert_send_sync::<Box<dyn AuditLog + '_>>();
        _assert_send_sync::<Box<dyn EventLog + '_>>();
        _assert_send_sync::<Box<dyn Idempotency + '_>>();
    }
}
