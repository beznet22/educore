## Wave 4 Foundation Audit Report — `educore-storage` (Port Trait)

**Scope:** `crates/infra/storage/` — the port-trait crate
(`StorageAdapter`, `Transaction`, `Outbox`, `AuditLog`, `Idempotency`,
`EventLog`, `Repository`, `ChangeStream`, change-stream wire types,
`StudentAttendanceRow`). Cross-referenced against
`docs/ports/storage.md`, `docs/schemas/{database,audit,event,command,tenancy}-schema.md`,
`migrations/engine/0000_engine_core.postgres.sql`, and the AGENTS.md
engine rules.

**Total findings:** 36

---

### FINDING 1

- **id:** PORT-STORE-001
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/port.rs:30-33`
- **description:** The `StorageAdapter` trait exposes `migrate()` as the schema-emission entry point, but every consumer-facing doc (`AGENTS.md:544, 561`, `docs/build-plan.md:175-179`, `docs/architecture.md:322`, `docs/schemas/sql-dialects/README.md:193-198`, `migrations/engine/README.md:11`) refers to `storage.create_schema().await` as the runtime DDL entry. The method does not exist on the trait; all four adapters (`storage-postgres`, `storage-mysql`, `storage-sqlite`, `storage-surrealdb`) ship a `migrate()` that the docs then rename to `create_schema()` at the consumer boundary. The port name and the consumer name are different for the same operation.
- **expected:** `docs/build-plan.md:175-179` lists the trait surface as `("create_schema", "apply_command", "query", "begin_tx", …)`. `docs/architecture.md:322` states "the schema is emitted at runtime via `storage.create_schema().await`". The trait is the canonical contract for that method.
- **evidence:**
  ```rust
  crates/infra/storage/src/port.rs:30-33
  /// Applies the engine's DDL to bring the schema up to the
  /// engine's current version. Idempotent: running on an
  /// already-migrated database returns a no-op report.
  async fn migrate(&self) -> Result<MigrationReport>;
  ```
  `grep -rn "fn create_schema" crates/infra/storage/src/` returns no results.

---

### FINDING 2

- **id:** PORT-STORE-002
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/transaction.rs:51-75` (entire trait body)
- **description:** The `Transaction` trait carries no `school_id` / `TenantContext` field. The sub-port handles `outbox()`, `audit_log()`, `idempotency()`, and `event_log()` are bare `&dyn Trait` references with no tenant anchor on the trait surface. Per `docs/schemas/tenancy-schema.md` § 4 the storage adapter MUST "reject writes whose `school_id` does not match the caller's `TenantContext::school_id`" — but the trait has no way to receive or expose the active `TenantContext`, so the adapter must hold it in a thread-local or a sibling field. The `bulk_insert_student_attendances` doc-comment at `port.rs:64-66` and `transaction.rs:88-90` says "the row's `school_id` MUST equal the transaction's scoped school (enforced by the adapter)" — but the trait surface never tells the adapter which school that is.
- **expected:** `docs/schemas/tenancy-schema.md:97-103`: "The storage adapter is responsible for enforcing tenant isolation. The engine always passes a `SchoolId` filter; the adapter MUST add a `school_id = $1` predicate to every read query." The Transaction trait must carry the `TenantContext` (or at least `SchoolId` + `ActorId`) for the sub-port impls to scope reads/writes against.
- **evidence:**
  ```rust
  crates/infra/storage/src/transaction.rs:52-75
  #[async_trait]
  pub trait Transaction: Send + Sync + std::fmt::Debug {
      async fn commit(self: Box<Self>) -> Result<()>;
      async fn rollback(self: Box<Self>) -> Result<()>;
      fn outbox(&self) -> &dyn Outbox;
      fn audit_log(&self) -> &dyn AuditLog;
      fn idempotency(&self) -> &dyn Idempotency;
      fn event_log(&self) -> &dyn EventLog;
      ...
  }
  ```
  No `tenant()`, `school_id()`, or `actor_id()` accessor.

---

### FINDING 3

- **id:** PORT-STORE-003
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/outbox.rs:78-115`
- **description:** `Outbox::append`, `Outbox::pending`, `Outbox::mark_published`, and `Outbox::pending_count` have no `school_id` parameter on the trait surface, yet the doc-comment at line 105 states "The outbox is partitioned by `school_id` so callers see only envelopes for their school." The trait has no `school_id()` accessor (unlike the `bulk_insert_student_attendances` row, which carries its own `school_id` for cross-validation). Every adapter must hold the school internally, but the trait does not declare this invariant — and `Outbox::pending_count(school_id: SchoolId)` at line 113 takes an explicit school_id that can be *any* school, bypassing the adapter's own scoping. Wave-3 finding ADAPTER-PG-013 confirms this is exploited in practice.
- **expected:** `docs/schemas/tenancy-schema.md:97-103` — "The storage adapter is responsible for enforcing tenant isolation." The `Outbox` trait must expose a `school_id()` accessor (or a `&TenantContext` field) so the adapter cannot drift, and `pending_count(school_id: SchoolId)` must be removed in favour of a parameterless `pending_count()` that uses the impl's scoped school.
- **evidence:**
  ```rust
  crates/infra/storage/src/outbox.rs:108-115
  async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>>;
  async fn mark_published(&self, ids: &[EventId]) -> Result<()>;
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      ...
  }
  ```

---

### FINDING 4

- **id:** PORT-STORE-004
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/audit.rs:96-114`
- **description:** `AuditLogEntry` is missing seven fields that `docs/schemas/audit-schema.md` § 2 mandates as part of the audit record: `audit_id` (UUIDv7 PK), `actor_type` (`user`/`system`/`agent`/`api_key`), `command_id`, `ip: Option<IpAddr>`, `user_agent: Option<String>`, `session_id: Option<SessionId>`, `cross_tenant: bool`, `source: AuditSource`, and a separate `recorded_at` distinct from `occurred_at`. The canonical PG DDL at `migrations/engine/0000_engine_core.postgres.sql:96-119` declares all 22 columns; the port struct carries 12. Every storage adapter that builds an `AuditLogEntry` cannot populate the missing columns — the wave-3 PG finding ADAPTER-PG-020 documents `#[allow(dead_code)]` annotations on the row struct for the same fields.
- **expected:** `docs/schemas/audit-schema.md` § 2 (audit record shape) — every field is part of the audit record contract; `migrations/engine/0000_engine_core.postgres.sql:96-119` lists the columns the engine must write.
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:62-93
  pub struct AuditLogEntry {
      pub school_id: SchoolId,
      pub actor_id: UserId,
      pub action: String,
      pub target_type: String,
      pub target_id: Uuid,
      pub before: Option<bytes::Bytes>,
      pub after: Option<bytes::Bytes>,
      pub event_id: Option<EventId>,
      pub correlation_id: CorrelationId,
      pub occurred_at: Timestamp,
      pub active_status: ActiveStatus,
      pub metadata: serde_json::Value,
  }
  ```
  No `audit_id`, no `actor_type`, no `command_id`, no `ip`, no `user_agent`, no `session_id`, no `cross_tenant`, no `source`, no separate `recorded_at`.

---

### FINDING 5

- **id:** PORT-STORE-005
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/idempotency.rs:29-44`
- **description:** `IdempotencyRecord.command_type` is typed `&'static str`. This forces every adapter that reads the column back to `Box::leak` the VARCHAR value to satisfy the `&'static str` lifetime — wave-3 finding ADAPTER-PG-011 documents the unbounded heap growth in the PG adapter, and wave-3 finding ADAPTER-SQ-006 documents the same leak in the SQLite adapter. The port shape is the cause: `&'static str` is impossible to construct from a heap-allocated column value without a leak. AGENTS.md § "Type Safety" forbids `Box::leak` in production paths.
- **expected:** `AGENTS.md` § "Type Safety": "No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler. Delete unused code, wire it in, or open a follow-up issue." The port struct must use `String` (or `Cow<'static, str>`) so adapters can return owned data without leaking.
- **evidence:**
  ```rust
  crates/infra/storage/src/idempotency.rs:29-44
  pub struct IdempotencyRecord {
      pub school_id: SchoolId,
      pub command_type: &'static str,
      pub idempotency_key: IdempotencyKey,
      pub outcome: bytes::Bytes,
      pub outcome_version: u32,
      pub recorded_at: educore_core::value_objects::Timestamp,
      pub affected_aggregate_ids: Vec<Uuid>,
  }
  ```
  And the analogous field on the lookup key struct at line 58:
  ```rust
  pub command_type: &'static str,
  ```

