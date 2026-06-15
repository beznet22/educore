//! # educore-communication
//!
//!  Notices, complaints, chat, email/SMS dispatch, notifications, contact forms, speech sliders, phone-call logs.
//!
//! This crate is a member of the Educore workspace. See
//! `docs/architecture.md` and the domain spec in
//! `docs/specs/communication/` for behavioral details.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(unused_imports)]

pub mod value_objects;

mod aggregate;
pub mod commands;
mod entities;
mod errors;
pub mod events;
pub mod query;
mod repository;
pub mod services;

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-communication";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Prelude: re-export the engine-wide types + the headline surface.
// Mirrors educore-library::prelude.
#[allow(missing_docs)]
pub mod prelude {
    pub use chrono::NaiveDate;
    pub use educore_core::clock::{Clock, IdGenerator, SystemClock, SystemIdGen};
    pub use educore_core::error::{DomainError, Result};
    pub use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
    pub use educore_core::tenant::TenantContext;
    pub use educore_core::value_objects::Timestamp;
    pub use educore_events::domain_event::DomainEvent;
    pub use educore_events::envelope::EventEnvelope;
    pub use educore_rbac::value_objects::Capability;

    // Cross-crate id re-exports (from educore_academic, educore_hr)
    pub use educore_academic::{AcademicYearId, ClassId, SectionId, StudentId, SubjectId};
    pub use educore_hr::value_objects::{RoleId, StaffId};

    // 26 headline aggregate roots (from crate::aggregate)
    pub use crate::aggregate::{
        AbsentNotificationTimeSetup, ChatBlockUser, ChatConversation, ChatGroup,
        ChatGroupMessageRecipient, ChatGroupMessageRemove, ChatGroupUser, ChatInvitation,
        ChatInvitationType, ChatMessage, ChatStatusRecord, Complaint, ComplaintType, ContactMessage,
        CustomSmsSetting, EmailLog, EmailSetting, Notification, NotificationSetting, Notice,
        PhoneCallLog, SendMessage, SpeechSlider, SmsGateway, SmsLog, SmsTemplate,
    };

    // 15 child entities
    pub use crate::entities::{
        AbsentNotificationDispatch, ChatConversationLastRead, ChatGroupAvatar, ChatGroupMessage,
        ComplaintNote, ContactMessageReply, EmailSettingSecret, NoticeAttachment,
        NotificationDeliveryAttempt, SendMessageRecipient, SmsGatewayCredential,
        CustomSmsSettingParam, NoticeAudience, NotificationSettingAudience, SmsTemplateVariable,
    };

    // 73 typed events (alphabetised)
    pub use crate::events::{
        AbsentNotificationDeleted, AbsentNotificationDisabled, AbsentNotificationEnabled,
        AbsentNotificationScheduled, AbsentNotificationSent, ChatConversationClosed,
        ChatConversationOpened, ChatGroupCreated, ChatGroupDeleted, ChatGroupReadOnlySet,
        ChatGroupUpdated, ChatGroupUserAdded, ChatGroupUserRemoved, ChatGroupUserRoleChanged,
        ChatInvitationAccepted, ChatInvitationClassified, ChatInvitationRejected,
        ChatInvitationSent, ChatMessageDeleted, ChatMessageSeen, ChatMessageSent,
        ChatStatusSet, ComplaintAssigned, ComplaintNoteAdded, ComplaintRegistered,
        ComplaintResolved, ComplaintStatusChanged, ComplaintTypeCreated, ComplaintTypeDeleted,
        ComplaintTypeUpdated, ContactMessageReceived, ContactMessageReplied, ContactMessageViewed,
        CustomSmsSettingCreated, CustomSmsSettingDeleted, CustomSmsSettingUpdated,
        EmailLogged, EmailSettingActivated, EmailSettingConfigured, EmailSettingDeleted,
        GroupMessageMarkedRead, GroupMessageRecipientRecorded, GroupMessageRemovedForUser,
        NoticeCreated, NoticeDeleted, NoticePublished, NoticeUnpublished, NoticeUpdated,
        NotificationRead, NotificationSent, NotificationWithdrawn, PhoneCallFollowUpUpdated,
        PhoneCallLogged, SendMessageCancelled, SendMessageCreated, SendMessageDispatched,
        SmsLogged, SmsTemplateCreated, SmsTemplateDeleted, SmsTemplateDisabled,
        SmsTemplateEnabled, SmsTemplateUpdated, SpeechSliderCreated, SpeechSliderDeleted,
        SpeechSliderUpdated, UserBlocked, UserUnblocked,
    };

    // 26 query stubs
    pub use crate::query::{
        AbsentNotificationTimeSetupQuery, ChatBlockUserQuery, ChatConversationQuery,
        ChatGroupMessageRecipientQuery, ChatGroupMessageRemoveQuery, ChatGroupQuery,
        ChatGroupUserQuery, ChatInvitationQuery, ChatInvitationTypeQuery, ChatMessageQuery,
        ChatStatusQuery, ComplaintQuery, ComplaintTypeQuery, ContactMessageQuery,
        CustomSmsSettingQuery, EmailLogQuery, EmailSettingQuery, NotificationQuery,
        NotificationSettingQuery, NoticeQuery, PhoneCallLogQuery, SendMessageQuery,
        SmsGatewayQuery, SmsLogQuery, SmsTemplateQuery, SpeechSliderQuery,
    };

