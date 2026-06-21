//! # educore-settings repository port traits
//!
//! Per `docs/specs/settings/repositories.md`. 15 repository port
//! traits, one per root aggregate. Each is object-safe (the
//! `_assert_*_object_safe` helpers prove it).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use async_trait::async_trait;

use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    BackgroundSetting, BaseGroup, BaseSetup, BehaviorRecordSetting, Color, ColorTheme, CustomLink,
    DashboardSetting, DateFormat, GeneralSettings, Language, LanguagePhrase, SetupAdmin, Style,
    Theme,
};
use crate::entities::GeneralSettingsPatch;
use crate::value_objects::{
    BackgroundSettingId, BaseGroupId, BaseSetupId, ColorId, ColorThemeId, DashboardSectionId,
    DashboardSettingId, DateFormatId, LanguageCode, LanguageId, LanguagePhraseId, LocaleCode,
    SetupAdminId, SetupAdminType, StyleId, ThemeId, Translation,
};

// =============================================================================
// === GeneralSettingsRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`GeneralSettings`](crate::aggregate::GeneralSettings).
#[async_trait]
pub trait GeneralSettingsRepository: Send + Sync {
    /// Fetch the per-school singleton `GeneralSettings`.
    async fn get(&self, school: SchoolId) -> StorageResult<Option<GeneralSettings>>;
    /// Insert the per-school singleton.
    async fn insert(&self, s: &GeneralSettings) -> StorageResult<()>;
    /// Update the singleton.
    async fn update(&self, s: &GeneralSettings) -> StorageResult<()>;
    /// Apply a patch to the singleton.
    async fn patch(
        &self,
        school: SchoolId,
        patch: GeneralSettingsPatch,
    ) -> StorageResult<GeneralSettings>;
}

fn _assert_general_settings_object_safe() {
    fn _f(_: Box<dyn GeneralSettingsRepository>) {}
}

// === GeneralSettingsRepository section end ===

// =============================================================================
// === LanguageRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`Language`](crate::aggregate::Language).
#[async_trait]
pub trait LanguageRepository: Send + Sync {
    /// Fetch a `Language` by id.
    async fn get(&self, id: LanguageId) -> StorageResult<Option<Language>>;
    /// Find a `Language` by ISO code.
    async fn find_by_code(
        &self,
        school: SchoolId,
        code: &LanguageCode,
    ) -> StorageResult<Option<Language>>;
    /// List languages for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Language>>;
    /// List active languages for a school.
    async fn list_active(&self, school: SchoolId) -> StorageResult<Vec<Language>>;
    /// List right-to-left languages for a school.
    async fn list_rtl(&self, school: SchoolId) -> StorageResult<Vec<Language>>;
    /// Insert a new `Language`.
    async fn insert(&self, l: &Language) -> StorageResult<()>;
    /// Update an existing `Language`.
    async fn update(&self, l: &Language) -> StorageResult<()>;
    /// Soft-delete a `Language`.
    async fn delete(&self, id: LanguageId) -> StorageResult<()>;
    /// Return the number of phrases associated with a language.
    async fn phrase_count(&self, id: LanguageId) -> StorageResult<u64>;
}

fn _assert_language_object_safe() {
    fn _f(_: Box<dyn LanguageRepository>) {}
}

// === LanguageRepository section end ===

// =============================================================================
// === LanguagePhraseRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`LanguagePhrase`](crate::aggregate::LanguagePhrase).
#[async_trait]
pub trait LanguagePhraseRepository: Send + Sync {
    /// Fetch a phrase by id.
    async fn get(&self, id: LanguagePhraseId) -> StorageResult<Option<LanguagePhrase>>;
    /// List phrases for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<LanguagePhrase>>;
    /// List phrases for a module.
    async fn list_for_module(
        &self,
        school: SchoolId,
        module: &str,
    ) -> StorageResult<Vec<LanguagePhrase>>;
    /// Insert a new phrase.
    async fn insert(&self, p: &LanguagePhrase) -> StorageResult<()>;
    /// Update an existing phrase.
    async fn update(&self, p: &LanguagePhrase) -> StorageResult<()>;
    /// Soft-delete a phrase.
    async fn delete(&self, id: LanguagePhraseId) -> StorageResult<()>;
    /// Set a locale's translation on the phrase.
    async fn translate(
        &self,
        id: LanguagePhraseId,
        locale: LocaleCode,
        translation: Translation,
    ) -> StorageResult<()>;
}

