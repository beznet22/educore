# Wave 5 — Documentation Audit (Docs Group 4)

**Scope:** `docs/ports/*.md` (8 port contracts), `docs/commands/*.md` (15 command catalogs), `docs/events/*.md` (15 event catalogs).

**Audit date:** 2026-06-23.

**Checks performed:**
1. Port contract vs implementation drift — does the trait method / struct / enum exist in the corresponding port crate?
2. Command catalog vs implementation — is the typed command struct defined in `crates/domains/<d>/src/commands.rs`?
3. Event catalog vs implementation — is the typed event struct defined in `crates/domains/<d>/src/events.rs`?
4. Wire-form consistency — do doc-event names match the `EVENT_TYPE` constants the events emit?

**Notes on scope:**
- `docs/ports/` contains 8 files; the prompt's stated scope of 7 port contracts is treated as approximate. All 8 are audited.
- "Documented" counts come from extracting unique `| \`XxxYyy\`` rows from each markdown table. "Implemented" counts come from `grep -E "^pub struct XxxYyyCommand" crates/domains/<d>/src/commands.rs` (or `pub struct XxxYyy` for events).

---

### FINDING 1

- **id:** DOC-PORT-001
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/storage.md:214-229` (StorageError enum) vs `crates/infra/core/src/error.rs:19-63` (DomainError enum)
- **description:** The storage port spec defines a dedicated `StorageError` enum with 9 variants (`Connection`, `Conflict`, `Deadlock`, `UniqueViolation`, `ForeignKey`, `Check`, `NotFound`, `Infrastructure`, `Timeout`, `SerializationFailure`). The engine actually has a single engine-wide `DomainError` enum with 7 variants (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `TenantViolation`, `Infrastructure`, `NotSupported`); there is no `StorageError` type anywhere in the workspace. The doc explicitly states the engine maps `StorageError::Infrastructure` to `DomainError::Infrastructure` and translates other variants to domain errors — i.e. the spec describes an adapter-facing error type that does not exist.
- **expected:** `StorageError` enum per `docs/ports/storage.md:218-229` with the 10 variants above.
- **evidence:**
  - `docs/ports/storage.md:218-229` — `pub enum StorageError { #[error("connection failed: {0}")] Connection(String), #[error("transaction conflict: {0}")] Conflict(String), #[error("deadlock detected")] Deadlock, #[error("unique violation: {0}")] UniqueViolation { constraint: String }, #[error("foreign key violation: {0}")] ForeignKey { constraint: String }, #[error("check constraint violation: {0}")] Check { constraint: String }, #[error("row not found")] NotFound, #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), #[error("timeout")] Timeout, #[error("serialization failure")] SerializationFailure, }`
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), NotSupported(String), }`
  - No `StorageError` symbol exists: `rg "pub enum StorageError|pub struct StorageError" crates/` returns 0 matches.

---

### FINDING 2

- **id:** DOC-PORT-002
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/storage.md:17-89` (StorageAdapter trait) vs `crates/infra/storage/src/port.rs:34-150` (StorageAdapter trait)
- **description:** The storage port spec shows the `StorageAdapter` trait carrying ~24 `fn xxx_repository(&self) -> Arc<dyn XxxRepository>` accessors (students, guardians, classes, sections, class_sections, subjects, class_subjects, academic_years, class_routines, homeworks, lessons, lesson_topics, lesson_plans, student_records, student_promotions, student_categories, student_groups, registration_fields, certificates, id_cards, admission_queries, class_rooms, class_times) plus the comment "…one handle per aggregate, across all 15 domains (~80+ total)". The actual `StorageAdapter` trait carries no repository accessors — only `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. Per the impl's source comment, the trait is intentionally minimal at Phase 0 and a generic `Repository<A>` (not 80+ per-aggregate traits) is the actual surface.
- **expected:** `StorageAdapter` exposes `students() -> Arc<dyn StudentRepository>`, `guardians() -> Arc<dyn GuardianRepository>`, `classes() -> Arc<dyn ClassRepository>`, `sections() -> Arc<dyn SectionRepository>`, …, `class_times() -> Arc<dyn ClassTimeRepository>` and one handle per aggregate (~80+ total).
- **evidence:**
  - `docs/ports/storage.md:25-50` — `fn students(&self) -> Arc<dyn StudentRepository>; fn guardians(&self) -> Arc<dyn GuardianRepository>; fn classes(&self) -> Arc<dyn ClassRepository>; fn sections(&self) -> Arc<dyn SectionRepository>; fn class_sections(&self) -> Arc<dyn ClassSectionRepository>; fn subjects(&self) -> Arc<dyn SubjectRepository>; fn class_subjects(&self) -> Arc<dyn ClassSubjectRepository>; fn academic_years(&self) -> Arc<dyn AcademicYearRepository>; fn class_routines(&self) -> Arc<dyn ClassRoutineRepository>; fn homeworks(&self) -> Arc<dyn HomeworkRepository>; fn lessons(&self) -> Arc<dyn LessonRepository>; fn lesson_topics(&self) -> Arc<dyn LessonTopicRepository>; fn lesson_plans(&self) -> Arc<dyn LessonPlanRepository>; fn student_records(&self) -> Arc<dyn StudentRecordRepository>; fn student_promotions(&self) -> Arc<dyn StudentPromotionRepository>; fn student_categories(&self) -> Arc<dyn StudentCategoryRepository>; fn student_groups(&self) -> Arc<dyn StudentGroupRepository>; fn registration_fields(&self) -> Arc<dyn RegistrationFieldRepository>; fn certificates(&self) -> Arc<dyn CertificateRepository>; fn id_cards(&self) -> Arc<dyn IdCardRepository>; fn admission_queries(&self) -> Arc<dyn AdmissionQueryRepository>; fn class_rooms(&self) -> Arc<dyn ClassRoomRepository>; fn class_times(&self) -> Arc<dyn ClassTimeRepository>; // ... one handle per aggregate, across all 15 domains (~80+ total)`
  - `crates/infra/storage/src/port.rs:34-150` — trait body lists only `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`; `fn students` / `fn guardians` / `fn classes` / etc. do not exist on this trait.
  - `crates/infra/storage/src/repository.rs:25-72` — single generic `pub trait Repository<A>: Send + Sync where A: Send + Sync + Clone + 'static { async fn get(...); async fn get_including_retired(...); async fn list(...); async fn count(...); async fn insert(...); async fn update(...); async fn soft_delete(...); }`; no `StudentRepository`, `GuardianRepository`, etc.

---

### FINDING 3

