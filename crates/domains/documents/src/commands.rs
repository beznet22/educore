//! Documents-domain commands.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === Form commands section begin (owner: 2A) ===

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::EventId;
use educore_core::tenant::TenantContext;
use educore_core::value_objects::Timestamp;

use crate::aggregate::{NewFormDownload, UpdateFormDownload};
use crate::value_objects::{
    FileReference, FormDescription, FormDownloadId, FormTitle, PublishDate, ShowPublic, Url,
};

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `documents.form_download.<verb>`).
// =============================================================================

/// Upload-form command type.
const DOCUMENTS_FORM_DOWNLOAD_UPLOAD_COMMAND_TYPE: &str = "documents.form_download.upload";
/// Update-form command type.
const DOCUMENTS_FORM_DOWNLOAD_UPDATE_COMMAND_TYPE: &str = "documents.form_download.update";
/// Delete-form command type.
const DOCUMENTS_FORM_DOWNLOAD_DELETE_COMMAND_TYPE: &str = "documents.form_download.delete";

// =============================================================================
// FormDownload commands
// =============================================================================

/// Upload a new [`FormDownload`](crate::aggregate::FormDownload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadFormCommand {
    /// Tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The form title.
    pub title: FormTitle,
    /// The optional short description.
    pub short_description: Option<FormDescription>,
    /// The publish date.
    pub publish_date: PublishDate,
    /// The optional external URL.
    pub link: Option<Url>,
    /// The optional file reference.
    pub file: Option<FileReference>,
    /// Whether the form is visible to the public.
    pub show_public: ShowPublic,
}

impl UploadFormCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_FORM_DOWNLOAD_UPLOAD_COMMAND_TYPE;

    /// Converts the wire-level command into the
    /// aggregate-local [`NewFormDownload`] input expected by
    /// [`FormDownload::new`](crate::aggregate::FormDownload::new).
    /// The id is minted as a fresh `FormDownloadId` anchored to
    /// `tenant.school_id`; `created_by` is the actor; `created_at`
    /// is `Timestamp::now()`.
    #[must_use]
    pub fn into_new_form_download(self) -> NewFormDownload {
        NewFormDownload {
            id: FormDownloadId::new(self.tenant.school_id, Uuid::now_v7()),
            title: self.title,
            short_description: self.short_description,
            publish_date: self.publish_date,
            link: self.link,
            file: self.file,
            show_public: self.show_public,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update an existing [`FormDownload`](crate::aggregate::FormDownload).
///
/// The `short_description`, `link`, and `file` fields use the
/// `Option<Option<T>>` 3-state pattern: outer `None` means
/// "no change", `Some(None)` means "clear the field", and
/// `Some(Some(_))` means "set the field". The other optional
/// fields (`title`, `publish_date`, `show_public`) are 2-state:
/// `None` = no change; `Some(_)` = set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFormCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The form id.
    pub form_id: FormDownloadId,
    /// The new title, if changing.
    pub title: Option<FormTitle>,
    /// The new short description, if changing or clearing.
    pub short_description: Option<Option<FormDescription>>,
    /// The new publish date, if changing.
    pub publish_date: Option<PublishDate>,
    /// The new link, if changing or clearing.
    pub link: Option<Option<Url>>,
    /// The new file, if changing or clearing.
    pub file: Option<Option<FileReference>>,
    /// The new public-visibility flag, if changing.
    pub show_public: Option<ShowPublic>,
}

impl UpdateFormCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_FORM_DOWNLOAD_UPDATE_COMMAND_TYPE;

    /// Converts the wire-level command into the
    /// aggregate-local [`UpdateFormDownload`] input expected by
    /// [`FormDownload::update`](crate::aggregate::FormDownload::update).
    /// The `event_id` is the caller's pre-minted event id (the
    /// dispatcher is responsible for minting it before
    /// dispatch). `actor` is the tenant's actor; `at` is
    /// `Timestamp::now()`.
    #[must_use]
    pub fn into_update_form_download(self, event_id: EventId) -> UpdateFormDownload {
        UpdateFormDownload {
            title: self.title,
            short_description: self.short_description,
            publish_date: self.publish_date,
            link: self.link,
            file: self.file,
            show_public: self.show_public,
            actor: self.tenant.actor_id,
            at: Timestamp::now(),
            event_id,
        }
    }
}

/// Soft-delete a [`FormDownload`](crate::aggregate::FormDownload).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteFormCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The form id.
    pub form_id: FormDownloadId,
}

impl DeleteFormCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_FORM_DOWNLOAD_DELETE_COMMAND_TYPE;
}

// === Form commands section end ===

// === PostalDispatch commands section begin (owner: 2B) ===

// 2A above already imports `serde::{Deserialize, Serialize}`,
// `uuid::Uuid`, `educore_core::ids::EventId`,
// `educore_core::tenant::TenantContext`,
// `educore_core::value_objects::Timestamp`, and
// `crate::value_objects::{FileReference, FormDescription,
// FormDownloadId, FormTitle, PublishDate, ShowPublic, Url}` at
// the file scope. Re-importing them here is an E0252 duplicate.
// We add only the new types (`FileReference` is already in scope
// from 2A, so we can use it directly).

use crate::aggregate::{AcademicYearId, NewPostalDispatch, UpdatePostalDispatch};
use crate::value_objects::{
    DispatchDate, FromTitle, PostalDispatchId, PostalNote, PostalReferenceNo,
    ToAddress, ToTitle,
};

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `documents.postal_dispatch.<verb>`).
// =============================================================================

