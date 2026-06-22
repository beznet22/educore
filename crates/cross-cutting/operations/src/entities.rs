//! # educore-operations child entities
//!
//! Per `docs/specs/operations/entities.md`. Embedded values and
//! owned-by-root children for the 8 operations aggregates.

#![allow(missing_docs, dead_code, clippy::all)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{SchoolId, UserId};
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearRef, BackupFileType, BackupLangType, BackupRetentionId, BackupScheduleId,
    BackupSourceLink, FailedJobException, FileReference, HistoryNotes, HistoryReleaseDate,
    HistoryUrl, HistoryVersion, IpAddress, JobQueue, LoginFailureReason, LoginOutcome,
    MaintenanceSubTitle, MaintenanceTitle, PermissionId, RoleId, SidebarActiveStatus,
    SidebarIgnoreFlag, SidebarIsSystemDefined, SidebarLevel, SidebarParentRoute, SidebarPosition,
    SidebarSectionId, UserAgent, VersionFeatures, VersionTitle, WorkerId,
};

// =============================================================================
// BackupFile (typed projection)
// =============================================================================

/// A typed projection of the backup file. Owned by [`Backup`](crate::aggregate::Backup).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupFile {
    pub school_id: SchoolId,
    pub backup_id: Uuid,
    pub file_name: String,
    pub source_link: BackupSourceLink,
    pub file_type: BackupFileType,
    pub lang_type: Option<BackupLangType>,
}

impl BackupFile {
    /// Constructs a new `BackupFile`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        backup_id: Uuid,
        file_name: String,
        source_link: BackupSourceLink,
        file_type: BackupFileType,
        lang_type: Option<BackupLangType>,
    ) -> Self {
        Self {
            school_id,
            backup_id,
            file_name,
            source_link,
            file_type,
            lang_type,
        }
    }
}

// === BackupFile section end ===

// =============================================================================
// BackupSchedule
// =============================================================================

/// A cron-style schedule for automatic backups. Port-driven.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub schedule_id: BackupScheduleId,
    pub school_id: SchoolId,
    pub cron_expression: String,
    pub file_type: BackupFileType,
    pub active_status: bool,
    pub created_at: Timestamp,
}

impl BackupSchedule {
    /// Constructs a new `BackupSchedule`.
    #[must_use]
    pub const fn new(
        schedule_id: BackupScheduleId,
        school_id: SchoolId,
        cron_expression: String,
        file_type: BackupFileType,
        active_status: bool,
        created_at: Timestamp,
    ) -> Self {
        Self {
            schedule_id,
            school_id,
            cron_expression,
            file_type,
            active_status,
            created_at,
        }
    }
}

// === BackupSchedule section end ===

// =============================================================================
// BackupRetention
// =============================================================================

/// A retention policy. Port-driven.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupRetention {
    pub retention_id: BackupRetentionId,
    pub school_id: SchoolId,
    pub keep_count: u32,
    pub max_age_days: u32,
}

impl BackupRetention {
    /// Constructs a new `BackupRetention`.
    #[must_use]
    pub const fn new(
        retention_id: BackupRetentionId,
        school_id: SchoolId,
        keep_count: u32,
        max_age_days: u32,
    ) -> Self {
        Self {
            retention_id,
            school_id,
            keep_count,
            max_age_days,
        }
    }
}

// === BackupRetention section end ===

// =============================================================================
// JobPayload entity (embedded typed view)
// =============================================================================

/// A typed projection of a job's payload (command envelope, target, args).
/// Embedded value owned by [`Job`](crate::aggregate::Job).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobPayloadEnvelope {
    pub command_type: String,
    pub target_aggregate: String,
    pub args_json: String,
}

impl JobPayloadEnvelope {
    /// Constructs a new `JobPayloadEnvelope`.
    #[must_use]
    pub const fn new(command_type: String, target_aggregate: String, args_json: String) -> Self {
        Self {
            command_type,
            target_aggregate,
            args_json,
        }
    }
}

// === JobPayload section end ===