    // 26 repository ports
    pub use crate::repository::{
        AbsentNotificationTimeSetupRepository, ChatBlockUserRepository, ChatConversationRepository,
        ChatGroupMessageRecipientRepository, ChatGroupMessageRemoveRepository,
        ChatGroupRepository, ChatGroupUserRepository, ChatInvitationRepository,
        ChatInvitationTypeRepository, ChatMessageRepository, ChatStatusRepository,
        ComplaintRepository, ComplaintTypeRepository, ContactMessageRepository,
        CustomSmsSettingRepository, EmailLogRepository, EmailSettingRepository,
        NotificationRepository, NotificationSettingRepository, NoticeRepository,
        PhoneCallLogRepository, SendMessageRepository, SmsGatewayRepository, SmsLogRepository,
        SmsTemplateRepository, SpeechSliderRepository,
    };

    // 7 headline service fns
    pub use crate::services::{
        mark_as_read, notify_user, send_chat_message, send_complaint_message,
        send_email_message, send_notice_message, send_sms_message,
    };

    // 70 pure factory service fns (re-export all from crate::services)
    pub use crate::services::{
        accept_chat_invitation, activate_email_setting, activate_sms_gateway,
        add_user_to_chat_group, block_user, cancel_send_message, classify_chat_invitation,
        close_chat_conversation, configure_absent_notification, configure_email_setting,
        configure_sms_gateway, create_chat_group, create_complaint_type,
        create_custom_sms_setting, create_notification_setting, create_send_message,
        create_sms_template, create_speech_slider, delete_chat_group, delete_chat_message,
        delete_complaint_type, delete_custom_sms_setting,
        delete_email_setting, delete_notification_setting,
        delete_sms_gateway, delete_sms_template, delete_speech_slider,
        disable_absent_notification, disable_sms_template, dispatch_send_message,
        enable_absent_notification, enable_sms_template, log_email_sent, log_phone_call,
        log_sms_sent, mark_chat_message_seen, mark_contact_message_viewed,
        mark_group_message_read, mark_notification_read, open_chat_conversation,
        publish_notice, receive_contact_message, record_group_message_recipient,
        register_complaint, reject_chat_invitation, remove_user_from_chat_group,
        reply_to_contact_message, send_chat_invitation, send_notification,
        set_chat_group_read_only, set_chat_group_user_role, set_chat_status,
        unblock_user, unpublish_notice, update_chat_group, update_complaint_status,
        update_custom_sms_setting, update_notification_setting, update_phone_call_follow_up,
        update_speech_slider, withdraw_notification,
    };

    // 7 service structs
    pub use crate::services::{
        AbsentNotificationService, ActiveRecipients, ChatInvitePolicy, ChatService,
        ComplaintService, NoticesPublishedInRange, NotificationService, SmsDispatchPolicy,
        TemplateService,
    };

    // 72 typed commands (re-export all)
    pub use crate::commands::{
        AcceptChatInvitationCommand, ActivateEmailSettingCommand, ActivateSmsGatewayCommand,
        AddUserToChatGroupCommand, BlockUserCommand, CancelSendMessageCommand,
        ClassifyChatInvitationCommand, CloseChatConversationCommand,
        ConfigureAbsentNotificationCommand, ConfigureEmailSettingCommand,
        ConfigureSmsGatewayCommand, CreateChatGroupCommand, CreateComplaintTypeCommand,
        CreateCustomSmsSettingCommand,
        CreateNotificationSettingCommand, CreateNoticeCommand, CreateSendMessageCommand,
        CreateSmsTemplateCommand, CreateSpeechSliderCommand, DeleteChatGroupCommand,
        DeleteChatMessageCommand, DeleteComplaintTypeCommand,
        DeleteCustomSmsSettingCommand, DeleteEmailSettingCommand,
        DeleteNotificationSettingCommand,
        DeleteSmsGatewayCommand, DeleteSmsTemplateCommand,
        DeleteSpeechSliderCommand, DisableAbsentNotificationCommand,
        DisableSmsTemplateCommand, DispatchSendMessageCommand, EnableAbsentNotificationCommand,
        EnableSmsTemplateCommand, LogEmailSentCommand, LogPhoneCallCommand, LogSmsSentCommand,
        MarkChatMessageSeenCommand, MarkContactMessageViewedCommand, MarkGroupMessageReadCommand,
        MarkNotificationReadCommand, OpenChatConversationCommand, PublishNoticeCommand,
        ReceiveContactMessageCommand, RecordGroupMessageRecipientCommand,
        RegisterComplaintCommand, RejectChatInvitationCommand, RemoveUserFromChatGroupCommand,
        ReplyToContactMessageCommand, ResolveComplaintCommand, SendChatInvitationCommand,
        SendChatMessageCommand, SendNotificationCommand, SetChatGroupReadOnlyCommand,
        SetChatGroupUserRoleCommand, SetChatStatusCommand, UnblockUserCommand,
        UnpublishNoticeCommand, UpdateChatGroupCommand, UpdateComplaintStatusCommand,
        UpdateComplaintTypeCommand, UpdateCustomSmsSettingCommand,
        UpdateNotificationSettingCommand, UpdateNoticeCommand,
        UpdatePhoneCallFollowUpCommand, UpdateSpeechSliderCommand, WithdrawNotificationCommand,
    };

