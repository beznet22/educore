//! Integration tests for the **FormDownload service-factory handlers**.
//!
//! Exercises [`upload_form_service`] and [`update_form_service`]
//! end-to-end against in-memory mocks of the repository, audit
//! log, event bus, and capability check ports. The tests verify
//! the full vertical slice:
//!
//!   * capability gate (RBAC),
//!   * aggregate construction / mutation,
//!   * persistence via the repository port,
//!   * audit row write.
//!
//! Two scenarios are pinned:
//!
//!   1. **Happy path** — `upload_form_service` followed by
//!      `update_form_service` against valid input. The aggregate
//!      ends up persisted in the active state with the
//!      requested fields, an audit row of action `create` is
//!      written for the upload, and an audit row of action
//!      `update` is written for the update (with non-empty
//!      `before` and `after` payloads).
//!   2. **Validation failure** — `upload_form_service` with
//!      both `link = None` and `file = None`. The aggregate
//!      constructor rejects the input with
//!      [`DocumentsError::FormHasNoContent`]; nothing is
//!      persisted or audited.
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
use educore_audit::prelude::{AuditWriter, RetentionPolicy};
use educore_core::clock::{IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::{SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_documents::prelude::*;
use educore_event_bus::InProcessEventBus;
use educore_events::event_bus::EventBus;
use educore_rbac::ids::RoleId;
use educore_rbac::services::InMemoryCapabilityCheck;
use educore_rbac::value_objects::Capability;
use educore_storage::audit::{AuditLog, AuditLogEntry};

// =============================================================================
// In-memory mocks
// =============================================================================

/// In-memory form repository that retains every inserted /
/// updated row so tests can assert on persistence side-effects.
#[derive(Debug, Default)]
struct InMemoryFormRepo {
    rows: Mutex<Vec<FormDownload>>,
}

#[async_trait]
impl FormDownloadRepository for InMemoryFormRepo {
    async fn get(&self, id: FormDownloadId) -> educore_core::error::Result<Option<FormDownload>> {
        Ok(self.rows.lock().unwrap().iter().find(|f| f.id == id).cloned())
    }

    async fn list(
        &self,
        _school: SchoolId,
        _q: FormDownloadQuery,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn list_public(
        &self,
        _school: SchoolId,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn insert(&self, form: &FormDownload) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().push(form.clone());
        Ok(())
    }

    async fn update(&self, form: &FormDownload) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|f| f.id == form.id) {
            *existing = form.clone();
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "form {} not found",
                form.id.as_uuid()
            )))
        }
    }

    async fn by_publish_date(
        &self,
        _school: SchoolId,
        _from: chrono::NaiveDate,
        _to: chrono::NaiveDate,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn count(
        &self,
        _school: SchoolId,
        _q: FormDownloadQuery,
    ) -> educore_core::error::Result<u64> {
        Ok(self.rows.lock().unwrap().len() as u64)
    }

    async fn page(
        &self,
        _school: SchoolId,
        _q: FormDownloadQuery,
        _offset: u32,
        _limit: u32,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
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
    bus: Arc<InProcessEventBus>,
    audit_log: Arc<InMemoryAuditLog>,
    audit_writer: Arc<AuditWriter>,
    capability_check: Arc<InMemoryCapabilityCheck>,
    form_repo: Arc<InMemoryFormRepo>,
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
        let form_repo = Arc::new(InMemoryFormRepo::default());
        Self {
            bus,
            audit_log,
            audit_writer,
            capability_check,
            form_repo,
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

    /// Snapshot of the persisted form rows.
    fn persisted_forms(&self) -> Vec<FormDownload> {
        self.form_repo.rows.lock().unwrap().clone()
    }
}

// =============================================================================
// Fixtures
// =============================================================================

struct FreshTenant {
    tenant: TenantContext,
    school: SchoolId,
    actor: UserId,
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

fn upload_cmd(
    tenant: &TenantContext,
    link: Option<Url>,
    file: Option<FileReference>,
) -> UploadFormCommand {
    UploadFormCommand {
        tenant: tenant.clone(),
        title: FormTitle::new("Vertical Slice Form").unwrap(),
        short_description: Some(FormDescription::new("Slice description").unwrap()),
        publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        link,
        file,
        show_public: ShowPublic::new(true),
    }
}

// =============================================================================
// Happy-path test
// =============================================================================

/// End-to-end happy path: a `SchoolAdmin` with the upload and
/// update capabilities uploads a new form, then updates the
/// title. After the upload the row is persisted, an audit row
/// of action `"create"` is written, and the bus deliverable
/// returns without error (the in-process bus publishes
/// successfully by default). After the update the persisted
/// row carries the new title, an audit row of action
/// `"update"` is written (with both `before` and `after`
/// snapshots), and the bus deliverable returns without error.
#[tokio::test]
async fn form_handlers_happy_path_upload_then_update_persists_and_audits() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    env.grant(ft.school, Capability::FormDownloadUpload);
    env.grant(ft.school, Capability::FormDownloadUpdate);

    // 1) Upload.
    let link = Url::new("https://example.com/vertical-slice.pdf").unwrap();
    let cmd = upload_cmd(&ft.tenant, Some(link.clone()), None);
    let form = upload_form_service(
        cmd,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("upload_form_service must succeed for valid input");

    assert!(form.is_active(), "uploaded form must be active");
    assert!(form.is_public(), "uploaded form must be public");
    assert!(form.is_deliverable(), "uploaded form must be deliverable");
    assert_eq!(form.school_id, ft.school);
    assert_eq!(form.created_by, ft.actor);

    // The form must be persisted by the repo.
    let persisted = env
        .form_repo
        .get(form.id)
        .await
        .expect("repo.get ok")
        .expect("form present after upload");
    assert_eq!(persisted.id, form.id);
    assert_eq!(persisted.title.as_str(), "Vertical Slice Form");

    // Exactly one audit row of action "create" must be appended.
    let entries = env.audit_entries();
    assert_eq!(entries.len(), 1, "exactly one audit row after upload");
    let created = &entries[0];
    assert_eq!(created.action, "create");
    assert_eq!(created.school_id, ft.school);
    assert_eq!(created.actor_id, ft.actor);
    assert_eq!(created.target_id, form.id.as_uuid());
    assert!(
        created.before.is_none(),
        "upload audit row must have no before snapshot"
    );
    assert!(
        created.after.is_some(),
        "upload audit row must have an after snapshot"
    );

    // 2) Update the title.
    let update_cmd = UpdateFormCommand {
        tenant: ft.tenant.clone(),
        form_id: form.id,
        title: Some(FormTitle::new("Vertical Slice Form (Renamed)").unwrap()),
        short_description: None,
        publish_date: None,
        link: None,
        file: None,
        show_public: None,
    };
    let updated = update_form_service(
        update_cmd,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("update_form_service must succeed for active form");

    assert_eq!(updated.id, form.id);
    assert_eq!(updated.title.as_str(), "Vertical Slice Form (Renamed)");
    assert_eq!(
        updated.version,
        form.version.next(),
        "version must bump on update"
    );
    assert_eq!(updated.last_event_id.is_some(), true);

    // The repo must reflect the new title.
    let persisted_after = env
        .form_repo
        .get(form.id)
        .await
        .expect("repo.get ok")
        .expect("form still present after update");
    assert_eq!(
        persisted_after.title.as_str(),
        "Vertical Slice Form (Renamed)"
    );

    // A second audit row of action "update" must be appended.
    let entries = env.audit_entries();
    assert_eq!(entries.len(), 2, "exactly two audit rows after update");
    let updated_audit = &entries[1];
    assert_eq!(updated_audit.action, "update");
    assert_eq!(updated_audit.school_id, ft.school);
    assert_eq!(updated_audit.actor_id, ft.actor);
    assert_eq!(updated_audit.target_id, form.id.as_uuid());
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

/// Validation failure path: `upload_form_service` is invoked
/// with `link = None` and `file = None`. Per spec invariant 2,
/// the form MUST have at least one of `link` or `file` set, so
/// the aggregate constructor returns
/// [`DocumentsError::FormHasNoContent`]. The service factory
/// MUST short-circuit at this validation step and MUST NOT
/// touch the repo or the audit log.
#[tokio::test]
async fn form_handlers_validation_failure_rejects_missing_link_and_file_without_side_effects() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    env.grant(ft.school, Capability::FormDownloadUpload);

    // Both link and file are None — spec violation.
    let cmd = upload_cmd(&ft.tenant, None, None);

    let err = upload_form_service(
        cmd,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect_err("upload must fail when neither link nor file is set");

    assert!(
        matches!(err, DocumentsError::FormHasNoContent),
        "expected FormHasNoContent, got {err:?}"
    );

    // No row may have been persisted.
    let rows = env.persisted_forms();
    assert!(
        rows.is_empty(),
        "no row may be persisted on validation failure; got {} rows",
        rows.len()
    );

    // No audit row may have been written.
    let entries = env.audit_entries();
    assert!(
        entries.is_empty(),
        "no audit row may be written on validation failure; got {} entries",
        entries.len()
    );
}
