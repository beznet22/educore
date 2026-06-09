# Settings Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## General Settings

### UpdateGeneralSettings

```rust
pub struct UpdateGeneralSettingsCommand {
    pub tenant: TenantContext,
    pub school_name: Option<SchoolName>,
    pub site_title: Option<SiteTitle>,
    pub address: Option<Address>,
    pub phone: Option<PhoneNumber>,
    pub email: Option<EmailAddress>,
    pub file_size: Option<FileSize>,
    pub currency: Option<CurrencyCode>,
    pub currency_symbol: Option<CurrencySymbol>,
    pub currency_format: Option<CurrencyFormat>,
    pub logo: Option<FileReference>,
    pub favicon: Option<FileReference>,
    pub copyright_text: Option<CopyrightText>,
    pub website_url: Option<WebsiteUrl>,
    pub week_start_id: Option<WeekStartId>,
    pub time_zone_id: Option<TimeZoneId>,
    pub attendance_layout: Option<AttendanceLayout>,
    pub session_id: Option<AcademicYearId>,
    pub language_id: Option<LanguageId>,
    pub date_format_id: Option<DateFormatId>,
    pub email_driver: Option<EmailDriver>,
    pub fcm_key: Option<FcmKey>,
    pub multiple_roll: Option<bool>,
    pub sub_topic_enable: Option<bool>,
    pub direct_fees_assign: Option<bool>,
    pub with_guardian: Option<bool>,
    pub preloader_status: Option<bool>,
    pub preloader_style: Option<PreloaderStyle>,
    pub preloader_type: Option<PreloaderType>,
    pub preloader_image: Option<FileReference>,
    pub due_fees_login: Option<bool>,
    pub active_theme: Option<ActiveTheme>,
    pub queue_connection: Option<QueueConnection>,
    pub is_comment: Option<bool>,
    pub auto_approve: Option<bool>,
    pub blog_search: Option<bool>,
    pub recent_blog: Option<bool>,
    pub result_type: Option<ResultType>,
    pub phone_number_privacy: Option<PhoneNumberPrivacy>,
    pub language_name: Option<LanguageCode>,
    pub session_year: Option<SessionYear>,
    pub module_toggles: ModuleTogglePatch,
    pub behavior_records: Option<bool>,
    pub download_center: Option<bool>,
    pub ai_content: Option<bool>,
    pub whatsapp_support: Option<bool>,
    pub in_app_live_class: Option<bool>,
    pub fees_status: Option<i32>,
    pub lms_checkout: Option<bool>,
}
```

`ModuleTogglePatch` is a struct of `Option<bool>` for every
module-level feature flag.

**Capability:** `Settings.General.Update`
**Effects:** Emits `GeneralSettingsUpdated`.

### SelectActiveTheme

```rust
pub struct SelectActiveThemeCommand {
    pub tenant: TenantContext,
    pub theme_name: ActiveTheme,
}
```

**Capability:** `Settings.Theme.Select`
**Effects:** Emits `ActiveThemeChanged`.

### SelectLanguage / SelectDateFormat / SelectTimeZone / SelectSession

```rust
pub struct SelectLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}
```

**Capabilities:** `Settings.Language.Select`,
`Settings.DateFormat.Select`, `Settings.TimeZone.Select`,
`Settings.Session.Select`.

**Effects:** Emit `LanguageChanged`, `DateFormatChanged`,
`TimeZoneChanged`, `SessionChanged`.

### EnableTwoFactor / DisableTwoFactor

```rust
pub struct EnableTwoFactorCommand {
    pub tenant: TenantContext,
}
```

**Capability:** `Settings.TwoFactor.Toggle`
**Effects:** Sets `two_factor=true` and emits `TwoFactorToggled`.

## Language

### AddLanguage

```rust
pub struct AddLanguageCommand {
    pub tenant: TenantContext,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub native: LanguageNative,
    pub rtl: RtlFlag,
}
```

**Capability:** `Settings.Language.Add`
**Effects:** Emits `LanguageAdded`.

