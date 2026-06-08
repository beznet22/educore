# RBAC Domain — Services

Domain services encapsulate business logic that does not fit cleanly
in a single aggregate. They are stateless, sync, and pure (no I/O).

## RoleService

```rust
pub struct RoleService;

impl RoleService {
    pub fn effective_capabilities(role: &Role, assignments: &[AssignPermission]) -> BTreeSet<Capability> { ... }
    pub fn is_system(role: &Role) -> bool { ... }
    pub fn can_delete(role: &Role, user_binding_count: u32) -> Result<(), ConflictError> { ... }
    pub fn can_rename(role: &Role, new_name: &RoleName) -> Result<(), ValidationError> { ... }
    pub fn expand_with_inheritance(role: &Role, all_roles: &[Role]) -> BTreeSet<RoleId> { ... }
}
```

`RoleService::effective_capabilities` returns the union of
capabilities assigned to the role. The caller is responsible for
loading the assignments; the service is pure.

## CapabilityCheckService

```rust
pub struct CapabilityCheckService<'a> {
    catalog: &'a CapabilityCatalog,
    cache: &'a CapabilityCache,
}

impl<'a> CapabilityCheckService<'a> {
    pub fn has(&self, actor: &Actor, capability: Capability) -> Result<bool, AuthError> { ... }
    pub fn has_any(&self, actor: &Actor, capabilities: &[Capability]) -> Result<bool, AuthError> { ... }
    pub fn has_all(&self, actor: &Actor, capabilities: &[Capability]) -> Result<bool, AuthError> { ... }
    pub fn explain(&self, actor: &Actor, capability: Capability) -> Result<CapabilityExplanation, AuthError> { ... }
    pub fn invalidate_cache(&self, role_id: RoleId) -> Result<(), InfraError> { ... }
}
```

`CapabilityCheckService` is the only service other domains call. It
is implemented in the engine's `rbac` adapter, holds an in-memory
cache of `(role, capability)` grants, and consults the catalog for
the canonical answer.

`CapabilityExplanation` is a typed value object describing how a
decision was reached:

```rust
pub struct CapabilityExplanation {
    pub capability: Capability,
    pub decision: bool,
    pub role_grants: Vec<RoleId>,
    pub overrides: Vec<PermissionOverrideId>,
    pub system_fallback: bool,
}
```

## TwoFactorService

```rust
pub struct TwoFactorService;

impl TwoFactorService {
    pub fn policy_for_role(setting: &TwoFactorSetting, role: &Role) -> TwoFactorMode { ... }
    pub fn is_required(setting: &TwoFactorSetting, role: &Role) -> bool { ... }
    pub fn select_channel(setting: &TwoFactorSetting, requested: Option<TwoFactorChannel>) -> Result<TwoFactorChannel, ValidationError> { ... }
    pub fn is_expired(issued_at: Timestamp, expiry: TwoFactorExpiry, now: Timestamp) -> bool { ... }
    pub fn remaining_seconds(issued_at: Timestamp, expiry: TwoFactorExpiry, now: Timestamp) -> u32 { ... }
}
```

## PermissionSectionService

```rust
pub struct PermissionSectionService;

impl PermissionSectionService {
    pub fn can_delete(section: &PermissionSection, referencing_permissions: u32) -> Result<(), ConflictError> { ... }
    pub fn reorder(sections: &mut [PermissionSection], new_positions: &BTreeMap<PermissionSectionId, i32>) -> Result<(), ValidationError> { ... }
    pub fn unique_name_in_school(sections: &[PermissionSection], name: &str, school: SchoolId) -> bool { ... }
}
```

## MenuLinkService

```rust
pub struct MenuLinkService;

impl MenuLinkService {
    pub fn effective_links(role: &Role, role_permissions: &[RolePermission]) -> BTreeSet<ModuleLinkId> { ... }
    pub fn hidden_links(role: &Role, role_permissions: &[RolePermission]) -> BTreeSet<ModuleLinkId> { ... }
}
```

## ModulePermissionService

```rust
pub struct ModulePermissionService;

impl ModulePermissionService {
    pub fn effective_cards(role: &Role, assigns: &[ModulePermissionAssign]) -> BTreeSet<ModulePermissionId> { ... }
    pub fn can_delete(permission: &ModulePermission, assignment_count: u32) -> Result<(), ConflictError> { ... }
}
```

## OverrideService

```rust
pub struct OverrideService;

impl OverrideService {
    pub fn effective_for_actor(actor: &Actor, overrides: &[PermissionOverride]) -> BTreeMap<Capability, bool> { ... }
    pub fn is_expired(override_: &PermissionOverride, now: Timestamp) -> bool { ... }
    pub fn purge_expired(overrides: &mut Vec<PermissionOverride>, now: Timestamp) -> Vec<PermissionOverride> { ... }
}
```

## BootstrapService

```rust
pub struct BootstrapService;

impl BootstrapService {
    pub fn seed_role_catalog(catalog: &CapabilityCatalog) -> Vec<Role> { ... }
    pub fn default_two_factor_setting() -> TwoFactorSetting { ... }
    pub fn system_role_for(capability: Capability) -> Option<RoleId> { ... }
}
```

`BootstrapService` is called once per school, at activation time.
It is the only service that may create `RoleType::System` rows.

## Policy: SystemRoleImmutability

```rust
pub struct SystemRoleImmutability;

impl Policy<DeleteRoleCommand> for SystemRoleImmutability {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &DeleteRoleCommand) -> Outcome { ... }
}
```

Rejects `DeleteRoleCommand` when the target role is of type
`System`.

## Policy: SelfRevocationGuard

```rust
pub struct SelfRevocationGuard;

impl Policy<RevokeCapabilityCommand> for SelfRevocationGuard {
    type Outcome = Allow | Deny { reason: &'static str };
    fn check(&self, ctx: &Context, cmd: &RevokeCapabilityCommand) -> Outcome { ... }
}
```

Rejects a revocation that would remove the last remaining
`Rbac.Capability.Revoke` grant from any user in the school.

## Specification: RolesWithCapability

```rust
pub struct RolesWithCapability(pub Capability);

impl Specification<Role> for RolesWithCapability {
    fn is_satisfied_by(&self, r: &Role) -> bool { ... }
}
```

Used by read-model queries that list "which roles hold X?".

## Specification: ActiveRoles

```rust
pub struct ActiveRoles;

impl Specification<Role> for ActiveRoles {
    fn is_satisfied_by(&self, r: &Role) -> bool { ... }
}
```

Composed with `And`, `Or`, `Not` for queries.

## Cross-Domain Coordinator

RBAC does not coordinate with other domains directly. It publishes
events; the platform domain subscribes to keep its in-memory cache
in sync. There is no service-to-service call from RBAC to any
other domain.
