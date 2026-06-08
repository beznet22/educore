# Settings Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## GeneralSettingsPatch

**Identity:** Embedded value
**Owner:** `GeneralSettings`

A typed patch object used by `UpdateGeneralSettings`. The patch is a
`struct` with `Option<T>` for every mutable field. The engine
rejects patches that leave the aggregate in an invalid state (e.g.
clearing `school_name`).

## LanguageTranslation

**Identity:** `LanguageTranslationId(SchoolId, Uuid)`
**Owner:** `LanguagePhrase`

A typed projection of a single locale's translation. The base
`LanguagePhrase` row stores translations as text columns; the
`LanguageTranslation` entity is the structured view (locale,
text, translator, last_updated).

## LanguageTranslationHistory

**Identity:** `LanguageTranslationHistoryId(SchoolId, Uuid)`
**Owner:** `LanguagePhrase`

A historical snapshot of a translation. The engine keeps the
last N revisions per `(phrase, locale)` to support rollback.

## DateFormatPreview

**Identity:** Embedded value
**Owner:** `DateFormat`

The `normal_view` field of a `DateFormat` is rendered by the
`DateFormatPreview` projection: it shows the format applied to
today's date.

## StyleChartPalette

**Identity:** `StyleChartPaletteId(SchoolId, Uuid)`
**Owner:** `Style`

A typed projection of a `Style`'s chart palette
(`barchart1`, `barchart2`, `barcharttextfamily`).

## ThemeBackground

**Identity:** `ThemeBackgroundId(SchoolId, Uuid)`
**Owner:** `Theme`

A typed projection of a `Theme`'s background (`background_type`,
`background_color`, `background_image`).

## ColorSwatch

**Identity:** `ColorSwatchId(Uuid)` (global)
**Owner:** `Color`

A typed projection of a `Color` carrying the hex value and a
human-readable name.

## ColorThemeBinding

**Identity:** `ColorThemeBindingId(Uuid)` (global)
**Owner:** `ColorTheme`

A typed projection of a `ColorTheme` row carrying the theme id,
the color id, and the value.

## DashboardCard

**Identity:** `DashboardCardId(SchoolId, Uuid)`
**Owner:** `DashboardSetting` (logical)

A logical card on the dashboard. The base `DashboardSetting` row
binds a card to a role; the `DashboardCard` entity carries the
card's title, icon, route, and ordering.

## CustomLinkEntry

**Identity:** `CustomLinkEntryId(SchoolId, Uuid)`
**Owner:** `CustomLink`

A single (label, href) pair within the bundle. The base
`CustomLink` row carries 16 inline pairs; the
`CustomLinkEntry` is the structured view.

## BehaviorRecordFlag

**Identity:** Embedded value
**Owner:** `BehaviorRecordSetting`

A typed projection of one of the four flags. The flag is a small
integer (0 = off, 1 = on, 2 = inherited); the entity is the
typed view.

## SetupAdminTranslation

**Identity:** `SetupAdminTranslationId(SchoolId, Uuid)`
**Owner:** `SetupAdmin`

A per-locale translation of a `SetupAdmin` entry.

## BaseSetupOrder

**Identity:** `BaseSetupOrderId(SchoolId, Uuid)`
**Owner:** `BaseGroup`

A typed ordering hint for the setups in a group (used to render
dropdowns in a consistent order).

## StyleFontFamily

**Identity:** Embedded value
**Owner:** `Style`

A typed projection of `barcharttextfamily` (a CSS font-family
list).

## ThemeReplicate

**Identity:** `ThemeReplicateId(SchoolId, Uuid)`
**Owner:** `Theme`

A replication record. When a user clicks "replicate this theme",
the engine creates a new `Theme` with the same `color_theme`
bindings and stores a `ThemeReplicate` row pointing to the
source.

## SettingsAuditEntry

**Identity:** `SettingsAuditEntryId(SchoolId, Uuid)`
**Owner:** `GeneralSettings`

A per-event audit row recording a settings change. The event log
is the source of truth; this entity is a denormalized
read-model for the settings history screen.

## LanguageActivationSnapshot

**Identity:** `LanguageActivationSnapshotId(SchoolId, Uuid)`
**Owner:** `Language`

A point-in-time record of which language is active. The engine
records an entry on every `LanguageActivated` event.

## DashboardCardPermission

**Identity:** `DashboardCardPermissionId(SchoolId, Uuid)`
**Owner:** `DashboardSetting`

A secondary binding that limits a dashboard card to specific
capabilities. The card is only shown to users that hold all
listed capabilities.

## CustomLinkSocial

**Identity:** Embedded value
**Owner:** `CustomLink`

A typed projection of the five social URLs (`facebook_url`,
`twitter_url`, `dribble_url`, `linkedin_url`, `behance_url`).

## PreloaderConfig

**Identity:** `PreloaderConfigId(SchoolId, Uuid)`
**Owner:** `GeneralSettings`

A typed projection of the preloader settings
(`preloader_status`, `preloader_style`, `preloader_type`,
`preloader_image`).

## EmailDriverConfig

**Identity:** `EmailDriverConfigId(SchoolId, Uuid)`
**Owner:** `GeneralSettings`

A typed projection of the email driver settings
(`email_driver`, `fcm_key`).

## FcmKey

**Identity:** Embedded value
**Owner:** `GeneralSettings`

The FCM key used by the notification port for push delivery. The
base row stores the key as `text`; the entity is the typed
projection (key, project_id, sender_id).
