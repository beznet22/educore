# Communication Domain — Value Objects

Value objects are immutable, validated at construction, and have no
identity. They are compared by value.

## Identifiers

All identifiers in the communication domain are typed and tenant-scoped.
The generic `Id<S, T>` wrapper carries the `SchoolId` of the owning
school and the local id (`Uuid`).

| Identifier                          | Backing Type                | Source Column                          |
| ----------------------------------- | --------------------------- | -------------------------------------- |
| `NoticeId`                          | `Id<Notice>`                | `communication_notice_boards.id`                  |
| `ComplaintId`                       | `Id<Complaint>`             | `communication_complaints.id`                     |
| `ComplaintTypeId`                   | `Id<ComplaintType>`         | derived from `communication_complaints.complaint_type` |
| `NotificationId`                    | `Id<Notification>`          | `communication_notifications.id`                  |
| `EmailLogId`                        | `Id<EmailLog>`              | `communication_email_sms_logs.id`                 |
| `SmsLogId`                          | `Id<SmsLog>`                | `communication_email_sms_logs.id`                 |
| `SmsTemplateId`                     | `Id<SmsTemplate>`           | `sms_templates.id`                     |
| `EmailSettingId`                    | `Id<EmailSetting>`          | `communication_email_settings.id`                 |
| `SmsGatewayId`                      | `Id<SmsGateway>`            | `communication_sms_gateways.id`                   |
| `NotificationSettingId`             | `Id<NotificationSetting>`   | `communication_notification_settings.id`          |
| `AbsentNotificationTimeSetupId`     | `Id<AbsentNotificationTimeSetup>` | `absent_notification_time_setups.id` |
| `ChatMessageId`                     | `Id<ChatMessage>`           | `chat_conversations.id`                |
| `ChatConversationId`                | `Id<ChatConversation>`      | `chat_conversations.id`                |
| `ChatGroupId`                       | `Id<ChatGroup>`             | `chat_groups.id`                       |
| `ChatGroupUserId`                   | `Id<ChatGroupUser>`         | `chat_group_users.id`                  |
| `ChatGroupMessageRecipientId`       | `Id<ChatGroupMessageRecipient>` | `chat_group_message_recipients.id` |
| `ChatGroupMessageRemoveId`          | `Id<ChatGroupMessageRemove>`| `chat_group_message_removes.id`        |
| `ChatBlockUserId`                   | `Id<ChatBlockUser>`         | `chat_block_users.id`                  |
| `ChatInvitationId`                  | `Id<ChatInvitation>`        | `chat_invitations.id`                  |
| `ChatInvitationTypeId`              | `Id<ChatInvitationType>`    | `chat_invitation_types.id`             |
| `ChatStatusId`                      | `Id<ChatStatus>`            | `chat_statuses.id`                     |
| `SendMessageId`                     | `Id<SendMessage>`           | `communication_send_messages.id`                  |
| `ContactMessageId`                  | `Id<ContactMessage>`        | `communication_contact_messages.id`               |
| `SpeechSliderId`                    | `Id<SpeechSlider>`          | `speech_sliders.id`                    |
| `PhoneCallLogId`                    | `Id<PhoneCallLog>`          | `communication_phone_call_logs.id`                |
| `CustomSmsSettingId`                | `Id<CustomSmsSetting>`      | `custom_sms_settings.id`               |

## Names and Free Text

| Type                 | Constraints                                                     |
| -------------------- | --------------------------------------------------------------- |
| `NoticeTitle`        | 1..200 chars                                                    |
| `NoticeBody`         | 1..5000 chars                                                   |
| `ComplaintDescription` | 1..5000 chars                                                 |
| `SpeechText`         | 1..5000 chars                                                   |
| `ChatMessageBody`    | 1..10000 chars                                                  |
| `TemplateBody`       | 1..65535 chars                                                  |
| `EmailSubject`       | 1..200 chars                                                    |
| `CallDescription`    | 1..5000 chars                                                   |

## Channels and Destinations

| Type                 | Values / Constraints                                                |
| -------------------- | ------------------------------------------------------------------- |
| `Channel`            | `Email`, `Sms`, `Web`, `App`, `Push`                                |
| `Destination`        | `Email`, `Sms`, `Web`, `App` (combinable flags)                     |
| `MessageType`        | `Text`, `Image`, `Pdf`, `Document`, `Voice`                         |
| `CallType`           | `Incoming`, `Outgoing`, `Missed`                                    |
| `GatewayType`        | `Clickatell`, `Twilio`, `Msg91`, `Textlocal`, `AfricaTalking`, `Custom` |
| `MailEncryption`     | `None`, `TLS`, `STARTTLS`                                           |
| `MailDriver`         | `Smtp`, `Sendmail`, `Mailgun`, `Ses`, `Postmark`                    |
| `RequestMethod`      | `Get`, `Post`                                                       |

