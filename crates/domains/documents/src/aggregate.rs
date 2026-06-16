//! Documents-domain aggregate roots.
//!
//! Three aggregate roots per the spec at
//! `docs/specs/documents/aggregates.md`:
//!
//! - `FormDownload` — owner 1A
//! - `PostalDispatch` — owner 1B
//! - `PostalReceive` — owner 1C
//!
//! The placeholder structs declared here use the same names as the
//! real aggregate types so the prelude's `pub use` lines resolve
//! during the scaffold phase. The owner subagents will replace the
//! bodies with the full domain implementation, preserving the
//! public names.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === FormDownload section begin (owner: 1A) ===

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::{CorrelationId, EventId, SchoolId, UserId};
use educore_core::value_objects::{Etag, Timestamp, Version};

use crate::entities::{FormDownloadFileId, FormDownloadLinkId};
use crate::errors::DocumentsError;
use crate::value_objects::{
    ActiveStatus, FileReference, FormDescription, FormDownloadId, FormTitle, PublishDate,
    ShowPublic, Url,
};

// =============================================================================
// FormDownload — root aggregate (owner 1A)
// =============================================================================

/// Aggregate-local input for [`FormDownload::new`]. The
/// wire-level command lives in `commands::UploadFormCommand` and
/// `From`-converts into this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFormDownload {
    /// The typed id.
    pub id: FormDownloadId,
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
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`FormDownload::update`]. The
/// wire-level command lives in `commands::UpdateFormCommand` and
/// `From`-converts into this shape. The `Option<Option<T>>`
/// pattern for `short_description`, `link`, and `file` allows
/// "no change" (outer `None`), "clear" (`Some(None)`), and
/// "set" (`Some(Some(_))`) semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateFormDownload {
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
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// A downloadable form published by the school. Forms may have
/// at most one `file` and at most one `link`; at least one of
/// `file` or `link` MUST be set (per spec invariant 2). The
/// aggregate is anchored to a school and is never hard-deleted
/// (spec invariant 4): the soft-delete path is the only delete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormDownload {
    /// The typed id.
    pub id: FormDownloadId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The form title (1..=191 chars).
    pub title: FormTitle,
    /// The optional short description (1..=200 chars).
    pub short_description: Option<FormDescription>,
    /// The publish date.
    pub publish_date: PublishDate,
    /// The optional external URL.
    pub link: Option<Url>,
    /// The optional file reference.
    pub file: Option<FileReference>,
    /// Whether the form is visible to the public.
    pub show_public: ShowPublic,
    /// The soft-delete flag (`true` = active, `false` =
    /// archived).
    pub active_status: ActiveStatus,
    // ---- Audit footer (8 fields, mirrors the engine standard) ----
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id for the request that created the row.
    pub correlation_id: CorrelationId,
}

