# Operations Domain — Business Analysis

## Purpose

The operations domain owns the school's
infrastructure-level concerns: backups, jobs and
failed jobs, system versions, maintenance mode,
the user log (login audit), and the sidebar layout
projection. It is the engine's hygiene layer.

This document describes how these operational
concerns work in real schools, with the edge
cases that real schools hit.

## Key Concepts

- **Backup** — a database / file / image backup
  record.
- **Job** — a pending background job.
- **FailedJob** — a job that exhausted its retry
  budget.
- **SystemVersion** — a released version of the
  engine.
- **VersionHistory** — a version bump record.
- **MaintenanceSetting** — the school's
  maintenance mode configuration.
- **UserLog** — a login event record.
- **Sidebar** — a per-role, per-user sidebar
  layout projection.

## Real-World Scenarios

### Backup

The school backs up its data regularly:

1. The admin (or the consumer's scheduled job)
   triggers a backup.
2. The engine's storage adapter exports the
   database to a file (`.sql` for PostgreSQL,
   `.sqlite3` for SQLite).
3. The file is uploaded to the file storage
   (S3, GCS, local).
4. The engine's `Backup` aggregate records the
   backup with the timestamp, the size, the
   type (`Database`, `File`, `Image`), and the
   storage reference.
5. The admin sees the backup in the backup
   list.

A real school's backup policy:
- **Daily database backup** — automated at a
  scheduled time.
- **Weekly full backup** — including uploaded
  files and images.
- **Monthly archival** — to a long-term
  storage (Glacier, cold storage).
- **Retention** — daily backups kept for 30
  days; weekly for 12 weeks; monthly for 7
  years.

### Backup Restore

A disaster strikes (database corruption, accidental
deletion). The admin restores from a backup:

1. The admin selects the backup to restore.
2. The admin confirms (a destructive
   operation).
3. The engine's storage adapter downloads
   the backup file and applies it to the
   database.
4. The engine emits `BackupRestored`.
5. The platform domain invalidates its
   in-memory caches.
6. The admin verifies the data.

A real school's restore is a rare, high-stakes
operation. The engine's audit log captures every
restore with the actor, the timestamp, and the
backup reference.

### Job Scheduling

The engine schedules background jobs:

- **Daily attendance reminder** — at 6pm, send
  a reminder to teachers who have not marked
  attendance.
- **Monthly payroll generation** — on the 25th,
  generate payroll for the current month.
- **Weekly library overdue check** — every
  Monday, mark overdue books.
- **Annual academic year transition** — on the
  configured date, close the current year and
  open the next.

The engine's `Job` aggregate captures a pending
job. The `JobQueue` is a port-driven
implementation (in-process, Redis, NATS).

### Failed Job

A job fails (e.g. SMS gateway is down). The
engine:

1. Retries the job with exponential backoff
   (configurable per job type).
2. After N retries, marks the job as
   `Failed`.
3. The `FailedJob` aggregate records the
   failure with the exception, the stack
   trace, and the retry count.
4. The admin is notified.
5. The admin can retry manually or discard.

In real schools, failed jobs are a normal
occurrence. The engine's job queue is
designed for at-least-once delivery with
idempotency.

### System Version

The engine has a `SystemVersion` aggregate
that records the current version. The
`VersionHistory` records every version bump
with the date, the changes, and the actor.

A school's admin sees the current version in
the settings page. When a new version is
released, the engine emits
`SystemVersionBumped`; the settings domain
refreshes its `system_version` field on
`GeneralSettings`.

### Maintenance Mode

A school may put the system in maintenance
mode for a planned outage (e.g. database
migration, hardware upgrade):

1. The admin enables maintenance mode with
   a title, a message, and an optional
   image.
2. The maintenance mode is effective
   immediately (or at a scheduled time).
3. The portal shows the maintenance page to
   users; only the school admin can log in.
4. The admin performs the maintenance.
5. The admin disables maintenance mode.
6. The portal returns to normal.

The engine's `MaintenanceSetting` aggregate
captures the configuration. The
`maintenance_mode` flag is checked at
authentication time.

### User Login Log

Every login is recorded:

- The user id.
- The login time.
- The IP address.
- The user agent.
- The outcome (success, failure, locked).

The engine's `UserLog` aggregate captures
every login event. The audit log mirrors the
event. The platform domain's
`User::last_login_at` is updated on success.

### Login Failure Alert

A user has 5 failed login attempts. The engine
emits `LoginFailedRepeatedly`. The security
admin is alerted. The user's account is
locked for N minutes (per the school's
policy). The admin can unlock manually.

### Sidebar Layout

The engine's UI has a sidebar (navigation
menu). The sidebar items are capability-
driven. The engine's `Sidebar` aggregate
captures the per-role / per-user sidebar
configuration:

