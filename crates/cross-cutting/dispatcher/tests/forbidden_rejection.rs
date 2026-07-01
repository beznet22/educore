//! Integration tests: CommandDispatcher rejects unauthorized commands.
//!
//! Per `docs/architecture.md` § "Command Bus + Dispatcher" and
//! `docs/ports/authentication.md` § "RBAC enforcement at the command
//! boundary", every state-changing command flows through
//! [`CommandDispatcher::dispatch`]. The dispatcher's first pipeline
//! step is a per-capability RBAC check — a denied capability
//! short-circuits with [`DomainError::Forbidden`] **before** any
//! storage I/O, transaction begin, outbox append, audit row, or
//! bus publish.
//!
//! These tests prove that contract end-to-end by wiring an
//! in-process `CapabilityCheck` mock that returns `false` for the
//! target capability and asserting the dispatcher surfaces
//! [`DomainError::Forbidden`]. Each test exercises a different
//! RBAC domain (attendance, communication, documents, academic,
//! assessment, finance, hr) so the suite guards against accidental
//! regressions where the dispatcher drops a required-capability
//! check for a specific domain.
//!
//! The capability strings used here are the canonical dotted
//! `<Domain>.<Aggregate>.<Action>` form returned by
//! `educore_rbac::value_objects::Capability::as_str()`. The
//! dispatcher's `CapabilityCheck` port is intentionally string-typed
//! (see `crates/cross-cutting/dispatcher/src/dispatcher.rs` §
//! "Why a local `CapabilityCheck` trait?") so the strings used
//! here are the same wire format the production `RbacPort::require`
//! adapter bridges to the typed `Capability` enum.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::future::Future;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;

use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{EventId, IdempotencyKey, SchoolId, UserId};
use educore_core::tenant::{TenantContext, UserType};
use educore_core::value_objects::Timestamp;
use educore_dispatcher::{CapabilityCheck, CommandBounds, CommandDispatcher};
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;
use educore_events::event_bus::{
    BatchReceipt, EventBus, EventSubscription, PublishReceipt, SubscribeOptions,
};
use educore_storage::idempotency::{
    IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};
use educore_storage::outbox::{Outbox, SerializedEnvelope};
use educore_storage::transaction::{TenantTransaction, Transaction};
use educore_storage::{
    AuditLog, AuditLogEntry, EventLog, EventLogEntry, EventLogFilter, StorageAdapter,
};

// ============================================================================
// Canonical RBAC capability strings used in these tests.
//
// Mirrors the wire form returned by
// `educore_rbac::value_objects::Capability::as_str()`. The dispatcher's
// `CapabilityCheck` port takes `&str`, so the strings below are the
// exact bytes the production RBAC adapter would forward.
// ============================================================================

/// `Attendance.Staff.Create` — mark a daily staff attendance record
/// (the canonical "mark attendance" capability; the source comment
/// in `educore_rbac` describes `AttendanceStaffCreate` as
/// "Create (mark) a daily staff attendance record").
const CAP_ATTENDANCE_MARK: &str = "Attendance.Staff.Create";

/// `Communication.Notification.Read.All` — read all notifications
/// across the tenant.
const CAP_COMMUNICATION_NOTIFICATION_READ_ALL: &str =
    "Communication.Notification.Read.All";

/// `Documents.Form.Read.Public` — read a public form download.
const CAP_DOCUMENTS_FORM_READ_PUBLIC: &str = "Documents.Form.Read.Public";

/// `Academic.Guardian` — operate on the guardian aggregate
/// (e.g. attach/detach a guardian to a student).
const CAP_ACADEMIC_GUARDIAN: &str = "Academic.Guardian";

/// `Assessment.Exam.Setup` — configure an exam's setup metadata
/// (date, duration, room, invigilator).
const CAP_ASSESSMENT_EXAM_SETUP: &str = "Assessment.Exam.Setup";

