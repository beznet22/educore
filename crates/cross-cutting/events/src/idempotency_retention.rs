//! # Idempotency TTL sweep events
//!
//! The engine's idempotency table grows unboundedly: every
//! mutating command writes one row. Without a TTL sweep, the
//! table grows by ~50K rows/day at 50K events/day (per
//! `ADR-014-Idempotency.md`'s sizing analysis).
//!
//! Per the same ADR, idempotency rows are kept for the
//! consumer-configured `idempotency_ttl` (default **7 days**).
//! After that, the row is eligible for archival / deletion.
//!
//! This module exposes the typed event the engine publishes
//! when a sweep is due:
//!
//! - [`IdempotencyTtlSweepDue`] — published by the
//!   idempotency-TTL sweeper when the configurable threshold is
//!   reached. Consumers (typically a background job in the
//!   consumer's deployment) subscribe to
//!   `idempotency.ttl.sweep_due` and execute the actual
//!   `DELETE FROM idempotency WHERE recorded_at < ?` sweep.
//!
//! Per the engine pattern, the TTL job is a **consumer
//! concern**: the engine emits the signal, the consumer
//! performs the deletion. This keeps the engine's idempotency
//! writer append-only (it never deletes rows itself) and the
//! deletion transactional and observable through the event bus.
//!
//! Parallel to the audit crate's [`RetentionSweepDue`] event
//! and to the [`EventLogRetentionSweepDue`] event in
//! `event_retention.rs`.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::domain_event::DomainEvent;

/// Internal helper: constructs an `EventId` from a `Uuid`.
#[inline]
fn event_id_from(uuid: Uuid) -> EventId {
    EventId::from_uuid(uuid)
}

/// A TTL sweep is due for the `idempotency` table. Consumers
/// (typically a background job subscribed to
/// `idempotency.ttl.sweep_due`) execute the actual
/// `DELETE FROM idempotency WHERE recorded_at < cutoff`.
///
/// `cutoff` is the timestamp provided to the `DELETE`
/// statement — rows with `recorded_at < cutoff` are eligible
/// for removal. `at` is the clock time the engine decided the
/// sweep was due; `event_id` is the mint-time UUIDv7 of this
/// event.
///
/// Note: the idempotency table is global (not per-school);
/// however, we still carry `school_id` in the event payload
/// so multi-tenant consumer deployments can shard the sweep
/// per-tenant if they prefer (the consumer is free to scope
/// the DELETE by `school_id` as well as by `recorded_at`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyTtlSweepDue {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor — the school whose idempotency rows
    /// are being swept. May be a tenant-id sentinel for
    /// "all schools" sweeps.
    pub school_id: SchoolId,
    /// The cutoff timestamp: rows with `recorded_at < cutoff`
    /// are eligible for removal.
    pub cutoff: Timestamp,
    /// The clock time the engine decided the sweep was due.
    pub at: Timestamp,
}

impl IdempotencyTtlSweepDue {
    /// Mints a fresh `IdempotencyTtlSweepDue` event with a new
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

impl DomainEvent for IdempotencyTtlSweepDue {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"idempotency.ttl.sweep_due"`.
    const EVENT_TYPE: &'static str = "idempotency.ttl.sweep_due";

    /// Schema version of the payload shape. Bumped on
    /// backward-incompatible payload changes.
    const SCHEMA_VERSION: u32 = 1;

    /// The aggregate type name. The "aggregate" for TTL is
    /// the idempotency table itself, so we use a distinct
    /// `idempotency_ttl` tag to keep it separate from the
    /// command's primary aggregate.
    const AGGREGATE_TYPE: &'static str = "idempotency_ttl";

    fn event_id(&self) -> EventId {
        event_id_from(self.event_id)
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

/// Retention policy applied to the idempotency table.
///
/// Per `ADR-014-Idempotency.md`, the engine defaults to a
/// **7-day TTL** for idempotency records (a retried command
/// after a week is treated as a fresh command). Consumers
/// override via the `idempotency_ttl` setting on the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRetentionPolicy {
    /// Number of days a row is retained before it is eligible
    /// for archival / deletion. Default: **7 days** per
    /// `ADR-014-Idempotency.md`.
    pub ttl_days: u32,
    /// Minimum wall-clock duration between two sweep checks.
    /// The check is opportunistic: the engine runs it after
    /// every idempotency write, but only emits the sweep-due
    /// event when this interval has elapsed since the last
    /// emission. Default: **6 hours** (TTL is short, so we
    /// sweep frequently).
    pub sweep_check_interval: std::time::Duration,
}

impl Default for IdempotencyRetentionPolicy {
    fn default() -> Self {
        Self {
            ttl_days: 7,
            sweep_check_interval: std::time::Duration::from_secs(6 * 60 * 60),
        }
    }
}
