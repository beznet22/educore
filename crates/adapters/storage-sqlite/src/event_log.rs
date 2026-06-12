//! SQLite-backed `EventLog` sub-port.
//!
//! Stores each entry as a row in the `event_log` table. The
//! schema is defined by the canonical .sql migration (loaded
//! by `SqliteStorageAdapter::migrate`).
//!
//! Tenant isolation: every read/count query filters by
//! `school_id = ?`. The `event_types`, `aggregate_id`,
//! `since`, and `until` filters are optional and use a
//! dynamically-constructed `WHERE` clause via
//! `sqlx::QueryBuilder`.

use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use tracing::trace;
use uuid::fmt::Hyphenated;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};

use crate::error::StringError;
use crate::util::{bytes_to_json, json_to_bytes};

/// The row shape stored in the SQLite `event_log` table.
#[derive(sqlx::FromRow)]
struct EventLogRow {
    event_id: Hyphenated,
    event_type: String,
    event_version: i32,
    school_id: Hyphenated,
    aggregate_id: Hyphenated,
    aggregate_type: String,
    actor_id: Hyphenated,
    correlation_id: Hyphenated,
    causation_id: Option<Hyphenated>,
    occurred_at: DateTime<Utc>,
    recorded_at: DateTime<Utc>,
    payload: sqlx::types::Json<serde_json::Value>,
}

impl EventLogRow {
    /// Maps a row back to an `EventLogEntry`. The
    /// `active_status` field has no schema column (see
    /// `event_log` DDL); reads always return `Active`.
    fn to_entry(&self) -> EventLogEntry {
        EventLogEntry {
            event_id: EventId::from_uuid(*self.event_id.as_uuid()),
            school_id: SchoolId::from_uuid(*self.school_id.as_uuid()),
            event_type: self.event_type.clone(),
            schema_version: u32::try_from(self.event_version).unwrap_or(0),
            aggregate_id: *self.aggregate_id.as_uuid(),
            aggregate_type: self.aggregate_type.clone(),
            actor_id: UserId::from_uuid(*self.actor_id.as_uuid()),
            correlation_id: CorrelationId::from_uuid(*self.correlation_id.as_uuid()),
            causation_id: self.causation_id.map(|u| EventId::from_uuid(*u.as_uuid())),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            recorded_at: Timestamp::from_datetime(self.recorded_at),
            payload: json_to_bytes(&self.payload.0),
            active_status: ActiveStatus::Active,
        }
    }
}

/// The SQLite-backed `EventLog` implementation.
#[derive(Clone)]
pub struct SqliteEventLog {
    pool: SqlitePool,
    school: SchoolId,
}

impl fmt::Debug for SqliteEventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteEventLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteEventLog {
    /// Constructs a new event-log handle bound to `pool` and
    /// scoped to `school`.
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl EventLog for SqliteEventLog {
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        let event_version = i32::try_from(entry.schema_version).unwrap_or(0);
        let payload_json = sqlx::types::Json(bytes_to_json(&entry.payload));
        sqlx::query::<sqlx::Sqlite>(
            "INSERT INTO event_log ( \
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, \
                recorded_at, payload \
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(entry.event_id.as_uuid().hyphenated())
        .bind(&entry.event_type)
        .bind(event_version)
        .bind(entry.school_id.as_uuid().hyphenated())
        .bind(entry.aggregate_id.hyphenated())
        .bind(&entry.aggregate_type)
        .bind(entry.actor_id.as_uuid().hyphenated())
        .bind(entry.correlation_id.as_uuid().hyphenated())
        .bind(entry.causation_id.map(|c| c.as_uuid().hyphenated()))
        .bind(entry.occurred_at.as_datetime())
        .bind(entry.recorded_at.as_datetime())
        .bind(payload_json)
        .execute(&self.pool)
        .await
        .map_err(|e| StringError(format!("event_log append: {e}")))?;
        trace!(event_id = %entry.event_id, "event_log append");
        Ok(())
    }

    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let mut qb: QueryBuilder<Sqlite> = build_read_query(&filter, false);
        let rows: Vec<EventLogRow> = qb
            .build_query_as::<EventLogRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StringError(format!("event_log read: {e}")))?;
        Ok(rows.iter().map(EventLogRow::to_entry).collect())
    }

    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let mut qb: QueryBuilder<Sqlite> = build_read_query(&filter, true);
        // `QueryAs<(i64,)>` extracts the first column of the
        // first row from the `SELECT COUNT(*)` query.
        let row: (i64,) = qb
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StringError(format!("event_log count: {e}")))?;
        u64::try_from(row.0).map_err(|e| {
            DomainError::infrastructure(StringError(format!(
                "event_log count: count overflow: {e}"
            )))
        })
    }
}

/// Builds the `SELECT ... FROM event_log WHERE ...` prefix
/// shared by `read` and `count`. The optional clauses are
/// emitted only when the corresponding filter field is set;
/// this keeps the prepared plan cheap for the common "all
/// events for a school" case.
fn build_read_query<'a>(filter: &'a EventLogFilter, count_only: bool) -> QueryBuilder<'a, Sqlite> {
    let mut qb: QueryBuilder<'a, Sqlite> = if count_only {
        QueryBuilder::new("SELECT COUNT(*) FROM event_log WHERE school_id = ")
    } else {
        QueryBuilder::new(
            "SELECT event_id, event_type, event_version, school_id, \
             aggregate_id, aggregate_type, actor_id, \
             correlation_id, causation_id, occurred_at, \
             recorded_at, payload \
             FROM event_log WHERE school_id = ",
        )
    };
    qb.push_bind(filter.school_id.as_uuid().hyphenated());
    if !filter.event_types.is_empty() {
        qb.push(" AND event_type IN (");
        let mut sep = qb.separated(", ");
        for t in &filter.event_types {
            sep.push_bind(t);
        }
        sep.push_unseparated(")");
    }
    if let Some(agg_id) = filter.aggregate_id {
        qb.push(" AND aggregate_id = ")
            .push_bind(agg_id.hyphenated());
    }
    if let Some(since) = filter.since {
        qb.push(" AND recorded_at >= ")
            .push_bind(since.as_datetime());
    }
    if let Some(until) = filter.until {
        qb.push(" AND recorded_at < ")
            .push_bind(until.as_datetime());
    }
    if !count_only {
        qb.push(" ORDER BY recorded_at ASC LIMIT ")
            .push_bind(i64::from(filter.limit));
    }
    qb
}