/// `Finance.Fees.Assign.Discount.Update` — update a fees-assignment
/// discount.
const CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE: &str =
    "Finance.Fees.Assign.Discount.Update";

/// `Hr.Staff.AssignClassTeacher.Update` — update the class-teacher
/// assignment for a staff member.
const CAP_HR_STAFF_ASSIGN_CLASS_TEACHER_UPDATE: &str =
    "Hr.Staff.AssignClassTeacher.Update";

// ============================================================================
// Test command + event + aggregate
// ============================================================================

/// Minimal command for the integration tests. Each test builds one
/// and points it at a different RBAC domain via the
/// `required_capabilities` slice passed to `dispatch`.
#[derive(Debug)]
struct TestCommand {
    tenant: TenantContext,
    command_type: &'static str,
    idempotency_key: Option<IdempotencyKey>,
    action: &'static str,
    target_type: &'static str,
}

impl CommandBounds for TestCommand {
    fn tenant(&self) -> &TenantContext {
        &self.tenant
    }
    fn command_type(&self) -> &'static str {
        self.command_type
    }
    fn idempotency_key(&self) -> Option<IdempotencyKey> {
        self.idempotency_key
    }
    fn action(&self) -> &'static str {
        self.action
    }
    fn target_type(&self) -> &'static str {
        self.target_type
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
struct TestAggregate {
    id: uuid::Uuid,
    name: String,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct TestEvent {
    event_id: EventId,
    aggregate_id: uuid::Uuid,
    school_id: SchoolId,
    occurred_at: Timestamp,
    name: String,
}

impl DomainEvent for TestEvent {
    const EVENT_TYPE: &'static str = "test.aggregate.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "test_aggregate";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> uuid::Uuid {
        self.aggregate_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ============================================================================
// In-process mocks for the five ports
// ============================================================================

/// In-memory bus. The dispatcher publishes exactly one envelope on
/// the happy path; this mock records every published envelope so
/// the RBAC tests can assert "no envelope was published" on a
/// forbidden command.
#[derive(Debug, Default)]
struct InMemoryBus {
    published: Mutex<Vec<EventEnvelope>>,
}

impl InMemoryBus {
    fn new() -> Self {
        Self::default()
    }
    fn published_len(&self) -> usize {
        self.published
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len()
    }
}

#[async_trait]
impl EventBus for InMemoryBus {
    async fn publish(&self, envelope: EventEnvelope) -> Result<PublishReceipt> {
        let receipt = PublishReceipt {
            event_id: envelope.event_id,
            topic: String::new(),
            accepted_at: envelope.occurred_at,
        };
        self.published
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(envelope);
        Ok(receipt)
    }

    async fn publish_batch(&self, _envelopes: Vec<EventEnvelope>) -> Result<BatchReceipt> {
        Err(DomainError::not_supported(
            "InMemoryBus::publish_batch is not exercised by the RBAC tests",
        ))
    }

    async fn subscribe(
        &self,
        _options: SubscribeOptions,
    ) -> Result<Box<dyn EventSubscription>> {
        Err(DomainError::not_supported(
            "InMemoryBus::subscribe is not exercised by the RBAC tests",
        ))
    }
}

/// In-memory outbox. Records every `append` call so the RBAC
/// tests can assert "no outbox row was staged" on a forbidden
/// command.
#[derive(Debug, Default)]
struct InMemoryOutbox {
    rows: Mutex<Vec<SerializedEnvelope>>,
}

impl InMemoryOutbox {
    fn new() -> Self {
        Self::default()
    }
    fn rows_len(&self) -> usize {
        self.rows.lock().unwrap_or_else(|p| p.into_inner()).len()
    }
}

#[async_trait]
impl Outbox for InMemoryOutbox {
    async fn append(
        &self,
        school_id: SchoolId,
        envelope: SerializedEnvelope,
    ) -> Result<()> {
        assert_eq!(
            envelope.school_id, school_id,
            "Outbox::append: school_id argument must match envelope.school_id",
        );
        self.rows
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(envelope);
        Ok(())
    }

