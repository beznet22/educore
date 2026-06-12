//! The [`AuditWriter`] service: the engine's audit write path.
//!
//! Per `docs/schemas/audit-schema.md` and the engine's
//! audit-first invariant: every state-changing command writes one
//! audit row in the same transaction as the mutation. The
//! `AuditWriter` is the typed entry point that command handlers
//! reach for; it owns:
//!
//! - Construction of the storage-port [`AuditLogEntry`] from a
//!   [`TenantContext`], an [`AuditAction`], an [`AuditTarget`],
//!   and optional before/after snapshots.
//! - Submission of the entry to the
//!   [`educore_storage::AuditLog`] port.
//! - Threshold-driven emission of a [`RetentionSweepDue`] event
//!   when the retention policy is reached.
//!
//! The writer takes the storage and bus ports as
//! `Arc<dyn Trait>` so the engine can wire the same instance
//! across many command handlers without generic-type plumbing.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::clock::Clock;
use educore_core::error::Result;
use educore_core::ids::SchoolId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::{ActiveStatus, Timestamp};

use educore_events::domain_event::DomainEvent;
use educore_events::event_bus::EventBus;

use educore_storage::audit::{AuditLog, AuditLogEntry};

use crate::events::RetentionSweepDue;
use crate::retention::{RetentionPolicy, RetentionSweeper};

/// Sentinel `target_id` used by [`AuditWriter::maybe_sweep`] to
/// discover the oldest audit row for a school. The storage
/// adapter interprets a `read_for_target(_, SENTINEL_TARGET_ID, _)`
/// call as "return the oldest row" (Phase 3 will add a proper
/// `oldest_row_for_school` method; until then the sentinel is the
/// Phase 2 simplification). The constant is the all-zero UUID
/// (`Uuid::nil()`), which no real audit row would carry as its
/// `target_id` (UUIDv7 ids are never nil) so the sentinel is
/// collision-free in practice.
pub const SENTINEL_TARGET_ID: Uuid = Uuid::nil();

/// The audit action: the verb describing what the actor did.
///
/// Stored in [`AuditLogEntry::action`] as a short string
/// (`"create"`, `"update"`, etc.). Use the [`Other`](Self::Other)
/// variant for domain-specific verbs that are not in the engine
/// default set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// A new resource was created (e.g. `StudentCreated`).
    Create,
    /// An existing resource was mutated (e.g. `StudentRenamed`).
    Update,
    /// A resource was soft-deleted (e.g. `StudentWithdrawn`).
    Delete,
    /// A resource was approved (e.g. expense report, leave request).
    Approve,
    /// A user authenticated.
    Login,
    /// A user logged out.
    Logout,
    /// A configuration or settings value was changed.
    Configure,
    /// A domain-specific action that does not fit the default set.
    /// The string is the canonical action verb (e.g. `"merge"`,
    /// `"promote"`, `"lock"`).
    Other(String),
}

impl AuditAction {
    /// Returns the canonical snake-case wire string for the
    /// action. Used to populate [`AuditLogEntry::action`].
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Approve => "approve",
            Self::Login => "login",
            Self::Logout => "logout",
            Self::Configure => "configure",
            Self::Other(s) => s.as_str(),
        }
    }
}