## Status Enums

| Type                       | Values                                                              |
| -------------------------- | ------------------------------------------------------------------- |
| `NoticeStatus`             | `Draft`, `Scheduled`, `Published`, `Unpublished`                    |
| `ComplaintStatus`          | `Open`, `InProgress`, `Resolved`                                    |
| `ComplaintSource`          | `WalkIn`, `Phone`, `Email`, `Web`, `Other`, `Anonymous`             |
| `NotificationStatus`       | `Pending`, `Dispatched`, `Delivered`, `Failed`, `Read`, `Withdrawn`|
| `SmsTemplateStatus`        | `Enabled`, `Disabled`                                               |
| `AbsentNotificationStatus` | `Enabled`, `Disabled`                                               |
| `ChatGroupPrivacy`         | `Public`, `Private`, `Class`                                        |
| `ChatGroupType`            | `Open`, `Closed`                                                    |
| `ChatGroupRole`            | `Member`, `Admin`                                                   |
| `ChatStatus`               | `Inactive`, `Active`, `Away`, `Busy`                                |
| `ChatInvitationStatus`     | `Pending`, `Connected`, `Blocked`                                   |
| `ChatInvitationTypeEnum`   | `OneToOne`, `Group`, `ClassTeacher`                                 |
| `ChatMessageStatus`        | `Unread`, `Seen`                                                    |
| `SendMessageStatus`        | `Draft`, `Dispatched`, `Cancelled`, `Completed`                     |
| `ContactMessageViewStatus` | `Unviewed`, `Viewed`                                                |
| `ContactMessageReplyStatus`| `Unreplied`, `Replied`                                              |

## Routing and Recipients

| Type                  | Notes                                                              |
| --------------------- | ------------------------------------------------------------------ |
| `RoleId`              | From `educore-rbac`                                                |
| `ClassId`             | From `educore-academic`                                            |
| `SectionId`           | From `educore-academic`                                            |
| `StudentId`           | From `educore-academic`                                            |
| `StaffId`             | From `educore-hr`                                                  |
| `GuardianId`          | From `educore-academic`                                            |
| `UserId`              | From `educore-platform` — the actor of a chat message              |
| `AudienceDescriptor`  | `Vec<RoleId>` OR `ClassId`+`SectionId` OR `Vec<UserId>` OR `All`   |
| `NotificationRoute`   | `(event: String, destination: Destination, recipient: AudienceDescriptor)` |

## Time and Schedule

| Type                       | Notes                                                       |
| -------------------------- | ----------------------------------------------------------- |
| `TimeWindow`               | `time_from: NaiveTime`, `time_to: NaiveTime`                |
| `TimeOfDay`                | 24h clock string `HH:MM`                                    |
| `CallDuration`             | 1..100 chars, format `HH:MM:SS`                             |
| `DispatchDate`             | `NaiveDate`                                                 |
| `PublishOn`                | `NaiveDate` — may be null for "immediate"                   |
| `NoticeDate`               | `NaiveDate`                                                 |

## Files and Secrets

| Type                 | Notes                                                          |
| -------------------- | -------------------------------------------------------------- |
| `FileReference`      | From `educore-platform` (port-owned)                          |
| `SecretReference`    | Opaque reference to a secret in the secret-store port         |
| `Url`                | Validated URL, max 2048 chars                                  |
| `EmailAddress`       | RFC 5322 with length cap 200                                   |
| `PhoneNumber`        | E.164 format preferred; alternative national formats accepted  |
| `PersonName`         | 1..200 chars                                                   |
| `Slug`               | URL-safe slug, 1..200 chars, `[a-z0-9-]`                       |
| `StarRating`         | `u8` in `1..5`                                                 |

## Tenant Bindings

| Type                 | Notes                                                          |
| -------------------- | -------------------------------------------------------------- |
| `SchoolId`           | From `educore-platform`                                        |
| `UserId`             | From `educore-platform`                                        |
| `TenantContext`      | `(SchoolId, UserId, ...)` from `educore-platform`             |
| `AcademicYearId`     | From `educore-academic`                                        |
| `CorrelationId`      | From `educore-events`                                          |

## Variable Substitution

| Type                  | Notes                                                          |
| --------------------- | -------------------------------------------------------------- |
| `TemplateVariable`    | `(name: String, description: String)`                          |
| `RenderedBody`        | The final body after substitution; immutable                   |
| `TemplateKey`         | 1..100 chars; stable across versions                           |

## Validation Rules

All value objects implement `Validate` and refuse construction when
validation fails:

```rust
pub trait Validate {
    fn validate(&self) -> Result<(), ValueError>;
}
```

