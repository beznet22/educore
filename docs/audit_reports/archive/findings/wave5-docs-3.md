# Wave 5 — Documentation Audit (Docs Group 3)

**Scope:** `docs/library-docs.md`, `docs/query_layer.md`, `docs/handoff/PHASE-*-HANDOFF.md` (all 17).

**Audit date:** 2026-06-23.

**Checks performed:**
1. `library-docs.md` claims vs the actual `Engine` API surface (`crates/tools/sdk/src/{engine.rs,facade.rs}` + every adapter and domain crate).
2. `query_layer.md` claims vs the actual `#[derive(DomainQuery)]` macro emission (`crates/infra/query-derive/src/lib.rs`) and the runtime AST (`crates/infra/core/src/query.rs`).
3. Handoff headline counts vs the actual counts in the matching `crates/domains/<d>/src/*.rs` files (aggregates, commands, events, services, tests).

---

### FINDING 1

- **id:** DOC-LIB-001
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:181-188`
- **description:** The "Common Workflows" section claims `engine.students().admit(cmd).await?` is the consumer API for admitting a student. No `Engine::students()` method exists; the SDK's `Engine` exposes `engine.admission()` (returning `AdmissionService`), not `engine.students()`. The actual `admit` flow is `educore_academic::services::admit_student(cmd, &clock, &ids, &uniqueness)`, a free function with a 4-arg signature — not a method on any engine struct.
- **expected:** `engine.students().admit(cmd).await?` per `docs/library-docs.md:181`.
- **evidence:**
  - `docs/library-docs.md:181` — `engine.students().admit(cmd).await?` — admit a student.
  - `crates/tools/sdk/src/engine.rs:123-127` — `/// Returns a handle to the admission facade.` / `pub fn admission(&self) -> AdmissionService<'_> { AdmissionService::new(self) }` — only `admission()`, no `students()`.
  - `crates/tools/sdk/src/engine.rs:128-147` — `Engine` exposes `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`; no `students()` / `assessment()` / `hr()` / `fees()`.

---

### FINDING 2

- **id:** DOC-LIB-002
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:182-188`
- **description:** The "Common Workflows" list includes `engine.assessment()`, `engine.fees()`, and `engine.hr()` as consumer entry points. None of these exist on the `Engine` struct. The SDK exposes only `admission()` (academic), `attendance()`, `payment_svc()`, and `notify_svc()` facade services — there are no facade methods for assessment, fees, hr, or students.
- **expected:** `engine.assessment().enter_marks(cmd).await?`, `engine.fees().generate_invoice(cmd).await?`, `engine.hr().generate_payroll(cmd).await?` per `docs/library-docs.md:184-188`.
- **evidence:**
  - `docs/library-docs.md:184` — `engine.assessment().enter_marks(cmd).await?` — enter marks.
  - `docs/library-docs.md:186` — `engine.fees().generate_invoice(cmd).await?` — generate a fees invoice.
  - `docs/library-docs.md:188` — `engine.hr().generate_payroll(cmd).await?` — generate monthly payroll.
  - `crates/tools/sdk/src/engine.rs:123-146` — `Engine` has no `assessment()`, `fees()`, or `hr()` method; only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`.

---

### FINDING 3

- **id:** DOC-LIB-003
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:154-165, 169-176`
- **description:** The doc claims `engine.events().subscribe::<StudentAdmitted>().await?` and `engine.rbac().has_capability(...)`. No `Engine::events()` or `Engine::rbac()` method exists; the event bus is exposed as `engine.bus()` (returning `&Arc<dyn EventBus>`), and there is no RBAC accessor at all on `Engine`. The subscribe signature itself is also wrong: `subscribe` takes `SubscribeOptions` (with `consumer`, `topic`, `filter`, `start`, `batch_size`, `visibility_timeout`) and returns `Box<dyn EventSubscription>` — not a turbofish-only generic call returning a stream.
- **expected:** `engine.events().subscribe::<StudentAdmitted>().await?` per `docs/library-docs.md:157-160`; `engine.rbac().has_capability(...)` per `docs/library-docs.md:171-172`.
- **evidence:**
  - `docs/library-docs.md:157-160` — `let mut sub = engine.events().subscribe::<StudentAdmitted>().await?;`
  - `docs/library-docs.md:171-172` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? {`
  - `crates/tools/sdk/src/engine.rs:104-109` — `/// Returns a reference to the event bus.` / `pub fn bus(&self) -> &Arc<dyn EventBus> { &self.bus }` — only `bus()`, no `events()`.
  - `crates/tools/sdk/src/engine.rs:42-147` — no `rbac()` accessor on `Engine`.
  - `crates/cross-cutting/events/src/event_bus.rs:48` — `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;` — the actual signature, not a generic-turbofish call.

---

### FINDING 4

