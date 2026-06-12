//! SurrealDB connection management.
//!
//! Phase 0 supports the in-memory backend (`Mem`); the
//! RocksDB / TiKV / HTTP backends land in a later phase.

use std::sync::Arc;

use surrealdb::engine::local::Db as LocalDb;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

use educore_core::error::Result;
use educore_core::ids::SchoolId;

/// The concrete SurrealDB database type used by the adapter
/// in Phase 0. The local backend's `Db` type alias is what
/// `Surreal::new::<Mem>(())` produces.
pub type Db = Surreal<LocalDb>;

/// A handle to a connected SurrealDB instance. The connection
/// is cheap to clone (it's an `Arc` over the underlying
/// `Surreal`).
#[derive(Clone)]
pub struct SurrealConnection {
    inner: Arc<Db>,
    /// The school the adapter is scoped to. Every cross-cutting
    /// table read/write is filtered by this school; the engine
    /// enforces tenant isolation at the `TenantContext` layer
    /// (see `docs/schemas/tenancy-schema.md`).
    school: SchoolId,
}

impl std::fmt::Debug for SurrealConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurrealConnection")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SurrealConnection {
    /// Opens an in-memory SurrealDB instance and sets the
    /// engine's `educore` namespace / `engine` database. This
    /// is the default connection for tests and single-process
    /// embedded deployments (per `ADR-017`).
    ///
    /// # Errors
    /// - `Infrastructure` if the in-memory engine fails to
    ///   start (e.g. out of memory, internal panic).
    pub async fn in_memory(school: SchoolId) -> Result<Self> {
        let db: Db = Surreal::new::<Mem>(())
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        db.use_ns("educore")
            .use_db("engine")
            .await
            .map_err(educore_core::error::DomainError::infrastructure)?;
        Ok(Self {
            inner: Arc::new(db),
            school,
        })
    }

    /// Returns the inner SurrealDB handle. Used by the
    /// adapter, the transaction, and the sub-port impls.
    pub fn db(&self) -> &Db {
        &self.inner
    }

    /// Returns the school the connection is scoped to.
    pub fn school(&self) -> SchoolId {
        self.school
    }

    /// Consumes the connection and returns the inner `Db`. Used
    /// by `SurrealStorageAdapter::close` which needs to consume
    /// the trait object.
    pub fn into_inner(self) -> Arc<Db> {
        self.inner
    }
}
