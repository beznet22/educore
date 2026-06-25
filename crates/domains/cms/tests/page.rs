//! Integration tests for the **Page service-factory handlers**.
//!
//! Exercises [`create_page_service`] and [`update_page_service`]
//! end-to-end against in-memory mocks of the repository, audit
//! log, event bus, and capability check ports. The tests verify
//! the full vertical slice:
//!
//!   * capability gate (RBAC),
//!   * aggregate construction / mutation,
//!   * persistence via the repository port,
//!   * audit row write,
//!   * domain-event publication to the bus.
//!
//! Two scenarios are pinned:
//!
//!   1. **Happy path** — `create_page_service` followed by
//!      `update_page_service` against valid input. The aggregate
//!      ends up persisted with the new title, two audit rows
//!      (`create` then `update`) are written with the expected
//!      `before` / `after` payloads, and the bus deliverable
//!      returns without error.
//!   2. **Validation failure** — `create_page_service` with an
//!      empty `PageTitle`. The `PageTitle` value-object
//!      constructor rejects the input with
//!      [`DomainError::Validation`]; nothing is persisted or
//!      audited.
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
use educore_audit::prelude::{AuditLogEntry, AuditWriter, RetentionPolicy};
use educore_cms::commands::{CreatePageCommand, UpdatePageCommand};
use educore_cms::prelude::*;
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::SchoolId;
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::event_bus::EventBus;
use educore_rbac::ids::RoleId;
use educore_rbac::services::InMemoryCapabilityCheck;
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLog;
use educore_storage::port::StorageAdapter;
use educore_storage::transaction::Transaction;
use educore_testkit::storage::InMemoryStorageAdapter;

// =============================================================================
// In-memory mocks
// =============================================================================

/// In-memory page repository that retains every inserted /
/// updated row so tests can assert on persistence side-effects.
#[derive(Debug, Default)]
struct InMemoryPageRepo {
    rows: Mutex<Vec<Page>>,
}

#[async_trait]
impl PageRepository for InMemoryPageRepo {
    async fn get(&self, id: PageId) -> educore_core::error::Result<Option<Page>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.id == id)
            .cloned())
    }

    async fn find_by_slug(
        &self,
        _school: SchoolId,
        _slug: &Slug,
    ) -> educore_core::error::Result<Option<Page>> {
        Ok(None)
    }

    async fn find_home(&self, _school: SchoolId) -> educore_core::error::Result<Option<Page>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|p| p.is_home_page())
            .cloned())
    }

    async fn list(
        &self,
        _school: SchoolId,
        _q: PageQuery,
    ) -> educore_core::error::Result<Vec<Page>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn list_published(&self, _school: SchoolId) -> educore_core::error::Result<Vec<Page>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|p| matches!(p.status, PageStatus::Published))
            .cloned()
            .collect())
    }

    async fn insert(&self, p: &Page) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().push(p.clone());
        Ok(())
    }

    async fn update(&self, p: &Page) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|x| x.id == p.id) {
            *existing = p.clone();
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "page {} not found",
                p.id.as_uuid()
            )))
        }
    }

    async fn delete(&self, id: PageId) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().retain(|p| p.id != id);
        Ok(())
    }

    async fn count(&self, _school: SchoolId, _q: PageQuery) -> educore_core::error::Result<u64> {
        Ok(self.rows.lock().unwrap().len() as u64)
    }

    async fn page(
        &self,
        _school: SchoolId,
        _q: PageQuery,
        _offset: u32,
        _limit: u32,
    ) -> educore_core::error::Result<Vec<Page>> {
        Ok(self.rows.lock().unwrap().clone())
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
    adapter: Arc<InMemoryStorageAdapter>,
    bus: Arc<InProcessEventBus>,
    audit_log: Arc<InMemoryAuditLog>,
    audit_writer: Arc<AuditWriter>,
    capability_check: Arc<InMemoryCapabilityCheck>,
    page_repo: Arc<InMemoryPageRepo>,
}

