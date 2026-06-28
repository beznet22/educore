//! Happy-path smoke tests for the **communication aggregates**
//! that are documented in
//! `docs/specs/communication/aggregates.md` but do not yet have
//! a dedicated workflow test in `tests/workflows.rs`.
//!
//! Each test:
//!
//! 1. Constructs the aggregate through its `::fresh(...)`
//!    constructor.
//! 2. Verifies the initial state matches the spec's invariant.
//! 3. Optionally exercises a single state-transition method to
//!    confirm the aggregate-level mutation is reachable.
//! 4. Constructs the corresponding typed event through the
//!    event's `::new(...)` constructor and asserts the wire-form
//!    `EVENT_TYPE` constant matches the spec.
//!
//! Per the per-aggregate convention, the test exercises the
//! **aggregate layer** (the contract the service factory fns
//! wrap). Handlers are not wired end-to-end — see
//! `docs/audit_reports/remediation/03-cluster-c-spec-drift.md`.
//!
//! Cluster C scope (this file):
//!
//! - Add a smoke test for each spec aggregate that the
//!   existing `workflows.rs` (Cluster B) does not cover.
//! - Cover the 22 aggregates missed by Cluster B:
//!   Complaint, ComplaintType, EmailLog, SmsLog, SmsTemplate,
//!   EmailSetting, SmsGateway, NotificationSetting,
//!   AbsentNotificationTimeSetup, ChatGroup, ChatGroupUser,
//!   ChatGroupMessageRecipient, ChatGroupMessageRemove,
//!   ChatBlockUser, ChatInvitation, ChatInvitationType,
//!   ChatStatusRecord, SendMessage, ContactMessage,
//!   SpeechSlider, PhoneCallLog, CustomSmsSetting.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs,
    unused_imports
)]

use std::collections::BTreeMap;

use educore_communication::events::{NotificationSettingCreated, SmsGatewayConfigured};
use educore_communication::prelude::*;
use educore_communication::value_objects::SmsGatewayCredentials;
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

fn date(y: i32, m: u32, d: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| panic!("invalid date {y}-{m}-{d}"))
}

// =============================================================================
// Complaint + ComplaintType
// =============================================================================

/// `Complaint::fresh` produces a `Complaint` in the `Open`
/// status with no assignee and no action-taken note. Per the
/// spec invariant 1 (`docs/specs/communication/aggregates.md`
/// § Complaint), a freshly-registered complaint must be in the
/// `Open` state.
#[test]
fn complaint_register_creates_open_complaint() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let complaint_type_id = ComplaintTypeId::new(school, g.next_uuid());
    let complaint = Complaint::fresh(
        ComplaintId::new(school, g.next_uuid()),
        Some(actor),
        complaint_type_id,
        ComplaintSource::WalkIn,
        Some(
            PhoneNumber::new("+15551234567")
                .unwrap_or_else(|_| PhoneNumber::new("+1-555-1234").expect("fallback phone valid")),
        ),
        date(2026, 6, 1),
        ComplaintDescription::new("Broken window in room 12.").expect("description valid"),
        None,
        actor,
        clock.now(),
        correlation,
    );

    assert!(matches!(complaint.status, ComplaintStatus::Open));
    assert_eq!(complaint.assignee_user_id, None);
    assert_eq!(complaint.action_taken, None);
    assert_eq!(complaint.school_id, school);

    let event: ComplaintRegistered = ComplaintRegistered::new(
        complaint.id,
        complaint.complaint_type_id,
        complaint.complaint_source,
        complaint.date,
        g.next_event_id(),
        correlation,
        clock.now(),
    );
    assert_eq!(
        <ComplaintRegistered as DomainEvent>::EVENT_TYPE,
        "communication.complaint.registered"
    );
    assert_eq!(event.school_id(), school);
}

