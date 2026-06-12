//! PostgreSQL-backed `EventLog` sub-port.
//!
//! Stores each `EventLogEntry` as a row in the
//! `engine.event_log` table. The schema is defined by the
//! canonical `migrations/engine/0000_engine_core.postgres.sql`
//! migration loaded by `PostgresStorageAdapter::migrate`.
//!
//! ## Filter handling
//!
//! The `EventLogFilter` has four optional predicates
//! (`event_types`, `since`, `until`, `aggregate_id`). The
//! `read` and `count` methods build the `WHERE` clause
//! dynamically based on which predicates are `Some` (or, for
//! `event_types`, non-empty). The dynamic SQL is built by a
//! `String` concatenation against a typed parameter vector;
//! the parameters are bound positionally so no `format!`
//! interpolation is performed on user input.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Row};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};

use crate::connection_helpers::{bytes_to_json_value, json_value_to_bytes};

/// The row shape read out of the `event_log` table.
#[derive(Debug, FromRow)]
struct EventLogRow {
    event_id: Uuid,
    event_type: String,
    event_version: i32,
    school_id: Uuid,
    aggregate_id: Uuid,
    aggregate_type: String,
    actor_id: Uuid,
    correlation_id: Uuid,
    causation_id: Option<Uuid>,
    occurred_at: DateTime<Utc>,
    recorded_at: DateTime<Utc>,
    payload: Json<Value>,
}

impl EventLogRow {
    /// Maps a database row back to an `EventLogEntry`. The DDL
    /// does not carry an `active_status` column; the engine
    /// exposes one for forward compatibility with a retire
    /// operation, so the adapter returns `Active` on read.
    fn into_entry(self) -> EventLogEntry {
        EventLogEntry {
            event_id: EventId::from_uuid(self.event_id),
            school_id: SchoolId::from_uuid(self.school_id),
            event_type: self.event_type,
            // `event_version` is `INT` in the DDL and `u32` in
            // the engine type. The schema versions are small
            // positive integers, so the conversion is total in
            // practice; we use `try_from(...).unwrap_or(0)` to
            // satisfy the workspace's `cast_possible_wrap` /
            // `cast_sign_loss` deny lints.
            schema_version: u32::try_from(self.event_version).unwrap_or(0),
            aggregate_id: self.aggregate_id,
            aggregate_type: self.aggregate_type,
            actor_id: UserId::from_uuid(self.actor_id),
            correlation_id: CorrelationId::from_uuid(self.correlation_id),
            causation_id: self.causation_id.map(EventId::from_uuid),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            recorded_at: Timestamp::from_datetime(self.recorded_at),
            payload: json_value_to_bytes(&self.payload.0),
            active_status: ActiveStatus::Active,
        }
    }
}

/// The PostgreSQL-backed `EventLog` implementation.
#[derive(Clone)]
pub struct PostgresEventLog {
    pool: PgPool,
    #[allow(dead_code)]
    school: SchoolId,
}

