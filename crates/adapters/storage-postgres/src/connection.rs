//! PostgreSQL connection management.
//!
//! The `PostgresConnection` wraps a `sqlx::PgPool` and the
//! `SchoolId` the adapter is scoped to. The pool is created via
//! `sqlx::postgres::PgPoolOptions` and a `sqlx::postgres::PgConnectOptions`
//! that carries an `after_connect` hook. The hook issues
//! `SET search_path = engine, public` on every new connection so
//! the adapter can reference the engine's six cross-cutting
//! tables by their short names (`outbox`, `audit_log`, ...) in
//! SQL strings.
//!
//! See `docs/schemas/sql-dialects/postgresql.md` § "Schemas" for
//! the `search_path` strategy and `migrations/engine/0000_engine_core.postgres.sql`
//! for the DDL that creates the `engine` schema and the six
//! tables.

use std::fmt;

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

/// A handle to a connected PostgreSQL instance. The `PgPool` is
/// cheaply cloneable (it is internally an `Arc`); the wrapper
/// derives `Clone` so the storage adapter, the transaction, and
/// the four sub-port handles can each hold their own reference
/// without lifetime gymnastics.
#[derive(Clone)]
pub struct PostgresConnection {
    inner: PgPool,
    /// The school the adapter is scoped to. Every cross-cutting
    /// table read/write is filtered by this school; the engine
    /// enforces tenant isolation at the `TenantContext` layer
    /// (see `docs/schemas/tenancy-schema.md`).
    school: SchoolId,
}

impl fmt::Debug for PostgresConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostgresConnection")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl PostgresConnection {
    /// Opens a `sqlx::PgPool` against the URL and registers the
    /// `engine` schema as the default search path. The pool's
    /// connection hook is the recommended sqlx pattern for
    /// per-session setup (it runs once per new connection, not
    /// per query).
    ///
    /// The URL must be a syntactically valid `postgres://` URL
    /// (sqlx's `PgConnectOptions::from_str` parser is strict).
    ///
    /// # Errors
    /// - `Infrastructure` if the URL cannot be parsed or the
    ///   pool cannot reach the database server.
    #[instrument(skip(url), fields(school = %school))]
    pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
        // `sqlx::postgres::PgPoolOptions` exposes `after_connect`
        // for per-connection setup. The closure runs once per
        // new connection (not per query) and returns a
        // boxed future. The `Send + Sync + 'static` bound on
        // the closure is required because the pool shares the
        // callback across all worker threads.
        let pool = PgPoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set the per-connection search path so
                    // the adapter can use unqualified table
                    // names like `outbox` and `audit_log` in
                    // its queries. The canonical DDL
                    // (`0000_engine_core.postgres.sql`) also
                    // issues `SET search_path = engine,
                    // public` at the script level, but doing
                    // it on every connection covers consumers
                    // that connect to an already-migrated
                    // database.
                    sqlx::query("SET search_path = engine, public")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(url)
            .await
            .map_err(DomainError::infrastructure)?;
        Ok(Self {
            inner: pool,
            school,
        })
    }

    /// Returns the inner `sqlx::PgPool`. Used by the adapter, the
    /// transaction, and the sub-port impls.
    pub fn db(&self) -> &PgPool {
        &self.inner
    }

    /// Returns the school the connection is scoped to.
    pub fn school(&self) -> SchoolId {
        self.school
    }

    /// Consumes the connection and returns the inner `PgPool`.
    /// Used by `PostgresStorageAdapter::close` and by the
    /// sub-port impls that need owned pool access.
    pub fn into_inner(self) -> PgPool {
        self.inner
    }
}
