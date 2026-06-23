## Wave 4 Tools Audit Report — `educore-cli` + `educore-sdk`

**Scope:** `crates/tools/cli/` (`educore-cli` binary, 3 `.rs` files + Cargo.toml + README), `crates/tools/sdk/` (`educore-sdk` lib, 4 `.rs` files + Cargo.toml), `crates/educore/tests/consumer_e2e.rs`, `docs/library-docs.md` (consumer SDK docs), `docs/build-plan.md:1646-1702` (Phase 16 — sdk + cli delivery), `docs/handoff/PHASE-16-HANDOFF.md`, `docs/ports/event-bus.md`.

**Total findings:** 22

---

### FINDING 1

- **id:** CLI-SDK-001
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `crates/tools/sdk/src/engine.rs:41-149`
- **description:** `docs/library-docs.md:18` shows `Engine::builder()` as the canonical construction entry point, and `crates/tools/sdk/Cargo.toml:8` advertises `Engine::builder` in the package description. Neither exists. The `Engine` impl block (lines 41-149) only defines `test_world()` + 9 port accessors + 4 facade handles. Consumers following the library-docs.md sample will fail to compile on `Engine::builder()`.
- **expected:** `docs/library-docs.md:11-22` (Construction section) — `let engine = Engine::builder()...build().await?;`
- **evidence:**
  ```rust
  // crates/tools/sdk/src/engine.rs:41-148
  impl Engine {
      /// Constructs a fresh `Engine` with all 7 ports wired to the
      /// in-memory testkit impls and the default `InProcessEventBus`.
      /// Convenience for consumer tests and dogfooding.
      #[must_use]
      pub fn test_world() -> Self { ... }

      /// Returns a reference to the storage adapter.
      #[must_use]
      pub fn storage(&self) -> &Arc<dyn StorageAdapter> { ... }
      // ... 8 more accessors, NO `builder()` factory ...
  }
  ```

---

### FINDING 2

- **id:** CLI-SDK-002
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `crates/tools/sdk/src/engine.rs:258-289`
- **description:** `EngineBuilder::build()` is declared `pub fn build(self) -> Result<Engine, SdkError>` (synchronous, no `async`). `docs/library-docs.md:22` shows the consumer sample ending with `.build().await?;`. Awaiting a non-async function is a compile error. The library-docs sample is non-functional as written.
- **expected:** `docs/library-docs.md:22` — `.build().await?;`
- **evidence:**
  ```rust
  // crates/tools/sdk/src/engine.rs:258-289
  /// Builds the `Engine`. Returns `Err(SdkError::MissingPort)`
  /// if any required port is not set.
  pub fn build(self) -> Result<Engine, SdkError> {
      let storage = self.storage.ok_or(SdkError::MissingPort("storage"))?;
      let auth = self.auth.ok_or(SdkError::MissingPort("auth"))?;
      // ...
  }
  ```

---

### FINDING 3

- **id:** CLI-SDK-003
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `crates/tools/sdk/src/engine.rs:125-149` (Engine impl) vs `docs/library-docs.md:179-189`
- **description:** `docs/library-docs.md:179-189` (Common Workflows) advertises 8 high-level accessors that do not exist on `Engine`: `engine.students()`, `engine.students().admit(cmd)`, `engine.students().promote(cmd)`, `engine.attendance().mark(cmd)`, `engine.assessment().enter_marks(cmd)`, `engine.assessment().publish_result(cmd)`, `engine.fees().generate_invoice(cmd)`, `engine.fees().record_payment(cmd)`, `engine.hr().generate_payroll(cmd)`. None of these methods are defined. The Engine exposes only `storage/auth/notify/payment/files/integrations/bus/clock/id_gen/admission/attendance/payment_svc/notify_svc` (lines 71-149). None of the documented high-level workflows are reachable.
- **expected:** `docs/library-docs.md:179-189` (Common Workflows section).
- **evidence:**
  ```rust
  // crates/tools/sdk/src/engine.rs:125-149
      /// Returns a handle to the admission facade.
      #[must_use]
      pub fn admission(&self) -> AdmissionService<'_> { ... }
      /// Returns a handle to the attendance facade.
      #[must_use]
      pub fn attendance(&self) -> AttendanceService<'_> { ... }
      /// Returns a handle to the payment facade.
      #[must_use]
      pub fn payment_svc(&self) -> PaymentService<'_> { ... }
      /// Returns a handle to the notification facade.
      #[must_use]
      pub fn notify_svc(&self) -> NotificationService<'_> { ... }
      // NO `students()`, NO `fees()`, NO `assessment()`, NO `hr()`,
      // NO `rbac()`, NO `events()`
  ```

