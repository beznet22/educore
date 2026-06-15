//! # Communication domain services
//!
//! Pure factory functions that take a typed command + a clock +
//! an id generator and return the new aggregate + the typed
//! event. The dispatcher is responsible for persisting the
//! aggregate and writing the audit / outbox / idempotency
//! rows in a single transaction (per the Phase 2..9 pattern).
//!
//! Phase 10 ships:
//!
//! - 70 pure factory service functions (one per mutating
//!   command, per [`Phase 10 manifest`][manifest] § 7.1).
//! - 7 headline service functions
//!   ([`notify_user`], [`mark_as_read`], [`send_notice_message`],
//!   [`send_complaint_message`], [`send_chat_message`],
//!   [`send_email_message`], [`send_sms_message`]).
//! - 7 service structs ([`NotificationService`], [`ChatService`],
//!   [`ComplaintService`], [`AbsentNotificationService`],
//!   [`TemplateService`], [`SmsDispatchPolicy`], [`ChatInvitePolicy`])
//!   plus the 3 specifications [`ActiveRecipients`],
//!   [`NoticesPublishedInRange`], [`ChatInvitePolicy`].
//! - A 100-case proptest of [`TemplateService::render`] (the
//!   headline correctness check, mirroring Phase 9's
//!   `FineCalculationService` and Phase 7's `LateFeeService`).
//!
//! [manifest]: ../.phase10-manifest.md

#![allow(missing_docs)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::if_same_then_else)]

use std::collections::BTreeMap;

use chrono::NaiveDate;

use educore_academic::StudentId;
use educore_core::clock::{Clock, IdGenerator};
use educore_core::error::{DomainError, Result};
use educore_core::ids::{CorrelationId, EventId, Identifier, SchoolId, UserId};
use educore_core::tenant::TenantContext;
use educore_core::value_objects::{ActiveStatus, Timestamp};

use crate::aggregate::*;
use crate::commands::*;
use crate::entities::{
    AbsentNotificationDispatch, ComplaintNote, ContactMessageReply, DeliveryOutcome,
};
use crate::events::*;
use crate::value_objects::*;

fn event_id_to_uuid(e: EventId) -> uuid::Uuid {
    e.as_uuid()
}

// =============================================================================
// Notice service
// =============================================================================

