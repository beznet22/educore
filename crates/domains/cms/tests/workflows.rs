//! Integration tests for the **CMS domain workflows**.
//!
//! Implements: `docs/specs/cms/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the CMS aggregate methods and asserts that the
//! expected typed event is emitted (or, on the error path,
//! that the expected [`CmsError`] is returned and no event is
//! produced).
//!
//! The tests are written as **pure synchronous** tests: the CMS
//! aggregate methods (`Page::new`, `page.publish`,
//! `page.archive`, `page.soft_delete`, `News::new`,
//! `news.publish`, `news.unpublish`, `news.soft_delete`,
//! `ContentShareList::new`, `list.dispatch`, `list.cancel`)
//! are sync, take a `Timestamp` + `EventId`, and return
//! `Result<(), CmsError>` for state-machine transitions. The
//! test wires a [`TestClock`] and a [`SystemIdGen`], and
//! constructs the typed events directly from the aggregate +
//! clock instant to verify the event payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **aggregate layer** that the service
//! factory fns (`create_page_service`, `publish_page_service`,
//! etc.) and the eventual dispatcher wrap. When the handlers
//! land, the same test bodies will gain a `+ outbox + bus
//! subscriber` assertion without changes to the assertions on
//! the returned event.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use educore_cms::aggregate::{NewContent, NewContentShareList, NewNews, NewPage};
use educore_cms::prelude::*;
use educore_cms::value_objects::{HomePage, IsDefault};
use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::CorrelationId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Test fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a freshly-minted school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn page_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> PageId {
    PageId::new(school, g.next_uuid())
}

fn news_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> NewsId {
    NewsId::new(school, g.next_uuid())
}

fn news_category_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> NewsCategoryId {
    NewsCategoryId::new(school, g.next_uuid())
}

fn content_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ContentId {
    ContentId::new(school, g.next_uuid())
}

fn content_type_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ContentTypeId {
    ContentTypeId::new(school, g.next_uuid())
}

fn share_list_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ContentShareListId {
    ContentShareListId::new(school, g.next_uuid())
}