---

### FINDING 4

- **id:** CLI-SDK-004
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `crates/tools/sdk/src/facade.rs:11-31`
- **description:** `AdmissionService` is documented in `library-docs.md` (Common Workflows § `engine.students().admit(cmd)`) as the surface for admitting students. The actual `AdmissionService` impl only exposes `storage()` (line 29) which returns a `&Arc<dyn StorageAdapter>`. There is no `admit(cmd)` method, no `promote(cmd)` method, no academic command surface. The docstring at lines 22-27 admits this as a stub ("this facade is a thin re-export of the storage adapter for now"). Consumers cannot admit a student through the SDK.
- **expected:** `docs/library-docs.md:181` — `engine.students().admit(cmd).await?`
- **evidence:**
  ```rust
  // crates/tools/sdk/src/facade.rs:11-31
  pub struct AdmissionService<'a> {
      engine: &'a Engine,
  }

  impl<'a> AdmissionService<'a> {
      /// Constructs a new admission service bound to `engine`.
      #[must_use]
      pub fn new(engine: &'a Engine) -> Self { ... }

      /// Admits a student. The full command flow lives in the
      /// academic domain crate; this facade is a thin
      /// re-export of the storage adapter for now.
      /// ...
      #[must_use]
      pub fn storage(&self) -> &std::sync::Arc<dyn educore_storage::StorageAdapter> {
          self.engine.storage()
      }
  }
  ```

---

### FINDING 5

- **id:** CLI-SDK-005
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `crates/tools/cli/src/commands.rs:23-53` (fn admit)
- **description:** The `admit` CLI command takes `--first`, `--last`, `--class`, `--section` as required args (per `lib.rs:30-43`) but never persists a student. The handler only parses the school UUID (line 35), generates two synthetic UUIDs (`student_id`, `correlation_id`), and prints them in a JSON envelope. `class_id` and `section_id` are parsed from the CLI args (lines 36-37) then dropped without ever being used. No call to `educore_academic::services::admit_student` or any storage insert. The handoff doc (`PHASE-16-HANDOFF.md` "What's wired and working" § `educore-cli`) describes this as "academic admission" — the runtime behavior is a JSON echo.
- **expected:** `docs/build-plan.md:1664` — "`educore-cli`: a sample binary demonstrating daily operations (admit a student, mark attendance, record a payment)"
- **evidence:**
  ```rust
  // crates/tools/cli/src/commands.rs:23-53
  pub async fn admit(
      school: String, first: String, last: String, class: String, section: String,
  ) -> Result<()> {
      let _world = test_world();
      let school_id = parse_school(&school)?;
      let class_id = parse_uuid(&class, "class")?;   // parsed, dropped
      let section_id = parse_uuid(&section, "section")?;  // parsed, dropped
      let g = SystemIdGen;
      let user = g.next_user_id();
      let corr = g.next_correlation_id();
      let _ctx = TenantContext::for_user(school_id, user, corr, UserType::SchoolAdmin);
      let student_id = g.next_uuid();  // synthetic
      let out = serde_json::json!({
          "school_id": school_id.as_uuid().to_string(),
          "student_id": student_id.to_string(),
          "first_name": first, "last_name": last,
          "class_id": class_id.to_string(), "section_id": section_id.to_string(),
          // ...
      });
      tracing::info!("{}", serde_json::to_string_pretty(&out)?);
      Ok(())
  }
  ```

---

### FINDING 6

- **id:** CLI-SDK-006
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `docs/library-docs.md:223-230` vs workspace tree
- **description:** `docs/library-docs.md:223-230` (Sample Programs section) states: "A complete `examples/admit_and_enroll.rs` is provided in the workspace that..." and proceeds to list 7 workflow steps (admits, enrolls, attendance, marks, fees, pays, prints). No `examples/` directory exists at the workspace root or under `crates/educore/`. Consumers searching for the canonical example find nothing.
- **expected:** `docs/library-docs.md:223-230` — Sample Programs section promises `examples/admit_and_enroll.rs`.
- **evidence:**
  ```text
  $ ls examples/ 2>&1
  ls: cannot access 'examples/': No such file or directory
  $ find . -name "admit_and_enroll.rs" -not -path "./target/*" -not -path "./schoolify/*"
  (no output)
  ```