- **id:** DOC-LIB-004
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:9-26`
- **description:** The "Construction" example uses `Engine::builder()`, `.build().await?`, `JwtAuthProvider::from_env()?`, `EmailNotifier::from_env()?`, `InProcessBus::new()`, and `UuidV7Generator::new()`. None of these identifiers exist: the actual builder is `EngineBuilder::new()`, the build is synchronous (`pub fn build(self) -> Result<Engine, SdkError>`), the JWT provider is `JwtAuthProviderBuilder::new().build()` (no `from_env()`), the notifier is `EmailProviderBuilder::new()` (the struct is `EmailProvider`, not `EmailNotifier`, and there is no `from_env()`), the bus is `InProcessEventBus::new()` (no `InProcessBus`), and the id generator is `SystemIdGen` (a unit struct, no `new()`).
- **expected:** `Engine::builder().storage(...).auth(JwtAuthProvider::from_env()?)....build().await?` per `docs/library-docs.md:9-26`.
- **evidence:**
  - `docs/library-docs.md:14-22` — full builder example using `Engine::builder()`, `.build().await?`, `JwtAuthProvider::from_env()?`, `EmailNotifier::from_env()?`, `InProcessBus::new()`, `UuidV7Generator::new()`.
  - `crates/tools/sdk/src/engine.rs:179-191` — `pub fn new() -> Self { Self { storage: None, ... } }` — the constructor is `EngineBuilder::new()`, not `Engine::builder()`.
  - `crates/tools/sdk/src/engine.rs:258` — `pub fn build(self) -> Result<Engine, SdkError> { ... }` — build is sync, returns `SdkError`, not `await`-able.
  - `crates/adapters/auth/src/jwt.rs:161-167, 224-225` — builder constructor is `JwtAuthProviderBuilder::new()`, then `.build()`; no `JwtAuthProvider::from_env()`.
  - `crates/adapters/notify/src/email.rs:75, 204-217, 261` — `pub struct EmailProvider` (not `EmailNotifier`), with `EmailProviderBuilder::new()`; no `from_env()`.
  - `crates/adapters/event-bus/src/in_process.rs:123, 161` — `pub struct InProcessEventBus` (not `InProcessBus`); `InProcessEventBus::new()` exists.
  - `crates/infra/core/src/clock.rs:143` — `pub struct SystemIdGen;` (a unit struct with `impl IdGenerator for SystemIdGen`); no `UuidV7Generator`.

---

### FINDING 5

- **id:** DOC-LIB-005
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:33-39, 169-176`
- **description:** The "Tenant Context" and "Capability Check" examples construct `TenantContext::new(session.school_id(), session.user_id())` and reference `Capability::StudentAdmit`. Neither constructor nor variant exists. The actual constructor is `TenantContext::for_user(school_id, actor_id, correlation_id, user_type)` and the academic-admission capability variant is `Capability::AcademicStudentCreate`.
- **expected:** `TenantContext::new(session.school_id(), session.user_id())` and `Capability::StudentAdmit` per `docs/library-docs.md:38` and `:172`.
- **evidence:**
  - `docs/library-docs.md:38` — `let tenant = TenantContext::new(session.school_id(), session.user_id());`
  - `docs/library-docs.md:172` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? {`
  - `crates/infra/core/src/tenant.rs:56-75` — `impl TenantContext { pub fn for_user(school_id: SchoolId, actor_id: UserId, correlation_id: CorrelationId, user_type: UserType) -> Self {...} }` — no `TenantContext::new(SchoolId, UserId)`.
  - `crates/cross-cutting/rbac/src/value_objects.rs:73, 75, 77, 79` — `AcademicStudentCreate, AcademicStudentRead, AcademicStudentUpdate, AcademicStudentDelete` — the academic student capabilities; no `StudentAdmit` variant.

---

### FINDING 6

- **id:** DOC-LIB-006
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:43-68`
- **description:** The "Calling a Command" example constructs `AdmitStudentCommand` with fields `admission_no: AdmissionNumber`, `guardian: GuardianSpec { ... full_name, relation: GuardianRelation::Mother, phone: PhoneNumber, email: EmailAddress }`, `class_id: ClassId::new(tenant.school_id())`, `section_id: SectionId::new(tenant.school_id())`, `academic_year: AcademicYear::current(tenant.school_id(), &clock)`. None of those types or constructors exist in the academic crate, the admission flow does not take a guardian, and `ClassId` / `SectionId` / `AcademicYearId` are typed ids that already embed `SchoolId` (so `new(tenant.school_id())` is doubly wrong: `Id::new` takes `(school_id, uuid)`). The actual `AdmitStudentCommand` requires `student_id`, `admission_no: String`, `date_of_birth`, `gender`, `admission_date`, `class_id`, `section_id`, `academic_year_id` (a typed id), etc. — there is no `AcademicYear::current` and no `GuardianSpec`.
- **expected:** `engine.students().admit(AdmitStudentCommand { admission_no: AdmissionNumber::new("ADM-2026-0001")?, ..., guardian: GuardianSpec { ... }, class_id: ClassId::new(tenant.school_id()), ... academic_year: AcademicYear::current(...), })` per `docs/library-docs.md:43-65`.
- **evidence:**
  - `docs/library-docs.md:48-64` — the full `AdmitStudentCommand { ... }` literal.
  - `crates/domains/academic/src/commands.rs:62-106` — `pub struct AdmitStudentCommand` with fields `tenant: TenantContext, student_id: StudentId, admission_no: String, first_name: String, last_name: String, date_of_birth: NaiveDate, gender: ..., blood_group: Option<...>, religion: Option<String>, caste: Option<String>, mobile: Option<String>, email: Option<String>, current_address: Option<String>, permanent_address: Option<String>, admission_date: NaiveDate, class_id: ClassId, section_id: SectionId, academic_year_id: AcademicYearId, roll_no: Option<String>, custom_fields: BTreeMap<String, String>`. There is no `guardian: GuardianSpec` field and no `AdmissionNumber` field (it is `String`).
  - `crates/domains/academic/src/value_objects.rs:264-294` — `AdmissionNumber` exists as a wrapper type with `pub fn new(s: impl Into<String>) -> Result<Self>`, but the command takes `String`, not `AdmissionNumber`.
  - `crates/domains/academic/src/commands.rs:112-147` — `AdmitStudentCommand::new(...)` takes `tenant, student_id, admission_no: String, first_name, last_name, date_of_birth, gender, admission_date, class_id, section_id, academic_year_id` — no guardian, no `AcademicYear::current`.
  - `crates/domains/academic/src/lib.rs:67, 86-92` — no `GuardianSpec`, `GuardianRelation`, `AcademicYear::current`, `PhoneNumber`, `EmailAddress` re-exported from the academic crate.

---

### FINDING 7

- **id:** DOC-LIB-007
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:190-201`
- **description:** The "Error Handling" example pattern-matches `DomainError::Validation { field, reason }`, `DomainError::Conflict { entity, reason }`, `DomainError::NotFound { entity, id }`, `DomainError::Forbidden { reason }`, `DomainError::Infrastructure(source)`. The actual `DomainError` enum has none of these tuple-struct-style or named-struct-style variants — every variant takes a single `String` (or a boxed error for `Infrastructure`). Variants `Validation(String)`, `NotFound(String)`, `Conflict(String)`, `Forbidden(String)`, `TenantViolation(String)`, `Infrastructure(Box<dyn Error + Send + Sync>)`, `NotSupported(String)`.
- **expected:** `Err(DomainError::Validation { field, reason })` and friends per `docs/library-docs.md:195-199`.
- **evidence:**
  - `docs/library-docs.md:195-199` — `Err(DomainError::Validation { field, reason })`, `Err(DomainError::Conflict { entity, reason })`, `Err(DomainError::NotFound { entity, id })`, `Err(DomainError::Forbidden { reason })`, `Err(DomainError::Infrastructure(source))`.
  - `crates/infra/core/src/error.rs:18-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(Box<dyn Error + Send + Sync>), NotSupported(String) }` — every body is a single `String` or boxed error; no struct-like named-field variants.

---

### FINDING 8

- **id:** DOC-LIB-008
- **area:** documentation
- **severity:** Medium
- **location:** `docs/library-docs.md:208-218`
- **description:** The "Sample Programs" section claims an `examples/admit_and_enroll.rs` exists in the workspace. No such file exists anywhere in the repo (the only `examples/` directories are inside `target/` build outputs and inside `schoolify/vendor/`).
- **expected:** "A complete `examples/admit_and_enroll.rs` is provided in the workspace that..." per `docs/library-docs.md:210`.
- **evidence:**
  - `docs/library-docs.md:210-218` — the sample-programs claim.
  - No `examples/admit_and_enroll.rs` anywhere in the repo: `find /home/beznet/Workspace/smscore -name "examples" -type d` returns only `target/debug/examples` and `schoolify/vendor/**/examples`. No crate under `crates/` ships an `examples/` directory.

---

### FINDING 9

- **id:** DOC-LIB-009
- **area:** documentation
- **severity:** Medium
- **location:** `docs/library-docs.md:53, 58-59`
- **description:** The "Calling a Command" example uses `NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` directly inline. The engine's code standards forbid `unwrap()` in production paths (per `AGENTS.md` § "Type Safety" and `docs/code-standards.md`); the example contradicts the engine's own invariant.
- **expected:** `NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` per `docs/library-docs.md:53`.
- **evidence:**
  - `docs/library-docs.md:53` — `date_of_birth: NaiveDate::from_ymd_opt(2010, 12, 10).unwrap(),`
  - `AGENTS.md` § "Type Safety" — "No `unwrap()` or `expect()` in production paths. Propagate errors via `?` or document the invariant that makes panic impossible."

---

### FINDING 10

- **id:** DOC-LIB-010
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:84-150`
- **description:** The library-doc query examples assume the `#[derive(DomainQuery)]` macro emits a generic `where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) })` plus `.active()` / `.in_class(class_id)` extension traits. The macro does emit `where_has_Parent(...)` as a per-relation typed method (no relation parameter), and `.active()` / `.in_class()` are not defined anywhere in the academic crate — they would have to be developer-authored extension traits per the query_layer spec. The library-doc example also assumes a `Student { ..., parent: Option<Parent>, }` field that does not exist on the `Student` aggregate.
- **expected:** `engine.students().query().active().where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) }).order_by(StudentField::LastName).page(0, 50).await?` with `student.parent` per `docs/library-docs.md:89-99` and `:115-122`.
- **evidence:**
  - `docs/library-docs.md:89-99` — the full `.active() / .in_class() / .where_has(StudentRelation::Parent, ...)` chained query.
  - `crates/domains/academic/src/aggregate.rs:56-119` — `pub struct Student { ... }` has no `parent: Option<Parent>` field; the `Student` aggregate carries only own-data fields (`id, school_id, admission_no, first_name, last_name, date_of_birth, gender, blood_group, ..., custom_fields, version, etag, created_at, ...`).
  - `crates/infra/query-derive/src/lib.rs:602-632` — the macro emits one typed method per relation (e.g. `where_has_Parent`), NOT a generic `where_has(StudentRelation::Parent, |p| { ... })`.
  - `crates/domains/academic/src/query.rs:26-122` — the academic `StudentQuery` is a hand-written stub with `with_status / with_class_id / ...` setters; no `.active()` / `.in_class()` / `.where_has_*()` methods exist.

