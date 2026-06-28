//! # Transaction commit / rollback parity (Phase 16)
//!
//! Asserts the transaction lifecycle contract uniformly across
//! every backend:
//!
//! 1. A committed transaction does not error, and the staged
//!    writes are durable in the same school (a subsequent
//!    `begin` sees them).
//! 2. A rolled-back transaction does not error.
//!
//! **Known limitation:** every storage adapter shipped in
//! Phase 1 (testkit, SQLite, SurrealDB, PostgreSQL, MySQL)
//! currently implements `Transaction::commit` and
//! `Transaction::rollback` as flag-only operations. The
//! sub-port writes are auto-committed at the query boundary
//! (SQLite/SurrealDB) or live in shared state without
//! per-transaction isolation (testkit). A rolled-back
//! transaction therefore MAY leave its writes visible to a
//! subsequent transaction. The parity test asserts the
//! non-error contract today and flags the atomicity gap so
//! the test surface stays honest about the engine's current
//! state. Closing this gap is tracked in the engine
//! backlog.
//!
//! The contract (once the gap is closed) is documented in
//! `docs/ports/storage.md` § 3.
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
use educore_storage::audit::AuditLogEntry;
use educore_storage::event_log::{EventLogEntry, EventLogFilter};
use educore_storage::outbox::SerializedEnvelope;
use educore_storage::StorageAdapter;

/// Builds a fresh outbox envelope, audit row, and event log
/// row for the same school + aggregate. Centralised so the
/// per-backend tests do not drift.
fn fresh_writes(school: SchoolId) -> (SerializedEnvelope, AuditLogEntry, EventLogEntry) {
    let g = SystemIdGen;
    let actor = g.next_user_id();
    let corr = g.next_correlation_id();
    let agg = g.next_uuid();
    let env = SerializedEnvelope {
        event_id: g.next_event_id(),
        event_type: "parity.test.committed".to_owned(),
        schema_version: 1,
        school_id: school,
        aggregate_id: agg,
        aggregate_type: "parity".to_owned(),
        actor_id: actor,
        correlation_id: corr,
        causation_id: None,
        occurred_at: educore_core::value_objects::Timestamp::now(),
        payload: bytes::Bytes::from_static(br#"{"id":"x"}"#),
    };
    let audit = AuditLogEntry::create(
        school,
        actor,
        "parity",
        agg,
        bytes::Bytes::from_static(br#"{"id":"x"}"#),
        corr,
    );
    let event = EventLogEntry {
        event_id: env.event_id,
        school_id: school,
        event_type: env.event_type.clone(),
        schema_version: env.schema_version,
        aggregate_id: env.aggregate_id,
        aggregate_type: env.aggregate_type.clone(),
        actor_id: env.actor_id,
        correlation_id: env.correlation_id,
        causation_id: None,
        occurred_at: env.occurred_at,
        recorded_at: educore_core::value_objects::Timestamp::now(),
        payload: env.payload.clone(),
        active_status: educore_core::value_objects::ActiveStatus::Active,
    };
    (env, audit, event)
}

async fn assert_commit_and_rollback_lifecycle(adapter: &dyn StorageAdapter, school: SchoolId) {
    // ---- 1. Commit path --------------------------------------------------
    let (env_a, audit_a, event_a) = fresh_writes(school);
    {
        let tx = adapter.begin().await.expect("begin-commit");
        tx.outbox()
            .append(school, env_a.clone())
            .await
            .expect("append env");
        tx.audit_log()
            .append(audit_a.clone())
            .await
            .expect("append audit");
        tx.event_log()
            .append(event_a.clone())
            .await
            .expect("append event");
        tx.commit().await.expect("commit must not error");
    }

    // After commit, a fresh transaction sees the audit row
    // and the event log row. (The outbox is consumed by the
    // relay on the testkit backend; the engine's invariant is
    // that audit_log + event_log are durable.)
    {
        let tx = adapter.begin().await.expect("begin-read-after-commit");
        let audit = tx
            .audit_log()
            .read_for_target(school, env_a.aggregate_id, 10)
            .await
            .expect("read audit");
        let events = tx
            .event_log()
            .read(EventLogFilter::for_school(school))
            .await
            .expect("read events");
        tx.commit().await.expect("commit read");
        assert_eq!(
            audit.len(),
            1,
            "committed audit row must be visible after commit (got {})",
            audit.len()
        );
        assert!(
            events.iter().any(|e| e.event_id == event_a.event_id),
            "committed event log row must be visible after commit (event_id={})",
            event_a.event_id
        );
    }

    // ---- 2. Rollback path: must not error -------------------------------
    let (_env_b, _audit_b, _event_b) = fresh_writes(school);
    let tx = adapter.begin().await.expect("begin-rollback");
    tx.outbox()
        .append(
            school,
            SerializedEnvelope {
                event_type: "parity.test.rolled_back".to_owned(),
                .._env_b.clone()
            },
        )
        .await
        .expect("append env rollback");
    tx.audit_log()
        .append(_audit_b.clone())
        .await
        .expect("append audit rollback");
    tx.event_log()
        .append(EventLogEntry {
            event_type: "parity.test.rolled_back".to_owned(),
            .._event_b.clone()
        })
        .await
        .expect("append event rollback");
    tx.rollback()
        .await
        .expect("rollback must not error (atomicity is a separate gate)");
}

// ---------------------------------------------------------------------------
// Always-on backends
// ---------------------------------------------------------------------------

#[tokio::test]
async fn transaction_commit_rollback_testkit() {
    let (adapter, school, _ctx) = common::setup_testkit();
    adapter.migrate().await.expect("migrate testkit");
    assert_commit_and_rollback_lifecycle(&*adapter, school).await;
}

#[tokio::test]
async fn transaction_commit_rollback_sqlite() {
    let (adapter, school, _ctx) = common::setup_sqlite().await;
    assert_commit_and_rollback_lifecycle(&*adapter, school).await;
}

#[tokio::test]
async fn transaction_commit_rollback_surrealdb() {
    let (adapter, school, _ctx) = common::setup_surrealdb().await;
    assert_commit_and_rollback_lifecycle(&*adapter, school).await;
}

// ---------------------------------------------------------------------------
// Env-gated backends
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires EDUCORE_PG_URL; run with: cargo test -- --ignored"]
async fn transaction_commit_rollback_postgres() {
    let Some((adapter, school, _ctx)) = common::setup_pg().await else {
        return;
    };
    assert_commit_and_rollback_lifecycle(&*adapter, school).await;
}

#[tokio::test]
#[ignore = "requires EDUCORE_MYSQL_URL; run with: cargo test -- --ignored"]
async fn transaction_commit_rollback_mysql() {
    let Some((adapter, school, _ctx)) = common::setup_mysql().await else {
        return;
    };
    assert_commit_and_rollback_lifecycle(&*adapter, school).await;
}
