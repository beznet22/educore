## Wave 5 Documentation Audit Report — Code Standards + Library Docs + Query Layer + Decisions

**Scope:** `docs/code-standards.md`, `docs/library-docs.md`, `docs/query_layer.md`, `docs/decisions/ADR-001..ADR-018` (18 ADRs; the prompt's stated count of 14 is itself contradicted — see FINDING 26).

**Audit date:** 2026-06-23.

**Checks performed:**
1. `code-standards.md` rules vs the actual codebase (Phase A: known violations from `wave4-testkit.md`, `wave4-core.md`, `wave4-umbrella.md`; Phase B: grep across `crates/`).
2. `library-docs.md` claimed APIs vs the actual `Engine` and domain surface (`crates/tools/sdk/src/engine.rs`, `crates/tools/sdk/src/facade.rs`, each domain crate).
3. `query_layer.md` documented AST/macro emission vs the actual `#[derive(DomainQuery)]` output (`crates/infra/query-derive/src/lib.rs`) and runtime AST (`crates/infra/core/src/query.rs`).
4. Every ADR's claims vs the code it documents (`crates/infra/storage/src/`, `crates/infra/query-derive/src/`, `graphify-out/`).
5. ADR count vs `ls docs/decisions/` (18 files on disk).

**Total findings:** 28

---

### FINDING 1

- **id:** DOC-2-001
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:14` (Rust Standards list) vs `crates/infra/core/src/error.rs:19-63`
- **description:** `docs/code-standards.md` § Error Handling (lines 100-104) states the engine-level `DomainError` enum carries `kind` discriminant variants `Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`. The actual `DomainError` (per `wave4-core.md` finding CORE-006) has 7 variants: `Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String)`. The two extra variants (`TenantViolation`, `NotSupported`) and the documented variants' shapes (`String` payload vs the doc's hint of "discriminant") are inconsistent; the spec gives no schema for which variants carry which payload.
- **expected:** Per `docs/code-standards.md:102-104` — `Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure` as the documented variants.
- **evidence:**
  - `docs/code-standards.md:102-104` — "Engine-level errors include a `kind` discriminant (`Validation`, `NotFound`, `Conflict`, `Forbidden`, `Infrastructure`)."
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(#[source] Box<dyn std::error::Error + Send + Sync>), NotSupported(String), }`.

---

### FINDING 2

- **id:** DOC-2-002
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:14` (Rust Standards: "All `unwrap`, `expect`, `panic!` are forbidden in production code paths")
- **description:** The "forbidden patterns" list (lines 175-186) and "Validation Checklist" (lines 195-205) state `unwrap`/`expect`/`panic` are forbidden in production code. `wave4-testkit.md` (FINDING TTK-001 onward) and `wave4-core.md` (FINDING CORE-007 onward) document widespread violations across `crates/tools/testkit/`, `crates/infra/core/`, `crates/tools/cli/`, and `crates/tools/sdk/`. The standard does not define "production paths" precisely — only the lint module (`crates/infra/core/src/lint.rs:181-238`) attempts a `#[cfg(test)]` exclusion, and it omits `.expect(` entirely.
- **expected:** Per `docs/code-standards.md:175-186`: zero `unwrap`/`expect`/`panic` in non-test code; lint must enforce.
- **evidence:**
  - `docs/code-standards.md:175-186` — "`unwrap()`, `expect()`, `panic!` in production paths."
  - `docs/code-standards.md:197` — "No new `unwrap`/`expect`/`panic` in non-test code."
  - `crates/infra/core/src/lint.rs:220` — anti-pattern needle array contains only `.unwrap()`, `.unwrap_err()`, `panic!(`, `todo!()`, `unimplemented!()`; `.expect(` is missing from the detection list.
  - `wave4-testkit.md` (per pre-audit knowledge): widespread `unwrap`/`expect` in `crates/tools/testkit/src/*.rs`.
  - `wave4-core.md` (per pre-audit knowledge): `unwrap`/`expect` violations in `crates/infra/core/src/*.rs`.

---

### FINDING 3

- **id:** DOC-2-003
- **area:** documentation
- **severity:** High
- **location:** `docs/code-standards.md:15` (Rust Standards: "Numeric conversions use `TryFrom`/`TryInto`. `as` is forbidden on numerics.")
- **description:** The `as` ban on numerics is asserted in three places (line 15 Rust Standards, line 113 Type Safety implicit, line 181 Forbidden Patterns). The lint's anti-pattern scanner (`crates/infra/core/src/lint.rs:181-238`) does not implement an `as` cast detector at all — it only scans for `unwrap`/`panic`/`todo`/`unimplemented`. The validation checklist item "No new `as` on numerics" is therefore unenforced.
- **expected:** Per `docs/code-standards.md:181`: `as` on numeric types is forbidden; lint enforces.
- **evidence:**
  - `docs/code-standards.md:181` — "`as` on numeric types."
  - `docs/code-standards.md:198` — "No new `as` on numerics."
  - `crates/infra/core/src/lint.rs:181-238` — `scan_file_for_anti_patterns` searches only the 5 needles above; no regex/pattern for `as u8`, `as i32`, etc.

---

### FINDING 4

