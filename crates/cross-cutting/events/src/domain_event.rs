//! # `DomainEvent` trait
//!
//! The trait every domain event implements. Provides the metadata
//! that the engine stamps onto the bus-port
//! [`EventEnvelope`](crate::envelope::EventEnvelope) (event_type,
//! schema_version, aggregate_id, aggregate_type, occurred_at) and
//! the `to_value` serialiser for the cross-domain JSON payload.
//!
//! See `docs/specs/events/events.md` (calendar domain, the design
//! reference) and `docs/ports/event-bus.md` for the bus-port
//! contract. The trait shape is Phase 2; the calendar-domain
//! `DomainEvent` impls land in Phase 13 (educore-events-domain).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::envelope::EventEnvelope;

/// The trait every domain event implements.
///
/// The trait is **object-safe** for `dyn DomainEvent` dispatch and
/// also has an `into_envelope` helper for static (generic)
/// construction.
///
/// # Example
///
/// ```ignore
/// use educore_events::{DomainEvent, EventEnvelope};
/// use educore_core::tenant::TenantContext;
///
/// pub struct SchoolCreated {
///     pub school_id: SchoolId,
///     pub name: String,
///     pub occurred_at: Timestamp,
/// }
///
/// impl DomainEvent for SchoolCreated {
///     const EVENT_TYPE: &'static str = "platform.school.created";
///     const SCHEMA_VERSION: u32 = 1;
///     const AGGREGATE_TYPE: &'static str = "school";
///     fn event_id(&self) -> EventId { /* see SchoolCreated::event_id */ self.event_id_field }
///     fn aggregate_id(&self) -> Uuid { self.school_id.as_uuid() }
///     fn school_id(&self) -> SchoolId { self.school_id }
///     fn occurred_at(&self) -> Timestamp { self.occurred_at }
///     fn to_value(&self) -> serde_json::Value { serde_json::to_value(self).ok().map(serde_json::Value::Object).unwrap_or(serde_json::Value::Null) }
/// }
/// ```
pub trait DomainEvent: Send + Sync + 'static {
    /// The stable dotted `event_type` string. Two events with the
    /// same `event_type` are conceptually the same kind of event.
    const EVENT_TYPE: &'static str;

    /// The schema version of this event's payload shape. Bumped
    /// when the payload's serialised form changes in a
    /// backward-incompatible way.
    const SCHEMA_VERSION: u32;

    /// The aggregate type name (e.g. `"school"`).
    const AGGREGATE_TYPE: &'static str;

    /// Returns the canonical event id (UUIDv7).
    fn event_id(&self) -> EventId;

    /// Returns the root aggregate id this event describes.
    fn aggregate_id(&self) -> Uuid;

    /// Returns the tenant anchor.
    fn school_id(&self) -> SchoolId;

    /// Returns the clock time the event occurred.
    fn occurred_at(&self) -> Timestamp;

    /// Serialises the event payload to JSON. The default
    /// implementation uses `serde_json::to_value`; override only if
    /// you need a custom serialiser (e.g. a wire-format-specific
    /// projection).
    ///
    /// On serialisation failure the default returns
    /// [`serde_json::Value::Null`]. Implementors that need to
    /// surface errors should override this method and return
    /// `Result<_, _>` from a domain-specific helper.
    fn to_value(&self) -> serde_json::Value
    where
        Self: Serialize,
    {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    /// Helper: builds the bus-port [`EventEnvelope`] from a
    /// typed event. Populates `actor_id`, `correlation_id`, and
    /// `causation_id` from the [`TenantContext`].
    fn into_envelope(self, ctx: &TenantContext) -> EventEnvelope
    where
        Self: Sized + Serialize,
    {
        let event_id = self.event_id();
        let aggregate_id = self.aggregate_id();
        let school_id = self.school_id();
        let occurred_at = self.occurred_at();
        let payload = self.to_value();
        let event_type = Self::EVENT_TYPE.to_owned();
        let aggregate_type = Self::AGGREGATE_TYPE.to_owned();
        let schema_version = Self::SCHEMA_VERSION;
        EventEnvelope {
            event_id,
            event_type,
            schema_version,
            school_id,
            aggregate_id,
            aggregate_type,
            actor_id: ctx.actor_id,
            correlation_id: ctx.correlation_id,
            causation_id: ctx.causation_id,
            occurred_at,
            published_at: None,
            payload,
        }
    }
}