impl FormDownload {
    /// Constructs a new `FormDownload` in the active state.
    /// Validates the "at least one of `link` or `file`" invariant;
    /// returns [`DocumentsError::FormHasNoContent`] when neither
    /// is set.
    pub fn new(cmd: NewFormDownload) -> Result<Self, DocumentsError> {
        if cmd.link.is_none() && cmd.file.is_none() {
            return Err(DocumentsError::FormHasNoContent);
        }
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            title: cmd.title,
            short_description: cmd.short_description,
            publish_date: cmd.publish_date,
            link: cmd.link,
            file: cmd.file,
            show_public: cmd.show_public,
            active_status: ActiveStatus::new(true),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies changes to the form. Re-validates the
    /// link-or-file invariant after applying (the form must
    /// still have at least one of `link` or `file`). Rejects
    /// updates on soft-deleted records.
    pub fn update(&mut self, cmd: UpdateFormDownload) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "cannot update a soft-deleted form".to_owned(),
            ));
        }
        if let Some(t) = cmd.title {
            self.title = t;
        }
        if let Some(d) = cmd.short_description {
            self.short_description = d;
        }
        if let Some(d) = cmd.publish_date {
            self.publish_date = d;
        }
        if let Some(l) = cmd.link {
            self.link = l;
        }
        if let Some(f) = cmd.file {
            self.file = f;
        }
        if let Some(sp) = cmd.show_public {
            self.show_public = sp;
        }
        if self.link.is_none() && self.file.is_none() {
            return Err(DocumentsError::FormHasNoContent);
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Soft-deletes the form. Sets `active_status = false` and
    /// bumps the version. Returns
    /// [`DocumentsError::Conflict`] when the form is already
    /// soft-deleted.
    pub fn soft_delete(
        &mut self,
        actor: UserId,
        at: Timestamp,
    ) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "form is already soft-deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::new(false);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the form is active (not soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }

    /// Returns `true` if the form is visible to the public.
    #[must_use]
    pub fn is_public(&self) -> bool {
        self.show_public.is_public()
    }

    /// Returns `true` if the form has at least one of `link`
    /// or `file` set (i.e. it is deliverable).
    #[must_use]
    pub fn is_deliverable(&self) -> bool {
        self.link.is_some() || self.file.is_some()
    }
}

// =============================================================================
// FormDownloadFile — child entity (owner 1A)
// =============================================================================

/// An optional `FileReference` for a [`FormDownload`]. Forms
/// may have at most one file; the 1:1 cardinality is enforced
/// at the aggregate level. The child entity has its own typed
/// id (`FormDownloadFileId`) but is loaded and persisted only
/// through its aggregate root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormDownloadFile {
    /// The typed id.
    pub id: FormDownloadFileId,
    /// The owning form id (FK to the parent aggregate).
    pub form_id: FormDownloadId,
    /// The owning school (immutable, equals `id.school_id()` and
    /// `form_id.school_id()`).
    pub school_id: SchoolId,
    /// The file content handle.
    pub file: FileReference,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
}

impl FormDownloadFile {
    /// Constructs a new `FormDownloadFile` in the initial state.
    /// The id is generated as a UUIDv7 via [`Uuid::now_v7`].
    /// The tenant-invariant (`school_id == form_id.school_id()`)
    /// is checked via `debug_assert_eq!`; passing mismatched
    /// ids is a dispatcher bug, not a user error.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        school_id: SchoolId,
        form_id: FormDownloadId,
        file: FileReference,
        at: Timestamp,
        actor: UserId,
    ) -> Self {
        debug_assert_eq!(school_id, form_id.school_id());
        let id = FormDownloadFileId::new(school_id, Uuid::now_v7());
        Self {
            id,
            form_id,
            school_id,
            file,
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
        }
    }
}

// =============================================================================
// FormDownloadLink — child entity (owner 1A)
// =============================================================================

/// An optional `Url` for an external resource linked from a
/// [`FormDownload`]. Forms may have at most one link; the 1:1
/// cardinality is enforced at the aggregate level. The child
/// entity has its own typed id (`FormDownloadLinkId`) but is
/// loaded and persisted only through its aggregate root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormDownloadLink {
    /// The typed id.
    pub id: FormDownloadLinkId,
    /// The owning form id (FK to the parent aggregate).
    pub form_id: FormDownloadId,
    /// The owning school (immutable, equals `id.school_id()` and
    /// `form_id.school_id()`).
    pub school_id: SchoolId,
    /// The external URL.
    pub url: Url,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
}

impl FormDownloadLink {
    /// Constructs a new `FormDownloadLink` in the initial state.
    /// The id is generated as a UUIDv7 via [`Uuid::now_v7`].
    /// The tenant-invariant (`school_id == form_id.school_id()`)
    /// is checked via `debug_assert_eq!`; passing mismatched
    /// ids is a dispatcher bug, not a user error.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        school_id: SchoolId,
        form_id: FormDownloadId,
        url: Url,
        at: Timestamp,
        actor: UserId,
    ) -> Self {
        debug_assert_eq!(school_id, form_id.school_id());
        let id = FormDownloadLinkId::new(school_id, Uuid::now_v7());
        Self {
            id,
            form_id,
            school_id,
            url,
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
        }
    }
}

