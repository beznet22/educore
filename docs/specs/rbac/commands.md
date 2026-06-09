# RBAC Domain — Commands

Commands describe intent. They are validated, authorized, and
dispatched to the relevant aggregate. Every command produces zero or
more events that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability. RBAC commands
require elevated capabilities (typically `Rbac.*`); the engine refuses
to execute any RBAC command against a school other than the actor's
home school.

## Role

### CreateRole

```rust
pub struct CreateRoleCommand {
    pub tenant: TenantContext,
    pub name: RoleName,
    pub role_type: RoleType,
    pub is_replicated: bool,
}
```

**Capability:** `Rbac.Role.Create`
**Pre-conditions:**
- `name` is unique within the school.
- `role_type` is allowed for the actor (system roles require
  `Rbac.Role.Manage`).

**Effects:** Creates a `Role` and emits `RoleCreated`. The legacy
`InfixRole` shadow aggregate is removed — `is_replicated` is a
flag on the engine's `Role`.

### UpdateRole

```rust
pub struct UpdateRoleCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub name: Option<RoleName>,
    pub is_replicated: Option<bool>,
}
```

**Capability:** `Rbac.Role.Update`
**Pre-conditions:** Role exists. If `role_type == System`, the
`name` field is immutable.

**Effects:** Emits `RoleUpdated`.

### DeleteRole

```rust
pub struct DeleteRoleCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
}
```

**Capability:** `Rbac.Role.Delete`
**Pre-conditions:** Role is not of type `System`. No users are
bound to the role (the platform domain reports the count).

**Effects:** Removes the `Role`, cascades to all `AssignPermission`
and `RolePermission` rows, and emits `RoleDeleted`.

### CloneRole

```rust
pub struct CloneRoleCommand {
    pub tenant: TenantContext,
    pub source_role_id: RoleId,
    pub new_name: RoleName,
}
```

**Capability:** `Rbac.Role.Create`
**Effects:** Creates a new `Role` of type `Custom` and copies all
`AssignPermission` and `RolePermission` rows from the source.
Emits `RoleCloned`.

## Capability Assignment

### AssignCapability

```rust
pub struct AssignCapabilityCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub capability: Capability,
    pub menu_status: MenuStatus,
    pub saas_schools: Option<BTreeSet<SchoolId>>,
}
```

**Capability:** `Rbac.Capability.Assign`
**Pre-conditions:** Role exists. The capability is a known variant.
If `saas_schools` is supplied, the school is in SaaS mode.

**Effects:** Creates (or updates) an `AssignPermission` row with
`status=Granted`. Emits `CapabilityAssigned`.

### RevokeCapability

```rust
pub struct RevokeCapabilityCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub capability: Capability,
}
```

**Capability:** `Rbac.Capability.Revoke`
**Effects:** Sets the `AssignPermission::status` to `Revoked` (a
deliberate denial) and emits `CapabilityRevoked`. To remove the row
entirely, use `DeletePermissionAssignment`.

### DeletePermissionAssignment

```rust
pub struct DeletePermissionAssignmentCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub capability: Capability,
}
```

**Capability:** `Rbac.Capability.Revoke`
**Effects:** Hard-deletes the `AssignPermission` row. The capability
is no longer granted **or** denied by this role.

### UpdatePermissionAssignment

```rust
pub struct UpdatePermissionAssignmentCommand {
    pub tenant: TenantContext,
    pub role_id: RoleId,
    pub capability: Capability,
    pub menu_status: Option<MenuStatus>,
    pub saas_schools: Option<BTreeSet<SchoolId>>,
}
```

**Capability:** `Rbac.Capability.Assign`
**Effects:** Updates the metadata of an existing assignment without
changing the grant itself. Emits `PermissionAssignmentUpdated`.

## Module Permission

### CreateModulePermission

```rust
pub struct CreateModulePermissionCommand {
    pub tenant: TenantContext,
    pub name: ModuleName,
    pub dashboard_id: DashboardId,
}
```

**Capability:** `Rbac.ModulePermission.Create`
**Effects:** Emits `ModulePermissionCreated`.