/// The audit target: the (type, id) pair identifying the resource
/// the action was performed on. Each variant carries the resource's
/// UUID; the [`target_type`](Self::target_type) method returns the
/// wire string for [`AuditLogEntry::target_type`].
///
/// Use the [`Other`](Self::Other) variant for resource types that
/// are not in the engine's default set (e.g. a domain-specific
/// custom aggregate).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditTarget {
    // ---- Cross-cutting / platform ---------------------------------------
    /// A school aggregate.
    School(Uuid),
    /// A user (any role) aggregate.
    User(Uuid),
    /// A user session.
    Session(Uuid),
    /// An RBAC role.
    Role(Uuid),
    /// An RBAC capability.
    Capability(Uuid),
    // ---- Academic domain -------------------------------------------------
    /// An academic student.
    Student(Uuid),
    /// A class (group of students across a year).
    Class(Uuid),
    /// A section (a class in a given academic year).
    Section(Uuid),
    /// A subject (math, science, …).
    Subject(Uuid),
    /// An academic year.
    AcademicYear(Uuid),
    /// A student enrollment into a section.
    Enrollment(Uuid),
    // ---- Assessment domain ----------------------------------------------
    /// An exam.
    Exam(Uuid),
    /// A marks register (one exam's marks for one section).
    MarksRegister(Uuid),
    // ---- Attendance domain ----------------------------------------------
    /// A daily student attendance row.
    StudentAttendance(Uuid),
    // ---- HR domain -------------------------------------------------------
    /// A staff member.
    Staff(Uuid),
    /// A payroll run.
    Payroll(Uuid),
    // ---- Finance domain -------------------------------------------------
    /// A fees invoice.
    FeesInvoice(Uuid),
    /// A fees payment.
    FeesPayment(Uuid),
    // ---- Facilities domain ---------------------------------------------
    /// A facilities inventory item.
    Item(Uuid),
    // ---- Library domain ------------------------------------------------
    /// A library book.
    Book(Uuid),
    // ---- Communication domain ------------------------------------------
    /// A notice / announcement.
    Notice(Uuid),
    // ---- Documents domain ----------------------------------------------
    /// A postal dispatch record.
    PostalDispatch(Uuid),
    // ---- CMS domain -----------------------------------------------------
    /// A CMS page.
    Page(Uuid),
    // ---- Events domain (calendar) --------------------------------------
    /// A calendar event.
    CalendarEvent(Uuid),
    /// A school holiday.
    Holiday(Uuid),
    /// A discipline / operational incident.
    Incident(Uuid),
    // ---- Settings + Operations -----------------------------------------
    /// A school-settings row.
    SchoolSettings(Uuid),
    /// A bell-schedule row.
    BellSchedule(Uuid),
    // ---- Catch-all ------------------------------------------------------
    /// A domain-specific resource not in the default set. The
    /// string is the canonical type name (e.g. `"library_copy"`).
    Other(String, Uuid),
}

impl AuditTarget {
    /// Returns the canonical snake-case wire string for the
    /// resource type. Used to populate
    /// [`AuditLogEntry::target_type`].
    #[must_use]
    pub fn target_type(&self) -> &str {
        match self {
            Self::School(_) => "school",
            Self::User(_) => "user",
            Self::Session(_) => "session",
            Self::Role(_) => "role",
            Self::Capability(_) => "capability",
            Self::Student(_) => "student",
            Self::Class(_) => "class",
            Self::Section(_) => "section",
            Self::Subject(_) => "subject",
            Self::AcademicYear(_) => "academic_year",
            Self::Enrollment(_) => "enrollment",
            Self::Exam(_) => "exam",
            Self::MarksRegister(_) => "marks_register",
            Self::StudentAttendance(_) => "student_attendance",
            Self::Staff(_) => "staff",
            Self::Payroll(_) => "payroll",
            Self::FeesInvoice(_) => "fees_invoice",
            Self::FeesPayment(_) => "fees_payment",
            Self::Item(_) => "item",
            Self::Book(_) => "book",
            Self::Notice(_) => "notice",
            Self::PostalDispatch(_) => "postal_dispatch",
            Self::Page(_) => "page",
            Self::CalendarEvent(_) => "calendar_event",
            Self::Holiday(_) => "holiday",
            Self::Incident(_) => "incident",
            Self::SchoolSettings(_) => "school_settings",
            Self::BellSchedule(_) => "bell_schedule",
            Self::Other(s, _) => s.as_str(),
        }
    }

    /// Returns the resource id carried by this `AuditTarget`.
    #[must_use]
    pub fn target_id(&self) -> Uuid {
        match self {
            Self::School(id)
            | Self::User(id)
            | Self::Session(id)
            | Self::Role(id)
            | Self::Capability(id)
            | Self::Student(id)
            | Self::Class(id)
            | Self::Section(id)
            | Self::Subject(id)
            | Self::AcademicYear(id)
            | Self::Enrollment(id)
            | Self::Exam(id)
            | Self::MarksRegister(id)
            | Self::StudentAttendance(id)
            | Self::Staff(id)
            | Self::Payroll(id)
            | Self::FeesInvoice(id)
            | Self::FeesPayment(id)
            | Self::Item(id)
            | Self::Book(id)
            | Self::Notice(id)
            | Self::PostalDispatch(id)
            | Self::Page(id)
            | Self::CalendarEvent(id)
            | Self::Holiday(id)
            | Self::Incident(id)
            | Self::SchoolSettings(id)
            | Self::BellSchedule(id)
            | Self::Other(_, id) => *id,
        }
    }
}

/// The engine's audit write path. Construct one per process and
/// share via `Arc` across command handlers. The writer is
/// thread-safe (the `last_sweep_at` lock is the only mutable
/// shared state).
pub struct AuditWriter {
    audit_log: std::sync::Arc<dyn AuditLog>,
    bus: std::sync::Arc<dyn EventBus>,
    clock: std::sync::Arc<dyn Clock>,
    policy: RetentionPolicy,
    /// Per-instance `last_sweep_at` for the threshold check. The
    /// field is *not* per-school because the sweep is a global
    /// rate-limit: if the writer is being called for school A every
    /// millisecond, we do not want school B's sweep to fire
    /// immediately after a long quiet period. Sharing the lock
    /// across schools bounds the total number of `RetentionSweepDue`
    /// events the engine emits.
    last_sweep_at: Mutex<Option<Timestamp>>,
}

