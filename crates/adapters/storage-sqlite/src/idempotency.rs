//! SQLite-backed `Idempotency` sub-port.
//!
//! Stores each record as a row in the `idempotency` table.
//! The schema is defined by the canonical .sql migration
//! (loaded by `SqliteStorageAdapter::migrate`).
//!
//! ## Struct <-> schema mapping notes
//!
//! The engine's `IdempotencyRecord` struct carries fields the
//! canonical `idempotency` table does not (`outcome_version`,
//! `affected_aggregate_ids`). Fields not carried by the
//! schema are populated with adapter-level defaults on write
//! and reset to empty on read. The `command_type` is stored
//! as TEXT and recovered with a `Box::leak` on read (see
//! "Known limitation" below).
//!
//! | Schema column    | Source on write                            |
//! |------------------|--------------------------------------------|
//! | `command_id`     | `uuid::Uuid::now_v7()` (fresh per record)  |
//! | `expires_at`     | `recorded_at + 30 days`                    |

use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use tracing::trace;
use uuid::fmt::Hyphenated;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{IdempotencyKey, Identifier as _, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey, IdempotencyRecord};

use crate::error::StringError;

/// The row shape stored in the SQLite `idempotency` table.
#[derive(sqlx::FromRow)]
#[allow(dead_code)] // `command_id` and `expires_at` are written but not currently read back.
struct IdempotencyRow {
    school_id: Hyphenated,
    command_type: String,
    idempotency_key: Hyphenated,
    command_id: Hyphenated,
    outcome: String,
    recorded_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl IdempotencyRow {
    /// Maps a row back to an `IdempotencyRecord`.
    ///
    /// ## Known limitation
    ///
    /// `IdempotencyRecord::command_type` is typed as
    /// `&'static str`, which means a runtime-derived value can
    /// only be produced by leaking the string. The leak is
    /// intentional for Phase 1; a future PR should change the
    /// `IdempotencyRecord` field to `String` so adapters can
    /// round-trip the value without leaking. Tracked by the
    /// `command_type` field's `&'static str` signature in
    /// `educore-storage`.
    fn to_record(&self) -> IdempotencyRecord {
        IdempotencyRecord {
            school_id: SchoolId::from_uuid(*self.school_id.as_uuid()),
            command_type: Box::leak(self.command_type.clone().into_boxed_str()),
            idempotency_key: IdempotencyKey::from_uuid(*self.idempotency_key.as_uuid()),
            outcome: Bytes::from(self.outcome.clone().into_bytes()),
            outcome_version: 0,
            recorded_at: Timestamp::from_datetime(self.recorded_at),
            affected_aggregate_ids: Vec::new(),
        }
    }
}

/// The SQLite-backed `Idempotency` implementation.
#[derive(Clone)]
pub struct SqliteIdempotency {
    pool: SqlitePool,
    school: SchoolId,
}

impl fmt::Debug for SqliteIdempotency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteIdempotency")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteIdempotency {
    /// Constructs a new idempotency handle bound to `pool`
    /// and scoped to `school`.
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl Idempotency for SqliteIdempotency {
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        let row: Option<IdempotencyRow> = sqlx::query_as::<sqlx::Sqlite, IdempotencyRow>(
            "SELECT school_id, command_type, idempotency_key, \
                    command_id, outcome, recorded_at, expires_at \
             FROM idempotency \
             WHERE school_id = ?1 AND command_type = ?2 AND idempotency_key = ?3",
        )
        .bind(key.school_id.as_uuid().hyphenated())
        .bind(key.command_type)
        .bind(key.idempotency_key.as_uuid().hyphenated())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StringError(format!("idempotency lookup: {e}")))?;
        // `as_ref().map(...)` converts `Option<IdempotencyRow>`
        // to `Option<&IdempotencyRow>` so `to_record` (which
        // takes `&self`) satisfies the `FnOnce(T) -> U` bound
        // on `Option::map`.
        Ok(row.as_ref().map(IdempotencyRow::to_record))
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        let command_id = Uuid::now_v7();
        // SQLite has no native DECIMAL/TIMESTAMP type; the
        // `expires_at` column is TEXT, so we pre-compute the
        // cutoff on the application side and bind as ISO 8601
        // via `chrono::DateTime<Utc>`.
        let expires_at = record.recorded_at.as_datetime() + Duration::days(30);
        // `outcome` is `bytes::Bytes`; the schema column is
        // TEXT. Round-trip through UTF-8 (lossy, matching the
        // SurrealDB impl's payload path).
        let outcome_str = String::from_utf8_lossy(&record.outcome).into_owned();
        sqlx::query::<sqlx::Sqlite>(
            "INSERT OR REPLACE INTO idempotency ( \
                school_id, command_type, idempotency_key, \
                command_id, outcome, recorded_at, expires_at \
             ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record.school_id.as_uuid().hyphenated())
        .bind(record.command_type)
        .bind(record.idempotency_key.as_uuid().hyphenated())
        .bind(command_id.hyphenated())
        .bind(outcome_str)
        .bind(record.recorded_at.as_datetime())
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StringError(format!("idempotency record: {e}")))?;
        trace!(command_id = %command_id, "idempotency record");
        Ok(())
    }

    async fn purge_older_than(&self, school_id: SchoolId, cutoff: Timestamp) -> Result<u64> {
        let result = sqlx::query::<sqlx::Sqlite>(
            "DELETE FROM idempotency \
             WHERE school_id = ?1 AND recorded_at < ?2",
        )
        .bind(school_id.as_uuid().hyphenated())
        .bind(cutoff.as_datetime())
        .execute(&self.pool)
        .await
        .map_err(|e| StringError(format!("idempotency purge: {e}")))?;
        let n = result.rows_affected();
        trace!(rows = n, "idempotency purge");
        // `rows_affected` already returns `u64` in sqlx 0.8;
        // no conversion needed.
        Ok(n)
    }
}
