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

/// Authorize a single capability. Returns
/// [`DocumentsError::Forbidden`] when the actor does not hold
/// the capability. The error message includes the wire-form
/// capability name so audit rows are self-describing.
///
/// Note: `CapabilityCheck::has` returns
/// `Result<bool, DomainError>` — the `Ok(false)` case is NOT
/// an error and must be checked explicitly. This helper
/// centralises the check so every service factory enforces
/// the capability gate identically.
async fn require_capability(
    cap: &dyn CapabilityCheck,
    tenant: &educore_core::tenant::TenantContext,
    capability: Capability,
) -> Result<(), DocumentsError> {
    if cap.has(tenant, capability).await? {
        Ok(())
    } else {
        Err(DocumentsError::Forbidden(format!(
            "missing capability {}",
            capability.as_str()
        )))
    }
}

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
    require_capability(cap, &cmd.tenant, Capability::FormDownloadUpload).await?;
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
    require_capability(cap, &cmd.tenant, Capability::FormDownloadUpdate).await?;
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
    require_capability(cap, &cmd.tenant, Capability::FormDownloadDelete).await?;
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

// 3A above already imports `std::sync::Arc`, `bytes::Bytes`,
// `educore_audit::writer::{AuditAction, AuditTarget,
// AuditWriter}`, `educore_core::value_objects::Timestamp`,
// `educore_events::domain_event::DomainEvent`,
// `educore_events::event_bus::EventBus`,
// `educore_rbac::services::CapabilityCheck`,
// `educore_rbac::value_objects::Capability`, and
// `educore_core::ids::EventId` (the latter via the section's
// `use educore_core::ids::EventId;`). 3C below imports
// `uuid::Uuid` and `crate::aggregate::PostalReceive`.
// Re-importing any of them here is an E0252 duplicate. The
// `NewPostalDispatch`, `PostalDispatch`,
// `UpdatePostalDispatch`, `AcademicYearId` (from
// `crate::aggregate`), `DeletePostalDispatchCommand`,
// `DispatchPostalCommand`, `UpdatePostalDispatchCommand`
// (from `crate::commands`), `PostalDispatchDeleted`,
// `PostalDispatchUpdated`, `PostalDispatched` (from
// `crate::events`), `PostalDispatchRepository` (from
// `crate::repository`), and `PostalAddress`,
// `PostalDispatchId`, `PostalReceiveId`, `PostalReferenceNo`
// (from `crate::value_objects`) imports below are all new
// to this section. `Uuid` and `PostalReceive` come from
// 3C's import block below.

use crate::aggregate::{AcademicYearId, NewPostalDispatch, PostalDispatch, UpdatePostalDispatch};
use crate::commands::{
    DeletePostalDispatchCommand, DispatchPostalCommand, UpdatePostalDispatchCommand,
};
use crate::events::{PostalDispatchDeleted, PostalDispatchUpdated, PostalDispatched};
use crate::repository::PostalDispatchRepository;
use crate::value_objects::{PostalAddress, PostalDispatchId, PostalReceiveId, PostalReferenceNo};

/// A reference triple for the within-year filter view. A
/// `(dispatch_id?, receive_id?, reference_no)` triple that
/// carries just enough metadata to identify a postal row
/// within a given academic year. Records without a
/// `reference_no` are excluded by the producer.
#[derive(Debug, Clone, PartialEq)]
pub struct PostalReference {
    /// The dispatch id, if the row is a dispatch.
    pub dispatch_id: Option<PostalDispatchId>,
    /// The receive id, if the row is a receive.
    pub receive_id: Option<PostalReceiveId>,
    /// The shared reference number.
    pub reference_no: PostalReferenceNo,
}

/// Pure helpers (no I/O) for the postal aggregates (dispatch
/// and receive — shared between workstreams 3B and 3C). The
/// type itself lives in the 3B section; the [`PostalPair`]
/// it produces reuses the 3C-defined pairing struct via the
/// `services` module scope.
pub struct PostalService;

impl PostalService {
    /// True if the given `reference_no` is not in the list of
    /// existing references. The check is a service-level
    /// guard; the storage adapter also enforces uniqueness at
    /// the DB level (via a composite unique index on
    /// `(school_id, academic_id, reference_no)`).
    #[must_use]
    pub fn reference_unique(
        reference: &PostalReferenceNo,
        existing: &[PostalReferenceNo],
    ) -> bool {
        !existing.iter().any(|r| r == reference)
    }

