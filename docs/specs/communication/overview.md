# Communication Domain Overview

## Purpose

The communication domain owns the school's outward and inward messaging
fabric. It coordinates notices, complaints, chat, email and SMS dispatch
and logging, notifications, bulk messaging, contact-form submissions,
speech-slider content, and phone-call logging. It is the integration
surface between the school and its stakeholders — staff, students,
guardians, and the public.

The domain is intentionally **port-agnostic**: it models the *intent* to
send, the *record* of what was sent, and the *configuration* of channels
and templates. Actual delivery to SMTP, SMS gateways, push, or web is
performed by consumer-supplied notification adapters.

## Responsibilities

- Notice publication and dispatch (school-wide messages).
- Complaint intake, classification, assignment, and resolution.
- One-to-one and group chat conversations, including invitations,
  blocking, presence, and read receipts.
- Email and SMS dispatch with template-driven rendering.
- Email and SMS delivery logging for audit and review.
- In-app notifications to roles and individual users.
- Notification routing rules (event → channel → recipient → template).
- SMS gateway and email engine configuration.
- Custom SMS gateway configuration (consumer-defined gateway shapes).
- Bulk send-message jobs to roles, classes, and individuals.
- Absent-notification scheduling and dispatch.
- Contact-form message intake from the public website.
- Speech-slider content (leadership messages displayed on the website).
- Phone-call logging for follow-up workflows.

## Boundaries

The communication domain does **not** own:

- HTTP transport, mail servers, or SMS provider integrations. These are
  ports provided by consumer adapters (SMTP, Twilio, FCM, etc.).
- Authentication of staff, students, or guardians. The domain relies on
  the platform identity for the actor of a command.
- Push-notification fan-out logic for mobile apps. The `Notification`
  aggregate is the source of intent; an adapter performs the fan-out.
- Storage of files (form uploads, audio messages). The file storage port
  holds the bytes; the domain holds only the `FileReference`.
- Bulk SMS provider economics, rate limits, retry policies. The
  `NotificationProvider` port is responsible for these.

The communication domain **does** provide identifier types and value
objects that other domains depend on: `NoticeId`, `ComplaintId`,
`ChatConversationId`, `ChatGroupId`, `NotificationId`, `EmailLogId`,
`SmsLogId`, `SmsTemplateId`, `EmailSettingId`, `SmsGatewayId`,
`SendMessageId`, `ContactMessageId`, `PhoneCallLogId`,
`AbsentNotificationTimeSetupId`, `SpeechSliderId`.

## Dependencies

- `smscore-core` — error types, result, identifier trait.
- `smscore-platform` — `SchoolId`, `UserId`, `TenantContext`.
- `smscore-rbac` — capability checks.
- `smscore-events` — domain event publishing.
- `smscore-attendance` — emits `StudentMarkedAbsent` (read-only consumer).
- `smscore-academic` — `ClassId`, `SectionId`, `StudentId` for role-based
  routing.

## Domain Invariants

1. A `Notice` belongs to exactly one school.
2. A `Notice` may be unpublished (`is_published = false`) and is then
   invisible to recipients. Publishing is a one-way direction.
3. A `Complaint` is either `Open`, `InProgress`, or `Resolved`. No other
   statuses are valid.
4. A `Complaint` is anonymous only when `complaint_by` is empty and
   `complaint_source` is `Anonymous`; the resolver cannot re-identify
   the source from the event log.
5. A `ChatConversation` is between exactly two user references; a
   `ChatGroup` may have one or more members and a `Privacy` policy.
6. A `ChatGroup` is anchored to a school; cross-school groups are
   impossible.
7. A `ChatBlockUser` is one-way: blocking A by B does not imply blocking
   B by A.
8. A `Notification` is a domain record; whether it is *delivered* is
   tracked separately on the `Notification` aggregate by
   `NotificationStatus`.
9. A `SmsTemplate` is unique by `(school_id, purpose, channel)`.
10. A `SmsGateway` configuration contains at most one active gateway
    per `GatewayType` at any time. Activating a new gateway demotes the
    previous one for the same type.
11. An `AbsentNotificationTimeSetup` defines a `time_from` / `time_to`
    window during which absence notifications are eligible for
    dispatch. Outside the window, dispatches queue for the next window.
