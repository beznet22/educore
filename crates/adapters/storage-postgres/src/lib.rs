//! `educore-storage-postgres` — the PostgreSQL storage adapter
//! (Phase 1 primary target per the Educore build plan).
//!
//! This crate implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against PostgreSQL 14+. All four engine cross-cutting sub-ports
//! (outbox, audit log, event log, idempotency) are implemented as
//! real impls — the Phase 0 stub-on-`NotSupported` pattern from the
//! SurrealDB adapter is **not** used here.
//!
//! ## Schema
//!
//! The DDL for the 6 engine cross-cutting tables (outbox, audit_log,
//! idempotency, event_log, schema_registry, system_user) is
//! `include_str!`'d from
//! `migrations/engine/0000_engine_core.postgres.sql` at compile
//! time. The DDL wraps the tables in the `engine` schema; the
//! adapter sets `search_path = engine, public` on every new
//! connection via a `sqlx` `after_connect` hook so unqualified
//! table names in queries resolve to the `engine` namespace.
//!
//! ## Tenant isolation
//!
//! Per `docs/schemas/sql-dialects/postgresql.md`, tenant isolation
//! is enforced via a `school_id` predicate on every query. Row-
//! level security policies are Phase 2 work; the Phase 1 adapter
//! relies on the application-level filter.
//!
//! ## Transactions
//!
//! The adapter follows the same design as `educore-storage-sqlite`:
//! it does not hold a `sqlx::Transaction` (which would borrow the
//! pool for the transaction's lifetime and conflict with the
//! `&dyn SubPort` accessors on the `Transaction` trait). Each
//! sub-port call runs its own short-lived transaction via
//! `pool.begin()`. `commit` and `rollback` on the
//! `PostgresTransaction` are no-ops; the engine's at-least-once
//! semantics on the outbox ensure that a duplicate dispatch is
//! idempotent at the storage layer (the `ON CONFLICT DO NOTHING`
//! in `Idempotency::record` and the `event_id` primary key on the
//! outbox are the safety net).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit_log;
pub mod bulk_attendance;
pub mod connection;
pub(crate) mod connection_helpers;
pub mod ddl;
pub mod error;
pub mod event_log;
pub mod idempotency;
pub mod outbox;
pub mod schema;
pub mod storage;
pub mod transaction;

pub use audit_log::PostgresAuditLog;
pub use bulk_attendance::PostgresBulkAttendance;
pub use connection::PostgresConnection;
pub use event_log::PostgresEventLog;
pub use idempotency::PostgresIdempotency;
pub use outbox::PostgresOutbox;
pub use storage::PostgresStorageAdapter;
pub use transaction::PostgresTransaction;