### UpdateLanguage / DeleteLanguage

Standard CRUD on `Language`.

**Capabilities:** `Settings.Language.Update`,
`Settings.Language.Delete`.

### ActivateLanguage / DeactivateLanguage

```rust
pub struct ActivateLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}
```

**Capability:** `Settings.Language.Activate`
**Effects:** Sets `active_status=1` and emits
`LanguageActivated`.

## Language Phrase

### AddLanguagePhrase

```rust
pub struct AddLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub modules: PhraseModule,
    pub default_phrases: DefaultPhrase,
    pub translations: BTreeMap<LocaleCode, Translation>,
}
```

**Capability:** `Settings.LanguagePhrase.Add`
**Effects:** Emits `LanguagePhraseAdded`.

### UpdateLanguagePhrase / DeleteLanguagePhrase

Standard CRUD on `LanguagePhrase`.

**Capabilities:** `Settings.LanguagePhrase.Update`,
`Settings.LanguagePhrase.Delete`.

### TranslateLanguagePhrase

```rust
pub struct TranslateLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub phrase_id: LanguagePhraseId,
    pub locale: LocaleCode,
    pub translation: Translation,
}
```

**Capability:** `Settings.LanguagePhrase.Translate`
**Effects:** Sets the locale column and emits
`LanguagePhraseTranslated`.

## Base Setup

### AddBaseGroup / UpdateBaseGroup / DeleteBaseGroup

Standard CRUD on `BaseGroup`.

**Capabilities:** `Settings.BaseGroup.Add`,
`Settings.BaseGroup.Update`, `Settings.BaseGroup.Delete`.

### AddBaseSetup / UpdateBaseSetup / DeleteBaseSetup

```rust
pub struct AddBaseSetupCommand {
    pub tenant: TenantContext,
    pub base_setup_name: BaseSetupName,
    pub base_group_id: BaseGroupId,
}
```

**Capabilities:** `Settings.BaseSetup.Add`,
`Settings.BaseSetup.Update`, `Settings.BaseSetup.Delete`.

## Date Format

### AddDateFormat / UpdateDateFormat / DeleteDateFormat

```rust
pub struct AddDateFormatCommand {
    pub tenant: TenantContext,
    pub format: DateFormatPattern,
    pub normal_view: DateFormatPreview,
}
```

**Capabilities:** `Settings.DateFormat.Add`,
`Settings.DateFormat.Update`, `Settings.DateFormat.Delete`.

## Style

### CreateStyle / UpdateStyle / ActivateStyle / DeleteStyle

```rust
pub struct CreateStyleCommand {
    pub tenant: TenantContext,
    pub style_name: StyleName,
    pub path_main_style: StylePath,
    pub path_style: StylePath,
    pub primary_color: ColorHex,
    pub primary_color2: ColorHex,
    pub title_color: ColorHex,
    pub text_color: ColorHex,
    pub white: ColorHex,
    pub black: ColorHex,
    pub sidebar_bg: ColorHex,
    pub barchart1: ColorHex,
    pub barchart2: ColorHex,
    pub barcharttextcolor: ColorHex,
    pub barcharttextfamily: FontFamily,
    pub areachartlinecolor1: ColorHex,
    pub areachartlinecolor2: ColorHex,
    pub dashboardbackground: ColorHex,
}
```

**Capabilities:** `Settings.Style.Create`, `Settings.Style.Update`,
`Settings.Style.Activate`, `Settings.Style.Delete`.

`ActivateStyle` is the only command that mutates the `is_active`
flag. It demotes the previously active style.

## Background

### CreateBackgroundSetting / UpdateBackgroundSetting / DeleteBackgroundSetting

Standard CRUD on `BackgroundSetting`.

**Capabilities:** `Settings.Background.Create`,
`Settings.Background.Update`, `Settings.Background.Delete`.

## Dashboard

### CreateDashboardSetting / UpdateDashboardSetting / DeleteDashboardSetting