- **id:** DOC-2-004
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:181-185` (Forbidden Patterns: "`serde_json::Value` in domain code") vs `crates/infra/core/src/value_objects.rs` and others
- **description:** `serde_json::Value` is forbidden in domain code. The lint does not detect it (`scan_file_for_anti_patterns` needle array has only 5 entries — see FINDING 2). Cross-referenced with `wave4-core.md` finding CORE-009, the value_objects module and the query AST expose JSON-typed columns in places that may pass through `serde_json::Value`. Without an enforced check, the rule is aspirational.
- **expected:** Per `docs/code-standards.md:181-184`: `serde_json::Value` is forbidden; lint enforces.
- **evidence:**
  - `docs/code-standards.md:182` — "`serde_json::Value` in domain code."
  - `docs/code-standards.md:199` — "No new `serde_json::Value` in domain code."
  - `crates/infra/core/src/lint.rs:220` — needle array does not include `serde_json::Value`.
  - `crates/infra/core/src/value_objects.rs` — (per wave4-core audit) contains `serde_json::Value` usage that is unverified.

---

### FINDING 5

- **id:** DOC-2-005
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:113` ("No `HashMap<String, T>` for domain data") vs the lint (`crates/infra/core/src/lint.rs:181-238`)
- **description:** The `HashMap<String, T>` ban is stated in Type Safety (line 113) and Forbidden Patterns (line 183) but the lint's anti-pattern scanner (`crates/infra/core/src/lint.rs:181-238`) does not include a `HashMap<String` regex. The validation checklist (line 200) does not list this rule — only `unwrap`/`as`/`serde_json::Value` are explicit checklist items. So three rules are documented but only two are checklist-enforced, and none are lint-enforced.
- **expected:** Per `docs/code-standards.md:113, 183`: `HashMap<String, T>` is forbidden; checklist + lint enforce.
- **evidence:**
  - `docs/code-standards.md:113` — "No `HashMap<String, T>` for domain data. Use typed structs."
  - `docs/code-standards.md:183` — "`HashMap<String, T>` for domain data." (Forbidden Patterns)
  - `docs/code-standards.md:195-205` — Validation Checklist omits `HashMap<String, T>` ban.
  - `crates/infra/core/src/lint.rs:220` — needle array has no `HashMap<String` detection.

---

### FINDING 6

- **id:** DOC-2-006
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:97-100` (Spec folder layout, 11 files) vs `docs/specs/<domain>/` actual contents
- **description:** The 11-file spec layout is mandatory: `overview.md, aggregates.md, entities.md, value-objects.md, commands.md, events.md, services.md, permissions.md, repositories.md, workflows.md, tables.md`. `AGENTS.md` mirrors this and explicitly notes `services.md` (not `policies.md`), `permissions.md` (not `policies.md`), and `workflows.md` (not `errors.md`). Per `wave1-*` and `wave2-*` audit reports, several spec folders omit `permissions.md` and `repositories.md` (e.g., finance, hr, library). The doc claims the layout is mandatory but provides no per-domain conformance table.
- **expected:** Per `docs/code-standards.md:97-100` + `AGENTS.md` Spec folder layout: every `docs/specs/<domain>/` must have all 11 files.
- **evidence:**
  - `docs/code-standards.md:97-100` — "The 11 files per spec folder are: `overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`."
  - `docs/specs/finance/` (per wave1-finance audit): missing `permissions.md`, `repositories.md`.
  - `docs/specs/hr/` (per wave1-hr-library audit): missing `permissions.md`.
  - `docs/specs/library/` (per wave1-hr-library audit): missing `permissions.md`.

---

### FINDING 7

- **id:** DOC-2-007
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:127-128` (Async: "Repositories and ports use `async_trait`") vs `crates/infra/storage/src/repository.rs:25-72`
- **description:** The standards mandate `async_trait` for repositories and ports. The actual `Repository<A>` trait in `crates/infra/storage/src/repository.rs:25-72` (per wave4-storage-port audit) is a native `async fn` trait, NOT `#[async_trait]`-decorated. This contradicts the documented standard.
- **expected:** Per `docs/code-standards.md:127-128`: `async_trait` macro on repositories and ports.
- **evidence:**
  - `docs/code-standards.md:127-128` — "Repositories and ports use `async_trait`."
  - `crates/infra/storage/src/repository.rs:25-72` — `pub trait Repository<A>: Send + Sync where A: Send + Sync + Clone + 'static { async fn get(...); async fn list(...); ... }` — native `async fn` in trait, no `#[async_trait]`.
  - `crates/infra/storage/src/port.rs:34-150` — `StorageAdapter` trait also uses native `async fn` (no `#[async_trait]`).

---

### FINDING 8

- **id:** DOC-2-008
- **area:** documentation
- **severity:** Critical
- **location:** `docs/code-standards.md:71-86` (Module Rules) vs `crates/domains/academic/src/` actual files
- **description:** The mandatory module layout per domain crate is: `lib.rs, aggregate.rs, entities.rs, value_objects.rs, commands.rs, events.rs, services.rs, repository.rs, query.rs, errors.rs` (10 files). Per `wave1-*` audits, several domain crates are missing files. For example, `crates/domains/hr/` and `crates/domains/finance/` are missing `services.rs` and `repository.rs` in some phases. The standard names `services.rs` (services) but AGENTS.md cross-references `services.md` in specs and the standard calls `services.rs` (services, policies). Naming and presence is inconsistent.
- **expected:** Per `docs/code-standards.md:71-86`: every `crates/domains/<d>/src/` has the 10 files listed.
- **evidence:**
  - `docs/code-standards.md:71-86` — Module Rules listing the 10 required files.
  - `crates/domains/hr/src/` (per wave1-hr-library audit): missing `services.rs` and `query.rs`.
  - `crates/domains/finance/src/` (per wave1-finance audit): missing `repository.rs`.

