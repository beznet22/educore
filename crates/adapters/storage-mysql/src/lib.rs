//! `educore-storage-mysql` — the MySQL storage adapter
//! (Phase 1 reference target per the Educore build plan).
//!
//! This crate implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against MySQL 8+ via `sqlx` 0.8 (the same driver family as
//! the PostgreSQL and SQLite adapters). All four engine
//! cross-cutting sub-ports (outbox, audit log, event log,
//! idempotency) are implemented as real impls — the Phase 0
//! stub-on-`NotSupported` pattern is **not** used here.
//!
//! ## Schema
//!
//! The DDL for the 6 engine cross-cutting tables (outbox,
//! audit_log, idempotency, event_log, schema_registry,
//! system_user) is `include_str!`'d from
//! `migrations/engine/0000_engine_core.mysql.sql` at compile
//! time. The DDL is MySQL 8+ specific: backtick-quoted
//! identifiers, `ENGINE=InnoDB`, `CHARSET=utf8mb4`,
//! `COLLATE=utf8mb4_unicode_ci`, `JSON` columns (not `JSONB`),
//! `CHAR(36)` UUID columns, and a `SET FOREIGN_KEY_CHECKS=0`
//! wrapper around the script for idempotent re-runs.
//!
//! ## Tenant isolation
//!
//! Per `docs/schemas/sql-dialects/mysql.md`, tenant isolation
//! is enforced via a `school_id` predicate on every query.
//! MySQL 8 supports row-level security, but the engine's
//! Phase 1 adapter relies on the application-level filter; an
//! RLS overlay is deferred to a future release.
//!
//! ## Connection setup
//!
//! The adapter parses a standard `mysql://user:pass@host:port/db`
//! URL into `MySqlConnectOptions` via
//! `MySqlConnectOptions::from_str`. sqlx 0.8 does not expose a
//! builder method for `multi_statements`; the only knob is the
//! URL's query string, so the adapter's
//! [`MysqlConnection::connect`](crate::connection::MysqlConnection::connect)
//! appends `?multi_statements=true` (or
//! `&multi_statements=true`) to the URL if it is not already
//! present. This lets the `migrate()` path run the
//! multi-statement DDL in a single round-trip.
//! The `after_connect` hook issues
//! `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci` on every new
//! connection so the connection's character set matches the DDL's
//! `utf8mb4_unicode_ci` collation (the default
//! `utf8mb4_0900_ai_ci` is accent-insensitive; the engine prefers
//! `unicode_ci` for predictable tenant-data sorting).
//!
//! ## Transactions
//!
//! The adapter follows the same design as the PostgreSQL and
//! SQLite adapters: it does not hold a `sqlx::Transaction` (which
//! would borrow the pool for the transaction's lifetime and
//! conflict with the `&dyn SubPort` accessors on the `Transaction`
//! trait). Each sub-port call runs its own short-lived
//! transaction via `pool.begin()`. `commit` and `rollback` on
//! the `MysqlTransaction` are no-ops; the engine's at-least-once
//! semantics on the outbox ensure that a duplicate dispatch is
//! idempotent at the storage layer (the `ON DUPLICATE KEY UPDATE
//! command_id = command_id` no-op assignment in
//! `Idempotency::record` and the `event_id` primary key on the
//! outbox are the safety net).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit_log;
pub mod bulk_attendance;
pub mod connection;
pub(crate) mod connection_helpers;
pub mod error;
pub mod event_log;
pub mod idempotency;
pub mod outbox;
pub mod schema;
pub mod storage;
pub mod transaction;

pub use audit_log::MysqlAuditLog;
pub use bulk_attendance::MysqlBulkAttendance;
pub use connection::MysqlConnection;
pub use event_log::MysqlEventLog;
pub use idempotency::MysqlIdempotency;
pub use outbox::MysqlOutbox;
pub use schema::{
    build_fk_ddl, build_index_ddl, build_schema_statements, build_table_ddl, column_type_to_mysql,
    create_schema, create_schema_with, create_schema_with_report, fk_action_to_mysql,
    register_entity_descriptor, registered_descriptors,
};
pub use storage::MysqlStorageAdapter;
pub use transaction::MysqlTransaction;
