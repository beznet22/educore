//! # educore-settings typed commands
//!
//! Per `docs/specs/settings/commands.md`. Typed command shapes
//! across 15 aggregates. Every command carries a
//! [`TenantContext`](educore_core::tenant::TenantContext).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::aggregate::{
    NewBackgroundSetting, NewBaseGroup, NewBaseSetup, NewColor, NewColorTheme, NewDashboardSetting,
    NewDateFormat, NewGeneralSettings, NewLanguage, NewLanguagePhrase, NewSetupAdmin, NewStyle,
    NewTheme,
};
use crate::value_objects::{
    AcademicYearRef, ActiveTheme, BackgroundColor, BackgroundImage, BackgroundTitle,
    BackgroundType, BaseGroupId, BaseGroupName, BaseGroupOrder, BaseSetupId, BaseSetupName,
    BehaviorFlag, ColorHex, ColorId, ColorMode, ColorName, ColorStatus, ColorThemeId, ColorValue,
    DashboardSectionId, DashboardSettingId, DateFormatId, DateFormatPattern, DateFormatPreview,
    EmailDriver, FileReference, FontFamily, GeneralSettingsId, IsColor, LanguageCode, LanguageId,
    LanguageName, LanguageNative, LanguagePhraseId, LawnGreen, LinkHref, LinkLabel, LocaleCode,
    ModuleTogglePatch, PhoneNumberPrivacy, PreloaderStyle, PreloaderType, QueueConnection, RtlFlag,
    SetupAdminDescription, SetupAdminId, SetupAdminName, SetupAdminType, SocialUrl, StyleId,
    StyleName, StylePath, ThemeId, ThemePath, ThemeTitle, Translation,
};

// =============================================================================
// === GeneralSettings commands section begin (owner: A) ===
// =============================================================================

/// Patch a `GeneralSettings` row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateGeneralSettingsCommand {
    pub tenant: TenantContext,
    pub settings_id: GeneralSettingsId,
    pub school_name: Option<String>,
    pub site_title: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub file_size: Option<u64>,
    pub currency: Option<String>,
    pub currency_symbol: Option<String>,
    pub currency_format: Option<crate::value_objects::CurrencyFormat>,
    pub logo: Option<FileReference>,
    pub favicon: Option<FileReference>,
    pub copyright_text: Option<String>,
    pub website_url: Option<LinkHref>,
    pub week_start_id: Option<i32>,
    pub time_zone_id: Option<String>,
    pub attendance_layout: Option<i32>,
    pub session_id: Option<AcademicYearRef>,
    pub language_id: Option<LanguageId>,
    pub date_format_id: Option<DateFormatId>,
    pub email_driver: Option<EmailDriver>,
    pub fcm_key: Option<String>,
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
    pub result_type: Option<String>,
    pub phone_number_privacy: Option<PhoneNumberPrivacy>,
    pub language_name: Option<LanguageCode>,
    pub session_year: Option<String>,
    pub module_toggles: Option<ModuleTogglePatch>,
    pub behavior_records: Option<bool>,
    pub download_center: Option<bool>,
    pub ai_content: Option<bool>,
    pub whatsapp_support: Option<bool>,
    pub in_app_live_class: Option<bool>,
    pub fees_status: Option<i32>,
    pub lms_checkout: Option<bool>,
}

impl UpdateGeneralSettingsCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.update";

    /// Converts to a `GeneralSettingsPatch` aggregate input.
    #[must_use]
    pub fn into_patch(self) -> crate::entities::GeneralSettingsPatch {
        crate::entities::GeneralSettingsPatch {
            school_name: self.school_name,
            site_title: self.site_title,
            address: self.address,
            phone: self.phone,
            email: self.email,
            file_size: self.file_size,
            currency: self.currency,
            currency_symbol: self.currency_symbol,
            currency_format: self.currency_format,
            logo: self.logo,
            favicon: self.favicon,
            copyright_text: self.copyright_text,
            website_url: self.website_url,
            week_start_id: self.week_start_id,
            time_zone_id: self.time_zone_id,
            attendance_layout: self.attendance_layout,
            session_id: self.session_id,
            language_id: self.language_id,
            date_format_id: self.date_format_id,
            email_driver: self.email_driver,
            fcm_key: self.fcm_key,
            multiple_roll: self.multiple_roll,
            sub_topic_enable: self.sub_topic_enable,
            direct_fees_assign: self.direct_fees_assign,
            with_guardian: self.with_guardian,
            preloader_status: self.preloader_status,
            preloader_style: self.preloader_style,
            preloader_type: self.preloader_type,
            preloader_image: self.preloader_image,
            due_fees_login: self.due_fees_login,
            active_theme: self.active_theme,
            queue_connection: self.queue_connection,
            is_comment: self.is_comment,
            auto_approve: self.auto_approve,
            blog_search: self.blog_search,
            recent_blog: self.recent_blog,
            result_type: self.result_type,
            phone_number_privacy: self.phone_number_privacy,
            language_name: self.language_name,
            session_year: self.session_year,
            module_toggles: self.module_toggles,
            behavior_records: self.behavior_records,
            download_center: self.download_center,
            ai_content: self.ai_content,
            whatsapp_support: self.whatsapp_support,
            in_app_live_class: self.in_app_live_class,
            fees_status: self.fees_status,
            lms_checkout: self.lms_checkout,
        }
    }
}

/// Constructor for the (per-school singleton) `GeneralSettings` row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeedGeneralSettingsCommand {
    pub tenant: TenantContext,
    pub school_name: String,
    pub site_title: String,
    pub school_code: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub file_size: u64,
    pub currency: String,
    pub currency_symbol: String,
    pub currency_format: crate::value_objects::CurrencyFormat,
    pub system_version: String,
    pub copyright_text: String,
    pub week_start_id: i32,
    pub time_zone_id: String,
    pub attendance_layout: i32,
    pub active_theme: ActiveTheme,
    pub queue_connection: QueueConnection,
}

impl SeedGeneralSettingsCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.seed";

    /// Converts to a `NewGeneralSettings` aggregate input.
    #[must_use]
    pub fn into_new_settings(self, id: GeneralSettingsId) -> NewGeneralSettings {
        let now = Timestamp::now();
        NewGeneralSettings {
            id,
            school_name: self.school_name,
            site_title: self.site_title,
            school_code: self.school_code,
            address: self.address,
            phone: self.phone,
            email: self.email,
            file_size: self.file_size,
            currency: self.currency,
            currency_symbol: self.currency_symbol,
            currency_format: self.currency_format,
            system_version: self.system_version,
            copyright_text: self.copyright_text,
            week_start_id: self.week_start_id,
            time_zone_id: self.time_zone_id,
            attendance_layout: self.attendance_layout,
            active_theme: self.active_theme,
            queue_connection: self.queue_connection,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Select the active theme for the school.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SelectActiveThemeCommand {
    pub tenant: TenantContext,
    pub theme_name: ActiveTheme,
}

impl SelectActiveThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.select_active_theme";
}

/// Select the default language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SelectLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}

impl SelectLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.select_language";
}

/// Select the default date format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SelectDateFormatCommand {
    pub tenant: TenantContext,
    pub date_format_id: DateFormatId,
}

impl SelectDateFormatCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.select_date_format";
}

/// Select the active time zone.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SelectTimeZoneCommand {
    pub tenant: TenantContext,
    pub time_zone_id: String,
}

impl SelectTimeZoneCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.select_time_zone";
}

/// Select the active session (academic year).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SelectSessionCommand {
    pub tenant: TenantContext,
    pub session_id: AcademicYearRef,
}

impl SelectSessionCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.select_session";
}

/// Enable two-factor authentication.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnableTwoFactorCommand {
    pub tenant: TenantContext,
}

impl EnableTwoFactorCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.enable_two_factor";
}

/// Disable two-factor authentication.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DisableTwoFactorCommand {
    pub tenant: TenantContext,
}

impl DisableTwoFactorCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.general_settings.disable_two_factor";
}

// === GeneralSettings commands section end ===

// =============================================================================
// === Language commands section begin (owner: A) ===
// =============================================================================

/// Add a new language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddLanguageCommand {
    pub tenant: TenantContext,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub native: LanguageNative,
    pub rtl: RtlFlag,
}

impl AddLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language.add";

    /// Converts to a `NewLanguage` aggregate input.
    #[must_use]
    pub fn into_new_language(self, id: LanguageId) -> NewLanguage {
        let now = Timestamp::now();
        NewLanguage {
            id,
            code: self.code,
            name: self.name,
            native: self.native,
            rtl: self.rtl,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
    pub name: Option<LanguageName>,
    pub native: Option<LanguageNative>,
    pub rtl: Option<RtlFlag>,
}

impl UpdateLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language.update";
}

