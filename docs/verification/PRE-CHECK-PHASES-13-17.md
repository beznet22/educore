# Pre-Check: Phases 13-17

> Read-only snapshot of the pre-implementation state for the 5 unimplemented phases (13, 14, 15, 16, 17). Consumed by the per-phase subagents (V14-V18) when they write the per-phase verify prompts. After the verify prompts are written, this file is historical.

---

## Phase 13 — Events (calendar)

**Build-plan coverage:** (lines 1400-1435)

- **Deliverables.** `educore-events-domain`. **Distinct** from `educore-events` (the envelope crate from Phase 2). This is the calendar domain: `CalendarEvent`, `Holiday`, `Incident`, `Weekend`.
- **Tasks.** Aggregates per `docs/specs/events/aggregates.md`: `CalendarEvent`, `Holiday`, `Incident`, `Weekend`; recurrence rule service (RFC 5545 RRULE subset); integration test for a weekly recurring event with holiday exclusion; phase completion documentation (handoff + next-phase prompt).
- **Exit criteria.** As Phases 3-4, plus the RRULE test.
- **Risks.** *The two `events` crates are easy to confuse.* Mitigation: `crates/cross-cutting/events/` is the envelope; `crates/cross-cutting/events-domain/` is the calendar. Document this explicitly in both `lib.rs` headers and in `AGENTS.md`.

**Spec:** `docs/specs/events/` (11 files) — `overview.md`, `aggregates.md`, `commands.md`, `entities.md`, `events.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`. Command catalog also exists at `docs/commands/events.md`; event catalog at `docs/events/events.md`. The `events/aggregates.md` lists **7 root aggregates** (CalendarEvent, Holiday, Weekend, Incident, AssignIncident, IncidentComment, CalendarSetting) — the build-plan's "4 aggregates" line is a headline subset.

**Build-plan section:** lines 1400-1435 (between `## Phase 13` and `## Phase 14` headings).

**Coverage rows in `docs/coverage.toml` for this phase:**
- `events_calendar_events_aggregate` — `status = "Pending"` (spec: `docs/specs/events/aggregates.md`)

