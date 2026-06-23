//! `StorageAdapter::create_schema` for the SQLite adapter.
//!
//! Walks the macro-emitted [`EntityDescriptor`] AST (introduced
//! in commit `e036f73`, see `educore_core::query::EntityDescriptor`)
//! and emits SQLite-specific DDL:
//!
//! 1. `PRAGMA foreign_keys = ON;` — required for the FK
//!    constraints below to be enforced. (Note: the connection
//!    layer in `crate::connection` also issues this PRAGMA in
//!    `after_connect`; the explicit `PRAGMA` here is
//!    defence-in-depth and keeps the create_schema output
//!    self-contained if a future path bypasses the connection
//!    layer.)
//! 2. The canonical engine DDL for the 6 cross-cutting tables
//!    (`include_str!`'d from
//!    `migrations/engine/0000_engine_core.sqlite.sql`). Those
//!    6 tables — `outbox`, `audit_log`, `idempotency`,
//!    `event_log`, `schema_registry`, `system_user` — are owned
//!    by the engine, not by any domain.
//! 3. For every macro-emitted [`EntityDescriptor`] registered
//!    via [`register`], emit `CREATE TABLE IF NOT EXISTS` with
//!    the descriptor's columns (mapped to SQLite types via
//!    [`map_column_type`]), inline `REFERENCES` clauses for
//!    foreign keys, and `CREATE [UNIQUE] INDEX IF NOT EXISTS`
//!    for each declared index.
//!
//! # RLS
//!
//! RLS policies declared on the descriptor are **silently
//! skipped** with a `// TODO: SQLite RLS` comment in the
//! generated DDL. SQLite has no native row-level security
//! (every connection sees every row); the engine's multi-
//! tenancy guarantee is enforced at the application layer via
//! `TenantContext` predicates (see
//! `docs/schemas/tenancy-schema.md`). A future PR may add an
//! optional `AUTHORIZE` filter via SQLite hooks, but that is
//! out of scope for this PR.
//!
//! # Idempotency
//!
//! Every DDL statement uses `IF NOT EXISTS`, so running this
//! function against an already-migrated database is a no-op.
//! Tests in [`tests`] assert this against an in-memory adapter.
//!
//! # Registry
//!
//! Domain crates call [`register`] at startup to publish their
//! macro-emitted `ENTITY_DESCRIPTOR`. The registry is
//! process-global; the umbrella's `boot` path walks every
//! domain crate's init function once per process lifetime.
//! The underlying storage is a `Mutex<Vec<&'static EntityDescriptor>>`
//! guarded by [`std::sync::OnceLock`] so registration is safe to
//! call concurrently.

use std::sync::{Mutex, OnceLock};

use educore_core::error::{DomainError, Result};
use educore_core::query::{
    ColumnDescriptor, ColumnType, EntityDescriptor, ForeignKeyAction, ForeignKeyDescriptor,
    IndexDescriptor,
};

use crate::error::StringError;
use crate::storage::SqliteStorageAdapter;

/// The canonical SQLite DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time from the engine
/// migration file (per
/// `docs/schemas/sql-dialects/sqlite.md#the-6-engine-cross-cutting-tables--sqlite-ddl`).
const SCHEMA_SQL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.sqlite.sql");

// ============================================================================
// Registry — process-global list of macro-emitted descriptors.
// ============================================================================

/// Process-global registry of [`EntityDescriptor`] values to
/// emit at `create_schema` time. Wrapped in
/// `OnceLock<Mutex<Vec<...>>>` so registration is safe from any
/// thread without external coordination.
static REGISTRY: OnceLock<Mutex<Vec<&'static EntityDescriptor>>> = OnceLock::new();

/// Returns the registry's inner `Mutex`, initialising it on
/// first call.
fn registry() -> &'static Mutex<Vec<&'static EntityDescriptor>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Register a macro-emitted [`EntityDescriptor`] with the
/// SQLite adapter's schema emitter. Idempotent: registering
/// the same pointer twice is a no-op, so the umbrella's boot
/// path can register every domain without coordination.
///
/// Domain crates typically call this from a one-time init
/// function exposed in their `lib.rs`, e.g.:
///
/// ```ignore
/// pub fn register_schema() {
///     educore_storage_sqlite::schema::register(&Student::ENTITY_DESCRIPTOR);
///     educore_storage_sqlite::schema::register(&Class::ENTITY_DESCRIPTOR);
/// }
/// ```
pub fn register(descriptor: &'static EntityDescriptor) {
    let mut guard = match registry().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if !guard.iter().any(|d| std::ptr::eq(*d, descriptor)) {
        guard.push(descriptor);
    }
}

