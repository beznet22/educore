//! # Event envelope
//!
//! The [`EventEnvelope`] struct is the canonical wire format for
//! domain events flowing through the bus. The shape is locked to
//! [`docs/ports/event-bus.md`](../docs/ports/event-bus.md); the
//! fields, types, and order are part of the engine's public
//! contract.
//!
//! Per the bus port: "`payload` is `serde_json::Value` only at the
//! bus boundary. Inside the engine, payloads are typed Rust
//! structs. Adapters serialize to JSON for cross-process
//! transport."
//!
//! The bridge from a typed [`DomainEvent`](crate::domain_event::DomainEvent)
//! to the envelope is the
//! [`DomainEvent::into_envelope`](crate::domain_event::DomainEvent::into_envelope)
//! helper. The reverse bridge — envelope to a typed event — is the
//! consumer's responsibility (call sites know the expected
//! `event_type` and `schema_version` for their projection).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;

/// The canonical event envelope carried by the bus and stored in
/// the outbox / event log.
///
/// The struct is intentionally non-generic: the bus-port contract
/// is "JSON payload for cross-domain transport", so the wire type
/// is `serde_json::Value`. Domain code holds typed event structs
/// internally and serializes them via
/// [`DomainEvent::to_value`](crate::domain_event::DomainEvent::to_value).
///
/// **Stability:** the field set, names, and order are part of the
/// engine's public API. Renames or removals are breaking changes
/// and require an ADR.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// UUIDv7; the canonical primary key of the event.
    pub event_id: EventId,
    /// Stable dotted string of the form
    /// `<domain>.<aggregate>.<verb>` (e.g. `"platform.school.created"`).
    /// Owned `String` so the envelope round-trips through the
    /// outbox-port [`SerializedEnvelope`](crate::relay_envelope::SerializedEnvelope)
    /// without requiring a `&'static` lifetime; in practice the
    /// value is always constructed from a `DomainEvent::EVENT_TYPE`
    /// `const` or the engine's event-type registry, both of which
    /// produce compile-time-stable strings.
    pub event_type: String,
    /// Schema version of `payload`. Producers MUST send a payload
    /// that matches the declared `schema_version`; consumers handle
    /// newer versions or migrate.
    pub schema_version: u32,
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// The root aggregate id this event describes.
    pub aggregate_id: Uuid,
    /// The aggregate type name (e.g. `"school"`). Owned `String`
    /// for the same reason as `event_type` (see above).
    pub aggregate_type: String,
    /// The user (or `SYSTEM_USER_ID`) that triggered the change.
    pub actor_id: UserId,
    /// Propagated to every event in the same request / workflow.
    pub correlation_id: CorrelationId,
    /// For events caused by another event, the causing event id.
    /// `None` for top-level commands.
    pub causation_id: Option<EventId>,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// The time the bus received the event. `None` for envelopes
    /// constructed inside the engine (set by the bus adapter on
    /// `publish`).
    pub published_at: Option<Timestamp>,
    /// Serialised payload. `serde_json::Value` at the bus boundary;
    /// typed Rust structs inside the engine.
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Returns the dot-separated aggregate name (`<domain>.<aggregate>`),
    /// e.g. `"platform.school"` for `aggregate_type = "school"`. The
    /// domain prefix is the segment of `event_type` before the
    /// first `.`. If `event_type` has no `.`, the function
    /// returns just the `aggregate_type` (i.e. no prefix is
    /// prepended).
    #[must_use]
    pub fn aggregate_topic(&self) -> String {
        match self.event_type.split_once('.') {
            Some((domain, _)) if !domain.is_empty() => {
                format!("{domain}.{}", self.aggregate_type)
            }
            _ => self.aggregate_type.to_owned(),
        }
    }

    /// Returns `true` if the envelope's `school_id` matches the
    /// given school. Used by cross-tenant subscription filters.
    #[must_use]
    pub fn is_for_school(&self, school: SchoolId) -> bool {
        self.school_id == school
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

    fn sample() -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: "platform.school.created".to_owned(),
            schema_version: 1,
            school_id: g.next_school_id(),
            aggregate_id: g.next_uuid(),
            aggregate_type: "school".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload: serde_json::json!({"name": "Test School"}),
        }
    }

    #[test]
    fn envelope_payload_serde_round_trip() {
        // `EventEnvelope` carries owned `String`s for `event_type`
        // and `aggregate_type` (so it round-trips through the
        // outbox-port `SerializedEnvelope`), but the rest of the
        // envelope's typed fields (UUIDs, timestamps) still don't
        // justify a `DeserializeOwned` blanket; we only round-trip
        // the payload here.
        let env = sample();
        let bytes = crate::outbox::payload_bytes(&env);
        let back: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, env.payload);
    }

    #[test]
    fn aggregate_topic_uses_event_type_domain() {
        let env = sample();
        assert_eq!(env.aggregate_topic(), "platform.school");
    }

    #[test]
    fn aggregate_topic_falls_back_to_aggregate_type() {
        let mut env = sample();
        env.event_type = "school".to_owned(); // no domain prefix
        assert_eq!(env.aggregate_topic(), "school");
    }

    #[test]
    fn is_for_school_matches() {
        let env = sample();
        assert!(env.is_for_school(env.school_id));
        let other = SystemIdGen.next_school_id();
        assert!(!env.is_for_school(other));
    }
}
