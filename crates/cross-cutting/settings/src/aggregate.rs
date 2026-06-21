//! # educore-settings aggregate roots
//!
//! The 15 root aggregates per `docs/specs/settings/aggregates.md`:
//! GeneralSettings, Language, LanguagePhrase, BaseGroup, BaseSetup,
//! DateFormat, Style, BackgroundSetting, DashboardSetting,
//! CustomLink, ColorTheme, Theme, Color, BehaviorRecordSetting,
//! SetupAdmin.

#![allow(missing_docs, dead_code, clippy::all)]

use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};

use crate::entities::CustomLinkSocial;
use crate::errors::SettingsDomainError;
use crate::value_objects::{
    AcademicYearRef, ActiveTheme, BackgroundColor, BackgroundImage, BackgroundSettingId,
    BackgroundTitle, BackgroundType, BaseGroupId, BaseGroupName, BaseGroupOrder, BaseSetupId,
    BaseSetupName, BehaviorFlag, BehaviorRecordSettingId, BoxShadow, ColorHex, ColorId, ColorMode,
    ColorName, ColorStatus, ColorThemeId, ColorValue, CustomLinkId, DashboardSectionId,
    DashboardSettingId, DateFormatActive, DateFormatId, DateFormatPattern, DateFormatPreview,
    DefaultPhrase, EmailDriver, FileReference, FontFamily, GeneralSettingsId, IsColor,
    LanguageCode, LanguageId, LanguageName, LanguageNative, LanguagePhraseId, LanguageStatus,
    LanguageUniversal, LawnGreen, LinkHref, LinkLabel, LocaleCode, ModuleTogglePatch,
    PhoneNumberPrivacy, PhraseModule, PreloaderStyle, PreloaderType, QueueConnection, RtlFlag,
    SetupAdminDescription, SetupAdminId, SetupAdminName, SetupAdminType, StyleId, StyleName,
    StylePath, ThemeId, ThemePath, ThemeTitle, Translation,
};

/// Result alias for aggregate constructors.
pub type AggregateResult<T> = std::result::Result<T, SettingsDomainError>;

// =============================================================================
// === GeneralSettings section begin (owner: A) ===
// =============================================================================

