//! # Communication domain commands
//!
//! Every command is a typed struct carrying a `TenantContext`
//! and the typed id of the affected aggregate. Commands are
//! validated, authorized (via the `educore-rbac::Capability`
//! capability check at the dispatcher), and dispatched to the
//! relevant aggregate.
//!
//! Every command produces zero or more events that are recorded
//! in the event log via the bus-port contract
//! (`communication.<aggregate>.<verb>`).
//!
//! Phase 10 ships 72 typed command shapes that drive the 26
//! headline aggregates. The headline service fns (one per
//! command shape that the dispatcher routes to a service
//! factory) are re-exported from the prelude.

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use educore_academic::{ClassId, SectionId, SubjectId};
use educore_core::ids::UserId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;
use educore_hr::value_objects::StaffId;

use crate::entities::{CustomSmsSettingParam, NotificationSettingAudience};
use crate::value_objects::{
    AbsentNotificationTimeSetupId, AudienceDescriptor, CallDescription, CallDuration, CallType,
    Channel, ChatConversationId, ChatGroupId, ChatGroupMessageRecipientId, ChatGroupPrivacy,
    ChatGroupRole, ChatGroupType, ChatInvitationId, ChatInvitationTypeEnum, ChatMessageBody,
    ChatMessageId, ChatStatus, ComplaintDescription, ComplaintId, ComplaintSource, ComplaintStatus,
    ComplaintTypeId, ContactMessageId, CustomSmsSettingId, Destination, EmailAddress,
    EmailSettingId, EmailSubject, FileReference, GatewayName, GatewayType, MailDriver,
    MailEncryption, MessageId, MessageType, NoticeBody, NoticeId, NoticeTitle, NotificationId,
    NotificationMessage, NotificationSettingId, NotificationType, PersonName, PhoneCallLogId,
    PhoneNumber, RequestMethod, SecretReference, SendMessageId, SmsGatewayCredentials,
    SmsGatewayId, SmsTemplateId, SpeechSliderId, SpeechText, TemplateBody, TemplateVariable,
    TimeOfDay, Url,
};

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `communication.<aggregate>.<verb>`).
// =============================================================================

/// Create-notice command type.
pub const COMMUNICATION_NOTICE_CREATE_COMMAND_TYPE: &str = "communication.notice.create";
/// Update-notice command type.
pub const COMMUNICATION_NOTICE_UPDATE_COMMAND_TYPE: &str = "communication.notice.update";
/// Publish-notice command type.
pub const COMMUNICATION_NOTICE_PUBLISH_COMMAND_TYPE: &str = "communication.notice.publish";
/// Unpublish-notice command type.
pub const COMMUNICATION_NOTICE_UNPUBLISH_COMMAND_TYPE: &str = "communication.notice.unpublish";
/// Delete-notice command type.
pub const COMMUNICATION_NOTICE_DELETE_COMMAND_TYPE: &str = "communication.notice.delete";

/// Register-complaint command type.
pub const COMMUNICATION_COMPLAINT_REGISTER_COMMAND_TYPE: &str = "communication.complaint.register";
/// Assign-complaint command type.
pub const COMMUNICATION_COMPLAINT_ASSIGN_COMMAND_TYPE: &str = "communication.complaint.assign";
/// Update-complaint-status command type.
pub const COMMUNICATION_COMPLAINT_UPDATE_STATUS_COMMAND_TYPE: &str =
    "communication.complaint.update_status";
/// Resolve-complaint command type.
pub const COMMUNICATION_COMPLAINT_RESOLVE_COMMAND_TYPE: &str = "communication.complaint.resolve";
/// Add-complaint-note command type.
pub const COMMUNICATION_COMPLAINT_ADD_NOTE_COMMAND_TYPE: &str = "communication.complaint.add_note";

/// Create-complaint-type command type.
pub const COMMUNICATION_COMPLAINT_TYPE_CREATE_COMMAND_TYPE: &str =
    "communication.complaint_type.create";
/// Update-complaint-type command type.
pub const COMMUNICATION_COMPLAINT_TYPE_UPDATE_COMMAND_TYPE: &str =
    "communication.complaint_type.update";
/// Delete-complaint-type command type.
pub const COMMUNICATION_COMPLAINT_TYPE_DELETE_COMMAND_TYPE: &str =
    "communication.complaint_type.delete";

/// Send-notification command type.
pub const COMMUNICATION_NOTIFICATION_SEND_COMMAND_TYPE: &str = "communication.notification.send";
/// Mark-notification-read command type.
pub const COMMUNICATION_NOTIFICATION_READ_COMMAND_TYPE: &str = "communication.notification.read";
/// Withdraw-notification command type.
pub const COMMUNICATION_NOTIFICATION_WITHDRAW_COMMAND_TYPE: &str =
    "communication.notification.withdraw";

/// Log-email-sent command type.
pub const COMMUNICATION_EMAIL_LOG_LOG_COMMAND_TYPE: &str = "communication.email_log.log";
/// Log-sms-sent command type.
pub const COMMUNICATION_SMS_LOG_LOG_COMMAND_TYPE: &str = "communication.sms_log.log";

