//! Documents-domain repository port traits.

#![allow(dead_code, clippy::all)]
#![allow(missing_docs)]

// === FormDownload repository section begin (owner: 3A) ===

use async_trait::async_trait;
use chrono::NaiveDate;

use educore_core::error::Result as StorageResult;
use educore_core::ids::SchoolId;

use crate::aggregate::FormDownload;
use crate::query::FormDownloadQuery;
use crate::value_objects::FormDownloadId;

/// Repository port for the `FormDownload` aggregate. The
/// soft-delete invariant is enforced at the aggregate level
/// (the trait still exposes `update()` so the soft-delete path
/// can persist the `active_status = false` flip).
#[async_trait]
pub trait FormDownloadRepository: Send + Sync {
    /// Fetch a form by its typed id. Returns `Ok(None)` if the
    /// row does not exist or is soft-deleted.
    async fn get(&self, id: FormDownloadId) -> StorageResult<Option<FormDownload>>;
    /// List forms for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
    ) -> StorageResult<Vec<FormDownload>>;
    /// List public-visible forms for a school.
    async fn list_public(&self, school: SchoolId) -> StorageResult<Vec<FormDownload>>;
    /// Insert a new form (or upsert on a soft-delete update).
    async fn insert(&self, form: &FormDownload) -> StorageResult<()>;
    /// Update an existing form.
    async fn update(&self, form: &FormDownload) -> StorageResult<()>;
    /// List forms whose `publish_date` falls within the
    /// inclusive range `[from, to]`.
    async fn by_publish_date(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> StorageResult<Vec<FormDownload>>;
    /// Count forms for a school matching the typed query.
    async fn count(&self, school: SchoolId, q: FormDownloadQuery) -> StorageResult<u64>;
    /// Page forms for a school, oldest-first by `publish_date`.
    /// Returns a `Vec<FormDownload>` of length `<= limit`,
    /// starting at `offset`. Returns an empty `Vec` if there
    /// are no further rows.
    async fn page(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
        offset: u32,
        limit: u32,
    ) -> StorageResult<Vec<FormDownload>>;
}

/// Object-safety smoke test (compile-time).
fn _assert_object_safe() {
    fn _f(_: Box<dyn FormDownloadRepository>) {}
}

// === FormDownload repository section end ===

// === PostalDispatch repository section begin (owner: 3B) ===

// 3A above already imports `async_trait::async_trait`,
// `chrono::NaiveDate`, and `educore_core::ids::SchoolId`, and
// renames `educore_core::error::Result` to `StorageResult`.
// Re-importing any of them here is an E0252 duplicate. The
// `AcademicYearId`, `PostalDispatch`, `PostalDispatchQuery`,
// and `PostalReferenceNo` types below are all new to this
// section. The trait uses `StorageResult` (3A's rename) to
// match the engine convention for repository port traits.

use crate::aggregate::{AcademicYearId, PostalDispatch};
use crate::query::PostalDispatchQuery;
use crate::value_objects::{PostalDispatchId, PostalReferenceNo};

/// Repository port for the [`PostalDispatch`](crate::aggregate::PostalDispatch)
/// aggregate. Storage adapters (PostgreSQL, MySQL, SQLite)
/// implement this trait. The default CRUD set is `get` /
/// `list` / `insert` / `update`; `delete` is omitted because
/// the engine never hard-deletes a postal dispatch (the
/// soft-delete path is the only delete; the soft-delete flag
/// is on the aggregate itself and is updated via
/// [`update`](Self::update)). The four `find_*` / `between` /
/// `by_academic_year` accessors support the Postal Service's
/// query paths.
#[async_trait]
pub trait PostalDispatchRepository: Send + Sync {
    /// Fetch a dispatch by its typed id.
    async fn get(&self, id: PostalDispatchId) -> StorageResult<Option<PostalDispatch>>;

    /// List dispatches for a school matching the typed query.
    async fn list(
        &self,
        school: SchoolId,
        q: PostalDispatchQuery,
    ) -> StorageResult<Vec<PostalDispatch>>;

    /// Insert a new dispatch.
    async fn insert(&self, dispatch: &PostalDispatch) -> StorageResult<()>;

    /// Update an existing dispatch. Used by the update and
    /// soft-delete service paths.
    async fn update(&self, dispatch: &PostalDispatch) -> StorageResult<()>;