---

### FINDING 11

- **id:** DOC-LIB-011
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:70-82`
- **description:** The "Querying" prose claims "there is no hand-written `StudentField` in the consumer codebase, the type is generated from the struct definition on every compile" and that "the macro produces a structurally complete but semantically neutral builder". This contradicts the academic crate, which ships a hand-written `StudentQuery` struct (not macro-generated) at `crates/domains/academic/src/query.rs:28-43`, with hand-written setter methods, and explicitly defers the macro-generated builder to a later phase.
- **expected:** Per `docs/library-docs.md:74-82`: "the macro emits a typed `*Field` enum and a `*QueryBuilder` state struct per aggregate — there is no hand-written `StudentField` in the consumer codebase".
- **evidence:**
  - `docs/library-docs.md:74-77` — "the macro emits a typed `*Field` enum and a `*QueryBuilder` state struct per aggregate — there is no hand-written `StudentField` in the consumer codebase, the type is generated from the struct definition on every compile."
  - `crates/domains/academic/src/query.rs:27-122` — `pub struct StudentQuery { pub status_filter: Option<StudentStatus>, pub class_id_filter: Option<ClassId>, pub section_id_filter: Option<SectionId>, pub academic_year_id_filter: Option<AcademicYearId>, pub first_name_contains: Option<String>, pub last_name_contains: Option<String>, pub admission_no_contains: Option<String>, }` — a hand-written struct, not macro-generated.
  - `crates/domains/academic/src/query.rs:115-121` — the `execute` method returns `Err(DomainError::not_supported("StudentQuery::execute is a Phase 3 stub; the typed query executor lands in Phase 4+"))` — explicitly a stub.

---

### FINDING 12

- **id:** DOC-QL-001
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:241-245` vs `crates/infra/query-derive/src/lib.rs:634-644`
- **description:** The doc shows the macro as emitting a generic `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>`. The macro does emit a method by that name, but it is a no-op stub: it discards both arguments (`let _ = relation; let _ = __build;`) and returns `self` unchanged without pushing any `QueryNode::HasRelation` onto `self.filters`. The typed method `where_has_<Relation>` (e.g. `where_has_Parent`) is what actually adds a `HasRelation` node.
- **expected:** Per `docs/query_layer.md:241-245`: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` — the closure is invoked, the result is compiled to a `QueryNode`, and the node is wrapped in `QueryNode::HasRelation` and pushed onto `self.filters`.
- **evidence:**
  - `docs/query_layer.md:241-245` — `pub fn where_has<R, F>(self, relation: R, build: F) -> Self\n    where\n        R: Into<StudentRelation>,\n        F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`
  - `crates/infra/query-derive/src/lib.rs:634-644` — `let generic_where_has = quote! { #struct_vis fn where_has<__R, __F>(mut self, relation: __R, __build: __F) -> Self where __R: ::std::convert::Into<::educore_core::query::Relation>, __F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField, { let _ = relation; let _ = __build; self } };` — both arguments discarded.

---

### FINDING 13

- **id:** DOC-QL-002
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:130-138` vs `crates/infra/query-derive/src/lib.rs:406-431`
- **description:** The doc shows `pub struct StudentQueryBuilder { school_id: SchoolId, filters: Vec<QueryNode<StudentField>>, orders: Vec<OrderNode<StudentField>>, offset: u32, limit: u32, relations: BTreeSet<StudentRelation>, }`. The actual macro emits `school_id: Option<SchoolId>` (not `SchoolId`) and `limit: Option<u32>` (not `u32`), and omits the `relations: BTreeSet<StudentRelation>` field when no relations are declared (only added in the `relations.is_empty()` else-branch).
- **expected:** `pub struct StudentQueryBuilder { school_id: SchoolId, ..., offset: u32, limit: u32, relations: BTreeSet<StudentRelation>, }` per `docs/query_layer.md:130-138`.
- **evidence:**
  - `docs/query_layer.md:130-138` — full builder struct definition.
  - `crates/infra/query-derive/src/lib.rs:411-417` — `school_id: ::std::option::Option<::educore_core::ids::SchoolId>`, `limit: ::std::option::Option<u32>` — both are `Option`, not bare types.
  - `crates/infra/query-derive/src/lib.rs:427-430` — `relations: ::std::collections::BTreeSet<#relation_enum_name>` is added in the `relations.is_empty()` else-branch only; structs without relations emit no `relations` field at all (the doc shows it unconditionally).

---

### FINDING 14

- **id:** DOC-QL-003
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:213, 332-335` vs `crates/infra/core/src/query.rs:331-357`
- **description:** The doc types the `OrderNode` as `OrderNode<F: FieldKind>` (line 213: `pub struct OrderNode<F: FieldKind> { pub field: F, pub direction: OrderDirection, }`) and the related builder as `RelatedQueryBuilder<R>` (line 244). Neither `FieldKind` nor `RelatedQueryBuilder` exists in the engine. The actual trait bound is `OrderNode<F: Field>` (the `Field` trait in `educore-core::query`), and the typed where-has passes `RelatedField` (a unit placeholder struct), not a builder type.
- **expected:** `pub struct OrderNode<F: FieldKind>` and `FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>` per `docs/query_layer.md:213, 244`.
- **evidence:**
  - `docs/query_layer.md:213` — `pub struct OrderNode<F: FieldKind> { pub field: F, pub direction: OrderDirection, }`
  - `docs/query_layer.md:244` — `F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`
  - `crates/infra/core/src/query.rs:34` — `pub trait Field: Clone + Copy + PartialEq + Eq + Hash + fmt::Debug { fn column_name(self) -> &'static str; }` — the trait is `Field`, not `FieldKind`.
  - `crates/infra/core/src/query.rs:331-337` — `pub struct OrderNode<F: Field> { pub field: F, pub direction: OrderDirection, }` — bounded by `Field`.
  - `crates/infra/core/src/query.rs:379-392` — `pub struct RelationalField;` (a unit struct), used as the field type inside `QueryNode::HasRelation`. There is no `RelatedQueryBuilder<R>`.
  - `crates/infra/query-derive/src/lib.rs:638` — the actual generic `where_has` constraint: `__F: ::std::ops::FnOnce(::educore_core::query::RelationalField) -> ::educore_core::query::RelationalField,` — `RelationalField`, not `RelatedQueryBuilder`.

---

### FINDING 15

- **id:** DOC-QL-004
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:228-246` vs `crates/infra/query-derive/src/lib.rs:484-552`
- **description:** The doc shows `where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self` (and `where_between` with `FieldValue` args). The macro emits `where_in<V>(self, field: FieldEnum, values: Vec<V>) -> Self where V: Into<Value>` and `where_between<V>(self, field: FieldEnum, lo: V, hi: V) -> Self where V: Into<Value>`. The element type is generic over `Into<Value>`, not a fixed `FieldValue` (which is not a name that exists in the codebase — the type is `Value`).
- **expected:** `pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self;` and `pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self;` per `docs/query_layer.md:235-236`.
- **evidence:**
  - `docs/query_layer.md:235-236` — `pub fn where_in(self, field: StudentField, values: Vec<FieldValue>) -> Self;` / `pub fn where_between(self, field: StudentField, lo: FieldValue, hi: FieldValue) -> Self;`
  - `crates/infra/query-derive/src/lib.rs:484-512` — `pub fn where_in<V>(mut self, field: #field_enum_name, values: ::std::vec::Vec<V>) -> Self where V: ::std::convert::Into<::educore_core::query::Value>` — generic `Into<Value>`, not `Vec<FieldValue>`.
  - `crates/infra/core/src/query.rs:63-87` — `pub enum Value` is the runtime filter value type; no `FieldValue` alias.

---

### FINDING 16

- **id:** DOC-QL-005
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:484-498, 502-514` vs `crates/infra/core/src/query.rs:398-412` and `crates/infra/query-derive/src/lib.rs`
- **description:** The doc claims (1) the macro emits a `StudentAggregate` enum and an `aggregate(StudentAggregate::Count).group_by(...).execute()` method chain, and (2) the runtime `pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }` carries items + total + is generic over `T`. Neither exists. The macro emits no aggregate methods, no `StudentAggregate` enum. The runtime `Page` struct is non-generic with only `offset: u32` and `limit: u32` (no `items`, no `total`).
- **expected:** `pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }` per `docs/query_layer.md:505-510`; `engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).where_eq(...).execute().await?` per `docs/query_layer.md:486-493`.
- **evidence:**
  - `docs/query_layer.md:505-510` — `pub struct Page<T> { pub items: Vec<T>, pub total: u64, pub offset: u32, pub limit: u32, }`
  - `docs/query_layer.md:486-493` — the full aggregate/group_by/execute chain.
  - `crates/infra/core/src/query.rs:398-411` — `pub struct Page { pub offset: u32, pub limit: u32, }` — non-generic, no `items`, no `total`.
  - `crates/infra/query-derive/src/lib.rs:570-586` — the macro emits `limit`, `offset`, `page` methods only; no `aggregate`, no `group_by`, no `StudentAggregate` (no `aggregate` text appears anywhere in `crates/infra/query-derive/src/lib.rs`).

---

### FINDING 17

- **id:** DOC-QL-006
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:516-527` vs `crates/infra/query-derive/src/lib.rs:803-807, 588-597`
- **description:** The doc states the `StudentQueryBuilder` "is constructed only via `StudentQuery::new(school_id)`" and "The default constructor is private. A query that omits the school id is a compile error." In the actual macro, the builder is constructed via `new()` (no args, public) and `school_id` is `Option<SchoolId>` — the user calls `.for_school(school_id)` after `new()`, and the macro raises a runtime error from `build_query_node()` if `for_school` was never called, not a compile error.
- **expected:** Per `docs/query_layer.md:518-527`: `let q = StudentQueryBuilder::new(tenant.school_id()).where_eq(...)` — required `school_id` argument, no default `new()`.
- **evidence:**
  - `docs/query_layer.md:518-520` — "The `StudentQueryBuilder` is constructed only via `StudentQuery::new(school_id)`. The default constructor is private."
  - `docs/query_layer.md:524-527` — `let q = StudentQueryBuilder::new(tenant.school_id()).where_eq(StudentField::Status, StudentStatus::Active);`
  - `crates/infra/query-derive/src/lib.rs:803-807` — `pub fn new() -> Self { Self::default() }` — a public zero-arg `new()`.
  - `crates/infra/query-derive/src/lib.rs:588-597` — `pub fn for_school(mut self, school_id: ::educore_core::ids::SchoolId) -> Self { self.school_id = ::std::option::Option::Some(school_id); self }` — `for_school` is separate from `new`.
  - `crates/infra/query-derive/src/lib.rs:773-801` — `build_query_node` returns `Err(DomainError::validation(...))` if `self.school_id.is_none()`; the gate is a runtime check, not a compile error.

