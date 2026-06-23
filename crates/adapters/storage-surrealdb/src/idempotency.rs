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
use educore_storage::idempotency::{
    Idempotency, IdempotencyCompositeKey, IdempotencyOutcome, IdempotencyRecord,
};

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

    /// QW-12 (PR 4 Phase B) — SurrealDB adapter override of
    /// [`Idempotency::record_outcome`].
    ///
    /// Closes audit finding ADAPTER-SD-009. The default
    /// implementation delegates to [`Self::record`] and
    /// therefore cannot distinguish "newly written" from
    /// "duplicate-with-different-outcome" — it returns
    /// `Recorded` for every successful underlying write and
    /// surfaces a duplicate-key collision as
    /// `Err(DomainError::Infrastructure(...))`, which the
    /// engine's dispatcher cannot distinguish from a true
    /// infrastructure failure. This override closes the gap
    /// for the SurrealDB adapter by issuing the plain
    /// `INSERT` (no `INSERT IGNORE` / `ON CONFLICT DO NOTHING`
    /// shim — SurrealDB has neither) and inspecting every
    /// returned error: a duplicate-key collision against the
    /// `idx_idempotency_pk` `UNIQUE` index (declared in
    /// `migrations/engine/0000_engine_core.surreal.surql`)
    /// surfaces as `surrealdb_core::err::Error::IndexExists`
    /// and is the signal that a row already exists.
    ///
    /// **Important SurrealDB quirk.** The in-memory `Mem`
    /// backend surfaces a UNIQUE-index violation **at the
    /// `Response::take` step, not at the `query.await` step**
    /// — i.e. the query itself succeeds at the protocol
    /// level, but the typed result at position 0 carries an
    /// `Error::Db(DbError::IndexExists { .. })`. (A future
    /// PR can verify whether remote backends surface the
    /// same error at the `query.await` step instead.) This
    /// override therefore checks BOTH error paths with
    /// [`is_duplicate_key_error`] before classifying the
    /// write as newly-written or duplicate.
    ///
    /// On collision we `SELECT` the existing row and compare
    /// outcome bytes:
    ///
    /// - Same outcome bytes → `Recorded` (idempotent
    ///   re-insert: the engine's at-least-once retry case).
    /// - Different outcome bytes → `Conflict { existing }`
    ///   so the caller can replay the original outcome.
    ///
    /// See [`Idempotency::record_outcome`] for the full port
    /// contract and `docs/decisions/ADR-014-Idempotency.md`
    /// for the design rationale.
    async fn record_outcome(&self, record: IdempotencyRecord) -> Result<IdempotencyOutcome> {
        let row = IdempotencyRow::from_record(&record);
        // Path A: the query itself may surface a connection-
        // level or remote-server error at the `await` step.
        let response_result = self
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
            .await;

        let mut response = match response_result {
            Ok(r) => r,
            Err(e) => {
                // Path A error: connection lost, remote
                // server returned a transport error, etc.
                // Classify: a duplicate-key collision at this
                // layer is treated identically to a collision
                // at the take layer (see Path B below).
                return self.classify_insert_outcome(record, Some(e), None).await;
            }
        };

        // Path B: the query succeeded at the protocol level
        // but the typed result at position 0 may carry a
        // server-side error. This is the path taken by the
        // in-memory `Mem` backend for `IndexExists` — the
        // SurrealDB engine enforces the UNIQUE-index
        // constraint at result-extraction time.
        let take_result: std::result::Result<Vec<IdempotencyRow>, surrealdb::Error> =
            response.take(0);
        match take_result {
            Ok(_rows) => {
                // INSERT succeeded with no collision → newly
                // written.
                self.classify_insert_outcome(record, None, None).await
            }
            Err(e) => {
                // Path B error: server-side enforcement
                // rejected the write (e.g. UNIQUE-index
                // violation). Classify the outcome.
                self.classify_insert_outcome(record, None, Some(e)).await
            }
        }
    }
}