12. A `SendMessage` is a single bulk job. The set of recipients is
    resolved at the time of dispatch and is then frozen.
13. A `PhoneCallLog` is non-mutating; once written, only `next_follow_up_date`
    may be updated.

## Aggregate Roots

| Aggregate                       | Root Type                  | Purpose                                      |
| ------------------------------- | -------------------------- | -------------------------------------------- |
| Notice                          | `Notice`                   | School-wide notice publication               |
| Complaint                       | `Complaint`                | Complaint intake and lifecycle               |
| ComplaintType                   | `ComplaintType`            | Categorization for complaints                |
| Notification                    | `Notification`             | In-app notification record                   |
| EmailLog                        | `EmailLog`                 | Email dispatch audit record                  |
| SmsLog                          | `SmsLog`                   | SMS dispatch audit record                    |
| SmsTemplate                     | `SmsTemplate`              | Reusable template (SMS or email)             |
| EmailSetting                    | `EmailSetting`             | Email engine configuration                   |
| SmsGateway                      | `SmsGateway`               | SMS provider configuration                   |
| NotificationSetting             | `NotificationSetting`      | Event → channel routing rule                 |
| AbsentNotificationTimeSetup     | `AbsentNotificationTimeSetup` | Dispatch window for absence notifications |
| ChatMessage                     | `ChatMessage`              | Single chat message                          |
| ChatConversation                | `ChatConversation`         | Two-party conversation stream                |
| ChatGroup                       | `ChatGroup`                | Multi-party chat room                        |
| ChatGroupUser                   | `ChatGroupUser`            | Membership of a group                        |
| ChatGroupMessageRecipient       | `ChatGroupMessageRecipient`| Per-recipient delivery state                 |
| ChatGroupMessageRemove          | `ChatGroupMessageRemove`   | Per-user removal of a message                |
| ChatBlockUser                   | `ChatBlockUser`            | One-way block between users                  |
| ChatInvitation                  | `ChatInvitation`           | One-to-one chat invitation                   |
| ChatInvitationType              | `ChatInvitationType`       | Variant of an invitation                     |
| ChatStatus                      | `ChatStatus`               | Presence status of a user                    |
| SendMessage                     | `SendMessage`              | Bulk send-message job                        |
| ContactMessage                  | `ContactMessage`           | Public contact-form submission               |
| SpeechSlider                    | `SpeechSlider`             | Front-page leadership message                |
| PhoneCallLog                    | `PhoneCallLog`             | Phone-call follow-up record                  |
| CustomSmsSetting                | `CustomSmsSetting`         | Consumer-defined SMS gateway shape           |

Each aggregate is documented in detail under
`docs/specs/communication/aggregates.md`.

## Cross-Domain Impact

When the **attendance** domain emits `StudentMarkedAbsent`, the
communication domain subscribes via the `AbsentNotificationService`,
which evaluates the configured time window, picks the notification
setting that matches the absence event, renders the template, and emits
`NotificationSent` and `SmsLogged` events. The actual delivery to a
gateway is performed by the notification adapter.

When a **complaint is resolved**, the engine may emit a domain event
that the website / mobile app subscribes to in order to update the
parent-visible status.

When a **send-message job** is dispatched, individual `Notification`
aggregates are created per recipient. Recipients are computed from the
job's audience (role, class, section, or individual user list).

## Consumers

- Web admin UI (compose notices, resolve complaints, view logs).
- Mobile parent app (receive notices, chat with teacher, view
  notifications).
- Mobile student app (chat, notifications, complaints).
- Mobile teacher app (chat, complaint triage, send messages).
- Web public site (contact form, speech slider, notice board).
- AI agent (compose notices, log calls, schedule notifications, moderate
  chat invitations).

## Anti-Goals

- The communication domain does not present data to humans. It exposes
  commands, events, and queries.
- The communication domain does not implement a delivery channel.
  Channels are ports.
- The communication domain does not decide school policy (e.g. "absence
  triggers a notification"). That is a configuration value managed by
  the consumer through `NotificationSetting`.
- The communication domain does not own a chat transport. Messages
  produced here are domain events; a real-time adapter (websocket, push)
  is a consumer concern.
