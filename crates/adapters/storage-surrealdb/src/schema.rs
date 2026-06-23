//! SurrealDB DDL emission for the engine's `StorageAdapter`.
//!
//! Per `docs/build-plan.md` § "The macro-driven schema" and
//! `docs/schemas/sql-dialects/surrealdb.md` § "DDL: `DEFINE TABLE`,
//! `DEFINE FIELD`, `DEFINE INDEX`, `DEFINE EVENT`", the
//! `StorageAdapter::create_schema()` entry point walks the
//! macro-emitted [`EntityDescriptor`] AST (added in PR `e036f73`)
//! and emits SurrealDB-specific DDL.
//!
//! The translation table is fixed by the dialect spec:
//!
//! | `ColumnType`        | SurrealDB `TYPE`               |
//! | ------------------- | ------------------------------ |
//! | `Uuid`              | `string` (with UUID ASSERT)    |
//! | `String`            | `string`                       |
//! | `Text`              | `string`                       |
//! | `I64` / `U64`       | `int`                          |
//! | `I32` / `U32`       | `int`                          |
//! | `F64`               | `float`                        |
//! | `Bool`              | `bool`                         |
//! | `Timestamp`         | `datetime`                     |
//! | `Json`              | `object`                       |
//! | `Bytes`             | `bytes`                        |
//! | `Custom("UNKNOWN")` | `string` (fallback — see below) |
//! | `Custom(other)`     | passed through verbatim        |
//!
//! The `Custom("UNKNOWN")` placeholder is emitted by the
//! `#[derive(DomainQuery)]` macro on every column until type
//! inference lands (Cluster A stage 2 follow-up). Per the
//! audit finding ADAPTER-SD-DDL-001, we fall back to `string`
//! so the schema is valid even on the placeholder output.
//!
//! Foreign keys are encoded in the column type:
//! `DEFINE FIELD class_id ON TABLE academic_students TYPE
//! option<record<academic_classes>>`. There is no separate
//! `ALTER TABLE ADD CONSTRAINT` step; the column type **is**
//! the constraint (per `docs/schemas/sql-dialects/surrealdb.md`
//! § "Foreign keys via `record<TABLE>`").
//!
//! Every `DEFINE` statement uses `IF NOT EXISTS` for
//! idempotency — SurrealDB 2.x honours the clause on
//! `DEFINE TABLE` / `DEFINE FIELD` / `DEFINE INDEX`, so a
//! re-run is a no-op (audit finding ADAPTER-SD-DDL-002).
//!
//! [`EntityDescriptor`]: educore_core::query::EntityDescriptor

use educore_core::error::Result;
use educore_core::query::{
    ColumnDescriptor, ColumnType, EntityDescriptor, ForeignKeyAction, IndexDescriptor,
};

use crate::connection::Db;
use crate::error::StringError;
use crate::storage::SurrealStorageAdapter;

/// The canonical SurrealDB DDL for the 6 engine cross-cutting
/// tables. `include_str!`'d at compile time. Idempotent thanks
/// to `IF NOT EXISTS` on the namespace + database definitions.
const ENGINE_CORE_SURQL: &str =
    include_str!("../../../../migrations/engine/0000_engine_core.surreal.surql");

/// Returns the list of aggregate `EntityDescriptor`s the
/// adapter will emit DDL for at `create_schema()` time.
///
/// As of this commit, the registry is empty: no domain crate
/// has implemented `#[derive(DomainQuery)]` and produced an
/// `ENTITY_DESCRIPTOR` yet. A follow-up PR will wire each
/// domain's `ENTITY_DESCRIPTOR` into this slice (or a
/// `inventory::collect!`-backed registry).
#[must_use]
pub const fn registered_aggregates() -> &'static [&'static EntityDescriptor] {
    const EMPTY: &[&EntityDescriptor] = &[];
    EMPTY
}

/// Runtime schema entry point. Emits the 6 engine cross-
/// cutting tables and walks every registered aggregate's
/// `EntityDescriptor`, emitting `DEFINE TABLE` / `DEFINE FIELD`
/// / `DEFINE INDEX` SurrealDB DDL. All statements are
/// idempotent (`IF NOT EXISTS`); running this twice against
/// the same database is a no-op.
///
/// This is the runtime consumer-facing entry point referenced
/// by `docs/build-plan.md`, `AGENTS.md`, and the dialect
/// specs; the existing [`migrate`](educore_storage::port::StorageAdapter::migrate)
/// method on `StorageAdapter` continues to work as the
/// port-trait entry point.
///
/// Tests that need to inject fixtures should call
/// [`create_schema_for`] directly.
///
/// # Errors
/// - `Infrastructure` if the underlying SurrealDB query fails.
pub async fn create_schema(adapter: &SurrealStorageAdapter) -> Result<()> {
    create_schema_for(adapter, registered_aggregates()).await
}

