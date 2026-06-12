//! The `EventLog` sub-port — the durable, append-only event log.
//!
//! Per `docs/ports/storage.md` § 7 and
//! `docs/schemas/event-schema.md`, every event the engine emits
//! lands in two places: the **outbox** (transactional, drained by
//! the relay) and the **event log** (durable, queryable by
//! consumers for projections and analytics). The event log is
//! per-school; the outbox is per-school too.
//!
//! The trait is distinct from `Outbox` because the outbox is
//! short-lived and transactional (drained in seconds), while the
//! event log is the long-term history (retained for the school's
//! configured retention window, default 365 days).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};

/// Custom serde adapter for `bytes::Bytes` that round-trips
/// through `Vec<u8>` (see the same module in `outbox.rs` for
/// the rationale).
mod bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(value: &bytes::Bytes, ser: S) -> Result<S::Ok, S::Error> {
        value.as_ref().serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<bytes::Bytes, D::Error> {
        let vec = Vec::<u8>::deserialize(de)?;
        Ok(bytes::Bytes::from(vec))
    }
}

/// A row in the event log. Distinct from the outbox envelope in
/// two ways:
///
/// 1. The event log carries `active_status` so consumers can
///    retire events (e.g. for GDPR erasure) without deleting
///    the row (audit trails must remain).
/// 2. The event log includes `recorded_at` separately from
///    `occurred_at` so the engine can record ingestion latency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogEntry {
    /// The event id; canonical primary key.
    pub event_id: EventId,
    /// The school the event belongs to.
    pub school_id: SchoolId,
    /// Stable dotted string (`<domain>.<aggregate>.<verb>`).
    /// `String` (not `&'static str`) so the type is
    /// `DeserializeOwned`.
    pub event_type: String,
    /// Schema version of the payload.
    pub schema_version: u32,
    /// Root aggregate id.
    pub aggregate_id: Uuid,
    /// Aggregate type name. `String` so the type is
    /// `DeserializeOwned`.
    pub aggregate_type: String,
    /// The user (or `SYSTEM`) that triggered the event.
    pub actor_id: UserId,
    /// Propagated to every event in the same request.
    pub correlation_id: CorrelationId,
    /// For events caused by another event, the causing event id.
    pub causation_id: Option<EventId>,
    /// Wall-clock time of the event.
    pub occurred_at: Timestamp,
    /// Wall-clock time of the persistence (≥ `occurred_at`).
    pub recorded_at: Timestamp,
    /// Serialized payload. Uses the custom `bytes_via_vec`
    /// adapter so the parent type is `DeserializeOwned`.
    #[serde(with = "bytes_via_vec")]
    pub payload: bytes::Bytes,
    /// Soft-delete flag. `Retired` when an operator marks the
    /// event as superseded (e.g. for GDPR erasure or
    /// replays).
    pub active_status: ActiveStatus,
}

/// A filter for `EventLog::read`. Per
/// `docs/schemas/event-schema.md` § 6, consumers query by
/// `(school_id, [event_type], since, until)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogFilter {
    /// The school. Mandatory.
    pub school_id: SchoolId,
    /// If non-empty, only events with these types are returned.
    /// `String` (not `&'static str`) so the type can be
    /// deserialised.
    pub event_types: Vec<String>,
    /// If `Some`, only events with `recorded_at >= since`.
    pub since: Option<Timestamp>,
    /// If `Some`, only events with `recorded_at < until`.
    pub until: Option<Timestamp>,
    /// If `Some`, only events for this aggregate id.
    pub aggregate_id: Option<Uuid>,
    /// Maximum rows to return. The storage adapter enforces the
    /// cap; consumers should paginate.
    pub limit: u32,
}

impl EventLogFilter {
    /// Constructs a filter for the given school, returning all
    /// event types.
    #[must_use]
    pub fn for_school(school_id: SchoolId) -> Self {
        Self {
            school_id,
            event_types: Vec::new(),
            since: None,
            until: None,
            aggregate_id: None,
            limit: 1000,
        }
    }

    /// Restricts the filter to the given event types.
    #[must_use]
    pub fn only_types<I, S>(mut self, types: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.event_types = types.into_iter().map(Into::into).collect();
        self
    }

    /// Restricts the filter to a time window.
    #[must_use]
    pub fn in_window(mut self, since: Timestamp, until: Timestamp) -> Self {
        self.since = Some(since);
        self.until = Some(until);
        self
    }
}

/// The `EventLog` sub-port trait.
#[async_trait]
pub trait EventLog: Send + Sync {
    /// Appends `entry` to the log. Per
    /// `docs/schemas/event-schema.md` § 1.1, the log is the
    /// canonical source of truth for events; the outbox is the
    /// staging area. The relay drains the outbox and writes to
    /// the log via this method.
    async fn append(&self, entry: EventLogEntry) -> Result<()>;

    /// Returns events matching `filter` ordered by `recorded_at`
    /// ascending. The cap is `filter.limit`; the adapter may
    /// enforce a lower cap for safety.
    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>>;

    /// Returns the count of events for `school_id` matching
    /// `filter` (ignoring `limit`). Used by the analytics
    /// consumer to size its cursors.
    async fn count(&self, filter: EventLogFilter) -> Result<u64>;
}

/// Helper: builds an [`EventLogEntry`] from a
/// [`SerializedEnvelope`](super::outbox::SerializedEnvelope) that
/// the relay just drained from the outbox. The `recorded_at`
/// stamp is set to "now" (the moment the relay wrote the row to
/// the event log) and `active_status` is `Active`.
impl EventLogEntry {
    /// Constructs an event-log row from an outbox envelope. The
    /// `recorded_at` stamp is set to the current clock time
    /// (the moment the relay wrote the row); `active_status`
    /// is `Active` (the row is fresh; consumers may transition
    /// to `Retired` for GDPR erasure without deleting the
    /// row).
    #[must_use]
    pub fn from_serialized_envelope(env: &super::outbox::SerializedEnvelope) -> Self {
        Self {
            event_id: env.event_id,
            school_id: env.school_id,
            event_type: env.event_type.clone(),
            schema_version: env.schema_version,
            aggregate_id: env.aggregate_id,
            aggregate_type: env.aggregate_type.clone(),
            actor_id: env.actor_id,
            correlation_id: env.correlation_id,
            causation_id: env.causation_id,
            occurred_at: env.occurred_at,
            recorded_at: Timestamp::now(),
            payload: env.payload.clone(),
            active_status: ActiveStatus::Active,
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

    #[test]
    fn filter_for_school() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let f = EventLogFilter::for_school(school);
        assert_eq!(f.school_id, school);
        assert_eq!(f.limit, 1000);
        assert!(f.event_types.is_empty());
    }

    #[test]
    fn filter_only_types() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let f = EventLogFilter::for_school(school).only_types(vec!["academic.student.admitted"]);
        assert_eq!(f.event_types, vec!["academic.student.admitted"]);
    }
}