fn academic_year_id(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> educore_academic::AcademicYearId {
    educore_academic::AcademicYearId::new(school, g.next_uuid())
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Construct a fresh draft `Page` aggregate for a given school + actor.
fn new_draft_page(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    slug: Option<&str>,
    home_page: bool,
    is_default: bool,
) -> Page {
    let at = Timestamp::now();
    Page::new(NewPage {
        id: page_id(g, school),
        title: PageTitle::new(title).unwrap(),
        description: None,
        slug: slug.map(|s| Slug::new(s).unwrap()),
        settings: None,
        home_page: HomePage::new(home_page),
        is_default: IsDefault::new(is_default),
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("Page::new must succeed for valid title")
}

/// Construct a fresh `News` aggregate for a given school + actor.
#[allow(clippy::too_many_arguments)]
fn new_news_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    category: NewsCategoryId,
    title: &str,
    body: &str,
    auto_approve: bool,
    is_comment: bool,
) -> News {
    let at = Timestamp::now();
    News::new(NewNews {
        id: news_id(g, school),
        news_title: NewsTitle::new(title).unwrap(),
        category_id: category,
        image: None,
        image_thumb: None,
        news_body: NewsBody::new(body).unwrap(),
        publish_date: PublishDate::new(date(2026, 6, 1)),
        is_global: IsGlobal::new(false),
        auto_approve: AutoApprove::new(auto_approve),
        is_comment: IsComment::new(is_comment),
        order: None,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("News::new must succeed for valid title")
}

/// Construct a fresh `Content` aggregate for a given school + actor.
fn new_content_aggregate(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    content_type: ContentTypeId,
    file_name: &str,
    file_size: i64,
    academic: educore_academic::AcademicYearId,
) -> Content {
    let at = Timestamp::now();
    Content::new(NewContent {
        id: content_id(g, school),
        file_name: file_name.to_owned(),
        file_size,
        content_type_id: content_type,
        youtube_link: None,
        upload_file: None,
        available_for_role: None,
        available_for_class: None,
        available_for_section: None,
        academic_id: academic,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("Content::new must succeed for valid file_name")
}

// =============================================================================
// 1. Page Lifecycle Workflow (`workflows.md` § "Page Publishing Workflow")
// =============================================================================

/// Page lifecycle step 1: creating a page emits
/// [`PageCreated`] with the supplied title and slug.
#[test]
fn page_lifecycle_create_emits_page_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let page = new_draft_page(
        &g,
        school,
        actor,
        "About Us",
        Some("about-us"),
        false,
        false,
    );
    let event: PageCreated = PageCreated::new(&page, correlation, clock.now());

    assert_eq!(<PageCreated as DomainEvent>::EVENT_TYPE, "cms.page.created");
    assert_eq!(event.school_id, school);
    assert_eq!(event.title.as_str(), "About Us");
    assert_eq!(event.slug.as_ref().unwrap().as_str(), "about-us");
    assert!(!event.home_page);
    assert_eq!(event.page_id, page.id);
}

/// Page lifecycle step 3: publishing a draft page must emit
/// [`PagePublished`] and transition the aggregate to
/// `PageStatus::Published`.
#[test]
fn page_lifecycle_publish_transitions_to_published() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut page = new_draft_page(
        &g,
        school,
        actor,
        "Admissions",
        Some("admissions"),
        false,
        false,
    );
    assert!(matches!(page.status, PageStatus::Draft));

    page.publish(actor, clock.now(), g.next_event_id()).unwrap();

    let event: PagePublished = PagePublished::new(&page, actor, correlation, clock.now());

    assert_eq!(
        <PagePublished as DomainEvent>::EVENT_TYPE,
        "cms.page.published"
    );
    assert!(matches!(page.status, PageStatus::Published));
    assert!(page.is_published());
    assert!(!page.is_home_page());
    assert_eq!(event.published_by, actor);
    assert_eq!(event.page_id, page.id);
}

/// Page lifecycle step 5: archiving a published page must emit
/// [`PageArchived`] and transition the aggregate back to
/// `PageStatus::Draft` (per the `PageStatusAction::Archive`
/// contract: archive moves a published page back to Draft).
#[test]
fn page_lifecycle_archive_transitions_back_to_draft() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut page = new_draft_page(&g, school, actor, "History", Some("history"), false, false);
    page.publish(actor, clock.now(), g.next_event_id()).unwrap();
    assert!(matches!(page.status, PageStatus::Published));

    page.archive(actor, clock.now(), g.next_event_id()).unwrap();

    let event: PageArchived = PageArchived::new(&page, actor, correlation, clock.now());

    assert_eq!(
        <PageArchived as DomainEvent>::EVENT_TYPE,
        "cms.page.archived"
    );
    assert!(matches!(page.status, PageStatus::Draft));
    assert!(!page.is_published());
    assert_eq!(event.archived_by, actor);
}

/// Page lifecycle end-of-life: a non-default page can be
/// soft-deleted (transitions to `ActiveStatus::inactive`),
/// emitting [`PageDeleted`].
#[test]
fn page_lifecycle_soft_delete_emits_page_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut page = new_draft_page(
        &g,
        school,
        actor,
        "Old Page",
        Some("old-page"),
        false,
        false,
    );
    assert!(page.is_active());

    page.soft_delete(actor, clock.now()).unwrap();

    let event: PageDeleted = PageDeleted::new(&page, actor, correlation, clock.now());

    assert_eq!(<PageDeleted as DomainEvent>::EVENT_TYPE, "cms.page.deleted");
    assert!(!page.is_active());
    assert_eq!(event.deleted_by, actor);
}

/// Page lifecycle failure path: per spec invariant 5, a default
/// page (`is_default = true`) cannot be deleted. The aggregate
/// must reject the soft-delete with `CmsError::DefaultPageNotDeletable`
/// and the page must remain active.
#[test]
fn page_lifecycle_default_page_cannot_be_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut page = new_draft_page(
        &g,
        school,
        actor,
        "Default Page",
        Some("default"),
        false,
        true,
    );
    assert!(page.is_active());

    let err = page
        .soft_delete(actor, clock.now())
        .expect_err("default page must not be deletable");
    assert!(
        matches!(err, CmsError::DefaultPageNotDeletable(_)),
        "got {err:?}"
    );
    // Page must remain active after a rejected soft-delete.
    assert!(page.is_active());
}

/// Page lifecycle failure path: per spec invariant 1, a page
/// title must be non-empty. `PageTitle::new` must reject empty
/// titles so that `Page::new` can never receive an empty title.
#[test]
fn page_lifecycle_empty_title_returns_validation_error() {
    let res = PageTitle::new(String::new());
    assert!(res.is_err(), "empty PageTitle must fail validation");
}

// =============================================================================
// 2. News Article Lifecycle (`workflows.md` § "News Lifecycle Workflow")
// =============================================================================

/// News lifecycle step 1: creating a news article emits
/// [`NewsCreated`] with the supplied title, category, and
/// publish date.
#[test]
fn news_lifecycle_create_emits_news_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let category = news_category_id(&g, school);

    let news = new_news_aggregate(
        &g,
        school,
        actor,
        category,
        "Welcome back to school",
        "The new academic year begins on Monday.",
        true,
        true,
    );

    let event: NewsCreated = NewsCreated::new(&news, correlation, clock.now());

    assert_eq!(<NewsCreated as DomainEvent>::EVENT_TYPE, "cms.news.created");
    assert_eq!(event.school_id, school);
    assert_eq!(event.news_title.as_str(), "Welcome back to school");
    assert_eq!(event.category_id, category);
    assert!(!event.is_global);
    assert_eq!(event.news_id, news.id);
}

/// News lifecycle steps 3-4: publishing a news article emits
/// [`NewsPublished`], and the `is_visible` predicate returns
/// `true` on or after the publish date.
#[test]
fn news_lifecycle_publish_emits_news_published_and_is_visible() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let category = news_category_id(&g, school);

    let mut news = new_news_aggregate(
        &g,
        school,
        actor,
        category,
        "Sports Day",
        "Annual sports day highlights.",
        false,
        true,
    );

    news.publish(actor, clock.now(), g.next_event_id());
    let event: NewsPublished = NewsPublished::new(&news, actor, correlation, clock.now());

    assert_eq!(
        <NewsPublished as DomainEvent>::EVENT_TYPE,
        "cms.news.published"
    );
    assert_eq!(event.published_by, actor);
    assert_eq!(event.news_id, news.id);

    // Visible on the publish date and after.
    assert!(news.is_visible(date(2026, 6, 1)));
    assert!(news.is_visible(date(2026, 7, 15)));
    // Not visible before the publish date.
    assert!(!news.is_visible(date(2026, 5, 31)));
}