/// Create-sms-template command type.
pub const COMMUNICATION_SMS_TEMPLATE_CREATE_COMMAND_TYPE: &str =
    "communication.sms_template.create";
/// Update-sms-template command type.
pub const COMMUNICATION_SMS_TEMPLATE_UPDATE_COMMAND_TYPE: &str =
    "communication.sms_template.update";
/// Enable-sms-template command type.
pub const COMMUNICATION_SMS_TEMPLATE_ENABLE_COMMAND_TYPE: &str =
    "communication.sms_template.enable";
/// Disable-sms-template command type.
pub const COMMUNICATION_SMS_TEMPLATE_DISABLE_COMMAND_TYPE: &str =
    "communication.sms_template.disable";
/// Delete-sms-template command type.
pub const COMMUNICATION_SMS_TEMPLATE_DELETE_COMMAND_TYPE: &str =
    "communication.sms_template.delete";

/// Configure-email-setting command type.
pub const COMMUNICATION_EMAIL_SETTING_CONFIGURE_COMMAND_TYPE: &str =
    "communication.email_setting.configure";
/// Activate-email-setting command type.
pub const COMMUNICATION_EMAIL_SETTING_ACTIVATE_COMMAND_TYPE: &str =
    "communication.email_setting.activate";
/// Delete-email-setting command type.
pub const COMMUNICATION_EMAIL_SETTING_DELETE_COMMAND_TYPE: &str =
    "communication.email_setting.delete";

/// Configure-sms-gateway command type.
pub const COMMUNICATION_SMS_GATEWAY_CONFIGURE_COMMAND_TYPE: &str =
    "communication.sms_gateway.configure";
/// Activate-sms-gateway command type.
pub const COMMUNICATION_SMS_GATEWAY_ACTIVATE_COMMAND_TYPE: &str =
    "communication.sms_gateway.activate";
/// Delete-sms-gateway command type.
pub const COMMUNICATION_SMS_GATEWAY_DELETE_COMMAND_TYPE: &str = "communication.sms_gateway.delete";

/// Create-custom-sms-setting command type.
pub const COMMUNICATION_CUSTOM_SMS_SETTING_CREATE_COMMAND_TYPE: &str =
    "communication.custom_sms_setting.create";
/// Update-custom-sms-setting command type.
pub const COMMUNICATION_CUSTOM_SMS_SETTING_UPDATE_COMMAND_TYPE: &str =
    "communication.custom_sms_setting.update";
/// Delete-custom-sms-setting command type.
pub const COMMUNICATION_CUSTOM_SMS_SETTING_DELETE_COMMAND_TYPE: &str =
    "communication.custom_sms_setting.delete";

/// Create-notification-setting command type.
pub const COMMUNICATION_NOTIFICATION_SETTING_CREATE_COMMAND_TYPE: &str =
    "communication.notification_setting.create";
/// Update-notification-setting command type.
pub const COMMUNICATION_NOTIFICATION_SETTING_UPDATE_COMMAND_TYPE: &str =
    "communication.notification_setting.update";
/// Delete-notification-setting command type.
pub const COMMUNICATION_NOTIFICATION_SETTING_DELETE_COMMAND_TYPE: &str =
    "communication.notification_setting.delete";

/// Configure-absent-notification command type.
pub const COMMUNICATION_ABSENT_NOTIFICATION_CONFIGURE_COMMAND_TYPE: &str =
    "communication.absent_notification.configure";
/// Enable-absent-notification command type.
pub const COMMUNICATION_ABSENT_NOTIFICATION_ENABLE_COMMAND_TYPE: &str =
    "communication.absent_notification.enable";
/// Disable-absent-notification command type.
pub const COMMUNICATION_ABSENT_NOTIFICATION_DISABLE_COMMAND_TYPE: &str =
    "communication.absent_notification.disable";
/// Delete-absent-notification command type.
pub const COMMUNICATION_ABSENT_NOTIFICATION_DELETE_COMMAND_TYPE: &str =
    "communication.absent_notification.delete";

/// Open-chat-conversation command type.
pub const COMMUNICATION_CHAT_CONVERSATION_OPEN_COMMAND_TYPE: &str =
    "communication.chat_conversation.open";
/// Close-chat-conversation command type.
pub const COMMUNICATION_CHAT_CONVERSATION_CLOSE_COMMAND_TYPE: &str =
    "communication.chat_conversation.close";
/// Send-chat-message command type.
pub const COMMUNICATION_CHAT_MESSAGE_SEND_COMMAND_TYPE: &str = "communication.chat_message.send";
/// Mark-chat-message-seen command type.
pub const COMMUNICATION_CHAT_MESSAGE_SEEN_COMMAND_TYPE: &str = "communication.chat_message.seen";
/// Delete-chat-message command type.
pub const COMMUNICATION_CHAT_MESSAGE_DELETE_COMMAND_TYPE: &str =
    "communication.chat_message.delete";