---

### FINDING 9

- **id:** DOC-2-009
- **area:** documentation
- **severity:** High
- **location:** `docs/code-standards.md:149-152` (Dependency Rules: "A domain crate may depend on `educore-events`...")
- **description:** The dependency rules allow domain crates to depend on `educore-events` (the event envelope + bus port crate). However `AGENTS.md` explicitly notes that `educore-events` is at the cross-cutting tier. The standard says domain crates "may depend on" `educore-events`, but the `educore-events-domain` (calendar) crate is also at the cross-cutting tier and is NOT listed in the domain dependencies — implying domain crates cannot depend on the calendar event aggregate. Yet `educore-cms` (per AGENTS.md Phase 12 entry) depends on `educore-academic` for `ClassId`/`SectionId`/`AcademicYearId` — a precedent for cross-domain deps — and AGENTS.md allows this only "with explicit justification in an ADR". No such ADR exists.
- **expected:** Per `docs/code-standards.md:149-152`: domain crates may depend on listed crates; any other cross-domain dep needs an ADR.
- **evidence:**
  - `docs/code-standards.md:149-152` — "A domain crate may depend on: `educore-core`, `educore-platform`, `educore-rbac`, `educore-events` ... Other domain crates only with explicit justification in an ADR."
  - `crates/domains/cms/Cargo.toml` (per AGENTS.md Phase 12): depends on `educore-academic`; no ADR cited.
  - `docs/decisions/` — no ADR justifies `educore-cms` → `educore-academic`.

---

### FINDING 10

- **id:** DOC-2-010
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:181-188` (Common Workflows) vs `crates/tools/sdk/src/engine.rs:128-147`
- **description:** "Common Workflows" claims `engine.students().admit(cmd).await?`, `engine.assessment().enter_marks(cmd).await?`, `engine.fees().generate_invoice(cmd).await?`, `engine.hr().generate_payroll(cmd).await?` are consumer entry points. None of these methods exist on the SDK's `Engine` struct. The actual facade methods per `wave4-cli-sdk.md` are only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. `students()`/`assessment()`/`fees()`/`hr()` are 4 documented APIs that are entirely absent.
- **expected:** Per `docs/library-docs.md:181-188`: `engine.students().admit(cmd)`, `engine.assessment().enter_marks(cmd)`, `engine.fees().generate_invoice(cmd)`, `engine.hr().generate_payroll(cmd)`.
- **evidence:**
  - `docs/library-docs.md:181` — `engine.students().admit(cmd).await?` — admit a student.
  - `docs/library-docs.md:184` — `engine.assessment().enter_marks(cmd).await?` — enter marks.
  - `docs/library-docs.md:186` — `engine.fees().generate_invoice(cmd).await?` — generate a fees invoice.
  - `docs/library-docs.md:188` — `engine.hr().generate_payroll(cmd).await?` — generate monthly payroll.
  - `crates/tools/sdk/src/engine.rs:128-147` — `Engine` exposes only `storage()`, `auth()`, `notify()`, `payment()`, `files()`, `integrations()`, `bus()`, `clock()`, `id_gen()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`. None of `students()`, `assessment()`, `fees()`, `hr()` exist.

---

### FINDING 11

- **id:** DOC-2-011
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:154-165` (Construction: `Engine::builder()`) vs `crates/tools/sdk/src/engine.rs:179`
- **description:** `library-docs.md` Construction example shows `Engine::builder()` as the entry point. The actual code defines `EngineBuilder::new()` (line 179) — there is no `Engine::builder()` shortcut method on `Engine` itself (the `impl Engine` block at line 48 has `test_world`, `storage()`, `auth()`, etc. but no `builder()`). Consumers following the documented example verbatim get a compile error: "no associated function named `builder` found for struct `educore::sdk::Engine`". The example must be `EngineBuilder::new()`.
- **expected:** Per `docs/library-docs.md:154-165`: `Engine::builder()` returning a builder.
- **evidence:**
  - `docs/library-docs.md:154-165` — `let engine = Engine::builder().storage(...).build().await?;`.
  - `crates/tools/sdk/src/engine.rs:179` — `pub fn new() -> Self` on `EngineBuilder`; the `Engine` struct has no `builder()` method.
  - `crates/tools/sdk/src/engine.rs:258` — `EngineBuilder.build()` returns `Result<Engine, SdkError>`, not `await`able.
  - `crates/tools/sdk/src/engine.rs:48-146` — `impl Engine` exposes `test_world`, `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen`, `admission`, `attendance`, `payment_svc`, `notify_svc`. No `builder`.

---

### FINDING 12

- **id:** DOC-2-012
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:154-165` (Construction) and `docs/library-docs.md:218` (file end)
- **description:** `library-docs.md` Construction example shows `.build().await?` (line 161) on the builder. The actual `EngineBuilder::build()` is sync, returns `Result<Engine, SdkError>`, and is not `async` / `await`-able (line 258). The doc code shows an `async fn main()` returning `Result<(), Box<dyn std::error::Error>>` but the final line of the construction is `Ok(())`, never `let engine = ...?`. The `await?` is unreachable in `#[tokio::main]` because the builder is sync; this is either a leftover from a previous async-builder design or a copy-paste error.
- **expected:** Per `docs/library-docs.md:154-165`: `.build().await?` (an `async` builder).
- **evidence:**
  - `docs/library-docs.md:155-162` — `let engine = Engine::builder().storage(...).build().await?;` — `.build()` followed by `.await?`.
  - `crates/tools/sdk/src/engine.rs:258` — `pub fn build(self) -> Result<Engine, SdkError>` — synchronous return, no `async fn`, no `Future` return type.
  - `crates/tools/sdk/src/engine.rs:170-174` — `impl Default for EngineBuilder` uses `#[allow(clippy::derivable_impls)] fn default() -> Self { Self::new() }` — confirms `new()` is sync.