- **id:** DOC-PORT-003
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/storage.md:159-166` (Outbox trait) vs `crates/infra/storage/src/outbox.rs:89-124` (Outbox trait)
- **description:** The doc spec defines the `Outbox` sub-port as `append(envelope: EventEnvelope) -> Result<()>`, `pending(limit: u32) -> Result<Vec<EventEnvelope>>`, and `mark_published(ids: &[EventId]) -> Result<()>`. The actual `Outbox` trait in `crates/infra/storage/src/outbox.rs` accepts a `SerializedEnvelope` (a concrete, deserialize-owned row type), not `EventEnvelope`; the `EventEnvelope` is the bus-port type from `crates/cross-cutting/events/src/envelope.rs`. The impl also adds a 4th method `pending_count(school_id)` (defaulted) that is absent from the spec.
- **expected:** Per `docs/ports/storage.md:162-164`, `Outbox::append(&self, envelope: EventEnvelope)` and `Outbox::pending(&self, limit: u32) -> Result<Vec<EventEnvelope>>`.
- **evidence:**
  - `docs/ports/storage.md:159-166` — `pub trait Outbox: Send + Sync { async fn append(&self, envelope: EventEnvelope) -> Result<()>; async fn pending(&self, limit: u32) -> Result<Vec<EventEnvelope>>; async fn mark_published(&self, ids: &[EventId]) -> Result<()>; }`
  - `crates/infra/storage/src/outbox.rs:89-124` — `async fn append(&self, envelope: SerializedEnvelope) -> Result<()>;` (line 102), `async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>>;` (line 108), `async fn mark_published(&self, ids: &[EventId]) -> Result<()>;` (line 112), plus `async fn pending_count(&self, school_id: SchoolId) -> Result<u64>` (line 117).

---

### FINDING 4

- **id:** DOC-PORT-004
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/storage.md:120-130` (Transaction trait) vs `crates/infra/storage/src/transaction.rs:32-92` (Transaction trait)
- **description:** The doc spec defines `Transaction` as carrying `commit`, `rollback`, `repositories() -> &dyn TransactionalRepositories`, and `outbox() -> &dyn Outbox`. The actual `Transaction` trait carries `commit`, `rollback`, `outbox()`, `audit_log() -> &dyn AuditLog`, `idempotency() -> &dyn Idempotency`, `event_log() -> &dyn EventLog`, and `bulk_insert_student_attendances(...)`. There is no `repositories()` accessor and no `TransactionalRepositories` type anywhere in the workspace; in their place are three other sub-port accessors plus a bulk-insert convenience method.
- **expected:** `Transaction` exposes `commit`, `rollback`, `repositories() -> &dyn TransactionalRepositories`, `outbox() -> &dyn Outbox`.
- **evidence:**
  - `docs/ports/storage.md:122-130` — `pub trait Transaction: Send + Sync { async fn commit(self: Box<Self>) -> Result<()>; async fn rollback(self: Box<Self>) -> Result<()>; fn repositories(&self) -> &dyn TransactionalRepositories; fn outbox(&self) -> &dyn Outbox; }`
  - `crates/infra/storage/src/transaction.rs:32-92` — `async fn commit(...)` (line 43), `async fn rollback(...)` (line 47), `fn outbox(&self) -> &dyn Outbox;` (line 51), `fn audit_log(&self) -> &dyn AuditLog;` (line 55), `fn idempotency(&self) -> &dyn Idempotency;` (line 60), `fn event_log(&self) -> &dyn EventLog;` (line 64), `async fn bulk_insert_student_attendances(...)` (line 86).
  - `rg "TransactionalRepositories" crates/` returns 0 matches.

---

### FINDING 5

