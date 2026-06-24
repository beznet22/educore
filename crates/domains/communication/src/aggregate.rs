//! # Communication aggregate roots
//!
//! The 26 root aggregates per the spec at
//! `docs/specs/communication/aggregates.md`. Each follows the
//! standard audit-footer pattern (per AGENTS.md):
//!
//! - 1 typed id (e.g. `NoticeId`) + 1 derived `school_id` anchor
//! - domain fields
//! - audit-metadata fields: `version`, `etag`, `created_at`,
//!   `updated_at`, `created_by`, `updated_by`, `active_status`,
//!   `last_event_id`, `correlation_id`
//!
//! `school_id` is **derived from `id.school_id()`**, never taken
//! from the caller.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_academic::{ClassId, SectionId, StudentId, SubjectId};
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{ActiveStatus, Etag, Timestamp, Version};
use educore_hr::value_objects::StaffId;

use crate::entities::{
    CustomSmsSettingParam as EntitiesCustomSmsSettingParam,
    NotificationSettingAudience as EntitiesNotificationSettingAudience,
};
use crate::value_objects::*;

fn fresh_etag() -> Etag {
    Etag::placeholder()
}

/// Flatten an [`AudienceDescriptor`] to a `Vec<UserId>`. Only
/// the [`AudienceDescriptor::Users`] variant carries an explicit
/// user list; the other variants (`Roles`, `ClassSection`, `All`)
/// are reduced to an empty list because the actual user set is
/// resolved lazily by the dispatcher's audience resolver.
fn flatten_audience(audience: &AudienceDescriptor) -> Vec<UserId> {
    match audience {
        AudienceDescriptor::Users(v) => v.clone(),
        AudienceDescriptor::Roles(_)
        | AudienceDescriptor::ClassSection { .. }
        | AudienceDescriptor::All => Vec::new(),
    }
}

// =============================================================================
// Notice
// =============================================================================