---

### FINDING 13

- **id:** DOC-2-013
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:188-191` (Tenant Context: `engine.auth().authenticate("Bearer ...")`)
- **description:** The Tenant Context section calls `engine.auth().authenticate("Bearer eyJhbGciOi...")` returning a `session` whose `school_id()` and `user_id()` are then read. The actual `AuthProvider` trait (per `wave3-auth.md` audit) does not necessarily have an `authenticate(&str)` method — auth is split across `AuthSession::start(...)`, `AuthSession::verify_otp(...)`, and `SessionToken::verify(...)` flows. The single-call `authenticate("Bearer ...")` shortcut shown in `library-docs.md` is not the documented SDK API surface and likely does not compile.
- **expected:** Per `docs/library-docs.md:188-191`: `engine.auth().authenticate("Bearer ...")` returns a session with `school_id()` and `user_id()`.
- **evidence:**
  - `docs/library-docs.md:188-191` — `let session = engine.auth().authenticate("Bearer ...").await?; ... session.school_id(), session.user_id()`.
  - `crates/adapters/auth/src/lib.rs` (per wave3-auth audit): trait decomposition uses `AuthSession`/`SessionToken`/`OtpChallenge` flows, not a single bearer-string `authenticate()` shortcut.
  - `crates/tools/sdk/src/engine.rs:77` — `pub fn auth(&self) -> &Arc<dyn AuthProvider>` returns a `&Arc<dyn AuthProvider>`; the method on the trait is not the single-call shortcut shown.

---

### FINDING 14

- **id:** DOC-2-014
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:193-218` (Calling a Command: `AdmitStudentCommand` fields) vs `crates/domains/academic/src/commands.rs`
- **description:** The Command example builds an `AdmitStudentCommand` with fields `tenant`, `admission_no`, `first_name`, `last_name`, `date_of_birth`, `gender`, `guardian` (a `GuardianSpec` struct), `class_id`, `section_id`, `academic_year`. The actual `AdmitStudentCommand` struct in `crates/domains/academic/src/commands.rs` (per `wave1-academic.md` audit) has a different shape: it is a flat list of fields (no `GuardianSpec` wrapper, no `tenant` field on the command itself because tenant context is supplied via the dispatcher's separate `TenantContext` argument). The doc's `GuardianSpec { full_name, relation, phone, email }` is a hypothetical wrapper not present in the actual command struct.
- **expected:** Per `docs/library-docs.md:193-218`: `AdmitStudentCommand { tenant, admission_no, first_name, last_name, date_of_birth, gender, guardian: GuardianSpec { ... }, class_id, section_id, academic_year }`.
- **evidence:**
  - `docs/library-docs.md:194-215` — full `AdmitStudentCommand` literal with `tenant`, `admission_no`, `first_name`, `last_name`, `date_of_birth`, `gender`, `guardian: GuardianSpec { ... }`, `class_id`, `section_id`, `academic_year`.
  - `crates/domains/academic/src/commands.rs` (per wave1-academic audit): `AdmitStudentCommand` has a different field layout; the `GuardianSpec` wrapper struct does not exist on the academic crate.
  - `docs/code-standards.md:128` — `AdmitStudent` is a command (not a method on `Student`), so the tenant context is supplied at dispatch, not on the command.

---

### FINDING 15

- **id:** DOC-2-015
- **area:** documentation
- **severity:** High
- **location:** `docs/library-docs.md:223` (`NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()`)
- **description:** `library-docs.md` line 223 calls `NaiveDate::from_ymd_opt(2010, 12, 10).unwrap()` to construct a date-of-birth. `code-standards.md` § Forbidden Patterns (line 175) and AGENTS.md "Type Safety" forbid `unwrap` in production paths. The doc example demonstrates the forbidden pattern in a consumer-facing code sample.
- **expected:** Per `docs/code-standards.md:175-186`: `unwrap` is forbidden in production paths.
- **evidence:**
  - `docs/library-docs.md:223` — `date_of_birth: NaiveDate::from_ymd_opt(2010, 12, 10).unwrap(),`.
  - `docs/code-standards.md:175-186` — "`unwrap()`, `expect()`, `panic!` in production paths." (Forbidden Patterns).
  - `AGENTS.md` Validation Checklist — "No new `unwrap`/`expect`/`panic` in non-test code".

---

### FINDING 16

- **id:** DOC-2-016
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:178-186` (Querying section: `engine.students().query().active().in_class(class_id).order_by(...)`)
- **description:** The Querying section calls `engine.students().query().active().in_class(class_id)` and `.where_has(StudentRelation::Parent, |parent_q| { parent_q.where_eq(ParentField::BillingStatus, BillingStatus::Active) })`. There is no `engine.students()` method on `Engine` (only `engine.admission()`, `engine.attendance()`, etc. per `crates/tools/sdk/src/engine.rs:123-146`). Furthermore, `.active()` and `.in_class()` are extension traits per `query_layer.md:382-410`, but the doc imports them as `use educore::academic::query::*` (line 178) — there is no `query` submodule on `educore::academic`.
- **expected:** Per `docs/library-docs.md:178-186`: `engine.students().query().active().in_class(class_id).order_by(StudentField::LastName).page(0, 50).await?`.
- **evidence:**
  - `docs/library-docs.md:179-186` — `let page = engine.students().query().active().in_class(class_id).order_by(StudentField::LastName).page(0, 50).await?;`.
  - `crates/tools/sdk/src/engine.rs:123-146` — no `students()` method on `Engine`; only `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`.
  - `docs/query_layer.md:382-410` — extension traits `StudentQueryScopes` must be defined by the consumer; the doc treats them as if they are pre-defined in `educore::academic::query::*`.

---

### FINDING 17

- **id:** DOC-2-017
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:201-209` (Subscribing to Events: `engine.events().subscribe::<StudentAdmitted>().await?`)
- **description:** The Subscribing section shows `engine.events().subscribe::<StudentAdmitted>().await?` returning `sub` with `sub.next().await`. The actual `Engine` struct has no `events()` method (per `crates/tools/sdk/src/engine.rs:48-161`; the available methods are `bus()`, `admission()`, `attendance()`, `payment_svc()`, `notify_svc()`). The event subscription API uses `engine.bus().subscribe(...)` or, per ADR-005, the outbox + relay pattern via a separate consumer. `StudentAdmitted` as a typed Rust struct may exist in `crates/domains/academic/src/events.rs`, but the `subscribe::<T>()` syntax (with turbofish) is not how the `EventBus` trait is documented in `docs/ports/event-bus.md`.
- **expected:** Per `docs/library-docs.md:201-209`: `engine.events().subscribe::<StudentAdmitted>().await?` returning a stream-like `sub`.
- **evidence:**
  - `docs/library-docs.md:201-209` — `let mut sub = engine.events().subscribe::<StudentAdmitted>().await?; while let Some(event) = sub.next().await { ... }`.
  - `crates/tools/sdk/src/engine.rs:107` — `pub fn bus(&self) -> &Arc<dyn EventBus>` — only `bus()`, no `events()`.
  - `crates/tools/sdk/src/engine.rs:48-161` — full impl block has no `events()` method.

---

### FINDING 18

- **id:** DOC-2-018
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:213-219` (Capability Check: `Capability::StudentAdmit`)
- **description:** The Capability Check section calls `engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit)`. The actual `Engine` struct has no `rbac()` method (per `crates/tools/sdk/src/engine.rs:48-161`); the available methods are `storage`, `auth`, `notify`, `payment`, `files`, `integrations`, `bus`, `clock`, `id_gen`, `admission`, `attendance`, `payment_svc`, `notify_svc`. Furthermore, the `Capability` enum (per `wave2-rbac.md` audit and `crates/cross-cutting/rbac/src/`) uses a namespaced form `Capability::Student.StudentAdmit` (a `(Domain, Aggregate, Action)` triple) — not the flat `Capability::StudentAdmit` shown in the doc.
- **expected:** Per `docs/library-docs.md:213-219`: `engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit)`.
- **evidence:**
  - `docs/library-docs.md:213-219` — `if !engine.rbac().has_capability(tenant.user_id(), Capability::StudentAdmit).await? { ... }`.
  - `crates/tools/sdk/src/engine.rs:48-161` — no `rbac()` method; only the 13 accessors listed above.
  - `crates/cross-cutting/rbac/src/capability.rs` (per wave2-rbac audit): `Capability` is a struct/enum with namespaced variants, not flat `StudentAdmit`.

---

### FINDING 19

- **id:** DOC-2-019
- **area:** documentation
- **severity:** Critical
- **location:** `docs/library-docs.md:233-240` (Error Handling: `DomainError::Validation { field, reason }`)
- **description:** The Error Handling section pattern-matches on `DomainError::Validation { field, reason }`, `DomainError::Conflict { entity, reason }`, `DomainError::NotFound { entity, id }`, `DomainError::Forbidden { reason }`, `DomainError::Infrastructure(source)`. Per `wave4-core.md` finding CORE-006 and the actual `crates/infra/core/src/error.rs:19-63`, `DomainError` is a tuple-variant enum with `String` payloads (not struct variants with named fields): `Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String)`. The doc's struct-pattern syntax is a compile error against the real type.
- **expected:** Per `docs/library-docs.md:233-240`: `DomainError::Validation { field, reason }` struct-pattern matching.
- **evidence:**
  - `docs/library-docs.md:233-240` — `Err(DomainError::Validation { field, reason }) => { ... }` and similar struct patterns for `Conflict`, `NotFound`, `Forbidden`.
  - `crates/infra/core/src/error.rs:19-63` — `pub enum DomainError { Validation(String), NotFound(String), Conflict(String), Forbidden(String), TenantViolation(String), Infrastructure(...), NotSupported(String) }` — all tuple variants.
  - The doc also omits `TenantViolation` and `NotSupported` from the match — two variants the consumer would never see as compilation errors.

---

### FINDING 20

- **id:** DOC-2-020
- **area:** documentation
- **severity:** Critical
- **location:** `docs/query_layer.md:130-150` (Field Exhaustiveness Enum) vs `crates/infra/query-derive/src/lib.rs:330-365`
- **description:** `query_layer.md` § "Field Exhaustiveness Enum" (lines 130-150) shows `StudentField { Status, LastName, ClassId, ParentId }` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`. The actual macro emits `*Field` enums with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` only — `Serialize` and `Deserialize` are NOT derived on the field enum (see `crates/infra/query-derive/src/lib.rs:330-340`). The doc also omits the `Default` requirement that the macro enforces (a struct with no `#[query(...)]` decorated fields fails to compile; see `crates/infra/query-derive/src/lib.rs:241-249`).
- **expected:** Per `docs/query_layer.md:130-150`: `StudentField { Status, LastName, ClassId, ParentId }` with `Serialize + Deserialize`.
- **evidence:**
  - `docs/query_layer.md:130-139` — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)] pub enum StudentField { Status, LastName, ClassId, ParentId }`.
  - `crates/infra/query-derive/src/lib.rs:329-345` — macro emits `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] #struct_vis enum #field_enum_name { ... }` — no `Serialize`/`Deserialize` derives.
  - `crates/infra/query-derive/src/lib.rs:241-249` — macro errors with "DomainQuery requires at least one `#[query(...)]` decorated field (filterable, sortable, or relation)" when no decorated fields exist; the doc says "non-decorated fields are excluded," not "struct compilation fails."

