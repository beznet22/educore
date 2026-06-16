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
pub struct DispatchPostalCommand;
pub struct UpdatePostalDispatchCommand;
pub struct DeletePostalDispatchCommand;
// === PostalDispatch commands section end ===

// === PostalReceive commands section begin (owner: 2C) ===
pub struct ReceivePostalCommand;
pub struct UpdatePostalReceiveCommand;
pub struct DeletePostalReceiveCommand;
pub struct TrackPostalCommand;
// === PostalReceive commands section end ===
