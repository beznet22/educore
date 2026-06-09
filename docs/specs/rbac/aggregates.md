# RBAC Domain — Aggregates

## Role

**Root type:** `Role`
**Identity:** `RoleId(SchoolId, Uuid)`
**Tenant:** `SchoolId`

### Purpose

A named bundle of capabilities. Roles are what an actor (user) is
assigned to gain the union of capabilities of all roles they hold.

### Owned Children

- `RolePermission` — module-link bindings (menu visibility per role).
- `ModulePermissionAssign` — module-permission bindings (dashboard
  visibility per role).

### Invariants

1. A `Role` belongs to exactly one school.
2. A `Role::name` is unique within `(school_id, name)`.
3. A `Role` of `RoleType::System` cannot be deleted; it can be
   renamed only by an actor with `Role.Manage` capability.
4. A `Role` of `RoleType::Custom` may be created, updated, and
   deleted.
5. A `Role` may not be its own ancestor (no cycles in role hierarchy —
   the engine does not yet support role inheritance, but reserves
   the slot).
6. Deleting a `Role` revokes all `AssignPermission` rows that
   reference it.
7. A `Role` carries an `is_replicated` flag indicating whether it is
   available across sibling schools in a SaaS deployment. Replicated
   roles are seeded by the platform admin into each sibling school
   via the `Role.Assign` command; the engine does not have a
   separate "promote to SaaS" command — replication is a provisioning
   action, not a data-state change.

### Commands

- `CreateRole`
- `UpdateRole`
- `DeleteRole`
- `CloneRole`

### Events

- `RoleCreated`
- `RoleUpdated`
- `RoleDeleted`
- `RoleCloned`

### Consistency Boundary

All role mutations are serialized through the `Role` aggregate root.
A role is loaded by id, mutated in memory, validated, and persisted
with its events in a single transaction. Capability assignments are
written as separate `AssignPermission` events but reference the role
by id and are not part of the role's own consistency boundary.

---

## Capability

**Root type:** `Capability` (typed enum)
**Identity:** N/A (capabilities are values, not entities)
**Tenant:** None at the value level; assignments are tenant-scoped

### Purpose

The atomic unit of authorization. A `Capability` is a closed enum
listing every named permission the engine recognizes. The engine
rejects any unknown capability string at parse time.

### Owned Children

None. Capabilities are leaves.

### Invariants

1. The set of `Capability` variants is fixed at compile time. New
   capabilities require a code change, a migration to seed
   `Permission` rows, and a new platform release.
2. A `Capability` has a stable string form
   `<Domain>.<Aggregate>.<Action>` (e.g. `Student.Admit`,
   `Finance.Invoice.Create`).
3. A `Capability` may carry metadata in the `Permission` table
   (route, parent route, lang name, icon, section id) for UI
   consumption, but these do not affect authorization.

### Commands

- `RegisterCapability` (engine-internal; emitted by code generation
  at build time; not user-callable)
- `UpdatePermissionMetadata` (cosmetic; updates the icon, route,
  or language name of a registered capability without changing its
  identity)

### Events

- `CapabilityRegistered` (build-time)
- `PermissionMetadataUpdated`

### Consistency Boundary

The capability catalog is immutable per build. Metadata updates are
allowed but never affect what a capability grants.

---

## PermissionSection

**Root type:** `PermissionSection`
**Identity:** `PermissionSectionId(SchoolId, Uuid)`

### Purpose

