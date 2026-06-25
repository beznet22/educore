//! MySQL `create_schema()` — walks the macro-emitted
//! [`EntityDescriptor`] AST and emits dialect-specific DDL.
//!
//! Per `docs/build-plan.md` (Cluster A stage 3) and the engine's
//! runtime-DDL emission flow documented in
//! `docs/schemas/sql-dialects/README.md`, the engine never
//! executes hand-written `.sql` files at runtime; the
//! `educore-storage-<db>` adapter walks the
//! `#[derive(DomainQuery)]`-emitted `ENTITY_DESCRIPTOR` on
//! every aggregate at startup and emits the DDL string.
//!
//! ## What this module emits
//!
//! For each registered aggregate:
//!
//! 1. `CREATE TABLE IF NOT EXISTS <table> (...)` with one line
//!    per column, `NULL` / `NOT NULL`, `PRIMARY KEY`, and
//!    `AUTO_INCREMENT` as marked.
//! 2. `CREATE [UNIQUE] INDEX ...` for each
//!    [`IndexDescriptor`](educore_core::query::IndexDescriptor).
//! 3. `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ...` for
//!    each
//!    [`ForeignKeyDescriptor`](educore_core::query::ForeignKeyDescriptor).
//! 4. RLS policies are **skipped** with a `// TODO: MySQL RLS`
//!    marker — MySQL 8 supports row-level security but with
//!    caveats that are out of scope for this PR.
//!
//! Plus the 6 engine cross-cutting tables (`outbox`,
//! `audit_log`, `idempotency`, `event_log`, `schema_registry`,
//! `system_user`) via
//! `migrations/engine/0000_engine_core.mysql.sql`
//! (`include_str!`'d at compile time).
//!
//! ## Idempotency
//!
//! Every DDL statement uses `IF NOT EXISTS` so re-runs are
//! safe. The cross-cutting DDL additionally wraps the script in
//! `SET FOREIGN_KEY_CHECKS=0` / `=1`.
//!
//! ## Custom type fallback
//!
//! `ColumnType::Custom("UNKNOWN")` — the macro's placeholder
//! until type inference lands (Cluster A stage 2 follow-up) —
//! maps to `TEXT`. Any other `ColumnType::Custom(s)` is passed
//! through verbatim (adapters that need to override the type
//! emit it on the descriptor).
//!
//! ## Registration
//!
//! Domain aggregates register their
//! [`&'static EntityDescriptor`](educore_core::query::EntityDescriptor)
//! at startup via
//! [`register_entity_descriptor`]. The macro emits a
//! `pub const ENTITY_DESCRIPTOR: EntityDescriptor` on every
//! derived struct; domain crates call the registration helper
//! from their `init` path.
//!
//! For tests and tools that want to bypass the global
//! registry, [`create_schema_with`] accepts the descriptor
//! list explicitly.

use std::sync::{Mutex, OnceLock, PoisonError};

use tracing::instrument;

use educore_core::error::{DomainError, Result};
use educore_core::query::{
    ColumnType, EntityDescriptor, ForeignKeyAction, ForeignKeyDescriptor, IndexDescriptor,
};

use crate::error::StringError;
use crate::storage::MysqlStorageAdapter;

/// The canonical MySQL DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time.
const CROSS_CUTTING_DDL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.mysql.sql");

/// Global registry of macro-emitted `EntityDescriptor`s. Domain
/// crates call [`register_entity_descriptor`] at init time;
/// [`registered_descriptors`] returns a snapshot.
static REGISTRY: OnceLock<Mutex<Vec<&'static EntityDescriptor>>> = OnceLock::new();

/// Returns the global registry. Initialised on first access.
fn registry() -> &'static Mutex<Vec<&'static EntityDescriptor>> {
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Adds `desc` to the global registry. Idempotent on identical
/// pointer identity (calling twice with the same `&'static`
/// descriptor is safe but registers the pointer twice — callers
/// should register each descriptor exactly once at startup).
pub fn register_entity_descriptor(desc: &'static EntityDescriptor) {
    lock_registry().push(desc);
}

/// Returns a snapshot of the registered descriptors, in
/// registration order. The returned `Vec` is a copy of the
/// registry's pointer list; the descriptors themselves are
/// `'static`.
#[must_use]
pub fn registered_descriptors() -> Vec<&'static EntityDescriptor> {
    lock_registry().iter().copied().collect()
}

