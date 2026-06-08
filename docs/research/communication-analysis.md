# Communication Domain — Business Analysis

## Purpose

The communication domain owns the school's interaction
with parents, students, staff, and external parties. It
is the school's voice: notices, complaints, chat,
notifications.

This document describes how communication works in
real schools, with the edge cases that real schools hit.

## Key Concepts

- **Notice** — an announcement sent to a defined
  audience (all parents, a class, a section, an
  individual).
- **Complaint** — a parent's or student's complaint or
  feedback, tracked to resolution.
- **ChatThread** — a multi-party chat conversation
  (parent-teacher, group chat, etc.).
- **ChatMessage** — a single message in a chat thread.
- **Notification** — a system-generated message
  (push, SMS, email) sent to a user.
- **NoticeAudience** — the target group for a notice
  (all, class, section, individual, role).
- **NoticeChannel** — the delivery channel (in-app,
  email, SMS, push).

## Real-World Scenarios

### Notice Publishing

The school wants to announce "Parent-Teacher Meeting
on Saturday." The admin:

1. Creates a `Notice` with title, body, audience
   (all parents), and channels (in-app, SMS, email).
2. Sets a publication date (today) and an
   expiry date (the day after the meeting).
3. Publishes the notice.

The engine emits `NoticePublished`. The communication
domain's notification worker fans out to the audience:
- For each parent in the audience, send an in-app
  notification.
- For each parent with a phone, send an SMS.
- For each parent with an email, send an email.

The audit log records every delivery attempt. A
failed delivery (e.g. invalid phone number) is logged
but does not abort the batch.

### Class-Specific Notice

A class teacher wants to inform "Grade 7A parents
that tomorrow's class is cancelled." The teacher
creates a notice with audience = "Class 7A" and
channels = "in-app, SMS."

The engine resolves the audience: parents of students
in Class 7A for the current academic year. The
notification worker sends.

### Individual Notice

The principal wants to send a confidential note to a
specific parent. The principal creates a notice with
audience = "individual" and the parent's user id.

### Complaint Filing

A parent files a complaint: "The bus was late
yesterday." The parent:

1. Creates a `Complaint` with subject, description,
   and optional attachment.
2. The complaint is assigned a unique id and a
   status (`Open`).
3. The school admin (or the transport in-charge)
   sees the complaint in the admin queue.
4. The admin responds; the complaint's status
   becomes `InProgress`.
5. The admin resolves; the status becomes `Resolved`.
6. The parent is notified at each transition.

The engine's `Complaint` aggregate captures the
complaint and its lifecycle.

### Complaint Categories

A school categorizes complaints:
- Transport.
- Facilities.
- Academics.
- Behavior.
- Finance.
- Other.

The engine's `ComplaintCategory` is a per-school
configuration.

### Anonymous Complaint

A school may allow anonymous complaints. The
engine's `Complaint` aggregate supports
`is_anonymous = true`; the parent's user id is
recorded internally but not displayed publicly.

### Chat (Parent-Teacher)

A parent wants to message the class teacher. The
parent opens a chat thread. The engine creates a
`ChatThread` with the parent and the teacher as
participants. The parent and teacher exchange
`ChatMessage`s in real time.

In real schools, chat is moderated:
- Working hours only (e.g. 8am-6pm).
- A school may have a "no chat" policy and use
  only formal notices and complaints.
- A school may have an emergency hotline separate
  from chat.

The engine's chat is **port-driven** for delivery.
The default implementation is in-app; the consumer
may add SMS or email integration.

### Group Chat (Class Group)

A class teacher creates a group chat for all parents
of the class. The chat is read-only for parents
(announcement style) or open (group style). The
engine's `ChatThreadType` distinguishes.

### Notifications

Notifications are system-generated messages sent
to a user. Sources:
- `AttendanceMarked` → "Your child is absent today."
- `PaymentCollected` → "Your payment of ₹5,000 is
  received."
- `ResultPublished` → "Your child's result is
  published."
- `NoticePublished` → "New notice: Annual Day."
- `LeaveApproved` → "Your leave application is
  approved."
- `PayrollPaid` → "Your salary for October is paid."

The engine's `Notification` aggregate records
every notification with status (`Pending`,
`Sent`, `Delivered`, `Read`, `Failed`).

The engine's notification worker subscribes to
events from every domain and produces
notifications. The worker is rate-limited and
batched.

### Notification Channels

A school configures which channels are active:
- In-app (always on).
- Email (configurable per user).
- SMS (configurable per user; charged per message).
- Push (mobile app).

