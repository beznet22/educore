# Operations Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration,
audit, and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Backup Lifecycle

### BackupCreated

```rust
pub struct BackupCreated {
    pub backup_id: BackupId,
    pub file_name: BackupFileName,
    pub file_type: BackupFileType,
    pub created_at: Timestamp,
}
```

### BackupDeleted

```rust
pub struct BackupDeleted {
    pub backup_id: BackupId,
    pub prior_file_name: BackupFileName,
}
```

### BackupRestored

```rust
pub struct BackupRestored {
    pub backup_id: BackupId,
    pub restored_at: Timestamp,
    pub restored_by: UserId,
}
```

**Subscribers:**
- `platform` — invalidates in-memory caches.

### BackupMarkedActive / BackupMarkedInactive

```rust
pub struct BackupMarkedActive {
    pub backup_id: BackupId,
}

pub struct BackupMarkedInactive {
    pub backup_id: BackupId,
}
```

## Job Lifecycle

### JobScheduled

```rust
pub struct JobScheduled {
    pub job_id: JobId,
    pub queue: JobQueue,
    pub available_at: JobAvailableAt,
}
```

### JobCancelled

```rust
pub struct JobCancelled {
    pub job_id: JobId,
    pub queue: JobQueue,
}
```

### JobReserved

```rust
pub struct JobReserved {
    pub job_id: JobId,
    pub worker_id: String,
    pub reserved_at: Timestamp,
}
```

### JobCompleted

```rust
pub struct JobCompleted {
    pub job_id: JobId,
    pub queue: JobQueue,
    pub attempts: JobAttempts,
    pub completed_at: Timestamp,
}
```

### JobFailed

```rust
pub struct JobFailed {
    pub job_id: JobId,
    pub queue: JobQueue,
    pub attempts: JobAttempts,
    pub failed_at: Timestamp,
}
```

**Subscribers:**
- `FailedJob` is created from this event by the operations
  subscriber.

## Failed Job Lifecycle

### FailedJobRecorded

```rust
pub struct FailedJobRecorded {
    pub failed_job_id: FailedJobId,
    pub original_job_id: JobId,
    pub queue: FailedJobQueue,
    pub exception: String,
    pub failed_at: Timestamp,
}
```

### FailedJobRetried

```rust
pub struct FailedJobRetried {
    pub failed_job_id: FailedJobId,
    pub new_job_id: JobId,
    pub retried_at: Timestamp,
}
```

### FailedJobDeleted

```rust
pub struct FailedJobDeleted {
    pub failed_job_id: FailedJobId,
}
```

## System Version Lifecycle

### SystemVersionRegistered

```rust
pub struct SystemVersionRegistered {
    pub version_id: SystemVersionId,
    pub version_name: VersionName,
    pub title: VersionTitle,
}
```

**Subscribers:**
- `settings` — refreshes its `system_version` field on
  `GeneralSettings`.

### SystemVersionUpdated

```rust
pub struct SystemVersionUpdated {
    pub version_id: SystemVersionId,
    pub changed_fields: Vec<&'static str>,
}
```

## Version History Lifecycle

### VersionHistoryRecorded

```rust
pub struct VersionHistoryRecorded {
    pub history_id: VersionHistoryId,
    pub version: HistoryVersion,
    pub release_date: HistoryReleaseDate,
}
```

### SystemVersionBumped

```rust
pub struct SystemVersionBumped {
    pub from_version: Option<VersionName>,
    pub to_version: VersionName,
    pub bumped_at: Timestamp,
}
```

This is a derived event emitted by the operations domain when
both a `SystemVersionRegistered` and a `VersionHistoryRecorded`
have been observed for the same version. It is the canonical
"the system just upgraded" event.

**Subscribers:**
- `platform` — refreshes the system-version display.
- `settings` — refreshes the `system_version` field on
  `GeneralSettings`.

## User Log Lifecycle

### UserLogged

```rust
pub struct UserLogged {
    pub log_id: UserLogId,
    pub user_id: UserId,
    pub role_id: RoleId,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub logged_at: Timestamp,
}
```

**Subscribers:**
- `platform` — updates the user's `last_login_at` projection.

## Maintenance Lifecycle

### MaintenanceConfigured

```rust
pub struct MaintenanceConfigured {
    pub school_id: SchoolId,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
    pub applicable_for: MaintenanceApplicableFor,
}
```

### MaintenanceEnabled

```rust
pub struct MaintenanceEnabled {
    pub school_id: SchoolId,
    pub enabled_at: Timestamp,
}
```

**Subscribers:**
- `platform` — begins rejecting non-admin logins for the school.

### MaintenanceDisabled

```rust
pub struct MaintenanceDisabled {
    pub school_id: SchoolId,
    pub disabled_at: Timestamp,
}
```

**Subscribers:**
- `platform` — resumes normal authentication flow.

## Sidebar Lifecycle

### SidebarEntryCreated

```rust
pub struct SidebarEntryCreated {
    pub sidebar_id: SidebarId,
    pub role_id: RoleId,
    pub permission_id: PermissionId,
    pub level: SidebarLevel,
}
```

### SidebarEntryUpdated / SidebarEntryDeleted

```rust
pub struct SidebarEntryUpdated {
    pub sidebar_id: SidebarId,
    pub changed_fields: Vec<&'static str>,
}

pub struct SidebarEntryDeleted {
    pub sidebar_id: SidebarId,
    pub prior_role_id: RoleId,
}
```

### SidebarReordered

```rust
pub struct SidebarReordered {
    pub role_id: RoleId,
    pub reordered_entries: u32,
}
```