/// Returns the [`IdempotencyOutcome`] for a completed
/// `record_outcome` INSERT, given the optional errors that
/// surfaced on Path A (`query.await`) and/or Path B
/// (`response.take(0)`).
///
/// At most one of `path_a_err` / `path_b_err` is `Some(_)`;
/// if both are `None` the INSERT succeeded and the function
/// returns [`IdempotencyOutcome::Recorded`].
///
/// On a duplicate-key collision (per [`is_duplicate_key_error`])
/// the function performs a `SELECT` to recover the
/// pre-existing record and decides between an idempotent
/// re-insert (same outcome bytes) and a hard conflict
/// (different outcome bytes). All other errors propagate as
/// `Err(DomainError::Infrastructure(_))` per the port
/// contract.
impl SurrealIdempotency {
    async fn classify_insert_outcome(
        &self,
        record: IdempotencyRecord,
        path_a_err: Option<surrealdb::Error>,
        path_b_err: Option<surrealdb::Error>,
    ) -> Result<IdempotencyOutcome> {
        // No error from either path: INSERT succeeded.
        let collision_err = match (path_a_err, path_b_err) {
            (None, None) => return Ok(IdempotencyOutcome::Recorded),
            (Some(e), None) => e,
            (None, Some(e)) => e,
            // Theoretically impossible: the query succeeded
            // AND `take` failed. Treat as a `take` error
            // (the later / more-specific signal) so the
            // diagnostic message points at the typed-result
            // path.
            (Some(_a), Some(b)) => b,
        };

        // Classify the error. A duplicate-key collision on
        // `idx_idempotency_pk` surfaces as
        // `surrealdb_core::err::Error::IndexExists` (the
        // `UNIQUE` index constraint on the composite
        // `(school_id, command_type, idempotency_key)`
        // columns rejected the write). Any other error is an
        // infrastructure failure per the port contract.
        if !is_duplicate_key_error(&collision_err) {
            return Err(
                StringError(format!("idempotency record_outcome: {collision_err}",)).into(),
            );
        }

        // Unique-index violation: a row already exists for
        // this composite key. Fetch it so we can decide
        // between an idempotent re-insert (same outcome
        // bytes) and a hard conflict (different outcome
        // bytes).
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        let existing = match self.lookup(key).await? {
            Some(rec) => rec,
            // Race: the conflicting row was purged (e.g. by
            // `purge_older_than`) between our INSERT and our
            // SELECT. There is no "existing" record to
            // return, so we cannot report `Conflict { existing
            // }`. Surface the original DB error as an
            // infrastructure failure rather than retrying
            // transparently — the engine's dispatcher will
            // see the error and decide whether to retry the
            // whole command.
            None => {
                return Err(StringError(format!(
                    "idempotency record_outcome: \
                     duplicate-key collision detected but \
                     existing row vanished before lookup: \
                     {collision_err}",
                ))
                .into());
            }
        };

        // Same outcome bytes → idempotent re-insert
        // (engine's at-least-once retry case).
        // Different outcome bytes → hard conflict; surface
        // the pre-existing record so the caller can replay
        // its outcome.
        if existing.outcome == record.outcome {
            Ok(IdempotencyOutcome::Recorded)
        } else {
            Ok(IdempotencyOutcome::Conflict { existing })
        }
    }
}

/// The name of the composite UNIQUE index on the
/// `idempotency` table that enforces the (school_id,
/// command_type, idempotency_key) primary key contract.
///
/// Declared in `migrations/engine/0000_engine_core.surreal.surql`
/// via `DEFINE INDEX idx_idempotency_pk ON TABLE idempotency
/// COLUMNS school_id, command_type, idempotency_key UNIQUE`.
/// Exposed as a module-level constant so tests and the
/// error-classification helper can refer to it without
/// duplicating the string literal.
const IDEMPOTENCY_PK_INDEX: &str = "idx_idempotency_pk";