/// News lifecycle step 4: a published news view count
/// increments monotonically (per spec invariant 8: view count
/// is non-decreasing).
#[test]
fn news_lifecycle_view_count_increments_monotonically() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let category = news_category_id(&g, school);

    let mut news = new_news_aggregate(
        &g,
        school,
        actor,
        category,
        "Library Week",
        "Celebrating our library volunteers.",
        false,
        true,
    );
    news.publish(actor, clock.now(), g.next_event_id());

    assert_eq!(news.view_count, 0);
    news.increment_view();
    news.increment_view();
    news.increment_view();
    assert_eq!(news.view_count, 3);

    let event: NewsViewIncremented = NewsViewIncremented::new(&news, correlation, clock.now());
    assert_eq!(
        <NewsViewIncremented as DomainEvent>::EVENT_TYPE,
        "cms.news.view_incremented"
    );
    assert_eq!(event.new_count, 3);
}

/// News lifecycle step 7: unpublishing a news article emits
/// [`NewsUnpublished`]. The aggregate retains its
/// `active_status = Active` (unpublish is a logical state,
/// not a soft-delete).
#[test]
fn news_lifecycle_unpublish_emits_news_unpublished() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let category = news_category_id(&g, school);

    let mut news = new_news_aggregate(
        &g,
        school,
        actor,
        category,
        "Exam Schedule",
        "Mid-term exams begin next week.",
        false,
        false,
    );
    news.publish(actor, clock.now(), g.next_event_id());

    news.unpublish(actor, clock.now(), g.next_event_id());
    let event: NewsUnpublished = NewsUnpublished::new(&news, actor, correlation, clock.now());

    assert_eq!(
        <NewsUnpublished as DomainEvent>::EVENT_TYPE,
        "cms.news.unpublished"
    );
    assert_eq!(event.unpublished_by, actor);
    // Soft-delete flag unchanged — unpublish is a logical state.
    assert!(news.active_status.is_active());
}

/// News lifecycle failure path: per spec invariant 1, a news
/// title must be non-empty. `NewsTitle::new` must reject empty
/// titles so that `News::new` can never receive an empty title.
#[test]
fn news_lifecycle_empty_title_returns_validation_error() {
    let res = NewsTitle::new(String::new());
    assert!(res.is_err(), "empty NewsTitle must fail validation");
}

