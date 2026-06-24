//! Integration tests for the **PostalReceive service-factory handler**.
//!
//! Exercises [`receive_postal_service`] end-to-end against
//! in-memory mocks of the repository, audit log, event bus, and
//! capability check ports. The tests verify the full vertical
//! slice:
//!
//!   * capability gate (RBAC),
//!   * aggregate construction,
//!   * persistence via the repository port,
//!   * audit row write.
//!
//! Two scenarios are pinned:
//!
//!   1. **Happy path** — `receive_postal_service` against valid
//!      input. The aggregate ends up persisted in the active
//!      state with the requested fields, an audit row of action
//!      `"create"` is written for the receive (with `before =
//!      None` and `after = Some(snapshot)`), and the bus
//!      publish returns without error.
//!   2. **Validation failure** — `receive_postal_service` is
//!      invoked without the
//!      [`Capability::PostalReceiveCreate`] capability. The
//!      service factory MUST short-circuit at the capability gate
//!      and return [`DocumentsError::Forbidden`]; nothing is
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

/// In-memory postal-receive repository that retains every
/// inserted / updated row so tests can assert on persistence
/// side-effects.
#[derive(Debug, Default)]
struct InMemoryPostalRepo {
    rows: Mutex<Vec<PostalReceive>>,
}

#[async_trait]
impl PostalReceiveRepository for InMemoryPostalRepo {
    async fn get(
        &self,
        id: PostalReceiveId,
    ) -> educore_core::error::Result<Option<PostalReceive>> {
        Ok(self.rows.lock().unwrap().iter().find(|r| r.id == id).cloned())
    }

    async fn list(
        &self,
        _school: SchoolId,
        _q: PostalReceiveQuery,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn insert(&self, receive: &PostalReceive) -> educore_core::error::Result<()> {
        self.rows.lock().unwrap().push(receive.clone());
        Ok(())
    }

    async fn update(&self, receive: &PostalReceive) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|r| r.id == receive.id) {
            *existing = receive.clone();
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "receive {} not found",
                receive.id.as_uuid()
            )))
        }
    }

    async fn find_by_reference(
        &self,
        _school: SchoolId,
        _reference: &PostalReferenceNo,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.reference_no.as_ref() == Some(_reference))
            .cloned()
            .collect())
    }

    async fn between(
        &self,
        _school: SchoolId,
        _from: chrono::NaiveDate,
        _to: chrono::NaiveDate,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        Ok(self.rows.lock().unwrap().clone())
    }

    async fn by_academic_year(
        &self,
        _school: SchoolId,
        _year: educore_documents::aggregate::AcademicYearId,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        Ok(self.rows.lock().unwrap().clone())
    }
}