// === FormDownload section end ===

// === PostalDispatch section begin (owner: 1B) ===

// The cross-cutting imports (`serde`, `uuid`, the
// `educore_core::ids` and `educore_core::value_objects`
// prelude, plus `DocumentsError`, `ActiveStatus`,
// `FileReference`, `FromTitle`, `PostalNote`,
// `PostalReferenceNo`, `ToTitle`, and the documents' typed
// ids) are already pulled in by the FormDownload section
// above. The `PostalReceive` section below us (1C owner)
// also pulls in `FromTitle`, `PostalNote`,
// `PostalReferenceNo`, and `ToTitle` for its own code;
// re-importing those names at file scope is an `E0252` error.
// This section therefore imports only the **unique-to-1B**
// value-object types and uses fully-qualified paths for the
// shared ones. (`crate::entities::PostalDispatchAttachmentId`
// is also already pulled in by FormDownload.)
use crate::value_objects::{DispatchDate, PostalDispatchId, ToAddress};

// NOTE: `AcademicYearId` is defined in the 1C section below
// (the only other `documents` section that needs it for the
// same reason). We don't redeclare the type alias here to
// avoid an `E0428` duplicate-definition error; we use the
// 1C definition by name. A follow-up PR should add an
// `educore-academic` dependency on `educore-documents` and
// replace both aliases with
// `educore_academic::value_objects::AcademicYearId`.

// =============================================================================
// PostalDispatch — root aggregate (owner 1B)
// =============================================================================

/// Aggregate-local input for [`PostalDispatch::new`]. The
/// wire-level command lives in
/// `commands::DispatchPostalCommand` and `From`-converts into
/// this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewPostalDispatch {
    /// The typed id.
    pub id: PostalDispatchId,
    /// The academic year scope (per `(school_id, academic_id)`
    /// uniqueness for `reference_no`).
    pub academic_id: AcademicYearId,
    /// The recipient's name/title (1..=191 chars).
    pub to_title: crate::value_objects::ToTitle,
    /// The sender's name/title (1..=191 chars).
    pub from_title: crate::value_objects::FromTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<crate::value_objects::PostalReferenceNo>,
    /// The recipient's address (1..=191 chars).
    pub address: ToAddress,
    /// The dispatch date (may be in the past for back-filling).
    pub date: DispatchDate,
    /// The optional note (1..=5000 chars).
    pub note: Option<crate::value_objects::PostalNote>,
    /// The optional file attachment (scanned copy of the
    /// letter or its envelope).
    pub file: Option<FileReference>,
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`PostalDispatch::update`]. The
/// wire-level command lives in
/// `commands::UpdatePostalDispatchCommand` and `From`-converts
/// into this shape. The `reference_no`, `note`, and `file`
/// fields use the `Option<Option<T>>` pattern: outer `None`
/// means "no change", `Some(None)` means "clear the field",
/// and `Some(Some(_))` means "set the field". The
/// `reference_no` field carries an extra invariant enforced
/// inside [`PostalDispatch::update`]: the reference number is
/// **immutable once set**; an attempt to change or clear it
/// returns [`DocumentsError::ReferenceNoImmutable`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePostalDispatch {
    /// The new academic year scope, if changing.
    pub academic_id: Option<AcademicYearId>,
    /// The new recipient name/title, if changing.
    pub to_title: Option<crate::value_objects::ToTitle>,
    /// The new sender name/title, if changing.
    pub from_title: Option<crate::value_objects::FromTitle>,
    /// The new reference number, if changing or clearing.
    /// See type-level docs for the immutability rule.
    pub reference_no: Option<Option<crate::value_objects::PostalReferenceNo>>,
    /// The new recipient address, if changing.
    pub address: Option<ToAddress>,
    /// The new dispatch date, if changing.
    pub date: Option<DispatchDate>,
    /// The new note, if changing or clearing.
    pub note: Option<Option<crate::value_objects::PostalNote>>,
    /// The new file attachment, if changing or clearing.
    pub file: Option<Option<FileReference>>,
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// A postal item dispatched by the school. The dispatch is
/// recorded with a `to_title`, `from_title`, an optional
/// reference number, an address, a date, an optional note, and
/// an optional attachment. The `reference_no` is **unique
/// within `(school_id, academic_id)` when set** (per
/// `docs/specs/documents/aggregates.md` § "PostalDispatch"
/// invariant 2) and is **immutable once set** (per the Postal
/// Dispatch Tracking workflow, step 3). The aggregate is
/// anchored to a school and an academic year and is never
/// hard-deleted (invariant 5): the soft-delete path is the
/// only delete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatch {
    /// The typed id.
    pub id: PostalDispatchId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year scope (per `(school_id, academic_id)`
    /// uniqueness for `reference_no`).
    pub academic_id: AcademicYearId,
    /// The recipient's name/title (1..=191 chars).
    pub to_title: crate::value_objects::ToTitle,
    /// The sender's name/title (1..=191 chars).
    pub from_title: crate::value_objects::FromTitle,
    /// The optional reference number (unique within
    /// `(school_id, academic_id)`; immutable once set).
    pub reference_no: Option<crate::value_objects::PostalReferenceNo>,
    /// The recipient's address (1..=191 chars).
    pub address: ToAddress,
    /// The dispatch date (may be in the past for back-filling).
    pub date: DispatchDate,
    /// The optional note (1..=5000 chars).
    pub note: Option<crate::value_objects::PostalNote>,
    /// The optional file attachment (scanned copy of the
    /// letter or its envelope).
    pub file: Option<FileReference>,
    /// The soft-delete flag (`true` = active, `false` =
    /// archived).
    pub active_status: ActiveStatus,
    // ---- Audit footer (8 fields, mirrors the engine standard) ----
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id for the request that created the row.
    pub correlation_id: CorrelationId,
}

