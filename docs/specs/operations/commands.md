# Operations Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability. Job,
system-version, and OAuth commands use a system tenant context
(the engine treats them as platform-internal).

## Backup

### CreateBackup

```rust
pub struct CreateBackupCommand {
    pub tenant: TenantContext,
    pub file_name: BackupFileName,
    pub source_link: BackupSourceLink,
    pub file_type: BackupFileType,
    pub lang_type: Option<BackupLangType>,
}
```

**Capability:** `Operations.Backup.Create`
**Pre-conditions:** `file_name` is unique within the school.

**Effects:** Creates a `Backup` row, emits `BackupCreated`. The
actual file is produced by the file-storage port.

### DeleteBackup

```rust
pub struct DeleteBackupCommand {
    pub tenant: TenantContext,
    pub backup_id: BackupId,
}
```

**Capability:** `Operations.Backup.Delete`
**Effects:** Deletes the `Backup` row and the underlying file,
emits `BackupDeleted`.

### RestoreBackup

```rust
pub struct RestoreBackupCommand {
    pub tenant: TenantContext,
    pub backup_id: BackupId,
}
```

**Capability:** `Operations.Backup.Restore`
**Pre-conditions:** No other restore is in progress for the
school.

**Effects:** Triggers the restore through the storage port,
emits `BackupRestored`. After restore, the platform domain
invalidates its in-memory caches.

### MarkBackupActive / MarkBackupInactive

```rust
pub struct MarkBackupActiveCommand {
    pub tenant: TenantContext,
    pub backup_id: BackupId,
}
```

**Capabilities:** `Operations.Backup.Activate`,
`Operations.Backup.Deactivate`.

## Job

### ScheduleJob

```rust
pub struct ScheduleJobCommand {
    pub tenant: TenantContext, // system tenant
    pub queue: JobQueue,
    pub payload: JobPayload,
    pub available_at: JobAvailableAt,
}
```

**Capability:** `Operations.Job.Schedule` (system)
**Effects:** Creates a `Job` row and emits `JobScheduled`. A
job runner (port) consumes the row.

### CancelJob

```rust
pub struct CancelJobCommand {
    pub tenant: TenantContext, // system tenant
    pub job_id: JobId,
}
```

**Capability:** `Operations.Job.Cancel` (system)
**Effects:** Deletes the `Job` row (if not yet reserved) and
emits `JobCancelled`. A reserved job cannot be cancelled.

### MarkJobReserved / MarkJobCompleted / MarkJobFailed

System-internal commands issued by the job runner. Not
user-callable.

```rust
pub struct MarkJobReservedCommand {
    pub job_id: JobId,
    pub worker_id: String,
}

pub struct MarkJobCompletedCommand {
    pub job_id: JobId,
}

pub struct MarkJobFailedCommand {
    pub job_id: JobId,
    pub exception: FailedJobException,
}
```

**Capabilities:** `Operations.Job.Reserve` (system),
`Operations.Job.Complete` (system), `Operations.Job.Fail` (system).

## Failed Job

### RecordFailedJob

```rust
pub struct RecordFailedJobCommand {
    pub job_id: JobId,
    pub exception: FailedJobException,
}
```

**Capability:** `Operations.Job.Fail` (system)
**Effects:** Creates a `FailedJob` row, deletes the `Job` row,
emits `FailedJobRecorded`.

### RetryFailedJob

```rust
pub struct RetryFailedJobCommand {
    pub tenant: TenantContext, // system tenant
    pub failed_job_id: FailedJobId,
}
```

**Capability:** `Operations.Job.Retry` (system)
**Effects:** Creates a new `Job` row with the original payload
and emits `FailedJobRetried`. The original `FailedJob` row is
preserved for audit.

### DeleteFailedJob

```rust
pub struct DeleteFailedJobCommand {
    pub tenant: TenantContext,
    pub failed_job_id: FailedJobId,
}
```

**Capability:** `Operations.Job.Purge`
**Effects:** Deletes the `FailedJob` row and emits
`FailedJobDeleted`.

## System Version

### RegisterSystemVersion

```rust
pub struct RegisterSystemVersionCommand {
    pub tenant: TenantContext, // system tenant
    pub version_name: VersionName,
    pub title: VersionTitle,
    pub features: VersionFeatures,
}
```

**Capability:** `Operations.Version.Register` (system, build-time)
**Effects:** Creates a `SystemVersion` row and emits
`SystemVersionRegistered`. Subscribers (settings) refresh their
`system_version` field.

