//! # educore-operations
//!
//! Operations domain — backups, jobs, system versions, user logs,
//! runtime maintenance, sidebar.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md`, the domain spec in
//! `docs/specs/operations/`, and `docs/build-plan.md` § "Phase 14"
//! for behavioral details.
//!
//! ## CRITICAL: Two `events` crates are easy to confuse
//!
//! - `crates/cross-cutting/operations/` is **THIS** crate (Phase 14,
//!   the operations domain — backups, jobs, system versions, user
//!   logs, maintenance, sidebar).
//! - `crates/cross-cutting/events/` is the **envelope** crate (Phase
//!   2, locked) — `DomainEvent` trait, `EventEnvelope`, `EventBus`
//!   port.
//! - `crates/cross-cutting/events-domain/` is the **Calendar**
//!   domain (Phase 13) — CalendarEvent, Holiday, Weekend, Incident,
//!   etc.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod aggregate;
pub mod commands;
pub mod entities;
pub mod errors;
pub mod events;
pub mod query;
pub mod repository;
pub mod services;
pub mod value_objects;

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-operations";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude: the public surface of the operations crate.
#[allow(missing_docs)]
pub mod prelude {
    pub use crate::aggregate::{
        Backup, FailedJob, Job, MaintenanceSetting, Sidebar, SystemVersion, UserLog, VersionHistory,
    };
    pub use crate::entities::{
        AuditPartition, BackupFile, BackupRetention, BackupSchedule, BackupStorageRef,
        FailedJobExceptionView, JobAttempt, JobPayloadEnvelope, JobQueueStats, MaintenanceMessage,
        MaintenanceOverride, SidebarEntry, SidebarReorderMap, SidebarRoleBinding, SidebarRoute,
        SystemVersionCapability, SystemVersionFeature, SystemVersionInput, SystemVersionManifest,
        UserLogContext, UserLogInput, UserLogSession, VersionHistoryInput, VersionHistoryNote,
        VersionMigration, VersionMigrationStatus,
    };
    pub use crate::errors::{OperationsDomainError, Result};
    pub use crate::value_objects::{
        AcademicYearRef, AuditPartitionId, BackupFileName, BackupFileType, BackupId,
        BackupLangType, BackupRetentionId, BackupScheduleId, BackupSourceLink, FailedJobConnection,
        FailedJobException, FailedJobId, FailedJobQueue, FailedJobUuid, FileReference,
        HistoryNotes, HistoryReleaseDate, HistoryUrl, HistoryVersion, IpAddress, JobAttemptId,
        JobAttempts, JobAvailableAt, JobId, JobPayload, JobQueue, JobStatus, LoginFailureReason,
        LoginOutcome, MaintenanceApplicableFor, MaintenanceImage, MaintenanceSettingId,
        MaintenanceSubTitle, MaintenanceTitle, PermissionId, RoleId, SidebarActiveStatus,
        SidebarId, SidebarIgnoreFlag, SidebarIsSystemDefined, SidebarLevel, SidebarParentRoute,
        SidebarPosition, SidebarSectionId, SystemVersionId, UserAgent, UserLogId, VersionFeatures,
        VersionHistoryId, VersionName, VersionTitle, WorkerId,
    };
    pub use educore_core::ids::SchoolId;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-operations");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_exports_expected_symbols() {
        let _: Option<crate::aggregate::Backup> = None;
        let _: Option<crate::aggregate::Job> = None;
        let _: Option<crate::aggregate::FailedJob> = None;
        let _: Option<crate::aggregate::SystemVersion> = None;
        let _: Option<crate::aggregate::VersionHistory> = None;
        let _: Option<crate::aggregate::UserLog> = None;
        let _: Option<crate::aggregate::MaintenanceSetting> = None;
        let _: Option<crate::aggregate::Sidebar> = None;
    }
}
