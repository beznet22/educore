//! SQLite-backed `Outbox` sub-port.
//!
//! Stores each envelope as a row in the `outbox` table. The
//! schema is defined by the canonical .sql migration (loaded
//! by `SqliteStorageAdapter::migrate`).
//!
//! ## UUID encoding
//!
//! The schema stores UUIDs as `TEXT` columns with a
//! `CHECK (length(x) = 36)` invariant. sqlx 0.8's
//! `uuid::Uuid` impl for SQLite encodes as `BLOB` (16 bytes),
//! which would fail the length check. We therefore bind and
//! decode UUIDs as `uuid::fmt::Hyphenated` (the 36-char
//! hyphenated text form). See
//! `docs/schemas/sql-dialects/sqlite.md` for the canonical
//! wire format.
//!
//! Tenant isolation: `pending` / `pending_count` always filter
//! by `school_id = ?`. `append` writes the row's `school_id`
//! from the envelope (caller-supplied); `mark_published`
//! updates by `event_id` only, matching the SurrealDB
//! Phase 0 pattern (the caller is expected to pass event_ids
//! it received from a previous `pending` call on the same
//! school).

use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::trace;
use uuid::fmt::Hyphenated;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_storage::outbox::{Outbox, SerializedEnvelope};

use crate::error::StringError;
use crate::util::{bytes_to_json, json_to_bytes};

/// The row shape stored in the SQLite `outbox` table. Mirrors
/// the column types in the canonical DDL. `#[derive(FromRow)]`
/// is the typed mapping used by `query_as` on `pending`.
#[derive(sqlx::FromRow)]
#[allow(dead_code)] // `recorded_at`, `enqueued_at`, `published_at`, `attempts`, `last_error` are read for future parity tests.
struct OutboxRow {
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
    enqueued_at: DateTime<Utc>,
    published_at: Option<DateTime<Utc>>,
    attempts: i32,
    last_error: Option<String>,
}

impl OutboxRow {
    /// Maps a row back to a `SerializedEnvelope`. The row is
    /// borrowed (not consumed) so callers can iterate over the
    /// query result without cloning each row.
    fn to_envelope(&self) -> SerializedEnvelope {
        SerializedEnvelope {
            event_id: EventId::from_uuid(*self.event_id.as_uuid()),
            event_type: self.event_type.clone(),
            // Mirrors the SurrealDB impl's `try_from(...).unwrap_or(0)`:
            // any negative `event_version` is clamped to `0` on read.
            schema_version: u32::try_from(self.event_version).unwrap_or(0),
            school_id: SchoolId::from_uuid(*self.school_id.as_uuid()),
            aggregate_id: *self.aggregate_id.as_uuid(),
            aggregate_type: self.aggregate_type.clone(),
            actor_id: UserId::from_uuid(*self.actor_id.as_uuid()),
            correlation_id: CorrelationId::from_uuid(*self.correlation_id.as_uuid()),
            causation_id: self.causation_id.map(|u| EventId::from_uuid(*u.as_uuid())),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            payload: json_to_bytes(&self.payload.0),
        }
    }
}

/// The SQLite-backed `Outbox` implementation.
#[derive(Clone)]
pub struct SqliteOutbox {
    pool: SqlitePool,
    school: SchoolId,
}

impl fmt::Debug for SqliteOutbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteOutbox")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteOutbox {
    /// Constructs a new outbox handle bound to `pool` and
    /// filtered to `school`.
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        Self { pool, school }
    }

    /// Returns the school the outbox is filtered to.
    pub fn school(&self) -> SchoolId {
        self.school
    }
}

/// Extension trait: `Hyphenated::into_uuid()` consumes the
/// borrowed-text wrapper and returns the inner `uuid::Uuid`.
/// Mirrors the SurrealDB outbox's `from_surreal_datetime`
/// helper.
#[allow(dead_code)]
pub(crate) trait IntoUuid {
    fn into_uuid(self) -> uuid::Uuid;
}

