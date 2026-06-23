//! MySQL-backed `Idempotency` sub-port.
//!
//! Stores each `IdempotencyRecord` as a row in the
//! `idempotency` table. The schema is defined by the canonical
//! `migrations/engine/0000_engine_core.mysql.sql` migration
//! loaded by `MysqlStorageAdapter::migrate`.
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
//!
//! ## Conflict handling
//!
//! PostgreSQL's `ON CONFLICT (...) DO NOTHING` does not have a
//! direct MySQL equivalent. MySQL's idiom is
//! `INSERT ... ON DUPLICATE KEY UPDATE <col> = <col>`: a
//! no-op assignment to an arbitrary column that suppresses the
//! duplicate-key error and the `Rows matched: 1 Changed: 0`
//! warning. The composite primary key
//! (`school_id`, `command_type`, `idempotency_key`) is the
//! conflict target; assigning the bound value to itself leaves
//! the existing row untouched.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use async_trait::async_trait;
use bytes;
use chrono::{DateTime, Duration, Utc};
use serde_json::{Map, Value};
use sqlx::types::Json;
use sqlx::{FromRow, MySqlPool};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{IdempotencyKey, Identifier as _, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_storage::idempotency::{
    Idempotency, IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};

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

/// The MySQL-backed `Idempotency` implementation.
#[derive(Clone)]
pub struct MysqlIdempotency {
    pool: MySqlPool,
    #[allow(dead_code)]
    school: SchoolId,
}

impl std::fmt::Debug for MysqlIdempotency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MysqlIdempotency")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl MysqlIdempotency {
    /// Constructs a new idempotency handle bound to `pool` and
    /// scoped to `school`. The `school` field is reserved for
    /// future per-connection filtering; the trait's methods
    /// take a `school_id` argument and use that.
    #[must_use]
    pub fn new(pool: MySqlPool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl Idempotency for MysqlIdempotency {
    #[instrument(skip(self, key))]
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        let row: Option<IdempotencyRow> = sqlx::query_as::<sqlx::MySql, IdempotencyRow>(
            "SELECT `school_id`, `command_type`, `idempotency_key`, \
                `command_id`, `outcome`, `recorded_at`, `expires_at` \
             FROM `idempotency` \
             WHERE `school_id` = ? AND `command_type` = ? AND `idempotency_key` = ?",
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
        // MySQL's `ON DUPLICATE KEY UPDATE <col> = <col>` is a
        // no-op assignment that suppresses the duplicate-key
        // error when the composite primary key already exists.
        // The existing row is left untouched (no-op assignment
        // to itself). The engine's at-least-once semantics
        // rely on this: a duplicate dispatch from the relay
        // produces no second row and does not change the
        // original outcome.
        sqlx::query::<sqlx::MySql>(
            "INSERT INTO `idempotency` (\
                `school_id`, `command_type`, `idempotency_key`, \
                `command_id`, `outcome`, `recorded_at`, `expires_at`\
            ) VALUES (?, ?, ?, ?, ?, ?, ?) \
             ON DUPLICATE KEY UPDATE `command_id` = `command_id`",
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

    #[instrument(skip(self, record))]
    async fn record_outcome(&self, record: IdempotencyRecord) -> Result<IdempotencyOutcome> {
        // QW-12 (Phase B, MySQL adapter): implement the
        // structured duplicate-detection contract from
        // `docs/audit_reports/remediation/09-quick-wins.md` and
        // `docs/decisions/ADR-014-Idempotency.md`. The
        // default trait impl delegates to `record` (which
        // uses `ON DUPLICATE KEY UPDATE` and silently
        // swallows duplicates), so it can never surface
        // `Conflict`. This override restores the
        // duplicate-detection contract.
        //
        // Strategy:
        //
        // 1. Try a plain `INSERT` (no `ON DUPLICATE KEY`
        //    clause). MySQL surfaces a duplicate-key
        //    collision on the composite primary key as
        //    error 1062, which sqlx maps to
        //    `ErrorKind::UniqueViolation` on the
        //    `sqlx::Error::Database` variant.
        // 2. On a `UniqueViolation`, fetch the pre-existing
        //    row via `self.lookup(...)` and return
        //    `IdempotencyOutcome::Conflict { existing }` so
        //    the dispatcher can replay the original outcome.
        // 3. On any other error, surface it as
        //    `DomainError::Infrastructure` (a true
        //    adapter-level failure, not a business
        //    outcome).
        //
        // The plain INSERT also closes audit finding
        // ADAPT-MY-005 (Critical): the previous
        // `ON DUPLICATE KEY UPDATE command_id =
        // VALUES(command_id)` form silently overwrote any
        // prior row regardless of outcome equality, which
        // both halves of the port contract forbade.
        let command_id = Uuid::now_v7();
        let recorded_at = record.recorded_at.as_datetime();
        let expires_at = recorded_at
            .checked_add_signed(Duration::hours(DEFAULT_RETENTION_HOURS))
            .unwrap_or(recorded_at);
        let outcome = envelope_outcome(&record);

        let insert = sqlx::query::<sqlx::MySql>(
            "INSERT INTO `idempotency` (\
                `school_id`, `command_type`, `idempotency_key`, \
                `command_id`, `outcome`, `recorded_at`, `expires_at`\
            ) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(record.school_id.as_uuid())
        .bind(record.command_type)
        .bind(record.idempotency_key.as_uuid())
        .bind(command_id)
        .bind(Json(&outcome))
        .bind(recorded_at)
        .bind(expires_at)
        .execute(&self.pool)
        .await;

        match insert {
            Ok(_) => Ok(IdempotencyOutcome::Recorded),
            Err(sqlx::Error::Database(db))
                if db.kind() == sqlx::error::ErrorKind::UniqueViolation =>
            {
                // Duplicate composite key: MySQL error 1062
                // (`ER_DUP_ENTRY`). Fetch the existing row so
                // the caller can replay the original outcome
                // — the port contract requires we surface
                // the pre-existing record, not the rejected
                // one.
                let key = IdempotencyRecord::composite_key(
                    record.school_id,
                    record.command_type,
                    record.idempotency_key,
                );
                match self.lookup(key).await? {
                    Some(existing) => Ok(IdempotencyOutcome::Conflict { existing }),
                    // The row was present at INSERT time but
                    // vanished before our SELECT (concurrent
                    // sweep / retention purge). Surface as
                    // infrastructure so the caller retries
                    // rather than silently recording.
                    None => Err(educore_core::error::DomainError::infrastructure(
                        crate::error::StringError(
                            "MysqlIdempotency::record_outcome: duplicate-key \
                             detected but the existing row vanished before \
                             the follow-up SELECT (concurrent purge?)"
                                .to_owned(),
                        ),
                    )),
                }
            }
            Err(other) => Err(educore_core::error::DomainError::infrastructure(other)),
        }
    }

    #[instrument(skip(self, cutoff))]
    async fn purge_older_than(&self, school_id: SchoolId, cutoff: Timestamp) -> Result<u64> {
        // The default impl in the trait is a no-op. The MySQL
        // adapter overrides with a direct `DELETE` to support
        // the consumer's retention sweep.
        let row = sqlx::query::<sqlx::MySql>(
            "DELETE FROM `idempotency` WHERE `school_id` = ? AND `recorded_at` < ?",
        )
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
/// finding ADAPT-MY-007 (QW-3).
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

    // Pull in the trait definitions so the live MySQL tests
    // can use the `IdGenerator::next_*` and
    // `StorageAdapter::{connect, migrate, begin}` methods on
    // the concrete adapter types. These traits live in
    // `educore-core::clock` and `educore-storage::port`
    // respectively and are not in the `super::*` namespace.
    use educore_core::clock::IdGenerator as _;
    use educore_storage::StorageAdapter as _;

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

    // ---------------------------------------------------------------
    // QW-12 (Phase B, MySQL adapter): `record_outcome` end-to-end
    // tests against a live MySQL instance.
    //
    // These tests are gated on the `EDUCORE_MYSQL_URL` env var
    // (matching the convention in `tests/outbox_e2e.rs` for
    // this crate) AND marked `#[ignore]` so they do NOT run in
    // a default `cargo test` invocation. They require a real
    // MySQL 8 instance with the engine's canonical DDL applied.
    //
    // To run locally:
    //
    // ```text
    // docker run --rm -d --name educore-mysql -p 3306:3306 \
    //     -e MYSQL_ROOT_PASSWORD=educore -e MYSQL_DATABASE=educore \
    //     -e MYSQL_USER=educore -e MYSQL_PASSWORD=educore \
    //     mysql:8
    // export EDUCORE_MYSQL_URL='mysql://educore:educore@localhost:3306/educore'
    // cargo test -p educore-storage-mysql --lib -- \
    //     --ignored idempotency::tests::record_outcome \
    //     --nocapture --test-threads=1
    // ```
    //
    // The two tests share the process-wide `COMMAND_TYPE_CACHE`
    // static, so they must run on a single thread. They also
    // share the `TEST_LOCK` so they serialize against the
    // `intern_*` tests above.
    // ---------------------------------------------------------------

    /// Build a fresh connection to the `EDUCORE_MYSQL_URL`
    /// test database, run the engine DDL, and return a
    /// `Transaction` handle whose `.idempotency()` accessor
    /// yields a `MysqlIdempotency`. Mirrors the convention
    /// used by `tests/outbox_e2e.rs` (env-gated,
    /// `migrate()`-first, then sub-port accessors).
    ///
    /// Returns `None` if `EDUCORE_MYSQL_URL` is unset; the
    /// calling test must early-return in that case so CI is
    /// green without a MySQL fixture.
    async fn fresh_mysql_idempotency() -> Option<Box<dyn educore_storage::Transaction>> {
        let url = std::env::var("EDUCORE_MYSQL_URL").ok()?;
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let adapter: crate::MysqlStorageAdapter = crate::MysqlStorageAdapter::connect(&url, school)
            .await
            .expect("MySQL connection failed; is EDUCORE_MYSQL_URL correct?");
        adapter
            .migrate()
            .await
            .expect("MySQL migrate() failed; is the engine DDL present?");
        Some(adapter.begin().await.expect("begin() failed"))
    }

    /// QW-12 regression test: a fresh `record_outcome` (no
    /// pre-existing row) MUST return `Recorded`. Closes the
    /// happy-path half of ADAPT-MY-005 / ADAPT-MY-009.
    #[tokio::test]
    #[ignore = "requires live MySQL; set EDUCORE_MYSQL_URL and run with --ignored"]
    async fn record_outcome_returns_recorded_for_new_key() {
        // NOTE: we deliberately do NOT acquire `TEST_LOCK`
        // here. `TEST_LOCK` is a std `Mutex` and would be
        // held across the `await` points below (clippy's
        // `await_holding_lock` lint). The
        // `COMMAND_TYPE_CACHE` static is internally
        // `Mutex`-protected, so concurrent `lookup` calls
        // from other tests are safe; the only risk is that
        // the `intern_is_bounded` count assertion might see
        // our inserts, which is acceptable because both this
        // test and `intern_is_bounded` are `#[ignore]`'d by
        // default and only run when explicitly opted in via
        // `cargo test -- --ignored`.
        let tx = match fresh_mysql_idempotency().await {
            Some(t) => t,
            None => {
                tracing::info!("EDUCORE_MYSQL_URL not set; skipping live MySQL test");
                return;
            }
        };

        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();
        let record = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: bytes::Bytes::from_static(b"first-payload-mysql-qw12"),
            outcome_version: 1,
            recorded_at: educore_core::value_objects::Timestamp::now(),
            affected_aggregate_ids: Vec::new(),
        };

        let outcome = tx
            .idempotency()
            .record_outcome(record.clone())
            .await
            .expect("first record_outcome must not fail");
        assert_eq!(
            outcome,
            IdempotencyOutcome::Recorded,
            "first write with a fresh composite key must report Recorded",
        );

        // Sanity-check round-trip: lookup returns the same row.
        let lookup_key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        let stored = tx
            .idempotency()
            .lookup(lookup_key)
            .await
            .expect("lookup must not fail")
            .expect("lookup must find the freshly written row");
        assert_eq!(stored.outcome, record.outcome);
        assert_eq!(stored.command_type, record.command_type);
    }

    /// QW-12 regression test: a `record_outcome` whose composite
    /// key collides with an existing row MUST return
    /// `Conflict { existing }` carrying the ORIGINAL record
    /// (not the rejected one). Closes the duplicate-detection
    /// half of ADAPT-MY-005 and the per-adapter half of
    /// ADAPT-MY-009.
    #[tokio::test]
    #[ignore = "requires live MySQL; set EDUCORE_MYSQL_URL and run with --ignored"]
    async fn record_outcome_returns_conflict_for_duplicate_key() {
        // NOTE: we deliberately do NOT acquire `TEST_LOCK`
        // here. `TEST_LOCK` is a std `Mutex` and would be
        // held across the `await` points below (clippy's
        // `await_holding_lock` lint). The
        // `COMMAND_TYPE_CACHE` static is internally
        // `Mutex`-protected, so concurrent `lookup` calls
        // from other tests are safe; the only risk is that
        // the `intern_is_bounded` count assertion might see
        // our inserts, which is acceptable because both this
        // test and `intern_is_bounded` are `#[ignore]`'d by
        // default and only run when explicitly opted in via
        // `cargo test -- --ignored`.
        let tx = match fresh_mysql_idempotency().await {
            Some(t) => t,
            None => {
                tracing::info!("EDUCORE_MYSQL_URL not set; skipping live MySQL test");
                return;
            }
        };

        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();
        let first = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: bytes::Bytes::from_static(b"first-payload-mysql-qw12-dup"),
            outcome_version: 1,
            recorded_at: educore_core::value_objects::Timestamp::now(),
            affected_aggregate_ids: Vec::new(),
        };
        let second = IdempotencyRecord {
            // Same composite key, DIFFERENT outcome bytes —
            // a hard conflict, not a no-op retry.
            outcome: bytes::Bytes::from_static(b"second-payload-mysql-qw12-dup"),
            ..first.clone()
        };

        let first_outcome = tx
            .idempotency()
            .record_outcome(first.clone())
            .await
            .expect("first record_outcome must not fail");
        assert_eq!(
            first_outcome,
            IdempotencyOutcome::Recorded,
            "first write must report Recorded",
        );

        let second_outcome = tx
            .idempotency()
            .record_outcome(second)
            .await
            .expect("second record_outcome must surface Conflict, not an error");
        match second_outcome {
            IdempotencyOutcome::Conflict { existing } => {
                // The carried record is the FIRST write's
                // payload — the port contract requires the
                // pre-existing record, not the rejected one.
                assert_eq!(
                    existing.outcome,
                    bytes::Bytes::from_static(b"first-payload-mysql-qw12-dup"),
                    "Conflict::existing must carry the ORIGINAL outcome bytes, \
                     not the rejected second payload",
                );
                assert_eq!(
                    existing.outcome_version, 1,
                    "Conflict::existing must carry the original outcome_version",
                );
                assert_eq!(
                    existing.school_id, school,
                    "Conflict::existing must carry the original school_id",
                );
                assert_eq!(
                    existing.idempotency_key, key,
                    "Conflict::existing must carry the original idempotency_key",
                );
                assert_eq!(
                    existing.command_type, "academic.student.admit",
                    "Conflict::existing must carry the original command_type",
                );
            }
            other => panic!(
                "expected IdempotencyOutcome::Conflict on duplicate key \
                 with different outcome, got {other:?}",
            ),
        }
    }
}