fn _assert_language_phrase_object_safe() {
    fn _f(_: Box<dyn LanguagePhraseRepository>) {}
}

// === LanguagePhraseRepository section end ===

// =============================================================================
// === BaseGroupRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`BaseGroup`](crate::aggregate::BaseGroup).
#[async_trait]
pub trait BaseGroupRepository: Send + Sync {
    /// Fetch a base group by id.
    async fn get(&self, id: BaseGroupId) -> StorageResult<Option<BaseGroup>>;
    /// List base groups for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<BaseGroup>>;
    /// Insert a new base group.
    async fn insert(&self, g: &BaseGroup) -> StorageResult<()>;
    /// Update a base group.
    async fn update(&self, g: &BaseGroup) -> StorageResult<()>;
    /// Soft-delete a base group.
    async fn delete(&self, id: BaseGroupId) -> StorageResult<()>;
    /// Returns true if the (school_id, name) pair is unique.
    async fn unique_name_in_group(&self, school: SchoolId, name: &str) -> StorageResult<bool>;
    /// Returns the number of `BaseSetup` rows referencing this group.
    async fn referencing_setups(&self, id: BaseGroupId) -> StorageResult<u64>;
}

fn _assert_base_group_object_safe() {
    fn _f(_: Box<dyn BaseGroupRepository>) {}
}

// === BaseGroupRepository section end ===

// =============================================================================
// === BaseSetupRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`BaseSetup`](crate::aggregate::BaseSetup).
#[async_trait]
pub trait BaseSetupRepository: Send + Sync {
    /// Fetch a base setup by id.
    async fn get(&self, id: BaseSetupId) -> StorageResult<Option<BaseSetup>>;
    /// List base setups for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<BaseSetup>>;
    /// List base setups in a group.
    async fn list_for_group(&self, group: BaseGroupId) -> StorageResult<Vec<BaseSetup>>;
    /// Insert a new base setup.
    async fn insert(&self, s: &BaseSetup) -> StorageResult<()>;
    /// Update a base setup.
    async fn update(&self, s: &BaseSetup) -> StorageResult<()>;
    /// Soft-delete a base setup.
    async fn delete(&self, id: BaseSetupId) -> StorageResult<()>;
    /// Returns the number of rows in the school that reference the
    /// given base setup.
    async fn usage_count(&self, id: BaseSetupId) -> StorageResult<u64>;
}

fn _assert_base_setup_object_safe() {
    fn _f(_: Box<dyn BaseSetupRepository>) {}
}

// === BaseSetupRepository section end ===

// =============================================================================
// === DateFormatRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`DateFormat`](crate::aggregate::DateFormat).
#[async_trait]
pub trait DateFormatRepository: Send + Sync {
    /// Fetch a date format by id.
    async fn get(&self, id: DateFormatId) -> StorageResult<Option<DateFormat>>;
    /// List date formats for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<DateFormat>>;
    /// List active date formats.
    async fn list_active(&self, school: SchoolId) -> StorageResult<Vec<DateFormat>>;
    /// Insert a date format.
    async fn insert(&self, f: &DateFormat) -> StorageResult<()>;
    /// Update a date format.
    async fn update(&self, f: &DateFormat) -> StorageResult<()>;
    /// Soft-delete a date format.
    async fn delete(&self, id: DateFormatId) -> StorageResult<()>;
    /// Returns the number of `GeneralSettings` rows referencing the
    /// given date format.
    async fn referencing_settings(&self, id: DateFormatId) -> StorageResult<u64>;
}

fn _assert_date_format_object_safe() {
    fn _f(_: Box<dyn DateFormatRepository>) {}
}

// === DateFormatRepository section end ===

// =============================================================================
// === StyleRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`Style`](crate::aggregate::Style).
#[async_trait]
pub trait StyleRepository: Send + Sync {
    /// Fetch a style by id.
    async fn get(&self, id: StyleId) -> StorageResult<Option<Style>>;
    /// List styles for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Style>>;
    /// List active styles.
    async fn list_active(&self, school: SchoolId) -> StorageResult<Vec<Style>>;
    /// Insert a style.
    async fn insert(&self, s: &Style) -> StorageResult<()>;
    /// Update a style.
    async fn update(&self, s: &Style) -> StorageResult<()>;
    /// Soft-delete a style.
    async fn delete(&self, id: StyleId) -> StorageResult<()>;
    /// Returns the number of users referencing the given style.
    async fn user_count(&self, id: StyleId) -> StorageResult<u64>;
}

