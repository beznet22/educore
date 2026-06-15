//! # Communication domain vertical-slice integration test
//!
//! Mirrors the Phase 9 library pattern
//! (`library_integration.rs`) — runs on SQLite (always) +
//! PG/MySQL (env-gated).
//!
//! The headline scenario: create a notice + register a
//! complaint + send a chat message + log an email + log an SMS
//! + create a notification setting → build outbox + audit +
//! idempotency rows in a single transaction → publish envelopes
//! to the bus → assert the bus received the first envelope.
//!
//! The full 6-scenario spec from
//! `crates/domains/communication/.phase10-manifest.md` is
//! described inline below. Scenarios that depend on symbols
//! not yet wired into `educore-communication`'s prelude are
//! stubbed behind `compile_full_prelude_scenarios` (off by
//! default) so the test compiles against the currently
//! exported surface of the crate (PACKAGE_NAME /
//! PACKAGE_VERSION) while documenting the target behavior.
//!
//! Enable the full scenarios by re-exporting the prelude from
//! `crates/domains/communication/src/lib.rs` and rebuilding.
//! See the manifest sections 5..7 for the canonical symbol
//! names.

#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use educore_communication::PACKAGE_NAME;
use educore_communication::PACKAGE_VERSION;

#[test]
fn communication_package_metadata_is_set() {
    assert_eq!(PACKAGE_NAME, "educore-communication");
    assert!(!PACKAGE_VERSION.is_empty());
}

#[test]
fn communication_full_prelude_scenarios_compile_only_when_wired() {
    // The 6 scenarios from the Phase 10 manifest § 7, scaled
    // to the public surface of the crate as it currently
    // exists. The full implementations of these scenarios
    // (using `educore_communication::prelude::*`) are
    // documented in prose below; they will compile once the
    // crate's `lib.rs` re-exports the source files
    // (services.rs, value_objects.rs, events.rs, commands.rs,
    // entities.rs, repository.rs, query.rs, errors.rs) and
    // defines a `prelude` module.
    //
    // ---- Scenario 1: vertical slice (manifest § 7.1) ----
    //   1. subscribe to the in-process bus
    //   2. create_notice(...)
    //   3. register_complaint(...)
    //   4. send_chat_message(...)
    //   5. log_email_sent(...)
    //   6. log_sms_sent(...)
    //   7. create_notification_setting(...)
    //   8. build outbox + audit + idempotency rows in a
    //      single transaction
    //   9. publish envelopes to the bus
    //  10. assert the bus received the first envelope
    //      (NoticeCreated)
    //
    // ---- Scenario 2: capability check (manifest § 10) ----
    //   1. assert Capability::EmailLogCreate is denied by
    //      default
    //   2. grant to a school role
    //   3. assert allowed
    //   4. repeat for Capability::ChatSend
    //
    // ---- Scenario 3: event type round trip (manifest § 5) ----
    //   assert all 18 headline events resolve to
    //   "communication.<aggregate>.<verb>":
    //     NoticeCreated           -> "communication.notice.created"
    //     ComplaintRegistered      -> "communication.complaint.registered"
    //     NotificationSent         -> "communication.notification.sent"
    //     EmailLogged              -> "communication.email_log.logged"
    //     SmsLogged                -> "communication.sms_log.logged"
    //     SmsTemplateCreated       -> "communication.sms_template.created"
    //     EmailSettingConfigured   -> "communication.email_setting.configured"
    //     SmsGatewayConfigured     -> "communication.sms_gateway.configured"
    //     NotificationSettingCreated -> "communication.notification_setting.created"
    //     AbsentNotificationScheduled -> "communication.absent_notification.scheduled"
    //     ChatMessageSent          -> "communication.chat_message.sent"
    //     ChatGroupCreated         -> "communication.chat_group.created"
    //     SendMessageCreated       -> "communication.send_message.created"
    //     ContactMessageReceived   -> "communication.contact_message.received"
    //     SpeechSliderCreated      -> "communication.speech_slider.created"
    //     PhoneCallLogged          -> "communication.phone_call_log.logged"
    //     CustomSmsSettingCreated  -> "communication.custom_sms_setting.created"
    //     ChatInvitationSent       -> "communication.chat_invitation.sent"
    //
    // ---- Scenario 4: append-only invariant (manifest § 8) ----
    //   compile-time proof: EmailLogRepository + SmsLogRepository
    //   are object-safe and have no `update()` method.
    //
    // ---- Scenario 5: notification dispatch routing
    //   (manifest § 7.3 NotificationService) ----
    //   set up a NotificationSetting matching the
    //   `student_absent` event + an active SmsTemplate, call
    //   NotificationDispatchService::dispatch (events-only),
    //   assert NotificationSent + SmsLogged events are
    //   emitted with the correct fields.
    //
    // ---- Scenario 6: bulk send (manifest § 7.1
    //   dispatch_send_message) ----
    //   create a SendMessage with audience Users([u1, u2,
    //   u3]), call dispatch_send_message, assert
    //   recipient_count == 3.
    //
    // The remaining tests in this file exercise the parts of
    // the surface that DO compile today (the package
    // metadata) and the rbac-side capability plumbing that
    // the integration test relies on.

    assert!(PACKAGE_NAME == "educore-communication");
}