- **id:** DOC-PORT-005
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/storage.md:204-212` (StudentRepository stream method) vs `crates/infra/storage/src/repository.rs:25-72` (Repository<A> trait)
- **description:** The doc spec defines a `StudentRepository` trait with at least one method `stream(q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>`. The actual storage port does not expose per-aggregate repository traits and does not declare a `stream` method anywhere; the generic `Repository<A>` trait exposes `get`, `get_including_retired`, `list`, `count`, `insert`, `update`, `soft_delete`. Streaming is not part of the trait surface.
- **expected:** `pub trait StudentRepository: Send + Sync { async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>; }`
- **evidence:**
  - `docs/ports/storage.md:206-210` — `pub trait StudentRepository: Send + Sync { async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>; }`
  - `crates/infra/storage/src/repository.rs:25-72` — generic `Repository<A>` carries `get`, `get_including_retired`, `list`, `count`, `insert`, `update`, `soft_delete`; no `stream` method, no `StudentRepository` type, no `BoxStream` import.
  - `rg "pub trait StudentRepository|fn stream\(.*Query" crates/` returns 0 matches.

---

### FINDING 6

- **id:** DOC-PORT-006
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/sync.md:36-46` (SyncAdapter trait) vs `crates/cross-cutting/sync/src/port.rs:37-75` (SyncAdapter trait)
- **description:** The sync port spec defines `SyncAdapter` with four methods `dispatch`, `subscribe`, `snapshot`, `health`, taking `CommandEnvelope`, `EventFilter`, `SchoolId`, and `&self` respectively. The actual `SyncAdapter` trait carries a completely different five-method set: `start(school: SchoolId)`, `pause(school: SchoolId)`, `resume(school: SchoolId)`, `stop(school: SchoolId)`, and `health()`. None of `dispatch`, `subscribe`, `snapshot` exist on the trait; none of `start`, `pause`, `resume`, `stop` exist on the trait as documented. The supporting types `CommandEnvelope`, `EventFilter` (sync variant), `SchoolSnapshot`, and `EventStream` documented in the spec are also absent from the implementation.
- **expected:** Per `docs/ports/sync.md:38-46`, `SyncAdapter` exposes `dispatch`, `subscribe`, `snapshot`, `health`.
- **evidence:**
  - `docs/ports/sync.md:38-46` — `pub trait SyncAdapter: Send + Sync + std::fmt::Debug { async fn dispatch(&self, envelope: CommandEnvelope) -> Result<CommandOutcome>; async fn subscribe(&self, filter: EventFilter) -> Result<EventStream>; async fn snapshot(&self, school_id: SchoolId) -> Result<SchoolSnapshot>; async fn health(&self) -> Result<SyncHealth>; }`
  - `crates/cross-cutting/sync/src/port.rs:36-75` — `pub trait SyncAdapter: Send + Sync { async fn start(&self, school: SchoolId) -> Result<()>; async fn pause(&self, school: SchoolId) -> Result<()>; async fn resume(&self, school: SchoolId) -> Result<()>; async fn stop(&self, school: SchoolId) -> Result<()>; async fn health(&self) -> Result<SyncHealth>; }`
  - `crates/cross-cutting/sync/src/command.rs:25-39` defines a separate `SyncCommand` enum with `Start`, `Pause`, `Resume`, `Stop` variants; no `CommandEnvelope`, no `CommandOutcome`, no `SchoolSnapshot`, no `EventStream` types exist (`rg "pub struct CommandEnvelope|pub struct SchoolSnapshot|pub trait EventStream" crates/` returns 0 matches).

---

### FINDING 7

- **id:** DOC-PORT-007
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/sync.md:210-224` (SyncHealth struct) vs `crates/cross-cutting/sync/src/health.rs:67-75` (SyncHealth struct)
- **description:** The sync port spec defines `SyncHealth` with fields `reachable: bool`, `latency_ms: u32`, `server_version: &'static str`, and `server_schema_version: u32`. The actual `SyncHealth` struct in `crates/cross-cutting/sync/src/health.rs` has fields `status: SyncStatus` (an enum with `Running`/`Paused`/`Stopped` variants) and `last_event_at: Option<Timestamp>`. The two structs have no fields in common.
- **expected:** Per `docs/ports/sync.md:212-218`, `SyncHealth { reachable: bool, latency_ms: u32, server_version: &'static str, server_schema_version: u32 }`.
- **evidence:**
  - `docs/ports/sync.md:212-218` — `pub struct SyncHealth { pub reachable: bool, pub latency_ms: u32, pub server_version: &'static str, pub server_schema_version: u32, }`
  - `crates/cross-cutting/sync/src/health.rs:67-75` — `pub struct SyncHealth { pub status: SyncStatus, pub last_event_at: Option<Timestamp>, }` and `pub enum SyncStatus { Running, Paused, Stopped, }` at lines 23-34.

---

### FINDING 8

- **id:** DOC-PORT-008
- **area:** documentation
- **severity:** Critical
- **location:** `docs/ports/authentication.md:88-99` (Engine.rbac method, RbacPort trait) vs `crates/tools/sdk/src/engine.rs` (Engine struct) vs `crates/adapters/auth/src/port.rs:306-326` (RbacPort trait)
- **description:** The authentication port spec shows `impl Engine { pub fn rbac(&self) -> &dyn RbacPort { &*self.rbac_port } }` and defines the `RbacPort` trait in the engine's main surface. The actual `Engine` struct (`crates/tools/sdk/src/engine.rs:21-40`) does not carry a `rbac_port` field and does not expose a `rbac()` method; the available accessors are `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`. The `RbacPort` trait does exist in `crates/adapters/auth/src/port.rs:306-326` but is not wired into the engine.
- **expected:** Per `docs/ports/authentication.md:89-92`, `Engine` carries a `rbac_port: Arc<dyn RbacPort>` field and exposes `pub fn rbac(&self) -> &dyn RbacPort`.
- **evidence:**
  - `docs/ports/authentication.md:89-92` — `impl Engine { pub fn rbac(&self) -> &dyn RbacPort { &*self.rbac_port } }`
  - `crates/tools/sdk/src/engine.rs:20-40` — `pub struct Engine { storage: Arc<dyn StorageAdapter>, auth: Arc<dyn AuthProvider>, notify: Arc<dyn NotificationProvider>, payment: Arc<dyn PaymentProvider>, files: Arc<dyn FileStorage>, integrations: Arc<dyn IntegrationGateway>, bus: Arc<dyn EventBus>, clock: Arc<dyn Clock>, id_gen: Arc<dyn IdGenerator>, }`; no `rbac_port` field.
  - `crates/tools/sdk/src/engine.rs:48-146` lists only `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `rbac()` method.
  - `crates/adapters/auth/src/port.rs:306-326` defines the `RbacPort` trait but nothing references it from the SDK engine.

---

### FINDING 9

- **id:** DOC-PORT-009
- **area:** documentation
- **severity:** High
- **location:** `docs/ports/event-bus.md:183-192` (EventBusError enum) vs `crates/cross-cutting/events/src/errors.rs:18-45` (EventError enum)
- **description:** The doc spec defines `EventBusError` with 5 variants: `TopicNotFound`, `SubscriptionClosed`, `PublishFailed`, `DeserializeFailed`, `Infrastructure`. The actual `EventError` enum (the type the bus port actually returns, per `crates/cross-cutting/events/src/event_bus.rs`) has 6 variants: `TopicNotFound`, `SubscriptionClosed`, `PublishFailed`, `DeserializeFailed`, `NotSupported`, `Infrastructure`. The `NotSupported` variant is undocumented in the spec; the doc's named enum `EventBusError` does not exist in the codebase.
- **expected:** `EventBusError` enum per `docs/ports/event-bus.md:185-191` with 5 variants.
- **evidence:**
  - `docs/ports/event-bus.md:184-191` — `pub enum EventBusError { #[error("topic not found: {0}")] TopicNotFound(Topic), #[error("subscription closed")] SubscriptionClosed, #[error("publish failed: {0}")] PublishFailed(String), #[error("deserialize failed: {0}")] DeserializeFailed(String), #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `crates/cross-cutting/events/src/errors.rs:18-45` — `pub enum EventError { TopicNotFound(Topic), SubscriptionClosed, PublishFailed(String), DeserializeFailed(String), NotSupported(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `rg "pub enum EventBusError" crates/` returns 0 matches.

---

### FINDING 10

- **id:** DOC-PORT-010
- **area:** documentation
- **severity:** High
- **location:** `docs/ports/event-bus.md:84-90` (EventSubscription trait) vs `crates/cross-cutting/events/src/event_bus.rs:69-85` (EventSubscription trait)
- **description:** The doc spec defines `EventSubscription::ack(&mut self, event_id: EventId) -> Result<()>` and `EventSubscription::nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>`. The actual trait returns `Result<AckOutcome>` (an enum with `Accepted` / `Unknown` / `Failed` variants) for both methods. The return type is a structural drift.
- **expected:** Per `docs/ports/event-bus.md:87-88`, `ack` returns `Result<()>`, `nack` returns `Result<()>`.
- **evidence:**
  - `docs/ports/event-bus.md:84-90` — `pub trait EventSubscription: Send + Sync { async fn next(&mut self) -> Option<Result<EventEnvelope>>; async fn ack(&mut self, event_id: EventId) -> Result<()>; async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<()>; async fn close(self: Box<Self>) -> Result<()>; }`
  - `crates/cross-cutting/events/src/event_bus.rs:69-85` — `pub trait EventSubscription: Send + Sync { async fn next(&mut self) -> Option<Result<EventEnvelope>>; async fn ack(&mut self, event_id: EventId) -> Result<AckOutcome>; async fn nack(&mut self, event_id: EventId, requeue: bool) -> Result<AckOutcome>; async fn close(self: Box<Self>) -> Result<()>; }` plus `pub enum AckOutcome { Accepted, Unknown, Failed, }` at lines 53-61.

---

### FINDING 11

- **id:** DOC-PORT-011
- **area:** documentation
- **severity:** High
- **location:** `docs/ports/notifications.md:196-206` (NotificationError::Infrastructure) vs `crates/adapters/notify/src/errors.rs:75-120` (NotificationError)
- **description:** The doc spec defines `NotificationError::Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` carrying a boxed error source. The actual `NotificationError::Infrastructure(String)` carries a pre-rendered string (the source error's `Display` output); the variant does not derive `std::error::Error::source` because the boxed source has been collapsed to a string at construction time. The impl's variant order also differs (`MissingVariable` precedes `InvalidRecipient` per the spec but follows it in the impl, and the impl has no `RbacPort` variant order issue; the spec lists `TemplateNotFound`, `MissingVariable`, `InvalidRecipient`, `RateLimited`, `Provider`, `QuotaExceeded`, `Infrastructure`).
- **expected:** `NotificationError::Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>)` per `docs/ports/notifications.md:205`.
- **evidence:**
  - `docs/ports/notifications.md:197-206` — `pub enum NotificationError { #[error("template not found: {0}")] TemplateNotFound(NotificationTemplateId), #[error("missing variable: {0}")] MissingVariable(String), #[error("invalid recipient: {0}")] InvalidRecipient(String), #[error("rate limited")] RateLimited, #[error("provider error: {0}")] Provider(String), #[error("quota exceeded")] QuotaExceeded, #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), }`
  - `crates/adapters/notify/src/errors.rs:75-120` — `pub enum NotificationError { TemplateNotFound(NotificationTemplateId), MissingVariable(String), InvalidRecipient(String), RateLimited, Provider(String), QuotaExceeded, Infrastructure(String), }` (line 118 shows `Infrastructure(String)` with no `#[source]` attribute).
  - The file's doc comment at lines 71-74 explicitly notes the deviation: "The engine never stores a live source error chain across a port boundary — it logs the source via `tracing` immediately and serialises only the string representation, so the `Infrastructure` variant is itself a `String` (not a `Box<dyn Error>`)."

---

### FINDING 12

- **id:** DOC-PORT-012
- **area:** documentation
- **severity:** High
- **location:** `docs/ports/authentication.md:13-20` (AuthProvider return type) vs `crates/adapters/auth/src/port.rs:259-289` (AuthProvider trait)
- **description:** The doc spec writes `AuthProvider`'s `authenticate`, `validate`, `refresh` methods as returning `Result<Session>` (unqualified), with `Result` implicitly being `core::result::Result<T, std::io::Error>` or some other default. The actual trait returns `Result<Session, crate::errors::AuthError>` (and `Result<(), AuthError>` for `revoke`). The unqualified `Result` in the doc is the bus-port `educore_core::error::Result<T>` alias (which is `Result<T, DomainError>`), not the spec's implied error type. A consumer following the doc would compile mismatched error mappings.
- **expected:** `async fn authenticate(&self, credential: Credential) -> Result<Session>` with a clearly specified error type (the doc does not name one but the surrounding `AuthError` enum implies `AuthError`).
- **evidence:**
  - `docs/ports/authentication.md:13-19` — `pub trait AuthProvider: Send + Sync + std::fmt::Debug { async fn authenticate(&self, credential: Credential) -> Result<Session>; async fn validate(&self, token: &AuthToken) -> Result<Session>; async fn revoke(&self, token: &AuthToken) -> Result<()>; async fn refresh(&self, token: &AuthToken) -> Result<Session>; }`
  - `crates/adapters/auth/src/port.rs:259-289` — `pub trait AuthProvider: Send + Sync + std::fmt::Debug { async fn authenticate(&self, credential: Credential) -> Result<Session, crate::errors::AuthError>; async fn validate(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>; async fn revoke(&self, token: &AuthToken) -> Result<(), crate::errors::AuthError>; async fn refresh(&self, token: &AuthToken) -> Result<Session, crate::errors::AuthError>; }` — every method carries an explicit `AuthError` error parameter.

---

### FINDING 13

- **id:** DOC-PORT-013
- **area:** documentation
- **severity:** Medium
- **location:** `docs/ports/event-bus.md:199,219` (`engine.events()`) vs `crates/tools/sdk/src/engine.rs:107-109` (`engine.bus()`)
- **description:** The event-bus port doc uses `engine.events().subscribe(SubscribeOptions { ... })` as the canonical worked example. The actual `Engine` struct exposes `bus() -> &Arc<dyn EventBus>` (singular noun), not `events()`. Consumers following the doc would receive a compile error.
- **expected:** `engine.events().subscribe(...)` per `docs/ports/event-bus.md:199` and `:219`.
- **evidence:**
  - `docs/ports/event-bus.md:199` — `let mut sub = engine.events().subscribe(SubscribeOptions {`
  - `docs/ports/event-bus.md:219` — `engine.events().subscribe(SubscribeOptions {`
  - `crates/tools/sdk/src/engine.rs:105-109` — `pub fn bus(&self) -> &Arc<dyn EventBus> { &self.bus }`
  - `rg "pub fn events" crates/` returns 0 matches.

---

### FINDING 14

- **id:** DOC-PORT-014
- **area:** documentation
- **severity:** Medium
- **location:** `docs/ports/payments.md:202` (`engine.payments()`) vs `crates/tools/sdk/src/engine.rs:87-91` (`engine.payment()`)
- **description:** The payment port doc uses `engine.payments().charge(ChargeRequest { ... })` as the canonical worked example. The actual `Engine` struct exposes `payment() -> &Arc<dyn PaymentProvider>` (singular noun), not `payments()`. Same drift as finding 13 — the accessor name does not match the doc.
- **expected:** `engine.payments().charge(ChargeRequest { ... })` per `docs/ports/payments.md:202`.
- **evidence:**
  - `docs/ports/payments.md:202` — `let receipt = engine.payments().charge(ChargeRequest {`
  - `crates/tools/sdk/src/engine.rs:87-91` — `pub fn payment(&self) -> &Arc<dyn PaymentProvider> { &self.payment }`
  - `rg "pub fn payments" crates/` returns 0 matches.

---

### FINDING 15

- **id:** DOC-PORT-015
- **area:** documentation
- **severity:** Medium
- **location:** `docs/ports/payments.md:216` (`engine.fees().record_payment`) and `docs/ports/file-storage.md:171` (`engine.students().admit`)
- **description:** The payment port doc shows `engine.fees().record_payment(RecordPaymentCommand { ... })` and the file-storage port doc shows `engine.students().admit(AdmitStudentCommand { ... })`. Neither `engine.fees()` nor `engine.students()` accessors exist in the SDK `Engine` struct or anywhere else in the workspace. Domain facades (`admission`, `attendance`, `payment_svc`, `notify_svc` in `crates/tools/sdk/src/engine.rs:125-145`) are exposed as separate service handles, not as `engine.<domain>()`.
- **expected:** `engine.fees().record_payment(...)` and `engine.students().admit(...)`.
- **evidence:**
  - `docs/ports/payments.md:216` — `engine.fees().record_payment(RecordPaymentCommand {`
  - `docs/ports/file-storage.md:171` — `let student = engine.students().admit(AdmitStudentCommand {`
  - `crates/tools/sdk/src/engine.rs:48-146` lists `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `fees()`, no `students()`.
  - `rg "pub fn fees\b|pub fn students\b" crates/` returns 0 matches.

---

### FINDING 16

- **id:** DOC-PORT-016
- **area:** documentation
- **severity:** Low
- **location:** `docs/ports/storage.md:438-454` (worked example) vs `crates/tools/sdk/src/engine.rs:194-282` (EngineBuilder)
- **description:** The storage port doc's worked example shows a fully wired engine that does not exist in the SDK crate. `Engine::builder()` is referenced (line 447), but the actual builder is `EngineBuilder::new()` (`crates/tools/sdk/src/engine.rs:179`). The builder ports in the doc's example (`storage`, `auth`, `notify`, `event_bus`) do not exhaustively match the actual builder's required ports (which also requires `payment`, `files`, `integrations`, `clock`, `id_gen` per `crates/tools/sdk/src/engine.rs:258-281`). The doc's example would fail to compile if transcribed verbatim.
- **expected:** The worked example should produce a compilable engine.
- **evidence:**
  - `docs/ports/storage.md:447-454` — `let engine = Engine::builder() .storage(storage.clone()) .auth(auth_provider) .notify(notify_provider) .event_bus(InProcessBus::new()) .build() .await?;`
  - `crates/tools/sdk/src/engine.rs:179-281` — `pub fn new() -> Self` and `pub fn build(self) -> Result<Engine, SdkError>` requiring every port: `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen` (`let storage = self.storage.ok_or(SdkError::MissingPort("storage"))?; … let id_gen = self.id_gen.ok_or(SdkError::MissingPort("id_gen"))?;` at lines 259-269).
  - `rg "Engine::builder|fn builder\b" crates/` returns 0 matches — the API is `EngineBuilder::new()`, not `Engine::builder()`.

---

### FINDING 17

- **id:** DOC-CAT-001
- **area:** documentation
- **severity:** High
- **location:** `docs/commands/academic.md:17-92` (table) vs `crates/domains/academic/src/commands.rs:64-461`
- **description:** The academic command catalog documents 73 distinct command names (e.g. `AddStudentToGroup`, `AddSubTopicToLessonPlan`, `AssignClassRoom`, `AssignClassTeacher`, `AssignOptionalSubject`, `AssignStudentToSection`, `AssignSubjectTeacher`, `AssignSubjectToClass`, `CancelHomework`, `ChangeStudentCategory`, `ConvertAdmissionQuery`, `CreateCertificate`, `CreateClassRoutine`, `CreateClassSection`, `CreateHomework`, `CreateIdCard`, `CreateLesson`, `CreateLessonPlan`, `CreateLessonTopic`, `CreateRegistrationField`, `CreateStudentCategory`, `CreateStudentGroup`, `DeleteCertificate`, `DeleteClassRoutine`, `DeleteClassSection`, `DeleteIdCard`, `DeleteLesson`, `DeleteLessonPlan`, `DeleteLessonTopic`, `DeleteRegistrationField`, `DeleteStudentCategory`, `DeleteStudentGroup`, `EvaluateHomework`, `FollowUpAdmissionQuery`, `MarkLessonPlanCompleted`, `MarkLessonTopicCompleted`, `ReassignSubjectTeacher`, `RegisterAdmissionQuery`, `RemoveStudentFromGroup`, `SubmitHomework`, `SwapClassRoutinePeriods`, `UnassignSubjectFromClass`, `UpdateCertificate`, `UpdateClassRoutinePeriod`, `UpdateHomework`, `UpdateIdCard`, `UpdateLesson`, `UpdateLessonPlan`, `UpdateRegistrationField`, `UpdateStudentCategory`, `UpdateStudentGroup`, `UploadStudentDocument`). The Phase-3 implementation only ships 22 command structs (`AdmitStudent`, `CloseAcademicYear`, `CreateAcademicYear`, `CreateClass`, `CreateSection`, `CreateSubject`, `DeleteClass`, `DeleteSection`, `DeleteSubject`, `GraduateStudent`, `PromoteStudent`, `ReinstateStudent`, `SetCurrentAcademicYear`, `SetOptionalSubjectGpaThreshold`, `SuspendStudent`, `TransferStudent`, `UpdateAcademicYearDates`, `UpdateClass`, `UpdateSection`, `UpdateStudentProfile`, `UpdateSubject`, `WithdrawStudent`). 51 catalog commands have no `*Command` struct in the crate.
- **expected:** Per `docs/commands/academic.md:17-92`, 73 academic command structs in `crates/domains/academic/src/commands.rs`.
- **evidence:**
  - `docs/commands/academic.md:17-92` enumerates 73 `| \`XxxYyy\`` rows; awk-extracted unique command names = 73 (`awk -F"\`" '$2 ~ /^[A-Z][a-zA-Z]+$/ && $4 ~ /^[A-Z][a-zA-Z]+\.[A-Z]/ {print $2}' docs/commands/academic.md | sort -u | wc -l` → 73).
  - `crates/domains/academic/src/commands.rs:64-461` defines 22 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/academic/src/commands.rs | wc -l` → 22).
  - `crates/domains/academic/src/lib.rs:7-15` and `:24` acknowledge the partial scope: "Phase 3 delivers the **prompt-named subset only**" and "The remaining 27 academic aggregates … land in later phases" — but `docs/commands/academic.md` is presented as the complete catalog for the domain, not a Phase 3 subset.

---

### FINDING 18

- **id:** DOC-CAT-002
- **area:** documentation
- **severity:** High
- **location:** `docs/events/academic.md:10-96` (table) vs `crates/domains/academic/src/events.rs:53-1439`
- **description:** The academic event catalog documents 85 distinct event names (e.g. `StudentAssignedToSection`, `StudentCategoryChanged`, `OptionalSubjectAssigned`, `StudentDocumentUploaded`, `GuardianRegistered`, `GuardianContactUpdated`, `GuardianLinkedToStudent`, `GuardianUnlinkedFromStudent`, `PrimaryGuardianMarked`, `ClassSectionCreated`, `ClassTeacherAssigned`, `SubjectTeacherAssigned`, `ClassRoomAssigned`, `ClassSectionDeleted`, `SubjectAssignedToClass`, `TeacherReassigned`, `SubjectUnassigned`, `ClassRoutineCreated`, `ClassRoutinePeriodUpdated`, `ClassRoutinePeriodsSwapped`, `ClassRoutineDeleted`, `HomeworkCreated`, `HomeworkUpdated`, `HomeworkSubmitted`, `HomeworkEvaluated`, `HomeworkCancelled`, `LessonCreated`, `LessonUpdated`, `LessonDeleted`, `LessonTopicCreated`, `LessonTopicCompleted`, `LessonTopicDeleted`, `LessonPlanCreated`, `LessonPlanUpdated`, `LessonPlanCompleted`, `SubTopicAdded`, `LessonPlanDeleted`, `StudentRecordCreated`, `RollNumberAssigned`, `DefaultRecordSet`, `StudentMarkedGraduate`, `StudentCategoryCreated`, `StudentCategoryUpdated`, `StudentCategoryDeleted`, `StudentGroupCreated`, `StudentGroupUpdated`, `StudentAddedToGroup`, `StudentRemovedFromGroup`, `StudentGroupDeleted`, `RegistrationFieldCreated`, `RegistrationFieldUpdated`, `RegistrationFieldDeleted`, `CertificateCreated`, `CertificateUpdated`, `CertificateDeleted`, `IdCardCreated`, `IdCardUpdated`, `IdCardDeleted`, `AdmissionQueryRegistered`, `AdmissionQueryFollowedUp`, `AdmissionQueryConverted`, `AdmissionQueryClosed`). The Phase-3 implementation ships 22 event structs (`StudentAdmitted`, `StudentProfileUpdated`, `StudentSuspended`, `StudentReinstated`, `StudentWithdrawn`, `StudentTransferred`, `StudentPromoted`, `StudentGraduated`, `ClassCreated`, `ClassUpdated`, `OptionalSubjectGpaThresholdSet`, `ClassDeleted`, `SectionCreated`, `SectionUpdated`, `SectionDeleted`, `SubjectCreated`, `SubjectUpdated`, `SubjectDeleted`, `AcademicYearCreated`, `AcademicYearDatesUpdated`, `CurrentAcademicYearSet`, `AcademicYearClosed`, `AcademicYearCopied`). 63 catalog events have no struct.
- **expected:** Per `docs/events/academic.md:10-96`, 85 academic event structs in `crates/domains/academic/src/events.rs`.
- **evidence:**
  - `docs/events/academic.md:10-96` enumerates 85 `| \`XxxYyy\`` rows (`awk -F"\`" '$2 ~ /^[A-Z][a-zA-Z]+$/ && $4 ~ /^[A-Z][a-zA-Z]+$/ {print $2}' docs/events/academic.md | sort -u | wc -l` → 85).
  - `crates/domains/academic/src/events.rs:53-1439` defines 22 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/academic/src/events.rs | wc -l` → 22 events; only 23 incl. `AcademicYearCopied`).
  - The crate's own `lib.rs:7-15` notes the partial scope, but the doc is presented as the full domain event catalog.

---

### FINDING 19

- **id:** DOC-CAT-003
- **area:** documentation
- **severity:** High
- **location:** `docs/events/finance.md` (table) vs `crates/domains/finance/src/events.rs:40-701`
- **description:** The finance event catalog documents 179 distinct event names. The finance crate's events module only ships 10 event structs (`WalletCreated`, `WalletCredited`, `WalletDebited`, `WalletRefundRequested`, `WalletTransactionApproved`, `WalletTransactionRejected`, `InvoiceNumberingConfigured`, `ExpenseRecorded`, `PaymentReceived`, `PayrollPaymentRecorded`). The crate's own `lib.rs` declares "Workstream A ships the 5 headline events for `Wallet` + `WalletTransaction`" but the catalog covers 179. The catalog includes large event families (`FeesGroup*`, `FeesType*`, `FeesMaster*`, `FeesAssign*`, `FeesDiscount*`, `FeesInstallment*`, `DirectFeesInstallment*`, `DirectFeesReminder*`, `DirectFeesSetting*`, `BankAccount*`, `BankStatement*`, `BankPaymentSlip*`, `ChartOfAccount*`, `Donor*`, `IncomeHead*`, `ExpenseHead*`, `Inventory*`, `Product*`, `Payroll*`, `WalletTransaction*`, `Wallet*`, etc.) that have no event struct in the crate.
- **expected:** Per `docs/events/finance.md`, 179 finance event structs in `crates/domains/finance/src/events.rs`.
- **evidence:**
  - `docs/events/finance.md` enumerates 179 `| \`XxxYyy\`` rows (`grep -E "^\| \`[A-Z]" docs/events/finance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 179).
  - `crates/domains/finance/src/events.rs` defines 10 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/finance/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 10).
  - `crates/domains/finance/src/events.rs:11-15` confirms partial scope: "Workstream A ships the 5 headline events for `Wallet` + `WalletTransaction` … + `FeesInvoiceConfigured` … + `ExpenseRecorded`."

---

### FINDING 20

- **id:** DOC-CAT-004
- **area:** documentation
- **severity:** High
- **location:** `docs/commands/assessment.md` (table) vs `crates/domains/assessment/src/commands.rs`
- **description:** The assessment command catalog documents 128 distinct command names (a sample: `AddOnlineExamQuestion`, `AddQuestionOption`, `AddTeacherRemark`, `AdmitCardGenerated`, `AdmitCardSettingUpdated`, `ApproveTeacherEvaluation`, `ConfigureAdmitCardSettings`, `ConfigureCustomResultSettings`, `ConfigureSeatPlanSettings`, `ConfigureTeacherEvaluation`, `CreateExamSetting`, `CreateExamType`, `CreateMarksGrade`, `CreateOnlineExam`, `CreateQuestion`, `CreateQuestionGroup`, `CreateQuestionLevel`, `CustomResultSettingUpdated`, `DeleteExamSetting`, `DeleteExamType`, …). The assessment crate's commands module only ships 19 command structs. 109 catalog commands have no struct.
- **expected:** Per `docs/commands/assessment.md`, 128 assessment command structs in `crates/domains/assessment/src/commands.rs`.
- **evidence:**
  - `docs/commands/assessment.md` enumerates 128 command rows (`grep -E "^\| \`[A-Z]" docs/commands/assessment.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 128).
  - `crates/domains/assessment/src/commands.rs` defines 19 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/assessment/src/commands.rs | wc -l` → 19).

---

### FINDING 21

- **id:** DOC-CAT-005
- **area:** documentation
- **severity:** High
- **location:** `docs/events/assessment.md` (table) vs `crates/domains/assessment/src/events.rs`
- **description:** The assessment event catalog documents 114 distinct event names (sample: `AdmitCard`, `AdmitCardSetting`, `AdmitCardSettingUpdated`, `CustomResultSetting`, `CustomResultSettingUpdated`, `Exam`, `ExamAttendance`, `ExamAttendanceMarked`, `ExamAttendanceUpdated`, `ExamRoutinePage`, `ExamRoutinePageUpdated`, `ExamSchedule`, `ExamSetting`, `ExamSettingCreated`, `ExamSettingDeleted`, `ExamSettingUpdated`, `ExamSetup`, `ExamSetupCreated`, `ExamSetupDeleted`, `ExamSetupUpdated`, …). The assessment crate's events module only ships 21 event structs. 93 catalog events have no struct.
- **expected:** Per `docs/events/assessment.md`, 114 assessment event structs in `crates/domains/assessment/src/events.rs`.
- **evidence:**
  - `docs/events/assessment.md` enumerates 114 event rows (`grep -E "^\| \`[A-Z]" docs/events/assessment.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 114).
  - `crates/domains/assessment/src/events.rs` defines 21 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/assessment/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 21).

---

### FINDING 22

- **id:** DOC-CAT-006
- **area:** documentation
- **severity:** High
- **location:** `docs/commands/cms.md` (table) vs `crates/domains/cms/src/commands.rs`
- **description:** The CMS command catalog documents 129 distinct command names. The CMS crate's commands module ships only 10 command structs (`CreatePage`, `PublishPage`, `ArchivePage`, `DeletePage`, `CreateNews`, `CreateTestimonial`, `CreateHomeSlider`, `CreateContent`, `CreateContentShareList`, `ConfigureHomePage`). 119 catalog commands have no struct, despite the Phase 12 handoff language in `AGENTS.md` claiming the CMS domain is "spec-faithful".
- **expected:** Per `docs/commands/cms.md`, 129 CMS command structs in `crates/domains/cms/src/commands.rs`.
- **evidence:**
  - `docs/commands/cms.md` enumerates 129 command rows (`grep -E "^\| \`[A-Z]" docs/commands/cms.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 129).
  - `crates/domains/cms/src/commands.rs` defines 10 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/cms/src/commands.rs | wc -l` → 10).
  - `AGENTS.md` line for educore-cms claims "20 root aggregates … ~67 events, ~67 commands, 86 Cms caps", but only 10 commands exist on disk.

---

### FINDING 23

- **id:** DOC-CAT-007
- **area:** documentation
- **severity:** High
- **location:** `docs/events/cms.md` (table) vs `crates/domains/cms/src/events.rs`
- **description:** The CMS event catalog documents 85 distinct event names. The CMS crate's events module ships 67 event structs. 18 catalog events have no struct.
- **expected:** Per `docs/events/cms.md`, 85 CMS event structs in `crates/domains/cms/src/events.rs`.
- **evidence:**
  - `docs/events/cms.md` enumerates 85 event rows (`grep -E "^\| \`[A-Z]" docs/events/cms.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 85).
  - `crates/domains/cms/src/events.rs` defines 67 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/cms/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 67).
  - `crates/domains/cms/src/lib.rs:45-60` re-exports 67 events by name; the missing 18 events are catalog rows that have no corresponding export.

---

### FINDING 24

- **id:** DOC-CAT-008
- **area:** documentation
- **severity:** High
- **location:** `docs/commands/hr.md` (table) vs `crates/domains/hr/src/commands.rs`
- **description:** The HR command catalog documents 122 distinct command names. The HR crate's commands module ships only 21 command structs. 101 catalog commands have no struct.
- **expected:** Per `docs/commands/hr.md`, 122 HR command structs in `crates/domains/hr/src/commands.rs`.
- **evidence:**
  - `docs/commands/hr.md` enumerates 122 command rows (`grep -E "^\| \`[A-Z]" docs/commands/hr.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 122).
  - `crates/domains/hr/src/commands.rs` defines 21 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/hr/src/commands.rs | wc -l` → 21).

---

### FINDING 25

- **id:** DOC-CAT-009
- **area:** documentation
- **severity:** High
- **location:** `docs/events/hr.md` (table) vs `crates/domains/hr/src/events.rs`
- **description:** The HR event catalog documents 78 distinct event names. The HR crate's events module ships 46 event structs. 32 catalog events have no struct.
- **expected:** Per `docs/events/hr.md`, 78 HR event structs in `crates/domains/hr/src/events.rs`.
- **evidence:**
  - `docs/events/hr.md` enumerates 78 event rows (`grep -E "^\| \`[A-Z]" docs/events/hr.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 78).
  - `crates/domains/hr/src/events.rs` defines 46 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/hr/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 46).

