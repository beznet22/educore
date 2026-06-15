//! # Communication domain events
//!
//! Every aggregate's state change emits an event implementing
//! [`DomainEvent`](::educore_events::domain_event::DomainEvent).
//! The full set follows the spec at
//! `docs/specs/communication/events.md`.
//!
//! Wire form: `communication.<aggregate>.<verb>` (e.g.
//! `communication.notice.created`,
//! `communication.chat_group_message_recipient.recorded`).
//!
//! Phase 10 ships the 73 typed events enumerated in the
//! `educore-communication` manifest (section 5), covering the 26
//! aggregate roots.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;

use educore_academic::StudentId;

use crate::value_objects::{
    AbsentNotificationTimeSetupId, AudienceDescriptor, CallType, Channel,
    ChatConversationId, ChatGroupId, ChatGroupMessageRecipientId,
    ChatGroupMessageRemoveId, ChatGroupPrivacy, ChatGroupRole, ChatGroupType,
    ChatInvitationId, ChatInvitationStatus, ChatInvitationTypeEnum, ChatInvitationTypeId,
    ChatMessageId, ChatMessageStatus, ChatStatus, ComplaintId, ComplaintSource, ComplaintStatus,
    ComplaintTypeId, ContactMessageId, CustomSmsSettingId, Destination, EmailAddress, EmailLogId,
    EmailSettingId, EmailSubject, GatewayType, MailDriver, MessageId, MessageType, NoticeDate,
    NoticeId, NoticeTitle, NotificationId, NotificationSettingId, NotificationType, PersonName,
    PhoneCallLogId, PhoneNumber, RequestMethod, SendMessageId, SmsGatewayId, SmsLogId,
    SmsTemplateId, SpeechSliderId, TemplateKey, TimeOfDay, Url,
};

/// Stable UUID namespace used to derive `aggregate_id()` for the
/// small set of events whose underlying state has no typed id
/// ([`UserBlocked`], [`UserUnblocked`], [`ChatStatusSet`]). The
/// `UserId` itself is a global, non-school-scoped identifier, so
/// the namespace lets us produce a deterministic v5 UUID that is
/// stable across replays and unique to the communication crate.
///
/// Never reuse this namespace outside this module; it is version-
/// pinned to the educore-communication Phase 10 event layout.
const AGGREGATE_ID_NAMESPACE: Uuid = Uuid::from_u128(0x6f_d1_a1_4b_2c_7e_4b_2c_a9_8e_3f_4a_5b_6c);

// =============================================================================
// Notice events (5)
// =============================================================================