**Scaffold crates (line count in src/lib.rs):**
- `crates/cross-cutting/events-domain/` — `Cargo.toml` exists (19 lines, declares deps on `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `educore-settings`); `src/lib.rs` is **27 lines** (the standard `PACKAGE_NAME`/`PACKAGE_VERSION` scaffold); scaffold-only.
- `crates/cross-cutting/events/` (envelope crate, **closed in Phase 2**, do not touch): `src/lib.rs` is a real implementation, not a scaffold.

**Pre-implementation gaps found:**
- **Coverage row undercount.** Only 1 row exists for Phase 13 (`events_calendar_events_aggregate`). The spec defines 7 root aggregates; the headline 4 (CalendarEvent, Holiday, Incident, Weekend) should each have a `events_<name>_aggregate` row. The `events_calendar_settings_aggregate`, `events_assign_incidents_aggregate`, and `events_incident_comments_aggregate` are also missing. Per prior-phase precedent (Phase 9 = 10 rows for 6 aggregates, Phase 10 = 13 rows for 26 aggregates), the verify prompt for Phase 13 should expect the subagent to add ~4-7 coverage rows.
- **Aggregate-row mismatch.** The build-plan's "4 aggregates" headline subset is consistent with the `events_calendar_events_aggregate` coverage row id (singular); but the spec's 7-root view implies the matrix is sparse. The verify prompt should confirm whether the subagent is shipping spec-faithful (7 aggregates) or headline-only (4 aggregates).
- **No `educore-academic` dep in `events-domain/Cargo.toml`.** The spec's `overview.md` `## Dependencies` lists `educore-academic` for `ClassId`, `SectionId`, `SubjectId` audience references. The current scaffold has no `educore-academic` dep. Either the Phase 13 subagent adds it, or the dep is added in a Prereq commit (mirroring Phase 5's `educore-assessment` cross-crate dep, or Phase 5's `educore-event-bus` for tests).
- **`educore-settings` dep declared but not implemented.** The `events-domain/Cargo.toml` declares `educore-settings` as a dep, but `educore-settings` is **also a Phase 14 deliverable**. This is a forward-reference risk: the workspace will not build Phase 13 alone (settings is unimplemented). The verify prompt for Phase 13 should confirm the subagent either (a) drops the `educore-settings` dep until Phase 14 lands, or (b) keeps it and accepts that Phase 13 needs Phase 14 to compile (which is a circular dep on the build order).
- **Stale progress-tracker row** is not yet validated for Phase 13 (the `docs/progress-tracker.md` row should exist as "Pending").

**Carry-forward rules relevant to this phase:**
- The two `events` crates naming risk (per `AGENTS.md` § "Note on `educore-events` vs `educore-events-domain`" and the build-plan's "Risks" section). The verify prompt's "Per-Phase Specifics" should remind the subagent to set the `lib.rs` header distinction and to read `AGENTS.md` § "Tier System" + § "Note on `educore-events` vs `educore-events-domain`" before writing code.
- **Phase 9 OQ #6 (`LibrarySettings` per-school ownership)** implies the `Settings` domain will own per-school config; Phase 14 ships it; the events-domain references it.
- **No `educore-finance` dep** rule (Phase 8 OQ #6 + Phase 10 OQ #3 + Phase 11 OQ #3 carry-forward) — events-domain has no finance cross-domain coord.
- **No `educore-notify` dep** rule (Phase 10 OQ #4 + Phase 11 OQ #4 carry-forward) — events-domain emits `EventCreated`/`HolidayCreated`/`IncidentReported` facts; the bus handles notification fan-out (the `overview.md` `## Cross-Domain Impact` section says communication subscribes to dispatch notifications).
- **Spec-faithful vs headline interpretation precedent** (Phase 8 = 11 aggregates, Phase 9 = 6 aggregates, Phase 10 = 26 aggregates, Phase 11 = 3 aggregates) — all phases after Phase 8 ship spec-faithful, including the "lower-traffic" surfaces as first-class ports. The verify prompt should confirm the subagent reads `Phase 9/10/11` hand-offs for the pattern.
- The `educore-events` envelope crate (Phase 2) provides `DomainEvent` trait, `EventEnvelope`, and the bus port. The events-domain crate depends on it for emitting domain events. The subagent must NOT duplicate the envelope types in the calendar domain.

---

## Phase 14 — Settings + Operations

**Build-plan coverage:** (lines 1437-1468)

- **Deliverables.** `educore-settings`, `educore-operations`.
- **Tasks.** `educore-settings`: per-school configuration, language phrases, base setups. Aggregates per `docs/specs/settings/aggregates.md`. `educore-operations` (new in v1): school-day operations — `AcademicSession`, `BellSchedule`, `Substitution`, `TimetableChange`, `DailyDiary`. Aggregates per `docs/specs/operations/aggregates.md`. Integration tests per domain, as in Phases 3-4. Phase completion documentation.
- **Exit criteria.** As Phases 3-4, for both crates.
- **Coverage matrix updates.** All `settings_*` and `operations_*` rows.

**Spec:** Two spec directories:
- `docs/specs/settings/` (11 files) — `overview.md`, `aggregates.md`, `commands.md`, `entities.md`, `events.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`. Command catalog at `docs/commands/settings.md`; event catalog at `docs/events/settings.md`. The `aggregates.md` is **526 lines** (the largest aggregate file in the engine after finance); it covers `GeneralSettings`, `Language`, `DateFormat`, `TimeZone`, `Currency`, `Theme`, `EmailSetting`, `SmsSetting`, `EmailTemplate`, `SmsTemplate`, `BaseSetup`, `Session`, `PrefixSetting`, `Role`, `Module`, `SidebarMenu`, etc.
- `docs/specs/operations/` (11 files) — `overview.md`, `aggregates.md`, `commands.md`, `entities.md`, `events.md`, `permissions.md`, `repositories.md`, `services.md`, `tables.md`, `value-objects.md`, `workflows.md`. Command catalog at `docs/commands/operations.md`; event catalog at `docs/events/operations.md`. The `aggregates.md` covers `Backup`, `Job`, `FailedJob`, `SystemVersion`, `UserLog`, `RuntimeMaintenance`.

**Build-plan section:** lines 1437-1468 (between `## Phase 14` and `## Phase 15` headings).

**Coverage rows in `docs/coverage.toml` for this phase:**
- `settings_school_settings_aggregate` — `status = "Pending"` (spec: `docs/specs/settings/aggregates.md`)
- `operations_bell_schedules_aggregate` — `status = "Pending"` (spec: `docs/specs/operations/aggregates.md`)

**Scaffold crates (line count in src/lib.rs):**
- `crates/cross-cutting/settings/` — `Cargo.toml` exists (13 lines, `educore-core` + scaffold-only note); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/cross-cutting/operations/` — `Cargo.toml` exists (14 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only); also has a 3-line `README.md`.

**Pre-implementation gaps found:**
- **Coverage row undercount.** Only 2 rows exist for Phase 14 (1 per crate). The settings spec has ~15+ aggregates (GeneralSettings, Language, DateFormat, TimeZone, Currency, Theme, EmailSetting, EmailTemplate, SmsSetting, SmsTemplate, BaseSetup, Session, PrefixSetting, Role, Module, SidebarMenu, …); the operations spec has ~6 aggregates (Backup, Job, FailedJob, SystemVersion, UserLog, RuntimeMaintenance). Per prior-phase precedent (Phase 9: 10 rows for 6 aggregates, Phase 10: 13 rows for 26 aggregates), the verify prompt should expect ~15-20 settings rows and ~6-8 operations rows added by the subagent.
- **Headline-aggregate naming mismatch.** The `operations_bell_schedules_aggregate` coverage row id and the build-plan's headline list (`AcademicSession`, `BellSchedule`, `Substitution`, `TimetableChange`, `DailyDiary`) **do not appear in the operations spec's `aggregates.md`**, which lists `Backup`, `Job`, `FailedJob`, `SystemVersion`, `UserLog`, `RuntimeMaintenance`. This is a **substantial spec/build-plan divergence**: the build-plan's headline list describes school-day operations (bell schedules, substitutions) that the spec models as operations-domain *infrastructure* (backups, jobs, runtime maintenance). The verify prompt must ask the subagent to either (a) add the 5 build-plan-named aggregates to the spec, or (b) update the build-plan and the coverage row id to match the spec's 6 aggregates.
- **`educore-settings` is a forward-dep of `educore-events-domain`** (Phase 13's `Cargo.toml` already declares it). The Phase 13 cargo.toml and the events-domain overview confirm this. The verify prompt for Phase 14 should remind the subagent that the events-domain crate's `educore-settings` dep means the settings API surface is part of the Phase 13 + Phase 14 contract.
- **No `Cargo.toml` deps declared for either Phase 14 crate** — both are scaffold-only with no `educore-core`/`educore-platform`/`educore-rbac`/`educore-events` deps declared. The verify prompt should remind the subagent to add the standard deps (matching the other cross-cutting crates' pattern).

**Carry-forward rules relevant to this phase:**
- **Phase 9 OQ #6 (`LibrarySettings` per-school ownership)** — `LibrarySettings` is owned by the `settings` domain. Phase 14 ships it; the library crate reads but does not own it. The verify prompt should ask the subagent to confirm the `LibrarySettings` value object is exported from `educore-settings` (the Phase 9 hand-off is the source of truth for the contract).
- **Phase 11 OQ #2 (`FileStorage` port)** — `FileReference` is currently re-exported from `educore-platform`. The actual file storage port lands in Phase 15; the operations spec's `Backup` aggregate references `FileStorage`. The verify prompt should ask the subagent how the `Backup` aggregate references files — is it the typed value object (Phase 11 pattern) or the port (deferred to Phase 15)?
- **Spec-faithful vs headline interpretation precedent** (Phase 8 = 11 aggregates, Phase 9 = 6, Phase 10 = 26, Phase 11 = 3, Phase 13 = 7) — the verify prompt should confirm the subagent reads the Phase 9/10/11 hand-offs for the pattern.
- **No `educore-finance` dep** (Phase 8 OQ #6 + Phase 10 OQ #3 + Phase 11 OQ #3 carry-forward) — settings has no finance cross-domain coord.
- **No `educore-notify` dep** (Phase 10 OQ #4 + Phase 11 OQ #4 carry-forward) — settings has no notification fan-out.
- **No `educore-attendance` dep** (Phase 10 OQ #5 + Phase 11 OQ #5 carry-forward) — settings has no attendance integration.
- **Build-plan `educore-operations` "new in v1" note** — the operations domain is a new bounded context introduced at Phase 14 (no prior scaffold analog). The verify prompt should confirm the subagent reads `AGENTS.md` § "Tier System" to verify the `cross-cutting/` tier placement is correct (operations was previously envisioned as a domain; the current scaffold places it in `cross-cutting/`).

---

## Phase 15 — Port adapters

**Build-plan coverage:** (lines 1470-1527)

- **Deliverables.** `educore-auth`, `educore-notify`, `educore-payment`, `educore-files`, `educore-integrations`. Port trait **plus** one reference impl per port.
- **Tasks.** `educore-auth`: `AuthProvider` port + `JwtAuthProvider` reference impl. `educore-notify`: `NotificationProvider` port + email and SMS reference impls. `educore-payment`: `PaymentProvider` port + a Stripe reference impl. `educore-files`: `FileStorage` port + S3 and local reference impls. `educore-integrations`: `IntegrationGateway` port + LMS and video-conferencing reference impls. For each port, an integration test that wires a real reference impl against a docker-compose stack (mailhog, localstack S3, stripe-mock, etc.). Phase completion documentation.
- **Exit criteria.** All 5 port traits have a Rust trait definition and a reference impl; `Box<dyn NotificationProvider>` (and the other four ports) compiles — verifying object safety; each reference impl has a green integration test; `cargo test --workspace` green.
- **Risks.** *Stripe API drift.* Mitigation: pin the stripe-mock version. *S3 SDK weight.* Mitigation: feature-gate it; consumers who only need the local impl don't pay the binary-size cost.
- **Note on `educore-event-bus`.** The user's pre-check prompt mentions "Phase 15 — Port adapters: `educore-auth` + `educore-event-bus` (already done in Phase 2) + ...". The build-plan's Phase 15 deliverables are 5 port adapters and do **not** include `educore-event-bus` (it closed in Phase 2). The verify prompt should treat `educore-event-bus` as out-of-scope for Phase 15.

**Spec:** No domain spec (this is an adapters-tier phase, not a domain). Port contracts live at `docs/ports/`:
- `docs/ports/authentication.md` (7,076 bytes)
- `docs/ports/notifications.md` (7,767 bytes)
- `docs/ports/payments.md` (7,849 bytes)
- `docs/ports/file-storage.md` (6,195 bytes)
- `docs/ports/integrations.md` (7,841 bytes)
- (Plus the cross-cutting `event-bus.md`, `storage.md`, `sync.md` — not in Phase 15 scope.)

**Build-plan section:** lines 1470-1527 (between `## Phase 15` and `## Phase 16` headings).

**Coverage rows in `docs/coverage.toml` for this phase:**
- `auth_provider_port` — `status = "Pending"` (spec: `docs/ports/authentication.md`)
- `auth_provider_jwt_impl` — `status = "Pending"` (spec: `docs/ports/authentication.md`)
- `notification_provider_port` — `status = "Pending"` (spec: `docs/ports/notifications.md`)
- `payment_provider_port` — `status = "Pending"` (spec: `docs/ports/payments.md`)
- `file_storage_port` — `status = "Pending"` (spec: `docs/ports/file-storage.md`)
- `integration_gateway_port` — `status = "Pending"` (spec: `docs/ports/integrations.md`)

**Scaffold crates (line count in src/lib.rs):**
- `crates/adapters/auth/` — `Cargo.toml` exists (20 lines; declares `educore-core`, `educore-platform`, `educore-rbac`, `educore-events`, `tokio`, `async-trait`); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/adapters/notify/` — `Cargo.toml` exists (~20 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/adapters/payment/` — `Cargo.toml` exists (~20 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/adapters/files/` — `Cargo.toml` exists (~20 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/adapters/integrations/` — `Cargo.toml` exists (~20 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/adapters/event-bus/` — **NOT in Phase 15 scope**; closed in Phase 2; `src/lib.rs` is 99 lines (real impl).

**Pre-implementation gaps found:**
- **Coverage row undercount.** Only 6 rows exist for Phase 15. The build-plan says "All 5 port traits have a Rust trait definition and a reference impl" — so 5 ports + 5 reference impls = 10 rows. Currently:
  - `auth_provider_port` + `auth_provider_jwt_impl` ✓ (2 rows for auth)
  - `notification_provider_port` only (1 row; **missing**: `notification_provider_email_impl`, `notification_provider_sms_impl`)
  - `payment_provider_port` only (1 row; **missing**: `payment_provider_stripe_impl`)
  - `file_storage_port` only (1 row; **missing**: `file_storage_s3_impl`, `file_storage_local_impl`)
  - `integration_gateway_port` only (1 row; **missing**: `integration_gateway_lms_impl`, `integration_gateway_video_conferencing_impl`)
  - **Missing**: ~7 reference-impl rows.
  - The verify prompt should expect the subagent to add the ~7 missing reference-impl rows.
- **No `Box<dyn XxxProvider>` object-safety smoke tests** are yet defined in coverage rows. The build-plan exit criterion 2 says "`Box<dyn NotificationProvider>` (and the other four ports) compiles" — this should be tracked as a row or as a test convention.
- **No `docker-compose.yml`** for the integration tests (mailhog, localstack S3, stripe-mock). The build-plan task 6 says "wires a real reference impl against a docker-compose stack" — the verify prompt should ask the subagent to add the compose file at `tools/docker-compose.yml` or a similar location.
- **No Stripe SDK pin in workspace `Cargo.toml`.** The verify prompt should ask the subagent to add `stripe` (or `stripe-rust`) to the workspace deps; the Phase 0 ADR-015 MSRV pinning policy applies.
- **No `aws-sdk-s3` feature flag** in scaffold. The build-plan's "Risks" section says "feature-gate it" — the verify prompt should confirm the subagent reads the feature-flag plan in `docs/decisions/ADR-015-ExternalCrates.md`.

**Carry-forward rules relevant to this phase:**
- **Phase 7 Q10 (deprecated `PaymentProvider` trait in `educore-finance`)** — the trait is marked `#[deprecated(since = "0.7.0")]` and moves to `educore-payment` in Phase 15. The verify prompt should remind the subagent to move the trait (not duplicate it) and to update all call sites (HR's `PayrollService`, finance's `FeesPaymentService`).
- **Phase 9 OQ #1 (fine payment integration)** — `FineCalculated` is emitted by library; the finance subscriber is not wired. Phase 15's `educore-payment` is the real wiring target. The verify prompt should ask the subagent to add a finance-side payment subscriber (or document the deferral as an open question).
- **Phase 10 OQ #2 (NotificationProvider port)** — the `NotificationDispatchService` is events-only; Phase 15 ships the real port. The verify prompt should ask the subagent to wire the bus subscriber from `educore-communication` to a real `NotificationProvider` impl.
- **Phase 11 OQ #2 (FileStorage port)** — `FileReference` is the value object; `FileStorage::put/get/delete` is deferred to Phase 15. The verify prompt should ask the subagent to add the real impl and to wire `educore-documents` (FormDownload file) and `educore-operations` (Backup file) to the port.
- **Phase 10 OQ #4 + Phase 11 OQ #4** — the verify prompt should remind the subagent that the `educore-notify` crate is the real wiring target; no `educore-notify` dep was added in Phases 10 or 11.
- **`educore-finance` `PaymentProvider` trait move** — the verify prompt should ask the subagent to read `docs/handoff/PHASE-7-HANDOFF.md` Q10 for the move plan.
- **Tier boundary rules** (`AGENTS.md` § "Tier System") — adapter crates may depend on `infra` + `cross-cutting` but **not** on `domains` or `tools`. The Phase 15 ports are wired by domain crates, not the other way around. The verify prompt should confirm the subagent does not add domain deps to the adapter crates.

---

## Phase 16 — Test infrastructure + SDK

**Build-plan coverage:** (lines 1529-1585)

- **Deliverables.** `educore-testkit`, `educore-storage-parity`, `educore-sdk`, `educore-cli`.
- **Tasks.** `educore-testkit`: in-memory impls of all 6 ports (`StorageAdapter`, `AuthProvider`, `NotificationProvider`, `PaymentProvider`, `FileStorage`, `EventBus`). `educore-storage-parity`: a cross-adapter parity test suite (PG, MySQL, SQLite, in-memory testkit) asserting identical observable behavior. `educore-sdk`: a high-level consumer facade — `Engine::builder()` wires the umbrella crate's re-exports into a single configuration surface. `educore-cli`: a sample binary demonstrating daily operations. A consumer-facing integration test in `crates/educore/tests/consumer_e2e.rs` that uses the SDK + testkit. Phase completion documentation.
- **Exit criteria.** `educore-testkit` ports compile and pass their own unit tests; parity suite runs in <60 s on a developer laptop and is green on all four backends; CLI binary builds and the three sample commands work end-to-end against an in-memory backend; `cargo test --workspace` green.
- **Risks.** *Parity suite flakiness across backends.* Mitigation: assert against a documented behavior matrix, not byte-identical SQL output.

**Spec:** No domain spec (this is a tools-tier phase, not a domain). Reference docs:
- `docs/library-docs.md` (the SDK is "the public face of the engine for the consumer").
- `docs/ports/storage.md` (for the parity suite behavior matrix).
- `docs/guides/saas-backend.md` (for the consumer_e2e test scenarios).

**Build-plan section:** lines 1529-1585 (between `## Phase 16` and `## Phase 17` headings).

**Coverage rows in `docs/coverage.toml` for this phase:**
- `testkit_in_memory_adapters` — `status = "Pending"` (spec: `docs/ports/storage.md`)
- `sdk_high_level_facade` — `status = "Pending"` (spec: `docs/library-docs.md`)
- `cli_sample_binary` — `status = "Pending"` (spec: `docs/guides/saas-backend.md`)

**Scaffold crates (line count in src/lib.rs):**
- `crates/tools/testkit/` — `Cargo.toml` exists (14 lines, `# No production dependencies yet — scaffold only`); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/tools/storage-parity/` — `Cargo.toml` exists (1,588 bytes, has been expanded during Phase 0-11 work — see `crates/tools/storage-parity/Cargo.toml`); `src/lib.rs` is **27 lines** (the prelude was added in earlier phases but the suite is still scaffold); `tests/` dir exists (multiple integration tests added by Phases 2-11).
- `crates/tools/sdk/` — `Cargo.toml` exists (~20 lines, scaffold-only); `src/lib.rs` is **27 lines** (scaffold-only).
- `crates/tools/cli/` — `Cargo.toml` exists (18 lines, declares a `[[bin]]` for `educore-cli` at `src/main.rs`); `src/lib.rs` is **27 lines** (scaffold-only); has a 391-byte `README.md`. **Note:** the `[[bin]]` points at `src/main.rs`, not `src/lib.rs`; Phase 16 must populate `main.rs` (not just `lib.rs`).

**Pre-implementation gaps found:**
- **Coverage row undercount.** Only 3 rows exist for Phase 16. The build-plan task 2 says "a cross-adapter parity test suite that runs the same scenario against PG, MySQL, SQLite, and the in-memory testkit impl, asserting identical observable behavior (modulo documented dialect differences)". Per prior-phase precedent, each adapter scenario + each parity test class should have its own row. The verify prompt should expect the subagent to add rows for: `parity_test_outbox`, `parity_test_audit_log`, `parity_test_event_log`, `parity_test_idempotency`, `parity_test_transaction`, `cli_admit_student`, `cli_mark_attendance`, `cli_record_payment`, `consumer_e2e_admission_workflow`, etc.
- **Coverage row `storage_parity_suite` (phase 0, Pending)** — this row is orphaned between Phase 0 and Phase 16. The verify prompt should ask the subagent to either (a) flip it to `Tested` in Phase 16, or (b) re-tag it to `phase = 16` and flip it. The build-plan § "Orphaned items" table acknowledges this.
- **No `crates/educore/tests/consumer_e2e.rs` file** — the umbrella crate's `tests/` directory does not yet have a consumer-facing test. The verify prompt should ask the subagent to add the file per the build-plan task 5.
- **`educore-cli` `[[bin]]` path mismatch** — the scaffold's `Cargo.toml` points the binary at `src/main.rs`, but the `src/` directory only has `lib.rs`. Phase 16 must add `src/main.rs`.
- **No `Engine::builder()` type** in the SDK. The verify prompt should ask the subagent to design the builder pattern (it is the canonical public surface; consumers wire all 34 crates through it).
- **Parity suite behavior matrix not documented.** The build-plan's "Risks" says "assert against a documented behavior matrix". The verify prompt should ask the subagent to add the matrix as `docs/ports/storage.md` § "Parity behavior matrix" (or a new `docs/research/parity-matrix.md`).
- **No docker-compose / no testkit integration test infra.** The build-plan task 2 says "modulo documented dialect differences" — the verify prompt should ask the subagent to document which test classes are env-gated vs always-on.

**Carry-forward rules relevant to this phase:**
- **Phase 0 OQ (`storage_parity_suite` row orphaned between Phase 0 and Phase 16)** — the verify prompt should ask the subagent to close this gap.
- **Phase 1 outcome (`Per-call transaction model`)** — the parity suite must exercise the flag-based transaction model across all 4 SQL adapters. The verify prompt should remind the subagent that the testkit's in-memory `StorageAdapter` is the 4th impl and must mirror the at-least-once dedup contract.
- **Phase 2 OQ #5 (flag-based transactions validation)** — closed by Phase 3-11 vertical-slice tests; Phase 16's parity suite is the broader validation target.
- **All Phase 13-15 deliverable crates** (events-domain, settings, operations, auth, notify, payment, files, integrations) are wired through the SDK builder. The verify prompt should ask the subagent to confirm the SDK exports all 34 crates (per the umbrella's `crates/educore/src/lib.rs` re-exports).
- **Tier boundary rules** (`AGENTS.md` § "Tier System") — tools-tier crates may depend on `infra` + `cross-cutting` + `domains` (the only tier that may). The verify prompt should confirm the subagent reads the tier table.
- **`educore-cli` is a `[[bin]]` not a `lib`** — the build-plan task 4 says "a sample binary demonstrating daily operations". The verify prompt should ask the subagent to confirm the binary is **not** re-exported via the umbrella (`educore::cli` is not a thing; the CLI is invoked via `cargo run -p educore-cli -- admit-student`).

---

## Phase 17 — Production readiness

**Build-plan coverage:** (lines 1587-1641)

- **Deliverables.** Integration test suite, load test, cross-compile, security review, documentation audit.
- **Tasks.** Multi-tenant integration test suite — 50+ scenarios from `docs/guides/saas-backend.md`, run nightly against all three backends. Load test: 10k students, bulk fee invoice generation (Phase 7 finance); target p95 < 500 ms; documented in `docs/research/load-test-results.md`. Cross-compile verification on Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64, Windows x86_64. Security review of every public command surface (TenantContext, RBAC capability, idempotency). Documentation audit against the 10-point validation checklist in `AGENTS.md`. Phase completion documentation.
- **Exit criteria.** All 10 validation questions in `AGENTS.md` answer "Yes". 5 cross-compile targets green. CI green on all five targets. Load-test report committed under `docs/research/`. Security-review report committed under `docs/decisions/`.
- **Risks.** *Cross-compile surprises (Windows path handling, musl allocator).* Mitigation: smoke-test the SDK on each target in Phase 16, before Phase 17 hardens the matrix.

**Spec:** No domain spec. Reference docs:
- `docs/guides/saas-backend.md` (the multi-tenant scenarios; "building a production SaaS on top of the library").
- `AGENTS.md` § "Validation Checklist" (the 10-point documentation audit).
- `docs/research/load-test-results.md` (to be created in Phase 17).
- `docs/decisions/ADR-019-SecurityReview.md` (or similar; to be created in Phase 17).

**Build-plan section:** lines 1587-1641 (between `## Phase 17` and `## The Coverage Matrix` headings).

**Coverage rows in `docs/coverage.toml` for this phase:**
- `multi_tenant_integration_suite` — `status = "Pending"` (spec: `docs/guides/saas-backend.md`; crate: `educore-sdk`)
- `load_test_10k_students` — `status = "Pending"` (spec: `docs/guides/saas-backend.md`; crate: `educore-finance`)
- `cross_compile_5_targets` — `status = "Pending"` (spec: `AGENTS.md`; crate: `workspace`)
- `security_review_all_commands` — `status = "Pending"` (spec: `AGENTS.md`; crate: `workspace`)
- `documentation_audit_10_point` — `status = "Pending"` (spec: `AGENTS.md`; crate: `workspace`)

**Scaffold crates (line count in src/lib.rs):**
- **No new crates.** Phase 17 is a production-hardening phase that touches existing crates (`educore-finance` for the load test, `educore-sdk` for the consumer integration suite) and workspace-level infra (CI matrix, security review document, docs audit).
- The load test crate (if any) is a new file — likely `crates/educore-finance/tests/load_test_bulk_invoice.rs` or a new `tools/load-test/` directory. Not yet scaffolded.

**Pre-implementation gaps found:**
- **No `docs/research/load-test-results.md`** — must be created in Phase 17.
- **No `docs/decisions/ADR-NNN-SecurityReview.md`** (or similar) — must be created in Phase 17.
- **No `.github/workflows/` CI matrix** for the 5 cross-compile targets. The verify prompt should ask the subagent to add the matrix to the existing CI workflow (or create a new one).
- **No `tools/scripts/check-graph-freshness.sh`** is yet integrated into CI (per the build-plan § "The No-Gaps Gates" item 4). The verify prompt should ask the subagent to add it to the CI workflow.
- **No `tools/load-test/` directory** for the load test. The verify prompt should ask the subagent to add a benchmark crate (e.g. `tools/load-test/Cargo.toml`).
- **No nightly CI job** for the multi-tenant integration suite. The verify prompt should ask the subagent to add a scheduled workflow.
- **The 10-point validation checklist** is in `AGENTS.md` § "Validation Checklist" (per the build-plan's "Authoritative Documents" list). The verify prompt should ask the subagent to walk each of the 10 questions and produce a "Yes/No" report.
- **Coverage row closure plan** — Phase 17 is the terminal phase ("The matrix reaches 100%"). All `Pending` rows from Phases 13-16 must be flipped to `Tested` (or `Implemented`, or `Deprecated`) in Phase 17 or the prior phases. The verify prompt should ask the subagent to audit the full `coverage.toml` and identify any remaining `Pending` rows.

**Carry-forward rules relevant to this phase:**
- **All OQs from Phases 0-16** are carry-forwards. Phase 17's verification step should confirm the subagent reads all 12 hand-off docs and resolves all open questions (or explicitly defers them with an ADR).
- **The cross-compile matrix** requires `rustls` (not `native-tls`) per `AGENTS.md` § "Code Standards" and `docs/decisions/ADR-015-ExternalCrates.md`. The verify prompt should ask the subagent to verify the MSRV floor (1.75) pins all 11 MSRV-conflict crates.
- **The 5 cross-compile targets** include `aarch64-unknown-linux-musl` (the musl allocator is the documented risk). The verify prompt should ask the subagent to smoke-test the SDK on each target in CI.
- **The security review** must check (a) `TenantContext` is read by every command handler, (b) the `school_id` matches the command's `school_id`, (c) RBAC capability is checked, (d) idempotency is enforced. The verify prompt should ask the subagent to produce a per-command matrix (one row per command in `docs/commands/<domain>.md`).
- **The 10-point documentation audit** is in `AGENTS.md` § "Validation Checklist" (per the build-plan's "Authoritative Documents" list). The verify prompt should ask the subagent to walk the checklist and produce a "Yes/No" report.
- **No `phase-18-prompt.md`** is created (per the build-plan: "Phase 17 is the last phase — do not create a `phase-18-prompt.md` unless a Phase 18+ is explicitly planned").
- **Final hand-off** is `docs/handoff/PHASE-17-HANDOFF.md` (the only hand-off that does not have a next-phase prompt).

---

## Summary table

| Phase | Title | Spec? | Build-plan lines | Coverage rows (Pending/Tested) | Scaffold crates | Gaps |
|---|---|---|---|---|---|---|
| 13 | Events (calendar) | `docs/specs/events/` (11 files) + `docs/commands/events.md` + `docs/events/events.md` | 1400-1435 | 1 Pending / 0 Tested | `crates/cross-cutting/events-domain/` (27-line lib.rs, 19-line Cargo.toml with forward-dep on `educore-settings`) | Coverage row undercount (1 row for 7 spec aggregates); build-plan "4 aggregates" vs spec "7 aggregates" headline mismatch; `educore-academic` dep missing from `Cargo.toml`; `educore-settings` forward-dep; `events-domain` is *not* in `crates/domains/` (lives in `cross-cutting/`) |
| 14 | Settings + Operations | `docs/specs/settings/` (11 files) + `docs/specs/operations/` (11 files) + `docs/commands/{settings,operations}.md` + `docs/events/{settings,operations}.md` | 1437-1468 | 2 Pending / 0 Tested | `crates/cross-cutting/settings/` (27-line lib.rs, 13-line Cargo.toml, no deps declared); `crates/cross-cutting/operations/` (27-line lib.rs, 14-line Cargo.toml, no deps declared) | Coverage row undercount (2 rows for ~15 settings + 6 operations aggregates); build-plan headline list (`AcademicSession`, `BellSchedule`, `Substitution`, `TimetableChange`, `DailyDiary`) **does not match** spec's 6 aggregates (`Backup`, `Job`, `FailedJob`, `SystemVersion`, `UserLog`, `RuntimeMaintenance`) — substantial divergence; both Cargo.tomls have no deps declared |
| 15 | Port adapters | No domain spec; 5 port contracts in `docs/ports/` (`authentication.md`, `notifications.md`, `payments.md`, `file-storage.md`, `integrations.md`) | 1470-1527 | 6 Pending / 0 Tested | `crates/adapters/auth/` (27-line lib.rs, 20-line Cargo.toml with 6 deps); `crates/adapters/notify/` (27-line lib.rs, ~20-line Cargo.toml); `crates/adapters/payment/` (27-line lib.rs, ~20-line Cargo.toml); `crates/adapters/files/` (27-line lib.rs, ~20-line Cargo.toml); `crates/adapters/integrations/` (27-line lib.rs, ~20-line Cargo.toml); (`crates/adapters/event-bus/` is Phase 2, **not** in scope) | Coverage row undercount (6 rows for 5 ports × 7 reference impls = 12 rows); ~7 reference-impl rows missing; no `Box<dyn X>` object-safety rows; no `docker-compose.yml` for integration tests; no Stripe SDK pin; no `aws-sdk-s3` feature flag plan |
| 16 | Test infrastructure + SDK + CLI | No domain spec; references `docs/library-docs.md`, `docs/ports/storage.md`, `docs/guides/saas-backend.md` | 1529-1585 | 3 Pending / 0 Tested | `crates/tools/testkit/` (27-line lib.rs, 14-line Cargo.toml, no deps); `crates/tools/storage-parity/` (27-line lib.rs, 1,588-byte Cargo.toml, has integration test dir from prior phases); `crates/tools/sdk/` (27-line lib.rs, ~20-line Cargo.toml); `crates/tools/cli/` (27-line lib.rs, 18-line Cargo.toml with `[[bin]]` pointing at non-existent `src/main.rs`) | Coverage row undercount (3 rows; build-plan implies ~10 rows for parity suite + CLI + SDK); `storage_parity_suite` row orphaned between Phase 0 and Phase 16 (build-plan's "Orphaned items" table); no `crates/educore/tests/consumer_e2e.rs`; `educore-cli` `[[bin]]` path mismatch (points at `src/main.rs` which doesn't exist); no `Engine::builder()` type; no parity behavior matrix |
| 17 | Production readiness | No domain spec; references `docs/guides/saas-backend.md`, `AGENTS.md` § "Validation Checklist", `docs/research/load-test-results.md` (TBD), `docs/decisions/ADR-NNN-SecurityReview.md` (TBD) | 1587-1641 | 5 Pending / 0 Tested | None (no new crates; touches `educore-finance` for load test, `educore-sdk` for consumer integration suite, and workspace-level CI/docs) | No `docs/research/load-test-results.md`; no security-review ADR; no `.github/workflows/` CI matrix for 5 cross-compile targets; no `tools/scripts/check-graph-freshness.sh` in CI; no `tools/load-test/` directory; no nightly CI job for multi-tenant suite; no 10-point validation audit report; 100% coverage closure plan missing |