/// Returns a snapshot of the registered descriptors in
/// registration order. The snapshot is a plain `Vec` so the
/// caller does not need to hold the lock across an `await`
/// point (which would risk poisoning across the executor).
fn snapshot() -> Vec<&'static EntityDescriptor> {
    let guard = match registry().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.iter().copied().collect()
}

/// Clears the registry. Intended for unit tests; production
/// code must not call this.
#[cfg(test)]
pub(crate) fn clear_for_tests() {
    let mut guard = match registry().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    guard.clear();
}

/// Registers a descriptor only for the lifetime of the
/// returned guard; on drop, the registry is cleared. Intended
/// for unit tests so one test's registrations do not bleed
/// into another.
///
/// The descriptor is leaked into a `'static` slot via
/// `Box::leak`; the resulting allocation lives for the rest of
/// the process, which is acceptable in tests but must not be
/// used in production paths.
#[cfg(test)]
pub(crate) struct TestRegistryGuard;

#[cfg(test)]
impl Drop for TestRegistryGuard {
    fn drop(&mut self) {
        clear_for_tests();
    }
}

#[cfg(test)]
pub(crate) fn register_for_tests(descriptor: EntityDescriptor) -> TestRegistryGuard {
    let leaked: &'static EntityDescriptor = Box::leak(Box::new(descriptor));
    register(leaked);
    TestRegistryGuard
}

// ============================================================================
// create_schema — the public entry point.
// ============================================================================

/// Creates the SQLite schema for the 6 engine cross-cutting
/// tables and every macro-emitted domain aggregate registered
/// via [`register`].
///
/// Steps:
///
/// 1. `PRAGMA foreign_keys = ON;` — required for the FK
///    constraints emitted below to be enforced.
/// 2. Apply the canonical engine DDL via
///    `migrations/engine/0000_engine_core.sqlite.sql`.
/// 3. For each registered descriptor, emit
///    `CREATE TABLE IF NOT EXISTS <table> (...)` with the
///    descriptor's columns (mapped to SQLite types), primary
///    key, and inline `REFERENCES` clauses for foreign keys.
/// 4. For each `indexes` entry, emit
///    `CREATE [UNIQUE] INDEX IF NOT EXISTS <name> ON <table>
///    (<cols>)`.
///
/// RLS is intentionally not emitted (SQLite has no native RLS;
/// tenancy is enforced at the application layer via
/// `TenantContext`).
///
/// Idempotent: every DDL statement uses `IF NOT EXISTS`, so
/// running this against an already-migrated database is a
/// no-op.
///
/// # Errors
///
/// - `Infrastructure` if any DDL statement fails to execute.
///   The error message is prefixed with the table name (when
///   known) so log readers can pinpoint which aggregate's DDL
///   triggered the failure.
pub async fn create_schema(adapter: &SqliteStorageAdapter) -> Result<()> {
    let pool = adapter.pool();

    // 1. PRAGMA foreign_keys = ON
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .map_err(|e| {
            DomainError::infrastructure(StringError(format!(
                "sqlite create_schema PRAGMA foreign_keys: {e}"
            )))
        })?;

    // 2. Canonical engine DDL (6 cross-cutting tables + their
    //    indexes + the system_user seed row).
    sqlx::raw_sql(SCHEMA_SQL).execute(pool).await.map_err(|e| {
        DomainError::infrastructure(StringError(format!(
            "sqlite create_schema engine cross-cutting: {e}"
        )))
    })?;

    // 3 + 4. Walk registered descriptors.
    for descriptor in snapshot() {
        let ddl = render_ddl(descriptor);
        sqlx::raw_sql(&ddl).execute(pool).await.map_err(|e| {
            DomainError::infrastructure(StringError(format!(
                "sqlite create_schema aggregate `{}`: {e}",
                descriptor.table
            )))
        })?;
    }

    Ok(())
}

// ============================================================================
// DDL rendering — pure functions, exposed for unit tests.
// ============================================================================