    /// Pair dispatches with receives by matching
    /// `reference_no`. The algorithm is:
    ///
    /// 1. For each dispatch with a `reference_no`, find the
    ///    first unused receive with the same `reference_no`
    ///    and pair them.
    /// 2. Dispatches that have no match become a
    ///    `(Some(_), None)` pair.
    /// 3. Receives that have no match become a
    ///    `(None, Some(_))` pair.
    /// 4. Dispatches with no `reference_no` are skipped (they
    ///    cannot be paired by definition).
    #[must_use]
    pub fn pair_by_reference(
        dispatches: &[PostalDispatch],
        receives: &[crate::aggregate::PostalReceive],
    ) -> Vec<PostalPair> {
        let mut pairs: Vec<PostalPair> = Vec::new();
        let mut used_receives = vec![false; receives.len()];

        for d in dispatches {
            if let Some(ref_d) = &d.reference_no {
                let mut matched = false;
                for (idx, r) in receives.iter().enumerate() {
                    if !used_receives[idx] && r.reference_no.as_ref() == Some(ref_d) {
                        pairs.push(PostalPair {
                            dispatch: Some(d.clone()),
                            receive: Some(r.clone()),
                        });
                        used_receives[idx] = true;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    pairs.push(PostalPair {
                        dispatch: Some(d.clone()),
                        receive: None,
                    });
                }
            }
        }
        for (idx, r) in receives.iter().enumerate() {
            if !used_receives[idx] {
                pairs.push(PostalPair {
                    dispatch: None,
                    receive: Some(r.clone()),
                });
            }
        }
        pairs
    }

    /// Filter dispatches + receives to those whose
    /// `academic_id` matches the given year AND which carry a
    /// `reference_no`. The output is a flat
    /// `Vec<PostalReference>` (one row per matching source
    /// row; the dispatch/receive id disambiguates which).
    #[must_use]
    pub fn within_year(
        dispatches: &[PostalDispatch],
        receives: &[crate::aggregate::PostalReceive],
        year: AcademicYearId,
    ) -> Vec<PostalReference> {
        let mut out: Vec<PostalReference> = Vec::new();
        for d in dispatches {
            if d.academic_id == year {
                if let Some(ref_d) = &d.reference_no {
                    out.push(PostalReference {
                        dispatch_id: Some(d.id),
                        receive_id: None,
                        reference_no: ref_d.clone(),
                    });
                }
            }
        }
        for r in receives {
            if r.academic_id == year {
                if let Some(ref_r) = &r.reference_no {
                    out.push(PostalReference {
                        dispatch_id: None,
                        receive_id: Some(r.id),
                        reference_no: ref_r.clone(),
                    });
                }
            }
        }
        out
    }

    /// Format a postal address for display. The display
    /// form is the raw address string (no normalisation;
    /// addresses are free-text in the engine).
    #[must_use]
    pub fn format_address(addr: &PostalAddress) -> String {
        addr.as_str().to_owned()
    }
}

/// JSON snapshot of a postal dispatch, used for the audit
/// row's `before` / `after` columns. A `serde_json` failure
/// falls back to an empty payload (audit rows are
/// best-effort; a missing snapshot is preferable to a failed
/// write that would roll back the underlying mutation).
/// Named `snapshot_dispatch` to avoid clashing with the
/// FormDownload (3A) and `snapshot_receive` (3C) helpers.
fn snapshot_dispatch(dispatch: &PostalDispatch) -> Bytes {
    Bytes::from(serde_json::to_vec(dispatch).unwrap_or_default())
}

/// Service factory: record a new [`PostalDispatch`].
/// Capability-gates on
/// [`Capability::PostalDispatchCreate`], constructs the
/// aggregate, persists it via the repository, writes the
/// audit row, and publishes the [`PostalDispatched`] event
/// to the bus.
///
/// The `academic_id` is the active academic-year scope (per
/// the `(school_id, academic_id)` uniqueness for
/// `reference_no`).
#[allow(clippy::too_many_arguments)]
pub async fn dispatch_postal_service<R, B>(
    cmd: DispatchPostalCommand,
    academic_id: AcademicYearId,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<PostalDispatch, DocumentsError>
where
    R: PostalDispatchRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalDispatchCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = PostalDispatchId::new(tenant.school_id, Uuid::now_v7());
    let new: NewPostalDispatch = cmd.into_new_postal_dispatch(id, academic_id);
    let dispatch = PostalDispatch::new(new)?;
    repo.insert(&dispatch).await?;
    let after = snapshot_dispatch(&dispatch);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::PostalDispatch(dispatch.id.as_uuid()),
            None,
            Some(after),
        )
        .await?;
    let event = PostalDispatched::new(
        &dispatch,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(dispatch)
}

/// Service factory: update an existing [`PostalDispatch`].
/// Capability-gates on
/// [`Capability::PostalDispatchUpdate`], re-loads the
/// dispatch, applies the changes, persists the updated row,
/// writes the audit row, and publishes the
/// [`PostalDispatchUpdated`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn update_postal_dispatch_service<R, B>(
    cmd: UpdatePostalDispatchCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<PostalDispatch, DocumentsError>
where
    R: PostalDispatchRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalDispatchUpdate).await?;
    let mut dispatch = repo
        .get(cmd.postal_dispatch_id)
        .await?
        .ok_or(DocumentsError::PostalDispatchNotFound(
            cmd.postal_dispatch_id.as_uuid(),
        ))?;
    let before = snapshot_dispatch(&dispatch);

    let mut changes: Vec<String> = Vec::new();
    if cmd.to_title.is_some() {
        changes.push("to_title".to_owned());
    }
    if cmd.from_title.is_some() {
        changes.push("from_title".to_owned());
    }
    if cmd.address.is_some() {
        changes.push("address".to_owned());
    }
    if cmd.date.is_some() {
        changes.push("date".to_owned());
    }
    if cmd.note.is_some() {
        changes.push("note".to_owned());
    }
    if cmd.file.is_some() {
        changes.push("file".to_owned());
    }

    let tenant = cmd.tenant.clone();
    let event_id = EventId(Uuid::now_v7());
    let update_cmd: UpdatePostalDispatch = cmd.into_update_postal_dispatch(event_id);
    dispatch.update(update_cmd)?;
    repo.update(&dispatch).await?;
    let after = snapshot_dispatch(&dispatch);
    audit
        .write(
            &tenant,
            AuditAction::Update,
            AuditTarget::PostalDispatch(dispatch.id.as_uuid()),
            Some(before),
            Some(after),
        )
        .await?;
    let event = PostalDispatchUpdated::new(
        &dispatch,
        changes,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(dispatch)
}

/// Service factory: soft-delete a [`PostalDispatch`].
/// Capability-gates on
/// [`Capability::PostalDispatchDelete`], re-loads the
/// dispatch, flips `active_status` to `false`, persists the
/// updated row, writes the audit row, and publishes the
/// [`PostalDispatchDeleted`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn delete_postal_dispatch_service<R, B>(
    cmd: DeletePostalDispatchCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<(), DocumentsError>
where
    R: PostalDispatchRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalDispatchDelete).await?;
    let mut dispatch = repo
        .get(cmd.postal_dispatch_id)
        .await?
        .ok_or(DocumentsError::PostalDispatchNotFound(
            cmd.postal_dispatch_id.as_uuid(),
        ))?;
    let before = snapshot_dispatch(&dispatch);
    let tenant = cmd.tenant.clone();
    dispatch.soft_delete(tenant.actor_id, Timestamp::now())?;
    repo.update(&dispatch).await?;
    audit
        .write(
            &tenant,
            AuditAction::Delete,
            AuditTarget::PostalDispatch(dispatch.id.as_uuid()),
            Some(before),
            None,
        )
        .await?;
    let event = PostalDispatchDeleted::new(
        &dispatch,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(())
}

// === PostalDispatch services section end ===

// === PostalReceive services section begin (owner: 3C) ===

// 3A and 3B above import the rest of what this section needs
// (e.g. `Arc`, `Bytes`, `AuditAction`, `AuditTarget`,
// `AuditWriter`, `EventId`, `Timestamp`, `DomainEvent`,
// `EventBus`, `CapabilityCheck`, `Capability`,
// `DocumentsError`, `PostalReceiveId`, `PostalReferenceNo`,
// `AcademicYearId`). Re-importing any of them here is an
// E0252 duplicate. `uuid::Uuid` and the 3C-owned
// aggregate/command/event/repo types and the [`PostalPair`]
// struct are new to this section.

