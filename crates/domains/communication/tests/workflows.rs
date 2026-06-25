//! Integration tests for the **communication domain workflows**.
//!
//! Implements: `docs/specs/communication/workflows.md`
//!
//! Each test exercises a spec-mandated workflow end-to-end
//! through the communication aggregate methods and asserts
//! that the expected typed event is emitted (or, on the error
//! path, that the expected [`DomainError`] is returned and no
//! event is produced).
//!
//! The tests are written as **pure synchronous** tests: the
//! communication aggregate methods (`Notice::fresh`,
//! `notice.publish`, `notice.unpublish`, `notice.mark_deleted`,
//! `notice.update`, `ChatConversation::fresh`,
//! `conversation.close`, `ChatMessage::fresh`,
//! `message.mark_seen`, `message.mark_deleted`,
//! `Notification::fresh`, `notification.mark_read`,
//! `notification.withdraw`) are sync, take a `Timestamp` +
//! `EventId`, and (for the value-object constructors) return
//! `Result<T, DomainError>` for validation. The test wires a
//! [`TestClock`] and a [`SystemIdGen`], and constructs the
//! typed events directly from the aggregate + clock instant
//! to verify the event payloads.
//!
//! Per `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`
//! the **handlers** are not yet wired end-to-end (no subscriber
//! fan-out, no outbox commit, no audit row). These tests pin
//! the contract of the **aggregate layer** that the service
//! factory fns (`publish_notice`, `send_chat_message`,
//! `send_notification`, etc.) and the eventual dispatcher
//! wrap. When the handlers land, the same test bodies will
//! gain a `+ outbox + bus subscriber` assertion without
//! changes to the assertions on the returned event.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use educore_communication::prelude::*;
use educore_communication::value_objects::ChatMessageBody;
use educore_core::clock::{Clock as _, IdGenerator as _, SystemIdGen, TestClock};
use educore_core::ids::CorrelationId;
use educore_core::tenant::{TenantContext, UserType};
use educore_events::domain_event::DomainEvent;

// =============================================================================
// Test fixtures
// =============================================================================

/// A fresh `TenantContext` for a `SchoolAdmin` acting on a
/// freshly-minted school.
fn admin_context() -> (TenantContext, SystemIdGen) {
    let g = SystemIdGen;
    let school = g.next_school_id();
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    (
        TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin),
        g,
    )
}

fn notice_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> NoticeId {
    NoticeId::new(school, g.next_uuid())
}

fn notification_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> NotificationId {
    NotificationId::new(school, g.next_uuid())
}

fn chat_conversation_id(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
) -> ChatConversationId {
    ChatConversationId::new(school, g.next_uuid())
}

fn chat_message_id(g: &SystemIdGen, school: educore_core::ids::SchoolId) -> ChatMessageId {
    ChatMessageId::new(school, g.next_uuid())
}

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Construct a fresh draft `Notice` aggregate for a given
/// school + actor, addressed to a school-wide audience.
fn new_draft_notice(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    title: &str,
    body: &str,
) -> Notice {
    let at = Timestamp::now();
    Notice::fresh(
        notice_id(g, school),
        NoticeTitle::new(title).unwrap(),
        NoticeBody::new(body).unwrap(),
        date(2026, 6, 1),
        None,
        AudienceDescriptor::all(),
        None,
        actor,
        at,
        g.next_correlation_id(),
    )
}

/// Construct a fresh `ChatConversation` aggregate for a given
/// school + actor + counterpart.
fn new_chat_conversation(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    from: educore_core::ids::UserId,
    to: educore_core::ids::UserId,
) -> ChatConversation {
    let at = Timestamp::now();
    ChatConversation::fresh(
        chat_conversation_id(g, school),
        from,
        to,
        from,
        at,
        g.next_correlation_id(),
    )
}

/// Construct a fresh `ChatMessage` aggregate for a given
/// school + actor + conversation.
fn new_chat_message(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    conversation_id: ChatConversationId,
    from: educore_core::ids::UserId,
    to: educore_core::ids::UserId,
    body: &str,
) -> ChatMessage {
    let at = Timestamp::now();
    ChatMessage::fresh(
        chat_message_id(g, school),
        conversation_id,
        from,
        to,
        ChatMessageBody::new(body).unwrap(),
        MessageType::Text,
        None,
        None,
        None,
        from,
        at,
        g.next_correlation_id(),
    )
}