// =============================================================================
// JobAttempt
// =============================================================================

/// A single attempt at running a job. Global id. Owned by
/// [`Job`](crate::aggregate::Job).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobAttempt {
    pub attempt_id: crate::value_objects::JobAttemptId,
    pub job_id: crate::value_objects::JobId,
    pub attempt_number: u8,
    pub worker_id: Option<WorkerId>,
    pub reserved_at: Option<Timestamp>,
    pub completed_at: Option<Timestamp>,
    pub failed_at: Option<Timestamp>,
    pub exception: Option<FailedJobException>,
}

impl JobAttempt {
    /// Constructs a new `JobAttempt`.
    #[must_use]
    pub const fn new(
        attempt_id: crate::value_objects::JobAttemptId,
        job_id: crate::value_objects::JobId,
        attempt_number: u8,
    ) -> Self {
        Self {
            attempt_id,
            job_id,
            attempt_number,
            worker_id: None,
            reserved_at: None,
            completed_at: None,
            failed_at: None,
            exception: None,
        }
    }
}

// === JobAttempt section end ===

// =============================================================================
// FailedJobException (typed projection)
// =============================================================================

/// A typed projection of the captured exception. Owned by
/// [`FailedJob`](crate::aggregate::FailedJob).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedJobExceptionView {
    pub failed_job_id: Uuid,
    pub kind: Option<String>,
    pub message: String,
    pub stack_trace: Option<String>,
    pub root_cause: Option<String>,
}

impl FailedJobExceptionView {
    /// Constructs a new `FailedJobExceptionView`.
    #[must_use]
    pub const fn new(failed_job_id: Uuid, message: String) -> Self {
        Self {
            failed_job_id,
            kind: None,
            message,
            stack_trace: None,
            root_cause: None,
        }
    }
}

// === FailedJobException section end ===

// =============================================================================
// SystemVersionFeature (typed list projection)
// =============================================================================

/// A single feature entry in a `SystemVersion`'s features blurb.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionFeature {
    pub feature_id: Uuid,
    pub version_id: crate::value_objects::SystemVersionId,
    pub title: String,
    pub blurb: String,
}

impl SystemVersionFeature {
    /// Constructs a new `SystemVersionFeature`.
    #[must_use]
    pub const fn new(
        feature_id: Uuid,
        version_id: crate::value_objects::SystemVersionId,
        title: String,
        blurb: String,
    ) -> Self {
        Self {
            feature_id,
            version_id,
            title,
            blurb,
        }
    }
}

// === SystemVersionFeature section end ===

// =============================================================================
// VersionHistoryNote (typed list projection)
// =============================================================================

/// A single note line in a `VersionHistory::notes` blurb.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistoryNote {
    pub note_id: Uuid,
    pub history_id: crate::value_objects::VersionHistoryId,
    pub line: String,
    pub ordering: u32,
}

impl VersionHistoryNote {
    /// Constructs a new `VersionHistoryNote`.
    #[must_use]
    pub const fn new(
        note_id: Uuid,
        history_id: crate::value_objects::VersionHistoryId,
        line: String,
        ordering: u32,
    ) -> Self {
        Self {
            note_id,
            history_id,
            line,
            ordering,
        }
    }
}

// === VersionHistoryNote section end ===

// =============================================================================
// UserLogContext
// =============================================================================

/// A typed projection of the request context that produced a `UserLog`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLogContext {
    pub school_id: SchoolId,
    pub log_id: crate::value_objects::UserLogId,
    pub parsed_ip_v4: Option<String>,
    pub parsed_ip_v6: Option<String>,
    pub user_agent_browser: Option<String>,
    pub user_agent_os: Option<String>,
    pub user_agent_device: Option<String>,
    pub request_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

impl UserLogContext {
    /// Constructs a new `UserLogContext`.
    #[must_use]
    pub const fn new(school_id: SchoolId, log_id: crate::value_objects::UserLogId) -> Self {
        Self {
            school_id,
            log_id,
            parsed_ip_v4: None,
            parsed_ip_v6: None,
            user_agent_browser: None,
            user_agent_os: None,
            user_agent_device: None,
            request_id: None,
            correlation_id: None,
        }
    }
}