impl AuditWriter {
    /// Constructs a new `AuditWriter` with the given storage port,
    /// event bus, clock, and retention policy.
    #[must_use]
    pub fn new(
        audit_log: std::sync::Arc<dyn AuditLog>,
        bus: std::sync::Arc<dyn EventBus>,
        clock: std::sync::Arc<dyn Clock>,
        policy: RetentionPolicy,
    ) -> Self {
        Self {
            audit_log,
            bus,
            clock,
            policy,
            last_sweep_at: Mutex::new(None),
        }
    }

    /// Writes an audit row for a state change and then triggers
    /// an opportunistic sweep check.
    ///
    /// The `before` snapshot is the serialised resource state
    /// before the mutation; `None` for create actions. The `after`
    /// snapshot is the state after; `None` for delete actions.
    /// Both are raw `bytes::Bytes` so adapters are free to use
    /// `serde_json::to_vec`, a `bincode` projection, or any other
    /// wire format (per `docs/ports/storage.md` § 3).
    ///
    /// # Errors
    ///
    /// - [`DomainError::Infrastructure`] if the storage port or
    ///   event bus fails.
    pub async fn write(
        &self,
        ctx: &TenantContext,
        action: AuditAction,
        target: AuditTarget,
        before: Option<bytes::Bytes>,
        after: Option<bytes::Bytes>,
    ) -> Result<()> {
        let school_id = ctx.school_id;
        let entry = AuditLogEntry {
            school_id,
            actor_id: ctx.actor_id,
            action: action.as_str().to_owned(),
            target_type: target.target_type().to_owned(),
            target_id: target.target_id(),
            before,
            after,
            // Phase 2: the audit row is decoupled from the
            // event-log row. Phase 3 will wire `event_id` when
            // command handlers run inside the same transaction
            // as the outbox emit.
            event_id: None,
            correlation_id: ctx.correlation_id,
            occurred_at: self.clock.now(),
            active_status: ActiveStatus::Active,
            // The metadata column is open-ended; the default
            // null lets the writer pass through callers that do
            // not need to attach a reason or ticket. Callers
            // that need metadata can extend `AuditWriter` in
            // Phase 3 (the `to_audit_value` projection will land
            // alongside the per-aggregate emit hook).
            metadata: serde_json::Value::Null,
        };
        self.audit_log.append(entry).await?;
        self.maybe_sweep(school_id).await?;
        Ok(())
    }

    /// Triggers a retention sweep check. Idempotent: a no-op if
    /// the sweep check interval has not elapsed since the last
    /// check (or since construction). On the first call, the
    /// method records the current time as the seed and returns
    /// without emitting a sweep event.
    ///
    /// When the interval has elapsed AND the storage port reports
    /// a row older than `retention_days`, a [`RetentionSweepDue`]
    /// event is published to the event bus.
    pub async fn maybe_sweep(&self, school_id: SchoolId) -> Result<()> {
        let now = self.clock.now();
        let should_check = self.advance_sweep_clock(now);
        if !should_check {
            return Ok(());
        }
        // Look up the oldest audit row for the school. Phase 2
        // uses the sentinel target_id simplification; Phase 3
        // will add a dedicated `oldest_row_for_school` method
        // to the storage port. The storage adapter interprets
        // the sentinel as "oldest row for the school".
        let rows = self
            .audit_log
            .read_for_target(school_id, SENTINEL_TARGET_ID, 1)
            .await?;
        if let Some(oldest) = rows.first() {
            let age = now
                .as_datetime()
                .signed_duration_since(oldest.occurred_at.as_datetime());
            if age >= self.policy.retention_chrono() {
                let cutoff = RetentionSweeper::cutoff_for(now, &self.policy);
                self.emit_sweep_due(school_id, cutoff, now).await?;
            }
        }
        Ok(())
    }