    /// Find dispatches whose `reference_no` matches the given
    /// value within a school. The reference number is unique
    /// within `(school_id, academic_id)`, but the same
    /// reference can appear across multiple academic years,
    /// so this returns a `Vec`.
    async fn find_by_reference(
        &self,
        school: SchoolId,
        reference: &PostalReferenceNo,
    ) -> StorageResult<Vec<PostalDispatch>>;

    /// List dispatches whose `date` falls within the inclusive
    /// range `[from, to]`.
    async fn between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> StorageResult<Vec<PostalDispatch>>;

    /// List dispatches scoped to the given academic year
    /// within a school.
    async fn by_academic_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> StorageResult<Vec<PostalDispatch>>;
}

/// Compile-time object-safety smoke test. If the trait ever
/// gains a generic method, an associated type, or a `Self:
/// Sized` bound, this function will fail to compile.
fn _assert_postal_dispatch_repo_object_safe() {
    fn _f(_: Box<dyn PostalDispatchRepository>) {}
}

// === PostalDispatch repository section end ===

// === PostalReceive repository section begin (owner: 3C) ===

// 3A above already imports `async_trait::async_trait`,
// `chrono::NaiveDate`, and `educore_core::ids::SchoolId`, and
// renames `educore_core::error::Result` to `StorageResult`.
// 3B above imports `AcademicYearId` (from `crate::aggregate`)
// and `PostalReferenceNo` (from `crate::value_objects`).
// Re-importing any of them here is an E0252 duplicate. The
// `PostalReceive`, `PostalReceiveQuery`, and `PostalReceiveId`
// types below are all new to this section.

use crate::aggregate::PostalReceive;
use crate::query::PostalReceiveQuery;
use crate::value_objects::PostalReceiveId;

