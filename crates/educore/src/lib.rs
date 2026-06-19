//! # Educore
//!
//! The Educore umbrella crate. Re-exports every domain, port, and
//! adapter crate under a single, stable path.
//!
//! Consumers should depend on `educore` and import the submodules
//! they need:
//!
//! ```rust,ignore
//! use educore::prelude::*;
//! use educore::academic::commands::*;
//! ```
//!
//! See `docs/project-overview.md` for the engine philosophy and
//! `docs/architecture.md` for the system map.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

// ---- Domain crates ------------------------------------------------------
pub use educore_academic as academic;
pub use educore_assessment as assessment;
pub use educore_attendance as attendance;
pub use educore_cms as cms;
pub use educore_communication as communication;
pub use educore_core as core;
pub use educore_documents as documents;
pub use educore_events as events;
pub use educore_events_domain as events_domain;
pub use educore_facilities as facilities;
pub use educore_finance as finance;
pub use educore_hr as hr;
pub use educore_library as library;
pub use educore_operations as operations;
pub use educore_platform as platform;
pub use educore_rbac as rbac;
pub use educore_settings as settings;

// ---- Port adapters -------------------------------------------------------
pub use educore_auth as auth;
pub use educore_event_bus as event_bus;
pub use educore_files as files;
pub use educore_integrations as integrations;
pub use educore_notify as notify;
pub use educore_payment as payment;
pub use educore_storage as storage;
pub use educore_storage_mysql as storage_mysql;
pub use educore_storage_parity as storage_parity;
pub use educore_storage_postgres as storage_postgres;
pub use educore_storage_sqlite as storage_sqlite;
pub use educore_storage_surrealdb as storage_surrealdb;

// ---- Sync engine (cross-cutting port, Phase 0 per ADR-018) ---------------
pub use educore_sync as sync;
pub use educore_sync_inprocess as sync_inprocess;

// ---- Test infrastructure -------------------------------------------------
pub use educore_audit as audit;
pub use educore_testkit as testkit;

// ---- High-level SDK ------------------------------------------------------
pub use educore_sdk as sdk;

/// Educore version, sourced from the package manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude of common types consumers are expected to import.
///
/// The prelude's typed re-exports are wired in incrementally as the
/// underlying crates are implemented in Phase 0 (PRs 3-8). At the
/// scaffold stage the prelude is intentionally a thin re-export of
/// crate-level paths so the workspace builds; richer re-exports land
/// alongside the `DomainError`, `TenantContext`, `EventEnvelope`, and
/// `Capability` types in the relevant PRs. Phase 14 adds the
/// `educore_settings` + `educore_operations` re-exports.
pub mod prelude {
    pub use educore_core;
    pub use educore_events;
    pub use educore_operations;
    pub use educore_platform;
    pub use educore_rbac;
    pub use educore_settings;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
