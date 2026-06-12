//! # RBAC query builders
//!
//! Phase 2 ships typed query marker structs. The full macro-driven
//! query layer (per `docs/query_layer.md`) lands in a later phase;
//! these stubs establish the type names and the public surface so
//! downstream code can start using them.
//!
//! The actual filtering, ordering, and pagination surface is added
//! alongside the storage adapters in Phase 1 follow-up work.

use educore_core::ids::SchoolId;

use crate::ids::RoleId;
use crate::value_objects::{Capability, RoleType};

/// Typed query builder for [`Role`](crate::aggregate::Role) listings.
///
/// Phase 2 marker only; the fluent filtering surface lands with the
/// macro-driven query layer.
#[derive(Debug, Clone, Default)]
pub struct RoleQuery {
    /// Tenant anchor. Mandatory for every query.
    pub school_id: Option<SchoolId>,
    /// Optional role-type filter.
    pub role_type: Option<RoleType>,
    /// If `true`, only system roles are returned. If `false`, only
    /// custom roles.
    pub system_only: Option<bool>,
    /// If `true`, only replicated roles are returned.
    pub replicated_only: bool,
}

impl RoleQuery {
    /// Creates a new `RoleQuery` scoped to the given school.
    #[must_use]
    pub fn for_school(school: SchoolId) -> Self {
        Self {
            school_id: Some(school),
            role_type: None,
            system_only: None,
            replicated_only: false,
        }
    }

    /// Filters to roles of the given type.
    #[must_use]
    pub fn of_type(mut self, t: RoleType) -> Self {
        self.role_type = Some(t);
        self
    }

    /// Filters to system roles only.
    #[must_use]
    pub fn system(mut self) -> Self {
        self.system_only = Some(true);
        self
    }

    /// Filters to custom roles only.
    #[must_use]
    pub fn custom(mut self) -> Self {
        self.system_only = Some(false);
        self
    }

    /// Filters to replicated roles only.
    #[must_use]
    pub fn replicated(mut self) -> Self {
        self.replicated_only = true;
        self
    }

    /// Returns the tenant anchor if set.
    #[must_use]
    pub fn school_id(&self) -> Option<SchoolId> {
        self.school_id
    }
}

/// Typed query builder for [`AssignPermission`](crate::entities::AssignPermission)
/// listings.
#[derive(Debug, Clone, Default)]
pub struct AssignPermissionQuery {
    /// Tenant anchor.
    pub school_id: Option<SchoolId>,
    /// Optional role filter.
    pub role_id: Option<RoleId>,
    /// Optional capability filter.
    pub capability: Option<Capability>,
    /// If `true`, only granted rows. If `false`, only revoked.
    pub granted_only: Option<bool>,
}

impl AssignPermissionQuery {
    /// Creates a new `AssignPermissionQuery` scoped to the given school.
    #[must_use]
    pub fn for_school(school: SchoolId) -> Self {
        Self {
            school_id: Some(school),
            role_id: None,
            capability: None,
            granted_only: None,
        }
    }

    /// Filters to assignments for the given role.
    #[must_use]
    pub fn for_role(mut self, role: RoleId) -> Self {
        self.role_id = Some(role);
        self
    }

    /// Filters to assignments for the given capability.
    #[must_use]
    pub fn for_capability(mut self, cap: Capability) -> Self {
        self.capability = Some(cap);
        self
    }

    /// Filters to granted rows only.
    #[must_use]
    pub fn granted(mut self) -> Self {
        self.granted_only = Some(true);
        self
    }

    /// Returns the tenant anchor if set.
    #[must_use]
    pub fn school_id(&self) -> Option<SchoolId> {
        self.school_id
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
    use educore_core::ids::Identifier;
    use uuid::Uuid;

    #[test]
    fn role_query_chains_filters() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let q = RoleQuery::for_school(school)
            .of_type(RoleType::System)
            .replicated();
        assert_eq!(q.school_id(), Some(school));
        assert_eq!(q.role_type, Some(RoleType::System));
        assert!(q.replicated_only);
    }

    #[test]
    fn assign_permission_query_chains_filters() {
        let school = SchoolId::from_uuid(Uuid::now_v7());
        let role = RoleId::new(school, Uuid::now_v7());
        let q = AssignPermissionQuery::for_school(school)
            .for_role(role)
            .for_capability(Capability::RbacRoleCreate)
            .granted();
        assert_eq!(q.role_id, Some(role));
        assert_eq!(q.capability, Some(Capability::RbacRoleCreate));
        assert_eq!(q.granted_only, Some(true));
    }
}