    async fn pending(
        &self,
        _school_id: SchoolId,
        _limit: u32,
    ) -> Result<Vec<SerializedEnvelope>> {
        Ok(Vec::new())
    }

    async fn mark_published(
        &self,
        _school_id: SchoolId,
        _ids: &[EventId],
    ) -> Result<()> {
        Ok(())
    }
}

/// In-memory audit log. Records every `append` call so the RBAC
/// tests can assert "no audit row was staged" on a forbidden
/// command.
#[derive(Debug, Default)]
struct InMemoryAuditLog {
    rows: Mutex<Vec<AuditLogEntry>>,
}

impl InMemoryAuditLog {
    fn new() -> Self {
        Self::default()
    }
    fn rows_len(&self) -> usize {
        self.rows.lock().unwrap_or_else(|p| p.into_inner()).len()
    }
}

#[async_trait]
impl AuditLog for InMemoryAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        self.rows
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(entry);
        Ok(())
    }

    async fn read_for_target(
        &self,
        _school_id: SchoolId,
        _target_id: uuid::Uuid,
        _limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        Ok(Vec::new())
    }
}

/// In-memory idempotency store. The dispatcher does not write a
/// record on the RBAC-rejection path, so the mock exists primarily
/// so the transaction's `idempotency()` accessor returns a real
/// trait object.
#[derive(Debug, Default)]
struct InMemoryIdempotency {
    records: Mutex<std::collections::HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
}

impl InMemoryIdempotency {
    fn new() -> Self {
        Self::default()
    }
    fn records_len(&self) -> usize {
        self.records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .len()
    }
}

#[async_trait]
impl educore_storage::idempotency::Idempotency for InMemoryIdempotency {
    async fn lookup(
        &self,
        key: IdempotencyCompositeKey,
    ) -> Result<Option<IdempotencyRecord>> {
        Ok(self
            .records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&key)
            .cloned())
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        self.records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key, record);
        Ok(())
    }

    async fn record_outcome(
        &self,
        record: IdempotencyRecord,
    ) -> Result<IdempotencyOutcome> {
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        let mut store = self.records.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(existing) = store.get(&key) {
            if existing.outcome == record.outcome {
                return Ok(IdempotencyOutcome::Recorded);
            }
            return Ok(IdempotencyOutcome::Conflict {
                existing: existing.clone(),
            });
        }
        store.insert(key, record);
        Ok(IdempotencyOutcome::Recorded)
    }
}

/// Stub event log. The dispatcher does not call `event_log` on the
/// RBAC-rejection path; we still need an impl so the transaction
/// can return `&dyn EventLog`.
#[derive(Debug, Default)]
struct StubEventLog;

#[async_trait]
impl EventLog for StubEventLog {
    async fn append(&self, _entry: EventLogEntry) -> Result<()> {
        Ok(())
    }

    async fn read(
        &self,
        _filter: EventLogFilter,
    ) -> Result<Vec<EventLogEntry>> {
        Ok(Vec::new())
    }

    async fn count(
        &self,
        _filter: EventLogFilter,
    ) -> Result<u64> {
        Ok(0)
    }
}

/// In-memory transaction. Forwards every sub-port to the shared
/// mocks. `commit` and `rollback` are no-ops (the sub-ports write
/// directly to the shared state).
#[derive(Debug)]
struct InMemoryTransaction {
    school_id: SchoolId,
    outbox: Arc<InMemoryOutbox>,
    audit_log: Arc<InMemoryAuditLog>,
    idempotency: Arc<InMemoryIdempotency>,
    event_log: StubEventLog,
}