---

### FINDING 26

- **id:** DOC-CAT-010
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/facilities.md` (table) vs `crates/domains/facilities/src/commands.rs`
- **description:** The facilities command catalog documents 100 distinct command names. The facilities crate's commands module ships 49 command structs. 51 catalog commands have no struct.
- **expected:** Per `docs/commands/facilities.md`, 100 facilities command structs in `crates/domains/facilities/src/commands.rs`.
- **evidence:**
  - `docs/commands/facilities.md` enumerates 100 command rows (`grep -E "^\| \`[A-Z]" docs/commands/facilities.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 100).
  - `crates/domains/facilities/src/commands.rs` defines 49 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/facilities/src/commands.rs | wc -l` → 49).

---

### FINDING 27

- **id:** DOC-CAT-011
- **area:** documentation
- **severity:** Medium
- **location:** `docs/events/facilities.md` (table) vs `crates/domains/facilities/src/events.rs`
- **description:** The facilities event catalog documents 63 distinct event names. The facilities crate's events module ships 23 event structs. 40 catalog events have no struct.
- **expected:** Per `docs/events/facilities.md`, 63 facilities event structs in `crates/domains/facilities/src/events.rs`.
- **evidence:**
  - `docs/events/facilities.md` enumerates 63 event rows (`grep -E "^\| \`[A-Z]" docs/events/facilities.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 63).
  - `crates/domains/facilities/src/events.rs` defines 23 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/facilities/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 23).

