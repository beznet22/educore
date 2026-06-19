//! # educore-settings value objects
//!
//! Typed ids, value objects, and closed enums per
//! `docs/specs/settings/value-objects.md`.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
pub use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed settings id
// =============================================================================

macro_rules! settings_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        $vis struct $name {
            pub school_id: SchoolId,
            pub value: Uuid,
        }

        impl $name {
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed root ids (15)
// =============================================================================

settings_typed_id! { pub struct GeneralSettingsId; }
settings_typed_id! { pub struct LanguageId; }
settings_typed_id! { pub struct LanguagePhraseId; }
settings_typed_id! { pub struct BaseSetupId; }
settings_typed_id! { pub struct BaseGroupId; }
settings_typed_id! { pub struct DateFormatId; }
settings_typed_id! { pub struct StyleId; }
settings_typed_id! { pub struct BackgroundSettingId; }
settings_typed_id! { pub struct DashboardSettingId; }
settings_typed_id! { pub struct CustomLinkId; }
settings_typed_id! { pub struct ColorThemeId; }
settings_typed_id! { pub struct ThemeId; }
settings_typed_id! { pub struct ColorId; }
settings_typed_id! { pub struct BehaviorRecordSettingId; }
settings_typed_id! { pub struct SetupAdminId; }

// =============================================================================
// AcademicYearRef
// =============================================================================

/// A typed reference to an `AcademicYear` aggregate in the
/// `educore-academic` domain. We deliberately do **not** depend on
/// `educore-academic` here; the spec treats the id as opaque from
/// the settings domain's point of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcademicYearRef {
    /// The owning school.
    pub school_id: SchoolId,
    /// The local id.
    pub value: Uuid,
}

impl AcademicYearRef {
    /// Constructs a new `AcademicYearRef`.
    #[must_use]
    pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
        Self { school_id, value }
    }

    /// Returns the local UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.value
    }
}

impl fmt::Display for AcademicYearRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.school_id, self.value)
    }
}

// =============================================================================
// Closed enums
// =============================================================================

/// Currency display format. Per
/// `docs/specs/settings/value-objects.md` (CurrencyFormat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyFormat {
    /// `$100.00` (symbol before amount).
    SymbolAmount,
    /// `100.00$` (amount before symbol).
    AmountSymbol,
}

impl CurrencyFormat {
    /// Returns the canonical wire string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SymbolAmount => "SymbolAmount",
            Self::AmountSymbol => "AmountSymbol",
        }
    }
}

impl fmt::Display for CurrencyFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Language activation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageStatus {
    /// Language is in use.
    Active,
    /// Language is retired.
    Inactive,
}

impl LanguageStatus {
    /// Returns the canonical wire string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
        }
    }
}