// === UserLogContext section end ===

// =============================================================================
// UserLogSession
// =============================================================================

/// A typed projection of the session that a `UserLog` login created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLogSession {
    pub school_id: SchoolId,
    pub session_id: Uuid,
    pub log_id: crate::value_objects::UserLogId,
    pub session_hash: String,
    pub created_at: Timestamp,
    pub invalidated_at: Option<Timestamp>,
}

impl UserLogSession {
    /// Constructs a new `UserLogSession`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        session_id: Uuid,
        log_id: crate::value_objects::UserLogId,
        session_hash: String,
        created_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            session_id,
            log_id,
            session_hash,
            created_at,
            invalidated_at: None,
        }
    }
}

// === UserLogSession section end ===

// =============================================================================
// MaintenanceOverride
// =============================================================================

/// A per-role override for the school's maintenance setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceOverride {
    pub school_id: SchoolId,
    pub setting_id: crate::value_objects::MaintenanceSettingId,
    pub role_id: RoleId,
    pub allowed: bool,
}

impl MaintenanceOverride {
    /// Constructs a new `MaintenanceOverride`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        setting_id: crate::value_objects::MaintenanceSettingId,
        role_id: RoleId,
        allowed: bool,
    ) -> Self {
        Self {
            school_id,
            setting_id,
            role_id,
            allowed,
        }
    }
}

// === MaintenanceOverride section end ===

// =============================================================================
// MaintenanceMessage (embedded typed view)
// =============================================================================

/// A typed projection of the maintenance setting's title + sub_title.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceMessage {
    pub locale: String,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
}

impl MaintenanceMessage {
    /// Constructs a new `MaintenanceMessage`.
    #[must_use]
    pub const fn new(
        locale: String,
        title: MaintenanceTitle,
        sub_title: MaintenanceSubTitle,
    ) -> Self {
        Self {
            locale,
            title,
            sub_title,
        }
    }
}

// === MaintenanceMessage section end ===

// =============================================================================
// SidebarEntry (typed projection)
// =============================================================================

/// A typed projection of a single sidebar entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarEntry {
    pub sidebar_id: crate::value_objects::SidebarId,
    pub school_id: SchoolId,
    pub permission_id: PermissionId,
    pub role_id: RoleId,
    pub position: SidebarPosition,
    pub section_id: SidebarSectionId,
    pub parent: Option<crate::value_objects::SidebarId>,
    pub parent_route: Option<SidebarParentRoute>,
    pub level: SidebarLevel,
    pub label: String,
    pub icon: Option<FileReference>,
    pub route: String,
    pub is_system_defined: SidebarIsSystemDefined,
    pub ignore: SidebarIgnoreFlag,
    pub active_status: SidebarActiveStatus,
    pub user_id: UserId,
}

impl SidebarEntry {
    /// Constructs a new `SidebarEntry`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        sidebar_id: crate::value_objects::SidebarId,
        school_id: SchoolId,
        permission_id: PermissionId,
        role_id: RoleId,
        position: SidebarPosition,
        section_id: SidebarSectionId,
        level: SidebarLevel,
        label: String,
        route: String,
        is_system_defined: SidebarIsSystemDefined,
        ignore: SidebarIgnoreFlag,
        active_status: SidebarActiveStatus,
        user_id: UserId,
    ) -> Self {
        Self {
            sidebar_id,
            school_id,
            permission_id,
            role_id,
            position,
            section_id,
            parent: None,
            parent_route: None,
            level,
            label,
            icon: None,
            route,
            is_system_defined,
            ignore,
            active_status,
            user_id,
        }
    }
}

// === SidebarEntry section end ===

// =============================================================================
// SidebarRoute
// =============================================================================

