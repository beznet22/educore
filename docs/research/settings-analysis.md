# Settings Domain — Business Analysis

## Purpose

The settings domain owns the school's
configuration: theme, language, currency, time
zone, academic year format, behavior record
settings, base setups, and the per-tenant feature
flags. Every other domain reads from the settings
domain.

This document describes how school configuration
works in real schools, with the edge cases that
real schools hit.

## Key Concepts

- **GeneralSettings** — the school's master
  configuration.
- **Theme** — the school's visual theme (colors,
  fonts, layout).
- **Color** — a color in the theme palette.
- **BackgroundSetting** — a background image or
  color for login / public pages.
- **BaseGroup** — a grouping for `BaseSetup`s.
- **BaseSetup** — a configurable lookup value
  (gender, religion, occupation, etc.).
- **CustomLink** — a custom link on the public
  website footer or header.
- **BehaviorRecordSetting** — the school's
  behavior record policy.
- **Weekend** — the school's weekend days.
- **SystemVersion** — the current engine version
  (consumed from operations).
- **ModuleManager** — the per-school module
  enablement.

## Real-World Scenarios

### General Settings

The school admin configures the school's
general settings:

- **School name** — the display name.
- **School code** — the unique identifier.
- **Address** — the school's address.
- **Contact** — phone, email, website.
- **Logo** — the school logo (file).
- **Favicon** — the browser tab icon.
- **Currency** — the default currency (INR,
  USD, EUR, etc.).
- **Time zone** — the school's time zone.
- **Language** — the default language.
- **Date format** — DD/MM/YYYY or MM/DD/YYYY.
- **Time format** — 12-hour or 24-hour.
- **Academic year start** — the month the
  academic year starts (e.g. April in India,
  September in the US).
- **Weekend days** — Saturday + Sunday (default);
  Friday + Saturday (in some Middle Eastern
  countries).
- **Theme** — the active theme.

The engine's `GeneralSettings` aggregate
captures the school-wide configuration.

### Theme Customization

A school wants a custom theme:

1. The school admin selects the colors for the
   primary, secondary, accent, and other
   elements.
2. The admin uploads a custom logo.
3. The admin selects the font family.
4. The engine updates the `Theme` aggregate.
5. The portal renders with the new theme.

In real schools, theme customization is
**branding**. The school uses its colors,
logo, and font to project its identity.

### Language Configuration

A school serves a multi-lingual community:

1. The school admin configures the supported
   languages (e.g. English, Hindi, Tamil).
