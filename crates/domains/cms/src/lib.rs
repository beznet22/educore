//! # educore-cms
//!
//!  Website pages, news, content uploads, testimonials, sliders, course pages.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/cms/` for behavioral details.

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
pub const PACKAGE_NAME: &str = "educore-cms";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Convenient prelude: the public surface of the CMS crate.
#[allow(missing_docs)]
pub mod prelude {
    // 20 headline aggregate roots.
    pub use crate::aggregate::{
        AboutPage, ContactPage, Content, ContentShareList, ContentType, CoursePage, FrontendPage,
        HomePageSetting, HomeSlider, News, NewsCategory, NewsComment, NewsPage, NoticeBoard, Page,
        SpeechSlider, TeacherUploadContent, Testimonial, UploadContent,
    };

    // 6 child entities.
    pub use crate::entities::{
        ContentRevision, ContentShareListAudience, ContentShareListContent, NewsImage,
        NewsRevision, PageRevision,
    };

    // 73 typed events (alphabetised).
    pub use crate::events::{
        AboutPageCreated, AboutPageDeleted, AboutPageUpdated, ContactPageCreated,
        ContactPageDeleted, ContactPageUpdated, ContentCreated, ContentDeleted,
        ContentShareListCancelled, ContentShareListCreated, ContentShareListDeleted,
        ContentShareListDispatched, ContentShareListUpdated, ContentTypeCreated,
        ContentTypeDeleted, ContentTypeUpdated, ContentUpdated, CoursePageCreated,
        CoursePageDeleted, CoursePageUpdated, FrontendPageCreated, FrontendPageDeleted,
        FrontendPageUpdated, HomePageSettingConfigured, HomePageSettingDeleted,
        HomePageSettingUpdated, HomeSliderCreated, HomeSliderDeleted, HomeSliderUpdated,
        NewsCategoryCreated, NewsCategoryDeleted, NewsCategoryUpdated, NewsCommentAdded,
        NewsCommentApproved, NewsCommentDeleted, NewsCommentHidden, NewsCreated, NewsDeleted,
        NewsPageCreated, NewsPageDeleted, NewsPageUpdated, NewsPublished, NewsUnpublished,
        NewsUpdated, NewsViewIncremented, NoticeBoardCreated, NoticeBoardDeleted,
        NoticeBoardPublished, NoticeBoardUnpublished, NoticeBoardUpdated, PageArchived,
        PageCreated, PageDeleted, PagePublished, PageUpdated, SpeechSliderCreated,
        SpeechSliderDeleted, SpeechSliderUpdated, TeacherUploadContentCreated,
        TeacherUploadContentDeleted, TeacherUploadContentUpdated, TestimonialCreated,
        TestimonialDeleted, TestimonialUpdated, UploadContentCreated, UploadContentDeleted,
        UploadContentUpdated,
    };

    // 20 typed query stubs.
    pub use crate::query::{
        AboutPageQuery, ContactPageQuery, ContentQuery, ContentShareListQuery, ContentTypeQuery,
        CoursePageQuery, FrontendPageQuery, HomePageSettingQuery, HomeSliderQuery,
        NewsCategoryQuery, NewsCommentQuery, NewsPageQuery, NewsQuery, NoticeBoardQuery, PageQuery,
        SpeechSliderQuery, TeacherUploadContentQuery, TestimonialQuery, UploadContentQuery,
    };

    // 20 repository port traits.
    pub use crate::repository::{
        AboutPageRepository, ContactPageRepository, ContentRepository, ContentShareListRepository,
        ContentTypeRepository, CoursePageRepository, FrontendPageRepository,
        HomePageSettingRepository, HomeSliderRepository, NewsCategoryRepository,
        NewsCommentRepository, NewsPageRepository, NewsRepository, NoticeBoardRepository,
        PageRepository, SpeechSliderRepository, TeacherUploadContentRepository,
        TestimonialRepository, UploadContentRepository,
    };

    // 6 service factory functions + 6 service structs.
    pub use crate::services::{
        configure_home_page_service, content_service, content_share_list_service,
        create_news_service, create_page_service, create_testimonial_service,
        form_uploaded_public_indexing_subscriber, ContentService, ContentShareListService,
        FormIndexAction, HomeSliderService, NewsService, PageService, TestimonialService,
    };

    // 20 typed root ids + 6 child ids (re-exported for the public surface).
    pub use crate::value_objects::{
        AboutPageId, ClassId, ContactPageId, ContentId, ContentRevisionId,
        ContentShareListAudienceId, ContentShareListContentId, ContentShareListId, ContentTypeId,
        CoursePageId, FrontendPageId, HomePageSettingId, HomeSliderId, NewsCategoryId,
        NewsCommentId, NewsId, NewsImageId, NewsPageId, NewsRevisionId, NoticeBoardId, PageId,
        PageRevisionId, SectionId, SpeechSliderId, TeacherUploadContentId, TestimonialId,
        UploadContentId,
    };

    // Value objects.
    pub use crate::value_objects::{
        ActiveStatus, AudienceDescriptor, AutoApprove, AvailableForAdmin, AvailableForAllClasses,
        ButtonText, ButtonUrl, CategoryName, CommentMessage, ContentDescription,
        ContentShareListStatus, ContentShareListTitle, ContentShareType, ContentTitle,
        ContentTypeName, CoursePageDescription, CoursePageTitle, Designation, EmailAddress,
        FileReference, GoogleMapAddress, HomePageLongTitle, HomePageShortDescription,
        HomePageTitle, HomeSliderLinkLabel, InstitutionName, IsComment, IsDefault, IsDynamic,
        IsGlobal, IsParent, IsPublished, Latitude, Longitude, NewsBody, NewsCommentStatus,
        NewsStatus, NewsTitle, NoticeDate, NoticeMessage, NoticeTitle, PageDescription,
        PageSettings, PageStatus, PageSubTitle, PageTitle, PersonName, PhoneNumber, PostalAddress,
        PublishDate, ShareDate, Slug, SourceUrl, SpeechText, StarRating, TeacherContentType,
        TestimonialDescription, UploadDate, Url, ValidUntil, YoutubeLink, ZoomLevel,
    };

    // Errors.
    pub use crate::errors::{CmsError, Result};

    // Re-export `SchoolId::PUBLIC` for the public-content special case.
    pub use educore_core::ids::PUBLIC_SCHOOL_ID;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-cms");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_exports_expected_symbols() {
        // Smoke test: every headline aggregate root resolves
        // through the prelude. The compiler enforces the names.
        let _: Option<Page> = None;
        let _: Option<News> = None;
        let _: Option<NewsCategory> = None;
        let _: Option<NewsComment> = None;
        let _: Option<NewsPage> = None;
        let _: Option<NoticeBoard> = None;
        let _: Option<Testimonial> = None;
        let _: Option<HomeSlider> = None;
        let _: Option<SpeechSlider> = None;
        let _: Option<Content> = None;
        let _: Option<ContentType> = None;
        let _: Option<ContentShareList> = None;
        let _: Option<TeacherUploadContent> = None;
        let _: Option<UploadContent> = None;
        let _: Option<AboutPage> = None;
        let _: Option<ContactPage> = None;
        let _: Option<CoursePage> = None;
        let _: Option<HomePageSetting> = None;
        let _: Option<FrontendPage> = None;

        let _: Option<NewsImage> = None;
        let _: Option<PageRevision> = None;
        let _: Option<NewsRevision> = None;
        let _: Option<ContentRevision> = None;
        let _: Option<ContentShareListAudience> = None;
        let _: Option<ContentShareListContent> = None;
    }
}
