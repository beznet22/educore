//! The audit-domain typed events.
//!
//! Currently exposes a single event:
//!
//! - [`RetentionSweepDue`] — published by [`crate::writer::AuditWriter`]
//!   when the retention policy is reached. Consumers (typically a
//!   background job in the consumer's deployment) subscribe to
//!   `audit.retention.sweep_due` and execute the actual
//!   `DELETE FROM audit_log WHERE occurred_at < ?` sweep.
//!
//! Per `docs/schemas/audit-schema.md` § 9, the retention job is a
//! consumer concern: the engine emits the signal, the consumer
//! performs the deletion. This keeps the engine's audit writer
//! write-only (it never deletes rows itself) and the deletion
//! transactional and observable through the event bus.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use educore_events::domain_event::DomainEvent;

/// A retention sweep is due for `school_id`. Consumers (typically a
/// background job subscribed to `audit.retention.sweep_due`) execute
/// the actual `DELETE FROM audit_log WHERE occurred_at < cutoff` and
/// optionally archive the rows to cold storage first.
///
/// `cutoff` is the timestamp provided to the `DELETE` statement —
/// rows with `occurred_at < cutoff` are eligible for removal.
/// `at` is the clock time the engine decided the sweep was due;
/// `event_id` is the mint-time UUIDv7 of this event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionSweepDue {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor — the school whose audit log needs sweeping.
    pub school_id: SchoolId,
    /// The cutoff timestamp: rows with `occurred_at < cutoff` are
    /// eligible for removal.
    pub cutoff: Timestamp,
    /// The clock time the engine decided the sweep was due.
    pub at: Timestamp,
}

impl RetentionSweepDue {
    /// Mints a fresh `RetentionSweepDue` event with a new
    /// UUIDv7 `event_id` and the given `cutoff` and `at` timestamps.
    #[must_use]
    pub fn new(school_id: SchoolId, cutoff: Timestamp, at: Timestamp) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            school_id,
            cutoff,
            at,
        }
    }

    /// Returns the inner event id as a typed [`EventId`].
    #[must_use]
    pub fn typed_event_id(&self) -> EventId {
        EventId::from_uuid(self.event_id)
    }
}

impl DomainEvent for RetentionSweepDue {
    /// Stable dotted event-type string. The subscription key for
    /// consumers is the string `"audit.retention.sweep_due"`.
    const EVENT_TYPE: &'static str = "audit.retention.sweep_due";

    /// Schema version of the payload shape. Bumped on
    /// backward-incompatible payload changes.
    const SCHEMA_VERSION: u32 = 1;

    /// The aggregate type name. The "aggregate" for retention is
    /// the school's audit log itself, so we use a distinct
    /// `audit_retention` tag to keep it separate from the
    /// domain aggregates the audit log is recording.
    const AGGREGATE_TYPE: &'static str = "audit_retention";

    fn event_id(&self) -> EventId {
        self.typed_event_id()
    }

    fn aggregate_id(&self) -> Uuid {
        self.school_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.at
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
    use educore_core::clock::IdGenerator;
    use educore_core::clock::SystemIdGen;
    use educore_core::tenant::TenantContext;

    #[test]
    fn event_type_is_audit_retention_sweep_due() {
        assert_eq!(RetentionSweepDue::EVENT_TYPE, "audit.retention.sweep_due");
        assert_eq!(RetentionSweepDue::SCHEMA_VERSION, 1);
        assert_eq!(RetentionSweepDue::AGGREGATE_TYPE, "audit_retention");
    }

    #[test]
    fn new_mints_v7_event_id() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let now = Timestamp::now();
        let cutoff = Timestamp::epoch();
        let event = RetentionSweepDue::new(school, cutoff, now);
        assert_eq!(event.event_id.get_version_num(), 7);
        assert_eq!(event.school_id, school);
        assert_eq!(event.cutoff, cutoff);
        assert_eq!(event.at, now);
    }

    #[test]
    fn into_envelope_stamps_school_and_aggregate() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let corr = g.next_correlation_id();
        let ctx = TenantContext::system(school, corr);
        let event = RetentionSweepDue::new(
            school,
            Timestamp::from_datetime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap(),
            ),
            Timestamp::now(),
        );
        let env = event.into_envelope(&ctx);
        assert_eq!(env.event_type, "audit.retention.sweep_due");
        assert_eq!(env.aggregate_type, "audit_retention");
        assert_eq!(env.school_id, school);
        assert_eq!(env.actor_id, ctx.actor_id);
        assert_eq!(env.correlation_id, corr);
    }

    #[test]
    fn payload_round_trips_through_serde() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let event = RetentionSweepDue::new(school, Timestamp::epoch(), Timestamp::now());
        let json = serde_json::to_string(&event).unwrap();
        let back: RetentionSweepDue = serde_json::from_str(&json).unwrap();
        assert_eq!(event, back);
    }

    #[test]
    fn payload_contains_cutoff_and_at() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let ctx = TenantContext::system(school, g.next_correlation_id());
        let event = RetentionSweepDue::new(school, Timestamp::epoch(), Timestamp::now());
        let env = event.into_envelope(&ctx);
        assert!(env.payload.get("cutoff").is_some());
        assert!(env.payload.get("at").is_some());
        assert!(env.payload.get("school_id").is_some());
    }
}