// =============================================================================
// 3. Content Workflow (`workflows.md` § "Content Sharing Workflow")
//
// The spec's "Content Sharing Workflow" is the closest
// review-approve-publish pattern in the CMS domain: a
// `ContentShareList` is created in `Draft` (review), the
// audience is resolved (approve), then dispatched (publish).
// After dispatch the audience is frozen; subsequent dispatch
// is rejected. Cancellation is only valid in `Draft`.
// =============================================================================

/// Content workflow step 1: creating a content share list
/// emits [`ContentShareListCreated`] with the supplied
/// title, share window, and content count.
#[test]
fn content_workflow_create_emits_content_share_list_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let content_type = content_type_id(&g, school);
    let academic = academic_year_id(&g, school);

    // Seed a couple of content ids for the share list.
    let _c1 = new_content_aggregate(
        &g,
        school,
        actor,
        content_type,
        "math-syllabus.pdf",
        4096,
        academic,
    );
    let c2 = new_content_aggregate(
        &g,
        school,
        actor,
        content_type,
        "physics-syllabus.pdf",
        2048,
        academic,
    );

    let at = Timestamp::now();
    let list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Mid-term syllabi").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: Some("Class 10 mid-term syllabi for parents".to_owned()),
        send_type: ContentShareType::Groups,
        content_ids: vec![c2.id],
        gr_role_ids: Some(vec![g.next_uuid(), g.next_uuid()]),
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .expect("ContentShareList::new must succeed for valid window");

    let event: ContentShareListCreated =
        ContentShareListCreated::new(&list, correlation, clock.now());

    assert_eq!(
        <ContentShareListCreated as DomainEvent>::EVENT_TYPE,
        "cms.content_share_list.created"
    );
    assert_eq!(event.school_id, school);
    assert_eq!(event.title.as_str(), "Mid-term syllabi");
    assert_eq!(event.content_count, 1);
    assert_eq!(event.share_list_id, list.id);
}

/// Content workflow step 2 (review): the audience for a
/// `Groups`-scoped share list resolves to the supplied role
/// ids; the audience is frozen at dispatch time.
#[test]
fn content_workflow_resolve_audience_extracts_role_ids() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let academic = academic_year_id(&g, school);
    let role1 = g.next_uuid();
    let role2 = g.next_uuid();

    let at = Timestamp::now();
    let list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Parent updates").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: None,
        send_type: ContentShareType::Groups,
        content_ids: vec![],
        gr_role_ids: Some(vec![role1, role2]),
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic,
        created_by: actor,
        created_at: at,
        correlation_id: g.next_correlation_id(),
    })
    .unwrap();

    let resolved = ContentShareListService::resolve_audience(&list);
    assert_eq!(resolved.roles, vec![role1, role2]);
    assert!(resolved.users.is_empty());
    assert!(resolved.class_section.is_none());

    // The list is within its share window on day 1 and on day 15.
    assert!(list.is_within_share_window(date(2026, 6, 1)));
    assert!(list.is_within_share_window(date(2026, 6, 15)));
    // And no longer within the window after valid_upto.
    assert!(!list.is_within_share_window(date(2026, 7, 1)));
}

/// Content workflow step 3 (publish): dispatching the share
/// list transitions the aggregate to `Dispatched` and emits
/// [`ContentShareListDispatched`] with the recipient count.
#[test]
fn content_workflow_dispatch_emits_content_share_list_dispatched() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Holiday notice").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: None,
        send_type: ContentShareType::Individual,
        content_ids: vec![],
        gr_role_ids: None,
        ind_user_ids: Some((0..3).map(|_| g.next_uuid()).collect()),
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic_year_id(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    })
    .unwrap();
    assert!(matches!(list.status, ContentShareListStatus::Draft));

    let recipient_count = list
        .ind_user_ids
        .as_ref()
        .map(|v| u32::try_from(v.len()).unwrap_or(u32::MAX))
        .unwrap_or(0);

    list.dispatch(actor, clock.now(), g.next_event_id())
        .unwrap();
    let event: ContentShareListDispatched =
        ContentShareListDispatched::new(&list, recipient_count, correlation, clock.now());

    assert_eq!(
        <ContentShareListDispatched as DomainEvent>::EVENT_TYPE,
        "cms.content_share_list.dispatched"
    );
    assert!(matches!(list.status, ContentShareListStatus::Dispatched));
    assert_eq!(event.recipient_count, 3);
    assert_eq!(event.share_list_id, list.id);
}

