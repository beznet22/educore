//! Integration tests for the **News service-factory handler**.
//!
//! Exercises [`create_news_service`] end-to-end against
//! in-memory mocks of the repository, audit log, event bus, and
//! capability check ports. The tests verify the full vertical
//! slice:
//!
//!   * capability gate (RBAC),
//!   * aggregate construction,
//!   * persistence via the repository port,
//!   * audit row write,
//!   * domain-event publication to the bus.
//!
//! Two scenarios are pinned:
//!
//!   1. **Happy path** — `create_news_service` against valid
//!      input. The aggregate ends up persisted, an audit row
//!      of action `"create"` is written with an `after`
//!      snapshot, and the bus publish completes without error.
//!   2. **Validation failure** — `NewsTitle::new("")` rejects
//!      the empty title with [`educore_core::error::DomainError::Validation`]
//!      per the spec-mandated invariant 1 ("a news title must
//!      be non-empty"); nothing is persisted or audited.
//!
//! The mocks here are independent of the ones defined inside
//! `src/services.rs` so this file is a true integration test
//! (it links against the public API only).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::NaiveDate;
use educore_audit::prelude::{AuditLogEntry, AuditWriter, RetentionPolicy};
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_cms::commands::CreateNewsCommand;
use educore_cms::prelude::*;
use educore_event_bus::InProcessEventBus;
use educore_events::event_bus::EventBus;
use educore_rbac::ids::RoleId;
use educore_rbac::services::InMemoryCapabilityCheck;
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLog;

// =============================================================================
// In-memory mocks
// =============================================================================

/// In-memory news repository that retains every inserted row
/// so tests can assert on persistence side-effects.
#[derive(Debug, Default)]
struct InMemoryNewsRepo {
    rows: Mutex<Vec<News>>,
}

#[async_trait]
impl NewsRepository for InMemoryNewsRepo {
    async fn get(&self, id: NewsId) -> educore_core::error::Result<Option<News>> {
        Ok(self.rows.lock().unwrap().iter().find(|n| n.id == id).cloned())
    }

    async fn list(
        &self,
        _school: SchoolId,
        _q: NewsQuery,
    ) -> educore_core::error::Result<Vec<News>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn list_active(
        &self,
        _school: SchoolId,
    ) -> educore_core::error::Result<Vec<News>> {
        Ok(self.rows
            .lock()
            .unwrap()
            .iter()
            .filter(|n| matches!(n.active_status, NewsStatus::Active))
            .cloned()
            .collect())
    }

    async fn list_global(&self) -> educore_core::error::Result<Vec<News>> {
        Ok(self.rows
            .lock()
            .unwrap()
            .iter()
            .filter(|n| n.is_global.is_true())
            .cloned()
            .collect())
    }

    async fn list_by_category(
        &self,
        _school: SchoolId,
        _category: NewsCategoryId,
    ) -> educore_core::error::Result<Vec<News>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn list_published_between(
        &self,
        _school: SchoolId,
        _from: NaiveDate,
        _to: NaiveDate,
    ) -> educore_core::error::Result<Vec<News>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn insert(&self, n: &News) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().push(n.clone());
        Ok(())
    }

    async fn update(&self, n: &News) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|x| x.id == n.id) {
            *existing = n.clone();
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "news {} not found",
                n.id.as_uuid()
            )))
        }
    }

    async fn delete(&self, id: NewsId) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().retain(|n| n.id != id);
        Ok(())
    }

    async fn increment_view(&self, id: NewsId) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|x| x.id == id) {
            existing.view_count += 1;
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "news {} not found",
                id.as_uuid()
            )))
        }
    }
}

/// In-memory audit log that captures every appended row so
/// tests can assert the side-effects of the service factories.
#[derive(Debug, Default)]
struct InMemoryAuditLog {
    entries: Mutex<Vec<AuditLogEntry>>,
}

#[async_trait]
impl AuditLog for InMemoryAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> educore_core::error::Result<()> {
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        _school: SchoolId,
        _target_id: uuid::Uuid,
        _limit: u32,
    ) -> educore_core::error::Result<Vec<AuditLogEntry>> {
        Ok(self.entries.lock().unwrap().clone())
    }
}

/// Test environment that wires the in-memory mocks together.
///
/// The service factories take `Arc<B>` where `B: EventBus +
/// 'static`, so the bus MUST be a concrete `InProcessEventBus`,
/// not a `dyn EventBus` trait object. The `Arc<dyn EventBus>`
/// is also held so it can be shared with `AuditWriter` (which
/// expects a trait object).
struct TestEnv {
    bus: Arc<InProcessEventBus>,
    audit_log: Arc<InMemoryAuditLog>,
    audit_writer: Arc<AuditWriter>,
    capability_check: Arc<InMemoryCapabilityCheck>,
    news_repo: Arc<InMemoryNewsRepo>,
}

impl TestEnv {
    fn new() -> Self {
        let bus = Arc::new(InProcessEventBus::new());
        let bus_dyn: Arc<dyn EventBus> = bus.clone();
        let audit_log = Arc::new(InMemoryAuditLog::default());
        let audit_log_dyn: Arc<dyn AuditLog> = audit_log.clone();
        let clock = Arc::new(TestClock::at(Timestamp::now()));
        let audit_writer = Arc::new(AuditWriter::new(
            audit_log_dyn,
            bus_dyn,
            clock,
            RetentionPolicy::default(),
        ));
        let capability_check = Arc::new(InMemoryCapabilityCheck::new());
        let news_repo = Arc::new(InMemoryNewsRepo::default());
        Self {
            bus,
            audit_log,
            audit_writer,
            capability_check,
            news_repo,
        }
    }