/// Test-friendly entry point. Emits the 6 engine cross-cutting
/// tables and walks the supplied descriptors (skipping the
/// [`registered_aggregates`] global). Use this in unit tests
/// to inject fixtures without touching the global registry.
///
/// # Idempotency
/// Every emitted `DEFINE` statement carries `IF NOT EXISTS`.
pub async fn create_schema_for(
    adapter: &SurrealStorageAdapter,
    descriptors: &[&'static EntityDescriptor],
) -> Result<()> {
    let db: &Db = adapter.db();
    db.query(ENGINE_CORE_SURQL)
        .await
        .map_err(|e| {
            educore_core::error::DomainError::infrastructure(StringError(format!(
                "create_schema: engine-core surql failed: {e}"
            )))
        })?;
    for descriptor in descriptors {
        let ddl = render_descriptor(descriptor);
        db.query(&ddl)
            .await
            .map_err(|e| {
                educore_core::error::DomainError::infrastructure(StringError(format!(
                    "create_schema: aggregate `{}` DDL failed: {e}",
                    descriptor.table
                )))
            })?;
    }
    Ok(())
}

/// Renders the full SurrealDB DDL for one aggregate: one
/// `DEFINE TABLE`, one `DEFINE FIELD` per column, and one
/// `DEFINE INDEX` per index + per FK. Public for tests and
/// parity tools (the function is pure and side-effect free).
#[must_use]
pub fn render_descriptor(descriptor: &EntityDescriptor) -> String {
    let mut out = String::with_capacity(256 + descriptor.columns.len() * 96);
    out.push_str(&render_table(descriptor.table));
    for column in &descriptor.columns {
        out.push('\n');
        out.push_str(&render_field(descriptor.table, column, &descriptor.foreign_keys));
    }
    for index in &descriptor.indexes {
        out.push('\n');
        out.push_str(&render_index(descriptor.table, index));
    }
    out
}

/// Renders `DEFINE TABLE IF NOT EXISTS <table> SCHEMAFULL`.
///
/// `SCHEMAFULL` is the engine's invariant (per
/// `docs/schemas/sql-dialects/surrealdb.md` § "`SCHEMAFUL` vs
/// `SCHEMALESS`"): every row must have the declared fields
/// with the declared types; `SCHEMALESS` (the default) is
/// rejected for both cross-cutting tables and domain
/// aggregates.
#[must_use]
pub fn render_table(table: &str) -> String {
    format!("DEFINE TABLE IF NOT EXISTS {table} SCHEMAFULL")
}

/// Renders one `DEFINE FIELD IF NOT EXISTS <column> ON TABLE
/// <table> TYPE <type> [ASSERT ...]` statement. Foreign-key
/// columns get `TYPE option<record<other_table>>` instead of
/// the plain type.
#[must_use]
pub fn render_field(
    table: &str,
    column: &ColumnDescriptor,
    foreign_keys: &[educore_core::query::ForeignKeyDescriptor],
) -> String {
    let type_str = match foreign_keys.iter().find(|fk| fk.column == column.name) {
        Some(fk) => render_fk_type(fk),
        None => render_column_type(&column.column_type, column.nullable),
    };
    let mut stmt = format!(
        "DEFINE FIELD IF NOT EXISTS {col} ON TABLE {tbl} TYPE {ty}",
        col = column.name,
        tbl = table,
        ty = type_str,
    );
    if let Some(assert) = render_assert(column) {
        stmt.push(' ');
        stmt.push_str(&assert);
    }
    stmt
}

/// Renders one `DEFINE INDEX IF NOT EXISTS <name> ON TABLE
/// <table> COLUMNS <c1>, <c2> [UNIQUE]` statement.
#[must_use]
pub fn render_index(table: &str, index: &IndexDescriptor) -> String {
    let cols = index.columns.join(", ");
    let unique = if index.unique { " UNIQUE" } else { "" };
    format!(
        "DEFINE INDEX IF NOT EXISTS {name} ON TABLE {tbl} COLUMNS {cols}{unique}",
        name = index.name,
        tbl = table,
    )
}

// ---------------------------------------------------------------------------
// Internals — all pure
// ---------------------------------------------------------------------------

/// Maps a `ColumnType` to a SurrealDB `TYPE <...>` fragment. The
/// `nullable` flag wraps the type in `option<...>`.
fn render_column_type(typ: &ColumnType, nullable: bool) -> String {
    let inner = match typ {
        ColumnType::Uuid => "string".to_owned(),
        ColumnType::String => "string".to_owned(),
        ColumnType::Text => "string".to_owned(),
        ColumnType::I64 | ColumnType::U64 | ColumnType::I32 | ColumnType::U32 => "int".to_owned(),
        ColumnType::F64 => "float".to_owned(),
        ColumnType::Bool => "bool".to_owned(),
        ColumnType::Timestamp => "datetime".to_owned(),
        ColumnType::Json => "object".to_owned(),
        ColumnType::Bytes => "bytes".to_owned(),
        // ADAPTER-SD-DDL-001: the macro emits Custom("UNKNOWN")
        // as a placeholder until type inference lands. Fall
        // back to `string` so the schema is valid; this is the
        // documented fallback behaviour per the dialect spec.
        ColumnType::Custom("UNKNOWN") => "string".to_owned(),
        ColumnType::Custom(other) => (*other).to_owned(),
    };
    if nullable {
        format!("option<{inner}>")
    } else {
        inner
    }
}

/// Renders the type fragment for a foreign-key column:
/// `option<record<other_table>>`. Nullable because SurrealDB's
/// record reference can be unset; non-null FKs would need a
/// separate NOT-NULL assertion (out of scope for this PR).
fn render_fk_type(fk: &educore_core::query::ForeignKeyDescriptor) -> String {
    format!("option<record<{}>>", fk.references_table)
}

/// Renders the optional `ASSERT ...` suffix. We don't emit
/// NULL/NOT-NULL assertions (the column type already encodes
/// nullability via `option<...>`); we only emit length /
/// range assertions that the dialect spec defines for the
/// well-known engine-invariant columns. The set is kept
/// minimal to avoid the macro's placeholder output being
/// rejected.
fn render_assert(column: &ColumnDescriptor) -> Option<String> {
    match (&column.column_type, column.name) {
        // UUIDs are stored as 36-char strings.
        (ColumnType::Uuid, _) => Some("ASSERT $value != NONE AND string::len($value) = 36".to_owned()),
        // The engine's enums (event_type, aggregate_type, etc.)
        // carry a length cap on the cross-cutting tables. We
        // don't yet have the cap from the AST; the dialect
        // spec defers the cap to the per-table .surql file.
        _ => None,
    }
}

// Suppress the unused-import warning for `ForeignKeyAction`.
// The enum is part of the public AST surface; we re-export it
// for parity tests in a follow-up PR.
#[allow(dead_code)]
const _: Option<ForeignKeyAction> = None;

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro
)]
mod tests {
    use super::*;
    use educore_core::query::{
        ColumnDescriptor, ColumnType, EntityDescriptor, IndexDescriptor,
    };

