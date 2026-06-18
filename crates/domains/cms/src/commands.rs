//! CMS-domain typed command shapes.
//!
//! Per `docs/specs/cms/commands.md`, the CMS domain ships ~60
//! typed command shapes (3 per aggregate avg). Each command
//! carries a `TenantContext` and is rejected if the actor lacks
//! the required capability. The wire form is
//! `cms.<aggregate>.<verb>`.
//!
//! Phase 12 ships the typed command shapes + their
//! `into_new_*` helpers that convert to the aggregate-local
//! `New*` inputs. The async service factory fns in `services.rs`
//! wire these to the engine's ports.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use educore_academic::{AcademicYearId, ClassId, SectionId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregate::{
    NewContent, NewContentShareList, NewHomePageSetting, NewHomeSlider, NewNews, NewPage,
    NewTestimonial,
};
use crate::value_objects::{
    ContentShareListTitle, Designation, FileReference, HomePageLongTitle, HomePageShortDescription,
    HomePageTitle, HomeSliderLinkLabel, InstitutionName, NewsBody, NewsTitle, PageDescription,
    PageSettings, PageTitle, PersonName, PublishDate, ShareDate, Slug, StarRating,
    TestimonialDescription, Url, ValidUntil, YoutubeLink,
};

// =============================================================================
// Command type constants
// =============================================================================

const CMS_PAGE_CREATE_COMMAND_TYPE: &str = "cms.page.create";
const CMS_PAGE_PUBLISH_COMMAND_TYPE: &str = "cms.page.publish";
const CMS_PAGE_ARCHIVE_COMMAND_TYPE: &str = "cms.page.archive";
const CMS_PAGE_DELETE_COMMAND_TYPE: &str = "cms.page.delete";

const CMS_NEWS_CREATE_COMMAND_TYPE: &str = "cms.news.create";

const CMS_TESTIMONIAL_CREATE_COMMAND_TYPE: &str = "cms.testimonial.create";
const CMS_HOME_SLIDER_CREATE_COMMAND_TYPE: &str = "cms.home_slider.create";
const CMS_CONTENT_CREATE_COMMAND_TYPE: &str = "cms.content.create";
const CMS_CONTENT_SHARE_LIST_CREATE_COMMAND_TYPE: &str = "cms.content_share_list.create";
const CMS_HOME_PAGE_SETTING_CONFIGURE_COMMAND_TYPE: &str = "cms.home_page_setting.configure";

// =============================================================================
// Page commands (4)
// =============================================================================

/// Create a new [`Page`](crate::aggregate::Page).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatePageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The page title.
    pub title: PageTitle,
    /// The optional description.
    pub description: Option<PageDescription>,
    /// The optional slug.
    pub slug: Option<Slug>,
    /// The optional settings.
    pub settings: Option<PageSettings>,
    /// Whether this is the home page.
    pub home_page: bool,
    /// Whether this is a default template.
    pub is_default: bool,
}

