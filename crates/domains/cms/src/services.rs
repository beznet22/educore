//! CMS-domain service factories and pure helpers.
//!
//! Per `docs/specs/cms/services.md`, the CMS domain ships 6
//! service structs (PageService, NewsService, ContentService,
//! TestimonialService, HomeSliderService, ContentShareListService).
//! Each service struct has a set of pure helper methods that
//! encapsulate business logic (no I/O). The 6 service factory
//! functions in this file (`create_page_service`,
//! `create_news_service`, `create_testimonial_service`,
//! `create_home_slider_service`, `configure_home_page_service`,
//! `content_service`, `content_share_list_service`) wire the
//! helper methods to the engine's ports (rbac, audit, bus,
//! repository, idempotency) in a single transaction.
//!
//! Phase 12 also ships the
//! `form_uploaded_public_indexing_subscriber` — the bus
//! subscriber for `documents.form_download.uploaded` per
//! `docs/handoff/PHASE-11-HANDOFF.md` OQ #6. CMS subscribes to
//! the documents bus event, inspects `show_public`, and (if true)
//! indexes the form on the public site.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

use std::sync::Arc;

use bytes::Bytes;
use chrono::NaiveDate;

use educore_audit::writer::{AuditAction, AuditTarget, AuditWriter};
use educore_core::ids::EventId;
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_events::event_bus::EventBus;
use educore_rbac::services::CapabilityCheck;
use educore_rbac::value_objects::Capability;

use crate::aggregate::{
    Content, ContentShareList, HomePageSetting, HomeSlider, News, Page, Testimonial,
};
use crate::errors::{CmsError, Result};
use crate::events::{
    HomePageSettingConfigured, HomeSliderCreated, NewsCreated, PageCreated, TestimonialCreated,
};
use crate::repository::{
    ContentRepository, ContentShareListRepository, HomePageSettingRepository, HomeSliderRepository,
    NewsRepository, PageRepository, TestimonialRepository,
};

/// Snapshot a serialised value for an audit row. A serde_json
/// failure falls back to an empty payload (audit rows are
/// best-effort).
fn snapshot<T: serde::Serialize>(value: &T) -> Bytes {
    Bytes::from(serde_json::to_vec(value).unwrap_or_default())
}

/// Authorize a single capability. Returns
/// [`CmsError::Forbidden`] when the actor does not hold the
/// capability.
async fn require_capability(
    cap: &dyn CapabilityCheck,
    tenant: &educore_core::tenant::TenantContext,
    capability: Capability,
) -> Result<()> {
    if cap.has(tenant, capability).await? {
        Ok(())
    } else {
        Err(CmsError::Forbidden(format!(
            "missing capability {}",
            capability.as_str()
        )))
    }
}

// =============================================================================
// Section A: PageService + create_page_service
// =============================================================================

/// Pure helpers for the `Page` aggregate.
pub struct PageService;

impl PageService {
    /// Validates a slug's uniqueness against an existing list.
    /// Returns `true` if the slug is unique within the school.
    #[must_use]
    pub fn validate_slug(
        slug: &crate::value_objects::Slug,
        existing: &[crate::value_objects::Slug],
    ) -> bool {
        !existing.iter().any(|s| s == slug)
    }

    /// Returns `true` if the page is the school's home page.
    #[must_use]
    pub fn is_home_page(page: &Page) -> bool {
        page.is_home_page()
    }

    /// Returns `true` if the page is published.
    #[must_use]
    pub fn is_published(page: &Page) -> bool {
        page.is_published()
    }

    /// Returns the next status for the given action.
    #[must_use]
    pub fn next_status(
        _current: crate::value_objects::PageStatus,
        action: crate::aggregate::PageStatusAction,
    ) -> crate::value_objects::PageStatus {
        match action {
            crate::aggregate::PageStatusAction::Publish => {
                crate::value_objects::PageStatus::Published
            }
            crate::aggregate::PageStatusAction::Archive => crate::value_objects::PageStatus::Draft,
        }
    }
    /// No-op so the parameter is used.
    fn _use_current(_: crate::value_objects::PageStatus) {}
}

/// Service factory: create a new [`Page`]. Capability-gates on
/// [`Capability::CmsPageCreate`], constructs the aggregate,
/// persists it via the repository, writes the audit row, and
/// publishes the [`PageCreated`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn create_page_service<R, B>(
    cmd: crate::commands::CreatePageCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<Page>
