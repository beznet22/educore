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
// Gated behind the `sync` feature flag (ADR-018 § 4). Default build excludes
// the sync surface entirely; pass `--features sync` to the umbrella to include.
#[cfg(feature = "sync")]
pub use educore_sync as sync;
#[cfg(feature = "sync")]
pub use educore_sync_inprocess as sync_inprocess;

// ---- Test infrastructure -------------------------------------------------
pub use educore_audit as audit;
pub use educore_testkit as testkit;

// ---- High-level SDK ------------------------------------------------------
pub use educore_sdk as sdk;

// ---- CLI and proc-macro re-exports ---------------------------------------
// `educore-cli` is the reference command-line binary (Phase 16). It is a
// binary crate, so the re-export exposes the library surface for embedding
// (e.g. library-mode consumers that link the CLI parser as a sub-binary).
// `educore-query-derive` provides the `#[derive(DomainQuery)]` proc-macro
// (Phase 0 foundation). Re-exporting it makes the umbrella the single
// import surface for the entire engine per AGENTS.md § "Naming Convention
// (Enforced)".
pub use educore_cli as cli;
pub use educore_query_derive as query_derive;

// ---- Cross-domain subscriber wiring ---------------------------------------
// `subscribers::register_all_subscribers` wires the spec-mandated
// cross-domain handlers (form_uploaded_public_indexing,
// student_promoted_fee_structure, staff_registered_salary_template,
// payroll_paid_mark_paid) onto the `SubscriberRegistry`. The SDK
// facade calls this at server startup (per Cluster A/B SDK work).
// See `docs/audit_reports/findings/wave7-workflows.md` WF-002 /
// WF-016.
pub mod subscribers;

// ---- Async command handle (SCHEMA-CMD-ASYNC, § 13) -------------------------
// `command_handle::CommandRegistry` is the engine-facing port for
// asynchronous commands (bulk imports, payroll generation, report
// generation). `CommandHandle` is the handle returned by
// `engine.commands.submit` and consumed by `engine.commands.status` /
// `engine.commands.await`. See `docs/schemas/command-schema.md` § 13.
pub mod command_handle;

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
    pub use crate::command_handle::{
        CommandHandle, CommandOutcome, CommandRegistry, CommandStatus,
    };
    pub use educore_core;
    pub use educore_events;
    pub use educore_operations;
    pub use educore_platform;
    pub use educore_rbac;
    pub use educore_sdk;
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
