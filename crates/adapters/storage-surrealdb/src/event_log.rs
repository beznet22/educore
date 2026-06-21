//! SurrealDB-backed `EventLog` sub-port.
//!
//! Stores each event log row in the `event_log` table. The schema
//! is defined by the canonical .surql migration (loaded by
//! `SurrealStorageAdapter::migrate`).
//!
//! Wired into `lib.rs` by A'.1 (Phase 16); the stub in
//! `stubs.rs` has been removed in the same commit.

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use surrealdb::sql::{Bytes as SurrealBytes, Datetime, Uuid as SurrealUuid};
use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};

use crate::connection::Db;
use crate::error::StringError;

/// The row shape stored in the SurrealDB `event_log` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct EventRow {
    pub event_id: SurrealUuid,
    pub school_id: Option<SurrealUuid>,
    pub event_type: String,
    pub schema_version: u32,
    pub aggregate_id: SurrealUuid,
    pub aggregate_type: String,
    pub actor_id: SurrealUuid,
    pub correlation_id: SurrealUuid,
    pub causation_id: Option<SurrealUuid>,
    pub occurred_at: Datetime,
    pub recorded_at: Datetime,
    pub payload: SurrealBytes,
    pub active_status: String,
}

impl EventRow {
    pub fn from_entry(entry: &EventLogEntry) -> Self {
        Self {
            event_id: SurrealUuid::from(entry.event_id.as_uuid()),
            school_id: Some(SurrealUuid::from(entry.school_id.as_uuid())),
            event_type: entry.event_type.clone(),
            schema_version: entry.schema_version,
            aggregate_id: SurrealUuid::from(entry.aggregate_id),
            aggregate_type: entry.aggregate_type.clone(),
            actor_id: SurrealUuid::from(entry.actor_id.as_uuid()),
            correlation_id: SurrealUuid::from(entry.correlation_id.as_uuid()),
            causation_id: entry.causation_id.map(|e| SurrealUuid::from(e.as_uuid())),
            occurred_at: Datetime::from(entry.occurred_at.as_datetime()),
            recorded_at: Datetime::from(entry.recorded_at.as_datetime()),
            payload: SurrealBytes::from(entry.payload.to_vec()),
            active_status: entry.active_status.to_string(),
        }
    }

    pub fn to_entry(&self) -> EventLogEntry {
        let school_id = self
            .school_id
            .map(|u| SchoolId::from_uuid(u.0))
            .unwrap_or_else(|| SchoolId::from_uuid(uuid::Uuid::nil()));
        let actor_id = UserId::from_uuid(self.actor_id.0);
        let correlation_id = CorrelationId::from_uuid(self.correlation_id.0);
        let event_id = EventId::from_uuid(self.event_id.0);
        let causation_id = self.causation_id.map(|u| EventId::from_uuid(u.0));
        let occurred_at = Timestamp::from_datetime(self.occurred_at.0);
        let recorded_at = Timestamp::from_datetime(self.recorded_at.0);
        let payload = Bytes::from(self.payload.to_vec());
        let active_status = match self.active_status.as_str() {
            "active" => ActiveStatus::Active,
            _ => ActiveStatus::Retired,
        };
        EventLogEntry {
            event_id,
            school_id,
            event_type: self.event_type.clone(),
            schema_version: self.schema_version,
            aggregate_id: self.aggregate_id.0,
            aggregate_type: self.aggregate_type.clone(),
            actor_id,
            correlation_id,
            causation_id,
            occurred_at,
            recorded_at,
            payload,
            active_status,
        }
    }
}

