//! # educore-settings child entities
//!
//! Per `docs/specs/settings/entities.md`. Embedded values and
//! owned-by-root children for the 15 settings aggregates.

#![allow(missing_docs, dead_code, clippy::all)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::value_objects::{
    AcademicYearRef, BackgroundColor, BackgroundImage, BackgroundTitle, BackgroundType,
    BaseGroupId, BaseSetupId, BehaviorFlag, ColorHex, ColorId, ColorThemeId, ColorValue,
    DashboardSectionId, DateFormatId, DateFormatPattern, DateFormatPreview, EmailDriver,
    FileReference, LanguageId, LanguagePhraseId, ModuleTogglePatch, ThemeId, ThemeTitle,
};

// =============================================================================
// GeneralSettingsPatch section begin (owner: A)
// =============================================================================

/// A typed patch object for [`GeneralSettings`](crate::aggregate::GeneralSettings).
/// Embedded value. The engine rejects patches that leave the
/// aggregate in an invalid state.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettingsPatch {
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
    pub website_url: Option<crate::value_objects::LinkHref>,
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
    pub preloader_style: Option<crate::value_objects::PreloaderStyle>,
    pub preloader_type: Option<crate::value_objects::PreloaderType>,
    pub preloader_image: Option<FileReference>,
    pub due_fees_login: Option<bool>,
    pub active_theme: Option<crate::value_objects::ActiveTheme>,
    pub queue_connection: Option<crate::value_objects::QueueConnection>,
    pub is_comment: Option<bool>,
    pub auto_approve: Option<bool>,
    pub blog_search: Option<bool>,
    pub recent_blog: Option<bool>,
    pub result_type: Option<String>,
    pub phone_number_privacy: Option<crate::value_objects::PhoneNumberPrivacy>,
    pub language_name: Option<crate::value_objects::LanguageCode>,
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

impl GeneralSettingsPatch {
    /// Returns true if no field is set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.school_name.is_none()
            && self.site_title.is_none()
            && self.address.is_none()
            && self.phone.is_none()
            && self.email.is_none()
            && self.file_size.is_none()
            && self.currency.is_none()
            && self.currency_symbol.is_none()
            && self.currency_format.is_none()
            && self.logo.is_none()
            && self.favicon.is_none()
            && self.copyright_text.is_none()
            && self.website_url.is_none()
            && self.week_start_id.is_none()
            && self.time_zone_id.is_none()
            && self.attendance_layout.is_none()
            && self.session_id.is_none()
            && self.language_id.is_none()
            && self.date_format_id.is_none()
            && self.email_driver.is_none()
            && self.fcm_key.is_none()
            && self.multiple_roll.is_none()
            && self.sub_topic_enable.is_none()
            && self.direct_fees_assign.is_none()
            && self.with_guardian.is_none()
            && self.preloader_status.is_none()
            && self.preloader_style.is_none()
            && self.preloader_type.is_none()
            && self.preloader_image.is_none()
            && self.due_fees_login.is_none()
            && self.active_theme.is_none()
            && self.queue_connection.is_none()
            && self.is_comment.is_none()
            && self.auto_approve.is_none()
            && self.blog_search.is_none()
            && self.recent_blog.is_none()
            && self.result_type.is_none()
            && self.phone_number_privacy.is_none()
            && self.language_name.is_none()
            && self.session_year.is_none()
            && self.module_toggles.is_none()
            && self.behavior_records.is_none()
            && self.download_center.is_none()
            && self.ai_content.is_none()
            && self.whatsapp_support.is_none()
            && self.in_app_live_class.is_none()
            && self.fees_status.is_none()
            && self.lms_checkout.is_none()
    }
}

// === GeneralSettingsPatch section end ===

// =============================================================================
// LanguageTranslation section begin (owner: A)
// =============================================================================

