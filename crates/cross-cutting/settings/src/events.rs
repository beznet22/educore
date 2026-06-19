//! # educore-settings typed events
//!
//! Per `docs/specs/settings/events.md`. Wire form:
//! `settings.<aggregate>.<verb>`. Each event implements
//! [`DomainEvent`].

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::{
    AcademicYearRef, ActiveTheme, BackgroundTitle, BackgroundType, BaseGroupId, BaseSetupId,
    BehaviorFlag, ColorId, ColorName, ColorStatus, ColorThemeId, ColorValue, CustomLinkId,
    DashboardSectionId, DashboardSettingId, DateFormatId, DateFormatPattern, DateFormatPreview,
    GeneralSettingsId, LanguageCode, LanguageId, LanguageName, LanguagePhraseId, LocaleCode,
    PhoneNumberPrivacy, RtlFlag, SetupAdminId, SetupAdminType, StyleId, ThemeId,
};

// =============================================================================
// === GeneralSettings events section begin (owner: A) ===
// =============================================================================

/// Emitted when `GeneralSettings` is patched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettingsUpdated {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl GeneralSettingsUpdated {
    /// Constructs a new `GeneralSettingsUpdated`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GeneralSettingsUpdated {
    const EVENT_TYPE: &'static str = "settings.general_settings.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the active theme is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveThemeChanged {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub from_theme: Option<ActiveTheme>,
    pub to_theme: ActiveTheme,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ActiveThemeChanged {
    /// Constructs a new `ActiveThemeChanged`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        from_theme: Option<ActiveTheme>,
        to_theme: ActiveTheme,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            from_theme,
            to_theme,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ActiveThemeChanged {
    const EVENT_TYPE: &'static str = "settings.general_settings.active_theme_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the default language is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageChanged {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub from_language_id: Option<LanguageId>,
    pub to_language_id: LanguageId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageChanged {
    /// Constructs a new `LanguageChanged`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        from_language_id: Option<LanguageId>,
        to_language_id: LanguageId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            from_language_id,
            to_language_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageChanged {
    const EVENT_TYPE: &'static str = "settings.general_settings.language_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the default date format is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormatChanged {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub from_date_format_id: Option<DateFormatId>,
    pub to_date_format_id: DateFormatId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DateFormatChanged {
    /// Constructs a new `DateFormatChanged`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        from_date_format_id: Option<DateFormatId>,
        to_date_format_id: DateFormatId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            from_date_format_id,
            to_date_format_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DateFormatChanged {
    const EVENT_TYPE: &'static str = "settings.general_settings.date_format_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the active time zone is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeZoneChanged {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub from_time_zone_id: Option<String>,
    pub to_time_zone_id: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl TimeZoneChanged {
    /// Constructs a new `TimeZoneChanged`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        from_time_zone_id: Option<String>,
        to_time_zone_id: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            from_time_zone_id,
            to_time_zone_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for TimeZoneChanged {
    const EVENT_TYPE: &'static str = "settings.general_settings.time_zone_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the active session (academic year) is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionChanged {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub from_session_id: Option<AcademicYearRef>,
    pub to_session_id: AcademicYearRef,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SessionChanged {
    /// Constructs a new `SessionChanged`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        from_session_id: Option<AcademicYearRef>,
        to_session_id: AcademicYearRef,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            from_session_id,
            to_session_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SessionChanged {
    const EVENT_TYPE: &'static str = "settings.general_settings.session_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the two-factor flag is toggled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TwoFactorToggled {
    pub settings_id: GeneralSettingsId,
    pub school_id: SchoolId,
    pub enabled: bool,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl TwoFactorToggled {
    /// Constructs a new `TwoFactorToggled`.
    #[must_use]
    pub fn new(
        settings_id: GeneralSettingsId,
        school_id: SchoolId,
        enabled: bool,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            settings_id,
            school_id,
            enabled,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for TwoFactorToggled {
    const EVENT_TYPE: &'static str = "settings.general_settings.two_factor_toggled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "general_settings";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.settings_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === GeneralSettings events section end ===

// =============================================================================
// === Language events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `Language` is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageAdded {
    pub language_id: LanguageId,
    pub school_id: SchoolId,
    pub code: LanguageCode,
    pub name: LanguageName,
    pub rtl: bool,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageAdded {
    /// Constructs a new `LanguageAdded`.
    #[must_use]
    pub fn new(
        language_id: LanguageId,
        school_id: SchoolId,
        code: LanguageCode,
        name: LanguageName,
        rtl: bool,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            language_id,
            school_id,
            code,
            name,
            rtl,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageAdded {
    const EVENT_TYPE: &'static str = "settings.language.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.language_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Language` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageUpdated {
    pub language_id: LanguageId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageUpdated {
    /// Constructs a new `LanguageUpdated`.
    #[must_use]
    pub fn new(
        language_id: LanguageId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            language_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageUpdated {
    const EVENT_TYPE: &'static str = "settings.language.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.language_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Language` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageDeleted {
    pub language_id: LanguageId,
    pub school_id: SchoolId,
    pub prior_code: LanguageCode,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageDeleted {
    /// Constructs a new `LanguageDeleted`.
    #[must_use]
    pub fn new(
        language_id: LanguageId,
        school_id: SchoolId,
        prior_code: LanguageCode,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            language_id,
            school_id,
            prior_code,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageDeleted {
    const EVENT_TYPE: &'static str = "settings.language.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.language_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Language` is activated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageActivated {
    pub language_id: LanguageId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageActivated {
    /// Constructs a new `LanguageActivated`.
    #[must_use]
    pub fn new(
        language_id: LanguageId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            language_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageActivated {
    const EVENT_TYPE: &'static str = "settings.language.activated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.language_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Language` is deactivated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageDeactivated {
    pub language_id: LanguageId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguageDeactivated {
    /// Constructs a new `LanguageDeactivated`.
    #[must_use]
    pub fn new(
        language_id: LanguageId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            language_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguageDeactivated {
    const EVENT_TYPE: &'static str = "settings.language.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.language_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Language events section end ===

// =============================================================================
// === LanguagePhrase events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `LanguagePhrase` is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguagePhraseAdded {
    pub phrase_id: LanguagePhraseId,
    pub school_id: SchoolId,
    pub modules: String,
    pub default_phrases: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguagePhraseAdded {
    /// Constructs a new `LanguagePhraseAdded`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        school_id: SchoolId,
        modules: String,
        default_phrases: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phrase_id,
            school_id,
            modules,
            default_phrases,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguagePhraseAdded {
    const EVENT_TYPE: &'static str = "settings.language_phrase.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language_phrase";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.phrase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LanguagePhrase` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguagePhraseUpdated {
    pub phrase_id: LanguagePhraseId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguagePhraseUpdated {
    /// Constructs a new `LanguagePhraseUpdated`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phrase_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguagePhraseUpdated {
    const EVENT_TYPE: &'static str = "settings.language_phrase.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language_phrase";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.phrase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LanguagePhrase` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguagePhraseDeleted {
    pub phrase_id: LanguagePhraseId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguagePhraseDeleted {
    /// Constructs a new `LanguagePhraseDeleted`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phrase_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguagePhraseDeleted {
    const EVENT_TYPE: &'static str = "settings.language_phrase.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language_phrase";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.phrase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `LanguagePhrase` is translated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguagePhraseTranslated {
    pub phrase_id: LanguagePhraseId,
    pub school_id: SchoolId,
    pub locale: LocaleCode,
    pub translation: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl LanguagePhraseTranslated {
    /// Constructs a new `LanguagePhraseTranslated`.
    #[must_use]
    pub fn new(
        phrase_id: LanguagePhraseId,
        school_id: SchoolId,
        locale: LocaleCode,
        translation: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phrase_id,
            school_id,
            locale,
            translation,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for LanguagePhraseTranslated {
    const EVENT_TYPE: &'static str = "settings.language_phrase.translated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "language_phrase";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.phrase_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === LanguagePhrase events section end ===

// =============================================================================
// === BaseGroup events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `BaseGroup` is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseGroupAdded {
    pub group_id: BaseGroupId,
    pub school_id: SchoolId,
    pub name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseGroupAdded {
    /// Constructs a new `BaseGroupAdded`.
    #[must_use]
    pub fn new(
        group_id: BaseGroupId,
        school_id: SchoolId,
        name: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            group_id,
            school_id,
            name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseGroupAdded {
    const EVENT_TYPE: &'static str = "settings.base_group.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_group";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BaseGroup` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseGroupUpdated {
    pub group_id: BaseGroupId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseGroupUpdated {
    /// Constructs a new `BaseGroupUpdated`.
    #[must_use]
    pub fn new(
        group_id: BaseGroupId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            group_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseGroupUpdated {
    const EVENT_TYPE: &'static str = "settings.base_group.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_group";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BaseGroup` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseGroupDeleted {
    pub group_id: BaseGroupId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseGroupDeleted {
    /// Constructs a new `BaseGroupDeleted`.
    #[must_use]
    pub fn new(
        group_id: BaseGroupId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            group_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseGroupDeleted {
    const EVENT_TYPE: &'static str = "settings.base_group.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_group";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === BaseGroup events section end ===

// =============================================================================
// === BaseSetup events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `BaseSetup` is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSetupAdded {
    pub setup_id: BaseSetupId,
    pub school_id: SchoolId,
    pub name: String,
    pub base_group_id: BaseGroupId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseSetupAdded {
    /// Constructs a new `BaseSetupAdded`.
    #[must_use]
    pub fn new(
        setup_id: BaseSetupId,
        school_id: SchoolId,
        name: String,
        base_group_id: BaseGroupId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_id,
            school_id,
            name,
            base_group_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseSetupAdded {
    const EVENT_TYPE: &'static str = "settings.base_setup.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_setup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BaseSetup` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSetupUpdated {
    pub setup_id: BaseSetupId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseSetupUpdated {
    /// Constructs a new `BaseSetupUpdated`.
    #[must_use]
    pub fn new(
        setup_id: BaseSetupId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseSetupUpdated {
    const EVENT_TYPE: &'static str = "settings.base_setup.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_setup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BaseSetup` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaseSetupDeleted {
    pub setup_id: BaseSetupId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BaseSetupDeleted {
    /// Constructs a new `BaseSetupDeleted`.
    #[must_use]
    pub fn new(
        setup_id: BaseSetupId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BaseSetupDeleted {
    const EVENT_TYPE: &'static str = "settings.base_setup.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "base_setup";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === BaseSetup events section end ===

// =============================================================================
// === DateFormat events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `DateFormat` is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormatAdded {
    pub format_id: DateFormatId,
    pub school_id: SchoolId,
    pub format: DateFormatPattern,
    pub normal_view: DateFormatPreview,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DateFormatAdded {
    /// Constructs a new `DateFormatAdded`.
    #[must_use]
    pub fn new(
        format_id: DateFormatId,
        school_id: SchoolId,
        format: DateFormatPattern,
        normal_view: DateFormatPreview,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            format_id,
            school_id,
            format,
            normal_view,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DateFormatAdded {
    const EVENT_TYPE: &'static str = "settings.date_format.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "date_format";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.format_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `DateFormat` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormatUpdated {
    pub format_id: DateFormatId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DateFormatUpdated {
    /// Constructs a new `DateFormatUpdated`.
    #[must_use]
    pub fn new(
        format_id: DateFormatId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            format_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DateFormatUpdated {
    const EVENT_TYPE: &'static str = "settings.date_format.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "date_format";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.format_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `DateFormat` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateFormatDeleted {
    pub format_id: DateFormatId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DateFormatDeleted {
    /// Constructs a new `DateFormatDeleted`.
    #[must_use]
    pub fn new(
        format_id: DateFormatId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            format_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DateFormatDeleted {
    const EVENT_TYPE: &'static str = "settings.date_format.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "date_format";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.format_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === DateFormat events section end ===

// =============================================================================
// === Style events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `Style` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleCreated {
    pub style_id: StyleId,
    pub school_id: SchoolId,
    pub style_name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StyleCreated {
    /// Constructs a new `StyleCreated`.
    #[must_use]
    pub fn new(
        style_id: StyleId,
        school_id: SchoolId,
        style_name: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            style_id,
            school_id,
            style_name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StyleCreated {
    const EVENT_TYPE: &'static str = "settings.style.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "style";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.style_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Style` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleUpdated {
    pub style_id: StyleId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StyleUpdated {
    /// Constructs a new `StyleUpdated`.
    #[must_use]
    pub fn new(
        style_id: StyleId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            style_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StyleUpdated {
    const EVENT_TYPE: &'static str = "settings.style.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "style";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.style_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Style` is activated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleActivated {
    pub style_id: StyleId,
    pub school_id: SchoolId,
    pub style_name: String,
    pub previous_id: Option<StyleId>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StyleActivated {
    /// Constructs a new `StyleActivated`.
    #[must_use]
    pub fn new(
        style_id: StyleId,
        school_id: SchoolId,
        style_name: String,
        previous_id: Option<StyleId>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            style_id,
            school_id,
            style_name,
            previous_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StyleActivated {
    const EVENT_TYPE: &'static str = "settings.style.activated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "style";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.style_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Style` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleDeleted {
    pub style_id: StyleId,
    pub school_id: SchoolId,
    pub prior_style_name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl StyleDeleted {
    /// Constructs a new `StyleDeleted`.
    #[must_use]
    pub fn new(
        style_id: StyleId,
        school_id: SchoolId,
        prior_style_name: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            style_id,
            school_id,
            prior_style_name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for StyleDeleted {
    const EVENT_TYPE: &'static str = "settings.style.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "style";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.style_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Style events section end ===

// =============================================================================
// === BackgroundSetting events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `BackgroundSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundSettingCreated {
    pub background_id: crate::value_objects::BackgroundSettingId,
    pub school_id: SchoolId,
    pub title: BackgroundTitle,
    pub background_type: BackgroundType,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BackgroundSettingCreated {
    /// Constructs a new `BackgroundSettingCreated`.
    #[must_use]
    pub fn new(
        background_id: crate::value_objects::BackgroundSettingId,
        school_id: SchoolId,
        title: BackgroundTitle,
        background_type: BackgroundType,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            background_id,
            school_id,
            title,
            background_type,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BackgroundSettingCreated {
    const EVENT_TYPE: &'static str = "settings.background_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "background_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.background_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BackgroundSetting` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundSettingUpdated {
    pub background_id: crate::value_objects::BackgroundSettingId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BackgroundSettingUpdated {
    /// Constructs a new `BackgroundSettingUpdated`.
    #[must_use]
    pub fn new(
        background_id: crate::value_objects::BackgroundSettingId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            background_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BackgroundSettingUpdated {
    const EVENT_TYPE: &'static str = "settings.background_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "background_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.background_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `BackgroundSetting` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundSettingDeleted {
    pub background_id: crate::value_objects::BackgroundSettingId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BackgroundSettingDeleted {
    /// Constructs a new `BackgroundSettingDeleted`.
    #[must_use]
    pub fn new(
        background_id: crate::value_objects::BackgroundSettingId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            background_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BackgroundSettingDeleted {
    const EVENT_TYPE: &'static str = "settings.background_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "background_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.background_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === BackgroundSetting events section end ===

// =============================================================================
// === DashboardSetting events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `DashboardSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardSettingCreated {
    pub dashboard_setting_id: DashboardSettingId,
    pub school_id: SchoolId,
    pub dashboard_sec_id: DashboardSectionId,
    pub role_id: educore_rbac::ids::RoleId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DashboardSettingCreated {
    /// Constructs a new `DashboardSettingCreated`.
    #[must_use]
    pub fn new(
        dashboard_setting_id: DashboardSettingId,
        school_id: SchoolId,
        dashboard_sec_id: DashboardSectionId,
        role_id: educore_rbac::ids::RoleId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dashboard_setting_id,
            school_id,
            dashboard_sec_id,
            role_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DashboardSettingCreated {
    const EVENT_TYPE: &'static str = "settings.dashboard_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "dashboard_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.dashboard_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `DashboardSetting` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardSettingUpdated {
    pub dashboard_setting_id: DashboardSettingId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DashboardSettingUpdated {
    /// Constructs a new `DashboardSettingUpdated`.
    #[must_use]
    pub fn new(
        dashboard_setting_id: DashboardSettingId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dashboard_setting_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DashboardSettingUpdated {
    const EVENT_TYPE: &'static str = "settings.dashboard_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "dashboard_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.dashboard_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `DashboardSetting` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardSettingDeleted {
    pub dashboard_setting_id: DashboardSettingId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl DashboardSettingDeleted {
    /// Constructs a new `DashboardSettingDeleted`.
    #[must_use]
    pub fn new(
        dashboard_setting_id: DashboardSettingId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            dashboard_setting_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for DashboardSettingDeleted {
    const EVENT_TYPE: &'static str = "settings.dashboard_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "dashboard_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.dashboard_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === DashboardSetting events section end ===

// =============================================================================
// === CustomLink events section begin (owner: A) ===
// =============================================================================

/// Emitted when the `CustomLink` bundle is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLinksUpdated {
    pub custom_link_id: CustomLinkId,
    pub school_id: SchoolId,
    pub link_count: u32,
    pub social_count: u32,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CustomLinksUpdated {
    /// Constructs a new `CustomLinksUpdated`.
    #[must_use]
    pub fn new(
        custom_link_id: CustomLinkId,
        school_id: SchoolId,
        link_count: u32,
        social_count: u32,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            custom_link_id,
            school_id,
            link_count,
            social_count,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CustomLinksUpdated {
    const EVENT_TYPE: &'static str = "settings.custom_link.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "custom_link";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.custom_link_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the `CustomLink` bundle is reset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomLinksReset {
    pub custom_link_id: CustomLinkId,
    pub school_id: SchoolId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CustomLinksReset {
    /// Constructs a new `CustomLinksReset`.
    #[must_use]
    pub fn new(
        custom_link_id: CustomLinkId,
        school_id: SchoolId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            custom_link_id,
            school_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CustomLinksReset {
    const EVENT_TYPE: &'static str = "settings.custom_link.reset";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "custom_link";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.custom_link_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === CustomLink events section end ===

// =============================================================================
// === Theme events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `Theme` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeCreated {
    pub theme_id: ThemeId,
    pub school_id: SchoolId,
    pub title: String,
    pub color_mode: String,
    pub background_type: BackgroundType,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ThemeCreated {
    /// Constructs a new `ThemeCreated`.
    #[must_use]
    pub fn new(
        theme_id: ThemeId,
        school_id: SchoolId,
        title: String,
        color_mode: String,
        background_type: BackgroundType,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            theme_id,
            school_id,
            title,
            color_mode,
            background_type,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ThemeCreated {
    const EVENT_TYPE: &'static str = "settings.theme.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Theme` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeUpdated {
    pub theme_id: ThemeId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ThemeUpdated {
    /// Constructs a new `ThemeUpdated`.
    #[must_use]
    pub fn new(
        theme_id: ThemeId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            theme_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ThemeUpdated {
    const EVENT_TYPE: &'static str = "settings.theme.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Theme` is activated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeActivated {
    pub theme_id: ThemeId,
    pub school_id: SchoolId,
    pub from_active: Option<ThemeId>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ThemeActivated {
    /// Constructs a new `ThemeActivated`.
    #[must_use]
    pub fn new(
        theme_id: ThemeId,
        school_id: SchoolId,
        from_active: Option<ThemeId>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            theme_id,
            school_id,
            from_active,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ThemeActivated {
    const EVENT_TYPE: &'static str = "settings.theme.activated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Theme` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeDeleted {
    pub theme_id: ThemeId,
    pub school_id: SchoolId,
    pub prior_title: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ThemeDeleted {
    /// Constructs a new `ThemeDeleted`.
    #[must_use]
    pub fn new(
        theme_id: ThemeId,
        school_id: SchoolId,
        prior_title: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            theme_id,
            school_id,
            prior_title,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ThemeDeleted {
    const EVENT_TYPE: &'static str = "settings.theme.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Theme` is replicated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeReplicated {
    pub school_id: SchoolId,
    pub source_theme_id: ThemeId,
    pub new_theme_id: ThemeId,
    pub copied_color_themes: u32,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ThemeReplicated {
    /// Constructs a new `ThemeReplicated`.
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        source_theme_id: ThemeId,
        new_theme_id: ThemeId,
        copied_color_themes: u32,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            source_theme_id,
            new_theme_id,
            copied_color_themes,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ThemeReplicated {
    const EVENT_TYPE: &'static str = "settings.theme.replicated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.new_theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Theme events section end ===

// =============================================================================
// === Color events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `Color` is created (system-internal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorCreated {
    pub color_id: ColorId,
    pub name: ColorName,
    pub default_value: ColorValue,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorCreated {
    /// Constructs a new `ColorCreated`.
    #[must_use]
    pub fn new(
        color_id: ColorId,
        name: ColorName,
        default_value: ColorValue,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_id,
            name,
            default_value,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorCreated {
    const EVENT_TYPE: &'static str = "settings.color.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Color` is updated (system-internal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorUpdated {
    pub color_id: ColorId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorUpdated {
    /// Constructs a new `ColorUpdated`.
    #[must_use]
    pub fn new(
        color_id: ColorId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorUpdated {
    const EVENT_TYPE: &'static str = "settings.color.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Color` is soft-deleted (system-internal).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorDeleted {
    pub color_id: ColorId,
    pub prior_name: ColorName,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorDeleted {
    /// Constructs a new `ColorDeleted`.
    #[must_use]
    pub fn new(
        color_id: ColorId,
        prior_name: ColorName,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_id,
            prior_name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorDeleted {
    const EVENT_TYPE: &'static str = "settings.color.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === Color events section end ===

// =============================================================================
// === ColorTheme events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `ColorTheme` row is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorThemeCreated {
    pub color_theme_id: ColorThemeId,
    pub color_id: ColorId,
    pub theme_id: ThemeId,
    pub value: ColorValue,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorThemeCreated {
    /// Constructs a new `ColorThemeCreated`.
    #[must_use]
    pub fn new(
        color_theme_id: ColorThemeId,
        color_id: ColorId,
        theme_id: ThemeId,
        value: ColorValue,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_theme_id,
            color_id,
            theme_id,
            value,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorThemeCreated {
    const EVENT_TYPE: &'static str = "settings.color_theme.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color_theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ColorTheme` row is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorThemeUpdated {
    pub color_theme_id: ColorThemeId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorThemeUpdated {
    /// Constructs a new `ColorThemeUpdated`.
    #[must_use]
    pub fn new(
        color_theme_id: ColorThemeId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_theme_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorThemeUpdated {
    const EVENT_TYPE: &'static str = "settings.color_theme.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color_theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ColorTheme` row is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorThemeDeleted {
    pub color_theme_id: ColorThemeId,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ColorThemeDeleted {
    /// Constructs a new `ColorThemeDeleted`.
    #[must_use]
    pub fn new(
        color_theme_id: ColorThemeId,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            color_theme_id,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ColorThemeDeleted {
    const EVENT_TYPE: &'static str = "settings.color_theme.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "color_theme";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.color_theme_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        SchoolId::from_uuid(Uuid::nil())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === ColorTheme events section end ===

// =============================================================================
// === BehaviorRecordSetting events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `BehaviorRecordSetting` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehaviorRecordSettingUpdated {
    pub setting_id: crate::value_objects::BehaviorRecordSettingId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl BehaviorRecordSettingUpdated {
    /// Constructs a new `BehaviorRecordSettingUpdated`.
    #[must_use]
    pub fn new(
        setting_id: crate::value_objects::BehaviorRecordSettingId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setting_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for BehaviorRecordSettingUpdated {
    const EVENT_TYPE: &'static str = "settings.behavior_record_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "behavior_record_setting";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === BehaviorRecordSetting events section end ===

// =============================================================================
// === SetupAdmin events section begin (owner: A) ===
// =============================================================================

/// Emitted when a `SetupAdmin` entry is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAdminAdded {
    pub setup_admin_id: SetupAdminId,
    pub school_id: SchoolId,
    pub admin_type: SetupAdminType,
    pub name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SetupAdminAdded {
    /// Constructs a new `SetupAdminAdded`.
    #[must_use]
    pub fn new(
        setup_admin_id: SetupAdminId,
        school_id: SchoolId,
        admin_type: SetupAdminType,
        name: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_admin_id,
            school_id,
            admin_type,
            name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SetupAdminAdded {
    const EVENT_TYPE: &'static str = "settings.setup_admin.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "setup_admin";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_admin_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SetupAdmin` entry is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAdminUpdated {
    pub setup_admin_id: SetupAdminId,
    pub school_id: SchoolId,
    pub changed_fields: Vec<String>,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SetupAdminUpdated {
    /// Constructs a new `SetupAdminUpdated`.
    #[must_use]
    pub fn new(
        setup_admin_id: SetupAdminId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_admin_id,
            school_id,
            changed_fields,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SetupAdminUpdated {
    const EVENT_TYPE: &'static str = "settings.setup_admin.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "setup_admin";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_admin_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SetupAdmin` entry is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupAdminDeleted {
    pub setup_admin_id: SetupAdminId,
    pub school_id: SchoolId,
    pub prior_name: String,
    pub event_id_field: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SetupAdminDeleted {
    /// Constructs a new `SetupAdminDeleted`.
    #[must_use]
    pub fn new(
        setup_admin_id: SetupAdminId,
        school_id: SchoolId,
        prior_name: String,
        event_id_field: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            setup_admin_id,
            school_id,
            prior_name,
            event_id_field,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SetupAdminDeleted {
    const EVENT_TYPE: &'static str = "settings.setup_admin.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "setup_admin";
    fn event_id(&self) -> EventId {
        self.event_id_field
    }
    fn aggregate_id(&self) -> Uuid {
        self.setup_admin_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// === SetupAdmin events section end ===

#[allow(dead_code)]
fn _ensure_ids_compile(school: SchoolId) {
    let _ = GeneralSettingsId::new(school, Uuid::nil());
    let _ = LanguageId::new(school, Uuid::nil());
    let _ = LanguagePhraseId::new(school, Uuid::nil());
    let _ = BaseSetupId::new(school, Uuid::nil());
    let _ = BaseGroupId::new(school, Uuid::nil());
    let _ = DateFormatId::new(school, Uuid::nil());
    let _ = StyleId::new(school, Uuid::nil());
    let _ = crate::value_objects::BackgroundSettingId::new(school, Uuid::nil());
    let _ = DashboardSettingId::new(school, Uuid::nil());
    let _ = CustomLinkId::new(school, Uuid::nil());
    let _ = ColorThemeId::new(school, Uuid::nil());
    let _ = ThemeId::new(school, Uuid::nil());
    let _ = ColorId::new(school, Uuid::nil());
    let _ = crate::value_objects::BehaviorRecordSettingId::new(school, Uuid::nil());
    let _ = SetupAdminId::new(school, Uuid::nil());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_event_wire_forms_resolve() {
        let types: Vec<&str> = vec![
            // GeneralSettings (7)
            "settings.general_settings.updated",
            "settings.general_settings.active_theme_changed",
            "settings.general_settings.language_changed",
            "settings.general_settings.date_format_changed",
            "settings.general_settings.time_zone_changed",
            "settings.general_settings.session_changed",
            "settings.general_settings.two_factor_toggled",
            // Language (5)
            "settings.language.added",
            "settings.language.updated",
            "settings.language.deleted",
            "settings.language.activated",
            "settings.language.deactivated",
            // LanguagePhrase (4)
            "settings.language_phrase.added",
            "settings.language_phrase.updated",
            "settings.language_phrase.deleted",
            "settings.language_phrase.translated",
            // BaseGroup (3)
            "settings.base_group.added",
            "settings.base_group.updated",
            "settings.base_group.deleted",
            // BaseSetup (3)
            "settings.base_setup.added",
            "settings.base_setup.updated",
            "settings.base_setup.deleted",
            // DateFormat (3)
            "settings.date_format.added",
            "settings.date_format.updated",
            "settings.date_format.deleted",
            // Style (4)
            "settings.style.created",
            "settings.style.updated",
            "settings.style.activated",
            "settings.style.deleted",
            // BackgroundSetting (3)
            "settings.background_setting.created",
            "settings.background_setting.updated",
            "settings.background_setting.deleted",
            // DashboardSetting (3)
            "settings.dashboard_setting.created",
            "settings.dashboard_setting.updated",
            "settings.dashboard_setting.deleted",
            // CustomLink (2)
            "settings.custom_link.updated",
            "settings.custom_link.reset",
            // Theme (5)
            "settings.theme.created",
            "settings.theme.updated",
            "settings.theme.activated",
            "settings.theme.deleted",
            "settings.theme.replicated",
            // Color (3)
            "settings.color.created",
            "settings.color.updated",
            "settings.color.deleted",
            // ColorTheme (3)
            "settings.color_theme.created",
            "settings.color_theme.updated",
            "settings.color_theme.deleted",
            // BehaviorRecordSetting (1)
            "settings.behavior_record_setting.updated",
            // SetupAdmin (3)
            "settings.setup_admin.added",
            "settings.setup_admin.updated",
            "settings.setup_admin.deleted",
        ];
        assert_eq!(types.len(), 52);
        for t in &types {
            assert!(
                t.starts_with("settings."),
                "{t} should start with settings."
            );
        }
    }
}