Construction is the only entry point:

```rust
let body = NoticeBody::new("School closed tomorrow")?;
```

Parsing returns `Result<NoticeBody, ValueError>`. There are no setters
that bypass validation.

## Additional Identifiers

| Identifier | Backing Type | Notes |
| ---------- | ------------ | ----- |
| `AbsentNotificationDispatchId` | `Id<AbsentNotificationDispatch>` | A `AbsentNotificationDispatch` identifier |
| `ChatConversationLastReadId` | `Id<ChatConversationLastRead>` | A `ChatConversationLastRead` identifier |
| `ChatGroupAvatarId` | `Id<ChatGroupAvatar>` | A `ChatGroupAvatar` identifier |
| `ChatGroupMessageId` | `Id<ChatGroupMessage>` | A `ChatGroupMessage` identifier |
| `ComplaintNoteId` | `Id<ComplaintNote>` | A `ComplaintNote` identifier |
| `ContactMessageReplyId` | `Id<ContactMessageReply>` | A `ContactMessageReply` identifier |
| `EmailSettingSecretId` | `Id<EmailSettingSecret>` | A `EmailSettingSecret` identifier |
| `MessageId` | `Id<Message>` | A `Message` identifier |
| `NoticeAttachmentId` | `Id<NoticeAttachment>` | A `NoticeAttachment` identifier |
| `NotificationDeliveryAttemptId` | `Id<NotificationDeliveryAttempt>` | A `NotificationDeliveryAttempt` identifier |
| `SendMessageRecipientId` | `Id<SendMessageRecipient>` | A `SendMessageRecipient` identifier |
| `SmsGatewayCredentialId` | `Id<SmsGatewayCredential>` | A `SmsGatewayCredential` identifier |

## Additional Enums

| Type | Values |
| ---- | ------ |
| `ComplaintAction` | (status/classification enum, see code) |
| `CustomSmsSettingParam` | (status/classification enum, see code) |
| `GatewayName` | (status/classification enum, see code) |
| `MailDriverName` | (status/classification enum, see code) |
| `NoticeAudience` | (status/classification enum, see code) |
| `NoticeType` | (status/classification enum, see code) |
| `NotificationMessage` | (status/classification enum, see code) |
| `NotificationType` | (status/classification enum, see code) |
| `RenderWarning` | (status/classification enum, see code) |
| `SmsGatewayCredentials` | (status/classification enum, see code) |
| `SmsTemplateVariable` | (status/classification enum, see code) |

## Additional Identifiers

| Identifier | Backing Type | Notes |
| ---------- | ------------ | ----- |
| `AbsentNotificationDispatchId` | `Id<AbsentNotificationDispatch>` | A `AbsentNotificationDispatch` identifier |
| `ChatConversationLastReadId` | `Id<ChatConversationLastRead>` | A `ChatConversationLastRead` identifier |
| `ChatGroupAvatarId` | `Id<ChatGroupAvatar>` | A `ChatGroupAvatar` identifier |
| `ChatGroupMessageId` | `Id<ChatGroupMessage>` | A `ChatGroupMessage` identifier |
| `ComplaintNoteId` | `Id<ComplaintNote>` | A `ComplaintNote` identifier |
| `ContactMessageReplyId` | `Id<ContactMessageReply>` | A `ContactMessageReply` identifier |
| `EmailSettingSecretId` | `Id<EmailSettingSecret>` | A `EmailSettingSecret` identifier |
| `MessageId` | `Id<Message>` | A `Message` identifier |
| `NoticeAttachmentId` | `Id<NoticeAttachment>` | A `NoticeAttachment` identifier |
| `NotificationDeliveryAttemptId` | `Id<NotificationDeliveryAttempt>` | A `NotificationDeliveryAttempt` identifier |
| `SendMessageRecipientId` | `Id<SendMessageRecipient>` | A `SendMessageRecipient` identifier |
| `SmsGatewayCredentialId` | `Id<SmsGatewayCredential>` | A `SmsGatewayCredential` identifier |

## Additional Enums

| Type | Values |
| ---- | ------ |
| `ComplaintAction` | (status/classification enum, see code) |
| `CustomSmsSettingParam` | (status/classification enum, see code) |
| `GatewayName` | (status/classification enum, see code) |
| `MailDriverName` | (status/classification enum, see code) |
| `NoticeAudience` | (status/classification enum, see code) |
| `NoticeType` | (status/classification enum, see code) |
| `NotificationMessage` | (status/classification enum, see code) |
| `NotificationType` | (status/classification enum, see code) |
| `RenderWarning` | (status/classification enum, see code) |
| `SmsGatewayCredentials` | (status/classification enum, see code) |
| `SmsTemplateVariable` | (status/classification enum, see code) |
