# Communication Domain — Repositories

Repositories are ports (Rust traits). Adapters implement them. The
default adapter targets PostgreSQL; an SQLite adapter is provided for
embedded deployments.

## NoticeRepository

```rust
#[async_trait]
pub trait NoticeRepository: Send + Sync {
    async fn get(&self, id: NoticeId) -> Result<Option<Notice>>;
    async fn list(&self, school: SchoolId, q: NoticeQuery) -> Result<Vec<Notice>>;
    async fn insert(&self, n: &Notice) -> Result<()>;
    async fn update(&self, n: &Notice) -> Result<()>;
    async fn delete(&self, id: NoticeId) -> Result<()>;
    async fn count(&self, school: SchoolId, q: NoticeQuery) -> Result<u64>;
    async fn page(&self, school: SchoolId, q: NoticeQuery, offset: u32, limit: u32) -> Result<Page<Notice>>;
    async fn published_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<Notice>>;
    async fn for_audience(&self, school: SchoolId, audience: &AudienceDescriptor) -> Result<Vec<Notice>>;
}
```

## ComplaintRepository

```rust
#[async_trait]
pub trait ComplaintRepository: Send + Sync {
    async fn get(&self, id: ComplaintId) -> Result<Option<Complaint>>;
    async fn list(&self, school: SchoolId, q: ComplaintQuery) -> Result<Vec<Complaint>>;
    async fn insert(&self, c: &Complaint) -> Result<()>;
    async fn update(&self, c: &Complaint) -> Result<()>;
    async fn open(&self, school: SchoolId) -> Result<Vec<Complaint>>;
    async fn in_progress(&self, school: SchoolId) -> Result<Vec<Complaint>>;
    async fn by_assignee(&self, school: SchoolId, user: UserId) -> Result<Vec<Complaint>>;
    async fn by_type(&self, school: SchoolId, type_id: ComplaintTypeId) -> Result<Vec<Complaint>>;
}
```

## ComplaintTypeRepository

```rust
#[async_trait]
pub trait ComplaintTypeRepository: Send + Sync {
    async fn get(&self, id: ComplaintTypeId) -> Result<Option<ComplaintType>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ComplaintType>>;
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<ComplaintType>>;
    async fn insert(&self, t: &ComplaintType) -> Result<()>;
    async fn update(&self, t: &ComplaintType) -> Result<()>;
    async fn delete(&self, id: ComplaintTypeId) -> Result<()>;
}
```

## NotificationRepository

```rust
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn get(&self, id: NotificationId) -> Result<Option<Notification>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<Notification>>;
    async fn unread_for_user(&self, user: UserId) -> Result<Vec<Notification>>;
    async fn list(&self, school: SchoolId, q: NotificationQuery) -> Result<Vec<Notification>>;
    async fn insert(&self, n: &Notification) -> Result<()>;
    async fn update(&self, n: &Notification) -> Result<()>;
    async fn mark_read(&self, id: NotificationId) -> Result<()>;
    async fn withdraw(&self, id: NotificationId) -> Result<()>;
}
```

## EmailLogRepository

```rust
#[async_trait]
pub trait EmailLogRepository: Send + Sync {
    async fn get(&self, id: EmailLogId) -> Result<Option<EmailLog>>;
    async fn list(&self, school: SchoolId, q: EmailLogQuery) -> Result<Vec<EmailLog>>;
    async fn insert(&self, l: &EmailLog) -> Result<()>;
    async fn by_recipient(&self, school: SchoolId, to: &EmailAddress) -> Result<Vec<EmailLog>>;
    async fn sent_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<EmailLog>>;
}
```

## SmsLogRepository

```rust
#[async_trait]
pub trait SmsLogRepository: Send + Sync {
    async fn get(&self, id: SmsLogId) -> Result<Option<SmsLog>>;
    async fn list(&self, school: SchoolId, q: SmsLogQuery) -> Result<Vec<SmsLog>>;
    async fn insert(&self, l: &SmsLog) -> Result<()>;
    async fn by_recipient(&self, school: SchoolId, to: &PhoneNumber) -> Result<Vec<SmsLog>>;
    async fn sent_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<SmsLog>>;
}
```

## SmsTemplateRepository

