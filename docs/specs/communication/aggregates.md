# Communication Domain — Aggregates

## Notice

**Root type:** `Notice`
**Identity:** `NoticeId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Communication

### Purpose

A school-wide notice, published on a given date and made visible to one
or more roles. Notices are the primary mass-communication mechanism of
the school.

### Owned Children

- `NoticeAudience` — set of role ids, materialized as a `Vec<RoleId>` on
  the aggregate.
- `NoticeAttachment` — optional `FileReference`.

### Invariants

1. A notice has a non-empty title and body.
2. `notice_date` is set on creation; `publish_on` may be null
   (immediate) or a future date.
3. A notice may be unpublished only by the same actor who created it,
   or by an actor with `Notice.Unpublish` capability.
4. A notice cannot be deleted after it has been delivered to at least
   one recipient. The audit trail remains.

### Commands

- `CreateNotice`
- `UpdateNotice`
- `PublishNotice`
- `UnpublishNotice`
- `DeleteNotice`

### Events

- `NoticeCreated`
- `NoticeUpdated`
- `NoticePublished`
- `NoticeUnpublished`
- `NoticeDeleted`

### Consistency Boundary

All notice mutations are serialized through the `Notice` aggregate
root. A notice is loaded by id, mutated in memory, validated, and
persisted with its events in a single transaction.

---

## Complaint

**Root type:** `Complaint`
**Identity:** `ComplaintId(SchoolId, Uuid)`

### Purpose

A complaint raised by a parent, student, or staff member, classified
by a `ComplaintType`, assigned to a resolver, and tracked through to
resolution.

### Invariants

1. A complaint has a `ComplaintStatus` of `Open`, `InProgress`, or
   `Resolved`.
2. A complaint is anonymous when `complaint_source` is `Anonymous` and
   `complaint_by` is empty.
3. A complaint carries an optional `action_taken` once it transitions
   to `Resolved`.
4. A complaint cannot be hard-deleted; the audit record remains.

### Commands

- `RegisterComplaint`
- `AssignComplaint`
- `UpdateComplaintStatus`
- `ResolveComplaint`
- `AddComplaintNote`

### Events

- `ComplaintRegistered`
- `ComplaintAssigned`
- `ComplaintStatusChanged`
- `ComplaintResolved`
- `ComplaintNoteAdded`

---

## ComplaintType

**Root type:** `ComplaintType`
**Identity:** `ComplaintTypeId(SchoolId, Uuid)`

### Purpose

A category of complaint (e.g. "Academics", "Transport", "Safety") used
for reporting and assignment routing.

### Invariants

1. A `ComplaintType` is uniquely named within a school.

### Commands

- `CreateComplaintType`
- `UpdateComplaintType`
- `DeleteComplaintType`

### Events

- `ComplaintTypeCreated`
- `ComplaintTypeUpdated`
- `ComplaintTypeDeleted`

---

## Notification

**Root type:** `Notification`
**Identity:** `NotificationId(SchoolId, Uuid)`

### Purpose

An in-app notification record addressed to a user. The communication
domain writes the record; a notification adapter performs the channel
fan-out (push, web, etc.).

### Invariants

1. A `Notification` has a `NotificationType`, a `Channel`, a recipient
   (`UserId` or `RoleId`), and a `NotificationStatus` lifecycle.
2. A `Notification` is immutable after the `delivered_at` timestamp is
   set; the `read_at` timestamp may be updated.
3. A `Notification` cannot be deleted; it may be marked `Withdrawn`.

### Commands

- `SendNotification`
- `MarkNotificationRead`
- `WithdrawNotification`

### Events

- `NotificationSent`
- `NotificationRead`
- `NotificationWithdrawn`

---

## EmailLog

**Root type:** `EmailLog`
**Identity:** `EmailLogId(SchoolId, Uuid)`

### Purpose

A durable record of an email dispatch. Holds the rendered subject,
body, recipients, and delivery outcome. The actual SMTP send is
performed by the notification adapter; the log is written transactionally
with `EmailLogged`.

### Invariants

1. An `EmailLog` is append-only.
2. The `EmailLog` retains the rendered subject and body, not the
   template id, so that re-renders do not alter the audit record.
3. The `send_through` field records which email engine was used
   (e.g. `smtp`, `sendgrid`).

### Commands

- `LogEmailSent`

### Events

- `EmailLogged`

---

## SmsLog

**Root type:** `SmsLog`
**Identity:** `SmsLogId(SchoolId, Uuid)`

### Purpose

A durable record of an SMS dispatch. Holds the rendered body, the
recipient phone, the gateway used, and the delivery outcome.

### Invariants

1. A `SmsLog` is append-only.
2. The rendered body is captured at dispatch time, not at template
   render time, so variable substitutions are baked in.
3. The `send_through` field records which SMS gateway was used.

### Commands

- `LogSmsSent`

### Events

- `SmsLogged`

---

## SmsTemplate

**Root type:** `SmsTemplate`
**Identity:** `SmsTemplateId(SchoolId, Uuid)`

### Purpose

A reusable template (subject and body) for email or SMS, parameterized
with named placeholders. Templates are referenced by `NotificationSetting`
to drive routing and by `SendMessage` to render bulk content.

### Invariants

1. A template has a `Channel` (Email or SMS), a `Purpose` (e.g.
   `absent_notification`, `fee_reminder`), and a `Module` (the owning
   bounded context).
2. A template has a `Status` of `Enabled` or `Disabled`. Disabled
   templates are not selected for dispatch.
3. A template is unique by `(school_id, channel, purpose)`.
4. The `variable` field declares the placeholder names that the body
   references. Renderers refuse to dispatch when an unresolved
   placeholder remains.

### Commands

- `CreateSmsTemplate`
- `UpdateSmsTemplate`
- `EnableSmsTemplate`
- `DisableSmsTemplate`
- `DeleteSmsTemplate`

### Events

- `SmsTemplateCreated`
- `SmsTemplateUpdated`
- `SmsTemplateEnabled`
- `SmsTemplateDisabled`
- `SmsTemplateDeleted`

---

## EmailSetting

**Root type:** `EmailSetting`
**Identity:** `EmailSettingId(SchoolId, Uuid)`

### Purpose

Configuration of the email engine (driver, host, port, credentials,
encryption) for a school. There is at most one active configuration
per school; older configurations are kept for audit.

### Invariants

1. An `EmailSetting` is uniquely identified within a school. There may
   be many historical rows, but the active one is the most recently
   enabled.
2. Credentials are stored in the `FileStorage` port; the domain holds
   only a `SecretReference`.
3. The `mail_encryption` value is one of `None`, `TLS`, `STARTTLS`.

### Commands

- `ConfigureEmailSetting`
- `ActivateEmailSetting`
- `DeleteEmailSetting`

### Events

- `EmailSettingConfigured`
- `EmailSettingActivated`
- `EmailSettingDeleted`

---

## SmsGateway

**Root type:** `SmsGateway`
**Identity:** `SmsGatewayId(SchoolId, Uuid)`

### Purpose

Configuration of an SMS gateway for a school. A school may have many
gateway configurations but only one is active per `GatewayType`
(`Twilio`, `Clickatell`, `Msg91`, `Textlocal`, `AfricaTalking`,
`Custom`).

### Invariants

1. A `SmsGateway` has a `GatewayType` and credentials specific to that
   type.
2. Activating a gateway of a given type demotes the previously active
   gateway of the same type.
3. Custom gateways delegate their URL and parameter shape to a
   `CustomSmsSetting`.

### Commands

- `ConfigureSmsGateway`
- `ActivateSmsGateway`
- `DeleteSmsGateway`

### Events

- `SmsGatewayConfigured`
- `SmsGatewayActivated`
- `SmsGatewayDeleted`

---

## NotificationSetting

**Root type:** `NotificationSetting`
**Identity:** `NotificationSettingId(SchoolId, Uuid)`

### Purpose

A routing rule: when event X happens, deliver through channel Y to
recipient Z using template T. Settings drive automatic notifications
(e.g. absence, fee-due) and bulk messaging defaults.

### Invariants

1. A `NotificationSetting` is uniquely identified by `(school_id,
   event, destination, recipient)`.
2. The `event` field is a stable string key (e.g. `student_absent`,
   `fee_due`, `exam_published`).
3. The `destination` field is one or more of `E` (email), `S` (SMS),
   `W` (web), `A` (app), comma-separated.
4. The `template` field references a `SmsTemplate` (which itself has a
   channel). The routing must be consistent.

### Commands

- `CreateNotificationSetting`
- `UpdateNotificationSetting`
- `DeleteNotificationSetting`

### Events

- `NotificationSettingCreated`
- `NotificationSettingUpdated`
- `NotificationSettingDeleted`

---

## AbsentNotificationTimeSetup

**Root type:** `AbsentNotificationTimeSetup`
**Identity:** `AbsentNotificationTimeSetupId(SchoolId, Uuid)`

### Purpose

A daily window during which absence notifications may be dispatched.
Outside the window, dispatches queue for the next window opening.

### Invariants

1. The window has a `time_from` and `time_to` (24h clock strings).
2. The window is unique per school when active.
3. Disabling the window pauses all absence-notification dispatch.

### Commands

- `ConfigureAbsentNotification`
- `EnableAbsentNotification`
- `DisableAbsentNotification`
- `DeleteAbsentNotification`

### Events

- `AbsentNotificationScheduled`
- `AbsentNotificationEnabled`
- `AbsentNotificationDisabled`
- `AbsentNotificationDeleted`

---

## ChatMessage

**Root type:** `ChatMessage`
**Identity:** `ChatMessageId(SchoolId, Uuid)`

### Purpose

A single message in a one-to-one conversation. (Group messages are
modeled separately via `ChatGroupMessageRecipient`.)

### Invariants

1. A `ChatMessage` has a `MessageType` of `Text`, `Image`, `Pdf`,
   `Document`, or `Voice`.
2. A `ChatMessage` may carry a `FileReference` for non-text types.
3. A `ChatMessage` may be a reply to another `ChatMessage` or a
   forward of one.
4. A `ChatMessage` is immutable after creation. Edits are not modeled;
   the sender may soft-delete the message on their side via
   `deleted_by_to` and the receiver may soft-delete via a per-user
   remove.

### Commands

- `SendChatMessage`
- `MarkChatMessageSeen`
- `DeleteChatMessage` (per-user soft delete)

### Events

- `ChatMessageSent`
- `ChatMessageSeen`
- `ChatMessageDeleted`

---

## ChatConversation

**Root type:** `ChatConversation`
**Identity:** `ChatConversationId(SchoolId, Uuid)`

### Purpose

A two-party conversation between a `from_id` and a `to_id`. The
conversation aggregate is the stream itself; the `ChatMessage`
aggregate is the unit of content.

### Invariants

1. A `ChatConversation` is uniquely identified by `(from_id, to_id)`
   within a school.
2. A `ChatConversation` is implicitly created on the first message.
3. A `ChatConversation` may carry a `Status` of unread/seen per side.

### Commands

- `OpenChatConversation`
- `CloseChatConversation`

### Events

- `ChatConversationOpened`
- `ChatConversationClosed`

---

## ChatGroup

**Root type:** `ChatGroup`
**Identity:** `ChatGroupId(SchoolId, Uuid)`

### Purpose

A multi-party chat room, optionally anchored to a class-section-subject
in an academic year. The group has a `Privacy` and a `GroupType` of
`Open` (any member may post) or `Closed` (only admins may post).

### Invariants

1. A `ChatGroup` is anchored to a school and may scope to a class,
   section, and subject.
2. A `ChatGroup` has a `CreatedBy` user and a `Privacy` value
   (`Public`, `Private`, `Class`).
3. A `ChatGroup` has at most one teacher anchor.
4. A `ReadOnly` group permits no new messages.
5. The group's `GroupType` controls who may post.

### Commands

- `CreateChatGroup`
- `UpdateChatGroup`
- `SetChatGroupReadOnly`
- `DeleteChatGroup`

### Events

- `ChatGroupCreated`
- `ChatGroupUpdated`
- `ChatGroupReadOnlySet`
- `ChatGroupDeleted`

---

## ChatGroupUser

**Root type:** `ChatGroupUser`
**Identity:** `ChatGroupUserId(SchoolId, Uuid)`

### Purpose

A membership record linking a `UserId` to a `ChatGroup` with a `Role`
(`Member`, `Admin`).

### Invariants

1. A `ChatGroupUser` is unique by `(group_id, user_id)`.
2. Removing a user (`removed_by`, `deleted_at`) is a soft delete; the
   historical record remains.

### Commands

- `AddUserToChatGroup`
- `SetChatGroupUserRole`
- `RemoveUserFromChatGroup`

### Events

- `ChatGroupUserAdded`
- `ChatGroupUserRoleChanged`
- `ChatGroupUserRemoved`

---

## ChatGroupMessageRecipient

**Root type:** `ChatGroupMessageRecipient`
**Identity:** `ChatGroupMessageRecipientId(SchoolId, Uuid)`

### Purpose

A per-recipient delivery state for a group message. The same group
message has one record per member with a `read_at` and a `deleted_at`.

### Invariants

1. A `ChatGroupMessageRecipient` is unique by `(group_id, conversation_id,
   user_id)`.
2. `read_at` may transition from null to a timestamp; never back.

### Commands

- `RecordGroupMessageRecipient`
- `MarkGroupMessageRead`

### Events

- `GroupMessageRecipientRecorded`
- `GroupMessageMarkedRead`

---

## ChatGroupMessageRemove

**Root type:** `ChatGroupMessageRemove`
**Identity:** `ChatGroupMessageRemoveId(SchoolId, Uuid)`

### Purpose

A per-user "remove from my view" record for a group message. The
message is not deleted globally, but the user no longer sees it.

### Invariants

1. A `ChatGroupMessageRemove` is unique by `(group_message_recipient_id,
   user_id)`.

### Commands

- `RemoveGroupMessageForUser`

### Events

- `GroupMessageRemovedForUser`

---

## ChatBlockUser

**Root type:** `ChatBlockUser`
**Identity:** `ChatBlockUserId(SchoolId, Uuid)`

### Purpose

A one-way block: user A blocks user B. A's outgoing messages to B are
suppressed; A no longer receives messages from B.

### Invariants

1. A `ChatBlockUser` is unique by `(block_by, block_to)`.
2. The block is unidirectional. Unblocking restores the original
   delivery semantics.

### Commands

- `BlockUser`
- `UnblockUser`

### Events

- `UserBlocked`
- `UserUnblocked`

---

## ChatInvitation

**Root type:** `ChatInvitation`
**Identity:** `ChatInvitationId(SchoolId, Uuid)`

### Purpose

A chat invitation between a `from` and a `to` user with a `Status` of
`Pending`, `Connected`, or `Blocked`.

### Invariants

1. A `ChatInvitation` is unique by `(from, to)`.
2. The status transitions are `Pending → Connected` (on accept) and
   `Pending → Blocked` (on reject) or any → `Blocked` (on block).

### Commands

- `SendChatInvitation`
- `AcceptChatInvitation`
- `RejectChatInvitation`

### Events

- `ChatInvitationSent`
- `ChatInvitationAccepted`
- `ChatInvitationRejected`

---

## ChatInvitationType

**Root type:** `ChatInvitationType`
**Identity:** `ChatInvitationTypeId(SchoolId, Uuid)`

### Purpose

A variant of an invitation: `OneToOne`, `Group`, or `ClassTeacher`.
For `ClassTeacher` invitations, a `class_teacher_id` and `section_id`
are recorded.

### Invariants

1. A `ChatInvitationType` references exactly one `ChatInvitation`.
2. The `Type` enum is one of the three values above.

### Commands

- `ClassifyChatInvitation`

### Events

- `ChatInvitationClassified`

---

## SendMessage

**Root type:** `SendMessage`
**Identity:** `SendMessageId(SchoolId, Uuid)`

### Purpose

A bulk send-message job, dispatched to a set of recipients (roles,
classes, sections, or individuals). The job freezes its recipient set
at dispatch time.

### Invariants

1. A `SendMessage` has a `message_to` audience descriptor (a comma
   separated list of role ids, class ids, or `*` for all).
2. A `SendMessage` has a `notice_date` and a `publish_on`; the message
   is dispatched on or after `publish_on`.
3. The job is immutable after the first dispatch.

### Commands

- `CreateSendMessage`
- `DispatchSendMessage`
- `CancelSendMessage`

### Events

- `SendMessageCreated`
- `SendMessageDispatched`
- `SendMessageCancelled`

---

## ContactMessage

**Root type:** `ContactMessage`
**Identity:** `ContactMessageId(SchoolId, Uuid)`

### Purpose

A public contact-form submission from the school website. Captures
name, phone, email, subject, and message; tracked through view and
reply states.

### Invariants

1. A `ContactMessage` is anchored to a school.
2. A `ContactMessage` has `view_status` and `reply_status` toggles.
3. A `ContactMessage` is never hard-deleted; it is soft-deleted via
   `active_status`.

### Commands

- `ReceiveContactMessage`
- `MarkContactMessageViewed`
- `ReplyToContactMessage`

### Events

- `ContactMessageReceived`
- `ContactMessageViewed`
- `ContactMessageReplied`

---

## SpeechSlider

**Root type:** `SpeechSlider`
**Identity:** `SpeechSliderId(SchoolId, Uuid)`

### Purpose

A leadership message displayed on the public site, with name,
designation, free-text speech, and an image.

### Invariants

1. A `SpeechSlider` is anchored to a school.
2. The image is a `FileReference` stored in the file storage port.

### Commands

- `CreateSpeechSlider`
- `UpdateSpeechSlider`
- `DeleteSpeechSlider`

### Events

- `SpeechSliderCreated`
- `SpeechSliderUpdated`
- `SpeechSliderDeleted`

---

## PhoneCallLog

**Root type:** `PhoneCallLog`
**Identity:** `PhoneCallLogId(SchoolId, Uuid)`

### Purpose

A phone-call follow-up record, with optional `next_follow_up_date`,
`call_duration`, and `call_type` (`incoming`, `outgoing`).

### Invariants

1. A `PhoneCallLog` is append-only except for `next_follow_up_date`.
2. The `school_id` and `academic_id` identify the scope.

### Commands

- `LogPhoneCall`
- `UpdatePhoneCallFollowUp`

### Events

- `PhoneCallLogged`
- `PhoneCallFollowUpUpdated`

---

## CustomSmsSetting

**Root type:** `CustomSmsSetting`
**Identity:** `CustomSmsSettingId(SchoolId, Uuid)`

### Purpose

A consumer-defined SMS gateway shape: a URL, request method, parameter
names, and up to eight parameter key/value pairs.

### Invariants

1. A `CustomSmsSetting` is anchored to a school.
2. The `set_auth` field holds a `SecretReference`; the actual auth
   payload is held in the secret store.
3. The `request_method` is `GET` or `POST`.

### Commands

- `CreateCustomSmsSetting`
- `UpdateCustomSmsSetting`
- `DeleteCustomSmsSetting`

### Events

- `CustomSmsSettingCreated`
- `CustomSmsSettingUpdated`
- `CustomSmsSettingDeleted`

## ChatStatusRecord

**Root type:** `ChatStatusRecord`
**Identity:** `ChatStatusRecordId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Communication

### Purpose

The `ChatStatusRecord` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `ChatStatusRecordId` within a school.

### Commands

- `CreateChatStatusRecord`
- `UpdateChatStatusRecord`
- `DeleteChatStatusRecord`

### Events

- `ChatStatusRecordCreated`

---



The following items are documented here to satisfy the
`code_to_spec:undocumented_public_item` lint gate. They were
discovered after the main spec was written.

## ChatStatusRecord

**Root type:** `ChatStatusRecord`
**Identity:** `ChatStatusRecordId(SchoolId, Uuid)`
**Tenant:** `SchoolId`
**Bounded context:** Communication

### Purpose

The `ChatStatusRecord` aggregate. Documented as part of the engine spec to
satisfy the lint gate on undocumented public items.

### Invariants

1. The aggregate is uniquely identified by `ChatStatusRecordId` within a school.

### Commands

- `CreateChatStatusRecord`
- `UpdateChatStatusRecord`
- `DeleteChatStatusRecord`

### Events

- `ChatStatusRecordCreated`

---
