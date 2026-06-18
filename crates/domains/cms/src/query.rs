//! CMS-domain typed query stubs.
//!
//! Per `docs/specs/cms/repositories.md`, the CMS domain ships 19
//! repository port traits (one per root aggregate except
//! `SpeechSlider` which is unique-per-school and shares the
//! home slider's query pattern). Each repository has a typed
//! query builder that mirrors the engine-wide `QueryNode<F>`
//! AST. Phase 12 ships the typed query builders; the typed
//! executors land in a follow-up phase alongside the
//! `#[derive(DomainQuery)]` macro emissions.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::value_objects::{
    ActiveStatus, CategoryName, ContentShareType, ContentTypeName, Designation, InstitutionName,
    IsComment, IsGlobal, IsParent, IsPublished, NewsCommentStatus, NewsStatus, NewsTitle,
    PageStatus, PageSubTitle, PageTitle, PersonName, Slug, StarRating,
};

// =============================================================================
// PageQuery (owner: A)
// =============================================================================

/// Typed query builder for [`Page`](crate::aggregate::Page).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PageQuery {
    /// Filter by title (exact match).
    pub title: Option<PageTitle>,
    /// Filter by slug (exact match).
    pub slug: Option<Slug>,
    /// Filter by status.
    pub status: Option<PageStatus>,
    /// Filter by home_page flag.
    pub home_page: Option<bool>,
    /// Filter by is_default flag.
    pub is_default: Option<bool>,
    /// Filter by active_status (default `None` = active only).
    pub active_status: Option<ActiveStatus>,
}

impl PageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, title: PageTitle) -> Self {
        self.title = Some(title);
        self
    }
    /// Filter by slug.
    #[must_use]
    pub fn with_slug(mut self, slug: Slug) -> Self {
        self.slug = Some(slug);
        self
    }
    /// Filter by status.
    #[must_use]
    pub fn with_status(mut self, status: PageStatus) -> Self {
        self.status = Some(status);
        self
    }
    /// Filter by home_page.
    #[must_use]
    pub fn with_home_page(mut self, home_page: bool) -> Self {
        self.home_page = Some(home_page);
        self
    }
    /// Filter by is_default.
    #[must_use]
    pub fn with_is_default(mut self, is_default: bool) -> Self {
        self.is_default = Some(is_default);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, active: ActiveStatus) -> Self {
        self.active_status = Some(active);
        self
    }
}

// =============================================================================
// NewsQuery + NewsCategoryQuery + NewsCommentQuery + NewsPageQuery
// (owner: B)
// =============================================================================

/// Typed query builder for [`News`](crate::aggregate::News).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsQuery {
    /// Filter by news title.
    pub news_title: Option<NewsTitle>,
    /// Filter by category id.
    pub category_id: Option<crate::value_objects::NewsCategoryId>,
    /// Filter by is_global.
    pub is_global: Option<IsGlobal>,
    /// Filter by is_comment.
    pub is_comment: Option<IsComment>,
    /// Filter by status.
    pub active_status: Option<NewsStatus>,
}

impl NewsQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by news title.
    #[must_use]
    pub fn with_title(mut self, t: NewsTitle) -> Self {
        self.news_title = Some(t);
        self
    }
    /// Filter by category id.
    #[must_use]
    pub fn with_category(mut self, c: crate::value_objects::NewsCategoryId) -> Self {
        self.category_id = Some(c);
        self
    }
    /// Filter by is_global.
    #[must_use]
    pub fn with_is_global(mut self, g: IsGlobal) -> Self {
        self.is_global = Some(g);
        self
    }
    /// Filter by is_comment.
    #[must_use]
    pub fn with_is_comment(mut self, c: IsComment) -> Self {
        self.is_comment = Some(c);
        self
    }
    /// Filter by status.
    #[must_use]
    pub fn with_active(mut self, s: NewsStatus) -> Self {
        self.active_status = Some(s);
        self
    }
}

/// Typed query builder for [`NewsCategory`](crate::aggregate::NewsCategory).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsCategoryQuery {
    /// Filter by category name.
    pub category_name: Option<CategoryName>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl NewsCategoryQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by category name.
    #[must_use]
    pub fn with_name(mut self, n: CategoryName) -> Self {
        self.category_name = Some(n);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`NewsComment`](crate::aggregate::NewsComment).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsCommentQuery {
    /// Filter by news id.
    pub news_id: Option<crate::value_objects::NewsId>,
    /// Filter by user id.
    pub user_id: Option<educore_core::ids::UserId>,
    /// Filter by status.
    pub status: Option<NewsCommentStatus>,
}

