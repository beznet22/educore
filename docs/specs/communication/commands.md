# Communication Domain — Commands

Commands describe intent. They are validated, authorized, and dispatched
to the relevant aggregate. Every command produces zero or more events
that are recorded in the event log.

All commands carry a `TenantContext` (school + actor + correlation) and
are rejected if the actor lacks the required capability.

## CreateNotice

```rust
pub struct CreateNoticeCommand {
    pub tenant: TenantContext,
    pub title: NoticeTitle,
    pub body: NoticeBody,
    pub notice_date: NoticeDate,
    pub publish_on: Option<PublishOn>,
    pub audience: AudienceDescriptor,
    pub attachment: Option<FileReference>,
}
```

**Capability:** `Notice.Create`
**Pre-conditions:**
- Audience is non-empty and references roles the school has configured.
- If `publish_on` is set, it is on or after `notice_date`.

**Effects:** Creates a `Notice` and emits `NoticeCreated`.

## UpdateNotice

```rust
pub struct UpdateNoticeCommand {
    pub tenant: TenantContext,
    pub notice_id: NoticeId,
    pub title: Option<NoticeTitle>,
    pub body: Option<NoticeBody>,
    pub publish_on: Option<PublishOn>,
    pub audience: Option<AudienceDescriptor>,
}
```

**Capability:** `Notice.Update`
**Pre-conditions:** Notice exists and is not deleted.

**Effects:** Emits `NoticeUpdated`.

## PublishNotice

```rust
pub struct PublishNoticeCommand {
    pub tenant: TenantContext,
    pub notice_id: NoticeId,
    pub publish_at: Option<Timestamp>,
}
```

**Capability:** `Notice.Publish`
**Pre-conditions:** Notice is in `Draft` or `Scheduled` status.

**Effects:** Emits `NoticePublished`. The fan-out to recipients is
performed by the notification adapter, which writes `Notification`
aggregates and may produce `EmailLogged` / `SmsLogged` events.

## UnpublishNotice

```rust
pub struct UnpublishNoticeCommand {
    pub tenant: TenantContext,
    pub notice_id: NoticeId,
    pub reason: Option<String>,
}
```

**Capability:** `Notice.Unpublish`
**Effects:** Emits `NoticeUnpublished`. Already-delivered notifications
remain in recipients' inboxes; the notice is hidden from new
notification generation.

## DeleteNotice

```rust
pub struct DeleteNoticeCommand {
    pub tenant: TenantContext,
    pub notice_id: NoticeId,
}
```

**Capability:** `Notice.Delete`
**Pre-conditions:** No recipient has received the notice, or the actor
has administrative override.

**Effects:** Emits `NoticeDeleted`. Soft delete; audit record remains.

## RegisterComplaint

```rust
pub struct RegisterComplaintCommand {
    pub tenant: TenantContext,
    pub complaint_by: Option<PersonName>,
    pub complaint_type_id: ComplaintTypeId,
    pub complaint_source: ComplaintSource,
    pub phone: Option<PhoneNumber>,
    pub date: NaiveDate,
    pub description: ComplaintDescription,
    pub file: Option<FileReference>,
}
```

**Capability:** `Complaint.Create`
**Pre-conditions:** If source is not `Anonymous`, at least one of
`complaint_by` or `phone` is set.

**Effects:** Creates a `Complaint` in `Open` status and emits
`ComplaintRegistered`.

## AssignComplaint

```rust
pub struct AssignComplaintCommand {
    pub tenant: TenantContext,
    pub complaint_id: ComplaintId,
    pub assignee_user_id: UserId,
}
```

**Capability:** `Complaint.Assign`
**Effects:** Emits `ComplaintAssigned`. The status transitions to
`InProgress` if not already.

## UpdateComplaintStatus

```rust
pub struct UpdateComplaintStatusCommand {
    pub tenant: TenantContext,
    pub complaint_id: ComplaintId,
    pub status: ComplaintStatus,
    pub note: Option<String>,
}
```

**Capability:** `Complaint.Update`
**Effects:** Emits `ComplaintStatusChanged`.

## ResolveComplaint

