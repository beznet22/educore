//! # educore-documents
//!
//! Form downloads, postal dispatch, postal receive.
//!
//! See `docs/specs/documents/` for the canonical spec.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod aggregate;
pub mod commands;
pub mod entities;
pub mod errors;
pub mod events;
pub mod query;
pub mod repository;
pub mod services;
pub mod value_objects;

pub use crate::aggregate::{FormDownload, PostalDispatch, PostalReceive};

/// Package name constant. Re-exported so consumers can assert they
/// are using the right crate version at compile time.
pub const PACKAGE_NAME: &str = "educore-documents";

/// Package version at compile time.
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Convenient prelude: the public surface of the documents crate.
pub mod prelude {
    pub use crate::aggregate::{
        FormDownload, FormDownloadFile, FormDownloadLink, PostalDispatch, PostalDispatchAttachment,
        PostalReceive, PostalReceiveAttachment,
    };
    pub use crate::commands::{
        DeleteFormCommand, DeletePostalDispatchCommand, DeletePostalReceiveCommand,
        DispatchPostalCommand, ReceivePostalCommand, TrackPostalCommand, UpdateFormCommand,
        UpdatePostalDispatchCommand, UpdatePostalReceiveCommand, UploadFormCommand,
    };
    pub use crate::entities::{
        FormDownloadFileId, FormDownloadLinkId, PostalDispatchAttachmentId,
        PostalReceiveAttachmentId,
    };
    pub use crate::errors::{DocumentsError, Result};
    pub use crate::events::{
        FormDeleted, FormUpdated, FormUploaded, PostalDispatchDeleted, PostalDispatchUpdated,
        PostalDispatched, PostalReceiveDeleted, PostalReceiveUpdated, PostalReceived,
    };
    pub use crate::query::{FormDownloadQuery, PostalDispatchQuery, PostalReceiveQuery};
    pub use crate::repository::{
        FormDownloadRepository, PostalDispatchRepository, PostalReceiveRepository,
    };
    pub use crate::services::{
        delete_form_service, delete_postal_dispatch_service, delete_postal_receive_service,
        dispatch_postal_service, receive_postal_service, track_postal_service, update_form_service,
        update_postal_dispatch_service, update_postal_receive_service, upload_form_service,
        FormService, PostalService,
    };
    pub use crate::value_objects::{
        ActiveStatus, DispatchDate, DocumentType, DocumentVisibility, FileReference,
        FormDescription, FormDownloadId, FormTitle, FromAddress, FromTitle, PostalAddress,
        PostalDispatchId, PostalNote, PostalReceiveId, PostalReferenceNo, PostalTitle, PublishDate,
        ReceiveDate, ShowPublic, ToAddress, ToTitle, Url,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{
        DeleteFormCommand, DeletePostalDispatchCommand, DeletePostalReceiveCommand,
        DispatchPostalCommand, FormDeleted, FormDownloadFile, FormDownloadFileId, FormDownloadLink,
        FormDownloadLinkId, FormDownloadQuery, FormDownloadRepository, FormUpdated, FormUploaded,
        PostalDispatchAttachment, PostalDispatchAttachmentId, PostalDispatchDeleted,
        PostalDispatchQuery, PostalDispatchRepository, PostalDispatchUpdated, PostalDispatched,
        PostalReceiveAttachment, PostalReceiveAttachmentId, PostalReceiveDeleted,
        PostalReceiveQuery, PostalReceiveRepository, PostalReceiveUpdated, PostalReceived,
        ReceivePostalCommand, TrackPostalCommand, UpdateFormCommand, UpdatePostalDispatchCommand,
        UpdatePostalReceiveCommand, UploadFormCommand,
    };
    use crate::value_objects::{
        ActiveStatus, DispatchDate, DocumentType, DocumentVisibility, FileReference,
        FormDescription, FormDownloadId, FormTitle, FromAddress, FromTitle, PostalAddress,
        PostalDispatchId, PostalNote, PostalReceiveId, PostalReferenceNo, PostalTitle, PublishDate,
        ReceiveDate, ShowPublic, ToAddress, ToTitle, Url,
    };
    #[test]
    fn package_metadata_is_set() {
        assert_eq!(PACKAGE_NAME, "educore-documents");
        assert!(!PACKAGE_VERSION.is_empty());
    }
    #[test]
    fn prelude_re_exports_resolve() {
        // Compile-time test: every public type listed in the prelude exists.
        // (No runtime assertion needed — this fails to compile if a re-export
        // points at a non-existent type.)
        let _: fn() -> FormDownload = || unreachable!();
        let _: fn() -> PostalDispatch = || unreachable!();
        let _: fn() -> PostalReceive = || unreachable!();
    }

    // -------------------------------------------------------------------------
    // Phase 11 / 4-tests — extended prelude re-export smoke tests.
    //
    // The body of each function is `unreachable!()`; the test passes if
    // and only if the type path resolves. This is the engine's
    // compile-time proof that the prelude is intact.
    // -------------------------------------------------------------------------

    #[test]
    fn prelude_commands_resolve() {
        let _: fn() -> DeleteFormCommand = || unreachable!();
        let _: fn() -> DeletePostalDispatchCommand = || unreachable!();
        let _: fn() -> DeletePostalReceiveCommand = || unreachable!();
        let _: fn() -> DispatchPostalCommand = || unreachable!();
        let _: fn() -> ReceivePostalCommand = || unreachable!();
        let _: fn() -> TrackPostalCommand = || unreachable!();
        let _: fn() -> UpdateFormCommand = || unreachable!();
        let _: fn() -> UpdatePostalDispatchCommand = || unreachable!();
        let _: fn() -> UpdatePostalReceiveCommand = || unreachable!();
        let _: fn() -> UploadFormCommand = || unreachable!();
    }

    #[test]
    fn prelude_events_resolve() {
        let _: fn() -> FormDeleted = || unreachable!();
        let _: fn() -> FormUpdated = || unreachable!();
        let _: fn() -> FormUploaded = || unreachable!();
        let _: fn() -> PostalDispatchDeleted = || unreachable!();
        let _: fn() -> PostalDispatchUpdated = || unreachable!();
        let _: fn() -> PostalDispatched = || unreachable!();
        let _: fn() -> PostalReceiveDeleted = || unreachable!();
        let _: fn() -> PostalReceiveUpdated = || unreachable!();
        let _: fn() -> PostalReceived = || unreachable!();
    }

    #[test]
    fn prelude_entities_resolve() {
        let _: fn() -> FormDownloadFileId = || unreachable!();
        let _: fn() -> FormDownloadLinkId = || unreachable!();
        let _: fn() -> PostalDispatchAttachmentId = || unreachable!();
        let _: fn() -> PostalReceiveAttachmentId = || unreachable!();
    }

    #[test]
    fn prelude_value_objects_resolve() {
        let _: fn() -> ActiveStatus = || unreachable!();
        let _: fn() -> DispatchDate = || unreachable!();
        let _: fn() -> DocumentType = || unreachable!();
        let _: fn() -> DocumentVisibility = || unreachable!();
        let _: fn() -> FileReference = || unreachable!();
        let _: fn() -> FormDescription = || unreachable!();
        let _: fn() -> FormDownloadId = || unreachable!();
        let _: fn() -> FormTitle = || unreachable!();
        let _: fn() -> FromAddress = || unreachable!();
        let _: fn() -> FromTitle = || unreachable!();
        let _: fn() -> PostalAddress = || unreachable!();
        let _: fn() -> PostalDispatchId = || unreachable!();
        let _: fn() -> PostalNote = || unreachable!();
        let _: fn() -> PostalReceiveId = || unreachable!();
        let _: fn() -> PostalReferenceNo = || unreachable!();
        let _: fn() -> PostalTitle = || unreachable!();
        let _: fn() -> PublishDate = || unreachable!();
        let _: fn() -> ReceiveDate = || unreachable!();
        let _: fn() -> ShowPublic = || unreachable!();
        let _: fn() -> ToAddress = || unreachable!();
        let _: fn() -> ToTitle = || unreachable!();
        let _: fn() -> Url = || unreachable!();
    }

    #[test]
    fn prelude_aggregate_children_resolve() {
        let _: fn() -> FormDownloadFile = || unreachable!();
        let _: fn() -> FormDownloadLink = || unreachable!();
        let _: fn() -> PostalDispatchAttachment = || unreachable!();
        let _: fn() -> PostalReceiveAttachment = || unreachable!();
    }

    #[test]
    fn prelude_repositories_resolve() {
        // The traits are object-safe (their existence as
        // `Box<dyn ...>` is proven in the `repository.rs` test
        // block). The compile-time check here is that the
        // trait re-exports are reachable. We use
        // `std::any::TypeId::of` on a `dyn Trait` reference;
        // the reference requires the trait name to resolve
        // and to be object-safe.
        let _ = std::any::TypeId::of::<dyn FormDownloadRepository>();
        let _ = std::any::TypeId::of::<dyn PostalDispatchRepository>();
        let _ = std::any::TypeId::of::<dyn PostalReceiveRepository>();
    }

    #[test]
    fn prelude_query_structs_resolve() {
        let _: fn() -> FormDownloadQuery = || unreachable!();
        let _: fn() -> PostalDispatchQuery = || unreachable!();
        let _: fn() -> PostalReceiveQuery = || unreachable!();
    }
}