impl PostalDispatch {
    /// Constructs a new `PostalDispatch` in the active state.
    /// `school_id` is **derived from `id.school_id()`** and is
    /// never taken from the caller.
    pub fn new(cmd: NewPostalDispatch) -> Result<Self, DocumentsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            academic_id: cmd.academic_id,
            to_title: cmd.to_title,
            from_title: cmd.from_title,
            reference_no: cmd.reference_no,
            address: cmd.address,
            date: cmd.date,
            note: cmd.note,
            file: cmd.file,
            active_status: ActiveStatus::new(true),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies changes to the dispatch. Rejects updates on
    /// soft-deleted records. **Rejects any attempt to change
    /// the `reference_no`** — the reference number is
    /// immutable once set; setting or clearing it returns
    /// [`DocumentsError::ReferenceNoImmutable`]. The check is
    /// strict (any new value different from the existing one
    /// is rejected) and tolerates idempotent no-op calls
    /// where the caller resends the existing value.
    pub fn update(&mut self, cmd: UpdatePostalDispatch) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "cannot update a soft-deleted postal dispatch".to_owned(),
            ));
        }
        if let Some(rid) = cmd.academic_id {
            self.academic_id = rid;
        }
        if let Some(t) = cmd.to_title {
            self.to_title = t;
        }
        if let Some(t) = cmd.from_title {
            self.from_title = t;
        }
        if let Some(rn) = cmd.reference_no {
            if rn != self.reference_no {
                return Err(DocumentsError::ReferenceNoImmutable);
            }
        }
        if let Some(a) = cmd.address {
            self.address = a;
        }
        if let Some(d) = cmd.date {
            self.date = d;
        }
        if let Some(n) = cmd.note {
            self.note = n;
        }
        if let Some(f) = cmd.file {
            self.file = f;
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Soft-deletes the dispatch. Sets `active_status = false`
    /// and bumps the version. Returns [`DocumentsError::Conflict`]
    /// when the dispatch is already soft-deleted.
    pub fn soft_delete(
        &mut self,
        actor: UserId,
        at: Timestamp,
    ) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "postal dispatch is already soft-deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::new(false);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the dispatch is active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }
}