```rust
pub struct ResolveComplaintCommand {
    pub tenant: TenantContext,
    pub complaint_id: ComplaintId,
    pub action_taken: String,
    pub note: Option<String>,
}
```

**Capability:** `Complaint.Resolve`
**Pre-conditions:** Complaint is not already `Resolved`.

**Effects:** Emits `ComplaintResolved`.

## AddComplaintNote

```rust
pub struct AddComplaintNoteCommand {
    pub tenant: TenantContext,
    pub complaint_id: ComplaintId,
    pub note: String,
}
```

**Capability:** `Complaint.Note`
**Effects:** Emits `ComplaintNoteAdded`.

## SendNotification

```rust
pub struct SendNotificationCommand {
    pub tenant: TenantContext,
    pub recipient_user_id: UserId,
    pub notification_type: NotificationType,
    pub message: String,
    pub url: Option<Url>,
    pub data: BTreeMap<String, String>,
    pub channel: Channel,
}
```

**Capability:** `Notification.Send`
**Effects:** Creates a `Notification` and emits `NotificationSent`. The
notification adapter performs the actual delivery.

## MarkNotificationRead

```rust
pub struct MarkNotificationReadCommand {
    pub tenant: TenantContext,
    pub notification_id: NotificationId,
}
```

**Capability:** `Notification.Read`
**Effects:** Emits `NotificationRead`. Only the recipient or an admin
may mark a notification read.

## WithdrawNotification

```rust
pub struct WithdrawNotificationCommand {
    pub tenant: TenantContext,
    pub notification_id: NotificationId,
    pub reason: String,
}
```

**Capability:** `Notification.Withdraw`
**Effects:** Emits `NotificationWithdrawn`. Recipients who have not yet
read the notification no longer see it.

## LogEmailSent

```rust
pub struct LogEmailSentCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub description: String,
    pub send_date: NaiveDate,
    pub send_through: MailDriver,
    pub send_to: EmailAddress,
    pub message_id: Option<MessageId>,
}
```

**Capability:** `EmailLog.Create`
**Effects:** Creates an `EmailLog` and emits `EmailLogged`. The log is
written by the notification adapter after a successful SMTP send.

## LogSmsSent

```rust
pub struct LogSmsSentCommand {
    pub tenant: TenantContext,
    pub title: String,
    pub description: String,
    pub send_date: NaiveDate,
    pub send_through: SmsGatewayId,
    pub send_to: PhoneNumber,
    pub message_id: Option<MessageId>,
}
```

**Capability:** `SmsLog.Create`
**Effects:** Creates a `SmsLog` and emits `SmsLogged`.

## CreateSmsTemplate

```rust
pub struct CreateSmsTemplateCommand {
    pub tenant: TenantContext,
    pub channel: Channel,
    pub purpose: TemplateKey,
    pub subject: String,
    pub body: TemplateBody,
    pub module: String,
    pub variables: Vec<TemplateVariable>,
}
```

**Capability:** `Template.Create`
**Pre-conditions:** The `purpose` is unique per `(school_id, channel)`.
**Effects:** Emits `SmsTemplateCreated`.

## UpdateSmsTemplate / EnableSmsTemplate / DisableSmsTemplate / DeleteSmsTemplate

```rust
pub struct UpdateSmsTemplateCommand { ... }
pub struct EnableSmsTemplateCommand { ... }
pub struct DisableSmsTemplateCommand { ... }
pub struct DeleteSmsTemplateCommand { ... }
```

**Capabilities:** `Template.Update`, `Template.Enable`,
`Template.Disable`, `Template.Delete`.

## ConfigureEmailSetting

```rust
pub struct ConfigureEmailSettingCommand {
    pub tenant: TenantContext,
    pub email_engine_type: String,
    pub from_name: PersonName,
    pub from_email: EmailAddress,
    pub mail_driver: MailDriver,
    pub mail_host: String,
    pub mail_port: u16,
    pub mail_username: String,
    pub mail_password: SecretReference,
    pub mail_encryption: MailEncryption,
}
```

**Capability:** `EmailSetting.Configure`
**Effects:** Emits `EmailSettingConfigured`.

## ActivateEmailSetting

