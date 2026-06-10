# ADR-009: Capability-Based Permissions

## Status

Accepted.

## Context

The school has many roles: SuperAdmin, SchoolAdmin, Teacher,
Student, Parent, Accountant, Librarian, Transport, Hostel.
The number of roles grows with the school: a school may add
roles for "Vice Principal," "Counselor," "Bus Driver,"
"Lunch Monitor," "Sports Coach." The number of users grows
with the enrollment: thousands of parents, hundreds of staff,
thousands of students.

The role-based access control (RBAC) problem is well-known.
The naive approach — assigning permissions to roles, roles
to users — has a hidden complexity:

- A role can have **hundreds** of permissions. An
  accountant's role lists every `Finance.*` capability, but
  a single typo in the assignment silently denies a
  permission that the accountant needs.
- Roles **leak** across the system. Code that says
  `if user.role == "admin"` is a stringly-typed check that
  is duplicated, refactored inconsistently, and impossible
  to audit.
- A user may hold **multiple roles** in multiple schools.
  Resolving "what is this user allowed to do in school A
  right now?" requires a per-school capability resolution.
- Capabilities are **granted, not inherited.** A role does
  not have a parent role; the model is many-to-many, not
  hierarchical.

The role name is a UI concept (a human label for a bundle
of permissions), not an authorization concept. The
authorization concept is the **capability** — the atomic
permission to perform a specific action on a specific
aggregate.

## Decision

Educore adopts **capability-based permissions** as the
authorization model. Roles exist as a UI and management
convenience; the engine authorizes on capabilities.

Concretely:

1. **A capability is a typed enum value** with a stable
   string form: `<Domain>.<Aggregate>.<Action>`. There is
   exactly one capability for every sanctioned action.
2. **The RBAC domain owns the capability catalog** — the
   type definitions, the parsing, the validation. No
   string is accepted as a capability without going
   through the catalog.
3. **Roles group capabilities.** A role has zero or more
   capabilities. A capability is in zero or more roles.
   Many-to-many.
4. **Roles are anchored to a `SchoolId`.** A role in school
   A is invisible to school B. The `SuperAdmin` role is
   the exception: it is anchored to the bootstrap school
   and is cross-tenant.
5. **A user holds zero or more roles per school.** The
   active `TenantContext` resolves the user's effective
   capabilities for the active school.
6. **The engine authorizes on capability, never on role.**
   Domain code calls
   `rbac.check(actor, Capability::StudentAdmit)`. There is
   no `if actor.role == "admin"` anywhere in the engine.
7. **Capability checks are explicit.** Every command
   declares the capabilities it requires. The dispatcher
   checks them before the aggregate is loaded.
8. **A default role catalog is seeded for every new
   school.** The default is overridable per school.
9. **Capability / role changes are audited.** Every
   assignment, revocation, and catalog mutation is
   recorded in the audit log.
10. **Capability resolution is cached** in the platform
    layer, invalidated by `RoleAssigned` / `RoleRevoked`
    events.

The full schema is in `permission-schema.md` and in
`docs/specs/rbac/`.

## Consequences

### Positive

- **A typo in a capability is a compile error.** The
  typed enum prevents the stringly-typed check.
- **Refactoring a role is safe.** Adding a capability to
  a role adds it for every user holding that role, with
  no code change.
- **Authorization is auditable.** The engine logs every
  capability check (and every denial) to the audit log.
- **AI agents see the same model.** The capability
  catalog is the agent's tool list; the agent must
  request a capability before invoking a command.
- **Cross-tenant authorization is explicit.** Only
  `SuperAdmin` (with `Platform.CrossTenant`) can act
  across schools.
- **A user in school A cannot use a role from school B.**
  The type system and the storage adapter enforce
  this.

### Negative

- **The capability catalog is large.** A domain with 50
  commands and 10 aggregates may have 200+ capabilities.
  The default role catalog is correspondingly large.
- **Capabilities are not hierarchical.** A consumer
  cannot say "give me the entire `Finance.*` namespace";
  they assign each capability explicitly. The default
  role catalog absorbs the boilerplate.
- **Role names in UIs are still useful.** A consumer's
  UI shows "Accountant" to the user, even though
  authorization is on capabilities. The mapping is
  consumer-side.
- **The cache invalidation logic must be correct.** A
  stale cache lets a user retain a revoked capability
  for the cache's TTL. The engine's event-driven
  invalidation closes the window.

### Mitigations

- A `Capability` derive macro generates the enum
  variants from a `capabilities!` declaration.
- A `RoleTemplate` mechanism lets consumers define
  reusable role templates (e.g. "Accountant
  (Tier 1)") that bundle the typical capability set.
- The platform domain subscribes to
  `CapabilityAssigned` / `CapabilityRevoked` events
  and invalidates the in-memory cache immediately.
- The RBAC domain's `CapabilityCheckService` port
  accepts a custom cache implementation; the default
  is an in-process `HashMap` keyed by `(user_id,
  school_id)`.

## Alternatives Considered

### 1. Role-name-based checks

`if user.role == "admin"`. Rejected because it
duplicates authorization across the codebase, makes
refactoring unsafe, and prevents fine-grained
permissions.

### 2. ABAC (attribute-based access control)

Authorization on attributes ("the user is a teacher
in the same class as the student"). Powerful but
hard to audit, hard to UI, and hard to test. We
adopt attribute checks **inside** the capability
check (a capability may be conditional on
attributes), but the **decision** is on the
capability.

### 3. ACLs (access control lists) per resource

A `Student` has a list of users who can read it.
Rejected because the model is per-resource and
does not scale to thousands of students with
overlapping access lists.

### 4. Permissions inheriting from roles in a tree

"Vice Principal" inherits from "Principal."
Rejected because the inheritance model is
fragile (a rename of the parent breaks the
child) and because the school's authorization
model is not a tree.

### 5. ReBAC (relationship-based access control)

A capability is granted by relationship ("a
guardian can read their child's marks").
Powerful but complex. We model relationships
explicitly (`Guardian` aggregates link to
`Student` aggregates), and the engine's
policies enforce the relationship at the
aggregate level. The capability check is the
front door; the relationship check is inside.
