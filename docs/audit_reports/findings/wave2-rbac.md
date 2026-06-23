# Audit findings: educore-rbac (Phase 2 / cross-cutting)

**Scope:** `crates/cross-cutting/rbac/` (10 src files: lib.rs,
aggregate.rs, entities.rs, value_objects.rs, commands.rs, events.rs,
services.rs, repository.rs, query.rs, errors.rs + 1 ids.rs),
`docs/specs/rbac/` (11 files), `docs/commands/rbac.md`,
`docs/events/rbac.md`, `docs/coverage.toml` (rbac rows),
`docs/ports/authentication.md`, `docs/handoff/PHASE-2-HANDOFF.md`,
`AGENTS.md` (the rbac row).

**Total findings:** 36

---

### FINDING 1

- **id:** CROSSCUT-RBAC-001
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/value_objects.rs:4148-4767
- **description:** `Capability::all()` is missing 46 of the 654 enum
  variants — every Phase 15 variant (Auth, Notify, Payment, Files,
  Integrations). The function enumerates 608 variants; the enum
  contains 654 variants. The 46 missing are: `AuthLogin`,
  `AuthLogout`, `AuthRefresh`, `AuthRevoke`, `AuthPasswordReset`,
  `OAuthAccessTokenRead`, `OAuthAccessTokenRevoke`, `OAuthClientRead`,
  `OAuthClientManage`, `PasswordResetRequest`, `PasswordResetConfirm`,
  `MfaEnroll`, `MfaVerify`, `NotifyEmailSend`, `NotifySmsSend`,
  `NotifyPushSend`, `NotifyInApp`, `NotifyVoice`, `NotifyWebhook`,
  `NotifyTemplateRead`, `NotifyTemplateWrite`, `NotifyBulkSend`,
  `PaymentCharge`, `PaymentRefund`, `PaymentStatus`, `PaymentMethodList`,
  `PaymentWebhook`, `PaymentSettlement`, `BankSlipGenerate`,
  `BankSlipApprove`, `FilesPut`, `FilesGet`, `FilesDelete`,
  `FilesSignedUrl`, `FilesCopy`, `FilesMove`, `FilesVisibilityChange`,
  `FilesLifecycle`, `IntegrationInvoke`,
  `IntegrationListCapabilities`, `IntegrationHealth`,
  `IntegrationConfigure`, `WebhookOut`, `PollingIn`, `LmsRosterSync`,
  `VideoSchedule`.
- **expected:** `Capability::all()` must enumerate every variant of the
  `Capability` enum so consumers can iterate the full catalog.
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:4765`
  final entry is `Self::OperationsSidebarReorder`; the enum continues
  with `AuthLogin` at line 1375, `NotifyEmailSend` at 1411,
  `PaymentCharge` at 1436, `FilesPut` at 1459, `IntegrationInvoke` at
  1480. `comm -23` between the enum variant set and the `all()` set
  shows exactly 46 missing items (all Phase 15 caps). Confirmed by
  `bash> comm -23 /tmp/enum_variants.txt /tmp/all_variants.txt | wc -l`
  returning `46`.

---

### FINDING 2

- **id:** CROSSCUT-RBAC-002
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/services.rs:344-347
- **description:** `DefaultRoleCatalog::super_admin()` is defined as
  `Capability::all().iter().copied().collect()`. Because `all()` is
  missing 46 variants (Finding 1), the `super_admin` set is missing
  every Auth, Notify, Payment, Files, and Integrations capability. The
  spec mandates: "The SuperAdmin role is a system role and cannot be
  deleted. It holds every registered `Capability` at the time of
  school creation."
- **expected:** `docs/specs/rbac/permissions.md:84-86` — "The SuperAdmin
  role is a system role and cannot be deleted. It holds every
  registered Capability at the time of school creation and is
  refreshed on engine startup to pick up newly registered
  capabilities."
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:344-347`:
  ```rust
  #[must_use]
  pub fn super_admin() -> BTreeSet<Capability> {
      Capability::all().iter().copied().collect()
  }
  ```
  No explicit seeding of the 46 Phase 15 caps; relies on
  `Capability::all()`.

---

### FINDING 3

- **id:** CROSSCUT-RBAC-003
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/services.rs:762-776
- **description:** `DefaultRoleCatalog::school_admin()` adds a
  filter-on-`Capability::all()` for `Auth./Notify./Payment./Files./
  Integrations.` prefixed capabilities. Because `all()` is missing all
  46 Phase 15 variants (Finding 1), the filter is a no-op and the
  `school_admin` set has zero Auth/Notify/Payment/Files/Integrations
  capabilities — contradicting the spec mapping.
