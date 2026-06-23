//! Runtime schema emission for the PostgreSQL adapter.
//!
//! Cluster A stage 3 (postgres): this module implements
//! [`create_schema`], the consumer-facing entry point that
//! emits Postgres-specific DDL for the 6 engine cross-cutting
//! tables and for every aggregate registered with the engine.
//!
//! ## What `create_schema` does
//!
//! 1. Emits the canonical DDL for the 6 engine cross-cutting
//!    tables (`outbox`, `audit_log`, `idempotency`, `event_log`,
//!    `schema_registry`, `system_user`) from the
//!    `include_str!`'d `migrations/engine/0000_engine_core.postgres.sql`
//!    file.
//! 2. Walks the registered domain aggregates
//!    ([`registered_aggregates`]) and emits per-aggregate DDL
//!    (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`,
//!    `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ...`,
//!    and RLS policies).
//!
//! Every emitted statement is idempotent (`IF NOT EXISTS`,
//! `IF NOT EXISTS`, or `DO $$ BEGIN ... EXCEPTION WHEN
//! duplicate_object THEN NULL; END $$;` for constraints).
//!
//! ## Mapping `ColumnType` to Postgres
//!
//! The macro-emitted `ColumnDescriptor::column_type` is
//! dialect-agnostic. The adapter maps each variant to the
//! Postgres native type via [`column_type_to_pg_sql`]:
//!
//! | `ColumnType`     | Postgres type     |
//! | ---------------- | ----------------- |
//! | `Uuid`           | `UUID`            |
//! | `String`         | `VARCHAR(255)`    |
//! | `Text`           | `TEXT`            |
//! | `I64` / `U64`    | `BIGINT`          |
//! | `I32` / `U32`    | `INTEGER`         |
//! | `F64`            | `DOUBLE PRECISION`|
//! | `Bool`           | `BOOLEAN`         |
//! | `Timestamp`      | `TIMESTAMPTZ`     |
//! | `Json`           | `JSONB`           |
//! | `Bytes`          | `BYTEA`           |
//! | `Custom(s)`      | `s` (verbatim)    |
//!
//! The macro currently emits `Custom("UNKNOWN")` as a
//! placeholder until type inference lands (Cluster A stage 2
//! follow-up). Per the design constraint, this fallback is
//! mapped to `TEXT` rather than `UNKNOWN` so the resulting
//! DDL is valid against a live PostgreSQL instance.
//!
//! ## Aggregate registry
//!
//! The registry returned by [`registered_aggregates`] is a
//! `&'static [EntityDescriptor]`. It is intentionally empty
//! in this commit: no domain crate has implemented
//! `#[derive(DomainQuery)]` and produced an
//! `ENTITY_DESCRIPTOR` yet (per `AGENTS.md` § "Status", domain
//! implementation begins in Phase 3+). A follow-up PR will
//! register each domain aggregate via the umbrella crate's
//! `inventory` pattern once the descriptors are emitted.
//!
//! Tests in this module construct `EntityDescriptor` values
//! directly and pass them to [`build_aggregate_ddl`] to verify
//! the emitted DDL without needing a live database.
//!
//! ## Why not also drive RLS here?
//!
//! Per `docs/schemas/sql-dialects/postgresql.md:122-159`, the
//! adapter must `SET LOCAL app.current_school_id = ?` on every
//! new transaction so RLS policies can resolve the tenant. The
//! transaction hook (ADAPTER-PG-003) is a separate concern
//! and is intentionally out of scope for this commit. RLS
//! policy emission itself is in scope and is implemented
//! below; the policies use `current_setting('app.current_school_id')::UUID`
//! as the `USING` and `WITH CHECK` predicate so they pair
//! with that hook.

use std::fmt::Write as _;

use educore_core::error::{DomainError, Result};
use educore_core::query::{
    ColumnType, EntityDescriptor, ForeignKeyAction, ForeignKeyDescriptor, IndexDescriptor,
    RlsPolicy,
};

use crate::storage::PostgresStorageAdapter;

/// The canonical PostgreSQL DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time from
/// `migrations/engine/0000_engine_core.postgres.sql`. The DDL
/// is idempotent (`CREATE SCHEMA IF NOT EXISTS`,
/// `CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`)
/// so a re-run is a no-op.
const CROSS_CUTTING_SQL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.postgres.sql");

