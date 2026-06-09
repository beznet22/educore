# Communication Domain — Tables

The communication domain is backed by the following tables. Each table
maps to one or more aggregates; the `aggregate` column tells you which
aggregate owns the row.

| Table                                        | Aggregate                    | Notes                                          |
| -------------------------------------------- | ---------------------------- | ---------------------------------------------- |
| `communication_notice_boards`                           | Notice                       | School-wide notice publication                 |
| `communication_complaints`                              | Complaint                    | Complaint intake and lifecycle                 |
| `communication_complaint_types`                         | ComplaintType                | Categorization                                 |
| `communication_notifications`                           | Notification                 | In-app notification record                     |
| `notifications`                              | Notification (alt)           | Generic notification inbox                      |
| `communication_notification_settings`                   | NotificationSetting          | Event → channel routing rule                   |
| `communication_email_sms_logs`                          | EmailLog / SmsLog            | Email and SMS dispatch audit                   |
| `sms_templates`                              | SmsTemplate                  | Reusable template (SMS or email)               |
| `communication_email_settings`                          | EmailSetting                 | Email engine configuration                     |
| `communication_sms_gateways`                            | SmsGateway                   | SMS provider configuration                     |
| `custom_sms_settings`                        | CustomSmsSetting             | Custom gateway parameter shape                 |
| `absent_notification_time_setups`            | AbsentNotificationTimeSetup  | Daily window for absence notification dispatch |
| `chat_conversations`                         | ChatMessage / ChatConversation | One-to-one message and conversation stream   |
| `chat_groups`                                | ChatGroup                    | Multi-party chat room                          |
| `chat_group_users`                           | ChatGroupUser                | Group membership                               |
| `chat_group_message_recipients`              | ChatGroupMessageRecipient    | Per-recipient delivery state                   |
| `chat_group_message_removes`                 | ChatGroupMessageRemove       | Per-user message removal                       |
| `chat_block_users`                           | ChatBlockUser                | One-way block between users                    |
| `chat_invitations`                           | ChatInvitation               | One-to-one chat invitation                     |
| `chat_invitation_types`                      | ChatInvitationType           | Variant of an invitation                       |
| `chat_statuses`                              | ChatStatus                   | Presence status of a user                      |
| `communication_send_messages`                           | SendMessage                  | Bulk send-message job                          |
| `communication_contact_messages`                        | ContactMessage               | Public contact-form submission                 |
| `speech_sliders`                             | SpeechSlider                 | Front-page leadership message                  |
| `communication_phone_call_logs`                         | PhoneCallLog                 | Phone-call follow-up record                    |

## Notes

- Every school-scoped table includes `school_id` for multi-tenant
  isolation. The `school_id` is `NOT NULL DEFAULT 1` for the bootstrap
  school.
- Every school-scoped table includes `academic_id` referencing
  `academic_academic_years`. The communication domain rarely uses
  `academic_id` to scope behavior, but the column is preserved for
  reporting consistency.
- Every table includes `created_at`, `updated_at`, `created_by`,
  `updated_by`, `active_status` (where applicable). These are managed
  by the engine's storage adapter.
- Chat-related tables intentionally do not include `school_id` because
  the chat domain relies on user-level identity (from the platform
  layer) and the chat_group / chat_block_user scope is derived from
  the participating users' schools. Consumers MUST add a row-level
  filter on `school_id` for chat queries, derived from the actor's
  session.
- The `notifications` table is a generic inbox table that the
  `Notification` aggregate may write to for compatibility with
  consumer applications that already integrate with that inbox.