- The order of items.
- The visibility of items.
- The grouping of items.

The sidebar is a **projection** over the
capability catalog. The platform domain
subscribes to `CapabilityAssigned` /
`CapabilityRevoked` and refreshes the
sidebar.

### Database Migration

The engine's schema evolves. The consumer
runs migrations:

1. The consumer's `smscore migrate` CLI
   command runs.
2. The engine's storage adapter applies the
   pending migrations.
3. The migration is recorded in the
   `migrations` table.
4. The engine emits `MigrationApplied`.

Migrations are **owned by the consumer**, not
by the engine. The engine's storage port
defines the schema; the consumer implements
the migration strategy.

### Scheduled Maintenance Job

A school has a scheduled maintenance job
(e.g. "purge idempotency records older than
7 days"):

1. The engine's job scheduler runs the job
   at the configured time.
2. The job purges the records.
3. The job is marked `Completed`.
4. The audit log records the purge.

## Business Rules

1. A `Backup` belongs to exactly one school.
2. A `Backup::file_type` is `Database`,
   `File`, or `Image`.
3. A `Job` is identified globally (jobs are
   not tenant-scoped; they are platform-
   internal).
4. A `FailedJob` is a terminal record; once
   failed, a job is not retried automatically.
5. A `SystemVersion` is identified by
   `version_name` (semantic version).
6. A `VersionHistory` row is append-only.
7. A `MaintenanceSetting` exists at most once
   per `SchoolId`.
8. A `UserLog` row is append-only; it is
   never updated.
9. A `Sidebar` row is identified by
   `(school_id, role_id, permission_id)`.
10. A `Backup` cannot be deleted from the
    engine; it can be removed from storage
    (per the retention policy).
11. A `MaintenanceSetting` with
    `maintenance_mode = true` blocks all
    non-admin logins.

## Edge Cases

### Backup During Heavy Load

A backup is triggered during peak load
(attendance marking for 1,000 students).
The engine's backup port uses a separate
connection / transaction; it does not block
the main workload.

### Backup File Too Large

A school's database is 50GB. The backup
file is 50GB. The upload to S3 takes 30
minutes. The engine's backup command
returns a `BackupPending` status; a
background process uploads and updates the
status to `Completed`.

### Failed Job Retry

A job fails with a transient error (network
timeout). The engine retries with
exponential backoff. After 3 retries, the
job succeeds. The audit log captures the
retries.

### Maintenance Mode Mid-Operation

A school admin enables maintenance mode
while a teacher is marking attendance. The
teacher's command is in flight. The engine
allows the in-flight command to complete;
subsequent commands are rejected until
maintenance mode is disabled.

### Failed Login Detection

An attacker tries to log in as the school
admin. The engine detects the repeated
failures (5 in 5 minutes). The account is
locked; the security admin is alerted; the
audit log captures the incident.

### Sidebar for New Role

A school creates a new role (e.g. "Bus
Driver"). The engine's sidebar projection
creates default entries based on the role's
capabilities. The admin customizes the
sidebar order and visibility.

### Backup Retention Expiry

A daily backup is 8 days old. The retention
policy is 7 days. The engine's purge job
deletes the backup. The `Backup` record
remains in the engine (with a
`purged_at` timestamp) for audit; the
storage file is deleted.

### Job Run During Maintenance

A scheduled job runs while maintenance
mode is active. The job runs; the engine
does not block jobs during maintenance.
(Admins use maintenance mode for
infrastructure work, not for blocking
background tasks.)

## Notes for SMScore Implementation

- The **operations** crate depends on
  `smscore-platform` for `SchoolId` and
  `UserId`, and on `smscore-events` for
  event publishing.
- The operations domain is **append-only**
  for its core entities. `UserLog`,
  `VersionHistory`, `FailedJob` are never
  updated.
- The operations domain's **backups** are
  storage-port driven. The engine's
  `Backup` aggregate is the metadata; the
  file is in the consumer's file storage.
- The operations domain's **jobs** are
  queue-port driven. The consumer provides
  the queue implementation.
- The operations domain's **maintenance
  mode** is checked at authentication
  time. The engine's auth provider enforces
  the block.
- The operations domain's **login log**
  feeds the platform domain's
  `User::last_login_at` and the security
  admin's alerts.
- The operations domain's **sidebar** is
  a projection. The platform domain
  subscribes to capability changes and
  refreshes the sidebar.
- The operations domain's **migrations**
  are consumer-owned. The engine's storage
  port defines the schema; the consumer
  implements the migration strategy.
- The operations domain's **audit log**
  is the same engine-wide audit log. Every
  operational event is recorded.
