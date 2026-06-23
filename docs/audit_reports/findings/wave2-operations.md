## Wave 2 Operations Domain Audit Report

**Scope:** `crates/cross-cutting/operations/`, `docs/specs/operations/`, `docs/commands/operations.md`, `docs/events/operations.md`, `docs/coverage.toml` (operations rows), `docs/handoff/PHASE-14-HANDOFF.md`.

**Total findings:** 56

---

### FINDING 1

- **id:** CROSSCUT-OPS-001
- **area:** cross-cutting
- **severity:** Critical
- **location:** `crates/cross-cutting/operations/src/lib.rs:24` and `crates/cross-cutting/operations/src/aggregate.rs:9,  commands.rs:10-11,  entities.rs:6,  events.rs:8-9,  query.rs:6-7,  repository.rs:9-10,  services.rs:7-8,  value_objects.rs:6-7`
- **description:** The crate-level `#![deny(missing_docs)]` at `lib.rs:24` is silently shadowed by `#![allow(missing_docs)]` at the top of every other source file. The compiler accepts the inner allows, so the deny has no effect anywhere in the crate. Every public item in `aggregate.rs`, `commands.rs`, `entities.rs`, `events.rs`, `query.rs`, `repository.rs`, `services.rs`, and `value_objects.rs` is published without rustdoc.
- **expected:** Public items in the operations crate are documented with rustdoc per `AGENTS.md` and `docs/code-standards.md` ("All public APIs are documented with rustdoc; `#![deny(missing_docs)]`").
- **evidence:** `crates/cross-cutting/operations/src/lib.rs:24` `#![deny(missing_docs)]` vs. `crates/cross-cutting/operations/src/aggregate.rs:9` `#![allow(missing_docs, dead_code, clippy::all)]` (and the matching allow lines on the other 7 source files).

### FINDING 2

- **id:** CROSSCUT-OPS-002
- **area:** cross-cutting
- **severity:** Critical
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:39-56` (`Backup`), `:142-159` (`Job`), `:276-293` (`FailedJob`), `:342-355` (`SystemVersion`), `:439-451` (`VersionHistory`), `:490-507` (`UserLog`), `:551-568` (`MaintenanceSetting`), `:659-681` (`Sidebar`)
- **description:** None of the 8 root aggregates has a `delete` (or `soft_delete`) method. The spec defines delete commands for `Backup` (`DeleteBackupCommand`), `FailedJob` (`DeleteFailedJobCommand`), `MaintenanceSetting` (implicit in `ConfigureMaintenanceCommand` reconfig flow), and `Sidebar` (`DeleteSidebarEntryCommand`), and the spec says "the engine refuses to delete system-defined sidebar rows" - a rule that can only be enforced by a `delete` method on the aggregate. The Phase 14 hand-off acknowledges "all root aggregates set `active_status = false` on delete" but no aggregate has the method to do it.
- **expected:** Each root aggregate (other than the append-only `UserLog` / `VersionHistory`) exposes a `delete(actor, at, event_id)` (or `soft_delete`) method that sets `active_status = false` and emits the corresponding event.
- **evidence:** Spec at `docs/specs/operations/aggregates.md:32` lists `DeleteBackup`, `:115` `DeleteFailedJob`, `:284` `DeleteSidebarEntry`. `crates/cross-cutting/operations/src/aggregate.rs:39-56` (`Backup` struct + impl block, no `delete`); `:142-159` (`Job`, no `delete`); `:659-781` (`Sidebar`, only `new`, `is_system`, `reorder`, `set_ignore`, `set_active` - no `delete`).

### FINDING 3

- **id:** CROSSCUT-OPS-003
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:659-781` (`Sidebar`)
- **description:** The `Sidebar` aggregate has no `update` method to back `UpdateSidebarEntryCommand`, which can patch `position`, `level`, `ignore`, and `active_status`. The aggregate has `set_ignore` and `set_active` setters but no `set_position`, `set_level`, or general `update`; the `Sidebar` invariant ("`rbac_sidebars.is_system_defined` flags system-defined rows; the engine refuses to delete them") has no enforcement point.
- **expected:** A `Sidebar::update(position, level, ignore, active_status, actor, at, event_id)` method (or per-field setters) that emits `SidebarEntryUpdated`.
- **evidence:** `docs/specs/operations/commands.rs:436-444` `UpdateSidebarEntryCommand` carries `position`, `level`, `ignore`, `active_status`. `crates/cross-cutting/operations/src/aggregate.rs:703-777` impl block has `new`, `is_system`, `reorder`, `set_ignore`, `set_active` only. `docs/specs/operations/aggregates.md:270-273` is_system_defined invariant.

### FINDING 4

- **id:** CROSSCUT-OPS-004
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`)
- **description:** The `Backup` aggregate uses raw `bool` for `active_status` and `restore_in_progress`, plus a raw `bool` for `MaintenanceSetting::maintenance_mode`. The spec defines typed wrappers `BackupActiveStatus` (`docs/specs/operations/value-objects.md:35`), `MaintenanceMode` (`:94`), and `SidebarIsSaas` (`:106`) that are absent from `value_objects.rs`.
- **expected:** Per the engine rule "Compile-time safety over strings" (and its analogue for booleans), the aggregate fields should use the typed wrappers `BackupActiveStatus`, `MaintenanceMode`, and the per-school sidebar should carry a typed `SidebarIsSaas`.
- **evidence:** `docs/specs/operations/value-objects.md:34-35` `| BackupActiveStatus | bool |`; `:94` `| MaintenanceMode | bool |`; `:106` `| SidebarIsSaas | bool |`. `crates/cross-cutting/operations/src/value_objects.rs` has no struct named `BackupActiveStatus`, `MaintenanceMode`, or `SidebarIsSaas`. `crates/cross-cutting/operations/src/aggregate.rs:46-47` `pub active_status: bool, pub restore_in_progress: bool,`; `:558` `pub maintenance_mode: bool,`.

### FINDING 5

- **id:** CROSSCUT-OPS-005
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/value_objects.rs` (entire file) and `entities.rs:31-200`
- **description:** Six typed value objects mandated by `docs/specs/operations/value-objects.md` are absent: `JobReservedAt` (`:44`), `JobCreatedAt` (`:46`), `FailedAt` (`:58`), `MigrationName` (`:113`), `MigrationBatch` (`:114`), and the entire OAuth / PasswordReset section (`:118-135`). The first three are engine-owned; the Migration / OAuth / PasswordReset types are port-driven per `docs/specs/operations/commands.md:313-342` and `tables.md:87-95`, so the spec is OK on those being port-driven - but `JobReservedAt`, `JobCreatedAt`, and `FailedAt` are owned by the operations domain's `Job` / `FailedJob` aggregates and have no typed wrapper in code.
- **expected:** `JobReservedAt`, `JobCreatedAt`, `FailedAt` typed wrappers in `value_objects.rs` (parallel to `JobAvailableAt` which is already a `pub use Timestamp as JobAvailableAt;` alias at `:1037`).
- **evidence:** `docs/specs/operations/value-objects.md:44-46` lists `JobReservedAt`, `JobAvailableAt`, `JobCreatedAt`; `:58` lists `FailedAt`. `crates/cross-cutting/operations/src/value_objects.rs:1037` `pub use educore_core::value_objects::Timestamp as JobAvailableAt;` - only one of the three Job timestamp types is wrapped; the other two are missing.

### FINDING 6