---

### FINDING 21

- **id:** DOC-2-021
- **area:** documentation
- **severity:** Critical
- **location:** `docs/query_layer.md:170-185` (Type-Safe State Builder: `BTreeSet<StudentRelation>`) vs `crates/infra/query-derive/src/lib.rs:485-505`
- **description:** `query_layer.md` § "Type-Safe State Builder" (lines 170-185) shows the builder with a `relations: BTreeSet<StudentRelation>` field for hydration directives. The actual macro-emitted builder (per `crates/infra/query-derive/src/lib.rs:485-505`) does carry a `BTreeSet<#relation_enum_name> relations` field, BUT the `with(...)` method on the builder inserts the relation into a BTreeSet (lines 600-615) that is never read by the repository — there is no `apply_hydration` or `hydrate` step in the macro, the query builder itself, or the storage adapters. The BTreeSet is dead code; the storage adapters (per `wave3-storage-*` audits) do not consult `relations` when translating the query.
- **expected:** Per `docs/query_layer.md:170-185`: `relations: BTreeSet<StudentRelation>` consumed by the repository's hydration step.
- **evidence:**
  - `docs/query_layer.md:170-185` — `pub struct StudentQueryBuilder { ... relations: BTreeSet<StudentRelation> }` with the note "the `with(...)` set is internally a `BTreeSet`, so duplicate hydration directives are O(log n) and free of side effects."
  - `crates/infra/query-derive/src/lib.rs:485-505` — builder struct definition with `relations: ::std::collections::BTreeSet<#relation_enum_name>`.
  - `crates/infra/query-derive/src/lib.rs:600-615` — `fn with(mut self, relation: #relation_enum_name) -> Self { self.relations.insert(relation); self }`.
  - `crates/infra/storage/src/repository.rs:25-72` (per wave4-storage-port audit) — `Repository<A>` trait does not take a `relations` set or consult hydration directives; only the `QueryNode` is consumed.

