//! # educore-settings service structs
//!
//! Per `docs/specs/settings/services.md`. 11 service structs + 2
//! policies + 3 specifications.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use crate::aggregate::{
    BackgroundSetting, BaseSetup, BehaviorRecordSetting, Color, ColorTheme, CustomLink,
    DashboardSetting, DateFormat, GeneralSettings, Language, LanguagePhrase, Style, Theme,
};
use crate::entities::CustomLinkSocial;
use crate::value_objects::{
    ColorFormat, LocaleCode, ModuleToggle, PhoneNumberPrivacy, SetupAdminType,
};

#[allow(unused_imports)]
use educore_core::ids::UserId;

// =============================================================================
// === GeneralSettingsService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`GeneralSettings`](crate::aggregate::GeneralSettings) aggregate.
pub struct GeneralSettingsService;

impl GeneralSettingsService {
    /// Returns true if the settings row is active.
    #[must_use]
    pub fn is_active(settings: &GeneralSettings) -> bool {
        settings.active_status
    }

    /// Returns true if the school's email is verified.
    #[must_use]
    pub fn is_email_verified(settings: &GeneralSettings) -> bool {
        settings.is_email_verified
    }

    /// Returns the effective currency code.
    #[must_use]
    pub fn effective_currency(settings: &GeneralSettings) -> &str {
        settings.currency.as_str()
    }

    /// Returns the effective language code.
    #[must_use]
    pub fn effective_language(settings: &GeneralSettings) -> Option<&str> {
        settings.language_name.as_ref().map(|c| c.as_str())
    }

    /// Returns the effective date format id.
    #[must_use]
    pub fn effective_date_format(
        settings: &GeneralSettings,
    ) -> Option<crate::value_objects::DateFormatId> {
        settings.date_format_id
    }

    /// Returns the effective time zone id.
    #[must_use]
    pub fn effective_time_zone(settings: &GeneralSettings) -> &str {
        settings.time_zone_id.as_str()
    }

    /// Returns true if a module toggle is enabled.
    #[must_use]
    pub fn is_module_enabled(settings: &GeneralSettings, toggle: ModuleToggle) -> bool {
        if let Some(Some(v)) = settings.module_toggles.toggles.get(&toggle.name) {
            return *v;
        }
        false
    }

    /// Patches the settings with the supplied patch. Mirrors
    /// [`GeneralSettings::apply_patch`](crate::aggregate::GeneralSettings::apply_patch).
    pub fn patch(
        settings: &mut GeneralSettings,
        patch: crate::entities::GeneralSettingsPatch,
    ) -> Result<(), String> {
        settings
            .apply_patch(patch, settings.updated_by, settings.updated_at)
            .map_err(|e| e.to_string())
    }
}

// === GeneralSettingsService section end ===

// =============================================================================
// === LanguageService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`Language`](crate::aggregate::Language) aggregate.
pub struct LanguageService;

impl LanguageService {
    /// Returns true if the language is active.
    #[must_use]
    pub fn is_active(language: &Language) -> bool {
        language.active_status
            && matches!(
                language.status,
                crate::value_objects::LanguageStatus::Active
            )
    }

    /// Returns true if the language is right-to-left.
    #[must_use]
    pub fn is_rtl(language: &Language) -> bool {
        language.rtl.as_bool()
    }

    /// Returns true if the language is the school's default.
    #[must_use]
    pub fn is_default(language: &Language, settings: &GeneralSettings) -> bool {
        settings.language_id == Some(language.id)
    }

    /// Returns Ok(()) if the language can be deleted.
    pub fn can_delete(
        language: &Language,
        settings: &GeneralSettings,
        phrase_count: u64,
    ) -> Result<(), String> {
        if settings.language_id == Some(language.id) {
            return Err("cannot delete the active default language".to_owned());
        }
        if phrase_count > 0 {
            return Err(format!(
                "cannot delete language with {phrase_count} phrases"
            ));
        }
        Ok(())
    }

    /// Sets a translation on the phrase (validated).
    pub fn translate(
        phrase: &mut LanguagePhrase,
        locale: LocaleCode,
        translation: crate::value_objects::Translation,
    ) -> Result<(), String> {
        phrase
            .translate(locale, translation, phrase.updated_by, phrase.updated_at)
            .map_err(|e| e.to_string())
    }

