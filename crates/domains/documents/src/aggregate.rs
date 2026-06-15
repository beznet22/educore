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
pub struct PostalDispatch;
pub struct PostalDispatchAttachment;
// === PostalDispatch section end ===

// === PostalReceive section begin (owner: 1C) ===
pub struct PostalReceive;
pub struct PostalReceiveAttachment;
// === PostalReceive section end ===