/// Repository port for the `PostalReceive` aggregate. The
/// soft-delete invariant is enforced at the aggregate level
/// (the trait still exposes `update()` so the soft-delete path
/// can persist the `active_status = false` flip).
///
/// Every read method that filters on `school_id` enforces the
/// tenant isolation invariant; adapters MUST apply the
/// `school_id` predicate to every emitted query and the engine
/// `lint` sub-module cross-checks that the call sites do too.
#[async_trait]
pub trait PostalReceiveRepository: Send + Sync {
    /// Fetch a postal receive by its typed id. Returns
    /// `Ok(None)` if the row does not exist or is soft-deleted.
    async fn get(&self, id: PostalReceiveId) -> StorageResult<Option<PostalReceive>>;
    /// List postal receives for a school matching the typed
    /// query.
    async fn list(
        &self,
        school: SchoolId,
        q: PostalReceiveQuery,
    ) -> StorageResult<Vec<PostalReceive>>;
    /// Insert a new postal receive (or upsert on a soft-delete
    /// update).
    async fn insert(&self, receive: &PostalReceive) -> StorageResult<()>;
    /// Update an existing postal receive.
    async fn update(&self, receive: &PostalReceive) -> StorageResult<()>;
    /// List postal receives whose `reference_no` matches the
    /// given value (within a school). Used by the
    /// [`TrackPostalCommand`](crate::commands::TrackPostalCommand)
    /// query command. Returns an empty `Vec` when no match is
    /// found.
    async fn find_by_reference(
        &self,
        school: SchoolId,
        reference: &PostalReferenceNo,
    ) -> StorageResult<Vec<PostalReceive>>;
    /// List postal receives whose `date` falls within the
    /// inclusive range `[from, to]`.
    async fn between(
        &self,
        school: SchoolId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> StorageResult<Vec<PostalReceive>>;
    /// List postal receives scoped to a given academic year
    /// within a school.
    async fn by_academic_year(
        &self,
        school: SchoolId,
        year: AcademicYearId,
    ) -> StorageResult<Vec<PostalReceive>>;
}

/// Object-safety smoke test (compile-time). If a future
/// revision of [`PostalReceiveRepository`] loses object
/// safety (e.g. gains an associated type), this function will
/// fail to compile, signalling the regression immediately.
fn _assert_postal_receive_repo_object_safe() {
    fn _f(_: Box<dyn PostalReceiveRepository>) {}
}

// === PostalReceive repository section end ===

// === NewFormDownload repository section begin (owner: wave9.1) ===

// Sections 3A/3B/3C above already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. Re-importing any of them here is an E0252
// duplicate. The `NewFormDownload`, `NewFormDownloadId` types below
// are new to this section.

use crate::aggregate::NewFormDownload;
use crate::value_objects::NewFormDownloadId;

/// Repository port for the
/// [`NewFormDownload`](crate::aggregate::NewFormDownload) aggregate.
/// Minimal CRUD surface per wave 9.1 (`get`, `insert`, `update`).
#[async_trait]
pub trait NewFormDownloadRepository: Send + Sync {
    /// Fetch a `NewFormDownload` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: NewFormDownloadId) -> StorageResult<Option<NewFormDownload>>;
    /// Insert a new `NewFormDownload`.
    async fn insert(&self, value: &NewFormDownload) -> StorageResult<()>;
    /// Update an existing `NewFormDownload`.
    async fn update(&self, value: &NewFormDownload) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_new_form_download_repo_object_safe() {
    fn _f(_: Box<dyn NewFormDownloadRepository>) {}
}

// === NewFormDownload repository section end ===

// === UpdateFormDownload repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `UpdateFormDownload`,
// `UpdateFormDownloadId` types below are new to this section.

use crate::aggregate::UpdateFormDownload;
use crate::value_objects::UpdateFormDownloadId;

/// Repository port for the
/// [`UpdateFormDownload`](crate::aggregate::UpdateFormDownload)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait UpdateFormDownloadRepository: Send + Sync {
    /// Fetch an `UpdateFormDownload` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: UpdateFormDownloadId) -> StorageResult<Option<UpdateFormDownload>>;
    /// Insert a new `UpdateFormDownload`.
    async fn insert(&self, value: &UpdateFormDownload) -> StorageResult<()>;
    /// Update an existing `UpdateFormDownload`.
    async fn update(&self, value: &UpdateFormDownload) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_update_form_download_repo_object_safe() {
    fn _f(_: Box<dyn UpdateFormDownloadRepository>) {}
}

// === UpdateFormDownload repository section end ===

// === FormDownloadFile repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `FormDownloadFile`,
// `FormDownloadFileId` types below are new to this section.

use crate::aggregate::FormDownloadFile;
use crate::value_objects::FormDownloadFileId;

/// Repository port for the
/// [`FormDownloadFile`](crate::aggregate::FormDownloadFile)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait FormDownloadFileRepository: Send + Sync {
    /// Fetch a `FormDownloadFile` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: FormDownloadFileId) -> StorageResult<Option<FormDownloadFile>>;
    /// Insert a new `FormDownloadFile`.
    async fn insert(&self, value: &FormDownloadFile) -> StorageResult<()>;
    /// Update an existing `FormDownloadFile`.
    async fn update(&self, value: &FormDownloadFile) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_form_download_file_repo_object_safe() {
    fn _f(_: Box<dyn FormDownloadFileRepository>) {}
}

// === FormDownloadFile repository section end ===

// === FormDownloadLink repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `FormDownloadLink`,
// `FormDownloadLinkId` types below are new to this section.

use crate::aggregate::FormDownloadLink;
use crate::value_objects::FormDownloadLinkId;

/// Repository port for the
/// [`FormDownloadLink`](crate::aggregate::FormDownloadLink)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait FormDownloadLinkRepository: Send + Sync {
    /// Fetch a `FormDownloadLink` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: FormDownloadLinkId) -> StorageResult<Option<FormDownloadLink>>;
    /// Insert a new `FormDownloadLink`.
    async fn insert(&self, value: &FormDownloadLink) -> StorageResult<()>;
    /// Update an existing `FormDownloadLink`.
    async fn update(&self, value: &FormDownloadLink) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_form_download_link_repo_object_safe() {
    fn _f(_: Box<dyn FormDownloadLinkRepository>) {}
}

// === FormDownloadLink repository section end ===

// === NewPostalDispatch repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `NewPostalDispatch`,
// `NewPostalDispatchId` types below are new to this section.

use crate::aggregate::NewPostalDispatch;
use crate::value_objects::NewPostalDispatchId;

/// Repository port for the
/// [`NewPostalDispatch`](crate::aggregate::NewPostalDispatch)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait NewPostalDispatchRepository: Send + Sync {
    /// Fetch a `NewPostalDispatch` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: NewPostalDispatchId) -> StorageResult<Option<NewPostalDispatch>>;
    /// Insert a new `NewPostalDispatch`.
    async fn insert(&self, value: &NewPostalDispatch) -> StorageResult<()>;
    /// Update an existing `NewPostalDispatch`.
    async fn update(&self, value: &NewPostalDispatch) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_new_postal_dispatch_repo_object_safe() {
    fn _f(_: Box<dyn NewPostalDispatchRepository>) {}
}

// === NewPostalDispatch repository section end ===

// === UpdatePostalDispatch repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `UpdatePostalDispatch`,
// `UpdatePostalDispatchId` types below are new to this section.

use crate::aggregate::UpdatePostalDispatch;
use crate::value_objects::UpdatePostalDispatchId;

/// Repository port for the
/// [`UpdatePostalDispatch`](crate::aggregate::UpdatePostalDispatch)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait UpdatePostalDispatchRepository: Send + Sync {
    /// Fetch an `UpdatePostalDispatch` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: UpdatePostalDispatchId) -> StorageResult<Option<UpdatePostalDispatch>>;
    /// Insert a new `UpdatePostalDispatch`.
    async fn insert(&self, value: &UpdatePostalDispatch) -> StorageResult<()>;
    /// Update an existing `UpdatePostalDispatch`.
    async fn update(&self, value: &UpdatePostalDispatch) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_update_postal_dispatch_repo_object_safe() {
    fn _f(_: Box<dyn UpdatePostalDispatchRepository>) {}
}

// === UpdatePostalDispatch repository section end ===

// === PostalDispatchAttachment repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `PostalDispatchAttachment`,
// `PostalDispatchAttachmentId` types below are new to this section.

use crate::aggregate::PostalDispatchAttachment;
use crate::value_objects::PostalDispatchAttachmentId;

/// Repository port for the
/// [`PostalDispatchAttachment`](crate::aggregate::PostalDispatchAttachment)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait PostalDispatchAttachmentRepository: Send + Sync {
    /// Fetch a `PostalDispatchAttachment` by its typed id.
    /// Returns `Ok(None)` if the row does not exist.
    async fn get(
        &self,
        id: PostalDispatchAttachmentId,
    ) -> StorageResult<Option<PostalDispatchAttachment>>;
    /// Insert a new `PostalDispatchAttachment`.
    async fn insert(&self, value: &PostalDispatchAttachment) -> StorageResult<()>;
    /// Update an existing `PostalDispatchAttachment`.
    async fn update(&self, value: &PostalDispatchAttachment) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_postal_dispatch_attachment_repo_object_safe() {
    fn _f(_: Box<dyn PostalDispatchAttachmentRepository>) {}
}

// === PostalDispatchAttachment repository section end ===

// === NewPostalReceive repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `NewPostalReceive`,
// `NewPostalReceiveId` types below are new to this section.

use crate::aggregate::NewPostalReceive;
use crate::value_objects::NewPostalReceiveId;

/// Repository port for the
/// [`NewPostalReceive`](crate::aggregate::NewPostalReceive)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait NewPostalReceiveRepository: Send + Sync {
    /// Fetch a `NewPostalReceive` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: NewPostalReceiveId) -> StorageResult<Option<NewPostalReceive>>;
    /// Insert a new `NewPostalReceive`.
    async fn insert(&self, value: &NewPostalReceive) -> StorageResult<()>;
    /// Update an existing `NewPostalReceive`.
    async fn update(&self, value: &NewPostalReceive) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_new_postal_receive_repo_object_safe() {
    fn _f(_: Box<dyn NewPostalReceiveRepository>) {}
}

// === NewPostalReceive repository section end ===

// === UpdatePostalReceive repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `UpdatePostalReceive`,
// `UpdatePostalReceiveId` types below are new to this section.

use crate::aggregate::UpdatePostalReceive;
use crate::value_objects::UpdatePostalReceiveId;

/// Repository port for the
/// [`UpdatePostalReceive`](crate::aggregate::UpdatePostalReceive)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait UpdatePostalReceiveRepository: Send + Sync {
    /// Fetch an `UpdatePostalReceive` by its typed id. Returns
    /// `Ok(None)` if the row does not exist.
    async fn get(&self, id: UpdatePostalReceiveId) -> StorageResult<Option<UpdatePostalReceive>>;
    /// Insert a new `UpdatePostalReceive`.
    async fn insert(&self, value: &UpdatePostalReceive) -> StorageResult<()>;
    /// Update an existing `UpdatePostalReceive`.
    async fn update(&self, value: &UpdatePostalReceive) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_update_postal_receive_repo_object_safe() {
    fn _f(_: Box<dyn UpdatePostalReceiveRepository>) {}
}

// === UpdatePostalReceive repository section end ===

// === PostalReceiveAttachment repository section begin (owner: wave9.1) ===

// Prior sections already import `async_trait::async_trait`,
// `chrono::NaiveDate`, `educore_core::ids::SchoolId`, and the
// `StorageResult` rename. The `PostalReceiveAttachment`,
// `PostalReceiveAttachmentId` types below are new to this section.

use crate::aggregate::PostalReceiveAttachment;
use crate::value_objects::PostalReceiveAttachmentId;

/// Repository port for the
/// [`PostalReceiveAttachment`](crate::aggregate::PostalReceiveAttachment)
/// aggregate. Minimal CRUD surface per wave 9.1
/// (`get`, `insert`, `update`).
#[async_trait]
pub trait PostalReceiveAttachmentRepository: Send + Sync {
    /// Fetch a `PostalReceiveAttachment` by its typed id.
    /// Returns `Ok(None)` if the row does not exist.
    async fn get(
        &self,
        id: PostalReceiveAttachmentId,
    ) -> StorageResult<Option<PostalReceiveAttachment>>;
    /// Insert a new `PostalReceiveAttachment`.
    async fn insert(&self, value: &PostalReceiveAttachment) -> StorageResult<()>;
    /// Update an existing `PostalReceiveAttachment`.
    async fn update(&self, value: &PostalReceiveAttachment) -> StorageResult<()>;
}

/// Object-safety smoke test (compile-time).
fn _assert_postal_receive_attachment_repo_object_safe() {
    fn _f(_: Box<dyn PostalReceiveAttachmentRepository>) {}
}

// === PostalReceiveAttachment repository section end ===

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

