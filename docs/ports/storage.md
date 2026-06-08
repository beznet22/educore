# Storage Port

## Purpose

The storage port defines the contract that consumer-supplied storage
adapters must implement. The engine never writes directly to a database.
All persistence flows through the storage port, which provides:

- A handle to a transactional unit of work.
- A repository per aggregate root.
- Tenant isolation enforcement.
- Connection lifecycle management.
- Cross-adapter parity guarantees.

## Trait: `StorageAdapter`

```rust
#[async_trait]
pub trait StorageAdapter: Send + Sync + std::fmt::Debug {
    async fn begin(&self) -> Result<Transaction>;
    async fn migrate(&self) -> Result<MigrationReport>;
    async fn ping(&self) -> Result<()>;
    async fn close(&self) -> Result<()>;

    // Repository handles — one per aggregate.
    fn students(&self) -> Arc<dyn StudentRepository>;
    fn guardians(&self) -> Arc<dyn GuardianRepository>;
    fn classes(&self) -> Arc<dyn ClassRepository>;
    fn sections(&self) -> Arc<dyn SectionRepository>;
    fn class_sections(&self) -> Arc<dyn ClassSectionRepository>;
    fn subjects(&self) -> Arc<dyn SubjectRepository>;
    fn class_subjects(&self) -> Arc<dyn ClassSubjectRepository>;
    fn academic_years(&self) -> Arc<dyn AcademicYearRepository>;
    fn class_routines(&self) -> Arc<dyn ClassRoutineRepository>;
    fn homeworks(&self) -> Arc<dyn HomeworkRepository>;
    fn lessons(&self) -> Arc<dyn LessonRepository>;
    fn lesson_topics(&self) -> Arc<dyn LessonTopicRepository>;
    fn lesson_plans(&self) -> Arc<dyn LessonPlanRepository>;
    fn student_records(&self) -> Arc<dyn StudentRecordRepository>;
    fn student_promotions(&self) -> Arc<dyn StudentPromotionRepository>;
    fn student_categories(&self) -> Arc<dyn StudentCategoryRepository>;
    fn student_groups(&self) -> Arc<dyn StudentGroupRepository>;
    fn registration_fields(&self) -> Arc<dyn RegistrationFieldRepository>;
    fn certificates(&self) -> Arc<dyn CertificateRepository>;
    fn id_cards(&self) -> Arc<dyn IdCardRepository>;
    fn admission_queries(&self) -> Arc<dyn AdmissionQueryRepository>;
    fn class_rooms(&self) -> Arc<dyn ClassRoomRepository>;
    fn class_times(&self) -> Arc<dyn ClassTimeRepository>;

    // ... one handle per aggregate, across all 15 domains (~80+ total)
}
```

The trait is object-safe. Consumers typically use it as
`Arc<dyn StorageAdapter>`.

## Trait: `Transaction`

```rust
#[async_trait]
pub trait Transaction: Send + Sync {
    async fn commit(self: Box<Self>) -> Result<()>;
    async fn rollback(self: Box<Self>) -> Result<()>;
    fn repositories(&self) -> &dyn TransactionalRepositories;
    fn outbox(&self) -> &dyn Outbox;
}
```

A `Transaction` is acquired via `StorageAdapter::begin()`. Repositories
obtained from a transaction are bound to that transaction. Reads see
writes from the same transaction. On `commit` the writes are persisted
and the outbox events are released to the event bus. On `rollback` the
writes are discarded and the outbox is cleared.

## Tenant Isolation

The storage adapter is responsible for **enforcing tenant isolation**.
The engine always passes a `SchoolId` filter; the adapter MUST add a
`school_id = $1` predicate to every read query. Recommended defenses:

- Row-level security policies in the database (`CREATE POLICY ... USING
  (school_id = current_setting('app.school_id')::int)`).
- Wrapper functions on top of raw SQL that always inject the predicate.
- A test suite that attempts to read across tenants and fails.

The engine refuses to issue a query that lacks a tenant filter. The
typed query layer ensures this at compile time.

## Outbox

The outbox is a transactional event publication mechanism. Within a
transaction, the engine writes events to the outbox table. On commit,
the outbox relay publishes them to the event bus. On rollback, they are
discarded.

```rust
#[async_trait]
pub trait Outbox: Send + Sync {
    async fn append(&self, envelope: EventEnvelope) -> Result<()>;
    async fn pending(&self, limit: u32) -> Result<Vec<EventEnvelope>>;
    async fn mark_published(&self, ids: &[EventId]) -> Result<()>;
}
```