// =============================================================================
// PostalDispatchAttachment — child entity (owner 1B)
// =============================================================================

/// An optional `FileReference` attached to a
/// [`PostalDispatch`], typically a scanned copy of the letter
/// or its envelope. The child entity has its own typed id
/// (`PostalDispatchAttachmentId`) but is loaded and persisted
/// only through its aggregate root. The 1:1 cardinality is
/// enforced at the aggregate level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalDispatchAttachment {
    /// The typed id.
    pub id: crate::entities::PostalDispatchAttachmentId,
    /// The owning dispatch id (FK to the parent aggregate).
    pub dispatch_id: PostalDispatchId,
    /// The owning school (immutable, equals
    /// `id.school_id()` and `dispatch_id.school_id()`).
    pub school_id: SchoolId,
    /// The scanned file reference.
    pub file: FileReference,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
}

impl PostalDispatchAttachment {
    /// Constructs a new `PostalDispatchAttachment` in the
    /// initial state. The id is generated as a UUIDv7 via
    /// [`Uuid::now_v7`]. The tenant-invariant
    /// (`school_id == dispatch_id.school_id()`) is checked via
    /// `debug_assert_eq!`; passing mismatched ids is a
    /// dispatcher bug, not a user error.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        school_id: SchoolId,
        dispatch_id: PostalDispatchId,
        file: FileReference,
        at: Timestamp,
        actor: UserId,
    ) -> Self {
        debug_assert_eq!(school_id, dispatch_id.school_id());
        let id = crate::entities::PostalDispatchAttachmentId::new(school_id, Uuid::now_v7());
        Self {
            id,
            dispatch_id,
            school_id,
            file,
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
        }
    }
}

// === PostalDispatch section end ===

// === PostalReceive section begin (owner: 1C) ===

use crate::entities::PostalReceiveAttachmentId;
use crate::value_objects::{
    FromAddress, FromTitle, PostalNote, PostalReceiveId, PostalReferenceNo, ReceiveDate, ToTitle,
};

// TODO(phase-11/1C): replace this local alias with
// `educore_academic::value_objects::AcademicYearId` once the
// `educore-documents` crate gains the `educore-academic`
// dependency in its `Cargo.toml`. The local alias keeps this
// section self-contained for the Phase 11 / 1C slice; the
// academic crate already publishes `AcademicYearId` as a typed
// id wrapper of the form `Id<AcademicYear>` (see
// `crates/domains/academic/src/value_objects.rs:113`).
pub type AcademicYearId = Uuid;

// =============================================================================
// PostalReceive — root aggregate (owner 1C)
// =============================================================================

/// Aggregate-local input for [`PostalReceive::new`]. The
/// wire-level command lives in
/// `commands::ReceivePostalCommand` and `From`-converts into
/// this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewPostalReceive {
    /// The typed id.
    pub id: PostalReceiveId,
    /// The academic year scope (per `(school_id, academic_id)`
    /// uniqueness for `reference_no`).
    pub academic_id: AcademicYearId,
    /// The sender's name/title (1..=191 chars).
    pub from_title: FromTitle,
    /// The recipient's name/title (1..=191 chars).
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
    /// The creating user.
    pub created_by: UserId,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The correlation id.
    pub correlation_id: CorrelationId,
}