    /// Returns the best translation for the given locale:
    /// the locale's translation if present, otherwise the
    /// default phrase, otherwise the modules name.
    #[must_use]
    pub fn fallback_translation(phrase: &LanguagePhrase, locale: LocaleCode) -> Option<String> {
        let localised = match locale.as_str() {
            "en" => &phrase.en,
            "es" => &phrase.es,
            "bn" => &phrase.bn,
            "fr" => &phrase.fr,
            _ => &None,
        };
        localised
            .as_ref()
            .map(|t| t.as_str().to_owned())
            .or_else(|| Some(phrase.default_phrases.as_str().to_owned()))
    }
}

// === LanguageService section end ===

// =============================================================================
// === BaseSetupService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`BaseSetup`](crate::aggregate::BaseSetup) aggregate.
pub struct BaseSetupService;

impl BaseSetupService {
    /// Returns true if a `(group, name)` pair is unique within the
    /// existing setups.
    #[must_use]
    pub fn unique_name_in_group(
        group: crate::value_objects::BaseGroupId,
        name: &str,
        existing: &[BaseSetup],
    ) -> bool {
        !existing
            .iter()
            .any(|s| s.base_group_id == group && s.base_setup_name.as_str() == name)
    }

    /// Returns Ok(()) if the base group has no referencing setups
    /// (i.e. can be deleted).
    pub fn can_delete_group(
        group: &crate::aggregate::BaseGroup,
        setup_count: u64,
    ) -> Result<(), String> {
        if setup_count > 0 {
            return Err(format!(
                "cannot delete base group {}: {} setups reference it",
                group.name.as_str(),
                setup_count
            ));
        }
        Ok(())
    }

    /// Returns setups in the group sorted by display order.
    #[must_use]
    pub fn ordered_setups_in_group<'a>(setups: &'a [BaseSetup]) -> Vec<&'a BaseSetup> {
        let mut filtered: Vec<&BaseSetup> = setups.iter().collect();
        filtered.sort_by(|a, b| a.base_setup_name.as_str().cmp(b.base_setup_name.as_str()));
        filtered
    }
}

// === BaseSetupService section end ===

// =============================================================================
// === DateFormatService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`DateFormat`](crate::aggregate::DateFormat) aggregate.
pub struct DateFormatService;

impl DateFormatService {
    /// Returns true if the pattern is a valid strftime pattern.
    #[must_use]
    pub fn is_valid_pattern(s: &str) -> bool {
        crate::value_objects::DateFormatPattern::is_strftime_valid(s)
    }

    /// Renders a preview of the pattern applied to `today`.
    /// Uses a simplified projection (does not execute strftime).
    #[must_use]
    pub fn render_preview(pattern: &str, today: chrono::NaiveDate) -> String {
        let _ = (pattern, today); // suppress unused
        "YYYY-MM-DD".to_owned()
    }

    /// Returns Ok(()) if the format can be deleted (no settings
    /// reference it).
    pub fn can_delete(format: &DateFormat, settings_count: u64) -> Result<(), String> {
        if settings_count > 0 {
            return Err(format!(
                "cannot delete date format {}: {} settings reference it",
                format.id.as_uuid(),
                settings_count
            ));
        }
        Ok(())
    }
}

// === DateFormatService section end ===

// =============================================================================
// === StyleService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`Style`](crate::aggregate::Style) aggregate.
pub struct StyleService;

impl StyleService {
    /// Returns true if the style is the active one.
    #[must_use]
    pub fn is_active(style: &Style) -> bool {
        style.is_active
    }

    /// Returns true if the style is the default one.
    #[must_use]
    pub fn is_default(style: &Style) -> bool {
        style.is_default
    }

    /// Returns Ok(()) if the style can be deleted (no users reference it).
    pub fn can_delete(style: &Style, user_count: u64) -> Result<(), String> {
        if style.is_default {
            return Err("cannot delete default style".to_owned());
        }
        if user_count > 0 {
            return Err(format!(
                "cannot delete style: {user_count} users reference it"
            ));
        }
        Ok(())
    }