/// The SurrealDB-backed `EventLog` implementation.
#[derive(Clone)]
pub struct SurrealEventLog {
    pub(crate) db: Db,
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealEventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealEventLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SurrealEventLog {
    /// Constructs a new event log handle bound to `db` and
    /// scoped to `school`.
    pub fn new(db: Db, school: SchoolId) -> Self {
        Self { db, school }
    }
}

#[async_trait]
impl EventLog for SurrealEventLog {
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        let row = EventRow::from_entry(&entry);
        let _ = self
            .db
            .query(
                "INSERT INTO event_log { \
                    event_id: $event_id, \
                    school_id: $school_id, \
                    event_type: $event_type, \
                    schema_version: $schema_version, \
                    aggregate_id: $aggregate_id, \
                    aggregate_type: $aggregate_type, \
                    actor_id: $actor_id, \
                    correlation_id: $correlation_id, \
                    causation_id: $causation_id, \
                    occurred_at: $occurred_at, \
                    recorded_at: $recorded_at, \
                    payload: $payload, \
                    active_status: $active_status \
                }",
            )
            .bind(("event_id", row.event_id))
            .bind(("school_id", row.school_id))
            .bind(("event_type", row.event_type))
            .bind(("schema_version", row.schema_version))
            .bind(("aggregate_id", row.aggregate_id))
            .bind(("aggregate_type", row.aggregate_type))
            .bind(("actor_id", row.actor_id))
            .bind(("correlation_id", row.correlation_id))
            .bind(("causation_id", row.causation_id))
            .bind(("occurred_at", row.occurred_at))
            .bind(("recorded_at", row.recorded_at))
            .bind(("payload", row.payload))
            .bind(("active_status", row.active_status))
            .await
            .map_err(|e| StringError(format!("event_log append: {e}")))?;
        Ok(())
    }

    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let school_uuid = SurrealUuid::from(filter.school_id.as_uuid());
        let type_filter = if filter.event_types.is_empty() {
            String::from("true")
        } else {
            let types = filter
                .event_types
                .iter()
                .map(|t| format!("'{t}'"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("event_type IN [{types}]")
        };
        let since_clause = filter
            .since
            .as_ref()
            .map(|s| format!(" AND recorded_at >= datetime::from('{}')", s.as_datetime().to_rfc3339()))
            .unwrap_or_default();
        let until_clause = filter
            .until
            .as_ref()
            .map(|u| format!(" AND recorded_at < datetime::from('{}')", u.as_datetime().to_rfc3339()))
            .unwrap_or_default();
        let agg_clause = filter
            .aggregate_id
            .map(|a| format!(" AND aggregate_id = SurrealUuid::from('{}')", a))
            .unwrap_or_default();
        let query = format!(
            "SELECT event_id, school_id, event_type, schema_version, aggregate_id, \
                    aggregate_type, actor_id, correlation_id, causation_id, occurred_at, \
                    recorded_at, payload, active_status \
             FROM event_log \
             WHERE school_id = $school AND {type_filter}{since_clause}{until_clause}{agg_clause} \
             ORDER BY recorded_at ASC \
             LIMIT $limit"
        );
        let mut response = self
            .db
            .query(&query)
            .bind(("school", school_uuid))
            .bind(("limit", i64::from(filter.limit)))
            .await
            .map_err(|e| StringError(format!("event_log read: {e}")))?;
        let rows: Vec<EventRow> = response
            .take(0)
            .map_err(|e| StringError(format!("event_log read take: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|row| EventRow::to_entry(&row))
            .collect())
    }

    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let school_uuid = SurrealUuid::from(filter.school_id.as_uuid());
        let type_filter = if filter.event_types.is_empty() {
            String::from("true")
        } else {
            let types = filter
                .event_types
                .iter()
                .map(|t| format!("'{t}'"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("event_type IN [{types}]")
        };
        let since_clause = filter
            .since
            .as_ref()
            .map(|s| format!(" AND recorded_at >= datetime::from('{}')", s.as_datetime().to_rfc3339()))
            .unwrap_or_default();
        let until_clause = filter
            .until
            .as_ref()
            .map(|u| format!(" AND recorded_at < datetime::from('{}')", u.as_datetime().to_rfc3339()))
            .unwrap_or_default();
        let query = format!(
            "SELECT count() AS n FROM event_log \
             WHERE school_id = $school AND {type_filter}{since_clause}{until_clause} \
             GROUP ALL"
        );
        let mut response = self
            .db
            .query(&query)
            .bind(("school", school_uuid))
            .await
            .map_err(|e| StringError(format!("event_log count: {e}")))?;
        #[derive(serde::Deserialize)]
        struct CountRow {
            n: i64,
        }
        let rows: Vec<CountRow> = response
            .take(0)
            .map_err(|e| StringError(format!("event_log count take: {e}")))?;
        Ok(rows.first().map(|r| r.n as u64).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::SchoolId;
    use educore_storage::StorageAdapter;

    async fn setup(school: SchoolId) -> SurrealEventLog {
        let adapter = crate::storage::SurrealStorageAdapter::in_memory(school)
            .await
            .expect("in-memory adapter should construct");
        adapter.migrate().await.expect("migration should succeed");
        SurrealEventLog::new(adapter.db().clone(), school)
    }

    fn sample_entry(school: SchoolId) -> EventLogEntry {
        let g = SystemIdGen;
        EventLogEntry {
            event_id: g.next_event_id(),
            school_id: school,
            event_type: "academic.student.admitted".to_owned(),
            schema_version: 1,
            aggregate_id: g.next_uuid(),
            aggregate_type: "student".to_owned(),
            actor_id: g.next_user_id(),
            correlation_id: g.next_correlation_id(),
            causation_id: None,
            occurred_at: Timestamp::now(),
            recorded_at: Timestamp::now(),
            payload: Bytes::from_static(b"{}"),
            active_status: ActiveStatus::Active,
        }
    }

    #[tokio::test]
    async fn append_then_read_for_school_round_trips() {
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let log = setup(school).await;
        let entry = sample_entry(school);
        log.append(entry.clone()).await.unwrap();
        let filter = EventLogFilter::for_school(school);
        let rows = log.read(filter).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_id, entry.event_id);
    }

    #[tokio::test]
    async fn read_filters_by_event_type() {
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let log = setup(school).await;
        let mut e1 = sample_entry(school);
        e1.event_type = "academic.student.admitted".to_owned();
        let mut e2 = sample_entry(school);
        e2.event_type = "academic.student.transferred".to_owned();
        log.append(e1).await.unwrap();
        log.append(e2).await.unwrap();
        let mut filter = EventLogFilter::for_school(school);
        filter.event_types = vec!["academic.student.admitted".to_owned()];
        let rows = log.read(filter).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].event_type, "academic.student.admitted");
    }

    #[tokio::test]
    async fn read_respects_limit() {
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let log = setup(school).await;
        for _ in 0..5 {
            log.append(sample_entry(school)).await.unwrap();
        }
        let mut filter = EventLogFilter::for_school(school);
        filter.limit = 3;
        let rows = log.read(filter).await.unwrap();
        assert_eq!(rows.len(), 3);
    }

    #[tokio::test]
    async fn count_returns_matching_count() {
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let log = setup(school).await;
        for _ in 0..4 {
            log.append(sample_entry(school)).await.unwrap();
        }
        let filter = EventLogFilter::for_school(school);
        let n = log.count(filter).await.unwrap();
        assert_eq!(n, 4);
    }
}
