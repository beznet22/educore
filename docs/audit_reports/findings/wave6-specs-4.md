# Spec Folder Audit Wave 6 — Specs Group 4

**Scope:** `docs/specs/operations/`, `docs/specs/platform/`, `docs/specs/rbac/`, `docs/specs/settings/`, `docs/specs/sync/`.

**11 spec files per folder (per `docs/code-standards.md` § "Spec folder layout"):**
`overview.md`, `aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`.

**File counts observed:**
- `operations/`: 11 files (complete)
- `platform/`: 11 files (complete)
- `rbac/`: 11 files (complete)
- `settings/`: 11 files (complete)
- `sync/`: 1 file (`overview.md` only) — **10 files missing**

---

### FINDING 1

- **id:** SPEC-4-001
- **area:** spec
- **severity:** Critical
- **location:** `docs/specs/sync/` (directory listing)
- **description:** The `docs/specs/sync/` spec folder contains only `overview.md` (1162 lines). The other 10 spec files (`aggregates.md`, `entities.md`, `value-objects.md`, `commands.md`, `events.md`, `services.md`, `permissions.md`, `repositories.md`, `workflows.md`, `tables.md`) are absent. Per `docs/code-standards.md` § "Spec folder layout" each spec folder must contain exactly 11 files.
- **expected:** All 11 spec files present in `docs/specs/sync/`, including dedicated `commands.md`, `events.md`, and `tables.md` files documenting the sync aggregates (`OutboxEntry`, `SyncCursor`, `ConflictRecord`, `SyncSubscription`).
- **evidence:** `docs/specs/sync/`: `total 52` containing `-rw-rw-r-- 1 beznet beznet 43150 Jun 13 09:32 overview.md` only; `ls /home/beznet/Workspace/smscore/docs/specs/sync/` returns `overview.md` as the sole file.

---

### FINDING 2

- **id:** SPEC-4-002
- **area:** spec
- **severity:** High
- **location:** `docs/specs/operations/commands.md:152,166` vs `docs/specs/operations/permissions.md:33,39,41`
- **description:** The namespace for the failed-job retry and purge capabilities is inconsistent within the operations spec. `commands.md` advertises `Operations.Job.Retry` (line 152, on `RetryFailedJobCommand`) and `Operations.Job.Purge` (line 166, on `DeleteFailedJobCommand`), while `permissions.md` documents the same operations as `Operations.FailedJob.Retry` (line 39) and `Operations.FailedJob.Purge` (line 41) in the `### FailedJob` section. A reader cannot determine which namespace the engine actually enforces.
- **expected:** All references to the failed-job retry and purge capabilities use a single namespace (either `Operations.FailedJob.*` or `Operations.Job.*`) consistently across `commands.md` and `permissions.md`.
- **evidence:** `docs/specs/operations/commands.md:152` `**Capability:** `Operations.Job.Retry` (system)`; `:166` `**Capability:** `Operations.Job.Purge``; `docs/specs/operations/permissions.md:33` `- `Operations.Job.Retry` (system)`; `:39` `- `Operations.FailedJob.Retry` (system)`; `:41` `- `Operations.FailedJob.Purge``.

---

### FINDING 3

- **id:** SPEC-4-003
- **area:** spec
- **severity:** High
- **location:** `docs/specs/settings/tables.md:106`
- **description:** The `## Cross-Domain Tables (Referenced)` table in settings refers to a `rbac_role_prototypes` table owned by the rbac domain. The rbac spec (`docs/specs/rbac/tables.md:7-17`) defines only `assign_permissions`, `permissions`, `permission_sections`, `roles`, `rbac_module_permissions`, `rbac_module_permission_assigns`, `rbac_role_permissions`, `two_factor_settings`. There is no `rbac_role_prototypes` table anywhere in the workspace, and `rbac/value-objects.md:12-25` does not list such an identifier.
- **expected:** Settings spec references `rbac.roles` (the actual rbac spec table for `Role`) for the `settings_dashboard_settings.role_id` and `settings_general_settings` references, not the non-existent `rbac_role_prototypes`.
- **evidence:** `docs/specs/settings/tables.md:106` `| `rbac_role_prototypes`                | rbac          | Referenced by `role_id`                |`. rbac tables are listed at `docs/specs/rbac/tables.md:7-17` and include `| `roles`                          | Role                     | A role within a school                         |` (line 12) but no `rbac_role_prototypes`.

