# RBAC Domain — Tables

The RBAC domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                            | Aggregate                | Notes                                          |
| -------------------------------- | ------------------------ | ---------------------------------------------- |
| `assign_permissions`             | AssignPermission         | Capability-to-role grant with overrides        |
| `permissions`                    | Capability (storage row) | Catalog row carrying capability + metadata     |
| `permission_sections`            | PermissionSection        | UI grouping label                              |
| `roles`                          | Role                     | A role within a school                         |
| `rbac_module_permissions`        | ModulePermission         | A dashboard-level permission group             |
| `rbac_module_permission_assigns` | ModulePermissionAssign   | A module-permission-to-role grant              |
| `rbac_role_permissions`          | RolePermission           | A module-link-to-role grant (menu binding)     |
| `two_factor_settings`            | TwoFactorSetting         | The school's 2FA policy                        |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `CHAR(36) NOT NULL` (UUIDv7) for the active school.
- Every table includes `id`, `created_at`, `updated_at`,
  `created_by`, `updated_by`, `active_status`, `version`, `etag`,
  `last_event_id`, `correlation_id`, `source`. These are managed
  by the engine's storage adapter; the seven engine invariants
  per `docs/schemas/database-schema.md` § 2, § 5, § 9.
- `permissions.module` is a denormalized grouping key (e.g.
  `Student`, `Finance`). It is not the same as the engine's domain
  name.
- `permissions.type` carries `1=Menu`, `2=SubMenu`, `3=Action`.
- `permissions.lang_name` is an i18n key; the translated text is
  read from the settings domain's phrase catalog.
- `rbac_role_permissions.module_link_id` references the
  `platform_module_links` table owned by the platform domain.
- `two_factor_settings.for_student|for_parent|for_teacher|for_staff|for_admin`
  is encoded `1=Required`, `2=Optional`, `3=Disabled`.
- `two_factor_settings.expired_time` is a numeric in seconds.
- `roles.is_replicated` indicates that the role is replicated to
  sibling schools in SaaS mode. The engine's `Role` is the only
  record (the legacy `InfixRole` shadow aggregate is removed).

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `platform_schools`     | platform      | Parent of every RBAC row (FK target)   |
| `platform_module_links`| platform      | Referenced by `rbac_role_permissions`    |
| `platform_users`       | platform      | Referenced by audit columns            |
| `rbac_roles`           | rbac          | Self-reference (FK target on `parent_role_id`) |
