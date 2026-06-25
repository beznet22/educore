//! The `Outbox` sub-port — the transactional outbox pattern.
//!
//! Per `docs/ports/storage.md` § 4 and `docs/schemas/event-schema.md`,
//! every state change is written to the outbox **in the same
//! transaction** as the aggregate mutation. A separate relay reads
//! pending events and publishes them to the event bus. Consumers
//! see at-least-once delivery; the consumer side dedupes by
//! `event_id`.
//!
//! The outbox stores a `SerializedEnvelope` (concrete, no
//! generics) so the storage adapter does not need to know about
//! specific event types. The serialisation format is a
//! storage-adapter concern (JSON by default).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::Timestamp;

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

    /// Serialise the bytes as a `serde_bytes::Bytes`-style byte
    /// sequence. Adapters that produce JSON will emit a base64
    /// string; binary adapters can emit the raw bytes.
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
/// outbox. This is what `Outbox::append` takes; the `payload`
/// bytes are the JSON (or MessagePack) representation of the
/// typed `EventEnvelope::payload` from `educore-events`.
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

/// The `Outbox` sub-port trait. Storage adapters that participate
/// in event-driven workflows implement this; the in-memory
/// `educore-testkit` also implements it.
///
/// ## Tenant isolation (FND-PORT-STORE-003)
///
/// Every method that reads from or writes to the outbox takes
/// `school_id: SchoolId` as the **first** argument. The
/// `school_id` is the tenant anchor for the call and MUST be
/// enforced by every adapter, even when the adapter holds a
/// school-scoped handle (the handle MUST verify that the
/// caller-supplied `school_id` matches its own scope and
/// reject mismatches with
/// [`DomainError::TenantViolation`](educore_core::error::DomainError::TenantViolation)).
/// This is defense-in-depth alongside the application-layer
/// [`TenantGuard`](educore_core::rbac::TenantGuard) — a bug in
/// the application layer that passes the wrong `school_id`
/// will still be caught at the storage port boundary.
#[async_trait]
pub trait Outbox: Send + Sync {
    /// Appends `envelope` to the outbox in the current
    /// transaction. The envelope is durable the moment the
    /// transaction commits. Per
    /// `docs/schemas/event-schema.md` § 1.1, the event is
    /// uniquely identified by `event_id`; duplicates must be
    /// rejected (or, equivalently, stored but never published).
    ///
    /// ## Tenant scope
    ///
    /// `school_id` is the tenant anchor for the append. The
    /// adapter MUST bind `school_id` (not just
    /// `envelope.school_id`) into the persistence layer so
    /// that the row carries the caller-supplied tenant. For
    /// school-scoped handles, the adapter MUST verify that
    /// `school_id` matches the handle's scope.
    ///
    /// # Errors
    /// - `Conflict` if an envelope with the same `event_id` was
    ///   already appended in the same school.
    /// - `TenantViolation` if `school_id` does not match the
    ///   handle's scope (school-scoped adapters only).
    /// - `Infrastructure` for any underlying storage error.
    async fn append(&self, school_id: SchoolId, envelope: SerializedEnvelope) -> Result<()>;