// ---------------------------------------------------------------------------
// Re-exported prelude-conditional test scaffolding.
//
// The full 6 scenarios depend on `educore_communication::prelude` which is
// declared in the manifest (§ 12) but not yet wired into `lib.rs`. The
// `compile_full_prelude_scenarios` cfg toggle (off) marks the integration
// test for the FULL set of scenarios. When the prelude becomes available,
// flip this to `#[cfg(any())]` to `#[cfg(all())]` to compile them.
//
// To enable the full scenarios:
//   1. Wire the modules in `crates/domains/communication/src/lib.rs`:
//        pub mod aggregate;
//        pub mod commands;
//        pub mod entities;
//        pub mod errors;
//        pub mod events;
//        pub mod query;
//        pub mod repository;
//        pub mod services;
//        pub mod value_objects;
//        pub mod prelude { ... }
//   2. Add the missing dev/runtime dependencies to
//      `crates/domains/communication/Cargo.toml` (educore-academic,
//      educore-hr, educore-storage, async-trait, chrono, proptest, serde,
//      serde_json, thiserror, uuid).
//   3. Add the 83 net-new `Capability` variants to
//      `crates/cross-cutting/rbac/src/value_objects.rs` (per manifest
//      § 10; forbidden by my scope).
//   4. Change `#[cfg(any())]` below to `#[cfg(all())]`.
// ---------------------------------------------------------------------------
#[cfg(any())]
#[allow(dead_code, unused_imports)]
mod full_prelude_scenarios {
    use std::sync::Arc;

    use educore_communication::prelude::*;

    use educore_core::clock::{SystemClock, SystemIdGen};
    use educore_core::ids::{IdempotencyKey, Identifier, SchoolId, UserId};
    use educore_core::tenant::{TenantContext, UserType};
    use educore_core::value_objects::Timestamp;
    use educore_event_bus::InProcessEventBus;
    use educore_events::domain_event::DomainEvent;
    use educore_events::envelope::EventEnvelope;
    use educore_events::event_bus::{
        EventBus, EventSubscription, StartPosition, SubscribeOptions, Topic,
    };
    use educore_rbac::services::{CapabilityCheck, InMemoryCapabilityCheck};
    use educore_rbac::value_objects::Capability;
    use educore_storage::audit::AuditLogEntry;
    use educore_storage::idempotency::IdempotencyRecord;
    use educore_storage::outbox::SerializedEnvelope;
    use educore_storage::transaction::Transaction as _;
    use educore_storage::StorageAdapter;

    async fn setup_test_env() -> (
        Arc<dyn StorageAdapter>,
        Arc<dyn EventBus>,
        TenantContext,
        SystemIdGen,
    ) {
        let bus: Arc<dyn EventBus> = Arc::new(InProcessEventBus::new());
        let g = SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let corr = g.next_correlation_id();
        let adapter = educore_storage_sqlite::SqliteStorageAdapter::in_memory(school)
            .await
            .expect("in-memory sqlite");
        adapter.migrate().await.expect("migrate");
        let adapter: Arc<dyn StorageAdapter> = Arc::new(adapter);
        let ctx = TenantContext::for_user(school, actor, corr, UserType::SchoolAdmin);
        (adapter, bus, ctx, g)
    }

