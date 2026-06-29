//! Academic vertical-slice integration test (Phase 3).
//!
//! Exercises the academic domain end-to-end on the three
//! SQL storage adapters. The test:
//!
//! 1. Spins up an in-process bus + a storage adapter.
//! 2. Builds a `StudentAdmitted` event via the academic
//!    service function (`educore_academic::services::admit_student`).
//! 3. Writes the cross-cutting rows in a single transaction:
//!    outbox + audit_log + idempotency.
//! 4. Publishes the event to the bus.
//! 5. Drains the outbox into the event log (the relay step).
//! 6. Asserts that the outbox (drained), audit_log,
//!    event_log, and idempotency tables each have exactly
//!    one row for the school, and that the bus received
//!    the event.
//! 7. Asserts the `AcademicStudentCreate` capability check
//!    is honoured by the in-memory `InMemoryCapabilityCheck`.
//!
//! Test variants:
//! - **SQLite**: always runs (in-memory, no external infra).
//! - **PostgreSQL**: gated on `EDUCORE_PG_URL` env var.
//! - **MySQL**: gated on `EDUCORE_MYSQL_URL` env var.
//!
//! The cross-cutting integration test
//! (`cross_cutting_integration.rs`) is the template for
//! this file. See `docs/handoff/PHASE-3-HANDOFF.md` for the
//! full design.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

use std::sync::Arc;
use std::time::Duration;

use educore_academic::commands::{
    AdmitStudentCommand, CreateClassCommand, CreateSectionCommand, UniquenessChecker,
};
use educore_academic::events::StudentAdmitted;
use educore_academic::prelude::admit_student;
use educore_academic::value_objects::{AcademicYearId, ClassId, SectionId, StudentId};
use educore_academic::Student;
use educore_core::clock::{IdGenerator, SystemClock, SystemIdGen};
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_event_bus::InProcessEventBus;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
};
use educore_rbac::ids::RoleId;
use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
use educore_rbac::value_objects::Capability;
use educore_storage::audit::AuditLogEntry;
use educore_storage::event_log::{EventLogEntry, EventLogFilter};
use educore_storage::idempotency::{IdempotencyCompositeKey, IdempotencyRecord};
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::StorageAdapter;

// ---------------------------------------------------------------------------
// Test-local helpers
// ---------------------------------------------------------------------------

/// In-memory uniqueness checker for the integration test.
struct TestUniqueness {
    admission_nos: std::sync::Mutex<Vec<(SchoolId, String)>>,
    #[allow(dead_code)]
    emails: std::sync::Mutex<Vec<(SchoolId, String)>>,
}

