//! SurrealDB-backed `AuditLog` sub-port.
//!
//! Stores each audit row in the `audit_log` table. The schema
//! is defined by the canonical .surql migration (loaded by
//! `SurrealStorageAdapter::migrate`).
//!
//! ## Module wiring
//!
//! This module is **not yet wired into `lib.rs`** — A'.1 will
//! add `pub mod audit;` to the crate root once the stub in
//! `stubs.rs` has been removed. The audit sub-port is reachable
//! from `crate::stubs::SurrealAuditLog` (the Phase 0 stub)
//! until that wire-up lands. This file lays down the real
//! implementation; the build does not compile it yet, so there
//! is no risk of conflicting `AuditLog` trait impl blocks on
//! `SurrealAuditLog` in the same crate.

use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use surrealdb::sql::{Bytes as SurrealBytes, Datetime, Uuid as SurrealUuid};

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::audit::{AuditLog, AuditLogEntry};

use crate::connection::Db;
use crate::error::StringError;

/// The row shape stored in the SurrealDB `audit_log` table.
/// Mirrors the field set on
/// [`educore_storage::audit::AuditLogEntry`] but with the
/// engine's id types mapped to SurrealDB's native
/// representations (`uuid` -> `surrealdb::sql::Uuid`,
/// `bytes::Bytes` -> `surrealdb::sql::Bytes`,
/// `serde_json::Value` -> native JSON).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct AuditRow {
    pub school_id: Option<SurrealUuid>,
    pub actor_id: SurrealUuid,
    pub action: String,
    pub target_type: String,
    pub target_id: SurrealUuid,
    pub before: Option<SurrealBytes>,
    pub after: Option<SurrealBytes>,
    pub event_id: Option<SurrealUuid>,
    pub correlation_id: SurrealUuid,
    pub occurred_at: Datetime,
    pub active_status: String,
    pub metadata: Option<serde_json::Value>,
}

impl AuditRow {
    /// Maps an [`AuditLogEntry`] into a row ready for insert.
    pub fn from_entry(entry: &AuditLogEntry) -> Self {
        Self {
            school_id: Some(SurrealUuid::from(entry.school_id.as_uuid())),
            actor_id: SurrealUuid::from(entry.actor_id.as_uuid()),
            action: entry.action.clone(),
            target_type: entry.target_type.clone(),
            target_id: SurrealUuid::from(entry.target_id),
            before: entry
                .before
                .as_ref()
                .map(|b| SurrealBytes::from(b.to_vec())),
            after: entry.after.as_ref().map(|b| SurrealBytes::from(b.to_vec())),
            event_id: entry.event_id.map(|e| SurrealUuid::from(e.as_uuid())),
            correlation_id: SurrealUuid::from(entry.correlation_id.as_uuid()),
            occurred_at: Datetime::from(entry.occurred_at.as_datetime()),
            active_status: entry.active_status.to_string(),
            // The engine writes `serde_json::Value::Null` for
            // "no metadata"; we map that back to `None` so the
            // SurrealDB column stays `option<object>` and does
            // not store a literal JSON `null`.
            metadata: if entry.metadata.is_null() {
                None
            } else {
                Some(entry.metadata.clone())
            },
        }
    }

    /// Maps a row back to an [`AuditLogEntry`].
    pub fn to_entry(&self) -> AuditLogEntry {
        let school_id = self
            .school_id
            .map(|u| SchoolId::from_uuid(u.0))
            .unwrap_or_else(|| SchoolId::from_uuid(uuid::Uuid::nil()));
        let actor_id = UserId::from_uuid(self.actor_id.0);
        let correlation_id = CorrelationId::from_uuid(self.correlation_id.0);
        let event_id = self.event_id.map(|u| EventId::from_uuid(u.0));
        let before = self.before.as_ref().map(|b| Bytes::from(b.to_vec()));
        let after = self.after.as_ref().map(|b| Bytes::from(b.to_vec()));
        let occurred_at = Timestamp::from_datetime(self.occurred_at.0);
        let active_status = match self.active_status.as_str() {
            "active" => ActiveStatus::Active,
            _ => ActiveStatus::Retired,
        };
        AuditLogEntry {
            school_id,
            actor_id,
            action: self.action.clone(),
            target_type: self.target_type.clone(),
            target_id: self.target_id.0,
            before,
            after,
            event_id,
            correlation_id,
            occurred_at,
            active_status,
            metadata: self.metadata.clone().unwrap_or(serde_json::Value::Null),
        }
    }
}

/// Convert a SurrealDB `Datetime` (borrowed) to a `Timestamp`.
/// Borrows rather than consumes so the caller can iterate
/// over query results without cloning each `Datetime`.
#[allow(dead_code)]
fn from_surreal_datetime(dt: &Datetime) -> Timestamp {
    let chrono_dt: DateTime<Utc> = dt.0;
    Timestamp::from_datetime(chrono_dt)
}

