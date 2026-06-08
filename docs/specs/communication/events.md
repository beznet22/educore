# Communication Domain — Events

Domain events describe facts that have already happened. They are
immutable, append-only records used for cross-domain integration, audit,
and event sourcing.

All events implement:

```rust
pub trait DomainEvent: Serialize + DeserializeOwned + Send + Sync {
    const TYPE: &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn school_id(&self) -> SchoolId;
    fn occurred_at(&self) -> Timestamp;
}
```

The event envelope wraps the event with metadata:

```rust
pub struct EventEnvelope<E> {
    pub event_id: EventId,
    pub event_type: &'static str,
    pub school_id: SchoolId,
    pub aggregate_id: Uuid,
    pub aggregate_type: &'static str,
    pub actor_id: UserId,
    pub correlation_id: CorrelationId,
    pub causation_id: Option<EventId>,
    pub occurred_at: Timestamp,
    pub payload: E,
}
```

## Notice Lifecycle

```rust
pub struct NoticeCreated {
    pub notice_id: NoticeId,
    pub title: NoticeTitle,
    pub notice_date: NaiveDate,
    pub publish_on: Option<NaiveDate>,
    pub audience: AudienceDescriptor,
}

pub struct NoticeUpdated { pub notice_id: NoticeId, pub changes: Vec<&'static str> }
pub struct NoticePublished { pub notice_id: NoticeId, pub published_at: Timestamp }
pub struct NoticeUnpublished { pub notice_id: NoticeId, pub reason: Option<String> }
pub struct NoticeDeleted { pub notice_id: NoticeId }
```

**Subscribers:**
- The notification adapter fans out `NoticePublished` to the audience
  and writes `Notification` aggregates.

## Complaint Lifecycle

```rust
pub struct ComplaintRegistered {
    pub complaint_id: ComplaintId,
    pub complaint_type_id: ComplaintTypeId,
    pub complaint_source: ComplaintSource,
    pub date: NaiveDate,
}

pub struct ComplaintAssigned { pub complaint_id: ComplaintId, pub assignee_user_id: UserId }
pub struct ComplaintStatusChanged { pub complaint_id: ComplaintId, pub from: ComplaintStatus, pub to: ComplaintStatus }
pub struct ComplaintResolved { pub complaint_id: ComplaintId, pub action_taken: String, pub resolved_at: Timestamp }
pub struct ComplaintNoteAdded { pub complaint_id: ComplaintId, pub note: String, pub author: UserId }
pub struct ComplaintTypeCreated { pub complaint_type_id: ComplaintTypeId, pub name: String }
pub struct ComplaintTypeUpdated { pub complaint_type_id: ComplaintTypeId, pub changes: Vec<&'static str> }
pub struct ComplaintTypeDeleted { pub complaint_type_id: ComplaintTypeId }
```

## Notification Lifecycle

```rust
pub struct NotificationSent {
    pub notification_id: NotificationId,
    pub recipient_user_id: UserId,
    pub notification_type: NotificationType,
    pub channel: Channel,
}

pub struct NotificationRead { pub notification_id: NotificationId, pub read_at: Timestamp }
pub struct NotificationWithdrawn { pub notification_id: NotificationId, pub reason: String }
```

## Email & SMS Logs

```rust
pub struct EmailLogged {
    pub email_log_id: EmailLogId,
    pub title: String,
    pub send_through: MailDriver,
    pub send_to: EmailAddress,
    pub send_date: NaiveDate,
    pub source_message_id: Option<MessageId>,
}

pub struct SmsLogged {
    pub sms_log_id: SmsLogId,
    pub title: String,
    pub send_through: SmsGatewayId,
    pub send_to: PhoneNumber,
    pub send_date: NaiveDate,
    pub source_message_id: Option<MessageId>,
}
```

## Templates