#[async_trait]
impl Transaction for InMemoryTransaction {
    async fn commit(self: Box<Self>) -> Result<()> {
        Ok(())
    }
    async fn rollback(self: Box<Self>) -> Result<()> {
        Ok(())
    }
    fn outbox(&self) -> &dyn Outbox {
        &*self.outbox
    }
    fn audit_log(&self) -> &dyn AuditLog {
        &*self.audit_log
    }
    fn idempotency(&self) -> &dyn educore_storage::idempotency::Idempotency {
        &*self.idempotency
    }
    fn event_log(&self) -> &dyn EventLog {
        &self.event_log
    }
}

impl TenantTransaction for InMemoryTransaction {
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
}

/// In-memory storage adapter. Hands out a fresh
/// `InMemoryTransaction` per `begin` call.
#[derive(Debug)]
struct InMemoryStorage {
    outbox: Arc<InMemoryOutbox>,
    audit_log: Arc<InMemoryAuditLog>,
    idempotency: Arc<InMemoryIdempotency>,
}

impl InMemoryStorage {
    fn new(
        outbox: Arc<InMemoryOutbox>,
        audit_log: Arc<InMemoryAuditLog>,
        idempotency: Arc<InMemoryIdempotency>,
    ) -> Self {
        Self {
            outbox,
            audit_log,
            idempotency,
        }
    }
}

#[async_trait]
impl StorageAdapter for InMemoryStorage {
    async fn begin(&self) -> Result<Box<dyn Transaction>> {
        Ok(Box::new(InMemoryTransaction {
            school_id: SchoolId(uuid::Uuid::nil()),
            outbox: self.outbox.clone(),
            audit_log: self.audit_log.clone(),
            idempotency: self.idempotency.clone(),
            event_log: StubEventLog,
        }))
    }
    async fn migrate(
        &self,
    ) -> Result<educore_storage::change_stream::MigrationReport> {
        Ok(educore_storage::change_stream::MigrationReport {
            version: 0,
            statements_executed: 0,
            duration: std::time::Duration::from_secs(0),
            already_at_version: true,
        })
    }
    async fn ping(&self) -> Result<()> {
        Ok(())
    }
    async fn close(self: Box<Self>) -> Result<()> {
        Ok(())
    }
}

/// RBAC mock. The dispatcher's `CapabilityCheck` port is
/// string-typed; this mock returns `true` only for the
/// configured allow-list. Tests wire the allow-list to **not**
/// include the capability under test, then assert
/// [`DomainError::Forbidden`].
#[derive(Debug)]
struct DenyAllRbac;

#[async_trait]
impl CapabilityCheck for DenyAllRbac {
    async fn has(
        &self,
        _ctx: &TenantContext,
        _capability: &str,
    ) -> Result<bool> {
        // Every capability is denied. This is the strictest
        // possible RBAC posture and proves the dispatcher
        // short-circuits on the first denied capability.
        Ok(false)
    }
}

// ============================================================================
// Test harness
// ============================================================================

/// Wires a [`CommandDispatcher`] backed by all in-process mocks
/// and an RBAC port that denies every capability. Returns the
/// dispatcher plus the shared mock handles so each test can
/// assert "no side effects occurred" on the forbidden path.
fn build_dispatcher() -> (
    CommandDispatcher,
    Arc<InMemoryBus>,
    Arc<InMemoryOutbox>,
    Arc<InMemoryAuditLog>,
    Arc<InMemoryIdempotency>,
) {
    let outbox = Arc::new(InMemoryOutbox::new());
    let audit_log = Arc::new(InMemoryAuditLog::new());
    let idempotency = Arc::new(InMemoryIdempotency::new());
    let bus = Arc::new(InMemoryBus::new());

    let storage: Arc<dyn StorageAdapter> = Arc::new(InMemoryStorage::new(
        outbox.clone(),
        audit_log.clone(),
        idempotency.clone(),
    ));
    let rbac: Arc<dyn CapabilityCheck> = Arc::new(DenyAllRbac);
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let id_gen: Arc<dyn IdGenerator> = Arc::new(SystemIdGen);

    let dispatcher = CommandDispatcher::new(storage, rbac, bus.clone(), clock, id_gen);
    (dispatcher, bus, outbox, audit_log, idempotency)
}

