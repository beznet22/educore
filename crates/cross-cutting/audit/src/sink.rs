//! # AuditSink port
//!
//! Per `docs/schemas/audit-schema.md` § 7 the audit crate
//! exposes a `AuditSink` trait that callers use to write audit
//! log entries. The trait is the **port**; the concrete
//! implementation [`crate::writer::AuditWriter`] writes to the
//! storage adapter via the `AuditLog` sub-port.
//!
//! Consumers hold an `Arc<dyn AuditSink>` so the writer
//! implementation can be swapped (e.g. for in-memory tests)
//! without recompiling the consumer. The trait is object-safe.

use async_trait::async_trait;

use educore_core::error::Result;
use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;

use crate::errors::AuditError;
use educore_storage::AuditLogEntry;

/// The audit sink port.
///
/// Object-safe; held as `Arc<dyn AuditSink>` by the command
/// dispatcher and the workflow subscribers.
///
/// Implementations MUST:
/// - Persist every entry to the underlying storage atomically
///   with the originating command's transaction (the writer
///   implementation enforces this via `&dyn Transaction`).
/// - Reject writes whose `school_id` does not match the sink's
///   tenant binding (FND-SEC-AUDIT-001).
/// - Be infallible for writes whose `actor_id` is the system
///   actor (system-originated audit rows are always allowed).
#[async_trait]
pub trait AuditSink: Send + Sync + std::fmt::Debug + 'static {
    /// Writes `entry` to the underlying audit log. Returns
    /// `Ok(())` on success or `Err(_)` if the write fails.
    ///
    /// # Errors
    ///
    /// Implementations MUST return `Err(AuditError::TenantViolation(_))`
    /// if `entry.school_id` does not match the sink's bound
    /// tenant. Other errors are implementation-defined (typically
    /// storage-layer failures mapped to `AuditError::Infrastructure(_)`).
    async fn write(&self, entry: AuditLogEntry) -> Result<()>;

    /// Returns the school_id the sink is bound to. Audit rows
    /// for any other school MUST be rejected (FND-SEC-AUDIT-001).
    fn school_id(&self) -> SchoolId;

    /// Returns the wall-clock timestamp of the most recent
    /// successful write, if any. Used by retention sweepers
    /// to determine the "since" cutoff.
    #[allow(clippy::async_yields_async)]
    async fn last_write_at(&self) -> Result<Option<Timestamp>> {
        // Default implementation returns `None` (write-only
        // sinks don't track this); implementations can override.
        Ok(None)
    }
}

/// Helper to build an `AuditLogEntry` for a typed action.
///
/// Most consumers never need this directly; they call
/// [`AuditSink::write`] with a pre-built entry. The helper is
/// exposed for tests and for adapters that synthesize audit
/// rows outside the command dispatcher.
#[allow(dead_code)]
pub fn entry_for_action(
    school_id: SchoolId,
    actor_id: UserId,
    action: String,
    target_type: String,
    target_id: uuid::Uuid,
) -> AuditLogEntry {
    use educore_core::clock::IdGenerator;
    let g = educore_core::clock::SystemIdGen;
    let mut entry = AuditLogEntry::create(
        school_id,
        actor_id,
        &target_type,
        target_id,
        bytes::Bytes::new(),
        g.next_correlation_id(),
    );
    entry.action = action;
    entry
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
    use educore_core::clock::IdGenerator;

    #[test]
    fn trait_object_safe() {
        // Compile-time check: a Box<dyn AuditSink> is valid
        // (the trait is object-safe).
        fn _accepts_sink(_: Box<dyn AuditSink>) {}
    }

    #[test]
    fn entry_for_action_builds_entry() {
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let target = uuid::Uuid::now_v7();
        let entry = entry_for_action(
            school,
            actor,
            "test.action".to_owned(),
            "test_target".to_owned(),
            target,
        );
        assert_eq!(entry.school_id, school);
        assert_eq!(entry.actor_id, actor);
        assert_eq!(entry.target_id, target);
    }
}