/// A typed projection of `Sidebar::parent_route`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarRoute {
    pub school_id: SchoolId,
    pub sidebar_id: crate::value_objects::SidebarId,
    pub parent_sidebar_id: Option<crate::value_objects::SidebarId>,
    pub route_id: SidebarParentRoute,
}

impl SidebarRoute {
    /// Constructs a new `SidebarRoute`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        sidebar_id: crate::value_objects::SidebarId,
        parent_sidebar_id: Option<crate::value_objects::SidebarId>,
        route_id: SidebarParentRoute,
    ) -> Self {
        Self {
            school_id,
            sidebar_id,
            parent_sidebar_id,
            route_id,
        }
    }
}

// === SidebarRoute section end ===

// =============================================================================
// JobQueue (logical projection)
// =============================================================================

/// A logical grouping of jobs by queue name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobQueueStats {
    pub school_id: Option<SchoolId>,
    pub queue: JobQueue,
    pub pending_count: u64,
    pub reserved_count: u64,
    pub failed_count: u64,
    pub last_run_at: Option<Timestamp>,
}

impl JobQueueStats {
    /// Constructs a new `JobQueueStats`.
    #[must_use]
    pub const fn new(
        school_id: Option<SchoolId>,
        queue: JobQueue,
        pending_count: u64,
        reserved_count: u64,
        failed_count: u64,
    ) -> Self {
        Self {
            school_id,
            queue,
            pending_count,
            reserved_count,
            failed_count,
            last_run_at: None,
        }
    }
}

// === JobQueue section end ===

// =============================================================================
// BackupStorageRef (embedded typed view)
// =============================================================================

/// A typed projection of `Backup::source_link` (provider + bucket + key).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupStorageRef {
    pub provider: String,
    pub bucket: Option<String>,
    pub key: String,
}

impl BackupStorageRef {
    /// Constructs a new `BackupStorageRef`.
    #[must_use]
    pub const fn new(provider: String, bucket: Option<String>, key: String) -> Self {
        Self {
            provider,
            bucket,
            key,
        }
    }
}

// === BackupStorageRef section end ===

// =============================================================================
// SystemVersionManifest
// =============================================================================

/// A typed projection of a version's deployment manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionManifest {
    pub manifest_id: Uuid,
    pub version_id: crate::value_objects::SystemVersionId,
    pub targets: Vec<String>,
    pub capabilities_introduced: Vec<String>,
    pub migrations_required: Vec<String>,
    pub manifest_json: Option<String>,
}

impl SystemVersionManifest {
    /// Constructs a new `SystemVersionManifest`.
    #[must_use]
    pub const fn new(manifest_id: Uuid, version_id: crate::value_objects::SystemVersionId) -> Self {
        Self {
            manifest_id,
            version_id,
            targets: Vec::new(),
            capabilities_introduced: Vec::new(),
            migrations_required: Vec::new(),
            manifest_json: None,
        }
    }
}

// === SystemVersionManifest section end ===

// =============================================================================
// AuditPartition
// =============================================================================

/// A logical time partition for the `UserLog`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditPartition {
    pub partition_id: crate::value_objects::AuditPartitionId,
    pub school_id: SchoolId,
    pub label: String,
    pub period_start: Timestamp,
    pub period_end: Timestamp,
    pub entry_count: u64,
}

impl AuditPartition {
    /// Constructs a new `AuditPartition`.
    #[must_use]
    pub const fn new(
        partition_id: crate::value_objects::AuditPartitionId,
        school_id: SchoolId,
        label: String,
        period_start: Timestamp,
        period_end: Timestamp,
    ) -> Self {
        Self {
            partition_id,
            school_id,
            label,
            period_start,
            period_end,
            entry_count: 0,
        }
    }
}

// === AuditPartition section end ===

// =============================================================================
// SidebarRoleBinding
// =============================================================================

/// A typed binding between a sidebar entry and a role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarRoleBinding {
    pub school_id: SchoolId,
    pub sidebar_id: crate::value_objects::SidebarId,
    pub role_id: RoleId,
    pub role_name: Option<String>,
    pub capability_requirement: Option<String>,
}

