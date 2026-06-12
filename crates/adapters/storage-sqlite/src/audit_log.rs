//! SQLite-backed `AuditLog` sub-port.
//!
//! Stores each entry as a row in the `audit_log` table. The
//! schema is defined by the canonical .sql migration (loaded
//! by `SqliteStorageAdapter::migrate`).
//!
//! ## Struct <-> schema mapping notes
//!
//! The engine's `AuditLogEntry` struct (the port's API
//! surface) is narrower than the canonical `audit_log` table.
//! Fields not carried by the struct are populated with
//! adapter-level defaults on write and ignored on read:
//!
//! | Schema column    | Source on write                            |
//! |------------------|--------------------------------------------|
//! | `audit_id`       | `uuid::Uuid::now_v7()` (fresh per append)  |
//! | `actor_type`     | `"user"` (literal)                         |
//! | `source`         | `"system"` (literal)                       |
//! | `recorded_at`    | `chrono::Utc::now()` (admission time)      |
//! | `command_id`     | `NULL`                                     |
//! | `ip`             | `NULL`                                     |
//! | `user_agent`     | `NULL`                                     |
//! | `session_id`     | `NULL`                                     |
//! | `cross_tenant`   | `0` (not crossed)                          |

use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::trace;
use uuid::fmt::Hyphenated;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::{CorrelationId, EventId, Identifier as _, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Timestamp};
use educore_storage::audit::{AuditLog, AuditLogEntry};

use crate::error::StringError;
use crate::util::{bytes_to_json, json_to_bytes};

/// The row shape stored in the SQLite `audit_log` table.
#[derive(sqlx::FromRow)]
#[allow(dead_code)] // Some columns (audit_id, recorded_at, ip, …) are not used in `to_entry`; the SELECT is full-row for future expansion.
struct AuditLogRow {
    audit_id: Hyphenated,
    school_id: Hyphenated,
    actor_id: Hyphenated,
    actor_type: String,
    action: String,
    resource_type: String,
    resource_id: Hyphenated,
    event_id: Option<Hyphenated>,
    command_id: Option<Hyphenated>,
    correlation_id: Hyphenated,
    occurred_at: DateTime<Utc>,
    recorded_at: DateTime<Utc>,
    ip: Option<String>,
    user_agent: Option<String>,
    session_id: Option<Hyphenated>,
    before_snapshot: Option<sqlx::types::Json<serde_json::Value>>,
    after_snapshot: Option<sqlx::types::Json<serde_json::Value>>,
    metadata: Option<sqlx::types::Json<serde_json::Value>>,
    cross_tenant: i32,
    source: String,
}

impl AuditLogRow {
    /// Maps a row back to an `AuditLogEntry`. Adapter-level
    /// fields (`actor_type`, `source`, `recorded_at`, …) are
    /// dropped because the entry struct has no slot for them.
    fn to_entry(&self) -> AuditLogEntry {
        AuditLogEntry {
            school_id: SchoolId::from_uuid(*self.school_id.as_uuid()),
            actor_id: UserId::from_uuid(*self.actor_id.as_uuid()),
            action: self.action.clone(),
            target_type: self.resource_type.clone(),
            target_id: *self.resource_id.as_uuid(),
            before: self.before_snapshot.as_ref().map(|j| json_to_bytes(&j.0)),
            after: self.after_snapshot.as_ref().map(|j| json_to_bytes(&j.0)),
            event_id: self.event_id.map(|u| EventId::from_uuid(*u.as_uuid())),
            correlation_id: CorrelationId::from_uuid(*self.correlation_id.as_uuid()),
            occurred_at: Timestamp::from_datetime(self.occurred_at),
            active_status: ActiveStatus::Active,
            metadata: self
                .metadata
                .as_ref()
                .map(|j| j.0.clone())
                .unwrap_or(serde_json::Value::Null),
        }
    }
}

/// The SQLite-backed `AuditLog` implementation.
#[derive(Clone)]
pub struct SqliteAuditLog {
    pool: SqlitePool,
    school: SchoolId,
}

impl fmt::Debug for SqliteAuditLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteAuditLog")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteAuditLog {
    /// Constructs a new audit-log handle bound to `pool` and
    /// scoped to `school`.
    pub fn new(pool: SqlitePool, school: SchoolId) -> Self {
        Self { pool, school }
    }
}

#[async_trait]
impl AuditLog for SqliteAuditLog {
    async fn append(&self, entry: AuditLogEntry) -> Result<()> {
        let audit_id = Uuid::now_v7();
        let before_json: Option<sqlx::types::Json<serde_json::Value>> = entry
            .before
            .as_ref()
            .map(|b| sqlx::types::Json(bytes_to_json(b)));
        let after_json: Option<sqlx::types::Json<serde_json::Value>> = entry
            .after
            .as_ref()
            .map(|b| sqlx::types::Json(bytes_to_json(b)));
        let metadata_json = sqlx::types::Json(entry.metadata.clone());
        let recorded_at = Utc::now();
        sqlx::query::<sqlx::Sqlite>(
            "INSERT INTO audit_log ( \
                audit_id, school_id, actor_id, actor_type, \
                action, resource_type, resource_id, event_id, \
                command_id, correlation_id, occurred_at, \
                recorded_at, ip, user_agent, session_id, \
                before_snapshot, after_snapshot, metadata, \
                cross_tenant, source \
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, NULL, NULL, NULL, ?, ?, ?, 0, ?)",
        )
        .bind(audit_id.hyphenated())
        .bind(entry.school_id.as_uuid().hyphenated())
        .bind(entry.actor_id.as_uuid().hyphenated())
        .bind("user")
        .bind(&entry.action)
        .bind(&entry.target_type)
        .bind(entry.target_id.hyphenated())
        .bind(entry.event_id.map(|e| e.as_uuid().hyphenated()))
        .bind(entry.correlation_id.as_uuid().hyphenated())
        .bind(entry.occurred_at.as_datetime())
        .bind(recorded_at)
        .bind(before_json)
        .bind(after_json)
        .bind(metadata_json)
        .bind("system")
        .execute(&self.pool)
        .await
        .map_err(|e| StringError(format!("audit_log append: {e}")))?;
        trace!(%audit_id, "audit_log append");
        Ok(())
    }

    async fn read_for_target(
        &self,
        school_id: SchoolId,
        target_id: Uuid,
        limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        let rows: Vec<AuditLogRow> = sqlx::query_as::<sqlx::Sqlite, AuditLogRow>(
            "SELECT \
                audit_id, school_id, actor_id, actor_type, \
                action, resource_type, resource_id, event_id, \
                command_id, correlation_id, occurred_at, \
                recorded_at, ip, user_agent, session_id, \
                before_snapshot, after_snapshot, metadata, \
                cross_tenant, source \
             FROM audit_log \
             WHERE school_id = ?1 AND resource_id = ?2 \
             ORDER BY occurred_at ASC \
             LIMIT ?3",
        )
        .bind(school_id.as_uuid().hyphenated())
        .bind(target_id.hyphenated())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StringError(format!("audit_log read_for_target: {e}")))?;
        Ok(rows.iter().map(AuditLogRow::to_entry).collect())
    }
}