#[allow(dead_code)]
impl IntoUuid for Hyphenated {
    fn into_uuid(self) -> uuid::Uuid {
        // `Hyphenated` is a `Copy` wrapper around `uuid::Uuid`
        // in the `uuid` crate, so dereferencing yields the
        // inner value without consuming.
        *self.as_uuid()
    }
}

#[async_trait]
impl Outbox for SqliteOutbox {
    async fn append(&self, envelope: SerializedEnvelope) -> Result<()> {
        // SurrealDB outbox pattern: `u32` -> `i32` for the
        // SQLite INTEGER column, with a default of `0` on
        // overflow. Schema versions are small positive
        // integers in practice; the default only fires on
        // pathological input.
        let event_version = i32::try_from(envelope.schema_version).unwrap_or(0);
        let payload_json = sqlx::types::Json(bytes_to_json(&envelope.payload));
        let now = Utc::now();
        sqlx::query::<sqlx::Sqlite>(
            "INSERT INTO outbox ( \
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, \
                recorded_at, payload, enqueued_at, \
                published_at, attempts, last_error \
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, 0, NULL)",
        )
        .bind(envelope.event_id.as_uuid().hyphenated())
        .bind(&envelope.event_type)
        .bind(event_version)
        .bind(envelope.school_id.as_uuid().hyphenated())
        .bind(envelope.aggregate_id.hyphenated())
        .bind(&envelope.aggregate_type)
        .bind(envelope.actor_id.as_uuid().hyphenated())
        .bind(envelope.correlation_id.as_uuid().hyphenated())
        .bind(envelope.causation_id.map(|c| c.as_uuid().hyphenated()))
        .bind(envelope.occurred_at.as_datetime())
        .bind(envelope.occurred_at.as_datetime())
        .bind(payload_json)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| StringError(format!("outbox append: {e}")))?;
        trace!(event_id = %envelope.event_id, "outbox append");
        Ok(())
    }

    async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
        let rows: Vec<OutboxRow> = sqlx::query_as::<sqlx::Sqlite, OutboxRow>(
            "SELECT \
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, \
                recorded_at, payload, enqueued_at, \
                published_at, attempts, last_error \
             FROM outbox \
             WHERE school_id = ?1 AND published_at IS NULL \
             ORDER BY enqueued_at ASC \
             LIMIT ?2",
        )
        .bind(self.school.as_uuid().hyphenated())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StringError(format!("outbox pending: {e}")))?;
        Ok(rows.iter().map(OutboxRow::to_envelope).collect())
    }

    async fn mark_published(&self, ids: &[EventId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        // sqlx 0.8 SQLite does not implement `Encode<Sqlite>`
        // for `Vec<Hyphenated>` (only `Vec<u8>`), so we
        // build the `IN (?, ?, ...)` clause dynamically with
        // a `QueryBuilder`. This mirrors the event_log
        // read path.
        let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
            "UPDATE outbox \
             SET published_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE event_id IN (",
        );
        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(id.as_uuid().hyphenated());
        }
        qb.push(")");
        let result = qb
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| StringError(format!("outbox mark_published: {e}")))?;
        trace!(rows = result.rows_affected(), "outbox mark_published");
        Ok(())
    }

    async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
        let count: i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
            "SELECT COUNT(*) FROM outbox \
             WHERE school_id = ?1 AND published_at IS NULL",
        )
        .bind(school_id.as_uuid().hyphenated())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| StringError(format!("outbox pending_count: {e}")))?;
        u64::try_from(count).map_err(|e| {
            DomainError::infrastructure(StringError(format!(
                "outbox pending_count: count overflow: {e}"
            )))
        })
    }
}

// Suppress dead-code warning on `Bytes` import; the type is
// re-exported via `SerializedEnvelope::payload` and used by
// the `to_envelope` round-trip path.
#[allow(dead_code)]
const _: fn() = || {
    let _b: Bytes = Bytes::new();
};
