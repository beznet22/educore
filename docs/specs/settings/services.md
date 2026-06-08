# Settings Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## GeneralSettingsService

```rust
pub struct GeneralSettingsService;

impl GeneralSettingsService {
    pub fn is_active(settings: &GeneralSettings) -> bool { ... }
    pub fn is_email_verified(settings: &GeneralSettings) -> bool { ... }
    pub fn effective_currency(settings: &GeneralSettings) -> CurrencyCode { ... }
    pub fn effective_language(settings: &GeneralSettings) -> LanguageCode { ... }
    pub fn effective_date_format(settings: &GeneralSettings) -> &DateFormat { ... }
    pub fn effective_time_zone(settings: &GeneralSettings) -> &TimeZone { ... }
    pub fn is_module_enabled(settings: &GeneralSettings, toggle: ModuleToggle) -> bool { ... }
    pub fn patch(settings: &mut GeneralSettings, patch: GeneralSettingsPatch) -> Result<(), ValidationError> { ... }
}
```

## LanguageService

```rust
pub struct LanguageService;

impl LanguageService {
    pub fn is_active(language: &Language) -> bool { ... }
    pub fn is_rtl(language: &Language) -> bool { ... }
    pub fn is_default(language: &Language, settings: &GeneralSettings) -> bool { ... }
    pub fn can_delete(language: &Language, settings: &GeneralSettings, phrase_count: u64) -> Result<(), ConflictError> { ... }
    pub fn translate(phrase: &mut LanguagePhrase, locale: LocaleCode, translation: Translation) -> Result<(), ValidationError> { ... }
    pub fn fallback_translation(phrase: &LanguagePhrase, locale: LocaleCode) -> Option<Translation> { ... }
}
```

`LanguageService::fallback_translation` returns the translation
for `locale` if present, otherwise the `default_phrases`, otherwise
the module name. Used by the i18n port.

## BaseSetupService

```rust
pub struct BaseSetupService;

impl BaseSetupService {
    pub fn unique_name_in_group(group: BaseGroupId, name: &str, existing: &[BaseSetup]) -> bool { ... }
    pub fn can_delete_group(group: &BaseGroup, setup_count: u64) -> Result<(), ConflictError> { ... }
    pub fn ordered_setups_in_group(group: &BaseGroup, setups: &[BaseSetup]) -> Vec<&BaseSetup> { ... }
}
```

## DateFormatService

```rust
pub struct DateFormatService;

impl DateFormatService {
    pub fn is_valid_pattern(s: &str) -> bool { ... }
    pub fn render_preview(pattern: &str, today: NaiveDate) -> String { ... }
    pub fn can_delete(format: &DateFormat, settings_count: u64) -> Result<(), ConflictError> { ... }
}
```

## StyleService

```rust
pub struct StyleService;

impl StyleService {
    pub fn is_active(style: &Style) -> bool { ... }
    pub fn is_default(style: &Style) -> bool { ... }
    pub fn can_delete(style: &Style, user_count: u64) -> Result<(), ConflictError> { ... }
    pub fn activate(target: &mut Style, previous: Option<&mut Style>) -> Result<(), ValidationError> { ... }
}
```

## ThemeService

```rust
pub struct ThemeService;

impl ThemeService {
    pub fn is_default(theme: &Theme) -> bool { ... }
    pub fn is_system(theme: &Theme) -> bool { ... }
    pub fn can_delete(theme: &Theme) -> Result<(), ConflictError> { ... }
    pub fn replicate(source: &Theme, new_title: ThemeTitle, school: SchoolId) -> Result<Theme, ValidationError> { ... }
    pub fn bind_color(theme: &Theme, color: &Color, value: ColorHex) -> ColorTheme { ... }
}
```

## DashboardService

```rust
pub struct DashboardService;

impl DashboardService {
    pub fn cards_for_role(role: RoleId, settings: &[DashboardSetting]) -> BTreeSet<DashboardSectionId> { ... }
    pub fn can_delete(setting: &DashboardSetting, role_count: u64) -> Result<(), ConflictError> { ... }
}
```

## CustomLinkService

```rust
pub struct CustomLinkService;

impl CustomLinkService {
    pub fn validate_link(label: &LinkLabel, href: &LinkHref) -> Result<(), ValidationError> { ... }
    pub fn count_links(bundle: &CustomLink) -> u32 { ... }
    pub fn count_socials(bundle: &CustomLink) -> u32 { ... }
    pub fn reset_to(bundle: &mut CustomLink) { ... }
}
```

## BehaviorRecordService

```rust
pub struct BehaviorRecordService;

impl BehaviorRecordService {
    pub fn student_can_comment(setting: &BehaviorRecordSetting) -> bool { ... }
    pub fn parent_can_comment(setting: &BehaviorRecordSetting) -> bool { ... }
    pub fn student_can_view(setting: &BehaviorRecordSetting) -> bool { ... }
    pub fn parent_can_view(setting: &BehaviorRecordSetting) -> bool { ... }
    pub fn patch(setting: &mut BehaviorRecordSetting, patch: BehaviorRecordPatch) { ... }
}
```

## SetupAdminService

```rust
pub struct SetupAdminService;

impl SetupAdminService {
    pub fn by_type(setups: &[SetupAdmin], admin_type: SetupAdminType) -> Vec<&SetupAdmin> { ... }
    pub fn can_delete(setup: &SetupAdmin, usage_count: u64) -> Result<(), ConflictError> { ... }
}
```

## ColorService

```rust
pub struct ColorService;

impl ColorService {
    pub fn is_active(color: &Color) -> bool { ... }
    pub fn can_delete(color: &Color, theme_binding_count: u64) -> Result<(), ConflictError> { ... }
}
```

## BackgroundService

```rust
pub struct BackgroundService;

impl BackgroundService {
    pub fn is_default(background: &BackgroundSetting) -> bool { ... }
    pub fn validate(background: &BackgroundSetting) -> Result<(), ValidationError> { ... }
}
```

## Policy: OnlyOneActiveStyle

```rust
pub struct OnlyOneActiveStyle;

impl Policy<ActivateStyleCommand> for OnlyOneActiveStyle {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &ActivateStyleCommand) -> Outcome { ... }
}
```

## Policy: OneDefaultThemePerSchool

```rust
pub struct OneDefaultThemePerSchool;

impl Policy<CreateThemeCommand> for OneDefaultThemePerSchool {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &CreateThemeCommand) -> Outcome { ... }
}
```

## Specification: ActiveLanguages

```rust
pub struct ActiveLanguages;

impl Specification<Language> for ActiveLanguages {
    fn is_satisfied_by(&self, l: &Language) -> bool { ... }
}
```

Composed with `And`, `Or`, `Not` for queries.

## Specification: RtlLanguages

```rust
pub struct RtlLanguages;

impl Specification<Language> for RtlLanguages {
    fn is_satisfied_by(&self, l: &Language) -> bool { ... }
}
```

## Specification: ActiveThemes

```rust
pub struct ActiveThemes;

impl Specification<Theme> for ActiveThemes {
    fn is_satisfied_by(&self, t: &Theme) -> bool { ... }
}
```

## Cross-Domain Coordinator

The settings domain publishes events; other domains subscribe. The
settings domain does not call other domains' services directly.
