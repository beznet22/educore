//! The `Idempotency` sub-port — command deduplication.
//!
//! Per `docs/ports/storage.md` § 6 and
//! `docs/decisions/ADR-014-Idempotency.md`, every mutating
//! command carries an `IdempotencyKey` (a UUIDv7 supplied by the
//! caller). The engine stores the (school_id, command_type,
//! idempotency_key) → outcome mapping so a retried command
//! returns the same result without re-executing side effects.
//!
//! The trait exposes `record` (store a new outcome), `lookup`
//! (fetch the stored outcome for a key), and `exists` (cheap
//! check used by the dispatcher to decide whether to fast-path a
//! retry).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{IdempotencyKey, SchoolId};

/// The stored outcome of a previously-executed command. Per
/// `ADR-014-Idempotency.md`, the engine stores a small JSON
/// payload (the serialised command outcome) plus the version of
/// the engine that produced it, for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// The school the command targeted.
    pub school_id: SchoolId,
    /// The command type, e.g. `"academic.student.admit"`.
    pub command_type: &'static str,
    /// The caller's idempotency key.
    pub idempotency_key: IdempotencyKey,
    /// The serialised outcome of the original command. The wire
    /// format is engine-defined (JSON by default).
    pub outcome: bytes::Bytes,
    /// The schema version of the `outcome` payload.
    pub outcome_version: u32,
    /// Wall-clock time the record was written.
    pub recorded_at: educore_core::value_objects::Timestamp,
    /// The aggregate ids touched by the original command. Used
    /// by the dispatcher to detect "same idempotency key, but
    /// different target" misuse.
    pub affected_aggregate_ids: Vec<Uuid>,
}

impl IdempotencyRecord {
    /// Returns the composite key used to look up the record.
    #[must_use]
    pub fn composite_key(
        school_id: SchoolId,
        command_type: &'static str,
        key: IdempotencyKey,
    ) -> IdempotencyCompositeKey {
        IdempotencyCompositeKey {
            school_id,
            command_type,
            idempotency_key: key,
        }
    }
}

/// The lookup key for an idempotency record. Per
/// `ADR-014-Idempotency.md`, the key is the tuple
/// `(school_id, command_type, idempotency_key)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyCompositeKey {
    /// The school the command targeted.
    pub school_id: SchoolId,
    /// The command type, e.g. `"academic.student.admit"`.
    pub command_type: &'static str,
    /// The caller's idempotency key.
    pub idempotency_key: IdempotencyKey,
}

/// The structured outcome of an idempotent write.
///
/// Per `docs/decisions/ADR-014-Idempotency.md` and
/// `docs/schemas/command-schema.md` § 6, the engine treats a
/// duplicate-key write as a **non-fatal business outcome**, not
/// an error. The caller can recover the original outcome from
/// [`IdempotencyOutcome::Conflict::existing`] and return it
/// without re-executing the command — the engine's at-least-once
/// delivery semantics rely on this replay.
///
/// Distinction from [`Idempotency::record`]:
///
/// - [`Idempotency::record`] returns `Result<()>`. On success
///   the engine cannot distinguish "newly written" from "no-op
///   write of an identical outcome". On duplicate-with-different-
///   outcome it must surface `Err(DomainError::Conflict(_))`,
///   which conflates a business outcome with an
///   infrastructure failure.
/// - [`Idempotency::record_outcome`] (added by PR 4 Phase A of
///   `docs/audit_reports/remediation/09-quick-wins.md` / QW-12)
///   returns `Result<IdempotencyOutcome>`. The `Conflict`
///   variant carries the pre-existing record so the caller can
///   recover the original outcome without a second `lookup`
///   call. Only an adapter-level failure (lost connection,
///   deadlock, serialisation error, etc.) produces an
///   `Err(DomainError::Infrastructure)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdempotencyOutcome {
    /// The record was newly written. The adapter performed an
    /// `INSERT` (or equivalent) that did not collide with an
    /// existing row.
    Recorded,
    /// A record already exists for the same composite key.
    ///
    /// The pre-existing record is returned so the caller can
    /// recover the original outcome bytes and decide whether
    /// to replay them or surface a hard conflict to the API
    /// consumer. The carried record has the **same composite
    /// key** as the rejected write — the caller already knows
    /// that key, but the carried record also exposes the
    /// original outcome, `outcome_version`, and
    /// `affected_aggregate_ids` for replay.
    Conflict {
        /// The pre-existing record that caused the
        /// duplicate-key rejection. Same composite key as the
        /// rejected write.
        existing: IdempotencyRecord,
    },
}