---

### FINDING 7

- **id:** CLI-SDK-007
- **area:** tools-cli-sdk
- **severity:** Critical
- **location:** `docs/library-docs.md:103-119, 124-132, 215-221` vs `crates/tools/sdk/src/engine.rs:71-149`
- **description:** Three accessor methods are documented in library-docs.md but absent on Engine: `engine.auth()` is documented (line 104) but the actual `auth()` method (engine.rs:77) returns `&Arc<dyn AuthProvider>` not a session; `engine.events().subscribe::<T>()` is documented (lines 124-132) but `Engine` has no `events()` method; `engine.rbac().has_capability(...)` is documented (lines 215-221) but `Engine` has no `rbac()` method. None of these 3 accessor paths compile.
- **expected:** `docs/library-docs.md:103-119` (Subscribing to Events), `docs/library-docs.md:215-221` (Capability Check).
- **evidence:**
  ```rust
  // crates/tools/sdk/src/engine.rs:71-149 -- the complete accessor set
      pub fn storage(&self) -> &Arc<dyn StorageAdapter> { ... }
      pub fn auth(&self) -> &Arc<dyn AuthProvider> { ... }
      pub fn notify(&self) -> &Arc<dyn NotificationProvider> { ... }
      pub fn payment(&self) -> &Arc<dyn PaymentProvider> { ... }
      pub fn files(&self) -> &Arc<dyn FileStorage> { ... }
      pub fn integrations(&self) -> &Arc<dyn IntegrationGateway> { ... }
      pub fn bus(&self) -> &Arc<dyn EventBus> { ... }
      pub fn clock(&self) -> &Arc<dyn Clock> { ... }
      pub fn id_gen(&self) -> &Arc<dyn IdGenerator> { ... }
      pub fn admission(&self) -> AdmissionService<'_> { ... }
      pub fn attendance(&self) -> AttendanceService<'_> { ... }
      pub fn payment_svc(&self) -> PaymentService<'_> { ... }
      pub fn notify_svc(&self) -> NotificationService<'_> { ... }
      // NO `events()`, NO `rbac()`, NO `students()`, NO `fees()`,
      // NO `assessment()`, NO `hr()`
  ```

---

### FINDING 8

- **id:** CLI-SDK-008
- **area:** tools-cli-sdk
- **severity:** High
- **location:** `crates/tools/sdk/Cargo.toml:15-37` (dependencies)
- **description:** The SDK Cargo.toml declares 14 dependencies that are not referenced anywhere in `crates/tools/sdk/src/`: `educore-platform` (no import), `educore-rbac` (no import), `educore-academic` (no import — `AdmissionService` does NOT call academic), `educore-assessment` (no import), `educore-attendance` (no import), `educore-finance` (no import), `educore-hr` (no import), `educore-event-bus` (no import — only the cross-cutting `educore_events::event_bus::EventBus` trait is used), `async-trait` (no `#[async_trait]` macro used), `tracing` (no `tracing::*` macro used), `parking_lot` (no `parking_lot::*` import), `bytes` (no `bytes::*` import), `anyhow` (no `anyhow::*` macro used). Bloat in the dependency closure.
- **expected:** `docs/code-standards.md` § "Engine Rules" — minimal dependency surface.
- **evidence:**
  ```bash
  $ grep -h "use " crates/tools/sdk/src/*.rs
  use std::sync::Arc;
  use educore_auth::port::AuthProvider;
  use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
  use educore_events::event_bus::EventBus;
  use educore_files::port::FileStorage;
  use educore_integrations::port::IntegrationGateway;
  use educore_notify::port::NotificationProvider;
  use educore_payment::port::PaymentProvider;
  use educore_storage::StorageAdapter;
  use educore_testkit::TestkitWorld;
  use crate::errors::SdkError;
  use crate::facade::{AdmissionService, AttendanceService, NotificationService, PaymentService};
  use thiserror::Error;
  use educore_core::tenant::TenantContext;
  use crate::engine::Engine;
  use crate::errors::SdkError;
  ```