use uuid::Uuid;

use crate::aggregate::{NewPostalReceive, PostalReceive, UpdatePostalReceive};
use crate::commands::{
    DeletePostalReceiveCommand, ReceivePostalCommand, TrackPostalCommand,
    UpdatePostalReceiveCommand,
};
use crate::events::{PostalReceiveDeleted, PostalReceiveUpdated, PostalReceived};
use crate::repository::PostalReceiveRepository;

/// A pair of dispatch + receive records that share a
/// [`PostalReferenceNo`](crate::value_objects::PostalReferenceNo).
/// Returned by [`track_postal_service`]. The fields are
/// `Option` because a reference may be present in only one of
/// the two tables (e.g. an outgoing dispatch with no matching
/// receive, or an incoming receive with no matching dispatch).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PostalPair {
    /// The matching dispatch, if any.
    pub dispatch: Option<crate::aggregate::PostalDispatch>,
    /// The matching receive, if any.
    pub receive: Option<PostalReceive>,
}

/// JSON snapshot of a postal receive, used for the audit row's
/// `before` / `after` columns. A `serde_json` failure falls
/// back to an empty payload (audit rows are best-effort; a
/// missing snapshot is preferable to a failed write that would
/// roll back the underlying mutation). Named
/// `snapshot_receive` to avoid clashing with the
/// `FormDownload` snapshot helper defined in the 3A section.
fn snapshot_receive(receive: &PostalReceive) -> Bytes {
    Bytes::from(serde_json::to_vec(receive).unwrap_or_default())
}

/// Service factory: record a new [`PostalReceive`]. Capability-
/// gates on [`Capability::PostalReceiveCreate`], constructs the
/// aggregate, persists it via the repository, writes the audit
/// row, and publishes the [`PostalReceived`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn receive_postal_service<R, B>(
    cmd: ReceivePostalCommand,
    academic_id: AcademicYearId,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<PostalReceive, DocumentsError>
where
    R: PostalReceiveRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalReceiveCreate).await?;
    let tenant = cmd.tenant.clone();
    let id = PostalReceiveId::new(tenant.school_id, Uuid::now_v7());
    let new: NewPostalReceive = cmd.into_new_postal_receive(id, academic_id);
    let receive = PostalReceive::new(new)?;
    repo.insert(&receive).await?;
    let after = snapshot_receive(&receive);
    audit
        .write(
            &tenant,
            AuditAction::Create,
            AuditTarget::PostalReceive(receive.id.as_uuid()),
            None,
            Some(after),
        )
        .await?;
    let event = PostalReceived::new(
        &receive,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(receive)
}

/// Service factory: update an existing [`PostalReceive`].
/// Capability-gates on [`Capability::PostalReceiveUpdate`],
/// re-loads the receive, applies the changes, persists the
/// updated row, writes the audit row, and publishes the
/// [`PostalReceiveUpdated`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn update_postal_receive_service<R, B>(
    cmd: UpdatePostalReceiveCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<PostalReceive, DocumentsError>
where
    R: PostalReceiveRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalReceiveUpdate).await?;
    let mut receive = repo
        .get(cmd.postal_receive_id)
        .await?
        .ok_or(DocumentsError::PostalReceiveNotFound(
            cmd.postal_receive_id.as_uuid(),
        ))?;
    let before = snapshot_receive(&receive);

    let mut changes: Vec<String> = Vec::new();
    if cmd.from_title.is_some() {
        changes.push("from_title".to_owned());
    }
    if cmd.to_title.is_some() {
        changes.push("to_title".to_owned());
    }
    if cmd.address.is_some() {
        changes.push("address".to_owned());
    }
    if cmd.date.is_some() {
        changes.push("date".to_owned());
    }
    if cmd.note.is_some() {
        changes.push("note".to_owned());
    }
    if cmd.file.is_some() {
        changes.push("file".to_owned());
    }

    let tenant = cmd.tenant.clone();
    let event_id = EventId(Uuid::now_v7());
    let update_cmd: UpdatePostalReceive = cmd.into_update_postal_receive(event_id);
    receive.update(update_cmd)?;
    repo.update(&receive).await?;
    let after = snapshot_receive(&receive);
    audit
        .write(
            &tenant,
            AuditAction::Update,
            AuditTarget::PostalReceive(receive.id.as_uuid()),
            Some(before),
            Some(after),
        )
        .await?;
    let event = PostalReceiveUpdated::new(
        &receive,
        changes,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(receive)
}

/// Service factory: soft-delete a [`PostalReceive`]. Capability-
/// gates on [`Capability::PostalReceiveDelete`], re-loads the
/// receive, flips `active_status` to `false`, persists the
/// updated row, writes the audit row, and publishes the
/// [`PostalReceiveDeleted`] event to the bus.
#[allow(clippy::too_many_arguments)]
pub async fn delete_postal_receive_service<R, B>(
    cmd: DeletePostalReceiveCommand,
    repo: Arc<R>,
    bus: Arc<B>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<(), DocumentsError>
where
    R: PostalReceiveRepository + 'static,
    B: EventBus + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalReceiveDelete).await?;
    let mut receive = repo
        .get(cmd.postal_receive_id)
        .await?
        .ok_or(DocumentsError::PostalReceiveNotFound(
            cmd.postal_receive_id.as_uuid(),
        ))?;
    let before = snapshot_receive(&receive);
    let tenant = cmd.tenant.clone();
    receive.soft_delete(tenant.actor_id, Timestamp::now())?;
    repo.update(&receive).await?;
    audit
        .write(
            &tenant,
            AuditAction::Delete,
            AuditTarget::PostalReceive(receive.id.as_uuid()),
            Some(before),
            None,
        )
        .await?;
    let event = PostalReceiveDeleted::new(
        &receive,
        tenant.actor_id,
        Timestamp::now(),
        tenant.correlation_id,
    );
    bus.publish(event.into_envelope(&tenant)).await?;
    Ok(())
}