---

### FINDING 6

- **id:** PORT-STORE-006
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/transaction.rs:25-37`
- **description:** The `commit` doc-comment claims "Conflict on a unique-key violation, deadlock, or serialisation failure (the engine retries the command automatically)" — but the port defines no retry policy, no `is_retryable(&self, err)` method, no error classification scheme that lets the engine distinguish retryable conflicts from non-retryable ones. The wave-3 PG finding ADAPTER-PG-025 confirms the engine has no retry path: a bulk-insert unique-key violation returns `DomainError::conflict` and the caller is on its own. The port's stated contract is unenforceable.
- **expected:** `docs/ports/storage.md:124-127` — "On commit the writes are persisted and the outbox events are released to the event bus." `docs/ports/storage.md:131-137` — the engine retries on conflict. The port must expose a `Conflict`-vs-`Permanent` error distinction (or a retry predicate) so the engine can drive the retry policy.
- **evidence:**
  ```rust
  crates/infra/storage/src/transaction.rs:26-37
  async fn commit(self: Box<Self>) -> Result<()>;
  /// Rolls the transaction back. ...
  async fn rollback(self: Box<Self>) -> Result<()>;
  ```
  Doc comment lines 32-36:
  ```text
  /// # Errors
  /// - `Conflict` on a unique-key violation, deadlock, or
  ///   serialisation failure (the engine retries the command
  ///   automatically).
  ```
  No retry predicate, no error classification.

---

### FINDING 7

- **id:** PORT-STORE-007
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/audit.rs:14-50` (`AuditLogEntry`)
- **description:** `AuditLogEntry::action` is typed `String` and `AuditLogEntry::target_type` is typed `String`, while `docs/schemas/audit-schema.md` § 2 specifies them as `AuditAction` and `ResourceType` enums. The engine rule at AGENTS.md § "Compile-time safety over strings" mandates macro-generated enums, not free-form strings. The audit-schema canonical DDL declares `action VARCHAR(191) NOT NULL` and `resource_type VARCHAR(64) NOT NULL` — the port's `String` defeats the type-level audit-routing guarantees the spec promises.
- **expected:** AGENTS.md § "Engine Rules" rule 2: "Compile-time safety over strings. Use macro-generated enums (`StudentField::Status`) — never string field names."
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:67-69
  pub action: String,
  ...
  pub target_type: String,
  ```
  vs `migrations/engine/0000_engine_core.postgres.sql:101-102`:
  ```
  action          VARCHAR(191) NOT NULL,
  resource_type   VARCHAR(64)  NOT NULL,
  ```
  No `AuditAction` or `ResourceType` enum exists in the crate (`grep -rn "enum AuditAction\|enum ResourceType" crates/infra/storage/src/` returns no results).

---

### FINDING 8

- **id:** PORT-STORE-008
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/audit.rs:106-114` (`AuditLog` trait)
- **description:** The `AuditLog` trait exposes only `append` and `read_for_target` (a per-aggregate history). The audit-schema.md § 5 mandate requires a full `AuditQuery` port with `list`, `get`, `resource_history`, `actor_history`, and filter variants `ByAction`, `ByResource`, `ByActor`, `ByCorrelation`, `ByTimeRange`, `ByEventType`, `ByCustom`. The port provides none of these — there is no method to query "every record of action X in the last 30 days" or "every record for actor Y in window W". `read_for_target` returns at most `limit` rows, with no offset, no cursor, no actor filter, no time-range filter, no correlation filter.
- **expected:** `docs/schemas/audit-schema.md` § 5 — `AuditQuery` trait with `list`, `get`, `resource_history`, `actor_history` and the `AuditFilter` enum.
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:106-114
  #[async_trait]
  pub trait AuditLog: Send + Sync {
      async fn append(&self, entry: AuditLogEntry) -> Result<()>;
      async fn read_for_target(
          &self,
          school_id: SchoolId,
          target_id: Uuid,
          limit: u32,
      ) -> Result<Vec<AuditLogEntry>>;
  }
  ```
  `read_for_target` accepts only `(school_id, target_id, limit)` — no actor, no action, no correlation, no time range, no offset, no cursor.

---

### FINDING 9

- **id:** PORT-STORE-009
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/repository.rs:33-92`
- **description:** The `Repository<A>` trait takes only `school_id: SchoolId` on every method; there is no `TenantContext` parameter, so the adapter has no `actor_id`, `correlation_id`, `session_id`, `user_agent`, or `ip` to stamp `created_by`/`updated_by`/`cross_tenant` columns with on `insert`/`update`/`soft_delete`. The audit-schema.md § 14 columns include `created_by` and `updated_by` as NOT NULL — the storage adapter cannot populate them without a separate out-of-band channel. The trait comment at line 31 admits "the engine never observes a half-built result" but provides no way for the engine to pass per-call actor identity.
- **expected:** `docs/schemas/audit-schema.md` § 14 — `created_by`, `updated_by` columns are mandatory. `docs/schemas/database-schema.md` § 2 — `created_by` and `updated_by` are required on every aggregate table. The port must accept `&TenantContext` so the adapter can read `actor_id` and stamp the row.
- **evidence:**
  ```rust
  crates/infra/storage/src/repository.rs:36-92
  async fn get(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>>;
  async fn get_including_retired(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>>;
  async fn list(&self, school_id: SchoolId, offset: u32, limit: u32) -> Result<Vec<A>>;
  async fn count(&self, school_id: SchoolId) -> Result<u64>;
  async fn insert(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;
  async fn update(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;
  async fn soft_delete(&self, school_id: SchoolId, id: Uuid) -> Result<()>;
  ```
  No `&TenantContext` parameter anywhere.

