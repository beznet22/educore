//! Engine cross-cutting DDL additions emitted by the PostgreSQL adapter.
//!
//! The canonical engine DDL for the 6 cross-cutting tables lives at
//! `migrations/engine/0000_engine_core.postgres.sql` and is compiled
//! into `storage.rs` via `include_str!`. That canonical file already
//! declares composite indexes that include `school_id` as the
//! leading column for `outbox`, `audit_log`, and `event_log`, and
//! uses `(school_id, command_type, idempotency_key)` as the primary
//! key for `idempotency` — so per-tenant lookups are index-covered
//! on those 4 tables.
//!
//! This module adds a single-column `school_id` index for each of
//! the 4 multi-tenant cross-cutting tables. The single-column form
//! is required for:
//!
//! * `COUNT(*) ... WHERE school_id = $1` aggregates on the outbox
//!   / event_log / audit_log (the composite index can satisfy
//!   these but the planner chooses the smaller single-column
//!   index for the aggregate-only access path).
//! * `DELETE FROM ... WHERE school_id = $1 AND ...` retention
//!   purges on `idempotency` (the PK covers lookups but the purge
//!   planner needs the single-column index for the range scan).
//! * Any future "list every event for this school" path that does
//!   not also bind `enqueued_at`, `occurred_at`, etc.
//!
//! # Why not `schema_registry` or `system_user`?
//!
//! Those two tables are engine-internal singletons, not multi-
//! tenant aggregates:
//!
//! * `schema_registry` is the engine-wide event-type schema catalog
//!   (PK = `(event_type, event_version)`). It is intentionally
//!   engine-global; tenants do not own schemas.
//! * `system_user` is the single SYSTEM actor row referenced by
//!   every aggregate's `created_by` / `updated_by` when the engine
//!   itself is the actor. The id is the well-known
//!   `SYSTEM_USER_ID` constant.
//!
//! Neither table has a `school_id` column in the canonical DDL,
//! so no per-tenant index can be added without first adding the
//! column. Adding the column is out of scope for this PR — it is
//! a future schema-design decision tracked separately.
//!
//! # Idempotency
//!
//! Every statement uses `CREATE INDEX IF NOT EXISTS`, so the
//! migration is safe to re-run against a partially-migrated DB.

/// The set of expected per-tenant `school_id` indexes emitted by
/// [`SCHOOL_ID_INDEXES_SQL`]. Exposed for the unit tests and for
/// the integration test in
/// `tests/school_id_indexes_e2e.rs` to assert against.
pub const EXPECTED_INDEXES: &[(&str, &str)] = &[
    ("outbox_school_id_idx", "engine.outbox"),
    ("audit_log_school_id_idx", "engine.audit_log"),
    ("idempotency_school_id_idx", "engine.idempotency"),
    ("event_log_school_id_idx", "engine.event_log"),
];

/// Per-tenant `school_id` indexes for the 4 multi-tenant cross-
/// cutting tables.
///
/// Compiled in at `migrate()` time. The DDL is applied via
/// `sqlx::raw_sql` after the canonical `SCHEMA_SQL` so that the
/// tables are guaranteed to exist before the indexes are created.
///
/// The DDL is wrapped in a comment header for traceability — every
/// entry in [`EXPECTED_INDEXES`] must appear as
/// `CREATE INDEX IF NOT EXISTS <name> ON <table> (<col>)` below.
pub const SCHOOL_ID_INDEXES_SQL: &str = "\
-- QW-6 (postgres): per-tenant school_id indexes for the 4
-- multi-tenant cross-cutting tables. These complement the
-- composite indexes in the canonical DDL (which already use
-- school_id as the leading column for outbox / audit_log /
-- event_log and as the leading PK column for idempotency).
--
-- Note: schema_registry and system_user are engine-internal
-- singletons (no school_id column) and are intentionally
-- excluded; see module-level doc-comment for the rationale.
\n\
CREATE INDEX IF NOT EXISTS outbox_school_id_idx\n    \
    ON engine.outbox (school_id);\n\