- **expected:** `docs/specs/rbac/permissions.md:75` — SchoolAdmin row
  says "All Rbac.Role.*, all Rbac.Capability.*, all domain
  capabilities".
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:762-776`:
  ```rust
  s.extend(
      crate::value_objects::Capability::all()
          .iter()
          .copied()
          .filter(|c| {
              let s = c.as_str();
              s.starts_with("Settings.")
                  || s.starts_with("Operations.")
                  || s.starts_with("Auth.")
                  || s.starts_with("Notify.")
                  || s.starts_with("Payment.")
                  || s.starts_with("Files.")
                  || s.starts_with("Integrations.")
          }),
  );
  ```
  Filter depends on `Capability::all()`, which is missing the 46
  Phase 15 caps.

---

### FINDING 4

- **id:** CROSSCUT-RBAC-004
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/rbac/src/services.rs:1088-1093,
  crates/cross-cutting/rbac/tests/rbac_e2e.rs:58-64
- **description:** `super_admin_role_includes_every_capability` test
  passes by symmetry: both the test assertion and the
  `super_admin()` impl iterate `Capability::all()`. Because `all()` is
  missing 46 variants (Finding 1), the test only confirms the
  (truncated) 608 are in the (truncated) 608 — it does not confirm
  the spec invariant that "SuperAdmin holds every registered
  Capability".
- **expected:** A test that asserts `super_admin` contains every
  variant of the `Capability` enum, not just every entry in `all()`.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:1088-1093`:
  ```rust
  #[test]
  fn super_admin_role_includes_every_capability() {
      let all: BTreeSet<Capability> = DefaultRoleCatalog::super_admin();
      for c in Capability::all() {
          assert!(all.contains(c), "missing capability {c:?} in super_admin");
      }
  }
  ```
  Identical pattern in `crates/cross-cutting/rbac/tests/rbac_e2e.rs:58-64`.

---

### FINDING 5

- **id:** CROSSCUT-RBAC-005
- **area:** cross-cutting
- **severity:** High
- **location:** crates/cross-cutting/rbac/src/aggregate.rs:1-7,
  docs/specs/rbac/aggregates.md:189-320
- **description:** 5 of 8 RBAC aggregates declared in the spec are
  absent from `aggregate.rs` (or anywhere in the crate). The spec
  declares: `ModulePermission`, `ModulePermissionAssign`,
  `RolePermission`, `TwoFactorSetting`, `PermissionOverride`. The
  crate's own `lib.rs` docstring acknowledges this: "The five
  secondary RBAC aggregates (`TwoFactorSetting`, `Override`,
  `ModulePermission`, `ModulePermissionAssign`, `RolePermission`)
  land in later phases."
- **expected:** `docs/specs/rbac/aggregates.md` lines 189-320 list 5
  secondary aggregates; each has a spec-defined Commands, Events,
  and Invariants section.
- **evidence:** `crates/cross-cutting/rbac/src/aggregate.rs:1-7`
  module docstring confirms only `Role`, `Permission`,
  `PermissionSection` are in scope. `grep -nE "struct
  ModulePermission|struct RolePermission|struct TwoFactorSetting|struct
  PermissionOverride" crates/cross-cutting/rbac/src/` returns no
  matches.

---

### FINDING 6

- **id:** CROSSCUT-RBAC-006
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/commands.rs:1-193
- **description:** 17 of 22 RBAC commands declared in
  `docs/commands/rbac.md` are absent. Implemented: `CreateRole`,
  `UpdateRole`, `DeleteRole`, `AssignCapability`, `RevokeCapability`
  (5). Missing: `CloneRole`, `DeletePermissionAssignment`,
  `UpdatePermissionAssignment`, `CreateModulePermission`,
  `UpdateModulePermission`, `DeleteModulePermission`,
  `AssignModulePermission`, `RevokeModulePermission`, `GrantMenuLink`,
  `RevokeMenuLink`, `CreatePermissionSection`,
  `UpdatePermissionSection`, `DeletePermissionSection`,
  `ConfigureTwoFactor`, `TestTwoFactorDelivery`,
  `SetPermissionOverride`, `ClearPermissionOverride`.
- **expected:** `docs/commands/rbac.md:11-34` — table of 22 commands.
- **evidence:** `crates/cross-cutting/rbac/src/commands.rs:1-193`
  defines only the 5 phase-2 command structs. `grep -nE "CloneRole
  Command|DeletePermissionAssignmentCommand|UpdatePermissionAssign
  mentCommand|ConfigureTwoFactorCommand|SetPermissionOverrideCommand"
  crates/cross-cutting/rbac/src/commands.rs` returns no matches.

---

### FINDING 7

- **id:** CROSSCUT-RBAC-007
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/events.rs:1-478
- **description:** 18 of 23 RBAC events declared in
  `docs/events/rbac.md` are absent. Implemented: `RoleCreated`,
  `RoleUpdated`, `RoleDeleted`, `CapabilityAssigned`,
  `CapabilityRevoked` (5). Missing: `RoleCloned`,
  `CapabilityRegistered`, `PermissionMetadataUpdated`,
  `PermissionAssignmentUpdated`, `ModulePermissionCreated`,
  `ModulePermissionUpdated`, `ModulePermissionDeleted`,
  `ModulePermissionAssigned`, `ModulePermissionRevoked`,
  `MenuLinkGranted`, `MenuLinkRevoked`, `PermissionSectionCreated`,
  `PermissionSectionUpdated`, `PermissionSectionDeleted`,
  `TwoFactorConfigured`, `TwoFactorDeliveryTested`,
  `PermissionOverrideSet`, `PermissionOverrideCleared`.
