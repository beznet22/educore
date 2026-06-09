//! # SMSengine
//!
//! The SMSengine umbrella crate. Re-exports every domain, port, and
//! adapter crate under a single, stable path.
//!
//! Consumers should depend on `smsengine` and import the submodules
//! they need:
//!
//! ```rust,ignore
//! use smsengine::prelude::*;
//! use smsengine::academic::commands::*;
//! ```
//!
//! See `docs/project-overview.md` for the engine philosophy and
//! `docs/architecture.md` for the system map.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

// ---- Domain crates ------------------------------------------------------
pub use smsengine_core as core;
pub use smsengine_academic as academic;
pub use smsengine_assessment as assessment;
pub use smsengine_attendance as attendance;
pub use smsengine_cms as cms;
pub use smsengine_communication as communication;
pub use smsengine_documents as documents;
pub use smsengine_events as events;
pub use smsengine_events_domain as events_domain;
pub use smsengine_facilities as facilities;
pub use smsengine_finance as finance;
pub use smsengine_hr as hr;
pub use smsengine_library as library;
pub use smsengine_operations as operations;
pub use smsengine_platform as platform;
pub use smsengine_rbac as rbac;
pub use smsengine_settings as settings;

// ---- Port adapters -------------------------------------------------------
pub use smsengine_auth as auth;
pub use smsengine_event_bus as event_bus;
pub use smsengine_files as files;
pub use smsengine_integrations as integrations;
pub use smsengine_notify as notify;
pub use smsengine_payment as payment;
pub use smsengine_storage as storage;
pub use smsengine_storage_mysql as storage_mysql;
pub use smsengine_storage_postgres as storage_postgres;
pub use smsengine_storage_sqlite as storage_sqlite;
pub use smsengine_storage_parity as storage_parity;

// ---- Test infrastructure -------------------------------------------------
pub use smsengine_audit as audit;
pub use smsengine_testkit as testkit;

// ---- High-level SDK ------------------------------------------------------
pub use smsengine_sdk as sdk;

/// SMSengine version, sourced from the package manifest.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude of common types consumers are expected to import.
pub mod prelude {
    pub use smsengine_core::prelude::*;
    pub use smsengine_core::{DomainError, Id, Result, SchoolId, TenantContext, UserId};
    pub use smsengine_events::EventEnvelope;
    pub use smsengine_platform::{School, User};
    pub use smsengine_rbac::Capability;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