fn _assert_style_object_safe() {
    fn _f(_: Box<dyn StyleRepository>) {}
}

// === StyleRepository section end ===

// =============================================================================
// === BackgroundSettingRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`BackgroundSetting`](crate::aggregate::BackgroundSetting).
#[async_trait]
pub trait BackgroundSettingRepository: Send + Sync {
    /// Fetch a background by id.
    async fn get(&self, id: BackgroundSettingId) -> StorageResult<Option<BackgroundSetting>>;
    /// List backgrounds for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<BackgroundSetting>>;
    /// List default backgrounds for a school.
    async fn list_default(&self, school: SchoolId) -> StorageResult<Vec<BackgroundSetting>>;
    /// Insert a background.
    async fn insert(&self, b: &BackgroundSetting) -> StorageResult<()>;
    /// Update a background.
    async fn update(&self, b: &BackgroundSetting) -> StorageResult<()>;
    /// Soft-delete a background.
    async fn delete(&self, id: BackgroundSettingId) -> StorageResult<()>;
}

fn _assert_background_setting_object_safe() {
    fn _f(_: Box<dyn BackgroundSettingRepository>) {}
}

// === BackgroundSettingRepository section end ===

// =============================================================================
// === DashboardSettingRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`DashboardSetting`](crate::aggregate::DashboardSetting).
#[async_trait]
pub trait DashboardSettingRepository: Send + Sync {
    /// Fetch a dashboard setting by id.
    async fn get(&self, id: DashboardSettingId) -> StorageResult<Option<DashboardSetting>>;
    /// List dashboard settings for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<DashboardSetting>>;
    /// List dashboard settings for a role.
    async fn list_for_role(
        &self,
        role: educore_rbac::ids::RoleId,
    ) -> StorageResult<Vec<DashboardSetting>>;
    /// Insert a dashboard setting.
    async fn insert(&self, d: &DashboardSetting) -> StorageResult<()>;
    /// Update a dashboard setting.
    async fn update(&self, d: &DashboardSetting) -> StorageResult<()>;
    /// Soft-delete a dashboard setting.
    async fn delete(&self, id: DashboardSettingId) -> StorageResult<()>;
    /// Returns the number of roles referencing the section.
    async fn role_count(&self, dashboard_sec_id: DashboardSectionId) -> StorageResult<u64>;
}

fn _assert_dashboard_setting_object_safe() {
    fn _f(_: Box<dyn DashboardSettingRepository>) {}
}

// === DashboardSettingRepository section end ===

// =============================================================================
// === CustomLinkRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`CustomLink`](crate::aggregate::CustomLink).
#[async_trait]
pub trait CustomLinkRepository: Send + Sync {
    /// Fetch the per-school `CustomLink` bundle.
    async fn get(&self, school: SchoolId) -> StorageResult<Option<CustomLink>>;
    /// Insert a new bundle.
    async fn insert(&self, c: &CustomLink) -> StorageResult<()>;
    /// Update the bundle.
    async fn update(&self, c: &CustomLink) -> StorageResult<()>;
    /// Reset the bundle (clear all links and social URLs).
    async fn reset(&self, school: SchoolId) -> StorageResult<()>;
}

fn _assert_custom_link_object_safe() {
    fn _f(_: Box<dyn CustomLinkRepository>) {}
}

// === CustomLinkRepository section end ===

// =============================================================================
// === ThemeRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`Theme`](crate::aggregate::Theme).
#[async_trait]
pub trait ThemeRepository: Send + Sync {
    /// Fetch a theme by id.
    async fn get(&self, id: ThemeId) -> StorageResult<Option<Theme>>;
    /// List themes for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<Theme>>;
    /// List default themes.
    async fn list_default(&self, school: SchoolId) -> StorageResult<Vec<Theme>>;
    /// List system themes.
    async fn list_system(&self, school: SchoolId) -> StorageResult<Vec<Theme>>;
    /// Insert a theme.
    async fn insert(&self, t: &Theme) -> StorageResult<()>;
    /// Update a theme.
    async fn update(&self, t: &Theme) -> StorageResult<()>;
    /// Soft-delete a theme.
    async fn delete(&self, id: ThemeId) -> StorageResult<()>;
}

fn _assert_theme_object_safe() {
    fn _f(_: Box<dyn ThemeRepository>) {}
}

// === ThemeRepository section end ===

