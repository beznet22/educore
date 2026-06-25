//! PostgreSQL-backed `Outbox` sub-port.
//!
//! Stores each envelope as a row in the `engine.outbox` table.
//! The schema is defined by the canonical
//! `migrations/engine/0000_engine_core.postgres.sql` migration
//! loaded by `PostgresStorageAdapter::migrate`.
//!
//! ## Per-call transaction
//!
//! Per the design note in `crate::transaction`, every method on
//! this struct opens a fresh `pool.begin()` and commits on
//! drop. The sub-port is `Clone` (the `PgPool` is cheaply
//! cloneable), so the `PostgresTransaction` can hand out
//! `&self` references without lifetime issues.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{FromRow, PgPool, Row};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_storage::outbox::{Outbox, SerializedEnvelope};

use crate::connection_helpers::{bytes_to_json_value, json_value_to_bytes};

/// The row shape read out of the `outbox` table. Field types
/// mirror the PostgreSQL column types per
/// `docs/schemas/sql-dialects/postgresql.md` § "Type mapping".
#[derive(Debug, FromRow)]
struct OutboxRow {
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
    payload: Json<Value>,
}

impl OutboxRow {
    /// Maps a database row back to a `SerializedEnvelope`.
    fn into_envelope(self) -> SerializedEnvelope {
        SerializedEnvelope {
            event_id: EventId::from_uuid(self.event_id),
            event_type: self.event_type,
            // `event_version` is `INT` in the DDL and `u32` in
            // the engine type. The schema versions are small
            // positive integers, so the conversion is total in
            // practice; we use `try_from(...).unwrap_or(0)` to
            // satisfy the workspace's `cast_possible_wrap` /
            // `cast_sign_loss` deny lints.
            schema_version: u32::try_from(self.event_version).unwrap_or(0),
            school_id: SchoolId::from_uuid(self.school_id),
            aggregate_id: self.aggregate_id,
            aggregate_type: self.aggregate_type,
            actor_id: UserId::from_uuid(self.actor_id),
            correlation_id: CorrelationId::from_uuid(self.correlation_id),
            causation_id: self.causation_id.map(EventId::from_uuid),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            payload: json_value_to_bytes(&self.payload.0),
        }
    }
}

/// The PostgreSQL-backed `Outbox` implementation.
#[derive(Clone)]
pub struct PostgresOutbox {
    pool: PgPool,
    school: SchoolId,
}

impl std::fmt::Debug for PostgresOutbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresOutbox")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl PostgresOutbox {
    /// Constructs a new outbox handle bound to `pool` and
    /// scoped to `school`.
    #[must_use]
    pub fn new(pool: PgPool, school: SchoolId) -> Self {
        Self { pool, school }
    }

    /// Returns the school the outbox is scoped to.
    #[must_use]
    pub fn school(&self) -> SchoolId {
        self.school
    }
}

#[async_trait]
impl Outbox for PostgresOutbox {
    #[instrument(skip(self, envelope), fields(event_id = %envelope.event_id))]
    async fn append(
        &self,
        _school_id: educore_core::ids::SchoolId,
        envelope: SerializedEnvelope,
    ) -> Result<()> {
        // `enqueued_at` and `published_at` are handled at the DB
        // level (`NOW()` and `NULL` respectively) per the
        // canonical DDL. `attempts` defaults to 0,
        // `last_error` is NULL.
        sqlx::query(
            "INSERT INTO outbox (\
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, \
                recorded_at, payload, enqueued_at, published_at, \
                attempts, last_error\
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NULL, 0, NULL)",
        )
        .bind(envelope.event_id.as_uuid())
        .bind(&envelope.event_type)
        .bind(i32::try_from(envelope.schema_version).unwrap_or(0))
        .bind(envelope.school_id.as_uuid())
        .bind(envelope.aggregate_id)
        .bind(&envelope.aggregate_type)
        .bind(envelope.actor_id.as_uuid())
        .bind(envelope.correlation_id.as_uuid())
        .bind(envelope.causation_id.map(|c| c.as_uuid()))
        .bind(envelope.occurred_at.as_datetime())
        .bind(envelope.occurred_at.as_datetime())
        .bind(Json(bytes_to_json_value(&envelope.payload)))
        .execute(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pending(
        &self,
        _school_id: educore_core::ids::SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>> {
        // QW-13 / school-partitioning contract: the `WHERE
        // school_id = $1` predicate is the engine's tenant-isolation
        // guarantee for this handle. `$1` is bound to
        // `self.school` (the handle's scope) — never to a
        // caller-supplied value — so cross-tenant leakage is
        // impossible at this method.
        let rows: Vec<OutboxRow> = sqlx::query_as::<_, OutboxRow>(
            "SELECT \
                event_id, event_type, event_version, school_id, \
                aggregate_id, aggregate_type, actor_id, \
                correlation_id, causation_id, occurred_at, payload \
             FROM outbox \
             WHERE school_id = $1 AND published_at IS NULL \
             ORDER BY enqueued_at ASC \
             LIMIT $2",
        )
        .bind(self.school.as_uuid())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        let envelopes = rows.into_iter().map(OutboxRow::into_envelope).collect();
        Ok(envelopes)
    }

    #[instrument(skip(self, ids))]
    async fn mark_published(
        &self,
        _school_id: educore_core::ids::SchoolId,
        ids: &[EventId],
    ) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let id_uuids: Vec<Uuid> = ids.iter().map(|i| i.as_uuid()).collect();
        sqlx::query("UPDATE outbox SET published_at = NOW() WHERE event_id = ANY($1)")
            .bind(&id_uuids)
            .execute(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
        // QW-13 / ADAPTER-PG-013, ADAPTER-PG-029: the explicit
        // school argument MUST match the handle's scope. The
        // previous implementation bound the caller-supplied
        // `school_id` directly into the `WHERE` clause, which
        // let any tenant read any other tenant's pending
        // outbox depth (a back-pressure side channel). Reject
        // mismatches with `TenantViolation`; bind `self.school`
        // into the SQL.
        if school_id != self.school {
            return Err(DomainError::tenant_violation(format!(
                "outbox::pending_count called with school_id {} but \
                 handle is scoped to school_id {}",
                school_id.as_uuid(),
                self.school.as_uuid(),
            )));
        }
        // The default impl in the trait materialises every
        // pending row just to count them, which is O(n) memory
        // for a 1-line aggregate. Override with a direct
        // `COUNT(*)` for back-pressure sizing.
        let row = sqlx::query(
            "SELECT COUNT(*) AS n FROM outbox WHERE school_id = $1 AND published_at IS NULL",
        )
        .bind(self.school.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        let n: i64 = row
            .try_get("n")
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(u64::try_from(n).unwrap_or(0))
    }
}
