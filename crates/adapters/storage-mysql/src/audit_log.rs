//! MySQL-backed `AuditLog` sub-port.
//!
//! Stores each `AuditLogEntry` as a row in the `audit_log`
//! table. The schema is defined by the canonical
//! `migrations/engine/0000_engine_core.mysql.sql` migration
//! loaded by `MysqlStorageAdapter::migrate`.
//!
//! ## Column mapping
//!
//! The engine's `AuditLogEntry` carries a subset of the columns
//! the DDL exposes. On write, the adapter derives the missing
//! fields from sensible defaults:
//!
//! - `audit_id` — a fresh UUIDv7 generated at write time
//! - `actor_type` — `"system"` if the actor is
//!   [`educore_core::ids::SYSTEM_USER_ID`], otherwise `"user"`
//! - `recorded_at` — copies `occurred_at` (the audit row is
//!   written transactionally with the state change, so the two
//!   are the same in this design)
//! - `ip`, `user_agent`, `session_id`, `command_id` — `NULL`
//! - `cross_tenant` — `FALSE` (the engine records the
//!   `TenantViolation` separately in the outbox and reserves
//!   this column for cross-tenant operations that *succeed*)
//! - `source` — defaults to `"api"`
//!
//! The DDL does not carry an `active_status` column; the audit
//! log is append-only. The adapter maps `entry.active_status`
//! to the engine's `Active` value on read (the column is
//! present in the DDL for forward compatibility with a future
//! retire operation).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{FromRow, MySqlPool};
use tracing::instrument;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{
    CorrelationId, EventId, Identifier as _, SchoolId, UserId, SYSTEM_USER_ID,
};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::audit::{AuditLog, AuditLogEntry};

use crate::connection_helpers::{opt_bytes_to_json_value, opt_json_to_opt_bytes};

/// The row shape read out of the `audit_log` table. Field
/// types mirror the MySQL column types per
/// `docs/schemas/sql-dialects/mysql.md` § "Type mapping".
#[derive(Debug, FromRow)]
struct AuditLogRow {
    #[allow(dead_code)]
    audit_id: Uuid,
    school_id: Uuid,
    actor_id: Uuid,
    #[allow(dead_code)]
    actor_type: String,
    action: String,
    resource_type: String,
    resource_id: Uuid,
    event_id: Option<Uuid>,
    #[allow(dead_code)]
    command_id: Option<Uuid>,
    correlation_id: Uuid,
    occurred_at: DateTime<Utc>,
    #[allow(dead_code)]
    recorded_at: DateTime<Utc>,
    #[allow(dead_code)]
    ip: Option<String>,
    #[allow(dead_code)]
    user_agent: Option<String>,
    #[allow(dead_code)]
    session_id: Option<Uuid>,
    before_snapshot: Option<Json<Value>>,
    after_snapshot: Option<Json<Value>>,
    metadata: Json<Value>,
    #[allow(dead_code)]
    cross_tenant: bool,
    #[allow(dead_code)]
    source: String,
}

impl AuditLogRow {
    /// Maps a database row back to an `AuditLogEntry`. The DDL
    /// is a superset of the entry; the extra columns
    /// (`audit_id`, `actor_type`, `command_id`, `ip`, etc.) are
    /// not carried in the entry and are dropped.
    fn into_entry(self) -> AuditLogEntry {
        AuditLogEntry {
            school_id: SchoolId::from_uuid(self.school_id),
            actor_id: UserId::from_uuid(self.actor_id),
            action: self.action,
            target_type: self.resource_type,
            target_id: self.resource_id,
            before: opt_json_to_opt_bytes(&self.before_snapshot),
            after: opt_json_to_opt_bytes(&self.after_snapshot),
            event_id: self.event_id.map(EventId::from_uuid),
            correlation_id: CorrelationId::from_uuid(self.correlation_id),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            // The DDL does not carry an `active_status`
            // column. The audit log is append-only; we set
            // `Active` on read and rely on the engine's
            // `INCLUDE_RETIRED` query to surface retired
            // rows in the future.
            active_status: ActiveStatus::Active,
            metadata: self.metadata.0,
        }
    }
}

/// The MySQL-backed `AuditLog` implementation.
#[derive(Clone)]
pub struct MysqlAuditLog {
    pool: MySqlPool,
    #[allow(dead_code)]
    school: SchoolId,
}

impl std::fmt::Debug for MysqlAuditLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MysqlAuditLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl MysqlAuditLog {
    /// Constructs a new audit-log handle bound to `pool` and
    /// scoped to `school`. The `school` field is reserved for
    /// future per-connection filtering (the trait's
    /// `read_for_target` takes a `school_id` argument; the
    /// handle's school is used for symmetry with the other
    /// sub-ports).
    #[must_use]
    pub fn new(pool: MySqlPool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl AuditLog for MysqlAuditLog {
    #[instrument(skip(self, entry), fields(actor = %entry.actor_id, target_type = %entry.target_type))]
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        let audit_id = Uuid::now_v7();
        let actor_type: &'static str = if entry.actor_id == SYSTEM_USER_ID {
            "system"
        } else {
            "user"
        };
        // The audit row is written transactionally with the
        // state change, so `recorded_at` mirrors `occurred_at`.
        let recorded_at: DateTime<Utc> = entry.occurred_at.as_datetime();
        sqlx::query::<sqlx::MySql>(
            "INSERT INTO `audit_log` (\
                `audit_id`, `school_id`, `actor_id`, `actor_type`, `action`, \
                `resource_type`, `resource_id`, `event_id`, `command_id`, \
                `correlation_id`, `occurred_at`, `recorded_at`, `ip`, \
                `user_agent`, `session_id`, `before_snapshot`, \
                `after_snapshot`, `metadata`, `cross_tenant`, `source`\
            ) VALUES (\
                ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, NULL, NULL, NULL, \
                ?, ?, ?, FALSE, 'api'\
            )",
        )
        .bind(audit_id)
        .bind(entry.school_id.as_uuid())
        .bind(entry.actor_id.as_uuid())
        .bind(actor_type)
        .bind(&entry.action)
        .bind(&entry.target_type)
        .bind(entry.target_id)
        .bind(entry.event_id.map(|e| e.as_uuid()))
        .bind(entry.correlation_id.as_uuid())
        .bind(entry.occurred_at.as_datetime())
        .bind(recorded_at)
        .bind(opt_bytes_to_json_value(&entry.before))
        .bind(opt_bytes_to_json_value(&entry.after))
        .bind(Json(&entry.metadata))
        .execute(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let rows: Vec<AuditLogRow> = sqlx::query_as::<sqlx::MySql, AuditLogRow>(
            "SELECT \
                `audit_id`, `school_id`, `actor_id`, `actor_type`, `action`, \
                `resource_type`, `resource_id`, `event_id`, `command_id`, \
                `correlation_id`, `occurred_at`, `recorded_at`, `ip`, \
                `user_agent`, `session_id`, `before_snapshot`, \
                `after_snapshot`, `metadata`, `cross_tenant`, `source` \
             FROM `audit_log` \
             WHERE `school_id` = ? AND `resource_id` = ? \
             ORDER BY `occurred_at` ASC \
             LIMIT ?",
        )
        .bind(school_id.as_uuid())
        .bind(target_id)
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(educore_core::error::DomainError::infrastructure)?;
        let entries = rows.into_iter().map(AuditLogRow::into_entry).collect();
        Ok(entries)
    }
}