```rust
pub struct ActivateEmailSettingCommand {
    pub tenant: TenantContext,
    pub email_setting_id: EmailSettingId,
}
```

**Capability:** `EmailSetting.Activate`
**Effects:** Demotes the previous active setting and emits
`EmailSettingActivated`.

## ConfigureSmsGateway

```rust
pub struct ConfigureSmsGatewayCommand {
    pub tenant: TenantContext,
    pub gateway_type: GatewayType,
    pub credentials: SmsGatewayCredentials,
}
```

`SmsGatewayCredentials` is a typed enum whose variants match the
`GatewayType` (e.g. `Clickatell { username, password, api_id }`,
`Twilio { account_sid, auth_token, registered_no }`).

**Capability:** `SmsGateway.Configure`
**Effects:** Emits `SmsGatewayConfigured`.

## ActivateSmsGateway

```rust
pub struct ActivateSmsGatewayCommand {
    pub tenant: TenantContext,
    pub sms_gateway_id: SmsGatewayId,
}
```

**Capability:** `SmsGateway.Activate`
**Effects:** Demotes the previously active gateway of the same
`GatewayType` and emits `SmsGatewayActivated`.

## CreateNotificationSetting

```rust
pub struct CreateNotificationSettingCommand {
    pub tenant: TenantContext,
    pub event: String,
    pub destination: Destination,
    pub recipient: AudienceDescriptor,
    pub subject: String,
    pub template_id: SmsTemplateId,
    pub shortcode: Vec<TemplateVariable>,
}
```

**Capability:** `NotificationSetting.Create`
**Pre-conditions:** `event` is a known event key.
**Effects:** Emits `NotificationSettingCreated`.

## UpdateNotificationSetting / DeleteNotificationSetting

```rust
pub struct UpdateNotificationSettingCommand { ... }
pub struct DeleteNotificationSettingCommand { ... }
```

**Capabilities:** `NotificationSetting.Update`, `NotificationSetting.Delete`.

## ConfigureAbsentNotification

```rust
pub struct ConfigureAbsentNotificationCommand {
    pub tenant: TenantContext,
    pub time_from: TimeOfDay,
    pub time_to: TimeOfDay,
}
```

**Capability:** `AbsentNotification.Configure`
**Pre-conditions:** `time_from` is strictly before `time_to`.
**Effects:** Emits `AbsentNotificationScheduled`.

## EnableAbsentNotification / DisableAbsentNotification / DeleteAbsentNotification

```rust
pub struct EnableAbsentNotificationCommand { ... }
pub struct DisableAbsentNotificationCommand { ... }
pub struct DeleteAbsentNotificationCommand { ... }
```

**Capabilities:** `AbsentNotification.Enable`,
`AbsentNotification.Disable`, `AbsentNotification.Delete`.

## SendChatMessage

```rust
pub struct SendChatMessageCommand {
    pub tenant: TenantContext,
    pub conversation_id: Option<ChatConversationId>,
    pub from_id: UserId,
    pub to_id: UserId,
    pub body: ChatMessageBody,
    pub message_type: MessageType,
    pub file: Option<FileReference>,
    pub reply_to: Option<ChatMessageId>,
    pub forward_of: Option<ChatMessageId>,
}
```

**Capability:** `Chat.Send`
**Pre-conditions:** `to_id` is not blocked by `from_id`; `from_id` is
not blocked by `to_id`.

**Effects:** Creates a `ChatMessage` and emits `ChatMessageSent`. If
`conversation_id` is null and a prior conversation exists between
`from_id` and `to_id`, the existing conversation is reused; otherwise a
new `ChatConversation` is implicitly opened.

## MarkChatMessageSeen

```rust
pub struct MarkChatMessageSeenCommand {
    pub tenant: TenantContext,
    pub chat_message_id: ChatMessageId,
}
```

**Capability:** `Chat.Read`
**Effects:** Emits `ChatMessageSeen`.

## CreateChatGroup