/// A typed projection of a single locale's translation. Owned by
/// [`LanguagePhrase`](crate::aggregate::LanguagePhrase).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageTranslation {
    pub school_id: SchoolId,
    pub phrase_id: LanguagePhraseId,
    pub locale: crate::value_objects::LocaleCode,
    pub translation: crate::value_objects::Translation,
    pub translator: Option<Uuid>,
    pub last_updated: Timestamp,
}

impl LanguageTranslation {
    /// Constructs a new `LanguageTranslation`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        locale: crate::value_objects::LocaleCode,
        translation: crate::value_objects::Translation,
    ) -> Self {
        Self {
            school_id: phrase_id.school_id(),
            phrase_id,
            locale,
            translation,
            translator: None,
            last_updated: Timestamp::now(),
        }
    }
}

// === LanguageTranslation section end ===

// =============================================================================
// LanguageTranslationHistory section begin (owner: A)
// =============================================================================

/// A historical snapshot of a translation. The engine keeps the
/// last N revisions per `(phrase, locale)` to support rollback.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageTranslationHistory {
    pub school_id: SchoolId,
    pub history_id: Uuid,
    pub phrase_id: LanguagePhraseId,
    pub locale: crate::value_objects::LocaleCode,
    pub translation: crate::value_objects::Translation,
    pub superseded_at: Timestamp,
    pub superseded_by: Option<Uuid>,
}

impl LanguageTranslationHistory {
    /// Constructs a new `LanguageTranslationHistory`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        locale: crate::value_objects::LocaleCode,
        translation: crate::value_objects::Translation,
    ) -> Self {
        Self {
            school_id: phrase_id.school_id(),
            history_id: Uuid::new_v4(),
            phrase_id,
            locale,
            translation,
            superseded_at: Timestamp::now(),
            superseded_by: None,
        }
    }
}

// === LanguageTranslationHistory section end ===

// =============================================================================
// DateFormatPreview section begin (owner: A)
// =============================================================================

/// Embedded value. The `normal_view` field of a `DateFormat`,
/// optionally expanded into today's date for live preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormatPreviewEntity {
    pub pattern: DateFormatPattern,
    pub preview: DateFormatPreview,
    pub rendered_today: Option<String>,
}

impl DateFormatPreviewEntity {
    /// Constructs a new `DateFormatPreviewEntity`.
    #[must_use]
    pub fn new(pattern: DateFormatPattern, preview: DateFormatPreview) -> Self {
        Self {
            pattern,
            preview,
            rendered_today: None,
        }
    }
}

// === DateFormatPreview section end ===

// =============================================================================
// StyleChartPalette section begin (owner: A)
// =============================================================================

/// A typed projection of a `Style`'s chart palette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleChartPalette {
    pub school_id: SchoolId,
    pub style_id: crate::value_objects::StyleId,
    pub barchart1: ColorHex,
    pub barchart2: ColorHex,
    pub barcharttextcolor: ColorHex,
    pub barcharttextfamily: crate::value_objects::FontFamily,
    pub areachartlinecolor1: ColorHex,
    pub areachartlinecolor2: ColorHex,
}

impl StyleChartPalette {
    /// Constructs a new `StyleChartPalette`.
    #[must_use]
    pub fn new(style_id: crate::value_objects::StyleId) -> Self {
        Self {
            school_id: style_id.school_id(),
            style_id,
            barchart1: ColorHex("#1f77b4".to_owned()),
            barchart2: ColorHex("#ff7f0e".to_owned()),
            barcharttextcolor: ColorHex("#000000".to_owned()),
            barcharttextfamily: crate::value_objects::FontFamily("Inter, sans-serif".to_owned()),
            areachartlinecolor1: ColorHex("#1f77b4".to_owned()),
            areachartlinecolor2: ColorHex("#ff7f0e".to_owned()),
        }
    }
}

// === StyleChartPalette section end ===

// =============================================================================
// ThemeBackground section begin (owner: A)
// =============================================================================

/// A typed projection of a `Theme`'s background.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeBackground {
    pub school_id: SchoolId,
    pub theme_id: ThemeId,
    pub background_type: BackgroundType,
    pub background_color: Option<BackgroundColor>,
    pub background_image: Option<BackgroundImage>,
}

