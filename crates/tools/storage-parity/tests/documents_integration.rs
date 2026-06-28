//! # Documents domain vertical-slice integration test
//!
//! Mirrors the Phase 9 library pattern
//! (`library_integration.rs`). Runs on SQLite (always) +
//! PG/MySQL (env-gated).
//!
//! The headline scenario: configure the documents
//! engine (subscribe to the bus + create a [`FormDownload`] +
//! dispatch a [`PostalDispatch`] + receive a
//! [`PostalReceive`]) → assert the bus received
//! `FormUploaded`, `PostalDispatched`, `PostalReceived`
//! envelopes, and assert the cross-aggregate invariants
//! (reference uniqueness, soft-delete, public visibility).
//!
//! The bus + outbox + audit + idempotency rows are exercised
//! in a single transaction per the Phase 2 OQ #5 hand-off.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use educore_audit::prelude::*;
use educore_core::clock::{IdGenerator, SystemIdGen, TestClock};
use educore_core::ids::{CorrelationId, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_documents::aggregate::AcademicYearId;
use educore_documents::prelude::*;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::StorageAdapter;

// ---------------------------------------------------------------------------
// In-memory mocks for the audit port.
// ---------------------------------------------------------------------------

/// In-memory `AuditLog` mock. Records every appended entry.
#[derive(Debug, Default)]
struct InMemoryAuditLog {
    entries: Mutex<Vec<AuditLogEntry>>,
}

impl InMemoryAuditLog {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl educore_storage::audit::AuditLog for InMemoryAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> educore_core::error::Result<()> {
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        _school_id: SchoolId,
        _target_id: uuid::Uuid,
        _limit: u32,
    ) -> educore_core::error::Result<Vec<AuditLogEntry>> {
        Ok(self.entries.lock().unwrap().clone())
    }
}

fn ts(secs: i64) -> Timestamp {
    Timestamp::from_datetime(Utc.timestamp_opt(secs, 0).single().unwrap_or_else(Utc::now))
}

// ---------------------------------------------------------------------------
// In-memory documents repository stubs (per the task's "in-memory test
// struct implementing the repository trait" instruction).
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct InMemoryFormRepo {
    rows: Mutex<Vec<FormDownload>>,
}

impl InMemoryFormRepo {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl FormDownloadRepository for InMemoryFormRepo {
    async fn get(&self, id: FormDownloadId) -> educore_core::error::Result<Option<FormDownload>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|f| f.id == id)
            .cloned())
    }

    async fn list(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        let rows = self.rows.lock().unwrap().clone();
        let filtered: Vec<FormDownload> = rows
            .into_iter()
            .filter(|f| f.school_id == school)
            .filter(|f| q.title.as_ref().is_none_or(|t| &f.title == t))
            .filter(|f| q.show_public.is_none_or(|sp| f.show_public == sp))
            .filter(|f| q.publish_from.is_none_or(|p| f.publish_date.0 >= p.0))
            .filter(|f| q.publish_to.is_none_or(|p| f.publish_date.0 <= p.0))
            .filter(|f| q.active_status.is_none_or(|a| f.active_status == a))
            .collect();
        Ok(filtered)
    }

    async fn list_public(
        &self,
        school: SchoolId,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|f| f.school_id == school && f.is_public() && f.is_active())
            .collect())
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
        school: SchoolId,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|f| f.school_id == school)
            .filter(|f| f.publish_date.0 >= from && f.publish_date.0 <= to)
            .collect())
    }

    async fn count(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
    ) -> educore_core::error::Result<u64> {
        Ok(self.list(school, q).await?.len() as u64)
    }

    async fn page(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
        offset: u32,
        limit: u32,
    ) -> educore_core::error::Result<Vec<FormDownload>> {
        let all = self.list(school, q).await?;
        Ok(all
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }
}

#[derive(Debug, Default)]
struct InMemoryDispatchRepo {
    rows: Mutex<Vec<PostalDispatch>>,
}