- **expected:** `docs/events/rbac.md:9-33` — table of 23 events.
- **evidence:** `crates/cross-cutting/rbac/src/events.rs:1-478`
  defines only the 5 phase-2 events. `grep -nE
  "RoleCloned|CapabilityRegistered|PermissionMetadataUpdated|
  ModulePermissionCreated|TwoFactorConfigured|PermissionOverrideSet"
  crates/cross-cutting/rbac/src/events.rs` returns no matches.

---

### FINDING 8

- **id:** CROSSCUT-RBAC-008
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/services.rs:283-331
- **description:** 7 of 8 services declared in
  `docs/specs/rbac/services.md` are absent. Implemented:
  `CapabilityCheck` (named `CapabilityCheck` instead of
  `CapabilityCheckService` per spec) and a partial `RoleService`
  (missing `expand_with_inheritance`). Missing: `TwoFactorService`,
  `PermissionSectionService`, `MenuLinkService`,
  `ModulePermissionService`, `OverrideService`,
  `BootstrapService`, plus the two policies (`SystemRoleImmutability`,
  `SelfRevocationGuard`) and the two specifications
  (`RolesWithCapability`, `ActiveRoles`).
- **expected:** `docs/specs/rbac/services.md:1-191` — 8 services + 2
  policies + 2 specifications.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:1-1192`
  defines only `CapabilityCheck`, `InMemoryCapabilityCheck`,
  `RoleService`, `DefaultRoleCatalog`. `grep -nE
  "struct TwoFactorService|struct PermissionSectionService|struct
  MenuLinkService|struct ModulePermissionService|struct
  OverrideService|struct BootstrapService|struct
  SystemRoleImmutability|struct SelfRevocationGuard|struct
  RolesWithCapability|struct ActiveRoles"
  crates/cross-cutting/rbac/src/services.rs` returns no matches.

---

### FINDING 9

- **id:** CROSSCUT-RBAC-009
- **area:** cross-cutting
- **severity:** Critical
- **location:** crates/cross-cutting/rbac/src/repository.rs:1-162
- **description:** 6 of 10 RBAC repository port traits declared in
  `docs/specs/rbac/repositories.md` are absent. Implemented:
  `RoleRepository`, `AssignPermissionRepository`,
  `PermissionRepository`, `PermissionSectionRepository` (4). Missing:
  `ModulePermissionRepository`, `ModulePermissionAssignRepository`,
  `RolePermissionRepository`, `TwoFactorSettingRepository`,
  `PermissionOverrideRepository`, `TwoFactorDeliveryRepository`.
- **expected:** `docs/specs/rbac/repositories.md:1-135` — 10 port
  traits.
- **evidence:** `crates/cross-cutting/rbac/src/repository.rs:1-162`
  defines 4 traits. `grep -nE "trait
  ModulePermissionRepository|trait RolePermissionRepository|trait
  TwoFactorSettingRepository|trait PermissionOverrideRepository"
  crates/cross-cutting/rbac/src/repository.rs` returns no matches.

---

### FINDING 10

- **id:** CROSSCUT-RBAC-010
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/value-objects.md:12-24
- **description:** 8 of 11 typed identifiers declared in the spec
  value-objects table are absent. The spec lists: `RoleId`,
  `CapabilityId`, `PermissionSectionId`, `AssignPermissionId`,
  `ModulePermissionId`, `ModulePermissionAssignId`, `RolePermissionId`,
  `TwoFactorSettingId`, `RoleBindingId`, `PermissionOverrideId`,
  `TwoFactorDeliveryId`. Code has: `RoleId`, `PermissionId` (renamed
  from `CapabilityId`), `PermissionSectionId`, `AssignPermissionId` (4
  of 11). Missing: `ModulePermissionId`, `ModulePermissionAssignId`,
  `RolePermissionId`, `TwoFactorSettingId`, `RoleBindingId`,
  `PermissionOverrideId`, `TwoFactorDeliveryId`.
- **expected:** `docs/specs/rbac/value-objects.md:12-24` — table of
  11 typed identifiers.
- **evidence:** `crates/cross-cutting/rbac/src/ids.rs:57-75` defines
  4 typed id structs. `grep -nE "ModulePermissionId|RolePermissionId|
  TwoFactorSettingId|PermissionOverrideId" crates/cross-cutting/rbac/
  src/ids.rs` returns no matches.

---

### FINDING 11

- **id:** CROSSCUT-RBAC-011
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/value-objects.md:122-130
- **description:** 4 of 4 two-factor value objects from the spec are
  absent from `value_objects.rs`. Spec defines: `TwoFactorChannel`
  (`Sms | Email`), `TwoFactorMode` (present, but spec also requires
  `Required | Optional | Disabled` as the only variants),
  `TwoFactorExpiry` (`u32` seconds, 0..86400, typically 60..3600),
  `OtpCode` (4..10 digits), `OtpHash` (OtpCode after hashing).
  Code has only `TwoFactorMode` (with the correct variants but
  missing the `from_repr` constructor spec mentions) and is missing
  `TwoFactorChannel`, `TwoFactorExpiry`, `OtpCode`, `OtpHash`.
- **expected:** `docs/specs/rbac/value-objects.md:122-130`.
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:1-6206`
  defines 8 value objects; `TwoFactorChannel` and `TwoFactorExpiry`
  are not present. `grep -nE "TwoFactorChannel|TwoFactorExpiry|
  OtpCode|OtpHash" crates/cross-cutting/rbac/src/value_objects.rs`
  returns no matches (the literal `TwoFactor` appears only inside
  the comment at line 5762-5764).