---

### FINDING 4

- **id:** SPEC-4-004
- **area:** spec
- **severity:** High
- **location:** `docs/specs/operations/tables.md:60,84` vs `docs/specs/rbac/tables.md:11`
- **description:** Operations spec refers to a `rbac_permissions` table owned by rbac, used as the FK target of `rbac_sidebars.permission_id`. The rbac spec defines only `permissions` (no prefix) as the storage table for `Capability`; there is no `rbac_permissions` table. This is a name mismatch between the operations spec's cross-domain reference and the rbac spec's actual table inventory.
- **expected:** Operations spec references `rbac.permissions` (the actual rbac storage table) for `rbac_sidebars.permission_id`, matching the rbac spec's table name.
- **evidence:** `docs/specs/operations/tables.md:60` `- `rbac_sidebars.permission_id` references `rbac_permissions``; `:84` `| `rbac_permissions`              | rbac          | Referenced by `rbac_sidebars`          |`. `docs/specs/rbac/tables.md:11` `| `permissions`                    | Capability (storage row) | Catalog row carrying capability + metadata     |` (no `rbac_` prefix).

---

### FINDING 5

- **id:** SPEC-4-005
- **area:** spec
- **severity:** High
- **location:** `docs/specs/rbac/repositories.md:46,53`
- **description:** `PermissionRepository::get` and `PermissionRepository::delete` declare their `id` parameter as `PermissionSectionId`, but the repository is the storage row for `Capability` (per the heading at line 41) and the spec's identifier for that row is `CapabilityId` (per `docs/specs/rbac/value-objects.md:15` `| `CapabilityId`               | `Id<Capability>`            | A permission row (a capability)    |`). The methods cannot logically take a `PermissionSectionId`; this is a copy/paste from `PermissionSectionRepository` (lines 59-68).
- **expected:** `PermissionRepository::get` and `PermissionRepository::delete` take `CapabilityId` (or a `PermissionId` newtype if one is introduced), not `PermissionSectionId`. Only `list_for_section` should take `PermissionSectionId`.
- **evidence:** `docs/specs/rbac/repositories.md:46` `async fn get(&self, id: PermissionSectionId) -> Result<Option<Permission>>;`; `:53` `async fn delete(&self, id: PermissionSectionId) -> Result<()>;`. The repository heading at `:41` is `## PermissionRepository (the storage row for a `Capability`)` — confirming these should take the capability-row id, not the section id.

---

### FINDING 6

- **id:** SPEC-4-006
- **area:** spec
- **severity:** High
- **location:** `docs/specs/sync/overview.md:1134-1140`
- **description:** The "Phase 0 status" section uses a different command/event vocabulary than the spec body. The spec body (`sync/overview.md:501-588`) defines 6 commands (`RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`) and 7 events (`SyncStarted`, `SyncCompleted`, `SnapshotHydrated`, `ConflictReported`, `ConflictResolved`, `OutboxDrained`, `SubscriptionStateChanged`); the Phase 0 status block reports 4 commands shipped as `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta` and 5 events as `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`. The Phase 0 names do not appear in the spec body; `SyncRequestDelta`, `DeltaAvailable`, `DeltaAcknowledged`, and `SyncAcknowledge` are introduced without definitions.
- **expected:** Phase 0 status uses the same command and event names as the spec body, or the spec body is updated with the Phase 0 names if those are the canonical ones.
- **evidence:** `docs/specs/sync/overview.md:1134-1140` `- **Commands shipped (4 of 6):** `SyncStart`, `SyncPause`, `SyncResume`, `SyncRequestDelta`. The `SyncAcknowledge` command is deferred...` `- **Events shipped (5 of 7):** `SyncStarted`, `SyncPaused`, `SyncResumed`, `DeltaAvailable`, `DeltaAcknowledged`. `SyncConflictDetected` and `SyncStopped` are deferred.` Spec body commands at `:501,517,533,547,563,579` are `RequestSyncCommand`, `PauseSyncCommand`, `ResumeSyncCommand`, `ResolveConflictCommand`, `SwitchSchoolCommand`, `ApplyRemoteChangeCommand`. Spec body events at `:355,372,390,409,427,447,461` are `SyncStarted`, `SyncCompleted`, `SnapshotHydrated`, `ConflictReported`, `ConflictResolved`, `OutboxDrained`, `SubscriptionStateChanged`.

