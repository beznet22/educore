//! # Sync events
//!
//! The four typed domain events emitted by the sync engine. Per
//! Phase 0 open question #2, these replace the ad-hoc
//! `SyncEvent` enum that previously lived in `educore-sync`. The
//! in-process sync impl ([`educore-sync-inprocess`]) now publishes
//! these via the bus instead of broadcasting through a custom
//! channel.
//!
//! Each event carries `(school_id, at: Timestamp)`. The
//! `event_type` is namespaced under `sync.*` to make subscription
//! filtering trivial.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{EventId, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;

use crate::domain_event::DomainEvent;

/// Internal helper: constructs an `EventId` from a `Uuid` (for the
/// `event_id()` method on each event).
#[inline]
fn event_id_from(uuid: Uuid) -> EventId {
    EventId::from_uuid(uuid)
}

/// A sync session was started for `school_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStarted {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// Clock time of the event.
    pub at: Timestamp,
}

impl SyncStarted {
    /// Mints a fresh `SyncStarted` with the current clock time.
    #[must_use]
    pub fn now(school_id: SchoolId) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
        }
    }
}

impl DomainEvent for SyncStarted {
    const EVENT_TYPE: &'static str = "sync.session.started";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sync_session";
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

/// A sync session was paused for `school_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncPaused {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// Clock time of the event.
    pub at: Timestamp,
    /// The `event_id` of the `SyncStarted` that opened this
    /// session, if known. Allows consumers to correlate
    /// start/pause/resume/stop sequences.
    pub session_started_event_id: Option<Uuid>,
}

impl SyncPaused {
    /// Mints a fresh `SyncPaused` with the current clock time.
    #[must_use]
    pub fn now(school_id: SchoolId) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: None,
        }
    }

    /// Mints a fresh `SyncPaused` correlated to a `SyncStarted`.
    #[must_use]
    pub fn for_session(school_id: SchoolId, started: &SyncStarted) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: Some(started.event_id),
        }
    }
}

impl DomainEvent for SyncPaused {
    const EVENT_TYPE: &'static str = "sync.session.paused";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sync_session";
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

/// A sync session was resumed for `school_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncResumed {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// Clock time of the event.
    pub at: Timestamp,
    /// The `event_id` of the `SyncStarted` that opened this
    /// session, if known.
    pub session_started_event_id: Option<Uuid>,
}

impl SyncResumed {
    /// Mints a fresh `SyncResumed` with the current clock time.
    #[must_use]
    pub fn now(school_id: SchoolId) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: None,
        }
    }

    /// Mints a fresh `SyncResumed` correlated to a `SyncStarted`.
    #[must_use]
    pub fn for_session(school_id: SchoolId, started: &SyncStarted) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: Some(started.event_id),
        }
    }
}

impl DomainEvent for SyncResumed {
    const EVENT_TYPE: &'static str = "sync.session.resumed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sync_session";
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

/// A sync session was stopped for `school_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncStopped {
    /// The mint-time event id (UUIDv7).
    pub event_id: Uuid,
    /// The tenant anchor.
    pub school_id: SchoolId,
    /// Clock time of the event.
    pub at: Timestamp,
    /// The `event_id` of the `SyncStarted` that opened this
    /// session, if known.
    pub session_started_event_id: Option<Uuid>,
}

impl SyncStopped {
    /// Mints a fresh `SyncStopped` with the current clock time.
    #[must_use]
    pub fn now(school_id: SchoolId) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: None,
        }
    }

    /// Mints a fresh `SyncStopped` correlated to a `SyncStarted`.
    #[must_use]
    pub fn for_session(school_id: SchoolId, started: &SyncStarted) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            school_id,
            at: Timestamp::now(),
            session_started_event_id: Some(started.event_id),
        }
    }
}

impl DomainEvent for SyncStopped {
    const EVENT_TYPE: &'static str = "sync.session.stopped";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sync_session";
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

/// Convenience constructor: given a `TenantContext` and a
/// started session, mints all four events with correlated ids.
/// Useful for tests.
#[must_use]
pub fn sync_lifecycle_events(
    school_id: SchoolId,
) -> (SyncStarted, SyncPaused, SyncResumed, SyncStopped) {
    let started = SyncStarted::now(school_id);
    let paused = SyncPaused::for_session(school_id, &started);
    let resumed = SyncResumed::for_session(school_id, &started);
    let stopped = SyncStopped::for_session(school_id, &started);
    (started, paused, resumed, stopped)
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
    use educore_core::tenant::{TenantContext, UserType};
    use serde_json::json;

    #[test]
    fn sync_started_event_type_is_namespaced() {
        let g = SystemIdGen;
        let env = SyncStarted::now(g.next_school_id()).into_envelope(&TenantContext::system(
            g.next_school_id(),
            g.next_correlation_id(),
        ));
        assert_eq!(env.event_type, "sync.session.started");
        assert_eq!(env.aggregate_type, "sync_session");
        assert_eq!(env.schema_version, 1);
    }

    #[test]
    fn sync_paused_correlates_to_started() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let started = SyncStarted::now(school);
        let paused = SyncPaused::for_session(school, &started);
        assert_eq!(paused.session_started_event_id, Some(started.event_id));
    }

    #[test]
    fn all_four_event_types_are_unique() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let ctx = TenantContext::for_user(
            school,
            g.next_user_id(),
            g.next_correlation_id(),
            UserType::System,
        );
        let (s, p, r, t) = sync_lifecycle_events(school);
        let envs = [
            s.into_envelope(&ctx),
            p.into_envelope(&ctx),
            r.into_envelope(&ctx),
            t.into_envelope(&ctx),
        ];
        let types: Vec<&str> = envs.iter().map(|e| e.event_type).collect();
        assert_eq!(
            types,
            vec![
                "sync.session.started",
                "sync.session.paused",
                "sync.session.resumed",
                "sync.session.stopped",
            ]
        );
    }

    #[test]
    fn sync_started_serialises_to_expected_json_shape() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let started = SyncStarted::now(school);
        let env: crate::envelope::EventEnvelope =
            started.into_envelope(&TenantContext::system(school, g.next_correlation_id()));
        let payload: serde_json::Value = env.payload;
        assert!(payload.get("event_id").is_some());
        assert!(payload.get("at").is_some());
        assert!(payload.get("school_id").is_some());
        let _ = json!(payload);
    }
}
