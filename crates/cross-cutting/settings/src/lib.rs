//! # educore-settings
//!
//! General settings, language catalog, themes, base setups, custom links.
//!
//! This crate is a member of the Educore workspace. The settings
//! domain owns the school's cosmetic and behavioral configuration:
//! general settings, language catalog, language phrases, base setup
//! groupings, custom links, dashboard layout, themes, color settings,
//! date formats, styles, background settings, the behaviour record
//! feature flag, and the per-school admin setup catalog.
//!
//! See `docs/architecture.md`, `docs/specs/settings/`, and
//! `docs/build-plan.md` § "Phase 14" for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod aggregate;
pub mod commands;
pub mod entities;
pub mod errors;
pub mod events;
pub mod query;
pub mod repository;
pub mod services;
pub mod value_objects;

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-settings";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude: the public surface of the settings crate.
#[allow(missing_docs)]
pub mod prelude {
    pub use crate::aggregate::{
        BackgroundSetting, BaseGroup, BaseSetup, BehaviorRecordSetting, Color, ColorTheme,
        CustomLink, DashboardSetting, DateFormat, GeneralSettings, Language, LanguagePhrase,
        SetupAdmin, Style, Theme,
    };
    pub use crate::entities::{
        BaseSetupOrder, BehaviorRecordFlag, ColorSwatch, ColorThemeBinding, CustomLinkEntry,
        CustomLinkSocial, DashboardCard, DashboardCardPermission, EmailDriverConfig, FcmKey,
        GeneralSettingsPatch, LanguageActivationSnapshot, LanguageTranslation,
        LanguageTranslationHistory, PreloaderConfig, SettingsAuditEntry, SetupAdminTranslation,
        StyleChartPalette, StyleFontFamily, ThemeBackground, ThemeReplicate,
    };
    pub use crate::errors::{Result, SettingsDomainError};
    pub use crate::value_objects::{
        AcademicYearRef, BackgroundColor, BackgroundImage, BackgroundSettingId, BackgroundTitle,
        BackgroundType, BaseGroupId, BaseGroupName, BaseGroupOrder, BaseSetupId, BaseSetupName,
        BehaviorFlag, BehaviorRecordSettingId, BoxShadow, ColorFormat, ColorHex, ColorId,
        ColorMode, ColorName, ColorStatus, ColorThemeId, ColorValue, CustomLinkId,
        DashboardSectionId, DashboardSettingId, DateFormatActive, DateFormatId, DateFormatPattern,
        DateFormatPreview, DefaultPhrase, EmailDriver, FileReference, FontFamily, IsColor,
        LanguageCode, LanguageId, LanguageName, LanguageNative, LanguagePhraseId, LanguageStatus,
        LanguageUniversal, LawnGreen, LinkHref, LinkLabel, LocaleCode, ModuleToggle,
        PhoneNumberPrivacy, PhraseModule, PreloaderStyle, PreloaderType, QueueConnection, RtlFlag,
        RtlLtl, SetupAdminDescription, SetupAdminId, SetupAdminName, SetupAdminType, SocialUrl,
        StyleId, StyleName, StylePath, ThemeId, ThemePath, ThemeTitle, Translation,
    };
    pub use educore_core::ids::SchoolId;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-settings");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_exports_expected_symbols() {
        let _: Option<crate::aggregate::GeneralSettings> = None;
        let _: Option<crate::aggregate::Language> = None;
        let _: Option<crate::aggregate::LanguagePhrase> = None;
        let _: Option<crate::aggregate::BaseSetup> = None;
        let _: Option<crate::aggregate::BaseGroup> = None;
        let _: Option<crate::aggregate::DateFormat> = None;
        let _: Option<crate::aggregate::Style> = None;
        let _: Option<crate::aggregate::BackgroundSetting> = None;
        let _: Option<crate::aggregate::DashboardSetting> = None;
        let _: Option<crate::aggregate::CustomLink> = None;
        let _: Option<crate::aggregate::ColorTheme> = None;
        let _: Option<crate::aggregate::Theme> = None;
        let _: Option<crate::aggregate::Color> = None;
        let _: Option<crate::aggregate::BehaviorRecordSetting> = None;
        let _: Option<crate::aggregate::SetupAdmin> = None;
    }
}