/// Emitted when a new `Notice` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeCreated {
    pub notice_id: NoticeId,
    pub title: NoticeTitle,
    pub notice_date: NaiveDate,
    pub publish_on: Option<NaiveDate>,
    pub audience: AudienceDescriptor,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NoticeCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        notice_id: NoticeId,
        title: NoticeTitle,
        notice_date: NaiveDate,
        publish_on: Option<NaiveDate>,
        audience: AudienceDescriptor,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notice_id,
            title,
            notice_date,
            publish_on,
            audience,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NoticeCreated {
    const EVENT_TYPE: &'static str = "communication.notice.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notice` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeUpdated {
    pub notice_id: NoticeId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NoticeUpdated {
    pub fn new(
        notice_id: NoticeId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notice_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NoticeUpdated {
    const EVENT_TYPE: &'static str = "communication.notice.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notice` is published.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticePublished {
    pub notice_id: NoticeId,
    pub published_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NoticePublished {
    pub fn new(
        notice_id: NoticeId,
        published_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notice_id,
            published_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NoticePublished {
    const EVENT_TYPE: &'static str = "communication.notice.published";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notice` is unpublished.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeUnpublished {
    pub notice_id: NoticeId,
    pub reason: Option<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NoticeUnpublished {
    pub fn new(
        notice_id: NoticeId,
        reason: Option<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notice_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NoticeUnpublished {
    const EVENT_TYPE: &'static str = "communication.notice.unpublished";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notice` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeDeleted {
    pub notice_id: NoticeId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NoticeDeleted {
    pub fn new(
        notice_id: NoticeId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notice_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NoticeDeleted {
    const EVENT_TYPE: &'static str = "communication.notice.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notice";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notice_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notice_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Complaint events (8)
// =============================================================================

/// Emitted when a new `Complaint` is registered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintRegistered {
    pub complaint_id: ComplaintId,
    pub complaint_type_id: ComplaintTypeId,
    pub complaint_source: ComplaintSource,
    pub date: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintRegistered {
    pub fn new(
        complaint_id: ComplaintId,
        complaint_type_id: ComplaintTypeId,
        complaint_source: ComplaintSource,
        date: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_id,
            complaint_type_id,
            complaint_source,
            date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintRegistered {
    const EVENT_TYPE: &'static str = "communication.complaint.registered";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Complaint` is assigned to a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintAssigned {
    pub complaint_id: ComplaintId,
    pub assignee_user_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintAssigned {
    pub fn new(
        complaint_id: ComplaintId,
        assignee_user_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_id,
            assignee_user_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintAssigned {
    const EVENT_TYPE: &'static str = "communication.complaint.assigned";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the status of a `Complaint` changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintStatusChanged {
    pub complaint_id: ComplaintId,
    pub from: ComplaintStatus,
    pub to: ComplaintStatus,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintStatusChanged {
    pub fn new(
        complaint_id: ComplaintId,
        from: ComplaintStatus,
        to: ComplaintStatus,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_id,
            from,
            to,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintStatusChanged {
    const EVENT_TYPE: &'static str = "communication.complaint.status_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Complaint` is resolved.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintResolved {
    pub complaint_id: ComplaintId,
    pub action_taken: String,
    pub resolved_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintResolved {
    pub fn new(
        complaint_id: ComplaintId,
        action_taken: String,
        resolved_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_id,
            action_taken,
            resolved_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintResolved {
    const EVENT_TYPE: &'static str = "communication.complaint.resolved";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a note is added to a `Complaint`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintNoteAdded {
    pub complaint_id: ComplaintId,
    pub note: String,
    pub author: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintNoteAdded {
    pub fn new(
        complaint_id: ComplaintId,
        note: String,
        author: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_id,
            note,
            author,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintNoteAdded {
    const EVENT_TYPE: &'static str = "communication.complaint.note_added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a new `ComplaintType` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintTypeCreated {
    pub complaint_type_id: ComplaintTypeId,
    pub name: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintTypeCreated {
    pub fn new(
        complaint_type_id: ComplaintTypeId,
        name: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_type_id,
            name,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintTypeCreated {
    const EVENT_TYPE: &'static str = "communication.complaint_type.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ComplaintType` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintTypeUpdated {
    pub complaint_type_id: ComplaintTypeId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintTypeUpdated {
    pub fn new(
        complaint_type_id: ComplaintTypeId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_type_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintTypeUpdated {
    const EVENT_TYPE: &'static str = "communication.complaint_type.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ComplaintType` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintTypeDeleted {
    pub complaint_type_id: ComplaintTypeId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ComplaintTypeDeleted {
    pub fn new(
        complaint_type_id: ComplaintTypeId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            complaint_type_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ComplaintTypeDeleted {
    const EVENT_TYPE: &'static str = "communication.complaint_type.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "complaint_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.complaint_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.complaint_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Notification events (3)
// =============================================================================

/// Emitted when a `Notification` is sent to a recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationSent {
    pub notification_id: NotificationId,
    pub recipient_user_id: UserId,
    pub notification_type: NotificationType,
    pub channel: Channel,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationSent {
    pub fn new(
        notification_id: NotificationId,
        recipient_user_id: UserId,
        notification_type: NotificationType,
        channel: Channel,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_id,
            recipient_user_id,
            notification_type,
            channel,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationSent {
    const EVENT_TYPE: &'static str = "communication.notification.sent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notification` is read by its recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationRead {
    pub notification_id: NotificationId,
    pub read_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationRead {
    pub fn new(
        notification_id: NotificationId,
        read_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_id,
            read_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationRead {
    const EVENT_TYPE: &'static str = "communication.notification.read";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `Notification` is withdrawn before the recipient
/// reads it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationWithdrawn {
    pub notification_id: NotificationId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationWithdrawn {
    pub fn new(
        notification_id: NotificationId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationWithdrawn {
    const EVENT_TYPE: &'static str = "communication.notification.withdrawn";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// EmailLog event (1)
// =============================================================================

/// Emitted when an outbound email is recorded in the audit log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailLogged {
    pub email_log_id: EmailLogId,
    pub title: String,
    pub send_through: MailDriver,
    pub send_to: EmailAddress,
    pub send_date: NaiveDate,
    pub source_message_id: Option<MessageId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EmailLogged {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        email_log_id: EmailLogId,
        title: String,
        send_through: MailDriver,
        send_to: EmailAddress,
        send_date: NaiveDate,
        source_message_id: Option<MessageId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            email_log_id,
            title,
            send_through,
            send_to,
            send_date,
            source_message_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for EmailLogged {
    const EVENT_TYPE: &'static str = "communication.email_log.logged";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "email_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.email_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.email_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SmsLog event (1)
// =============================================================================

/// Emitted when an outbound SMS is recorded in the audit log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsLogged {
    pub sms_log_id: SmsLogId,
    pub title: String,
    pub send_through: SmsGatewayId,
    pub send_to: PhoneNumber,
    pub send_date: NaiveDate,
    pub source_message_id: Option<MessageId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsLogged {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sms_log_id: SmsLogId,
        title: String,
        send_through: SmsGatewayId,
        send_to: PhoneNumber,
        send_date: NaiveDate,
        source_message_id: Option<MessageId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_log_id,
            title,
            send_through,
            send_to,
            send_date,
            source_message_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsLogged {
    const EVENT_TYPE: &'static str = "communication.sms_log.logged";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SmsTemplate events (5)
// =============================================================================

/// Emitted when a new `SmsTemplate` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateCreated {
    pub sms_template_id: SmsTemplateId,
    pub channel: Channel,
    pub purpose: TemplateKey,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsTemplateCreated {
    pub fn new(
        sms_template_id: SmsTemplateId,
        channel: Channel,
        purpose: TemplateKey,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_template_id,
            channel,
            purpose,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsTemplateCreated {
    const EVENT_TYPE: &'static str = "communication.sms_template.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SmsTemplate` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateUpdated {
    pub sms_template_id: SmsTemplateId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsTemplateUpdated {
    pub fn new(
        sms_template_id: SmsTemplateId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_template_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsTemplateUpdated {
    const EVENT_TYPE: &'static str = "communication.sms_template.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SmsTemplate` is enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateEnabled {
    pub sms_template_id: SmsTemplateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsTemplateEnabled {
    pub fn new(
        sms_template_id: SmsTemplateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_template_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsTemplateEnabled {
    const EVENT_TYPE: &'static str = "communication.sms_template.enabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SmsTemplate` is disabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateDisabled {
    pub sms_template_id: SmsTemplateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsTemplateDisabled {
    pub fn new(
        sms_template_id: SmsTemplateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_template_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsTemplateDisabled {
    const EVENT_TYPE: &'static str = "communication.sms_template.disabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SmsTemplate` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateDeleted {
    pub sms_template_id: SmsTemplateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsTemplateDeleted {
    pub fn new(
        sms_template_id: SmsTemplateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_template_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsTemplateDeleted {
    const EVENT_TYPE: &'static str = "communication.sms_template.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_template";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_template_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_template_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// EmailSetting events (3)
// =============================================================================

/// Emitted when an `EmailSetting` is configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSettingConfigured {
    pub email_setting_id: EmailSettingId,
    pub mail_driver: MailDriver,
    pub mail_host: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EmailSettingConfigured {
    pub fn new(
        email_setting_id: EmailSettingId,
        mail_driver: MailDriver,
        mail_host: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            email_setting_id,
            mail_driver,
            mail_host,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for EmailSettingConfigured {
    const EVENT_TYPE: &'static str = "communication.email_setting.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "email_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.email_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.email_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `EmailSetting` is activated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSettingActivated {
    pub email_setting_id: EmailSettingId,
    pub previous_id: Option<EmailSettingId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EmailSettingActivated {
    pub fn new(
        email_setting_id: EmailSettingId,
        previous_id: Option<EmailSettingId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            email_setting_id,
            previous_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for EmailSettingActivated {
    const EVENT_TYPE: &'static str = "communication.email_setting.activated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "email_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.email_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.email_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `EmailSetting` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSettingDeleted {
    pub email_setting_id: EmailSettingId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl EmailSettingDeleted {
    pub fn new(
        email_setting_id: EmailSettingId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            email_setting_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for EmailSettingDeleted {
    const EVENT_TYPE: &'static str = "communication.email_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "email_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.email_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.email_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SmsGateway events (3)
// =============================================================================

/// Emitted when an `SmsGateway` is configured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsGatewayConfigured {
    pub sms_gateway_id: SmsGatewayId,
    pub gateway_type: GatewayType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsGatewayConfigured {
    pub fn new(
        sms_gateway_id: SmsGatewayId,
        gateway_type: GatewayType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_gateway_id,
            gateway_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsGatewayConfigured {
    const EVENT_TYPE: &'static str = "communication.sms_gateway.configured";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_gateway";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_gateway_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_gateway_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `SmsGateway` is activated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsGatewayActivated {
    pub sms_gateway_id: SmsGatewayId,
    pub gateway_type: GatewayType,
    pub previous_id: Option<SmsGatewayId>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsGatewayActivated {
    pub fn new(
        sms_gateway_id: SmsGatewayId,
        gateway_type: GatewayType,
        previous_id: Option<SmsGatewayId>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_gateway_id,
            gateway_type,
            previous_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsGatewayActivated {
    const EVENT_TYPE: &'static str = "communication.sms_gateway.activated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_gateway";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_gateway_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_gateway_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `SmsGateway` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsGatewayDeleted {
    pub sms_gateway_id: SmsGatewayId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SmsGatewayDeleted {
    pub fn new(
        sms_gateway_id: SmsGatewayId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            sms_gateway_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SmsGatewayDeleted {
    const EVENT_TYPE: &'static str = "communication.sms_gateway.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "sms_gateway";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.sms_gateway_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.sms_gateway_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// NotificationSetting events (3)
// =============================================================================

/// Emitted when a new `NotificationSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationSettingCreated {
    pub notification_setting_id: NotificationSettingId,
    pub event: String,
    pub destination: Destination,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationSettingCreated {
    pub fn new(
        notification_setting_id: NotificationSettingId,
        event: String,
        destination: Destination,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_setting_id,
            event,
            destination,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationSettingCreated {
    const EVENT_TYPE: &'static str = "communication.notification_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `NotificationSetting` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationSettingUpdated {
    pub notification_setting_id: NotificationSettingId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationSettingUpdated {
    pub fn new(
        notification_setting_id: NotificationSettingId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_setting_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationSettingUpdated {
    const EVENT_TYPE: &'static str = "communication.notification_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `NotificationSetting` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationSettingDeleted {
    pub notification_setting_id: NotificationSettingId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl NotificationSettingDeleted {
    pub fn new(
        notification_setting_id: NotificationSettingId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            notification_setting_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for NotificationSettingDeleted {
    const EVENT_TYPE: &'static str = "communication.notification_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "notification_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.notification_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.notification_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// AbsentNotification events (5)
// =============================================================================

/// Emitted when an `AbsentNotificationTimeSetup` is scheduled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationScheduled {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub time_from: TimeOfDay,
    pub time_to: TimeOfDay,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsentNotificationScheduled {
    pub fn new(
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        time_from: TimeOfDay,
        time_to: TimeOfDay,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            absent_notification_time_setup_id,
            time_from,
            time_to,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsentNotificationScheduled {
    const EVENT_TYPE: &'static str = "communication.absent_notification.scheduled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "absent_notification_time_setup";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.absent_notification_time_setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.absent_notification_time_setup_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `AbsentNotificationTimeSetup` is enabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationEnabled {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsentNotificationEnabled {
    pub fn new(
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            absent_notification_time_setup_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsentNotificationEnabled {
    const EVENT_TYPE: &'static str = "communication.absent_notification.enabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "absent_notification_time_setup";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.absent_notification_time_setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.absent_notification_time_setup_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `AbsentNotificationTimeSetup` is disabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationDisabled {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsentNotificationDisabled {
    pub fn new(
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            absent_notification_time_setup_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsentNotificationDisabled {
    const EVENT_TYPE: &'static str = "communication.absent_notification.disabled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "absent_notification_time_setup";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.absent_notification_time_setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.absent_notification_time_setup_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an `AbsentNotificationTimeSetup` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationDeleted {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsentNotificationDeleted {
    pub fn new(
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            absent_notification_time_setup_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsentNotificationDeleted {
    const EVENT_TYPE: &'static str = "communication.absent_notification.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "absent_notification_time_setup";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.absent_notification_time_setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.absent_notification_time_setup_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when an absent-notification dispatch is sent to a
/// student's guardian. The aggregate id is the
/// `absent_notification_time_setup_id` (the schedule row that
/// triggered the dispatch), per the Phase 10 spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationSent {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub student_id: StudentId,
    pub channel: Channel,
    pub template_id: SmsTemplateId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl AbsentNotificationSent {
    pub fn new(
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        student_id: StudentId,
        channel: Channel,
        template_id: SmsTemplateId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            absent_notification_time_setup_id,
            student_id,
            channel,
            template_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for AbsentNotificationSent {
    const EVENT_TYPE: &'static str = "communication.absent_notification.sent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "absent_notification_time_setup";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.absent_notification_time_setup_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.absent_notification_time_setup_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat 1-to-1 events (5)
// =============================================================================

/// Emitted when a new 1-to-1 `ChatConversation` is opened.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConversationOpened {
    pub chat_conversation_id: ChatConversationId,
    pub from_id: UserId,
    pub to_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatConversationOpened {
    pub fn new(
        chat_conversation_id: ChatConversationId,
        from_id: UserId,
        to_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_conversation_id,
            from_id,
            to_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatConversationOpened {
    const EVENT_TYPE: &'static str = "communication.chat_conversation.opened";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_conversation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_conversation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_conversation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a 1-to-1 `ChatConversation` is closed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConversationClosed {
    pub chat_conversation_id: ChatConversationId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatConversationClosed {
    pub fn new(
        chat_conversation_id: ChatConversationId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_conversation_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatConversationClosed {
    const EVENT_TYPE: &'static str = "communication.chat_conversation.closed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_conversation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_conversation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_conversation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatMessage` is sent in a 1-to-1 conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageSent {
    pub chat_message_id: ChatMessageId,
    pub chat_conversation_id: ChatConversationId,
    pub from_id: UserId,
    pub to_id: UserId,
    pub message_type: MessageType,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatMessageSent {
    pub fn new(
        chat_message_id: ChatMessageId,
        chat_conversation_id: ChatConversationId,
        from_id: UserId,
        to_id: UserId,
        message_type: MessageType,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_message_id,
            chat_conversation_id,
            from_id,
            to_id,
            message_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatMessageSent {
    const EVENT_TYPE: &'static str = "communication.chat_message.sent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatMessage` is seen by its recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageSeen {
    pub chat_message_id: ChatMessageId,
    pub seen_by: UserId,
    pub seen_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatMessageSeen {
    pub fn new(
        chat_message_id: ChatMessageId,
        seen_by: UserId,
        seen_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_message_id,
            seen_by,
            seen_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatMessageSeen {
    const EVENT_TYPE: &'static str = "communication.chat_message.seen";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatMessage` is soft-deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessageDeleted {
    pub chat_message_id: ChatMessageId,
    pub deleted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatMessageDeleted {
    pub fn new(
        chat_message_id: ChatMessageId,
        deleted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_message_id,
            deleted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatMessageDeleted {
    const EVENT_TYPE: &'static str = "communication.chat_message.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat group events (4)
// =============================================================================

/// Emitted when a new `ChatGroup` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupCreated {
    pub chat_group_id: ChatGroupId,
    pub name: String,
    pub privacy: ChatGroupPrivacy,
    pub group_type: ChatGroupType,
    pub created_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupCreated {
    pub fn new(
        chat_group_id: ChatGroupId,
        name: String,
        privacy: ChatGroupPrivacy,
        group_type: ChatGroupType,
        created_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            name,
            privacy,
            group_type,
            created_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupCreated {
    const EVENT_TYPE: &'static str = "communication.chat_group.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatGroup` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupUpdated {
    pub chat_group_id: ChatGroupId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupUpdated {
    pub fn new(
        chat_group_id: ChatGroupId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupUpdated {
    const EVENT_TYPE: &'static str = "communication.chat_group.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatGroup`'s read-only flag is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupReadOnlySet {
    pub chat_group_id: ChatGroupId,
    pub read_only: bool,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupReadOnlySet {
    pub fn new(
        chat_group_id: ChatGroupId,
        read_only: bool,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            read_only,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupReadOnlySet {
    const EVENT_TYPE: &'static str = "communication.chat_group.read_only_set";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatGroup` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupDeleted {
    pub chat_group_id: ChatGroupId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupDeleted {
    pub fn new(
        chat_group_id: ChatGroupId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupDeleted {
    const EVENT_TYPE: &'static str = "communication.chat_group.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat group membership events (3)
// =============================================================================

/// Emitted when a user is added to a `ChatGroup`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupUserAdded {
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
    pub role: ChatGroupRole,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupUserAdded {
    pub fn new(
        chat_group_id: ChatGroupId,
        user_id: UserId,
        role: ChatGroupRole,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            user_id,
            role,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupUserAdded {
    const EVENT_TYPE: &'static str = "communication.chat_group_user.added";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_user";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `ChatGroup` member's role is changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupUserRoleChanged {
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
    pub from: ChatGroupRole,
    pub to: ChatGroupRole,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupUserRoleChanged {
    pub fn new(
        chat_group_id: ChatGroupId,
        user_id: UserId,
        from: ChatGroupRole,
        to: ChatGroupRole,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            user_id,
            from,
            to,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupUserRoleChanged {
    const EVENT_TYPE: &'static str = "communication.chat_group_user.role_changed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_user";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a user is removed from a `ChatGroup`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupUserRemoved {
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
    pub removed_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatGroupUserRemoved {
    pub fn new(
        chat_group_id: ChatGroupId,
        user_id: UserId,
        removed_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_id,
            user_id,
            removed_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatGroupUserRemoved {
    const EVENT_TYPE: &'static str = "communication.chat_group_user.removed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_user";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat group delivery events (3)
// =============================================================================

/// Emitted when a per-recipient delivery row is recorded for a
/// group message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupMessageRecipientRecorded {
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl GroupMessageRecipientRecorded {
    pub fn new(
        chat_group_message_recipient_id: ChatGroupMessageRecipientId,
        chat_group_id: ChatGroupId,
        user_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_message_recipient_id,
            chat_group_id,
            user_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GroupMessageRecipientRecorded {
    const EVENT_TYPE: &'static str = "communication.chat_group_message_recipient.recorded";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_message_recipient";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_message_recipient_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_message_recipient_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a recipient marks a group message as read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupMessageMarkedRead {
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    pub read_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl GroupMessageMarkedRead {
    pub fn new(
        chat_group_message_recipient_id: ChatGroupMessageRecipientId,
        read_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_message_recipient_id,
            read_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GroupMessageMarkedRead {
    const EVENT_TYPE: &'static str = "communication.chat_group_message_recipient.marked_read";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_message_recipient";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_message_recipient_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_message_recipient_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a group message is removed for a single user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupMessageRemovedForUser {
    pub chat_group_message_remove_id: ChatGroupMessageRemoveId,
    pub user_id: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl GroupMessageRemovedForUser {
    pub fn new(
        chat_group_message_remove_id: ChatGroupMessageRemoveId,
        user_id: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_group_message_remove_id,
            user_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for GroupMessageRemovedForUser {
    const EVENT_TYPE: &'static str = "communication.chat_group_message_remove.removed_for_user";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_group_message_remove";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_group_message_remove_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_group_message_remove_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat block / invitation / status events (6)
// =============================================================================

/// Emitted when a user blocks another user. The block row has no
/// typed id (per the manifest, the natural key is the
/// `(block_by, block_to)` pair), so `aggregate_id()` is derived
/// deterministically from `block_by` via a v5 UUID in the
/// `AGGREGATE_ID_NAMESPACE`. The `school_id` is carried on the
/// event payload because `UserId` is not school-scoped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserBlocked {
    pub school_id: SchoolId,
    pub block_by: UserId,
    pub block_to: UserId,
    pub blocked_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl UserBlocked {
    pub fn new(
        school_id: SchoolId,
        block_by: UserId,
        block_to: UserId,
        blocked_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            block_by,
            block_to,
            blocked_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserBlocked {
    const EVENT_TYPE: &'static str = "communication.chat_block_user.blocked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_block_user";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        Uuid::new_v5(&AGGREGATE_ID_NAMESPACE, self.block_by.as_uuid().as_bytes())
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a user unblocks another user. `aggregate_id()` is
/// derived from `block_by` for the same reason as
/// [`UserBlocked`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserUnblocked {
    pub school_id: SchoolId,
    pub block_by: UserId,
    pub block_to: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl UserUnblocked {
    pub fn new(
        school_id: SchoolId,
        block_by: UserId,
        block_to: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            block_by,
            block_to,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for UserUnblocked {
    const EVENT_TYPE: &'static str = "communication.chat_block_user.unblocked";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_block_user";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        Uuid::new_v5(&AGGREGATE_ID_NAMESPACE, self.block_by.as_uuid().as_bytes())
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a chat invitation is sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitationSent {
    pub chat_invitation_id: ChatInvitationId,
    pub from: UserId,
    pub to: UserId,
    pub invitation_type: ChatInvitationTypeEnum,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatInvitationSent {
    pub fn new(
        chat_invitation_id: ChatInvitationId,
        from: UserId,
        to: UserId,
        invitation_type: ChatInvitationTypeEnum,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_invitation_id,
            from,
            to,
            invitation_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatInvitationSent {
    const EVENT_TYPE: &'static str = "communication.chat_invitation.sent";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_invitation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_invitation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_invitation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a chat invitation is accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitationAccepted {
    pub chat_invitation_id: ChatInvitationId,
    pub accepted_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatInvitationAccepted {
    pub fn new(
        chat_invitation_id: ChatInvitationId,
        accepted_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_invitation_id,
            accepted_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatInvitationAccepted {
    const EVENT_TYPE: &'static str = "communication.chat_invitation.accepted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_invitation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_invitation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_invitation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a chat invitation is rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitationRejected {
    pub chat_invitation_id: ChatInvitationId,
    pub rejected_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatInvitationRejected {
    pub fn new(
        chat_invitation_id: ChatInvitationId,
        rejected_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_invitation_id,
            rejected_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatInvitationRejected {
    const EVENT_TYPE: &'static str = "communication.chat_invitation.rejected";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_invitation";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_invitation_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_invitation_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a user sets their chat status. There is no typed
/// id for the per-user status row, so `aggregate_id()` is
/// deterministically derived from `user_id` via a v5 UUID in the
/// `AGGREGATE_ID_NAMESPACE`. The `school_id` is carried on the
/// event payload because `UserId` is not school-scoped.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatStatusSet {
    pub school_id: SchoolId,
    pub user_id: UserId,
    pub status: ChatStatus,
    pub set_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatStatusSet {
    pub fn new(
        school_id: SchoolId,
        user_id: UserId,
        status: ChatStatus,
        set_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            school_id,
            user_id,
            status,
            set_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatStatusSet {
    const EVENT_TYPE: &'static str = "communication.chat_status.set";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_status";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        Uuid::new_v5(&AGGREGATE_ID_NAMESPACE, self.user_id.as_uuid().as_bytes())
    }
    fn school_id(&self) -> SchoolId {
        self.school_id
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// Chat invitation classification event (1)
// =============================================================================

/// Emitted when a chat invitation is classified into a
/// `ChatInvitationType` row. Per the manifest, `aggregate_id()` is
/// the `chat_invitation_type_id` (the classification row, not the
/// invitation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatInvitationClassified {
    pub chat_invitation_type_id: ChatInvitationTypeId,
    pub invitation_id: ChatInvitationId,
    pub invitation_type: ChatInvitationTypeEnum,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ChatInvitationClassified {
    pub fn new(
        chat_invitation_type_id: ChatInvitationTypeId,
        invitation_id: ChatInvitationId,
        invitation_type: ChatInvitationTypeEnum,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            chat_invitation_type_id,
            invitation_id,
            invitation_type,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ChatInvitationClassified {
    const EVENT_TYPE: &'static str = "communication.chat_invitation_type.classified";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "chat_invitation_type";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.chat_invitation_type_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.chat_invitation_type_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SendMessage events (3)
// =============================================================================

/// Emitted when a `SendMessage` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageCreated {
    pub send_message_id: SendMessageId,
    pub audience: AudienceDescriptor,
    pub publish_on: Option<NaiveDate>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SendMessageCreated {
    pub fn new(
        send_message_id: SendMessageId,
        audience: AudienceDescriptor,
        publish_on: Option<NaiveDate>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            send_message_id,
            audience,
            publish_on,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SendMessageCreated {
    const EVENT_TYPE: &'static str = "communication.send_message.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "send_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.send_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.send_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SendMessage` is dispatched to its audience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageDispatched {
    pub send_message_id: SendMessageId,
    pub recipient_count: u32,
    pub dispatched_at: Timestamp,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SendMessageDispatched {
    pub fn new(
        send_message_id: SendMessageId,
        recipient_count: u32,
        dispatched_at: Timestamp,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            send_message_id,
            recipient_count,
            dispatched_at,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SendMessageDispatched {
    const EVENT_TYPE: &'static str = "communication.send_message.dispatched";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "send_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.send_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.send_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SendMessage` is cancelled before dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageCancelled {
    pub send_message_id: SendMessageId,
    pub reason: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SendMessageCancelled {
    pub fn new(
        send_message_id: SendMessageId,
        reason: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            send_message_id,
            reason,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SendMessageCancelled {
    const EVENT_TYPE: &'static str = "communication.send_message.cancelled";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "send_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.send_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.send_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// ContactMessage events (3)
// =============================================================================

/// Emitted when a public contact-form message is received.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMessageReceived {
    pub contact_message_id: ContactMessageId,
    pub name: PersonName,
    pub email: EmailAddress,
    pub phone: PhoneNumber,
    pub subject: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ContactMessageReceived {
    pub fn new(
        contact_message_id: ContactMessageId,
        name: PersonName,
        email: EmailAddress,
        phone: PhoneNumber,
        subject: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            contact_message_id,
            name,
            email,
            phone,
            subject,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ContactMessageReceived {
    const EVENT_TYPE: &'static str = "communication.contact_message.received";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.contact_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a staff member views a contact message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMessageViewed {
    pub contact_message_id: ContactMessageId,
    pub viewed_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ContactMessageViewed {
    pub fn new(
        contact_message_id: ContactMessageId,
        viewed_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            contact_message_id,
            viewed_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ContactMessageViewed {
    const EVENT_TYPE: &'static str = "communication.contact_message.viewed";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.contact_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a staff member replies to a contact message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMessageReplied {
    pub contact_message_id: ContactMessageId,
    pub reply_channel: Channel,
    pub replied_by: UserId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl ContactMessageReplied {
    pub fn new(
        contact_message_id: ContactMessageId,
        reply_channel: Channel,
        replied_by: UserId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            contact_message_id,
            reply_channel,
            replied_by,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for ContactMessageReplied {
    const EVENT_TYPE: &'static str = "communication.contact_message.replied";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "contact_message";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.contact_message_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.contact_message_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// SpeechSlider events (3)
// =============================================================================

/// Emitted when a new `SpeechSlider` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderCreated {
    pub speech_slider_id: SpeechSliderId,
    pub name: String,
    pub designation: String,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SpeechSliderCreated {
    pub fn new(
        speech_slider_id: SpeechSliderId,
        name: String,
        designation: String,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            speech_slider_id,
            name,
            designation,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SpeechSliderCreated {
    const EVENT_TYPE: &'static str = "communication.speech_slider.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.speech_slider_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SpeechSlider` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderUpdated {
    pub speech_slider_id: SpeechSliderId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SpeechSliderUpdated {
    pub fn new(
        speech_slider_id: SpeechSliderId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            speech_slider_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SpeechSliderUpdated {
    const EVENT_TYPE: &'static str = "communication.speech_slider.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.speech_slider_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `SpeechSlider` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeechSliderDeleted {
    pub speech_slider_id: SpeechSliderId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl SpeechSliderDeleted {
    pub fn new(
        speech_slider_id: SpeechSliderId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            speech_slider_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for SpeechSliderDeleted {
    const EVENT_TYPE: &'static str = "communication.speech_slider.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "speech_slider";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.speech_slider_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.speech_slider_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// PhoneCallLog events (2)
// =============================================================================

/// Emitted when a phone call is logged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneCallLogged {
    pub phone_call_log_id: PhoneCallLogId,
    pub name: PersonName,
    pub phone: PhoneNumber,
    pub call_type: CallType,
    pub date: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PhoneCallLogged {
    pub fn new(
        phone_call_log_id: PhoneCallLogId,
        name: PersonName,
        phone: PhoneNumber,
        call_type: CallType,
        date: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phone_call_log_id,
            name,
            phone,
            call_type,
            date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PhoneCallLogged {
    const EVENT_TYPE: &'static str = "communication.phone_call_log.logged";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "phone_call_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.phone_call_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.phone_call_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when the follow-up date on a phone-call log is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneCallFollowUpUpdated {
    pub phone_call_log_id: PhoneCallLogId,
    pub next_follow_up_date: NaiveDate,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl PhoneCallFollowUpUpdated {
    pub fn new(
        phone_call_log_id: PhoneCallLogId,
        next_follow_up_date: NaiveDate,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            phone_call_log_id,
            next_follow_up_date,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for PhoneCallFollowUpUpdated {
    const EVENT_TYPE: &'static str = "communication.phone_call_log.follow_up_updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "phone_call_log";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.phone_call_log_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.phone_call_log_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

// =============================================================================
// CustomSmsSetting events (3)
// =============================================================================

/// Emitted when a new `CustomSmsSetting` is created.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomSmsSettingCreated {
    pub custom_sms_setting_id: CustomSmsSettingId,
    pub gateway_id: SmsGatewayId,
    pub gateway_url: Url,
    pub request_method: RequestMethod,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CustomSmsSettingCreated {
    pub fn new(
        custom_sms_setting_id: CustomSmsSettingId,
        gateway_id: SmsGatewayId,
        gateway_url: Url,
        request_method: RequestMethod,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            custom_sms_setting_id,
            gateway_id,
            gateway_url,
            request_method,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CustomSmsSettingCreated {
    const EVENT_TYPE: &'static str = "communication.custom_sms_setting.created";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "custom_sms_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.custom_sms_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.custom_sms_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `CustomSmsSetting` is updated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomSmsSettingUpdated {
    pub custom_sms_setting_id: CustomSmsSettingId,
    pub changes: Vec<String>,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CustomSmsSettingUpdated {
    pub fn new(
        custom_sms_setting_id: CustomSmsSettingId,
        changes: Vec<String>,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            custom_sms_setting_id,
            changes,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CustomSmsSettingUpdated {
    const EVENT_TYPE: &'static str = "communication.custom_sms_setting.updated";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "custom_sms_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.custom_sms_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.custom_sms_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}

/// Emitted when a `CustomSmsSetting` is deleted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomSmsSettingDeleted {
    pub custom_sms_setting_id: CustomSmsSettingId,
    pub event_id: EventId,
    pub correlation_id: CorrelationId,
    pub occurred_at: Timestamp,
}

impl CustomSmsSettingDeleted {
    pub fn new(
        custom_sms_setting_id: CustomSmsSettingId,
        event_id: EventId,
        correlation_id: CorrelationId,
        occurred_at: Timestamp,
    ) -> Self {
        Self {
            custom_sms_setting_id,
            event_id,
            correlation_id,
            occurred_at,
        }
    }
}

impl DomainEvent for CustomSmsSettingDeleted {
    const EVENT_TYPE: &'static str = "communication.custom_sms_setting.deleted";
    const SCHEMA_VERSION: u32 = 1;
    const AGGREGATE_TYPE: &'static str = "custom_sms_setting";
    fn event_id(&self) -> EventId {
        self.event_id
    }
    fn aggregate_id(&self) -> Uuid {
        self.custom_sms_setting_id.as_uuid()
    }
    fn school_id(&self) -> SchoolId {
        self.custom_sms_setting_id.school_id()
    }
    fn occurred_at(&self) -> Timestamp {
        self.occurred_at
    }
}