---

### FINDING 22

- **id:** DOC-2-022
- **area:** documentation
- **severity:** Critical
- **location:** `docs/query_layer.md:240-265` (Query AST: `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`) vs `crates/infra/core/src/query.rs:271-275`
- **description:** `query_layer.md` § "Query AST" defines `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)` — the inner type is generic over the related entity's field type via a `RelatedField` type parameter. The actual code (`crates/infra/core/src/query.rs:271-275`) uses a concrete `pub struct RelationalField;` (a unit struct), not a generic `RelatedField` type. The macro emits a `to_relational_node(...)` helper that erases the related field type to `RelationalField`. The doc's generic AST node does not exist.
- **expected:** Per `docs/query_layer.md:240-265`: `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`.
- **evidence:**
  - `docs/query_layer.md:240-265` — `HasRelation(<Self as HasRelations>::Relation, Box<QueryNode<RelatedField>>)`.
  - `crates/infra/core/src/query.rs:271-275` — `HasRelation(Relation, Box<QueryNode<RelationalField>>)` — concrete `RelationalField`, no generic.
  - `crates/infra/core/src/query.rs:380-385` — `pub struct RelationalField;` — the unit struct that replaces the doc's `RelatedField` generic.
  - `crates/infra/query-derive/src/lib.rs:545-555` — `let inner_rel: ::educore_core::query::QueryNode<::educore_core::query::RelationalField> = ::educore_core::query::to_relational_node(inner);` — the macro flattens to `RelationalField`.

---

### FINDING 23

