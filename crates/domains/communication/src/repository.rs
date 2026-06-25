//! # Communication domain repository ports
//!
//! The 27 repository traits that storage adapters implement
//! for the `educore-communication` domain. Each trait takes a
//! `SchoolId` (or operates on a typed identifier that already
//! embeds it) and refuses to return data from another school.
//! Tenant isolation is structural.
//!
//! ## Append-only invariants
//!
//! - `EmailLogRepository` and `SmsLogRepository` are
//!   append-only; no `update()` method. Email/SMS log entries
//!   are immutable records of dispatch and are auditable as-is.
//! - `ChatStatusRepository` and `ChatStatusRecordRepository`
//!   are append-only; the latest entry per user is authoritative
//!   for the current presence status. Historical rows are
//!   retained for audit. `ChatStatusRecordRepository` operates
//!   on the full `ChatStatusRecord` aggregate row (with audit
//!   footer), whereas `ChatStatusRepository` operates on the
//!   `ChatStatus` enum value projection.
//! - `PhoneCallLogRepository` has no generic `update()`
//!   method, only `update_follow_up(id, next)` to bump the
//!   next-follow-up date on an existing call log.
//!
//! All other repositories expose the standard
//! `get` / `list` / `insert` / `update` / `delete` set, with
//! some omitting `delete` where the underlying aggregate is
//! lifecycle-tracked (status-changed) rather than destroyed.
//!
//! Mirrors the library pattern: `#[async_trait]` on every
//! trait, `pub trait XxxRepository: Send + Sync`, object-safety
//! smoke tests in `mod tests`.

#![allow(missing_docs)]
#![allow(unused_imports)]

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_academic::{ClassId, SectionId};
use educore_core::error::Result;
use educore_core::ids::{SchoolId, UserId};

use crate::aggregate::*;
use crate::query::*;
use crate::value_objects::*;

// =============================================================================
// 1. NoticeRepository
// =============================================================================

/// Port for the `Notice` aggregate.
#[async_trait]
pub trait NoticeRepository: Send + Sync {
    /// Fetch a notice by its typed id.
    async fn get(&self, id: NoticeId) -> Result<Option<Notice>>;
    /// List notices for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: NoticeQuery) -> Result<Vec<Notice>>;
    /// Insert a new notice.
    async fn insert(&self, n: &Notice) -> Result<()>;
    /// Update an existing notice.
    async fn update(&self, n: &Notice) -> Result<()>;
    /// Delete (soft) a notice.
    async fn delete(&self, id: NoticeId) -> Result<()>;
    /// Count notices for a school.
    async fn count(&self, school: SchoolId) -> Result<u64>;
    /// Page notices for a school, ordered most-recent first.
    async fn page(&self, school: SchoolId, limit: u32, offset: u32) -> Result<Vec<Notice>>;
    /// List notices published within the inclusive date range
    /// `[from, to]`.
    async fn published_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<Notice>>;
    /// List notices targeted at the given audience.
    async fn for_audience(
        &self,
        school: SchoolId,
        audience: AudienceDescriptor,
    ) -> Result<Vec<Notice>>;
}

// =============================================================================
// 2. ComplaintRepository
// =============================================================================

/// Port for the `Complaint` aggregate.
#[async_trait]
pub trait ComplaintRepository: Send + Sync {
    /// Fetch a complaint by its typed id.
    async fn get(&self, id: ComplaintId) -> Result<Option<Complaint>>;
    /// List complaints for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: ComplaintQuery) -> Result<Vec<Complaint>>;
    /// Insert a new complaint.
    async fn insert(&self, c: &Complaint) -> Result<()>;
    /// Update an existing complaint.
    async fn update(&self, c: &Complaint) -> Result<()>;
    /// List all open complaints for a school.
    async fn open(&self, school: SchoolId) -> Result<Vec<Complaint>>;
    /// List all in-progress complaints for a school.
    async fn in_progress(&self, school: SchoolId) -> Result<Vec<Complaint>>;
    /// List complaints assigned to a given user.
    async fn by_assignee(&self, school: SchoolId, assignee: UserId) -> Result<Vec<Complaint>>;
    /// List complaints of a given complaint type.
    async fn by_type(
        &self,
        school: SchoolId,
        complaint_type: ComplaintTypeId,
    ) -> Result<Vec<Complaint>>;
}

// =============================================================================
// 3. ComplaintTypeRepository
// =============================================================================

/// Port for the `ComplaintType` aggregate (the classification
/// taxonomy of complaints, not the complaint itself).
#[async_trait]
pub trait ComplaintTypeRepository: Send + Sync {
    /// Fetch a complaint type by its typed id.
    async fn get(&self, id: ComplaintTypeId) -> Result<Option<ComplaintType>>;
    /// List all complaint types for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<ComplaintType>>;
    /// Find a complaint type by its display name within a school.
    async fn find_by_name(&self, school: SchoolId, name: &str) -> Result<Option<ComplaintType>>;
    /// Insert a new complaint type.
    async fn insert(&self, t: &ComplaintType) -> Result<()>;
    /// Update an existing complaint type.
    async fn update(&self, t: &ComplaintType) -> Result<()>;
    /// Delete (soft) a complaint type.
    async fn delete(&self, id: ComplaintTypeId) -> Result<()>;
}

// =============================================================================
// 4. NotificationRepository
// =============================================================================

/// Port for the `Notification` aggregate.
#[async_trait]
pub trait NotificationRepository: Send + Sync {
    /// Fetch a notification by its typed id.
    async fn get(&self, id: NotificationId) -> Result<Option<Notification>>;
    /// List notifications addressed to a given user.
    async fn list_for_user(&self, user: UserId) -> Result<Vec<Notification>>;
    /// List unread notifications addressed to a given user.
    async fn unread_for_user(&self, user: UserId) -> Result<Vec<Notification>>;
    /// List notifications for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: NotificationQuery) -> Result<Vec<Notification>>;
    /// Insert a new notification.
    async fn insert(&self, n: &Notification) -> Result<()>;
    /// Update an existing notification.
    async fn update(&self, n: &Notification) -> Result<()>;
    /// Mark a notification as read.
    async fn mark_read(&self, id: NotificationId) -> Result<()>;
    /// Withdraw a notification with the given reason.
    async fn withdraw(&self, id: NotificationId, reason: String) -> Result<()>;
}

// =============================================================================
// 5. EmailLogRepository (APPEND-ONLY — no update method)
// =============================================================================