---

### FINDING 10

- **id:** PORT-STORE-010
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/repository.rs:33-92`
- **description:** The `Repository<A>` trait provides no `stream` method, but `docs/ports/storage.md:148-153` explicitly mandates "the adapter may expose a streaming method … `async fn stream(&self, q: StudentQuery) -> Result<BoxStream<'static, Result<Student>>>`". The build-plan § Phase 5 exit criteria require the bulk-attendance load test to handle 10k rows in <5s; without streaming, the entire result set is loaded into memory. The `list(school_id, offset, limit)` signature at line 73 caps `offset`/`limit` at `u32` (4 billion rows), but a school with 10k attendance marks per day for 365 days = 3.65M rows — loaded as a `Vec<A>` the trait demands materialisation.
- **expected:** `docs/ports/storage.md:148-153` — streaming method on every per-aggregate repository.
- **evidence:**
  ```rust
  crates/infra/storage/src/repository.rs:73
  async fn list(&self, school_id: SchoolId, offset: u32, limit: u32) -> Result<Vec<A>>;
  ```
  No `stream` method on the trait. `grep -n "fn stream" crates/infra/storage/src/repository.rs` returns no results.

---

### FINDING 11

- **id:** PORT-STORE-011
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/idempotency.rs:62-118` (`Idempotency` trait)
- **description:** The `Idempotency` trait has no `expires_at` field on `IdempotencyRecord` and no `purge_expired` / `purge` method. The `purge_older_than(school_id, cutoff)` method at line 108 is the only maintenance entry point and is documented as "the consumer configures" — but there is no per-record TTL contract on the port. `docs/schemas/command-schema.md` § 6 mandates "the engine retains idempotency records for the duration the consumer configures (default 7 days)" — yet the port cannot represent `expires_at` in the record, cannot enforce TTL on `lookup` (i.e. "if expired, treat as not-found"), and provides no way for the engine to schedule retention sweeps. Adapters that don't override `purge_older_than` (its default returns `Ok(0)` at line 110-112) silently accumulate rows forever.
- **expected:** `docs/schemas/command-schema.md` § 6: "The engine retains idempotency records for the duration the consumer configures (default 7 days)." The port must carry an `expires_at` field, an `is_expired()` predicate, and a non-default `purge_older_than` contract.
- **evidence:**
  ```rust
  crates/infra/storage/src/idempotency.rs:29-44
  pub struct IdempotencyRecord {
      pub school_id: SchoolId,
      pub command_type: &'static str,
      pub idempotency_key: IdempotencyKey,
      pub outcome: bytes::Bytes,
      pub outcome_version: u32,
      pub recorded_at: educore_core::value_objects::Timestamp,
      pub affected_aggregate_ids: Vec<Uuid>,
  }
  ```
  No `expires_at`. And `purge_older_than` default at lines 107-112:
  ```rust
  async fn purge_older_than(
      &self,
      _school_id: SchoolId,
      _cutoff: educore_core::value_objects::Timestamp,
  ) -> Result<u64> {
      Ok(0)
  }
  ```

---

### FINDING 12

- **id:** PORT-STORE-012
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/idempotency.rs:86-99`
- **description:** The `record(record: IdempotencyRecord)` doc-comment promises "Returns `Err(Conflict)` if a record with the same `(school_id, command_type, idempotency_key)` already exists with a different outcome" — but the trait declares no method for comparing outcomes (hash? bytes equality? semantic equivalence?) and no marker field on `IdempotencyRecord` that lets the adapter decide. Wave-3 finding ADAPTER-PG-031 documents the PG adapter collapsing both branches into `DO NOTHING`; the trait-level contract is too ambiguous to implement correctly.
- **expected:** `docs/schemas/command-schema.md` § 6 and `ADR-014-Idempotency.md` — the port must declare the outcome-comparison contract (a `outcome_hash: Hash` field on `IdempotencyRecord` plus an `equal_outcome(&self, other: &IdempotencyRecord) -> bool` method) so every adapter can determine "different outcome" without ambiguity.
- **evidence:**
  ```rust
  crates/infra/storage/src/idempotency.rs:86-99
  /// Stores `record`. Returns `Err(Conflict)` if a record with
  /// the same `(school_id, command_type, idempotency_key)`
  /// already exists with a different outcome. Returns `Ok(())`
  /// if the record is a no-op write (same key, same outcome
  /// hash) — the engine uses this for at-least-once delivery
  /// of retries.
  async fn record(&self, record: IdempotencyRecord) -> Result<()>;
  ```
  No `outcome_hash`, no comparison method, no `equal_outcome` predicate.

---

### FINDING 13

