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
| `continets`                    | Continent (legacy)        | Older alias of `continents`        |
| `countries`                    | Country                   | Country catalog (global)           |
| `infix_module_infos`           | InfixModuleInfo           | Module display info projection     |
| `infix_module_managers`        | ModuleManager (legacy)    | Saas-aware module manager          |
| `languages`                    | Language (per-school)     | Per-school language list           |
| `personal_access_tokens`       | PersonalAccessToken        | API tokens (global)                |
| `plugins`                      | Plugin                    | Front-office plugin catalog        |
| `school_modules`               | SchoolModule              | Per-school module enablement       |
| `sm_add_ons`                   | AddOn                     | Registered add-ons                 |
| `sm_amount_transfers`          | AmountTransfer            | Fund transfers                     |
| `sm_base_groups`               | BaseGroup                 | Base-setup groupings               |
| `sm_chart_of_accounts`         | ChartOfAccount            | Accounting heads                   |
| `sm_countries`                 | Country (legacy)          | Older alias of `countries`         |
| `sm_courses`                   | Course                    | Online courses                     |
| `sm_course_categories`         | CourseCategory            | Course groupings                   |
| `sm_currencies`                | Currency                  | Currency catalog                   |
| `sm_custom_fields`             | CustomField               | Custom field definitions           |
| `sm_custom_field_values`       | CustomFieldValue          | Custom field values                |
| `sm_expert_teachers`           | ExpertTeacher             | Featured staff                     |
| `sm_frontend_persmissions`     | FrontendPermission        | Public-facing permissions          |
| `sm_header_menu_managers`      | HeaderMenuManager         | Header menu items                  |
| `sm_instructions`              | Instruction               | Front-office instructions          |
| `sm_modules`                   | Module                    | Top-level functional areas         |
| `sm_module_links`              | ModuleLink                | Menu items within modules          |
| `sm_module_student_parent_infos` | ModuleStudentParentInfo | Student/parent menu visibility    |
| `sm_photo_galleries`           | PhotoGallery              | Photo galleries                    |
| `sm_schools`                   | School                    | The tenant root                    |
| `sm_social_media_icons`        | SocialMediaIcon           | Social links                       |
| `sm_time_zones`                | TimeZone                  | Timezone catalog (global)          |
| `sm_to_dos`                    | ToDo                      | To-do items                        |
| `sm_video_galleries`           | VideoGallery              | Video galleries                    |
| `sm_visitors`                  | Visitor                   | Visitor log entries                |
| `users`                        | User                      | The actor                          |
| `user_otp_codes`               | OtpCode                   | OTP rows                           |
| `video_uploads`                | VideoUpload               | Class-section video uploads        |

## Notes

- The bootstrap `School` (id 1) is the engine's seed school and
  cannot be deleted.
- `users.role_id` references `infix_roles` (RBAC domain) and is
  cascade-deleted.
- `users.school_id` is the tenant anchor; every per-school table
  carries it.
- `sm_module_student_parent_infos` carries `modules` and `menus`
  as `longtext` JSON blobs.
- `sm_frontend_persmissions` is misspelled in the legacy schema
  (note the double `ss`); the engine's domain spelling is
  `FrontendPermission`.
- `sm_currencies` carries `currency_type` and
  `currency_position` as `varchar(2)` encoded values whose
  meaning is documented in the value objects.
- `sm_currencies.school_id` is the tenant anchor even though the
  ISO currency code is global; the school controls the format
  (decimal separator, etc.) per locale.
- `sm_currencies.academic_id` is a legacy compatibility field.
- `infix_module_managers` is a SaaS-aware legacy mirror of
  `sm_module_managers` (which is defined in the RBAC domain's
  storage layer but logically owned by the platform).
- `personal_access_tokens.tokenable_type` is the aggregate type
  name (e.g. `User`); `tokenable_id` is the local id within the
  tenant.
- `sm_visitordate` carries the visitor date as a `date` (no
  time); `in_time` and `out_time` are stored as strings
  `"HH:MM:SS"` to preserve the legacy format.
- `sm_add_ons` has no columns in the migration; the engine
  treats the row existence as the registration record. The
  metadata is stored in a sibling `add_on_manifests` view that
  the engine materializes from a JSON config.

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `sm_schools`           | platform      | Self-reference (FK target)             |
| `sm_academic_years`    | academic      | Referenced by `academic_id` FKs        |
| `infix_roles`          | rbac          | Referenced by `users.role_id`          |
| `sm_module_links`      | platform      | Referenced by RBAC's `RolePermission`  |
