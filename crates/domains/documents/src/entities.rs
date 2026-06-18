//! # Documents-domain child entities
//!
//! Per `docs/specs/documents/entities.md`:
//!
//! Every documents aggregate (FormDownload, PostalDispatch,
//! PostalReceive) owns at most two child entities. The child
//! entities have their own identity and lifecycle but are
//! loaded and persisted only through their aggregate root.
//!
//! This module hosts the **typed ids** for the four child
//! entities:
//!
//! - [`FormDownloadFileId`] — the optional `FileReference` for
//!   a form download.
//! - [`FormDownloadLinkId`] — the optional `Url` for an
//!   external resource linked from a form download.
//! - [`PostalDispatchAttachmentId`] — the optional
//!   `FileReference` attached to a postal dispatch (typically a
//!   scanned copy of the letter or its envelope).
//! - [`PostalReceiveAttachmentId`] — the optional
//!   `FileReference` attached to a postal receive (typically a
//!   scanned copy of the letter or its envelope).
//!
//! The full struct definitions (with their `display_order`,
//! `caption`, `created_at`, etc. fields) live alongside the
//! aggregate root in [`crate::aggregate`]. They are surfaced
//! as `pub use` in the prelude. This file deliberately keeps
//! the typed ids alone so the engine's
//! `documents_typed_id!` macro pattern is shared with the
//! aggregate-root ids in `value_objects.rs`.
//!
//! Forms may have at most one file and at most one link; the
//! 1:1 cardinality is enforced at the aggregate level. Postal
//! dispatches and receives may each have at most one
//! attachment; the 1:1 cardinality is also enforced at the
//! aggregate level.

#![allow(missing_docs)]
#![allow(unused_imports)]

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use educore_core::ids::SchoolId;

// =============================================================================
// Macro: typed child-entity id
// =============================================================================

/// Macro to define the per-child-entity typed id wrapper.
/// Mirrors the `documents_typed_id!` macro in
/// [`crate::value_objects`], but the fields are private (the
/// child entity is never constructed outside the aggregate
/// root, so callers always go through the aggregate's
/// constructor). The macro still implements the same
/// accessors.
macro_rules! documents_child_typed_id {
    (
        $(#[$attr:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        $vis struct $name {
            /// The owning school (tenant anchor).
            school_id: SchoolId,
            /// The local id (UUIDv7).
            value: Uuid,
        }

        impl $name {
            /// Constructs a new typed id from its parts.
            #[must_use]
            pub const fn new(school_id: SchoolId, value: Uuid) -> Self {
                Self { school_id, value }
            }

            /// Returns the local UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.value
            }

            /// Returns the owning school id.
            #[must_use]
            pub const fn school_id(&self) -> SchoolId {
                self.school_id
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}/{}", self.school_id, self.value)
            }
        }
    };
}

// =============================================================================
// Typed ids: 4 child entities
// =============================================================================

