//! # Communication domain entities
//!
//! Child entities have identity and lifecycle but are not aggregate
//! roots. They are loaded and persisted only through their
//! aggregate root.
//!
//! The 11 typed-id child entities (per the manifest's section 2
//! + section 5 events that reference them):
//!
//! - [`NoticeAttachment`] — a file attached to a notice.
//! - [`ComplaintNote`] — a free-text note added to a complaint.
//! - [`NotificationDeliveryAttempt`] — a single delivery attempt
//!   of a notification to one channel.
//! - [`ChatGroupAvatar`] — one avatar snapshot of a chat group.
//! - [`ChatGroupMessage`] — a single message posted in a chat
//!   group.
//! - [`ChatConversationLastRead`] — the per-user "last read"
//!   pointer on a 1-to-1 chat conversation.
//! - [`SendMessageRecipient`] — the per-recipient delivery
//!   outcome of a `SendMessage` dispatch.
//! - [`EmailSettingSecret`] — a secret-rotation snapshot for an
//!   email setting (the secret is never stored in clear text).
//! - [`SmsGatewayCredential`] — a credential-rotation snapshot
//!   for an SMS gateway.
//! - [`AbsentNotificationDispatch`] — one out-bound dispatch
//!   triggered by an absent-notification time setup.
//! - [`ContactMessageReply`] — a staff reply to a contact
//!   message.
//!
//! The 4 embedded value-object types colocated here for
//! proximity (per the prompt's scope):
//!
//! - [`NoticeAudience`] — non-empty list of role ids targeted by
//!   a notice.
//! - [`SmsTemplateVariable`] — a declared placeholder inside an
//!   SMS template body (manifest VO #24).
//! - [`NotificationSettingAudience`] — the typed audience enum
//!   for a notification setting.
//! - [`CustomSmsSettingParam`] — a single key/value parameter
//!   for a custom SMS gateway.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use serde::{Deserialize, Serialize};

use educore_core::error::DomainError;
use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};

use educore_academic::{ClassId, SectionId, StudentId};
use educore_hr::value_objects::RoleId;

use crate::value_objects::{
    AbsentNotificationDispatchId, AbsentNotificationTimeSetupId, ChatConversationId,
    ChatConversationLastReadId, ChatGroupAvatarId, ChatGroupId, ChatGroupMessageId,
    ChatMessageBody, ChatMessageId, Channel, ComplaintId, ComplaintNoteId, ContactMessageId,
    ContactMessageReplyId, EmailSettingId, EmailSettingSecretId, FileReference, MessageType,
    NoticeAttachmentId, NoticeId, NotificationDeliveryAttemptId, NotificationId,
    SecretReference, SendMessageId, SendMessageRecipientId, SmsGatewayCredentialId, SmsGatewayId,
};

// =============================================================================
// DeliveryOutcome (local closed enum)
// =============================================================================

/// The outcome of a single notification delivery attempt.
///
/// Distinct from the per-notification aggregate
/// [`NotificationStatus`](crate::value_objects::NotificationStatus):
/// a single notification may produce many `DeliveryAttempt`s, one
/// per (channel, adapter) pair, each with its own outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryOutcome {
    /// The delivery has been queued but not yet attempted by the
    /// adapter.
    Pending,
    /// The adapter confirmed delivery (HTTP 2xx, gateway ack,
    /// etc.).
    Delivered,
    /// The adapter returned a permanent error; the recipient
    /// will not be retried.
    Failed,
    /// The adapter requested a later retry (rate-limited,
    /// temporary outage).
    Deferred,
}

impl DeliveryOutcome {
    /// Wire-form snake_case string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Deferred => "deferred",
        }
    }
}

// =============================================================================
// NoticeAttachment
// =============================================================================