    // Compile-time object-safety smoke tests. Each block forces
    // the trait to be coerced into a `Box<dyn ...>`; if the trait
    // loses object safety (e.g. gains an associated type or a
    // `Self: Sized` bound), the coercion fails to compile.

    fn _assert_object_safe() {
        fn _f(_: Box<dyn FormDownloadRepository>) {}
        fn _g(_: Box<dyn PostalDispatchRepository>) {}
        fn _h(_: Box<dyn PostalReceiveRepository>) {}
        fn _i(_: Box<dyn NewFormDownloadRepository>) {}
        fn _j(_: Box<dyn UpdateFormDownloadRepository>) {}
        fn _k(_: Box<dyn FormDownloadFileRepository>) {}
        fn _l(_: Box<dyn FormDownloadLinkRepository>) {}
        fn _m(_: Box<dyn NewPostalDispatchRepository>) {}
        fn _n(_: Box<dyn UpdatePostalDispatchRepository>) {}
        fn _o(_: Box<dyn PostalDispatchAttachmentRepository>) {}
        fn _p(_: Box<dyn NewPostalReceiveRepository>) {}
        fn _q(_: Box<dyn UpdatePostalReceiveRepository>) {}
        fn _r(_: Box<dyn PostalReceiveAttachmentRepository>) {}
    }

