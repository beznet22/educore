//! # RBAC domain events
//!
//! Phase 2 ships five events: `RoleCreated`, `RoleUpdated`,
//! `RoleDeleted`, `CapabilityAssigned`, `CapabilityRevoked`. All
//! implement [`DomainEvent`](educore_events::DomainEvent) and are
//! serialised through [`EventEnvelope`](educore_events::EventEnvelope)
//! on the bus.
//!
//! The event types are the wire contract for cross-domain
//! subscribers (platform, audit, sync). Renames are breaking changes
//! and require an ADR.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_events::envelope::EventEnvelope;

use crate::ids::RoleId;
use crate::value_objects::{Capability, RoleType};

/// Helper: builds an `EventEnvelope` from the common event metadata.
fn envelope_from<E: DomainEvent + Serialize>(event: E, ctx: &TenantContext) -> EventEnvelope {
    event.into_envelope(ctx)
}

/// A role was created. Emitted by the `CreateRole` command handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleCreated {
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Owning school.
    pub school_id: SchoolId,
    /// Created role.
    pub role_id: RoleId,
    /// Display name.
    pub name: String,
    /// System or custom.
    pub role_type: RoleType,
    /// Provisioned for sibling schools in SaaS deployments.
    pub is_replicated: bool,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Mint-time correlation id (for projections that need it).
    pub correlation_id: CorrelationId,
    /// The user that triggered the command.
    pub actor_id: UserId,
}

impl RoleCreated {
    /// Mints a fresh `RoleCreated` with a v7 event id and the given
    /// `occurred_at`. Use [`DomainEvent::into_envelope`] to wrap it
    /// in a bus envelope.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        school_id: SchoolId,
        role_id: RoleId,
        name: String,
        role_type: RoleType,
        is_replicated: bool,
        occurred_at: Timestamp,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            role_id,
            name,
            role_type,
            is_replicated,
            occurred_at,
            correlation_id,
            actor_id,
        }
    }
}

impl DomainEvent for RoleCreated {
    const EVENT_TYPE: &'static str = "rbac.role.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "rbac_role";

    fn event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A role was updated. `changed_fields` is a list of the field names
/// that actually changed (e.g. `["name", "is_replicated"]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleUpdated {
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Owning school.
    pub school_id: SchoolId,
    /// Updated role.
    pub role_id: RoleId,
    /// Names of the fields that changed.
    pub changed_fields: Vec<String>,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Mint-time correlation id.
    pub correlation_id: CorrelationId,
    /// The user that triggered the command.
    pub actor_id: UserId,
}

impl RoleUpdated {
    /// Mints a fresh `RoleUpdated`.
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        role_id: RoleId,
        changed_fields: Vec<String>,
        occurred_at: Timestamp,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            role_id,
            changed_fields,
            occurred_at,
            correlation_id,
            actor_id,
        }
    }
}

impl DomainEvent for RoleUpdated {
    const EVENT_TYPE: &'static str = "rbac.role.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "rbac_role";

    fn event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A role was deleted (cascades to `AssignPermission` rows).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDeleted {
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Owning school.
    pub school_id: SchoolId,
    /// Deleted role.
    pub role_id: RoleId,
    /// The name the role had at deletion (for audit).
    pub previous_name: String,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Mint-time correlation id.
    pub correlation_id: CorrelationId,
    /// The user that triggered the command.
    pub actor_id: UserId,
}

impl RoleDeleted {
    /// Mints a fresh `RoleDeleted`.
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        role_id: RoleId,
        previous_name: String,
        occurred_at: Timestamp,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            role_id,
            previous_name,
            occurred_at,
            correlation_id,
            actor_id,
        }
    }
}

impl DomainEvent for RoleDeleted {
    const EVENT_TYPE: &'static str = "rbac.role.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "rbac_role";

    fn event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A capability was assigned to a role. Emitted by the
/// `AssignCapability` command handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityAssigned {
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Owning school.
    pub school_id: SchoolId,
    /// Role that received the grant.
    pub role_id: RoleId,
    /// Granted capability.
    pub capability: Capability,
    /// Optional SaaS scope.
    pub saas_schools: Option<BTreeSet<SchoolId>>,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Mint-time correlation id.
    pub correlation_id: CorrelationId,
    /// The user that triggered the command.
    pub actor_id: UserId,
}