/// A file attached to a notice. The `display_order` may be
/// amended after creation (e.g. drag-to-reorder in the UI), so
/// this child entity carries the full 8-field audit footer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeAttachment {
    /// The typed id.
    pub id: NoticeAttachmentId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The notice this attachment belongs to.
    pub notice_id: NoticeId,
    /// The file reference (storage key, never the raw bytes).
    pub file: FileReference,
    /// The optional caption rendered beneath the attachment.
    pub caption: Option<String>,
    /// The display order (0-indexed; lower = earlier).
    pub display_order: u32,
    /// Optimistic-concurrency version.
    pub version: Version,
    /// Conditional-update etag.
    pub etag: Etag,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last-update timestamp.
    pub updated_at: Timestamp,
    /// The user who created the attachment.
    pub created_by: UserId,
    /// The user who last amended the attachment.
    pub updated_by: UserId,
    /// The id of the last event that mutated this row.
    pub last_event_id: Option<EventId>,
    /// The correlation id at creation time.
    pub correlation_id: CorrelationId,
}

impl NoticeAttachment {
    /// Constructs a new `NoticeAttachment`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: NoticeAttachmentId,
        notice_id: NoticeId,
        file: FileReference,
        caption: Option<String>,
        display_order: u32,
        created_by: UserId,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            notice_id,
            file,
            caption,
            display_order,
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at,
            updated_at: created_at,
            created_by,
            updated_by: created_by,
            last_event_id: None,
            correlation_id,
        }
    }
}

// =============================================================================
// ComplaintNote
// =============================================================================

/// A free-text administrative note added to a complaint.
/// Append-only: amendments are recorded as new rows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplaintNote {
    /// The typed id.
    pub id: ComplaintNoteId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The complaint this note is attached to.
    pub complaint_id: ComplaintId,
    /// The author of the note.
    pub author: UserId,
    /// The note body (free-text).
    pub body: String,
    /// The created-at timestamp.
    pub created_at: Timestamp,
    /// The correlation id at creation time.
    pub correlation_id: CorrelationId,
}

impl ComplaintNote {
    /// Constructs a new `ComplaintNote`.
    pub fn new(
        id: ComplaintNoteId,
        complaint_id: ComplaintId,
        author: UserId,
        body: String,
        created_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            complaint_id,
            author,
            body,
            created_at,
            correlation_id,
        }
    }
}

// =============================================================================
// NotificationDeliveryAttempt
// =============================================================================

/// A single delivery attempt of a notification to one channel
/// via one adapter. Append-only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationDeliveryAttempt {
    /// The typed id.
    pub id: NotificationDeliveryAttemptId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The notification being delivered.
    pub notification_id: NotificationId,
    /// The channel the attempt targeted.
    pub channel: Channel,
    /// The adapter name that handled the attempt (e.g. "smtp",
    /// "twilio", "web").
    pub adapter: String,
    /// The outcome of the attempt.
    pub outcome: DeliveryOutcome,
    /// The wall-clock time the attempt was dispatched.
    pub attempted_at: Timestamp,
    /// The optional error message (populated on `Failed` or
    /// `Deferred`).
    pub error: Option<String>,
    /// The correlation id at attempt time.
    pub correlation_id: CorrelationId,
}

impl NotificationDeliveryAttempt {
    /// Constructs a new `NotificationDeliveryAttempt`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: NotificationDeliveryAttemptId,
        notification_id: NotificationId,
        channel: Channel,
        adapter: String,
        outcome: DeliveryOutcome,
        attempted_at: Timestamp,
        error: Option<String>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            notification_id,
            channel,
            adapter,
            outcome,
            attempted_at,
            error,
            correlation_id,
        }
    }
}

// =============================================================================
// ChatGroupAvatar
// =============================================================================

/// One avatar snapshot of a chat group. Append-only: a new avatar
/// upload produces a new row; the latest row is the current
/// avatar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupAvatar {
    /// The typed id.
    pub id: ChatGroupAvatarId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The chat group this avatar belongs to.
    pub chat_group_id: ChatGroupId,
    /// The avatar file reference.
    pub file: FileReference,
    /// The user who set this avatar.
    pub set_by: UserId,
    /// The timestamp the avatar was set.
    pub set_at: Timestamp,
    /// The correlation id at set time.
    pub correlation_id: CorrelationId,
}

impl ChatGroupAvatar {
    /// Constructs a new `ChatGroupAvatar`.
    pub fn new(
        id: ChatGroupAvatarId,
        chat_group_id: ChatGroupId,
        file: FileReference,
        set_by: UserId,
        set_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_group_id,
            file,
            set_by,
            set_at,
            correlation_id,
        }
    }
}

