//! Generic per-aggregate `Repository<A>` trait.
//!
//! Per `docs/ports/storage.md` § 2, the storage adapter hands
//! out a repository per aggregate root. The full spec enumerates
//! ~80+ per-aggregate repository types (one per aggregate across
//! all 15 domains). For Phase 0 minimum-viable, we expose a
//! single generic `Repository<A>` trait that all domain crates
//! can use; when a domain needs aggregate-specific methods it
//! can wrap or extend the generic trait.
//!
//! The trait is generic over the aggregate type `A`. The
//! aggregate must be `Send + Sync + Clone + 'static` so the
//! repository can return owned values and the storage adapter
//! can hold `Arc<dyn Repository<A>>` in a multi-threaded runtime.

use async_trait::async_trait;
use uuid::Uuid;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

/// A read/write handle to one aggregate type within a storage
/// adapter. Object-safe: consumers typically hold
/// `Arc<dyn Repository<Student>>`.
#[async_trait]
pub trait Repository<A>: Send + Sync
where
    A: Send + Sync + Clone + 'static,
{
    /// Fetches the aggregate with the given id, scoped to
    /// `school_id`. Returns `Ok(None)` if the row does not
    /// exist (not an error — the dispatcher treats `None` as
    /// a `NotFound` domain error).
    async fn get(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>>;

    /// Fetches the aggregate with the given id, scoped to
    /// `school_id`, with the `IncludeRetired` flag set. By
    /// default, soft-deleted rows are excluded.
    async fn get_including_retired(&self, school_id: SchoolId, id: Uuid) -> Result<Option<A>> {
        // Default implementation: same as `get`. Adapters with
        // a dedicated `IncludeRetired` flag override this.
        let _ = school_id;
        let _ = id;
        self.get(school_id, id).await
    }

    /// Lists all aggregates for `school_id`, paginated. Per
    /// `docs/ports/storage.md` § 2, hydration is page-based
    /// (`offset`, `limit`).
    async fn list(&self, school_id: SchoolId, offset: u32, limit: u32) -> Result<Vec<A>>;

    /// Returns the count of aggregates for `school_id`,
    /// excluding soft-deleted rows.
    async fn count(&self, school_id: SchoolId) -> Result<u64>;

    /// Inserts a new aggregate. Returns `Err(Conflict)` if a
    /// row with the same primary key already exists in the
    /// school.
    async fn insert(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;

    /// Updates an existing aggregate. Returns `Err(NotFound)`
    /// if the row does not exist; `Err(Conflict)` on
    /// optimistic-concurrency mismatch (the engine retries
    /// the command after reloading the aggregate).
    async fn update(&self, school_id: SchoolId, aggregate: &A) -> Result<()>;

    /// Soft-deletes the aggregate (sets `active_status = 0`).
    /// Returns `Err(NotFound)` if the row does not exist.
    /// Hard-delete is reserved for GDPR erasure and is exposed
    /// via a separate operator-only path on the storage
    /// adapter, not on the repository.
    async fn soft_delete(&self, school_id: SchoolId, id: Uuid) -> Result<()>;
}