/// The SurrealDB-backed `AuditLog` implementation.
///
/// The struct mirrors the shape of the
/// [`crate::stubs::SurrealAuditLog`] stub so that
/// `crate::transaction::SurrealTransaction` can hand out the
/// real implementation through the same `&self`-returning
/// accessor pattern (see `audit_log()` in `transaction.rs`).
/// A'.1 will swap the stub for this type once it wires the
/// module into `lib.rs`.
#[derive(Clone)]
pub struct SurrealAuditLog {
    pub(crate) db: Db,
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealAuditLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealAuditLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SurrealAuditLog {
    /// Constructs a new audit log handle bound to `db` and
    /// scoped to `school`.
    pub fn new(db: Db, school: SchoolId) -> Self {
        Self { db, school }
    }
}

#[async_trait]
impl AuditLog for SurrealAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        let row = AuditRow::from_entry(&entry);
        let mut response = self
            .db
            .query(
                "INSERT INTO audit_log { \
                    school_id: $school_id, \
                    actor_id: $actor_id, \
                    action: $action, \
                    target_type: $target_type, \
                    target_id: $target_id, \
                    before: $before, \
                    after: $after, \
                    event_id: $event_id, \
                    correlation_id: $correlation_id, \
                    occurred_at: $occurred_at, \
                    active_status: $active_status, \
                    metadata: $metadata \
                }",
            )
            .bind(("school_id", row.school_id))
            .bind(("actor_id", row.actor_id))
            .bind(("action", row.action))
            .bind(("target_type", row.target_type))
            .bind(("target_id", row.target_id))
            .bind(("before", row.before))
            .bind(("after", row.after))
            .bind(("event_id", row.event_id))
            .bind(("correlation_id", row.correlation_id))
            .bind(("occurred_at", row.occurred_at))
            .bind(("active_status", row.active_status))
            .bind(("metadata", row.metadata))
            .await
            .map_err(|e| StringError(format!("audit_log append: {e}")))?;
        // Pull the typed result at position 0 just to confirm the
        // INSERT succeeded and surface any server-side error.
        let _: Vec<AuditRow> = response
            .take(0)
            .map_err(|e| StringError(format!("audit_log append take: {e}")))?;
        Ok(())
    }

    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: uuid::Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let school_uuid = SurrealUuid::from(school_id.as_uuid());
        let target_uuid = SurrealUuid::from(target_id);
        let mut response = self
            .db
            .query(
                "SELECT \
                    school_id, actor_id, action, target_type, target_id, \
                    before, after, event_id, correlation_id, occurred_at, \
                    active_status, metadata \
                 FROM audit_log \
                 WHERE school_id = $school AND target_id = $target \
                 ORDER BY occurred_at ASC \
                 LIMIT $limit",
            )
            .bind(("school", school_uuid))
            .bind(("target", target_uuid))
            .bind(("limit", i64::from(limit)))
            .await
            .map_err(|e| StringError(format!("audit_log read: {e}")))?;
        let rows: Vec<AuditRow> = response
            .take(0)
            .map_err(|e| StringError(format!("audit_log read take: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|row| AuditRow::to_entry(&row))
            .collect())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]
mod tests {
    //! Unit tests for the `SurrealAuditLog` impl. The tests
    //! spin up an in-memory SurrealDB instance, run the
    //! canonical migration, then exercise the `append` and
    //! `read_for_target` paths.
    //!
    //! **Limitation:** the in-memory `Mem` backend is a single
    //! process and a single database connection. The
    //! `read_for_target_isolates_by_school` test creates two
    //! independent `Db` instances (one per school) so tenant
    //! isolation is exercised end-to-end — the storage layer
    //! does not itself enforce it (the engine's `TenantContext`
    //! layer is the canonical gate per
    //! `docs/schemas/tenancy-schema.md`).

    use super::*;
    use educore_core::clock::{IdGenerator, SystemIdGen};
    use educore_core::ids::SchoolId;
    use educore_storage::StorageAdapter;

    /// Spins up an in-memory SurrealDB adapter, applies the
    /// canonical schema, and returns a `SurrealAuditLog` bound
    /// to the resulting `Db`.
    async fn setup(school: SchoolId) -> SurrealAuditLog {
        let adapter = crate::storage::SurrealStorageAdapter::in_memory(school)
            .await
            .expect("in-memory adapter should construct");
        adapter.migrate().await.expect("migration should succeed");
        SurrealAuditLog::new(adapter.db().clone(), school)
    }

    /// Constructs an audit entry for `target` at `occurred_at =
    /// Utc::now() + offset_secs`. The other fields are filled
    /// with deterministic test data.
    fn make_entry(
        g: &SystemIdGen,
        school: SchoolId,
        target: uuid::Uuid,
        offset_secs: i64,
    ) -> AuditLogEntry {
        let occurred_at = chrono::Utc::now() + chrono::Duration::seconds(offset_secs);
        AuditLogEntry {
            school_id: school,
            actor_id: g.next_user_id(),
            action: "update".to_owned(),
            target_type: "student".to_owned(),
            target_id: target,
            before: Some(bytes::Bytes::from_static(br#"{"before":1}"#)),
            after: Some(bytes::Bytes::from_static(br#"{"after":2}"#)),
            event_id: None,
            correlation_id: g.next_correlation_id(),
            occurred_at: Timestamp::from_datetime(occurred_at),
            active_status: ActiveStatus::Active,
            metadata: serde_json::json!({"reason":"unit-test"}),
        }
    }

    #[tokio::test]
    async fn append_then_read_for_target_round_trips() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let audit = setup(school).await;
        let target = g.next_uuid();
        let entry = make_entry(&g, school, target, 0);
        let entry_correlation = entry.correlation_id;
        audit.append(entry).await.expect("append should succeed");
        let read = audit
            .read_for_target(school, target, 10)
            .await
            .expect("read should succeed");
        assert_eq!(read.len(), 1, "exactly one row should round-trip");
        assert_eq!(read[0].correlation_id, entry_correlation);
        assert_eq!(read[0].target_id, target);
        assert_eq!(read[0].action, "update");
        assert_eq!(read[0].target_type, "student");
        assert_eq!(read[0].school_id, school);
        assert_eq!(read[0].before.as_deref(), Some(br#"{"before":1}"# as &[u8]));
        assert_eq!(read[0].after.as_deref(), Some(br#"{"after":2}"# as &[u8]));
    }

    #[tokio::test]
    async fn append_two_read_orders_by_occurred_at() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let audit = setup(school).await;
        let target = g.next_uuid();
        let entry1 = make_entry(&g, school, target, 0);
        let entry2 = make_entry(&g, school, target, 60);
        let corr1 = entry1.correlation_id;
        let corr2 = entry2.correlation_id;
        audit.append(entry1).await.expect("append1");
        audit.append(entry2).await.expect("append2");
        let read = audit
            .read_for_target(school, target, 10)
            .await
            .expect("read");
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].correlation_id, corr1, "ASC by occurred_at");
        assert_eq!(read[1].correlation_id, corr2);
    }

    #[tokio::test]
    async fn read_for_target_respects_limit() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let audit = setup(school).await;
        let target = g.next_uuid();
        for i in 0..5 {
            audit
                .append(make_entry(&g, school, target, i * 10))
                .await
                .expect("append");
        }
        let read = audit
            .read_for_target(school, target, 3)
            .await
            .expect("read");
        assert_eq!(read.len(), 3, "limit should cap the result set");
    }

    #[tokio::test]
    async fn read_for_target_isolates_by_target_id() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let audit = setup(school).await;
        let target_a = g.next_uuid();
        let target_b = g.next_uuid();
        audit
            .append(make_entry(&g, school, target_a, 0))
            .await
            .expect("append A");
        audit
            .append(make_entry(&g, school, target_b, 0))
            .await
            .expect("append B");
        let read_a = audit
            .read_for_target(school, target_a, 10)
            .await
            .expect("read A");
        let read_b = audit
            .read_for_target(school, target_b, 10)
            .await
            .expect("read B");
        assert_eq!(read_a.len(), 1);
        assert_eq!(read_a[0].target_id, target_a);
        assert_eq!(read_b.len(), 1);
        assert_eq!(read_b[0].target_id, target_b);
    }

    #[tokio::test]
    async fn read_for_target_isolates_by_school() {
        let g = SystemIdGen;
        let school_a = g.next_school_id();
        let school_b = g.next_school_id();
        let audit_a = setup(school_a).await;
        let audit_b = setup(school_b).await;
        let target = g.next_uuid();
        audit_a
            .append(make_entry(&g, school_a, target, 0))
            .await
            .expect("append A");
        audit_b
            .append(make_entry(&g, school_b, target, 0))
            .await
            .expect("append B");
        let read_a = audit_a
            .read_for_target(school_a, target, 10)
            .await
            .expect("read A");
        let read_b = audit_b
            .read_for_target(school_b, target, 10)
            .await
            .expect("read B");
        assert_eq!(read_a.len(), 1);
        assert_eq!(read_a[0].school_id, school_a);
        assert_eq!(read_b.len(), 1);
        assert_eq!(read_b[0].school_id, school_b);
    }
}