// =============================================================================
// ChatGroupMessage
// =============================================================================

/// A single message posted in a chat group. Append-only: edits
/// and deletes are recorded as new events, not amendments to
/// this row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatGroupMessage {
    /// The typed id.
    pub id: ChatGroupMessageId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The chat group the message was posted in.
    pub chat_group_id: ChatGroupId,
    /// The user who sent the message.
    pub sender: UserId,
    /// The message body.
    pub body: ChatMessageBody,
    /// The message type (text, image, pdf, document, voice).
    pub message_type: MessageType,
    /// The optional attached file (for non-text messages).
    pub file: Option<FileReference>,
    /// The sent-at timestamp.
    pub sent_at: Timestamp,
    /// The correlation id at send time.
    pub correlation_id: CorrelationId,
}

impl ChatGroupMessage {
    /// Constructs a new `ChatGroupMessage`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ChatGroupMessageId,
        chat_group_id: ChatGroupId,
        sender: UserId,
        body: ChatMessageBody,
        message_type: MessageType,
        file: Option<FileReference>,
        sent_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_group_id,
            sender,
            body,
            message_type,
            file,
            sent_at,
            correlation_id,
        }
    }
}

// =============================================================================
// ChatConversationLastRead
// =============================================================================

/// The per-user "last read" pointer on a 1-to-1 chat
/// conversation. Each row is a snapshot in time; the latest row
/// per `(chat_conversation_id, user)` is the current pointer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatConversationLastRead {
    /// The typed id.
    pub id: ChatConversationLastReadId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The 1-to-1 chat conversation.
    pub chat_conversation_id: ChatConversationId,
    /// The user whose pointer this is.
    pub user: UserId,
    /// The last message the user has read.
    pub last_read_message_id: ChatMessageId,
    /// The read-at timestamp.
    pub read_at: Timestamp,
    /// The correlation id at read time.
    pub correlation_id: CorrelationId,
}

impl ChatConversationLastRead {
    /// Constructs a new `ChatConversationLastRead`.
    pub fn new(
        id: ChatConversationLastReadId,
        chat_conversation_id: ChatConversationId,
        user: UserId,
        last_read_message_id: ChatMessageId,
        read_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            chat_conversation_id,
            user,
            last_read_message_id,
            read_at,
            correlation_id,
        }
    }
}

// =============================================================================
// SendMessageRecipient
// =============================================================================

/// The per-recipient delivery outcome of a `SendMessage`
/// dispatch. One row per recipient; the aggregate counts rows
/// by outcome to compute dispatch statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendMessageRecipient {
    /// The typed id.
    pub id: SendMessageRecipientId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The `SendMessage` dispatch this row belongs to.
    pub send_message_id: SendMessageId,
    /// The recipient user.
    pub user: UserId,
    /// The channel the message was sent on.
    pub channel: Channel,
    /// The delivery outcome for this recipient.
    pub outcome: DeliveryOutcome,
    /// The delivered-at timestamp (populated on `Delivered`,
    /// `None` for `Pending`/`Failed`/`Deferred`).
    pub delivered_at: Option<Timestamp>,
    /// The correlation id at dispatch time.
    pub correlation_id: CorrelationId,
}

impl SendMessageRecipient {
    /// Constructs a new `SendMessageRecipient`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: SendMessageRecipientId,
        send_message_id: SendMessageId,
        user: UserId,
        channel: Channel,
        outcome: DeliveryOutcome,
        delivered_at: Option<Timestamp>,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            send_message_id,
            user,
            channel,
            outcome,
            delivered_at,
            correlation_id,
        }
    }
}

// =============================================================================
// EmailSettingSecret
// =============================================================================

/// One secret-rotation snapshot for an email setting. The
/// `secret` is a `SecretReference` (vault key / KMS handle) —
/// the cleartext password is never persisted on this row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSettingSecret {
    /// The typed id.
    pub id: EmailSettingSecretId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The email setting this secret belongs to.
    pub email_setting_id: EmailSettingId,
    /// The reference to the secret in the secret store.
    pub secret: SecretReference,
    /// The user who set this secret.
    pub set_by: UserId,
    /// The set-at timestamp.
    pub set_at: Timestamp,
    /// The correlation id at set time.
    pub correlation_id: CorrelationId,
}