---

### FINDING 18

- **id:** DOC-QL-007
- **area:** documentation
- **severity:** Medium
- **location:** `docs/query_layer.md:529-540` vs `crates/infra/query-derive/src/lib.rs` (no `StudentCursor`/`next_page`)
- **description:** The doc shows `StudentCursor::after(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())` and `engine.students().next_page(cursor, 100).await?`. Neither `StudentCursor` nor `next_page` is emitted by the macro. There is no `Cursor` type in `crates/infra/query-derive/src/lib.rs` or `crates/infra/core/src/query.rs`.
- **expected:** `StudentCursor::after(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())` and `engine.students().next_page(cursor, 100).await?` per `docs/query_layer.md:535-538`.
- **evidence:**
  - `docs/query_layer.md:535-540` — the full cursor example.
  - `grep "StudentCursor\|next_page\|Cursor" crates/infra/query-derive/src/lib.rs crates/infra/core/src/query.rs` — no matches for `StudentCursor` or `next_page`; only `crates/infra/core/src/query.rs` has a `C` cursor type in trait bounds (unrelated).

---

### FINDING 19

- **id:** DOC-QL-008
- **area:** documentation
- **severity:** Medium
- **location:** `docs/query_layer.md:430-437` vs `crates/infra/query-derive/src/lib.rs:444-453` and `crates/infra/core/src/query.rs:445-475`
- **description:** The doc shows `async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>` as the repository signature. The macro does not emit a `StudentQuery` value type at all — the macro emits `StudentQueryBuilder` (a builder), and `build_query_node()` returns `(QueryNode<StudentField>, Page)`. The `to_relational_node` conversion the macro uses for `HasRelation` lives in `educore_core::query::to_relational_node` (a free function), not on a trait method.
- **expected:** `async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;` per `docs/query_layer.md:447`.
- **evidence:**
  - `docs/query_layer.md:445-453` — `#[async_trait]\npub trait StudentRepository: Send + Sync {\n    async fn query(&self, q: StudentQuery) -> Result<Vec<Student>>;\n    ...`
  - `crates/infra/query-derive/src/lib.rs:773-801` — `pub fn build_query_node(self) -> ::educore_core::error::Result<(::educore_core::query::QueryNode<#field_enum_name>, ::educore_core::query::Page)>` — the macro emits a builder + a builder-to-AST conversion, not a `StudentQuery` value type.
  - `crates/infra/core/src/query.rs:448-475` — `pub fn to_relational_node<F: Field>(node: QueryNode<F>) -> QueryNode<RelationalField>` — a free function.