- **id:** CROSSCUT-OPS-006
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/entities.rs:31-201, 541-573` (entity structs)
- **description:** 13 entity-level typed identifiers mandated by `docs/specs/operations/entities.md` are absent from `value_objects.rs` and the entity fields use raw `Uuid`: `FailedJobExceptionId`, `SystemVersionFeatureId`, `VersionHistoryNoteId`, `UserLogContextId`, `UserLogSessionId`, `MaintenanceOverrideId`, `SidebarEntryId`, `SidebarRouteId`, `JobQueueId`, `SystemVersionManifestId`, `SidebarRoleBindingId`, `SystemVersionCapabilityId`, `VersionMigrationId`. The spec's `entities.md` declares each of these as `Identity: <Name>(SchoolId, Uuid)` or `<Name>(Uuid)` and the engine's safety rule requires typed identifiers.
- **expected:** 13 typed identifier structs in `value_objects.rs`; entity fields in `entities.rs` use those typed ids instead of raw `Uuid`.
- **evidence:** `docs/specs/operations/entities.md:54-55` `FailedJobException | Identity: FailedJobExceptionId(Uuid)`; `:64-65` `SystemVersionFeatureId(Uuid)`; `:73-74` `VersionHistoryNoteId(Uuid)`; `:81-82` `UserLogContextId(SchoolId, Uuid)`; `:91-92` `UserLogSessionId(SchoolId, Uuid)`; `:99-100` `MaintenanceOverrideId(SchoolId, Uuid)`; `:118-119` `SidebarEntryId(SchoolId, Uuid)`; `:126-127` `SidebarRouteId(SchoolId, Uuid)`; `:154-155` `JobQueueId(SchoolId, Uuid)`; `:171-172` `SystemVersionManifestId(Uuid)`; `:179-180` `AuditPartitionId(SchoolId, Uuid)`; `:188-189` `SidebarRoleBindingId(SchoolId, Uuid)`; `:197-198` `SystemVersionCapabilityId(Uuid)`; `:206-208` `VersionMigrationId(Uuid)`. `crates/cross-cutting/operations/src/entities.rs:212, 222, 242, 252, 275, 285, 315-316, 347, 359, 611, 622, 715, 725, 747, 759` all use raw `Uuid`. `AuditPartitionId` and `BackupScheduleId`/`BackupRetentionId`/`JobAttemptId` are present, but the other 13 are not.

### FINDING 7

- **id:** CROSSCUT-OPS-007
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:99-101` (`JobService::is_reserved`)
- **description:** `JobService::is_reserved` ignores the `now: Timestamp` parameter that the spec mandates. The spec signature at `docs/specs/operations/services.md:27` is `pub fn is_reserved(job: &Job, now: Timestamp) -> bool`; the code at `services.rs:99-101` declares only `pub fn is_reserved(job: &Job) -> bool` and implements it as `matches!(job.status, JobStatus::Reserved) && job.reserved_at.is_some()`. The spec's intent (parity with `is_available` on line 106-108) is to compare the reservation timestamp against `now` so stale reservations can be detected.
- **expected:** `pub fn is_reserved(job: &Job, now: Timestamp) -> bool` with body that checks the reservation has not expired (e.g. `reserved_at + lease > now`).
- **evidence:** `docs/specs/operations/services.md:27` `pub fn is_reserved(job: &Job, now: Timestamp) -> bool { ... }`. `crates/cross-cutting/operations/src/services.rs:99-101` `pub fn is_reserved(job: &Job) -> bool { matches!(job.status, JobStatus::Reserved) && job.reserved_at.is_some() }`.

### FINDING 8

- **id:** CROSSCUT-OPS-008
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:181-200` (`FailedJobService::extract_exception_type`)
- **description:** The spec signature at `docs/specs/operations/services.md:44` is `pub fn extract_exception_type(exception: &str) -> Option<&'static str>`; the code at `services.rs:181` declares `pub fn extract_exception_type(exception: &str) -> Option<&str>`. The function returns a slice of the input `exception` (it cannot be `'static`), so the return type is at minimum inconsistent with the spec - a caller relying on `'static` will not compile against the actual signature.
- **expected:** Code matches spec exactly, or the spec is corrected and a return-type contract is documented.
- **evidence:** `docs/specs/operations/services.md:44` `pub fn extract_exception_type(exception: &str) -> Option<&'static str> { ... }`. `crates/cross-cutting/operations/src/services.rs:181` `pub fn extract_exception_type(exception: &str) -> Option<&str> {`.

### FINDING 9

- **id:** CROSSCUT-OPS-009
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:291-299` (`UserLogService::partition`)
- **description:** The spec signature at `docs/specs/operations/services.md:77` is `pub fn partition(log: &[UserLog], partition: AuditPartition) -> Vec<&UserLog>`; the code at `services.rs:291-299` takes `partition_label: &str` and matches on `correlation_id.to_string() == partition_label`. The spec intends a partition-bucketed view via the `AuditPartition` entity (with `label`, `period_start`, `period_end`, `entry_count`); the code's correlation-id-as-label proxy is a different (and more error-prone) partitioning scheme.
- **expected:** `pub fn partition(log: &[UserLog], partition: &AuditPartition) -> Vec<&UserLog>` that filters by `period_start <= logged_at <= period_end`.
- **evidence:** `docs/specs/operations/services.md:77` `pub fn partition(log: &[UserLog], partition: AuditPartition) -> Vec<&UserLog> { ... }`. `crates/cross-cutting/operations/src/services.rs:291-299` `pub fn partition<'a>(log: &'a [UserLog], partition_label: &str) -> Vec<&'a UserLog> { log.iter().filter(|l| l.correlation_id.to_string() == partition_label).collect() }`.

### FINDING 10

- **id:** CROSSCUT-OPS-010
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:362-372` (`MaintenanceService::applies_to_role`)
- **description:** The spec signature at `docs/specs/operations/services.md:96` is `pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool`; the code at `services.rs:362-372` takes `role_label: &str` and string-matches against `setting.applicable_for`. The `Role` struct is owned by the `educore-rbac` domain and the spec explicitly notes the cross-domain binding.
- **expected:** `pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool` (with a local `Role` mirror or via the rbac re-export).
- **evidence:** `docs/specs/operations/services.md:96` `pub fn applies_to_role(setting: &MaintenanceSetting, role: &Role) -> bool { ... }`. `crates/cross-cutting/operations/src/services.rs:362` `pub fn applies_to_role(setting: &MaintenanceSetting, role_label: &str) -> bool { if setting.applicable_for.is_all() { return true; } setting.applicable_for.as_str().split(',').map(str::trim).any(|s| s == role_label) }`.

### FINDING 11

- **id:** CROSSCUT-OPS-011
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:418-482` (`SidebarService`)
- **description:** The spec at `docs/specs/operations/services.md:108` defines `pub fn tree(entries: &[Sidebar], role: RoleId) -> Vec<SidebarNode>` returning a tree of `SidebarNode` values. The code defines only `pub fn tree_order(entries: &[Sidebar]) -> Vec<(crate::value_objects::SidebarId, i32)>` returning a flat `Vec<(id, level)>`. Both the `tree` method and the `SidebarNode` struct are absent.
- **expected:** A `SidebarNode` struct (with `id`, `level`, `children: Vec<SidebarNode>` or similar) and a `tree` method that builds the hierarchical projection for a role.
- **evidence:** `docs/specs/operations/services.md:108-111` `pub fn tree(entries: &[Sidebar], role: RoleId) -> Vec<SidebarNode> { ... }` (and 3 sub-methods). `crates/cross-cutting/operations/src/services.rs:426` `pub fn tree_order(entries: &[Sidebar]) -> Vec<(crate::value_objects::SidebarId, i32)> { ... }`. No `tree` method, no `SidebarNode` struct in `entities.rs` or `services.rs`.

### FINDING 12

- **id:** CROSSCUT-OPS-012
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:567-637` (policies and specifications)
- **description:** The spec at `docs/specs/operations/services.md:130-205` defines `Policy<Cmd>` and `Specification<T>` traits and gives the `OneRestoreInProgress`, `MaintenanceLockout`, and `DisableMaintenanceGuard` policies plus `ActiveBackups`, `DatabaseBackups`, `SuccessfulLogins`, and `FailedLogins` specifications as implementations of those traits. The code declares all seven as zero-sized unit structs with free `check` / `is_satisfied_by` functions, not as trait impls. A consumer that wants to call a generic policy dispatcher (e.g. `policy_registry.dispatch::<RestoreBackupCommand>(cmd)`) cannot do so.
- **expected:** `pub trait Policy<C: Command> { type Outcome; fn check(&self, ctx, cmd) -> Outcome; }` and `pub trait Specification<T> { fn is_satisfied_by(&self, t: &T) -> bool; }`, with the seven concrete types implementing the traits.
- **evidence:** `docs/specs/operations/services.md:131-137` `impl Policy<RestoreBackupCommand> for OneRestoreInProgress`; `:172-174` `impl Specification<Backup> for ActiveBackups`. `crates/cross-cutting/operations/src/services.rs:567-579` (`OneRestoreInProgress` with free `check` function, no `impl Policy`); `:646-654` (`ActiveBackups` with free `is_satisfied_by`, no `impl Specification`).

### FINDING 13

- **id:** CROSSCUT-OPS-013
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:620-637` (`DisableMaintenanceGuard`)
- **description:** The spec at `docs/specs/operations/permissions.md:124-129` and `services.md:153-162` says the guard "refuses to disable maintenance for the last remaining `SuperAdmin` in a school." The code at `services.rs:620-637` checks the actor's role label only (`super_admin` / `school_admin` / case-insensitive variants) and never checks the count of remaining `SuperAdmin` actors. The "self-authorization" semantic is missing.
- **expected:** A second-actor check: if the only remaining `SuperAdmin` for the school issues `DisableMaintenance`, the policy returns `Deny`.
- **evidence:** `docs/specs/operations/permissions.md:124-128` "The engine refuses to disable maintenance for the last remaining SuperAdmin in a school. A DisableMaintenance command from a non-SuperAdmin while maintenance is enabled is rejected with ForbiddenError::MaintenanceLockout." `crates/cross-cutting/operations/src/services.rs:620-637` `pub fn check(actor_role_label: &str) -> Result<(), String> { if actor_role_label.eq_ignore_ascii_case("super_admin") || ... { Ok(()) } else { Err(...) } }` - no count check, no school id parameter.