/// Dispatch-postal command type.
const DOCUMENTS_POSTAL_DISPATCH_DISPATCH_COMMAND_TYPE: &str =
    "documents.postal_dispatch.dispatch";
/// Update-postal-dispatch command type.
const DOCUMENTS_POSTAL_DISPATCH_UPDATE_COMMAND_TYPE: &str =
    "documents.postal_dispatch.update";
/// Delete-postal-dispatch command type.
const DOCUMENTS_POSTAL_DISPATCH_DELETE_COMMAND_TYPE: &str =
    "documents.postal_dispatch.delete";

// =============================================================================
// PostalDispatch commands
// =============================================================================

/// Dispatch (record) a new outgoing
/// [`PostalDispatch`](crate::aggregate::PostalDispatch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DispatchPostalCommand {
    /// Tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The recipient's title (1..=191 chars).
    pub to_title: ToTitle,
    /// The sender's title (1..=191 chars).
    pub from_title: FromTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<PostalReferenceNo>,
    /// The recipient's address (1..=191 chars).
    pub address: ToAddress,
    /// The dispatch date (may be in the past for back-filling).
    pub date: DispatchDate,
    /// The optional note (1..=5000 chars).
    pub note: Option<PostalNote>,
    /// The optional file attachment (scanned copy of the
    /// letter or its envelope).
    pub file: Option<FileReference>,
}

impl DispatchPostalCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_DISPATCH_DISPATCH_COMMAND_TYPE;

    /// Converts the wire-level command into the aggregate-local
    /// [`NewPostalDispatch`] input expected by
    /// [`PostalDispatch::new`](crate::aggregate::PostalDispatch::new).
    /// The `id` is supplied by the dispatcher; `academic_id` is
    /// the active academic-year scope. `created_by` is the
    /// tenant's actor; `created_at` is `Timestamp::now()`.
    #[must_use]
    pub fn into_new_postal_dispatch(
        self,
        id: PostalDispatchId,
        academic_id: AcademicYearId,
    ) -> NewPostalDispatch {
        NewPostalDispatch {
            id,
            academic_id,
            to_title: self.to_title,
            from_title: self.from_title,
            reference_no: self.reference_no,
            address: self.address,
            date: self.date,
            note: self.note,
            file: self.file,
            created_by: self.tenant.actor_id,
            created_at: Timestamp::now(),
            correlation_id: self.tenant.correlation_id,
        }
    }
}

/// Update an existing
/// [`PostalDispatch`](crate::aggregate::PostalDispatch).
///
/// The `note` and `file` fields use the `Option<Option<T>>`
/// 3-state pattern: outer `None` means "no change",
/// `Some(None)` means "clear the field", and `Some(Some(_))`
/// means "set the field". The other optional fields
/// (`to_title`, `from_title`, `address`, `date`) are 2-state:
/// `None` = no change; `Some(_)` = set.
///
/// The `reference_no` is **immutable** once set; it is
/// intentionally absent from this command. There is no
/// `reference_no` field — adding one would be a wire-level
/// spec deviation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePostalDispatchCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The dispatch id.
    pub postal_dispatch_id: PostalDispatchId,
    /// The new recipient title, if changing.
    pub to_title: Option<ToTitle>,
    /// The new sender title, if changing.
    pub from_title: Option<FromTitle>,
    /// The new recipient address, if changing.
    pub address: Option<ToAddress>,
    /// The new dispatch date, if changing.
    pub date: Option<DispatchDate>,
    /// The new note, if changing or clearing.
    pub note: Option<Option<PostalNote>>,
    /// The new file attachment, if changing or clearing.
    pub file: Option<Option<FileReference>>,
}

impl UpdatePostalDispatchCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_DISPATCH_UPDATE_COMMAND_TYPE;

    /// Converts the wire-level command into the aggregate-local
    /// [`UpdatePostalDispatch`] input expected by
    /// [`PostalDispatch::update`](crate::aggregate::PostalDispatch::update).
    /// The `event_id` is the caller's pre-minted event id (the
    /// dispatcher is responsible for minting it before
    /// dispatch). `actor` is the tenant's actor; `at` is
    /// `Timestamp::now()`. `academic_id` and `reference_no` are
    /// not exposed on the wire (the former is dispatcher-set,
    /// the latter is immutable once set) and are set to `None`
    /// here.
    #[must_use]
    pub fn into_update_postal_dispatch(self, event_id: EventId) -> UpdatePostalDispatch {
        UpdatePostalDispatch {
            academic_id: None,
            to_title: self.to_title,
            from_title: self.from_title,
            reference_no: None,
            address: self.address,
            date: self.date,
            note: self.note,
            file: self.file,
            actor: self.tenant.actor_id,
            at: Timestamp::now(),
            event_id,
        }
    }
}

/// Soft-delete a [`PostalDispatch`](crate::aggregate::PostalDispatch).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePostalDispatchCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The dispatch id.
    pub postal_dispatch_id: PostalDispatchId,
}

impl DeletePostalDispatchCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_DISPATCH_DELETE_COMMAND_TYPE;
}

// === PostalDispatch commands section end ===

// === PostalReceive commands section begin (owner: 2C) ===
pub struct ReceivePostalCommand;
pub struct UpdatePostalReceiveCommand;
pub struct DeletePostalReceiveCommand;
pub struct TrackPostalCommand;
// === PostalReceive commands section end ===