---

### FINDING 9

- **id:** CLI-SDK-009
- **area:** tools-cli-sdk
- **severity:** High
- **location:** `crates/tools/cli/src/commands.rs:73-89` (StudentAttendanceRow construction)
- **description:** The `attendance` CLI command accepts `--student <uuid>` (line 88 in lib.rs) and writes a `StudentAttendanceRow`, but `student_record_id` (line 85), `class_id` (line 86), and `section_id` (line 87) are populated from random `g.next_uuid()` calls rather than the student's actual record/class/section. The persisted row has no referential integrity to the student. Any downstream read or query by class/section will miss these rows.
- **expected:** `docs/build-plan.md:1664` — "demonstrating daily operations (admit a student, mark attendance, record a payment)". A real attendance row would resolve `student_record_id`/`class_id`/`section_id` from the student lookup, not fabricate them.
- **evidence:**
  ```rust
  // crates/tools/cli/src/commands.rs:73-89
  let row = StudentAttendanceRow {
      school_id,
      id: g.next_uuid(),
      student_id,
      student_record_id: g.next_uuid(),  // <-- synthetic, not resolved from student_id
      class_id: g.next_uuid(),           // <-- synthetic
      section_id: g.next_uuid(),         // <-- synthetic
      attendance_date: date,
      // ...
  };
  ```

---

### FINDING 10

- **id:** CLI-SDK-010
- **area:** tools-cli-sdk
- **severity:** High
- **location:** `crates/tools/cli/src/commands.rs:25-27, 60-62, 137-139`
- **description:** All three CLI handlers call `let _world = test_world();` (or `let world = test_world();`) — the in-memory testkit backend. The CLI is documented in `docs/build-plan.md:1664` as "a sample binary demonstrating daily operations". But every CLI invocation spawns a fresh in-process `TestkitWorld` that dies when the binary exits. There is no persistence, no shared state between invocations, no config file, no daemon mode. The CLI cannot actually be used to "admit a student" in any operational sense.
- **expected:** `docs/build-plan.md:1664` — "demonstrating daily operations (admit a student, mark attendance, record a payment) for developer ergonomics and dogfooding."
- **evidence:**
  ```rust
  // crates/tools/cli/src/commands.rs:25-27 (admit)
      let _world = test_world();
      let school_id = parse_school(&school)?;
      let class_id = parse_uuid(&class, "class")?;
  // crates/tools/cli/src/commands.rs:60-62 (attendance)
      let world = test_world();
      let school_id = parse_school(&school)?;
      let student_id = parse_uuid(&student, "student")?;
  // crates/tools/cli/src/commands.rs:137-139 (payment)
      let world = test_world();
      let school_id = parse_school(&school)?;
  ```

---

### FINDING 11

- **id:** CLI-SDK-011
- **area:** tools-cli-sdk
- **severity:** High
- **location:** `crates/educore/src/lib.rs:75-83` (prelude module)
- **description:** `docs/library-docs.md:8` shows the consumer sample as `use educore::prelude::*;`. The umbrella's prelude module (lines 75-83) re-exports only `educore_core`, `educore_events`, `educore_operations`, `educore_platform`, `educore_rbac`, `educore_sdk`, `educore_settings`. It does NOT flatten `Engine`, `EngineBuilder`, or any of the 4 facade services. Even though `educore_sdk` is re-exported as a module, consumers using `use educore::prelude::*;` must still write `educore::sdk::Engine` or `educore::prelude::educore_sdk::Engine` to reach the SDK surface. The promised "prelude" does not provide the documented ergonomic surface.
- **expected:** `docs/library-docs.md:8` — `use educore::prelude::*;`
- **evidence:**
  ```rust
  // crates/educore/src/lib.rs:75-83
  pub mod prelude {
      pub use educore_core;
      pub use educore_events;
      pub use educore_operations;
      pub use educore_platform;
      pub use educore_rbac;
      pub use educore_sdk;
      pub use educore_settings;
  }
  ```

---

### FINDING 12