/// Port for the `EmailLog` aggregate.
///
/// **Append-only.** Email log entries are immutable records of
/// dispatch and must not be edited after the fact. The trait
/// deliberately omits any `update` method.
#[async_trait]
pub trait EmailLogRepository: Send + Sync {
    /// Fetch an email log entry by its typed id.
    async fn get(&self, id: EmailLogId) -> Result<Option<EmailLog>>;
    /// List email log entries for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: EmailLogQuery) -> Result<Vec<EmailLog>>;
    /// Append a new email log entry.
    async fn insert(&self, e: &EmailLog) -> Result<()>;
    /// List email log entries sent to a given recipient address.
    async fn by_recipient(
        &self,
        school: SchoolId,
        recipient: EmailAddress,
    ) -> Result<Vec<EmailLog>>;
    /// List email log entries sent within the inclusive date
    /// range `[from, to]`.
    async fn sent_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<EmailLog>>;
}

// =============================================================================
// 6. SmsLogRepository (APPEND-ONLY — no update method)
// =============================================================================

/// Port for the `SmsLog` aggregate.
///
/// **Append-only.** SMS log entries are immutable records of
/// dispatch and must not be edited after the fact. The trait
/// deliberately omits any `update` method.
#[async_trait]
pub trait SmsLogRepository: Send + Sync {
    /// Fetch an SMS log entry by its typed id.
    async fn get(&self, id: SmsLogId) -> Result<Option<SmsLog>>;
    /// List SMS log entries for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: SmsLogQuery) -> Result<Vec<SmsLog>>;
    /// Append a new SMS log entry.
    async fn insert(&self, s: &SmsLog) -> Result<()>;
    /// List SMS log entries sent to a given recipient phone number.
    async fn by_recipient(&self, school: SchoolId, recipient: PhoneNumber) -> Result<Vec<SmsLog>>;
    /// List SMS log entries sent within the inclusive date
    /// range `[from, to]`.
    async fn sent_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<SmsLog>>;
}

// =============================================================================
// 7. SmsTemplateRepository
// =============================================================================

/// Port for the `SmsTemplate` aggregate.
#[async_trait]
pub trait SmsTemplateRepository: Send + Sync {
    /// Fetch an SMS template by its typed id.
    async fn get(&self, id: SmsTemplateId) -> Result<Option<SmsTemplate>>;
    /// List all SMS templates for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<SmsTemplate>>;
    /// Find an SMS template for the given channel and purpose.
    async fn find(
        &self,
        school: SchoolId,
        channel: Channel,
        purpose: &str,
    ) -> Result<Option<SmsTemplate>>;
    /// List all enabled SMS templates for a school.
    async fn find_enabled(&self, school: SchoolId) -> Result<Vec<SmsTemplate>>;
    /// Insert a new SMS template.
    async fn insert(&self, t: &SmsTemplate) -> Result<()>;
    /// Update an existing SMS template.
    async fn update(&self, t: &SmsTemplate) -> Result<()>;
    /// Delete (soft) an SMS template.
    async fn delete(&self, id: SmsTemplateId) -> Result<()>;
}

// =============================================================================
// 8. EmailSettingRepository
// =============================================================================

/// Port for the `EmailSetting` aggregate.
#[async_trait]
pub trait EmailSettingRepository: Send + Sync {
    /// Fetch an email setting by its typed id.
    async fn get(&self, id: EmailSettingId) -> Result<Option<EmailSetting>>;
    /// Fetch the currently-active email setting for a school.
    /// At most one email setting is active per school.
    async fn active(&self, school: SchoolId) -> Result<Option<EmailSetting>>;
    /// List all email settings for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<EmailSetting>>;
    /// Insert a new email setting.
    async fn insert(&self, s: &EmailSetting) -> Result<()>;
    /// Mark an email setting as the active one for its school.
    /// The adapter is responsible for deactivating any prior
    /// active row.
    async fn activate(&self, id: EmailSettingId) -> Result<()>;
    /// Delete (soft) an email setting.
    async fn delete(&self, id: EmailSettingId) -> Result<()>;
}

// =============================================================================
// 9. SmsGatewayRepository
// =============================================================================

/// Port for the `SmsGateway` aggregate.
#[async_trait]
pub trait SmsGatewayRepository: Send + Sync {
    /// Fetch an SMS gateway by its typed id.
    async fn get(&self, id: SmsGatewayId) -> Result<Option<SmsGateway>>;
    /// Fetch the currently-active SMS gateway for the given
    /// type within a school.
    async fn active(
        &self,
        school: SchoolId,
        gateway_type: GatewayType,
    ) -> Result<Option<SmsGateway>>;
    /// List all SMS gateways for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<SmsGateway>>;
    /// Insert a new SMS gateway.
    async fn insert(&self, g: &SmsGateway) -> Result<()>;
    /// Mark an SMS gateway as the active one of its type for
    /// its school. The adapter deactivates any prior active row
    /// of the same type.
    async fn activate(&self, id: SmsGatewayId) -> Result<()>;
    /// Delete (soft) an SMS gateway.
    async fn delete(&self, id: SmsGatewayId) -> Result<()>;
}

// =============================================================================
// 10. NotificationSettingRepository
// =============================================================================

/// Port for the `NotificationSetting` aggregate.
#[async_trait]
pub trait NotificationSettingRepository: Send + Sync {
    /// Fetch a notification setting by its typed id.
    async fn get(&self, id: NotificationSettingId) -> Result<Option<NotificationSetting>>;
    /// List all notification settings for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<NotificationSetting>>;
    /// Find the notification setting for the given domain event
    /// and destination combination.
    async fn find(
        &self,
        school: SchoolId,
        event: &str,
        destination: Destination,
    ) -> Result<Option<NotificationSetting>>;
    /// Insert a new notification setting.
    async fn insert(&self, s: &NotificationSetting) -> Result<()>;
    /// Update an existing notification setting.
    async fn update(&self, s: &NotificationSetting) -> Result<()>;
    /// Delete (soft) a notification setting.
    async fn delete(&self, id: NotificationSettingId) -> Result<()>;
}

// =============================================================================
// 11. AbsentNotificationTimeSetupRepository
// =============================================================================

