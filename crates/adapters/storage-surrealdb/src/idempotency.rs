//! SurrealDB-backed `Idempotency` sub-port.
//!
//! Stores each idempotency record in the `idempotency` table
//! keyed by the composite (school_id, command_type,
//! idempotency_key). The schema is defined by the canonical
//! .surql migration (loaded by `SurrealStorageAdapter::migrate`).
//!
//! Wired into `lib.rs` by A'.1 (Phase 16); the stub in
//! `stubs.rs` has been removed in the same commit.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use async_trait::async_trait;
use bytes::Bytes;
use surrealdb::sql::{Bytes as SurrealBytes, Uuid as SurrealUuid};

use educore_core::error::Result;
use educore_core::ids::{IdempotencyKey, Identifier, SchoolId};
use educore_core::value_objects::Timestamp;
use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey, IdempotencyRecord};

use crate::connection::Db;
use crate::error::StringError;

/// The row shape stored in the SurrealDB `idempotency` table.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct IdempotencyRow {
    pub school_id: Option<SurrealUuid>,
    pub command_type: String,
    pub idempotency_key: SurrealUuid,
    pub outcome: SurrealBytes,
    pub outcome_version: u32,
    pub recorded_at: surrealdb::sql::Datetime,
    pub affected_aggregate_ids: Option<Vec<SurrealUuid>>,
}

impl IdempotencyRow {
    pub fn from_record(record: &IdempotencyRecord) -> Self {
        Self {
            school_id: Some(SurrealUuid::from(record.school_id.as_uuid())),
            command_type: record.command_type.to_owned(),
            idempotency_key: SurrealUuid::from(record.idempotency_key.as_uuid()),
            outcome: SurrealBytes::from(record.outcome.to_vec()),
            outcome_version: record.outcome_version,
            recorded_at: surrealdb::sql::Datetime::from(record.recorded_at.as_datetime()),
            affected_aggregate_ids: Some(
                record
                    .affected_aggregate_ids
                    .iter()
                    .map(|u| SurrealUuid::from(*u))
                    .collect(),
            ),
        }
    }

    pub fn to_record(&self) -> IdempotencyRecord {
        let school_id = self
            .school_id
            .map(|u| SchoolId::from_uuid(u.0))
            .unwrap_or_else(|| SchoolId::from_uuid(uuid::Uuid::nil()));
        let idempotency_key = IdempotencyKey::from(self.idempotency_key.0);
        let outcome = Bytes::from(self.outcome.to_vec());
        let recorded_at = Timestamp::from_datetime(self.recorded_at.0);
        let affected_aggregate_ids = self
            .affected_aggregate_ids
            .as_ref()
            .map(|v| v.iter().map(|u| u.0).collect())
            .unwrap_or_default();
        IdempotencyRecord {
            school_id,
            command_type: intern_command_type(&self.command_type),
            idempotency_key,
            outcome,
            outcome_version: self.outcome_version,
            recorded_at,
            affected_aggregate_ids,
        }
    }
}

/// The SurrealDB-backed `Idempotency` implementation.
#[derive(Clone)]
pub struct SurrealIdempotency {
    pub(crate) db: Db,
}

impl std::fmt::Debug for SurrealIdempotency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealIdempotency").finish_non_exhaustive()
    }
}

impl SurrealIdempotency {
    /// Constructs a new idempotency handle bound to `db`.
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Idempotency for SurrealIdempotency {
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        let school_uuid = SurrealUuid::from(key.school_id.as_uuid());
        let idem_uuid = SurrealUuid::from(key.idempotency_key.as_uuid());
        let mut response = self
            .db
            .query(
                "SELECT school_id, command_type, idempotency_key, outcome, outcome_version, \
                        recorded_at, affected_aggregate_ids \
                 FROM idempotency \
                 WHERE school_id = $school AND command_type = $cmd AND idempotency_key = $key \
                 LIMIT 1",
            )
            .bind(("school", school_uuid))
            .bind(("cmd", key.command_type.to_owned()))
            .bind(("key", idem_uuid))
            .await
            .map_err(|e| StringError(format!("idempotency lookup: {e}")))?;
        let rows: Vec<IdempotencyRow> = response
            .take(0)
            .map_err(|e| StringError(format!("idempotency lookup take: {e}")))?;
        Ok(rows
            .into_iter()
            .next()
            .map(|r| IdempotencyRow::to_record(&r)))
    }

