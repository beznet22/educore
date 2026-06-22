//! # Audit log cross-tenant isolation (Phase 16)
//!
//! Asserts that an audit row written for `school_a` is NOT
//! visible to `read_for_target` when the caller passes
//! `school_b`, even if the `target_id` matches. The contract
//! is documented in `docs/ports/storage.md` § 5 and
//! `docs/schemas/tenancy-schema.md` § 4.
//!
//! Backends:
//! - **testkit** — always-on, in-memory.
//! - **sqlite** — always-on, in-memory.
//! - **surrealdb** — always-on, in-memory.
//! - **postgres** — env-gated on `EDUCORE_PG_URL`.
//! - **mysql** — env-gated on `EDUCORE_MYSQL_URL`.

#![cfg(test)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::dbg_macro,
    missing_docs
)]

mod common;

use educore_core::clock::{IdGenerator, SystemIdGen};
use educore_storage::audit::AuditLogEntry;
use educore_storage::StorageAdapter;

/// Exercises the cross-tenant isolation guarantee:
/// 1. School A writes an audit row for target T.
/// 2. School A's `read_for_target(school_a, T, …)` returns the
///    row.
/// 3. School B's `read_for_target(school_b, T, …)` returns
///    zero rows (the row is invisible across tenants).
async fn assert_cross_tenant_isolation(adapter: &dyn StorageAdapter) {
    let g = SystemIdGen;
    let school_a = g.next_school_id();
    let school_b = g.next_school_id();
    let target = g.next_uuid();
    let actor = g.next_user_id();

    // Write one audit row for school_a on target T.
    let entry = AuditLogEntry::create(
        school_a,
        actor,
        "school",
        target,
        bytes::Bytes::from_static(br#"{"id":"x"}"#),
        g.next_correlation_id(),
    );
    let tx = adapter.begin().await.expect("begin");
    tx.audit_log().append(entry).await.expect("append");
    tx.commit().await.expect("commit");

    // Same school, same target → row visible.
    let tx = adapter.begin().await.expect("begin");
    let rows_a = tx
        .audit_log()
        .read_for_target(school_a, target, 10)
        .await
        .expect("read school_a");
    tx.commit().await.expect("commit read_a");
    assert_eq!(
        rows_a.len(),
        1,
        "school_a must see its own audit row for target {target} (got {} rows)",
        rows_a.len()
    );
    assert_eq!(rows_a[0].school_id, school_a);

    // Different school, same target → row invisible.
    let tx = adapter.begin().await.expect("begin");
    let rows_b = tx
        .audit_log()
        .read_for_target(school_b, target, 10)
        .await
        .expect("read school_b");
    tx.commit().await.expect("commit read_b");
    assert_eq!(
        rows_b.len(),
        0,
        "school_b must NOT see school_a's audit row for target {target} (got {} rows)",
        rows_b.len()
    );

    // Sanity: school_b can write its own audit row on the same
    // target_id without disturbing school_a's row. This is the
    // (school, target) composite-key independence guarantee.
    let entry_b = AuditLogEntry::create(
        school_b,
        actor,
        "school",
        target,
        bytes::Bytes::from_static(br#"{"id":"y"}"#),
        g.next_correlation_id(),
    );
    let tx = adapter.begin().await.expect("begin");
    tx.audit_log().append(entry_b).await.expect("append_b");
    tx.commit().await.expect("commit_b");

    let tx = adapter.begin().await.expect("begin");
    let rows_a_after = tx
        .audit_log()
        .read_for_target(school_a, target, 10)
        .await
        .expect("read school_a after");
    let rows_b_after = tx
        .audit_log()
        .read_for_target(school_b, target, 10)
        .await
        .expect("read school_b after");
    tx.commit().await.expect("commit read_after");
    assert_eq!(
        rows_a_after.len(),
        1,
        "school_a must still see only its own row (got {})",
        rows_a_after.len()
    );
    assert_eq!(rows_a_after[0].school_id, school_a);
    assert_eq!(
        rows_b_after.len(),
        1,
        "school_b must see its own row (got {})",
        rows_b_after.len()
    );
    assert_eq!(rows_b_after[0].school_id, school_b);
}

// ---------------------------------------------------------------------------
// Always-on backends
// ---------------------------------------------------------------------------

#[tokio::test]
async fn audit_cross_tenant_isolation_testkit() {
    let (adapter, _school, _ctx) = common::setup_testkit();
    adapter.migrate().await.expect("migrate testkit");
    assert_cross_tenant_isolation(&*adapter).await;
}

#[tokio::test]
async fn audit_cross_tenant_isolation_sqlite() {
    let (adapter, _school, _ctx) = common::setup_sqlite().await;
    assert_cross_tenant_isolation(&*adapter).await;
}

#[tokio::test]
async fn audit_cross_tenant_isolation_surrealdb() {
    let (adapter, _school, _ctx) = common::setup_surrealdb().await;
    assert_cross_tenant_isolation(&*adapter).await;
}

// ---------------------------------------------------------------------------
// Env-gated backends
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: cargo test -- --ignored"]
async fn audit_cross_tenant_isolation_postgres() {
    let Some((adapter, _school, _ctx)) = common::setup_pg().await else {
        return;
    };
    assert_cross_tenant_isolation(&*adapter).await;
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: cargo test -- --ignored"]
async fn audit_cross_tenant_isolation_mysql() {
    let Some((adapter, _school, _ctx)) = common::setup_mysql().await else {
        return;
    };
    assert_cross_tenant_isolation(&*adapter).await;
}
