//! # RBAC repository ports
//!
//! Phase 2 ships three repository traits: [`RoleRepository`],
//! [`AssignPermissionRepository`], and [`PermissionRepository`].
//! Adapters (PostgreSQL / MySQL / SQLite) implement these.
//!
//! Per the engine rule "ports in `cross-cutting`/`domains`/infra;
//! implementations in `adapters`", the RBAC domain only defines the
//! traits. Adapter code lives elsewhere.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

use crate::aggregate::{Permission, PermissionSection, Role};
use crate::entities::AssignPermission;
use crate::ids::{AssignPermissionId, PermissionId, PermissionSectionId, RoleId};
use crate::value_objects::Capability;

/// Storage port for [`Role`] aggregates.
#[async_trait]
pub trait RoleRepository: Send + Sync {
    /// Loads a role by id. Returns `None` if the row is missing.
    async fn get(&self, id: RoleId) -> Result<Option<Role>>;

    /// Looks up a role by `(school_id, name)`. Returns `None` if no
    /// such role exists.
    async fn get_by_name(&self, school: SchoolId, name: &str) -> Result<Option<Role>>;

    /// Lists every role in the school (active and retired).
    async fn list(&self, school: SchoolId) -> Result<Vec<Role>>;

    /// Lists system roles in the school.
    async fn list_system(&self, school: SchoolId) -> Result<Vec<Role>>;

    /// Lists custom roles in the school.
    async fn list_custom(&self, school: SchoolId) -> Result<Vec<Role>>;

    /// Lists roles flagged `is_replicated = true`. Used by the SaaS
    /// provisioning tool to enumerate the baseline catalog.
    async fn list_replicated(&self, school: SchoolId) -> Result<Vec<Role>>;

    /// Inserts a new role row. Returns `Err(Conflict)` if a row
    /// with the same `(school_id, name)` already exists.
    async fn insert(&self, role: &Role) -> Result<()>;

    /// Updates an existing role row. The storage adapter MUST check
    /// the optimistic-concurrency version.
    async fn update(&self, role: &Role) -> Result<()>;

    /// Hard-deletes a role row. Cascading to `AssignPermission` rows
    /// is the adapter's responsibility.
    async fn delete(&self, id: RoleId) -> Result<()>;

    /// Returns the number of live user bindings to this role.
    /// Implemented by the platform adapter; the RBAC adapter
    /// delegates.
    async fn user_binding_count(&self, id: RoleId) -> Result<u64>;
}

/// Storage port for [`AssignPermission`] rows.
#[async_trait]
pub trait AssignPermissionRepository: Send + Sync {
    /// Loads an `AssignPermission` by id.
    async fn get(&self, id: AssignPermissionId) -> Result<Option<AssignPermission>>;

    /// Finds the `AssignPermission` for `(role, capability)`.
    async fn find(&self, role: RoleId, capability: Capability) -> Result<Option<AssignPermission>>;

    /// Lists every `AssignPermission` for a role.
    async fn list_for_role(&self, role: RoleId) -> Result<Vec<AssignPermission>>;

    /// Lists every `AssignPermission` for a capability in a school.
    async fn list_for_capability(
        &self,
        school: SchoolId,
        capability: Capability,
    ) -> Result<Vec<AssignPermission>>;

    /// Lists every `AssignPermission` in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<AssignPermission>>;

    /// Inserts a new row. The adapter MUST enforce the
    /// `(permission_id, role_id)` uniqueness constraint within
    /// `school_id`.
    async fn insert(&self, a: &AssignPermission) -> Result<()>;

    /// Updates an existing row. Optimistic-concurrency version is
    /// checked by the adapter.
    async fn update(&self, a: &AssignPermission) -> Result<()>;

    /// Hard-deletes a row.
    async fn delete(&self, id: AssignPermissionId) -> Result<()>;

    /// Hard-deletes every row referencing the role. Used by the
    /// `DeleteRole` command handler. Returns the number of rows
    /// removed.
    async fn delete_for_role(&self, role: RoleId) -> Result<u64>;
}

/// Storage port for [`Permission`] (the storage row for a
/// [`Capability`] with metadata).
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    /// Loads a `Permission` by id.
    async fn get(&self, id: PermissionId) -> Result<Option<Permission>>;

    /// Looks up the `Permission` row for a `(school, capability)` pair.
    async fn find_by_capability(
        &self,
        school: SchoolId,
        capability: Capability,
    ) -> Result<Option<Permission>>;

    /// Lists every `Permission` in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<Permission>>;

    /// Lists every `Permission` for a module (e.g. `"rbac"`, `"platform"`).
    async fn list_for_module(&self, school: SchoolId, module: &str) -> Result<Vec<Permission>>;

    /// Lists every `Permission` in a section.
    async fn list_for_section(
        &self,
        school: SchoolId,
        section_id: PermissionSectionId,
    ) -> Result<Vec<Permission>>;

    /// Inserts a new `Permission` row.
    async fn insert(&self, p: &Permission) -> Result<()>;

    /// Updates an existing row.
    async fn update(&self, p: &Permission) -> Result<()>;

    /// Hard-deletes a row.
    async fn delete(&self, id: PermissionId) -> Result<()>;
}

/// Storage port for [`PermissionSection`] rows.
#[async_trait]
pub trait PermissionSectionRepository: Send + Sync {
    /// Loads a section by id.
    async fn get(&self, id: PermissionSectionId) -> Result<Option<PermissionSection>>;

    /// Lists every section in a school.
    async fn list_for_school(&self, school: SchoolId) -> Result<Vec<PermissionSection>>;

    /// Inserts a new section.
    async fn insert(&self, s: &PermissionSection) -> Result<()>;

    /// Updates an existing section.
    async fn update(&self, s: &PermissionSection) -> Result<()>;

    /// Hard-deletes a section. Returns `Err(Conflict)` if any
    /// `Permission` row still references the section.
    async fn delete(&self, id: PermissionSectionId) -> Result<()>;

    /// Returns the number of `Permission` rows referencing the
    /// section. Used by the `DeletePermissionSection` command to
    /// pre-check the cascade count.
    async fn referencing_permissions(&self, id: PermissionSectionId) -> Result<u64>;
}
