# RBAC Domain — Events

Quick reference of every event the RBAC domain emits. Events are
immutable, append-only records. Every event carries a typed
`EventEnvelope` and is durably persisted to the aggregate's event
log. RBAC events drive platform cache invalidation and audit
recording.

| Event                              | Aggregate            | Subscribers                                                  | Description                                                                              | Durable? | Replicated? | Replayable? |
| ---------------------------------- | -------------------- | ------------------------------------------------------------ | ---------------------------------------------------------------------------------------- | -------- | ----------- | ----------- |
| `RoleCreated`                      | `Role`               | `platform`                                                   | A role was created.                                                                      | yes      | yes         | yes         |
| `RoleUpdated`                      | `Role`               | `platform`                                                   | A role was patched.                                                                      | yes      | yes         | yes         |
| `RoleDeleted`                      | `Role`               | `platform`                                                   | A role was soft-deleted.                                                                 | yes      | yes         | yes         |
| `RoleCloned`                       | `Role`               | `platform`                                                   | A role was cloned.                                                                       | yes      | yes         | yes         |
| `CapabilityRegistered`             | `Capability`         | —                                                            | A capability was registered at build time.                                               | yes      | yes         | yes         |
| `PermissionMetadataUpdated`        | `Capability`         | `platform`                                                   | A capability's metadata was patched.                                                     | yes      | yes         | yes         |
| `CapabilityAssigned`               | `AssignPermission`   | `platform`                                                   | A capability was granted to a role.                                                      | yes      | yes         | yes         |
| `CapabilityRevoked`                | `AssignPermission`   | `platform`                                                   | A capability assignment was revoked (denial) or hard-deleted.                            | yes      | yes         | yes         |
| `PermissionAssignmentUpdated`      | `AssignPermission`   | `platform`                                                   | A capability assignment's metadata was patched.                                          | yes      | yes         | yes         |
| `ModulePermissionCreated`          | `ModulePermission`   | `platform`                                                   | A module permission was created.                                                         | yes      | yes         | yes         |
| `ModulePermissionUpdated`          | `ModulePermission`   | `platform`                                                   | A module permission was patched.                                                         | yes      | yes         | yes         |
| `ModulePermissionDeleted`          | `ModulePermission`   | `platform`                                                   | A module permission was soft-deleted.                                                    | yes      | yes         | yes         |
| `ModulePermissionAssigned`         | `ModulePermission`   | `platform`                                                   | A module permission was assigned to a role.                                              | yes      | yes         | yes         |
| `ModulePermissionRevoked`          | `ModulePermission`   | `platform`                                                   | A module permission was revoked from a role.                                             | yes      | yes         | yes         |
| `MenuLinkGranted`                  | `RolePermission`     | `platform`                                                   | A menu link was granted to a role.                                                       | yes      | yes         | yes         |
| `MenuLinkRevoked`                  | `RolePermission`     | `platform`                                                   | A menu link was revoked (hidden) for a role.                                             | yes      | yes         | yes         |
| `PermissionSectionCreated`         | `PermissionSection`  | `platform`                                                   | A permission section was created.                                                        | yes      | yes         | yes         |
| `PermissionSectionUpdated`         | `PermissionSection`  | `platform`                                                   | A permission section was patched.                                                        | yes      | yes         | yes         |
| `PermissionSectionDeleted`         | `PermissionSection`  | `platform`                                                   | A permission section was soft-deleted.                                                   | yes      | yes         | yes         |
| `TwoFactorConfigured`              | `TwoFactorSetting`   | `platform`                                                   | The two-factor policy was configured.                                                    | yes      | yes         | yes         |
| `TwoFactorDeliveryTested`          | `TwoFactorSetting`   | —                                                            | A two-factor delivery test was performed.                                                | yes      | yes         | yes         |
| `PermissionOverrideSet`            | `PermissionOverride` | `platform`                                                   | A per-user permission override was set.                                                  | yes      | yes         | yes         |
| `PermissionOverrideCleared`        | `PermissionOverride` | `platform`                                                   | A per-user permission override was cleared.                                              | yes      | yes         | yes         |

**See also:** `docs/specs/rbac/events.md` for full Rust struct
definitions, the canonical `EventEnvelope`, and per-event
subscribers.
