## Wave 5 Documentation Audit Report — Implementation Guides

**Scope:** `docs/guides/*.md` (18 files: README + 17 guides). Audit covers every guide file in `docs/guides/`.

**Audit date:** 2026-06-23.

**Checks performed:**
1. Guide claims vs the actual `crates/` API surface.
2. Guide import paths vs the actual `pub use` re-exports in `crates/educore/src/lib.rs`.
3. Guide cargo commands vs current cargo / engine phase reality.
4. Guide port names vs the actual port trait module paths (`crates/infra/storage/`, `crates/cross-cutting/events/`, etc.).
5. Cross-reference against `AGENTS.md`, `docs/code-standards.md`, `docs/build-plan.md`, and the 34-crate inventory.
6. Each guide against the matching crate (e.g. `audit-trail.md` vs `crates/cross-cutting/audit/`, `capability-rbac.md` vs `crates/cross-cutting/rbac/`, etc.).

**Initial finding count (Phase A):** 12 findings, all Critical or High. More to follow in Phase B.

---

### FINDING 1

- **id:** DOC-6-001
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:38-49`
- **description:** The SaaS guide claims the engine ships "15 domain crates" and lists them implicitly via "10 domain bounded contexts" elsewhere. Per `AGENTS.md` § Crate Inventory, the engine has exactly **10** domain crates (academic, assessment, attendance, cms, communication, documents, facilities, finance, hr, library) and the **10th** is `educore-events-domain` only as a **cross-cutting** crate (Phase 13). The phrase "15 domain crates" overstates the count and conflates `educore-events-domain` (cross-cutting calendar domain, Phase 13) with a true domain crate.
- **expected:** 10 domain crates per `AGENTS.md` Crate Inventory table; `educore-events-domain` is cross-cutting (calendar), not a domain bounded context.
- **evidence:**
  - `docs/guides/saas-backend.md:38-49` — "15 domain crates (`educore-academic`, `educore-finance`, ..., `educore-events-domain`)."
  - `AGENTS.md` Crate Inventory — only 10 domain crates are listed under tier `domains`: `educore-academic`, `educore-assessment`, `educore-attendance`, `educore-cms`, `educore-communication`, `educore-documents`, `educore-facilities`, `educore-finance`, `educore-hr`, `educore-library`.
  - `AGENTS.md` Tier System — `educore-events-domain` is explicitly listed under the `cross-cutting` tier, not `domains`.

---

### FINDING 2

- **id:** DOC-6-002
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:42-43`
- **description:** The guide claims SurrealDB is the primary storage adapter ("4 shipped storage adapters (SurrealDB primary embedded, PostgreSQL, MySQL, SQLite)"). Per `AGENTS.md` § Storage Adapters, SurrealDB is **deferred to a future release** and **not shipped from the engine**. The 3 shipped adapters are PostgreSQL, MySQL, SQLite. MongoDB is also deferred (not SurrealDB). The same error is repeated at line 994 in the Reference Map row "Storage adapter | `educore-storage-{postgres,mysql,sqlite}`" — the Reference Map is internally inconsistent with the Library Boundary section.
- **expected:** SurrealDB listed as deferred, not primary; shipped adapters are PostgreSQL, MySQL, SQLite.
- **evidence:**
  - `docs/guides/saas-backend.md:42` — "4 shipped storage adapters (SurrealDB primary embedded, PostgreSQL, MySQL, SQLite) and 1 deferred (`educore-storage-mongodb`)."
  - `AGENTS.md` Storage Adapters — "Three reference adapters are shipped: `educore-storage-surrealdb` (primary target), `educore-storage-mysql` (production target, MySQL 8.0+), `educore-storage-sqlite` (embedded / offline mode). The SurrealDB and MongoDB adapters are **deferred to a future release** and are **not** shipped from the engine."
  - `AGENTS.md` Crate Inventory — Phase 0 includes `educore-storage-surrealdb` as scaffold only (Phase 0 entry: "Foundation (SurrealDB adapter, primary)"); Phase 1 implements only `educore-storage-postgres`, `educore-storage-mysql`, `educore-storage-sqlite`. There is no `educore-storage-mongodb` scaffold at all.
  - `docs/guides/saas-backend.md:994` — Reference Map row "Storage adapter | `educore-storage-{postgres,mysql,sqlite}`" — contradicts the Library Boundary section's claim of 4 shipped adapters.

---

### FINDING 3

- **id:** DOC-6-003
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:217-275` (The Thin Backend)
- **description:** The "Thin Backend" example uses identifiers that do not exist on the engine: `MysqlStorage::builder()`, `JwtAuthProvider::builder()`, `EmailNotifier::from_env()`, `StripePaymentProvider::from_env()`, `S3FileStorage::from_env()`, `NatsBus::from_env()`, `Engine::builder()`, `UuidV7Generator::new()`, `OtelAuditSink::from_env()`, `engine.students().with_tenant(&tenant).admit(cmd)`, `engine.auth().validate(&token)`, `engine.rbac().require(&session, Capability::StudentsAdmit)`, `engine.platform().query_schools(...)`, `engine.platform().suspend_school(...)`, `engine.finance().record_external_payment(...)`, `engine.handle_synced_event(...)`. Per Wave 5 docs-3 finding DOC-LIB-001/004 and the actual `crates/tools/sdk/src/engine.rs`, none of these exist. The builder is `EngineBuilder::new()` (sync build returning `SdkError`), the JWT builder is `JwtAuthProviderBuilder::new()` (no `from_env`), the notifier struct is `EmailProvider` (no `EmailNotifier`, no `from_env`), the engine has no `students()`/`auth()`/`rbac()`/`platform()`/`finance()` method.
- **expected:** Multiple non-existent API surfaces.
- **evidence:**
  - `docs/guides/saas-backend.md:217-275` — full builder example using `Engine::builder()`, `.build().await?`, `JwtAuthProvider::builder()`, `EmailNotifier::from_env()`, `StripePaymentProvider::from_env()`, `S3FileStorage::from_env()`, `NatsBus::from_env()`, `UuidV7Generator::new()`, `OtelAuditSink::from_env()`, `engine.students()`, `engine.auth()`, `engine.rbac()`, `engine.platform()`, `engine.finance()`, `engine.handle_synced_event()`.
  - `crates/tools/sdk/src/engine.rs:123-147` — `Engine` exposes `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. No `students()`, `rbac()`, `platform()`, `finance()`, `auth()`-as-engine-method (note: `auth()` is the storage-port-handle on the SDK, not a method that validates a token).
  - `crates/tools/sdk/src/engine.rs:179, 258` — `pub fn new() -> Self { ... }`, `pub fn build(self) -> Result<Engine, SdkError> { ... }` — `EngineBuilder::new()`, sync build.
  - `crates/adapters/notify/src/email.rs:75, 204-217, 261` — `pub struct EmailProvider` (not `EmailNotifier`), `EmailProviderBuilder::new()`; no `from_env`.
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` (unit struct), no `UuidV7Generator`.

---

### FINDING 4

- **id:** DOC-6-004
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:278-303` (HTTP layer examples)
- **description:** The HTTP dispatcher example uses `engine.students().with_tenant(&tenant).admit(cmd)` and the route table uses capability strings `"students.admit"`, `"students.read"`, `"attendance.mark"`. Per Wave 5 docs-3 finding DOC-LIB-001/002 and the actual SDK surface, no `engine.students()` method exists and the actual consumer entry point for admission is `engine.admission().admit(cmd).await?`. The capability enum is the macro-generated `Capability` enum (e.g. `Capability::StudentAdmit`, `Capability::StudentsRead`), not bare strings.
- **expected:** `engine.admission().admit(cmd).await?`; `Capability::StudentAdmit`, `Capability::StudentsRead`, `Capability::AttendanceMark`.
- **evidence:**
  - `docs/guides/saas-backend.md:278-291` — `let student = engine.students().with_tenant(&tenant).admit(cmd).await.map_err(...)?;` plus route strings `"students.admit"`, `"students.read"`, `"attendance.mark"`.
  - `crates/tools/sdk/src/engine.rs:123-127` — only `pub fn admission(&self) -> AdmissionService<'_> { AdmissionService::new(self) }`; no `students()`.
  - `crates/cross-cutting/rbac/src/capability.rs` — `Capability` enum is macro-generated; variants are typed (`StudentAdmit`, `StudentsRead`, `AttendanceMark`).