### UpdateSystemVersion

```rust
pub struct UpdateSystemVersionCommand {
    pub tenant: TenantContext, // system tenant
    pub version_id: SystemVersionId,
    pub title: Option<VersionTitle>,
    pub features: Option<VersionFeatures>,
}
```

**Capability:** `Operations.Version.Update` (system)
**Effects:** Emits `SystemVersionUpdated`.

## Version History

### RecordVersionHistory

```rust
pub struct RecordVersionHistoryCommand {
    pub tenant: TenantContext, // system tenant
    pub version: HistoryVersion,
    pub release_date: HistoryReleaseDate,
    pub url: Option<HistoryUrl>,
    pub notes: HistoryNotes,
}
```

**Capability:** `Operations.VersionHistory.Record` (system,
build-time)
**Effects:** Creates a `VersionHistory` row and emits
`VersionHistoryRecorded`.

## User Log

### RecordUserLog

```rust
pub struct RecordUserLogCommand {
    pub tenant: TenantContext, // system tenant
    pub user_id: UserId,
    pub role_id: RoleId,
    pub ip_address: IpAddress,
    pub user_agent: UserAgent,
    pub outcome: LoginOutcome,
    pub failure_reason: Option<LoginFailureReason>,
}
```

**Capability:** `Operations.Audit.Record` (system)
**Effects:** Creates a `UserLog` row and emits `UserLogged`.

## Maintenance

### ConfigureMaintenance

```rust
pub struct ConfigureMaintenanceCommand {
    pub tenant: TenantContext,
    pub title: MaintenanceTitle,
    pub sub_title: MaintenanceSubTitle,
    pub image: Option<MaintenanceImage>,
    pub applicable_for: MaintenanceApplicableFor,
}
```

**Capability:** `Operations.Maintenance.Configure`
**Effects:** Creates or updates the school's
`MaintenanceSetting` row and emits `MaintenanceConfigured`.

### EnableMaintenance

```rust
pub struct EnableMaintenanceCommand {
    pub tenant: TenantContext,
}
```

**Capability:** `Operations.Maintenance.Enable`
**Effects:** Sets `maintenance_mode=true` and emits
`MaintenanceEnabled`. The platform domain's authentication flow
rejects non-admin logins.

### DisableMaintenance

```rust
pub struct DisableMaintenanceCommand {
    pub tenant: TenantContext,
}
```

**Capability:** `Operations.Maintenance.Disable`
**Effects:** Sets `maintenance_mode=false` and emits
`MaintenanceDisabled`.

## Sidebar

### CreateSidebarEntry

```rust
pub struct CreateSidebarEntryCommand {
    pub tenant: TenantContext,
    pub permission_id: PermissionId,
    pub position: SidebarPosition,
    pub section_id: SidebarSectionId,
    pub parent: Option<SidebarId>,
    pub parent_route: Option<SidebarParentRoute>,
    pub level: SidebarLevel,
    pub role_id: RoleId,
    pub is_system_defined: SidebarIsSystemDefined,
    pub ignore: SidebarIgnore,
}
```

**Capability:** `Operations.Sidebar.Create`
**Effects:** Emits `SidebarEntryCreated`.

### UpdateSidebarEntry / DeleteSidebarEntry / ReorderSidebar

Standard CRUD plus reorder on `Sidebar`.

**Capabilities:** `Operations.Sidebar.Update`,
`Operations.Sidebar.Delete`, `Operations.Sidebar.Reorder`.

## OAuth (Port-Driven, Documented)

The engine documents the OAuth commands as port contracts. They
are implemented by the consumer's auth provider adapter, not by
the engine.

```rust
pub struct IssueOAuthAccessTokenCommand { ... }
pub struct RevokeOAuthAccessTokenCommand { ... }
pub struct IssueOAuthAuthorizationCodeCommand { ... }
pub struct ExchangeOAuthAuthorizationCodeCommand { ... }
pub struct RegisterOAuthClientCommand { ... }
pub struct RevokeOAuthRefreshTokenCommand { ... }
```

These are not in the engine's command catalog; the
`AuthProvider` port trait declares the relevant methods.

## Password Reset (Port-Driven, Documented)

```rust
pub struct RequestPasswordResetCommand {
    pub email: PasswordResetEmail,
}
pub struct CompletePasswordResetCommand {
    pub email: PasswordResetEmail,
    pub token: PasswordResetToken,
    pub new_password_hash: PasswordHash,
}
```

Implemented by the `AuthProvider` port trait, not by the engine.