    /// Returns up to `limit` envelopes for `school_id` that
    /// have not yet been marked as published. The order is
    /// FIFO by append time within a school.
    ///
    /// ## School partitioning contract (FND-PORT-STORE-003 / QW-13)
    ///
    /// The outbox is **logically partitioned by `school_id`** —
    /// every row carries the tenant that wrote it, and adapters
    /// MUST return only envelopes belonging to `school_id`.
    /// Cross-tenant reads are a security violation (per
    /// `docs/schemas/tenancy-schema.md` § 2 and
    /// `docs/ports/storage.md` § "Tenant Isolation").
    ///
    /// Adapters that scope the handle to a single `SchoolId`
    /// at construction time (e.g. `PostgresOutbox::new(pool,
    /// school)`) MUST verify that `school_id` matches the
    /// handle's scope and return
    /// [`DomainError::TenantViolation`](educore_core::error::DomainError::TenantViolation)
    /// otherwise. Adapters that don't carry a scoped school
    /// MUST filter the result by `school_id`.
    ///
    /// This method replaces the prior
    /// `pending(limit)` / `pending_for_school(school_id,
    /// limit)` pair — the unified API surfaces the tenant
    /// anchor at the port layer so callers cannot forget it.
    async fn pending(
        &self,
        school_id: SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>>;

    /// Marks the given envelopes as published. Idempotent:
    /// calling twice with the same id is a no-op.
    ///
    /// ## Tenant scope
    ///
    /// `school_id` is the tenant anchor. Adapters that scope
    /// the handle to a single `SchoolId` MUST verify that
    /// `school_id` matches the handle's scope and return
    /// [`DomainError::TenantViolation`](educore_core::error::DomainError::TenantViolation)
    /// otherwise. Adapters without a scoped handle MUST bind
    /// `school_id` into the `UPDATE` predicate so a tenant
    /// cannot mark another tenant's envelopes as published
    /// (which would let one tenant's relay silently drain
    /// another tenant's queue — a denial-of-service
    /// cross-tenant vector).
    async fn mark_published(&self, school_id: SchoolId, ids: &[EventId]) -> Result<()>;

    /// Returns the count of envelopes currently in the outbox
    /// for `school_id` that have not been marked as published.
    /// Used by the relay for back-pressure decisions.
    ///
    /// ## School partitioning contract (QW-13, ADAPTER-PG-013)
    ///
    /// Adapters that scope the handle to a single `SchoolId`
    /// MUST verify that `school_id` matches the handle's scope
    /// and return
    /// [`DomainError::TenantViolation`](educore_core::error::DomainError::TenantViolation)
    /// otherwise. The handle may not be used to read another
    /// tenant's outbox depth — that would leak back-pressure
    /// signals across tenants.
    ///
    /// The default implementation counts via
    /// [`pending`](Self::pending) and checks length, which is
    /// `O(n)` memory for a one-line aggregate. Adapters with
    /// efficient `COUNT(*)` support should override.
    async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
        // Default implementation: count via `pending` and check
        // length. Adapters with efficient `COUNT(*)` support may
        // override.
        let n = self.pending(school_id, u32::MAX).await?.len();
        u64::try_from(n)
            .map_err(|_| DomainError::validation("pending count exceeds u64::MAX"))
    }
}

impl SerializedEnvelope {
    /// Constructs a `SerializedEnvelope` from the engine's bus-port
    /// [`educore_events::envelope::EventEnvelope`]. The `event_type` and
    /// `aggregate_type` are cloned from `&'static str` into
    /// `String` (the storage port's `DeserializeOwned`
    /// requirement); the payload is JSON-encoded as `bytes::Bytes`.
    ///
    /// The `published_at` field on the bus-port envelope is
    /// intentionally dropped: the outbox row is created *before*
    /// the bus accepts the envelope, so the `published_at`
    /// semantics belong to the bus-port record, not the outbox
    /// row.
    #[must_use]
    pub fn from_event_envelope(envelope: &educore_events::envelope::EventEnvelope) -> Self {
        Self {
            event_id: envelope.event_id,
            event_type: envelope.event_type.to_owned(),
            schema_version: envelope.schema_version,
            school_id: envelope.school_id,
            aggregate_id: envelope.aggregate_id,
            aggregate_type: envelope.aggregate_type.to_owned(),
            actor_id: envelope.actor_id,
            correlation_id: envelope.correlation_id,
            causation_id: envelope.causation_id,
            occurred_at: envelope.occurred_at,
            payload: bytes::Bytes::from(serde_json::to_vec(&envelope.payload).unwrap_or_default()),
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

    fn sample_envelope(school_id: SchoolId) -> SerializedEnvelope {
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
        let env = sample_envelope(school);
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
}