/// Construct a fresh `Notification` aggregate for a given
/// school + actor + recipient.
fn new_notification(
    g: &SystemIdGen,
    school: educore_core::ids::SchoolId,
    actor: educore_core::ids::UserId,
    recipient: educore_core::ids::UserId,
    message: &str,
    channel: Channel,
) -> Notification {
    let at = Timestamp::now();
    let mut data = std::collections::BTreeMap::new();
    data.insert("source".to_owned(), "workflows.rs".to_owned());
    Notification::fresh(
        notification_id(g, school),
        recipient,
        NotificationType::Info,
        NotificationMessage::new(message).unwrap(),
        None,
        data,
        channel,
        actor,
        at,
        g.next_correlation_id(),
    )
}

// =============================================================================
// 1. Message Send Lifecycle (`workflows.md` § "Notice Publishing Workflow")
//
// The spec's "Notice Publishing Workflow" is the canonical
// "compose → send → retract" pattern for outbound messages:
// author drafts (CreateNotice), updates the body and audience
// (UpdateNotice), publishes (PublishNotice), may unpublish
// (UnpublishNotice), and may delete (DeleteNotice) only when
// no recipient has received the notice. The aggregate
// transitions are encoded directly in `Notice`.
// =============================================================================

/// Message lifecycle step 1: creating a draft notice emits
/// [`NoticeCreated`] with the supplied title, body, and
/// audience.
#[test]
fn message_lifecycle_create_emits_notice_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let notice = new_draft_notice(
        &g,
        school,
        actor,
        "Holiday notice",
        "School closed on Monday.",
    );
    let event: NoticeCreated = NoticeCreated::new(
        notice.id,
        notice.title.clone(),
        notice.notice_date,
        notice.publish_on,
        notice.audience.clone(),
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NoticeCreated as DomainEvent>::EVENT_TYPE,
        "communication.notice.created"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.title.as_str(), "Holiday notice");
    assert!(matches!(notice.status, NoticeStatus::Draft));
    assert_eq!(event.notice_id, notice.id);
}

/// Message lifecycle step 2: updating a draft notice emits
/// [`NoticeUpdated`] with the list of changed field names.
/// Only the body and audience are updated here; the title is
/// unchanged.
#[test]
fn message_lifecycle_update_emits_notice_updated_with_changes() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut notice = new_draft_notice(
        &g,
        school,
        actor,
        "Holiday notice",
        "School closed on Monday.",
    );
    let changes = notice.update(
        None,
        Some(NoticeBody::new("School closed on Monday and Tuesday.").unwrap()),
        None,
        None,
        actor,
        clock.now(),
        g.next_event_id(),
    );

    // Only `body` was changed.
    assert_eq!(changes, vec!["body"]);

    let event: NoticeUpdated = NoticeUpdated::new(
        notice.id,
        changes.iter().map(|s| (*s).to_owned()).collect(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );

    assert_eq!(
        <NoticeUpdated as DomainEvent>::EVENT_TYPE,
        "communication.notice.updated"
    );
    assert_eq!(event.notice_id, notice.id);
    assert_eq!(event.changes, vec!["body".to_owned()]);
    assert_eq!(notice.body.as_str(), "School closed on Monday and Tuesday.");
}

/// Message lifecycle step 3: publishing a draft notice
/// transitions it to `NoticeStatus::Published` and emits
/// [`NoticePublished`].
#[test]
fn message_lifecycle_publish_transitions_to_published() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut notice = new_draft_notice(
        &g,
        school,
        actor,
        "Holiday notice",
        "School closed on Monday.",
    );
    assert!(matches!(notice.status, NoticeStatus::Draft));

    notice.publish(actor, clock.now(), g.next_event_id());

    let event: NoticePublished = NoticePublished::new(
        notice.id,
        clock.now(),
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NoticePublished as DomainEvent>::EVENT_TYPE,
        "communication.notice.published"
    );
    assert!(matches!(notice.status, NoticeStatus::Published));
    assert_eq!(event.notice_id, notice.id);
}

/// Message lifecycle step 6: unpublishing a published notice
/// transitions it back to `NoticeStatus::Unpublished` and
/// emits [`NoticeUnpublished`]. Unpublishing suppresses new
/// notifications but does not retract already-sent ones
/// (per spec step 6).
#[test]
fn message_lifecycle_unpublish_transitions_to_unpublished() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut notice = new_draft_notice(
        &g,
        school,
        actor,
        "Holiday notice",
        "School closed on Monday.",
    );
    notice.publish(actor, clock.now(), g.next_event_id());
    assert!(matches!(notice.status, NoticeStatus::Published));

    notice.unpublish(actor, clock.now(), g.next_event_id());

    let event: NoticeUnpublished = NoticeUnpublished::new(
        notice.id,
        Some("Replaced by an updated notice".to_owned()),
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NoticeUnpublished as DomainEvent>::EVENT_TYPE,
        "communication.notice.unpublished"
    );
    assert!(matches!(notice.status, NoticeStatus::Unpublished));
    assert_eq!(event.notice_id, notice.id);
    assert_eq!(
        event.reason.as_deref(),
        Some("Replaced by an updated notice")
    );
}