/// Renders the full DDL for one [`EntityDescriptor`]: the
/// `CREATE TABLE` body plus one `CREATE INDEX` per declared
/// index. Trailing newline included. Pure: no side effects,
/// no DB access.
pub(crate) fn render_ddl(descriptor: &EntityDescriptor) -> String {
    let mut out = String::with_capacity(256);
    out.push_str(&render_table_ddl(descriptor));
    if !descriptor.rls.is_empty() {
        // TODO: SQLite RLS — see module-level doc-comment.
        // SQLite has no native RLS. The engine's multi-tenancy
        // guarantee is enforced at the application layer via
        // TenantContext predicates. The empty policy list is
        // intentionally tolerated here so a descriptor
        // declaring RLS policies still produces valid DDL on
        // SQLite; the policies are silently dropped.
        let _ = &descriptor.rls;
    }
    for index in &descriptor.indexes {
        out.push('\n');
        out.push_str(&render_index_ddl(descriptor.table, index));
    }
    out
}

/// Renders the `CREATE TABLE IF NOT EXISTS <table>` DDL for
/// one [`EntityDescriptor`], with inline `REFERENCES` clauses
/// for foreign keys and a `PRIMARY KEY (...)` table constraint
/// when the PK spans multiple columns. Pure.
pub(crate) fn render_table_ddl(descriptor: &EntityDescriptor) -> String {
    let mut sql = String::new();
    sql.push_str("CREATE TABLE IF NOT EXISTS ");
    sql.push_str(descriptor.table);
    sql.push_str(" (\n");

    // Decide whether the PK is multi-column. If yes, the
    // table-level `PRIMARY KEY (...)` constraint captures it
    // and individual columns must NOT carry an inline
    // `PRIMARY KEY` clause (SQLite forbids two PKs on one
    // table). If no, single-column PKs are emitted inline.
    let pk_columns: Vec<&str> = descriptor
        .columns
        .iter()
        .filter(|c| c.primary_key)
        .map(|c| c.name)
        .collect();
    let multi_column_pk = pk_columns.len() > 1;

    let mut lines: Vec<String> = Vec::with_capacity(
        descriptor.columns.len() + descriptor.foreign_keys.len() + 1,
    );
    for col in &descriptor.columns {
        lines.push(render_column_line(col, multi_column_pk));
    }
    for fk in &descriptor.foreign_keys {
        lines.push(render_inline_fk(fk));
    }

    // Multi-column primary key: emit as a table-level
    // constraint. Single-column PKs are already declared
    // inline via `render_column_line`.
    if multi_column_pk {
        lines.push(format!("    PRIMARY KEY ({})", pk_columns.join(", ")));
    }

    sql.push_str(&lines.join(",\n"));
    sql.push_str("\n);\n");
    sql
}

/// Renders one column line for the `CREATE TABLE` body.
/// Example: `    id TEXT NOT NULL PRIMARY KEY`.
///
/// When `multi_column_pk` is true the column-level
/// `PRIMARY KEY` clause is suppressed even if `col.primary_key`
/// is set, because the table-level `PRIMARY KEY (...)`
/// constraint emitted by [`render_table_ddl`] captures the
/// full PK and SQLite forbids two PK definitions on one table.
fn render_column_line(col: &ColumnDescriptor, multi_column_pk: bool) -> String {
    let mut line = format!("    {} {}", col.name, map_column_type(&col.column_type));
    if !col.nullable {
        line.push_str(" NOT NULL");
    }
    if col.primary_key && !multi_column_pk {
        line.push_str(" PRIMARY KEY");
    }
    if col.unique && !col.primary_key {
        line.push_str(" UNIQUE");
    }
    line
}

/// Maps a dialect-agnostic [`ColumnType`] to the SQLite type
/// affinity that the engine's data layer will rely on.
///
/// SQLite uses **type affinity**, not strict types (per
/// [`docs/schemas/sql-dialects/sqlite.md`]); the application
/// layer is responsible for type coercion on read/write. The
/// mapping below matches the engine's SQLite dialect
/// conventions documented there.
///
/// `Custom("UNKNOWN")` (the placeholder emitted by the macro
/// in `e036f73` until type inference lands) falls back to
/// `TEXT` so the generated DDL is valid SQLite even before
/// the descriptor is fully populated.
pub(crate) fn map_column_type(col_type: &ColumnType) -> &'static str {
    match col_type {
        ColumnType::Uuid => "TEXT",
        ColumnType::String => "TEXT",
        ColumnType::Text => "TEXT",
        ColumnType::I64 => "INTEGER",
        ColumnType::U64 => "INTEGER",
        ColumnType::I32 => "INTEGER",
        ColumnType::U32 => "INTEGER",
        ColumnType::F64 => "REAL",
        ColumnType::Bool => "INTEGER",
        ColumnType::Timestamp => "TEXT",
        ColumnType::Json => "TEXT",
        ColumnType::Bytes => "BLOB",
        ColumnType::Custom(s) => {
            // The macro emits `Custom("UNKNOWN")` as a
            // placeholder until type inference lands (Cluster A
            // stage 2 follow-up). Fall back to TEXT so the
            // generated DDL is valid SQLite in the meantime.
            if *s == "UNKNOWN" {
                "TEXT"
            } else {
                s
            }
        }
    }
}