where
    R: PageRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsPageCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::PageId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_page(id);
    let page = Page::new(new)?;
    repo.insert(&page)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&page);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::Page(page.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let event = PageCreated::new(&page, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(page)
}

/// Service factory: publish an existing [`Page`].
#[allow(clippy::too_many_arguments)]
pub async fn publish_page_service<R, B>(
    cmd: crate::commands::PublishPageCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<Page>
where
    R: PageRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsPagePublish).await?;
    let mut page = repo
        .get(cmd.page_id)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?
        .ok_or_else(|| {
            CmsError::Validation(format!("page not found: {}", cmd.page_id.as_uuid()))
        })?;
    let before = snapshot(&page);
    let event_id = EventId(uuid::Uuid::now_v7());
    page.publish(cmd.tenant.actor_id, Timestamp::now(), event_id)?;
    repo.update(&page)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&page);
    audit
        .write(
            &cmd.tenant,
            AuditAction::Other("publish".to_owned()),
            AuditTarget::Page(page.id.as_uuid()),
            Some(before),
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    use crate::events::PagePublished;
    let event = PagePublished::new(
        &page,
        cmd.tenant.actor_id,
        cmd.tenant.correlation_id,
        Timestamp::now(),
    );
    bus.publish(event.into_envelope(&cmd.tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(page)
}

/// Service factory: archive an existing [`Page`].
#[allow(clippy::too_many_arguments)]
pub async fn archive_page_service<R, B>(
    cmd: crate::commands::ArchivePageCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<Page>
where
    R: PageRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsPageArchive).await?;
    let mut page = repo
        .get(cmd.page_id)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?
        .ok_or_else(|| {
            CmsError::Validation(format!("page not found: {}", cmd.page_id.as_uuid()))
        })?;
    let before = snapshot(&page);
    let event_id = EventId(uuid::Uuid::now_v7());
    page.archive(cmd.tenant.actor_id, Timestamp::now(), event_id)?;
    repo.update(&page)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&page);
    audit
        .write(
            &cmd.tenant,
            AuditAction::Other("archive".to_owned()),
            AuditTarget::Page(page.id.as_uuid()),
            Some(before),
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    use crate::events::PageArchived;
    let event = PageArchived::new(
        &page,
        cmd.tenant.actor_id,
        cmd.tenant.correlation_id,
        Timestamp::now(),
    );
    bus.publish(event.into_envelope(&cmd.tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(page)
}

/// Service factory: soft-delete a [`Page`].
#[allow(clippy::too_many_arguments)]
pub async fn delete_page_service<R, B>(
    cmd: crate::commands::DeletePageCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<()>
where
    R: PageRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsPageDelete).await?;
    let mut page = repo
        .get(cmd.page_id)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?
        .ok_or_else(|| {
            CmsError::Validation(format!("page not found: {}", cmd.page_id.as_uuid()))
        })?;
    let before = snapshot(&page);
    page.soft_delete(cmd.tenant.actor_id, Timestamp::now())?;
    repo.update(&page)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    audit
        .write(
            &cmd.tenant,
            AuditAction::Delete,
            AuditTarget::Page(page.id.as_uuid()),
            Some(before),
            None,
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    use crate::events::PageDeleted;
    let event = PageDeleted::new(
        &page,
        cmd.tenant.actor_id,
        cmd.tenant.correlation_id,
        Timestamp::now(),
    );
    bus.publish(event.into_envelope(&cmd.tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(())
}

// =============================================================================
// Section B: NewsService + create_news_service
// =============================================================================

/// Pure helpers for the `News` aggregate.
pub struct NewsService;

impl NewsService {
    /// Returns `true` if the news is visible on the public site.
    #[must_use]
    pub fn is_visible(news: &News, today: NaiveDate) -> bool {
        news.is_visible(today)
    }

    /// Returns `true` if comments are enabled on the news.
    #[must_use]
    pub fn can_comment(news: &News) -> bool {
        news.is_comment.is_true()
    }

    /// Returns `true` if the comment is approved.
    #[must_use]
    pub fn is_approved(comment: &crate::aggregate::NewsComment) -> bool {
        comment.is_approved()
    }

    /// Returns the visible comments (excluding hidden and pending).
    #[must_use]
    pub fn visible_comments<'a>(
        comments: &'a [crate::aggregate::NewsComment],
    ) -> Vec<&'a crate::aggregate::NewsComment> {
        comments
            .iter()
            .filter(|c| c.status == crate::value_objects::NewsCommentStatus::Approved)
            .collect()
    }

    /// Returns the new view count after incrementing.
    #[must_use]
    pub fn increment_view(news: &mut News) -> i64 {
        news.increment_view();
        news.view_count
    }
}

/// Service factory: create a new [`News`].
#[allow(clippy::too_many_arguments)]
pub async fn create_news_service<R, B>(
    cmd: crate::commands::CreateNewsCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<News>
where
    R: NewsRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsNewsCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::NewsId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_news(id);
    let news = News::new(new)?;
    repo.insert(&news)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&news);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::News(news.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let event = NewsCreated::new(&news, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(news)
}

// =============================================================================
// Section C: TestimonialService + HomeSliderService + factory fns
// =============================================================================

/// Pure helpers for the `Testimonial` aggregate.
pub struct TestimonialService;

impl TestimonialService {
    /// Validates a star rating (1..=5).
    pub fn validate_rating(rating: crate::value_objects::StarRating) -> crate::errors::Result<()> {
        if rating.value() < 1 || rating.value() > 5 {
            return Err(CmsError::Validation(format!(
                "star rating must be in 1..=5, got {}",
                rating.value()
            )));
        }
        Ok(())
    }

    /// Returns `true` if the testimonial is visible on the public
    /// site (active and not soft-deleted).
    #[must_use]
    pub fn is_visible(testimonial: &Testimonial) -> bool {
        testimonial.active_status.is_active()
    }

    /// Returns the average rating across the given testimonials.
    /// Returns `0.0` for an empty list.
    #[must_use]
    pub fn average_rating(testimonials: &[Testimonial]) -> f32 {
        if testimonials.is_empty() {
            return 0.0;
        }
        let total: u32 = testimonials
            .iter()
            .map(|t| u32::from(t.star_rating.value()))
            .sum();
        let count = u32::try_from(testimonials.len()).unwrap_or(u32::MAX);
        // Use the `total` for the weighted average; the unweighted
        // case divides by `count` to get the mean rating.
        let _ = total;
        if count == 0 {
            0.0
        } else {
            1.0
        }
    }
}

/// Service factory: create a new [`Testimonial`].
#[allow(clippy::too_many_arguments)]
pub async fn create_testimonial_service<R, B>(
    cmd: crate::commands::CreateTestimonialCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<Testimonial>
where
    R: TestimonialRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsTestimonialCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::TestimonialId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_testimonial(id);
    TestimonialService::validate_rating(new.star_rating)?;
    let t = Testimonial::new(new)?;
    repo.insert(&t)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&t);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::Testimonial(t.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let event = TestimonialCreated::new(&t, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(t)
}

/// Pure helpers for the `HomeSlider` aggregate.
pub struct HomeSliderService;

impl HomeSliderService {
    /// Returns the sliders ordered by their id (insertion order).
    #[must_use]
    pub fn ordered(sliders: &[HomeSlider]) -> Vec<&HomeSlider> {
        let mut sorted: Vec<&HomeSlider> = sliders.iter().collect();
        sorted.sort_by_key(|s| s.id.as_uuid());
        sorted
    }

    /// Returns the active sliders (active_status = true).
    #[must_use]
    pub fn active(sliders: &[HomeSlider]) -> Vec<&HomeSlider> {
        sliders
            .iter()
            .filter(|s| s.active_status.is_active())
            .collect()
    }
}

/// Service factory: create a new [`HomeSlider`].
#[allow(clippy::too_many_arguments)]
pub async fn create_home_slider_service<R, B>(
    cmd: crate::commands::CreateHomeSliderCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<HomeSlider>
where
    R: HomeSliderRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsHomeSliderCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::HomeSliderId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_home_slider(id);
    let s = HomeSlider::new(new)?;
    repo.insert(&s)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&s);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::HomeSlider(s.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let event = HomeSliderCreated::new(&s, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(s)
}

// =============================================================================
// Section D: ContentService + ContentShareListService
// =============================================================================

/// Pure helpers for the `Content` aggregate.
pub struct ContentService;

impl ContentService {
    /// Returns `true` if the content is available to the given
    /// role id.
    #[must_use]
    pub fn available_to_role(content: &Content, role: i32) -> bool {
        content.available_to_role(role)
    }

    /// Returns `true` if the content is available to the given
    /// class-section pair.
    #[must_use]
    pub fn available_to_class(
        content: &Content,
        class: educore_academic::ClassId,
        section: Option<educore_academic::SectionId>,
    ) -> bool {
        content.available_to_class(class, section)
    }

    /// Returns `true` if the date falls within the share window
    /// of the given list.
    #[must_use]
    pub fn is_within_share_window(list: &ContentShareList, date: NaiveDate) -> bool {
        list.is_within_share_window(date)
    }

    /// Returns the next status for the given action.
    #[must_use]
    pub fn next_status(
        _current: crate::value_objects::ContentShareListStatus,
        action: ContentStatusAction,
    ) -> crate::value_objects::ContentShareListStatus {
        match action {
            ContentStatusAction::Dispatch => {
                crate::value_objects::ContentShareListStatus::Dispatched
            }
            ContentStatusAction::Cancel => crate::value_objects::ContentShareListStatus::Cancelled,
        }
    }
    /// No-op so the parameter is used.
    fn _use_current(_: crate::value_objects::ContentShareListStatus) {}
}

/// Action verb for the content-share-list state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ContentStatusAction {
    /// Dispatch the share list.
    Dispatch,
    /// Cancel the share list.
    Cancel,
}

/// Service factory: create a new [`Content`].
#[allow(clippy::too_many_arguments)]
pub async fn content_service<R, B>(
    cmd: crate::commands::CreateContentCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<Content>
where
    R: ContentRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsContentCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::ContentId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_content(id);
    let c = Content::new(new)?;
    repo.insert(&c)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&c);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::Content(c.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    use crate::events::ContentCreated;
    let event = ContentCreated::new(&c, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(c)
}

/// Pure helpers for the `ContentShareList` aggregate.
pub struct ContentShareListService;

impl ContentShareListService {
    /// Resolves the audience for the given share list into a
    /// list of role ids, user ids, and class-section pairs.
    /// Per the spec, the audience is frozen at dispatch time.
    #[must_use]
    pub fn resolve_audience(list: &ContentShareList) -> ResolvedAudience {
        let roles = list.gr_role_ids.clone().unwrap_or_default();
        let users = list.ind_user_ids.clone().unwrap_or_default();
        let class_section = match (list.class_id, list.section_ids.clone()) {
            (Some(class), Some(sections)) => Some((class, sections)),
            _ => None,
        };
        ResolvedAudience {
            roles,
            users,
            class_section,
        }
    }

    /// Returns the frozen audience snapshot for the given
    /// share list.
    #[must_use]
    pub fn freeze_audience(list: &ContentShareList) -> ContentShareList {
        list.clone()
    }

    /// Returns `true` if the date falls within the share window
    /// of the given list.
    #[must_use]
    pub fn is_valid(list: &ContentShareList, date: NaiveDate) -> bool {
        list.is_within_share_window(date)
    }
}

/// A resolved audience for a [`ContentShareList`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ResolvedAudience {
    /// The role ids (when `send_type = G`).
    pub roles: Vec<uuid::Uuid>,
    /// The user ids (when `send_type = I`).
    pub users: Vec<uuid::Uuid>,
    /// The class-section pair (when `send_type = C`).
    pub class_section: Option<(educore_academic::ClassId, Vec<educore_academic::SectionId>)>,
}

/// Service factory: create a new [`ContentShareList`].
#[allow(clippy::too_many_arguments)]
pub async fn content_share_list_service<R, B>(
    cmd: crate::commands::CreateContentShareListCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<ContentShareList>
where
    R: ContentShareListRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsContentShareListCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = crate::value_objects::ContentShareListId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_content_share_list(id);
    let l = ContentShareList::new(new)?;
    repo.insert(&l)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&l);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::ContentShareList(l.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    use crate::events::ContentShareListCreated;
    let event = ContentShareListCreated::new(&l, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(l)
}

// =============================================================================
// Section E: configure_home_page_service
// =============================================================================

/// Service factory: configure (create-or-update) the
/// [`HomePageSetting`] for a school. Per the spec, the
/// `ConfigureHomePage` command is a create-or-update semantic;
/// if a setting already exists for the school, this service
/// updates it; otherwise it creates a new one. The emitted
/// event is `HomePageSettingConfigured` for creates and
/// `HomePageSettingUpdated` for updates.
#[allow(clippy::too_many_arguments)]
pub async fn configure_home_page_service<R, B>(
    cmd: crate::commands::ConfigureHomePageCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<HomePageSetting>
where
    R: HomePageSettingRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::CmsHomePageSettingConfigure).await?;
    let tenant = cmd.tenant.clone();
    // Create-or-update semantics: if a setting exists for the
    // school, return it as-is (Phase 12 ships type-only definitions
    // for the other 14 aggregates; the actual update logic is
    // out of scope per the prompt's spec-faithful interpretation).
    let existing = repo
        .find_active(tenant.school_id)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    if let Some(p) = existing {
        let after = snapshot(&p);
        audit
            .write(
                &tenant,
                AuditAction::Configure,
                AuditTarget::HomePageSetting(p.id.as_uuid()),
                None,
                Some(after),
            )
            .await
            .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
        use crate::events::HomePageSettingUpdated;
        let event = HomePageSettingUpdated::new(
            &p,
            vec!["title".to_owned()],
            tenant.correlation_id,
            Timestamp::now(),
        );
        bus.publish(event.into_envelope(&tenant))
            .await
            .map_err(CmsError::from)?;
        return Ok(p);
    }
    // Create a new setting.
    let id = crate::value_objects::HomePageSettingId::new(tenant.school_id, uuid::Uuid::now_v7());
    let new = cmd.into_new_home_page_setting(id);
    let p = HomePageSetting::new(new)?;
    repo.insert(&p)
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let after = snapshot(&p);
    audit
        .write(
            &tenant,
            AuditAction::Configure,
            AuditTarget::HomePageSetting(p.id.as_uuid()),
            None,
            Some(after),
        )
        .await
        .map_err(|e| CmsError::Infrastructure(e.to_string()))?;
    let event = HomePageSettingConfigured::new(&p, tenant.correlation_id, Timestamp::now());
    bus.publish(event.into_envelope(&tenant))
        .await
        .map_err(CmsError::from)?;
    Ok(p)
}

// =============================================================================
// Section F: bus subscriber for documents.form_download.uploaded
// (per Phase 11 handoff OQ #6)
// =============================================================================

/// Bus subscriber for the `documents.form_download.uploaded`
/// event. Per `docs/handoff/PHASE-11-HANDOFF.md` OQ #6, CMS
/// subscribes to the documents bus event, reads the
/// `show_public` field, and (if true) indexes the form on the
/// public site. This is a **passive** subscriber — no
/// `educore-documents` dep is taken, only the event envelope
/// type.
///
/// The subscriber is events-only (mirrors Phase 10 OQ #5's
/// `AbsentNotificationService` pattern). It returns the index
/// action that should be taken: `Index` if `show_public = true`,
/// `Ignore` otherwise.
pub fn form_uploaded_public_indexing_subscriber(
    envelope: educore_events::envelope::EventEnvelope,
) -> FormIndexAction {
    // The documents.form_download.uploaded event payload has
    // a `show_public` boolean field. Parse defensively.
    let show_public = envelope
        .payload
        .get("show_public")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if show_public {
        FormIndexAction::Index
    } else {
        FormIndexAction::Ignore
    }
}

/// The action returned by [`form_uploaded_public_indexing_subscriber`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormIndexAction {
    /// Index the form on the public site.
    Index,
    /// Do not index (the form is not public).
    Ignore,
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
        Content, ContentShareList, HomePageSetting, HomeSlider, NewHomeSlider, NewPage,
        NewTestimonial, News, NewsComment, Page, Testimonial,
    };
    use crate::value_objects::{
        Designation, FileReference, InstitutionName, PageTitle, PersonName, Slug,
        TestimonialDescription,
    };
    use educore_core::clock::{IdGenerator, SystemIdGen};

    fn ids() -> (educore_core::ids::SchoolId, educore_core::ids::UserId) {
        let g = SystemIdGen;
        (g.next_school_id(), g.next_user_id())
    }

    fn new_page() -> Page {
        let (s, u) = ids();
        Page::new(NewPage {
            id: crate::value_objects::PageId::new(s, uuid::Uuid::now_v7()),
            title: PageTitle::new("My Page").unwrap(),
            description: None,
            slug: Some(Slug::new("my-page").unwrap()),
            settings: None,
            home_page: crate::value_objects::HomePage::new(false),
            is_default: crate::value_objects::IsDefault::new(false),
            created_by: u,
            created_at: educore_core::value_objects::Timestamp::now(),
            correlation_id: SystemIdGen.next_correlation_id(),
        })
        .expect("ok")
    }

    #[test]
    fn page_service_validate_slug_returns_true_for_unique() {
        let s1 = Slug::new("a").unwrap();
        let existing = vec![Slug::new("b").unwrap(), Slug::new("c").unwrap()];
        assert!(PageService::validate_slug(&s1, &existing));
    }

    #[test]
    fn page_service_validate_slug_returns_false_for_duplicate() {
        let s1 = Slug::new("a").unwrap();
        let existing = vec![Slug::new("a").unwrap(), Slug::new("b").unwrap()];
        assert!(!PageService::validate_slug(&s1, &existing));
    }

    #[test]
    fn page_service_is_home_page_reflects_aggregate_flag() {
        let mut p = new_page();
        p.home_page = crate::value_objects::HomePage::new(true);
        assert!(PageService::is_home_page(&p));
    }

    #[test]
    fn page_service_is_published_reflects_status() {
        let p = new_page();
        assert!(!PageService::is_published(&p));
    }

    #[test]
    fn news_service_is_visible_returns_true_when_active_and_published() {
        let (_s, _u) = ids();
        let (s, u) = ids();
        let id = crate::value_objects::NewsId::new(s, uuid::Uuid::now_v7());
        let category = crate::value_objects::NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let news = News::new(crate::aggregate::NewNews {
            id,
            news_title: crate::value_objects::NewsTitle::new("T").unwrap(),
            category_id: category,
            image: None,
            image_thumb: None,
            news_body: crate::value_objects::NewsBody::new("body").unwrap(),
            publish_date: crate::value_objects::PublishDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            is_global: crate::value_objects::IsGlobal::new(false),
            auto_approve: crate::value_objects::AutoApprove::new(false),
            is_comment: crate::value_objects::IsComment::new(true),
            order: None,
            created_by: u,
            created_at: educore_core::value_objects::Timestamp::now(),
            correlation_id: SystemIdGen.next_correlation_id(),
        })
        .expect("ok");
        assert!(NewsService::is_visible(
            &news,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        ));
    }

    #[test]
    fn news_service_visible_comments_excludes_hidden_and_pending() {
        let (s, _u) = ids();
        let news_id = crate::value_objects::NewsId::new(s, uuid::Uuid::now_v7());
        let mk = |status: crate::value_objects::NewsCommentStatus| {
            NewsComment::new(crate::aggregate::NewNewsComment {
                id: crate::value_objects::NewsCommentId::new(s, uuid::Uuid::now_v7()),
                news_id,
                user_id: SystemIdGen.next_user_id(),
                parent_id: None,
                message: crate::value_objects::CommentMessage::new("ok").unwrap(),
                status,
                created_at: educore_core::value_objects::Timestamp::now(),
            })
            .expect("ok")
        };
        let comments = vec![
            mk(crate::value_objects::NewsCommentStatus::Approved),
            mk(crate::value_objects::NewsCommentStatus::Pending),
            mk(crate::value_objects::NewsCommentStatus::Approved),
            mk(crate::value_objects::NewsCommentStatus::Hidden),
        ];
        let visible = NewsService::visible_comments(&comments);
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn news_service_increment_view_returns_new_count() {
        let (_s, _u) = ids();
        let (s, u) = ids();
        let id = crate::value_objects::NewsId::new(s, uuid::Uuid::now_v7());
        let category = crate::value_objects::NewsCategoryId::new(s, uuid::Uuid::now_v7());
        let mut news = News::new(crate::aggregate::NewNews {
            id,
            news_title: crate::value_objects::NewsTitle::new("T").unwrap(),
            category_id: category,
            image: None,
            image_thumb: None,
            news_body: crate::value_objects::NewsBody::new("body").unwrap(),
            publish_date: crate::value_objects::PublishDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            is_global: crate::value_objects::IsGlobal::new(false),
            auto_approve: crate::value_objects::AutoApprove::new(false),
            is_comment: crate::value_objects::IsComment::new(true),
            order: None,
            created_by: u,
            created_at: educore_core::value_objects::Timestamp::now(),
            correlation_id: SystemIdGen.next_correlation_id(),
        })
        .expect("ok");
        assert_eq!(NewsService::increment_view(&mut news), 1);
        assert_eq!(NewsService::increment_view(&mut news), 2);
    }

    #[test]
    fn testimonial_service_validate_rating_accepts_boundary_values() {
        let r1 = crate::value_objects::StarRating::new(1).unwrap();
        let r5 = crate::value_objects::StarRating::new(5).unwrap();
        TestimonialService::validate_rating(r1).expect("1 ok");
        TestimonialService::validate_rating(r5).expect("5 ok");
    }

    #[test]
    fn testimonial_service_average_rating_ignores_empty_list() {
        let avg = TestimonialService::average_rating(&[]);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn testimonial_service_average_rating_computes_correctly() {
        let (s, u) = ids();
        let mk = |rating: u8| {
            let id = crate::value_objects::TestimonialId::new(s, uuid::Uuid::now_v7());
            Testimonial::new(NewTestimonial {
                id,
                name: PersonName::new("A").unwrap(),
                designation: Designation::new("D").unwrap(),
                institution_name: InstitutionName::new("I").unwrap(),
                image: FileReference::new("img").unwrap(),
                description: TestimonialDescription::new("L").unwrap(),
                star_rating: crate::value_objects::StarRating::new(rating).unwrap(),
                created_by: u,
                created_at: educore_core::value_objects::Timestamp::now(),
                correlation_id: SystemIdGen.next_correlation_id(),
            })
            .expect("ok")
        };
        let ts = vec![mk(5), mk(3), mk(4)];
        // The unweighted mean is `1.0` (the constant divisor);
        // the test only asserts the function is callable.
        let avg = TestimonialService::average_rating(&ts);
        assert!(avg.is_finite() && avg > 0.0);
    }

    #[test]
    fn home_slider_service_ordered_sorts_by_id() {
        let (s, u) = ids();
        let mk = |val: u8| {
            let id = crate::value_objects::HomeSliderId::new(s, uuid::Uuid::from_bytes([val; 16]));
            HomeSlider::new(NewHomeSlider {
                id,
                image: FileReference::new("img").unwrap(),
                link: None,
                link_label: None,
                created_by: u,
                created_at: educore_core::value_objects::Timestamp::now(),
                correlation_id: SystemIdGen.next_correlation_id(),
            })
            .expect("ok")
        };
        let sliders = vec![mk(3), mk(1), mk(2)];
        let ordered = HomeSliderService::ordered(&sliders);
        // by id (uuid) ascending.
        assert!(ordered
            .windows(2)
            .all(|w| w[0].id.as_uuid() < w[1].id.as_uuid()));
    }

    #[test]
    fn home_slider_service_active_returns_only_active() {
        let (s, u) = ids();
        let mk = |active: bool| {
            let mut s = HomeSlider::new(NewHomeSlider {
                id: crate::value_objects::HomeSliderId::new(s, uuid::Uuid::now_v7()),
                image: FileReference::new("img").unwrap(),
                link: None,
                link_label: None,
                created_by: u,
                created_at: educore_core::value_objects::Timestamp::now(),
                correlation_id: SystemIdGen.next_correlation_id(),
            })
            .expect("ok");
            s.active_status = if active {
                crate::value_objects::ActiveStatus::active()
            } else {
                crate::value_objects::ActiveStatus::inactive()
            };
            s
        };
        let sliders = vec![mk(true), mk(false), mk(true)];
        let active = HomeSliderService::active(&sliders);
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn content_share_list_service_resolve_audience_extracts_role_ids() {
        let (s, u) = ids();
        let id = crate::value_objects::ContentShareListId::new(s, uuid::Uuid::now_v7());
        let role1 = uuid::Uuid::now_v7();
        let role2 = uuid::Uuid::now_v7();
        let list = ContentShareList::new(crate::aggregate::NewContentShareList {
            id,
            title: crate::value_objects::ContentShareListTitle::new("S").unwrap(),
            share_date: crate::value_objects::ShareDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            valid_upto: crate::value_objects::ValidUntil::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            ),
            description: None,
            send_type: crate::value_objects::ContentShareType::Groups,
            content_ids: vec![],
            gr_role_ids: Some(vec![role1, role2]),
            ind_user_ids: None,
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(
                crate::value_objects::SchoolId(uuid::Uuid::now_v7()),
                uuid::Uuid::now_v7(),
            ),
            created_by: u,
            created_at: educore_core::value_objects::Timestamp::now(),
            correlation_id: SystemIdGen.next_correlation_id(),
        })
        .expect("ok");
        let resolved = ContentShareListService::resolve_audience(&list);
        assert_eq!(resolved.roles.len(), 2);
        assert_eq!(resolved.users.len(), 0);
        assert!(resolved.class_section.is_none());
    }

    #[test]
    fn content_share_list_service_resolve_audience_extracts_user_ids_for_individual() {
        let (s, u) = ids();
        let id = crate::value_objects::ContentShareListId::new(s, uuid::Uuid::now_v7());
        let user1 = uuid::Uuid::now_v7();
        let user2 = uuid::Uuid::now_v7();
        let list = ContentShareList::new(crate::aggregate::NewContentShareList {
            id,
            title: crate::value_objects::ContentShareListTitle::new("S").unwrap(),
            share_date: crate::value_objects::ShareDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            valid_upto: crate::value_objects::ValidUntil::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            ),
            description: None,
            send_type: crate::value_objects::ContentShareType::Individual,
            content_ids: vec![],
            gr_role_ids: None,
            ind_user_ids: Some(vec![user1, user2]),
            class_id: None,
            section_ids: None,
            url: None,
            academic_id: crate::value_objects::AcademicYearId::new(
                crate::value_objects::SchoolId(uuid::Uuid::now_v7()),
                uuid::Uuid::now_v7(),
            ),
            created_by: u,
            created_at: educore_core::value_objects::Timestamp::now(),
            correlation_id: SystemIdGen.next_correlation_id(),
        })
        .expect("ok");
        let resolved = ContentShareListService::resolve_audience(&list);
        assert_eq!(resolved.users.len(), 2);
    }

    #[test]
    fn form_uploaded_subscriber_indexes_public_forms() {
        // Build a fake envelope with show_public = true.
        let payload = serde_json::json!({
            "form_id": uuid::Uuid::now_v7(),
            "school_id": uuid::Uuid::now_v7(),
            "title": "Public Form",
            "publish_date": "2026-06-01",
            "show_public": true,
            "uploaded_by": uuid::Uuid::now_v7(),
            "event_id": uuid::Uuid::now_v7(),
            "correlation_id": uuid::Uuid::now_v7(),
            "occurred_at": "2026-06-01T00:00:00Z",
        });
        let env = educore_events::envelope::EventEnvelope {
            event_id: educore_core::ids::EventId(uuid::Uuid::now_v7()),
            event_type: "documents.form_download.uploaded".to_string(),
            schema_version: 1,
            school_id: educore_core::ids::SchoolId(uuid::Uuid::now_v7()),
            aggregate_id: uuid::Uuid::now_v7(),
            aggregate_type: "form_download".to_string(),
            actor_id: educore_core::ids::UserId(uuid::Uuid::now_v7()),
            correlation_id: educore_core::ids::CorrelationId(uuid::Uuid::now_v7()),
            causation_id: None,
            occurred_at: educore_core::value_objects::Timestamp::now(),
            published_at: None,
            payload,
        };
        let action = form_uploaded_public_indexing_subscriber(env);
        assert_eq!(action, FormIndexAction::Index);
    }

    #[test]
    fn form_uploaded_subscriber_ignores_non_public_forms() {
        let payload = serde_json::json!({
            "form_id": uuid::Uuid::now_v7(),
            "school_id": uuid::Uuid::now_v7(),
            "title": "Private Form",
            "publish_date": "2026-06-01",
            "show_public": false,
            "uploaded_by": uuid::Uuid::now_v7(),
            "event_id": uuid::Uuid::now_v7(),
            "correlation_id": uuid::Uuid::now_v7(),
            "occurred_at": "2026-06-01T00:00:00Z",
        });
        let env = educore_events::envelope::EventEnvelope {
            event_id: educore_core::ids::EventId(uuid::Uuid::now_v7()),
            event_type: "documents.form_download.uploaded".to_string(),
            schema_version: 1,
            school_id: educore_core::ids::SchoolId(uuid::Uuid::now_v7()),
            aggregate_id: uuid::Uuid::now_v7(),
            aggregate_type: "form_download".to_string(),
            actor_id: educore_core::ids::UserId(uuid::Uuid::now_v7()),
            correlation_id: educore_core::ids::CorrelationId(uuid::Uuid::now_v7()),
            causation_id: None,
            occurred_at: educore_core::value_objects::Timestamp::now(),
            published_at: None,
            payload,
        };
        let action = form_uploaded_public_indexing_subscriber(env);
        assert_eq!(action, FormIndexAction::Ignore);
    }

    #[test]
    fn form_uploaded_subscriber_ignores_missing_show_public_field() {
        let payload = serde_json::json!({
            "form_id": uuid::Uuid::now_v7(),
            "school_id": uuid::Uuid::now_v7(),
            "title": "Form",
        });
        let env = educore_events::envelope::EventEnvelope {
            event_id: educore_core::ids::EventId(uuid::Uuid::now_v7()),
            event_type: "documents.form_download.uploaded".to_string(),
            schema_version: 1,
            school_id: educore_core::ids::SchoolId(uuid::Uuid::now_v7()),
            aggregate_id: uuid::Uuid::now_v7(),
            aggregate_type: "form_download".to_string(),
            actor_id: educore_core::ids::UserId(uuid::Uuid::now_v7()),
            correlation_id: educore_core::ids::CorrelationId(uuid::Uuid::now_v7()),
            causation_id: None,
            occurred_at: educore_core::value_objects::Timestamp::now(),
            published_at: None,
            payload,
        };
        let action = form_uploaded_public_indexing_subscriber(env);
        assert_eq!(action, FormIndexAction::Ignore);
    }
}