/// Builds a test command tagged with the domain under test. The
/// `command_type` / `action` / `target_type` values are cosmetic
/// here — the dispatcher's RBAC check happens **before** any
/// command-introspection — but they are realistic so a future
/// audit reading the test output can identify the domain at a
/// glance.
fn make_command(
    school: SchoolId,
    actor: UserId,
    domain: Domain,
) -> TestCommand {
    let (command_type, action, target_type) = match domain {
        Domain::Attendance => ("attendance.staff.mark", "mark", "attendance"),
        Domain::Communication => (
            "communication.notification.read_all",
            "read",
            "notification",
        ),
        Domain::Documents => ("documents.form.read_public", "read", "form"),
        Domain::Academic => ("academic.guardian.attach", "attach", "guardian"),
        Domain::Assessment => ("assessment.exam.setup", "update", "exam"),
        Domain::Finance => (
            "finance.fees_assign.discount.update",
            "update",
            "fees_discount",
        ),
        Domain::Hr => (
            "hr.staff.assign_class_teacher.update",
            "update",
            "class_teacher_assignment",
        ),
    };

    let id_gen = SystemIdGen;
    TestCommand {
        tenant: TenantContext::for_user(
            school,
            actor,
            id_gen.next_correlation_id(),
            UserType::Teacher,
        ),
        command_type,
        idempotency_key: Some(id_gen.next_idempotency_key()),
        action,
        target_type,
    }
}

/// The set of RBAC domains exercised by these tests. Used by
/// [`make_command`] to tag the command with realistic metadata.
#[derive(Debug, Clone, Copy)]
enum Domain {
    Attendance,
    Communication,
    Documents,
    Academic,
    Assessment,
    Finance,
    Hr,
}

/// Service-call closure used by every test. The closure returns
/// the same `(aggregate, event)` pair regardless of which
/// capability is being denied — the closure must never run on the
/// RBAC-rejection path, so its return value is irrelevant to the
/// test's assertions.
fn build_service_call(
    school: SchoolId,
) -> impl FnOnce() -> std::pin::Pin<
    Box<dyn Future<Output = Result<(TestAggregate, TestEvent)>> + Send>,
> {
    let id_gen = SystemIdGen;
    let aggregate_id = id_gen.next_uuid();
    let event_id = id_gen.next_event_id();
    let school_inner = school;
    let occurred_at = Timestamp::now();
    move || {
        Box::pin(async move {
            Ok((
                TestAggregate {
                    id: aggregate_id,
                    name: "test-aggregate".to_owned(),
                },
                TestEvent {
                    event_id,
                    aggregate_id,
                    school_id: school_inner,
                    occurred_at,
                    name: "test-event".to_owned(),
                },
            ))
        })
    }
}

/// Asserts that `result` is a [`DomainError::Forbidden`] carrying
/// the supplied capability string, and that **no** side effects
/// were staged on the storage / bus ports. This is the single
/// invariant the test suite is designed to guard.
fn assert_forbidden_and_clean(
    result: Result<(TestAggregate, TestEvent)>,
    expected_capability: &str,
    outbox: &InMemoryOutbox,
    audit_log: &InMemoryAuditLog,
    idempotency: &InMemoryIdempotency,
    bus: &InMemoryBus,
) {
    match result {
        Err(DomainError::Forbidden(reason)) => {
            assert!(
                reason.contains(expected_capability),
                "Forbidden reason {reason:?} must contain the missing capability \
                 string {expected_capability:?}",
            );
        }
        Err(other) => panic!(
            "expected DomainError::Forbidden, got {other:?} \
             (capability {expected_capability})",
        ),
        Ok(_) => panic!(
            "expected DomainError::Forbidden, got Ok (capability {expected_capability})",
        ),
    }
    assert_eq!(
        outbox.rows_len(),
        0,
        "no outbox rows must be staged on the RBAC-rejection path \
         (capability {expected_capability})",
    );
    assert_eq!(
        audit_log.rows_len(),
        0,
        "no audit rows must be staged on the RBAC-rejection path \
         (capability {expected_capability})",
    );
    assert_eq!(
        idempotency.records_len(),
        0,
        "no idempotency records must be staged on the RBAC-rejection path \
         (capability {expected_capability})",
    );
    assert_eq!(
        bus.published_len(),
        0,
        "no events must be published on the RBAC-rejection path \
         (capability {expected_capability})",
    );
}