```rust
#[async_trait]
pub trait SmsTemplateRepository: Send + Sync {
    async fn get(&self, id: SmsTemplateId) -> Result<Option<SmsTemplate>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SmsTemplate>>;
    async fn find(&self, school: SchoolId, channel: Channel, purpose: &TemplateKey) -> Result<Option<SmsTemplate>>;
    async fn find_enabled(&self, school: SchoolId, channel: Channel, purpose: &TemplateKey) -> Result<Option<SmsTemplate>>;
    async fn insert(&self, t: &SmsTemplate) -> Result<()>;
    async fn update(&self, t: &SmsTemplate) -> Result<()>;
    async fn delete(&self, id: SmsTemplateId) -> Result<()>;
}
```

## EmailSettingRepository

```rust
#[async_trait]
pub trait EmailSettingRepository: Send + Sync {
    async fn get(&self, id: EmailSettingId) -> Result<Option<EmailSetting>>;
    async fn active(&self, school: SchoolId) -> Result<Option<EmailSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<EmailSetting>>;
    async fn insert(&self, s: &EmailSetting) -> Result<()>;
    async fn activate(&self, id: EmailSettingId) -> Result<()>;
    async fn delete(&self, id: EmailSettingId) -> Result<()>;
}
```

## SmsGatewayRepository

```rust
#[async_trait]
pub trait SmsGatewayRepository: Send + Sync {
    async fn get(&self, id: SmsGatewayId) -> Result<Option<SmsGateway>>;
    async fn active(&self, school: SchoolId, gateway_type: GatewayType) -> Result<Option<SmsGateway>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SmsGateway>>;
    async fn insert(&self, g: &SmsGateway) -> Result<()>;
    async fn activate(&self, id: SmsGatewayId) -> Result<()>;
    async fn delete(&self, id: SmsGatewayId) -> Result<()>;
}
```

## NotificationSettingRepository

```rust
#[async_trait]
pub trait NotificationSettingRepository: Send + Sync {
    async fn get(&self, id: NotificationSettingId) -> Result<Option<NotificationSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<NotificationSetting>>;
    async fn find(&self, school: SchoolId, event: &str, destination: Destination) -> Result<Vec<NotificationSetting>>;
    async fn insert(&self, s: &NotificationSetting) -> Result<()>;
    async fn update(&self, s: &NotificationSetting) -> Result<()>;
    async fn delete(&self, id: NotificationSettingId) -> Result<()>;
}
```

## AbsentNotificationTimeSetupRepository

```rust
#[async_trait]
pub trait AbsentNotificationTimeSetupRepository: Send + Sync {
    async fn active(&self, school: SchoolId) -> Result<Option<AbsentNotificationTimeSetup>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<AbsentNotificationTimeSetup>>;
    async fn get(&self, id: AbsentNotificationTimeSetupId) -> Result<Option<AbsentNotificationTimeSetup>>;
    async fn insert(&self, s: &AbsentNotificationTimeSetup) -> Result<()>;
    async fn update(&self, s: &AbsentNotificationTimeSetup) -> Result<()>;
    async fn delete(&self, id: AbsentNotificationTimeSetupId) -> Result<()>;
}
```

## ChatMessageRepository

```rust
#[async_trait]
pub trait ChatMessageRepository: Send + Sync {
    async fn get(&self, id: ChatMessageId) -> Result<Option<ChatMessage>>;
    async fn list_for_conversation(&self, conversation: ChatConversationId) -> Result<Vec<ChatMessage>>;
    async fn insert(&self, m: &ChatMessage) -> Result<()>;
    async fn mark_seen(&self, id: ChatMessageId) -> Result<()>;
    async fn soft_delete(&self, id: ChatMessageId, by: UserId) -> Result<()>;
}
```

## ChatConversationRepository

```rust
#[async_trait]
pub trait ChatConversationRepository: Send + Sync {
    async fn get(&self, id: ChatConversationId) -> Result<Option<ChatConversation>>;
    async fn find(&self, school: SchoolId, a: UserId, b: UserId) -> Result<Option<ChatConversation>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ChatConversation>>;
    async fn insert(&self, c: &ChatConversation) -> Result<()>;
    async fn close(&self, id: ChatConversationId) -> Result<()>;
}
```

## ChatGroupRepository

