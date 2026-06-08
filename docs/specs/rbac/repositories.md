# RBAC Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## RoleRepository

```rust
#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn get(&self, id: RoleId) -> Result<Option<Role>>;
    async fn get_by_name(&self, school: SchoolId, name: &RoleName) -> Result<Option<Role>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<Role>>;
    async fn list_system(&self, school: SchoolId) -> Result<Vec<Role>>;
    async fn list_custom(&self, school: SchoolId) -> Result<Vec<Role>>;
    async fn insert(&self, role: &Role) -> Result<()>;
    async fn update(&self, role: &Role) -> Result<()>;
    async fn delete(&self, id: RoleId) -> Result<()>;
    async fn user_binding_count(&self, id: RoleId) -> Result<u64>;
}
```

## AssignPermissionRepository

```rust
#[async_trait]
pub trait AssignPermissionRepository: Send + Sync {
    async fn get(&self, id: AssignPermissionId) -> Result<Option<AssignPermission>>;
    async fn find(&self, role: RoleId, capability: Capability) -> Result<Option<AssignPermission>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<AssignPermission>>;
    async fn list_for_capability(&self, school: SchoolId, capability: Capability) -> Result<Vec<AssignPermission>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<AssignPermission>>;
    async fn insert(&self, a: &AssignPermission) -> Result<()>;
    async fn update(&self, a: &AssignPermission) -> Result<()>;
    async fn delete(&self, id: AssignPermissionId) -> Result<()>;
    async fn delete_for_role(&self, role: RoleId) -> Result<u64>;
}
```

## PermissionRepository (the storage row for a `Capability`)

```rust
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn get(&self, id: PermissionSectionId) -> Result<Option<Permission>>;
    async fn find_by_capability(&self, school: SchoolId, capability: Capability) -> Result<Option<Permission>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Permission>>;
    async fn list_for_module(&self, school: SchoolId, module: &str) -> Result<Vec<Permission>>;
    async fn list_for_section(&self, school: SchoolId, section_id: PermissionSectionId) -> Result<Vec<Permission>>;
    async fn insert(&self, p: &Permission) -> Result<()>;
    async fn update(&self, p: &Permission) -> Result<()>;
    async fn delete(&self, id: PermissionSectionId) -> Result<()>;
}
```

## PermissionSectionRepository

```rust
#[async_trait]
pub trait PermissionSectionRepository: Send + Sync {
    async fn get(&self, id: PermissionSectionId) -> Result<Option<PermissionSection>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PermissionSection>>;
    async fn insert(&self, s: &PermissionSection) -> Result<()>;
    async fn update(&self, s: &PermissionSection) -> Result<()>;
    async fn delete(&self, id: PermissionSectionId) -> Result<()>;
    async fn referencing_permissions(&self, id: PermissionSectionId) -> Result<u64>;
}
```

## ModulePermissionRepository

```rust
#[async_trait]
pub trait ModulePermissionRepository: Send + Sync {
    async fn get(&self, id: ModulePermissionId) -> Result<Option<ModulePermission>>;
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<ModulePermission>>;
    async fn list_for_dashboard(&self, school: SchoolId, dashboard_id: DashboardId) -> Result<Vec<ModulePermission>>;
    async fn insert(&self, m: &ModulePermission) -> Result<()>;
    async fn update(&self, m: &ModulePermission) -> Result<()>;
    async fn delete(&self, id: ModulePermissionId) -> Result<()>;
}
```

## ModulePermissionAssignRepository

```rust
#[async_trait]
pub trait ModulePermissionAssignRepository: Send + Sync {
    async fn find(&self, module_id: ModulePermissionId, role_id: RoleId) -> Result<Option<ModulePermissionAssign>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<ModulePermissionAssign>>;
    async fn list_for_module(&self, module_id: ModulePermissionId) -> Result<Vec<ModulePermissionAssign>>;
    async fn insert(&self, a: &ModulePermissionAssign) -> Result<()>;
    async fn delete(&self, module_id: ModulePermissionId, role_id: RoleId) -> Result<()>;
}
```

## RolePermissionRepository

```rust
#[async_trait]
pub trait RolePermissionRepository: Send + Sync {
    async fn find(&self, module_link_id: ModuleLinkId, role_id: RoleId) -> Result<Option<RolePermission>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<RolePermission>>;
    async fn list_for_link(&self, module_link_id: ModuleLinkId) -> Result<Vec<RolePermission>>;
    async fn insert(&self, r: &RolePermission) -> Result<()>;
    async fn update(&self, r: &RolePermission) -> Result<()>;
    async fn delete(&self, module_link_id: ModuleLinkId, role_id: RoleId) -> Result<()>;
}
```

## InfixRoleRepository