```rust
pub struct CreateDashboardSettingCommand {
    pub tenant: TenantContext,
    pub dashboard_sec_id: DashboardSectionId,
    pub role_id: RoleId,
}
```

**Capabilities:** `Settings.Dashboard.Create`,
`Settings.Dashboard.Update`, `Settings.Dashboard.Delete`.

## Custom Link

### UpdateCustomLinks

```rust
pub struct UpdateCustomLinksCommand {
    pub tenant: TenantContext,
    pub links: Vec<CustomLinkEntry>, // up to 16
    pub facebook_url: Option<SocialUrl>,
    pub twitter_url: Option<SocialUrl>,
    pub dribble_url: Option<SocialUrl>,
    pub linkedin_url: Option<SocialUrl>,
    pub behance_url: Option<SocialUrl>,
}
```

**Capability:** `Settings.CustomLink.Update`
**Effects:** Emits `CustomLinksUpdated`.

### ResetCustomLinks

```rust
pub struct ResetCustomLinksCommand {
    pub tenant: TenantContext,
}
```

**Capability:** `Settings.CustomLink.Reset`
**Effects:** Clears all link and social URL fields and emits
`CustomLinksReset`.

## Theme

### CreateTheme

```rust
pub struct CreateThemeCommand {
    pub tenant: TenantContext,
    pub title: ThemeTitle,
    pub path_main_style: ThemePath,
    pub path_style: ThemePath,
    pub color_mode: ColorMode,
    pub box_shadow: BoxShadow,
    pub background_type: BackgroundType,
    pub background_color: Option<ColorHex>,
    pub background_image: Option<FileReference>,
}
```

**Capability:** `Settings.Theme.Create`
**Effects:** Emits `ThemeCreated`.

### UpdateTheme / ActivateTheme / DeleteTheme / ReplicateTheme

Standard CRUD on `Theme` plus `ReplicateTheme` (which clones a
theme's `settings_color_theme` bindings into a new theme row).

**Capabilities:** `Settings.Theme.Update`, `Settings.Theme.Activate`,
`Settings.Theme.Delete`, `Settings.Theme.Replicate`.

## Color

### CreateColor / UpdateColor / DeleteColor

```rust
pub struct CreateColorCommand {
    pub tenant: TenantContext,
    pub name: ColorName,
    pub default_value: ColorValue,
    pub lawn_green: LawnGreen,
    pub is_color: IsColor,
    pub status: ColorStatus,
}
```

**Capability:** `Settings.Color.Create` (system-internal)
**Effects:** Emits `ColorCreated`.

### CreateColorTheme / UpdateColorTheme / DeleteColorTheme

```rust
pub struct CreateColorThemeCommand {
    pub tenant: TenantContext,
    pub color_id: ColorId,
    pub theme_id: ThemeId,
    pub value: ColorValue,
}
```

**Capability:** `Settings.ColorTheme.Create`
**Effects:** Emits `ColorThemeCreated`.

## Behavior Record

### UpdateBehaviorRecordSetting

```rust
pub struct UpdateBehaviorRecordSettingCommand {
    pub tenant: TenantContext,
    pub student_comment: Option<BehaviorFlag>,
    pub parent_comment: Option<BehaviorFlag>,
    pub student_view: Option<BehaviorFlag>,
    pub parent_view: Option<BehaviorFlag>,
}
```

**Capability:** `Settings.BehaviorRecord.Update`
**Effects:** Emits `BehaviorRecordSettingUpdated`.

## Setup Admin

### AddSetupAdmin / UpdateSetupAdmin / DeleteSetupAdmin

```rust
pub struct AddSetupAdminCommand {
    pub tenant: TenantContext,
    pub admin_type: SetupAdminType,
    pub name: SetupAdminName,
    pub description: Option<SetupAdminDescription>,
}
```

**Capabilities:** `Settings.SetupAdmin.Add`,
`Settings.SetupAdmin.Update`, `Settings.SetupAdmin.Delete`.