/// Returns the list of aggregate `EntityDescriptor`s the
/// adapter will emit DDL for at `create_schema()` time.
///
/// As of this commit, the registry is empty: no domain crate
/// has implemented `#[derive(DomainQuery)]` and produced an
/// `ENTITY_DESCRIPTOR` yet. A follow-up PR will wire each
/// domain's `ENTITY_DESCRIPTOR` into this slice (or a
/// `inventory::collect!`-backed registry).
///
/// The function is `const` so the slice lives in `.rodata`
/// and is reachable from `no_std`-friendly callers in the
/// future (the engine's umbrella crate may compile to
/// `no_std`+alloc for embedded targets per ADR-015).
#[must_use]
pub const fn registered_aggregates() -> &'static [&'static EntityDescriptor] {
    const EMPTY: &[&EntityDescriptor] = &[];
    EMPTY
}

/// Runtime schema entry point. Emits the 6 engine cross-
/// cutting tables and walks every registered aggregate's
/// `EntityDescriptor`, emitting `CREATE TABLE`, `CREATE INDEX`,
/// `ALTER TABLE ... ADD CONSTRAINT`, and row-level security
/// policy DDL. All statements are idempotent; running this
/// twice against the same database is a no-op.
///
/// This is the runtime consumer-facing entry point referenced
/// by `docs/build-plan.md`, `AGENTS.md`, and the dialect
/// specs; the existing [`migrate`](educore_storage::port::StorageAdapter::migrate)
/// method on `StorageAdapter` continues to work as the
/// port-trait entry point and delegates here for the
/// aggregate DDL portion.
///
/// The aggregate list is sourced from [`registered_aggregates`];
/// tests that need to inject fixtures should call
/// [`create_schema_for`] directly.
///
/// # Errors
/// - `Infrastructure` if the underlying `sqlx` driver fails
///   to apply any of the emitted statements.
#[tracing::instrument(skip(adapter))]
pub async fn create_schema(adapter: &PostgresStorageAdapter) -> Result<()> {
    create_schema_for(adapter, registered_aggregates()).await
}

