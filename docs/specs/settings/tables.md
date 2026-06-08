# Settings Domain — Tables

The settings domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you
which aggregate owns the row.

| Table                          | Aggregate                 | Notes                              |
| ------------------------------ | ------------------------- | ---------------------------------- |
| `behaviour_record_settings`    | BehaviorRecordSetting     | Behavior record feature flags      |
| `colors`                       | Color                     | Color entries (global)             |
| `color_theme`                  | ColorTheme                | Color bindings within themes       |
| `sm_background_settings`       | BackgroundSetting         | Background image or color presets  |
| `sm_base_setups`               | BaseSetup                 | Lookup values                      |
| `sm_custom_links`              | CustomLink                | Footer/sidebar custom link bundle  |
| `sm_dashboard_settings`        | DashboardSetting          | Dashboard card bindings            |
| `sm_date_formats`              | DateFormat                | `strftime` patterns                |
| `sm_general_settings`          | GeneralSettings           | The school's primary config row    |
| `sm_languages`                 | Language                  | Per-school language list (legacy)  |
| `sm_language_phrases`          | LanguagePhrase            | Translatable phrase catalog        |
| `sm_setup_admins`              | SetupAdmin                | Purpose/complaint/source/reference |
| `sm_styles`                    | Style                     | Color/typography style profiles    |
| `themes`                       | Theme                     | Theme registry                     |

## Notes

- The `sm_languages` table is the legacy per-school language row;
  the settings domain also reads the platform domain's global
  `languages` table for the canonical ISO codes.
- `sm_general_settings` is the single per-school configuration
  row; the engine enforces uniqueness through the unique index on
  `school_id`.
- `sm_general_settings.two_factor` is a boolean enabling the 2FA
  flow at the school level; the RBAC domain's
  `two_factor_settings` table carries the per-role and per-channel
  policy.
- `sm_general_settings.currency_code` is the ISO code;
  `currency_symbol` is the display symbol; `currency_format` is the
  rendering mode.
- `sm_general_settings.active_theme` is a free-form theme name
  (e.g. `default`, `edulia`).
- `sm_general_settings.module_toggles` are stored as flat
  boolean/integer columns (e.g. `Lesson`, `Chat`, `FeesCollection`,
  `InfixBiometrics`, `ResultReports`, `TemplateSettings`,
  `MenuManage`, `RolePermission`, `RazorPay`, `Saas`,
  `StudentAbsentNotification`, `ParentRegistration`, `Zoom`,
  `BBB`, `VideoWatch`, `Jitsi`, `OnlineExam`,
  `SaasRolePermission`, `BulkPrint`, `HimalayaSms`,
  `XenditPayment`, `Wallet`, `Lms`, `ExamPlan`, `University`,
  `Gmeet`, `KhaltiPayment`, `Raudhahpay`, `AppSlider`,
  `BehaviourRecords`, `DownloadCenter`, `AiContent`,
  `WhatsappSupport`, `InAppLiveClass`).
- `sm_general_settings.api_url` is an integer (consumer-defined
  flag) used to switch API hosts.
- `sm_general_settings.ss_page_load` is a page-load count to
  seed (used by the consumer's bootstrap).
- `sm_general_settings.email_driver` selects the email port
  adapter (e.g. `smtp`, `sendmail`, `php`).
- `sm_general_settings.queue_connection` selects the queue port
  adapter.
- `sm_general_settings.preloader_status` is a boolean; style,
  type, and image are styling hints.
- `sm_general_settings.due_fees_login` controls whether login is
  blocked when fees are due.
- `sm_general_settings.phone_number_privacy` is `1 = masked`,
  `2 = visible`.
- `sm_general_settings.is_custom_saas` is a SaaS-only flag.
- `sm_general_settings.un_academic_id` is a legacy reference to
  the bootstrap academic year.
- `sm_general_settings.academic_id` is the active academic year
  (nullable).
- `colors` and `color_theme` are global (no `school_id`).
- `color_theme.theme_id` references `themes.id` and cascades on
  delete.
- `color_theme.color_id` references `colors.id` and cascades on
  delete.
- `sm_base_setups.base_group_id` references `sm_base_groups.id`
  and cascades on delete.
- `sm_dashboard_settings.role_id` references `roles.id` in the
  RBAC domain and cascades on delete.
- `sm_dashboard_settings.dashboard_sec_id` is an `i32` referencing
  a dashboard section id (consumer-defined).
- `sm_general_settings.date_format_id` references
  `sm_date_formats.id` and is nullable (legacy compatibility).
- `sm_general_settings.language_id` references `sm_languages.id`
  and is nullable.
- `sm_general_settings.session_id` references
  `sm_academic_years.id` in the academic domain and is nullable.
- `themes.is_default` is `bool`; the engine refuses to delete a
  default theme.
- `themes.is_system` is `bool`; system themes are seeded by the
  engine.

## Cross-Domain Tables (Referenced)

| Table                  | Owning Domain | Notes                                  |
| ---------------------- | ------------- | -------------------------------------- |
| `sm_schools`           | platform      | Tenant anchor (FK target)              |
| `sm_academic_years`    | academic      | Referenced by `session_id`             |
| `sm_languages`         | settings      | Self-reference (FK target)             |
| `sm_date_formats`      | settings      | Self-reference (FK target)             |
| `roles`                | rbac          | Referenced by `role_id`                |
| `sm_time_zones`        | platform      | Referenced by `time_zone_id`           |