/// The `Idempotency` sub-port trait. Storage adapters implement
/// this; the in-memory `educore-testkit` also implements it for
/// unit tests.
#[async_trait]
pub trait Idempotency: Send + Sync {
    /// Returns `Some(record)` if a previous command with the same
    /// `(school_id, command_type, idempotency_key)` is on file;
    /// `None` otherwise.
    async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>>;

    /// Returns `true` if a record exists for the given key. The
    /// default implementation calls `lookup` and checks
    /// `is_some`; adapters with a cheap existence check may
    /// override.
    async fn exists(&self, key: IdempotencyCompositeKey) -> Result<bool> {
        Ok(self.lookup(key).await?.is_some())
    }

    /// Stores `record`. Returns `Err(Conflict)` if a record with
    /// the same `(school_id, command_type, idempotency_key)`
    /// already exists with a different outcome. Returns `Ok(())`
    /// if the record is a no-op write (same key, same outcome
    /// hash) — the engine uses this for at-least-once delivery
    /// of retries.
    ///
    /// **Prefer [`Self::record_outcome`] for new code.** This
    /// method conflates two distinct cases under `Ok(())`
    /// ("newly written" vs "no-op write of identical outcome")
    /// and surfaces a duplicate-key collision as
    /// `Err(DomainError::Conflict)`, which the engine cannot
    /// distinguish from a true infrastructure failure. The
    /// default implementation of `record_outcome` delegates to
    /// this method and therefore inherits those limitations;
    /// adapters that can atomically detect a duplicate-key
    /// collision should override `record_outcome` instead.
    async fn record(&self, record: IdempotencyRecord) -> Result<()>;

    /// Stores `record` and returns a structured outcome that
    /// distinguishes "newly written" from "duplicate detected".
    ///
    /// Per `docs/decisions/ADR-014-Idempotency.md` and
    /// `docs/schemas/command-schema.md` § 6, a duplicate-key
    /// write is a **business outcome**, not an error: the
    /// engine should replay the original outcome rather than
    /// re-execute the command. This method exposes that
    /// outcome as a structured return value so the engine's
    /// command dispatcher can branch on it without parsing
    /// error strings.
    ///
    /// # Contract
    ///
    /// - First write for a `(school_id, command_type,
    ///   idempotency_key)` triple → `Ok(IdempotencyOutcome::Recorded)`.
    /// - Subsequent write for the same triple with an
    ///   equivalent outcome (same `outcome` bytes) →
    ///   `Ok(IdempotencyOutcome::Recorded)` (idempotent
    ///   re-insert; the engine treats this as a successful
    ///   retry of an already-executed command).
    /// - Subsequent write for the same triple with a
    ///   *different* outcome → `Ok(IdempotencyOutcome::Conflict
    ///   { existing })`. The carried record exposes the
    ///   original outcome for replay.
    /// - Adapter-level failure (lost connection, deadlock,
    ///   serialisation error, etc.) →
    ///   `Err(DomainError::Infrastructure)`.
    ///
    /// # Default implementation
    ///
    /// The default delegates to [`Self::record`] and assumes
    /// any successful write is a new write (`Recorded`).
    /// Adapters that cannot atomically detect a
    /// duplicate-key collision inherit this default and
    /// therefore can never return the `Conflict` variant —
    /// the engine will see `Recorded` for every retry. Such
    /// adapters must override this method to honour the
    /// port contract. The four shipped adapters
    /// (`storage-postgres`, `storage-mysql`, `storage-sqlite`,
    /// `storage-surrealdb`) override this method in PR 4
    /// Phase B of `docs/audit_reports/remediation/09-quick-wins.md`.
    async fn record_outcome(&self, record: IdempotencyRecord) -> Result<IdempotencyOutcome> {
        self.record(record).await?;
        Ok(IdempotencyOutcome::Recorded)
    }

