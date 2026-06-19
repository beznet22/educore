//! # educore-operations typed commands
//!
//! Per `docs/specs/operations/commands.md`. 28 typed command
//! shapes across 8 aggregates. Every command carries a
//! [`TenantContext`](educore_core::tenant::TenantContext). System
//! commands (ScheduleJob, MarkJobReserved/Completed/Failed,
//! RegisterSystemVersion, RecordVersionHistory, RecordUserLog) use
//! a system tenant context.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearRef, BackupFileName, BackupFileType, BackupLangType, BackupSourceLink,
    FailedJobException, HistoryNotes, HistoryReleaseDate, HistoryUrl, HistoryVersion, IpAddress,
    LoginFailureReason, LoginOutcome, MaintenanceApplicableFor, MaintenanceImage,
    MaintenanceSubTitle, MaintenanceTitle, PermissionId, RoleId, SidebarActiveStatus,
    SidebarIgnoreFlag, SidebarIsSystemDefined, SidebarLevel, SidebarParentRoute, SidebarPosition,
    SidebarSectionId, UserAgent, VersionFeatures, VersionTitle, WorkerId,
};

// =============================================================================
// === Backup commands section begin (owner: B) ===
// =============================================================================

/// Create a new `Backup`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateBackupCommand {
    pub tenant: TenantContext,
    pub file_name: BackupFileName,
    pub source_link: BackupSourceLink,
    pub file_type: BackupFileType,
    pub lang_type: Option<BackupLangType>,
}

impl CreateBackupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.backup.create";
}

/// Delete a `Backup`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteBackupCommand {
    pub tenant: TenantContext,
    pub backup_id: crate::value_objects::BackupId,
}

impl DeleteBackupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.backup.delete";
}

/// Restore from a `Backup`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RestoreBackupCommand {
    pub tenant: TenantContext,
    pub backup_id: crate::value_objects::BackupId,
}

impl RestoreBackupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.backup.restore";
}

/// Mark a `Backup` active.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkBackupActiveCommand {
    pub tenant: TenantContext,
    pub backup_id: crate::value_objects::BackupId,
}

impl MarkBackupActiveCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.backup.mark_active";
}

/// Mark a `Backup` inactive.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkBackupInactiveCommand {
    pub tenant: TenantContext,
    pub backup_id: crate::value_objects::BackupId,
}

impl MarkBackupInactiveCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.backup.mark_inactive";
}

// === Backup commands section end ===

// =============================================================================
// === Job commands section begin (owner: B) ===
// =============================================================================

/// Schedule a new `Job`. System tenant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleJobCommand {
    pub tenant: TenantContext, // system tenant
    pub queue: crate::value_objects::JobQueue,
    pub payload: crate::value_objects::JobPayload,
    pub available_at: Timestamp,
}

impl ScheduleJobCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.job.schedule";

    /// Converts to a `NewJob` aggregate input.
    #[must_use]
    pub fn into_new_job(self, id: crate::value_objects::JobId) -> crate::aggregate::NewJob {
        crate::aggregate::NewJob {
            id,
            queue: self.queue,
            payload: self.payload,
            available_at: self.available_at,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Cancel a `Job`. System tenant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CancelJobCommand {
    pub tenant: TenantContext, // system tenant
    pub job_id: crate::value_objects::JobId,
}

impl CancelJobCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.job.cancel";
}

/// Mark a `Job` as reserved by a worker. System-internal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkJobReservedCommand {
    pub job_id: crate::value_objects::JobId,
    pub worker_id: WorkerId,
}

impl MarkJobReservedCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.job.reserve";
}

/// Mark a `Job` as completed. System-internal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkJobCompletedCommand {
    pub job_id: crate::value_objects::JobId,
}

impl MarkJobCompletedCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.job.complete";
}

/// Mark a `Job` as terminally failed. System-internal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MarkJobFailedCommand {
    pub job_id: crate::value_objects::JobId,
    pub exception: FailedJobException,
}

impl MarkJobFailedCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.job.fail";
}

// === Job commands section end ===

// =============================================================================
// === FailedJob commands section begin (owner: B) ===
// =============================================================================

/// Record a `FailedJob`. System-internal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordFailedJobCommand {
    pub job_id: crate::value_objects::JobId,
    pub exception: FailedJobException,
}

impl RecordFailedJobCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.failed_job.record";
}

/// Retry a `FailedJob`. System tenant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RetryFailedJobCommand {
    pub tenant: TenantContext, // system tenant
    pub failed_job_id: crate::value_objects::FailedJobId,
}

impl RetryFailedJobCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.failed_job.retry";
}

/// Delete a `FailedJob`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteFailedJobCommand {
    pub tenant: TenantContext,
    pub failed_job_id: crate::value_objects::FailedJobId,
}

impl DeleteFailedJobCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.failed_job.delete";
}

// === FailedJob commands section end ===

// =============================================================================
// === SystemVersion commands section begin (owner: B) ===
// =============================================================================

