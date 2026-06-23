//! `educore-storage-surrealdb` — the SurrealDB storage adapter
//! (Phase 0 primary per [`ADR-017`]).
//!
//! This crate implements the
//! [`StorageAdapter`](educore_storage::port::StorageAdapter) port
//! against SurrealDB. The Phase 0 minimum viable implementation
//! supports the in-memory backend (`Mem`); a future PR adds the
//! RocksDB / TiKV / HTTP backends.
//!
//! [`ADR-017`]: ../../docs/decisions/ADR-017-SurrealDBFirst.md

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod audit;
pub mod connection;
pub(crate) mod error;
pub mod event_log;
pub mod idempotency;
pub mod outbox;
pub mod schema;
pub mod storage;
pub mod stubs;
pub mod transaction;

pub use audit::SurrealAuditLog;
pub use connection::SurrealConnection;
pub use event_log::SurrealEventLog;
pub use idempotency::SurrealIdempotency;
pub use outbox::SurrealOutbox;
pub use storage::SurrealStorageAdapter;
pub use transaction::SurrealTransaction;