impl EmailSettingSecret {
    /// Constructs a new `EmailSettingSecret`.
    pub fn new(
        id: EmailSettingSecretId,
        email_setting_id: EmailSettingId,
        secret: SecretReference,
        set_by: UserId,
        set_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            email_setting_id,
            secret,
            set_by,
            set_at,
            correlation_id,
        }
    }
}

// =============================================================================
// SmsGatewayCredential
// =============================================================================

/// One credential-rotation snapshot for an SMS gateway. The
/// `secret` is a `SecretReference` — cleartext credentials are
/// never persisted on this row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsGatewayCredential {
    /// The typed id.
    pub id: SmsGatewayCredentialId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The SMS gateway this credential belongs to.
    pub sms_gateway_id: SmsGatewayId,
    /// The reference to the credential in the secret store.
    pub secret: SecretReference,
    /// The user who set this credential.
    pub set_by: UserId,
    /// The set-at timestamp.
    pub set_at: Timestamp,
    /// The correlation id at set time.
    pub correlation_id: CorrelationId,
}

impl SmsGatewayCredential {
    /// Constructs a new `SmsGatewayCredential`.
    pub fn new(
        id: SmsGatewayCredentialId,
        sms_gateway_id: SmsGatewayId,
        secret: SecretReference,
        set_by: UserId,
        set_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            sms_gateway_id,
            secret,
            set_by,
            set_at,
            correlation_id,
        }
    }
}

// =============================================================================
// AbsentNotificationDispatch
// =============================================================================

/// One out-bound dispatch triggered by an absent-notification
/// time setup, for a specific student on a specific day. The
/// rendered body and the resolved recipient list are
/// snapshotted on this row (the templates and audience are
/// versioned independently on the aggregate).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsentNotificationDispatch {
    /// The typed id.
    pub id: AbsentNotificationDispatchId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The absent-notification time setup that triggered the
    /// dispatch.
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    /// The student the dispatch is about.
    pub student_id: StudentId,
    /// The channel the dispatch used.
    pub channel: Channel,
    /// The rendered (post-template-substitution) body.
    pub rendered_body: String,
    /// The resolved recipient list.
    pub recipients: Vec<UserId>,
    /// The delivery outcome.
    pub outcome: DeliveryOutcome,
    /// The dispatched-at timestamp.
    pub dispatched_at: Timestamp,
    /// The correlation id at dispatch time.
    pub correlation_id: CorrelationId,
}

impl AbsentNotificationDispatch {
    /// Constructs a new `AbsentNotificationDispatch`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: AbsentNotificationDispatchId,
        absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
        student_id: StudentId,
        channel: Channel,
        rendered_body: String,
        recipients: Vec<UserId>,
        outcome: DeliveryOutcome,
        dispatched_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            absent_notification_time_setup_id,
            student_id,
            channel,
            rendered_body,
            recipients,
            outcome,
            dispatched_at,
            correlation_id,
        }
    }
}

// =============================================================================
// ContactMessageReply
// =============================================================================

/// A staff reply to a contact message. Append-only: follow-up
/// replies are recorded as new rows, not amendments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactMessageReply {
    /// The typed id.
    pub id: ContactMessageReplyId,
    /// The owning school (derived from the id).
    pub school_id: SchoolId,
    /// The contact message this reply is attached to.
    pub contact_message_id: ContactMessageId,
    /// The user who wrote the reply.
    pub author: UserId,
    /// The reply body.
    pub body: String,
    /// The channel the reply was sent on.
    pub channel: Channel,
    /// The replied-at timestamp.
    pub replied_at: Timestamp,
    /// The correlation id at reply time.
    pub correlation_id: CorrelationId,
}

impl ContactMessageReply {
    /// Constructs a new `ContactMessageReply`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ContactMessageReplyId,
        contact_message_id: ContactMessageId,
        author: UserId,
        body: String,
        channel: Channel,
        replied_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> Self {
        Self {
            school_id: id.school_id(),
            id,
            contact_message_id,
            author,
            body,
            channel,
            replied_at,
            correlation_id,
        }
    }
}

// =============================================================================
// Embedded value-object types (colocated for proximity)
// =============================================================================