/// Renders an inline foreign-key clause for the `CREATE
/// TABLE` body. Example:
/// `    FOREIGN KEY (school_id) REFERENCES schools(id) ON DELETE RESTRICT ON UPDATE NO ACTION`.
fn render_inline_fk(fk: &ForeignKeyDescriptor) -> String {
    format!(
        "    FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE {} ON UPDATE {}",
        fk.column,
        fk.references_table,
        fk.references_column,
        fk_action_sql(&fk.on_delete),
        fk_action_sql(&fk.on_update),
    )
}

/// Maps a [`ForeignKeyAction`] to its SQL keyword (with the
/// space for multi-word actions like `NO ACTION`).
fn fk_action_sql(action: &ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "NO ACTION",
        ForeignKeyAction::Restrict => "RESTRICT",
        ForeignKeyAction::Cascade => "CASCADE",
        ForeignKeyAction::SetNull => "SET NULL",
        ForeignKeyAction::SetDefault => "SET DEFAULT",
    }
}

/// Renders the `CREATE [UNIQUE] INDEX IF NOT EXISTS <name>
/// ON <table> (<cols>);` DDL for one [`IndexDescriptor`].
fn render_index_ddl(table: &str, index: &IndexDescriptor) -> String {
    let unique = if index.unique { "UNIQUE " } else { "" };
    format!(
        "CREATE {}INDEX IF NOT EXISTS {} ON {} ({});\n",
        unique,
        index.name,
        table,
        index.columns.join(", "),
    )
}

// ============================================================================
// Tests.
// ============================================================================

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::print_stderr
)]
mod tests {
    use super::*;
    use educore_core::clock::{IdGenerator as _, SystemIdGen};
    use educore_core::ids::SchoolId;
    use educore_core::query::{ColumnDescriptor, ColumnType, EntityDescriptor, IndexDescriptor};