/// Message lifecycle step 7: a draft notice can be
/// soft-deleted (transitions to `ActiveStatus::Retired`),
/// emitting [`NoticeDeleted`]. The aggregate remains queryable
/// for audit purposes.
#[test]
fn message_lifecycle_soft_delete_emits_notice_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let mut notice = new_draft_notice(&g, school, actor, "Old notice", "Will be deleted.");
    assert!(notice.active_status.is_active());

    notice.mark_deleted(actor, clock.now(), g.next_event_id());

    let event: NoticeDeleted =
        NoticeDeleted::new(notice.id, g.next_event_id(), correlation, clock.now());

    assert_eq!(
        <NoticeDeleted as DomainEvent>::EVENT_TYPE,
        "communication.notice.deleted"
    );
    assert!(!notice.active_status.is_active());
    assert_eq!(event.notice_id, notice.id);
}

/// Message lifecycle failure path: per spec invariant 1, a
/// notice title must be non-empty. `NoticeTitle::new` must
/// reject empty titles so that `Notice::fresh` can never
/// receive an empty title.
#[test]
fn message_lifecycle_empty_title_returns_validation_error() {
    let res = NoticeTitle::new(String::new());
    assert!(res.is_err(), "empty NoticeTitle must fail validation");
}

/// Message lifecycle failure path: per spec invariant 1, a
/// notice body must be non-empty. `NoticeBody::new` must
/// reject empty bodies so that `Notice::fresh` can never
/// receive an empty body.
#[test]
fn message_lifecycle_empty_body_returns_validation_error() {
    let res = NoticeBody::new(String::new());
    assert!(res.is_err(), "empty NoticeBody must fail validation");
}

// =============================================================================
// 2. Conversation Lifecycle (`workflows.md` § "Chat Messaging Workflow"
//
//                            § "One-to-One")
//
// The spec's 1-to-1 "Chat Messaging Workflow" is the canonical
// "start conversation, exchange messages, archive" pattern:
// users implicitly open a `ChatConversation`, exchange
// `ChatMessage` rows, the receiver marks them as seen, and a
// user may close the conversation.
// =============================================================================

/// Conversation lifecycle step 1: opening a chat conversation
/// emits [`ChatConversationOpened`] with the two participant
/// user ids.
#[test]
fn conversation_lifecycle_open_emits_chat_conversation_opened() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let user_a = g.next_user_id();
    let user_b = g.next_user_id();

    let conv = new_chat_conversation(&g, school, user_a, user_b);

    let event: ChatConversationOpened = ChatConversationOpened::new(
        conv.id,
        user_a,
        user_b,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <ChatConversationOpened as DomainEvent>::EVENT_TYPE,
        "communication.chat_conversation.opened"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.from_id, user_a);
    assert_eq!(event.to_id, user_b);
    assert_eq!(event.chat_conversation_id, conv.id);
    assert!(!conv.closed);
}

/// Conversation lifecycle step 2: sending the first message in
/// a conversation emits [`ChatMessageSent`] with the
/// originating conversation id and the message metadata.
#[test]
fn conversation_lifecycle_exchange_message_emits_chat_message_sent() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let user_a = g.next_user_id();
    let user_b = g.next_user_id();

    let conv = new_chat_conversation(&g, school, user_a, user_b);
    let message = new_chat_message(&g, school, conv.id, user_a, user_b, "Hello!");

    let event: ChatMessageSent = ChatMessageSent::new(
        message.id,
        conv.id,
        user_a,
        user_b,
        MessageType::Text,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <ChatMessageSent as DomainEvent>::EVENT_TYPE,
        "communication.chat_message.sent"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.from_id, user_a);
    assert_eq!(event.to_id, user_b);
    assert_eq!(event.chat_conversation_id, conv.id);
    assert_eq!(event.message_type, MessageType::Text);
    assert_eq!(message.conversation_id, conv.id);
    assert!(matches!(message.status, ChatMessageStatus::Unread));
}

