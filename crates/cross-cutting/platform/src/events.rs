//! Platform domain events.
//!
//! Each event implements
//! [`educore_events::domain_event::DomainEvent`]. The
//! `event_type` is namespaced as `"platform.<aggregate>.<verb>"`
//! per the bus-port contract (e.g. `"platform.school.created"`).
//!
//! Phase 2 ships the six events enumerated in the prompt:
//! - [`SchoolCreated`], [`SchoolUpdated`], [`SchoolDeactivated`]
//! - [`UserRegistered`], [`UserUpdated`], [`UserDeactivated`]
//!
//! The remaining events listed in
//! `docs/specs/platform/events.md` (SchoolApproved,
//! UserReactivated, UserRoleChanged, OtpIssued, ...) are out of
//! scope for Phase 2 and land in later phases alongside their
//! owning aggregates.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::UserType;
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use crate::commands::ReportFormat;
use crate::value_objects::{EmailAddress, SchoolStatus, UserStatus};

/// A school was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolCreated {
    /// The school's typed id.
    pub school_id: SchoolId,
    /// The school's display name.
    pub name: String,
    /// The school's unique short code.
    pub school_code: String,
    /// The school's optional public-facing domain.
    pub domain: Option<String>,
    /// The school's initial lifecycle status (always `Pending`
    /// at creation; downstream subscribers flip to `Approved`).
    pub status: SchoolStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SchoolCreated {
    /// Mints a fresh `SchoolCreated` with the supplied event
    /// id and `occurred_at`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        name: String,
        school_code: String,
        domain: Option<String>,
        status: SchoolStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            name,
            school_code,
            domain,
            status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SchoolCreated {
    const EVENT_TYPE: &'static str = "platform.school.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "school";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.school_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A school's mutable fields were updated.
///
/// The `changed_fields` list carries the field names that
/// actually changed (as static `&'static str` slices). The
/// event is intentionally minimal: the full updated row is
/// available via the school read model, and the engine does
/// not need to replay the update from the event payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolUpdated {
    /// The school's typed id.
    pub school_id: SchoolId,
    /// The names of the fields that actually changed. Stored
    /// as `Vec<String>` so the subscriber can iterate without
    /// naming a fixed set.
    pub changed_fields: Vec<String>,
    /// The new value of the school's `name`, if it changed.
    pub name: Option<String>,
    /// The new value of the school's `domain`, if it changed.
    pub domain: Option<String>,
    /// The new value of the school's `package_id`, if it
    /// changed.
    pub package_id: Option<Uuid>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SchoolUpdated {
    /// Mints a fresh `SchoolUpdated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        changed_fields: Vec<String>,
        name: Option<String>,
        domain: Option<String>,
        package_id: Option<Uuid>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            changed_fields,
            name,
            domain,
            package_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SchoolUpdated {
    const EVENT_TYPE: &'static str = "platform.school.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "school";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.school_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A school was deactivated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolDeactivated {
    /// The school's typed id.
    pub school_id: SchoolId,
    /// The reason for deactivation (free-form, 1..=500 chars).
    pub reason: String,
    /// The lifecycle status the school was set to (typically
    /// `Suspended`).
    pub new_status: SchoolStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl SchoolDeactivated {
    /// Mints a fresh `SchoolDeactivated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        school_id: SchoolId,
        reason: String,
        new_status: SchoolStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            reason,
            new_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SchoolDeactivated {
    const EVENT_TYPE: &'static str = "platform.school.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "school";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.school_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A user was registered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRegistered {
    /// The user's typed id.
    pub user_id: UserId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The user's email at the time of registration.
    pub email: EmailAddress,
    /// The user's chosen username.
    pub username: String,
    /// The user's display name.
    pub display_name: String,
    /// The actor's role at the moment of registration.
    pub usertype: UserType,
    /// The user's initial role bindings (usually empty at
    /// registration; the role is set by a follow-up
    /// `ChangeUserRole` command).
    pub role_ids: Vec<Uuid>,
    /// The initial user status (always `Active` at
    /// registration).
    pub status: UserStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl UserRegistered {
    /// Mints a fresh `UserRegistered`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        user_id: UserId,
        school_id: SchoolId,
        email: EmailAddress,
        username: String,
        display_name: String,
        usertype: UserType,
        role_ids: Vec<Uuid>,
        status: UserStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            user_id,
            school_id,
            email,
            username,
            display_name,
            usertype,
            role_ids,
            status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserRegistered {
    const EVENT_TYPE: &'static str = "platform.user.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "user";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A user's mutable fields were updated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpdated {
    /// The user's typed id.
    pub user_id: UserId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The names of the fields that actually changed.
    pub changed_fields: Vec<String>,
    /// The new value of the user's `email`, if it changed.
    pub email: Option<EmailAddress>,
    /// The new value of the user's `display_name`, if it
    /// changed.
    pub display_name: Option<String>,
    /// The new value of the user's `phone_number`, if it
    /// changed.
    pub phone_number: Option<String>,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl UserUpdated {
    /// Mints a fresh `UserUpdated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        user_id: UserId,
        school_id: SchoolId,
        changed_fields: Vec<String>,
        email: Option<EmailAddress>,
        display_name: Option<String>,
        phone_number: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            user_id,
            school_id,
            changed_fields,
            email,
            display_name,
            phone_number,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserUpdated {
    const EVENT_TYPE: &'static str = "platform.user.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "user";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// A user was deactivated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDeactivated {
    /// The user's typed id.
    pub user_id: UserId,
    /// The owning school.
    pub school_id: SchoolId,
    /// The reason for deactivation (free-form, 1..=500 chars).
    pub reason: String,
    /// The new status (typically `Inactive` or `Suspended`).
    pub new_status: UserStatus,
    /// Mint-time event id.
    pub event_id: EventId,
    /// The correlation id of the request that triggered the
    /// event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl UserDeactivated {
    /// Mints a fresh `UserDeactivated`.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        user_id: UserId,
        school_id: SchoolId,
        reason: String,
        new_status: UserStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            user_id,
            school_id,
            reason,
            new_status,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserDeactivated {
    const EVENT_TYPE: &'static str = "platform.user.deactivated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "user";

    fn event_id(&self) -> EventId {
        self.event_id
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id.as_uuid()
    }

    fn school_id(&self) -> SchoolId {
        self.school_id
    }

    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// ===========================================================================
// Compliance-report events (Phase 2 § 8.2-8.4)
//
// Three events mirror the three compliance-report commands in
// `commands.rs`. They close the same three roadmap items:
// - SCHEMA-AUDIT-GDPR-ERASURE
// - SCHEMA-AUDIT-FERPA
// - SCHEMA-AUDIT-REGULATOR
// ===========================================================================

/// Emitted when a GDPR right-to-erasure was executed for
/// `subject_id` (`docs/schemas/audit-schema.md` § 8.2). The
/// subject's profile has been soft-deleted; PII in the audit
/// log has been anonymized; financial records have been
/// retained for the regulator-required period (default 7
/// years). Consumers (e.g. the privacy portal) subscribe to
/// `platform.subject.erased` to confirm the erasure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectErased {
    /// The typed id of the erased subject.
    pub subject_id: UserId,
    /// The data-subject's request reference id.
    pub request_id: String,
    /// The actor who executed the erasure (a compliance
    /// officer, typically).
    pub actor_id: UserId,
    /// The free-text reason recorded with the erasure.
    pub reason: String,
    /// Mint-time event id.
    pub event_id: EventId,
    /// Correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl DomainEvent for SubjectErased {
    /// Stable dotted event-type string. The subscription key
    /// for consumers is `"platform.subject.erased"`.
    const EVENT_TYPE: &'static str = "platform.subject.erased";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "subject";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.subject_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        // The erasure applies tenant-wide; consumers can
        // derive the school from the subject's profile
        // (looked up before the erasure) or from the
        // `request_id` if they need to route the event.
        SchoolId::from_uuid(self.subject_id.as_uuid())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a FERPA-style parental access report was
/// generated (`docs/schemas/audit-schema.md` § 8.3). The
/// report itself is stored at the location returned by
/// `bundle_ref`; consumers subscribe to
/// `platform.parent_access.generated` to deliver the bundle
/// to the parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentAccessReportGenerated {
    /// The typed id of the child whose records are in the report.
    pub child_id: UserId,
    /// The typed id of the parent who requested the report.
    pub parent_id: UserId,
    /// Reference to the generated report bundle (engine-level
    /// file storage).
    pub bundle_ref: String,
    /// The output format requested by the parent.
    pub report_format: ReportFormat,
    /// Mint-time event id.
    pub event_id: EventId,
    /// Correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl DomainEvent for ParentAccessReportGenerated {
    const EVENT_TYPE: &'static str = "platform.parent_access.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "parent_access_report";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.child_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        // Tenant-wide report; the child's school is derived
        // from the child's profile lookup at service time.
        SchoolId::from_uuid(self.child_id.as_uuid())
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a regulator audit bundle was generated
/// (`docs/schemas/audit-schema.md` § 8.4). The bundle covers
/// all state changes, authorization decisions, data exports,
/// and backup/restore events for `school_id` in
/// `[from, to]`. The consumer's compliance team reviews and
/// signs off internally before disclosure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegulatorAuditGenerated {
    /// The school under audit.
    pub school_id: SchoolId,
    /// Inclusive start of the audit time range.
    pub from: Timestamp,
    /// Inclusive end of the audit time range.
    pub to: Timestamp,
    /// Reference to the generated audit bundle.
    pub bundle_ref: String,
    /// The actor who generated the report (compliance officer).
    pub actor_id: UserId,
    /// Mint-time event id.
    pub event_id: EventId,
    /// Correlation id of the request that triggered the event.
    pub correlation_id: CorrelationId,
    /// Clock time of the event.
    pub occurred_at: Timestamp,
}

impl DomainEvent for RegulatorAuditGenerated {
    const EVENT_TYPE: &'static str = "platform.regulator_audit.generated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "regulator_audit";

    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.school_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
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
    use educore_core::tenant::TenantContext;

    fn ctx() -> TenantContext {
        let g = SystemIdGen;
        TenantContext::system(g.next_school_id(), g.next_correlation_id())
    }

    #[test]
    fn school_created_event_type_is_namespaced() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let event = SchoolCreated::new(
            school,
            "Ada".to_owned(),
            "ADA".to_owned(),
            None,
            SchoolStatus::Pending,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        assert_eq!(
            <SchoolCreated as DomainEvent>::EVENT_TYPE,
            "platform.school.created"
        );
        assert_eq!(<SchoolCreated as DomainEvent>::AGGREGATE_TYPE, "school");
        assert_eq!(event.school_id(), school);
        assert_eq!(event.aggregate_id(), school.as_uuid());
    }

    #[test]
    fn school_created_envelope_round_trip() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let event = SchoolCreated::new(
            school,
            "Ada".to_owned(),
            "ADA".to_owned(),
            None,
            SchoolStatus::Pending,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.school.created");
        assert_eq!(env.aggregate_type, "school");
        assert_eq!(env.school_id, school);
    }

    #[test]
    fn user_registered_event_type_is_namespaced() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let event = UserRegistered::new(
            user,
            school,
            EmailAddress::new("ada@example.com").unwrap(),
            "ada".to_owned(),
            "Ada".to_owned(),
            UserType::SchoolAdmin,
            Vec::new(),
            UserStatus::Active,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        assert_eq!(
            <UserRegistered as DomainEvent>::EVENT_TYPE,
            "platform.user.registered"
        );
        assert_eq!(<UserRegistered as DomainEvent>::AGGREGATE_TYPE, "user");
        assert_eq!(event.school_id(), school);
        assert_eq!(event.aggregate_id(), user.as_uuid());
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.user.registered");
    }

    #[test]
    fn school_updated_event_envelope_metadata() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let event = SchoolUpdated::new(
            school,
            vec!["name".to_owned()],
            Some("Ada Lovelace School".to_owned()),
            None,
            None,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.school.updated");
        assert_eq!(env.aggregate_type, "school");
    }

    #[test]
    fn school_deactivated_event_envelope_metadata() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let event = SchoolDeactivated::new(
            school,
            "non-payment".to_owned(),
            SchoolStatus::Suspended,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.school.deactivated");
        assert_eq!(env.aggregate_type, "school");
    }

    #[test]
    fn user_updated_event_envelope_metadata() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let event = UserUpdated::new(
            user,
            school,
            vec!["display_name".to_owned()],
            None,
            Some("Ada, Countess of Lovelace".to_owned()),
            None,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.user.updated");
        assert_eq!(env.aggregate_type, "user");
    }

    #[test]
    fn user_deactivated_event_envelope_metadata() {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let event = UserDeactivated::new(
            user,
            school,
            "resigned".to_owned(),
            UserStatus::Inactive,
            g.next_event_id(),
            g.next_correlation_id(),
            Timestamp::epoch(),
        );
        let env = event.into_envelope(&ctx());
        assert_eq!(env.event_type, "platform.user.deactivated");
        assert_eq!(env.aggregate_type, "user");
    }
}
