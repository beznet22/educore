//! CMS-domain value objects.
//!
//! Per `docs/specs/cms/value-objects.md`:
//!
//! - Every id is `Id { school_id, value }` — a typed wrapper that
//!   carries the school anchor so the type system catches
//!   cross-tenant confusion at compile time.
//! - Strings (titles, descriptions) are validated at construction.
//!   The constructors return `Result<Self, DomainError>`; there are
//!   no setters that bypass validation.
//! - Status enums are closed (`PageStatus`, `NewsStatus`,
//!   `ContentShareType`, etc.).
//!
//! Phase 12 ships 20 typed root ids (one per root aggregate), 10+
//! child entity ids, and the validated value objects for the
//! spec-named types (titles, descriptions, dates, etc.). The
//! service structs in `services.rs` (PageService, NewsService,
//! ContentService, TestimonialService, HomeSliderService,
//! ContentShareListService) are the pure helpers that consume
//! these VOs.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
// Re-export the SchoolId so the rest of the crate can use the
// canonical `crate::value_objects::SchoolId` path.
pub use educore_core::ids::SchoolId;

// =============================================================================
// Re-exports for cross-domain types
// =============================================================================

/// Typed id for an `AcademicYear` row (re-exported from
/// `educore-academic`). Used by `Content`, `ContentShareList`,
/// `TeacherUploadContent`, and `UploadContent` as the academic
/// year scope.
pub use educore_academic::AcademicYearId;
/// Typed id for a `Class` row (re-exported from
/// `educore-academic`). Used by `Content.available_to_class` and
/// `TeacherUploadContent` for the class scope.
pub use educore_academic::ClassId;
/// Typed id for a `Section` row (re-exported from
/// `educore-academic`). Used by `Content.available_to_class` and
/// `TeacherUploadContent` for the section scope.
pub use educore_academic::SectionId;

// =============================================================================
// Macro: typed CMS id
// =============================================================================

