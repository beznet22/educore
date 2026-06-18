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
    DispatchDate, FromTitle, PostalDispatchId, PostalNote, PostalReferenceNo, ToAddress, ToTitle,
};

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `documents.postal_dispatch.<verb>`).
// =============================================================================

/// Dispatch-postal command type.
const DOCUMENTS_POSTAL_DISPATCH_DISPATCH_COMMAND_TYPE: &str = "documents.postal_dispatch.dispatch";
/// Update-postal-dispatch command type.
const DOCUMENTS_POSTAL_DISPATCH_UPDATE_COMMAND_TYPE: &str = "documents.postal_dispatch.update";
/// Delete-postal-dispatch command type.
const DOCUMENTS_POSTAL_DISPATCH_DELETE_COMMAND_TYPE: &str = "documents.postal_dispatch.delete";

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

// 2A above already imports `serde::{Deserialize, Serialize}`,
// `uuid::Uuid`, `educore_core::ids::EventId`,
// `educore_core::tenant::TenantContext`, and
// `educore_core::value_objects::Timestamp` at the file scope.
// 2B above already imports `FileReference` (via 2A), `FromTitle`,
// `PostalNote`, `PostalReferenceNo`, `ToTitle` (from
// `crate::value_objects`), and `AcademicYearId` (from
// `crate::aggregate`). Re-importing them here is an E0252
// duplicate. We add only the new types.

use crate::aggregate::{NewPostalReceive, UpdatePostalReceive};
use crate::value_objects::{FromAddress, PostalReceiveId, ReceiveDate};

// =============================================================================
// Command type constants (one per command shape; matches the wire form
// `documents.postal_receive.<verb>` and `documents.postal.track`).
// =============================================================================

/// Receive-postal command type.
const DOCUMENTS_POSTAL_RECEIVE_RECEIVE_COMMAND_TYPE: &str = "documents.postal_receive.receive";
/// Update-postal-receive command type.
const DOCUMENTS_POSTAL_RECEIVE_UPDATE_COMMAND_TYPE: &str = "documents.postal_receive.update";
/// Delete-postal-receive command type.
const DOCUMENTS_POSTAL_RECEIVE_DELETE_COMMAND_TYPE: &str = "documents.postal_receive.delete";
/// Track-postal command type (query command; no event emitted).
const DOCUMENTS_POSTAL_TRACK_COMMAND_TYPE: &str = "documents.postal.track";

// =============================================================================
// PostalReceive commands
// =============================================================================

/// Record a new incoming
/// [`PostalReceive`](crate::aggregate::PostalReceive).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceivePostalCommand {
    /// Tenant context (school, actor, correlation).
    pub tenant: TenantContext,
    /// The sender's title (1..=191 chars).
    pub from_title: FromTitle,
    /// The recipient's title (1..=191 chars).
    pub to_title: ToTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<PostalReferenceNo>,
    /// The sender's address (1..=191 chars).
    pub address: FromAddress,
    /// The receive date (may be in the past for back-filling).
    pub date: ReceiveDate,
    /// The optional note (1..=5000 chars).
    pub note: Option<PostalNote>,
    /// The optional file attachment (scanned copy of the
    /// letter or its envelope).
    pub file: Option<FileReference>,
}

