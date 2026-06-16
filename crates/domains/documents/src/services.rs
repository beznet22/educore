//! Documents-domain service factories.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === FormDownload services section begin (owner: 3A) ===

use std::sync::Arc;

use bytes::Bytes;
use chrono::NaiveDate;

use educore_audit::writer::{AuditAction, AuditTarget, AuditWriter};
use educore_core::error::DomainError;
use educore_core::ids::EventId;
use educore_core::value_objects::Timestamp;
use educore_events::domain_event::DomainEvent;
use educore_events::errors::EventError;
use educore_events::event_bus::EventBus;
use educore_rbac::services::CapabilityCheck;
use educore_rbac::value_objects::Capability;

use crate::aggregate::{FormDownload, NewFormDownload, UpdateFormDownload};
use crate::commands::{DeleteFormCommand, UpdateFormCommand, UploadFormCommand};
use crate::errors::DocumentsError;
use crate::events::{FormDeleted, FormUpdated, FormUploaded};
use crate::repository::FormDownloadRepository;
use crate::value_objects::{FileReference, Url};

/// Pure helpers (no I/O) for the `FormDownload` aggregate.
pub struct FormService;

impl FormService {
    /// Validates that the form has at least one of `link` or
    /// `file`. Returns
    /// [`DocumentsError::FormHasNoContent`] when neither is set.
    pub fn validate_content(
        link: Option<&Url>,
        file: Option<&FileReference>,
    ) -> Result<(), DocumentsError> {
        if link.is_none() && file.is_none() {
            return Err(DocumentsError::FormHasNoContent);
        }
        Ok(())
    }

    /// True if the form is visible to the public.
    #[must_use]
    pub fn is_public(form: &FormDownload) -> bool {
        form.is_public()
    }

    /// True if the form has at least one of `link` or `file`
    /// set (i.e. it is deliverable).
    #[must_use]
    pub fn is_deliverable(form: &FormDownload) -> bool {
        form.is_deliverable()
    }

    /// True if the form's `publish_date` is on or before the
    /// given date.
    #[must_use]
    pub fn matches_publish_date(form: &FormDownload, date: NaiveDate) -> bool {
        form.publish_date.0 <= date
    }
}

/// Map an engine-wide [`DomainError`] into the documents crate's
/// [`DocumentsError`]. Used by the service factories to
/// `?`-propagate errors from the audit / bus / capability-check
/// / storage ports.
impl From<DomainError> for DocumentsError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::Forbidden(msg) | DomainError::TenantViolation(msg) => {
                DocumentsError::Forbidden(msg)
            }
            DomainError::Conflict(msg) => DocumentsError::Conflict(msg),
            DomainError::Validation(msg) | DomainError::NotFound(msg) => {
                DocumentsError::Validation(msg)
            }
            DomainError::NotSupported(msg) => DocumentsError::Validation(msg),
            DomainError::Infrastructure(src) => {
                DocumentsError::Infrastructure(src.to_string())
            }
        }
    }
}

/// Map a bus-port [`EventError`] into [`DocumentsError`].
impl From<EventError> for DocumentsError {
    fn from(err: EventError) -> Self {
        DocumentsError::Infrastructure(err.to_string())
    }
}

/// JSON snapshot of a form, used for the audit row's
/// `before` / `after` columns. A `serde_json` failure falls
/// back to an empty payload (audit rows are best-effort; a
/// missing snapshot is preferable to a failed write that would
/// roll back the underlying mutation).
fn snapshot(form: &FormDownload) -> Bytes {
    Bytes::from(serde_json::to_vec(form).unwrap_or_default())
}

/// Service factory: upload a new [`FormDownload`]. Capability-
/// gates on [`Capability::FormDownloadUpload`], constructs the
/// aggregate, persists it via the repository, writes the audit
/// row, and publishes the [`FormUploaded`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn upload_form_service<R, B>(
    cmd: UploadFormCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<FormDownload, DocumentsError>