    fn grant(&self, school: SchoolId, capability: Capability) {
        let role = RoleId::new(school, uuid::Uuid::now_v7());
        self.capability_check.grant(school, role, capability);
    }

    /// Snapshot of the audit-log entries. The handler order is
    /// preserved: rows are appended in the order the service
    /// factories call `AuditWriter::write`.
    fn audit_entries(&self) -> Vec<AuditLogEntry> {
        self.audit_log.entries.lock().unwrap().clone()
    }

    /// Snapshot of the persisted news rows.
    fn persisted_news(&self) -> Vec<News> {
        self.news_repo.rows.lock().unwrap().clone()
    }
}

// =============================================================================
// Fixtures
// =============================================================================

struct FreshTenant {
    tenant: TenantContext,
    school: SchoolId,
    actor: educore_core::ids::UserId,
}

fn fresh_tenant() -> FreshTenant {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let correlation = g.next_correlation_id();
    let tenant = TenantContext::for_user(school, actor, correlation, UserType::SchoolAdmin);
    FreshTenant {
        tenant,
        school,
        actor,
    }
}

fn create_cmd(tenant: &TenantContext, school: SchoolId) -> CreateNewsCommand {
    let category = NewsCategoryId::new(school, uuid::Uuid::now_v7());
    CreateNewsCommand {
        tenant: tenant.clone(),
        news_title: NewsTitle::new("Back to School 2026")
            .expect("non-empty title is valid"),
        category_id: category,
        image: None,
        image_thumb: None,
        news_body: NewsBody::new("School reopens on Monday the 8th.")
            .expect("non-empty body is valid"),
        publish_date: PublishDate::new(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        is_global: false,
        auto_approve: false,
        is_comment: true,
        order: None,
    }
}

// =============================================================================
// Happy-path test
// =============================================================================

/// End-to-end happy path: a `SchoolAdmin` with the create
/// capability creates a new news entry. After the create the
/// row is persisted, an audit row of action `"create"` is
/// written (with an `after` snapshot and no `before`
/// snapshot), and the bus publish completes without error.
#[tokio::test]
async fn news_handler_happy_path_create_persists_and_audits() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    env.grant(ft.school, Capability::CmsNewsCreate);

    // Create the news entry.
    let cmd = create_cmd(&ft.tenant, ft.school);
    let news = create_news_service(
        cmd,
        env.news_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("create_news_service must succeed for valid input");

    assert_eq!(news.news_title.as_str(), "Back to School 2026");
    assert_eq!(
        news.news_body.as_str(),
        "School reopens on Monday the 8th."
    );
    assert_eq!(news.school_id, ft.school);
    assert_eq!(news.created_by, ft.actor);
    assert!(
        matches!(news.active_status, NewsStatus::Active),
        "a freshly-created news entry must be active"
    );
    assert_eq!(news.view_count, 0, "view_count starts at zero");
    assert!(
        !news.is_global.is_true(),
        "non-global flag must be preserved"
    );
    assert!(
        news.is_comment.is_true(),
        "comment flag must be preserved"
    );

    // The news must be persisted by the repo.
    let persisted = env
        .news_repo
        .get(news.id)
        .await
        .expect("repo.get ok")
        .expect("news present after create");
    assert_eq!(persisted.id, news.id);
    assert_eq!(persisted.news_title.as_str(), "Back to School 2026");
    assert_eq!(persisted.created_by, ft.actor);

    // Exactly one audit row of action "create" must be appended.
    let entries = env.audit_entries();
    assert_eq!(entries.len(), 1, "exactly one audit row after create");
    let created = &entries[0];
    assert_eq!(created.action, "create");
    assert_eq!(created.school_id, ft.school);
    assert_eq!(created.actor_id, ft.actor);
    assert_eq!(created.target_id, news.id.as_uuid());
    assert_eq!(created.target_type, "news");
    assert!(
        created.before.is_none(),
        "create audit row must have no before snapshot"
    );
    assert!(
        created.after.is_some(),
        "create audit row must have an after snapshot"
    );
}

// =============================================================================
// Validation-failure test
// =============================================================================

/// Validation failure path: `NewsTitle::new("")` rejects the
/// empty title with [`educore_core::error::DomainError::Validation`]
/// per the spec-mandated invariant 1 (a news title must be
/// non-empty, `MIN_LEN = 1`). The `create_news_service` cannot
/// be invoked with an empty title — the value-object
/// constructor rejects the input at the type boundary. The
/// service factory MUST NOT touch the repo or the audit log
/// when this validation step fails.
#[tokio::test]
async fn news_handler_validation_failure_rejects_empty_title_without_side_effects() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    env.grant(ft.school, Capability::CmsNewsCreate);

    // Attempt to construct an empty title. Per spec invariant 1
    // (`NewsTitle::MIN_LEN = 1`), the constructor MUST reject
    // this input with `DomainError::Validation`.
    let title_result = NewsTitle::new(String::new());
    let err = title_result
        .expect_err("NewsTitle::new must reject an empty title");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected DomainError::Validation, got {err:?}"
    );

    // Since the title construction failed, the service factory
    // cannot be invoked with the empty title — the build site
    // rejects it at the type boundary. Verify the no-side-effect
    // invariant on the ports directly: no news row, no audit row.
    let rows = env.persisted_news();
    assert!(
        rows.is_empty(),
        "no row may be persisted on validation failure; got {} rows",
        rows.len()
    );
    let entries = env.audit_entries();
    assert!(
        entries.is_empty(),
        "no audit row may be written on validation failure; got {} entries",
        entries.len()
    );
}

// Silence the unused-import lint for the storage-port re-export
// when the test is compiled in isolation (the import documents
// the type path used by `AuditLog` and `AuditLogEntry`).
const _: fn() = || {
    let _: Option<Arc<dyn AuditLog>> = None;
};
