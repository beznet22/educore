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
//! as TEXT and recovered through a process-wide interner on
//! read (see `intern_command_type` below).
//!
//! | Schema column    | Source on write                            |
//! |------------------|--------------------------------------------|
//! | `command_id`     | `uuid::Uuid::now_v7()` (fresh per record)  |
//! | `expires_at`     | `recorded_at + 30 days`                    |

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock, PoisonError};

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
    /// only be produced by leaking the string. The string is
    /// interned via a process-wide cache so the leak occurs at
    /// most once per distinct value. A future PR (QW-12) will
    /// change the `IdempotencyRecord` field to `String` so
    /// adapters can round-trip the value without leaking at
    /// all.
    fn to_record(&self) -> IdempotencyRecord {
        IdempotencyRecord {
            school_id: SchoolId::from_uuid(*self.school_id.as_uuid()),
            command_type: intern_command_type(&self.command_type),
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

/// Process-wide interner for `command_type` strings read out
/// of the `idempotency` table.
///
/// The `IdempotencyRecord::command_type` field is `&'static str`
/// (per the storage port's design — the engine's command
/// catalogue is a fixed enum), but the database column is
/// `TEXT` and yields a `String` on read. Each distinct value is
/// leaked into a `&'static str` exactly once; subsequent
/// lookups return the same pointer from the cache. Memory is
/// therefore bounded by the number of distinct `command_type`
/// values observed over the process lifetime, not by the
/// number of lookups.
///
/// This replaces the previous `Box::leak(...)` per-call in
/// `IdempotencyRow::to_record`, which leaked a fresh
/// allocation on every `lookup`. Closes audit finding
/// ADAPTER-SQ-006 (QW-3).
static COMMAND_TYPE_CACHE: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();

/// Intern a `command_type` value: return a `&'static str`
/// pointer that is shared across all callers for the same
/// input. The first call for a given string allocates and
/// leaks a copy; later calls return the cached pointer.
///
/// This function is only used by `IdempotencyRow::to_record`;
/// it lives at the bottom of the file to keep the hot-path
/// code above it readable.
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
            payloads.push(format!("bounded-test-{i}-{}", Uuid::new_v4()));
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