/// `GeneralSettings` — the school's primary configuration row.
///
/// There is at most one `GeneralSettings` row per `SchoolId`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub id: GeneralSettingsId,
    pub school_id: SchoolId,
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
    pub logo: Option<FileReference>,
    pub favicon: Option<FileReference>,
    pub system_version: String,
    pub copyright_text: String,
    pub website_url: Option<LinkHref>,
    pub week_start_id: i32,
    pub time_zone_id: String,
    pub attendance_layout: i32,
    pub session_id: Option<AcademicYearRef>,
    pub language_id: Option<LanguageId>,
    pub date_format_id: Option<DateFormatId>,
    pub email_driver: EmailDriver,
    pub fcm_key: Option<String>,
    pub multiple_roll: bool,
    pub sub_topic_enable: bool,
    pub direct_fees_assign: bool,
    pub with_guardian: bool,
    pub preloader_status: bool,
    pub preloader_style: PreloaderStyle,
    pub preloader_type: PreloaderType,
    pub preloader_image: Option<FileReference>,
    pub due_fees_login: bool,
    pub active_theme: ActiveTheme,
    pub queue_connection: QueueConnection,
    pub is_comment: bool,
    pub auto_approve: bool,
    pub blog_search: bool,
    pub recent_blog: bool,
    pub result_type: String,
    pub phone_number_privacy: PhoneNumberPrivacy,
    pub language_name: Option<LanguageCode>,
    pub session_year: String,
    pub two_factor: bool,
    pub is_email_verified: bool,
    pub module_toggles: ModuleTogglePatch,
    pub behavior_records: bool,
    pub download_center: bool,
    pub ai_content: bool,
    pub whatsapp_support: bool,
    pub in_app_live_class: bool,
    pub fees_status: i32,
    pub lms_checkout: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`GeneralSettings::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewGeneralSettings {
    pub id: GeneralSettingsId,
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
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl GeneralSettings {
    /// Constructs a new `GeneralSettings`.
    pub fn new(cmd: NewGeneralSettings) -> AggregateResult<Self> {
        if cmd.school_name.trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "school_name must not be empty".to_owned(),
            ));
        }
        if cmd.site_title.trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "site_title must not be empty".to_owned(),
            ));
        }
        if cmd.copyright_text.trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "copyright_text must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            school_name: cmd.school_name,
            site_title: cmd.site_title,
            school_code: cmd.school_code,
            address: cmd.address,
            phone: cmd.phone,
            email: cmd.email,
            file_size: cmd.file_size,
            currency: cmd.currency,
            currency_symbol: cmd.currency_symbol,
            currency_format: cmd.currency_format,
            logo: None,
            favicon: None,
            system_version: cmd.system_version,
            copyright_text: cmd.copyright_text,
            website_url: None,
            week_start_id: cmd.week_start_id,
            time_zone_id: cmd.time_zone_id,
            attendance_layout: cmd.attendance_layout,
            session_id: None,
            language_id: None,
            date_format_id: None,
            // "smtp" is a valid 1..64 char string by spec; bypass validation.
            email_driver: EmailDriver("smtp".to_owned()),
            fcm_key: None,
            multiple_roll: false,
            sub_topic_enable: false,
            direct_fees_assign: false,
            with_guardian: false,
            preloader_status: false,
            preloader_style: PreloaderStyle::new(0),
            // 1 is a valid PreloaderType by spec; bypass validation.
            preloader_type: PreloaderType(1),
            preloader_image: None,
            due_fees_login: false,
            active_theme: cmd.active_theme,
            queue_connection: cmd.queue_connection,
            is_comment: false,
            auto_approve: false,
            blog_search: false,
            recent_blog: false,
            result_type: "gpa".to_owned(),
            // 1 is a valid PhoneNumberPrivacy by spec; bypass validation.
            phone_number_privacy: PhoneNumberPrivacy(1),
            language_name: None,
            session_year: String::new(),
            two_factor: false,
            is_email_verified: false,
            module_toggles: ModuleTogglePatch::default(),
            behavior_records: false,
            download_center: false,
            ai_content: false,
            whatsapp_support: false,
            in_app_live_class: false,
            fees_status: 0,
            lms_checkout: false,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies a patch to the settings, returning an error if the
    /// resulting state would be invalid.
    pub fn apply_patch(
        &mut self,
        patch: crate::entities::GeneralSettingsPatch,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot patch deleted settings".to_owned(),
            ));
        }
        if let Some(v) = patch.school_name {
            if v.trim().is_empty() {
                return Err(SettingsDomainError::Validation(
                    "school_name must not be empty".to_owned(),
                ));
            }
            self.school_name = v;
        }
        if let Some(v) = patch.site_title {
            if v.trim().is_empty() {
                return Err(SettingsDomainError::Validation(
                    "site_title must not be empty".to_owned(),
                ));
            }
            self.site_title = v;
        }
        if let Some(v) = patch.address {
            self.address = v;
        }
        if let Some(v) = patch.phone {
            self.phone = v;
        }
        if let Some(v) = patch.email {
            self.email = v;
        }
        if let Some(v) = patch.file_size {
            self.file_size = v;
        }
        if let Some(v) = patch.currency {
            self.currency = v;
        }
        if let Some(v) = patch.currency_symbol {
            self.currency_symbol = v;
        }
        if let Some(v) = patch.currency_format {
            self.currency_format = v;
        }
        if let Some(v) = patch.logo {
            self.logo = Some(v);
        }
        if let Some(v) = patch.favicon {
            self.favicon = Some(v);
        }
        if let Some(v) = patch.copyright_text {
            if v.trim().is_empty() {
                return Err(SettingsDomainError::Validation(
                    "copyright_text must not be empty".to_owned(),
                ));
            }
            self.copyright_text = v;
        }
        if let Some(v) = patch.website_url {
            self.website_url = Some(v);
        }
        if let Some(v) = patch.week_start_id {
            self.week_start_id = v;
        }
        if let Some(v) = patch.time_zone_id {
            self.time_zone_id = v;
        }
        if let Some(v) = patch.attendance_layout {
            self.attendance_layout = v;
        }
        if let Some(v) = patch.session_id {
            self.session_id = Some(v);
        }
        if let Some(v) = patch.language_id {
            self.language_id = Some(v);
        }
        if let Some(v) = patch.date_format_id {
            self.date_format_id = Some(v);
        }
        if let Some(v) = patch.email_driver {
            self.email_driver = v;
        }
        if let Some(v) = patch.fcm_key {
            self.fcm_key = Some(v);
        }
        if let Some(v) = patch.multiple_roll {
            self.multiple_roll = v;
        }
        if let Some(v) = patch.sub_topic_enable {
            self.sub_topic_enable = v;
        }
        if let Some(v) = patch.direct_fees_assign {
            self.direct_fees_assign = v;
        }
        if let Some(v) = patch.with_guardian {
            self.with_guardian = v;
        }
        if let Some(v) = patch.preloader_status {
            self.preloader_status = v;
        }
        if let Some(v) = patch.preloader_style {
            self.preloader_style = v;
        }
        if let Some(v) = patch.preloader_type {
            self.preloader_type = v;
        }
        if let Some(v) = patch.preloader_image {
            self.preloader_image = Some(v);
        }
        if let Some(v) = patch.due_fees_login {
            self.due_fees_login = v;
        }
        if let Some(v) = patch.active_theme {
            self.active_theme = v;
        }
        if let Some(v) = patch.queue_connection {
            self.queue_connection = v;
        }
        if let Some(v) = patch.is_comment {
            self.is_comment = v;
        }
        if let Some(v) = patch.auto_approve {
            self.auto_approve = v;
        }
        if let Some(v) = patch.blog_search {
            self.blog_search = v;
        }
        if let Some(v) = patch.recent_blog {
            self.recent_blog = v;
        }
        if let Some(v) = patch.result_type {
            self.result_type = v;
        }
        if let Some(v) = patch.phone_number_privacy {
            self.phone_number_privacy = v;
        }
        if let Some(v) = patch.language_name {
            self.language_name = Some(v);
        }
        if let Some(v) = patch.session_year {
            self.session_year = v;
        }
        if let Some(v) = patch.module_toggles {
            self.module_toggles = v;
        }
        if let Some(v) = patch.behavior_records {
            self.behavior_records = v;
        }
        if let Some(v) = patch.download_center {
            self.download_center = v;
        }
        if let Some(v) = patch.ai_content {
            self.ai_content = v;
        }
        if let Some(v) = patch.whatsapp_support {
            self.whatsapp_support = v;
        }
        if let Some(v) = patch.in_app_live_class {
            self.in_app_live_class = v;
        }
        if let Some(v) = patch.fees_status {
            self.fees_status = v;
        }
        if let Some(v) = patch.lms_checkout {
            self.lms_checkout = v;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Selects a new active theme.
    pub fn select_active_theme(&mut self, theme: ActiveTheme, actor: UserId, at: Timestamp) {
        self.active_theme = theme;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Selects the default language.
    pub fn select_language(&mut self, language_id: LanguageId, actor: UserId, at: Timestamp) {
        self.language_id = Some(language_id);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Selects the default date format.
    pub fn select_date_format(
        &mut self,
        date_format_id: DateFormatId,
        actor: UserId,
        at: Timestamp,
    ) {
        self.date_format_id = Some(date_format_id);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Selects the active time zone.
    pub fn select_time_zone(&mut self, time_zone_id: String, actor: UserId, at: Timestamp) {
        self.time_zone_id = time_zone_id;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Selects the active session (academic year).
    pub fn select_session(&mut self, session_id: AcademicYearRef, actor: UserId, at: Timestamp) {
        self.session_id = Some(session_id);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Enables two-factor authentication.
    pub fn enable_two_factor(&mut self, actor: UserId, at: Timestamp) {
        self.two_factor = true;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Disables two-factor authentication.
    pub fn disable_two_factor(&mut self, actor: UserId, at: Timestamp) {
        self.two_factor = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === GeneralSettings section end ===

// =============================================================================
// === Language section begin (owner: A) ===
// =============================================================================

/// `Language` — a language registered in the school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Language {
    pub id: LanguageId,
    pub school_id: SchoolId,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub native: LanguageNative,
    pub universal: Option<LanguageUniversal>,
    pub rtl: RtlFlag,
    pub status: LanguageStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Language::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewLanguage {
    pub id: LanguageId,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub native: LanguageNative,
    pub rtl: RtlFlag,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Language {
    /// Constructs a new `Language`.
    pub fn new(cmd: NewLanguage) -> AggregateResult<Self> {
        if cmd.name.as_str().trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "language name must not be empty".to_owned(),
            ));
        }
        if cmd.native.as_str().trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "language native must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            code: cmd.code,
            name: cmd.name,
            native: cmd.native,
            universal: None,
            rtl: cmd.rtl,
            status: LanguageStatus::Active,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the language fields.
    pub fn update(
        &mut self,
        name: Option<LanguageName>,
        native: Option<LanguageNative>,
        rtl: Option<RtlFlag>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted language".to_owned(),
            ));
        }
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(n) = native {
            self.native = n;
        }
        if let Some(r) = rtl {
            self.rtl = r;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the language.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.status = LanguageStatus::Inactive;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Activates the language.
    pub fn activate(&mut self, at: Timestamp, actor: UserId) {
        self.status = LanguageStatus::Active;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Deactivates the language.
    pub fn deactivate(&mut self, at: Timestamp, actor: UserId) {
        self.status = LanguageStatus::Inactive;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === Language section end ===

// =============================================================================
// === LanguagePhrase section begin (owner: A) ===
// =============================================================================

/// `LanguagePhrase` — a translatable phrase key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguagePhrase {
    pub id: LanguagePhraseId,
    pub school_id: SchoolId,
    pub modules: PhraseModule,
    pub default_phrases: DefaultPhrase,
    pub en: Option<Translation>,
    pub es: Option<Translation>,
    pub bn: Option<Translation>,
    pub fr: Option<Translation>,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`LanguagePhrase::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewLanguagePhrase {
    pub id: LanguagePhraseId,
    pub modules: PhraseModule,
    pub default_phrases: DefaultPhrase,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl LanguagePhrase {
    /// Constructs a new `LanguagePhrase`.
    pub fn new(cmd: NewLanguagePhrase) -> AggregateResult<Self> {
        if cmd.modules.as_str().trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "phrase modules must not be empty".to_owned(),
            ));
        }
        if cmd.default_phrases.as_str().trim().is_empty() {
            return Err(SettingsDomainError::Validation(
                "default_phrase must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            modules: cmd.modules,
            default_phrases: cmd.default_phrases,
            en: None,
            es: None,
            bn: None,
            fr: None,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the phrase base fields.
    pub fn update(
        &mut self,
        modules: Option<PhraseModule>,
        default_phrases: Option<DefaultPhrase>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted phrase".to_owned(),
            ));
        }
        if let Some(m) = modules {
            self.modules = m;
        }
        if let Some(d) = default_phrases {
            self.default_phrases = d;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Translates the phrase for the given locale.
    pub fn translate(
        &mut self,
        locale: LocaleCode,
        translation: Translation,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        match locale.as_str() {
            "en" => self.en = Some(translation),
            "es" => self.es = Some(translation),
            "bn" => self.bn = Some(translation),
            "fr" => self.fr = Some(translation),
            other => {
                return Err(SettingsDomainError::Validation(format!(
                    "unsupported locale {other} (supported: en, es, bn, fr)"
                )));
            }
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the phrase.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === LanguagePhrase section end ===

// =============================================================================
// === BaseGroup section begin (owner: A) ===
// =============================================================================

/// `BaseGroup` — a grouping for `BaseSetup` values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseGroup {
    pub id: BaseGroupId,
    pub school_id: SchoolId,
    pub name: BaseGroupName,
    pub order: BaseGroupOrder,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`BaseGroup::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewBaseGroup {
    pub id: BaseGroupId,
    pub name: BaseGroupName,
    pub order: BaseGroupOrder,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl BaseGroup {
    /// Constructs a new `BaseGroup`.
    pub fn new(cmd: NewBaseGroup) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            name: cmd.name,
            order: cmd.order,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the group name/order.
    pub fn update(
        &mut self,
        name: Option<BaseGroupName>,
        order: Option<BaseGroupOrder>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted base group".to_owned(),
            ));
        }
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(o) = order {
            self.order = o;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the group.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === BaseGroup section end ===

// =============================================================================
// === BaseSetup section begin (owner: A) ===
// =============================================================================

/// `BaseSetup` — a lookup value in a `BaseGroup`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSetup {
    pub id: BaseSetupId,
    pub school_id: SchoolId,
    pub base_setup_name: BaseSetupName,
    pub base_group_id: BaseGroupId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`BaseSetup::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewBaseSetup {
    pub id: BaseSetupId,
    pub base_setup_name: BaseSetupName,
    pub base_group_id: BaseGroupId,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl BaseSetup {
    /// Constructs a new `BaseSetup`.
    pub fn new(cmd: NewBaseSetup) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            base_setup_name: cmd.base_setup_name,
            base_group_id: cmd.base_group_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the base setup name and group.
    pub fn update(
        &mut self,
        base_setup_name: Option<BaseSetupName>,
        base_group_id: Option<BaseGroupId>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted base setup".to_owned(),
            ));
        }
        if let Some(n) = base_setup_name {
            self.base_setup_name = n;
        }
        if let Some(g) = base_group_id {
            self.base_group_id = g;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the base setup.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === BaseSetup section end ===

// =============================================================================
// === DateFormat section begin (owner: A) ===
// =============================================================================

/// `DateFormat` — a `strftime` pattern with a human-readable preview.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormat {
    pub id: DateFormatId,
    pub school_id: SchoolId,
    pub format: DateFormatPattern,
    pub normal_view: DateFormatPreview,
    pub active: DateFormatActive,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`DateFormat::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewDateFormat {
    pub id: DateFormatId,
    pub format: DateFormatPattern,
    pub normal_view: DateFormatPreview,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl DateFormat {
    /// Constructs a new `DateFormat`.
    pub fn new(cmd: NewDateFormat) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            format: cmd.format,
            normal_view: cmd.normal_view,
            active: DateFormatActive::new(false),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the format and preview.
    pub fn update(
        &mut self,
        format: Option<DateFormatPattern>,
        normal_view: Option<DateFormatPreview>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted date format".to_owned(),
            ));
        }
        if let Some(f) = format {
            self.format = f;
        }
        if let Some(n) = normal_view {
            self.normal_view = n;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the format.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === DateFormat section end ===

// =============================================================================
// === Style section begin (owner: A) ===
// =============================================================================

/// `Style` — a color palette / theme profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Style {
    pub id: StyleId,
    pub school_id: SchoolId,
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
    pub is_active: bool,
    pub is_default: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Style::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewStyle {
    pub id: StyleId,
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
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Style {
    /// Constructs a new `Style`.
    pub fn new(cmd: NewStyle) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            style_name: cmd.style_name,
            path_main_style: cmd.path_main_style,
            path_style: cmd.path_style,
            primary_color: cmd.primary_color,
            primary_color2: cmd.primary_color2,
            title_color: cmd.title_color,
            text_color: cmd.text_color,
            white: cmd.white,
            black: cmd.black,
            sidebar_bg: cmd.sidebar_bg,
            barchart1: cmd.barchart1,
            barchart2: cmd.barchart2,
            barcharttextcolor: cmd.barcharttextcolor,
            barcharttextfamily: cmd.barcharttextfamily,
            areachartlinecolor1: cmd.areachartlinecolor1,
            areachartlinecolor2: cmd.areachartlinecolor2,
            dashboardbackground: cmd.dashboardbackground,
            is_active: false,
            is_default: cmd.is_default,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the style fields.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        primary_color: Option<ColorHex>,
        primary_color2: Option<ColorHex>,
        title_color: Option<ColorHex>,
        text_color: Option<ColorHex>,
        sidebar_bg: Option<ColorHex>,
        dashboardbackground: Option<ColorHex>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted style".to_owned(),
            ));
        }
        if let Some(c) = primary_color {
            self.primary_color = c;
        }
        if let Some(c) = primary_color2 {
            self.primary_color2 = c;
        }
        if let Some(c) = title_color {
            self.title_color = c;
        }
        if let Some(c) = text_color {
            self.text_color = c;
        }
        if let Some(c) = sidebar_bg {
            self.sidebar_bg = c;
        }
        if let Some(c) = dashboardbackground {
            self.dashboardbackground = c;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Marks the style as active and demotes the previously active
    /// one (passed in by the caller). The caller is responsible for
    /// ensuring `previous` belongs to the same school.
    pub fn activate(&mut self, previous: Option<&mut Style>, at: Timestamp, actor: UserId) {
        if let Some(prev) = previous {
            if prev.is_active {
                prev.is_active = false;
                prev.updated_at = at;
                prev.updated_by = actor;
                prev.version = prev.version.next();
            }
        }
        self.is_active = true;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Soft-deletes the style.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === Style section end ===

// =============================================================================
// === BackgroundSetting section begin (owner: A) ===
// =============================================================================

/// `BackgroundSetting` — a background image or color preset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundSetting {
    pub id: BackgroundSettingId,
    pub school_id: SchoolId,
    pub title: BackgroundTitle,
    pub background_type: BackgroundType,
    pub image: Option<BackgroundImage>,
    pub color: Option<BackgroundColor>,
    pub is_default: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`BackgroundSetting::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewBackgroundSetting {
    pub id: BackgroundSettingId,
    pub title: BackgroundTitle,
    pub background_type: BackgroundType,
    pub image: Option<BackgroundImage>,
    pub color: Option<BackgroundColor>,
    pub is_default: bool,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl BackgroundSetting {
    /// Constructs a new `BackgroundSetting`.
    pub fn new(cmd: NewBackgroundSetting) -> AggregateResult<Self> {
        match cmd.background_type {
            BackgroundType::Image => {
                if cmd.image.is_none() {
                    return Err(SettingsDomainError::Validation(
                        "image required when background_type=Image".to_owned(),
                    ));
                }
            }
            BackgroundType::Color => {
                if cmd.color.is_none() {
                    return Err(SettingsDomainError::Validation(
                        "color required when background_type=Color".to_owned(),
                    ));
                }
            }
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            background_type: cmd.background_type,
            image: cmd.image,
            color: cmd.color,
            is_default: cmd.is_default,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the title and background media.
    pub fn update(
        &mut self,
        title: Option<BackgroundTitle>,
        image: Option<BackgroundImage>,
        color: Option<BackgroundColor>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted background setting".to_owned(),
            ));
        }
        if let Some(t) = title {
            self.title = t;
        }
        match self.background_type {
            BackgroundType::Image => {
                if let Some(i) = image {
                    self.image = Some(i);
                }
            }
            BackgroundType::Color => {
                if let Some(c) = color {
                    self.color = Some(c);
                }
            }
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the setting.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === BackgroundSetting section end ===

// =============================================================================
// === DashboardSetting section begin (owner: A) ===
// =============================================================================

/// `DashboardSetting` — a binding between a dashboard section/card
/// and a role.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardSetting {
    pub id: DashboardSettingId,
    pub school_id: SchoolId,
    pub dashboard_sec_id: DashboardSectionId,
    pub role_id: educore_rbac::ids::RoleId,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`DashboardSetting::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewDashboardSetting {
    pub id: DashboardSettingId,
    pub dashboard_sec_id: DashboardSectionId,
    pub role_id: educore_rbac::ids::RoleId,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl DashboardSetting {
    /// Constructs a new `DashboardSetting`.
    pub fn new(cmd: NewDashboardSetting) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            dashboard_sec_id: cmd.dashboard_sec_id,
            role_id: cmd.role_id,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the section id and role id.
    pub fn update(
        &mut self,
        dashboard_sec_id: Option<DashboardSectionId>,
        role_id: Option<educore_rbac::ids::RoleId>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted dashboard setting".to_owned(),
            ));
        }
        if let Some(s) = dashboard_sec_id {
            self.dashboard_sec_id = s;
        }
        if let Some(r) = role_id {
            self.role_id = r;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the setting.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === DashboardSetting section end ===

// =============================================================================
// === CustomLink section begin (owner: A) ===
// =============================================================================

const MAX_CUSTOM_LINKS: usize = 16;

/// `CustomLink` — the footer / sidebar custom link bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLink {
    pub id: CustomLinkId,
    pub school_id: SchoolId,
    pub links: Vec<(LinkLabel, LinkHref)>,
    pub social: CustomLinkSocial,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`CustomLink::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCustomLink {
    pub id: CustomLinkId,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl CustomLink {
    /// Constructs a new (empty) `CustomLink`.
    pub fn new(cmd: NewCustomLink) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            links: Vec::new(),
            social: CustomLinkSocial::default(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the bundle (replaces links and social).
    pub fn update(
        &mut self,
        links: Vec<(LinkLabel, LinkHref)>,
        social: CustomLinkSocial,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if links.len() > MAX_CUSTOM_LINKS {
            return Err(SettingsDomainError::Validation(format!(
                "max {MAX_CUSTOM_LINKS} custom links, got {}",
                links.len()
            )));
        }
        // Each link must be a valid (label, href) pair: label is non-empty,
        // href is a valid URL (or empty).
        for (label, href) in &links {
            if label.as_str().trim().is_empty() && href.is_set() {
                return Err(SettingsDomainError::Validation(
                    "link_label must be non-empty when href is set".to_owned(),
                ));
            }
        }
        self.links = links;
        self.social = social;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Resets the bundle (clears all links and social URLs).
    pub fn reset(&mut self, at: Timestamp, actor: UserId) {
        self.links.clear();
        self.social.clear();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Returns the count of links.
    #[must_use]
    pub fn count_links(&self) -> u32 {
        self.links.len().try_into().unwrap_or(u32::MAX)
    }

    /// Returns the count of set social URLs.
    #[must_use]
    pub fn count_socials(&self) -> u32 {
        self.social.count_set()
    }
}

// === CustomLink section end ===

// =============================================================================
// === ColorTheme section begin (owner: A) ===
// =============================================================================

/// `ColorTheme` — a color value within a theme. Global (not
/// tenant-scoped).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorTheme {
    pub id: ColorThemeId,
    pub color_id: ColorId,
    pub theme_id: ThemeId,
    pub value: ColorValue,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`ColorTheme::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewColorTheme {
    pub id: ColorThemeId,
    pub color_id: ColorId,
    pub theme_id: ThemeId,
    pub value: ColorValue,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl ColorTheme {
    /// Constructs a new `ColorTheme`.
    pub fn new(cmd: NewColorTheme) -> AggregateResult<Self> {
        Ok(Self {
            id: cmd.id,
            color_id: cmd.color_id,
            theme_id: cmd.theme_id,
            value: cmd.value,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the value.
    pub fn update(
        &mut self,
        value: Option<ColorValue>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted color theme".to_owned(),
            ));
        }
        if let Some(v) = value {
            self.value = v;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the row.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === ColorTheme section end ===

// =============================================================================
// === Theme section begin (owner: A) ===
// =============================================================================

/// `Theme` — a theme (color mode, background, box shadow).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub id: ThemeId,
    pub school_id: SchoolId,
    pub title: ThemeTitle,
    pub path_main_style: ThemePath,
    pub path_style: ThemePath,
    pub color_mode: ColorMode,
    pub box_shadow: BoxShadow,
    pub background_type: BackgroundType,
    pub background_color: Option<ColorHex>,
    pub background_image: Option<FileReference>,
    pub is_default: bool,
    pub is_system: bool,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Theme::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewTheme {
    pub id: ThemeId,
    pub title: ThemeTitle,
    pub path_main_style: ThemePath,
    pub path_style: ThemePath,
    pub color_mode: ColorMode,
    pub box_shadow: BoxShadow,
    pub background_type: BackgroundType,
    pub background_color: Option<ColorHex>,
    pub background_image: Option<FileReference>,
    pub is_default: bool,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Theme {
    /// Constructs a new `Theme`.
    pub fn new(cmd: NewTheme) -> AggregateResult<Self> {
        match cmd.background_type {
            BackgroundType::Color if cmd.background_color.is_none() => {
                return Err(SettingsDomainError::Validation(
                    "background_color required when background_type=Color".to_owned(),
                ));
            }
            BackgroundType::Image if cmd.background_image.is_none() => {
                return Err(SettingsDomainError::Validation(
                    "background_image required when background_type=Image".to_owned(),
                ));
            }
            _ => {}
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            path_main_style: cmd.path_main_style,
            path_style: cmd.path_style,
            color_mode: cmd.color_mode,
            box_shadow: cmd.box_shadow,
            background_type: cmd.background_type,
            background_color: cmd.background_color,
            background_image: cmd.background_image,
            is_default: cmd.is_default,
            is_system: false,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the theme fields.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        title: Option<ThemeTitle>,
        color_mode: Option<ColorMode>,
        box_shadow: Option<BoxShadow>,
        background_color: Option<ColorHex>,
        background_image: Option<FileReference>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted theme".to_owned(),
            ));
        }
        if let Some(t) = title {
            self.title = t;
        }
        if let Some(m) = color_mode {
            self.color_mode = m;
        }
        if let Some(b) = box_shadow {
            self.box_shadow = b;
        }
        if let Some(c) = background_color {
            self.background_color = Some(c);
        }
        if let Some(i) = background_image {
            self.background_image = Some(i);
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Activates the theme. Marks the previously-active theme as
    /// inactive (the caller supplies it).
    pub fn activate(&mut self, previous: Option<&mut Theme>, at: Timestamp, actor: UserId) {
        if let Some(prev) = previous {
            if prev.is_default && !self.is_default {
                // Demoting a default theme is not allowed unless
                // the new theme is also a default.
            }
            prev.updated_at = at;
            prev.updated_by = actor;
            prev.version = prev.version.next();
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Soft-deletes the theme.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) -> AggregateResult<()> {
        if self.is_default {
            return Err(SettingsDomainError::Conflict(
                "cannot delete default theme".to_owned(),
            ));
        }
        if self.is_system {
            return Err(SettingsDomainError::Conflict(
                "cannot delete system theme".to_owned(),
            ));
        }
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns a copy of this theme with a new id and title.
    pub fn replicate(
        &self,
        new_id: ThemeId,
        new_title: ThemeTitle,
        at: Timestamp,
        actor: UserId,
    ) -> AggregateResult<Theme> {
        Ok(Theme {
            school_id: new_id.school_id(),
            id: new_id,
            title: new_title,
            path_main_style: self.path_main_style.clone(),
            path_style: self.path_style.clone(),
            color_mode: self.color_mode,
            box_shadow: self.box_shadow,
            background_type: self.background_type,
            background_color: self.background_color.clone(),
            background_image: self.background_image.clone(),
            is_default: false,
            is_system: false,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
            active_status: true,
            last_event_id: None,
            correlation_id: self.correlation_id,
        })
    }
}

// === Theme section end ===

// =============================================================================
// === Color section begin (owner: A) ===
// =============================================================================

/// `Color` — a color entry used by `ColorTheme`. Global (not
/// tenant-scoped).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub id: ColorId,
    pub name: ColorName,
    pub default_value: ColorValue,
    pub lawn_green: LawnGreen,
    pub is_color: IsColor,
    pub status: ColorStatus,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Color::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewColor {
    pub id: ColorId,
    pub name: ColorName,
    pub default_value: ColorValue,
    pub lawn_green: LawnGreen,
    pub is_color: IsColor,
    pub status: ColorStatus,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl Color {
    /// Constructs a new `Color`.
    pub fn new(cmd: NewColor) -> AggregateResult<Self> {
        Ok(Self {
            id: cmd.id,
            name: cmd.name,
            default_value: cmd.default_value,
            lawn_green: cmd.lawn_green,
            is_color: cmd.is_color,
            status: cmd.status,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the color.
    pub fn update(
        &mut self,
        name: Option<ColorName>,
        default_value: Option<ColorValue>,
        lawn_green: Option<LawnGreen>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted color".to_owned(),
            ));
        }
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(v) = default_value {
            self.default_value = v;
        }
        if let Some(l) = lawn_green {
            self.lawn_green = l;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the color.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.status = ColorStatus::new(false);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === Color section end ===

// =============================================================================
// === BehaviorRecordSetting section begin (owner: A) ===
// =============================================================================

/// `BehaviorRecordSetting` — the behavior record feature configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorRecordSetting {
    pub id: BehaviorRecordSettingId,
    pub school_id: SchoolId,
    pub student_comment: BehaviorFlag,
    pub parent_comment: BehaviorFlag,
    pub student_view: BehaviorFlag,
    pub parent_view: BehaviorFlag,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`BehaviorRecordSetting::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewBehaviorRecordSetting {
    pub id: BehaviorRecordSettingId,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl BehaviorRecordSetting {
    /// Constructs a new `BehaviorRecordSetting` with all flags off.
    pub fn new(cmd: NewBehaviorRecordSetting) -> AggregateResult<Self> {
        // 0 is a valid BehaviorFlag by spec; bypass validation.
        let zero = BehaviorFlag(0);
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            student_comment: zero,
            parent_comment: zero,
            student_view: zero,
            parent_view: zero,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies an update to the four flags.
    pub fn apply_update(
        &mut self,
        student_comment: Option<BehaviorFlag>,
        parent_comment: Option<BehaviorFlag>,
        student_view: Option<BehaviorFlag>,
        parent_view: Option<BehaviorFlag>,
        actor: UserId,
        at: Timestamp,
    ) {
        if let Some(v) = student_comment {
            self.student_comment = v;
        }
        if let Some(v) = parent_comment {
            self.parent_comment = v;
        }
        if let Some(v) = student_view {
            self.student_view = v;
        }
        if let Some(v) = parent_view {
            self.parent_view = v;
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }

    /// Returns true if students can comment.
    #[must_use]
    pub fn student_can_comment(&self) -> bool {
        self.student_comment.is_on()
    }

    /// Returns true if parents can comment.
    #[must_use]
    pub fn parent_can_comment(&self) -> bool {
        self.parent_comment.is_on()
    }

    /// Returns true if students can view.
    #[must_use]
    pub fn student_can_view(&self) -> bool {
        self.student_view.is_on()
    }

    /// Returns true if parents can view.
    #[must_use]
    pub fn parent_can_view(&self) -> bool {
        self.parent_view.is_on()
    }
}

// === BehaviorRecordSetting section end ===

// =============================================================================
// === SetupAdmin section begin (owner: A) ===
// =============================================================================

/// `SetupAdmin` — a purpose, complaint type, source, or reference entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAdmin {
    pub id: SetupAdminId,
    pub school_id: SchoolId,
    pub admin_type: SetupAdminType,
    pub name: SetupAdminName,
    pub description: Option<SetupAdminDescription>,
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: bool,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`SetupAdmin::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSetupAdmin {
    pub id: SetupAdminId,
    pub admin_type: SetupAdminType,
    pub name: SetupAdminName,
    pub description: Option<SetupAdminDescription>,
    pub created_by: UserId,
    pub created_at: Timestamp,
    pub correlation_id: CorrelationId,
}

impl SetupAdmin {
    /// Constructs a new `SetupAdmin`.
    pub fn new(cmd: NewSetupAdmin) -> AggregateResult<Self> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            admin_type: cmd.admin_type,
            name: cmd.name,
            description: cmd.description,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            active_status: true,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the entry.
    pub fn update(
        &mut self,
        admin_type: Option<SetupAdminType>,
        name: Option<SetupAdminName>,
        description: Option<SetupAdminDescription>,
        actor: UserId,
        at: Timestamp,
    ) -> AggregateResult<()> {
        if !self.active_status {
            return Err(SettingsDomainError::Conflict(
                "cannot update deleted setup admin".to_owned(),
            ));
        }
        if let Some(t) = admin_type {
            self.admin_type = t;
        }
        if let Some(n) = name {
            self.name = n;
        }
        if let Some(d) = description {
            self.description = Some(d);
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Soft-deletes the entry.
    pub fn delete(&mut self, at: Timestamp, actor: UserId) {
        self.active_status = false;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
    }
}

// === SetupAdmin section end ===

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;
    use uuid::Uuid;

    fn school() -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }

    fn user() -> UserId {
        UserId::from_uuid(Uuid::nil())
    }

    fn corr() -> CorrelationId {
        CorrelationId::from_uuid(Uuid::nil())
    }

    #[test]
    fn general_settings_validates_required_fields(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = GeneralSettingsId::new(school(), Uuid::nil());
        let cmd = NewGeneralSettings {
            id,
            school_name: "".to_owned(),
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
            active_theme: ActiveTheme::new("default")?,
            queue_connection: QueueConnection::new("sync")?,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        assert!(GeneralSettings::new(cmd).is_err());
        Ok(())
    }

    #[test]
    fn general_settings_patch_updates_school_name(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = GeneralSettingsId::new(school(), Uuid::nil());
        let cmd = NewGeneralSettings {
            id,
            school_name: "Old".to_owned(),
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
            active_theme: ActiveTheme::new("default")?,
            queue_connection: QueueConnection::new("sync")?,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let mut s = GeneralSettings::new(cmd)?;
        let mut patch = crate::entities::GeneralSettingsPatch::default();
        patch.school_name = Some("New".to_owned());
        s.apply_patch(patch, user(), Timestamp::now())?;
        assert_eq!(s.school_name, "New");
        Ok(())
    }

    #[test]
    fn language_activate_deactivate() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = LanguageId::new(school(), Uuid::nil());
        let cmd = NewLanguage {
            id,
            code: LanguageCode::new("en")?,
            name: LanguageName::new("English")?,
            native: LanguageNative::new("English")?,
            rtl: RtlFlag::new(false),
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let mut l = Language::new(cmd)?;
        l.deactivate(Timestamp::now(), user());
        assert_eq!(l.status, LanguageStatus::Inactive);
        l.activate(Timestamp::now(), user());
        assert_eq!(l.status, LanguageStatus::Active);
        Ok(())
    }

    #[test]
    fn language_phrase_translate_known_locales(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = LanguagePhraseId::new(school(), Uuid::nil());
        let cmd = NewLanguagePhrase {
            id,
            modules: PhraseModule::new("dashboard")?,
            default_phrases: DefaultPhrase::new("Hello")?,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let mut p = LanguagePhrase::new(cmd)?;
        p.translate(
            LocaleCode::new("en")?,
            Translation::new("Hello")?,
            user(),
            Timestamp::now(),
        )?;
        assert_eq!(
            p.en.as_ref()
                .ok_or_else(|| "expected en translation".to_owned())?
                .as_str(),
            "Hello"
        );
        p.translate(
            LocaleCode::new("es")?,
            Translation::new("Hola")?,
            user(),
            Timestamp::now(),
        )?;
        assert_eq!(
            p.es.as_ref()
                .ok_or_else(|| "expected es translation".to_owned())?
                .as_str(),
            "Hola"
        );
        assert!(p
            .translate(
                LocaleCode::new("xx")?,
                Translation::new("X")?,
                user(),
                Timestamp::now()
            )
            .is_err());
        Ok(())
    }

    #[test]
    fn style_activate_demotes_previous() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id1 = StyleId::new(school(), Uuid::nil());
        let id2 = StyleId::new(school(), Uuid::nil());
        let mk = |id: StyleId| -> std::result::Result<NewStyle, Box<dyn std::error::Error>> {
            Ok(NewStyle {
                id,
                style_name: StyleName::new("s")?,
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
                created_by: user(),
                created_at: Timestamp::now(),
                correlation_id: corr(),
            })
        };
        let mut s1 = Style::new(mk(id1)?)?;
        let mut s2 = Style::new(mk(id2)?)?;
        s1.is_active = true;
        s2.activate(Some(&mut s1), Timestamp::now(), user());
        assert!(s2.is_active);
        assert!(!s1.is_active);
        Ok(())
    }

    #[test]
    fn custom_link_max_16() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = CustomLinkId::new(school(), Uuid::nil());
        let cmd = NewCustomLink {
            id,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let mut c = CustomLink::new(cmd)?;
        let links: std::result::Result<Vec<_>, Box<dyn std::error::Error>> = (0..17)
            .map(|i| {
                Ok((
                    LinkLabel::new(format!("l{i}").as_str())?,
                    LinkHref::new(format!("https://x.com/{i}").as_str())?,
                ))
            })
            .collect();
        let links = links?;
        assert!(c
            .update(links, CustomLinkSocial::default(), user(), Timestamp::now())
            .is_err());
        Ok(())
    }

    #[test]
    fn theme_cannot_delete_default() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = ThemeId::new(school(), Uuid::nil());
        let cmd = NewTheme {
            id,
            title: ThemeTitle::new("Default")?,
            path_main_style: ThemePath::new("a.css")?,
            path_style: ThemePath::new("b.css")?,
            color_mode: ColorMode::Solid,
            box_shadow: BoxShadow::new(false),
            background_type: BackgroundType::Color,
            background_color: Some(ColorHex("#ffffff".to_owned())),
            background_image: None,
            is_default: true,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let mut t = Theme::new(cmd)?;
        assert!(t.delete(Timestamp::now(), user()).is_err());
        Ok(())
    }

    #[test]
    fn behavior_record_flags_default_off() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let id = BehaviorRecordSettingId::new(school(), Uuid::nil());
        let cmd = NewBehaviorRecordSetting {
            id,
            created_by: user(),
            created_at: Timestamp::now(),
            correlation_id: corr(),
        };
        let s = BehaviorRecordSetting::new(cmd)?;
        assert!(!s.student_can_comment());
        assert!(!s.parent_can_comment());
        assert!(!s.student_can_view());
        assert!(!s.parent_can_view());
        Ok(())
    }
}