### FINDING 14

- **id:** CROSSCUT-OPS-014
- **area:** cross-cutting
- **severity:** Critical
- **location:** `crates/cross-cutting/operations/src/events.rs` and `docs/specs/operations/events.md` (no `UserLogDeleted` listed)
- **description:** The spec at `docs/specs/operations/workflows.md:72` says the nightly purge is "logged as a `DeleteUserLog` event for compliance." Neither `docs/specs/operations/events.md` (events catalog) nor `crates/cross-cutting/operations/src/events.rs` defines a `UserLogDeleted` / `DeleteUserLog` event. The compliance audit trail for the per-tenant `UserLog` retention sweep is therefore not modeled.
- **expected:** A `UserLogDeleted` event in `events.md` and `events.rs` (with `log_id`, `school_id`, `actor_id`, `purged_at`, etc.) that the retention job emits on every deleted row.
- **evidence:** `docs/specs/operations/workflows.md:64-73` "User Log Retention Workflow" - step 6: "The purge is logged as a DeleteUserLog event for compliance." `docs/specs/operations/events.md:228-242` lists `UserLogged` only. `crates/cross-cutting/operations/src/events.rs:972-1036` defines `UserLogged` only (no `UserLogDeleted` struct, no `EVENT_TYPE` for `"operations.user_log.deleted"`). `crates/cross-cutting/operations/src/commands.rs:307-336` defines `RecordUserLogCommand` only (no `DeleteUserLogCommand`).

### FINDING 15

- **id:** CROSSCUT-OPS-015
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs:561-637, 645-705` (policies and specifications) and `crates/cross-cutting/operations/src/lib.rs:26-34` (module surface)
- **description:** No `Policy` or `Specification` trait is declared in the crate (see Finding 12) and no subscriber or dispatcher is wired to enforce the `OneRestoreInProgress` policy on `RestoreBackupCommand`, the `MaintenanceLockout` policy on `LoginCommand`, or the `DisableMaintenanceGuard` policy on `DisableMaintenanceCommand`. The seven unit-struct policies are never invoked from any handler.
- **expected:** A `services::dispatch_policies` module (or `educore-core`-level policy registry) that wires each policy to the matching command handler.
- **evidence:** `docs/specs/operations/permissions.md:91-97` "Capabilities are checked at the command boundary: `if !engine.rbac().has(actor_id, Capability::OperationsBackupRestore).await? { return Err(DomainError::forbidden(...)) }`." `crates/cross-cutting/operations/src/services.rs:567-637` declares the policies but no handler calls them; no `mod dispatch` or `mod handler` exists in `lib.rs:26-34`.

### FINDING 16

- **id:** CROSSCUT-OPS-016
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/commands.rs:307-336` (`RecordUserLogCommand`) vs `docs/specs/operations/commands.md:225-235`
- **description:** `RecordUserLogCommand` in the code carries an extra `pub academic_id: Option<AcademicYearRef>` field that the spec does not declare. The spec's `RecordUserLogCommand` lists `tenant`, `user_id`, `role_id`, `ip_address`, `user_agent`, `outcome`, `failure_reason` - 7 fields. The code adds `academic_id` as the 8th. Per the engine's typed-wrapper rule, an extra undocumented field is a drift; the spec at `docs/specs/operations/aggregates.md:206-207` (UserLog invariant 7) talks about `UserLog::academic_id` so the field is required at the aggregate level, but the command shape is supposed to be the wire form.
- **expected:** Either the spec is updated to list `academic_id` in the `RecordUserLogCommand` struct literal, or the command is left at the spec's 7 fields and the `academic_id` is filled in by the dispatcher from the actor's current academic year.
- **evidence:** `docs/specs/operations/commands.md:225-235` `pub struct RecordUserLogCommand { pub tenant: TenantContext, pub user_id: UserId, pub role_id: RoleId, pub ip_address: IpAddress, pub user_agent: UserAgent, pub outcome: LoginOutcome, pub failure_reason: Option<LoginFailureReason>, }` (no `academic_id`). `crates/cross-cutting/operations/src/commands.rs:307-316` includes `pub academic_id: Option<AcademicYearRef>,` (line 311).

### FINDING 17

- **id:** CROSSCUT-OPS-017
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/commands.rs:99-105` (`ScheduleJobCommand::available_at`)
- **description:** `ScheduleJobCommand::available_at` is typed as `Timestamp` (the raw `educore_core::value_objects::Timestamp`); the spec at `docs/specs/operations/commands.md:81` declares the field as `JobAvailableAt` (a typed wrapper). The same drift applies to every other command field that the spec wraps in a typed value object (e.g. `JobQueue` in `ScheduleJobCommand` matches; `JobPayload` in `ScheduleJobCommand` matches; `FailedJobException` in `RecordFailedJobCommand` and `MarkJobFailedCommand` matches - these are OK; the drift is on the timestamp).
- **expected:** `pub available_at: crate::value_objects::JobAvailableAt` (which is already declared as `pub use Timestamp as JobAvailableAt;` at `value_objects.rs:1037`).
- **evidence:** `docs/specs/operations/commands.md:78-84` `pub struct ScheduleJobCommand { ..., pub available_at: JobAvailableAt, }`. `crates/cross-cutting/operations/src/commands.rs:99-105` `pub available_at: Timestamp,`.

### FINDING 18

- **id:** CROSSCUT-OPS-018
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/value_objects.rs:457-489` (`SidebarIgnoreFlag`)
- **description:** The spec at `docs/specs/operations/value-objects.md:105` declares the type as `SidebarIgnore` (no `Flag` suffix); the code at `value_objects.rs:457` names it `SidebarIgnoreFlag`. The same prefix drift appears in the `repositories.md` "Indexes" section which references the column as `rbac_sidebars.ignore` (consistent with both names) - but the type name in the public API is non-conformant.
- **expected:** `pub struct SidebarIgnore(pub i32);` matching the spec.
- **evidence:** `docs/specs/operations/value-objects.md:105` `| SidebarIgnore | i32 (0=Show, 1=Hide, 2=Disabled) |`. `crates/cross-cutting/operations/src/value_objects.rs:457` `pub struct SidebarIgnoreFlag(pub i32);` and re-exported in `lib.rs:66` as `pub use crate::value_objects::..., SidebarIgnoreFlag, ...`.

### FINDING 19

- **id:** CROSSCUT-OPS-019
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/commands.rs:345-352` (`ConfigureMaintenanceCommand`)
- **description:** All four fields of `ConfigureMaintenanceCommand` are typed as `Option<...>` in the code; the spec at `docs/specs/operations/commands.md:244-252` declares them as required (no `Option`). The current type makes the command unable to express the spec's "creates or updates the school's `MaintenanceSetting`" effect with required fields, and the implementation needs an additional `reconfigure`-style command to express the partial-update flow.
- **expected:** The spec's signature (title, sub_title, image, applicable_for all required) - or, if the partial-update use case is intended, a separate `ReconfigureMaintenanceCommand` (which already exists as `MaintenanceSetting::reconfigure` on the aggregate at `aggregate.rs:607-632`).
- **evidence:** `docs/specs/operations/commands.md:244-252` `pub struct ConfigureMaintenanceCommand { pub tenant: TenantContext, pub title: MaintenanceTitle, pub sub_title: MaintenanceSubTitle, pub image: Option<MaintenanceImage>, pub applicable_for: MaintenanceApplicableFor, }` (note: `image` is `Option`, but `title`, `sub_title`, `applicable_for` are required). `crates/cross-cutting/operations/src/commands.rs:345-352` `pub struct ConfigureMaintenanceCommand { pub tenant: TenantContext, pub title: Option<MaintenanceTitle>, pub sub_title: Option<MaintenanceSubTitle>, pub image: Option<MaintenanceImage>, pub applicable_for: Option<MaintenanceApplicableFor>, }` (all four are `Option`).

### FINDING 20

- **id:** CROSSCUT-OPS-020
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:607-632` (`MaintenanceSetting::reconfigure`)
- **description:** `MaintenanceSetting::reconfigure` takes `image: Option<Option<MaintenanceImage>>` (a double-`Option` to distinguish "leave unchanged" from "clear the image"); the spec at `docs/specs/operations/commands.md:243-256` defines `ConfigureMaintenanceCommand::image: Option<MaintenanceImage>` (single `Option`). The double-`Option` is not driven by any command shape in the code (the command uses single `Option`) - the aggregate API is not reachable from the command as written.
- **expected:** A matching single-`Option` API, or a separate clear-image command.
- **evidence:** `crates/cross-cutting/operations/src/aggregate.rs:607-632` `pub fn reconfigure(&mut self, title: Option<MaintenanceTitle>, sub_title: Option<MaintenanceSubTitle>, image: Option<Option<MaintenanceImage>>, applicable_for: Option<MaintenanceApplicableFor>, ...)`. `crates/cross-cutting/operations/src/commands.rs:345-352` `pub struct ConfigureMaintenanceCommand { pub image: Option<MaintenanceImage>, ... }` (single `Option`).