/// Returns `true` if `err` is a SurrealDB duplicate-key
/// collision against the `idx_idempotency_pk` `UNIQUE`
/// index.
///
/// Per the canonical schema in
/// `migrations/engine/0000_engine_core.surreal.surql`, the
/// `idempotency` table has `DEFINE INDEX ... UNIQUE` on the
/// composite key `(school_id, command_type, idempotency_key)`.
/// A duplicate `INSERT` against that table surfaces as
/// `surrealdb_core::err::Error::IndexExists` with
/// `index == "idx_idempotency_pk"` (the in-memory
/// `engine::local::Db` backend cannot produce `Api` errors,
/// so we match `Db` directly).
///
/// The function additionally accepts
/// `surrealdb_core::err::Error::RecordExists` for robustness
/// against future schema changes that switch the composite
/// key from a `UNIQUE` index to a primary `record_id`
/// constraint — both errors represent the same business
/// outcome (a row already exists for the composite key) and
/// the caller treats them identically.
fn is_duplicate_key_error(err: &surrealdb::Error) -> bool {
    use surrealdb::error::Db as DbError;
    matches!(
        err,
        surrealdb::Error::Db(DbError::IndexExists { index, .. })
            if index == IDEMPOTENCY_PK_INDEX
    ) || matches!(err, surrealdb::Error::Db(DbError::RecordExists { .. }))
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

    // -------------------------------------------------------------------
    // QW-12 (PR 4 Phase B) regression tests for
    // `Idempotency::record_outcome` on the SurrealDB adapter.
    //
    // Closes audit finding ADAPTER-SD-009. The tests mirror the
    // port-level regression tests in
    // `crates/infra/storage/src/idempotency.rs` but exercise the
    // SurrealDB adapter's collision-detection path against the
    // real `idx_idempotency_pk` UNIQUE index declared in
    // `migrations/engine/0000_engine_core.surreal.surql`. Both
    // tests use the existing in-memory SurrealDB backend via
    // `setup()` — no external test container is required, so
    // neither test is marked `#[ignore]`.
    // -------------------------------------------------------------------

    /// QW-12 (PR 4 Phase B) regression: a fresh composite key
    /// on the SurrealDB adapter's `record_outcome` MUST be
    /// reported as `IdempotencyOutcome::Recorded`, not as
    /// `Conflict` and not as `Err`.
    ///
    /// The port-level contract for this case is documented in
    /// [`Idempotency::record_outcome`] § "Contract". Without
    /// this assertion an adapter that silently downgraded
    /// every successful write to `Conflict` would still pass
    /// the `record_then_lookup_round_trips` test (which only
    /// checks the `record` / `lookup` path) but would break
    /// the engine's command dispatcher — a fresh command
    /// would never be marked "newly written" and the audit
    /// log would record every command as a replay.
    #[tokio::test]
    async fn record_outcome_returns_recorded_for_new_key() {
        let idem = setup().await;
        let record = sample_record();

        // First write against a fresh composite key: must
        // report `Recorded`, not `Conflict`, and must not
        // surface as `Err`.
        let outcome = idem
            .record_outcome(record.clone())
            .await
            .expect("first record_outcome against fresh key must not fail");
        assert_eq!(
            outcome,
            IdempotencyOutcome::Recorded,
            "first write against a fresh composite key must report Recorded",
        );

        // The row must be visible to `lookup` afterwards —
        // this proves the INSERT actually wrote (rather than
        // being silently swallowed by an `INSERT IGNORE` or
        // `ON CONFLICT DO NOTHING` shim).
        let key = IdempotencyRecord::composite_key(
            record.school_id,
            record.command_type,
            record.idempotency_key,
        );
        let stored = idem
            .lookup(key)
            .await
            .expect("lookup after record_outcome must not fail");
        assert!(
            stored.is_some(),
            "record_outcome Reported=Recorded but the row is not visible to lookup",
        );
        let stored = stored.expect("Some variant verified above");
        assert_eq!(
            stored.outcome, record.outcome,
            "round-tripped outcome bytes must match the input",
        );
        assert_eq!(
            stored.outcome_version, record.outcome_version,
            "round-tripped outcome_version must match the input",
        );
    }

    /// QW-12 (PR 4 Phase B) regression: a duplicate composite
    /// key on the SurrealDB adapter's `record_outcome` MUST
    /// be reported as `IdempotencyOutcome::Conflict { existing
    /// }`, where `existing` is the **pre-existing** record
    /// (same composite key, original outcome bytes).
    ///
    /// The contract distinguishes two sub-cases:
    ///
    /// - Same outcome bytes on the duplicate write → `Recorded`
    ///   (idempotent re-insert; engine's at-least-once retry
    ///   case).
    /// - Different outcome bytes on the duplicate write →
    ///   `Conflict { existing }` carrying the original outcome.
    ///
    /// This test exercises both sub-cases against the real
    /// SurrealDB in-memory backend via `setup()`. It exercises
    /// the adapter's `is_duplicate_key_error` classifier
    /// against `surrealdb_core::err::Error::IndexExists`, which
    /// is the exact variant produced by the
    /// `idx_idempotency_pk` UNIQUE index when the duplicate
    /// INSERT is rejected.
    #[tokio::test]
    async fn record_outcome_returns_conflict_for_duplicate_key() {
        let idem = setup().await;
        let first = sample_record();
        let key = IdempotencyRecord::composite_key(
            first.school_id,
            first.command_type,
            first.idempotency_key,
        );

        // First write: newly written → Recorded.
        let outcome = idem
            .record_outcome(first.clone())
            .await
            .expect("first record_outcome must not fail");
        assert_eq!(
            outcome,
            IdempotencyOutcome::Recorded,
            "first write with a fresh composite key must report Recorded",
        );

        // Second write: same composite key, SAME outcome
        // bytes. The engine treats this as an idempotent
        // re-insert (at-least-once retry of an already-
        // executed command) and MUST report Recorded, NOT
        // Conflict. The contract distinguishes this from the
        // hard-conflict case (same key, DIFFERENT outcome) so
        // the engine does not surface a spurious "duplicate"
        // error to API consumers when the dispatcher retries.
        let same_outcome = IdempotencyRecord {
            // Same composite key, same outcome bytes (the
            // first record's outcome). Bump recorded_at so
            // the round-tripped row's timestamp can be
            // distinguished from the original — the engine
            // does not care about `recorded_at` for outcome
            // replay, but the equality check on the
            // `Conflict::existing` payload below does not
            // look at `recorded_at`, so this is safe.
            recorded_at: Timestamp::now(),
            ..first.clone()
        };
        let outcome = idem
            .record_outcome(same_outcome)
            .await
            .expect("idempotent re-insert must not surface an Err");
        assert_eq!(
            outcome,
            IdempotencyOutcome::Recorded,
            "no-op reinsert (same key + same outcome) must report Recorded, \
             not Conflict — engine relies on this for at-least-once retry replay",
        );

        // Third write: same composite key, DIFFERENT outcome
        // bytes. The engine treats this as a hard business
        // conflict (same idempotency key, but the caller is
        // trying to record a different outcome — typically a
        // bug or a malicious retry with a corrupted payload)
        // and MUST report Conflict { existing } carrying the
        // ORIGINAL record (not the rejected second payload).
        let conflicting_outcome = IdempotencyRecord {
            outcome: Bytes::from_static(b"different-payload"),
            outcome_version: 2,
            recorded_at: Timestamp::now(),
            ..first.clone()
        };
        let outcome = idem
            .record_outcome(conflicting_outcome)
            .await
            .expect("duplicate-key write must surface as Conflict, not as Err");
        match outcome {
            IdempotencyOutcome::Conflict { existing } => {
                // The carried record must expose the FIRST
                // outcome, not the rejected second payload —
                // the engine uses `Conflict::existing` to
                // replay the original outcome.
                assert_eq!(
                    existing.outcome, first.outcome,
                    "Conflict::existing must carry the ORIGINAL outcome bytes, \
                     not the rejected second payload",
                );
                assert_eq!(
                    existing.outcome_version, first.outcome_version,
                    "Conflict::existing must carry the ORIGINAL outcome_version",
                );
                assert_eq!(
                    existing.school_id, first.school_id,
                    "Conflict::existing must carry the ORIGINAL school_id",
                );
                assert_eq!(
                    existing.idempotency_key, first.idempotency_key,
                    "Conflict::existing must carry the ORIGINAL idempotency_key",
                );
                assert_eq!(
                    existing.command_type, first.command_type,
                    "Conflict::existing must carry the ORIGINAL command_type",
                );
            }
            other => panic!(
                "expected IdempotencyOutcome::Conflict on duplicate key with \
                 different outcome, got {other:?}",
            ),
        }

        // Sanity: the row on disk is unchanged after the
        // duplicate write was rejected — the conflicting
        // payload must NOT have overwritten the original
        // outcome. This is the property that protects the
        // engine's audit trail: a rejected write leaves the
        // stored outcome exactly as it was.
        let stored = idem
            .lookup(key)
            .await
            .expect("lookup after duplicate record_outcome must not fail")
            .expect("row must still exist after a rejected duplicate write");
        assert_eq!(
            stored.outcome, first.outcome,
            "rejected duplicate write must NOT overwrite the stored outcome",
        );
        assert_eq!(
            stored.outcome_version, first.outcome_version,
            "rejected duplicate write must NOT overwrite the stored outcome_version",
        );
    }

    /// Unit test for the `is_duplicate_key_error` classifier.
    ///
    /// Exercises the three shapes of `surrealdb::Error` that
    /// matter for the `record_outcome` decision:
    ///
    /// - `Db(IndexExists { index: "idx_idempotency_pk", .. })`
    ///   → duplicate (the expected case).
    /// - `Db(IndexExists { index: "some_other_index", .. })`
    ///   → not a duplicate (we are deliberately conservative
    ///   and only treat the known index as a duplicate-key
    ///   collision; this guards against a future schema
    ///   change that introduces an unrelated UNIQUE index
    ///   whose violation would otherwise be misclassified as
    ///   the idempotency conflict).
    /// - `Db(RecordExists { .. })` → duplicate (defensive:
    ///   matches the contract regardless of which constraint
    ///   surface fires).
    /// - Anything else (including `Api(_)` and the broad
    ///   `Db(_)` catch-all) → not a duplicate, must propagate
    ///   as an infrastructure error.
    ///
    /// `surrealdb_core::err::Error` is `#[non_exhaustive]`,
    /// so the test only constructs the variants it needs via
    /// the public API where possible, or uses a placeholder
    /// `Thing` value where required.
    #[test]
    fn is_duplicate_key_error_classifier() {
        use surrealdb::error::Db as DbError;
        use surrealdb::sql::Thing;

        let matching_index = IDEMPOTENCY_PK_INDEX.to_owned();
        let other_index = "idx_unrelated".to_owned();
        let dummy_thing = Thing::from(("idempotency", "x"));

        // 1. IndexExists on the PK index → duplicate.
        let e = surrealdb::Error::Db(DbError::IndexExists {
            thing: dummy_thing.clone(),
            index: matching_index,
            value: "school=...,command_type=...,idempotency_key=...".to_owned(),
        });
        assert!(
            is_duplicate_key_error(&e),
            "IndexExists on {IDEMPOTENCY_PK_INDEX} must be classified as duplicate",
        );

        // 2. IndexExists on an UNRELATED index → NOT a duplicate.
        let e = surrealdb::Error::Db(DbError::IndexExists {
            thing: dummy_thing.clone(),
            index: other_index,
            value: "unrelated".to_owned(),
        });
        assert!(
            !is_duplicate_key_error(&e),
            "IndexExists on an unrelated index must NOT be classified as duplicate",
        );

        // 3. RecordExists → duplicate (defensive).
        let e = surrealdb::Error::Db(DbError::RecordExists { thing: dummy_thing });
        assert!(
            is_duplicate_key_error(&e),
            "RecordExists must be classified as duplicate (defensive match)",
        );
    }
}