/// Same as [`create_schema`], but accepts the aggregate list
/// directly. Used by the trait method override on
/// `PostgresStorageAdapter` (which receives the descriptor
/// slice from the consumer) and by tests that need to inject
/// fixtures without touching the global [`registered_aggregates`]
/// slice.
///
/// The two SQL streams are concatenated into a single
/// `sqlx::raw_sql` call so PostgreSQL parses the entire
/// script in one round-trip.
#[tracing::instrument(skip(adapter, aggregates))]
pub async fn create_schema_for(
    adapter: &PostgresStorageAdapter,
    aggregates: &[&'static EntityDescriptor],
) -> Result<()> {
    let sql = build_full_ddl(CROSS_CUTTING_SQL, aggregates);
    sqlx::raw_sql(&sql)
        .execute(adapter.db())
        .await
        .map_err(DomainError::infrastructure)?;
    Ok(())
}

/// Builds the full DDL script: the 6 cross-cutting tables
/// followed by every registered aggregate's per-aggregate DDL
/// (`CREATE TABLE` + indexes + FKs + RLS). Exposed at module
/// scope so tests can verify the script string without a live
/// database.
#[must_use]
pub fn build_full_ddl(cross_cutting: &str, aggregates: &[&'static EntityDescriptor]) -> String {
    // Size hint: cross-cutting (~5 KB) + ~1 KB per aggregate
    // (table + indexes + FKs + RLS). Pre-sizing avoids the
    // String's geometric re-allocation.
    let mut buf = String::with_capacity(cross_cutting.len() + aggregates.len() * 1024);
    buf.push_str(cross_cutting);
    if !cross_cutting.ends_with('\n') {
        buf.push('\n');
    }
    for agg in aggregates {
        buf.push_str(&build_aggregate_ddl(agg));
    }
    buf
}

/// Builds the per-aggregate DDL: `CREATE TABLE` (with engine
/// invariants columns), `CREATE INDEX` for every entry in
/// `indexes`, `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY`
/// for every entry in `foreign_keys`, and the row-level
/// security policy statements for every entry in `rls`.
///
/// Foreign keys are emitted after the `CREATE TABLE` block
/// because Postgres requires the referenced table to exist
/// when adding a constraint. The script assumes the
/// referenced tables are created in the same `create_schema`
/// run (the iteration order over `aggregates` is the caller's
/// responsibility; the adapter currently passes
/// `registered_aggregates()` which is sorted at registration
/// time by `table` to keep referential order deterministic).
///
/// RLS statements are wrapped in a `DO $$ ... EXCEPTION WHEN
/// duplicate_object THEN NULL; END $$;` block so re-runs are
/// idempotent: the second run sees the policy already exists
/// and swallows the error. This matches the dialect spec's
/// idempotency requirement.
#[must_use]
pub fn build_aggregate_ddl(aggregate: &EntityDescriptor) -> String {
    let mut buf = String::with_capacity(512);
    buf.push_str(&build_create_table_sql(aggregate));
    for idx in &aggregate.indexes {
        buf.push_str(&build_index_sql(idx, aggregate.table));
    }
    for fk in &aggregate.foreign_keys {
        buf.push_str(&build_foreign_key_sql(fk, aggregate.table));
    }
    for policy in &aggregate.rls {
        buf.push_str(&build_rls_policy_sql(policy, aggregate.table));
    }
    buf
}

/// Emits a `CREATE TABLE IF NOT EXISTS <table> (...)` statement
/// for the given aggregate. Columns are emitted in declaration
/// order; the engine invariants (`id`, `school_id`, `created_at`,
/// `updated_at`, `created_by`, `updated_by`, `version`, `etag`)
/// are expected to already be present in
/// `aggregate.columns` (the macro emits them; this function
/// does not synthesize them).
///
/// Primary-key columns are collected and emitted as a trailing
/// `PRIMARY KEY (...)` clause. A column marked `primary_key =
/// true` and `auto_generated = true` is also annotated with
/// `DEFAULT gen_random_uuid()` so inserts that omit the
/// column still satisfy the constraint (the engine otherwise
/// always supplies the id, but the default is a safety net
/// for ad-hoc SQL).
#[must_use]
pub fn build_create_table_sql(aggregate: &EntityDescriptor) -> String {
    let mut buf = String::with_capacity(256);
    let _ = writeln!(
        buf,
        "CREATE TABLE IF NOT EXISTS {table} (",
        table = aggregate.table
    );
    let mut pk_columns: Vec<&str> = Vec::with_capacity(1);
    let mut first = true;
    for col in &aggregate.columns {
        if !first {
            buf.push_str(",\n");
        }
        first = false;
        buf.push_str("    ");
        buf.push_str(col.name);
        buf.push(' ');
        buf.push_str(column_type_to_pg_sql(&col.column_type));
        if !col.nullable {
            buf.push_str(" NOT NULL");
        }
        if col.unique && !col.primary_key {
            buf.push_str(" UNIQUE");
        }
        if col.primary_key {
            pk_columns.push(col.name);
        }
        if col.auto_generated && matches!(col.column_type, ColumnType::Uuid) {
            buf.push_str(" DEFAULT gen_random_uuid()");
        }
    }
    if !pk_columns.is_empty() {
        buf.push_str(",\n    PRIMARY KEY (");
        for (i, name) in pk_columns.iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            buf.push_str(name);
        }
        buf.push(')');
    }
    buf.push_str("\n);\n");
    buf
}

/// Emits a `CREATE INDEX IF NOT EXISTS <name> ON <table> (<cols>)`
/// statement. `unique = true` adds the `UNIQUE` keyword.
///
/// Column names are joined with `, `. Per the engine's
/// "Parameterized SQL only (no string interpolation)" rule,
/// the index name, table name, and column names are spliced
/// into the SQL verbatim because they originate from the
/// compile-time `&'static str` AST — there is no user-supplied
/// runtime input on this code path.
#[must_use]
pub fn build_index_sql(index: &IndexDescriptor, table: &str) -> String {
    let mut buf = String::with_capacity(64 + index.name.len() + table.len());
    if index.unique {
        buf.push_str("CREATE UNIQUE INDEX IF NOT EXISTS ");
    } else {
        buf.push_str("CREATE INDEX IF NOT EXISTS ");
    }
    buf.push_str(index.name);
    buf.push_str(" ON ");
    buf.push_str(table);
    buf.push_str(" (");
    for (i, col) in index.columns.iter().enumerate() {
        if i > 0 {
            buf.push_str(", ");
        }
        buf.push_str(col);
    }
    buf.push_str(");\n");
    buf
}

/// Emits an `ALTER TABLE <table> ADD CONSTRAINT IF NOT EXISTS`
/// is NOT supported by Postgres (the `IF NOT EXISTS` clause
/// is only valid for `CREATE TABLE` / `CREATE INDEX`). We
/// instead use the standard Postgres pattern of a `DO $$
/// ... EXCEPTION WHEN duplicate_object THEN NULL; END $$;`
/// block. This swallows the duplicate-constraint error so a
/// re-run is a no-op.
#[must_use]
pub fn build_foreign_key_sql(fk: &ForeignKeyDescriptor, table: &str) -> String {
    let constraint_name = format!(
        "{table}_{col}_fk",
        table = table,
        col = fk.column
    );
    let on_delete = foreign_key_action_sql(&fk.on_delete);
    let on_update = foreign_key_action_sql(&fk.on_update);
    format!(
        "DO $$ BEGIN\n\
         \x20\x20\x20\x20ALTER TABLE {table} ADD CONSTRAINT {constraint_name}\n\
         \x20\x20\x20\x20\x20\x20\x20\x20FOREIGN KEY ({col}) REFERENCES {references_table}({references_column})\n\
         \x20\x20\x20\x20\x20\x20\x20\x20ON DELETE {on_delete} ON UPDATE {on_update};\n\
         EXCEPTION WHEN duplicate_object THEN NULL;\n\
         END $$;\n",
        table = table,
        constraint_name = constraint_name,
        col = fk.column,
        references_table = fk.references_table,
        references_column = fk.references_column,
        on_delete = on_delete,
        on_update = on_update,
    )
}

/// Emits the row-level security policy DDL for one policy:
/// `ALTER TABLE <table> ENABLE ROW LEVEL SECURITY`,
/// `ALTER TABLE <table> FORCE ROW LEVEL SECURITY`, and
/// `CREATE POLICY <name> ON <table> USING (...) [WITH CHECK (...)]`.
///
/// Postgres's `CREATE POLICY` does not support `IF NOT
/// EXISTS`, so the whole block is wrapped in a `DO $$ ...
/// EXCEPTION WHEN duplicate_object THEN NULL; END $$;`
/// construct, matching the dialect spec's idempotency
/// requirement.
#[must_use]
pub fn build_rls_policy_sql(policy: &RlsPolicy, table: &str) -> String {
    let with_check = policy.with_check_expr.unwrap_or(policy.using_expr);
    format!(
        "DO $$ BEGIN\n\
         \x20\x20\x20\x20ALTER TABLE {table} ENABLE ROW LEVEL SECURITY;\n\
         \x20\x20\x20\x20ALTER TABLE {table} FORCE ROW LEVEL SECURITY;\n\
         \x20\x20\x20\x20CREATE POLICY {name} ON {table}\n\
         \x20\x20\x20\x20\x20\x20\x20\x20USING ({using_expr})\n\
         \x20\x20\x20\x20\x20\x20\x20\x20WITH CHECK ({with_check});\n\
         EXCEPTION WHEN duplicate_object THEN NULL;\n\
         END $$;\n",
        name = policy.name,
        using_expr = policy.using_expr,
        with_check = with_check,
    )
}

/// Maps the dialect-agnostic [`ColumnType`] to the
/// corresponding Postgres native type.
///
/// `ColumnType::Custom(s)` is emitted verbatim — the macro
/// may carry a fully-qualified Postgres type for adapters
/// that need a non-default mapping (e.g. `CITEXT`,
/// `JSONB PATH`). Per the design constraint, this is the
/// escape hatch until type inference lands.
///
/// The "UNKNOWN" placeholder the macro emits today is mapped
/// to `TEXT` so the resulting DDL is valid against a live
/// PostgreSQL instance. Once type inference lands, the
/// macro will emit real `ColumnType` variants and this
/// branch becomes a no-op for the engine-emitted descriptors.
///
/// Not `const fn` because `PartialEq` on `&'static str` is
/// not yet const-stable (Rust 1.75).
#[must_use]
pub fn column_type_to_pg_sql(ty: &ColumnType) -> &'static str {
    match ty {
        ColumnType::Uuid => "UUID",
        ColumnType::String => "VARCHAR(255)",
        ColumnType::Text => "TEXT",
        ColumnType::I64 | ColumnType::U64 => "BIGINT",
        ColumnType::I32 | ColumnType::U32 => "INTEGER",
        ColumnType::F64 => "DOUBLE PRECISION",
        ColumnType::Bool => "BOOLEAN",
        ColumnType::Timestamp => "TIMESTAMPTZ",
        ColumnType::Json => "JSONB",
        ColumnType::Bytes => "BYTEA",
        // The macro currently emits `Custom("UNKNOWN")` as a
        // placeholder. Treat it as TEXT so the DDL is valid;
        // type inference will replace it with the real type.
        // Any other custom string is emitted verbatim so the
        // adapter-specific escape hatch works.
        ColumnType::Custom(s) => {
            // The macro currently emits `Custom("UNKNOWN")` as a
            // placeholder. Treat it as TEXT so the DDL is valid;
            // type inference will replace it with the real type.
            // Any other custom string is emitted verbatim so the
            // adapter-specific escape hatch works.
            if *s == "UNKNOWN" {
                "TEXT"
            } else {
                // Dereference the `&&'static str` to satisfy
                // the return type; clippy::explicit_auto_deref
                // is suppressed because the deref is required
                // for the type to line up.
                #[allow(clippy::explicit_auto_deref)]
                {
                    *s
                }
            }
        }
    }
}