/// Aggregate-local input for [`PostalReceive::update`]. The
/// wire-level command lives in
/// `commands::UpdatePostalReceiveCommand` and `From`-converts
/// into this shape. The `reference_no`, `note`, and `file`
/// fields use the `Option<Option<T>>` pattern: outer `None`
/// means "no change", `Some(None)` means "clear the field",
/// and `Some(Some(_))` means "set the field". The
/// `reference_no` field carries an extra invariant enforced
/// inside [`PostalReceive::update`]: the reference number is
/// **immutable once set**; an attempt to change or clear it
/// returns [`DocumentsError::ReferenceNoImmutable`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePostalReceive {
    /// The new academic year scope, if changing.
    pub academic_id: Option<AcademicYearId>,
    /// The new sender name/title, if changing.
    pub from_title: Option<FromTitle>,
    /// The new recipient name/title, if changing.
    pub to_title: Option<ToTitle>,
    /// The new reference number, if changing or clearing.
    /// See type-level docs for the immutability rule.
    pub reference_no: Option<Option<PostalReferenceNo>>,
    /// The new sender address, if changing.
    pub address: Option<FromAddress>,
    /// The new receive date, if changing.
    pub date: Option<ReceiveDate>,
    /// The new note, if changing or clearing.
    pub note: Option<Option<PostalNote>>,
    /// The new file attachment, if changing or clearing.
    pub file: Option<Option<FileReference>>,
    /// The acting user.
    pub actor: UserId,
    /// The update timestamp.
    pub at: Timestamp,
    /// The event id for the update.
    pub event_id: EventId,
}

/// A postal item received by the school. The receive is
/// recorded with a `from_title`, `to_title`, an optional
/// reference number, an address, a date, an optional note, and
/// an optional attachment. The `reference_no` is **unique
/// within `(school_id, academic_id)` when set** (per
/// `docs/specs/documents/aggregates.md` § "PostalReceive"
/// invariant 2) and is **immutable once set** (per the Postal
/// Receive Tracking workflow, step 3). The aggregate is
/// anchored to a school and an academic year and is never
/// hard-deleted (invariant 5): the soft-delete path is the
/// only delete.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalReceive {
    /// The typed id.
    pub id: PostalReceiveId,
    /// The owning school (derived from `id.school_id()`).
    pub school_id: SchoolId,
    /// The academic year scope (per `(school_id, academic_id)`
    /// uniqueness for `reference_no`).
    pub academic_id: AcademicYearId,
    /// The sender's name/title (1..=191 chars).
    pub from_title: FromTitle,
    /// The recipient's name/title (1..=191 chars).
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
    /// The soft-delete flag (`true` = active, `false` =
    /// archived).
    pub active_status: ActiveStatus,
    // ---- Audit footer (8 fields, mirrors the engine standard) ----
    /// The optimistic-concurrency version.
    pub version: Version,
    /// The content hash for conflict resolution.
    pub etag: Etag,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
    /// The id of the last event that mutated this aggregate.
    pub last_event_id: Option<EventId>,
    /// The correlation id for the request that created the row.
    pub correlation_id: CorrelationId,
}

impl PostalReceive {
    /// Constructs a new `PostalReceive` in the active state.
    /// `school_id` is **derived from `id.school_id()`** and is
    /// never taken from the caller.
    pub fn new(cmd: NewPostalReceive) -> Result<Self, DocumentsError> {
        Ok(Self {
            school_id: cmd.id.school_id(),
            id: cmd.id,
            academic_id: cmd.academic_id,
            from_title: cmd.from_title,
            to_title: cmd.to_title,
            reference_no: cmd.reference_no,
            address: cmd.address,
            date: cmd.date,
            note: cmd.note,
            file: cmd.file,
            active_status: ActiveStatus::new(true),
            version: Version::initial(),
            etag: Etag::placeholder(),
            created_at: cmd.created_at,
            updated_at: cmd.created_at,
            created_by: cmd.created_by,
            updated_by: cmd.created_by,
            last_event_id: None,
            correlation_id: cmd.correlation_id,
        })
    }