impl ReceivePostalCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_RECEIVE_RECEIVE_COMMAND_TYPE;

    /// Converts the wire-level command into the aggregate-local
    /// [`NewPostalReceive`] input expected by
    /// [`PostalReceive::new`](crate::aggregate::PostalReceive::new).
    /// The `id` is supplied by the dispatcher; `academic_id` is
    /// the active academic-year scope. `created_by` is the
    /// tenant's actor; `created_at` is `Timestamp::now()`.
    #[must_use]
    pub fn into_new_postal_receive(
        self,
        id: PostalReceiveId,
        academic_id: AcademicYearId,
    ) -> NewPostalReceive {
        NewPostalReceive {
            id,
            academic_id,
            from_title: self.from_title,
            to_title: self.to_title,
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
/// [`PostalReceive`](crate::aggregate::PostalReceive).
///
/// The `note` and `file` fields use the `Option<Option<T>>`
/// 3-state pattern: outer `None` means "no change",
/// `Some(None)` means "clear the field", and `Some(Some(_))`
/// means "set the field". The other optional fields
/// (`from_title`, `to_title`, `address`, `date`) are 2-state:
/// `None` = no change; `Some(_)` = set.
///
/// The `reference_no` is **immutable** once set; it is
/// intentionally absent from this command. There is no
/// `reference_no` field — adding one would be a wire-level
/// spec deviation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePostalReceiveCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The receive id.
    pub postal_receive_id: PostalReceiveId,
    /// The new sender title, if changing.
    pub from_title: Option<FromTitle>,
    /// The new recipient title, if changing.
    pub to_title: Option<ToTitle>,
    /// The new sender address, if changing.
    pub address: Option<FromAddress>,
    /// The new receive date, if changing.
    pub date: Option<ReceiveDate>,
    /// The new note, if changing or clearing.
    pub note: Option<Option<PostalNote>>,
    /// The new file attachment, if changing or clearing.
    pub file: Option<Option<FileReference>>,
}

impl UpdatePostalReceiveCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_RECEIVE_UPDATE_COMMAND_TYPE;

    /// Converts the wire-level command into the aggregate-local
    /// [`UpdatePostalReceive`] input expected by
    /// [`PostalReceive::update`](crate::aggregate::PostalReceive::update).
    /// The `event_id` is the caller's pre-minted event id (the
    /// dispatcher is responsible for minting it before
    /// dispatch). `actor` is the tenant's actor; `at` is
    /// `Timestamp::now()`. `academic_id` and `reference_no` are
    /// not exposed on the wire (the former is dispatcher-set,
    /// the latter is immutable once set) and are set to `None`
    /// here.
    #[must_use]
    pub fn into_update_postal_receive(self, event_id: EventId) -> UpdatePostalReceive {
        UpdatePostalReceive {
            academic_id: None,
            from_title: self.from_title,
            to_title: self.to_title,
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

/// Soft-delete a [`PostalReceive`](crate::aggregate::PostalReceive).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeletePostalReceiveCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The receive id.
    pub postal_receive_id: PostalReceiveId,
}

impl DeletePostalReceiveCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_RECEIVE_DELETE_COMMAND_TYPE;
}

/// Look up dispatch and receive records sharing a
/// [`PostalReferenceNo`].
///
/// This is a **query command** (per spec § "TrackPostal"): it
/// surfaces a list of dispatch and receive records matching the
/// reference number within the school. It emits **no** domain
/// event. There is no `into_*` method on this command — the
/// dispatcher returns a read-only [`PostalTrackResult`](
/// crate::services::PostalTrackResult) (or equivalent) directly
/// without going through the aggregate write path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackPostalCommand {
    /// Tenant context.
    pub tenant: TenantContext,
    /// The reference number to track (unique within
    /// `(school_id, academic_id)`).
    pub reference_no: PostalReferenceNo,
}

impl TrackPostalCommand {
    /// The wire-form command type.
    pub const COMMAND_TYPE: &'static str = DOCUMENTS_POSTAL_TRACK_COMMAND_TYPE;
}