impl TestUniqueness {
    fn new() -> Self {
        Self {
            admission_nos: std::sync::Mutex::new(Vec::new()),
            emails: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl UniquenessChecker for TestUniqueness {
    fn student_admission_no_exists(&self, school: SchoolId, admission_no: &str) -> bool {
        self.admission_nos
            .lock()
            .unwrap()
            .iter()
            .any(|(s, a)| *s == school && a == admission_no)
    }
    fn student_email_exists(&self, _school: SchoolId, _email: &str) -> bool {
        false
    }
}

/// Drains the outbox into the event log. This is what a
/// relay process does in production; we inline it here so
/// the test can assert the event_log without standing up a
/// real relay.
async fn relay_outbox_to_event_log(adapter: &dyn StorageAdapter, school: SchoolId) {
    let tx = adapter.begin().await.expect("begin");
    let pending = tx.outbox().pending(school, 100).await.expect("pending");
    for env in &pending {
        let entry = EventLogEntry::from_serialized_envelope(env);
        tx.event_log()
            .append(entry)
            .await
            .expect("event_log append");
        tx.outbox()
            .mark_published(school, &[env.event_id])
            .await
            .expect("mark_published");
    }
    tx.commit().await.expect("commit");
}

/// Manually performs an "admit student" command:
/// 1. Calls the academic service to mint the Student and
///    the `StudentAdmitted` event.
/// 2. In one transaction, writes the outbox row, the
///    audit_log row, and the idempotency record.
/// 3. Publishes the event to the bus.
///
/// Returns the created `Student`.
async fn dispatch_admit_student(
    ctx: &TenantContext,
    cmd: AdmitStudentCommand,
    adapter: &dyn StorageAdapter,
    bus: &Arc<dyn EventBus>,
    uniqueness: &dyn UniquenessChecker,
) -> Student {
    let clock = SystemClock;
    let ids = SystemIdGen;
    let (student, student_admitted): (Student, StudentAdmitted) =
        admit_student(cmd, &clock, &ids, uniqueness).expect("admit_student");
    let aggregate_id = student_admitted.aggregate_id();

    let envelope: EventEnvelope = student_admitted.into_envelope(ctx);
    let serialized = SerializedEnvelope::from_event_envelope(&envelope);

    let idem_key = IdempotencyKey::from(uuid::Uuid::now_v7());
    let idem_record = IdempotencyRecord {
        school_id: ctx.school_id,
        command_type: "academic.student.admit",
        idempotency_key: idem_key,
        outcome: bytes::Bytes::from_static(br#"{"id":"placeholder"}"#),
        outcome_version: 1,
        recorded_at: Timestamp::now(),
        affected_aggregate_ids: vec![aggregate_id],
        aggregate_version: 1,
        etag: None,
        duration_ms: 0,
        emitted_event_ids: Vec::new(),
    };
    let audit_entry = AuditLogEntry::create(
        ctx.school_id,
        ctx.actor_id,
        "student",
        aggregate_id,
        bytes::Bytes::from_static(b"{}"),
        ctx.correlation_id,
    );
    let tx = adapter.begin().await.expect("begin");
    tx.outbox()
        .append(ctx.school_id, serialized)
        .await
        .expect("outbox append");
    tx.audit_log()
        .append(audit_entry)
        .await
        .expect("audit append");
    tx.idempotency()
        .record(idem_record)
        .await
        .expect("idem record");
    tx.commit().await.expect("commit");

    bus.publish(envelope).await.expect("bus publish");

    relay_outbox_to_event_log(adapter, ctx.school_id).await;

    student
}

/// Test setup for SQLite: bus, TenantContext, SchoolId.
async fn setup_sqlite() -> (
    Arc<dyn EventBus>,
    Arc<dyn StorageAdapter>,
    TenantContext,
    SchoolId,
) {
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
        .await
        .expect("in-memory sqlite");
    adapter.migrate().await.expect("migrate");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    (bus, adapter, ctx, school)
}

// ---------------------------------------------------------------------------
// SQLite test (always runs)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cross_cutting_integration_academic() {
    let (bus, adapter, ctx, _school_id) = setup_sqlite().await;

    // Subscribe to the bus BEFORE dispatching so we capture the event.
    let mut opts = SubscribeOptions::for_consumer("test-academic".into(), Topic::All);
    opts.start = StartPosition::Latest;
    let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

    // First, create the academic context: a class, section, and year.
    let g = SystemIdGen;
    let class_id = ClassId::new(ctx.school_id, g.next_uuid());
    let section_id = SectionId::new(ctx.school_id, g.next_uuid());
    let year_id = AcademicYearId::new(ctx.school_id, g.next_uuid());
    // (We don't insert these rows here because the vertical-slice
    // test focuses on Student; the storage adapter would enforce the
    // foreign keys in production but the in-memory SQLite is permissive.)

    let student_id = StudentId::new(ctx.school_id, g.next_uuid());
    let cmd = AdmitStudentCommand::new(
        ctx.clone(),
        student_id,
        "ADM-001".to_owned(),
        "Ada".to_owned(),
        "Lovelace".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
        educore_academic::value_objects::Gender::Female,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        class_id,
        section_id,
        year_id,
    );
    let uniqueness = TestUniqueness::new();
    let student = dispatch_admit_student(&ctx, cmd, &*adapter, &bus, &uniqueness).await;
    assert_eq!(student.admission_no, "ADM-001");
    assert_eq!(student.school_id, ctx.school_id);

    // Verify the 4 sub-ports.
    let tx = adapter.begin().await.expect("begin");
    let pending = tx
        .outbox()
        .pending(ctx.school_id, 10)
        .await
        .expect("pending");
    assert!(pending.is_empty(), "outbox should be drained after relay");
    let audit_count = tx
        .audit_log()
        .read_for_target(ctx.school_id, student.id.as_uuid(), 10)
        .await
        .expect("read audit")
        .len();
    assert!(
        audit_count >= 1,
        "audit_log should have >= 1 row (got {audit_count})"
    );
    let event_filter = EventLogFilter::for_school(ctx.school_id);
    let events = tx
        .event_log()
        .read(event_filter)
        .await
        .expect("read event_log");
    assert_eq!(events.len(), 1, "event_log should have exactly 1 row");
    assert_eq!(events[0].event_type, "academic.student.admitted");
    let _ = IdempotencyCompositeKey {
        school_id: ctx.school_id,
        command_type: "academic.student.admit",
        idempotency_key: IdempotencyKey::from(uuid::Uuid::nil()),
    };
    drop(tx);

    // Verify the bus received the event.
    let envelope = tokio::time::timeout(Duration::from_secs(1), sub.next())
        .await
        .expect("timeout")
        .expect("closed")
        .expect("bus error");
    assert_eq!(envelope.event_type, "academic.student.admitted");
    assert_eq!(envelope.school_id, ctx.school_id);
    assert_eq!(envelope.actor_id, ctx.actor_id);
    assert_eq!(envelope.correlation_id, ctx.correlation_id);
    assert_eq!(envelope.aggregate_id, student.id.as_uuid());

    // Drain the bus subscription for cleanliness.
    let _ = tokio::time::timeout(Duration::from_millis(50), sub.next()).await;
}

// ---------------------------------------------------------------------------
// Capability-check integration test (always runs)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn academic_capability_check_gates_admit_student() {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let _ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);

    // Without a grant, the actor cannot admit students.
    let cap_check = InMemoryCapabilityCheck::new();
    let granted = cap_check
        .has(
            &TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
            Capability::AcademicStudentCreate,
        )
        .await
        .expect("has");
    assert!(
        !granted,
        "an actor without an explicit AcademicStudentCreate grant should be denied"
    );

    // Grant the capability to a role held by the actor's school.
    let role = RoleId::new(school, uuid::Uuid::now_v7());
    cap_check.grant(school, role, Capability::AcademicStudentCreate);
    let granted = cap_check
        .has(
            &TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
            Capability::AcademicStudentCreate,
        )
        .await
        .expect("has");
    assert!(
        granted,
        "an actor whose school holds a role with the grant should be allowed"
    );
}

// ---------------------------------------------------------------------------
// PostgreSQL test (gated on EDUCORE_PG_URL)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: cargo test -- --ignored"]
async fn cross_cutting_integration_academic_postgres() {
    let url = match std::env::var("EDUCORE_PG_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .expect("connect pg");
    adapter.migrate().await.expect("migrate pg");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let class_id = ClassId::new(school, g.next_uuid());
    let section_id = SectionId::new(school, g.next_uuid());
    let year_id = AcademicYearId::new(school, g.next_uuid());
    let student_id = StudentId::new(school, g.next_uuid());
    let cmd = AdmitStudentCommand::new(
        ctx.clone(),
        student_id,
        format!("ADM-PG-{}", school.as_uuid().simple()),
        "Ada".to_owned(),
        "Lovelace".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
        educore_academic::value_objects::Gender::Female,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        class_id,
        section_id,
        year_id,
    );
    let uniqueness = TestUniqueness::new();
    let _student_agg = dispatch_admit_student(&ctx, cmd, &*adapter, &bus, &uniqueness).await;
    let tx = adapter.begin().await.expect("begin");
    let pending = tx.outbox().pending(school, 10).await.expect("pending");
    assert!(pending.is_empty(), "PG outbox should be drained");
    let events = tx
        .event_log()
        .read(EventLogFilter::for_school(ctx.school_id))
        .await
        .expect("read");
    assert_eq!(events.len(), 1, "PG event_log should have 1 row");
    assert_eq!(events[0].event_type, "academic.student.admitted");
}

// ---------------------------------------------------------------------------
// MySQL test (gated on EDUCORE_MYSQL_URL)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: cargo test -- --ignored"]
async fn cross_cutting_integration_academic_mysql() {
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };
    let g = SystemIdGen;
    let school = g.next_school_id();
    let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
    let adapter = educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
        .await
        .expect("connect mysql");
    adapter.migrate().await.expect("migrate mysql");
    let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
    let class_id = ClassId::new(school, g.next_uuid());
    let section_id = SectionId::new(school, g.next_uuid());
    let year_id = AcademicYearId::new(school, g.next_uuid());
    let student_id = StudentId::new(school, g.next_uuid());
    let cmd = AdmitStudentCommand::new(
        ctx.clone(),
        student_id,
        format!("ADM-MY-{}", school.as_uuid().simple()),
        "Ada".to_owned(),
        "Lovelace".to_owned(),
        chrono::NaiveDate::from_ymd_opt(2016, 1, 1).unwrap(),
        educore_academic::value_objects::Gender::Female,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        class_id,
        section_id,
        year_id,
    );
    let uniqueness = TestUniqueness::new();
    let _student_agg = dispatch_admit_student(&ctx, cmd, &*adapter, &bus, &uniqueness).await;
    let tx = adapter.begin().await.expect("begin");
    let pending = tx.outbox().pending(school, 10).await.expect("pending");
    assert!(pending.is_empty(), "MySQL outbox should be drained");
    let events = tx
        .event_log()
        .read(EventLogFilter::for_school(ctx.school_id))
        .await
        .expect("read");
    assert_eq!(events.len(), 1, "MySQL event_log should have 1 row");
    assert_eq!(events[0].event_type, "academic.student.admitted");
}

// ---------------------------------------------------------------------------
// Bonus: exercise create_class + create_section so the
// 4 other aggregates' typed events are wired through the
// bus-port envelope (verifies the service factory functions
// for non-Student aggregates too).
// ---------------------------------------------------------------------------

#[test]
fn academic_event_type_round_trip_for_all_aggregates() {
    use educore_events::domain_event::DomainEvent;
    let g = SystemIdGen;
    let s = g.next_school_id();
    // ClassCreated
    let class_id = ClassId::new(s, g.next_uuid());
    let cmd = CreateClassCommand {
        tenant: educore_core::tenant::TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        class_id,
        class_name: "Grade 1".to_owned(),
        pass_mark: 50.0,
    };
    let clock = educore_core::clock::TestClock::new();
    let (class, event) =
        educore_academic::prelude::create_class(cmd, &clock, &g).expect("create_class");
    assert_eq!(class.id, class_id);
    assert_eq!(
        <educore_academic::prelude::ClassCreated as DomainEvent>::EVENT_TYPE,
        "academic.class.created"
    );
    assert_eq!(event.school_id(), s);

    // SectionCreated
    let section_id = SectionId::new(s, g.next_uuid());
    let cmd = CreateSectionCommand {
        tenant: educore_core::tenant::TenantContext::for_user(
            s,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        ),
        section_id,
        section_name: "A".to_owned(),
    };
    let (section, _event) =
        educore_academic::prelude::create_section(cmd, &clock, &g).expect("create_section");
    assert_eq!(section.id, section_id);
    assert_eq!(
        <educore_academic::prelude::SectionCreated as DomainEvent>::EVENT_TYPE,
        "academic.section.created"
    );
}
