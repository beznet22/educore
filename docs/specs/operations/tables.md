# Operations Domain — Tables

The operations domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                          | Aggregate          | Notes                                  |
| ------------------------------ | ------------------ | -------------------------------------- |
| `failed_jobs`                  | FailedJob          | Terminal job failure records (global)  |
| `jobs`                         | Job                | Pending job queue (global)             |
| `operations_maintenance_settings`         | MaintenanceSetting | Per-school maintenance mode config     |
| `migrations`                   | (infrastructure)   | Migration tracking (consumer concern)  |
| `oauth_access_tokens`          | (infrastructure)   | OAuth bearer tokens (port concern)     |
| `oauth_auth_codes`             | (infrastructure)   | OAuth authorization codes              |
| `oauth_clients`                | (infrastructure)   | OAuth client registrations             |
| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |
| `oauth_refresh_tokens`         | (infrastructure)   | OAuth refresh tokens                   |
| `password_resets`              | (infrastructure)   | Password reset requests                |
| `rbac_sidebars`                | Sidebar            | Per-role sidebar layout projection     |
| `operations_backups`           | Backup             | Backup records                         |
| `operations_system_versions`   | SystemVersion      | Released version metadata              |
| `operations_user_logs`         | UserLog            | Login event audit log                  |
| `operations_version_histories`            | VersionHistory     | Version bump records (global)          |

## Notes

- `failed_jobs.uuid` is a unique business identifier separate
  from the auto-increment `id`.
- `failed_jobs.payload` and `failed_jobs.exception` are
  `LONGTEXT` blobs; the engine validates the payload format at
  dequeue.
- `jobs.payload` is `LONGTEXT`; the engine validates the payload
  format at dequeue.
- `jobs.reserved_at` is a Unix timestamp (seconds since epoch).
- `jobs.attempts` is `TINYINT UNSIGNED`; values above 255 are
  impossible at the storage level.
- `migrations` is owned by the consumer's migration tool, not
  the engine. The operations domain documents it for
  completeness.
- `oauth_access_tokens.scopes` is a space-separated string.
- `oauth_access_tokens.revoked` is a string (`"1"` or `"0"`);
  the engine normalizes it to a boolean.
- `oauth_clients.secret` is hashed in storage.
- `password_resets.email` is indexed for lookup; the token is
  hashed.
- `operations_backups.file_type` is encoded `0=Database`,
  `1=File`, `2=Image`.
- `operations_backups.source_link` is a URL or file-storage reference.
- `operations_backups.lang_type` is an `INTEGER` consumer-defined hint.
- `operations_system_versions.version_name` is a unique semver string.
- `operations_user_logs` carries `ip_address` and `user_agent`
  as `VARCHAR(191)`. The engine validates `ip_address` as IPv4
  or IPv6; the validator normalizes v4-mapped v6 addresses to v4.
- `operations_user_logs.role_id` references `rbac_roles` (RBAC
  domain) with `ON DELETE RESTRICT` per § 4.
- `operations_user_logs.user_id` references `platform_users`
  (platform domain) with `ON DELETE RESTRICT`.
- `operations_user_logs.academic_id` references
  `academic_academic_years` (academic domain).
- `rbac_sidebars.permission_id` references `rbac_permissions`
  (RBAC domain).
- `rbac_sidebars.user_id` references `platform_users` (platform
  domain).
- `rbac_sidebars.level` is encoded `1=Parent`, `2=Child`,
  `3=SubChild`.
- `rbac_sidebars.parent` references another `rbac_sidebars` row
  (self-reference).
- `rbac_sidebars.parent_route` references a parent sidebar entry.
- `rbac_sidebars.ignore` is `0=Show`, `1=Hide`, `2=Disabled`.
- `rbac_sidebars.is_system_defined` flags system-defined rows;
  the engine refuses to delete them.
- `operations_version_histories` is global; no `school_id`.
- `operations_maintenance_settings.applicable_for` is a free-form string
  (e.g. `all`, `student,parent`).

## Cross-Domain Tables (Referenced)

| Table                          | Owning Domain | Notes                                  |
| ------------------------------ | ------------- | -------------------------------------- |
| `platform_schools`             | platform      | Tenant anchor (FK target)              |
| `platform_users`               | platform      | Referenced by `operations_user_logs`   |
| `rbac_roles`                    | rbac          | Referenced by `operations_user_logs`   |
| `academic_academic_years`      | academic      | Referenced by `operations_user_logs`   |
| `rbac_permissions`              | rbac          | Referenced by `rbac_sidebars`          |
| `platform_module_links`        | platform      | Referenced by `rbac_sidebars.parent_route` |

## Infrastructure Tables

The `oauth_*`, `password_resets`, `migrations`, and
`personal_access_tokens` (the latter owned by the platform
domain) tables are documented here for completeness. The
engine treats them as port-driven: consumer adapters may or
may not implement them, and the engine does not enforce
invariants on them directly. The repositories in
`repositories.md` declare the port traits; concrete
implementations live in the consumer's auth adapter crate.
