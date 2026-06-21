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
        let _: Option<FormDownload> = None;
        let _: Option<PostalDispatch> = None;
        let _: Option<PostalReceive> = None;
    }

    // -------------------------------------------------------------------------
    // Phase 11 / 4-tests — extended prelude re-export smoke tests.
    //
    // The body of each test binds a typed `Option<T>` to `None`; the test
    // passes if and only if the type path resolves. This is the engine's
    // compile-time proof that the prelude is intact.
    // -------------------------------------------------------------------------

    #[test]
    fn prelude_commands_resolve() {
        let _: Option<DeleteFormCommand> = None;
        let _: Option<DeletePostalDispatchCommand> = None;
        let _: Option<DeletePostalReceiveCommand> = None;
        let _: Option<DispatchPostalCommand> = None;
        let _: Option<ReceivePostalCommand> = None;
        let _: Option<TrackPostalCommand> = None;
        let _: Option<UpdateFormCommand> = None;
        let _: Option<UpdatePostalDispatchCommand> = None;
        let _: Option<UpdatePostalReceiveCommand> = None;
        let _: Option<UploadFormCommand> = None;
    }

    #[test]
    fn prelude_events_resolve() {
        let _: Option<FormDeleted> = None;
        let _: Option<FormUpdated> = None;
        let _: Option<FormUploaded> = None;
        let _: Option<PostalDispatchDeleted> = None;
        let _: Option<PostalDispatchUpdated> = None;
        let _: Option<PostalDispatched> = None;
        let _: Option<PostalReceiveDeleted> = None;
        let _: Option<PostalReceiveUpdated> = None;
        let _: Option<PostalReceived> = None;
    }

    #[test]
    fn prelude_entities_resolve() {
        let _: Option<FormDownloadFileId> = None;
        let _: Option<FormDownloadLinkId> = None;
        let _: Option<PostalDispatchAttachmentId> = None;
        let _: Option<PostalReceiveAttachmentId> = None;
    }

    #[test]
    fn prelude_value_objects_resolve() {
        let _: Option<ActiveStatus> = None;
        let _: Option<DispatchDate> = None;
        let _: Option<DocumentType> = None;
        let _: Option<DocumentVisibility> = None;
        let _: Option<FileReference> = None;
        let _: Option<FormDescription> = None;
        let _: Option<FormDownloadId> = None;
        let _: Option<FormTitle> = None;
        let _: Option<FromAddress> = None;
        let _: Option<FromTitle> = None;
        let _: Option<PostalAddress> = None;
        let _: Option<PostalDispatchId> = None;
        let _: Option<PostalNote> = None;
        let _: Option<PostalReceiveId> = None;
        let _: Option<PostalReferenceNo> = None;
        let _: Option<PostalTitle> = None;
        let _: Option<PublishDate> = None;
        let _: Option<ReceiveDate> = None;
        let _: Option<ShowPublic> = None;
        let _: Option<ToAddress> = None;
        let _: Option<ToTitle> = None;
        let _: Option<Url> = None;
    }

    #[test]
    fn prelude_aggregate_children_resolve() {
        let _: Option<FormDownloadFile> = None;
        let _: Option<FormDownloadLink> = None;
        let _: Option<PostalDispatchAttachment> = None;
        let _: Option<PostalReceiveAttachment> = None;
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