---

### FINDING 12

- **id:** CROSSCUT-RBAC-012
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/value-objects.md:96-112
- **description:** 11 of 12 permission-metadata value objects from
  the spec are absent. Spec defines: `PermissionName` (1..191 chars),
  `Route` (1..191 chars), `ParentRoute`, `PermissionType` (present),
  `LangName` (1..191 chars), `Icon` (up to 2000 chars), `Position`
  (`i32`), `RelateToChild` (`bool`), `IsMenu` (`bool`),
  `IsAdmin` (`bool`), `IsTeacher` (`bool`), `IsStudent` (`bool`),
  `IsParent` (`bool`), `IsAlumni` (`bool`), `AlternateModule`. Code
  has only `PermissionType` (encoded as enum with `Menu/SubMenu/
  Action` byte variants). Missing: `PermissionName`, `Route`,
  `ParentRoute`, `LangName`, `Icon`, `Position`, `RelateToChild`,
  `IsMenu`, `IsAdmin`, `IsTeacher`, `IsStudent`, `IsParent`,
  `IsAlumni`, `AlternateModule`.
- **expected:** `docs/specs/rbac/value-objects.md:96-112` — table of
  12 permission-metadata types.
- **evidence:** `grep -nE "struct PermissionName|struct Route|struct
  LangName|struct Icon|struct Position" crates/cross-cutting/rbac/
  src/value_objects.rs` returns no matches. The `Permission` struct
  in `aggregate.rs:97-126` carries only `lang_name: String` (no
  validation), `module: String`, `type_: PermissionType`, no
  `route`, `parent_route`, `icon`, `position`, `is_menu`,
  `is_admin`, `is_teacher`, `is_student`, `is_parent`, `is_alumni`,
  or `alternate_module` fields.

---

### FINDING 13

- **id:** CROSSCUT-RBAC-013
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/value-objects.md:132-140
- **description:** 4 of 4 module value objects from the spec are
  absent. Spec defines: `ModuleName` (1..200 chars, unique within
  `school_id`), `DashboardId` (`u32`), `ModulePosition` (`i32`),
  `ModuleStatus` (`Active | Inactive`). Code has none.
- **expected:** `docs/specs/rbac/value-objects.md:132-140`.
- **evidence:** `grep -nE "struct ModuleName|struct DashboardId|
  struct ModulePosition|struct ModuleStatus" crates/cross-cutting/
  rbac/src/value_objects.rs` returns no matches.

---

### FINDING 14

- **id:** CROSSCUT-RBAC-014
- **area:** cross-cutting
- **severity:** Medium
- **location:** docs/specs/rbac/value-objects.md:39-44
- **description:** 4 of 4 spec value-object categories are absent
  from `value_objects.rs`: `CapabilityString` (newtype around
  `String` with validated construction; spec line 167-177),
  `CapabilityAction` (verb in present tense enum; spec line 42),
  `CapabilityScope` (`Tenant | System`; spec line 43), and the
  `RoleStatus` / `RoleNamePatch` value objects from the Role
  category (spec line 32-33).
- **expected:** `docs/specs/rbac/value-objects.md:32-44` and 167-177.
- **evidence:** `grep -nE "struct CapabilityString|enum
  CapabilityAction|enum CapabilityScope|enum RoleStatus|struct
  RoleNamePatch" crates/cross-cutting/rbac/src/value_objects.rs`
  returns no matches. The `Capability` enum has a `domain()` and
  `aggregate()` method but no `action()` enumeration as a typed
  enum (the `action()` method returns `&'static str`).

---

### FINDING 15

- **id:** CROSSCUT-RBAC-015
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/entities.md:1-150
- **description:** All 15 RBAC entities declared in
  `docs/specs/rbac/entities.md` are absent (some are platform-side
  projections, some are owned by the RBAC domain). Spec entities:
  `RoleBinding`, `PermissionOverride`, `ModuleLinkBinding`,
  `CapabilityGrantEventProjection`, `RoleHierarchyEdge`,
  `TwoFactorDelivery`, `OtpCodeRow`, `DashboardSection`,
  `PermissionTranslation`, `RoleMembershipSnapshot`,
  `CapabilityCatalog`, `SidebarEntry`, `SidebarPosition`,
  `TwoFactorAuditEntry`, `CapabilitySearchIndex`. Code has only
  `AssignPermission` (not in the entities.md list, but in
  `aggregates.md`).
- **expected:** `docs/specs/rbac/entities.md:1-150` — 15 entities
  documented.