A UI grouping label for permission categories (e.g. "Student
Information", "Fees Collection"). Used to render the permission
management screen.

### Invariants

1. A `PermissionSection::name` is unique within `(school_id, name)`.
2. A `PermissionSection::position` controls display ordering.
3. Deleting a `PermissionSection` is rejected if any `Permission`
   row still references it.

### Commands

- `CreatePermissionSection`
- `UpdatePermissionSection`
- `DeletePermissionSection`

### Events

- `PermissionSectionCreated`
- `PermissionSectionUpdated`
- `PermissionSectionDeleted`

---

## AssignPermission

**Root type:** `AssignPermission`
**Identity:** `AssignPermissionId(SchoolId, Uuid)`

### Purpose

The many-to-many junction between a `Role` and a `Permission` (a
permission row is the storage representation of a `Capability` with
its metadata). Carries per-grant overrides: `status` (granted or
revoked), `menu_status` (visible in menus), and `saas_schools`
(comma-separated list of school ids that the grant applies to in
SaaS mode).

### Invariants

1. An `AssignPermission` references exactly one `Permission` and
   one `Role`.
2. The pair `(permission_id, role_id)` is unique within `school_id`.
3. `status` is a boolean: 1 = granted, 0 = revoked. A row with
   `status=0` is a deliberate denial, not an absence.
4. `menu_status` does not affect authorization; it only affects UI
   rendering.
5. Deleting the referenced `Role` cascades. Deleting the
   referenced `Permission` cascades.

### Commands

- `AssignCapability`
- `RevokeCapability`
- `UpdatePermissionAssignment`

### Events

- `CapabilityAssigned`
- `CapabilityRevoked`
- `PermissionAssignmentUpdated`

### Consistency Boundary

Each `AssignPermission` is its own aggregate. The role does not
own its assignment rows; the role and the assignment are mutated
independently. This allows bulk permission edits to scale without
loading the role aggregate.

---

## ModulePermission

**Root type:** `ModulePermission`
**Identity:** `ModulePermissionId(SchoolId, Uuid)`

### Purpose

A named group of dashboard-level permissions (e.g. "View
Attendance Dashboard", "Print Report Card"). Each `ModulePermission`
is rendered as a card on the dashboard and is granted to roles
through `ModulePermissionAssign`.

### Invariants

1. A `ModulePermission::name` is unique within `school_id`.
2. A `ModulePermission` is associated with one `dashboard_id`
   (the dashboard card it represents).
3. A `ModulePermission` cannot be deleted if any
   `ModulePermissionAssign` references it.

### Commands

- `CreateModulePermission`
- `UpdateModulePermission`
- `DeleteModulePermission`

### Events

- `ModulePermissionCreated`
- `ModulePermissionUpdated`
- `ModulePermissionDeleted`

---

## ModulePermissionAssign

**Root type:** `ModulePermissionAssign`
**Identity:** `ModulePermissionAssignId(SchoolId, Uuid)`

### Purpose

The many-to-many junction between `ModulePermission` and `Role`.

### Invariants

1. The pair `(module_id, role_id)` is unique within `school_id`.
2. Deleting the referenced `ModulePermission` or `Role` cascades.

### Commands

- `AssignModulePermission`
- `RevokeModulePermission`

### Events

- `ModulePermissionAssigned`
- `ModulePermissionRevoked`

---

## RolePermission

**Root type:** `RolePermission`
**Identity:** `RolePermissionId(SchoolId, Uuid)`

### Purpose

The binding between a `Role` and a `ModuleLink` (a menu item). This
is the storage representation of "this role can see this menu item".
It is distinct from `AssignPermission`, which binds roles to
`Permission` (action) rows.

### Invariants

1. The pair `(module_link_id, role_id)` is unique within
   `school_id`.
2. `active_status` is a boolean: 1 = visible, 0 = hidden.
3. Deleting the referenced `ModuleLink` or `Role` cascades.

### Commands

- `GrantMenuLink`
- `RevokeMenuLink`

### Events

- `MenuLinkGranted`
- `MenuLinkRevoked`

---

## TwoFactorSetting

**Root type:** `TwoFactorSetting`
**Identity:** `TwoFactorSettingId(SchoolId, Uuid)`

### Purpose

The school's two-factor authentication policy. There is at most one
`TwoFactorSetting` row per `SchoolId`. It controls:

- Which delivery channels are available (`via_sms`, `via_email`).
- Which roles are required to use 2FA (`for_student`, `for_parent`,
  `for_teacher`, `for_staff`, `for_admin` — each set to a status
  value: 1 = required, 2 = optional, 3 = disabled).
- The OTP expiry in seconds (`expired_time`).

### Invariants

1. At most one `TwoFactorSetting` exists per `school_id`.
2. `expired_time >= 0` (typically 60..3600 seconds).
3. Exactly one of `via_sms` or `via_email` must be true (or both),
   never neither.
4. `for_student`, `for_parent`, `for_teacher`, `for_staff`,
   `for_admin` each have a value in `{1, 2, 3}` meaning Required,
   Optional, Disabled.

### Commands

- `ConfigureTwoFactor` (idempotent: creates or updates the school's
  row)

### Events

- `TwoFactorConfigured`

### Consistency Boundary

The `TwoFactorSetting` is loaded, mutated, and persisted as a single
aggregate. Concurrent `ConfigureTwoFactor` commands on the same
school are serialized through an optimistic version check.