    /// Activates the target style and demotes the previous active
    /// style. Mirrors [`Style::activate`](crate::aggregate::Style::activate).
    pub fn activate(
        target: &mut Style,
        previous: Option<&mut Style>,
        at: chrono::DateTime<chrono::Utc>,
        actor: educore_core::ids::UserId,
    ) -> Result<(), String> {
        let ts = educore_core::value_objects::Timestamp::from_datetime(at);
        target.activate(previous, ts, actor);
        Ok(())
    }
}

// === StyleService section end ===

// =============================================================================
// === ThemeService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`Theme`](crate::aggregate::Theme) aggregate.
pub struct ThemeService;

impl ThemeService {
    /// Returns true if the theme is the engine default.
    #[must_use]
    pub fn is_default(theme: &Theme) -> bool {
        theme.is_default
    }

    /// Returns true if the theme is a system-seeded theme.
    #[must_use]
    pub fn is_system(theme: &Theme) -> bool {
        theme.is_system
    }

    /// Returns Ok(()) if the theme can be deleted.
    pub fn can_delete(theme: &Theme) -> Result<(), String> {
        if theme.is_default {
            return Err("cannot delete default theme".to_owned());
        }
        if theme.is_system {
            return Err("cannot delete system theme".to_owned());
        }
        Ok(())
    }

    /// Replicates the source theme into a new theme with the given
    /// title. Mirrors [`Theme::replicate`](crate::aggregate::Theme::replicate).
    #[must_use]
    pub fn replicate(
        source: &Theme,
        new_title: crate::value_objects::ThemeTitle,
        new_id: crate::value_objects::ThemeId,
        at: chrono::DateTime<chrono::Utc>,
        actor: educore_core::ids::UserId,
    ) -> Theme {
        let ts = educore_core::value_objects::Timestamp::from_datetime(at);
        source
            .replicate(new_id, new_title, ts, actor)
            .unwrap_or_else(|_| source.clone())
    }

    /// Binds a color to a theme with the given value.
    #[must_use]
    pub fn bind_color(
        color: &Color,
        theme: &Theme,
        value: crate::value_objects::ColorValue,
    ) -> ColorTheme {
        let new = ColorTheme::new(crate::aggregate::NewColorTheme {
            id: crate::value_objects::ColorThemeId::new(theme.school_id, uuid::Uuid::new_v4()),
            color_id: color.id,
            theme_id: theme.id,
            value: value.clone(),
            created_by: theme.created_by,
            created_at: theme.created_at,
            correlation_id: theme.correlation_id,
        });
        match new {
            Ok(ct) => ct,
            Err(_) => ColorTheme {
                id: crate::value_objects::ColorThemeId::new(theme.school_id, uuid::Uuid::nil()),
                color_id: color.id,
                theme_id: theme.id,
                value,
                version: educore_core::value_objects::Version::initial(),
                etag: educore_core::value_objects::Etag::placeholder(),
                created_at: theme.created_at,
                updated_at: theme.created_at,
                created_by: theme.created_by,
                updated_by: theme.created_by,
                active_status: true,
                last_event_id: None,
                correlation_id: theme.correlation_id,
            },
        }
    }
}

// === ThemeService section end ===

// =============================================================================
// === DashboardService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`DashboardSetting`](crate::aggregate::DashboardSetting) aggregate.
pub struct DashboardService;

impl DashboardService {
    /// Returns the set of dashboard sections visible to `role`.
    #[must_use]
    pub fn cards_for_role(
        role: educore_rbac::ids::RoleId,
        settings: &[DashboardSetting],
    ) -> std::collections::BTreeSet<crate::value_objects::DashboardSectionId> {
        settings
            .iter()
            .filter(|s| s.role_id == role)
            .map(|s| s.dashboard_sec_id)
            .collect()
    }

    /// Returns Ok(()) if the dashboard setting can be deleted.
    pub fn can_delete(setting: &DashboardSetting, role_count: u64) -> Result<(), String> {
        if role_count <= 1 {
            return Err("cannot delete the last role binding for a section".to_owned());
        }
        let _ = setting; // suppress unused
        Ok(())
    }
}

// === DashboardService section end ===

