# Settings Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## GeneralSettingsRepository

```rust
#[async_trait]
pub trait GeneralSettingsRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<GeneralSettings>>;
    async fn insert(&self, s: &GeneralSettings) -> Result<()>;
    async fn update(&self, s: &GeneralSettings) -> Result<()>;
    async fn patch(&self, school: SchoolId, patch: GeneralSettingsPatch) -> Result<GeneralSettings>;
}
```

## LanguageRepository

```rust
#[async_trait]
pub trait LanguageRepository: Send + Sync {
    async fn get(&self, id: LanguageId) -> Result<Option<Language>>;
    async fn find_by_code(&self, school: SchoolId, code: &LanguageCode) -> Result<Option<Language>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Language>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Language>>;
    async fn list_rtl(&self, school: SchoolId) -> Result<Vec<Language>>;
    async fn insert(&self, l: &Language) -> Result<()>;
    async fn update(&self, l: &Language) -> Result<()>;
    async fn delete(&self, id: LanguageId) -> Result<()>;
    async fn phrase_count(&self, id: LanguageId) -> Result<u64>;
}
```

## LanguagePhraseRepository

```rust
#[async_trait]
pub trait LanguagePhraseRepository: Send + Sync {
    async fn get(&self, id: LanguagePhraseId) -> Result<Option<LanguagePhrase>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<LanguagePhrase>>;
    async fn list_for_module(&self, school: SchoolId, module: &str) -> Result<Vec<LanguagePhrase>>;
    async fn insert(&self, p: &LanguagePhrase) -> Result<()>;
    async fn update(&self, p: &LanguagePhrase) -> Result<()>;
    async fn delete(&self, id: LanguagePhraseId) -> Result<()>;
    async fn translate(&self, id: LanguagePhraseId, locale: LocaleCode, translation: Translation) -> Result<()>;
}
```

## BaseGroupRepository / BaseSetupRepository

Each follows the same pattern: `get`, `list`, `list_for_school`,
`insert`, `update`, `delete`, plus `unique_name_in_group` and
`referencing_setups` / `referencing_groups`.

## DateFormatRepository

```rust
#[async_trait]
pub trait DateFormatRepository: Send + Sync {
    async fn get(&self, id: DateFormatId) -> Result<Option<DateFormat>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<DateFormat>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<DateFormat>>;
    async fn insert(&self, f: &DateFormat) -> Result<()>;
    async fn update(&self, f: &DateFormat) -> Result<()>;
    async fn delete(&self, id: DateFormatId) -> Result<()>;
    async fn referencing_settings(&self, id: DateFormatId) -> Result<u64>;
}
```

## StyleRepository

```rust
#[async_trait]
pub trait StyleRepository: Send + Sync {
    async fn get(&self, id: StyleId) -> Result<Option<Style>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Style>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Style>>;
    async fn insert(&self, s: &Style) -> Result<()>;
    async fn update(&self, s: &Style) -> Result<()>;
    async fn delete(&self, id: StyleId) -> Result<()>;
    async fn user_count(&self, id: StyleId) -> Result<u64>;
}
```

## BackgroundSettingRepository

```rust
#[async_trait]
pub trait BackgroundSettingRepository: Send + Sync {
    async fn get(&self, id: BackgroundSettingId) -> Result<Option<BackgroundSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<BackgroundSetting>>;
    async fn list_default(&self, school: SchoolId) -> Result<Vec<BackgroundSetting>>;
    async fn insert(&self, b: &BackgroundSetting) -> Result<()>;
    async fn update(&self, b: &BackgroundSetting) -> Result<()>;
    async fn delete(&self, id: BackgroundSettingId) -> Result<()>;
}
```

## DashboardSettingRepository

```rust
#[async_trait]
pub trait DashboardSettingRepository: Send + Sync {
    async fn get(&self, id: DashboardSettingId) -> Result<Option<DashboardSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<DashboardSetting>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<DashboardSetting>>;
    async fn insert(&self, d: &DashboardSetting) -> Result<()>;
    async fn update(&self, d: &DashboardSetting) -> Result<()>;
    async fn delete(&self, id: DashboardSettingId) -> Result<()>;
    async fn role_count(&self, dashboard_sec_id: DashboardSectionId) -> Result<u64>;
}
```

## CustomLinkRepository

```rust
#[async_trait]
pub trait CustomLinkRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<CustomLink>>;
    async fn insert(&self, c: &CustomLink) -> Result<()>;
    async fn update(&self, c: &CustomLink) -> Result<()>;
    async fn reset(&self, school: SchoolId) -> Result<()>;
}
```

## ThemeRepository

```rust
#[async_trait]
pub trait ThemeRepository: Send + Sync {
    async fn get(&self, id: ThemeId) -> Result<Option<Theme>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Theme>>;
    async fn list_active(&self, school: SchoolId) -> Result<Vec<Theme>>;
    async fn list_default(&self, school: SchoolId) -> Result<Vec<Theme>>;
    async fn insert(&self, t: &Theme) -> Result<()>;
    async fn update(&self, t: &Theme) -> Result<()>;
    async fn delete(&self, id: ThemeId) -> Result<()>;
}
```