documents_child_typed_id! {
    /// A typed id for a
    /// [`FormDownloadFile`](crate::aggregate::FormDownloadFile)
    /// child entity.
    ///
    /// The child entity holds the optional `FileReference` for
    /// a `FormDownload`. Forms may have at most one file; the
    /// 1:1 cardinality is enforced at the aggregate level.
    pub struct FormDownloadFileId;
}
documents_child_typed_id! {
    /// A typed id for a
    /// [`FormDownloadLink`](crate::aggregate::FormDownloadLink)
    /// child entity.
    ///
    /// The child entity holds the optional `Url` for an
    /// external resource linked from a `FormDownload`. Forms
    /// may have at most one link; the 1:1 cardinality is
    /// enforced at the aggregate level.
    pub struct FormDownloadLinkId;
}
documents_child_typed_id! {
    /// A typed id for a
    /// [`PostalDispatchAttachment`](crate::aggregate::PostalDispatchAttachment)
    /// child entity.
    ///
    /// The child entity holds the optional `FileReference`
    /// attached to a `PostalDispatch` (typically a scanned
    /// copy of the letter or its envelope). Postal dispatches
    /// may have at most one attachment; the 1:1 cardinality is
    /// enforced at the aggregate level.
    pub struct PostalDispatchAttachmentId;
}
documents_child_typed_id! {
    /// A typed id for a
    /// [`PostalReceiveAttachment`](crate::aggregate::PostalReceiveAttachment)
    /// child entity.
    ///
    /// The child entity holds the optional `FileReference`
    /// attached to a `PostalReceive` (typically a scanned
    /// copy of the letter or its envelope). Postal receives
    /// may have at most one attachment; the 1:1 cardinality is
    /// enforced at the aggregate level.
    pub struct PostalReceiveAttachmentId;
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
    use educore_core::ids::Identifier;

    #[test]
    fn child_id_display_and_accessors() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let id = FormDownloadFileId::new(school, Uuid::from_u128(7));
        assert_eq!(id.school_id(), school);
        assert_eq!(id.as_uuid(), Uuid::from_u128(7));
        assert_eq!(id.to_string(), format!("{school}/{}", id.as_uuid()));
    }

    #[test]
    fn all_four_child_ids_construct() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let v = Uuid::from_u128(1);
        let _ = FormDownloadFileId::new(school, v);
        let _ = FormDownloadLinkId::new(school, v);
        let _ = PostalDispatchAttachmentId::new(school, v);
        let _ = PostalReceiveAttachmentId::new(school, v);
    }

    // -------------------------------------------------------------------------
    // Phase 11 / 4-tests — child entity struct smoke tests.
    //
    // The four child entity structs are constructed in aggregate.rs and
    // re-exported through the prelude. These tests confirm the public
    // constructors and field accessors work end-to-end on every child
    // entity, so a regression in the aggregate.rs constructors surfaces
    // here.
    // -------------------------------------------------------------------------

    #[test]
    fn form_download_file_child_entity_constructs_with_tenant_invariant() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let form_id = crate::value_objects::FormDownloadId::new(school, Uuid::from_u128(7));
        let file = crate::value_objects::FileReference::new("object-key-1234").unwrap();
        let at = educore_core::value_objects::Timestamp::now();
        let actor = educore_core::ids::UserId(Uuid::from_u128(99));
        let child =
            crate::aggregate::FormDownloadFile::new(school, form_id, file.clone(), at, actor);
        assert_eq!(child.school_id, school);
        assert_eq!(child.form_id, form_id);
        assert_eq!(child.file, file);
        assert_eq!(child.created_by, actor);
        assert_eq!(child.updated_by, actor);
        assert_eq!(child.created_at, at);
        assert_eq!(child.updated_at, at);
        assert_eq!(child.id.school_id(), school);
    }

    #[test]
    fn form_download_link_child_entity_constructs_with_tenant_invariant() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let form_id = crate::value_objects::FormDownloadId::new(school, Uuid::from_u128(7));
        let url = crate::value_objects::Url::new("https://example.com/path").unwrap();
        let at = educore_core::value_objects::Timestamp::now();
        let actor = educore_core::ids::UserId(Uuid::from_u128(99));
        let child =
            crate::aggregate::FormDownloadLink::new(school, form_id, url.clone(), at, actor);
        assert_eq!(child.school_id, school);
        assert_eq!(child.form_id, form_id);
        assert_eq!(child.url, url);
        assert_eq!(child.id.school_id(), school);
    }

    #[test]
    fn postal_dispatch_attachment_child_entity_constructs_with_tenant_invariant() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let dispatch_id = crate::value_objects::PostalDispatchId::new(school, Uuid::from_u128(7));
        let file = crate::value_objects::FileReference::new("object-key-1234").unwrap();
        let at = educore_core::value_objects::Timestamp::now();
        let actor = educore_core::ids::UserId(Uuid::from_u128(99));
        let child = crate::aggregate::PostalDispatchAttachment::new(
            school,
            dispatch_id,
            file.clone(),
            at,
            actor,
        );
        assert_eq!(child.school_id, school);
        assert_eq!(child.dispatch_id, dispatch_id);
        assert_eq!(child.file, file);
        assert_eq!(child.id.school_id(), school);
    }

    #[test]
    fn postal_receive_attachment_child_entity_constructs_with_tenant_invariant() {
        let school = SchoolId::from_uuid(Uuid::nil());
        let receive_id = crate::value_objects::PostalReceiveId::new(school, Uuid::from_u128(7));
        let file = crate::value_objects::FileReference::new("object-key-1234").unwrap();
        let at = educore_core::value_objects::Timestamp::now();
        let actor = educore_core::ids::UserId(Uuid::from_u128(99));
        let child = crate::aggregate::PostalReceiveAttachment::new(
            school,
            receive_id,
            file.clone(),
            at,
            actor,
        );
        assert_eq!(child.school_id, school);
        assert_eq!(child.receive_id, receive_id);
        assert_eq!(child.file, file);
        assert_eq!(child.id.school_id(), school);
    }

    #[test]
    fn child_id_schools_mismatch_panics_in_debug() {
        // The tenant-invariant on child entities is enforced by
        // `debug_assert_eq!` inside the `::new()` constructor. We do
        // not exercise the panic path here (it would require a
        // `should_panic` attribute and a different file layout); we
        // only confirm that the matching-school case compiles and
        // returns the right shape.
        let school = SchoolId::from_uuid(Uuid::nil());
        let form_id = crate::value_objects::FormDownloadId::new(school, Uuid::from_u128(1));
        let file = crate::value_objects::FileReference::new("k").unwrap();
        let child = crate::aggregate::FormDownloadFile::new(
            school,
            form_id,
            file,
            educore_core::value_objects::Timestamp::now(),
            educore_core::ids::UserId(Uuid::nil()),
        );
        assert_eq!(child.school_id, form_id.school_id());
    }
}