/// Service factory: track a postal item by reference number.
/// Returns the matching dispatch + receive records (a query,
/// not a mutation). Emits NO event per spec; one audit row is
/// written for the read.
///
/// The `dispatch_repo` parameter is reserved for the
/// 3B-owned `find_by_reference` method on
/// [`PostalDispatchRepository`]. Once that method lands, the
/// dispatch side of the [`PostalPair`] will be populated.
/// Until then the dispatch side is always `None`.
#[allow(unused_variables, clippy::too_many_arguments)]
pub async fn track_postal_service<DRepo, RRepo>(
    cmd: TrackPostalCommand,
    dispatch_repo: Arc<DRepo>,
    receive_repo: Arc<RRepo>,
    audit: Arc<AuditWriter>,
    cap: &dyn CapabilityCheck,
) -> Result<PostalPair, DocumentsError>
where
    DRepo: PostalDispatchRepository + 'static,
    RRepo: PostalReceiveRepository + 'static,
{
    require_capability(cap, &cmd.tenant, Capability::PostalRead).await?;
    let _ = dispatch_repo;
    let receives = receive_repo
        .find_by_reference(cmd.tenant.school_id, &cmd.reference_no)
        .await?;
    let pair = PostalPair {
        dispatch: None,
        receive: receives.into_iter().next(),
    };
    let tenant = cmd.tenant.clone();
    audit
        .write(
            &tenant,
            AuditAction::Other("read".to_owned()),
            AuditTarget::Other("postal_track".to_owned(), Uuid::now_v7()),
            None,
            None,
        )
        .await?;
    Ok(pair)
}

// === PostalReceive services section end ===