    async fn record(&self, record: IdempotencyRecord) -> Result<()> {
        let row = IdempotencyRow::from_record(&record);
        let _ = self
            .db
            .query(
                "INSERT INTO idempotency { \
                    school_id: $school_id, \
                    command_type: $command_type, \
                    idempotency_key: $idempotency_key, \
                    outcome: $outcome, \
                    outcome_version: $outcome_version, \
                    recorded_at: $recorded_at, \
                    affected_aggregate_ids: $affected_aggregate_ids \
                }",
            )
            .bind(("school_id", row.school_id))
            .bind(("command_type", row.command_type))
            .bind(("idempotency_key", row.idempotency_key))
            .bind(("outcome", row.outcome))
            .bind(("outcome_version", row.outcome_version))
            .bind(("recorded_at", row.recorded_at))
            .bind(("affected_aggregate_ids", row.affected_aggregate_ids))
            .await
            .map_err(|e| StringError(format!("idempotency record: {e}")))?;
        Ok(())
    }
}

/// Process-wide interner for `command_type` strings read out
/// of the `idempotency` table.
///
/// The `IdempotencyRecord::command_type` field is `&'static str`
/// (per the storage port's design — the engine's command
/// catalogue is a fixed enum), but the database column yields
/// a `String` on read. Each distinct value is leaked into a
/// `&'static str` exactly once; subsequent lookups return the
/// same pointer from the cache. Memory is therefore bounded
/// by the number of distinct `command_type` values observed
/// over the process lifetime, not by the number of lookups.
///
/// This replaces the previous `Box::leak(...)` per-call in
/// `IdempotencyRow::to_record`, which leaked a fresh
/// allocation on every `lookup`. Closes audit finding
/// ADAPTER-SQ-006 (QW-3) — the SurrealDB adapter shares the
/// same pattern and the same port-level constraint, so the
/// fix is applied here in lockstep with the relational
/// adapters.
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
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::SchoolId;
    use educore_storage::StorageAdapter;

    async fn setup() -> SurrealIdempotency {
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let adapter = crate::storage::SurrealStorageAdapter::in_memory(school)
            .await
            .expect("in-memory adapter");
        adapter.migrate().await.expect("migration");
        SurrealIdempotency::new(adapter.db().clone())
    }

    fn sample_record() -> IdempotencyRecord {
        let g = SystemIdGen;
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: g.next_idempotency_key(),
            outcome: Bytes::from_static(b"{\"id\":\"x\"}"),
            outcome_version: 1,
            recorded_at: Timestamp::now(),
            affected_aggregate_ids: vec![],
        }
    }

    #[tokio::test]
    async fn record_then_lookup_round_trips() {
        let idem = setup().await;
        let record = sample_record();
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        idem.record(record.clone()).await.unwrap();
        let got = idem.lookup(key).await.unwrap();
        assert!(got.is_some());
    }

    #[tokio::test]
    async fn exists_returns_true_after_record() {
        let idem = setup().await;
        let record = sample_record();
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        idem.record(record).await.unwrap();
        assert!(idem.exists(key).await.unwrap());
    }

    #[tokio::test]
    async fn lookup_unknown_key_returns_none() {
        let idem = setup().await;
        let g = SystemIdGen;
        let school = SchoolId::from_uuid(uuid::Uuid::new_v4());
        let key = IdempotencyRecord::composite_key(school, "x.y.z", g.next_idempotency_key());
        assert!(idem.lookup(key).await.unwrap().is_none());
    }

    // Serialize the interner tests in this module because
    // they share the process-wide `COMMAND_TYPE_CACHE` static.
    // Without this lock, parallel execution would let one
    // test's inserts leak into the other's count assertions.
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
