//! # educore-operations aggregate roots
//!
//! The 8 root aggregates per `docs/specs/operations/aggregates.md`:
//! Backup, Job, FailedJob, SystemVersion, VersionHistory, UserLog,
//! MaintenanceSetting, Sidebar. Job, FailedJob, SystemVersion, and
//! VersionHistory are global (no `school_id`); the others are
//! tenant-scoped.

#![allow(missing_docs, dead_code, clippy::all)]

use serde::{Deserialize, Serialize};

#[cfg(test)]
use educore_core::ids::Identifier;
use educore_core::ids::{CorrelationId, EventId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};
#[cfg(test)]
use uuid::Uuid;

use crate::entities::{UserLogInput, VersionHistoryInput};
use crate::errors::OperationsDomainError;
use crate::value_objects::{
    AcademicYearRef, BackupFileName, BackupFileType, BackupLangType, BackupSourceLink,
    FailedJobConnection, FailedJobException, FailedJobQueue, FailedJobUuid, HistoryNotes,
    HistoryReleaseDate, HistoryUrl, HistoryVersion, IpAddress, JobAttempts, JobPayload, JobQueue,
    LoginFailureReason, LoginOutcome, MaintenanceApplicableFor, MaintenanceImage,
    MaintenanceSubTitle, MaintenanceTitle, PermissionId, RoleId, SidebarActiveStatus,
    SidebarIgnoreFlag, SidebarIsSystemDefined, SidebarLevel, SidebarParentRoute, SidebarPosition,
    SidebarSectionId, UserAgent, VersionFeatures, VersionTitle, WorkerId,
};

/// Result alias for aggregate constructors.
pub type AggregateResult<T> = std::result::Result<T, OperationsDomainError>;

// =============================================================================
// === Backup section begin (owner: B) ===
// =============================================================================