// === PostalReceive commands section end ===

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
    use educore_core::clock::IdGenerator as _;

    fn ctx() -> (
        educore_core::tenant::TenantContext,
        educore_core::ids::SchoolId,
    ) {
        let g = educore_core::clock::SystemIdGen;
        let s = g.next_school_id();
        let u = g.next_user_id();
        let c = g.next_correlation_id();
        let tenant = educore_core::tenant::TenantContext::for_user(
            s,
            u,
            c,
            educore_core::tenant::UserType::SchoolAdmin,
        );
        (tenant, s)
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
        crate::value_objects::PublishDate::new(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap())
    }

    // ---- Form commands ----

    #[test]
    fn upload_form_command_into_new_form_download_has_id_and_actor() {
        let (tenant, s) = ctx();
        let cmd = UploadFormCommand {
            tenant: tenant.clone(),
            title: title(),
            short_description: None,
            publish_date: publish_date(),
            link: Some(url()),
            file: None,
            show_public: crate::value_objects::ShowPublic::new(true),
        };
        assert_eq!(
            UploadFormCommand::COMMAND_TYPE,
            "documents.form_download.upload"
        );
        let new = cmd.into_new_form_download();
        assert_eq!(new.id.school_id(), s);
        assert_eq!(new.created_by, tenant.actor_id);
        assert_eq!(new.correlation_id, tenant.correlation_id);
        assert_eq!(new.show_public.is_public(), true);
    }

    #[test]
    fn update_form_command_into_update_form_download_carries_fields() {
        let (tenant, s) = ctx();
        let form_id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::now_v7());
        let cmd = UpdateFormCommand {
            tenant: tenant.clone(),
            form_id,
            title: Some(title()),
            short_description: Some(None),
            publish_date: None,
            link: None,
            file: None,
            show_public: None,
        };
        assert_eq!(
            UpdateFormCommand::COMMAND_TYPE,
            "documents.form_download.update"
        );
        let eid = educore_core::clock::SystemIdGen.next_event_id();
        let u = cmd.into_update_form_download(eid);
        assert_eq!(u.actor, tenant.actor_id);
        assert_eq!(u.event_id, eid);
    }

    #[test]
    fn delete_form_command_carries_tenant_and_form_id() {
        let (tenant, s) = ctx();
        let form_id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::now_v7());
        let cmd = DeleteFormCommand {
            tenant: tenant.clone(),
            form_id,
        };
        assert_eq!(
            DeleteFormCommand::COMMAND_TYPE,
            "documents.form_download.delete"
        );
        assert_eq!(cmd.form_id, form_id);
        assert_eq!(cmd.tenant.school_id, s);
    }

    // ---- PostalDispatch commands ----

    #[test]
    fn dispatch_postal_command_into_new_postal_dispatch_carries_fields() {
        let (tenant, s) = ctx();
        let cmd = DispatchPostalCommand {
            tenant: tenant.clone(),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Mr Smith").unwrap(),
            ),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-2026-0001").unwrap(),
            ),
            address: crate::value_objects::ToAddress::new(
                crate::value_objects::PostalAddress::new("1 Main St").unwrap(),
            ),
            date: crate::value_objects::DispatchDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
        };
        assert_eq!(
            DispatchPostalCommand::COMMAND_TYPE,
            "documents.postal_dispatch.dispatch"
        );
        let id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
        let academic_id = uuid::Uuid::now_v7();
        let new = cmd.into_new_postal_dispatch(id, academic_id);
        assert_eq!(new.id, id);
        assert_eq!(new.academic_id, academic_id);
        assert_eq!(new.created_by, tenant.actor_id);
        assert_eq!(new.correlation_id, tenant.correlation_id);
    }

    #[test]
    fn update_postal_dispatch_command_into_update_carries_no_reference_no() {
        let (tenant, s) = ctx();
        let dispatch_id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
        let cmd = UpdatePostalDispatchCommand {
            tenant: tenant.clone(),
            postal_dispatch_id: dispatch_id,
            to_title: Some(crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("X").unwrap(),
            )),
            from_title: None,
            address: None,
            date: None,
            note: None,
            file: None,
        };
        assert_eq!(
            UpdatePostalDispatchCommand::COMMAND_TYPE,
            "documents.postal_dispatch.update"
        );
        let eid = educore_core::clock::SystemIdGen.next_event_id();
        let u = cmd.into_update_postal_dispatch(eid);
        // The aggregate-local `reference_no` MUST be `None` on
        // the update (the field is immutable; a change is
        // rejected at the aggregate level).
        assert!(u.reference_no.is_none());
        assert!(u.academic_id.is_none());
        assert_eq!(u.event_id, eid);
    }

    #[test]
    fn delete_postal_dispatch_command_carries_tenant_and_id() {
        let (tenant, s) = ctx();
        let dispatch_id = crate::value_objects::PostalDispatchId::new(s, uuid::Uuid::now_v7());
        let cmd = DeletePostalDispatchCommand {
            tenant: tenant.clone(),
            postal_dispatch_id: dispatch_id,
        };
        assert_eq!(
            DeletePostalDispatchCommand::COMMAND_TYPE,
            "documents.postal_dispatch.delete"
        );
        assert_eq!(cmd.postal_dispatch_id, dispatch_id);
    }

    // ---- PostalReceive commands ----

    #[test]
    fn receive_postal_command_into_new_postal_receive_carries_fields() {
        let (tenant, s) = ctx();
        let cmd = ReceivePostalCommand {
            tenant: tenant.clone(),
            from_title: crate::value_objects::FromTitle::new(
                crate::value_objects::PostalTitle::new("Acme Vendor").unwrap(),
            ),
            to_title: crate::value_objects::ToTitle::new(
                crate::value_objects::PostalTitle::new("Acme School").unwrap(),
            ),
            reference_no: Some(
                crate::value_objects::PostalReferenceNo::new("REF-IN-0001").unwrap(),
            ),
            address: crate::value_objects::FromAddress::new(
                crate::value_objects::PostalAddress::new("5 Vendor Rd").unwrap(),
            ),
            date: crate::value_objects::ReceiveDate::new(
                chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            ),
            note: None,
            file: None,
        };
        assert_eq!(
            ReceivePostalCommand::COMMAND_TYPE,
            "documents.postal_receive.receive"
        );
        let id = crate::value_objects::PostalReceiveId::new(s, uuid::Uuid::now_v7());
        let academic_id = uuid::Uuid::now_v7();
        let new = cmd.into_new_postal_receive(id, academic_id);
        assert_eq!(new.id, id);
        assert_eq!(new.academic_id, academic_id);
        assert_eq!(new.created_by, tenant.actor_id);
    }

    #[test]
    fn update_postal_receive_command_into_update_carries_no_reference_no() {
        let (tenant, s) = ctx();
        let receive_id = crate::value_objects::PostalReceiveId::new(s, uuid::Uuid::now_v7());
        let cmd = UpdatePostalReceiveCommand {
            tenant: tenant.clone(),
            postal_receive_id: receive_id,
            from_title: None,
            to_title: None,
            address: None,
            date: None,
            note: None,
            file: None,
        };
        assert_eq!(
            UpdatePostalReceiveCommand::COMMAND_TYPE,
            "documents.postal_receive.update"
        );
        let eid = educore_core::clock::SystemIdGen.next_event_id();
        let u = cmd.into_update_postal_receive(eid);
        assert!(u.reference_no.is_none());
        assert!(u.academic_id.is_none());
        assert_eq!(u.event_id, eid);
    }

    #[test]
    fn delete_postal_receive_command_carries_tenant_and_id() {
        let (tenant, s) = ctx();
        let receive_id = crate::value_objects::PostalReceiveId::new(s, uuid::Uuid::now_v7());
        let cmd = DeletePostalReceiveCommand {
            tenant: tenant.clone(),
            postal_receive_id: receive_id,
        };
        assert_eq!(
            DeletePostalReceiveCommand::COMMAND_TYPE,
            "documents.postal_receive.delete"
        );
        assert_eq!(cmd.postal_receive_id, receive_id);
    }

    #[test]
    fn track_postal_command_carries_reference_no() {
        let (tenant, s) = ctx();
        let cmd = TrackPostalCommand {
            tenant: tenant.clone(),
            reference_no: crate::value_objects::PostalReferenceNo::new("REF-2026-0001").unwrap(),
        };
        assert_eq!(TrackPostalCommand::COMMAND_TYPE, "documents.postal.track");
        assert_eq!(cmd.reference_no.as_str(), "REF-2026-0001");
        assert_eq!(cmd.tenant.school_id, s);
    }
}