- **evidence:** `crates/cross-cutting/rbac/src/entities.rs:1-162`
  defines only `AssignPermission`. `grep -nE "RoleBinding|
  PermissionOverride|ModuleLinkBinding|TwoFactorDelivery" crates/
  cross-cutting/rbac/src/entities.rs` returns no matches (the
  comment at line 6-7 lists them as "land in later phases").

---

### FINDING 16

- **id:** CROSSCUT-RBAC-016
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/permissions.md:19-63
- **description:** 6 of 9 permission groups declared in
  `docs/specs/rbac/permissions.md` have no corresponding
  `Capability` variants. The spec lists 9 groups: `Rbac.Role` (8
  caps), `Rbac.Capability` (4 caps), `Rbac.Section` (4 caps), `Rbac.
  ModulePermission` (6 caps), `Rbac.TwoFactor` (3 caps), `Rbac.
  Override` (3 caps). Code has 12 `Rbac.*` caps (`RbacRole*` +
  `RbacCapability*` + `RbacBootstrap`) but zero `Rbac.Section`,
  `Rbac.ModulePermission`, `Rbac.TwoFactor`, or `Rbac.Override`
  variants — the cap string forms `Rbac.Section.Create` etc. are
  not parseable.
- **expected:** `docs/specs/rbac/permissions.md:19-63` — 9 permission
  groups and 30+ cap string forms.
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:46-69`
  lists 12 `Rbac.*` variants. `grep -nE "RbacSection|RbacModule
  Permission|RbacTwoFactor|RbacOverride" crates/cross-cutting/rbac/
  src/value_objects.rs` returns no matches. The `from_str_opt`
  function at line 4772-5442 has no arm for any of the 20+ spec
  `Rbac.Section.* / Rbac.ModulePermission.* / Rbac.TwoFactor.* /
  Rbac.Override.*` strings.

---

### FINDING 17

- **id:** CROSSCUT-RBAC-017
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/permissions.md:23-28
- **description:** 2 of 8 `Rbac.Role` cap variants from the spec are
  absent. Spec: `Rbac.Role.GrantMenu`, `Rbac.Role.RevokeMenu`. Code
  has `RbacRoleCreate/Read/Update/Delete/Manage/Clone` but not
  `RbacRoleGrantMenu` or `RbacRoleRevokeMenu`.
- **expected:** `docs/specs/rbac/permissions.md:27-28` — "`Rbac.Role.
  GrantMenu`, `Rbac.Role.RevokeMenu`".
- **evidence:** `crates/cross-cutting/rbac/src/value_objects.rs:46-69`
  lists 6 RbacRole caps. `grep -nE "RbacRoleGrantMenu|RbacRoleRevoke
  Menu" crates/cross-cutting/rbac/src/value_objects.rs` returns no
  matches. The commands in `commands.md:26-27` (`GrantMenuLink`,
  `RevokeMenuLink`) reference these cap strings.

---

### FINDING 18

- **id:** CROSSCUT-RBAC-018
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/workflows.md:79-92
- **description:** Two-Factor Enrollment Workflow (workflows.md
  § 4) is unimplemented. The workflow describes 7 steps:
  `SchoolAdmin updates TwoFactorSetting` → `TwoFactorConfigured`
  emitted → 2FA prompted on login → OTP via channel → user enters
  OTP → session granted → `TwoFactorDeliveryTested` emitted. The
  `ConfigureTwoFactor` command, the `TwoFactorSetting` aggregate,
  the `TwoFactorConfigured` event, the `TwoFactorDeliveryTested`
  event, and the `TwoFactorService` are all absent (see Findings
  5, 6, 7, 8).
- **expected:** `docs/specs/rbac/workflows.md:79-92`.
- **evidence:** `grep -rnE "TwoFactorConfigured|TwoFactorDelivery
  Tested|ConfigureTwoFactor" crates/cross-cutting/rbac/src/`
  returns no matches. `grep -nE "TwoFactor" crates/cross-cutting/
  rbac/src/` returns only one comment reference at
  `value_objects.rs:5762`.

---

### FINDING 19

- **id:** CROSSCUT-RBAC-019
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/workflows.md:115-126
- **description:** Role Cloning Workflow is unimplemented. The
  workflow requires `CloneRole` command + `RoleCloned` event + the
  ability to copy `AssignPermission`, `RolePermission`, and
  `ModulePermissionAssign` rows. None of these are present (see
  Findings 6, 7, 9).
- **expected:** `docs/specs/rbac/workflows.md:115-126` — 8-step
  workflow ending in `RoleCloned` emit.
- **evidence:** `grep -nE "CloneRole|RoleCloned" crates/cross-
  cutting/rbac/src/` returns no matches. The `Capability::RbacRole
  Clone` variant exists at `value_objects.rs:58` but no command or
  event uses it.

---

### FINDING 20

- **id:** CROSSCUT-RBAC-020
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/workflows.md:128-141
- **description:** Override Workflow is unimplemented. The workflow
  requires `SetPermissionOverride` command, `PermissionOverrideSet`
  event, `PermissionOverride` aggregate, `OverrideService`, and
  `PermissionOverrideRepository` — all absent (see Findings 5, 6, 7,
  8, 9).
- **expected:** `docs/specs/rbac/workflows.md:128-141` — 6-step
  workflow.