```rust
pub struct SmsTemplateCreated {
    pub sms_template_id: SmsTemplateId,
    pub channel: Channel,
    pub purpose: TemplateKey,
}

pub struct SmsTemplateUpdated { pub sms_template_id: SmsTemplateId, pub changes: Vec<&'static str> }
pub struct SmsTemplateEnabled { pub sms_template_id: SmsTemplateId }
pub struct SmsTemplateDisabled { pub sms_template_id: SmsTemplateId }
pub struct SmsTemplateDeleted { pub sms_template_id: SmsTemplateId }
```

## Email Engine Configuration

```rust
pub struct EmailSettingConfigured {
    pub email_setting_id: EmailSettingId,
    pub mail_driver: MailDriver,
    pub mail_host: String,
}

pub struct EmailSettingActivated { pub email_setting_id: EmailSettingId, pub previous_id: Option<EmailSettingId> }
pub struct EmailSettingDeleted { pub email_setting_id: EmailSettingId }
```

## SMS Gateway Configuration

```rust
pub struct SmsGatewayConfigured {
    pub sms_gateway_id: SmsGatewayId,
    pub gateway_type: GatewayType,
}

pub struct SmsGatewayActivated { pub sms_gateway_id: SmsGatewayId, pub gateway_type: GatewayType, pub previous_id: Option<SmsGatewayId> }
pub struct SmsGatewayDeleted { pub sms_gateway_id: SmsGatewayId }
```

## Notification Routing

```rust
pub struct NotificationSettingCreated {
    pub notification_setting_id: NotificationSettingId,
    pub event: String,
    pub destination: Destination,
}

pub struct NotificationSettingUpdated { pub notification_setting_id: NotificationSettingId, pub changes: Vec<&'static str> }
pub struct NotificationSettingDeleted { pub notification_setting_id: NotificationSettingId }
```

## Absent Notification

```rust
pub struct AbsentNotificationScheduled {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub time_from: TimeOfDay,
    pub time_to: TimeOfDay,
}

pub struct AbsentNotificationEnabled { pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId }
pub struct AbsentNotificationDisabled { pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId }
pub struct AbsentNotificationDeleted { pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId }
pub struct AbsentNotificationSent {
    pub absent_notification_time_setup_id: AbsentNotificationTimeSetupId,
    pub student_id: StudentId,
    pub channel: Channel,
    pub template_id: SmsTemplateId,
}
```

**Subscribers:**
- The notification adapter performs the actual send and writes an
  `EmailLogged` or `SmsLogged` event.

## Chat — One-to-One

```rust
pub struct ChatConversationOpened { pub chat_conversation_id: ChatConversationId, pub from_id: UserId, pub to_id: UserId }
pub struct ChatConversationClosed { pub chat_conversation_id: ChatConversationId }
pub struct ChatMessageSent {
    pub chat_message_id: ChatMessageId,
    pub chat_conversation_id: ChatConversationId,
    pub from_id: UserId,
    pub to_id: UserId,
    pub message_type: MessageType,
}

pub struct ChatMessageSeen { pub chat_message_id: ChatMessageId, pub seen_by: UserId, pub seen_at: Timestamp }
pub struct ChatMessageDeleted { pub chat_message_id: ChatMessageId, pub deleted_by: UserId }
```

## Chat — Groups

```rust
pub struct ChatGroupCreated {
    pub chat_group_id: ChatGroupId,
    pub name: String,
    pub privacy: ChatGroupPrivacy,
    pub group_type: ChatGroupType,
    pub created_by: UserId,
}

pub struct ChatGroupUpdated { pub chat_group_id: ChatGroupId, pub changes: Vec<&'static str> }
pub struct ChatGroupReadOnlySet { pub chat_group_id: ChatGroupId, pub read_only: bool }
pub struct ChatGroupDeleted { pub chat_group_id: ChatGroupId }
pub struct ChatGroupUserAdded { pub chat_group_id: ChatGroupId, pub user_id: UserId, pub role: ChatGroupRole }
pub struct ChatGroupUserRoleChanged { pub chat_group_id: ChatGroupId, pub user_id: UserId, pub from: ChatGroupRole, pub to: ChatGroupRole }
pub struct ChatGroupUserRemoved { pub chat_group_id: ChatGroupId, pub user_id: UserId, pub removed_by: UserId }
```