/// Register a new `SystemVersion`. System tenant, build-time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RegisterSystemVersionCommand {
    pub tenant: TenantContext, // system tenant
    pub version_name: crate::value_objects::VersionName,
    pub title: VersionTitle,
    pub features: VersionFeatures,
}

impl RegisterSystemVersionCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.system_version.register";

    /// Converts to a `NewSystemVersion` aggregate input.
    #[must_use]
    pub fn into_new_version(
        self,
        id: crate::value_objects::SystemVersionId,
    ) -> crate::aggregate::NewSystemVersion {
        let now = Timestamp::now();
        crate::aggregate::NewSystemVersion {
            id,
            version_name: self.version_name,
            title: self.title,
            features: self.features,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a registered `SystemVersion`. System tenant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateSystemVersionCommand {
    pub tenant: TenantContext, // system tenant
    pub version_id: crate::value_objects::SystemVersionId,
    pub title: Option<VersionTitle>,
    pub features: Option<VersionFeatures>,
}

impl UpdateSystemVersionCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.system_version.update";
}

// === SystemVersion commands section end ===

// =============================================================================
// === VersionHistory commands section begin (owner: B) ===
// =============================================================================

/// Record a `VersionHistory` row. System tenant, build-time.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordVersionHistoryCommand {
    pub tenant: TenantContext, // system tenant
    pub version: HistoryVersion,
    pub release_date: HistoryReleaseDate,
    pub url: Option<HistoryUrl>,
    pub notes: HistoryNotes,
}

impl RecordVersionHistoryCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.version_history.record";

    /// Converts to a `VersionHistoryInput` for the aggregate.
    #[must_use]
    pub fn into_input(self) -> crate::entities::VersionHistoryInput {
        crate::entities::VersionHistoryInput {
            version: self.version,
            release_date: self.release_date,
            url: self.url,
            notes: self.notes,
        }
    }
}

// === VersionHistory commands section end ===

// =============================================================================
// === UserLog commands section begin (owner: B) ===
// =============================================================================

/// Record a `UserLog` row. System tenant.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecordUserLogCommand {
    pub tenant: TenantContext, // system tenant
    pub user_id: educore_core::ids::UserId,
    pub role_id: RoleId,
    pub academic_id: Option<AcademicYearRef>,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub failure_reason: Option<LoginFailureReason>,
}

impl RecordUserLogCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.user_log.record";

    /// Converts to a `UserLogInput` for the aggregate.
    #[must_use]
    pub fn into_input(self) -> crate::entities::UserLogInput {
        crate::entities::UserLogInput {
            school_id: self.tenant.school_id,
            user_id: self.user_id,
            role_id: self.role_id,
            academic_id: self.academic_id,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            outcome: self.outcome,
            failure_reason: self.failure_reason,
        }
    }
}

// === UserLog commands section end ===

// =============================================================================
// === Maintenance commands section begin (owner: B) ===
// =============================================================================

/// Configure (create or update) the school's `MaintenanceSetting`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConfigureMaintenanceCommand {
    pub tenant: TenantContext,
    pub title: Option<MaintenanceTitle>,
    pub sub_title: Option<MaintenanceSubTitle>,
    pub image: Option<MaintenanceImage>,
    pub applicable_for: Option<MaintenanceApplicableFor>,
}

impl ConfigureMaintenanceCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.maintenance.configure";
}

/// Enable maintenance mode for the school.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnableMaintenanceCommand {
    pub tenant: TenantContext,
}

impl EnableMaintenanceCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.maintenance.enable";
}

/// Disable maintenance mode for the school.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DisableMaintenanceCommand {
    pub tenant: TenantContext,
}

impl DisableMaintenanceCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.maintenance.disable";
}

// === Maintenance commands section end ===

// =============================================================================
// === Sidebar commands section begin (owner: B) ===
// =============================================================================

/// Create a new `Sidebar` entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateSidebarEntryCommand {
    pub tenant: TenantContext,
    pub permission_id: PermissionId,
    pub position: SidebarPosition,
    pub section_id: SidebarSectionId,
    pub parent: Option<crate::value_objects::SidebarId>,
    pub parent_route: Option<SidebarParentRoute>,
    pub level: SidebarLevel,
    pub role_id: RoleId,
    pub is_system_defined: SidebarIsSystemDefined,
    pub ignore: SidebarIgnoreFlag,
    pub active_status: SidebarActiveStatus,
    pub user_id: educore_core::ids::UserId,
}