/// `Backup` — a per-school backup record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Backup {
    pub id: crate::value_objects::BackupId,
    pub school_id: educore_core::ids::SchoolId,
    pub file_name: BackupFileName,
    pub source_link: BackupSourceLink,
    pub file_type: BackupFileType,
    pub lang_type: Option<BackupLangType>,
    pub active_status: bool,
    pub restore_in_progress: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Backup::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewBackup {
    pub id: crate::value_objects::BackupId,
    pub file_name: BackupFileName,
    pub source_link: BackupSourceLink,
    pub file_type: BackupFileType,
    pub lang_type: Option<BackupLangType>,
    pub active_status: bool,
    pub restore_in_progress: bool,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Backup {
    /// Constructs a new `Backup`.
    pub fn new(cmd: NewBackup) -> AggregateResult<Self> {
        if !cmd.active_status && cmd.restore_in_progress {
            return Err(OperationsDomainError::Validation(
                "inactive backup cannot have restore in progress".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            file_name: cmd.file_name,
            source_link: cmd.source_link,
            file_type: cmd.file_type,
            lang_type: cmd.lang_type,
            active_status: cmd.active_status,
            restore_in_progress: cmd.restore_in_progress,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Marks this backup as currently being restored.
    pub fn mark_restoring(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.restore_in_progress = true;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Marks this backup as no longer being restored (post-restore).
    pub fn clear_restoring(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.restore_in_progress = false;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Marks this backup active.
    pub fn mark_active(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = true;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Marks this backup inactive.
    pub fn mark_inactive(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = false;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }
}

// === Backup section end ===

// =============================================================================
// === Job section begin (owner: B) ===
// =============================================================================

/// `Job` — a pending job in the queue. Global (no `school_id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub id: crate::value_objects::JobId,
    pub queue: JobQueue,
    pub payload: JobPayload,
    pub attempts: JobAttempts,
    pub available_at: Timestamp,
    pub reserved_at: Option<Timestamp>,
    pub worker_id: Option<WorkerId>,
    pub status: crate::value_objects::JobStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Job::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewJob {
    pub id: crate::value_objects::JobId,
    pub queue: JobQueue,
    pub payload: JobPayload,
    pub available_at: Timestamp,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Job {
    /// Constructs a new `Job`.
    pub fn new(cmd: NewJob) -> AggregateResult<Self> {
        if matches!(cmd.payload.0.as_str(), "") {
            return Err(OperationsDomainError::Validation(
                "job payload must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            id: cmd.id,
            queue: cmd.queue,
            payload: cmd.payload,
            attempts: JobAttempts::new(0),
            available_at: cmd.available_at,
            reserved_at: None,
            worker_id: None,
            status: crate::value_objects::JobStatus::Pending,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Reserves the job for the given worker.
    pub fn reserve(
        &mut self,
        worker: WorkerId,
        at: Timestamp,
        event_id: EventId,
    ) -> AggregateResult<()> {
        if matches!(self.status, crate::value_objects::JobStatus::Reserved) {
            return Err(OperationsDomainError::Conflict(
                "job is already reserved".to_owned(),
            ));
        }
        if matches!(self.status, crate::value_objects::JobStatus::Completed) {
            return Err(OperationsDomainError::Conflict(
                "job already completed".to_owned(),
            ));
        }
        self.status = crate::value_objects::JobStatus::Reserved;
        self.reserved_at = Some(at);
        self.worker_id = Some(worker);
        self.attempts = JobAttempts::new(self.attempts.0.saturating_add(1));
        self.updated_at = at;
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Marks the job as completed.
    pub fn complete(&mut self, at: Timestamp, event_id: EventId) -> AggregateResult<()> {
        if !matches!(self.status, crate::value_objects::JobStatus::Reserved) {
            return Err(OperationsDomainError::Conflict(
                "only a reserved job can be completed".to_owned(),
            ));
        }
        self.status = crate::value_objects::JobStatus::Completed;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Marks the job as terminally failed.
    pub fn fail(&mut self, at: Timestamp, event_id: EventId) -> AggregateResult<()> {
        if !matches!(self.status, crate::value_objects::JobStatus::Reserved) {
            return Err(OperationsDomainError::Conflict(
                "only a reserved job can be failed".to_owned(),
            ));
        }
        self.status = crate::value_objects::JobStatus::Failed;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Cancels the job (only valid in `Pending` state).
    pub fn cancel(&mut self, at: Timestamp, event_id: EventId) -> AggregateResult<()> {
        if !matches!(self.status, crate::value_objects::JobStatus::Pending) {
            return Err(OperationsDomainError::Conflict(
                "only a pending job can be cancelled".to_owned(),
            ));
        }
        self.status = crate::value_objects::JobStatus::Completed;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
        Ok(())
    }
}

// === Job section end ===

// =============================================================================
// === FailedJob section begin (owner: B) ===
// =============================================================================

/// `FailedJob` — a terminal record of a job that has exhausted its
/// retry budget. Global (no `school_id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedJob {
    pub id: crate::value_objects::FailedJobId,
    pub uuid: FailedJobUuid,
    pub connection: FailedJobConnection,
    pub queue: FailedJobQueue,
    pub payload: JobPayload,
    pub exception: FailedJobException,
    pub original_job_id: crate::value_objects::JobId,
    pub failed_at: Timestamp,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`FailedJob::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFailedJob {
    pub id: crate::value_objects::FailedJobId,
    pub uuid: FailedJobUuid,
    pub connection: FailedJobConnection,
    pub queue: FailedJobQueue,
    pub payload: JobPayload,
    pub exception: FailedJobException,
    pub original_job_id: crate::value_objects::JobId,
    pub failed_at: Timestamp,
    pub created_by: UserId,
    pub correlation_id: CorrelationId,
}

impl FailedJob {
    /// Constructs a new `FailedJob`.
    pub fn new(cmd: NewFailedJob) -> AggregateResult<Self> {
        Ok(Self {
            id: cmd.id,
            uuid: cmd.uuid,
            connection: cmd.connection,
            queue: cmd.queue,
            payload: cmd.payload,
            exception: cmd.exception,
            original_job_id: cmd.original_job_id,
            failed_at: cmd.failed_at,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.failed_at,
            updated_at: cmd.failed_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }
}

// === FailedJob section end ===

// =============================================================================
// === SystemVersion section begin (owner: B) ===
// =============================================================================

/// `SystemVersion` — a released version metadata record. Global.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersion {
    pub id: crate::value_objects::SystemVersionId,
    pub version_name: crate::value_objects::VersionName,
    pub title: VersionTitle,
    pub features: VersionFeatures,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`SystemVersion::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSystemVersion {
    pub id: crate::value_objects::SystemVersionId,
    pub version_name: crate::value_objects::VersionName,
    pub title: VersionTitle,
    pub features: VersionFeatures,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl SystemVersion {
    /// Constructs a new `SystemVersion`.
    pub fn new(cmd: NewSystemVersion) -> AggregateResult<Self> {
        if cmd.title.as_str().trim().is_empty() {
            return Err(OperationsDomainError::Validation(
                "system_version title must not be empty".to_owned(),
            ));
        }
        if cmd.features.as_str().trim().is_empty() {
            return Err(OperationsDomainError::Validation(
                "system_version features must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            id: cmd.id,
            version_name: cmd.version_name,
            title: cmd.title,
            features: cmd.features,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the `title` and/or `features` of a registered version.
    pub fn update(
        &mut self,
        title: Option<VersionTitle>,
        features: Option<VersionFeatures>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> AggregateResult<()> {
        if let Some(t) = title {
            if t.as_str().trim().is_empty() {
                return Err(OperationsDomainError::Validation(
                    "system_version title must not be empty".to_owned(),
                ));
            }
            self.title = t;
        }
        if let Some(f) = features {
            if f.as_str().trim().is_empty() {
                return Err(OperationsDomainError::Validation(
                    "system_version features must not be empty".to_owned(),
                ));
            }
            self.features = f;
        }
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
        Ok(())
    }
}

// === SystemVersion section end ===

// =============================================================================
// === VersionHistory section begin (owner: B) ===
// =============================================================================

/// `VersionHistory` — an append-only record of version bumps.
/// Global.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistory {
    pub id: crate::value_objects::VersionHistoryId,
    pub version: HistoryVersion,
    pub release_date: HistoryReleaseDate,
    pub url: Option<HistoryUrl>,
    pub notes: HistoryNotes,
    pub version_: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub created_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl VersionHistory {
    /// Constructs a new `VersionHistory` row. The aggregate is
    /// append-only: there is no `update` method (per spec invariant
    /// 5 — `VersionHistory` rows are append-only).
    #[must_use]
    pub fn new(
        id: crate::value_objects::VersionHistoryId,
        input: VersionHistoryInput,
        created_by: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            id,
            version: input.version,
            release_date: input.release_date,
            url: input.url,
            notes: input.notes,
            version_: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            created_by,
            last_event_id: None,
            correlation_id,
        }
    }
}

// === VersionHistory section end ===

// =============================================================================
// === UserLog section begin (owner: B) ===
// =============================================================================

/// `UserLog` — a per-login audit record. Append-only (per spec
/// invariant 8 — `UserLog` rows are never updated).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLog {
    pub id: crate::value_objects::UserLogId,
    pub school_id: educore_core::ids::SchoolId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub academic_id: Option<AcademicYearRef>,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub failure_reason: Option<LoginFailureReason>,
    pub logged_at: Timestamp,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub created_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl UserLog {
    /// Constructs a new `UserLog` row. The aggregate is
    /// append-only: there is no `update` method (per spec
    /// invariant 8).
    #[must_use]
    pub fn new(
        id: crate::value_objects::UserLogId,
        input: UserLogInput,
        created_by: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            id,
            school_id: input.school_id,
            user_id: input.user_id,
            role_id: input.role_id,
            academic_id: input.academic_id,
            ip_address: input.ip_address,
            user_agent: input.user_agent,
            outcome: input.outcome,
            failure_reason: input.failure_reason,
            logged_at: at,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            created_by,
            last_event_id: None,
            correlation_id,
        }
    }
}

// === UserLog section end ===

// =============================================================================
// === MaintenanceSetting section begin (owner: B) ===
// =============================================================================

/// `MaintenanceSetting` — per-school singleton maintenance mode
/// configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceSetting {
    pub id: crate::value_objects::MaintenanceSettingId,
    pub school_id: educore_core::ids::SchoolId,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
    pub image: Option<MaintenanceImage>,
    pub applicable_for: MaintenanceApplicableFor,
    pub maintenance_mode: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`MaintenanceSetting::configure`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewMaintenanceSetting {
    pub id: crate::value_objects::MaintenanceSettingId,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
    pub image: Option<MaintenanceImage>,
    pub applicable_for: MaintenanceApplicableFor,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl MaintenanceSetting {
    /// Constructs a new per-school `MaintenanceSetting` singleton.
    pub fn configure(cmd: NewMaintenanceSetting) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            sub_title: cmd.sub_title,
            image: cmd.image,
            applicable_for: cmd.applicable_for,
            maintenance_mode: false,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Reconfigures the maintenance setting's title/subtitle/image/applicability.
    pub fn reconfigure(
        &mut self,
        title: Option<MaintenanceTitle>,
        sub_title: Option<MaintenanceSubTitle>,
        image: Option<Option<MaintenanceImage>>,
        applicable_for: Option<MaintenanceApplicableFor>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(s) = sub_title {
            self.sub_title = s;
        }
        if let Some(i) = image {
            self.image = i;
        }
        if let Some(a) = applicable_for {
            self.applicable_for = a;
        }
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Enables maintenance mode.
    pub fn enable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.maintenance_mode = true;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Disables maintenance mode.
    pub fn disable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.maintenance_mode = false;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }
}

// === MaintenanceSetting section end ===

// =============================================================================
// === Sidebar section begin (owner: B) ===
// =============================================================================

/// `Sidebar` — a per-role sidebar layout projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sidebar {
    pub id: crate::value_objects::SidebarId,
    pub school_id: educore_core::ids::SchoolId,
    pub permission_id: PermissionId,
    pub role_id: RoleId,
    pub position: SidebarPosition,
    pub section_id: SidebarSectionId,
    pub parent: Option<crate::value_objects::SidebarId>,
    pub parent_route: Option<SidebarParentRoute>,
    pub level: SidebarLevel,
    pub is_system_defined: SidebarIsSystemDefined,
    pub ignore: SidebarIgnoreFlag,
    pub active_status: SidebarActiveStatus,
    pub user_id: UserId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Sidebar::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSidebar {
    pub id: crate::value_objects::SidebarId,
    pub permission_id: PermissionId,
    pub role_id: RoleId,
    pub position: SidebarPosition,
    pub section_id: SidebarSectionId,
    pub parent: Option<crate::value_objects::SidebarId>,
    pub parent_route: Option<SidebarParentRoute>,
    pub level: SidebarLevel,
    pub is_system_defined: SidebarIsSystemDefined,
    pub ignore: SidebarIgnoreFlag,
    pub active_status: SidebarActiveStatus,
    pub user_id: UserId,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Sidebar {
    /// Constructs a new `Sidebar` entry.
    pub fn new(cmd: NewSidebar) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            permission_id: cmd.permission_id,
            role_id: cmd.role_id,
            position: cmd.position,
            section_id: cmd.section_id,
            parent: cmd.parent,
            parent_route: cmd.parent_route,
            level: cmd.level,
            is_system_defined: cmd.is_system_defined,
            ignore: cmd.ignore,
            active_status: cmd.active_status,
            user_id: cmd.user_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Returns true if this is a system-defined entry (cannot be deleted).
    #[must_use]
    pub const fn is_system(&self) -> bool {
        self.is_system_defined.0
    }

    /// Reorders this sidebar entry to a new position.
    pub fn reorder(
        &mut self,
        new_position: SidebarPosition,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.position = new_position;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Updates the `ignore` flag.
    pub fn set_ignore(
        &mut self,
        ignore: SidebarIgnoreFlag,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.ignore = ignore;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }

    /// Toggles `active_status`.
    pub fn set_active(
        &mut self,
        active: SidebarActiveStatus,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.active_status = active;
        self.updated_by = actor;
        self.updated_at = at;
        self.last_event_id = Some(event_id);
    }
}

// === Sidebar section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value_objects::{
        BackupSourceLink, FailedJobConnection, FailedJobException, FailedJobQueue, FailedJobUuid,
        FileReference, HistoryNotes, HistoryReleaseDate, HistoryVersion, IpAddress, JobPayload,
        JobQueue, MaintenanceApplicableFor, MaintenanceSubTitle, MaintenanceTitle, PermissionId,
        RoleId, UserAgent, VersionFeatures, VersionTitle, WorkerId,
    };
    use educore_core::ids::Identifier;

    fn make_school() -> educore_core::ids::SchoolId {
        educore_core::ids::SchoolId::from_uuid(Uuid::nil())
    }

    #[test]
    fn backup_new_constructs() {
        let school = make_school();
        let id = crate::value_objects::BackupId::new(school, Uuid::nil());
        let b = Backup::new(NewBackup {
            id,
            file_name: BackupFileName::new("backup.sql").unwrap(),
            source_link: BackupSourceLink::new("s3://bucket/backup.sql").unwrap(),
            file_type: BackupFileType::Database,
            lang_type: None,
            active_status: true,
            restore_in_progress: false,
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert_eq!(b.school_id, school);
        assert!(b.active_status);
        assert!(!b.restore_in_progress);
    }
    #[test]
    fn job_lifecycle_pending_reserved_completed() {
        let id = crate::value_objects::JobId::new(Uuid::nil());
        let mut job = Job::new(NewJob {
            id,
            queue: JobQueue::new("default").unwrap(),
            payload: JobPayload::new("{}").unwrap(),
            available_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert_eq!(job.attempts.0, 0);
        let event_id = EventId::from_uuid(Uuid::from_u128(1));
        job.reserve(
            WorkerId::new("worker-1").unwrap(),
            Timestamp::now(),
            event_id,
        )
        .unwrap();
        assert_eq!(job.attempts.0, 1);
        assert!(matches!(
            job.status,
            crate::value_objects::JobStatus::Reserved
        ));
        job.complete(Timestamp::now(), event_id).unwrap();
        assert!(matches!(
            job.status,
            crate::value_objects::JobStatus::Completed
        ));
    }

    #[test]
    fn job_cannot_complete_unreserved() {
        let id = crate::value_objects::JobId::new(Uuid::nil());
        let mut job = Job::new(NewJob {
            id,
            queue: JobQueue::new("default").unwrap(),
            payload: JobPayload::new("{}").unwrap(),
            available_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        let event_id = EventId::from_uuid(Uuid::from_u128(1));
        let err = job.complete(Timestamp::now(), event_id).unwrap_err();
        assert!(matches!(err, OperationsDomainError::Conflict(_)));
    }

    #[test]
    fn system_version_new_validates() {
        let id = crate::value_objects::SystemVersionId::new(Uuid::nil());
        let ok = SystemVersion::new(NewSystemVersion {
            id,
            version_name: crate::value_objects::VersionName::new("8.2.3").unwrap(),
            title: VersionTitle::new("8.2.3").unwrap(),
            features: VersionFeatures::new("Initial release").unwrap(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        });
        assert!(ok.is_ok());
    }

    #[test]
    fn version_history_is_append_only() {
        let id = crate::value_objects::VersionHistoryId::new(Uuid::nil());
        let h = VersionHistory::new(
            id,
            VersionHistoryInput::new(
                HistoryVersion::new("8.2.3").unwrap(),
                HistoryReleaseDate::new("2026-06-13").unwrap(),
                HistoryNotes::new("Initial release").unwrap(),
            ),
            UserId::from_uuid(Uuid::nil()),
            CorrelationId::from_uuid(Uuid::nil()),
            Timestamp::now(),
        );
        // No update method exists; we just check it constructed.
        assert_eq!(h.version, HistoryVersion::new("8.2.3").unwrap());
    }

    #[test]
    fn user_log_records_login() {
        let school = make_school();
        let id = crate::value_objects::UserLogId::new(school, Uuid::nil());
        let log = UserLog::new(
            id,
            UserLogInput::new(
                school,
                UserId::from_uuid(Uuid::nil()),
                RoleId::new(school, Uuid::nil()),
                IpAddress::new("192.0.2.1").unwrap(),
                UserAgent::new("Mozilla/5.0").unwrap(),
                LoginOutcome::Success,
            ),
            UserId::from_uuid(Uuid::nil()),
            CorrelationId::from_uuid(Uuid::nil()),
            Timestamp::now(),
        );
        assert!(matches!(log.outcome, LoginOutcome::Success));
        assert_eq!(log.ip_address.as_str(), "192.0.2.1");
    }

    #[test]
    fn maintenance_setting_enable_disable() {
        let school = make_school();
        let id = crate::value_objects::MaintenanceSettingId::new(school, Uuid::nil());
        let mut m = MaintenanceSetting::configure(NewMaintenanceSetting {
            id,
            title: MaintenanceTitle::new("We will be back soon!").unwrap(),
            sub_title: MaintenanceSubTitle::new("Sorry for the inconvenience...").unwrap(),
            image: None,
            applicable_for: MaintenanceApplicableFor::new("all").unwrap(),
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert!(!m.maintenance_mode);
        m.enable(
            UserId::from_uuid(Uuid::nil()),
            Timestamp::now(),
            EventId::from_uuid(Uuid::nil()),
        );
        assert!(m.maintenance_mode);
        m.disable(
            UserId::from_uuid(Uuid::nil()),
            Timestamp::now(),
            EventId::from_uuid(Uuid::nil()),
        );
        assert!(!m.maintenance_mode);
    }

    #[test]
    fn sidebar_new_constructs() {
        let school = make_school();
        let id = crate::value_objects::SidebarId::new(school, Uuid::from_u128(1));
        let s = Sidebar::new(NewSidebar {
            id,
            permission_id: PermissionId::new(school, Uuid::from_u128(2)),
            role_id: RoleId::new(school, Uuid::from_u128(3)),
            position: SidebarPosition::new(0).unwrap(),
            section_id: SidebarSectionId::new(1),
            parent: None,
            parent_route: None,
            level: SidebarLevel::new(1).unwrap(),
            is_system_defined: SidebarIsSystemDefined::new(false),
            ignore: SidebarIgnoreFlag::new(0).unwrap(),
            active_status: SidebarActiveStatus::new(true),
            user_id: UserId::from_uuid(Uuid::from_u128(4)),
            created_by: UserId::from_uuid(Uuid::from_u128(4)),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert!(!s.is_system());
        assert_eq!(s.level.get(), 1);
    }

    #[test]
    fn failed_job_new_constructs() {
        let id = crate::value_objects::FailedJobId::new(Uuid::nil());
        let f = FailedJob::new(NewFailedJob {
            id,
            uuid: FailedJobUuid::new(Uuid::from_u128(1)),
            connection: FailedJobConnection::new("database").unwrap(),
            queue: FailedJobQueue::new("default").unwrap(),
            payload: JobPayload::new("{}").unwrap(),
            exception: FailedJobException::new("RuntimeError: kaboom").unwrap(),
            original_job_id: crate::value_objects::JobId::new(Uuid::from_u128(2)),
            failed_at: Timestamp::now(),
            created_by: UserId::from_uuid(Uuid::nil()),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        assert_eq!(f.queue.as_str(), "default");
    }

    #[test]
    fn backup_marks_active_and_inactive() {
        let school = make_school();
        let id = crate::value_objects::BackupId::new(school, Uuid::nil());
        let mut b = Backup::new(NewBackup {
            id,
            file_name: BackupFileName::new("backup.sql").unwrap(),
            source_link: BackupSourceLink::new("s3://bucket/backup.sql").unwrap(),
            file_type: BackupFileType::Database,
            lang_type: None,
            active_status: true,
            restore_in_progress: false,
            created_by: UserId::from_uuid(Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        let event_id = EventId::from_uuid(Uuid::from_u128(1));
        b.mark_active(UserId::from_uuid(Uuid::nil()), Timestamp::now(), event_id);
        assert!(b.active_status);
        b.mark_inactive(UserId::from_uuid(Uuid::nil()), Timestamp::now(), event_id);
        assert!(!b.active_status);
    }

    #[test]
    fn sidebar_reorder_updates_position() {
        let school = make_school();
        let id = crate::value_objects::SidebarId::new(school, Uuid::from_u128(1));
        let mut s = Sidebar::new(NewSidebar {
            id,
            permission_id: PermissionId::new(school, Uuid::from_u128(2)),
            role_id: RoleId::new(school, Uuid::from_u128(3)),
            position: SidebarPosition::new(0).unwrap(),
            section_id: SidebarSectionId::new(1),
            parent: None,
            parent_route: None,
            level: SidebarLevel::new(1).unwrap(),
            is_system_defined: SidebarIsSystemDefined::new(false),
            ignore: SidebarIgnoreFlag::new(0).unwrap(),
            active_status: SidebarActiveStatus::new(true),
            user_id: UserId::from_uuid(Uuid::from_u128(4)),
            created_by: UserId::from_uuid(Uuid::from_u128(4)),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(Uuid::nil()),
        })
        .unwrap();
        let event_id = EventId::from_uuid(Uuid::from_u128(1));
        s.reorder(
            SidebarPosition::new(5).unwrap(),
            UserId::from_uuid(Uuid::nil()),
            Timestamp::now(),
            event_id,
        );
        assert_eq!(s.position.get(), 5);
    }

    #[test]
    fn job_payload_empty_is_rejected() {
        // The JobPayload type-safe wrapper rejects empty strings at
        // construction, so we can never build a NewJob with one.
        assert!(crate::value_objects::JobPayload::new("").is_err());
    }
}
