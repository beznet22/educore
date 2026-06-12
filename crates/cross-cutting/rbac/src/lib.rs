//! # educore-rbac
//!
//! Capability-based roles, permission assignments, two-factor
//! configuration.
//!
//! Phase 2 of the Educore engine implements the prompt-named
//! subset: [`Capability`] (typed enum), [`Role`], [`Permission`]
//! aggregates, the [`CapabilityCheck`] port, the
//! [`DefaultRoleCatalog`], and the [`is_replicated`](Role::is_replicated)
//! flag on `Role`. The five secondary RBAC aggregates
//! (`TwoFactorSetting`, `Override`, `ModulePermission`,
//! `ModulePermissionAssign`, `RolePermission`) land in later
//! phases.
//!
//! The crate depends on `educore-core`, `educore-platform`, and
//! `educore-events`. Domain code receives a [`TenantContext`] at
//! the command boundary and calls [`CapabilityCheck::has`] to
//! authorize every state change.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-rbac";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// RBAC aggregate roots: [`Role`], [`Permission`], and
/// [`PermissionSection`].
pub mod aggregate;

/// RBAC command shapes.
pub mod commands;

/// RBAC junction entities (the M:N [`AssignPermission`]).
pub mod entities;

/// RBAC domain errors. The engine has a single `DomainError`; this
/// module adds RBAC-specific helper constructors.
pub mod errors;

/// RBAC domain events.
pub mod events;

/// RBAC typed identifiers.
pub mod ids;

/// RBAC query builders.
pub mod query;

/// RBAC repository ports.
pub mod repository;

/// RBAC services: [`CapabilityCheck`] port, the in-memory
/// implementation, [`RoleService`], and [`DefaultRoleCatalog`].
pub mod services;

/// RBAC value objects: the closed-enum [`Capability`] and its
/// companion types.
pub mod value_objects;

/// Convenience re-exports of the most-used types.
pub mod prelude {
    pub use crate::aggregate::{Permission, PermissionSection, Role};
    pub use crate::commands::{
        AssignCapabilityCommand, CreateRoleCommand, DeleteRoleCommand, RevokeCapabilityCommand,
        UpdateRoleCommand,
    };
    pub use crate::entities::AssignPermission;
    pub use crate::errors::{
        missing_capability, permission_not_found, role_has_bindings, role_name_not_unique,
        role_not_found, self_revocation_violation, system_role_immutable,
        system_role_rename_denied,
    };
    pub use crate::events::{
        CapabilityAssigned, CapabilityRevoked, RoleCreated, RoleDeleted, RoleUpdated,
    };
    pub use crate::ids::{AssignPermissionId, PermissionId, PermissionSectionId, RoleId};
    pub use crate::query::{AssignPermissionQuery, RoleQuery};
    pub use crate::repository::{
        AssignPermissionRepository, PermissionRepository, PermissionSectionRepository,
        RoleRepository,
    };
    pub use crate::services::{
        CapabilityCheck, CapabilityExplanation, CapabilityOverride, DefaultRoleCatalog,
        InMemoryCapabilityCheck, RoleService,
    };
    pub use crate::value_objects::{
        AssignmentStatus, Capability, CapabilityDomain, MenuStatus, PermissionType, RoleName,
        RoleType, TwoFactorMode,
    };
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;

    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-rbac");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_wires_expected_types() {
        use crate::prelude::*;
        let _: Capability = Capability::RbacRoleCreate;
        let _: RoleType = RoleType::System;
        let _: AssignmentStatus = AssignmentStatus::Granted;
        let _: MenuStatus = MenuStatus::Visible;
        let _: PermissionType = PermissionType::Action;
        let _: CapabilityDomain = CapabilityDomain::Rbac;
    }
}