- **id:** CLI-SDK-012
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/tools/sdk/src/facade.rs:11-31` (AdmissionService)
- **description:** `AdmissionService` is the only facade service that does not have a domain-specific method. The docstring at lines 22-27 admits "the full command flow lives in the academic domain crate; this facade is a thin re-export of the storage adapter for now". The `AdmissionService` is effectively a typedef for `&Arc<dyn StorageAdapter>` — it does not implement the `AdmitStudentCommand` flow that the academic domain crate provides (`crates/domains/academic/src/services.rs:90`). Consumers wiring the SDK's `AdmissionService` get a storage handle, not an admission workflow.
- **expected:** `docs/library-docs.md:179-189` — Common Workflows section promises `engine.students().admit(cmd).await?`.
- **evidence:**
  ```rust
  // crates/tools/sdk/src/facade.rs:11-31
  impl<'a> AdmissionService<'a> {
      /// Admits a student. The full command flow lives in the
      /// academic domain crate; this facade is a thin
      /// re-export of the storage adapter for now.
      /// ...
      #[must_use]
      pub fn storage(&self) -> &std::sync::Arc<dyn educore_storage::StorageAdapter> {
          self.engine.storage()
      }
  }
  ```

---

### FINDING 13

- **id:** CLI-SDK-013
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/tools/sdk/src/facade.rs:53-72, 86-99, 119-128`
- **description:** All three facade methods (`AttendanceService::mark_bulk`, `PaymentService::charge`, `NotificationService::send`) map the underlying port error to `SdkError::Facade { service: "<name>", message: e.to_string() }`. The `to_string()` call discards the structured error variant. Consumers cannot match on `DomainError::NotFound`, `DomainError::Conflict`, `DomainError::Validation`, etc. — they only see a string. Per `docs/library-docs.md:191-201` (Error Handling section), consumers expect to pattern-match `DomainError` variants; the SDK facade breaks that contract.
- **expected:** `docs/library-docs.md:191-201` — Error Handling section demonstrates `match ... Err(DomainError::Validation { ... }) ...`.
- **evidence:**
  ```rust
  // crates/tools/sdk/src/facade.rs:53-72 (mark_bulk)
      pub async fn mark_bulk(
          &self,
          ctx: &TenantContext,
          rows: &[...],
      ) -> Result<(), SdkError> {
          self.engine
              .storage()
              .bulk_insert_student_attendances(ctx, rows)
              .await
              .map_err(|e| SdkError::Facade {
                  service: "AttendanceService",
                  message: e.to_string(),  // discards structured error
              })
      }
  // crates/tools/sdk/src/facade.rs:86-99 (charge)
              .map_err(|e| SdkError::Facade {
                  service: "PaymentService",
                  message: e.to_string(),
              })
  // crates/tools/sdk/src/facade.rs:119-128 (send)
              .map_err(|e| SdkError::Facade {
                  service: "NotificationService",
                  message: e.to_string(),
              })
  ```

---

### FINDING 14

- **id:** CLI-SDK-014
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/tools/cli/src/commands.rs:160-164` (payment handler)
- **description:** The `payment` CLI handler takes `--invoice <uuid>` (line 64) and wraps it as `CustomerRef::External(CustomerId::new(invoice))` (line 164). The `invoice` arg is parsed as a string with no UUID validation — `CustomerId::new` accepts any string. There is no validation that the invoice exists, no lookup against `finance::Invoice`, no FK constraint enforcement. The CLI happily charges a payment against a non-existent invoice id.
- **expected:** `docs/ports/payments.md` — payment port contract requires invoice reference to be a valid, existing entity.
- **evidence:**
  ```rust
  // crates/tools/cli/src/commands.rs:160-164
      let req = ChargeRequest::new(
          ctx,
          money,
          payment_method,
          CustomerRef::External(CustomerId::new(invoice)),  // string parsed, not validated
          g.next_idempotency_key(),
          g.next_correlation_id(),
      );
  ```

---

### FINDING 15

- **id:** CLI-SDK-015
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/tools/cli/src/commands.rs:109-126` (attendance handler)
- **description:** After `world.storage.bulk_insert_student_attendances(...)` (lines 109-112), the handler prints the synthetic `row.id` (line 119) without reading back from the storage adapter to verify the row was actually persisted. If the insert silently failed (e.g. storage adapter returns `Ok(())` without writing), the CLI exits with success and reports a row id that doesn't exist. There is no read-back assertion.
- **expected:** Standard CLI hygiene for state-mutating operations — verify the write by re-reading or by asserting on a side effect.
- **evidence:**
  ```rust
  // crates/tools/cli/src/commands.rs:108-126
      world
          .storage
          .bulk_insert_student_attendances(&ctx, std::slice::from_ref(&row))
          .await
          .map_err(|e| anyhow!("attendance insert failed: {e}"))?;

      let out = serde_json::json!({
          "row_id": row.id.to_string(),  // never read back to confirm persistence
          "school_id": school_id.as_uuid().to_string(),
          "student_id": student_id.to_string(),
          "attendance_date": date.to_string(),
          "attendance_type": attendance_type,
          "is_absent": is_absent,
      });
      tracing::info!("{}", serde_json::to_string_pretty(&out)?);
      Ok(())
  ```