/// Soft-delete a language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}

impl DeleteLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language.delete";
}

/// Activate a language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivateLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}

impl ActivateLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language.activate";
}

/// Deactivate a language.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeactivateLanguageCommand {
    pub tenant: TenantContext,
    pub language_id: LanguageId,
}

impl DeactivateLanguageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language.deactivate";
}

// === Language commands section end ===

// =============================================================================
// === LanguagePhrase commands section begin (owner: A) ===
// =============================================================================

/// Add a new language phrase.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub modules: crate::value_objects::PhraseModule,
    pub default_phrases: crate::value_objects::DefaultPhrase,
    pub translations: std::collections::BTreeMap<LocaleCode, Translation>,
}

impl AddLanguagePhraseCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language_phrase.add";

    /// Converts to a `NewLanguagePhrase` aggregate input.
    #[must_use]
    pub fn into_new_phrase(self, id: LanguagePhraseId) -> NewLanguagePhrase {
        let now = Timestamp::now();
        NewLanguagePhrase {
            id,
            modules: self.modules,
            default_phrases: self.default_phrases,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a language phrase.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub phrase_id: LanguagePhraseId,
    pub modules: Option<crate::value_objects::PhraseModule>,
    pub default_phrases: Option<crate::value_objects::DefaultPhrase>,
}

impl UpdateLanguagePhraseCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language_phrase.update";
}

/// Soft-delete a language phrase.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub phrase_id: LanguagePhraseId,
}

impl DeleteLanguagePhraseCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language_phrase.delete";
}

/// Translate a language phrase.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TranslateLanguagePhraseCommand {
    pub tenant: TenantContext,
    pub phrase_id: LanguagePhraseId,
    pub locale: LocaleCode,
    pub translation: Translation,
}

impl TranslateLanguagePhraseCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.language_phrase.translate";
}

// === LanguagePhrase commands section end ===

// =============================================================================
// === BaseGroup commands section begin (owner: A) ===
// =============================================================================

/// Add a base group.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddBaseGroupCommand {
    pub tenant: TenantContext,
    pub name: BaseGroupName,
    pub order: BaseGroupOrder,
}

impl AddBaseGroupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_group.add";

    /// Converts to a `NewBaseGroup` aggregate input.
    #[must_use]
    pub fn into_new_group(self, id: BaseGroupId) -> NewBaseGroup {
        let now = Timestamp::now();
        NewBaseGroup {
            id,
            name: self.name,
            order: self.order,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a base group.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateBaseGroupCommand {
    pub tenant: TenantContext,
    pub group_id: BaseGroupId,
    pub name: Option<BaseGroupName>,
    pub order: Option<BaseGroupOrder>,
}

impl UpdateBaseGroupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_group.update";
}

/// Soft-delete a base group.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteBaseGroupCommand {
    pub tenant: TenantContext,
    pub group_id: BaseGroupId,
}

impl DeleteBaseGroupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_group.delete";
}

// === BaseGroup commands section end ===

// =============================================================================
// === BaseSetup commands section begin (owner: A) ===
// =============================================================================

/// Add a base setup.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddBaseSetupCommand {
    pub tenant: TenantContext,
    pub base_setup_name: BaseSetupName,
    pub base_group_id: BaseGroupId,
}

impl AddBaseSetupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_setup.add";

    /// Converts to a `NewBaseSetup` aggregate input.
    #[must_use]
    pub fn into_new_setup(self, id: BaseSetupId) -> NewBaseSetup {
        let now = Timestamp::now();
        NewBaseSetup {
            id,
            base_setup_name: self.base_setup_name,
            base_group_id: self.base_group_id,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a base setup.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateBaseSetupCommand {
    pub tenant: TenantContext,
    pub setup_id: BaseSetupId,
    pub base_setup_name: Option<BaseSetupName>,
    pub base_group_id: Option<BaseGroupId>,
}

impl UpdateBaseSetupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_setup.update";
}

/// Soft-delete a base setup.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteBaseSetupCommand {
    pub tenant: TenantContext,
    pub setup_id: BaseSetupId,
}

impl DeleteBaseSetupCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.base_setup.delete";
}