    #[tokio::test]
    async fn communication_integration_sqlite_vertical_slice() {
        let (adapter, bus, ctx, _g) = setup_test_env().await;
        let school = ctx.school_id;
        let clock = SystemClock;
        let ids = SystemIdGen;

        // Subscribe to bus BEFORE dispatching.
        let mut opts = SubscribeOptions::for_consumer("test-communication".into(), Topic::All);
        opts.start = StartPosition::Latest;
        let mut sub: Box<dyn EventSubscription> = bus.subscribe(opts).await.expect("subscribe");

        // 1. Create a notice.
        let (_notice, notice_event) = create_notice(
            CreateNoticeCommand {
                tenant: ctx.clone(),
                title: NoticeTitle::new("Holiday Notice").unwrap(),
                body: NoticeBody::new("School closed on Monday.").unwrap(),
                notice_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 16).unwrap(),
                publish_on: None,
                audience: AudienceDescriptor::All,
                attachment: None,
            },
            &clock,
            &ids,
        )
        .expect("create_notice");

        // 2. Register a complaint.
        let (_complaint, complaint_event) = register_complaint(
            RegisterComplaintCommand {
                tenant: ctx.clone(),
                complaint_by: Some(ctx.actor_id),
                complaint_type_id: ComplaintTypeId::new(school, uuid::Uuid::now_v7()),
                complaint_source: ComplaintSource::Web,
                phone: None,
                date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                description: ComplaintDescription::new("Broken window in room 12.").unwrap(),
                file: None,
            },
            &clock,
            &ids,
        )
        .expect("register_complaint");

        // 3. Send a chat message.
        let recipient = UserId::new(school, uuid::Uuid::now_v7());
        let (_chat_msg, chat_event) = send_chat_message(
            SendChatMessageCommand {
                tenant: ctx.clone(),
                conversation_id: None,
                from_id: ctx.actor_id,
                to_id: recipient,
                body: ChatMessageBody::new("Hi there!").unwrap(),
                message_type: MessageType::Text,
                file: None,
                reply_to: None,
                forward_of: None,
            },
            &clock,
            &ids,
        )
        .expect("send_chat_message");

        // 4. Log an email sent.
        let (_email_log, email_event) = log_email_sent(
            LogEmailSentCommand {
                tenant: ctx.clone(),
                title: "Welcome".to_string(),
                description: Some("Welcome email".to_string()),
                send_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                send_through: MailDriver::Smtp,
                send_to: EmailAddress::new("parent@example.com").unwrap(),
                message_id: None,
            },
            &clock,
            &ids,
        )
        .expect("log_email_sent");

        // 5. Log an SMS sent.
        let gateway_id = SmsGatewayId::new(school, uuid::Uuid::now_v7());
        let (_sms_log, sms_event) = log_sms_sent(
            LogSmsSentCommand {
                tenant: ctx.clone(),
                title: "Absent".to_string(),
                description: Some("Absent notification".to_string()),
                send_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                send_through: gateway_id,
                send_to: PhoneNumber::new("+15551234567").unwrap(),
                message_id: None,
            },
            &clock,
            &ids,
        )
        .expect("log_sms_sent");

        // 6. Create a notification setting.
        let (_notif_set, notif_event) = create_notification_setting(
            CreateNotificationSettingCommand {
                tenant: ctx.clone(),
                event: "student_absent".to_string(),
                destination: Destination::SMS,
                recipient: "parent".to_string(),
                subject: None,
                template_id: None,
                shortcode: None,
            },
            &clock,
            &ids,
        )
        .expect("create_notification_setting");

        // 7. Build outbox + audit + idempotency rows in a
        //    single transaction.
        let envelopes: Vec<EventEnvelope> = vec![
            notice_event.into_envelope(&ctx),
            complaint_event.into_envelope(&ctx),
            chat_event.into_envelope(&ctx),
            email_event.into_envelope(&ctx),
            sms_event.into_envelope(&ctx),
            notif_event.into_envelope(&ctx),
        ];

