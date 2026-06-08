# Communication Domain — Workflows

Workflows orchestrate commands, queries, and policies to fulfill a
business goal. They are documented as ordered, conditional steps.

## Notice Publishing Workflow

```text
1. Author drafts a notice (CreateNotice).
2. Author updates the body and audience (UpdateNotice).
3. Author schedules or publishes (PublishNotice).
4. The notification adapter resolves the audience to a recipient list.
5. For each recipient, the adapter:
   a. Picks a template (NotificationSetting for the matching event).
   b. Renders the body with variable substitution.
   c. Selects a channel (Email or SMS) per setting and per recipient.
   d. Sends via the active EmailSetting or SmsGateway.
   e. Writes EmailLogged or SmsLogged.
   f. Writes a Notification record (Channel = Web or App).
6. Author may unpublish (UnpublishNotice), which suppresses new
   notifications but does not retract already-sent ones.
7. Author may delete (DeleteNotice) only when no recipient has received
   the notice.
```

**Pre-conditions:**
- The active EmailSetting or SmsGateway is configured.
- A NotificationSetting exists for the audience / event.

**Failure paths:**
- Missing template → `ValidationError::TemplateNotFound`.
- Channel disabled → `ConflictError::ChannelUnavailable`.
- All gateways down → `InfrastructureError::NoActiveGateway`.

## Complaint Lifecycle Workflow

```text
1. Complainant raises a complaint (RegisterComplaint).
2. Complaint is in status Open.
3. Reception assigns a resolver (AssignComplaint).
   a. Status moves to InProgress.
4. Resolver adds notes (AddComplaintNote).
5. Resolver resolves the complaint (ResolveComplaint).
   a. Status moves to Resolved.
   b. action_taken is captured.
6. Reports show open complaints by type, by source, by age.
```

**Edge cases:**
- A complainant withdraws → `UpdateComplaintStatus` to a `Withdrawn`
  pseudo-status (a future extension). The audit record remains.
- A complaint cannot be hard-deleted; the audit log is the source of
  truth.

## Chat Messaging Workflow

### One-to-One

```text
1. User A sends a chat invitation to User B (SendChatInvitation).
2. User B accepts (AcceptChatInvitation) or rejects (RejectChatInvitation).
3. If accepted, a ChatConversation is implicitly opened.
4. User A or B sends messages (SendChatMessage).
5. The receiver marks messages as seen (MarkChatMessageSeen).
6. A user may block the other (BlockUser), which suppresses future
   messages in both directions for the blocker.
```

### Group

```text
1. User creates a ChatGroup (CreateChatGroup) with privacy and type.
2. Initial members are added (AddUserToChatGroup).
3. Members post messages; the message is recorded with a
   ChatGroupMessageRecipient entry per member.
4. Each member marks the message as read (MarkGroupMessageRead).
5. A member may remove the message from their view
   (RemoveGroupMessageForUser).
6. Admins may post in a Closed group; only admins may post in a
   ReadOnly group.
7. Admins may add or remove members; removed members no longer receive
   future messages.
```

## Absence Notification Workflow

```text
1. SchoolAdmin configures the time window (ConfigureAbsentNotification).
2. The attendance domain emits StudentMarkedAbsent for a student.
3. AbsentNotificationService subscribes:
   a. If the current local time is within the configured window, the
      service picks the matching NotificationSetting and template.
   b. Renders the body with the student's name, class, and date.
   c. Selects a channel (Email / SMS) and a recipient (the primary
      guardian).
   d. Emits AbsentNotificationSent.
4. The notification adapter dispatches via the active gateway and
   writes EmailLogged or SmsLogged.
5. If the current time is outside the window, the dispatch is queued
   for the next window opening.
6. The teacher may disable the schedule (DisableAbsentNotification)
   to pause the workflow.
```

## Bulk Messaging Workflow

```text
1. SchoolAdmin creates a SendMessage job (CreateSendMessage) with an
   audience descriptor.
2. SchoolAdmin dispatches the job (DispatchSendMessage):
   a. The audience is resolved to a frozen recipient list.
   b. A Notification aggregate is created per recipient.
   c. The notification adapter sends the message via the configured
      channel and writes EmailLogged / SmsLogged.
3. SchoolAdmin may cancel a not-yet-dispatched job (CancelSendMessage).
4. Reports summarize sent volume by channel, by audience, by date.
```

## SMS Gateway Configuration Workflow

```text
1. SchoolAdmin configures a gateway (ConfigureSmsGateway) with
   credentials and a GatewayType.
2. SchoolAdmin activates the gateway (ActivateSmsGateway).
   a. The previous active gateway of the same type is demoted.
3. For Custom gateways, SchoolAdmin creates a CustomSmsSetting that
   defines the URL and parameter shape.
4. The notification adapter resolves to the active gateway at dispatch
   time.
5. SchoolAdmin may delete (DeleteSmsGateway) a non-active gateway.
```

## Email Engine Configuration Workflow

```text
1. SchoolAdmin configures an email setting (ConfigureEmailSetting)
   with driver, host, port, credentials (SecretReference), and
   encryption.
2. SchoolAdmin activates it (ActivateEmailSetting).
3. The notification adapter dispatches email through the active
   setting.
4. SchoolAdmin may delete (DeleteEmailSetting) a non-active setting.
```

## Contact Form Workflow

```text
1. A visitor on the public site submits a contact form
   (ReceiveContactMessage).
2. Reception staff views the message (MarkContactMessageViewed).
3. Reception staff replies (ReplyToContactMessage) via email or SMS.
4. The visitor's reply (if any) is appended to the same thread.
5. Reports show messages by view status, by reply status, by date.
```

## Speech Slider Workflow

```text
1. Marketing or principal authors a speech slider (CreateSpeechSlider).
2. The slider appears on the public site's home page.
3. The author updates (UpdateSpeechSlider) and removes (DeleteSpeechSlider)
   on rotation.
4. The order of rotation is managed through the public-site adapter
   (a port); the domain holds the records.
```

## Phone Call Follow-Up Workflow

```text
1. Front-office or teacher logs a call (LogPhoneCall) with name,
   phone, date, description, and optional duration.
2. The log may carry a follow-up date.
3. On follow-up, the date is updated (UpdatePhoneCallFollowUp).
4. Reports list open follow-ups by date, by staff, by status.
```

## Template Authoring Workflow

```text
1. SchoolAdmin creates a template (CreateSmsTemplate) with channel,
   purpose, subject, body, and declared variables.
2. SchoolAdmin enables the template (EnableSmsTemplate).
3. The template is referenced by a NotificationSetting or by a
   SendMessage dispatch.
4. Renderers validate that all variables are resolved before dispatch.
5. SchoolAdmin may disable (DisableSmsTemplate) without deleting.
```

## Idempotency

- `RegisterComplaint` is idempotent on `(complaint_type, date, phone)`.
  Re-issuing a complaint for the same phone on the same day returns
  the prior record.
- `SendChatMessage` is **not** idempotent: re-sends produce new
  `ChatMessage` aggregates.
- `BlockUser` is idempotent on `(block_by, block_to)`. A duplicate is
  a no-op success.
- `ConfigureAbsentNotification` is idempotent on `(school_id,
  time_from, time_to)`.
- `DispatchSendMessage` is **not** idempotent: re-dispatch produces
  duplicate notifications.

## Audit Requirements

Every state-changing command writes a durable audit record with the
actor, the correlation id, and a hash of the payload. PII (phone
numbers, email bodies) is captured but redacted in audit summaries.