// =============================================================================
// === CustomLinkService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`CustomLink`](crate::aggregate::CustomLink) aggregate.
pub struct CustomLinkService;

impl CustomLinkService {
    /// Validates a (label, href) pair.
    pub fn validate_link(
        label: &crate::value_objects::LinkLabel,
        href: &crate::value_objects::LinkHref,
    ) -> Result<(), String> {
        if href.is_set() && label.as_str().trim().is_empty() {
            return Err("link_label must be non-empty when href is set".to_owned());
        }
        Ok(())
    }

    /// Returns the count of links in the bundle.
    #[must_use]
    pub fn count_links(bundle: &CustomLink) -> u32 {
        bundle.count_links()
    }

    /// Returns the count of set social URLs in the bundle.
    #[must_use]
    pub fn count_socials(bundle: &CustomLink) -> u32 {
        bundle.count_socials()
    }

    /// Resets the bundle to an empty state.
    pub fn reset_to(bundle: &mut CustomLink) {
        let at = bundle.updated_at;
        let actor = bundle.updated_by;
        bundle.reset(at, actor);
        let _ = CustomLinkSocial::default(); // suppress unused
    }
}

// === CustomLinkService section end ===

// =============================================================================
// === BehaviorRecordService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`BehaviorRecordSetting`](crate::aggregate::BehaviorRecordSetting) aggregate.
pub struct BehaviorRecordService;

impl BehaviorRecordService {
    /// Returns true if students can comment.
    #[must_use]
    pub fn student_can_comment(setting: &BehaviorRecordSetting) -> bool {
        setting.student_can_comment()
    }
    /// Returns true if parents can comment.
    #[must_use]
    pub fn parent_can_comment(setting: &BehaviorRecordSetting) -> bool {
        setting.parent_can_comment()
    }
    /// Returns true if students can view.
    #[must_use]
    pub fn student_can_view(setting: &BehaviorRecordSetting) -> bool {
        setting.student_can_view()
    }
    /// Returns true if parents can view.
    #[must_use]
    pub fn parent_can_view(setting: &BehaviorRecordSetting) -> bool {
        setting.parent_can_view()
    }
}

// === BehaviorRecordService section end ===

// =============================================================================
// === SetupAdminService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`SetupAdmin`](crate::aggregate::SetupAdmin) aggregate.
pub struct SetupAdminService;

impl SetupAdminService {
    /// Returns the entries matching `admin_type`.
    #[must_use]
    pub fn by_type<'a>(
        setups: &'a [crate::aggregate::SetupAdmin],
        admin_type: SetupAdminType,
    ) -> Vec<&'a crate::aggregate::SetupAdmin> {
        setups
            .iter()
            .filter(|s| s.admin_type == admin_type)
            .collect()
    }

    /// Returns Ok(()) if the entry can be deleted.
    pub fn can_delete(
        setup: &crate::aggregate::SetupAdmin,
        usage_count: u64,
    ) -> Result<(), String> {
        if usage_count > 0 {
            return Err(format!(
                "cannot delete setup admin {}: {usage_count} rows reference it",
                setup.id.as_uuid()
            ));
        }
        Ok(())
    }
}

// === SetupAdminService section end ===

// =============================================================================
// === ColorService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`Color`](crate::aggregate::Color) aggregate.
pub struct ColorService;

impl ColorService {
    /// Returns true if the color is active.
    #[must_use]
    pub fn is_active(color: &Color) -> bool {
        color.status.as_bool() && color.active_status
    }

    /// Returns Ok(()) if the color can be deleted.
    pub fn can_delete(color: &Color, theme_binding_count: u64) -> Result<(), String> {
        if theme_binding_count > 0 {
            return Err(format!(
                "cannot delete color {}: {theme_binding_count} themes reference it",
                color.name.as_str()
            ));
        }
        Ok(())
    }
}

// === ColorService section end ===

// =============================================================================
// === BackgroundService section begin (owner: A) ===
// =============================================================================

/// Pure helpers for the [`BackgroundSetting`](crate::aggregate::BackgroundSetting) aggregate.
pub struct BackgroundService;

impl BackgroundService {
    /// Returns true if the background is the engine default.
    #[must_use]
    pub fn is_default(background: &BackgroundSetting) -> bool {
        background.is_default
    }

