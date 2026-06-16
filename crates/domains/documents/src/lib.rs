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
        FormDownload, FormDownloadFile, FormDownloadLink, PostalDispatch,
        PostalDispatchAttachment, PostalReceive, PostalReceiveAttachment,
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
        dispatch_postal_service, FormService, PostalService, receive_postal_service,
        track_postal_service, update_form_service, update_postal_dispatch_service,
        update_postal_receive_service, upload_form_service,
    };
    pub use crate::value_objects::{
        ActiveStatus, DispatchDate, DocumentType, DocumentVisibility, FileReference, FormDescription,
        FormDownloadId, FormTitle, FromAddress, FromTitle, PostalAddress, PostalDispatchId,
        PostalNote, PostalReceiveId, PostalReferenceNo, PostalTitle, PublishDate, ReceiveDate,
        ShowPublic, ToAddress, ToTitle, Url,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
