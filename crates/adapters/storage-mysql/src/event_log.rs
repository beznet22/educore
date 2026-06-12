//! MySQL-backed `EventLog` sub-port.
//!
//! Stores each `EventLogEntry` as a row in the `event_log`
//! table. The schema is defined by the canonical
//! `migrations/engine/0000_engine_core.mysql.sql` migration
//! loaded by `MysqlStorageAdapter::migrate`.
//!
//! ## Filter handling
//!
//! The `EventLogFilter` has four optional predicates
//! (`event_types`, `since`, `until`, `aggregate_id`). The
//! `read` and `count` methods build the `WHERE` clause
//! dynamically with a `sqlx::QueryBuilder`; the optional
//! clauses are emitted only when the corresponding filter
//! field is set, which keeps the prepared plan cheap for the
//! common "all events for a school" case.
//!
//! ## `IN (?, ?, ...)` for `event_types`
//!
//! Unlike the PostgreSQL adapter (which uses `event_type =
//! ANY($N)` with a `Vec<String>`), this adapter expands the
//! `IN` clause manually via `QueryBuilder::separated`. MySQL's
//! `sqlx` driver does not implement `Encode<MySql>` for
//! `Vec<T>` (only `Vec<u8>`), so the multi-bind path is
//! built into the SQL string at query time. The bind
//! semantics are equivalent to PostgreSQL's `ANY(?)`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{FromRow, MySqlPool, QueryBuilder};
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

/// The MySQL-backed `EventLog` implementation.
#[derive(Clone)]
pub struct MysqlEventLog {
    pool: MySqlPool,
    #[allow(dead_code)]
    school: SchoolId,
}

impl std::fmt::Debug for MysqlEventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MysqlEventLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl MysqlEventLog {
    /// Constructs a new event-log handle bound to `pool` and
    /// scoped to `school`. The `school` field is reserved for
    /// future per-connection filtering; the trait's methods
    /// take a `school_id` argument and use that.
    #[must_use]
    pub fn new(pool: MySqlPool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl EventLog for MysqlEventLog {
    #[instrument(skip(self, entry), fields(event_id = %entry.event_id))]
    async fn append(&self, entry: EventLogEntry) -> Result<()> {
        sqlx::query::<sqlx::MySql>(
            "INSERT INTO `event_log` (\
                `event_id`, `event_type`, `event_version`, `school_id`, \
                `aggregate_id`, `aggregate_type`, `actor_id`, \
                `correlation_id`, `causation_id`, `occurred_at`, \
                `recorded_at`, `payload`\
            ) VALUES (\
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?\
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
        let mut qb: QueryBuilder<sqlx::MySql> = build_read_query(&filter, false);
        let rows: Vec<EventLogRow> = qb
            .build_query_as::<EventLogRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        let entries = rows.into_iter().map(EventLogRow::into_entry).collect();
        Ok(entries)
    }

    #[instrument(skip(self, filter))]
    async fn count(&self, filter: EventLogFilter) -> Result<u64> {
        let mut qb: QueryBuilder<sqlx::MySql> = build_read_query(&filter, true);
        // `QueryAs<(i64,)>` extracts the first column of the
        // first row from the `SELECT COUNT(*)` query.
        let row: (i64,) = qb
            .build_query_as::<(i64,)>()
            .fetch_one(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(u64::try_from(row.0).unwrap_or(0))
    }
}

/// Builds the `SELECT ... FROM event_log WHERE ...` prefix
/// shared by `read` and `count`. The optional clauses are
/// emitted only when the corresponding filter field is set;
/// this keeps the prepared plan cheap for the common "all
/// events for a school" case.
fn build_read_query(filter: &EventLogFilter, count_only: bool) -> QueryBuilder<'_, sqlx::MySql> {
    let mut qb: QueryBuilder<'_, sqlx::MySql> = if count_only {
        QueryBuilder::new("SELECT COUNT(*) FROM `event_log` WHERE `school_id` = ")
    } else {
        QueryBuilder::new(
            "SELECT `event_id`, `event_type`, `event_version`, `school_id`, \
             `aggregate_id`, `aggregate_type`, `actor_id`, \
             `correlation_id`, `causation_id`, `occurred_at`, \
             `recorded_at`, `payload` \
             FROM `event_log` WHERE `school_id` = ",
        )
    };
    qb.push_bind(filter.school_id.as_uuid());
    if !filter.event_types.is_empty() {
        // MySQL's sqlx driver does not implement
        // `Encode<MySql>` for `Vec<String>`; expand the `IN`
        // clause at SQL-build time with a `separated` cursor.
        qb.push(" AND `event_type` IN (");
        let mut sep = qb.separated(", ");
        for t in &filter.event_types {
            sep.push_bind(t);
        }
        sep.push_unseparated(")");
    }
    if let Some(agg_id) = filter.aggregate_id {
        qb.push(" AND `aggregate_id` = ").push_bind(agg_id);
    }
    if let Some(since) = filter.since {
        qb.push(" AND `recorded_at` >= ")
            .push_bind(since.as_datetime());
    }
    if let Some(until) = filter.until {
        qb.push(" AND `recorded_at` < ")
            .push_bind(until.as_datetime());
    }
    if !count_only {
        qb.push(" ORDER BY `recorded_at` ASC LIMIT ")
            .push_bind(i64::from(filter.limit));
    }
    qb
}