/// Port for the `AbsentNotificationTimeSetup` aggregate
/// (the recurring time window during which the engine
/// dispatches absent-notification SMS to guardians).
#[async_trait]
pub trait AbsentNotificationTimeSetupRepository: Send + Sync {
    /// Fetch the currently-active absent-notification time
    /// setup for a school. At most one is active per school.
    async fn active(&self, school: SchoolId) -> Result<Option<AbsentNotificationTimeSetup>>;
    /// List all absent-notification time setups for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<AbsentNotificationTimeSetup>>;
    /// Fetch an absent-notification time setup by its typed id.
    async fn get(
        &self,
        id: AbsentNotificationTimeSetupId,
    ) -> Result<Option<AbsentNotificationTimeSetup>>;
    /// Insert a new absent-notification time setup.
    async fn insert(&self, s: &AbsentNotificationTimeSetup) -> Result<()>;
    /// Update an existing absent-notification time setup.
    async fn update(&self, s: &AbsentNotificationTimeSetup) -> Result<()>;
    /// Delete (soft) an absent-notification time setup.
    async fn delete(&self, id: AbsentNotificationTimeSetupId) -> Result<()>;
}

// =============================================================================
// 12. ChatMessageRepository
// =============================================================================

/// Port for the `ChatMessage` aggregate.
#[async_trait]
pub trait ChatMessageRepository: Send + Sync {
    /// Fetch a chat message by its typed id.
    async fn get(&self, id: ChatMessageId) -> Result<Option<ChatMessage>>;
    /// List all chat messages in a conversation, oldest first.
    async fn list_for_conversation(
        &self,
        conversation: ChatConversationId,
    ) -> Result<Vec<ChatMessage>>;
    /// Insert a new chat message.
    async fn insert(&self, m: &ChatMessage) -> Result<()>;
    /// Mark a chat message as seen.
    async fn mark_seen(&self, id: ChatMessageId) -> Result<()>;
    /// Soft-delete a chat message. The `by` user is recorded
    /// for the audit trail.
    async fn soft_delete(&self, id: ChatMessageId, by: UserId) -> Result<()>;
}

// =============================================================================
// 13. ChatConversationRepository
// =============================================================================

/// Port for the `ChatConversation` aggregate (1-to-1 chat
/// thread between two users).
#[async_trait]
pub trait ChatConversationRepository: Send + Sync {
    /// Fetch a chat conversation by its typed id.
    async fn get(&self, id: ChatConversationId) -> Result<Option<ChatConversation>>;
    /// Find the existing 1-to-1 conversation between users
    /// `a` and `b` within a school (returns at most one row,
    /// regardless of argument order).
    async fn find(
        &self,
        school: SchoolId,
        a: UserId,
        b: UserId,
    ) -> Result<Option<ChatConversation>>;
    /// List all conversations a user participates in.
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ChatConversation>>;
    /// Insert a new chat conversation.
    async fn insert(&self, c: &ChatConversation) -> Result<()>;
    /// Close a chat conversation. The aggregate transitions to
    /// a closed state; rows are retained for audit.
    async fn close(&self, id: ChatConversationId) -> Result<()>;
}

// =============================================================================
// 14. ChatGroupRepository
// =============================================================================

/// Port for the `ChatGroup` aggregate.
#[async_trait]
pub trait ChatGroupRepository: Send + Sync {
    /// Fetch a chat group by its typed id.
    async fn get(&self, id: ChatGroupId) -> Result<Option<ChatGroup>>;
    /// List all chat groups for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<ChatGroup>>;
    /// List all chat groups a user is a member of.
    async fn for_user(&self, user: UserId) -> Result<Vec<ChatGroup>>;
    /// List all chat groups scoped to a given class (and
    /// optionally a section of that class).
    async fn for_class(
        &self,
        school: SchoolId,
        class_id: ClassId,
        section_id: Option<SectionId>,
    ) -> Result<Vec<ChatGroup>>;
    /// Insert a new chat group.
    async fn insert(&self, g: &ChatGroup) -> Result<()>;
    /// Update an existing chat group.
    async fn update(&self, g: &ChatGroup) -> Result<()>;
    /// Delete (soft) a chat group.
    async fn delete(&self, id: ChatGroupId) -> Result<()>;
}

// =============================================================================
// 15. ChatGroupUserRepository
// =============================================================================

/// Port for the `ChatGroupUser` aggregate (the membership row
/// joining a user to a chat group with a role).
#[async_trait]
pub trait ChatGroupUserRepository: Send + Sync {
    /// Fetch a chat group membership by its typed id.
    async fn get(&self, id: ChatGroupUserId) -> Result<Option<ChatGroupUser>>;
    /// List all members of a chat group.
    async fn list_for_group(&self, group: ChatGroupId) -> Result<Vec<ChatGroupUser>>;
    /// Find the membership row for the (group, user) pair.
    async fn find(&self, group: ChatGroupId, user: UserId) -> Result<Option<ChatGroupUser>>;
    /// Insert a new chat group membership.
    async fn insert(&self, m: &ChatGroupUser) -> Result<()>;
    /// Set the role of a user within a chat group.
    async fn set_role(&self, group: ChatGroupId, user: UserId, role: ChatGroupRole) -> Result<()>;
    /// Remove a user from a chat group.
    async fn remove(&self, group: ChatGroupId, user: UserId) -> Result<()>;
}

// =============================================================================
// 16. ChatGroupMessageRecipientRepository
// =============================================================================

/// Port for the `ChatGroupMessageRecipient` aggregate
/// (per-user delivery record for a group chat message).
#[async_trait]
pub trait ChatGroupMessageRecipientRepository: Send + Sync {
    /// Fetch a delivery record by its typed id.
    async fn get(
        &self,
        id: ChatGroupMessageRecipientId,
    ) -> Result<Option<ChatGroupMessageRecipient>>;
    /// List all delivery records for a given group message.
    async fn list_for_message(
        &self,
        message: ChatMessageId,
    ) -> Result<Vec<ChatGroupMessageRecipient>>;
    /// List all delivery records addressed to a given user.
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ChatGroupMessageRecipient>>;
    /// Insert a new delivery record.
    async fn insert(&self, r: &ChatGroupMessageRecipient) -> Result<()>;
    /// Mark a delivery record as read.
    async fn mark_read(&self, id: ChatGroupMessageRecipientId) -> Result<()>;
}

// =============================================================================
// 17. ChatGroupMessageRemoveRepository
// =============================================================================

/// Port for the `ChatGroupMessageRemove` aggregate
/// (per-user tombstone for a group chat message that was
/// removed from that user's view, but remains in the group).
#[async_trait]
pub trait ChatGroupMessageRemoveRepository: Send + Sync {
    /// Fetch a remove-record by its typed id.
    async fn get(&self, id: ChatGroupMessageRemoveId) -> Result<Option<ChatGroupMessageRemove>>;
    /// List all remove-records affecting a given user.
    async fn list_for_user(&self, user: UserId) -> Result<Vec<ChatGroupMessageRemove>>;
    /// Insert a new remove-record.
    async fn insert(&self, r: &ChatGroupMessageRemove) -> Result<()>;
}

