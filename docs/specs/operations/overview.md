# Operations Domain Overview

**Implementation crate:** `smsengine-operations` (path: `crates/operations/`)
**Spec status:** spec'd, scaffolded; implementation begins in Phase 14 (per `docs/build-plan.md`).

## Purpose

The operations domain owns the school's infrastructure-level
concerns: backups, jobs and failed jobs, system versions, the
version history, the per-school maintenance setting, the user log
(audit trail of logins), and the sidebar layout projection. The
operations domain also documents the OAuth and password-reset
infrastructure that consumer adapters implement, even though the
engine itself does not own those tables as aggregates.

## Responsibilities

- Backup lifecycle (database/file/image backup records and
  restore coordination).
- Job scheduling and failed-job tracking.
- System version records and version history.
- Maintenance mode (per-school "we are down for maintenance"
  state).
- User login log (per-login audit record).
- Sidebar layout projection (a per-role, per-user menu binding).
- Documentation of the OAuth token tables, password reset table,
  and migrations table as infrastructure concerns. These are
  port-driven in the engine; the operations domain is the home
  for their documentation and audit.

## Boundaries

The operations domain does **not** own:

- User identity, profile, or role binding — see `specs/platform/`
  and `specs/rbac/`.
- Settings, settings_themes, language — see `specs/settings/`.
- Academic, finance, attendance, etc.

The operations domain **does** provide:

- A canonical home for system-versioning records (so any domain
  can read the current version).
- The audit log projection for security reviews.
- The backup port (storage of backup files is the file-storage
  port; the operations domain owns the metadata and the
  lifecycle).

## Dependencies

- `smsengine-core` — error types, identifier trait.
- `smsengine-platform` — `SchoolId`, `UserId`, `TenantContext`.

## Domain Invariants

1. A `Backup` belongs to exactly one school and is identified by
   `(school_id, file_name)`.
2. A `Backup::file_type` is `Database`, `File`, or `Image`.
3. A `Job` is identified globally (jobs are not tenant-scoped;
   they are platform-internal).
4. A `FailedJob` is a terminal record; once failed, a job is
   not retried automatically.
5. A `SystemVersion` is identified by `version_name` (semantic
   version).
6. A `VersionHistory` row is append-only.
7. A `MaintenanceSetting` exists at most once per `SchoolId`.
8. A `UserLog` row is append-only; it is never updated.
9. A `Sidebar` row is identified by `(school_id, role_id,
   permission_id)`.
10. The `oauth_*`, `password_resets`, `migrations` tables are
    infrastructure tables documented for completeness; the
    engine treats them as port-driven and does not enforce
    invariants on them directly.

## Aggregate Roots

| Aggregate          | Root Type            | Purpose                                       |
| ------------------ | -------------------- | --------------------------------------------- |
| Backup             | `Backup`             | A database/file/image backup record           |
| Job                | `Job`                | A pending job in the queue                    |
| FailedJob          | `FailedJob`          | A job that has exhausted its retry budget     |
| SystemVersion      | `SystemVersion`      | A released version metadata record            |
| VersionHistory     | `VersionHistory`     | A version bump record                         |
| UserLog            | `UserLog`            | A login event record                          |
| MaintenanceSetting | `MaintenanceSetting` | The school's maintenance mode config          |
| Sidebar            | `Sidebar`            | A per-role sidebar layout projection          |

Each aggregate is documented in detail under
`docs/specs/operations/aggregates.md`.

## Infrastructure Tables (Documented, Not Owned)

The following tables are documented in `entities.md` and
`tables.md` for completeness. The engine treats them as
**port-driven**: consumer adapters may or may not implement them.
The operations domain does not own the aggregates behind them.

- `migrations` — migration tracking (consumer concern).
- `oauth_access_tokens` — OAuth bearer tokens (port concern).
- `oauth_auth_codes` — OAuth authorization codes (port concern).
- `oauth_clients` — OAuth client registrations (port concern).
- `oauth_personal_access_clients` — OAuth PAT clients (port
  concern).
- `oauth_refresh_tokens` — OAuth refresh tokens (port concern).
- `password_resets` — password reset requests (port concern).
- `personal_access_tokens` — personal access tokens (port concern,
  also documented in `specs/platform/`).

## Cross-Domain Impact

When a `Backup` is restored, the operations domain emits
`BackupRestored`. The platform domain subscribes to invalidate its
in-memory caches (capability cache, role cache, user cache).

When a `SystemVersion` is added (a new version is published), the
operations domain emits `SystemVersionBumped`. The settings
domain subscribes to refresh its `system_version` field on
`GeneralSettings`.

When a `UserLog` is recorded, the platform domain subscribes to
update the user's `last_login_at` (a derived field on the
`User` projection).

## Subscribers

- `platform` subscribes to `BackupRestored`,
  `UserLogged`, `SystemVersionBumped`.
- `settings` subscribes to `SystemVersionBumped`.

## Consumers

- Web admin UI (backup list, restore, system version,
  maintenance mode, audit log).
- Mobile apps (read system version for upgrade prompts).
- Operations tools (backup, restore, job queue).
- Security auditors (read user logs).

## Anti-Goals

- The operations domain does not implement authentication
  flows. OAuth, password reset, and personal access token tables
  are documented for completeness; the engine does not own
  their lifecycle.
- The operations domain does not run background jobs. The job
  runner is a port; the operations domain owns the queue
  records and the failed-job history.
- The operations domain does not perform the actual backup. The
  backup file is produced by an adapter (typically a database
  dump or filesystem snapshot); the operations domain records
  the metadata and coordinates the restore.
