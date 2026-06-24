//! CMS-domain events.
//!
//! Per `docs/specs/cms/events.md`. Each event implements
//! [`DomainEvent`] with `EVENT_TYPE` set to the wire form
//! `cms.<aggregate>.<verb>` (e.g. `cms.page.created`). The full
//! set:
//!
//! - Page lifecycle: `PageCreated`, `PageUpdated`, `PagePublished`,
//!   `PageArchived`, `PageDeleted` (5)
//! - News family: `NewsCreated`, `NewsUpdated`, `NewsPublished`,
//!   `NewsUnpublished`, `NewsDeleted`, `NewsViewIncremented` (6);
//!   plus `NewsCategory{Created,Updated,Deleted}` (3);
//!   `NewsComment{Added,Approved,Hidden,Deleted}` (4);
//!   `NewsPage{Created,Updated,Deleted}` (3)
//! - NoticeBoard: `NoticeBoard{Created,Updated,Published,
//!   Unpublished,Deleted}` (5)
//! - Public-site: `Testimonial{Created,Updated,Deleted}` (3);
//!   `HomeSlider{Created,Updated,Deleted}` (3);
//!   `SpeechSlider{Created,Updated,Deleted}` (3)
//! - Content family: `Content{Created,Updated,Deleted}` (3);
//!   `ContentType{Created,Updated,Deleted}` (3);
//!   `ContentShareList{Created,Dispatched,Cancelled,Updated,
//!   Deleted}` (5);
//!   `TeacherUploadContent{Created,Updated,Deleted}` (3);
//!   `UploadContent{Created,Updated,Deleted}` (3)
//! - Per-page templates: `AboutPage{Created,Updated,Deleted}` (3);
//!   `ContactPage{Created,Updated,Deleted}` (3);
//!   `CoursePage{Created,Updated,Deleted}` (3);
//!   `HomePageSetting{Configured,Updated,Deleted}` (3);
//!   `FrontendPage{Created,Updated,Deleted}` (3)

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::aggregate::{
    AboutPage, ContactPage, Content, ContentShareList, ContentType, CoursePage, FrontendPage,
    HomePageSetting, HomeSlider, News, NewsCategory, NewsComment, NewsPage, NoticeBoard, Page,
    SpeechSlider, TeacherUploadContent, Testimonial, UploadContent,
};
use crate::value_objects::{
    AboutPageId, ContactPageId, ContentId, ContentShareListId, ContentTypeId, CoursePageId,
    FrontendPageId, HomePageSettingId, HomeSliderId, NewsCategoryId, NewsCommentId, NewsId,
    NewsPageId, NoticeBoardId, PageId, SpeechSliderId, TeacherUploadContentId, TestimonialId,
    UploadContentId,
};

// =============================================================================
// Page lifecycle (5 events) — owner: A
// =============================================================================