---

### FINDING 7

- **id:** SPEC-4-007
- **area:** spec
- **severity:** High
- **location:** `docs/specs/sync/overview.md:883,1048` (and `:42,64,849,885,941`)
- **description:** The sync spec refers to a worker binary called `educore-worker` (line 883, 1048) and a server crate called `educore-sync-server` / `educore-sync-server-http` (lines 42, 64, 849, 885, 941). Neither crate/binary exists in the workspace. The actual crates are `educore-sync` and `educore-sync-inprocess` (per `docs/specs/sync/overview.md:1132-1133` and `Cargo.toml:88-89`); the binary in the spec inventory is `educore-cli` (per AGENTS.md Crate Inventory row 35).
- **expected:** The sync spec references the actual crate names (`educore-sync`, `educore-sync-inprocess`, `educore-sync-http`, `educore-sync-null`) and the actual binary (`educore-cli`), not the non-existent `educore-worker` and `educore-sync-server*`.
- **evidence:** `docs/specs/sync/overview.md:883` `The worker binary (`educore-worker`) runs in a different process`; `:1048` `- The **worker binary** (`educore-worker` + `WorkerHttpSync`; `:42` `transport protocol. The wire format is the responsibility of `educore-sync-server` and the worker's HTTP client.`; `:64` `- `educore-sync-server` (port) and the wire implementation`; `:849` `uses to talk to a remote `educore-sync-server`.`; `:885` `uses the **HTTP transport** to talk to `educore-sync-server`.`; `:941` `CommandEnvelope` to `educore-sync-server`.`.

---

### FINDING 8

- **id:** SPEC-4-008
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/operations/tables.md:19` vs `docs/specs/operations/aggregates.md:256-292`
- **description:** The `Sidebar` aggregate is declared to belong to the operations domain (`aggregates.md` heading at line 257 and `overview.md:86`), but the storage table is named `rbac_sidebars` (line 19 of `tables.md`). The rbac spec does not list `rbac_sidebars` as a table (rbac/tables.md:7-17 has `rbac_role_permissions` instead, a different aggregate). The table-naming convention from `docs/code-standards.md` and other spec folders would put a per-school operations-owned table under an `operations_` prefix (as `operations_backups`, `operations_maintenance_settings` are, on `operations/tables.md:11,20`).
- **expected:** Either the `Sidebar` storage table is renamed to `operations_sidebars` to match its owning domain, or the aggregate ownership is moved to the rbac domain and the spec explicitly disambiguates it from `RolePermission`.
- **evidence:** `docs/specs/operations/tables.md:19` `| `rbac_sidebars`                | Sidebar            | Per-role sidebar layout projection     |`. `docs/specs/operations/aggregates.md:256-292` declares `## Sidebar` as an operations-owned aggregate (heading at `:257`; root type `Sidebar`; tenant `SchoolId`). `docs/specs/operations/tables.md:11` `| `operations_maintenance_settings`         | MaintenanceSetting | Per-school maintenance mode config     |` and `:20` `| `operations_backups`           | Backup             | Backup records                         |` show the per-domain prefix convention.

---

### FINDING 9