impl fmt::Display for LanguageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Color mode for a `Theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorMode {
    /// A gradient fill.
    Gradient,
    /// A solid fill.
    Solid,
}

impl ColorMode {
    /// Returns the canonical wire string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Gradient => "Gradient",
            Self::Solid => "Solid",
        }
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a `Theme` renders with a CSS box-shadow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoxShadow(pub bool);

impl BoxShadow {
    /// Constructs a new `BoxShadow`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn as_bool(&self) -> bool {
        self.0
    }
}

/// Whether a `BackgroundSetting` is image-typed or color-typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackgroundType {
    /// Image-backed background.
    Image,
    /// Color-backed background.
    Color,
}

impl BackgroundType {
    /// Returns the canonical wire string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Image => "Image",
            Self::Color => "Color",
        }
    }
}

impl fmt::Display for BackgroundType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether a `Color` row is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorStatus(pub bool);

impl ColorStatus {
    /// Constructs a new `ColorStatus`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn as_bool(&self) -> bool {
        self.0
    }
}

/// A `BehaviorRecord` feature flag. Per
/// `docs/specs/settings/value-objects.md` (BehaviorFlag): 0 = off,
/// 1 = on, 2 = inherited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BehaviorFlag(pub i32);

impl BehaviorFlag {
    /// Constructs a new `BehaviorFlag`.
    pub fn new(v: i32) -> Result<Self> {
        if !(0..=2).contains(&v) {
            return Err(DomainError::Validation(format!(
                "behavior_flag must be 0..=2, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
    /// Returns true if this flag is "on".
    #[must_use]
    pub const fn is_on(self) -> bool {
        self.0 == 1
    }
}

/// `SetupAdmin` aggregate discriminator. 1 = purpose, 2 = complaint
/// type, 3 = source, 4 = reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetupAdminType(pub i32);

impl SetupAdminType {
    /// Constructs a new `SetupAdminType`.
    pub fn new(v: i32) -> Result<Self> {
        if !(1..=4).contains(&v) {
            return Err(DomainError::Validation(format!(
                "setup_admin_type must be 1..=4, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
    /// Returns the canonical name.
    #[must_use]
    pub const fn as_name(self) -> &'static str {
        match self.0 {
            1 => "Purpose",
            2 => "ComplaintType",
            3 => "Source",
            4 => "Reference",
            _ => "Unknown",
        }
    }
}

/// A boolean feature-flag toggle in the module-toggle map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleToggle {
    /// The toggle name (e.g. `lesson_enabled`).
    pub name: String,
    /// Whether the toggle is on.
    pub enabled: bool,
}

/// Preloader style id. Per spec: an `i32` style id chosen by the
/// consumer (the engine stores it opaquely).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreloaderStyle(pub i32);

impl PreloaderStyle {
    /// Constructs a new `PreloaderStyle`.
    #[must_use]
    pub const fn new(v: i32) -> Self {
        Self(v)
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Preloader type. 1 = spinner, 2 = image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PreloaderType(pub i32);

impl PreloaderType {
    /// Constructs a new `PreloaderType`.
    pub fn new(v: i32) -> Result<Self> {
        if !(1..=2).contains(&v) {
            return Err(DomainError::Validation(format!(
                "preloader_type must be 1..=2, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Whether phone numbers are masked or visible. 1 = masked, 2 = visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhoneNumberPrivacy(pub i32);

impl PhoneNumberPrivacy {
    /// Constructs a new `PhoneNumberPrivacy`.
    pub fn new(v: i32) -> Result<Self> {
        if !(1..=2).contains(&v) {
            return Err(DomainError::Validation(format!(
                "phone_number_privacy must be 1..=2, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns true if masked.
    #[must_use]
    pub const fn is_masked(self) -> bool {
        self.0 == 1
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

/// Text direction. 1 = RTL, 2 = LTR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RtlLtl(pub i32);

impl RtlLtl {
    /// Constructs a new `RtlLtl`.
    pub fn new(v: i32) -> Result<Self> {
        if !(1..=2).contains(&v) {
            return Err(DomainError::Validation(format!(
                "rtl_ltl must be 1..=2, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns true if right-to-left.
    #[must_use]
    pub const fn is_rtl(self) -> bool {
        self.0 == 1
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

// =============================================================================
// Type-safe wrappers (validated at construction)
// =============================================================================

/// Validated hex color string. Accepts `#RGB`, `#RRGGBB`, or `#RRGGBBAA`
/// plus the CSS named color keywords (e.g. `red`). Per
/// `docs/specs/settings/value-objects.md` (ColorHex).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorHex(pub String);

