//! # educore-audit
//!
//! The audit crate is the engine's writer for the immutable,
//! append-only `audit_log` table. Every state change in the engine
//! produces exactly one audit row inside the same transaction as
//! the state change itself, and the audit crate owns the retention
//! and redaction policies that govern how those rows age. See
//! `docs/schemas/audit-schema.md` for the full spec.
//!
//! ## Modules
//!
//! - [`writer`] — the [`writer::AuditWriter`] service, the typed
//!   [`writer::AuditAction`] and [`writer::AuditTarget`] enums,
//!   and the [`writer::SENTINEL_TARGET_ID`] constant.
//! - [`query`] — the [`query::AuditQuery`] read-side trait, the
//!   [`query::AuditFilter`] enum, and the [`query::Page`]
//!   pagination struct (limit capped at [`query::MAX_PAGE_LIMIT`]).
//! - [`retention`] — the [`retention::RetentionPolicy`] and the
//!   [`retention::RetentionSweeper`] threshold checker.
//! - [`events`] — the [`events::RetentionSweepDue`] event emitted
//!   when the retention policy is reached.
//! - [`redactor`] — the [`redactor::Redactor`] port trait,
//!   [`redactor::RedactionKind`] enum, and the
//!   [`redactor::DefaultRedactor`] regex/keyword-based
//!   implementation. Wired into the audit writer in a later
//!   phase; ships as a standalone module for now.
//! - [`errors`] — the [`errors::AuditError`] re-export
//!   (alias for [`educore_core::error::DomainError`]).
//!
//! ## Re-exports
//!
//! The storage-port [`AuditLogEntry`](educore_storage::AuditLogEntry)
//! is re-exported under the local name so consumers that depend
//! only on `educore-audit` do not need to also depend on
//! `educore-storage` to construct or read audit rows.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-audit";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The audit error type (alias for [`educore_core::error::DomainError`]).
pub mod errors;

/// The [`events::RetentionSweepDue`] typed event.
pub mod events;

/// The [`query::AuditQuery`] read-side trait, [`query::AuditFilter`]
/// enum, [`query::Page`] pagination struct, and the fully
/// denormalised [`query::AuditRecord`] read shape. Implements the
/// read side of `docs/schemas/audit-schema.md` § 5.
pub mod query;

/// The retention policy and threshold checker.
pub mod retention;

/// The PII / secret redactor port trait, the [`redactor::RedactionKind`]
/// enum, and the [`redactor::DefaultRedactor`] implementation.
pub mod redactor;

/// The [`writer::AuditWriter`] service and the typed action / target enums.
pub mod writer;

/// The [`sink::AuditSink`] port — the trait surface consumers use
/// to write audit log entries. Concrete impls include
/// [`writer::AuditWriter`]. Object-safe; held as `Arc<dyn AuditSink>`.
pub mod sink;

// ---- Re-exports ------------------------------------------------------------

/// The storage-port [`AuditLogEntry`] re-exported so consumers can
/// construct and read audit rows without depending on
/// `educore-storage` directly. The type is **not** redefined: it is
/// the same struct as [`educore_storage::AuditLogEntry`], accessed
/// under the local path `educore_audit::AuditLogEntry`.
pub use educore_storage::AuditLogEntry;

/// Convenience re-exports of the most-used types. Consumers of the
/// audit crate typically `use educore_audit::prelude::*;` once at
/// the top of a file.
pub mod prelude {
    pub use crate::errors::{AuditError, Result};
    pub use crate::events::RetentionSweepDue;
    pub use crate::query::{
        ActorType, AuditFilter, AuditId, AuditQuery, AuditRecord, AuditSource, CommandId, Page,
        ResourceId, ResourceType, MAX_PAGE_LIMIT,
    };
    pub use crate::redactor::{DefaultRedactor, RedactionKind, Redactor};
    pub use crate::retention::{RetentionPolicy, RetentionSweeper};
    pub use crate::writer::{AuditAction, AuditTarget, AuditWriter, SENTINEL_TARGET_ID};
    pub use educore_storage::AuditLogEntry;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-audit");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    use educore_core::ids::Identifier;

    #[test]
    fn audit_log_entry_is_reexported() {
        // Constructing a typed id and calling the re-exported
        // AuditLogEntry::create confirms the type path is wired
        // through without redefinition.
        let school = educore_core::ids::SchoolId::from_uuid(uuid::Uuid::now_v7());
        let user = educore_core::ids::UserId::from_uuid(uuid::Uuid::now_v7());
        let target = uuid::Uuid::now_v7();
        let corr = educore_core::ids::CorrelationId::from_uuid(uuid::Uuid::now_v7());
        let entry = AuditLogEntry::create(
            school,
            user,
            "student",
            target,
            bytes::Bytes::from_static(b"{\"id\":\"x\"}"),
            corr,
        );
        assert_eq!(entry.action, "create");
        assert!(entry.before.is_none());
        assert!(entry.after.is_some());
    }
}