- **id:** SPEC-4-009
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/operations/tables.md:7-23` (column alignment, lines 11/16/20/23)
- **description:** The `operations/tables.md` table is mis-formatted: the second column (`Aggregate`) has inconsistent column widths, and several rows have leading double-spaces in the second column from a copy-paste artifact (rows at lines 11, 16, 23). The malformed rows render as visibly broken Markdown in tools that respect whitespace alignment.
- **expected:** Markdown table is uniformly aligned with consistent single-space column separators.
- **evidence:** `docs/specs/operations/tables.md:11` `| `operations_maintenance_settings`         | MaintenanceSetting | Per-school maintenance mode config     |` (excess leading whitespace before `MaintenanceSetting`); `:16` `| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |` (column separator misplaced); `:23` `| `operations_version_histories`            | VersionHistory     | Version bump records (global)          |`.

---

### FINDING 10

- **id:** SPEC-4-010
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/settings/tables.md:72-75`
- **description:** The `settings_general_settings.academic_id` column is documented twice with contradictory descriptions in adjacent bullets: the first bullet says "is a legacy reference to the bootstrap academic year" (lines 72-73) and the second says "is the active academic year (nullable)" (lines 74-75). Both bullets reference the same column; the duplicate contradicts itself.
- **expected:** A single bullet describing `settings_general_settings.academic_id` (the active academic year, nullable). If a separate legacy bootstrap-academic reference column exists, it is named distinctly and has its own bullet.
- **evidence:** `docs/specs/settings/tables.md:72-75` `- `settings_general_settings.academic_id` is a legacy reference to` `  the bootstrap academic year.` `- `settings_general_settings.academic_id` is the active academic year` `  (nullable).`.

---

### FINDING 11

- **id:** SPEC-4-011
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/settings/value-objects.md:90-131` vs `docs/specs/settings/tables.md:42-56`
- **description:** The settings spec documents a large set of `Module Toggle` value objects (`LessonEnabled`, `ChatEnabled`, `FeesCollectionEnabled`, ..., `LmsCheckout`, etc., 35+ entries) in `value-objects.md:90-131`, while `tables.md:42-56` declares that "**These are dropped in the engine migration; the engine's module system is capability-based and the consumer's `platform_packages.modules` JSON column carries the enabled modules.**" The two files contradict each other: the value-objects file treats these toggles as first-class typed wrappers, while the tables file says they do not exist in the engine.
- **expected:** Either the module-toggle value objects are removed from `value-objects.md` (since the storage rows do not exist), or the tables file is updated to indicate the toggles are retained as typed wrappers over the per-package JSON column.
- **evidence:** `docs/specs/settings/value-objects.md:90-131` "### Module Toggles" lists 35 `bool` toggles including `| `LessonEnabled`     | `bool`                                                       |` through `| `LmsCheckout`       | `bool`                                                       |`. `docs/specs/settings/tables.md:42-56` lists `settings_general_settings.module_toggles` as 35 column names followed by `**These are dropped in the engine migration; the engine's module system is capability-based and the consumer's `platform_packages.modules` JSON column carries the enabled modules.**`.

---

### FINDING 12

- **id:** SPEC-4-012
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/platform/commands.md` (whole file, line range inspected 1-673)
- **description:** The `ModuleManager` aggregate's commands are listed in `platform/aggregates.md:545-547` as `RegisterModuleManager`, `UpdateModuleManager`, `RotatePurchaseCode` but none of these commands is documented in `platform/commands.md`. `commands.md` covers Locale (lines 477-506), AddOn (`InstallAddOn`/`UninstallAddOn` at lines 448-475), and Module (`EnableModule`/`DisableModule` at lines 422-446), but has no `## Module Manager` section.
- **expected:** A `## Module Manager` section in `platform/commands.md` documenting the `RegisterModuleManager`, `UpdateModuleManager`, and `RotatePurchaseCode` commands with their `Command` structs, capabilities, and effects, matching the aggregate-level command list.
- **evidence:** `docs/specs/platform/aggregates.md:543-554` lists `### Commands` `- `RegisterModuleManager` (engine-internal)`, `- `UpdateModuleManager``, `- `RotatePurchaseCode`` under the `## ModuleManager` heading. `docs/specs/platform/commands.md:1-673` contains no `RegisterModuleManager` (verified via search) and no `UpdateModuleManager`. The only `RotatePurchaseCode` reference in the platform spec is in `permissions.md:121`.

---

### FINDING 13