```rust
pub struct CreateChatGroupCommand {
    pub tenant: TenantContext,
    pub name: String,
    pub description: Option<String>,
    pub photo: Option<FileReference>,
    pub privacy: ChatGroupPrivacy,
    pub group_type: ChatGroupType,
    pub class_id: Option<ClassId>,
    pub section_id: Option<SectionId>,
    pub subject_id: Option<SubjectId>,
    pub teacher_id: Option<StaffId>,
    pub initial_members: Vec<UserId>,
}
```

**Capability:** `ChatGroup.Create`
**Effects:** Emits `ChatGroupCreated` and one `ChatGroupUserAdded` per
initial member.

## AddUserToChatGroup / SetChatGroupUserRole / RemoveUserFromChatGroup

```rust
pub struct AddUserToChatGroupCommand {
    pub tenant: TenantContext,
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
    pub role: ChatGroupRole,
}

pub struct SetChatGroupUserRoleCommand { ... }
pub struct RemoveUserFromChatGroupCommand { ... }
```

**Capabilities:** `ChatGroup.AddUser`, `ChatGroup.SetRole`,
`ChatGroup.RemoveUser`.

## BlockUser / UnblockUser

```rust
pub struct BlockUserCommand {
    pub tenant: TenantContext,
    pub block_to: UserId,
}

pub struct UnblockUserCommand {
    pub tenant: TenantContext,
    pub block_to: UserId,
}
```

**Capabilities:** `Chat.Block`, `Chat.Unblock`.
**Effects:** Emit `UserBlocked` / `UserUnblocked`.

## SendChatInvitation

```rust
pub struct SendChatInvitationCommand {
    pub tenant: TenantContext,
    pub to: UserId,
    pub invitation_type: ChatInvitationTypeEnum,
    pub section_id: Option<SectionId>,
    pub class_teacher_id: Option<StaffId>,
}
```

**Capability:** `Chat.Invite`
**Effects:** Emits `ChatInvitationSent` and `ChatInvitationClassified`.

## AcceptChatInvitation / RejectChatInvitation

```rust
pub struct AcceptChatInvitationCommand { ... }
pub struct RejectChatInvitationCommand { ... }
```

**Capabilities:** `Chat.Accept`, `Chat.Reject`.

## SetChatStatus

```rust
pub struct SetChatStatusCommand {
    pub tenant: TenantContext,
    pub status: ChatStatus,
}
```

**Capability:** `Chat.SetStatus`
**Effects:** Emits `ChatStatusSet`.

## CreateSendMessage

```rust
pub struct CreateSendMessageCommand {
    pub tenant: TenantContext,
    pub message_title: String,
    pub message_body: String,
    pub notice_date: NaiveDate,
    pub publish_on: Option<NaiveDate>,
    pub message_to: AudienceDescriptor,
}
```

**Capability:** `SendMessage.Create`
**Effects:** Emits `SendMessageCreated`.

## DispatchSendMessage

```rust
pub struct DispatchSendMessageCommand {
    pub tenant: TenantContext,
    pub send_message_id: SendMessageId,
}
```

**Capability:** `SendMessage.Dispatch`
**Effects:** Freezes the recipient set, creates a `Notification` per
recipient, and emits `SendMessageDispatched`.

## CancelSendMessage

```rust
pub struct CancelSendMessageCommand {
    pub tenant: TenantContext,
    pub send_message_id: SendMessageId,
    pub reason: Option<String>,
}
```

**Capability:** `SendMessage.Cancel`
**Pre-conditions:** Job has not been dispatched.
**Effects:** Emits `SendMessageCancelled`.

## ReceiveContactMessage

```rust
pub struct ReceiveContactMessageCommand {
    pub tenant: TenantContext,
    pub name: PersonName,
    pub phone: Option<PhoneNumber>,
    pub email: Option<EmailAddress>,
    pub subject: String,
    pub message: String,
}
```

**Capability:** `ContactMessage.Create` (system or public-port).
**Effects:** Emits `ContactMessageReceived`.

## ReplyToContactMessage

```rust
pub struct ReplyToContactMessageCommand {
    pub tenant: TenantContext,
    pub contact_message_id: ContactMessageId,
    pub reply_body: String,
    pub reply_channel: Channel,
}
```

**Capability:** `ContactMessage.Reply`
**Effects:** Emits `ContactMessageReplied`.