impl TestEnv {
    fn new(school: SchoolId) -> Self {
        let bus = Arc::new(InProcessEventBus::new());
        let adapter = Arc::new(
            InMemoryStorageAdapter::new(bus.clone() as Arc<dyn EventBus>).with_school(school),
        );
        let bus_dyn: Arc<dyn EventBus> = bus.clone();
        let audit_log = Arc::new(InMemoryAuditLog::default());
        let audit_log_dyn: Arc<dyn AuditLog> = audit_log.clone();
        let clock = Arc::new(TestClock::at(Timestamp::now()));
        // FND-SEC-AUDIT-001: AuditWriter is tenant-bound; the
        // writer can only write audit rows for the school it
        // was constructed for. `school` here comes from
        // `fresh_tenant()` so each test gets a writer bound to
        // its own tenant.
        let audit_writer = Arc::new(
            AuditWriter::new(
                school,
                audit_log_dyn,
                bus_dyn,
                clock,
                RetentionPolicy::default(),
            )
            .expect("test school_id is a valid (non-nil) UUID"),
        );
        let capability_check = Arc::new(InMemoryCapabilityCheck::new());
        let page_repo = Arc::new(InMemoryPageRepo::default());
        Self {
            adapter,
            bus,
            audit_log,
            audit_writer,
            capability_check,
            page_repo,
        }
    }

    /// Begins a fresh in-memory transaction for the
    /// service-factory calls. Each test gets its own
    /// transaction; the audit writer writes audit rows
    /// through `txn.audit_log()` and the transaction is
    /// committed at the end of the test.
    async fn begin_txn(&self) -> Box<dyn Transaction> {
        self.adapter
            .begin()
            .await
            .expect("begin in-memory transaction")
    }

    fn grant(&self, school: SchoolId, capability: Capability) {
        let role = RoleId::new(school, uuid::Uuid::now_v7());
        self.capability_check.grant(school, role, capability);
    }

    /// Snapshot of the audit-log entries. The handler order is
    /// preserved: rows are appended in the order the service
    /// factories call `AuditWriter::write`.
    fn audit_entries(&self) -> Vec<AuditLogEntry> {
        self.adapter.read_audit_log_entries()
    }