The engine's `NotificationPreference` is a per-user
configuration. A user may opt out of certain
notification types or channels.

### Read Receipts

A notice is marked "read" when the user opens it
in the app. The engine's `NoticeRead` event
captures the read. The school's admin sees
read-receipts (who has read the notice, who has
not).

## Business Rules

1. A `Notice` requires a title, a body, an
   audience, and at least one channel.
2. A `Notice` cannot be sent to an audience with
   zero members. The engine validates at creation.
3. A `Complaint` requires a subject, a description,
   and a category.
4. A `Complaint`'s status transitions are
   `Open → InProgress → Resolved` (or
   `Open → Rejected`).
5. A `Complaint` cannot be deleted; it can be
   closed with a reason.
6. A `ChatThread` requires at least two
   participants.
7. A `ChatMessage` cannot be edited after 5
   minutes (configurable per school). The
   edit is auditable.
8. A `Notification` is **at-least-once**. The
   engine retries on transient failures; a
   permanent failure is logged and escalated.
9. A user may opt out of certain notification
   types. The engine respects the preference.
10. A `Notice`'s publication date is in the
    future or the present. Past dates are not
    allowed for new notices; the engine's
    scheduler handles scheduled publication.

## Edge Cases

### Bulk Notice Sending Failure

A school sends a notice to 5,000 parents. The SMS
provider's rate limit is hit. The engine's
notification worker queues the unsent messages
and retries with exponential backoff. A subset
fails permanently (e.g. invalid phone numbers);
the engine logs the failures and reports to the
admin.

### Notice with Sensitive Content

A school sends a notice about a specific student's
expulsion. The notice's audience is the
**individual parent**, not the whole class. The
engine's per-individual audience ensures the
notice reaches only the intended parent.

### Parent Opt-Out

A parent opts out of SMS notifications. The
engine's notification worker skips the SMS
channel for that parent. The in-app and email
channels remain active.

### Parent with Multiple Children

A parent has three children. The parent receives
notices for each child separately, and notices
targeting the parent's role (e.g. "PTA meeting
for all parents"). The engine's audience
resolution handles both cases.

### Anonymous Complaint Retaliation

A parent files an anonymous complaint about a
teacher. The teacher, suspecting the parent,
retaliates by failing the parent's child. The
school investigates. The engine's audit log
shows the complaint, the resolution, and any
subsequent changes to the child's marks (which
would be flagged by the assessment domain's
audit).

### Chat Outside Working Hours

A parent messages the teacher at 10pm. The
engine's chat is delivered immediately (in-app
push). The teacher sees the message the next
morning. The school may configure a
"do-not-disturb" window that suppresses
push notifications for chat but retains the
in-app message.

### Notification Storm

A school has 1,000 students. An exam is
published. The engine emits 1,000
`ResultPublished` events. The notification
worker fans out 1,000 parent notifications.
The engine's batched delivery coalesces the
notifications into a single digest per
parent.

### Read Receipts and Privacy

A parent reads a notice. The school admin sees
the read receipt. The engine does not expose
read receipts across schools or to other
parents.

### Notice Scheduled in the Past

The admin schedules a notice for "yesterday" by
mistake. The engine rejects the schedule (the
publication date must be in the future or
present). The admin reschedules.

### Complaint with Attachment

A parent files a complaint with a photo (e.g.
damaged property). The engine's `Complaint`
aggregate supports attachments. The file is
stored in the consumer's file storage; the
engine records the reference.

## Notes for SMSengine Implementation

- The **communication** crate depends on
  `smscore-events` (for event subscriptions) and
  `smscore-rbac`. It does not depend on
  operational domains; it consumes their events.
- The domain is **event-driven**. Notices,
  complaints, and notifications are produced
  by reactions to events from other domains.
- The domain's **notification worker** is a
  background process that subscribes to events
  and produces notifications. The worker is
  rate-limited, batched, and retried.
- The domain's **delivery** is a port. The
  consumer provides the SMS / email / push
  provider.
- The domain's **read receipts** are
  capability-gated. Only the school's admin
  can see who has read a notice.
- The domain's **chat** is real-time. The
  consumer's frontend uses WebSockets or
  Server-Sent Events for live updates; the
  engine's chat is the source of truth.
- The domain's **complaints** are auditable.
  Every status transition is recorded; the
  full history is queryable.
- The domain's **per-user preferences** are
  stored in the platform domain's `User`
  aggregate. The engine reads them at
  notification dispatch time.