- **evidence:** `grep -rnE "SetPermissionOverride|PermissionOverride
  Set|PermissionOverride" crates/cross-cutting/rbac/src/`
  returns only one reference at
  `services.rs:29` (a comment about a Phase 2 follow-up).

---

### FINDING 21

- **id:** CROSSCUT-RBAC-021
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/workflows.md:143-153
- **description:** Menu Visibility Workflow is unimplemented. The
  workflow requires `GrantMenuLink` / `RevokeMenuLink` commands,
  `MenuLinkGranted` / `MenuLinkRevoked` events, `RolePermission`
  aggregate, `MenuLinkService`, `RolePermissionRepository` — all
  absent (see Findings 5, 6, 7, 8, 9).
- **expected:** `docs/specs/rbac/workflows.md:143-153` — 5-step
  workflow.
- **evidence:** `grep -rnE "GrantMenuLink|RevokeMenuLink|MenuLink"
  crates/cross-cutting/rbac/src/` returns no matches. The
  `Capability::RbacRoleGrantMenu` / `RbacRoleRevokeMenu` cap
  variants are also absent (Finding 17).

---

### FINDING 22

- **id:** CROSSCUT-RBAC-022
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/workflows.md:6-31
- **description:** School Bootstrap Workflow is unimplemented. The
  workflow describes 7 steps: create school → bootstrap
  SuperAdmin role → seed every Capability → create first user →
  create default PermissionSection list → create default
  TwoFactorSetting → seed baseline ModulePermissions. The crate
  has no `BootstrapService` (Finding 8) and no command handler
  implementation, so the bootstrap path is not wired.
- **expected:** `docs/specs/rbac/workflows.md:6-31` — 7-step
  workflow with pre-conditions and failure paths.
- **evidence:** `grep -rnE "BootstrapService|seed_role_catalog|
  default_two_factor_setting" crates/cross-cutting/rbac/src/`
  returns no matches. The `PHASE-2-HANDOFF.md:130-131` claims
  the 10 default role constructors are shipped, but does not
  describe a BootstrapService that wires them into a new school.

---

### FINDING 23

- **id:** CROSSCUT-RBAC-023
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/commands.md:69-82
- **description:** `CloneRoleCommand` struct is absent despite
  `RbacRoleClone` capability and `RoleCloned` event both being
  documented. The `Capability::RbacRoleClone` variant exists
  (value_objects.rs:58) but no command consumes it.
- **expected:** `docs/specs/rbac/commands.md:69-82` — full
  `CloneRoleCommand` struct definition.
- **evidence:** `grep -nE "CloneRoleCommand|struct CloneRole"
  crates/cross-cutting/rbac/src/commands.rs` returns no matches.

---

### FINDING 24

- **id:** CROSSCUT-RBAC-024
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/commands.md:120-148
- **description:** `DeletePermissionAssignmentCommand` and
  `UpdatePermissionAssignmentCommand` structs are absent. The
  spec defines both with their own `Capability` requirements
  and effects. The hard-delete vs. soft-denial distinction
  is collapsed into a single `as_denial: bool` flag on
  `RevokeCapabilityCommand` (commands.rs:81-91), which conflates
  two distinct user-intents from the spec.
- **expected:** `docs/specs/rbac/commands.md:120-148` — two
  separate commands.
- **evidence:** `grep -nE "DeletePermissionAssignmentCommand|Update
  PermissionAssignmentCommand" crates/cross-cutting/rbac/src/
  commands.rs` returns no matches.

---

### FINDING 25

- **id:** CROSSCUT-RBAC-025
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/commands.md:241-275
- **description:** `ConfigureTwoFactorCommand` and
  `TestTwoFactorDeliveryCommand` structs are absent despite
  the `Rbac.TwoFactor.Configure` capability being documented
  (spec permissions.md:54-57).
- **expected:** `docs/specs/rbac/commands.md:241-275` — full
  command struct definitions including the per-role
  `TwoFactorMode` fields and the test-delivery recipient.
- **evidence:** `grep -nE "ConfigureTwoFactorCommand|TestTwoFactor
  DeliveryCommand" crates/cross-cutting/rbac/src/commands.rs`
  returns no matches.

---

### FINDING 26

- **id:** CROSSCUT-RBAC-026
- **area:** cross-cutting
- **severity:** High
- **location:** docs/specs/rbac/commands.md:279-307
- **description:** `SetPermissionOverrideCommand` and
  `ClearPermissionOverrideCommand` structs are absent. The
  spec defines them with `OverrideReason`, `expires_at: Option
  <Timestamp>`, and `PermissionOverrideId` typed identifiers.
- **expected:** `docs/specs/rbac/commands.md:279-307` — full
  command struct definitions.
- **evidence:** `grep -nE "SetPermissionOverrideCommand|Clear
  PermissionOverrideCommand" crates/cross-cutting/rbac/src/
  commands.rs` returns no matches.

---

### FINDING 27

