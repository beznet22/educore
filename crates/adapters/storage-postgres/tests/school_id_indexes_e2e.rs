//! QW-6 end-to-end test: per-tenant `school_id` indexes are
//! created by `migrate()` against a live PostgreSQL instance.
//!
//! Closes the audit finding ADAPTER-PG-013-equivalent (the PG
//! counterpart of `ADAPT-MY-008` in
//! `docs/audit_reports/findings/wave3-storage-mysql.md`): the
//! cross-cutting tables now have a dedicated single-column
//! `school_id` index for the 4 multi-tenant tables
//! (`outbox`, `audit_log`, `idempotency`, `event_log`).
//!
//! The test is gated on the `EDUCORE_PG_URL` environment
//! variable, matching the pattern in `outbox_e2e.rs` and
//! `idempotency_e2e.rs`: when the variable is unset (the
//! default in CI), the test logs a skip notice via `tracing`
//! and returns early (passing). When the variable is set
//! (e.g. a contributor with a local PostgreSQL instance), the
//! test runs the full migration and queries `pg_indexes` to
//! assert that every expected index was created on the
//! expected table.
//!
//! To run locally:
//!
//! ```text
//! docker run --rm -d --name educore-pg -p 5432:5432 \
//!     -e POSTGRES_USER=educore -e POSTGRES_PASSWORD=educore \
//!     -e POSTGRES_DB=educore postgres:16
//! export EDUCORE_PG_URL=postgres://educore:educore@localhost:5432/educore
//! cargo test -p educore-storage-postgres --test school_id_indexes_e2e -- --nocapture
//! ```

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    clippy::print_stderr,
    missing_docs
)]

use educore_core::clock::{IdGenerator as _, SystemIdGen};
use educore_storage::StorageAdapter;
use educore_storage_postgres::ddl::EXPECTED_INDEXES;

/// Skip-with-notice helper. Returns `Some(())` when the
/// `EDUCORE_PG_URL` env var is set, `None` otherwise. The
/// notice is routed through `tracing` so it shows up under
/// the test harness's normal logging.
fn pg_url_or_skip(test_name: &str) -> Option<String> {
    match std::env::var("EDUCORE_PG_URL") {
        Ok(u) => Some(u),
        Err(_) => {
            tracing::info!(
                test = test_name,
                "EDUCORE_PG_URL not set; skipping PG school_id indexes e2e",
            );
            None
        }
    }
}

/// QW-6 test: after `migrate()`, every expected
/// `<table>_school_id_idx` index must exist on the
/// `<table>` (engine schema) — verified via `pg_indexes`.
///
/// This is the live-DB counterpart to the string-level unit
/// tests in `src/ddl.rs`. The unit tests guard against
/// typos in the SQL string; this test guards against the
/// string being valid but the indexes being silently dropped
/// (e.g. by a sqlx quoting bug or a future change that
/// short-circuits the second `raw_sql` call).
#[tokio::test]
async fn migrate_creates_all_school_id_indexes() {
    let Some(url) = pg_url_or_skip("migrate_creates_all_school_id_indexes") else {
        return;
    };

    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();

    // Run migrate twice to prove the second invocation is a
    // no-op (the IF NOT EXISTS clauses make every statement
    // idempotent). The MigrationReport should also report the
    // expected SCHEMA_VERSION = 2.
    let report_first = adapter.migrate().await.unwrap();
    assert_eq!(
        report_first.version, 2,
        "SCHEMA_VERSION must be bumped to 2 after QW-6 \
         (school_id indexes are a new migration step)",
    );
    let report_second = adapter.migrate().await.unwrap();
    assert_eq!(
        report_first.statements_executed, report_second.statements_executed,
        "migrate() must be idempotent — running it twice must \
         execute the same number of statements (the second run \
         hits the IF NOT EXISTS short-circuit on every CREATE)",
    );

    // Query pg_indexes for every expected index name. The
    // engine schema is the canonical home for the 6
    // cross-cutting tables per the canonical DDL, so we
    // filter by schemaname = 'engine'.
    let pool = adapter.db().clone();
    for (index_name, table_name) in EXPECTED_INDEXES {
        // table_name is "engine.outbox"; split into schema +
        // relname for the pg_indexes query.
        let (schema, relname) = table_name.split_once('.').unwrap_or(("public", table_name));
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT schemaname, indexname FROM pg_indexes \
             WHERE schemaname = $1 AND tablename = $2 \
               AND indexname = $3",
        )
        .bind(schema)
        .bind(relname)
        .bind(index_name)
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            panic!("pg_indexes query failed for {schema}.{relname}.{index_name}: {e}",)
        });

        assert!(
            !rows.is_empty(),
            "expected pg_indexes to contain an entry for \
             {schema}.{relname}.{index_name}, got 0 rows",
        );
        assert_eq!(
            rows.len(),
            1,
            "expected exactly 1 pg_indexes row for \
             {schema}.{relname}.{index_name}, got {} \
             (rows={rows:?})",
            rows.len(),
        );
        let (got_schema, got_index) = &rows[0];
        assert_eq!(got_schema, schema);
        assert_eq!(got_index, index_name);
    }
}

/// QW-6 test (idempotency leg): running `migrate()` against a
/// fully-migrated database must not error. The IF NOT EXISTS
/// clauses should turn every statement into a no-op without
/// raising. This is the runtime counterpart to the
/// `sql_has_no_duplicate_index_declarations` unit test in
/// `src/ddl.rs`.
#[tokio::test]
async fn migrate_is_idempotent_against_fully_migrated_db() {
    let Some(url) = pg_url_or_skip("migrate_is_idempotent_against_fully_migrated_db") else {
        return;
    };

    let g = SystemIdGen;
    let school = g.next_school_id();
    let adapter = educore_storage_postgres::PostgresStorageAdapter::connect(&url, school)
        .await
        .unwrap();

    // Three back-to-back runs must all succeed.
    for i in 0..3 {
        adapter
            .migrate()
            .await
            .unwrap_or_else(|e| panic!("migrate() run #{i} failed: {e}"));
    }
}