---

### FINDING 28

- **id:** DOC-CAT-012
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/communication.md` (table) vs `crates/domains/communication/src/commands.rs`
- **description:** The communication command catalog documents 54 distinct command names. The communication crate's commands module ships 72 command structs. 18 commands exist on disk that are not enumerated in the catalog (i.e. implementation outpaces documentation). Sample additional commands: `CreateBulkNotificationTemplate`, `UpdateBulkNotificationTemplate`, `DeleteBulkNotificationTemplate`, `CreateWhatsAppTemplate`, `UpdateWhatsAppTemplate`, `DeleteWhatsAppTemplate`, `SendBulkNotification`, `SendBulkEmail`, `SendBulkSms`, `SendBulkPush`, `SendBulkChat`, `SendBulkVoice`, `SendBulkWebhook`, `SendBulkInApp`.
- **expected:** Per `docs/commands/communication.md`, 54 communication command structs; the actual count is 72.
- **evidence:**
  - `docs/commands/communication.md` enumerates 54 command rows (`grep -E "^\| \`[A-Z]" docs/commands/communication.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 54).
  - `crates/domains/communication/src/commands.rs` defines 72 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/communication/src/commands.rs | wc -l` → 72).
  - `comm -13 <(sort -u <(grep -E "^\| \`[A-Z]" docs/commands/communication.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`")) <(grep -oE "struct [A-Z][a-zA-Z]+Command" crates/domains/communication/src/commands.rs | awk '{print $2}' | sed 's/Command$//' | sort -u) | head` returns 18 struct names not present in the catalog.

---

### FINDING 29

- **id:** DOC-CAT-013
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/library.md` (table) vs `crates/domains/library/src/commands.rs`
- **description:** The library command catalog documents 37 distinct command names. The library crate's commands module ships 22 command structs. 15 catalog commands have no struct.
- **expected:** Per `docs/commands/library.md`, 37 library command structs in `crates/domains/library/src/commands.rs`.
- **evidence:**
  - `docs/commands/library.md` enumerates 37 command rows (`grep -E "^\| \`[A-Z]" docs/commands/library.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 37).
  - `crates/domains/library/src/commands.rs` defines 22 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/library/src/commands.rs | wc -l` → 22).

---

### FINDING 30

- **id:** DOC-CAT-014
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/attendance.md` (table) vs `crates/domains/attendance/src/commands.rs`
- **description:** The attendance command catalog documents 28 distinct command names. The attendance crate's commands module ships 14 command structs. 14 catalog commands have no struct.
- **expected:** Per `docs/commands/attendance.md`, 28 attendance command structs in `crates/domains/attendance/src/commands.rs`.
- **evidence:**
  - `docs/commands/attendance.md` enumerates 28 command rows (`grep -E "^\| \`[A-Z]" docs/commands/attendance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 28).
  - `crates/domains/attendance/src/commands.rs` defines 14 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/attendance/src/commands.rs | wc -l` → 14).

---

### FINDING 31

- **id:** DOC-CAT-015
- **area:** documentation
- **severity:** Medium
- **location:** `docs/events/attendance.md` (table) vs `crates/domains/attendance/src/events.rs`
- **description:** The attendance event catalog documents 24 distinct event names. The attendance crate's events module ships 21 event structs. 3 catalog events have no struct.
- **expected:** Per `docs/events/attendance.md`, 24 attendance event structs in `crates/domains/attendance/src/events.rs`.
- **evidence:**
  - `docs/events/attendance.md` enumerates 24 event rows (`grep -E "^\| \`[A-Z]" docs/events/attendance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 24).
  - `crates/domains/attendance/src/events.rs` defines 21 `pub struct *Event` types (`grep -oE "pub struct [A-Z][a-zA-Z]+\b" crates/domains/attendance/src/events.rs | grep -oE "[A-Z][a-zA-Z]+$" | sort -u | wc -l` → 21).

---

### FINDING 32

- **id:** DOC-CAT-016
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/documents.md` (table) vs `crates/domains/documents/src/commands.rs`
- **description:** The documents command catalog documents 19 distinct command names. The documents crate's commands module ships 10 command structs. 9 catalog commands have no struct.
- **expected:** Per `docs/commands/documents.md`, 19 documents command structs in `crates/domains/documents/src/commands.rs`.
- **evidence:**
  - `docs/commands/documents.md` enumerates 19 command rows (`grep -E "^\| \`[A-Z]" docs/commands/documents.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 19).
  - `crates/domains/documents/src/commands.rs` defines 10 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/documents/src/commands.rs | wc -l` → 10).

---

### FINDING 33

- **id:** DOC-CAT-017
- **area:** documentation
- **severity:** Medium
- **location:** `docs/commands/platform.md` (table) vs `crates/cross-cutting/platform/src` (no `commands.rs`)
- **description:** The platform command catalog documents a non-trivial number of commands but the `educore-platform` crate (cross-cutting tier) does not contain a `commands.rs` module. Platform commands are routed through the `events-domain` calendar crate or the `platform` value-objects/ids module; no `*Command` structs exist in `crates/cross-cutting/platform/`. This is mentioned because the prompt scope covers the docs; the cross-cutting `platform` commands live in a different location than the doc implies.
- **expected:** Platform command structs in `crates/cross-cutting/platform/src/commands.rs` (or in a clearly cross-referenced location).
- **evidence:**
  - `docs/commands/platform.md` exists and is in scope (`ls docs/commands/`).
  - `ls crates/cross-cutting/platform/src/` shows only the platform module's value-objects / id / tenant modules; no `commands.rs` (`find crates/cross-cutting/platform -name commands.rs` returns nothing).
  - The docs/ports/platform.md or specs/platform/commands.md should be cross-referenced from `docs/commands/platform.md` to direct readers to the actual location.

---

### FINDING 34

- **id:** DOC-CAT-018
- **area:** documentation
- **severity:** Low
- **location:** `crates/domains/academic/src/events.rs:116,170,233,292,355,419,502,564,626,688,742,792,850,908,958,1029,1096,1146,1221,1279,1334,1384,1440` vs `docs/events/academic.md:12,13,14,…,96`
- **description:** Wire-form consistency spot check on the academic events that ARE implemented. The doc uses the PascalCase type name in column 1 (`StudentAdmitted`), while the actual `EVENT_TYPE` constants on the `DomainEvent` impl use the dotted lowercase string per the bus-port contract (e.g. `academic.student.admitted`). The bus-port spec at `docs/ports/event-bus.md:27` states the `event_type` is `&'static str` and the convention is `<domain>.<aggregate>.<verb>`; the doc table column header does not distinguish between the type name and the wire event_type, which is a documentation precision gap rather than a true wire-form drift.
- **expected:** Each catalog row should make the event type name and the wire event_type string explicit (e.g. separate columns or a sidebar block).
- **evidence:**
  - `docs/events/academic.md:12` — `| \`StudentAdmitted\` | \`Student\` | … |`
  - `crates/domains/academic/src/events.rs:115-118` — `impl DomainEvent for StudentAdmitted { const EVENT_TYPE: &'static str = "academic.student.admitted"; const SCHEMA_VERSION: u32 = 1; const AGGREGATE_TYPE: &'static str = "student"; }`
  - The doc column at `docs/events/academic.md:10` header reads `| Event | Aggregate | Subscribers | Description | Durable? | Replicated? | Replayable? |` — no `event_type` column.

---

### FINDING 35

- **id:** DOC-CAT-019
- **area:** documentation
- **severity:** Low
- **location:** `docs/commands/finance.md:14-96` (table) vs `crates/domains/finance/src/commands.rs:64+` (118 `*Command` structs)
- **description:** The finance command catalog documents 79 distinct command names; the finance crate's commands module actually ships 118 `*Command` structs. The implementation has 39 commands not enumerated in the catalog (sample: `ApproveBankSlip`, `ApproveExpense`, `ApproveIncome`, `ApprovePayroll`, `ApprovePayrollPayment`, `ApproveWalletTransaction`, `BlockLoginForDueFees`, `CancelInvoice`, `CarryForwardFeesBalance`, `ConfigureDueFeesBlockSetting`, `ConfigureFeesCarryForward`, `ConfigureFeesGroup`, `ConfigureFeesType`, `CreateAmountTransfer`, `CreateDirectFeesInstallment`, `CreateDirectFeesInstallmentAssign`, `CreateDirectFeesReminder`, `CreateDirectFeesSetting`, `CreateExpenseHead`, `CreateFeesAssign`, `CreateFeesDiscount`, `CreateFeesGroup`, `CreateFeesInstallment`, `CreateFeesMaster`, `CreateFeesType`, `CreateIncome`, `CreateIncomeHead`, `CreatePaymentGateway`, `CreatePaymentMethod`, `DeleteBankAccount`, `DeleteDirectFeesInstallment`, `DeleteDirectFeesInstallmentAssign`, `DeleteDirectFeesReminder`, `DeleteDirectFeesSetting`, `DeleteExpense`, `DeleteExpenseHead`, `DeleteFeesAssign`, `DeleteFeesDiscount`, `DeleteFeesGroup`, `DeleteFeesInstallment`). Like finding 28 for communication, the implementation outpaces the doc here.
- **expected:** Catalog covers every implemented command.
- **evidence:**
  - `docs/commands/finance.md` enumerates 79 command rows (`grep -E "^\| \`[A-Z]" docs/commands/finance.md | grep -oE "\`[A-Z][a-zA-Z]+\`" | tr -d "\`" | sort -u | wc -l` → 79).
  - `crates/domains/finance/src/commands.rs` defines 118 `pub struct *Command` types (`grep -E "^pub struct \w+Command" crates/domains/finance/src/commands.rs | wc -l` → 118).
  - The catalog/doc was rebuilt during Phase 7 spec cleanup; the implementations kept growing after the snapshot.

---

### END FINDINGS

Total findings: **35**

Counts by severity:
- Critical: 8 (DOC-PORT-001, DOC-PORT-002, DOC-PORT-003, DOC-PORT-004, DOC-PORT-005, DOC-PORT-006, DOC-PORT-007, DOC-PORT-008)
- High: 13 (DOC-PORT-009, DOC-PORT-010, DOC-PORT-011, DOC-PORT-012, DOC-CAT-001, DOC-CAT-002, DOC-CAT-003, DOC-CAT-004, DOC-CAT-005, DOC-CAT-006, DOC-CAT-007, DOC-CAT-008, DOC-CAT-009)
- Medium: 11 (DOC-PORT-013, DOC-PORT-014, DOC-PORT-015, DOC-CAT-010, DOC-CAT-011, DOC-CAT-012, DOC-CAT-013, DOC-CAT-014, DOC-CAT-015, DOC-CAT-016, DOC-CAT-017)
- Low: 3 (DOC-PORT-016, DOC-CAT-018, DOC-CAT-019)

Counts by area:
- Port contracts: 16 findings (DOC-PORT-001 through DOC-PORT-016)
- Command catalogs: 11 findings (DOC-CAT-001, DOC-CAT-004, DOC-CAT-006, DOC-CAT-008, DOC-CAT-010, DOC-CAT-012, DOC-CAT-013, DOC-CAT-014, DOC-CAT-016, DOC-CAT-017, DOC-CAT-019)
- Event catalogs: 8 findings (DOC-CAT-002, DOC-CAT-003, DOC-CAT-005, DOC-CAT-007, DOC-CAT-009, DOC-CAT-011, DOC-CAT-015, DOC-CAT-018)
