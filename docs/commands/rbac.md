# RBAC Domain — Commands

Quick reference of every command the RBAC domain exposes. These
commands cover roles, capability assignment, module permission
linking, menu link grants, permission sections, two-factor
configuration, and per-user permission overrides.

The "Events" column lists the events the command emits; consult the
per-domain spec for payload structure.

| Command                          | Capability                          | Description                                                                                       | Events                                       | Idempotent? | Offline? |
| -------------------------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------- | -------------------------------------------- | ----------- | -------- |
| `CreateRole`                     | `Rbac.Role.Create`                  | Create a new role.                                                                                | `RoleCreated`                                | no          | yes      |
| `UpdateRole`                     | `Rbac.Role.Update`                  | Patch a role's mutable fields.                                                                    | `RoleUpdated`                                | no          | yes      |
| `DeleteRole`                     | `Rbac.Role.Delete`                  | Soft-delete a non-system role with no user bindings.                                               | `RoleDeleted`                                | no          | yes      |
| `CloneRole`                      | `Rbac.Role.Create`                  | Clone a role's capabilities into a new role.                                                       | `RoleCloned`                                 | no          | yes      |
| `AssignCapability`               | `Rbac.Capability.Assign`            | Grant a capability to a role.                                                                     | `CapabilityAssigned`                         | yes         | yes      |
| `RevokeCapability`               | `Rbac.Capability.Revoke`            | Mark a capability assignment as denied.                                                            | `CapabilityRevoked` (as denial)              | yes         | yes      |
| `DeletePermissionAssignment`     | `Rbac.Capability.Revoke`            | Hard-delete a capability assignment.                                                               | `CapabilityRevoked` (hard delete)            | yes         | yes      |
| `UpdatePermissionAssignment`     | `Rbac.Capability.Assign`            | Patch the metadata of a capability assignment.                                                     | `PermissionAssignmentUpdated`                | no          | yes      |
| `CreateModulePermission`         | `Rbac.ModulePermission.Create`      | Create a module permission.                                                                       | `ModulePermissionCreated`                    | no          | yes      |
| `UpdateModulePermission`         | `Rbac.ModulePermission.Update`      | Patch a module permission.                                                                        | `ModulePermissionUpdated`                    | no          | yes      |
| `DeleteModulePermission`         | `Rbac.ModulePermission.Delete`      | Soft-delete a module permission.                                                                  | `ModulePermissionDeleted`                    | no          | yes      |
| `AssignModulePermission`         | `Rbac.ModulePermission.Assign`      | Assign a module permission to a role.                                                              | `ModulePermissionAssigned`                   | no          | yes      |
| `RevokeModulePermission`         | `Rbac.ModulePermission.Revoke`      | Revoke a module permission from a role.                                                            | `ModulePermissionRevoked`                    | no          | yes      |
| `GrantMenuLink`                  | `Rbac.Role.GrantMenu`               | Grant a menu link to a role.                                                                      | `MenuLinkGranted`                            | no          | yes      |
| `RevokeMenuLink`                 | `Rbac.Role.RevokeMenu`              | Hide a menu link for a role.                                                                      | `MenuLinkRevoked`                            | no          | yes      |
| `CreatePermissionSection`        | `Rbac.Section.Create`               | Create a permission section.                                                                      | `PermissionSectionCreated`                   | no          | yes      |
| `UpdatePermissionSection`        | `Rbac.Section.Update`               | Patch a permission section.                                                                       | `PermissionSectionUpdated`                   | no          | yes      |
| `DeletePermissionSection`        | `Rbac.Section.Delete`               | Soft-delete a permission section.                                                                  | `PermissionSectionDeleted`                   | no          | yes      |
| `ConfigureTwoFactor`             | `Rbac.TwoFactor.Configure`          | Configure the school's two-factor policy.                                                          | `TwoFactorConfigured`                        | yes         | yes      |
| `TestTwoFactorDelivery`          | `Rbac.TwoFactor.Configure`          | Send a test two-factor delivery.                                                                  | `TwoFactorDeliveryTested`                    | no          | no       |
| `SetPermissionOverride`          | `Rbac.Override.Set`                 | Set a per-user permission override.                                                               | `PermissionOverrideSet`                      | no          | yes      |
| `ClearPermissionOverride`        | `Rbac.Override.Clear`               | Remove a permission override.                                                                     | `PermissionOverrideCleared`                  | no          | yes      |

**See also:** `docs/specs/rbac/commands.md` for full Rust struct
definitions, pre-conditions, effects, and edge-case handling.
