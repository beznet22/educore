//! # Relay envelope (storage-port `SerializedEnvelope`)
//!
//! The concrete, serialization-ready envelope the
//! `OutboxRelay` reads from the storage-port [`Outbox`] and
//! forwards to the bus-port [`EventBus`].
//!
//! **Architectural note.** The outbox storage-port trait
//! [`educore_storage::outbox::Outbox`] is the *only* place the
//! engine commits an envelope to durable storage; the relay is
//! the *only* place the engine promotes that durable row back
//! to a bus-port [`EventEnvelope`] for fan-out. To break the
//! `educore-events` ↔ `educore-storage` dependency cycle (the
//! storage port depends on `educore-events` for the
//! `EventEnvelope` wire type; the relay depends on the storage
//! port for the `Outbox` trait), this struct lives in
//! `educore-events` and is re-exported from
//! `educore_storage::outbox::SerializedEnvelope` for
//! backward-compat. All 36+ existing construction sites
//! continue to compile through the re-export.
//!
//! The wire format mirrors the bus-port
//! [`EventEnvelope`](crate::envelope::EventEnvelope) field-for-field
//! except `event_type` / `aggregate_type` are owned `String`s
//! (the storage-port row is `DeserializeOwned` — the bus-port
//! envelope holds `&'static str` for compile-time event-type
//! safety).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;

use crate::envelope::EventEnvelope;

/// Custom serde adapter for `bytes::Bytes` that round-trips
/// through `Vec<u8>`. The default `bytes::Bytes` serde impl
/// carries a `T: 'static` bound that prevents the parent
/// `SerializedEnvelope` from implementing `DeserializeOwned`,
/// which is needed by `serde_json::from_value` and any
/// `for<'de> Deserialize<'de>` consumer. This module keeps the
/// Rust type as `bytes::Bytes` (zero-copy, `Arc`-backed) while
/// presenting a `Vec<u8>`-shaped wire form to serde.
mod bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialise the bytes as a sequence of `u8`s. JSON adapters
    /// will emit a base64 string; binary adapters can emit the
    /// raw bytes.
    pub fn serialize<S: Serializer>(value: &bytes::Bytes, ser: S) -> Result<S::Ok, S::Error> {
        value.as_ref().serialize(ser)
    }

    /// Deserialise the bytes from a `Vec<u8>`-shaped wire form
    /// (any sequence of `u8`).
    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<bytes::Bytes, D::Error> {
        let vec = Vec::<u8>::deserialize(de)?;
        Ok(bytes::Bytes::from(vec))
    }
}

/// A concrete, serialization-ready envelope stored in the
/// outbox and consumed by the [`crate::relay::OutboxRelay`].
///
/// `event_type` and `aggregate_type` are owned `String`s so the
/// type implements `DeserializeOwned` (the bus-port
/// [`EventEnvelope`] uses `&'static str` for compile-time
/// safety; the outbox row must round-trip through arbitrary
/// storage adapters without a `&'static` lifetime).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializedEnvelope {
    /// UUIDv7, time-ordered.
    pub event_id: EventId,
    /// Stable dotted string of the form `<domain>.<aggregate>.<verb>`.
    /// `String` (not `&'static str`) so the type is
    /// `DeserializeOwned`.
    pub event_type: String,
    /// Schema version of the payload.
    pub schema_version: u32,
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// Root aggregate id this event describes.
    pub aggregate_id: Uuid,
    /// Aggregate type name (e.g. `"student"`). `String` (not
    /// `&'static str`) so the type is `DeserializeOwned`.
    pub aggregate_type: String,
    /// The user (or `SYSTEM`) that triggered the change.
    pub actor_id: UserId,
    /// Propagated to every event in the same request/workflow.
    pub correlation_id: CorrelationId,
    /// For events caused by another event, the causing event id.
    pub causation_id: Option<EventId>,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
    /// Serialized payload (JSON or MessagePack). The wire format
    /// is a storage-adapter concern. Uses a custom serde
    /// adapter (see `bytes_via_vec`) so the parent type
    /// implements `DeserializeOwned`.
    #[serde(with = "bytes_via_vec")]
    pub payload: bytes::Bytes,
}