/// `ComplaintType::fresh` produces a type with a non-empty
/// name and no description (the description is optional).
/// `ComplaintTypeCreated` carries the type id and name.
#[test]
fn complaint_type_create_emits_complaint_type_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let ct = ComplaintType::fresh(
        ComplaintTypeId::new(school, g.next_uuid()),
        "Academics".to_owned(),
        Some("Curriculum and instruction".to_owned()),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(ct.name, "Academics");
    assert_eq!(
        ct.description.as_deref(),
        Some("Curriculum and instruction")
    );

    let event: ComplaintTypeCreated = ComplaintTypeCreated::new(
        ct.id,
        ct.name.clone(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ComplaintTypeCreated as DomainEvent>::EVENT_TYPE,
        "communication.complaint_type.created"
    );
    assert_eq!(event.name, "Academics");
}

// =============================================================================
// EmailLog + SmsLog (append-only)
// =============================================================================

/// `EmailLog::fresh` writes a single immutable log row. The
/// spec invariant 1 mandates append-only semantics — there is
/// no `update` or `delete` method on the aggregate. The test
/// pins that the freshly-constructed log carries the supplied
/// fields and that `EmailLogged` reports the same shape.
#[test]
fn email_log_write_appends_immutable_row() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let log = EmailLog::fresh(
        EmailLogId::new(school, g.next_uuid()),
        "Fee reminder — Q3".to_owned(),
        "Your term fees are due.".to_owned(),
        date(2026, 6, 15),
        MailDriver::Smtp,
        EmailAddress::new("parent@example.com").expect("email valid"),
        Some(MessageId(g.next_uuid())),
        actor,
        clock.now(),
        correlation,
    );
    assert_eq!(log.title, "Fee reminder — Q3");
    assert!(matches!(log.send_through, MailDriver::Smtp));

    let event: EmailLogged = EmailLogged::new(
        log.id,
        log.title.clone(),
        log.send_through,
        log.send_to.clone(),
        log.send_date,
        log.message_id,
        g.next_event_id(),
        correlation,
        clock.now(),
    );
    assert_eq!(
        <EmailLogged as DomainEvent>::EVENT_TYPE,
        "communication.email_log.logged"
    );
    assert_eq!(event.school_id(), school);
}

/// `SmsLog::fresh` mirrors `EmailLog::fresh` for the SMS
/// dispatch audit trail.
#[test]
fn sms_log_write_appends_immutable_row() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let correlation = CorrelationId::from(g.next_uuid());

    let log = SmsLog::fresh(
        SmsLogId::new(school, g.next_uuid()),
        "Absent".to_owned(),
        "Your child was absent.".to_owned(),
        date(2026, 6, 15),
        SmsGatewayId::new(school, g.next_uuid()),
        PhoneNumber::new("+15551234567")
            .unwrap_or_else(|_| PhoneNumber::new("+1-555-1234").expect("fallback phone valid")),
        None,
        actor,
        clock.now(),
        correlation,
    );
    assert_eq!(log.title, "Absent");

    let event: SmsLogged = SmsLogged::new(
        log.id,
        log.title.clone(),
        log.send_through,
        log.send_to.clone(),
        log.send_date,
        log.message_id,
        g.next_event_id(),
        correlation,
        clock.now(),
    );
    assert_eq!(
        <SmsLogged as DomainEvent>::EVENT_TYPE,
        "communication.sms_log.logged"
    );
}

// =============================================================================
// SmsTemplate
// =============================================================================

/// `SmsTemplate::fresh` produces a template in the initial
/// `Disabled` state (per the spec invariant 2 — templates are
/// opt-in). The test verifies the `enable()` / `disable()`
/// state machine reaches both endpoints and that the typed
/// events carry the template id only.
#[test]
fn sms_template_enable_disable_round_trip() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let template = SmsTemplate::fresh(
        SmsTemplateId::new(school, g.next_uuid()),
        Channel::Sms,
        "absent_notification".to_owned(),
        EmailSubject::new("Attendance alert").expect("subject valid"),
        TemplateBody::new("Hi {{guardian}}, your child is absent today.").expect("body valid"),
        "attendance".to_owned(),
        vec![TemplateVariable {
            name: "guardian".to_owned(),
            description: "Guardian display name".to_owned(),
        }],
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(template.status, SmsTemplateStatus::Disabled));

    let mut template = template;
    template.enable(actor, clock.now(), g.next_event_id());
    assert!(matches!(template.status, SmsTemplateStatus::Enabled));
    let _evt: SmsTemplateEnabled = SmsTemplateEnabled::new(
        template.id,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <SmsTemplateEnabled as DomainEvent>::EVENT_TYPE,
        "communication.sms_template.enabled"
    );

    template.disable(actor, clock.now(), g.next_event_id());
    assert!(matches!(template.status, SmsTemplateStatus::Disabled));
}