    /// A representative descriptor covering every column-type
    /// variant. Used by the type-mapping and FK tests.
    fn full_descriptor() -> EntityDescriptor {
        EntityDescriptor {
            table: "full_sample",
            columns: vec![
                ColumnDescriptor {
                    name: "id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: true,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "name",
                    column_type: ColumnType::String,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: true,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "notes",
                    column_type: ColumnType::Text,
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "count",
                    column_type: ColumnType::I64,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "amount",
                    column_type: ColumnType::F64,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "active",
                    column_type: ColumnType::Bool,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "created_at",
                    column_type: ColumnType::Timestamp,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "metadata",
                    column_type: ColumnType::Json,
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "blob",
                    column_type: ColumnType::Bytes,
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "placeholder",
                    column_type: ColumnType::Custom("UNKNOWN"),
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "tag",
                    column_type: ColumnType::Custom("VARCHAR(16)"),
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
            ],
            indexes: vec![
                IndexDescriptor {
                    name: "full_sample_school_idx",
                    columns: vec!["name"],
                    unique: false,
                },
                IndexDescriptor {
                    name: "full_sample_count_uq",
                    columns: vec!["count"],
                    unique: true,
                },
            ],
            foreign_keys: vec![ForeignKeyDescriptor {
                column: "name",
                references_table: "schools",
                references_column: "id",
                on_delete: ForeignKeyAction::Restrict,
                on_update: ForeignKeyAction::NoAction,
            }],
            rls: vec![],
        }
    }

    #[test]
    fn map_column_type_uses_engine_conventions() {
        assert_eq!(map_column_type(&ColumnType::Uuid), "TEXT");
        assert_eq!(map_column_type(&ColumnType::String), "TEXT");
        assert_eq!(map_column_type(&ColumnType::Text), "TEXT");
        assert_eq!(map_column_type(&ColumnType::I64), "INTEGER");
        assert_eq!(map_column_type(&ColumnType::U64), "INTEGER");
        assert_eq!(map_column_type(&ColumnType::I32), "INTEGER");
        assert_eq!(map_column_type(&ColumnType::U32), "INTEGER");
        assert_eq!(map_column_type(&ColumnType::F64), "REAL");
        assert_eq!(map_column_type(&ColumnType::Bool), "INTEGER");
        assert_eq!(map_column_type(&ColumnType::Timestamp), "TEXT");
        assert_eq!(map_column_type(&ColumnType::Json), "TEXT");
        assert_eq!(map_column_type(&ColumnType::Bytes), "BLOB");
        // Unknown Custom falls back to TEXT.
        assert_eq!(map_column_type(&ColumnType::Custom("UNKNOWN")), "TEXT");
        // Known Custom passes through verbatim.
        assert_eq!(
            map_column_type(&ColumnType::Custom("VARCHAR(16)")),
            "VARCHAR(16)"
        );
        assert_eq!(map_column_type(&ColumnType::Custom("DECIMAL")), "DECIMAL");
    }

    #[test]
    fn render_table_ddl_emits_columns_and_pk_and_fk() {
        let d = full_descriptor();
        let sql = render_table_ddl(&d);

        // Header
        assert!(
            sql.starts_with("CREATE TABLE IF NOT EXISTS full_sample ("),
            "expected CREATE TABLE header, got: {sql}",
        );

        // Each column maps to the right SQLite type.
        assert!(sql.contains("id TEXT NOT NULL PRIMARY KEY"), "id: {sql}");
        assert!(sql.contains("name TEXT NOT NULL"), "name: {sql}");
        assert!(sql.contains("notes TEXT"), "notes: {sql}");
        assert!(sql.contains("count INTEGER NOT NULL"), "count: {sql}");
        assert!(sql.contains("amount REAL NOT NULL"), "amount: {sql}");
        assert!(sql.contains("active INTEGER NOT NULL"), "active: {sql}");
        assert!(
            sql.contains("created_at TEXT NOT NULL"),
            "created_at: {sql}",
        );
        assert!(sql.contains("metadata TEXT"), "metadata: {sql}");
        assert!(sql.contains("blob BLOB"), "blob: {sql}");

        // Custom("UNKNOWN") → TEXT fallback.
        assert!(sql.contains("placeholder TEXT"), "placeholder: {sql}");

        // Custom known types pass through verbatim.
        assert!(sql.contains("tag VARCHAR(16) NOT NULL"), "tag: {sql}");

        // Foreign key clause (inline REFERENCES, SQL convention).
        assert!(
            sql.contains(
                "FOREIGN KEY (name) REFERENCES schools(id) ON DELETE RESTRICT ON UPDATE NO ACTION"
            ),
            "FK clause: {sql}",
        );

        // Indexes appear in render_ddl but not in render_table_ddl.
        assert!(!sql.contains("CREATE INDEX"), "FK clause: {sql}");
    }

    #[test]
    fn render_ddl_emits_indexes_after_table() {
        let d = full_descriptor();
        let sql = render_ddl(&d);

        // Non-unique index.
        assert!(
            sql.contains("CREATE INDEX IF NOT EXISTS full_sample_school_idx ON full_sample (name)"),
            "non-unique index: {sql}",
        );
        // Unique index.
        assert!(
            sql.contains(
                "CREATE UNIQUE INDEX IF NOT EXISTS full_sample_count_uq ON full_sample (count)",
            ),
            "unique index: {sql}",
        );
        // CREATE TABLE appears before the indexes (table first).
        let table_pos = sql.find("CREATE TABLE").expect("table stmt");
        let index_pos = sql.find("CREATE INDEX").expect("index stmt");
        assert!(
            table_pos < index_pos,
            "table must precede indexes; sql={sql}",
        );
    }

    #[test]
    fn multi_column_pk_renders_table_level_constraint() {
        let d = EntityDescriptor {
            table: "join_table",
            columns: vec![
                ColumnDescriptor {
                    name: "left_id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "right_id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
            ],
            indexes: vec![],
            foreign_keys: vec![],
            rls: vec![],
        };
        let sql = render_table_ddl(&d);
        // Neither column should carry an inline PRIMARY KEY
        // (the column line would look like `left_id TEXT NOT NULL PRIMARY KEY`).
        assert!(
            !sql.contains("NOT NULL PRIMARY KEY"),
            "no inline PRIMARY KEY on columns: {sql}",
        );
        // The table-level constraint captures both columns.
        assert!(
            sql.contains("PRIMARY KEY (left_id, right_id)"),
            "table pk: {sql}",
        );
    }

    // ----------------------------------------------------------------
    // Live-DB tests (require sqlite feature). These run against an
    // in-memory adapter created in the calling test function.
    // ----------------------------------------------------------------

    /// Returns an in-memory adapter scoped to a fresh school.
    async fn in_memory_adapter() -> SqliteStorageAdapter {
        let g = SystemIdGen;
        let school: SchoolId = g.next_school_id();
        SqliteStorageAdapter::in_memory(school)
            .await
            .expect("in_memory adapter")
    }

    /// Returns the SQL `sqlite_master` entry for a table, if any.
    async fn master_entry(pool: &sqlx::SqlitePool, table: &str) -> Option<String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?1",
        )
        .bind(table)
        .fetch_optional(pool)
        .await
        .expect("sqlite_master query");
        row.map(|(sql,)| sql)
    }