impl SerializedEnvelope {
    /// Constructs a `SerializedEnvelope` from the engine's
    /// bus-port [`EventEnvelope`]. The `event_type` and
    /// `aggregate_type` are cloned from `&'static str` /
    /// `String` (depending on the variant) into owned `String`s
    /// (the storage-port's `DeserializeOwned` requirement); the
    /// payload is JSON-encoded as `bytes::Bytes`.
    ///
    /// The `published_at` field on the bus-port envelope is
    /// intentionally dropped: the outbox row is created *before*
    /// the bus accepts the envelope, so the `published_at`
    /// semantics belong to the bus-port record, not the outbox
    /// row.
    #[must_use]
    pub fn from_event_envelope(envelope: &EventEnvelope) -> Self {
        Self {
            event_id: envelope.event_id,
            event_type: envelope.event_type.clone(),
            schema_version: envelope.schema_version,
            school_id: envelope.school_id,
            aggregate_id: envelope.aggregate_id,
            aggregate_type: envelope.aggregate_type.clone(),
            actor_id: envelope.actor_id,
            correlation_id: envelope.correlation_id,
            causation_id: envelope.causation_id,
            occurred_at: envelope.occurred_at,
            payload: bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default()),
        }
    }

    /// Returns the JSON payload as `serde_json::Value`. The
    /// relay calls this when reconstructing a bus-port
    /// [`EventEnvelope`] for fan-out.
    #[must_use]
    pub fn payload_value(&self) -> serde_json::Value {
        serde_json::from_slice(&self.payload).unwrap_or(serde_json::Value::Null)
    }

    /// Returns the bus-port [`EventEnvelope`] for this row.
    /// The relay calls this when draining the outbox; the
    /// resulting envelope is what `EventBus::publish` accepts.
    #[must_use]
    pub fn into_event_envelope(self) -> EventEnvelope {
        let payload = self.payload_value();
        EventEnvelope {
            event_id: self.event_id,
            event_type: self.event_type,
            schema_version: self.schema_version,
            school_id: self.school_id,
            aggregate_id: self.aggregate_id,
            aggregate_type: self.aggregate_type,
            actor_id: self.actor_id,
            correlation_id: self.correlation_id,
            causation_id: self.causation_id,
            occurred_at: self.occurred_at,
            published_at: None,
            payload,
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

    fn sample_event_envelope(school_id: SchoolId) -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: "academic.student.admitted".to_owned(),
            schema_version: 1,
            school_id,
            aggregate_id: g.next_uuid(),
            aggregate_type: "student".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            published_at: None,
            payload: serde_json::json!({"id": "abc"}),
        }
    }

    fn sample_serialized(school_id: SchoolId) -> SerializedEnvelope {
        let g = SystemIdGen;
        SerializedEnvelope {
            event_id: g.next_event_id(),
            event_type: "academic.student.admitted".to_owned(),
            schema_version: 1,
            school_id,
            aggregate_id: g.next_uuid(),
            aggregate_type: "student".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            payload: bytes::Bytes::from_static(b"{}"),
        }
    }

    #[test]
    fn envelope_serde_round_trip() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let env = sample_serialized(school);
        // Round-trip via `to_value` / `from_value`. The custom
        // `bytes_via_vec` adapter on the `payload` field
        // round-trips through `Vec<u8>`, which IS
        // `DeserializeOwned` — so the parent `SerializedEnvelope`
        // is also `DeserializeOwned`, unlike the default
        // `bytes::Bytes` serde impl which carries a `T: 'static`
        // bound.
        let value = serde_json::to_value(&env).unwrap();
        let back: SerializedEnvelope = serde_json::from_value(value).unwrap();
        assert_eq!(env, back);
    }

    #[test]
    fn from_event_envelope_preserves_fields() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let env = sample_event_envelope(school);
        let serialized = SerializedEnvelope::from_event_envelope(&env);
        assert_eq!(serialized.event_id, env.event_id);
        assert_eq!(serialized.event_type, env.event_type);
        assert_eq!(serialized.aggregate_type, env.aggregate_type);
        assert_eq!(serialized.school_id, env.school_id);
        assert_eq!(serialized.payload_value(), env.payload);
        // `published_at` is intentionally dropped by `from_event_envelope`.
        assert!(env.published_at.is_none());
    }

    #[test]
    fn into_event_envelope_round_trips() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let env = sample_event_envelope(school);
        let serialized = SerializedEnvelope::from_event_envelope(&env);
        let back = serialized.into_event_envelope();
        assert_eq!(back.event_id, env.event_id);
        assert_eq!(back.event_type, env.event_type);
        assert_eq!(back.aggregate_type, env.aggregate_type);
        assert_eq!(back.school_id, env.school_id);
        assert_eq!(back.payload, env.payload);
    }
}