    // 41 typed ids + 32 VOs + 27 enums + 4 embedded (re-export all from value_objects)
    pub use crate::value_objects::{
        // 37 typed ids (26 root + 11 child)
        AbsentNotificationDispatchId, AbsentNotificationTimeSetupId, ChatBlockUserId,
        ChatConversationId, ChatConversationLastReadId, ChatGroupAvatarId, ChatGroupId,
        ChatGroupMessageId, ChatGroupMessageRecipientId, ChatGroupMessageRemoveId,
        ChatGroupUserId, ChatInvitationId, ChatInvitationTypeId, ChatMessageId,
        ChatStatusId, ComplaintId, ComplaintNoteId, ComplaintTypeId, ContactMessageId,
        ContactMessageReplyId, CustomSmsSettingId, EmailLogId, EmailSettingId,
        EmailSettingSecretId, NoticeAttachmentId, NoticeId, NotificationDeliveryAttemptId,
        NotificationId, NotificationSettingId, PhoneCallLogId, SendMessageId,
        SendMessageRecipientId, SmsGatewayCredentialId, SmsGatewayId, SmsLogId,
        SmsTemplateId, SpeechSliderId,
        // 32 VOs (all re-exported)
        AudienceDescriptor, CallDescription, CallDuration, ComplaintDescription, ComplaintSource,
        Destination, DispatchDate, EmailAddress, EmailSubject, FileReference, GatewayName,
        MailDriverName, MessageId, NoticeBody, NoticeDate, NoticeTitle, NotificationMessage,
        NotificationRoute, PersonName, PhoneNumber, PublishOn, RenderedBody, RenderWarning,
        SecretReference, Slug, SpeechText, StarRating, SmsGatewayCredentials, TemplateBody,
        TemplateKey, TemplateVariable, TimeOfDay, TimeWindow, Url,
        // 27 enums
        AbsentNotificationStatus, CallType, Channel, ChatGroupPrivacy, ChatGroupRole,
        ChatGroupType, ChatInvitationStatus, ChatInvitationTypeEnum, ChatMessageStatus,
        ChatStatus, ComplaintAction, ComplaintStatus, ContactMessageReplyStatus,
        ContactMessageViewStatus, GatewayType, MailDriver, MailEncryption, MessageType,
        NoticeStatus, NoticeType, NotificationStatus, NotificationType, RequestMethod,
        SendMessageStatus, SmsTemplateStatus,
    };

    // Errors
    pub use crate::errors::CommunicationError;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-communication");
        assert!(!PACKAGE_VERSION.is_empty());
    }

    #[test]
    fn prelude_exports_expected_symbols() {
        // Smoke test: every headline aggregate and event is
        // reachable through the prelude. The compiler enforces
        // the names.
        let _: Option<Notice> = None;
        let _: Option<Complaint> = None;
        let _: Option<ComplaintType> = None;
        let _: Option<Notification> = None;
        let _: Option<EmailLog> = None;
        let _: Option<SmsLog> = None;
        let _: Option<SmsTemplate> = None;
        let _: Option<EmailSetting> = None;
        let _: Option<SmsGateway> = None;
        let _: Option<NotificationSetting> = None;
        let _: Option<AbsentNotificationTimeSetup> = None;
        let _: Option<ChatMessage> = None;
        let _: Option<ChatConversation> = None;
        let _: Option<ChatGroup> = None;
        let _: Option<ChatGroupUser> = None;
        let _: Option<ChatGroupMessageRecipient> = None;
        let _: Option<ChatGroupMessageRemove> = None;
        let _: Option<ChatBlockUser> = None;
        let _: Option<ChatInvitation> = None;
        let _: Option<ChatInvitationType> = None;
        let _: Option<ChatStatusRecord> = None;
        let _: Option<SendMessage> = None;
        let _: Option<ContactMessage> = None;
        let _: Option<SpeechSlider> = None;
        let _: Option<PhoneCallLog> = None;
        let _: Option<CustomSmsSetting> = None;
        let _: Option<ComplaintType> = None;

        let _: Option<NoticeAttachment> = None;
        let _: Option<ComplaintNote> = None;
        let _: Option<NotificationDeliveryAttempt> = None;
        let _: Option<ChatGroupAvatar> = None;
        let _: Option<ChatGroupMessage> = None;
        let _: Option<ChatConversationLastRead> = None;
        let _: Option<SendMessageRecipient> = None;
        let _: Option<EmailSettingSecret> = None;
        let _: Option<SmsGatewayCredential> = None;
        let _: Option<AbsentNotificationDispatch> = None;
        let _: Option<ContactMessageReply> = None;
    }
}