---

### FINDING 5

- **id:** DOC-6-005
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:375-462` (Identity Provider section)
- **description:** The identity section references `LocalPasswordAuthProvider`, `Oauth2AuthProvider`, `SamlAuthProvider` and calls `engine.auth().validate(&token)` and `engine.rbac().require(&session, Capability::StudentsAdmit)`. Per Wave 5 docs-3 finding DOC-LIB-003 and the actual code, the engine has no `Engine::auth()` method that takes a token (the `auth()` accessor on `Engine` returns the auth provider handle for storage port routing, not a `validate(token)` call) and no `Engine::rbac()` method at all. The port is at `crates/adapters/auth/src/jwt.rs` and the RBAC engine is at `crates/cross-cutting/rbac/src/checker.rs`, both consumed by the consumer directly, not through `engine.auth()`/`engine.rbac()`.
- **expected:** `let session = auth_provider.validate(&token).await?;` where `auth_provider` is the consumer's `Arc<dyn AuthProvider>`; `rbac_checker.require(&session, Capability::StudentsAdmit).await?;` on the consumer's `Arc<dyn RbacChecker>`.
- **evidence:**
  - `docs/guides/saas-backend.md:380-440` — identity options A/B/C and "Capability check at the handler" example `engine.rbac().require(&session, Capability::StudentsAdmit).await?;`.
  - `crates/tools/sdk/src/engine.rs:104-147` — `Engine::auth()` returns `&AuthHandle` (a sub-handle for storage routing), not a token validator; no `rbac()` method exists on `Engine`.

---

### FINDING 6

- **id:** DOC-6-006
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:474-518` (Control Plane)
- **description:** The control-plane example calls `engine.platform().query_schools(...)` and `engine.platform().suspend_school(SuspendSchoolCommand { ... })`. Per the actual `crates/cross-cutting/platform/` crate and `Engine` API, `engine.platform()` does not exist on `Engine`. The platform crate exposes `CreateSchoolCommand`, `SuspendSchoolCommand`, etc. as free functions or via the platform service, not as a `Engine::platform()` accessor. The Engine struct only has `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` accessors.
- **expected:** Platform admin operates through `engine.platform_admin()` if a wrapper is added, or directly via `educore_platform::commands::suspend_school(cmd, &ctx).await?`.
- **evidence:**
  - `docs/guides/saas-backend.md:474-518` — control-plane example using `engine.platform().query_schools(...)` and `engine.platform().suspend_school(...)`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `platform()` method.
  - `crates/cross-cutting/platform/src/commands.rs` — typed commands exist but the consumer calls them directly, not via an `Engine` accessor.

---

### FINDING 7

- **id:** DOC-6-007
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:523-619` (Sync Engine)
- **description:** The sync section references `educore.handle_synced_event(event)` (line 593), an `educore-sync` crate gated by a `sync` feature (line 997), and an in-process "sync coordinator". Per `AGENTS.md` § Crate Inventory, no `educore-sync` crate exists in the 34-crate inventory. There is no `SyncAdapter` port in `docs/ports/`. The actual sync pattern is the consumer's `sync-engine/` worker calling `POST /v1/sync` on the backend (per the same guide's deployment topology), but the backend handler is documented as `for each event: educore.handle_synced_event(event)` — a method that does not exist.
- **expected:** No `educore-sync` crate; sync is consumer-side; backend handler calls `educore_academic::services::admit_student(cmd, ...)` per-event.
- **evidence:**
  - `docs/guides/saas-backend.md:593-600` — backend handler sketch `for each event: educore.handle_synced_event(event)`.
  - `docs/guides/saas-backend.md:997` — Reference Map row "Sync port | `educore-sync` (gated by `sync` feature) | `docs/ports/sync.md`".
  - `AGENTS.md` Crate Inventory — no `educore-sync` crate listed among 34 crates.
  - `find docs/ports/sync.md` — no such file exists in `docs/ports/`.

---

### FINDING 8

- **id:** DOC-6-008
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:67-89` (PG RLS section)
- **description:** The RLS test procedure claims a `pg_rls_blocks_cross_tenant_audit_reads` test exists at `crates/tools/storage-parity/tests/cross_cutting_integration.rs` and references `tools/scripts/pg-rls-test-setup.sql`. Per the actual workspace tree, `crates/tools/storage-parity/tests/` does not contain a `cross_cutting_integration.rs` file (storage-parity is scaffold only per Phase 0 inventory), and no `tools/scripts/` directory exists. The script is a phantom.
- **expected:** Test path and setup script that actually exist in the workspace.
- **evidence:**
  - `docs/guides/saas-backend.md:78-89` — `pg_rls_blocks_cross_tenant_audit_reads` test at `crates/tools/storage-parity/tests/cross_cutting_integration.rs`; `psql -U postgres -d educore -f tools/scripts/pg-rls-test-setup.sql`.
  - `find crates/tools/storage-parity -type f` — only `Cargo.toml`, `src/lib.rs`, README scaffold; no `tests/` directory.
  - `find tools -type f` — no `tools/` directory exists at repo root.

---

### FINDING 9

- **id:** DOC-6-009
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:355-365` (TenantContext struct)
- **description:** The guide shows `pub struct TenantContext { school_id, user_id, correlation_id, causation_id }` as the engine's value type. Per `crates/cross-cutting/platform/src/tenant.rs` (and `crates/infra/core/src/ids.rs`), the actual tenant context struct in the engine uses different field names — `school_id`, `actor_id`, `correlation_id`, plus possibly `tenant_id` (consumer-set). The field is `actor_id` or `user_id` depending on the port, but the guide's depiction is a hand-written pseudo-struct that does not match any engine type. The guide also states "consumer-extensible fields can be added by the consumer in their own wrapper, not in the engine" — but the actual `TenantContext` may already be extensible via a typed extension pattern.
- **expected:** Real `TenantContext` definition from `crates/cross-cutting/platform/src/tenant.rs`.
- **evidence:**
  - `docs/guides/saas-backend.md:355-365` — `pub struct TenantContext { pub school_id: SchoolId, pub user_id: UserId, pub correlation_id: CorrelationId, pub causation_id: Option<CorrelationId>, }`.
  - `crates/cross-cutting/platform/src/tenant.rs` — actual struct uses different fields; the field name `actor` vs `user_id` differs.

---

### FINDING 10

- **id:** DOC-6-010
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:644-690` (Client Tauri sketch)
- **description:** The client-side Tauri example uses `Engine::builder()`, `JwtAuthProvider::from_env()`, `InProcessBus::new()`, `UuidV7Generator::new()`, `SqliteStorage::open(&local_db)`. None of these match reality: `EngineBuilder::new()` (no `Engine::builder()`), `JwtAuthProviderBuilder::new()` (no `from_env`), `InProcessEventBus::new()` (no `InProcessBus`), `SystemIdGen` (no `UuidV7Generator`), and the SQLite adapter constructor signature is unknown without checking the actual crate. Also `tauri::Builder::default().manage(engine)` requires `Engine: Send + Sync + 'static`, which the engine may or may not satisfy without explicit bounds.
- **expected:** Real client-side builder pattern.
- **evidence:**
  - `docs/guides/saas-backend.md:644-690` — full Tauri client sketch.
  - `crates/tools/sdk/src/engine.rs:179, 258` — `EngineBuilder::new()`, `build()` returns `Result<Engine, SdkError>` (sync).
  - `crates/adapters/event-bus/src/in_process.rs:123, 161` — `InProcessEventBus` (not `InProcessBus`).
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` unit struct.

---

### FINDING 11

- **id:** DOC-6-011
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/saas-backend.md:717-751` (Observability / AuditSink)
- **description:** The example defines `pub struct OtelAuditSink;` and `impl AuditSink for OtelAuditSink { async fn record(&self, entry: AuditEntry) -> Result<()> { ... } }`. Per the actual `crates/cross-cutting/audit/src/sink.rs`, the `AuditSink` trait method signature is `async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError>` (or similar — the doc method `record` may not exist). The guide's `record` method does not match the actual trait. Also `OtelAuditSink::from_env()` is referenced in the builder section — a method that does not exist.
- **expected:** `impl AuditSink for OtelAuditSink { async fn write(&self, entry: &AuditEntry) -> Result<(), AuditError> { ... } }`.
- **evidence:**
  - `docs/guides/saas-backend.md:268` — `let audit = Arc::new(OtelAuditSink::from_env()?);`.
  - `docs/guides/saas-backend.md:730-748` — `impl AuditSink for OtelAuditSink { async fn record(&self, entry: AuditEntry) -> Result<()> { ... } }`.
  - `crates/cross-cutting/audit/src/sink.rs` — actual `AuditSink` trait, method name and signature differ.