## ColorRepository

```rust
#[async_trait]
pub trait ColorRepository: Send + Sync {
    async fn get(&self, id: ColorId) -> Result<Option<Color>>;
    async fn list(&self) -> Result<Vec<Color>>;
    async fn insert(&self, c: &Color) -> Result<()>;
    async fn update(&self, c: &Color) -> Result<()>;
    async fn delete(&self, id: ColorId) -> Result<()>;
    async fn theme_binding_count(&self, id: ColorId) -> Result<u64>;
}
```

## ColorThemeRepository

```rust
#[async_trait]
pub trait ColorThemeRepository: Send + Sync {
    async fn get(&self, id: ColorThemeId) -> Result<Option<ColorTheme>>;
    async fn list_for_theme(&self, theme: ThemeId) -> Result<Vec<ColorTheme>>;
    async fn list_for_color(&self, color: ColorId) -> Result<Vec<ColorTheme>>;
    async fn insert(&self, ct: &ColorTheme) -> Result<()>;
    async fn update(&self, ct: &ColorTheme) -> Result<()>;
    async fn delete(&self, id: ColorThemeId) -> Result<()>;
    async fn delete_for_theme(&self, theme: ThemeId) -> Result<u64>;
    async fn copy_for_theme(&self, source: ThemeId, target: ThemeId) -> Result<u32>;
}
```

## BehaviorRecordSettingRepository

```rust
#[async_trait]
pub trait BehaviorRecordSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<BehaviorRecordSetting>>;
    async fn insert(&self, s: &BehaviorRecordSetting) -> Result<()>;
    async fn update(&self, s: &BehaviorRecordSetting) -> Result<()>;
}
```

## SetupAdminRepository

```rust
#[async_trait]
pub trait SetupAdminRepository: Send + Sync {
    async fn get(&self, id: SetupAdminId) -> Result<Option<SetupAdmin>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SetupAdmin>>;
    async fn list_by_type(&self, school: SchoolId, admin_type: SetupAdminType) -> Result<Vec<SetupAdmin>>;
    async fn insert(&self, s: &SetupAdmin) -> Result<()>;
    async fn update(&self, s: &SetupAdmin) -> Result<()>;
    async fn delete(&self, id: SetupAdminId) -> Result<()>;
    async fn usage_count(&self, id: SetupAdminId) -> Result<u64>;
}
```

## Indexes (recommended)

```sql
CREATE UNIQUE INDEX ux_general_settings_school_id ON general_settings (school_id);
CREATE INDEX ix_general_settings_session_id ON general_settings (session_id);
CREATE INDEX ix_general_settings_language_id ON general_settings (language_id);
CREATE INDEX ix_general_settings_date_format_id ON general_settings (date_format_id);
CREATE INDEX ix_general_settings_time_zone_id ON general_settings (time_zone_id);
CREATE INDEX ix_general_settings_academic_id ON general_settings (academic_id);
CREATE UNIQUE INDEX ux_languages_school_id_code ON languages (school_id, code);
CREATE INDEX ix_languages_school_id_active ON languages (school_id, active_status);
CREATE INDEX ix_language_phrases_school_id_module ON language_phrases (school_id, modules);
CREATE UNIQUE INDEX ux_base_groups_school_id_name ON base_groups (school_id, name);
CREATE UNIQUE INDEX ux_base_setups_school_id_group_name ON base_setups (school_id, base_group_id, base_setup_name);
CREATE UNIQUE INDEX ux_date_formats_school_id_format ON date_formats (school_id, format);
CREATE UNIQUE INDEX ux_styles_school_id_name ON styles (school_id, style_name);
CREATE INDEX ix_background_settings_school_id_default ON background_settings (school_id, is_default);
CREATE INDEX ix_dashboard_settings_school_id_role ON dashboard_settings (school_id, role_id);
CREATE UNIQUE INDEX ux_dashboard_settings_school_id_sec_role ON dashboard_settings (school_id, dashboard_sec_id, role_id);
CREATE UNIQUE INDEX ux_custom_links_school_id ON custom_links (school_id);
CREATE UNIQUE INDEX ux_themes_school_id_title ON themes (school_id, title);
CREATE INDEX ix_color_themes_theme_id ON color_themes (theme_id);
CREATE INDEX ix_color_themes_color_id ON color_themes (color_id);
CREATE UNIQUE INDEX ux_color_themes_color_theme ON color_themes (color_id, theme_id);
CREATE UNIQUE INDEX ux_behavior_record_settings_school_id ON behavior_record_settings (school_id);
CREATE INDEX ix_setup_admins_school_id_type ON setup_admins (school_id, type);
```

The `school_id` predicate is mandatory for tenant isolation.