    /// Purges idempotency records older than the configured
    /// retention window. The default implementation is a no-op
    /// (in-memory and other test backends do not need a sweep).
    /// SQL adapters override with a `DELETE ... WHERE
    /// recorded_at < $1` statement.
    async fn purge_older_than(
        &self,
        _school_id: SchoolId,
        _cutoff: educore_core::value_objects::Timestamp,
    ) -> Result<u64> {
        Ok(0)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use bytes::Bytes;

    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::value_objects::Timestamp;

    /// In-memory mock that detects duplicate-key collisions
    /// and surfaces them via the structured
    /// `IdempotencyOutcome::Conflict` variant. Mirrors the
    /// expected per-adapter behaviour once PR 4 Phase B lands.
    #[derive(Default)]
    struct MockIdempotency {
        records: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
    }

    #[async_trait]
    impl Idempotency for MockIdempotency {
        async fn lookup(&self, key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
            let store = self.records.lock().unwrap_or_else(|p| p.into_inner());
            Ok(store.get(&key).cloned())
        }

        async fn record(&self, record: IdempotencyRecord) -> Result<()> {
            let key = IdempotencyRecord::composite_key(
                record.school_id,
                record.command_type,
                record.idempotency_key,
            );
            let mut store = self.records.lock().unwrap_or_else(|p| p.into_inner());
            store.insert(key, record);
            Ok(())
        }

        async fn record_outcome(&self, record: IdempotencyRecord) -> Result<IdempotencyOutcome> {
            let key = IdempotencyRecord::composite_key(
                record.school_id,
                record.command_type,
                record.idempotency_key,
            );
            let mut store = self.records.lock().unwrap_or_else(|p| p.into_inner());
            if let Some(existing) = store.get(&key) {
                // Same outcome bytes: idempotent re-insert →
                // Recorded (engine's at-least-once retry case).
                if existing.outcome == record.outcome {
                    return Ok(IdempotencyOutcome::Recorded);
                }
                // Different outcome: a hard conflict, surface
                // the pre-existing record so the caller can
                // decide whether to replay its outcome or
                // surface a hard error to the API consumer.
                return Ok(IdempotencyOutcome::Conflict {
                    existing: existing.clone(),
                });
            }
            store.insert(key, record);
            Ok(IdempotencyOutcome::Recorded)
        }
    }

    #[test]
    fn composite_key_equality() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();
        let a = IdempotencyRecord::composite_key(school, "academic.student.admit", key);
        let b = IdempotencyRecord::composite_key(school, "academic.student.admit", key);
        assert_eq!(a, b);
    }

    /// Regression test for QW-12 (PR 4 Phase A): an adapter
    /// that detects duplicate-key collisions via
    /// `record_outcome` MUST surface them as
    /// `IdempotencyOutcome::Conflict`, not as
    /// `Err(DomainError::Conflict(_))` and not as a silent
    /// `Recorded`. Closing audit finding PORT-STORE-011
    /// (port-level); per-adapter findings will close when
    /// PR 4 Phase B lands.
    #[test]
    fn record_outcome_returns_conflict_on_duplicate_key() {
        let mock = MockIdempotency::default();
        let g = SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();

        let first = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: Bytes::from_static(b"first-payload"),
            outcome_version: 1,
            recorded_at: Timestamp::now(),
            affected_aggregate_ids: Vec::new(),
        };
        let second = IdempotencyRecord {
            // Same composite key, different outcome bytes.
            outcome: Bytes::from_static(b"second-payload"),
            ..first.clone()
        };

        // First write: newly written → Recorded.
        let outcome = futures::executor::block_on(mock.record_outcome(first.clone()))
            .expect("first record_outcome must not fail");
        assert_eq!(
            outcome,
            IdempotencyOutcome::Recorded,
            "first write with a fresh composite key must report Recorded",
        );