- **id:** PORT-STORE-013
- **area:** infra-storage
- **severity:** Critical
- **location:** `crates/infra/storage/src/change_stream.rs:46-83`
- **description:** `ChangeFilter` carries only `school_id`, `since: Option<VersionCursor>`, and `aggregate_types: Vec<AggregateTypeFilter>` — but no `event_types` filter, no `actor_id` filter, no `correlation_id` filter, no `since_time: Option<Timestamp>` (only `since: VersionCursor`), no `until_time` / `until_cursor`. A consumer that wants "all `finance.invoice.generated` events for school X since cursor Y" cannot express that query. The `ChangeEvent` payload (lines 119-135) also lacks `event_type`, so consumers can't route on event-type without re-parsing the payload bytes.
- **expected:** `docs/schemas/event-schema.md` § 10 (subscription model) — `subscribe("finance.invoice.*")` and `subscribe_aggregate(...)` and `subscribe_school(...)`. The change-stream port must accept an `event_types` filter and the `ChangeEvent` must carry `event_type` as a first-class field.
- **evidence:**
  ```rust
  crates/infra/storage/src/change_stream.rs:46-56
  pub struct ChangeFilter {
      pub school_id: SchoolId,
      pub since: Option<VersionCursor>,
      pub aggregate_types: Vec<AggregateTypeFilter>,
  }
  ```
  No `event_types`, no `actor_id`, no `correlation_id`, no time-range filter.
  ```rust
  crates/infra/storage/src/change_stream.rs:119-135
  pub struct ChangeEvent {
      pub event_id: EventId,
      pub school_id: SchoolId,
      pub aggregate_type: String,
      pub aggregate_id: uuid::Uuid,
      #[serde(with = "bytes_via_vec")]
      pub payload: bytes::Bytes,
      pub cursor: VersionCursor,
  }
  ```
  No `event_type` field.

---

### FINDING 14

- **id:** PORT-STORE-014
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/port.rs:64-82` (`bulk_insert_student_attendances` default impl)
- **description:** The default implementation of `bulk_insert_student_attendances` returns `DomainError::NotSupported` and the doc-comment at lines 64-82 justifies this with "adapters that don't (e.g. the Phase 0 SurrealDB stub) get the unsupported error, which is the correct answer for that topology." However, `docs/ports/storage.md:469-477` lists a 10k-attendance-row load test as a port requirement, and `docs/build-plan.md` § Phase 5 names the bulk-marking service as a Phase 5 exit criterion. A silent `NotSupported` from the port's default impl allows an adapter to ship without implementing the feature, and the consumer sees `NotSupported` at the first attendance mark — too late to reconfigure the deployment.
- **expected:** The trait should distinguish "this adapter does not support bulk-marking" (terminal) from "this adapter has not yet implemented bulk-marking" (placeholder that fails loudly at startup). The port contract at `docs/ports/storage.md:469-477` requires 10k attendance marks in <5s — a hard port requirement, not an optional feature.
- **evidence:**
  ```rust
  crates/infra/storage/src/port.rs:64-82
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> {
      let _ = (ctx, rows);
      Err(educore_core::error::DomainError::not_supported(
          "StorageAdapter::bulk_insert_student_attendances is not supported by this adapter",
      ))
  }
  ```

---

### FINDING 15

- **id:** PORT-STORE-015
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/event_log.rs:142-148` (`EventLog::read` and `count`)
- **description:** `EventLog::read(filter)` and `EventLog::count(filter)` take the filter by value (consuming it), and `EventLogFilter::limit: u32` is the only pagination control. There is no `cursor: Option<EventId>` / `cursor: Option<VersionCursor>` field on `EventLogFilter`, so a consumer cannot paginate "read up to event Y, then continue from Y+1" — it can only do "events in window [since, until] with limit". The doc-comment at line 145 admits "consumers should paginate" but provides no pagination primitive. For a school's 7-year retention with millions of events, this is a hard limit at `u32::MAX = 4.29B` rows per query, with no way to resume a partial read.
- **expected:** `docs/schemas/event-schema.md` § 9: "Replay is supported: a consumer can request 'all events of type X since event_id Y' or 'all events of aggregate Z up to time T'." The `EventLogFilter` must carry a `since_event_id: Option<EventId>` cursor and the trait must return the next-page cursor alongside the rows.
- **evidence:**
  ```rust
  crates/infra/storage/src/event_log.rs:96-104
  pub struct EventLogFilter {
      pub school_id: SchoolId,
      pub event_types: Vec<String>,
      pub since: Option<Timestamp>,
      pub until: Option<Timestamp>,
      pub aggregate_id: Option<Uuid>,
      pub limit: u32,
  }
  ```
  No `since_event_id` cursor. And `EventLog::read` at line 147 takes `filter` by value:
  ```rust
  async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>>;
  ```

---

### FINDING 16

- **id:** PORT-STORE-016
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/change_stream.rs:78-83` (`ChangeStream`)
- **description:** `ChangeStream::close(self)` is documented as "drops the inner stream, which closes the underlying channel" (line 100-103). For a live CDC subscription (PG `LISTEN/NOTIFY`, SurrealDB `LIVE SELECT`), there is no graceful close — the underlying socket/connection is dropped without an unsubscribe / unlisten handshake, leaving the server side holding the subscription until it times out. For a sync engine with N schools and M concurrent subscribers, this accumulates server-side resource leaks proportional to churn.
- **expected:** The trait must distinguish "drop" (cancels) from "close" (graceful unsubscribe). `docs/ports/storage.md:111-116` describes sync primitives as "live" CDC with reconnect/resume — a hard-drop close breaks the contract.
- **evidence:**
  ```rust
  crates/infra/storage/src/change_stream.rs:78-103
  pub struct ChangeStream {
      pub inner: Pin<Box<dyn futures::Stream<Item = Result<ChangeEvent, educore_core::error::DomainError>> + Send + Sync>>,
  }
  ...
  pub async fn close(self) -> Result<(), educore_core::error::DomainError> {
      drop(self);
      Ok(())
  }
  ```

---

### FINDING 17

- **id:** PORT-STORE-017
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/transaction.rs:51-75`
- **description:** The `Transaction` trait has no `begin_nested` / `savepoint` method. `docs/ports/storage.md` lists no savepoints in the contract, but the engine's bulk-marking service (Phase 5) and the multi-step platform-domain commands (Phase 16) require savepoints to scope per-item error handling without rolling back the parent transaction. Without savepoints, a single failed item in a bulk operation forces a full rollback, contradicting `docs/schemas/command-schema.md` § 12's `CollectErrors` failure policy ("records per-item errors in a result list without aborting the batch").
- **expected:** `docs/schemas/command-schema.md` § 12: "`failure_policy`: default `FailFast`; alternative is `CollectErrors` which records per-item errors in a result list without aborting the batch." The Transaction port must expose `begin_nested(&self) -> Result<Box<dyn Transaction>>` for savepoint scoping.
- **evidence:**
  ```rust
  crates/infra/storage/src/transaction.rs:51-75
  async fn commit(self: Box<Self>) -> Result<()>;
  async fn rollback(self: Box<Self>) -> Result<()>;
  fn outbox(&self) -> &dyn Outbox;
  fn audit_log(&self) -> &dyn AuditLog;
  fn idempotency(&self) -> &dyn Idempotency;
  fn event_log(&self) -> &dyn EventLog;
  ```
  No `begin_nested`, no `savepoint`, no nested-transaction contract.