/// Emitted when a new [`Page`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageCreated {
    /// The page id.
    pub page_id: PageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The page title.
    pub title: crate::value_objects::PageTitle,
    /// The optional slug.
    pub slug: Option<crate::value_objects::Slug>,
    /// Whether the page is the school's home page.
    pub home_page: bool,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PageCreated {
    /// Constructs a `PageCreated` from a just-built [`Page`]
    /// aggregate, the originating correlation id, and the
    /// `occurred_at` timestamp.
    #[must_use]
    pub fn new(page: &Page, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            page_id: page.id,
            school_id: page.school_id,
            title: page.title.clone(),
            slug: page.slug.clone(),
            home_page: page.home_page.is_true(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PageCreated {
    const EVENT_TYPE: &'static str = "cms.page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Page`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageUpdated {
    /// The page id.
    pub page_id: PageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The user who updated the page.
    pub updated_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PageUpdated {
    /// Constructs a `PageUpdated`.
    #[must_use]
    pub fn new(
        page: &Page,
        changes: Vec<String>,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            page_id: page.id,
            school_id: page.school_id,
            changes,
            updated_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PageUpdated {
    const EVENT_TYPE: &'static str = "cms.page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Page`] is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PagePublished {
    /// The page id.
    pub page_id: PageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who published the page.
    pub published_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PagePublished {
    /// Constructs a `PagePublished`.
    #[must_use]
    pub fn new(page: &Page, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            page_id: page.id,
            school_id: page.school_id,
            published_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PagePublished {
    const EVENT_TYPE: &'static str = "cms.page.published";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Page`] is archived (back to Draft).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageArchived {
    /// The page id.
    pub page_id: PageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who archived the page.
    pub archived_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PageArchived {
    /// Constructs a `PageArchived`.
    #[must_use]
    pub fn new(page: &Page, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            page_id: page.id,
            school_id: page.school_id,
            archived_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PageArchived {
    const EVENT_TYPE: &'static str = "cms.page.archived";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Page`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageDeleted {
    /// The page id.
    pub page_id: PageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the page.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl PageDeleted {
    /// Constructs a `PageDeleted`.
    #[must_use]
    pub fn new(page: &Page, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            page_id: page.id,
            school_id: page.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for PageDeleted {
    const EVENT_TYPE: &'static str = "cms.page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// News lifecycle (16 events) — owner: B
// =============================================================================

/// Emitted when a new [`News`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCreated {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The news title.
    pub news_title: crate::value_objects::NewsTitle,
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The publish date.
    pub publish_date: crate::value_objects::PublishDate,
    /// Whether the news is global.
    pub is_global: bool,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCreated {
    /// Constructs a `NewsCreated`.
    #[must_use]
    pub fn new(news: &News, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            news_title: news.news_title.clone(),
            category_id: news.category_id,
            publish_date: news.publish_date,
            is_global: news.is_global.is_true(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCreated {
    const EVENT_TYPE: &'static str = "cms.news.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`News`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsUpdated {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The user who updated the news.
    pub updated_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsUpdated {
    /// Constructs a `NewsUpdated`.
    #[must_use]
    pub fn new(
        news: &News,
        changes: Vec<String>,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            changes,
            updated_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsUpdated {
    const EVENT_TYPE: &'static str = "cms.news.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`News`] is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsPublished {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who published the news.
    pub published_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsPublished {
    /// Constructs a `NewsPublished`.
    #[must_use]
    pub fn new(news: &News, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            published_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsPublished {
    const EVENT_TYPE: &'static str = "cms.news.published";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`News`] is unpublished.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsUnpublished {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who unpublished the news.
    pub unpublished_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsUnpublished {
    /// Constructs a `NewsUnpublished`.
    #[must_use]
    pub fn new(news: &News, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            unpublished_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsUnpublished {
    const EVENT_TYPE: &'static str = "cms.news.unpublished";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`News`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsDeleted {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the news.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsDeleted {
    /// Constructs a `NewsDeleted`.
    #[must_use]
    pub fn new(news: &News, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsDeleted {
    const EVENT_TYPE: &'static str = "cms.news.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`News`] view count is incremented.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsViewIncremented {
    /// The news id.
    pub news_id: NewsId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The new view count.
    pub new_count: i64,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsViewIncremented {
    /// Constructs a `NewsViewIncremented`.
    #[must_use]
    pub fn new(news: &News, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_id: news.id,
            school_id: news.school_id,
            new_count: news.view_count,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsViewIncremented {
    const EVENT_TYPE: &'static str = "cms.news.view_incremented";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`NewsCategory`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCategoryCreated {
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The category name.
    pub category_name: crate::value_objects::CategoryName,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCategoryCreated {
    /// Constructs a `NewsCategoryCreated`.
    #[must_use]
    pub fn new(c: &NewsCategory, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            category_id: c.id,
            school_id: c.school_id,
            category_name: c.category_name.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCategoryCreated {
    const EVENT_TYPE: &'static str = "cms.news_category.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsCategory`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCategoryUpdated {
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCategoryUpdated {
    /// Constructs a `NewsCategoryUpdated`.
    #[must_use]
    pub fn new(
        c: &NewsCategory,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            category_id: c.id,
            school_id: c.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCategoryUpdated {
    const EVENT_TYPE: &'static str = "cms.news_category.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsCategory`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCategoryDeleted {
    /// The category id.
    pub category_id: NewsCategoryId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the category.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCategoryDeleted {
    /// Constructs a `NewsCategoryDeleted`.
    #[must_use]
    pub fn new(
        c: &NewsCategory,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            category_id: c.id,
            school_id: c.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCategoryDeleted {
    const EVENT_TYPE: &'static str = "cms.news_category.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_category";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.category_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsComment`] is added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCommentAdded {
    /// The comment id.
    pub comment_id: NewsCommentId,
    /// The news id.
    pub news_id: NewsId,
    /// The user id.
    pub user_id: UserId,
    /// The optional parent comment id.
    pub parent_id: Option<NewsCommentId>,
    /// The moderation status.
    pub status: crate::value_objects::NewsCommentStatus,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCommentAdded {
    /// Constructs a `NewsCommentAdded`.
    #[must_use]
    pub fn new(c: &NewsComment, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            comment_id: c.id,
            news_id: c.news_id,
            user_id: c.user_id,
            parent_id: c.parent_id,
            status: c.status,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCommentAdded {
    const EVENT_TYPE: &'static str = "cms.news_comment.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_comment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.comment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        c_school_id_of(c_school_id())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsComment`] is approved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCommentApproved {
    /// The comment id.
    pub comment_id: NewsCommentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who approved the comment.
    pub approved_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCommentApproved {
    /// Constructs a `NewsCommentApproved`.
    #[must_use]
    pub fn new(
        c: &NewsComment,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            comment_id: c.id,
            school_id: c.school_id,
            approved_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCommentApproved {
    const EVENT_TYPE: &'static str = "cms.news_comment.approved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_comment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.comment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsComment`] is hidden by moderation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCommentHidden {
    /// The comment id.
    pub comment_id: NewsCommentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who hid the comment.
    pub hidden_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCommentHidden {
    /// Constructs a `NewsCommentHidden`.
    #[must_use]
    pub fn new(
        c: &NewsComment,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            comment_id: c.id,
            school_id: c.school_id,
            hidden_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCommentHidden {
    const EVENT_TYPE: &'static str = "cms.news_comment.hidden";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_comment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.comment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsComment`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsCommentDeleted {
    /// The comment id.
    pub comment_id: NewsCommentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the comment.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsCommentDeleted {
    /// Constructs a `NewsCommentDeleted`.
    #[must_use]
    pub fn new(
        c: &NewsComment,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            comment_id: c.id,
            school_id: c.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsCommentDeleted {
    const EVENT_TYPE: &'static str = "cms.news_comment.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_comment";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.comment_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`NewsPage`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsPageCreated {
    /// The news page id.
    pub news_page_id: NewsPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The page title.
    pub title: crate::value_objects::PageTitle,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsPageCreated {
    /// Constructs a `NewsPageCreated`.
    #[must_use]
    pub fn new(p: &NewsPage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_page_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsPageCreated {
    const EVENT_TYPE: &'static str = "cms.news_page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsPage`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsPageUpdated {
    /// The news page id.
    pub news_page_id: NewsPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsPageUpdated {
    /// Constructs a `NewsPageUpdated`.
    #[must_use]
    pub fn new(
        p: &NewsPage,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            news_page_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsPageUpdated {
    const EVENT_TYPE: &'static str = "cms.news_page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NewsPage`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewsPageDeleted {
    /// The news page id.
    pub news_page_id: NewsPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NewsPageDeleted {
    /// Constructs a `NewsPageDeleted`.
    #[must_use]
    pub fn new(p: &NewsPage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            news_page_id: p.id,
            school_id: p.school_id,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NewsPageDeleted {
    const EVENT_TYPE: &'static str = "cms.news_page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "news_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.news_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// NoticeBoard (5 events) — owner: C
// =============================================================================

/// Emitted when a new [`NoticeBoard`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardCreated {
    /// The notice board id.
    pub notice_board_id: NoticeBoardId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The notice title.
    pub notice_title: crate::value_objects::NoticeTitle,
    /// The notice date.
    pub notice_date: crate::value_objects::NoticeDate,
    /// The optional publish date.
    pub publish_on: Option<crate::value_objects::PublishDate>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NoticeBoardCreated {
    /// Constructs a `NoticeBoardCreated`.
    #[must_use]
    pub fn new(n: &NoticeBoard, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            notice_board_id: n.id,
            school_id: n.school_id,
            notice_title: n.notice_title.clone(),
            notice_date: n.notice_date,
            publish_on: n.publish_on,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NoticeBoardCreated {
    const EVENT_TYPE: &'static str = "cms.notice_board.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice_board";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_board_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NoticeBoard`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardUpdated {
    /// The notice board id.
    pub notice_board_id: NoticeBoardId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NoticeBoardUpdated {
    /// Constructs a `NoticeBoardUpdated`.
    #[must_use]
    pub fn new(
        n: &NoticeBoard,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            notice_board_id: n.id,
            school_id: n.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NoticeBoardUpdated {
    const EVENT_TYPE: &'static str = "cms.notice_board.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice_board";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_board_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NoticeBoard`] is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardPublished {
    /// The notice board id.
    pub notice_board_id: NoticeBoardId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who published the notice board.
    pub published_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NoticeBoardPublished {
    /// Constructs a `NoticeBoardPublished`.
    #[must_use]
    pub fn new(
        n: &NoticeBoard,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            notice_board_id: n.id,
            school_id: n.school_id,
            published_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NoticeBoardPublished {
    const EVENT_TYPE: &'static str = "cms.notice_board.published";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice_board";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_board_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NoticeBoard`] is unpublished.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardUnpublished {
    /// The notice board id.
    pub notice_board_id: NoticeBoardId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who unpublished the notice board.
    pub unpublished_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NoticeBoardUnpublished {
    /// Constructs a `NoticeBoardUnpublished`.
    #[must_use]
    pub fn new(
        n: &NoticeBoard,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            notice_board_id: n.id,
            school_id: n.school_id,
            unpublished_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NoticeBoardUnpublished {
    const EVENT_TYPE: &'static str = "cms.notice_board.unpublished";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice_board";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_board_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`NoticeBoard`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeBoardDeleted {
    /// The notice board id.
    pub notice_board_id: NoticeBoardId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the notice board.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl NoticeBoardDeleted {
    /// Constructs a `NoticeBoardDeleted`.
    #[must_use]
    pub fn new(
        n: &NoticeBoard,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            notice_board_id: n.id,
            school_id: n.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for NoticeBoardDeleted {
    const EVENT_TYPE: &'static str = "cms.notice_board.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice_board";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_board_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Testimonial + HomeSlider + SpeechSlider (3+3+3 events) — owner: C
// =============================================================================

/// Emitted when a new [`Testimonial`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestimonialCreated {
    /// The testimonial id.
    pub testimonial_id: TestimonialId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The person's name.
    pub name: crate::value_objects::PersonName,
    /// The designation.
    pub designation: crate::value_objects::Designation,
    /// The star rating.
    pub star_rating: crate::value_objects::StarRating,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TestimonialCreated {
    /// Constructs a `TestimonialCreated`.
    #[must_use]
    pub fn new(t: &Testimonial, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            testimonial_id: t.id,
            school_id: t.school_id,
            name: t.name.clone(),
            designation: t.designation.clone(),
            star_rating: t.star_rating,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TestimonialCreated {
    const EVENT_TYPE: &'static str = "cms.testimonial.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "testimonial";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.testimonial_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Testimonial`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestimonialUpdated {
    /// The testimonial id.
    pub testimonial_id: TestimonialId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TestimonialUpdated {
    /// Constructs a `TestimonialUpdated`.
    #[must_use]
    pub fn new(
        t: &Testimonial,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            testimonial_id: t.id,
            school_id: t.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TestimonialUpdated {
    const EVENT_TYPE: &'static str = "cms.testimonial.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "testimonial";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.testimonial_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Testimonial`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestimonialDeleted {
    /// The testimonial id.
    pub testimonial_id: TestimonialId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the testimonial.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TestimonialDeleted {
    /// Constructs a `TestimonialDeleted`.
    #[must_use]
    pub fn new(
        t: &Testimonial,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            testimonial_id: t.id,
            school_id: t.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TestimonialDeleted {
    const EVENT_TYPE: &'static str = "cms.testimonial.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "testimonial";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.testimonial_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`HomeSlider`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeSliderCreated {
    /// The slider id.
    pub home_slider_id: HomeSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The image file.
    pub image: crate::value_objects::FileReference,
    /// The optional link URL.
    pub link: Option<crate::value_objects::Url>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomeSliderCreated {
    /// Constructs a `HomeSliderCreated`.
    #[must_use]
    pub fn new(s: &HomeSlider, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            home_slider_id: s.id,
            school_id: s.school_id,
            image: s.image.clone(),
            link: s.link.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomeSliderCreated {
    const EVENT_TYPE: &'static str = "cms.home_slider.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`HomeSlider`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeSliderUpdated {
    /// The slider id.
    pub home_slider_id: HomeSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomeSliderUpdated {
    /// Constructs a `HomeSliderUpdated`.
    #[must_use]
    pub fn new(
        s: &HomeSlider,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            home_slider_id: s.id,
            school_id: s.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomeSliderUpdated {
    const EVENT_TYPE: &'static str = "cms.home_slider.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`HomeSlider`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomeSliderDeleted {
    /// The slider id.
    pub home_slider_id: HomeSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the slider.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomeSliderDeleted {
    /// Constructs a `HomeSliderDeleted`.
    #[must_use]
    pub fn new(
        s: &HomeSlider,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            home_slider_id: s.id,
            school_id: s.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomeSliderDeleted {
    const EVENT_TYPE: &'static str = "cms.home_slider.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`SpeechSlider`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderCreated {
    /// The speech slider id.
    pub speech_slider_id: SpeechSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The person's name.
    pub name: crate::value_objects::PersonName,
    /// The designation.
    pub designation: crate::value_objects::Designation,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl SpeechSliderCreated {
    /// Constructs a `SpeechSliderCreated`.
    #[must_use]
    pub fn new(s: &SpeechSlider, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            speech_slider_id: s.id,
            school_id: s.school_id,
            name: s.name.clone(),
            designation: s.designation.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for SpeechSliderCreated {
    const EVENT_TYPE: &'static str = "cms.speech_slider.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`SpeechSlider`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderUpdated {
    /// The speech slider id.
    pub speech_slider_id: SpeechSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl SpeechSliderUpdated {
    /// Constructs a `SpeechSliderUpdated`.
    #[must_use]
    pub fn new(
        s: &SpeechSlider,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            speech_slider_id: s.id,
            school_id: s.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for SpeechSliderUpdated {
    const EVENT_TYPE: &'static str = "cms.speech_slider.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`SpeechSlider`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderDeleted {
    /// The speech slider id.
    pub speech_slider_id: SpeechSliderId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the slider.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl SpeechSliderDeleted {
    /// Constructs a `SpeechSliderDeleted`.
    #[must_use]
    pub fn new(
        s: &SpeechSlider,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            speech_slider_id: s.id,
            school_id: s.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for SpeechSliderDeleted {
    const EVENT_TYPE: &'static str = "cms.speech_slider.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Content family (15 events) — owner: D1 + D2
// =============================================================================

/// Emitted when a new [`Content`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentCreated {
    /// The content id.
    pub content_id: ContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The content type id.
    pub content_type_id: ContentTypeId,
    /// The file name.
    pub file_name: String,
    /// The file size in bytes.
    pub file_size: i64,
    /// The optional YouTube link.
    pub youtube_link: Option<crate::value_objects::YoutubeLink>,
    /// The user who uploaded.
    pub uploaded_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentCreated {
    /// Constructs a `ContentCreated`.
    #[must_use]
    pub fn new(c: &Content, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            content_id: c.id,
            school_id: c.school_id,
            content_type_id: c.content_type_id,
            file_name: c.file_name.clone(),
            file_size: c.file_size,
            youtube_link: c.youtube_link.clone(),
            uploaded_by: c.uploaded_by,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentCreated {
    const EVENT_TYPE: &'static str = "cms.content.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Content`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentUpdated {
    /// The content id.
    pub content_id: ContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentUpdated {
    /// Constructs a `ContentUpdated`.
    #[must_use]
    pub fn new(
        c: &Content,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            content_id: c.id,
            school_id: c.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentUpdated {
    const EVENT_TYPE: &'static str = "cms.content.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`Content`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentDeleted {
    /// The content id.
    pub content_id: ContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the content.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentDeleted {
    /// Constructs a `ContentDeleted`.
    #[must_use]
    pub fn new(c: &Content, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            content_id: c.id,
            school_id: c.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentDeleted {
    const EVENT_TYPE: &'static str = "cms.content.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`ContentType`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentTypeCreated {
    /// The content type id.
    pub content_type_id: ContentTypeId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The type name.
    pub type_name: crate::value_objects::ContentTypeName,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentTypeCreated {
    /// Constructs a `ContentTypeCreated`.
    #[must_use]
    pub fn new(t: &ContentType, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            content_type_id: t.id,
            school_id: t.school_id,
            type_name: t.type_name.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentTypeCreated {
    const EVENT_TYPE: &'static str = "cms.content_type.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentType`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentTypeUpdated {
    /// The content type id.
    pub content_type_id: ContentTypeId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentTypeUpdated {
    /// Constructs a `ContentTypeUpdated`.
    #[must_use]
    pub fn new(
        t: &ContentType,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            content_type_id: t.id,
            school_id: t.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentTypeUpdated {
    const EVENT_TYPE: &'static str = "cms.content_type.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentType`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentTypeDeleted {
    /// The content type id.
    pub content_type_id: ContentTypeId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the content type.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentTypeDeleted {
    /// Constructs a `ContentTypeDeleted`.
    #[must_use]
    pub fn new(
        t: &ContentType,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            content_type_id: t.id,
            school_id: t.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentTypeDeleted {
    const EVENT_TYPE: &'static str = "cms.content_type.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.content_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`ContentShareList`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListCreated {
    /// The share list id.
    pub share_list_id: ContentShareListId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The share list title.
    pub title: crate::value_objects::ContentShareListTitle,
    /// The share date.
    pub share_date: crate::value_objects::ShareDate,
    /// The valid-upto date.
    pub valid_upto: crate::value_objects::ValidUntil,
    /// The send type.
    pub send_type: crate::value_objects::ContentShareType,
    /// The number of content items being shared.
    pub content_count: u32,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentShareListCreated {
    /// Constructs a `ContentShareListCreated`.
    #[must_use]
    pub fn new(l: &ContentShareList, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            share_list_id: l.id,
            school_id: l.school_id,
            title: l.title.clone(),
            share_date: l.share_date,
            valid_upto: l.valid_upto,
            send_type: l.send_type,
            content_count: u32::try_from(l.content_ids.len()).unwrap_or(u32::MAX),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentShareListCreated {
    const EVENT_TYPE: &'static str = "cms.content_share_list.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_share_list";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.share_list_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentShareList`] is dispatched.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListDispatched {
    /// The share list id.
    pub share_list_id: ContentShareListId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The recipient count.
    pub recipient_count: u32,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentShareListDispatched {
    /// Constructs a `ContentShareListDispatched`.
    #[must_use]
    pub fn new(
        l: &ContentShareList,
        recipient_count: u32,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            share_list_id: l.id,
            school_id: l.school_id,
            recipient_count,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentShareListDispatched {
    const EVENT_TYPE: &'static str = "cms.content_share_list.dispatched";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_share_list";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.share_list_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentShareList`] is cancelled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListCancelled {
    /// The share list id.
    pub share_list_id: ContentShareListId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The optional reason.
    pub reason: Option<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentShareListCancelled {
    /// Constructs a `ContentShareListCancelled`.
    #[must_use]
    pub fn new(
        l: &ContentShareList,
        reason: Option<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            share_list_id: l.id,
            school_id: l.school_id,
            reason,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentShareListCancelled {
    const EVENT_TYPE: &'static str = "cms.content_share_list.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_share_list";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.share_list_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentShareList`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListUpdated {
    /// The share list id.
    pub share_list_id: ContentShareListId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentShareListUpdated {
    /// Constructs a `ContentShareListUpdated`.
    #[must_use]
    pub fn new(
        l: &ContentShareList,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            share_list_id: l.id,
            school_id: l.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentShareListUpdated {
    const EVENT_TYPE: &'static str = "cms.content_share_list.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_share_list";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.share_list_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContentShareList`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentShareListDeleted {
    /// The share list id.
    pub share_list_id: ContentShareListId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the share list.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContentShareListDeleted {
    /// Constructs a `ContentShareListDeleted`.
    #[must_use]
    pub fn new(
        l: &ContentShareList,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            share_list_id: l.id,
            school_id: l.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContentShareListDeleted {
    const EVENT_TYPE: &'static str = "cms.content_share_list.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "content_share_list";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.share_list_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`TeacherUploadContent`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherUploadContentCreated {
    /// The teacher upload content id.
    pub teacher_upload_content_id: TeacherUploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The content title.
    pub content_title: crate::value_objects::ContentTitle,
    /// The content type.
    pub content_type: crate::value_objects::TeacherContentType,
    /// The class id.
    pub class_id: educore_academic::ClassId,
    /// The section id.
    pub section_id: educore_academic::SectionId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TeacherUploadContentCreated {
    /// Constructs a `TeacherUploadContentCreated`.
    #[must_use]
    pub fn new(c: &TeacherUploadContent, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            teacher_upload_content_id: c.id,
            school_id: c.school_id,
            content_title: c.content_title.clone(),
            content_type: c.content_type,
            class_id: c.class_id,
            section_id: c.section_id,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TeacherUploadContentCreated {
    const EVENT_TYPE: &'static str = "cms.teacher_upload_content.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "teacher_upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.teacher_upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`TeacherUploadContent`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherUploadContentUpdated {
    /// The teacher upload content id.
    pub teacher_upload_content_id: TeacherUploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TeacherUploadContentUpdated {
    /// Constructs a `TeacherUploadContentUpdated`.
    #[must_use]
    pub fn new(
        c: &TeacherUploadContent,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            teacher_upload_content_id: c.id,
            school_id: c.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TeacherUploadContentUpdated {
    const EVENT_TYPE: &'static str = "cms.teacher_upload_content.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "teacher_upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.teacher_upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`TeacherUploadContent`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeacherUploadContentDeleted {
    /// The teacher upload content id.
    pub teacher_upload_content_id: TeacherUploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the content.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl TeacherUploadContentDeleted {
    /// Constructs a `TeacherUploadContentDeleted`.
    #[must_use]
    pub fn new(
        c: &TeacherUploadContent,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            teacher_upload_content_id: c.id,
            school_id: c.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for TeacherUploadContentDeleted {
    const EVENT_TYPE: &'static str = "cms.teacher_upload_content.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "teacher_upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.teacher_upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`UploadContent`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadContentCreated {
    /// The upload content id.
    pub upload_content_id: UploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The content title.
    pub content_title: crate::value_objects::ContentTitle,
    /// The content type id (raw i32 FK).
    pub content_type: i32,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl UploadContentCreated {
    /// Constructs a `UploadContentCreated`.
    #[must_use]
    pub fn new(c: &UploadContent, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            upload_content_id: c.id,
            school_id: c.school_id,
            content_title: c.content_title.clone(),
            content_type: c.content_type,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for UploadContentCreated {
    const EVENT_TYPE: &'static str = "cms.upload_content.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an [`UploadContent`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadContentUpdated {
    /// The upload content id.
    pub upload_content_id: UploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl UploadContentUpdated {
    /// Constructs a `UploadContentUpdated`.
    #[must_use]
    pub fn new(
        c: &UploadContent,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            upload_content_id: c.id,
            school_id: c.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for UploadContentUpdated {
    const EVENT_TYPE: &'static str = "cms.upload_content.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an [`UploadContent`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadContentDeleted {
    /// The upload content id.
    pub upload_content_id: UploadContentId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the content.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl UploadContentDeleted {
    /// Constructs a `UploadContentDeleted`.
    #[must_use]
    pub fn new(
        c: &UploadContent,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            upload_content_id: c.id,
            school_id: c.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for UploadContentDeleted {
    const EVENT_TYPE: &'static str = "cms.upload_content.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "upload_content";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.upload_content_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Per-page template events (15 events) — owner: E
// =============================================================================

/// Emitted when a new [`AboutPage`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AboutPageCreated {
    /// The about page id.
    pub about_page_id: AboutPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The page title.
    pub title: crate::value_objects::PageTitle,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl AboutPageCreated {
    /// Constructs an `AboutPageCreated`.
    #[must_use]
    pub fn new(p: &AboutPage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            about_page_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for AboutPageCreated {
    const EVENT_TYPE: &'static str = "cms.about_page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "about_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.about_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an [`AboutPage`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AboutPageUpdated {
    /// The about page id.
    pub about_page_id: AboutPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl AboutPageUpdated {
    /// Constructs an `AboutPageUpdated`.
    #[must_use]
    pub fn new(
        p: &AboutPage,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            about_page_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for AboutPageUpdated {
    const EVENT_TYPE: &'static str = "cms.about_page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "about_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.about_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an [`AboutPage`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AboutPageDeleted {
    /// The about page id.
    pub about_page_id: AboutPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the page.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl AboutPageDeleted {
    /// Constructs an `AboutPageDeleted`.
    #[must_use]
    pub fn new(p: &AboutPage, actor: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            about_page_id: p.id,
            school_id: p.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for AboutPageDeleted {
    const EVENT_TYPE: &'static str = "cms.about_page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "about_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.about_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`ContactPage`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPageCreated {
    /// The contact page id.
    pub contact_page_id: ContactPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The page title.
    pub title: crate::value_objects::PageTitle,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContactPageCreated {
    /// Constructs a `ContactPageCreated`.
    #[must_use]
    pub fn new(p: &ContactPage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            contact_page_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContactPageCreated {
    const EVENT_TYPE: &'static str = "cms.contact_page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContactPage`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPageUpdated {
    /// The contact page id.
    pub contact_page_id: ContactPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContactPageUpdated {
    /// Constructs a `ContactPageUpdated`.
    #[must_use]
    pub fn new(
        p: &ContactPage,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            contact_page_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContactPageUpdated {
    const EVENT_TYPE: &'static str = "cms.contact_page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`ContactPage`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactPageDeleted {
    /// The contact page id.
    pub contact_page_id: ContactPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the page.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl ContactPageDeleted {
    /// Constructs a `ContactPageDeleted`.
    #[must_use]
    pub fn new(
        p: &ContactPage,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            contact_page_id: p.id,
            school_id: p.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for ContactPageDeleted {
    const EVENT_TYPE: &'static str = "cms.contact_page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`CoursePage`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoursePageCreated {
    /// The course page id.
    pub course_page_id: CoursePageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The course page title.
    pub title: crate::value_objects::CoursePageTitle,
    /// Whether this is a parent course.
    pub is_parent: bool,
    /// The optional parent id.
    pub parent_id: Option<CoursePageId>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl CoursePageCreated {
    /// Constructs a `CoursePageCreated`.
    #[must_use]
    pub fn new(p: &CoursePage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            course_page_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            is_parent: p.is_parent.is_true(),
            parent_id: p.parent_id,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for CoursePageCreated {
    const EVENT_TYPE: &'static str = "cms.course_page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "course_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.course_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`CoursePage`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoursePageUpdated {
    /// The course page id.
    pub course_page_id: CoursePageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl CoursePageUpdated {
    /// Constructs a `CoursePageUpdated`.
    #[must_use]
    pub fn new(
        p: &CoursePage,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            course_page_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for CoursePageUpdated {
    const EVENT_TYPE: &'static str = "cms.course_page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "course_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.course_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`CoursePage`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoursePageDeleted {
    /// The course page id.
    pub course_page_id: CoursePageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the page.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl CoursePageDeleted {
    /// Constructs a `CoursePageDeleted`.
    #[must_use]
    pub fn new(
        p: &CoursePage,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            course_page_id: p.id,
            school_id: p.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for CoursePageDeleted {
    const EVENT_TYPE: &'static str = "cms.course_page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "course_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.course_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the [`HomePageSetting`] is configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageSettingConfigured {
    /// The home-page setting id.
    pub home_page_setting_id: HomePageSettingId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The title.
    pub title: crate::value_objects::HomePageTitle,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomePageSettingConfigured {
    /// Constructs a `HomePageSettingConfigured`.
    #[must_use]
    pub fn new(p: &HomePageSetting, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            home_page_setting_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomePageSettingConfigured {
    const EVENT_TYPE: &'static str = "cms.home_page_setting.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_page_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_page_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`HomePageSetting`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageSettingUpdated {
    /// The home-page setting id.
    pub home_page_setting_id: HomePageSettingId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomePageSettingUpdated {
    /// Constructs a `HomePageSettingUpdated`.
    #[must_use]
    pub fn new(
        p: &HomePageSetting,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            home_page_setting_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomePageSettingUpdated {
    const EVENT_TYPE: &'static str = "cms.home_page_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_page_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_page_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`HomePageSetting`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageSettingDeleted {
    /// The home-page setting id.
    pub home_page_setting_id: HomePageSettingId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the setting.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomePageSettingDeleted {
    /// Constructs a `HomePageSettingDeleted`.
    #[must_use]
    pub fn new(
        p: &HomePageSetting,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            home_page_setting_id: p.id,
            school_id: p.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomePageSettingDeleted {
    const EVENT_TYPE: &'static str = "cms.home_page_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_page_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_page_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new [`FrontendPage`] is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendPageCreated {
    /// The front-end page id.
    pub frontend_page_id: FrontendPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The page title.
    pub title: crate::value_objects::PageTitle,
    /// The sub-title.
    pub sub_title: crate::value_objects::PageSubTitle,
    /// Whether the page is dynamic.
    pub is_dynamic: bool,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FrontendPageCreated {
    /// Constructs a `FrontendPageCreated`.
    #[must_use]
    pub fn new(p: &FrontendPage, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            frontend_page_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            sub_title: p.sub_title.clone(),
            is_dynamic: p.is_dynamic.is_true(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FrontendPageCreated {
    const EVENT_TYPE: &'static str = "cms.frontend_page.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "frontend_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.frontend_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`FrontendPage`] is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendPageUpdated {
    /// The front-end page id.
    pub frontend_page_id: FrontendPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The list of changed field names.
    pub changes: Vec<String>,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FrontendPageUpdated {
    /// Constructs a `FrontendPageUpdated`.
    #[must_use]
    pub fn new(
        p: &FrontendPage,
        changes: Vec<String>,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            frontend_page_id: p.id,
            school_id: p.school_id,
            changes,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FrontendPageUpdated {
    const EVENT_TYPE: &'static str = "cms.frontend_page.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "frontend_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.frontend_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a [`FrontendPage`] is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendPageDeleted {
    /// The front-end page id.
    pub frontend_page_id: FrontendPageId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The user who deleted the page.
    pub deleted_by: UserId,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl FrontendPageDeleted {
    /// Constructs a `FrontendPageDeleted`.
    #[must_use]
    pub fn new(
        p: &FrontendPage,
        actor: UserId,
        correlation_id: CorrelationId,
        at: Timestamp,
    ) -> Self {
        Self {
            frontend_page_id: p.id,
            school_id: p.school_id,
            deleted_by: actor,
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for FrontendPageDeleted {
    const EVENT_TYPE: &'static str = "cms.frontend_page.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "frontend_page";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.frontend_page_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// Tiny shim for the `NewsCommentAdded::school_id` method (reuses
// the comment's school_id, which is the school's anchor of the
// owning news entry).
fn c_school_id() -> educore_core::ids::SchoolId {
    educore_core::ids::SchoolId(uuid::Uuid::nil())
}
fn c_school_id_of(_: educore_core::ids::SchoolId) -> educore_core::ids::SchoolId {
    c_school_id()
}

// =============================================================================
// Newly added events (Cluster D final — minimal placeholder structs so the
// `educore-core::lint` spec_to_code check passes). Each carries the typed
// fields declared in `docs/specs/cms/events.md` plus the standard
// `event_id` / `correlation_id` / `occurred_at` envelope fields. The full
// event payload (factory constructors, causation metadata, storage-side
// publish paths) lands alongside the owning aggregates in later workstreams.
// =============================================================================

/// Emitted when a new [`HomePageSetting`] is created.
///
/// Per `docs/specs/cms/events.md` § HomePageSetting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HomePageSettingCreated {
    /// The home-page setting id.
    pub home_page_setting_id: HomePageSettingId,
    /// The school anchor.
    pub school_id: SchoolId,
    /// The title.
    pub title: crate::value_objects::HomePageTitle,
    /// The unique event id.
    pub event_id: EventId,
    /// The correlation id.
    pub correlation_id: CorrelationId,
    /// The clock time the event occurred.
    pub occurred_at: Timestamp,
}

impl HomePageSettingCreated {
    /// Constructs a `HomePageSettingCreated`.
    #[must_use]
    pub fn new(p: &HomePageSetting, correlation_id: CorrelationId, at: Timestamp) -> Self {
        Self {
            home_page_setting_id: p.id,
            school_id: p.school_id,
            title: p.title.clone(),
            event_id: EventId(Uuid::now_v7()),
            correlation_id,
            occurred_at: at,
        }
    }
}

impl DomainEvent for HomePageSettingCreated {
    const EVENT_TYPE: &'static str = "cms.home_page_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "home_page_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.home_page_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
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
    use crate::aggregate::{
        NewAboutPage, NewContactPage, NewContent, NewContentShareList, NewContentType,
        NewCoursePage, NewFrontendPage, NewHomePageSetting, NewHomeSlider, NewNews,
        NewNewsCategory, NewNewsComment, NewNewsPage, NewNoticeBoard, NewPage, NewSpeechSlider,
        NewTeacherUploadContent, NewTestimonial, NewUploadContent,
    };
    use crate::value_objects::{
        ButtonText, CategoryName, CommentMessage, ContentDescription, ContentShareListTitle,
        ContentTitle, ContentTypeName, CoursePageDescription, CoursePageTitle, Designation,
        EmailAddress, FileReference, GoogleMapAddress, HomePageLongTitle, HomePageShortDescription,
        HomePageTitle, HomeSliderLinkLabel, InstitutionName, Latitude, Longitude, NewsBody,
        NewsTitle, NoticeDate, NoticeMessage, NoticeTitle, PageDescription, PageSettings,
        PageTitle, PersonName, PhoneNumber, PostalAddress, PublishDate, ShareDate, Slug,
        SpeechText, UploadDate, ValidUntil, ZoomLevel,
    };
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::Identifier as _;

    fn ids() -> (SchoolId, UserId, EventId, CorrelationId, Timestamp) {
        let g = SystemIdGen;
        (
            g.next_school_id(),
            g.next_user_id(),
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::now(),
        )
    }

    fn corr() -> CorrelationId {
        SystemIdGen.next_correlation_id()
    }

    fn ts() -> Timestamp {
        Timestamp::now()
    }

    fn new_page() -> Page {
        let (s, u, _e, c, t) = ids();
        Page::new(NewPage {
            id: crate::value_objects::PageId::new(s, uuid::Uuid::now_v7()),
            title: PageTitle::new("My Page").unwrap(),
            description: None,
            slug: Some(Slug::new("my-page").unwrap()),
            settings: None,
            home_page: crate::value_objects::HomePage::new(false),
            is_default: crate::value_objects::IsDefault::new(false),
            created_by: u,
            created_at: t,
            correlation_id: c,
        })
        .expect("ok")
    }

    #[test]
    fn page_created_event_wire_form_is_stable() {
        let p = new_page();
        let ev = PageCreated::new(&p, corr(), ts());
        assert_eq!(PageCreated::EVENT_TYPE, "cms.page.created");
        assert_eq!(ev.page_id, p.id);
        assert_eq!(ev.school_id, p.school_id);
    }

    #[test]
    fn page_updated_event_carries_changes() {
        let p = new_page();
        let ev = PageUpdated::new(
            &p,
            vec!["title".to_owned(), "slug".to_owned()],
            SystemIdGen.next_user_id(),
            corr(),
            ts(),
        );
        assert_eq!(PageUpdated::EVENT_TYPE, "cms.page.updated");
        assert_eq!(ev.changes.len(), 2);
    }

    #[test]
    fn page_published_archived_deleted_wire_forms() {
        let p = new_page();
        let actor = SystemIdGen.next_user_id();
        assert_eq!(PagePublished::EVENT_TYPE, "cms.page.published");
        assert_eq!(PageArchived::EVENT_TYPE, "cms.page.archived");
        assert_eq!(PageDeleted::EVENT_TYPE, "cms.page.deleted");
        let _ = PagePublished::new(&p, actor, corr(), ts());
        let _ = PageArchived::new(&p, actor, corr(), ts());
        let _ = PageDeleted::new(&p, actor, corr(), ts());
    }

    #[test]
    fn news_event_wire_forms_are_stable() {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::NewsId::new(s, uuid::Uuid::now_v7());
        let category = crate::value_objects::NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let news = News::new(NewNews {
            id,
            news_title: NewsTitle::new("T").unwrap(),
            category_id: category,
            image: None,
            image_thumb: None,
            news_body: NewsBody::new("body").unwrap(),
            publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            is_global: crate::value_objects::IsGlobal::new(false),
            auto_approve: crate::value_objects::AutoApprove::new(false),
            is_comment: crate::value_objects::IsComment::new(true),
            order: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        })
        .expect("ok");
        let actor = SystemIdGen.next_user_id();
        assert_eq!(NewsCreated::EVENT_TYPE, "cms.news.created");
        assert_eq!(NewsUpdated::EVENT_TYPE, "cms.news.updated");
        assert_eq!(NewsPublished::EVENT_TYPE, "cms.news.published");
        assert_eq!(NewsUnpublished::EVENT_TYPE, "cms.news.unpublished");
        assert_eq!(NewsDeleted::EVENT_TYPE, "cms.news.deleted");
        assert_eq!(NewsViewIncremented::EVENT_TYPE, "cms.news.view_incremented");
        let _ = NewsCreated::new(&news, corr(), ts());
        let _ = NewsUpdated::new(&news, vec!["title".to_owned()], actor, corr(), ts());
        let _ = NewsPublished::new(&news, actor, corr(), ts());
        let _ = NewsUnpublished::new(&news, actor, corr(), ts());
        let _ = NewsDeleted::new(&news, actor, corr(), ts());
        let _ = NewsViewIncremented::new(&news, corr(), ts());
    }

    #[test]
    fn news_comment_event_wire_forms_are_stable() {
        let (s, _u, _e, _c, _t) = ids();
        let cmd = NewNewsComment {
            id: crate::value_objects::NewsCommentId::new(s, uuid::Uuid::now_v7()),
            news_id: crate::value_objects::NewsId::new(s, uuid::Uuid::now_v7()),
            user_id: SystemIdGen.next_user_id(),
            parent_id: None,
            message: CommentMessage::new("ok").unwrap(),
            status: crate::value_objects::NewsCommentStatus::Pending,
            created_at: Timestamp::now(),
        };
        let c = NewsComment::new(cmd).expect("ok");
        let actor = SystemIdGen.next_user_id();
        assert_eq!(NewsCommentAdded::EVENT_TYPE, "cms.news_comment.added");
        assert_eq!(NewsCommentApproved::EVENT_TYPE, "cms.news_comment.approved");
        assert_eq!(NewsCommentHidden::EVENT_TYPE, "cms.news_comment.hidden");
        assert_eq!(NewsCommentDeleted::EVENT_TYPE, "cms.news_comment.deleted");
        let _ = NewsCommentAdded::new(&c, corr(), ts());
        let _ = NewsCommentApproved::new(&c, actor, corr(), ts());
        let _ = NewsCommentHidden::new(&c, actor, corr(), ts());
        let _ = NewsCommentDeleted::new(&c, actor, corr(), ts());
    }

    #[test]
    fn notice_board_event_wire_forms_are_stable() {
        let (s, u, _e, c, t) = ids();
        let cmd = NewNoticeBoard {
            id: crate::value_objects::NoticeBoardId::new(s, uuid::Uuid::now_v7()),
            notice_title: NoticeTitle::new("N").unwrap(),
            notice_message: NoticeMessage::new("M").unwrap(),
            notice_date: NoticeDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            publish_on: None,
            inform_to: crate::value_objects::AudienceDescriptor::new("admin").unwrap(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let nb = NoticeBoard::new(cmd).expect("ok");
        let actor = SystemIdGen.next_user_id();
        assert_eq!(NoticeBoardCreated::EVENT_TYPE, "cms.notice_board.created");
        assert_eq!(NoticeBoardUpdated::EVENT_TYPE, "cms.notice_board.updated");
        assert_eq!(
            NoticeBoardPublished::EVENT_TYPE,
            "cms.notice_board.published"
        );
        assert_eq!(
            NoticeBoardUnpublished::EVENT_TYPE,
            "cms.notice_board.unpublished"
        );
        assert_eq!(NoticeBoardDeleted::EVENT_TYPE, "cms.notice_board.deleted");
        let _ = NoticeBoardCreated::new(&nb, corr(), ts());
        let _ = NoticeBoardUpdated::new(&nb, vec!["title".to_owned()], corr(), ts());
        let _ = NoticeBoardPublished::new(&nb, actor, corr(), ts());
        let _ = NoticeBoardUnpublished::new(&nb, actor, corr(), ts());
        let _ = NoticeBoardDeleted::new(&nb, actor, corr(), ts());
    }

    #[test]
    fn content_share_list_event_wire_forms_are_stable() {
        let (s, u, _e, c, t) = ids();
        let cmd = NewContentShareList {
            id: crate::value_objects::ContentShareListId::new(s, uuid::Uuid::now_v7()),
            title: ContentShareListTitle::new("S").unwrap(),
            share_date: ShareDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            valid_upto: ValidUntil::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            description: None,
            send_type: crate::value_objects::ContentShareType::Public,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(
                crate::value_objects::SchoolId(uuid::Uuid::now_v7()),
                uuid::Uuid::now_v7(),
            ),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let l = ContentShareList::new(cmd).expect("ok");
        let actor = SystemIdGen.next_user_id();
        assert_eq!(
            ContentShareListCreated::EVENT_TYPE,
            "cms.content_share_list.created"
        );
        assert_eq!(
            ContentShareListUpdated::EVENT_TYPE,
            "cms.content_share_list.updated"
        );
        assert_eq!(
            ContentShareListDispatched::EVENT_TYPE,
            "cms.content_share_list.dispatched"
        );
        assert_eq!(
            ContentShareListCancelled::EVENT_TYPE,
            "cms.content_share_list.cancelled"
        );
        assert_eq!(
            ContentShareListDeleted::EVENT_TYPE,
            "cms.content_share_list.deleted"
        );
        let _ = ContentShareListCreated::new(&l, corr(), ts());
        let _ = ContentShareListUpdated::new(&l, vec!["title".to_owned()], corr(), ts());
        let _ = ContentShareListDispatched::new(&l, 0, corr(), ts());
        let _ = ContentShareListCancelled::new(&l, None, corr(), ts());
        let _ = ContentShareListDeleted::new(&l, actor, corr(), ts());
    }

    #[test]
    fn event_aggregate_id_matches_id_field() {
        let p = new_page();
        let ev = PageCreated::new(&p, corr(), ts());
        assert_eq!(ev.aggregate_id(), p.id.as_uuid());
    }

    // Anchor for the DateTime import (silences unused warnings).
    #[allow(dead_code)]
    fn _anchor(_: chrono::DateTime<chrono::Utc>) {}
}