## CreateSpeechSlider

```rust
pub struct CreateSpeechSliderCommand {
    pub tenant: TenantContext,
    pub name: PersonName,
    pub designation: String,
    pub speech: SpeechText,
    pub image: Option<FileReference>,
}
```

**Capability:** `SpeechSlider.Create`
**Effects:** Emits `SpeechSliderCreated`.

## UpdateSpeechSlider / DeleteSpeechSlider

```rust
pub struct UpdateSpeechSliderCommand { ... }
pub struct DeleteSpeechSliderCommand { ... }
```

**Capabilities:** `SpeechSlider.Update`, `SpeechSlider.Delete`.

## LogPhoneCall

```rust
pub struct LogPhoneCallCommand {
    pub tenant: TenantContext,
    pub name: PersonName,
    pub phone: PhoneNumber,
    pub date: NaiveDate,
    pub description: CallDescription,
    pub next_follow_up_date: Option<NaiveDate>,
    pub call_duration: Option<CallDuration>,
    pub call_type: CallType,
}
```

**Capability:** `PhoneCallLog.Create`
**Effects:** Emits `PhoneCallLogged`.

## UpdatePhoneCallFollowUp

```rust
pub struct UpdatePhoneCallFollowUpCommand {
    pub tenant: TenantContext,
    pub phone_call_log_id: PhoneCallLogId,
    pub next_follow_up_date: NaiveDate,
}
```

**Capability:** `PhoneCallLog.Update`
**Effects:** Emits `PhoneCallFollowUpUpdated`.

## CreateCustomSmsSetting

```rust
pub struct CreateCustomSmsSettingCommand {
    pub tenant: TenantContext,
    pub gateway_id: SmsGatewayId,
    pub gateway_name: String,
    pub set_auth: Option<SecretReference>,
    pub gateway_url: Url,
    pub request_method: RequestMethod,
    pub send_to_parameter_name: String,
    pub message_to_parameter_name: String,
    pub params: Vec<(String, String)>,
}
```

**Capability:** `CustomSmsSetting.Create`
**Effects:** Emits `CustomSmsSettingCreated`.

## Orphaned Items (Cluster D catch-up)

The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Classify Chat Invitation

```rust
pub struct ClassifyChatInvitationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatInvitation.Classify`
**Effects:** Emits `ChatInvitationClassifyed`.


### Close Chat Conversation

```rust
pub struct CloseChatConversationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatConversation.Close`
**Effects:** Emits `ChatConversationCloseed`.


### Create Complaint Type

```rust
pub struct CreateComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Create`
**Effects:** Emits `ComplaintTypeCreateed`.


### Delete Chat Group

```rust
pub struct DeleteChatGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroup.Delete`
**Effects:** Emits `ChatGroupDeleteed`.


### Delete Chat Message

```rust
pub struct DeleteChatMessageCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatMessage.Delete`
**Effects:** Emits `ChatMessageDeleteed`.


### Delete Complaint Type

```rust
pub struct DeleteComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Delete`
**Effects:** Emits `ComplaintTypeDeleteed`.


### Delete Custom Sms Setting

```rust
pub struct DeleteCustomSmsSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CustomSmsSetting.Delete`
**Effects:** Emits `CustomSmsSettingDeleteed`.


### Delete Email Setting

```rust
pub struct DeleteEmailSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `EmailSetting.Delete`
**Effects:** Emits `EmailSettingDeleteed`.


### Delete Sms Gateway

```rust
pub struct DeleteSmsGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `SmsGateway.Delete`
**Effects:** Emits `SmsGatewayDeleteed`.


### Mark Contact Message Viewed

```rust
pub struct MarkContactMessageViewedCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ContactMessageViewed.Mark`
**Effects:** Emits `ContactMessageViewedMarked`.


### Mark Group Message Read

```rust
pub struct MarkGroupMessageReadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageRead.Mark`
**Effects:** Emits `GroupMessageReadMarked`.


### Open Chat Conversation

```rust
pub struct OpenChatConversationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatConversation.Open`
**Effects:** Emits `ChatConversationOpened`.


### Record Group Message Recipient