// =============================================================================
// === ColorRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`Color`](crate::aggregate::Color). Global
/// (no `school_id`).
#[async_trait]
pub trait ColorRepository: Send + Sync {
    /// Fetch a color by id.
    async fn get(&self, id: ColorId) -> StorageResult<Option<Color>>;
    /// List all colors.
    async fn list(&self) -> StorageResult<Vec<Color>>;
    /// Insert a color.
    async fn insert(&self, c: &Color) -> StorageResult<()>;
    /// Update a color.
    async fn update(&self, c: &Color) -> StorageResult<()>;
    /// Soft-delete a color.
    async fn delete(&self, id: ColorId) -> StorageResult<()>;
    /// Returns the number of `ColorTheme` rows referencing the color.
    async fn theme_binding_count(&self, id: ColorId) -> StorageResult<u64>;
}

fn _assert_color_object_safe() {
    fn _f(_: Box<dyn ColorRepository>) {}
}

// === ColorRepository section end ===

// =============================================================================
// === ColorThemeRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`ColorTheme`](crate::aggregate::ColorTheme). Global.
#[async_trait]
pub trait ColorThemeRepository: Send + Sync {
    /// Fetch a color-theme row by id.
    async fn get(&self, id: ColorThemeId) -> StorageResult<Option<ColorTheme>>;
    /// List color-theme rows for a theme.
    async fn list_for_theme(&self, theme: ThemeId) -> StorageResult<Vec<ColorTheme>>;
    /// List color-theme rows for a color.
    async fn list_for_color(&self, color: ColorId) -> StorageResult<Vec<ColorTheme>>;
    /// Insert a color-theme row.
    async fn insert(&self, ct: &ColorTheme) -> StorageResult<()>;
    /// Update a color-theme row.
    async fn update(&self, ct: &ColorTheme) -> StorageResult<()>;
    /// Soft-delete a color-theme row.
    async fn delete(&self, id: ColorThemeId) -> StorageResult<()>;
    /// Delete every color-theme row for a theme. Returns the count deleted.
    async fn delete_for_theme(&self, theme: ThemeId) -> StorageResult<u64>;
    /// Copy every color-theme row from `source` to `target`. Returns the count copied.
    async fn copy_for_theme(&self, source: ThemeId, target: ThemeId) -> StorageResult<u32>;
}

fn _assert_color_theme_object_safe() {
    fn _f(_: Box<dyn ColorThemeRepository>) {}
}

// === ColorThemeRepository section end ===

// =============================================================================
// === BehaviorRecordSettingRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`BehaviorRecordSetting`](crate::aggregate::BehaviorRecordSetting).
#[async_trait]
pub trait BehaviorRecordSettingRepository: Send + Sync {
    /// Fetch the per-school `BehaviorRecordSetting` singleton.
    async fn get(&self, school: SchoolId) -> StorageResult<Option<BehaviorRecordSetting>>;
    /// Insert the singleton.
    async fn insert(&self, s: &BehaviorRecordSetting) -> StorageResult<()>;
    /// Update the singleton.
    async fn update(&self, s: &BehaviorRecordSetting) -> StorageResult<()>;
}

fn _assert_behavior_record_setting_object_safe() {
    fn _f(_: Box<dyn BehaviorRecordSettingRepository>) {}
}

// === BehaviorRecordSettingRepository section end ===

// =============================================================================
// === SetupAdminRepository section begin (owner: A) ===
// =============================================================================

/// Repository port for [`SetupAdmin`](crate::aggregate::SetupAdmin).
#[async_trait]
pub trait SetupAdminRepository: Send + Sync {
    /// Fetch by id.
    async fn get(&self, id: SetupAdminId) -> StorageResult<Option<SetupAdmin>>;
    /// List for a school.
    async fn list(&self, school: SchoolId) -> StorageResult<Vec<SetupAdmin>>;
    /// List by admin type.
    async fn list_by_type(
        &self,
        school: SchoolId,
        admin_type: SetupAdminType,
    ) -> StorageResult<Vec<SetupAdmin>>;
    /// Insert a new entry.
    async fn insert(&self, s: &SetupAdmin) -> StorageResult<()>;
    /// Update an entry.
    async fn update(&self, s: &SetupAdmin) -> StorageResult<()>;
    /// Soft-delete an entry.
    async fn delete(&self, id: SetupAdminId) -> StorageResult<()>;
    /// Returns the number of rows in the school referencing this entry.
    async fn usage_count(&self, id: SetupAdminId) -> StorageResult<u64>;
}

fn _assert_setup_admin_object_safe() {
    fn _f(_: Box<dyn SetupAdminRepository>) {}
}

// === SetupAdminRepository section end ===
