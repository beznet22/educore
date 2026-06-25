//! SurrealDB-backed `Outbox` sub-port.
//!
//! Stores each envelope as a row in the `outbox` table. The
//! schema is defined by the canonical .surql migration
//! (loaded by `SurrealStorageAdapter::migrate`).

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use surrealdb::sql::{Datetime, Uuid as SurrealUuid};

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_storage::outbox::{Outbox, SerializedEnvelope};

use crate::connection::Db;
use crate::error::StringError;

/// The row shape stored in the SurrealDB `outbox` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct OutboxRow {
    pub event_id: SurrealUuid,
    pub event_type: String,
    pub event_version: i32,
    pub school_id: Option<SurrealUuid>,
    pub aggregate_id: SurrealUuid,
    pub aggregate_type: String,
    pub actor_id: SurrealUuid,
    pub correlation_id: SurrealUuid,
    pub causation_id: Option<SurrealUuid>,
    pub occurred_at: Datetime,
    pub recorded_at: Datetime,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub enqueued_at: Option<Datetime>,
}

impl OutboxRow {
    /// Maps a `SerializedEnvelope` to a row ready for insert.
    pub fn from_envelope(env: &SerializedEnvelope) -> Self {
        let payload_value: serde_json::Value =
            serde_json::from_slice(&env.payload).unwrap_or_else(|_| {
                serde_json::Value::String(String::from_utf8_lossy(&env.payload).into_owned())
            });
        Self {
            event_id: SurrealUuid::from(env.event_id.as_uuid()),
            event_type: env.event_type.clone(),
            // SurrealDB's `i32` ↔ engine's `u32`. The engine's
            // schema versions are small positive integers (the
            // schema is bumped at most a handful of times per
            // release), so the conversion can never lose data in
            // practice. We use `try_from` + `unwrap_or(0)` to
            // satisfy the `cast_possible_wrap` / `cast_sign_loss`
            // deny lints; a default of `0` matches the row's
            // historical invariant for uninitialized rows.
            event_version: i32::try_from(env.schema_version).unwrap_or(0),
            school_id: Some(SurrealUuid::from(env.school_id.as_uuid())),
            aggregate_id: SurrealUuid::from(env.aggregate_id),
            aggregate_type: env.aggregate_type.clone(),
            actor_id: SurrealUuid::from(env.actor_id.as_uuid()),
            correlation_id: SurrealUuid::from(env.correlation_id.as_uuid()),
            causation_id: env.causation_id.map(|c| SurrealUuid::from(c.as_uuid())),
            occurred_at: Datetime::from(env.occurred_at.as_datetime()),
            recorded_at: Datetime::from(env.occurred_at.as_datetime()),
            payload: payload_value,
            enqueued_at: None,
        }
    }

    /// Maps a row back to a `SerializedEnvelope`. The row is
    /// borrowed (not consumed) so callers can iterate over the
    /// query result without cloning each row.
    pub fn to_envelope(&self) -> SerializedEnvelope {
        let event_id = EventId::from_uuid(self.event_id.0);
        let school_id = match self.school_id {
            Some(u) => SchoolId::from_uuid(u.0),
            None => SchoolId::from_uuid(uuid::Uuid::nil()),
        };
        let aggregate_id = self.aggregate_id.0;
        let actor_id = UserId::from_uuid(self.actor_id.0);
        let correlation_id = CorrelationId::from_uuid(self.correlation_id.0);
        let causation_id = self.causation_id.map(|u| EventId::from_uuid(u.0));
        let occurred_at = from_surreal_datetime(&self.occurred_at);
        let payload = payload_to_bytes(&self.payload);
        SerializedEnvelope {
            event_id,
            event_type: self.event_type.clone(),
            // Mirrors the `try_from(0)` on the insert path: any
            // negative `event_version` is clamped to `0` on read.
            schema_version: u32::try_from(self.event_version).unwrap_or(0),
            school_id,
            aggregate_id,
            aggregate_type: self.aggregate_type.clone(),
            actor_id,
            correlation_id,
            causation_id,
            occurred_at,
            payload,
        }
    }
}

/// Convert a SurrealDB `Datetime` (borrowed) to a `Timestamp`.
/// Borrows rather than consumes so the caller can iterate
/// over query results without cloning each `Datetime`.
fn from_surreal_datetime(dt: &Datetime) -> Timestamp {
    let chrono_dt: DateTime<Utc> = dt.0;
    Timestamp::from_datetime(chrono_dt)
}

#[allow(dead_code)]
fn parse_event_id(s: &str) -> std::result::Result<EventId, StringError> {
    let u = parse_uuid(s).map_err(|e| StringError(format!("bad event_id: {e}")))?;
    Ok(EventId::from_uuid(u))
}

#[allow(dead_code)]
fn parse_school_id_opt(s: Option<&str>) -> std::result::Result<SchoolId, StringError> {
    match s {
        Some(s) => {
            let u = parse_uuid(s).map_err(|e| StringError(format!("bad school_id: {e}")))?;
            Ok(SchoolId::from_uuid(u))
        }
        None => Ok(SchoolId::from_uuid(uuid::Uuid::nil())),
    }
}