    /// Applies changes to the receive. Rejects updates on
    /// soft-deleted records. **Rejects any attempt to change
    /// the `reference_no`** — the reference number is
    /// immutable once set; setting or clearing it returns
    /// [`DocumentsError::ReferenceNoImmutable`]. The check is
    /// strict (any new value different from the existing one
    /// is rejected) and tolerates idempotent no-op calls
    /// where the caller resends the existing value.
    pub fn update(&mut self, cmd: UpdatePostalReceive) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "cannot update a soft-deleted postal receive".to_owned(),
            ));
        }
        if let Some(rid) = cmd.academic_id {
            self.academic_id = rid;
        }
        if let Some(t) = cmd.from_title {
            self.from_title = t;
        }
        if let Some(t) = cmd.to_title {
            self.to_title = t;
        }
        if let Some(rn) = cmd.reference_no {
            if rn != self.reference_no {
                return Err(DocumentsError::ReferenceNoImmutable);
            }
        }
        if let Some(a) = cmd.address {
            self.address = a;
        }
        if let Some(d) = cmd.date {
            self.date = d;
        }
        if let Some(n) = cmd.note {
            self.note = n;
        }
        if let Some(f) = cmd.file {
            self.file = f;
        }
        self.updated_at = cmd.at;
        self.updated_by = cmd.actor;
        self.version = self.version.next();
        self.last_event_id = Some(cmd.event_id);
        Ok(())
    }

    /// Soft-deletes the receive. Sets `active_status = false`
    /// and bumps the version. Returns [`DocumentsError::Conflict`]
    /// when the receive is already soft-deleted.
    pub fn soft_delete(
        &mut self,
        actor: UserId,
        at: Timestamp,
    ) -> Result<(), DocumentsError> {
        if !self.active_status.is_active() {
            return Err(DocumentsError::Conflict(
                "postal receive is already soft-deleted".to_owned(),
            ));
        }
        self.active_status = ActiveStatus::new(false);
        self.updated_at = at;
        self.updated_by = actor;
        self.version = self.version.next();
        Ok(())
    }

    /// Returns `true` if the receive is active (not
    /// soft-deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active_status.is_active()
    }
}

// =============================================================================
// PostalReceiveAttachment — child entity (owner 1C)
// =============================================================================

/// An optional `FileReference` attached to a
/// [`PostalReceive`], typically a scanned copy of the letter
/// or its envelope. The child entity has its own typed id
/// (`PostalReceiveAttachmentId`) but is loaded and persisted
/// only through its aggregate root. The 1:1 cardinality is
/// enforced at the aggregate level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostalReceiveAttachment {
    /// The typed id.
    pub id: PostalReceiveAttachmentId,
    /// The owning receive id (FK to the parent aggregate).
    pub receive_id: PostalReceiveId,
    /// The owning school (immutable, equals `id.school_id()` and
    /// `receive_id.school_id()`).
    pub school_id: SchoolId,
    /// The scanned file reference.
    pub file: FileReference,
    /// The creation timestamp.
    pub created_at: Timestamp,
    /// The last-update timestamp.
    pub updated_at: Timestamp,
    /// The creating user.
    pub created_by: UserId,
    /// The last-updating user.
    pub updated_by: UserId,
}

impl PostalReceiveAttachment {
    /// Constructs a new `PostalReceiveAttachment` in the
    /// initial state. The id is generated as a UUIDv7 via
    /// [`Uuid::now_v7`]. The tenant-invariant
    /// (`school_id == receive_id.school_id()`) is checked via
    /// `debug_assert_eq!`; passing mismatched ids is a
    /// dispatcher bug, not a user error.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        school_id: SchoolId,
        receive_id: PostalReceiveId,
        file: FileReference,
        at: Timestamp,
        actor: UserId,
    ) -> Self {
        debug_assert_eq!(school_id, receive_id.school_id());
        let id = PostalReceiveAttachmentId::new(school_id, Uuid::now_v7());
        Self {
            id,
            receive_id,
            school_id,
            file,
            created_at: at,
            updated_at: at,
            created_by: actor,
            updated_by: actor,
        }
    }
}

// === PostalReceive section end ===