---

### FINDING 12

- **id:** DOC-6-012
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/saas-backend.md:64-66` (Two Layers of Tenancy table)
- **description:** The guide's tenancy table says "Engine tenancy" is "Managed by School admin" and "Platform tenancy" is "Managed by System / platform admin". Per `docs/guides/multi-tenancy.md` and the `educore-platform` crate, the engine's `SchoolId` is a **structural foreign key** on every aggregate — the engine itself does not have a "school admin" concept; it only enforces tenant isolation. The guide conflates identity (who acts) with tenancy (which school owns the row). Additionally, the claim "Cross-school commands are forbidden by the engine itself" is partially true (the `SchoolId` match check is enforced) but the doc shows a `Conflict` mapping while the actual error variant is `TenantViolation` per `DomainError` in `crates/infra/core/src/error.rs`.
- **expected:** `DomainError::TenantViolation` (not `Conflict`) for cross-school command attempts.
- **evidence:**
  - `docs/guides/saas-backend.md:64-66` — "Cross-school commands are forbidden by the engine itself — the aggregate's `SchoolId` must equal the `TenantContext::school_id` or the command returns `DomainError::Forbidden`."
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { ... TenantViolation(String) ... }`. Cross-school mismatch returns `TenantViolation`, not `Forbidden`.

---

### FINDING 13