impl ThemeBackground {
    /// Constructs a new `ThemeBackground`.
    #[must_use]
    pub fn new(theme_id: ThemeId, background_type: BackgroundType) -> Self {
        Self {
            school_id: theme_id.school_id(),
            theme_id,
            background_type,
            background_color: None,
            background_image: None,
        }
    }
}

// === ThemeBackground section end ===

// =============================================================================
// ColorSwatch section begin (owner: A)
// =============================================================================

/// A typed projection of a `Color` carrying the hex value and a
/// human-readable name. Global (no `school_id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorSwatch {
    pub id: ColorId,
    pub name: crate::value_objects::ColorName,
    pub default_value: ColorValue,
}

impl ColorSwatch {
    /// Constructs a new `ColorSwatch`.
    #[must_use]
    pub fn new(
        id: ColorId,
        name: crate::value_objects::ColorName,
        default_value: ColorValue,
    ) -> Self {
        Self {
            id,
            name,
            default_value,
        }
    }
}

// === ColorSwatch section end ===

// =============================================================================
// ColorThemeBinding section begin (owner: A)
// =============================================================================

/// A typed projection of a `ColorTheme` row. Global.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorThemeBinding {
    pub id: ColorThemeId,
    pub theme_id: ThemeId,
    pub color_id: ColorId,
    pub value: ColorValue,
}

impl ColorThemeBinding {
    /// Constructs a new `ColorThemeBinding`.
    #[must_use]
    pub fn new(id: ColorThemeId, theme_id: ThemeId, color_id: ColorId, value: ColorValue) -> Self {
        Self {
            id,
            theme_id,
            color_id,
            value,
        }
    }
}

// === ColorThemeBinding section end ===

// =============================================================================
// DashboardCard section begin (owner: A)
// =============================================================================

/// A logical card on the dashboard. Owned by `DashboardSetting`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardCard {
    pub school_id: SchoolId,
    pub dashboard_setting_id: crate::value_objects::DashboardSettingId,
    pub section_id: DashboardSectionId,
    pub title: String,
    pub icon: Option<String>,
    pub route: Option<String>,
    pub order: i32,
}

impl DashboardCard {
    /// Constructs a new `DashboardCard`.
    #[must_use]
    pub fn new(
        dashboard_setting_id: crate::value_objects::DashboardSettingId,
        section_id: DashboardSectionId,
        title: String,
    ) -> Self {
        Self {
            school_id: dashboard_setting_id.school_id(),
            dashboard_setting_id,
            section_id,
            title,
            icon: None,
            route: None,
            order: 0,
        }
    }
}

// === DashboardCard section end ===

// =============================================================================
// CustomLinkEntry section begin (owner: A)
// =============================================================================

/// A single (label, href) pair within a `CustomLink` bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLinkEntry {
    pub school_id: SchoolId,
    pub entry_id: Uuid,
    pub custom_link_id: crate::value_objects::CustomLinkId,
    pub label: crate::value_objects::LinkLabel,
    pub href: crate::value_objects::LinkHref,
    pub order: i32,
}

impl CustomLinkEntry {
    /// Constructs a new `CustomLinkEntry`.
    #[must_use]
    pub fn new(
        custom_link_id: crate::value_objects::CustomLinkId,
        label: crate::value_objects::LinkLabel,
        href: crate::value_objects::LinkHref,
        order: i32,
    ) -> Self {
        Self {
            school_id: custom_link_id.school_id(),
            entry_id: Uuid::new_v4(),
            custom_link_id,
            label,
            href,
            order,
        }
    }
}

// === CustomLinkEntry section end ===

// =============================================================================
// BehaviorRecordFlag section begin (owner: A)
// =============================================================================

/// Embedded value. A typed projection of one of the four flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorRecordFlag {
    pub name: String,
    pub value: BehaviorFlag,
}