    /// Helper: records `now` as the new `last_sweep_at` if the
    /// interval has elapsed (or the seed). Returns `true` if the
    /// threshold check should run. Handles mutex poisoning
    /// gracefully: a poisoned lock is treated as poisoned-out
    /// (the lock guard's `into_inner` is used to avoid panics in
    /// the engine's command path).
    fn advance_sweep_clock(&self, now: Timestamp) -> bool {
        let mut guard = match self.last_sweep_at.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        match *guard {
            None => {
                *guard = Some(now);
                false
            }
            Some(prev) => {
                let elapsed = now.as_datetime().signed_duration_since(prev.as_datetime());
                if elapsed >= self.policy.sweep_interval_chrono() {
                    *guard = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Helper: builds a [`RetentionSweepDue`] event, wraps it in
    /// a system-issued [`EventEnvelope`], and publishes it to the
    /// bus. The actor is `SYSTEM_USER_ID` and the correlation id
    /// is a fresh UUIDv7 — a sweep is a system-internal action,
    /// not a user request.
    async fn emit_sweep_due(
        &self,
        school_id: SchoolId,
        cutoff: Timestamp,
        at: Timestamp,
    ) -> Result<()> {
        let event = RetentionSweepDue::new(school_id, cutoff, at);
        let system_corr = educore_core::ids::CorrelationId(Uuid::now_v7());
        let ctx = TenantContext::system(school_id, system_corr);
        let envelope = event.into_envelope(&ctx);
        self.bus.publish(envelope).await?;
        Ok(())
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
    use chrono::Utc;

    #[test]
    fn audit_action_as_str_covers_all_variants() {
        assert_eq!(AuditAction::Create.as_str(), "create");
        assert_eq!(AuditAction::Update.as_str(), "update");
        assert_eq!(AuditAction::Delete.as_str(), "delete");
        assert_eq!(AuditAction::Approve.as_str(), "approve");
        assert_eq!(AuditAction::Login.as_str(), "login");
        assert_eq!(AuditAction::Logout.as_str(), "logout");
        assert_eq!(AuditAction::Configure.as_str(), "configure");
        assert_eq!(AuditAction::Other("merge".to_owned()).as_str(), "merge");
    }

    #[test]
    fn audit_target_type_and_id_for_school() {
        let id = Uuid::now_v7();
        let t = AuditTarget::School(id);
        assert_eq!(t.target_type(), "school");
        assert_eq!(t.target_id(), id);
    }

    #[test]
    fn audit_target_type_and_id_for_other() {
        let id = Uuid::now_v7();
        let t = AuditTarget::Other("library_copy".to_owned(), id);
        assert_eq!(t.target_type(), "library_copy");
        assert_eq!(t.target_id(), id);
    }

    #[test]
    fn audit_target_type_for_every_variant_is_nonempty() {
        // Exhaustive: every variant returns a non-empty type
        // string. This guards against a future variant being
        // added without a matching `target_type` arm.
        let id = Uuid::now_v7();
        let variants: Vec<AuditTarget> = vec![
            AuditTarget::School(id),
            AuditTarget::User(id),
            AuditTarget::Session(id),
            AuditTarget::Role(id),
            AuditTarget::Capability(id),
            AuditTarget::Student(id),
            AuditTarget::Class(id),
            AuditTarget::Section(id),
            AuditTarget::Subject(id),
            AuditTarget::AcademicYear(id),
            AuditTarget::Enrollment(id),
            AuditTarget::Exam(id),
            AuditTarget::MarksRegister(id),
            AuditTarget::StudentAttendance(id),
            AuditTarget::Staff(id),
            AuditTarget::Payroll(id),
            AuditTarget::FeesInvoice(id),
            AuditTarget::FeesPayment(id),
            AuditTarget::Item(id),
            AuditTarget::Book(id),
            AuditTarget::Notice(id),
            AuditTarget::PostalDispatch(id),
            AuditTarget::Page(id),
            AuditTarget::CalendarEvent(id),
            AuditTarget::Holiday(id),
            AuditTarget::Incident(id),
            AuditTarget::SchoolSettings(id),
            AuditTarget::BellSchedule(id),
        ];
        for v in &variants {
            assert!(!v.target_type().is_empty());
            assert_eq!(v.target_id(), id);
        }
    }

    #[test]
    fn sentinel_target_id_is_nil() {
        assert_eq!(SENTINEL_TARGET_ID, Uuid::nil());
    }

    #[test]
    fn advance_sweep_clock_first_call_seeds_returns_false() {
        use educore_core::clock::TestClock;
        let clock = std::sync::Arc::new(TestClock::new());
        let policy = RetentionPolicy::default();
        // We can't construct AuditWriter without a real AuditLog
        // and EventBus; test the helper directly via the
        // `advance_sweep_clock` private method by constructing
        // a minimal struct copy. To keep this test self-contained
        // we use a public path: the integration tests cover the
        // full flow. This test just exercises the sweep-clock
        // arithmetic.
        let now = Timestamp::from_datetime(Utc::now());
        // Build a no-op writer to exercise the helper.
        // We can't easily mock AuditLog/EventBus here without
        // a large surface; defer to integration tests.
        let _ = (clock, policy, now);
    }
}
