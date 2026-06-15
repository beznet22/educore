//! # Communication domain queries
//!
//! Phase 10 ships the 26 typed query stubs (one per root
//! aggregate). Each query has a `query_type` method that returns
//! a stable dotted string, and an `execute` method that returns
//! `Err(DomainError::not_supported(...))` for now. The typed
//! executors land in a follow-up phase alongside the
//! `#[derive(DomainQuery)]` macro emissions (per the Phase 7
//! Workstream P pattern).
//!
//! Mirrors `crates/domains/library/src/query.rs` and
//! `crates/domains/facilities/src/query.rs`.

#![allow(missing_docs)]
#![allow(unused_imports)]

use chrono::NaiveDate;

use educore_core::error::{DomainError, Result};
use educore_core::ids::{SchoolId, UserId};

use crate::aggregate::*;
use crate::value_objects::*;

// =============================================================================
// NoticeQuery
// =============================================================================

/// A typed query for `Notice` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct NoticeQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only currently-published, in-window notices.
    pub active_only: bool,
    /// Filter by audience descriptor.
    pub audience_filter: Option<AudienceDescriptor>,
    /// Optional inclusive date range on `notice_date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional notice-type filter.
    pub notice_type: Option<NoticeType>,
    /// Optional status filter.
    pub status: Option<NoticeStatus>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl NoticeQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.notice.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10; the typed executor lands in a follow-up phase.
    pub async fn execute(&self) -> Result<Vec<Notice>> {
        Err(DomainError::not_supported(
            "NoticeQuery::execute is a Phase 10 stub; real executor lands with the DomainQuery macro",
        ))
    }
}

// =============================================================================
// ComplaintQuery
// =============================================================================