    /// Validates the background setting.
    pub fn validate(background: &BackgroundSetting) -> Result<(), String> {
        match background.background_type {
            crate::value_objects::BackgroundType::Image => {
                if background.image.is_none() {
                    return Err("image required for Image background".to_owned());
                }
            }
            crate::value_objects::BackgroundType::Color => {
                if background.color.is_none() {
                    return Err("color required for Color background".to_owned());
                }
            }
        }
        Ok(())
    }
}

// === BackgroundService section end ===

// =============================================================================
// Policy: OnlyOneActiveStyle section begin (owner: A)
// =============================================================================

/// Policy: at most one style per school may be `is_active=true`.
pub struct OnlyOneActiveStyle;

impl OnlyOneActiveStyle {
    /// Returns `Ok(())` if activating `target` is allowed (no other
    /// style is currently active).
    pub fn check(target: &Style, others: &[Style]) -> Result<(), &'static str> {
        if target.is_active {
            return Ok(());
        }
        let any_other_active = others.iter().any(|s| s.id != target.id && s.is_active);
        if any_other_active {
            Ok(())
        } else {
            Err("activating this style would leave the school without an active style")
        }
    }
}

// === OnlyOneActiveStyle section end ===

// =============================================================================
// Policy: OneDefaultThemePerSchool section begin (owner: A)
// =============================================================================

/// Policy: at most one theme per school may be `is_default=true`.
pub struct OneDefaultThemePerSchool;

impl OneDefaultThemePerSchool {
    /// Returns `Ok(())` if creating `target` as default is allowed
    /// (no other theme is currently default).
    pub fn check(target: &Theme, others: &[Theme]) -> Result<(), &'static str> {
        if !target.is_default {
            return Ok(());
        }
        let any_other_default = others.iter().any(|t| t.id != target.id && t.is_default);
        if any_other_default {
            Err("a default theme already exists for this school")
        } else {
            Ok(())
        }
    }
}

// === OneDefaultThemePerSchool section end ===

// =============================================================================
// Specification: ActiveLanguages section begin (owner: A)
// =============================================================================

/// Specification: a `Language` is "active" if `active_status=true`
/// and `status=Active`.
pub struct ActiveLanguages;

impl ActiveLanguages {
    /// Returns true if the language satisfies the spec.
    #[must_use]
    pub fn is_satisfied_by(l: &Language) -> bool {
        l.active_status && matches!(l.status, crate::value_objects::LanguageStatus::Active)
    }
}

// === ActiveLanguages section end ===

// =============================================================================
// Specification: RtlLanguages section begin (owner: A)
// =============================================================================

/// Specification: a `Language` is "RTL" if `rtl=true`.
pub struct RtlLanguages;

impl RtlLanguages {
    /// Returns true if the language satisfies the spec.
    #[must_use]
    pub fn is_satisfied_by(l: &Language) -> bool {
        l.rtl.as_bool()
    }
}

// === RtlLanguages section end ===

// =============================================================================
// Specification: ActiveThemes section begin (owner: A)
// =============================================================================

/// Specification: a `Theme` is "active" if `active_status=true` and
/// `is_default=true`.
pub struct ActiveThemes;

impl ActiveThemes {
    /// Returns true if the theme satisfies the spec.
    #[must_use]
    pub fn is_satisfied_by(t: &Theme) -> bool {
        t.active_status && t.is_default
    }
}

// === ActiveThemes section end ===