// =============================================================================
// 18. ChatBlockUserRepository
// =============================================================================

/// Port for the `ChatBlockUser` aggregate (a block placed by
/// one user against another).
#[async_trait]
pub trait ChatBlockUserRepository: Send + Sync {
    /// List all blocks involving the given user (whether as
    /// blocker or blockee).
    async fn list_for(&self, user: UserId) -> Result<Vec<ChatBlockUser>>;
    /// Find the block placed by `block_by` against `block_to`.
    async fn find(&self, block_by: UserId, block_to: UserId) -> Result<Option<ChatBlockUser>>;
    /// Insert a new block.
    async fn insert(&self, b: &ChatBlockUser) -> Result<()>;
    /// Remove a block by its typed id.
    async fn delete(&self, id: ChatBlockUserId) -> Result<()>;
}

// =============================================================================
// 19. ChatInvitationRepository
// =============================================================================

/// Port for the `ChatInvitation` aggregate (an invitation
/// from one user to another to start a chat conversation or
/// join a group).
#[async_trait]
pub trait ChatInvitationRepository: Send + Sync {
    /// Fetch a chat invitation by its typed id.
    async fn get(&self, id: ChatInvitationId) -> Result<Option<ChatInvitation>>;
    /// List all chat invitations for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<ChatInvitation>>;
    /// Find the chat invitation sent by `from` to `to`.
    async fn find(&self, from: UserId, to: UserId) -> Result<Option<ChatInvitation>>;
    /// Insert a new chat invitation.
    async fn insert(&self, i: &ChatInvitation) -> Result<()>;
    /// Update an existing chat invitation (e.g. on accept /
    /// reject / status transitions).
    async fn update(&self, i: &ChatInvitation) -> Result<()>;
    /// Delete (soft) a chat invitation.
    async fn delete(&self, id: ChatInvitationId) -> Result<()>;
}

// =============================================================================
// 20. ChatInvitationTypeRepository
// =============================================================================

/// Port for the `ChatInvitationType` aggregate (the
/// classification row linking an invitation to a type — e.g.
/// 1-to-1, group, or class-teacher).
#[async_trait]
pub trait ChatInvitationTypeRepository: Send + Sync {
    /// Fetch a chat invitation type by its typed id.
    async fn get(&self, id: ChatInvitationTypeId) -> Result<Option<ChatInvitationType>>;
    /// List all chat invitation types for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<ChatInvitationType>>;
    /// Find the chat invitation type classification row for
    /// a given invitation.
    async fn find_for_invitation(
        &self,
        invitation: ChatInvitationId,
    ) -> Result<Option<ChatInvitationType>>;
    /// Insert a new chat invitation type.
    async fn insert(&self, t: &ChatInvitationType) -> Result<()>;
    /// Delete (soft) a chat invitation type.
    async fn delete(&self, id: ChatInvitationTypeId) -> Result<()>;
}

// =============================================================================
// 21. ChatStatusRepository (APPEND-ONLY — no update method)
// =============================================================================

/// Port for the `ChatStatus` aggregate (a presence row
/// recording a user's chat status at a point in time).
///
/// **Append-only.** The latest row per user is authoritative
/// for the current presence status. Historical rows are
/// retained for audit and replay. The trait deliberately
/// omits any `update` method.
#[async_trait]
pub trait ChatStatusRepository: Send + Sync {
    /// Fetch the most-recent chat status for a user (if any).
    async fn current(&self, user: UserId) -> Result<Option<ChatStatus>>;
    /// List all chat status rows for a user, newest first.
    async fn list_for(&self, user: UserId) -> Result<Vec<ChatStatus>>;
    /// Append a new chat status row.
    async fn insert(&self, s: &ChatStatus) -> Result<()>;
}

// =============================================================================
// 22. SendMessageRepository
// =============================================================================

/// Port for the `SendMessage` aggregate (a one-shot
/// send-to-many dispatch; lifecycle-tracked, no hard delete).
#[async_trait]
pub trait SendMessageRepository: Send + Sync {
    /// Fetch a send message by its typed id.
    async fn get(&self, id: SendMessageId) -> Result<Option<SendMessage>>;
    /// List send messages for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: SendMessageQuery) -> Result<Vec<SendMessage>>;
    /// Insert a new send message.
    async fn insert(&self, m: &SendMessage) -> Result<()>;
    /// Update an existing send message.
    async fn update(&self, m: &SendMessage) -> Result<()>;
    /// List send messages dispatched within the inclusive date
    /// range `[from, to]`.
    async fn dispatched_between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<SendMessage>>;
}

// =============================================================================
// 23. ContactMessageRepository
// =============================================================================

/// Port for the `ContactMessage` aggregate (an inbound
/// message from a public contact form).
#[async_trait]
pub trait ContactMessageRepository: Send + Sync {
    /// Fetch a contact message by its typed id.
    async fn get(&self, id: ContactMessageId) -> Result<Option<ContactMessage>>;
    /// List contact messages for a school matching the typed
    /// query.
    async fn list(&self, school: SchoolId, q: ContactMessageQuery) -> Result<Vec<ContactMessage>>;
    /// List contact messages that have not yet been replied to.
    async fn unreplied(&self, school: SchoolId) -> Result<Vec<ContactMessage>>;
    /// Insert a new contact message.
    async fn insert(&self, m: &ContactMessage) -> Result<()>;
    /// Update an existing contact message (e.g. on view / reply).
    async fn update(&self, m: &ContactMessage) -> Result<()>;
}

// =============================================================================
// 24. SpeechSliderRepository
// =============================================================================

/// Port for the `SpeechSlider` aggregate (a public-facing
/// rotating quote / speech tile on the school's website).
#[async_trait]
pub trait SpeechSliderRepository: Send + Sync {
    /// Fetch a speech slider by its typed id.
    async fn get(&self, id: SpeechSliderId) -> Result<Option<SpeechSlider>>;
    /// List all speech sliders for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<SpeechSlider>>;
    /// Insert a new speech slider.
    async fn insert(&self, s: &SpeechSlider) -> Result<()>;
    /// Update an existing speech slider.
    async fn update(&self, s: &SpeechSlider) -> Result<()>;
    /// Delete (soft) a speech slider.
    async fn delete(&self, id: SpeechSliderId) -> Result<()>;
}

// =============================================================================
// 25. PhoneCallLogRepository (NO update — only update_follow_up)
// =============================================================================