- **id:** CROSSCUT-RBAC-027
- **area:** cross-cutting
- **severity:** Medium
- **location:** docs/specs/rbac/value-objects.md:14-15
- **description:** The spec calls the storage-row id `CapabilityId`
  (`Id<Capability>`), but the code names it `PermissionId`. This
  is a doc-vs-code drift; consumers reading the spec expect
  `CapabilityId` and the public API exposes `PermissionId`.
- **expected:** `docs/specs/rbac/value-objects.md:15` — "`CapabilityId`
  | `Id<Capability>` | A permission row (a capability)".
- **evidence:** `crates/cross-cutting/rbac/src/ids.rs:62-65`:
  ```rust
  rbac_typed_id! {
      /// A typed id for a [`Permission`](crate::aggregate::Permission) row.
      pub struct PermissionId;
  }
  ```
  The struct is named `PermissionId`, not `CapabilityId`.

---

### FINDING 28

- **id:** CROSSCUT-RBAC-028
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/value_objects.rs:1110
- **description:** The `Permission` struct stores `lang_name: String`
  with no validation, despite the spec requiring `LangName` to be
  1..191 chars (spec value-objects.md:103). The associated value
  object `LangName` is not implemented (Finding 12).
- **expected:** `docs/specs/rbac/value-objects.md:103` — "`LangName` |
  1..191 chars, the i18n key".
- **evidence:** `crates/cross-cutting/rbac/src/aggregate.rs:108-109`:
  ```rust
  /// Localized display name (i18n key, not the translated text).
  pub lang_name: String,
  ```
  No length check at construction; the `Permission` struct uses
  `String` rather than a typed `LangName` newtype.

---

### FINDING 29

- **id:** CROSSCUT-RBAC-029
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/commands.rs:27-37
- **description:** The `CreateRoleCommand` does not validate that
  `role_type == System` requires the `RbacRoleManage` capability.
  The spec mandates "system roles require `Rbac.Role.Manage`" but
  the validation is pushed to the (non-existent) command handler.
  The struct itself accepts any `RoleType` regardless of caller
  capability.
- **expected:** `docs/specs/rbac/commands.md:27-30` — "Pre-conditions:
  ... `role_type` is allowed for the actor (system roles require
  `Rbac.Role.Manage`)."
- **evidence:** `crates/cross-cutting/rbac/src/commands.rs:27-37`
  has no validation method; the struct is a passive data carrier.
  `grep -n "validate\|RbacRoleManage" crates/cross-cutting/rbac/
  src/commands.rs` returns no matches.

---

### FINDING 30

- **id:** CROSSCUT-RBAC-030
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/services.rs:199-213
- **description:** The bootstrap backstop in
  `apply_bootstrap_backstop` grants every `Rbac.*` capability to
  any actor with `RbacRoleManage` (or any system actor), including
  the `RbacBootstrap` capability. The spec self-revocation guard
  (permissions.md:148-156) requires that the engine refuse any
  revocation that would leave the actor without the capability
  required to undo the command. There is no `SelfRevocationGuard`
  policy / spec implementation (Finding 8).
- **expected:** `docs/specs/rbac/permissions.md:146-157` —
  "Self-Revocation Guard" section.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:199-213`
  hard-codes the backstop. `grep -nE "SelfRevocationGuard|self_revo
  cation" crates/cross-cutting/rbac/src/` returns only
  `errors.rs:78-83` (an unused error constructor; no enforcement).

---

### FINDING 31

- **id:** CROSSCUT-RBAC-031
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/rbac/src/services.rs:42-54
- **description:** The `CapabilityExplanation` struct uses
  `pub overrides: Vec<CapabilityOverride>` (a Phase 2 stub with
  `id: Uuid, granted: bool`) but the spec defines
  `overrides: Vec<PermissionOverrideId>` (a typed id). The struct
  is intended to be the wire contract for the audit log and the
  "why is this denied?" diagnostic screen, and the divergence
  means the explanation payload is non-conformant.
- **expected:** `docs/specs/rbac/services.md:53-56` — "`pub overrides:
  Vec<PermissionOverrideId>`".
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:42-54`:
  ```rust
  pub struct CapabilityExplanation {
      pub capability: Capability,
      pub decision: bool,
      pub role_grants: Vec<RoleId>,
      pub overrides: Vec<CapabilityOverride>,
      pub system_fallback: bool,
  }
  ```
  `overrides` carries `CapabilityOverride` (id+granted bool), not
  `PermissionOverrideId`.

---

### FINDING 32

- **id:** CROSSCUT-RBAC-032
- **area:** cross-cutting
- **severity:** Low
- **location:** crates/cross-cutting/rbac/src/services.rs:162-180
- **description:** `InMemoryCapabilityCheck::grants_for` comments
  acknowledge that "the storage-backed impl will read the
  user→role bindings" but the in-memory implementation
  enumerates ALL roles in the school, not the actor's bound
  roles. This means a Teacher in a school where any role holds
  `RbacRoleManage` will be reported as having `RbacBootstrap`
  (and every other Rbac.* cap). The `rbac_bootstrap_is_never_
  revocable` test at rbac_e2e.rs:111-151 works around this by
  using a fresh `other_school` to verify the deny path.
