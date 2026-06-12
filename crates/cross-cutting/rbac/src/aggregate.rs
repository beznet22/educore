//! # RBAC aggregates
//!
//! The [`Role`], [`Permission`], and [`PermissionSection`] aggregate
//! roots. The five secondary aggregates (`TwoFactorSetting`,
//! `Override`, `ModulePermission`, `ModulePermissionAssign`,
//! `RolePermission`) land in later phases.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::ids::{PermissionId, PermissionSectionId, RoleId};
use crate::value_objects::{Capability, PermissionType, RoleType};

/// A named bundle of capabilities. A `Role` is what an actor is
/// assigned to gain the union of capabilities of every role they
/// hold.
///
/// Per `docs/specs/rbac/aggregates.md` invariant 1, a `Role` belongs
/// to exactly one `SchoolId`. Invariant 7: a role carries an
/// `is_replicated` flag indicating whether the role is provisioned
/// into sibling schools in a SaaS deployment. The flag is metadata
/// only — the engine does not have a "promote to SaaS" command;
/// replication is a provisioning action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    /// Typed id.
    pub id: RoleId,
    /// Owning school.
    pub school_id: SchoolId,
    /// Display name (validated, 1..=100 chars).
    pub name: String,
    /// System (immutable, engine-seeded) vs Custom (full lifecycle).
    pub role_type: RoleType,
    /// System roles are immutable: the engine refuses to delete
    /// them, and the rename of a system role is gated on
    /// `RbacRoleManage`.
    pub is_system: bool,
    /// True if the role is replicated into sibling schools in a SaaS
    /// deployment. Set by the platform admin during provisioning;
    /// the engine does not auto-replicate on assignment.
    pub is_replicated: bool,
    /// Direct capability grants (denormalised for fast lookups).
    /// The authoritative store is the `AssignPermission` rows; this
    /// set is the cache the [`CapabilityCheck`](crate::services::CapabilityCheck)
    /// service consults.
    pub capabilities: BTreeSet<Capability>,
    /// Optimistic-concurrency counter.
    pub version: Version,
    /// Content hash for conflict resolution.
    pub etag: Etag,
    /// Creation time.
    pub created_at: Timestamp,
    /// Last-mutation time.
    pub updated_at: Timestamp,
    /// User that created the role.
    pub created_by: UserId,
    /// User that last mutated the role.
    pub updated_by: UserId,
    /// Soft-delete flag.
    pub active_status: ActiveStatus,
    /// Last event id that touched this role (for projection / audit).
    pub last_event_id: Option<EventId>,
    /// Correlation id of the request that created this role.
    pub correlation_id: CorrelationId,
}

impl Role {
    /// Returns `true` if the role is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the role is a system role.
    #[must_use]
    pub fn is_system(&self) -> bool {
        self.is_system || self.role_type.is_system()
    }

    /// Returns `true` if the role holds the capability in its
    /// direct-grant cache.
    #[must_use]
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }
}

/// The storage row for a [`Capability`] (with metadata for UI
/// consumption). The capability is the primary key semantically;
/// the database row carries the display name, route, section, and
/// permission type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    /// Typed id.
    pub id: PermissionId,
    /// Owning school.
    pub school_id: SchoolId,
    /// The capability this row describes.
    pub capability: Capability,
    /// Module segment (e.g. `"platform"`, `"rbac"`).
    pub module: String,
    /// Menu / SubMenu / Action.
    pub type_: PermissionType,
    /// Localized display name (i18n key, not the translated text).
    pub lang_name: String,
    /// Section this row belongs to, for UI grouping.
    pub section_id: Option<PermissionSectionId>,
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
}

impl Permission {
    /// Returns `true` if the row is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }
}

/// A UI grouping label for permission categories (e.g. "Student
/// Information", "Fees Collection"). Used to render the permission
/// management screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionSection {
    /// Typed id.
    pub id: PermissionSectionId,
    /// Owning school.
    pub school_id: SchoolId,
    /// Display name (unique within `school_id`).
    pub name: String,
    /// Display ordering hint.
    pub display_order: u32,
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
}

impl PermissionSection {
    /// Returns `true` if the section is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
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

    use educore_core::clock::{IdGenerator, SystemIdGen};

    fn sample_role() -> Role {
        let g = SystemIdGen;
        Role {
            id: RoleId::new(g.next_school_id(), g.next_uuid()),
            school_id: g.next_school_id(),
            name: "Teacher".to_owned(),
            role_type: RoleType::Custom,
            is_system: false,
            is_replicated: false,
            capabilities: BTreeSet::new(),
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
    fn is_replicated_flag_round_trips() {
        let mut role = sample_role();
        assert!(!role.is_replicated);
        role.is_replicated = true;
        assert!(role.is_replicated);
    }

    #[test]
    fn is_system_includes_role_type_system() {
        let mut role = sample_role();
        assert!(!role.is_system());
        role.is_system = true;
        assert!(role.is_system());
        role.is_system = false;
        role.role_type = RoleType::System;
        assert!(role.is_system());
    }

    #[test]
    fn has_capability_checks_cache() {
        let mut role = sample_role();
        assert!(!role.has_capability(Capability::PlatformUserRead));
        role.capabilities.insert(Capability::PlatformUserRead);
        assert!(role.has_capability(Capability::PlatformUserRead));
    }
}
