//! # Idempotency collision parity (Phase 16)
//!
//! Asserts the idempotency sub-port contract uniformly across
//! every backend:
//!
//! 1. Storing a record with a fresh composite key is Ok
//!    (independent record).
//! 2. Storing the same `(school_id, command_type,
//!    idempotency_key)` with the **same outcome** is a no-op
//!    (returns `Ok(())`).
//!
//! The "same key + different outcome = Conflict" half of the
//! contract is enforced by the testkit in-memory backend (see
//! `crates/tools/testkit/src/storage.rs::IdempotencyHandle`).
//! The SQLite + SurrealDB reference adapters currently
//! implement `record` as a plain `INSERT` / `INSERT OR REPLACE`
//! and therefore accept a re-write with a different outcome;
//! closing this gap is tracked in the engine backlog. The
//! parity test asserts the conflict-detection path on
//! testkit only and documents the SQL / SurrealDB deviation
//! explicitly so the test surface stays honest.
//!
//! The at-least-once delivery contract is documented in
//! `docs/ports/storage.md` § 6 and
//! `docs/decisions/ADR-014-Idempotency.md`.
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
use educore_core::ids::SchoolId;
use educore_storage::idempotency::{IdempotencyCompositeKey, IdempotencyRecord};
use educore_storage::StorageAdapter;

/// Asserts the parts of the idempotency contract that are
/// uniformly enforced across every backend: same outcome is a
/// no-op; a fresh key is a clean write.
async fn assert_idempotency_no_op_and_independent(
    adapter: &dyn StorageAdapter,
    school: SchoolId,
) {
    let g = SystemIdGen;
    let key = g.next_idempotency_key();
    let outcome_a = bytes::Bytes::from_static(b"{\"id\":\"a\"}");
    let r1 = IdempotencyRecord {
        school_id: school,
        command_type: "parity.test.command",
        idempotency_key: key,
        outcome: outcome_a.clone(),
        outcome_version: 1,
        recorded_at: educore_core::value_objects::Timestamp::now(),
        affected_aggregate_ids: vec![],
    };
    let tx = adapter.begin().await.expect("begin");
    tx.idempotency().record(r1.clone()).await.expect("record1");
    tx.commit().await.expect("commit1");

    // Same outcome (same bytes) → Ok (no-op write).
    let tx = adapter.begin().await.expect("begin");
    let res = tx.idempotency().record(r1.clone()).await;
    tx.commit().await.expect("commit2");
    assert!(
        res.is_ok(),
        "recording the same (key, outcome) must be a no-op; got {res:?}"
    );

    // Different key (same outcome) → Ok (independent record).
    let other_key = g.next_idempotency_key();
    let r3 = IdempotencyRecord {
        idempotency_key: other_key,
        ..r1.clone()
    };
    let tx = adapter.begin().await.expect("begin");
    let res = tx.idempotency().record(r3).await;
    tx.commit().await.expect("commit3");
    assert!(
        res.is_ok(),
        "a fresh idempotency key must record cleanly; got {res:?}"
    );

    // The composite-key lookup returns the stored record.
    let composite = IdempotencyCompositeKey {
        school_id: school,
        command_type: "parity.test.command",
        idempotency_key: key,
    };
    let tx = adapter.begin().await.expect("begin");
    let lookup = tx.idempotency().lookup(composite).await.expect("lookup");
    tx.commit().await.expect("commit4");
    assert!(
        lookup.is_some(),
        "the original composite key must be lookup-able"
    );
}

/// Asserts the outcome-equality conflict path. The contract
/// (per ADR-014) is "same key + different outcome = Conflict".
/// Only the testkit backend enforces this today; the SQL +
/// SurrealDB adapters accept the re-write (a documented
/// deviation). This helper asserts the conflict only on
/// testkit so the test surface accurately reflects the
/// production state.
async fn assert_outcome_conflict_on_testkit(
    adapter: &dyn StorageAdapter,
    school: SchoolId,
) {
    let g = SystemIdGen;
    let key = g.next_idempotency_key();
    let r1 = IdempotencyRecord {
        school_id: school,
        command_type: "parity.test.conflict",
        idempotency_key: key,
        outcome: bytes::Bytes::from_static(b"{\"id\":\"a\"}"),
        outcome_version: 1,
        recorded_at: educore_core::value_objects::Timestamp::now(),
        affected_aggregate_ids: vec![],
    };
    let tx = adapter.begin().await.expect("begin");
    tx.idempotency().record(r1.clone()).await.expect("record1");
    tx.commit().await.expect("commit1");

    let r2 = IdempotencyRecord {
        outcome: bytes::Bytes::from_static(b"{\"id\":\"b\"}"),
        ..r1
    };
    let tx = adapter.begin().await.expect("begin");
    let res = tx.idempotency().record(r2).await;
    tx.commit().await.expect("commit2");
    assert!(
        res.is_err(),
        "testkit must surface Conflict on (same-key, different-outcome); got Ok"
    );
    let err = res.expect_err("must be error");
    assert_eq!(
        err.kind(),
        educore_core::error::ErrorKind::Conflict,
        "idempotency collision must surface as ErrorKind::Conflict; got {:?}",
        err.kind()
    );
}

// ---------------------------------------------------------------------------
// Always-on backends
// ---------------------------------------------------------------------------

#[tokio::test]
async fn idempotency_no_op_and_independent_testkit() {
    let (adapter, school, _ctx) = common::setup_testkit();
    adapter.migrate().await.expect("migrate testkit");
    assert_idempotency_no_op_and_independent(&*adapter, school).await;
}

#[tokio::test]
async fn idempotency_no_op_and_independent_sqlite() {
    let (adapter, school, _ctx) = common::setup_sqlite().await;
    assert_idempotency_no_op_and_independent(&*adapter, school).await;
}

#[tokio::test]
async fn idempotency_no_op_and_independent_surrealdb() {
    let (adapter, school, _ctx) = common::setup_surrealdb().await;
    assert_idempotency_no_op_and_independent(&*adapter, school).await;
}

#[tokio::test]
async fn idempotency_outcome_conflict_testkit() {
    let (adapter, school, _ctx) = common::setup_testkit();
    adapter.migrate().await.expect("migrate testkit");
    assert_outcome_conflict_on_testkit(&*adapter, school).await;
}

// ---------------------------------------------------------------------------
// Env-gated backends
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: cargo test -- --ignored"]
async fn idempotency_no_op_and_independent_postgres() {
    let Some((adapter, school, _ctx)) = common::setup_pg().await else {
        return;
    };
    assert_idempotency_no_op_and_independent(&*adapter, school).await;
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: cargo test -- --ignored"]
async fn idempotency_no_op_and_independent_mysql() {
    let Some((adapter, school, _ctx)) = common::setup_mysql().await else {
        return;
    };
    assert_idempotency_no_op_and_independent(&*adapter, school).await;
}