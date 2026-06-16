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
    async fn get(
        &self,
        id: FormDownloadId,
    ) -> StorageResult<Option<FormDownload>>;
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
    async fn count(
        &self,
        school: SchoolId,
        q: FormDownloadQuery,
    ) -> StorageResult<u64>;
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
    async fn get(
        &self,
        id: PostalReceiveId,
    ) -> StorageResult<Option<PostalReceive>>;
    /// List postal receives for a school matching the typed
    /// query.
    async fn list(
        &self,
        school: SchoolId,
        q: PostalReceiveQuery,
    ) -> StorageResult<Vec<PostalReceive>>;
    /// Insert a new postal receive (or upsert on a soft-delete
    /// update).
    async fn insert(
        &self,
        receive: &PostalReceive,
    ) -> StorageResult<()>;
    /// Update an existing postal receive.
    async fn update(
        &self,
        receive: &PostalReceive,
    ) -> StorageResult<()>;
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