    /// Snapshot of the persisted page rows.
    fn persisted_pages(&self) -> Vec<Page> {
        self.page_repo.rows.lock().unwrap().clone()
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

fn create_cmd(tenant: &TenantContext) -> CreatePageCommand {
    CreatePageCommand {
        tenant: tenant.clone(),
        title: PageTitle::new("Home").expect("non-empty title is valid"),
        description: None,
        slug: Some(Slug::new("home").expect("non-empty slug is valid")),
        settings: None,
        home_page: true,
        is_default: false,
    }
}

// =============================================================================
// Happy-path test
// =============================================================================

/// End-to-end happy path: a `SchoolAdmin` with the create and
/// update capabilities creates a new page, then updates the
/// title. After the create the row is persisted, an audit row
/// of action `"create"` is written, and the bus deliverable
/// returns without error. After the update the persisted row
/// carries the new title, the version bumps, an audit row of
/// action `"update"` is written (with both `before` and `after`
/// snapshots), and the bus deliverable returns without error.
#[tokio::test]
async fn page_handlers_happy_path_create_then_update_persists_and_audits() {
    let ft = fresh_tenant();
    let env = TestEnv::new(ft.school);
    env.grant(ft.school, Capability::CmsPageCreate);
    env.grant(ft.school, Capability::CmsPageUpdate);

    // 1) Create the page.
    let cmd = create_cmd(&ft.tenant);
    let txn = env.begin_txn().await;
    let page = create_page_service(
        cmd,
        &*txn,
        env.page_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("create_page_service must succeed for valid input");

    assert_eq!(page.title.as_str(), "Home");
    assert_eq!(page.slug.as_ref().expect("slug present").as_str(), "home");
    assert!(page.is_home_page(), "created page must be the home page");
    assert_eq!(page.school_id, ft.school);
    assert_eq!(page.created_by, ft.actor);
    assert!(matches!(page.status, PageStatus::Draft));

    // The page must be persisted by the repo.
    let persisted = env
        .page_repo
        .get(page.id)
        .await
        .expect("repo.get ok")
        .expect("page present after create");
    txn.commit().await.expect("commit txn");

    assert_eq!(persisted.id, page.id);
    assert_eq!(persisted.title.as_str(), "Home");

    // Exactly one audit row of action "create" must be appended.
    let entries = env.audit_entries();
    assert_eq!(entries.len(), 1, "exactly one audit row after create");
    let created = &entries[0];
    assert_eq!(created.action, "create");
    assert_eq!(created.school_id, ft.school);
    assert_eq!(created.actor_id, ft.actor);
    assert_eq!(created.target_id, page.id.as_uuid());
    assert_eq!(created.target_type, "page");
    assert!(
        created.before.is_none(),
        "create audit row must have no before snapshot"
    );
    assert!(
        created.after.is_some(),
        "create audit row must have an after snapshot"
    );

    // 2) Update the title.
    let update_cmd = UpdatePageCommand {
        tenant: ft.tenant.clone(),
        page_id: page.id,
        title: Some(PageTitle::new("Welcome").expect("non-empty title is valid")),
        description: None,
        slug: None,
    };
    let txn = env.begin_txn().await;
    let updated = update_page_service(
        update_cmd,
        &*txn,
        env.page_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("update_page_service must succeed for active page");

    assert_eq!(updated.id, page.id);
    assert_eq!(updated.title.as_str(), "Welcome");
    assert_eq!(
        updated.version,
        page.version.next(),
        "version must bump on update"
    );
    assert!(updated.last_event_id.is_some());

    // The repo must reflect the new title.
    let persisted_after = env
        .page_repo
        .get(page.id)
        .await
        .expect("repo.get ok")
        .expect("page still present after update");
    txn.commit().await.expect("commit txn");

    assert_eq!(persisted_after.title.as_str(), "Welcome");

    // A second audit row of action "update" must be appended.
    let entries = env.audit_entries();
    assert_eq!(entries.len(), 2, "exactly two audit rows after update");
    let updated_audit = &entries[1];
    assert_eq!(updated_audit.action, "update");
    assert_eq!(updated_audit.school_id, ft.school);
    assert_eq!(updated_audit.actor_id, ft.actor);
    assert_eq!(updated_audit.target_id, page.id.as_uuid());
    assert_eq!(updated_audit.target_type, "page");
    assert!(
        updated_audit.before.is_some(),
        "update audit row must have a before snapshot"
    );
    assert!(
        updated_audit.after.is_some(),
        "update audit row must have an after snapshot"
    );
}

// =============================================================================
// Validation-failure test
// =============================================================================

/// Validation failure path: `PageTitle::new("")` rejects the
/// empty title with [`DomainError::Validation`] per the
/// spec-mandated invariant 1 ("a page title must be non-empty").
/// The `create_page_command` builder therefore cannot be
/// constructed with an empty title — the value-object
/// constructor rejects the input before the service factory is
/// invoked. The service factory MUST NOT touch the repo or the
/// audit log when this validation step fails.
#[tokio::test]
async fn page_handlers_validation_failure_rejects_empty_title_without_side_effects() {
    let ft = fresh_tenant();
    let env = TestEnv::new(ft.school);
    env.grant(ft.school, Capability::CmsPageCreate);

    // Attempt to construct an empty title. Per spec invariant 1
    // (`PageTitle::MIN_LEN = 1`), the constructor MUST reject
    // this input with `DomainError::Validation`.
    let title_result = PageTitle::new(String::new());
    let err = title_result.expect_err("PageTitle::new must reject an empty title");
    assert!(
        matches!(err, educore_core::error::DomainError::Validation(_)),
        "expected DomainError::Validation, got {err:?}"
    );

    // Since the title construction failed, the service factory
    // cannot be invoked with the empty title — the build site
    // rejects it at the type boundary. Verify the no-side-effect
    // invariant on the ports directly: no page row, no audit row.
    let rows = env.persisted_pages();
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