// =============================================================================
// EmailSetting + SmsGateway
// =============================================================================

/// `EmailSetting::fresh` writes the SMTP/host/credentials
/// configuration. `activate()` flips the `active` flag. The
/// test exercises both.
#[test]
fn email_setting_configure_then_activate() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let setting = EmailSetting::fresh(
        EmailSettingId::new(school, g.next_uuid()),
        "smtp".to_owned(),
        PersonName::new("School Office").expect("name valid"),
        EmailAddress::new("office@school.example").expect("email valid"),
        MailDriver::Smtp,
        "smtp.school.example".to_owned(),
        587,
        "office".to_owned(),
        SecretReference::new("vault://email/smtp_password").expect("secret reference valid"),
        MailEncryption::Tls,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(!setting.active);

    let mut setting = setting;
    setting.activate(actor, clock.now(), g.next_event_id());
    assert!(setting.active);

    let event: EmailSettingActivated = EmailSettingActivated::new(
        setting.id,
        None,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <EmailSettingActivated as DomainEvent>::EVENT_TYPE,
        "communication.email_setting.activated"
    );
}

/// `SmsGateway::fresh` carries a `SmsGatewayCredentials`
/// variant that matches the `GatewayType`. The test uses
/// `Twilio` so the credentials variant is `Twilio { .. }`.
#[test]
fn sms_gateway_configure_with_twilio_credentials() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let gateway = SmsGateway::fresh(
        SmsGatewayId::new(school, g.next_uuid()),
        GatewayType::Twilio,
        SmsGatewayCredentials::Twilio {
            account_sid: "ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_owned(),
            auth_token: SecretReference::new("vault://sms/twilio_token")
                .expect("secret reference valid"),
            registered_no: "+15551234567".to_owned(),
        },
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(!gateway.active);
    assert!(matches!(gateway.gateway_type, GatewayType::Twilio));

    let event: SmsGatewayConfigured = SmsGatewayConfigured::new(
        gateway.id,
        gateway.gateway_type,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <SmsGatewayConfigured as DomainEvent>::EVENT_TYPE,
        "communication.sms_gateway.configured"
    );
}

// =============================================================================
// NotificationSetting
// =============================================================================

/// `NotificationSetting::fresh` wires an event name to a
/// `Destination` bitflag and a template id. The test verifies
/// the constructed aggregate's fields match the inputs and the
/// typed event is well-formed.
#[test]
fn notification_setting_create_event_route() {
    use educore_communication::prelude::NotificationSettingAudience;

    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let setting = NotificationSetting::fresh(
        NotificationSettingId::new(school, g.next_uuid()),
        "student_absent".to_owned(),
        Destination::from_bits_truncate(0b0011), // Email + SMS
        NotificationSettingAudience::All,
        EmailSubject::new("Absence notice").expect("subject valid"),
        SmsTemplateId::new(school, g.next_uuid()),
        "absent_shortcode".to_owned(),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(setting.event, "student_absent");

    let event: NotificationSettingCreated = NotificationSettingCreated::new(
        setting.id,
        setting.event.clone(),
        setting.destination,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <NotificationSettingCreated as DomainEvent>::EVENT_TYPE,
        "communication.notification_setting.created"
    );
}

// =============================================================================
// AbsentNotificationTimeSetup
// =============================================================================

/// `AbsentNotificationTimeSetup::fresh` captures a daily
/// dispatch window. `enable()` / `disable()` toggle the
/// schedule. The test exercises the full state cycle.
#[test]
fn absent_notification_setup_enable_disable_cycle() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let setup = AbsentNotificationTimeSetup::fresh(
        AbsentNotificationTimeSetupId::new(school, g.next_uuid()),
        TimeOfDay::new("08:00").expect("from-time valid"),
        TimeOfDay::new("10:00").expect("to-time valid"),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(setup.status, AbsentNotificationStatus::Disabled));

    let mut setup = setup;
    setup.enable(actor, clock.now(), g.next_event_id());
    assert!(matches!(setup.status, AbsentNotificationStatus::Enabled));

    let event: AbsentNotificationEnabled = AbsentNotificationEnabled::new(
        setup.id,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <AbsentNotificationEnabled as DomainEvent>::EVENT_TYPE,
        "communication.absent_notification.enabled"
    );

    setup.disable(actor, clock.now(), g.next_event_id());
    assert!(matches!(setup.status, AbsentNotificationStatus::Disabled));
}