// === BaseSetup commands section end ===

// =============================================================================
// === DateFormat commands section begin (owner: A) ===
// =============================================================================

/// Add a date format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddDateFormatCommand {
    pub tenant: TenantContext,
    pub format: DateFormatPattern,
    pub normal_view: DateFormatPreview,
}

impl AddDateFormatCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.date_format.add";

    /// Converts to a `NewDateFormat` aggregate input.
    #[must_use]
    pub fn into_new_format(self, id: DateFormatId) -> NewDateFormat {
        let now = Timestamp::now();
        NewDateFormat {
            id,
            format: self.format,
            normal_view: self.normal_view,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a date format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateDateFormatCommand {
    pub tenant: TenantContext,
    pub format_id: DateFormatId,
    pub format: Option<DateFormatPattern>,
    pub normal_view: Option<DateFormatPreview>,
}

impl UpdateDateFormatCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.date_format.update";
}

/// Soft-delete a date format.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteDateFormatCommand {
    pub tenant: TenantContext,
    pub format_id: DateFormatId,
}

impl DeleteDateFormatCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.date_format.delete";
}

// === DateFormat commands section end ===

// =============================================================================
// === Style commands section begin (owner: A) ===
// =============================================================================

/// Create a style.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub is_default: bool,
}

impl CreateStyleCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.style.create";

    /// Converts to a `NewStyle` aggregate input.
    #[must_use]
    pub fn into_new_style(self, id: StyleId) -> NewStyle {
        let now = Timestamp::now();
        NewStyle {
            id,
            style_name: self.style_name,
            path_main_style: self.path_main_style,
            path_style: self.path_style,
            primary_color: self.primary_color,
            primary_color2: self.primary_color2,
            title_color: self.title_color,
            text_color: self.text_color,
            white: self.white,
            black: self.black,
            sidebar_bg: self.sidebar_bg,
            barchart1: self.barchart1,
            barchart2: self.barchart2,
            barcharttextcolor: self.barcharttextcolor,
            barcharttextfamily: self.barcharttextfamily,
            areachartlinecolor1: self.areachartlinecolor1,
            areachartlinecolor2: self.areachartlinecolor2,
            dashboardbackground: self.dashboardbackground,
            is_default: self.is_default,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a style.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateStyleCommand {
    pub tenant: TenantContext,
    pub style_id: StyleId,
    pub primary_color: Option<ColorHex>,
    pub primary_color2: Option<ColorHex>,
    pub title_color: Option<ColorHex>,
    pub text_color: Option<ColorHex>,
    pub sidebar_bg: Option<ColorHex>,
    pub dashboardbackground: Option<ColorHex>,
}

impl UpdateStyleCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.style.update";
}

/// Activate a style (demotes the previously active style).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivateStyleCommand {
    pub tenant: TenantContext,
    pub style_id: StyleId,
}

impl ActivateStyleCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.style.activate";
}

/// Soft-delete a style.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteStyleCommand {
    pub tenant: TenantContext,
    pub style_id: StyleId,
}

impl DeleteStyleCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.style.delete";
}

// === Style commands section end ===

// =============================================================================
// === BackgroundSetting commands section begin (owner: A) ===
// =============================================================================

/// Create a background setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateBackgroundSettingCommand {
    pub tenant: TenantContext,
    pub title: BackgroundTitle,
    pub background_type: BackgroundType,
    pub image: Option<BackgroundImage>,
    pub color: Option<BackgroundColor>,
    pub is_default: bool,
}

impl CreateBackgroundSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.background_setting.create";

    /// Converts to a `NewBackgroundSetting` aggregate input.
    #[must_use]
    pub fn into_new_background(
        self,
        id: crate::value_objects::BackgroundSettingId,
    ) -> NewBackgroundSetting {
        let now = Timestamp::now();
        NewBackgroundSetting {
            id,
            title: self.title,
            background_type: self.background_type,
            image: self.image,
            color: self.color,
            is_default: self.is_default,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a background setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateBackgroundSettingCommand {
    pub tenant: TenantContext,
    pub background_id: crate::value_objects::BackgroundSettingId,
    pub title: Option<BackgroundTitle>,
    pub image: Option<BackgroundImage>,
    pub color: Option<BackgroundColor>,
}

impl UpdateBackgroundSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.background_setting.update";
}

/// Soft-delete a background setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteBackgroundSettingCommand {
    pub tenant: TenantContext,
    pub background_id: crate::value_objects::BackgroundSettingId,
}