---

### FINDING 20

- **id:** DOC-HO-001
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-3-HANDOFF.md:7, 61, 66-79, 100-103, 327, 335`
- **description:** PHASE-3 handoff headline claims "23 typed commands, 19 typed events" but the code has 22 typed commands and 23 typed events. The handoff's "What's wired and working" body (lines 61-92) and footer ("crates/domains/academic/src/events.rs — 19 typed events") repeat the 19-events / 23-commands claim, but the actual file counts are inverted.
- **expected:** Per `docs/handoff/PHASE-3-HANDOFF.md:7`: "5 aggregates (Student, Class, Section, Subject, AcademicYear), 23 typed commands, 19 typed events".
- **evidence:**
  - `docs/handoff/PHASE-3-HANDOFF.md:7` — "5 aggregates (Student, Class, Section, Subject, AcademicYear), 23 typed commands, 19 typed events".
  - `docs/handoff/PHASE-3-HANDOFF.md:61` — "**23 typed commands** (8 student lifecycle, 4 class CRUD, 3 section CRUD, 3 subject CRUD, 5 academic-year CRUD)".
  - `docs/handoff/PHASE-3-HANDOFF.md:66` — "**19 typed events** implementing".
  - `crates/domains/academic/src/commands.rs` — `grep -c "^pub struct " crates/domains/academic/src/commands.rs` = **22** (Admit, Update, Suspend, Reinstate, Withdraw, Transfer, Promote, Graduate, CreateClass, UpdateClass, SetOptionalSubjectGpaThreshold, DeleteClass, CreateSection, UpdateSection, DeleteSection, CreateSubject, UpdateSubject, DeleteSubject, CreateAcademicYear, UpdateAcademicYearDates, SetCurrentAcademicYear, CloseAcademicYear).
  - `crates/domains/academic/src/events.rs` — `grep -c "^pub struct " crates/domains/academic/src/events.rs` = **23** (8 student lifecycle + 4 class events + 3 section events + 3 subject events + 5 academic-year events).

---

### FINDING 21

- **id:** DOC-HO-002
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-3-HANDOFF.md:72-79, 335-336`
- **description:** The handoff claims "19 pure factory services" and the matching crate-level prelude asserts the same count, but `services.rs` ships 23 `pub fn` factory functions plus 1 helper (`school_matches`). The handoff and the crate docstring both undercount by 4.
- **expected:** "**19 pure factory services**" per `docs/handoff/PHASE-3-HANDOFF.md:72`.
- **evidence:**
  - `docs/handoff/PHASE-3-HANDOFF.md:72-73` — "**19 pure factory services** (mirror `educore-platform::services::create_school` exactly)."
  - `crates/domains/academic/src/lib.rs:83-92` — `pub use crate::services::{admit_student, ... 24 services re-exported }` — the prelude re-exports 24 functions.
  - `crates/domains/academic/src/services.rs` — `grep -c "^pub fn " crates/domains/academic/src/services.rs` = **24** (23 factory fns + `school_matches` helper).