// =============================================================================
// Tests (including the headline 100-case proptest)
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
    use educore_core::clock::IdGenerator as _;

    fn ids() -> (
        educore_core::ids::SchoolId,
        educore_core::ids::UserId,
        educore_core::ids::EventId,
        educore_core::ids::CorrelationId,
        educore_core::value_objects::Timestamp,
    ) {
        let g = educore_core::clock::SystemIdGen;
        let s = g.next_school_id();
        let u = g.next_user_id();
        let e = g.next_event_id();
        let c = g.next_correlation_id();
        let t = educore_core::value_objects::Timestamp::now();
        (s, u, e, c, t)
    }

    fn title() -> crate::value_objects::FormTitle {
        crate::value_objects::FormTitle::new("Form Title").unwrap()
    }

    fn url() -> crate::value_objects::Url {
        crate::value_objects::Url::new("https://example.com/x.pdf").unwrap()
    }

    fn file_ref() -> crate::value_objects::FileReference {
        crate::value_objects::FileReference::new("k1").unwrap()
    }

    fn publish_date() -> crate::value_objects::PublishDate {
        crate::value_objects::PublishDate::new(
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        )
    }

    fn make_form_with(link: Option<crate::value_objects::Url>, file: Option<crate::value_objects::FileReference>) -> crate::aggregate::FormDownload {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::now_v7());
        let cmd = crate::aggregate::NewFormDownload {
            id,
            title: title(),
            short_description: None,
            publish_date: publish_date(),
            link,
            file,
            show_public: crate::value_objects::ShowPublic::default(),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        crate::aggregate::FormDownload::new(cmd).expect("ok")
    }

    // -------------------------------------------------------------------------
    // FormService pure-helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn form_service_validate_content_rejects_both_none() {
        let err = FormService::validate_content(None, None).unwrap_err();
        assert!(matches!(err, DocumentsError::FormHasNoContent));
    }

    #[test]
    fn form_service_validate_content_accepts_link_only() {
        FormService::validate_content(Some(&url()), None).expect("link only ok");
    }

    #[test]
    fn form_service_validate_content_accepts_file_only() {
        FormService::validate_content(None, Some(&file_ref())).expect("file only ok");
    }

    #[test]
    fn form_service_is_public_reflects_aggregate_flag() {
        let (s, u, _e, c, t) = ids();
        let id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::now_v7());
        let cmd = crate::aggregate::NewFormDownload {
            id,
            title: title(),
            short_description: None,
            publish_date: publish_date(),
            link: Some(url()),
            file: None,
            show_public: crate::value_objects::ShowPublic::new(true),
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let form = crate::aggregate::FormDownload::new(cmd).expect("ok");
        assert!(FormService::is_public(&form));
    }

    #[test]
    fn form_service_is_deliverable_reflects_aggregate_flag() {
        let form = make_form_with(Some(url()), None);
        assert!(FormService::is_deliverable(&form));
    }

    #[test]
    fn form_service_matches_publish_date_strict_inequality() {
        let form = make_form_with(Some(url()), None);
        // publish_date is 2026-06-01.
        // Before the publish date: NOT a match.
        let before =
            chrono::NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        assert!(!FormService::matches_publish_date(&form, before));
        // On the publish date: IS a match.
        let on = chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        assert!(FormService::matches_publish_date(&form, on));
        // After the publish date: IS a match.
        let after = chrono::NaiveDate::from_ymd_opt(2026, 6, 2).unwrap();
        assert!(FormService::matches_publish_date(&form, after));
    }

    // -------------------------------------------------------------------------
    // PostalService pure-helper tests
    // -------------------------------------------------------------------------

    #[test]
    fn postal_service_reference_unique_returns_true_when_not_in_existing() {
        let r = crate::value_objects::PostalReferenceNo::new("REF-001").unwrap();
        let existing: Vec<crate::value_objects::PostalReferenceNo> = vec![];
        assert!(PostalService::reference_unique(&r, &existing));
    }

    #[test]
    fn postal_service_reference_unique_returns_false_when_in_existing() {
        let r = crate::value_objects::PostalReferenceNo::new("REF-001").unwrap();
        let existing = vec![r.clone()];
        assert!(!PostalService::reference_unique(&r, &existing));
    }

    #[test]
    fn postal_service_pair_by_reference_matches_dispatch_with_receive() {
        let (s, u, _e, c, t) = ids();
        let dispatch_id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
        let dispatch_cmd = crate::aggregate::NewPostalDispatch {
            id: dispatch_id,
            academic_id: uuid::Uuid::now_v7(),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("X").unwrap(),
            ),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Y").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-SHARED").unwrap(),
            ),
            address: crate::value_objects::ToAddress::new(
                crate::value_objects::PostalAddress::new("1 St").unwrap(),
            ),
            date: crate::value_objects::DispatchDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let dispatch = crate::aggregate::PostalDispatch::new(dispatch_cmd).expect("ok");
        let receive_cmd = crate::aggregate::NewPostalReceive {
            id: crate::value_objects::PostalReceiveId::new(s, uuid::Uuid::now_v7()),
            academic_id: uuid::Uuid::now_v7(),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("X").unwrap(),
            ),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Y").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-SHARED").unwrap(),
            ),
            address: crate::value_objects::FromAddress::new(
                crate::value_objects::PostalAddress::new("1 St").unwrap(),
            ),
            date: crate::value_objects::ReceiveDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
            created_by: u,
            created_at: t,
            correlation_id: c,
        };
        let receive = crate::aggregate::PostalReceive::new(receive_cmd).expect("ok");
        let pairs = PostalService::pair_by_reference(&[dispatch], &[receive]);
        assert_eq!(pairs.len(), 1);
        assert!(pairs[0].dispatch.is_some());
        assert!(pairs[0].receive.is_some());
    }

    #[test]
    fn postal_service_within_year_filters_to_matching_year_and_reference() {
        let (s, u, _e, c, t) = ids();
        let year_in = uuid::Uuid::now_v7();
        let year_out = uuid::Uuid::now_v7();
        let dispatch_in = {
            let id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
            let cmd = crate::aggregate::NewPostalDispatch {
                id,
                academic_id: year_in,
                to_title: crate::value_objects::ToTitle::new(
                    crate::value_objects::PostalTitle::new("X").unwrap(),
                ),
                from_title: crate::value_objects::FromTitle::new(
                    crate::value_objects::PostalTitle::new("Y").unwrap(),
                ),
                reference_no: Some(
                    crate::value_objects::PostalReferenceNo::new("REF-IN").unwrap(),
                ),
                address: crate::value_objects::ToAddress::new(
                    crate::value_objects::PostalAddress::new("1 St").unwrap(),
                ),
                date: crate::value_objects::DispatchDate::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                ),
                note: None,
                file: None,
                created_by: u,
                created_at: t,
                correlation_id: c,
            };
            crate::aggregate::PostalDispatch::new(cmd).expect("ok")
        };
        let dispatch_other_year = {
            let id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
            let cmd = crate::aggregate::NewPostalDispatch {
                id,
                academic_id: year_out,
                to_title: crate::value_objects::ToTitle::new(
                    crate::value_objects::PostalTitle::new("X").unwrap(),
                ),
                from_title: crate::value_objects::FromTitle::new(
                    crate::value_objects::PostalTitle::new("Y").unwrap(),
                ),
                reference_no: Some(
                    crate::value_objects::PostalReferenceNo::new("REF-OTHER").unwrap(),
                ),
                address: crate::value_objects::ToAddress::new(
                    crate::value_objects::PostalAddress::new("1 St").unwrap(),
                ),
                date: crate::value_objects::DispatchDate::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                ),
                note: None,
                file: None,
                created_by: u,
                created_at: t,
                correlation_id: c,
            };
            crate::aggregate::PostalDispatch::new(cmd).expect("ok")
        };
        let refs = PostalService::within_year(&[dispatch_in, dispatch_other_year], &[], year_in);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].reference_no.as_str(), "REF-IN");
    }

    #[test]
    fn postal_service_format_address_round_trips_string() {
        let addr = crate::value_objects::PostalAddress::new("1 Main St").unwrap();
        assert_eq!(PostalService::format_address(&addr), "1 Main St");
    }

    // -------------------------------------------------------------------------
    // From<DomainError> and From<EventError> for DocumentsError
    // -------------------------------------------------------------------------

    #[test]
    fn from_domain_error_forbidden_maps_to_documents_forbidden() {
        let src = educore_core::error::DomainError::Forbidden("nope".to_owned());
        let dst: DocumentsError = src.into();
        assert!(matches!(dst, DocumentsError::Forbidden(_)));
    }

    #[test]
    fn from_domain_error_conflict_maps_to_documents_conflict() {
        let src = educore_core::error::DomainError::Conflict("dup".to_owned());
        let dst: DocumentsError = src.into();
        assert!(matches!(dst, DocumentsError::Conflict(_)));
    }

    #[test]
    fn from_domain_error_validation_maps_to_documents_validation() {
        let src = educore_core::error::DomainError::Validation("bad".to_owned());
        let dst: DocumentsError = src.into();
        assert!(matches!(dst, DocumentsError::Validation(_)));
    }

    #[test]
    fn from_event_error_maps_to_documents_infrastructure() {
        let src = educore_events::errors::EventError::PublishFailed("down".to_owned());
        let dst: DocumentsError = src.into();
        assert!(matches!(dst, DocumentsError::Infrastructure(_)));
    }

    // -------------------------------------------------------------------------
    // Property-based tests (100 cases each)
    //
    // The two headline correctness properties are:
    //   1. `FormService::is_deliverable` is true iff the form has at
    //      least one of `link` or `file` set.
    //   2. `PostalService::reference_unique` is true iff the
    //      reference is not in the existing list.
    // -------------------------------------------------------------------------

    proptest::proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(100))]

        /// Property 1: `FormService::is_deliverable` is true iff
        /// the form has at least one of `link` or `file` set.
        #[test]
        fn prop_form_is_deliverable_iff_link_or_file_set(
            has_link in proptest::bool::ANY,
            has_file in proptest::bool::ANY,
        ) {
            let link = if has_link { Some(url()) } else { None };
            let file = if has_file { Some(file_ref()) } else { None };
            // We can't construct a form with both `None` via
            // `FormDownload::new` (the invariant forbids it), so
            // when both flags are false we exercise the
            // `is_deliverable` accessor on a never-saved state
            // by constructing a form, then clearing both
            // fields and asserting the deliverable status.
            if has_link || has_file {
                let form = make_form_with(link.clone(), file.clone());
                assert_eq!(FormService::is_deliverable(&form), has_link || has_file);
            } else {
                // Construct with a link, then clear it (bypassing
                // the constructor invariant that would reject
                // the all-None case).
                let mut form = make_form_with(Some(url()), None);
                form.link = None;
                form.file = None;
                assert!(!FormService::is_deliverable(&form));
            }
        }

        /// Property 1b: `FormService::matches_publish_date` is true
        /// iff the form's publish_date is on or before the query
        /// date.
        #[test]
        fn prop_form_matches_publish_date_iff_query_ge_publish(
            offset in -30i32..30,
        ) {
            let form = make_form_with(Some(url()), None);
            let publish = form.publish_date.0;
            let query = publish + chrono::Duration::days(i64::from(offset));
            let expected = offset >= 0;
            assert_eq!(FormService::matches_publish_date(&form, query), expected);
        }

        /// Property 2: `PostalService::reference_unique` returns
        /// `true` iff the reference is NOT in the existing list.
        /// We test by generating 3 candidate references, picking
        /// one of them as the "candidate", and verifying the
        /// function against the list (with and without the
        /// candidate).
        #[test]
        fn prop_postal_reference_unique_iff_not_in_existing(
            tag1 in 0u32..10_000,
            tag2 in 0u32..10_000,
        ) {
            let candidate = crate::value_objects::PostalReferenceNo::new(
                format!("REF-{tag1}"),
            )
            .unwrap();
            let other = crate::value_objects::PostalReferenceNo::new(
                format!("REF-{tag2}"),
            )
            .unwrap();
            // Empty list -> unique.
            assert!(PostalService::reference_unique(&candidate, &[]));
            // List containing only the candidate -> not unique.
            assert!(!PostalService::reference_unique(
                &candidate,
                std::slice::from_ref(&candidate),
            ));
            // List containing only the other -> unique.
            assert!(PostalService::reference_unique(&candidate, std::slice::from_ref(&other)));
            // List containing both -> not unique.
            assert!(!PostalService::reference_unique(
                &candidate,
                &[candidate.clone(), other.clone()],
            ));
        }
    }

    // -------------------------------------------------------------------------
    // Phase 11 / 4-tests — service-factory scenario tests.
    //
    // The service factories (`upload_form_service`,
    // `dispatch_postal_service`, `receive_postal_service`,
    // `delete_form_service`, `track_postal_service`) are exercised
    // with in-memory mocks. The mocks live in the test module
    // (not in the integration test crate) so the service layer's
    // own tests cover the headline path without taking a
    // dependency on the integration test crate.
    // -------------------------------------------------------------------------

    use std::sync::Mutex as StdMutex;

    use async_trait::async_trait;
    use educore_audit::prelude::{AuditWriter, RetentionPolicy};
    use educore_event_bus::InProcessEventBus;
    use educore_events::event_bus::EventBus;
    use educore_rbac::services::InMemoryCapabilityCheck;
    use educore_storage::audit::AuditLogEntry;

    /// In-memory `AuditLog` for the service-factory unit tests.
    #[derive(Debug, Default)]
    struct FactoryTestAuditLog {
        entries: StdMutex<Vec<AuditLogEntry>>,
    }

    #[async_trait]
    impl educore_storage::audit::AuditLog for FactoryTestAuditLog {
        async fn append(&self, entry: AuditLogEntry) -> educore_core::error::Result<()> {
            self.entries.lock().unwrap().push(entry);
            Ok(())
        }
        async fn read_for_target(
            &self,
            _school_id: educore_core::ids::SchoolId,
            _target_id: uuid::Uuid,
            _limit: u32,
        ) -> educore_core::error::Result<Vec<AuditLogEntry>> {
            Ok(self.entries.lock().unwrap().clone())
        }
    }

    /// In-memory `FormDownloadRepository` for the service-factory
    /// unit tests.
    #[derive(Debug, Default)]
    struct FactoryTestFormRepo {
        rows: StdMutex<Vec<crate::aggregate::FormDownload>>,
    }

    #[async_trait]
    impl crate::repository::FormDownloadRepository for FactoryTestFormRepo {
        async fn get(
            &self,
            id: crate::value_objects::FormDownloadId,
        ) -> educore_core::error::Result<Option<crate::aggregate::FormDownload>> {
            Ok(self.rows.lock().unwrap().iter().find(|f| f.id == id).cloned())
        }
        async fn list(
            &self,
            _school: educore_core::ids::SchoolId,
            _q: crate::query::FormDownloadQuery,
        ) -> educore_core::error::Result<Vec<crate::aggregate::FormDownload>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn list_public(
            &self,
            _school: educore_core::ids::SchoolId,
        ) -> educore_core::error::Result<Vec<crate::aggregate::FormDownload>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn insert(
            &self,
            form: &crate::aggregate::FormDownload,
        ) -> educore_core::error::Result<()> {
            self.rows.lock().unwrap().push(form.clone());
            Ok(())
        }
        async fn update(
            &self,
            form: &crate::aggregate::FormDownload,
        ) -> educore_core::error::Result<()> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(e) = rows.iter_mut().find(|f| f.id == form.id) {
                *e = form.clone();
                Ok(())
            } else {
                Err(educore_core::error::DomainError::NotFound(format!(
                    "form {} not found",
                    form.id.as_uuid()
                )))
            }
        }
        async fn by_publish_date(
            &self,
            _school: educore_core::ids::SchoolId,
            _from: chrono::NaiveDate,
            _to: chrono::NaiveDate,
        ) -> educore_core::error::Result<Vec<crate::aggregate::FormDownload>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn count(
            &self,
            _school: educore_core::ids::SchoolId,
            _q: crate::query::FormDownloadQuery,
        ) -> educore_core::error::Result<u64> {
            Ok(self.rows.lock().unwrap().len() as u64)
        }
        async fn page(
            &self,
            _school: educore_core::ids::SchoolId,
            _q: crate::query::FormDownloadQuery,
            _offset: u32,
            _limit: u32,
        ) -> educore_core::error::Result<Vec<crate::aggregate::FormDownload>> {
            Ok(self.rows.lock().unwrap().clone())
        }
    }

    /// In-memory `PostalDispatchRepository` for the service-factory
    /// unit tests (enforces the `(school, academic_id,
    /// reference_no)` uniqueness invariant).
    #[derive(Debug, Default)]
    struct FactoryTestDispatchRepo {
        rows: StdMutex<Vec<crate::aggregate::PostalDispatch>>,
    }

    #[async_trait]
    impl crate::repository::PostalDispatchRepository for FactoryTestDispatchRepo {
        async fn get(
            &self,
            id: crate::value_objects::PostalDispatchId,
        ) -> educore_core::error::Result<Option<crate::aggregate::PostalDispatch>> {
            Ok(self.rows.lock().unwrap().iter().find(|d| d.id == id).cloned())
        }
        async fn list(
            &self,
            _school: educore_core::ids::SchoolId,
            _q: crate::query::PostalDispatchQuery,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalDispatch>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn insert(
            &self,
            dispatch: &crate::aggregate::PostalDispatch,
        ) -> educore_core::error::Result<()> {
            let rows = self.rows.lock().unwrap();
            if let Some(r) = &dispatch.reference_no {
                if rows
                    .iter()
                    .any(|d| d.academic_id == dispatch.academic_id && d.reference_no.as_ref() == Some(r))
                {
                    return Err(educore_core::error::DomainError::Conflict(format!(
                        "duplicate reference_no: {}",
                        r.as_str()
                    )));
                }
            }
            drop(rows);
            self.rows.lock().unwrap().push(dispatch.clone());
            Ok(())
        }
        async fn update(
            &self,
            dispatch: &crate::aggregate::PostalDispatch,
        ) -> educore_core::error::Result<()> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(e) = rows.iter_mut().find(|d| d.id == dispatch.id) {
                *e = dispatch.clone();
                Ok(())
            } else {
                Err(educore_core::error::DomainError::NotFound(format!(
                    "dispatch {} not found",
                    dispatch.id.as_uuid()
                )))
            }
        }
        async fn find_by_reference(
            &self,
            _school: educore_core::ids::SchoolId,
            reference: &crate::value_objects::PostalReferenceNo,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalDispatch>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.reference_no.as_ref() == Some(reference))
                .cloned()
                .collect())
        }
        async fn between(
            &self,
            _school: educore_core::ids::SchoolId,
            _from: chrono::NaiveDate,
            _to: chrono::NaiveDate,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalDispatch>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn by_academic_year(
            &self,
            _school: educore_core::ids::SchoolId,
            _year: crate::aggregate::AcademicYearId,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalDispatch>> {
            Ok(self.rows.lock().unwrap().clone())
        }
    }

    /// In-memory `PostalReceiveRepository` for the
    /// service-factory unit tests.
    #[derive(Debug, Default)]
    struct FactoryTestReceiveRepo {
        rows: StdMutex<Vec<crate::aggregate::PostalReceive>>,
    }

    #[async_trait]
    impl crate::repository::PostalReceiveRepository for FactoryTestReceiveRepo {
        async fn get(
            &self,
            id: crate::value_objects::PostalReceiveId,
        ) -> educore_core::error::Result<Option<crate::aggregate::PostalReceive>> {
            Ok(self.rows.lock().unwrap().iter().find(|r| r.id == id).cloned())
        }
        async fn list(
            &self,
            _school: educore_core::ids::SchoolId,
            _q: crate::query::PostalReceiveQuery,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalReceive>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn insert(
            &self,
            receive: &crate::aggregate::PostalReceive,
        ) -> educore_core::error::Result<()> {
            let rows = self.rows.lock().unwrap();
            if let Some(r) = &receive.reference_no {
                if rows
                    .iter()
                    .any(|x| x.academic_id == receive.academic_id && x.reference_no.as_ref() == Some(r))
                {
                    return Err(educore_core::error::DomainError::Conflict(format!(
                        "duplicate reference_no: {}",
                        r.as_str()
                    )));
                }
            }
            drop(rows);
            self.rows.lock().unwrap().push(receive.clone());
            Ok(())
        }
        async fn update(
            &self,
            receive: &crate::aggregate::PostalReceive,
        ) -> educore_core::error::Result<()> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(e) = rows.iter_mut().find(|r| r.id == receive.id) {
                *e = receive.clone();
                Ok(())
            } else {
                Err(educore_core::error::DomainError::NotFound(format!(
                    "receive {} not found",
                    receive.id.as_uuid()
                )))
            }
        }
        async fn find_by_reference(
            &self,
            _school: educore_core::ids::SchoolId,
            reference: &crate::value_objects::PostalReferenceNo,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalReceive>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.reference_no.as_ref() == Some(reference))
                .cloned()
                .collect())
        }
        async fn between(
            &self,
            _school: educore_core::ids::SchoolId,
            _from: chrono::NaiveDate,
            _to: chrono::NaiveDate,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalReceive>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn by_academic_year(
            &self,
            _school: educore_core::ids::SchoolId,
            _year: crate::aggregate::AcademicYearId,
        ) -> educore_core::error::Result<Vec<crate::aggregate::PostalReceive>> {
            Ok(self.rows.lock().unwrap().clone())
        }
    }

    /// Construct an in-memory test environment for the
    /// service-factory tests.
    async fn factory_test_env() -> (
        Arc<InProcessEventBus>,
        Arc<AuditWriter>,
        Arc<InMemoryCapabilityCheck>,
        Arc<FactoryTestFormRepo>,
        Arc<FactoryTestDispatchRepo>,
        Arc<FactoryTestReceiveRepo>,
        educore_core::ids::SchoolId,
        educore_core::ids::UserId,
        educore_core::ids::CorrelationId,
    ) {
        let bus: Arc<InProcessEventBus> = Arc::new(InProcessEventBus::new());
        let bus_dyn: Arc<dyn EventBus> = bus.clone();
        let g = educore_core::clock::SystemIdGen;
        let school = g.next_school_id();
        let actor = g.next_user_id();
        let corr = g.next_correlation_id();
        let clock = Arc::new(educore_core::clock::TestClock::at(
            educore_core::value_objects::Timestamp::now(),
        ));
        let audit_log: Arc<dyn educore_storage::audit::AuditLog> =
            Arc::new(FactoryTestAuditLog::default());
        let audit = Arc::new(AuditWriter::new(
            audit_log,
            bus_dyn,
            clock,
            RetentionPolicy::default(),
        ));
        let cap = Arc::new(InMemoryCapabilityCheck::new());
        let form_repo = Arc::new(FactoryTestFormRepo::default());
        let dispatch_repo = Arc::new(FactoryTestDispatchRepo::default());
        let receive_repo = Arc::new(FactoryTestReceiveRepo::default());
        (bus, audit, cap, form_repo, dispatch_repo, receive_repo, school, actor, corr)
    }

    fn grant(school: educore_core::ids::SchoolId, cap: &InMemoryCapabilityCheck, c: educore_rbac::value_objects::Capability) {
        let role = educore_rbac::ids::RoleId::new(school, uuid::Uuid::now_v7());
        cap.grant(school, role, c);
    }

    fn ctx(school: educore_core::ids::SchoolId, actor: educore_core::ids::UserId, corr: educore_core::ids::CorrelationId) -> educore_core::tenant::TenantContext {
        educore_core::tenant::TenantContext::for_user(
            school,
            actor,
            corr,
            educore_core::tenant::UserType::SchoolAdmin,
        )
    }

    fn upload_cmd_v(
        school: educore_core::ids::SchoolId,
        actor: educore_core::ids::UserId,
        corr: educore_core::ids::CorrelationId,
    ) -> UploadFormCommand {
        UploadFormCommand {
            tenant: ctx(school, actor, corr),
            title: crate::value_objects::FormTitle::new("Factory Test Form").unwrap(),
            short_description: None,
            publish_date: crate::value_objects::PublishDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            link: Some(crate::value_objects::Url::new("https://example.com/factory.pdf").unwrap()),
            file: None,
            show_public: crate::value_objects::ShowPublic::new(true),
        }
    }

    fn dispatch_cmd_v(
        school: educore_core::ids::SchoolId,
        actor: educore_core::ids::UserId,
        corr: educore_core::ids::CorrelationId,
        ref_no: &str,
    ) -> DispatchPostalCommand {
        DispatchPostalCommand {
            tenant: ctx(school, actor, corr),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Mr Factory").unwrap(),
            ),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(crate::value_objects::PostalReferenceNo::new(ref_no).unwrap()),
            address: crate::value_objects::ToAddress::new(
                crate::value_objects::PostalAddress::new("1 Main St").unwrap(),
            ),
            date: crate::value_objects::DispatchDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
        }
    }

    fn receive_cmd_v(
        school: educore_core::ids::SchoolId,
        actor: educore_core::ids::UserId,
        corr: educore_core::ids::CorrelationId,
        ref_no: &str,
    ) -> ReceivePostalCommand {
        ReceivePostalCommand {
            tenant: ctx(school, actor, corr),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme Vendor").unwrap(),
            ),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(crate::value_objects::PostalReferenceNo::new(ref_no).unwrap()),
            address: crate::value_objects::FromAddress::new(
                crate::value_objects::PostalAddress::new("5 Vendor Rd").unwrap(),
            ),
            date: crate::value_objects::ReceiveDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
        }
    }

    #[tokio::test]
    async fn factory_upload_form_service_succeeds_with_capability() {
        let (bus, audit, cap, form_repo, _d, _r, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::FormDownloadUpload);
        let form = upload_form_service(
            upload_cmd_v(s, u, c),
            form_repo.clone(),
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .expect("upload_form_service ok");
        assert!(form.is_active());
        assert!(form.is_public());
    }

    #[tokio::test]
    async fn factory_upload_form_service_returns_forbidden_without_capability() {
        let (bus, audit, cap, form_repo, _d, _r, s, u, c) = factory_test_env().await;
        // No grant -> Forbidden.
        let err = upload_form_service(
            upload_cmd_v(s, u, c),
            form_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DocumentsError::Forbidden(_)));
    }

    #[tokio::test]
    async fn factory_upload_form_service_returns_form_has_no_content_when_both_none() {
        let (bus, audit, cap, form_repo, _d, _r, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::FormDownloadUpload);
        let mut cmd = upload_cmd_v(s, u, c);
        cmd.link = None;
        cmd.file = None;
        let err = upload_form_service(cmd, form_repo, bus, audit, cap.as_ref())
            .await
            .unwrap_err();
        assert!(matches!(err, DocumentsError::FormHasNoContent));
    }

    #[tokio::test]
    async fn factory_dispatch_postal_service_succeeds_with_capability() {
        let (bus, audit, cap, _f, dispatch_repo, _r, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::PostalDispatchCreate);
        let d = dispatch_postal_service(
            dispatch_cmd_v(s, u, c, "REF-FAC-001"),
            uuid::Uuid::now_v7(),
            dispatch_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .expect("dispatch_postal_service ok");
        assert!(d.is_active());
        assert_eq!(d.reference_no.as_ref().unwrap().as_str(), "REF-FAC-001");
    }

    #[tokio::test]
    async fn factory_dispatch_postal_service_returns_forbidden_without_capability() {
        let (bus, audit, cap, _f, dispatch_repo, _r, s, u, c) = factory_test_env().await;
        let err = dispatch_postal_service(
            dispatch_cmd_v(s, u, c, "REF-FAC-002"),
            uuid::Uuid::now_v7(),
            dispatch_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DocumentsError::Forbidden(_)));
    }

    #[tokio::test]
    async fn factory_receive_postal_service_succeeds_with_capability() {
        let (bus, audit, cap, _f, _d, receive_repo, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::PostalReceiveCreate);
        let r = receive_postal_service(
            receive_cmd_v(s, u, c, "REF-FAC-IN-001"),
            uuid::Uuid::now_v7(),
            receive_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .expect("receive_postal_service ok");
        assert!(r.is_active());
        assert_eq!(
            r.reference_no.as_ref().unwrap().as_str(),
            "REF-FAC-IN-001"
        );
    }

    #[tokio::test]
    async fn factory_receive_postal_service_returns_forbidden_without_capability() {
        let (bus, audit, cap, _f, _d, receive_repo, s, u, c) = factory_test_env().await;
        let err = receive_postal_service(
            receive_cmd_v(s, u, c, "REF-FAC-IN-002"),
            uuid::Uuid::now_v7(),
            receive_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DocumentsError::Forbidden(_)));
    }

    #[tokio::test]
    async fn factory_delete_form_service_succeeds_with_capability() {
        let (bus, audit, cap, form_repo, _d, _r, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::FormDownloadUpload);
        grant(s, &cap, Capability::FormDownloadDelete);
        let form = upload_form_service(
            upload_cmd_v(s, u, c),
            form_repo.clone(),
            bus.clone(),
            audit.clone(),
            cap.as_ref(),
        )
        .await
        .expect("upload");
        let form_id = form.id;
        delete_form_service(
            DeleteFormCommand {
                tenant: ctx(s, u, c),
                form_id,
            },
            form_repo.clone(),
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .expect("delete_form_service ok");
        // The form is still queryable but is_active() is false.
        let fetched = form_repo.get(form_id).await.expect("get").expect("still there");
        assert!(!fetched.is_active());
    }

    #[tokio::test]
    async fn factory_delete_form_service_returns_forbidden_without_capability() {
        let (bus, audit, cap, form_repo, _d, _r, s, u, c) = factory_test_env().await;
        grant(s, &cap, Capability::FormDownloadUpload);
        let form = upload_form_service(
            upload_cmd_v(s, u, c),
            form_repo.clone(),
            bus.clone(),
            audit.clone(),
            cap.as_ref(),
        )
        .await
        .expect("upload");
        let form_id = form.id;
        let err = delete_form_service(
            DeleteFormCommand {
                tenant: ctx(s, u, c),
                form_id,
            },
            form_repo,
            bus,
            audit,
            cap.as_ref(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DocumentsError::Forbidden(_)));
    }
}