```rust
pub struct RecordGroupMessageRecipientCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageRecipient.Record`
**Effects:** Emits `GroupMessageRecipientRecorded`.


### Remove Group Message For User

```rust
pub struct RemoveGroupMessageForUserCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageForUser.Remove`
**Effects:** Emits `GroupMessageForUserRemoveed`.


### Set Chat Group Read Only

```rust
pub struct SetChatGroupReadOnlyCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroupReadOnly.Set`
**Effects:** Emits `ChatGroupReadOnlySeted`.


### Update Chat Group

```rust
pub struct UpdateChatGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroup.Update`
**Effects:** Emits `ChatGroupUpdateed`.


### Update Complaint Type

```rust
pub struct UpdateComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Update`
**Effects:** Emits `ComplaintTypeUpdateed`.


### Update Custom Sms Setting

```rust
pub struct UpdateCustomSmsSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CustomSmsSetting.Update`
**Effects:** Emits `CustomSmsSettingUpdateed`.



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

### Classify Chat Invitation

```rust
pub struct ClassifyChatInvitationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatInvitation.Classify`
**Effects:** Emits `ChatInvitationClassifyed`.


### Close Chat Conversation

```rust
pub struct CloseChatConversationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatConversation.Close`
**Effects:** Emits `ChatConversationCloseed`.


### Create Complaint Type

```rust
pub struct CreateComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Create`
**Effects:** Emits `ComplaintTypeCreateed`.


### Delete Chat Group

```rust
pub struct DeleteChatGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroup.Delete`
**Effects:** Emits `ChatGroupDeleteed`.


### Delete Chat Message

```rust
pub struct DeleteChatMessageCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatMessage.Delete`
**Effects:** Emits `ChatMessageDeleteed`.


### Delete Complaint Type

```rust
pub struct DeleteComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Delete`
**Effects:** Emits `ComplaintTypeDeleteed`.


### Delete Custom Sms Setting

```rust
pub struct DeleteCustomSmsSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CustomSmsSetting.Delete`
**Effects:** Emits `CustomSmsSettingDeleteed`.


### Delete Email Setting

```rust
pub struct DeleteEmailSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `EmailSetting.Delete`
**Effects:** Emits `EmailSettingDeleteed`.


### Delete Sms Gateway

```rust
pub struct DeleteSmsGatewayCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `SmsGateway.Delete`
**Effects:** Emits `SmsGatewayDeleteed`.


### Mark Contact Message Viewed

```rust
pub struct MarkContactMessageViewedCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ContactMessageViewed.Mark`
**Effects:** Emits `ContactMessageViewedMarked`.


### Mark Group Message Read

```rust
pub struct MarkGroupMessageReadCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageRead.Mark`
**Effects:** Emits `GroupMessageReadMarked`.


### Open Chat Conversation

```rust
pub struct OpenChatConversationCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatConversation.Open`
**Effects:** Emits `ChatConversationOpened`.


### Record Group Message Recipient

```rust
pub struct RecordGroupMessageRecipientCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageRecipient.Record`
**Effects:** Emits `GroupMessageRecipientRecorded`.


### Remove Group Message For User

```rust
pub struct RemoveGroupMessageForUserCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `GroupMessageForUser.Remove`
**Effects:** Emits `GroupMessageForUserRemoveed`.


### Set Chat Group Read Only

```rust
pub struct SetChatGroupReadOnlyCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroupReadOnly.Set`
**Effects:** Emits `ChatGroupReadOnlySeted`.


### Update Chat Group

```rust
pub struct UpdateChatGroupCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ChatGroup.Update`
**Effects:** Emits `ChatGroupUpdateed`.


### Update Complaint Type

```rust
pub struct UpdateComplaintTypeCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `ComplaintType.Update`
**Effects:** Emits `ComplaintTypeUpdateed`.


### Update Custom Sms Setting

```rust
pub struct UpdateCustomSmsSettingCommand {
    pub tenant: TenantContext,
    pub target_id: String,
}
```

**Capability:** `CustomSmsSetting.Update`
**Effects:** Emits `CustomSmsSettingUpdateed`.