    // Send + Sync smoke tests. The trait is declared as
    // `Send + Sync`; the assertions below force the compiler to
    // verify the bound at the test site.

    fn _assert_send_sync<T: Send + Sync + ?Sized>() {}

    #[test]
    fn form_download_repository_is_send_and_sync() {
        _assert_send_sync::<dyn FormDownloadRepository>();
    }

    #[test]
    fn postal_dispatch_repository_is_send_and_sync() {
        _assert_send_sync::<dyn PostalDispatchRepository>();
    }

    #[test]
    fn postal_receive_repository_is_send_and_sync() {
        _assert_send_sync::<dyn PostalReceiveRepository>();
    }

    #[test]
    fn new_form_download_repository_is_send_and_sync() {
        _assert_send_sync::<dyn NewFormDownloadRepository>();
    }

    #[test]
    fn update_form_download_repository_is_send_and_sync() {
        _assert_send_sync::<dyn UpdateFormDownloadRepository>();
    }

    #[test]
    fn form_download_file_repository_is_send_and_sync() {
        _assert_send_sync::<dyn FormDownloadFileRepository>();
    }

    #[test]
    fn form_download_link_repository_is_send_and_sync() {
        _assert_send_sync::<dyn FormDownloadLinkRepository>();
    }

    #[test]
    fn new_postal_dispatch_repository_is_send_and_sync() {
        _assert_send_sync::<dyn NewPostalDispatchRepository>();
    }

