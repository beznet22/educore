# Settings Domain — Aggregates

## GeneralSettings

**Root type:** `GeneralSettings`
**Identity:** `GeneralSettingsId(SchoolId, Uuid)`

### Purpose

The single, per-school configuration row that captures the school's
identity, contact, currency, language, theme, file size, copyright,
and a large set of feature flags. There is at most one
`GeneralSettings` row per `SchoolId`.

### Owned Children

None. The `GeneralSettings` row references many other aggregates
(by id) and is itself a leaf.

### Invariants

1. A `GeneralSettings` exists at most once per `SchoolId`.
2. `school_name`, `site_title` are non-empty.
3. `school_code` matches the school's `SchoolCode`.
4. `currency` is a valid `CurrencyCode`.
5. `currency_symbol` is non-empty.
6. `currency_format` is `SymbolAmount`, `AmountSymbol`, or a
   consumer-defined variant.
7. `file_size` is a positive integer (bytes).
8. `system_version` is a semantic version string.
9. `language_name` is a valid `LanguageCode`.
10. `session_year` is a string (e.g. `"2025-2026"`) or
    `AcademicYearId`.
11. `session_id` references the currently selected academic year.
12. `language_id` references a `LanguageId`.
13. `date_format_id` references a `DateFormatId`.
14. `time_zone_id` references a `TimeZoneId`.
15. `active_status` is a boolean.
16. The `two_factor` flag is a boolean enabling the 2FA flow.
17. The `active_theme` is a non-empty theme name.
18. The `is_email_verified` flag is a boolean.
19. The `copyright_text` is a non-empty string.
20. `preloader_status`, `preloader_style`, `preloader_type` are
    numeric codes.
21. `email_driver` is a string identifier for the email port
    adapter.
22. `queue_connection` is a string identifier for the queue port
    adapter.

### Commands

- `UpdateGeneralSettings`
- `SelectActiveTheme`
- `SelectLanguage`
- `SelectDateFormat`
- `SelectTimeZone`
- `SelectSession`
- `EnableTwoFactor` / `DisableTwoFactor`

### Events

- `GeneralSettingsUpdated`
- `ActiveThemeChanged`
- `LanguageChanged`
- `DateFormatChanged`
- `TimeZoneChanged`
- `SessionChanged`
- `TwoFactorToggled`

### Consistency Boundary

The `GeneralSettings` aggregate is loaded, mutated, and persisted
in a single transaction. Concurrent `UpdateGeneralSettings`
commands are serialized through an optimistic version check.

---

## Language

**Root type:** `Language`
**Identity:** `LanguageId(SchoolId, Uuid)`

### Purpose

A language registered in the school. The settings domain owns the
school-level language list; the per-locale phrase catalog is owned
by `LanguagePhrase`.

### Invariants

1. A `Language::code` is unique within `(school_id, code)`.
2. A `Language::name` is non-empty.
3. A `Language::native` is non-empty.
4. `rtl` is a boolean.
5. `active_status` is a boolean.

### Commands

- `AddLanguage`
- `UpdateLanguage`
- `DeleteLanguage`
- `ActivateLanguage`
- `DeactivateLanguage`

### Events

- `LanguageAdded`
- `LanguageUpdated`
- `LanguageDeleted`
- `LanguageActivated`
- `LanguageDeactivated`

---

## LanguagePhrase

**Root type:** `LanguagePhrase`
**Identity:** `LanguagePhraseId(SchoolId, Uuid)`

### Purpose

A translatable phrase key. The base row stores the `modules` and
`default_phrases`; per-locale columns (`en`, `es`, `bn`, `fr`)
hold the translations. The engine treats the locale columns as
opaque strings; the i18n port maps them to typed locales.

### Invariants

1. A `LanguagePhrase::modules` is non-empty.
2. A `LanguagePhrase::default_phrases` is non-empty.
3. Each non-null locale column is a string of 1..65000 chars.
4. `active_status` is a boolean.

### Commands

- `AddLanguagePhrase`
- `UpdateLanguagePhrase`
- `DeleteLanguagePhrase`
- `TranslateLanguagePhrase`

### Events

- `LanguagePhraseAdded`
- `LanguagePhraseUpdated`
- `LanguagePhraseDeleted`
- `LanguagePhraseTranslated`