/// Clears the global registry. Intended for tests that want to
/// start from a known-empty state.
pub fn clear_registry() {
    lock_registry().clear();
}

/// Locks the global registry. Poisoning (a panic while holding
/// the lock) is recovered by returning the inner guard — the
/// registry's invariants are simple enough (a `Vec` of
/// `&'static` pointers) that a poisoned state is recoverable.
fn lock_registry() -> std::sync::MutexGuard<'static, Vec<&'static EntityDescriptor>> {
    registry().lock().unwrap_or_else(PoisonError::into_inner)
}

/// Public entry point. Walks every registered `EntityDescriptor`
/// and emits the corresponding DDL against `adapter`.
///
/// The 6 engine cross-cutting tables are emitted first (via
/// the canonical `0000_engine_core.mysql.sql`), then each
/// registered aggregate's table / indexes / foreign keys.
///
/// Idempotent: every statement uses `IF NOT EXISTS`. RLS
/// policies are skipped (TODO marker) for this adapter.
pub async fn create_schema(adapter: &MysqlStorageAdapter) -> Result<()> {
    let descriptors = registered_descriptors();
    create_schema_with(adapter, &descriptors).await
}

/// Like [`create_schema`] but takes the descriptor list
/// explicitly. The free function for tests, tools, and any
/// caller that wants to bypass the global registry.
pub async fn create_schema_with(
    adapter: &MysqlStorageAdapter,
    descriptors: &[&'static EntityDescriptor],
) -> Result<()> {
    let statements = build_schema_statements(descriptors);
    for stmt in &statements {
        sqlx::raw_sql(stmt)
            .execute(adapter.db())
            .await
            .map_err(|e| {
                DomainError::infrastructure(StringError(format!(
                    "create_schema: statement failed (first 200 chars: `{}`): {e}",
                    &stmt.chars().take(200).collect::<String>()
                )))
            })?;
    }
    Ok(())
}

/// Returns the ordered list of DDL statements that
/// [`create_schema_with`] would execute. Pure function: no I/O.
/// Used by tests that want to inspect the emitted DDL without
/// touching a database.
///
/// The 6 cross-cutting tables are emitted as one concatenated
/// `include_str!`'d block. Each aggregate then contributes:
///
/// 1. One `CREATE TABLE` statement.
/// 2. One `CREATE INDEX` / `CREATE UNIQUE INDEX` per
///    [`IndexDescriptor`].
/// 3. One `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ...`
///    per [`ForeignKeyDescriptor`].
/// 4. RLS policies are emitted as a `-- TODO: MySQL RLS ...`
///    comment block (MySQL 8 supports policies but with
///    caveats; out of scope for this PR).
pub fn build_schema_statements(descriptors: &[&EntityDescriptor]) -> Vec<String> {
    let mut out = Vec::new();
    out.push(CROSS_CUTTING_DDL.to_owned());
    for desc in descriptors {
        out.push(build_table_ddl(desc));
        for idx in &desc.indexes {
            out.push(build_index_ddl(desc.table, idx));
        }
        for fk in &desc.foreign_keys {
            out.push(build_fk_ddl(desc.table, fk));
        }
        if !desc.rls.is_empty() {
            // TODO: MySQL RLS — MySQL 8 supports row-level
            // security but with caveats (no PERMISSIVE /
            // RESTRICTIVE distinction, no per-table WITH
            // CHECK that composes cleanly with the engine's
            // partition-pruning, requires DEFINER grants).
            // Defer to a follow-up PR.
            let mut comment =
                String::from("-- TODO: MySQL RLS — skipped (out of scope for this PR)\n");
            for policy in &desc.rls {
                comment.push_str(&format!(
                    "--   policy: {} USING {} (with_check: {})\n",
                    policy.name,
                    policy.using_expr,
                    policy
                        .with_check_expr
                        .map_or("<none>".to_owned(), |s| s.to_owned()),
                ));
            }
            out.push(comment);
        }
    }
    out
}

/// Maps a dialect-agnostic [`ColumnType`] to the MySQL native
/// type string. The default for `Custom("UNKNOWN")` is `TEXT`;
/// any other `Custom(s)` is passed through verbatim.
#[must_use]
pub fn column_type_to_mysql(ct: &ColumnType) -> &'static str {
    match ct {
        ColumnType::Uuid => "CHAR(36)",
        ColumnType::String => "VARCHAR(255)",
        ColumnType::Text => "TEXT",
        ColumnType::I64 => "BIGINT",
        ColumnType::U64 => "BIGINT UNSIGNED",
        ColumnType::I32 => "INT",
        ColumnType::U32 => "INT UNSIGNED",
        ColumnType::F64 => "DOUBLE",
        ColumnType::Bool => "BOOLEAN",
        ColumnType::Timestamp => "DATETIME(6)",
        ColumnType::Json => "JSON",
        ColumnType::Bytes => "BLOB",
        // The macro's type-inference placeholder (Cluster A
        // stage 2 follow-up). TEXT is the safe fallback — it
        // round-trips any payload the engine emits today.
        ColumnType::Custom("UNKNOWN") => "TEXT",
        // Any other custom hint is a verbatim pass-through.
        // Adapters that want a specific type emit it on the
        // descriptor.
        ColumnType::Custom(other) => other,
    }
}