impl DeleteBackgroundSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.background_setting.delete";
}

// === BackgroundSetting commands section end ===

// =============================================================================
// === DashboardSetting commands section begin (owner: A) ===
// =============================================================================

/// Create a dashboard setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateDashboardSettingCommand {
    pub tenant: TenantContext,
    pub dashboard_sec_id: DashboardSectionId,
    pub role_id: educore_rbac::ids::RoleId,
}

impl CreateDashboardSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.dashboard_setting.create";

    /// Converts to a `NewDashboardSetting` aggregate input.
    #[must_use]
    pub fn into_new_dashboard(self, id: DashboardSettingId) -> NewDashboardSetting {
        let now = Timestamp::now();
        NewDashboardSetting {
            id,
            dashboard_sec_id: self.dashboard_sec_id,
            role_id: self.role_id,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a dashboard setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateDashboardSettingCommand {
    pub tenant: TenantContext,
    pub dashboard_setting_id: DashboardSettingId,
    pub dashboard_sec_id: Option<DashboardSectionId>,
    pub role_id: Option<educore_rbac::ids::RoleId>,
}

impl UpdateDashboardSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.dashboard_setting.update";
}

/// Soft-delete a dashboard setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteDashboardSettingCommand {
    pub tenant: TenantContext,
    pub dashboard_setting_id: DashboardSettingId,
}

impl DeleteDashboardSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.dashboard_setting.delete";
}

// === DashboardSetting commands section end ===

// =============================================================================
// === CustomLink commands section begin (owner: A) ===
// =============================================================================

/// Update the custom link bundle.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateCustomLinksCommand {
    pub tenant: TenantContext,
    pub links: Vec<(LinkLabel, LinkHref)>,
    pub facebook_url: Option<SocialUrl>,
    pub twitter_url: Option<SocialUrl>,
    pub dribble_url: Option<SocialUrl>,
    pub linkedin_url: Option<SocialUrl>,
    pub behance_url: Option<SocialUrl>,
}

impl UpdateCustomLinksCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.custom_link.update";

    /// Builds a `CustomLinkSocial` aggregate input.
    #[must_use]
    pub fn into_social(self) -> crate::entities::CustomLinkSocial {
        crate::entities::CustomLinkSocial {
            facebook_url: self.facebook_url,
            twitter_url: self.twitter_url,
            dribble_url: self.dribble_url,
            linkedin_url: self.linkedin_url,
            behance_url: self.behance_url,
        }
    }
}

/// Reset the custom link bundle.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResetCustomLinksCommand {
    pub tenant: TenantContext,
}

impl ResetCustomLinksCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.custom_link.reset";
}

// === CustomLink commands section end ===

// =============================================================================
// === Theme commands section begin (owner: A) ===
// =============================================================================

/// Create a theme.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateThemeCommand {
    pub tenant: TenantContext,
    pub title: ThemeTitle,
    pub path_main_style: ThemePath,
    pub path_style: ThemePath,
    pub color_mode: ColorMode,
    pub box_shadow: crate::value_objects::BoxShadow,
    pub background_type: BackgroundType,
    pub background_color: Option<ColorHex>,
    pub background_image: Option<FileReference>,
    pub is_default: bool,
}

impl CreateThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.theme.create";

    /// Converts to a `NewTheme` aggregate input.
    #[must_use]
    pub fn into_new_theme(self, id: ThemeId) -> NewTheme {
        let now = Timestamp::now();
        NewTheme {
            id,
            title: self.title,
            path_main_style: self.path_main_style,
            path_style: self.path_style,
            color_mode: self.color_mode,
            box_shadow: self.box_shadow,
            background_type: self.background_type,
            background_color: self.background_color,
            background_image: self.background_image,
            is_default: self.is_default,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a theme.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateThemeCommand {
    pub tenant: TenantContext,
    pub theme_id: ThemeId,
    pub title: Option<ThemeTitle>,
    pub color_mode: Option<ColorMode>,
    pub box_shadow: Option<crate::value_objects::BoxShadow>,
    pub background_color: Option<ColorHex>,
    pub background_image: Option<FileReference>,
}

impl UpdateThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.theme.update";
}

/// Activate a theme.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivateThemeCommand {
    pub tenant: TenantContext,
    pub theme_id: ThemeId,
}

