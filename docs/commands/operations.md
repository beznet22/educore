# Operations Domain — Commands

Quick reference of every command the operations domain exposes. These
commands cover backups, jobs and failed jobs, system version and
version history, user log (audit), maintenance mode, and sidebar
configuration. Some commands require a system tenant and are not
user-callable.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                          | Capability                          | Description                                                                                  | Events                                          | Idempotent? | Offline? |
| -------------------------------- | ----------------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------- | ----------- | -------- |
| `CreateBackup`                   | `Operations.Backup.Create`          | Create a backup record and file.                                                             | `BackupCreated`                                 | no          | no       |
| `DeleteBackup`                   | `Operations.Backup.Delete`          | Delete a backup record and file.                                                             | `BackupDeleted`                                 | no          | no       |
| `RestoreBackup`                  | `Operations.Backup.Restore`         | Restore a backup through the storage port.                                                   | `BackupRestored`                                | no          | no       |
| `MarkBackupActive`               | `Operations.Backup.Activate`        | Mark a backup as the school's active backup.                                                 | `BackupMarkedActive`                            | no          | yes      |
| `MarkBackupInactive`             | `Operations.Backup.Deactivate`      | Clear a backup's active flag.                                                                | `BackupMarkedInactive`                          | no          | yes      |
| `ScheduleJob`                    | `Operations.Job.Schedule`           | Schedule a job for a queue.                                                                  | `JobScheduled`                                  | no          | yes      |
| `CancelJob`                      | `Operations.Job.Cancel`             | Cancel a job that has not yet been reserved.                                                 | `JobCancelled`                                  | no          | yes      |
| `MarkJobReserved`                | `Operations.Job.Reserve` (system)   | Mark a job as reserved by a worker.                                                          | `JobReserved`                                   | no          | no       |
| `MarkJobCompleted`               | `Operations.Job.Complete` (system)  | Mark a reserved job as completed.                                                            | `JobCompleted`                                  | no          | no       |
| `MarkJobFailed`                  | `Operations.Job.Fail` (system)      | Mark a reserved job as failed; records a `FailedJob`.                                        | `JobFailed`, `FailedJobRecorded`                | no          | no       |
| `RecordFailedJob`                | `Operations.Job.Fail` (system)      | Manually record a failed job.                                                                | `FailedJobRecorded`                             | no          | no       |
| `RetryFailedJob`                 | `Operations.Job.Retry` (system)     | Re-queue a failed job.                                                                       | `FailedJobRetried`                              | no          | yes      |
| `DeleteFailedJob`                | `Operations.Job.Purge`              | Purge a failed job record.                                                                   | `FailedJobDeleted`                              | no          | yes      |
| `RegisterSystemVersion`          | `Operations.Version.Register`       | Register a system version.                                                                   | `SystemVersionRegistered`                       | no          | yes      |
| `UpdateSystemVersion`            | `Operations.Version.Update`         | Patch a system version.                                                                      | `SystemVersionUpdated`                          | no          | yes      |
| `RecordVersionHistory`           | `Operations.VersionHistory.Record`  | Record a version history entry.                                                              | `VersionHistoryRecorded`                        | no          | yes      |
| `RecordUserLog`                  | `Operations.Audit.Record` (system)  | Record a user authentication outcome.                                                        | `UserLogged`                                    | no          | no       |
| `ConfigureMaintenance`           | `Operations.Maintenance.Configure`  | Configure the maintenance page.                                                              | `MaintenanceConfigured`                         | no          | yes      |
| `EnableMaintenance`              | `Operations.Maintenance.Enable`     | Turn maintenance mode on.                                                                    | `MaintenanceEnabled`                            | no          | yes      |
| `DisableMaintenance`             | `Operations.Maintenance.Disable`    | Turn maintenance mode off.                                                                   | `MaintenanceDisabled`                           | no          | yes      |
| `CreateSidebarEntry`             | `Operations.Sidebar.Create`         | Create a sidebar entry.                                                                      | `SidebarEntryCreated`                           | no          | yes      |
| `UpdateSidebarEntry`             | `Operations.Sidebar.Update`         | Patch a sidebar entry.                                                                       | `SidebarEntryUpdated`                           | no          | yes      |
| `DeleteSidebarEntry`             | `Operations.Sidebar.Delete`         | Soft-delete a sidebar entry.                                                                  | `SidebarEntryDeleted`                           | no          | yes      |
| `ReorderSidebar`                 | `Operations.Sidebar.Reorder`        | Reorder sidebar entries.                                                                     | `SidebarReordered`                              | no          | yes      |

## Port-Driven Commands (Documented, Not in Engine Catalog)

The following commands are port contracts implemented by the
consumer's auth provider adapter, not by the engine itself. They are
documented for completeness but are not part of the operations
domain's command catalog:

- `IssueOAuthAccessToken`, `RevokeOAuthAccessToken`,
  `IssueOAuthAuthorizationCode`, `ExchangeOAuthAuthorizationCode`,
  `RegisterOAuthClient`, `RevokeOAuthRefreshToken`.
- `RequestPasswordReset`, `CompletePasswordReset`.

**See also:** `docs/specs/operations/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