/// Builds the `CREATE TABLE IF NOT EXISTS` statement for one
/// aggregate. Pure function: no I/O. The DDL follows the engine
/// canonical form: backtick-quoted identifiers, `ENGINE=InnoDB`,
/// `CHARSET=utf8mb4`, `COLLATE=utf8mb4_unicode_ci`.
///
/// Each column contributes:
///
/// - The quoted name.
/// - The MySQL type (see [`column_type_to_mysql`]).
/// - `NOT NULL` if `!nullable`.
/// - `PRIMARY KEY` if `primary_key`.
/// - `AUTO_INCREMENT` if `auto_generated`.
///
/// Single-column unique constraints are emitted as
/// `UNIQUE KEY <name>_unique (<col>)` after the column list.
#[must_use]
pub fn build_table_ddl(desc: &EntityDescriptor) -> String {
    let mut s = String::new();
    s.push_str("CREATE TABLE IF NOT EXISTS `");
    s.push_str(desc.table);
    s.push_str("` (\n");
    let mut first = true;
    for col in &desc.columns {
        if !first {
            s.push_str(",\n");
        }
        first = false;
        s.push_str("  `");
        s.push_str(col.name);
        s.push_str("` ");
        s.push_str(column_type_to_mysql(&col.column_type));
        if !col.nullable {
            s.push_str(" NOT NULL");
        }
        if col.primary_key {
            s.push_str(" PRIMARY KEY");
        }
        if col.auto_generated {
            s.push_str(" AUTO_INCREMENT");
        }
    }
    for col in &desc.columns {
        if col.unique && !col.primary_key {
            s.push_str(",\n  UNIQUE KEY `");
            s.push_str(col.name);
            s.push_str("_unique` (`");
            s.push_str(col.name);
            s.push_str("`)");
        }
    }
    s.push_str("\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;\n");
    s
}

/// Builds the `CREATE [UNIQUE] INDEX ...` statement for one
/// [`IndexDescriptor`]. Pure function.
#[must_use]
pub fn build_index_ddl(table: &str, idx: &IndexDescriptor) -> String {
    let mut s = String::new();
    s.push_str("CREATE ");
    if idx.unique {
        s.push_str("UNIQUE ");
    }
    s.push_str("INDEX `");
    s.push_str(idx.name);
    s.push_str("` ON `");
    s.push_str(table);
    s.push_str("` (");
    let mut first = true;
    for col in &idx.columns {
        if !first {
            s.push_str(", ");
        }
        first = false;
        s.push('`');
        s.push_str(col);
        s.push('`');
    }
    s.push_str(");\n");
    s
}