impl CapabilityAssigned {
    /// Mints a fresh `CapabilityAssigned`.
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        role_id: RoleId,
        capability: Capability,
        saas_schools: Option<BTreeSet<SchoolId>>,
        occurred_at: Timestamp,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            role_id,
            capability,
            saas_schools,
            occurred_at,
            correlation_id,
            actor_id,
        }
    }
}

impl DomainEvent for CapabilityAssigned {
    const EVENT_TYPE: &'static str = "rbac.capability.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "rbac_role";

    fn event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A capability was revoked from a role.
///
/// `as_denial = true` means the row is preserved with
/// `status = Revoked` (a deliberate denial).
/// `as_denial = false` means the row was hard-deleted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityRevoked {
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Owning school.
    pub school_id: SchoolId,
    /// Role that lost the grant.
    pub role_id: RoleId,
    /// Revoked capability.
    pub capability: Capability,
    /// True if the row is preserved as a deliberate denial; false
    /// if it was hard-deleted.
    pub as_denial: bool,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Mint-time correlation id.
    pub correlation_id: CorrelationId,
    /// The user that triggered the command.
    pub actor_id: UserId,
}

impl CapabilityRevoked {
    /// Mints a fresh `CapabilityRevoked`.
    #[must_use]
    pub fn new(
        school_id: SchoolId,
        role_id: RoleId,
        capability: Capability,
        as_denial: bool,
        occurred_at: Timestamp,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            role_id,
            capability,
            as_denial,
            occurred_at,
            correlation_id,
            actor_id,
        }
    }
}

impl DomainEvent for CapabilityRevoked {
    const EVENT_TYPE: &'static str = "rbac.capability.revoked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "rbac_role";

    fn event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }

    fn aggregate_id(&self) -> Uuid {
        self.role_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Re-exports the five event types and the `DomainEvent` trait for
/// convenience.
pub mod prelude {
    pub use super::{CapabilityAssigned, CapabilityRevoked, RoleCreated, RoleDeleted, RoleUpdated};
    pub use educore_events::domain_event::DomainEvent;
}

/// Helper that wraps the four most common event factories in a
/// single envelope call. Useful for tests.
pub fn into_envelope<E: DomainEvent + Serialize>(event: E, ctx: &TenantContext) -> EventEnvelope {
    envelope_from(event, ctx)
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

    fn ctx() -> (TenantContext, SchoolId) {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        (
            TenantContext::for_user(school, user, corr, UserType::SchoolAdmin),
            school,
        )
    }

    #[test]
    fn role_created_envelope_event_type() {
        let (ctx, school) = ctx();
        let g = SystemIdGen;
        let role_id = RoleId::new(school, g.next_uuid());
        let ev = RoleCreated::new(
            school,
            role_id,
            "Teacher".to_owned(),
            RoleType::Custom,
            false,
            Timestamp::now(),
            ctx.correlation_id,
            ctx.actor_id,
        );
        let env = ev.into_envelope(&ctx);
        assert_eq!(env.event_type, "rbac.role.created");
        assert_eq!(env.aggregate_type, "rbac_role");
        assert_eq!(env.school_id, school);
        assert_eq!(env.actor_id, ctx.actor_id);
        assert_eq!(env.correlation_id, ctx.correlation_id);
        assert_eq!(env.aggregate_id, role_id.as_uuid());
    }

    #[test]
    fn role_updated_changed_fields_round_trip() {
        let (ctx, school) = ctx();
        let g = SystemIdGen;
        let role_id = RoleId::new(school, g.next_uuid());
        let ev = RoleUpdated::new(
            school,
            role_id,
            vec!["name".to_owned(), "is_replicated".to_owned()],
            Timestamp::now(),
            ctx.correlation_id,
            ctx.actor_id,
        );
        let env = ev.into_envelope(&ctx);
        assert_eq!(env.event_type, "rbac.role.updated");
        assert_eq!(
            env.payload["changed_fields"],
            serde_json::json!(["name", "is_replicated"])
        );
    }

    #[test]
    fn capability_revoked_as_denial_default_false() {
        let (ctx, school) = ctx();
        let g = SystemIdGen;
        let role_id = RoleId::new(school, g.next_uuid());
        let ev = CapabilityRevoked::new(
            school,
            role_id,
            Capability::PlatformUserRead,
            false,
            Timestamp::now(),
            ctx.correlation_id,
            ctx.actor_id,
        );
        assert!(!ev.as_denial);
        let env = ev.into_envelope(&ctx);
        assert_eq!(env.event_type, "rbac.capability.revoked");
        assert_eq!(env.payload["as_denial"], serde_json::json!(false));
    }
}