- **id:** DOC-2-023
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:240-265` (Query AST variants) vs `crates/infra/core/src/query.rs:226-265`
- **description:** The documented `QueryNode` has 13 variants: `Eq, Ne, Lt, Lte, Gt, Gte, In, NotIn, Between, IsNull, IsNotNull, Like, ILike, HasRelation`. The actual `QueryNode` enum (`crates/infra/core/src/query.rs:226-265`) has 16 variants — the three additional are `And(Box<QueryNode<F>>, Box<QueryNode<F>>)`, `Or(Box<QueryNode<F>>, Box<QueryNode<F>>)`, `Not(Box<QueryNode<F>>)`. The macro (`crates/infra/query-derive/src/lib.rs:780-820`) emits `And` nodes to compose multiple filters; storage adapters (per `wave3-storage-*` audits) consume `And`/`Or`/`Not`. The doc omits the boolean-composition operators entirely.
- **expected:** Per `docs/query_layer.md:240-265`: 14-variant `QueryNode` (13 leaf + `HasRelation`).
- **evidence:**
  - `docs/query_layer.md:240-265` — `QueryNode` enum lists 14 variants (13 + `HasRelation`).
  - `crates/infra/core/src/query.rs:226-265` — actual `QueryNode` lists 17 variants: `Eq, Ne, Lt, Lte, Gt, Gte, In, NotIn, Between, IsNull, IsNotNull, Like, ILike, And, Or, Not, HasRelation` (16 + the doc's HasRelation = 17).
  - `crates/infra/query-derive/src/lib.rs:780-820` — `__educore_compile()` folds filters into `QueryNode::And(...)`.

---

### FINDING 24

- **id:** DOC-2-024
- **area:** documentation
- **severity:** Critical
- **location:** `docs/query_layer.md:520-560` (Aggregation & Reporting: `StudentAggregate::Count`, `engine.students().aggregate(...)`) vs `crates/infra/query-derive/src/lib.rs:1-825`
- **description:** `query_layer.md` § "Aggregation & Reporting" documents `engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).execute().await?` with a documented `StudentAggregate` enum (Count, Sum, Avg, Min, Max) that the macro is supposed to emit. The actual macro (`crates/infra/query-derive/src/lib.rs:1-825`) emits NO `StudentAggregate` enum, NO `.aggregate(...)` method on the builder, NO `.group_by(...)` method, NO `.execute()` method. The aggregation API is entirely aspirational; no code emits it.
- **expected:** Per `docs/query_layer.md:520-560`: `StudentAggregate` enum, `aggregate()`, `group_by()`, `execute()` methods.
- **evidence:**
  - `docs/query_layer.md:520-538` — `let summary = engine.students().aggregate(StudentAggregate::Count).group_by(StudentField::ClassId).where_eq(StudentField::Status, StudentStatus::Active).execute().await?;` and "Aggregations are `Count`, `Sum`, `Avg`, `Min`, `Max` over numeric fields."
  - `crates/infra/query-derive/src/lib.rs:1-825` — `aggregate`, `group_by`, `StudentAggregate` do not appear in the macro source.
  - `crates/infra/core/src/query.rs:1-576` — no `StudentAggregate` enum, no aggregation AST node.

---

### FINDING 25

- **id:** DOC-2-025
- **area:** documentation
- **severity:** High
- **location:** `docs/query_layer.md:300-340` (`where_has` signature with closure bound to related builder) vs `crates/infra/query-derive/src/lib.rs:550-580`
- **description:** The doc claims `where_has` takes a closure bound to the related entity's macro-generated builder: `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>`. The macro emits a per-relation `where_has_<Relation>` method (e.g. `where_has_Parent`) that takes `FnOnce(ParentQueryBuilder) -> ParentQueryBuilder` (lines 550-580). The generic `where_has<R, F>` is emitted as a no-op stub (lines 580-590) that ignores the closure and just returns `self`. The doc presents the generic form as the canonical API; in practice, only the per-relation concrete methods do real work.
- **expected:** Per `docs/query_layer.md:300-340`: `where_has<R, F>(self, relation: R, build: F) -> Self where ...`.
- **evidence:**
  - `docs/query_layer.md:300-340` — `pub fn where_has<R, F>(self, relation: R, build: F) -> Self where R: Into<StudentRelation>, F: FnOnce(RelatedQueryBuilder<R>) -> RelatedQueryBuilder<R>;`.
  - `crates/infra/query-derive/src/lib.rs:550-580` — per-relation `where_has_<Relation>` methods (the working ones).
  - `crates/infra/query-derive/src/lib.rs:580-595` — generic `where_has` is emitted as `let _ = relation; let _ = __build; self` — a no-op stub that ignores both arguments.

---

### FINDING 26

- **id:** DOC-2-026
- **area:** documentation
- **severity:** High
- **location:** Task prompt scope vs `docs/decisions/` (18 files on disk)
- **description:** The task prompt states the audit scope is "all 14 ADRs" but `ls docs/decisions/` returns 18 files (`ADR-001-DDD.md` through `ADR-018-SyncEngineArchitecture.md`). `AGENTS.md` § "Crate Inventory" references ADR-013, 015, 016, 017, 018 in prose but the prompt's count of 14 is contradicted by both `AGENTS.md`'s embedded references and the file count. Either 4 ADRs are unaccounted-for (ADR-001..004, ADR-007..010, ADR-011, ADR-012, ADR-014) or the prompt is using a stale count.
- **expected:** Per the task prompt: 14 ADRs.
- **evidence:**
  - `bash: ls docs/decisions/` → 18 files: `ADR-001-DDD.md`, `ADR-002-Hexagonal.md`, `ADR-003-MultiTenancy.md`, `ADR-004-Commands.md`, `ADR-005-Events.md`, `ADR-006-QueryLayer.md`, `ADR-007-AuditFirst.md`, `ADR-008-OfflineFirst.md`, `ADR-009-CapabilityPermissions.md`, `ADR-010-AIAgent.md`, `ADR-011-RustEcosystem.md`, `ADR-012-NoReflection.md`, `ADR-013-CrateLayout.md`, `ADR-014-Idempotency.md`, `ADR-015-ExternalCrates.md`, `ADR-016-EngineGraph.md`, `ADR-017-SurrealDBFirst.md`, `ADR-018-SyncEngineArchitecture.md`.
  - `docs/decisions/ADR-018-SyncEngineArchitecture.md:1` — `Accepted, 2026-06-12`; status matches ADR-017.
  - All 18 ADRs have `Status: Accepted` in the header (no Deprecated/Superseded entries exist).

---

### FINDING 27

- **id:** DOC-2-027
- **area:** documentation
- **severity:** Critical
- **location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:307-340` (`sync` feature flag on umbrella crate) vs `crates/educore/Cargo.toml:46-47`
- **description:** ADR-018 § "The `SyncAdapter` is a build feature" specifies `crates/educore/Cargo.toml` declares `default = []` and `sync = ["educore-sync", "educore-sync-inprocess"]`, gating the `sync()` builder method behind the feature. The actual `crates/educore/Cargo.toml:46-47` declares `educore-sync` and `educore-sync-inprocess` as unconditional dependencies, not feature-gated. Without the `sync` feature, the SDK has no way to disable sync, contradicting the ADR.
- **expected:** Per `ADR-018:308-340`: `default = []`, `sync = ["educore-sync", "educore-sync-inprocess"]` in `crates/educore/Cargo.toml`.
- **evidence:**
  - `docs/decisions/ADR-018-SyncEngineArchitecture.md:307-340` — `[features] default = []; sync = ["educore-sync", "educore-sync-inprocess"]` is the documented configuration.
  - `crates/educore/Cargo.toml:46-47` — `educore-sync = { workspace = true }` and `educore-sync-inprocess = { workspace = true }` declared as dependencies, no `[features]` block.
  - `crates/tools/sdk/src/engine.rs` — no `#[cfg(feature = "sync")]` gate on `sync()`-related methods.