### FINDING 21

- **id:** CROSSCUT-OPS-021
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:172` (`Job::new`)
- **description:** `Job::new` validates that the payload is non-empty by inspecting the `JobPayload` wrapper's inner string (line 176-180), but the spec at `docs/specs/operations/aggregates.md:65` says "A `Job::payload` is a serialized command envelope" - the spec means a JSON-encoded command, not just a non-empty string. The current `JobPayload::new` (at `value_objects.rs:996-1012`) accepts any non-empty 1..65000-char string, so a plain `"hello"` would pass validation even though it's not a valid command envelope.
- **expected:** `JobPayload::new` calls (or is paired with) a JSON-deserialization step that verifies the payload matches a `CommandEnvelope` schema.
- **evidence:** `docs/specs/operations/aggregates.md:62-66` "Invariants: 1. A Job::queue is a non-empty string. 2. A Job::payload is a serialized command envelope." `crates/cross-cutting/operations/src/value_objects.rs:996-1012` `pub fn new(s: impl Into<String>) -> Result<Self> { if s.is_empty() || s.len() > 65000 { return Err(...); } Ok(Self(s)) }` - no JSON shape check.

### FINDING 22

- **id:** CROSSCUT-OPS-022
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:439-478` (`VersionHistory`)
- **description:** The `VersionHistory` struct has a field named `version_` (trailing underscore) to avoid colliding with the `version` field. The trailing-underscore is the engine's documented anti-pattern ("No `#[allow(dead_code)]` or `_var` prefixes to silence the compiler" in `AGENTS.md` and `docs/code-standards.md`). The field is the audit version counter (an Engine implementation detail) and is not declared in the spec; the public field is misnamed.
- **expected:** The aggregate field is renamed (e.g. to `audit_version` or the audit counter is stored as a private field).
- **evidence:** `crates/cross-cutting/operations/src/aggregate.rs:445-446` `pub version_: Version,`. `AGENTS.md` section "Type Safety" prohibits `_var` prefixes.

### FINDING 23

- **id:** CROSSCUT-OPS-023
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:439-478` (`VersionHistory`)
- **description:** `VersionHistory::new` does not initialize the `updated_at` / `updated_by` / `etag` fields; the struct does not have those fields at all, so the append-only invariant is enforced at the type level (no `update` method), but the audit-trail fields are missing. The spec at `docs/specs/operations/aggregates.md:166-174` lists 5 invariants; the implementation does not model `updated_at` even though every other aggregate has it.
- **expected:** A consistent audit-field set across all 8 root aggregates, with `VersionHistory` either explicitly append-only (no `updated_at`) and the field set documented, or carrying the same audit fields as the others.
- **evidence:** `docs/specs/operations/aggregates.md:165-174` "Invariants: 1. A VersionHistory::version is non-empty. ... 5. VersionHistory rows are append-only." (no mention of `updated_at`). `crates/cross-cutting/operations/src/aggregate.rs:445-451` has `id`, `version`, `release_date`, `url`, `notes`, `version_`, `etag`, `created_at`, `created_by`, `last_event_id`, `correlation_id` (no `updated_at` / `updated_by`).

### FINDING 24

- **id:** CROSSCUT-OPS-024
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `services.rs:362-372` (`MaintenanceService::applies_to_role`)
- **description:** `Backup::new` validates `if !cmd.active_status && cmd.restore_in_progress` (line 76-80) but `RestoreBackup` is implemented as a flag flip on the aggregate (`mark_restoring` at `aggregate.rs:102-107`) and not as a port-driven storage operation. The spec at `docs/specs/operations/aggregates.md:23-25` and `commands.md:45-60` says `RestoreBackup` "triggers the restore through the storage port" and "After restore, the platform domain invalidates its in-memory caches." Neither the storage port invocation nor the platform cache invalidation is implemented.
- **expected:** A `RestoreBackupService::execute(backup, storage_port, platform_cache)` that calls the storage port, emits `BackupRestored`, and notifies the platform subscriber.
- **evidence:** `docs/specs/operations/aggregates.md:25` "A Backup cannot be hard-deleted while a restore is in progress." `docs/specs/operations/commands.md:45-60` `RestoreBackupCommand` effects: "Triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` only flip the flag - no storage port call, no platform subscriber.

### FINDING 25

- **id:** CROSSCUT-OPS-025
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:141-265` (`Job`)
- **description:** `Job::fail` (line 241-251) sets the job's status to `Failed` and updates `last_event_id`, but it does not create a `FailedJob` row. The spec at `docs/specs/operations/events.md:135-137` says "`FailedJob` is created from this event by the operations subscriber" (referring to `JobFailed`); the spec at `docs/specs/operations/workflows.md:42-46` says step 6-7: "On failure, the runner increments attempts; if the retry budget is exhausted, the runner issues MarkJobFailedCommand... The operations domain records a FailedJob row and emits FailedJobRecorded." There is no `JobFailedSubscriber` (or equivalent) in the operations crate.
- **expected:** A subscriber that observes `JobFailed` and creates a `FailedJob` row (emitting `FailedJobRecorded`).
- **evidence:** `docs/specs/operations/events.md:135-137` **Subscribers:** `FailedJob is created from this event by the operations subscriber`. `docs/specs/operations/workflows.md:42-46` step 6-7. `crates/cross-cutting/operations/src/aggregate.rs:241-251` `Job::fail` only mutates the job. `crates/cross-cutting/operations/src/` has no `subscriber.rs`, no `on_job_failed` handler, no `RecordFailedJob` aggregator.

### FINDING 26

- **id:** CROSSCUT-OPS-026
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/services.rs` (entire) and `crates/cross-cutting/operations/src/lib.rs:26-34` (module surface)
- **description:** The 9 service structs and the 3 policies / 4 specifications are pure helper modules - there is no command-handler / dispatcher module in the operations crate. The spec at `docs/specs/operations/overview.md:111-123` says events drive cross-domain flows; `docs/specs/operations/permissions.md:91-97` says capabilities are checked at the command boundary. Neither a command handler, a capability check, nor an event subscriber is implemented. All 24 commands are wire-form-only data structs.
- **expected:** A `handlers.rs` (or per-aggregate `*_service.rs`) module that wires each command to its aggregate method, runs the capability check, emits the events, and writes the audit / outbox / idempotency rows.
- **evidence:** `docs/specs/operations/commands.md` (24 commands listed); `docs/specs/operations/permissions.md:91-97` capability check pattern. `crates/cross-cutting/operations/src/lib.rs:26-34` declares `aggregate, commands, entities, errors, events, query, repository, services, value_objects` - no `handler` or `dispatch` module.

### FINDING 27