where
    R: FormDownloadRepository + 'static,
    B: EventBus + 'static,
{
    cap.has(&cmd.tenant, Capability::FormDownloadUpload).await?;
    let tenant = cmd.tenant.clone();
    let new: NewFormDownload = cmd.into_new_form_download();
    let form = FormDownload::new(new)?;
    repo.insert(&form).await?;
    let after = snapshot(&form);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::FormDownload(form.id.as_uuid()),
            None,
            Some(after),
        )
        .await?;
    let event = FormUploaded::new(
        &form,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(form)
}

/// Service factory: update an existing [`FormDownload`].
/// Capability-gates on [`Capability::FormDownloadUpdate`],
/// re-loads the form, applies the changes, persists the
/// updated row, writes the audit row, and publishes the
/// [`FormUpdated`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn update_form_service<R, B>(
    cmd: UpdateFormCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<FormDownload, DocumentsError>
where
    R: FormDownloadRepository + 'static,
    B: EventBus + 'static,
{
    cap.has(&cmd.tenant, Capability::FormDownloadUpdate).await?;
    let mut form = repo
        .get(cmd.form_id)
        .await?
        .ok_or(DocumentsError::FormNotFound(cmd.form_id.as_uuid()))?;
    let before = snapshot(&form);

    let mut changes: Vec<String> = Vec::new();
    if cmd.title.is_some() {
        changes.push("title".to_owned());
    }
    if cmd.short_description.is_some() {
        changes.push("short_description".to_owned());
    }
    if cmd.publish_date.is_some() {
        changes.push("publish_date".to_owned());
    }
    if cmd.link.is_some() {
        changes.push("link".to_owned());
    }
    if cmd.file.is_some() {
        changes.push("file".to_owned());
    }
    if cmd.show_public.is_some() {
        changes.push("show_public".to_owned());
    }

    let tenant = cmd.tenant.clone();
    let event_id = EventId(uuid::Uuid::now_v7());
    let update_cmd: UpdateFormDownload = cmd.into_update_form_download(event_id);
    form.update(update_cmd)?;
    repo.update(&form).await?;
    let after = snapshot(&form);
    audit
        .write(
            &tenant,
            AuditAction::Update,
            AuditTarget::FormDownload(form.id.as_uuid()),
            Some(before),
            Some(after),
        )
        .await?;
    let event = FormUpdated::new(
        &form,
        changes,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(form)
}

/// Service factory: soft-delete a [`FormDownload`]. Capability-
/// gates on [`Capability::FormDownloadDelete`], re-loads the
/// form, flips `active_status` to `false`, persists the updated
/// row, writes the audit row, and publishes the [`FormDeleted`]
/// event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn delete_form_service<R, B>(
    cmd: DeleteFormCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<(), DocumentsError>
where
    R: FormDownloadRepository + 'static,
    B: EventBus + 'static,
{
    cap.has(&cmd.tenant, Capability::FormDownloadDelete).await?;
    let mut form = repo
        .get(cmd.form_id)
        .await?
        .ok_or(DocumentsError::FormNotFound(cmd.form_id.as_uuid()))?;
    let before = snapshot(&form);
    let tenant = cmd.tenant.clone();
    form.soft_delete(tenant.actor_id, Timestamp::now())?;
    repo.update(&form).await?;
    audit
        .write(
            &tenant,
            AuditAction::Delete,
            AuditTarget::FormDownload(form.id.as_uuid()),
            Some(before),
            None,
        )
        .await?;
    let event = FormDeleted::new(
        &form,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(())
}

// === FormDownload services section end ===

// === PostalDispatch services section begin (owner: 3B) ===
pub struct PostalService;
pub fn dispatch_postal_service() {}
pub fn update_postal_dispatch_service() {}
pub fn delete_postal_dispatch_service() {}
// === PostalDispatch services section end ===

// === PostalReceive services section begin (owner: 3C) ===
pub fn receive_postal_service() {}
pub fn update_postal_receive_service() {}
pub fn delete_postal_receive_service() {}
pub fn track_postal_service() {}
// === PostalReceive services section end ===
