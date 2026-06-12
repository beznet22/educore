//! SurrealDB-backed sub-port stubs.
//!
//! Phase 0 implements the `Outbox` sub-port (see
//! `super::outbox`); the `AuditLog`, `EventLog`, and
//! `Idempotency` sub-ports are stubbed to return
//! `DomainError::NotSupported` until the e2e suite exercises
//! them. The stubs keep the trait surface complete so the
//! engine's command dispatcher can hold
//! `Box<dyn AuditLog>` etc. without an in-memory
//! implementation.

use async_trait::async_trait;
use uuid::Uuid;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;
use educore_storage::audit::{AuditLog, AuditLogEntry};
use educore_storage::event_log::{EventLog, EventLogEntry, EventLogFilter};
use educore_storage::idempotency::{Idempotency, IdempotencyCompositeKey, IdempotencyRecord};

use crate::connection::Db;

/// `AuditLog` stub. Returns `NotSupported` for both
/// operations; the Phase 0 e2e suite does not exercise audit.
#[derive(Clone)]
pub struct SurrealAuditLog {
    #[allow(dead_code)]
    pub(crate) db: Db,
    #[allow(dead_code)]
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealAuditLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealAuditLog").finish_non_exhaustive()
    }
}

#[async_trait]
impl AuditLog for SurrealAuditLog {
    async fn append(&self, _entry: AuditLogEntry) -> Result<()> {
        Err(DomainError::not_supported(
            "SurrealAuditLog::append is not yet implemented (Phase 0 stub)",
        ))
    }
    async fn read_for_target(
        &self,
        _school_id: SchoolId,
        _target_id: Uuid,
        _limit: u32,
    ) -> Result<Vec<AuditLogEntry>> {
        Err(DomainError::not_supported(
            "SurrealAuditLog::read_for_target is not yet implemented (Phase 0 stub)",
        ))
    }
}

/// `EventLog` stub. Returns `NotSupported`; the relay-side
/// `event_log` write path lands in a later phase.
#[derive(Clone)]
pub struct SurrealEventLog {
    #[allow(dead_code)]
    pub(crate) db: Db,
    #[allow(dead_code)]
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealEventLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealEventLog").finish_non_exhaustive()
    }
}

#[async_trait]
impl EventLog for SurrealEventLog {
    async fn append(&self, _entry: EventLogEntry) -> Result<()> {
        Err(DomainError::not_supported(
            "SurrealEventLog::append is not yet implemented (Phase 0 stub)",
        ))
    }
    async fn read(&self, _filter: EventLogFilter) -> Result<Vec<EventLogEntry>> {
        Err(DomainError::not_supported(
            "SurrealEventLog::read is not yet implemented (Phase 0 stub)",
        ))
    }
    async fn count(&self, _filter: EventLogFilter) -> Result<u64> {
        Err(DomainError::not_supported(
            "SurrealEventLog::count is not yet implemented (Phase 0 stub)",
        ))
    }
}

/// `Idempotency` stub. Returns `NotSupported`; idempotency
/// is a future-phase concern.
#[derive(Clone)]
pub struct SurrealIdempotency {
    #[allow(dead_code)]
    pub(crate) db: Db,
    #[allow(dead_code)]
    pub(crate) school: SchoolId,
}

impl std::fmt::Debug for SurrealIdempotency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealIdempotency").finish_non_exhaustive()
    }
}

#[async_trait]
impl Idempotency for SurrealIdempotency {
    async fn lookup(&self, _key: IdempotencyCompositeKey) -> Result<Option<IdempotencyRecord>> {
        Err(DomainError::not_supported(
            "SurrealIdempotency::lookup is not yet implemented (Phase 0 stub)",
        ))
    }
    async fn record(&self, _record: IdempotencyRecord) -> Result<()> {
        Err(DomainError::not_supported(
            "SurrealIdempotency::record is not yet implemented (Phase 0 stub)",
        ))
    }
}