```rust
#[async_trait]
pub trait ChatGroupRepository: Send + Sync {
    async fn get(&self, id: ChatGroupId) -> Result<Option<ChatGroup>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<ChatGroup>>;
    async fn for_user(&self, user: UserId) -> Result<Vec<ChatGroup>>;
    async fn for_class(&self, class: ClassId, section: Option<SectionId>) -> Result<Vec<ChatGroup>>;
    async fn insert(&self, g: &ChatGroup) -> Result<()>;
    async fn update(&self, g: &ChatGroup) -> Result<()>;
    async fn delete(&self, id: ChatGroupId) -> Result<()>;
}
```

## ChatGroupUserRepository

```rust
#[async_trait]
pub trait ChatGroupUserRepository: Send + Sync {
    async fn get(&self, id: ChatGroupUserId) -> Result<Option<ChatGroupUser>>;
    async fn list_for_group(&self, group: ChatGroupId) -> Result<Vec<ChatGroupUser>>;
    async fn find(&self, group: ChatGroupId, user: UserId) -> Result<Option<ChatGroupUser>>;
    async fn insert(&self, m: &ChatGroupUser) -> Result<()>;
    async fn set_role(&self, id: ChatGroupUserId, role: ChatGroupRole) -> Result<()>;
    async fn remove(&self, id: ChatGroupUserId, by: UserId) -> Result<()>;
}
```

## ChatGroupMessageRecipientRepository

```rust
#[async_trait]
pub trait ChatGroupMessageRecipientRepository: Send + Sync {
    async fn get(&self, id: ChatGroupMessageRecipientId) -> Result<Option<ChatGroupMessageRecipient>>;
    async fn list_for_message(&self, group_message_id: ChatMessageId) -> Result<Vec<ChatGroupMessageRecipient>>;
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ChatGroupMessageRecipient>>;
    async fn insert(&self, r: &ChatGroupMessageRecipient) -> Result<()>;
    async fn mark_read(&self, id: ChatGroupMessageRecipientId) -> Result<()>;
}
```

## ChatBlockUserRepository

```rust
#[async_trait]
pub trait ChatBlockUserRepository: Send + Sync {
    async fn list_for(&self, user: UserId) -> Result<Vec<ChatBlockUser>>;
    async fn find(&self, block_by: UserId, block_to: UserId) -> Result<Option<ChatBlockUser>>;
    async fn insert(&self, b: &ChatBlockUser) -> Result<()>;
    async fn delete(&self, id: ChatBlockUserId) -> Result<()>;
}
```

## ChatInvitationRepository / ChatInvitationTypeRepository / ChatStatusRepository / ChatGroupMessageRemoveRepository

Each follows the same pattern: `get`, `list`, `insert`, `update`,
`delete`, plus domain-specific queries.

## SendMessageRepository

```rust
#[async_trait]
pub trait SendMessageRepository: Send + Sync {
    async fn get(&self, id: SendMessageId) -> Result<Option<SendMessage>>;
    async fn list(&self, school: SchoolId, q: SendMessageQuery) -> Result<Vec<SendMessage>>;
    async fn insert(&self, m: &SendMessage) -> Result<()>;
    async fn update(&self, m: &SendMessage) -> Result<()>;
    async fn dispatched_between(&self, school: SchoolId, from: NaiveDate, to: NaiveDate) -> Result<Vec<SendMessage>>;
}
```

## ContactMessageRepository

```rust
#[async_trait]
pub trait ContactMessageRepository: Send + Sync {
    async fn get(&self, id: ContactMessageId) -> Result<Option<ContactMessage>>;
    async fn list(&self, school: SchoolId, q: ContactMessageQuery) -> Result<Vec<ContactMessage>>;
    async fn unreplied(&self, school: SchoolId) -> Result<Vec<ContactMessage>>;
    async fn insert(&self, m: &ContactMessage) -> Result<()>;
    async fn update(&self, m: &ContactMessage) -> Result<()>;
}
```

## SpeechSliderRepository

```rust
#[async_trait]
pub trait SpeechSliderRepository: Send + Sync {
    async fn get(&self, id: SpeechSliderId) -> Result<Option<SpeechSlider>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<SpeechSlider>>;
    async fn insert(&self, s: &SpeechSlider) -> Result<()>;
    async fn update(&self, s: &SpeechSlider) -> Result<()>;
    async fn delete(&self, id: SpeechSliderId) -> Result<()>;
}
```