    #[tokio::test]
    async fn create_schema_emits_create_table_for_each_registered_aggregate() {
        let _guard = register_for_tests(full_descriptor());
        let adapter = in_memory_adapter().await;

        // Before create_schema: full_sample must not exist.
        let before = master_entry(adapter.pool(), "full_sample").await;
        assert!(
            before.is_none(),
            "full_sample unexpectedly exists before create_schema: {before:?}",
        );

        // Run create_schema.
        create_schema(&adapter).await.expect("create_schema");

        // After create_schema: full_sample exists, with every column.
        let after = master_entry(adapter.pool(), "full_sample")
            .await
            .expect("full_sample must exist after create_schema");
        for column in [
            "id",
            "name",
            "notes",
            "count",
            "amount",
            "active",
            "created_at",
            "metadata",
            "blob",
            "placeholder",
            "tag",
        ] {
            assert!(
                after.contains(column),
                "full_sample CREATE TABLE missing column `{column}`: {after}",
            );
        }
        // FK clause survived the round trip.
        assert!(
            after.contains("REFERENCES schools(id)"),
            "full_sample FK clause missing: {after}",
        );
        // Primary key survived the round trip.
        assert!(
            after.contains("PRIMARY KEY"),
            "full_sample PRIMARY KEY missing: {after}",
        );

        // Indexes survived the round trip.
        let idx_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND tbl_name = 'full_sample'",
        )
        .fetch_one(adapter.pool())
        .await
        .expect("index count");
        // 1 sqlite-internal auto-index for the PK + 2 declared indexes.
        assert!(
            idx_count.0 >= 3,
            "expected >=3 indexes on full_sample, got {}",
            idx_count.0,
        );
    }

    #[tokio::test]
    async fn create_schema_handles_custom_unknown_by_emitting_text() {
        let _guard = register_for_tests(full_descriptor());
        let adapter = in_memory_adapter().await;
        create_schema(&adapter).await.expect("create_schema");

        // The `placeholder` column (Custom("UNKNOWN")) must be
        // stored as TEXT. We assert via a round-trip: insert a
        // TEXT value into the column and read it back. If the
        // column had been declared as anything else (e.g. INTEGER),
        // the INSERT would still succeed because SQLite uses type
        // affinity, so the more durable assertion is on the
        // sqlite_master entry — which preserves the declared type.
        let master = master_entry(adapter.pool(), "full_sample")
            .await
            .expect("full_sample");
        assert!(
            master.contains("placeholder TEXT"),
            "placeholder column must be declared TEXT (sqlite_master): {master}",
        );
    }

    #[tokio::test]
    async fn create_schema_is_idempotent() {
        let _guard = register_for_tests(full_descriptor());
        let adapter = in_memory_adapter().await;

        // First run creates everything.
        create_schema(&adapter).await.expect("first create_schema");
        // Second run must succeed (no-op via IF NOT EXISTS).
        create_schema(&adapter)
            .await
            .expect("second create_schema (idempotent)");
        // Third run for good measure — also a no-op.
        create_schema(&adapter)
            .await
            .expect("third create_schema (idempotent)");

        // The schema is unchanged: still exactly one row in
        // sqlite_master for full_sample.
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'full_sample'",
        )
        .fetch_one(adapter.pool())
        .await
        .expect("master count");
        assert_eq!(count.0, 1, "full_sample must exist exactly once");
    }
}