---

### FINDING 22

- **id:** DOC-HO-003
- **area:** documentation
- **severity:** Low
- **location:** `docs/handoff/PHASE-3-HANDOFF.md:18-20, 100-103`
- **description:** The handoff claims "Phase 3 adds 66 unit tests in `educore-academic`" but `grep -c "#\[test\]" crates/domains/academic/src/*.rs` totals **67** (4 lib + 2 entities + 15 services + 5 query + 4 events + 6 aggregate + 19 value_objects + 10 commands = 67).
- **expected:** "66 unit tests" per `docs/handoff/PHASE-3-HANDOFF.md:18-20`.
- **evidence:**
  - `docs/handoff/PHASE-3-HANDOFF.md:18-20` — "Phase 3 adds 66 unit tests in `educore-academic`".
  - `docs/handoff/PHASE-3-HANDOFF.md:100-103` — "**66 unit tests** in `educore-academic`".
  - Sum across `crates/domains/academic/src/{lib,aggregate,entities,value_objects,commands,events,services,query,repository}.rs`: 4 + 6 + 2 + 19 + 10 + 4 + 15 + 5 + 0 = **67** `#[test]` attributes.

---

### FINDING 23

- **id:** DOC-HO-004
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-9-HANDOFF.md:5-10, 81-94`
- **description:** PHASE-9 handoff claims "18 events + 18 typed commands + 6 service factories" but the actual file counts are 19 events and 22 commands. The handoff's own narrative at lines 84-91 lists **19 event names** (`BookCategoryCreated`, `BookCategoryUpdated`, ..., `FineWaived`) — inconsistent with the headline "18 events". 6 service factories claim matches the actual `services.rs`.
- **expected:** Per `docs/handoff/PHASE-9-HANDOFF.md:5-10`: "6 aggregates + 3 child entities + 18 events + 18 typed commands + 6 service factories".
- **evidence:**
  - `docs/handoff/PHASE-9-HANDOFF.md:5-10` — "6 aggregates + 3 child entities + 18 events + 18 typed commands + 6 service factories + 6 repository ports + 6 query stubs + the `FineCalculationService`".
  - `docs/handoff/PHASE-9-HANDOFF.md:81-91` — the body lists **19** event names: `BookCategoryCreated, BookCategoryUpdated, BookCategoryDeleted, BookAdded, BookUpdated, BookDeleted, BookQuantityAdjusted, LibraryMemberRegistered, LibraryMemberUpdated, LibraryMemberDeactivated, LibraryMemberReactivated, LibraryMemberDeleted, BookIssued, BookReturned, BookRenewed, BookMarkedLost, BookReturnRecorded, FineCalculated, FineWaived`.
  - `crates/domains/library/src/events.rs` — `grep -c "^pub struct " crates/domains/library/src/events.rs` = **19**.
  - `crates/domains/library/src/commands.rs` — `grep -c "^pub struct " crates/domains/library/src/commands.rs` = **22** (the handoff claims 18).

---

### FINDING 24

- **id:** DOC-HO-005
- **area:** documentation
- **severity:** Low
- **location:** `docs/handoff/PHASE-9-HANDOFF.md:16-19`
- **description:** The handoff claims "**31 passed**" unit tests in `educore-library`, but the actual `#[test]` count is 30.
- **expected:** "**31 passed**" per `docs/handoff/PHASE-9-HANDOFF.md:16`.
- **evidence:**
  - `docs/handoff/PHASE-9-HANDOFF.md:16-19` — "`cargo test -p educore-library --lib` — **31 passed**".
  - Sum across `crates/domains/library/src/{lib,events,query,repository,services,value_objects}.rs`: 2 + 1 + 1 + 1 + 13 + 12 = **30** `#[test]` attributes.