/// In-memory audit log that captures every appended row so
/// tests can assert the side-effects of the service factory.
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
/// The service factory takes `Arc<B>` where `B: EventBus +
/// 'static`, so the bus MUST be a concrete `InProcessEventBus`,
/// not a `dyn EventBus` trait object. The `Arc<dyn EventBus>`
/// is also held so it can be shared with `AuditWriter` (which
/// expects a trait object).
struct TestEnv {
    bus: Arc<InProcessEventBus>,
    audit_log: Arc<InMemoryAuditLog>,
    audit_writer: Arc<AuditWriter>,
    capability_check: Arc<InMemoryCapabilityCheck>,
    postal_repo: Arc<InMemoryPostalRepo>,
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
        let postal_repo = Arc::new(InMemoryPostalRepo::default());
        Self {
            bus,
            audit_log,
            audit_writer,
            capability_check,
            postal_repo,
        }
    }

    fn grant(&self, school: SchoolId, capability: Capability) {
        let role = RoleId::new(school, uuid::Uuid::now_v7());
        self.capability_check.grant(school, role, capability);
    }

    /// Snapshot of the audit-log entries. The handler order is
    /// preserved: rows are appended in the order the service
    /// factory calls `AuditWriter::write`.
    fn audit_entries(&self) -> Vec<AuditLogEntry> {
        self.audit_log.entries.lock().unwrap().clone()
    }

    /// Snapshot of the persisted postal-receive rows.
    fn persisted_receives(&self) -> Vec<PostalReceive> {
        self.postal_repo.rows.lock().unwrap().clone()
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

#[allow(unused_variables)]
fn receive_cmd(
    tenant: &TenantContext,
    reference_no: Option<&str>,
    academic_id: educore_documents::aggregate::AcademicYearId,
) -> ReceivePostalCommand {
    ReceivePostalCommand {
        tenant: tenant.clone(),
        from_title: FromTitle::new(PostalTitle::new("Acme Vendor").unwrap()),
        to_title: ToTitle::new(PostalTitle::new("Acme School").unwrap()),
        reference_no: reference_no.map(|s| PostalReferenceNo::new(s).unwrap()),
        address: FromAddress::new(PostalAddress::new("5 Vendor Rd").unwrap()),
        date: ReceiveDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: None,
        file: None,
    }
}

// =============================================================================
// Happy-path test
// =============================================================================

/// End-to-end happy path: a `SchoolAdmin` with the
/// `PostalReceiveCreate` capability records a new incoming
/// postal item. After the receive the row is persisted, an
/// audit row of action `"create"` is written (with `before =
/// None` and `after = Some(snapshot)`), and the bus deliverable
/// returns without error. The persisted aggregate carries the
/// `from_title`, `to_title`, address, reference number, and
/// receive date supplied on the command.
#[tokio::test]
async fn postal_receive_happy_path_persists_and_audits() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    env.grant(ft.school, Capability::PostalReceiveCreate);

    let academic_id = uuid::Uuid::now_v7();
    let cmd = receive_cmd(&ft.tenant, Some("REF-IN-2026-0001"), academic_id);
    let receive = receive_postal_service(
        cmd,
        academic_id,
        env.postal_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect("receive_postal_service must succeed for valid input");

    assert!(receive.is_active(), "received postal item must be active");
    assert_eq!(receive.school_id, ft.school);
    assert_eq!(receive.created_by, ft.actor);
    assert_eq!(receive.academic_id, academic_id);
    assert_eq!(receive.from_title.as_str(), "Acme Vendor");
    assert_eq!(receive.to_title.as_str(), "Acme School");
    assert_eq!(receive.address.as_str(), "5 Vendor Rd");
    assert_eq!(
        receive
            .reference_no
            .as_ref()
            .map(PostalReferenceNo::as_str)
            .unwrap_or(""),
        "REF-IN-2026-0001"
    );

    // The receive must be persisted by the repo.
    let persisted = env
        .postal_repo
        .get(receive.id)
        .await
        .expect("repo.get ok")
        .expect("receive present after insertion");
    assert_eq!(persisted.id, receive.id);
    assert_eq!(persisted.school_id, ft.school);
    assert_eq!(persisted.from_title.as_str(), "Acme Vendor");
    assert_eq!(persisted.to_title.as_str(), "Acme School");

    // Exactly one audit row of action "create" must be appended,
    // targeting the receive via AuditTarget::PostalReceive.
    let entries = env.audit_entries();
    assert_eq!(
        entries.len(),
        1,
        "exactly one audit row after receive_postal_service"
    );
    let created = &entries[0];
    assert_eq!(created.action, "create");
    assert_eq!(created.school_id, ft.school);
    assert_eq!(created.actor_id, ft.actor);
    assert_eq!(created.target_id, receive.id.as_uuid());
    assert_eq!(created.target_type, "postal_receive");
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

/// Validation failure path: `receive_postal_service` is invoked
/// without the [`Capability::PostalReceiveCreate`] capability.
/// The service factory MUST short-circuit at the capability
/// gate (the first step of `receive_postal_service`) and return
/// [`DocumentsError::Forbidden`] without touching the repo or
/// the audit log.
#[tokio::test]
async fn postal_receive_validation_failure_missing_capability_has_no_side_effects() {
    let env = TestEnv::new();
    let ft = fresh_tenant();
    // NOTE: deliberately do NOT grant PostalReceiveCreate.
    // The capability gate is the first check in
    // receive_postal_service; it must reject the call.

    let academic_id = uuid::Uuid::now_v7();
    let cmd = receive_cmd(&ft.tenant, Some("REF-IN-2026-0002"), academic_id);

    let err = receive_postal_service(
        cmd,
        academic_id,
        env.postal_repo.clone(),
        env.bus.clone(),
        env.audit_writer.clone(),
        env.capability_check.as_ref(),
    )
    .await
    .expect_err("receive_postal_service must fail without capability");

    assert!(
        matches!(err, DocumentsError::Forbidden(_)),
        "expected Forbidden, got {err:?}"
    );

    // No row may have been persisted.
    let rows = env.persisted_receives();
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