---

### FINDING 18

- **id:** PORT-STORE-018
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/outbox.rs:189-194` (`SerializedEnvelope::from_event_envelope`)
- **description:** `SerializedEnvelope::from_event_envelope` uses `serde_json::to_vec(&envelope.payload).unwrap_or_default()` (line 194) — a serialization failure silently produces an empty `bytes::Bytes` payload (`{}`) rather than propagating the error. A consumer that subscribes to the event bus sees an event with `payload = "{}"`, with no diagnostic that the original payload failed to serialize. The `metadata` field on the source `EventEnvelope` is also dropped without being copied to the outbox row (the canonical PG DDL declares a `metadata` JSONB column on the outbox table).
- **expected:** `docs/schemas/event-schema.md` § 3 — payload integrity is a wire-format invariant. The port must propagate serialization errors and must copy `metadata` into the outbox row.
- **evidence:**
  ```rust
  crates/infra/storage/src/outbox.rs:189-194
  pub fn from_event_envelope(envelope: &educore_events::envelope::EventEnvelope) -> Self {
      Self {
          ...
          payload: bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default()),
      }
  }
  ```
  No `metadata` field on `SerializedEnvelope` (outbox.rs:60-86) — the bus-port envelope's `metadata` is dropped.

---

### FINDING 19

- **id:** PORT-STORE-019
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/audit.rs:55-58`
- **description:** The `AuditLogEntry` struct's doc-comment says "`before` and `after` are serialised `serde_json::Value` (adapters are free to use any serializable type via `to_audit_value`)" but `before` and `after` are typed `Option<bytes::Bytes>` (line 80-86) and `metadata` is typed `serde_json::Value` (line 92). `docs/schemas/audit-schema.md` § 2.1 mandates three snapshot policies (`None`, `Diff`, `Full`) — the port has no `AuditSnapshotPolicy` field on `AuditLogEntry`, no `to_audit_value` helper, and no way for the caller to declare which policy to apply.
- **expected:** `docs/schemas/audit-schema.md` § 2.1 — three snapshot policies configurable per domain. The port must carry the policy marker so adapters know whether to capture `None`, `Diff`, or `Full`.
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:55-58 (module doc)
  /// before and after are serialised `serde_json::Value`
  /// (adapters are free to use any serializable type via
  /// `to_audit_value`).
  ```
  And the struct fields at lines 80-92:
  ```rust
  pub before: Option<bytes::Bytes>,
  pub after: Option<bytes::Bytes>,
  pub metadata: serde_json::Value,
  ```
  No `to_audit_value` helper exists (`grep -rn "fn to_audit_value" crates/infra/storage/src/` returns no results).

---

### FINDING 20

- **id:** PORT-STORE-020
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/audit.rs:53-93`
- **description:** `AuditLogEntry::metadata` is typed `serde_json::Value`. AGENTS.md § "Type Safety" forbids `serde_json::Value` in domain code: "No `serde_json::Value` in domain code. Use typed wrappers." Audit metadata is a domain concern; the open-ended `Value` defeats the type-safety guarantees the engine's other sub-ports provide.
- **expected:** AGENTS.md § "Type Safety": "No `serde_json::Value` in domain code. Use typed wrappers."
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:92
  pub metadata: serde_json::Value,
  ```
  And `educore_core::value_objects` exports typed wrappers (the engine's audit and event envelopes use `AuditMetadata` / `EventMetadata` structs elsewhere); the storage port uses raw `Value` instead.

---

### FINDING 21

- **id:** PORT-STORE-021
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/outbox.rs:60-86` (`SerializedEnvelope`)
- **description:** `SerializedEnvelope` declares `aggregate_id: Uuid` (line 73) as a raw UUID instead of a typed `AggregateId` (the engine's typed-identifier pattern). Per AGENTS.md § "Type Safety" and `docs/schemas/database-schema.md` § 1.4: "Identifiers are opaque to consumers. Strings are never parsed" — and "The default implementation uses UUIDv7 (time-ordered) for distributed generation and global uniqueness. Adapter implementations MAY swap to ULID, snowflake, or auto-increment integers behind the storage port, but the engine API always returns typed identifier wrappers." The same struct then exposes `aggregate_type: String` (line 74) instead of an enum.
- **expected:** AGENTS.md § "Type Safety" and `docs/schemas/database-schema.md` § 1.4 — typed identifier wrappers throughout.
- **evidence:**
  ```rust
  crates/infra/storage/src/outbox.rs:72-74
  pub aggregate_id: Uuid,
  /// Aggregate type name (e.g. "student"). `String` (not
  /// `&'static str`) so the type is `DeserializeOwned`.
  pub aggregate_type: String,
  ```
  No `AggregateId` or `AggregateType` typed wrapper exists in the crate.

---

### FINDING 22

- **id:** PORT-STORE-022
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/transaction.rs:51-75`
- **description:** The `Transaction` trait is `Drop`-unsafe by construction. A consumer that holds a `Box<dyn Transaction>` and lets it drop without calling `commit` or `rollback` will trigger the default `Drop` impl on the trait object, which performs no rollback (the trait does not require `Drop`). For an SQL adapter that opens a real `sqlx::Transaction`, dropping the trait object also drops the inner `sqlx::Transaction` — the SQL adapter must implement `Drop` on its concrete type to rollback. But the trait surface never declares this requirement, and the testkit adapter (wave-4 finding TOOL-TK-002) shows the in-memory impl does not roll back on drop — the two implementations behave differently for the same `let _ = tx;` consumer code.
- **expected:** The port must declare a `Drop` requirement (e.g. "impls MUST rollback on drop if neither commit nor rollback has been called") or expose an explicit `discard()` method. The current ambiguity is a silent data-loss path on panic.
- **evidence:**
  ```rust
  crates/infra/storage/src/transaction.rs:51-75
  #[async_trait]
  pub trait Transaction: Send + Sync + std::fmt::Debug {
      async fn commit(self: Box<Self>) -> Result<()>;
      async fn rollback(self: Box<Self>) -> Result<()>;
      ...
  }
  ```
  No `Drop` requirement, no `discard()`, no cancellation-safety contract. Compare the testkit finding at `crates/tools/testkit/src/storage.rs:454-461` which has `rollback` as a no-op and waves-3 ADAPTER-PG-026 which has `commit` as a no-op.

---

### FINDING 23

- **id:** PORT-STORE-023
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/repository.rs:17-32` (`Repository<A>` trait declaration)
- **description:** The `Repository<A>` trait comment at lines 17-32 admits "For Phase 0 minimum-viable, we expose a single generic `Repository<A>` trait that all domain crates can use; when a domain needs aggregate-specific methods it can wrap or extend the generic trait." However, `docs/ports/storage.md:13-25` lists ~22 named repository handles per domain (`students`, `guardians`, `classes`, …) with `Arc<dyn StudentRepository>` style wiring — not a single generic `Repository<A>`. The port's generic shape does not match the contract's per-aggregate-handle shape.
- **expected:** `docs/ports/storage.md:14-25` — one named repository handle per aggregate root, e.g. `fn students(&self) -> Arc<dyn StudentRepository>`. The Phase 0 minimum-viable single-trait shape is a deviation from the documented contract.
- **evidence:**
  ```rust
  crates/infra/storage/src/repository.rs:33-36
  #[async_trait]
  pub trait Repository<A>: Send + Sync
  where
      A: Send + Sync + Clone + 'static,
  {
  ```
  Single generic trait. Compare `docs/ports/storage.md:14-25` which lists:
  ```text
  fn students(&self) -> Arc<dyn StudentRepository>;
  fn guardians(&self) -> Arc<dyn GuardianRepository>;
  fn classes(&self) -> Arc<dyn ClassRepository>;
  ... (22 named handles)
  ```

---

### FINDING 24

- **id:** PORT-STORE-024
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/change_stream.rs:62-71` (`AggregateTypeFilter`)
- **description:** `AggregateTypeFilter::Any` is the wildcard variant, but the doc-comment at line 69-71 says "Storage adapters that don't support wildcards may treat this as 'all types'." This is silent semantic drift between adapters — a consumer that subscribes with `Any` and expects a SQL `LIKE '%'` on a no-wildcard backend gets an undocumented "all types" expansion that may or may not include future aggregate types added after subscription start. The contract provides no way for the consumer to detect which behaviour the adapter implements.
- **expected:** The trait must declare whether `Any` is a literal wildcard match or an "all currently-known types" expansion, and adapters must report their capability (e.g. `supports_wildcard()`).
- **evidence:**
  ```rust
  crates/infra/storage/src/change_stream.rs:62-71
  pub enum AggregateTypeFilter {
      Exact(String),
      /// Match any aggregate type. Storage adapters that don't
      /// support wildcards may treat this as "all types".
      Any,
  }
  ```

---

### FINDING 25

- **id:** PORT-STORE-025
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/outbox.rs:113-121` (`Outbox::pending_count` default impl)
- **description:** `Outbox::pending_count`'s default impl materialises every pending row by calling `self.pending(u32::MAX).await?` and counting the resulting `Vec`. For a school with 1M pending outbox rows (a backlog scenario after a sync-engine outage), this allocates a 1M-element `Vec<SerializedEnvelope>` just to read its `len()`. The doc-comment at line 116-121 says "Adapters with efficient `COUNT(*)` support may override" but provides no upper-bound cap on `u32::MAX` and no fallback for adapters that don't override. The default impl is unbounded memory.
- **expected:** The default impl must use a streaming/chunked count (e.g. `COUNT(*)` via a separate dedicated query, or chunked iteration with `pending(limit)` capped at e.g. 10000), and the trait must enforce a memory bound.
- **evidence:**
  ```rust
  crates/infra/storage/src/outbox.rs:113-121
  async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
      // Default implementation: count via `pending` and check
      // length. Adapters with efficient `COUNT(*)` support may
      // override.
      let _ = school_id;
      Ok(self.pending(u32::MAX).await?.len() as u64)
  }
  ```

---

### FINDING 26

- **id:** PORT-STORE-026
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/event_log.rs:118-132` (`EventLogEntry`)
- **description:** `EventLogEntry` does not enforce the invariant `recorded_at >= occurred_at`. The doc-comment at line 121 says "Wall-clock time of the persistence (≥ `occurred_at`)" but neither the struct nor the `append` method nor any `new` constructor validates this. An adapter that constructs `EventLogEntry::from_serialized_envelope(env)` (line 154-171) sets `recorded_at: Timestamp::now()` — fine if the clock advances monotonically, but a clock skew / NTP step backwards would produce a row with `recorded_at < occurred_at`, breaking the engine's latency-projection invariant.
- **expected:** The struct must enforce the invariant at construction (e.g. `EventLogEntry::new(...)` returns `Result<Self, DomainError>` with a `Validation` error on `recorded_at < occurred_at`).
- **evidence:**
  ```rust
  crates/infra/storage/src/event_log.rs:154-171
  pub fn from_serialized_envelope(env: &super::outbox::SerializedEnvelope) -> Self {
      Self {
          ...
          occurred_at: env.occurred_at,
          recorded_at: Timestamp::now(),
          ...
      }
  }
  ```
  No `if recorded_at < occurred_at` check.

---

### FINDING 27

- **id:** PORT-STORE-027
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/outbox.rs:102-107` (`Outbox::append`)
- **description:** The `append` doc-comment is contradictory: "uniquely identified by `event_id`; duplicates must be rejected (or, equivalently, stored but never published)". The "or equivalently stored but never published" branch allows an adapter to silently swallow duplicate appends — but the `# Errors` section then says "Conflict if an envelope with the same `event_id` was already appended in the same school." An adapter implementing the silent-swallow branch would never return `Conflict`, breaking callers that rely on the error path to detect duplicate dispatch.
- **expected:** The contract must be precise: either `Conflict` on duplicate (the `# Errors` arm) or `Ok(())` with no error and no observable side effect (the "equivalent" arm). The doc-comment should pick one and remove the ambiguity.
- **evidence:**
  ```rust
  crates/infra/storage/src/outbox.rs:102-107
  /// ... the event is uniquely identified by `event_id`;
  /// duplicates must be rejected (or, equivalently, stored but
  /// never published).
  ///
  /// # Errors
  /// - `Conflict` if an envelope with the same `event_id` was
  ///   already appended in the same school.
  ```

---

### FINDING 28

- **id:** PORT-STORE-028
- **area:** infra-storage
- **severity:** High
- **location:** `crates/infra/storage/src/change_stream.rs:65-83` (`ChangeStream::inner`)
- **description:** `ChangeStream.inner` is declared `pub` (line 65-66). Per AGENTS.md § "Type Safety" / "Public items documented" and `docs/code-standards.md` "Public APIs are documented with rustdoc", a `pub` field requires rustdoc. The field has no doc comment, but the `pub` visibility exposes the inner stream directly, allowing external code to poll the stream bypassing the `next()` wrapper (which provides error transposition) and bypassing any backpressure the wrapper might apply.
- **expected:** The field must be `pub(crate)` or `pub(super)`, not `pub`. External consumers should use `ChangeStream::next()` and `ChangeStream::close()` only.
- **evidence:**
  ```rust
  crates/infra/storage/src/change_stream.rs:64-72
  pub struct ChangeStream {
      /// The inner stream of change events. Boxed and pinned to
      /// keep the type `dyn`-compatible and awaitable.
      pub inner: Pin<Box<...>>,
  }
  ```
  `pub inner: ...` — directly reachable.

---

### FINDING 29

- **id:** PORT-STORE-029
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/idempotency.rs:91-93` (`Idempotency::exists`)
- **description:** The default implementation of `Idempotency::exists` calls `self.lookup(key).await?` which fully deserialises the outcome payload (a JSON blob that may be MB-sized for a bulk-import outcome). The doc-comment says "adapters with a cheap existence check may override" — but for adapters that do not override, every existence probe on a hot path (e.g. the command dispatcher) deserialises the entire outcome payload. Wave-3 finding ADAPTER-SQ-007 documents the consequence: the SQLite adapter doesn't even deserialise `outcome_version` / `affected_aggregate_ids`, hard-coding them to `0` / `Vec::new()`.
- **expected:** The trait must declare a separate `is_expired(key, now) -> bool` predicate and the default `exists` should not deserialise the payload (a cheap `EXISTS(...)` query or an `outcome_hash` equality check on the key alone).
- **evidence:**
  ```rust
  crates/infra/storage/src/idempotency.rs:91-93
  async fn exists(&self, key: IdempotencyCompositeKey) -> Result<bool> {
      Ok(self.lookup(key).await?.is_some())
  }
  ```

---

### FINDING 30

- **id:** PORT-STORE-030
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/audit.rs` (entire file, no `audit_id` field)
- **description:** The canonical PG DDL declares `audit_id UUID NOT NULL` as the primary key (`migrations/engine/0000_engine_core.postgres.sql:96, 116`), but `AuditLogEntry` has no `audit_id` field. The adapter must generate the audit id externally and the port provides no helper. For tamper-evidence (`docs/schemas/audit-schema.md` § 3: "the engine provides no update_audit or delete_audit operation"), the audit id is the anchor for hash-chain / MAC contracts that the port surface never declares.
- **expected:** The port must carry `audit_id: AuditId` (a typed UUIDv7 wrapper) and expose a tamper-evidence hook (e.g. `audit_hash: Etag` or `audit_signature: Signature`).
- **evidence:**
  ```rust
  crates/infra/storage/src/audit.rs:62-93
  pub struct AuditLogEntry {
      pub school_id: SchoolId,
      pub actor_id: UserId,
      ...
  }
  ```
  No `audit_id`. Compare `migrations/engine/0000_engine_core.postgres.sql:96`:
  ```
  audit_id        UUID         NOT NULL,
  ...
  PRIMARY KEY (audit_id)
  ```

---

### FINDING 31

- **id:** PORT-STORE-031
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/student_attendance_row.rs:108-209`
- **description:** `StudentAttendanceRow` exposes `*_bytes()` accessors that convert UUID fields to 16-byte big-endian `Vec<u8>` — the accessors' doc-comments (lines 110-114, 116-119, etc.) state "Storage adapters bind UUID columns as raw bytes (`BYTEA` / `VARBINARY` / `BLOB`) per the `attendance_student_attendances` DDL". But the canonical PG DDL for the engine cross-cutting tables at `migrations/engine/0000_engine_core.postgres.sql` uses native `UUID` columns (e.g. line 9: `event_id UUID NOT NULL`). The bulk-attendance storage format is "decoupled from the canonical engine form" (the wave-3 finding ADAPTER-PG-021 acknowledges this), and the port wires the decoupling into the type system. Adapters that follow the port's bytes-on-the-wire contract store `BYTEA`; adapters that follow the engine's UUID-native contract store `UUID` — the port shape picks one and forces it on every adapter.
- **expected:** The port should expose only typed `Uuid` / `SchoolId` accessors, and the storage adapters should handle dialect-native binding (sqlx has `Type<Uuid>` for native UUID, `Type<Bytes>` for BYTEA).
- **evidence:**
  ```rust
  crates/infra/storage/src/student_attendance_row.rs:108-118
  pub fn school_id_bytes(&self) -> Vec<u8> {
      self.school_id.as_uuid().as_bytes().to_vec()
  }
  /// Returns the row's `id` as a 16-byte big-endian `Vec<u8>`.
  #[must_use]
  pub fn id_bytes(&self) -> Vec<u8> {
      self.id.as_bytes().to_vec()
  }
  ```

---

### FINDING 32

- **id:** PORT-STORE-032
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/lib.rs:46-49`
- **description:** `lib.rs` re-exports `educore_core::error::Result` (line 48) but does not re-export `educore_core::error::DomainError`. Every public trait method returns `Result<T, DomainError>`, but callers must import `DomainError` from `educore_core` separately. This causes every adapter to write `use educore_core::error::DomainError;` redundantly, and creates an inconsistency where the storage port is reachable as `educore_storage::Result` but the error type must be imported from a different path.
- **expected:** `lib.rs` should re-export `DomainError` alongside `Result` so the public surface is `educore_storage::{Result, DomainError, ...}`.
- **evidence:**
  ```rust
  crates/infra/storage/src/lib.rs:46-49
  pub use audit::{AuditLog, AuditLogEntry};
  pub use change_stream::{...};
  ...
  // Re-export the `educore_core::error::Result` alias for convenience.
  pub use educore_core::error::Result;
  ```
  No `pub use educore_core::error::DomainError;`.

---

### FINDING 33

- **id:** PORT-STORE-033
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/port.rs:88-94`, 100-106, 115-122, 127-133
- **description:** The four sync-primitive methods (`watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) all use `let _ = arg;` in their default impls to suppress unused-variable warnings — a pattern clippy `#[must_use]` would flag and a pattern that loses the named-argument intent. Wave-3 finding ADAPTER-PG-036 confirms the PG adapter overrides `watch_changes` and silently swallows all callers; the default `NotSupported` is the right contract, but the silenced argument names hide the API surface from grep / doc extraction.
- **expected:** Use `_arg` underscore prefixes (`_filter`, `_snapshot`, `_school_id`, `_to`) or `#[allow(unused_variables)]` on the method bodies; do not bind named variables only to drop them.
- **evidence:**
  ```rust
  crates/infra/storage/src/port.rs:88-94
  async fn watch_changes(&self, filter: ChangeFilter) -> Result<ChangeStream> {
      let _ = filter;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn apply_snapshot(&self, snapshot: SchoolSnapshot) -> Result<()> {
      let _ = snapshot;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn cursor_for(&self, school_id: SchoolId) -> Result<VersionCursor> {
      let _ = school_id;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  async fn advance_cursor(&self, school_id: SchoolId, to: VersionCursor) -> Result<()> {
      let _ = school_id;
      let _ = to;
      Err(educore_core::error::DomainError::not_supported(...))
  }
  ```

