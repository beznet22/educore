//! CMS-domain repository port traits.
//!
//! Per `docs/specs/cms/repositories.md`. The CMS domain ships 27
//! repository port traits: 19 root-aggregate traits (one per
//! root aggregate; `SpeechSlider` is unique-per-school and
//! reuses the same query pattern as `HomeSlider`) plus 8
//! command-shaped traits for the New*/Update* aggregates that
//! the spec carves out as standalone persisted commands
//! (NewHomeSlider, NewSpeechSlider, NewTestimonial, NewNews,
//! NewNewsCategory, NewNewsComment, NewNoticeBoard,
//! UpdateNews). Each trait is object-safe (the
//! `_assert_object_safe` helpers in this module prove it).

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_academic::ClassId;
use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;

use crate::aggregate::{
    AboutPage, ContactPage, Content, ContentShareList, ContentType, CoursePage, FrontendPage,
    HomePageSetting, HomeSlider, NewHomeSlider, NewNews, NewNewsCategory, NewNewsComment,
    NewNoticeBoard, NewSpeechSlider, NewTestimonial, News, NewsCategory, NewsComment, NewsPage,
    NoticeBoard, Page, SpeechSlider, TeacherUploadContent, Testimonial, UpdateNews, UploadContent,
};
use crate::query::{
    AboutPageQuery, ContactPageQuery, ContentQuery, ContentShareListQuery, ContentTypeQuery,
    CoursePageQuery, FrontendPageQuery, HomePageSettingQuery, HomeSliderQuery, NewsCategoryQuery,
    NewsCommentQuery, NewsPageQuery, NewsQuery, NoticeBoardQuery, PageQuery, SpeechSliderQuery,
    TeacherUploadContentQuery, TestimonialQuery, UploadContentQuery,
};
use crate::value_objects::{
    AboutPageId, ContactPageId, ContentId, ContentShareListId, ContentTypeId, CoursePageId,
    FrontendPageId, HomePageSettingId, HomeSliderId, NewHomeSliderId, NewNewsCategoryId,
    NewNewsCommentId, NewNewsId, NewNoticeBoardId, NewSpeechSliderId, NewTestimonialId,
    NewsCategoryId, NewsCommentId, NewsId, NewsPageId, NoticeBoardId, PageId, SpeechSliderId,
    TeacherUploadContentId, TestimonialId, UpdateNewsId, UploadContentId,
};

// =============================================================================
// PageRepository (owner: A)
// =============================================================================

/// Repository port for [`Page`](crate::aggregate::Page).
#[async_trait]
pub trait PageRepository: Send + Sync {
    /// Fetch a page by its typed id.
    async fn get(&self, id: PageId) -> StorageResult<Option<Page>>;
    /// Find a page by `(school_id, slug)`.
    async fn find_by_slug(
        &self,
        school: SchoolId,
        slug: &crate::value_objects::Slug,
    ) -> StorageResult<Option<Page>>;
    /// Find the school's home page (where `home_page = true`).
    async fn find_home(&self, school: SchoolId) -> StorageResult<Option<Page>>;
    /// List pages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: PageQuery) -> StorageResult<Vec<Page>>;
    /// List published pages for a school.
    async fn list_published(&self, school: SchoolId) -> StorageResult<Vec<Page>>;
    /// Insert a new page.
    async fn insert(&self, p: &Page) -> StorageResult<()>;
    /// Update an existing page.
    async fn update(&self, p: &Page) -> StorageResult<()>;
    /// Soft-delete a page (sets `active_status = false`).
    async fn delete(&self, id: PageId) -> StorageResult<()>;
    /// Count pages matching the typed query.
    async fn count(&self, school: SchoolId, q: PageQuery) -> StorageResult<u64>;
    /// Page through pages matching the typed query.
    async fn page(
        &self,
        school: SchoolId,
        q: PageQuery,
        offset: u32,
        limit: u32,
    ) -> StorageResult<Vec<Page>>;
}

/// Object-safety smoke test.
fn _assert_page_object_safe() {
    fn _f(_: Box<dyn PageRepository>) {}
}

// =============================================================================
// NewsRepository + NewsCategoryRepository + NewsCommentRepository +
// NewsPageRepository (owner: B)
// =============================================================================

