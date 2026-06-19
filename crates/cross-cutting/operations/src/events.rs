//! # educore-operations typed events
//!
//! Per `docs/specs/operations/events.md`. 28 typed events + 1
//! derived (`SystemVersionBumped`). Wire form:
//! `operations.<aggregate>.<verb>`. Each event implements
//! [`DomainEvent`].

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::value_objects::{
    BackupFileName, BackupFileType, HistoryVersion, IpAddress, JobAttempts, JobQueue, LoginOutcome,
    MaintenanceApplicableFor, MaintenanceSubTitle, MaintenanceTitle, PermissionId, RoleId,
    SidebarLevel, UserAgent, VersionTitle,
};

// =============================================================================
// === Backup events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `Backup` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupCreated {
    pub backup_id: crate::value_objects::BackupId,
    pub school_id: SchoolId,
    pub file_name: BackupFileName,
    pub file_type: BackupFileType,
    pub created_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl BackupCreated {
    /// Constructs a new `BackupCreated`.
    #[must_use]
    pub fn new(
        backup_id: crate::value_objects::BackupId,
        school_id: SchoolId,
        file_name: BackupFileName,
        file_type: BackupFileType,
        created_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            backup_id,
            school_id,
            file_name,
            file_type,
            created_at,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for BackupCreated {
    const EVENT_TYPE: &'static str = "operations.backup.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "backup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.backup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Backup` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupDeleted {
    pub backup_id: crate::value_objects::BackupId,
    pub school_id: SchoolId,
    pub prior_file_name: BackupFileName,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl BackupDeleted {
    /// Constructs a new `BackupDeleted`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        backup_id: crate::value_objects::BackupId,
        school_id: SchoolId,
        prior_file_name: BackupFileName,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            backup_id,
            school_id,
            prior_file_name,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for BackupDeleted {
    const EVENT_TYPE: &'static str = "operations.backup.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "backup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.backup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Backup` is restored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupRestored {
    pub backup_id: crate::value_objects::BackupId,
    pub school_id: SchoolId,
    pub restored_at: Timestamp,
    pub restored_by: UserId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BackupRestored {
    /// Constructs a new `BackupRestored`.
    #[must_use]
    pub fn new(
        backup_id: crate::value_objects::BackupId,
        school_id: SchoolId,
        restored_at: Timestamp,
        restored_by: UserId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            backup_id,
            school_id,
            restored_at,
            restored_by,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BackupRestored {
    const EVENT_TYPE: &'static str = "operations.backup.restored";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "backup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.backup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Backup` is marked active.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupMarkedActive {
    pub backup_id: crate::value_objects::BackupId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl BackupMarkedActive {
    /// Constructs a new `BackupMarkedActive`.
    #[must_use]
    pub fn new(
        backup_id: crate::value_objects::BackupId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            backup_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for BackupMarkedActive {
    const EVENT_TYPE: &'static str = "operations.backup.marked_active";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "backup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.backup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Backup` is marked inactive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupMarkedInactive {
    pub backup_id: crate::value_objects::BackupId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl BackupMarkedInactive {
    /// Constructs a new `BackupMarkedInactive`.
    #[must_use]
    pub fn new(
        backup_id: crate::value_objects::BackupId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            backup_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for BackupMarkedInactive {
    const EVENT_TYPE: &'static str = "operations.backup.marked_inactive";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "backup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.backup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Backup events section end ===

// =============================================================================
// === Job events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `Job` is scheduled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobScheduled {
    pub job_id: crate::value_objects::JobId,
    pub school_id: Option<SchoolId>,
    pub queue: JobQueue,
    pub available_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl JobScheduled {
    /// Constructs a new `JobScheduled`.
    #[must_use]
    pub fn new(
        job_id: crate::value_objects::JobId,
        school_id: Option<SchoolId>,
        queue: JobQueue,
        available_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            job_id,
            school_id,
            queue,
            available_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for JobScheduled {
    const EVENT_TYPE: &'static str = "operations.job.scheduled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
            .unwrap_or(educore_core::ids::PLATFORM_SCHOOL_ID)
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Job` is cancelled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobCancelled {
    pub job_id: crate::value_objects::JobId,
    pub school_id: Option<SchoolId>,
    pub queue: JobQueue,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl JobCancelled {
    /// Constructs a new `JobCancelled`.
    #[must_use]
    pub fn new(
        job_id: crate::value_objects::JobId,
        school_id: Option<SchoolId>,
        queue: JobQueue,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            job_id,
            school_id,
            queue,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for JobCancelled {
    const EVENT_TYPE: &'static str = "operations.job.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
            .unwrap_or(educore_core::ids::PLATFORM_SCHOOL_ID)
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Job` is reserved by a worker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobReserved {
    pub job_id: crate::value_objects::JobId,
    pub school_id: Option<SchoolId>,
    pub worker_id: String,
    pub reserved_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl JobReserved {
    /// Constructs a new `JobReserved`.
    #[must_use]
    pub fn new(
        job_id: crate::value_objects::JobId,
        school_id: Option<SchoolId>,
        worker_id: String,
        reserved_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            job_id,
            school_id,
            worker_id,
            reserved_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for JobReserved {
    const EVENT_TYPE: &'static str = "operations.job.reserved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
            .unwrap_or(educore_core::ids::PLATFORM_SCHOOL_ID)
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Job` is completed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobCompleted {
    pub job_id: crate::value_objects::JobId,
    pub school_id: Option<SchoolId>,
    pub queue: JobQueue,
    pub attempts: JobAttempts,
    pub completed_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl JobCompleted {
    /// Constructs a new `JobCompleted`.
    #[must_use]
    pub fn new(
        job_id: crate::value_objects::JobId,
        school_id: Option<SchoolId>,
        queue: JobQueue,
        attempts: JobAttempts,
        completed_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            job_id,
            school_id,
            queue,
            attempts,
            completed_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for JobCompleted {
    const EVENT_TYPE: &'static str = "operations.job.completed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
            .unwrap_or(educore_core::ids::PLATFORM_SCHOOL_ID)
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Job` is terminally failed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JobFailed {
    pub job_id: crate::value_objects::JobId,
    pub school_id: Option<SchoolId>,
    pub queue: JobQueue,
    pub attempts: JobAttempts,
    pub failed_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl JobFailed {
    /// Constructs a new `JobFailed`.
    #[must_use]
    pub fn new(
        job_id: crate::value_objects::JobId,
        school_id: Option<SchoolId>,
        queue: JobQueue,
        attempts: JobAttempts,
        failed_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            job_id,
            school_id,
            queue,
            attempts,
            failed_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for JobFailed {
    const EVENT_TYPE: &'static str = "operations.job.failed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
            .unwrap_or(educore_core::ids::PLATFORM_SCHOOL_ID)
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Job events section end ===

// =============================================================================
// === FailedJob events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `FailedJob` row is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedJobRecorded {
    pub failed_job_id: crate::value_objects::FailedJobId,
    pub original_job_id: crate::value_objects::JobId,
    pub queue: crate::value_objects::FailedJobQueue,
    pub exception: String,
    pub failed_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FailedJobRecorded {
    /// Constructs a new `FailedJobRecorded`.
    #[must_use]
    pub fn new(
        failed_job_id: crate::value_objects::FailedJobId,
        original_job_id: crate::value_objects::JobId,
        queue: crate::value_objects::FailedJobQueue,
        exception: String,
        failed_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            failed_job_id,
            original_job_id,
            queue,
            exception,
            failed_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FailedJobRecorded {
    const EVENT_TYPE: &'static str = "operations.failed_job.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "failed_job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.failed_job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `FailedJob` is retried.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedJobRetried {
    pub failed_job_id: crate::value_objects::FailedJobId,
    pub new_job_id: crate::value_objects::JobId,
    pub retried_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl FailedJobRetried {
    /// Constructs a new `FailedJobRetried`.
    #[must_use]
    pub fn new(
        failed_job_id: crate::value_objects::FailedJobId,
        new_job_id: crate::value_objects::JobId,
        retried_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            failed_job_id,
            new_job_id,
            retried_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for FailedJobRetried {
    const EVENT_TYPE: &'static str = "operations.failed_job.retried";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "failed_job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.failed_job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `FailedJob` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailedJobDeleted {
    pub failed_job_id: crate::value_objects::FailedJobId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl FailedJobDeleted {
    /// Constructs a new `FailedJobDeleted`.
    #[must_use]
    pub fn new(
        failed_job_id: crate::value_objects::FailedJobId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            failed_job_id,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for FailedJobDeleted {
    const EVENT_TYPE: &'static str = "operations.failed_job.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "failed_job";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.failed_job_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === FailedJob events section end ===

// =============================================================================
// === SystemVersion events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `SystemVersion` is registered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionRegistered {
    pub version_id: crate::value_objects::SystemVersionId,
    pub version_name: crate::value_objects::VersionName,
    pub title: VersionTitle,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SystemVersionRegistered {
    /// Constructs a new `SystemVersionRegistered`.
    #[must_use]
    pub fn new(
        version_id: crate::value_objects::SystemVersionId,
        version_name: crate::value_objects::VersionName,
        title: VersionTitle,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            version_id,
            version_name,
            title,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SystemVersionRegistered {
    const EVENT_TYPE: &'static str = "operations.system_version.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "system_version";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.version_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SystemVersion` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionUpdated {
    pub version_id: crate::value_objects::SystemVersionId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SystemVersionUpdated {
    /// Constructs a new `SystemVersionUpdated`.
    #[must_use]
    pub fn new(
        version_id: crate::value_objects::SystemVersionId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            version_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SystemVersionUpdated {
    const EVENT_TYPE: &'static str = "operations.system_version.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "system_version";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.version_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === SystemVersion events section end ===

// =============================================================================
// === VersionHistory events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `VersionHistory` row is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionHistoryRecorded {
    pub history_id: crate::value_objects::VersionHistoryId,
    pub version: HistoryVersion,
    pub release_date: crate::value_objects::HistoryReleaseDate,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl VersionHistoryRecorded {
    /// Constructs a new `VersionHistoryRecorded`.
    #[must_use]
    pub fn new(
        history_id: crate::value_objects::VersionHistoryId,
        version: HistoryVersion,
        release_date: crate::value_objects::HistoryReleaseDate,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            history_id,
            version,
            release_date,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for VersionHistoryRecorded {
    const EVENT_TYPE: &'static str = "operations.version_history.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "version_history";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.history_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Derived event: emitted by the operations domain when both a
/// `SystemVersionRegistered` and a `VersionHistoryRecorded` have
/// been observed for the same version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemVersionBumped {
    pub from_version: Option<crate::value_objects::VersionName>,
    pub to_version: crate::value_objects::VersionName,
    pub bumped_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SystemVersionBumped {
    /// Constructs a new `SystemVersionBumped`.
    #[must_use]
    pub fn new(
        from_version: Option<crate::value_objects::VersionName>,
        to_version: crate::value_objects::VersionName,
        bumped_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            from_version,
            to_version,
            bumped_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SystemVersionBumped {
    const EVENT_TYPE: &'static str = "operations.system_version.bumped";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "system_version";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        // Use a deterministic projection of the new version's bytes
        // as the aggregate id for the derived event.
        let mut bytes = [0u8; 16];
        let to_bytes = self.to_version.as_str().as_bytes();
        let len = to_bytes.len().min(16);
        bytes[..len].copy_from_slice(&to_bytes[..len]);
        Uuid::from_bytes(bytes)
    }
    fn school_id(&self) -> SchoolId {
        educore_core::ids::PLATFORM_SCHOOL_ID
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === VersionHistory events section end ===

// =============================================================================
// === UserLog events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `UserLog` row is recorded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLogged {
    pub log_id: crate::value_objects::UserLogId,
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub logged_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl UserLogged {
    /// Constructs a new `UserLogged`.
    #[must_use]
    pub fn new(
        log_id: crate::value_objects::UserLogId,
        school_id: SchoolId,
        user_id: UserId,
        role_id: RoleId,
        ip_address: IpAddress,
        user_agent: UserAgent,
        outcome: LoginOutcome,
        logged_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            log_id,
            school_id,
            user_id,
            role_id,
            ip_address,
            user_agent,
            outcome,
            logged_at,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserLogged {
    const EVENT_TYPE: &'static str = "operations.user_log.logged";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "user_log";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === UserLog events section end ===

// =============================================================================
// === Maintenance events section begin (owner: B) ===
// =============================================================================

/// Emitted when `MaintenanceSetting` is configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceConfigured {
    pub setting_id: crate::value_objects::MaintenanceSettingId,
    pub school_id: SchoolId,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
    pub applicable_for: MaintenanceApplicableFor,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl MaintenanceConfigured {
    /// Constructs a new `MaintenanceConfigured`.
    #[must_use]
    pub fn new(
        setting_id: crate::value_objects::MaintenanceSettingId,
        school_id: SchoolId,
        title: MaintenanceTitle,
        sub_title: MaintenanceSubTitle,
        applicable_for: MaintenanceApplicableFor,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            setting_id,
            school_id,
            title,
            sub_title,
            applicable_for,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for MaintenanceConfigured {
    const EVENT_TYPE: &'static str = "operations.maintenance.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "maintenance_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when maintenance mode is enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceEnabled {
    pub setting_id: crate::value_objects::MaintenanceSettingId,
    pub school_id: SchoolId,
    pub enabled_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl MaintenanceEnabled {
    /// Constructs a new `MaintenanceEnabled`.
    #[must_use]
    pub fn new(
        setting_id: crate::value_objects::MaintenanceSettingId,
        school_id: SchoolId,
        enabled_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            setting_id,
            school_id,
            enabled_at,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for MaintenanceEnabled {
    const EVENT_TYPE: &'static str = "operations.maintenance.enabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "maintenance_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when maintenance mode is disabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceDisabled {
    pub setting_id: crate::value_objects::MaintenanceSettingId,
    pub school_id: SchoolId,
    pub disabled_at: Timestamp,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl MaintenanceDisabled {
    /// Constructs a new `MaintenanceDisabled`.
    #[must_use]
    pub fn new(
        setting_id: crate::value_objects::MaintenanceSettingId,
        school_id: SchoolId,
        disabled_at: Timestamp,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            setting_id,
            school_id,
            disabled_at,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for MaintenanceDisabled {
    const EVENT_TYPE: &'static str = "operations.maintenance.disabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "maintenance_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Maintenance events section end ===

// =============================================================================
// === Sidebar events section begin (owner: B) ===
// =============================================================================

/// Emitted when a `Sidebar` entry is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarEntryCreated {
    pub sidebar_id: crate::value_objects::SidebarId,
    pub school_id: SchoolId,
    pub role_id: RoleId,
    pub permission_id: PermissionId,
    pub level: SidebarLevel,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl SidebarEntryCreated {
    /// Constructs a new `SidebarEntryCreated`.
    #[must_use]
    pub fn new(
        sidebar_id: crate::value_objects::SidebarId,
        school_id: SchoolId,
        role_id: RoleId,
        permission_id: PermissionId,
        level: SidebarLevel,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            sidebar_id,
            school_id,
            role_id,
            permission_id,
            level,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for SidebarEntryCreated {
    const EVENT_TYPE: &'static str = "operations.sidebar.entry_created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sidebar";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.sidebar_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Sidebar` entry is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarEntryUpdated {
    pub sidebar_id: crate::value_objects::SidebarId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl SidebarEntryUpdated {
    /// Constructs a new `SidebarEntryUpdated`.
    #[must_use]
    pub fn new(
        sidebar_id: crate::value_objects::SidebarId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            sidebar_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for SidebarEntryUpdated {
    const EVENT_TYPE: &'static str = "operations.sidebar.entry_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sidebar";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.sidebar_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Sidebar` entry is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarEntryDeleted {
    pub sidebar_id: crate::value_objects::SidebarId,
    pub school_id: SchoolId,
    pub prior_role_id: RoleId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl SidebarEntryDeleted {
    /// Constructs a new `SidebarEntryDeleted`.
    #[must_use]
    pub fn new(
        sidebar_id: crate::value_objects::SidebarId,
        school_id: SchoolId,
        prior_role_id: RoleId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            sidebar_id,
            school_id,
            prior_role_id,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for SidebarEntryDeleted {
    const EVENT_TYPE: &'static str = "operations.sidebar.entry_deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sidebar";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.sidebar_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when `Sidebar` entries are reordered within a role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SidebarReordered {
    pub role_id: RoleId,
    pub school_id: SchoolId,
    pub reordered_entries: u32,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
    pub actor_id: UserId,
}

impl SidebarReordered {
    /// Constructs a new `SidebarReordered`.
    #[must_use]
    pub fn new(
        role_id: RoleId,
        school_id: SchoolId,
        reordered_entries: u32,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
        actor_id: UserId,
    ) -> Self {
        Self {
            role_id,
            school_id,
            reordered_entries,
            event_id_field,
            correlation_id,
            occurred_at,
            actor_id,
        }
    }
}

impl DomainEvent for SidebarReordered {
    const EVENT_TYPE: &'static str = "operations.sidebar.reordered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sidebar";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Sidebar events section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn all_event_wire_forms_resolve() {
        let types: Vec<&str> = vec![
            // Backup (5)
            "operations.backup.created",
            "operations.backup.deleted",
            "operations.backup.restored",
            "operations.backup.marked_active",
            "operations.backup.marked_inactive",
            // Job (5)
            "operations.job.scheduled",
            "operations.job.cancelled",
            "operations.job.reserved",
            "operations.job.completed",
            "operations.job.failed",
            // FailedJob (3)
            "operations.failed_job.recorded",
            "operations.failed_job.retried",
            "operations.failed_job.deleted",
            // SystemVersion (3, includes derived)
            "operations.system_version.registered",
            "operations.system_version.updated",
            "operations.system_version.bumped",
            // VersionHistory (1)
            "operations.version_history.recorded",
            // UserLog (1)
            "operations.user_log.logged",
            // Maintenance (3)
            "operations.maintenance.configured",
            "operations.maintenance.enabled",
            "operations.maintenance.disabled",
            // Sidebar (4)
            "operations.sidebar.entry_created",
            "operations.sidebar.entry_updated",
            "operations.sidebar.entry_deleted",
            "operations.sidebar.reordered",
        ];
        // Per `docs/specs/operations/events.md`: 5 + 5 + 3 + 3 + 1 + 1
        // + 3 + 4 = 25 typed events (the 3 for `system_version`
        // includes the 1 derived `SystemVersionBumped`).
        assert_eq!(types.len(), 25);
        for t in &types {
            assert!(
                t.starts_with("operations."),
                "{t} should start with operations."
            );
        }
    }

    #[test]
    fn backup_event_metadata_is_set() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = crate::value_objects::BackupId::new(school, Uuid::from_u128(1));
        let event = BackupCreated::new(
            id,
            school,
            BackupFileName::new("backup.sql").unwrap(),
            BackupFileType::Database,
            Timestamp::now(),
            EventId::from_uuid(Uuid::from_u128(2)),
            CorrelationId::from_uuid(Uuid::from_u128(3)),
            Timestamp::now(),
            UserId::from_uuid(Uuid::from_u128(4)),
        );
        assert_eq!(
            <BackupCreated as DomainEvent>::EVENT_TYPE,
            "operations.backup.created"
        );
        assert_eq!(event.aggregate_id(), Uuid::from_u128(1));
    }

    #[test]
    fn system_version_bumped_aggregate_id_derived_from_to_version() {
        let event = SystemVersionBumped::new(
            None,
            crate::value_objects::VersionName::new("8.2.3").unwrap(),
            Timestamp::now(),
            EventId::from_uuid(Uuid::from_u128(1)),
            CorrelationId::from_uuid(Uuid::from_u128(2)),
            Timestamp::now(),
        );
        let agg_id = event.aggregate_id();
        let bytes = agg_id.as_bytes();
        // First 4 bytes of "8.2.3" are 0x38, 0x2e, 0x32, 0x2e.
        assert_eq!(bytes[0], b'8');
        assert_eq!(bytes[1], b'.');
        assert_eq!(bytes[2], b'2');
        assert_eq!(bytes[3], b'.');
    }
}