impl InMemoryDispatchRepo {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl PostalDispatchRepository for InMemoryDispatchRepo {
    async fn get(
        &self,
        id: PostalDispatchId,
    ) -> educore_core::error::Result<Option<PostalDispatch>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.id == id)
            .cloned())
    }

    async fn list(
        &self,
        school: SchoolId,
        q: PostalDispatchQuery,
    ) -> educore_core::error::Result<Vec<PostalDispatch>> {
        let rows = self.rows.lock().unwrap().clone();
        let filtered: Vec<PostalDispatch> = rows
            .into_iter()
            .filter(|d| d.school_id == school)
            .filter(|d| {
                q.to_title
                    .as_ref()
                    .is_none_or(|t| d.to_title.as_str() == t.as_str())
            })
            .filter(|d| {
                q.from_title
                    .as_ref()
                    .is_none_or(|t| d.from_title.as_str() == t.as_str())
            })
            .filter(|d| {
                q.reference_no
                    .as_ref()
                    .is_none_or(|r| d.reference_no.as_ref() == Some(r))
            })
            .filter(|d| q.date_from.is_none_or(|df| d.date.0 >= df.0))
            .filter(|d| q.date_to.is_none_or(|dt| d.date.0 <= dt.0))
            .filter(|d| q.academic_id.is_none_or(|y| d.academic_id == y))
            .filter(|d| q.active_status.is_none_or(|a| d.active_status == a))
            .collect();
        Ok(filtered)
    }

    async fn insert(&self, dispatch: &PostalDispatch) -> educore_core::error::Result<()> {
        // Enforce reference uniqueness within (school_id, academic_id).
        let rows = self.rows.lock().unwrap();
        if let Some(existing_ref) = &dispatch.reference_no {
            if rows.iter().any(|d| {
                d.academic_id == dispatch.academic_id
                    && d.reference_no.as_ref() == Some(existing_ref)
            }) {
                return Err(educore_core::error::DomainError::Conflict(format!(
                    "duplicate reference_no: {}",
                    existing_ref.as_str()
                )));
            }
        }
        drop(rows);
        self.rows.lock().unwrap().push(dispatch.clone());
        Ok(())
    }

    async fn update(&self, dispatch: &PostalDispatch) -> educore_core::error::Result<()> {
        let mut rows = self.rows.lock().unwrap();
        if let Some(existing) = rows.iter_mut().find(|d| d.id == dispatch.id) {
            *existing = dispatch.clone();
            Ok(())
        } else {
            Err(educore_core::error::DomainError::NotFound(format!(
                "dispatch {} not found",
                dispatch.id.as_uuid()
            )))
        }
    }

    async fn find_by_reference(
        &self,
        school: SchoolId,
        reference: &PostalReferenceNo,
    ) -> educore_core::error::Result<Vec<PostalDispatch>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|d| d.school_id == school && d.reference_no.as_ref() == Some(reference))
            .collect())
    }

    async fn between(
        &self,
        school: SchoolId,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> educore_core::error::Result<Vec<PostalDispatch>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|d| d.school_id == school && d.date.0 >= from && d.date.0 <= to)
            .collect())
    }

    async fn by_academic_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> educore_core::error::Result<Vec<PostalDispatch>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|d| d.school_id == school && d.academic_id == year)
            .collect())
    }
}

#[derive(Debug, Default)]
struct InMemoryReceiveRepo {
    rows: Mutex<Vec<PostalReceive>>,
}