## Chat — Group Message Delivery

```rust
pub struct GroupMessageRecipientRecorded {
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    pub chat_group_id: ChatGroupId,
    pub user_id: UserId,
}

pub struct GroupMessageMarkedRead {
    pub chat_group_message_recipient_id: ChatGroupMessageRecipientId,
    pub read_at: Timestamp,
}

pub struct GroupMessageRemovedForUser {
    pub chat_group_message_remove_id: ChatGroupMessageRemoveId,
    pub user_id: UserId,
}
```

## Chat — Block, Invitation, Status

```rust
pub struct UserBlocked { pub block_by: UserId, pub block_to: UserId, pub blocked_at: Timestamp }
pub struct UserUnblocked { pub block_by: UserId, pub block_to: UserId }
pub struct ChatInvitationSent {
    pub chat_invitation_id: ChatInvitationId,
    pub from: UserId,
    pub to: UserId,
    pub invitation_type: ChatInvitationTypeEnum,
}

pub struct ChatInvitationAccepted { pub chat_invitation_id: ChatInvitationId, pub accepted_by: UserId }
pub struct ChatInvitationRejected { pub chat_invitation_id: ChatInvitationId, pub rejected_by: UserId }
pub struct ChatInvitationClassified { pub chat_invitation_type_id: ChatInvitationTypeId, pub invitation_id: ChatInvitationId, pub type: ChatInvitationTypeEnum }
pub struct ChatStatusSet { pub user_id: UserId, pub status: ChatStatus, pub set_at: Timestamp }
```

## Send Message (Bulk)

```rust
pub struct SendMessageCreated {
    pub send_message_id: SendMessageId,
    pub audience: AudienceDescriptor,
    pub publish_on: Option<NaiveDate>,
}

pub struct SendMessageDispatched {
    pub send_message_id: SendMessageId,
    pub recipient_count: u32,
    pub dispatched_at: Timestamp,
}

pub struct SendMessageCancelled { pub send_message_id: SendMessageId, pub reason: Option<String> }
```

**Subscribers:**
- The notification adapter writes a `Notification` per recipient and
  may produce `EmailLogged` / `SmsLogged` events.

## Contact Message

```rust
pub struct ContactMessageReceived {
    pub contact_message_id: ContactMessageId,
    pub name: PersonName,
    pub email: Option<EmailAddress>,
    pub phone: Option<PhoneNumber>,
    pub subject: String,
}

pub struct ContactMessageViewed { pub contact_message_id: ContactMessageId, pub viewed_by: UserId }
pub struct ContactMessageReplied { pub contact_message_id: ContactMessageId, pub reply_channel: Channel, pub replied_by: UserId }
```

## Speech Slider

```rust
pub struct SpeechSliderCreated { pub speech_slider_id: SpeechSliderId, pub name: PersonName, pub designation: String }
pub struct SpeechSliderUpdated { pub speech_slider_id: SpeechSliderId, pub changes: Vec<&'static str> }
pub struct SpeechSliderDeleted { pub speech_slider_id: SpeechSliderId }
```

## Phone Call

```rust
pub struct PhoneCallLogged {
    pub phone_call_log_id: PhoneCallLogId,
    pub name: PersonName,
    pub phone: PhoneNumber,
    pub call_type: CallType,
    pub date: NaiveDate,
}

pub struct PhoneCallFollowUpUpdated { pub phone_call_log_id: PhoneCallLogId, pub next_follow_up_date: NaiveDate }
```

## Custom SMS Gateway

```rust
pub struct CustomSmsSettingCreated {
    pub custom_sms_setting_id: CustomSmsSettingId,
    pub gateway_id: SmsGatewayId,
    pub gateway_url: Url,
    pub request_method: RequestMethod,
}

pub struct CustomSmsSettingUpdated { pub custom_sms_setting_id: CustomSmsSettingId, pub changes: Vec<&'static str> }
pub struct CustomSmsSettingDeleted { pub custom_sms_setting_id: CustomSmsSettingId }
```