impl BehaviorRecordFlag {
    /// Constructs a new `BehaviorRecordFlag`.
    #[must_use]
    pub fn new(name: impl Into<String>, value: BehaviorFlag) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// Returns true if the flag is "on".
    #[must_use]
    pub fn is_on(&self) -> bool {
        self.value.is_on()
    }
}

// === BehaviorRecordFlag section end ===

// =============================================================================
// SetupAdminTranslation section begin (owner: A)
// =============================================================================

/// A per-locale translation of a `SetupAdmin` entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAdminTranslation {
    pub school_id: SchoolId,
    pub setup_admin_id: crate::value_objects::SetupAdminId,
    pub locale: crate::value_objects::LocaleCode,
    pub name: crate::value_objects::SetupAdminName,
    pub description: Option<crate::value_objects::SetupAdminDescription>,
    pub updated_at: Timestamp,
}

impl SetupAdminTranslation {
    /// Constructs a new `SetupAdminTranslation`.
    #[must_use]
    pub fn new(
        setup_admin_id: crate::value_objects::SetupAdminId,
        locale: crate::value_objects::LocaleCode,
        name: crate::value_objects::SetupAdminName,
    ) -> Self {
        Self {
            school_id: setup_admin_id.school_id(),
            setup_admin_id,
            locale,
            name,
            description: None,
            updated_at: Timestamp::now(),
        }
    }
}

// === SetupAdminTranslation section end ===

// =============================================================================
// BaseSetupOrder section begin (owner: A)
// =============================================================================

/// A typed ordering hint for the setups in a group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSetupOrder {
    pub school_id: SchoolId,
    pub base_group_id: BaseGroupId,
    pub setup_id: BaseSetupId,
    pub order: i32,
}

impl BaseSetupOrder {
    /// Constructs a new `BaseSetupOrder`.
    #[must_use]
    pub fn new(base_group_id: BaseGroupId, setup_id: BaseSetupId, order: i32) -> Self {
        Self {
            school_id: base_group_id.school_id(),
            base_group_id,
            setup_id,
            order,
        }
    }
}

// === BaseSetupOrder section end ===

// =============================================================================
// StyleFontFamily section begin (owner: A)
// =============================================================================

/// Embedded value. A typed projection of `barcharttextfamily`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleFontFamily {
    pub style_id: crate::value_objects::StyleId,
    pub value: crate::value_objects::FontFamily,
}

impl StyleFontFamily {
    /// Constructs a new `StyleFontFamily`.
    #[must_use]
    pub fn new(
        style_id: crate::value_objects::StyleId,
        value: crate::value_objects::FontFamily,
    ) -> Self {
        Self { style_id, value }
    }
}

// === StyleFontFamily section end ===

// =============================================================================
// ThemeReplicate section begin (owner: A)
// =============================================================================

/// A replication record. When a user clicks "replicate this theme",
/// the engine creates a new `Theme` with the same color bindings and
/// stores a `ThemeReplicate` row pointing to the source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeReplicate {
    pub school_id: SchoolId,
    pub id: Uuid,
    pub source_theme_id: ThemeId,
    pub new_theme_id: ThemeId,
    pub copied_color_themes: u32,
    pub replicated_at: Timestamp,
}

impl ThemeReplicate {
    /// Constructs a new `ThemeReplicate`.
    #[must_use]
    pub fn new(source_theme_id: ThemeId, new_theme_id: ThemeId, copied: u32) -> Self {
        Self {
            school_id: source_theme_id.school_id(),
            id: Uuid::new_v4(),
            source_theme_id,
            new_theme_id,
            copied_color_themes: copied,
            replicated_at: Timestamp::now(),
        }
    }
}

// === ThemeReplicate section end ===

// =============================================================================
// SettingsAuditEntry section begin (owner: A)
// =============================================================================

/// A per-event audit row recording a settings change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingsAuditEntry {
    pub school_id: SchoolId,
    pub entry_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub changed_fields: Vec<String>,
    pub actor_id: Uuid,
    pub occurred_at: Timestamp,
}