/// Create-chat-group command type.
pub const COMMUNICATION_CHAT_GROUP_CREATE_COMMAND_TYPE: &str = "communication.chat_group.create";
/// Update-chat-group command type.
pub const COMMUNICATION_CHAT_GROUP_UPDATE_COMMAND_TYPE: &str = "communication.chat_group.update";
/// Set-chat-group-read-only command type.
pub const COMMUNICATION_CHAT_GROUP_READ_ONLY_SET_COMMAND_TYPE: &str =
    "communication.chat_group.read_only_set";
/// Delete-chat-group command type.
pub const COMMUNICATION_CHAT_GROUP_DELETE_COMMAND_TYPE: &str = "communication.chat_group.delete";

/// Add-user-to-chat-group command type.
pub const COMMUNICATION_CHAT_GROUP_USER_ADD_COMMAND_TYPE: &str =
    "communication.chat_group_user.add";
/// Set-chat-group-user-role command type.
pub const COMMUNICATION_CHAT_GROUP_USER_SET_ROLE_COMMAND_TYPE: &str =
    "communication.chat_group_user.set_role";
/// Remove-user-from-chat-group command type.
pub const COMMUNICATION_CHAT_GROUP_USER_REMOVE_COMMAND_TYPE: &str =
    "communication.chat_group_user.remove";

/// Record-group-message-recipient command type.
pub const COMMUNICATION_CHAT_GROUP_MESSAGE_RECIPIENT_RECORD_COMMAND_TYPE: &str =
    "communication.chat_group_message_recipient.record";
/// Mark-group-message-read command type.
pub const COMMUNICATION_CHAT_GROUP_MESSAGE_RECIPIENT_MARK_READ_COMMAND_TYPE: &str =
    "communication.chat_group_message_recipient.mark_read";

/// Remove-group-message-for-user command type.
pub const COMMUNICATION_CHAT_GROUP_MESSAGE_REMOVE_REMOVE_COMMAND_TYPE: &str =
    "communication.chat_group_message_remove.remove";

/// Block-user command type.
pub const COMMUNICATION_CHAT_BLOCK_USER_BLOCK_COMMAND_TYPE: &str =
    "communication.chat_block_user.block";
/// Unblock-user command type.
pub const COMMUNICATION_CHAT_BLOCK_USER_UNBLOCK_COMMAND_TYPE: &str =
    "communication.chat_block_user.unblock";

/// Send-chat-invitation command type.
pub const COMMUNICATION_CHAT_INVITATION_SEND_COMMAND_TYPE: &str =
    "communication.chat_invitation.send";
/// Accept-chat-invitation command type.
pub const COMMUNICATION_CHAT_INVITATION_ACCEPT_COMMAND_TYPE: &str =
    "communication.chat_invitation.accept";
/// Reject-chat-invitation command type.
pub const COMMUNICATION_CHAT_INVITATION_REJECT_COMMAND_TYPE: &str =
    "communication.chat_invitation.reject";
/// Classify-chat-invitation command type.
pub const COMMUNICATION_CHAT_INVITATION_TYPE_CLASSIFY_COMMAND_TYPE: &str =
    "communication.chat_invitation_type.classify";

/// Set-chat-status command type.
pub const COMMUNICATION_CHAT_STATUS_SET_COMMAND_TYPE: &str = "communication.chat_status.set";

/// Create-send-message command type.
pub const COMMUNICATION_SEND_MESSAGE_CREATE_COMMAND_TYPE: &str =
    "communication.send_message.create";
/// Dispatch-send-message command type.
pub const COMMUNICATION_SEND_MESSAGE_DISPATCH_COMMAND_TYPE: &str =
    "communication.send_message.dispatch";
/// Cancel-send-message command type.
pub const COMMUNICATION_SEND_MESSAGE_CANCEL_COMMAND_TYPE: &str =
    "communication.send_message.cancel";

/// Receive-contact-message command type.
pub const COMMUNICATION_CONTACT_MESSAGE_RECEIVE_COMMAND_TYPE: &str =
    "communication.contact_message.receive";
/// Mark-contact-message-viewed command type.
pub const COMMUNICATION_CONTACT_MESSAGE_VIEW_COMMAND_TYPE: &str =
    "communication.contact_message.view";
/// Reply-to-contact-message command type.
pub const COMMUNICATION_CONTACT_MESSAGE_REPLY_COMMAND_TYPE: &str =
    "communication.contact_message.reply";

/// Create-speech-slider command type.
pub const COMMUNICATION_SPEECH_SLIDER_CREATE_COMMAND_TYPE: &str =
    "communication.speech_slider.create";
/// Update-speech-slider command type.
pub const COMMUNICATION_SPEECH_SLIDER_UPDATE_COMMAND_TYPE: &str =
    "communication.speech_slider.update";
/// Delete-speech-slider command type.
pub const COMMUNICATION_SPEECH_SLIDER_DELETE_COMMAND_TYPE: &str =
    "communication.speech_slider.delete";