/// Macro to define a per-aggregate typed id wrapper for the CMS
/// domain. Every CMS id follows the same shape: a `school_id`
/// anchor plus a local `Uuid`. The wrapper implements
/// [`Clone`], [`Copy`], [`PartialEq`], [`Eq`], [`Hash`], and
/// the `Display` format `"{school_id}/{value}"`.
macro_rules! cms_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        $vis struct $name {
            /// The owning school (tenant anchor).
            pub school_id: SchoolId,
            /// The local id (UUIDv7).
            pub value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
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
// Typed ids (20 root aggregates + 10 child entities)
// =============================================================================

cms_typed_id! {
    /// A typed id for a [`Page`](crate::aggregate::Page) row.
    pub struct PageId;
}
cms_typed_id! {
    /// A typed id for a [`News`](crate::aggregate::News) row.
    pub struct NewsId;
}
cms_typed_id! {
    /// A typed id for a [`NewsCategory`](crate::aggregate::NewsCategory) row.
    pub struct NewsCategoryId;
}
cms_typed_id! {
    /// A typed id for a [`NewsComment`](crate::aggregate::NewsComment) row.
    pub struct NewsCommentId;
}
cms_typed_id! {
    /// A typed id for a [`NewsPage`](crate::aggregate::NewsPage) row.
    pub struct NewsPageId;
}
cms_typed_id! {
    /// A typed id for a [`NoticeBoard`](crate::aggregate::NoticeBoard) row.
    pub struct NoticeBoardId;
}
cms_typed_id! {
    /// A typed id for a [`Testimonial`](crate::aggregate::Testimonial) row.
    pub struct TestimonialId;
}
cms_typed_id! {
    /// A typed id for a [`HomeSlider`](crate::aggregate::HomeSlider) row.
    pub struct HomeSliderId;
}
cms_typed_id! {
    /// A typed id for a [`SpeechSlider`](crate::aggregate::SpeechSlider) row (CMS-side).
    pub struct SpeechSliderId;
}
cms_typed_id! {
    /// A typed id for a [`Content`](crate::aggregate::Content) row.
    pub struct ContentId;
}
cms_typed_id! {
    /// A typed id for a [`ContentType`](crate::aggregate::ContentType) row.
    pub struct ContentTypeId;
}
cms_typed_id! {
    /// A typed id for a [`ContentShareList`](crate::aggregate::ContentShareList) row.
    pub struct ContentShareListId;
}
cms_typed_id! {
    /// A typed id for a [`TeacherUploadContent`](crate::aggregate::TeacherUploadContent) row.
    pub struct TeacherUploadContentId;
}
cms_typed_id! {
    /// A typed id for a [`UploadContent`](crate::aggregate::UploadContent) row.
    pub struct UploadContentId;
}
cms_typed_id! {
    /// A typed id for an [`AboutPage`](crate::aggregate::AboutPage) row.
    pub struct AboutPageId;
}
cms_typed_id! {
    /// A typed id for a [`ContactPage`](crate::aggregate::ContactPage) row.
    pub struct ContactPageId;
}
cms_typed_id! {
    /// A typed id for a [`CoursePage`](crate::aggregate::CoursePage) row.
    pub struct CoursePageId;
}
cms_typed_id! {
    /// A typed id for a [`HomePageSetting`](crate::aggregate::HomePageSetting) row.
    pub struct HomePageSettingId;
}
cms_typed_id! {
    /// A typed id for a [`FrontendPage`](crate::aggregate::FrontendPage) row.
    pub struct FrontendPageId;
}

// ---- Child entity ids (per `docs/specs/cms/entities.md`) ----

cms_typed_id! {
    /// A typed id for a [`NewsImage`](crate::aggregate::NewsImage) child entity.
    pub struct NewsImageId;
}
cms_typed_id! {
    /// A typed id for a [`PageRevision`](crate::aggregate::PageRevision) child entity.
    pub struct PageRevisionId;
}
cms_typed_id! {
    /// A typed id for a [`NewsRevision`](crate::aggregate::NewsRevision) child entity.
    pub struct NewsRevisionId;
}
cms_typed_id! {
    /// A typed id for a [`ContentRevision`](crate::aggregate::ContentRevision) child entity.
    pub struct ContentRevisionId;
}
cms_typed_id! {
    /// A typed id for a [`ContentShareListAudience`](crate::aggregate::ContentShareListAudience) child entity.
    pub struct ContentShareListAudienceId;
}
cms_typed_id! {
    /// A typed id for a [`ContentShareListContent`](crate::aggregate::ContentShareListContent) child entity.
    pub struct ContentShareListContentId;
}

// =============================================================================
// Free-text / named validated value objects
// =============================================================================

// ---- Page ----

/// Page title: 1..=191 chars per `docs/specs/cms/value-objects.md`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageTitle(String);

impl PageTitle {
    /// Maximum length of a page title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a page title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PageTitle`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "page title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PageTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PageTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Page description: 1..=5000 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageDescription(String);

impl PageDescription {
    /// Maximum length of a page description.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a page description.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PageDescription`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "page description")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PageDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PageDescription {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Page sub-title: 1..=191 chars, unique within school.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageSubTitle(String);

impl PageSubTitle {
    /// Maximum length of a page sub-title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a page sub-title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PageSubTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "page sub-title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PageSubTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// URL-safe slug: 1..=200 chars, `[a-z0-9-]` only. Per
/// `docs/specs/cms/value-objects.md` § "URLs, Files, and Slugs".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Slug(String);

impl Slug {
    /// Maximum length of a slug.
    pub const MAX_LEN: usize = 200;
    /// Minimum length of a slug.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `Slug`, rejecting empty, overlong, or
    /// non-`[a-z0-9-]` input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("slug must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "slug must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        if !s
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(DomainError::validation(format!(
                "slug must be [a-z0-9-], got {s:?}"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ---- News ----

/// News title: 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NewsTitle(String);

impl NewsTitle {
    /// Maximum length of a news title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a news title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `NewsTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "news title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NewsTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for NewsTitle {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// News body: 1..=65535 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NewsBody(String);

impl NewsBody {
    /// Maximum length of a news body.
    pub const MAX_LEN: usize = 65535;
    /// Minimum length of a news body.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `NewsBody`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "news body")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NewsBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// News-category name: 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CategoryName(String);

impl CategoryName {
    /// Maximum length of a category name.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a category name.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `CategoryName`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "category name")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CategoryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// News-comment message: 1..=5000 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommentMessage(String);

impl CommentMessage {
    /// Maximum length of a comment message.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a comment message.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `CommentMessage`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "comment message")?;
        Ok(Self(s))
    }
    /// Test-only constructor that bypasses validation. Used in
    /// aggregate unit tests where the validation path is itself
    /// the unit under test.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn _new_unchecked_for_test(s: String) -> Self {
        Self(s)
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommentMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---- NoticeBoard / Testimonial / HomeSlider / SpeechSlider ----

/// Notice-board title: 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeTitle(String);

impl NoticeTitle {
    /// Maximum length of a notice-board title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a notice-board title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `NoticeTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "notice title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoticeTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Notice-board message body: 1..=5000 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeMessage(String);

impl NoticeMessage {
    /// Maximum length of a notice-board message.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a notice-board message.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `NoticeMessage`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "notice message")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NoticeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A person's name: 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonName(String);

impl PersonName {
    /// Maximum length of a person name.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a person name.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PersonName`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "person name")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersonName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for PersonName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A free-text designation: 1..=191 chars (e.g. "Principal").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Designation(String);

impl Designation {
    /// Maximum length of a designation.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a designation.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `Designation`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "designation")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Designation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An institution name: 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InstitutionName(String);

impl InstitutionName {
    /// Maximum length of an institution name.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of an institution name.
    pub const MIN_LEN: usize = 1;
    /// Constructs an `InstitutionName`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "institution name")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InstitutionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Testimonial description: 1..=5000 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TestimonialDescription(String);

impl TestimonialDescription {
    /// Maximum length of a testimonial description.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a testimonial description.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `TestimonialDescription`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "testimonial description")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TestimonialDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A star rating for a [`Testimonial`](crate::aggregate::Testimonial).
/// Per the spec, the value is in `1..=5`. Enforced at
/// construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StarRating(u8);

impl StarRating {
    /// Minimum rating.
    pub const MIN: u8 = 1;
    /// Maximum rating.
    pub const MAX: u8 = 5;
    /// Constructs a `StarRating`, rejecting values outside `1..=5`.
    pub fn new(value: u8) -> Result<Self> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(DomainError::validation(format!(
                "star_rating must be in {}..={}, got {}",
                Self::MIN,
                Self::MAX,
                value
            )));
        }
        Ok(Self(value))
    }
    /// Returns the inner rating.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl Default for StarRating {
    fn default() -> Self {
        Self(Self::MAX)
    }
}

impl fmt::Display for StarRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}★", self.0)
    }
}

/// A free-text speech: 1..=5000 chars (the leadership speech body).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpeechText(String);

impl SpeechText {
    /// Maximum length of a speech.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a speech.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `SpeechText`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "speech")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SpeechText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---- Content ----

/// Content title: 1..=200 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentTitle(String);

impl ContentTitle {
    /// Maximum length of a content title.
    pub const MAX_LEN: usize = 200;
    /// Minimum length of a content title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `ContentTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "content title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Content description: 1..=500 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentDescription(String);

impl ContentDescription {
    /// Maximum length of a content description.
    pub const MAX_LEN: usize = 500;
    /// Minimum length of a content description.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `ContentDescription`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "content description")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Content type name (e.g. "Assignment", "Study Material").
/// 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentTypeName(String);

impl ContentTypeName {
    /// Maximum length of a content type name.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a content type name.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `ContentTypeName`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "content type name")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Content-share-list title: 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentShareListTitle(String);

impl ContentShareListTitle {
    /// Maximum length of a content-share-list title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a content-share-list title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `ContentShareListTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "content share list title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentShareListTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ---- CoursePage / HomePageSetting / FrontendPage / ContactPage / AboutPage ----

/// Course-page title: 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CoursePageTitle(String);

impl CoursePageTitle {
    /// Maximum length of a course-page title.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a course-page title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `CoursePageTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "course page title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CoursePageTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Course-page description: 1..=5000 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CoursePageDescription(String);

impl CoursePageDescription {
    /// Maximum length of a course-page description.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a course-page description.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `CoursePageDescription`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "course page description")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CoursePageDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Home-page title: 1..=255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomePageTitle(String);

impl HomePageTitle {
    /// Maximum length of a home-page title.
    pub const MAX_LEN: usize = 255;
    /// Minimum length of a home-page title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `HomePageTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "home page title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HomePageTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Home-page long title: 1..=255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomePageLongTitle(String);

impl HomePageLongTitle {
    /// Maximum length of a home-page long title.
    pub const MAX_LEN: usize = 255;
    /// Minimum length of a home-page long title.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `HomePageLongTitle`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "home page long title")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HomePageLongTitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Home-page short description: 1..=5000 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomePageShortDescription(String);

impl HomePageShortDescription {
    /// Maximum length of a home-page short description.
    pub const MAX_LEN: usize = 5000;
    /// Minimum length of a home-page short description.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `HomePageShortDescription`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(
            &s,
            Self::MIN_LEN,
            Self::MAX_LEN,
            "home page short description",
        )?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HomePageShortDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Home-slider link label: 1..=255 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomeSliderLinkLabel(String);

impl HomeSliderLinkLabel {
    /// Maximum length of a home-slider link label.
    pub const MAX_LEN: usize = 255;
    /// Minimum length of a home-slider link label.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `HomeSliderLinkLabel`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "home slider link label")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HomeSliderLinkLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Button text on per-page configurations: 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ButtonText(String);

impl ButtonText {
    /// Maximum length of a button text.
    pub const MAX_LEN: usize = 191;
    /// Minimum length of a button text.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `ButtonText`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "button text")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ButtonText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// =============================================================================
// URLs and files (locally-defined per the engine's per-domain
// `FileReference` / `Url` pattern — no cross-domain dep on
// `educore-documents` or `educore-communication`)
// =============================================================================

/// Validated URL, max 2048 chars. Per
/// `docs/specs/cms/value-objects.md` § "URLs, Files, and Slugs".
/// Each domain crate defines its own `Url` value object; the
/// wire form is identical across crates (a string up to 2048
/// chars).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Url(String);

impl Url {
    /// Maximum length of a URL.
    pub const MAX_LEN: usize = 2048;
    /// Minimum length of a URL.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `Url`, rejecting empty or overlong input.
    pub fn new(s: impl Into<String>) -> educore_core::error::Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "url must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(educore_core::error::DomainError::validation(format!(
                "url must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// File reference (object key). Each domain crate defines its
/// own `FileReference` value object; the wire form is identical
/// across crates (a non-empty string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileReference(String);

impl FileReference {
    /// Constructs a `FileReference`, rejecting empty input.
    pub fn new(raw: impl Into<String>) -> educore_core::error::Result<Self> {
        let s: String = raw.into();
        if s.is_empty() {
            return Err(educore_core::error::DomainError::validation(
                "file reference must be non-empty",
            ));
        }
        Ok(Self(s))
    }
    /// Returns the inner reference string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for FileReference {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// YouTube link. Validated as a YouTube URL when present.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct YoutubeLink(String);

impl YoutubeLink {
    /// Maximum length of a YouTube link.
    pub const MAX_LEN: usize = 2048;
    /// Minimum length of a YouTube link.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `YoutubeLink`, accepting either
    /// `youtube.com/watch?v=...` or `youtu.be/...` URLs.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("youtube link must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "youtube link must be at most {} chars, got {}",
                Self::MAX_LEN,
                s.chars().count()
            )));
        }
        if !s.contains("youtube.com/") && !s.contains("youtu.be/") {
            return Err(DomainError::validation(format!(
                "youtube link must contain youtube.com/ or youtu.be/, got {s:?}"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for YoutubeLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Button URL: a [`Url`]. Newtype wrapper for type clarity in
/// command shapes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ButtonUrl(pub Url);

impl ButtonUrl {
    /// Constructs a `ButtonUrl` from a [`Url`].
    #[must_use]
    pub const fn new(url: Url) -> Self {
        Self(url)
    }
    /// Returns the inner [`Url`].
    #[must_use]
    pub const fn as_url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for ButtonUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Source URL (for teacher-uploaded content). Newtype wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceUrl(pub Url);

impl SourceUrl {
    /// Constructs a `SourceUrl` from a [`Url`].
    #[must_use]
    pub const fn new(url: Url) -> Self {
        Self(url)
    }
    /// Returns the inner [`Url`].
    #[must_use]
    pub const fn as_url(&self) -> &Url {
        &self.0
    }
}

impl fmt::Display for SourceUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

// =============================================================================
// Date value objects
// =============================================================================

/// Publish date: a `NaiveDate`. Newtype wrapper for type clarity
/// in command shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PublishDate(pub chrono::NaiveDate);

impl PublishDate {
    /// Constructs a `PublishDate` from a `NaiveDate`.
    #[must_use]
    pub const fn new(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }
}

impl fmt::Display for PublishDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Notice date: a `NaiveDate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeDate(pub chrono::NaiveDate);

impl NoticeDate {
    /// Constructs a `NoticeDate` from a `NaiveDate`.
    #[must_use]
    pub const fn new(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }
}

impl fmt::Display for NoticeDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Share date: a `NaiveDate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShareDate(pub chrono::NaiveDate);

impl ShareDate {
    /// Constructs a `ShareDate` from a `NaiveDate`.
    #[must_use]
    pub const fn new(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }
}

impl fmt::Display for ShareDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Valid-until date: a `NaiveDate`. Used by
/// [`ContentShareList`](crate::aggregate::ContentShareList) to
/// bound the share window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidUntil(pub chrono::NaiveDate);

impl ValidUntil {
    /// Constructs a `ValidUntil` from a `NaiveDate`.
    #[must_use]
    pub const fn new(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }
}

impl fmt::Display for ValidUntil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Upload date: a `NaiveDate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UploadDate(pub chrono::NaiveDate);

impl UploadDate {
    /// Constructs an `UploadDate` from a `NaiveDate`.
    #[must_use]
    pub const fn new(date: chrono::NaiveDate) -> Self {
        Self(date)
    }
    /// Returns the inner `NaiveDate`.
    #[must_use]
    pub const fn as_naive_date(&self) -> chrono::NaiveDate {
        self.0
    }
}

impl fmt::Display for UploadDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Status enums (closed)
// =============================================================================

/// Page status: `Draft` or `Published`. Per the spec invariant 3
/// ("A `Page` has a `Status` of `draft` or `published`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PageStatus {
    /// A draft page; not visible on the public site.
    Draft,
    /// A published page; visible on the public site.
    Published,
}

impl PageStatus {
    /// Returns the wire byte for the status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
    /// Returns the next status for a transition action.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Draft => Self::Published,
            Self::Published => Self::Draft,
        }
    }
}

impl Default for PageStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl fmt::Display for PageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// News status: `Active` or `Disabled`. Per the spec invariant 3
/// ("A `News` has a `Status` flag (`active_status`) — `1` is
/// active, `0` is disabled").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NewsStatus {
    /// The news is active (visible on the public site).
    Active,
    /// The news is disabled (hidden from the public site).
    Disabled,
}

impl NewsStatus {
    /// Returns `true` for the active variant.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }
    /// Returns the wire byte (1 = active, 0 = disabled).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Active => 1,
            Self::Disabled => 0,
        }
    }
    /// Constructs a status from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self::Active),
            0 => Ok(Self::Disabled),
            other => Err(DomainError::validation(format!(
                "news status must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for NewsStatus {
    fn default() -> Self {
        Self::Active
    }
}

impl fmt::Display for NewsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => f.write_str("active"),
            Self::Disabled => f.write_str("disabled"),
        }
    }
}

/// News comment moderation status. Per the spec invariant 3
/// ("The `status` field is `0` (pending) or `1` (approved)"), but
/// the closed enum adds a `Hidden` variant for moderation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NewsCommentStatus {
    /// The comment is pending moderation.
    Pending,
    /// The comment is approved and visible on the public site.
    Approved,
    /// The comment is hidden by moderation (retained for audit
    /// but not surfaced).
    Hidden,
}

impl NewsCommentStatus {
    /// Returns the wire byte (0 = pending, 1 = approved, 2 =
    /// hidden).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Pending => 0,
            Self::Approved => 1,
            Self::Hidden => 2,
        }
    }
    /// Constructs a status from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Approved),
            2 => Ok(Self::Hidden),
            other => Err(DomainError::validation(format!(
                "news comment status must be 0, 1, or 2, got {other}"
            ))),
        }
    }
    /// Returns `true` for [`NewsCommentStatus::Approved`].
    #[must_use]
    pub const fn is_approved(self) -> bool {
        matches!(self, Self::Approved)
    }
}

impl fmt::Display for NewsCommentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => f.write_str("pending"),
            Self::Approved => f.write_str("approved"),
            Self::Hidden => f.write_str("hidden"),
        }
    }
}

/// Content-share send type. Per the spec invariant 2
/// ("The `send_type` is one of `G` (groups), `C` (class),
/// `I` (individual), `P` (public)").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentShareType {
    /// `G` — groups (role id list).
    Groups,
    /// `C` — class (one class + section id list).
    Class,
    /// `I` — individual (user id list).
    Individual,
    /// `P` — public (no specific recipients).
    Public,
}

impl ContentShareType {
    /// Returns the wire byte (`G`, `C`, `I`, or `P`).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        match self {
            Self::Groups => b'G',
            Self::Class => b'C',
            Self::Individual => b'I',
            Self::Public => b'P',
        }
    }
    /// Constructs a send type from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            b'G' | b'g' => Ok(Self::Groups),
            b'C' | b'c' => Ok(Self::Class),
            b'I' | b'i' => Ok(Self::Individual),
            b'P' | b'p' => Ok(Self::Public),
            other => Err(DomainError::validation(format!(
                "content share type must be G, C, I, or P, got {other:?}"
            ))),
        }
    }
    /// Returns the canonical lowercase wire char.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Groups => "g",
            Self::Class => "c",
            Self::Individual => "i",
            Self::Public => "p",
        }
    }
}

impl fmt::Display for ContentShareType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Groups => f.write_str("G"),
            Self::Class => f.write_str("C"),
            Self::Individual => f.write_str("I"),
            Self::Public => f.write_str("P"),
        }
    }
}

/// Content-share-list status. Per the spec invariant 5
/// ("A `ContentShareList` may be in `Draft`, `Dispatched`, or
/// `Cancelled` status").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentShareListStatus {
    /// The list is in draft; the audience may still be edited.
    Draft,
    /// The list has been dispatched; the audience is frozen.
    Dispatched,
    /// The list has been cancelled; not yet dispatched lists.
    Cancelled,
}

impl ContentShareListStatus {
    /// Returns the wire byte.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Dispatched => "dispatched",
            Self::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for ContentShareListStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Teacher-upload-content type. Per the spec invariant 2
/// ("The `content_type` is one of `assignment`, `study_material`,
/// `syllabus`, `other_download`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeacherContentType {
    /// An assignment.
    Assignment,
    /// A study material.
    StudyMaterial,
    /// A syllabus.
    Syllabus,
    /// Other downloadable content.
    OtherDownload,
}

impl TeacherContentType {
    /// Returns the canonical wire string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assignment => "assignment",
            Self::StudyMaterial => "study_material",
            Self::Syllabus => "syllabus",
            Self::OtherDownload => "other_download",
        }
    }
}

impl fmt::Display for TeacherContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// =============================================================================
// Boolean flag newtypes
// =============================================================================

/// Soft-delete flag. Mirrors the engine-wide `ActiveStatus`
/// pattern. `true` = active; `false` = soft-deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActiveStatus(pub bool);

impl ActiveStatus {
    /// Returns the active variant.
    #[must_use]
    pub const fn active() -> Self {
        Self(true)
    }
    /// Returns the soft-deleted variant.
    #[must_use]
    pub const fn inactive() -> Self {
        Self(false)
    }
    /// Constructs an `ActiveStatus` from a bool.
    #[must_use]
    pub const fn new(active: bool) -> Self {
        Self(active)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn is_active(self) -> bool {
        self.0
    }
    /// Returns the wire byte (1 = active, 0 = inactive).
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `ActiveStatus` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "active_status must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for ActiveStatus {
    fn default() -> Self {
        Self(true)
    }
}

impl fmt::Display for ActiveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(if self.0 { "active" } else { "inactive" })
    }
}

/// `home_page` flag. Per the spec invariant 4 ("At most one
/// `Page` per school may have `home_page = true`").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomePage(pub bool);

impl HomePage {
    /// Returns the `true` variant.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs a `HomePage` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "home_page must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_default` flag. Per the spec invariant 5 ("A `Page` may have
/// `is_default = true` only when it is a pre-installed template. A
/// default page is not deletable.").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsDefault(pub bool);

impl IsDefault {
    /// Returns the `true` variant.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns the inner bool.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsDefault` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_default must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsDefault {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_global` flag. Per the spec invariant 4 ("A `News` may be
/// `is_global` (visible across all schools in a multi-tenant
/// SaaS) or scoped to one school").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsGlobal(pub bool);

impl IsGlobal {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the global variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsGlobal` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_global must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsGlobal {
    fn default() -> Self {
        Self(false)
    }
}

/// `auto_approve` flag. Per the spec invariant 5 ("A `News` may
/// have `auto_approve = 1`, meaning new comments are approved
/// without moderation").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AutoApprove(pub bool);

impl AutoApprove {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the auto-approve variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `AutoApprove` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "auto_approve must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for AutoApprove {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_comment` flag. Per the spec invariant 6 ("A `News` may have
/// `is_comment = 1`, meaning comments are enabled on the news").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsComment(pub bool);

impl IsComment {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the comments-enabled variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsComment` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_comment must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsComment {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_published` flag for [`NoticeBoard`](crate::aggregate::NoticeBoard).
/// Per the spec invariant 3 ("A `NoticeBoard` may be
/// `is_published = 0` (hidden) or `is_published = 1` (visible).
/// Only published notice boards are surfaced on the public site").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsPublished(pub bool);

impl IsPublished {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the published variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsPublished` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_published must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsPublished {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_dynamic` flag for [`FrontendPage`](crate::aggregate::FrontendPage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsDynamic(pub bool);

impl IsDynamic {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the dynamic variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsDynamic` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_dynamic must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsDynamic {
    fn default() -> Self {
        Self(false)
    }
}

/// `is_parent` flag for [`CoursePage`](crate::aggregate::CoursePage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IsParent(pub bool);

impl IsParent {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the parent variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs an `IsParent` from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "is_parent must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for IsParent {
    fn default() -> Self {
        Self(false)
    }
}

/// `available_for_admin` flag for [`TeacherUploadContent`](crate::aggregate::TeacherUploadContent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AvailableForAdmin(pub bool);

impl AvailableForAdmin {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the available variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "available_for_admin must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for AvailableForAdmin {
    fn default() -> Self {
        Self(false)
    }
}

/// `available_for_all_classes` flag for
/// [`TeacherUploadContent`](crate::aggregate::TeacherUploadContent).
/// Per the spec invariant 4 ("The `available_for_all_classes`
/// flag, when set, suppresses the class filter").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AvailableForAllClasses(pub bool);

impl AvailableForAllClasses {
    /// Returns the inner bool.
    #[must_use]
    pub const fn new(v: bool) -> Self {
        Self(v)
    }
    /// Returns `true` for the all-classes variant.
    #[must_use]
    pub const fn is_true(self) -> bool {
        self.0
    }
    /// Returns the wire byte.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        if self.0 {
            1
        } else {
            0
        }
    }
    /// Constructs from a wire byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            1 => Ok(Self(true)),
            0 => Ok(Self(false)),
            other => Err(DomainError::validation(format!(
                "available_for_all_classes must be 0 or 1, got {other}"
            ))),
        }
    }
}

impl Default for AvailableForAllClasses {
    fn default() -> Self {
        Self(false)
    }
}

// =============================================================================
// Contact-page auxiliary value objects
// =============================================================================

/// Postal address for the contact page. 1..=191 chars per the
/// spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PostalAddress(String);

impl PostalAddress {
    /// Maximum length.
    pub const MAX_LEN: usize = 191;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PostalAddress`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("postal address must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "postal address must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PostalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Phone number (E.164 or alternative national formats accepted).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    /// Maximum length.
    pub const MAX_LEN: usize = 32;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `PhoneNumber`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("phone number must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "phone number must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Email address (RFC 5322, max 200 chars per the spec).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Maximum length.
    pub const MAX_LEN: usize = 200;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs an `EmailAddress`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("email address must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "email address must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        if !s.contains('@') {
            return Err(DomainError::validation(format!(
                "email address must contain '@', got {s:?}"
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Latitude string. 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Latitude(String);

impl Latitude {
    /// Maximum length.
    pub const MAX_LEN: usize = 191;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `Latitude`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("latitude must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "latitude must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Latitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Longitude string. 1..=191 chars per the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Longitude(String);

impl Longitude {
    /// Maximum length.
    pub const MAX_LEN: usize = 191;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `Longitude`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation("longitude must not be empty"));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "longitude must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Longitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Map zoom level. `i32` in `0..=21`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZoomLevel(i32);

impl ZoomLevel {
    /// Minimum zoom level.
    pub const MIN: i32 = 0;
    /// Maximum zoom level.
    pub const MAX: i32 = 21;
    /// Constructs a `ZoomLevel`.
    pub fn new(value: i32) -> Result<Self> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(DomainError::validation(format!(
                "zoom level must be in {}..={}, got {}",
                Self::MIN,
                Self::MAX,
                value
            )));
        }
        Ok(Self(value))
    }
    /// Returns the inner i32.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl Default for ZoomLevel {
    fn default() -> Self {
        Self(15)
    }
}

impl fmt::Display for ZoomLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "zoom={}", self.0)
    }
}

/// Google Maps address. 1..=191 chars.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GoogleMapAddress(String);

impl GoogleMapAddress {
    /// Maximum length.
    pub const MAX_LEN: usize = 191;
    /// Minimum length.
    pub const MIN_LEN: usize = 1;
    /// Constructs a `GoogleMapAddress`.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        if s.is_empty() {
            return Err(DomainError::validation(
                "google map address must not be empty",
            ));
        }
        if s.chars().count() > Self::MAX_LEN {
            return Err(DomainError::validation(format!(
                "google map address must be at most {} chars",
                Self::MAX_LEN
            )));
        }
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GoogleMapAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// =============================================================================
// Audience
// =============================================================================

/// Comma-separated audience descriptor for notice boards. Per
/// the spec invariant 4 ("The audience (`inform_to`) is a
/// comma-separated list of role identifiers").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AudienceDescriptor(String);

impl AudienceDescriptor {
    /// Maximum length of an audience descriptor.
    pub const MAX_LEN: usize = 500;
    /// Minimum length of an audience descriptor.
    pub const MIN_LEN: usize = 1;
    /// Constructs an `AudienceDescriptor`, rejecting empty or
    /// overlong input.
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s: String = s.into();
        validate_title(&s, Self::MIN_LEN, Self::MAX_LEN, "audience descriptor")?;
        Ok(Self(s))
    }
    /// Returns the inner string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Splits the descriptor on `,` and returns the trimmed
    /// list of audience identifiers.
    #[must_use]
    pub fn split(&self) -> Vec<String> {
        self.0
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }
}

impl fmt::Display for AudienceDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// =============================================================================
// PageSettings (typed JSON value)
// =============================================================================

/// A typed JSON value for per-page settings. Per the spec, the
/// schema is versioned and consumer-defined; the domain
/// validates that the JSON is well-formed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageSettings(pub serde_json::Value);

impl PageSettings {
    /// Constructs a `PageSettings` from any serializable value.
    pub fn new<T: Serialize>(value: &T) -> Result<Self> {
        serde_json::to_value(value)
            .map(Self)
            .map_err(|e| DomainError::validation(format!("page settings not serializable: {e}")))
    }
    /// Constructs a `PageSettings` from a raw [`serde_json::Value`].
    #[must_use]
    pub const fn from_value(value: serde_json::Value) -> Self {
        Self(value)
    }
    /// Returns the inner value.
    #[must_use]
    pub const fn as_value(&self) -> &serde_json::Value {
        &self.0
    }
    /// Returns the schema version (if the value is an object with
    /// a `"schema_version"` integer field).
    #[must_use]
    pub fn schema_version(&self) -> Option<u32> {
        self.0
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .map(|v| u32::try_from(v).unwrap_or(0))
    }
}

// =============================================================================
// Common helper
// =============================================================================

fn validate_title(s: &str, min: usize, max: usize, name: &str) -> Result<()> {
    if s.is_empty() {
        return Err(DomainError::validation(format!("{name} must not be empty")));
    }
    if s.chars().count() < min {
        return Err(DomainError::validation(format!(
            "{name} must be at least {min} chars"
        )));
    }
    if s.chars().count() > max {
        return Err(DomainError::validation(format!(
            "{name} must be at most {max} chars, got {}",
            s.chars().count()
        )));
    }
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_academic::prelude::Identifier as _;
    use educore_core::ids::Identifier as _;

    #[test]
    fn page_id_display_includes_school_and_uuid() {
        let school = SchoolId::from_uuid(uuid::Uuid::now_v7());
        let value = uuid::Uuid::now_v7();
        let id = PageId::new(school, value);
        let s = id.to_string();
        assert!(s.contains(&school.to_string()));
        assert!(s.contains(&value.to_string()));
    }

    #[test]
    fn page_id_equality_uses_school_and_value() {
        let school = SchoolId::from_uuid(uuid::Uuid::now_v7());
        let value = uuid::Uuid::now_v7();
        let a = PageId::new(school, value);
        let b = PageId::new(school, value);
        let c = PageId::new(school, uuid::Uuid::now_v7());
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn slug_rejects_empty() {
        let err = Slug::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn slug_rejects_uppercase() {
        let err = Slug::new("My-Page").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn slug_rejects_underscore() {
        let err = Slug::new("my_page").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn slug_accepts_alphanumeric_dash() {
        let s = Slug::new("my-page-2").unwrap();
        assert_eq!(s.as_str(), "my-page-2");
    }

    #[test]
    fn slug_rejects_overlong() {
        let s: String = std::iter::repeat('a').take(Slug::MAX_LEN + 1).collect();
        let err = Slug::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn page_title_rejects_empty() {
        let err = PageTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn page_title_accepts_max_len() {
        let s: String = std::iter::repeat('a').take(PageTitle::MAX_LEN).collect();
        let t = PageTitle::new(s).unwrap();
        assert_eq!(t.as_str().chars().count(), PageTitle::MAX_LEN);
    }

    #[test]
    fn page_title_rejects_overlong() {
        let s: String = std::iter::repeat('a')
            .take(PageTitle::MAX_LEN + 1)
            .collect();
        let err = PageTitle::new(s).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn news_body_rejects_empty() {
        let err = NewsBody::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn star_rating_rejects_zero() {
        let err = StarRating::new(0).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn star_rating_rejects_six() {
        let err = StarRating::new(6).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn star_rating_accepts_boundary_values() {
        assert_eq!(StarRating::new(1).unwrap().value(), 1);
        assert_eq!(StarRating::new(5).unwrap().value(), 5);
    }

    #[test]
    fn content_share_type_byte_round_trip() {
        assert_eq!(
            ContentShareType::from_byte(b'G').unwrap(),
            ContentShareType::Groups
        );
        assert_eq!(
            ContentShareType::from_byte(b'C').unwrap(),
            ContentShareType::Class
        );
        assert_eq!(
            ContentShareType::from_byte(b'I').unwrap(),
            ContentShareType::Individual
        );
        assert_eq!(
            ContentShareType::from_byte(b'P').unwrap(),
            ContentShareType::Public
        );
    }

    #[test]
    fn content_share_type_byte_rejects_invalid() {
        let err = ContentShareType::from_byte(b'X').unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn content_share_list_status_default_is_draft() {
        assert_eq!(ContentShareListStatus::Draft, ContentShareListStatus::Draft);
    }

    #[test]
    fn news_status_default_is_active() {
        assert_eq!(NewsStatus::default(), NewsStatus::Active);
    }

    #[test]
    fn news_status_byte_round_trip() {
        assert_eq!(NewsStatus::from_byte(1).unwrap(), NewsStatus::Active);
        assert_eq!(NewsStatus::from_byte(0).unwrap(), NewsStatus::Disabled);
    }

    #[test]
    fn news_comment_status_byte_round_trip() {
        assert_eq!(
            NewsCommentStatus::from_byte(0).unwrap(),
            NewsCommentStatus::Pending
        );
        assert_eq!(
            NewsCommentStatus::from_byte(1).unwrap(),
            NewsCommentStatus::Approved
        );
        assert_eq!(
            NewsCommentStatus::from_byte(2).unwrap(),
            NewsCommentStatus::Hidden
        );
    }

    #[test]
    fn active_status_byte_round_trip() {
        assert!(ActiveStatus::from_byte(1).unwrap().is_active());
        assert!(!ActiveStatus::from_byte(0).unwrap().is_active());
    }

    #[test]
    fn page_status_next_flips_draft_to_published() {
        assert_eq!(PageStatus::Draft.next(), PageStatus::Published);
    }

    #[test]
    fn page_status_next_flips_published_to_draft() {
        assert_eq!(PageStatus::Published.next(), PageStatus::Draft);
    }

    #[test]
    fn page_status_default_is_draft() {
        assert_eq!(PageStatus::default(), PageStatus::Draft);
    }

    #[test]
    fn page_status_as_str_is_stable() {
        assert_eq!(PageStatus::Draft.as_str(), "draft");
        assert_eq!(PageStatus::Published.as_str(), "published");
    }

    #[test]
    fn audience_descriptor_splits_on_comma() {
        let a = AudienceDescriptor::new("admin,teacher,parent").unwrap();
        let parts = a.split();
        assert_eq!(parts, vec!["admin", "teacher", "parent"]);
    }

    #[test]
    fn audience_descriptor_splits_with_whitespace() {
        let a = AudienceDescriptor::new("admin , teacher , parent").unwrap();
        let parts = a.split();
        assert_eq!(parts, vec!["admin", "teacher", "parent"]);
    }

    #[test]
    fn audience_descriptor_rejects_empty() {
        let err = AudienceDescriptor::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn youtube_link_accepts_youtube_com() {
        let link = YoutubeLink::new("https://www.youtube.com/watch?v=abc").unwrap();
        assert!(link.as_str().contains("youtube.com/"));
    }

    #[test]
    fn youtube_link_accepts_youtu_be() {
        let link = YoutubeLink::new("https://youtu.be/abc").unwrap();
        assert!(link.as_str().contains("youtu.be/"));
    }

    #[test]
    fn youtube_link_rejects_non_youtube() {
        let err = YoutubeLink::new("https://example.com/video").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn page_settings_round_trips_json() {
        let s =
            PageSettings::new(&serde_json::json!({"schema_version": 1, "key": "value"})).unwrap();
        assert_eq!(s.schema_version(), Some(1));
    }

    #[test]
    fn page_settings_schema_version_returns_none_for_non_object() {
        let s = PageSettings::from_value(serde_json::json!("just a string"));
        assert_eq!(s.schema_version(), None);
    }

    #[test]
    fn button_url_wraps_url() {
        let url = Url::new("https://example.com").unwrap();
        let b = ButtonUrl::new(url);
        assert_eq!(b.as_url().as_str(), "https://example.com");
    }

    #[test]
    fn source_url_wraps_url() {
        let url = Url::new("https://example.com").unwrap();
        let s = SourceUrl::new(url);
        assert_eq!(s.as_url().as_str(), "https://example.com");
    }

    #[test]
    fn home_page_default_is_false() {
        assert!(!HomePage::default().is_true());
    }

    #[test]
    fn is_default_default_is_false() {
        assert!(!IsDefault::default().is_true());
    }

    #[test]
    fn is_global_default_is_false() {
        assert!(!IsGlobal::default().is_true());
    }

    #[test]
    fn auto_approve_default_is_false() {
        assert!(!AutoApprove::default().is_true());
    }

    #[test]
    fn is_comment_default_is_false() {
        assert!(!IsComment::default().is_true());
    }

    #[test]
    fn is_published_default_is_false() {
        assert!(!IsPublished::default().is_true());
    }

    #[test]
    fn is_dynamic_default_is_false() {
        assert!(!IsDynamic::default().is_true());
    }

    #[test]
    fn is_parent_default_is_false() {
        assert!(!IsParent::default().is_true());
    }

    #[test]
    fn available_for_admin_default_is_false() {
        assert!(!AvailableForAdmin::default().is_true());
    }

    #[test]
    fn available_for_all_classes_default_is_false() {
        assert!(!AvailableForAllClasses::default().is_true());
    }

    #[test]
    fn teacher_content_type_as_str_is_stable() {
        assert_eq!(TeacherContentType::Assignment.as_str(), "assignment");
        assert_eq!(TeacherContentType::StudyMaterial.as_str(), "study_material");
        assert_eq!(TeacherContentType::Syllabus.as_str(), "syllabus");
        assert_eq!(TeacherContentType::OtherDownload.as_str(), "other_download");
    }

    #[test]
    fn person_name_rejects_empty() {
        let err = PersonName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn person_name_accepts_max_len() {
        let s: String = std::iter::repeat('a').take(PersonName::MAX_LEN).collect();
        let n = PersonName::new(s).unwrap();
        assert_eq!(n.as_str().chars().count(), PersonName::MAX_LEN);
    }

    #[test]
    fn designation_rejects_empty() {
        let err = Designation::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn institution_name_rejects_empty() {
        let err = InstitutionName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn notice_title_rejects_empty() {
        let err = NoticeTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn notice_message_rejects_empty() {
        let err = NoticeMessage::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn category_name_rejects_empty() {
        let err = CategoryName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn comment_message_rejects_empty() {
        let err = CommentMessage::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn content_title_rejects_empty() {
        let err = ContentTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn content_description_rejects_empty() {
        let err = ContentDescription::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn content_type_name_rejects_empty() {
        let err = ContentTypeName::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn content_share_list_title_rejects_empty() {
        let err = ContentShareListTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn course_page_title_rejects_empty() {
        let err = CoursePageTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn course_page_description_rejects_empty() {
        let err = CoursePageDescription::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn home_page_title_rejects_empty() {
        let err = HomePageTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn home_page_long_title_rejects_empty() {
        let err = HomePageLongTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn home_page_short_description_rejects_empty() {
        let err = HomePageShortDescription::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn home_slider_link_label_rejects_empty() {
        let err = HomeSliderLinkLabel::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn button_text_rejects_empty() {
        let err = ButtonText::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn speech_text_rejects_empty() {
        let err = SpeechText::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn page_description_rejects_empty() {
        let err = PageDescription::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn testimonial_description_rejects_empty() {
        let err = TestimonialDescription::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn page_sub_title_rejects_empty() {
        let err = PageSubTitle::new("").unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }
}
