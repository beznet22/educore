//! # Bulk command lifecycle events
//!
//! Per `docs/schemas/command-schema.md` § 12, every bulk command
//! (e.g. `Attendance.Session.MarkBulk`,
//! `Finance.Payment.CollectBulk`) emits three typed events on
//! the bus:
//!
//! - [`BulkCommandStarted`] — emitted once when the engine
//!   accepts the bulk envelope. Carries the total item count.
//! - [`BulkCommandItemProcessed`] — emitted once per item in
//!   the bulk. Carries the 1-based `item_index`, the item's
//!   command type, and the item's aggregate id.
//! - [`BulkCommandCompleted`] — emitted once when the bulk
//!   finishes (success or failure per the `failure_policy`).
//!   Carries the count of successes and failures.
//!
//! The three events share the same `bulk_id` (a UUIDv7 minted
//! at `BulkCommandStarted`) so consumers can group all events
//! from one bulk invocation.
//!
//! Per the spec § 12, the bulk envelope is **all-or-nothing**
//! under the default `FailFast` policy: any failure rolls back
//! the entire batch and `BulkCommandCompleted` reports
//! `successes = 0, failures = item_count`. Under
//! `CollectErrors`, partial success is allowed and the
//! per-item counts reflect the actual outcome.
//!
//! These are scaffolded for the lint gate; the engine's bulk
//! dispatcher (the `educore` umbrella) emits them alongside
//! the existing per-item domain events.

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

/// A bulk command was accepted by the engine
/// (`docs/schemas/command-schema.md` § 12). The engine has
/// validated the envelope and is about to dispatch the items.
/// Consumers (e.g. progress dashboards) subscribe to this
/// event to render "bulk job started" UI.
///
/// The `bulk_id` is the join key for `BulkCommandItemProcessed`
/// and `BulkCommandCompleted` — every item and the completion
/// event for one bulk invocation share the same `bulk_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkCommandStarted {
    /// The bulk's join key (UUIDv7 minted on accept).
    pub bulk_id: Uuid,
    /// The typed id of the school the bulk targets.
    pub school_id: SchoolId,
    /// The bulk command type, e.g. `"attendance.session.mark_bulk"`.
    pub command_type: String,
    /// The total number of items in the bulk envelope.
    pub item_count: u32,
    /// The failure policy the dispatcher will use
    /// (`FailFast` default; `CollectErrors` for partial-success
    /// bulks).
    pub failure_policy: BulkFailurePolicy,
    /// The concurrency limit the dispatcher will use (default
    /// `1` = sequential).
    pub concurrency_limit: u32,
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl BulkCommandStarted {
    /// Mints a fresh `BulkCommandStarted` with the current
    /// clock time.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        bulk_id: Uuid,
        school_id: SchoolId,
        command_type: String,
        item_count: u32,
        failure_policy: BulkFailurePolicy,
        concurrency_limit: u32,
    ) -> Self {
        Self {
            bulk_id,
            school_id,
            command_type,
            item_count,
            failure_policy,
            concurrency_limit,
            event_id: Uuid::now_v7(),
            occurred_at: Timestamp::now(),
        }
    }
}

