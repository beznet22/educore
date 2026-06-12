//! MySQL connection management.
//!
//! The `MysqlConnection` wraps a `sqlx::MySqlPool` and the
//! `SchoolId` the adapter is scoped to. The pool is created via
//! `sqlx::mysql::MySqlPoolOptions` and a
//! `sqlx::mysql::MySqlConnectOptions` that carries an
//! `after_connect` hook. The hook issues
//! `SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci` on every new
//! connection so the connection's character set matches the DDL's
//! `utf8mb4_unicode_ci` collation (the engine prefers this over
//! MySQL 8's default `utf8mb4_0900_ai_ci`, which is
//! accent-insensitive; see
//! `docs/schemas/sql-dialects/mysql.md` § "Default settings for
//! every table").
//!
//! ## `multi_statements`
//!
//! The `MySqlConnectOptions` are built with
//! `enable_multi_statements(true)` so the `migrate()` path
//! (which runs the 6-table DDL via `sqlx::raw_sql`) can execute
//! the multi-statement `.sql` file in a single round-trip. The
//! DDL is otherwise plain DDL/DML and does not depend on
//! `multi_statements`; consumers that want a stricter connection
//! profile can use a non-migrating connection at runtime.
//!
//! See `migrations/engine/0000_engine_core.mysql.sql` for the
//! DDL the adapter embeds.

use std::fmt;
use std::str::FromStr;

use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::ids::SchoolId;

/// A handle to a connected MySQL instance. The `MySqlPool` is
/// cheaply cloneable (it is internally an `Arc`); the wrapper
/// derives `Clone` so the storage adapter, the transaction, and
/// the four sub-port handles can each hold their own reference
/// without lifetime gymnastics.
#[derive(Clone)]
pub struct MysqlConnection {
    inner: MySqlPool,
    /// The school the adapter is scoped to. Every cross-cutting
    /// table read/write is filtered by this school; the engine
    /// enforces tenant isolation at the `TenantContext` layer
    /// (see `docs/schemas/tenancy-schema.md`).
    school: SchoolId,
}

impl fmt::Debug for MysqlConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MysqlConnection")
            .field("school", &self.school)
            .finish_non_exhaustive()
    }
}

impl MysqlConnection {
    /// Opens a `sqlx::MySqlPool` against the URL and registers a
    /// per-connection `SET NAMES` to lock in the engine's
    /// preferred character set and collation. The pool's
    /// connection hook is the recommended sqlx pattern for
    /// per-session setup (it runs once per new connection, not
    /// per query).
    ///
    /// The URL must be a syntactically valid `mysql://` URL
    /// (sqlx's `MySqlConnectOptions::from_str` parser is
    /// strict). If the URL does not already include
    /// `multi_statements=true`, this constructor appends it so
    /// the `migrate()` path can run the multi-statement DDL
    /// file in a single round-trip.
    ///
    /// # Errors
    /// - `Infrastructure` if the URL cannot be parsed, the
    ///   connection cannot reach the database server, or the
    ///   per-connection `SET NAMES` fails.
    #[instrument(skip(url), fields(school = %school))]
    pub async fn connect(url: &str, school: SchoolId) -> Result<Self> {
        // sqlx 0.8 does not expose a builder method for
        // `multi_statements`; the only knob is the URL's query
        // string. Append `multi_statements=true` (or
        // `&multi_statements=true`) if the URL doesn't already
        // include it. This is idempotent: a URL that already
        // has the parameter is left alone.
        let url = ensure_multi_statements(url);
        let opts = MySqlConnectOptions::from_str(&url).map_err(|e| {
            DomainError::infrastructure(crate::error::StringError(format!(
                "invalid mysql URL {url:?}: {e}"
            )))
        })?;
        // `sqlx::mysql::MySqlPoolOptions` exposes
        // `after_connect` for per-connection setup. The closure
        // runs once per new connection (not per query) and
        // returns a boxed future. The `Send + 'static` bound on
        // the closure is required because the pool shares the
        // callback across all worker threads.
        let pool = MySqlPoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Lock the connection's character set to the
                    // engine's preferred collation. The DDL
                    // declares every table with
                    // `CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci`;
                    // issuing `SET NAMES` here makes the
                    // connection agree with the DDL at the
                    // client layer too (string literals,
                    // parameter data, error messages). The
                    // canonical DDL is also a no-op on a
                    // re-migrate, so consumers that connect to
                    // an already-migrated database still get
                    // the right character set.
                    sqlx::query::<sqlx::MySql>("SET NAMES utf8mb4 COLLATE utf8mb4_unicode_ci")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(opts)
            .await
            .map_err(|e| {
                DomainError::infrastructure(crate::error::StringError(format!(
                    "mysql connect {url:?}: {e}"
                )))
            })?;
        Ok(Self {
            inner: pool,
            school,
        })
    }

    /// Returns the inner `sqlx::MySqlPool`. Used by the adapter,
    /// the transaction, and the sub-port impls.
    pub fn db(&self) -> &MySqlPool {
        &self.inner
    }

    /// Returns the school the connection is scoped to.
    pub fn school(&self) -> SchoolId {
        self.school
    }

    /// Consumes the connection and returns the inner
    /// `MySqlPool`. Used by `MysqlStorageAdapter::close` and by
    /// the sub-port impls that need owned pool access.
    pub fn into_inner(self) -> MySqlPool {
        self.inner
    }
}

/// Append `multi_statements=true` to a MySQL URL if it is not
/// already present. sqlx 0.8 has no builder method for this
/// option; the only knob is the URL's query string. The
/// function is case-insensitive on the parameter name and
/// idempotent: a URL that already has the parameter (with any
/// value) is returned unchanged.
///
/// We split the URL on `?` so the host / path / fragment portion
/// of the URL is not re-encoded (sqlx's URL parser is strict
/// about percent-encoding, so the simple String approach is
/// more robust than a full `url::Url` parse here).
fn ensure_multi_statements(url: &str) -> String {
    if let Some((head, query)) = url.split_once('?') {
        if query.split('&').any(|kv| {
            kv.split('=')
                .next()
                .is_some_and(|k| k.eq_ignore_ascii_case("multi_statements"))
        }) {
            return url.to_owned();
        }
        format!("{head}?{query}&multi_statements=true")
    } else {
        format!("{url}?multi_statements=true")
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::ensure_multi_statements;

    #[test]
    fn appends_when_no_query_string() {
        assert_eq!(
            ensure_multi_statements("mysql://u:p@h:3306/db"),
            "mysql://u:p@h:3306/db?multi_statements=true",
        );
    }

    #[test]
    fn appends_with_amp_when_query_present() {
        assert_eq!(
            ensure_multi_statements("mysql://u:p@h:3306/db?ssl-mode=required"),
            "mysql://u:p@h:3306/db?ssl-mode=required&multi_statements=true",
        );
    }

    #[test]
    fn idempotent_when_already_present() {
        assert_eq!(
            ensure_multi_statements("mysql://u:p@h:3306/db?multi_statements=true"),
            "mysql://u:p@h:3306/db?multi_statements=true",
        );
        assert_eq!(
            ensure_multi_statements(
                "mysql://u:p@h:3306/db?ssl-mode=required&multi_statements=true"
            ),
            "mysql://u:p@h:3306/db?ssl-mode=required&multi_statements=true",
        );
    }

    #[test]
    fn case_insensitive_on_param_name() {
        assert_eq!(
            ensure_multi_statements("mysql://u:p@h:3306/db?Multi_Statements=true"),
            "mysql://u:p@h:3306/db?Multi_Statements=true",
        );
    }
}