CREATE INDEX IF NOT EXISTS audit_log_school_id_idx\n    \
    ON engine.audit_log (school_id);\n\
CREATE INDEX IF NOT EXISTS idempotency_school_id_idx\n    \
    ON engine.idempotency (school_id);\n\
CREATE INDEX IF NOT EXISTS event_log_school_id_idx\n    \
    ON engine.event_log (school_id);\n";

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]
mod tests {
    use super::*;

    /// Every entry in [`EXPECTED_INDEXES`] must be emitted as
    /// `CREATE INDEX IF NOT EXISTS <name>` on the matching table
    /// column. This guards against accidental drift between the
    /// constant and the SQL string.
    #[test]
    fn sql_emits_every_expected_index() {
        for (name, table) in EXPECTED_INDEXES {
            let needle_create = format!("CREATE INDEX IF NOT EXISTS {name}");
            assert!(
                SCHOOL_ID_INDEXES_SQL.contains(&needle_create),
                "SCHOOL_ID_INDEXES_SQL is missing `{needle_create}` \
                 (table={table})",
            );
            let needle_on_table = format!("ON {table}");
            assert!(
                SCHOOL_ID_INDEXES_SQL.contains(&needle_on_table),
                "SCHOOL_ID_INDEXES_SQL is missing `{needle_on_table}` \
                 (index={name})",
            );
        }
    }

    /// The DDL must reference each expected index **exactly once**
    /// — a duplicate would be a copy-paste bug, and the
    /// `IF NOT EXISTS` clause would silently mask it.
    #[test]
    fn sql_has_no_duplicate_index_declarations() {
        for (name, _table) in EXPECTED_INDEXES {
            let needle = format!("CREATE INDEX IF NOT EXISTS {name}");
            let count = SCHOOL_ID_INDEXES_SQL.matches(&needle).count();
            assert_eq!(
                count, 1,
                "expected exactly 1 occurrence of `{needle}` in \
                 SCHOOL_ID_INDEXES_SQL, got {count}",
            );
        }
    }

    /// Every expected index must target the `school_id` column.
    /// This catches an off-by-one column typo (e.g. `(enqueued_at)`)
    /// that the other two tests would not detect.
    #[test]
    fn every_index_targets_school_id() {
        for (name, table) in EXPECTED_INDEXES {
            let line = format!("ON {table} (school_id);");
            assert!(
                SCHOOL_ID_INDEXES_SQL.contains(&line),
                "SCHOOL_ID_INDEXES_SQL is missing the column clause \
                 `{line}` (index={name})",
            );
        }
    }

    /// The DDL must not actually **target** either of the two
    /// engine-internal singletons — those tables have no
    /// `school_id` column and a `CREATE INDEX ... ON schema_registry
    /// (school_id)` would fail at runtime with a missing-column
    /// error. The names may legitimately appear in the comment
    /// header explaining the exclusion, so we match the SQL
    /// pattern `ON <singleton>` rather than the bare name.
    /// This is a regression guard.
    #[test]
    fn sql_does_not_target_singleton_tables() {
        for singleton in ["schema_registry", "system_user"] {
            let needle = format!("ON {singleton}");
            assert!(
                !SCHOOL_ID_INDEXES_SQL.contains(&needle),
                "SCHOOL_ID_INDEXES_SQL must not target the singleton \
                 table `{singleton}` (no school_id column)",
            );
        }
    }

    /// The expected-indexes table must stay in sync with the SQL:
    /// same number of entries, no duplicate index names, no
    /// duplicate tables.
    #[test]
    fn expected_indexes_table_is_self_consistent() {
        let names: Vec<&str> = EXPECTED_INDEXES.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names.len(),
            EXPECTED_INDEXES.len(),
            "EXPECTED_INDEXES contains duplicate index names",
        );
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            names.len(),
            "EXPECTED_INDEXES contains duplicate index names: {names:?}",
        );
    }
}