- **id:** CROSSCUT-OPS-027
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/commands.rs:439-444` (`UpdateSidebarEntryCommand`) and `aggregate.rs:659-781` (`Sidebar`)
- **description:** `UpdateSidebarEntryCommand` carries `position`, `level`, `ignore`, `active_status` - but the `Sidebar` aggregate has no `update` method that takes these four fields together, no `set_position`, no `set_level`. The two existing setters (`set_ignore` at `aggregate.rs:752-762`, `set_active` at `aggregate.rs:766-777`) handle two of the four fields individually. A command handler that wires `UpdateSidebarEntryCommand` to the aggregate has no method to call.
- **expected:** A `Sidebar::update(&mut self, position, level, ignore, active_status, actor, at, event_id)` method (or a handler that calls the four setters in sequence).
- **evidence:** `docs/specs/operations/commands.md:436-444` `pub struct UpdateSidebarEntryCommand { pub tenant: TenantContext, pub sidebar_id: SidebarId, pub position: Option<SidebarPosition>, pub level: Option<SidebarLevel>, pub ignore: Option<SidebarIgnoreFlag>, pub active_status: Option<SidebarActiveStatus>, }`. `crates/cross-cutting/operations/src/aggregate.rs:703-777` impl block lists only `new`, `is_system`, `reorder`, `set_ignore`, `set_active`.

### FINDING 28

- **id:** CROSSCUT-OPS-028
- **area:** cross-cutting
- **severity:** High
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:175-199` (`Job::new`) and `services.rs:113-114` (`JobService::can_retry`)
- **description:** The spec at `docs/specs/operations/aggregates.md:67-70` defines invariant 4 (`A Job::available_at is a Unix timestamp; the job is not runnable before this time.`) and invariant 5 (`A Job::reserved_at is a Unix timestamp; if set, the job is currently being processed by a worker.`). `Job::new` does not validate that `available_at` is not in the past (which would be a worker-port concern, not a domain invariant), but the more important issue is that `JobService::can_retry` at `services.rs:112-114` is `pub fn can_retry(job: &Job, max_attempts: u8) -> bool { job.attempts.0 < max_attempts }` and never inspects the job's `available_at` or `reserved_at` - a job that has been reserved but is "available" again would be flagged as retryable.
- **expected:** `can_retry` also checks `job.status == Pending` and `job.available_at <= now`.
- **evidence:** `docs/specs/operations/aggregates.md:64-70` invariants 3-5. `crates/cross-cutting/operations/src/services.rs:112-114` `pub fn can_retry(job: &Job, max_attempts: u8) -> bool { job.attempts.0 < max_attempts }`.

### FINDING 29

- **id:** CROSSCUT-OPS-029
- **area:** cross-cutting
- **severity:** Critical
- **location:** `crates/cross-cutting/operations/src/query.rs:15-153` (8 query stubs)
- **description:** All 8 query stubs in `query.rs` are empty structs with only a `Default` impl and a `new()` constructor; none of them uses the `#[derive(DomainQuery)]` macro, none of them references the 15 tables in `docs/specs/operations/tables.md`, and none of them has a field, a `where_has` clause, or a `with` clause. The tables listed in `tables.md` are: `failed_jobs`, `jobs`, `operations_maintenance_settings`, `migrations`, `oauth_access_tokens`, `oauth_auth_codes`, `oauth_clients`, `oauth_personal_access_clients`, `oauth_refresh_tokens`, `password_resets`, `rbac_sidebars`, `operations_backups`, `operations_system_versions`, `operations_user_logs`, `operations_version_histories` (15 rows) - none is emitted by the `DomainQuery` macro from this crate. The crate does not depend on `educore-query-derive`.
- **expected:** At least the 5 owned-aggregate tables (`operations_backups`, `jobs`, `failed_jobs`, `operations_system_versions`, `operations_version_histories`, `operations_user_logs`, `operations_maintenance_settings`, `rbac_sidebars`) have `#[derive(DomainQuery)]` structs with typed fields and a `with`/`.active()` style API.
- **evidence:** `docs/specs/operations/tables.md:7-23` (15 table rows). `crates/cross-cutting/operations/src/query.rs:15-153` (8 empty struct stubs). `crates/cross-cutting/operations/Cargo.toml:13-27` does not include `educore-query-derive`. `crates/cross-cutting/operations/src/` has zero `#[derive(DomainQuery)]` attributes (grep returns no matches).

### FINDING 30

- **id:** CROSSCUT-OPS-030
- **area:** cross-cutting
- **severity:** Critical
- **location:** `crates/cross-cutting/operations/tests/` (does not exist)
- **description:** There is no `crates/cross-cutting/operations/tests/` directory; the operations crate has zero integration tests in the conventional cargo location. The 47 unit tests in the source files pass, but the engine's validation rule ("At least one integration test per PR") and the "9-file layout" template require a `tests/` directory for the crate. The only operations integration tests are in `crates/tools/storage-parity/tests/operations_integration.rs` (which is in a different crate).
- **expected:** A `crates/cross-cutting/operations/tests/` directory with at least one integration test file (e.g. `tests/integration.rs`) covering command to aggregate to event flow.
- **evidence:** `find /home/beznet/Workspace/smscore/crates/cross-cutting/operations -type d` returns only `crates/cross-cutting/operations` and `crates/cross-cutting/operations/src` (no `tests/`). `crates/tools/storage-parity/tests/operations_integration.rs:1-194` is the only operations-integration test file in the workspace.

### FINDING 31

- **id:** CROSSCUT-OPS-031
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:39-99` (`Backup`)
- **description:** `Backup::new` returns the actor's id as both `created_by` and `updated_by` (line 94-95), and there is no `delete` method. The spec at `docs/specs/operations/aggregates.md:5-25` and `commands.md:32-43` defines a `DeleteBackupCommand` that emits `BackupDeleted`; the aggregate's `updated_by` field is never set to the actor who deletes the row because there is no `delete` method. A consumer attempting to call `repository.delete(backup_id)` cannot populate the audit trail from the aggregate.
- **expected:** A `Backup::delete(actor, at, event_id)` method that sets `active_status = false`, `updated_by = actor`, and returns the row state.
- **evidence:** `docs/specs/operations/commands.md:33-43` `DeleteBackupCommand` effects: "Deletes the Backup row and the underlying file, emits BackupDeleted." `crates/cross-cutting/operations/src/aggregate.rs:73-132` impl block: `new`, `mark_restoring`, `clear_restoring`, `mark_active`, `mark_inactive` only.

### FINDING 32

- **id:** CROSSCUT-OPS-032
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:312-332` (`FailedJob`)
- **description:** `FailedJob` has only `new` and no `delete` (which the spec at `docs/specs/operations/aggregates.md:115` declares as `DeleteFailedJob`). A failed-job retention sweep that hard-deletes rows cannot be modelled - the repository's `delete` is the only way to remove rows, bypassing any audit-trail update.
- **expected:** A `FailedJob::delete(actor, at, event_id)` method (or a port-driven purge that the service layer calls).
- **evidence:** `docs/specs/operations/aggregates.md:113-116` lists `RecordFailedJob`, `RetryFailedJob`, `DeleteFailedJob` commands. `crates/cross-cutting/operations/src/aggregate.rs:310-332` impl block: only `new` (no `delete` or `retry` method either).

### FINDING 33

- **id:** CROSSCUT-OPS-033
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:550-649` (`MaintenanceSetting`)
- **description:** `MaintenanceSetting` has no `delete` (or `soft_delete`) method. The spec at `docs/specs/operations/aggregates.md:241-251` implies the singleton can be replaced via `ConfigureMaintenanceCommand` but doesn't model a hard delete; however, the per-school `MaintenanceSettingId` is a typed `Id<MaintenanceSetting>` and the repository at `repository.rs:244-251` has only `get`, `insert`, `update` - no `delete`. The missing aggregate `delete` is paired with a missing repository `delete`, leaving the per-school singleton effectively un-removable.
- **expected:** A repository `delete(&self, school: SchoolId)` and an aggregate `MaintenanceSetting::soft_delete(actor, at, event_id)` (or a documented "singleton cannot be deleted" invariant).
- **evidence:** `docs/specs/operations/aggregates.md:241-251` "ConfigureMaintenance, EnableMaintenance, DisableMaintenance" (no `DeleteMaintenance` listed). `crates/cross-cutting/operations/src/aggregate.rs:583-649` `MaintenanceSetting` impl block: `configure`, `reconfigure`, `enable`, `disable` only. `crates/cross-cutting/operations/src/repository.rs:243-251` `MaintenanceSettingRepository` trait: `get`, `insert`, `update` (no `delete`).

### FINDING 34

- **id:** CROSSCUT-OPS-034
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`)
- **description:** `Backup` has no `restore_completed` method (only `mark_restoring` and `clear_restoring` at `aggregate.rs:102-115`). The spec at `docs/specs/operations/aggregates.md:23-25` and `commands.md:45-60` says `RestoreBackup` "triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." The current `clear_restoring` only flips the boolean; it does not emit `BackupRestored` (the spec event for restore is `BackupRestored`, not `BackupMarkedInactive`).
- **expected:** A `Backup::restore_complete(actor, at, event_id)` method that sets `restore_in_progress = false`, populates audit fields, and (per the dispatcher) emits `BackupRestored`.
- **evidence:** `docs/specs/operations/commands.md:55-60` "Effects: Triggers the restore through the storage port, emits BackupRestored. After restore, the platform domain invalidates its in-memory caches." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` - no event emission, no `BackupRestored` mapping.

### FINDING 35

- **id:** CROSSCUT-OPS-035
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:399-427` (`SystemVersion::update`)
- **description:** `SystemVersion::update` only accepts `Option<VersionTitle>` and `Option<VersionFeatures>` and silently ignores the case where both are `None` (the body just falls through). The spec at `docs/specs/operations/commands.md:188-200` defines `UpdateSystemVersionCommand` with both `title` and `features` as `Option`, but the expected effect is "Emits `SystemVersionUpdated`" - a no-op update would still emit the event.
- **expected:** Either reject the call when both fields are `None` (returning an `Err`), or always emit `SystemVersionUpdated` even when both are `None`.
- **evidence:** `docs/specs/operations/commands.md:198-200` "Effects: Emits SystemVersionUpdated." `crates/cross-cutting/operations/src/aggregate.rs:399-427` `pub fn update(&mut self, title: Option<VersionTitle>, features: Option<VersionFeatures>, actor: UserId, at: Timestamp, event_id: EventId) -> AggregateResult<()>` - body does nothing when both are `None`, and the return value is unused by the dispatcher (which would always emit the event).