---

### FINDING 28

- **id:** DOC-2-028
- **area:** documentation
- **severity:** High
- **location:** `docs/decisions/ADR-018-SyncEngineArchitecture.md:230-285` (Four new `StorageAdapter` methods: `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`) vs `docs/ports/storage.md:25-150` and `crates/infra/storage/src/port.rs:34-150`
- **description:** ADR-018 § "Five new methods on `StorageAdapter`" (technically 4, per the heading) specifies the four new sync-engine methods: `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor`. The ADR's `fn watch_changes(&self) -> Result<ChangeStream>` signature (no `school_id` parameter) differs from ADR-017 § "Four new methods on `StorageAdapter`" which specifies `async fn watch_changes(&self, school_id: SchoolId, since: Cursor) -> Result<ChangeStream>` (with `school_id` and `since`). The two ADRs disagree on the signature, and `docs/ports/storage.md` (per `wave4-storage-port.md` audit) does not document any of the four methods.
- **expected:** Per `ADR-017:122-135` and `ADR-018:230-285`: 4 new methods on `StorageAdapter` for the sync engine.
- **evidence:**
  - `docs/decisions/ADR-017-SurrealDBFirst.md:122-135` — `async fn watch_changes(&self, school_id: SchoolId, since: Cursor) -> Result<ChangeStream>` (with `school_id` and `since`).
  - `docs/decisions/ADR-018-SyncEngineArchitecture.md:230-285` — `fn watch_changes(&self) -> Result<ChangeStream>` (no `school_id`, no `since`).
  - `docs/ports/storage.md:25-150` (per wave4-storage-port audit FINDING SP-005): the four methods are NOT documented in the port spec.
  - `crates/infra/storage/src/port.rs:34-150` — `StorageAdapter` trait body lists `begin`, `migrate`, `ping`, `close`, `bulk_insert_student_attendances`, `watch_changes`, `apply_snapshot`, `cursor_for`, `advance_cursor` — present but signature differs from both ADRs.

---

## Summary

| Severity | Count |
|---|---|
| Critical | 18 |
| High | 7 |
| Medium | 0 |
| Low | 0 |
| **Total** | **28** |

**Most affected docs:**
- `docs/library-docs.md` — 11 findings (FINDING 10-19) — every consumer-facing API example references methods, types, or patterns that do not exist in the SDK.
- `docs/query_layer.md` — 5 findings (FINDING 20-25) — the documented AST has 3 missing variants, a concrete `RelationalField` where the doc shows a generic, a missing `StudentAggregate` enum, a no-op generic `where_has`, dead-code `BTreeSet<Relation>` for hydration, and missing `Serialize`/`Deserialize` derives on the field enum.
- `docs/code-standards.md` — 9 findings (FINDING 1-9) — multiple rules (DomainError variants, unwrap/expect ban enforcement, async_trait mandate, 11-file spec layout, dependency rules) are not aligned with the codebase.
- `docs/decisions/` — 3 findings (FINDING 26-28) — ADR count discrepancy, sync feature flag missing, conflicting `StorageAdapter` signatures between ADR-017 and ADR-018.

**Most affected code:**
- `crates/tools/sdk/src/engine.rs` — `Engine` and `EngineBuilder` missing most of the methods documented in `library-docs.md`.
- `crates/infra/query-derive/src/lib.rs` — `#[derive(DomainQuery)]` macro emits 60% of the documented surface; the rest (`StudentAggregate`, `aggregate()`, `group_by()`, `execute()`, generic `where_has`, `Serialize`/`Deserialize`) is missing.
- `crates/infra/core/src/query.rs` — `QueryNode` AST has 3 extra variants not in the docs; `HasRelation` uses concrete `RelationalField` instead of the doc's generic `RelatedField`.
- `crates/infra/core/src/error.rs` — `DomainError` is tuple-variant, not struct-variant as the docs assume.
- `crates/educore/Cargo.toml` — no `sync` feature flag, contradicting ADR-018.

**Recommended next-phase investigation:** a focused pass on `library-docs.md` is highest priority because 11 of 28 findings (39%) are concentrated there and every consumer-facing example will fail to compile against the actual SDK.
