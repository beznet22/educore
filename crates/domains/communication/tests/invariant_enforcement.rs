//! Wave-32 invariant enforcement tests.
//!
//! Each test exercises a single spec invariant that the
//! aggregate layer was missing in the Phase-1 audit at
//! `docs/audit_reports/stub_vs_implementation.md` §
//! "communication — Deep Invariant Audit":
//!
//! | # | Aggregate                        | Invariant                                                                       |
//! |---|----------------------------------|---------------------------------------------------------------------------------|
//! | 1 | `Notice::unpublish`              | actor must be the original creator (spec item 3)                                |
//! | 2 | `Notice::mark_deleted`           | cannot delete a published notice (spec item 4)                                  |
//! | 3 | `Notification::mark_read`        | cannot read a withdrawn notification (spec item 2 & 3)                          |
//! | 4 | `Notification::withdraw`         | cannot withdraw a delivered notification (spec item 2)                          |
//! | 5 | `SendMessage::cancel`            | cannot cancel after first dispatch (spec item 3)                                |
//! | 6 | `ChatGroupMessageRecipient`      | cannot mark_read on a soft-deleted row (spec item 2 — monotonic read)           |
//!
//! The tests pin both the happy path (where the aggregate is
//! constructed and the rule does NOT fire) and the failure path
//! (where the invariant rejects an illegal state transition).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use educore_communication::prelude::*;
use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen, TestClock};
use educore_core::error::DomainError;
use educore_core::ids::CorrelationId;

// =============================================================================
// Test fixtures
// =============================================================================

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

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| panic!("invalid date {y}-{m}-{d}"))
}

// =============================================================================
// 1. Notice::unpublish — creator-only
// =============================================================================

/// Spec invariant: "A notice may be unpublished only by the same
/// actor who created it, or by an actor with `Notice.Unpublish`
/// capability." The capability check is dispatcher-side; the
/// aggregate enforces the creator-only constraint.
#[test]
fn notice_unpublish_rejects_non_creator() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let creator = tenant.actor_id;
    let other = g.next_user_id();
    let clock = TestClock::new();

    let mut notice = Notice::fresh(
        NoticeId::new(school, g.next_uuid()),
        NoticeTitle::new("Holiday").expect("title"),
        NoticeBody::new("Closed Monday.").expect("body"),
        date(2026, 9, 1),
        Some(date(2026, 9, 1)),
        AudienceDescriptor::All,
        None,
        creator,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );
    notice.publish(creator, clock.now(), g.next_event_id());

    let err = notice
        .unpublish(other, clock.now(), g.next_event_id())
        .expect_err("non-creator must be rejected");
    assert!(
        matches!(err, DomainError::Forbidden(_)),
        "expected Forbidden, got {err:?}"
    );
    // Status must remain Published — invariant rejected the mutation.
    assert!(matches!(notice.status, NoticeStatus::Published));
}

/// Counter-test: the original creator can always unpublish.
#[test]
fn notice_unpublish_allows_creator() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let creator = tenant.actor_id;
    let clock = TestClock::new();

    let mut notice = Notice::fresh(
        NoticeId::new(school, g.next_uuid()),
        NoticeTitle::new("Holiday").expect("title"),
        NoticeBody::new("Closed Monday.").expect("body"),
        date(2026, 9, 1),
        Some(date(2026, 9, 1)),
        AudienceDescriptor::All,
        None,
        creator,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );
    notice.publish(creator, clock.now(), g.next_event_id());

    notice
        .unpublish(creator, clock.now(), g.next_event_id())
        .expect("creator can unpublish");
    assert!(matches!(notice.status, NoticeStatus::Unpublished));
}

// =============================================================================
// 2. Notice::mark_deleted — reject if Published
// =============================================================================