/// Port for the `PhoneCallLog` aggregate (a single inbound,
/// outbound, or missed call recorded by reception).
///
/// The aggregate is append-only for the call body itself.
/// The only mutation permitted is `update_follow_up`, which
/// bumps the `next_follow_up_date` column on an existing row.
/// The trait deliberately omits a generic `update` method.
#[async_trait]
pub trait PhoneCallLogRepository: Send + Sync {
    /// Fetch a phone call log by its typed id.
    async fn get(&self, id: PhoneCallLogId) -> Result<Option<PhoneCallLog>>;
    /// List phone call logs for a school matching the typed query.
    async fn list(&self, school: SchoolId, q: PhoneCallLogQuery) -> Result<Vec<PhoneCallLog>>;
    /// List phone call logs whose `next_follow_up_date` is on
    /// or before `as_of`.
    async fn follow_ups_due(&self, school: SchoolId, as_of: NaiveDate)
        -> Result<Vec<PhoneCallLog>>;
    /// Append a new phone call log.
    async fn insert(&self, c: &PhoneCallLog) -> Result<()>;
    /// Bump the `next_follow_up_date` on an existing phone
    /// call log to `next`.
    async fn update_follow_up(&self, id: PhoneCallLogId, next: NaiveDate) -> Result<()>;
}

// =============================================================================
// 26. CustomSmsSettingRepository
// =============================================================================

/// Port for the `CustomSmsSetting` aggregate (the
/// gateway-specific HTTP request shape used by a custom SMS
/// gateway).
#[async_trait]
pub trait CustomSmsSettingRepository: Send + Sync {
    /// Fetch a custom SMS setting by its typed id.
    async fn get(&self, id: CustomSmsSettingId) -> Result<Option<CustomSmsSetting>>;
    /// List all custom SMS settings bound to a given SMS gateway.
    async fn for_gateway(&self, gateway: SmsGatewayId) -> Result<Vec<CustomSmsSetting>>;
    /// List all custom SMS settings for a school.
    async fn list(&self, school: SchoolId) -> Result<Vec<CustomSmsSetting>>;
    /// Insert a new custom SMS setting.
    async fn insert(&self, s: &CustomSmsSetting) -> Result<()>;
    /// Update an existing custom SMS setting.
    async fn update(&self, s: &CustomSmsSetting) -> Result<()>;
    /// Delete (soft) a custom SMS setting.
    async fn delete(&self, id: CustomSmsSettingId) -> Result<()>;
}

// =============================================================================
// 27. ChatStatusRecordRepository (APPEND-ONLY — no update method)
// =============================================================================

/// Port for the `ChatStatusRecord` aggregate (a per-user
/// presence row recording the user's chat status at a
/// point in time, with a full audit footer).
///
/// **Append-only.** The latest row per user (ordered by
/// `set_at`) is authoritative for the current presence
/// status; older rows are retained for audit and replay.
/// The trait deliberately omits any `update` method,
/// mirroring `ChatStatusRepository` for the underlying
/// `ChatStatus` enum projection.
#[async_trait]
pub trait ChatStatusRecordRepository: Send + Sync {
    /// Fetch the most-recent chat status record for a user
    /// within a school (if any).
    async fn current(&self, school: SchoolId, user: UserId) -> Result<Option<ChatStatusRecord>>;
    /// List all chat status records for a user within a
    /// school, newest first by `set_at`.
    async fn list_for(&self, school: SchoolId, user: UserId) -> Result<Vec<ChatStatusRecord>>;
    /// Append a new chat status record row.
    async fn insert(&self, r: &ChatStatusRecord) -> Result<()>;
}