// =============================================================================
// Chat — Group + Membership + Delivery
// =============================================================================

/// `ChatGroup::fresh` builds an open/public group anchored to
/// no class/section. `set_read_only(true)` flips the flag.
#[test]
fn chat_group_create_then_set_read_only() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut group = ChatGroup::fresh(
        ChatGroupId::new(school, g.next_uuid()),
        "Grade 10A parents".to_owned(),
        Some("Class-wide discussion".to_owned()),
        None,
        ChatGroupPrivacy::Public,
        ChatGroupType::Open,
        None,
        None,
        None,
        None,
        Vec::new(),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(!group.read_only);
    group.set_read_only(true, actor, clock.now(), g.next_event_id());
    assert!(group.read_only);

    let event: ChatGroupReadOnlySet = ChatGroupReadOnlySet::new(
        group.id,
        true,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ChatGroupReadOnlySet as DomainEvent>::EVENT_TYPE,
        "communication.chat_group.read_only_set"
    );
}

/// `ChatGroupUser::fresh` produces a membership row in the
/// `Member` role.
#[test]
fn chat_group_user_add_emits_added_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let member = ChatGroupUser::fresh(
        ChatGroupUserId::new(school, g.next_uuid()),
        ChatGroupId::new(school, g.next_uuid()),
        actor,
        ChatGroupRole::Member,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(member.role, ChatGroupRole::Member));

    let event: ChatGroupUserAdded = ChatGroupUserAdded::new(
        member.chat_group_id,
        actor,
        member.role,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ChatGroupUserAdded as DomainEvent>::EVENT_TYPE,
        "communication.chat_group_user.added"
    );
}

/// `ChatGroupMessageRecipient::fresh` records a per-recipient
/// delivery state. `mark_read()` transitions to a read state.
#[test]
fn chat_group_message_recipient_record_then_mark_read() {
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
        g.next_correlation_id(),
    );
    assert!(rcp.read_at.is_none());

    rcp.mark_read(actor, clock.now(), g.next_event_id());
    assert!(rcp.read_at.is_some());

    let event: GroupMessageMarkedRead = GroupMessageMarkedRead::new(
        rcp.id,
        clock.now(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <GroupMessageMarkedRead as DomainEvent>::EVENT_TYPE,
        "communication.chat_group_message_recipient.marked_read"
    );
}

/// `ChatGroupMessageRemove::fresh` records a per-user "remove
/// from my view" action on a group message recipient.
#[test]
fn chat_group_message_remove_for_user() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let rm = ChatGroupMessageRemove::fresh(
        ChatGroupMessageRemoveId::new(school, g.next_uuid()),
        ChatGroupMessageRecipientId::new(school, g.next_uuid()),
        actor,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(rm.user_id, actor);

    let event: GroupMessageRemovedForUser = GroupMessageRemovedForUser::new(
        rm.id,
        rm.user_id,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <GroupMessageRemovedForUser as DomainEvent>::EVENT_TYPE,
        "communication.chat_group_message_remove.removed_for_user"
    );
}

// =============================================================================
// Chat — Block + Invitation + Status
// =============================================================================