impl SidebarRoleBinding {
    /// Constructs a new `SidebarRoleBinding`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        sidebar_id: crate::value_objects::SidebarId,
        role_id: RoleId,
    ) -> Self {
        Self {
            school_id,
            sidebar_id,
            role_id,
            role_name: None,
            capability_requirement: None,
        }
    }
}

// === SidebarRoleBinding section end ===

// =============================================================================
// SystemVersionCapability
// =============================================================================

/// A capability introduced in a system version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionCapability {
    pub capability_id: Uuid,
    pub version_id: crate::value_objects::SystemVersionId,
    pub capability_name: String,
    pub description: Option<String>,
}

impl SystemVersionCapability {
    /// Constructs a new `SystemVersionCapability`.
    #[must_use]
    pub const fn new(
        capability_id: Uuid,
        version_id: crate::value_objects::SystemVersionId,
        capability_name: String,
    ) -> Self {
        Self {
            capability_id,
            version_id,
            capability_name,
            description: None,
        }
    }
}

// === SystemVersionCapability section end ===

// =============================================================================
// VersionMigration
// =============================================================================

/// A migration script required for a version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionMigration {
    pub migration_id: Uuid,
    pub history_id: crate::value_objects::VersionHistoryId,
    pub script_name: String,
    pub status: VersionMigrationStatus,
    pub batch: i32,
    pub applied_at: Option<Timestamp>,
}

impl VersionMigration {
    /// Constructs a new `VersionMigration`.
    #[must_use]
    pub const fn new(
        migration_id: Uuid,
        history_id: crate::value_objects::VersionHistoryId,
        script_name: String,
        batch: i32,
    ) -> Self {
        Self {
            migration_id,
            history_id,
            script_name,
            status: VersionMigrationStatus::Pending,
            batch,
            applied_at: None,
        }
    }
}

/// Status of a `VersionMigration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionMigrationStatus {
    /// Migration applied successfully.
    Applied,
    /// Migration pending application.
    Pending,
    /// Migration failed.
    Failed,
}

impl VersionMigrationStatus {
    /// Returns the canonical wire name.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Applied => "Applied",
            Self::Pending => "Pending",
            Self::Failed => "Failed",
        }
    }
}

impl std::fmt::Display for VersionMigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// === VersionMigration section end ===

// =============================================================================
// ReorderMap (typed projection for Sidebar::ReorderSidebar)
// =============================================================================

/// A typed map of `SidebarId -> SidebarPosition` used by `ReorderSidebar`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SidebarReorderMap {
    /// The position assignments keyed by sidebar id.
    pub entries: BTreeMap<crate::value_objects::SidebarId, SidebarPosition>,
}

impl SidebarReorderMap {
    /// Constructs a new empty `SidebarReorderMap`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Adds a sidebar id → position mapping.
    #[must_use]
    pub fn with(mut self, id: crate::value_objects::SidebarId, position: SidebarPosition) -> Self {
        self.entries.insert(id, position);
        self
    }
}

// === SidebarReorderMap section end ===

// =============================================================================
// UserLog construction input (high-level DTO)
// =============================================================================

/// A high-level input struct for building a [`UserLog`](crate::aggregate::UserLog).
/// Owned by `UserLog`, used by the service layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLogInput {
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub academic_id: Option<AcademicYearRef>,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub failure_reason: Option<LoginFailureReason>,
}

impl UserLogInput {
    /// Constructs a new `UserLogInput`.
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        user_id: UserId,
        role_id: RoleId,
        ip_address: IpAddress,
        user_agent: UserAgent,
        outcome: LoginOutcome,
    ) -> Self {
        Self {
            school_id,
            user_id,
            role_id,
            academic_id: None,
            ip_address,
            user_agent,
            outcome,
            failure_reason: None,
        }
    }
}

// === UserLogInput section end ===

// =============================================================================
// VersionHistory construction input
// =============================================================================

