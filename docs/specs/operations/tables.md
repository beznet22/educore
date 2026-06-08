# Operations Domain — Tables

The operations domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                  | Aggregate          | Notes                                  |
| ---------------------- | ------------------ | -------------------------------------- |
| `failed_jobs`          | FailedJob          | Terminal job failure records (global)  |
| `jobs`                 | Job                | Pending job queue (global)             |
| `maintenance_settings` | MaintenanceSetting | Per-school maintenance mode config     |
| `migrations`           | (infrastructure)   | Migration tracking (consumer concern)  |
| `oauth_access_tokens`  | (infrastructure)   | OAuth bearer tokens (port concern)     |
| `oauth_auth_codes`     | (infrastructure)   | OAuth authorization codes              |
| `oauth_clients`        | (infrastructure)   | OAuth client registrations             |
| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |
| `oauth_refresh_tokens` | (infrastructure)   | OAuth refresh tokens                   |
| `password_resets`      | (infrastructure)   | Password reset requests                |
| `sidebars`             | Sidebar            | Per-role sidebar layout projection     |
| `sm_backups`           | Backup             | Backup records                         |
| `sm_system_versions`   | SystemVersion      | Released version metadata              |
| `sm_user_logs`         | UserLog            | Login event audit log                  |
| `version_histories`    | VersionHistory     | Version bump records (global)          |

## Notes

- `failed_jobs.uuid` is a unique business identifier separate
  from the auto-increment `id`.
- `failed_jobs.payload` and `failed_jobs.exception` are
  `longtext` blobs; the engine validates the payload format at
  dequeue.
- `jobs.payload` is `longtext`; the engine validates the payload
  format at dequeue.
- `jobs.reserved_at` is a Unix timestamp (not a datetime); the
  engine interprets it as seconds since epoch.
- `jobs.attempts` is `tinyint unsigned`; values above 255 are
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
- `sm_backups.file_type` is encoded `0=Database`, `1=File`,
  `2=Image`.
- `sm_backups.source_link` is a URL or file-storage reference.
- `sm_backups.lang_type` is an `i32` consumer-defined hint.
- `sm_system_versions.version_name` is a unique semver string.
- `sm_user_logs` carries `ip_address` and `user_agent` as
  `varchar(191)`. The engine validates `ip_address` as IPv4 or
  IPv6; the validator normalizes v4-mapped v6 addresses to v4.
- `sm_user_logs.role_id` references `infix_roles` (RBAC
  domain) and cascades on delete.
- `sm_user_logs.user_id` references `users` (platform domain)
  and cascades on delete.
- `sm_user_logs.academic_id` references `sm_academic_years`
  (academic domain) and cascades on delete.
- `sidebars.permission_id` references `permissions` (RBAC
  domain) by legacy id (not a foreign key in the migration
  schema).
- `sidebars.user_id` references `users` (platform domain) and
  cascades on delete.
- `sidebars.level` is encoded `1=Parent`, `2=Child`,
  `3=SubChild`.
- `sidebars.parent` is an `i32` referencing another `sidebar`
  row (self-reference) by legacy id.
- `sidebars.parent_route` is an `i32` referencing a parent
  sidebar entry's route by legacy id.
- `sidebars.ignore` is `0=Show`, `1=Hide`, `2=Disabled`.
- `version_histories` is global; no `school_id`.
- `maintenance_settings.applicable_for` is a free-form string
  (e.g. `all`, `student,parent`).

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `sm_schools`           | platform      | Tenant anchor (FK target)              |
| `users`                | platform      | Referenced by `sm_user_logs`           |
| `infix_roles`          | rbac          | Referenced by `sm_user_logs`           |
| `sm_academic_years`    | academic      | Referenced by `sm_user_logs`           |
| `permissions`          | rbac          | Referenced by `sidebars`               |
| `sm_module_links`      | platform      | Referenced by `sidebars.parent_route`  |

## Infrastructure Tables

The `oauth_*`, `password_resets`, `migrations`, and
`personal_access_tokens` (the latter owned by the platform
domain) tables are documented here for completeness. The
engine treats them as port-driven: consumer adapters may or
may not implement them, and the engine does not enforce
invariants on them directly. The repositories in
`repositories.md` declare the port traits; concrete
implementations live in the consumer's auth adapter crate.