// ============================================================================
// Tests — one per RBAC domain
// ============================================================================

/// **Attendance**: a user without `Attendance.Staff.Create` cannot
/// dispatch a mark-attendance command. The capability string
/// matches the form `educore_rbac::value_objects::Capability::
/// AttendanceStaffCreate.as_str()`.
#[tokio::test]
async fn attendance_staff_mark_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Attendance);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_ATTENDANCE_MARK], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_ATTENDANCE_MARK,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **Communication**: a user without
/// `Communication.Notification.Read.All` cannot dispatch a
/// read-all-notifications command.
#[tokio::test]
async fn communication_notification_read_all_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Communication);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_COMMUNICATION_NOTIFICATION_READ_ALL], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_COMMUNICATION_NOTIFICATION_READ_ALL,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **Documents**: a user without `Documents.Form.Read.Public`
/// cannot dispatch a read-public-form command.
#[tokio::test]
async fn documents_form_read_public_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Documents);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_DOCUMENTS_FORM_READ_PUBLIC], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_DOCUMENTS_FORM_READ_PUBLIC,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **Academic**: a user without `Academic.Guardian` cannot
/// dispatch an attach-guardian command.
#[tokio::test]
async fn academic_guardian_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Academic);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_ACADEMIC_GUARDIAN], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_ACADEMIC_GUARDIAN,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **Assessment**: a user without `Assessment.Exam.Setup` cannot
/// dispatch an exam-setup command.
#[tokio::test]
async fn assessment_exam_setup_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Assessment);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_ASSESSMENT_EXAM_SETUP], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_ASSESSMENT_EXAM_SETUP,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **Finance**: a user without
/// `Finance.Fees.Assign.Discount.Update` cannot dispatch a
/// discount-update command.
#[tokio::test]
async fn finance_fees_assign_discount_update_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Finance);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

/// **HR**: a user without `Hr.Staff.AssignClassTeacher.Update`
/// cannot dispatch an assign-class-teacher command.
#[tokio::test]
async fn hr_staff_assign_class_teacher_update_without_capability_is_forbidden() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Hr);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_HR_STAFF_ASSIGN_CLASS_TEACHER_UPDATE], || {
            build_service_call(school)()
        })
        .await;

    assert_forbidden_and_clean(
        result,
        CAP_HR_STAFF_ASSIGN_CLASS_TEACHER_UPDATE,
        &outbox,
        &audit_log,
        &idempotency,
        &bus,
    );
}

// ============================================================================
// Bonus coverage — same-shape invariants the suite should keep
// guarding even when a domain's RBAC catalogue shifts.
// ============================================================================

