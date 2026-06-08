# Communication Domain — Entities

Entities have identity and lifecycle but are not aggregate roots. They
are loaded and persisted only through their aggregate root.

## NoticeAudience

**Identity:** Embedded in `Notice`
**Owner:** `Notice`

A materialized list of role ids the notice is addressed to. The audience
is captured at the moment of dispatch and is not retroactively updated
when role membership changes.

## NoticeAttachment

**Identity:** `NoticeAttachmentId(SchoolId, Uuid)`
**Owner:** `Notice`

A `FileReference` attached to a notice, with an optional caption and
display order.

## ComplaintNote

**Identity:** `ComplaintNoteId(SchoolId, Uuid)`
**Owner:** `Complaint`

A free-text note added to a complaint, with a `UserId` author and a
timestamp. Notes are append-only.

## NotificationDeliveryAttempt

**Identity:** `NotificationDeliveryAttemptId(SchoolId, Uuid)`
**Owner:** `Notification`

A record of one delivery attempt for a `Notification`, including the
`Channel`, the adapter used, and the outcome (`Delivered`, `Failed`,
`Deferred`). Multiple attempts are possible.

## SmsTemplateVariable

**Identity:** Embedded list in `SmsTemplate`
**Owner:** `SmsTemplate`

A declared placeholder name and a description. The renderer validates
that all variables are resolved at dispatch time.

## NotificationSettingAudience

**Identity:** Embedded in `NotificationSetting`
**Owner:** `NotificationSetting`

A description of the recipient class: a role id list, a class id, a
section id, or a combination. Stored as a typed enum on the
`NotificationSetting` aggregate.

## ChatGroupAvatar

**Identity:** `ChatGroupAvatarId(SchoolId, Uuid)`
**Owner:** `ChatGroup`

A `FileReference` for the group's avatar image.

## ChatGroupMessage

**Identity:** `ChatGroupMessageId(SchoolId, Uuid)`
**Owner:** `ChatGroup` (with recipients tracked separately)

The message body of a group message. The fan-out is tracked by
`ChatGroupMessageRecipient`; the body itself is stored once on this
entity.

## ChatConversationLastRead

**Identity:** `ChatConversationLastReadId(SchoolId, Uuid)`
**Owner:** `ChatConversation`

A per-side last-read message id and timestamp for a two-party
conversation.

## SendMessageRecipient

**Identity:** `SendMessageRecipientId(SchoolId, Uuid)`
**Owner:** `SendMessage`

A materialized recipient with the channel used, the delivery outcome,
and the timestamp. Created when `SendMessage` is dispatched.

## EmailSettingSecret

**Identity:** `EmailSettingSecretId(SchoolId, Uuid)`
**Owner:** `EmailSetting`

A `SecretReference` for the SMTP password. The secret is held in the
secret-store port; the aggregate holds only the reference.

## SmsGatewayCredential

**Identity:** `SmsGatewayCredentialId(SchoolId, Uuid)`
**Owner:** `SmsGateway`

A typed credential record. The shape is determined by the
`GatewayType` enum. Secrets are stored via `SecretReference`.

## CustomSmsSettingParam

**Identity:** Embedded in `CustomSmsSetting`
**Owner:** `CustomSmsSetting`

A single parameter key/value pair on a custom gateway. Up to eight are
supported.

## AbsentNotificationDispatch

**Identity:** `AbsentNotificationDispatchId(SchoolId, Uuid)`
**Owner:** `AbsentNotificationTimeSetup`

A scheduled dispatch for a specific absence event, with the recipient
list, the rendered message, and the dispatch outcome. Created by
`AbsentNotificationService` when an `StudentMarkedAbsent` event is
received and the time window is open.

## ContactMessageReply

**Identity:** `ContactMessageReplyId(SchoolId, Uuid)`
**Owner:** `ContactMessage`

The reply sent back to the contact, with author `UserId`, body, and
timestamp.
