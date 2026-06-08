# RBAC Domain Overview

## Purpose

The RBAC (Role-Based Access Control) domain owns the school's authorization
model: roles, capabilities (the atomic unit of permission), the assignment
of capabilities to roles, and the school's two-factor authentication
policy. It is the gatekeeper through which every other domain's command
must pass.

RBAC is **special** within the engine: while every other domain
*consumes* capabilities for authorization, the RBAC domain *produces and
administers* the catalog of capabilities. RBAC is the only domain whose
mutations are capable of changing what other domains allow.

## Responsibilities

- Defining and managing the catalog of `Capability` values used across
  the engine.
- Defining `Role` aggregates, which group capabilities.
- Assigning capabilities to roles (the permission assignment table).
- Granting, revoking, and overriding menu/visibility flags per
  role/capability pair.
- Defining the dashboard layout per role.
- Defining module-level permissions and module-permission assignments.
- Configuring two-factor authentication per role and per delivery
  channel.
- Recording audit events for every change to the permission system.
- Providing the `CapabilityCheckService` that other domains call.

## Boundaries

The RBAC domain does **not** own:

- User identity, profile, or contact information (see `specs/platform/`).
- Authentication of credentials (passwords, OAuth tokens, JWT signing) —
  these are port concerns.
- Session lifecycle. RBAC answers "may this actor do X?" not "who is the
  actor?".
- Multi-school SaaS license/entitlement enforcement (see `specs/platform/`
  and `specs/operations/`).
- UI rendering of menus, sidebars, or dashboards (those are consumer
  concerns, but their **configuration** is owned here).

## Dependencies

- `smscore-core` — error types, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-events` — domain event publishing.

## Domain Invariants

1. A `Capability` is a typed enum value with a stable string form. The
   engine rejects unknown capability strings at parse time.
2. A `Role` belongs to exactly one `SchoolId`.
3. A `Role` of `RoleType::System` is seeded by the engine and cannot be
   deleted, only renamed within its school. `RoleType::Custom` roles
   are full user lifecycle (create, update, delete).
4. A `Role` may have zero or more `RolePermission` entries; absence
   means no access.
5. A `Capability` may be assigned to many `Role`s and a `Role` may
   hold many `Capability`s (many-to-many).
6. The capability "Role.Assign" is required to create or modify any
   role; this rule is enforced by RBAC on itself.
7. A `ModulePermission` belongs to one `SchoolId` and groups
   dashboard-level capabilities (e.g. "view attendance dashboard").
8. A `TwoFactorSetting` row exists at most once per `SchoolId`.
9. `TwoFactorSetting::expired_time` is in seconds and is non-negative.
10. The default `TwoFactorSetting` for a new school is `via_email=true,
    via_sms=false, expired_time=300`.

## Aggregate Roots

| Aggregate                | Root Type                | Purpose                                     |
| ------------------------ | ------------------------ | ------------------------------------------- |
| Role                     | `Role`                   | A named bundle of capabilities              |
| Capability               | `Capability`             | The atomic permission value (typed enum)    |
| PermissionSection        | `PermissionSection`      | UI grouping label for permission categories |
| AssignPermission         | `AssignPermission`       | A capability-to-role grant with overrides   |
| InfixRole                | `InfixRole`              | SaaS-flavored role alternate (is_saas flag) |
| InfixPermissionAssign    | `InfixPermissionAssign`  | SaaS-flavored permission assignment         |
| ModulePermission         | `ModulePermission`       | A named dashboard-level permission group    |
| ModulePermissionAssign   | `ModulePermissionAssign` | A module-permission-to-role grant           |
| RolePermission           | `RolePermission`         | A module-link-to-role grant (menu binding)  |
| TwoFactorSetting         | `TwoFactorSetting`       | 2FA policy for a school                     |

Each aggregate is documented in detail under `docs/specs/rbac/aggregates.md`.

## Cross-Domain Impact

Every other domain calls `CapabilityCheckService::has(actor, capability)`
before executing a command. The RBAC domain therefore indirectly
constrains the whole engine.

When a `Capability` is added to the catalog, no existing role
automatically receives it. Capabilities must be assigned explicitly.

When a `TwoFactorSetting` changes (e.g. from `disabled` to `enabled`),
the platform domain's authentication flow begins to require the second
factor for the affected roles.

## Subscribers

- `platform` subscribes to `CapabilityAssigned` and `CapabilityRevoked`
  to update its in-memory capability cache.
- `platform` subscribes to `TwoFactorConfigured` to update its
  authentication flow.

## Consumers

- Web admin UI (role management, capability editor, 2FA configuration).
- Mobile apps (read-only role/capability introspection for menus).
- AI agents (capability catalog lookup before command invocation).
- Automation systems (capability-driven scheduled jobs).

## Anti-Goals

- RBAC does not implement authentication. It answers authorization
  questions only.
- RBAC does not perform password resets, OAuth flows, or session
  management.
- RBAC does not render sidebars or menus. The data it owns is consumed
  by the consumer to render them.
- RBAC does not store secrets, private keys, or hashed passwords.
