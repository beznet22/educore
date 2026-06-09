# Settings Domain Overview

## Purpose

The settings domain owns the school's cosmetic and behavioral
configuration: general settings, language catalog, language phrases,
base setup groupings, custom links, dashboard layout, settings_themes, color
settings_themes, date formats, styles, background settings, and the behaviour
record feature flag. The settings domain also owns the per-school
admin setup catalog (purpose, complaint type, source, reference).

## Responsibilities

- General school settings (name, address, contact, currency,
  language, session, file size, system version, copyright, etc.).
- Language and language phrase management.
- Base setup groupings and values (lookup tables).
- Custom links (footer / sidebar links).
- Dashboard card layout per role.
- Date format patterns.
- Style profiles (color palette, font, chart settings_colors).
- Color theme and color definitions.
- Theme registry (active theme, color mode, background).
- Background settings (image or color).
- Behavior record settings (comment/view flags for student and
  parent).
- Per-school admin setup catalog.

## Boundaries

The settings domain does **not** own:

- Capability-based permissions — see `specs/rbac/`.
- The course catalog or user identity — see `specs/platform/`.
- Backups, jobs, audit logs — see `specs/operations/`.
- The look-and-feel of the running consumer application
  (templates, CSS). The settings domain owns the **data**; the
  consumer renders.

## Dependencies

- `smsengine-core` — error types, identifier trait.
- `smsengine-platform` — `SchoolId`, `UserId`, `TenantContext`.

## Domain Invariants

1. A `GeneralSettings` row exists at most once per `SchoolId`.
2. A `GeneralSettings::session_id` references a valid
   `AcademicYearId` in the academic domain.
3. A `GeneralSettings::language_id` references a valid
   `LanguageId` in the platform domain.
4. A `GeneralSettings::date_format_id` references a valid
   `DateFormatId`.
5. A `GeneralSettings::time_zone_id` references a valid
   `TimeZoneId`.
6. A `Language` is unique by `(school_id, code)`.
7. A `LanguagePhrase::modules` and `default_phrases` are non-empty.
8. A `BaseSetup` belongs to exactly one `BaseGroup`.
9. A `BaseGroup::name` is unique within `(school_id, name)`.
10. A `CustomLink` is per-school; the `link_href_n` fields are
    validated as URLs when non-empty.
11. A `DashboardSetting::role_id` references a valid `RoleId`.
12. The pair `(dashboard_sec_id, role_id)` is unique within
    `(school_id)`.
13. A `DateFormat::format` is a valid `strftime` pattern.
14. A `Style` is unique by `(school_id, style_name)`.
15. A `Theme::is_default=true` means it is the engine's bundled
    default; the engine refuses to delete a default theme.
16. A `Color` may be referenced by a `ColorTheme` row; deleting a
    `Color` cascades to its `ColorTheme` rows.
17. A `BehaviorRecordSetting` row exists at most once per
    `SchoolId`.
18. A `BackgroundSetting` may be image-type or color-type.

## Aggregate Roots

| Aggregate                | Root Type                | Purpose                                     |
| ------------------------ | ------------------------ | ------------------------------------------- |
| GeneralSettings          | `GeneralSettings`        | The school's primary configuration row      |
| Language                 | `Language`               | A language registered in the school         |
| LanguagePhrase           | `LanguagePhrase`         | A translatable phrase key                   |
| BaseSetup                | `BaseSetup`              | A lookup value in a `BaseGroup`             |
| BaseGroup                | `BaseGroup`              | A grouping for `BaseSetup`                  |
| DateFormat               | `DateFormat`             | A `strftime` pattern                        |
| Style                    | `Style`                  | A color palette / theme profile             |
| BackgroundSetting        | `BackgroundSetting`      | A background image or color preset          |
| DashboardSetting         | `DashboardSetting`       | A dashboard card binding to a role          |
| CustomLink               | `CustomLink`             | The footer/sidebar custom link bundle       |
| ColorTheme               | `ColorTheme`             | A color value within a theme                |
| Theme                    | `Theme`                  | A theme (color mode, background)            |
| Color                    | `Color`                  | A color entry used by `ColorTheme`          |
| BehaviorRecordSetting    | `BehaviorRecordSetting`  | The behavior record feature flag            |
| SetupAdmin               | `SetupAdmin`             | A purpose/complaint/source/reference entry  |

Each aggregate is documented in detail under
`docs/specs/settings/aggregates.md`.

## Cross-Domain Impact

When a `School` is created (in the platform domain), the settings
domain subscribes and seeds:

- The default `GeneralSettings` row (with the school's name, the
  default language, the default currency, the default date format).
- The default `BehaviorRecordSetting` row (all flags off).
- The default `DashboardSetting` rows (one per dashboard section,
  bound to the `SuperAdmin` role).
- The default `Theme` (the engine's bundled `default` theme).

When a module is enabled (in the platform domain), the settings
domain subscribes and inserts the module's dashboard cards into
the `DashboardSetting` table, bound to the `SuperAdmin` role.

## Subscribers

- `rbac` subscribes to `GeneralSettingsUpdated` to refresh the
  school's branding in audit emails.

## Consumers

- Web admin UI (settings panel, theme picker, dashboard
  configurator).
- Mobile apps (read the active language, theme, and date format).
- Public website (theme, color, header menu).
- AI agents (read configuration before rendering reports).

## Anti-Goals

- The settings domain does not render the UI. It owns the data;
  consumers render.
- The settings domain does not implement internationalization
  itself; it stores the phrase catalog and relies on a port-driven
  i18n provider to translate.
- The settings domain does not enforce business policy. It stores
  the values (e.g. currency code); the consuming domain validates
  the policy.