/// A notice (announcement) published by a school. The
/// publication state is `Draft → Scheduled → Published` with
/// `Unpublished` as a terminal soft-state. The `publish_on`
/// field is a single-level `Option<NaiveDate>` matching the
/// event payload (no `PublishOn` wrapper) — the wrapper exists
/// in the value-objects module for callers that want a
/// self-documenting constructor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notice {
    /// The typed id.
    pub id: NoticeId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The title (1..=200 chars).
    pub title: NoticeTitle,
    /// The body (1..=5000 chars).
    pub body: NoticeBody,
    /// The notice type.
    pub notice_type: NoticeType,
    /// The notice date.
    pub notice_date: NaiveDate,
    /// The scheduled publish-on date (None = publish immediately).
    pub publish_on: Option<NaiveDate>,
    /// The audience descriptor.
    pub audience: AudienceDescriptor,
    /// The optional attachment.
    pub attachment: Option<FileReference>,
    /// The current status.
    pub status: NoticeStatus,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Notice {
    /// Constructs a new `Notice` in the initial `Draft` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: NoticeId,
        title: NoticeTitle,
        body: NoticeBody,
        notice_date: NaiveDate,
        publish_on: Option<NaiveDate>,
        audience: AudienceDescriptor,
        attachment: Option<FileReference>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            title,
            body,
            notice_type: NoticeType::General,
            notice_date,
            publish_on,
            audience,
            attachment,
            status: NoticeStatus::Draft,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Publishes the notice. Sets status to `Published`.
    pub fn publish(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = NoticeStatus::Published;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Unpublishes the notice. Sets status to `Unpublished`.
    pub fn unpublish(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = NoticeStatus::Unpublished;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the notice. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Updates the mutable notice fields. Returns the list of
    /// field names that were actually changed. `publish_on` is
    /// a single-level `Option<NaiveDate>` to match the event
    /// payload; `None` means "no change", `Some(None)` means
    /// "clear the schedule (publish immediately)", and
    /// `Some(Some(date))` means "publish on this date".
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        title: Option<NoticeTitle>,
        body: Option<NoticeBody>,
        publish_on: Option<Option<NaiveDate>>,
        audience: Option<AudienceDescriptor>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(t) = title {
            self.title = t;
            changes.push("title");
        }
        if let Some(b) = body {
            self.body = b;
            changes.push("body");
        }
        if let Some(po) = publish_on {
            self.publish_on = po;
            changes.push("publish_on");
        }
        if let Some(aud) = audience {
            self.audience = aud;
            changes.push("audience");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }
}

// =============================================================================
// Complaint
// =============================================================================

/// A complaint filed by a parent/staff/anonymous source. The
/// status state machine is `Open → InProgress → Resolved`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Complaint {
    /// The typed id.
    pub id: ComplaintId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The complaint type id.
    pub complaint_type_id: ComplaintTypeId,
    /// The optional complainant user id (None = anonymous).
    pub complaint_by: Option<UserId>,
    /// The source channel.
    pub complaint_source: ComplaintSource,
    /// The optional phone number.
    pub phone: Option<PhoneNumber>,
    /// The date of the complaint.
    pub date: NaiveDate,
    /// The description.
    pub description: ComplaintDescription,
    /// The optional file reference.
    pub file: Option<FileReference>,
    /// The optional assignee.
    pub assignee_user_id: Option<UserId>,
    /// The current status.
    pub status: ComplaintStatus,
    /// The optional action-taken note.
    pub action_taken: Option<String>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Complaint {
    /// Constructs a new `Complaint` in the initial `Open` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ComplaintId,
        complaint_by: Option<UserId>,
        complaint_type_id: ComplaintTypeId,
        complaint_source: ComplaintSource,
        phone: Option<PhoneNumber>,
        date: NaiveDate,
        description: ComplaintDescription,
        file: Option<FileReference>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            complaint_type_id,
            complaint_by,
            complaint_source,
            phone,
            date,
            description,
            file,
            assignee_user_id: None,
            status: ComplaintStatus::Open,
            action_taken: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Assigns the complaint to a user.
    pub fn assign(&mut self, assignee: UserId, actor: UserId, at: Timestamp, event_id: EventId) {
        self.assignee_user_id = Some(assignee);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Updates the complaint status.
    pub fn update_status(
        &mut self,
        new_status: ComplaintStatus,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.status = new_status;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Resolves the complaint. Sets `status = Resolved` and
    /// records the action taken.
    pub fn resolve(
        &mut self,
        action_taken: String,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.status = ComplaintStatus::Resolved;
        self.action_taken = Some(action_taken);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the complaint. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ComplaintType
// =============================================================================

/// A category of complaint (e.g. "Cleanliness", "Safety").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintType {
    /// The typed id.
    pub id: ComplaintTypeId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The category name.
    pub name: String,
    /// The optional description.
    pub description: Option<String>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ComplaintType {
    /// Constructs a new `ComplaintType`.
    pub fn fresh(
        id: ComplaintTypeId,
        name: String,
        description: Option<String>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            description,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names.
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<Option<String>>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(n) = name {
            self.name = n;
            changes.push("name");
        }
        if let Some(d) = description {
            self.description = d;
            changes.push("description");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Soft-deletes the complaint type. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// Notification
// =============================================================================

/// An in-app / push notification dispatched to a user. The
/// status state machine is `Pending → Dispatched → Delivered` with
/// `Read`, `Withdrawn`, or `Failed` as terminal soft-states.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    /// The typed id.
    pub id: NotificationId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The recipient user id.
    pub recipient_user_id: UserId,
    /// The visual severity.
    pub notification_type: NotificationType,
    /// The notification body.
    pub message: NotificationMessage,
    /// The optional URL.
    pub url: Option<Url>,
    /// The structured data payload.
    pub data: BTreeMap<String, String>,
    /// The dispatch channel.
    pub channel: Channel,
    /// The current status.
    pub status: NotificationStatus,
    /// The optional delivered-at timestamp.
    pub delivered_at: Option<Timestamp>,
    /// The optional read-at timestamp.
    pub read_at: Option<Timestamp>,
    /// The optional withdrawn-at timestamp.
    pub withdrawn_at: Option<Timestamp>,
    /// The optional withdrawal reason.
    pub withdrawn_reason: Option<String>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl Notification {
    /// Constructs a new `Notification` in the initial `Pending` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: NotificationId,
        recipient_user_id: UserId,
        notification_type: NotificationType,
        message: NotificationMessage,
        url: Option<Url>,
        data: BTreeMap<String, String>,
        channel: Channel,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            recipient_user_id,
            notification_type,
            message,
            url,
            data,
            channel,
            status: NotificationStatus::Pending,
            delivered_at: None,
            read_at: None,
            withdrawn_at: None,
            withdrawn_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Marks the notification as read.
    pub fn mark_read(&mut self, by: UserId, at: Timestamp, event_id: EventId) {
        self.status = NotificationStatus::Read;
        self.read_at = Some(at);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Withdraws the notification. Records the reason.
    pub fn withdraw(&mut self, reason: String, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = NotificationStatus::Withdrawn;
        self.withdrawn_at = Some(at);
        self.withdrawn_reason = Some(reason);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// EmailLog (APPEND-ONLY)
// =============================================================================

/// An append-only log row recording an outbound email message.
/// No update / delete methods are exposed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailLog {
    /// The typed id.
    pub id: EmailLogId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The log title (free-text).
    pub title: String,
    /// The log description.
    pub description: String,
    /// The send date.
    pub send_date: NaiveDate,
    /// The mail driver used.
    pub send_through: MailDriver,
    /// The recipient address.
    pub send_to: EmailAddress,
    /// The optional source message id.
    pub message_id: Option<MessageId>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl EmailLog {
    /// Constructs a new `EmailLog` row.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: EmailLogId,
        title: String,
        description: String,
        send_date: NaiveDate,
        send_through: MailDriver,
        send_to: EmailAddress,
        message_id: Option<MessageId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            title,
            description,
            send_date,
            send_through,
            send_to,
            message_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// SmsLog (APPEND-ONLY)
// =============================================================================

/// An append-only log row recording an outbound SMS message.
/// No update / delete methods are exposed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsLog {
    /// The typed id.
    pub id: SmsLogId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The log title (free-text).
    pub title: String,
    /// The log description.
    pub description: String,
    /// The send date.
    pub send_date: NaiveDate,
    /// The SMS gateway used.
    pub send_through: SmsGatewayId,
    /// The recipient phone number.
    pub send_to: PhoneNumber,
    /// The optional source message id.
    pub message_id: Option<MessageId>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SmsLog {
    /// Constructs a new `SmsLog` row.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: SmsLogId,
        title: String,
        description: String,
        send_date: NaiveDate,
        send_through: SmsGatewayId,
        send_to: PhoneNumber,
        message_id: Option<MessageId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            title,
            description,
            send_date,
            send_through,
            send_to,
            message_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// SmsTemplate
// =============================================================================

/// An SMS / email template. The `body` is rendered by
/// [`crate::services::TemplateService::render`], which substitutes
/// `{{var}}` placeholders with concrete values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplate {
    /// The typed id.
    pub id: SmsTemplateId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The dispatch channel.
    pub channel: Channel,
    /// The template purpose label.
    pub purpose: String,
    /// The template subject.
    pub subject: EmailSubject,
    /// The template body.
    pub body: TemplateBody,
    /// The module name (e.g. "attendance").
    pub module: String,
    /// The declared template variables.
    pub variables: Vec<TemplateVariable>,
    /// The enabled/disabled state.
    pub status: SmsTemplateStatus,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SmsTemplate {
    /// Constructs a new `SmsTemplate` in the initial `Disabled` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: SmsTemplateId,
        channel: Channel,
        purpose: String,
        subject: EmailSubject,
        body: TemplateBody,
        module: String,
        variables: Vec<TemplateVariable>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            channel,
            purpose,
            subject,
            body,
            module,
            variables,
            status: SmsTemplateStatus::Disabled,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names.
    pub fn update(
        &mut self,
        subject: Option<EmailSubject>,
        body: Option<TemplateBody>,
        variables: Option<Vec<TemplateVariable>>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(s) = subject {
            self.subject = s;
            changes.push("subject");
        }
        if let Some(b) = body {
            self.body = b;
            changes.push("body");
        }
        if let Some(v) = variables {
            self.variables = v;
            changes.push("variables");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Enables the template.
    pub fn enable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = SmsTemplateStatus::Enabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Disables the template.
    pub fn disable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = SmsTemplateStatus::Disabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the template. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// EmailSetting
// =============================================================================

/// An email-server configuration. The `active` flag identifies
/// the currently-active setting per school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSetting {
    /// The typed id.
    pub id: EmailSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The email engine type identifier.
    pub email_engine_type: String,
    /// The "from" name.
    pub from_name: PersonName,
    /// The "from" email address.
    pub from_email: EmailAddress,
    /// The mail driver.
    pub mail_driver: MailDriver,
    /// The mail host.
    pub mail_host: String,
    /// The mail port.
    pub mail_port: u16,
    /// The mail username.
    pub mail_username: String,
    /// The mail password (secret reference).
    pub mail_password: SecretReference,
    /// The mail encryption.
    pub mail_encryption: MailEncryption,
    /// The active flag.
    pub active: bool,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl EmailSetting {
    /// Constructs a new `EmailSetting` (inactive by default).
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: EmailSettingId,
        email_engine_type: String,
        from_name: PersonName,
        from_email: EmailAddress,
        mail_driver: MailDriver,
        mail_host: String,
        mail_port: u16,
        mail_username: String,
        mail_password: SecretReference,
        mail_encryption: MailEncryption,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            email_engine_type,
            from_name,
            from_email,
            mail_driver,
            mail_host,
            mail_port,
            mail_username,
            mail_password,
            mail_encryption,
            active: false,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Activates the email setting. Returns the id of the
    /// previously active setting, if any. The aggregate alone
    /// does not know about other settings, so it returns `None`;
    /// the dispatcher is responsible for looking up the previous
    /// active setting and including it in the event.
    pub fn activate(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Option<EmailSettingId> {
        self.active = true;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        None
    }

    /// Soft-deletes the email setting. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active = false;
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// SmsGateway
// =============================================================================

/// An SMS gateway configuration. The `active` flag identifies
/// the currently-active gateway per school and gateway type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsGateway {
    /// The typed id.
    pub id: SmsGatewayId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The gateway vendor.
    pub gateway_type: GatewayType,
    /// The gateway credentials.
    pub credentials: SmsGatewayCredentials,
    /// The active flag.
    pub active: bool,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SmsGateway {
    /// Constructs a new `SmsGateway` (inactive by default).
    pub fn fresh(
        id: SmsGatewayId,
        gateway_type: GatewayType,
        credentials: SmsGatewayCredentials,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            gateway_type,
            credentials,
            active: false,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Activates the SMS gateway. Returns the id of the
    /// previously active gateway, if any. The aggregate alone
    /// does not know about other gateways, so it returns `None`;
    /// the dispatcher is responsible for looking up the previous
    /// active gateway and including it in the event.
    pub fn activate(
        &mut self,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Option<SmsGatewayId> {
        self.active = true;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        None
    }

    /// Soft-deletes the SMS gateway. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active = false;
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// NotificationSetting
// =============================================================================

/// A notification-setting that maps an event name (e.g.
/// `"student.absent"`) to a destination bitflag, a recipient
/// descriptor, a subject, an SMS template id, and a shortcode
/// (a free-text snippet passed to the template renderer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationSetting {
    /// The typed id.
    pub id: NotificationSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The event name.
    pub event: String,
    /// The destination bitflag.
    pub destination: Destination,
    /// The recipient descriptor.
    pub recipient: EntitiesNotificationSettingAudience,
    /// The subject line.
    pub subject: EmailSubject,
    /// The SMS template id.
    pub template_id: SmsTemplateId,
    /// The shortcode (free-text snippet).
    pub shortcode: String,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl NotificationSetting {
    /// Constructs a new `NotificationSetting`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: NotificationSettingId,
        event: String,
        destination: Destination,
        recipient: EntitiesNotificationSettingAudience,
        subject: EmailSubject,
        template_id: SmsTemplateId,
        shortcode: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            event,
            destination,
            recipient,
            subject,
            template_id,
            shortcode,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names. `event` is not mutable after creation.
    pub fn update(
        &mut self,
        destination: Option<Destination>,
        recipient: Option<EntitiesNotificationSettingAudience>,
        subject: Option<EmailSubject>,
        template_id: Option<SmsTemplateId>,
        shortcode: Option<String>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(d) = destination {
            self.destination = d;
            changes.push("destination");
        }
        if let Some(r) = recipient {
            self.recipient = r;
            changes.push("recipient");
        }
        if let Some(s) = subject {
            self.subject = s;
            changes.push("subject");
        }
        if let Some(t) = template_id {
            self.template_id = t;
            changes.push("template_id");
        }
        if let Some(sc) = shortcode {
            self.shortcode = sc;
            changes.push("shortcode");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Soft-deletes the setting. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// AbsentNotificationTimeSetup
// =============================================================================

/// The schedule for the absent-notification dispatcher. The
/// dispatcher fires for each absent student whose absence is
/// recorded within `[time_from, time_to)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationTimeSetup {
    /// The typed id.
    pub id: AbsentNotificationTimeSetupId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The window start (inclusive).
    pub time_from: TimeOfDay,
    /// The window end (exclusive).
    pub time_to: TimeOfDay,
    /// The enabled/disabled state.
    pub status: AbsentNotificationStatus,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl AbsentNotificationTimeSetup {
    /// Constructs a new `AbsentNotificationTimeSetup` in the
    /// `Disabled` initial state.
    pub fn fresh(
        id: AbsentNotificationTimeSetupId,
        time_from: TimeOfDay,
        time_to: TimeOfDay,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            time_from,
            time_to,
            status: AbsentNotificationStatus::Disabled,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Enables the schedule.
    pub fn enable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = AbsentNotificationStatus::Enabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Disables the schedule.
    pub fn disable(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = AbsentNotificationStatus::Disabled;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the schedule. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatMessage
// =============================================================================

/// A 1-to-1 chat message. The status state machine is
/// `Unread → Seen` with optional soft-delete. The
/// `conversation_id` field is required (per the
/// manifest); orphan messages (no associated conversation)
/// are not a first-class concept.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The typed id.
    pub id: ChatMessageId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The owning conversation id.
    pub conversation_id: ChatConversationId,
    /// The sender user id.
    pub from_id: UserId,
    /// The recipient user id.
    pub to_id: UserId,
    /// The message body.
    pub body: ChatMessageBody,
    /// The message type.
    pub message_type: MessageType,
    /// The optional file reference.
    pub file: Option<FileReference>,
    /// The optional replied-to chat message id.
    pub reply_to: Option<ChatMessageId>,
    /// The optional forwarded-of chat message id.
    pub forward_of: Option<ChatMessageId>,
    /// The current read status.
    pub status: ChatMessageStatus,
    /// The optional seen-at timestamp.
    pub seen_at: Option<Timestamp>,
    /// The optional soft-delete actor.
    pub deleted_by: Option<UserId>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatMessage {
    /// Constructs a new `ChatMessage` in the initial `Unread` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ChatMessageId,
        conversation_id: ChatConversationId,
        from_id: UserId,
        to_id: UserId,
        body: ChatMessageBody,
        message_type: MessageType,
        file: Option<FileReference>,
        reply_to: Option<ChatMessageId>,
        forward_of: Option<ChatMessageId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            conversation_id,
            from_id,
            to_id,
            body,
            message_type,
            file,
            reply_to,
            forward_of,
            status: ChatMessageStatus::Unread,
            seen_at: None,
            deleted_by: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Marks the message as seen.
    pub fn mark_seen(&mut self, by: UserId, at: Timestamp, event_id: EventId) {
        self.status = ChatMessageStatus::Seen;
        self.seen_at = Some(at);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the message. Records the actor.
    pub fn mark_deleted(&mut self, by: UserId, at: Timestamp, event_id: EventId) {
        self.deleted_by = Some(by);
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatConversation
// =============================================================================

/// A 1-to-1 chat conversation between two users. The
/// `created_at` field is the business creation timestamp and
/// also serves as the audit-footer `created_at` (per the
/// per-aggregate 17-field pattern).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConversation {
    /// The typed id.
    pub id: ChatConversationId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The first user.
    pub from_id: UserId,
    /// The second user.
    pub to_id: UserId,
    /// The closed flag.
    pub closed: bool,
    /// The created-at timestamp (business + audit).
    pub created_at: Timestamp,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatConversation {
    /// Constructs a new `ChatConversation` in the open state.
    pub fn fresh(
        id: ChatConversationId,
        from_id: UserId,
        to_id: UserId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            from_id,
            to_id,
            closed: false,
            created_at,
            version: Version::initial(),
            etag: fresh_etag(),
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Closes the conversation.
    pub fn close(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.closed = true;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatGroup
// =============================================================================

/// A chat group (open or closed, public or private). The
/// `created_by` field is the business creator and also serves
/// as the audit-footer `created_by`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroup {
    /// The typed id.
    pub id: ChatGroupId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The group name.
    pub name: String,
    /// The optional description.
    pub description: Option<String>,
    /// The optional photo file reference.
    pub photo: Option<FileReference>,
    /// The group privacy.
    pub privacy: ChatGroupPrivacy,
    /// The group type.
    pub group_type: ChatGroupType,
    /// The optional class id.
    pub class_id: Option<ClassId>,
    /// The optional section id.
    pub section_id: Option<SectionId>,
    /// The optional subject id.
    pub subject_id: Option<SubjectId>,
    /// The optional teacher (staff) id.
    pub teacher_id: Option<StaffId>,
    /// The creator user id (business + audit).
    pub created_by: UserId,
    /// The read-only flag.
    pub read_only: bool,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatGroup {
    /// Constructs a new `ChatGroup`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ChatGroupId,
        name: String,
        description: Option<String>,
        photo: Option<FileReference>,
        privacy: ChatGroupPrivacy,
        group_type: ChatGroupType,
        class_id: Option<ClassId>,
        section_id: Option<SectionId>,
        subject_id: Option<SubjectId>,
        teacher_id: Option<StaffId>,
        _initial_members: Vec<UserId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            description,
            photo,
            privacy,
            group_type,
            class_id,
            section_id,
            subject_id,
            teacher_id,
            created_by,
            read_only: false,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names. `privacy`, `group_type`, `class_id`,
    /// `section_id`, `subject_id`, `teacher_id`, and `created_by`
    /// are immutable after creation.
    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        photo: Option<FileReference>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(n) = name {
            self.name = n;
            changes.push("name");
        }
        if let Some(d) = description {
            self.description = Some(d);
            changes.push("description");
        }
        if let Some(p) = photo {
            self.photo = Some(p);
            changes.push("photo");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Sets the read-only flag.
    pub fn set_read_only(
        &mut self,
        read_only: bool,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.read_only = read_only;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the group. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatGroupUser
// =============================================================================

/// A membership row in a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupUser {
    /// The typed id.
    pub id: ChatGroupUserId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The owning chat group id.
    pub chat_group_id: ChatGroupId,
    /// The user id.
    pub user_id: UserId,
    /// The role.
    pub role: ChatGroupRole,
    /// The adder user id.
    pub added_by: UserId,
    /// The added-at timestamp.
    pub added_at: Timestamp,
    /// The optional remover user id.
    pub removed_by: Option<UserId>,
    /// The optional removed-at timestamp.
    pub deleted_at: Option<Timestamp>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatGroupUser {
    /// Constructs a new `ChatGroupUser` membership row.
    pub fn fresh(
        id: ChatGroupUserId,
        chat_group_id: ChatGroupId,
        user_id: UserId,
        role: ChatGroupRole,
        added_by: UserId,
        added_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_group_id,
            user_id,
            role,
            added_by,
            added_at,
            removed_by: None,
            deleted_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: added_at,
            updated_at: added_at,
            created_by: added_by,
            updated_by: added_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Sets a new role.
    pub fn set_role(
        &mut self,
        new_role: ChatGroupRole,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.role = new_role;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Marks the membership as removed. Records the remover and
    /// the removal timestamp.
    pub fn mark_removed(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.removed_by = Some(actor);
        self.deleted_at = Some(at);
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatGroupMessageRecipient
// =============================================================================

/// A per-recipient fan-out row for a group message. Tracks the
/// read-state of the recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupMessageRecipient {
    /// The typed id.
    pub id: ChatGroupMessageRecipientId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The recipient user id.
    pub user_id: UserId,
    /// The group message id (a `ChatMessageId`).
    pub group_message_id: ChatMessageId,
    /// The optional read-at timestamp.
    pub read_at: Option<Timestamp>,
    /// The optional soft-delete timestamp.
    pub deleted_at: Option<Timestamp>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatGroupMessageRecipient {
    /// Constructs a new `ChatGroupMessageRecipient` row.
    pub fn fresh(
        id: ChatGroupMessageRecipientId,
        chat_group_id: ChatGroupId,
        user_id: UserId,
        group_message_id: ChatMessageId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_group_id,
            user_id,
            group_message_id,
            read_at: None,
            deleted_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Marks the recipient as having read the message.
    pub fn mark_read(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.read_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatGroupMessageRemove (APPEND-ONLY)
// =============================================================================

/// An append-only row recording that a specific group message
/// was removed for a specific user. No update methods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupMessageRemove {
    /// The typed id.
    pub id: ChatGroupMessageRemoveId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The recipient row that was removed.
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    /// The user who was removed from the recipient set.
    pub user_id: UserId,
    /// The removed-at timestamp.
    pub removed_at: Timestamp,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatGroupMessageRemove {
    /// Constructs a new `ChatGroupMessageRemove` row.
    pub fn fresh(
        id: ChatGroupMessageRemoveId,
        chat_group_message_recipient_id: ChatGroupMessageRecipientId,
        user_id: UserId,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_group_message_recipient_id,
            user_id,
            removed_at: created_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// ChatBlockUser
// =============================================================================

/// A user-to-user block row. "Unblock" is modelled as a soft
/// delete (the audit trail of the original block is preserved).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatBlockUser {
    /// The typed id.
    pub id: ChatBlockUserId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The user who placed the block.
    pub block_by: UserId,
    /// The user who was blocked.
    pub block_to: UserId,
    /// The blocked-at timestamp.
    pub blocked_at: Timestamp,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatBlockUser {
    /// Constructs a new `ChatBlockUser` row in the active state.
    pub fn fresh(
        id: ChatBlockUserId,
        block_by: UserId,
        block_to: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            block_by,
            block_to,
            blocked_at: created_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by: block_by,
            updated_by: block_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Soft-deletes the block (unblock). Retires the active
    /// status; the underlying block record is preserved.
    pub fn mark_unblocked(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Returns `true` if the block is still active (the user is
    /// still blocked).
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self.active_status, ActiveStatus::Active)
    }
}

// =============================================================================
// ChatInvitation
// =============================================================================

/// A chat invitation between two users. The status state
/// machine is `Pending → Connected | Blocked`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitation {
    /// The typed id.
    pub id: ChatInvitationId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The sender user id.
    pub from: UserId,
    /// The recipient user id.
    pub to: UserId,
    /// The invitation classification.
    pub invitation_type: ChatInvitationTypeEnum,
    /// The current status.
    pub status: ChatInvitationStatus,
    /// The optional section id (for `ClassTeacher` invites).
    pub section_id: Option<SectionId>,
    /// The optional class-teacher (staff) id.
    pub class_teacher_id: Option<StaffId>,
    /// The sent-at timestamp.
    pub sent_at: Timestamp,
    /// The optional responded-at timestamp.
    pub responded_at: Option<Timestamp>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatInvitation {
    /// Constructs a new `ChatInvitation` in the `Pending` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ChatInvitationId,
        from: UserId,
        to: UserId,
        invitation_type: ChatInvitationTypeEnum,
        section_id: Option<SectionId>,
        class_teacher_id: Option<StaffId>,
        _sent_at_actor: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            from,
            to,
            invitation_type,
            status: ChatInvitationStatus::Pending,
            section_id,
            class_teacher_id,
            sent_at: created_at,
            responded_at: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by: from,
            updated_by: from,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Accepts the invitation. Sets `status = Connected` and
    /// records the response timestamp.
    pub fn accept(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = ChatInvitationStatus::Connected;
        self.responded_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Rejects the invitation. Sets `status = Blocked` and
    /// records the response timestamp.
    pub fn reject(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.status = ChatInvitationStatus::Blocked;
        self.responded_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Soft-deletes the invitation. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatInvitationType
// =============================================================================

/// The classification row attached to a [`ChatInvitation`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitationType {
    /// The typed id.
    pub id: ChatInvitationTypeId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The owning invitation id.
    pub invitation_id: ChatInvitationId,
    /// The classification.
    pub invitation_type: ChatInvitationTypeEnum,
    /// The optional section id.
    pub section_id: Option<SectionId>,
    /// The optional class-teacher (staff) id.
    pub class_teacher_id: Option<StaffId>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatInvitationType {
    /// Constructs a new `ChatInvitationType` row.
    pub fn fresh(
        id: ChatInvitationTypeId,
        invitation_id: ChatInvitationId,
        invitation_type: ChatInvitationTypeEnum,
        section_id: Option<SectionId>,
        class_teacher_id: Option<StaffId>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            invitation_id,
            invitation_type,
            section_id,
            class_teacher_id,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Soft-deletes the row. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ChatStatusRecord (APPEND-ONLY)
// =============================================================================

/// An append-only presence row. The current status is the
/// latest row by `set_at` for the user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatStatusRecord {
    /// The typed id.
    pub id: ChatStatusId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The user id.
    pub user_id: UserId,
    /// The presence state.
    pub status: ChatStatus,
    /// The set-at timestamp.
    pub set_at: Timestamp,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ChatStatusRecord {
    /// Constructs a new `ChatStatusRecord` row.
    pub fn fresh(
        id: ChatStatusId,
        user_id: UserId,
        status: ChatStatus,
        set_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            user_id,
            status,
            set_at,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at: set_at,
            updated_at: set_at,
            created_by: user_id,
            updated_by: user_id,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// SendMessage
// =============================================================================

/// A broadcast message to a list of users. The status state
/// machine is `Draft → Dispatched → Completed` with `Cancelled`
/// as a terminal soft-state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessage {
    /// The typed id.
    pub id: SendMessageId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The message title.
    pub message_title: String,
    /// The message body.
    pub message_body: String,
    /// The notice date.
    pub notice_date: NaiveDate,
    /// The scheduled publish-on date.
    pub publish_on: Option<NaiveDate>,
    /// The recipient descriptor.
    pub message_to: AudienceDescriptor,
    /// The flattened user-id list derived from `message_to`.
    /// This is the list consulted by
    /// [`crate::services::SmsDispatchPolicy`].
    pub audience: Vec<UserId>,
    /// The current status.
    pub status: SendMessageStatus,
    /// The optional recipient count.
    pub recipient_count: Option<u32>,
    /// The optional dispatched-at timestamp.
    pub dispatched_at: Option<Timestamp>,
    /// The optional cancellation reason.
    pub cancelled_reason: Option<String>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SendMessage {
    /// Constructs a new `SendMessage` in the `Draft` state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: SendMessageId,
        message_title: String,
        message_body: String,
        notice_date: NaiveDate,
        publish_on: Option<NaiveDate>,
        message_to: AudienceDescriptor,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        let audience = flatten_audience(&message_to);
        Self {
            school_id: id.school_id(),
            id,
            message_title,
            message_body,
            notice_date,
            publish_on,
            message_to,
            audience,
            status: SendMessageStatus::Draft,
            recipient_count: None,
            dispatched_at: None,
            cancelled_reason: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Dispatches the message. Sets `status = Dispatched`,
    /// records the dispatch timestamp, and computes the
    /// recipient count. Returns the computed recipient count.
    pub fn dispatch(&mut self, actor: UserId, at: Timestamp, event_id: EventId) -> u32 {
        // `self.audience` is bounded by the dispatcher input cap
        // (well below `u32::MAX` in practice); the `TryFrom` form
        // documents the invariant and satisfies the engine lint.
        let count = u32::try_from(self.audience.len()).unwrap_or(u32::MAX);
        self.status = SendMessageStatus::Dispatched;
        self.recipient_count = Some(count);
        self.dispatched_at = Some(at);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        count
    }

    /// Cancels the message. Records the optional reason.
    pub fn cancel(
        &mut self,
        reason: Option<String>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.status = SendMessageStatus::Cancelled;
        self.cancelled_reason = reason;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// ContactMessage
// =============================================================================

/// A contact-form message received from an external party (e.g.
/// a prospective parent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMessage {
    /// The typed id.
    pub id: ContactMessageId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The sender name.
    pub name: PersonName,
    /// The optional phone number.
    pub phone: Option<PhoneNumber>,
    /// The optional email address.
    pub email: Option<EmailAddress>,
    /// The subject.
    pub subject: String,
    /// The message body.
    pub message: String,
    /// The view state.
    pub view_status: ContactMessageViewStatus,
    /// The reply state.
    pub reply_status: ContactMessageReplyStatus,
    /// The optional viewer user id.
    pub viewed_by: Option<UserId>,
    /// The optional replier user id.
    pub replied_by: Option<UserId>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl ContactMessage {
    /// Constructs a new `ContactMessage` in the unviewed,
    /// unreplied state.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: ContactMessageId,
        name: PersonName,
        phone: Option<PhoneNumber>,
        email: Option<EmailAddress>,
        subject: String,
        message: String,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            phone,
            email,
            subject,
            message,
            view_status: ContactMessageViewStatus::Unviewed,
            reply_status: ContactMessageReplyStatus::Unreplied,
            viewed_by: None,
            replied_by: None,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Marks the message as viewed.
    pub fn mark_viewed(&mut self, by: UserId, at: Timestamp, event_id: EventId) {
        self.view_status = ContactMessageViewStatus::Viewed;
        self.viewed_by = Some(by);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }

    /// Records a reply to the message.
    pub fn reply(&mut self, by: UserId, at: Timestamp, event_id: EventId) {
        self.reply_status = ContactMessageReplyStatus::Replied;
        self.replied_by = Some(by);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// SpeechSlider
// =============================================================================

/// A speech-slider entry (a quote, motto, or speech by a school
/// leader that is featured on the public site).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSlider {
    /// The typed id.
    pub id: SpeechSliderId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The person's name.
    pub name: PersonName,
    /// The person's designation.
    pub designation: String,
    /// The speech text.
    pub speech: SpeechText,
    /// The optional image file reference.
    pub image: Option<FileReference>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl SpeechSlider {
    /// Constructs a new `SpeechSlider`.
    pub fn fresh(
        id: SpeechSliderId,
        name: PersonName,
        designation: String,
        speech: SpeechText,
        image: Option<FileReference>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            designation,
            speech,
            image,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names.
    pub fn update(
        &mut self,
        name: Option<PersonName>,
        designation: Option<String>,
        speech: Option<SpeechText>,
        image: Option<FileReference>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(n) = name {
            self.name = n;
            changes.push("name");
        }
        if let Some(d) = designation {
            self.designation = d;
            changes.push("designation");
        }
        if let Some(s) = speech {
            self.speech = s;
            changes.push("speech");
        }
        if let Some(i) = image {
            self.image = Some(i);
            changes.push("image");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Soft-deletes the entry. Retires the active status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// PhoneCallLog
// =============================================================================

/// A logged phone call (incoming, outgoing, or missed). The
/// only update operation is to set the next follow-up date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneCallLog {
    /// The typed id.
    pub id: PhoneCallLogId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The contact name.
    pub name: PersonName,
    /// The contact phone number.
    pub phone: PhoneNumber,
    /// The call date.
    pub date: NaiveDate,
    /// The call description / notes.
    pub description: CallDescription,
    /// The optional next follow-up date.
    pub next_follow_up_date: Option<NaiveDate>,
    /// The optional call duration.
    pub call_duration: Option<CallDuration>,
    /// The call type.
    pub call_type: CallType,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl PhoneCallLog {
    /// Constructs a new `PhoneCallLog`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: PhoneCallLogId,
        name: PersonName,
        phone: PhoneNumber,
        date: NaiveDate,
        description: CallDescription,
        next_follow_up_date: Option<NaiveDate>,
        call_duration: Option<CallDuration>,
        call_type: CallType,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            name,
            phone,
            date,
            description,
            next_follow_up_date,
            call_duration,
            call_type,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the next follow-up date.
    pub fn update_follow_up(
        &mut self,
        next_follow_up: NaiveDate,
        by: UserId,
        at: Timestamp,
        event_id: EventId,
    ) {
        self.next_follow_up_date = Some(next_follow_up);
        self.updated_at = at;
        self.updated_by = by;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// CustomSmsSetting
// =============================================================================

/// A custom-HTTP SMS gateway configuration (the `Custom`
/// variant of [`SmsGatewayCredentials`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomSmsSetting {
    /// The typed id.
    pub id: CustomSmsSettingId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The owning SMS gateway id.
    pub gateway_id: SmsGatewayId,
    /// The gateway name.
    pub gateway_name: GatewayName,
    /// The optional auth secret reference.
    pub set_auth: Option<bool>,
    /// The gateway endpoint URL.
    pub gateway_url: Url,
    /// The HTTP method.
    pub request_method: RequestMethod,
    /// The send-to parameter name.
    pub send_to_parameter_name: String,
    /// The message-to parameter name.
    pub message_to_parameter_name: String,
    /// The request parameters.
    pub params: Vec<EntitiesCustomSmsSettingParam>,
    /// Audit footer (10 fields).
    pub version: Version,
    pub etag: Etag,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub created_by: UserId,
    pub updated_by: UserId,
    pub active_status: ActiveStatus,
    pub last_event_id: Option<EventId>,
    pub correlation_id: CorrelationId,
}

impl CustomSmsSetting {
    /// Constructs a new `CustomSmsSetting`.
    #[allow(clippy::too_many_arguments)]
    pub fn fresh(
        id: CustomSmsSettingId,
        gateway_id: SmsGatewayId,
        gateway_name: GatewayName,
        set_auth: Option<bool>,
        gateway_url: Url,
        request_method: RequestMethod,
        send_to_parameter_name: String,
        message_to_parameter_name: String,
        params: Vec<EntitiesCustomSmsSettingParam>,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            gateway_id,
            gateway_name,
            set_auth,
            gateway_url,
            request_method,
            send_to_parameter_name,
            message_to_parameter_name,
            params,
            version: Version::initial(),
            etag: fresh_etag(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            active_status: ActiveStatus::Active,
            last_event_id: None,
            correlation_id,
        }
    }

    /// Updates the mutable fields. Returns the list of changed
    /// field names. `gateway_id` is immutable after creation.
    pub fn update(
        &mut self,
        gateway_name: Option<GatewayName>,
        set_auth: Option<bool>,
        gateway_url: Option<Url>,
        request_method: Option<RequestMethod>,
        params: Option<Vec<EntitiesCustomSmsSettingParam>>,
        actor: UserId,
        at: Timestamp,
        event_id: EventId,
    ) -> Vec<&'static str> {
        let mut changes: Vec<&'static str> = Vec::new();
        if let Some(g) = gateway_name {
            self.gateway_name = g;
            changes.push("gateway_name");
        }
        if let Some(s) = set_auth {
            self.set_auth = Some(s);
            changes.push("set_auth");
        }
        if let Some(u) = gateway_url {
            self.gateway_url = u;
            changes.push("gateway_url");
        }
        if let Some(m) = request_method {
            self.request_method = m;
            changes.push("request_method");
        }
        if let Some(p) = params {
            self.params = p;
            changes.push("params");
        }
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
        changes
    }

    /// Soft-deletes the custom SMS setting. Retires the active
    /// status.
    pub fn mark_deleted(&mut self, actor: UserId, at: Timestamp, event_id: EventId) {
        self.active_status = ActiveStatus::Retired;
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        self.last_event_id = Some(event_id);
    }
}

// =============================================================================
// Unused-imports suppressor
// =============================================================================

// Some types are part of the public type surface or are
// re-exported via the prelude but aren't directly used by every
// aggregate in this file. The `_unused_imports` helper silences
// the "unused import" lints without changing the module's
// behaviour.
#[allow(dead_code)]
fn _unused_imports(_: StudentId, _: BTreeMap<String, String>) {}
