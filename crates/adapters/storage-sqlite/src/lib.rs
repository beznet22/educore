//! `educore-storage-sqlite` — the SQLite storage adapter (Phase 1).
//!
//! Implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against SQLite 3.x. The adapter targets both embedded /
//! offline deployments (Tauri, mobile, CLI) and single-process
//! production deployments. Multi-writer scenarios are
//! out-of-scope: SQLite is the engine's embedded / offline mode
//! (per [`ADR-017`]).
//!
//! The DDL is `include_str!`'d from
//! `migrations/engine/0000_engine_core.sqlite.sql` so the schema
//! stays in lockstep with the engine's other SQL adapters.
//!
//! [`ADR-017`]: ../../docs/decisions/ADR-017-SurrealDBFirst.md

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit_log;
pub mod bulk_attendance;
pub mod connection;
pub(crate) mod error;
pub mod event_log;
pub mod idempotency;
pub mod outbox;
pub mod storage;
pub mod transaction;
pub(crate) mod util;

pub use audit_log::SqliteAuditLog;
pub use bulk_attendance::SqliteBulkAttendance;
pub use connection::SqliteConnection;
pub use event_log::SqliteEventLog;
pub use idempotency::SqliteIdempotency;
pub use outbox::SqliteOutbox;
pub use storage::SqliteStorageAdapter;
pub use transaction::SqliteTransaction;
