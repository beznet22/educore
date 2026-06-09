# Settings Domain — Tables

The settings domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                          | Aggregate                 | Notes                              |
| ------------------------------ | ------------------------- | ---------------------------------- |
| `settings_behaviour_record_settings`    | BehaviorRecordSetting     | Behavior record feature flags      |
| `settings_colors`                       | Color                     | Color entries (global)             |
| `settings_color_theme`                  | ColorTheme                | Color bindings within settings_themes       |
| `settings_background_settings`       | BackgroundSetting         | Background image or color presets  |
| `settings_base_setups`               | BaseSetup                 | Lookup values                      |
| `settings_custom_links`              | CustomLink                | Footer/sidebar custom link bundle  |
| `settings_dashboard_settings`        | DashboardSetting          | Dashboard card bindings            |
| `settings_date_formats`              | DateFormat                | `strftime` patterns                |
| `settings_general_settings`          | GeneralSettings           | The school's primary config row    |
| `settings_languages`                 | Language                  | Per-school language list (legacy)  |
| `settings_language_phrases`          | LanguagePhrase            | Translatable phrase catalog        |
| `settings_setup_admins`              | SetupAdmin                | Purpose/complaint/source/reference |
| `settings_styles`                    | Style                     | Color/typography style profiles    |
| `settings_themes`                       | Theme                     | Theme registry                     |

## Notes

- The `settings_languages` table is the legacy per-school language row;
  the settings domain also reads the platform domain's global
  `platform_languages` table for the canonical ISO codes.
- `settings_general_settings` is the single per-school configuration
  row; the engine enforces uniqueness through the unique index on
  `school_id`.
- `settings_general_settings.two_factor` is a boolean enabling the 2FA
  flow at the school level; the RBAC domain's
  `two_factor_settings` table carries the per-role and per-channel
  policy.
- `settings_general_settings.currency_code` is the ISO code;
  `currency_symbol` is the display symbol; `currency_format` is the
  rendering mode.
- `settings_general_settings.active_theme` is a free-form theme name
  (e.g. `default`, `edulia`).
- `settings_general_settings.module_toggles` are stored as flat
  boolean/integer columns (e.g. `Lesson`, `Chat`, `FeesCollection`,
  `ResultReports`, `TemplateSettings`,
  `MenuManage`, `RolePermission`, `RazorPay`, `Saas`,
  `StudentAbsentNotification`, `ParentRegistration`, `Zoom`,
  `BBB`, `VideoWatch`, `Jitsi`, `OnlineExam`,
  `SaasRolePermission`, `BulkPrint`, `HimalayaSms`,
  `XenditPayment`, `Wallet`, `Lms`, `ExamPlan`, `University`,
  `Gmeet`, `KhaltiPayment`, `Raudhahpay`, `AppSlider`,
  `BehaviourRecords`, `DownloadCenter`, `AiContent`,
  `WhatsappSupport`, `InAppLiveClass`). **These are dropped
  in the engine migration; the engine's module system is
  capability-based and the consumer's `platform_packages.modules`
  JSON column carries the enabled modules.** The legacy
  `InfixBiometrics` column is included in this list as a brand
  artifact and is dropped.
- `settings_general_settings.api_url` is an integer (consumer-defined
  flag) used to switch API hosts.
- `settings_general_settings.ss_page_load` is a page-load count to
  seed (used by the consumer's bootstrap).
- `settings_general_settings.email_driver` selects the email port
  adapter (e.g. `smtp`, `sendmail`, `php`).
- `settings_general_settings.queue_connection` selects the queue port
  adapter.
- `settings_general_settings.preloader_status` is a boolean; style,
  type, and image are styling hints.
- `settings_general_settings.due_fees_login` controls whether login is
  blocked when fees are due.
- `settings_general_settings.phone_number_privacy` is `1 = masked`,
  `2 = visible`.
- `settings_general_settings.is_custom_saas` is a SaaS-only flag.
- `settings_general_settings.academic_id` is a legacy reference to
  the bootstrap academic year.
- `settings_general_settings.academic_id` is the active academic year
  (nullable).
- `settings_colors` and `settings_color_theme` are global (no `school_id`).
- `settings_color_theme.theme_id` references `settings_themes.id` and cascades on
  delete.
- `settings_color_theme.color_id` references `settings_colors.id` and cascades on
  delete.
- `settings_base_setups.base_group_id` references `settings_base_groups.id`
  and cascades on delete.
- `settings_dashboard_settings.role_id` references `roles.id` in the
  RBAC domain and cascades on delete.
- `settings_dashboard_settings.dashboard_sec_id` is an `i32` referencing
  a dashboard section id (consumer-defined).
- `settings_general_settings.date_format_id` references
  `settings_date_formats.id` and is nullable (legacy compatibility).
- `settings_general_settings.language_id` references `settings_languages.id`
  and is nullable.
- `settings_general_settings.session_id` references
  `academic_academic_years.id` in the academic domain and is nullable.
- `settings_themes.is_default` is `bool`; the engine refuses to delete a
  default theme.
- `settings_themes.is_system` is `bool`; system settings_themes are seeded by the
  engine.

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `platform_schools`           | platform      | Tenant anchor (FK target)              |
| `academic_academic_years`    | academic      | Referenced by `session_id`             |
| `settings_languages`         | settings      | Self-reference (FK target)             |
| `settings_date_formats`      | settings      | Self-reference (FK target)             |
| `rbac_role_prototypes`                | rbac          | Referenced by `role_id`                |
| `platform_time_zones`        | platform      | Referenced by `time_zone_id`           |