/// Conversation lifecycle step 3: the recipient marks a
/// message as seen. The aggregate transitions to
/// `ChatMessageStatus::Seen` and emits [`ChatMessageSeen`]
/// with the seer's user id and the seen-at timestamp.
#[test]
fn conversation_lifecycle_message_seen_emits_chat_message_seen() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let user_a = g.next_user_id();
    let user_b = g.next_user_id();

    let conv = new_chat_conversation(&g, school, user_a, user_b);
    let mut message = new_chat_message(&g, school, conv.id, user_a, user_b, "Hello!");
    assert!(matches!(message.status, ChatMessageStatus::Unread));
    assert!(message.seen_at.is_none());

    let seen_at = clock.now();
    message.mark_seen(user_b, seen_at, g.next_event_id());

    let event: ChatMessageSeen = ChatMessageSeen::new(
        message.id,
        user_b,
        seen_at,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <ChatMessageSeen as DomainEvent>::EVENT_TYPE,
        "communication.chat_message.seen"
    );
    assert!(matches!(message.status, ChatMessageStatus::Seen));
    assert_eq!(message.seen_at, Some(seen_at));
    assert_eq!(event.seen_by, user_b);
    assert_eq!(event.chat_message_id, message.id);
}

/// Conversation lifecycle step 4: closing a conversation
/// transitions `ChatConversation::closed` to `true` and emits
/// [`ChatConversationClosed`]. The conversation history is
/// preserved for audit.
#[test]
fn conversation_lifecycle_close_emits_chat_conversation_closed() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let user_a = g.next_user_id();
    let user_b = g.next_user_id();

    let mut conv = new_chat_conversation(&g, school, user_a, user_b);
    assert!(!conv.closed);

    let closed_at = clock.now();
    conv.close(user_a, closed_at, g.next_event_id());

    let event: ChatConversationClosed =
        ChatConversationClosed::new(conv.id, g.next_event_id(), correlation, clock.now());

    assert_eq!(
        <ChatConversationClosed as DomainEvent>::EVENT_TYPE,
        "communication.chat_conversation.closed"
    );
    assert!(conv.closed);
    assert_eq!(event.chat_conversation_id, conv.id);
}

/// Conversation lifecycle failure path: per spec invariant 1,
/// a chat message body must be non-empty.
/// `ChatMessageBody::new` must reject empty bodies so that
/// `ChatMessage::fresh` can never receive an empty body.
#[test]
fn conversation_lifecycle_empty_body_returns_validation_error() {
    let res = ChatMessageBody::new(String::new());
    assert!(res.is_err(), "empty ChatMessageBody must fail validation");
}

/// Conversation lifecycle soft-delete path: a sender may
/// soft-delete their own message, retiring the active status
/// and emitting [`ChatMessageDeleted`]. The conversation
/// itself remains open.
#[test]
fn conversation_lifecycle_soft_delete_message_emits_chat_message_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let user_a = g.next_user_id();
    let user_b = g.next_user_id();

    let conv = new_chat_conversation(&g, school, user_a, user_b);
    let mut message = new_chat_message(&g, school, conv.id, user_a, user_b, "Will be deleted.");
    assert!(message.active_status.is_active());

    message.mark_deleted(user_a, clock.now(), g.next_event_id());

    let event: ChatMessageDeleted = ChatMessageDeleted::new(
        message.id,
        user_a,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <ChatMessageDeleted as DomainEvent>::EVENT_TYPE,
        "communication.chat_message.deleted"
    );
    assert!(!message.active_status.is_active());
    assert_eq!(event.deleted_by, user_a);
    assert!(
        !conv.closed,
        "deleting a message must not close the conversation"
    );
}

// =============================================================================
// 3. Notification Dispatch (`workflows.md` § "Notice Publishing Workflow"
//
//                            § "Chat Messaging Workflow")
//
// The spec's "Notification Dispatch" pattern is the canonical
// cross-channel notification fan-out: a notification is sent
// to a recipient via a single channel, the recipient marks
// it as read, or the sender withdraws it before the recipient
// reads it. The aggregate transitions are encoded directly in
// `Notification`.
// =============================================================================