### UpdateModulePermission / DeleteModulePermission

Standard CRUD on the `ModulePermission` aggregate.

**Capabilities:** `Rbac.ModulePermission.Update`,
`Rbac.ModulePermission.Delete`.

### AssignModulePermission

```rust
pub struct AssignModulePermissionCommand {
    pub tenant: TenantContext,
    pub module_id: ModulePermissionId,
    pub role_id: RoleId,
}
```

**Capability:** `Rbac.ModulePermission.Assign`
**Effects:** Emits `ModulePermissionAssigned`.

### RevokeModulePermission

```rust
pub struct RevokeModulePermissionCommand {
    pub tenant: TenantContext,
    pub module_id: ModulePermissionId,
    pub role_id: RoleId,
}
```

**Capability:** `Rbac.ModulePermission.Revoke`
**Effects:** Emits `ModulePermissionRevoked`.

## Menu Link

### GrantMenuLink

```rust
pub struct GrantMenuLinkCommand {
    pub tenant: TenantContext,
    pub module_link_id: ModuleLinkId,
    pub role_id: RoleId,
}
```

**Capability:** `Rbac.Role.GrantMenu`
**Effects:** Creates a `RolePermission` row with
`active_status=true` and emits `MenuLinkGranted`.

### RevokeMenuLink

```rust
pub struct RevokeMenuLinkCommand {
    pub tenant: TenantContext,
    pub module_link_id: ModuleLinkId,
    pub role_id: RoleId,
}
```

**Capability:** `Rbac.Role.RevokeMenu`
**Effects:** Sets `RolePermission::active_status=false` (a
deliberate hide) and emits `MenuLinkRevoked`.

## Permission Section

### CreatePermissionSection / UpdatePermissionSection / DeletePermissionSection

Standard CRUD on `PermissionSection`.

**Capabilities:** `Rbac.Section.Create`, `Rbac.Section.Update`,
`Rbac.Section.Delete`.

## Two-Factor

### ConfigureTwoFactor

```rust
pub struct ConfigureTwoFactorCommand {
    pub tenant: TenantContext,
    pub via_sms: bool,
    pub via_email: bool,
    pub for_student: TwoFactorMode,
    pub for_parent: TwoFactorMode,
    pub for_teacher: TwoFactorMode,
    pub for_staff: TwoFactorMode,
    pub for_admin: TwoFactorMode,
    pub expired_time: TwoFactorExpiry,
}
```

**Capability:** `Rbac.TwoFactor.Configure`
**Pre-conditions:** At least one of `via_sms`, `via_email` is true.
`expired_time` is within bounds.

**Effects:** Creates or updates the school's `TwoFactorSetting`
row, updates the audit log, and emits `TwoFactorConfigured`.

### TestTwoFactorDelivery

```rust
pub struct TestTwoFactorDeliveryCommand {
    pub tenant: TenantContext,
    pub channel: TwoFactorChannel,
    pub recipient_user_id: UserId,
}
```

**Capability:** `Rbac.TwoFactor.Configure`
**Effects:** Sends a test OTP to the recipient via the configured
`Notification` port. The test OTP is short-lived and never
authoritative. Emits `TwoFactorDeliveryTested`.

## Permission Override

### SetPermissionOverride

```rust
pub struct SetPermissionOverrideCommand {
    pub tenant: TenantContext,
    pub actor_id: UserId,
    pub capability: Capability,
    pub granted: bool,
    pub reason: OverrideReason,
    pub expires_at: Option<Timestamp>,
}
```

**Capability:** `Rbac.Override.Set`
**Effects:** Creates a `PermissionOverride` row. The override takes
precedence over role grants until the override is removed or
expires. Emits `PermissionOverrideSet`.

### ClearPermissionOverride

```rust
pub struct ClearPermissionOverrideCommand {
    pub tenant: TenantContext,
    pub override_id: PermissionOverrideId,
}
```

**Capability:** `Rbac.Override.Clear`
**Effects:** Removes the override. Emits `PermissionOverrideCleared`.
