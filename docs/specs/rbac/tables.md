# RBAC Domain — Tables

The RBAC domain is backed by the following tables. Each table maps
to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                            | Aggregate                | Notes                                          |
| -------------------------------- | ------------------------ | ---------------------------------------------- |
| `assign_permissions`             | AssignPermission         | Capability-to-role grant with overrides        |
| `infix_permission_assigns`       | InfixPermissionAssign    | SaaS shadow of `assign_permissions`            |
| `infix_roles`                    | InfixRole                | SaaS shadow of `roles`                         |
| `permissions`                    | Capability (storage row) | Catalog row carrying capability + metadata     |
| `permission_sections`            | PermissionSection        | UI grouping label                              |
| `roles`                          | Role                     | A role within a school                         |
| `sm_module_permissions`          | ModulePermission         | A dashboard-level permission group             |
| `sm_module_permission_assigns`   | ModulePermissionAssign   | A module-permission-to-role grant              |
| `sm_role_permissions`            | RolePermission           | A module-link-to-role grant (menu binding)     |
| `two_factor_settings`            | TwoFactorSetting         | The school's 2FA policy                        |

## Notes

- Every table includes `school_id` for multi-tenant isolation. The
  `school_id` is `NOT NULL DEFAULT 1` for the bootstrap school.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are
  managed by the engine's storage adapter.
- `permissions.module` is a denormalized grouping key (e.g. `Student`,
  `Finance`). It is not the same as the engine's domain name.
- `permissions.type` carries `1=Menu`, `2=SubMenu`, `3=Action`.
- `permissions.lang_name` is an i18n key; the translated text is
  read from the settings domain's phrase catalog.
- `sm_role_permissions.module_link_id` references the
  `sm_module_links` table owned by the platform domain.
- `two_factor_settings.for_student|for_parent|for_teacher|for_staff|for_admin`
  is encoded `1=Required`, `2=Optional`, `3=Disabled`.
- `two_factor_settings.expired_time` is a double in seconds.
- `infix_roles.is_saas` indicates that the role is replicated to
  sibling schools in SaaS mode.

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `sm_schools`           | platform      | Parent of every RBAC row (FK target)   |
| `sm_module_links`      | platform      | Referenced by `sm_role_permissions`    |
| `users`                | platform      | Referenced by audit columns            |
| `infix_roles`          | rbac          | Self-reference (FK target)             |