/// Log-phone-call command type.
pub const COMMUNICATION_PHONE_CALL_LOG_LOG_COMMAND_TYPE: &str = "communication.phone_call_log.log";
/// Update-phone-call-follow-up command type.
pub const COMMUNICATION_PHONE_CALL_LOG_UPDATE_FOLLOW_UP_COMMAND_TYPE: &str =
    "communication.phone_call_log.update_follow_up";

// =============================================================================
// Notice commands
// =============================================================================

/// Create a notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateNoticeCommand {
    /// Tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The notice title.
    pub title: NoticeTitle,
    /// The notice body.
    pub body: NoticeBody,
    /// The notice date.
    pub notice_date: NaiveDate,
    /// The optional scheduled publish date (None = immediate).
    pub publish_on: Option<NaiveDate>,
    /// The audience descriptor.
    pub audience: AudienceDescriptor,
    /// The attachment file reference.
    pub attachment: Option<FileReference>,
}

impl CreateNoticeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTICE_CREATE_COMMAND_TYPE;
}

/// Update a notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateNoticeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notice id.
    pub notice_id: NoticeId,
    /// The new title.
    pub title: Option<NoticeTitle>,
    /// The new body.
    pub body: Option<NoticeBody>,
    /// The new publish-on date.
    pub publish_on: Option<NaiveDate>,
    /// The new audience descriptor.
    pub audience: Option<AudienceDescriptor>,
}

impl UpdateNoticeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTICE_UPDATE_COMMAND_TYPE;
}

/// Publish a notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishNoticeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notice id.
    pub notice_id: NoticeId,
    /// The optional publish-at timestamp.
    pub publish_at: Option<Timestamp>,
}

impl PublishNoticeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTICE_PUBLISH_COMMAND_TYPE;
}

/// Unpublish a notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnpublishNoticeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notice id.
    pub notice_id: NoticeId,
    /// The optional reason.
    pub reason: Option<String>,
}

impl UnpublishNoticeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTICE_UNPUBLISH_COMMAND_TYPE;
}

/// Delete a notice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteNoticeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notice id.
    pub notice_id: NoticeId,
}

impl DeleteNoticeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTICE_DELETE_COMMAND_TYPE;
}

// =============================================================================
// Complaint commands
// =============================================================================

/// Register a new complaint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegisterComplaintCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The optional complainant user id (None = anonymous).
    pub complaint_by: Option<UserId>,
    /// The complaint type id.
    pub complaint_type_id: ComplaintTypeId,
    /// The complaint source.
    pub complaint_source: ComplaintSource,
    /// The optional phone number.
    pub phone: Option<PhoneNumber>,
    /// The complaint date.
    pub date: NaiveDate,
    /// The complaint description.
    pub description: ComplaintDescription,
    /// The optional file reference.
    pub file: Option<FileReference>,
}

impl RegisterComplaintCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_REGISTER_COMMAND_TYPE;
}

/// Assign a complaint to a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignComplaintCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint id.
    pub complaint_id: ComplaintId,
    /// The assignee user id.
    pub assignee_user_id: UserId,
}

impl AssignComplaintCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_ASSIGN_COMMAND_TYPE;
}

/// Update the status of a complaint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateComplaintStatusCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint id.
    pub complaint_id: ComplaintId,
    /// The new status.
    pub status: ComplaintStatus,
    /// The optional note.
    pub note: Option<String>,
}

impl UpdateComplaintStatusCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_UPDATE_STATUS_COMMAND_TYPE;
}

/// Resolve a complaint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolveComplaintCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint id.
    pub complaint_id: ComplaintId,
    /// The action taken.
    pub action_taken: String,
    /// The optional note.
    pub note: Option<String>,
}

impl ResolveComplaintCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_RESOLVE_COMMAND_TYPE;
}

/// Add a note to a complaint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddComplaintNoteCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint id.
    pub complaint_id: ComplaintId,
    /// The note text.
    pub note: String,
}

impl AddComplaintNoteCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_ADD_NOTE_COMMAND_TYPE;
}

// =============================================================================
// ComplaintType commands
// =============================================================================

/// Create a new complaint type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateComplaintTypeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The name.
    pub name: String,
    /// The optional description.
    pub description: Option<String>,
}

impl CreateComplaintTypeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_TYPE_CREATE_COMMAND_TYPE;
}

/// Update a complaint type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateComplaintTypeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint type id.
    pub complaint_type_id: ComplaintTypeId,
    /// The new name.
    pub name: Option<String>,
    /// The new description.
    pub description: Option<String>,
}

impl UpdateComplaintTypeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_TYPE_UPDATE_COMMAND_TYPE;
}

/// Delete a complaint type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteComplaintTypeCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The complaint type id.
    pub complaint_type_id: ComplaintTypeId,
}

impl DeleteComplaintTypeCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_COMPLAINT_TYPE_DELETE_COMMAND_TYPE;
}

// =============================================================================
// Notification commands
// =============================================================================

/// Send a notification to a recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The recipient user id.
    pub recipient_user_id: UserId,
    /// The notification type.
    pub notification_type: NotificationType,
    /// The notification message.
    pub message: NotificationMessage,
    /// The optional URL.
    pub url: Option<Url>,
    /// The free-form data map.
    pub data: BTreeMap<String, String>,
    /// The dispatch channel.
    pub channel: Channel,
}

