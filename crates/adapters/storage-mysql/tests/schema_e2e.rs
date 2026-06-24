//! `create_schema` end-to-end test against the MySQL adapter.
//!
//! The test is gated on the `EDUCORE_MYSQL_URL` environment
//! variable. When the variable is unset (the default in CI), the
//! test logs a skip notice via `tracing` and returns early
//! (passing). When the variable is set (e.g. a contributor with
//! a local MySQL instance), the test runs the full round-trip
//! and asserts on the engine invariants.
//!
//! To run locally:
//!
//! ```text
//! docker run --rm -d --name educore-mysql -p 3306:3306 \
//!     -e MYSQL_ROOT_PASSWORD=educore -e MYSQL_DATABASE=educore \
//!     -e MYSQL_USER=educore -e MYSQL_PASSWORD=educore \
//!     mysql:8
//! export EDUCORE_MYSQL_URL='mysql://educore:educore@localhost:3306/educore'
//! cargo test -p educore-storage-mysql --test schema_e2e -- --nocapture
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

/// Test 3 of the cluster-A stage 3 PR: `create_schema` is
/// idempotent. We run `create_schema` twice against the same
/// MySQL instance and verify both calls succeed (a second run
/// on an already-bootstrapped database must be a no-op thanks
/// to the `IF NOT EXISTS` clauses and the
/// `SET FOREIGN_KEY_CHECKS=0/1` wrapper).
#[tokio::test]
async fn create_schema_is_idempotent_against_live_mysql() {
    let url = match std::env::var("EDUCORE_MYSQL_URL") {
        Ok(u) => u,
        Err(_) => {
            tracing::info!("EDUCORE_MYSQL_URL not set; skipping MySQL create_schema e2e");
            return;
        }
    };
    let g = SystemIdGen;
    let school = g.next_school_id();

    // Two adapters against the same MySQL instance — one per
    // `create_schema` call — each opens its own pool and runs
    // the DDL. The test exercises the cross-cutting engine
    // tables; with an empty global registry no aggregate
    // DDL is emitted, so this verifies that the 6 cross-cutting
    // `CREATE TABLE IF NOT EXISTS` statements are themselves
    // idempotent.

    // First run: bootstraps the 6 cross-cutting tables.
    let adapter_a =
        educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
            .await
            .expect("connect adapter_a");
    adapter_a
        .create_schema()
        .await
        .expect("first create_schema");

    // Second run: same database. The IF NOT EXISTS clauses
    // and the FOREIGN_KEY_CHECKS=0/1 wrapper must make this a
    // no-op (every statement succeeds).
    let adapter_b =
        educore_storage_mysql::MysqlStorageAdapter::connect(&url, school)
            .await
            .expect("connect adapter_b");
    adapter_b
        .create_schema()
        .await
        .expect("second create_schema must be idempotent");
}