/// Helper for event types that need a "builder" that mints the
/// `event_id` and `occurred_at` and forwards a `TenantContext` for
/// envelope construction. The recommended pattern is:
///
/// ```ignore
/// let created = SchoolCreated::new(school_id, name, &clock, &ids, &ctx);
/// let envelope = created.into_envelope(&ctx);
/// ```
///
/// This trait is implemented for the Phase 2 sync events and
/// serves as a template for Phase 3+ domain events.
pub trait EventFactory: DomainEvent + Sized {
    /// Mint a new event with a fresh `event_id` and the given
    /// `occurred_at`.
    #[must_use]
    fn mint(occurred_at: Timestamp, event_id: EventId) -> Self;
}

/// A simple wrapper that pairs a `T: DomainEvent` with the
/// `TenantContext` it was emitted under. The `into_envelope` call
/// consumes the pair.
pub struct EmittedEvent<T: DomainEvent + Serialize> {
    /// The typed event.
    pub event: T,
    /// The tenant context.
    pub ctx: TenantContext,
}

impl<T: DomainEvent + Serialize> EmittedEvent<T> {
    /// Constructs the pair.
    #[must_use]
    pub const fn new(event: T, ctx: TenantContext) -> Self {
        Self { event, ctx }
    }

    /// Consumes the pair and returns the bus-port envelope.
    #[must_use]
    pub fn into_envelope(self) -> EventEnvelope {
        self.event.into_envelope(&self.ctx)
    }
}

/// Re-exports of the `DomainEvent` types most consumers reach for.
pub mod prelude {
    pub use crate::domain_event::{DomainEvent, EmittedEvent, EventFactory};
    pub use crate::envelope::EventEnvelope;
}

/// Helper: derive a `correlation_id`-tagged JSON wrapper for a
/// payload. Most consumers don't need this; it exists for the
/// audit / outbox writers that need to stamp the `correlation_id`
/// into the JSON body when no typed event is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPayload {
    /// The serialised event payload.
    pub payload: serde_json::Value,
    /// The correlation id (mirrored from the envelope for the
    /// audit-sink writer).
    pub correlation_id: CorrelationId,
    /// The actor id (mirrored from the envelope for the
    /// audit-sink writer).
    pub actor_id: UserId,
}

impl RawPayload {
    /// Wraps a payload with audit-sink correlation metadata.
    #[must_use]
    pub fn new(
        payload: serde_json::Value,
        correlation_id: CorrelationId,
        actor_id: UserId,
    ) -> Self {
        Self {
            payload,
            correlation_id,
            actor_id,
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::tenant::UserType;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestEvent {
        event_id: EventId,
        school_id: SchoolId,
        aggregate_id: Uuid,
        occurred_at: Timestamp,
        name: String,
    }

    impl DomainEvent for TestEvent {
        const EVENT_TYPE: &'static str = "test.event.created";
        const SCHEMA_VERSION: u32 = 1;
        const AGGREGATE_TYPE: &'static str = "test_event";
        fn event_id(&self) -> EventId {
            self.event_id
        }
        fn aggregate_id(&self) -> Uuid {
            self.aggregate_id
        }
        fn school_id(&self) -> SchoolId {
            self.school_id
        }
        fn occurred_at(&self) -> Timestamp {
            self.occurred_at
        }
    }

    #[test]
    fn into_envelope_populates_metadata_from_context() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let cause = g.next_event_id();
        let ctx = TenantContext::for_user(school, user, corr, UserType::Teacher)
            .builder()
            .causation_id(cause)
            .build();
        let event = TestEvent {
            event_id: g.next_event_id(),
            school_id: school,
            aggregate_id: g.next_uuid(),
            occurred_at: Timestamp::now(),
            name: "hello".to_owned(),
        };
        let env = event.into_envelope(&ctx);
        assert_eq!(env.event_type, "test.event.created");
        assert_eq!(env.schema_version, 1);
        assert_eq!(env.aggregate_type, "test_event");
        assert_eq!(env.school_id, school);
        assert_eq!(env.actor_id, user);
        assert_eq!(env.correlation_id, corr);
        assert_eq!(env.causation_id, Some(cause));
        assert_eq!(env.payload["name"], "hello");
    }

    #[test]
    fn emitted_event_into_envelope_matches() {
        let g = SystemIdGen;
        let ctx = TenantContext::system(g.next_school_id(), g.next_correlation_id());
        let event = TestEvent {
            event_id: g.next_event_id(),
            school_id: ctx.school_id,
            aggregate_id: g.next_uuid(),
            occurred_at: Timestamp::now(),
            name: "x".to_owned(),
        };
        let pair = EmittedEvent::new(event, ctx.clone());
        let env = pair.into_envelope();
        assert_eq!(env.school_id, ctx.school_id);
        assert_eq!(env.actor_id, ctx.actor_id);
        assert_eq!(env.correlation_id, ctx.correlation_id);
    }
}