---

### FINDING 25

- **id:** DOC-HO-006
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-12-HANDOFF.md:6-23, 27-42`
- **description:** PHASE-12 handoff opens with "**20 root aggregates**" but the file ships 19 root aggregates. The handoff itself enumerates only 19 names ("Page, News, NewsCategory, NewsComment, NewsPage, NoticeBoard, Testimonial, HomeSlider, SpeechSlider, Content, ContentType, ContentShareList, TeacherUploadContent, UploadContent, AboutPage, ContactPage, CoursePage, HomePageSetting, FrontendPage"). Repository traits match (19), query stubs match (19) — but the headline "20 root aggregates" and the "20 query stubs" sub-claim are wrong.
- **expected:** Per `docs/handoff/PHASE-12-HANDOFF.md:6-7` and `:27-28`: "**20 root aggregates**".
- **evidence:**
  - `docs/handoff/PHASE-12-HANDOFF.md:7-13` — "all 20 root aggregates per `docs/specs/cms/aggregates.md` (`Page`, `News`, `NewsCategory`, `NewsComment`, `NewsPage`, `NoticeBoard`, `Testimonial`, `HomeSlider`, `SpeechSlider`, `Content`, `ContentType`, `ContentShareList`, `TeacherUploadContent`, `UploadContent`, `AboutPage`, `ContactPage`, `CoursePage`, `HomePageSetting`, `FrontendPage`)" — only **19** names listed despite "20 root aggregates".
  - `docs/handoff/PHASE-12-HANDOFF.md:27-28` — "**20 root aggregates** ship as first-class ports".
  - `docs/handoff/PHASE-12-HANDOFF.md:457` — "`crates/domains/cms/src/query.rs` — 19 typed query stubs" — sub-claim is 19, not 20.
  - `crates/domains/cms/src/aggregate.rs` — `grep "^pub struct \w\+ {$" crates/domains/cms/src/aggregate.rs | wc -l` after filtering out `NewPage / NewNews / ...` DTOs yields **19** main aggregates.
  - `crates/domains/cms/src/repository.rs` — `grep "pub trait " | wc -l` = **19** traits.
  - `crates/domains/cms/src/query.rs` — `grep "^pub struct" | grep -c "Query"` = **19** query stubs.

---

### FINDING 26

- **id:** DOC-HO-007
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-12-HANDOFF.md:13-17, 30-32`
- **description:** The handoff claims "~67 typed commands" and "10 `CMS_*_COMMAND_TYPE` constants", but the file ships exactly **10 command structs** (not "~67"). The 67 count matches the events file (and the handoff correctly says "~67 typed events"), so the "~67 typed commands" claim is a copy/paste error from the events row.
- **expected:** "~67 typed commands" per `docs/handoff/PHASE-12-HANDOFF.md:30-32`.
- **evidence:**
  - `docs/handoff/PHASE-12-HANDOFF.md:30-32` — "**~67 typed commands** with the matching `<Domain>.<Aggregate>.<Action>` wire form (10 `CMS_*_COMMAND_TYPE` constants; the headline factory fns)."
  - `crates/domains/cms/src/commands.rs` — `grep -c "^pub struct " crates/domains/cms/src/commands.rs` = **10** (`CreatePageCommand`, `PublishPageCommand`, `ArchivePageCommand`, `DeletePageCommand`, `CreateNewsCommand`, `CreateTestimonialCommand`, `CreateHomeSliderCommand`, `CreateContentCommand`, `CreateContentShareListCommand`, `ConfigureHomePageCommand`).
  - `crates/domains/cms/src/commands.rs` `CMS_*_COMMAND_TYPE` consts (lines 442-451): **10** constants, matching the 10 command structs.