---

### FINDING 34

- **id:** PORT-STORE-034
- **area:** infra-storage
- **severity:** Medium
- **location:** `crates/infra/storage/src/port.rs:130-133` (`close(self: Box<Self>)`)
- **description:** `close` consumes `self: Box<Self>` (line 130). The trait is object-safe and consumed as `Arc<dyn StorageAdapter>` (per the module doc at line 10), but `Arc<dyn StorageAdapter>::close` would require the `Arc` to be unwrapped to a `Box` first — the consumer must `Arc::try_unwrap` (which fails if the Arc is shared) and then `Box::new(arc.into_inner())`. The consumer-facing API is `storage.close().await` (per `docs/ports/storage.md:21`), but no method signature on the public `StorageAdapter` accepts `&self`-style close. The port's signature forces the consumer to perform an `Arc::try_unwrap` dance that may fail at runtime.
- **expected:** The trait should expose a `close(&self)` that signals shutdown via an internal flag (the wave-3 PG adapter uses an `AtomicBool` for exactly this reason), with `Drop` releasing the connection pool as a fallback.
- **evidence:**
  ```rust
  crates/infra/storage/src/port.rs:130-133
  /// Closes the adapter, releasing all underlying
  /// connections. After `close`, any further call returns
  /// `Err(Infrastructure)`.
  async fn close(self: Box<Self>) -> Result<()>;
  ```
  And the storage-port docs at `docs/ports/storage.md:21`:
  ```text
  async fn close(&self) -> Result<()>;
  ```
  — note `&self`, not `self: Box<Self>`.

