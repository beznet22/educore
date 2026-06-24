//! The `AuditLog` sub-port — append-only audit history.
//!
//! Per `docs/ports/storage.md` § 5 and `docs/schemas/audit-schema.md`:
//! every state-changing command writes one audit row in the same
//! transaction as the mutation. The audit log is write-only at
//! the API boundary — no update or delete is supported. The
//! engine enforces this by only exposing `append` and `read` on
//! the trait.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};

/// Custom serde adapter for `bytes::Bytes` that round-trips
/// through `Vec<u8>` (see the same module in `outbox.rs` for
/// the rationale).
#[allow(dead_code)] // referenced via `#[serde(with = "...")]`
mod bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(value: &bytes::Bytes, ser: S) -> Result<S::Ok, S::Error> {
        value.as_ref().serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<bytes::Bytes, D::Error> {
        let vec = Vec::<u8>::deserialize(de)?;
        Ok(bytes::Bytes::from(vec))
    }
}

/// Custom serde adapter for `Option<bytes::Bytes>` that
/// round-trips through `Option<Vec<u8>>`. Required because
/// `#[serde(with = "...")]` does not transparently adapt
/// `Option<T>` when the inner type has a custom adapter.
#[allow(dead_code)] // referenced via `#[serde(with = "...")]`
mod option_bytes_via_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<bytes::Bytes>,
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        value.as_ref().map(|b| b.as_ref()).serialize(ser)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Option<bytes::Bytes>, D::Error> {
        let opt = Option::<Vec<u8>>::deserialize(de)?;
        Ok(opt.map(bytes::Bytes::from))
    }
}

/// One row in the audit log. The struct is intentionally narrow:
/// `before` and `after` carry serialised snapshots of the
/// aggregate (typically JSON bytes). The `action` is a short verb (`"create"`,
/// `"update"`, `"delete"`, `"approve"`, …); `target_type` is the
/// aggregate name (`"student"`); `target_id` is the aggregate id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// The school the audit row belongs to.
    pub school_id: SchoolId,
    /// The user (or `SYSTEM`) that performed the action.
    pub actor_id: UserId,
    /// The action verb, e.g. `"create"`, `"update"`, `"delete"`,
    /// `"approve"`, `"reject"`, `"login"`. `String` (not
    /// `&'static str`) so the type can be deserialised.
    pub action: String,
    /// The aggregate type (e.g. `"student"`). `String` (not
    /// `&'static str`) so the type can be deserialised.
    pub target_type: String,
    /// The aggregate id this audit row is about.
    pub target_id: Uuid,
    /// Serialised snapshot of the aggregate **before** the
    /// action. `null` for create actions.
    #[serde(with = "option_bytes_via_vec", default)]
    pub before: Option<bytes::Bytes>,
    /// Serialised snapshot of the aggregate **after** the action.
    /// `null` for delete actions.
    #[serde(with = "option_bytes_via_vec", default)]
    pub after: Option<bytes::Bytes>,
    /// The event that this audit row was written alongside (if
    /// any). Lets auditors correlate audit rows to event log
    /// rows.
    pub event_id: Option<EventId>,
    /// The correlation id of the originating request.
    pub correlation_id: CorrelationId,
    /// Wall-clock time of the audit write.
    pub occurred_at: Timestamp,
    /// Soft-delete flag for the audit row itself. Audit rows are
    /// never hard-deleted; this is `Retired` when an auditor
    /// marks a row as superseded.
    pub active_status: ActiveStatus,
    /// Free-form metadata. Per `docs/schemas/audit-schema.md`, the
    /// `metadata` column is open-ended; common keys are
    /// `"reason"`, `"ticket"`, `"request_id"`. Carried as raw
    /// JSON bytes (mirroring the `before` / `after` snapshot
    /// encoding) so the port stays serialisation-agnostic.
    pub metadata: bytes::Bytes,
}

impl AuditLogEntry {
    /// Constructs an audit row for a create action (no `before`).
    #[must_use]
    pub fn create(
        school_id: SchoolId,
        actor_id: UserId,
        target_type: &str,
        target_id: Uuid,
        after: bytes::Bytes,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id,
            actor_id,
            action: "create".to_owned(),
            target_type: target_type.to_owned(),
            target_id,
            before: None,
            after: Some(after),
            event_id: None,
            correlation_id,
            occurred_at: Timestamp::now(),
            active_status: ActiveStatus::Active,
            metadata: bytes::Bytes::new(),
        }
    }
}

/// The `AuditLog` sub-port trait. Append-only at the API boundary.
#[async_trait]
pub trait AuditLog: Send + Sync {
    /// Appends `entry` to the audit log in the current
    /// transaction. Per `docs/schemas/audit-schema.md` § 6 the
    /// audit log is write-only — no `update` or `delete` method
    /// exists.
    ///
    /// # Errors
    /// - `Infrastructure` for any underlying storage error.
    async fn append(&self, entry: AuditLogEntry) -> Result<()>;

    /// Returns the audit rows for `target_id` ordered by
    /// `occurred_at` ascending. The cap is enforced by the
    /// storage adapter; values above 1000 should be paginated
    /// by the caller.
    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>>;
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
    fn create_entry_has_no_before() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let target = g.next_uuid();
        let corr = g.next_correlation_id();
        let entry = AuditLogEntry::create(
            school,
            user,
            "student",
            target,
            bytes::Bytes::from_static(b"{\"id\":\"x\"}"),
            corr,
        );
        assert_eq!(entry.action, "create");
        assert!(entry.before.is_none());
        assert!(entry.after.is_some());
    }

    #[test]
    fn entry_serde_round_trip() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let entry = AuditLogEntry::create(
            school,
            g.next_user_id(),
            "student",
            g.next_uuid(),
            bytes::Bytes::from_static(b"{}"),
            g.next_correlation_id(),
        );
        let json = serde_json::to_string(&entry).unwrap();
        let back: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.school_id, back.school_id);
        assert_eq!(entry.action, back.action);
        assert_eq!(entry.target_id, back.target_id);
    }
}