## PhoneCallLogRepository

```rust
#[async_trait]
pub trait PhoneCallLogRepository: Send + Sync {
    async fn get(&self, id: PhoneCallLogId) -> Result<Option<PhoneCallLog>>;
    async fn list(&self, school: SchoolId, q: PhoneCallLogQuery) -> Result<Vec<PhoneCallLog>>;
    async fn follow_ups_due(&self, school: SchoolId, on: NaiveDate) -> Result<Vec<PhoneCallLog>>;
    async fn insert(&self, l: &PhoneCallLog) -> Result<()>;
    async fn update_follow_up(&self, id: PhoneCallLogId, next: NaiveDate) -> Result<()>;
}
```

## CustomSmsSettingRepository

```rust
#[async_trait]
pub trait CustomSmsSettingRepository: Send + Sync {
    async fn get(&self, id: CustomSmsSettingId) -> Result<Option<CustomSmsSetting>>;
    async fn for_gateway(&self, gateway: SmsGatewayId) -> Result<Option<CustomSmsSetting>>;
    async fn list(&self, school: SchoolId) -> Result<Vec<CustomSmsSetting>>;
    async fn insert(&self, s: &CustomSmsSetting) -> Result<()>;
    async fn update(&self, s: &CustomSmsSetting) -> Result<()>;
    async fn delete(&self, id: CustomSmsSettingId) -> Result<()>;
}
```

## Indexes (recommended)

The default PostgreSQL adapter documents the following indexes; consumers
should declare them in their migrations:

```sql
CREATE INDEX ix_notice_boards_school_id_publish ON notice_boards (school_id, publish_on);
CREATE INDEX ix_notice_boards_school_id_audience ON notice_boards (school_id, message_to);
CREATE INDEX ix_complaints_school_id_status ON sm_complaints (school_id, active_status);
CREATE INDEX ix_complaints_school_id_type ON sm_complaints (school_id, complaint_type);
CREATE INDEX ix_complaints_school_id_assigned ON sm_complaints (school_id, assigned);
CREATE INDEX ix_notifications_school_id_user ON sm_notifications (school_id, user_id, is_read);
CREATE INDEX ix_notifications_school_id_role ON sm_notifications (school_id, role_id);
CREATE INDEX ix_email_sms_logs_school_id_send_date ON sm_email_sms_logs (school_id, send_date);
CREATE INDEX ix_email_sms_logs_school_id_send_through ON sm_email_sms_logs (school_id, send_through);
CREATE INDEX ix_sms_templates_school_id_channel_purpose ON sms_templates (school_id, type, purpose);
CREATE INDEX ix_email_settings_school_id_active ON sm_email_settings (school_id, active_status);
CREATE INDEX ix_sms_gateways_school_id_type_active ON sm_sms_gateways (school_id, gateway_type, active_status);
CREATE INDEX ix_notification_settings_school_id_event ON sm_notification_settings (school_id, event);
CREATE INDEX ix_chat_conversations_from_to ON chat_conversations (from_id, to_id);
CREATE INDEX ix_chat_conversations_to_from ON chat_conversations (to_id, from_id);
CREATE INDEX ix_chat_groups_school_id_class ON chat_groups (school_id, class_id, section_id);
CREATE INDEX ix_chat_group_users_group_user ON chat_group_users (group_id, user_id);
CREATE INDEX ix_chat_group_message_recipients_user ON chat_group_message_recipients (user_id, read_at);
CREATE INDEX ix_chat_block_users_block_by_to ON chat_block_users (block_by, block_to);
CREATE INDEX ix_chat_invitations_from_to ON chat_invitations (from, to);
CREATE INDEX ix_send_messages_school_id_publish ON sm_send_messages (school_id, publish_on);
CREATE INDEX ix_contact_messages_school_id_view ON sm_contact_messages (school_id, view_status);
CREATE INDEX ix_phone_call_logs_school_id_follow_up ON sm_phone_call_logs (school_id, next_follow_up_date);
```

The `school_id` predicate is mandatory for tenant isolation.