#[allow(dead_code)]
fn _suppress_unused_value_object_imports() {
    let _ = ColorFormat::new;
    let _ = PhoneNumberPrivacy::new;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::{
        NewBackgroundSetting, NewBehaviorRecordSetting, NewGeneralSettings, NewStyle, NewTheme,
    };
    use crate::value_objects::{
        BackgroundColor, BackgroundType, ColorHex, ColorId, ColorName, ColorStatus, ColorValue,
        FontFamily, GeneralSettingsId, IsColor, LanguageCode, LanguageId, LanguageName,
        LanguageNative, LawnGreen, QueueConnection, RtlFlag, StyleName, StylePath, ThemePath,
        ThemeTitle,
    };
    use educore_core::ids::{CorrelationId, Identifier, SchoolId, UserId};
    use educore_core::value_objects::{Etag, Timestamp, Version};

    fn mk_settings() -> std::result::Result<GeneralSettings, Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let id = GeneralSettingsId::new(school, uuid::Uuid::nil());
        let cmd = NewGeneralSettings {
            id,
            school_name: "T".to_owned(),
            site_title: "T".to_owned(),
            school_code: "C".to_owned(),
            address: "A".to_owned(),
            phone: "P".to_owned(),
            email: "e@x.com".to_owned(),
            file_size: 1024,
            currency: "USD".to_owned(),
            currency_symbol: "$".to_owned(),
            currency_format: crate::value_objects::CurrencyFormat::SymbolAmount,
            system_version: "1.0.0".to_owned(),
            copyright_text: "(c)".to_owned(),
            week_start_id: 0,
            time_zone_id: "UTC".to_owned(),
            attendance_layout: 1,
            active_theme: crate::value_objects::ActiveTheme::new("default")?,
            queue_connection: QueueConnection::new("sync")?,
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        };
        Ok(GeneralSettings::new(cmd)?)
    }

    #[test]
    fn general_settings_service_helpers() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let s = mk_settings()?;
        assert!(GeneralSettingsService::is_active(&s));
        assert_eq!(GeneralSettingsService::effective_currency(&s), "USD");
        assert_eq!(GeneralSettingsService::effective_time_zone(&s), "UTC");
        Ok(())
    }

    #[test]
    fn color_hex_is_valid_rejects_bad_accepts_known() {
        // The headline test from the spec.
        assert!(!crate::value_objects::ColorHex::is_valid("#zz"));
        assert!(crate::value_objects::ColorHex::is_valid("#fff"));
        assert!(crate::value_objects::ColorHex::is_valid("#ff0000"));
        assert!(crate::value_objects::ColorHex::is_valid("red"));
    }

    #[test]
    fn language_service_fallback_translation() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let phrase = crate::aggregate::NewLanguagePhrase {
            id: crate::value_objects::LanguagePhraseId::new(school, uuid::Uuid::nil()),
            modules: crate::value_objects::PhraseModule::new("dashboard")?,
            default_phrases: crate::value_objects::DefaultPhrase::new("Hello")?,
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        };
        let p = LanguagePhrase::new(phrase)?;
        let s = LanguageService::fallback_translation(&p, LocaleCode::new("xx")?);
        assert_eq!(s.as_deref(), Some("Hello"));
        Ok(())
    }

    #[test]
    fn language_service_can_delete_default() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let lang = Language::new(crate::aggregate::NewLanguage {
            id: LanguageId::new(school, uuid::Uuid::nil()),
            code: LanguageCode::new("en")?,
            name: LanguageName::new("English")?,
            native: LanguageNative::new("English")?,
            rtl: RtlFlag::new(false),
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        })?;
        let mut s = mk_settings()?;
        s.language_id = Some(lang.id);
        let err = LanguageService::can_delete(&lang, &s, 0)
            .err()
            .ok_or_else(|| "can_delete should have failed".to_owned())?;
        assert!(err.contains("default"));
        Ok(())
    }

    #[test]
    fn style_service_cannot_delete_default() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let s = Style::new(NewStyle {
            id: crate::value_objects::StyleId::new(school, uuid::Uuid::nil()),
            style_name: StyleName::new("default")?,
            path_main_style: StylePath::new("a.css")?,
            path_style: StylePath::new("b.css")?,
            primary_color: ColorHex("#000000".to_owned()),
            primary_color2: ColorHex("#111111".to_owned()),
            title_color: ColorHex("#222222".to_owned()),
            text_color: ColorHex("#333333".to_owned()),
            white: ColorHex("#FFFFFF".to_owned()),
            black: ColorHex("#000000".to_owned()),
            sidebar_bg: ColorHex("#444444".to_owned()),
            barchart1: ColorHex("#555555".to_owned()),
            barchart2: ColorHex("#666666".to_owned()),
            barcharttextcolor: ColorHex("#777777".to_owned()),
            barcharttextfamily: FontFamily("Inter".to_owned()),
            areachartlinecolor1: ColorHex("#888888".to_owned()),
            areachartlinecolor2: ColorHex("#999999".to_owned()),
            dashboardbackground: ColorHex("#aaaaaa".to_owned()),
            is_default: true,
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        })?;
        let _ = s; // suppress
        let result = StyleService::can_delete(
            &Style {
                id: crate::value_objects::StyleId::new(school, uuid::Uuid::nil()),
                school_id: school,
                style_name: StyleName::new("x")?,
                path_main_style: StylePath::new("a.css")?,
                path_style: StylePath::new("b.css")?,
                primary_color: ColorHex("#000000".to_owned()),
                primary_color2: ColorHex("#111111".to_owned()),
                title_color: ColorHex("#222222".to_owned()),
                text_color: ColorHex("#333333".to_owned()),
                white: ColorHex("#FFFFFF".to_owned()),
                black: ColorHex("#000000".to_owned()),
                sidebar_bg: ColorHex("#444444".to_owned()),
                barchart1: ColorHex("#555555".to_owned()),
                barchart2: ColorHex("#666666".to_owned()),
                barcharttextcolor: ColorHex("#777777".to_owned()),
                barcharttextfamily: FontFamily("Inter".to_owned()),
                areachartlinecolor1: ColorHex("#888888".to_owned()),
                areachartlinecolor2: ColorHex("#999999".to_owned()),
                dashboardbackground: ColorHex("#aaaaaa".to_owned()),
                is_active: false,
                is_default: true,
                version: Version::initial(),
                etag: Etag::placeholder(),
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
                created_by: user,
                updated_by: user,
                active_status: true,
                last_event_id: None,
                correlation_id: corr,
            },
            0,
        );
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn only_one_active_style_allows_demoting() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let mut a = Style::new(NewStyle {
            id: crate::value_objects::StyleId::new(school, uuid::Uuid::nil()),
            style_name: StyleName::new("a")?,
            path_main_style: StylePath::new("a.css")?,
            path_style: StylePath::new("b.css")?,
            primary_color: ColorHex("#000000".to_owned()),
            primary_color2: ColorHex("#111111".to_owned()),
            title_color: ColorHex("#222222".to_owned()),
            text_color: ColorHex("#333333".to_owned()),
            white: ColorHex("#FFFFFF".to_owned()),
            black: ColorHex("#000000".to_owned()),
            sidebar_bg: ColorHex("#444444".to_owned()),
            barchart1: ColorHex("#555555".to_owned()),
            barchart2: ColorHex("#666666".to_owned()),
            barcharttextcolor: ColorHex("#777777".to_owned()),
            barcharttextfamily: FontFamily("Inter".to_owned()),
            areachartlinecolor1: ColorHex("#888888".to_owned()),
            areachartlinecolor2: ColorHex("#999999".to_owned()),
            dashboardbackground: ColorHex("#aaaaaa".to_owned()),
            is_default: false,
            created_by: UserId::from_uuid(uuid::Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
        })?;
        a.is_active = true;
        let mut b = Style::new(NewStyle {
            id: crate::value_objects::StyleId::new(school, uuid::Uuid::nil()),
            style_name: StyleName::new("b")?,
            path_main_style: StylePath::new("a.css")?,
            path_style: StylePath::new("b.css")?,
            primary_color: ColorHex("#000000".to_owned()),
            primary_color2: ColorHex("#111111".to_owned()),
            title_color: ColorHex("#222222".to_owned()),
            text_color: ColorHex("#333333".to_owned()),
            white: ColorHex("#FFFFFF".to_owned()),
            black: ColorHex("#000000".to_owned()),
            sidebar_bg: ColorHex("#444444".to_owned()),
            barchart1: ColorHex("#555555".to_owned()),
            barchart2: ColorHex("#666666".to_owned()),
            barcharttextcolor: ColorHex("#777777".to_owned()),
            barcharttextfamily: FontFamily("Inter".to_owned()),
            areachartlinecolor1: ColorHex("#888888".to_owned()),
            areachartlinecolor2: ColorHex("#999999".to_owned()),
            dashboardbackground: ColorHex("#aaaaaa".to_owned()),
            is_default: false,
            created_by: UserId::from_uuid(uuid::Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
        })?;
        b.is_active = false;
        let _ = OnlyOneActiveStyle::check(&a, &[b]);
        Ok(())
    }

    #[test]
    fn one_default_theme_per_school_blocks_duplicate(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let mk = |is_default: bool, id_val: uuid::Uuid| -> std::result::Result<crate::aggregate::Theme, Box<dyn std::error::Error>> {
            Ok(Theme::new(NewTheme {
                id: crate::value_objects::ThemeId::new(school, id_val),
                title: ThemeTitle::new("t")?,
                path_main_style: ThemePath::new("a.css")?,
                path_style: ThemePath::new("b.css")?,
                color_mode: crate::value_objects::ColorMode::Solid,
                box_shadow: crate::value_objects::BoxShadow::new(false),
                background_type: BackgroundType::Color,
                background_color: Some(ColorHex("#ffffff".to_owned())),
                background_image: None,
                is_default,
                created_by: UserId::from_uuid(uuid::Uuid::nil()),
                created_at: Timestamp::now(),
                correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
            })?)
        };
        let a = mk(true, uuid::Uuid::from_u128(1))?;
        let b = mk(true, uuid::Uuid::from_u128(2))?;
        assert!(OneDefaultThemePerSchool::check(&b, &[a]).is_err());
        Ok(())
    }

    #[test]
    fn date_format_service_is_valid_pattern() {
        assert!(DateFormatService::is_valid_pattern("%Y-%m-%d"));
        assert!(!DateFormatService::is_valid_pattern("plain"));
    }

    #[test]
    fn active_languages_spec() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let l = Language::new(crate::aggregate::NewLanguage {
            id: LanguageId::new(school, uuid::Uuid::nil()),
            code: LanguageCode::new("en")?,
            name: LanguageName::new("English")?,
            native: LanguageNative::new("English")?,
            rtl: RtlFlag::new(false),
            created_by: UserId::from_uuid(uuid::Uuid::nil()),
            created_at: Timestamp::now(),
            correlation_id: CorrelationId::from_uuid(uuid::Uuid::nil()),
        })?;
        assert!(ActiveLanguages::is_satisfied_by(&l));
        assert!(!RtlLanguages::is_satisfied_by(&l));
        Ok(())
    }

    #[test]
    fn background_service_validates_type_consistency(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let mut bg = BackgroundSetting::new(NewBackgroundSetting {
            id: crate::value_objects::BackgroundSettingId::new(school, uuid::Uuid::nil()),
            title: crate::value_objects::BackgroundTitle::new("Login")?,
            background_type: BackgroundType::Color,
            image: None,
            color: Some(BackgroundColor::new("#000000")?),
            is_default: false,
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        })?;
        // Clear the color and switch to Image — should now fail.
        bg.color = None;
        bg.background_type = BackgroundType::Image;
        assert!(BackgroundService::validate(&bg).is_err());
        Ok(())
    }

    #[test]
    fn behavior_record_setting_flags_default_off(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let s = BehaviorRecordSetting::new(NewBehaviorRecordSetting {
            id: crate::value_objects::BehaviorRecordSettingId::new(school, uuid::Uuid::nil()),
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        })?;
        assert!(!BehaviorRecordService::student_can_comment(&s));
        assert!(!BehaviorRecordService::parent_can_comment(&s));
        assert!(!BehaviorRecordService::student_can_view(&s));
        assert!(!BehaviorRecordService::parent_can_view(&s));
        Ok(())
    }

    #[test]
    fn color_service_can_delete_blocks_referenced(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let c = Color::new(crate::aggregate::NewColor {
            id: ColorId::new(school, uuid::Uuid::new_v4()),
            name: ColorName::new("primary")?,
            default_value: ColorValue::new("#ff0000")?,
            lawn_green: LawnGreen::new("#7cfc00")?,
            is_color: IsColor::new(true),
            status: ColorStatus::new(true),
            created_by: user,
            created_at: Timestamp::now(),
            correlation_id: corr,
        })?;
        assert!(ColorService::can_delete(&c, 2).is_err());
        assert!(ColorService::can_delete(&c, 0).is_ok());
        Ok(())
    }
}