impl SendNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_SEND_COMMAND_TYPE;
}

/// Mark a notification as read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkNotificationReadCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notification id.
    pub notification_id: NotificationId,
}

impl MarkNotificationReadCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_READ_COMMAND_TYPE;
}

/// Withdraw a previously-sent notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithdrawNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notification id.
    pub notification_id: NotificationId,
    /// The reason for withdrawal.
    pub reason: String,
}

impl WithdrawNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_WITHDRAW_COMMAND_TYPE;
}

// =============================================================================
// EmailLog command
// =============================================================================

/// Log an email that was sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEmailSentCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The log entry title.
    pub title: String,
    /// The log entry description.
    pub description: String,
    /// The send date.
    pub send_date: NaiveDate,
    /// The mail driver used.
    pub send_through: MailDriver,
    /// The recipient email address.
    pub send_to: EmailAddress,
    /// The optional related message id.
    pub message_id: Option<MessageId>,
}

impl LogEmailSentCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_EMAIL_LOG_LOG_COMMAND_TYPE;
}

// =============================================================================
// SmsLog command
// =============================================================================

/// Log an SMS that was sent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogSmsSentCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The log entry title.
    pub title: String,
    /// The log entry description.
    pub description: String,
    /// The send date.
    pub send_date: NaiveDate,
    /// The SMS gateway used.
    pub send_through: SmsGatewayId,
    /// The recipient phone number.
    pub send_to: PhoneNumber,
    /// The optional related message id.
    pub message_id: Option<MessageId>,
}

impl LogSmsSentCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_LOG_LOG_COMMAND_TYPE;
}

// =============================================================================
// SmsTemplate commands
// =============================================================================

/// Create a new SMS template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSmsTemplateCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The dispatch channel.
    pub channel: Channel,
    /// The template purpose label.
    pub purpose: String,
    /// The template subject.
    pub subject: EmailSubject,
    /// The template body.
    pub body: TemplateBody,
    /// The module name.
    pub module: String,
    /// The declared template variables.
    pub variables: Vec<TemplateVariable>,
}

impl CreateSmsTemplateCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_TEMPLATE_CREATE_COMMAND_TYPE;
}

/// Update an SMS template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSmsTemplateCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS template id.
    pub sms_template_id: SmsTemplateId,
    /// The new subject.
    pub subject: Option<EmailSubject>,
    /// The new body.
    pub body: Option<TemplateBody>,
    /// The new declared variables.
    pub variables: Option<Vec<TemplateVariable>>,
}

impl UpdateSmsTemplateCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_TEMPLATE_UPDATE_COMMAND_TYPE;
}

/// Enable an SMS template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnableSmsTemplateCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS template id.
    pub sms_template_id: SmsTemplateId,
}

impl EnableSmsTemplateCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_TEMPLATE_ENABLE_COMMAND_TYPE;
}

/// Disable an SMS template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisableSmsTemplateCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS template id.
    pub sms_template_id: SmsTemplateId,
}

impl DisableSmsTemplateCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_TEMPLATE_DISABLE_COMMAND_TYPE;
}

/// Delete an SMS template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSmsTemplateCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS template id.
    pub sms_template_id: SmsTemplateId,
}

impl DeleteSmsTemplateCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_TEMPLATE_DELETE_COMMAND_TYPE;
}

// =============================================================================
// EmailSetting commands
// =============================================================================

/// Configure an email setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureEmailSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
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
}

impl ConfigureEmailSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_EMAIL_SETTING_CONFIGURE_COMMAND_TYPE;
}

/// Activate an email setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivateEmailSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The email setting id.
    pub email_setting_id: EmailSettingId,
}

impl ActivateEmailSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_EMAIL_SETTING_ACTIVATE_COMMAND_TYPE;
}

/// Delete an email setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteEmailSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The email setting id.
    pub email_setting_id: EmailSettingId,
}

impl DeleteEmailSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_EMAIL_SETTING_DELETE_COMMAND_TYPE;
}

// =============================================================================
// SmsGateway commands
// =============================================================================

/// Configure an SMS gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureSmsGatewayCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The gateway type.
    pub gateway_type: GatewayType,
    /// The gateway credentials (variant matches `gateway_type`).
    pub credentials: SmsGatewayCredentials,
}

impl ConfigureSmsGatewayCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_GATEWAY_CONFIGURE_COMMAND_TYPE;
}

/// Activate an SMS gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivateSmsGatewayCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS gateway id.
    pub sms_gateway_id: SmsGatewayId,
}

impl ActivateSmsGatewayCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_GATEWAY_ACTIVATE_COMMAND_TYPE;
}

/// Delete an SMS gateway.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSmsGatewayCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The SMS gateway id.
    pub sms_gateway_id: SmsGatewayId,
}

impl DeleteSmsGatewayCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SMS_GATEWAY_DELETE_COMMAND_TYPE;
}