impl std::fmt::Debug for PostgresEventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresEventLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl PostgresEventLog {
    /// Constructs a new event-log handle bound to `pool` and
    /// scoped to `school`. The `school` field is reserved for
    /// future per-connection filtering; the trait's methods
    /// take a `school_id` argument and use that.
    #[must_use]
    pub fn new(pool: PgPool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

/// Builds the `SELECT` statement and the parameter vector for an
/// `event_log` query that respects the given filter. Returns
/// the SQL string and a `Vec<Box<dyn Encode>>`-style parameter
/// vector. We use `sqlx::query_as` with a `Query` builder here
/// for simplicity; the parameter list is built by appending
/// each filter element to the same query.
fn build_select(
    select_columns: &'static str,
    from_clause: &'static str,
    filter: &EventLogFilter,
) -> (String, Vec<FilterParam>) {
    // We build a `String` SQL plus a parallel `Vec<FilterParam>`
    // so the caller can bind the parameters positionally. The
    // SQL is built from a fixed template; the only user
    // input that ever lands in the string is the comparison
    // operators (`=`, `>=`, `<`, `ANY($N)`) and column names,
    // which are hard-coded. Values are bound, not interpolated.
    let mut sql = String::with_capacity(256);
    sql.push_str("SELECT ");
    sql.push_str(select_columns);
    sql.push(' ');
    sql.push_str(from_clause);
    sql.push_str(" WHERE school_id = $1");
    let mut params: Vec<FilterParam> = Vec::with_capacity(8);
    params.push(FilterParam::Uuid(filter.school_id.as_uuid()));
    if !filter.event_types.is_empty() {
        params.push(FilterParam::StrVec(filter.event_types.clone()));
        sql.push_str(" AND event_type = ANY($");
        // append the next index (params.len() before this push + 1)
        let idx = params.len();
        // We need to write the index without `format!` to keep
        // the build clippy-clean. Push the digit chars one by
        // one.
        let idx_str = idx.to_string();
        sql.push_str(&idx_str);
        sql.push(')');
    }
    if let Some(agg_id) = filter.aggregate_id {
        params.push(FilterParam::Uuid(agg_id));
        sql.push_str(" AND aggregate_id = $");
        let idx_str = params.len().to_string();
        sql.push_str(&idx_str);
    }
    if let Some(since) = filter.since {
        params.push(FilterParam::Timestamp(since.as_datetime()));
        sql.push_str(" AND recorded_at >= $");
        let idx_str = params.len().to_string();
        sql.push_str(&idx_str);
    }
    if let Some(until) = filter.until {
        params.push(FilterParam::Timestamp(until.as_datetime()));
        sql.push_str(" AND recorded_at < $");
        let idx_str = params.len().to_string();
        sql.push_str(&idx_str);
    }
    (sql, params)
}

/// One parameter in a dynamic `event_log` filter query. The
/// variants are bound in the same order they were added to
/// the `WHERE` clause by `build_select`.
enum FilterParam {
    Uuid(Uuid),
    StrVec(Vec<String>),
    Timestamp(DateTime<Utc>),
}

const SELECT_COLUMNS: &str = "\
    event_id, event_type, event_version, school_id, \
    aggregate_id, aggregate_type, actor_id, correlation_id, \
    causation_id, occurred_at, recorded_at, payload";
const FROM_CLAUSE: &str = "FROM event_log";

#[async_trait]
impl EventLog for PostgresEventLog {
    #[instrument(skip(self, entry), fields(event_id = %entry.event_id))]
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO event_log (\
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, \
                recorded_at, payload\
            ) VALUES (\
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12\
            )",
        )
        .bind(entry.event_id.as_uuid())
        .bind(&entry.event_type)
        .bind(i32::try_from(entry.schema_version).unwrap_or(0))
        .bind(entry.school_id.as_uuid())
        .bind(entry.aggregate_id)
        .bind(&entry.aggregate_type)
        .bind(entry.actor_id.as_uuid())
        .bind(entry.correlation_id.as_uuid())
        .bind(entry.causation_id.map(|c| c.as_uuid()))
        .bind(entry.occurred_at.as_datetime())
        .bind(entry.recorded_at.as_datetime())
        .bind(Json(bytes_to_json_value(&entry.payload)))
        .execute(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self, filter))]
    async fn read(&self, filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        let (mut sql, params) = build_select(SELECT_COLUMNS, FROM_CLAUSE, &filter);
        // Append the ORDER BY and LIMIT, with the LIMIT index
        // computed as `params.len() + 1` (LIMIT is the last
        // positional parameter).
        let limit_idx = params.len() + 1;
        let limit_idx_str = limit_idx.to_string();
        sql.push_str(" ORDER BY recorded_at ASC LIMIT $");
        sql.push_str(&limit_idx_str);
        // Apply the limit
        let limit_i64 = i64::from(filter.limit);
        let mut query = sqlx::query_as::<_, EventLogRow>(&sql);
        for p in &params {
            query = match p {
                FilterParam::Uuid(u) => query.bind(*u),
                FilterParam::StrVec(v) => query.bind(v),
                FilterParam::Timestamp(t) => query.bind(*t),
            };
        }
        query = query.bind(limit_i64);
        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        let entries = rows.into_iter().map(EventLogRow::into_entry).collect();
        Ok(entries)
    }

    #[instrument(skip(self, filter))]
    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let (sql, params) = build_select("COUNT(*) AS n", FROM_CLAUSE, &filter);
        let mut query = sqlx::query(&sql);
        for p in &params {
            query = match p {
                FilterParam::Uuid(u) => query.bind(*u),
                FilterParam::StrVec(v) => query.bind(v),
                FilterParam::Timestamp(t) => query.bind(*t),
            };
        }
        let row = query
            .fetch_one(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        let n: i64 = row
            .try_get("n")
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(u64::try_from(n).unwrap_or(0))
    }
}