#[allow(dead_code)]
fn parse_user_id(s: &str) -> std::result::Result<UserId, StringError> {
    let u = parse_uuid(s).map_err(|e| StringError(format!("bad actor_id: {e}")))?;
    Ok(UserId::from_uuid(u))
}

#[allow(dead_code)]
fn parse_correlation_id(s: &str) -> std::result::Result<CorrelationId, StringError> {
    let u = parse_uuid(s).map_err(|e| StringError(format!("bad correlation_id: {e}")))?;
    Ok(CorrelationId::from_uuid(u))
}

#[allow(dead_code)]
fn parse_uuid(s: &str) -> std::result::Result<uuid::Uuid, uuid::Error> {
    uuid::Uuid::parse_str(s)
}

/// Convert a `serde_json::Value` payload back to `Bytes` for
/// the in-memory envelope. Object values are re-serialised to
/// JSON; everything else is stringified.
fn payload_to_bytes(v: &serde_json::Value) -> Bytes {
    let s = match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    Bytes::from(s.into_bytes())
}

/// The SurrealDB-backed `Outbox` implementation.
#[derive(Clone)]
pub struct SurrealOutbox {
    pub(crate) db: Db,
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealOutbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealOutbox")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SurrealOutbox {
    /// Constructs a new outbox handle bound to `db` and
    /// filtered to `school`.
    pub fn new(db: Db, school: SchoolId) -> Self {
        Self { db, school }
    }
}

#[async_trait]
impl Outbox for SurrealOutbox {
    async fn append(&self, _school_id: educore_core::ids::SchoolId, envelope: SerializedEnvelope) -> Result<()> {
        let row = OutboxRow::from_envelope(&envelope);
        let mut response = self
            .db
            .query(
                "INSERT INTO outbox { \
                    event_id: $event_id, \
                    event_type: $event_type, \
                    event_version: $event_version, \
                    school_id: $school_id, \
                    aggregate_id: $aggregate_id, \
                    aggregate_type: $aggregate_type, \
                    actor_id: $actor_id, \
                    correlation_id: $correlation_id, \
                    causation_id: $causation_id, \
                    occurred_at: $occurred_at, \
                    recorded_at: $recorded_at, \
                    payload: $payload, \
                    enqueued_at: time::now(), \
                    published_at: NONE, \
                    attempts: 0, \
                    last_error: NONE \
                }",
            )
            .bind(("event_id", row.event_id))
            .bind(("event_type", row.event_type.clone()))
            .bind(("event_version", row.event_version))
            .bind(("school_id", row.school_id))
            .bind(("aggregate_id", row.aggregate_id))
            .bind(("aggregate_type", row.aggregate_type.clone()))
            .bind(("actor_id", row.actor_id))
            .bind(("correlation_id", row.correlation_id))
            .bind(("causation_id", row.causation_id))
            .bind(("occurred_at", row.occurred_at))
            .bind(("recorded_at", row.recorded_at))
            .bind(("payload", row.payload))
            .await
            .map_err(|e| StringError(format!("outbox append: {e}")))?;
        // Pull the typed result at position 0 just to confirm the
        // INSERT succeeded and surface any server-side error.
        let _: Vec<OutboxRow> = response
            .take(0)
            .map_err(|e| StringError(format!("outbox append take: {e}")))?;
        Ok(())
    }

    async fn pending(&self, _school_id: educore_core::ids::SchoolId, limit: u32) -> Result<Vec<SerializedEnvelope>> {
        let school_uuid = SurrealUuid::from(self.school.as_uuid());
        let mut response = self
            .db
            .query(
                "SELECT \
                    event_id, event_type, event_version, school_id, \
                    aggregate_id, aggregate_type, actor_id, correlation_id, \
                    causation_id, occurred_at, recorded_at, payload, enqueued_at \
                 FROM outbox \
                 WHERE school_id = $school AND published_at IS NONE \
                 ORDER BY enqueued_at ASC \
                 LIMIT $limit",
            )
            .bind(("school", school_uuid))
            .bind(("limit", i64::from(limit)))
            .await
            .map_err(|e| StringError(format!("outbox pending: {e}")))?;
        let rows: Vec<OutboxRow> = response
            .take(0)
            .map_err(|e| StringError(format!("outbox pending take: {e}")))?;
        let mut envelopes = Vec::with_capacity(rows.len());
        for row in rows {
            let env = row.to_envelope();
            envelopes.push(env);
        }
        Ok(envelopes)
    }

    async fn mark_published(&self, _school_id: educore_core::ids::SchoolId, ids: &[EventId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let id_uuids: Vec<SurrealUuid> =
            ids.iter().map(|i| SurrealUuid::from(i.as_uuid())).collect();
        let mut response = self
            .db
            .query(
                "UPDATE outbox SET published_at = time::now() \
                 WHERE event_id IN $ids",
            )
            .bind(("ids", id_uuids))
            .await
            .map_err(|e| StringError(format!("outbox mark_published: {e}")))?;
        let _: Vec<OutboxRow> = response
            .take(0)
            .map_err(|e| StringError(format!("outbox mark_published take: {e}")))?;
        Ok(())
    }
}
