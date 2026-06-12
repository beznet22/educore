//! SQLite connection management.
//!
//! Wraps `sqlx::SqlitePool` with the engine's per-school
//! scoping and the dialect-mandated PRAGMAs (WAL, NORMAL
//! synchronous, foreign keys ON). The PRAGMAs are issued on
//! every new connection via `SqlitePoolOptions::after_connect`
//! so they apply to both the production file-based path and the
//! in-memory test path.

use std::fmt;
use std::str::FromStr;

use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use tracing::debug;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

/// A handle to a connected SQLite database. The pool is
/// cheaply cloneable (its inner state is `Arc`-shared), so the
/// adapter can hand sub-port clones without re-acquiring
/// connections.
#[derive(Clone)]
pub struct SqliteConnection {
    pool: SqlitePool,
    /// The school the adapter is scoped to. Every cross-cutting
    /// table read/write is filtered by this school; the engine
    /// enforces tenant isolation at the `TenantContext` layer
    /// (see `docs/schemas/tenancy-schema.md`).
    school: SchoolId,
}

impl fmt::Debug for SqliteConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnection")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl SqliteConnection {
    /// Opens an in-memory SQLite database and sets the
    /// engine-mandated PRAGMAs. The pool is constrained to a
    /// single connection so every consumer in the same process
    /// sees the same in-memory database. This is the default
    /// connection for tests and single-process embedded
    /// deployments (per `docs/schemas/sql-dialects/sqlite.md`).
    ///
    /// # Errors
    /// - `Infrastructure` if the in-memory engine fails to
    ///   start (e.g. out of memory, internal panic).
    pub async fn in_memory(school: SchoolId) -> Result<Self> {
        let connect_opts = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "invalid in-memory URL: {e}"
                )))
            })?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("PRAGMA journal_mode = WAL")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA synchronous = NORMAL")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(connect_opts)
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "sqlite in-memory connect: {e}"
                )))
            })?;
        debug!(school = %school, "opened in-memory sqlite");
        Ok(Self { pool, school })
    }

    /// Opens a file-backed SQLite database at `url` and sets
    /// the engine-mandated PRAGMAs. The URL follows the
    /// `sqlx` convention (`sqlite://path/to.db`,
    /// `sqlite:path/to.db`, etc.).
    ///
    /// # Errors
    /// - `Infrastructure` if the URL is malformed, the
    ///   database file is not writable, or the PRAGMA setup
    ///   fails.
    pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
        let connect_opts = SqliteConnectOptions::from_str(url)
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "invalid sqlite URL {url:?}: {e}"
                )))
            })?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("PRAGMA journal_mode = WAL")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA synchronous = NORMAL")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA foreign_keys = ON")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(connect_opts)
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "sqlite connect {url:?}: {e}"
                )))
            })?;
        debug!(school = %school, url, "opened file-backed sqlite");
        Ok(Self { pool, school })
    }

    /// Returns a borrow of the inner `SqlitePool`. Used by the
    /// adapter, the transaction, and the sub-port impls.
    pub fn db(&self) -> &SqlitePool {
        &self.pool
    }

    /// Returns the school the connection is scoped to.
    pub fn school(&self) -> SchoolId {
        self.school
    }
}