---

### FINDING 27

- **id:** DOC-HO-008
- **area:** documentation
- **severity:** High
- **location:** `docs/handoff/PHASE-12-HANDOFF.md:13-17`
- **description:** The handoff asserts "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)". In the actual file there are **19 repository traits, including a separate `SpeechSliderRepository`** — `SpeechSlider` does NOT share the home-slider pattern, it has its own first-class trait.
- **expected:** Per `docs/handoff/PHASE-12-HANDOFF.md:15-17`: "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)".
- **evidence:**
  - `docs/handoff/PHASE-12-HANDOFF.md:13-17` — "19 repository port traits (one per root aggregate except the `SpeechSlider` shares the home-slider pattern)".
  - `crates/domains/cms/src/repository.rs:279` — `pub trait SpeechSliderRepository: Send + Sync {` — a separate, first-class `SpeechSliderRepository` exists.
  - `crates/domains/cms/src/repository.rs` total `pub trait` count = **19**, matching the headline, but the parenthetical "except `SpeechSlider` shares the home-slider pattern" is incorrect.

---

### FINDING 28

- **id:** DOC-HO-009
- **area:** documentation
- **severity:** Medium
- **location:** `docs/handoff/PHASE-13-HANDOFF.md:25-27`
- **description:** The handoff headline claims "**5 service factory structs**" but then lists six names in parentheses (`CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService`), and `services.rs` actually defines 6 structs. The body sub-section "### 5 service factory structs" at line 130 has the same inconsistency.
- **expected:** "**5 service factory structs** (CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService)" per `docs/handoff/PHASE-13-HANDOFF.md:25-27`.
- **evidence:**
  - `docs/handoff/PHASE-13-HANDOFF.md:25-27` — "**5 service factory structs** (CalendarService, RecurrenceService, HolidayService, CalendarSettingService, IncidentService, WeekendService) + 1 `WeekendChange` enum." — six names in the parenthetical despite "5" headline.
  - `crates/cross-cutting/events-domain/src/services.rs` — `grep "^pub struct" crates/cross-cutting/events-domain/src/services.rs` returns **6** structs (`CalendarService`, `RecurrenceService`, `HolidayService`, `CalendarSettingService`, `IncidentService`, `WeekendService`).

---

### FINDING 29

- **id:** DOC-HO-010
- **area:** documentation
- **severity:** Medium
- **location:** `docs/handoff/PHASE-16-HANDOFF.md:148-151`
- **description:** The handoff claims "**59 unit tests** pass (53 in `storage`, 3 in `auth`, 6 in `notify`, 4 in `payment`, 8 in `files`, 3 in `integrations`, 1 in `sync`, 1 in `event_bus`, 3 in `lib`)." Summing the per-file breakdown yields 53+3+6+4+8+3+1+1+3 = **82**, not 59, and the actual file counts (14 in storage.rs, 6 in auth.rs, 6 in notify.rs, 8 in payment.rs, 0 in files.rs, 2 in integrations.rs, 2 in sync.rs, 1 in event_bus.rs, 3 in lib.rs) sum to 42. Neither 82 nor 42 matches the 59 headline.
- **expected:** "**59 unit tests** pass" per `docs/handoff/PHASE-16-HANDOFF.md:148`.
- **evidence:**
  - `docs/handoff/PHASE-16-HANDOFF.md:148-151` — "**59 unit tests** pass (53 in `storage`, 3 in `auth`, 6 in `notify`, 4 in `payment`, 8 in `files`, 3 in `integrations`, 1 in `sync`, 1 in `event_bus`, 3 in `lib`)."
  - `crates/tools/testkit/src/{storage,auth,notify,payment,files,integrations,sync,event_bus,lib,errors}.rs` `#[test]` counts: storage.rs=14, auth.rs=6, notify.rs=6, payment.rs=8, files.rs=0, integrations.rs=2, sync.rs=2, event_bus.rs=1, lib.rs=3, errors.rs=0 → **42** total.

---

### END FINDINGS

**Total findings: 29**

(11 `DOC-LIB-*` for `docs/library-docs.md`, 8 `DOC-QL-*` for `docs/query_layer.md`, 10 `DOC-HO-*` for the handoff docs.)