impl SettingsAuditEntry {
    /// Constructs a new `SettingsAuditEntry`.
    #[must_use]
    pub fn new(aggregate_type: impl Into<String>, aggregate_id: Uuid, actor_id: Uuid) -> Self {
        Self {
            school_id: SchoolId::from_uuid(Uuid::nil()),
            entry_id: Uuid::new_v4(),
            aggregate_type: aggregate_type.into(),
            aggregate_id,
            changed_fields: Vec::new(),
            actor_id,
            occurred_at: Timestamp::now(),
        }
    }

    /// Adds a changed-field name.
    #[must_use]
    pub fn with_field(mut self, name: impl Into<String>) -> Self {
        self.changed_fields.push(name.into());
        self
    }
}

// === SettingsAuditEntry section end ===

// =============================================================================
// LanguageActivationSnapshot section begin (owner: A)
// =============================================================================

/// A point-in-time record of which language is active.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageActivationSnapshot {
    pub school_id: SchoolId,
    pub snapshot_id: Uuid,
    pub language_id: LanguageId,
    pub activated_at: Timestamp,
    pub activated_by: Uuid,
}

impl LanguageActivationSnapshot {
    /// Constructs a new `LanguageActivationSnapshot`.
    #[must_use]
    pub fn new(language_id: LanguageId, activated_by: Uuid) -> Self {
        Self {
            school_id: language_id.school_id(),
            snapshot_id: Uuid::new_v4(),
            language_id,
            activated_at: Timestamp::now(),
            activated_by,
        }
    }
}

// === LanguageActivationSnapshot section end ===

// =============================================================================
// DashboardCardPermission section begin (owner: A)
// =============================================================================

/// A secondary binding that limits a dashboard card to specific
/// capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardCardPermission {
    pub school_id: SchoolId,
    pub id: Uuid,
    pub dashboard_setting_id: crate::value_objects::DashboardSettingId,
    pub capabilities: Vec<String>,
}

impl DashboardCardPermission {
    /// Constructs a new `DashboardCardPermission`.
    #[must_use]
    pub fn new(
        dashboard_setting_id: crate::value_objects::DashboardSettingId,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            school_id: dashboard_setting_id.school_id(),
            id: Uuid::new_v4(),
            dashboard_setting_id,
            capabilities,
        }
    }
}

// === DashboardCardPermission section end ===

// =============================================================================
// CustomLinkSocial section begin (owner: A)
// =============================================================================

/// Embedded value. The five social URLs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLinkSocial {
    pub facebook_url: Option<crate::value_objects::SocialUrl>,
    pub twitter_url: Option<crate::value_objects::SocialUrl>,
    pub dribble_url: Option<crate::value_objects::SocialUrl>,
    pub linkedin_url: Option<crate::value_objects::SocialUrl>,
    pub behance_url: Option<crate::value_objects::SocialUrl>,
}

impl CustomLinkSocial {
    /// Returns the count of set URLs.
    #[must_use]
    pub fn count_set(&self) -> u32 {
        let mut count = 0u32;
        if self.facebook_url.is_some() {
            count += 1;
        }
        if self.twitter_url.is_some() {
            count += 1;
        }
        if self.dribble_url.is_some() {
            count += 1;
        }
        if self.linkedin_url.is_some() {
            count += 1;
        }
        if self.behance_url.is_some() {
            count += 1;
        }
        count
    }

    /// Clears all five social URLs.
    pub fn clear(&mut self) {
        self.facebook_url = None;
        self.twitter_url = None;
        self.dribble_url = None;
        self.linkedin_url = None;
        self.behance_url = None;
    }
}

impl Default for CustomLinkSocial {
    fn default() -> Self {
        Self {
            facebook_url: None,
            twitter_url: None,
            dribble_url: None,
            linkedin_url: None,
            behance_url: None,
        }
    }
}

// === CustomLinkSocial section end ===

// =============================================================================
// PreloaderConfig section begin (owner: A)
// =============================================================================