---

## BaseSetup

**Root type:** `BaseSetup`
**Identity:** `BaseSetupId(SchoolId, Uuid)`

### Purpose

A lookup value. Owned by the settings domain but consumed by every
domain that needs configurable dropdowns. Belongs to a `BaseGroup`.

### Invariants

1. A `BaseSetup::base_setup_name` is non-empty.
2. A `BaseSetup` references exactly one `BaseGroup`.
3. The pair `(base_group_id, base_setup_name)` is unique within
   `school_id`.

### Commands

- `AddBaseSetup`
- `UpdateBaseSetup`
- `DeleteBaseSetup`

### Events

- `BaseSetupAdded`
- `BaseSetupUpdated`
- `BaseSetupDeleted`

---

## BaseGroup

**Root type:** `BaseGroup`
**Identity:** `BaseGroupId(SchoolId, Uuid)`

### Purpose

A grouping of `BaseSetup` values.

### Invariants

1. A `BaseGroup::name` is unique within `(school_id, name)`.
2. A `BaseGroup` cannot be deleted if any `BaseSetup` references
   it.

### Commands

- `AddBaseGroup`
- `UpdateBaseGroup`
- `DeleteBaseGroup`

### Events

- `BaseGroupAdded`
- `BaseGroupUpdated`
- `BaseGroupDeleted`

---

## DateFormat

**Root type:** `DateFormat`
**Identity:** `DateFormatId(SchoolId, Uuid)`

### Purpose

A `strftime` pattern with a human-readable preview (`normal_view`).

### Invariants

1. A `DateFormat::format` is a valid `strftime` pattern.
2. A `DateFormat::normal_view` is the human-readable example
   (e.g. `"YYYY-MM-DD"`).
3. `active_status` is a boolean.

### Commands

- `AddDateFormat`
- `UpdateDateFormat`
- `DeleteDateFormat`

### Events

- `DateFormatAdded`
- `DateFormatUpdated`
- `DateFormatDeleted`

---

## Style

**Root type:** `Style`
**Identity:** `StyleId(SchoolId, Uuid)`

### Purpose

A color palette and chart palette. Drives the dashboard chart
settings_colors, sidebar background, and primary text/background settings_colors.

### Invariants

1. A `Style::style_name` is unique within `(school_id, style_name)`.
2. `is_active` and `is_default` are booleans; at most one style
   per school may be `is_default=true`.
3. `active_status` is a boolean.

### Commands

- `CreateStyle`
- `UpdateStyle`
- `ActivateStyle`
- `DeleteStyle`

### Events

- `StyleCreated`
- `StyleUpdated`
- `StyleActivated`
- `StyleDeleted`

---

## BackgroundSetting

**Root type:** `BackgroundSetting`
**Identity:** `BackgroundSettingId(SchoolId, Uuid)`

### Purpose

A background image or color preset for the login screen or
dashboard.

### Invariants

1. A `BackgroundSetting::type` is `Image` or `Color`.
2. A `BackgroundSetting::image` is a file reference when
   `type=Image`.
3. A `BackgroundSetting::color` is a hex string when
   `type=Color`.
4. `is_default` is a boolean.

### Commands

- `CreateBackgroundSetting`
- `UpdateBackgroundSetting`
- `DeleteBackgroundSetting`

### Events

- `BackgroundSettingCreated`
- `BackgroundSettingUpdated`
- `BackgroundSettingDeleted`

---

## DashboardSetting

**Root type:** `DashboardSetting`
**Identity:** `DashboardSettingId(SchoolId, Uuid)`

### Purpose

A binding between a dashboard section/card and a role. Determines
which dashboard cards a role sees.

### Invariants

1. A `DashboardSetting::dashboard_sec_id` is a positive integer.
2. A `DashboardSetting::role_id` references a valid `RoleId`.
3. The pair `(dashboard_sec_id, role_id)` is unique within
   `school_id`.
4. `active_status` is a boolean.

### Commands

- `CreateDashboardSetting`
- `UpdateDashboardSetting`
- `DeleteDashboardSetting`

### Events

- `DashboardSettingCreated`
- `DashboardSettingUpdated`
- `DashboardSettingDeleted`

---

## CustomLink