- **id:** SPEC-4-013
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/platform/commands.md` (whole file) vs `docs/specs/platform/aggregates.md:511`
- **description:** The `AddOn` aggregate lists `RegisterAddOn` as a command in `aggregates.md:511` (`- `RegisterAddOn` (engine-internal, build-time)`), but `commands.md` documents only `InstallAddOn` (line 448) and `UninstallAddOn` (line 464) for the AddOn aggregate. The `RegisterAddOn` command has no struct, no capability, and no effects documented anywhere in `platform/commands.md`.
- **expected:** A `RegisterAddOnCommand` struct documented in `platform/commands.md` (alongside `InstallAddOn`/`UninstallAddOn`), with capability (`Platform.AddOn.Register` is already in `permissions.md:112`) and effects (`AddOnRegistered` is in `events.md:300-307`).
- **evidence:** `docs/specs/platform/aggregates.md:511` `- `RegisterAddOn` (engine-internal, build-time)`. `docs/specs/platform/commands.md` line 448 begins `### InstallAddOn` and line 464 begins `### UninstallAddOn`; no `RegisterAddOn` heading exists. `docs/specs/platform/events.md:300-307` declares `### AddOnRegistered` with payload struct, indicating the event exists but the producing command is not documented.

---

### FINDING 14

- **id:** SPEC-4-014
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/platform/entities.md:311-319` vs `docs/specs/platform/aggregates.md:523-554`
- **description:** `ModuleManager` is documented twice: once as an aggregate root in `aggregates.md:523-554` (heading `## ModuleManager` with identity, invariants, commands, events) and once as an entity in `entities.md:311-319` (heading `## ModuleManager` with identity `ModuleManagerId(Uuid)` and `Owner: ModuleManager` — a self-reference that is nonsensical for an entity). Both entries describe the same root type and reference the legacy `InfixModuleManager` brand artifact.
- **expected:** `ModuleManager` exists only in `aggregates.md`; the duplicate entity entry in `entities.md:311-319` is removed.
- **evidence:** `docs/specs/platform/aggregates.md:523-554` `## ModuleManager` ... `**Root type:** `ModuleManager`` ... `### Commands` `RegisterModuleManager` ... `### Events` `ModuleManagerRegistered`. `docs/specs/platform/entities.md:311-319` `## ModuleManager` ... `**Identity:** `ModuleManagerId(Uuid)` (global)` ... `**Owner:** `ModuleManager`` (self-reference). Both reference the same brand artifact (aggregates.md:554 mentions `ModuleManager` aggregate; entities.md:318 says `aggregate replaces the legacy `InfixModuleManager``).

---

### FINDING 15

- **id:** SPEC-4-015
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/operations/aggregates.md:51-87,90-123` (Job / FailedJob) vs `docs/specs/operations/events.md:84-138`
- **description:** `operations/events.md` documents a `SystemVersionBumped` event (lines 208-227) that is described as a "derived event emitted by the operations domain when both a `SystemVersionRegistered` and a `VersionHistoryRecorded` have been observed for the same version." The event is not listed in either the `SystemVersion` aggregate's events (operations/aggregates.md:149-153 lists only `SystemVersionRegistered`, `SystemVersionUpdated`) or the `VersionHistory` aggregate's events (operations/aggregates.md:180-183 lists only `VersionHistoryRecorded`). The event has no owning aggregate.
- **expected:** `SystemVersionBumped` is attributed to one of the existing aggregates (most naturally `SystemVersion`, since it bumps the version), or it is added as a new aggregate root (`SystemVersionBump`) with its own commands and consistency boundary.
- **evidence:** `docs/specs/operations/events.md:208-227` `### SystemVersionBumped` ... `This is a derived event emitted by the operations domain when both a `SystemVersionRegistered` and a `VersionHistoryRecorded` have been observed for the same version.`. `docs/specs/operations/aggregates.md:149-153` lists only `- `SystemVersionRegistered``, `- `SystemVersionUpdated`` for `SystemVersion`; `:180-183` lists only `- `VersionHistoryRecorded`` for `VersionHistory`.

---

### FINDING 16

