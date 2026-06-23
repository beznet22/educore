//! PostgreSQL-backed `Idempotency` sub-port.
//!
//! Stores each `IdempotencyRecord` as a row in the
//! `engine.idempotency` table. The schema is defined by the
//! canonical `migrations/engine/0000_engine_core.postgres.sql`
//! migration loaded by `PostgresStorageAdapter::migrate`.
//!
//! ## Column mapping
//!
//! The engine's `IdempotencyRecord` carries two fields the DDL
//! does not expose (`outcome_version`, `affected_aggregate_ids`).
//! On write, the adapter wraps the outcome in a JSON envelope
//! that carries the version and the affected aggregate ids as
//! siblings of the original payload. On read, the envelope is
//! parsed back into the record.
//!
//! The DDL also has two columns the record does not carry
//! (`command_id`, `expires_at`):
//!
//! - `command_id` is a fresh UUIDv7 generated at write time
//! - `expires_at` is `recorded_at + 24h` by default; consumers
//!   that need a different retention window use `purge_older_than`
//!   to sweep the store

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use async_trait::async_trait;
use bytes;
use chrono::{DateTime, Duration, Utc};
use serde_json::{Map, Value};
use sqlx::types::Json;
use sqlx::{FromRow, PgPool};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{IdempotencyKey, Identifier as _, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey, IdempotencyRecord};

use crate::connection_helpers::{bytes_to_json_value, json_value_to_bytes};

/// Default retention for idempotency records. After this many
/// hours, the `purge_older_than` call is expected to sweep the
/// row. The value is conservative (24h); consumers that need
/// longer retention override via a custom sweep job.
const DEFAULT_RETENTION_HOURS: i64 = 24;

/// The row shape read out of the `idempotency` table. We
/// select all seven columns so the read path can reconstruct
/// the envelope.
#[derive(Debug, FromRow)]
struct IdempotencyRow {
    school_id: Uuid,
    command_type: String,
    idempotency_key: Uuid,
    #[allow(dead_code)]
    command_id: Uuid,
    outcome: Json<Value>,
    recorded_at: DateTime<Utc>,
    #[allow(dead_code)]
    expires_at: DateTime<Utc>,
}

/// Build the JSON envelope that wraps the original outcome,
/// the outcome version, and the affected aggregate ids.
///
/// The envelope is a JSON object with three keys: `payload`
/// (the original outcome, re-serialised), `version` (the
/// schema version of the outcome), and `affected_aggregate_ids`
/// (the array of uuids the command touched).
fn envelope_outcome(record: &IdempotencyRecord) -> Value {
    let mut obj = Map::with_capacity(3);
    obj.insert("payload".to_owned(), bytes_to_json_value(&record.outcome));
    obj.insert(
        "version".to_owned(),
        Value::Number(serde_json::Number::from(record.outcome_version)),
    );
    let agg_ids: Vec<Value> = record
        .affected_aggregate_ids
        .iter()
        .map(|u| Value::String(u.to_string()))
        .collect();
    obj.insert("affected_aggregate_ids".to_owned(), Value::Array(agg_ids));
    Value::Object(obj)
}

/// Unwrap a JSON envelope produced by [`envelope_outcome`]. If
/// the value is not an envelope (e.g. a legacy row written by
/// an earlier version of the adapter), the missing fields are
/// replaced with safe defaults.
fn unwrap_envelope(value: &Value) -> (bytes::Bytes, u32, Vec<Uuid>) {
    let mut payload: bytes::Bytes = json_value_to_bytes(value);
    let mut version: u32 = 0;
    let mut agg_ids: Vec<Uuid> = Vec::new();
    if let Value::Object(map) = value {
        if let Some(v) = map.get("payload") {
            payload = json_value_to_bytes(v);
        }
        if let Some(v) = map.get("version") {
            if let Some(n) = v.as_u64() {
                version = u32::try_from(n).unwrap_or(0);
            }
        }
        if let Some(Value::Array(arr)) = map.get("affected_aggregate_ids") {
            agg_ids = arr
                .iter()
                .filter_map(|x| x.as_str().and_then(|s| Uuid::parse_str(s).ok()))
                .collect();
        }
    }
    (payload, version, agg_ids)
}

/// The PostgreSQL-backed `Idempotency` implementation.
#[derive(Clone)]
pub struct PostgresIdempotency {
    pool: PgPool,
    #[allow(dead_code)]
    school: SchoolId,
}