impl DomainEvent for BulkCommandStarted {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"engine.bulk_command.started"`.
    const EVENT_TYPE: &'static str = "engine.bulk_command.started";
    const SCHEMA_VERSION: u32 = 1;
    /// Bulk commands are cross-domain; the aggregate type is
    /// the bulk's `command_type` (encoded in the envelope's
    /// `aggregate_type` via `bulk.command_type`).
    const AGGREGATE_TYPE: &'static str = "bulk_command";

    fn event_id(&self) -> EventId {
        event_id_from(self.event_id)
    }
    fn aggregate_id(&self) -> Uuid {
        self.bulk_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// One item in a bulk was processed. Emitted once per item in
/// the bulk envelope (so a 400-item bulk emits 400 of these).
///
/// `item_index` is 1-based and matches the position of the
/// item in the bulk envelope. The item's aggregate id is
/// reported in `aggregate_id`; consumers (e.g. a per-row
/// progress bar) use this to highlight the row in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkCommandItemProcessed {
    /// The bulk's join key (matches the parent
    /// [`BulkCommandStarted`]).
    pub bulk_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// 1-based index of this item in the bulk envelope.
    pub item_index: u32,
    /// The item's command type (e.g. the concrete
    /// `attendance.session.mark` for a `MarkBulk` bulk).
    pub command_type: String,
    /// The aggregate id the item touched.
    pub aggregate_id: Uuid,
    /// `true` if the item succeeded; `false` if it failed
    /// (under `CollectErrors`; never `false` under
    /// `FailFast` since the whole bulk is rolled back).
    pub success: bool,
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl BulkCommandItemProcessed {
    /// Mints a fresh `BulkCommandItemProcessed` with the
    /// current clock time.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        bulk_id: Uuid,
        school_id: SchoolId,
        item_index: u32,
        command_type: String,
        aggregate_id: Uuid,
        success: bool,
    ) -> Self {
        Self {
            bulk_id,
            school_id,
            item_index,
            command_type,
            aggregate_id,
            success,
            event_id: Uuid::now_v7(),
            occurred_at: Timestamp::now(),
        }
    }
}

impl DomainEvent for BulkCommandItemProcessed {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"engine.bulk_command.item_processed"`.
    const EVENT_TYPE: &'static str = "engine.bulk_command.item_processed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_command";

    fn event_id(&self) -> EventId {
        event_id_from(self.event_id)
    }
    fn aggregate_id(&self) -> Uuid {
        self.bulk_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A bulk command finished
/// (`docs/schemas/command-schema.md` § 12). The event is
/// emitted exactly once per bulk invocation, regardless of
/// `failure_policy`. Under `FailFast`, `successes = 0` and
/// `failures = item_count` whenever any item fails. Under
/// `CollectErrors`, the counts reflect the actual per-item
/// outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkCommandCompleted {
    /// The bulk's join key (matches the parent
    /// [`BulkCommandStarted`]).
    pub bulk_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// The bulk command type.
    pub command_type: String,
    /// Total items in the bulk envelope.
    pub item_count: u32,
    /// Items that succeeded (under `CollectErrors`).
    pub successes: u32,
    /// Items that failed (under `CollectErrors`).
    pub failures: u32,
    /// The failure policy that was in effect.
    pub failure_policy: BulkFailurePolicy,
    /// Total wall-clock duration of the bulk dispatch.
    pub duration_ms: u64,
    /// Mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl BulkCommandCompleted {
    /// Mints a fresh `BulkCommandCompleted` with the current
    /// clock time.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        bulk_id: Uuid,
        school_id: SchoolId,
        command_type: String,
        item_count: u32,
        successes: u32,
        failures: u32,
        failure_policy: BulkFailurePolicy,
        duration_ms: u64,
    ) -> Self {
        Self {
            bulk_id,
            school_id,
            command_type,
            item_count,
            successes,
            failures,
            failure_policy,
            duration_ms,
            event_id: Uuid::now_v7(),
            occurred_at: Timestamp::now(),
        }
    }
}

impl DomainEvent for BulkCommandCompleted {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"engine.bulk_command.completed"`.
    const EVENT_TYPE: &'static str = "engine.bulk_command.completed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "bulk_command";

    fn event_id(&self) -> EventId {
        event_id_from(self.event_id)
    }
    fn aggregate_id(&self) -> Uuid {
        self.bulk_id
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Failure policy for a bulk command
/// (`docs/schemas/command-schema.md` § 12). The default is
/// `FailFast`; consumers can request `CollectErrors` to allow
/// partial success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BulkFailurePolicy {
    /// Abort the bulk on the first failure (default). The
    /// entire batch is rolled back.
    FailFast,
    /// Continue processing items after a failure, recording
    /// per-item errors in a result list. The bulk commits
    /// successful items.
    CollectErrors,
}