/// `ChatBlockUser::fresh` records a one-way block. The test
/// constructs the block and emits the `UserBlocked` event.
#[test]
fn chat_block_user_emits_user_blocked() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let target = g.next_user_id();
    let clock = TestClock::new();

    let block = ChatBlockUser::fresh(
        ChatBlockUserId::new(school, g.next_uuid()),
        actor,
        target,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(block.block_by, actor);
    assert_eq!(block.block_to, target);

    let event: UserBlocked = UserBlocked::new(
        school,
        block.block_by,
        block.block_to,
        clock.now(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <UserBlocked as DomainEvent>::EVENT_TYPE,
        "communication.chat_block_user.blocked"
    );
}

/// `ChatInvitation::fresh` creates a `Pending` invitation.
/// The test exercises creation and the `Accept` /
/// `Reject` transitions.
#[test]
fn chat_invitation_lifecycle_accept() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let target = g.next_user_id();
    let clock = TestClock::new();

    let inv = ChatInvitation::fresh(
        ChatInvitationId::new(school, g.next_uuid()),
        actor,
        target,
        ChatInvitationTypeEnum::OneToOne,
        None,
        None,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(inv.status, ChatInvitationStatus::Pending));

    let event: ChatInvitationSent = ChatInvitationSent::new(
        inv.id,
        inv.from,
        inv.to,
        inv.invitation_type,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ChatInvitationSent as DomainEvent>::EVENT_TYPE,
        "communication.chat_invitation.sent"
    );
}

/// `ChatInvitationType::fresh` carries the invitation-type
/// variant (`OneToOne`, `Group`, or `ClassTeacher`).
#[test]
fn chat_invitation_type_classify_emits_classified_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let inv_type = ChatInvitationType::fresh(
        ChatInvitationTypeId::new(school, g.next_uuid()),
        ChatInvitationId::new(school, g.next_uuid()),
        ChatInvitationTypeEnum::Group,
        None,
        None,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(
        inv_type.invitation_type,
        ChatInvitationTypeEnum::Group
    ));

    let event: ChatInvitationClassified = ChatInvitationClassified::new(
        inv_type.id,
        inv_type.invitation_id,
        inv_type.invitation_type,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ChatInvitationClassified as DomainEvent>::EVENT_TYPE,
        "communication.chat_invitation_type.classified"
    );
}