- **expected:** Per-role binding lookup, not school-wide sum.
- **evidence:** `crates/cross-cutting/rbac/src/services.rs:162-180`:
  ```rust
  // The Phase 2 in-memory check accepts a single role id via
  // the session. For now we just sum all roles in the school
  // — the storage-backed impl will read the user→role
  // bindings.
  let mut caps = BTreeSet::new();
  for set in by_school.values() {
      caps.extend(set.iter().copied());
  }
  ```
  No `actor.role_ids` filter; the test at rbac_e2e.rs:143-150
  explicitly creates an `other_school` to avoid this bug.

---

### FINDING 33

- **id:** CROSSCUT-RBAC-033
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/src/commands.rs:55-61
- **description:** `DeleteRoleCommand` does not check
  `RoleService::can_delete` at construction. The pre-condition
  (non-system, no user bindings) is delegated to the (non-
  existent) command handler. The struct carries no validation
  method, so any caller can submit a `DeleteRoleCommand` for a
  system role.
- **expected:** `docs/specs/rbac/commands.md:62-65` — "Pre-
  conditions: Role is not of type System. No users are bound to
  the role (the platform domain reports the count)."
- **evidence:** `crates/cross-cutting/rbac/src/commands.rs:55-61`:
  ```rust
  pub struct DeleteRoleCommand {
      pub tenant: TenantContext,
      pub role_id: RoleId,
  }
  ```
  No `validate(&self, role: &Role) -> Result<()>` method, no
  reference to `RoleService::can_delete`. The validation helper
  exists in `services.rs:312-320` but is only invoked from
  tests, never from the command.

---

### FINDING 34

- **id:** CROSSCUT-RBAC-034
- **area:** cross-cutting
- **severity:** Medium
- **location:** crates/cross-cutting/rbac/tests/rbac_e2e.rs:1-566
- **description:** 19 integration tests exist but none of them
  test the actual command-handler path, the event-bus
  subscription, the storage adapter, or the
  `super_admin`-vs-`Capability::all()` invariant
  (Finding 1). The 5 Phase 15 cap-roundtrip tests
  (`tests/auth_caps.rs`, `tests/notify_caps.rs`,
  `tests/payment_caps.rs`, `tests/files_caps.rs`,
  `tests/integrations_caps.rs`) verify the variants exist in
  the enum and parse, but never check that they appear in
  `Capability::all()`.
- **expected:** Tests that exercise command handlers, the event
  bus, the storage adapter, and a `super_admin`-covers-`all
  Capability` invariant that does not iterate the broken
  `all()` (see Finding 4).
- **evidence:** `grep -nE "Command|Handler|EventBus|Storage"
  crates/cross-cutting/rbac/tests/rbac_e2e.rs` returns no
  matches for any handler invocation. The 5 phase-15 cap tests
  iterate a hard-coded `const *_VARIANTS: &[Capability]` array
  rather than `Capability::all()`.

---

### FINDING 35

- **id:** CROSSCUT-RBAC-035
- **area:** cross-cutting
- **severity:** Medium
- **location:** docs/coverage.toml:395-411
- **description:** The coverage matrix has 2 rbac rows
  (`rbac_roles_aggregate`, `rbac_capabilities_aggregate`). It
  does not represent the spec surface of 8 tables (Finding
  36), 22 commands (Finding 6), 23 events (Finding 7), 8
  services (Finding 8), or 10 repository traits (Finding 9).
  The Phase 2 hand-off says "2 `coverage.toml` rows flipped"
  but the spec folder defines a much larger surface that the
  matrix does not enumerate.
- **expected:** Per-table / per-command / per-event / per-service
  / per-repository coverage rows.
- **evidence:** `grep -nE "rbac_(section|module|menu|override|
  two_factor|permission_assignment)" docs/coverage.toml` returns
  no matches. The only `rbac_*` rows are at lines 396 and 405.

---

### FINDING 36

- **id:** CROSSCUT-RBAC-036
- **area:** cross-cutting
- **severity:** Medium
- **location:** docs/specs/rbac/tables.md:7-16
- **description:** `docs/specs/rbac/tables.md` lists 8 tables:
  `assign_permissions`, `permissions`, `permission_sections`,
  `roles`, `rbac_module_permissions`,
  `rbac_module_permission_assigns`, `rbac_role_permissions`,
  `two_factor_settings`. The codebase implements 4 (the
  3 aggregates + 1 entity from Finding 5); the 4 secondary
  tables (`rbac_module_permissions`,
  `rbac_module_permission_assigns`, `rbac_role_permissions`,
  `two_factor_settings`) have no DDL or repository. The PHASE
  -2-HANDOFF.md:296-299 acknowledges the deferral ("Do NOT
  add the 5 secondary RBAC aggregates... Phase 2's `docs/
  build-plan.md` § 'Phase 3' only requires the academic
  domain") but the spec doc remains the source of truth and
  is non-conformant.
- **expected:** `docs/specs/rbac/tables.md:7-16` — 8 tables.
- **evidence:** `grep -nE "rbac_module_permissions|rbac_role_permis
  sions|two_factor_settings" crates/cross-cutting/rbac/src/`
  returns no matches.

---

### END FINDINGS

Total findings: **36**.