- **id:** SPEC-4-016
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/operations/repositories.md:179`
- **description:** `repositories.md:179` defines an index `ix_backups_school_id_academic_id ON backups (school_id, academic_id);`, but the `Backup` aggregate's invariants (operations/aggregates.md:17-26) do not include an `academic_id` field, and the `Backup` value objects (operations/value-objects.md:29-35) also do not include an `academic_id` value type. The index references a column that is not documented anywhere in the spec for the `Backup` aggregate.
- **expected:** Either the index is removed (if `backups` has no `academic_id` column), or the `Backup` aggregate adds an `academic_id` invariant and the corresponding value object is documented.
- **evidence:** `docs/specs/operations/repositories.md:179` `CREATE INDEX ix_backups_school_id_academic_id ON backups (school_id, academic_id);`. `docs/specs/operations/aggregates.md:17-26` lists 6 invariants for `Backup` (file_name, file_type, source_link, active_status, restore-in-progress, deletion); none reference `academic_id`. `docs/specs/operations/value-objects.md:29-35` lists `BackupFileName`, `BackupSourceLink`, `BackupFileType`, `BackupLangType`, `BackupActiveStatus`; no `AcademicYearId` or `BackupAcademicId` value object.

---

### FINDING 17

- **id:** SPEC-4-017
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/platform/entities.md:299-310` (ModuleInfo) vs `docs/specs/platform/aggregates.md:88-126` (aggregate table)
- **description:** `ModuleInfo` is declared as an entity in `entities.md:299-310` (with `Owner: Module (logical; used by RBAC to map module ids to their display info)`), but it is also listed as an aggregate root in `overview.md:104` and as a table owner in `tables.md:15` (`| `platform_module_infos`       | ModuleInfo                | Module display info projection     |`). The `aggregates.md` aggregate-by-aggregate definitions do not include a `## ModuleInfo` section (counting 37 aggregates in overview vs. 37 root-type headings in aggregates.md, with no `ModuleInfo` heading).
- **expected:** `ModuleInfo` is consistently classified: either a full aggregate root (with its own invariants, commands, and events in `aggregates.md`) or a pure entity (removed from `overview.md`'s aggregate list and `tables.md`'s aggregate column).
- **evidence:** `docs/specs/platform/overview.md:104` `| ModuleInfo                 | `ModuleInfo`              | A module display info projection                |` (in Aggregate Roots table). `docs/specs/platform/tables.md:15` `| `platform_module_infos`       | ModuleInfo                | Module display info projection     |`. `docs/specs/platform/entities.md:299-310` declares `## ModuleInfo` with `**Owner:** `Module` (logical; used by RBAC to map module ids to their display info)`. `docs/specs/platform/aggregates.md` (heading list) has 37 `## Aggregate` headings but no `## ModuleInfo` heading.

---

### FINDING 18

- **id:** SPEC-4-018
- **area:** spec
- **severity:** Medium
- **location:** `docs/specs/sync/overview.md:903-925` (Permissions section)
- **description:** The sync spec declares six capabilities `Sync.Request`, `Sync.Pause`, `Sync.Resume`, `Sync.ResolveConflict`, `Sync.SwitchSchool`, `Sync.CompactOutbox` in its `## Permissions` section (lines 914-919) and says "Sync capabilities are defined in `educore-rbac`" (line 58 and `:909`). The rbac spec (`docs/specs/rbac/permissions.md`) has no `## Sync` capability section; no `Sync.*` capability is listed anywhere in rbac/spec or rbac/commands. The sync spec's claim that these are defined in the rbac domain is not corroborated by any other file.
- **expected:** Either the sync spec adds the `Sync.*` capabilities to `docs/specs/rbac/permissions.md` under a new `### Rbac.Sync` (or similar) section, or the sync spec is amended to declare that the sync capabilities are owned by the sync subsystem rather than the rbac domain.
- **evidence:** `docs/specs/sync/overview.md:914-919` `| `Sync.Request`           | Any authenticated user with school access | Start a sync session for a school   | ... | `Sync.CompactOutbox`     | Server-side operator role only    | Manually trigger outbox compaction         |`. `docs/specs/rbac/permissions.md` (whole file, 165 lines) contains no `Sync.` capability (no `### Sync` heading or `Sync.` line). The Capability enum fragment at `docs/specs/rbac/value-objects.md:50-86` lists only domain capabilities (Rbac, Platform, Settings, Operations, Student, Finance, etc.) and no `Sync*` variants.

---

### FINDING 19