**Root root:** `CustomLink`
**Identity:** `CustomLinkId(SchoolId, Uuid)`

### Purpose

The footer / sidebar custom link bundle. Each row carries up to 16
link label/URL pairs and 5 social URLs.

### Invariants

1. There is at most one `CustomLink` row per `SchoolId` (it is a
   per-school singleton).
2. Each `link_href_n` is a valid URL or empty.
3. Each `link_label_n` is non-empty when the href is set.
4. `facebook_url`, `twitter_url`, `dribble_url`, `linkedin_url`,
   `behance_url` are valid URLs or empty.

### Commands

- `UpdateCustomLinks`
- `ResetCustomLinks`

### Events

- `CustomLinksUpdated`
- `CustomLinksReset`

---

## ColorTheme

**Root type:** `ColorTheme`
**Identity:** `ColorThemeId(Uuid)` (global, not tenant-scoped;
the table is global in the engine's storage)

### Purpose

A color value within a theme. Each `ColorTheme` row binds a
`Color` to a `Theme` with a hex value.

### Invariants

1. A `ColorTheme` references exactly one `Color` and one `Theme`.
2. `value` is a valid hex color string.

### Commands

- `CreateColorTheme`
- `UpdateColorTheme`
- `DeleteColorTheme`

### Events

- `ColorThemeCreated`
- `ColorThemeUpdated`
- `ColorThemeDeleted`

---

## Theme

**Root type:** `Theme`
**Identity:** `ThemeId(SchoolId, Uuid)`

### Purpose

A theme (color mode, background, box shadow) used to render the
consumer.

### Invariants

1. A `Theme::title` is non-empty.
2. `color_mode` is `Gradient` or `Solid`.
3. `background_type` is `Image` or `Color`.
4. `background_color` is a hex string when `background_type=Color`.
5. `background_image` is a file reference when
   `background_type=Image`.
6. `box_shadow` is a boolean.
7. `is_default` is a boolean; the engine refuses to delete a
   default theme.
8. `is_system` is a boolean; system settings_themes are seeded by the
   engine and cannot be deleted.

### Commands

- `CreateTheme`
- `UpdateTheme`
- `ActivateTheme`
- `DeleteTheme`
- `ReplicateTheme`

### Events

- `ThemeCreated`
- `ThemeUpdated`
- `ThemeActivated`
- `ThemeDeleted`
- `ThemeReplicated`

---

## Color

**Root type:** `Color`
**Identity:** `ColorId(Uuid)` (global)

### Purpose

A color entry used by `ColorTheme`. The base row carries the
color's display name, a `lawn_green` value (a default green for
previewing), and a `default_value` (a hex).

### Invariants

1. A `Color::name` is non-empty.
2. `is_color` and `status` are booleans.
3. `default_value` is a valid hex color string.

### Commands

- `CreateColor`
- `UpdateColor`
- `DeleteColor`

### Events

- `ColorCreated`
- `ColorUpdated`
- `ColorDeleted`

---

## BehaviorRecordSetting

**Root root:** `BehaviorRecordSetting`
**Identity:** `BehaviorRecordSettingId(SchoolId, Uuid)`

### Purpose

The school's behavior record feature configuration. Carries four
boolean-ish flags: `student_comment`, `parent_comment`,
`student_view`, `parent_view`.

### Invariants

1. A `BehaviorRecordSetting` exists at most once per `SchoolId`.
2. Each flag is a non-negative integer (0/1; some legacy
   configurations may use 2 for "inherited").

### Commands

- `UpdateBehaviorRecordSetting`

### Events

- `BehaviorRecordSettingUpdated`

---

## SetupAdmin

**Root type:** `SetupAdmin`
**Identity:** `SetupAdminId(SchoolId, Uuid)`

### Purpose

A purpose, complaint type, source, or reference entry. The
`type` field discriminates: `1` = purpose, `2` = complaint type,
`3` = source, `4` = reference.

### Invariants

1. A `SetupAdmin::type` is in `1..=4`.
2. A `SetupAdmin::name` is non-empty.
3. `active_status` is a boolean.

### Commands

- `AddSetupAdmin`
- `UpdateSetupAdmin`
- `DeleteSetupAdmin`

### Events

- `SetupAdminAdded`
- `SetupAdminUpdated`
- `SetupAdminDeleted`