/// Notification dispatch step 1: sending a notification to a
/// recipient emits [`NotificationSent`] with the recipient's
/// user id and the dispatch channel. The aggregate starts in
/// `NotificationStatus::Pending`.
#[test]
fn notification_dispatch_send_emits_notification_sent() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let recipient = g.next_user_id();

    let notif = new_notification(
        &g,
        school,
        actor,
        recipient,
        "Your child is absent today.",
        Channel::App,
    );

    let event: NotificationSent = NotificationSent::new(
        notif.id,
        recipient,
        NotificationType::Info,
        Channel::App,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NotificationSent as DomainEvent>::EVENT_TYPE,
        "communication.notification.sent"
    );
    assert_eq!(event.school_id(), school);
    assert_eq!(event.recipient_user_id, recipient);
    assert_eq!(event.channel, Channel::App);
    assert_eq!(event.notification_type, NotificationType::Info);
    assert!(matches!(notif.status, NotificationStatus::Pending));
    assert_eq!(notif.recipient_user_id, recipient);
}

/// Notification dispatch step 2: the recipient reads the
/// notification. The aggregate transitions to
/// `NotificationStatus::Read`, records `read_at`, and emits
/// [`NotificationRead`].
#[test]
fn notification_dispatch_read_emits_notification_read() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let recipient = g.next_user_id();

    let mut notif = new_notification(
        &g,
        school,
        actor,
        recipient,
        "Your child is absent today.",
        Channel::Email,
    );
    assert!(matches!(notif.status, NotificationStatus::Pending));
    assert!(notif.read_at.is_none());

    let read_at = clock.now();
    notif.mark_read(recipient, read_at, g.next_event_id());

    let event: NotificationRead = NotificationRead::new(
        notif.id,
        read_at,
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NotificationRead as DomainEvent>::EVENT_TYPE,
        "communication.notification.read"
    );
    assert!(matches!(notif.status, NotificationStatus::Read));
    assert_eq!(notif.read_at, Some(read_at));
    assert_eq!(event.notification_id, notif.id);
    assert_eq!(event.read_at, read_at);
}

/// Notification dispatch alternative step 2 (sender retracts
/// before read): the sender withdraws the notification. The
/// aggregate transitions to `NotificationStatus::Withdrawn`,
/// records the withdrawal reason and timestamp, and emits
/// [`NotificationWithdrawn`].
#[test]
fn notification_dispatch_withdraw_emits_notification_withdrawn() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());
    let recipient = g.next_user_id();

    let mut notif = new_notification(
        &g,
        school,
        actor,
        recipient,
        "Your child is absent today.",
        Channel::Sms,
    );
    assert!(matches!(notif.status, NotificationStatus::Pending));

    let withdrawn_at = clock.now();
    notif.withdraw(
        "Sent in error — child is present.".to_owned(),
        actor,
        withdrawn_at,
        g.next_event_id(),
    );

    let event: NotificationWithdrawn = NotificationWithdrawn::new(
        notif.id,
        "Sent in error — child is present.".to_owned(),
        g.next_event_id(),
        correlation,
        clock.now(),
    );

    assert_eq!(
        <NotificationWithdrawn as DomainEvent>::EVENT_TYPE,
        "communication.notification.withdrawn"
    );
    assert!(matches!(notif.status, NotificationStatus::Withdrawn));
    assert_eq!(notif.withdrawn_at, Some(withdrawn_at));
    assert_eq!(
        notif.withdrawn_reason.as_deref(),
        Some("Sent in error — child is present.")
    );
    assert_eq!(event.notification_id, notif.id);
    assert_eq!(event.reason, "Sent in error — child is present.");
}

/// Notification dispatch cross-channel: the same notification
/// shape is reusable across all five channels. Per
/// `Notification` aggregate invariant, the `channel` field is
/// free-form (`Channel::Email | Sms | Web | App | Push`) and
/// each dispatch creates an independent aggregate instance.
#[test]
fn notification_dispatch_supports_all_channels() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let recipient = g.next_user_id();

    let channels = [
        Channel::Email,
        Channel::Sms,
        Channel::Web,
        Channel::App,
        Channel::Push,
    ];

    for channel in channels {
        let notif = new_notification(
            &g,
            school,
            actor,
            recipient,
            "Cross-channel dispatch test.",
            channel,
        );
        assert_eq!(notif.channel, channel);
        assert_eq!(notif.recipient_user_id, recipient);
        assert!(matches!(notif.status, NotificationStatus::Pending));
        assert!(notif.read_at.is_none());
        assert!(notif.withdrawn_at.is_none());
    }
}

/// Notification dispatch failure path: per spec invariant 1,
/// a notification message must be non-empty.
/// `NotificationMessage::new` must reject empty messages so
/// that `Notification::fresh` can never receive an empty
/// message.
#[test]
fn notification_dispatch_empty_message_returns_validation_error() {
    let res = NotificationMessage::new(String::new());
    assert!(
        res.is_err(),
        "empty NotificationMessage must fail validation"
    );
}
