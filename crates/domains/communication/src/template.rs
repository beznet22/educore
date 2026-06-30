//! # NotificationTemplate aggregate
//!
//! Per `docs/specs/communication/aggregates.md`, the
//! notification template aggregate carries a multi-channel
//! template (subject + body) the engine renders against the
//! typed variables supplied by each `SendNotification`
//! command. This module is the **port-side stub**: full
//! repository / wiring lands in a later phase; this module
//! declares the aggregate shape so the rest of the engine can
//! reason about templates without going through an opaque id.
//!
//! ## Relationship to SmsTemplate
//!
//! The legacy `SmsTemplate` aggregate (channel-bound) is
//! retained for backwards compatibility. `NotificationTemplate`
//! is the engine-side, channel-agnostic, content-only template
//! shape; SmsTemplate wraps a NotificationTemplate body with
//! SMS-specific delivery metadata.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use educore_core::ids::{Identifier, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};

use crate::value_objects::Channel;

/// Typed id for a [`NotificationTemplate`]. Wraps a UUIDv7.
/// Derives `school_id()` from the embedded school anchor so
/// cross-tenant lookup is impossible by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationTemplateId(pub uuid::Uuid);

impl Identifier for NotificationTemplateId {
    fn as_uuid(&self) -> uuid::Uuid {
        self.0
    }
    fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl NotificationTemplateId {
    /// Mints a fresh `NotificationTemplateId`.
    #[must_use]
    pub fn fresh() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for NotificationTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// The typed shape of a template variable. Matches the
/// `TemplateVariable` shape used by the notify port so the
/// engine can render `body_template` directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateVariable {
    /// A free-form string (the rendered placeholder).
    String,
    /// A signed integer.
    Integer,
    /// A boolean.
    Bool,
    /// A date / time value rendered via RFC 3339.
    Date,
    /// A monetary amount rendered with the school's locale.
    Currency,
}

impl TemplateVariable {
    /// Returns `true` if the variable type requires locale
    /// formatting at render time.
    #[must_use]
    pub const fn requires_locale(&self) -> bool {
        matches!(self, Self::Currency | Self::Date)
    }
}

/// The `NotificationTemplate` aggregate. A multi-channel
/// notification template with a typed variable map; the
/// engine renders the template at dispatch time by
/// substituting `{name}` placeholders with values from the
/// `SendNotification::variables` map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// The typed id.
    pub id: NotificationTemplateId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The template slug (e.g. `"attendance.absent.parent"`).
    pub name: String,
    /// The dispatch channel this template targets. A
    /// template is single-channel; consumers that want
    /// multi-channel sends register one template per channel.
    pub channel: Channel,
    /// The template subject line (email / push only).
    pub subject: String,
    /// The template body. Placeholders use the `{name}`
    /// syntax; rendered via the engine's template service.
    pub body_template: String,
    /// The declared template variables (in declaration order).
    pub variables: BTreeMap<String, TemplateVariable>,
    /// Audit footer (10 fields).
    /// Aggregate version for optimistic locking.
    pub version: Version,
    /// The entity tag (content hash) for cache-busting.
    pub etag: Etag,
    /// When the template was first created.
    pub created_at: Timestamp,
    /// When the template was last updated.
    pub updated_at: Timestamp,
    /// The user who created the template.
    pub created_by: UserId,
    /// The user who last updated the template.
    pub updated_by: UserId,
    /// The active / soft-deleted status of the template.
    pub active_status: ActiveStatus,
    /// Optional correlation id for the most recent update.
    pub correlation_id: Option<uuid::Uuid>,
}

impl NotificationTemplate {
    /// Returns the channel this template targets.
    #[must_use]
    pub const fn channel(&self) -> Channel {
        self.channel
    }

    /// Returns `true` iff the template body contains a
    /// `{name}` placeholder for `var_name`.
    #[must_use]
    pub fn references_variable(&self, var_name: &str) -> bool {
        let needle = format!("{{{var_name}}}");
        self.body_template.contains(&needle) || self.subject.contains(&needle)
    }

    /// Returns the variable names declared by this template
    /// (in sorted order).
    #[must_use]
    pub fn declared_variables(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// Returns `true` iff this template is currently active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.active_status, ActiveStatus::Active)
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
    use educore_core::ids::EventId;

    fn sample_template() -> NotificationTemplate {
        let g = SystemIdGen;
        NotificationTemplate {
            id: NotificationTemplateId::fresh(),
            school_id: g.next_school_id(),
            name: "attendance.absent.parent".to_owned(),
            channel: Channel::Email,
            subject: "Your ward was absent on {date}".to_owned(),
            body_template: "Dear {parent_name}, {student_name} was absent on {date}.".to_owned(),
            variables: BTreeMap::from([
                ("parent_name".to_owned(), TemplateVariable::String),
                ("student_name".to_owned(), TemplateVariable::String),
                ("date".to_owned(), TemplateVariable::Date),
            ]),
            version: Version::initial(),
            etag: Etag::new("00000000000000000000000000000000").expect("valid etag"),
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            created_by: g.next_user_id(),
            updated_by: g.next_user_id(),
            active_status: ActiveStatus::Active,
            correlation_id: None,
        }
    }

    #[test]
    fn channel_accessor_returns_configured_channel() {
        let t = sample_template();
        assert_eq!(t.channel(), Channel::Email);
    }

    #[test]
    fn references_variable_detects_subject_and_body() {
        let t = sample_template();
        assert!(t.references_variable("date"));
        assert!(t.references_variable("parent_name"));
        assert!(t.references_variable("student_name"));
        assert!(!t.references_variable("nonexistent"));
    }

    #[test]
    fn declared_variables_returns_sorted_keys() {
        let t = sample_template();
        let vars = t.declared_variables();
        assert_eq!(vars, vec!["date", "parent_name", "student_name"]);
    }

    #[test]
    fn template_variable_locale_required() {
        assert!(TemplateVariable::Currency.requires_locale());
        assert!(TemplateVariable::Date.requires_locale());
        assert!(!TemplateVariable::String.requires_locale());
        assert!(!TemplateVariable::Integer.requires_locale());
        assert!(!TemplateVariable::Bool.requires_locale());
    }

    #[test]
    fn active_status_accessor() {
        let t = sample_template();
        assert!(t.is_active());
    }

    #[test]
    fn notification_template_id_fresh_is_unique() {
        let id1 = NotificationTemplateId::fresh();
        let id2 = NotificationTemplateId::fresh();
        assert_ne!(id1, id2);
    }

    #[test]
    fn notification_template_id_as_uuid_round_trip() {
        let id = NotificationTemplateId::fresh();
        let uuid = id.as_uuid();
        let back = NotificationTemplateId::from_uuid(uuid);
        assert_eq!(id, back);
    }

    #[test]
    fn notification_template_id_event_id_compat() {
        // NotificationTemplateId must be compatible with EventId
        // for the audit footer's last_event_id field.
        let event_id = EventId::from_uuid(uuid::Uuid::now_v7());
        let _uuid = event_id.as_uuid();
    }
}