impl CreatePageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_PAGE_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewPage`].
    #[must_use]
    pub fn into_new_page(self, id: crate::value_objects::PageId) -> NewPage {
        NewPage {
            id,
            title: self.title,
            description: self.description,
            slug: self.slug,
            settings: self.settings,
            home_page: crate::value_objects::HomePage::new(self.home_page),
            is_default: crate::value_objects::IsDefault::new(self.is_default),
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Publish a [`Page`](crate::aggregate::Page).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishPageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The page id.
    pub page_id: crate::value_objects::PageId,
}

impl PublishPageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_PAGE_PUBLISH_COMMAND_TYPE;
}

/// Archive a [`Page`](crate::aggregate::Page).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchivePageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The page id.
    pub page_id: crate::value_objects::PageId,
}

impl ArchivePageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_PAGE_ARCHIVE_COMMAND_TYPE;
}

/// Soft-delete a [`Page`](crate::aggregate::Page).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The page id.
    pub page_id: crate::value_objects::PageId,
}

impl DeletePageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_PAGE_DELETE_COMMAND_TYPE;
}

// =============================================================================
// News command (1 — the headline for the Phase 12 service-factory fn set)
// =============================================================================

/// Create a new [`News`](crate::aggregate::News).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateNewsCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The news title.
    pub news_title: NewsTitle,
    /// The category id.
    pub category_id: crate::value_objects::NewsCategoryId,
    /// The optional image.
    pub image: Option<FileReference>,
    /// The optional image thumbnail.
    pub image_thumb: Option<FileReference>,
    /// The news body.
    pub news_body: NewsBody,
    /// The publish date.
    pub publish_date: PublishDate,
    /// Whether the news is global.
    pub is_global: bool,
    /// Whether comments are auto-approved.
    pub auto_approve: bool,
    /// Whether comments are enabled.
    pub is_comment: bool,
    /// The optional explicit ordering string.
    pub order: Option<String>,
}

impl CreateNewsCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_NEWS_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewNews`].
    #[must_use]
    pub fn into_new_news(self, id: crate::value_objects::NewsId) -> NewNews {
        NewNews {
            id,
            news_title: self.news_title,
            category_id: self.category_id,
            image: self.image,
            image_thumb: self.image_thumb,
            news_body: self.news_body,
            publish_date: self.publish_date,
            is_global: crate::value_objects::IsGlobal::new(self.is_global),
            auto_approve: crate::value_objects::AutoApprove::new(self.auto_approve),
            is_comment: crate::value_objects::IsComment::new(self.is_comment),
            order: self.order,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

// =============================================================================
// Testimonial command (1)
// =============================================================================

/// Create a new [`Testimonial`](crate::aggregate::Testimonial).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTestimonialCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The person's name.
    pub name: PersonName,
    /// The designation.
    pub designation: Designation,
    /// The institution name.
    pub institution_name: InstitutionName,
    /// The image file.
    pub image: FileReference,
    /// The description.
    pub description: TestimonialDescription,
    /// The star rating.
    pub star_rating: StarRating,
}

impl CreateTestimonialCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_TESTIMONIAL_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewTestimonial`].
    #[must_use]
    pub fn into_new_testimonial(self, id: crate::value_objects::TestimonialId) -> NewTestimonial {
        NewTestimonial {
            id,
            name: self.name,
            designation: self.designation,
            institution_name: self.institution_name,
            image: self.image,
            description: self.description,
            star_rating: self.star_rating,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

// =============================================================================
// HomeSlider command (1)
// =============================================================================

/// Create a new [`HomeSlider`](crate::aggregate::HomeSlider).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateHomeSliderCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The image file.
    pub image: FileReference,
    /// The optional link URL.
    pub link: Option<Url>,
    /// The optional link label.
    pub link_label: Option<HomeSliderLinkLabel>,
}

impl CreateHomeSliderCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_HOME_SLIDER_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewHomeSlider`].
    #[must_use]
    pub fn into_new_home_slider(self, id: crate::value_objects::HomeSliderId) -> NewHomeSlider {
        NewHomeSlider {
            id,
            image: self.image,
            link: self.link,
            link_label: self.link_label,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

// =============================================================================
// Content command (1)
// =============================================================================

/// Create a new [`Content`](crate::aggregate::Content).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateContentCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The file name.
    pub file_name: String,
    /// The file size in bytes.
    pub file_size: i64,
    /// The content type id.
    pub content_type_id: crate::value_objects::ContentTypeId,
    /// The optional YouTube link.
    pub youtube_link: Option<YoutubeLink>,
    /// The optional file reference.
    pub upload_file: Option<FileReference>,
    /// The optional role scope id.
    pub available_for_role: Option<i32>,
    /// The optional class scope id.
    pub available_for_class: Option<i32>,
    /// The optional section scope id.
    pub available_for_section: Option<i32>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
}

impl CreateContentCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_CONTENT_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewContent`].
    #[must_use]
    pub fn into_new_content(self, id: crate::value_objects::ContentId) -> NewContent {
        NewContent {
            id,
            file_name: self.file_name,
            file_size: self.file_size,
            content_type_id: self.content_type_id,
            youtube_link: self.youtube_link,
            upload_file: self.upload_file,
            available_for_role: self.available_for_role,
            available_for_class: self.available_for_class,
            available_for_section: self.available_for_section,
            academic_id: self.academic_id,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

// =============================================================================
// ContentShareList command (1)
// =============================================================================

/// Create a new [`ContentShareList`](crate::aggregate::ContentShareList).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateContentShareListCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The share list title.
    pub title: ContentShareListTitle,
    /// The share date.
    pub share_date: ShareDate,
    /// The valid-upto date.
    pub valid_upto: ValidUntil,
    /// The optional description.
    pub description: Option<String>,
    /// The send type.
    pub send_type: crate::value_objects::ContentShareType,
    /// The content ids being shared.
    pub content_ids: Vec<crate::value_objects::ContentId>,
    /// The optional role ids.
    pub gr_role_ids: Option<Vec<Uuid>>,
    /// The optional user ids.
    pub ind_user_ids: Option<Vec<Uuid>>,
    /// The optional class id.
    pub class_id: Option<ClassId>,
    /// The optional section ids.
    pub section_ids: Option<Vec<SectionId>>,
    /// The optional URL.
    pub url: Option<Url>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
}

impl CreateContentShareListCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_CONTENT_SHARE_LIST_CREATE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewContentShareList`].
    #[must_use]
    pub fn into_new_content_share_list(
        self,
        id: crate::value_objects::ContentShareListId,
    ) -> NewContentShareList {
        NewContentShareList {
            id,
            title: self.title,
            share_date: self.share_date,
            valid_upto: self.valid_upto,
            description: self.description,
            send_type: self.send_type,
            content_ids: self.content_ids,
            gr_role_ids: self.gr_role_ids,
            ind_user_ids: self.ind_user_ids,
            class_id: self.class_id,
            section_ids: self.section_ids,
            url: self.url,
            academic_id: self.academic_id,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

// =============================================================================
// HomePageSetting command (1 — the create-or-update semantics)
// =============================================================================

/// Configure (create-or-update) the [`HomePageSetting`](crate::aggregate::HomePageSetting).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureHomePageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The title.
    pub title: HomePageTitle,
    /// The optional long title.
    pub long_title: Option<HomePageLongTitle>,
    /// The optional short description.
    pub short_description: Option<HomePageShortDescription>,
    /// The optional link label.
    pub link_label: Option<HomeSliderLinkLabel>,
    /// The optional link URL.
    pub link_url: Option<Url>,
    /// The optional image.
    pub image: Option<FileReference>,
}

impl ConfigureHomePageCommand {
    /// Wire-form command type.
    pub const COMMAND_TYPE: &'static str = CMS_HOME_PAGE_SETTING_CONFIGURE_COMMAND_TYPE;
    /// Converts to the aggregate-local [`NewHomePageSetting`].
    #[must_use]
    pub fn into_new_home_page_setting(
        self,
        id: crate::value_objects::HomePageSettingId,
    ) -> NewHomePageSetting {
        NewHomePageSetting {
            id,
            title: self.title,
            long_title: self.long_title,
            short_description: self.short_description,
            link_label: self.link_label,
            link_url: self.link_url,
            image: self.image,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
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
    fn create_page_command_type_is_stable() {
        assert_eq!(CreatePageCommand::COMMAND_TYPE, "cms.page.create");
    }

    #[test]
    fn publish_page_command_type_is_stable() {
        assert_eq!(PublishPageCommand::COMMAND_TYPE, "cms.page.publish");
    }

    #[test]
    fn archive_page_command_type_is_stable() {
        assert_eq!(ArchivePageCommand::COMMAND_TYPE, "cms.page.archive");
    }

    #[test]
    fn delete_page_command_type_is_stable() {
        assert_eq!(DeletePageCommand::COMMAND_TYPE, "cms.page.delete");
    }

    #[test]
    fn create_news_command_type_is_stable() {
        assert_eq!(CreateNewsCommand::COMMAND_TYPE, "cms.news.create");
    }

    #[test]
    fn create_testimonial_command_type_is_stable() {
        assert_eq!(
            CreateTestimonialCommand::COMMAND_TYPE,
            "cms.testimonial.create"
        );
    }

    #[test]
    fn create_home_slider_command_type_is_stable() {
        assert_eq!(
            CreateHomeSliderCommand::COMMAND_TYPE,
            "cms.home_slider.create"
        );
    }

    #[test]
    fn create_content_command_type_is_stable() {
        assert_eq!(CreateContentCommand::COMMAND_TYPE, "cms.content.create");
    }

    #[test]
    fn create_content_share_list_command_type_is_stable() {
        assert_eq!(
            CreateContentShareListCommand::COMMAND_TYPE,
            "cms.content_share_list.create"
        );
    }

    #[test]
    fn configure_home_page_command_type_is_stable() {
        assert_eq!(
            ConfigureHomePageCommand::COMMAND_TYPE,
            "cms.home_page_setting.configure"
        );
    }

    // Anchor for unused imports in test module.
    #[allow(dead_code)]
    fn _anchor(_: educore_core::ids::SchoolId, _: educore_core::ids::UserId) {}
}