impl NewsCommentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by news id.
    #[must_use]
    pub fn with_news(mut self, n: crate::value_objects::NewsId) -> Self {
        self.news_id = Some(n);
        self
    }
    /// Filter by user id.
    #[must_use]
    pub fn with_user(mut self, u: educore_core::ids::UserId) -> Self {
        self.user_id = Some(u);
        self
    }
    /// Filter by status.
    #[must_use]
    pub fn with_status(mut self, s: NewsCommentStatus) -> Self {
        self.status = Some(s);
        self
    }
}

/// Typed query builder for [`NewsPage`](crate::aggregate::NewsPage).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NewsPageQuery {
    /// Filter by title.
    pub title: Option<PageTitle>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl NewsPageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, t: PageTitle) -> Self {
        self.title = Some(t);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

// =============================================================================
// NoticeBoardQuery + TestimonialQuery + HomeSliderQuery + SpeechSliderQuery
// (owner: C)
// =============================================================================

/// Typed query builder for [`NoticeBoard`](crate::aggregate::NoticeBoard).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardQuery {
    /// Filter by notice title.
    pub notice_title: Option<crate::value_objects::NoticeTitle>,
    /// Filter by is_published flag.
    pub is_published: Option<IsPublished>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl NoticeBoardQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by notice title.
    #[must_use]
    pub fn with_title(mut self, t: crate::value_objects::NoticeTitle) -> Self {
        self.notice_title = Some(t);
        self
    }
    /// Filter by is_published.
    #[must_use]
    pub fn with_is_published(mut self, p: IsPublished) -> Self {
        self.is_published = Some(p);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`Testimonial`](crate::aggregate::Testimonial).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TestimonialQuery {
    /// Filter by name.
    pub name: Option<PersonName>,
    /// Filter by designation.
    pub designation: Option<Designation>,
    /// Filter by institution_name.
    pub institution_name: Option<InstitutionName>,
    /// Filter by minimum star_rating.
    pub min_rating: Option<StarRating>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl TestimonialQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by name.
    #[must_use]
    pub fn with_name(mut self, n: PersonName) -> Self {
        self.name = Some(n);
        self
    }
    /// Filter by designation.
    #[must_use]
    pub fn with_designation(mut self, d: Designation) -> Self {
        self.designation = Some(d);
        self
    }
    /// Filter by institution_name.
    #[must_use]
    pub fn with_institution(mut self, i: InstitutionName) -> Self {
        self.institution_name = Some(i);
        self
    }
    /// Filter by minimum rating.
    #[must_use]
    pub fn with_min_rating(mut self, r: StarRating) -> Self {
        self.min_rating = Some(r);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`HomeSlider`](crate::aggregate::HomeSlider).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HomeSliderQuery {
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl HomeSliderQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`SpeechSlider`](crate::aggregate::SpeechSlider).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderQuery {
    /// Filter by name.
    pub name: Option<PersonName>,
    /// Filter by designation.
    pub designation: Option<Designation>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl SpeechSliderQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by name.
    #[must_use]
    pub fn with_name(mut self, n: PersonName) -> Self {
        self.name = Some(n);
        self
    }
    /// Filter by designation.
    #[must_use]
    pub fn with_designation(mut self, d: Designation) -> Self {
        self.designation = Some(d);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

// =============================================================================
// Content family queries (owner: D1 + D2)
// =============================================================================

/// Typed query builder for [`Content`](crate::aggregate::Content).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentQuery {
    /// Filter by file name.
    pub file_name: Option<String>,
    /// Filter by content type id.
    pub content_type_id: Option<crate::value_objects::ContentTypeId>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl ContentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by file name.
    #[must_use]
    pub fn with_file_name(mut self, n: String) -> Self {
        self.file_name = Some(n);
        self
    }
    /// Filter by content type id.
    #[must_use]
    pub fn with_content_type(mut self, c: crate::value_objects::ContentTypeId) -> Self {
        self.content_type_id = Some(c);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`ContentType`](crate::aggregate::ContentType).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentTypeQuery {
    /// Filter by type name.
    pub type_name: Option<ContentTypeName>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl ContentTypeQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by type name.
    #[must_use]
    pub fn with_name(mut self, n: ContentTypeName) -> Self {
        self.type_name = Some(n);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`ContentShareList`](crate::aggregate::ContentShareList).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListQuery {
    /// Filter by send_type.
    pub send_type: Option<ContentShareType>,
    /// Filter by status.
    pub status: Option<crate::value_objects::ContentShareListStatus>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl ContentShareListQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by send_type.
    #[must_use]
    pub fn with_send_type(mut self, s: ContentShareType) -> Self {
        self.send_type = Some(s);
        self
    }
    /// Filter by status.
    #[must_use]
    pub fn with_status(mut self, s: crate::value_objects::ContentShareListStatus) -> Self {
        self.status = Some(s);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`TeacherUploadContent`](crate::aggregate::TeacherUploadContent).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TeacherUploadContentQuery {
    /// Filter by content title.
    pub content_title: Option<crate::value_objects::ContentTitle>,
    /// Filter by content_type.
    pub content_type: Option<crate::value_objects::TeacherContentType>,
    /// Filter by class id.
    pub class_id: Option<educore_academic::ClassId>,
    /// Filter by section id.
    pub section_id: Option<educore_academic::SectionId>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl TeacherUploadContentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by content title.
    #[must_use]
    pub fn with_title(mut self, t: crate::value_objects::ContentTitle) -> Self {
        self.content_title = Some(t);
        self
    }
    /// Filter by content_type.
    #[must_use]
    pub fn with_content_type(mut self, c: crate::value_objects::TeacherContentType) -> Self {
        self.content_type = Some(c);
        self
    }
    /// Filter by class id.
    #[must_use]
    pub fn with_class(mut self, c: educore_academic::ClassId) -> Self {
        self.class_id = Some(c);
        self
    }
    /// Filter by section id.
    #[must_use]
    pub fn with_section(mut self, s: educore_academic::SectionId) -> Self {
        self.section_id = Some(s);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`UploadContent`](crate::aggregate::UploadContent).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UploadContentQuery {
    /// Filter by content title.
    pub content_title: Option<crate::value_objects::ContentTitle>,
    /// Filter by content_type (raw i32 FK).
    pub content_type: Option<i32>,
    /// Filter by role id.
    pub available_for_role: Option<i32>,
    /// Filter by class id.
    pub available_for_class: Option<i32>,
    /// Filter by section id.
    pub available_for_section: Option<i32>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl UploadContentQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by content title.
    #[must_use]
    pub fn with_title(mut self, t: crate::value_objects::ContentTitle) -> Self {
        self.content_title = Some(t);
        self
    }
    /// Filter by content type.
    #[must_use]
    pub fn with_content_type(mut self, c: i32) -> Self {
        self.content_type = Some(c);
        self
    }
    /// Filter by role.
    #[must_use]
    pub fn with_role(mut self, r: i32) -> Self {
        self.available_for_role = Some(r);
        self
    }
    /// Filter by class.
    #[must_use]
    pub fn with_class(mut self, c: i32) -> Self {
        self.available_for_class = Some(c);
        self
    }
    /// Filter by section.
    #[must_use]
    pub fn with_section(mut self, s: i32) -> Self {
        self.available_for_section = Some(s);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

// =============================================================================
// Per-page template queries (owner: E)
// =============================================================================

/// Typed query builder for [`AboutPage`](crate::aggregate::AboutPage).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AboutPageQuery {
    /// Filter by title.
    pub title: Option<PageTitle>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl AboutPageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, t: PageTitle) -> Self {
        self.title = Some(t);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`ContactPage`](crate::aggregate::ContactPage).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ContactPageQuery {
    /// Filter by title.
    pub title: Option<PageTitle>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl ContactPageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, t: PageTitle) -> Self {
        self.title = Some(t);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`CoursePage`](crate::aggregate::CoursePage).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CoursePageQuery {
    /// Filter by title.
    pub title: Option<crate::value_objects::CoursePageTitle>,
    /// Filter by is_parent.
    pub is_parent: Option<IsParent>,
    /// Filter by parent_id.
    pub parent_id: Option<crate::value_objects::CoursePageId>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl CoursePageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, t: crate::value_objects::CoursePageTitle) -> Self {
        self.title = Some(t);
        self
    }
    /// Filter by is_parent.
    #[must_use]
    pub fn with_is_parent(mut self, p: IsParent) -> Self {
        self.is_parent = Some(p);
        self
    }
    /// Filter by parent_id.
    #[must_use]
    pub fn with_parent(mut self, p: crate::value_objects::CoursePageId) -> Self {
        self.parent_id = Some(p);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`HomePageSetting`](crate::aggregate::HomePageSetting).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HomePageSettingQuery {
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl HomePageSettingQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
}

/// Typed query builder for [`FrontendPage`](crate::aggregate::FrontendPage).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FrontendPageQuery {
    /// Filter by title.
    pub title: Option<PageTitle>,
    /// Filter by sub_title (exact match).
    pub sub_title: Option<PageSubTitle>,
    /// Filter by slug (exact match).
    pub slug: Option<Slug>,
    /// Filter by active_status.
    pub active_status: Option<ActiveStatus>,
}

impl FrontendPageQuery {
    /// Constructs a new empty query.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Filter by title.
    #[must_use]
    pub fn with_title(mut self, t: PageTitle) -> Self {
        self.title = Some(t);
        self
    }
    /// Filter by sub_title.
    #[must_use]
    pub fn with_sub_title(mut self, s: PageSubTitle) -> Self {
        self.sub_title = Some(s);
        self
    }
    /// Filter by slug.
    #[must_use]
    pub fn with_slug(mut self, s: Slug) -> Self {
        self.slug = Some(s);
        self
    }
    /// Filter by active_status.
    #[must_use]
    pub fn with_active(mut self, a: ActiveStatus) -> Self {
        self.active_status = Some(a);
        self
    }
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

    #[test]
    fn page_query_default_is_empty() {
        let q = PageQuery::default();
        assert!(q.title.is_none());
        assert!(q.slug.is_none());
        assert!(q.status.is_none());
        assert!(q.home_page.is_none());
        assert!(q.is_default.is_none());
        assert!(q.active_status.is_none());
    }

    #[test]
    fn page_query_with_title_accumulates_filter() {
        let q = PageQuery::new().with_title(PageTitle::new("My Page").unwrap());
        assert!(q.title.is_some());
    }

    #[test]
    fn page_query_with_status_accumulates_filter() {
        let q = PageQuery::new().with_status(PageStatus::Published);
        assert_eq!(q.status, Some(PageStatus::Published));
    }

    #[test]
    fn news_query_default_is_empty() {
        let q = NewsQuery::default();
        assert!(q.news_title.is_none());
        assert!(q.category_id.is_none());
        assert!(q.is_global.is_none());
        assert!(q.is_comment.is_none());
        assert!(q.active_status.is_none());
    }

    #[test]
    fn news_query_with_active_accumulates_filter() {
        let q = NewsQuery::new().with_active(NewsStatus::Active);
        assert_eq!(q.active_status, Some(NewsStatus::Active));
    }

    #[test]
    fn news_comment_query_with_status_accumulates_filter() {
        let q = NewsCommentQuery::new().with_status(NewsCommentStatus::Pending);
        assert_eq!(q.status, Some(NewsCommentStatus::Pending));
    }

    #[test]
    fn testimonial_query_with_min_rating_accumulates_filter() {
        let q = TestimonialQuery::new().with_min_rating(StarRating::new(4).unwrap());
        assert_eq!(q.min_rating.unwrap().value(), 4);
    }

    #[test]
    fn content_share_list_query_with_send_type_accumulates_filter() {
        let q = ContentShareListQuery::new().with_send_type(ContentShareType::Public);
        assert_eq!(q.send_type, Some(ContentShareType::Public));
    }

    #[test]
    fn content_share_list_query_with_status_accumulates_filter() {
        let q = ContentShareListQuery::new()
            .with_status(crate::value_objects::ContentShareListStatus::Draft);
        assert_eq!(
            q.status,
            Some(crate::value_objects::ContentShareListStatus::Draft)
        );
    }

    #[test]
    fn frontend_page_query_with_sub_title_accumulates_filter() {
        let q = FrontendPageQuery::new().with_sub_title(PageSubTitle::new("Welcome").unwrap());
        assert!(q.sub_title.is_some());
    }
}
