# Permission Schema

This document is **normative**. It defines the shape of capabilities,
the binding of capabilities to roles, the default role catalog, and
the rules for capability checks. The RBAC domain is the sole producer
and administrator of capabilities; every other domain consumes them.

## 1. Capabilities Are the Atomic Unit

A **capability** is a single, named permission to perform a single
action. It is the only thing an actor is allowed or denied.

- Capabilities are typed enum values with a stable string form.
- Capabilities are **not** role names.
- Capabilities are **not** hierarchical. There is no wildcard
  resolution.
- A capability that is not assigned is denied. There is no implicit
  grant.

The string form of a capability is `<Domain>.<Aggregate>.<Action>`.

## 2. Capability String Format

```text
<Domain>.<Aggregate>.<Action>[.<SubAction>]
```

Rules:

- **Domain** is one of the engine's bounded contexts in upper camel
  case without separators (`Student`, `Finance`, `Hr`, `Rbac`,
  `Platform`, etc.).
- **Aggregate** is the singular noun of the root (`Student`, `Invoice`,
  `Payroll`, `Role`).
- **Action** is a verb in upper camel case (`Admit`, `Update`,
  `Generate`, `Approve`).
- **SubAction** is an optional second-level verb for fine-grained
  actions (`Upload`, `Download`, `Configure`).

Examples:

- `Student.Admit`
- `Student.Update`
- `Student.AssignSection`
- `Student.Withdraw`
- `Student.Graduate`
- `FeesAssign.Create`
- `FeesAssign.Discount.Update`
- `Invoice.Generate`
- `Invoice.Setting.Configure`
- `Payment.Collect`
- `Payment.Reverse`
- `PaymentGateway.Configure`
- `Payroll.Generate`
- `Payroll.Approve`
- `Payroll.Pay`
- `Bank.Reconcile`
- `Expense.Approve`
- `Wallet.Approve`
- `Role.Assign`
- `Role.Create`
- `Capability.Create`
- `User.Create`
- `User.Suspend`
- `School.Onboard`
- `School.Suspend`
- `Platform.CrossTenant`
- `AuditLog.Read`
- `Report.Finance.Read`

## 3. Capability Assignment

A capability is assigned to a **role**, not directly to a user. A user
inherits capabilities from every role they hold.

```text
Capability  ──many-to-many──  Role  ──many-to-many──  User
```

A role MAY hold zero capabilities (an empty role is meaningful: it
denies everything). A capability MAY be assigned to many roles.

The assignment carries optional flags:

- **`is_menu`** — When `true`, the capability drives a sidebar / nav
  entry. UI adapters use this to render menus.
- **`is_admin`** — When `true`, the capability is restricted to admin
  roles. The engine does not enforce this flag; it is a UI hint.
- **`position`** — An integer used to order UI rendering.
- **`module_link_id`** — When present, the capability is bound to a
  specific module link; the engine uses this to compose dashboards.
- **`created_at`, `updated_at`, `created_by`, `updated_by`** — audit.

## 4. Default Role Catalog

The engine ships with a default catalog of roles, seeded for every new
school. Roles are seeded with a **default capability set** that the
consumer may override per school.

| Role          | Purpose                                                                | Default Scope                                                                                                            |
| ------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `SuperAdmin`  | Platform-level administrator across all schools                       | All capabilities across all domains; bypasses per-school checks. Lives in `SchoolId::PLATFORM`.                          |
| `SchoolAdmin` | Top administrator of a single school                                  | All capabilities within the school, EXCEPT platform-wide capabilities (e.g. cross-tenant, school-onboarding).             |
| `Teacher`     | Academic staff: classes, subjects, attendance, homework, lesson plans | Read/update on assigned classes, sections, subjects; homework.create / submit / evaluate; attendance take; marks entry.  |
| `Student`     | A student enrolled in a school                                         | Read on own profile, class routine, homework, marks, attendance (own), library (own), transport (own), notices.           |
| `Parent`      | Guardian of one or more students                                      | Read on linked students' profile, marks, attendance, fees, transport, notices; limited write (e.g. complaint, feedback).|
| `Accountant`  | Finance operator                                                       | All `Finance.*` capabilities EXCEPT payroll approval; bank reconciliation; expense approval up to a limit.                |
| `Librarian`   | Library operator                                                      | All `Library.*` capabilities; read-only on student profiles for book-issue context.                                      |
| `Transport`   | Transport operator                                                     | All `Facilities.Transport.*` capabilities; read on assigned students' profiles.                                          |
| `Hostel`      | Dormitory operator                                                     | All `Facilities.Hostel.*` capabilities.                                                                                  |

The `SuperAdmin` role is a **system role** with `RoleType::System` and
cannot be deleted. All other roles are `RoleType::Custom` and are
fully lifecycle-managed by the school.

## 5. Tenant Binding

Roles are anchored to a `SchoolId`. A `Role` defined in school A is
invisible to school B. The `SuperAdmin` role is anchored to a sentinel
`PLATFORM` school and is the only cross-tenant role.

A user may hold different roles in different schools. The
`TenantContext` carries both the active `SchoolId` and the user's
binding to that school, so the engine resolves the active role for the
session.

## 6. Self-Referential Capabilities

A few capabilities apply to the RBAC domain itself. These are checked
**before** the operation runs:

- `Role.Create`, `Role.Update`, `Role.Delete` — required to mutate
  roles.
- `Capability.Create`, `Capability.Update` — required to mutate the
  capability catalog.
