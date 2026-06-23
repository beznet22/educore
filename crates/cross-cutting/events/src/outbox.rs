//! # Outbox payload helpers
//!
//! The bridge from a typed [`EventEnvelope`] to the bytes that
//! get stored in the outbox / event-log `payload` column. The full
//! row-shape conversion (the storage-port
//! [`SerializedEnvelope`](educore_storage::outbox::SerializedEnvelope))
//! lives in `educore-storage` because of a tier-direction
//! constraint: `educore-storage` already depends on `educore-events`,
//! so the `From<&EventEnvelope>` impl on the storage-port type is
//! the natural home for the row-shape conversion.
//!
//! This module provides the payload-only helper that any consumer
//! (audit writer, integration test, custom command dispatcher) can
//! use without depending on `educore-storage`.

use crate::envelope::EventEnvelope;

/// Returns the serialised payload bytes for an envelope. The
/// returned `bytes::Bytes` is the JSON encoding of
/// `EventEnvelope::payload`; the storage adapter writes these
/// bytes verbatim to the outbox / event-log `payload` column.
#[must_use]
pub fn payload_bytes(envelope: &EventEnvelope) -> bytes::Bytes {
    bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default())
}

/// Returns the serialised bytes for the *entire* envelope. Useful
/// for log forwarding and audit-sink writers that need to record
/// the full event.
#[must_use]
pub fn envelope_bytes(envelope: &EventEnvelope) -> bytes::Bytes {
    bytes::Bytes::from(serde_json::to_vec(envelope).unwrap_or_default())
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
    use serde_json::json;

    fn sample() -> EventEnvelope {
        let g = SystemIdGen;
        EventEnvelope {
            event_id: g.next_event_id(),
            event_type: "test.event.created".to_owned(),
            schema_version: 1,
            school_id: g.next_school_id(),
            aggregate_id: g.next_uuid(),
            aggregate_type: "test_event".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: educore_core::value_objects::Timestamp::now(),
            published_at: None,
            payload: json!({"hello": "world"}),
        }
    }

    #[test]
    fn payload_bytes_round_trips_through_json() {
        let env = sample();
        let bytes = payload_bytes(&env);
        let back: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, env.payload);
    }

    #[test]
    fn envelope_bytes_can_serialize_to_value() {
        // `EventEnvelope` has `&'static str` fields, so it does
        // NOT implement `DeserializeOwned`. We only assert the
        // serialise side; consumers that need full round-trip
        // use the `SerializedEnvelope` (which has `String`
        // fields) at the storage-port boundary.
        let env = sample();
        let bytes = envelope_bytes(&env);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(value.get("event_type").is_some());
        assert!(value.get("school_id").is_some());
    }
}
