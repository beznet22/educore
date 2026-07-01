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
            DomainError::IdempotencyConflict { key, .. }
            | DomainError::IdempotencyPending { key, .. } => DocumentsError::Conflict(key),
            DomainError::Validation(msg) | DomainError::NotFound(msg) => {
                DocumentsError::Validation(msg)
            }
            DomainError::Infrastructure(src) => DocumentsError::Infrastructure(src.to_string()),
            _ => DocumentsError::Validation(err.to_string()),
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
    pub fn reference_unique(reference: &PostalReferenceNo, existing: &[PostalReferenceNo]) -> bool {
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
    let mut dispatch =
        repo.get(cmd.postal_dispatch_id)
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
    let mut dispatch =
        repo.get(cmd.postal_dispatch_id)
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
    let mut receive =
        repo.get(cmd.postal_receive_id)
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
    let mut receive =
        repo.get(cmd.postal_receive_id)
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
            txn,
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
    txn: &dyn educore_storage::transaction::Transaction,
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
            txn,
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