/// A high-level input struct for building a [`VersionHistory`](crate::aggregate::VersionHistory).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistoryInput {
    pub version: HistoryVersion,
    pub release_date: HistoryReleaseDate,
    pub url: Option<HistoryUrl>,
    pub notes: HistoryNotes,
}

impl VersionHistoryInput {
    /// Constructs a new `VersionHistoryInput`.
    #[must_use]
    pub const fn new(
        version: HistoryVersion,
        release_date: HistoryReleaseDate,
        notes: HistoryNotes,
    ) -> Self {
        Self {
            version,
            release_date,
            url: None,
            notes,
        }
    }
}

// === VersionHistoryInput section end ===

// =============================================================================
// SystemVersion construction input
// =============================================================================

/// A high-level input struct for building a [`SystemVersion`](crate::aggregate::SystemVersion).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionInput {
    pub version_name: crate::value_objects::VersionName,
    pub title: VersionTitle,
    pub features: VersionFeatures,
}

impl SystemVersionInput {
    /// Constructs a new `SystemVersionInput`.
    #[must_use]
    pub const fn new(
        version_name: crate::value_objects::VersionName,
        title: VersionTitle,
        features: VersionFeatures,
    ) -> Self {
        Self {
            version_name,
            title,
            features,
        }
    }
}

// === SystemVersionInput section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn backup_file_constructs() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let f = BackupFile::new(
            school,
            Uuid::nil(),
            "backup.sql".to_owned(),
            BackupSourceLink::new("s3://bucket/backup.sql")?,
            BackupFileType::Database,
            None,
        );
        assert_eq!(f.file_type.as_i32(), 0);
        Ok(())
    }

    #[test]
    fn sidebar_entry_constructs_with_required_fields(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let sidebar_id = crate::value_objects::SidebarId::new(school, Uuid::from_u128(1));
        let permission = PermissionId::new(school, Uuid::from_u128(2));
        let role = RoleId::new(school, Uuid::from_u128(3));
        let entry = SidebarEntry::new(
            sidebar_id,
            school,
            permission,
            role,
            SidebarPosition::new(0)?,
            SidebarSectionId::new(1),
            SidebarLevel::new(1)?,
            "Dashboard".to_owned(),
            "/dashboard".to_owned(),
            SidebarIsSystemDefined::new(false),
            SidebarIgnoreFlag::new(0)?,
            SidebarActiveStatus::new(true),
            UserId::from_uuid(Uuid::from_u128(4)),
        );
        assert_eq!(entry.level.get(), 1);
        assert!(entry.ignore.is_shown());
        Ok(())
    }

    #[test]
    fn sidebar_reorder_map_builds() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let a = crate::value_objects::SidebarId::new(school, Uuid::from_u128(1));
        let b = crate::value_objects::SidebarId::new(school, Uuid::from_u128(2));
        let map = SidebarReorderMap::new()
            .with(a, SidebarPosition::new(0)?)
            .with(b, SidebarPosition::new(1)?);
        assert_eq!(map.entries.len(), 2);
        Ok(())
    }

    #[test]
    fn version_migration_status_display() {
        assert_eq!(VersionMigrationStatus::Applied.to_string(), "Applied");
        assert_eq!(VersionMigrationStatus::Pending.to_string(), "Pending");
        assert_eq!(VersionMigrationStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn user_log_input_builder() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(Uuid::nil());
        let user = UserId::from_uuid(Uuid::nil());
        let role = RoleId::new(school, Uuid::nil());
        let ip = IpAddress::new("192.0.2.1")?;
        let ua = UserAgent::new("Mozilla/5.0")?;
        let input = UserLogInput::new(school, user, role, ip, ua, LoginOutcome::Success);
        assert_eq!(input.school_id, school);
        assert!(matches!(input.outcome, LoginOutcome::Success));
        assert!(input.failure_reason.is_none());
        Ok(())
    }

    #[test]
    fn backup_storage_ref_constructs() {
        let r = BackupStorageRef::new("s3".to_owned(), Some("bucket".to_owned()), "key".to_owned());
        assert_eq!(r.provider, "s3");
    }
}