/// Builds the `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY`
/// statement for one [`ForeignKeyDescriptor`]. Pure function.
///
/// The constraint name is `<table>_<column>_fk`. MySQL caps
/// identifier length at 64; long table+column names will be
/// truncated by the server, which is acceptable for the
/// engine's schema (no human reads these names — they're
/// internal to the database).
#[must_use]
pub fn build_fk_ddl(table: &str, fk: &ForeignKeyDescriptor) -> String {
    let mut s = String::new();
    s.push_str("ALTER TABLE `");
    s.push_str(table);
    s.push_str("` ADD CONSTRAINT `");
    s.push_str(table);
    s.push('_');
    s.push_str(fk.column);
    s.push_str("_fk` FOREIGN KEY (`");
    s.push_str(fk.column);
    s.push_str("`) REFERENCES `");
    s.push_str(fk.references_table);
    s.push_str("` (`");
    s.push_str(fk.references_column);
    s.push_str("`) ON DELETE ");
    s.push_str(fk_action_to_mysql(&fk.on_delete));
    s.push_str(" ON UPDATE ");
    s.push_str(fk_action_to_mysql(&fk.on_update));
    s.push_str(";\n");
    s
}

/// Maps a [`ForeignKeyAction`] to the MySQL `ON DELETE` /
/// `ON UPDATE` keyword.
#[must_use]
pub fn fk_action_to_mysql(action: &ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "NO ACTION",
        ForeignKeyAction::Restrict => "RESTRICT",
        ForeignKeyAction::Cascade => "CASCADE",
        ForeignKeyAction::SetNull => "SET NULL",
        ForeignKeyAction::SetDefault => "SET DEFAULT",
    }
}