/// Multi-capability commands reject on the **first** denied
/// capability and never reach the service call. The dispatcher
/// iterates `required_capabilities` in order; this test wires
/// two capabilities, denies both, and asserts the rejected
/// reason names the first capability (proving short-circuit).
#[tokio::test]
async fn first_denied_capability_short_circuits_before_second_check() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Academic);

    // Both capabilities are denied; the first one must surface.
    let result = dispatcher
        .dispatch(
            &cmd,
            &[CAP_ACADEMIC_GUARDIAN, CAP_ASSESSMENT_EXAM_SETUP],
            || build_service_call(school)(),
        )
        .await;

    match result {
        Err(DomainError::Forbidden(reason)) => {
            assert!(
                reason.contains(CAP_ACADEMIC_GUARDIAN),
                "Forbidden reason {reason:?} must name the FIRST denied capability \
                 ({CAP_ACADEMIC_GUARDIAN:?}), not the second",
            );
        }
        Err(other) => panic!("expected Forbidden, got {other:?}"),
        Ok(_) => panic!("expected Forbidden, got Ok"),
    }
    // Clean-shutdown invariants still hold.
    assert_eq!(outbox.rows_len(), 0);
    assert_eq!(audit_log.rows_len(), 0);
    assert_eq!(idempotency.records_len(), 0);
    assert_eq!(bus.published_len(), 0);
}

/// Regression guard: an empty `required_capabilities` slice is
/// legal — the dispatcher must skip the RBAC step entirely and
/// proceed to the service call (the engine lets the service
/// decide whether a capability-free command is allowed; the
/// dispatcher itself does not refuse). This protects against a
/// future "always require at least one capability" change that
/// would break the CMS / Settings / Operations domains whose
/// commands legitimately do not require RBAC checks.
#[tokio::test]
async fn empty_required_capabilities_proceeds_to_service_call() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, bus, outbox, audit_log, idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Academic);

    let result = dispatcher
        .dispatch(&cmd, &[], || build_service_call(school)())
        .await;

    // The service call must have produced an event and the
    // dispatcher must have staged an outbox row + audit row +
    // bus publish. (The in-memory mocks let us assert side
    // effects directly.)
    assert!(
        result.is_ok(),
        "empty required_capabilities must let the command proceed, got {result:?}",
    );
    assert_eq!(
        outbox.rows_len(),
        1,
        "one outbox row must be staged when RBAC is skipped",
    );
    assert_eq!(
        audit_log.rows_len(),
        1,
        "one audit row must be staged when RBAC is skipped",
    );
    assert_eq!(
        idempotency.records_len(),
        1,
        "one idempotency record must be staged (the test command \
         always carries an idempotency key) when RBAC is skipped",
    );
    assert_eq!(
        bus.published_len(),
        1,
        "exactly one event must be published post-commit when \
         RBAC is skipped",
    );
}

/// The forbidden reason must include the exact capability string
/// the dispatcher denied. This is a contract surface for the
/// security-monitor pipeline (it parses the reason to attribute
/// the rejection to a missing capability). Asserting on the
/// reason guards against future refactors that switch to a
/// structured `Forbidden { capability: Capability }` enum but
/// forget to preserve the string-form reason.
#[tokio::test]
async fn forbidden_reason_includes_missing_capability_string() {
    let id_gen = SystemIdGen;
    let school = id_gen.next_school_id();
    let actor = id_gen.next_user_id();

    let (dispatcher, _bus, _outbox, _audit_log, _idempotency) = build_dispatcher();
    let cmd = make_command(school, actor, Domain::Finance);

    let result = dispatcher
        .dispatch(&cmd, &[CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE], || {
            build_service_call(school)()
        })
        .await;

    match result {
        Err(DomainError::Forbidden(reason)) => {
            assert!(
                reason.contains(CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE),
                "Forbidden reason {reason:?} must contain the missing \
                 capability string {CAP_FINANCE_FEES_ASSIGN_DISCOUNT_UPDATE:?}",
            );
            // The dispatcher wraps the missing capability in a
            // "missing capability ..." prefix; sanity-check the
            // prefix is present so a future refactor that drops
            // it does not silently change the wire format.
            assert!(
                reason.contains("missing capability"),
                "Forbidden reason {reason:?} must start with the \
                 'missing capability' prefix",
            );
        }
        Err(other) => panic!("expected Forbidden, got {other:?}"),
        Ok(_) => panic!("expected Forbidden, got Ok"),
    }
}