impl std::fmt::Debug for PostgresIdempotency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresIdempotency")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl PostgresIdempotency {
    /// Constructs a new idempotency handle bound to `pool` and
    /// scoped to `school`. The `school` field is reserved for
    /// future per-connection filtering; the trait's methods
    /// take a `school_id` argument and use that.
    #[must_use]
    pub fn new(pool: PgPool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl Idempotency for PostgresIdempotency {
    #[instrument(skip(self, key))]
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        let row: Option<IdempotencyRow> = sqlx::query_as::<_, IdempotencyRow>(
            "SELECT school_id, command_type, idempotency_key, \
                command_id, outcome, recorded_at, expires_at \
             FROM idempotency \
             WHERE school_id = $1 AND command_type = $2 AND idempotency_key = $3",
        )
        .bind(key.school_id.as_uuid())
        .bind(key.command_type)
        .bind(key.idempotency_key.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        match row {
            None => Ok(None),
            Some(r) => {
                let (payload, version, agg_ids) = unwrap_envelope(&r.outcome.0);
                Ok(Some(IdempotencyRecord {
                    school_id: SchoolId::from_uuid(r.school_id),
                    command_type: intern_command_type(&r.command_type),
                    idempotency_key: IdempotencyKey::from_uuid(r.idempotency_key),
                    outcome: payload,
                    outcome_version: version,
                    recorded_at: Timestamp::from_datetime(r.recorded_at),
                    affected_aggregate_ids: agg_ids,
                }))
            }
        }
    }

    #[instrument(skip(self, record))]
    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        // Generate the synthetic columns the DDL requires but
        // the record does not carry.
        let command_id = Uuid::now_v7();
        let recorded_at = record.recorded_at.as_datetime();
        let expires_at = recorded_at
            .checked_add_signed(Duration::hours(DEFAULT_RETENTION_HOURS))
            .unwrap_or(recorded_at);
        let outcome = envelope_outcome(&record);
        // `ON CONFLICT DO NOTHING` makes the write a no-op
        // when a row with the same composite key already
        // exists. The engine's at-least-once semantics rely on
        // this: a duplicate dispatch from the relay produces
        // no second row.
        sqlx::query(
            "INSERT INTO idempotency (\
                school_id, command_type, idempotency_key, \
                command_id, outcome, recorded_at, expires_at\
            ) VALUES ($1, $2, $3, $4, $5, $6, $7) \
             ON CONFLICT (school_id, command_type, idempotency_key) DO NOTHING",
        )
        .bind(record.school_id.as_uuid())
        .bind(record.command_type)
        .bind(record.idempotency_key.as_uuid())
        .bind(command_id)
        .bind(Json(&outcome))
        .bind(recorded_at)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self, cutoff))]
    async fn purge_older_than(&self, school_id: SchoolId, cutoff: Timestamp) -> Result<u64> {
        // The default impl in the trait is a no-op. The PG
        // adapter overrides with a direct `DELETE` to support
        // the consumer's retention sweep.
        let row = sqlx::query("DELETE FROM idempotency WHERE school_id = $1 AND recorded_at < $2")
            .bind(school_id.as_uuid())
            .bind(cutoff.as_datetime())
            .execute(&self.pool)
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        let n: i64 = row.rows_affected().try_into().unwrap_or(i64::MAX);
        Ok(u64::try_from(n).unwrap_or(0))
    }
}

/// Process-wide interner for `command_type` strings read out
/// of the `idempotency` table.
///
/// The `IdempotencyRecord::command_type` field is `&'static str`
/// (per the storage port's design — the engine's command
/// catalogue is a fixed enum), but the database column is
/// `VARCHAR` and yields a `String` on read. Each distinct value
/// is leaked into a `&'static str` exactly once; subsequent
/// lookups return the same pointer from the cache. Memory is
/// therefore bounded by the number of distinct `command_type`
/// values observed over the process lifetime, not by the
/// number of lookups.
///
/// This replaces the previous `Box::leak(...)` per-call, which
/// leaked a fresh allocation on every `lookup`. Closes audit
/// finding ADAPTER-PG-011 (QW-3).
static COMMAND_TYPE_CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();

/// Intern a `command_type` value: return a `&'static str`
/// pointer that is shared across all callers for the same
/// input. The first call for a given string allocates and
/// leaks a copy; later calls return the cached pointer.
///
/// This function is only used by `lookup`; it lives at the
/// bottom of the file to keep the hot-path code above it
/// readable.
fn intern_command_type(s: &str) -> &'static str {
    let cache = COMMAND_TYPE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache = cache.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(&interned) = cache.get(s) {
        return interned;
    }
    // First use of this string: leak an owned copy so the
    // `&'static str` requirement is satisfied. The cache
    // ensures we leak at most once per distinct value.
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    cache.insert(leaked.to_owned(), leaked);
    leaked
}

/// Number of distinct `command_type` values currently cached.
/// Exposed for tests so they can verify bounded growth without
/// touching the private static directly.
#[cfg(test)]
fn cached_command_type_count() -> usize {
    let cache = COMMAND_TYPE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let cache = cache.lock().unwrap_or_else(PoisonError::into_inner);
    cache.len()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::dbg_macro
    )]

    use super::*;

    // Serialize the tests in this module because they share
    // the process-wide `COMMAND_TYPE_CACHE` static. Without
    // this lock, parallel execution would let one test's
    // inserts leak into the other's count assertions.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn intern_is_idempotent() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        // Calling intern twice with the same string must
        // return the exact same `&'static str` pointer; the
        // second call is a cache hit and allocates nothing.
        let first = intern_command_type("academic.student.admit");
        let second = intern_command_type("academic.student.admit");
        assert_eq!(
            first as *const str, second as *const str,
            "intern returned different pointers for the same input",
        );
        // The pointer must also be stable across many calls;
        // verify a third call returns the same address.
        let third = intern_command_type("academic.student.admit");
        assert_eq!(first as *const str, third as *const str);
    }

    #[test]
    fn intern_is_bounded() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);

        // Insert 100 distinct strings; the cache must grow
        // by exactly 100 entries (one per distinct value).
        let n = 100_usize;
        let baseline = cached_command_type_count();

        // Build payloads first so the strings live for the
        // duration of both phases.
        let mut payloads: Vec<String> = Vec::with_capacity(n);
        for i in 0..n {
            payloads.push(format!("bounded-test-{i}-{}", uuid::Uuid::new_v4()));
        }

        // Phase 1: first-time insertion; cache grows by `n`.
        for p in &payloads {
            let _ = intern_command_type(p);
        }
        let after_inserts = cached_command_type_count();
        assert_eq!(
            after_inserts - baseline,
            n,
            "cache should grow by exactly {n} entries on first insert",
        );

        // Phase 2: re-intern the SAME strings; cache must
        // NOT grow further (these are cache hits).
        for p in &payloads {
            let _ = intern_command_type(p);
        }
        let after_reinserts = cached_command_type_count();
        assert_eq!(
            after_reinserts, after_inserts,
            "re-interning the same strings must not grow the cache",
        );
    }
}
