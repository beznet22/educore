# RBAC Domain — Business Analysis

## Purpose

The RBAC (Role-Based Access Control) domain owns
the school's authorization model: roles,
capabilities, the assignment of capabilities to
roles, the dashboard layout, the module
permissions, and the two-factor authentication
policy. RBAC is the gatekeeper through which
every other domain's command must pass.

This document describes how roles and permissions
work in real schools, with the edge cases that
real schools hit.

## Key Concepts

- **Role** — a named bundle of capabilities.
  Examples: SuperAdmin, SchoolAdmin, Teacher,
  Student, Parent, Accountant, Librarian,
  Transport, Hostel.
- **RoleType** — `System` (engine-seeded) or
  `Custom` (school-defined).
- **Capability** — the atomic permission value
  (typed enum). Example: `Student.Admit`,
  `Invoice.Generate`.
- **PermissionSection** — a UI grouping label for
  permission categories.
- **AssignPermission** — a capability-to-role
  grant with optional overrides.
- **ModulePermission** — a named dashboard-level
  permission group.
- **ModulePermissionAssign** — a module-
  permission-to-role grant.
- **RolePermission** — a module-link-to-role grant
  (menu binding).
- **TwoFactorSetting** — 2FA policy for a school.

## Real-World Scenarios

### Default Role Seeding

A new school is onboarded:

1. The engine seeds the default `Role`s:
   - `SuperAdmin` (anchored to bootstrap school;
     cross-tenant).
   - `SchoolAdmin` (school-scoped; full
     capability set for the school).
   - `Teacher` (school-scoped; read on academic
     structures, write on homework, marks,
     attendance for assigned classes).
   - `Student` (school-scoped; read on own
     profile, class routine, marks,
     attendance).
   - `Parent` (school-scoped; read on linked
     students' data).
   - `Accountant` (school-scoped; full Finance
     except payroll approval).
   - `Librarian` (school-scoped; full Library).
   - `Transport` (school-scoped; full
     Facilities.Transport).
   - `Hostel` (school-scoped; full
     Facilities.Hostel).
2. Each role is assigned its default capabilities.
3. The engine emits `RoleSeeded` events.

The default catalog is **overrideable per
school**. A school may rename, add, or remove
roles; the engine supports it.

### Role Customization

A school wants to add a "Vice Principal" role:

1. The school admin creates a new role with
   type `Custom`.
2. The admin assigns capabilities (typically
   a subset of SchoolAdmin's capabilities).
3. The engine creates the role and emits
   `RoleCreated`.
4. The admin assigns users to the role.

In real schools, role customization is common.
A school may have:
- "Vice Principal" — most SchoolAdmin
  capabilities, except settings and user
  management.
- "Counselor" — read-only on students,
  write on behavioral notes.
- "Sports Coach" — read on students in their
  sport, write on sports attendance.

### Role Assignment to User

The school admin assigns a role to a user:

1. The admin selects the user and the role.
2. The engine binds the user to the role for
   the active school.
3. The engine emits `UserRoleAssigned`.
4. The platform domain's cache is invalidated;
   the user now has the role's capabilities.

A user may hold multiple roles in the same
school. A user with both Teacher and Parent
roles has the union of capabilities (with
context-dependent role resolution).

### Capability Assignment

The school admin adds a capability to a role:

1. The admin selects the role and the
   capability.
2. The engine creates the `AssignPermission`
   record.
3. The engine emits `CapabilityAssigned`.
4. The platform domain's cache is invalidated;
   all users with the role now have the
   capability.

In real schools, capability assignment is
**granular**. A role may have 100+
capabilities. The engine's UI groups them
by `PermissionSection` for usability.

### Role-Based Dashboard

A school's dashboard is different per role:

- **SchoolAdmin** — sees the school overview,
  recent activity, financial summary, alerts.
- **Teacher** — sees their classes, today's
  schedule, pending homework evaluations.
- **Student** — sees their homework, marks,
  attendance, library issues.
- **Parent** — sees their children's
  attendance, marks, fees, notices.
- **Accountant** — sees today's collection,
  pending fees, bank balances, payroll
  status.

The engine's dashboard is a **projection** over
the role's capabilities. The
`DashboardLayout` per role is configurable.

### Module Permission

A school enables the "Online Course" module:

1. The school admin enables the module.
2. The engine creates the `ModulePermission`
   records.
3. The admin assigns the module permission to
   the relevant role (e.g. Teacher).
4. The sidebar shows the module; the dashboard
   shows the module's widgets.

In real schools, module permissions are
**per-role**. A school may enable the module
for Teachers and Students but not for
Parents.

### Menu Customization

A school wants to hide the "Library" menu
from the Accountant role:

1. The admin selects the "Library" module
   link and the Accountant role.
2. The admin marks it as hidden.
3. The engine updates the `RolePermission`
   record.
4. The sidebar no longer shows "Library" for
   Accountants.

### Two-Factor Authentication

A school enables 2FA for the SchoolAdmin
role:

1. The school admin configures 2FA: enabled,
   via email, expiry 300 seconds.
2. The engine updates the `TwoFactorSetting`.
3. The next time a SchoolAdmin logs in, they
   receive a 2FA code.
4. The admin enters the code; the engine
   grants access.

The engine's 2FA is per-school, per-role.
A school may enable 2FA for SchoolAdmin and
Accountant but not for Teacher (who logs in
less often from more devices).

### Permission Audit

A regulator audits the school. They want
to see the role catalog and the capability
assignments. The engine's
`AuditLog.Read` capability (in the Audit
section) allows the audit. The auditor sees
the role definitions and the capability
history.

### Role Deletion

A school wants to delete a "Bus Monitor"
role that is no longer used:

1. The admin selects the role.
2. The engine checks: are there any users with
   this role? If yes, the deletion is
   rejected.
3. The admin unassigns the users from the
   role.
4. The admin deletes the role.
5. The engine emits `RoleDeleted`.

A system role (e.g. SuperAdmin) cannot be
deleted. It can be renamed (per school) but
not removed.

## Business Rules

1. A `Capability` is a typed enum value with
   a stable string form. The engine rejects
   unknown capability strings at parse time.
2. A `Role` belongs to exactly one `SchoolId`.
3. A `Role` of `RoleType::System` is seeded
   by the engine and cannot be deleted, only
   renamed within its school. `RoleType::
   Custom` roles are full user lifecycle
   (create, update, delete).
4. A `Role` may have zero or more
   `RolePermission` entries; absence means no
   access.
5. A `Capability` may be assigned to many
   `Role`s and a `Role` may hold many
   `Capability`s (many-to-many).
6. The capability "Role.Assign" is required
   to create or modify any role; this rule
   is enforced by RBAC on itself.
7. A `ModulePermission` belongs to one
   `SchoolId` and groups dashboard-level
   capabilities (e.g. "view attendance
   dashboard").
8. A `TwoFactorSetting` row exists at most
   once per `SchoolId`.
9. `TwoFactorSetting::expired_time` is in
   seconds and is non-negative.
10. The default `TwoFactorSetting` for a new
    school is `via_email = true, via_sms =
    false, expired_time = 300`.

## Edge Cases

### User Has Role in School A but Not School B

A teacher works in two schools. The
teacher has the Teacher role in school A
and the Principal role in school B. The
`TenantContext` resolves the active role
based on the active school.

### Role Renamed but Capabilities Preserved

A school renames "Bus Monitor" to
"Transport Assistant." The role's
capabilities are preserved. The
`RoleUpdated` event captures the rename;
the audit log records the change.

### Capability Removed from Role

A school removes `FeesCollect.Collect` from
the Accountant role. All users with the
Accountant role lose the capability
immediately (the cache is invalidated).
The next attempt to collect a fee is
rejected with `Forbidden`.

### 2FA Lockout

A SchoolAdmin loses their 2FA device. They
contact the platform admin. The platform
admin resets the 2FA. The admin can log in
with the new device.

### Role with No Capabilities

A school creates a "Read-Only Auditor"
role with no capabilities. The role is
valid (it denies everything). The
auditor's portal shows an empty menu.

### Multiple SuperAdmins

A school has two SuperAdmins (e.g. the
principal and the vice principal). Both
have full capabilities. The audit log
distinguishes their actions by user id.

### Permission Inheritance

A school asks: "Does the Vice Principal
role inherit from the Principal role?"
The answer is no. The engine's RBAC model
is many-to-many, not hierarchical. The
admin must assign the Vice Principal's
capabilities explicitly.

### Custom Capability

A school wants a custom capability (e.g.
"Canteen.OrderMeal"). The platform admin
adds the capability to the engine's
catalog (via a build-time registration).
The school can then assign it to a role.

The engine supports custom capabilities
within the typed enum (the admin adds a
variant). The capability is then
discoverable in the catalog.

### Permission Section Reordered

A school reorders the permission sections
in the UI (e.g. "Fees" before
"Academics"). The engine's
`PermissionSection::position` is updated;
the UI reorders.

### Dashboard Widget Hidden

A school wants to hide the "Birthday
Today" widget for the Accountant role.
The admin marks the widget as hidden for
the role. The dashboard projection
excludes the widget.

## Notes for SMSengine Implementation

- The **rbac** crate depends on
  `smsengine-core`, `smsengine-platform`, and
  `smsengine-events`. It is the only domain
  whose mutations change what other domains
  allow.
- The RBAC domain provides the
  `CapabilityCheckService` that every other
  domain calls. The service is the
  gatekeeper.
- The RBAC domain's **default role
  catalog** is seeded at school onboarding.
  The seeding is part of the
  `Platform.School.Onboard` workflow.
- The RBAC domain's **capability cache**
  is invalidated by events. The platform
  domain subscribes to `CapabilityAssigned`
  and `CapabilityRevoked`.
- The RBAC domain's **2FA policy** is
  per-school. The platform domain's
  authentication flow reads the policy.
- The RBAC domain's **dashboard
  projection** is per-role. The UI
  consumes the projection.
- The RBAC domain's **audit log**
  captures every change. Regulators can
  reconstruct the role / capability
  history.
- The RBAC domain's **capability
  catalog** is **closed at compile time**.
  Custom capabilities are added at the
  build step; runtime mutation of the
  catalog is not supported (to keep the
  type system authoritative).
- The RBAC domain's **self-referential
  capabilities** (`Role.Create`,
  `Role.Assign`) are enforced before the
  operation runs.
- The RBAC domain's **multi-tenant
  resolution** is structural. A user
  with no role in the active school has
  zero capabilities in that school.