impl CreateSidebarEntryCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.sidebar.create";

    /// Converts to a `NewSidebar` aggregate input.
    #[must_use]
    pub fn into_new_sidebar(
        self,
        id: crate::value_objects::SidebarId,
    ) -> crate::aggregate::NewSidebar {
        let now = Timestamp::now();
        crate::aggregate::NewSidebar {
            id,
            permission_id: self.permission_id,
            role_id: self.role_id,
            position: self.position,
            section_id: self.section_id,
            parent: self.parent,
            parent_route: self.parent_route,
            level: self.level,
            is_system_defined: self.is_system_defined,
            ignore: self.ignore,
            active_status: self.active_status,
            user_id: self.user_id,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a `Sidebar` entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateSidebarEntryCommand {
    pub tenant: TenantContext,
    pub sidebar_id: crate::value_objects::SidebarId,
    pub position: Option<SidebarPosition>,
    pub level: Option<SidebarLevel>,
    pub ignore: Option<SidebarIgnoreFlag>,
    pub active_status: Option<SidebarActiveStatus>,
}

impl UpdateSidebarEntryCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.sidebar.update";
}

/// Delete a `Sidebar` entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteSidebarEntryCommand {
    pub tenant: TenantContext,
    pub sidebar_id: crate::value_objects::SidebarId,
}

impl DeleteSidebarEntryCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.sidebar.delete";
}

/// Reorder `Sidebar` entries within a role.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReorderSidebarCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub new_positions: crate::entities::SidebarReorderMap,
}

impl ReorderSidebarCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "operations.sidebar.reorder";
}

// === Sidebar commands section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::{CorrelationId, Identifier, SchoolId};
    use educore_core::tenant::UserType;
    use uuid::Uuid;

    #[test]
    fn command_types_have_wire_form() {
        // Backup (5)
        assert_eq!(
            CreateBackupCommand::COMMAND_TYPE,
            "operations.backup.create"
        );
        assert_eq!(
            DeleteBackupCommand::COMMAND_TYPE,
            "operations.backup.delete"
        );
        assert_eq!(
            RestoreBackupCommand::COMMAND_TYPE,
            "operations.backup.restore"
        );
        assert_eq!(
            MarkBackupActiveCommand::COMMAND_TYPE,
            "operations.backup.mark_active"
        );
        assert_eq!(
            MarkBackupInactiveCommand::COMMAND_TYPE,
            "operations.backup.mark_inactive"
        );
        // Job (5)
        assert_eq!(ScheduleJobCommand::COMMAND_TYPE, "operations.job.schedule");
        assert_eq!(CancelJobCommand::COMMAND_TYPE, "operations.job.cancel");
        assert_eq!(
            MarkJobReservedCommand::COMMAND_TYPE,
            "operations.job.reserve"
        );
        assert_eq!(
            MarkJobCompletedCommand::COMMAND_TYPE,
            "operations.job.complete"
        );
        assert_eq!(MarkJobFailedCommand::COMMAND_TYPE, "operations.job.fail");
        // FailedJob (3)
        assert_eq!(
            RecordFailedJobCommand::COMMAND_TYPE,
            "operations.failed_job.record"
        );
        assert_eq!(
            RetryFailedJobCommand::COMMAND_TYPE,
            "operations.failed_job.retry"
        );
        assert_eq!(
            DeleteFailedJobCommand::COMMAND_TYPE,
            "operations.failed_job.delete"
        );
        // SystemVersion (2)
        assert_eq!(
            RegisterSystemVersionCommand::COMMAND_TYPE,
            "operations.system_version.register"
        );
        assert_eq!(
            UpdateSystemVersionCommand::COMMAND_TYPE,
            "operations.system_version.update"
        );
        // VersionHistory (1)
        assert_eq!(
            RecordVersionHistoryCommand::COMMAND_TYPE,
            "operations.version_history.record"
        );
        // UserLog (1)
        assert_eq!(
            RecordUserLogCommand::COMMAND_TYPE,
            "operations.user_log.record"
        );
        // Maintenance (3)
        assert_eq!(
            ConfigureMaintenanceCommand::COMMAND_TYPE,
            "operations.maintenance.configure"
        );
        assert_eq!(
            EnableMaintenanceCommand::COMMAND_TYPE,
            "operations.maintenance.enable"
        );
        assert_eq!(
            DisableMaintenanceCommand::COMMAND_TYPE,
            "operations.maintenance.disable"
        );
        // Sidebar (4)
        assert_eq!(
            CreateSidebarEntryCommand::COMMAND_TYPE,
            "operations.sidebar.create"
        );
        assert_eq!(
            UpdateSidebarEntryCommand::COMMAND_TYPE,
            "operations.sidebar.update"
        );
        assert_eq!(
            DeleteSidebarEntryCommand::COMMAND_TYPE,
            "operations.sidebar.delete"
        );
        assert_eq!(
            ReorderSidebarCommand::COMMAND_TYPE,
            "operations.sidebar.reorder"
        );
    }

    #[test]
    fn schedule_job_into_new_job() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let user = educore_core::ids::UserId::from_uuid(Uuid::nil());
        let corr = CorrelationId::from_uuid(Uuid::nil());
        let tenant = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);
        let cmd = ScheduleJobCommand {
            tenant,
            queue: crate::value_objects::JobQueue::new("default").unwrap(),
            payload: crate::value_objects::JobPayload::new("{}").unwrap(),
            available_at: Timestamp::now(),
        };
        let id = crate::value_objects::JobId::new(Uuid::nil());
        let new = cmd.into_new_job(id);
        assert_eq!(new.queue.as_str(), "default");
    }
}
