# Platform Domain — Tables

The platform domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                          | Aggregate                 | Notes                              |
| ------------------------------ | ------------------------- | ---------------------------------- |
| `comments`                     | Comment                   | Free-text comments with tagging    |
| `comment_pivots`               | CommentPivot              | Join between comments and tags     |
| `comment_tags`                 | CommentTag                | Tag values                         |
| `continents`                   | Continent                 | Continent catalog (global)         |
| `continents_typo_legacy`       | (drop)                    | Legacy misspelling; replaced by `continents` |
| `countries`                    | Country                   | Country catalog (global)           |
| `platform_module_infos`       | ModuleInfo                | Module display info projection     |
| `platform_module_managers`    | ModuleManager             | Per-tenant module manager         |
| `languages`                    | Language (per-school)     | Per-school language list           |
| `personal_access_tokens`       | PersonalAccessToken        | API tokens (global)                |
| `plugins`                      | Plugin                    | Front-office plugin catalog        |
| `school_modules`               | SchoolModule              | Per-school module enablement       |
| `platform_add_ons`             | AddOn                     | Registered add-ons                 |
| `platform_amount_transfers`    | AmountTransfer            | Fund transfers                     |
| `platform_base_groups`         | BaseGroup                 | Base-setup groupings               |
| `platform_chart_of_accounts`   | ChartOfAccount            | Accounting heads                   |
| `platform_countries`           | Country (canonical)       | Country catalog                    |
| `platform_courses`             | Course                    | Online courses                     |
| `platform_course_categories`   | CourseCategory            | Course groupings                   |
| `platform_currencies`          | Currency                  | Currency catalog                   |
| `platform_custom_fields`       | CustomField               | Custom field definitions           |
| `platform_custom_field_values` | CustomFieldValue          | Custom field values                |
| `platform_expert_teachers`     | ExpertTeacher             | Featured staff                     |
| `platform_frontend_permissions`| FrontendPermission        | Public-facing permissions          |
| `platform_header_menu_managers`| HeaderMenuManager         | Header menu items                  |
| `platform_instructions`        | Instruction               | Front-office instructions          |
| `platform_modules`             | Module                    | Top-level functional areas         |
| `platform_module_links`        | ModuleLink                | Menu items within modules          |
| `platform_student_parent_menus`| ModuleStudentParentInfo | Student/parent menu visibility    |
| `platform_photo_galleries`     | PhotoGallery              | Photo galleries                    |
| `platform_schools`             | School                    | The tenant root                    |
| `platform_social_media_icons`  | SocialMediaIcon           | Social links                       |
| `platform_time_zones`          | TimeZone                  | Timezone catalog (global)          |
| `platform_to_dos`              | ToDo                      | To-do items                        |
| `platform_video_galleries`     | VideoGallery              | Video galleries                    |
| `platform_visitors`            | Visitor                   | Visitor log entries                |
| `platform_users`               | User                      | The actor                          |
| `user_otp_codes`               | OtpCode                   | OTP rows                           |
| `video_uploads`                | VideoUpload               | Class-section video uploads        |

## Notes

- The bootstrap `School` (id `PLATFORM_BOOTSTRAP`) is the engine's
  seed school and cannot be deleted.
- `platform_users.role_id` references `rbac_roles` (RBAC domain)
  with `ON DELETE RESTRICT` per `docs/schemas/database-schema.md` § 4.
- `platform_users.school_id` is the tenant anchor; every per-school
  table carries it.
- `platform_student_parent_menus` carries `modules` and `menus`
  as `JSON` blobs.
- `platform_frontend_permissions` is the correctly-spelled
  canonical name. The legacy `cms_frontend_permissions` had a
  double-`s` typo; the engine renames it per
  `docs/schemas/data-migration/06-field-data-flow.md`.
- `platform_currencies` carries `currency_type` and
  `currency_position` as `VARCHAR(2)` encoded values.
- `platform_currencies.school_id` is the tenant anchor; the school
  controls the format (decimal separator, etc.) per locale.
- `platform_module_managers` is the engine's per-tenant module
  manager. It supersedes the legacy `platform_module_managers`
  shadow aggregate; there is no separate SaaS-scoped record.
- `personal_access_tokens.tokenable_type` is the aggregate type
  name (e.g. `User`); `tokenable_id` is the local id within the
  tenant.
- `platform_visitordate` carries the visitor date as a `DATE`; the
  legacy `in_time` / `out_time` strings are parsed to `TIME` on
  backfill.

## Cross-Domain Tables (Referenced)

| Table                          | Owning Domain | Notes                                  |
| ------------------------------ | ------------- | -------------------------------------- |
| `platform_schools`             | platform      | Self-reference (FK target)             |
| `academic_academic_years`      | academic      | Referenced by `academic_id` FKs        |
| `rbac_roles`                    | rbac          | Referenced by `platform_users.role_id` |
| `platform_module_links`        | platform      | Referenced by RBAC's `RolePermission`  |