/// Spec invariant: "A notice cannot be deleted after it has been
/// delivered to at least one recipient." A published notice has
/// already been delivered (the publish event triggered the
/// dispatch), so deletion must be rejected.
#[test]
fn notice_mark_deleted_rejects_published() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut notice = Notice::fresh(
        NoticeId::new(school, g.next_uuid()),
        NoticeTitle::new("Reminder").expect("title"),
        NoticeBody::new("Body").expect("body"),
        date(2026, 9, 1),
        None,
        AudienceDescriptor::All,
        None,
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );
    notice.publish(actor, clock.now(), g.next_event_id());

    let err = notice
        .mark_deleted(actor, clock.now(), g.next_event_id())
        .expect_err("published notice cannot be deleted");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    // Active status remains Active — invariant rejected the mutation.
    assert!(matches!(notice.active_status, ActiveStatus::Active));
}

/// Counter-test: an unpublished / draft notice can still be
/// soft-deleted (no delivery has happened).
#[test]
fn notice_mark_deleted_allows_draft() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut notice = Notice::fresh(
        NoticeId::new(school, g.next_uuid()),
        NoticeTitle::new("Draft").expect("title"),
        NoticeBody::new("Body").expect("body"),
        date(2026, 9, 1),
        None,
        AudienceDescriptor::All,
        None,
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    notice
        .mark_deleted(actor, clock.now(), g.next_event_id())
        .expect("draft notice can be deleted");
    assert!(matches!(notice.active_status, ActiveStatus::Retired));
}

// =============================================================================
// 3. Notification::mark_read — reject if Withdrawn
// =============================================================================

/// Spec invariant: a notification is post-delivery-immutable and
/// cannot be read once it has been withdrawn.
#[test]
fn notification_mark_read_rejects_withdrawn() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let recipient = g.next_user_id();
    let clock = TestClock::new();

    let mut notif = Notification::fresh(
        NotificationId::new(school, g.next_uuid()),
        recipient,
        NotificationType::Info,
        NotificationMessage::new("Test").expect("msg"),
        None,
        std::collections::BTreeMap::new(),
        Channel::Web,
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    notif
        .withdraw("sent in error".to_owned(), actor, clock.now(), g.next_event_id())
        .expect("fresh notification can be withdrawn");

    let err = notif
        .mark_read(recipient, clock.now(), g.next_event_id())
        .expect_err("withdrawn notification cannot be marked read");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert!(matches!(notif.status, NotificationStatus::Withdrawn));
    assert!(notif.read_at.is_none());
}

// =============================================================================
// 4. Notification::withdraw — reject if Delivered
// =============================================================================

/// Spec invariant: the notification is post-delivery-immutable.
/// Once `delivered_at` is set, withdrawal must be rejected —
/// the recipient has already seen it.
#[test]
fn notification_withdraw_rejects_delivered() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let recipient = g.next_user_id();
    let clock = TestClock::new();

    let mut notif = Notification::fresh(
        NotificationId::new(school, g.next_uuid()),
        recipient,
        NotificationType::Info,
        NotificationMessage::new("Test").expect("msg"),
        None,
        std::collections::BTreeMap::new(),
        Channel::Web,
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    // Simulate delivery by setting delivered_at directly (no
    // public deliver() method — the dispatcher writes the
    // timestamp). The invariant under test is the deliver-immutable
    // rule.
    notif.delivered_at = Some(clock.now());

    let err = notif
        .withdraw("too late".to_owned(), actor, clock.now(), g.next_event_id())
        .expect_err("delivered notification cannot be withdrawn");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert!(matches!(notif.status, NotificationStatus::Pending));
    assert!(notif.withdrawn_at.is_none());
}

// =============================================================================
// 5. SendMessage::cancel — reject if Dispatched
// =============================================================================

/// Spec invariant: "The job is immutable after the first dispatch."
/// Cancelling a dispatched broadcast would mislead recipients.
#[test]
fn send_message_cancel_rejects_dispatched() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut sm = SendMessage::fresh(
        SendMessageId::new(school, g.next_uuid()),
        NoticeTitle::new("Bulk").expect("title"),
        NoticeBody::new("Body").expect("body"),
        date(2026, 9, 1),
        Some(date(2026, 9, 1)),
        AudienceDescriptor::Users(vec![actor]),
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    let _ = sm.dispatch(actor, clock.now(), g.next_event_id());

    let err = sm
        .cancel(Some("too late".to_owned()), actor, clock.now(), g.next_event_id())
        .expect_err("dispatched message cannot be cancelled");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert!(matches!(sm.status, SendMessageStatus::Dispatched));
}

