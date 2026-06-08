# Operations Domain — Aggregates

## Backup

**Root type:** `Backup`
**Identity:** `BackupId(SchoolId, Uuid)`

### Purpose

A backup record. The actual backup file is produced by the
file-storage port; the `Backup` row records the metadata
(`file_name`, `source_link`, `file_type`, language hint,
`active_status`, audit fields).

### Invariants

1. A `Backup` belongs to exactly one school.
2. A `Backup::file_name` is non-empty and unique within
   `(school_id, file_name)`.
3. A `Backup::file_type` is `Database` (0), `File` (1), or
   `Image` (2).
4. A `Backup::source_link` is a URL or file-storage reference.
5. A `Backup::active_status` is a boolean.
6. A `Backup` cannot be hard-deleted while a restore is in
   progress.

### Commands

- `CreateBackup`
- `DeleteBackup`
- `RestoreBackup`
- `MarkBackupActive`
- `MarkBackupInactive`

### Events

- `BackupCreated`
- `BackupDeleted`
- `BackupRestored`
- `BackupMarkedActive`
- `BackupMarkedInactive`

### Consistency Boundary

A `Backup` is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction. Concurrent
`RestoreBackup` commands on the same backup are serialized.

---

## Job

**Root type:** `Job`
**Identity:** `JobId(Uuid)` (global, not tenant-scoped)

### Purpose

A pending job in the queue. The job runner (a port) consumes
`Job` rows, executes the payload, and either deletes the job
on success or moves it to `FailedJob` on terminal failure.

### Invariants

1. A `Job::queue` is a non-empty string.
2. A `Job::payload` is a serialized command envelope.
3. A `Job::attempts` is a `u8`, 0..=255.
4. A `Job::available_at` is a Unix timestamp; the job is not
   runnable before this time.
5. A `Job::reserved_at` is a Unix timestamp; if set, the job is
   currently being processed by a worker.

### Commands

- `ScheduleJob`
- `CancelJob`
- `MarkJobReserved`
- `MarkJobCompleted`
- `MarkJobFailed`

### Events

- `JobScheduled`
- `JobCancelled`
- `JobReserved`
- `JobCompleted`
- `JobFailed`

---

## FailedJob

**Root type:** `FailedJob`
**Identity:** `FailedJobId(Uuid)` (global)

### Purpose

A terminal record of a job that has exhausted its retry budget.
The `FailedJob` row is the input to operator-driven retry
workflows.

### Invariants

1. A `FailedJob::uuid` is unique.
2. A `FailedJob::connection` is a non-empty string.
3. A `FailedJob::queue` is a non-empty string.
4. A `FailedJob::payload` is the original job payload.
5. A `FailedJob::exception` is the captured exception text.
6. A `FailedJob::failed_at` is a timestamp.
7. A `FailedJob` is not retried automatically; a new `Job` is
   created on operator-driven retry.

### Commands

- `RecordFailedJob`
- `RetryFailedJob`
- `DeleteFailedJob`

### Events

- `FailedJobRecorded`
- `FailedJobRetried`
- `FailedJobDeleted`

---

## SystemVersion

**Root type:** `SystemVersion`
**Identity:** `SystemVersionId(Uuid)` (global)

### Purpose

A released version metadata record. Each row carries the
`version_name` (semver), `title` (human label), and `features`
(blurb).

### Invariants

1. A `SystemVersion::version_name` is a valid semver string and
   is unique.
2. A `SystemVersion::title` is non-empty.
3. A `SystemVersion::features` is non-empty.

### Commands

- `RegisterSystemVersion` (build-time, system-internal)
- `UpdateSystemVersion`

### Events

- `SystemVersionRegistered`
- `SystemVersionUpdated`

---

## VersionHistory

**Root type:** `VersionHistory`
**Identity:** `VersionHistoryId(Uuid)` (global)

### Purpose

An append-only record of version bumps. Each row carries the
`version`, `release_date`, `url` (release notes URL), and `notes`
(short summary).

### Invariants

1. A `VersionHistory::version` is non-empty.
2. A `VersionHistory::release_date` is a string (consumer
   format; typically `YYYY-MM-DD`).
3. A `VersionHistory::url` is a URL or empty.
4. A `VersionHistory::notes` is a string of up to 191 chars.
5. `VersionHistory` rows are append-only.

### Commands

- `RecordVersionHistory` (build-time, system-internal)

### Events

- `VersionHistoryRecorded`

---

## UserLog

**Root type:** `UserLog`
**Identity:** `UserLogId(SchoolId, Uuid)`

### Purpose

A per-login audit record. Records the IP address, user agent,
user id, role id, school, and academic year.

### Invariants

1. A `UserLog` is append-only. It is never updated or deleted
   (operator-driven hard delete is allowed only for GDPR
   compliance, with audit trail).
2. A `UserLog::ip_address` is a valid IP address or empty.
3. A `UserLog::user_agent` is a string of up to 191 chars.
4. A `UserLog::user_id` references a valid `UserId`.
5. A `UserLog::role_id` references a valid `RoleId`.
6. A `UserLog::school_id` is the tenant anchor.
7. A `UserLog::academic_id` references a valid
   `AcademicYearId`.

### Commands

- `RecordUserLog`

### Events

- `UserLogged`

---

## MaintenanceSetting

**Root type:** `MaintenanceSetting`
**Identity:** `MaintenanceSettingId(SchoolId, Uuid)`

### Purpose

The school's maintenance mode config. There is at most one
`MaintenanceSetting` row per `SchoolId`. When `maintenance_mode`
is true, the platform domain's authentication flow rejects
non-admin logins for the school.

### Invariants

1. A `MaintenanceSetting` exists at most once per `SchoolId`.
2. `maintenance_mode` is a boolean.
3. `title` and `sub_title` are non-empty (the default values
   are used when the operator leaves them blank).
4. `image` is a file reference or empty.
5. `applicable_for` is a string indicating which user types are
   affected (e.g. `all`, `student,parent`).

### Commands

- `ConfigureMaintenance`
- `EnableMaintenance`
- `DisableMaintenance`

### Events

- `MaintenanceConfigured`
- `MaintenanceEnabled`
- `MaintenanceDisabled`

---

## Sidebar

**Root type:** `Sidebar`
**Identity:** `SidebarId(SchoolId, Uuid)`

### Purpose

A per-role, per-permission sidebar layout projection. The base
`Sidebar` row carries the `permission_id`, `position`,
`section_id`, `parent`, `parent_route`, `level`, `user_id`
(creator), `is_saas` flag, `ignore` flag, `role_id`, and
`active_status`.

### Invariants

1. A `Sidebar` belongs to exactly one school.
2. A `Sidebar::permission_id` references a `Permission` row in
   the RBAC domain.
3. A `Sidebar::role_id` references a `Role` row in the RBAC
   domain.
4. A `Sidebar::level` is `Parent` (1), `Child` (2), or
   `SubChild` (3).
5. A `Sidebar::position` is a non-negative integer.
6. A `Sidebar::active_status` is a boolean.

### Commands

- `CreateSidebarEntry`
- `UpdateSidebarEntry`
- `DeleteSidebarEntry`
- `ReorderSidebar`

### Events

- `SidebarEntryCreated`
- `SidebarEntryUpdated`
- `SidebarEntryDeleted`
- `SidebarReordered`
