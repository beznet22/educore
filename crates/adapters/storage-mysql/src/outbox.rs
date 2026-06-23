//! MySQL-backed `Outbox` sub-port.
//!
//! Stores each envelope as a row in the `outbox` table. The
//! schema is defined by the canonical
//! `migrations/engine/0000_engine_core.mysql.sql` migration
//! loaded by `MysqlStorageAdapter::migrate`.
//!
//! ## UUID encoding
//!
//! The schema stores UUIDs as `CHAR(36)` columns (the canonical
//! hyphenated text form). `sqlx` 0.8's `uuid::Uuid` impl for
//! MySQL encodes to that 36-char form automatically when bound
//! to a `CHAR(36)` column, so the round-trip is lossless
//! without any explicit `Hyphenated` wrapping. This is the
//! difference from the SQLite adapter (which uses `TEXT` and
//! requires explicit `Hyphenated` adapters because the SQLite
//! driver would otherwise bind `Uuid` as a 16-byte `BLOB`).
//!
//! ## Per-call transaction
//!
//! Per the design note in `crate::transaction`, every method on
//! this struct opens a fresh `pool.begin()` and commits on
//! drop. The sub-port is `Clone` (the `MySqlPool` is cheaply
//! cloneable), so the `MysqlTransaction` can hand out `&self`
//! references without lifetime issues.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{FromRow, MySqlPool, Row};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_storage::outbox::{Outbox, SerializedEnvelope};

use crate::connection_helpers::{bytes_to_json_value, json_value_to_bytes};

/// The row shape read out of the `outbox` table. Field types
/// mirror the MySQL column types per
/// `docs/schemas/sql-dialects/mysql.md` § "Type mapping".
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

/// The MySQL-backed `Outbox` implementation.
#[derive(Clone)]
pub struct MysqlOutbox {
    pool: MySqlPool,
    school: SchoolId,
}

impl std::fmt::Debug for MysqlOutbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MysqlOutbox")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl MysqlOutbox {
    /// Constructs a new outbox handle bound to `pool` and
    /// scoped to `school`.
    #[must_use]
    pub fn new(pool: MySqlPool, school: SchoolId) -> Self {
        Self { pool, school }
    }

    /// Returns the school the outbox is scoped to.
    #[must_use]
    pub fn school(&self) -> SchoolId {
        self.school
    }
}

#[async_trait]
impl Outbox for MysqlOutbox {
    #[instrument(skip(self, envelope), fields(event_id = %envelope.event_id))]
    async fn append(&self, envelope: SerializedEnvelope) -> Result<()> {
        // `enqueued_at` and `published_at` are handled at the DB
        // level (`UTC_TIMESTAMP(6)` and `NULL` respectively) per
        // the canonical DDL. `attempts` defaults to 0,
        // `last_error` is NULL.
        sqlx::query::<sqlx::MySql>(
            "INSERT INTO `outbox` (\
                `event_id`, `event_type`, `event_version`, `school_id`, \
                `aggregate_id`, `aggregate_type`, `actor_id`, \
                `correlation_id`, `causation_id`, `occurred_at`, \
                `recorded_at`, `payload`, `enqueued_at`, `published_at`, \
                `attempts`, `last_error`\
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, UTC_TIMESTAMP(6), NULL, 0, NULL)",
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
    async fn pending(&self, limit: u32) -> Result<Vec<SerializedEnvelope>> {
        // School partition: the `WHERE school_id = ?` clause
        // scopes the drain to the adapter's bound `self.school`
        // (set at `MysqlStorageAdapter::connect` time).
        // Per `crates/infra/storage/src/outbox.rs`, "the outbox
        // is partitioned by `school_id` so callers see only
        // envelopes for their school." This is the MySQL
        // adapter's enforcement of that invariant; the testkit
        // finding (TOOL-TK-004) closed the in-memory side of
        // the same invariant.
        let rows: Vec<OutboxRow> = sqlx::query_as::<sqlx::MySql, OutboxRow>(
            "SELECT \
                `event_id`, `event_type`, `event_version`, `school_id`, \
                `aggregate_id`, `aggregate_type`, `actor_id`, \
                `correlation_id`, `causation_id`, `occurred_at`, `payload` \
             FROM `outbox` \
             WHERE `school_id` = ? AND `published_at` IS NULL \
             ORDER BY `enqueued_at` ASC \
             LIMIT ?",
        )
        .bind(self.school.as_uuid())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        let envelopes = rows.into_iter().map(OutboxRow::into_envelope).collect();
        Ok(envelopes)
    }

    #[instrument(skip(self))]
    async fn pending_for_school(
        &self,
        school_id: SchoolId,
        limit: u32,
    ) -> Result<Vec<SerializedEnvelope>> {
        // QW-13 / ADAPT-MY-OUTBOX-*: the explicit school
        // argument MUST match the handle's scope. A mismatched
        // `school_id` is a cross-tenant read attempt and is
        // rejected with `TenantViolation` (per
        // `docs/schemas/tenancy-schema.md` § 2). Once the
        // assertion passes, the underlying `pending` query is
        // already partitioned by `self.school`, so no extra
        // SQL is needed.
        if school_id != self.school {
            return Err(DomainError::tenant_violation(format!(
                "outbox::pending_for_school called with school_id {} but \
                 handle is scoped to school_id {}",
                school_id.as_uuid(),
                self.school.as_uuid(),
            )));
        }
        self.pending(limit).await
    }

    #[instrument(skip(self, ids))]
    async fn mark_published(&self, ids: &[EventId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        // School partition: the `school_id = ?` predicate is
        // bound to `self.school` so a relay holding an
        // `EventId` from another school cannot mutate that
        // school's outbox row. Defense in depth alongside the
        // `pending()` filter above; closes the MySQL half of
        // TOOL-TK-004 / ADAPT-MY-OUTBOX-*.
        //
        // MySQL's `sqlx` driver does not implement
        // `Encode<MySql>` for `Vec<T>` (only `Vec<u8>`), so we
        // build the `IN (?, ?, ...)` clause dynamically with a
        // `QueryBuilder`. The expansion is functionally
        // equivalent to PostgreSQL's `ANY(?)` and to the
        // sqlite outbox's `QueryBuilder` pattern.
        let mut qb: sqlx::QueryBuilder<sqlx::MySql> = sqlx::QueryBuilder::new(
            "UPDATE `outbox` SET `published_at` = UTC_TIMESTAMP(6) \
             WHERE `school_id` = ? AND `event_id` IN (",
        );
        qb.push_bind(self.school.as_uuid());
        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(id.as_uuid());
        }
        qb.push(")");
        qb.build()
            .execute(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn pending_count(&self, school_id: SchoolId) -> Result<u64> {
        // QW-13 / ADAPT-MY-OUTBOX-*: the explicit school
        // argument MUST match the handle's scope. The previous
        // implementation bound the caller-supplied `school_id`
        // directly into the `WHERE` clause, which let any
        // tenant read any other tenant's pending outbox depth
        // (a back-pressure side channel — the relay uses this
        // count for back-pressure sizing and a cross-tenant
        // probe would leak tenant activity). Reject mismatches
        // with `TenantViolation`; bind `self.school` into the
        // SQL.
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
        let row = sqlx::query::<sqlx::MySql>(
            "SELECT COUNT(*) AS n FROM `outbox` WHERE `school_id` = ? AND `published_at` IS NULL",
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
