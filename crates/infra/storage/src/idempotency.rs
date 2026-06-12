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
    async fn record(&self, record: IdempotencyRecord) -> Result<()>;

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
    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};

    #[test]
    fn composite_key_equality() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let key = g.next_idempotency_key();
        let a = IdempotencyRecord::composite_key(school, "academic.student.admit", key);
        let b = IdempotencyRecord::composite_key(school, "academic.student.admit", key);
        assert_eq!(a, b);
    }
}