- **id:** SPEC-4-019
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/sync/overview.md:67-105` (Domain Invariants 9) vs `docs/specs/operations/tables.md` / `docs/specs/settings/tables.md`
- **description:** Sync invariant #9 (`docs/specs/sync/overview.md:99-101`) says "Every sync command carries an `IdempotencyKey`. Resubmitting the same key within the dedupe window returns the prior result, not a duplicate execution." The `IdempotencyKey` type is not declared in the sync spec's own value-object section (`:668-764` lists `CommandEnvelope`, `EventFilter`, `SchoolSnapshot`, `SnapshotRow`, `VersionCursor`, `ConflictId`, `ConflictResolution`). The other domain specs (operations, settings, rbac, platform) also do not declare `IdempotencyKey`; its home is implied to be `educore-core` but no spec file documents it.
- **expected:** `IdempotencyKey` is declared in a value-objects or types section (in `docs/specs/sync/overview.md`'s `## Value Objects` block, or in a shared `docs/specs/core/` types spec).
- **evidence:** `docs/specs/sync/overview.md:99-101` `9. **Idempotency on commands.** Every sync command carries an` `IdempotencyKey`. Resubmitting the same key within the dedupe` `window returns the prior result, not a duplicate execution.`. The sync spec's value-object list at `:668-764` includes `CommandEnvelope` (which carries `idempotency_key: IdempotencyKey` per `:682`) but does not declare `IdempotencyKey` itself.

---

### FINDING 20

- **id:** SPEC-4-020
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/settings/overview.md:53-74` (Domain Invariants)
- **description:** The settings spec's invariants 9 and 12 both reference `BaseGroup`/`role_id` uniqueness, but the spec uses different identifier naming for the same role-binding target. Invariant 9 says "A `BaseGroup::name` is unique within `(school_id, name)`" and invariant 12 says "The pair `(dashboard_sec_id, role_id)` is unique within `(school_id)`." However the spec defines `DashboardSetting` to bind to a role (aggregates.md:309-336), and the value-object table at value-objects.md:21 declares `DashboardSettingId` as the row id — yet `commands.md:274` uses `DashboardSectionId` as the type for `dashboard_sec_id` (a foreign key to the section catalog). The `DashboardSectionId` type is referenced in commands and repositories but never declared as a typed value object or identifier in value-objects.md.
- **expected:** Either `DashboardSectionId` is added to `value-objects.md`'s identifier table (alongside `DashboardSettingId`) with a backing type and notes, or the spec clarifies that `dashboard_sec_id` is a plain `i32` per `tables.md:85` and removes the `DashboardSectionId` type from commands/repositories.
- **evidence:** `docs/specs/settings/commands.md:274` `pub dashboard_sec_id: DashboardSectionId,` (in `CreateDashboardSettingCommand`). `docs/specs/settings/repositories.md:112` `async fn role_count(&self, dashboard_sec_id: DashboardSectionId) -> Result<u64>;`. `docs/specs/settings/value-objects.md:10-27` lists identifiers but `DashboardSectionId` is absent. `docs/specs/settings/tables.md:85` `- `settings_dashboard_settings.dashboard_sec_id` is an `i32` referencing` (treating it as a plain integer).

---

### FINDING 21

- **id:** SPEC-4-021
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/rbac/commands.md:33-35`
- **description:** `commands.md:32-35` for `CreateRole` says "The legacy `InfixRole` shadow aggregate is removed — `is_replicated` is a flag on the engine's `Role`." This is the only file in the rbac spec that references `InfixRole`. AGENTS.md § "Engine Rules" forbids legacy brand references in "new code, comments, commit messages, or documentation." The reference documents a legacy artefact that is otherwise absent from the engine spec.
- **expected:** The `InfixRole` mention is removed from `commands.md` (the engine's `Role` aggregate is the only record; the legacy `InfixRole` does not need to be acknowledged in spec text).
- **evidence:** `docs/specs/rbac/commands.md:33-35` `**Effects:** Creates a `Role` and emits `RoleCreated`. The legacy` ``InfixRole`` `shadow aggregate is removed — `is_replicated` is a flag on the engine's `Role`.`. AGENTS.md § Engine Rules "No legacy names are permitted in new code, comments, commit messages, or documentation." Same reference also appears in `docs/specs/rbac/tables.md:38-40`.

---

### FINDING 22