/// A typed query for `Complaint` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplaintQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only open (unresolved) complaints.
    pub active_only: bool,
    /// Filter by complaint status.
    pub status: Option<ComplaintStatus>,
    /// Filter by assignee.
    pub assignee_user_id: Option<UserId>,
    /// Filter by complaint type.
    pub complaint_type_id: Option<ComplaintTypeId>,
    /// Filter by complaint source channel.
    pub complaint_source: Option<ComplaintSource>,
    /// Optional inclusive date range on `date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ComplaintQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.complaint.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<Complaint>> {
        Err(DomainError::not_supported(
            "ComplaintQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ComplaintTypeQuery
// =============================================================================

/// A typed query for `ComplaintType` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplaintTypeQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted complaint types.
    pub active_only: bool,
    /// Free-text search by type name.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ComplaintTypeQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.complaint_type.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ComplaintType>> {
        Err(DomainError::not_supported(
            "ComplaintTypeQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// NotificationQuery
// =============================================================================

/// A typed query for `Notification` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-withdrawn, non-failed notifications.
    pub active_only: bool,
    /// Filter by recipient.
    pub recipient_user_id: Option<UserId>,
    /// Filter by notification type.
    pub notification_type: Option<NotificationType>,
    /// Filter by status.
    pub status: Option<NotificationStatus>,
    /// If `Some(true)`, return only unread notifications for
    /// `recipient_user_id`.
    pub unread_only: Option<bool>,
    /// Optional inclusive date range on `occurred_at`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl NotificationQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.notification.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<Notification>> {
        Err(DomainError::not_supported(
            "NotificationQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// EmailLogQuery
// =============================================================================

/// A typed query for `EmailLog` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailLogQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only successful deliveries.
    pub active_only: bool,
    /// Filter by recipient address.
    pub recipient: Option<EmailAddress>,
    /// Filter by mail driver.
    pub mail_driver: Option<MailDriver>,
    /// Optional inclusive date range on `send_date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl EmailLogQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.email_log.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<EmailLog>> {
        Err(DomainError::not_supported(
            "EmailLogQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// SmsLogQuery
// =============================================================================

/// A typed query for `SmsLog` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SmsLogQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only successful deliveries.
    pub active_only: bool,
    /// Filter by recipient phone number.
    pub recipient: Option<PhoneNumber>,
    /// Filter by gateway.
    pub gateway_id: Option<SmsGatewayId>,
    /// Optional inclusive date range on `send_date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl SmsLogQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.sms_log.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<SmsLog>> {
        Err(DomainError::not_supported(
            "SmsLogQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// SmsTemplateQuery
// =============================================================================

/// A typed query for `SmsTemplate` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SmsTemplateQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only enabled templates.
    pub active_only: bool,
    /// Filter by channel.
    pub channel: Option<Channel>,
    /// Filter by enabled/disabled status.
    pub status: Option<SmsTemplateStatus>,
    /// Free-text search by purpose or subject.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl SmsTemplateQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.sms_template.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<SmsTemplate>> {
        Err(DomainError::not_supported(
            "SmsTemplateQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// EmailSettingQuery
// =============================================================================

/// A typed query for `EmailSetting` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailSettingQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only the currently-active email setting.
    pub active_only: bool,
    /// Filter by mail driver.
    pub mail_driver: Option<MailDriver>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl EmailSettingQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.email_setting.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<EmailSetting>> {
        Err(DomainError::not_supported(
            "EmailSettingQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// SmsGatewayQuery
// =============================================================================

/// A typed query for `SmsGateway` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SmsGatewayQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only the currently-active gateway.
    pub active_only: bool,
    /// Filter by gateway type.
    pub gateway_type: Option<GatewayType>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl SmsGatewayQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.sms_gateway.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<SmsGateway>> {
        Err(DomainError::not_supported(
            "SmsGatewayQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// NotificationSettingQuery
// =============================================================================

/// A typed query for `NotificationSetting` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationSettingQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted settings.
    pub active_only: bool,
    /// Filter by event key.
    pub event: Option<String>,
    /// Filter by destination bitflag.
    pub destination: Option<Destination>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl NotificationSettingQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.notification_setting.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<NotificationSetting>> {
        Err(DomainError::not_supported(
            "NotificationSettingQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// AbsentNotificationTimeSetupQuery
// =============================================================================

/// A typed query for `AbsentNotificationTimeSetup` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct AbsentNotificationTimeSetupQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only enabled time setups.
    pub active_only: bool,
    /// Filter by enabled/disabled status.
    pub status: Option<AbsentNotificationStatus>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl AbsentNotificationTimeSetupQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.absent_notification_time_setup.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<AbsentNotificationTimeSetup>> {
        Err(DomainError::not_supported(
            "AbsentNotificationTimeSetupQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatMessageQuery
// =============================================================================

/// A typed query for `ChatMessage` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessageQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted messages.
    pub active_only: bool,
    /// Filter by 1-to-1 conversation.
    pub conversation_id: Option<ChatConversationId>,
    /// Filter by sender.
    pub from_id: Option<UserId>,
    /// Filter by recipient.
    pub to_id: Option<UserId>,
    /// Filter by message type.
    pub message_type: Option<MessageType>,
    /// Filter by seen/unseen status.
    pub status: Option<ChatMessageStatus>,
    /// Optional inclusive date range on `sent_at`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatMessageQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_message.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatMessage>> {
        Err(DomainError::not_supported(
            "ChatMessageQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatConversationQuery
// =============================================================================

/// A typed query for `ChatConversation` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatConversationQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only currently-open conversations.
    pub active_only: bool,
    /// Filter by participant.
    pub user_id: Option<UserId>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatConversationQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_conversation.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatConversation>> {
        Err(DomainError::not_supported(
            "ChatConversationQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatGroupQuery
// =============================================================================

/// A typed query for `ChatGroup` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatGroupQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted groups.
    pub active_only: bool,
    /// Filter by privacy.
    pub privacy: Option<ChatGroupPrivacy>,
    /// Filter by group type (open/closed).
    pub group_type: Option<ChatGroupType>,
    /// Filter by member.
    pub member_user_id: Option<UserId>,
    /// Free-text search by name.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatGroupQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_group.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatGroup>> {
        Err(DomainError::not_supported(
            "ChatGroupQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatGroupUserQuery
// =============================================================================

/// A typed query for `ChatGroupUser` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatGroupUserQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only currently-active memberships.
    pub active_only: bool,
    /// Filter by group.
    pub group_id: Option<ChatGroupId>,
    /// Filter by user.
    pub user_id: Option<UserId>,
    /// Filter by role (member/admin).
    pub role: Option<ChatGroupRole>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatGroupUserQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_group_user.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatGroupUser>> {
        Err(DomainError::not_supported(
            "ChatGroupUserQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatGroupMessageRecipientQuery
// =============================================================================

/// A typed query for `ChatGroupMessageRecipient` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatGroupMessageRecipientQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-removed recipients.
    pub active_only: bool,
    /// Filter by group.
    pub group_id: Option<ChatGroupId>,
    /// Filter by user.
    pub user_id: Option<UserId>,
    /// Filter by source message.
    pub message_id: Option<ChatMessageId>,
    /// If `Some(true)`, return only unread recipient rows.
    pub unread_only: Option<bool>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatGroupMessageRecipientQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_group_message_recipient.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatGroupMessageRecipient>> {
        Err(DomainError::not_supported(
            "ChatGroupMessageRecipientQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatGroupMessageRemoveQuery
// =============================================================================

/// A typed query for `ChatGroupMessageRemove` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatGroupMessageRemoveQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-retracted removal records.
    pub active_only: bool,
    /// Filter by group.
    pub group_id: Option<ChatGroupId>,
    /// Filter by user.
    pub user_id: Option<UserId>,
    /// Filter by source message.
    pub message_id: Option<ChatMessageId>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatGroupMessageRemoveQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_group_message_remove.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatGroupMessageRemove>> {
        Err(DomainError::not_supported(
            "ChatGroupMessageRemoveQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatBlockUserQuery
// =============================================================================

/// A typed query for `ChatBlockUser` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatBlockUserQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only currently-active blocks.
    pub active_only: bool,
    /// Filter by blocker.
    pub block_by: Option<UserId>,
    /// Filter by blocked user.
    pub block_to: Option<UserId>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatBlockUserQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_block_user.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatBlockUser>> {
        Err(DomainError::not_supported(
            "ChatBlockUserQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatInvitationQuery
// =============================================================================

/// A typed query for `ChatInvitation` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatInvitationQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only pending invitations.
    pub active_only: bool,
    /// Filter by sender.
    pub from: Option<UserId>,
    /// Filter by recipient.
    pub to: Option<UserId>,
    /// Filter by invitation status.
    pub status: Option<ChatInvitationStatus>,
    /// Filter by invitation type.
    pub invitation_type: Option<ChatInvitationTypeEnum>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatInvitationQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_invitation.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatInvitation>> {
        Err(DomainError::not_supported(
            "ChatInvitationQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatInvitationTypeQuery
// =============================================================================

/// A typed query for `ChatInvitationType` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatInvitationTypeQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted classifications.
    pub active_only: bool,
    /// Filter by invitation type.
    pub invitation_type: Option<ChatInvitationTypeEnum>,
    /// Filter by source invitation.
    pub invitation_id: Option<ChatInvitationId>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatInvitationTypeQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_invitation_type.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatInvitationType>> {
        Err(DomainError::not_supported(
            "ChatInvitationTypeQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ChatStatusQuery
// =============================================================================

/// A typed query for `ChatStatus` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatStatusQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only the latest status per user.
    pub active_only: bool,
    /// Filter by user.
    pub user_id: Option<UserId>,
    /// Filter by status.
    pub status: Option<ChatStatus>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ChatStatusQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.chat_status.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ChatStatus>> {
        Err(DomainError::not_supported(
            "ChatStatusQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// SendMessageQuery
// =============================================================================

/// A typed query for `SendMessage` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SendMessageQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-cancelled, non-completed sends.
    pub active_only: bool,
    /// Filter by send status.
    pub status: Option<SendMessageStatus>,
    /// Filter by audience descriptor.
    pub audience_filter: Option<AudienceDescriptor>,
    /// Optional inclusive date range on `notice_date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl SendMessageQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.send_message.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<SendMessage>> {
        Err(DomainError::not_supported(
            "SendMessageQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// ContactMessageQuery
// =============================================================================

/// A typed query for `ContactMessage` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactMessageQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted messages.
    pub active_only: bool,
    /// Filter by viewed/unviewed status.
    pub view_status: Option<ContactMessageViewStatus>,
    /// Filter by replied/unreplied status.
    pub reply_status: Option<ContactMessageReplyStatus>,
    /// Optional inclusive date range on `received_at`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Free-text search by subject or name.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl ContactMessageQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.contact_message.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<ContactMessage>> {
        Err(DomainError::not_supported(
            "ContactMessageQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// SpeechSliderQuery
// =============================================================================

/// A typed query for `SpeechSlider` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct SpeechSliderQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted sliders.
    pub active_only: bool,
    /// Free-text search by name or designation.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl SpeechSliderQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.speech_slider.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<SpeechSlider>> {
        Err(DomainError::not_supported(
            "SpeechSliderQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// PhoneCallLogQuery
// =============================================================================

/// A typed query for `PhoneCallLog` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct PhoneCallLogQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted call logs.
    pub active_only: bool,
    /// Filter by call direction/type.
    pub call_type: Option<CallType>,
    /// If `Some(true)`, return only calls whose
    /// `next_follow_up_date` is on or before today.
    pub follow_up_due_only: Option<bool>,
    /// Optional inclusive date range on `date`.
    pub date_range: Option<(NaiveDate, NaiveDate)>,
    /// Free-text search by name or phone.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl PhoneCallLogQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.phone_call_log.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<PhoneCallLog>> {
        Err(DomainError::not_supported(
            "PhoneCallLogQuery::execute is a Phase 10 stub",
        ))
    }
}

// =============================================================================
// CustomSmsSettingQuery
// =============================================================================

/// A typed query for `CustomSmsSetting` rows.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomSmsSettingQuery {
    /// Tenant anchor.
    pub school_id: SchoolId,
    /// If `true`, return only non-deleted custom settings.
    pub active_only: bool,
    /// Filter by gateway.
    pub gateway_id: Option<SmsGatewayId>,
    /// Filter by request method.
    pub request_method: Option<RequestMethod>,
    /// Free-text search by gateway name.
    pub search: Option<String>,
    /// Optional limit on the number of rows returned.
    pub limit: Option<u32>,
}

impl CustomSmsSettingQuery {
    /// Returns a stable query type identifier.
    #[must_use]
    pub const fn query_type() -> &'static str {
        "communication.custom_sms_setting.query"
    }

    /// Executes the query. Returns `Err(not_supported)` in
    /// Phase 10.
    pub async fn execute(&self) -> Result<Vec<CustomSmsSetting>> {
        Err(DomainError::not_supported(
            "CustomSmsSettingQuery::execute is a Phase 10 stub",
        ))
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

    #[tokio::test]
    async fn every_query_returns_not_supported() {
        let g = SystemIdGen;
        let s = g.next_school_id();
        let u = g.next_user_id();

        let q1 = NoticeQuery {
            school_id: s,
            active_only: true,
            audience_filter: None,
            date_range: None,
            notice_type: None,
            status: None,
            limit: None,
        };
        assert!(q1.execute().await.is_err());

        let q2 = ComplaintQuery {
            school_id: s,
            active_only: true,
            status: None,
            assignee_user_id: Some(u),
            complaint_type_id: None,
            complaint_source: None,
            date_range: None,
            limit: None,
        };
        assert!(q2.execute().await.is_err());

        let q3 = ComplaintTypeQuery {
            school_id: s,
            active_only: true,
            search: None,
            limit: None,
        };
        assert!(q3.execute().await.is_err());

        let q4 = NotificationQuery {
            school_id: s,
            active_only: true,
            recipient_user_id: Some(u),
            notification_type: None,
            status: None,
            unread_only: Some(true),
            date_range: None,
            limit: None,
        };
        assert!(q4.execute().await.is_err());

        let q5 = EmailLogQuery {
            school_id: s,
            active_only: true,
            recipient: None,
            mail_driver: None,
            date_range: None,
            limit: None,
        };
        assert!(q5.execute().await.is_err());

        let q6 = SmsLogQuery {
            school_id: s,
            active_only: true,
            recipient: None,
            gateway_id: None,
            date_range: None,
            limit: None,
        };
        assert!(q6.execute().await.is_err());

        let q7 = SmsTemplateQuery {
            school_id: s,
            active_only: true,
            channel: None,
            status: None,
            search: None,
            limit: None,
        };
        assert!(q7.execute().await.is_err());

        let q8 = EmailSettingQuery {
            school_id: s,
            active_only: true,
            mail_driver: None,
            limit: None,
        };
        assert!(q8.execute().await.is_err());

        let q9 = SmsGatewayQuery {
            school_id: s,
            active_only: true,
            gateway_type: None,
            limit: None,
        };
        assert!(q9.execute().await.is_err());

        let q10 = NotificationSettingQuery {
            school_id: s,
            active_only: true,
            event: None,
            destination: None,
            limit: None,
        };
        assert!(q10.execute().await.is_err());

        let q11 = AbsentNotificationTimeSetupQuery {
            school_id: s,
            active_only: true,
            status: None,
            limit: None,
        };
        assert!(q11.execute().await.is_err());

        let q12 = ChatMessageQuery {
            school_id: s,
            active_only: true,
            conversation_id: None,
            from_id: Some(u),
            to_id: None,
            message_type: None,
            status: None,
            date_range: None,
            limit: None,
        };
        assert!(q12.execute().await.is_err());

        let q13 = ChatConversationQuery {
            school_id: s,
            active_only: true,
            user_id: Some(u),
            limit: None,
        };
        assert!(q13.execute().await.is_err());

        let q14 = ChatGroupQuery {
            school_id: s,
            active_only: true,
            privacy: None,
            group_type: None,
            member_user_id: Some(u),
            search: None,
            limit: None,
        };
        assert!(q14.execute().await.is_err());

        let q15 = ChatGroupUserQuery {
            school_id: s,
            active_only: true,
            group_id: None,
            user_id: Some(u),
            role: None,
            limit: None,
        };
        assert!(q15.execute().await.is_err());

        let q16 = ChatGroupMessageRecipientQuery {
            school_id: s,
            active_only: true,
            group_id: None,
            user_id: Some(u),
            message_id: None,
            unread_only: Some(false),
            limit: None,
        };
        assert!(q16.execute().await.is_err());

        let q17 = ChatGroupMessageRemoveQuery {
            school_id: s,
            active_only: true,
            group_id: None,
            user_id: Some(u),
            message_id: None,
            limit: None,
        };
        assert!(q17.execute().await.is_err());

        let q18 = ChatBlockUserQuery {
            school_id: s,
            active_only: true,
            block_by: Some(u),
            block_to: None,
            limit: None,
        };
        assert!(q18.execute().await.is_err());

        let q19 = ChatInvitationQuery {
            school_id: s,
            active_only: true,
            from: Some(u),
            to: None,
            status: None,
            invitation_type: None,
            limit: None,
        };
        assert!(q19.execute().await.is_err());

        let q20 = ChatInvitationTypeQuery {
            school_id: s,
            active_only: true,
            invitation_type: None,
            invitation_id: None,
            limit: None,
        };
        assert!(q20.execute().await.is_err());

        let q21 = ChatStatusQuery {
            school_id: s,
            active_only: true,
            user_id: Some(u),
            status: None,
            limit: None,
        };
        assert!(q21.execute().await.is_err());

        let q22 = SendMessageQuery {
            school_id: s,
            active_only: true,
            status: None,
            audience_filter: None,
            date_range: None,
            limit: None,
        };
        assert!(q22.execute().await.is_err());

        let q23 = ContactMessageQuery {
            school_id: s,
            active_only: true,
            view_status: None,
            reply_status: None,
            date_range: None,
            search: None,
            limit: None,
        };
        assert!(q23.execute().await.is_err());

        let q24 = SpeechSliderQuery {
            school_id: s,
            active_only: true,
            search: None,
            limit: None,
        };
        assert!(q24.execute().await.is_err());

        let q25 = PhoneCallLogQuery {
            school_id: s,
            active_only: true,
            call_type: None,
            follow_up_due_only: Some(false),
            date_range: None,
            search: None,
            limit: None,
        };
        assert!(q25.execute().await.is_err());

        let q26 = CustomSmsSettingQuery {
            school_id: s,
            active_only: true,
            gateway_id: None,
            request_method: None,
            search: None,
            limit: None,
        };
        assert!(q26.execute().await.is_err());
    }

    #[test]
    fn query_type_strings_are_stable() {
        assert_eq!(NoticeQuery::query_type(), "communication.notice.query");
        assert_eq!(
            ComplaintQuery::query_type(),
            "communication.complaint.query"
        );
        assert_eq!(
            ComplaintTypeQuery::query_type(),
            "communication.complaint_type.query"
        );
        assert_eq!(
            NotificationQuery::query_type(),
            "communication.notification.query"
        );
        assert_eq!(EmailLogQuery::query_type(), "communication.email_log.query");
        assert_eq!(SmsLogQuery::query_type(), "communication.sms_log.query");
        assert_eq!(
            SmsTemplateQuery::query_type(),
            "communication.sms_template.query"
        );
        assert_eq!(
            EmailSettingQuery::query_type(),
            "communication.email_setting.query"
        );
        assert_eq!(
            SmsGatewayQuery::query_type(),
            "communication.sms_gateway.query"
        );
        assert_eq!(
            NotificationSettingQuery::query_type(),
            "communication.notification_setting.query"
        );
        assert_eq!(
            AbsentNotificationTimeSetupQuery::query_type(),
            "communication.absent_notification_time_setup.query"
        );
        assert_eq!(
            ChatMessageQuery::query_type(),
            "communication.chat_message.query"
        );
        assert_eq!(
            ChatConversationQuery::query_type(),
            "communication.chat_conversation.query"
        );
        assert_eq!(
            ChatGroupQuery::query_type(),
            "communication.chat_group.query"
        );
        assert_eq!(
            ChatGroupUserQuery::query_type(),
            "communication.chat_group_user.query"
        );
        assert_eq!(
            ChatGroupMessageRecipientQuery::query_type(),
            "communication.chat_group_message_recipient.query"
        );
        assert_eq!(
            ChatGroupMessageRemoveQuery::query_type(),
            "communication.chat_group_message_remove.query"
        );
        assert_eq!(
            ChatBlockUserQuery::query_type(),
            "communication.chat_block_user.query"
        );
        assert_eq!(
            ChatInvitationQuery::query_type(),
            "communication.chat_invitation.query"
        );
        assert_eq!(
            ChatInvitationTypeQuery::query_type(),
            "communication.chat_invitation_type.query"
        );
        assert_eq!(
            ChatStatusQuery::query_type(),
            "communication.chat_status.query"
        );
        assert_eq!(
            SendMessageQuery::query_type(),
            "communication.send_message.query"
        );
        assert_eq!(
            ContactMessageQuery::query_type(),
            "communication.contact_message.query"
        );
        assert_eq!(
            SpeechSliderQuery::query_type(),
            "communication.speech_slider.query"
        );
        assert_eq!(
            PhoneCallLogQuery::query_type(),
            "communication.phone_call_log.query"
        );
        assert_eq!(
            CustomSmsSettingQuery::query_type(),
            "communication.custom_sms_setting.query"
        );
    }
}