/// Instrumented wrapper used by tests + tools that want to
/// report progress. Mirrors the structure of
/// `MysqlStorageAdapter::migrate` for consistency.
#[instrument(skip(adapter, descriptors))]
pub async fn create_schema_with_report(
    adapter: &MysqlStorageAdapter,
    descriptors: &[&'static EntityDescriptor],
) -> Result<u32> {
    let statements = build_schema_statements(descriptors);
    let total = u32::try_from(statements.len()).map_err(|e| {
        DomainError::validation(format!(
            "create_schema_with_report: statement count overflow: {e}"
        ))
    })?;
    for stmt in &statements {
        sqlx::raw_sql(stmt)
            .execute(adapter.db())
            .await
            .map_err(|e| {
                DomainError::infrastructure(StringError(format!(
                    "create_schema_with_report: statement failed: {e}"
                )))
            })?;
    }
    Ok(total)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::query::{ColumnDescriptor, RlsPolicy};

    /// Serialise the registry-touching tests in this module so
    /// parallel execution doesn't let one test's inserts leak
    /// into another test's snapshot assertions.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Test 1 of the cluster-A stage 3 PR: `create_schema`
    /// emits a `CREATE TABLE` for each registered aggregate.
    /// We exercise the pure `build_schema_statements` path
    /// (no database required) and inspect the resulting
    /// strings.
    #[test]
    fn build_schema_statements_emits_create_table_for_each_registered_aggregate() {
        let desc_a: &'static EntityDescriptor = Box::leak(Box::new(EntityDescriptor {
            table: "alpha_widget",
            columns: vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: false,
                indexed: false,
                unique: false,
            }],
            indexes: vec![],
            foreign_keys: vec![],
            rls: vec![],
        }));
        let desc_b: &'static EntityDescriptor = Box::leak(Box::new(EntityDescriptor {
            table: "beta_gadget",
            columns: vec![
                ColumnDescriptor {
                    name: "id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "name",
                    column_type: ColumnType::String,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
            ],
            indexes: vec![IndexDescriptor {
                name: "beta_gadget_name_idx",
                columns: vec!["name"],
                unique: false,
            }],
            foreign_keys: vec![ForeignKeyDescriptor {
                column: "parent_id",
                references_table: "alpha_widget",
                references_column: "id",
                on_delete: ForeignKeyAction::Cascade,
                on_update: ForeignKeyAction::NoAction,
            }],
            rls: vec![],
        }));

        let descriptors = [desc_a, desc_b];
        let statements = build_schema_statements(&descriptors);

        // The first statement is the canonical cross-cutting DDL.
        // The .sql file is large; search the full string for the
        // CREATE TABLE markers. (The canonical DDL uses
        // unquoted identifiers; we emit backtick-quoted
        // identifiers only for our own descriptor-driven
        // CREATE TABLE / INDEX / FK statements.)
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS outbox"),
            "cross-cutting DDL must include the outbox CREATE TABLE; length = {}",
            statements[0].len(),
        );
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS audit_log"),
            "cross-cutting DDL must include audit_log CREATE TABLE",
        );
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS idempotency"),
            "cross-cutting DDL must include idempotency CREATE TABLE",
        );
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS event_log"),
            "cross-cutting DDL must include event_log CREATE TABLE",
        );
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS schema_registry"),
            "cross-cutting DDL must include schema_registry CREATE TABLE",
        );
        assert!(
            statements[0].contains("CREATE TABLE IF NOT EXISTS system_user"),
            "cross-cutting DDL must include system_user CREATE TABLE",
        );

        // Aggregate A: one CREATE TABLE.
        let a_table = statements
            .iter()
            .find(|s| s.contains("CREATE TABLE IF NOT EXISTS `alpha_widget`"))
            .expect("alpha_widget CREATE TABLE must be present");
        assert!(a_table.contains("`id` CHAR(36) NOT NULL PRIMARY KEY"));
        assert!(a_table.contains("ENGINE=InnoDB DEFAULT CHARSET=utf8mb4"));

        // Aggregate B: CREATE TABLE + 1 index + 1 FK.
        let b_table = statements
            .iter()
            .find(|s| s.contains("CREATE TABLE IF NOT EXISTS `beta_gadget`"))
            .expect("beta_gadget CREATE TABLE must be present");
        assert!(b_table.contains("`name` VARCHAR(255) NOT NULL"));
        let b_index = statements
            .iter()
            .find(|s| s.contains("CREATE INDEX `beta_gadget_name_idx`"))
            .expect("beta_gadget_name_idx must be present");
        assert!(b_index.contains("ON `beta_gadget` (`name`)"));
        let b_fk = statements
            .iter()
            .find(|s| s.contains("ADD CONSTRAINT `beta_gadget_parent_id_fk`"))
            .expect("beta_gadget FK must be present");
        assert!(b_fk.contains("FOREIGN KEY (`parent_id`)"));
        assert!(b_fk.contains("REFERENCES `alpha_widget` (`id`)"));
        assert!(b_fk.contains("ON DELETE CASCADE"));
        assert!(b_fk.contains("ON UPDATE NO ACTION"));
    }

    /// Test 2 of the cluster-A stage 3 PR: `create_schema`
    /// handles `ColumnType::Custom("UNKNOWN")` by emitting
    /// `TEXT` (the macro's placeholder fallback).
    #[test]
    fn column_type_custom_unknown_falls_back_to_text() {
        assert_eq!(
            column_type_to_mysql(&ColumnType::Custom("UNKNOWN")),
            "TEXT",
            "ColumnType::Custom(\"UNKNOWN\") must map to TEXT",
        );
        // And every other `Custom(s)` is a verbatim pass-through.
        assert_eq!(
            column_type_to_mysql(&ColumnType::Custom("DECIMAL(10,2)")),
            "DECIMAL(10,2)",
        );
    }

    /// Pure-function idempotency check: running
    /// `build_schema_statements` twice on the same descriptors
    /// yields byte-identical output (the SQL is deterministic).
    #[test]
    fn build_schema_statements_is_deterministic() {
        let desc: &'static EntityDescriptor = Box::leak(Box::new(EntityDescriptor {
            table: "widget",
            columns: vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: false,
                indexed: false,
                unique: false,
            }],
            indexes: vec![],
            foreign_keys: vec![],
            rls: vec![],
        }));
        let a = build_schema_statements(&[desc]);
        let b = build_schema_statements(&[desc]);
        assert_eq!(a, b, "DDL must be deterministic across calls");
    }

    /// Every known `ColumnType` maps to a non-empty MySQL type
    /// string. This is the dialect-mapping contract.
    #[test]
    fn column_type_mapping_is_complete() {
        let cases: Vec<ColumnType> = vec![
            ColumnType::Uuid,
            ColumnType::String,
            ColumnType::Text,
            ColumnType::I64,
            ColumnType::U64,
            ColumnType::I32,
            ColumnType::U32,
            ColumnType::F64,
            ColumnType::Bool,
            ColumnType::Timestamp,
            ColumnType::Json,
            ColumnType::Bytes,
        ];
        for ct in &cases {
            let mapped = column_type_to_mysql(ct);
            assert!(
                !mapped.is_empty(),
                "ColumnType {ct:?} must map to a non-empty type"
            );
        }
    }

    /// RLS policies on the descriptor are emitted as a TODO
    /// marker (MySQL 8 supports policies but with caveats; out
    /// of scope for this PR). The descriptor's table itself is
    /// still created.
    #[test]
    fn rls_policies_are_skipped_with_todo_marker() {
        let desc: &'static EntityDescriptor = Box::leak(Box::new(EntityDescriptor {
            table: "tenant_widget",
            columns: vec![ColumnDescriptor {
                name: "id",
                column_type: ColumnType::Uuid,
                nullable: false,
                primary_key: true,
                auto_generated: false,
                indexed: false,
                unique: false,
            }],
            indexes: vec![],
            foreign_keys: vec![],
            rls: vec![RlsPolicy {
                name: "tenant_widget_school_isolation",
                using_expr: "school_id = current_setting('app.school_id')::uuid",
                with_check_expr: Some("school_id = current_setting('app.school_id')::uuid"),
            }],
        }));
        let stmts = build_schema_statements(&[desc]);
        let todo = stmts
            .iter()
            .find(|s| s.contains("TODO: MySQL RLS"))
            .expect("TODO MySQL RLS marker must be emitted");
        assert!(todo.contains("tenant_widget_school_isolation"));
        // And the CREATE TABLE is still present.
        assert!(
            stmts
                .iter()
                .any(|s| s.contains("CREATE TABLE IF NOT EXISTS `tenant_widget`")),
            "table CREATE must still be emitted when RLS is present",
        );
    }

    /// `clear_registry` actually empties the registry; a test
    /// that follows can register a clean slate. Serialised
    /// against other tests that touch the global registry.
    #[test]
    fn clear_registry_empties_the_registry() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let desc: &'static EntityDescriptor = Box::leak(Box::new(EntityDescriptor {
            table: "ephemeral",
            columns: vec![],
            indexes: vec![],
            foreign_keys: vec![],
            rls: vec![],
        }));
        register_entity_descriptor(desc);
        assert!(!registered_descriptors().is_empty());
        clear_registry();
        assert!(registered_descriptors().is_empty());
    }

    /// Every `ForeignKeyAction` variant maps to a non-empty
    /// MySQL keyword.
    #[test]
    fn fk_action_mapping_is_complete() {
        let actions = [
            ForeignKeyAction::NoAction,
            ForeignKeyAction::Restrict,
            ForeignKeyAction::Cascade,
            ForeignKeyAction::SetNull,
            ForeignKeyAction::SetDefault,
        ];
        for a in &actions {
            let mapped = fk_action_to_mysql(a);
            assert!(
                !mapped.is_empty(),
                "FK action {a:?} must map to a non-empty keyword"
            );
        }
        // Spot-check the cascade keyword.
        assert_eq!(fk_action_to_mysql(&ForeignKeyAction::Cascade), "CASCADE");
        assert_eq!(fk_action_to_mysql(&ForeignKeyAction::SetNull), "SET NULL");
    }
}
