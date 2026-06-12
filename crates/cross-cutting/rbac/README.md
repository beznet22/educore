# educore-rbac

> Capability-based roles, permission assignments, two-factor configuration.

This crate is the **RBAC** (Role-Based Access Control) cross-cutting
crate of the **Educore** engine. It owns the school's authorization
model and is the gatekeeper through which every other domain's
command must pass.

## Scope (Phase 2)

The Phase 2 implementation covers the **prompt-named subset** of
the RBAC spec:

- The closed-enum [`Capability`] (Platform / RBAC; placeholder
  variants for the other domains).
- The [`Role`] aggregate with the `is_replicated` flag.
- The [`Permission`] aggregate (storage row for a `Capability` with
  metadata).
- The [`PermissionSection`] aggregate (UI grouping).
- The [`AssignPermission`] M:N entity (role ↔ capability).
- The [`CapabilityCheck`] port — the engine's capability check
  service.
- An in-memory [`InMemoryCapabilityCheck`] implementation.
- The [`DefaultRoleCatalog`] (ten default role constructors).
- Five events: `RoleCreated`, `RoleUpdated`, `RoleDeleted`,
  `CapabilityAssigned`, `CapabilityRevoked` (all
  `DomainEvent`).
- Five command shapes: `CreateRoleCommand`, `UpdateRoleCommand`,
  `DeleteRoleCommand`, `AssignCapabilityCommand`,
  `RevokeCapabilityCommand`.
- Repository port traits: `RoleRepository`,
  `AssignPermissionRepository`, `PermissionRepository`,
  `PermissionSectionRepository`.

The five secondary RBAC aggregates (`TwoFactorSetting`, `Override`,
`ModulePermission`, `ModulePermissionAssign`, `RolePermission`) land
in later phases.

## Modules

| File | Purpose |
| --- | --- |
| `lib.rs` | Crate root: module declarations, prelude, package metadata. |
| `ids.rs` | Typed `Id<S, T>` wrappers: `RoleId`, `PermissionId`, `PermissionSectionId`, `AssignPermissionId`. |
| `value_objects.rs` | The closed-enum [`Capability`] and the supporting types (`AssignmentStatus`, `MenuStatus`, `PermissionType`, `RoleType`, `TwoFactorMode`, `RoleName`, `CapabilityDomain`). |
| `aggregate.rs` | The aggregate roots: [`Role`], [`Permission`], [`PermissionSection`]. |
| `entities.rs` | The M:N junction: [`AssignPermission`]. |
| `commands.rs` | The five Phase 2 command shapes. |
| `events.rs` | The five Phase 2 events (all `DomainEvent`). |
| `services.rs` | [`CapabilityCheck`] port, [`InMemoryCapabilityCheck`], [`RoleService`], [`DefaultRoleCatalog`]. |
| `repository.rs` | Storage ports: `RoleRepository`, `AssignPermissionRepository`, `PermissionRepository`, `PermissionSectionRepository`. |
| `query.rs` | Typed query markers: `RoleQuery`, `AssignPermissionQuery`. |
| `errors.rs` | RBAC-specific helpers wrapping `DomainError`. |

## Capability String Form

The canonical string form is `<Domain>.<Aggregate>.<Action>`
(e.g. `"Platform.School.Create"`, `"Rbac.Role.Manage"`), with
two-segment exceptions for the cross-cutting
`Settings.Manage` and `Operations.Manage` capabilities and the
`Rbac.Bootstrap` capability.

```rust
use educore_rbac::prelude::*;
use std::str::FromStr;

let cap = Capability::from_str("Rbac.Role.Manage").unwrap();
assert_eq!(cap, Capability::RbacRoleManage);
assert_eq!(cap.domain(), CapabilityDomain::Rbac);
assert_eq!(cap.aggregate(), "Role");
assert_eq!(cap.action(), "Manage");
assert_eq!(cap.as_str(), "Rbac.Role.Manage");
```

## Capability Check

```rust
use educore_rbac::prelude::*;
use educore_rbac::services::InMemoryCapabilityCheck;
use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_core::ids::SchoolId;
use uuid::Uuid;

let g = SystemIdGen;
let school: SchoolId = g.next_school_id();
let role = RoleId::new(school, Uuid::now_v7());

let check = InMemoryCapabilityCheck::new();
check.grant(school, role, Capability::RbacRoleManage);
```

The in-memory check honours the `RbacBootstrap` invariant: any
actor that holds `RbacRoleManage` (or is a system actor) implicitly
holds every `Rbac.*` capability, including `RbacBootstrap`.

## Default Role Catalog

```rust
use educore_rbac::services::DefaultRoleCatalog;

let teacher_caps = DefaultRoleCatalog::teacher();
assert!(teacher_caps.contains(&Capability::AcademicClassRead));
assert!(!teacher_caps.contains(&Capability::FinanceInvoiceCreate));
```

The ten default roles: `super_admin`, `school_admin`, `teacher`,
`student`, `parent`, `accountant`, `receptionist`, `librarian`,
`driver`, `staff`.

## Events

All five events implement `educore_events::domain_event::DomainEvent`
and serialise through `EventEnvelope`. The wire types are
`"rbac.role.created"`, `"rbac.role.updated"`, `"rbac.role.deleted"`,
`"rbac.capability.assigned"`, `"rbac.capability.revoked"`.

## Dependencies

- `educore-core` — errors, identifiers, value objects, clock, ids.
- `educore-events` — `DomainEvent` trait and `EventEnvelope`.
- `async-trait`, `serde`, `thiserror`, `tracing`, `uuid`, `chrono`,
  `indexmap`, `validator`, `futures` (dev).

## License

MIT OR Apache-2.0.