    #[test]
    fn update_postal_dispatch_repository_is_send_and_sync() {
        _assert_send_sync::<dyn UpdatePostalDispatchRepository>();
    }

    #[test]
    fn postal_dispatch_attachment_repository_is_send_and_sync() {
        _assert_send_sync::<dyn PostalDispatchAttachmentRepository>();
    }

    #[test]
    fn new_postal_receive_repository_is_send_and_sync() {
        _assert_send_sync::<dyn NewPostalReceiveRepository>();
    }

    #[test]
    fn update_postal_receive_repository_is_send_and_sync() {
        _assert_send_sync::<dyn UpdatePostalReceiveRepository>();
    }

    #[test]
    fn postal_receive_attachment_repository_is_send_and_sync() {
        _assert_send_sync::<dyn PostalReceiveAttachmentRepository>();
    }

    // Trait-method compile-time proofs. Each helper function
    // exercises the trait method (the body never runs because
    // the async block is never polled in this sync test). If a
    // method is removed or its signature changes, the body of
    // this function will fail to compile.

    #[allow(dead_code, unused_variables)]
    fn _form_repo_methods(r: &dyn FormDownloadRepository) {
        // We call the trait methods inside an `async move`
        // block, which is a future we never poll. The block
        // forces the compiler to type-check every method call.
        // Using owned arguments (no borrowed references) avoids
        // the lifetime capture issue.
        let _f1 = async move {
            let _ = r
                .get(crate::value_objects::FormDownloadId::new(
                    educore_core::ids::SchoolId(uuid::Uuid::nil()),
                    uuid::Uuid::nil(),
                ))
                .await;
        };
        let _f2 = async move {
            let _ = r
                .list(
                    educore_core::ids::SchoolId(uuid::Uuid::nil()),
                    crate::query::FormDownloadQuery::default(),
                )
                .await;
        };
        let _f3 = async move {
            let _ = r
                .list_public(educore_core::ids::SchoolId(uuid::Uuid::nil()))
                .await;
        };
        let _f4 = async move {
            // `insert` and `update` take `&FormDownload`. The
            // form is constructed inline as proof of the trait
            // signature; the body never runs.
            let s = educore_core::ids::SchoolId(uuid::Uuid::nil());
            let id = crate::value_objects::FormDownloadId::new(s, uuid::Uuid::nil());
            let form = crate::aggregate::FormDownload {
                id,
                school_id: s,
                title: crate::value_objects::FormTitle::new("X").unwrap(),
                short_description: None,
                publish_date: crate::value_objects::PublishDate::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                ),
                link: None,
                file: None,
                show_public: crate::value_objects::ShowPublic::default(),
                active_status: crate::value_objects::ActiveStatus::new(true),
                version: educore_core::value_objects::Version::initial(),
                etag: educore_core::value_objects::Etag::placeholder(),
                created_at: educore_core::value_objects::Timestamp::now(),
                updated_at: educore_core::value_objects::Timestamp::now(),
                created_by: educore_core::ids::UserId(uuid::Uuid::nil()),
                updated_by: educore_core::ids::UserId(uuid::Uuid::nil()),
                last_event_id: None,
                correlation_id: educore_core::ids::CorrelationId(uuid::Uuid::nil()),
            };
            let _ = r.insert(&form).await;
            let _ = r.update(&form).await;
        };
        let _f5 = async move {
            let _ = r
                .by_publish_date(
                    educore_core::ids::SchoolId(uuid::Uuid::nil()),
                    chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
                )
                .await;
        };
        let _f6 = async move {
            let _ = r
                .count(
                    educore_core::ids::SchoolId(uuid::Uuid::nil()),
                    crate::query::FormDownloadQuery::default(),
                )
                .await;
        };
        let _f7 = async move {
            let _ = r
                .page(
                    educore_core::ids::SchoolId(uuid::Uuid::nil()),
                    crate::query::FormDownloadQuery::default(),
                    0,
                    10,
                )
                .await;
        };
    }
}