impl ColorHex {
    /// Constructs a new `ColorHex`, validating the format.
    pub fn new(s: &str) -> Result<Self> {
        if !Self::is_valid(s) {
            return Err(DomainError::Validation(format!("invalid color hex: {s:?}")));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns true if `s` is a valid hex color (or named keyword).
    #[must_use]
    pub fn is_valid(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.len() > 32 {
            return false;
        }
        if let Some(hex) = trimmed.strip_prefix('#') {
            (hex.len() == 3 || hex.len() == 6 || hex.len() == 8)
                && hex.chars().all(|c| c.is_ascii_hexdigit())
        } else {
            Self::is_named_color(trimmed)
        }
    }

    fn is_named_color(s: &str) -> bool {
        const NAMED: &[&str] = &[
            "aliceblue",
            "antiquewhite",
            "aqua",
            "aquamarine",
            "azure",
            "beige",
            "bisque",
            "black",
            "blanchedalmond",
            "blue",
            "blueviolet",
            "brown",
            "burlywood",
            "cadetblue",
            "chartreuse",
            "chocolate",
            "coral",
            "cornflowerblue",
            "cornsilk",
            "crimson",
            "cyan",
            "darkblue",
            "darkcyan",
            "darkgoldenrod",
            "darkgray",
            "darkgreen",
            "darkgrey",
            "darkkhaki",
            "darkmagenta",
            "darkolivegreen",
            "darkorange",
            "darkorchid",
            "darkred",
            "darksalmon",
            "darkseagreen",
            "darkslateblue",
            "darkslategray",
            "darkslategrey",
            "darkturquoise",
            "darkviolet",
            "deeppink",
            "deepskyblue",
            "dimgray",
            "dimgrey",
            "dodgerblue",
            "firebrick",
            "floralwhite",
            "forestgreen",
            "fuchsia",
            "gainsboro",
            "ghostwhite",
            "gold",
            "goldenrod",
            "gray",
            "green",
            "greenyellow",
            "grey",
            "honeydew",
            "hotpink",
            "indianred",
            "indigo",
            "ivory",
            "khaki",
            "lavender",
            "lavenderblush",
            "lawngreen",
            "lemonchiffon",
            "lightblue",
            "lightcoral",
            "lightcyan",
            "lightgoldenrodyellow",
            "lightgray",
            "lightgreen",
            "lightgrey",
            "lightpink",
            "lightsalmon",
            "lightseagreen",
            "lightskyblue",
            "lightslategray",
            "lightslategrey",
            "lightsteelblue",
            "lightyellow",
            "lime",
            "limegreen",
            "linen",
            "magenta",
            "maroon",
            "mediumaquamarine",
            "mediumblue",
            "mediumorchid",
            "mediumpurple",
            "mediumseagreen",
            "mediumslateblue",
            "mediumspringgreen",
            "mediumturquoise",
            "mediumvioletred",
            "midnightblue",
            "mintcream",
            "mistyrose",
            "moccasin",
            "navajowhite",
            "navy",
            "oldlace",
            "olive",
            "olivedrab",
            "orange",
            "orangered",
            "orchid",
            "palegoldenrod",
            "palegreen",
            "paleturquoise",
            "palevioletred",
            "papayawhip",
            "peachpuff",
            "peru",
            "pink",
            "plum",
            "powderblue",
            "purple",
            "rebeccapurple",
            "red",
            "rosybrown",
            "royalblue",
            "saddlebrown",
            "salmon",
            "sandybrown",
            "seagreen",
            "seashell",
            "sienna",
            "silver",
            "skyblue",
            "slateblue",
            "slategray",
            "slategrey",
            "snow",
            "springgreen",
            "steelblue",
            "tan",
            "teal",
            "thistle",
            "tomato",
            "turquoise",
            "violet",
            "wheat",
            "white",
            "whitesmoke",
            "yellow",
            "yellowgreen",
        ];
        NAMED.iter().any(|n| n.eq_ignore_ascii_case(s))
    }

    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validated URL or empty string. Per `docs/specs/settings/value-objects.md`
/// (LinkHref).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkHref(pub String);

impl LinkHref {
    /// Constructs a new `LinkHref`. Empty strings are allowed (the
    /// "href not set" case); non-empty values must look like a URL.
    pub fn new(s: &str) -> Result<Self> {
        if !Self::is_url(s) {
            return Err(DomainError::Validation(format!("invalid link href: {s:?}")));
        }
        Ok(Self(s.to_owned()))
    }

    /// Returns true if `s` is either empty or a URL.
    #[must_use]
    pub fn is_url(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return true;
        }
        if trimmed.len() > 2048 {
            return false;
        }
        trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
            || trimmed.starts_with("mailto:")
            || trimmed.starts_with("tel:")
            || trimmed.starts_with("/")
    }

    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true if the href is non-empty.
    #[must_use]
    pub fn is_set(&self) -> bool {
        !self.0.trim().is_empty()
    }
}

/// Validated social URL or empty string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SocialUrl(pub String);

impl SocialUrl {
    /// Constructs a new `SocialUrl`.
    pub fn new(s: &str) -> Result<Self> {
        if !LinkHref::is_url(s) {
            return Err(DomainError::Validation(format!(
                "invalid social url: {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The string identifier of the email port adapter (e.g. `smtp`,
/// `mailgun`, `ses`). Opaque to the settings domain; validated only
/// as non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EmailDriver(pub String);

impl EmailDriver {
    /// Constructs a new `EmailDriver`.
    pub fn new(s: &str) -> Result<Self> {
        if s.trim().is_empty() || s.len() > 64 {
            return Err(DomainError::Validation(format!(
                "email_driver must be 1..64 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The string identifier of the queue port adapter (e.g. `sync`,
/// `redis`, `sqs`). Opaque to the settings domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QueueConnection(pub String);

impl QueueConnection {
    /// Constructs a new `QueueConnection`.
    pub fn new(s: &str) -> Result<Self> {
        if s.trim().is_empty() || s.len() > 64 {
            return Err(DomainError::Validation(format!(
                "queue_connection must be 1..64 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated `strftime` pattern. The engine accepts any
/// `%-encoded` strftime string; the spec lists the common
/// conversions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DateFormatPattern(pub String);

impl DateFormatPattern {
    /// Constructs a new `DateFormatPattern`.
    pub fn new(s: &str) -> Result<Self> {
        if !Self::is_strftime_valid(s) {
            return Err(DomainError::Validation(format!(
                "invalid strftime pattern: {s:?}"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns true if `s` is a valid strftime pattern.
    #[must_use]
    pub fn is_strftime_valid(s: &str) -> bool {
        if s.is_empty() || s.len() > 64 {
            return false;
        }
        s.bytes().any(|b| b == b'%') && s.chars().all(|c| c.is_ascii_graphic() || c == ' ')
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Human-readable example of a date format (e.g. `YYYY-MM-DD`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DateFormatPreview(pub String);

impl DateFormatPreview {
    /// Constructs a new `DateFormatPreview`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 64 {
            return Err(DomainError::Validation(format!(
                "date_format_preview must be 1..64 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// CSS file path for a `Style`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StylePath(pub String);

impl StylePath {
    /// Constructs a new `StylePath`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "style_path must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Display name of a `Style`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StyleName(pub String);

impl StyleName {
    /// Constructs a new `StyleName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "style_name must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Display title of a `Theme`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThemeTitle(pub String);

impl ThemeTitle {
    /// Constructs a new `ThemeTitle`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "theme_title must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// CSS file path for a `Theme`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThemePath(pub String);

impl ThemePath {
    /// Constructs a new `ThemePath`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "theme_path must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Title of a `BackgroundSetting`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackgroundTitle(pub String);

impl BackgroundTitle {
    /// Constructs a new `BackgroundTitle`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "background_title must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Color value for a `BackgroundSetting` (when type=Color).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackgroundColor(pub ColorHex);

impl BackgroundColor {
    /// Constructs a new `BackgroundColor`.
    pub fn new(s: &str) -> Result<Self> {
        Ok(Self(ColorHex::new(s)?))
    }
    /// Returns the underlying hex.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Image file reference for a `BackgroundSetting` (when type=Image).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BackgroundImage(pub FileReference);

impl BackgroundImage {
    /// Constructs a new `BackgroundImage`.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(FileReference(name.into()))
    }
}

/// A general-purpose file reference (logo, favicon, attachment).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileReference(pub String);

impl FileReference {
    /// Constructs a new `FileReference`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() || s.len() > 1024 {
            return Err(DomainError::Validation(format!(
                "file_reference must be 1..1024 chars"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A general-purpose label for a `CustomLink` entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkLabel(pub String);

impl LinkLabel {
    /// Constructs a new `LinkLabel`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "link_label must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The name of a `SetupAdmin` entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetupAdminName(pub String);

impl SetupAdminName {
    /// Constructs a new `SetupAdminName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "setup_admin_name must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The description of a `SetupAdmin` entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetupAdminDescription(pub String);

impl SetupAdminDescription {
    /// Constructs a new `SetupAdminDescription`.
    pub fn new(s: &str) -> Result<Self> {
        if s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "setup_admin_description must be <= 65000 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Language / Phrase value objects
// =============================================================================

/// ISO 639-1 language code (2 chars). Stored as a 2..191 char string
/// so consumer-defined extensions (e.g. `en-US`) are accepted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageCode(pub String);

impl LanguageCode {
    /// Constructs a new `LanguageCode`.
    pub fn new(s: &str) -> Result<Self> {
        if s.len() < 2 || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "language_code must be 2..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Display name of a language (e.g. `English`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageName(pub String);

impl LanguageName {
    /// Constructs a new `LanguageName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "language_name must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Native-script name of a language (e.g. `Español`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageNative(pub String);

impl LanguageNative {
    /// Constructs a new `LanguageNative`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "language_native must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Universal/translated name of a language.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageUniversal(pub String);

impl LanguageUniversal {
    /// Constructs a new `LanguageUniversal`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "language_universal must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Right-to-left flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RtlFlag(pub bool);

impl RtlFlag {
    /// Constructs a new `RtlFlag`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn as_bool(&self) -> bool {
        self.0
    }
}

/// The module a phrase belongs to (e.g. `dashboard`, `fees`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhraseModule(pub String);

impl PhraseModule {
    /// Constructs a new `PhraseModule`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "phrase_module must be 1..65000 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The source-of-truth translation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DefaultPhrase(pub String);

impl DefaultPhrase {
    /// Constructs a new `DefaultPhrase`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "default_phrase must be 1..65000 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A per-locale translation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Translation(pub String);

impl Translation {
    /// Constructs a new `Translation`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 65000 {
            return Err(DomainError::Validation(format!(
                "translation must be 1..65000 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A locale code (ISO 639-1, e.g. `en`, `es`, `bn`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LocaleCode(pub String);

impl LocaleCode {
    /// Constructs a new `LocaleCode`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 16 {
            return Err(DomainError::Validation(format!(
                "locale_code must be 1..16 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =============================================================================
// Base setup / group
// =============================================================================

/// Display name of a `BaseGroup`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaseGroupName(pub String);

impl BaseGroupName {
    /// Constructs a new `BaseGroupName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 200 {
            return Err(DomainError::Validation(format!(
                "base_group_name must be 1..200 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Display name of a `BaseSetup`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaseSetupName(pub String);

impl BaseSetupName {
    /// Constructs a new `BaseSetupName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "base_setup_name must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Display order for a `BaseGroup`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BaseGroupOrder(pub i32);

impl BaseGroupOrder {
    /// Constructs a new `BaseGroupOrder`.
    #[must_use]
    pub const fn new(v: i32) -> Self {
        Self(v)
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

// =============================================================================
// Date format / Dashboard
// =============================================================================

/// Active flag for a `DateFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DateFormatActive(pub bool);

impl DateFormatActive {
    /// Constructs a new `DateFormatActive`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
}

/// A dashboard section id (i32).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DashboardSectionId(pub i32);

impl DashboardSectionId {
    /// Constructs a new `DashboardSectionId`.
    pub fn new(v: i32) -> Result<Self> {
        if v < 1 {
            return Err(DomainError::Validation(format!(
                "dashboard_section_id must be >= 1, got {v}"
            )));
        }
        Ok(Self(v))
    }
    /// Returns the inner integer.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

// =============================================================================
// Style / FontFamily / Color
// =============================================================================

/// A CSS font-family list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFamily(pub String);

impl FontFamily {
    /// Constructs a new `FontFamily`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 {
            return Err(DomainError::Validation(format!(
                "font_family must be 1..255 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a `Color` row is a color (vs. a swatch placeholder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IsColor(pub bool);

impl IsColor {
    /// Constructs a new `IsColor`.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
}

/// Display name of a `Color`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorName(pub String);

impl ColorName {
    /// Constructs a new `ColorName`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "color_name must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A hex color value (alias for [`ColorHex`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorValue(pub ColorHex);

impl ColorValue {
    /// Constructs a new `ColorValue`.
    pub fn new(s: &str) -> Result<Self> {
        Ok(Self(ColorHex::new(s)?))
    }
    /// Returns the underlying hex.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// The preview green of a `Color`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LawnGreen(pub ColorHex);

impl LawnGreen {
    /// Constructs a new `LawnGreen`.
    pub fn new(s: &str) -> Result<Self> {
        Ok(Self(ColorHex::new(s)?))
    }
    /// Returns the underlying hex.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Currency format alias — kept separate from [`CurrencyFormat`]
/// to follow the spec's "ColorFormat" naming without conflict.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorFormat(pub String);

impl ColorFormat {
    /// Constructs a new `ColorFormat`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 32 {
            return Err(DomainError::Validation(format!(
                "color_format must be 1..32 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
}

/// The active theme name (string; opaque to the settings domain).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActiveTheme(pub String);

impl ActiveTheme {
    /// Constructs a new `ActiveTheme`.
    pub fn new(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 191 {
            return Err(DomainError::Validation(format!(
                "active_theme must be 1..191 chars"
            )));
        }
        Ok(Self(s.to_owned()))
    }
    /// Returns the underlying string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A `ModuleTogglePatch` — a partial update to the module-toggle map.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleTogglePatch {
    /// Per-toggle optional values.
    pub toggles: BTreeMap<String, Option<bool>>,
}

impl ModuleTogglePatch {
    /// Constructs a new empty `ModuleTogglePatch`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a toggle update.
    #[must_use]
    pub fn with(mut self, name: impl Into<String>, value: Option<bool>) -> Self {
        self.toggles.insert(name.into(), value);
        self
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::ids::Identifier;

    #[test]
    fn typed_ids_smoke_test() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = GeneralSettingsId::new(school, Uuid::nil());
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::nil());
    }

    #[test]
    fn color_hex_validates_hex() {
        assert!(ColorHex::is_valid("#fff"));
        assert!(ColorHex::is_valid("#FF0000"));
        assert!(ColorHex::is_valid("#ff0000aa"));
        assert!(!ColorHex::is_valid("#zz"));
        assert!(!ColorHex::is_valid("#ff"));
        assert!(!ColorHex::is_valid(""));
    }

    #[test]
    fn color_hex_validates_named() {
        assert!(ColorHex::is_valid("red"));
        assert!(ColorHex::is_valid("blue"));
        assert!(!ColorHex::is_valid("zzz"));
    }

    #[test]
    fn color_hex_constructor_returns_err_on_invalid() {
        assert!(ColorHex::new("#zz").is_err());
        assert!(ColorHex::new("red").is_ok());
    }

    #[test]
    fn link_href_accepts_empty_and_urls() {
        assert!(LinkHref::new("").is_ok());
        assert!(LinkHref::new("https://example.com").is_ok());
        assert!(LinkHref::new("/relative").is_ok());
        assert!(LinkHref::new("mailto:hi@example.com").is_ok());
        assert!(LinkHref::new("not-a-url").is_err());
    }

    #[test]
    fn strftime_validates() {
        assert!(DateFormatPattern::is_strftime_valid("%Y-%m-%d"));
        assert!(DateFormatPattern::is_strftime_valid("%d/%m/%Y %H:%M"));
        assert!(!DateFormatPattern::is_strftime_valid("plain"));
        assert!(!DateFormatPattern::is_strftime_valid(""));
    }

    #[test]
    fn behavior_flag_constrains_to_0_1_2() {
        assert!(BehaviorFlag::new(0).is_ok());
        assert!(BehaviorFlag::new(1).is_ok());
        assert!(BehaviorFlag::new(2).is_ok());
        assert!(BehaviorFlag::new(3).is_err());
        assert!(BehaviorFlag::new(-1).is_err());
        assert!(BehaviorFlag::new(1).unwrap().is_on());
    }

    #[test]
    fn setup_admin_type_constrains_to_1_4() {
        assert!(SetupAdminType::new(1).is_ok());
        assert!(SetupAdminType::new(4).is_ok());
        assert!(SetupAdminType::new(0).is_err());
        assert!(SetupAdminType::new(5).is_err());
        assert_eq!(SetupAdminType::new(1).unwrap().as_name(), "Purpose");
        assert_eq!(SetupAdminType::new(2).unwrap().as_name(), "ComplaintType");
        assert_eq!(SetupAdminType::new(3).unwrap().as_name(), "Source");
        assert_eq!(SetupAdminType::new(4).unwrap().as_name(), "Reference");
    }

    #[test]
    fn preloader_type_constrains_to_1_2() {
        assert!(PreloaderType::new(1).is_ok());
        assert!(PreloaderType::new(2).is_ok());
        assert!(PreloaderType::new(3).is_err());
    }

    #[test]
    fn rtl_ltl_direction() {
        assert!(RtlLtl::new(1).unwrap().is_rtl());
        assert!(!RtlLtl::new(2).unwrap().is_rtl());
        assert!(RtlLtl::new(3).is_err());
    }

    #[test]
    fn enums_display() {
        assert_eq!(CurrencyFormat::SymbolAmount.to_string(), "SymbolAmount");
        assert_eq!(CurrencyFormat::AmountSymbol.to_string(), "AmountSymbol");
        assert_eq!(LanguageStatus::Active.to_string(), "Active");
        assert_eq!(ColorMode::Solid.to_string(), "Solid");
        assert_eq!(BackgroundType::Image.to_string(), "Image");
    }

    #[test]
    fn module_toggle_patch_builder() {
        let patch = ModuleTogglePatch::new()
            .with("lesson_enabled", Some(true))
            .with("fees_enabled", Some(false))
            .with("chat_enabled", None);
        assert_eq!(patch.toggles.get("lesson_enabled").unwrap(), &Some(true));
        assert_eq!(patch.toggles.get("fees_enabled").unwrap(), &Some(false));
        assert_eq!(patch.toggles.get("chat_enabled").unwrap(), &None);
    }

    #[test]
    fn url_validation_includes_empty_for_optional_fields() {
        assert!(LinkHref::new("").unwrap().is_set() == false);
        assert!(LinkHref::new("https://x.com").unwrap().is_set());
    }
}
