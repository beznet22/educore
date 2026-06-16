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
pub trait PostalDispatchRepository {}
// === PostalDispatch repository section end ===

// === PostalReceive repository section begin (owner: 3C) ===
pub trait PostalReceiveRepository {}
// === PostalReceive repository section end ===