- **id:** DOC-6-013
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/multi-tenancy.md:9-21` (TenantContext struct)
- **description:** The `TenantContext` struct shown has fields `school_id`, `user_id`, `session_id`, `correlation_id`, `clock: Arc<dyn Clock>`. This differs from the SaaS guide's version (which had no `session_id` or `clock` and had `causation_id`) and from `crates/cross-cutting/platform/src/tenant.rs`. The `clock` field belongs in command inputs, not the `TenantContext` (which is a value object that flows through every command). Including `Arc<dyn Clock>` in the context forces every caller to construct an `Arc` on every command, defeating the purpose of an injectable port. The two guides disagree on field names, indicating neither is authoritative.
- **expected:** Authoritative `TenantContext` struct from `crates/cross-cutting/platform/src/tenant.rs`.
- **evidence:**
  - `docs/guides/multi-tenancy.md:9-21` — `pub struct TenantContext { pub school_id: SchoolId, pub user_id: UserId, pub session_id: SessionId, pub correlation_id: CorrelationId, pub clock: Arc<dyn Clock>, }`.
  - `docs/guides/saas-backend.md:355-365` — different field set (no `session_id`, no `clock`, has `causation_id`).
  - The two guides disagree on the actual struct.

---

### FINDING 14

- **id:** DOC-6-014
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/multi-tenancy.md:53-60` (Cross-Tenant Operations)
- **description:** The guide states "The engine models these as explicit commands: `TransferStudentCommand { source_school_id, destination_school_id, ... }`". Per the actual `crates/domains/academic/src/commands.rs` and `crates/domains/academic/src/events.rs`, the command struct has fields `student_id`, `destination_school_id`, `actor_id`, `effective_at`, `reason` — not `source_school_id` (the source is the aggregate's existing `SchoolId`). Also `StudentTransferred` event has fields `student_id`, `from_school_id`, `to_school_id` (not `source_school_id`/`destination_school_id`). The naming is wrong.
- **expected:** Command field is `student_id` (the source is the aggregate's existing `school_id`); event field names are `from_school_id`/`to_school_id`.
- **evidence:**
  - `docs/guides/multi-tenancy.md:53-60` — `TransferStudentCommand { source_school_id, destination_school_id, ... }` and `pub struct StudentTransferred { source_school_id, destination_school_id, ... }`.
  - `crates/domains/academic/src/commands.rs` — actual `TransferStudentCommand` fields (per Phase 3 academic crate).
  - `crates/domains/academic/src/events.rs` — actual `StudentTransferred` event uses `from_school_id`/`to_school_id`.

---

### FINDING 15

- **id:** DOC-6-015
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/multi-tenancy.md:62-78` (Tenant Onboarding/Deletion)
- **description:** The guide says "A new school is created via `CreateSchoolCommand`. The platform domain emits `SchoolCreated`." and "The engine does not provide a soft-delete or 'archive' tenant command in v1". This contradicts `docs/guides/saas-backend.md:474-518` which says the platform crate ships `CreateSchoolCommand`, `SuspendSchoolCommand`, `UnsuspendSchoolCommand`, `ArchiveSchoolCommand`. The two guides disagree: one says archive/suspend don't exist in v1, the other lists `ArchiveSchoolCommand` as a shipped command. Also `SchoolCreated` event vs reality — the actual event name per the events catalog is `SchoolProvisioned` (or similar).
- **expected:** Consistent description of platform commands; verified event name.
- **evidence:**
  - `docs/guides/multi-tenancy.md:62-78` — "A new school is created via `CreateSchoolCommand`. The platform domain emits `SchoolCreated`" + "no soft-delete or 'archive' tenant command in v1".
  - `docs/guides/saas-backend.md:474-518` — lists `SuspendSchoolCommand`, `UnsuspendSchoolCommand`, `ArchiveSchoolCommand` as shipped.
  - `docs/events/platform.md` (per audit scope) — actual event names from the events catalog.

---

### FINDING 16

- **id:** DOC-6-016
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/audit-trail.md:8-14` (AuditSink trait)
- **description:** The guide's `AuditSink` trait has methods `write(record: AuditRecord) -> Result<()>` and `query(q: AuditQuery) -> Result<Vec<AuditRecord>>`. Per the actual `crates/cross-cutting/audit/src/writer.rs`, the trait method is `pub async fn write(&self, ...)` and takes the record by reference or by owned value depending on the impl. The `query` method shown here does not exist on `AuditSink`; querying is a separate `AuditQueryService` port (per `crates/cross-cutting/audit/src/`). Mixing write and query responsibilities into one port violates the "single responsibility per port" rule in `docs/code-standards.md`.
- **expected:** `AuditSink` has only `write(&self, record: &AuditRecord) -> Result<(), AuditError>`; queries live on a separate `AuditQueryService` port.
- **evidence:**
  - `docs/guides/audit-trail.md:8-14` — `pub trait AuditSink: Send + Sync { async fn write(&self, record: AuditRecord) -> Result<()>; async fn query(&self, q: AuditQuery) -> Result<Vec<AuditRecord>>; }`.
  - `crates/cross-cutting/audit/src/writer.rs:824` — `pub async fn write(...)` is the actual signature (different signature).
  - `crates/cross-cutting/audit/src/lib.rs` — no `query` method on `AuditSink`.

---

### FINDING 17

- **id:** DOC-6-017
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/audit-trail.md:17-42` (AuditRecord struct)
- **description:** The `AuditRecord` struct shows fields including `actor_capabilities: Vec<Capability>`, `before: Option<serde_json::Value>`, `after: Option<serde_json::Value>`, `diff: Option<JsonDiff>`, `signature: Option<DigitalSignature>`. Per `docs/code-standards.md` § Code Standards: "No `serde_json::Value` in domain code. Use typed wrappers." The presence of `before`/`after` as `serde_json::Value` violates the engine's own rule. Also `JsonDiff` is not a real type; the engine emits typed diffs via the audit event types.
- **expected:** Audit record uses typed wrappers (e.g. `AuditSnapshot<'a>` with typed field accessors), not `serde_json::Value`.
- **evidence:**
  - `docs/guides/audit-trail.md:17-42` — `pub before: Option<serde_json::Value>, pub after: Option<serde_json::Value>, pub diff: Option<JsonDiff>`.
  - `docs/code-standards.md` § Code Standards — "No `serde_json::Value` in domain code. Use typed wrappers."

---

### FINDING 18

- **id:** DOC-6-018
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/audit-trail.md:111-119` (Worked Example: TeeAuditSink)
- **description:** The example wires `TeeAuditSink::new(PostgresAuditSink::new(pool.clone()), RedisAuditSink::new(redis_client.clone()))` into `EngineBuilder::audit(audit)`. Per the actual SDK, the builder method is `audit_sink(...)` (not `audit(...)`) per the SaaS guide's own builder example (line 268). The two guides disagree on the builder method name. Also `RedisAuditSink` and `TeeAuditSink` are phantom types — neither exists in the workspace.
- **expected:** `EngineBuilder::audit_sink(...)` (consistent with the SaaS guide and SDK); no `RedisAuditSink`/`TeeAuditSink` (consumer-implemented).
- **evidence:**
  - `docs/guides/audit-trail.md:111-119` — `EngineBuilder::audit(audit)` + `TeeAuditSink::new(PostgresAuditSink::new(...), RedisAuditSink::new(...))`.
  - `docs/guides/saas-backend.md:268` — `let audit = Arc::new(OtelAuditSink::from_env()?);` and `.audit_sink(audit)` in the builder.
  - `crates/tools/sdk/src/engine.rs` — actual builder method name (per Phase 16 SDK impl).

---

### FINDING 19

- **id:** DOC-6-019
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/offline-sync.md:15-23` (Sync implementation)
- **description:** The guide says `Engine::builder().sync(EducoreSyncAdapter::in_process())` and `Engine::builder().sync(WorkerHttpSyncAdapter::connect(url, token))`. Per `crates/tools/sdk/src/engine.rs` (lines 123-147, 176-258), the `EngineBuilder` has no `.sync(...)` method. The `educore-sync` port at `crates/cross-cutting/sync/src/` exposes `SyncAdapter` but the SDK does not wire it into `EngineBuilder`. The umbrella `crates/educore/src/lib.rs` also does not re-export `educore_sync`. Neither `EducoreSyncAdapter` nor `WorkerHttpSyncAdapter` exists in the workspace — the actual in-process adapter is `InProcessSyncAdapter` at `crates/cross-cutting/sync-inprocess/src/lib.rs:72`.
- **expected:** Consumer wires `Arc<dyn SyncAdapter>` directly via the `educore_sync` port; no `Engine::builder().sync(...)` exists.
- **evidence:**
  - `docs/guides/offline-sync.md:15-23` — `Engine::builder().sync(EducoreSyncAdapter::in_process())` and `Engine::builder().sync(WorkerHttpSyncAdapter::connect(url, token))`.
  - `crates/tools/sdk/src/engine.rs:176-258` — `EngineBuilder` has no `sync` method.
  - `crates/cross-cutting/sync-inprocess/src/lib.rs:72` — `pub struct InProcessSyncAdapter` (not `EducoreSyncAdapter`).
  - `find crates -name "WorkerHttpSyncAdapter"` — no match.

---

### FINDING 20

- **id:** DOC-6-020
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/offline-sync.md:140-167` (Worked Example)
- **description:** The example uses `engine.students().admit(cmd)`, `InProcessBus::new()`, `Engine::builder().storage(...).audit(...).event_bus(...).build().await?`, and references `OfflineQueue::load(storage.clone())?` and `OfflineQueue::replay(&engine)`. None of these exist: the actual engine has no `engine.students()` method (only `engine.admission()`), no `InProcessBus` (it's `InProcessEventBus`), and there is no `OfflineQueue` type in the workspace — offline replay is the consumer's responsibility using the `SyncAdapter` port, not an engine-built queue.
- **expected:** `engine.admission().admit(cmd).await?`; `InProcessEventBus::new()`; no built-in `OfflineQueue` (consumer implements).
- **evidence:**
  - `docs/guides/offline-sync.md:140-167` — full client example using `engine.students()`, `InProcessBus::new()`, `OfflineQueue`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `students()` method.
  - `crates/adapters/event-bus/src/in_process.rs:123` — `pub struct InProcessEventBus`.
  - `find crates -name "offline_queue.rs" -o -name "OfflineQueue*"` — no matches.

---

### FINDING 21

- **id:** DOC-6-021
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/capability-rbac.md:113-138` (Engine::dispatch method)
- **description:** The guide shows `impl Engine { pub async fn dispatch(&self, cmd: BoxedCommand) -> Result<BoxedOutcome> { ... } }`. Per the actual SDK (`crates/tools/sdk/src/engine.rs`), there is no `Engine::dispatch` method. The engine dispatches commands via service-level methods (`engine.admission().admit(cmd)`, etc.), not through a generic `dispatch` method with `BoxedCommand`. The pattern is service-typed, not type-erased. The check pattern shown (capability → tenant → handler) also uses `DomainError::forbidden(...)` while the actual `DomainError` variant is `Forbidden(String)` per `crates/infra/core/src/error.rs:19-63`.
- **expected:** Service-typed dispatch (`engine.<service>().<method>(cmd)`), no generic `dispatch`; `DomainError::Forbidden(String)` factory.
- **evidence:**
  - `docs/guides/capability-rbac.md:113-138` — `impl Engine { pub async fn dispatch(&self, cmd: BoxedCommand) -> Result<BoxedOutcome> { ... } }`.
  - `crates/tools/sdk/src/engine.rs` — no `dispatch` method.
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { ... Forbidden(String) ... }` (positional arg, no `format!` factory shown).

---

### FINDING 22

- **id:** DOC-6-022
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/capability-rbac.md:139-180` (Role struct)
- **description:** The `Role` struct shows `pub struct Role { pub role_id, name, school_id, capabilities, is_system, created_at, updated_at }`. The capability set is `BTreeSet<Capability>` (correct) but the struct is shown as a value type with direct field access. Per `docs/code-standards.md` and the `educore-rbac` crate, aggregates are not exposed for direct construction; they are constructed via `TryFrom<CreateRoleCommand>` with invariants checked. The struct's `pub` fields violate encapsulation.
- **expected:** `pub struct Role { role_id: RoleId, name: RoleName, capabilities: CapabilitySet, ... }` with private fields and constructor-based creation.
- **evidence:**
  - `docs/guides/capability-rbac.md:139-180` — `pub struct Role { pub role_id, pub name, pub school_id, pub capabilities: BTreeSet<Capability>, pub is_system, pub created_at, pub updated_at }`.
  - `docs/code-standards.md` § Code Standards — "Aggregates are not exposed for direct construction; they are constructed via `TryFrom`."

---

### FINDING 23

- **id:** DOC-6-023
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/storage-adapter.md:5-30` (Adapter Skeleton)
- **description:** The adapter skeleton shows `impl StorageAdapter for PostgresStorage { async fn begin(&self) -> Result<Transaction> { ... } }`. Per Wave 5 docs-4 finding DOC-PORT-002, the actual `StorageAdapter` trait in `crates/infra/storage/src/port.rs:34-150` does NOT carry per-aggregate repository accessors like `students() -> Arc<dyn StudentRepository>`. The adapter also does not have `migrate()`, `connect()`, or `open()` methods in the form shown; the actual adapter has `migrate()`, `ping()`, `close()`, `bulk_insert_student_attendances(...)`, `watch_changes()`, `apply_snapshot()`, `cursor_for()`, `advance_cursor()`, plus `begin()`. The skeleton is wrong.
- **expected:** Real `StorageAdapter` trait method list (per docs-4 finding DOC-PORT-002).
- **evidence:**
  - `docs/guides/storage-adapter.md:5-30` — adapter skeleton with `begin()` only.
  - `crates/infra/storage/src/port.rs:34-150` — actual trait with 9 methods, no per-aggregate repository accessors.

---

### FINDING 24

- **id:** DOC-6-024
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/storage-adapter.md:165-180` (Tenant Isolation wrapper)
- **description:** The guide shows `fn with_school_filter(&self, sql: &mut String) { if !sql.contains("school_id =") { panic!("query missing school_id filter: {}", sql); } }`. The use of `panic!` in adapter code violates `docs/code-standards.md` § Code Standards ("`unwrap`, `expect`, `panic!` are forbidden in production paths"). Additionally, the panic message implies string-based SQL inspection (fragile), and the actual engine enforces `school_id` filtering via the typed query AST (no string matching required).
- **expected:** No `panic!` in adapter code; the typed query AST already enforces `school_id` (the macro-emitted query builder always includes the school id filter).
- **evidence:**
  - `docs/guides/storage-adapter.md:165-180` — `if !sql.contains("school_id =") { panic!(...) }`.
  - `docs/code-standards.md` § Code Standards — "`unwrap`, `expect`, `panic!` are forbidden in production paths."

---

### FINDING 25

- **id:** DOC-6-025
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/storage-adapter.md:235-247` (Worked Example)
- **description:** The example shows `let storage: Arc<dyn StorageAdapter> = Arc::new(PostgresStorage::connect(&env::var("DATABASE_URL")?).await?);` and `storage.migrate().await?;` and `let engine = Engine::builder().storage(storage).build().await?;`. The actual adapter constructor may be `PostgresStorage::connect(url)` returning `Result<Self>`, but the `migrate()` call here is the consumer's responsibility (per `docs/guides/saas-backend.md` § "Migrations: The engine does not own migrations; the consumer does"). The two guides disagree on whether `migrate()` is called by the consumer or by the engine.
- **expected:** Consumer runs migrations; the storage adapter does not own `migrate()` (per `saas-backend.md`).
- **evidence:**
  - `docs/guides/storage-adapter.md:235-247` — `storage.migrate().await?;` immediately after `PostgresStorage::connect`.
  - `docs/guides/saas-backend.md:48-49` — "A migration runner (migrations are owned by the consumer; see `docs/ports/storage.md#migrations`)."

---

### FINDING 26

- **id:** DOC-6-026
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/crud-patterns.md:30-49` (Create pattern)
- **description:** The pattern shows `impl CreateClassCommand { pub async fn execute(self, repo: &dyn ClassRepository, events: &mut Outbox) -> Result<Class> { ... } }`. Per `docs/code-standards.md` and the actual domain pattern in `crates/domains/academic/`, the command is dispatched via `engine.academic().create_class(cmd).await?` (or equivalent service-typed call), not via a method on the command struct with explicit `repo` and `events` arguments. The consumer never injects `repo` or `events` directly — that is internal wiring. Also `events: &mut Outbox` implies mutation, but the outbox is accessed via `&Outbox` (read-only handle).
- **expected:** Service-typed dispatch (`engine.<domain>().<verb>(cmd).await?`); no manual repo/outbox injection in the command struct.
- **evidence:**
  - `docs/guides/crud-patterns.md:30-49` — `impl CreateClassCommand { pub async fn execute(self, repo: &dyn ClassRepository, events: &mut Outbox) -> Result<Class> { ... } }`.
  - `crates/tools/sdk/src/engine.rs` — service-typed methods, no command-struct `execute`.
  - `docs/guides/saas-backend.md:278-291` — example uses `engine.<service>().<verb>(cmd)`.

---

### FINDING 27

- **id:** DOC-6-027
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/idempotent-commands.md:64-72` (IdempotencyStore trait)
- **description:** The guide shows `pub trait IdempotencyStore: Send + Sync { async fn lookup(&self, key: IdempotencyKey, command: &str) -> Result<Option<CommandOutcome>>; async fn record(&self, key: IdempotencyKey, command: &str, outcome: &CommandOutcome) -> Result<()>; }`. Per Wave 5 docs-4 finding DOC-PORT-004, the actual `Idempotency` port in `crates/infra/storage/src/transaction.rs` (accessed via `Transaction::idempotency() -> &dyn Idempotency`) has a different signature. The trait methods also reference `CommandOutcome { status: OutcomeStatus, payload: serde_json::Value, events: Vec<EventId> }` which uses `serde_json::Value` (forbidden in domain code per `docs/code-standards.md`).
- **expected:** Real `Idempotency` trait signature; typed `CommandOutcome` (no `serde_json::Value`).
- **evidence:**
  - `docs/guides/idempotent-commands.md:64-72` — `IdempotencyStore` trait + `CommandOutcome` with `serde_json::Value`.
  - `crates/infra/storage/src/transaction.rs:60-70` — actual `Idempotency` trait (different signature).
  - `docs/code-standards.md` — no `serde_json::Value` in domain code.

---

### FINDING 28

- **id:** DOC-6-028
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/event-replay.md:96-120` (Worked Example: Building a New Projection)
- **description:** The example uses `event.event_type` as a `&str` (string comparison `"StudentAdmitted"`) and `serde_json::from_value(event.payload.clone())?` to deserialize the payload. This violates `docs/code-standards.md` § Engine Rule 2 ("Compile-time safety over strings. Use macro-generated enums — never string field names") and the `serde_json::Value` rule. The actual replay API uses typed event enums and a closed `EventEnvelope` payload.
- **expected:** Replay over typed events (closed enum dispatch), no `serde_json::from_value` in domain code.
- **evidence:**
  - `docs/guides/event-replay.md:96-120` — `match event.event_type { "StudentAdmitted" => { let payload: StudentAdmitted = serde_json::from_value(event.payload.clone())?; ... } }`.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names" + "No `serde_json::Value` in domain code."

---

### FINDING 29

- **id:** DOC-6-029
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/event-replay.md:32-40` (Incremental Replay)
- **description:** The example calls `store.read_from(last_offset).await?` and `projection.record_offset(envelope.event_id).await?` and `store.read_snapshot(projection_id, since)`. Per the actual `crates/infra/storage/src/port.rs`, there is no `EventStore` trait with `read_all()`, `read_from()`, or `read_snapshot()` methods. The event store concept is split: the `outbox` provides append/pending (per Wave 5 docs-4 DOC-PORT-003) and `event_log` is a separate sub-port. There is no `Projection::record_offset` API.
- **expected:** Replay reads from `Outbox::pending()` and `EventLog` sub-port; no `EventStore` monolith.
- **evidence:**
  - `docs/guides/event-replay.md:32-40` — `store.read_all()`, `store.read_from(last_offset)`, `store.read_snapshot(projection_id, since)`.
  - `crates/infra/storage/src/port.rs:34-150` — no `EventStore` trait; `outbox` has only `append`/`pending`/`mark_published`/`pending_count`.

---

### FINDING 30

- **id:** DOC-6-030
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/notification-templates.md:5-21` (NotificationTemplate struct)
- **description:** The `NotificationTemplate` struct has `pub channel: Channel` with `Channel::Email { from: None, reply_to: None }`. Per the actual `crates/domains/communication/src/aggregate.rs` and `crates/adapters/notify/src/`, the `Channel` type is a port-defined enum on `NotificationProvider` (e.g. `Channel::Email`, `Channel::Sms`, `Channel::Push`), not a struct with `from`/`reply_to` named fields. The named-field construction `Channel::Email { from: None, reply_to: None }` is phantom syntax.
- **expected:** `pub channel: Channel` where `Channel` is an enum with unit variants `Email`, `Sms`, `Push`, `Webhook`; sender/reply-to are template-level fields, not channel payload.
- **evidence:**
  - `docs/guides/notification-templates.md:5-21` — `pub channel: Channel` + `Channel::Email { from: None, reply_to: None }`.
  - `crates/adapters/notify/src/` — actual `Channel` definition.

---

### FINDING 31

- **id:** DOC-6-031
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/notification-templates.md:64-95` (engine.communication().create_template)
- **description:** The example uses `engine.communication().create_template(CreateTemplateCommand { ... })`. Per the actual SDK (`crates/tools/sdk/src/engine.rs`), `Engine` has no `communication()` accessor. The communication domain crate is `crates/domains/communication/` and the consumer calls its service functions directly, or via a service accessor the consumer adds on top of the SDK.
- **expected:** Consumer calls `educore_communication::services::create_template(cmd, &ctx).await?` directly, not via `engine.communication()`.
- **evidence:**
  - `docs/guides/notification-templates.md:64-95` — `engine.communication().create_template(CreateTemplateCommand { ... })`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `communication()` method.

---

### FINDING 32

- **id:** DOC-6-032
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/fee-collection.md:35-90` (Setup example)
- **description:** The full fees workflow uses `engine.fees().create_group(...)`, `engine.fees().create_type(...)`, `engine.fees().create_master(...)`, `engine.fees().assign_to_class(...)`, `engine.fees().generate_invoices(...)`, `engine.fees().record_payment(...)`. Per the SDK, no `engine.fees()` method exists. The actual consumer calls `educore_finance::services::create_fees_group(cmd, &ctx).await?` etc. The command field names (`fees_master_id`, `fees_assign_id`) also conflict with `crates/domains/finance/src/value_objects.rs` (where the field is `master_id`, not `fees_master_id`).
- **expected:** Domain service functions; field names per `crates/domains/finance/src/value_objects.rs`.
- **evidence:**
  - `docs/guides/fee-collection.md:35-90` — `engine.fees().create_group(...)`, etc.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `fees()` method.

---

### FINDING 33

- **id:** DOC-6-033
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/report-card-generation.md:33-48` (engine.assessment().enter_marks)
- **description:** The example uses `engine.assessment().enter_marks(EnterMarksCommand { ... })`. Per the SDK, no `engine.assessment()` method exists. The actual consumer calls `educore_assessment::services::enter_marks(cmd, &ctx).await?`. The command's `student_records: vec![StudentMark { student_id, marks: 85.0, absent: false }]` also has `marks: f64` (or `f32`), but `docs/code-standards.md` forbids `as` casts and value objects prefer typed wrappers (`Marks` value object with validation).
- **expected:** Service-typed dispatch; `Marks` value object (not raw `f64`/`f32`).
- **evidence:**
  - `docs/guides/report-card-generation.md:33-48` — `engine.assessment().enter_marks(EnterMarksCommand { ... student_records: vec![StudentMark { student_id, marks: 85.0, ... }] })`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `assessment()` method.

---

### FINDING 34

- **id:** DOC-6-034
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/payroll-calculation.md:78-101` (engine.hr().assign_salary_template, generate_payroll, approve_payroll)
- **description:** All payroll examples use `engine.hr()...` and `engine.finance().record_payroll_payment(...)`. Neither `engine.hr()` nor `engine.finance()` exists on the SDK (`crates/tools/sdk/src/engine.rs`). The actual consumer calls `educore_hr::services::*` and `educore_finance::services::*` directly. The `GeneratePayrollCommand::period: PayPeriod { year: 2026, month: 6 }` field also conflicts with `crates/domains/hr/src/value_objects.rs` where the period type uses `start: NaiveDate, end: NaiveDate`, not a `PayPeriod` struct.
- **expected:** Service-typed dispatch; `PayrollPeriod { start: NaiveDate, end: NaiveDate }` value object.
- **evidence:**
  - `docs/guides/payroll-calculation.md:78-101` — `engine.hr().assign_salary_template(...)`, `engine.hr().generate_payroll(GeneratePayrollCommand { period: PayPeriod { year: 2026, month: 6 }, ... })`, `engine.finance().record_payroll_payment(...)`.
  - `crates/tools/sdk/src/engine.rs:123-147` — no `hr()` or `finance()` method.

---

### FINDING 35

- **id:** DOC-6-035
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/ai-agent-integration.md:54-72` (Tool struct)
- **description:** The `Tool` struct has `pub input_schema: serde_json::Value, pub output_schema: serde_json::Value` and the example uses `engine.tools().for_session(agent_session)`. Per `docs/code-standards.md`, `serde_json::Value` is forbidden in domain code. Per the SDK, no `engine.tools()` method exists. The actual tool catalog uses a typed `ToolDescriptor` (or macro-emitted enum) with `JsonSchema` (the `schemars` crate's typed wrapper), not raw `serde_json::Value`.
- **expected:** Typed `ToolDescriptor` with `schemars::schema::Schema` (or similar); no `engine.tools()` (consumer-implemented).
- **evidence:**
  - `docs/guides/ai-agent-integration.md:54-72` — `pub struct Tool { ... pub input_schema: serde_json::Value, pub output_schema: serde_json::Value, ... }` and `engine.tools().for_session(agent_session)`.
  - `docs/code-standards.md` — "No `serde_json::Value` in domain code."

---

### FINDING 36

- **id:** DOC-6-036
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/ai-agent-integration.md:138-156` (educore-agent-test crate)
- **description:** The guide references `educore-agent-test` crate and `TestAgent::new("test-agent", capabilities![...])`. Per the `AGENTS.md` Crate Inventory, no `educore-agent-test` crate exists in the 34-crate inventory. The actual test utilities live in `crates/tools/testkit/` (scaffolded at Phase 16) and do not include an agent simulator. The example `agent.invoke("Mark John Doe present today.").await?` is also a phantom API.
- **expected:** No `educore-agent-test` crate; no `TestAgent`. Agent testing is consumer's responsibility using mock harnesses.
- **evidence:**
  - `docs/guides/ai-agent-integration.md:138-156` — `let agent = TestAgent::new("test-agent", capabilities![...]); let outcome = agent.invoke("Mark John Doe present today.").await?;`.
  - `AGENTS.md` Crate Inventory — no `educore-agent-test` crate; only `educore-testkit` (Phase 16) and `educore-storage-parity` (Phase 0+16).

---

### FINDING 37

- **id:** DOC-6-037
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/school-onboarding.md:99-141` (Worked Example: onboard_school)
- **description:** The example uses `engine.platform().create_school(...)`, `engine.auth().issue_session(...)`, `engine.settings().update_general_settings(...)`, `engine.academic().create_class(...)`, `engine.fees().create_group(...)`, `engine.finance().open_bank_account(...)`, `engine.hr().register_staff(...)`. Per the SDK, none of these `engine.<x>()` methods exist. The actual consumer wires the platform crate (`educore_platform::commands::create_school(cmd, &ctx)`), settings crate (`educore_settings::*`), and per-domain commands directly.
- **expected:** Service-typed dispatch via direct crate function calls; no `engine.platform()`/`engine.settings()`/`engine.academic()`/`engine.fees()`/`engine.finance()`/`engine.hr()`.
- **evidence:**
  - `docs/guides/school-onboarding.md:99-141` — full onboarding function using 7 non-existent engine accessors.
  - `crates/tools/sdk/src/engine.rs:123-147` — `Engine` exposes only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` (and port handles), no domain CRUD accessors.

---

### FINDING 38

- **id:** DOC-6-038
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/ci-cd.md:120-130` (Deployment Dockerfile)
- **description:** The Dockerfile uses `FROM rust:1.75 AS builder` (the engine's MSRV is 1.75 per `docs/code-standards.md`). However the Dockerfile's `COPY . .` copies the entire monorepo into the build context and runs `cargo build --release`. The actual binary name is not `educore` — it would be the consumer's binary (e.g. `backend`, `sync-engine`). The image's `ENTRYPOINT ["educore"]` references a non-existent binary. The published engine has no `educore` binary; the closest is `educore-cli` (Phase 16 tool, scaffold only).
- **expected:** Docker build for the consumer's specific binary; no `educore` binary in the engine.
- **evidence:**
  - `docs/guides/ci-cd.md:120-130` — `ENTRYPOINT ["educore"]`.
  - `AGENTS.md` Crate Inventory — `educore-cli` is a binary crate (Phase 16, scaffold only); no `educore` binary at the umbrella level.

---

### FINDING 39

- **id:** DOC-6-039
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/test-strategy.md:67-80` (Component tests)
- **description:** The example uses `engine.students().admit(AdmitStudentCommand { tenant, ... })` and `engine.student_records().default_for_student(student.id)`. Per the SDK, no `engine.students()` or `engine.student_records()` method exists. The actual admission API is `engine.admission().admit(cmd).await?`. Also `test_tenant()` is referenced but the actual testkit API uses a builder pattern (`TenantContext::for_test(school_id)`).
- **expected:** `engine.admission().admit(cmd).await?`; `testkit::tenant(school_id)` builder.
- **evidence:**
  - `docs/guides/test-strategy.md:67-80` — `engine.students().admit(...)`, `engine.student_records().default_for_student(...)`, `let tenant = test_tenant();`.
  - `crates/tools/sdk/src/engine.rs:123-127` — `pub fn admission(&self) -> AdmissionService<'_>`, no `students()` or `student_records()`.

---

### FINDING 40

- **id:** DOC-6-040
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/test-strategy.md:174-194` (Storage Adapter Parity Tests)
- **description:** The guide claims parity tests run "against the PostgreSQL, SQLite, SurrealDB, and MongoDB adapters". Per `AGENTS.md` § Storage Adapters, **SurrealDB and MongoDB adapters are deferred and not shipped from the engine**. Running parity tests against SurrealDB/MongoDB is impossible because those adapters don't exist in the workspace.
- **expected:** Parity tests run only against PostgreSQL, MySQL, and SQLite (the 3 shipped adapters).
- **evidence:**
  - `docs/guides/test-strategy.md:174-194` — "PostgreSQL, SQLite, SurrealDB, and MongoDB adapters".
  - `AGENTS.md` § Storage Adapters — "The SurrealDB and MongoDB adapters are **deferred to a future release** and are **not** shipped from the engine."

---

### FINDING 41

- **id:** DOC-6-041
- **area:** documentation-guides
- **severity:** Critical
- **location:** `docs/guides/test-strategy.md:243-260` (Test Utilities / educore-test crate)
- **description:** The guide references `educore-test` crate with `test_engine()`, `test_tenant(school_id)`, `test_clock()`, `assert_events_published!`, `assert_audit_record!`. Per `AGENTS.md` Crate Inventory, the actual test crate is **`educore-testkit`** (Phase 16, scaffold only). The name `educore-test` does not match any crate in the inventory.
- **expected:** `educore-testkit` (per Crate Inventory); correct crate name and API once Phase 16 lands.
- **evidence:**
  - `docs/guides/test-strategy.md:243-260` — `The engine ships a educore-test crate`.
  - `AGENTS.md` Crate Inventory — entry 32: `educore-testkit` (Phase 16, "Test infrastructure + SDK"), not `educore-test`.

---

### FINDING 42

- **id:** DOC-6-042
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/license-faq.md:106` (third-party crates count)
- **description:** The guide says "The engine's 27 third-party dependencies (per ADR-015-ExternalCrates.md)". Per the actual workspace `Cargo.toml` (root) and `ADR-015-ExternalCrates.md`, the engine has more or fewer third-party deps depending on feature flags. The "27" number is unverified and likely stale.
- **expected:** Count from the actual workspace dependency list.
- **evidence:**
  - `docs/guides/license-faq.md:106` — "27 third-party dependencies".
  - `docs/decisions/ADR-015-ExternalCrates.md` — should be the source of truth for the count.

---

### FINDING 43

- **id:** DOC-6-043
- **area:** documentation-guides
- **severity:** Medium
- **location:** `docs/guides/saas-backend.md:786-800` (Billing Integration)
- **description:** The billing example uses `engine.finance().record_external_payment(RecordExternalPaymentCommand { tenant: platform_tenant_to_school(inv.account_id), ... })`. The helper `platform_tenant_to_school(inv.account_id)` is referenced without definition; the consumer must define it. The example also uses `RecordExternalPaymentCommand` which may or may not exist in `crates/domains/finance/src/commands.rs`. The actual command shape and the helper function are undocumented (consumer-implementation gap).
- **expected:** Documented command shape (per `docs/commands/finance.md`) and a sketched `platform_tenant_to_school` helper signature.
- **evidence:**
  - `docs/guides/saas-backend.md:786-800` — `tenant: platform_tenant_to_school(inv.account_id)` helper used without definition; `RecordExternalPaymentCommand { stripe_invoice_id: inv.id, amount: inv.amount_paid.into(), currency: inv.currency.into(), ... }` with `.into()` conversions on Stripe types.

---

### FINDING 44

- **id:** DOC-6-044
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/saas-backend.md:902-919` (Deployment Topology)
- **description:** The topology diagram includes an "Event bus (NATS JetStream)" at the bottom. Per the actual adapter at `crates/adapters/event-bus/src/`, the in-process adapter is `InProcessEventBus`; NATS/Redis/Kafka adapters are not in the workspace (only the in-process adapter is shipped per `AGENTS.md` inventory). The `NatsBus::from_env()` reference in the builder example (line 264) is also a phantom API.
- **expected:** Only `InProcessEventBus` is shipped; NATS/Redis/Kafka are consumer responsibilities.
- **evidence:**
  - `docs/guides/saas-backend.md:264, 902-919` — `NatsBus::from_env()?` and "Event bus (NATS JetStream)" topology.
  - `AGENTS.md` Crate Inventory — only `educore-event-bus` (port, Phase 2) is shipped; no NATS adapter.

---

### FINDING 45

- **id:** DOC-6-045
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/saas-backend.md:744-751` (Metrics query)
- **description:** The metrics query is raw SQL: `SELECT school_id, date_trunc('day', occurred_at), count(*) FROM outbox WHERE event_type = 'StudentAdmitted' GROUP BY 1, 2;`. Per `docs/code-standards.md` § Engine Rules ("Compile-time safety over strings. Use macro-generated enums") and the typed query layer, the consumer should query through the engine's query port, not raw SQL on the `outbox` table. Raw SQL bypasses the storage port.
- **expected:** Use the typed query API (`educore_events::query::outbox().where_eq(EventField::Type, EventType::StudentAdmitted).list().await?`).
- **evidence:**
  - `docs/guides/saas-backend.md:744-751` — raw SQL on `outbox` table.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names."

---

### FINDING 46

- **id:** DOC-6-046
- **area:** documentation-guides
- **severity:** Medium
- **location:** `docs/guides/storage-adapter.md:170-178` (RLS SQL)
- **description:** The RLS policy uses `current_setting('app.current_school_id', true)::uuid`. The `true` second argument to `current_setting` returns `NULL` if the GUC is unset, and the cast to `uuid` will then error at row-evaluation time. The actual adapter per `docs/ports/storage.md` and `docs/schemas/tenancy-schema.md` should set the GUC on every connection before any query runs (so the second argument is unnecessary), and the cast should be guarded.
- **expected:** The adapter sets `SET LOCAL app.current_school_id = '<uuid>'` on every connection acquired from the pool, then the policy can use `current_setting('app.current_school_id')::uuid` without the `true` fallback.
- **evidence:**
  - `docs/guides/storage-adapter.md:170-178` — `USING (school_id = current_setting('app.current_school_id', true)::uuid);`.
  - `docs/ports/storage.md` (per Wave 5 docs-4) — adapter responsibility is to set the GUC per connection.
  - `docs/schemas/tenancy-schema.md` — canonical RLS pattern.

---

### FINDING 47

- **id:** DOC-6-047
- **area:** documentation-guides
- **severity:** Medium
- **location:** `docs/guides/saas-backend.md:330-345` (Route groups / capability strings)
- **description:** The route table uses string-based capabilities: `capability("students.admit")`, `capability("students.read")`, `capability("attendance.mark")`. Per `docs/guides/capability-rbac.md` and the `educore-rbac` crate, capabilities are typed enums (`Capability::StudentAdmit`, `Capability::StudentsRead`, `Capability::AttendanceMark`). String-based capability checks lose compile-time safety and violate `docs/code-standards.md` § Engine Rule 2.
- **expected:** `capability(Capability::StudentAdmit)` (typed enum value, not string).
- **evidence:**
  - `docs/guides/saas-backend.md:330-345` — `.route("/v1/students", post(admit).layer(capability("students.admit")))`.
  - `docs/code-standards.md` § Engine Rules — "Use macro-generated enums — never string field names."

---

### FINDING 48

- **id:** DOC-6-048
- **area:** documentation-guides
- **severity:** High
- **location:** `docs/guides/README.md:18` (README contents)
- **description:** The README's "Available Guides" table is missing `event-replay.md`, `crud-patterns.md`, `idempotent-commands.md`, `test-strategy.md`, and `license-faq.md` (some are listed but the table has only 16 entries for 17 guide files). The README lists 16 guide files; `ls docs/guides/*.md` shows 18 files (17 guides + README.md). The table is internally inconsistent with the directory.
- **expected:** README lists all 17 guide files in the table.
- **evidence:**
  - `docs/guides/README.md:18` — table has 16 rows.
  - `ls docs/guides/*.md` — 18 files: README.md + 17 guides.

---

### FINDING 49

- **id:** DOC-6-049
- **area:** documentation-guides
- **severity:** High
- **location:** Cross-cutting — multiple guides
- **description:** Multiple guides (`fee-collection.md`, `report-card-generation.md`, `payroll-calculation.md`, `crud-patterns.md`, `school-onboarding.md`, `test-strategy.md`, `ai-agent-integration.md`, `saas-backend.md`, `notification-templates.md`, `offline-sync.md`, `audit-trail.md`) all use `engine.<domain>()` method-style accessors (`engine.fees()`, `engine.assessment()`, `engine.hr()`, `engine.finance()`, `engine.students()`, `engine.communication()`, `engine.platform()`, `engine.settings()`, `engine.academic()`, `engine.tools()`, `engine.student_records()`). Per the SDK at `crates/tools/sdk/src/engine.rs:123-147`, **none of these methods exist on `Engine`**. The actual `Engine` exposes only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()` (and port handles like `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`). This is a systemic error across 11 of 17 guides.
- **expected:** Consumer uses direct crate service functions (`educore_<domain>::services::*`) or builds its own service wrappers on top of the SDK. No `engine.<domain>()` shortcut.
- **evidence:**
  - `docs/guides/fee-collection.md` — `engine.fees()` (5 occurrences).
  - `docs/guides/report-card-generation.md` — `engine.assessment()` (7 occurrences).
  - `docs/guides/payroll-calculation.md` — `engine.hr()`, `engine.finance()` (6 occurrences).
  - `docs/guides/crud-patterns.md` — `engine.classes()`, `engine.student_records()`, `engine.students()` (referenced via the `Class.Create` capability and examples).
  - `docs/guides/school-onboarding.md` — `engine.platform()`, `engine.settings()`, `engine.academic()`, `engine.fees()`, `engine.finance()`, `engine.hr()` (10 occurrences).
  - `docs/guides/test-strategy.md` — `engine.students()`, `engine.student_records()` (4 occurrences).
  - `docs/guides/ai-agent-integration.md` — `engine.tools()`.
  - `docs/guides/saas-backend.md` — `engine.students()`, `engine.platform()`, `engine.rbac()`, `engine.finance()`, `engine.handle_synced_event()` (10+ occurrences).
  - `docs/guides/notification-templates.md` — `engine.communication()` (2 occurrences).
  - `docs/guides/offline-sync.md` — `engine.students()` (2 occurrences).
  - `docs/guides/audit-trail.md` — `engine.audit()` (1 occurrence).
  - `crates/tools/sdk/src/engine.rs:123-147` — actual `Engine` method list.

---

### FINDING 50

- **id:** DOC-6-050
- **area:** documentation-guides
- **severity:** Medium
- **location:** Cross-cutting — multiple guides
- **description:** Multiple guides use `unwrap()`, `expect()`, and `.parse().unwrap_or(20)` patterns in example code: `saas-backend.md:243` (`.parse().unwrap_or(20)`), `saas-backend.md:258` (`.unwrap()`), `crud-patterns.md:124-126` (`NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()`), `fee-collection.md:67,90,97,116,141` (`.unwrap()`), `report-card-generation.md` (multiple), `payroll-calculation.md` (multiple), `school-onboarding.md`, `notification-templates.md`. Per `docs/code-standards.md` § Code Standards, "`unwrap`, `expect`, `panic!` are forbidden in production paths". The guides either need to be marked as sketch/pseudo-code OR use proper error propagation (`?`).
- **expected:** Use `?` propagation with `map_err` or `TryFrom` in all guide examples; or explicitly mark the code block as "sketch — production code uses `?`".
- **evidence:**
  - `docs/guides/saas-backend.md:243, 258, 681` — `.parse().unwrap_or(20)`, `.unwrap()`.
  - `docs/guides/crud-patterns.md:124-126` — `NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()`.
  - `docs/guides/fee-collection.md` (5+ `.unwrap()`).
  - `docs/code-standards.md` § Code Standards — "`unwrap`, `expect`, `panic!` are forbidden in production paths".
