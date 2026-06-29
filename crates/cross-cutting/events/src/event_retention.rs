//! # Event log retention events
//!
//! Per `docs/schemas/event-schema.md` § 9, the engine retains
//! events for the consumer-configured retention period; after
//! that, events are compacted to a projection (raw events
//! remain available in cold storage for compliance).
//!
//! This module exposes the typed event the engine publishes
//! when a sweep is due:
//!
//! - [`EventLogRetentionSweepDue`] — published by the event-log
//!   retention sweeper when the configurable threshold is
//!   reached. Consumers (typically a background job in the
//!   consumer's deployment) subscribe to
//!   `event_log.retention.sweep_due` and execute the actual
//!   `DELETE FROM event_log WHERE occurred_at < ?` sweep.
//!
//! Per the spec, the retention job is a **consumer concern**:
//! the engine emits the signal, the consumer performs the
//! deletion. This keeps the engine's event-log writer
//! append-only (it never deletes rows itself) and the deletion
//! transactional and observable through the event bus.
//!
//! The audit crate has an analogous [`RetentionSweepDue`] event
//! (`audit.retention.sweep_due`); this is the parallel for the
//! event_log.
//!
//! [`RetentionSweepDue`]: https://docs.rs/educore-audit

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::domain_event::DomainEvent;

/// A retention sweep is due for the `event_log` of `school_id`.
/// Consumers (typically a background job subscribed to
/// `event_log.retention.sweep_due`) execute the actual
/// `DELETE FROM event_log WHERE occurred_at < cutoff` and
/// optionally archive the rows to cold storage first.
///
/// `cutoff` is the timestamp provided to the `DELETE` statement
/// — rows with `occurred_at < cutoff` are eligible for removal.
/// `at` is the clock time the engine decided the sweep was due;
/// `event_id` is the mint-time UUIDv7 of this event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogRetentionSweepDue {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor — the school whose event_log needs sweeping.
    pub school_id: SchoolId,
    /// The cutoff timestamp: rows with `occurred_at < cutoff` are
    /// eligible for removal.
    pub cutoff: Timestamp,
    /// The clock time the engine decided the sweep was due.
    pub at: Timestamp,
}

impl EventLogRetentionSweepDue {
    /// Mints a fresh `EventLogRetentionSweepDue` event with a new
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

impl DomainEvent for EventLogRetentionSweepDue {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"event_log.retention.sweep_due"`.
    const EVENT_TYPE: &'static str = "event_log.retention.sweep_due";

    /// Schema version of the payload shape. Bumped on
    /// backward-incompatible payload changes.
    const SCHEMA_VERSION: u32 = 1;

    /// The aggregate type name. The "aggregate" for retention
    /// is the school's event_log itself, so we use a distinct
    /// `event_log_retention` tag to keep it separate from the
    /// domain aggregates the event_log is recording.
    const AGGREGATE_TYPE: &'static str = "event_log_retention";

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

/// Retention policy applied to the event log.
///
/// Mirrors the audit crate's [`RetentionPolicy`] but is
/// independently configurable — most deployments keep the
/// event_log longer than the audit_log (e.g. 365 days vs. 90
/// days) because the event_log is the source for replay and
/// compliance reconstruction. Both share the same shape so
/// consumers can reuse the same job runner.
///
/// [`RetentionPolicy`]: https://docs.rs/educore-audit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogRetentionPolicy {
    /// Number of days a row is retained before it is eligible
    /// for archival / deletion. Default: **365 days** per
    /// `docs/schemas/event-schema.md` § 9.
    pub retention_days: u32,
    /// Minimum wall-clock duration between two sweep checks.
    /// The check is opportunistic: the engine runs it after
    /// every event-log write, but only emits the sweep-due
    /// event when this interval has elapsed since the last
    /// emission. Default: **24 hours**.
    pub sweep_check_interval: std::time::Duration,
}

impl Default for EventLogRetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 365,
            sweep_check_interval: std::time::Duration::from_secs(24 * 60 * 60),
        }
    }
}
