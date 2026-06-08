# Operations Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## BackupFile

**Identity:** `BackupFileId(SchoolId, Uuid)`
**Owner:** `Backup`

A typed projection of the backup file. The base `Backup` row
carries `file_name` and `source_link`; `BackupFile` is the typed
view (name, mime, size, checksum, encryption).

## BackupSchedule

**Identity:** `BackupScheduleId(SchoolId, Uuid)`
**Owner:** `Backup` (logical)

A cron-style schedule for automatic backups. The engine does not
own a job runner; the schedule is a port-driven configuration
that an external adapter (e.g. a cron job in the consumer
deployment) reads and acts on.

## BackupRetention

**Identity:** `BackupRetentionId(SchoolId, Uuid)`
**Owner:** `School`

A retention policy: keep N most recent backups, drop older.
Applied by a port-driven consumer-side job.

## JobPayload

**Identity:** Embedded value
**Owner:** `Job`

A typed view of the job's serialized command envelope. The base
`Job` row stores the payload as `longtext`; `JobPayload` is the
typed projection (command type, target aggregate, args).

## JobAttempt

**Identity:** `JobAttemptId(Uuid)` (global)
**Owner:** `Job`

A single attempt at running a job. The base `Job` row carries
`attempts` and `reserved_at`; `JobAttempt` is a detailed
projection that records per-attempt timing, worker id, and
result.

## FailedJobException

**Identity:** `FailedJobExceptionId(Uuid)` (global)
**Owner:** `FailedJob`

A typed projection of the captured exception. The base
`FailedJob` row stores `exception` as `longtext`;
`FailedJobException` is the typed view (kind, message, stack
trace, root cause).

## SystemVersionFeature

**Identity:** `SystemVersionFeatureId(Uuid)` (global)
**Owner:** `SystemVersion`

A typed projection of the features blurb. The base row stores
`features` as `varchar(255)`; `SystemVersionFeature` is the
typed view (a list of feature entries with id, title, blurb).

## VersionHistoryNote

**Identity:** `VersionHistoryNoteId(Uuid)` (global)
**Owner:** `VersionHistory`

A typed projection of the `notes` string. The base row stores
`notes` as `varchar(191)`; `VersionHistoryNote` is the typed
view (a list of note lines).

## UserLogContext

**Identity:** `UserLogContextId(SchoolId, Uuid)`
**Owner:** `UserLog`

A typed projection of the request context: parsed IP (v4 or v6),
parsed user agent (browser, OS, device), request id, correlation
id.

## UserLogSession

**Identity:** `UserLogSessionId(SchoolId, Uuid)`
**Owner:** `UserLog`

A typed projection of the session that the login created. The
session id is hashed; the projection is a denormalized view
used by the security audit screen.

## MaintenanceOverride

**Identity:** `MaintenanceOverrideId(SchoolId, Uuid)`
**Owner:** `MaintenanceSetting`

A per-role override. The base `MaintenanceSetting` carries
`applicable_for` as a string; `MaintenanceOverride` is the typed
view (a set of `(role_id, allowed)` pairs).

## MaintenanceMessage

**Identity:** Embedded value
**Owner:** `MaintenanceSetting`

A typed projection of the title and sub-title. The base row
stores them as `varchar(191)`; `MaintenanceMessage` is the
typed view (locale-aware message with title and body).

## SidebarEntry

**Identity:** `SidebarEntryId(SchoolId, Uuid)`
**Owner:** `Sidebar`

A typed projection of a single sidebar entry. The base
`Sidebar` row carries a flat int `level`; `SidebarEntry` is the
typed view (level enum, parent, ordering, route, icon, label).

## SidebarRoute

**Identity:** `SidebarRouteId(SchoolId, Uuid)`
**Owner:** `Sidebar`

A typed projection of the `parent_route` integer. The base row
stores it as a flat int referencing another `Sidebar` row;
`SidebarRoute` is the typed reference.

## SidebarIgnoreFlag

**Identity:** Embedded value
**Owner:** `Sidebar`

A typed projection of the `ignore` int. The base row stores it
as a flat int; `SidebarIgnoreFlag` is the typed view
(`0=Show`, `1=Hide`, `2=Disabled`).

## SidebarLevel

**Identity:** Embedded value
**Owner:** `Sidebar`

A typed projection of the `level` int. The base row stores it
as a flat int; `SidebarLevel` is the typed view
(`Parent=1`, `Child=2`, `SubChild=3`).

## JobQueue

**Identity:** `JobQueueId(SchoolId, Uuid)`
**Owner:** `Job` (logical)

A logical grouping of jobs by queue name. Used by the
operations UI to render per-queue statistics.

## BackupStorageRef

**Identity:** Embedded value
**Owner:** `Backup`

A typed projection of the `source_link` field. The base row
stores it as a `varchar(255)`; `BackupStorageRef` is the typed
view (storage provider, bucket, key).

## SystemVersionManifest

**Identity:** `SystemVersionManifestId(Uuid)` (global)
**Owner:** `SystemVersion`

A typed projection of the version's deployment manifest
(targets, capabilities introduced, migration scripts required).

## AuditPartition

**Identity:** `AuditPartitionId(SchoolId, Uuid)`
**Owner:** `UserLog`

A logical time partition for the user log (per month, per
quarter). Used by the operations UI to render "Q1 2026 audit
log" views.

## SidebarRoleBinding

**Identity:** `SidebarRoleBindingId(SchoolId, Uuid)`
**Owner:** `Sidebar`

A typed binding between a sidebar entry and a role. The base
`Sidebar::role_id` is the binding; `SidebarRoleBinding` is the
typed view carrying role name and the entry's effective
capability requirement.

## SystemVersionCapability

**Identity:** `SystemVersionCapabilityId(Uuid)` (global)
**Owner:** `SystemVersion`

A capability introduced in this version. The operations domain
records the capability to enable capability-versioning reports
("when did `Finance.Invoice.Adjust` first appear?").

## VersionMigration

**Identity:** `VersionMigrationId(Uuid)` (global)
**Owner:** `VersionHistory`

A migration script required for this version. The operations
domain records the script name and status (Applied, Pending,
Failed) but does not execute the migration.