/// A typed projection of the preloader settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreloaderConfig {
    pub status: bool,
    pub style: crate::value_objects::PreloaderStyle,
    pub preloader_type: crate::value_objects::PreloaderType,
    pub image: Option<FileReference>,
}

impl Default for PreloaderConfig {
    fn default() -> Self {
        Self {
            status: false,
            style: crate::value_objects::PreloaderStyle::new(0),
            // 1 is a valid PreloaderType by spec; bypass validation.
            preloader_type: crate::value_objects::PreloaderType(1),
            image: None,
        }
    }
}

impl PreloaderConfig {
    /// Returns the underlying integer type.
    #[must_use]
    pub fn type_int(&self) -> i32 {
        self.preloader_type.get()
    }
}

// === PreloaderConfig section end ===

// =============================================================================
// EmailDriverConfig section begin (owner: A)
// =============================================================================

/// A typed projection of the email driver settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailDriverConfig {
    pub driver: EmailDriver,
    pub fcm_key: Option<String>,
}

impl EmailDriverConfig {
    /// Constructs a new `EmailDriverConfig`.
    #[must_use]
    pub fn new(driver: EmailDriver) -> Self {
        Self {
            driver,
            fcm_key: None,
        }
    }
}

// === EmailDriverConfig section end ===

// =============================================================================
// FcmKey section begin (owner: A)
// =============================================================================

/// Embedded value. The FCM key used by the notification port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FcmKey {
    pub key: String,
    pub project_id: Option<String>,
    pub sender_id: Option<String>,
}

impl FcmKey {
    /// Constructs a new `FcmKey`.
    pub fn new(key: impl Into<String>) -> Result<Self, educore_core::error::DomainError> {
        let key = key.into();
        if key.is_empty() || key.len() > 1024 {
            return Err(educore_core::error::DomainError::Validation(format!(
                "fcm_key must be 1..1024 chars"
            )));
        }
        Ok(Self {
            key,
            project_id: None,
            sender_id: None,
        })
    }
}

// === FcmKey section end ===

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_settings_patch_is_empty_default() {
        let p = GeneralSettingsPatch::default();
        assert!(p.is_empty());
        let mut p2 = GeneralSettingsPatch::default();
        p2.school_name = Some("Test".to_owned());
        assert!(!p2.is_empty());
    }

    #[test]
    fn behavior_record_flag_is_on() {
        let flag = BehaviorRecordFlag::new("student_comment", BehaviorFlag::new(1).unwrap());
        assert!(flag.is_on());
        let flag = BehaviorRecordFlag::new("student_comment", BehaviorFlag::new(0).unwrap());
        assert!(!flag.is_on());
    }

    #[test]
    fn custom_link_social_count() {
        let mut s = CustomLinkSocial::default();
        assert_eq!(s.count_set(), 0);
        s.facebook_url = Some(crate::value_objects::SocialUrl::new("https://fb.com/x").unwrap());
        s.twitter_url =
            Some(crate::value_objects::SocialUrl::new("https://twitter.com/x").unwrap());
        assert_eq!(s.count_set(), 2);
        s.clear();
        assert_eq!(s.count_set(), 0);
    }

    #[test]
    fn settings_audit_entry_field_addition() {
        let entry = SettingsAuditEntry::new("language", Uuid::nil(), Uuid::nil())
            .with_field("name")
            .with_field("rtl");
        assert_eq!(entry.changed_fields.len(), 2);
    }

    #[test]
    fn language_translation_constructor() {
        let phrase = LanguagePhraseId::new(SchoolId::from_uuid(Uuid::nil()), Uuid::nil());
        let locale = crate::value_objects::LocaleCode::new("en").unwrap();
        let translation = crate::value_objects::Translation::new("Hello").unwrap();
        let lt = LanguageTranslation::new(phrase, locale, translation);
        assert_eq!(lt.school_id, phrase.school_id());
        assert_eq!(lt.locale.as_str(), "en");
    }
}