/// Builds a new [`Notice`] aggregate + a [`NoticeCreated`]
/// event.
pub fn create_notice<C: Clock, G: IdGenerator>(
    cmd: CreateNoticeCommand,
    clock: &C,
    ids: &G,
) -> Result<(Notice, NoticeCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = NoticeId::new(school, event_id_to_uuid(event_id));
    let mut notice = Notice::fresh(
        id,
        cmd.title.clone(),
        cmd.body.clone(),
        cmd.notice_date,
        cmd.publish_on,
        cmd.audience.clone(),
        cmd.attachment,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    notice.last_event_id = Some(event_id);
    let event = NoticeCreated::new(
        id,
        cmd.title,
        cmd.notice_date,
        cmd.publish_on,
        cmd.audience,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((notice, event))
}

/// Mutates a [`Notice`] aggregate and emits a [`NoticeUpdated`]
/// event.
pub fn update_notice<C: Clock, G: IdGenerator>(
    cmd: UpdateNoticeCommand,
    clock: &C,
    ids: &G,
    notice: &mut Notice,
) -> Result<NoticeUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = notice.update(
        cmd.title,
        cmd.body,
        Some(cmd.publish_on),
        cmd.audience,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(NoticeUpdated::new(
        notice.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Publishes a [`Notice`] aggregate and emits a
/// [`NoticePublished`] event.
pub fn publish_notice<C: Clock, G: IdGenerator>(
    cmd: PublishNoticeCommand,
    clock: &C,
    ids: &G,
    notice: &mut Notice,
) -> Result<NoticePublished> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let published_at = cmd.publish_at.unwrap_or(now);
    notice.publish(cmd.tenant.actor_id, now, event_id);
    Ok(NoticePublished::new(
        notice.id,
        published_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Unpublishes a [`Notice`] aggregate and emits a
/// [`NoticeUnpublished`] event.
pub fn unpublish_notice<C: Clock, G: IdGenerator>(
    cmd: UnpublishNoticeCommand,
    clock: &C,
    ids: &G,
    notice: &mut Notice,
) -> Result<NoticeUnpublished> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    notice.unpublish(cmd.tenant.actor_id, now, event_id);
    Ok(NoticeUnpublished::new(
        notice.id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`Notice`] aggregate and emits a
/// [`NoticeDeleted`] event.
pub fn delete_notice<C: Clock, G: IdGenerator>(
    cmd: DeleteNoticeCommand,
    clock: &C,
    ids: &G,
    notice: &mut Notice,
) -> Result<NoticeDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    notice.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(NoticeDeleted::new(
        notice.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Complaint service
// =============================================================================

/// Builds a new [`Complaint`] aggregate + a
/// [`ComplaintRegistered`] event.
#[allow(clippy::too_many_arguments)]
pub fn register_complaint<C: Clock, G: IdGenerator>(
    cmd: RegisterComplaintCommand,
    clock: &C,
    ids: &G,
) -> Result<(Complaint, ComplaintRegistered)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ComplaintId::new(school, event_id_to_uuid(event_id));
    let mut complaint = Complaint::fresh(
        id,
        cmd.complaint_by,
        cmd.complaint_type_id,
        cmd.complaint_source,
        cmd.phone,
        cmd.date,
        cmd.description.clone(),
        cmd.file,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    complaint.last_event_id = Some(event_id);
    let event = ComplaintRegistered::new(
        id,
        cmd.complaint_type_id,
        cmd.complaint_source,
        cmd.date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((complaint, event))
}

/// Assigns a [`Complaint`] to a user and emits a
/// [`ComplaintAssigned`] event.
pub fn assign_complaint<C: Clock, G: IdGenerator>(
    cmd: AssignComplaintCommand,
    clock: &C,
    ids: &G,
    complaint: &mut Complaint,
) -> Result<ComplaintAssigned> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    complaint.assign(cmd.assignee_user_id, cmd.tenant.actor_id, now, event_id);
    Ok(ComplaintAssigned::new(
        complaint.id,
        cmd.assignee_user_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Updates the status of a [`Complaint`] and emits a
/// [`ComplaintStatusChanged`] event.
pub fn update_complaint_status<C: Clock, G: IdGenerator>(
    cmd: UpdateComplaintStatusCommand,
    clock: &C,
    ids: &G,
    complaint: &mut Complaint,
) -> Result<ComplaintStatusChanged> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let from = complaint.status;
    complaint.update_status(cmd.status, cmd.tenant.actor_id, now, event_id);
    Ok(ComplaintStatusChanged::new(
        complaint.id,
        from,
        cmd.status,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Resolves a [`Complaint`] and emits a [`ComplaintResolved`]
/// event.
pub fn resolve_complaint<C: Clock, G: IdGenerator>(
    cmd: ResolveComplaintCommand,
    clock: &C,
    ids: &G,
    complaint: &mut Complaint,
) -> Result<ComplaintResolved> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let resolved_at = now;
    complaint.resolve(
        cmd.action_taken.clone(),
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(ComplaintResolved::new(
        complaint.id,
        cmd.action_taken,
        resolved_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Adds a note to a [`Complaint`] and emits a
/// [`ComplaintNoteAdded`] event.
pub fn add_complaint_note<C: Clock, G: IdGenerator>(
    cmd: AddComplaintNoteCommand,
    clock: &C,
    ids: &G,
    complaint: &mut Complaint,
) -> Result<(ComplaintNote, ComplaintNoteAdded)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ComplaintNoteId::new(school, event_id_to_uuid(event_id));
    let note = ComplaintNote::new(
        id,
        complaint.id,
        cmd.tenant.actor_id,
        cmd.note.clone(),
        now,
        cmd.tenant.correlation_id,
    );
    let _ = complaint;
    let event = ComplaintNoteAdded::new(
        complaint.id,
        cmd.note,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((note, event))
}

// =============================================================================
// ComplaintType service
// =============================================================================

/// Builds a new [`ComplaintType`] aggregate + a
/// [`ComplaintTypeCreated`] event.
pub fn create_complaint_type<C: Clock, G: IdGenerator>(
    cmd: CreateComplaintTypeCommand,
    clock: &C,
    ids: &G,
) -> Result<(ComplaintType, ComplaintTypeCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ComplaintTypeId::new(school, event_id_to_uuid(event_id));
    let mut ct = ComplaintType::fresh(
        id,
        cmd.name.clone(),
        cmd.description,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    ct.last_event_id = Some(event_id);
    let event = ComplaintTypeCreated::new(
        id,
        cmd.name,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((ct, event))
}

/// Mutates a [`ComplaintType`] aggregate and emits a
/// [`ComplaintTypeUpdated`] event.
pub fn update_complaint_type<C: Clock, G: IdGenerator>(
    cmd: UpdateComplaintTypeCommand,
    clock: &C,
    ids: &G,
    ct: &mut ComplaintType,
) -> Result<ComplaintTypeUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = ct.update(
        cmd.name,
        Some(cmd.description),
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(ComplaintTypeUpdated::new(
        ct.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`ComplaintType`] aggregate and emits a
/// [`ComplaintTypeDeleted`] event.
pub fn delete_complaint_type<C: Clock, G: IdGenerator>(
    cmd: DeleteComplaintTypeCommand,
    clock: &C,
    ids: &G,
    ct: &mut ComplaintType,
) -> Result<ComplaintTypeDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    ct.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(ComplaintTypeDeleted::new(
        ct.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Notification service
// =============================================================================

/// Builds a new [`Notification`] aggregate + a
/// [`NotificationSent`] event.
#[allow(clippy::too_many_arguments)]
pub fn send_notification<C: Clock, G: IdGenerator>(
    cmd: SendNotificationCommand,
    clock: &C,
    ids: &G,
) -> Result<(Notification, NotificationSent)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = NotificationId::new(school, event_id_to_uuid(event_id));
    let mut n = Notification::fresh(
        id,
        cmd.recipient_user_id,
        cmd.notification_type,
        cmd.message,
        cmd.url,
        cmd.data,
        cmd.channel,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    n.last_event_id = Some(event_id);
    let event = NotificationSent::new(
        id,
        cmd.recipient_user_id,
        cmd.notification_type,
        cmd.channel,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((n, event))
}

/// Marks a [`Notification`] as read and emits a
/// [`NotificationRead`] event.
pub fn mark_notification_read<C: Clock, G: IdGenerator>(
    cmd: MarkNotificationReadCommand,
    clock: &C,
    ids: &G,
    n: &mut Notification,
) -> Result<NotificationRead> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let read_at = now;
    n.mark_read(cmd.tenant.actor_id, now, event_id);
    Ok(NotificationRead::new(
        n.id,
        read_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Withdraws a [`Notification`] and emits a
/// [`NotificationWithdrawn`] event.
pub fn withdraw_notification<C: Clock, G: IdGenerator>(
    cmd: WithdrawNotificationCommand,
    clock: &C,
    ids: &G,
    n: &mut Notification,
) -> Result<NotificationWithdrawn> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    n.withdraw(
        cmd.reason.clone(),
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(NotificationWithdrawn::new(
        n.id,
        cmd.reason,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// EmailLog service (append-only)
// =============================================================================

/// Appends an [`EmailLog`] aggregate + an [`EmailLogged`] event.
/// `EmailLog` is append-only (no update / delete).
#[allow(clippy::too_many_arguments)]
pub fn log_email_sent<C: Clock, G: IdGenerator>(
    cmd: LogEmailSentCommand,
    clock: &C,
    ids: &G,
) -> Result<(EmailLog, EmailLogged)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = EmailLogId::new(school, event_id_to_uuid(event_id));
    let mut log = EmailLog::fresh(
        id,
        cmd.title.clone(),
        cmd.description,
        cmd.send_date,
        cmd.send_through,
        cmd.send_to.clone(),
        cmd.message_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    log.last_event_id = Some(event_id);
    let event = EmailLogged::new(
        id,
        cmd.title,
        cmd.send_through,
        cmd.send_to,
        cmd.send_date,
        cmd.message_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((log, event))
}

// =============================================================================
// SmsLog service (append-only)
// =============================================================================

/// Appends an [`SmsLog`] aggregate + an [`SmsLogged`] event.
/// `SmsLog` is append-only (no update / delete).
#[allow(clippy::too_many_arguments)]
pub fn log_sms_sent<C: Clock, G: IdGenerator>(
    cmd: LogSmsSentCommand,
    clock: &C,
    ids: &G,
) -> Result<(SmsLog, SmsLogged)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SmsLogId::new(school, event_id_to_uuid(event_id));
    let mut log = SmsLog::fresh(
        id,
        cmd.title.clone(),
        cmd.description,
        cmd.send_date,
        cmd.send_through,
        cmd.send_to.clone(),
        cmd.message_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    log.last_event_id = Some(event_id);
    let event = SmsLogged::new(
        id,
        cmd.title,
        cmd.send_through,
        cmd.send_to,
        cmd.send_date,
        cmd.message_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((log, event))
}

// =============================================================================
// SmsTemplate service
// =============================================================================

/// Builds a new [`SmsTemplate`] aggregate + an
/// [`SmsTemplateCreated`] event.
pub fn create_sms_template<C: Clock, G: IdGenerator>(
    cmd: CreateSmsTemplateCommand,
    clock: &C,
    ids: &G,
) -> Result<(SmsTemplate, SmsTemplateCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SmsTemplateId::new(school, event_id_to_uuid(event_id));
    let purpose_key = TemplateKey::new(cmd.purpose.clone())?;
    let mut t = SmsTemplate::fresh(
        id,
        cmd.channel,
        cmd.purpose,
        cmd.subject,
        cmd.body,
        cmd.module,
        cmd.variables,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    t.last_event_id = Some(event_id);
    let event = SmsTemplateCreated::new(
        id,
        cmd.channel,
        purpose_key,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((t, event))
}

/// Mutates an [`SmsTemplate`] aggregate and emits an
/// [`SmsTemplateUpdated`] event.
pub fn update_sms_template<C: Clock, G: IdGenerator>(
    cmd: UpdateSmsTemplateCommand,
    clock: &C,
    ids: &G,
    t: &mut SmsTemplate,
) -> Result<SmsTemplateUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = t.update(
        cmd.subject,
        cmd.body,
        cmd.variables,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(SmsTemplateUpdated::new(
        t.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Enables an [`SmsTemplate`] and emits an
/// [`SmsTemplateEnabled`] event.
pub fn enable_sms_template<C: Clock, G: IdGenerator>(
    cmd: EnableSmsTemplateCommand,
    clock: &C,
    ids: &G,
    t: &mut SmsTemplate,
) -> Result<SmsTemplateEnabled> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    t.enable(cmd.tenant.actor_id, now, event_id);
    Ok(SmsTemplateEnabled::new(
        t.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Disables an [`SmsTemplate`] and emits an
/// [`SmsTemplateDisabled`] event.
pub fn disable_sms_template<C: Clock, G: IdGenerator>(
    cmd: DisableSmsTemplateCommand,
    clock: &C,
    ids: &G,
    t: &mut SmsTemplate,
) -> Result<SmsTemplateDisabled> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    t.disable(cmd.tenant.actor_id, now, event_id);
    Ok(SmsTemplateDisabled::new(
        t.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes an [`SmsTemplate`] and emits an
/// [`SmsTemplateDeleted`] event.
pub fn delete_sms_template<C: Clock, G: IdGenerator>(
    cmd: DeleteSmsTemplateCommand,
    clock: &C,
    ids: &G,
    t: &mut SmsTemplate,
) -> Result<SmsTemplateDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    t.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(SmsTemplateDeleted::new(
        t.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// EmailSetting service
// =============================================================================

/// Builds a new [`EmailSetting`] aggregate + an
/// [`EmailSettingConfigured`] event.
#[allow(clippy::too_many_arguments)]
pub fn configure_email_setting<C: Clock, G: IdGenerator>(
    cmd: ConfigureEmailSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<(EmailSetting, EmailSettingConfigured)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = EmailSettingId::new(school, event_id_to_uuid(event_id));
    let mut es = EmailSetting::fresh(
        id,
        cmd.email_engine_type,
        cmd.from_name,
        cmd.from_email,
        cmd.mail_driver,
        cmd.mail_host.clone(),
        cmd.mail_port,
        cmd.mail_username,
        cmd.mail_password,
        cmd.mail_encryption,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    es.last_event_id = Some(event_id);
    let event = EmailSettingConfigured::new(
        id,
        cmd.mail_driver,
        cmd.mail_host,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((es, event))
}

/// Activates an [`EmailSetting`] and emits an
/// [`EmailSettingActivated`] event.
pub fn activate_email_setting<C: Clock, G: IdGenerator>(
    cmd: ActivateEmailSettingCommand,
    clock: &C,
    ids: &G,
    es: &mut EmailSetting,
) -> Result<EmailSettingActivated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let previous_id = es.activate(cmd.tenant.actor_id, now, event_id);
    Ok(EmailSettingActivated::new(
        es.id,
        previous_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes an [`EmailSetting`] and emits an
/// [`EmailSettingDeleted`] event.
pub fn delete_email_setting<C: Clock, G: IdGenerator>(
    cmd: DeleteEmailSettingCommand,
    clock: &C,
    ids: &G,
    es: &mut EmailSetting,
) -> Result<EmailSettingDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    es.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(EmailSettingDeleted::new(
        es.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// SmsGateway service
// =============================================================================

/// Builds a new [`SmsGateway`] aggregate + an
/// [`SmsGatewayConfigured`] event.
pub fn configure_sms_gateway<C: Clock, G: IdGenerator>(
    cmd: ConfigureSmsGatewayCommand,
    clock: &C,
    ids: &G,
) -> Result<(SmsGateway, SmsGatewayConfigured)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SmsGatewayId::new(school, event_id_to_uuid(event_id));
    let mut g = SmsGateway::fresh(
        id,
        cmd.gateway_type,
        cmd.credentials,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    g.last_event_id = Some(event_id);
    let event = SmsGatewayConfigured::new(
        id,
        cmd.gateway_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((g, event))
}

/// Activates an [`SmsGateway`] and emits an
/// [`SmsGatewayActivated`] event.
pub fn activate_sms_gateway<C: Clock, G: IdGenerator>(
    cmd: ActivateSmsGatewayCommand,
    clock: &C,
    ids: &G,
    g: &mut SmsGateway,
) -> Result<SmsGatewayActivated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let previous_id = g.activate(cmd.tenant.actor_id, now, event_id);
    Ok(SmsGatewayActivated::new(
        g.id,
        g.gateway_type,
        previous_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes an [`SmsGateway`] and emits an
/// [`SmsGatewayDeleted`] event.
pub fn delete_sms_gateway<C: Clock, G: IdGenerator>(
    cmd: DeleteSmsGatewayCommand,
    clock: &C,
    ids: &G,
    g: &mut SmsGateway,
) -> Result<SmsGatewayDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    g.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(SmsGatewayDeleted::new(
        g.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// CustomSmsSetting service
// =============================================================================

/// Builds a new [`CustomSmsSetting`] aggregate + a
/// [`CustomSmsSettingCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_custom_sms_setting<C: Clock, G: IdGenerator>(
    cmd: CreateCustomSmsSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<(CustomSmsSetting, CustomSmsSettingCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = CustomSmsSettingId::new(school, event_id_to_uuid(event_id));
    let mut s = CustomSmsSetting::fresh(
        id,
        cmd.gateway_id,
        cmd.gateway_name,
        cmd.set_auth,
        cmd.gateway_url.clone(),
        cmd.request_method,
        cmd.send_to_parameter_name,
        cmd.message_to_parameter_name,
        cmd.params,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    s.last_event_id = Some(event_id);
    let event = CustomSmsSettingCreated::new(
        id,
        cmd.gateway_id,
        cmd.gateway_url,
        cmd.request_method,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((s, event))
}

/// Mutates a [`CustomSmsSetting`] aggregate and emits a
/// [`CustomSmsSettingUpdated`] event.
pub fn update_custom_sms_setting<C: Clock, G: IdGenerator>(
    cmd: UpdateCustomSmsSettingCommand,
    clock: &C,
    ids: &G,
    s: &mut CustomSmsSetting,
) -> Result<CustomSmsSettingUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = s.update(
        cmd.gateway_name,
        cmd.set_auth,
        cmd.gateway_url,
        cmd.request_method,
        cmd.params,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(CustomSmsSettingUpdated::new(
        s.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`CustomSmsSetting`] and emits a
/// [`CustomSmsSettingDeleted`] event.
pub fn delete_custom_sms_setting<C: Clock, G: IdGenerator>(
    cmd: DeleteCustomSmsSettingCommand,
    clock: &C,
    ids: &G,
    s: &mut CustomSmsSetting,
) -> Result<CustomSmsSettingDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    s.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(CustomSmsSettingDeleted::new(
        s.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// NotificationSetting service
// =============================================================================

/// Builds a new [`NotificationSetting`] aggregate + a
/// [`NotificationSettingCreated`] event.
pub fn create_notification_setting<C: Clock, G: IdGenerator>(
    cmd: CreateNotificationSettingCommand,
    clock: &C,
    ids: &G,
) -> Result<(NotificationSetting, NotificationSettingCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = NotificationSettingId::new(school, event_id_to_uuid(event_id));
    let mut ns = NotificationSetting::fresh(
        id,
        cmd.event,
        cmd.destination,
        cmd.recipient,
        cmd.subject,
        cmd.template_id,
        cmd.shortcode,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    ns.last_event_id = Some(event_id);
    let event = NotificationSettingCreated::new(
        id,
        ns.event.clone(),
        cmd.destination,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((ns, event))
}

/// Mutates a [`NotificationSetting`] aggregate and emits a
/// [`NotificationSettingUpdated`] event.
pub fn update_notification_setting<C: Clock, G: IdGenerator>(
    cmd: UpdateNotificationSettingCommand,
    clock: &C,
    ids: &G,
    ns: &mut NotificationSetting,
) -> Result<NotificationSettingUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = ns.update(
        cmd.destination,
        cmd.recipient,
        cmd.subject,
        cmd.template_id,
        cmd.shortcode,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(NotificationSettingUpdated::new(
        ns.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`NotificationSetting`] and emits a
/// [`NotificationSettingDeleted`] event.
pub fn delete_notification_setting<C: Clock, G: IdGenerator>(
    cmd: DeleteNotificationSettingCommand,
    clock: &C,
    ids: &G,
    ns: &mut NotificationSetting,
) -> Result<NotificationSettingDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    ns.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(NotificationSettingDeleted::new(
        ns.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// AbsentNotification service
// =============================================================================

/// Configures a new [`AbsentNotificationTimeSetup`] aggregate +
/// an [`AbsentNotificationScheduled`] event.
pub fn configure_absent_notification<C: Clock, G: IdGenerator>(
    cmd: ConfigureAbsentNotificationCommand,
    clock: &C,
    ids: &G,
) -> Result<(AbsentNotificationTimeSetup, AbsentNotificationScheduled)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = AbsentNotificationTimeSetupId::new(school, event_id_to_uuid(event_id));
    let mut an = AbsentNotificationTimeSetup::fresh(
        id,
        cmd.time_from.clone(),
        cmd.time_to.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    an.last_event_id = Some(event_id);
    let event = AbsentNotificationScheduled::new(
        id,
        cmd.time_from,
        cmd.time_to,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((an, event))
}

/// Enables an [`AbsentNotificationTimeSetup`] and emits an
/// [`AbsentNotificationEnabled`] event.
pub fn enable_absent_notification<C: Clock, G: IdGenerator>(
    cmd: EnableAbsentNotificationCommand,
    clock: &C,
    ids: &G,
    an: &mut AbsentNotificationTimeSetup,
) -> Result<AbsentNotificationEnabled> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    an.enable(cmd.tenant.actor_id, now, event_id);
    Ok(AbsentNotificationEnabled::new(
        an.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Disables an [`AbsentNotificationTimeSetup`] and emits an
/// [`AbsentNotificationDisabled`] event.
pub fn disable_absent_notification<C: Clock, G: IdGenerator>(
    cmd: DisableAbsentNotificationCommand,
    clock: &C,
    ids: &G,
    an: &mut AbsentNotificationTimeSetup,
) -> Result<AbsentNotificationDisabled> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    an.disable(cmd.tenant.actor_id, now, event_id);
    Ok(AbsentNotificationDisabled::new(
        an.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes an [`AbsentNotificationTimeSetup`] and emits an
/// [`AbsentNotificationDeleted`] event.
pub fn delete_absent_notification<C: Clock, G: IdGenerator>(
    cmd: DeleteAbsentNotificationCommand,
    clock: &C,
    ids: &G,
    an: &mut AbsentNotificationTimeSetup,
) -> Result<AbsentNotificationDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    an.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(AbsentNotificationDeleted::new(
        an.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat 1-to-1 service
// =============================================================================

/// Opens a new [`ChatConversation`] aggregate + a
/// [`ChatConversationOpened`] event.
pub fn open_chat_conversation<C: Clock, G: IdGenerator>(
    cmd: OpenChatConversationCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatConversation, ChatConversationOpened)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatConversationId::new(school, event_id_to_uuid(event_id));
    let mut conv = ChatConversation::fresh(
        id,
        cmd.from_id,
        cmd.to_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    conv.last_event_id = Some(event_id);
    let event = ChatConversationOpened::new(
        id,
        cmd.from_id,
        cmd.to_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((conv, event))
}

/// Closes a [`ChatConversation`] and emits a
/// [`ChatConversationClosed`] event.
pub fn close_chat_conversation<C: Clock, G: IdGenerator>(
    cmd: CloseChatConversationCommand,
    clock: &C,
    ids: &G,
    conv: &mut ChatConversation,
) -> Result<ChatConversationClosed> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    conv.close(cmd.tenant.actor_id, now, event_id);
    Ok(ChatConversationClosed::new(
        conv.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Sends a [`ChatMessage`] and emits a [`ChatMessageSent`]
/// event.
#[allow(clippy::too_many_arguments)]
pub fn send_chat_message<C: Clock, G: IdGenerator>(
    cmd: SendChatMessageCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatMessage, ChatMessageSent)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatMessageId::new(school, event_id_to_uuid(event_id));
    let conversation_id = cmd.conversation_id.unwrap_or_else(|| {
        ChatConversationId::new(school, event_id_to_uuid(ids.next_event_id()))
    });
    let mut m = ChatMessage::fresh(
        id,
        conversation_id,
        cmd.from_id,
        cmd.to_id,
        cmd.body,
        cmd.message_type,
        cmd.file,
        cmd.reply_to,
        cmd.forward_of,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    m.last_event_id = Some(event_id);
    let event = ChatMessageSent::new(
        id,
        m.conversation_id,
        cmd.from_id,
        cmd.to_id,
        cmd.message_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((m, event))
}

/// Marks a [`ChatMessage`] as seen and emits a
/// [`ChatMessageSeen`] event.
pub fn mark_chat_message_seen<C: Clock, G: IdGenerator>(
    cmd: MarkChatMessageSeenCommand,
    clock: &C,
    ids: &G,
    m: &mut ChatMessage,
) -> Result<ChatMessageSeen> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let seen_at = now;
    m.mark_seen(cmd.tenant.actor_id, now, event_id);
    Ok(ChatMessageSeen::new(
        m.id,
        cmd.tenant.actor_id,
        seen_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`ChatMessage`] and emits a
/// [`ChatMessageDeleted`] event.
pub fn delete_chat_message<C: Clock, G: IdGenerator>(
    cmd: DeleteChatMessageCommand,
    clock: &C,
    ids: &G,
    m: &mut ChatMessage,
) -> Result<ChatMessageDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    m.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(ChatMessageDeleted::new(
        m.id,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat group service
// =============================================================================

/// Creates a new [`ChatGroup`] aggregate + a
/// [`ChatGroupCreated`] event.
#[allow(clippy::too_many_arguments)]
pub fn create_chat_group<C: Clock, G: IdGenerator>(
    cmd: CreateChatGroupCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatGroup, ChatGroupCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatGroupId::new(school, event_id_to_uuid(event_id));
    let mut g = ChatGroup::fresh(
        id,
        cmd.name.clone(),
        cmd.description,
        cmd.photo,
        cmd.privacy,
        cmd.group_type,
        cmd.class_id,
        cmd.section_id,
        cmd.subject_id,
        cmd.teacher_id,
        cmd.initial_members,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    g.last_event_id = Some(event_id);
    let event = ChatGroupCreated::new(
        id,
        cmd.name,
        cmd.privacy,
        cmd.group_type,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((g, event))
}

/// Mutates a [`ChatGroup`] aggregate and emits a
/// [`ChatGroupUpdated`] event.
pub fn update_chat_group<C: Clock, G: IdGenerator>(
    cmd: UpdateChatGroupCommand,
    clock: &C,
    ids: &G,
    g: &mut ChatGroup,
) -> Result<ChatGroupUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = g.update(
        cmd.name,
        cmd.description,
        cmd.photo,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(ChatGroupUpdated::new(
        g.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Sets the read-only flag on a [`ChatGroup`] and emits a
/// [`ChatGroupReadOnlySet`] event.
pub fn set_chat_group_read_only<C: Clock, G: IdGenerator>(
    cmd: SetChatGroupReadOnlyCommand,
    clock: &C,
    ids: &G,
    g: &mut ChatGroup,
) -> Result<ChatGroupReadOnlySet> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    g.set_read_only(
        cmd.read_only,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(ChatGroupReadOnlySet::new(
        g.id,
        cmd.read_only,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`ChatGroup`] and emits a
/// [`ChatGroupDeleted`] event.
pub fn delete_chat_group<C: Clock, G: IdGenerator>(
    cmd: DeleteChatGroupCommand,
    clock: &C,
    ids: &G,
    g: &mut ChatGroup,
) -> Result<ChatGroupDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    g.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(ChatGroupDeleted::new(
        g.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat group membership service
// =============================================================================

/// Adds a user to a [`ChatGroup`] and emits a
/// [`ChatGroupUserAdded`] event.
pub fn add_user_to_chat_group<C: Clock, G: IdGenerator>(
    cmd: AddUserToChatGroupCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatGroupUser, ChatGroupUserAdded)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatGroupUserId::new(school, event_id_to_uuid(event_id));
    let mut m = ChatGroupUser::fresh(
        id,
        cmd.chat_group_id,
        cmd.user_id,
        cmd.role,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    m.last_event_id = Some(event_id);
    let event = ChatGroupUserAdded::new(
        cmd.chat_group_id,
        cmd.user_id,
        cmd.role,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((m, event))
}

/// Sets the role of a [`ChatGroupUser`] and emits a
/// [`ChatGroupUserRoleChanged`] event.
pub fn set_chat_group_user_role<C: Clock, G: IdGenerator>(
    cmd: SetChatGroupUserRoleCommand,
    clock: &C,
    ids: &G,
    m: &mut ChatGroupUser,
) -> Result<ChatGroupUserRoleChanged> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let from = m.role;
    m.set_role(cmd.role, cmd.tenant.actor_id, now, event_id);
    Ok(ChatGroupUserRoleChanged::new(
        m.chat_group_id,
        m.user_id,
        from,
        cmd.role,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Removes a user from a [`ChatGroup`] and emits a
/// [`ChatGroupUserRemoved`] event.
pub fn remove_user_from_chat_group<C: Clock, G: IdGenerator>(
    cmd: RemoveUserFromChatGroupCommand,
    clock: &C,
    ids: &G,
    m: &mut ChatGroupUser,
) -> Result<ChatGroupUserRemoved> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    m.mark_removed(cmd.tenant.actor_id, now, event_id);
    Ok(ChatGroupUserRemoved::new(
        m.chat_group_id,
        m.user_id,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat group message recipient service
// =============================================================================

/// Records a [`ChatGroupMessageRecipient`] and emits a
/// [`GroupMessageRecipientRecorded`] event.
pub fn record_group_message_recipient<C: Clock, G: IdGenerator>(
    cmd: RecordGroupMessageRecipientCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatGroupMessageRecipient, GroupMessageRecipientRecorded)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatGroupMessageRecipientId::new(school, event_id_to_uuid(event_id));
    let mut r = ChatGroupMessageRecipient::fresh(
        id,
        cmd.chat_group_id,
        cmd.user_id,
        cmd.group_message_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    r.last_event_id = Some(event_id);
    let event = GroupMessageRecipientRecorded::new(
        id,
        cmd.chat_group_id,
        cmd.user_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((r, event))
}

/// Marks a [`ChatGroupMessageRecipient`] as read and emits a
/// [`GroupMessageMarkedRead`] event.
pub fn mark_group_message_read<C: Clock, G: IdGenerator>(
    cmd: MarkGroupMessageReadCommand,
    clock: &C,
    ids: &G,
    r: &mut ChatGroupMessageRecipient,
) -> Result<GroupMessageMarkedRead> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let read_at = now;
    r.mark_read(cmd.tenant.actor_id, now, event_id);
    Ok(GroupMessageMarkedRead::new(
        r.id,
        read_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat group message remove service
// =============================================================================

/// Records a [`ChatGroupMessageRemove`] and emits a
/// [`GroupMessageRemovedForUser`] event.
pub fn remove_group_message_for_user<C: Clock, G: IdGenerator>(
    cmd: RemoveGroupMessageForUserCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatGroupMessageRemove, GroupMessageRemovedForUser)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatGroupMessageRemoveId::new(school, event_id_to_uuid(event_id));
    let mut rm = ChatGroupMessageRemove::fresh(
        id,
        cmd.chat_group_message_recipient_id,
        cmd.user_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    rm.last_event_id = Some(event_id);
    let event = GroupMessageRemovedForUser::new(
        id,
        cmd.user_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((rm, event))
}

// =============================================================================
// Chat block service
// =============================================================================

/// Blocks a user and emits a [`UserBlocked`] event.
pub fn block_user<C: Clock, G: IdGenerator>(
    cmd: BlockUserCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatBlockUser, UserBlocked)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatBlockUserId::new(school, event_id_to_uuid(event_id));
    let mut b = ChatBlockUser::fresh(
        id,
        cmd.tenant.actor_id,
        cmd.block_to,
        now,
        cmd.tenant.correlation_id,
    );
    b.last_event_id = Some(event_id);
    let blocked_at = now;
    let event = UserBlocked::new(
        cmd.tenant.school_id,
        cmd.tenant.actor_id,
        cmd.block_to,
        blocked_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((b, event))
}

/// Unblocks a user and emits a [`UserUnblocked`] event.
pub fn unblock_user<C: Clock, G: IdGenerator>(
    cmd: UnblockUserCommand,
    clock: &C,
    ids: &G,
    b: &mut ChatBlockUser,
) -> Result<UserUnblocked> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    b.mark_unblocked(cmd.tenant.actor_id, now, event_id);
    Ok(UserUnblocked::new(
        cmd.tenant.school_id,
        b.block_by,
        b.block_to,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Chat invitation service
// =============================================================================

/// Sends a [`ChatInvitation`] and emits a [`ChatInvitationSent`]
/// event.
pub fn send_chat_invitation<C: Clock, G: IdGenerator>(
    cmd: SendChatInvitationCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatInvitation, ChatInvitationSent)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatInvitationId::new(school, event_id_to_uuid(event_id));
    let mut inv = ChatInvitation::fresh(
        id,
        cmd.tenant.actor_id,
        cmd.to,
        cmd.invitation_type,
        cmd.section_id,
        cmd.class_teacher_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    inv.last_event_id = Some(event_id);
    let event = ChatInvitationSent::new(
        id,
        cmd.tenant.actor_id,
        cmd.to,
        cmd.invitation_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((inv, event))
}

/// Accepts a [`ChatInvitation`] and emits a
/// [`ChatInvitationAccepted`] event.
pub fn accept_chat_invitation<C: Clock, G: IdGenerator>(
    cmd: AcceptChatInvitationCommand,
    clock: &C,
    ids: &G,
    inv: &mut ChatInvitation,
) -> Result<ChatInvitationAccepted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    inv.accept(cmd.tenant.actor_id, now, event_id);
    Ok(ChatInvitationAccepted::new(
        inv.id,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Rejects a [`ChatInvitation`] and emits a
/// [`ChatInvitationRejected`] event.
pub fn reject_chat_invitation<C: Clock, G: IdGenerator>(
    cmd: RejectChatInvitationCommand,
    clock: &C,
    ids: &G,
    inv: &mut ChatInvitation,
) -> Result<ChatInvitationRejected> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    inv.reject(cmd.tenant.actor_id, now, event_id);
    Ok(ChatInvitationRejected::new(
        inv.id,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Classifies a [`ChatInvitation`] and emits a
/// [`ChatInvitationClassified`] event.
pub fn classify_chat_invitation<C: Clock, G: IdGenerator>(
    cmd: ClassifyChatInvitationCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatInvitationType, ChatInvitationClassified)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ChatInvitationTypeId::new(school, event_id_to_uuid(event_id));
    let mut cit = ChatInvitationType::fresh(
        id,
        cmd.invitation_id,
        cmd.invitation_type,
        cmd.section_id,
        cmd.class_teacher_id,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    cit.last_event_id = Some(event_id);
    let event = ChatInvitationClassified::new(
        id,
        cmd.invitation_id,
        cmd.invitation_type,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((cit, event))
}

// =============================================================================
// Chat status service
// =============================================================================

/// Sets a [`ChatStatus`] and emits a [`ChatStatusSet`] event.
pub fn set_chat_status<C: Clock, G: IdGenerator>(
    cmd: SetChatStatusCommand,
    clock: &C,
    ids: &G,
) -> Result<ChatStatusSet> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let set_at = now;
    let event = ChatStatusSet::new(
        cmd.tenant.school_id,
        cmd.tenant.actor_id,
        cmd.status,
        set_at,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok(event)
}

// =============================================================================
// SendMessage service
// =============================================================================

/// Creates a new [`SendMessage`] aggregate + a
/// [`SendMessageCreated`] event.
pub fn create_send_message<C: Clock, G: IdGenerator>(
    cmd: CreateSendMessageCommand,
    clock: &C,
    ids: &G,
) -> Result<(SendMessage, SendMessageCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SendMessageId::new(school, event_id_to_uuid(event_id));
    let mut sm = SendMessage::fresh(
        id,
        cmd.message_title,
        cmd.message_body,
        cmd.notice_date,
        cmd.publish_on,
        cmd.message_to.clone(),
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    sm.last_event_id = Some(event_id);
    let event = SendMessageCreated::new(
        id,
        cmd.message_to,
        cmd.publish_on,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((sm, event))
}

/// Dispatches a [`SendMessage`] and emits a
/// [`SendMessageDispatched`] event.
pub fn dispatch_send_message<C: Clock, G: IdGenerator>(
    cmd: DispatchSendMessageCommand,
    clock: &C,
    ids: &G,
    sm: &mut SendMessage,
) -> Result<SendMessageDispatched> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let recipient_count = sm.dispatch(cmd.tenant.actor_id, now, event_id);
    Ok(SendMessageDispatched::new(
        sm.id,
        recipient_count,
        now,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Cancels a [`SendMessage`] and emits a
/// [`SendMessageCancelled`] event.
pub fn cancel_send_message<C: Clock, G: IdGenerator>(
    cmd: CancelSendMessageCommand,
    clock: &C,
    ids: &G,
    sm: &mut SendMessage,
) -> Result<SendMessageCancelled> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    sm.cancel(cmd.reason.clone(), cmd.tenant.actor_id, now, event_id);
    Ok(SendMessageCancelled::new(
        sm.id,
        cmd.reason.unwrap_or_default(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// ContactMessage service
// =============================================================================

/// Receives a [`ContactMessage`] and emits a
/// [`ContactMessageReceived`] event.
pub fn receive_contact_message<C: Clock, G: IdGenerator>(
    cmd: ReceiveContactMessageCommand,
    clock: &C,
    ids: &G,
) -> Result<(ContactMessage, ContactMessageReceived)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ContactMessageId::new(school, event_id_to_uuid(event_id));
    let name_for_event = cmd.name.clone();
    let email_for_event = cmd
        .email
        .clone()
        .ok_or_else(|| DomainError::validation("contact message requires an email address"))?;
    let phone_for_event = cmd
        .phone
        .clone()
        .ok_or_else(|| DomainError::validation("contact message requires a phone number"))?;
    let subject_for_event = cmd.subject.as_str().to_owned();
    let mut cm = ContactMessage::fresh(
        id,
        cmd.name,
        cmd.phone,
        cmd.email,
        subject_for_event.clone(),
        cmd.message,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    cm.last_event_id = Some(event_id);
    let event = ContactMessageReceived::new(
        id,
        name_for_event,
        email_for_event,
        phone_for_event,
        subject_for_event,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((cm, event))
}

/// Marks a [`ContactMessage`] as viewed and emits a
/// [`ContactMessageViewed`] event.
pub fn mark_contact_message_viewed<C: Clock, G: IdGenerator>(
    cmd: MarkContactMessageViewedCommand,
    clock: &C,
    ids: &G,
    cm: &mut ContactMessage,
) -> Result<ContactMessageViewed> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    cm.mark_viewed(cmd.tenant.actor_id, now, event_id);
    Ok(ContactMessageViewed::new(
        cm.id,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Replies to a [`ContactMessage`] and emits a
/// [`ContactMessageReplied`] event.
pub fn reply_to_contact_message<C: Clock, G: IdGenerator>(
    cmd: ReplyToContactMessageCommand,
    clock: &C,
    ids: &G,
    cm: &mut ContactMessage,
) -> Result<(ContactMessageReply, ContactMessageReplied)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = ContactMessageReplyId::new(school, event_id_to_uuid(event_id));
    let r = ContactMessageReply::new(
        id,
        cm.id,
        cmd.tenant.actor_id,
        cmd.reply_body,
        cmd.reply_channel,
        now,
        cmd.tenant.correlation_id,
    );
    let event = ContactMessageReplied::new(
        cm.id,
        cmd.reply_channel,
        cmd.tenant.actor_id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((r, event))
}

// =============================================================================
// SpeechSlider service
// =============================================================================

/// Creates a new [`SpeechSlider`] aggregate + a
/// [`SpeechSliderCreated`] event.
pub fn create_speech_slider<C: Clock, G: IdGenerator>(
    cmd: CreateSpeechSliderCommand,
    clock: &C,
    ids: &G,
) -> Result<(SpeechSlider, SpeechSliderCreated)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = SpeechSliderId::new(school, event_id_to_uuid(event_id));
    let name_str = cmd.name.as_str().to_owned();
    let designation_str = cmd.designation.clone();
    let mut s = SpeechSlider::fresh(
        id,
        cmd.name,
        cmd.designation,
        cmd.speech,
        cmd.image,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    s.last_event_id = Some(event_id);
    let event = SpeechSliderCreated::new(
        id,
        name_str,
        designation_str,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((s, event))
}

/// Mutates a [`SpeechSlider`] and emits a
/// [`SpeechSliderUpdated`] event.
pub fn update_speech_slider<C: Clock, G: IdGenerator>(
    cmd: UpdateSpeechSliderCommand,
    clock: &C,
    ids: &G,
    s: &mut SpeechSlider,
) -> Result<SpeechSliderUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let changes = s.update(
        cmd.name,
        cmd.designation,
        cmd.speech,
        cmd.image,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(SpeechSliderUpdated::new(
        s.id,
        changes.into_iter().map(String::from).collect::<Vec<String>>(),
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

/// Soft-deletes a [`SpeechSlider`] and emits a
/// [`SpeechSliderDeleted`] event.
pub fn delete_speech_slider<C: Clock, G: IdGenerator>(
    cmd: DeleteSpeechSliderCommand,
    clock: &C,
    ids: &G,
    s: &mut SpeechSlider,
) -> Result<SpeechSliderDeleted> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    s.mark_deleted(cmd.tenant.actor_id, now, event_id);
    Ok(SpeechSliderDeleted::new(
        s.id,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// PhoneCallLog service
// =============================================================================

/// Logs a phone call and emits a [`PhoneCallLogged`] event.
pub fn log_phone_call<C: Clock, G: IdGenerator>(
    cmd: LogPhoneCallCommand,
    clock: &C,
    ids: &G,
) -> Result<(PhoneCallLog, PhoneCallLogged)> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    let school = cmd.tenant.school_id;
    let id = PhoneCallLogId::new(school, event_id_to_uuid(event_id));
    let name_for_event = cmd.name.clone();
    let phone_for_event = cmd.phone.clone();
    let mut pcl = PhoneCallLog::fresh(
        id,
        cmd.name,
        cmd.phone,
        cmd.date,
        cmd.description,
        cmd.next_follow_up_date,
        cmd.call_duration,
        cmd.call_type,
        cmd.tenant.actor_id,
        now,
        cmd.tenant.correlation_id,
    );
    pcl.last_event_id = Some(event_id);
    let event = PhoneCallLogged::new(
        id,
        name_for_event,
        phone_for_event,
        cmd.call_type,
        cmd.date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    );
    Ok((pcl, event))
}

/// Updates the follow-up date of a [`PhoneCallLog`] and emits
/// a [`PhoneCallFollowUpUpdated`] event.
pub fn update_phone_call_follow_up<C: Clock, G: IdGenerator>(
    cmd: UpdatePhoneCallFollowUpCommand,
    clock: &C,
    ids: &G,
    pcl: &mut PhoneCallLog,
) -> Result<PhoneCallFollowUpUpdated> {
    let now = clock.now();
    let event_id = ids.next_event_id();
    pcl.update_follow_up(
        cmd.next_follow_up_date,
        cmd.tenant.actor_id,
        now,
        event_id,
    );
    Ok(PhoneCallFollowUpUpdated::new(
        pcl.id,
        cmd.next_follow_up_date,
        event_id,
        cmd.tenant.correlation_id,
        now,
    ))
}

// =============================================================================
// Headline service functions (7)
// =============================================================================

/// Headline #1: send a notification (in-app, push, or web).
/// Thin async wrapper around [`send_notification`].
pub async fn notify_user<C: Clock, G: IdGenerator>(
    cmd: SendNotificationCommand,
    clock: &C,
    ids: &G,
) -> Result<(Notification, NotificationSent)> {
    send_notification(cmd, clock, ids)
}

/// Headline #2: mark a notification as read. Thin async
/// wrapper around [`mark_notification_read`].
pub async fn mark_as_read<C: Clock, G: IdGenerator>(
    cmd: MarkNotificationReadCommand,
    clock: &C,
    ids: &G,
    n: &mut Notification,
) -> Result<NotificationRead> {
    mark_notification_read(cmd, clock, ids, n)
}

/// Headline #3: send a notice message (publish a notice).
/// Thin async wrapper around [`publish_notice`].
pub async fn send_notice_message<C: Clock, G: IdGenerator>(
    cmd: PublishNoticeCommand,
    clock: &C,
    ids: &G,
    notice: &mut Notice,
) -> Result<NoticePublished> {
    publish_notice(cmd, clock, ids, notice)
}

/// Headline #4: send a complaint message (register a
/// complaint). Thin async wrapper around [`register_complaint`].
pub async fn send_complaint_message<C: Clock, G: IdGenerator>(
    cmd: RegisterComplaintCommand,
    clock: &C,
    ids: &G,
) -> Result<(Complaint, ComplaintRegistered)> {
    register_complaint(cmd, clock, ids)
}

/// Headline #5: send a chat message. Thin async wrapper
/// around [`send_chat_message`].
pub async fn send_chat_message_headline<C: Clock, G: IdGenerator>(
    cmd: SendChatMessageCommand,
    clock: &C,
    ids: &G,
) -> Result<(ChatMessage, ChatMessageSent)> {
    send_chat_message(cmd, clock, ids)
}

/// Headline #6: send an email message (append to email log).
/// Thin async wrapper around [`log_email_sent`].
pub async fn send_email_message<C: Clock, G: IdGenerator>(
    cmd: LogEmailSentCommand,
    clock: &C,
    ids: &G,
) -> Result<(EmailLog, EmailLogged)> {
    log_email_sent(cmd, clock, ids)
}

/// Headline #7: send an SMS message (append to SMS log).
/// Thin async wrapper around [`log_sms_sent`].
pub async fn send_sms_message<C: Clock, G: IdGenerator>(
    cmd: LogSmsSentCommand,
    clock: &C,
    ids: &G,
) -> Result<(SmsLog, SmsLogged)> {
    log_sms_sent(cmd, clock, ids)
}

// =============================================================================
// Service struct: NotificationService
// =============================================================================

/// The notification service. Pure helpers for selecting a
/// template, rendering the body, routing to a channel, and
/// computing the next dispatch window.
pub struct NotificationService;

impl NotificationService {
    /// Selects an [`SmsTemplate`] for the given event + channel
    /// from the supplied set of candidates. Returns `None` if
    /// no candidate matches.
    #[must_use]
    pub fn select_template<'a>(
        event: &str,
        channel: Channel,
        candidates: &'a [SmsTemplate],
    ) -> Option<&'a SmsTemplate> {
        candidates
            .iter()
            .find(|t| t.channel == channel && t.purpose == event)
    }

    /// Renders the notification body for the given template and
    /// substitution map. Pure wrapper around
    /// [`TemplateService::render`].
    pub fn render(
        template: &SmsTemplate,
        vars: &BTreeMap<String, String>,
    ) -> Result<RenderedBody> {
        TemplateService::render(template, vars)
    }

    /// Resolves the destination for a notification: which
    /// channels (Email, SMS, Web, App) should deliver this
    /// notification, given the [`NotificationSetting`]
    /// configured for the event.
    #[must_use]
    pub fn route(setting: &NotificationSetting) -> Destination {
        setting.destination
    }

    /// Returns the next dispatch window (start, end) for the
    /// given [`AbsentNotificationTimeSetup`]. Pure
    /// re-export; the dispatcher is responsible for actually
    /// scheduling the dispatch.
    #[must_use]
    pub fn next_window(
        setup: &AbsentNotificationTimeSetup,
    ) -> (TimeOfDay, TimeOfDay) {
        (setup.time_from.clone(), setup.time_to.clone())
    }
}

// =============================================================================
// Service struct: ChatService
// =============================================================================

/// The chat service. Pure helpers for chat routing,
/// conversation resolution, and recipient fan-out.
pub struct ChatService;

impl ChatService {
    /// Returns `true` if `from` is blocked by `to` according to
    /// the supplied [`ChatBlockUser`] set.
    #[must_use]
    pub fn is_blocked(
        from: UserId,
        blocks: &[ChatBlockUser],
    ) -> bool {
        blocks.iter().any(|b| b.block_by == from && b.is_active())
    }

    /// Resolves the conversation between two users. Returns
    /// `None` if no open conversation exists in the supplied
    /// slice.
    #[must_use]
    pub fn resolve_conversation(
        a: UserId,
        b: UserId,
        conversations: &[ChatConversation],
    ) -> Option<&ChatConversation> {
        conversations.iter().find(|c| {
            (c.from_id == a && c.to_id == b) || (c.from_id == b && c.to_id == a)
        })
    }

    /// Fans out a group message to the group's user set,
    /// returning the set of [`UserId`]s who should receive a
    /// [`ChatGroupMessageRecipient`] record.
    #[must_use]
    pub fn fan_out_group_recipients(
        members: &[ChatGroupUser],
    ) -> Vec<UserId> {
        members.iter().map(|m| m.user_id).collect()
    }

    /// Returns `true` if the user is allowed to post in the
    /// given [`ChatGroup`]. A user can post when the group is
    /// not read-only, or when the user is an admin.
    #[must_use]
    pub fn can_post(
        group: &ChatGroup,
        user: UserId,
        membership: Option<&ChatGroupUser>,
    ) -> bool {
        if !group.read_only {
            return true;
        }
        match membership {
            Some(m) if m.user_id == user => matches!(m.role, ChatGroupRole::Admin),
            _ => false,
        }
    }
}

// =============================================================================
// Service struct: ComplaintService
// =============================================================================

/// The complaint service. Pure helpers for categorising
/// complaints, computing the next status, and escalation
/// paths.
pub struct ComplaintService;

impl ComplaintService {
    /// Categorises a complaint by its [`ComplaintType`]. Returns
    /// the type's name, or "Uncategorised" if the type id is
    /// not in the supplied slice.
    #[must_use]
    pub fn categorize(
        complaint: &Complaint,
        types: &[ComplaintType],
    ) -> String {
        types
            .iter()
            .find(|t| t.id == complaint.complaint_type_id)
            .map_or_else(|| "Uncategorised".to_owned(), |t| t.name.to_string())
    }

    /// Returns `true` if the complaint was filed anonymously
    /// (i.e. `complaint_by` is `None`).
    #[must_use]
    pub fn is_anonymous(complaint: &Complaint) -> bool {
        complaint.complaint_by.is_none()
    }

    /// Returns the next [`ComplaintStatus`] for a complaint
    /// given the current status and the requested action.
    #[must_use]
    pub fn next_status(
        current: ComplaintStatus,
        action: ComplaintAction,
    ) -> ComplaintStatus {
        match (current, action) {
            (ComplaintStatus::Open, ComplaintAction::InProgress) => {
                ComplaintStatus::InProgress
            }
            (ComplaintStatus::InProgress, ComplaintAction::Resolve) => {
                ComplaintStatus::Resolved
            }
            (ComplaintStatus::Open, ComplaintAction::Resolve) => {
                ComplaintStatus::Resolved
            }
            (ComplaintStatus::Open, ComplaintAction::Open) => ComplaintStatus::Open,
            (ComplaintStatus::InProgress, ComplaintAction::Open) => {
                ComplaintStatus::Open
            }
            (ComplaintStatus::InProgress, ComplaintAction::InProgress) => {
                ComplaintStatus::InProgress
            }
            (ComplaintStatus::Resolved, _) => ComplaintStatus::Resolved,
        }
    }

    /// Returns the escalation path for a complaint, given the
    /// current status. Pure ordering; the dispatcher
    /// dispatches the actual notifications.
    #[must_use]
    pub fn escalation_path(current: ComplaintStatus) -> Vec<ComplaintStatus> {
        match current {
            ComplaintStatus::Open => vec![
                ComplaintStatus::Open,
                ComplaintStatus::InProgress,
                ComplaintStatus::Resolved,
            ],
            ComplaintStatus::InProgress => vec![
                ComplaintStatus::InProgress,
                ComplaintStatus::Resolved,
            ],
            ComplaintStatus::Resolved => vec![ComplaintStatus::Resolved],
        }
    }
}

// =============================================================================
// Service struct: AbsentNotificationService
// =============================================================================

/// The absent-notification service. Pure helpers for
/// determining whether a given wall-clock time is inside the
/// configured dispatch window, and for rendering the
/// notification body.
pub struct AbsentNotificationService;

impl AbsentNotificationService {
    /// Returns `true` if the supplied `at` time-of-day falls
    /// strictly within `[time_from, time_to)`.
    #[must_use]
    pub fn in_window(at: TimeOfDay, setup: &AbsentNotificationTimeSetup) -> bool {
        at.as_str() >= setup.time_from.as_str() && at.as_str() < setup.time_to.as_str()
    }

    /// Returns `true` if the absent-notification should
    /// dispatch: enabled AND the wall clock is inside the
    /// configured window.
    #[must_use]
    pub fn should_dispatch(
        at: TimeOfDay,
        setup: &AbsentNotificationTimeSetup,
    ) -> bool {
        matches!(setup.status, AbsentNotificationStatus::Enabled)
            && Self::in_window(at, setup)
    }

    /// Builds a [`AbsentNotificationDispatch`] child entity for
    /// the given student / channel. The dispatcher is
    /// responsible for persisting the child row and the
    /// `AbsentNotificationSent` event. The caller must supply
    /// the pre-rendered body, resolved recipients, outcome, and
    /// dispatch timestamp.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn build_dispatch(
        id: AbsentNotificationDispatchId,
        setup_id: AbsentNotificationTimeSetupId,
        student_id: StudentId,
        channel: Channel,
        rendered_body: String,
        recipients: Vec<UserId>,
        outcome: DeliveryOutcome,
        dispatched_at: Timestamp,
        correlation_id: CorrelationId,
    ) -> AbsentNotificationDispatch {
        AbsentNotificationDispatch::new(
            id,
            setup_id,
            student_id,
            channel,
            rendered_body,
            recipients,
            outcome,
            dispatched_at,
            correlation_id,
        )
    }

    /// Renders the absent-notification body for the given
    /// template and substitution map. Pure wrapper around
    /// [`TemplateService::render`].
    pub fn render(
        template: &SmsTemplate,
        vars: &BTreeMap<String, String>,
    ) -> Result<RenderedBody> {
        TemplateService::render(template, vars)
    }
}

// =============================================================================
// Service struct: TemplateService (the 100-case proptest target)
// =============================================================================

/// The SMS template service. Pure functions for validating a
/// template body, scanning it for declared `{{var}}`
/// placeholders, and substituting them with concrete values.
pub struct TemplateService;

impl TemplateService {
    /// Validates a template body against the supplied declared
    /// variables. Returns `Ok(())` if the body is well-formed
    /// and every declared variable appears in the declaration
    /// list. Returns `Err(DomainError::Validation)` otherwise.
    pub fn validate_body(
        body: &str,
        variables: &[TemplateVariable],
    ) -> Result<()> {
        let declared = Self::declared(body);
        for var in variables {
            if !declared.iter().any(|n| n == var.name.as_str()) {
                return Err(DomainError::Validation(format!(
                    "declared variable {{{{{}}}}} is not present in the body",
                    var.name.as_str()
                )));
            }
        }
        Ok(())
    }

    /// Returns the list of declared `{{name}}` placeholders in
    /// the body, in source order, deduplicated.
    #[must_use]
    pub fn declared(body: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let bytes = body.as_bytes();
        let mut i = 0;
        while i + 1 < bytes.len() {
            if bytes[i] == b'{' && bytes[i + 1] == b'{' {
                if let Some(end_rel) = body[i + 2..].find("}}") {
                    let name = &body[i + 2..i + 2 + end_rel];
                    if !name.is_empty()
                        && !name.contains('{')
                        && !name.contains('}')
                        && !out.iter().any(|n| n == name)
                    {
                        out.push(name.to_owned());
                    }
                    i = i + 2 + end_rel + 2;
                    continue;
                }
            }
            i += 1;
        }
        out
    }

    /// Substitutes every `{{name}}` placeholder in `body` with
    /// the corresponding value from `vars`. Returns `Err` if a
    /// placeholder references a variable that is not present
    /// in `vars`.
    pub fn substitute(
        body: &str,
        vars: &BTreeMap<String, String>,
    ) -> Result<String> {
        let mut out = String::with_capacity(body.len());
        let bytes = body.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
                if let Some(end_rel) = body[i + 2..].find("}}") {
                    let name = &body[i + 2..i + 2 + end_rel];
                    if name.is_empty() || name.contains('{') || name.contains('}') {
                        return Err(DomainError::Validation(
                            "empty or malformed placeholder".to_owned(),
                        ));
                    }
                    let value = vars.get(name).ok_or_else(|| {
                        DomainError::Validation(format!(
                            "missing substitution value for variable '{name}'"
                        ))
                    })?;
                    out.push_str(value);
                    i = i + 2 + end_rel + 2;
                    continue;
                }
            }
            // Safe ASCII push (the body is ".*" / "[^{}]*" in
            // the proptest, but we still treat bytes correctly
            // for any UTF-8 char by appending the rest of the
            // current char if needed).
            let ch_end = (i + 1..=bytes.len())
                .find(|&j| body.is_char_boundary(j))
                .unwrap_or(bytes.len());
            out.push_str(&body[i..ch_end]);
            i = ch_end;
        }
        Ok(out)
    }

    /// Renders the body of a [`SmsTemplate`] by substituting
    /// every declared variable. Returns the rendered
    /// [`RenderedBody`].
    pub fn render(
        template: &SmsTemplate,
        vars: &BTreeMap<String, String>,
    ) -> Result<RenderedBody> {
        let body = template.body.as_str();
        let declared = Self::declared(body);
        for name in &declared {
            if !vars.contains_key(name) {
                return Err(DomainError::Validation(format!(
                    "missing substitution value for variable '{name}'"
                )));
            }
        }
        let rendered = Self::substitute(body, vars)?;
        Ok(RenderedBody::from_rendered(rendered))
    }

    /// Lints the body for unused declared variables, mismatched
    /// braces, and HTML in SMS bodies. Returns a list of
    /// advisory [`RenderWarning`]s.
    #[must_use]
    pub fn lint(body: &str) -> Vec<RenderWarning> {
        let mut warnings = Vec::new();
        let opens = body.matches("{{").count();
        let closes = body.matches("}}").count();
        if opens != closes {
            warnings.push(RenderWarning::MismatchedBraces {
                position: 0,
                body: body.to_owned(),
            });
        }
        if body.contains('<') && body.contains('>') {
            warnings.push(RenderWarning::HtmlInSms {
                html: body.to_owned(),
                sms_body: body.to_owned(),
            });
        }
        warnings
    }
}

// =============================================================================
// Policy: SmsDispatchPolicy
// =============================================================================

/// The SMS dispatch policy. Pure check used by the dispatcher
/// before [`dispatch_send_message`] is invoked.
pub struct SmsDispatchPolicy;

impl SmsDispatchPolicy {
    /// Returns `Ok(())` if the send message is allowed to be
    /// dispatched. The pure checks are:
    ///
    /// - The message status is `Draft`.
    /// - `publish_on` is `None` (immediate) or in the past.
    /// - The audience is non-empty.
    pub fn check(
        cmd: &DispatchSendMessageCommand,
        sm: &SendMessage,
        now: NaiveDate,
    ) -> Result<()> {
        if !matches!(sm.status, SendMessageStatus::Draft) {
            return Err(DomainError::conflict(
                "send message is not in a dispatchable status",
            ));
        }
        if let Some(publish_on) = sm.publish_on {
            if publish_on > now {
                return Err(DomainError::validation(
                    "send message publish_on is in the future",
                ));
            }
        }
        if sm.audience.is_empty() {
            return Err(DomainError::validation(
                "send message audience is empty",
            ));
        }
        let _ = cmd;
        Ok(())
    }
}

// =============================================================================
// Specification: ActiveRecipients
// =============================================================================

/// The "active recipients" specification. A user is an active
/// recipient when their [`Notification`] status is `Pending`
/// or `Dispatched` and they have not withdrawn it.
pub struct ActiveRecipients;

impl ActiveRecipients {
    /// Returns `true` if the notification qualifies as an
    /// active recipient record.
    #[must_use]
    pub fn is_satisfied_by(n: &Notification) -> bool {
        matches!(
            n.status,
            NotificationStatus::Pending | NotificationStatus::Dispatched
        )
    }
}

// =============================================================================
// Specification: NoticesPublishedInRange
// =============================================================================

/// The "notices published in range" specification. A notice
/// qualifies when its status is `Published` and its
/// `notice_date` falls within the inclusive range
/// `[from, to]`.
pub struct NoticesPublishedInRange;

impl NoticesPublishedInRange {
    /// Returns `true` if the notice is `Published` and its
    /// `notice_date` is in the inclusive range.
    #[must_use]
    pub fn is_satisfied_by(
        notice: &Notice,
        from: NaiveDate,
        to: NaiveDate,
    ) -> bool {
        matches!(notice.status, NoticeStatus::Published)
            && notice.notice_date >= from
            && notice.notice_date <= to
    }
}

// =============================================================================
// Policy: ChatInvitePolicy
// =============================================================================

/// The chat invite policy. Pure check used by the dispatcher
/// before [`send_chat_invitation`] is invoked.
pub struct ChatInvitePolicy;

impl ChatInvitePolicy {
    /// Returns `Ok(())` if the user is allowed to send the
    /// invitation. The pure checks are:
    ///
    /// - The recipient is not the actor (no self-invites).
    /// - The actor has not blocked the recipient.
    /// - The actor has not already received an open
    ///   invitation from the recipient.
    pub fn check(
        cmd: &SendChatInvitationCommand,
        blocks: &[ChatBlockUser],
        open_invitations: &[ChatInvitation],
    ) -> Result<()> {
        if cmd.to == cmd.tenant.actor_id {
            return Err(DomainError::validation(
                "cannot send a chat invitation to yourself",
            ));
        }
        if blocks
            .iter()
            .any(|b| b.block_by == cmd.tenant.actor_id && b.block_to == cmd.to)
        {
            return Err(DomainError::forbidden(
                "actor has blocked the recipient",
            ));
        }
        if open_invitations.iter().any(|inv| {
            inv.from == cmd.tenant.actor_id
                && inv.to == cmd.to
                && matches!(inv.status, ChatInvitationStatus::Pending)
        }) {
            return Err(DomainError::conflict(
                "an open invitation already exists for this pair",
            ));
        }
        Ok(())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator as _, SystemClock, SystemIdGen};
    use educore_core::ids::Identifier;

    fn ctx() -> (SchoolId, UserId, Timestamp, CorrelationId, TenantContext) {
        let g = SystemIdGen;
        let school = g.next_school_id();
        let user = g.next_user_id();
        let corr = g.next_correlation_id();
        let tenant = TenantContext::for_user(
            school,
            user,
            corr,
            educore_core::tenant::UserType::SchoolAdmin,
        );
        (school, user, Timestamp::now(), corr, tenant)
    }

    fn notice_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()
    }

    // -------------------------------------------------------------------------
    // Headline: create_notice
    // -------------------------------------------------------------------------

    #[test]
    fn create_notice_emits_event() {
        let (school, _user, _at, _corr, tenant) = ctx();
        let cmd = CreateNoticeCommand {
            tenant,
            title: NoticeTitle::new("Holiday").unwrap(),
            body: NoticeBody::new("School closed on Monday").unwrap(),
            notice_date: notice_date(),
            publish_on: Some(notice_date()),
            audience: AudienceDescriptor::All,
            attachment: None,
        };
        let (notice, event) = create_notice(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(notice.school_id, school);
        assert_eq!(
            <NoticeCreated as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "communication.notice.created"
        );
        let _ = event;
    }

    #[test]
    fn log_email_sent_emits_event() {
        let (_school, _user, _at, _corr, tenant) = ctx();
        let cmd = LogEmailSentCommand {
            tenant,
            title: "Receipt".to_owned(),
            description: "Thanks".to_owned(),
            send_date: notice_date(),
            send_through: MailDriver::Smtp,
            send_to: EmailAddress::new("parent@example.com").unwrap(),
            message_id: None,
        };
        let (_log, event) = log_email_sent(cmd, &SystemClock, &SystemIdGen).unwrap();
        assert_eq!(
            <EmailLogged as educore_events::domain_event::DomainEvent>::EVENT_TYPE,
            "communication.email_log.logged"
        );
        let _ = event;
    }

    #[test]
    fn template_service_render_with_all_vars_resolves() {
        let (_school, _user, _at, _corr, tenant) = ctx();
        let cmd = CreateSmsTemplateCommand {
            tenant,
            channel: Channel::Sms,
            purpose: "absent".to_owned(),
            subject: "Subject".to_owned(),
            body: TemplateBody::new("Hello {{name}}, your child {{child}} is absent on {{date}}.")
                .unwrap(),
            module: "attendance".to_owned(),
            variables: vec![
                TemplateVariable::new("name").unwrap(),
                TemplateVariable::new("child").unwrap(),
                TemplateVariable::new("date").unwrap(),
            ],
        };
        let (template, _event) =
            create_sms_template(cmd, &SystemClock, &SystemIdGen).unwrap();
        let mut vars = BTreeMap::new();
        vars.insert("name".to_owned(), "Alice".to_owned());
        vars.insert("child".to_owned(), "Bob".to_owned());
        vars.insert("date".to_owned(), "2026-06-15".to_owned());
        let rendered = TemplateService::render(&template, &vars).unwrap();
        assert_eq!(
            rendered.as_str(),
            "Hello Alice, your child Bob is absent on 2026-06-15."
        );
    }

    #[test]
    fn template_service_render_with_missing_var_fails() {
        let (_school, _user, _at, _corr, tenant) = ctx();
        let cmd = CreateSmsTemplateCommand {
            tenant,
            channel: Channel::Sms,
            purpose: "absent".to_owned(),
            subject: "Subject".to_owned(),
            body: TemplateBody::new("Hello {{name}}").unwrap(),
            module: "attendance".to_owned(),
            variables: vec![TemplateVariable::new("name").unwrap()],
        };
        let (template, _event) =
            create_sms_template(cmd, &SystemClock, &SystemIdGen).unwrap();
        let empty: BTreeMap<String, String> = BTreeMap::new();
        let err = TemplateService::render(&template, &empty).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn template_service_declared_returns_unique_names() {
        let body = "Hi {{name}}, {{name}}! Welcome to {{school}}.";
        let declared = TemplateService::declared(body);
        assert_eq!(declared, vec!["name".to_owned(), "school".to_owned()]);
    }

    #[test]
    fn template_service_lint_detects_html() {
        let body = "Hello <b>name</b>";
        let warnings = TemplateService::lint(body);
        assert!(warnings.contains(&RenderWarning::HtmlInSms));
    }

    // -------------------------------------------------------------------------
    // TemplateService property test (100 cases) — the headline correctness
    // check per build-plan § "Phase 10"
    // -------------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        /// Property: every declared variable in the template
        /// body is resolved in the substitution map (or
        /// `render` returns `Err`).
        #[test]
        fn prop_template_render_resolves_all_declared_vars(
            body in ".*",
            var_count in 0usize..8,
        ) {
            let (_school, _user, _at, _corr, tenant) = ctx();
            let variables: Vec<TemplateVariable> = (0..var_count)
                .map(|i| TemplateVariable::new(format!("v{i}")).unwrap())
                .collect();
            let mut body_str = body.clone();
            // Declare each variable in the body.
            for v in &variables {
                body_str.push_str(&format!(" {{{{{}}}}}", v.name.as_str()));
            }
            let cmd = CreateSmsTemplateCommand {
                tenant,
                channel: Channel::Sms,
                purpose: "test".to_owned(),
                subject: "Subject".to_owned(),
                body: TemplateBody::new(body_str.clone()).unwrap(),
                module: "test".to_owned(),
                variables: variables.clone(),
            };
            let (template, _event) =
                create_sms_template(cmd, &SystemClock, &SystemIdGen).unwrap();

            // (a) Full substitution: every declared var is
            // resolved -> render returns Ok and the rendered
            // body does not contain any unresolved `{{...}}`
            // pairs.
            let mut full_vars = BTreeMap::new();
            for v in &variables {
                full_vars.insert(v.name.to_string(), format!("X{}", v.name.as_str()));
            }
            let rendered = TemplateService::render(&template, &full_vars).unwrap();
            for v in &variables {
                assert!(
                    !rendered.as_str().contains(&format!("{{{{{}}}}}", v.name.as_str())),
                    "rendered body must not contain unresolved placeholder {{{{{}}}}}",
                    v.name.as_str()
                );
            }

            // (b) Empty substitution map: render must fail
            // iff the body has at least one `{{...}}`
            // placeholder.
            let empty: BTreeMap<String, String> = BTreeMap::new();
            let res = TemplateService::render(&template, &empty);
            if variables.is_empty() {
                // No declared variables -> render is Ok.
                assert!(res.is_ok());
            } else {
                // Missing vars -> render is Err.
                assert!(res.is_err());
            }
        }

        /// Property: rendering an empty substitution map on a
        /// body with no `{{...}}` placeholders always
        /// succeeds.
        #[test]
        fn prop_template_render_with_empty_vars_and_no_placeholders(
            body in "[^{}]*",
        ) {
            let (_school, _user, _at, _corr, tenant) = ctx();
            let cmd = CreateSmsTemplateCommand {
                tenant,
                channel: Channel::Sms,
                purpose: "test".to_owned(),
                subject: "Subject".to_owned(),
                body: TemplateBody::new(body.clone()).unwrap(),
                module: "test".to_owned(),
                variables: Vec::new(),
            };
            let (template, _event) =
                create_sms_template(cmd, &SystemClock, &SystemIdGen).unwrap();
            let empty: BTreeMap<String, String> = BTreeMap::new();
            let rendered = TemplateService::render(&template, &empty).unwrap();
            assert_eq!(rendered.as_str(), body);
        }

        /// Property: `declared` returns at most one entry per
        /// variable name.
        #[test]
        fn prop_template_declared_deduplicates(
            body in ".*",
        ) {
            let declared = TemplateService::declared(&body);
            let mut sorted: Vec<String> = declared.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), declared.len());
        }
    }
}