impl ActivateThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.theme.activate";
}

/// Soft-delete a theme.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteThemeCommand {
    pub tenant: TenantContext,
    pub theme_id: ThemeId,
}

impl DeleteThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.theme.delete";
}

/// Replicate a theme.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReplicateThemeCommand {
    pub tenant: TenantContext,
    pub source_theme_id: ThemeId,
    pub new_title: ThemeTitle,
}

impl ReplicateThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.theme.replicate";
}

// === Theme commands section end ===

// =============================================================================
// === Color commands section begin (owner: A) ===
// =============================================================================

/// Create a color (system-internal).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateColorCommand {
    pub tenant: TenantContext,
    pub name: ColorName,
    pub default_value: ColorValue,
    pub lawn_green: LawnGreen,
    pub is_color: IsColor,
    pub status: ColorStatus,
}

impl CreateColorCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color.create";

    /// Converts to a `NewColor` aggregate input.
    #[must_use]
    pub fn into_new_color(self, id: ColorId) -> NewColor {
        let now = Timestamp::now();
        NewColor {
            id,
            name: self.name,
            default_value: self.default_value,
            lawn_green: self.lawn_green,
            is_color: self.is_color,
            status: self.status,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a color (system-internal).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateColorCommand {
    pub tenant: TenantContext,
    pub color_id: ColorId,
    pub name: Option<ColorName>,
    pub default_value: Option<ColorValue>,
    pub lawn_green: Option<LawnGreen>,
}

impl UpdateColorCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color.update";
}

/// Soft-delete a color (system-internal).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteColorCommand {
    pub tenant: TenantContext,
    pub color_id: ColorId,
}

impl DeleteColorCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color.delete";
}

// === Color commands section end ===

// =============================================================================
// === ColorTheme commands section begin (owner: A) ===
// =============================================================================

/// Create a color theme binding.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CreateColorThemeCommand {
    pub tenant: TenantContext,
    pub color_id: ColorId,
    pub theme_id: ThemeId,
    pub value: ColorValue,
}

impl CreateColorThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color_theme.create";

    /// Converts to a `NewColorTheme` aggregate input.
    #[must_use]
    pub fn into_new_color_theme(self, id: ColorThemeId) -> NewColorTheme {
        let now = Timestamp::now();
        NewColorTheme {
            id,
            color_id: self.color_id,
            theme_id: self.theme_id,
            value: self.value,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a color theme binding.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateColorThemeCommand {
    pub tenant: TenantContext,
    pub color_theme_id: ColorThemeId,
    pub value: Option<ColorValue>,
}

impl UpdateColorThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color_theme.update";
}

/// Soft-delete a color theme binding.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteColorThemeCommand {
    pub tenant: TenantContext,
    pub color_theme_id: ColorThemeId,
}

impl DeleteColorThemeCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.color_theme.delete";
}

// === ColorTheme commands section end ===

// =============================================================================
// === BehaviorRecordSetting commands section begin (owner: A) ===
// =============================================================================

/// Update the behavior record setting.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateBehaviorRecordSettingCommand {
    pub tenant: TenantContext,
    pub student_comment: Option<BehaviorFlag>,
    pub parent_comment: Option<BehaviorFlag>,
    pub student_view: Option<BehaviorFlag>,
    pub parent_view: Option<BehaviorFlag>,
}

impl UpdateBehaviorRecordSettingCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.behavior_record_setting.update";
}

// === BehaviorRecordSetting commands section end ===

// =============================================================================
// === SetupAdmin commands section begin (owner: A) ===
// =============================================================================

/// Add a setup admin entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AddSetupAdminCommand {
    pub tenant: TenantContext,
    pub admin_type: SetupAdminType,
    pub name: SetupAdminName,
    pub description: Option<SetupAdminDescription>,
}