/// Repository port for [`News`](crate::aggregate::News).
#[async_trait]
pub trait NewsRepository: Send + Sync {
    /// Fetch a news entry by its typed id.
    async fn get(&self, id: NewsId) -> StorageResult<Option<News>>;
    /// List news entries for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: NewsQuery) -> StorageResult<Vec<News>>;
    /// List active news entries for a school.
    async fn list_active(&self, school: SchoolId) -> StorageResult<Vec<News>>;
    /// List global news entries (visible across all schools).
    async fn list_global(&self) -> StorageResult<Vec<News>>;
    /// List news entries for a category.
    async fn list_by_category(
        &self,
        school: SchoolId,
        category: NewsCategoryId,
    ) -> StorageResult<Vec<News>>;
    /// List news entries whose publish date falls within the
    /// inclusive range `[from, to]`.
    async fn list_published_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> StorageResult<Vec<News>>;
    /// Insert a new news entry.
    async fn insert(&self, n: &News) -> StorageResult<()>;
    /// Update an existing news entry.
    async fn update(&self, n: &News) -> StorageResult<()>;
    /// Soft-delete a news entry.
    async fn delete(&self, id: NewsId) -> StorageResult<()>;
    /// Atomically increment the view count for a news entry.
    async fn increment_view(&self, id: NewsId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_news_object_safe() {
    fn _f(_: Box<dyn NewsRepository>) {}
}

/// Repository port for [`NewsCategory`](crate::aggregate::NewsCategory).
#[async_trait]
pub trait NewsCategoryRepository: Send + Sync {
    /// Fetch a category by its typed id.
    async fn get(&self, id: NewsCategoryId) -> StorageResult<Option<NewsCategory>>;
    /// List categories for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: NewsCategoryQuery,
    ) -> StorageResult<Vec<NewsCategory>>;
    /// Find a category by name (exact match).
    async fn find_by_name(
        &self,
        school: SchoolId,
        name: &str,
    ) -> StorageResult<Option<NewsCategory>>;
    /// Insert a new category.
    async fn insert(&self, c: &NewsCategory) -> StorageResult<()>;
    /// Update an existing category.
    async fn update(&self, c: &NewsCategory) -> StorageResult<()>;
    /// Soft-delete a category.
    async fn delete(&self, id: NewsCategoryId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_news_category_object_safe() {
    fn _f(_: Box<dyn NewsCategoryRepository>) {}
}

/// Repository port for [`NewsComment`](crate::aggregate::NewsComment).
#[async_trait]
pub trait NewsCommentRepository: Send + Sync {
    /// Fetch a comment by its typed id.
    async fn get(&self, id: NewsCommentId) -> StorageResult<Option<NewsComment>>;
    /// List comments for a news entry.
    async fn list_for_news(&self, news: NewsId) -> StorageResult<Vec<NewsComment>>;
    /// List pending moderation comments for a school.
    async fn list_pending(&self, school: SchoolId) -> StorageResult<Vec<NewsComment>>;
    /// List comments matching the typed query.
    async fn list(&self, school: SchoolId, q: NewsCommentQuery) -> StorageResult<Vec<NewsComment>>;
    /// Insert a new comment.
    async fn insert(&self, c: &NewsComment) -> StorageResult<()>;
    /// Update an existing comment.
    async fn update(&self, c: &NewsComment) -> StorageResult<()>;
    /// Soft-delete a comment.
    async fn delete(&self, id: NewsCommentId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_news_comment_object_safe() {
    fn _f(_: Box<dyn NewsCommentRepository>) {}
}

/// Repository port for [`NewsPage`](crate::aggregate::NewsPage).
#[async_trait]
pub trait NewsPageRepository: Send + Sync {
    /// Fetch a news landing page by its typed id.
    async fn get(&self, id: NewsPageId) -> StorageResult<Option<NewsPage>>;
    /// Find the active news landing page for a school.
    async fn find_active(&self, school: SchoolId) -> StorageResult<Option<NewsPage>>;
    /// List news pages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: NewsPageQuery) -> StorageResult<Vec<NewsPage>>;
    /// Insert a new news page.
    async fn insert(&self, p: &NewsPage) -> StorageResult<()>;
    /// Update an existing news page.
    async fn update(&self, p: &NewsPage) -> StorageResult<()>;
    /// Soft-delete a news page.
    async fn delete(&self, id: NewsPageId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_news_page_object_safe() {
    fn _f(_: Box<dyn NewsPageRepository>) {}
}

// =============================================================================
// NoticeBoardRepository + TestimonialRepository + HomeSliderRepository +
// SpeechSliderRepository (owner: C)
// =============================================================================

/// Repository port for [`NoticeBoard`](crate::aggregate::NoticeBoard).
#[async_trait]
pub trait NoticeBoardRepository: Send + Sync {
    /// Fetch a notice board by its typed id.
    async fn get(&self, id: NoticeBoardId) -> StorageResult<Option<NoticeBoard>>;
    /// List notice boards for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: NoticeBoardQuery) -> StorageResult<Vec<NoticeBoard>>;
    /// List published notice boards for a school.
    async fn list_published(&self, school: SchoolId) -> StorageResult<Vec<NoticeBoard>>;
    /// List notice boards whose notice date falls within the
    /// inclusive range `[from, to]`.
    async fn list_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> StorageResult<Vec<NoticeBoard>>;
    /// Insert a new notice board.
    async fn insert(&self, n: &NoticeBoard) -> StorageResult<()>;
    /// Update an existing notice board.
    async fn update(&self, n: &NoticeBoard) -> StorageResult<()>;
    /// Soft-delete a notice board.
    async fn delete(&self, id: NoticeBoardId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_notice_board_object_safe() {
    fn _f(_: Box<dyn NoticeBoardRepository>) {}
}

/// Repository port for [`Testimonial`](crate::aggregate::Testimonial).
#[async_trait]
pub trait TestimonialRepository: Send + Sync {
    /// Fetch a testimonial by its typed id.
    async fn get(&self, id: TestimonialId) -> StorageResult<Option<Testimonial>>;
    /// List testimonials for a school.
    async fn list(&self, school: SchoolId, q: TestimonialQuery) -> StorageResult<Vec<Testimonial>>;
    /// Insert a new testimonial.
    async fn insert(&self, t: &Testimonial) -> StorageResult<()>;
    /// Update an existing testimonial.
    async fn update(&self, t: &Testimonial) -> StorageResult<()>;
    /// Soft-delete a testimonial.
    async fn delete(&self, id: TestimonialId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_testimonial_object_safe() {
    fn _f(_: Box<dyn TestimonialRepository>) {}
}

/// Repository port for [`HomeSlider`](crate::aggregate::HomeSlider).
#[async_trait]
pub trait HomeSliderRepository: Send + Sync {
    /// Fetch a home-slider entry by its typed id.
    async fn get(&self, id: HomeSliderId) -> StorageResult<Option<HomeSlider>>;
    /// List home-slider entries for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: HomeSliderQuery) -> StorageResult<Vec<HomeSlider>>;
    /// Insert a new slider entry.
    async fn insert(&self, s: &HomeSlider) -> StorageResult<()>;
    /// Update an existing slider entry.
    async fn update(&self, s: &HomeSlider) -> StorageResult<()>;
    /// Soft-delete a slider entry.
    async fn delete(&self, id: HomeSliderId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_home_slider_object_safe() {
    fn _f(_: Box<dyn HomeSliderRepository>) {}
}

/// Repository port for [`SpeechSlider`](crate::aggregate::SpeechSlider) (CMS-side).
#[async_trait]
pub trait SpeechSliderRepository: Send + Sync {
    /// Fetch a speech-slider entry by its typed id.
    async fn get(&self, id: SpeechSliderId) -> StorageResult<Option<SpeechSlider>>;
    /// List speech-slider entries for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: SpeechSliderQuery,
    ) -> StorageResult<Vec<SpeechSlider>>;
    /// Insert a new speech-slider entry.
    async fn insert(&self, s: &SpeechSlider) -> StorageResult<()>;
    /// Update an existing speech-slider entry.
    async fn update(&self, s: &SpeechSlider) -> StorageResult<()>;
    /// Soft-delete a speech-slider entry.
    async fn delete(&self, id: SpeechSliderId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_speech_slider_object_safe() {
    fn _f(_: Box<dyn SpeechSliderRepository>) {}
}

// =============================================================================
// Content + ContentType + UploadContent (owner: D1)
// =============================================================================

/// Repository port for [`Content`](crate::aggregate::Content).
#[async_trait]
pub trait ContentRepository: Send + Sync {
    /// Fetch a content item by its typed id.
    async fn get(&self, id: ContentId) -> StorageResult<Option<Content>>;
    /// List content items for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: ContentQuery) -> StorageResult<Vec<Content>>;
    /// List content items by content type.
    async fn list_by_type(
        &self,
        school: SchoolId,
        type_id: ContentTypeId,
    ) -> StorageResult<Vec<Content>>;
    /// List content items for a role id.
    async fn list_for_role(&self, school: SchoolId, role: i32) -> StorageResult<Vec<Content>>;
    /// List content items for a class-section pair.
    async fn list_for_class(
        &self,
        school: SchoolId,
        class: ClassId,
        section: Option<crate::value_objects::SectionId>,
    ) -> StorageResult<Vec<Content>>;
    /// Insert a new content item.
    async fn insert(&self, c: &Content) -> StorageResult<()>;
    /// Update an existing content item.
    async fn update(&self, c: &Content) -> StorageResult<()>;
    /// Soft-delete a content item.
    async fn delete(&self, id: ContentId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_content_object_safe() {
    fn _f(_: Box<dyn ContentRepository>) {}
}

/// Repository port for [`ContentType`](crate::aggregate::ContentType).
#[async_trait]
pub trait ContentTypeRepository: Send + Sync {
    /// Fetch a content type by its typed id.
    async fn get(&self, id: ContentTypeId) -> StorageResult<Option<ContentType>>;
    /// List content types for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: ContentTypeQuery) -> StorageResult<Vec<ContentType>>;
    /// Find a content type by name.
    async fn find_by_name(
        &self,
        school: SchoolId,
        name: &str,
    ) -> StorageResult<Option<ContentType>>;
    /// Insert a new content type.
    async fn insert(&self, t: &ContentType) -> StorageResult<()>;
    /// Update an existing content type.
    async fn update(&self, t: &ContentType) -> StorageResult<()>;
    /// Soft-delete a content type.
    async fn delete(&self, id: ContentTypeId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_content_type_object_safe() {
    fn _f(_: Box<dyn ContentTypeRepository>) {}
}

/// Repository port for [`UploadContent`](crate::aggregate::UploadContent).
#[async_trait]
pub trait UploadContentRepository: Send + Sync {
    /// Fetch an admin-uploaded content item by its typed id.
    async fn get(&self, id: UploadContentId) -> StorageResult<Option<UploadContent>>;
    /// List admin-uploaded content items for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: UploadContentQuery,
    ) -> StorageResult<Vec<UploadContent>>;
    /// List admin-uploaded content items for a role id.
    async fn list_for_role(&self, school: SchoolId, role: i32)
        -> StorageResult<Vec<UploadContent>>;
    /// List admin-uploaded content items for a class-section pair.
    async fn list_for_class(
        &self,
        school: SchoolId,
        class: i32,
        section: Option<i32>,
    ) -> StorageResult<Vec<UploadContent>>;
    /// Insert a new admin-uploaded content item.
    async fn insert(&self, c: &UploadContent) -> StorageResult<()>;
    /// Update an existing admin-uploaded content item.
    async fn update(&self, c: &UploadContent) -> StorageResult<()>;
    /// Soft-delete an admin-uploaded content item.
    async fn delete(&self, id: UploadContentId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_upload_content_object_safe() {
    fn _f(_: Box<dyn UploadContentRepository>) {}
}

// =============================================================================
// ContentShareList + TeacherUploadContent (owner: D2)
// =============================================================================

/// Repository port for [`ContentShareList`](crate::aggregate::ContentShareList).
#[async_trait]
pub trait ContentShareListRepository: Send + Sync {
    /// Fetch a content share list by its typed id.
    async fn get(&self, id: ContentShareListId) -> StorageResult<Option<ContentShareList>>;
    /// List content share lists for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: ContentShareListQuery,
    ) -> StorageResult<Vec<ContentShareList>>;
    /// List active share lists for the school on the given date.
    async fn list_active(
        &self,
        school: SchoolId,
        on: NaiveDate,
    ) -> StorageResult<Vec<ContentShareList>>;
    /// Insert a new share list.
    async fn insert(&self, l: &ContentShareList) -> StorageResult<()>;
    /// Update an existing share list.
    async fn update(&self, l: &ContentShareList) -> StorageResult<()>;
    /// Soft-delete a share list.
    async fn delete(&self, id: ContentShareListId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_content_share_list_object_safe() {
    fn _f(_: Box<dyn ContentShareListRepository>) {}
}

/// Repository port for [`TeacherUploadContent`](crate::aggregate::TeacherUploadContent).
#[async_trait]
pub trait TeacherUploadContentRepository: Send + Sync {
    /// Fetch a teacher-uploaded content item by its typed id.
    async fn get(&self, id: TeacherUploadContentId) -> StorageResult<Option<TeacherUploadContent>>;
    /// List teacher-uploaded content items for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: TeacherUploadContentQuery,
    ) -> StorageResult<Vec<TeacherUploadContent>>;
    /// List teacher-uploaded content items for a class-section pair.
    async fn list_for_class(
        &self,
        class: ClassId,
        section: crate::value_objects::SectionId,
    ) -> StorageResult<Vec<TeacherUploadContent>>;
    /// List teacher-uploaded content items for a teacher.
    async fn list_for_teacher(
        &self,
        teacher: educore_core::ids::UserId,
    ) -> StorageResult<Vec<TeacherUploadContent>>;
    /// Insert a new teacher-uploaded content item.
    async fn insert(&self, c: &TeacherUploadContent) -> StorageResult<()>;
    /// Update an existing teacher-uploaded content item.
    async fn update(&self, c: &TeacherUploadContent) -> StorageResult<()>;
    /// Soft-delete a teacher-uploaded content item.
    async fn delete(&self, id: TeacherUploadContentId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_teacher_upload_content_object_safe() {
    fn _f(_: Box<dyn TeacherUploadContentRepository>) {}
}

// =============================================================================
// Per-page template repositories (owner: E)
// =============================================================================

/// Repository port for [`AboutPage`](crate::aggregate::AboutPage).
#[async_trait]
pub trait AboutPageRepository: Send + Sync {
    /// Fetch an about page by its typed id.
    async fn get(&self, id: AboutPageId) -> StorageResult<Option<AboutPage>>;
    /// Find the active about page for a school.
    async fn find_active(&self, school: SchoolId) -> StorageResult<Option<AboutPage>>;
    /// List about pages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: AboutPageQuery) -> StorageResult<Vec<AboutPage>>;
    /// Insert a new about page.
    async fn insert(&self, p: &AboutPage) -> StorageResult<()>;
    /// Update an existing about page.
    async fn update(&self, p: &AboutPage) -> StorageResult<()>;
    /// Soft-delete an about page.
    async fn delete(&self, id: AboutPageId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_about_page_object_safe() {
    fn _f(_: Box<dyn AboutPageRepository>) {}
}

/// Repository port for [`ContactPage`](crate::aggregate::ContactPage).
#[async_trait]
pub trait ContactPageRepository: Send + Sync {
    /// Fetch a contact page by its typed id.
    async fn get(&self, id: ContactPageId) -> StorageResult<Option<ContactPage>>;
    /// Find the active contact page for a school.
    async fn find_active(&self, school: SchoolId) -> StorageResult<Option<ContactPage>>;
    /// List contact pages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: ContactPageQuery) -> StorageResult<Vec<ContactPage>>;
    /// Insert a new contact page.
    async fn insert(&self, p: &ContactPage) -> StorageResult<()>;
    /// Update an existing contact page.
    async fn update(&self, p: &ContactPage) -> StorageResult<()>;
    /// Soft-delete a contact page.
    async fn delete(&self, id: ContactPageId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_contact_page_object_safe() {
    fn _f(_: Box<dyn ContactPageRepository>) {}
}

/// Repository port for [`CoursePage`](crate::aggregate::CoursePage).
#[async_trait]
pub trait CoursePageRepository: Send + Sync {
    /// Fetch a course page by its typed id.
    async fn get(&self, id: CoursePageId) -> StorageResult<Option<CoursePage>>;
    /// Find a course page by id.
    async fn find_active(&self, school: SchoolId) -> StorageResult<Option<CoursePage>>;
    /// List course pages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: CoursePageQuery) -> StorageResult<Vec<CoursePage>>;
    /// Insert a new course page.
    async fn insert(&self, p: &CoursePage) -> StorageResult<()>;
    /// Update an existing course page.
    async fn update(&self, p: &CoursePage) -> StorageResult<()>;
    /// Soft-delete a course page.
    async fn delete(&self, id: CoursePageId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_course_page_object_safe() {
    fn _f(_: Box<dyn CoursePageRepository>) {}
}

/// Repository port for [`HomePageSetting`](crate::aggregate::HomePageSetting).
#[async_trait]
pub trait HomePageSettingRepository: Send + Sync {
    /// Fetch a home-page setting by its typed id.
    async fn get(&self, id: HomePageSettingId) -> StorageResult<Option<HomePageSetting>>;
    /// Find the active home-page setting for a school.
    async fn find_active(&self, school: SchoolId) -> StorageResult<Option<HomePageSetting>>;
    /// List home-page settings for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: HomePageSettingQuery,
    ) -> StorageResult<Vec<HomePageSetting>>;
    /// Insert a new home-page setting.
    async fn insert(&self, p: &HomePageSetting) -> StorageResult<()>;
    /// Update an existing home-page setting.
    async fn update(&self, p: &HomePageSetting) -> StorageResult<()>;
    /// Soft-delete a home-page setting.
    async fn delete(&self, id: HomePageSettingId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_home_page_setting_object_safe() {
    fn _f(_: Box<dyn HomePageSettingRepository>) {}
}

/// Repository port for [`FrontendPage`](crate::aggregate::FrontendPage).
#[async_trait]
pub trait FrontendPageRepository: Send + Sync {
    /// Fetch a front-end page by its typed id.
    async fn get(&self, id: FrontendPageId) -> StorageResult<Option<FrontendPage>>;
    /// Find the active front-end page by slug (when set).
    async fn find_active_by_slug(
        &self,
        school: SchoolId,
        slug: &crate::value_objects::Slug,
    ) -> StorageResult<Option<FrontendPage>>;
    /// List front-end pages for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: FrontendPageQuery,
    ) -> StorageResult<Vec<FrontendPage>>;
    /// Insert a new front-end page.
    async fn insert(&self, p: &FrontendPage) -> StorageResult<()>;
    /// Update an existing front-end page.
    async fn update(&self, p: &FrontendPage) -> StorageResult<()>;
    /// Soft-delete a front-end page.
    async fn delete(&self, id: FrontendPageId) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_frontend_page_object_safe() {
    fn _f(_: Box<dyn FrontendPageRepository>) {}
}

// =============================================================================
// Command-shaped aggregate repositories (owner: F)
// =============================================================================
//
// These repositories persist the New*/Update* command-shaped
// aggregates that the spec carves out as standalone persisted
// commands. Each carries its own typed id (e.g. `NewHomeSliderId`)
// for the get/insert/update operations. The shape is minimal
// CRUD: get, list_for_school, insert, update. The parent
// aggregates (HomeSlider, SpeechSlider, Testimonial, News, ...)
// already have their own Repository port traits.

/// Repository port for the [`NewHomeSlider`](crate::aggregate::NewHomeSlider)
/// command. Persists the home-slider create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewHomeSliderRepository: Send + Sync {
    /// Fetch a new-home-slider entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewHomeSliderId) -> StorageResult<Option<NewHomeSlider>>;
    /// List new-home-slider entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewHomeSlider>>;
    /// Insert a new home-slider command.
    async fn insert(&self, h: &NewHomeSlider) -> StorageResult<()>;
    /// Update an existing home-slider command.
    async fn update(&self, h: &NewHomeSlider) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_home_slider_object_safe() {
    fn _f(_: Box<dyn NewHomeSliderRepository>) {}
}

/// Repository port for the [`NewSpeechSlider`](crate::aggregate::NewSpeechSlider)
/// command. Persists the speech-slider create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewSpeechSliderRepository: Send + Sync {
    /// Fetch a new-speech-slider entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewSpeechSliderId) -> StorageResult<Option<NewSpeechSlider>>;
    /// List new-speech-slider entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewSpeechSlider>>;
    /// Insert a new speech-slider command.
    async fn insert(&self, h: &NewSpeechSlider) -> StorageResult<()>;
    /// Update an existing speech-slider command.
    async fn update(&self, h: &NewSpeechSlider) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_speech_slider_object_safe() {
    fn _f(_: Box<dyn NewSpeechSliderRepository>) {}
}

/// Repository port for the [`NewTestimonial`](crate::aggregate::NewTestimonial)
/// command. Persists the testimonial create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewTestimonialRepository: Send + Sync {
    /// Fetch a new-testimonial entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewTestimonialId) -> StorageResult<Option<NewTestimonial>>;
    /// List new-testimonial entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewTestimonial>>;
    /// Insert a new testimonial command.
    async fn insert(&self, h: &NewTestimonial) -> StorageResult<()>;
    /// Update an existing testimonial command.
    async fn update(&self, h: &NewTestimonial) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_testimonial_object_safe() {
    fn _f(_: Box<dyn NewTestimonialRepository>) {}
}

/// Repository port for the [`NewNews`](crate::aggregate::NewNews)
/// command. Persists the news create command. Minimal CRUD
/// interface.
#[async_trait]
pub trait NewNewsRepository: Send + Sync {
    /// Fetch a new-news entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewNewsId) -> StorageResult<Option<NewNews>>;
    /// List new-news entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewNews>>;
    /// Insert a new news command.
    async fn insert(&self, h: &NewNews) -> StorageResult<()>;
    /// Update an existing news command.
    async fn update(&self, h: &NewNews) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_news_object_safe() {
    fn _f(_: Box<dyn NewNewsRepository>) {}
}

/// Repository port for the [`NewNewsCategory`](crate::aggregate::NewNewsCategory)
/// command. Persists the news-category create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewNewsCategoryRepository: Send + Sync {
    /// Fetch a new-news-category entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewNewsCategoryId) -> StorageResult<Option<NewNewsCategory>>;
    /// List new-news-category entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewNewsCategory>>;
    /// Insert a new news-category command.
    async fn insert(&self, h: &NewNewsCategory) -> StorageResult<()>;
    /// Update an existing news-category command.
    async fn update(&self, h: &NewNewsCategory) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_news_category_object_safe() {
    fn _f(_: Box<dyn NewNewsCategoryRepository>) {}
}

/// Repository port for the [`NewNewsComment`](crate::aggregate::NewNewsComment)
/// command. Persists the news-comment create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewNewsCommentRepository: Send + Sync {
    /// Fetch a new-news-comment entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewNewsCommentId) -> StorageResult<Option<NewNewsComment>>;
    /// List new-news-comment entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewNewsComment>>;
    /// Insert a new news-comment command.
    async fn insert(&self, h: &NewNewsComment) -> StorageResult<()>;
    /// Update an existing news-comment command.
    async fn update(&self, h: &NewNewsComment) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_news_comment_object_safe() {
    fn _f(_: Box<dyn NewNewsCommentRepository>) {}
}

/// Repository port for the [`NewNoticeBoard`](crate::aggregate::NewNoticeBoard)
/// command. Persists the notice-board create command. Minimal
/// CRUD interface.
#[async_trait]
pub trait NewNoticeBoardRepository: Send + Sync {
    /// Fetch a new-notice-board entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: NewNoticeBoardId) -> StorageResult<Option<NewNoticeBoard>>;
    /// List new-notice-board entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<NewNoticeBoard>>;
    /// Insert a new notice-board command.
    async fn insert(&self, h: &NewNoticeBoard) -> StorageResult<()>;
    /// Update an existing notice-board command.
    async fn update(&self, h: &NewNoticeBoard) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_new_notice_board_object_safe() {
    fn _f(_: Box<dyn NewNoticeBoardRepository>) {}
}

/// Repository port for the [`UpdateNews`](crate::aggregate::UpdateNews)
/// command. Persists the news update command. Minimal CRUD
/// interface.
#[async_trait]
pub trait UpdateNewsRepository: Send + Sync {
    /// Fetch an update-news entry by its typed id. Returns
    /// `Ok(None)` if not found or soft-deleted.
    async fn get(&self, id: UpdateNewsId) -> StorageResult<Option<UpdateNews>>;
    /// List update-news entries for a school.
    async fn list_for_school(&self, school: SchoolId) -> StorageResult<Vec<UpdateNews>>;
    /// Insert an update-news command.
    async fn insert(&self, h: &UpdateNews) -> StorageResult<()>;
    /// Update an existing update-news command.
    async fn update(&self, h: &UpdateNews) -> StorageResult<()>;
}

/// Object-safety smoke test.
fn _assert_update_news_object_safe() {
    fn _f(_: Box<dyn UpdateNewsRepository>) {}
}