2. The admin uploads the translations (or the
   engine's default translations are used).
3. The user selects their preferred language
   at login or in the settings.
4. The portal renders in the selected
   language.

The engine's `Language` registry is global;
the per-school supported languages are a
configuration.

### Currency Configuration

A school operates in multiple currencies
(e.g. an international school):

1. The school admin configures the default
   currency.
2. The admin configures the exchange rates
   (or the engine reads from a provider).
3. The fees and payments are recorded in
   the configured currency.
4. The portal shows the amounts in the
   user's preferred currency.

### Base Setup Configuration

A school needs to configure lookup values:

- **Gender**: Male, Female, Other.
- **Religion**: Hindu, Muslim, Christian,
  Sikh, Buddhist, Jain, Other.
- **Caste**: General, OBC, SC, ST, Other.
- **Blood Group**: A+, A-, B+, B-, O+, O-,
  AB+, AB-.
- **Occupation**: Salaried, Self-Employed,
  Business, Retired, Homemaker, Other.
- **Mother Tongue**: a list of languages.

The engine's `BaseGroup` and `BaseSetup`
aggregates capture these. A school can
customize the values per group.

### Behavior Record Settings

A school tracks student behavior:

- The school admin configures the
  `BehaviorRecordSetting`:
  - "Student comment allowed": yes / no.
  - "Parent comment allowed": yes / no.
  - "Student view allowed": yes / no.
  - "Parent view allowed": yes / no.
- The behavior record capture (in the
  academic or a dedicated domain) honors
  the settings.

In real schools, behavior records are
sensitive. Some schools allow students to
see their records; others do not. The
engine's settings give the school control.

### Custom Links

A school has custom links on the public
website:

- "Privacy Policy" — links to the school's
  privacy policy.
- "Terms of Service" — links to the TOS.
- "Sitemap" — links to the sitemap.

The engine's `CustomLink` aggregate captures
the custom links. The public website renders
them in the footer or header.

### Module Enablement

A school enables or disables modules:

1. The school admin opens the module manager.
2. The admin enables "Library" (default:
   disabled).
3. The engine updates the `ModuleManager`
   aggregate.
4. The platform domain invalidates the
   module cache; the sidebar updates.

In real schools, module enablement is
**subscription-driven**. The school's package
includes a set of modules; the admin cannot
enable modules outside the package.

### System Version

The engine's current version is displayed
in the settings page. The version is
populated by the operations domain's
`SystemVersionBumped` event.

### Backup Settings

A school configures its backup policy:

- **Frequency**: daily, weekly, monthly.
- **Retention**: 30 days for daily, 12
  weeks for weekly, 7 years for monthly.
- **Storage**: S3, GCS, local.
- **Encryption**: enabled / disabled.

The engine's `BackupSetting` aggregate
captures the policy.

### Notification Settings

A school configures its notification policy:

- **Default channel**: in-app, email, SMS,
  push.
- **Per-type overrides**: which event types
  trigger which channels.
- **Per-user overrides**: individual users
  may opt out of certain types.

The engine's `NotificationSetting`
aggregate captures the policy.

### Academic Year Format

A school configures its academic year
format:

- "2025-2026" — academic year spanning
  calendar years.
- "2025" — academic year matching the start
  year.
- "AY 2025" — academic year with a prefix.

The engine's `AcademicYearFormat` is a
configuration value.

### Date and Time Format

A school configures its date and time
format:

- Date: DD/MM/YYYY, MM/DD/YYYY, YYYY-MM-DD.
- Time: 12-hour (AM/PM), 24-hour.
- Time zone: the school's time zone.

The engine reads the format from the
settings and applies it to all timestamps
in the portal.

## Business Rules

1. A `GeneralSettings` exists at most once
   per `SchoolId`.
2. A `Theme` is unique by `(school_id,
   theme_name)`.
3. A `BaseGroup` is unique by `(school_id,
   base_group_name)`.
4. A `BaseSetup` is unique by
   `(base_group_id, base_setup_name)`.
5. A `ModuleManager` exists at most once
   per `(school_id, module_id)`.
6. A `BehaviorRecordSetting` exists at
   most once per `SchoolId`.
7. Settings are **read at command
   dispatch time**. A change to a setting
   takes effect on the next command.
8. Settings are **versioned**. The
   `updated_at` and `updated_by` fields
   capture the change.
9. Settings are **audited**. The audit
   log captures every change.
10. Settings are **per-tenant**. There is
    no global default; each school has
    its own.

## Edge Cases

### Currency Change Mid-Year

A school changes its default currency
mid-year (e.g. country adopts a new
currency). The engine's
`GeneralSettings.currency` is updated.
Existing records retain their original
currency (the audit trail). New records
use the new currency. The portal shows
a "currency changed" notice.

### Theme Incompatibility

A school customizes the theme with colors
that do not meet accessibility standards.
The engine's UI does not enforce
accessibility; the school is responsible.
The audit log captures the change.

### Language with No Translations

A school enables a language (e.g.
Portuguese) but no translations are
available. The portal falls back to the
default language for missing keys. The
school admin is notified to provide
translations.

### Module Disabled Mid-Year

A school disables the Library module
mid-year. The module's data is
soft-archived. The librarian can no
longer access the module. The data is
retained for re-enable or export.

### Base Setup Deleted with Values

A school deletes a "Religion" base setup
(e.g. "Other"). The engine checks: are
there any users with this religion? If
yes, the deletion is rejected. The admin
must reassign the users to a different
religion first.

### Backup Storage Failure

The school's backup storage (S3) is
unavailable. The engine's backup job
fails. The settings aggregate records
the failure. The admin is notified. The
next scheduled backup retries.

### 2FA Lockout from Settings Change

The school admin changes the 2FA expiry
from 300 seconds to 30 seconds. Existing
2FA sessions are not affected; only new
2FA challenges use the new expiry. The
admin can still log in.

### Theme Reset

A school wants to reset the theme to
the engine default. The engine's
`Theme.reset` command restores the
default colors and fonts. The previous
customization is archived for rollback.

### Settings Conflict

A school has a custom date format
(DD/MM/YYYY) and a custom time zone
(UTC+5:30). The settings are independent;
no conflict.

## Notes for SMSengine Implementation

- The **settings** crate depends on
  `smsengine-core` and `smsengine-platform`.
  It is a per-tenant configuration
  registry.
- The settings domain's **read** is
  hot — every command may read a
  setting. The engine caches the active
  settings in memory; the cache is
  invalidated by `SettingsChanged`
  events.
- The settings domain's **write** is
  rare — only school admins change
  settings.
- The settings domain's
  **audit log** captures every change.
  The audit log is the canonical record
  for compliance.
- The settings domain's **per-tenant
  feature flags** are typed values
  (booleans, strings, numbers). The
  engine reads the flag at command
  dispatch time.
- The settings domain's **module
  enablement** is a configuration. The
  engine's module list reflects the
  enabled modules.
- The settings domain's **theme** is a
  consumer concern. The engine stores
  the theme metadata; the consumer's
  frontend renders it.
- The settings domain's **language**
  is a global registry plus a
  per-school supported list. The
  engine's i18n is consumer-driven.
- The settings domain's **backup
  settings** drive the operations
  domain's backup job. The settings
  change is picked up on the next
  scheduled run.