/// Content workflow failure path: dispatching an already-
/// dispatched share list must be rejected with
/// `CmsError::ContentShareListNotDispatchable`. Per the spec,
/// `DispatchContentShareList` is **not** idempotent — re-dispatch
/// produces duplicate notifications, so the aggregate rejects it.
#[test]
fn content_workflow_double_dispatch_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Re-dispatch attempt").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: None,
        send_type: ContentShareType::Groups,
        content_ids: vec![],
        gr_role_ids: Some(vec![g.next_uuid()]),
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic_year_id(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    })
    .unwrap();

    list.dispatch(actor, clock.now(), g.next_event_id())
        .unwrap();
    let err = list
        .dispatch(actor, clock.now(), g.next_event_id())
        .expect_err("re-dispatch must be rejected");
    assert!(
        matches!(err, CmsError::ContentShareListNotDispatchable(_)),
        "got {err:?}"
    );
}

/// Content workflow failure path: cancelling a dispatched
/// share list is rejected with
/// `CmsError::ContentShareListNotCancellable` (only Draft
/// lists can be cancelled).
#[test]
fn content_workflow_cancel_after_dispatch_returns_conflict() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Cancel-after-dispatch").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: None,
        send_type: ContentShareType::Public,
        content_ids: vec![],
        gr_role_ids: None,
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic_year_id(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    })
    .unwrap();

    list.dispatch(actor, clock.now(), g.next_event_id())
        .unwrap();
    let err = list
        .cancel(actor, clock.now(), g.next_event_id())
        .expect_err("cancel-after-dispatch must be rejected");
    assert!(
        matches!(err, CmsError::ContentShareListNotCancellable(_)),
        "got {err:?}"
    );
}

/// Content workflow happy path: a Draft share list can be
/// cancelled before dispatch (per spec step 5). Cancelling
/// emits [`ContentShareListCancelled`].
#[test]
fn content_workflow_cancel_draft_emits_content_share_list_cancelled() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut list = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Cancel me").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 1)),
        valid_upto: ValidUntil::new(date(2026, 6, 30)),
        description: None,
        send_type: ContentShareType::Groups,
        content_ids: vec![],
        gr_role_ids: Some(vec![g.next_uuid()]),
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic_year_id(&g, school),
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    })
    .unwrap();
    assert!(matches!(list.status, ContentShareListStatus::Draft));

    list.cancel(actor, clock.now(), g.next_event_id()).unwrap();
    let event: ContentShareListCancelled = ContentShareListCancelled::new(
        &list,
        Some("Superseded by an updated notice".to_owned()),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <ContentShareListCancelled as DomainEvent>::EVENT_TYPE,
        "cms.content_share_list.cancelled"
    );
    assert!(matches!(list.status, ContentShareListStatus::Cancelled));
    assert_eq!(
        event.reason.as_deref(),
        Some("Superseded by an updated notice")
    );
}

/// Content workflow failure path: per spec invariant 3, the
/// share list window is `valid_upto >= share_date`. An
/// inverted window must be rejected with
/// `CmsError::ContentShareListInvalidWindow`.
#[test]
fn content_workflow_inverted_share_window_returns_validation_error() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let academic = academic_year_id(&g, school);

    let res = ContentShareList::new(NewContentShareList {
        id: share_list_id(&g, school),
        title: ContentShareListTitle::new("Bad window").unwrap(),
        share_date: ShareDate::new(date(2026, 6, 30)),
        valid_upto: ValidUntil::new(date(2026, 6, 1)),
        description: None,
        send_type: ContentShareType::Public,
        content_ids: vec![],
        gr_role_ids: None,
        ind_user_ids: None,
        class_id: None,
        section_ids: None,
        url: None,
        academic_id: academic,
        created_by: actor,
        created_at: Timestamp::now(),
        correlation_id: g.next_correlation_id(),
    });
    let err = res.expect_err("inverted share window must fail");
    assert!(
        matches!(err, CmsError::ContentShareListInvalidWindow),
        "got {err:?}"
    );
}