- `Role.Assign` — required to assign a role to a user.
- `PermissionSection.Create` — required to add a new UI section for
  grouping permissions.
- `TwoFactor.Configure` — required to change the school's 2FA policy.

These capabilities are **never** granted to non-admin roles.

## 7. How the Engine Checks Capabilities

Every command follows this gate sequence:

```text
1. Authentication        — the AuthProvider port resolves the actor.
2. Tenant context        — the TenantContext is constructed for the session.
3. Capability check      — CapabilityCheckService::has(actor, capability) is called.
4. Pre-condition         — domain pre-conditions (state, references).
5. Aggregate mutation    — the aggregate is loaded and mutated.
6. Event emission        — events are recorded and persisted.
7. Audit write           — the audit record is written.
```

The capability check is a single function call:

```rust
pub trait CapabilityCheckService: Send + Sync {
    fn has(&self, actor: &Actor, capability: Capability) -> Result<Decision, RbacError>;
    fn has_any(&self, actor: &Actor, capabilities: &[Capability]) -> Result<Decision, RbacError>;
    fn has_all(&self, actor: &Actor, capabilities: &[Capability]) -> Result<Decision, RbacError>;
}
```

A `Decision` is `Allow` or `Deny`, optionally with a `reason` for
audit. The check is **synchronous** and never blocks on I/O after
warming the cache. Cache invalidation is event-driven:

- `Role.Assigned` / `Role.Revoked` invalidate the affected user.
- `CapabilityAssigned` / `CapabilityRevoked` invalidate the affected
  role's users.

## 8. Audit of Capability Changes

Every change to the capability / role graph produces an audit event:

- `rbac.role.created` / `rbac.role.updated` / `rbac.role.deleted`
- `rbac.capability.assigned` / `rbac.capability.revoked`
- `rbac.user.role_assigned` / `rbac.user.role_revoked`
- `rbac.two_factor.configured`

Audit records carry the actor, the target (role, capability, user),
the before and after state, and the change rationale (free text,
optional).

The audit log is **append-only**. There is no `update_audit` or
`delete_audit` operation.

## 9. Capability Groups for UI Rendering

Capabilities are organized into **sections** for UI rendering. A
`PermissionSection` is a UI-only grouping label. It carries:

- `name` (e.g. "Student Information", "Fees Collection",
  "Examination").
- `position` for ordering.
- An optional `icon` for sidebar rendering.

The engine does not require every capability to belong to a section.
Capabilities without a section are listed under "Other."

A typical default section catalog is:

| Section                    | Capabilities                                    |
| -------------------------- | ----------------------------------------------- |
| Student Information        | `Student.*`                                     |
| Academic Setup             | `Class.*`, `Section.*`, `Subject.*`, `AcademicYear.*` |
| Attendance                 | `Attendance.*`                                  |
| Examination                | `Exam.*`, `Mark.*`, `Result.*`, `ReportCard.*`  |
| Fees Collection            | `FeesGroup.*`, `FeesType.*`, `FeesMaster.*`, `Invoice.*`, `Payment.*`, `Bank.*` |
| Accounts                   | `Expense.*`, `Income.*`, `Wallet.*`, `Donor.*`  |
| Payroll                    | `Payroll.*`, `SalaryTemplate.*`                 |
| Human Resource             | `Staff.*`, `Leave.*`, `Department.*`, `Designation.*` |
| Library                    | `Library.*`                                     |
| Transport                  | `Facilities.Transport.*`                        |
| Hostel                     | `Facilities.Hostel.*`                           |
| Inventory                  | `Facilities.Inventory.*`                        |
| Communication              | `Notice.*`, `Complaint.*`, `Chat.*`, `Notification.*` |
| Reports                    | `Report.*`                                      |
| Settings                   | `Settings.*`, `Theme.*`                         |
| Roles & Permissions        | `Role.*`, `Capability.*`, `PermissionSection.*`, `TwoFactor.*` |
| Users                      | `User.*`                                        |
| School Management          | `School.*`                                      |
| Audit                      | `AuditLog.*`                                    |

## 10. Capability Resolution Rules

1. **No implicit grants.** A capability is granted only by an
   `AssignPermission` row in the database.
2. **No inheritance.** Capabilities are not inherited from a parent
   role. Roles do not form a tree.
3. **No role names in code.** Domain code references capabilities
   only by their typed identifier, e.g. `Capability::StudentAdmit`,
   never by a string role name like `"admin"`.
4. **Deny is final.** If a capability is explicitly revoked for a role,
   it is denied for every user holding that role. The engine does not
   attempt to merge or override.
5. **No session override.** A `TenantContext` cannot elevate an
   actor's role. Only a re-binding of the user to a role changes
   their capability set, and that re-binding is itself a capability-
   gated command (`Role.Assign`).

## 11. Multi-Tenant Capability Resolution

The same user may hold different roles in different schools. The
`TenantContext` carries the active `SchoolId` and the user's binding
to that school. The engine resolves:

```text
Effective Capabilities = ⋃
    capabilities_of(role_in_school(active_school_id, user))
```

A user with no role in the active school has **zero** capabilities in
that school. The `SuperAdmin` role is a special case: it is anchored
to `SchoolId::PLATFORM` and is effective across all schools.

## 12. Test-Time Capability Control

Test code MAY install a `CapabilityCheckService` implementation that
returns `Allow` for a fixed set of capabilities, or a fixed actor.
This is the only sanctioned way to bypass capability checks in tests;
production code MUST NOT use this implementation.