/// A non-empty list of role ids targeted by a notice.
///
/// Embedded inside the [`Notice`](crate::aggregate::Notice)
/// aggregate. The aggregate-level invariant is that a published
/// notice MUST have a non-empty audience; the constructor
/// enforces this at construction time so the invariant is
/// preserved end-to-end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeAudience(pub Vec<RoleId>);

impl NoticeAudience {
    /// Constructs a new `NoticeAudience`, enforcing non-empty.
    pub fn new(roles: Vec<RoleId>) -> Result<Self, DomainError> {
        if roles.is_empty() {
            return Err(DomainError::validation(
                "notice audience must be non-empty",
            ));
        }
        Ok(Self(roles))
    }

    /// Returns the contained role ids.
    pub fn roles(&self) -> &[RoleId] {
        &self.0
    }
}

/// A declared placeholder inside an SMS template body.
///
/// Manifest value object #24, colocated here for proximity to
/// the [`SmsTemplate`](crate::aggregate::SmsTemplate)
/// aggregate. The aggregate enforces `name` length
/// (1..=50 chars) and `description` length (0..=200 chars) on
/// insertion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsTemplateVariable {
    /// The placeholder name (matches `{{name}}` in the body).
    pub name: String,
    /// The placeholder description (human-readable; rendered
    /// in the template editor UI).
    pub description: String,
}

impl SmsTemplateVariable {
    /// Constructs a new `SmsTemplateVariable`. Length
    /// validation is performed at the aggregate level (the
    /// `SmsTemplate` aggregate calls into
    /// `SmsTemplate::validate_variables`).
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

/// The audience descriptor for a notification setting.
///
/// Embedded inside the
/// [`NotificationSetting`](crate::aggregate::NotificationSetting)
/// aggregate. Each variant is mutually exclusive: a setting
/// targets either roles, a class/section, specific users, or
/// the whole school.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationSettingAudience {
    /// All users holding any of the listed roles.
    Roles(Vec<RoleId>),
    /// All users in a class (optionally restricted to a
    /// specific section).
    ClassSection {
        /// The class id.
        class: ClassId,
        /// The optional section restriction.
        section: Option<SectionId>,
    },
    /// An explicit set of users.
    Users(Vec<UserId>),
    /// All users in the school.
    All,
}

impl NotificationSettingAudience {
    /// Returns a stable, machine-readable discriminant for the
    /// audience variant.
    pub const fn kind(&self) -> NotificationSettingAudienceKind {
        match self {
            Self::Roles(_) => NotificationSettingAudienceKind::Roles,
            Self::ClassSection { .. } => NotificationSettingAudienceKind::ClassSection,
            Self::Users(_) => NotificationSettingAudienceKind::Users,
            Self::All => NotificationSettingAudienceKind::All,
        }
    }
}

/// The discriminant of [`NotificationSettingAudience`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationSettingAudienceKind {
    /// All users holding any of the listed roles.
    Roles,
    /// All users in a class (optionally restricted to a section).
    ClassSection,
    /// An explicit set of users.
    Users,
    /// All users in the school.
    All,
}

impl NotificationSettingAudienceKind {
    /// Wire-form snake_case string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Roles => "roles",
            Self::ClassSection => "class_section",
            Self::Users => "users",
            Self::All => "all",
        }
    }
}

/// A single custom key/value parameter for a custom SMS
/// gateway. Embedded inside the
/// [`CustomSmsSetting`](crate::aggregate::CustomSmsSetting)
/// aggregate.
///
/// The aggregate enforces a maximum of 8 params per setting;
/// the per-row invariant enforced here is that the `key` is
/// non-empty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomSmsSettingParam {
    /// The parameter name (non-empty, validated by `new`).
    pub key: String,
    /// The parameter value (free-form).
    pub value: String,
}

impl CustomSmsSettingParam {
    /// Constructs a new `CustomSmsSettingParam`, enforcing
    /// non-empty `key`. The "max 8 params per setting" cap is
    /// enforced at the aggregate level, not here.
    pub fn new(key: String, value: String) -> Result<Self, DomainError> {
        if key.trim().is_empty() {
            return Err(DomainError::validation(
                "custom sms setting param key must be non-empty",
            ));
        }
        Ok(Self { key, value })
    }
}