// =============================================================================
// CustomSmsSetting commands
// =============================================================================

/// Create a custom SMS setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCustomSmsSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The gateway id.
    pub gateway_id: SmsGatewayId,
    /// The gateway name.
    pub gateway_name: GatewayName,
    /// Whether to set authentication.
    pub set_auth: Option<bool>,
    /// The gateway URL.
    pub gateway_url: Url,
    /// The request method.
    pub request_method: RequestMethod,
    /// The send-to parameter name.
    pub send_to_parameter_name: String,
    /// The message-to parameter name.
    pub message_to_parameter_name: String,
    /// The custom parameters.
    pub params: Vec<CustomSmsSettingParam>,
}

impl CreateCustomSmsSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CUSTOM_SMS_SETTING_CREATE_COMMAND_TYPE;
}

/// Update a custom SMS setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCustomSmsSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The custom SMS setting id.
    pub custom_sms_setting_id: CustomSmsSettingId,
    /// The new gateway name.
    pub gateway_name: Option<GatewayName>,
    /// The new set-auth flag.
    pub set_auth: Option<bool>,
    /// The new gateway URL.
    pub gateway_url: Option<Url>,
    /// The new request method.
    pub request_method: Option<RequestMethod>,
    /// The new custom parameters.
    pub params: Option<Vec<CustomSmsSettingParam>>,
}

impl UpdateCustomSmsSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CUSTOM_SMS_SETTING_UPDATE_COMMAND_TYPE;
}

/// Delete a custom SMS setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCustomSmsSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The custom SMS setting id.
    pub custom_sms_setting_id: CustomSmsSettingId,
}

impl DeleteCustomSmsSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CUSTOM_SMS_SETTING_DELETE_COMMAND_TYPE;
}

// =============================================================================
// NotificationSetting commands
// =============================================================================

/// Create a notification setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateNotificationSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The event name.
    pub event: String,
    /// The destination bitflag.
    pub destination: Destination,
    /// The recipient audience.
    pub recipient: NotificationSettingAudience,
    /// The subject.
    pub subject: EmailSubject,
    /// The template id.
    pub template_id: SmsTemplateId,
    /// The shortcode.
    pub shortcode: String,
}

impl CreateNotificationSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_SETTING_CREATE_COMMAND_TYPE;
}

/// Update a notification setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateNotificationSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notification setting id.
    pub notification_setting_id: NotificationSettingId,
    /// The new destination.
    pub destination: Option<Destination>,
    /// The new recipient audience.
    pub recipient: Option<NotificationSettingAudience>,
    /// The new subject.
    pub subject: Option<EmailSubject>,
    /// The new template id.
    pub template_id: Option<SmsTemplateId>,
    /// The new shortcode.
    pub shortcode: Option<String>,
}

impl UpdateNotificationSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_SETTING_UPDATE_COMMAND_TYPE;
}

/// Delete a notification setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteNotificationSettingCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The notification setting id.
    pub notification_setting_id: NotificationSettingId,
}

impl DeleteNotificationSettingCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_NOTIFICATION_SETTING_DELETE_COMMAND_TYPE;
}

// =============================================================================
// AbsentNotification commands
// =============================================================================

/// Configure the absent-notification dispatch window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigureAbsentNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The window start time.
    pub time_from: TimeOfDay,
    /// The window end time.
    pub time_to: TimeOfDay,
}

impl ConfigureAbsentNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_ABSENT_NOTIFICATION_CONFIGURE_COMMAND_TYPE;
}

/// Enable an absent-notification setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnableAbsentNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The absent-notification time-setup id.
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
}

impl EnableAbsentNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_ABSENT_NOTIFICATION_ENABLE_COMMAND_TYPE;
}

/// Disable an absent-notification setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisableAbsentNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The absent-notification time-setup id.
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
}

impl DisableAbsentNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_ABSENT_NOTIFICATION_DISABLE_COMMAND_TYPE;
}

/// Delete an absent-notification setup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteAbsentNotificationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The absent-notification time-setup id.
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
}

impl DeleteAbsentNotificationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_ABSENT_NOTIFICATION_DELETE_COMMAND_TYPE;
}

// =============================================================================
// Chat 1-to-1 commands
// =============================================================================

/// Open a chat conversation between two users.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenChatConversationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The sender user id.
    pub from_id: UserId,
    /// The recipient user id.
    pub to_id: UserId,
}

impl OpenChatConversationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_CONVERSATION_OPEN_COMMAND_TYPE;
}

/// Close a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloseChatConversationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat conversation id.
    pub chat_conversation_id: ChatConversationId,
}

impl CloseChatConversationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_CONVERSATION_CLOSE_COMMAND_TYPE;
}

/// Send a chat message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendChatMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The optional existing conversation id (None = create new).
    pub conversation_id: Option<ChatConversationId>,
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
}

impl SendChatMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_MESSAGE_SEND_COMMAND_TYPE;
}

/// Mark a chat message as seen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkChatMessageSeenCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat message id.
    pub chat_message_id: ChatMessageId,
}