```rust
#[async_trait]
pub trait InfixRoleRepository: Send + Sync {
    async fn get(&self, id: InfixRoleId) -> Result<Option<InfixRole>>;
    async fn list_saas(&self) -> Result<Vec<InfixRole>>;
    async fn insert(&self, r: &InfixRole) -> Result<()>;
    async fn update(&self, r: &InfixRole) -> Result<()>;
    async fn delete(&self, id: InfixRoleId) -> Result<()>;
}
```

## InfixPermissionAssignRepository

```rust
#[async_trait]
pub trait InfixPermissionAssignRepository: Send + Sync {
    async fn get(&self, id: InfixPermissionAssignId) -> Result<Option<InfixPermissionAssign>>;
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<InfixPermissionAssign>>;
    async fn insert(&self, a: &InfixPermissionAssign) -> Result<()>;
    async fn update(&self, a: &InfixPermissionAssign) -> Result<()>;
    async fn delete(&self, id: InfixPermissionAssignId) -> Result<()>;
}
```

## TwoFactorSettingRepository

```rust
#[async_trait]
pub trait TwoFactorSettingRepository: Send + Sync {
    async fn get(&self, school: SchoolId) -> Result<Option<TwoFactorSetting>>;
    async fn insert(&self, s: &TwoFactorSetting) -> Result<()>;
    async fn update(&self, s: &TwoFactorSetting) -> Result<()>;
}
```

## PermissionOverrideRepository

```rust
#[async_trait]
pub trait PermissionOverrideRepository: Send + Sync {
    async fn get(&self, id: PermissionOverrideId) -> Result<Option<PermissionOverride>>;
    async fn list_for_actor(&self, actor: UserId) -> Result<Vec<PermissionOverride>>;
    async fn list_for_capability(&self, school: SchoolId, capability: Capability) -> Result<Vec<PermissionOverride>>;
    async fn insert(&self, o: &PermissionOverride) -> Result<()>;
    async fn delete(&self, id: PermissionOverrideId) -> Result<()>;
    async fn purge_expired(&self, now: Timestamp) -> Result<u64>;
}
```

## TwoFactorDeliveryRepository (read-model for audit)

```rust
#[async_trait]
pub trait TwoFactorDeliveryRepository: Send + Sync {
    async fn list_for_user(&self, user: UserId, limit: u32) -> Result<Vec<TwoFactorDelivery>>;
    async fn list_for_school(&self, school: SchoolId, from: Timestamp, to: Timestamp) -> Result<Vec<TwoFactorDelivery>>;
    async fn insert(&self, d: &TwoFactorDelivery) -> Result<()>;
}
```

## Indexes (recommended)

```sql
CREATE INDEX ix_roles_school_id_name ON roles (school_id, name);
CREATE UNIQUE INDEX ux_roles_school_id_name_lower ON roles (school_id, lower(name));
CREATE INDEX ix_roles_school_id_type ON roles (school_id, type);
CREATE INDEX ix_assign_permissions_school_id_role_id ON assign_permissions (school_id, role_id);
CREATE INDEX ix_assign_permissions_school_id_permission_id ON assign_permissions (school_id, permission_id);
CREATE UNIQUE INDEX ux_assign_permissions_school_id_role_permission ON assign_permissions (school_id, role_id, permission_id);
CREATE INDEX ix_permissions_school_id_module ON permissions (school_id, module);
CREATE INDEX ix_permissions_school_id_section ON permissions (school_id, permission_section);
CREATE INDEX ix_permissions_school_id_name ON permissions (school_id, name);
CREATE INDEX ix_permission_sections_school_id_position ON permission_sections (school_id, position);
CREATE INDEX ix_module_permissions_school_id_dashboard ON module_permissions (school_id, dashboard_id);
CREATE UNIQUE INDEX ux_module_permissions_school_id_name ON module_permissions (school_id, name);
CREATE INDEX ix_module_permission_assigns_school_id_role ON module_permission_assigns (school_id, role_id);
CREATE UNIQUE INDEX ux_module_permission_assigns_school_id_module_role ON module_permission_assigns (school_id, module_id, role_id);
CREATE INDEX ix_role_permissions_school_id_role ON role_permissions (school_id, role_id);
CREATE UNIQUE INDEX ux_role_permissions_school_id_module_link_role ON role_permissions (school_id, module_link_id, role_id);
CREATE INDEX ix_infix_roles_school_id_saas ON infix_roles (school_id) WHERE is_saas = 1;
CREATE UNIQUE INDEX ux_two_factor_settings_school_id ON two_factor_settings (school_id);
CREATE INDEX ix_permission_overrides_school_id_actor ON permission_overrides (school_id, actor_id);
CREATE INDEX ix_permission_overrides_school_id_capability ON permission_overrides (school_id, capability);
CREATE INDEX ix_two_factor_deliveries_school_id_issued ON two_factor_deliveries (school_id, issued_at);
```

The `school_id` predicate is mandatory for tenant isolation.