### FINDING 36

- **id:** CROSSCUT-OPS-036
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `commands.rs:31-37` (`CreateBackupCommand`)
- **description:** `CreateBackupCommand` does not carry a `restore_in_progress` field (per spec at `docs/specs/operations/commands.md:17-24` - the spec command is `{ tenant, file_name, source_link, file_type, lang_type }` with no `restore_in_progress`); but `Backup::new` requires a `restore_in_progress: bool` field on `NewBackup` (line 67), defaulting it to `false` in the wire flow. The spec command struct cannot construct a `NewBackup` because the `restore_in_progress` field is not provided by the wire form.
- **expected:** The `Backup::new` API should derive `restore_in_progress` from aggregate state (it is always `false` at creation) and the `NewBackup` struct should not require it.
- **evidence:** `docs/specs/operations/commands.md:17-24` `CreateBackupCommand` (5 fields, no `restore_in_progress`). `crates/cross-cutting/operations/src/commands.rs:31-37` (matches the spec). `crates/cross-cutting/operations/src/aggregate.rs:60-71` `pub struct NewBackup { ..., pub restore_in_progress: bool, ... }` - the 6th field, not in the command, must be supplied by the dispatcher.

### FINDING 37

- **id:** CROSSCUT-OPS-037
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/services.rs:140-148` (`JobService::purge_completed`)
- **description:** `JobService::purge_completed` partitions a `Vec<Job>` into `Completed` and `pending`, but the "pending" name is misleading because it also includes `Reserved` and `Failed`. A reader of the function name "purge_completed" would expect a `Failed` purge option, and the returned vec semantics are conflated.
- **expected:** Rename the `pending` vec to `kept` and document the `Reserved + Failed` inclusion, or add a `purge_failed` companion.
- **evidence:** `docs/specs/operations/services.md:32` `pub fn purge_completed(jobs: &mut Vec<Job>) -> Vec<Job> { ... }`. `crates/cross-cutting/operations/src/services.rs:140-148` `let (done, pending): (Vec<Job>, Vec<Job>) = jobs.drain(..).partition(|j| matches!(j.status, JobStatus::Completed)); *jobs = pending; done`.

### FINDING 38

- **id:** CROSSCUT-OPS-038
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/repository.rs:244-251` (`MaintenanceSettingRepository`)
- **description:** The `MaintenanceSettingRepository` trait has no `delete` method, so the per-school singleton cannot be removed. The spec at `docs/specs/operations/repositories.md:99-108` lists `get`, `insert`, `update` only - but the spec at `docs/specs/operations/aggregates.md:233` says "A `MaintenanceSetting` exists at most once per `SchoolId`" and the spec at `commands.md:241-281` does not define a `DeleteMaintenance` command, so the missing repository `delete` is consistent with the spec. The aggregate also has no `delete` method, so the two are in sync. This is a documentation gap rather than a bug - but the audit's "Missing repositories" check expects `delete` to exist; the omission is a deliberate spec choice that is not documented as such.
- **expected:** Either add the repository `delete` (and the matching aggregate method), or document the "singleton is permanently addressable" invariant in `aggregates.md`.
- **evidence:** `docs/specs/operations/repositories.md:99-108` lists 3 methods (`get`, `insert`, `update`) - no `delete`. `crates/cross-cutting/operations/src/repository.rs:243-251` matches the spec. The spec at `docs/specs/operations/aggregates.md:231-251` is silent on whether the singleton can be deleted.

### FINDING 39

- **id:** CROSSCUT-OPS-039
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/repository.rs:108-128` (`FailedJobRepository`) and `docs/specs/operations/repositories.md:41-55`
- **description:** The spec at `docs/specs/operations/repositories.md:46-54` requires `get_by_uuid(&self, uuid: &FailedJobUuid) -> Result<Option<FailedJob>>`. The code at `repository.rs:108-128` defines the trait, but the aggregate `FailedJob` does not have a `uuid` field uniqueness validator (e.g. checking that the `uuid` is not nil). The repository can return a duplicate `FailedJob` for the same business uuid; the trait's `get_by_uuid` has no DB-level constraint backing it (since no DDL is emitted from the operations crate).
- **expected:** A unique constraint on `failed_jobs.uuid` (per spec at `tables.md:46` and `repositories.md:183` `CREATE UNIQUE INDEX ux_failed_jobs_uuid`).
- **evidence:** `docs/specs/operations/tables.md:46-48` "`failed_jobs.uuid` is a unique business identifier separate from the auto-increment id." `docs/specs/operations/repositories.md:183` `CREATE UNIQUE INDEX ux_failed_jobs_uuid ON failed_jobs (uuid);`. `crates/cross-cutting/operations/src/aggregate.rs:310-332` `FailedJob` has no constructor-time validation of the `uuid` (e.g. `uuid != Uuid::nil()`).

### FINDING 40

- **id:** CROSSCUT-OPS-040
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup`) and `docs/specs/operations/aggregates.md:17-25`
- **description:** The spec says `Backup::file_name` is unique within `(school_id, file_name)`; the code's `Backup::new` does not check uniqueness (the repository is expected to enforce it, but the spec says the aggregate is loaded, validated, and persisted in a single transaction). Without a uniqueness check in the aggregate, two `CreateBackupCommand` calls in the same transaction with the same `(school_id, file_name)` would both succeed at the aggregate level and fail at the database.
- **expected:** A uniqueness check in `Backup::new` (or a clear contract that the repository enforces it).
- **evidence:** `docs/specs/operations/aggregates.md:17-25` invariant 2: "A Backup::file_name is non-empty and unique within (school_id, file_name)." `docs/specs/operations/aggregates.md:43-47` "Consistency Boundary: A Backup is loaded by id, mutated in memory, validated, and persisted with its events in a single transaction." `crates/cross-cutting/operations/src/aggregate.rs:75-99` `Backup::new` does not check uniqueness.

### FINDING 41

- **id:** CROSSCUT-OPS-041
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/events.rs:75-86` (`BackupCreated` `aggregate_id`)
- **description:** `BackupCreated::aggregate_id` returns `self.backup_id.as_uuid()` - the local UUID without the school id. A consumer reconstructing the aggregate's `BackupId` from the event cannot recover the school id. The `BackupId` is tenant-scoped (`school_id: SchoolId, value: Uuid`), so `as_uuid()` strips the tenant.
- **expected:** The event includes `school_id` (it does - line 32 `pub school_id: SchoolId`) and consumers should reconstruct the typed id from `(school_id, value)`; the spec at `events.md:39-46` lists `BackupCreated { pub backup_id: BackupId, pub file_name: ..., pub file_type: ..., pub created_at: ... }` - the typed `BackupId` carries the school, so the spec is consistent, but the code's `as_uuid()` return is lossy.
- **evidence:** `docs/specs/operations/events.md:39-46` `pub struct BackupCreated { pub backup_id: BackupId, pub file_name: BackupFileName, pub file_type: BackupFileType, pub created_at: Timestamp, }`. `crates/cross-cutting/operations/src/events.rs:77-79` `fn aggregate_id(&self) -> Uuid { self.backup_id.as_uuid() }`. `crates/cross-cutting/operations/src/value_objects.rs:40-52` `BackupId::as_uuid` returns only the `value: Uuid`, dropping the school id.

### FINDING 42

- **id:** CROSSCUT-OPS-042
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:39-132` (`Backup`) and `aggregate.rs:102-115` (`mark_restoring` / `clear_restoring`)
- **description:** `Backup` has `restore_in_progress: bool` (a typed raw bool, see Finding 4) and a `mark_restoring` / `clear_restoring` pair that flips it. The spec at `docs/specs/operations/aggregates.md:43-47` says "Concurrent `RestoreBackup` commands on the same backup are serialized." There is no concurrency control in the aggregate; two `RestoreBackupCommand` calls in different transactions can both pass the aggregate check and both call `mark_restoring` - the second one will fail only at the database level (if a unique index exists). The spec's "serialized" invariant has no enforcement point.
- **expected:** A row-level lock or a serialized concurrency primitive on the `Backup` row (the spec at `docs/handoff/PHASE-14-HANDOFF.md:218-222` says "the dispatcher acquires the row-level lock" - but no dispatcher is implemented).
- **evidence:** `docs/specs/operations/aggregates.md:43-47` Consistency Boundary. `docs/handoff/PHASE-14-HANDOFF.md:218-222` "the dispatcher acquires the row-level lock on the relevant row." `crates/cross-cutting/operations/src/aggregate.rs:102-115` `mark_restoring` and `clear_restoring` - no concurrency guard.

