//! # RBAC commands
//!
//! Phase 2 ships five command shapes:
//! [`CreateRoleCommand`], [`UpdateRoleCommand`], [`DeleteRoleCommand`],
//! [`AssignCapabilityCommand`], [`RevokeCapabilityCommand`].
//!
//! Commands are validated, authorized, and dispatched to the relevant
//! aggregate. They carry a [`TenantContext`] (school + actor +
//! correlation) and are rejected if the actor lacks the required
//! capability.
//!
//! Per the engine rule "compile-time safety over strings", the
//! command shapes use typed ids and value objects, not `String`
//! fields.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;

use crate::ids::RoleId;
use crate::value_objects::{Capability, RoleName, RoleType};

/// Create a new role. Requires the `RbacRoleCreate` capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRoleCommand {
    /// The active tenant (school + actor + correlation).
    pub tenant: TenantContext,
    /// Display name (1..=100 chars, unique per school).
    pub name: RoleName,
    /// System or custom. System roles require `RbacRoleManage`.
    pub role_type: RoleType,
    /// Whether the role is replicated into sibling schools.
    pub is_replicated: bool,
}

/// Update an existing role. Requires `RbacRoleUpdate` (or
/// `RbacRoleManage` to rename a system role).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateRoleCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// Target role.
    pub role_id: RoleId,
    /// New name. `None` leaves the existing name in place.
    pub name: Option<RoleName>,
    /// New `is_replicated` value. `None` leaves it unchanged.
    pub is_replicated: Option<bool>,
}

/// Delete a role. Requires `RbacRoleDelete`. Refused for system
/// roles and for roles that have user bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteRoleCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// Target role.
    pub role_id: RoleId,
}

/// Grant a capability to a role. Requires `RbacCapabilityAssign`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignCapabilityCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// Target role.
    pub role_id: RoleId,
    /// Capability to grant.
    pub capability: Capability,
    /// Optional SaaS scope. `None` means single-tenant mode.
    pub saas_schools: Option<BTreeSet<SchoolId>>,
}

/// Revoke a capability from a role. Requires `RbacCapabilityRevoke`.
///
/// `as_denial = true` preserves the `AssignPermission` row with
/// `status = Revoked` (a deliberate denial).
/// `as_denial = false` hard-deletes the row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevokeCapabilityCommand {
    /// The active tenant.
    pub tenant: TenantContext,
    /// Target role.
    pub role_id: RoleId,
    /// Capability to revoke.
    pub capability: Capability,
    /// Whether to preserve the row as an explicit denial.
    pub as_denial: bool,
}

impl CreateRoleCommand {
    /// Returns the school the role will be created in.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

impl UpdateRoleCommand {
    /// Returns the school the role belongs to.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

impl DeleteRoleCommand {
    /// Returns the school the role belongs to.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

impl AssignCapabilityCommand {
    /// Returns the school the role belongs to.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
}

impl RevokeCapabilityCommand {
    /// Returns the school the role belongs to.
    #[must_use]
    pub fn school_id(&self) -> SchoolId {
        self.tenant.school_id
    }
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::UserType;
    use uuid::Uuid;

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::for_user(
            g.next_school_id(),
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::SchoolAdmin,
        )
    }

    #[test]
    fn create_role_carries_school_id() {
        let c = CreateRoleCommand {
            tenant: ctx(),
            name: RoleName::new("Teacher").unwrap(),
            role_type: RoleType::Custom,
            is_replicated: false,
        };
        assert_eq!(c.school_id(), c.tenant.school_id);
    }

    #[test]
    fn update_role_partial_fields_default_to_none() {
        let g = SystemIdGen;
        let c = UpdateRoleCommand {
            tenant: ctx(),
            role_id: RoleId::new(g.next_school_id(), Uuid::now_v7()),
            name: None,
            is_replicated: None,
        };
        assert!(c.name.is_none());
        assert!(c.is_replicated.is_none());
    }

    #[test]
    fn revoke_command_carries_as_denial_flag() {
        let g = SystemIdGen;
        let mut c = RevokeCapabilityCommand {
            tenant: ctx(),
            role_id: RoleId::new(g.next_school_id(), Uuid::now_v7()),
            capability: Capability::RbacRoleCreate,
            as_denial: false,
        };
        assert!(!c.as_denial);
        c.as_denial = true;
        assert!(c.as_denial);
    }
}