---

### FINDING 35

- **id:** PORT-STORE-035
- **area:** infra-storage
- **severity:** Low
- **location:** `crates/infra/storage/src/event_log.rs:155-159`
- **description:** `EventLogEntry` derives `Eq` (line 119) but the struct contains `Vec<u8>`-shaped `bytes::Bytes` payload which uses reference-counted shared memory; two `EventLogEntry` values are `Eq` only if they share the same `Arc` pointer. The engine's relays / projections commonly clone these entries to fan out to multiple consumers; two cloned entries compare `Eq` because they share the same `Arc`, but two entries with byte-identical payloads built independently are NOT `Eq` (they have different `Arc` pointers). This is a foot-gun for adapter parity tests.
- **expected:** Either drop `Eq` (use `PartialEq` only) or implement byte-wise equality on the payload.
- **evidence:**
  ```rust
  crates/infra/storage/src/event_log.rs:118-132
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct EventLogEntry {
      ...
      #[serde(with = "bytes_via_vec")]
      pub payload: bytes::Bytes,
      ...
  }
  ```

---

### FINDING 36

- **id:** PORT-STORE-036
- **area:** infra-storage
- **severity:** Low
- **location:** `crates/infra/storage/src/port.rs:60-83` (transaction.rs:78-105)
- **description:** The `bulk_insert_student_attendances` methods on both `StorageAdapter` and `Transaction` carry an identical signature (modulo the `&TenantContext` argument) and identical doc-comments, but the trait surfaces do not share a helper type or trait alias. The same field-by-field validation (school_id match, dedup on `(school_id, student_id, attendance_date)`) is duplicated across both methods and across every adapter (wave-3 finding ADAPTER-PG-025 confirms the dedup logic is re-implemented in `bulk_attendance.rs`). The port exposes no shared validator helper.
- **expected:** A `BulkAttendanceInsert` trait or `validate_bulk_attendance(ctx, rows) -> Result<Vec<&Row>>` free function in the port, so the validation contract has one source of truth.
- **evidence:**
  ```rust
  crates/infra/storage/src/port.rs:60-82
  async fn bulk_insert_student_attendances(
      &self,
      ctx: &TenantContext,
      rows: &[StudentAttendanceRow],
  ) -> Result<()> { ... }
  ```
  ```rust
  crates/infra/storage/src/transaction.rs:88-104
  async fn bulk_insert_student_attendances(&self, rows: &[StudentAttendanceRow]) -> Result<()> { ... }
  ```
  Two near-identical signatures, no shared validator.