### FINDING 43

- **id:** CROSSCUT-OPS-043
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `events.rs:28-86` (`BackupCreated`)
- **description:** `BackupCreated` event is emitted by the dispatcher (which doesn't exist - see Finding 26), but the event's `created_at` and `occurred_at` are independent timestamps (line 35 `pub created_at: Timestamp` and line 38 `pub occurred_at: Timestamp`). The spec at `docs/specs/operations/events.md:39-46` lists only `created_at`; the code adds `occurred_at` (per the `DomainEvent` trait). A consumer that filters by `occurred_at` will see a different value than a consumer that filters by `created_at` - the two are never reconciled.
- **expected:** A documented contract that `created_at == occurred_at` for `BackupCreated`, or a single `occurred_at` field.
- **evidence:** `docs/specs/operations/events.md:39-46` lists only `created_at`. `crates/cross-cutting/operations/src/events.rs:30-40` has both `pub created_at: Timestamp,` (line 35) and `pub occurred_at: Timestamp,` (line 38).

### FINDING 44

- **id:** CROSSCUT-OPS-044
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`) and `value_objects.rs:495-514` (`BackupFileName`)
- **description:** `BackupFileName::new` validates 1..255 chars but does not validate that the file name is a valid filename (e.g. no slashes, no nulls, no `..` path traversal). The spec at `docs/specs/operations/aggregates.md:18-19` says "A `Backup::file_name` is non-empty and unique within (school_id, file_name)" - silent on character set, but the spec at `value-objects.md:31` says "1..255 chars, unique within (school_id, file_name)". A `file_name` of `"../../etc/passwd"` would pass the current validator and could be passed to the file-storage port as a path.
- **expected:** A filename-shape validator (no path separators, no `..`, no null bytes).
- **evidence:** `crates/cross-cutting/operations/src/value_objects.rs:499-514` `pub fn new(s: &str) -> Result<Self> { if s.is_empty() || s.len() > 255 { return Err(...); } Ok(Self(s.to_owned())) }` - no character-class check. `docs/specs/operations/aggregates.md:18-19` and `value-objects.md:31` silent on character set.

### FINDING 45

- **id:** CROSSCUT-OPS-045
- **area:** cross-cutting
- **severity:** Medium
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:490-540` (`UserLog`) and `events.rs:973-1036` (`UserLogged`)
- **description:** `UserLog` is declared append-only in the spec (invariant 8) and the code has no `update` or `delete` method on the aggregate - good. But the spec at `docs/specs/operations/workflows.md:62-73` says "A nightly job (port) partitions the log by month" and "purges UserLog rows older than the school's retention policy (default 365 days). The purge is logged as a `DeleteUserLog` event for compliance." The repository's `UserLogRepository::purge_older_than` (at `repository.rs:229`) hard-deletes rows without going through the aggregate (which has no `delete` method), and no `UserLogDeleted` event is emitted (Finding 14).
- **expected:** A `UserLogService::purge_with_audit(rows, actor, at)` that hard-deletes each row and emits a `UserLogDeleted` event (which is missing - see Finding 14).
- **evidence:** `docs/specs/operations/workflows.md:62-73` "User Log Retention Workflow" steps 4-6. `crates/cross-cutting/operations/src/repository.rs:228-229` `async fn purge_older_than(&self, school: SchoolId, cutoff: Timestamp) -> StorageResult<u64>;`. `crates/cross-cutting/operations/src/aggregate.rs:509-540` `UserLog` impl has no `delete` or `purge` method.

### FINDING 46

- **id:** CROSSCUT-OPS-046
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/commands.rs:288-297` (`RecordVersionHistoryCommand::into_input`)
- **description:** `RecordVersionHistoryCommand::into_input` (line 288-297) consumes `self` and returns a `VersionHistoryInput`. The `VersionHistory::new` constructor (at `aggregate.rs:457-479`) takes `VersionHistoryInput` as its second argument. The connection from command to aggregate is wired for this one command, but the `VersionHistory` aggregate has no `id` field on `VersionHistoryInput` - the dispatcher must supply a `VersionHistoryId` from outside. Per the spec at `docs/specs/operations/commands.md:204-219` the command does not carry the id either. The convention should be documented or the command should carry an optional `Option<VersionHistoryId>` for the upsert case.
- **expected:** Either a documented "id is generated by the engine" contract in `commands.md` (the spec at `:212-219` is silent on the id), or an optional id field.
- **evidence:** `docs/specs/operations/commands.md:204-219` `RecordVersionHistoryCommand` has no id field. `crates/cross-cutting/operations/src/commands.rs:288-297` `into_input` returns a `VersionHistoryInput` (no id). `crates/cross-cutting/operations/src/aggregate.rs:457-479` `VersionHistory::new` takes `id: VersionHistoryId` as a separate parameter (line 459).

### FINDING 47

- **id:** CROSSCUT-OPS-047
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:312-332` (`FailedJob::new`)
- **description:** `FailedJob::new` takes `original_job_id: JobId` and `queue: FailedJobQueue` (which is a separate type from `JobQueue`, per `value_objects.rs:929-948` `FailedJobQueue(pub String)` vs `value_objects.rs:972-990` `JobQueue(pub String)`). Both wrap a `String` and both validate 1..191 chars, but the type split requires the dispatcher to construct two distinct types from the same source data. The spec at `docs/specs/operations/value-objects.md:55` says `FailedJobQueue | 1..191 chars` and at `:41` `JobQueue | 1..191 chars (e.g. default, emails, webhooks)` - the spec uses different type names, so the type split is correct per the spec, but the duplicate validation logic is an anti-pattern.
- **expected:** A shared `QueueName` newtype that both `JobQueue` and `FailedJobQueue` alias to.
- **evidence:** `docs/specs/operations/value-objects.md:41` `JobQueue | 1..191 chars` and `:55` `FailedJobQueue | 1..191 chars`. `crates/cross-cutting/operations/src/value_objects.rs:972-990` `JobQueue` and `crates/cross-cutting/operations/src/value_objects.rs:929-948` `FailedJobQueue` have duplicate 1..191-char validators (lines 935-937 vs 978-980).

### FINDING 48

- **id:** CROSSCUT-OPS-048
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/value_objects.rs:1037` (`JobAvailableAt` alias)
- **description:** `pub use educore_core::value_objects::Timestamp as JobAvailableAt;` (line 1037) is the only `Job*Timestamp` typed wrapper; the spec at `docs/specs/operations/value-objects.md:44-46` declares three (`JobReservedAt`, `JobAvailableAt`, `JobCreatedAt`). The other two are missing (see Finding 5). The single `JobAvailableAt` alias is correctly emitted but it is not used in `commands.rs` (see Finding 17).
- **expected:** Either remove the unused alias (if `Timestamp` is the canonical form) or use it in `ScheduleJobCommand` and elsewhere.
- **evidence:** `crates/cross-cutting/operations/src/value_objects.rs:1037` `pub use educore_core::value_objects::Timestamp as JobAvailableAt;`. `crates/cross-cutting/operations/src/commands.rs:99-105` `pub available_at: Timestamp,` (uses raw `Timestamp`, not the alias).

### FINDING 49

- **id:** CROSSCUT-OPS-049
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/value_objects.rs:1097-1108` (`MaintenanceImage`)
- **description:** `MaintenanceImage::new(name: impl Into<String>) -> Self` (line 1101-1107) bypasses the `FileReference::new` validator - the docstring on line 1105-1106 says "No validation here: callers may pre-validate via `FileReference::new`." The spec at `docs/specs/operations/value-objects.md:92` says `MaintenanceImage | FileReference?` (a `FileReference` optional); a direct `MaintenanceImage::new("")` succeeds.
- **expected:** `MaintenanceImage::new` calls `FileReference::new` and propagates the error.
- **evidence:** `docs/specs/operations/value-objects.md:92` `| MaintenanceImage | FileReference? |`. `crates/cross-cutting/operations/src/value_objects.rs:1097-1108` `pub fn new(name: impl Into<String>) -> Self { Self(FileReference(name.into())) }` - no validation, comment on line 1105-1106 admits the gap.

### FINDING 50

- **id:** CROSSCUT-OPS-050
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:75-99` (`Backup::new`)
- **description:** `Backup::new` rejects the combination `active_status=false && restore_in_progress=true` (line 76-80) with `Validation`. The spec at `docs/specs/operations/aggregates.md:23` says "A `Backup::active_status` is a boolean" and `:25` says "A `Backup` cannot be hard-deleted while a restore is in progress." The combined invariant "an inactive backup cannot have a restore in progress" is not in the spec - it is a code-original rule.
- **expected:** Document the combined invariant in `aggregates.md` or remove the check.
- **evidence:** `docs/specs/operations/aggregates.md:15-25` (spec lists 6 invariants, none is the combined check). `crates/cross-cutting/operations/src/aggregate.rs:76-80` `if !cmd.active_status && cmd.restore_in_progress { return Err(OperationsDomainError::Validation("inactive backup cannot have restore in progress".to_owned())); }`.

### FINDING 51

- **id:** CROSSCUT-OPS-051
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/services.rs:239-246` (`SystemVersionService::is_compatible`)
- **description:** `SystemVersionService::is_compatible` returns true if `c.0 == s.0 && c.0 != 0` (line 224-228). The `c.0 != 0` rule is not in the spec at `docs/specs/operations/services.md:54-58` which says "Returns true if `client` is compatible with `server` (same major version)." The spec's intent is "same major version" - a major version of 0 is typically the pre-release / development version, and the spec is silent on whether 0.0.0 should be considered compatible.
- **expected:** Either document the 0.0.0 rule or remove the `c.0 != 0` short-circuit.
- **evidence:** `docs/specs/operations/services.md:54-58` `pub fn is_compatible(client: &VersionName, server: &VersionName) -> bool { ... }` (no 0.0.0 mention). `crates/cross-cutting/operations/src/services.rs:222-228` `c.0 == s.0 && c.0 != 0`.

### FINDING 52

- **id:** CROSSCUT-OPS-052
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/value_objects.rs:754-788` (`IpAddress`)
- **description:** `IpAddress::is_valid` accepts a leading-zero octet (e.g. `010.0.0.1`) as `is_valid` returns `true` only if the part starts with a non-zero digit OR is exactly "0" - the unit test at `value_objects.rs:1283` `assert!(!IpAddress::is_valid("192.0.02.1"))` checks for the case but the `is_valid_ipv4` function at `value_objects.rs:790-815` correctly rejects `part.starts_with('0') && part.len() > 1` (line 799-801). This is correct - but `IpAddress::new("010.0.0.1")` is rejected by the constructor, so the spec is consistent. The finding is null: the test at line 1283 already proves the rejection. (Documented for completeness; no defect.)
- **expected:** N/A - this is a verification of correct behavior, not a defect. The unit test at `value_objects.rs:1283` proves the rule.
- **evidence:** `crates/cross-cutting/operations/src/value_objects.rs:799-801` rejects leading-zero octets. `crates/cross-cutting/operations/src/value_objects.rs:1283` `assert!(!IpAddress::is_valid("192.0.02.1"));` (leading zero).

### FINDING 53

- **id:** CROSSCUT-OPS-053
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:457-479` (`VersionHistory::new`)
- **description:** `VersionHistory::new` is a `#[must_use]` constructor (line 457) that does not return a `Result`, even though `BackupFileName` and `HistoryVersion` (used in `VersionHistoryInput::new`) both return `Result` and can fail. The path is: command to `into_input` (returns `VersionHistoryInput`) to `VersionHistory::new` (no Result). The validation that occurred at command construction is the only validation; the aggregate's `new` accepts the validated input and cannot fail. The pattern is correct but the `#[must_use]` is over-broad - the aggregate's `new` doesn't have a side-effecting builder, so `#[must_use]` is appropriate.
- **expected:** Either the spec should be updated to clarify the validation flow, or the `#[must_use]` annotation is fine as-is.
- **evidence:** `crates/cross-cutting/operations/src/aggregate.rs:457-479` `#[must_use] pub fn new(id: VersionHistoryId, input: VersionHistoryInput, created_by: UserId, correlation_id: CorrelationId, at: Timestamp) -> Self` (no Result). `crates/cross-cutting/operations/src/entities.rs:893-908` `VersionHistoryInput::new` returns `Self` (not Result), and the underlying `HistoryVersion` / `HistoryReleaseDate` / `HistoryNotes` validators are at `value_objects.rs:660-752`.

### FINDING 54

- **id:** CROSSCUT-OPS-054
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:514-540` (`UserLog::new`)
- **description:** `UserLog::new` (line 514-540) does not validate that the academic year (`academic_id: Option<AcademicYearRef>`) is consistent with the user's school. A `UserLog` could carry an `AcademicYearRef` with a different `school_id` than the `UserLog::school_id` (which is `input.school_id`). The spec at `docs/specs/operations/aggregates.md:206-207` says "A `UserLog::academic_id` references a valid `AcademicYearId`" but is silent on tenant consistency.
- **expected:** A check that `academic_id.school_id == log.school_id` when `academic_id.is_some()`.
- **evidence:** `docs/specs/operations/aggregates.md:203-208` invariants 4-7. `crates/cross-cutting/operations/src/aggregate.rs:514-540` `UserLog::new` body does not compare `input.academic_id.map(|a| a.school_id)` against `input.school_id`.

### FINDING 55

- **id:** CROSSCUT-OPS-055
- **area:** cross-cutting
- **severity:** Low
- **location:** `crates/cross-cutting/operations/src/aggregate.rs:733-735` (`Sidebar::is_system`)
- **description:** `Sidebar::is_system` is the only consumer-side check of the `is_system_defined` invariant, but the spec at `docs/specs/operations/aggregates.md:270-273` says "the engine refuses to delete them" - implying a `delete` method that checks the flag. With no `delete` method (Finding 2), the `is_system` helper is unused by any aggregate method. The audit's "test for system-defined deletion" is not possible to write.
- **expected:** A `Sidebar::delete(actor, at, event_id)` method that returns `Err(OperationsDomainError::Forbidden("system-defined sidebar cannot be deleted"))` when `is_system_defined.0 == true`.
- **evidence:** `docs/specs/operations/aggregates.md:270-273` "`is_system_defined` flags system-defined rows; the engine refuses to delete them." `crates/cross-cutting/operations/src/aggregate.rs:733-735` `pub const fn is_system(&self) -> bool { self.is_system_defined.0 }` - no caller in `aggregate.rs:703-777`.

### FINDING 56

- **id:** CROSSCUT-OPS-056
- **area:** cross-cutting
- **severity:** Low
- **location:** `docs/handoff/PHASE-14-HANDOFF.md:36-40` and `docs/coverage.toml:2039-2128`
- **description:** The Phase 14 hand-off says "47 unit tests in `educore-operations`" and the coverage matrix has 10 operations rows flipped from `Pending` to `Tested` (8 aggregate rows + 1 capability row + 1 audit-target row), all marked `Tested`. The actual test count from `#[test]` attributes is 12+11+11+2+3+6+2 = 47 in source files (matches the hand-off), but no row in `docs/coverage.toml` references the `tests` directory because the `tests/` directory does not exist (Finding 30). The coverage matrix therefore claims `Tested` for all 8 root aggregates but each row's `tests` field only points at the `aggregate.rs` unit tests - there is no integration test coverage for any of the 8 aggregates.
- **expected:** A row per aggregate pointing to the integration test file in `crates/cross-cutting/operations/tests/` (which doesn't exist).
- **evidence:** `docs/coverage.toml:2038-2128` 10 operations rows, all `status = "Tested"`. `docs/handoff/PHASE-14-HANDOFF.md:53-55` "**47 passed, 0 failed**" (unit tests only). `crates/cross-cutting/operations/tests/` does not exist (Finding 30).

### END FINDINGS
Total Findings: 56