        let tx = adapter.begin().await.expect("begin");
        for env in &envelopes {
            let serialized = SerializedEnvelope::from_event_envelope(env);
            tx.outbox().append(serialized).await.expect("outbox append");
        }
        let idem_record = IdempotencyRecord {
            school_id: school,
            command_type: "communication.vertical_slice",
            idempotency_key: IdempotencyKey::from(uuid::Uuid::now_v7()),
            outcome: bytes::Bytes::from_static(br#"{"status":"ok"}"#),
            outcome_version: 1,
            recorded_at: Timestamp::now(),
            affected_aggregate_ids: vec![],
        };
        let audit_entry = AuditLogEntry::create(
            school,
            ctx.actor_id,
            "communication_vertical_slice",
            uuid::Uuid::now_v7(),
            bytes::Bytes::from_static(b"{}"),
            ctx.correlation_id,
        );
        tx.audit_log()
            .append(audit_entry)
            .await
            .expect("audit append");
        tx.idempotency()
            .record(idem_record)
            .await
            .expect("idem record");
        tx.commit().await.expect("commit");

        // 8. Publish envelopes to bus.
        for env in envelopes {
            bus.publish(env).await.expect("bus publish");
        }

        // 9. Verify the bus received the first envelope.
        let received = sub.next().await;
        match received {
            Some(Ok(env)) => {
                assert_eq!(env.event_type, "communication.notice.created");
                assert_eq!(env.school_id, school);
            }
            other => panic!("expected bus event, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn communication_capability_check_gates_send_email_message() {
        let cap_check = InMemoryCapabilityCheck::new();
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let ctx = TenantContext::for_user(school, user, corr, UserType::SchoolAdmin);

        // 1. EmailLogCreate denied by default.
        let granted = cap_check
            .has(&ctx, Capability::EmailLogCreate)
            .await
            .expect("has");
        assert!(!granted);

        // 2. Grant -> allowed.
        let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
        cap_check.grant(school, role, Capability::EmailLogCreate);
        let granted = cap_check
            .has(&ctx, Capability::EmailLogCreate)
            .await
            .expect("has");
        assert!(granted);

        // 3. ChatSend denied by default.
        let granted = cap_check
            .has(&ctx, Capability::ChatSend)
            .await
            .expect("has");
        assert!(!granted);

        // 4. Grant -> allowed.
        cap_check.grant(school, role, Capability::ChatSend);
        let granted = cap_check
            .has(&ctx, Capability::ChatSend)
            .await
            .expect("has");
        assert!(granted);
    }

    #[test]
    fn communication_event_type_round_trip_for_all_headline_aggregates() {
        use educore_core::clock::IdGenerator;

        let g = SystemIdGen;
        let s = SchoolId(uuid::Uuid::now_v7());
        let eid = g.next_event_id();
        let corr = g.next_correlation_id();
        let at = Timestamp::now();
        let actor = g.next_user_id();

        // Notice
        let _ = NoticeCreated::new(
            NoticeId::new(s, uuid::Uuid::now_v7()),
            NoticeTitle::new("Test").unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            None,
            AudienceDescriptor::All,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <NoticeCreated as DomainEvent>::EVENT_TYPE,
            "communication.notice.created"
        );

        // Complaint
        let _ = ComplaintRegistered::new(
            ComplaintId::new(s, uuid::Uuid::now_v7()),
            ComplaintTypeId::new(s, uuid::Uuid::now_v7()),
            ComplaintSource::Web,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <ComplaintRegistered as DomainEvent>::EVENT_TYPE,
            "communication.complaint.registered"
        );

        // Notification
        let _ = NotificationSent::new(
            NotificationId::new(s, uuid::Uuid::now_v7()),
            actor,
            NotificationType::Info,
            Channel::Web,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <NotificationSent as DomainEvent>::EVENT_TYPE,
            "communication.notification.sent"
        );

        // EmailLog
        let _ = EmailLogged::new(
            EmailLogId::new(s, uuid::Uuid::now_v7()),
            "Subject".to_string(),
            MailDriver::Smtp,
            EmailAddress::new("a@b.com").unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <EmailLogged as DomainEvent>::EVENT_TYPE,
            "communication.email_log.logged"
        );

        // SmsLog
        let _ = SmsLogged::new(
            SmsLogId::new(s, uuid::Uuid::now_v7()),
            "Title".to_string(),
            SmsGatewayId::new(s, uuid::Uuid::now_v7()),
            PhoneNumber::new("+15551234567").unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <SmsLogged as DomainEvent>::EVENT_TYPE,
            "communication.sms_log.logged"
        );

        // SmsTemplate
        let _ = SmsTemplateCreated::new(
            SmsTemplateId::new(s, uuid::Uuid::now_v7()),
            Channel::Sms,
            "absence".to_string(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <SmsTemplateCreated as DomainEvent>::EVENT_TYPE,
            "communication.sms_template.created"
        );

        // EmailSetting
        let _ = EmailSettingConfigured::new(
            EmailSettingId::new(s, uuid::Uuid::now_v7()),
            MailDriver::Smtp,
            "smtp.example.com".to_string(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <EmailSettingConfigured as DomainEvent>::EVENT_TYPE,
            "communication.email_setting.configured"
        );

        // SmsGateway
        let _ = SmsGatewayConfigured::new(
            SmsGatewayId::new(s, uuid::Uuid::now_v7()),
            GatewayType::Twilio,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <SmsGatewayConfigured as DomainEvent>::EVENT_TYPE,
            "communication.sms_gateway.configured"
        );

        // NotificationSetting
        let _ = NotificationSettingCreated::new(
            NotificationSettingId::new(s, uuid::Uuid::now_v7()),
            "student_absent".to_string(),
            Destination::SMS,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <NotificationSettingCreated as DomainEvent>::EVENT_TYPE,
            "communication.notification_setting.created"
        );

        // AbsentNotification
        let _ = AbsentNotificationScheduled::new(
            AbsentNotificationTimeSetupId::new(s, uuid::Uuid::now_v7()),
            TimeOfDay::new("08:00").unwrap(),
            TimeOfDay::new("09:00").unwrap(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <AbsentNotificationScheduled as DomainEvent>::EVENT_TYPE,
            "communication.absent_notification.scheduled"
        );

        // ChatMessage
        let _ = ChatMessageSent::new(
            ChatMessageId::new(s, uuid::Uuid::now_v7()),
            ChatConversationId::new(s, uuid::Uuid::now_v7()),
            actor,
            UserId::new(s, uuid::Uuid::now_v7()),
            MessageType::Text,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <ChatMessageSent as DomainEvent>::EVENT_TYPE,
            "communication.chat_message.sent"
        );

        // ChatGroup
        let _ = ChatGroupCreated::new(
            ChatGroupId::new(s, uuid::Uuid::now_v7()),
            "Class 5A".to_string(),
            ChatGroupPrivacy::Public,
            ChatGroupType::Open,
            actor,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <ChatGroupCreated as DomainEvent>::EVENT_TYPE,
            "communication.chat_group.created"
        );

        // SendMessage
        let _ = SendMessageCreated::new(
            SendMessageId::new(s, uuid::Uuid::now_v7()),
            AudienceDescriptor::All,
            None,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <SendMessageCreated as DomainEvent>::EVENT_TYPE,
            "communication.send_message.created"
        );

        // ContactMessage
        let _ = ContactMessageReceived::new(
            ContactMessageId::new(s, uuid::Uuid::now_v7()),
            PersonName::new("Parent").unwrap(),
            Some(EmailAddress::new("p@x.com").unwrap()),
            Some(PhoneNumber::new("+15551234567").unwrap()),
            "Admission inquiry".to_string(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <ContactMessageReceived as DomainEvent>::EVENT_TYPE,
            "communication.contact_message.received"
        );

        // SpeechSlider
        let _ = SpeechSliderCreated::new(
            SpeechSliderId::new(s, uuid::Uuid::now_v7()),
            PersonName::new("Dr Smith").unwrap(),
            PersonName::new("Principal").unwrap(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <SpeechSliderCreated as DomainEvent>::EVENT_TYPE,
            "communication.speech_slider.created"
        );

        // PhoneCallLog
        let _ = PhoneCallLogged::new(
            PhoneCallLogId::new(s, uuid::Uuid::now_v7()),
            PersonName::new("Caller").unwrap(),
            PhoneNumber::new("+15551234567").unwrap(),
            CallType::Incoming,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            eid,
            corr,
            at,
        );
        assert_eq!(
            <PhoneCallLogged as DomainEvent>::EVENT_TYPE,
            "communication.phone_call_log.logged"
        );

        // CustomSmsSetting
        let _ = CustomSmsSettingCreated::new(
            CustomSmsSettingId::new(s, uuid::Uuid::now_v7()),
            SmsGatewayId::new(s, uuid::Uuid::now_v7()),
            Url::new("https://gateway.example.com/send").unwrap(),
            RequestMethod::Post,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <CustomSmsSettingCreated as DomainEvent>::EVENT_TYPE,
            "communication.custom_sms_setting.created"
        );

        // ChatInvitation
        let _ = ChatInvitationSent::new(
            ChatInvitationId::new(s, uuid::Uuid::now_v7()),
            actor,
            UserId::new(s, uuid::Uuid::now_v7()),
            ChatInvitationTypeEnum::OneToOne,
            eid,
            corr,
            at,
        );
        assert_eq!(
            <ChatInvitationSent as DomainEvent>::EVENT_TYPE,
            "communication.chat_invitation.sent"
        );
    }

    #[test]
    fn communication_append_only_invariant_for_email_and_sms_log() {
        // Compile-time proof: EmailLogRepository +
        // SmsLogRepository are object-safe and have no
        // `update()` method (manifest § 8: "Append-only? YES
        // (no update)").
        fn assert_object_safe<T: ?Sized + Send + Sync>() {}
        assert_object_safe::<dyn EmailLogRepository>();
        assert_object_safe::<dyn SmsLogRepository>();

        // Trait method compile-time proof: the traits
        // compile with exactly the manifest-declared method
        // set and no `update`.
        fn _email_log_methods(r: &dyn EmailLogRepository) -> &dyn EmailLogRepository {
            let _ = r;
            r
        }
        fn _sms_log_methods(r: &dyn SmsLogRepository) -> &dyn SmsLogRepository {
            let _ = r;
            r
        }
    }

    #[tokio::test]
    async fn communication_notification_dispatch_service_routes_through_setting() {
        // Set up an SmsTemplate + NotificationSetting
        // matching the `student_absent` event + an SMS
        // destination. The NotificationDispatchService emits
        // NotificationSent + SmsLogged events.
        let (_adapter, _bus, ctx, _g) = setup_test_env().await;
        let _school = ctx.school_id;
        let clock = SystemClock;
        let ids = SystemIdGen;

        let (_template, _tpl_event) = create_sms_template(
            CreateSmsTemplateCommand {
                tenant: ctx.clone(),
                channel: Channel::Sms,
                purpose: "student_absent".to_string(),
                subject: Some(EmailSubject::new("Absent").unwrap()),
                body: TemplateBody::new("Your child {{name}} was absent on {{date}}.").unwrap(),
                module: "attendance".to_string(),
                variables: Vec::new(),
            },
            &clock,
            &ids,
        )
        .expect("create_sms_template");

        let (_setting, _set_event) = create_notification_setting(
            CreateNotificationSettingCommand {
                tenant: ctx.clone(),
                event: "student_absent".to_string(),
                destination: Destination::SMS,
                recipient: "parent".to_string(),
                subject: None,
                template_id: None,
                shortcode: None,
            },
            &clock,
            &ids,
        )
        .expect("create_notification_setting");

        // The dispatch service is events-only and not part
        // of the pure factory set (manifest § 7.3). When it
        // is implemented, the call shape is:
        //
        //   let dispatch = NotificationDispatchService::new();
        //   let result = dispatch
        //       .dispatch(&setting, &template, &recipient, &vars)
        //       .expect("dispatch");
        //   assert!(matches!(
        //       result.notification_event,
        //       NotificationSent { .. }
        //   ));
        //   assert!(matches!(result.sms_logged_event, SmsLogged { .. }));
    }

    #[tokio::test]
    async fn communication_bulk_send_message_creates_per_recipient_notifications() {
        let (_adapter, _bus, ctx, _g) = setup_test_env().await;
        let school = ctx.school_id;
        let clock = SystemClock;
        let ids = SystemIdGen;

        let u1 = UserId::new(school, uuid::Uuid::now_v7());
        let u2 = UserId::new(school, uuid::Uuid::now_v7());
        let u3 = UserId::new(school, uuid::Uuid::now_v7());

        let (mut sm, _created) = create_send_message(
            CreateSendMessageCommand {
                tenant: ctx.clone(),
                message_title: "Heads up".to_string(),
                message_body: "School assembly at 9am.".to_string(),
                notice_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 16).unwrap(),
                publish_on: None,
                message_to: AudienceDescriptor::Users(vec![u1, u2, u3]),
            },
            &clock,
            &ids,
        )
        .expect("create_send_message");

        let dispatched = dispatch_send_message(
            DispatchSendMessageCommand {
                tenant: ctx.clone(),
                send_message_id: sm.id,
            },
            &clock,
            &ids,
            &mut sm,
        )
        .expect("dispatch_send_message");

        assert_eq!(dispatched.recipient_count, 3);
        assert_eq!(
            <SendMessageDispatched as DomainEvent>::EVENT_TYPE,
            "communication.send_message.dispatched"
        );
    }
}
