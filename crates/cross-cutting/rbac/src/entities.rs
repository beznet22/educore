//! # RBAC entities
//!
//! Junction aggregates loaded and persisted only through their own
//! consistency boundary. The Phase 2 scope is
//! [`AssignPermission`] (the role↔capability M:N grant).
//! The five other RBAC entities (`PermissionOverride`,
//! `ModuleLinkBinding`, `ModulePermissionAssign`, `RolePermission`,
//! `TwoFactorDelivery`) land in later phases.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::ids::{AssignPermissionId, PermissionId, RoleId};
use crate::value_objects::{AssignmentStatus, Capability, MenuStatus};

/// The many-to-many junction between a [`Role`](crate::aggregate::Role)
/// and a [`Permission`](crate::aggregate::Permission) (a permission
/// row is the storage representation of a [`Capability`] with its
/// metadata). Carries per-grant overrides: `status`, `menu_status`,
/// and `saas_schools`.
///
/// Invariant: the pair `(permission_id, role_id)` is unique within
/// `school_id`. A row with `status = Revoked` is a deliberate denial,
/// not an absence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignPermission {
    /// Typed id.
    pub id: AssignPermissionId,
    /// Owning school.
    pub school_id: SchoolId,
    /// Referenced role.
    pub role_id: RoleId,
    /// Referenced permission (storage row for a `Capability`).
    pub permission_id: PermissionId,
    /// The capability this row grants or denies (denormalised from
    /// the `Permission` row for fast lookups).
    pub capability: Capability,
    /// Granted or explicitly revoked.
    pub status: AssignmentStatus,
    /// UI menu visibility. Does not affect authorization.
    pub menu_status: MenuStatus,
    /// SaaS-only: the schools this grant applies to. `None` in
    /// single-tenant mode.
    pub saas_schools: Option<BTreeSet<SchoolId>>,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash for conflict resolution.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// User that created the row.
    pub created_by: UserId,
    /// User that last mutated the row.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id that touched this row.
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that created this row.
    pub correlation_id: CorrelationId,
}

impl AssignPermission {
    /// Returns `true` if the row is a granted (not denied) capability.
    #[must_use]
    pub fn is_granted(&self) -> bool {
        self.status.is_granted() && self.active_status.is_active()
    }

    /// Returns `true` if the row is an active explicit denial.
    #[must_use]
    pub fn is_explicit_denial(&self) -> bool {
        matches!(self.status, AssignmentStatus::Revoked) && self.active_status.is_active()
    }

    /// Returns `true` if the school in question is covered by this
    /// grant. In single-tenant mode (`saas_schools = None`) the row
    /// applies to the owning school only.
    #[must_use]
    pub fn applies_to(&self, school: SchoolId) -> bool {
        match &self.saas_schools {
            None => school == self.school_id,
            Some(set) => set.contains(&school),
        }
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
    use chrono::Utc;
    use educore_core::ids::Identifier;

    use educore_core::clock::{IdGenerator, SystemIdGen};

    fn sample() -> AssignPermission {
        let g = SystemIdGen;
        AssignPermission {
            id: AssignPermissionId::new(g.next_school_id(), g.next_uuid()),
            school_id: g.next_school_id(),
            role_id: RoleId::new(g.next_school_id(), g.next_uuid()),
            permission_id: PermissionId::new(g.next_school_id(), g.next_uuid()),
            capability: Capability::PlatformUserRead,
            status: AssignmentStatus::Granted,
            menu_status: MenuStatus::Visible,
            saas_schools: None,
            version: Version::initial(),
            etag: Etag::new("00000000000000000000000000000001").unwrap(),
            created_at: Timestamp::from_datetime(Utc::now()),
            updated_at: Timestamp::from_datetime(Utc::now()),
            created_by: g.next_user_id(),
            updated_by: g.next_user_id(),
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id: g.next_correlation_id(),
        }
    }

    #[test]
    fn is_granted_and_is_explicit_denial_reflect_status() {
        let mut row = sample();
        assert!(row.is_granted());
        assert!(!row.is_explicit_denial());
        row.status = AssignmentStatus::Revoked;
        assert!(!row.is_granted());
        assert!(row.is_explicit_denial());
    }

    #[test]
    fn applies_to_owner_school_in_single_tenant_mode() {
        let row = sample();
        assert!(row.applies_to(row.school_id));
        let other = SchoolId::from_uuid(uuid::Uuid::now_v7());
        assert!(!row.applies_to(other));
    }

    #[test]
    fn applies_to_listed_schools_in_saas_mode() {
        let mut row = sample();
        let other = SchoolId::from_uuid(uuid::Uuid::now_v7());
        let mut set = BTreeSet::new();
        set.insert(row.school_id);
        set.insert(other);
        row.saas_schools = Some(set);
        assert!(row.applies_to(row.school_id));
        assert!(row.applies_to(other));
        let third = SchoolId::from_uuid(uuid::Uuid::now_v7());
        assert!(!row.applies_to(third));
    }
}