---

### FINDING 16

- **id:** CLI-SDK-016
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/educore/tests/consumer_e2e.rs:65-70` (admit section)
- **description:** The test function is named `consumer_e2e_admission_attendance_payment_notify_chain`, and the section markers (lines 65-70) declare an "admit section" owned by the E.4 subagent. The section body is: `let _storage = engine.admission().storage(); let student_id = g.next_uuid();`. No admission occurs — the student_id is synthetic and `_storage` is unused. The test claims to verify the full admission workflow but the admit step is a no-op. The remaining 3 steps (attendance + payment + notify) run successfully against an engine that has no admitted students.
- **expected:** `docs/build-plan.md:1668-1670` — Phase 16 task #5: "A consumer-facing integration test in `crates/educore/tests/consumer_e2e.rs` that uses the SDK + testkit to run a full admission workflow without docker."
- **evidence:**
  ```rust
  // crates/educore/tests/consumer_e2e.rs:65-70
      // === admit section begin (owner: E.4) ===
      let _storage = engine.admission().storage();
      let student_id = g.next_uuid();
      // === admit section end ===
  ```

---

### FINDING 17

- **id:** CLI-SDK-017
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `crates/tools/sdk/README.md` (missing) vs `crates/tools/sdk/` (directory listing)
- **description:** The SDK crate has no `README.md`. `crates/tools/cli/README.md` exists but is 1 paragraph that doesn't match what `lib.rs:1-9` advertises (the lib claims 3 subcommands but the README says "starting the runtime, applying migrations, running scheduled jobs, and draining the outbox" — capabilities that don't exist in the binary). Consumers navigating to the SDK crate find zero onboarding material.
- **expected:** Per AGENTS.md Crate Layout convention — each crate ships with a README describing its purpose and entry point.
- **evidence:**
  ```bash
  $ ls crates/tools/sdk/
  Cargo.toml  src        # no README.md
  $ cat crates/tools/cli/README.md
  # educore-cli
  The cli crate is a sample binary that demonstrates how a consumer
  wires the Educore for daily operations — starting the runtime,
  applying migrations, running scheduled jobs, and draining the outbox.
  ```
  The CLI lib.rs advertises 3 subcommands (`admit`, `attendance`, `payment`) — the README mentions none of them and instead describes 4 unrelated operations.

---

### FINDING 18

- **id:** CLI-SDK-018
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `docs/library-docs.md:11-22` vs `crates/tools/sdk/src/engine.rs:244-256`
- **description:** The library-docs sample shows `Engine::builder().clock(SystemClock::new()).id_gen(UuidV7Generator::new())`. The actual builder at lines 244-256 expects `clock(Arc<dyn Clock>)` (so `Arc::new(SystemClock)`, not `SystemClock::new()`) and `id_gen(Arc<dyn IdGenerator>)` (so `Arc::new(SystemIdGen)`, not `UuidV7Generator::new()` — that type doesn't exist in the engine; the only impl is `SystemIdGen` at `crates/tools/sdk/src/engine.rs:65`).
- **expected:** `docs/library-docs.md:18-19` — Construction section.
- **evidence:**
  ```rust
  // crates/tools/sdk/src/engine.rs:244-256
      pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
          self.clock = Some(clock);
          self
      }

      pub fn id_gen(mut self, id_gen: Arc<dyn IdGenerator>) -> Self {
          self.id_gen = Some(id_gen);
          self
      }
  ```

---

### FINDING 19

- **id:** CLI-SDK-019
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `docs/library-docs.md:14-15` vs `crates/adapters/event-bus/src/in_process.rs` (InProcessEventBus type)
- **description:** `docs/library-docs.md:15` shows `.event_bus(InProcessBus::new())`. The actual type is `InProcessEventBus` (not `InProcessBus`) and lives in the `educore_event_bus` crate, not the umbrella `educore` crate. Consumers following the sample will fail to compile on `InProcessBus`.
- **expected:** `docs/library-docs.md:14-15` — Construction section.
- **evidence:**
  ```rust
  // docs/library-docs.md:13-15
      .event_bus(InProcessBus::new())   // <-- InProcessBus doesn't exist
  // crates/adapters/event-bus/README.md:43-46
  use educore_event_bus::InProcessEventBus;
  let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
  ```

---

### FINDING 20

- **id:** CLI-SDK-020
- **area:** tools-cli-sdk
- **severity:** Medium
- **location:** `docs/library-docs.md:124-132` vs `crates/cross-cutting/events/src/event_bus.rs:48`
- **description:** `docs/library-docs.md:128-132` shows `engine.events().subscribe::<StudentAdmitted>().await?`. The actual `EventBus::subscribe` signature is `async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>` (no generic type parameter; takes `SubscribeOptions`). The library-docs sample uses the wrong call shape (turbofish on a non-generic method) AND calls a non-existent `engine.events()` accessor (see CLI-SDK-007).
- **expected:** `docs/library-docs.md:124-132` (Subscribing to Events section).
- **evidence:**
  ```rust
  // crates/cross-cutting/events/src/event_bus.rs:35-48
  pub trait EventBus: Send + Sync + fmt::Debug {
      // ...
      async fn subscribe(&self, options: SubscribeOptions) -> Result<Box<dyn EventSubscription>>;
  }
  ```

---

### FINDING 21

- **id:** CLI-SDK-021
- **area:** tools-cli-sdk
- **severity:** Low
- **location:** `crates/tools/sdk/src/errors.rs:18-21`
- **description:** The `SdkError` enum declares an `Engine(String)` variant (lines 18-21) but `grep -rn 'SdkError::Engine' crates/tools/sdk/src/` returns no matches — the variant is never constructed by any SDK method. The three facade methods (`mark_bulk`, `charge`, `send`) all use `SdkError::Facade { service, message }` instead. The `Engine` variant is dead code in the public error enum.
- **expected:** `crates/tools/sdk/src/errors.rs` (current public surface) — variant should be reachable or removed.
- **evidence:**
  ```rust
  // crates/tools/sdk/src/errors.rs:5-21
  #[derive(Debug, Error)]
  pub enum SdkError {
      /// A required port was not provided to the builder.
      #[error("missing required port: {0}")]
      MissingPort(&'static str),

      /// A facade method delegation failed.
      #[error("facade error in {service}: {message}")]
      Facade { service: &'static str, message: String },

      /// The underlying engine returned an error.
      #[error("engine error: {0}")]
      Engine(String),  // never constructed in src/
  }
  ```

---

### FINDING 22

- **id:** CLI-SDK-022
- **area:** tools-cli-sdk
- **severity:** Low
- **location:** `crates/tools/cli/src/lib.rs:8`
- **description:** `crates/tools/cli/src/lib.rs:8` exports `pub use commands::{admit, attendance, dispatch, payment};` — making the three handler functions `pub` at the crate root. The lib is only consumed as a binary (`main.rs` only uses `dispatch`). Public re-export of internal handlers leaks the implementation surface; consumers wiring the CLI as a library could call `admit`/`attendance`/`payment` directly, bypassing the `Cli` parser and the `Command` enum.
- **expected:** `crates/tools/cli/Cargo.toml:13` — `[[bin]]` declares this as a binary crate, not a library; library consumers should not be importing its internals.
- **evidence:**
  ```rust
  // crates/tools/cli/src/lib.rs:1-8
  //! ...
  #![forbid(unsafe_code)]
  #![deny(missing_docs)]

  pub mod commands;
  pub use commands::{admit, attendance, dispatch, payment};
  ```

---