- **id:** SPEC-4-022
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/operations/tables.md:16` (column alignment)
- **description:** The `oauth_personal_access_clients` row at `operations/tables.md:16` is misaligned — the second column reads `(infrastructure)` with leading whitespace, but the table separator positions are inconsistent across rows.
- **expected:** Markdown table row aligned with the rest of the table (single space after `|`).
- **evidence:** `docs/specs/operations/tables.md:16` `| `oauth_personal_access_clients` | (infrastructure) | OAuth PAT clients                |` (column alignment inconsistent with lines 13, 14, 15, 17, 18).

---

### FINDING 23

- **id:** SPEC-4-023
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/settings/value-objects.md:86-88`
- **description:** `value-objects.md` declares two redundant identifiers `AcademicId` (typed as `AcademicYearId?`) and `UnAcademicId` (typed as `AcademicYearId`, default `1`). Both are scoped to the active academic year on the school; the second appears to be a "previous/legacy" identifier but is not explained anywhere else in the settings spec. `aggregates.md:22-49` defines `GeneralSettings` invariants that mention `session_id` and `language_id` and `date_format_id` but neither `academic_id` nor `un_academic_id`.
- **expected:** Either `AcademicId` is the canonical identifier (and `UnAcademicId` is removed or retyped as `PreviousAcademicYearId` with documentation), or both identifiers are described in the aggregate's invariants section.
- **evidence:** `docs/specs/settings/value-objects.md:86-88` `| `AcademicId`         | `AcademicYearId?`                                                 |` `| `UnAcademicId`       | `AcademicYearId` (default 1)                                      |`. `docs/specs/settings/aggregates.md:22-49` lists invariants for `GeneralSettings` (none of which mentions `AcademicId` or `UnAcademicId`).

---

### FINDING 24

- **id:** SPEC-4-024
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/platform/value-objects.md:191-192`
- **description:** `CurrencyPosition` is documented with the comment "`1` (prefix with space), `2` (suffix with space)". The description enumerates only two of what is typically four positions (also "prefix" without space and "suffix" without space). The platform spec otherwise documents `CurrencyPosition` only as a single concept (`CurrencyPosition` referenced from `platform/aggregates.md:683` as "`currency_type` and `currency_position` are encoded values whose meanings are documented in the value objects"). The enum is incomplete.
- **expected:** `CurrencyPosition` documents all positions: `1` (prefix with space), `2` (suffix with space), `3` (prefix no space), `4` (suffix no space) — or whatever the engine's canonical set is.
- **evidence:** `docs/specs/platform/value-objects.md:191-192` `| `CurrencyPosition`| `1` (prefix with space), `2` (suffix with space)                  |`. The platform spec's only other reference is `platform/aggregates.md:683` `4. `currency_type` and `currency_position` are encoded values` `whose meanings are documented in the value objects.`.

---

### FINDING 25

- **id:** SPEC-4-025
- **area:** spec
- **severity:** Low
- **location:** `docs/specs/operations/workflows.md:6-19` (Backup Lifecycle Workflow)
- **description:** The workflow text "SchoolAdmin (or scheduled job) issues CreateBackupCommand." (line 9) refers to a "scheduled job" producing backups, but the operations spec does not document a `ScheduleBackupCommand` or `BackupSchedule` aggregate. `operations/aggregates.md:15-42` lists `Backup` with commands `CreateBackup`, `DeleteBackup`, `RestoreBackup`, `MarkBackupActive`, `MarkBackupInactive` — no scheduled-create. `operations/entities.md:15-24` documents `BackupSchedule` as an entity "owned by `Backup` (logical)" with the note "The engine does not own a job runner; the schedule is a port-driven configuration that an external adapter reads and acts on." The workflow treats the scheduled job as a first-class actor without the matching command definition.
- **expected:** Either the workflow removes the "scheduled job" reference (since the engine has no scheduler), or a `ScheduleBackupCommand` is added to `commands.md` and listed under `Backup` in `aggregates.md`.
- **evidence:** `docs/specs/operations/workflows.md:9` `1. SchoolAdmin (or scheduled job) issues CreateBackupCommand.`. `docs/specs/operations/commands.md:14-72` documents `CreateBackup`, `DeleteBackup`, `RestoreBackup`, `MarkBackupActive`/`MarkBackupInactive`; no scheduled-create command.

---

### END FINDINGS
Total Findings: 25