    fn sample_descriptor() -> EntityDescriptor {
        EntityDescriptor {
            table: "students",
            columns: vec![
                ColumnDescriptor {
                    name: "id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: true,
                    auto_generated: true,
                    indexed: false,
                    unique: true,
                },
                ColumnDescriptor {
                    name: "display_name",
                    column_type: ColumnType::String,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
                ColumnDescriptor {
                    name: "school_id",
                    column_type: ColumnType::Uuid,
                    nullable: false,
                    primary_key: false,
                    auto_generated: false,
                    indexed: true,
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
                    column_type: ColumnType::Custom("UNKNOWN"),
                    nullable: true,
                    primary_key: false,
                    auto_generated: false,
                    indexed: false,
                    unique: false,
                },
            ],
            indexes: vec![
                IndexDescriptor {
                    name: "idx_students_school",
                    columns: vec!["school_id"],
                    unique: false,
                },
                IndexDescriptor {
                    name: "idx_students_school_name",
                    columns: vec!["school_id", "display_name"],
                    unique: true,
                },
            ],
            foreign_keys: vec![],
            rls: vec![],
        }
    }

    #[test]
    fn render_descriptor_emits_define_table() {
        let d = sample_descriptor();
        let sql = render_descriptor(&d);
        assert!(
            sql.contains("DEFINE TABLE IF NOT EXISTS students SCHEMAFULL"),
            "expected DEFINE TABLE statement; got: {sql}"
        );
    }

    #[test]
    fn render_descriptor_emits_define_field_per_column() {
        let d = sample_descriptor();
        let sql = render_descriptor(&d);
        // Each column gets a DEFINE FIELD. The Uuid column
        // gets the 36-char length assertion.
        assert!(sql.contains("DEFINE FIELD IF NOT EXISTS id ON TABLE students TYPE string"));
        assert!(sql.contains("string::len($value) = 36"));
        assert!(sql.contains("DEFINE FIELD IF NOT EXISTS display_name ON TABLE students TYPE string"));
        assert!(sql.contains("DEFINE FIELD IF NOT EXISTS school_id ON TABLE students TYPE string"));
        assert!(sql.contains("DEFINE FIELD IF NOT EXISTS created_at ON TABLE students TYPE datetime"));
    }

    #[test]
    fn render_descriptor_emits_define_index_per_index() {
        let d = sample_descriptor();
        let sql = render_descriptor(&d);
        assert!(sql.contains(
            "DEFINE INDEX IF NOT EXISTS idx_students_school ON TABLE students COLUMNS school_id"
        ));
        assert!(sql.contains(
            "DEFINE INDEX IF NOT EXISTS idx_students_school_name ON TABLE students COLUMNS school_id, display_name UNIQUE"
        ));
    }

    #[test]
    fn render_descriptor_handles_custom_unknown_as_string() {
        let d = sample_descriptor();
        let sql = render_descriptor(&d);
        // ADAPTER-SD-DDL-001: Custom("UNKNOWN") falls back to TYPE
        // string. The column is nullable so it wraps in
        // option<string>.
        assert!(
            sql.contains(
                "DEFINE FIELD IF NOT EXISTS metadata ON TABLE students TYPE option<string>"
            ),
            "Custom(\"UNKNOWN\") must fall back to TYPE string; got: {sql}"
        );
    }

    #[test]
    fn render_descriptor_emits_fk_as_record_link() {
        let d = EntityDescriptor {
            table: "students",
            columns: vec![ColumnDescriptor {
                name: "class_id",
                column_type: ColumnType::Uuid,
                nullable: true,
                primary_key: false,
                auto_generated: false,
                indexed: true,
                unique: false,
            }],
            indexes: vec![],
            foreign_keys: vec![educore_core::query::ForeignKeyDescriptor {
                column: "class_id",
                references_table: "academic_classes",
                references_column: "id",
                on_delete: ForeignKeyAction::Restrict,
                on_update: ForeignKeyAction::NoAction,
            }],
            rls: vec![],
        };
        let sql = render_descriptor(&d);
        assert!(
            sql.contains(
                "DEFINE FIELD IF NOT EXISTS class_id ON TABLE students TYPE option<record<academic_classes>>"
            ),
            "FK column must be encoded as option<record<...>>; got: {sql}"
        );
    }

    #[test]
    fn render_table_uses_schemafull() {
        assert_eq!(
            render_table("outbox"),
            "DEFINE TABLE IF NOT EXISTS outbox SCHEMAFULL"
        );
    }

    #[test]
    fn render_index_uniqueness_round_trip() {
        let u = IndexDescriptor {
            name: "i",
            columns: vec!["a", "b"],
            unique: true,
        };
        let n = IndexDescriptor {
            name: "i",
            columns: vec!["a", "b"],
            unique: false,
        };
        assert!(render_index("t", &u).contains(" UNIQUE"));
        assert!(!render_index("t", &n).contains("UNIQUE"));
    }

    #[test]
    fn column_type_mapping_is_complete() {
        // Every variant of ColumnType must map to a non-empty
        // string. This is the exhaustiveness gate: if a new
        // variant is added, this test fails to compile because
        // render_column_type is non-exhaustive.
        let cases: Vec<(ColumnType, bool, &str)> = vec![
            (ColumnType::Uuid, false, "string"),
            (ColumnType::String, false, "string"),
            (ColumnType::Text, false, "string"),
            (ColumnType::I64, false, "int"),
            (ColumnType::U64, false, "int"),
            (ColumnType::I32, false, "int"),
            (ColumnType::U32, false, "int"),
            (ColumnType::F64, false, "float"),
            (ColumnType::Bool, false, "bool"),
            (ColumnType::Timestamp, false, "datetime"),
            (ColumnType::Json, false, "object"),
            (ColumnType::Bytes, false, "bytes"),
            (ColumnType::Custom("UNKNOWN"), false, "string"),
            (ColumnType::Custom("geometry(point)"), false, "geometry(point)"),
            (ColumnType::Uuid, true, "option<string>"),
        ];
        for (typ, nullable, expected_inner) in cases {
            let rendered = render_column_type(&typ, nullable);
            assert!(
                rendered.contains(expected_inner),
                "ColumnType::{typ:?} (nullable={nullable}) did not contain {expected_inner:?}: {rendered}"
            );
        }
    }
}