// =============================================================================
// Object-safety smoke tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    //! The repository traits are object-safe. The smoke tests
    //! assert `Box<dyn XxxRepository>` compiles. If any trait
    //! stops being object-safe (e.g. gains an associated type
    //! or a generic method), the corresponding
    //! `_object_safety_check_*` function will fail to compile.

    use super::*;

    fn _object_safety_check_notice() -> Box<dyn NoticeRepository> {
        struct Impl;
        #[async_trait]
        impl NoticeRepository for Impl {
            async fn get(&self, _id: NoticeId) -> Result<Option<Notice>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId, _q: NoticeQuery) -> Result<Vec<Notice>> {
                unreachable!()
            }
            async fn insert(&self, _n: &Notice) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _n: &Notice) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: NoticeId) -> Result<()> {
                unreachable!()
            }
            async fn count(&self, _school: SchoolId) -> Result<u64> {
                unreachable!()
            }
            async fn page(
                &self,
                _school: SchoolId,
                _limit: u32,
                _offset: u32,
            ) -> Result<Vec<Notice>> {
                unreachable!()
            }
            async fn published_between(
                &self,
                _school: SchoolId,
                _from: NaiveDate,
                _to: NaiveDate,
            ) -> Result<Vec<Notice>> {
                unreachable!()
            }
            async fn for_audience(
                &self,
                _school: SchoolId,
                _audience: AudienceDescriptor,
            ) -> Result<Vec<Notice>> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_complaint() -> Box<dyn ComplaintRepository> {
        struct Impl;
        #[async_trait]
        impl ComplaintRepository for Impl {
            async fn get(&self, _id: ComplaintId) -> Result<Option<Complaint>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId, _q: ComplaintQuery) -> Result<Vec<Complaint>> {
                unreachable!()
            }
            async fn insert(&self, _c: &Complaint) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _c: &Complaint) -> Result<()> {
                unreachable!()
            }
            async fn open(&self, _school: SchoolId) -> Result<Vec<Complaint>> {
                unreachable!()
            }
            async fn in_progress(&self, _school: SchoolId) -> Result<Vec<Complaint>> {
                unreachable!()
            }
            async fn by_assignee(
                &self,
                _school: SchoolId,
                _assignee: UserId,
            ) -> Result<Vec<Complaint>> {
                unreachable!()
            }
            async fn by_type(
                &self,
                _school: SchoolId,
                _complaint_type: ComplaintTypeId,
            ) -> Result<Vec<Complaint>> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_complaint_type() -> Box<dyn ComplaintTypeRepository> {
        struct Impl;
        #[async_trait]
        impl ComplaintTypeRepository for Impl {
            async fn get(&self, _id: ComplaintTypeId) -> Result<Option<ComplaintType>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<ComplaintType>> {
                unreachable!()
            }
            async fn find_by_name(
                &self,
                _school: SchoolId,
                _name: &str,
            ) -> Result<Option<ComplaintType>> {
                unreachable!()
            }
            async fn insert(&self, _t: &ComplaintType) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _t: &ComplaintType) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: ComplaintTypeId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_notification() -> Box<dyn NotificationRepository> {
        struct Impl;
        #[async_trait]
        impl NotificationRepository for Impl {
            async fn get(&self, _id: NotificationId) -> Result<Option<Notification>> {
                unreachable!()
            }
            async fn list_for_user(&self, _user: UserId) -> Result<Vec<Notification>> {
                unreachable!()
            }
            async fn unread_for_user(&self, _user: UserId) -> Result<Vec<Notification>> {
                unreachable!()
            }
            async fn list(
                &self,
                _school: SchoolId,
                _q: NotificationQuery,
            ) -> Result<Vec<Notification>> {
                unreachable!()
            }
            async fn insert(&self, _n: &Notification) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _n: &Notification) -> Result<()> {
                unreachable!()
            }
            async fn mark_read(&self, _id: NotificationId) -> Result<()> {
                unreachable!()
            }
            async fn withdraw(&self, _id: NotificationId, _reason: String) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_email_log() -> Box<dyn EmailLogRepository> {
        struct Impl;
        #[async_trait]
        impl EmailLogRepository for Impl {
            async fn get(&self, _id: EmailLogId) -> Result<Option<EmailLog>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId, _q: EmailLogQuery) -> Result<Vec<EmailLog>> {
                unreachable!()
            }
            async fn insert(&self, _e: &EmailLog) -> Result<()> {
                unreachable!()
            }
            async fn by_recipient(
                &self,
                _school: SchoolId,
                _recipient: EmailAddress,
            ) -> Result<Vec<EmailLog>> {
                unreachable!()
            }
            async fn sent_between(
                &self,
                _school: SchoolId,
                _from: NaiveDate,
                _to: NaiveDate,
            ) -> Result<Vec<EmailLog>> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_sms_log() -> Box<dyn SmsLogRepository> {
        struct Impl;
        #[async_trait]
        impl SmsLogRepository for Impl {
            async fn get(&self, _id: SmsLogId) -> Result<Option<SmsLog>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId, _q: SmsLogQuery) -> Result<Vec<SmsLog>> {
                unreachable!()
            }
            async fn insert(&self, _s: &SmsLog) -> Result<()> {
                unreachable!()
            }
            async fn by_recipient(
                &self,
                _school: SchoolId,
                _recipient: PhoneNumber,
            ) -> Result<Vec<SmsLog>> {
                unreachable!()
            }
            async fn sent_between(
                &self,
                _school: SchoolId,
                _from: NaiveDate,
                _to: NaiveDate,
            ) -> Result<Vec<SmsLog>> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_sms_template() -> Box<dyn SmsTemplateRepository> {
        struct Impl;
        #[async_trait]
        impl SmsTemplateRepository for Impl {
            async fn get(&self, _id: SmsTemplateId) -> Result<Option<SmsTemplate>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<SmsTemplate>> {
                unreachable!()
            }
            async fn find(
                &self,
                _school: SchoolId,
                _channel: Channel,
                _purpose: &str,
            ) -> Result<Option<SmsTemplate>> {
                unreachable!()
            }
            async fn find_enabled(&self, _school: SchoolId) -> Result<Vec<SmsTemplate>> {
                unreachable!()
            }
            async fn insert(&self, _t: &SmsTemplate) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _t: &SmsTemplate) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: SmsTemplateId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_email_setting() -> Box<dyn EmailSettingRepository> {
        struct Impl;
        #[async_trait]
        impl EmailSettingRepository for Impl {
            async fn get(&self, _id: EmailSettingId) -> Result<Option<EmailSetting>> {
                unreachable!()
            }
            async fn active(&self, _school: SchoolId) -> Result<Option<EmailSetting>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<EmailSetting>> {
                unreachable!()
            }
            async fn insert(&self, _s: &EmailSetting) -> Result<()> {
                unreachable!()
            }
            async fn activate(&self, _id: EmailSettingId) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: EmailSettingId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_sms_gateway() -> Box<dyn SmsGatewayRepository> {
        struct Impl;
        #[async_trait]
        impl SmsGatewayRepository for Impl {
            async fn get(&self, _id: SmsGatewayId) -> Result<Option<SmsGateway>> {
                unreachable!()
            }
            async fn active(
                &self,
                _school: SchoolId,
                _gateway_type: GatewayType,
            ) -> Result<Option<SmsGateway>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<SmsGateway>> {
                unreachable!()
            }
            async fn insert(&self, _g: &SmsGateway) -> Result<()> {
                unreachable!()
            }
            async fn activate(&self, _id: SmsGatewayId) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: SmsGatewayId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_notification_setting() -> Box<dyn NotificationSettingRepository> {
        struct Impl;
        #[async_trait]
        impl NotificationSettingRepository for Impl {
            async fn get(&self, _id: NotificationSettingId) -> Result<Option<NotificationSetting>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<NotificationSetting>> {
                unreachable!()
            }
            async fn find(
                &self,
                _school: SchoolId,
                _event: &str,
                _destination: Destination,
            ) -> Result<Option<NotificationSetting>> {
                unreachable!()
            }
            async fn insert(&self, _s: &NotificationSetting) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _s: &NotificationSetting) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: NotificationSettingId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_absent_notification_time_setup(
    ) -> Box<dyn AbsentNotificationTimeSetupRepository> {
        struct Impl;
        #[async_trait]
        impl AbsentNotificationTimeSetupRepository for Impl {
            async fn active(
                &self,
                _school: SchoolId,
            ) -> Result<Option<AbsentNotificationTimeSetup>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<AbsentNotificationTimeSetup>> {
                unreachable!()
            }
            async fn get(
                &self,
                _id: AbsentNotificationTimeSetupId,
            ) -> Result<Option<AbsentNotificationTimeSetup>> {
                unreachable!()
            }
            async fn insert(&self, _s: &AbsentNotificationTimeSetup) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _s: &AbsentNotificationTimeSetup) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: AbsentNotificationTimeSetupId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_message() -> Box<dyn ChatMessageRepository> {
        struct Impl;
        #[async_trait]
        impl ChatMessageRepository for Impl {
            async fn get(&self, _id: ChatMessageId) -> Result<Option<ChatMessage>> {
                unreachable!()
            }
            async fn list_for_conversation(
                &self,
                _conversation: ChatConversationId,
            ) -> Result<Vec<ChatMessage>> {
                unreachable!()
            }
            async fn insert(&self, _m: &ChatMessage) -> Result<()> {
                unreachable!()
            }
            async fn mark_seen(&self, _id: ChatMessageId) -> Result<()> {
                unreachable!()
            }
            async fn soft_delete(&self, _id: ChatMessageId, _by: UserId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_conversation() -> Box<dyn ChatConversationRepository> {
        struct Impl;
        #[async_trait]
        impl ChatConversationRepository for Impl {
            async fn get(&self, _id: ChatConversationId) -> Result<Option<ChatConversation>> {
                unreachable!()
            }
            async fn find(
                &self,
                _school: SchoolId,
                _a: UserId,
                _b: UserId,
            ) -> Result<Option<ChatConversation>> {
                unreachable!()
            }
            async fn list_for_user(&self, _user: UserId) -> Result<Vec<ChatConversation>> {
                unreachable!()
            }
            async fn insert(&self, _c: &ChatConversation) -> Result<()> {
                unreachable!()
            }
            async fn close(&self, _id: ChatConversationId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_group() -> Box<dyn ChatGroupRepository> {
        struct Impl;
        #[async_trait]
        impl ChatGroupRepository for Impl {
            async fn get(&self, _id: ChatGroupId) -> Result<Option<ChatGroup>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<ChatGroup>> {
                unreachable!()
            }
            async fn for_user(&self, _user: UserId) -> Result<Vec<ChatGroup>> {
                unreachable!()
            }
            async fn for_class(
                &self,
                _school: SchoolId,
                _class_id: ClassId,
                _section_id: Option<SectionId>,
            ) -> Result<Vec<ChatGroup>> {
                unreachable!()
            }
            async fn insert(&self, _g: &ChatGroup) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _g: &ChatGroup) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: ChatGroupId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_group_user() -> Box<dyn ChatGroupUserRepository> {
        struct Impl;
        #[async_trait]
        impl ChatGroupUserRepository for Impl {
            async fn get(&self, _id: ChatGroupUserId) -> Result<Option<ChatGroupUser>> {
                unreachable!()
            }
            async fn list_for_group(&self, _group: ChatGroupId) -> Result<Vec<ChatGroupUser>> {
                unreachable!()
            }
            async fn find(
                &self,
                _group: ChatGroupId,
                _user: UserId,
            ) -> Result<Option<ChatGroupUser>> {
                unreachable!()
            }
            async fn insert(&self, _m: &ChatGroupUser) -> Result<()> {
                unreachable!()
            }
            async fn set_role(
                &self,
                _group: ChatGroupId,
                _user: UserId,
                _role: ChatGroupRole,
            ) -> Result<()> {
                unreachable!()
            }
            async fn remove(&self, _group: ChatGroupId, _user: UserId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_group_message_recipient(
    ) -> Box<dyn ChatGroupMessageRecipientRepository> {
        struct Impl;
        #[async_trait]
        impl ChatGroupMessageRecipientRepository for Impl {
            async fn get(
                &self,
                _id: ChatGroupMessageRecipientId,
            ) -> Result<Option<ChatGroupMessageRecipient>> {
                unreachable!()
            }
            async fn list_for_message(
                &self,
                _message: ChatMessageId,
            ) -> Result<Vec<ChatGroupMessageRecipient>> {
                unreachable!()
            }
            async fn list_for_user(&self, _user: UserId) -> Result<Vec<ChatGroupMessageRecipient>> {
                unreachable!()
            }
            async fn insert(&self, _r: &ChatGroupMessageRecipient) -> Result<()> {
                unreachable!()
            }
            async fn mark_read(&self, _id: ChatGroupMessageRecipientId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_group_message_remove() -> Box<dyn ChatGroupMessageRemoveRepository>
    {
        struct Impl;
        #[async_trait]
        impl ChatGroupMessageRemoveRepository for Impl {
            async fn get(
                &self,
                _id: ChatGroupMessageRemoveId,
            ) -> Result<Option<ChatGroupMessageRemove>> {
                unreachable!()
            }
            async fn list_for_user(&self, _user: UserId) -> Result<Vec<ChatGroupMessageRemove>> {
                unreachable!()
            }
            async fn insert(&self, _r: &ChatGroupMessageRemove) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_block_user() -> Box<dyn ChatBlockUserRepository> {
        struct Impl;
        #[async_trait]
        impl ChatBlockUserRepository for Impl {
            async fn list_for(&self, _user: UserId) -> Result<Vec<ChatBlockUser>> {
                unreachable!()
            }
            async fn find(
                &self,
                _block_by: UserId,
                _block_to: UserId,
            ) -> Result<Option<ChatBlockUser>> {
                unreachable!()
            }
            async fn insert(&self, _b: &ChatBlockUser) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: ChatBlockUserId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_invitation() -> Box<dyn ChatInvitationRepository> {
        struct Impl;
        #[async_trait]
        impl ChatInvitationRepository for Impl {
            async fn get(&self, _id: ChatInvitationId) -> Result<Option<ChatInvitation>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<ChatInvitation>> {
                unreachable!()
            }
            async fn find(&self, _from: UserId, _to: UserId) -> Result<Option<ChatInvitation>> {
                unreachable!()
            }
            async fn insert(&self, _i: &ChatInvitation) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _i: &ChatInvitation) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: ChatInvitationId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_invitation_type() -> Box<dyn ChatInvitationTypeRepository> {
        struct Impl;
        #[async_trait]
        impl ChatInvitationTypeRepository for Impl {
            async fn get(&self, _id: ChatInvitationTypeId) -> Result<Option<ChatInvitationType>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<ChatInvitationType>> {
                unreachable!()
            }
            async fn find_for_invitation(
                &self,
                _invitation: ChatInvitationId,
            ) -> Result<Option<ChatInvitationType>> {
                unreachable!()
            }
            async fn insert(&self, _t: &ChatInvitationType) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: ChatInvitationTypeId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_status() -> Box<dyn ChatStatusRepository> {
        struct Impl;
        #[async_trait]
        impl ChatStatusRepository for Impl {
            async fn current(&self, _user: UserId) -> Result<Option<ChatStatus>> {
                unreachable!()
            }
            async fn list_for(&self, _user: UserId) -> Result<Vec<ChatStatus>> {
                unreachable!()
            }
            async fn insert(&self, _s: &ChatStatus) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_send_message() -> Box<dyn SendMessageRepository> {
        struct Impl;
        #[async_trait]
        impl SendMessageRepository for Impl {
            async fn get(&self, _id: SendMessageId) -> Result<Option<SendMessage>> {
                unreachable!()
            }
            async fn list(
                &self,
                _school: SchoolId,
                _q: SendMessageQuery,
            ) -> Result<Vec<SendMessage>> {
                unreachable!()
            }
            async fn insert(&self, _m: &SendMessage) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _m: &SendMessage) -> Result<()> {
                unreachable!()
            }
            async fn dispatched_between(
                &self,
                _school: SchoolId,
                _from: NaiveDate,
                _to: NaiveDate,
            ) -> Result<Vec<SendMessage>> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_contact_message() -> Box<dyn ContactMessageRepository> {
        struct Impl;
        #[async_trait]
        impl ContactMessageRepository for Impl {
            async fn get(&self, _id: ContactMessageId) -> Result<Option<ContactMessage>> {
                unreachable!()
            }
            async fn list(
                &self,
                _school: SchoolId,
                _q: ContactMessageQuery,
            ) -> Result<Vec<ContactMessage>> {
                unreachable!()
            }
            async fn unreplied(&self, _school: SchoolId) -> Result<Vec<ContactMessage>> {
                unreachable!()
            }
            async fn insert(&self, _m: &ContactMessage) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _m: &ContactMessage) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_speech_slider() -> Box<dyn SpeechSliderRepository> {
        struct Impl;
        #[async_trait]
        impl SpeechSliderRepository for Impl {
            async fn get(&self, _id: SpeechSliderId) -> Result<Option<SpeechSlider>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<SpeechSlider>> {
                unreachable!()
            }
            async fn insert(&self, _s: &SpeechSlider) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _s: &SpeechSlider) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: SpeechSliderId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_phone_call_log() -> Box<dyn PhoneCallLogRepository> {
        struct Impl;
        #[async_trait]
        impl PhoneCallLogRepository for Impl {
            async fn get(&self, _id: PhoneCallLogId) -> Result<Option<PhoneCallLog>> {
                unreachable!()
            }
            async fn list(
                &self,
                _school: SchoolId,
                _q: PhoneCallLogQuery,
            ) -> Result<Vec<PhoneCallLog>> {
                unreachable!()
            }
            async fn follow_ups_due(
                &self,
                _school: SchoolId,
                _as_of: NaiveDate,
            ) -> Result<Vec<PhoneCallLog>> {
                unreachable!()
            }
            async fn insert(&self, _c: &PhoneCallLog) -> Result<()> {
                unreachable!()
            }
            async fn update_follow_up(&self, _id: PhoneCallLogId, _next: NaiveDate) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_custom_sms_setting() -> Box<dyn CustomSmsSettingRepository> {
        struct Impl;
        #[async_trait]
        impl CustomSmsSettingRepository for Impl {
            async fn get(&self, _id: CustomSmsSettingId) -> Result<Option<CustomSmsSetting>> {
                unreachable!()
            }
            async fn for_gateway(&self, _gateway: SmsGatewayId) -> Result<Vec<CustomSmsSetting>> {
                unreachable!()
            }
            async fn list(&self, _school: SchoolId) -> Result<Vec<CustomSmsSetting>> {
                unreachable!()
            }
            async fn insert(&self, _s: &CustomSmsSetting) -> Result<()> {
                unreachable!()
            }
            async fn update(&self, _s: &CustomSmsSetting) -> Result<()> {
                unreachable!()
            }
            async fn delete(&self, _id: CustomSmsSettingId) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    fn _object_safety_check_chat_status_record() -> Box<dyn ChatStatusRecordRepository> {
        struct Impl;
        #[async_trait]
        impl ChatStatusRecordRepository for Impl {
            async fn current(
                &self,
                _school: SchoolId,
                _user: UserId,
            ) -> Result<Option<ChatStatusRecord>> {
                unreachable!()
            }
            async fn list_for(
                &self,
                _school: SchoolId,
                _user: UserId,
            ) -> Result<Vec<ChatStatusRecord>> {
                unreachable!()
            }
            async fn insert(&self, _r: &ChatStatusRecord) -> Result<()> {
                unreachable!()
            }
        }
        Box::new(Impl)
    }

    #[test]
    fn repository_traits_are_object_safe() {
        // If any trait were not object-safe, these fn definitions
        // would fail to compile.
        let _a: Box<dyn NoticeRepository> = _object_safety_check_notice();
        let _b: Box<dyn ComplaintRepository> = _object_safety_check_complaint();
        let _c: Box<dyn ComplaintTypeRepository> = _object_safety_check_complaint_type();
        let _d: Box<dyn NotificationRepository> = _object_safety_check_notification();
        let _e: Box<dyn EmailLogRepository> = _object_safety_check_email_log();
        let _f: Box<dyn SmsLogRepository> = _object_safety_check_sms_log();
        let _g: Box<dyn SmsTemplateRepository> = _object_safety_check_sms_template();
        let _h: Box<dyn EmailSettingRepository> = _object_safety_check_email_setting();
        let _i: Box<dyn SmsGatewayRepository> = _object_safety_check_sms_gateway();
        let _j: Box<dyn NotificationSettingRepository> =
            _object_safety_check_notification_setting();
        let _k: Box<dyn AbsentNotificationTimeSetupRepository> =
            _object_safety_check_absent_notification_time_setup();
        let _l: Box<dyn ChatMessageRepository> = _object_safety_check_chat_message();
        let _m: Box<dyn ChatConversationRepository> = _object_safety_check_chat_conversation();
        let _n: Box<dyn ChatGroupRepository> = _object_safety_check_chat_group();
        let _o: Box<dyn ChatGroupUserRepository> = _object_safety_check_chat_group_user();
        let _p: Box<dyn ChatGroupMessageRecipientRepository> =
            _object_safety_check_chat_group_message_recipient();
        let _q: Box<dyn ChatGroupMessageRemoveRepository> =
            _object_safety_check_chat_group_message_remove();
        let _r: Box<dyn ChatBlockUserRepository> = _object_safety_check_chat_block_user();
        let _s: Box<dyn ChatInvitationRepository> = _object_safety_check_chat_invitation();
        let _t: Box<dyn ChatInvitationTypeRepository> = _object_safety_check_chat_invitation_type();
        let _u: Box<dyn ChatStatusRepository> = _object_safety_check_chat_status();
        let _v: Box<dyn SendMessageRepository> = _object_safety_check_send_message();
        let _w: Box<dyn ContactMessageRepository> = _object_safety_check_contact_message();
        let _x: Box<dyn SpeechSliderRepository> = _object_safety_check_speech_slider();
        let _y: Box<dyn PhoneCallLogRepository> = _object_safety_check_phone_call_log();
        let _z: Box<dyn CustomSmsSettingRepository> = _object_safety_check_custom_sms_setting();
        let _aa: Box<dyn ChatStatusRecordRepository> = _object_safety_check_chat_status_record();
    }
}