impl MarkChatMessageSeenCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_MESSAGE_SEEN_COMMAND_TYPE;
}

/// Delete a chat message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteChatMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat message id.
    pub chat_message_id: ChatMessageId,
}

impl DeleteChatMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_MESSAGE_DELETE_COMMAND_TYPE;
}

// =============================================================================
// Chat group commands
// =============================================================================

/// Create a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateChatGroupCommand {
    /// Tenant context.
    pub tenant: TenantContext,
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
    /// The initial member user ids.
    pub initial_members: Vec<UserId>,
}

impl CreateChatGroupCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_CREATE_COMMAND_TYPE;
}

/// Update a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateChatGroupCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The new name.
    pub name: Option<String>,
    /// The new description.
    pub description: Option<String>,
    /// The new photo file reference.
    pub photo: Option<FileReference>,
}

impl UpdateChatGroupCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_UPDATE_COMMAND_TYPE;
}

/// Set a chat group's read-only flag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetChatGroupReadOnlyCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The read-only flag.
    pub read_only: bool,
}

impl SetChatGroupReadOnlyCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_READ_ONLY_SET_COMMAND_TYPE;
}

/// Delete a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteChatGroupCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
}

impl DeleteChatGroupCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_DELETE_COMMAND_TYPE;
}

// =============================================================================
// Chat group membership commands
// =============================================================================

/// Add a user to a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddUserToChatGroupCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The user id to add.
    pub user_id: UserId,
    /// The role to grant.
    pub role: ChatGroupRole,
}

impl AddUserToChatGroupCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_USER_ADD_COMMAND_TYPE;
}

/// Set the role of a user in a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetChatGroupUserRoleCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The user id.
    pub user_id: UserId,
    /// The new role.
    pub role: ChatGroupRole,
}

impl SetChatGroupUserRoleCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_USER_SET_ROLE_COMMAND_TYPE;
}

/// Remove a user from a chat group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveUserFromChatGroupCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The user id to remove.
    pub user_id: UserId,
}

impl RemoveUserFromChatGroupCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_GROUP_USER_REMOVE_COMMAND_TYPE;
}

// =============================================================================
// Chat group message recipient commands
// =============================================================================

/// Record that a group message reached a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordGroupMessageRecipientCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group id.
    pub chat_group_id: ChatGroupId,
    /// The recipient user id.
    pub user_id: UserId,
    /// The originating group chat message id.
    pub group_message_id: ChatMessageId,
}

impl RecordGroupMessageRecipientCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str =
        COMMUNICATION_CHAT_GROUP_MESSAGE_RECIPIENT_RECORD_COMMAND_TYPE;
}

/// Mark a group message as read by a recipient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkGroupMessageReadCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group message recipient id.
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
}

impl MarkGroupMessageReadCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str =
        COMMUNICATION_CHAT_GROUP_MESSAGE_RECIPIENT_MARK_READ_COMMAND_TYPE;
}

// =============================================================================
// Chat group message remove command
// =============================================================================

/// Remove a group message for a specific user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoveGroupMessageForUserCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat group message recipient id.
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    /// The user id for whom the message is removed.
    pub user_id: UserId,
}

impl RemoveGroupMessageForUserCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str =
        COMMUNICATION_CHAT_GROUP_MESSAGE_REMOVE_REMOVE_COMMAND_TYPE;
}

// =============================================================================
// Chat block commands
// =============================================================================

/// Block a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockUserCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The user id being blocked.
    pub block_to: UserId,
}

impl BlockUserCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_BLOCK_USER_BLOCK_COMMAND_TYPE;
}

/// Unblock a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnblockUserCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The user id being unblocked.
    pub block_to: UserId,
}

impl UnblockUserCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_BLOCK_USER_UNBLOCK_COMMAND_TYPE;
}

// =============================================================================
// Chat invitation commands
// =============================================================================

/// Send a chat invitation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SendChatInvitationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The recipient user id.
    pub to: UserId,
    /// The invitation type.
    pub invitation_type: ChatInvitationTypeEnum,
    /// The optional section id.
    pub section_id: Option<SectionId>,
    /// The optional class-teacher (staff) id.
    pub class_teacher_id: Option<StaffId>,
}

impl SendChatInvitationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_INVITATION_SEND_COMMAND_TYPE;
}

/// Accept a chat invitation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptChatInvitationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat invitation id.
    pub chat_invitation_id: ChatInvitationId,
}

impl AcceptChatInvitationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_INVITATION_ACCEPT_COMMAND_TYPE;
}

/// Reject a chat invitation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectChatInvitationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat invitation id.
    pub chat_invitation_id: ChatInvitationId,
}

impl RejectChatInvitationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_INVITATION_REJECT_COMMAND_TYPE;
}

/// Classify a chat invitation (assigns an invitation type to a pending invitation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassifyChatInvitationCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat invitation id being classified.
    pub invitation_id: ChatInvitationId,
    /// The invitation type.
    pub invitation_type: ChatInvitationTypeEnum,
    /// The optional section id.
    pub section_id: Option<SectionId>,
    /// The optional class-teacher (staff) id.
    pub class_teacher_id: Option<StaffId>,
}