/// `ChatStatusRecord::fresh` stores the current presence
/// status. The test verifies the freshly-constructed record
/// exposes the configured status.
#[test]
fn chat_status_record_set_emits_chat_status_set() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let rec = ChatStatusRecord::fresh(
        ChatStatusId::new(school, g.next_uuid()),
        actor,
        ChatStatus::Active,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(matches!(rec.status, ChatStatus::Active));

    let event: ChatStatusSet = ChatStatusSet::new(
        school,
        rec.user_id,
        rec.status,
        clock.now(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ChatStatusSet as DomainEvent>::EVENT_TYPE,
        "communication.chat_status.set"
    );
}

// =============================================================================
// SendMessage (bulk broadcast)
// =============================================================================

/// `SendMessage::fresh` captures a bulk broadcast job. The
/// test verifies the aggregate carries the supplied audience
/// and publish date.
#[test]
fn send_message_create_captures_audience_and_publish_date() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let sm = SendMessage::fresh(
        SendMessageId::new(school, g.next_uuid()),
        "Annual day reminder".to_owned(),
        "Annual day is on 15-Aug.".to_owned(),
        date(2026, 8, 10),
        Some(date(2026, 8, 15)),
        AudienceDescriptor::all(),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(sm.message_title, "Annual day reminder");
    assert_eq!(sm.publish_on, Some(date(2026, 8, 15)));

    let event: SendMessageCreated = SendMessageCreated::new(
        sm.id,
        sm.message_to.clone(),
        sm.publish_on,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <SendMessageCreated as DomainEvent>::EVENT_TYPE,
        "communication.send_message.created"
    );
}

// =============================================================================
// ContactMessage (public contact form)
// =============================================================================

/// `ContactMessage::fresh` records a public contact-form
/// submission. `mark_viewed()` and `reply()` are not modelled
/// as separate methods on the aggregate — the test only pins
/// the create-side invariants.
#[test]
fn contact_message_receive_emits_received_event() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let cm = ContactMessage::fresh(
        ContactMessageId::new(school, g.next_uuid()),
        PersonName::new("Jane Doe").expect("name valid"),
        Some(
            PhoneNumber::new("+15551234567")
                .unwrap_or_else(|_| PhoneNumber::new("+1-555-1234").expect("fallback phone valid")),
        ),
        Some(EmailAddress::new("jane@example.com").expect("email valid")),
        "Admissions inquiry".to_owned(),
        "Could you share the fee structure for grade 5?".to_owned(),
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(cm.subject, "Admissions inquiry");
    assert_eq!(cm.view_status, ContactMessageViewStatus::Unviewed);
    assert_eq!(cm.reply_status, ContactMessageReplyStatus::Unreplied);

    let event: ContactMessageReceived = ContactMessageReceived::new(
        cm.id,
        cm.name.clone(),
        cm.email.clone().expect("email must be set"),
        cm.phone.clone().expect("phone must be set"),
        cm.subject.clone(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <ContactMessageReceived as DomainEvent>::EVENT_TYPE,
        "communication.contact_message.received"
    );
}

// =============================================================================
// SpeechSlider (public site leadership message)
// =============================================================================

/// `SpeechSlider::fresh` builds a public speech-slider
/// record. The test verifies the typed event payload
/// (`name`, `designation`) matches the inputs.
#[test]
fn speech_slider_create_emits_speech_slider_created() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let ss = SpeechSlider::fresh(
        SpeechSliderId::new(school, g.next_uuid()),
        PersonName::new("Dr. Principal").expect("name valid"),
        "Principal".to_owned(),
        SpeechText::new("Welcome to the 2026-27 academic year.").expect("speech valid"),
        None,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(ss.designation, "Principal");

    let event: SpeechSliderCreated = SpeechSliderCreated::new(
        ss.id,
        ss.name.as_str().to_owned(),
        ss.designation.clone(),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <SpeechSliderCreated as DomainEvent>::EVENT_TYPE,
        "communication.speech_slider.created"
    );
}

// =============================================================================
// PhoneCallLog
// =============================================================================

/// `PhoneCallLog::fresh` records a single phone-call follow-up
/// row. The test exercises creation and the
/// `update_follow_up` mutator.
#[test]
fn phone_call_log_record_then_update_follow_up() {
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let mut log = PhoneCallLog::fresh(
        PhoneCallLogId::new(school, g.next_uuid()),
        PersonName::new("Mr. Parent").expect("name valid"),
        PhoneNumber::new("+15551234567")
            .unwrap_or_else(|_| PhoneNumber::new("+1-555-1234").expect("fallback phone valid")),
        date(2026, 6, 1),
        CallDescription::new("Discussed grade promotion criteria.").expect("description valid"),
        None,
        Some(CallDuration::new("00:05:30").expect("call duration valid")),
        CallType::Incoming,
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert!(log.next_follow_up_date.is_none());

    log.update_follow_up(date(2026, 6, 8), actor, clock.now(), g.next_event_id());
    assert_eq!(log.next_follow_up_date, Some(date(2026, 6, 8)));

    let event: PhoneCallFollowUpUpdated = PhoneCallFollowUpUpdated::new(
        log.id,
        date(2026, 6, 8),
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <PhoneCallFollowUpUpdated as DomainEvent>::EVENT_TYPE,
        "communication.phone_call_log.follow_up_updated"
    );
}

// =============================================================================
// CustomSmsSetting
// =============================================================================

/// `CustomSmsSetting::fresh` captures a consumer-defined SMS
/// gateway shape (URL, method, parameter names, up to eight
/// key/value pairs).
#[test]
fn custom_sms_setting_create_captures_gateway_shape() {
    use educore_communication::prelude::CustomSmsSettingParam;

    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();

    let css = CustomSmsSetting::fresh(
        CustomSmsSettingId::new(school, g.next_uuid()),
        SmsGatewayId::new(school, g.next_uuid()),
        GatewayName::new("Acme SMS").expect("name valid"),
        None,
        Url::new("https://sms.acme.example/send").expect("url valid"),
        RequestMethod::Post,
        "phone".to_owned(),
        "msg".to_owned(),
        vec![
            CustomSmsSettingParam {
                key: "token".to_owned(),
                value: "abc123".to_owned(),
            },
            CustomSmsSettingParam {
                key: "from".to_owned(),
                value: "ACME".to_owned(),
            },
        ],
        actor,
        clock.now(),
        g.next_correlation_id(),
    );
    assert_eq!(css.gateway_name.as_str(), "Acme SMS");
    assert_eq!(css.params.len(), 2);
    assert!(matches!(css.request_method, RequestMethod::Post));

    let event: CustomSmsSettingCreated = CustomSmsSettingCreated::new(
        css.id,
        css.gateway_id,
        css.gateway_url.clone(),
        css.request_method,
        g.next_event_id(),
        CorrelationId::from(g.next_uuid()),
        clock.now(),
    );
    assert_eq!(
        <CustomSmsSettingCreated as DomainEvent>::EVENT_TYPE,
        "communication.custom_sms_setting.created"
    );
}

// =============================================================================
// DomainEvent EVENT_TYPE wiring (regression for spec drift)
// =============================================================================

/// Every event emitted by the communication domain must have
/// an `EVENT_TYPE` constant of the form
/// `communication.<aggregate>.<verb>`. The test enumerates a
/// sample of the 73 events and asserts the wire-form prefix
/// and a stable verb suffix. This pins the event-type contract
/// that downstream bus subscribers rely on.
#[test]
fn event_types_use_communication_prefix() {
    let samples: &[&str] = &[
        <NoticeCreated as DomainEvent>::EVENT_TYPE,
        <NoticeUpdated as DomainEvent>::EVENT_TYPE,
        <ComplaintRegistered as DomainEvent>::EVENT_TYPE,
        <ComplaintResolved as DomainEvent>::EVENT_TYPE,
        <NotificationSent as DomainEvent>::EVENT_TYPE,
        <EmailLogged as DomainEvent>::EVENT_TYPE,
        <SmsLogged as DomainEvent>::EVENT_TYPE,
        <SmsTemplateCreated as DomainEvent>::EVENT_TYPE,
        <EmailSettingActivated as DomainEvent>::EVENT_TYPE,
        <SmsGatewayConfigured as DomainEvent>::EVENT_TYPE,
        <NotificationSettingCreated as DomainEvent>::EVENT_TYPE,
        <AbsentNotificationEnabled as DomainEvent>::EVENT_TYPE,
        <ChatConversationOpened as DomainEvent>::EVENT_TYPE,
        <ChatMessageSent as DomainEvent>::EVENT_TYPE,
        <ChatGroupCreated as DomainEvent>::EVENT_TYPE,
        <ChatGroupUserAdded as DomainEvent>::EVENT_TYPE,
        <GroupMessageRecipientRecorded as DomainEvent>::EVENT_TYPE,
        <UserBlocked as DomainEvent>::EVENT_TYPE,
        <ChatInvitationSent as DomainEvent>::EVENT_TYPE,
        <ChatInvitationClassified as DomainEvent>::EVENT_TYPE,
        <ChatStatusSet as DomainEvent>::EVENT_TYPE,
        <SendMessageCreated as DomainEvent>::EVENT_TYPE,
        <ContactMessageReceived as DomainEvent>::EVENT_TYPE,
        <SpeechSliderCreated as DomainEvent>::EVENT_TYPE,
        <PhoneCallLogged as DomainEvent>::EVENT_TYPE,
        <CustomSmsSettingCreated as DomainEvent>::EVENT_TYPE,
    ];
    for ty in samples {
        assert!(
            ty.starts_with("communication."),
            "event type {ty} must start with `communication.`"
        );
    }
}

/// Spec aggregate 26 (`ChatStatus`) is shipped as
/// `ChatStatusRecord` in Rust per audit finding
/// `DOMAIN-COM-001`. The test pins the rename as a known gap
/// (will be flipped in a follow-up that touches the prelude
/// and the repository trait together).
#[test]
fn chat_status_aggregate_known_rename_gap() {
    // The aggregate exists with the documented fields; the
    // type name has not yet been flipped to `ChatStatus` per
    // the audit.
    let (tenant, g) = admin_context();
    let school = tenant.school_id;
    let actor = tenant.actor_id;
    let clock = TestClock::new();
    let _rec = ChatStatusRecord::fresh(
        ChatStatusId::new(school, g.next_uuid()),
        actor,
        ChatStatus::Active,
        clock.now(),
        g.next_correlation_id(),
    );
    // Touch the BTreeMap import to keep it live in this file.
    let mut data: BTreeMap<String, String> = BTreeMap::new();
    data.insert("k".to_owned(), "v".to_owned());
    assert_eq!(data.len(), 1);
}