impl AddSetupAdminCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.setup_admin.add";

    /// Converts to a `NewSetupAdmin` aggregate input.
    #[must_use]
    pub fn into_new_setup_admin(self, id: SetupAdminId) -> NewSetupAdmin {
        let now = Timestamp::now();
        NewSetupAdmin {
            id,
            admin_type: self.admin_type,
            name: self.name,
            description: self.description,
            created_by: self.tenant.actor_id,
            created_at: now,
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update a setup admin entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UpdateSetupAdminCommand {
    pub tenant: TenantContext,
    pub setup_admin_id: SetupAdminId,
    pub admin_type: Option<SetupAdminType>,
    pub name: Option<SetupAdminName>,
    pub description: Option<SetupAdminDescription>,
}

impl UpdateSetupAdminCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.setup_admin.update";
}

/// Soft-delete a setup admin entry.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeleteSetupAdminCommand {
    pub tenant: TenantContext,
    pub setup_admin_id: SetupAdminId,
}

impl DeleteSetupAdminCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = "settings.setup_admin.delete";
}

// === SetupAdmin commands section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::{CorrelationId, Identifier, SchoolId, UserId};
    use educore_core::tenant::UserType;

    #[test]
    fn command_types_have_wire_form() {
        // GeneralSettings commands
        assert_eq!(
            UpdateGeneralSettingsCommand::COMMAND_TYPE,
            "settings.general_settings.update"
        );
        assert_eq!(
            SeedGeneralSettingsCommand::COMMAND_TYPE,
            "settings.general_settings.seed"
        );
        assert_eq!(
            SelectActiveThemeCommand::COMMAND_TYPE,
            "settings.general_settings.select_active_theme"
        );
        assert_eq!(
            SelectLanguageCommand::COMMAND_TYPE,
            "settings.general_settings.select_language"
        );
        assert_eq!(
            SelectDateFormatCommand::COMMAND_TYPE,
            "settings.general_settings.select_date_format"
        );
        assert_eq!(
            SelectTimeZoneCommand::COMMAND_TYPE,
            "settings.general_settings.select_time_zone"
        );
        assert_eq!(
            SelectSessionCommand::COMMAND_TYPE,
            "settings.general_settings.select_session"
        );
        assert_eq!(
            EnableTwoFactorCommand::COMMAND_TYPE,
            "settings.general_settings.enable_two_factor"
        );
        assert_eq!(
            DisableTwoFactorCommand::COMMAND_TYPE,
            "settings.general_settings.disable_two_factor"
        );

        // Language
        assert_eq!(AddLanguageCommand::COMMAND_TYPE, "settings.language.add");
        assert_eq!(
            UpdateLanguageCommand::COMMAND_TYPE,
            "settings.language.update"
        );
        assert_eq!(
            DeleteLanguageCommand::COMMAND_TYPE,
            "settings.language.delete"
        );
        assert_eq!(
            ActivateLanguageCommand::COMMAND_TYPE,
            "settings.language.activate"
        );
        assert_eq!(
            DeactivateLanguageCommand::COMMAND_TYPE,
            "settings.language.deactivate"
        );

        // LanguagePhrase
        assert_eq!(
            AddLanguagePhraseCommand::COMMAND_TYPE,
            "settings.language_phrase.add"
        );
        assert_eq!(
            UpdateLanguagePhraseCommand::COMMAND_TYPE,
            "settings.language_phrase.update"
        );
        assert_eq!(
            DeleteLanguagePhraseCommand::COMMAND_TYPE,
            "settings.language_phrase.delete"
        );
        assert_eq!(
            TranslateLanguagePhraseCommand::COMMAND_TYPE,
            "settings.language_phrase.translate"
        );

        // BaseGroup
        assert_eq!(AddBaseGroupCommand::COMMAND_TYPE, "settings.base_group.add");
        assert_eq!(
            UpdateBaseGroupCommand::COMMAND_TYPE,
            "settings.base_group.update"
        );
        assert_eq!(
            DeleteBaseGroupCommand::COMMAND_TYPE,
            "settings.base_group.delete"
        );

        // BaseSetup
        assert_eq!(AddBaseSetupCommand::COMMAND_TYPE, "settings.base_setup.add");
        assert_eq!(
            UpdateBaseSetupCommand::COMMAND_TYPE,
            "settings.base_setup.update"
        );
        assert_eq!(
            DeleteBaseSetupCommand::COMMAND_TYPE,
            "settings.base_setup.delete"
        );

        // DateFormat
        assert_eq!(
            AddDateFormatCommand::COMMAND_TYPE,
            "settings.date_format.add"
        );
        assert_eq!(
            UpdateDateFormatCommand::COMMAND_TYPE,
            "settings.date_format.update"
        );
        assert_eq!(
            DeleteDateFormatCommand::COMMAND_TYPE,
            "settings.date_format.delete"
        );

        // Style
        assert_eq!(CreateStyleCommand::COMMAND_TYPE, "settings.style.create");
        assert_eq!(UpdateStyleCommand::COMMAND_TYPE, "settings.style.update");
        assert_eq!(
            ActivateStyleCommand::COMMAND_TYPE,
            "settings.style.activate"
        );
        assert_eq!(DeleteStyleCommand::COMMAND_TYPE, "settings.style.delete");

        // BackgroundSetting
        assert_eq!(
            CreateBackgroundSettingCommand::COMMAND_TYPE,
            "settings.background_setting.create"
        );
        assert_eq!(
            UpdateBackgroundSettingCommand::COMMAND_TYPE,
            "settings.background_setting.update"
        );
        assert_eq!(
            DeleteBackgroundSettingCommand::COMMAND_TYPE,
            "settings.background_setting.delete"
        );

        // DashboardSetting
        assert_eq!(
            CreateDashboardSettingCommand::COMMAND_TYPE,
            "settings.dashboard_setting.create"
        );
        assert_eq!(
            UpdateDashboardSettingCommand::COMMAND_TYPE,
            "settings.dashboard_setting.update"
        );
        assert_eq!(
            DeleteDashboardSettingCommand::COMMAND_TYPE,
            "settings.dashboard_setting.delete"
        );

        // CustomLink
        assert_eq!(
            UpdateCustomLinksCommand::COMMAND_TYPE,
            "settings.custom_link.update"
        );
        assert_eq!(
            ResetCustomLinksCommand::COMMAND_TYPE,
            "settings.custom_link.reset"
        );

        // Theme
        assert_eq!(CreateThemeCommand::COMMAND_TYPE, "settings.theme.create");
        assert_eq!(UpdateThemeCommand::COMMAND_TYPE, "settings.theme.update");
        assert_eq!(
            ActivateThemeCommand::COMMAND_TYPE,
            "settings.theme.activate"
        );
        assert_eq!(DeleteThemeCommand::COMMAND_TYPE, "settings.theme.delete");
        assert_eq!(
            ReplicateThemeCommand::COMMAND_TYPE,
            "settings.theme.replicate"
        );

        // Color
        assert_eq!(CreateColorCommand::COMMAND_TYPE, "settings.color.create");
        assert_eq!(UpdateColorCommand::COMMAND_TYPE, "settings.color.update");
        assert_eq!(DeleteColorCommand::COMMAND_TYPE, "settings.color.delete");

        // ColorTheme
        assert_eq!(
            CreateColorThemeCommand::COMMAND_TYPE,
            "settings.color_theme.create"
        );
        assert_eq!(
            UpdateColorThemeCommand::COMMAND_TYPE,
            "settings.color_theme.update"
        );
        assert_eq!(
            DeleteColorThemeCommand::COMMAND_TYPE,
            "settings.color_theme.delete"
        );

        // BehaviorRecordSetting
        assert_eq!(
            UpdateBehaviorRecordSettingCommand::COMMAND_TYPE,
            "settings.behavior_record_setting.update"
        );

        // SetupAdmin
        assert_eq!(
            AddSetupAdminCommand::COMMAND_TYPE,
            "settings.setup_admin.add"
        );
        assert_eq!(
            UpdateSetupAdminCommand::COMMAND_TYPE,
            "settings.setup_admin.update"
        );
        assert_eq!(
            DeleteSetupAdminCommand::COMMAND_TYPE,
            "settings.setup_admin.delete"
        );
    }

    #[test]
    fn add_language_command_into_new_language(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let school = SchoolId::from_uuid(uuid::Uuid::nil());
        let user = UserId::from_uuid(uuid::Uuid::nil());
        let corr = CorrelationId::from_uuid(uuid::Uuid::nil());
        let tenant = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);
        let cmd = AddLanguageCommand {
            tenant,
            code: LanguageCode::new("en")?,
            name: LanguageName::new("English")?,
            native: LanguageNative::new("English")?,
            rtl: RtlFlag::new(false),
        };
        let id = LanguageId::new(school, uuid::Uuid::nil());
        let new = cmd.into_new_language(id);
        assert_eq!(new.code.as_str(), "en");
        Ok(())
    }
}