        // Second write with the same composite key but a
        // different outcome: Conflict carrying the existing
        // record. The carried record exposes the FIRST
        // payload (the original), not the rejected second
        // payload — the engine uses this for outcome replay.
        let outcome = futures::executor::block_on(mock.record_outcome(second))
            .expect("second record_outcome must not fail");
        match outcome {
            IdempotencyOutcome::Conflict { existing } => {
                assert_eq!(
                    existing.outcome,
                    Bytes::from_static(b"first-payload"),
                    "Conflict::existing must carry the original outcome bytes, \
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

    /// Regression test for QW-12: a no-op re-insert (same
    /// composite key, same outcome bytes) MUST be reported
    /// as `Recorded`, not as `Conflict`. The engine relies
    /// on this case for at-least-once delivery of retries
    /// — a duplicate dispatch with an identical payload is
    /// not a business-level conflict, it is a successful
    /// retry of an already-executed command.
    #[test]
    fn record_outcome_returns_recorded_for_no_op_reinsert() {
        let mock = MockIdempotency::default();
        let g = SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();

        let record = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: Bytes::from_static(b"identical-payload"),
            outcome_version: 1,
            recorded_at: Timestamp::now(),
            affected_aggregate_ids: Vec::new(),
        };

        let first = futures::executor::block_on(mock.record_outcome(record.clone()))
            .expect("first record_outcome must not fail");
        assert_eq!(first, IdempotencyOutcome::Recorded);

        // Second write with identical composite key AND
        // identical outcome bytes: no-op reinsert →
        // Recorded (NOT Conflict).
        let second = futures::executor::block_on(mock.record_outcome(record))
            .expect("second record_outcome must not fail");
        assert_eq!(
            second,
            IdempotencyOutcome::Recorded,
            "no-op reinsert (same key + same outcome) must report Recorded, \
             not Conflict — engine relies on this for retry replay",
        );
    }

    /// Regression test for QW-12: the default implementation
    /// of `record_outcome` (inherited by adapters that do not
    /// override it) MUST compile, MUST return `Ok(...)` (never
    /// `Err`) on a successful underlying `record`, and MUST
    /// report `Recorded` — even though it cannot detect
    /// duplicate-key collisions. This is the documented
    /// fallback for adapters that lack atomic collision
    /// detection; PR 4 Phase B replaces it in the four
    /// shipped adapters.
    #[test]
    fn default_record_outcome_inherits_recorded_fallback() {
        /// Adapter that implements `record` but does NOT
        /// override `record_outcome`. Inherits the default
        /// impl on the trait, which delegates to `record`
        /// and returns `Recorded` on success.
        #[derive(Default)]
        struct NaiveAdapter {
            records: Mutex<HashMap<IdempotencyCompositeKey, IdempotencyRecord>>,
        }

        #[async_trait]
        impl Idempotency for NaiveAdapter {
            async fn lookup(
                &self,
                key: IdempotencyCompositeKey,
            ) -> Result<Option<IdempotencyRecord>> {
                let store = self.records.lock().unwrap_or_else(|p| p.into_inner());
                Ok(store.get(&key).cloned())
            }

            async fn record(&self, record: IdempotencyRecord) -> Result<()> {
                let key = IdempotencyRecord::composite_key(
                    record.school_id,
                    record.command_type,
                    record.idempotency_key,
                );
                self.records
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .insert(key, record);
                Ok(())
            }
            // Intentionally does NOT override record_outcome.
        }

        let naive = NaiveAdapter::default();
        let g = SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();
        let record = IdempotencyRecord {
            school_id: school,
            command_type: "academic.student.admit",
            idempotency_key: key,
            outcome: Bytes::from_static(b"payload"),
            outcome_version: 1,
            recorded_at: Timestamp::now(),
            affected_aggregate_ids: Vec::new(),
        };

        // The default impl delegates to `record` and reports
        // Recorded on success — even when the underlying
        // store silently overwrites an existing row. This is
        // the documented limitation of the default impl; PR 4
        // Phase B replaces it with collision-aware overrides.
        let first = futures::executor::block_on(naive.record_outcome(record.clone()))
            .expect("default record_outcome must not fail on a successful record");
        assert_eq!(first, IdempotencyOutcome::Recorded);
        let second = futures::executor::block_on(naive.record_outcome(record))
            .expect("default record_outcome must not fail on a successful record");
        assert_eq!(
            second,
            IdempotencyOutcome::Recorded,
            "default record_outcome cannot detect collisions and reports Recorded",
        );
    }
}
