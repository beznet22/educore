//! # CMS aggregate roots
//!
//! The 20 root aggregates per the spec at
//! `docs/specs/cms/aggregates.md`. Each follows the standard
//! audit-footer pattern (per AGENTS.md):
//!
//! - 1 typed id (e.g. `PageId`) + 1 derived `school_id` anchor
//! - domain fields
//! - audit-metadata fields: `version`, `etag`, `created_at`,
//!   `updated_at`, `created_by`, `updated_by`, `active_status`,
//!   `last_event_id`, `correlation_id`
//!
//! `school_id` is **derived from `id.school_id()`**, never taken
//! from the caller.

#![allow(missing_docs)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_literal_unwrap)]
#![allow(unused_imports)]
#![allow(dead_code)]

use chrono::{DateTime, NaiveDate, Utc};
use educore_academic::{AcademicYearId, ClassId, SectionId};
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::PageRevision;
use crate::errors::CmsError;
use crate::value_objects::*;

// =============================================================================
// Section: Page (root aggregate)
// =============================================================================

/// Aggregate-local input for [`Page::new`]. The wire-level
/// command lives in `commands::CreatePageCommand` and
/// `From`-converts into this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewPage {
    /// The typed id.
    pub id: PageId,
    /// The page title.
    pub title: PageTitle,
    /// The optional description.
    pub description: Option<PageDescription>,
    /// The optional slug.
    pub slug: Option<Slug>,
    /// The optional settings (typed JSON).
    pub settings: Option<PageSettings>,
    /// Whether this is the school's home page.
    pub home_page: HomePage,
    /// Whether this is a pre-installed template.
    pub is_default: IsDefault,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Page::update`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePage {
    /// The new title, if changing.
    pub title: Option<PageTitle>,
    /// The new description, if changing or clearing.
    pub description: Option<Option<PageDescription>>,
    /// The new slug, if changing or clearing.
    pub slug: Option<Option<Slug>>,
    /// The new settings, if changing or clearing.
    pub settings: Option<Option<PageSettings>>,
    /// The new home-page flag, if changing.
    pub home_page: Option<HomePage>,
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// Editable page on the school website. Per spec invariant 1,
/// the title is non-empty; per invariant 2, the slug is unique
/// within `(school_id, slug)`; per invariant 3, the status is
/// `draft` or `published`; per invariant 4, at most one page per
/// school may have `home_page = true`; per invariant 5, default
/// pages are not deletable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// The typed id.
    pub id: PageId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The page title.
    pub title: PageTitle,
    /// The optional description.
    pub description: Option<PageDescription>,
    /// The optional slug.
    pub slug: Option<Slug>,
    /// The optional settings.
    pub settings: Option<PageSettings>,
    /// Whether this is the home page.
    pub home_page: HomePage,
    /// Whether this is a default template.
    pub is_default: IsDefault,
    /// The status.
    pub status: PageStatus,
    /// The soft-delete flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl Page {
    /// Constructs a new `Page` in `Draft` status.
    pub fn new(cmd: NewPage) -> Result<Self, CmsError> {
        if cmd.title.as_str().is_empty() {
            return Err(CmsError::PageTitleEmpty);
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            description: cmd.description,
            slug: cmd.slug,
            settings: cmd.settings,
            home_page: cmd.home_page,
            is_default: cmd.is_default,
            status: PageStatus::default(),
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies changes to the page.
    pub fn update(&mut self, cmd: UpdatePage) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "cannot update a soft-deleted page".to_owned(),
            ));
        }
        if let Some(t) = cmd.title {
            self.title = t;
        }
        if let Some(d) = cmd.description {
            self.description = d;
        }
        if let Some(s) = cmd.slug {
            self.slug = s;
        }
        if let Some(s) = cmd.settings {
            self.settings = s;
        }
        if let Some(h) = cmd.home_page {
            self.home_page = h;
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Transitions the page to `Published` status.
    pub fn publish(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "cannot publish a soft-deleted page".to_owned(),
            ));
        }
        self.status = PageStatus::Published;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Transitions the page to `Draft` status (archive).
    pub fn archive(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "cannot archive a soft-deleted page".to_owned(),
            ));
        }
        self.status = PageStatus::Draft;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Soft-deletes the page. Per spec invariant 5, default
    /// pages (`is_default = true`) cannot be deleted.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "page is already soft-deleted".to_owned(),
            ));
        }
        if self.is_default.is_true() {
            return Err(CmsError::DefaultPageNotDeletable(self.id.as_uuid()));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the page is the school's home page.
    #[must_use]
    pub fn is_home_page(&self) -> bool {
        self.home_page.is_true()
    }

    /// Returns `true` if the page is published.
    #[must_use]
    pub fn is_published(&self) -> bool {
        matches!(self.status, PageStatus::Published)
    }

    /// Returns `true` if the page is active (not soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns the next status for the given action.
    #[must_use]
    pub fn next_status(&self, action: PageStatusAction) -> PageStatus {
        match action {
            PageStatusAction::Publish => PageStatus::Published,
            PageStatusAction::Archive => PageStatus::Draft,
        }
    }
}

/// Action verb for the page-status state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageStatusAction {
    /// Transition to `Published`.
    Publish,
    /// Transition to `Draft`.
    Archive,
}

/// A page revision (historical snapshot).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewPageRevision {
    /// The typed id.
    pub id: PageRevisionId,
    /// The owning page id.
    pub page_id: PageId,
    /// The revision body.
    pub body: String,
    /// The revision number.
    pub revision_number: u32,
    /// The creation timestamp.
    pub created_at: DateTime<Utc>,
    /// The user who created the revision.
    pub created_by: UserId,
}

impl NewPageRevision {
    /// Constructs a `PageRevision` (anchored to the page's
    /// school).
    #[must_use]
    pub fn build(self) -> PageRevision {
        PageRevision {
            id: self.id,
            page_id: self.page_id,
            school_id: self.page_id.school_id(),
            body: self.body,
            revision_number: self.revision_number,
            created_at: self.created_at,
            created_by: self.created_by,
        }
    }
}

// =============================================================================
// Section: News family
// =============================================================================

/// Aggregate-local input for [`News::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewNews {
    /// The typed id.
    pub id: NewsId,
    /// The news title.
    pub news_title: NewsTitle,
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The image.
    pub image: Option<FileReference>,
    /// The image thumbnail.
    pub image_thumb: Option<FileReference>,
    /// The news body.
    pub news_body: NewsBody,
    /// The publish date.
    pub publish_date: PublishDate,
    /// Whether the news is global.
    pub is_global: IsGlobal,
    /// Whether comments are auto-approved.
    pub auto_approve: AutoApprove,
    /// Whether comments are enabled.
    pub is_comment: IsComment,
    /// The optional explicit ordering string.
    pub order: Option<String>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`News::update`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateNews {
    /// The new title, if changing.
    pub news_title: Option<NewsTitle>,
    /// The new category id, if changing.
    pub category_id: Option<NewsCategoryId>,
    /// The new image, if changing or clearing.
    pub image: Option<Option<FileReference>>,
    /// The new image thumb, if changing or clearing.
    pub image_thumb: Option<Option<FileReference>>,
    /// The new body, if changing.
    pub news_body: Option<NewsBody>,
    /// The new publish date, if changing.
    pub publish_date: Option<PublishDate>,
    /// The new is_global, if changing.
    pub is_global: Option<IsGlobal>,
    /// The new auto_approve, if changing.
    pub auto_approve: Option<AutoApprove>,
    /// The new is_comment, if changing.
    pub is_comment: Option<IsComment>,
    /// The new order, if changing or clearing.
    pub order: Option<Option<String>>,
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// A news entry published on the school website.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct News {
    /// The typed id.
    pub id: NewsId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The news title.
    pub news_title: NewsTitle,
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The image.
    pub image: Option<FileReference>,
    /// The image thumbnail.
    pub image_thumb: Option<FileReference>,
    /// The news body.
    pub news_body: NewsBody,
    /// The publish date.
    pub publish_date: PublishDate,
    /// Whether the news is global.
    pub is_global: IsGlobal,
    /// Whether comments are auto-approved.
    pub auto_approve: AutoApprove,
    /// Whether comments are enabled.
    pub is_comment: IsComment,
    /// The optional explicit ordering string.
    pub order: Option<String>,
    /// The active_status flag.
    pub active_status: NewsStatus,
    /// The view count.
    pub view_count: i64,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl News {
    /// Constructs a new `News` in the `Active` status.
    pub fn new(cmd: NewNews) -> Result<Self, CmsError> {
        if cmd.news_title.as_str().is_empty() {
            return Err(CmsError::NewsTitleEmpty);
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            news_title: cmd.news_title,
            category_id: cmd.category_id,
            image: cmd.image,
            image_thumb: cmd.image_thumb,
            news_body: cmd.news_body,
            publish_date: cmd.publish_date,
            is_global: cmd.is_global,
            auto_approve: cmd.auto_approve,
            is_comment: cmd.is_comment,
            order: cmd.order,
            active_status: NewsStatus::Active,
            view_count: 0,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies changes to the news.
    pub fn update(&mut self, cmd: UpdateNews) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "cannot update a disabled news".to_owned(),
            ));
        }
        if let Some(t) = cmd.news_title {
            self.news_title = t;
        }
        if let Some(c) = cmd.category_id {
            self.category_id = c;
        }
        if let Some(i) = cmd.image {
            self.image = i;
        }
        if let Some(t) = cmd.image_thumb {
            self.image_thumb = t;
        }
        if let Some(b) = cmd.news_body {
            self.news_body = b;
        }
        if let Some(p) = cmd.publish_date {
            self.publish_date = p;
        }
        if let Some(g) = cmd.is_global {
            self.is_global = g;
        }
        if let Some(a) = cmd.auto_approve {
            self.auto_approve = a;
        }
        if let Some(c) = cmd.is_comment {
            self.is_comment = c;
        }
        if let Some(o) = cmd.order {
            self.order = o;
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Marks the news as published.
    pub fn publish(&mut self, _actor: UserId, at: Timestamp, event_id: EventId) {
        self.updated_at = at;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the news as unpublished.
    pub fn unpublish(&mut self, _actor: UserId, at: Timestamp, event_id: EventId) {
        self.updated_at = at;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the news.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict("news is already disabled".to_owned()));
        }
        self.active_status = NewsStatus::Disabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Increments the view count. Per spec invariant 8, the
    /// view count is non-decreasing.
    pub fn increment_view(&mut self) {
        self.view_count = self.view_count.saturating_add(1);
    }

    /// Returns `true` if the news is visible on the public
    /// site (`active_status = Active` and `publish_date <= today`).
    #[must_use]
    pub fn is_visible(&self, today: NaiveDate) -> bool {
        self.active_status.is_active() && self.publish_date.as_naive_date() <= today
    }
}

/// Aggregate-local input for [`NewsCategory::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewNewsCategory {
    /// The typed id.
    pub id: NewsCategoryId,
    /// The category name.
    pub category_name: CategoryName,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A news category taxonomy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCategory {
    /// The typed id.
    pub id: NewsCategoryId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The category name.
    pub category_name: CategoryName,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl NewsCategory {
    /// Constructs a new `NewsCategory`.
    pub fn new(cmd: NewNewsCategory) -> Result<Self, CmsError> {
        if cmd.category_name.as_str().is_empty() {
            return Err(CmsError::Validation(
                "news category name must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            category_name: cmd.category_name,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the category name.
    pub fn rename(
        &mut self,
        new_name: CategoryName,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.category_name = new_name;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the category.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "news category is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`NewsComment::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewNewsComment {
    /// The typed id.
    pub id: NewsCommentId,
    /// The news id.
    pub news_id: NewsId,
    /// The user id.
    pub user_id: UserId,
    /// The optional parent comment id.
    pub parent_id: Option<NewsCommentId>,
    /// The comment message.
    pub message: CommentMessage,
    /// The initial status (Pending or Approved depending on
    /// the news' `auto_approve` flag).
    pub status: NewsCommentStatus,
    /// The creation timestamp.
    pub created_at: Timestamp,
}

/// A per-user comment on a [`News`] entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsComment {
    /// The typed id.
    pub id: NewsCommentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The news id.
    pub news_id: NewsId,
    /// The user id.
    pub user_id: UserId,
    /// The optional parent comment id.
    pub parent_id: Option<NewsCommentId>,
    /// The comment message.
    pub message: CommentMessage,
    /// The moderation status.
    pub status: NewsCommentStatus,
    /// The creation timestamp.
    pub created_at: Timestamp,
}

impl NewsComment {
    /// Constructs a new `NewsComment`.
    pub fn new(cmd: NewNewsComment) -> Result<Self, CmsError> {
        if cmd.message.as_str().is_empty() {
            return Err(CmsError::CommentMessageEmpty);
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            news_id: cmd.news_id,
            user_id: cmd.user_id,
            parent_id: cmd.parent_id,
            message: cmd.message,
            status: cmd.status,
            created_at: cmd.created_at,
        })
    }

    /// Approves the comment.
    pub fn approve(&mut self) {
        self.status = NewsCommentStatus::Approved;
    }

    /// Hides the comment.
    pub fn hide(&mut self) {
        self.status = NewsCommentStatus::Hidden;
    }

    /// Returns `true` if the comment is approved.
    #[must_use]
    pub fn is_approved(&self) -> bool {
        self.status.is_approved()
    }
}

/// Aggregate-local input for [`NewsPage::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewNewsPage {
    /// The typed id.
    pub id: NewsPageId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The main title.
    pub main_title: Option<PageTitle>,
    /// The main description.
    pub main_description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// The public news landing-page configuration. Per the spec,
/// at most one per school may be active.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsPage {
    /// The typed id.
    pub id: NewsPageId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The main title.
    pub main_title: Option<PageTitle>,
    /// The main description.
    pub main_description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl NewsPage {
    /// Constructs a new `NewsPage`.
    pub fn new(cmd: NewNewsPage) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            description: cmd.description,
            main_title: cmd.main_title,
            main_description: cmd.main_description,
            image: cmd.image,
            main_image: cmd.main_image,
            button_text: cmd.button_text,
            button_url: cmd.button_url,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the news page.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "news page is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Section: NoticeBoard + Testimonial + HomeSlider + SpeechSlider
// =============================================================================

/// Aggregate-local input for [`NoticeBoard::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewNoticeBoard {
    /// The typed id.
    pub id: NoticeBoardId,
    /// The notice title.
    pub notice_title: NoticeTitle,
    /// The notice message body.
    pub notice_message: NoticeMessage,
    /// The notice date.
    pub notice_date: NoticeDate,
    /// The optional publish date.
    pub publish_on: Option<PublishDate>,
    /// The audience descriptor.
    pub inform_to: AudienceDescriptor,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// The public-site notice board (school-side). Distinct from
/// the communication domain's `Notice` aggregate which targets
/// staff and guardians.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoard {
    /// The typed id.
    pub id: NoticeBoardId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The notice title.
    pub notice_title: NoticeTitle,
    /// The notice message body.
    pub notice_message: NoticeMessage,
    /// The notice date.
    pub notice_date: NoticeDate,
    /// The optional publish date.
    pub publish_on: Option<PublishDate>,
    /// The audience descriptor.
    pub inform_to: AudienceDescriptor,
    /// The published flag.
    pub is_published: IsPublished,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl NoticeBoard {
    /// Constructs a new `NoticeBoard` (initially hidden).
    pub fn new(cmd: NewNoticeBoard) -> Result<Self, CmsError> {
        if cmd.notice_title.as_str().is_empty() {
            return Err(CmsError::Validation(
                "notice board title must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            notice_title: cmd.notice_title,
            notice_message: cmd.notice_message,
            notice_date: cmd.notice_date,
            publish_on: cmd.publish_on,
            inform_to: cmd.inform_to,
            is_published: IsPublished::new(false),
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Publishes the notice board.
    pub fn publish(&mut self, _actor: UserId, at: Timestamp, event_id: EventId) {
        self.is_published = IsPublished::new(true);
        self.updated_at = at;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Unpublishes the notice board.
    pub fn unpublish(&mut self, _actor: UserId, at: Timestamp, event_id: EventId) {
        self.is_published = IsPublished::new(false);
        self.updated_at = at;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the notice board.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "notice board is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`Testimonial::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewTestimonial {
    /// The typed id.
    pub id: TestimonialId,
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
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A testimonial surfaced on the public site.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Testimonial {
    /// The typed id.
    pub id: TestimonialId,
    /// The owning school.
    pub school_id: SchoolId,
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
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl Testimonial {
    /// Constructs a new `Testimonial`.
    pub fn new(cmd: NewTestimonial) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            name: cmd.name,
            designation: cmd.designation,
            institution_name: cmd.institution_name,
            image: cmd.image,
            description: cmd.description,
            star_rating: cmd.star_rating,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the rating.
    pub fn update_rating(
        &mut self,
        rating: StarRating,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.star_rating = rating;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the testimonial.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "testimonial is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`HomeSlider::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewHomeSlider {
    /// The typed id.
    pub id: HomeSliderId,
    /// The image file.
    pub image: FileReference,
    /// The optional link URL.
    pub link: Option<Url>,
    /// The optional link label.
    pub link_label: Option<HomeSliderLinkLabel>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A home-page slider entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeSlider {
    /// The typed id.
    pub id: HomeSliderId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The image file.
    pub image: FileReference,
    /// The optional link URL.
    pub link: Option<Url>,
    /// The optional link label.
    pub link_label: Option<HomeSliderLinkLabel>,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl HomeSlider {
    /// Constructs a new `HomeSlider`.
    pub fn new(cmd: NewHomeSlider) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            image: cmd.image,
            link: cmd.link,
            link_label: cmd.link_label,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the slider.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "home slider is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`SpeechSlider::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewSpeechSlider {
    /// The typed id.
    pub id: SpeechSliderId,
    /// The person's name.
    pub name: PersonName,
    /// The designation.
    pub designation: Designation,
    /// The free-text speech body.
    pub speech: SpeechText,
    /// The image file.
    pub image: FileReference,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// CMS-side speech slider (the public-page rendering reference).
/// The communication domain has its own `SpeechSlider`; per the
/// spec, the CMS owns the public-page rendering reference.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSlider {
    /// The typed id.
    pub id: SpeechSliderId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The person's name.
    pub name: PersonName,
    /// The designation.
    pub designation: Designation,
    /// The free-text speech body.
    pub speech: SpeechText,
    /// The image file.
    pub image: FileReference,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl SpeechSlider {
    /// Constructs a new `SpeechSlider`.
    pub fn new(cmd: NewSpeechSlider) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            name: cmd.name,
            designation: cmd.designation,
            speech: cmd.speech,
            image: cmd.image,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the speech slider.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "speech slider is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Section: Content family
// =============================================================================

/// Aggregate-local input for [`Content::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewContent {
    /// The typed id.
    pub id: ContentId,
    /// The file name.
    pub file_name: String,
    /// The file size in bytes.
    pub file_size: i64,
    /// The content type id.
    pub content_type_id: ContentTypeId,
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
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`Content::update`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateContent {
    /// The new file name, if changing.
    pub file_name: Option<String>,
    /// The new file size, if changing.
    pub file_size: Option<i64>,
    /// The new content type id, if changing.
    pub content_type_id: Option<ContentTypeId>,
    /// The new YouTube link, if changing or clearing.
    pub youtube_link: Option<Option<YoutubeLink>>,
    /// The new file reference, if changing or clearing.
    pub upload_file: Option<Option<FileReference>>,
    /// The new role scope, if changing or clearing.
    pub available_for_role: Option<Option<i32>>,
    /// The new class scope, if changing or clearing.
    pub available_for_class: Option<Option<i32>>,
    /// The new section scope, if changing or clearing.
    pub available_for_section: Option<Option<i32>>,
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// An uploaded content item. Anchored to a content type and an
/// academic year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    /// The typed id.
    pub id: ContentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The file name.
    pub file_name: String,
    /// The file size in bytes.
    pub file_size: i64,
    /// The content type id.
    pub content_type_id: ContentTypeId,
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
    /// The uploader (user id).
    pub uploaded_by: UserId,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl Content {
    /// Constructs a new `Content`.
    pub fn new(cmd: NewContent) -> Result<Self, CmsError> {
        if cmd.file_name.is_empty() {
            return Err(CmsError::Validation(
                "content file name must not be empty".to_owned(),
            ));
        }
        if cmd.file_size < 0 {
            return Err(CmsError::Validation(
                "content file size must be non-negative".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            file_name: cmd.file_name,
            file_size: cmd.file_size,
            content_type_id: cmd.content_type_id,
            youtube_link: cmd.youtube_link,
            upload_file: cmd.upload_file,
            available_for_role: cmd.available_for_role,
            available_for_class: cmd.available_for_class,
            available_for_section: cmd.available_for_section,
            academic_id: cmd.academic_id,
            uploaded_by: cmd.created_by,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the content.
    pub fn update(&mut self, cmd: UpdateContent) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "cannot update a deleted content".to_owned(),
            ));
        }
        if let Some(f) = cmd.file_name {
            self.file_name = f;
        }
        if let Some(s) = cmd.file_size {
            if s < 0 {
                return Err(CmsError::Validation(
                    "content file size must be non-negative".to_owned(),
                ));
            }
            self.file_size = s;
        }
        if let Some(c) = cmd.content_type_id {
            self.content_type_id = c;
        }
        if let Some(y) = cmd.youtube_link {
            self.youtube_link = y;
        }
        if let Some(f) = cmd.upload_file {
            self.upload_file = f;
        }
        if let Some(r) = cmd.available_for_role {
            self.available_for_role = r;
        }
        if let Some(c) = cmd.available_for_class {
            self.available_for_class = c;
        }
        if let Some(s) = cmd.available_for_section {
            self.available_for_section = s;
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Soft-deletes the content.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict("content is already deleted".to_owned()));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the content is available to the given
    /// role id (per spec invariant 13).
    #[must_use]
    pub fn available_to_role(&self, role_id: i32) -> bool {
        match self.available_for_role {
            Some(r) => r == role_id,
            None => true,
        }
    }

    /// Returns `true` if the content is available to the given
    /// class-section pair.
    #[must_use]
    pub fn available_to_class(&self, class: ClassId, section: Option<SectionId>) -> bool {
        match (self.available_for_class, self.available_for_section) {
            (None, None) => true,
            (Some(c), None) => {
                i64::from(c) == i64::try_from(class.as_uuid().as_u128() >> 64).unwrap_or(i64::MIN)
            }
            (None, Some(_)) => false,
            (Some(c), Some(s)) => {
                i64::from(c) == i64::try_from(class.as_uuid().as_u128() >> 64).unwrap_or(i64::MIN)
                    && section.is_some_and(|sec| {
                        i64::from(s)
                            == i64::try_from(sec.as_uuid().as_u128() >> 64).unwrap_or(i64::MIN)
                    })
            }
        }
    }
}

/// Aggregate-local input for [`ContentType::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewContentType {
    /// The typed id.
    pub id: ContentTypeId,
    /// The content type name.
    pub type_name: ContentTypeName,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A content type taxonomy entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentType {
    /// The typed id.
    pub id: ContentTypeId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The content type name.
    pub type_name: ContentTypeName,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl ContentType {
    /// Constructs a new `ContentType`.
    pub fn new(cmd: NewContentType) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            type_name: cmd.type_name,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Updates the content type name.
    pub fn rename(
        &mut self,
        new_name: ContentTypeName,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.type_name = new_name;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the content type.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "content type is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`ContentShareList::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewContentShareList {
    /// The typed id.
    pub id: ContentShareListId,
    /// The share list title.
    pub title: ContentShareListTitle,
    /// The share date.
    pub share_date: ShareDate,
    /// The valid-upto date.
    pub valid_upto: ValidUntil,
    /// The optional description.
    pub description: Option<String>,
    /// The send type (G/C/I/P).
    pub send_type: ContentShareType,
    /// The content ids being shared.
    pub content_ids: Vec<ContentId>,
    /// The optional role ids (for `G`).
    pub gr_role_ids: Option<Vec<Uuid>>,
    /// The optional user ids (for `I`).
    pub ind_user_ids: Option<Vec<Uuid>>,
    /// The optional class id (for `C`).
    pub class_id: Option<ClassId>,
    /// The optional section ids (for `C`).
    pub section_ids: Option<Vec<SectionId>>,
    /// The optional URL.
    pub url: Option<Url>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A bulk-share list of content items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareList {
    /// The typed id.
    pub id: ContentShareListId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The share list title.
    pub title: ContentShareListTitle,
    /// The share date.
    pub share_date: ShareDate,
    /// The valid-upto date.
    pub valid_upto: ValidUntil,
    /// The optional description.
    pub description: Option<String>,
    /// The send type.
    pub send_type: ContentShareType,
    /// The content ids being shared.
    pub content_ids: Vec<ContentId>,
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
    /// The status.
    pub status: ContentShareListStatus,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl ContentShareList {
    /// Constructs a new `ContentShareList` in `Draft` status.
    pub fn new(cmd: NewContentShareList) -> Result<Self, CmsError> {
        if cmd.valid_upto.as_naive_date() < cmd.share_date.as_naive_date() {
            return Err(CmsError::ContentShareListInvalidWindow);
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            share_date: cmd.share_date,
            valid_upto: cmd.valid_upto,
            description: cmd.description,
            send_type: cmd.send_type,
            content_ids: cmd.content_ids,
            gr_role_ids: cmd.gr_role_ids,
            ind_user_ids: cmd.ind_user_ids,
            class_id: cmd.class_id,
            section_ids: cmd.section_ids,
            url: cmd.url,
            academic_id: cmd.academic_id,
            status: ContentShareListStatus::Draft,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Dispatches the share list (freezes the audience).
    pub fn dispatch(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Result<(), CmsError> {
        if !matches!(self.status, ContentShareListStatus::Draft) {
            return Err(CmsError::ContentShareListNotDispatchable(self.id.as_uuid()));
        }
        self.status = ContentShareListStatus::Dispatched;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Cancels the share list (only valid in `Draft`).
    pub fn cancel(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Result<(), CmsError> {
        if !matches!(self.status, ContentShareListStatus::Draft) {
            return Err(CmsError::ContentShareListNotCancellable(self.id.as_uuid()));
        }
        self.status = ContentShareListStatus::Cancelled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        Ok(())
    }

    /// Soft-deletes the share list.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "content share list is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the share list is within the validity
    /// window (`share_date <= date <= valid_upto`).
    #[must_use]
    pub fn is_within_share_window(&self, date: NaiveDate) -> bool {
        let share = self.share_date.as_naive_date();
        let valid = self.valid_upto.as_naive_date();
        date >= share && date <= valid
    }
}

/// Aggregate-local input for [`TeacherUploadContent::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewTeacherUploadContent {
    /// The typed id.
    pub id: TeacherUploadContentId,
    /// The content title.
    pub content_title: ContentTitle,
    /// The content type.
    pub content_type: TeacherContentType,
    /// Whether the content is available to admins.
    pub available_for_admin: AvailableForAdmin,
    /// Whether the content is available to all classes.
    pub available_for_all_classes: AvailableForAllClasses,
    /// The upload date.
    pub upload_date: UploadDate,
    /// The optional description.
    pub description: Option<ContentDescription>,
    /// The optional source URL.
    pub source_url: Option<SourceUrl>,
    /// The optional file reference.
    pub upload_file: Option<FileReference>,
    /// The optional course id.
    pub course_id: Option<i32>,
    /// The optional parent course id.
    pub parent_course_id: Option<i32>,
    /// The class id.
    pub class_id: ClassId,
    /// The section id.
    pub section_id: SectionId,
    /// The optional chapter id.
    pub chapter_id: Option<i64>,
    /// The optional lesson id.
    pub lesson_id: Option<i64>,
    /// The optional parent content id.
    pub parent_id: Option<i32>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
    /// The creating user (the teacher).
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A teacher-uploaded content item (per class-section).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherUploadContent {
    /// The typed id.
    pub id: TeacherUploadContentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The content title.
    pub content_title: ContentTitle,
    /// The content type.
    pub content_type: TeacherContentType,
    /// Whether the content is available to admins.
    pub available_for_admin: AvailableForAdmin,
    /// Whether the content is available to all classes.
    pub available_for_all_classes: AvailableForAllClasses,
    /// The upload date.
    pub upload_date: UploadDate,
    /// The optional description.
    pub description: Option<ContentDescription>,
    /// The optional source URL.
    pub source_url: Option<SourceUrl>,
    /// The optional file reference.
    pub upload_file: Option<FileReference>,
    /// The optional course id.
    pub course_id: Option<i32>,
    /// The optional parent course id.
    pub parent_course_id: Option<i32>,
    /// The class id.
    pub class_id: ClassId,
    /// The section id.
    pub section_id: SectionId,
    /// The optional chapter id.
    pub chapter_id: Option<i64>,
    /// The optional lesson id.
    pub lesson_id: Option<i64>,
    /// The optional parent content id.
    pub parent_id: Option<i32>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl TeacherUploadContent {
    /// Constructs a new `TeacherUploadContent`.
    pub fn new(cmd: NewTeacherUploadContent) -> Result<Self, CmsError> {
        if cmd.content_title.as_str().is_empty() {
            return Err(CmsError::Validation(
                "teacher upload content title must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            content_title: cmd.content_title,
            content_type: cmd.content_type,
            available_for_admin: cmd.available_for_admin,
            available_for_all_classes: cmd.available_for_all_classes,
            upload_date: cmd.upload_date,
            description: cmd.description,
            source_url: cmd.source_url,
            upload_file: cmd.upload_file,
            course_id: cmd.course_id,
            parent_course_id: cmd.parent_course_id,
            class_id: cmd.class_id,
            section_id: cmd.section_id,
            chapter_id: cmd.chapter_id,
            lesson_id: cmd.lesson_id,
            parent_id: cmd.parent_id,
            academic_id: cmd.academic_id,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the teacher upload content.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "teacher upload content is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`UploadContent::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewUploadContent {
    /// The typed id.
    pub id: UploadContentId,
    /// The content title.
    pub content_title: ContentTitle,
    /// The content type id (FK to a `ContentType` taxonomy entry).
    pub content_type: i32,
    /// The optional role scope id.
    pub available_for_role: Option<i32>,
    /// The optional class scope id.
    pub available_for_class: Option<i32>,
    /// The optional section scope id.
    pub available_for_section: Option<i32>,
    /// The upload date.
    pub upload_date: UploadDate,
    /// The optional description.
    pub description: Option<ContentDescription>,
    /// The optional file reference.
    pub upload_file: Option<FileReference>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// An admin-uploaded content item (per role/class/section).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadContent {
    /// The typed id.
    pub id: UploadContentId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The content title.
    pub content_title: ContentTitle,
    /// The content type id.
    pub content_type: i32,
    /// The optional role scope id.
    pub available_for_role: Option<i32>,
    /// The optional class scope id.
    pub available_for_class: Option<i32>,
    /// The optional section scope id.
    pub available_for_section: Option<i32>,
    /// The upload date.
    pub upload_date: UploadDate,
    /// The optional description.
    pub description: Option<ContentDescription>,
    /// The optional file reference.
    pub upload_file: Option<FileReference>,
    /// The academic year scope.
    pub academic_id: AcademicYearId,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl UploadContent {
    /// Constructs a new `UploadContent`.
    pub fn new(cmd: NewUploadContent) -> Result<Self, CmsError> {
        if cmd.content_title.as_str().is_empty() {
            return Err(CmsError::Validation(
                "upload content title must not be empty".to_owned(),
            ));
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            content_title: cmd.content_title,
            content_type: cmd.content_type,
            available_for_role: cmd.available_for_role,
            available_for_class: cmd.available_for_class,
            available_for_section: cmd.available_for_section,
            upload_date: cmd.upload_date,
            description: cmd.description,
            upload_file: cmd.upload_file,
            academic_id: cmd.academic_id,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the upload content.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "upload content is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

// =============================================================================
// Section: per-page templates (AboutPage, ContactPage, CoursePage,
// HomePageSetting, FrontendPage)
// =============================================================================

/// Aggregate-local input for [`AboutPage::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewAboutPage {
    /// The typed id.
    pub id: AboutPageId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The main title.
    pub main_title: Option<PageTitle>,
    /// The main description.
    pub main_description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// The about-page configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AboutPage {
    /// The typed id.
    pub id: AboutPageId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The main title.
    pub main_title: Option<PageTitle>,
    /// The main description.
    pub main_description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl AboutPage {
    /// Constructs a new `AboutPage`.
    pub fn new(cmd: NewAboutPage) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            description: cmd.description,
            main_title: cmd.main_title,
            main_description: cmd.main_description,
            image: cmd.image,
            main_image: cmd.main_image,
            button_text: cmd.button_text,
            button_url: cmd.button_url,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the about page.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "about page is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`ContactPage::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewContactPage {
    /// The typed id.
    pub id: ContactPageId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The postal address.
    pub address: Option<PostalAddress>,
    /// The phone number.
    pub phone: Option<PhoneNumber>,
    /// The email address.
    pub email: Option<EmailAddress>,
    /// The latitude string.
    pub latitude: Option<Latitude>,
    /// The longitude string.
    pub longitude: Option<Longitude>,
    /// The zoom level.
    pub zoom_level: Option<ZoomLevel>,
    /// The Google Maps address.
    pub google_map_address: Option<GoogleMapAddress>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// The contact-page configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPage {
    /// The typed id.
    pub id: ContactPageId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The page title.
    pub title: PageTitle,
    /// The description.
    pub description: Option<PageDescription>,
    /// The image.
    pub image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// The postal address.
    pub address: Option<PostalAddress>,
    /// The phone number.
    pub phone: Option<PhoneNumber>,
    /// The email address.
    pub email: Option<EmailAddress>,
    /// The latitude string.
    pub latitude: Option<Latitude>,
    /// The longitude string.
    pub longitude: Option<Longitude>,
    /// The zoom level.
    pub zoom_level: Option<ZoomLevel>,
    /// The Google Maps address.
    pub google_map_address: Option<GoogleMapAddress>,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl ContactPage {
    /// Constructs a new `ContactPage`.
    pub fn new(cmd: NewContactPage) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            description: cmd.description,
            image: cmd.image,
            button_text: cmd.button_text,
            button_url: cmd.button_url,
            address: cmd.address,
            phone: cmd.phone,
            email: cmd.email,
            latitude: cmd.latitude,
            longitude: cmd.longitude,
            zoom_level: cmd.zoom_level,
            google_map_address: cmd.google_map_address,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the contact page.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "contact page is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`CoursePage::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewCoursePage {
    /// The typed id.
    pub id: CoursePageId,
    /// The course page title.
    pub title: CoursePageTitle,
    /// The course page description.
    pub description: Option<CoursePageDescription>,
    /// The main title.
    pub main_title: Option<String>,
    /// The main description.
    pub main_description: Option<String>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// Whether this is a top-level parent course.
    pub is_parent: IsParent,
    /// The optional parent course id.
    pub parent_id: Option<CoursePageId>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A course landing page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoursePage {
    /// The typed id.
    pub id: CoursePageId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The course page title.
    pub title: CoursePageTitle,
    /// The course page description.
    pub description: Option<CoursePageDescription>,
    /// The main title.
    pub main_title: Option<String>,
    /// The main description.
    pub main_description: Option<String>,
    /// The image.
    pub image: Option<FileReference>,
    /// The main image.
    pub main_image: Option<FileReference>,
    /// The button text.
    pub button_text: Option<ButtonText>,
    /// The button URL.
    pub button_url: Option<ButtonUrl>,
    /// Whether this is a top-level parent course.
    pub is_parent: IsParent,
    /// The optional parent course id.
    pub parent_id: Option<CoursePageId>,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl CoursePage {
    /// Constructs a new `CoursePage`.
    pub fn new(cmd: NewCoursePage) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            description: cmd.description,
            main_title: cmd.main_title,
            main_description: cmd.main_description,
            image: cmd.image,
            main_image: cmd.main_image,
            button_text: cmd.button_text,
            button_url: cmd.button_url,
            is_parent: cmd.is_parent,
            parent_id: cmd.parent_id,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the course page.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "course page is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`HomePageSetting::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewHomePageSetting {
    /// The typed id.
    pub id: HomePageSettingId,
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
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// The home-page setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageSetting {
    /// The typed id.
    pub id: HomePageSettingId,
    /// The owning school.
    pub school_id: SchoolId,
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
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl HomePageSetting {
    /// Constructs a new `HomePageSetting`.
    pub fn new(cmd: NewHomePageSetting) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            long_title: cmd.long_title,
            short_description: cmd.short_description,
            link_label: cmd.link_label,
            link_url: cmd.link_url,
            image: cmd.image,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the home-page setting.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "home page setting is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }
}

/// Aggregate-local input for [`FrontendPage::new`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFrontendPage {
    /// The typed id.
    pub id: FrontendPageId,
    /// The page title.
    pub title: PageTitle,
    /// The page sub-title (unique within school).
    pub sub_title: PageSubTitle,
    /// The optional slug (unique within school when set).
    pub slug: Option<Slug>,
    /// The optional header image.
    pub header_image: Option<FileReference>,
    /// The optional details (body).
    pub details: Option<PageDescription>,
    /// Whether the page is rendered dynamically.
    pub is_dynamic: IsDynamic,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// A generic front-end page record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendPage {
    /// The typed id.
    pub id: FrontendPageId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The page title.
    pub title: PageTitle,
    /// The page sub-title (unique within school).
    pub sub_title: PageSubTitle,
    /// The optional slug.
    pub slug: Option<Slug>,
    /// The optional header image.
    pub header_image: Option<FileReference>,
    /// The optional details (body).
    pub details: Option<PageDescription>,
    /// Whether the page is rendered dynamically.
    pub is_dynamic: IsDynamic,
    /// The active_status flag.
    pub active_status: ActiveStatus,
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

impl FrontendPage {
    /// Constructs a new `FrontendPage`.
    pub fn new(cmd: NewFrontendPage) -> Result<Self, CmsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            sub_title: cmd.sub_title,
            slug: cmd.slug,
            header_image: cmd.header_image,
            details: cmd.details,
            is_dynamic: cmd.is_dynamic,
            active_status: ActiveStatus::active(),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Soft-deletes the front-end page.
    pub fn soft_delete(&mut self, actor: UserId, at: Timestamp) -> Result<(), CmsError> {
        if !self.active_status.is_active() {
            return Err(CmsError::Conflict(
                "frontend page is already deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::inactive();
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
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
    clippy::dbg_macro,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value
)]
mod tests {
    use super::*;
    use educore_academic::prelude::Identifier as _;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::Identifier as _;

    fn ids() -> (SchoolId, UserId, EventId, CorrelationId, Timestamp) {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let u = g.next_user_id();
        let e = g.next_event_id();
        let c = g.next_correlation_id();
        let t = Timestamp::now();
        (s, u, e, c, t)
    }

    fn new_page() -> Page {
        let (s, u, _e, c, t) = ids();
        let id = PageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewPage {
            id,
            title: PageTitle::new("My Page").unwrap(),
            description: None,
            slug: Some(Slug::new("my-page").unwrap()),
            settings: None,
            home_page: HomePage::new(false),
            is_default: IsDefault::new(false),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        Page::new(cmd).expect("ok")
    }

    #[test]
    fn page_new_succeeds_and_default_status_is_draft() {
        let p = new_page();
        assert_eq!(p.status, PageStatus::Draft);
        assert!(p.is_active());
        assert!(!p.is_published());
        assert!(!p.is_home_page());
        assert!(!p.is_default.is_true());
        assert_eq!(p.school_id, p.id.school_id());
    }

    #[test]
    fn page_publish_transitions_status() {
        let (s, u, e, _c, t) = ids();
        let mut p = new_page();
        p.publish(u, t, e).expect("publish ok");
        assert!(p.is_published());
        assert_eq!(p.status, PageStatus::Published);
    }

    #[test]
    fn page_archive_transitions_back_to_draft() {
        let (s, u, e, _c, t) = ids();
        let mut p = new_page();
        p.publish(u, t, e).expect("publish ok");
        p.archive(u, t, e).expect("archive ok");
        assert_eq!(p.status, PageStatus::Draft);
    }

    #[test]
    fn page_soft_delete_succeeds() {
        let (_s, u, _e, _c, t) = ids();
        let mut p = new_page();
        p.soft_delete(u, t).expect("delete ok");
        assert!(!p.is_active());
    }

    #[test]
    fn page_double_soft_delete_returns_conflict() {
        let (_s, u, _e, _c, t) = ids();
        let mut p = new_page();
        p.soft_delete(u, t).expect("first delete ok");
        let err = p.soft_delete(u, t).unwrap_err();
        assert!(matches!(err, CmsError::Conflict(_)));
    }

    #[test]
    fn page_soft_delete_default_page_returns_default_not_deletable() {
        let (s, u, _e, c, t) = ids();
        let id = PageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewPage {
            id,
            title: PageTitle::new("Default").unwrap(),
            description: None,
            slug: None,
            settings: None,
            home_page: HomePage::new(false),
            is_default: IsDefault::new(true),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut p = Page::new(cmd).expect("ok");
        let err = p.soft_delete(u, t).unwrap_err();
        assert!(matches!(err, CmsError::DefaultPageNotDeletable(_)));
    }

    #[test]
    fn page_publish_on_soft_deleted_returns_conflict() {
        let (_s, u, _e, _c, t) = ids();
        let mut p = new_page();
        p.soft_delete(u, t).expect("delete ok");
        let err = p.publish(u, t, uuid::Uuid::now_v7().into()).unwrap_err();
        assert!(matches!(err, CmsError::Conflict(_)));
    }

    #[test]
    fn page_next_status_returns_published_for_publish_action() {
        let p = new_page();
        assert_eq!(
            p.next_status(PageStatusAction::Publish),
            PageStatus::Published
        );
    }

    #[test]
    fn page_next_status_returns_draft_for_archive_action() {
        let p = new_page();
        assert_eq!(p.next_status(PageStatusAction::Archive), PageStatus::Draft);
    }

    #[test]
    fn new_news_increments_view_count() {
        let (s, u, _e, c, t) = ids();
        let id = NewsId::new(s, uuid::Uuid::now_v7());
        let category_id = NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNews {
            id,
            news_title: NewsTitle::new("Title").unwrap(),
            category_id,
            image: None,
            image_thumb: None,
            news_body: NewsBody::new("body").unwrap(),
            publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            is_global: IsGlobal::new(false),
            auto_approve: AutoApprove::new(false),
            is_comment: IsComment::new(true),
            order: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut n = News::new(cmd).expect("ok");
        assert_eq!(n.view_count, 0);
        n.increment_view();
        n.increment_view();
        n.increment_view();
        assert_eq!(n.view_count, 3);
    }

    #[test]
    fn news_is_visible_iff_active_and_publish_date_in_past() {
        let (s, u, _e, c, t) = ids();
        let id = NewsId::new(s, uuid::Uuid::now_v7());
        let category_id = NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNews {
            id,
            news_title: NewsTitle::new("T").unwrap(),
            category_id,
            image: None,
            image_thumb: None,
            news_body: NewsBody::new("b").unwrap(),
            publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            is_global: IsGlobal::new(false),
            auto_approve: AutoApprove::new(false),
            is_comment: IsComment::new(true),
            order: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let n = News::new(cmd).expect("ok");
        assert!(n.is_visible(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
        assert!(n.is_visible(chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()));
        assert!(!n.is_visible(chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()));
    }

    #[test]
    fn news_soft_delete_sets_inactive_status() {
        let (s, u, _e, c, t) = ids();
        let id = NewsId::new(s, uuid::Uuid::now_v7());
        let category_id = NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNews {
            id,
            news_title: NewsTitle::new("T").unwrap(),
            category_id,
            image: None,
            image_thumb: None,
            news_body: NewsBody::new("b").unwrap(),
            publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            is_global: IsGlobal::new(false),
            auto_approve: AutoApprove::new(false),
            is_comment: IsComment::new(true),
            order: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut n = News::new(cmd).expect("ok");
        n.soft_delete(u, t).expect("ok");
        assert!(!n.active_status.is_active());
    }

    #[test]
    fn news_category_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNewsCategory {
            id,
            category_name: CategoryName::new("Sports").unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let cat = NewsCategory::new(cmd).expect("ok");
        assert_eq!(cat.category_name.as_str(), "Sports");
    }

    #[test]
    fn news_comment_new_with_empty_message_returns_error() {
        let (s, _u, _e, _c, _t) = ids();
        let id = NewsCommentId::new(s, uuid::Uuid::now_v7());
        let news_id = NewsId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNewsComment {
            id,
            news_id,
            user_id: UserId::from_uuid(uuid::Uuid::now_v7()),
            parent_id: None,
            message: CommentMessage::_new_unchecked_for_test(String::new()),
            status: NewsCommentStatus::Pending,
            created_at: Timestamp::now(),
        };
        let err = NewsComment::new(cmd).unwrap_err();
        assert!(matches!(err, CmsError::CommentMessageEmpty));
    }

    #[test]
    fn news_comment_approve_transitions_status() {
        let (s, _u, _e, _c, _t) = ids();
        let id = NewsCommentId::new(s, uuid::Uuid::now_v7());
        let news_id = NewsId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNewsComment {
            id,
            news_id,
            user_id: UserId::from_uuid(uuid::Uuid::now_v7()),
            parent_id: None,
            message: CommentMessage::new("ok").unwrap(),
            status: NewsCommentStatus::Pending,
            created_at: Timestamp::now(),
        };
        let mut c = NewsComment::new(cmd).expect("ok");
        c.approve();
        assert!(c.is_approved());
    }

    #[test]
    fn news_comment_hide_transitions_status() {
        let (s, _u, _e, _c, _t) = ids();
        let id = NewsCommentId::new(s, uuid::Uuid::now_v7());
        let news_id = NewsId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNewsComment {
            id,
            news_id,
            user_id: UserId::from_uuid(uuid::Uuid::now_v7()),
            parent_id: None,
            message: CommentMessage::new("ok").unwrap(),
            status: NewsCommentStatus::Approved,
            created_at: Timestamp::now(),
        };
        let mut c = NewsComment::new(cmd).expect("ok");
        c.hide();
        assert!(!c.is_approved());
    }

    #[test]
    fn news_page_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = NewsPageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNewsPage {
            id,
            title: PageTitle::new("News").unwrap(),
            description: None,
            main_title: None,
            main_description: None,
            image: None,
            main_image: None,
            button_text: None,
            button_url: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let np = NewsPage::new(cmd).expect("ok");
        assert!(np.active_status.is_active());
    }

    #[test]
    fn notice_board_publish_flips_flag() {
        let (s, u, _e, c, t) = ids();
        let id = NoticeBoardId::new(s, uuid::Uuid::now_v7());
        let cmd = NewNoticeBoard {
            id,
            notice_title: NoticeTitle::new("N").unwrap(),
            notice_message: NoticeMessage::new("M").unwrap(),
            notice_date: NoticeDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            publish_on: None,
            inform_to: AudienceDescriptor::new("admin,teacher").unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut nb = NoticeBoard::new(cmd).expect("ok");
        assert!(!nb.is_published.is_true());
        nb.publish(u, t, uuid::Uuid::now_v7().into());
        assert!(nb.is_published.is_true());
        nb.unpublish(u, t, uuid::Uuid::now_v7().into());
        assert!(!nb.is_published.is_true());
    }

    #[test]
    fn testimonial_update_rating_persists() {
        let (s, u, _e, c, t) = ids();
        let id = TestimonialId::new(s, uuid::Uuid::now_v7());
        let cmd = NewTestimonial {
            id,
            name: PersonName::new("Alice").unwrap(),
            designation: Designation::new("Principal").unwrap(),
            institution_name: InstitutionName::new("Acme").unwrap(),
            image: FileReference::new("img").unwrap(),
            description: TestimonialDescription::new("Great").unwrap(),
            star_rating: StarRating::new(3).unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut tm = Testimonial::new(cmd).expect("ok");
        tm.update_rating(
            StarRating::new(5).unwrap(),
            u,
            t,
            uuid::Uuid::now_v7().into(),
        );
        assert_eq!(tm.star_rating.value(), 5);
    }

    #[test]
    fn testimonial_soft_delete_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = TestimonialId::new(s, uuid::Uuid::now_v7());
        let cmd = NewTestimonial {
            id,
            name: PersonName::new("Alice").unwrap(),
            designation: Designation::new("Principal").unwrap(),
            institution_name: InstitutionName::new("Acme").unwrap(),
            image: FileReference::new("img").unwrap(),
            description: TestimonialDescription::new("Great").unwrap(),
            star_rating: StarRating::new(5).unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut tm = Testimonial::new(cmd).expect("ok");
        tm.soft_delete(u, t).expect("ok");
        assert!(!tm.active_status.is_active());
    }

    #[test]
    fn home_slider_new_succeeds_with_optional_link() {
        let (s, u, _e, c, t) = ids();
        let id = HomeSliderId::new(s, uuid::Uuid::now_v7());
        let cmd = NewHomeSlider {
            id,
            image: FileReference::new("img").unwrap(),
            link: Some(Url::new("https://example.com").unwrap()),
            link_label: Some(HomeSliderLinkLabel::new("Visit").unwrap()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let hs = HomeSlider::new(cmd).expect("ok");
        assert!(hs.link.is_some());
    }

    #[test]
    fn speech_slider_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = SpeechSliderId::new(s, uuid::Uuid::now_v7());
        let cmd = NewSpeechSlider {
            id,
            name: PersonName::new("Director").unwrap(),
            designation: Designation::new("Director").unwrap(),
            speech: SpeechText::new("Hello").unwrap(),
            image: FileReference::new("img").unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let ss = SpeechSlider::new(cmd).expect("ok");
        assert!(ss.active_status.is_active());
    }

    #[test]
    fn content_share_list_invalid_window_returns_error() {
        let (s, u, _e, c, _t) = ids();
        let id = ContentShareListId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContentShareList {
            id,
            title: ContentShareListTitle::new("Share").unwrap(),
            share_date: ShareDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap()),
            valid_upto: ValidUntil::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            description: None,
            send_type: ContentShareType::Public,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: Timestamp::now(),
            correlation_id: c,
        };
        let err = ContentShareList::new(cmd).unwrap_err();
        assert!(matches!(err, CmsError::ContentShareListInvalidWindow));
    }

    #[test]
    fn content_share_list_dispatch_transitions_status() {
        let (s, u, _e, c, t) = ids();
        let id = ContentShareListId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContentShareList {
            id,
            title: ContentShareListTitle::new("Share").unwrap(),
            share_date: ShareDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            valid_upto: ValidUntil::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            description: None,
            send_type: ContentShareType::Public,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut list = ContentShareList::new(cmd).expect("ok");
        list.dispatch(u, t, uuid::Uuid::now_v7().into())
            .expect("dispatch ok");
        assert_eq!(list.status, ContentShareListStatus::Dispatched);
        // Dispatch again should fail.
        let err = list
            .dispatch(u, t, uuid::Uuid::now_v7().into())
            .unwrap_err();
        assert!(matches!(err, CmsError::ContentShareListNotDispatchable(_)));
    }

    #[test]
    fn content_share_list_cancel_only_in_draft() {
        let (s, u, _e, c, t) = ids();
        let id = ContentShareListId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContentShareList {
            id,
            title: ContentShareListTitle::new("Share").unwrap(),
            share_date: ShareDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            valid_upto: ValidUntil::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            description: None,
            send_type: ContentShareType::Public,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let mut list = ContentShareList::new(cmd).expect("ok");
        list.cancel(u, t, uuid::Uuid::now_v7().into())
            .expect("cancel ok");
        assert_eq!(list.status, ContentShareListStatus::Cancelled);
        // Cancel again should fail (no longer Draft).
        let err = list.cancel(u, t, uuid::Uuid::now_v7().into()).unwrap_err();
        assert!(matches!(err, CmsError::ContentShareListNotCancellable(_)));
    }

    #[test]
    fn content_share_list_is_within_share_window_strict() {
        let (s, u, _e, c, t) = ids();
        let id = ContentShareListId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContentShareList {
            id,
            title: ContentShareListTitle::new("Share").unwrap(),
            share_date: ShareDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            valid_upto: ValidUntil::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            description: None,
            send_type: ContentShareType::Public,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let list = ContentShareList::new(cmd).expect("ok");
        let within = chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let before = chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let after = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        assert!(list.is_within_share_window(within));
        assert!(!list.is_within_share_window(before));
        assert!(!list.is_within_share_window(after));
        // Boundary dates are inclusive.
        assert!(list.is_within_share_window(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
        assert!(list.is_within_share_window(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()));
    }

    #[test]
    fn content_new_with_negative_size_returns_error() {
        let (s, u, _e, c, _t) = ids();
        let id = ContentId::new(s, uuid::Uuid::now_v7());
        let ct = ContentTypeId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContent {
            id,
            file_name: "file.pdf".to_owned(),
            file_size: -1,
            content_type_id: ct,
            youtube_link: None,
            upload_file: None,
            available_for_role: None,
            available_for_class: None,
            available_for_section: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: Timestamp::now(),
            correlation_id: c,
        };
        let err = Content::new(cmd).unwrap_err();
        assert!(matches!(err, CmsError::Validation(_)));
    }

    #[test]
    fn content_new_with_empty_file_name_returns_error() {
        let (s, u, _e, c, _t) = ids();
        let id = ContentId::new(s, uuid::Uuid::now_v7());
        let ct = ContentTypeId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContent {
            id,
            file_name: String::new(),
            file_size: 1024,
            content_type_id: ct,
            youtube_link: None,
            upload_file: None,
            available_for_role: None,
            available_for_class: None,
            available_for_section: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: Timestamp::now(),
            correlation_id: c,
        };
        let err = Content::new(cmd).unwrap_err();
        assert!(matches!(err, CmsError::Validation(_)));
    }

    #[test]
    fn content_available_to_role_with_no_filter_returns_true() {
        let (s, u, _e, c, _t) = ids();
        let id = ContentId::new(s, uuid::Uuid::now_v7());
        let ct = ContentTypeId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContent {
            id,
            file_name: "file.pdf".to_owned(),
            file_size: 1024,
            content_type_id: ct,
            youtube_link: None,
            upload_file: None,
            available_for_role: None,
            available_for_class: None,
            available_for_section: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: Timestamp::now(),
            correlation_id: c,
        };
        let c = Content::new(cmd).expect("ok");
        assert!(c.available_to_role(0));
    }

    #[test]
    fn content_type_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = ContentTypeId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContentType {
            id,
            type_name: ContentTypeName::new("Assignment").unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let ct = ContentType::new(cmd).expect("ok");
        assert_eq!(ct.type_name.as_str(), "Assignment");
    }

    #[test]
    fn teacher_upload_content_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = TeacherUploadContentId::new(s, uuid::Uuid::now_v7());
        let class = ClassId::new(s, uuid::Uuid::now_v7());
        let section = SectionId::new(s, uuid::Uuid::now_v7());
        let cmd = NewTeacherUploadContent {
            id,
            content_title: ContentTitle::new("Math Notes").unwrap(),
            content_type: TeacherContentType::StudyMaterial,
            available_for_admin: AvailableForAdmin::new(false),
            available_for_all_classes: AvailableForAllClasses::new(false),
            upload_date: UploadDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            description: Some(ContentDescription::new("Notes").unwrap()),
            source_url: None,
            upload_file: Some(FileReference::new("file").unwrap()),
            course_id: None,
            parent_course_id: None,
            class_id: class,
            section_id: section,
            chapter_id: None,
            lesson_id: None,
            parent_id: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let tu = TeacherUploadContent::new(cmd).expect("ok");
        assert_eq!(tu.content_type, TeacherContentType::StudyMaterial);
    }

    #[test]
    fn upload_content_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = UploadContentId::new(s, uuid::Uuid::now_v7());
        let cmd = NewUploadContent {
            id,
            content_title: ContentTitle::new("Syllabus").unwrap(),
            content_type: 1,
            available_for_role: Some(2),
            available_for_class: Some(3),
            available_for_section: Some(4),
            upload_date: UploadDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            description: None,
            upload_file: None,
            academic_id: crate::value_objects::AcademicYearId::new(s, uuid::Uuid::now_v7()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let up = UploadContent::new(cmd).expect("ok");
        assert_eq!(up.content_type, 1);
    }

    #[test]
    fn about_page_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = AboutPageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewAboutPage {
            id,
            title: PageTitle::new("About").unwrap(),
            description: Some(PageDescription::new("About us").unwrap()),
            main_title: None,
            main_description: None,
            image: None,
            main_image: None,
            button_text: None,
            button_url: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let p = AboutPage::new(cmd).expect("ok");
        assert!(p.active_status.is_active());
    }

    #[test]
    fn contact_page_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = ContactPageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewContactPage {
            id,
            title: PageTitle::new("Contact").unwrap(),
            description: Some(PageDescription::new("Contact us").unwrap()),
            image: None,
            button_text: Some(ButtonText::new("Email").unwrap()),
            button_url: None,
            address: Some(PostalAddress::new("1 Main St").unwrap()),
            phone: Some(PhoneNumber::new("+15551234567").unwrap()),
            email: Some(EmailAddress::new("info@example.com").unwrap()),
            latitude: Some(Latitude::new("37.7749").unwrap()),
            longitude: Some(Longitude::new("-122.4194").unwrap()),
            zoom_level: Some(ZoomLevel::new(15).unwrap()),
            google_map_address: Some(GoogleMapAddress::new("1 Main St, City").unwrap()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let p = ContactPage::new(cmd).expect("ok");
        assert_eq!(p.zoom_level.unwrap().value(), 15);
    }

    #[test]
    fn contact_page_email_rejects_non_at_sign() {
        assert!(EmailAddress::new("not-an-email").is_err());
    }

    #[test]
    fn contact_page_zoom_level_rejects_out_of_range() {
        assert!(ZoomLevel::new(-1).is_err());
        assert!(ZoomLevel::new(22).is_err());
        assert!(ZoomLevel::new(0).is_ok());
        assert!(ZoomLevel::new(21).is_ok());
    }

    #[test]
    fn course_page_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = CoursePageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewCoursePage {
            id,
            title: CoursePageTitle::new("Math").unwrap(),
            description: None,
            main_title: None,
            main_description: None,
            image: None,
            main_image: None,
            button_text: None,
            button_url: None,
            is_parent: IsParent::new(true),
            parent_id: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let p = CoursePage::new(cmd).expect("ok");
        assert!(p.is_parent.is_true());
        assert!(p.parent_id.is_none());
    }

    #[test]
    fn home_page_setting_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = HomePageSettingId::new(s, uuid::Uuid::now_v7());
        let cmd = NewHomePageSetting {
            id,
            title: HomePageTitle::new("Welcome").unwrap(),
            long_title: Some(HomePageLongTitle::new("Welcome to Acme School").unwrap()),
            short_description: Some(HomePageShortDescription::new("We are Acme.").unwrap()),
            link_label: Some(HomeSliderLinkLabel::new("Learn more").unwrap()),
            link_url: Some(Url::new("https://example.com").unwrap()),
            image: Some(FileReference::new("hero").unwrap()),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let p = HomePageSetting::new(cmd).expect("ok");
        assert!(p.active_status.is_active());
    }

    #[test]
    fn frontend_page_new_succeeds() {
        let (s, u, _e, c, t) = ids();
        let id = FrontendPageId::new(s, uuid::Uuid::now_v7());
        let cmd = NewFrontendPage {
            id,
            title: PageTitle::new("Welcome").unwrap(),
            sub_title: PageSubTitle::new("Welcome to Acme").unwrap(),
            slug: Some(Slug::new("welcome").unwrap()),
            header_image: None,
            details: Some(PageDescription::new("body").unwrap()),
            is_dynamic: IsDynamic::new(true),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let p = FrontendPage::new(cmd).expect("ok");
        assert!(p.is_dynamic.is_true());
    }

    #[test]
    fn page_revision_build_carries_school_id() {
        let (s, _u, _e, _c, _t) = ids();
        let page_id = PageId::new(s, uuid::Uuid::now_v7());
        let rev_id = PageRevisionId::new(s, uuid::Uuid::now_v7());
        let cmd = NewPageRevision {
            id: rev_id,
            page_id,
            body: "body".to_owned(),
            revision_number: 1,
            created_at: Utc::now(),
            created_by: UserId::from_uuid(uuid::Uuid::now_v7()),
        };
        let rev = cmd.build();
        assert_eq!(rev.school_id, s);
        assert_eq!(rev.page_id, page_id);
    }
}