/// Maps a [`ForeignKeyAction`] to the corresponding SQL keyword.
#[must_use]
pub const fn foreign_key_action_sql(action: &ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "NO ACTION",
        ForeignKeyAction::Restrict => "RESTRICT",
        ForeignKeyAction::Cascade => "CASCADE",
        ForeignKeyAction::SetNull => "SET NULL",
        ForeignKeyAction::SetDefault => "SET DEFAULT",
    }
}

// ============================================================================
// Tests
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
    //! Tests for the schema module.
    //!
    //! These tests construct `EntityDescriptor` fixtures by
    //! hand and exercise the SQL-building helpers. They do
    //! not need a live database — the runtime behavior is
    //! verified by the e2e test in
    //! `tests/create_schema_e2e.rs` (gated on `EDUCORE_PG_URL`,
    //! matching the convention in `tests/outbox_e2e.rs`).

    use super::*;
    use educore_core::query::ColumnDescriptor;

    /// Helper: build a minimal `EntityDescriptor` from a
    /// table name and a list of columns. Defaults every other
    /// field to its zero value (no indexes, no FKs, no RLS).
    fn fixture(table: &'static str, columns: Vec<ColumnDescriptor>) -> EntityDescriptor {
        EntityDescriptor {
            table,
            columns,
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            rls: Vec::new(),
        }
    }

    /// Helper: build a `ColumnDescriptor` with default
    /// nullability / indexing flags.
    fn col(name: &'static str, ty: ColumnType) -> ColumnDescriptor {
        ColumnDescriptor {
            name,
            column_type: ty,
            nullable: true,
            primary_key: false,
            auto_generated: false,
            indexed: false,
            unique: false,
        }
    }

    // -- ColumnType mapping --------------------------------------------------

    #[test]
    fn column_type_maps_known_variants_to_pg_types() {
        assert_eq!(column_type_to_pg_sql(&ColumnType::Uuid), "UUID");
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::String),
            "VARCHAR(255)"
        );
        assert_eq!(column_type_to_pg_sql(&ColumnType::Text), "TEXT");
        assert_eq!(column_type_to_pg_sql(&ColumnType::I64), "BIGINT");
        assert_eq!(column_type_to_pg_sql(&ColumnType::U64), "BIGINT");
        assert_eq!(column_type_to_pg_sql(&ColumnType::I32), "INTEGER");
        assert_eq!(column_type_to_pg_sql(&ColumnType::U32), "INTEGER");
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::F64),
            "DOUBLE PRECISION"
        );
        assert_eq!(column_type_to_pg_sql(&ColumnType::Bool), "BOOLEAN");
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::Timestamp),
            "TIMESTAMPTZ"
        );
        assert_eq!(column_type_to_pg_sql(&ColumnType::Json), "JSONB");
        assert_eq!(column_type_to_pg_sql(&ColumnType::Bytes), "BYTEA");
    }

    #[test]
    fn custom_unknown_type_falls_back_to_text() {
        // The macro currently emits `Custom("UNKNOWN")` as a
        // placeholder until type inference lands. The
        // adapter must emit valid SQL for this case (TEXT
        // is the safest choice).
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::Custom("UNKNOWN")),
            "TEXT",
        );
    }

    #[test]
    fn custom_non_unknown_type_emitted_verbatim() {
        // The adapter-specific escape hatch: a domain crate
        // can override the default mapping by emitting a
        // concrete Postgres type (e.g. `CITEXT`, `JSONB PATH`).
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::Custom("CITEXT")),
            "CITEXT",
        );
        assert_eq!(
            column_type_to_pg_sql(&ColumnType::Custom("JSONB PATH")),
            "JSONB PATH",
        );
    }

    // -- ForeignKeyAction mapping --------------------------------------------

    #[test]
    fn foreign_key_action_maps_to_sql_keywords() {
        assert_eq!(
            foreign_key_action_sql(&ForeignKeyAction::NoAction),
            "NO ACTION",
        );
        assert_eq!(
            foreign_key_action_sql(&ForeignKeyAction::Restrict),
            "RESTRICT",
        );
        assert_eq!(
            foreign_key_action_sql(&ForeignKeyAction::Cascade),
            "CASCADE",
        );
        assert_eq!(
            foreign_key_action_sql(&ForeignKeyAction::SetNull),
            "SET NULL",
        );
        assert_eq!(
            foreign_key_action_sql(&ForeignKeyAction::SetDefault),
            "SET DEFAULT",
        );
    }

    // -- build_create_table_sql ----------------------------------------------

    /// Test 1 (create_schema emits CREATE TABLE for each
    /// registered aggregate): when the registry contains one
    /// aggregate, `build_create_table_sql` must emit a
    /// `CREATE TABLE` statement that names the aggregate's
    /// table and includes every declared column.
    #[test]
    fn build_create_table_sql_emits_table_and_columns() {
        let descriptor = fixture(
            "students",
            vec![
                ColumnDescriptor {
                    name: "id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: true,
                    indexed: false,
                    unique: false,
                },
                col("school_id", ColumnType::Uuid),
                col("first_name", ColumnType::String),
                col("last_name", ColumnType::String),
                col("active", ColumnType::Bool),
            ],
        );
        let sql = build_create_table_sql(&descriptor);
        assert!(
            sql.starts_with("CREATE TABLE IF NOT EXISTS students (\n"),
            "expected CREATE TABLE IF NOT EXISTS students header, got: {sql}",
        );
        assert!(sql.contains("id UUID NOT NULL"));
        assert!(sql.contains("school_id UUID"));
        assert!(sql.contains("first_name VARCHAR(255)"));
        assert!(sql.contains("last_name VARCHAR(255)"));
        assert!(sql.contains("active BOOLEAN"));
        assert!(sql.contains("PRIMARY KEY (id)"));
        assert!(
            sql.trim_end().ends_with(';'),
            "expected trailing semicolon, got: {sql}",
        );
    }

    /// The CREATE TABLE statement must include
    /// `DEFAULT gen_random_uuid()` for `Uuid` columns marked
    /// `auto_generated = true`, so ad-hoc inserts that omit
    /// the id column still satisfy the NOT NULL PRIMARY KEY
    /// constraint.
    #[test]
    fn build_create_table_sql_emits_default_for_auto_uuid_pk() {
        let descriptor = fixture(
            "widgets",
            vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: true,
                indexed: false,
                unique: false,
            }],
        );
        let sql = build_create_table_sql(&descriptor);
        assert!(
            sql.contains("DEFAULT gen_random_uuid()"),
            "expected DEFAULT gen_random_uuid() on the auto UUID PK, got: {sql}",
        );
    }

    /// A non-PK `unique` column must emit a `UNIQUE` keyword
    /// inline (not a separate constraint — keeping the
    /// constraint count low makes the DDL easier to diff).
    #[test]
    fn build_create_table_sql_emits_unique_for_unique_columns() {
        let descriptor = fixture(
            "users",
            vec![
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
                    name: "email",
                    column_type: ColumnType::String,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: true,
                    unique: true,
                },
            ],
        );
        let sql = build_create_table_sql(&descriptor);
        assert!(
            sql.contains("email VARCHAR(255) NOT NULL UNIQUE"),
            "expected email column with UNIQUE inline, got: {sql}",
        );
    }

    // -- build_index_sql -----------------------------------------------------

    #[test]
    fn build_index_sql_emits_create_index() {
        let idx = IndexDescriptor {
            name: "students_school_id_idx",
            columns: vec!["school_id"],
            unique: false,
        };
        let sql = build_index_sql(&idx, "students");
        assert_eq!(sql, "CREATE INDEX IF NOT EXISTS students_school_id_idx ON students (school_id);\n");
    }

    #[test]
    fn build_index_sql_emits_unique_keyword_for_unique_indexes() {
        let idx = IndexDescriptor {
            name: "users_email_idx",
            columns: vec!["email"],
            unique: true,
        };
        let sql = build_index_sql(&idx, "users");
        assert!(sql.starts_with("CREATE UNIQUE INDEX IF NOT EXISTS users_email_idx"));
    }

    #[test]
    fn build_index_sql_joins_multiple_columns() {
        let idx = IndexDescriptor {
            name: "composite_idx",
            columns: vec!["school_id", "last_name"],
            unique: false,
        };
        let sql = build_index_sql(&idx, "students");
        assert!(sql.contains("(school_id, last_name)"));
    }

    // -- build_foreign_key_sql -----------------------------------------------

    #[test]
    fn build_foreign_key_sql_emits_deterministic_constraint_name() {
        let fk = ForeignKeyDescriptor {
            column: "school_id",
            references_table: "schools",
            references_column: "id",
            on_delete: ForeignKeyAction::Restrict,
            on_update: ForeignKeyAction::NoAction,
        };
        let sql = build_foreign_key_sql(&fk, "students");
        assert!(sql.contains("students_school_id_fk"));
        assert!(sql.contains("REFERENCES schools(id)"));
        assert!(sql.contains("ON DELETE RESTRICT"));
        assert!(sql.contains("ON UPDATE NO ACTION"));
        // Idempotency: the DO block swallows the
        // duplicate_object exception.
        assert!(sql.contains("EXCEPTION WHEN duplicate_object THEN NULL"));
    }

    // -- build_rls_policy_sql ------------------------------------------------

    #[test]
    fn build_rls_policy_sql_emits_enable_and_policy() {
        let policy = RlsPolicy {
            name: "students_school_isolation",
            using_expr: "school_id = current_setting('app.current_school_id')::UUID",
            with_check_expr: None,
        };
        let sql = build_rls_policy_sql(&policy, "students");
        assert!(sql.contains("ENABLE ROW LEVEL SECURITY"));
        assert!(sql.contains("FORCE ROW LEVEL SECURITY"));
        assert!(sql.contains("CREATE POLICY students_school_isolation ON students"));
        assert!(sql.contains(
            "USING (school_id = current_setting('app.current_school_id')::UUID)"
        ));
        // with_check defaults to the using expression when
        // the caller does not supply a separate check.
        assert!(sql.contains("WITH CHECK (school_id = current_setting"));
        assert!(sql.contains("EXCEPTION WHEN duplicate_object THEN NULL"));
    }

    #[test]
    fn build_rls_policy_sql_respects_explicit_with_check() {
        let policy = RlsPolicy {
            name: "students_insert",
            using_expr: "school_id = current_setting('app.current_school_id')::UUID",
            with_check_expr: Some("created_by = current_user"),
        };
        let sql = build_rls_policy_sql(&policy, "students");
        assert!(sql.contains("WITH CHECK (created_by = current_user)"));
    }

    // -- build_aggregate_ddl (composition) -----------------------------------

    /// Test 1: walking the registry must emit a `CREATE
    /// TABLE` for every registered aggregate. Verifies the
    /// composition of `build_aggregate_ddl` over a small
    /// list of fixtures.
    #[test]
    fn build_aggregate_ddl_emits_one_create_table_per_aggregate() {
        let students = fixture(
            "students",
            vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: true,
                indexed: false,
                unique: false,
            }],
        );
        let classes = fixture(
            "classes",
            vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: true,
                indexed: false,
                unique: false,
            }],
        );
        let mut buf = String::new();
        buf.push_str(&build_aggregate_ddl(&students));
        buf.push_str(&build_aggregate_ddl(&classes));
        assert!(
            buf.contains("CREATE TABLE IF NOT EXISTS students ("),
            "expected CREATE TABLE for students, got: {buf}",
        );
        assert!(
            buf.contains("CREATE TABLE IF NOT EXISTS classes ("),
            "expected CREATE TABLE for classes, got: {buf}",
        );
    }

    /// Test 2: a `ColumnType::Custom("UNKNOWN")` column must
    /// be emitted as `TEXT` (not as the literal string
    /// `UNKNOWN`). The macro emits `UNKNOWN` as a placeholder
    /// for type inference; the adapter must produce valid
    /// SQL in the meantime.
    #[test]
    fn build_create_table_sql_emits_text_for_custom_unknown() {
        let descriptor = fixture(
            "mystery",
            vec![
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
                    name: "untyped_payload",
                    column_type: ColumnType::Custom("UNKNOWN"),
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
            ],
        );
        let sql = build_create_table_sql(&descriptor);
        assert!(
            sql.contains("untyped_payload TEXT"),
            "expected UNKNOWN placeholder to fall back to TEXT, got: {sql}",
        );
        // The literal token `UNKNOWN` must not appear as a
        // SQL type — it would be a syntax error.
        assert!(
            !sql.contains(" UNKNOWN"),
            "the UNKNOWN placeholder must not leak into the emitted SQL: {sql}",
        );
    }

    /// Test 3: the emitted DDL must be idempotent. We verify
    /// this on the static-string side: every aggregate-level
    /// statement uses `IF NOT EXISTS` (for `CREATE TABLE` /
    /// `CREATE INDEX`) or the `DO $$ ... EXCEPTION WHEN
    /// duplicate_object` pattern (for `ALTER TABLE ADD
    /// CONSTRAINT` and `CREATE POLICY`, which Postgres does
    /// not support with `IF NOT EXISTS`).
    #[test]
    fn build_aggregate_ddl_is_idempotent_via_if_not_exists_or_do_block() {
        let descriptor = EntityDescriptor {
            table: "widgets",
            columns: vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: true,
                indexed: false,
                unique: false,
            }],
            indexes: vec![IndexDescriptor {
                name: "widgets_owner_idx",
                columns: vec!["owner_id"],
                unique: false,
            }],
            foreign_keys: vec![ForeignKeyDescriptor {
                column: "owner_id",
                references_table: "users",
                references_column: "id",
                on_delete: ForeignKeyAction::Cascade,
                on_update: ForeignKeyAction::NoAction,
            }],
            rls: vec![RlsPolicy {
                name: "widgets_school_isolation",
                using_expr: "school_id = current_setting('app.current_school_id')::UUID",
                with_check_expr: None,
            }],
        };
        let ddl = build_aggregate_ddl(&descriptor);
        // CREATE TABLE must use IF NOT EXISTS
        assert!(
            ddl.contains("CREATE TABLE IF NOT EXISTS widgets ("),
            "expected CREATE TABLE IF NOT EXISTS, got: {ddl}",
        );
        // CREATE INDEX must use IF NOT EXISTS
        assert!(
            ddl.contains("CREATE INDEX IF NOT EXISTS widgets_owner_idx"),
            "expected CREATE INDEX IF NOT EXISTS, got: {ddl}",
        );
        // FK and POLICY must use the DO-block pattern
        assert!(
            ddl.contains("EXCEPTION WHEN duplicate_object THEN NULL"),
            "expected DO-block idempotency for FK and POLICY, got: {ddl}",
        );
        // Two DO blocks (one for FK, one for RLS) — guard
        // against accidental over-/under-emission.
        let do_count = ddl.matches("DO $$ BEGIN").count();
        assert_eq!(
            do_count, 2,
            "expected exactly 2 DO blocks (FK + RLS), got {do_count} in: {ddl}",
        );
    }

    /// `build_full_ddl` must concatenate the cross-cutting
    /// DDL and the per-aggregate DDL without mangling either.
    /// This guards against a regression where the
    /// concatenation drops the cross-cutting prefix or skips
    /// the aggregate suffix.
    #[test]
    fn build_full_ddl_concatenates_cross_cutting_and_aggregates() {
        let agg: &'static EntityDescriptor = Box::leak(Box::new(fixture(
            "widgets",
            vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: true,
                indexed: false,
                unique: false,
            }],
        )));
        let cross_cutting = "-- cross-cutting prelude\nCREATE TABLE IF NOT EXISTS engine.outbox (event_id UUID NOT NULL);\n";
        let aggregates: &[&'static EntityDescriptor] = std::slice::from_ref(&agg);
        let script = build_full_ddl(cross_cutting, aggregates);
        assert!(script.starts_with("-- cross-cutting prelude"));
        assert!(script.contains("CREATE TABLE IF NOT EXISTS engine.outbox"));
        assert!(script.contains("CREATE TABLE IF NOT EXISTS widgets ("));
    }

    /// `registered_aggregates()` must return an empty slice
    /// until a domain crate wires its `ENTITY_DESCRIPTOR`
    /// into the registry. This is the current state per
    /// `AGENTS.md` § "Status" (domain implementation begins
    /// in Phase 3+).
    #[test]
    fn registered_aggregates_is_empty_initially() {
        assert!(
            registered_aggregates().is_empty(),
            "registry must be empty until domain crates register their ENTITY_DESCRIPTOR",
        );
    }
}