impl ClassifyChatInvitationCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_INVITATION_TYPE_CLASSIFY_COMMAND_TYPE;
}

// =============================================================================
// Chat status command
// =============================================================================

/// Set the current user's chat status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetChatStatusCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The chat status.
    pub status: ChatStatus,
}

impl SetChatStatusCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CHAT_STATUS_SET_COMMAND_TYPE;
}

// =============================================================================
// SendMessage commands
// =============================================================================

/// Create a send-message (mass-broadcast) record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSendMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The message title.
    pub message_title: String,
    /// The message body.
    pub message_body: String,
    /// The notice date.
    pub notice_date: NaiveDate,
    /// The optional scheduled publish date.
    pub publish_on: Option<NaiveDate>,
    /// The audience descriptor.
    pub message_to: AudienceDescriptor,
}

impl CreateSendMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SEND_MESSAGE_CREATE_COMMAND_TYPE;
}

/// Dispatch a previously-created send-message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchSendMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The send message id.
    pub send_message_id: SendMessageId,
}

impl DispatchSendMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SEND_MESSAGE_DISPATCH_COMMAND_TYPE;
}

/// Cancel a send-message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelSendMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The send message id.
    pub send_message_id: SendMessageId,
    /// The optional reason.
    pub reason: Option<String>,
}

impl CancelSendMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SEND_MESSAGE_CANCEL_COMMAND_TYPE;
}

// =============================================================================
// ContactMessage commands
// =============================================================================

/// Receive a contact-form message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiveContactMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The sender name.
    pub name: PersonName,
    /// The optional phone number.
    pub phone: Option<PhoneNumber>,
    /// The optional email address.
    pub email: Option<EmailAddress>,
    /// The subject.
    pub subject: EmailSubject,
    /// The message body.
    pub message: String,
}

impl ReceiveContactMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CONTACT_MESSAGE_RECEIVE_COMMAND_TYPE;
}

/// Mark a contact message as viewed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkContactMessageViewedCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The contact message id.
    pub contact_message_id: ContactMessageId,
}

impl MarkContactMessageViewedCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CONTACT_MESSAGE_VIEW_COMMAND_TYPE;
}

/// Reply to a contact message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplyToContactMessageCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The contact message id.
    pub contact_message_id: ContactMessageId,
    /// The reply body.
    pub reply_body: String,
    /// The channel used to send the reply.
    pub reply_channel: Channel,
}

impl ReplyToContactMessageCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_CONTACT_MESSAGE_REPLY_COMMAND_TYPE;
}

// =============================================================================
// SpeechSlider commands
// =============================================================================

/// Create a speech-slider (leadership message) entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateSpeechSliderCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The person's name.
    pub name: PersonName,
    /// The person's designation.
    pub designation: String,
    /// The speech text.
    pub speech: SpeechText,
    /// The optional image file reference.
    pub image: Option<FileReference>,
}

impl CreateSpeechSliderCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SPEECH_SLIDER_CREATE_COMMAND_TYPE;
}

/// Update a speech-slider entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateSpeechSliderCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The speech slider id.
    pub speech_slider_id: SpeechSliderId,
    /// The new name.
    pub name: Option<PersonName>,
    /// The new designation.
    pub designation: Option<String>,
    /// The new speech text.
    pub speech: Option<SpeechText>,
    /// The new image file reference.
    pub image: Option<FileReference>,
}

impl UpdateSpeechSliderCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SPEECH_SLIDER_UPDATE_COMMAND_TYPE;
}

/// Delete a speech-slider entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteSpeechSliderCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The speech slider id.
    pub speech_slider_id: SpeechSliderId,
}

impl DeleteSpeechSliderCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_SPEECH_SLIDER_DELETE_COMMAND_TYPE;
}

// =============================================================================
// PhoneCallLog commands
// =============================================================================

/// Log a phone call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogPhoneCallCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The caller's name.
    pub name: PersonName,
    /// The caller's phone number.
    pub phone: PhoneNumber,
    /// The call date.
    pub date: NaiveDate,
    /// The call description.
    pub description: CallDescription,
    /// The optional next follow-up date.
    pub next_follow_up_date: Option<NaiveDate>,
    /// The optional call duration.
    pub call_duration: Option<CallDuration>,
    /// The call type.
    pub call_type: CallType,
}

impl LogPhoneCallCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = COMMUNICATION_PHONE_CALL_LOG_LOG_COMMAND_TYPE;
}

/// Update the follow-up date for a phone-call log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePhoneCallFollowUpCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The phone call log id.
    pub phone_call_log_id: PhoneCallLogId,
    /// The new next follow-up date.
    pub next_follow_up_date: NaiveDate,
}

impl UpdatePhoneCallFollowUpCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str =
        COMMUNICATION_PHONE_CALL_LOG_UPDATE_FOLLOW_UP_COMMAND_TYPE;
}