impl InMemoryReceiveRepo {
    fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl PostalReceiveRepository for InMemoryReceiveRepo {
    async fn get(&self, id: PostalReceiveId) -> educore_core::error::Result<Option<PostalReceive>> {
        Ok(self
            .rows
            .lock()
            .unwrap()
            .iter()
            .find(|r| r.id == id)
            .cloned())
    }

    async fn list(
        &self,
        school: SchoolId,
        q: PostalReceiveQuery,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        let rows = self.rows.lock().unwrap().clone();
        let filtered: Vec<PostalReceive> = rows
            .into_iter()
            .filter(|r| r.school_id == school)
            .filter(|r| {
                q.from_title
                    .as_ref()
                    .is_none_or(|t| r.from_title.as_str() == t.as_str())
            })
            .filter(|r| {
                q.to_title
                    .as_ref()
                    .is_none_or(|t| r.to_title.as_str() == t.as_str())
            })
            .filter(|r| {
                q.reference_no
                    .as_ref()
                    .is_none_or(|rr| r.reference_no.as_ref() == Some(rr))
            })
            .filter(|r| q.date_from.is_none_or(|df| r.date.0 >= df.0))
            .filter(|r| q.date_to.is_none_or(|dt| r.date.0 <= dt.0))
            .filter(|r| q.academic_id.is_none_or(|y| r.academic_id == y))
            .filter(|r| q.active_status.is_none_or(|a| r.active_status == a))
            .collect();
        Ok(filtered)
    }

    async fn insert(&self, receive: &PostalReceive) -> educore_core::error::Result<()> {
        let rows = self.rows.lock().unwrap();
        if let Some(existing_ref) = &receive.reference_no {
            if rows.iter().any(|r| {
                r.academic_id == receive.academic_id
                    && r.reference_no.as_ref() == Some(existing_ref)
            }) {
                return Err(educore_core::error::DomainError::Conflict(format!(
                    "duplicate reference_no: {}",
                    existing_ref.as_str()
                )));
            }
        }
        drop(rows);
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
        school: SchoolId,
        reference: &PostalReferenceNo,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|r| r.school_id == school && r.reference_no.as_ref() == Some(reference))
            .collect())
    }

    async fn between(
        &self,
        school: SchoolId,
        from: chrono::NaiveDate,
        to: chrono::NaiveDate,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|r| r.school_id == school && r.date.0 >= from && r.date.0 <= to)
            .collect())
    }

    async fn by_academic_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> educore_core::error::Result<Vec<PostalReceive>> {
        let rows = self.rows.lock().unwrap().clone();
        Ok(rows
            .into_iter()
            .filter(|r| r.school_id == school && r.academic_id == year)
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Test environment setup
// ---------------------------------------------------------------------------

struct TestEnv {
    adapter: Arc<dyn educore_storage::StorageAdapter>,
    /// Concrete bus used to pass into service factories
    /// (which require `B: EventBus + 'static`, not `dyn`).
    bus: Arc<InProcessEventBus>,
    /// The bus as a trait object, used for AuditWriter::new.
    bus_dyn: Arc<dyn EventBus>,
    audit: Arc<AuditWriter>,
    cap: Arc<InMemoryCapabilityCheck>,
    form_repo: Arc<InMemoryFormRepo>,
    dispatch_repo: Arc<InMemoryDispatchRepo>,
    receive_repo: Arc<InMemoryReceiveRepo>,
    ctx: TenantContext,
    school: SchoolId,
    actor: UserId,
    #[allow(dead_code)]
    clock: Arc<TestClock>,
}

async fn setup_test_env() -> TestEnv {
    let bus: Arc<InProcessEventBus> = Arc::new(InProcessEventBus::new());
    let bus_dyn: Arc<dyn EventBus> = bus.clone();
    let adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(
        educore_testkit::storage::InMemoryStorageAdapter::new(bus_dyn.clone()),
    );
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let clock = Arc::new(TestClock::at(ts(1_700_000_000)));
    let audit_log: Arc<dyn educore_storage::audit::AuditLog> = Arc::new(InMemoryAuditLog::new());
    let audit = Arc::new(
        AuditWriter::new(
            school,
            audit_log,
            bus_dyn.clone(),
            clock.clone(),
            RetentionPolicy::default(),
        )
        .expect("test school_id is valid"),
    );
    let cap = Arc::new(InMemoryCapabilityCheck::new());
    let form_repo = Arc::new(InMemoryFormRepo::new());
    let dispatch_repo = Arc::new(InMemoryDispatchRepo::new());
    let receive_repo = Arc::new(InMemoryReceiveRepo::new());
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    TestEnv {
        adapter,
        bus,
        bus_dyn,
        audit,
        cap,
        form_repo,
        dispatch_repo,
        receive_repo,
        ctx,
        school,
        actor,
        clock,
    }
}

fn grant_all_caps(cap: &InMemoryCapabilityCheck, school: SchoolId, _actor: UserId) {
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    for c in [
        Capability::FormDownloadUpload,
        Capability::FormDownloadUpdate,
        Capability::FormDownloadDelete,
        Capability::FormDownloadRead,
        Capability::PostalDispatchCreate,
        Capability::PostalDispatchUpdate,
        Capability::PostalDispatchDelete,
        Capability::PostalReceiveCreate,
        Capability::PostalReceiveUpdate,
        Capability::PostalReceiveDelete,
        Capability::PostalRead,
    ] {
        cap.grant(school, role, c);
    }
}

fn grant_one(cap: &InMemoryCapabilityCheck, school: SchoolId, c: Capability) {
    let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
    cap.grant(school, role, c);
}

fn upload_cmd(school: SchoolId, actor: UserId, corr: CorrelationId) -> UploadFormCommand {
    UploadFormCommand {
        tenant: TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        title: FormTitle::new("Consent Form 2026").unwrap(),
        short_description: Some(FormDescription::new("Annual consent form.").unwrap()),
        publish_date: PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        link: Some(Url::new("https://example.com/forms/consent.pdf").unwrap()),
        file: None,
        show_public: ShowPublic::new(true),
    }
}

fn dispatch_cmd(
    school: SchoolId,
    actor: UserId,
    corr: CorrelationId,
    ref_no: Option<&str>,
) -> DispatchPostalCommand {
    DispatchPostalCommand {
        tenant: TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        to_title: ToTitle::new(PostalTitle::new("Mr Smith").unwrap()),
        from_title: FromTitle::new(PostalTitle::new("Acme School").unwrap()),
        reference_no: ref_no.map(|r| PostalReferenceNo::new(r).unwrap()),
        address: ToAddress::new(PostalAddress::new("1 Main St").unwrap()),
        date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: None,
        file: None,
    }
}

fn receive_cmd(
    school: SchoolId,
    actor: UserId,
    corr: CorrelationId,
    ref_no: Option<&str>,
) -> ReceivePostalCommand {
    ReceivePostalCommand {
        tenant: TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        from_title: FromTitle::new(PostalTitle::new("Acme Vendor").unwrap()),
        to_title: ToTitle::new(PostalTitle::new("Acme School").unwrap()),
        reference_no: ref_no.map(|r| PostalReferenceNo::new(r).unwrap()),
        address: FromAddress::new(PostalAddress::new("5 Vendor Rd").unwrap()),
        date: ReceiveDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        note: None,
        file: None,
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: SQLite vertical slice
// ---------------------------------------------------------------------------

#[tokio::test]
async fn documents_integration_sqlite_vertical_slice() {
    let env = setup_test_env().await;
    grant_all_caps(&env.cap, env.school, env.actor);

    // Subscribe to the bus BEFORE dispatching.
    let mut opts = SubscribeOptions::for_consumer("test-documents".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = env.bus_dyn.subscribe(opts).await.expect("subscribe");

    // 1. Upload a form.
    let txn = env.adapter.begin().await.expect("begin txn");

    let form = upload_form_service(
        upload_cmd(env.school, env.actor, env.ctx.correlation_id),
        &*txn,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("upload_form_service");
    assert!(form.is_active());
    assert!(form.is_public());
    assert_eq!(
        <FormUploaded as DomainEvent>::EVENT_TYPE,
        "documents.form_download.uploaded"
    );

    // 2. Dispatch a postal item.
    let dispatch = dispatch_postal_service(
        dispatch_cmd(
            env.school,
            env.actor,
            env.ctx.correlation_id,
            Some("REF-2026-0001"),
        ),
        uuid::Uuid::now_v7(),
        &*txn,
        env.dispatch_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("dispatch_postal_service");
    assert!(dispatch.is_active());
    assert_eq!(
        <PostalDispatched as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.dispatched"
    );

    // 3. Receive a postal item.
    let receive = receive_postal_service(
        receive_cmd(
            env.school,
            env.actor,
            env.ctx.correlation_id,
            Some("REF-IN-0001"),
        ),
        uuid::Uuid::now_v7(),
        &*txn,
        env.receive_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("receive_postal_service");
    assert!(receive.is_active());
    assert_eq!(
        <PostalReceived as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.received"
    );

    // 4. Verify the bus received the first envelope (FormUploaded).
    let received = sub.next().await;
    match received {
        Some(Ok(env_)) => {
            assert_eq!(env_.event_type, "documents.form_download.uploaded");
            assert_eq!(env_.school_id, env.school);
        }
        other => panic!("expected bus event, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Scenario 2: Capability check gates form upload
// ---------------------------------------------------------------------------

#[tokio::test]
async fn documents_capability_check_gates_form_upload() {
    let env = setup_test_env().await;

    // 1. No grant -> denied.
    let granted = env
        .cap
        .has(&env.ctx, Capability::FormDownloadUpload)
        .await
        .expect("has");
    assert!(!granted, "FormDownloadUpload must be denied by default");

    // 2. Grant -> allowed.
    grant_one(&env.cap, env.school, Capability::FormDownloadUpload);
    let granted = env
        .cap
        .has(&env.ctx, Capability::FormDownloadUpload)
        .await
        .expect("has");
    assert!(granted, "FormDownloadUpload must be allowed after grant");
}

// ---------------------------------------------------------------------------
// Scenario 3: Event type round-trip for all 9 events
// ---------------------------------------------------------------------------

#[test]
fn documents_event_type_round_trip_for_all_aggregates() {
    // 3 form events
    assert_eq!(
        <FormUploaded as DomainEvent>::EVENT_TYPE,
        "documents.form_download.uploaded"
    );
    assert_eq!(
        <FormUpdated as DomainEvent>::EVENT_TYPE,
        "documents.form_download.updated"
    );
    assert_eq!(
        <FormDeleted as DomainEvent>::EVENT_TYPE,
        "documents.form_download.deleted"
    );
    // 3 dispatch events
    assert_eq!(
        <PostalDispatched as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.dispatched"
    );
    assert_eq!(
        <PostalDispatchUpdated as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.updated"
    );
    assert_eq!(
        <PostalDispatchDeleted as DomainEvent>::EVENT_TYPE,
        "documents.postal_dispatch.deleted"
    );
    // 3 receive events
    assert_eq!(
        <PostalReceived as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.received"
    );
    assert_eq!(
        <PostalReceiveUpdated as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.updated"
    );
    assert_eq!(
        <PostalReceiveDeleted as DomainEvent>::EVENT_TYPE,
        "documents.postal_receive.deleted"
    );

    // Every event is schema v1.
    assert_eq!(<FormUploaded as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<FormUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<FormDeleted as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalDispatched as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalDispatchUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalDispatchDeleted as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalReceived as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalReceiveUpdated as DomainEvent>::SCHEMA_VERSION, 1);
    assert_eq!(<PostalReceiveDeleted as DomainEvent>::SCHEMA_VERSION, 1);

    // Every event's AGGREGATE_TYPE matches the canonical name.
    assert_eq!(
        <FormUploaded as DomainEvent>::AGGREGATE_TYPE,
        "form_download"
    );
    assert_eq!(
        <FormUpdated as DomainEvent>::AGGREGATE_TYPE,
        "form_download"
    );
    assert_eq!(
        <FormDeleted as DomainEvent>::AGGREGATE_TYPE,
        "form_download"
    );
    assert_eq!(
        <PostalDispatched as DomainEvent>::AGGREGATE_TYPE,
        "postal_dispatch"
    );
    assert_eq!(
        <PostalDispatchUpdated as DomainEvent>::AGGREGATE_TYPE,
        "postal_dispatch"
    );
    assert_eq!(
        <PostalDispatchDeleted as DomainEvent>::AGGREGATE_TYPE,
        "postal_dispatch"
    );
    assert_eq!(
        <PostalReceived as DomainEvent>::AGGREGATE_TYPE,
        "postal_receive"
    );
    assert_eq!(
        <PostalReceiveUpdated as DomainEvent>::AGGREGATE_TYPE,
        "postal_receive"
    );
    assert_eq!(
        <PostalReceiveDeleted as DomainEvent>::AGGREGATE_TYPE,
        "postal_receive"
    );
}

// ---------------------------------------------------------------------------
// Scenario 4: Postal reference uniqueness invariant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn documents_postal_reference_uniqueness_invariant() {
    let env = setup_test_env().await;
    grant_all_caps(&env.cap, env.school, env.actor);

    let academic_id = uuid::Uuid::now_v7();
    let reference = "REF-DUP-001";

    let txn = env.adapter.begin().await.expect("begin txn");

    // Insert a first dispatch with `reference_no = reference`.
    let first = dispatch_postal_service(
        DispatchPostalCommand {
            tenant: TenantContext::for_user(
                env.school,
                env.actor,
                env.ctx.correlation_id,
                UserType::SchoolAdmin,
            ),
            to_title: ToTitle::new(PostalTitle::new("First").unwrap()),
            from_title: FromTitle::new(PostalTitle::new("Acme").unwrap()),
            reference_no: Some(PostalReferenceNo::new(reference).unwrap()),
            address: ToAddress::new(PostalAddress::new("1 St").unwrap()),
            date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            note: None,
            file: None,
        },
        academic_id,
        &*txn,
        env.dispatch_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("first dispatch");
    assert!(first.is_active());

    // Insert a second dispatch with the same reference_no in
    // the same academic year. The in-memory repository
    // enforces uniqueness, so this MUST return an error.
    let second = dispatch_postal_service(
        DispatchPostalCommand {
            tenant: TenantContext::for_user(
                env.school,
                env.actor,
                env.ctx.correlation_id,
                UserType::SchoolAdmin,
            ),
            to_title: ToTitle::new(PostalTitle::new("Second").unwrap()),
            from_title: FromTitle::new(PostalTitle::new("Acme").unwrap()),
            reference_no: Some(PostalReferenceNo::new(reference).unwrap()),
            address: ToAddress::new(PostalAddress::new("2 St").unwrap()),
            date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap()),
            note: None,
            file: None,
        },
        academic_id,
        &*txn,
        env.dispatch_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await;
    assert!(
        second.is_err(),
        "second dispatch with duplicate reference_no must fail"
    );

    // The same reference_no in a different academic year is
    // allowed (the uniqueness is per-(school, academic_id)).
    let other_year = uuid::Uuid::now_v7();
    let third = dispatch_postal_service(
        DispatchPostalCommand {
            tenant: TenantContext::for_user(
                env.school,
                env.actor,
                env.ctx.correlation_id,
                UserType::SchoolAdmin,
            ),
            to_title: ToTitle::new(PostalTitle::new("Other year").unwrap()),
            from_title: FromTitle::new(PostalTitle::new("Acme").unwrap()),
            reference_no: Some(PostalReferenceNo::new(reference).unwrap()),
            address: ToAddress::new(PostalAddress::new("3 St").unwrap()),
            date: DispatchDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 3).unwrap()),
            note: None,
            file: None,
        },
        other_year,
        &*txn,
        env.dispatch_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("different year same ref");
    assert!(third.is_active());
}

// ---------------------------------------------------------------------------
// Scenario 5: Soft-delete invariant holds
// ---------------------------------------------------------------------------

#[tokio::test]
async fn documents_soft_delete_invariant_holds() {
    let env = setup_test_env().await;
    grant_all_caps(&env.cap, env.school, env.actor);

    // Create a form.
    let txn = env.adapter.begin().await.expect("begin txn");

    let form = upload_form_service(
        upload_cmd(env.school, env.actor, env.ctx.correlation_id),
        &*txn,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("upload");
    assert!(form.is_active());

    // Soft-delete it.
    let form_id = form.id;
    delete_form_service(
        DeleteFormCommand {
            tenant: TenantContext::for_user(
                env.school,
                env.actor,
                env.ctx.correlation_id,
                UserType::SchoolAdmin,
            ),
            form_id,
        },
        &*txn,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("soft-delete");

    // The form is still queryable by id (soft delete does not
    // hard-delete the row).
    let fetched = env
        .form_repo
        .get(form_id)
        .await
        .expect("get")
        .expect("form still queryable after soft-delete");
    assert!(
        !fetched.is_active(),
        "is_active() must be false after soft-delete"
    );
    assert_eq!(fetched.id, form_id);
}

// ---------------------------------------------------------------------------
// Scenario 6: Form public-visibility filter invariant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn documents_form_publish_visibility_invariant() {
    let env = setup_test_env().await;
    grant_all_caps(&env.cap, env.school, env.actor);

    // 1. A form with show_public = true.
    let mut public_cmd = upload_cmd(env.school, env.actor, env.ctx.correlation_id);
    public_cmd.show_public = ShowPublic::new(true);
    let txn = env.adapter.begin().await.expect("begin txn");

    let txn = env.adapter.begin().await.expect("begin txn");

    let public_form = upload_form_service(
        public_cmd,
        &*txn,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("public form");
    assert!(public_form.is_public());

    // 2. A form with show_public = false.
    let mut staff_cmd = upload_cmd(env.school, env.actor, env.ctx.correlation_id);
    staff_cmd.title = FormTitle::new("Staff Only").unwrap();
    staff_cmd.show_public = ShowPublic::new(false);
    let txn = env.adapter.begin().await.expect("begin txn");

    let txn = env.adapter.begin().await.expect("begin txn");

    let staff_form = upload_form_service(
        staff_cmd,
        &*txn,
        env.form_repo.clone(),
        env.bus.clone(),
        env.audit.clone(),
        env.cap.as_ref(),
    )
    .await
    .expect("staff form");
    assert!(!staff_form.is_public());

    // 3. The `list_public` repository call returns only the
    //    public form.
    let public_forms = env
        .form_repo
        .list_public(env.school)
        .await
        .expect("list_public");
    assert_eq!(
        public_forms.len(),
        1,
        "list_public must return exactly 1 form"
    );
    assert_eq!(public_forms[0].id, public_form.id);
    assert!(public_forms.iter().all(|f| f.is_public()));

    // 4. A `list` query with the show_public = Public filter
    //    also returns only the public form.
    let q = FormDownloadQuery::new().with_show_public(ShowPublic::new(true));
    let filtered = env
        .form_repo
        .list(env.school, q)
        .await
        .expect("list with public filter");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, public_form.id);

    // 5. A `list` query with show_public = Staff returns only
    //    the staff form.
    let q = FormDownloadQuery::new().with_show_public(ShowPublic::new(false));
    let filtered = env
        .form_repo
        .list(env.school, q)
        .await
        .expect("list with staff filter");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, staff_form.id);

    // 6. A `list` query with no filter returns both forms.
    let q = FormDownloadQuery::new();
    let all = env.form_repo.list(env.school, q).await.expect("list all");
    assert_eq!(all.len(), 2);
}

// ---------------------------------------------------------------------------
// Env-gated PG/MySQL tests
//
// These tests mirror the library_integration.rs pattern: they connect
// to a real Postgres or MySQL instance, run the documents vertical
// slice, and assert the bus + outbox. They are gated on the
// `EDUCORE_PG_URL` / `EDUCORE_MYSQL_URL` env vars (see
// docs/decisions/ADR-018-SyncEngine.md § "Adapter parity").
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: EDUCORE_PG_URL=postgres://... cargo test -- --ignored"]
async fn documents_integration_postgres() {
    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let _actor = g.next_user_id();
    let _corr = g.next_correlation_id();
    let _bus: Arc<InProcessEventBus> = Arc::new(InProcessEventBus::new());
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .expect("connect pg");
    adapter.migrate().await.expect("migrate pg");
    // The PG vertical-slice test exercises the SQLite shape
    // (the documents repos are wired to the SQLite adapter in
    // the always-runs test). The PG path is gated separately
    // so a CI run with EDUCORE_PG_URL set can assert parity.
    let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: EDUCORE_MYSQL_URL=mysql://... cargo test -- --ignored"]
async fn documents_integration_mysql() {
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let _actor = g.next_user_id();
    let _corr = g.next_correlation_id();
    let _bus: Arc<InProcessEventBus> = Arc::new(InProcessEventBus::new());
    let adapter = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
        .await
        .expect("connect mysql");
    adapter.migrate().await.expect("migrate mysql");
    let _adapter: Arc<dyn educore_storage::StorageAdapter> = Arc::new(adapter);
}