The outbox is part of the same transaction as the domain state change,
guaranteeing atomicity.

## Migrations

Migrations are owned by the consumer, not the engine. The engine
exposes `StorageAdapter::migrate()` so consumers can invoke their
migration runner programmatically. The engine does not prescribe a
migration tool. Consumers may use `refinery`, `sqlx-migrate`, `diesel`,
or hand-rolled SQL.

The engine documentation, however, lists the **expected schema** for
each domain in `docs/specs/<domain>/repositories.md` (tables, columns,
indexes). Consumers are responsible for translating these into
concrete migrations.

## Connection Pooling

The adapter owns the connection pool. The engine does not pool
connections itself. The adapter's `begin()` blocks until a connection
is available. Timeouts are configurable per adapter.

## Read Replicas

The adapter may use a read replica for read-only queries. Consistency
guarantees are adapter-defined:

- `ReadYourWrites`: After a write commits, subsequent reads see it.
- `Eventual`: Reads may lag behind writes. The engine does not assume
  read-after-write consistency by default. Commands that require it
  perform a read inside the same transaction.

## Streaming

For large result sets (e.g. attendance over a year), the adapter may
expose a streaming method:

```rust
#[async_trait]
pub trait StudentRepository: Send + Sync {
    async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>;
}
```

Streaming avoids loading millions of rows into memory.

## Error Type

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("connection failed: {0}")] Connection(String),
    #[error("transaction conflict: {0}")] Conflict(String),
    #[error("deadlock detected")] Deadlock,
    #[error("unique violation: {0}")] UniqueViolation { constraint: String },
    #[error("foreign key violation: {0}")] ForeignKey { constraint: String },
    #[error("check constraint violation: {0}")] Check { constraint: String },
    #[error("row not found")] NotFound,
    #[error("infrastructure error: {0}")] Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("timeout")] Timeout,
    #[error("serialization failure")] SerializationFailure,
}
```

The engine maps `StorageError::Infrastructure` to
`DomainError::Infrastructure` and logs the source via `tracing`. The
other variants are translated to domain errors (`Conflict`,
`NotFound`, etc.).

## Cross-Adapter Parity

The engine ships reference adapters for:

- PostgreSQL (primary target)
- SQLite (embedded deployments, including single-process offline mode)
- SurrealDB (consumer-supplied, experimental)
- MongoDB (consumer-supplied, document adapter)

Adapters must satisfy the **parity test suite** in
`crates/smscore-storage-parity` to be considered compliant. The suite
exercises every repository method against a seeded database and
verifies identical results across adapters.

## Configuration

The adapter is constructed by the consumer with their own
configuration. Example:

```rust
let storage: Arc<dyn StorageAdapter> = Arc::new(
    PostgresStorage::builder()
        .url(env::var("DATABASE_URL")?)
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .statement_cache_capacity(128)
        .build()
        .await?
);
```

The engine does not prescribe configuration names.

## Object Safety

`StorageAdapter` and all repository traits are object-safe. A consumer
may store `Arc<dyn StorageAdapter>` and use it across threads.

## Worked Example

A consumer wires the storage adapter into the engine:

```rust
let storage: Arc<dyn StorageAdapter> = Arc::new(
    PostgresStorage::connect("postgres://app:secret@db/smscore").await?
);

let engine = Engine::builder()
    .storage(storage.clone())
    .auth(auth_provider)
    .notify(notify_provider)
    .event_bus(InProcessBus::new())
    .build()
    .await?;
```

A command runs in a transaction:

```rust
let mut tx = storage.begin().await?;
let student = tx.students().get(student_id).await?
    .ok_or(DomainError::NotFound { entity: "Student", id: student_id.into() })?;
let updated = student.with_profile(new_profile);
tx.students().update(&updated).await?;
tx.outbox().append(StudentProfileUpdated { ... }.into()).await?;
tx.commit().await?;
```

## Testing

The port requires:

- Unit tests of every repository method.
- Integration tests against a real database (testcontainers).
- A parity test verifying identical behavior across adapters.
- A tenancy test verifying cross-tenant reads are blocked.
- A failure-injection test (e.g. deadlock retry, connection drop).
- A load test (10k attendance marks in <5s).

## Offline Mode

When the consumer runs in offline mode, the storage adapter may be
configured to use a local SQLite database. The outbox accumulates events
in the local database and is replayed to the central store when
connectivity is restored. The storage port does not change; only the
adapter implementation differs.

## Audit

The storage port is not directly responsible for audit logging. The
audit sink is a separate port (`AuditSink`) and the engine writes audit
records through it inside each command's transaction.