/// Counter-test: a draft broadcast can still be cancelled.
#[test]
fn send_message_cancel_allows_draft() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut sm = SendMessage::fresh(
        SendMessageId::new(school, g.next_uuid()),
        NoticeTitle::new("Bulk").expect("title"),
        NoticeBody::new("Body").expect("body"),
        date(2026, 9, 1),
        Some(date(2026, 9, 1)),
        AudienceDescriptor::Users(vec![actor]),
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    sm.cancel(None, actor, clock.now(), g.next_event_id())
        .expect("draft message can be cancelled");
    assert!(matches!(sm.status, SendMessageStatus::Cancelled));
}

// =============================================================================
// 6. ChatGroupMessageRecipient — reject mark_read after soft-delete
// =============================================================================

/// Spec invariant: "`read_at` may transition from null to a
/// timestamp; never back." Once `deleted_at` is set, the row is
/// retired and a subsequent `mark_read` would resurrect read-state
/// on a deleted row.
#[test]
fn chat_group_message_recipient_mark_read_rejects_deleted() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut rcp = ChatGroupMessageRecipient::fresh(
        ChatGroupMessageRecipientId::new(school, g.next_uuid()),
        ChatGroupId::new(school, g.next_uuid()),
        actor,
        ChatMessageId::new(school, g.next_uuid()),
        actor,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );

    // Simulate soft-delete (no public method exists — the
    // dispatcher retires the row). The invariant under test is
    // the monotonic-read rule.
    rcp.deleted_at = Some(clock.now());

    let err = rcp
        .mark_read(actor, clock.now(), g.next_event_id())
        .expect_err("deleted recipient row cannot be marked read");
    assert!(
        matches!(err, DomainError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
    assert!(rcp.read_at.is_none());
}

// =============================================================================
// Smoke: every modified aggregate still compiles and behaves under
// the public `create_*` / `unpublish_notice` service factory fns.
// =============================================================================

/// Sanity: `unpublish_notice` (the service-layer wrapper) returns
/// `DomainError::Forbidden` when the actor is not the creator.
/// This is the integration path that ties the aggregate rule to
/// the public command surface.
#[test]
fn unpublish_notice_service_rejects_non_creator() {
    use educore_communication::services::unpublish_notice;
    use educore_communication::value_objects::{AudienceDescriptor, NoticeBody, NoticeTitle};

    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let creator = tenant.actor_id;
    let other = g.next_user_id();
    let clock = TestClock::new();

    let mut notice = Notice::fresh(
        NoticeId::new(school, g.next_uuid()),
        NoticeTitle::new("Body").expect("title"),
        NoticeBody::new("Body").expect("body"),
        date(2026, 9, 1),
        None,
        AudienceDescriptor::All,
        None,
        creator,
        clock.now(),
        CorrelationId::from(g.next_uuid()),
    );
    notice.publish(creator, clock.now(), g.next_event_id());

    let cmd = UnpublishNoticeCommand {
        tenant: TenantContext::for_user(school, other, CorrelationId::from(g.next_uuid()), UserType::SchoolAdmin),
        notice_id: notice.id,
        reason: None,
    };

    let err = unpublish_notice(cmd, &clock, &g, &mut notice)
        .expect_err("service wrapper must surface the Forbidden signal");
    assert!(
        matches!(err, DomainError::Forbidden(_)),
        "expected Forbidden, got {err:?}"
    );
    assert!(matches!(notice.status, NoticeStatus::Published));
}

/// Silence "unused import" warnings on the smoke-only modules
/// when the test binary is built standalone.
#[test]
fn smoke_unused_imports() {
    let _ = SystemClock;
}
